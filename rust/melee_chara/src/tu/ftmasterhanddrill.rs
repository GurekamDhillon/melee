//! src/melee/ft/kinds/ftMasterHand/ftmasterhanddrill.c: Master Hand drill / walk-wait states.
//! No fused sites (fma_sites.py); the countdown compares an f32 against an s32 attribute as floats.
//! Smoke-tested by tests/tu_ftmasterhanddrill.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/3/8): re-exported under this TU.




/// Calls into TUs that are not ported yet. Each is a settable hook.
pub mod ext {
    pub use crate::tu::ftmasterhanddamage0::ext::ftMh_MS_389_80151018;
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FTBOSSLIB_8015C010, ftBossLib_8015C010, melee_fighter::tu::ftbosslib::ftBossLib_8015C010, (gobj: *mut melee_types::HSD_GObj, x_diff_max: f32));
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FTBOSSLIB_8015C190, ftBossLib_8015C190, melee_fighter::tu::ftbosslib::ftBossLib_8015C190, (gobj: *mut melee_types::HSD_GObj));
}

pub fn ftMh_WalkWait_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_WalkWait_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_WalkWait_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_WalkWait_Coll");}

pub fn ftMh_Drill_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Drill_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_Drill_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Drill_Phys");
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

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_WalkWait_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhanddrill.c).
pub fn ftMh_WalkWait_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_WalkWait_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_MS_352_801521DC` (src/melee/ft/kinds/ftMasterHand/ftmasterhanddrill.c).
pub fn ftMh_MS_352_801521DC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_352_801521DC");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 354 /* ftMh_MS_Drill */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.mh.unk0.x0 = f32::from_bits(0x42D60000) /* 107 */;
    }
}

/// `ftMh_WalkWait_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhanddrill.c).
pub fn ftMh_WalkWait_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_WalkWait_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_Drill_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhanddrill.c).
pub fn ftMh_Drill_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Drill_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
