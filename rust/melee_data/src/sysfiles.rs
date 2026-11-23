//! The Slippi system-file layer (`--slippi-menus on`): Slippi's patched menu files, served by name in place of the
//! disc's own copies. Follows Hero's `load_system_files` (Melee Unlocked `port/app/source_host.cpp`): 13 files, built
//! once at boot, all or nothing (`SlippiCSS_gui.dat` is optional), so vanilla and patched menu files never mix.
//!
//! Nothing here is committed data. Slippi's diffs and whole files are fetched at run time from Slippi's public GPL
//! repository (`project-slippi/Ishiiruka`, pinned revisions below) and checked against the sha256 list in
//! [`MANIFEST`]; the diff-built files are made from the player's own disc. Both live in the user's cache folder.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Where a Slippi source file comes from. `rev` pins a commit or tag of `project-slippi/Ishiiruka`.
#[derive(Debug, Clone, Copy)]
pub struct Source {
    /// The file name under `Data/Sys/GameFiles/GALE01/`.
    pub name: &'static str,
    pub rev: &'static str,
    pub size: usize,
    pub sha256: &'static str,
}

/// Ishiiruka `slippi` branch head as of 2026-11-20 (the same bytes Hero vendors in `port/slippi_sys`).
const REV_MAIN: &str = "e0e798cb6c8dfa6e8a5565805b6709974284468e";
/// `slpCSS.dat` was dropped from the tree later; v3.5.0 holds the copy Hero vendors.
const REV_SLPCSS: &str = "v3.5.0";

/// The fetched Slippi files and their hashes (hashes only; never the files).
pub const MANIFEST: &[Source] = &[
    Source { name: "MnMaAll.usd.diff", rev: REV_MAIN, size: 186_319, sha256: "e0e261203e013f7e51c29f808165ca36dca10296c3e613b1b225e0a6d4978263" },
    Source { name: "MnMaAll.dat.diff", rev: REV_MAIN, size: 185_895, sha256: "4f38bfa9716064fd59315c26210ec6383b62ee0d4445beb9b6cca1f09134ea75" },
    Source { name: "SdMenu.usd.diff", rev: REV_MAIN, size: 34_990, sha256: "00a92643e41af867dd22ba159f7ec022e6aa3b7288b537b2ba00cf162f5579cb" },
    Source { name: "SdMenu.dat.diff", rev: REV_MAIN, size: 33_387, sha256: "b284732a79b787129704e81c110d9ac11210bc3e8f579e27e8e38525b656fe50" },
    Source { name: "MnSlMap.usd.diff", rev: REV_MAIN, size: 58_850, sha256: "dcf99eeaab8f981a2aad25e341eda79bc9b417dd098fc39ff20cb328fbc1510d" },
    Source { name: "MnSlMap.dat.diff", rev: REV_MAIN, size: 59_031, sha256: "43860c806406b85e05d312401548ef7b0da15e62d44bbd279ff2885e4ea3ec27" },
    Source { name: "SdSlChr.usd.diff", rev: REV_MAIN, size: 2_513, sha256: "fde2f393c405244f9100a0522a769d4fcce9e892dd9b87b0e0c38670ef132f38" },
    Source { name: "SdSlChr.dat.diff", rev: REV_MAIN, size: 2_669, sha256: "0194bcbb4fd26df4e499edd52ea7a64743a493515b07d6114cf378c8720a9405" },
    Source { name: "MnExtAll.usd.diff", rev: REV_MAIN, size: 150, sha256: "c729c61a65d92aeaf4ab15ca7ec5a40ead336055b0defce9c74503e2651c9b18" },
    Source { name: "MnExtAll.dat.diff", rev: REV_MAIN, size: 149, sha256: "b6df247ddf12d0118d4b1deaf1d3cf8878ebb4d60b29fda7ad6a324246b05e22" },
    Source { name: "slpCSS.dat", rev: REV_SLPCSS, size: 254_495, sha256: "3b1a3f254b37cf5ce4f56b38d237d9809d0b11e72d05320f0e56fa3abd7a1692" },
    Source { name: "GameSetup_gui.dat", rev: REV_MAIN, size: 788_239, sha256: "824002d8835bebec42caf637b948b4da6178e4341668b30e3d3a2122c892892b" },
    Source { name: "SlippiCSS_gui.dat", rev: REV_MAIN, size: 237_571, sha256: "fe96bb798206d8090510292359fb65350da3f812ef5dbf44aa30b9f647d93e01" },
];

/// The 13 files the game is served, in Hero's order.
pub const SYSTEM_FILES: [&str; 13] = [
    "MnMaAll.usd", "MnMaAll.dat", "SdMenu.usd", "SdMenu.dat", "MnSlMap.usd", "MnSlMap.dat", "SdSlChr.usd",
    "SdSlChr.dat", "MnExtAll.usd", "MnExtAll.dat", "slpCSS.dat", "GameSetup_gui.dat", "SlippiCSS_gui.dat",
];

