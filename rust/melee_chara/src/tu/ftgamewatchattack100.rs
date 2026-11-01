//! src/melee/ft/kinds/ftGameWatch/ftgamewatchattack100.c: Mr. Game & Watch's rapid jab
//! (Start / Loop / End). No float arithmetic, so no fused sites (tools/rust-port/fma_sites.py).
//! Smoke-tested by tests/tu_ftgamewatchattack100.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftgamewatch::ftGw_Init_8014A538;
use crate::tu::ftgamewatchattack11::cb_ftGw_Attack11_ItemGreenhouseSetup;
use melee_fighter::tu::ftCo_Attack100 as co;
use melee_types::*;

/// Already ported (rounds 2/3/12): re-exported under this TU.




#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// Mr. Game & Watch's Rapid Jab Start Action State handler.
pub fn ftGw_Attack100Start_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100Start_Enter");
    unsafe {
        let fp = fpp(gobj);
        co::ftCo_800D6B00(gobj, ftGw_MS_Attack100Start as i32);
        (*fp).accessory4_cb = Some(cb_ftGw_Attack11_ItemGreenhouseSetup);
    }
}

/// Rapid Jab Start Collision callback.
pub fn ftGw_Attack100Start_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100Start_Coll");
    st::ft_80084104(gobj);
    ftGw_Init_8014A538(gobj);
}

/// Rapid Jab Loop Animation callback.
pub fn ftGw_Attack100Loop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100Loop_Anim");
    co::ftCo_800D6C60(gobj, ftGw_Attack100End_Enter);
}

/// Rapid Jab Loop IASA callback.
pub fn ftGw_Attack100Loop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100Loop_IASA");
    co::ftCo_Attack100Loop_IASA(gobj);
}

/// Rapid Jab Loop Collision callback.
pub fn ftGw_Attack100Loop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100Loop_Coll");
    st::ft_80084104(gobj);
    ftGw_Init_8014A538(gobj);
}

/// Rapid Jab End Animation callback.
pub fn ftGw_Attack100End_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100End_Anim");
    co::ftCo_Attack100End_Anim(gobj);
}

/// Rapid Jab End Collision callback.
pub fn ftGw_Attack100End_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100End_Coll");
    st::ft_80084104(gobj);
    ftGw_Init_8014A538(gobj);
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftGw_Attack100Start_IASA` (src/melee/ft/kinds/ftGameWatch/ftgamewatchattack100.c).
pub fn ftGw_Attack100Start_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100Start_IASA");
    unsafe {
        return;
    }
}

/// `ftGw_Attack100End_IASA` (src/melee/ft/kinds/ftGameWatch/ftgamewatchattack100.c).
pub fn ftGw_Attack100End_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100End_IASA");
    unsafe {
        return;
    }
}

/// `ftGw_Attack100Start_Phys` (src/melee/ft/kinds/ftGameWatch/ftgamewatchattack100.c).
pub fn ftGw_Attack100Start_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100Start_Phys");
    st::ft_80084FA8(gobj);
}

/// `ftGw_Attack100Loop_Phys` (src/melee/ft/kinds/ftGameWatch/ftgamewatchattack100.c).
pub fn ftGw_Attack100Loop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100Loop_Phys");
    st::ft_80084FA8(gobj);
}

/// `ftGw_Attack100End_Phys` (src/melee/ft/kinds/ftGameWatch/ftgamewatchattack100.c).
pub fn ftGw_Attack100End_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100End_Phys");
    st::ft_80084FA8(gobj);
}

/// `ftGw_Attack100Start_Anim` (src/melee/ft/kinds/ftGameWatch/ftgamewatchattack100.c).
pub fn ftGw_Attack100Start_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100Start_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            crate::chars::r3::ftGameWatch::ftgamewatchattack100::ftGw_Attack100Loop_Enter(gobj);
        }
    }
}

/// `ftGw_Attack100Loop_Enter` (src/melee/ft/kinds/ftGameWatch/ftgamewatchattack100.c).
pub fn ftGw_Attack100Loop_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100Loop_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 343 /* ftGw_MS_Attack100Loop */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(crate::chars::cb::ftgamewatchattack11__ftGw_Attack11_DecideAction);
    }
}

/// `ftGw_Attack100End_Enter` (src/melee/ft/kinds/ftGameWatch/ftgamewatchattack100.c).
pub fn ftGw_Attack100End_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack100End_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 344 /* ftGw_MS_Attack100End */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(crate::chars::cb::ftgamewatchattack11__ftGw_Attack11_DecideAction);
    }
}
