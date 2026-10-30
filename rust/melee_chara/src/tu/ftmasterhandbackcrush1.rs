//! src/melee/ft/kinds/ftMasterHand/ftmasterhandbackcrush1.c. No float arithmetic. Smoke-tested by
//! tests/tu_ftmasterhandbackcrush1.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

pub fn ftMh_BackCrush_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_BackCrush_Coll");}

pub fn ftMh_MS_371_801541C8(gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    melee_core::ct!("ftMh_MS_371_801541C8");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 372, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.mh.unk0.x4 = arg1;
    }
}