/// A file the menus can do without (Hero: the native CSS falls back to `slpCSS.dat`).
pub fn optional(name: &str) -> bool {
    name.eq_ignore_ascii_case("SlippiCSS_gui.dat")
}

pub fn source_url(s: &Source) -> String {
    format!("https://raw.githubusercontent.com/project-slippi/Ishiiruka/{}/Data/Sys/GameFiles/GALE01/{}", s.rev, s.name)
}

/// The manifest entry a system file is made from: `<name>.diff` (applied to the disc's copy) or the whole file.
pub fn source_for(name: &str) -> Option<(&'static Source, bool)> {
    let diff = format!("{name}.diff");
    MANIFEST
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(&diff))
        .map(|s| (s, true))
        .or_else(|| MANIFEST.iter().find(|s| s.name.eq_ignore_ascii_case(name)).map(|s| (s, false)))
}

/// Whether `data` is exactly the manifest's file.
pub fn verify(s: &Source, data: &[u8]) -> bool {
    data.len() == s.size && hex(&sha256(data)) == s.sha256
}

/// FNV-1a 32, the checksum Hero logs for each built file (`slippi menus: <path> <size> fnv1a`).
pub fn fnv1a(d: &[u8]) -> u32 {
    d.iter().fold(2_166_136_261u32, |h, &b| (h ^ u32::from(b)).wrapping_mul(16_777_619))
}

/// A verified source file: the cached copy when it checks out, else `fetch` (which must return the body), stored to
/// the cache only after it checks out. A mismatch is refused, never used.
pub fn obtain(s: &Source, cache: &Path, fetch: &mut dyn FnMut(&str) -> Result<Vec<u8>, String>) -> Result<Vec<u8>, String> {
    let p = cache.join(s.name);
    if let Ok(d) = std::fs::read(&p) {
        if verify(s, &d) {
            return Ok(d);
        }
        let _ = std::fs::remove_file(&p);
    }
    let d = fetch(&source_url(s))?;
    if !verify(s, &d) {
        return Err(format!("{}: hash mismatch ({} bytes), refused", s.name, d.len()));
    }
    let _ = std::fs::create_dir_all(cache);
    let tmp = cache.join(format!("{}.part", s.name));
    if std::fs::write(&tmp, &d).is_ok() {
        let _ = std::fs::rename(&tmp, &p);
    }
    Ok(d)
}

/// The served files, by lower-case bare name.
#[derive(Default, Debug)]
pub struct SysFiles {
    files: HashMap<String, Vec<u8>>,
    /// Built names in order, with size and fnv1a (for the log).
    pub built: Vec<(String, usize, u32)>,
}

impl SysFiles {
    /// Build every system file. `disc` reads the vanilla disc's copy by name. Any required file failing fails the
    /// whole layer (Err), so the caller boots the normal menus.
    pub fn build(
        cache: &Path,
        disc: &mut dyn FnMut(&str) -> Result<Vec<u8>, String>,
        fetch: &mut dyn FnMut(&str) -> Result<Vec<u8>, String>,
    ) -> Result<Self, String> {
        let mut out = Self::default();
        for name in SYSTEM_FILES {
            match Self::build_one(name, cache, disc, fetch) {
                Ok(d) => out.insert(name, d),
                Err(e) if optional(name) => out.built.push((format!("{name} left out ({e})"), 0, 0)),
                Err(e) => return Err(format!("cannot build {name}: {e}")),
            }
        }
        Ok(out)
    }

    fn build_one(
        name: &str,
        cache: &Path,
        disc: &mut dyn FnMut(&str) -> Result<Vec<u8>, String>,
        fetch: &mut dyn FnMut(&str) -> Result<Vec<u8>, String>,
    ) -> Result<Vec<u8>, String> {
        let (src, is_diff) = source_for(name).ok_or("no source")?;
        let body = obtain(src, cache, fetch)?;
        if !is_diff {
            return Ok(body);
        }
        let orig = disc(name)?;
        crate::vcdiff::decode(&orig, &body).map_err(|e| e.to_string())
    }

    pub fn insert(&mut self, name: &str, data: Vec<u8>) {
        self.built.push((name.to_string(), data.len(), fnv1a(&data)));
        self.files.insert(name.to_ascii_lowercase(), data);
    }

    /// The served copy of a file, by bare name (case-insensitive, any leading path ignored).
    pub fn get(&self, want: &str) -> Option<&[u8]> {
        let base = want.rsplit(['/', '\\']).next().unwrap_or(want);
        self.files.get(&base.to_ascii_lowercase()).map(Vec::as_slice)
    }

