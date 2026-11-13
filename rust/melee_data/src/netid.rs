//! Netplay content identity (notes/net.md "Mod content identity"): a hash over the CONTENTS of every file each
//! mounted mod folder contributes, so two installs whose mods differ in any byte that can reach the sim refuse
//! each other instead of desyncing.
//!
//! - Every regular file under the mod's folder is hashed (mod.json, files/, geno.json, anim/script overrides,
//!   wasm, anything a future loader reads): no hand-kept list. Dot-files/dot-folders (`.git`, editor swap files)
//!   and the cache file are skipped.
//! - Order: relative path with `/` separators, ASCII-lowercased, sorted bytewise. Listing order, the OS's path
//!   separator and timestamps never enter the hash. Two paths that differ only in case both count (sorted with
//!   the original spelling as a tie-break).
//! - Text files (the extensions `modreg::fingerprint_size` treats as text) hash with every CR of a CR LF pair
//!   dropped, so a git checkout with autocrlf matches its LF copy; binary files hash their exact bytes.
//! - Hash: FNV-1a 64 (already the handshake's hash; no new crate). Accidental differences are what it guards.
//!
//! The cache (`Cache`) keys a file's hash by path + size + mtime (ns). It can only go stale when a file is
//! rewritten with the same size AND its mtime restored to the nanosecond; netplay connect therefore never trusts it
//! (`Mode::Force` re-reads every byte and refreshes the cache); it serves start-up / spectator / replay identities.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
/// The cache file's name, written in the mods folder (never inside a mod, so it is never hashed).
pub const CACHE_NAME: &str = ".netid-cache";
const TEXT: [&str; 13] = [".lua", ".json", ".md", ".txt", ".wgsl", ".csv", ".cfg", ".toml", ".xml", ".glsl", ".yml", ".yaml", ".ini"];

fn fnv(mut h: u64, d: &[u8]) -> u64 {
    for &b in d {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// Whether the file's bytes are normalised as text (CR LF -> LF).
pub fn is_text(rel: &str) -> bool {
    let l = rel.to_ascii_lowercase();
    TEXT.iter().any(|t| l.ends_with(t))
}

/// FNV-1a 64 of a file's content (text files with CR LF folded to LF; streamed, a CR at a chunk edge is carried).
pub fn hash_content(path: &Path, text: bool) -> std::io::Result<u64> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut buf = vec![0u8; 1 << 16];
    let mut h = FNV_OFFSET;
    let mut held_cr = false;
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        if !text {
            h = fnv(h, &buf[..n]);
            continue;
        }
        for &c in &buf[..n] {
            if held_cr {
                held_cr = false;
                if c != b'\n' {
                    h = fnv(h, b"\r");
                }
            }
            if c == b'\r' {
                held_cr = true;
                continue;
            }
            h = fnv(h, &[c]);
        }
    }
    if held_cr {
        h = fnv(h, b"\r");
    }
    Ok(h)
}

/// How file hashes are obtained.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Trust a cache entry whose size and mtime match.
    Cached,
    /// Re-read every byte (netplay connect); the cache is refreshed with what was read.
    Force,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Entry {
    size: u64,
    mtime_ns: u128,
    hash: u64,
}

/// path + size + mtime -> content hash, persisted as text lines in `<mods>/.netid-cache`.
#[derive(Default)]
pub struct Cache {
    map: HashMap<String, Entry>,
    file: Option<PathBuf>,
    dirty: bool,
    /// Files actually read in this run (for the start-up cost line and the tests).
    pub hashed: usize,
    pub hits: usize,
}

impl Cache {
    pub fn in_memory() -> Cache {
        Cache::default()
    }

    /// Load `<dir>/.netid-cache` (an unreadable or malformed file is an empty cache).
    pub fn load(dir: &Path) -> Cache {
        let file = dir.join(CACHE_NAME);
        let mut c = Cache { file: Some(file.clone()), ..Cache::default() };
        if let Ok(t) = std::fs::read_to_string(&file) {
            for l in t.lines() {
                let mut it = l.splitn(4, '\t');
                let (Some(s), Some(m), Some(h), Some(p)) = (it.next(), it.next(), it.next(), it.next()) else { continue };
                let (Ok(size), Ok(mtime_ns), Ok(hash)) = (s.parse(), m.parse(), u64::from_str_radix(h, 16)) else { continue };
                c.map.insert(p.to_string(), Entry { size, mtime_ns, hash });
            }
        }
        c
    }

    /// Write the cache back when it changed (best effort: a read-only mods folder only loses the speed-up).
    pub fn save(&mut self) {
        let (Some(f), true) = (&self.file, self.dirty) else { return };
        let mut v: Vec<_> = self.map.iter().collect();
        v.sort_by(|a, b| a.0.cmp(b.0));
        let mut s = String::new();
        for (p, e) in v {
            s += &format!("{}\t{}\t{:016x}\t{}\n", e.size, e.mtime_ns, e.hash, p);
        }
        let tmp = f.with_extension("tmp");
        if std::fs::write(&tmp, s).is_ok() && std::fs::rename(&tmp, f).is_ok() {
            self.dirty = false;
        }
    }

