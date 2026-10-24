//! src/melee/ft/kinds/ftPeach/ftpeachspecialhi.c: Peach's up B (Parasol), ground and air, plus
//! the parasol item bookkeeping. No fused sites (tools/rust-port/fma_sites.py: 0 in every
//! function). Smoke-tested through recording hooks by tests/tu_ftpeachspecialhi.rs (the stick
//! lean math of `ftPe_SpecialHiStart_IASA` is checked against the C cut verbatim there).
//!
//! `ftPe_SpecialHiStart_IASA`: `deg = x80 * ((|lx| - x7C) / (1.0 - x7C))` is double arithmetic
//! (`1.0` is a double literal) rounded once to f32; `MTXDegToRad(a)` is `a * 0.01745329252f`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftpeachfloat::{cb_ftCo_80096CC8, ext as fext};
use melee_types::*;

/// Already ported (round 7 and the chars module): re-exported under this TU.
pub use crate::chars::ftPeach::ftpeachspecialhi::checkCmdVar2;
pub use crate::chars::r7::ftPeach::ftpeachspecialhi::{ftPe_SpecialAirHiEnd_Anim, ftPe_SpecialHiEnd_Anim};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *const Vec3, pos1: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, pos0, pos1),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; },
                }
            }
    pub static mut IT_8026B300: Option<fn(*mut Item_GObj) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn itGetKind(item: *mut Item_GObj) -> i32 {
                match unsafe { IT_8026B300 } {
                    Some(f) => f(item),
                    None => { return unsafe { melee_items::tu::it_26B1::itGetKind(item as _) as _ }; },
                }
            }
    pub static mut IT_8026BB44: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026BB44(item: *mut Item_GObj)  {
                match unsafe { IT_8026BB44 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026BB44(item as _); } return; },
                }
            }
    pub static mut IT_8026B724: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B724(item: *mut Item_GObj)  {
                match unsafe { IT_8026B724 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026B724(item as _); } return; },
                }
            }
    pub static mut IT_8026BB20: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026BB20(item: *mut Item_GObj)  {
                match unsafe { IT_8026BB20 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026BB20(item as _); } return; },
                }
            }
    pub static mut IT_8026B73C: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B73C(item: *mut Item_GObj)  {
                match unsafe { IT_8026B73C } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026B73C(item as _); } return; },
                }
            }
    pub static mut FTCOMMON_8007E6DC: Option<fn(*mut HSD_GObj, *mut HSD_GObj, i32) > = None;
            #[allow(non_snake_case)]
            pub fn ftCommon_8007E6DC(gobj: *mut HSD_GObj, item: *mut HSD_GObj, flag: i32)  {
                match unsafe { FTCOMMON_8007E6DC } {
                    Some(f) => f(gobj, item, flag),
                    None => { unsafe { let _ = melee_fighter::ftcommon3::ftcommon_8007e6dc(gobj as _, item as _, flag as _); } return; },
                }
            }
    pub static mut IT_802BDA64: Option<fn(*mut HSD_GObj, *mut Vec3, u32, f32) -> *mut Item_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802BDA64(gobj: *mut HSD_GObj, pos: *mut Vec3, part: u32, dir: f32) -> *mut Item_GObj {
                match unsafe { IT_802BDA64 } {
                    Some(f) => f(gobj, pos, part, dir),
                    None => { return unsafe { melee_items::tu::itpeachparasol::it_802BDA64(gobj as _, pos as _, part as _, dir as _) as _ }; },
                }
            }
    pub static mut FTPICKUPITEM_80094818: Option<fn(*mut HSD_GObj, bool) > = None;
            #[allow(non_snake_case)]
            pub fn ftpickupitem_80094818(gobj: *mut HSD_GObj, flag: bool)  {
                match unsafe { FTPICKUPITEM_80094818 } {
                    Some(f) => f(gobj, flag),
                    None => { unsafe { let _ = melee_fighter::tu::ftpickupitem::ftpickupitem_80094818(gobj as _, flag as _); } return; },
                }
            }
    pub static mut IT_802BDB94: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BDB94(item: *mut Item_GObj)  {
                match unsafe { IT_802BDB94 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itpeachparasol::it_802BDB94(item as _); } return; },
                }
            }
    pub static mut IT_802BDBF8: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BDBF8(item: *mut Item_GObj)  {
                match unsafe { IT_802BDBF8 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itpeachparasol::it_802BDBF8(item as _); } return; },
                }
            }
    pub static mut IT_802BDC18: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BDC18(item: *mut Item_GObj)  {
                match unsafe { IT_802BDC18 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itpeachparasol::it_802BDC18(item as _); } return; },
                }
            }
    pub static mut IT_802BDC38: Option<fn(*mut Item_GObj) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn it_802BDC38(item: *mut Item_GObj) -> bool {
                match unsafe { IT_802BDC38 } {
                    Some(f) => f(item),
                    None => { return unsafe { melee_items::tu::itpeachparasol::it_802BDC38(item as _) as _ }; },
                }
            }
    pub static mut FTCO_800CEFE0: Option<fn(*mut HSD_GObj, i32) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_800CEFE0(gobj: *mut HSD_GObj, arg1: i32)  {
                match unsafe { FTCO_800CEFE0 } {
                    Some(f) => f(gobj, arg1),
                    None => { unsafe { let _ = melee_fighter::tu::ftCo_ItemParasolOpen::ftCo_800CEFE0(gobj as _, arg1 as _); } return; },
                }
            }
    pub static mut FT_80083B68: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ft_80083B68(gobj: *mut HSD_GObj)  {
                match unsafe { FT_80083B68 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = melee_fighter::tu::ft_081b::ft_80083b68(gobj as _); } return; },
                }
            }
    pub use crate::tu::ftcaptainspecialhi::ext::{ftPartSetRotY, FTPARTSETROTY};
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// C `ABS` (`x < 0 ? -x : x`): `ABS(-0.0)` stays `-0.0`.
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `MTXDegToRad(a)`.
#[inline(always)]
fn MTXDegToRad(a: f32) -> f32 {
    a * 0.01745329252f32
}

