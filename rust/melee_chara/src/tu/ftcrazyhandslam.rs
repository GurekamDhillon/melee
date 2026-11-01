//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandslam.c: Crazy Hand slam / fail / tag-crush /
//! tag-applaud states. `--x24 <= 0` is a float countdown. No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandslam.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftcrazyhandfingerbeam0::ext::ftCh_GrabUnk1_8015BC88;
use crate::tu::ftcrazyhandfingergun2::ftCh_UpdateBossMotion;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.



/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    crate::port_hook!(FTCH_GRABUNK1_8015B800, ftCh_GrabUnk1_8015B800, crate::tu::ftcrazyhandcapturewaitcrazyhand::ftCh_GrabUnk1_8015B800, (gobj: *mut HSD_GObj));
}

pub fn ftCh_Slam_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Slam_Anim");
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

pub fn ftCh_Slam_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Slam_Coll");}

pub fn ftCh_Fail_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Fail_Anim");
    if melee_fighter::tu::ftbosslib::ftBossLib_8015C31C() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
        }
        ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_Fail_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Fail_Coll");}

pub fn ftCh_TagCrush_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagCrush_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_TagCrush_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_TagCrush_Coll");}

pub fn ftCh_TagApplaud_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagApplaud_Anim");
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
    melee_core::ct!("ftCh_TagApplaud_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        ftCh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftCh_TagApplaud_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagApplaud_Coll");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).mv.ch.unk0.x18 == 0.0 {
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            (*fp).self_vel.x = 0.0;
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_Slam_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandslam.c).
pub fn ftCh_Slam_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Slam_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Fail_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandslam.c).
pub fn ftCh_Fail_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Fail_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_GrabUnk1_8015A888` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandslam.c).
pub fn ftCh_GrabUnk1_8015A888(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_GrabUnk1_8015A888");
    st::Fighter_ChangeMotionState(gobj, 382 /* 0x17E */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftCh_TagCrush_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandslam.c).
pub fn ftCh_TagCrush_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagCrush_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Slam_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandslam.c).
pub fn ftCh_Slam_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Slam_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_Fail_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandslam.c).
pub fn ftCh_Fail_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Fail_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_TagCrush_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandslam.c).
pub fn ftCh_TagCrush_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagCrush_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_TagApplaud_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandslam.c).
pub fn ftCh_TagApplaud_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagApplaud_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftCh_TagApplaud_Coll_fp(fp: &mut Fighter) {
    melee_core::ct!("ftCh_TagApplaud_Coll_fp");
    crate::chars::ftCrazyHand::stop_if_x18_zero(fp)
}
