//! Calls from a ported TU into TUs that are not ported yet.
//!
//! Each such callee is declared in the TU file's `mod ext` with [`ext_fns!`], under exactly the
//! C function's name. Until the callee's TU is ported (and the declaration is replaced by a
//! `pub use` of the real function) a call goes through [`EXT_HOOK`]: unset, it panics with the
//! callee's name (the TODO stub); the TU state oracles (`tests/tu_*.rs`) install a mock there
//! that records the call and returns values from a seeded stream, the same stream the C oracle's
//! mock of that function draws from.
#![allow(static_mut_refs)]

/// Dispatcher installed by tests: callee name, raw arguments (integers sign-extended, floats as
/// their bit patterns, pointers as addresses) -> raw return value.
pub type ExtHook = unsafe fn(&'static str, &[u64]) -> u64;

pub static mut EXT_HOOK: Option<ExtHook> = None;

/// Answer for a callee that has no port and no default: the engine installs the "feature off" answers of its host
/// seams here (`melee_engine::seams_gen::ext_fallback`); unset, the call panics with the callee's name.
pub static mut EXT_FALLBACK: Option<ExtHook> = None;

/// A scalar that crosses the hook as a `u64`.
pub trait Val: Copy {
    fn to_u64(self) -> u64;
    fn from_u64(v: u64) -> Self;
}

macro_rules! val_int {
    ($($t:ty => $w:ty),*) => {$(
        impl Val for $t {
            #[inline] fn to_u64(self) -> u64 { self as $w as u64 }
            #[inline] fn from_u64(v: u64) -> Self { v as $t }
        }
    )*};
}
val_int!(i8 => i64, i16 => i64, i32 => i64, i64 => i64, isize => i64,
         u8 => u64, u16 => u64, u32 => u64, u64 => u64, usize => u64);

impl Val for bool {
    #[inline] fn to_u64(self) -> u64 { self as u64 }
    #[inline] fn from_u64(v: u64) -> Self { v != 0 }
}
impl Val for f32 {
    #[inline] fn to_u64(self) -> u64 { self.to_bits() as u64 }
    #[inline] fn from_u64(v: u64) -> Self { f32::from_bits(v as u32) }
}
impl Val for f64 {
    #[inline] fn to_u64(self) -> u64 { self.to_bits() }
    #[inline] fn from_u64(v: u64) -> Self { f64::from_bits(v) }
}
impl<T> Val for *mut T {
    #[inline] fn to_u64(self) -> u64 { self as usize as u64 }
    #[inline] fn from_u64(v: u64) -> Self { v as usize as *mut T }
}
impl<T> Val for *const T {
    #[inline] fn to_u64(self) -> u64 { self as usize as u64 }
    #[inline] fn from_u64(v: u64) -> Self { v as usize as *const T }
}
macro_rules! val_fnptr {
    ($(($($A:ident),*)),*) => {$(
        impl<$($A,)* R> Val for Option<unsafe extern "C" fn($($A),*) -> R> {
            #[inline] fn to_u64(self) -> u64 { self.map_or(0, |f| f as usize as u64) }
            #[inline] fn from_u64(v: u64) -> Self { if v == 0 { None } else { Some(unsafe { core::mem::transmute(v as usize) }) } }
        }
    )*};
}
val_fnptr!((), (A0), (A0, A1), (A0, A1, A2), (A0, A1, A2, A3), (A0, A1, A2, A3, A4), (A0, A1, A2, A3, A4, A5));

/// Declare stubs for callees in TUs that are not ported yet.
///
/// ```ignore
/// ext_fns! {
///     fn pl_80039450(a0: i32);                       // TODO: panics until the callee's TU is ported
///     fn gm_GetCurrentGameMode() -> u8;
///     fn fn_8017AD04(a0: i32, a1: i32) -> i32 = crate::gm::fn_8017AD04;   // ported elsewhere: called
/// }
/// ```
/// Under a test hook ([`EXT_HOOK`]) every stub goes to the hook, ported or not, so the state
/// oracles can mock the callee exactly as the C oracle does.
/// MSL `memzero(void*, ssize_t)`: the real behaviour, used by ext stubs when no hook is installed.
///
/// # Safety
/// `p` is valid for `n` bytes.
pub unsafe fn memzero(p: *mut core::ffi::c_void, n: i32) { unsafe { melee_runtime::tu::lb_00B0::memzero(p as _, n as _); } }

impl Val for () {
    #[inline] fn to_u64(self) -> u64 { 0 }
    #[inline] fn from_u64(_v: u64) -> Self {}
}

/// `()` when no return type is given.
#[doc(hidden)]
#[macro_export]
macro_rules! ext_rty {
    () => { () };
    ($r:ty) => { $r };
}

/// The call made when no hook is installed: the ported callee, or the TODO panic.
#[doc(hidden)]
#[macro_export]
macro_rules! ext_default {
    ([] ($($a:ident),*) $name:ident) => {
        melee_core::unported!(concat!("TODO ext: ", stringify!($name), " is in a TU that is not ported yet"))
    };
    ([$def:path] ($($a:ident),*) $name:ident) => {
        return $def($($a),*)
    };
}

/// The `None` arm of [`ext_fns!`]: the ported callee, else the engine's fallback, else the TODO panic.
#[doc(hidden)]
#[macro_export]
macro_rules! ext_none {
    ([] ($($a:ident),*) $name:ident [$($r:ty)?]) => {
        match $crate::tu::ext_hook::EXT_FALLBACK {
            Some(f) => <$crate::ext_rty!($($r)?) as $crate::tu::ext_hook::Val>::from_u64(
                f(stringify!($name), &[$($crate::tu::ext_hook::Val::to_u64($a)),*])),
            None => melee_core::unported!(concat!("TODO ext: ", stringify!($name), " is in a TU that is not ported yet")),
        }
    };
    ([$def:path] ($($a:ident),*) $name:ident [$($r:ty)?]) => {
        return $def($($a),*)
    };
}

/// Default arm of a local `ext_fns!`: the verified port (`= path`), or the TODO panic.
#[doc(hidden)]
#[macro_export]
macro_rules! ext_dflt {
    ([] ($($a:ident),*) $name:ident) => {
        melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported"))
    };
    ([$def:path] ($($a:ident),*) $name:ident) => {
        return $def($($a),*)
    };
}

#[macro_export]
macro_rules! ext_fns {
    () => {};
    // a callee whose address is taken as a C callback (`extern "C" fn` pointer)
    ($(#[$m:meta])* extern "C" fn $name:ident($($a:ident : $t:ty),* $(,)?) $(-> $r:ty)? $(= $def:path)? ; $($rest:tt)*) => {
        $(#[$m])*
        #[allow(non_snake_case, unused_variables, clippy::missing_safety_doc)]
        pub unsafe extern "C" fn $name($($a: $t),*) -> $crate::ext_rty!($($r)?) {
            match $crate::tu::ext_hook::EXT_HOOK {
                Some(h) => <$crate::ext_rty!($($r)?) as $crate::tu::ext_hook::Val>::from_u64(
                    h(stringify!($name), &[$($crate::tu::ext_hook::Val::to_u64($a)),*])),
                None => { $crate::ext_none!([$($def)?] ($($a),*) $name [$($r)?]) }
            }
        }
        $crate::ext_fns! { $($rest)* }
    };
    ($(#[$m:meta])* fn $name:ident($($a:ident : $t:ty),* $(,)?) $(-> $r:ty)? $(= $def:path)? ; $($rest:tt)*) => {
        $(#[$m])*
        #[allow(non_snake_case, unused_variables, clippy::missing_safety_doc)]
        pub unsafe fn $name($($a: $t),*) -> $crate::ext_rty!($($r)?) {
            match $crate::tu::ext_hook::EXT_HOOK {
                Some(h) => <$crate::ext_rty!($($r)?) as $crate::tu::ext_hook::Val>::from_u64(
                    h(stringify!($name), &[$($crate::tu::ext_hook::Val::to_u64($a)),*])),
                None => { $crate::ext_none!([$($def)?] ($($a),*) $name [$($r)?]) }
            }
        }
        $crate::ext_fns! { $($rest)* }
    };
}

/// Small records returned by value cross the hook as their bytes.
impl Val for melee_types::_GXColor {
    #[inline] fn to_u64(self) -> u64 { u32::from_le_bytes([self.r, self.g, self.b, self.a]) as u64 }
    #[inline] fn from_u64(v: u64) -> Self { let b = (v as u32).to_le_bytes(); melee_types::_GXColor { r: b[0], g: b[1], b: b[2], a: b[3] } }
}
