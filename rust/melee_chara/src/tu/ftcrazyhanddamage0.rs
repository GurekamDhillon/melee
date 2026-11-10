//! melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c: ports folded from the hand-written character modules (consolidation step 20).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};

// ---- folded from chars blocks (consolidation step 20) ----
/// `fn_80157080` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn fn_80157080(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80157080");
    st::Fighter_ChangeMotionState(gobj, 344 /* 0x158 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftCh_Damage_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_Damage_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Damage_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Init_8015737C` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_Init_8015737C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_8015737C");
    st::Fighter_ChangeMotionState(gobj, 346 /* ftMh_MS_WaitSweep */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftCh_WaitSweep_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_WaitSweep_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WaitSweep_Phys");
    st::ft_80085134(gobj);
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftCh_Damage_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_Damage_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Damage_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_Damage2_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_Damage2_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Damage2_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_WaitSweep_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_WaitSweep_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WaitSweep_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_Damage_Anim` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_Damage_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Damage_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftCh_Init_80157170(gobj);
    }
}

/// `ftCh_Damage_Coll` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_Damage_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Damage_Coll"); }

/// `ftCh_Init_80157170` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c). No fused sites
/// (fma_sites.py): `cur_pos.x - x28` is a plain subtract.
pub fn ftCh_Init_80157170(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80157170");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs;
        (*fp).mv.ch.unk0.xC.x = (*fp).cur_pos.x - (*da).x28;
        (*fp).mv.ch.unk0.xC.y = (*da).x24;
        (*fp).mv.ch.unk0.xC.z = 0.0;
        st::Fighter_ChangeMotionState(gobj, melee_types::ftMh_MS_Damage2 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        crate::adapt::ft_PlaySFX(fp, 0x4E207, 127, 64);
    }
}

/// `ftCh_Damage2_Anim` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c). The else branch is
/// `ftCh_Init_80157170` inlined.
pub fn ftCh_Damage2_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Damage2_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            let mut pos: melee_types::Vec3 = core::mem::zeroed();
            melee_world::tu::mplib::mpFloorGetRight(0, &mut pos as *mut _ as *mut _);
            if (*fp).cur_pos.x > pos.x {
                ftCh_Init_8015737C(gobj);
            } else {
                ftCh_Init_80157170(gobj);
            }
        }
    }
}

/// `ftCh_Damage2_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_Damage2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Damage2_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs;
        st::ft_80085134(gobj);
        let xc = (*fp).mv.ch.unk0.xC;
        melee_fighter::tu::ftbosslib::ftBossLib_8015BE40(gobj, &xc, &mut (*fp).mv.ch.unk0.x18, (*da).x14, (*da).x10);
    }
}

/// `ftCh_Damage2_Coll` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_Damage2_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Damage2_Coll"); }

/// `ftCh_WaitSweep_Anim` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_WaitSweep_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WaitSweep_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        crate::tu::ftcrazyhandfingerbeam0::ext::ftCh_GrabUnk1_8015BC88(gobj);
    }
}

/// `ftCh_WaitSweep_Coll` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_WaitSweep_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_WaitSweep_Coll"); }
