//! src/melee/ft/kinds/ftMasterHand/ftmasterhandslam.c: Master Hand slam state and the tag-crush
//! entry. No fused sites. Smoke-tested by tests/tu_ftmasterhandslam.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftmasterhandslap::ext::ftMh_MS_389_80151018;
use crate::tu::ftmasterhandsqueezing::{ext::ftMh_CaptureWaitMasterHand_80155D6C, ftCo_800DE2A8, ftCo_800DE7C0};
use melee_fighter::tu::ftbosslib as bl;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.


pub use crate::tu::ftcrazyhandsqueezing1::ftCh_Init_8015A2B0;

pub fn ftMh_Slam_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Slam_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[0] != 0 {
            (*fp).cmd_vars[0] = 0;
            ftMh_CaptureWaitMasterHand_80155D6C((*fp).victim_gobj as *mut HSD_GObj, ftCo_MS_ThrownMasterHand as i32);
            if !(*fp).victim_gobj.is_null() {
                let victim = (*fp).victim_gobj as *mut HSD_GObj;
                st::ftCommon_8007E2F4(&mut *fp, 0);
                ftCo_800DE2A8(gobj, victim);
                ftCo_800DE7C0(victim, core::ptr::null_mut(), false);
            }
            (*fp).mv.mh.unk0.x20 = 0;
        }
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_Slam_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Slam_Coll");}

pub fn ftMh_MS_380_80155194(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_380_80155194");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let gobj1 = bl::ftBossLib_GetFighterGObj(Ft_Kind_CrezyH);
        if !bl::ftBossLib_8015C31C() {
            ftCh_Init_8015A2B0(gobj1);
        }
        (*fp).x1A5C = gobj1;
        st::Fighter_ChangeMotionState(gobj, 382 /* ftMh_MS_TagCrush */, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_Slam_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandslam.c).
pub fn ftMh_Slam_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Slam_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_Slam_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandslam.c).
pub fn ftMh_Slam_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Slam_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
