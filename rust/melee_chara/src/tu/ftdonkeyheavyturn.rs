//! src/melee/ft/kinds/ftDonkey/ftdonkeyheavyturn.c: DK carrying a heavy item, turn.
//! No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftdonkeyheavyturn.rs.
//! `ext` also hosts the two callees shared by the other heavy-* TUs of this shard.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

pub use crate::chars::r2::ftDonkey::ftdonkeyheavyturn::ftDk_HeavyTurn_Phys;
pub use crate::chars::r5::ftDonkey::ftdonkeyheavyturn::ftDk_HeavyTurn_Coll;

pub mod ext {
    use melee_types::*;
    type G = *mut HSD_GObj;
    pub static mut FTCO_TURN_ANIM_INNER: Option<fn(G)> = None;
    pub fn ftCo_Turn_Anim_Inner(g: G) {
        match unsafe { FTCO_TURN_ANIM_INNER } {
            Some(f) => f(g),
            None => melee_fighter::tu::ftCo_Turn::ftCo_Turn_Anim_Inner(g),
        }
    }
    pub static mut FTCO_80094EA4: Option<fn(G) -> bool> = None;
    pub fn ftCo_80094EA4(g: G) -> bool {
        match unsafe { FTCO_80094EA4 } {
            Some(f) => f(g),
            None => melee_fighter::tu::ftCo_ItemThrow::ftCo_80094EA4(g),
        }
    }
    /// TODO: defined in ftdonkeyheavyfall.c / ftdonkeyheavyjump.c (another shard).
    pub static mut FTDK_MS_347_800E0378: Option<fn(G) -> bool> = None;
    pub fn ftDk_MS_347_800E0378(g: G) -> bool {
        match unsafe { FTDK_MS_347_800E0378 } {
            Some(f) => f(g),
            None => { return unsafe { crate::tu::ftdonkeyheavyfall::ftDk_MS_347_800E0378(g as _) != 0 }; },
        }
    }
}

pub fn ftDk_HeavyTurn_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_HeavyTurn_Anim");
    ext::ftCo_Turn_Anim_Inner(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        super::ftdonkeyheavywait0::ftDk_MS_341_800DF980(gobj);
    }
}

pub fn ftDk_HeavyTurn_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_HeavyTurn_IASA");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).mv.dk.unk5.x0 != 0 && !ext::ftCo_80094EA4(gobj) {
            if ext::ftDk_MS_347_800E0378(gobj) {
                return;
            }
        }
    }
}

/// Re-enter the cargo turn (also the air-collision landing callback of the heavy states).
pub fn ftDk_MS_345_800E0294(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftDk_MS_345_800E0294");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let a = (*fp).x2CC;
        st::Fighter_ChangeMotionState(gobj, (*a).motion_state + 6, 1, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_SetAnimRate(gobj, 0.0);
        if (*fp).ground_or_air == GA_Ground {
            st::ftCommon_8007D5D4(&mut *fp);
        }
    }
}

/// `ftDk_MS_345_800E0294` as a Fighter callback entry (what `chars::cb` points at).
pub unsafe extern "C" fn cb_ftDk_MS_345_800E0294(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftDk_MS_345_800E0294");
    ftDk_MS_345_800E0294(g);
}
