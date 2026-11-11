//! The Wii stage archive (`ARC\0`) and Nintendo LZ77 (types 0x10 and 0x11), as pure functions over byte slices.
//! Spec: bosspackun-format.md section 1.

use crate::error::{be16, be32, get, Error, Result};
use alloc::borrow::Cow;
use alloc::vec::Vec;

/// Largest decompressed body accepted (the boss stage's biggest entry is under 2 MB).
pub const MAX_DECOMPRESSED: usize = 64 << 20;
/// Deepest archive nesting walked.
pub const MAX_DEPTH: u32 = 8;

/// One entry of an `ARC\0` container.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArcEntry<'a> {
    pub kind: u16,
    pub id: u16,
    pub group: u8,
    pub body: &'a [u8],
}

pub fn is_arc(d: &[u8]) -> bool {
    d.len() >= 0x40 && &d[..4] == b"ARC\0"
}

/// Entries of an ARC container: 0x40-byte header (u16 count at +6), then per entry a 0x20-byte header (u16 type, u16
/// id, u32 body size, u8 group) and its body; the next entry starts at the body end aligned up to 32.
pub fn arc_entries(d: &[u8]) -> Result<Vec<ArcEntry<'_>>> {
    if !is_arc(d) {
        return Err(Error::BadMagic);
    }
    let n = be16(d, 6)? as usize;
    let mut out = Vec::with_capacity(n.min(1024));
    let mut p = 0x40usize;
    for _ in 0..n {
        let kind = be16(d, p)?;
        let id = be16(d, p + 2)?;
        let size = be32(d, p + 4)? as usize;
        let group = get(d, p + 8, 1)?[0];
        let body = get(d, p + 0x20, size)?;
        out.push(ArcEntry { kind, id, group, body });
        p = (p + 0x20 + size).checked_add(0x1f).ok_or(Error::TooLarge { what: "arc entry", value: size })? & !0x1f;
    }
    Ok(out)
}

pub fn is_lz77(d: &[u8]) -> bool {
    d.len() >= 4 && (d[0] == 0x10 || d[0] == 0x11)
}

/// Decompress Nintendo LZ77. Byte 0 is the type (0x10 or 0x11); bytes 1-3 the little-endian size, or, when those are
/// zero, a little-endian u32 at +4 with the stream starting at +8. Flag bytes are read MSB first; a set bit is a
/// back-reference. Type 0x10 references are 2 bytes: length (high nibble) + 3, distance (12 bits) + 1. Type 0x11 adds
/// two longer forms: high nibble 0 -> 3 bytes, 8-bit length + 0x11; high nibble 1 -> 4 bytes, 16-bit length + 0x111;
/// otherwise 2 bytes with length nibble + 1.
pub fn lz77_decompress(d: &[u8]) -> Result<Vec<u8>> {
    if !is_lz77(d) {
        return Err(Error::BadMagic);
    }
    let ty = d[0];
    let mut n = d[1] as usize | (d[2] as usize) << 8 | (d[3] as usize) << 16;
    let mut p = 4usize;
    if n == 0 {
        let b = get(d, 4, 4)?;
        n = u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize;
        p = 8;
    }
    if n > MAX_DECOMPRESSED {
        return Err(Error::TooLarge { what: "lz77 size", value: n });
    }
    let byte = |p: usize| d.get(p).copied().ok_or(Error::Lz77 { at: p });
    let mut o: Vec<u8> = Vec::with_capacity(n);
    while o.len() < n {
        let flags = byte(p)?;
        p += 1;
        for bit in 0..8 {
            if o.len() >= n {
                break;
            }
            if flags & (0x80 >> bit) == 0 {
                o.push(byte(p)?);
                p += 1;
                continue;
            }
            let (len, dist);
            let b0 = byte(p)? as usize;
            if ty == 0x10 {
                let b1 = byte(p + 1)? as usize;
                p += 2;
                len = (b0 >> 4) + 3;
                dist = ((b0 & 0xf) << 8 | b1) + 1;
            } else {
                match b0 >> 4 {
                    0 => {
                        let (b1, b2) = (byte(p + 1)? as usize, byte(p + 2)? as usize);
                        p += 3;
                        len = ((b0 & 0xf) << 4 | b1 >> 4) + 0x11;
                        dist = ((b1 & 0xf) << 8 | b2) + 1;
                    }
                    1 => {
                        let (b1, b2, b3) = (byte(p + 1)? as usize, byte(p + 2)? as usize, byte(p + 3)? as usize);
                        p += 4;
                        len = ((b0 & 0xf) << 12 | b1 << 4 | b2 >> 4) + 0x111;
                        dist = ((b2 & 0xf) << 8 | b3) + 1;
                    }
                    hi => {
                        let b1 = byte(p + 1)? as usize;
                        p += 2;
                        len = hi + 1;
                        dist = ((b0 & 0xf) << 8 | b1) + 1;
                    }
                }
            }
            if dist > o.len() {
                return Err(Error::Lz77 { at: p });
            }
            for _ in 0..len {
                if o.len() >= n {
                    break;
                }
                let c = o[o.len() - dist];
                o.push(c);
            }
        }
    }
    Ok(o)
}

/// A body as stored, or its decompressed form when it is LZ77.
pub fn unpack(body: &[u8]) -> Result<Cow<'_, [u8]>> {
    if is_lz77(body) {
        Ok(Cow::Owned(lz77_decompress(body)?))
    } else {
        Ok(Cow::Borrowed(body))
    }
}

