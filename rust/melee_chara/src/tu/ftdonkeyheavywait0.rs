//! src/melee/ft/kinds/ftDonkey/ftdonkeyheavywait0.c: DK carrying a heavy item, wait.
//! No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftdonkeyheavywait0.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use super::ftdonkeyheavyturn::ext as turn_ext;
use crate::stubs as st;
use melee_types::*;

pub use crate::chars::r2::ftDonkey::ftdonkeyheavywait0::ftDk_HeavyWait0_Phys;
pub use crate::chars::r5::ftDonkey::ftdonkeyheavywait0::ftDk_HeavyWait0_Coll;

pub mod ext {
    use melee_types::*;
    type G = *mut HSD_GObj;
    pub static mut FT_8008A1FC: Option<fn(G) -> bool> = None;
    pub fn ft_8008A1FC(g: G) -> bool {
        match unsafe { FT_8008A1FC } {
            Some(f) => f(g),
            None => melee_fighter::ft084e::ft_8008a1fc(g),
        }
    }
}

pub fn ftDk_MS_341_800DF938(gobj: *mut HSD_GObj) -> bool {
    if ext::ft_8008A1FC(gobj) {
        ftDk_MS_341_800DF980(gobj);
        true
    } else {
        false
    }
}

/// Enter the cargo wait state.
pub fn ftDk_MS_341_800DF980(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).ground_or_air == GA_Air {
            st::ftCommon_8007D7FC(&mut *fp);
        }
        let a = (*fp).x2CC;
        st::Fighter_ChangeMotionState(gobj, (*a).motion_state, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftDk_HeavyWait0_IASA(gobj: *mut HSD_GObj) {
    if !turn_ext::ftCo_80094EA4(gobj)
        && !turn_ext::ftDk_MS_347_800E0378(gobj)
        && !super::ftdonkeyms3450::ftDk_MS_345_800E0134(gobj)
    {
        if super::ftdonkeyheavywalk::ftDk_MS_341_800DFA98(gobj) {
            return;
        }
    }
}
