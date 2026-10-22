//! src/melee/ft/kinds/ftDonkey/ftdonkeyspecials.c: Headbutt (side B).
//! No fused sites (tools/rust-port/fma_sites.py). `ftDk_SpecialAirS_Enter`'s momentum divide and
//! `ftDk_SpecialAirS_Phys` are checked against the C by tests/tu_ftdonkeyspecials.rs; the rest is
//! smoke-tested through recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2, 12): re-exported under this TU.
pub use crate::chars::r12::ftDonkey::ftdonkeyspecials::ftDk_SpecialS_Enter;
pub use crate::chars::r2::ftDonkey::ftdonkeyspecials::{
    ftDk_SpecialAirS_Anim, ftDk_SpecialS_Anim, ftDk_SpecialS_Phys,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    /// `ftDk_SpecialLw_8010E148` (ftdonkeyspeciallw.c) as a Fighter callback entry
    /// (`accessory4_cb`). TODO: not ported; unset it panics like the C-name stubs in `chars::cb`.
    pub static mut FTDK_SPECIALLW_8010E148: Option<fn(gobj: *mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftDk_SpecialLw_8010E148(gobj: *mut HSD_GObj) {
        match unsafe { FTDK_SPECIALLW_8010E148 } {
            Some(f) => f(gobj),
            None => panic!("TODO: ftDk_SpecialLw_8010E148 is not ported"),
        }
    }
    #[allow(non_snake_case)]
    pub unsafe extern "C" fn cb_ftDk_SpecialLw_8010E148(gobj: *mut HSD_GObj) {
        ftDk_SpecialLw_8010E148(gobj)
    }
}

/// Ft_MF_KeepGfx | SkipHit | SkipMatAnim | SkipColAnim | UpdateCmd | SkipItemVis | Unk19 |
/// SkipModelPartVis | SkipModelFlags | Unk27.
pub const coll_mf: u32 = (Ft_MF_KeepGfx
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftDk_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *mut ftDonkeyAttributes;
        (*fp).self_vel.x /= (*da).SpecialS.x3C_MIN_STICK_X_MOMENTUM;
        (*fp).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, ftDk_MS_SpecialAirS as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cmd_vars[0] = 0;
        (*fp).accessory4_cb = Some(ext::cb_ftDk_SpecialLw_8010E148);
    }
}

pub fn ftDk_SpecialS_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftDk_SpecialAirS_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftDk_SpecialAirS_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *mut ftDonkeyAttributes;
        if (*fp).cmd_vars[0] != 0 {
            st::ftCommon_Fall(&mut *fp, (*da).SpecialS.x44_AERIAL_GRAVITY, (*fp).co_attrs.terminal_velocity);
        }
        melee_fighter::ftcommon::calc_self_accel_deaccel(&mut *fp, (*da).SpecialS.x40_MOMENTUM_TRANSITION_MODIFIER);
    }
}

pub fn ftDk_SpecialS_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] != 0 {
            if st::ft_800827A0(gobj) == 0 {
                doAirTransition(gobj);
            }
        } else if st::ft_80082708(gobj) == 0 {
            doAirTransition(gobj);
        }
    }
}

pub fn ftDk_SpecialAirS_Coll(gobj: *mut HSD_GObj) {
    if st::ft_80081D0C(gobj) != 0 {
        doGroundTransition(gobj);
    }
}

/// `doAirTransition` (static, forward-declared then defined below the Coll functions).
pub fn doAirTransition(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftDk_MS_SpecialAirS as i32, coll_mf);
        if (*fp).x2219_b0() == 1 {
            set_effect_hitlag_callbacks(fp);
        }
    }
}

/// `doGroundTransition` (static).
pub fn doGroundTransition(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftDk_MS_SpecialS as i32, coll_mf);
        if (*fp).x2219_b0() == 1 {
            set_effect_hitlag_callbacks(fp);
        }
    }
}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}
