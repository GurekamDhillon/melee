//! ftdevice.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use core::ffi::c_void;
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::ftdevice::{ftDevice_BuryThingCount, ftDevice_BuryThings, ft_80459A68, ft_80459A8C, ft_804D6570, ft_804D6578};

// ---- folded from ftdevice.rs (consolidation steps 21/22) ----
/// `ftCo_800C06C0_fp`: clear all device tables.
pub fn ftCo_800C06C0_fp() {
    melee_core::ct!("ftCo_800C06C0_fp");
    unsafe {
        ft_804D6570 = 0;
        ftDevice_BuryThingCount = 0;
        ft_804D6578.x0 = 0;
        ft_80459A68[0].ground = core::ptr::null_mut();
        for i in 0..2 {
            ftDevice_BuryThings[i].x0 = core::ptr::null_mut();
        }
        ft_80459A8C[0].ground = core::ptr::null_mut();
    }
}

/// `ftCo_800C06E8_fp`: register a wind device.
pub fn ftCo_800C06E8_fp(gobj: *mut Ground_GObj, arg1: i32, func: ftDevice_Callback0) {
    melee_core::ct!("ftCo_800C06E8_fp");
    unsafe {
        for e in ft_80459A68.iter_mut() {
            if e.ground.is_null() {
                e.ground = gobj;
                e.r#type = arg1 as u32;
                e.active_cb = func;
                ft_804D6578.x0 = ft_804D6578.x0.wrapping_add(1);
                return;
            }
        }
    }
    panic!("fighter chk device wind func num over!");
}

/// `ftCo_800C0764_fp`: register a bury ("catch") device.
pub fn ftCo_800C0764_fp(
    arg0: *mut Ground_GObj,
    arg1: u32,
    arg2: Option<unsafe extern "C" fn(*mut c_void, *mut Fighter_GObj) -> i32>,
) {
    melee_core::ct!("ftCo_800C0764_fp");
    unsafe {
        for e in ftDevice_BuryThings.iter_mut() {
            if e.x0.is_null() {
                e.x0 = arg0 as *mut c_void;
                e.x4 = arg1;
                e.cb = arg2;
                ftDevice_BuryThingCount = ftDevice_BuryThingCount.wrapping_add(1);
                return;
            }
        }
    }
    panic!("fighter chk device catch func num over!");
}

/// `ftCo_800C07F8_fp`: register a collision device.
pub fn ftCo_800C07F8_fp(arg0: *mut Ground_GObj, arg1: u32, arg2: ftDevice_Callback0) {
    melee_core::ct!("ftCo_800C07F8_fp");
    unsafe {
        for e in ft_80459A8C.iter_mut() {
            if e.ground.is_null() {
                e.ground = arg0;
                e.r#type = arg1;
                e.active_cb = arg2;
                ft_804D6570 = ft_804D6570.wrapping_add(1);
                return;
            }
        }
    }
    panic!("fighter chk device coll func num over!");
}
