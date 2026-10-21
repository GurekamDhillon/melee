//! Read one file from the user's GameCube disc image (GCM/ISO): the FST at header +0x424.
//! Local only: nothing read here is written anywhere or committed.

use std::io::{Read, Seek, SeekFrom};

pub struct Disc {
    f: std::fs::File,
    fst: Vec<u8>,
    /// gdmods mounted over the disc (`crate::mods`): a mounted file answers before the disc.
    pub mods: Option<crate::mods::Mods>,
    /// Files answered by a mod, in read order (for the run log).
    pub from_mods: Vec<String>,
}

fn be32(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}

impl Disc {
    pub fn open(path: &str) -> Result<Self, String> {
        let mut f = std::fs::File::open(path).map_err(|e| format!("{path}: {e}"))?;
        let h = read_at(&mut f, 0x424, 8)?;
        let fst = read_at(&mut f, be32(&h, 0) as u64, be32(&h, 4) as usize)?;
        Ok(Self { f, fst, mods: None, from_mods: vec![] })
    }

    /// A root-level or nested file by bare name (case-insensitive), as `iso_file` in tests/dat.rs.
    pub fn read(&mut self, want: &str) -> Result<Vec<u8>, String> {
        if let Some(p) = self.mods.as_ref().and_then(|m| m.lookup(want)) {
            let v = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
            self.from_mods.push(want.to_string());
            return Ok(v);
        }
        self.read_vanilla(want)
    }

    /// The vanilla disc's own copy, ignoring mounted mods.
    pub fn read_vanilla(&mut self, want: &str) -> Result<Vec<u8>, String> {
        let fst = &self.fst;
        let n = be32(fst, 8) as usize;
        let names = &fst[n * 12..];
        for i in 1..n {
            let e = &fst[i * 12..i * 12 + 12];
            if e[0] != 0 {
                continue; // directory
            }
            let no = (be32(e, 0) & 0xFF_FFFF) as usize;
            let end = names[no..].iter().position(|&c| c == 0).unwrap_or(0);
            if names[no..no + end].eq_ignore_ascii_case(want.as_bytes()) {
                let (o, l) = (be32(e, 4), be32(e, 8));
                return read_at(&mut self.f, o as u64, l as usize);
            }
        }
        Err(format!("{want}: not on the disc"))
    }
}

fn read_at(f: &mut std::fs::File, off: u64, n: usize) -> Result<Vec<u8>, String> {
    let mut v = vec![0; n];
    f.seek(SeekFrom::Start(off)).and_then(|_| f.read_exact(&mut v)).map_err(|e| format!("disc read @{off:#x}: {e}"))?;
    Ok(v)
}
