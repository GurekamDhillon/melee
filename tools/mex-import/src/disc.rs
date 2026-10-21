//! Disc sources: a plain GameCube image (GCM/ISO) read through its FST, or an extracted tree
//! (a Nucleus m-ex project or its `files/` root).
//!
//! Port of `tools/mex_port/mex_hsd.py` `Gcm` and `make_mod_from_disc.py` `Folder` (workspace repo).
//! The FST walk is the same: entries of 12 bytes, word 0 = dir flag << 24 | name offset, a
//! directory's word 2 is the index one past its last child. Read-only; nothing here writes.
//! nod (the workspace's `nod/` checkout) was not used: these discs are plain GCM, the FST walk is
//! 40 lines, and nod would add a large dependency tree to the workspace lock.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const CHUNK: usize = 1 << 20;

/// Where one disc file's bytes are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Loc {
    /// (offset, size) in the image.
    Image(u64, u64),
    /// A loose file in an extracted tree.
    Loose(PathBuf, u64),
}
impl Loc {
    pub fn size(&self) -> u64 {
        match self {
            Loc::Image(_, s) | Loc::Loose(_, s) => *s,
        }
    }
}

#[allow(dead_code)] // dol_offset / project: kept for Nucleus-project DOL checks
pub struct Disc {
    pub path: PathBuf,
    pub is_image: bool,
    /// Disc path (FST spelling, `/`-separated, no leading slash) -> location.
    pub files: BTreeMap<String, Loc>,
    /// Image only: main.dol offset (0x420).
    pub dol_offset: u64,
    /// Image only: the disc id (0x000, 6 bytes), e.g. "GALE01".
    pub game_id: String,
    /// Extracted tree only: the project directory (parent of `files/`).
    pub project: Option<PathBuf>,
}

fn be32(b: &[u8], at: usize) -> io::Result<u32> {
    b.get(at..at + 4)
        .map(|s| u32::from_be_bytes(s.try_into().unwrap()))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("FST truncated at {at:#x}")))
}

/// Parse an FST blob into path -> (offset, size). `mex_hsd.Gcm._parse_fst`.
pub fn parse_fst(fst: &[u8]) -> io::Result<BTreeMap<String, Loc>> {
    let n = be32(fst, 8)? as usize;
    let strings = n * 12;
    if strings > fst.len() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "FST entry count runs past its size"));
    }
    let name_at = |off: usize| -> io::Result<String> {
        let s = fst
            .get(strings + off..)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "FST name offset out of range"))?;
        let end = s.iter().position(|&c| c == 0).unwrap_or(s.len());
        Ok(String::from_utf8_lossy(&s[..end]).into_owned())
    };
    let mut out = BTreeMap::new();
    let mut stack: Vec<(usize, String)> = vec![(n, String::new())];
    for i in 1..n {
        let e = i * 12;
        let w0 = be32(fst, e)?;
        let is_dir = (w0 >> 24) & 1 != 0;
        let name = name_at((w0 & 0x00FF_FFFF) as usize)?;
        let (a1, a2) = (be32(fst, e + 4)?, be32(fst, e + 8)?);
        while stack.len() > 1 && i >= stack.last().unwrap().0 {
            stack.pop();
        }
        let prefix = stack.last().unwrap().1.clone();
        if is_dir {
            stack.push((a2 as usize, format!("{prefix}{name}/")));
        } else {
            out.insert(format!("{prefix}{name}"), Loc::Image(a1 as u64, a2 as u64));
        }
    }
    Ok(out)
}

/// Top-level names of an extracted project that are bookkeeping, not disc payload.
const NOT_PAYLOAD: [&str; 5] = ["data", "assets", "sys", "project.mexproj", "mod.json"];

impl Disc {
    pub fn open(path: &Path) -> io::Result<Disc> {
        if path.is_dir() {
            Self::open_tree(path)
        } else {
            Self::open_image(path)
        }
    }

