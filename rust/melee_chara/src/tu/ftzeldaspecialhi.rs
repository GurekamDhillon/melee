//! src/melee/ft/kinds/ftZelda/ftzeldaspecialhi.c: Zelda's Farore's Wind (up B). Fused sites
//! (tools/rust-port/fma_sites.py): the three Newton-Raphson steps of the hand-written square root in
//! `ftZd_SpecialHi_8013A058` / `_8013A244` (`3.0 - (g * g) * x` as one double fma, like math_ppc.h's
//! sqrtf), and the `x54 * stick + x58` of the launch speed (one site in the ground start, two in the
//! air start), written with `melee_core::fp::fmadds`. State-oracle checked against the C
//! (tests/oracle_ftzeldaspecialhi.rs, gen/oracle); the earlier rounds' thin ports are re-exported and
//! checked through the legacy `crate::stubs` hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::tu::ftinl;
use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_runtime::gekko::frsqrte;
use melee_types::*;

/// Already ported (rounds 2/3/4/11 and the first round): re-exported under this TU.
pub use crate::chars::r11::ftZelda::ftzeldaspecialhi::ftZd_SpecialAirHiStart_0_Coll;
pub use crate::chars::r11::ftZelda::ftzeldaspecialhi::ftZd_SpecialAirHi_Coll;
pub use crate::chars::r2::ftZelda::ftzeldaspecialhi::{ftZd_SpecialHiStart_0_Phys, ftZd_SpecialHi_Anim, ftZd_SpecialHi_Phys, ftZd_SpecialHi_8013A648};
pub use crate::chars::r3::ftZelda::ftzeldaspecialhi::{ftZd_SpecialHi_80139FE8, ftZd_SpecialHi_Coll};
pub use crate::chars::r4::ftZelda::ftzeldaspecialhi::{ftZd_SpecialHiStart_0_Coll, ftZd_SpecialHi_80139B44, ftZd_SpecialHi_80139BB0};

