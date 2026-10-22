//! A `Space` is one side's view of the world: an arena (heap objects the world builder and the
//! callee stubs allocate) plus the registered globals of that side (the Rust statics, or the C
//! statics reached through the oracle library's registry). Pointers are compared by what they
//! point to (`norm`), never by address, so a Rust run and a C run hash identically when they did
//! the same thing.
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::collections::HashMap;
use std::sync::OnceLock;

use super::gen_tables::{BfEnt, GSpec, TYPES, TYPE_BF};

pub const ARENA_SIZE: usize = 8 << 20;

pub const RAW: usize = usize::MAX;
/// an array of pointers (every 8-byte slot is a pointer)
pub const PTRS: usize = usize::MAX - 1;

pub fn layout_of(ty: usize) -> (usize, &'static [usize]) {
    if ty == RAW {
        (0, &[])
    } else if ty == PTRS {
        (8, &[0])
    } else {
        (TYPES[ty].1, TYPES[ty].2)
    }
}

pub fn type_index(name: &str) -> usize {
    static MAP: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    let m = MAP.get_or_init(|| TYPES.iter().enumerate().map(|(i, t)| (t.0, i)).collect());
    *m.get(name).unwrap_or_else(|| panic!("type {name} not in TYPES"))
}

pub fn type_index_opt(name: &str) -> Option<usize> {
    static MAP: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    let m = MAP.get_or_init(|| TYPES.iter().enumerate().map(|(i, t)| (t.0, i)).collect());
    m.get(name).copied()
}

#[derive(Clone, Debug)]
pub struct Obj {
    pub off: usize,
    pub size: usize,
    /// index into TYPES, or RAW (no pointer slots)
    pub ty: usize,
    pub count: usize,
}

#[derive(Clone)]
pub struct Region {
    pub addr: *mut u8,
    pub size: usize,
    /// further Rust statics that mirror the same C symbol (kept in sync, never hashed)
    pub dups: Vec<(*mut u8, usize)>,
}

pub struct Fnv(pub u64);
impl Fnv {
    pub fn new() -> Self {
        Fnv(0xcbf29ce484222325)
    }
    pub fn u8(&mut self, b: u8) {
        self.0 ^= b as u64;
        self.0 = self.0.wrapping_mul(0x100000001b3);
    }
    pub fn u64(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.u8(b);
        }
    }
    pub fn bytes(&mut self, b: &[u8]) {
        for &x in b {
            self.u8(x);
        }
    }
    pub fn str(&mut self, s: &str) {
        self.bytes(s.as_bytes());
        self.u8(0);
    }
}

pub fn name_hash(s: &str) -> u64 {
    let mut f = Fnv::new();
    f.str(s);
    f.0 & 0x0FFF_FFFF_FFFF
}

pub struct Space {
    pub base: *mut u8,
    pub used: usize,
    pub objs: Vec<Obj>,
    pub regions: Vec<Region>,
    pub fn_names: HashMap<usize, u64>,
    layout: Layout,
}

impl Space {
    pub fn new() -> Self {
        let layout = Layout::from_size_align(ARENA_SIZE, 4096).unwrap();
        let base = unsafe { alloc_zeroed(layout) };
        Space { base, used: 64, objs: Vec::new(), regions: Vec::new(), fn_names: HashMap::new(), layout }
    }

    pub fn alloc(&mut self, size: usize, ty: usize, count: usize) -> *mut u8 {
        let tot = (size * count + 15) & !15;
        assert!(self.used + tot < ARENA_SIZE, "arena full");
        let off = self.used;
        self.used += tot.max(16);
        self.objs.push(Obj { off, size, ty, count });
        unsafe { self.base.add(off) }
    }

    /// Allocate `count` zeroed elements of the C type `name` (size from TYPES).
    pub fn alloc_ty(&mut self, name: &str, count: usize) -> *mut u8 {
        let t = type_index(name);
        self.alloc(TYPES[t].1, t, count)
    }

