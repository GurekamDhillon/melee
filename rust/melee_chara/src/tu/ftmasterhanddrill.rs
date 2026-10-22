//! src/melee/ft/kinds/ftMasterHand/ftmasterhanddrill.c: Master Hand drill / walk-wait states.
//! No fused sites (fma_sites.py); the countdown compares an f32 against an s32 attribute as floats.
//! Smoke-tested by tests/tu_ftmasterhanddrill.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/3/8): re-exported under this TU.
pub use crate::chars::r2::ftMasterHand::ftmasterhanddrill::ftMh_WalkWait_Phys;
pub use crate::chars::r3::ftMasterHand::ftmasterhanddrill::ftMh_MS_352_801521DC;
pub use crate::chars::r8::ftMasterHand::ftmasterhanddrill::{ftMh_Drill_IASA, ftMh_WalkWait_IASA};

/// Calls into TUs that are not ported yet. Each is a settable hook.
pub mod ext {
    pub use crate::tu::ftmasterhanddamage0::ext::ftMh_MS_389_80151018;
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FTBOSSLIB_8015C010, ftBossLib_8015C010, melee_fighter::tu::ftbosslib::ftBossLib_8015C010, (gobj: *mut melee_types::HSD_GObj, x_diff_max: f32));
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FTBOSSLIB_8015C190, ftBossLib_8015C190, melee_fighter::tu::ftbosslib::ftBossLib_8015C190, (gobj: *mut melee_types::HSD_GObj));
}

pub fn ftMh_WalkWait_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_WalkWait_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftMh_Drill_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_Drill_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::ft_80085134(gobj);
        (*fp).mv.mh.unk0.x0 -= 1.0;
        if (*fp).mv.mh.unk0.x0 > (*da).x84 as f32 || (*fp).mv.mh.unk0.x0 < 0.0 {
            (*fp).self_vel.x = 0.0;
        } else {
            ext::ftBossLib_8015C010(gobj, (*da).x80);
        }
        ext::ftBossLib_8015C190(gobj);
    }
}
