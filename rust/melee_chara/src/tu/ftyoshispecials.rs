//! src/melee/ft/kinds/ftYoshi/ftyoshispecials.c: Yoshi's side B (Egg Roll). Fused sites
//! (tools/rust-port/fma_sites.py), all written out:
//! * f32 `fmadds`: the slope terms of `ftYs_SpecialAirSLoop_0_Phys` (`slope * (dir * x94) + 1`,
//!   `-(slope * dir - 1)` twice, and `gr_vel += delta * mpLib_800569EC(..)`), the same `gr_vel +=` of
//!   `Loop_1_Phys`, `abs_xE4 * (abs_vel / abs_x10) + -xE4` (four copies in `UpdateLoop1Rotation`),
//!   `ABS(x10) + 0.7F * ABS(x10)` of `Loop_1_Anim`, `pos.y += 0.5F * ABS(top + bottom)` of the wall bounce;
//! * f64 `mul_add` (a true fma): `M_TAU / 25.0 * (xD8 * |x1C|) + x14` of `Loop_0_Anim` / `Loop_3_Anim`,
//!   `2.0 * M_PI / 25.0 * xA0 + x14` of `Loop_1_Anim`, `M_PI / 30.0 * x60 + x14` of `Loop_2_Anim`,
//!   `delta / total * M_PI + M_PI_2` (and `+ 3.0 * M_PI_2`) of `UpdateLoop1Rotation2`, and the `Loop_3_Coll`
//!   `(2 * M_PI / 25.0) * (xD8 * |x1C|) + x14`.
//! State-oracle checked against the C (tests/oracle_ftyoshispecials.rs, gen/oracle); the earlier rounds'
//! thin ports are re-exported and checked through the legacy `crate::stubs` hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::tu::ftinl;
use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 1/2/3/5...): re-exported under this TU.
pub use crate::chars::ftYoshi::ftyoshispecials::fn_8012EFF4 as fn_8012EFF4_fp;
pub use crate::chars::ftYoshi::ftyoshispecials::{
    ftYs_SpecialAirSLanding_IASA, ftYs_SpecialAirSStart_0_Phys, ftYs_SpecialAirSStart_1_IASA,
};

const M_PI: f64 = 3.14159265358979323846;
const M_PI_2: f64 = M_PI / 2.0;
const M_TAU: f64 = 6.283185307179586;

/// `ftYs_Unk3_803CED48` (static): which of the 14 animation steps show the egg.
pub const ftYs_Unk3_803CED48: [i32; 15] = [0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 1];
/// `ftYs_Unk3_803CED84` / `ftYs_Unk3_803CED94`: the squash scale per step (double literals in a float array).
pub const ftYs_Unk3_803CED84: [f32; 4] = [0.65_f64 as f32, 0.7_f64 as f32, 0.8_f64 as f32, 1.0];
pub const ftYs_Unk3_803CED94: [f32; 4] = [1.1_f64 as f32, 1.35_f64 as f32, 1.3_f64 as f32, 1.2_f64 as f32];
/// `ftYs_Unk3_803CEDA4` (static): the wall-check collision box `{ 12, 0, -6, 6, 6, 6 }`.
pub static mut ftYs_Unk3_803CEDA4: ftCollisionBox =
    ftCollisionBox { top: 12.0, bottom: 0.0, left: Vec2 { x: -6.0, y: 6.0 }, right: Vec2 { x: 6.0, y: 6.0 } };