    pub fn alloc_raw(&mut self, size: usize) -> *mut u8 {
        self.alloc(size, RAW, 1)
    }

    pub fn in_arena(&self, v: usize) -> bool {
        let b = self.base as usize;
        v >= b && v < b + ARENA_SIZE
    }

    /// Pointer value -> position-independent tag.
    pub fn norm(&self, v: u64) -> u64 {
        if v == 0 {
            return 0;
        }
        let a = v as usize;
        let b = self.base as usize;
        if a >= b && a < b + ARENA_SIZE {
            return 0xA000_0000_0000_0000 | (a - b) as u64;
        }
        for (i, r) in self.regions.iter().enumerate() {
            let ra = r.addr as usize;
            if a >= ra && a < ra + r.size.max(1) {
                return 0xB000_0000_0000_0000 | ((i as u64) << 32) | (a - ra) as u64;
            }
            for &(d, ds) in &r.dups {
                let da = d as usize;
                if a >= da && a < da + ds.max(1) {
                    return 0xB000_0000_0000_0000 | ((i as u64) << 32) | (a - da) as u64;
                }
            }
        }
        if let Some(h) = self.fn_names.get(&a) {
            return 0xC000_0000_0000_0000 | h;
        }
        v
    }

    /// Inverse of `norm` for pointers into this space (used when copying a world).
    pub fn denorm(&self, t: u64) -> u64 {
        match t >> 60 {
            0xA => self.base as u64 + (t & 0x0FFF_FFFF_FFFF_FFFF),
            0xB => {
                let i = ((t >> 32) & 0xFFFFFFF) as usize;
                self.regions[i].addr as u64 + (t & 0xFFFF_FFFF)
            }
            _ => t,
        }
    }

    pub fn hash_slots(&self, h: &mut Fnv, addr: *const u8, len: usize, elem: usize, ptrs: &[usize]) {
        let mut buf = unsafe { std::slice::from_raw_parts(addr, len) }.to_vec();
        if elem > 0 {
            let n = len / elem;
            for e in 0..n {
                for &p in ptrs {
                    let at = e * elem + p;
                    if at + 8 <= len {
                        let v = u64::from_le_bytes(buf[at..at + 8].try_into().unwrap());
                        buf[at..at + 8].copy_from_slice(&self.norm(v).to_le_bytes());
                    }
                }
            }
        }
        h.bytes(&buf);
    }

    /// Hash of every non-input global and every arena object.
    pub fn state_hash(&self, specs: &[GSpec], h: &mut Fnv) {
        for (i, s) in specs.iter().enumerate() {
            if s.ro {
                continue;
            }
            let r = &self.regions[i];
            let n = s.cmp.min(r.size);
            h.str(s.name);
            self.hash_slots(h, r.addr, n, n.max(1), s.ptrs);
        }
        for o in &self.objs {
            h.u64(o.off as u64);
            let (elem, ptrs) = layout_of(o.ty);
            let len = o.size * o.count;
            self.hash_slots(h, unsafe { self.base.add(o.off) }, len, elem, ptrs);
        }
    }

