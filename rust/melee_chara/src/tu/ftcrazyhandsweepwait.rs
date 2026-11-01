//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c. The only float op is the f32 add
//! `self_vel.x += da->x40` (no multiply: no fused sites).
//! Smoke-tested by tests/tu_ftcrazyhandsweepwait.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;





pub fn ftCh_SweepWait_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_SweepWait_Coll");}

pub fn ftCh_Slap_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Slap_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        st::ft_80085134(gobj);
        (*fp).self_vel.x += (*da).x40;
    }
}

/// `static void fn_801577B4(gobj)`: accessory4 callback of the slap.
pub unsafe extern "C" fn fn_801577B4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801577B4");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        if (*fp).cur_pos.x > (*da).x3C {
            (*fp).self_vel.x = 0.0;
            ftCh_Init_801577F8(gobj);
        }
    }
}

pub fn ftCh_Walk2_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Walk2_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_Slap_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Slap_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `fn_8015755C` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c).
pub fn fn_8015755C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8015755C");
    st::Fighter_ChangeMotionState(gobj, 348 /* ftMh_MS_SweepWait */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftCh_SweepWait_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c).
pub fn ftCh_SweepWait_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_SweepWait_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Init_801577F8` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c).
pub fn ftCh_Init_801577F8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_801577F8");
    st::Fighter_ChangeMotionState(gobj, 350 /* ftMh_MS_Walk2 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftCh_Walk2_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c).
pub fn ftCh_Walk2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Walk2_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_SweepWait_Anim` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c).
pub fn ftCh_SweepWait_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_SweepWait_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            crate::chars::r3::ftCrazyHand::ftcrazyhandsweepwait::ftCh_Init_8015764C(gobj);
        }
    }
}

/// `ftCh_Init_8015764C` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c).
pub fn ftCh_Init_8015764C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_8015764C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 349 /* 0x15D */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftcrazyhandsweepwait__fn_801577B4);
    }
}

/// `ftCh_Slap_Anim` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c).
pub fn ftCh_Slap_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Slap_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            st::Fighter_ChangeMotionState(gobj, 349 /* 0x15D */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
            (*fp).accessory4_cb = Some(crate::chars::cb::ftcrazyhandsweepwait__fn_801577B4);
        }
    }
}

/// `ftCh_SweepWait_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c).
pub fn ftCh_SweepWait_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_SweepWait_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_Slap_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c).
pub fn ftCh_Slap_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Slap_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_Walk2_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c).
pub fn ftCh_Walk2_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Walk2_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
