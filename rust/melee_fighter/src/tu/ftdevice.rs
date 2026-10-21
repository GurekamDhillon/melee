//! src/melee/ft/ftdevice.c: stage "device" registration tables (wind / bury / collision
//! callbacks registered by grounds) and the colour-overlay selectors.
//! `ftCo_800C0658/0674/0694` were already ported in `crate::ft0df`; re-exported here.
#![allow(non_upper_case_globals, static_mut_refs)]

use crate::types::*;
use core::ffi::c_void;

pub use crate::ft0df::{ftco_800c0658, ftco_800c0674, ftco_800c0694};

pub static mut ft_80459A68: [ftDeviceUnk3; 1] = unsafe { core::mem::zeroed() };
pub static mut ft_80459A8C: [ftDeviceUnk3; 1] = unsafe { core::mem::zeroed() };
pub static mut ftDevice_BuryThings: [ftDeviceUnk5; 2] = unsafe { core::mem::zeroed() };
pub static mut ft_804D6578: ftDeviceUnk4 = unsafe { core::mem::zeroed() };
pub static mut ftDevice_BuryThingCount: i32 = 0;
pub static mut ft_804D6570: i32 = 0;

/// `ftCo_800C06B4`: `(int) &fp->x508.x28_colanim.ptr->x7B` (pointer truncated to int as in C).
pub fn ftco_800c06b4(fp: &Fighter) -> i32 {
    let p = unsafe { fp.x508.x28_colanim.ptr } as usize;
    p.wrapping_add(core::mem::offset_of!(ColorOverlay_UnkInner, x7B)) as i32
}

/// `ftCo_800C06C0`: clear all device tables.
pub fn ftco_800c06c0() {
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

/// `ftCo_800C06E8`: register a wind device. Full table: HSD_ASSERTREPORT (fatal).
pub fn ftco_800c06e8(gobj: *mut Ground_GObj, arg1: i32, func: ftDevice_Callback0) {
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

/// `ftCo_800C0764`: register a bury ("catch") device.
pub fn ftco_800c0764(
    arg0: *mut Ground_GObj,
    arg1: u32,
    arg2: Option<unsafe extern "C" fn(*mut c_void, *mut Fighter_GObj) -> i32>,
) {
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

/// `ftCo_800C07F8`: register a collision device.
pub fn ftco_800c07f8(arg0: *mut Ground_GObj, arg1: u32, arg2: ftDevice_Callback0) {
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