    /// Locate the first difference between two spaces' states (for failure reports).
    pub fn describe_diff(&self, other: &Space, specs: &[GSpec]) -> String {
        let mut out = String::new();
        for (i, s) in specs.iter().enumerate() {
            if s.ro {
                continue;
            }
            let (ra, rb) = (&self.regions[i], &other.regions[i]);
            let n = s.cmp.min(ra.size).min(rb.size);
            let (mut ha, mut hb) = (Fnv::new(), Fnv::new());
            self.hash_slots(&mut ha, ra.addr, n, n.max(1), s.ptrs);
            other.hash_slots(&mut hb, rb.addr, n, n.max(1), s.ptrs);
            if ha.0 != hb.0 {
                let (a, b) = unsafe { (std::slice::from_raw_parts(ra.addr, n), std::slice::from_raw_parts(rb.addr, n)) };
                let first = (0..n).find(|&k| a[k] != b[k]).unwrap_or(0);
                out += &format!("global {} differs near byte {:#x} (rust {:02x?} c {:02x?})\n", s.name, first, &a[first..(first + 8).min(n)], &b[first..(first + 8).min(n)]);
            }
        }
        for (k, (oa, ob)) in self.objs.iter().zip(other.objs.iter()).enumerate() {
            if oa.off != ob.off || oa.size != ob.size {
                out += &format!("arena object {k} layout differs\n");
                break;
            }
            let (elem, ptrs) = layout_of(oa.ty);
            let len = oa.size * oa.count;
            let (mut ha, mut hb) = (Fnv::new(), Fnv::new());
            self.hash_slots(&mut ha, unsafe { self.base.add(oa.off) }, len, elem, ptrs);
            other.hash_slots(&mut hb, unsafe { other.base.add(ob.off) }, len, elem, ptrs);
            if ha.0 != hb.0 {
                let (a, b) = unsafe { (std::slice::from_raw_parts(self.base.add(oa.off), len), std::slice::from_raw_parts(other.base.add(ob.off), len)) };
                let first = (0..len).find(|&q| a[q] != b[q]).unwrap_or(0);
                let tn = if oa.ty == RAW { "raw" } else if oa.ty == PTRS { "ptrs" } else { TYPES[oa.ty].0 };
                out += &format!("arena obj {k} ({tn} at {:#x}, elem {}) differs at byte {:#x} (elem {} off {:#x}) rust {:02x?} c {:02x?}
", oa.off, oa.size, first, first / oa.size.max(1), first % oa.size.max(1), &a[first..(first + 8).min(len)], &b[first..(first + 8).min(len)]);
                break;
            }
        }
        if other.objs.len() != self.objs.len() {
            out += &format!("arena object counts differ: {} vs {}\n", self.objs.len(), other.objs.len());
        }
        out
    }

    /// Copy this world into `dst` (same arena offsets, pointers re-based, globals written into
    /// dst's registered regions).
    pub fn copy_world_to(&self, dst: &mut Space, specs: &[GSpec]) {
        // arena
        dst.used = self.used;
        dst.objs = self.objs.clone();
        unsafe { std::ptr::copy_nonoverlapping(self.base, dst.base, self.used) };
        for o in &self.objs {
            if o.ty == RAW {
                continue;
            }
            let (elem, ptrs) = layout_of(o.ty);
            for e in 0..o.count {
                for &p in ptrs {
                    let at = o.off + e * elem + p;
                    self.rebase_slot(dst, dst.base as usize + at);
                }
            }
        }
        // globals
        for (i, s) in specs.iter().enumerate() {
            let (rs, rd) = (&self.regions[i], &dst.regions[i]);
            let n = rs.size.min(rd.size);
            unsafe { std::ptr::copy_nonoverlapping(rs.addr, rd.addr, n) };
            for &p in s.ptrs {
                if p + 8 <= n {
                    self.rebase_slot(dst, rd.addr as usize + p);
                }
            }
        }
    }

    /// `slot` is an address in `dst` holding a pointer copied from `self`'s world.
    fn rebase_slot(&self, dst: &Space, slot: usize) {
        let v = unsafe { *(slot as *const u64) };
        let t = self.norm(v);
        let nv = match t >> 60 {
            0xA | 0xB if t != v => dst.denorm(t),
            _ => v,
        };
        unsafe { *(slot as *mut u64) = nv };
    }
}

// ---------------------------------------------------------------------------------------------
// Bitfield transcoding. The Rust records hold a bitfield run as GC-order bytes (a big-endian
// integer of `len` bytes at `phys`, the field at `shift`); the native C build packs the same
// fields LSB first at `nat`. The C side's memory is converted to the Rust form after a run (and
// the Rust world to the C form before it), so states compare by field value.
// ---------------------------------------------------------------------------------------------
unsafe fn le_get(base: *const u8, bit: u32, w: u8) -> u64 {
    let mut v = 0u64;
    for i in 0..w as u32 {
        let b = bit + i;
        if *base.add((b / 8) as usize) >> (b % 8) & 1 != 0 {
            v |= 1 << i;
        }
    }
    v
}
unsafe fn le_set(base: *mut u8, bit: u32, w: u8, v: u64) {
    for i in 0..w as u32 {
        let b = bit + i;
        let p = base.add((b / 8) as usize);
        if v >> i & 1 != 0 {
            *p |= 1 << (b % 8);
        } else {
            *p &= !(1 << (b % 8));
        }
    }
}
unsafe fn be_get(base: *const u8, e: &BfEnt) -> u64 {
    let mut v = 0u64;
    for i in 0..e.w as u32 {
        let j = e.shift as u32 + i; // bit j of the big-endian group integer
        let byte = e.phys as usize + e.len as usize - 1 - (j / 8) as usize;
        if *base.add(byte) >> (j % 8) & 1 != 0 {
            v |= 1 << i;
        }
    }
    v
}
unsafe fn be_set(base: *mut u8, e: &BfEnt, v: u64) {
    for i in 0..e.w as u32 {
        let j = e.shift as u32 + i;
        let p = base.add(e.phys as usize + e.len as usize - 1 - (j / 8) as usize);
        if v >> i & 1 != 0 {
            *p |= 1 << (j % 8);
        } else {
            *p &= !(1 << (j % 8));
        }
    }
}
unsafe fn zero_groups(base: *mut u8, ents: &[BfEnt]) {
    for e in ents {
        for k in 0..e.len as usize {
            *base.add(e.phys as usize + k) = 0;
        }
    }
}
/// native -> Rust form (in place)
pub unsafe fn bf_to_rust(base: *mut u8, ents: &[BfEnt]) {
    if ents.is_empty() {
        return;
    }
    let vals: Vec<u64> = ents.iter().map(|e| le_get(base, e.nat, e.w)).collect();
    zero_groups(base, ents);
    for (e, v) in ents.iter().zip(vals) {
        be_set(base, e, v);
    }
}
/// Rust -> native form (in place)
pub unsafe fn bf_to_native(base: *mut u8, ents: &[BfEnt]) {
    if ents.is_empty() {
        return;
    }
    let vals: Vec<u64> = ents.iter().map(|e| be_get(base, e)).collect();
    // clear every native storage bit of the run first, then write the fields
    for e in ents {
        le_set(base, e.nat, e.w, 0);
    }
    for (e, v) in ents.iter().zip(vals) {
        le_set(base, e.nat, e.w, v);
    }
}
/// drop bits that belong to no field (padding) from a Rust-form image
pub unsafe fn bf_clean_rust(base: *mut u8, ents: &[BfEnt]) {
    if ents.is_empty() {
        return;
    }
    let vals: Vec<u64> = ents.iter().map(|e| be_get(base, e)).collect();
    zero_groups(base, ents);
    for (e, v) in ents.iter().zip(vals) {
        be_set(base, e, v);
    }
}

impl Space {
    fn bf_each(&self, specs: &[GSpec], f: unsafe fn(*mut u8, &[BfEnt])) {
        for o in &self.objs {
            if o.ty == RAW || o.ty == PTRS || TYPE_BF[o.ty].is_empty() {
                continue;
            }
            for e in 0..o.count {
                unsafe { f(self.base.add(o.off + e * o.size), TYPE_BF[o.ty]) };
            }
        }
        for (i, s) in specs.iter().enumerate() {
            if !s.bf.is_empty() {
                unsafe { f(self.regions[i].addr, s.bf) };
            }
        }
    }
    pub fn bf_to_rust_all(&self, specs: &[GSpec]) {
        self.bf_each(specs, bf_to_rust)
    }
    pub fn bf_to_native_all(&self, specs: &[GSpec]) {
        self.bf_each(specs, bf_to_native)
    }
    pub fn bf_clean_rust_all(&self, specs: &[GSpec]) {
        self.bf_each(specs, bf_clean_rust)
    }
}

impl Drop for Space {
    fn drop(&mut self) {
        unsafe { dealloc(self.base, self.layout) };
    }
}
