//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueezing1.c. No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandsqueezing1.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

use melee_fighter::tu::ftbosslib as bl;

pub use crate::chars::r2::ftCrazyHand::ftcrazyhandsqueezing1::ftCh_Squeezing1_Phys;
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandsqueezing1::ftCh_Squeezing1_IASA;

pub fn ftCh_Init_8015A2B0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).x1A5C = bl::ftBossLib_GetFighterGObj(Ft_Kind_MasterH);
        st::Fighter_ChangeMotionState(gobj, 0x179, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftCh_Squeezing1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        if bl::ftBossLib_8015C2E0() || bl::ftBossLib_8015C358() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
            crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj);
        }
    }
}

pub fn ftCh_Squeezing1_Coll(_gobj: *mut HSD_GObj) {}