    fn hash(&mut self, path: &Path, rel: &str, mode: Mode) -> std::io::Result<u64> {
        let md = std::fs::metadata(path)?;
        let mtime_ns = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_nanos()).unwrap_or(0);
        let key = path.to_string_lossy().into_owned();
        if mode == Mode::Cached && mtime_ns != 0 {
            if let Some(e) = self.map.get(&key) {
                if e.size == md.len() && e.mtime_ns == mtime_ns {
                    self.hits += 1;
                    return Ok(e.hash);
                }
            }
        }
        let hash = hash_content(path, is_text(rel))?;
        self.hashed += 1;
        let e = Entry { size: md.len(), mtime_ns, hash };
        if self.map.get(&key) != Some(&e) {
            self.map.insert(key, e);
            self.dirty = true;
        }
        Ok(hash)
    }
}

/// One hashed file of a mod: its normalised relative path and content hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileId {
    pub rel: String,
    pub hash: u64,
}

/// A mod folder's content identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModId {
    pub id: String,
    pub files: Vec<FileId>,
    pub digest: u64,
}

impl ModId {
    /// "id#hhhhhhhh" (the low 32 bits of the digest), as the handshake description lists it.
    pub fn short(&self) -> String {
        format!("{}#{:08x}", self.id, self.digest as u32)
    }
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> std::io::Result<()> {
    for e in std::fs::read_dir(dir)? {
        let e = e?;
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let p = e.path();
        let ft = e.file_type()?;
        if ft.is_dir() {
            walk(root, &p, out)?;
        } else if ft.is_file() || (ft.is_symlink() && p.is_file()) {
            let rel = p.strip_prefix(root).unwrap_or(&p).components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/");
            out.push((rel, p));
        }
    }
    Ok(())
}

/// Hash every file under one mod folder. A file that cannot be read fails the whole identity (an unreadable
/// file must not quietly drop out of the hash).
pub fn hash_mod(id: &str, root: &Path, cache: &mut Cache, mode: Mode) -> std::io::Result<ModId> {
    let mut list = Vec::new();
    walk(root, root, &mut list)?;
    // normalised key, then the original spelling so two names differing only in case still sort stably
    let mut keyed: Vec<(String, String, PathBuf)> = list.into_iter().map(|(r, p)| (r.to_ascii_lowercase(), r, p)).collect();
    keyed.sort();
    let mut files = Vec::with_capacity(keyed.len());
    let mut h = fnv(FNV_OFFSET, id.to_ascii_lowercase().as_bytes());
    h = fnv(h, &[0]);
    for (k, r, p) in keyed {
        let fh = cache.hash(&p, &r, mode)?;
        h = fnv(h, k.as_bytes());
        h = fnv(h, &[0]);
        h = fnv(h, &fh.to_le_bytes());
        files.push(FileId { rel: k, hash: fh });
    }
    Ok(ModId { id: id.to_string(), files, digest: if h == 0 { 1 } else { h } })
}

/// The set's digest: the mods sorted by lowercased id, each id and digest folded in. 0 for an empty set.
pub fn set_digest(mods: &[ModId]) -> u64 {
    if mods.is_empty() {
        return 0;
    }
    let mut v: Vec<&ModId> = mods.iter().collect();
    v.sort_by_key(|m| m.id.to_ascii_lowercase());
    let mut h = FNV_OFFSET;
    for m in v {
        h = fnv(h, m.id.to_ascii_lowercase().as_bytes());
        h = fnv(h, &m.digest.to_le_bytes());
    }
    if h == 0 {
        1
    } else {
        h
    }
}

// ---- hot reload freeze ----

static FROZEN: AtomicBool = AtomicBool::new(false);

/// Called when a netplay session opens (after the identity was agreed): from here on no loader may re-read a
/// sim-affecting mod file (Lab hot reload of geno.json / anim or script overrides). Lasts until `thaw`.
pub fn freeze() {
    FROZEN.store(true, Ordering::SeqCst);
}

/// The session closed: offline hot reload is allowed again.
pub fn thaw() {
    FROZEN.store(false, Ordering::SeqCst);
}

/// True while a netplay session holds the content identity: hot reloaders must refuse.
pub fn frozen() -> bool {
    FROZEN.load(Ordering::SeqCst)
}

/// The refusal text every hot reloader uses while `frozen()`.
pub const FROZEN_MSG: &str = "netplay session open: mod files are locked to the identity both players agreed on; reload after the session";

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("netid_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn id_of(d: &Path) -> ModId {
        hash_mod("m", d, &mut Cache::in_memory(), Mode::Force).unwrap()
    }