    /// Read `len` bytes at `off` of a served file; bytes past its end read as zeros (Hero's `read_system_file`).
    pub fn read_at(&self, want: &str, off: usize, len: usize) -> Option<Vec<u8>> {
        let d = self.get(want)?;
        let mut v = vec![0u8; len];
        if off < d.len() {
            let n = len.min(d.len() - off);
            v[..n].copy_from_slice(&d[off..off + n]);
        }
        Some(v)
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// Fetch one manifest URL with the system curl (Windows 10+ ships it); the disc-gated test only. The game fetches
/// in-process (nucleus/http.rs get_slippi_sys): no child process under the YouGame sandbox. Only Slippi's public GitHub raw files are
/// allowed; no credentials, no cookies. The caller checks the hash.
pub fn fetch_curl(url: &str) -> Result<Vec<u8>, String> {
    const ALLOWED: &str = "https://raw.githubusercontent.com/project-slippi/Ishiiruka/";
    if !url.starts_with(ALLOWED) || url.contains(['\'', '"', ' ', '\n']) {
        return Err("address not allowed".into());
    }
    let mut cmd = std::process::Command::new("curl");
    cmd.args(["-sSfL", "--max-time", "60", "--connect-timeout", "10", "--max-filesize", "4000000", "--proto", "=https", "--proto-redir", "=https", url]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let out = cmd.output().map_err(|_| "curl is not available".to_string())?;
    if !out.status.success() {
        return Err(format!("download failed ({})", String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(out.stdout)
}

/// The cache folder: `MELEE_SLIPPI_SYS_DIR`, else `SlippiSys` beside the exe.
pub fn default_cache_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("MELEE_SLIPPI_SYS_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(d);
    }
    std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("SlippiSys"))).unwrap_or_else(|| PathBuf::from("SlippiSys"))
}

// ---- sha256 (FIPS 180-4), so the crate keeps no dependencies ----

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01,
    0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
    0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08,
    0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut msg = data.to_vec();
    let bits = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bits.to_be_bytes());
    for block in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, c) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [t1.wrapping_add(t2), v[0], v[1], v[2], v[3].wrapping_add(t1), v[4], v[5], v[6]];
        }
        for (a, b) in h.iter_mut().zip(v) {
            *a = a.wrapping_add(b);
        }
    }
    let mut out = [0u8; 32];
    for (i, x) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&x.to_be_bytes());
    }
    out
}

pub fn hex(d: &[u8]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_values() {
        assert_eq!(hex(&sha256(b"")), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(hex(&sha256(b"abc")), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        let long = vec![b'a'; 1000];
        assert_eq!(hex(&sha256(&long)), "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3");
    }

    #[test]
    fn manifest_covers_every_system_file() {
        for n in SYSTEM_FILES {
            assert!(source_for(n).is_some(), "{n}");
        }
        assert_eq!(source_for("MnMaAll.usd").unwrap().1, true);
        assert_eq!(source_for("slpCSS.dat").unwrap().1, false);
        assert!(MANIFEST.iter().all(|s| s.sha256.len() == 64));
    }

    #[test]
    fn read_past_end_is_zero() {
        let mut s = SysFiles::default();
        s.insert("MnExtAll.usd", vec![1, 2, 3]);
        assert_eq!(s.read_at("/mnextall.usd", 1, 5).unwrap(), vec![2, 3, 0, 0, 0]);
        assert_eq!(s.read_at("MnExtAll.usd", 10, 2).unwrap(), vec![0, 0]);
        assert!(s.get("MnExtAll.dat").is_none());
    }

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("melee-sysfiles-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn obtain_refuses_mismatch_and_caches_good() {
        let body = b"hello slippi".to_vec();
        let h = hex(&sha256(&body));
        let s = Source { name: "x.dat", rev: "r", size: body.len(), sha256: Box::leak(h.into_boxed_str()) };
        let dir = tmpdir("obtain");
        let mut bad = |_: &str| Ok(b"hello SLIPPI".to_vec());
        assert!(obtain(&s, &dir, &mut bad).unwrap_err().contains("mismatch"));
        assert!(!dir.join("x.dat").exists());
        let mut calls = 0;
        let mut good = |u: &str| {
            calls += 1;
            assert!(u.starts_with("https://raw.githubusercontent.com/project-slippi/Ishiiruka/r/"));
            Ok(body.clone())
        };
        assert_eq!(obtain(&s, &dir, &mut good).unwrap(), body);
        let mut never = |_: &str| -> Result<Vec<u8>, String> { panic!("should use the cache") };
        assert_eq!(obtain(&s, &dir, &mut never).unwrap(), body);
        assert_eq!(calls, 1);
        // A corrupted cache file is dropped and refetched.
        std::fs::write(dir.join("x.dat"), b"junk").unwrap();
        let mut again = |_: &str| Ok(body.clone());
        assert_eq!(obtain(&s, &dir, &mut again).unwrap(), body);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn build_is_all_or_nothing() {
        let dir = tmpdir("build");
        // The fetch always fails: the layer must fail as a whole, not serve a partial set.
        let mut disc = |_: &str| Ok(vec![0u8; 16]);
        let mut fetch = |_: &str| -> Result<Vec<u8>, String> { Err("offline".into()) };
        let e = SysFiles::build(&dir, &mut disc, &mut fetch).unwrap_err();
        assert!(e.starts_with("cannot build MnMaAll.usd"), "{e}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
