//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandtagrockpaper.c. No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandtagrockpaper.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

use melee_fighter::tu::ftbosslib as bl;




pub fn ftCh_TagRockPaper_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagRockPaper_Anim");
    if bl::ftBossLib_8015C2E0() || bl::ftBossLib_8015C358() {
        crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj);
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::Fighter_ChangeMotionState(gobj, 0x180, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftCh_TagRockPaper_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_TagRockPaper_Phys");}

pub fn ftCh_TagRockPaper_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_TagRockPaper_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `fn_8015AAC8` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandtagrockpaper.c).
pub fn fn_8015AAC8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8015AAC8");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::Fighter_ChangeMotionState(gobj, 384 /* 0x180 */, 0u32, fp.cur_anim_frame, 1.0_f32, 0.0_f32, core::ptr::null_mut());
}

/// `ftCh_TagRockPaper_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandtagrockpaper.c).
pub fn ftCh_TagRockPaper_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagRockPaper_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
