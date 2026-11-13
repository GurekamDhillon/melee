//! Fighter engine (ft/*.c), shared fighter states (ftCommon), players (pl).
//!
//! Port group 3 (docs/rust-port/inventory.md). Raw c2rust output for this group's TUs is
//! written to `src/raw/` by `tools/rust-port/transpile.sh`; it is reference input for the
//! hand port and is deliberately not declared as a module here.
#![allow(dead_code)]

pub use melee_core;

pub mod types;
pub mod compat;

/// Mechanical c2rust translation of every group-3 TU (feature `c2r`; generated, gitignored).
#[cfg(feature = "c2r")]
pub mod c2r;
pub mod neutral_hash;
pub mod bridge;
pub mod chardata;
pub mod platform;
pub mod rng;
pub mod hooks;
pub mod ftcommon;
pub mod ftcommon_c;
pub mod ftcommon2;
pub mod stale;
pub mod player;
pub mod slippi_online;
pub mod script_caps;
pub mod ftlib;
pub mod plbonus;
pub mod plbonus2;
pub mod plbonus3;
pub mod ft0877;
pub mod ft0881;
pub mod ft0892;
pub mod pltrick;
pub mod ftcommon3;
pub mod ftaction;
pub mod fighter;
pub mod ftanim;
pub mod script_override;
pub mod anim_override;
pub mod ftcoll;
pub mod ft084e;
pub mod ft0df;
pub mod ftdmgfall;
pub mod ftcpu;
pub mod tu;

/// Pointer-valued arguments of the oracle seams (gen/mtu/rewire.py): references and raw pointers as `*mut c_void`
/// (taken by reference, so a `&mut` argument is reborrowed, not moved).
pub mod mtu_ptr {
    pub trait AsPtr { fn ptr_ref(&self) -> *mut core::ffi::c_void; }
    impl<T: ?Sized> AsPtr for &mut T { fn ptr_ref(&self) -> *mut core::ffi::c_void { &**self as *const T as *const () as *mut core::ffi::c_void } }
    impl<T: ?Sized> AsPtr for &T { fn ptr_ref(&self) -> *mut core::ffi::c_void { &**self as *const T as *const () as *mut core::ffi::c_void } }
    impl<T: ?Sized> AsPtr for *mut T { fn ptr_ref(&self) -> *mut core::ffi::c_void { *self as *const () as *mut core::ffi::c_void } }
    impl<T: ?Sized> AsPtr for *const T { fn ptr_ref(&self) -> *mut core::ffi::c_void { *self as *const () as *mut core::ffi::c_void } }
    pub fn ptr<P: AsPtr>(p: &P) -> *mut core::ffi::c_void { p.ptr_ref() }
    /// The address of a place (a struct field passed by pointer in the C).
    pub fn at<T>(p: &T) -> *mut core::ffi::c_void { p as *const T as *mut core::ffi::c_void }
}