/// `ftCheckThrowB3(fp)` (ft/inlines.h): test and clear `throw_flags_b3`.
unsafe fn ftCheckThrowB3(fp: *mut Fighter) -> bool {
    if (*fp).anon_2210.anon_0.throw_flags_b3() != 0 {
        (*fp).anon_2210.anon_0.set_throw_flags_b3(0);
        true
    } else {
        false
    }
}

/// `ftPe_SpecialHi_8011D620` as an `HSD_GObjEvent` (pre-hitlag callback).
pub unsafe extern "C" fn cb_ftPe_SpecialHi_8011D620(g: *mut HSD_GObj) {
    ftPe_SpecialHi_8011D620(g);
}
/// `ftPe_SpecialHi_8011D650` as an `HSD_GObjEvent` (post-hitlag callback).
pub unsafe extern "C" fn cb_ftPe_SpecialHi_8011D650(g: *mut HSD_GObj) {
    ftPe_SpecialHi_8011D650(g);
}
/// `ftPe_SpecialHi_8011D424` as the Accessory4 callback entry.
pub unsafe extern "C" fn cb_ftPe_SpecialHi_8011D424(g: *mut HSD_GObj) {
    ftPe_SpecialHi_8011D424(g);
}
/// `ftPe_SpecialHi_8011DD8C` as an `HSD_GObjEvent` (landing callback).
pub unsafe extern "C" fn cb_ftPe_SpecialHi_8011DD8C(g: *mut HSD_GObj) {
    ftPe_SpecialHi_8011DD8C(g);
}
/// `ftPe_SpecialHi_8011E064` as an `HSD_GObjEvent` (landing callback).
pub unsafe extern "C" fn cb_ftPe_SpecialHi_8011E064(g: *mut HSD_GObj) {
    ftPe_SpecialHi_8011E064(g);
}

