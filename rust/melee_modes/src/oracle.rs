//! State-oracle support for the `tu/` ports (docs: rust/melee_modes/gen/tu/README.md).
//!
//! A stateful function cannot be checked on generated scalars, so each `tu/<stem>.rs` port is run
//! on a synthetic game state and the C original (the real `src/melee/gm/<stem>.c`, compiled
//! natively with the shipping float contraction, its callees replaced by recording stubs) is run on
//! the byte-identical state. Both sides compare: the return value, a hash of the whole state
//! (arena + every registered global, word-normalised) and a hash of the call log.
//!
//! * [`Arena`]: one fixed-address block (same virtual address in the C process) holding every
//!   object a case builds. Pointers inside it are therefore equal on both sides.
//! * `globals!`: the TU's (and the externs it uses) globals as plain `static mut`s, registered so
//!   the harness can copy them in and out of the arena and hash them.
//! * `ext_fns!`: callees of unported TUs. Each stub first offers the call to the active recorder
//!   (this module) and only then forwards to the real port or panics with a TODO.
//! * [`run_tu`]: exports the cases (dev time) or checks them against `tests/data/tu_<stem>.txt`.
//!
//! The recorder's byte-exact behaviour is mirrored by `gen/tu/rt.c`; change both together.
#![allow(clippy::missing_safety_doc, static_mut_refs)]

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::Mutex;

// ------------------------------------------------------------------------------------------
// hashing, rng
// ------------------------------------------------------------------------------------------

pub const FNV_OFF: u64 = 0xcbf2_9ce4_8422_2325;
pub const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

pub fn fnv_bytes(b: &[u8]) -> u64 {
    let mut h = FNV_OFF;
    for &c in b {
        h ^= c as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

#[inline]
pub fn mix(h: u64, w: u64) -> u64 {
    (h ^ w).wrapping_mul(FNV_PRIME)
}

/// splitmix64 step on an explicit state.
#[inline]
pub fn splitmix(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Deterministic generator for scenario builders (Rust-only: the C side gets the built image).
#[derive(Clone, Debug)]
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        splitmix(self.0.wrapping_sub(0x9E37_79B9_7F4A_7C15))
    }
    pub fn u32(&mut self) -> u32 {
        (self.next() >> 16) as u32
    }
    /// Uniform in `lo..=hi`.
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next() % ((hi - lo + 1) as u64)) as i64
    }
    pub fn chance(&mut self, num: u64, den: u64) -> bool {
        self.next() % den < num
    }
    /// A float in about -1000.0..1000.0 with a short fraction, sometimes exactly 0/1/-1.
    pub fn f32(&mut self) -> f32 {
        match self.next() % 12 {
            0 => 0.0,
            1 => 1.0,
            2 => -1.0,
            _ => (self.range(-10000, 10000) as f32) * 0.1,
        }
    }
    /// A float in `lo..hi` with 1/1000 steps.
    pub fn f32_in(&mut self, lo: f32, hi: f32) -> f32 {
        let t = self.range(0, 1000) as f32 / 1000.0;
        lo + (hi - lo) * t
    }
    pub fn fill(&mut self, p: *mut u8, n: usize) {
        let mut i = 0;
        while i < n {
            let w = self.next().to_le_bytes();
            let k = (n - i).min(8);
            for j in 0..k {
                unsafe { *p.add(i + j) = w[j] };
            }
            i += k;
        }
    }
}

// ------------------------------------------------------------------------------------------
// the arena
// ------------------------------------------------------------------------------------------

pub const ARENA_BASE: usize = 0x0000_0200_0000_0000;
pub const ARENA_SIZE: usize = 32 << 20;

#[cfg(windows)]
unsafe fn map_fixed(addr: usize, size: usize) -> *mut u8 {
    extern "system" {
        fn VirtualAlloc(a: *mut u8, sz: usize, ty: u32, prot: u32) -> *mut u8;
    }
    VirtualAlloc(addr as *mut u8, size, 0x3000, 0x04)
}

