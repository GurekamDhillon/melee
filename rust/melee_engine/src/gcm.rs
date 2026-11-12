//! A GameCube disc image (GCM/ISO): the FST at header +0x424, files by entry number (the FST index, which is
//! what `DVDConvertPathToEntrynum` returns). Local only: nothing read here is written anywhere or committed.

use std::io::{Read, Seek, SeekFrom};

/// Answers a disc path with the bytes of a file that replaces it (gdmods mounted over the disc), or None.
pub type Overlay = Box<dyn Fn(&str) -> Option<Vec<u8>>>;

pub struct Gcm {
    f: Option<std::fs::File>,
    fst: Vec<u8>,
    /// Mounted mods: a path they answer resolves to the same entry number, whose bytes are replaced.
    overlay: Option<Overlay>,
    replaced: std::cell::RefCell<std::collections::HashMap<i32, std::sync::Arc<[u8]>>>,
    /// Paths only the mods have (a new fighter's PlBm.dat): an entry number past the FST each, by lowercased path.
    added: std::cell::RefCell<std::collections::HashMap<String, i32>>,
    /// The image's path (the asset service opens its own handle on it; None: no file, or core v1's host reader).
    pub path: Option<String>,
}


fn be32(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}

impl Gcm {
    pub fn open(path: &str) -> Result<Self, String> {
        #[cfg(feature = "corev1")]
        if HOST_READ.get().is_some() {
            // core v1: the host owns the disc; every read goes through its disc_read (the path is not opened)
            let h = read_at(None, 0x424, 8)?;
            let fst = read_at(None, be32(&h, 0) as u64, be32(&h, 4) as usize)?;
            return Ok(Self { f: None, fst, overlay: None, replaced: Default::default(), added: Default::default(), path: None });
        }
        let mut f = std::fs::File::open(path).map_err(|e| format!("{path}: {e}"))?;
        let h = read_at(Some(&mut f), 0x424, 8)?;
        let fst = read_at(Some(&mut f), be32(&h, 0) as u64, be32(&h, 4) as usize)?;
        Ok(Self { f: Some(f), fst, overlay: None, replaced: Default::default(), added: Default::default(), path: Some(path.to_string()) })
    }

    /// A disc with only the root directory: for headless runs that load no files.
    pub fn empty() -> Self {
        let mut fst = vec![0u8; 12];
        fst[0] = 1;
        fst[11] = 1;
        Self { f: None, fst, overlay: None, replaced: Default::default(), added: Default::default(), path: None }
    }

    /// Mount `f` over the disc: every file it answers replaces the disc's file of that path (same entry number).
    pub fn set_overlay(&mut self, f: Overlay) {
        self.overlay = Some(f);
    }

    /// Drop the cached replacement of disc file `path` (a file of the disc itself, not one only the overlay has), so
    /// the next open asks the overlay again: a cosmetic choice changed between matches (offline; never under a
    /// rollback session, whose snapshots would restore the old cache).
    pub fn forget_replaced(&self, path: &str) {
        let n = self.entry_for_path_disc(path);
        if n >= 0 {
            self.replaced.borrow_mut().remove(&n);
        }
    }

    /// `len` bytes of the game's main.dol at its RAM address `addr` (DOL header at the disc's +0x420 offset).
    pub fn dol_bytes(&mut self, addr: u32, len: usize) -> Result<Vec<u8>, String> {
        let h = read_at(self.f.as_mut(), 0x420, 4)?;
        let dol = be32(&h, 0) as u64;
        let hd = read_at(self.f.as_mut(), dol, 0x100)?;
        for k in 0..18 {
            let (off, a, sz) = (be32(&hd, k * 4), be32(&hd, 0x48 + k * 4), be32(&hd, 0x90 + k * 4));
            if sz != 0 && a <= addr && addr as u64 + len as u64 <= a as u64 + sz as u64 {
                return read_at(self.f.as_mut(), dol + off as u64 + (addr - a) as u64, len);
            }
        }
        Err(format!("DOL: {addr:#x}+{len:#x} is not in any section"))
    }

