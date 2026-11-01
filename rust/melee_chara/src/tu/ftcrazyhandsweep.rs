//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweep.c. No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandsweep.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;




pub fn ftCh_SweepLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_SweepLoop_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_SweepLoop_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_SweepLoop_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `fn_8015746C` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweep.c).
pub fn fn_8015746C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8015746C");
    st::Fighter_ChangeMotionState(gobj, 347 /* ftMh_MS_SweepLoop */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftCh_SweepLoop_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweep.c).
pub fn ftCh_SweepLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_SweepLoop_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_SweepLoop_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweep.c).
pub fn ftCh_SweepLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_SweepLoop_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
