//! src/melee/ft/kinds/ftDonkey/ftdonkeyheavyjump.c: DK carrying a heavy item, jump squat. No
//! float arithmetic (a copy of the startup-lag attribute). Smoke-tested by
//! tests/tu_ftdonkeyheavyjump.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (round 5): re-exported under this TU.
pub use crate::chars::r5::ftDonkey::ftdonkeyheavyjump::ftDk_HeavyJump_Coll;

pub fn ftDk_HeavyJump_IASA(gobj: *mut HSD_GObj) {
    if melee_fighter::tu::ftCo_ItemThrow::ftCo_80094EA4(gobj) {
        return;
    }
}

pub fn ftDk_HeavyJump_Phys(gobj: *mut HSD_GObj) {
    melee_fighter::tu::ftCo_Jump::ftCo_Jump_Phys_Inner(gobj);
}

pub fn ftDk_MS_348_800E04A4(gobj: *mut HSD_GObj, arg1: i32) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let donkey_attr = (*fp).x2CC;
        (*fp).mv.dk.unk7.x4 = arg1;
        (*fp).mv.dk.unk7.x0 = 0;
        (*fp).mv.dk.unk7.x8 = (*donkey_attr).cargo_hold.x24_JUMP_STARTUP_LAG;
        let donkey_attr2 = (*fp).x2CC;
        st::Fighter_ChangeMotionState(gobj, (*donkey_attr2).motion_state + 5, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_SetAnimRate(gobj, 0.0);
    }
}

pub fn ftDk_HeavyWait1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).mv.dk.unk7.x8 <= 0.0 {
            crate::tu::ftdonkeyheavyfall::ftDk_MS_347_800E03C0(gobj);
        }
        (*fp).mv.dk.unk7.x8 -= 1.0;
    }
}