    /// Every file on the disc as a `/`-separated path (FST order). Tools only (the layout walker).
    pub fn file_paths(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut stack: Vec<(usize, String)> = Vec::new();
        let n = self.count();
        let mut i = 1;
        while i < n {
            while let Some(&(end, _)) = stack.last() {
                if i >= end { stack.pop(); } else { break; }
            }
            let prefix: String = stack.iter().map(|(_, p)| format!("{p}/")).collect();
            let name = String::from_utf8_lossy(self.name(i)).into_owned();
            if self.is_dir(i) {
                stack.push((be32(&self.fst, i * 12 + 8) as usize, name));
            } else {
                out.push(prefix + &name);
            }
            i += 1;
        }
        out
    }

    fn count(&self) -> usize {
        be32(&self.fst, 8) as usize
    }

    fn name(&self, i: usize) -> &[u8] {
        let names = &self.fst[self.count() * 12..];
        let no = (be32(&self.fst, i * 12) & 0xFF_FFFF) as usize;
        let end = names[no..].iter().position(|&c| c == 0).unwrap_or(0);
        &names[no..no + end]
    }

    /// The game's own name of disc entry `e` (the file name in the disc's table, no folder), for diagnostics; an
    /// entry past the table (an overlay file) or a folder reads as empty.
    pub fn entry_name(&self, e: i32) -> &[u8] {
        match usize::try_from(e) {
            Ok(i) if i >= 1 && i < self.count() && !self.is_dir(i) => self.name(i),
            _ => b"",
        }
    }

    fn is_dir(&self, i: usize) -> bool {
        self.fst[i * 12] != 0
    }

    /// `DVDConvertPathToEntrynum`: an absolute or root-relative path walks the directory tree
    /// (case-insensitive); a bare name that is not at the root falls back to the first file of that name
    /// anywhere (how the port's `Disc::read` has always matched). -1 when there is none.
    pub fn entry_for_path(&self, path: &str) -> i32 {
        let n = self.entry_for_path_disc(path);
        if n >= 0 {
            if let Some(o) = &self.overlay {
                if !self.replaced.borrow().contains_key(&n) {
                    if let Some(b) = o(path) {
                        eprintln!("engine: {path} comes from the mods ({} bytes)", b.len());
                        self.replaced.borrow_mut().insert(n, b.into());
                    }
                }
            }
        } else if let Some(o) = &self.overlay {
            let key = path.trim_start_matches(['/', '\\']).to_ascii_lowercase();
            if let Some(&e) = self.added.borrow().get(&key) {
                return e;
            }
            if let Some(b) = o(path) {
                // past the FST, as the C's shim_dvd numbers them: the preload cache keeps entry numbers as s16
                let e = self.count() as i32 + self.added.borrow().len() as i32;
                eprintln!("engine: {path} is only in the mods ({} bytes, entry {e:#x})", b.len());
                self.added.borrow_mut().insert(key, e);
                self.replaced.borrow_mut().insert(e, b.into());
                return e;
            }
        }
        n
    }

