//! src/melee/ft/kinds/ftDonkey/ftdonkeyheavywait1.c: DK carrying a heavy item, kneebend/landing.
//! No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftdonkeyheavywait1.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use super::ftdonkeyheavyturn::ext as turn_ext;
use crate::stubs as st;
use melee_types::*;

pub use crate::chars::r2::ftDonkey::ftdonkeyheavywait1::ftDk_HeavyWait1_Phys;
pub use crate::chars::r5::ftDonkey::ftdonkeyheavywait1::ftDk_HeavyWait1_Coll;

pub mod ext {
    use melee_types::*;
    type G = *mut HSD_GObj;
    pub static mut FTCO_KNEEBEND_CHECK_SHORTHOP: Option<fn(G)> = None;
    pub fn ftCo_KneeBend_Check_ShortHop(g: G) {
        match unsafe { FTCO_KNEEBEND_CHECK_SHORTHOP } {
            Some(f) => f(g),
            None => melee_fighter::tu::ftCo_KneeBend::ftCo_KneeBend_Check_ShortHop(g),
        }
    }
    pub static mut FTCO_LANDING_ENTER: Option<fn(G, i32, bool, u32, f32, f32)> = None;
    pub fn ftCo_Landing_Enter(g: G, msid: i32, allow_interrupt: bool, flags: u32, anim_start: f32, anim_speed: f32) {
        match unsafe { FTCO_LANDING_ENTER } {
            Some(f) => f(g, msid, allow_interrupt, flags, anim_start, anim_speed),
            None => melee_fighter::tu::ftCo_Landing::ftCo_Landing_Enter(g, msid, allow_interrupt, flags, anim_start, anim_speed),
        }
    }
}

pub fn ftDk_HeavyWait1_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_HeavyWait1_IASA");
    if !turn_ext::ftCo_80094EA4(gobj) {
        ext::ftCo_KneeBend_Check_ShortHop(gobj);
    }
}

/// Start the kneebend landing (also the ground-collision callback of the heavy fall states).
pub fn ftDk_MS_346_800E05E4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_MS_346_800E05E4");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let a = (*fp).x2CC;
        (*fp).mv.dk.unk8.x4 = (*a).cargo_hold.x28_LANDING_LAG;
        let a = (*fp).x2CC;
        ext::ftCo_Landing_Enter(gobj, (*a).motion_state + 8, true, Ft_MF_None as u32, 0.0, 1.0);
        st::ftAnim_SetAnimRate(gobj, 0.0);
    }
}

/// `ftDk_MS_346_800E05E4` as a Fighter callback entry (what `chars::cb` points at).
pub unsafe extern "C" fn cb_ftDk_MS_346_800E05E4(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftDk_MS_346_800E05E4");
    ftDk_MS_346_800E05E4(g);
}

pub fn ftDk_HeavyLanding_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_HeavyLanding_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).mv.dk.unk8.x4 <= 0.0 {
            super::ftdonkeyheavywait0::ftDk_MS_341_800DF980(gobj);
        }
        (*fp).mv.dk.unk8.x4 -= 1.0;
    }
}
