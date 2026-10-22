//! src/melee/ft/kinds/ftSeak/ftseakspeciallw.c: Sheik's Transform (down B): the four start
//! states, the transform-finish states, and their accessory callbacks. No fused sites
//! (tools/rust-port/fma_sites.py: 0 fmuladd in every function); the `/=` scalings are plain
//! `f32` divides, and `M_PI / 3` is a `double` narrowed to the `float` parameter. Not yet tested: no C oracle and no smoke test (unverified).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/3/4 and the hand module): re-exported under this TU. The hand
/// module's Phys functions take the fighter (`&mut Fighter`) where the C takes the gobj.
pub use crate::chars::ftSeak::ftseakspeciallw::{ftSk_SpecialAirLw2_Phys, ftSk_SpecialAirLw_Phys};
pub use crate::chars::r2::ftSeak::ftseakspeciallw::{ftSk_SpecialLw2_Anim, ftSk_SpecialAirLw2_Anim, ftSk_SpecialLw2_Phys, ftSk_SpecialLw_Phys};
pub use crate::chars::r3::ftSeak::ftseakspeciallw::{
    ftSk_SpecialAirLw2_Coll, ftSk_SpecialAirLw_Coll, ftSk_SpecialLw2_Coll, ftSk_SpecialLw_801144B8,
    ftSk_SpecialLw_80114680, ftSk_SpecialLw_801146EC, ftSk_SpecialLw_8011444C, ftSk_SpecialLw_Coll,
};
pub use crate::chars::r4::ftSeak::ftseakspeciallw::{ftSk_SpecialAirLw_Anim, ftSk_SpecialLw_Anim};

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
    hook!(EFSYNC_SPAWN, efSync_Spawn, (id: i32, gobj: *mut HSD_GObj, jobj: *mut HSD_JObj));
    hook!(LB_8000B1CC, lb_8000B1CC, (joint: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3));
    hook!(LB_800119DC, lb_800119DC, (v: *mut Vec3, a: i32, b: f32, c: f32, d: f32));
    hook!(FTCOMMON_8007EFC8, ftCommon_8007EFC8, (gobj: *mut HSD_GObj, cb: HSD_GObjEvent));
    /// `ftZd_SpecialLw_8013B4D8` (ftZelda/ftzeldaspeciallw.c), passed as a callback.
    hook!(FTZD_SPECIALLW_8013B4D8, ftZd_SpecialLw_8013B4D8, (gobj: *mut HSD_GObj));
}

pub unsafe extern "C" fn cb_ftZd_SpecialLw_8013B4D8(g: *mut HSD_GObj) {
    ext::ftZd_SpecialLw_8013B4D8(g);
}

/// `ftSk_MF_SpecialLw_Coll`: `ftCommon_GroundAirColl_MF | KeepGfx | KeepColAnimHitStatus | SkipHit`.
pub const ftSk_MF_SpecialLw_Coll: u32 = (Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27)
    | Ft_MF_KeepGfx
    | Ft_MF_KeepColAnimHitStatus
    | Ft_MF_SkipHit;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftSeakAttributes {
    (*fp).dat_attrs as *const ftSeakAttributes
}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

pub unsafe extern "C" fn cb_fn_80114034(g: *mut HSD_GObj) {
    fn_80114034(g);
}
pub unsafe extern "C" fn cb_fn_801140B0(g: *mut HSD_GObj) {
    fn_801140B0(g);
}
pub unsafe extern "C" fn cb_fn_8011412C(g: *mut HSD_GObj) {
    fn_8011412C(g);
}

/// Ground transform start effect (accessory callback).
pub fn fn_80114034(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            ext::efSync_Spawn(0x4FC, gobj, (*(*fp).parts.add(FtPart_R2ndNb as usize)).joint);
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

/// Transform-finish effect (accessory callback).
pub fn fn_801140B0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            ext::efSync_Spawn(0x4FD, gobj, (*(*fp).parts.add(FtPart_HipN as usize)).joint);
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

/// Swap to Zelda (accessory callback at the end of the transform start).
pub fn fn_8011412C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).accessory4_cb = None;
        ext::ftCommon_8007EFC8(gobj, Some(cb_ftZd_SpecialLw_8013B4D8));
    }
}

/// `ftSk_unk_inline` (static): `lb_800119DC(v, 0x78, 0.4F, 0.003F, M_PI / 3)`.
fn ftSk_unk_inline(v: *mut Vec3) {
    ext::lb_800119DC(v, 0x78, 0.4_f32, 0.003_f32, (core::f64::consts::PI / 3.0) as f32);
}

/// The shared body of both Enter functions after the motion change.
unsafe fn enter_tail(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let attributes = da(fp);
    let mut sp20 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    (*fp).cmd_vars[0] = 0;
    (*fp).self_vel.x /= (*attributes).x60;
    (*fp).self_vel.y /= (*attributes).x64;
    (*fp).gr_vel /= (*attributes).x60;
    ext::lb_8000B1CC((*(*fp).parts.add(FtPart_TopN as usize)).joint, core::ptr::null(), &mut sp20);
    ftSk_unk_inline(&mut sp20);
    (*fp).accessory4_cb = Some(cb_fn_80114034);
}

/// Sheik_AS_361_Transform_Grounded
pub fn ftSk_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    st::Fighter_ChangeMotionState(gobj, 0x169, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
    unsafe { enter_tail(gobj) }
}

/// Sheik_AS_363_Transform_Aerial
pub fn ftSk_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    st::Fighter_ChangeMotionState(gobj, 0x16B, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
    unsafe { enter_tail(gobj) }
}

pub fn ftSk_SpecialLw_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftSk_SpecialAirLw_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftSk_SpecialLw2_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftSk_SpecialAirLw2_IASA(_gobj: *mut HSD_GObj) {}

/// AS_SheikFinishTransformation
pub fn ftSk_SpecialLw_80114758(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = da(fp);

        let var_r4 = if (*fp).ground_or_air == GA_Ground { 0x16A } else { 0x16C };
        st::Fighter_ChangeMotionState(gobj, var_r4, 0, (*attributes).x70, 1.0, 0.0, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(cb_fn_801140B0);
    }
}
