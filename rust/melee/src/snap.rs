//! Reader for the C build's gameplay state dump (`MELEE_SNAP_DUMP`, pc/platform/gw_snap.c
//! `gw_snap_state_dump`; format in that file's comment): all of MEM1 plus the game's section-3
//! globals by symbol name. Everything after the little-endian header is the game's own
//! big-endian guest memory; a pointer is a guest address, readable with [`Snap::bytes`] when it
//! lies in MEM1 (0x80000000..) or inside a dumped global.
//!
//! Records are decoded with the GameCube offset tables of `melee_types` ([`Snap::decode`]): the
//! generated `GcDecode` impls read each field at its GC offset (scalars byte-swapped, bitfield
//! bytes kept in GC order) and, in the decoder's raw-pointer mode, leave every pointer as its guest
//! address in the native pointer slot, for the caller to follow with another read.

use melee_core::gcdecode::{GcDecode, GcDecoder};
use std::collections::{HashMap, HashSet};

pub const MEM1_BASE: u32 = 0x8000_0000;

pub struct Snap {
    /// The match frame the dump was taken after (the same frame as the golden CSV's row).
    pub frame: i32,
    mem: Vec<u8>,
    syms: HashMap<String, (u32, u32, u32)>,
    /// (va, len, blob offset) sorted by va.
    ranges: Vec<(u32, u32, u32)>,
    glob: Vec<u8>,
}

fn le32(d: &[u8], at: usize) -> Result<u32, String> {
    d.get(at..at + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).ok_or_else(|| "snapshot truncated".to_string())
}

impl Snap {
    pub fn load(path: &str) -> Result<Snap, String> {
        let d = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        if d.get(..8) != Some(b"MELSNAP1") {
            return Err(format!("{path}: not a MELSNAP1 state dump"));
        }
        let version = le32(&d, 8)?;
        if version != 1 {
            return Err(format!("{path}: dump version {version}"));
        }
        let frame = le32(&d, 12)? as i32;
        let base = le32(&d, 16)?;
        let msize = le32(&d, 20)? as usize;
        let npages = le32(&d, 24)? as usize;
        let nsyms = le32(&d, 28)? as usize;
        let glen = le32(&d, 32)? as usize;
        if base != MEM1_BASE || npages * 4096 != msize {
            return Err(format!("{path}: unexpected memory layout (base {base:#x}, {msize} bytes)"));
        }
        let mut p = 36;
        let mut syms = HashMap::new();
        let mut ranges = Vec::new();
        for _ in 0..nsyms {
            let va = le32(&d, p)?;
            let len = le32(&d, p + 4)?;
            let off = le32(&d, p + 8)?;
            let nl = u16::from_le_bytes(d.get(p + 12..p + 14).ok_or("snapshot truncated")?.try_into().unwrap()) as usize;
            let name = String::from_utf8_lossy(d.get(p + 14..p + 14 + nl).ok_or("snapshot truncated")?).into_owned();
            p += 14 + nl;
            ranges.push((va, len, off));
            syms.insert(name, (va, len, off));
        }
        ranges.sort();
        let bml = npages.div_ceil(8);
        let bm = d.get(p..p + bml).ok_or("snapshot truncated")?.to_vec();
        p += bml;
        let mut mem = vec![0u8; msize];
        for pg in 0..npages {
            if bm[pg >> 3] >> (pg & 7) & 1 != 0 {
                mem[pg * 4096..(pg + 1) * 4096].copy_from_slice(d.get(p..p + 4096).ok_or("snapshot truncated")?);
                p += 4096;
            }
        }
        let glob = d.get(p..p + glen).ok_or("snapshot truncated")?.to_vec();
        if p + glen != d.len() {
            return Err(format!("{path}: {} trailing bytes", d.len() - p - glen));
        }
        Ok(Snap { frame, mem, syms, ranges, glob })
    }