/// Accessory4 callback (subaction event): the capsule becomes the parasol.
pub fn ftPe_SpecialHi_8011D424(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.pe.specialhi.kind == It_Kind_Capsule {
            (*fp).mv.pe.specialhi.kind = It_Kind_Peach_Parasol;
            if (*fp).u.pe.parasol_gobj_0.is_null() {
                let mut pos: Vec3 = core::mem::zeroed();
                ext::lb_8000B1CC((*(*fp).parts.add(FtPart_109 as usize)).joint, core::ptr::null(), &mut pos);
                {
                    let igobj = (*fp).item_gobj;
                    if !igobj.is_null() {
                        (*fp).u.pe.parasol_gobj_1 = igobj;
                        ext::it_8026BB44((*fp).item_gobj);
                        ext::it_8026B724((*fp).item_gobj);
                        ext::ftCommon_8007E6DC(gobj, (*fp).item_gobj, 1);
                    }
                }
                (*fp).u.pe.parasol_gobj_0 = ext::it_802BDA64(gobj, &mut pos, FtPart_109, (*fp).facing_dir);
                (*fp).item_gobj = (*fp).u.pe.parasol_gobj_0;
                if !(*fp).u.pe.parasol_gobj_0.is_null() {
                    (*fp).death3_cb = Some(crate::tu::ftpeach::cb_ftPe_Init_OnDeath2);
                    (*fp).take_dmg_cb = Some(crate::tu::ftpeach::cb_ftPe_Init_OnDeath2);
                }
            }
        }
        (*fp).accessory4_cb = None;
        (*fp).pre_hitlag_cb = Some(cb_ftPe_SpecialHi_8011D620);
        (*fp).post_hitlag_cb = Some(cb_ftPe_SpecialHi_8011D650);
    }
}

/// Give the held parasol item back: returns true if a stashed item was restored.
pub fn ftPe_8011D518(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.pe.parasol_gobj_0 = core::ptr::null_mut();
        (*fp).death3_cb = None;
        (*fp).take_dmg_cb = None;
        let parasol_gobj = (*fp).u.pe.parasol_gobj_1;
        if !parasol_gobj.is_null() {
            (*fp).item_gobj = parasol_gobj;
            (*fp).u.pe.parasol_gobj_1 = core::ptr::null_mut();
            ext::it_8026BB20((*fp).item_gobj);
            ext::it_8026B73C((*fp).item_gobj);
            ext::ftpickupitem_80094818(gobj, true);
            return true;
        }
        false
    }
}

pub fn ftPe_8011D598(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.pe.parasol_gobj_0.is_null() {
            ext::it_802BDB94((*fp).u.pe.parasol_gobj_0);
            ftPe_8011D518(gobj);
        }
    }
}

/// `ensureUnkItem` (static): run `cb` on the parasol item if there is one.
fn ensureUnkItem(gobj: *mut HSD_GObj, cb: fn(*mut Item_GObj)) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.pe.parasol_gobj_0.is_null() {
            cb((*fp).u.pe.parasol_gobj_0);
        }
    }
}

pub fn ftPe_SpecialHi_8011D620(gobj: *mut HSD_GObj) {
    ensureUnkItem(gobj, ext::it_802BDBF8);
}

pub fn ftPe_SpecialHi_8011D650(gobj: *mut HSD_GObj) {
    ensureUnkItem(gobj, ext::it_802BDC18);
}

