//! src/melee/ft/kinds/ftDonkey/ftdonkeyheavyfall.c: DK carrying a heavy item, falling. No float
//! arithmetic (the animation rate argument is the constant 0). Smoke-tested by
//! tests/tu_ftdonkeyheavyfall.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/5): re-exported under this TU.
pub use crate::chars::r2::ftDonkey::ftdonkeyheavyfall::ftDk_HeavyFall_Phys;
pub use crate::chars::r5::ftDonkey::ftdonkeyheavyfall::ftDk_HeavyFall_Coll;

pub fn ftDk_HeavyFall_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_HeavyFall_IASA");
    if melee_fighter::tu::ftCo_ItemThrow::ftCo_80094EA4(gobj) {
        return;
    }
}

/// `ftDk_MS_347_800E0378(gobj)`: jump input check; returns the C `bool` (0/1).
pub fn ftDk_MS_347_800E0378(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftDk_MS_347_800E0378");
    let result = melee_fighter::tu::ftCo_Jump::ftCo_Jump_GetInput(gobj);
    if result != 0 {
        crate::tu::ftdonkeyheavyjump::ftDk_MS_348_800E04A4(gobj, result as i32);
        1
    } else {
        0
    }
}

pub fn ftDk_MS_347_800E03C0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_MS_347_800E03C0");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        st::ftCommon_8007D5D4(&mut *fp);
        let donkey_attr = (*fp).x2CC;
        st::Fighter_ChangeMotionState(gobj, (*donkey_attr).motion_state + 7, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_SetAnimRate(gobj, 0.0);
        melee_fighter::tu::ftCo_Jump::ftCo_800CB110(gobj, true, 1.0);
    }
}
