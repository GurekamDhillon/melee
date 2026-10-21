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
//!     for a pointer to a known record on the way), a store maps a handle back to the native
//!     pointer; a partial pointer access or a store of a non-handle faults;
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

    /// Native location of GC byte `off` of record `rec`.
    pub fn locate(&self, rec: &str, off: u32) -> Loc {
        let Some(&(_, fields)) = self.by_name.get(rec) else { return Loc::Mismatch };
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
}

pub struct GuestSpace {
    pub layouts: Layouts,
    pub regions: Vec<Region>,
    /// Next guest address handed to an auto-registered native object.
    next: u32,
    pub trace: Option<Vec<Rewrite>>,
}

impl GuestSpace {
    pub fn new() -> GuestSpace {
        GuestSpace { layouts: Layouts::new(), regions: vec![], next: 0x9000_0000, trace: None }
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
            self.next = self.next.wrapping_add((size + 0x1F) & !0xF);
            b
        });
        self.regions.push(Region { base, size, backing: Backing::Native { rec, ptr, shadow: vec![0; size as usize] } });
        Ok(base)
    }

    pub fn bytes(&self, base: u32) -> Option<&[u8]> {
        self.regions.iter().find(|r| r.base == base).and_then(|r| match &r.backing {
            Backing::Bytes(b) => Some(&b[..]),
            _ => None,
        })
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
        for r in &self.regions {
            if let Backing::Native { ptr, .. } = r.backing {
                if ptr as usize == p {
                    return Ok(r.base);
                }
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
        self.regions.iter().find(|r| r.base == h).and_then(|r| match r.backing {
            Backing::Native { ptr, .. } => Some(ptr as usize),
            _ => None,
        })
    }

    fn load(&mut self, ea: u32, n: u32) -> Result<u64, Fault> {
        let i = self.find(ea, n)?;
        let r = &self.regions[i];
        let o = ea - r.base;
        match &r.backing {
            Backing::Bytes(b) => Ok(b[o as usize..(o + n) as usize].iter().fold(0u64, |v, &x| v << 8 | x as u64)),
            Backing::Native { rec, ptr, shadow } => {
                let (rec, ptr) = (*rec, *ptr);
                if let Some(t) = &mut self.trace {
                    t.push(Rewrite { rec, gc_off: o, size: n, store: false });
                }
                if let Loc::Ptr(nat, 0, pointee) = self.layouts.locate(rec, o) {
                    if n != 4 {
                        return Err(Fault::Mem(ea));
                    }
                    // SAFETY: add_native's contract; nat is inside the native object.
                    let p = unsafe { (ptr.add(nat) as *const usize).read_unaligned() };
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

    fn store(&mut self, ea: u32, n: u32, v: u64) -> Result<(), Fault> {
        let i = self.find(ea, n)?;
        let o = ea - self.regions[i].base;
        let (rec, ptr) = match &mut self.regions[i].backing {
            Backing::Bytes(b) => {
                for k in 0..n {
                    b[(o + k) as usize] = (v >> (8 * (n - 1 - k))) as u8;
                }
                return Ok(());
            }
            Backing::Native { rec, ptr, .. } => (*rec, *ptr),
        };
        if let Some(t) = &mut self.trace {
            t.push(Rewrite { rec, gc_off: o, size: n, store: true });
        }
        if let Loc::Ptr(nat, 0, _) = self.layouts.locate(rec, o) {
            if n != 4 {
                return Err(Fault::Mem(ea));
            }
            let p = self.native_of(v as u32).ok_or(Fault::Mem(ea))?;
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
