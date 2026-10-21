//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandslam.c: Crazy Hand slam / fail / tag-crush /
//! tag-applaud states. `--x24 <= 0` is a float countdown. No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandslam.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftcrazyhandfingerbeam0::ext::ftCh_GrabUnk1_8015BC88;
use crate::tu::ftcrazyhandfingergun2::ftCh_UpdateBossMotion;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.
pub use crate::chars::r2::ftCrazyHand::ftcrazyhandslam::{ftCh_Fail_Phys, ftCh_GrabUnk1_8015A888, ftCh_Slam_Phys, ftCh_TagCrush_Phys};
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandslam::{ftCh_Fail_IASA, ftCh_Slam_IASA, ftCh_TagApplaud_IASA, ftCh_TagCrush_IASA};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    crate::ch_hook!(FTCH_GRABUNK1_8015B800, ftCh_GrabUnk1_8015B800, (gobj: *mut HSD_GObj));
}

pub fn ftCh_Slam_Anim(gobj: *mut HSD_GObj) {
    if melee_fighter::tu::ftbosslib::ftBossLib_8015C2E0()
        || melee_fighter::tu::ftbosslib::ftBossLib_8015C358()
        || st::ftAnim_IsFramesRemaining(gobj) == 0
    {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            (*fp).self_vel.x = 0.0;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
        }
        ftCh_GrabUnk1_8015A888(gobj);
    }
}

pub fn ftCh_Slam_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftCh_Fail_Anim(gobj: *mut HSD_GObj) {
    if melee_fighter::tu::ftbosslib::ftBossLib_8015C31C() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
        }
        ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_Fail_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftCh_TagCrush_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_TagCrush_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftCh_TagApplaud_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).mv.ch.unk0.x24 -= 1.0;
        if (*fp).mv.ch.unk0.x24 <= 0.0 && (*fp).cmd_vars[0] != 0 {
            ext::ftCh_GrabUnk1_8015B800((*fp).victim_gobj as *mut HSD_GObj);
            (*fp).cmd_vars[0] = 0;
        }
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_TagApplaud_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        ftCh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftCh_TagApplaud_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).mv.ch.unk0.x18 == 0.0 {
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            (*fp).self_vel.x = 0.0;
        }
    }
}
