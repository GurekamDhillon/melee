//! Field-access rewriting: guest (GameCube) loads/stores -> native little-endian Rust structs.
//!
//! Mod code addresses a `Fighter` as `lfs f1, 0x84(r31)`. The native port keeps a `Fighter` as
//! an ordinary Rust struct (8-byte pointers, little-endian, other offsets). [`GuestSpace`] gives
//! such a struct a guest address range of the record's GC size and implements the interpreter's
//! [`Mem`] by rewriting every access through the generated tables (build.rs `native.rs`: each
//! field's GC offset, native `offset_of!`, and native type):
//!
//!   * a scalar byte at GC offset `o` (big-endian byte `k` of a `w`-byte field) is native byte
//!     `w - 1 - k` of that field, so any access width, aligned or straddling fields, comes out as
//!     the big-endian value the GameCube would have read;
//!   * `bool` reads 0/1 and stores `v != 0` (never an invalid bool);
//!   * byte arrays (bitfield storage units `bf_*`, `u8[N]`) map byte for byte (their native
//!     storage keeps GameCube byte order, as melee_types documents);
//!   * a pointer field is accessed only as its whole 4-byte word: a load hands the guest a
//!     handle (the guest address of a region registered for that native pointer, registering one
//!     for a pointer to a known record on the way; a native pointer into a bytes region is the
//!     matching guest address there), a store maps a handle (or an address inside a bytes
//!     region) back to the native pointer; a partial pointer access or a store of anything else
//!     faults;
//!   * padding and fields without a native counterpart (GC 4-byte `size_t` vs native 8) live in a
//!     per-region shadow of GC bytes (padding) or fault (mismatched widths).
//!
//! Plain guest memory (the blob, its stack, mod-private data) is [`Backing::Bytes`].

use crate::ppc::{Fault, Mem};
use std::collections::HashMap;

pub struct Ty {
    pub gc: u32,
    pub nat: usize,
    pub kind: Kind,
}

