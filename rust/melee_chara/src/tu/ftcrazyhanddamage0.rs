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
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_Damage2_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_Damage2_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Damage2_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_WaitSweep_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddamage0.c).
pub fn ftCh_WaitSweep_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WaitSweep_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
