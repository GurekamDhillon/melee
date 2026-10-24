//! Decode GameCube (big-endian, 4-byte-pointer) bytes into native structs, driven by the same
//! generated layout tables as [`crate::gc::GcLayout`].
//!
//! Every generated record in `melee_types` implements [`GcDecode`] (field by field at its GC
//! offset: scalars byte-swapped, bitfield runs copied verbatim because the native storage keeps
//! GameCube byte order, nested records recursively) and [`GcPointee`] (how a relocated pointer
//! to it becomes a native pointer). The archive reader (`melee_runtime::dat`) owns the bytes and
//! the relocation set; this module is the type-directed half.
//!
//! Pointers: a GC pointer word is followed only if its offset is in the relocation set (that is
//! what makes it a pointer in an HSD archive). The target is decoded once per (offset, type) and
//! shared, so cycles and DAGs come out as the same native graph. Objects are filled from a work
//! queue, not by recursion, so long `next` chains do not grow the stack.
//!
//! Not decodable from the bytes alone, and so left zero and listed in the issues: `void*`
//! targets (no type), function pointers (always zero in data files), pointers that are non-zero
//! but not relocated. A union decodes as its first pointer member when its first word is in
//! the relocation table, else as its first pointer-free member (byte order is the same for any
//! same-size scalar member; a union whose live member differs in element size from the chosen
//! one may come out wrong). A pointer to the first element of an array decodes one element;
//! use [`GcDecoder::array`] when the count is known. `char*` targets decode as C strings.

use core::any::TypeId;
use core::ffi::c_void;
use std::collections::{HashMap, HashSet};

/// A value that can be decoded from GameCube bytes.
pub trait GcDecode: Sized + 'static {
    /// Bytes this type occupies on the GameCube (its `sizeof` there).
    const GC_BYTES: u32;
    /// The all-zero value (null pointers, `None` function pointers).
    fn gc_zeroed() -> Self;
    /// Decode the value whose GameCube bytes start at data offset `at`.
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self;
}

/// What a relocated pointer to `Self` at data offset `target` decodes to.
pub trait GcPointee: 'static {
    fn gc_pointee(d: &mut GcDecoder<'_>, target: u32) -> *mut Self;
}

struct Pending {
    target: u32,
    dst: *mut u8,
    fill: unsafe fn(&mut GcDecoder<'_>, u32, *mut u8),
}

struct Owned {
    ptr: *mut u8,
    len: usize,
    free: unsafe fn(*mut u8, usize),
}

/// The native objects decoded from one archive. Pointers inside them point into this arena, so
/// they are valid exactly as long as it lives.
pub struct GcArena {
    owned: Vec<Owned>,
}

impl GcArena {
    /// Number of separate allocations (objects, arrays, strings).
    pub fn len(&self) -> usize {
        self.owned.len()
    }
    pub fn is_empty(&self) -> bool {
        self.owned.is_empty()
    }
}

impl Drop for GcArena {
    fn drop(&mut self) {
        for o in self.owned.drain(..) {
            // SAFETY: each entry was produced by Box::into_raw with the matching `free`.
            unsafe { (o.free)(o.ptr, o.len) }
        }
    }
}

unsafe fn free_box<T>(p: *mut u8, _: usize) {
    // SAFETY: p came from Box::<T>::into_raw (see `object`).
    drop(unsafe { Box::from_raw(p as *mut T) });
}
unsafe fn free_slice<T>(p: *mut u8, n: usize) {
    // SAFETY: p came from Box::<[T]>::into_raw with length n.
    drop(unsafe { Box::from_raw(core::ptr::slice_from_raw_parts_mut(p as *mut T, n)) });
}
unsafe fn fill<T: GcDecode>(d: &mut GcDecoder<'_>, target: u32, dst: *mut u8) {
    d.cur = (core::any::type_name::<T>(), target);
    let v = T::gc_decode(d, target);
    // SAFETY: dst is the Box<T> that `object` allocated for this (target, T).
    unsafe { (dst as *mut T).write(v) }
}

/// Type-directed decoder over one archive's data block.
pub struct GcDecoder<'a> {
    data: &'a [u8],
    relocs: &'a HashSet<u32>,
    memo: HashMap<(u32, TypeId), *mut u8>,
    queue: Vec<Pending>,
    arena: GcArena,
    issues: Vec<String>,
    /// Object being filled (type, offset), for issue messages.
    cur: (&'static str, u32),
    /// Raw-pointer mode (memory images, see [`GcDecoder::new_raw`]): a pointer word is not followed
    /// but stored as its guest address (`addr as *mut T`; never dereferenced).
    raw: bool,
}

impl<'a> GcDecoder<'a> {
    /// `data` is the archive's data block (pointer values are offsets into it); `relocs` holds the
    /// data offset of every pointer word.
    pub fn new(data: &'a [u8], relocs: &'a HashSet<u32>) -> Self {
        Self { data, relocs, memo: HashMap::new(), queue: Vec::new(), arena: GcArena { owned: Vec::new() },
               issues: Vec::new(), cur: ("(root)", 0), raw: false }
    }

