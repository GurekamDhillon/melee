//! src/melee/ft/kinds/ftKirby/ftkirbyattackdash.c: Kirby's dash attack (and the aerial
//! continuation). No fused sites (tools/rust-port/fma_sites.py), no pure math: the stateful
//! callbacks are smoke-tested by tests/tu_ftkirbyattackdash.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2, 3, 5): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyattackdash::{
    fn_800F1FDC, fn_800F20C4, ftKb_AttackDashAir_Anim, ftKb_AttackDash_Phys,
};
pub use crate::chars::r3::ftKirby::ftkirbyattackdash::ftKb_SpecialN_800F1F68;
pub use crate::chars::r5::ftKirby::ftkirbyattackdash::{ftKb_AttackDashAir_Coll, ftKb_AttackDash_Coll};

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftKb_AttackDash_Anim`: ftCommon's dash-attack animation callback.
pub fn ftKb_AttackDash_Anim(gobj: *mut HSD_GObj) {
    melee_fighter::tu::ftCo_AttackDash::ftCo_AttackDash_Anim(gobj);
}

/// `ftKb_AttackDash_IASA`: ftCommon's dash-attack interrupt check.
pub fn ftKb_AttackDash_IASA(gobj: *mut HSD_GObj) {
    melee_fighter::tu::ftCo_AttackDash::ftCo_AttackDash_IASA(gobj);
}

/// `ftKb_AttackDashAir_IASA`: empty in the C.
pub fn ftKb_AttackDashAir_IASA(_gobj: *mut HSD_GObj) {}

/// `ftKb_AttackDashAir_Phys`: `ft_80085204(gobj); ftCommon_FallBasic(fp);`.
pub fn ftKb_AttackDashAir_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ft_80085204(gobj);
        melee_fighter::ftcommon::fall_basic(&mut *fp);
    }
}

/// `ftKb_AttackDashAir_800F21C0`: rotation X of part 0.
pub fn ftKb_AttackDashAir_800F21C0(gobj: *mut HSD_GObj) -> f32 {
    unsafe { melee_fighter::tu::ftparts::ftPartGetRotX(&*fpp(gobj), 0) }
}
