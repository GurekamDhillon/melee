//! src/melee/ft/kinds/ftMasterHand/ftmasterhandtagapplaud.c: Master Hand tag-applaud state.
//! No fused sites. Smoke-tested by tests/tu_ftmasterhandtagapplaud.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use crate::tu::ftmasterhandslap::ext::ftMh_MS_389_80151018;
use melee_fighter::tu::ftbosslib as bl;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.
pub use crate::chars::r2::ftMasterHand::ftmasterhandtagapplaud::ftMh_TagApplaud_Phys;
pub use crate::chars::r8::ftMasterHand::ftmasterhandtagapplaud::ftMh_TagApplaud_IASA;

/// `ftCh_Init_8015A560` lives in ftcrazyhandthrow.c (ported: crate::tu::ftcrazyhandthrow).
pub use crate::tu::ftcrazyhandthrow::ftCh_Init_8015A560;

pub fn ftMh_TagApplaud_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[1] != 0 {
            bl::ftBossLib_8015C5F8(gobj);
            (*fp).cmd_vars[1] = 0;
        }
    }
    if bl::ftBossLib_8015C31C() || bl::ftBossLib_8015C3A0() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
        }
        ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_TagApplaud_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftMh_MS_383_80155484(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let gobj1 = bl::ftBossLib_GetFighterGObj(Ft_Kind_CrezyH);
        if !bl::ftBossLib_8015C31C() {
            ftCh_Init_8015A560(gobj1);
        }
        (*fp).x1A5C = gobj1;
        st::Fighter_ChangeMotionState(gobj, 384 /* ftMh_MS_TagRockPaper */, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}
