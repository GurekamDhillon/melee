//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueeze.c. No float math (no fused sites).
//! Smoke-tested by tests/tu_ftcrazyhandsqueeze.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

use melee_fighter::tu::ftbosslib as bl;

/// Already ported (rounds 2/8): re-exported under this TU.



pub fn ftCh_Init_8015A3F4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_8015A3F4");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[1] = 0;
        (*fp).x1A5C = bl::ftBossLib_GetFighterGObj(Ft_Kind_MasterH);
        st::Fighter_ChangeMotionState(gobj, 0x17A, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftCh_Squeeze_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Squeeze_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[1] != 0 {
            bl::ftBossLib_8015C5F8(gobj);
            (*fp).cmd_vars[1] = 0;
        }
        if bl::ftBossLib_8015C2E0() || bl::ftBossLib_8015C358() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
            crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj);
        }
    }
}

pub fn ftCh_Squeeze_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Squeeze_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_Squeeze_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueeze.c).
pub fn ftCh_Squeeze_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Squeeze_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Squeeze_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueeze.c).
pub fn ftCh_Squeeze_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Squeeze_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
