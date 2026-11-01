
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

pub use crate::tu::ftdevice::ftCo_800C06C0_fp as ftCo_800C06C0;
pub use crate::tu::ftdevice::ftCo_800C06E8_fp as ftCo_800C06E8;
pub use crate::tu::ftdevice::ftCo_800C0764_fp as ftCo_800C0764;
pub use crate::tu::ftdevice::ftCo_800C07F8_fp as ftCo_800C07F8;
