//! Generic hook sites for the wasm mod host (melee_modhost, hooks-v1 rows). A hooked game function becomes a wrapper:
//!
//! ```ignore
//! pub fn ftCo_Jump_Enter(g: G) {
//!     melee_core::mod_hook!(0x800CB250, [Arg::Fighter(g as _)], ftCo_Jump_Enter__orig(g));
//!     ftCo_Jump_Enter__orig(g)
//! }
//! ```
//!
//! With no mod loaded [`DISPATCH`] is None and the wrapper costs one load and branch, running the same body as before
//! (identical hashes by construction). With mods loaded, only the addresses in [`HOOKED`] reach the dispatcher; the
//! dispatcher runs the pre / replace / post chains and calls `orig` (the unmodified body) at most once per chain, or
//! once more per `gd_orig_call_*` from a replace hook.
#![allow(static_mut_refs)]

use core::ffi::c_void;

/// One argument as the game passed it. Pointers are raw; the host turns them into handles.
#[derive(Clone, Copy, Debug)]
pub enum Arg {
    /// A fighter GObj (`Fighter_GObj*`).
    Fighter(*mut c_void),
    /// An item GObj.
    Item(*mut c_void),
    /// Any GObj (gd_gobj).
    Gobj(*mut c_void),
    /// A `SpawnItem*` request.
    Spawn(*mut c_void),
    /// A pointer the ABI does not expose (gd_ptr: 0 to the mod).
    Ptr(*mut c_void),
    I32(i32),
    F32(f32),
}

/// A hooked function's result.
#[derive(Clone, Copy, Debug)]
pub enum Ret {
    Void,
    I32(i32),
    F32(f32),
    /// A pointer result (gd_gobj / gd_fighter / gd_item rows).
    Ptr(*mut c_void),
}

/// `(address, args, orig)` -> result. `orig` runs the unmodified body with the original arguments.
pub type Dispatch = fn(addr: u32, args: &[Arg], orig: &mut dyn FnMut() -> Ret) -> Ret;

pub static mut DISPATCH: Option<Dispatch> = None;
/// Sorted addresses that have at least one hook (set together with DISPATCH by the host).
pub static mut HOOKED: Vec<u32> = Vec::new();

/// The dispatcher when `addr` is hooked, else None (the common, mods-off path is the first branch).
#[inline(always)]
pub fn active(addr: u32) -> Option<Dispatch> {
    unsafe {
        let d = DISPATCH?;
        HOOKED.binary_search(&addr).is_ok().then_some(d)
    }
}

/// Result conversion for the wrapper macro.
pub trait HookRet: Sized {
    fn into_ret(self) -> Ret;
    fn from_ret(r: Ret) -> Self;
}
impl HookRet for () {
    fn into_ret(self) -> Ret { Ret::Void }
    fn from_ret(_: Ret) -> Self {}
}
impl HookRet for i32 {
    fn into_ret(self) -> Ret { Ret::I32(self) }
    fn from_ret(r: Ret) -> Self { if let Ret::I32(v) = r { v } else { 0 } }
}
impl HookRet for f32 {
    fn into_ret(self) -> Ret { Ret::F32(self) }
    fn from_ret(r: Ret) -> Self { if let Ret::F32(v) = r { v } else { 0.0 } }
}
impl<T> HookRet for *mut T {
    fn into_ret(self) -> Ret { Ret::Ptr(self as *mut c_void) }
    fn from_ret(r: Ret) -> Self { if let Ret::Ptr(p) = r { p as *mut T } else { core::ptr::null_mut() } }
}

/// Hook site: when `addr` is hooked, dispatch with `args` and `orig` (an expression calling the unmodified body) and
/// return the chain's result from the enclosing function. Otherwise falls through.
#[macro_export]
macro_rules! mod_hook {
    ($addr:expr, [$($arg:expr),* $(,)?], $orig:expr) => {
        if let Some(d) = $crate::modhook::active($addr) {
            #[allow(unused_unsafe)]
            let r = d($addr, &[$($arg),*], &mut || $crate::modhook::HookRet::into_ret(unsafe { $orig }));
            return $crate::modhook::HookRet::from_ret(r);
        }
    };
}
