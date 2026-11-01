//! src/melee/ft/kinds/ftMasterHand/ftmasterhandsweepwait.c: Master Hand sweep-wait state.
//! No fused sites. Smoke-tested by tests/tu_ftmasterhandsweepwait.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftmasterhandslap::ext::ftMh_MS_389_80151018;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.



pub fn ftMh_SweepWait_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_SweepWait_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_SweepWait_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_SweepWait_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_SweepWait_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandsweepwait.c).
pub fn ftMh_SweepWait_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_SweepWait_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_MS_348_80151BB8` (src/melee/ft/kinds/ftMasterHand/ftmasterhandsweepwait.c).
pub fn ftMh_MS_348_80151BB8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_348_80151BB8");
    st::Fighter_ChangeMotionState(gobj, 349 /* ftMh_MS_Slap */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftMh_SweepWait_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandsweepwait.c).
pub fn ftMh_SweepWait_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_SweepWait_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