/// The export name Petey's file carries and that identifies it (section 1 step 4).
pub const PETEY_MARKER: &[u8] = b"wpnHitDataSet_ArmLeft";

/// Does this leaf look like a boss parameter file: its first u32 equals its own length, its header is consistent,
/// and (when given) `marker` occurs in it?
pub fn looks_like_boss_file(d: &[u8], marker: Option<&[u8]>) -> bool {
    let Ok(size) = be32(d, 0) else { return false };
    if size as usize != d.len() || crate::file::Header::parse(d).is_err() {
        return false;
    }
    match marker {
        None => true,
        Some(m) => !m.is_empty() && d.windows(m.len()).any(|w| w == m),
    }
}

/// Walk an archive (possibly LZ77-compressed, with nested ARCs) and return the first leaf for which `want` is true.
/// Leaves are visited depth-first in entry order. Malformed nested entries are skipped; a malformed top level is an
/// error.
pub fn find_leaf(top: &[u8], want: &dyn Fn(&[u8]) -> bool) -> Result<Vec<u8>> {
    let top = unpack(top)?;
    if !is_arc(&top) {
        return if want(&top) { Ok(top.into_owned()) } else { Err(Error::NotFound) };
    }
    let mut found = None;
    walk(&top, 0, want, &mut found)?;
    found.ok_or(Error::NotFound)
}

fn walk(d: &[u8], depth: u32, want: &dyn Fn(&[u8]) -> bool, found: &mut Option<Vec<u8>>) -> Result<()> {
    for e in arc_entries(d)? {
        if found.is_some() {
            return Ok(());
        }
        let Ok(body) = unpack(e.body) else { continue };
        if is_arc(&body) {
            if depth + 1 < MAX_DEPTH {
                let _ = walk(&body, depth + 1, want, found);
            }
        } else if want(&body) {
            *found = Some(body.into_owned());
        }
    }
    Ok(())
}

/// Petey's boss file from the bytes of the boss stage archive (`stage/adventure/030101.pac`).
pub fn find_petey_file(stage_archive: &[u8]) -> Result<Vec<u8>> {
    find_leaf(stage_archive, &|b| looks_like_boss_file(b, Some(PETEY_MARKER)))
}

/// Builders for synthetic test inputs: an ARC container and an LZ77 stream. Not used on the player's files.
pub mod build {
    use alloc::vec::Vec;
    /// An ARC with the given (type, id, body) entries.
    pub fn arc(entries: &[(u16, u16, &[u8])]) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"ARC\0");
        o.extend_from_slice(&0x0101u16.to_be_bytes());
        o.extend_from_slice(&(entries.len() as u16).to_be_bytes());
        o.resize(0x40, 0);
        for (t, id, b) in entries {
            let start = o.len();
            o.extend_from_slice(&t.to_be_bytes());
            o.extend_from_slice(&id.to_be_bytes());
            o.extend_from_slice(&(b.len() as u32).to_be_bytes());
            o.resize(start + 0x20, 0);
            o.extend_from_slice(b);
            while o.len() % 32 != 0 {
                o.push(0);
            }
        }
        o
    }
    /// A type-0x10 LZ77 stream; runs of the previous byte become distance-1 back-references so the copy path is
    /// exercised too.
    pub fn lz77(d: &[u8]) -> Vec<u8> {
        let mut o = alloc::vec![0x10, d.len() as u8, (d.len() >> 8) as u8, (d.len() >> 16) as u8];
        let mut i = 0;
        while i < d.len() {
            let fpos = o.len();
            o.push(0);
            for bit in 0..8 {
                if i >= d.len() {
                    break;
                }
                let mut run = 0;
                while i > 0 && i + run < d.len() && d[i + run] == d[i - 1] && run < 18 {
                    run += 1;
                }
                if run >= 3 {
                    o[fpos] |= 0x80 >> bit;
                    o.push(((run - 3) << 4) as u8);
                    o.push(0);
                    i += run;
                } else {
                    o.push(d[i]);
                    i += 1;
                }
            }
        }
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lz_roundtrip() {
        let src: Vec<u8> = (0..3000u32).map(|i| if i % 100 < 40 { 7 } else { (i * 31 % 251) as u8 }).collect();
        let c = build::lz77(&src);
        assert_eq!(lz77_decompress(&c).unwrap(), src);
        assert!(c.len() < src.len());
    }
    #[test]
    fn lz_bad_distance() {
        assert!(matches!(lz77_decompress(&[0x10, 4, 0, 0, 0x80, 0x00, 0x00]), Err(Error::Lz77 { .. })));
    }
    #[test]
    fn lz11_long() {
        // type 0x11: literal 'a', then a 4-byte reference of 0x111 + 0x1B = 300 bytes at distance 1
        let s = [0x11, 45, 1, 0, 0x40, b'a', 0x10, 0x01, 0xB0, 0x00];
        let o = lz77_decompress(&s).unwrap();
        assert_eq!(o.len(), 301);
        assert!(o.iter().all(|&c| c == b'a'));
    }
    #[test]
    fn arc_walk() {
        let leaf = b"hello";
        let inner = build::arc(&[(1, 92, leaf)]);
        let comp = build::lz77(&inner);
        let outer = build::arc(&[(1, 0, &comp), (1, 1, b"other")]);
        let e = arc_entries(&outer).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[1].body, b"other");
        assert_eq!(find_leaf(&outer, &|b| b == leaf).unwrap(), leaf);
        assert_eq!(find_leaf(&outer, &|b| b == b"nope"), Err(Error::NotFound));
    }
}
