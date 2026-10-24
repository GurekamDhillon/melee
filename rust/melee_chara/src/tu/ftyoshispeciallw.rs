//! src/melee/ft/kinds/ftYoshi/ftyoshispeciallw.c: Yoshi's Down B (Yoshi Bomb). Fused site
//! (tools/rust-port/fma_sites.py): `-1.0f * star_offset.x + transn.x` in `fn_8012E644`, written with
//! `melee_core::fp::fmadds`. State-oracle checked against the C (tests/oracle_ftyoshispeciallw.rs,
//! gen/oracle); the earlier rounds' thin callbacks are re-exported and checked through the legacy
//! `crate::stubs` hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::tu::ftinl;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/3/4): re-exported under this TU.
pub use crate::chars::r2::ftYoshi::ftyoshispeciallw::{
    ftYs_SpecialLwLanding_Anim, ftYs_SpecialLwLanding_Coll, ftYs_SpecialLwLanding_Phys, ftYs_SpecialS_8012EA04,
};
pub use crate::chars::r3::ftYoshi::ftyoshispeciallw::{ftYs_SpecialLw_Anim, ftYs_SpecialS_8012EAD8};
pub use crate::chars::r4::ftYoshi::ftyoshispeciallw::{ftYs_SpecialAirLw_Anim, ftYs_SpecialLw_Phys};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use crate::tu::hookmac::hook;
    use melee_types::*;
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    pub static mut IT_802B2FC8: Option<fn(*mut HSD_GObj, *mut Vec3, f32) -> *mut Item_GObj> = None;
    #[allow(non_snake_case)]
    pub fn it_802B2FC8(parent: *mut HSD_GObj, pos: *mut Vec3, dir: f32) -> *mut Item_GObj {
        match unsafe { IT_802B2FC8 } { Some(f) => f(parent, pos, dir), None => { return unsafe { melee_items::tu::ityoshistar::it_802B2FC8(parent as _, pos as _, dir as _) as _ }; } }
    }
    pub use crate::tu::ftinl::ext::{
        ftAnim_8006EBA4, ftCliffCommon_80081298, ftCliffCommon_80081370, ftCommon_8007D5D4, ft_80085134,
        ft_CheckGroundAndLedge, Fighter_ChangeMotionState,
    };
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `fn_8012E644` as a Fighter callback entry (accessory4_cb).
pub unsafe extern "C" fn cb_fn_8012E644(g: *mut HSD_GObj) {
    fn_8012E644(g);
}

/// Spawn the two Yoshi Bomb stars either side of Yoshi's TransN joint.
pub fn fn_8012E644(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftYs_DatAttrs);
        let mut transn_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut star_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC((*(*fp).parts.add(FtPart_TransN as usize)).joint, core::ptr::null_mut(), &mut transn_pos);
        star_pos.x = fmadds(-1.0, da.speciallw_star_offset.x, transn_pos.x);
        star_pos.y = transn_pos.y + da.speciallw_star_offset.y;
        star_pos.z = transn_pos.z;
        ext::it_802B2FC8(gobj, &mut star_pos, -1.0);
        star_pos.x = transn_pos.x + da.speciallw_star_offset.x;
        ext::it_802B2FC8(gobj, &mut star_pos, 1.0);
        (*fp).accessory4_cb = None;
    }
}

/// `ftYoshi_SpecialLw_SetVars(gobj)` (static inline).
pub fn ftYoshi_SpecialLw_SetVars(gobj: *mut HSD_GObj) {
    unsafe { set_vars(gobj) }
}

/// `reset(gobj)` (static inline): the same stores as `ftYoshi_SpecialLw_SetVars`, in another order.
pub fn reset(gobj: *mut HSD_GObj) {
    unsafe { set_vars(gobj) }
}

unsafe fn set_vars(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    (*fp).cmd_vars[1] = 0;
    (*fp).cmd_vars[0] = 0;
    (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
    (*fp).self_vel.y = 0.0;
    (*fp).self_vel.x = 0.0;
    (*fp).x6A4_transNOffset.y = 0.0;
    (*fp).set_x2223_b4(1);
}

pub fn ftYs_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftCommon_8007D5D4(fp);
        ext::Fighter_ChangeMotionState(gobj, ftYs_MS_SpecialLw as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(gobj);
        ftYoshi_SpecialLw_SetVars(gobj);
    }
}

pub fn ftYs_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        ext::Fighter_ChangeMotionState(gobj, ftYs_MS_SpecialAirLw as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(gobj);
        reset(gobj);
    }
}

pub fn ftYs_SpecialAirLw_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        ext::ft_80085134(gobj);
        if (*fp).self_vel.y < da.x114 || (*fp).cmd_vars[1] != 0 {
            (*fp).self_vel.y = da.x114;
        }
    }
}

pub fn ftYs_SpecialLw_Coll(gobj: *mut HSD_GObj) {
    ftYs_SpecialAirLw_Coll(gobj);
}

pub fn ftYs_SpecialAirLw_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if ext::ft_CheckGroundAndLedge(gobj, ftinl::ftGetFacingDirInt(fp)) {
            if (*fp).cmd_vars[0] != 0 && (*fp).self_vel.y <= 0.0 {
                ftYs_SpecialS_8012EAD8(gobj);
            } else {
                ext::ftCommon_8007D5D4(fp);
            }
        } else if (*fp).cmd_vars[0] != 0 && ext::ftCliffCommon_80081298(gobj) {
            ext::ftCliffCommon_80081370(gobj);
        }
    }
}
