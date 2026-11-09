//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueezing1.c. No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandsqueezing1.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

use melee_fighter::tu::ftbosslib as bl;




pub fn ftCh_Init_8015A2B0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_8015A2B0");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).x1A5C = bl::ftBossLib_GetFighterGObj(Ft_Kind_MasterH);
        st::Fighter_ChangeMotionState(gobj, 0x179, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftCh_Squeezing1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Squeezing1_Anim");
    unsafe {
        if bl::ftBossLib_8015C2E0() || bl::ftBossLib_8015C358() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
            crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj);
        }
    }
}

pub fn ftCh_Squeezing1_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Squeezing1_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_Squeezing1_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueezing1.c).
pub fn ftCh_Squeezing1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Squeezing1_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Squeezing1_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueezing1.c).
pub fn ftCh_Squeezing1_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Squeezing1_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
