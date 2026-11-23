//! VCDIFF decoder (RFC 3284), written from the RFC.
//!
//! Used by the Slippi menu layer to rebuild Slippi's patched menu files from the player's own disc
//! files plus Slippi's public `.diff` patches (fetched at run time, never shipped). Supported: the
//! default code table, `VCD_SOURCE` / `VCD_TARGET` windows, the xdelta3 `VCD_ADLER32` window
//! checksum (checked), and an application header (skipped). Not supported (rejected with an error):
//! secondary compressors (`VCD_DECOMPRESS`), custom code tables (`VCD_CODETABLE`) and compressed
//! sections in the delta indicator.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VcdiffError {
    BadMagic,
    Unsupported(&'static str),
    Truncated,
    Corrupt(&'static str),
    Checksum,
}

impl fmt::Display for VcdiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic => write!(f, "vcdiff: bad magic"),
            Self::Unsupported(s) => write!(f, "vcdiff: unsupported {s}"),
            Self::Truncated => write!(f, "vcdiff: truncated"),
            Self::Corrupt(s) => write!(f, "vcdiff: corrupt {s}"),
            Self::Checksum => write!(f, "vcdiff: window checksum mismatch"),
        }
    }
}

impl std::error::Error for VcdiffError {}

const VCD_DECOMPRESS: u8 = 0x01;
const VCD_CODETABLE: u8 = 0x02;
const VCD_APPHEADER: u8 = 0x04;
const VCD_SOURCE: u8 = 0x01;
const VCD_TARGET: u8 = 0x02;
const VCD_ADLER32: u8 = 0x04;

const NOOP: u8 = 0;
const ADD: u8 = 1;
const RUN: u8 = 2;
const COPY: u8 = 3;

#[derive(Clone, Copy, Default)]
struct Inst {
    kind: u8,
    size: u8,
    mode: u8,
}

/// The RFC 3284 section 5.6 default code table: 256 entries of up to two instructions each.
fn default_code_table() -> [[Inst; 2]; 256] {
    let mut t = [[Inst::default(); 2]; 256];
    let mut i = 0usize;
    t[i][0] = Inst { kind: RUN, size: 0, mode: 0 };
    i += 1;
    for s in 0..=17u8 {
        t[i][0] = Inst { kind: ADD, size: s, mode: 0 };
        i += 1;
    }
    for m in 0..9u8 {
        t[i][0] = Inst { kind: COPY, size: 0, mode: m };
        i += 1;
        for s in 4..=18u8 {
            t[i][0] = Inst { kind: COPY, size: s, mode: m };
            i += 1;
        }
    }
    for m in 0..6u8 {
        for a in 1..=4u8 {
            for c in 4..=6u8 {
                t[i] = [Inst { kind: ADD, size: a, mode: 0 }, Inst { kind: COPY, size: c, mode: m }];
                i += 1;
            }
        }
    }
    for m in 6..9u8 {
        for a in 1..=4u8 {
            t[i] = [Inst { kind: ADD, size: a, mode: 0 }, Inst { kind: COPY, size: 4, mode: m }];
            i += 1;
        }
    }
    for m in 0..9u8 {
        t[i] = [Inst { kind: COPY, size: 4, mode: m }, Inst { kind: ADD, size: 1, mode: 0 }];
        i += 1;
    }
    debug_assert_eq!(i, 256);
    t
}

struct Reader<'a> {
    d: &'a [u8],
    p: usize,
}

