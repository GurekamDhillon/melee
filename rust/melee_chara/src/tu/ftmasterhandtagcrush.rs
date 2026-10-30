//! src/melee/ft/kinds/ftMasterHand/ftmasterhandtagcrush.c. No float math, no fused sites.
//! Smoke-tested by tests/tu_ftmasterhandtagcrush.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_fighter::tu::ftbosslib as bl;
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.
pub use crate::chars::r2::ftMasterHand::ftmasterhandtagcrush::ftMh_TagCrush_Phys;

pub fn ftMh_TagCrush_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagCrush_Anim");
    unsafe {
        if bl::ftBossLib_8015C31C() || bl::ftBossLib_8015C3A0() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
            crate::tu::ftmasterhandwait12::ftMh_MS_389_80151018(gobj);
        }
    }
}

pub fn ftMh_TagCrush_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagCrush_IASA");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if st::Player_GetPlayerSlotType((*fp).player_id as i32) == 0 {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

pub fn ftMh_TagCrush_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_TagCrush_Coll");}

pub fn ftMh_MS_382_801552F8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_382_801552F8");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[1] = 0;
        let gobj_2 = bl::ftBossLib_GetFighterGObj(Ft_Kind_CrezyH);
        if !bl::ftBossLib_8015C31C() {
            crate::tu::ftcrazyhandsqueeze::ftCh_Init_8015A3F4(gobj_2);
        }
        (*fp).x1A5C = gobj_2;
        st::Fighter_ChangeMotionState(gobj, ftMh_MS_TagApplaud as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}