/// `notUsingParasol` (static).
fn notUsingParasol(fp: *mut Fighter) -> bool {
    unsafe {
        let msid = (*fp).motion_id as i32;
        if msid >= ftPe_MS_ItemParasolOpen as i32 && msid <= ftPe_MS_ItemParasolFall as i32 {
            return false;
        }
        if msid >= ftCo_MS_ItemParasolOpen as i32 && msid <= ftCo_MS_ItemParasolFallSpecial as i32 {
            return false;
        }
        if msid >= ftCo_MS_FallSpecial as i32 && msid <= ftCo_MS_FallSpecialB as i32 {
            return false;
        }
        if msid == ftCo_MS_LandingFallSpecial as i32 && !ext::it_802BDC38((*fp).item_gobj) {
            return false;
        }
        true
    }
}

pub fn ftPe_SpecialHi_NotActive(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        let msid = (*fp).motion_id as i32;
        if msid >= ftPe_MS_SpecialHiStart as i32 && msid <= ftPe_MS_SpecialAirHiEnd as i32 {
            return false;
        }
        notUsingParasol(fp)
    }
}

/// `doEnter` (static).
fn doEnter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).cmd_vars[0] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[2] = 0;
        (*fp).anon_2210.throw_flags = 0;
        (*fp).mv.pe.specialhi.kind = It_Kind_Capsule;
        (*fp).set_x2222_b2(1);
        let igobj = (*fp).item_gobj;
        if !igobj.is_null() {
            if ext::itGetKind(igobj) == It_Kind_Parasol {
                (*fp).mv.pe.specialhi.kind = It_Kind_Parasol;
                return;
            }
            if ext::itGetKind((*fp).item_gobj) == It_Kind_Peach_Parasol {
                (*fp).mv.pe.specialhi.kind = It_Kind_Peach_Parasol;
                return;
            }
        }
        (*fp).accessory4_cb = Some(cb_ftPe_SpecialHi_8011D424);
    }
}

pub fn ftPe_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    st::Fighter_ChangeMotionState(gobj, ftPe_MS_SpecialHiStart as i32, FtMoveId_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
    doEnter(gobj);
}

pub fn ftPe_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftPe_DatAttrs;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x *= (*da).x84;
        st::Fighter_ChangeMotionState(gobj, ftPe_MS_SpecialAirHiStart as i32, FtMoveId_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        doEnter(gobj);
    }
}

pub fn ftPe_SpecialHiStart_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !checkCmdVar2(&mut *fp) && st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let da = (*fp).dat_attrs as *const ftPe_DatAttrs;
            if (*fp).mv.pe.specialhi.kind == It_Kind_Parasol {
                (*fp).motion_id = ftCo_MS_FallSpecial as _;
                ext::ftCo_800CEFE0(gobj, melee_fighter::hooks::common_data().x59C as i32);
            } else {
                (*fp).motion_id = ftCo_MS_FallSpecial as _;
                ext::ftCo_800CEFE0(gobj, (*da).x90);
            }
        }
    }
}

pub fn ftPe_SpecialAirHiStart_Anim(gobj: *mut HSD_GObj) {
    ftPe_SpecialHiStart_Anim(gobj);
}

pub fn ftPe_SpecialHiStart_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftPe_DatAttrs;
        let abs_lstick_x = abs_c((*fp).input.lstick[0].x);
        if (*fp).cmd_vars[0] == 0 && abs_lstick_x > (*da).x7C {
            let deg = ((*da).x80 as f64 * ((abs_lstick_x - (*da).x7C) as f64 / (1.0f64 - (*da).x7C as f64))) as f32;
            let rad = if (*fp).input.lstick[0].x > 0.0 { -MTXDegToRad(deg) } else { MTXDegToRad(deg) };
            let abs_lstick_angle = abs_c((*fp).lstick_angle);
            if abs_c(rad) > abs_lstick_angle {
                (*fp).lstick_angle = rad;
            }
        }
        if ftCheckThrowB3(fp) && abs_c((*fp).input.lstick[0].x) > (*da).x78 {
            st::ftCommon_UpdateFacing(&mut *fp);
            ext::ftPartSetRotY(fp, 0, (core::f64::consts::FRAC_PI_2 * (*fp).facing_dir as f64) as f32);
        }
    }
}