    #[test]
    fn mtime_does_not_matter() {
        let d = tmp("mtime");
        std::fs::write(d.join("a.bin"), b"abc").unwrap();
        let a = id_of(&d);
        let f = std::fs::File::options().write(true).open(d.join("a.bin")).unwrap();
        f.set_modified(std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000)).unwrap();
        drop(f);
        assert_eq!(a, id_of(&d));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn one_byte_changes_it() {
        let d = tmp("byte");
        std::fs::create_dir_all(d.join("anim")).unwrap();
        std::fs::write(d.join("anim").join("x.bin"), b"abcd").unwrap();
        let a = id_of(&d);
        std::fs::write(d.join("anim").join("x.bin"), b"abce").unwrap();
        let b = id_of(&d);
        assert_ne!(a.digest, b.digest);
        assert_ne!(set_digest(&[a]), set_digest(&[b]));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn listing_order_and_set_order_do_not_matter() {
        let (d1, d2) = (tmp("ord1"), tmp("ord2"));
        for n in ["b.txt", "a.txt", "c.bin"] {
            std::fs::write(d1.join(n), n).unwrap();
        }
        for n in ["c.bin", "a.txt", "b.txt"] {
            std::fs::write(d2.join(n), n).unwrap();
        }
        let (a, b) = (id_of(&d1), id_of(&d2));
        assert_eq!(a, b);
        let x = ModId { id: "x".into(), files: vec![], digest: 5 };
        assert_eq!(set_digest(&[a.clone(), x.clone()]), set_digest(&[x, a]));
        let _ = std::fs::remove_dir_all(&d1);
        let _ = std::fs::remove_dir_all(&d2);
    }

    #[test]
    fn crlf_and_case() {
        let (d1, d2) = (tmp("crlf1"), tmp("crlf2"));
        std::fs::write(d1.join("mod.json"), b"{\r\n\"a\":1\r\n}\r").unwrap();
        std::fs::write(d2.join("MOD.json"), b"{\n\"a\":1\n}\r").unwrap();
        // text: CR LF == LF, a lone CR is kept; the path is compared lowercased
        assert_eq!(id_of(&d1), id_of(&d2));
        // binary: CR LF is content
        std::fs::write(d1.join("x.bin"), b"\r\n").unwrap();
        std::fs::write(d2.join("x.bin"), b"\n").unwrap();
        assert_ne!(id_of(&d1), id_of(&d2));
        let _ = std::fs::remove_dir_all(&d1);
        let _ = std::fs::remove_dir_all(&d2);
    }

    #[test]
    fn dotfiles_skipped_subdirs_counted() {
        let d = tmp("dot");
        std::fs::write(d.join("a.bin"), b"1").unwrap();
        let a = id_of(&d);
        std::fs::create_dir_all(d.join(".git")).unwrap();
        std::fs::write(d.join(".git").join("x"), b"1").unwrap();
        assert_eq!(a, id_of(&d));
        std::fs::create_dir_all(d.join("script_overrides")).unwrap();
        std::fs::write(d.join("script_overrides").join("s.lua"), b"1").unwrap();
        let b = id_of(&d);
        assert_ne!(a, b);
        assert_eq!(b.files[1].rel, "script_overrides/s.lua");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn cache_hits_and_force() {
        let d = tmp("cache");
        let m = d.join("m");
        std::fs::create_dir_all(&m).unwrap();
        std::fs::write(m.join("a.bin"), b"aaaa").unwrap();
        let mut c = Cache::load(&d);
        let a = hash_mod("m", &m, &mut c, Mode::Cached).unwrap();
        c.save();
        let mut c = Cache::load(&d);
        assert_eq!(a, hash_mod("m", &m, &mut c, Mode::Cached).unwrap());
        assert_eq!((c.hits, c.hashed), (1, 0));
        // same size, mtime put back: the cache is fooled, a forced pass is not
        let t = std::fs::metadata(m.join("a.bin")).unwrap().modified().unwrap();
        std::fs::write(m.join("a.bin"), b"aaab").unwrap();
        let f = std::fs::File::options().write(true).open(m.join("a.bin")).unwrap();
        f.set_modified(t).unwrap();
        drop(f);
        let mut c = Cache::load(&d);
        assert_eq!(a, hash_mod("m", &m, &mut c, Mode::Cached).unwrap());
        let forced = hash_mod("m", &m, &mut c, Mode::Force).unwrap();
        assert_ne!(a, forced);
        // and the forced pass refreshed the entry
        assert_eq!(forced, hash_mod("m", &m, &mut c, Mode::Cached).unwrap());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn freeze_flag() {
        assert!(!frozen());
        freeze();
        assert!(frozen());
        thaw();
        assert!(!frozen());
    }
}
