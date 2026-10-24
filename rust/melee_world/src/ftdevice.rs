//! The stage "device" registration tables of `src/melee/ft/ftdevice.c` (wind, bury / catch and
//! collision callbacks that stages register for the fighters) and their four registration
//! functions. They live here, in the lowest crate that needs them (the stage TUs register
//! devices); `melee_fighter::tu::ftdevice` (which reads the tables and ports the colour-overlay
//! selectors of the file) re-exports the statics and forwards its `ftco_*` functions to these.
//! Table overflow is an `HSD_ASSERTREPORT` (a halt) in the C and a panic here.
//! Checked by the fighter's `tests/tu_ftdevice.rs`, which runs on these items.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use core::ffi::c_void;
use melee_types::*;

pub static mut ft_80459A68: [ftDeviceUnk3; 1] = unsafe { core::mem::zeroed() };
pub static mut ft_80459A8C: [ftDeviceUnk3; 1] = unsafe { core::mem::zeroed() };
pub static mut ftDevice_BuryThings: [ftDeviceUnk5; 2] = unsafe { core::mem::zeroed() };
pub static mut ft_804D6578: ftDeviceUnk4 = unsafe { core::mem::zeroed() };
pub static mut ftDevice_BuryThingCount: i32 = 0;
pub static mut ft_804D6570: i32 = 0;

/// `ftCo_800C06C0`: clear all device tables.
pub fn ftCo_800C06C0() {
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

/// `ftCo_800C06E8`: register a wind device.
pub fn ftCo_800C06E8(gobj: *mut Ground_GObj, arg1: i32, func: ftDevice_Callback0) {
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
pub fn ftCo_800C0764(
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
pub fn ftCo_800C07F8(arg0: *mut Ground_GObj, arg1: u32, arg2: ftDevice_Callback0) {
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
