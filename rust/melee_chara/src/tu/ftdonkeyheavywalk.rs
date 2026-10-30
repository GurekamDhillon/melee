//! src/melee/ft/kinds/ftDonkey/ftdonkeyheavywalk.c: DK carrying a heavy item, walk.
//! No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftdonkeyheavywalk.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use melee_types::*;

pub use crate::chars::r5::ftDonkey::ftdonkeyheavywalk::ftDk_HeavyWalk_Coll;

pub mod ext {
    use melee_types::*;
    type G = *mut HSD_GObj;
    pub static mut FTWALKCOMMON_800DFC70: Option<fn(G) -> bool> = None;
    pub fn ftWalkCommon_800DFC70(g: G) -> bool {
        match unsafe { FTWALKCOMMON_800DFC70 } {
            Some(f) => f(g),
            None => melee_fighter::tu::ftwalkcommon::ftwalkcommon_800dfc70(g),
        }
    }
    pub static mut FTWALKCOMMON_800DFCA4: Option<fn(G, i32, u32, f32, f32, f32, f32, f32, f32, f32, f32)> = None;
    #[allow(clippy::too_many_arguments)]
    pub fn ftWalkCommon_800DFCA4(g: G, msid: i32, fl: u32, start: f32, a: f32, b: f32, c: f32, d: f32, e: f32, f_: f32, mul: f32) {
        match unsafe { FTWALKCOMMON_800DFCA4 } {
            Some(f) => f(g, msid, fl, start, a, b, c, d, e, f_, mul),
            None => melee_fighter::tu::ftwalkcommon::ftwalkcommon_800dfca4(g, msid, fl, start, a, b, c, d, e, f_, mul),
        }
    }
    pub static mut FTWALKCOMMON_800DFDDC: Option<fn(G)> = None;
    pub fn ftWalkCommon_800DFDDC(g: G) {
        match unsafe { FTWALKCOMMON_800DFDDC } {
            Some(f) => f(g),
            None => melee_fighter::tu::ftwalkcommon::ftwalkcommon_800dfddc(g),
        }
    }
    pub static mut FTWALKCOMMON_800DFEC8: Option<fn(G, fn(G, f32))> = None;
    pub fn ftWalkCommon_800DFEC8(g: G, cb: fn(G, f32)) {
        match unsafe { FTWALKCOMMON_800DFEC8 } {
            Some(f) => f(g, cb),
            None => melee_fighter::tu::ftwalkcommon::ftwalkcommon_800dfec8(g, cb),
        }
    }
    pub static mut FTWALKCOMMON_800E0060: Option<fn(G)> = None;
    pub fn ftWalkCommon_800E0060(g: G) {
        match unsafe { FTWALKCOMMON_800E0060 } {
            Some(f) => f(g),
            None => melee_fighter::tu::ftwalkcommon::ftwalkcommon_800e0060(g),
        }
    }
}

/// Starts the walk when the stick is past the walk threshold in the facing direction.
pub fn ftDk_MS_341_800DFA98(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftDk_MS_341_800DFA98");
    if ext::ftWalkCommon_800DFC70(gobj) {
        ftDk_MS_341_800DFAE4(gobj, 0.0);
        true
    } else {
        false
    }
}

pub fn ftDk_MS_341_800DFAE4(gobj: *mut HSD_GObj, argf: f32) {
    melee_core::ct!("ftDk_MS_341_800DFAE4");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let a = (*fp).x2CC;
        ext::ftWalkCommon_800DFCA4(
            gobj,
            (*a).motion_state + 1,
            0,
            argf,
            (*a).x8,
            (*a).xC,
            (*a).x10,
            (*a).x14,
            (*a).x18,
            (*a).x1C,
            1.0,
        );
    }
}

pub fn ftDk_HeavyWalk_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_HeavyWalk_Anim");
    ext::ftWalkCommon_800DFDDC(gobj);
}

pub fn ftDk_HeavyWalk_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_HeavyWalk_IASA");
    if !super::ftdonkeyheavyturn::ext::ftCo_80094EA4(gobj)
        && !super::ftdonkeyheavyturn::ext::ftDk_MS_347_800E0378(gobj)
        && !super::ftdonkeyheavywait0::ftDk_MS_341_800DF938(gobj)
    {
        ext::ftWalkCommon_800DFEC8(gobj, ftDk_MS_341_800DFAE4);
    }
}

pub fn ftDk_HeavyWalk_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_HeavyWalk_Phys");
    ext::ftWalkCommon_800E0060(gobj);
}