pub fn ftPe_SpecialAirHiStart_IASA(gobj: *mut HSD_GObj) {
    ftPe_SpecialHiStart_IASA(gobj);
}

pub fn ftPe_SpecialHiStart_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        if (*fpp(gobj)).ground_or_air == GA_Air {
            st::ft_80085154(gobj);
        } else {
            st::ft_80084FA8(gobj);
        }
    }
}

pub fn ftPe_SpecialAirHiStart_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftPe_DatAttrs;
        if (*fp).cmd_vars[0] != 0 {
            st::ft_80085154(gobj);
            (*fp).self_vel.x = (*fp).self_vel.x * (*da).x8C;
            (*fp).self_vel.y = (*fp).self_vel.y * (*da).x8C;
            (*fp).self_vel.z = (*fp).self_vel.z * (*da).x8C;
        } else {
            st::ftCommon_Fall(&mut *fp, (*da).x88, (*fp).co_attrs.terminal_velocity);
            melee_fighter::ftcommon::calc_self_accel_deaccel_quick_air(&mut *fp);
        }
    }
}

/// `ftPe_SpecialHi_8011DD8C` (static): landing after the start phase.
pub fn ftPe_SpecialHi_8011DD8C(gobj: *mut HSD_GObj) {
    unsafe {
        let da = (*fpp(gobj)).dat_attrs as *const ftPe_DatAttrs;
        st::ftCo_LandingFallSpecial_Enter(gobj, 0, (*da).x74);
    }
}

/// `doColl` (static).
fn doColl(gobj: *mut HSD_GObj, cb: HSD_GObjEvent) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Air {
            if (*fp).cmd_vars[0] == 0 || (*fp).self_vel.y >= 0.0 {
                ext::ft_80083B68(gobj);
                return;
            }
            fext::ft_800831CC(gobj, Some(cb_ftCo_80096CC8), cb);
            return;
        }
        st::ft_80084104(gobj);
    }
}

pub fn ftPe_SpecialHiStart_Coll(gobj: *mut HSD_GObj) {
    doColl(gobj, Some(cb_ftPe_SpecialHi_8011DD8C));
}

pub fn ftPe_SpecialAirHiStart_Coll(gobj: *mut HSD_GObj) {
    doColl(gobj, Some(cb_ftPe_SpecialHi_8011DD8C));
}

pub fn ftPe_SpecialHiEnd_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftPe_SpecialAirHiEnd_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftPe_SpecialHiEnd_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Air {
            let vel_y = (*fp).self_vel.y;
            st::ft_80085154(gobj);
            (*fp).self_vel.y = vel_y;
            st::ftCommon_Fall(&mut *fp, (*fp).co_attrs.gravity, (*fp).co_attrs.terminal_velocity);
        } else {
            st::ft_80084FA8(gobj);
        }
    }
}

pub fn ftPe_SpecialAirHiEnd_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftPe_DatAttrs;
        let vel_y = (*fp).self_vel.y;
        st::ft_80085154(gobj);
        (*fp).self_vel.x *= (*da).x8C;
        (*fp).self_vel.y *= (*da).x8C;
        (*fp).self_vel.z *= (*da).x8C;
        (*fp).self_vel.y = vel_y;
        st::ftCommon_Fall(&mut *fp, (*fp).co_attrs.gravity, (*fp).co_attrs.terminal_velocity);
    }
}

pub fn ftPe_SpecialHi_8011E064(gobj: *mut HSD_GObj) {
    ftPe_SpecialHi_8011DD8C(gobj);
}

pub fn ftPe_SpecialHiEnd_Coll(gobj: *mut HSD_GObj) {
    doColl(gobj, Some(cb_ftPe_SpecialHi_8011E064));
}

pub fn ftPe_SpecialAirHiEnd_Coll(gobj: *mut HSD_GObj) {
    doColl(gobj, Some(cb_ftPe_SpecialHi_8011E064));
}
