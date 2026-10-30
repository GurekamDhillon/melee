//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandthrowncrazyhand.c. No float math beyond a copy of
//! `facing_dir`. Smoke-tested by tests/tu_ftcrazyhandthrowncrazyhand.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// `ftCh_GrabUnk1_8015B850(gobj, msid)`: `msid` is unused by the C.
pub fn ftCh_GrabUnk1_8015B850(gobj: *mut HSD_GObj, _msid: i32) {
    melee_core::ct!("ftCh_GrabUnk1_8015B850");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let victim_fp = (*(*fp).victim_gobj).user_data as *mut Fighter;
        (*fp).facing_dir = (*victim_fp).facing_dir;
        (*fp).mv.ch.unk4.x0 = 0;
        st::Fighter_ChangeMotionState(gobj, 0x153, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).set_invisible(0);
        (*fp).accessory1_cb = Some(melee_fighter::tu::ftCo_Thrown::ftCo_800DE508_cb);
        st::ftCommon_8007E2F4(&mut *fp, 511);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftCo_ThrownCrazyHand_Anim(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_ThrownCrazyHand_Anim");}
pub fn ftCo_ThrownCrazyHand_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_ThrownCrazyHand_IASA");}
pub fn ftCo_ThrownCrazyHand_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_ThrownCrazyHand_Phys");}
pub fn ftCo_ThrownCrazyHand_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_ThrownCrazyHand_Coll");}
