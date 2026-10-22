//! src/melee/ft/kinds/ftGameWatch/ftgamewatchattack100.c: Mr. Game & Watch's rapid jab
//! (Start / Loop / End). No float arithmetic, so no fused sites (tools/rust-port/fma_sites.py).
//! Smoke-tested by tests/tu_ftgamewatchattack100.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftgamewatch::ftGw_Init_8014A538;
use crate::tu::ftgamewatchattack11::cb_ftGw_Attack11_ItemGreenhouseSetup;
use melee_fighter::tu::ftCo_Attack100 as co;
use melee_types::*;

/// Already ported (rounds 2/3/12): re-exported under this TU.
pub use crate::chars::r12::ftGameWatch::ftgamewatchattack100::{ftGw_Attack100End_IASA, ftGw_Attack100Start_IASA};
pub use crate::chars::r2::ftGameWatch::ftgamewatchattack100::{
    ftGw_Attack100End_Phys, ftGw_Attack100Loop_Phys, ftGw_Attack100Start_Phys,
};
pub use crate::chars::r3::ftGameWatch::ftgamewatchattack100::{
    ftGw_Attack100End_Enter, ftGw_Attack100Loop_Enter, ftGw_Attack100Start_Anim,
};

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// Mr. Game & Watch's Rapid Jab Start Action State handler.
pub fn ftGw_Attack100Start_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        co::ftCo_800D6B00(gobj, ftGw_MS_Attack100Start as i32);
        (*fp).accessory4_cb = Some(cb_ftGw_Attack11_ItemGreenhouseSetup);
    }
}

/// Rapid Jab Start Collision callback.
pub fn ftGw_Attack100Start_Coll(gobj: *mut HSD_GObj) {
    st::ft_80084104(gobj);
    ftGw_Init_8014A538(gobj);
}

/// Rapid Jab Loop Animation callback.
pub fn ftGw_Attack100Loop_Anim(gobj: *mut HSD_GObj) {
    co::ftCo_800D6C60(gobj, ftGw_Attack100End_Enter);
}

/// Rapid Jab Loop IASA callback.
pub fn ftGw_Attack100Loop_IASA(gobj: *mut HSD_GObj) {
    co::ftCo_Attack100Loop_IASA(gobj);
}

/// Rapid Jab Loop Collision callback.
pub fn ftGw_Attack100Loop_Coll(gobj: *mut HSD_GObj) {
    st::ft_80084104(gobj);
    ftGw_Init_8014A538(gobj);
}

/// Rapid Jab End Animation callback.
pub fn ftGw_Attack100End_Anim(gobj: *mut HSD_GObj) {
    co::ftCo_Attack100End_Anim(gobj);
}

/// Rapid Jab End Collision callback.
pub fn ftGw_Attack100End_Coll(gobj: *mut HSD_GObj) {
    st::ft_80084104(gobj);
    ftGw_Init_8014A538(gobj);
}
