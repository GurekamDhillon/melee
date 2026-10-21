//! Files on the user's GameCube disc image (GCM/ISO), by full path through the FST at header
//! +0x424 (`audio/us/smash2.sem`), or by bare name when the path has no directory. Whole-file
//! reads for the SSM/SEM banks; ranged reads for HPS music, which is streamed block by block as
//! the C streams it through DevCom. Local only: nothing read here is written anywhere or committed.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn be32(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}

/// One FST file entry: where it is on the disc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// FST index (the C's DVD entrynum).
    pub entrynum: i32,
    pub offset: u32,
    pub length: u32,
}

pub struct Disc {
    f: File,
    fst: Vec<u8>,
}

impl Disc {
    pub fn open(path: &str) -> Result<Self, String> {
        let mut f = File::open(path).map_err(|e| format!("{path}: {e}"))?;
        let h = read_at(&mut f, 0x424, 8)?;
        let fst = read_at(&mut f, be32(&h, 0) as u64, be32(&h, 4) as usize)?;
        if fst.len() < 12 {
            return Err(format!("{path}: no FST"));
        }
        Ok(Self { f, fst })
    }

    fn name(&self, i: usize) -> &[u8] {
        let n = be32(&self.fst, 8) as usize;
        let names = &self.fst[n * 12..];
        let no = (be32(&self.fst, i * 12) & 0xFF_FFFF) as usize;
        let end = names[no..].iter().position(|&c| c == 0).unwrap_or(0);
        &names[no..no + end]
    }

    /// DVDConvertPathToEntrynum: a '/'-separated path from the root (case-insensitive, leading
    /// '/' optional); a bare name matches the first file of that name anywhere.
    pub fn find(&self, path: &str) -> Option<Entry> {
        let n = be32(&self.fst, 8) as usize;
        let path = path.trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let entry = |i: usize| Entry {
            entrynum: i as i32,
            offset: be32(&self.fst, i * 12 + 4),
            length: be32(&self.fst, i * 12 + 8),
        };
        if parts.len() == 1 {
            return (1..n)
                .find(|&i| self.fst[i * 12] == 0 && self.name(i).eq_ignore_ascii_case(parts[0].as_bytes()))
                .map(entry);
        }
        // Walk directories: entries (lo, hi) are the current directory's children.
        let (mut lo, mut hi) = (1usize, n);
        for (k, part) in parts.iter().enumerate() {
            let last = k + 1 == parts.len();
            let mut i = lo;
            let mut found = None;
            while i < hi {
                let is_dir = self.fst[i * 12] != 0;
                if self.name(i).eq_ignore_ascii_case(part.as_bytes()) && is_dir != last {
                    found = Some(i);
                    break;
                }
                i = if is_dir { be32(&self.fst, i * 12 + 8) as usize } else { i + 1 };
            }
            let i = found?;
            if last {
                return Some(entry(i));
            }
            lo = i + 1;
            hi = be32(&self.fst, i * 12 + 8) as usize;
        }
        None
    }

    pub fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        let e = self.find(path).ok_or_else(|| format!("{path}: not on the disc"))?;
        read_at(&mut self.f, e.offset as u64, e.length as usize)
    }

    /// Bytes `off..off+len` of a file, clipped to its end (the tail of a short read is zero, as a
    /// DVD read past EOF of a 32-byte-rounded request leaves the buffer).
    pub fn read_range(&mut self, e: Entry, off: u32, len: usize) -> Result<Vec<u8>, String> {
        let mut v = vec![0u8; len];
        if off < e.length {
            let n = len.min((e.length - off) as usize);
            let got = read_at(&mut self.f, e.offset as u64 + off as u64, n)?;
            v[..n].copy_from_slice(&got);
        }
        Ok(v)
    }
}

fn read_at(f: &mut File, off: u64, n: usize) -> Result<Vec<u8>, String> {
    let mut v = vec![0; n];
    f.seek(SeekFrom::Start(off)).and_then(|_| f.read_exact(&mut v)).map_err(|e| format!("disc read @{off:#x}: {e}"))?;
    Ok(v)
}

/// What the stream player reads music from: ranged reads of one opened file.
pub trait StreamSource: Send {
    fn read(&mut self, off: u32, len: usize) -> Vec<u8>;
}

/// A file on a [`Disc`], streamed.
pub struct DiscStream {
    pub disc: Disc,
    pub entry: Entry,
}

impl StreamSource for DiscStream {
    fn read(&mut self, off: u32, len: usize) -> Vec<u8> {
        self.disc.read_range(self.entry, off, len).unwrap_or_else(|_| vec![0; len])
    }
}

/// A file already in memory (tests).
impl StreamSource for Vec<u8> {
    fn read(&mut self, off: u32, len: usize) -> Vec<u8> {
        let mut v = vec![0u8; len];
        let off = off as usize;
        if off < self.len() {
            let n = len.min(self.len() - off);
            v[..n].copy_from_slice(&self[off..off + n]);
        }
        v
    }
}
