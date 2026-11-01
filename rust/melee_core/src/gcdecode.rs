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
/// Records decoded per entry of a `T**` table of arrays whose length is not in the data.
pub const MAX_TABLE_ARRAY: usize = 64;

pub struct GcDecoder<'a> {
    data: &'a [u8],
    relocs: &'a HashSet<u32>,
    memo: HashMap<(u32, TypeId), *mut u8>,
    /// [`GcDecoder::ptr_array`] results per (target, T, length): two records may name one array with different counts.
    arr_memo: HashMap<(u32, TypeId, usize), *mut u8>,
    queue: Vec<Pending>,
    arena: GcArena,
    issues: Vec<String>,
    /// Object being filled (type, offset), for issue messages.
    cur: (&'static str, u32),
    /// Raw-pointer mode (memory images, see [`GcDecoder::new_raw`]): a pointer word is not followed
    /// but stored as its guest address (`addr as *mut T`; never dereferenced).
    raw: bool,
    /// Sorted data offsets where some object starts (every relocation target, every public, the data end): lets a
    /// pointer to an array of records whose length the bytes do not state run to the next boundary. Empty = unknown.
    bounds: Vec<u32>,
    /// Data offsets of the archive's publics (roots).
    roots: Vec<u32>,
    /// Address of the archive's RAW data block (the big-endian file bytes the C reads through pointers). When set,
    /// byte-blob pointers (`char*`, `u8*`, `void*`: names, display lists, vertex arrays, image data, animation
    /// streams) point into it instead of being copied or read as null.
    raw_base: usize,
}