#[cfg(unix)]
unsafe fn map_fixed(addr: usize, size: usize) -> *mut u8 {
    extern "C" {
        fn mmap(a: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut u8;
    }
    // PROT_READ|WRITE, MAP_PRIVATE|MAP_ANONYMOUS|MAP_FIXED_NOREPLACE
    let p = mmap(addr as *mut u8, size, 3, 0x2 | 0x20 | 0x100000, -1, 0);
    if p as isize == -1 {
        core::ptr::null_mut()
    } else {
        p
    }
}

/// The fixed-address block all case objects live in. `hwm` is the bump pointer (shared with the
/// recorder's pool allocations); everything below it is hashed.
pub struct Arena {
    pub base: *mut u8,
    pub hwm: usize,
}

impl Arena {
    unsafe fn get() -> Arena {
        let p = map_fixed(ARENA_BASE, ARENA_SIZE);
        assert!(p as usize == ARENA_BASE, "oracle: cannot map the arena at {ARENA_BASE:#x}");
        Arena { base: p, hwm: 0 }
    }
    /// Zero the used part and reset the bump pointer to `start`.
    pub fn reset(&mut self, start: usize) {
        let used = MAXUSED.load(core::sync::atomic::Ordering::Relaxed).max(self.hwm).max(start);
        unsafe { core::ptr::write_bytes(self.base, 0, used) };
        self.hwm = start;
    }
    /// Zeroed, 16-aligned allocation of `n` bytes.
    pub fn alloc_bytes(&mut self, n: usize) -> *mut u8 {
        let off = (self.hwm + 15) & !15;
        assert!(off + n <= ARENA_SIZE, "oracle: arena full");
        self.hwm = off + n;
        unsafe { self.base.add(off) }
    }
    /// Zeroed object of type `T`.
    pub fn alloc<T>(&mut self) -> *mut T {
        self.alloc_bytes(core::mem::size_of::<T>()).cast()
    }
    /// Zeroed array of `n` objects of type `T`.
    pub fn alloc_n<T>(&mut self, n: usize) -> *mut T {
        self.alloc_bytes(core::mem::size_of::<T>() * n).cast()
    }
    /// `T` whose bytes are random (pointer fields get wired by the caller).
    pub fn alloc_random<T>(&mut self, rng: &mut Rng) -> *mut T {
        let p = self.alloc::<T>();
        rng.fill(p.cast(), core::mem::size_of::<T>());
        p
    }
    pub fn contains(&self, p: u64) -> bool {
        p >= ARENA_BASE as u64 && p < (ARENA_BASE + ARENA_SIZE) as u64
    }
}

// ------------------------------------------------------------------------------------------
// typed values crossing the callee boundary
// ------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum K {
    /// integer of this many bits (bool = 1)
    I(u8),
    P,
    F,
    D,
    V,
}

pub trait OVal: Copy {
    const K: K;
    fn to_bits(self) -> u64;
    fn from_bits(v: u64) -> Self;
}

macro_rules! oval_int {
    ($($t:ty => $bits:literal as $u:ty),*) => {$(
        impl OVal for $t {
            const K: K = K::I($bits);
            fn to_bits(self) -> u64 { self as $u as u64 }
            fn from_bits(v: u64) -> Self { v as $u as $t }
        }
    )*};
}
oval_int!(i8 => 8 as u8, u8 => 8 as u8, i16 => 16 as u16, u16 => 16 as u16, i32 => 32 as u32,
          u32 => 32 as u32, i64 => 64 as u64, u64 => 64 as u64, isize => 64 as u64, usize => 64 as u64);

impl OVal for bool {
    const K: K = K::I(1);
    fn to_bits(self) -> u64 {
        self as u64
    }
    fn from_bits(v: u64) -> Self {
        v & 1 != 0
    }
}
impl OVal for f32 {
    const K: K = K::F;
    fn to_bits(self) -> u64 {
        f32::to_bits(self) as u64
    }
    fn from_bits(v: u64) -> Self {
        f32::from_bits(v as u32)
    }
}
impl OVal for f64 {
    const K: K = K::D;
    fn to_bits(self) -> u64 {
        f64::to_bits(self)
    }
    fn from_bits(v: u64) -> Self {
        f64::from_bits(v)
    }
}
impl OVal for () {
    const K: K = K::V;
    fn to_bits(self) -> u64 {
        0
    }
    fn from_bits(_: u64) -> Self {}
}
impl<T> OVal for *mut T {
    const K: K = K::P;
    fn to_bits(self) -> u64 {
        self as usize as u64
    }
    fn from_bits(v: u64) -> Self {
        v as usize as *mut T
    }
}
impl<T> OVal for *const T {
    const K: K = K::P;
    fn to_bits(self) -> u64 {
        self as usize as u64
    }
    fn from_bits(v: u64) -> Self {
        v as usize as *const T
    }
}

macro_rules! oval_fnptr {
    ($($a:ident),*) => {
        impl<R, $($a),*> OVal for Option<unsafe extern "C" fn($($a),*) -> R> {
            const K: K = K::P;
            fn to_bits(self) -> u64 { match self { Some(f) => f as usize as u64, None => 0 } }
            fn from_bits(v: u64) -> Self { if v == 0 { None } else { Some(unsafe { core::mem::transmute::<usize, unsafe extern "C" fn($($a),*) -> R>(v as usize) }) } }
        }
        impl<R, $($a),*> OVal for unsafe extern "C" fn($($a),*) -> R {
            const K: K = K::P;
            fn to_bits(self) -> u64 { self as usize as u64 }
            fn from_bits(v: u64) -> Self { unsafe { core::mem::transmute::<usize, unsafe extern "C" fn($($a),*) -> R>(v as usize) } }
        }
    };
}
oval_fnptr!();
oval_fnptr!(A);
oval_fnptr!(A, B);
oval_fnptr!(A, B, C);
oval_fnptr!(A, B, C, D);
oval_fnptr!(A, B, C, D, E);

/// One argument as the recorder sees it.
#[derive(Clone, Copy, Debug)]
pub struct Arg {
    pub k: K,
    pub v: u64,
}
#[inline]
pub fn arg<T: OVal>(v: T) -> Arg {
    Arg { k: T::K, v: v.to_bits() }
}