    /// Decoder over a MEMORY IMAGE (a savestate's bytes, not a relocated archive): `data` holds
    /// the object's GameCube bytes, `at` offsets index it. Pointers are never followed; each
    /// decodes to its guest address stored in the native pointer (`addr as usize as *mut T`),
    /// for the caller to look up in the image itself. Function pointers decode to `None`.
    pub fn new_raw(data: &'a [u8], relocs: &'a HashSet<u32>) -> Self {
        let mut d = Self::new(data, relocs);
        d.raw = true;
        d
    }

    /// Record something the bytes could not say.
    pub fn issue(&mut self, s: String) {
        let (ty, at) = self.cur;
        let ty = ty.rsplit("::").next().unwrap_or(ty);
        self.issues.push(format!("{s} (in {ty} @ {at:#x})"));
    }

    fn bytes<const N: usize>(&mut self, at: u32) -> [u8; N] {
        let a = at as usize;
        match self.data.get(a..a + N) {
            Some(b) => b.try_into().unwrap(),
            None => {
                self.issue(format!("read of {N} bytes at {at:#x} past the data block"));
                [0; N]
            }
        }
    }
    pub fn u8_at(&mut self, at: u32) -> u8 {
        self.bytes::<1>(at)[0]
    }
    pub fn u16_at(&mut self, at: u32) -> u16 {
        u16::from_be_bytes(self.bytes(at))
    }
    pub fn u32_at(&mut self, at: u32) -> u32 {
        u32::from_be_bytes(self.bytes(at))
    }
    pub fn u64_at(&mut self, at: u32) -> u64 {
        u64::from_be_bytes(self.bytes(at))
    }

    /// Is the word at data offset `at` a pointer (listed in the relocation table)?
    pub fn is_reloc(&self, at: u32) -> bool {
        self.relocs.contains(&at)
    }

    /// The relocated target of the pointer word at `at`; None for null or not relocated.
    pub fn target(&mut self, at: u32) -> Option<u32> {
        let v = self.u32_at(at);
        if !self.relocs.contains(&at) {
            if v != 0 {
                self.issue(format!("non-zero word {v:#x} at {at:#x} is not relocated; read as null"));
            }
            return None;
        }
        if v as usize >= self.data.len() {
            self.issue(format!("pointer at {at:#x} targets {v:#x}, past the data block"));
            return None;
        }
        Some(v)
    }

    /// Decode the pointer word at `at` into a native pointer (null if not relocated).
    pub fn ptr<T: GcPointee>(&mut self, at: u32) -> *mut T {
        if self.raw {
            return self.u32_at(at) as usize as *mut T;
        }
        match self.target(at) {
            Some(t) => T::gc_pointee(self, t),
            None => core::ptr::null_mut(),
        }
    }

    /// One `T` at data offset `target`, decoded once and shared: the default pointee rule.
    pub fn object<T: GcDecode>(&mut self, target: u32) -> *mut T {
        let key = (target, TypeId::of::<T>());
        if let Some(&p) = self.memo.get(&key) {
            return p as *mut T;
        }
        let p = Box::into_raw(Box::new(T::gc_zeroed())) as *mut u8;
        self.arena.owned.push(Owned { ptr: p, len: 1, free: free_box::<T> });
        self.memo.insert(key, p);
        self.queue.push(Pending { target, dst: p, fill: fill::<T> });
        p as *mut T
    }

