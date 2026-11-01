//! gm_16A2.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use core::ptr::addr_of_mut;
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::gm::{lbl_8046B488, lbl_8046B668};

// ---- folded from gm.rs (consolidation steps 21/22) ----
/// `gm_1601_GetUnkData`.
pub unsafe fn gm_1601_GetUnkData() -> *mut lbl_8046B488_t {
    melee_core::ct!("gm_1601_GetUnkData");
    addr_of_mut!(lbl_8046B488)
}

/// `gm_80169394_fp`.
pub unsafe fn gm_80169394_fp() -> i32 {
    melee_core::ct!("gm_80169394_fp");
    (*gm_1601_GetUnkData()).unk_10_b3() as i32
}

/// `gm_80169520_fp`.
pub unsafe fn gm_80169520_fp() -> *mut core::ffi::c_void {
    melee_core::ct!("gm_80169520_fp");
    (*gm_1601_GetUnkData()).x20.as_mut_ptr() as *mut core::ffi::c_void
}

/// `gm_80169530_fp`.
pub unsafe fn gm_80169530_fp() -> *mut core::ffi::c_void {
    melee_core::ct!("gm_80169530_fp");
    (*gm_1601_GetUnkData()).xA2.as_mut_ptr() as *mut core::ffi::c_void
}

/// `gm_8016A1F8_fp`.
pub unsafe fn gm_8016A1F8_fp() -> bool {
    melee_core::ct!("gm_8016A1F8_fp");
    (*gm_1601_GetUnkData()).unk_10_b1() != 0
}

/// `gm_8016A97C_fp`.
pub unsafe fn gm_8016A97C_fp() -> *mut core::ffi::c_void {
    melee_core::ct!("gm_8016A97C_fp");
    lbl_8046B668.arr2.as_mut_ptr() as *mut core::ffi::c_void
}

/// `gm_8016A98C_fp`.
pub unsafe fn gm_8016A98C_fp() -> *mut lbl_8046B668_t {
    melee_core::ct!("gm_8016A98C_fp");
    addr_of_mut!(lbl_8046B668)
}