impl<'a> Reader<'a> {
    fn byte(&mut self) -> Result<u8, VcdiffError> {
        let b = *self.d.get(self.p).ok_or(VcdiffError::Truncated)?;
        self.p += 1;
        Ok(b)
    }
    fn varint(&mut self) -> Result<u64, VcdiffError> {
        let mut v: u64 = 0;
        for _ in 0..10 {
            let b = self.byte()?;
            v = v.checked_shl(7).ok_or(VcdiffError::Corrupt("integer"))? | u64::from(b & 0x7f);
            if v > u64::from(u32::MAX) * 4 {
                return Err(VcdiffError::Corrupt("integer"));
            }
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(VcdiffError::Corrupt("integer"))
    }
    fn usize(&mut self) -> Result<usize, VcdiffError> {
        usize::try_from(self.varint()?).map_err(|_| VcdiffError::Corrupt("integer"))
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], VcdiffError> {
        let end = self.p.checked_add(n).ok_or(VcdiffError::Truncated)?;
        let s = self.d.get(self.p..end).ok_or(VcdiffError::Truncated)?;
        self.p = end;
        Ok(s)
    }
    fn done(&self) -> bool {
        self.p >= self.d.len()
    }
}

/// Address cache (RFC 3284 section 5.1) with the default near (4) and same (3) sizes.
struct AddrCache {
    near: [u64; 4],
    next: usize,
    same: [u64; 3 * 256],
}

impl AddrCache {
    fn new() -> Self {
        Self { near: [0; 4], next: 0, same: [0; 768] }
    }
    fn update(&mut self, a: u64) {
        self.near[self.next] = a;
        self.next = (self.next + 1) % 4;
        self.same[(a % 768) as usize] = a;
    }
    fn decode(&mut self, here: u64, mode: u8, r: &mut Reader<'_>) -> Result<u64, VcdiffError> {
        let a = match mode {
            0 => r.varint()?,
            1 => here.checked_sub(r.varint()?).ok_or(VcdiffError::Corrupt("address"))?,
            2..=5 => self.near[(mode - 2) as usize] + r.varint()?,
            6..=8 => self.same[(mode as usize - 6) * 256 + r.byte()? as usize],
            _ => return Err(VcdiffError::Corrupt("mode")),
        };
        self.update(a);
        Ok(a)
    }
}

/// Adler-32 as xdelta3 writes it in the `VCD_ADLER32` window field.
pub fn adler32(d: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for c in d.chunks(5552) {
        for &x in c {
            a += u32::from(x);
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

/// Decode `delta` against `source`, returning the target file.
pub fn decode(source: &[u8], delta: &[u8]) -> Result<Vec<u8>, VcdiffError> {
    let mut r = Reader { d: delta, p: 0 };
    if r.take(3)? != [0xd6, 0xc3, 0xc4] {
        return Err(VcdiffError::BadMagic);
    }
    let _version = r.byte()?;
    let hdr = r.byte()?;
    if hdr & VCD_DECOMPRESS != 0 {
        return Err(VcdiffError::Unsupported("secondary compressor"));
    }
    if hdr & VCD_CODETABLE != 0 {
        return Err(VcdiffError::Unsupported("custom code table"));
    }
    if hdr & VCD_APPHEADER != 0 {
        let n = r.usize()?;
        r.take(n)?;
    }
    let table = default_code_table();
    let mut out: Vec<u8> = Vec::new();
    while !r.done() {
        let win = r.byte()?;
        if win & !(VCD_SOURCE | VCD_TARGET | VCD_ADLER32) != 0 || win & 3 == 3 {
            return Err(VcdiffError::Corrupt("window indicator"));
        }
        // (from_target, pos, end) of this window's source segment.
        let seg_ref: Option<(bool, usize, usize)> = if win & 3 != 0 {
            let len = r.usize()?;
            let pos = r.usize()?;
            let end = pos.checked_add(len).ok_or(VcdiffError::Corrupt("segment"))?;
            Some((win & VCD_TARGET != 0, pos, end))
        } else {
            None
        };
        let _delta_len = r.usize()?;
        let target_len = r.usize()?;
        if r.byte()? != 0 {
            return Err(VcdiffError::Unsupported("compressed sections"));
        }
        let data_len = r.usize()?;
        let inst_len = r.usize()?;
        let addr_len = r.usize()?;
        let checksum = if win & VCD_ADLER32 != 0 {
            let b = r.take(4)?;
            Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
        } else {
            None
        };
        let data = r.take(data_len)?;
        let inst = r.take(inst_len)?;
        let addr = r.take(addr_len)?;
        let tseg: Vec<u8>;
        let seg: &[u8] = match seg_ref {
            None => &[],
            Some((false, pos, end)) => source.get(pos..end).ok_or(VcdiffError::Corrupt("source segment"))?,
            Some((true, pos, end)) => {
                tseg = out.get(pos..end).ok_or(VcdiffError::Corrupt("target segment"))?.to_vec();
                &tseg
            }
        };
        let win_start = out.len();
        decode_window(&table, seg, data, inst, addr, target_len, &mut out)?;
        if let Some(c) = checksum {
            if adler32(&out[win_start..]) != c {
                return Err(VcdiffError::Checksum);
            }
        }
    }
    Ok(out)
}

fn decode_window(
    table: &[[Inst; 2]; 256],
    seg: &[u8],
    data: &[u8],
    inst: &[u8],
    addr: &[u8],
    target_len: usize,
    out: &mut Vec<u8>,
) -> Result<(), VcdiffError> {
    let base = out.len();
    out.reserve(target_len);
    let mut ir = Reader { d: inst, p: 0 };
    let mut dr = Reader { d: data, p: 0 };
    let mut ar = Reader { d: addr, p: 0 };
    let mut cache = AddrCache::new();
    let seg_len = seg.len() as u64;
    while !ir.done() {
        let code = ir.byte()?;
        for i in table[code as usize] {
            if i.kind == NOOP {
                continue;
            }
            let size = if i.size == 0 { ir.usize()? } else { i.size as usize };
            if out.len() - base + size > target_len {
                return Err(VcdiffError::Corrupt("window overflow"));
            }
            match i.kind {
                ADD => out.extend_from_slice(dr.take(size)?),
                RUN => {
                    let b = dr.byte()?;
                    out.resize(out.len() + size, b);
                }
                _ => {
                    let here = seg_len + (out.len() - base) as u64;
                    let a = cache.decode(here, i.mode, &mut ar)?;
                    if a >= here {
                        return Err(VcdiffError::Corrupt("copy address"));
                    }
                    for k in 0..size as u64 {
                        let p = a + k;
                        let b = if p < seg_len {
                            seg[p as usize]
                        } else {
                            out[base + (p - seg_len) as usize]
                        };
                        out.push(b);
                    }
                }
            }
        }
    }
    if out.len() - base != target_len {
        return Err(VcdiffError::Corrupt("window length"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn varint(mut v: u64, out: &mut Vec<u8>) {
        let mut tmp = vec![(v & 0x7f) as u8];
        v >>= 7;
        while v > 0 {
            tmp.push((v & 0x7f) as u8 | 0x80);
            v >>= 7;
        }
        tmp.reverse();
        out.extend(tmp);
    }

    enum Op<'a> {
        Add(&'a [u8]),
        Run(u8, usize),
        /// Copy `size` bytes from `addr` (mode 0, explicit size, opcode 19).
        Copy(u64, usize),
        /// Copy with mode 1 (HERE-relative).
        CopyHere(u64, usize),
    }

    /// A minimal encoder: one window, explicit-size opcodes (ADD=1, RUN=0, COPY mode0=19, mode1=35).
    fn encode(win: u8, seg: Option<(usize, usize)>, ops: &[Op<'_>], target: &[u8], adler: bool) -> Vec<u8> {
        let (mut data, mut inst, mut addr) = (Vec::new(), Vec::new(), Vec::new());
        let seg_len = seg.map_or(0, |s| s.0) as u64;
        let mut here = seg_len;
        for op in ops {
            match op {
                Op::Add(b) => {
                    inst.push(1);
                    varint(b.len() as u64, &mut inst);
                    data.extend_from_slice(b);
                    here += b.len() as u64;
                }
                Op::Run(b, n) => {
                    inst.push(0);
                    varint(*n as u64, &mut inst);
                    data.push(*b);
                    here += *n as u64;
                }
                Op::Copy(a, n) => {
                    inst.push(19);
                    varint(*n as u64, &mut inst);
                    varint(*a, &mut addr);
                    here += *n as u64;
                }
                Op::CopyHere(a, n) => {
                    inst.push(35);
                    varint(*n as u64, &mut inst);
                    varint(here - a, &mut addr);
                    here += *n as u64;
                }
            }
        }
        let mut body = Vec::new();
        varint(target.len() as u64, &mut body);
        body.push(0);
        varint(data.len() as u64, &mut body);
        varint(inst.len() as u64, &mut body);
        varint(addr.len() as u64, &mut body);
        if adler {
            body.extend(adler32(target).to_be_bytes());
        }
        body.extend(data);
        body.extend(inst);
        body.extend(addr);
        let mut out = vec![0xd6, 0xc3, 0xc4, 0, 0];
        out.push(win | if adler { VCD_ADLER32 } else { 0 });
        if let Some((len, pos)) = seg {
            varint(len as u64, &mut out);
            varint(pos as u64, &mut out);
        }
        varint(body.len() as u64, &mut out);
        out.extend(body);
        out
    }

    #[test]
    fn code_table_matches_rfc_landmarks() {
        let t = default_code_table();
        assert_eq!(t[0][0].kind, RUN);
        assert_eq!((t[1][0].kind, t[1][0].size), (ADD, 0));
        assert_eq!((t[18][0].kind, t[18][0].size), (ADD, 17));
        assert_eq!((t[19][0].kind, t[19][0].size, t[19][0].mode), (COPY, 0, 0));
        assert_eq!((t[162][0].kind, t[162][0].size, t[162][0].mode), (COPY, 18, 8));
        assert_eq!((t[163][0].kind, t[163][1].kind, t[163][1].size), (ADD, COPY, 4));
        assert_eq!((t[247][0].kind, t[247][0].size, t[247][1].kind, t[247][1].size), (COPY, 4, ADD, 1));
    }

    #[test]
    fn add_run_only() {
        let target = b"hello\0\0\0\0\0world";
        let d = encode(0, None, &[Op::Add(b"hello"), Op::Run(0, 5), Op::Add(b"world")], target, false);
        assert_eq!(decode(&[], &d).unwrap(), target);
    }

    #[test]
    fn copy_from_source_and_overlapping_target() {
        let source = b"0123456789abcdef";
        // "89ab" from source, then "XY", then a self-overlapping copy repeating "XY" 3 times.
        let target = b"89abXYXYXYXY";
        let d = encode(
            VCD_SOURCE,
            Some((source.len(), 0)),
            &[Op::Copy(8, 4), Op::Add(b"XY"), Op::CopyHere(source.len() as u64 + 4, 6)],
            target,
            true,
        );
        assert_eq!(decode(source, &d).unwrap(), target);
    }

    #[test]
    fn source_segment_offset() {
        let source = b"..........ABCDEFGH";
        let d = encode(VCD_SOURCE, Some((8, 10)), &[Op::Copy(2, 4)], b"CDEF", false);
        assert_eq!(decode(source, &d).unwrap(), b"CDEF");
    }

    #[test]
    fn two_windows_second_uses_target_segment() {
        let w1 = encode(0, None, &[Op::Add(b"abcdef")], b"abcdef", false);
        let w2 = encode(VCD_TARGET, Some((3, 2)), &[Op::Copy(0, 3), Op::Add(b"!")], b"cde!", false);
        let mut d = w1;
        d.extend_from_slice(&w2[5..]);
        assert_eq!(decode(&[], &d).unwrap(), b"abcdefcde!");
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(decode(&[], b"xyz\0\0"), Err(VcdiffError::BadMagic));
        assert_eq!(decode(&[], &[0xd6, 0xc3, 0xc4, 0, 1]), Err(VcdiffError::Unsupported("secondary compressor")));
        let mut d = encode(0, None, &[Op::Add(b"abc")], b"abc", true);
        let n = d.len();
        d[n - 4] ^= 1; // corrupt data byte "a" -> checksum fails
        assert!(decode(&[], &d).is_err());
        let d = encode(VCD_SOURCE, Some((4, 100)), &[Op::Copy(0, 4)], b"abcd", false);
        assert!(decode(b"abcd", &d).is_err());
        let d = encode(0, None, &[Op::Add(b"abc")], b"abc", false);
        assert_eq!(decode(&[], &d[..d.len() - 1]), Err(VcdiffError::Truncated));
    }

    #[test]
    fn adler32_known_value() {
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }
}