const M_PI: f64 = 3.14159265358979323846;
/// `Collide_CeilingMask` (mp/forward.h).
const Collide_CeilingMask: i32 = Collide_CeilingPush | Collide_CeilingHug;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::ftinl::ext::{
        ftAnim_8006EBA4, ftAnim_IsFramesRemaining, ftCliffCommon_80081298, ftCommon_8007D60C, ftCommon_8007D7FC,
        ftCommon_CalcSelfAccel_DeaccelAir, ftCommon_ClampSelfVelX, ftCommon_FallBasic, ftCommon_Fall,
        ftCommon_SetSelfMovementFromGroundedMovement, ft_80082708, ft_800827A0, ft_80084EEC, ft_80084F3C,
        ft_8008A2BC, ft_CheckGroundAndLedge, Fighter_ChangeMotionState,
    };
    hook_or!(FTPARTS_GETBONEINDEX, ftParts_GetBoneIndex, (fp: *mut Fighter, part: i32) -> i32,
        |fp: *mut Fighter, part: i32| unsafe { melee_fighter::hooks::ftparts_get_bone_index(&*fp, part) });
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    hook_or!(COSF, cosf, (x: f32) -> f32, melee_runtime::gekko::cosf);
    hook_or!(SINF, sinf, (x: f32) -> f32, melee_runtime::gekko::sinf);
    hook_or!(ATAN2F, atan2f, (y: f32, x: f32) -> f32, melee_runtime::lbtrigf::atan2f);
    pub static mut LB_800119DC: Option<fn(*mut Vec3, i32, f32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn lb_800119DC(pos: *mut Vec3, a: i32, b: f32, c: f32, d: f32) {
        match unsafe { LB_800119DC } { Some(f) => f(pos, a, b, c, d), None => { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800119DC(pos as _, a as _, b as _, c as _, d as _); } return; } }
    }
    hook!(EFSYNC_SPAWN, efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    pub static mut FTANIM_SETANIMRATE: Option<fn(*mut HSD_GObj, f32)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_SetAnimRate(gobj: *mut HSD_GObj, rate: f32) {
        match unsafe { FTANIM_SETANIMRATE } { Some(f) => f(gobj, rate), None => { unsafe { let _ = melee_fighter::tu::ftanim::ftAnim_SetAnimRate(gobj as _, rate as _); } return; } }
    }
    hook_or!(LBVECTOR_ANGLEXY, lbVector_AngleXY, (a: *mut Vec3, b: *mut Vec3) -> f32,
        |a: *mut Vec3, b: *mut Vec3| unsafe {
            melee_runtime::lbvector::angle_xy(
                &*(a as *const Vec3 as *const melee_core::vec::Vec3),
                &*(b as *const Vec3 as *const melee_core::vec::Vec3),
            )
        });
    hook_or!(FTCO_8009A134, ftCo_8009A134, (gobj: *mut HSD_GObj) -> bool, melee_fighter::tu::ftCo_Pass::ftCo_8009A134);
    hook_or!(FTCOMMON_UPDATEFACING, ftCommon_UpdateFacing, (fp: *mut Fighter),
        |fp: *mut Fighter| unsafe { melee_fighter::ftcommon::update_facing(&mut *fp) });
    hook_or!(FTCOMMON_8007DA24, ftCommon_8007DA24, (fp: *mut Fighter),
        |fp: *mut Fighter| unsafe { melee_fighter::ftcommon::update_facing_outside_deadzone(&mut *fp) });
    hook!(FTCOLL_8007B62C, ftColl_8007B62C, (gobj: *mut HSD_GObj, arg: i32));
    hook_or!(SCRIPTGAME_CAPSMAXJUMPS, ScriptGame_CapsMaxJumps, (fp: *mut Fighter) -> i32,
        |fp: *mut Fighter| unsafe { crate::chars::caps_max_jumps(&*fp) });
    pub static mut FTCO_80096900: Option<fn(*mut HSD_GObj, i32, i32, i32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_80096900(gobj: *mut HSD_GObj, a: i32, b: i32, allow_interrupt: i32, mobility: f32, landing_lag: f32) {
        match unsafe { FTCO_80096900 } { Some(f) => f(gobj, a, b, allow_interrupt, mobility, landing_lag), None => { unsafe { let _ = melee_fighter::tu::ftCo_FallSpecial::ftCo_80096900(gobj as _, a as _, b as _, allow_interrupt as _, mobility as _, landing_lag as _); } return; } }
    }
    hook!(FTCO_LANDINGFALLSPECIAL_ENTER, ftCo_LandingFallSpecial_Enter, (gobj: *mut HSD_GObj, allow_interrupt: i32, landing_lag: f32));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftZd_SpecialHi_801396AC` / `_8013979C` as accessory4 callback entries.
pub use crate::chars::cb::ftzeldaspecialhi__ftZd_SpecialHi_801396AC as cb_ftZd_SpecialHi_801396AC;
pub unsafe extern "C" fn cb_ftZd_SpecialHi_8013979C(g: *mut HSD_GObj) {
    ftZd_SpecialHi_8013979C(g);
}

/// `transition_flags0` / `transition_flags1` (static const).
pub const transition_flags0: u32 = Ft_MF_KeepGfx
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27;
pub const transition_flags1: u32 = transition_flags0 | Ft_MF_KeepColAnimHitStatus;

pub fn ftZd_SpecialHi_801396AC(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ftZd_SpecialHi_801396E0(gobj);
        (*fp).accessory4_cb = None;
    }
}

pub fn ftZd_SpecialHi_801396E0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_HipN as usize)).joint, core::ptr::null_mut(), &mut vec);
            if (*fp).ground_or_air == GA_Ground as _ {
                ext::efSync_Spawn(1270, gobj, &[(*(*fp).parts).joint as *mut c_void]);
            } else {
                ext::efSync_Spawn(1271, gobj, &[(*(*fp).parts).joint as *mut c_void]);
            }
            (*fp).set_x2219_b0(1);
        }
        ftinl::Fighter_SetEffectHitlagCallbacks(fp);
    }
}

pub fn ftZd_SpecialHi_8013979C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        {
            let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_HipN as usize)).joint, core::ptr::null_mut(), &mut vec);
            if (*fp).x2219_b0() == 0 {
                ext::efSync_Spawn(1285, gobj, &[&mut vec as *mut Vec3 as *mut c_void]);
                (*fp).set_x2219_b0(1);
            }
        }
        ftinl::Fighter_SetEffectHitlagCallbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

/// `60 * (float) M_PI / 180`, folded by the compiler in single precision.
const SIXTY_DEG_RAD: f32 = 60.0_f32 * (M_PI as f32) / 180.0_f32;

/// `ftZelda_SpecialHi_StartAction_Helper(fp)` (static): the shockwave at the hand.
pub fn ftZelda_SpecialHi_StartAction_Helper(fp: *mut Fighter) {
    unsafe {
        let bone_index = ext::ftParts_GetBoneIndex(fp, 4);
        let jobj = (*(*fp).parts.add(bone_index as usize)).joint;
        let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC(jobj, core::ptr::null_mut(), &mut vec);
        ext::lb_800119DC(&mut vec, 120, 1.5, 0.02_f64 as f32, SIXTY_DEG_RAD);
    }
}

pub fn ftZd_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let mut fp = fpp(gobj);
        (*fp).gr_vel = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = 0.0;
        ext::Fighter_ChangeMotionState(gobj, 349, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(gobj);
        fp = fpp(gobj);
        (*fp).cmd_vars[0] = 0;
        (*fp).mv.zd.specialhi.xC = 0;
        ftZelda_SpecialHi_StartAction_Helper(fp);
        (*fp).accessory4_cb = Some(cb_ftZd_SpecialHi_801396AC);
    }
}

pub fn ftZd_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        {
            let fp = fpp(gobj);
            let attributes = &*((*fp).dat_attrs as *const ftZelda_DatAttrs);
            (*fp).self_vel.x /= attributes.x38;
            (*fp).self_vel.y /= attributes.x3C;
            ext::Fighter_ChangeMotionState(gobj, 352, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            ext::ftAnim_8006EBA4(gobj);
        }
        {
            let fp = fpp(gobj);
            (*fp).cmd_vars[0] = 0;
            (*fp).mv.zd.specialhi.xC = 0;
            ftZelda_SpecialHi_StartAction_Helper(fp);
            (*fp).accessory4_cb = Some(cb_ftZd_SpecialHi_801396AC);
        }
    }
}

pub fn ftZd_SpecialHiStart_0_Anim(gobj: *mut HSD_GObj) {
    if !ext::ftAnim_IsFramesRemaining(gobj) {
        ftZd_SpecialHi_8013A058(gobj);
    }
}

pub fn ftZd_SpecialAirHiStart_0_Anim(gobj: *mut HSD_GObj) {
    if !ext::ftAnim_IsFramesRemaining(gobj) {
        ftZd_SpecialHi_8013A244(gobj);
    }
}

pub fn ftZd_SpecialHiStart_0_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftZd_SpecialAirHiStart_0_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftZd_SpecialAirHiStart_0_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftZelda_DatAttrs);
        let vec = Vec2 { x: attributes.x40, y: attributes.x44 };
        ext::ftCommon_Fall(fp, vec.x, vec.y);
        ext::ftCommon_CalcSelfAccel_DeaccelAir(fp);
    }
}

pub fn ftZd_SpecialHiStart_1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.zd.specialhi.x0 = (*fp).mv.zd.specialhi.x0.wrapping_sub(1);
        if (*fp).mv.zd.specialhi.x0 <= 0 {
            ftZd_SpecialHi_8013A6A8(gobj);
        }
    }
}

pub fn ftZd_SpecialAirHiStart_1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.zd.specialhi.x0 = (*fp).mv.zd.specialhi.x0.wrapping_sub(1);
        if (*fp).mv.zd.specialhi.x0 <= 0 {
            ftZd_SpecialHi_8013A764(gobj);
        }
    }
}

pub fn ftZd_SpecialHiStart_1_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftZd_SpecialAirHiStart_1_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftZd_SpecialHiStart_1_Phys(gobj: *mut HSD_GObj) {
    ext::ftCommon_SetSelfMovementFromGroundedMovement(gobj);
}

pub fn ftZd_SpecialAirHiStart_1_Phys(_gobj: *mut HSD_GObj) {}

pub fn ftZd_SpecialHiStart_1_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let coll_data = &mut (*fp).coll_data as *mut CollData;
        if ext::ft_80082708(gobj) == 0 {
            let env_flags = (*coll_data).env_flags as i32;
            if (env_flags & Collide_LeftWallMask) != 0 || (env_flags & Collide_RightWallMask) != 0 {
                ext::ftCommon_8007D60C(fp);
                ftZd_SpecialHi_8013A764(gobj);
                return;
            }
            ftZd_SpecialHi_80139F6C(gobj);
        } else {
            let env_flags = (*coll_data).env_flags as i32;
            if (env_flags & Collide_LeftWallMask) != 0 || (env_flags & Collide_RightWallMask) != 0 {
                ftZd_SpecialHi_8013A6A8(gobj);
            }
        }
    }
}

/// `ftZelda_80139D60_Helper(gobj)` (static).
pub fn ftZelda_80139D60_Helper(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fighter2 = fpp(gobj);
        let attributes2 = &*((*fighter2).dat_attrs as *const ftZelda_DatAttrs);
        if (*fighter2).mv.zd.specialhi.xC as f32 >= attributes2.x4C {
            true
        } else {
            !ext::ftCo_8009A134(gobj)
        }
    }
}

/// `ftCommon_HandleTeleportCollisions(gobj, fp, coll, angle_clamp, on_collide)` (ft/inlines.h).
unsafe fn handle_teleport_collisions(
    gobj: *mut HSD_GObj,
    fp: *mut Fighter,
    coll: *mut CollData,
    angle_clamp: *const i32,
    on_collide: fn(*mut HSD_GObj),
) {
    let limit = (90.0_f32 + *angle_clamp as f32) * 0.01745329252_f32;
    if ((*coll).env_flags as i32 & Collide_CeilingMask) != 0
        && ext::lbVector_AngleXY(&mut (*coll).ceiling.normal, &mut (*fp).self_vel) > limit
    {
        on_collide(gobj);
    }
    if ((*coll).env_flags as i32 & Collide_LeftWallMask) != 0
        && ext::lbVector_AngleXY(&mut (*coll).left_facing_wall.normal, &mut (*fp).self_vel) > limit
    {
        on_collide(gobj);
    }
    if ((*coll).env_flags as i32 & Collide_RightWallMask) != 0
        && ext::lbVector_AngleXY(&mut (*coll).right_facing_wall.normal, &mut (*fp).self_vel) > limit
    {
        on_collide(gobj);
    }
}

pub fn ftZd_SpecialAirHiStart_1_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let coll_data = &mut (*fp).coll_data as *mut CollData;
        let sa = (*fp).dat_attrs as *mut ftZelda_DatAttrs;
        (*fp).mv.zd.specialhi.xC = (*fp).mv.zd.specialhi.xC.wrapping_add(1);
        if ext::ft_CheckGroundAndLedge(gobj, ftinl::ftGetFacingDirInt(fp)) {
            if ftZelda_80139D60_Helper(gobj) {
                ftZd_SpecialHi_80139FE8(gobj);
                return;
            }
        }
        if !ext::ftCliffCommon_80081298(gobj) {
            handle_teleport_collisions(gobj, fp, coll_data, &(*sa).x60, ftZd_SpecialHi_8013A764);
        }
    }
}

pub fn ftZd_SpecialHi_80139F6C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftCommon_8007D60C(fp);
        ext::Fighter_ChangeMotionState(gobj, 353, transition_flags1, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
        (*fp).set_x2223_b4(1);
        (*fp).set_invisible(1);
    }
}

/// One step of math_ppc.h's Newton-Raphson reciprocal square root, as the hand-written copy in the C
/// does it: `_half * guess * (_three - guess * guess * x)` with `3.0 - (g*g)*x` contracted to one fma.
#[inline(always)]
fn nr_step(guess: f64, x: f64) -> f64 {
    (0.5 * guess) * (-(guess * guess)).mul_add(x, 3.0)
}

/// `sqrt(var_f4)` as the C computes it (frsqrte + three steps, through the volatile `y`).
#[inline(always)]
fn zd_sqrt(var_f4: f32) -> f32 {
    let x = var_f4 as f64;
    let mut guess = frsqrte(x);
    guess = nr_step(guess, x);
    guess = nr_step(guess, x);
    guess = nr_step(guess, x);
    (x * guess) as f32
}

pub fn ftZd_SpecialHi_8013A058(gobj: *mut HSD_GObj) {
    unsafe {
        let mut fp = fpp(gobj);
        let mut attributes = &*((*fp).dat_attrs as *const ftZelda_DatAttrs);
        let coll_data = &mut (*fp).coll_data as *mut CollData;
        let mut temp_f2 = (*fp).input.lstick[0].x;
        let mut temp_f1 = (*fp).input.lstick[0].y;
        temp_f2 = temp_f2 * temp_f2;
        temp_f1 = temp_f1 * temp_f1;
        let mut temp_f5 = temp_f2 + temp_f1;
        let var_f4 = temp_f5;
        // Essentially performing a square root operation (avoiding sqrtf to preserve stack alignment).
        if var_f4 > 0.0 {
            temp_f5 = zd_sqrt(var_f4);
        }
        let temp_f0: f32 = 1.0;
        let mut var_f31 = temp_f5;
        if temp_f5 > temp_f0 {
            var_f31 = temp_f0;
        }
        if !(var_f31 < attributes.x50) {
            let ground_vector = &mut (*coll_data).floor.normal as *mut Vec3;
            let mut input_vector = Vec3 { x: (*fp).input.lstick[0].x, y: (*fp).input.lstick[0].y, z: 0.0 };
            if !(ext::lbVector_AngleXY(ground_vector, &mut input_vector) < (M_PI / 2.0) as f32) {
                if !ext::ftCo_8009A134(gobj) {
                    ext::ftCommon_UpdateFacing(fp);
                    temp_f5 = ext::atan2f((*fp).input.lstick[0].y, (*fp).input.lstick[0].x * (*fp).facing_dir);
                    (*fp).mv.zd.specialhi.x4.x = input_vector.x;
                    (*fp).mv.zd.specialhi.x4.y = input_vector.y;
                    // Update ground velocity
                    let temp_f6 = fmadds(attributes.x54, var_f31, attributes.x58) * ext::cosf(temp_f5);
                    (*fp).gr_vel = (*fp).facing_dir * temp_f6;
                    ext::Fighter_ChangeMotionState(gobj, 350, 0, 35.0, 1.0, 0.0, core::ptr::null_mut());
                    ext::ftAnim_8006EBA4(gobj);
                    ext::ftAnim_SetAnimRate(gobj, 0.0);
                    fp = fpp(gobj);
                    attributes = &*((*fp).dat_attrs as *const ftZelda_DatAttrs);
                    (*fp).mv.zd.specialhi.x0 = attributes.x48;
                    (*fp).x1968_jumpsUsed = ext::ScriptGame_CapsMaxJumps(fp) as u8;
                    (*fp).set_x2223_b4(1);
                    ext::ftColl_8007B62C(gobj, 2);
                    (*fp).set_invisible(1);
                    return;
                }
            }
        }
        ext::ftCommon_8007D60C(fp);
        ftZd_SpecialHi_8013A244(gobj);
    }
}

pub fn ftZd_SpecialHi_8013A244(gobj: *mut HSD_GObj) {
    unsafe {
        let mut fp = fpp(gobj);
        let mut temp_f2 = (*fp).input.lstick[0].x;
        let mut temp_f1 = (*fp).input.lstick[0].y;
        let mut attributes = &*((*fp).dat_attrs as *const ftZelda_DatAttrs);
        temp_f1 = temp_f1 * temp_f1;
        temp_f2 = temp_f2 * temp_f2;
        let mut var_f5 = temp_f2 + temp_f1;
        let var_f4 = var_f5;
        if var_f4 > 0.0 {
            var_f5 = zd_sqrt(var_f4);
        }
        let temp_f0: f32 = 1.0;
        let mut var_f31 = var_f5;
        if var_f5 > temp_f0 {
            var_f31 = temp_f0;
        }
        let var_f30: f32;
        if var_f31 > attributes.x50 {
            temp_f1 = (*fp).input.lstick[0].x;
            if temp_f1 < 0.0 {
                temp_f1 = -temp_f1;
            }
            if temp_f1 > 0.001 {
                ext::ftCommon_UpdateFacing(fp);
            }
            var_f30 = ext::atan2f((*fp).input.lstick[0].y, (*fp).input.lstick[0].x * (*fp).facing_dir);
            (*fp).mv.zd.specialhi.x4.x = (*fp).input.lstick[0].x;
            (*fp).mv.zd.specialhi.x4.y = (*fp).input.lstick[0].y;
        } else {
            ext::ftCommon_8007DA24(fp);
            var_f30 = (M_PI / 2.0) as f32;
            (*fp).mv.zd.specialhi.x4.x = 0.0;
            var_f31 = 1.0;
            (*fp).mv.zd.specialhi.x4.y = var_f31;
        }
        (*fp).self_vel.x = (*fp).facing_dir * (fmadds(attributes.x54, var_f31, attributes.x58) * ext::cosf(var_f30));
        (*fp).self_vel.y = fmadds(attributes.x54, var_f31, attributes.x58) * ext::sinf(var_f30);
        ext::Fighter_ChangeMotionState(gobj, 353, 0, 35.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(gobj);
        ext::ftAnim_SetAnimRate(gobj, 0.0);
        fp = fpp(gobj);
        attributes = &*((*fp).dat_attrs as *const ftZelda_DatAttrs);
        (*fp).mv.zd.specialhi.x0 = attributes.x48;
        (*fp).x1968_jumpsUsed = ext::ScriptGame_CapsMaxJumps(fp) as u8;
        (*fp).set_x2223_b4(1);
        ext::ftColl_8007B62C(gobj, 2);
        (*fp).set_invisible(1);
    }
}

pub fn ftZd_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftZelda_DatAttrs);
        if !ext::ftAnim_IsFramesRemaining(gobj) {
            let x = attributes.x68;
            let y = attributes.x6C;
            ext::ftCo_80096900(gobj, 1, 0, 1, x, y);
        }
    }
}

pub fn ftZd_SpecialHi_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftZd_SpecialAirHi_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftZd_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attributes = &*((*fp).dat_attrs as *const ftZelda_DatAttrs);
        if (*fp).cmd_vars[0] != 0 {
            ext::ftCommon_FallBasic(fp);
            ext::ftCommon_ClampSelfVelX(fp, attributes.x5C * (*fp).co_attrs.air_drift_max);
            return;
        }
        {
            let vel_y = (*fp).self_vel.y;
            (*fp).self_vel.y = vel_y - vel_y / 10.0;
        }
        ext::ftCommon_CalcSelfAccel_DeaccelAir(fp);
    }
}

pub fn ftZd_SpecialHi_8013A6A8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp0 = fpp(gobj);
        let attributes = &*((*fp0).dat_attrs as *const ftZelda_DatAttrs);
        ext::Fighter_ChangeMotionState(gobj, 351, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(gobj);
        {
            let fp1 = fpp(gobj);
            (*fp1).mv.zd.specialhi.x10.x = (*fp1).self_vel.x;
            (*fp1).mv.zd.specialhi.x10.y = (*fp1).self_vel.y;
            (*fp1).mv.zd.specialhi.x18 = (*fp1).gr_vel;
            (*fp1).self_vel.y = 0.0;
            (*fp1).self_vel.x = 0.0;
            (*fp1).gr_vel = 0.0;
            (*fp1).set_invisible(0);
            (*fp1).accessory4_cb = Some(cb_ftZd_SpecialHi_8013979C);
        }
        (*fp0).gr_vel = (*fp0).mv.zd.specialhi.x18 * attributes.x64;
    }
}

pub fn ftZd_SpecialHi_8013A764(gobj: *mut HSD_GObj) {
    unsafe {
        let fp0 = fpp(gobj);
        let sa = &*((*fp0).dat_attrs as *const ftZelda_DatAttrs);
        ext::Fighter_ChangeMotionState(gobj, 354, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(gobj);
        {
            let fp1 = fpp(gobj);
            (*fp1).mv.zd.specialhi.x10.x = (*fp1).self_vel.x;
            (*fp1).mv.zd.specialhi.x10.y = (*fp1).self_vel.y;
            (*fp1).mv.zd.specialhi.x18 = (*fp1).gr_vel;
            (*fp1).self_vel.y = 0.0;
            (*fp1).self_vel.x = 0.0;
            (*fp1).gr_vel = 0.0;
            (*fp1).set_invisible(0);
            (*fp1).accessory4_cb = Some(cb_ftZd_SpecialHi_8013979C);
        }
        (*fp0).self_vel.x = (*fp0).mv.zd.specialhi.x10.x * sa.x64;
        (*fp0).self_vel.y = (*fp0).mv.zd.specialhi.x10.y * sa.x64;
    }
}
