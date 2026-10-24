//! src/melee/ft/kinds/ftKirby/ftkirbyspecialzelda.c: Kirby's Zelda copy ability (Nayru's Love).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested
//! through recording hooks by tests/tu_ftkirbyspecialzelda.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftkirbyspecialseak::cb_fn_80105FEC;
use melee_fighter::ftcommon as fc;
use melee_types::*;

/// Already ported (round 6): re-exported under this TU.
pub use crate::chars::r6::ftKirby::ftkirbyspecialzelda::ftKb_ZdSpecialN_Phys;

/// `ftZd_MF_SpecialN_Coll` (ftZelda/forward.h): `ftCommon_GroundAirColl_MF | KeepGfx |
/// KeepColAnimHitStatus | SkipHit`.
pub const ftZd_MF_SpecialN_Coll: u32 = 0x0C4C5080 | Ft_MF_KeepGfx as u32 | Ft_MF_KeepColAnimHitStatus as u32 | Ft_MF_SkipHit as u32;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing pointers as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    crate::port_hook!(FTCOLL_CREATEREFLECTHIT, ftColl_CreateReflectHit, crate::adapt::ftColl_CreateReflectHit, (gobj: *mut HSD_GObj, desc: *mut ReflectDesc, cb: HSD_GObjEvent));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftKb_DatAttrs {
    (*fp).dat_attrs as *mut ftKb_DatAttrs
}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// `ftKb_SpecialNZd_Helper` (static inline): clear the reflect state, load the gravity delay.
pub fn ftKb_SpecialNZd_Helper(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = da(fp);
        (*fp).cmd_vars[0] = 0;
        (*fp).mv.zd.specialn.x0 = (*attributes).specialn_zd_frames_before_gravity as _;
    }
}

/// `fn_80105A34` (static): ground bubble effect on the Kirby root joint, spawned once.
pub fn fn_80105A34(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            let bone = (*fp).parts;
            ext::efSync_Spawn(0x4B6, gobj, &[(*bone.add(1)).joint as *mut _]);
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

/// `fn_80105AB0` (static): the aerial variant, spawned once.
pub fn fn_80105AB0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            ext::efSync_Spawn(0x4B7, gobj, &[(*(*fp).parts.add(1)).joint as *mut _]);
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

/// `fn_80105A34` as a Fighter callback entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_fn_80105A34(g: *mut HSD_GObj) {
    fn_80105A34(g);
}

/// `fn_80105AB0` as a Fighter callback entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_fn_80105AB0(g: *mut HSD_GObj) {
    fn_80105AB0(g);
}

pub fn ftKb_SpecialNZd_80105B2C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_ZdSpecialN as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftKb_SpecialNZd_Helper(gobj);
        (*fp).accessory4_cb = Some(cb_fn_80105A34);
    }
}

pub fn ftKb_SpecialNZd_80105BA8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = (*fp).self_vel.x / (*da).specialn_zd_horizontal_momentum_preservation;
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_ZdSpecialAirN as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftKb_SpecialNZd_Helper(gobj);
        (*fp).accessory4_cb = Some(cb_fn_80105AB0);
    }
}

/// The reflect-window part both Anim callbacks share.
unsafe fn reflect_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let da = da(fp);
    if (*fp).cmd_vars[0] == 1 {
        (*fp).cmd_vars[0] = 2;
        ext::ftColl_CreateReflectHit(gobj, &mut (*da).specialn_zd_reflectdesc, Some(cb_fn_80105FEC));
    }
    if (*fp).cmd_vars[0] == 0 {
        (*fp).set_reflecting(0);
    }
}

pub fn ftKb_ZdSpecialN_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        reflect_anim(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_ZdSpecialAirN_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        reflect_anim(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftKb_ZdSpecialN_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftKb_ZdSpecialAirN_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftKb_ZdSpecialAirN_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let var1 = (*fp).mv.zd.specialn.x0;
        let da = da(fp);
        if var1 != 0 {
            (*fp).mv.zd.specialn.x0 = var1 - 1;
        } else {
            st::ftCommon_Fall(&mut *fp, (*da).specialn_zd_fall_acceleration, (*fp).co_attrs.terminal_velocity);
        }
        fc::calc_self_accel_deaccel_quick_air(&mut *fp);
        st::ftColl_8007AEF8(gobj);
    }
}

pub fn ftKb_ZdSpecialN_Coll(gobj: *mut HSD_GObj) {
    if st::ft_80082708(gobj) == GA_Ground as i32 {
        ftKb_SpecialNSk_80105E8C(gobj);
    }
}

pub fn ftKb_ZdSpecialAirN_Coll(gobj: *mut HSD_GObj) {
    if st::ft_80081D0C(gobj) != GA_Ground as i32 {
        ftKb_SpecialNSk_80105F3C(gobj);
    }
}

/// Restore the effect callbacks and the reflect bubble after a ground/air swap.
unsafe fn after_swap(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let da = da(fp);
    if (*fp).x2219_b0() == 1 {
        set_effect_hitlag_callbacks(fp);
    }
    if (*fp).cmd_vars[0] == 2 {
        ext::ftColl_CreateReflectHit(gobj, &mut (*da).specialn_zd_reflectdesc, Some(cb_fn_80105FEC));
    }
}

pub fn ftKb_SpecialNSk_80105E8C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftKb_MS_ZdSpecialAirN as i32, ftZd_MF_SpecialN_Coll);
        after_swap(gobj);
    }
}

pub fn ftKb_SpecialNSk_80105F3C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftKb_MS_ZdSpecialN as i32, ftZd_MF_SpecialN_Coll);
        after_swap(gobj);
    }
}