    /// `n` bytes at guest address `va`: MEM1 or inside one dumped global.
    pub fn bytes(&self, va: u32, n: usize) -> Option<&[u8]> {
        if va >= MEM1_BASE {
            let o = (va - MEM1_BASE) as usize;
            return self.mem.get(o..o.checked_add(n)?);
        }
        let i = self.ranges.partition_point(|r| r.0 <= va).checked_sub(1)?;
        let (rva, rlen, roff) = self.ranges[i];
        let o = (va - rva) as usize;
        if o + n > rlen as usize {
            return None;
        }
        let s = roff as usize + o;
        self.glob.get(s..s + n)
    }

    pub fn mapped(&self, va: u32, n: usize) -> bool {
        self.bytes(va, n).is_some()
    }

    fn must(&self, va: u32, n: usize) -> &[u8] {
        self.bytes(va, n).unwrap_or_else(|| panic!("snapshot: {n} bytes at guest {va:#x} are not in the dump"))
    }
    pub fn u8(&self, va: u32) -> u8 {
        self.must(va, 1)[0]
    }
    pub fn u16(&self, va: u32) -> u16 {
        u16::from_be_bytes(self.must(va, 2).try_into().unwrap())
    }
    pub fn u32(&self, va: u32) -> u32 {
        u32::from_be_bytes(self.must(va, 4).try_into().unwrap())
    }
    pub fn s32(&self, va: u32) -> i32 {
        self.u32(va) as i32
    }
    pub fn f32(&self, va: u32) -> f32 {
        f32::from_bits(self.u32(va))
    }

    /// The address of a dumped global (the map name with its leading underscore, e.g. `_seed`).
    pub fn sym(&self, name: &str) -> Option<u32> {
        self.syms.get(name).map(|s| s.0)
    }
    /// `sym`, also trying the game commons' `_gw_` form of the name.
    pub fn must_sym(&self, name: &str) -> Result<u32, String> {
        self.sym(name)
            .or_else(|| name.strip_prefix('_').and_then(|n| self.sym(&format!("_gw_{n}"))))
            .ok_or_else(|| format!("snapshot: global {name} is not in the dump"))
    }
    /// The guest pointer a `_gw_` pointer cell holds (`HSD_GObj** HSD_GObjPLinkHead` etc.).
    pub fn ptr_cell(&self, name: &str) -> Result<u32, String> {
        Ok(self.u32(self.must_sym(name)?))
    }

    /// The GameCube record at `va`, decoded with its generated `GcDecode` (raw-pointer mode).
    pub fn decode<T: GcDecode>(&self, va: u32) -> T {
        let relocs = melee_core::gcdecode::RelocSet::default();
        let mut d = GcDecoder::new_raw(self.must(va, T::GC_BYTES as usize), &relocs);
        T::gc_decode(&mut d, 0)
    }

    /// As [`Snap::decode`], but every pointer (and function pointer) decodes to null: for records
    /// the native sim keeps as plain data (its `Fighter`, the player table), where a guest address
    /// in a pointer slot would read as a live native pointer.
    pub fn decode_flat<T: GcDecode>(&self, va: u32) -> T {
        let relocs = melee_core::gcdecode::RelocSet::default();
        let mut d = GcDecoder::new(self.must(va, T::GC_BYTES as usize), &relocs);
        T::gc_decode(&mut d, 0)
    }

    /// `n` consecutive records.
    pub fn decode_n<T: GcDecode>(&self, va: u32, n: usize) -> Vec<T> {
        (0..n as u32).map(|i| self.decode::<T>(va + i * T::GC_BYTES)).collect()
    }

    /// The `HSD_GObj`s of `HSD_GObjPLinkHead[link]`, in list order.
    pub fn plink(&self, link: usize) -> Result<Vec<u32>, String> {
        let head = self.ptr_cell("_gw_HSD_GObjPLinkHead")?;
        let mut g = self.u32(head + 4 * link as u32);
        let mut v = Vec::new();
        while g != 0 {
            v.push(g);
            g = self.u32(g + 8); // next
            if v.len() > 4096 {
                return Err("snapshot: plink list does not end".into());
            }
        }
        Ok(v)
    }
}
