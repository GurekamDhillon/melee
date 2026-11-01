//! src/melee/ft/ftdevice.c: stage "device" registration tables (wind / bury / collision
//! callbacks registered by grounds) and the colour-overlay selectors.
//! `ftCo_800C0658/0674/0694` were already ported in `crate::ft0df`; re-exported here.
#![allow(non_upper_case_globals, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::ftcommon::abs_c;
#[allow(unused_imports)]
use crate::ftcommon2 as fc2;
#[allow(unused_imports)]
use crate::ftcommon3::ftco_get_cstick_angle;
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use crate::ft0df::{G, fp};
use crate::types::*;
use core::ffi::c_void;


/// The device tables are owned by `melee_world::ftdevice` (the stage TUs register devices).
pub use melee_world::ftdevice::{
    ftDevice_BuryThingCount, ftDevice_BuryThings, ft_804D6570, ft_804D6578, ft_80459A68, ft_80459A8C,
};

/// `ftCo_800C06B4`: `(int) &fp->x508.x28_colanim.ptr->x7B` (pointer truncated to int as in C).
pub fn ftco_800c06b4(fp: &Fighter) -> i32 {
    melee_core::ct!("ftco_800c06b4");
    let p = unsafe { fp.x508.x28_colanim.ptr } as usize;
    p.wrapping_add(core::mem::offset_of!(ColorOverlay_UnkInner, x7B)) as i32
}

/// `ftCo_800C06C0`: clear all device tables.
pub fn ftco_800c06c0() {
    melee_core::ct!("ftco_800c06c0");
    melee_world::ftdevice::ftCo_800C06C0()
}

/// `ftCo_800C06E8`: register a wind device. Full table: HSD_ASSERTREPORT (fatal).
pub fn ftco_800c06e8(gobj: *mut Ground_GObj, arg1: i32, func: ftDevice_Callback0) {
    melee_core::ct!("ftco_800c06e8");
    melee_world::ftdevice::ftCo_800C06E8(gobj, arg1, func)
}

/// `ftCo_800C0764`: register a bury ("catch") device.
pub fn ftco_800c0764(
    arg0: *mut Ground_GObj,
    arg1: u32,
    arg2: Option<unsafe extern "C" fn(*mut c_void, *mut Fighter_GObj) -> i32>,
) {
    melee_core::ct!("ftco_800c0764");
    melee_world::ftdevice::ftCo_800C0764(arg0, arg1, arg2)
}

/// `ftCo_800C07F8`: register a collision device.
pub fn ftco_800c07f8(arg0: *mut Ground_GObj, arg1: u32, arg2: ftDevice_Callback0) {
    melee_core::ct!("ftco_800c07f8");
    melee_world::ftdevice::ftCo_800C07F8(arg0, arg1, arg2)
}

// ---- folded from ft0df.rs (consolidation steps 21/22) ----
// ---- ftdevice.c leaves -------------------------------------------------------------------

/// `ftCo_800C0658`: `x408` if its colanim is set (C: `(int) ptr != 0`), else `x488`.
pub fn ftco_800c0658(f: &mut Fighter) -> *mut ColorOverlay {
    melee_core::ct!("ftco_800c0658");
    if unsafe { f.x408.x28_colanim.ptr } as usize as u32 != 0 { &mut f.x408 } else { &mut f.x488 }
}

pub fn ftco_800c0674(g: G) -> *mut ColorOverlay { melee_core::ct!("ftco_800c0674"); ftco_800c0658(fp(g)) }

pub fn ftco_800c0694(f: &mut Fighter) -> i32 { melee_core::ct!("ftco_800c0694"); unsafe { (*ftco_800c0658(f)).x28_colanim.i } }