    pub fn open_image(path: &Path) -> io::Result<Disc> {
        let mut f = File::open(path)?;
        let mut hdr = [0u8; 0x430];
        f.read_exact(&mut hdr)?;
        let dol = be32(&hdr, 0x420)? as u64;
        let (fo, fs) = (be32(&hdr, 0x424)? as u64, be32(&hdr, 0x428)? as usize);
        if fs == 0 || fs > 64 << 20 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "not a GameCube image (FST size)"));
        }
        let mut fst = vec![0u8; fs];
        f.seek(SeekFrom::Start(fo))?;
        f.read_exact(&mut fst)?;
        Ok(Disc {
            path: path.to_path_buf(),
            is_image: true,
            files: parse_fst(&fst)?,
            dol_offset: dol,
            game_id: String::from_utf8_lossy(&hdr[..6]).into_owned(),
            project: None,
        })
    }

    pub fn open_tree(path: &Path) -> io::Result<Disc> {
        let root = if path.join("files").is_dir() { path.join("files") } else { path.to_path_buf() };
        let project = if root != path { path.to_path_buf() } else { path.parent().unwrap_or(path).to_path_buf() };
        let mut files = BTreeMap::new();
        let mut seen = std::collections::HashSet::new();
        let mut stack = vec![root.clone()];
        while let Some(d) = stack.pop() {
            for ent in std::fs::read_dir(&d)? {
                let ent = ent?;
                let ft = ent.file_type()?;
                let p = ent.path();
                if ft.is_symlink() {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("symlink in a disc tree: {}", p.display())));
                }
                if ft.is_dir() {
                    stack.push(p);
                    continue;
                }
                let rel = p.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
                let top = rel.split('/').next().unwrap_or("").to_ascii_lowercase();
                if NOT_PAYLOAD.contains(&top.as_str()) {
                    continue;
                }
                if !seen.insert(rel.to_ascii_lowercase()) {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("case-insensitive duplicate disc path: {rel}")));
                }
                let size = ent.metadata()?.len();
                files.insert(rel, Loc::Loose(p, size));
            }
        }
        Ok(Disc { path: path.to_path_buf(), is_image: false, files, dol_offset: 0, game_id: String::new(), project: Some(project) })
    }

    /// Case-insensitive lookup.
    pub fn find(&self, path: &str) -> Option<(&String, &Loc)> {
        let p = path.trim_start_matches('/');
        self.files.get_key_value(p).or_else(|| self.files.iter().find(|(k, _)| k.eq_ignore_ascii_case(p)))
    }

    /// Stream a file's bytes in chunks.
    pub fn stream(&self, loc: &Loc, mut sink: impl FnMut(&[u8]) -> io::Result<()>) -> io::Result<()> {
        let (mut f, size) = match loc {
            Loc::Image(off, size) => {
                let mut f = File::open(&self.path)?;
                f.seek(SeekFrom::Start(*off))?;
                (f, *size)
            }
            Loc::Loose(p, size) => (File::open(p)?, *size),
        };
        let mut left = size;
        let mut buf = vec![0u8; CHUNK];
        while left > 0 {
            let n = (left as usize).min(CHUNK);
            f.read_exact(&mut buf[..n])?;
            sink(&buf[..n])?;
            left -= n as u64;
        }
        Ok(())
    }

    pub fn read(&self, loc: &Loc) -> io::Result<Vec<u8>> {
        let mut v = Vec::with_capacity(loc.size() as usize);
        self.stream(loc, |c| {
            v.extend_from_slice(c);
            Ok(())
        })?;
        Ok(v)
    }

    pub fn read_path(&self, path: &str) -> io::Result<Vec<u8>> {
        let (_, loc) = self
            .find(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("{path} not on {}", self.path.display())))?;
        self.read(&loc.clone())
    }

    pub fn sha256(&self, loc: &Loc) -> io::Result<String> {
        let mut h = crate::sha256::Sha256::new();
        self.stream(loc, |c| {
            h.update(c);
            Ok(())
        })?;
        Ok(h.hex())
    }
}

/// Byte-compare two same-size files on two discs (cheaper than hashing both sides).
pub fn same_content(a: &Disc, la: &Loc, b: &Disc, lb: &Loc) -> io::Result<bool> {
    if la.size() != lb.size() {
        return Ok(false);
    }
    let open = |d: &Disc, l: &Loc| -> io::Result<File> {
        Ok(match l {
            Loc::Image(off, _) => {
                let mut f = File::open(&d.path)?;
                f.seek(SeekFrom::Start(*off))?;
                f
            }
            Loc::Loose(p, _) => File::open(p)?,
        })
    };
    let (mut fa, mut fb) = (open(a, la)?, open(b, lb)?);
    let mut left = la.size();
    let (mut ba, mut bb) = (vec![0u8; CHUNK], vec![0u8; CHUNK]);
    while left > 0 {
        let n = (left as usize).min(CHUNK);
        fa.read_exact(&mut ba[..n])?;
        fb.read_exact(&mut bb[..n])?;
        if ba[..n] != bb[..n] {
            return Ok(false);
        }
        left -= n as u64;
    }
    Ok(true)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A synthetic FST: root, "a.dat", dir "audio" { "us" { "x.ssm" } }, "z.usd".
    pub fn fst_bytes(entries: &[(bool, &str, u32, u32)]) -> Vec<u8> {
        let n = entries.len() as u32 + 1;
        let mut names = Vec::new();
        let mut ents = vec![0u8; n as usize * 12];
        ents[0] = 1;
        ents[8..12].copy_from_slice(&n.to_be_bytes());
        for (i, (dir, name, a1, a2)) in entries.iter().enumerate() {
            let e = (i + 1) * 12;
            let w0 = ((*dir as u32) << 24) | names.len() as u32;
            ents[e..e + 4].copy_from_slice(&w0.to_be_bytes());
            ents[e + 4..e + 8].copy_from_slice(&a1.to_be_bytes());
            ents[e + 8..e + 12].copy_from_slice(&a2.to_be_bytes());
            names.extend_from_slice(name.as_bytes());
            names.push(0);
        }
        ents.extend_from_slice(&names);
        ents
    }

    #[test]
    fn fst_walk_nested_dirs() {
        // index: 1 a.dat, 2 audio (end 5), 3 us (end 5), 4 x.ssm, 5 z.usd
        let fst = fst_bytes(&[
            (false, "a.dat", 0x100, 4),
            (true, "audio", 0, 5),
            (true, "us", 2, 5),
            (false, "x.ssm", 0x200, 8),
            (false, "z.usd", 0x300, 2),
        ]);
        let m = parse_fst(&fst).unwrap();
        let keys: Vec<_> = m.keys().cloned().collect();
        assert_eq!(keys, ["a.dat", "audio/us/x.ssm", "z.usd"]);
        assert_eq!(m["audio/us/x.ssm"], Loc::Image(0x200, 8));
    }
}