impl<'a> GcDecoder<'a> {
    /// `data` is the archive's data block (pointer values are offsets into it); `relocs` holds the
    /// data offset of every pointer word.
    pub fn new(data: &'a [u8], relocs: &'a HashSet<u32>) -> Self {
        Self { data, relocs, memo: HashMap::new(), arr_memo: HashMap::new(), queue: Vec::new(), arena: GcArena { owned: Vec::new() },
               issues: Vec::new(), cur: ("(root)", 0), raw: false, bounds: Vec::new(), roots: Vec::new(), raw_base: 0 }
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

    /// Point blob pointers (`char*`, `u8*`, `void*`) at the raw archive image: `base` is the address of its data block.
    pub fn set_raw_base(&mut self, base: usize) {
        self.raw_base = base;
    }

    /// The raw-image address of data offset `t`, when a raw image is attached.
    pub fn raw_addr(&self, t: u32) -> Option<*mut u8> {
        (self.raw_base != 0).then(|| (self.raw_base + t as usize) as *mut u8)
    }

    /// Give the decoder the object boundaries of the archive (see the `bounds` field).
    pub fn set_bounds(&mut self, mut b: Vec<u32>) {
        b.sort_unstable();
        b.dedup();
        self.bounds = b;
    }

    /// Bytes from data offset `off` to the next boundary above it (at least 1; 1 when no bounds are set).
    pub fn extent(&self, off: u32) -> u32 {
        match self.bounds.binary_search(&off) {
            Ok(i) => self.bounds.get(i + 1).map_or(1, |n| n - off),
            Err(i) => self.bounds.get(i).map_or(1, |n| n - off),
        }
        .max(1)
    }

    /// Like [`GcDecoder::ptr_table`], but each non-null entry is an ARRAY of `T` (a `T**` whose entries the C indexes:
    /// `table[set][part]`, with the part count nowhere in the data). Too short an array is a crash on the first
    /// index past it, too long only decodes a few more records, so each array is `MAX_TABLE_ARRAY` records, cut at the
    /// next public / the end of the data (`set_roots`).
    pub fn ptr_table_arrays<T: GcDecode>(&mut self, at: u32) -> *mut *mut T {
        if self.raw {
            return self.u32_at(at) as usize as *mut *mut T;
        }
        let Some(t) = self.target(at) else { return core::ptr::null_mut() };
        struct TableArraysKey<T>(core::marker::PhantomData<T>);
        let key = (t, TypeId::of::<TableArraysKey<T>>());
        if let Some(&p) = self.memo.get(&key) {
            return p as *mut *mut T;
        }
        let mut n = 0u32;
        while self.relocs.contains(&(t + 4 * n)) {
            n += 1;
        }
        let v: Vec<*mut T> = (0..=n)
            .map(|i| match self.target(t + 4 * i) {
                Some(e) => {
                    let room = self.root_room(e) / T::GC_BYTES;
                    let cnt = (room as usize).clamp(1, MAX_TABLE_ARRAY);
                    self.array_memo::<T>(e, cnt)
                }
                None => core::ptr::null_mut(),
            })
            .collect();
        let len = v.len();
        let p = Box::into_raw(v.into_boxed_slice()) as *mut *mut T as *mut u8;
        self.arena.owned.push(Owned { ptr: p, len, free: free_slice::<*mut T> });
        self.memo.insert(key, p);
        p as *mut *mut T
    }

    /// Give the decoder the data offsets of the archive's publics (roots); see [`GcDecoder::ptr_table_arrays`].
    pub fn set_roots(&mut self, mut r: Vec<u32>) {
        r.sort_unstable();
        r.dedup();
        self.roots = r;
    }

    /// Bytes from `off` to the next root above it, or to the end of the data block.
    fn root_room(&self, off: u32) -> u32 {
        let end = self.data.len() as u32;
        let next = match self.roots.binary_search(&off) {
            Ok(i) => self.roots.get(i + 1).copied(),
            Err(i) => self.roots.get(i).copied(),
        };
        next.unwrap_or(end).min(end).saturating_sub(off).max(1)
    }

    /// Like [`GcDecoder::ptr_table`], but each non-null entry is an ARRAY of exactly `n` `T` (the length is a sibling
    /// field of the record holding the table pointer).
    pub fn ptr_table_arrays_n<T: GcDecode>(&mut self, at: u32, n: usize) -> *mut *mut T {
        if self.raw {
            return self.u32_at(at) as usize as *mut *mut T;
        }
        let Some(t) = self.target(at) else { return core::ptr::null_mut() };
        struct TableArraysNKey<T>(core::marker::PhantomData<T>);
        let key = (t, TypeId::of::<TableArraysNKey<T>>());
        if let Some(&p) = self.memo.get(&key) {
            return p as *mut *mut T;
        }
        let mut k = 0u32;
        while self.relocs.contains(&(t + 4 * k)) {
            k += 1;
        }
        let v: Vec<*mut T> = (0..=k)
            .map(|i| match self.target(t + 4 * i) {
                Some(e) => self.array_memo::<T>(e, n.max(1)),
                None => core::ptr::null_mut(),
            })
            .collect();
        let len = v.len();
        let p = Box::into_raw(v.into_boxed_slice()) as *mut *mut T as *mut u8;
        self.arena.owned.push(Owned { ptr: p, len, free: free_slice::<*mut T> });
        self.memo.insert(key, p);
        p as *mut *mut T
    }

    /// The pointer word at `at` read as an array of `T` that ends with the first element whose FIRST word equals
    /// `word` (that element is included): `HSD_VtxDescList[]` ends with `attr == GX_VA_NULL`.
    pub fn ptr_array_until_word<T: GcDecode>(&mut self, at: u32, word: u32) -> *mut T {
        if self.raw {
            return self.u32_at(at) as usize as *mut T;
        }
        let Some(t) = self.target(at) else { return core::ptr::null_mut() };
        struct UntilKey<T>(core::marker::PhantomData<T>);
        let key = (t, TypeId::of::<UntilKey<T>>());
        if let Some(&p) = self.memo.get(&key) {
            return p as *mut T;
        }
        let mut n = 0u32;
        while t + n * T::GC_BYTES + 4 <= self.data.len() as u32 && self.u32_at(t + n * T::GC_BYTES) != word {
            n += 1;
        }
        let p = self.array::<T>(t, n as usize + 1) as *mut u8;
        self.memo.insert(key, p);
        p as *mut T
    }

    /// An array of `T` ending with the first element whose FIRST word is not a pointer (null); that element is
    /// included. `HSD_EnvelopeDesc[]` ends with `joint == NULL`.
    pub fn array_until_null<T: GcDecode>(&mut self, target: u32) -> *mut T {
        struct UntilNullKey<T>(core::marker::PhantomData<T>);
        let key = (target, TypeId::of::<UntilNullKey<T>>());
        if let Some(&p) = self.memo.get(&key) {
            return p as *mut T;
        }
        let mut n = 0u32;
        while self.relocs.contains(&(target + n * T::GC_BYTES)) {
            n += 1;
        }
        let p = self.array::<T>(target, n as usize + 1) as *mut u8;
        self.memo.insert(key, p);
        p as *mut T
    }

    /// The pointer word at `at` read as a NULL-terminated table of pointers to arrays that each end with a
    /// null-first-word element (`HSD_EnvelopeDesc**`).
    pub fn ptr_table_null_arrays<T: GcDecode>(&mut self, at: u32) -> *mut *mut T {
        if self.raw {
            return self.u32_at(at) as usize as *mut *mut T;
        }
        let Some(t) = self.target(at) else { return core::ptr::null_mut() };
        struct TableNullKey<T>(core::marker::PhantomData<T>);
        let key = (t, TypeId::of::<TableNullKey<T>>());
        if let Some(&p) = self.memo.get(&key) {
            return p as *mut *mut T;
        }
        let mut k = 0u32;
        while self.relocs.contains(&(t + 4 * k)) {
            k += 1;
        }
        let v: Vec<*mut T> = (0..=k)
            .map(|i| match self.target(t + 4 * i) {
                Some(e) => self.array_until_null::<T>(e),
                None => core::ptr::null_mut(),
            })
            .collect();
        let len = v.len();
        let p = Box::into_raw(v.into_boxed_slice()) as *mut *mut T as *mut u8;
        self.arena.owned.push(Owned { ptr: p, len, free: free_slice::<*mut T> });
        self.memo.insert(key, p);
        p as *mut *mut T
    }

    /// The pointer word at `at` read as an array of `T` that runs to the next object boundary (`set_bounds`): for
    /// arrays nothing else points into and whose length is only implied by the layout of the file.
    pub fn ptr_array_extent<T: GcDecode>(&mut self, at: u32) -> *mut T {
        if self.raw {
            return self.u32_at(at) as usize as *mut T;
        }
        let Some(t) = self.target(at) else { return core::ptr::null_mut() };
        let n = (self.extent(t) / T::GC_BYTES).max(1) as usize;
        self.array_memo::<T>(t, n)
    }

    /// A table of pointers that runs to the next object boundary (null entries allowed), each non-null entry an
    /// array of exactly `n` `T`: `ftData_x8_x8.xC`, one `u16[x8]` row per costume visibility row.
    pub fn ptr_table_extent_arrays_n<T: GcDecode>(&mut self, at: u32, n: usize) -> *mut *mut T {
        if self.raw {
            return self.u32_at(at) as usize as *mut *mut T;
        }
        let Some(t) = self.target(at) else { return core::ptr::null_mut() };
        struct TabExtKey<T>(core::marker::PhantomData<T>);
        let key = (t, TypeId::of::<TabExtKey<T>>());
        if let Some(&p) = self.memo.get(&key) {
            return p as *mut *mut T;
        }
        let rows = (self.extent(t) / 4).max(1);
        let v: Vec<*mut T> = (0..rows)
            .map(|i| match self.target(t + 4 * i) {
                Some(e) => self.array_memo::<T>(e, n.max(1)),
                None => core::ptr::null_mut(),
            })
            .collect();
        let len = v.len();
        let p = Box::into_raw(v.into_boxed_slice()) as *mut *mut T as *mut u8;
        self.arena.owned.push(Owned { ptr: p, len, free: free_slice::<*mut T> });
        self.memo.insert(key, p);
        p as *mut *mut T
    }

    /// `n` consecutive `T`s at `target`, shared per (target, T).
    fn array_memo<T: GcDecode>(&mut self, target: u32, n: usize) -> *mut T {
        struct ArrKey<T>(core::marker::PhantomData<T>);
        let key = (target, TypeId::of::<ArrKey<T>>());
        if let Some(&p) = self.memo.get(&key) {
            return p as *mut T;
        }
        let p = self.array::<T>(target, n) as *mut u8;
        self.memo.insert(key, p);
        p as *mut T
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

    /// The pointer word at `at` read as a pointer to an ARRAY of `n` `T`s (the length is another field of the
    /// record; the generated `GcDecode` of a field listed in `melee_types/gen/arrays.tsv` calls this). Shared per
    /// (target, T, n). `n` = 0 still yields a pointer to a one-element allocation so the pointer stays non-null
    /// like the C's. In raw mode the word is returned as the guest address.
    pub fn ptr_array<T: GcDecode>(&mut self, at: u32, n: usize) -> *mut T {
        if self.raw {
            return self.u32_at(at) as usize as *mut T;
        }
        let Some(t) = self.target(at) else { return core::ptr::null_mut() };
        struct ArrayKey<T>(core::marker::PhantomData<T>);
        let key = (t, TypeId::of::<ArrayKey<T>>(), n.max(1));
        if let Some(&p) = self.arr_memo.get(&key) {
            return p as *mut T;
        }
        let p = self.array::<T>(t, n.max(1)) as *mut u8;
        self.arr_memo.insert(key, p);
        p as *mut T
    }

    /// The pointer word at `at` read as a pointer to a NULL-TERMINATED table of `T*` (a `T**` the C walks until it
    /// finds NULL, or indexes within a count the bytes do not state): consecutive relocated words are followed, the first
    /// word that is not relocated ends the table and is kept as the terminating NULL. Shared per target.
    pub fn ptr_table<T: GcPointee>(&mut self, at: u32) -> *mut *mut T {
        if self.raw {
            return self.u32_at(at) as usize as *mut *mut T;
        }
        let Some(t) = self.target(at) else { return core::ptr::null_mut() };
        struct TableKey<T>(core::marker::PhantomData<T>);
        let key = (t, TypeId::of::<TableKey<T>>());
        if let Some(&p) = self.memo.get(&key) {
            return p as *mut *mut T;
        }
        let mut n = 0u32;
        while self.relocs.contains(&(t + 4 * n)) {
            n += 1;
        }
        let v: Vec<*mut T> = (0..=n).map(|i| self.ptr::<T>(t + 4 * i)).collect();
        let len = v.len();
        let p = Box::into_raw(v.into_boxed_slice()) as *mut *mut T as *mut u8;
        self.arena.owned.push(Owned { ptr: p, len, free: free_slice::<*mut T> });
        self.memo.insert(key, p);
        p as *mut *mut T
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
    fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self {
        match d.raw_addr(t) { Some(p) => p as *mut i8, None => d.c_string(t) as *mut i8 }
    }
}
impl GcPointee for u8 {
    fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self {
        match d.raw_addr(t) { Some(p) => p, None => d.c_string(t) }
    }
}
/// `void*`: the bytes do not say what is there.
impl GcPointee for c_void {
    fn gc_pointee(d: &mut GcDecoder<'_>, t: u32) -> *mut Self {
        if let Some(p) = d.raw_addr(t) {
            return p as *mut c_void;
        }
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