// ------------------------------------------------------------------------------------------
// registered globals and function addresses
// ------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GMode {
    /// copied in from the arena before a case, copied back after
    Rw,
    /// never written by the port; the C side checks its own copy is byte-equal to ours
    Ro,
}

pub struct GlobalDesc {
    pub name: &'static str,
    pub addr: *mut u8,
    pub size: usize,
    pub mode: GMode,
}
unsafe impl Send for GlobalDesc {}
unsafe impl Sync for GlobalDesc {}

/// Declares globals as plain `static mut` / `static` items and a `oracle_globals()` table.
/// `rw VIS NAME: T = init;` is mutable state (`static mut`); `ro VIS NAME: T = init;` is a constant
/// table (`static`).
#[macro_export]
macro_rules! globals {
    ($( $(#[$m:meta])* $mode:ident $vis:vis $name:ident : $t:ty = $init:expr; )*) => {
        $( $crate::globals!(@decl $mode, [$(#[$m])*] $vis $name : $t = $init); )*
        /// Every global of this file, for the state oracle.
        pub fn oracle_globals() -> Vec<$crate::oracle::GlobalDesc> {
            vec![ $( $crate::oracle::GlobalDesc {
                name: stringify!($name),
                addr: core::ptr::addr_of!($name) as *mut u8,
                size: core::mem::size_of::<$t>(),
                mode: $crate::globals!(@mode $mode),
            } ),* ]
        }
    };
    (@decl rw, [$(#[$m:meta])*] $vis:vis $name:ident : $t:ty = $init:expr) => {
        $(#[$m])* #[allow(non_upper_case_globals, non_snake_case)] $vis static mut $name: $t = $init;
    };
    (@decl ro, [$(#[$m:meta])*] $vis:vis $name:ident : $t:ty = $init:expr) => {
        $(#[$m])* #[allow(non_upper_case_globals, non_snake_case)] $vis static $name: $t = $init;
    };
    (@mode rw) => { $crate::oracle::GMode::Rw };
    (@mode ro) => { $crate::oracle::GMode::Ro };
}

// ------------------------------------------------------------------------------------------
// the recorder (mirrors gen/tu/rt.c)
// ------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum RetSpec {
    Int(i64, i64),
    Ptr(u64),
    Pool(usize),
    Seq(Vec<u64>),
    Flt(u32, u32),
    Dbl(u64, u64),
}

#[derive(Clone, Debug, Default)]
pub struct Specs {
    pub ret: Vec<(String, RetSpec)>,
    /// (callee, arg index, nbytes)
    pub out: Vec<(String, usize, usize)>,
    pub str_: Vec<(String, usize)>,
    pub mem: Vec<(String, usize, usize)>,
    /// callees whose return value is the fixed constant (any kind), for tests
    pub trace: bool,
}

pub struct Rec {
    pub seed: u64,
    pub log: u64,
    pub ncalls: u64,
    pub specs: Specs,
    counts: HashMap<u64, u64>,
    fns: Vec<(u64, u64)>,           // (addr, tag) sorted by addr
    globs: Vec<(u64, u64, u64)>,    // (addr, size, tag-base) sorted by addr
    pub hwm: usize,
    pub text: String,
}

static MAXUSED: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
static REC: Mutex<Option<Rec>> = Mutex::new(None);
static RUN_LOCK: Mutex<()> = Mutex::new(());

fn fn_tag(name: &str) -> u64 {
    0xF100_0000_0000_0000 | (fnv_bytes(name.as_bytes()) & 0xFFFF_FFFF)
}
fn glob_tag(name: &str) -> u64 {
    0xF200_0000_0000_0000 | ((fnv_bytes(name.as_bytes()) & 0xFF_FFFF) << 32)
}

impl Rec {
    /// Pointer / word normalisation shared by argument logging and state hashing.
    pub fn norm(&self, w: u64) -> u64 {
        if w < 0x10000 {
            return w;
        }
        if let Ok(i) = self.fns.binary_search_by(|e| e.0.cmp(&w)) {
            return self.fns[i].1;
        }
        let i = self.globs.partition_point(|e| e.0 <= w);
        if i > 0 {
            let (a, s, t) = self.globs[i - 1];
            if w < a + s {
                return t | (w - a);
            }
        }
        w
    }
    fn norm_ptr(&self, w: u64) -> u64 {
        if w == 0 {
            return 0;
        }
        if w >= ARENA_BASE as u64 && w < (ARENA_BASE + ARENA_SIZE) as u64 {
            return 0xF300_0000_0000_0000 | (w - ARENA_BASE as u64);
        }
        let n = self.norm(w);
        if n != w {
            return n;
        }
        0xF400_0000_0000_0000
    }
    fn stream(&mut self, name_h: u64) -> u64 {
        let c = self.counts.entry(name_h).or_insert(0);
        let n = *c;
        *c += 1;
        splitmix(self.seed ^ name_h.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ n.wrapping_mul(0xD6E8_FEB8_6659_FD93))
    }
    fn pool(&mut self, n: usize) -> u64 {
        let off = (self.hwm + 15) & !15;
        self.hwm = off + n;
        assert!(self.hwm <= ARENA_SIZE, "oracle: pool exhausted");
        (ARENA_BASE + off) as u64
    }
}

fn flt_pick(lo: f32, hi: f32, x: u64) -> f32 {
    let t = (x % 10001) as f32 / 10000.0f32;
    let d = hi - lo;
    let m = d * t;
    lo + m
}

/// Called by every `ext` stub. Returns the stub's return value bits when a recorder is active.
pub fn ext_call(name: &'static str, args: &[Arg], ret: K) -> Option<u64> {
    let mut g = REC.lock().unwrap_or_else(|e| e.into_inner());
    let r = g.as_mut()?;
    let nh = fnv_bytes(name.as_bytes());
    r.ncalls += 1;
    r.log = mix(r.log, nh);
    let mut tr = String::new();
    if r.specs.trace {
        let _ = write!(tr, "CALL {name}");
    }
    for (i, a) in args.iter().enumerate() {
        let n = match a.k {
            K::I(b) => {
                if b >= 64 {
                    a.v
                } else {
                    a.v & ((1u64 << b) - 1)
                }
            }
            K::P => r.norm_ptr(a.v),
            K::F => a.v & 0xFFFF_FFFF,
            K::D => a.v,
            K::V => 0,
        };
        r.log = mix(r.log, n);
        if r.specs.trace {
            let _ = write!(tr, " {n:x}");
        }
        if a.k == K::P && a.v != 0 {
            let p = a.v as usize as *const u8;
            for (cn, ci) in &r.specs.str_.clone() {
                if cn == name && *ci == i {
                    let mut h = FNV_OFF;
                    for j in 0..256 {
                        let c = unsafe { *p.add(j) };
                        if c == 0 {
                            break;
                        }
                        h = (h ^ c as u64).wrapping_mul(FNV_PRIME);
                    }
                    r.log = mix(r.log, h);
                    if r.specs.trace {
                        let _ = write!(tr, " str:{h:x}");
                    }
                }
            }
            for (cn, ci, nb) in &r.specs.mem.clone() {
                if cn == name && *ci == i {
                    let mut h = FNV_OFF;
                    for j in 0..*nb {
                        h = (h ^ unsafe { *p.add(j) } as u64).wrapping_mul(FNV_PRIME);
                    }
                    r.log = mix(r.log, h);
                    if r.specs.trace {
                        let _ = write!(tr, " mem:{h:x}");
                    }
                }
            }
        }
    }
    // out parameters: pseudo-random bytes from the callee's own stream
    for (cn, ci, nb) in r.specs.out.clone() {
        if cn == name && ci < args.len() && args[ci].v != 0 {
            let p = args[ci].v as usize as *mut u8;
            let mut x = r.stream(nh ^ 0x5555);
            let mut j = 0;
            while j < nb {
                let w = x.to_le_bytes();
                for q in 0..8.min(nb - j) {
                    unsafe { *p.add(j + q) = w[q] };
                }
                j += 8;
                x = splitmix(x);
            }
        }
    }
    let spec = r.specs.ret.iter().find(|(n, _)| n == name).map(|(_, s)| s.clone());
    let v = match ret {
        K::V => 0,
        K::I(b) => {
            let x = r.stream(nh);
            let raw = match spec {
                Some(RetSpec::Int(lo, hi)) => (lo as u64).wrapping_add(x % ((hi.wrapping_sub(lo) as u64).wrapping_add(1))),
                Some(RetSpec::Seq(v)) => v[(r.counts[&nh] as usize - 1) % v.len()],
                Some(RetSpec::Ptr(a)) => a,
                _ => x % 4,
            };
            if b >= 64 { raw } else { raw & ((1u64 << b) - 1) }
        }
        K::P => {
            let x = r.stream(nh);
            let _ = x;
            match spec {
                Some(RetSpec::Ptr(a)) => a,
                Some(RetSpec::Pool(n)) => r.pool(n),
                Some(RetSpec::Seq(v)) => v[(r.counts[&nh] as usize - 1) % v.len()],
                Some(RetSpec::Int(lo, hi)) => (lo as u64).wrapping_add(x % ((hi.wrapping_sub(lo) as u64).wrapping_add(1))),
                _ => r.pool(0x100),
            }
        }
        K::F => {
            let x = r.stream(nh);
            let f = match spec {
                Some(RetSpec::Flt(lo, hi)) => flt_pick(f32::from_bits(lo), f32::from_bits(hi), x),
                _ => ((x % 2001) as i32 - 1000) as f32 * 0.1f32,
            };
            f.to_bits() as u64
        }
        K::D => {
            let x = r.stream(nh);
            let f = match spec {
                Some(RetSpec::Dbl(lo, hi)) => {
                    let (lo, hi) = (f64::from_bits(lo), f64::from_bits(hi));
                    let t = (x % 10001) as f64 / 10000.0;
                    let d = hi - lo;
                    let m = d * t;
                    lo + m
                }
                _ => ((x % 2001) as i32 - 1000) as f64 * 0.1,
            };
            f.to_bits()
        }
    };
    if r.specs.trace {
        let _ = write!(tr, " -> {v:x}");
        r.text.push_str(&tr);
        r.text.push('\n');
    }
    Some(v)
}

/// Declares callees of unported TUs (or ported ones the oracle must intercept).
/// `fn name(args) -> Ret;` = TODO stub; `fn name(args) -> Ret = path::to::port;` forwards.
#[macro_export]
macro_rules! ext_fns {
    ($( $(#[$m:meta])* fn $name:ident ( $($a:ident : $t:ty),* $(,)? ) -> $r:ty $(= $fwd:path)? ; )*) => {
        $(
            $(#[$m])*
            #[allow(non_snake_case, unused_variables)]
            pub unsafe extern "C" fn $name($($a: $t),*) -> $r {
                if let Some(v) = $crate::oracle::ext_call(stringify!($name),
                        &[$($crate::oracle::arg($a)),*], <$r as $crate::oracle::OVal>::K) {
                    return <$r as $crate::oracle::OVal>::from_bits(v);
                }
                $crate::ext_fns!(@body $name ( $($a),* ) $(= $fwd)?)
            }
        )*
    };
    (@body $name:ident ( $($a:ident),* ) = $fwd:path) => { $fwd($($a),*) };
    (@body $name:ident ( $($a:ident),* )) => { panic!(concat!("TODO(ext) ", stringify!($name), ": its TU is not ported")) };
}

// ------------------------------------------------------------------------------------------
// cases
// ------------------------------------------------------------------------------------------

/// What a builder works with.
pub struct Case {
    pub arena: Arena,
    pub rng: Rng,
    pub specs: Specs,
    pub args: Vec<u64>,
    /// the case index within the function
    pub idx: u32,
}

impl Case {
    pub fn arg<T: OVal>(&mut self, v: T) {
        self.args.push(v.to_bits());
    }
    pub fn ret_int(&mut self, callee: &str, lo: i64, hi: i64) {
        self.specs.ret.push((callee.into(), RetSpec::Int(lo, hi)));
    }
    pub fn ret_ptr<T>(&mut self, callee: &str, p: *mut T) {
        self.specs.ret.push((callee.into(), RetSpec::Ptr(p as usize as u64)));
    }
    pub fn ret_pool(&mut self, callee: &str, size: usize) {
        self.specs.ret.push((callee.into(), RetSpec::Pool(size)));
    }
    pub fn ret_seq<T>(&mut self, callee: &str, ps: &[*mut T]) {
        self.specs.ret.push((callee.into(), RetSpec::Seq(ps.iter().map(|&p| p as usize as u64).collect())));
    }
    pub fn ret_flt(&mut self, callee: &str, lo: f32, hi: f32) {
        self.specs.ret.push((callee.into(), RetSpec::Flt(lo.to_bits(), hi.to_bits())));
    }
    pub fn ret_dbl(&mut self, callee: &str, lo: f64, hi: f64) {
        self.specs.ret.push((callee.into(), RetSpec::Dbl(lo.to_bits(), hi.to_bits())));
    }
    pub fn out(&mut self, callee: &str, arg_idx: usize, nbytes: usize) {
        self.specs.out.push((callee.into(), arg_idx, nbytes));
    }
    pub fn str_arg(&mut self, callee: &str, arg_idx: usize) {
        self.specs.str_.push((callee.into(), arg_idx));
    }
    pub fn mem_arg(&mut self, callee: &str, arg_idx: usize, nbytes: usize) {
        self.specs.mem.push((callee.into(), arg_idx, nbytes));
    }
}

/// One function under test.
pub struct FuncDef {
    pub name: &'static str,
    /// argument kinds, one char each: i = integer, p = pointer, f = f32, d = f64
    pub argk: &'static str,
    /// return kind: v, i8/i16/i32/i64/b (bool), p, f, d
    pub retk: &'static str,
    pub ncases: u32,
    /// Build the state (globals, arena objects, specs, args) for case `c.idx`.
    pub build: fn(&mut Case),
    /// Run the port on `args`, return the raw return bits.
    pub call: unsafe fn(&[u64]) -> u64,
}

/// Result line for one case.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Outcome {
    pub ret: u64,
    pub state: u64,
    pub log: u64,
    pub ncalls: u64,
}

pub struct TuOracle<'a> {
    pub tu: &'a str,
    pub funcs: &'a [FuncDef],
    pub globals: Vec<GlobalDesc>,
    /// (C name, address) of the TU's functions that may be stored as pointers
    pub fn_table: Vec<(&'a str, usize)>,
}

fn ret_k(r: &str) -> K {
    match r {
        "v" => K::V,
        "b" => K::I(1),
        "i8" => K::I(8),
        "i16" => K::I(16),
        "i32" => K::I(32),
        "i64" => K::I(64),
        "p" => K::P,
        "f" => K::F,
        "d" => K::D,
        _ => panic!("bad ret kind {r}"),
    }
}

const GLOBALS_BASE: usize = 0x1000;

impl<'a> TuOracle<'a> {
    /// Arena offsets of the registered globals (sequential, 16-aligned).
    fn layout(&self) -> (Vec<usize>, usize) {
        let mut off = GLOBALS_BASE;
        let mut v = vec![];
        for g in &self.globals {
            off = (off + 15) & !15;
            v.push(off);
            off += g.size;
        }
        (v, (off + 0xFFF) & !0xFFF)
    }

    fn make_rec(&self, seed: u64, specs: Specs, hwm: usize) -> Rec {
        let mut fns: Vec<(u64, u64)> = self.fn_table.iter().map(|(n, a)| (*a as u64, fn_tag(n))).collect();
        fns.sort();
        let mut globs: Vec<(u64, u64, u64)> = self.globals.iter().map(|g| (g.addr as u64, g.size as u64, glob_tag(g.name))).collect();
        globs.sort();
        Rec { seed, log: FNV_OFF, ncalls: 0, specs, counts: HashMap::new(), fns, globs, hwm, text: String::new() }
    }

    /// Hash of the whole state: used arena words (globals region refreshed from the statics first).
    fn state_hash(&self, rec: &Rec, arena: &Arena, offs: &[usize], heap_start: usize) -> u64 {
        for (g, &o) in self.globals.iter().zip(offs) {
            unsafe { core::ptr::copy_nonoverlapping(g.addr, arena.base.add(o), g.size) };
        }
        let mut h = FNV_OFF;
        let end = rec.hwm.max(arena.hwm).max(heap_start);
        MAXUSED.fetch_max(end, core::sync::atomic::Ordering::Relaxed);
        let mut i = GLOBALS_BASE & !7;
        while i < end {
            let w = unsafe { (arena.base.add(i) as *const u64).read() };
            h = mix(h, rec.norm(w));
            i += 8;
        }
        h
    }

    /// Everything one case does on the Rust side, from a clean state to its outcome.
    pub fn run_case(&self, f: &FuncDef, idx: u32, init: &[Vec<u8>], arena: &mut Arena, trace: bool) -> (Outcome, String, Specs, Vec<u64>) {
        let (offs, heap_start) = self.layout();
        arena.reset(heap_start);
        // statics back to their initial bytes, then the builder
        for (g, b) in self.globals.iter().zip(init) {
            if g.mode == GMode::Rw {
                unsafe { core::ptr::copy_nonoverlapping(b.as_ptr(), g.addr, g.size) };
            }
        }
        let seed = splitmix(fnv_bytes(f.name.as_bytes()) ^ (idx as u64).wrapping_mul(0x1000_0000_01B3));
        let a2 = Arena { base: arena.base, hwm: arena.hwm };
        let mut c = Case { arena: a2, rng: Rng(seed ^ 0xA5A5), specs: Specs::default(), args: vec![], idx };
        c.specs.trace = trace;
        (f.build)(&mut c);
        arena.hwm = c.arena.hwm;
        let specs = c.specs.clone();
        let args = c.args.clone();
        let hwm = arena.hwm;
        let rec = self.make_rec(seed, specs.clone(), hwm);
        *REC.lock().unwrap_or_else(|e| e.into_inner()) = Some(rec);
        let ret = unsafe { (f.call)(&c.args) };
        let rec = REC.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap();
        let rk = ret_k(f.retk);
        let ret = match rk {
            K::V => 0,
            K::I(b) if b < 64 => ret & ((1u64 << b) - 1),
            K::P => rec.norm_ptr(ret),
            _ => ret,
        };
        let sh = self.state_hash(&rec, arena, &offs, heap_start);
        let o = Outcome { ret, state: sh, log: rec.log, ncalls: rec.ncalls };
        (o, rec.text.clone(), specs, args)
    }

    fn init_bytes(&self) -> Vec<Vec<u8>> {
        self.globals.iter().map(|g| unsafe { core::slice::from_raw_parts(g.addr, g.size) }.to_vec()).collect()
    }

    /// Write the C driver input (dev time): header, then every case with its image.
    pub fn export(&self, path: &std::path::Path) {
        let _lock = RUN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut arena = unsafe { Arena::get_once() };
        let init = self.init_bytes();
        let (offs, heap_start) = self.layout();
        let mut out = String::new();
        let _ = writeln!(out, "TU {}", self.tu);
        let _ = writeln!(out, "ARENA {:x} {:x}", ARENA_BASE, ARENA_SIZE);
        for (g, &o) in self.globals.iter().zip(&offs) {
            let _ = writeln!(out, "GLOBAL {} {:x} {:x} {}", g.name, o, g.size, if g.mode == GMode::Rw { "rw" } else { "ro" });
        }
        let _ = writeln!(out, "HEAP {heap_start:x}");
        for (n, _) in &self.fn_table {
            let _ = writeln!(out, "FNTAB {n}");
        }
        for (fi, f) in self.funcs.iter().enumerate() {
            let _ = writeln!(out, "FUNC {fi} {} {} {}", f.name, if f.argk.is_empty() { "-" } else { f.argk }, f.retk);
        }
        for (fi, f) in self.funcs.iter().enumerate() {
            for idx in 0..f.ncases {
                // build only (do not run): the C side runs the case
                arena.reset(heap_start);
                for (g, b) in self.globals.iter().zip(&init) {
                    if g.mode == GMode::Rw {
                        unsafe { core::ptr::copy_nonoverlapping(b.as_ptr(), g.addr, g.size) };
                    }
                }
                let seed = splitmix(fnv_bytes(f.name.as_bytes()) ^ (idx as u64).wrapping_mul(0x1000_0000_01B3));
                let a2 = Arena { base: arena.base, hwm: arena.hwm };
                let mut c = Case { arena: a2, rng: Rng(seed ^ 0xA5A5), specs: Specs::default(), args: vec![], idx };
                (f.build)(&mut c);
                arena.hwm = c.arena.hwm;
                for (g, &o) in self.globals.iter().zip(&offs) {
                    unsafe { core::ptr::copy_nonoverlapping(g.addr, arena.base.add(o), g.size) };
                }
                MAXUSED.fetch_max(arena.hwm, core::sync::atomic::Ordering::Relaxed);
                let nrec = self.make_rec(0, Specs::default(), arena.hwm);
                let _ = write!(out, "CASE {fi} {idx} {seed:x} {:x} {}", arena.hwm, c.args.len());
                for a in &c.args {
                    let _ = write!(out, " {a:x}");
                }
                out.push('\n');
                for (n, s) in &c.specs.ret {
                    match s {
                        RetSpec::Int(lo, hi) => { let _ = writeln!(out, "RET {n} int {lo} {hi}"); }
                        RetSpec::Ptr(a) => { let _ = writeln!(out, "RET {n} ptr {a:x}"); }
                        RetSpec::Pool(sz) => { let _ = writeln!(out, "RET {n} pool {sz}"); }
                        RetSpec::Seq(v) => {
                            let _ = write!(out, "RET {n} seq {}", v.len());
                            for a in v { let _ = write!(out, " {a:x}"); }
                            out.push('\n');
                        }
                        RetSpec::Flt(lo, hi) => { let _ = writeln!(out, "RET {n} flt {lo:x} {hi:x}"); }
                        RetSpec::Dbl(lo, hi) => { let _ = writeln!(out, "RET {n} dbl {lo:x} {hi:x}"); }
                    }
                }
                for (n, i, nb) in &c.specs.out { let _ = writeln!(out, "OUT {n} {i} {nb}"); }
                for (n, i) in &c.specs.str_ { let _ = writeln!(out, "STR {n} {i}"); }
                for (n, i, nb) in &c.specs.mem { let _ = writeln!(out, "MEM {n} {i} {nb}"); }
                // sparse image of [GLOBALS_BASE, hwm) in 64-byte blocks
                let end = (arena.hwm + 63) & !63;
                let mut o = GLOBALS_BASE & !63;
                while o < end {
                    let blk = unsafe { core::slice::from_raw_parts(arena.base.add(o), 64) };
                    if blk.iter().any(|&b| b != 0) {
                        let _ = write!(out, "IMG {o:x} ");
                        for b in blk { let _ = write!(out, "{b:02x}"); }
                        out.push('\n');
                    }
                    o += 64;
                }
                out.push_str("END\n");
            }
        }
        std::fs::write(path, out).unwrap();
    }

    /// Check every case against the C reference file (`R <func> <idx> <ret> <state> <log> <ncalls>`).
    /// Returns the list of mismatching `(func, idx)`; panics only on a missing file.
    pub fn check(&self, expected: &str) -> Vec<(String, u32, String)> {
        let _lock = RUN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut arena = unsafe { Arena::get_once() };
        let init = self.init_bytes();
        let mut exp: HashMap<(String, u32), Outcome> = HashMap::new();
        let mut bad_const = vec![];
        for l in expected.lines() {
            let t: Vec<&str> = l.split_whitespace().collect();
            if t.first() == Some(&"R") && t.len() == 7 {
                let h = |s: &str| u64::from_str_radix(s, 16).unwrap();
                exp.insert((t[1].to_string(), t[2].parse().unwrap()), Outcome { ret: h(t[3]), state: h(t[4]), log: h(t[5]), ncalls: h(t[6]) });
            } else if t.first() == Some(&"CONSTMISMATCH") {
                bad_const.push(l.to_string());
            }
        }
        let mut bad = vec![];
        for l in bad_const {
            bad.push(("const".to_string(), 0, l));
        }
        for f in self.funcs {
            for idx in 0..f.ncases {
                let want = match exp.get(&(f.name.to_string(), idx)) {
                    Some(w) => w.clone(),
                    None => {
                        bad.push((f.name.to_string(), idx, "no expected row".into()));
                        continue;
                    }
                };
                let (got, _, _, _) = self.run_case(f, idx, &init, &mut arena, false);
                if got != want {
                    bad.push((f.name.to_string(), idx, format!("got {got:x?} want {want:x?}")));
                }
            }
        }
        // leave the statics as found
        for (g, b) in self.globals.iter().zip(&init) {
            if g.mode == GMode::Rw {
                unsafe { core::ptr::copy_nonoverlapping(b.as_ptr(), g.addr, g.size) };
            }
        }
        bad
    }

    /// Debug: run one case with the call trace on and return the trace text.
    pub fn trace(&self, func: &str, idx: u32) -> String {
        let _lock = RUN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut arena = unsafe { Arena::get_once() };
        let init = self.init_bytes();
        let f = self.funcs.iter().find(|f| f.name == func).expect("no such function");
        let (o, text, _, args) = self.run_case(f, idx, &init, &mut arena, true);
        format!("args {args:x?}\nret {:x} state {:x} log {:x} calls {}\n{text}", o.ret, o.state, o.log, o.ncalls)
    }
}

static ARENA_PTR: Mutex<usize> = Mutex::new(0);
impl Arena {
    /// The process's one arena (mapped on first use).
    pub unsafe fn get_once() -> Arena {
        let mut g = ARENA_PTR.lock().unwrap_or_else(|e| e.into_inner());
        if *g == 0 {
            let a = Arena::get();
            *g = a.base as usize;
        }
        Arena { base: *g as *mut u8, hwm: 0 }
    }
}

/// Convenience for test files: export when `ORACLE_EXPORT=<dir>` is set, otherwise check against
/// `tests/data/tu_<tu>.txt` and panic on any mismatch.
pub fn run_tu(o: &TuOracle<'_>) {
    if let Ok(dir) = std::env::var("ORACLE_EXPORT") {
        std::fs::create_dir_all(&dir).unwrap();
        o.export(&std::path::Path::new(&dir).join(format!("tu_{}.cases", o.tu)));
        return;
    }
    if let Ok(spec) = std::env::var("ORACLE_TRACE") {
        // ORACLE_TRACE=func:idx
        let (f, i) = spec.split_once(':').unwrap();
        println!("{}", o.trace(f, i.parse().unwrap()));
        return;
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data").join(format!("tu_{}.txt", o.tu));
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing {}: run gen/tu/run.py {}", path.display(), o.tu));
    let bad = o.check(&expected);
    if !bad.is_empty() {
        let mut m = String::new();
        let mut per: Vec<(String, usize)> = vec![];
        for (f, i, why) in &bad {
            match per.iter_mut().find(|(n, _)| n == f) {
                Some(e) => {
                    e.1 += 1;
                    if e.1 > 2 {
                        continue;
                    }
                }
                None => per.push((f.clone(), 1)),
            }
            let _ = writeln!(m, "{f} case {i}: {why}");
        }
        for (n, c) in &per {
            let _ = writeln!(m, "  {n}: {c} bad");
        }
        panic!("{} mismatching cases\n{m}", bad.len());
    }
}

// ------------------------------------------------------------------------------------------
// builder / test helpers
// ------------------------------------------------------------------------------------------

/// Pointer argument `i` of a case.
pub fn p<T>(a: &[u64], i: usize) -> *mut T {
    a[i] as usize as *mut T
}
pub fn ai32(a: &[u64], i: usize) -> i32 {
    a[i] as u32 as i32
}
pub fn au32(a: &[u64], i: usize) -> u32 {
    a[i] as u32
}
pub fn au8(a: &[u64], i: usize) -> u8 {
    a[i] as u8
}
pub fn af32(a: &[u64], i: usize) -> f32 {
    f32::from_bits(a[i] as u32)
}
pub fn ab(a: &[u64], i: usize) -> bool {
    a[i] & 1 != 0
}
/// Return-value bits for the common kinds.
pub fn r<T: OVal>(v: T) -> u64 {
    v.to_bits()
}

/// `FuncDef` with an inline call body: `fd!("name", "pi", "v", ncases, build, |a| { ...; 0 })`.
#[macro_export]
macro_rules! fd {
    ($name:expr, $argk:expr, $retk:expr, $n:expr, $build:expr, |$a:ident| $body:block) => {
        $crate::oracle::FuncDef {
            name: $name,
            argk: $argk,
            retk: $retk,
            ncases: $n,
            build: $build,
            call: {
                #[allow(unused_unsafe, unused_variables)]
                unsafe fn f($a: &[u64]) -> u64 $body
                f
            },
        }
    };
}

impl Case {
    /// A zeroed buffer.
    pub fn zbuf(&mut self, n: usize) -> *mut u8 {
        self.arena.alloc_bytes(n)
    }
    /// A buffer of random bytes.
    pub fn buf(&mut self, n: usize) -> *mut u8 {
        let p = self.arena.alloc_bytes(n);
        self.rng.fill(p, n);
        p
    }
    /// An object of type `T` with random bytes (pointers inside are garbage: wire what is read).
    pub fn obj<T>(&mut self) -> *mut T {
        let p = self.arena.alloc::<T>();
        self.rng.fill(p.cast(), core::mem::size_of::<T>());
        p
    }
    /// A zeroed object.
    pub fn zobj<T>(&mut self) -> *mut T {
        self.arena.alloc::<T>()
    }
    /// Either null (1 in `den`) or a fresh random buffer of `n` bytes.
    pub fn buf_or_null(&mut self, n: usize, den: u64) -> *mut u8 {
        if self.rng.chance(1, den) {
            core::ptr::null_mut()
        } else {
            self.buf(n)
        }
    }
}