    /// `n` consecutive `T`s at `target` (GC stride `T::GC_BYTES`) as one native array.
    pub fn array<T: GcDecode>(&mut self, target: u32, n: usize) -> *mut T {
        let v: Vec<T> = (0..n).map(|i| T::gc_decode(self, target + i as u32 * T::GC_BYTES)).collect();
        let p = Box::into_raw(v.into_boxed_slice()) as *mut T as *mut u8;
        self.arena.owned.push(Owned { ptr: p, len: n, free: free_slice::<T> });
        p as *mut T
    }

    /// The NUL-terminated string at `target`, copied (with its NUL) into the arena; shared.
    pub fn c_string(&mut self, target: u32) -> *mut u8 {
        let key = (target, TypeId::of::<CStr>());
        if let Some(&p) = self.memo.get(&key) {
            return p;
        }
        let rest = &self.data[target as usize..];
        let mut v = match rest.iter().position(|&c| c == 0) {
            Some(i) => rest[..=i].to_vec(),
            None => {
                let v = rest.to_vec();
                self.issue(format!("string at {target:#x} has no NUL before the end of data"));
                v
            }
        };
        if v.last() != Some(&0) {
            v.push(0);
        }
        let len = v.len();
        let p = Box::into_raw(v.into_boxed_slice()) as *mut u8;
        self.arena.owned.push(Owned { ptr: p, len, free: free_slice::<u8> });
        self.memo.insert(key, p);
        p
    }

    /// Fill every object reached so far, and everything they reach.
    pub fn run(&mut self) {
        while let Some(job) = self.queue.pop() {
            // SAFETY: job.dst is the allocation `object` made for exactly this fill fn's type.
            unsafe { (job.fill)(self, job.target, job.dst) }
        }
    }