/// `ftYs_MF_SpecialS_Coll` (static const): `ftCommon_GroundAirColl_MF | KeepGfx | SkipModel`.
pub const ftYs_MF_SpecialS_Coll: u32 = 0xC4C5092 | Ft_MF_KeepGfx | Ft_MF_SkipModel;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::ftinl::ext::{
        ftAnim_8006EBA4, ftAnim_IsFramesRemaining, ftCo_Fall_Enter, ftCommon_8007D5D4, ftCommon_8007D7FC,
        ftCommon_CalcGroundAccel_Deaccel, ftCommon_Fall, ftCommon_SetSelfMovementFromGroundedMovement, ft_80081D0C, ft_80082708, ft_8008A2BC,
        Fighter_ChangeMotionState,
    };
    pub use crate::tu::ftinl::ext::{ft_CheckGroundAndLedge as _unused_ft_CheckGroundAndLedge, HSD_JObjSetMtxDirtySub};
    pub static mut FTANIM_SETANIMRATE: Option<fn(*mut HSD_GObj, f32)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_SetAnimRate(gobj: *mut HSD_GObj, rate: f32) {
        match unsafe { FTANIM_SETANIMRATE } { Some(f) => f(gobj, rate), None => { unsafe { let _ = melee_fighter::tu::ftanim::ftAnim_SetAnimRate(gobj as _, rate as _); } return; } }
    }
    pub static mut FTPARTS_80074B0C: Option<fn(*mut HSD_GObj, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn ftParts_80074B0C(gobj: *mut HSD_GObj, model_idx: i32, val: i32) {
        match unsafe { FTPARTS_80074B0C } { Some(f) => f(gobj, model_idx, val), None => { unsafe { let _ = melee_fighter::tu::ftparts::ftParts_80074B0C(gobj as _, model_idx as _, val as _); } return; } }
    }
    pub static mut FTYS_INIT_8012B918: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftYs_Init_8012B918(gobj: *mut HSD_GObj) {
        match unsafe { FTYS_INIT_8012B918 } { Some(f) => f(gobj), None => { unsafe { let _ = crate::tu::ftyoshi::ftYs_Init_8012B918(gobj as _); } return; } }
    }
    hook_or!(FTCOMMON_8007F5CC, ftCommon_8007F5CC, (gobj: *mut Item_GObj, flag: i32), melee_fighter::ftcommon3::ftcommon_8007f5cc);
    hook_or!(FTPARTSETROTY, ftPartSetRotY, (fp: *mut Fighter, part: i32, v: f32),
        |fp: *mut Fighter, part: i32, v: f32| unsafe { melee_fighter::tu::ftparts::ftPartSetRotY(&*fp, part, v) });
    hook_or!(FTPARTSETROTZ, ftPartSetRotZ, (fp: *mut Fighter, part: i32, v: f32),
        |fp: *mut Fighter, part: i32, v: f32| unsafe { melee_fighter::tu::ftparts::ftPartSetRotZ(&*fp, part, v) });
    hook_or!(FTPARTSETROTX, ftPartSetRotX, (fp: *mut Fighter, part: i32, v: f32),
        |fp: *mut Fighter, part: i32, v: f32| unsafe { melee_fighter::tu::ftparts::ftPartSetRotX(&*fp, part, v) });
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    hook!(EFSYNC_SPAWN, efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    hook_or!(FT_PLAYSFX, ft_PlaySFX, (fp: *mut Fighter, id: i32, vol: u8, pan: u8),
        |fp: *mut Fighter, id: i32, vol: u8, pan: u8| unsafe { melee_fighter::ft0877::play_sfx(&mut *fp, id, vol, pan) });
    pub static mut FT_80089824: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ft_80089824(gobj: *mut HSD_GObj) {
        match unsafe { FT_80089824 } { Some(f) => f(gobj), None => { unsafe { let _ = melee_fighter::ft0892::ft_80089824(gobj as _); } return; } }
    }
    hook!(FT_800892A0, ft_800892A0, (gobj: *mut HSD_GObj));
    hook!(FTCOLL_8007AFF8, ftColl_8007AFF8, (gobj: *mut HSD_GObj));
    hook!(FTCOLL_8007ABD0, ftColl_8007ABD0, (hit: *mut HitCapsule, damage: u32, gobj: *mut HSD_GObj));
    pub static mut FT_80082888: Option<fn(*mut HSD_GObj, *mut ftCollisionBox) -> bool> = None;
    #[allow(non_snake_case)]
    pub fn ft_80082888(gobj: *mut HSD_GObj, ecb: *mut ftCollisionBox) -> bool {
        match unsafe { FT_80082888 } { Some(f) => f(gobj, ecb), None => { return unsafe { melee_fighter::tu::ft_081b::ft_80082888(gobj as _, ecb as _) as _ }; } }
    }
    pub static mut FT_80082978: Option<fn(*mut HSD_GObj, *mut ftCollisionBox) -> bool> = None;
    #[allow(non_snake_case)]
    pub fn ft_80082978(gobj: *mut HSD_GObj, ecb: *mut ftCollisionBox) -> bool {
        match unsafe { FT_80082978 } { Some(f) => f(gobj, ecb), None => { return unsafe { melee_fighter::tu::ft_081b::ft_80082978(gobj as _, ecb as _) as _ }; } }
    }
    pub static mut FT_800824A0: Option<fn(*mut HSD_GObj, *mut ftCollisionBox) -> bool> = None;
    #[allow(non_snake_case)]
    pub fn ft_800824A0(gobj: *mut HSD_GObj, ecb: *mut ftCollisionBox) -> bool {
        match unsafe { FT_800824A0 } { Some(f) => f(gobj, ecb), None => { return unsafe { melee_fighter::tu::ft_081b::ft_800824a0(gobj as _, ecb as _) as _ }; } }
    }
    hook_or!(FTCOMMON_8007EBAC, ftCommon_8007EBAC, (fp: *mut Fighter, a: u32, b: u32),
        |fp: *mut Fighter, a: u32, b: u32| unsafe { melee_fighter::ftcommon2::ftcommon_8007ebac(&mut *fp, a, b) });
    pub static mut FTCO_80096900: Option<fn(*mut HSD_GObj, i32, i32, i32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_80096900(gobj: *mut HSD_GObj, a: i32, b: i32, allow_interrupt: i32, mobility: f32, landing_lag: f32) {
        match unsafe { FTCO_80096900 } { Some(f) => f(gobj, a, b, allow_interrupt, mobility, landing_lag), None => { unsafe { let _ = melee_fighter::tu::ftCo_FallSpecial::ftCo_80096900(gobj as _, a as _, b as _, allow_interrupt as _, mobility as _, landing_lag as _); } return; } }
    }
    hook!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, (kind: i32, pos: *mut Vec3));
    hook!(MPLIB_800569EC, mpLib_800569EC, (flags: u32) -> f32);
    hook_or!(ATAN2F, atan2f, (y: f32, x: f32) -> f32, melee_runtime::lbtrigf::atan2f);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ABS(x)` (Runtime/platform.h): `(x) < 0 ? -(x) : (x)`.
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// `M_PI_2 * facing_dir` converted to the float argument of ftPartSetRotY.
#[inline(always)]
fn pi_2_facing(fp: *mut Fighter) -> f32 {
    unsafe { (M_PI_2 * (*fp).facing_dir as f64) as f32 }
}

/// The Fighter callback entries of this TU (the statics stored in the SetCall inline).
pub unsafe extern "C" fn cb_fn_8012EC7C(g: *mut HSD_GObj) {
    fn_8012EC7C(g);
}
pub unsafe extern "C" fn cb_fn_8012EDE8(g: *mut HSD_GObj) {
    fn_8012EDE8(g);
}
pub unsafe extern "C" fn cb_fn_8012EFC0(g: *mut HSD_GObj) {
    fn_8012EFC0(g);
}
pub unsafe extern "C" fn cb_fn_8012EFF4(g: *mut HSD_GObj) {
    fn_8012EFF4(g);
}
pub unsafe extern "C" fn cb_fn_8013295C(g: *mut HSD_GObj) {
    fn_8013295C(g);
}

/// `fn_8012EFF4` over the gobj (the round-1 port, `fn_8012EFF4_fp`, takes the fighter).
pub fn fn_8012EFF4(gobj: *mut HSD_GObj) {
    unsafe { fn_8012EFF4_fp(&mut *fpp(gobj)) }
}

/// `ftYoshi_SpecialS_SetCall(gobj)` (static inline).
pub unsafe fn ftYoshi_SpecialS_SetCall(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    (*fp).death2_cb = Some(cb_fn_8012EC7C);
    (*fp).take_dmg_cb = Some(cb_fn_8012EDE8);
    (*fp).deal_dmg_cb = Some(cb_fn_8012EFF4);
    (*fp).x21F8 = Some(cb_fn_8013295C);
}

/// `ftYs_SpecialS_SpawnWallBounceEffect(gobj, cd, facing_right)` (static inline).
pub unsafe fn ftYs_SpecialS_SpawnWallBounceEffect(gobj: *mut HSD_GObj, cd: *mut CollData, facing_right: bool) {
    let fp = fpp(gobj);
    let mut angle: f32;
    let mut pos: Vec3 = (*fp).cur_pos;
    if facing_right {
        angle = ext_atan2f(-(*cd).left_facing_wall.normal.x, (*cd).left_facing_wall.normal.y);
        pos.x += abs_c((*fp).coll_data.ecb.right.x);
    } else {
        angle = ext_atan2f(-(*cd).right_facing_wall.normal.x, (*cd).right_facing_wall.normal.y);
        pos.x -= abs_c((*fp).coll_data.ecb.left.x);
    }
    pos.y = fmadds(0.5, abs_c((*fp).coll_data.ecb.top.y + (*fp).coll_data.ecb.bottom.y), pos.y);
    ext::efSync_Spawn(0x406, gobj, &[&mut pos as *mut Vec3 as *mut c_void, &mut angle as *mut f32 as *mut c_void]);
    ext::Camera_RequestQuake(QuakeKind_Medium as i32, &mut pos);
    ext::ftCommon_8007EBAC(fp, 0xC, 0xA);
}

/// `atan2f` (lb/lbtrigf.c): the game's, from melee_runtime, through the `ext` hook.
#[inline(always)]
fn ext_atan2f(y: f32, x: f32) -> f32 {
    ext::atan2f(y, x)
}

pub fn ftYs_SpecialS_8012EB48(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        match (*fp).cmd_vars[0] {
            1 => {
                (*fp).mv.ys.specials.x4 = (*fp).mv.ys.specials.x4.wrapping_add(1);
            }
            2 => {
                (*fp).mv.ys.specials.x4 = 0;
                (*fp).cmd_vars[1] = (*fp).cmd_vars[1].wrapping_add(1);
            }
            _ => {}
        }
        let x4 = (*fp).mv.ys.specials.x4;
        if x4 >= 0 && x4 < 14 {
            ext::ftParts_80074B0C(gobj, 0, ftYs_Unk3_803CED48[x4 as usize]);
            if ftYs_Unk3_803CED48[x4 as usize] == 1 {
                ext::ftYs_Init_8012B918(gobj);
                ext::ftCommon_8007F5CC(gobj, 0);
                (*fp).set_x221E_b4(0);
            } else {
                ext::ftCommon_8007F5CC(gobj, 1);
                (*fp).set_x221E_b4(1);
            }
            if (*fp).cmd_vars[0] == 2 {
                (*fp).mv.ys.specials.x4 = -1;
                (*fp).cmd_vars[0] = 0;
            }
        } else {
            (*fp).cmd_vars[0] = 0;
        }
    }
}

/// `fn_8012EC7C_inline(gobj)` (static inline); `fn_8012EC7C` is the same body.
pub unsafe fn fn_8012EC7C_inline(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let motion = (*fp).motion_id as i32;
    if (motion != 0x16B) && ((motion >= 0x16B) || (motion != 0x167)) {
        ext::ftParts_80074B0C(gobj, 0, 0);
    }
    ftinl::HSD_JObjSetScale(jobj, &(*fp).u.ys.x222C);
    ext::ftPartSetRotY(fp, 0, pi_2_facing(fp));
    ext::ftPartSetRotZ(fp, 2, 0.0);
    if (*fp).mv.ys.specials.x24 != 0.0 {
        (*fp).facing_dir = (*fp).mv.ys.specials.x24;
    }
    (*fp).mv.ys.specials.x24 = 0.0;
    ext::ftCommon_8007F5CC(gobj, 1);
    (*fp).set_x221E_b4(1);
}

/// `ftYs_SpecialS_UpdateScale(gobj, scale)` (static inline).
pub unsafe fn ftYs_SpecialS_UpdateScale(gobj: *mut HSD_GObj, scale: *mut Vec3) {
    let fp = fpp(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let x8 = (*fp).mv.ys.specials.x8;
    if x8 >= 0 && x8 < 4 {
        (*scale).x = (*fp).u.ys.x222C.x;
        (*scale).y = (*fp).u.ys.x222C.y * ftYs_Unk3_803CED84[x8 as usize];
        (*scale).z = (*fp).u.ys.x222C.z * ftYs_Unk3_803CED94[x8 as usize];
        ftinl::HSD_JObjSetScale(jobj, scale);
        (*fp).mv.ys.specials.x8 = (*fp).mv.ys.specials.x8.wrapping_add(1);
    } else {
        ftinl::HSD_JObjSetScale(jobj, &(*fp).u.ys.x222C);
    }
}

/// `ftYs_SpecialS_UpdateScale2(gobj)` (static inline): the same with a local scale.
pub unsafe fn ftYs_SpecialS_UpdateScale2(gobj: *mut HSD_GObj) {
    let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ftYs_SpecialS_UpdateScale(gobj, &mut scale);
}

/// `ftYs_SpecialS_WrapAndSetRotX(gobj)` (static inline).
pub unsafe fn ftYs_SpecialS_WrapAndSetRotX(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    while (*fp).mv.ys.specials.x14 < 0.0 {
        (*fp).mv.ys.specials.x14 = ((*fp).mv.ys.specials.x14 as f64 + 2.0 * M_PI) as f32;
    }
    while ((*fp).mv.ys.specials.x14 as f64) > 2.0 * M_PI {
        (*fp).mv.ys.specials.x14 = ((*fp).mv.ys.specials.x14 as f64 - 2.0 * M_PI) as f32;
    }
    ext::ftPartSetRotX(fp, 3, (*fp).mv.ys.specials.x14);
}

/// `ftYs_SpecialS_CheckButtonPressure(gobj)` (static inline).
pub unsafe fn ftYs_SpecialS_CheckButtonPressure(gobj: *mut HSD_GObj) -> i32 {
    let fp = fpp(gobj);
    let attrs = &*((*fp).dat_attrs as *const ftYoshiAttributes);
    let mut result = 0;
    if ((*fp).active_sticky.lstick.x as f32) < attrs.x98 {
        result = 1;
    }
    (*fp).active_sticky.lstick.x = 0;
    result
}

/// `perm_vel_inline(fp)` / `perm_neg_inline` / `perm_mul_inline` / `perm_vel_x_inline` (static inlines the
/// decompilation's permuter left in the file) and `stack_pad_hack` (stack-layout padding: no state).
pub unsafe fn perm_vel_inline(fp: *mut Fighter) -> f32 {
    (*fp).gr_vel
}
pub fn perm_neg_inline(x: f32) -> f32 {
    -x
}
pub fn perm_mul_inline(a: f32, b: f32) -> f32 {
    a * b
}
pub unsafe fn perm_vel_x_inline(fp: *mut Fighter) -> f32 {
    (*fp).self_vel.x
}
pub fn stack_pad_hack() {}

pub fn fn_8012EC7C(gobj: *mut HSD_GObj) {
    unsafe { fn_8012EC7C_inline(gobj) }
}

pub fn fn_8012EDE8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let motion = (*fp).motion_id as i32;
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        if (motion != 0x16B) && ((motion >= 0x16B) || (motion != 0x167)) {
            ext::lb_8000B1CC((*(*fp).parts.add(4)).joint, core::ptr::null_mut(), &mut v);
            ext::efSync_Spawn(0x4CF, gobj, &[&mut v as *mut Vec3 as *mut c_void, &mut (*fp).co_attrs.xBC as *mut _ as *mut c_void]);
            ext::ft_PlaySFX(fp, 0x44618, 0x7F, 0x40);
        }
        fn_8012EC7C_inline(gobj);
    }
}

pub fn fn_8012EFC0(gobj: *mut HSD_GObj) {
    ext::ft_80089824(gobj);
    ext::ft_800892A0(gobj);
}

pub fn ftYs_SpecialS_8012F0DC(gobj: *mut HSD_GObj, arg1: i32, arg2: i32, arg3: f32) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        ext::ftColl_8007AFF8(gobj);
        if arg1 == 0 {
            ext::Fighter_ChangeMotionState(gobj, 0x167, arg2 as u32, arg3, 1.0, 0.0, core::ptr::null_mut());
            (*fp).gr_vel *= attributes.xD0;
            (*fp).x74_self_accel.y = 0.0;
            (*fp).self_vel.y = 0.0;
        } else {
            ext::Fighter_ChangeMotionState(gobj, 0x16B, arg2 as u32, arg3, 1.0, 0.0, core::ptr::null_mut());
            (*fp).self_vel.x *= attributes.xD0;
            (*fp).self_vel.y *= attributes.xD4;
            (*fp).xE4_ground_accel_1 = 0.0;
            (*fp).gr_vel = 0.0;
        }
        ftYoshi_SpecialS_SetCall(gobj);
        fn_8012EC7C_inline(gobj);
        (*fp).mv.ys.specials.x24 = 0.0;
        ext::ftPartSetRotY(fp, 0, pi_2_facing(fp));
        ext::ftPartSetRotZ(fp, 2, 0.0);
    }
}

pub fn ftYs_SpecialS_8012F35C(gobj: *mut HSD_GObj) {
    unsafe {
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        ftinl::HSD_JObjGetScale(jobj, &mut (*fp).u.ys.x222C);
        let var_f0 = if (*fp).ground_or_air == GA_Ground as _ { attributes.x54 } else { attributes.x58 };
        (*fp).mv.ys.specials.x1C = var_f0;
        {
            let mut var_r5 = 0;
            let fp2 = fpp(gobj);
            let attrs2 = &*((*fp2).dat_attrs as *const ftYoshiAttributes);
            if ((*fp2).active_sticky.lstick.x as f32) < attrs2.x98 {
                var_r5 = 1;
            }
            (*fp2).active_sticky.lstick.x = 0;
            if var_r5 == 1 {
                (*fp).mv.ys.specials.x1C *= attributes.x68;
            }
        }
        (*fp).mv.ys.specials.x0 = attributes.x48;
        (*fp).mv.ys.specials.x4 = -1;
        (*fp).mv.ys.specials.x8 = -1;
        (*fp).mv.ys.specials.xC = 0;
        (*fp).mv.ys.specials.x14 = 0.0;
        (*fp).mv.ys.specials.x18 = 0.0;
        (*fp).mv.ys.specials.x24 = 0.0;
        (*fp).mv.ys.specials.x28 = 0;
        (*fp).mv.ys.specials.x2C = 0;
        (*fp).mv.ys.specials.x30 = 0;
        (*fp).deal_dmg_cb = Some(cb_fn_8012EFF4);
        (*fp).x21F8 = Some(cb_fn_8013295C);
    }
}

pub fn ftYs_SpecialS_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        (*fp).facing_dir = if (*fp).input.lstick[0].x > 0.0 { 1.0 } else { -1.0 };
        ext::Fighter_ChangeMotionState(gobj, 0x168, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        for i in (0..=3).rev() {
            (*fp).cmd_vars[i] = 0;
        }
        ext::ftCommon_8007D5D4(fp);
        ext::ftAnim_8006EBA4(gobj);
        ftYs_SpecialS_8012F35C(gobj);
        (*fp).self_vel.x = (*fp).mv.ys.specials.x1C * (*fp).facing_dir;
        (*fp).x74_self_accel.x = 0.0;
        (*fp).self_vel.y = attributes.x5C;
        (*fp).x74_self_accel.y = attributes.specials_start_gravity;
    }
}

pub fn ftYs_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        (*fp).facing_dir = if (*fp).input.lstick[0].x > 0.0 { 1.0 } else { -1.0 };
        ext::Fighter_ChangeMotionState(gobj, 0x168, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        for i in (0..=3).rev() {
            (*fp).cmd_vars[i] = 0;
        }
        ext::ftAnim_8006EBA4(gobj);
        ftYs_SpecialS_8012F35C(gobj);
        (*fp).self_vel.x = (*fp).mv.ys.specials.x1C * (*fp).facing_dir;
        (*fp).x74_self_accel.x = 0.0;
        (*fp).self_vel.y = attributes.x5C;
        (*fp).x74_self_accel.y = attributes.specials_start_gravity;
    }
}

pub fn ftYs_SpecialAirSStart_0_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        (*fp).mv.ys.specials.x24 = 0.0;
        ftYs_SpecialS_8012EB48(gobj);
        if !ext::ftAnim_IsFramesRemaining(gobj) {
            ext::ftCommon_8007D5D4(fp);
            ext::Fighter_ChangeMotionState(gobj, 0x169, 0x440012, 0.0, 0.0, 0.0, core::ptr::null_mut());
            ftYoshi_SpecialS_SetCall(gobj);
            (*fp).cur_anim_frame = 0.0;
            ext::ftAnim_SetAnimRate(gobj, 0.0);
            let temp_f0 = (*fp).mv.ys.specials.x1C * (*fp).facing_dir;
            (*fp).self_vel.x = temp_f0;
            (*fp).gr_vel = temp_f0;
            (*fp).x74_self_accel.x = 0.0;
            (*fp).xE4_ground_accel_1 = 0.0;
            (*fp).self_vel.y = attributes.x5C;
            (*fp).x74_self_accel.y = attributes.specials_start_gravity;
        }
    }
}

pub fn ftYs_SpecialAirSLoop_0_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        (*fp).mv.ys.specials.x24 = 0.0;
        ftYs_SpecialS_8012EB48(gobj);
        ftYs_SpecialS_UpdateScale(gobj, &mut scale);
        {
            let fp = fpp(gobj);
            let attrs = &*((*fp).dat_attrs as *const ftYoshiAttributes);
            (*fp).mv.ys.specials.xC = (*fp).mv.ys.specials.xC.wrapping_add(1);
            if (*fp).mv.ys.specials.xC >= attrs.xDC && (*fp).x914[0].state != HitCapsule_Disabled {
                (*fp).x914[0].x4 = (*fp).x914[0].x4.wrapping_add(1) % 2;
                (*fp).mv.ys.specials.xC = 0;
            }
        }
        {
            let fp = fpp(gobj);
            let attrs = &*((*fp).dat_attrs as *const ftYoshiAttributes);
            if (*fp).x914[0].state != HitCapsule_Disabled {
                let mut rate: i32;
                if (*fp).ground_or_air == GA_Air as _ {
                    rate = (attrs.xC4 as f32 * (attrs.xC0 + abs_c((*fp).self_vel.x))) as i32;
                } else {
                    rate = (attrs.xC4 as f32 * (attrs.xC0 + abs_c((*fp).gr_vel))) as i32;
                }
                if rate < 1 {
                    rate = 1;
                }
                ext::ftColl_8007ABD0(&mut (*fp).x914[0], rate as u32, gobj);
            }
        }
        {
            let inner = attributes.xD8 * abs_c((*fp).mv.ys.specials.x1C);
            (*fp).mv.ys.specials.x14 = (M_TAU / 25.0).mul_add(inner as f64, (*fp).mv.ys.specials.x14 as f64) as f32;
        }
        ftYs_SpecialS_WrapAndSetRotX(gobj);
        (*fp).mv.ys.specials.x0 = (*fp).mv.ys.specials.x0.wrapping_sub(1);
        if (*fp).mv.ys.specials.x0 <= 0 {
            (*fp).mv.ys.specials.x0 = 0;
            ftYs_SpecialS_8012F0DC(gobj, 0, 0x440012, 0.0);
        }
    }
}

/// `ftYs_SpecialS_UpdateLoop1Rotation(fp, attrs)` (static inline).
pub unsafe fn ftYs_SpecialS_UpdateLoop1Rotation(fp: *mut Fighter, attrs: &ftYoshiAttributes) {
    let abs_x10 = abs_c((*fp).mv.ys.specials.x10);
    let abs_vel = abs_c((*fp).gr_vel);
    let abs_xe4 = abs_c(attrs.xE4);
    if (*fp).mv.ys.specials.x10 > 0.0 {
        if perm_vel_inline(fp) > 0.0 {
            (*fp).mv.ys.specials.x18 = fmadds(abs_xe4, abs_vel / abs_x10, -attrs.xE4);
        } else {
            let mut scaled_abs_x10 = abs_x10;
            scaled_abs_x10 *= 0.7;
            (*fp).mv.ys.specials.x18 = fmadds(abs_xe4, abs_vel / scaled_abs_x10, -attrs.xE4);
        }
    } else if perm_vel_inline(fp) < 0.0 {
        (*fp).mv.ys.specials.x18 = fmadds(abs_xe4, abs_vel / abs_x10, -attrs.xE4);
    } else {
        let mut scaled_abs_x10 = abs_x10;
        scaled_abs_x10 *= 0.7;
        (*fp).mv.ys.specials.x18 = fmadds(abs_xe4, abs_vel / scaled_abs_x10, -attrs.xE4);
    }
    ext::ftPartSetRotZ(fp, 2, (*fp).mv.ys.specials.x18);
}

/// `ftYs_SpecialS_UpdateLoop1Rotation2(fp, angle, total)` (static inline).
pub unsafe fn ftYs_SpecialS_UpdateLoop1Rotation2(fp: *mut Fighter, angle: *mut f32, total: f32) {
    if (*fp).mv.ys.specials.x10 > 0.0 {
        let delta: f32;
        if (*fp).gr_vel > 0.0 {
            delta = abs_c((*fp).mv.ys.specials.x10) - abs_c((*fp).gr_vel);
        } else {
            delta = abs_c((*fp).mv.ys.specials.x10) + abs_c((*fp).gr_vel);
        }
        *angle = ((delta / total) as f64).mul_add(M_PI, M_PI_2) as f32;
    } else {
        let delta: f32;
        if (*fp).gr_vel < 0.0 {
            delta = abs_c((*fp).mv.ys.specials.x10) - abs_c((*fp).gr_vel);
        } else {
            delta = abs_c((*fp).mv.ys.specials.x10) + abs_c((*fp).gr_vel);
        }
        *angle = ((delta / total) as f64).mul_add(M_PI, 3.0 * M_PI_2) as f32;
    }
    while *angle < 0.0 {
        *angle = (*angle as f64 + 2.0 * M_PI) as f32;
    }
    while (*angle as f64) > 2.0 * M_PI {
        *angle = (*angle as f64 - 2.0 * M_PI) as f32;
    }
    ext::ftPartSetRotY(fp, 0, *angle);
}

pub fn ftYs_SpecialAirSLoop_1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let mut angle: f32 = 0.0;
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        ftYs_SpecialS_8012EB48(gobj);
        ftYs_SpecialS_UpdateScale2(gobj);
        (*fp).mv.ys.specials.x14 =
            (2.0 * M_PI / 25.0).mul_add(attributes.xA0 as f64, (*fp).mv.ys.specials.x14 as f64) as f32;
        ftYs_SpecialS_WrapAndSetRotX(gobj);
        ftYs_SpecialS_UpdateLoop1Rotation(fp, attributes);
        {
            let total = fmadds(0.7, abs_c((*fp).mv.ys.specials.x10), abs_c((*fp).mv.ys.specials.x10));
            ftYs_SpecialS_UpdateLoop1Rotation2(fp, &mut angle, total);
        }
        (*fp).mv.ys.specials.x0 = (*fp).mv.ys.specials.x0.wrapping_sub(1);
        if (*fp).mv.ys.specials.x0 <= 0 {
            (*fp).mv.ys.specials.x0 = 0;
            ftYs_SpecialS_8012F0DC(gobj, 0, 0x440012, 0.0);
            return;
        }
        if (*fp).mv.ys.specials.x28.wrapping_rem(attributes.xA4) == 0 {
            let floor_normal = &mut (*fp).coll_data.floor.normal as *mut Vec3;
            let mut dir: f32 = if (*fp).gr_vel < 0.0 { -1.0 } else { 1.0 };
            angle = ext_atan2f(-(*floor_normal).x, (*floor_normal).y);
            ext::efSync_Spawn(
                0x3FF,
                gobj,
                &[&mut (*fp).cur_pos as *mut Vec3 as *mut c_void, &mut dir as *mut f32 as *mut c_void, &mut angle as *mut f32 as *mut c_void],
            );
        }
        (*fp).mv.ys.specials.x28 = (*fp).mv.ys.specials.x28.wrapping_add(1);
    }
}

pub fn ftYs_SpecialAirSEnd_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let mut sp2c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        (*fp).mv.ys.specials.x24 = 0.0;
        if (*fp).cmd_vars[1] == 1 {
            (*fp).cmd_vars[1] = (*fp).cmd_vars[1].wrapping_add(1);
            ext::lb_8000B1CC((*(*fp).parts.add(4)).joint, core::ptr::null_mut(), &mut sp2c);
            ext::efSync_Spawn(0x4CF, gobj, &[&mut sp2c as *mut Vec3 as *mut c_void, &mut (*fp).co_attrs.xBC as *mut _ as *mut c_void]);
        }
        ftYs_SpecialS_8012EB48(gobj);
        ftYs_SpecialS_UpdateScale(gobj, &mut scale);
        if !ext::ftAnim_IsFramesRemaining(gobj) {
            fn_8012EC7C_inline(gobj);
            ext::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftYs_SpecialAirSStart_1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ys.specials.x24 = 0.0;
        (*fp).mv.ys.specials.x30 = 0;
        ftYs_SpecialS_8012EB48(gobj);
        if !ext::ftAnim_IsFramesRemaining(gobj) {
            ext::Fighter_ChangeMotionState(gobj, 0x169, 0x440012, 0.0, 0.0, 0.0, core::ptr::null_mut());
            ftYoshi_SpecialS_SetCall(gobj);
            (*fp).cur_anim_frame = 0.0;
            ext::ftAnim_SetAnimRate(gobj, 0.0);
        }
    }
}

pub fn ftYs_SpecialAirSLoop_2_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        (*fp).mv.ys.specials.x24 = 0.0;
        (*fp).mv.ys.specials.x30 = 0;
        ftYs_SpecialS_8012EB48(gobj);
        ftYs_SpecialS_UpdateScale(gobj, &mut scale);
        (*fp).mv.ys.specials.x14 =
            (M_PI / 30.0).mul_add(attributes.x60 as f64, (*fp).mv.ys.specials.x14 as f64) as f32;
        ftYs_SpecialS_WrapAndSetRotX(gobj);
        (*fp).mv.ys.specials.x0 = (*fp).mv.ys.specials.x0.wrapping_sub(1);
        if (*fp).mv.ys.specials.x0 <= 0 {
            (*fp).mv.ys.specials.x0 = 0;
            ftYs_SpecialS_8012F0DC(gobj, 1, 0x440012, 0.0);
        }
    }
}

pub fn ftYs_SpecialAirSLoop_3_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        (*fp).mv.ys.specials.x24 = 0.0;
        (*fp).mv.ys.specials.x30 = 0;
        ftYs_SpecialS_8012EB48(gobj);
        ftYs_SpecialS_UpdateScale(gobj, &mut scale);
        {
            let fp4 = fpp(gobj);
            let attrs4 = &*((*fp4).dat_attrs as *const ftYoshiAttributes);
            (*fp4).mv.ys.specials.xC = (*fp4).mv.ys.specials.xC.wrapping_add(1);
            if (*fp4).mv.ys.specials.xC >= attrs4.xDC && (*fp4).x914[0].state != HitCapsule_Disabled {
                (*fp4).x914[0].x4 = ((*fp4).x914[0].x4.wrapping_add(1)) & 1;
                (*fp4).mv.ys.specials.xC = 0;
            }
        }
        {
            let fp5 = fpp(gobj);
            let attrs5 = &*((*fp5).dat_attrs as *const ftYoshiAttributes);
            if (*fp5).x914[0].state != HitCapsule_Disabled {
                let mut rate: i32;
                if (*fp5).ground_or_air == GA_Air as _ {
                    let mut vel = (*fp5).self_vel.x;
                    if vel < 0.0 {
                        vel = -vel;
                    }
                    rate = (attrs5.xC4 as f32 * (attrs5.xC0 + vel)) as i32;
                } else {
                    let mut vel = (*fp5).gr_vel;
                    if vel < 0.0 {
                        vel = -vel;
                    }
                    rate = (attrs5.xC4 as f32 * (attrs5.xC0 + vel)) as i32;
                }
                if rate < 1 {
                    rate = 1;
                }
                ext::ftColl_8007ABD0(&mut (*fp5).x914[0], rate as u32, gobj);
            }
        }
        {
            let mut abs_speed = (*fp).mv.ys.specials.x1C;
            if abs_speed < 0.0 {
                abs_speed = -abs_speed;
            }
            let inner = attributes.xD8 * abs_speed;
            (*fp).mv.ys.specials.x14 = (2.0 * M_PI / 25.0).mul_add(inner as f64, (*fp).mv.ys.specials.x14 as f64) as f32;
        }
        ftYs_SpecialS_WrapAndSetRotX(gobj);
        (*fp).mv.ys.specials.x0 = (*fp).mv.ys.specials.x0.wrapping_sub(1);
        if (*fp).mv.ys.specials.x0 <= 0 {
            (*fp).mv.ys.specials.x0 = 0;
            ftYs_SpecialS_8012F0DC(gobj, 1, 0x440012, 0.0);
        }
    }
}

pub fn ftYs_SpecialAirSLanding_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let mut sp30 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        (*fp).mv.ys.specials.x24 = 0.0;
        (*fp).mv.ys.specials.x30 = 0;
        if (*fp).cmd_vars[1] == 1 {
            (*fp).cmd_vars[1] = (*fp).cmd_vars[1].wrapping_add(1);
            ext::lb_8000B1CC((*(*fp).parts.add(4)).joint, core::ptr::null_mut(), &mut sp30);
            ext::efSync_Spawn(0x4CF, gobj, &[&mut sp30 as *mut Vec3 as *mut c_void, &mut (*fp).co_attrs.xBC as *mut _ as *mut c_void]);
        }
        ftYs_SpecialS_8012EB48(gobj);
        ftYs_SpecialS_UpdateScale(gobj, &mut scale);
        if !ext::ftAnim_IsFramesRemaining(gobj) {
            fn_8012EC7C_inline(gobj);
            if attributes.xE8 == 0.0 {
                ext::ftCo_Fall_Enter(gobj);
            } else {
                ext::ftCo_80096900(gobj, 1, 0, 1, 1.0, attributes.xE8);
            }
        }
    }
}

pub fn ftYs_SpecialAirSStart_0_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftYs_SpecialAirSLoop_0_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        if (*fp).mv.ys.specials.x30 != 0 {
            let stick_x = (*fp).input.lstick[0].x;
            let abs_x = if stick_x < 0.0 { -stick_x } else { stick_x };
            if abs_x > attributes.x9C {
                let var_f31: f32 = if stick_x > 0.0 { 1.0 } else { -1.0 };
                if (*fp).facing_dir != var_f31 {
                    ext::ftColl_8007AFF8(gobj);
                    (*fp).mv.ys.specials.x24 = var_f31;
                    ext::Fighter_ChangeMotionState(gobj, 0x166, 0x4C4892, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
                    {
                        let fp2 = fpp(gobj);
                        (*fp2).death2_cb = Some(cb_fn_8012EC7C);
                        (*fp2).take_dmg_cb = Some(cb_fn_8012EDE8);
                        (*fp2).deal_dmg_cb = Some(cb_fn_8012EFF4);
                        (*fp2).x21F8 = Some(cb_fn_8013295C);
                    }
                    (*fp).mv.ys.specials.x10 = (*fp).gr_vel;
                    (*fp).mv.ys.specials.x18 = 0.0;
                    (*fp).mv.ys.specials.x20 = -0.05_f32 * (*fp).gr_vel;
                    (*fp).mv.ys.specials.x28 = 0;
                    {
                        let fp3 = fpp(gobj);
                        while (*fp3).mv.ys.specials.x14 < 0.0 {
                            (*fp3).mv.ys.specials.x14 = ((*fp3).mv.ys.specials.x14 as f64 + 2.0 * M_PI) as f32;
                        }
                        while ((*fp3).mv.ys.specials.x14 as f64) > 2.0 * M_PI {
                            (*fp3).mv.ys.specials.x14 = ((*fp3).mv.ys.specials.x14 as f64 - 2.0 * M_PI) as f32;
                        }
                        ext::ftPartSetRotX(fp3, 3, (*fp3).mv.ys.specials.x14);
                    }
                }
            }
        }
        if (*fp).mv.ys.specials.x0 < attributes.x48.wrapping_sub(attributes.x4C)
            && ((*fp).input.pressed_buttons as i32 & HSD_PAD_B) != 0
        {
            (*fp).mv.ys.specials.x24 = 0.0;
            ftYs_SpecialS_8012F0DC(gobj, 0, 0x440012, 0.0);
        }
    }
}

pub fn ftYs_SpecialAirSLoop_1_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        let x48 = attributes.x48;
        let x4c = attributes.x4C;
        let difference = x48.wrapping_sub(x4c);
        if (*fp).mv.ys.specials.x0 < difference && ((*fp).input.pressed_buttons as i32 & HSD_PAD_B) != 0 {
            ftYs_SpecialS_8012F0DC(gobj, 0, 0x440012, 0.0);
        }
    }
}

pub fn ftYs_SpecialAirSEnd_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftYs_SpecialAirSLoop_2_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        (*fp).mv.ys.specials.x30 = 0;
        if (*fp).mv.ys.specials.x0 < attributes.x48.wrapping_sub(attributes.x4C)
            && ((*fp).input.pressed_buttons as i32 & HSD_PAD_B) != 0
        {
            ftYs_SpecialS_8012F0DC(gobj, 1, 0x440012, 0.0);
        }
    }
}

pub fn ftYs_SpecialAirSLoop_3_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        (*fp).mv.ys.specials.x30 = 0;
        if (*fp).mv.ys.specials.x0 < attributes.x48.wrapping_sub(attributes.x4C)
            && ((*fp).input.pressed_buttons as i32 & HSD_PAD_B) != 0
        {
            ftYs_SpecialS_8012F0DC(gobj, 1, 0x440012, 0.0);
        }
    }
}

pub fn ftYs_SpecialAirSEnd_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftCommon_CalcGroundAccel_Deaccel(fp, (*fp).co_attrs.ground_friction);
        ext::ftCommon_SetSelfMovementFromGroundedMovement(gobj);
    }
}

pub fn ftYs_SpecialAirSStart_1_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        (*fp).mv.ys.specials.x30 = 0;
        ext::ftCommon_Fall(fp, attributes.specials_start_gravity, attributes.specials_start_terminal_vel);
    }
}

/// The body shared by `Loop_2_Phys` and `Loop_3_Phys`.
unsafe fn air_s_loop_phys(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
    (*fp).mv.ys.specials.x30 = 0;
    let temp = (*fp).input.lstick[0].x * attributes.xE0;
    (*fp).self_vel.x += temp;
    let var_f0 = if (*fp).self_vel.x < 0.0 { -(*fp).self_vel.x } else { (*fp).self_vel.x };
    let mut var_f2 = attributes.x90;
    if var_f0 > var_f2 {
        if (*fp).self_vel.x < 0.0 {
            var_f2 = -var_f2;
        }
        (*fp).self_vel.x = var_f2;
    }
    ext::ftCommon_Fall(fp, attributes.specials_start_gravity, attributes.specials_start_terminal_vel);
}

pub fn ftYs_SpecialAirSLoop_2_Phys(gobj: *mut HSD_GObj) {
    unsafe { air_s_loop_phys(gobj) }
}

pub fn ftYs_SpecialAirSLoop_3_Phys(gobj: *mut HSD_GObj) {
    unsafe { air_s_loop_phys(gobj) }
}

pub fn ftYs_SpecialAirSLanding_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        (*fp).mv.ys.specials.x30 = 0;
        ext::ftCommon_Fall(fp, attributes.specials_start_gravity, attributes.specials_start_terminal_vel);
    }
}

pub fn ftYs_SpecialAirSLoop_0_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        let dir: f32;
        let f5: f32;
        let f6: f32;
        let floor_nx = (*fp).coll_data.floor.normal.x;
        if floor_nx > 0.0 {
            dir = 1.0;
        } else {
            dir = -1.0;
        }
        let vel = (*fp).gr_vel;
        if vel > 0.0 {
            if floor_nx > 0.0 {
                let slope = 1.0 - (*fp).coll_data.floor.normal.y;
                f5 = fmadds(slope, dir * attributes.x94, 1.0);
                f6 = -fmadds(slope, dir, -1.0);
            } else {
                let temp = (1.0 - (*fp).coll_data.floor.normal.y) * dir;
                f5 = 1.0 + temp;
                f6 = 1.0 - temp;
            }
        } else if floor_nx < 0.0 {
            let neg_dir = -dir;
            let slope = 1.0 - (*fp).coll_data.floor.normal.y;
            f5 = fmadds(slope, neg_dir * attributes.x94, 1.0);
            f6 = -fmadds(slope, neg_dir, -1.0);
        } else {
            let temp = (1.0 - (*fp).coll_data.floor.normal.y) * -dir;
            f5 = 1.0 + temp;
            f6 = 1.0 - temp;
        }
        let f31 = attributes.x80 * f5;
        let target_speed = attributes.x7C * f5;
        let accel = attributes.x74 * f5;
        let accel2 = attributes.x78 * f6;
        let mut delta: f32;
        {
            let abs_vel = abs_c(vel);
            if abs_vel >= target_speed {
                delta = -accel2;
                if abs_vel + delta < target_speed {
                    delta = abs_vel - target_speed;
                }
            } else {
                delta = accel;
                if abs_vel + accel > target_speed {
                    delta = target_speed - abs_vel;
                }
            }
        }
        if vel < 0.0 {
            delta = -delta;
        }
        (*fp).gr_vel = fmadds(delta, ext::mpLib_800569EC((*fp).coll_data.floor.flags as u32), (*fp).gr_vel);
        if abs_c((*fp).gr_vel) > f31 {
            (*fp).gr_vel = if (*fp).gr_vel < 0.0 { -f31 } else { f31 };
        }
        (*fp).mv.ys.specials.x1C = abs_c((*fp).gr_vel);
        (*fp).x74_self_accel.y = 0.0;
        (*fp).self_vel.y = 0.0;
        ext::ftCommon_SetSelfMovementFromGroundedMovement(gobj);
    }
}

pub fn ftYs_SpecialAirSLoop_1_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).gr_vel = fmadds((*fp).mv.ys.specials.x20, ext::mpLib_800569EC((*fp).coll_data.floor.flags as u32), (*fp).gr_vel);
        (*fp).x74_self_accel.y = 0.0;
        (*fp).self_vel.y = 0.0;
        ext::ftCommon_SetSelfMovementFromGroundedMovement(gobj);
        let crossed = if (*fp).mv.ys.specials.x10 > 0.0 {
            (*fp).gr_vel < 0.0 && abs_c((*fp).gr_vel) > 0.7 * abs_c((*fp).mv.ys.specials.x10)
        } else {
            (*fp).gr_vel > 0.0 && abs_c((*fp).gr_vel) > 0.7 * abs_c((*fp).mv.ys.specials.x10)
        };
        if crossed {
            (*fp).x21EC = Some(cb_fn_8012EFC0);
            ext::Fighter_ChangeMotionState(gobj, 0x165, 0x02440812, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            ext::ft_PlaySFX(fpp(gobj), 0x44621, 0x7F, 0x40);
            ftYoshi_SpecialS_SetCall(gobj);
            if (*fp).mv.ys.specials.x24 != 0.0 {
                (*fp).facing_dir = (*fp).mv.ys.specials.x24;
            }
            (*fp).mv.ys.specials.x24 = 0.0;
            (*fp).mv.ys.specials.xC = 0;
            ext::ftPartSetRotY(fp, 0, pi_2_facing(fp));
            ext::ftPartSetRotZ(fp, 2, 0.0);
            ftYs_SpecialS_WrapAndSetRotX(gobj);
        }
    }
}

pub fn ftYs_SpecialAirSStart_0_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if ext::ft_80082708(gobj) == 0 {
            ftinl::ftCommon_GroundToAirStateChange(gobj, fp, 360, ftYs_MF_SpecialS_Coll);
            ftYoshi_SpecialS_SetCall(gobj);
        }
    }
}

pub fn ftYs_SpecialAirSLoop_0_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        let coll_result = ext::ft_80082888(gobj, &mut ftYs_Unk3_803CEDA4);
        let wall_hit: bool;
        if (*fp).facing_dir == 1.0 {
            wall_hit = ((*fp).coll_data.env_flags & 0x3F) != 0;
            if wall_hit {
                ftYs_SpecialS_SpawnWallBounceEffect(gobj, &mut (*fp).coll_data, true);
            }
        } else {
            wall_hit = ((*fp).coll_data.env_flags & 0xFC0) != 0;
            if wall_hit {
                ftYs_SpecialS_SpawnWallBounceEffect(gobj, &mut (*fp).coll_data, false);
            }
        }
        if wall_hit {
            ext::ftCommon_8007D5D4(fp);
            ftYs_SpecialS_8012F0DC(gobj, 1, 0x4C4092, 0.0);
            (*fp).self_vel.x *= perm_neg_inline(attributes.xAC);
            (*fp).self_vel.y = attributes.xB0;
        } else if !coll_result {
            ext::ftCommon_8007D5D4(fp);
            ext::Fighter_ChangeMotionState(gobj, 0x16A, 0x0C4C509A, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            ftYoshi_SpecialS_SetCall(gobj);
            ftYs_SpecialS_WrapAndSetRotX(gobj);
        }
        (*fp).mv.ys.specials.x30 = (*fp).mv.ys.specials.x30.wrapping_add(1);
    }
}

pub fn ftYs_SpecialAirSLoop_1_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        let coll_result: bool;
        if abs_c((*fp).gr_vel) > attributes.xA8 {
            coll_result = ext::ft_80082888(gobj, &mut ftYs_Unk3_803CEDA4);
        } else {
            coll_result = ext::ft_80082978(gobj, &mut ftYs_Unk3_803CEDA4);
        }
        let wall_hit: bool;
        if perm_vel_x_inline(fp) > 0.0 {
            wall_hit = ((*fp).coll_data.env_flags & 0x3F) != 0;
            if wall_hit {
                ftYs_SpecialS_SpawnWallBounceEffect(gobj, &mut (*fp).coll_data, true);
            }
        } else {
            wall_hit = ((*fp).coll_data.env_flags & 0xFC0) != 0;
            if wall_hit {
                ftYs_SpecialS_SpawnWallBounceEffect(gobj, &mut (*fp).coll_data, false);
            }
        }
        if wall_hit {
            ext::ftCommon_8007D5D4(fp);
            ftYs_SpecialS_8012F0DC(gobj, 1, 0x4C4092, 0.0);
            (*fp).self_vel.x *= -attributes.xAC;
            (*fp).self_vel.y = attributes.xB0;
        } else if !coll_result {
            ext::ftCommon_8007D5D4(fp);
            ext::Fighter_ChangeMotionState(gobj, 0x16A, 0x0C4C5892, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            ftYoshi_SpecialS_SetCall(gobj);
            if (*fp).mv.ys.specials.x24 != 0.0 {
                (*fp).facing_dir = (*fp).mv.ys.specials.x24;
                (*fp).mv.ys.specials.x1C = if perm_vel_x_inline(fp) < 0.0 { -perm_vel_x_inline(fp) } else { (*fp).self_vel.x };
            }
            (*fp).mv.ys.specials.x24 = 0.0;
            ext::ftPartSetRotY(fp, 0, pi_2_facing(fp));
            ext::ftPartSetRotZ(fp, 2, 0.0);
        }
    }
}

pub fn ftYs_SpecialAirSEnd_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if ext::ft_80082708(gobj) == GA_Ground as i32 {
            ext::ftCommon_8007D5D4(fp);
            ftYs_SpecialS_8012F0DC(gobj, 1, 0x4C4092, (*fp).cur_anim_frame);
        }
    }
}

pub fn ftYs_SpecialAirSStart_1_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ys.specials.x30 = 0;
        if ext::ft_80081D0C(gobj) != GA_Ground as i32 {
            ftinl::ftCommon_AirToGroundStateChange(gobj, fp, 356, ftYs_MF_SpecialS_Coll);
            ftYoshi_SpecialS_SetCall(gobj);
        }
    }
}

pub fn ftYs_SpecialAirSLoop_2_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        stack_pad_hack();
        (*fp).mv.ys.specials.x30 = 0;
        let coll_result = ext::ft_800824A0(gobj, &mut ftYs_Unk3_803CEDA4);
        let wall_hit: bool;
        if (*fp).facing_dir == 1.0 {
            wall_hit = ((*fp).coll_data.env_flags & 0x3F) != 0;
            if wall_hit {
                ftYs_SpecialS_SpawnWallBounceEffect(gobj, &mut (*fp).coll_data, true);
            }
        } else {
            wall_hit = ((*fp).coll_data.env_flags & 0xFC0) != 0;
            if wall_hit {
                ftYs_SpecialS_SpawnWallBounceEffect(gobj, &mut (*fp).coll_data, false);
            }
        }
        if wall_hit {
            ftYs_SpecialS_8012F0DC(gobj, 1, 0x4C4092, 0.0);
            (*fp).self_vel.x *= -attributes.xAC;
            (*fp).self_vel.y = attributes.xB0;
            return;
        }
        if coll_result {
            let vel_y = abs_c((*fp).self_vel.y * attributes.xB4);
            (*fp).self_vel.y = vel_y;
            if vel_y < attributes.xB8 {
                ext::ftCommon_8007D7FC(fp);
                ext::Fighter_ChangeMotionState(gobj, 0x165, 0x440012, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
                ext::ft_PlaySFX(fpp(gobj), 0x44621, 0x7F, 0x40);
                ftYoshi_SpecialS_SetCall(gobj);
                (*fp).self_vel.z = 0.0;
                (*fp).self_vel.y = 0.0;
                if abs_c((*fp).self_vel.x) < 0.01 {
                    (*fp).self_vel.x = 0.01 * (*fp).facing_dir;
                }
                (*fp).gr_vel = (*fp).self_vel.x;
                (*fp).mv.ys.specials.x1C = abs_c((*fp).gr_vel);
                if abs_c((*fp).input.lstick[0].x) > attributes.x9C {
                    (*fp).facing_dir = if (*fp).input.lstick[0].x > 0.0 { 1.0 } else { -1.0 };
                    (*fp).mv.ys.specials.x1C = attributes.xBC * abs_c((*fp).input.lstick[0].x);
                    let v = (*fp).mv.ys.specials.x1C * (*fp).facing_dir;
                    (*fp).gr_vel = v;
                    (*fp).self_vel.x = v;
                }
                ext::ftCommon_8007EBAC(fp, 1, 0);
            } else {
                ext::Fighter_ChangeMotionState(gobj, 0x16A, 0x440012, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
                ftYoshi_SpecialS_SetCall(gobj);
                ftYs_SpecialS_WrapAndSetRotX(gobj);
            }
            (*fp).mv.ys.specials.x1C = attributes.x64;
            if ftYs_SpecialS_CheckButtonPressure(gobj) == 1 {
                (*fp).mv.ys.specials.x1C *= attributes.x68;
            }
            (*fp).self_vel.x = (*fp).mv.ys.specials.x1C * (*fp).facing_dir;
            (*fp).mv.ys.specials.x8 = 0;
        }
    }
}

pub fn ftYs_SpecialAirSLoop_3_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        (*fp).mv.ys.specials.x30 = 0;
        let coll_result = ext::ft_800824A0(gobj, &mut ftYs_Unk3_803CEDA4);
        let wall_hit: i32;
        if (*fp).facing_dir == 1.0 {
            wall_hit = ((*fp).coll_data.env_flags & 0x3F) as i32;
            if wall_hit != 0 {
                ftYs_SpecialS_SpawnWallBounceEffect(gobj, &mut (*fp).coll_data, true);
            }
        } else {
            wall_hit = ((*fp).coll_data.env_flags & 0xFC0) as i32;
            if wall_hit != 0 {
                ftYs_SpecialS_SpawnWallBounceEffect(gobj, &mut (*fp).coll_data, false);
            }
        }
        if wall_hit != 0 {
            ftYs_SpecialS_8012F0DC(gobj, 1, 0x4C4092, 0.0);
            (*fp).self_vel.x *= -attributes.xAC;
            (*fp).self_vel.y = attributes.xB0;
            return;
        }
        if coll_result {
            let abs_y = abs_c((*fp).self_vel.y * attributes.xB4);
            (*fp).self_vel.y = abs_y;
            if abs_y < attributes.xB8 {
                ext::ftCommon_8007D7FC(fp);
                ext::Fighter_ChangeMotionState(gobj, 0x165, 0x0C4C509A, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
                ext::ft_PlaySFX(fpp(gobj), 0x44621, 0x7F, 0x40);
                ftYoshi_SpecialS_SetCall(gobj);
                if abs_c((*fp).mv.ys.specials.x1C) < 0.01 {
                    (*fp).mv.ys.specials.x1C = 0.01;
                }
                let v = (*fp).mv.ys.specials.x1C * (*fp).facing_dir;
                (*fp).gr_vel = v;
                (*fp).self_vel.x = v;
                (*fp).self_vel.z = 0.0;
                (*fp).self_vel.y = 0.0;
                let inner = perm_mul_inline(attributes.xD8, abs_c((*fp).mv.ys.specials.x1C));
                (*fp).mv.ys.specials.x14 =
                    (2.0 * M_PI / 25.0).mul_add(inner as f64, (*fp).mv.ys.specials.x14 as f64) as f32;
                ftYs_SpecialS_WrapAndSetRotX(gobj);
                ext::ftCommon_8007EBAC(fp, 1, 0);
            } else {
                if abs_c((*fp).input.lstick[0].x) > attributes.x9C {
                    (*fp).facing_dir = if (*fp).input.lstick[0].x > 0.0 { 1.0 } else { -1.0 };
                    (*fp).mv.ys.specials.x1C = attributes.xBC * abs_c((*fp).input.lstick[0].x);
                    let v = (*fp).mv.ys.specials.x1C * (*fp).facing_dir;
                    (*fp).gr_vel = v;
                    (*fp).self_vel.x = v;
                    ext::ftPartSetRotY(fp, 0, pi_2_facing(fp));
                }
                ftYs_SpecialS_WrapAndSetRotX(gobj);
            }
            (*fp).mv.ys.specials.x8 = 0;
        }
    }
}

pub fn ftYs_SpecialAirSLanding_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ys.specials.x30 = 0;
        if ext::ft_80081D0C(gobj) != GA_Ground as i32 {
            ext::ftCommon_8007D7FC(fp);
            ftYs_SpecialS_8012F0DC(gobj, 0, 0x4C4092, (*fp).cur_anim_frame);
        }
    }
}

pub fn fn_8013295C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftPartSetRotY(fp, 0, pi_2_facing(fp));
        (*fp).self_vel.x = -(*fp).self_vel.x;
        (*fp).gr_vel = -(*fp).gr_vel;
        (*fp).xE4_ground_accel_1 = -(*fp).xE4_ground_accel_1;
        (*fp).mv.ys.specials.x10 = -(*fp).mv.ys.specials.x10;
        (*fp).mv.ys.specials.x14 = -(*fp).mv.ys.specials.x14;
        (*fp).mv.ys.specials.x18 = -(*fp).mv.ys.specials.x18;
        (*fp).mv.ys.specials.x1C = -(*fp).mv.ys.specials.x1C;
        (*fp).mv.ys.specials.x20 = -(*fp).mv.ys.specials.x20;
        (*fp).mv.ys.specials.x24 = -(*fp).mv.ys.specials.x24;
    }
}