    fn entry_for_path_disc(&self, path: &str) -> i32 {
        let n = self.count();
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        let mut dir = 0usize;
        'walk: for (k, part) in parts.iter().enumerate() {
            let end = be32(&self.fst, dir * 12 + 8) as usize;
            let mut i = dir + 1;
            while i < end.min(n) {
                if self.name(i).eq_ignore_ascii_case(part.as_bytes()) {
                    if k + 1 == parts.len() {
                        return i as i32;
                    }
                    if self.is_dir(i) {
                        dir = i;
                        continue 'walk;
                    }
                    return -1;
                }
                i = if self.is_dir(i) { be32(&self.fst, i * 12 + 8) as usize } else { i + 1 };
            }
            break;
        }
        if parts.len() == 1 {
            for i in 1..n {
                if !self.is_dir(i) && self.name(i).eq_ignore_ascii_case(parts[0].as_bytes()) {
                    return i as i32;
                }
            }
        }
        -1
    }

    /// Length in bytes of file entry `entry` (None for a directory or an out-of-range entry).
    pub fn entry_len(&self, entry: i32) -> Option<u32> {
        if let Some(b) = self.replaced.borrow().get(&entry) {
            return Some(b.len() as u32);
        }
        let i = usize::try_from(entry).ok().filter(|&i| i >= 1 && i < self.count() && !self.is_dir(i))?;
        Some(be32(&self.fst, i * 12 + 8))
    }

    /// `len` bytes at `offset` inside file entry `entry`.
    pub fn read_entry(&mut self, entry: i32, offset: u32, len: usize) -> Result<Vec<u8>, String> {
        let flen = self.entry_len(entry).ok_or(format!("DVD entry {entry}: not a file"))?;
        if let Some(b) = self.replaced.borrow().get(&entry) {
            let (o, l) = (offset as usize, len);
            return b.get(o..o + l).map(|s| s.to_vec()).ok_or(format!("DVD entry {entry}: read {offset:#x}+{len:#x} past the end of the mod file"));
        }
        if offset as u64 + len as u64 > flen as u64 {
            return Err(format!("DVD entry {entry}: read {offset:#x}+{len:#x} past the end ({flen:#x})"));
        }
        let i = entry as usize;
        let start = be32(&self.fst, i * 12 + 4) as u64;
        read_at(self.f.as_mut(), start + offset as u64, len)
    }

    /// The disc's own copy of `want`, ignoring the overlay (a cosmetic check needs the vanilla costume).
    pub fn read_vanilla(&mut self, want: &str) -> Result<Vec<u8>, String> {
        let e = self.entry_for_path_disc(want);
        let i = usize::try_from(e).ok().filter(|&i| i >= 1 && i < self.count() && !self.is_dir(i)).ok_or_else(|| format!("{want}: not on the disc"))?;
        let (start, n) = (be32(&self.fst, i * 12 + 4) as u64, be32(&self.fst, i * 12 + 8) as usize);
        read_at(self.f.as_mut(), start, n)
    }

    /// Where disc entry `entry` lies in the image: (byte offset, length), or None when it is not a plain disc file or
    /// an overlay (mods) is mounted (slice 1 does not ask it). Pure: it
    /// asks the overlay nothing and caches nothing (the asset service's hints must not change game state).
    /// The disc entry of `path` on the disc itself (no overlay asked; -1: not there).
    pub fn disc_entry(&self, path: &str) -> i32 {
        self.entry_for_path_disc(path)
    }

    pub fn plain_location(&self, entry: i32) -> Option<(u64, usize)> {
        let i = usize::try_from(entry).ok().filter(|&i| i >= 1 && i < self.count() && !self.is_dir(i))?;
        if self.overlay.is_some() {
            return None;
        }
        Some((be32(&self.fst, i * 12 + 4) as u64, be32(&self.fst, i * 12 + 8) as usize))
    }

    /// The whole file `want` (a path, or a bare name anywhere on the disc).
    pub fn read(&mut self, want: &str) -> Result<Vec<u8>, String> {
        let e = self.entry_for_path(want);
        if e < 0 {
            return Err(format!("{want}: not on the disc"));
        }
        let n = self.entry_len(e).unwrap() as usize;
        self.read_entry(e, 0, n)
    }
}

fn read_at(f: Option<&mut std::fs::File>, off: u64, n: usize) -> Result<Vec<u8>, String> {
    #[cfg(feature = "corev1")]
    if f.is_none() {
        if let Some(r) = HOST_READ.get() {
            let mut v = vec![0u8; n];
            r(off, &mut v)?;
            return Ok(v);
        }
    }
    let f = f.ok_or_else(|| "no disc".to_string())?;
    // Not zeroed first: the read fills it (the zeroing was ~5% of the boot tick, first-touch page faults included).
    let mut v = Vec::with_capacity(n);
    if crate::cow::active() {
        // the arena is copy-on-write (a rollback run's snapshot base): the OS read cannot fault a protected page in
        // (ReadFile fails with ERROR_NOACCESS), so the buffer's pages are written once first
        v.resize(n, 0);
        f.seek(SeekFrom::Start(off)).and_then(|_| f.read_exact(&mut v)).map_err(|e| format!("disc read @{off:#x}: {e}"))?;
        return Ok(v);
    }
    f.seek(SeekFrom::Start(off)).and_then(|_| (&mut *f).take(n as u64).read_to_end(&mut v)).map_err(|e| format!("disc read @{off:#x}: {e}"))?;
    if v.len() != n {
        return Err(format!("disc read @{off:#x}: short read ({} of {n} bytes)", v.len()));
    }
    Ok(v)
}

/// Core v1 (feature `corev1`): the host's raw disc reader (`disc_read`), set once before the engine boots. With it
/// set, [`Gcm::open`] reads the disc through the host whatever path it is given.
#[cfg(feature = "corev1")]
pub static HOST_READ: std::sync::OnceLock<fn(u64, &mut [u8]) -> Result<(), String>> = std::sync::OnceLock::new();
