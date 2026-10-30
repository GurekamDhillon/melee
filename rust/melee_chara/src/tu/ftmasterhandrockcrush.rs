//! src/melee/ft/kinds/ftMasterHand/ftmasterhandrockcrush.c: Master Hand rock-crush rise / wait.
//! No fused sites. Smoke-tested by tests/tu_ftmasterhandrockcrush.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/3/8): re-exported under this TU.
pub use crate::chars::r2::ftMasterHand::ftmasterhandrockcrush::{ftMh_MS_354_80152370, ftMh_MS_355_8015247C, ftMh_RockCrushUp_Phys};
pub use crate::chars::r3::ftMasterHand::ftmasterhandrockcrush::ftMh_MS_356_801525E0;
pub use crate::chars::r8::ftMasterHand::ftmasterhandrockcrush::{ftMh_RockCrushUp_IASA, ftMh_RockCrushWait_IASA};

/// `ftMh_Drill_Coll` (defined in this C file).
pub fn ftMh_Drill_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Drill_Coll");}

pub fn ftMh_RockCrushUp_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_RockCrushUp_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
            ftMh_MS_355_8015247C(gobj);
            (*fp).mv.mh.unk0.x8 = (*da).xA0;
        }
    }
}

pub fn ftMh_RockCrushUp_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_RockCrushUp_Coll");}

pub fn ftMh_RockCrushWait_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_RockCrushWait_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).mv.mh.unk0.x8 = (*fp).mv.mh.unk0.x8.wrapping_sub(1);
        if (*fp).mv.mh.unk0.x8 == 0 {
            ftMh_MS_356_801525E0(gobj);
        }
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::Fighter_ChangeMotionState(gobj, 356 /* ftMh_MS_RockCrushWait */, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftMh_RockCrushWait_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_RockCrushWait_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::ft_80085134(gobj);
        melee_fighter::tu::ftbosslib::ftBossLib_8015C010(gobj, (*da).xA4);
        melee_fighter::tu::ftbosslib::ftBossLib_8015C190(gobj);
    }
}

pub fn ftMh_RockCrushWait_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_RockCrushWait_Coll");}
