//! src/melee/ft/kinds/ftKirby/ftkirbyattackdash.c: Kirby's dash attack (and the aerial
//! continuation). No fused sites (tools/rust-port/fma_sites.py), no pure math: the stateful
//! callbacks are smoke-tested by tests/tu_ftkirbyattackdash.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2, 3, 5): re-exported under this TU.




#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftKb_AttackDash_Anim`: ftCommon's dash-attack animation callback.
pub fn ftKb_AttackDash_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_AttackDash_Anim");
    melee_fighter::tu::ftCo_AttackDash::ftCo_AttackDash_Anim(gobj);
}

/// `ftKb_AttackDash_IASA`: ftCommon's dash-attack interrupt check.
pub fn ftKb_AttackDash_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_AttackDash_IASA");
    melee_fighter::tu::ftCo_AttackDash::ftCo_AttackDash_IASA(gobj);
}

/// `ftKb_AttackDashAir_IASA`: empty in the C.
pub fn ftKb_AttackDashAir_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKb_AttackDashAir_IASA");}

/// `ftKb_AttackDashAir_Phys`: `ft_80085204(gobj); ftCommon_FallBasic(fp);`.
pub fn ftKb_AttackDashAir_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_AttackDashAir_Phys");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80085204(gobj);
        melee_fighter::ftcommon::fall_basic(&mut *fp);
    }
}

/// `ftKb_AttackDashAir_800F21C0`: rotation X of part 0.
pub fn ftKb_AttackDashAir_800F21C0(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftKb_AttackDashAir_800F21C0");
    unsafe { melee_fighter::tu::ftparts::ftPartGetRotX(&*fpp(gobj), 0) }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `fn_800F1FDC` (src/melee/ft/kinds/ftKirby/ftkirbyattackdash.c).
pub fn fn_800F1FDC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F1FDC");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_8007D7FC(fp);
    st::Fighter_ChangeMotionState(gobj, 351 /* ftKb_MS_AttackDash */, 0xC4C5082u32 /* 0x0C4C5082U */, fp.cur_anim_frame, fp.frame_speed_mul, 0.0_f32, core::ptr::null_mut());
}

/// `ftKb_AttackDash_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyattackdash.c).
pub fn ftKb_AttackDash_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_AttackDash_Phys");
    st::ft_80084FA8(gobj);
}

/// `fn_800F20C4` (src/melee/ft/kinds/ftKirby/ftkirbyattackdash.c).
pub fn fn_800F20C4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F20C4");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_8007D5D4(fp);
    st::Fighter_ChangeMotionState(gobj, 352 /* ftKb_MS_AttackDashAir */, 0xC4C5082u32 /* 0x0C4C5082 */, fp.cur_anim_frame, fp.frame_speed_mul, 0.0_f32, core::ptr::null_mut());
}

/// `ftKb_AttackDashAir_Anim` (src/melee/ft/kinds/ftKirby/ftkirbyattackdash.c).
pub fn ftKb_AttackDashAir_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_AttackDashAir_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftKb_SpecialN_800F1F68` (src/melee/ft/kinds/ftKirby/ftkirbyattackdash.c).
pub fn ftKb_SpecialN_800F1F68(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialN_800F1F68");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).set_allow_interrupt(0);
        st::Fighter_ChangeMotionState(gobj, 351 /* ftKb_MS_AttackDash */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.co.attackdash.x0 = 0_i32;
    }
}

/// `ftKb_AttackDash_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyattackdash.c).
pub fn ftKb_AttackDash_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_AttackDash_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyattackdash__fn_800F20C4));
    }
}

/// `ftKb_AttackDashAir_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyattackdash.c).
pub fn ftKb_AttackDashAir_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_AttackDashAir_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyattackdash__fn_800F1FDC));
    }
}