    /// Finish decoding: the arena owning every object, and the issues found.
    pub fn finish(mut self) -> (GcArena, Vec<String>) {
        self.run();
        let arena = core::mem::replace(&mut self.arena, GcArena { owned: Vec::new() });
        (arena, core::mem::take(&mut self.issues))
    }
}

/// Memo key for C strings (distinct from any decoded `u8`/`i8` object at the same offset).
struct CStr;

macro_rules! scalar {
    ($($t:ty: $n:literal $rd:ident $conv:expr),* $(,)?) => {$(
        impl GcDecode for $t {
            const GC_BYTES: u32 = $n;
            #[inline] fn gc_zeroed() -> Self { 0 as $t }
            #[inline] fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self { let r = d.$rd(at); $conv(r) }
        }
    )*};
}
scalar!(
    u8: 1 u8_at |r| r, u16: 2 u16_at |r| r, u32: 4 u32_at |r| r, u64: 8 u64_at |r| r,
    i16: 2 u16_at |r: u16| r as i16, i32: 4 u32_at |r: u32| r as i32, i64: 8 u64_at |r: u64| r as i64,
    f32: 4 u32_at f32::from_bits, f64: 8 u64_at f64::from_bits,
    // size_t / ptrdiff_t are 4 bytes on the GameCube.
    usize: 4 u32_at |r: u32| r as usize, isize: 4 u32_at |r: u32| r as i32 as isize,
);
impl GcDecode for i8 {
    const GC_BYTES: u32 = 1;
    fn gc_zeroed() -> Self { 0 }
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self { d.u8_at(at) as i8 }
}
impl GcDecode for bool {
    const GC_BYTES: u32 = 1;
    fn gc_zeroed() -> Self { false }
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self { d.u8_at(at) != 0 }
}
macro_rules! object_pointee {
    ($($t:ty),*) => {$(
        impl GcPointee for $t {
            fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self { d.object::<$t>(t) }
        }
    )*};
}
object_pointee!(u16, u32, u64, i16, i32, i64, f32, f64, usize, isize, bool);
/// `char*` / `u8*` in a data file are strings (names, symbols).
impl GcPointee for i8 {
    fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self { d.c_string(t) as *mut i8 }
}
impl GcPointee for u8 {
    fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self { d.c_string(t) }
}
/// `void*`: the bytes do not say what is there.
impl GcPointee for c_void {
    fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self {
        d.issue(format!("void* to {t:#x}: untyped, read as null"));
        core::ptr::null_mut()
    }
}

impl<T: GcPointee> GcDecode for *mut T {
    const GC_BYTES: u32 = 4;
    fn gc_zeroed() -> Self { core::ptr::null_mut() }
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self { d.ptr::<T>(at) }
}
impl<T: GcPointee> GcDecode for *const T {
    const GC_BYTES: u32 = 4;
    fn gc_zeroed() -> Self { core::ptr::null() }
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self { d.ptr::<T>(at) as *const T }
}
impl<T: GcPointee> GcPointee for *mut T {
    fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self { d.object::<*mut T>(t) }
}
impl<T: GcPointee> GcPointee for *const T {
    fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self { d.object::<*const T>(t) }
}
/// Generated code spells C function pointers `Option<unsafe extern "C" fn(..)>` (often through a
/// typedef). A data file never holds a meaningful one: decode as None, flag a non-zero word.
impl<F: 'static> GcDecode for Option<F> {
    const GC_BYTES: u32 = 4;
    fn gc_zeroed() -> Self { None }
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self {
        let w = d.u32_at(at);
        if w != 0 {
            d.issue(format!("function pointer word {w:#x} at {at:#x}: read as None"));
        }
        None
    }
}
impl<F: 'static> GcPointee for Option<F> {
    fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self { d.object::<Option<F>>(t) }
}
impl<T: GcDecode, const N: usize> GcDecode for [T; N] {
    const GC_BYTES: u32 = T::GC_BYTES * N as u32;
    fn gc_zeroed() -> Self { core::array::from_fn(|_| T::gc_zeroed()) }
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self {
        core::array::from_fn(|i| T::gc_decode(d, at + i as u32 * T::GC_BYTES))
    }
}
impl<T: GcDecode, const N: usize> GcPointee for [T; N] {
    fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self { d.object::<[T; N]>(t) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    struct Node {
        v: f32,
        next: *mut Node,
        name: *mut i8,
    }
    impl GcDecode for Node {
        const GC_BYTES: u32 = 12;
        fn gc_zeroed() -> Self { Node { v: 0.0, next: core::ptr::null_mut(), name: core::ptr::null_mut() } }
        fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self {
            Node { v: f32::gc_decode(d, at), next: <*mut Node>::gc_decode(d, at + 4),
                   name: <*mut i8>::gc_decode(d, at + 8) }
        }
    }
    impl GcPointee for Node {
        fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self { d.object::<Node>(t) }
    }

    #[test]
    fn cycle_shares_objects_and_strings() {
        // node A @0 -> B @12 -> A; both named "hi" @24.
        let mut data = Vec::new();
        for (v, next) in [(1.5f32, 12u32), (-2.0, 0)] {
            data.extend(v.to_be_bytes());
            data.extend(next.to_be_bytes());
            data.extend(24u32.to_be_bytes());
        }
        data.extend(b"hi\0\0");
        let relocs: HashSet<u32> = [4, 8, 16, 20].into();
        let mut d = GcDecoder::new(&data, &relocs);
        let a = d.object::<Node>(0);
        let (arena, issues) = d.finish();
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(arena.len(), 3);
        unsafe {
            let b = (*a).next;
            assert_eq!((*a).v, 1.5);
            assert_eq!((*b).v, -2.0);
            assert_eq!((*b).next, a);
            assert_eq!((*a).name, (*b).name);
            assert_eq!(core::ffi::CStr::from_ptr((*a).name).to_bytes(), b"hi");
        }
    }

    #[test]
    fn unrelocated_word_is_null_with_issue() {
        let data = 0x1234u32.to_be_bytes();
        let relocs = HashSet::new();
        let mut d = GcDecoder::new(&data, &relocs);
        let p = <*mut u32>::gc_decode(&mut d, 0);
        assert!(p.is_null());
        assert_eq!(d.finish().1.len(), 1);
    }
}