pub enum Kind {
    Scalar,
    Bool,
    Ptr,
    /// Pointer to a record (GC name).
    PtrTo(&'static str),
    /// Nested record (GC name).
    Rec(&'static str),
    Arr(&'static Ty, usize),
}

pub struct NField {
    pub gc: u32,
    pub nat: usize,
    pub ty: &'static Ty,
}

mod gen {
    use super::{Kind, NField, Ty};
    include!(concat!(env!("OUT_DIR"), "/native.rs"));
}

/// Unions of per-owner variables kept as native 32-bit words (see [`Layouts::locate`]).
const WORD_UNIONS: [&str; 3] = ["Fighter_MotionVars", "Fighter_FighterVars", "Item_ItemVars"];

/// Native layout tables by GC record name.
pub struct Layouts {
    by_name: HashMap<&'static str, (usize, &'static [NField])>,
    gc_size: HashMap<&'static str, u32>,
}

/// Where one GameCube byte of a record lives natively.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loc {
    /// Native byte offset.
    Byte(usize),
    Bool(usize),
    /// Byte `k` of a pointer word whose native pointer is at this offset (+ pointee record).
    Ptr(usize, u32, Option<&'static str>),
    /// Padding: no field.
    Pad,
    /// A field whose native width differs from the GC one (not a pointer): not rewritable.
    Mismatch,
}

impl Default for Layouts {
    fn default() -> Self {
        Self::new()
    }
}

impl Layouts {
    pub fn new() -> Layouts {
        let mut by_name = HashMap::new();
        for &(n, sz, f) in gen::NATIVE {
            by_name.entry(n).or_insert((sz, f));
        }
        let gc_size = melee_types::GC_RECORDS.iter().map(|r| (r.0, r.1)).collect();
        Layouts { by_name, gc_size }
    }
    pub fn records(&self) -> usize {
        self.by_name.len()
    }
    pub fn has(&self, rec: &str) -> bool {
        self.by_name.contains_key(rec)
    }
    pub fn gc_size(&self, rec: &str) -> Option<u32> {
        self.gc_size.get(rec).copied()
    }
    pub fn native_size(&self, rec: &str) -> Option<usize> {
        self.by_name.get(rec).map(|r| r.0)
    }

    /// Native byte offset of the field (or nested field, or array element) that STARTS at GC offset `off` of `rec`:
    /// where a guest pointer to the interior of a native record points natively (`&fp->coll_data`).
    pub fn field_start(&self, rec: &str, off: u32) -> Option<usize> {
        if off == 0 {
            return Some(0);
        }
        if WORD_UNIONS.contains(&rec) {
            return (off % 4 == 0).then_some(off as usize);
        }
        let &(_, fields) = self.by_name.get(rec)?;
        let f = fields.iter().find(|f| off >= f.gc && off < f.gc + f.ty.gc.max(1))?;
        self.ty_start(f.ty, off - f.gc).map(|n| f.nat + n)
    }

    fn ty_start(&self, ty: &'static Ty, rel: u32) -> Option<usize> {
        if rel == 0 {
            return Some(0);
        }
        match ty.kind {
            Kind::Rec(r) => self.field_start(r, rel),
            Kind::Arr(e, _) => {
                let g = e.gc.max(1);
                self.ty_start(e, rel % g).map(|n| (rel / g) as usize * e.nat + n)
            }
            _ => None,
        }
    }

    /// Native location of GC byte `off` of record `rec`.
    pub fn locate(&self, rec: &str, off: u32) -> Loc {
        let Some(&(nsz, fields)) = self.by_name.get(rec) else { return Loc::Mismatch };
        // The per-character variable unions: their meaning is the owner's (an m-ex fighter's own layout), so they
        // are kept as native little-endian 32-bit words. A guest float or int there reads back as stored, and a native
        // function the guest hands a pointer into one (a Vec3 velocity in fp->mv) sees native floats.
        if WORD_UNIONS.contains(&rec) {
            let w = (off & !3) as usize + 3 - (off & 3) as usize;
            return if w < nsz { Loc::Byte(w) } else { Loc::Pad };
        }
        match fields.iter().find(|f| off >= f.gc && off < f.gc + f.ty.gc.max(1)) {
            Some(f) => self.ty_locate(f.ty, off - f.gc, f.nat),
            None => Loc::Pad,
        }
    }

    fn ty_locate(&self, ty: &'static Ty, rel: u32, nat: usize) -> Loc {
        match ty.kind {
            Kind::Scalar if ty.gc as usize == ty.nat => Loc::Byte(nat + (ty.nat - 1 - rel as usize)),
            Kind::Scalar => Loc::Mismatch,
            Kind::Bool => Loc::Bool(nat),
            Kind::Ptr => Loc::Ptr(nat, rel, None),
            Kind::PtrTo(r) => Loc::Ptr(nat, rel, Some(r)),
            Kind::Rec(r) => match self.locate(r, rel) {
                Loc::Byte(b) => Loc::Byte(nat + b),
                Loc::Bool(b) => Loc::Bool(nat + b),
                Loc::Ptr(b, k, p) => Loc::Ptr(nat + b, k, p),
                l => l,
            },
            Kind::Arr(e, _) => {
                let i = (rel / e.gc.max(1)) as usize;
                self.ty_locate(e, rel % e.gc.max(1), nat + i * e.nat)
            }
        }
    }
}

pub enum Backing {
    /// Plain guest bytes (big-endian values).
    Bytes(Vec<u8>),
    /// A native struct of GC record `rec`; `shadow` holds the GC bytes of its padding.
    Native { rec: &'static str, ptr: *mut u8, shadow: Vec<u8> },
    /// Plain guest bytes (big-endian values) the host owns at `ptr` (a buffer of the game heap the
    /// guest keeps its own data in, e.g. Fighter.dat_attrs_backup, so a rollback restores it).
    Raw { ptr: *mut u8 },
}

pub struct Region {
    pub base: u32,
    pub size: u32,
    pub backing: Backing,
}

/// One rewritten access, for traces and tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rewrite {
    pub rec: &'static str,
    pub gc_off: u32,
    pub size: u32,
    pub store: bool,
    pub ea: u32,
}

/// Where auto-registered native objects get their guest addresses.
const AUTO_BASE: u32 = 0x9000_0000;
/// Elements of an auto-registered object's array the guest may reach.
const ARRAY_REACH: u32 = 64;

/// The guest address span an auto-registered object of GC size `size` reserves for its array: ARRAY_REACH elements,
/// and at least 0x2000 bytes (Dedede's fp->parts has more than 64 FighterBones: bone 65 read the next object's
/// region, a JObj's class_info, and lb_8000B1CC faulted on it).
fn reach(size: u32) -> u32 {
    (size.max(0x10) * ARRAY_REACH).max(0x2000)
}

pub struct GuestSpace {
    pub layouts: Layouts,
    pub regions: Vec<Region>,
    /// Next guest address handed to an auto-registered native object.
    next: u32,
    pub trace: Option<Vec<Rewrite>>,
    /// Guest-only values of native pointer fields, by (region base, GC offset): what the guest
    /// reads there instead of a handle for the native pointer, which stays as it is. For an
    /// object the host keeps outside the native struct (the native match leaves `Fighter.ft_data`
    /// null and reads the fighter's ftData from its DAT bytes, so the guest gets those bytes'
    /// guest address). A guest store to the field drops the override.
    pub ptr_override: HashMap<(u32, u32), u32>,
    /// Guest code addresses as native function pointers (an executing host's trampolines): a
    /// guest store of a code address into a pointer field stores what this gives, and a load of
    /// such a native pointer gives the guest address back (`fn_guest`).
    pub fn_native: Option<fn(u32) -> Option<usize>>,
    /// The record an untyped (`void*`) pointer field holds, when the host can tell (`HSD_GObj.user_data` by the
    /// gobj's class, `HSD_GObj.hsd_obj` a joint): (owner record, GC offset, owner object, the pointer).
    pub void_pointee: Option<fn(&'static str, u32, *mut u8, usize) -> Option<&'static str>>,
    pub fn_guest: Option<fn(usize) -> Option<u32>>,
    /// A guest address stored into a typed pointer field that names no native object of that record (inside another
    /// object: Tails' `gen->userfunc = &fp->fv + 0x64`, guest hook words a `_PSUserFunc` is made of): the native object
    /// the host stands in for it, as (space, guest address, pointee record).
    pub ptr_native: Option<fn(&GuestSpace, u32, &'static str) -> Option<usize>>,
    /// Native objects the space made itself (`materialize`).
    owned: Vec<Box<[u64]>>,
}

impl GuestSpace {
    pub fn new() -> GuestSpace {
        GuestSpace { layouts: Layouts::new(), regions: vec![], next: AUTO_BASE, trace: None, ptr_override: HashMap::new(), fn_native: None, fn_guest: None, void_pointee: None, ptr_native: None, owned: Vec::new() }
    }

    /// Plain guest memory at `base` (code, stack, mod data).
    pub fn add_bytes(&mut self, base: u32, bytes: Vec<u8>) {
        let size = bytes.len() as u32;
        self.regions.push(Region { base, size, backing: Backing::Bytes(bytes) });
    }

    /// Give the native object at `ptr` (a `rec` as melee_types defines it) the guest address
    /// `base`, or a fresh one when `base` is None. Returns the guest address.
    ///
    /// # Safety
    /// `ptr` must point to a live, writable native `rec` for as long as the space is used, and
    /// every pointer field reachable from it that the guest follows must be valid or null.
    pub unsafe fn add_native(&mut self, base: Option<u32>, rec: &'static str, ptr: *mut u8) -> Result<u32, String> {
        let size = self.layouts.gc_size(rec).ok_or_else(|| format!("no GC record {rec}"))?;
        if !self.layouts.has(rec) {
            return Err(format!("no native layout for {rec}"));
        }
        let base = base.unwrap_or_else(|| {
            let b = self.next;
            // room for ARRAY_REACH elements: a pointer the guest indexes (fp->x5E8 parts, hurtboxes) reaches the
            // next ones through `extend`
            self.next = self.next.wrapping_add((reach(size) + 0x1F) & !0xF);
            b
        });
        self.regions.push(Region { base, size, backing: Backing::Native { rec, ptr, shadow: vec![0; size as usize] } });
        Ok(base)
    }

    /// Plain guest bytes the host owns: `len` bytes at native `ptr`, at guest address `base`.
    ///
    /// # Safety
    /// `ptr..ptr+len` must stay valid and writable for as long as the space is used.
    pub unsafe fn add_raw(&mut self, base: u32, ptr: *mut u8, len: u32) {
        self.regions.push(Region { base, size: len, backing: Backing::Raw { ptr } });
    }

    /// Drop every region registered for the native object at `ptr` (it is about to be freed).
    pub fn forget_native(&mut self, ptr: *mut u8) {
        let gone: Vec<u32> = self.regions.iter().filter(|r| matches!(r.backing, Backing::Native { ptr: q, .. } if q == ptr)).map(|r| r.base).collect();
        self.regions.retain(|r| !matches!(r.backing, Backing::Native { ptr: q, .. } if q == ptr));
        self.ptr_override.retain(|k, _| !gone.contains(&k.0));
    }

    /// For diagnostics: the record and GC offset of guest address `h` inside a registered native object.
    pub fn describe(&self, h: u32) -> Option<(&'static str, u32)> {
        let r = self.regions.iter().rev().filter(|r| h >= r.base && h < r.base + reach(r.size)).max_by_key(|r| r.base)?;
        match &r.backing {
            Backing::Native { rec, .. } => Some((rec, h - r.base)),
            _ => None,
        }
    }

    /// For diagnostics: every registered native object whose reserved span holds `h` (base, GC size, record).
    pub fn regions_at(&self, h: u32) -> Vec<(u32, u32, &'static str)> {
        self.regions
            .iter()
            .filter_map(|r| match &r.backing {
                Backing::Native { rec, .. } if h >= r.base && h < r.base + reach(r.size) => Some((r.base, r.size, *rec)),
                _ => None,
            })
            .collect()
    }

    /// The guest address of a registered native object (None: not registered).
    pub fn handle_of(&self, ptr: *mut u8) -> Option<u32> {
        self.regions.iter().find(|r| matches!(r.backing, Backing::Native { ptr: q, .. } if q == ptr)).map(|r| r.base)
    }

    /// The guest handle for native pointer `p` (registering a region for a pointer to a known
    /// record `pointee` on the way); see [`Self::native`] for the inverse.
    pub fn handle(&mut self, p: usize, pointee: Option<&'static str>) -> Result<u32, Fault> {
        self.handle_for(p, pointee)
    }

    /// The native address guest address `h` stands for: a registered object's pointer, or the
    /// host byte inside a bytes region.
    pub fn native(&self, h: u32) -> Option<usize> {
        self.native_of(h)
    }

    /// The native address of guest address `h` inside a registered native object (a field or element that starts
    /// there), None when `h` is not inside one or not at a field start.
    pub fn interior(&self, h: u32) -> Option<usize> {
        let r = self.regions.iter().find(|r| h >= r.base && h < r.base + r.size)?;
        match &r.backing {
            Backing::Native { rec, ptr, .. } => self.layouts.field_start(rec, h - r.base).map(|n| *ptr as usize + n),
            _ => None,
        }
    }

    pub fn bytes(&self, base: u32) -> Option<&[u8]> {
        self.regions.iter().find(|r| r.base == base).and_then(|r| match &r.backing {
            Backing::Bytes(b) => Some(&b[..]),
            _ => None,
        })
    }

    /// `find`, registering the element of an auto-registered native object's array that `ea` falls in (the guest
    /// indexes past the first element of what a pointer field points to).
    fn find_mut(&mut self, ea: u32, n: u32) -> Result<usize, Fault> {
        if let Ok(i) = self.find(ea, n) {
            return Ok(i);
        }
        // the object whose span holds `ea` nearest below it: an element region the guest reached earlier reserves a
        // span too, which can run over the next object (Tails' costume list element 39 over the costume array)
        let hit = self
            .regions
            .iter()
            .filter_map(|r| match &r.backing {
                Backing::Native { rec, ptr, .. } if r.base >= AUTO_BASE && ea >= r.base && ea < r.base + reach(r.size) => Some((r.base, r.size, *rec, *ptr)),
                _ => None,
            })
            .max_by_key(|h| h.0);
        let Some((base, gs, rec, ptr)) = hit else { return Err(Fault::Mem(ea)) };
        let k = (ea - base) / gs.max(1);
        let ns = self.layouts.native_size(rec).ok_or(Fault::Mem(ea))?;
        // SAFETY: add_native's contract, extended to the array the guest walks (as the C would read it).
        unsafe { self.add_native(Some(base + k * gs), rec, ptr.add(k as usize * ns)) }.map_err(|_| Fault::Mem(ea))?;
        self.find(ea, n)
    }

    fn find(&self, ea: u32, n: u32) -> Result<usize, Fault> {
        self.regions
            .iter()
            .position(|r| ea >= r.base && ea as u64 + n as u64 <= r.base as u64 + r.size as u64)
            .ok_or(Fault::Mem(ea))
    }

    /// The guest handle for a native pointer read out of a field (null -> 0).
    fn handle_for(&mut self, p: usize, pointee: Option<&'static str>) -> Result<u32, Fault> {
        if p == 0 {
            return Ok(0);
        }
        if let Some(g) = self.fn_guest.and_then(|f| f(p)) {
            return Ok(g);
        }
        // a native object registered as the record the caller names: the same address can stand for another record
        // (a freed object's memory reused for another type, a record at offset 0 of another), whose layout would
        // translate the guest's field offsets wrongly
        if let Some(want) = pointee.filter(|r| self.layouts.has(r)) {
            if let Some(r) = self.regions.iter().find(|r| matches!(&r.backing, Backing::Native { ptr, rec, .. } if *ptr as usize == p && *rec == want)) {
                return Ok(r.base);
            }
            if self.regions.iter().any(|r| matches!(&r.backing, Backing::Native { ptr, .. } if *ptr as usize == p)) {
                // SAFETY: GuestSpace::add_native's contract: pointers reachable from a registered object are valid.
                return unsafe { self.add_native(None, want, p as *mut u8) }.map_err(|_| Fault::Mem(0));
            }
        }
        for r in &self.regions {
            match &r.backing {
                Backing::Native { ptr, .. } if *ptr as usize == p => return Ok(r.base),
                // A native pointer into a bytes region (a buffer the host handed a void* field,
                // e.g. Fighter.dat_attrs_backup) is that region's guest address.
                Backing::Bytes(b) if p >= b.as_ptr() as usize && p < b.as_ptr() as usize + b.len() => {
                    return Ok(r.base + (p - b.as_ptr() as usize) as u32);
                }
                Backing::Raw { ptr } if p >= *ptr as usize && p < *ptr as usize + r.size as usize => {
                    return Ok(r.base + (p - *ptr as usize) as u32);
                }
                _ => {}
            }
        }
        match pointee {
            // SAFETY: GuestSpace::add_native's contract: pointers reachable from a registered
            // object are valid.
            Some(rec) if self.layouts.has(rec) => unsafe { self.add_native(None, rec, p as *mut u8) }.map_err(|_| Fault::Mem(0)),
            _ => Err(Fault::Mem(0)),
        }
    }

    fn native_of(&self, h: u32) -> Option<usize> {
        if h == 0 {
            return Some(0);
        }
        if let Some(r) = self.regions.iter().find(|r| r.base == h) {
            if let Backing::Native { ptr, .. } = r.backing {
                return Some(ptr as usize);
            }
        }
        // A guest address inside a bytes region -> the native address of that byte (the Vec's
        // heap buffer; regions never resize).
        self.regions.iter().find(|r| h >= r.base && h < r.base + r.size).and_then(|r| match &r.backing {
            Backing::Bytes(b) => Some(b.as_ptr() as usize + (h - r.base) as usize),
            Backing::Raw { ptr } => Some(*ptr as usize + (h - r.base) as usize),
            _ => None,
        })
    }

    fn load(&mut self, ea: u32, n: u32) -> Result<u64, Fault> {
        let i = self.find_mut(ea, n)?;
        let r = &self.regions[i];
        let o = ea - r.base;
        match &r.backing {
            Backing::Bytes(b) => Ok(b[o as usize..(o + n) as usize].iter().fold(0u64, |v, &x| v << 8 | x as u64)),
            // SAFETY: add_raw's contract.
            Backing::Raw { ptr } => Ok((0..n).fold(0u64, |v, k| v << 8 | unsafe { *ptr.add((o + k) as usize) } as u64)),
            Backing::Native { rec, ptr, shadow } => {
                let (rec, ptr) = (*rec, *ptr);
                if let Some(t) = &mut self.trace {
                    t.push(Rewrite { rec, gc_off: o, size: n, store: false, ea });
                }
                if let Loc::Ptr(nat, 0, pointee) = self.layouts.locate(rec, o) {
                    if n != 4 {
                        return Err(Fault::Mem(ea));
                    }
                    if let Some(&h) = self.ptr_override.get(&(r.base, o)) {
                        return Ok(h as u64);
                    }
                    // SAFETY: add_native's contract; nat is inside the native object.
                    let p = unsafe { (ptr.add(nat) as *const usize).read_unaligned() };
                    // the host's answer first: a void* field's pointee, or a union's member by its owner's tag
                    let pointee = self.void_pointee.and_then(|f| f(rec, o, ptr, p)).or(pointee);
                    return self.handle_for(p, pointee).map(|h| h as u64).map_err(|_| Fault::Mem(ea));
                }
                let mut v = 0u64;
                for k in 0..n {
                    let byte = match self.layouts.locate(rec, o + k) {
                        // SAFETY: as above.
                        Loc::Byte(b) => unsafe { *ptr.add(b) },
                        Loc::Bool(b) => unsafe { (*ptr.add(b) != 0) as u8 },
                        Loc::Pad => shadow[(o + k) as usize],
                        Loc::Ptr(..) | Loc::Mismatch => return Err(Fault::Mem(ea)),
                    };
                    v = v << 8 | byte as u64;
                }
                Ok(v)
            }
        }
    }

    /// Is guest address `h` plain guest bytes (no object registered there) that a typed pointer field can only hold
    /// as a native `rec`?
    fn materialize_ok(&self, h: u32, rec: &str) -> bool {
        if h == 0 || !self.layouts.has(rec) || self.layouts.gc_size(rec).is_none() {
            return false;
        }
        match self.find(h, 1) {
            Ok(i) => matches!(self.regions[i].backing, Backing::Bytes(_) | Backing::Raw { .. }),
            Err(_) => false,
        }
    }

    /// A native `rec` made from the guest bytes at `h` (a record of the mod's own data, e.g. its ftData's attribute
    /// block, stored into a typed pointer field the native code follows): decoded through the layout tables, pointer
    /// words translated, and registered at `h` in front of the bytes, so the guest's later accesses there reach the
    /// same native object. The object lives as long as the space (`owned`).
    fn materialize(&mut self, h: u32, rec: &'static str) -> Option<usize> {
        let gs = self.layouts.gc_size(rec)?;
        let ns = self.layouts.native_size(rec)?;
        let bytes: Vec<Option<u8>> = (0..gs).map(|k| self.load(h + k, 1).ok().map(|v| v as u8)).collect();
        let mut buf = vec![0u64; ns.div_ceil(8).max(1)].into_boxed_slice();
        let ptr = buf.as_mut_ptr() as *mut u8;
        self.owned.push(buf);
        self.regions.insert(0, Region { base: h, size: gs, backing: Backing::Native { rec, ptr, shadow: vec![0; gs as usize] } });
        let mut k = 0;
        while k < gs {
            match self.layouts.locate(rec, k) {
                Loc::Ptr(_, 0, _) if k + 4 <= gs => {
                    let w = (0..4).fold(0u32, |a, j| a << 8 | bytes[(k + j) as usize].unwrap_or(0) as u32);
                    let _ = self.store(h + k, 4, w as u64);
                    k += 4;
                }
                _ => {
                    if let Some(b) = bytes[k as usize] {
                        let _ = self.store(h + k, 1, b as u64);
                    }
                    k += 1;
                }
            }
        }
        Some(ptr as usize)
    }

    fn store(&mut self, ea: u32, n: u32, v: u64) -> Result<(), Fault> {
        let i = self.find_mut(ea, n)?;
        let o = ea - self.regions[i].base;
        let (rec, ptr) = match &mut self.regions[i].backing {
            Backing::Bytes(b) => {
                for k in 0..n {
                    b[(o + k) as usize] = (v >> (8 * (n - 1 - k))) as u8;
                }
                return Ok(());
            }
            Backing::Raw { ptr } => {
                for k in 0..n {
                    // SAFETY: add_raw's contract.
                    unsafe { *ptr.add((o + k) as usize) = (v >> (8 * (n - 1 - k))) as u8 };
                }
                return Ok(());
            }
            Backing::Native { rec, ptr, .. } => (*rec, *ptr),
        };
        if let Some(t) = &mut self.trace {
            t.push(Rewrite { rec, gc_off: o, size: n, store: true, ea });
        }
        if let Loc::Ptr(nat, 0, pointee) = self.layouts.locate(rec, o) {
            if n != 4 {
                return Err(Fault::Mem(ea));
            }
            let p = match self.fn_native.and_then(|f| f(v as u32)) {
                Some(p) => p,
                None => match pointee {
                    Some(r) if self.materialize_ok(v as u32, r) => self.materialize(v as u32, r).ok_or(Fault::Mem(ea))?,
                    Some(r) => match self.native_of(v as u32) {
                        Some(p) => p,
                        None => self.ptr_native.and_then(|f| f(self, v as u32, r)).ok_or(Fault::Mem(ea))?,
                    },
                    None => self.native_of(v as u32).ok_or(Fault::Mem(ea))?,
                },
            };
            let base = self.regions[i].base;
            self.ptr_override.remove(&(base, o));
            // SAFETY: add_native's contract.
            unsafe { (ptr.add(nat) as *mut usize).write_unaligned(p) };
            return Ok(());
        }
        // Check every byte first so a faulting store writes nothing.
        let locs: Vec<Loc> = (0..n).map(|k| self.layouts.locate(rec, o + k)).collect();
        if locs.iter().any(|l| matches!(l, Loc::Ptr(..) | Loc::Mismatch)) {
            return Err(Fault::Mem(ea));
        }
        for (k, l) in locs.into_iter().enumerate() {
            let byte = (v >> (8 * (n as usize - 1 - k))) as u8;
            match l {
                // SAFETY: add_native's contract.
                Loc::Byte(b) => unsafe { *ptr.add(b) = byte },
                Loc::Bool(b) => unsafe { *ptr.add(b) = (byte != 0) as u8 },
                Loc::Pad => {
                    if let Backing::Native { shadow, .. } = &mut self.regions[i].backing {
                        shadow[o as usize + k] = byte;
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }
}

impl Default for GuestSpace {
    fn default() -> Self {
        Self::new()
    }
}

impl Mem for GuestSpace {
    fn ld8(&mut self, ea: u32) -> Result<u8, Fault> {
        self.load(ea, 1).map(|v| v as u8)
    }
    fn ld16(&mut self, ea: u32) -> Result<u16, Fault> {
        self.load(ea, 2).map(|v| v as u16)
    }
    fn ld32(&mut self, ea: u32) -> Result<u32, Fault> {
        self.load(ea, 4).map(|v| v as u32)
    }
    fn ld64(&mut self, ea: u32) -> Result<u64, Fault> {
        self.load(ea, 8)
    }
    fn st8(&mut self, ea: u32, v: u8) -> Result<(), Fault> {
        self.store(ea, 1, v as u64)
    }
    fn st16(&mut self, ea: u32, v: u16) -> Result<(), Fault> {
        self.store(ea, 2, v as u64)
    }
    fn st32(&mut self, ea: u32, v: u32) -> Result<(), Fault> {
        self.store(ea, 4, v as u64)
    }
    fn st64(&mut self, ea: u32, v: u64) -> Result<(), Fault> {
        self.store(ea, 8, v)
    }
}
