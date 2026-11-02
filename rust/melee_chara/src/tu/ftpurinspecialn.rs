//! src/melee/ft/kinds/ftPurin/ftpurinspecialn.c: Jigglypuff's Rollout (neutral special), every state of
//! `ftPr_Init_MotionStateTable[5..21]` and the `ftPr_SpecialS_8013D658..8013DD54` helpers.
//! Fused sites (tools/rust-port/fma_sites.py): f32 `fmadds` in the charge-spin Anims
//! (`x14 += x34.x * (x2C * MTXDegToRad(xAC))`: NLoop/NFull/AirNChargeLoop/AirNChargeFull), in
//! `ftPr_SpecialAirNStartTurn_Anim` / `ftPr_SpecialNHit_Anim` (`x14 += xBC * (f32)(0.2 * x6C * -x34.x)`),
//! 4 in `ftPr_SpecialNTurn_Phys` (`gr_vel += scale * (x1C +- influence)`), 1 in `wallBounceEffect`
//! (`pos.y += 0.5f * ABS(top + bottom)`, shared with Kirby's copy); one f64 fma in
//! `ftPr_SpecialNTurn_Anim` (`x14 += 0.2 * x6C * -x34.x`, in double). `mv.pr.specialn.x2C` is an `int`.
//! Kirby's copy ability (tu/ftkirbyspecialpurin.rs) is the same code on Kirby's attributes; the static
//! inlines with identical bodies (ftPurin/inlines.h, playRollSFX, wallBounceEffect) are shared from there.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da;
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
use crate::stubs as st;
use crate::tu::ftkirbyspecialpurin::ext;
pub use crate::tu::ftkirbyspecialpurin::{ftPr_MirrorSpecialN, ftPr_NormalizeAndSetRollAngle, ftPr_SetFacingDir};
use crate::tu::ftkirbyspecialpurin::{ftKb_PrPlayRollSFX, ftKb_PrWallBounceEffect, turn_gr_vel, air_decel_floor};
use crate::tu::ftpurin::{ftPr_Init_803D05C8, ftPr_Init_803D05D8, ftPr_Init_803D0610};
use melee_core::fp::fmadds;
use melee_fighter::hooks as fh;
use melee_runtime::lbtrigf::atan2f;
use melee_types::*;

type G = *mut HSD_GObj;

const M_PI: f64 = core::f64::consts::PI;
const M_PI_2: f64 = core::f64::consts::FRAC_PI_2;

macro_rules! sn {
    ($fp:expr) => {
        (*$fp).mv.pr.specialn
    };
}

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn da_of(fp: *mut Fighter) -> &'static ftPurinAttributes {
    &*((*fp).dat_attrs as *const ftPurinAttributes)
}

#[inline(always)]
unsafe fn jobj_of(gobj: G) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

#[inline(always)]
fn abs_f(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// `SIGNF` (Runtime/platform.h): `x > 0 ? 1 : -1`.
#[inline(always)]
fn signf(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else {
        -1.0
    }
}

/// `MTXDegToRad(a)` (dolphin/mtx.h): `a * 0.01745329252f`.
#[inline(always)]
fn deg_to_rad(a: f32) -> f32 {
    a * 0.01745329252f32
}

/// `ftPartSetRotY(fp, part, <double>)`: the double narrowed to the f32 parameter.
#[inline(always)]
unsafe fn rot_y(fp: *mut Fighter, part: i32, v: f64) {
    ext::ftPartSetRotY(fp, part, v as f32);
}

pub unsafe extern "C" fn cb_ftPr_SpecialS_8013D658(g: G) {
    melee_core::ct!("cb_ftPr_SpecialS_8013D658");
    ftPr_SpecialS_8013D658(g)
}
pub unsafe extern "C" fn cb_ftPr_SpecialS_8013D764(g: G) {
    melee_core::ct!("cb_ftPr_SpecialS_8013D764");
    ftPr_SpecialS_8013D764(g)
}
pub unsafe extern "C" fn cb_ftPr_SpecialN_8014222C(g: G) {
    melee_core::ct!("cb_ftPr_SpecialN_8014222C");
    ftPr_SpecialN_8014222C(g)
}
pub unsafe extern "C" fn cb_ftPr_SpecialS_8013D8B0(g: G) {
    melee_core::ct!("cb_ftPr_SpecialS_8013D8B0");
    ftPr_SpecialS_8013D8B0(g)
}

/// `setupPurinCallbacks` (static inline).
unsafe fn setup_purin_callbacks(gobj: G) {
    let fp = fpp(gobj);
    (*fp).death2_cb = Some(cb_ftPr_SpecialS_8013D658);
    (*fp).take_dmg_cb = Some(cb_ftPr_SpecialS_8013D658);
    (*fp).deal_dmg_cb = Some(cb_ftPr_SpecialS_8013D764);
    (*fp).x21F8 = Some(cb_ftPr_SpecialN_8014222C);
}

/// `scaleAnimStep` (static inline): squash-and-stretch over 4 frames (y from `803D05C8`, z from `803D05D8`).
unsafe fn scale_anim_step(gobj: G, scale: &mut Vec3) {
    let fp = fpp(gobj);
    let jobj = jobj_of(gobj);
    let frame = sn!(fp).x8;
    if frame >= 0 && frame < 4 {
        scale.x = (*fp).u.pr.x2230.x;
        scale.y = (*fp).u.pr.x2230.y * ftPr_Init_803D05C8[frame as usize];
        scale.z = (*fp).u.pr.x2230.z * ftPr_Init_803D05D8[frame as usize];
        ext::HSD_JObjSetScale(jobj, scale);
        sn!(fp).x8 += 1;
    } else {
        ext::HSD_JObjSetScale(jobj, &(*fp).u.pr.x2230);
    }
}

/// `hitCapsuleToggle` (static inline).
unsafe fn hit_capsule_toggle(gobj: G) {
    let fp = fpp(gobj);
    let da = da_of(fp);
    sn!(fp).xC += 1;
    if sn!(fp).xC >= da.x9C && (*fp).x914[0].state != HitCapsule_Disabled {
        (*fp).x914[0].x4 = ((*fp).x914[0].x4 + 1) & 1;
        sn!(fp).xC = 0;
    }
}

/// `ftPr_SpecialS_8013D658`: death / take-damage callback: restore scale and facing.
pub fn ftPr_SpecialS_8013D658(gobj: G) {
    melee_core::ct!("ftPr_SpecialS_8013D658");
    unsafe {
        let fp = fpp(gobj);
        let jobj = jobj_of(gobj);
        ext::HSD_JObjSetScale(jobj, &(*fp).u.pr.x2230);
        rot_y(fp, FtPart_TopN as i32, M_PI_2 * (*fp).facing_dir as f64);
        if sn!(fp).facing_dir != 0.0 {
            (*fp).facing_dir = sn!(fp).facing_dir;
        }
        sn!(fp).facing_dir = 0.0;
    }
}

/// `ftPr_SpecialS_8013D764`: deal-damage callback: bounce off into SpecialNHit.
pub fn ftPr_SpecialS_8013D764(gobj: G) {
    melee_core::ct!("ftPr_SpecialS_8013D764");
    const MF: u32 = (1 << 1) | (1 << 4) | (1 << 7) | (1 << 9) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        sn!(fp).xC = 0;
        let m = (*fp).motion_id as i32;
        if m == ftPr_MS_SpecialNRelease as i32
            || m == ftPr_MS_SpecialNTurn as i32
            || m == ftPr_MS_SpecialAirNChargeRelease as i32
            || m == ftPr_MS_SpecialAirNStartTurn as i32
        {
            sn!(fp).x0 = sn!(fp).x0.wrapping_sub(da.x38);
            (*fp).facing_dir = sn!(fp).x34.x;
            ext::ftColl_8007AFF8(gobj);
            st::Fighter_ChangeMotionState(gobj, ftPr_MS_SpecialNHit as i32, MF, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            (*fp).death2_cb = Some(cb_ftPr_SpecialS_8013D658);
            (*fp).take_dmg_cb = Some(cb_ftPr_SpecialS_8013D658);
            (*fp).deal_dmg_cb = None;
            if (*fp).ground_or_air == GA_Ground {
                (*fp).self_vel.x = (*fp).gr_vel * da.specialn_vel.x;
                ext::ftCommon_8007D5D4(fp);
            } else {
                (*fp).self_vel.x *= da.specialn_vel.x;
            }
            (*fp).self_vel.y = da.specialn_vel.y;
            (*fp).x74_self_accel.y = 0.0;
            (*fp).x74_self_accel.x = 0.0;
            (*fp).xE4_ground_accel_1 = 0.0;
            (*fp).gr_vel = 0.0;
            sn!(fp).facing_dir = 0.0;
            rot_y(fp, FtPart_TopN as i32, M_PI_2);
            fh::play_sfx(&mut *fp, 250070, 127, 64);
        }
    }
}

/// `ftPr_SpecialS_8013D8B0`.
pub fn ftPr_SpecialS_8013D8B0(gobj: G) {
    melee_core::ct!("ftPr_SpecialS_8013D8B0");
    ext::ft_80089824(gobj);
    ext::ft_800892A0(gobj);
}

/// `ftPr_SpecialS_8013D8E4`: the roll hit capsule, off below `xCC`, damage scaled by speed.
pub fn ftPr_SpecialS_8013D8E4(gobj: G) {
    melee_core::ct!("ftPr_SpecialS_8013D8E4");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let state = (*fp).x914[0].state;
        let abs_vel = if (*fp).ground_or_air == GA_Air { abs_f((*fp).self_vel.x) } else { abs_f((*fp).gr_vel) };
        if abs_vel < da.xCC {
            (*fp).x914[0].state = HitCapsule_Disabled;
            ext::ftCo_800BFFAC(fp);
        } else if state == HitCapsule_Disabled {
            (*fp).x914[0].state = HitCapsule_Enabled;
        }
        if (*fp).x914[0].state != HitCapsule_Disabled {
            let v = if (*fp).ground_or_air == GA_Air { abs_f((*fp).self_vel.x) } else { abs_f((*fp).gr_vel) };
            let mut damage = (da.x84 * (da.x80 + v)) as i32;
            if damage < 1 {
                damage = 1;
            }
            ext::ftColl_8007ABD0(&mut (*fp).x914[0], damage as u32, gobj);
        }
    }
}

/// `ftPr_SpecialS_8013DA24`: end of the roll (`is_air` picks the AirNEnd states).
pub fn ftPr_SpecialS_8013DA24(gobj: G, is_air: bool, flags: u32, anim_start: f32) {
    melee_core::ct!("ftPr_SpecialS_8013DA24");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        (*fp).facing_dir = sn!(fp).x34.x;
        ext::ftColl_8007AFF8(gobj);
        if !is_air {
            let msid = if (*fp).facing_dir == 1.0 { ftPr_MS_SpecialNEndR } else { ftPr_MS_SpecialNEndL };
            st::Fighter_ChangeMotionState(gobj, msid as i32, flags, anim_start, 1.0, 0.0, core::ptr::null_mut());
            (*fp).gr_vel *= da.x90;
            (*fp).x74_self_accel.y = 0.0;
            (*fp).self_vel.y = 0.0;
        } else {
            let msid = if (*fp).facing_dir == 1.0 { ftPr_MS_SpecialAirNEndR } else { ftPr_MS_SpecialAirNEndL };
            st::Fighter_ChangeMotionState(gobj, msid as i32, flags, anim_start, 1.0, 0.0, core::ptr::null_mut());
            (*fp).self_vel.x *= da.x90;
            (*fp).self_vel.y *= da.x94;
            (*fp).xE4_ground_accel_1 = 0.0;
            (*fp).gr_vel = 0.0;
        }
        setup_purin_callbacks(gobj);
        ftPr_SpecialS_8013D658(gobj);
        sn!(fp).facing_dir = 0.0;
        rot_y(fp, FtPart_TopN as i32, M_PI_2 * (*fp).facing_dir as f64);
    }
}

/// `ftPr_SpecialS_8013DC64`: initialise the roll state.
pub fn ftPr_SpecialS_8013DC64(gobj: G) {
    melee_core::ct!("ftPr_SpecialS_8013DC64");
    unsafe {
        let jobj = jobj_of(gobj);
        let fp = fpp(gobj);
        let da = da_of(fp);
        ext::HSD_JObjGetScale(jobj, &mut (*fp).u.pr.x2230);
        (*fp).gr_vel = 0.0;
        sn!(fp).x0 = da.x34;
        sn!(fp).x4 = -1;
        sn!(fp).x8 = -1;
        sn!(fp).xC = 0;
        sn!(fp).x14 = 0.0;
        sn!(fp).x34.y = 0.0;
        sn!(fp).facing_dir = 0.0;
        sn!(fp).x24 = 0;
        sn!(fp).x28 = 0;
        sn!(fp).x2C = da.xA0 as i32;
        sn!(fp).x30 = 0;
        (*fp).deal_dmg_cb = Some(cb_ftPr_SpecialS_8013D764);
        (*fp).x21F8 = Some(cb_ftPr_SpecialN_8014222C);
    }
}

/// `ftPr_SpecialS_8013DD54`: wrap the roll angle, play the roll sfx when it passes the turn point.
pub fn ftPr_SpecialS_8013DD54(gobj: G, arg1: bool) {
    melee_core::ct!("ftPr_SpecialS_8013DD54");
    unsafe {
        let fp = fpp(gobj);
        while sn!(fp).x14 < 0.0 {
            sn!(fp).x14 = (sn!(fp).x14 as f64 + M_PI * 2.0) as f32;
        }
        while sn!(fp).x14 as f64 > M_PI * 2.0 {
            sn!(fp).x14 = (sn!(fp).x14 as f64 - M_PI * 2.0) as f32;
        }
        if sn!(fp).x34.y == 0.0 {
            ftKb_PrPlayRollSFX(gobj);
        } else if sn!(fp).x34.x == (if arg1 { -1.0 } else { 1.0 }) {
            if sn!(fp).x14 < sn!(fp).x34.y {
                ftKb_PrPlayRollSFX(gobj);
            }
        } else if sn!(fp).x14 > sn!(fp).x34.y {
            ftKb_PrPlayRollSFX(gobj);
        }
        sn!(fp).x34.y = sn!(fp).x14;
    }
}

pub fn ftPr_SpecialN_Enter(gobj: G) {
    melee_core::ct!("ftPr_SpecialN_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        sn!(fp).x34.x = (*fp).facing_dir;
        let msid = if sn!(fp).x34.x == 1.0 { ftPr_MS_SpecialNStartR } else { ftPr_MS_SpecialNStartL };
        st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::Fighter_ClearCmdVars(&mut *fp);
        st::ftAnim_8006EBA4(gobj);
        ftPr_SpecialS_8013DC64(gobj);
        (*fp).x74_self_accel.y = 0.0;
        (*fp).self_vel.y = 0.0;
        sn!(fp).x1C = da.x44;
    }
}

pub fn ftPr_SpecialAirN_Enter(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirN_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        sn!(fp).x34.x = (*fp).facing_dir;
        let msid = if sn!(fp).x34.x == 1.0 { ftPr_MS_SpecialAirNStartR } else { ftPr_MS_SpecialAirNStartL };
        st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::Fighter_ClearCmdVars(&mut *fp);
        st::ftAnim_8006EBA4(gobj);
        ftPr_SpecialS_8013DC64(gobj);
        (*fp).x74_self_accel.y = da.x3C;
        sn!(fp).x1C = da.x54;
    }
}

// ---------------------------------------------------------------------------------------------
// Anim
// ---------------------------------------------------------------------------------------------

const MF_START: u32 = (1 << 1) | (1 << 4) | (1 << 18);
const MF_FULL: u32 = (1 << 1) | (1 << 4) | (1 << 7) | (1 << 9) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);
const MF_RELEASE: u32 = (1 << 1) | (1 << 4) | (1 << 7) | (1 << 12) | (1 << 18);
const MF_COLL: u32 = (1 << 1) | (1 << 4) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);
const MF_ROLL_COLL: u32 =
    (1 << 1) | (1 << 3) | (1 << 4) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);

unsafe fn start_anim(gobj: G, loop_ms: i32, ground: bool) {
    let fp = fpp(gobj);
    sn!(fp).facing_dir = 0.0;
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::Fighter_ChangeMotionState(gobj, loop_ms, MF_START, 0.0, 0.0, 0.0, core::ptr::null_mut());
        setup_purin_callbacks(gobj);
        (*fp).cur_anim_frame = 0.0;
        st::ftAnim_SetAnimRate(gobj, 0.0);
        if ground {
            let v = (*fp).facing_dir * 0.0001f32;
            (*fp).self_vel.x = v;
            (*fp).gr_vel = v;
            (*fp).x74_self_accel.x = 0.0;
            (*fp).xE4_ground_accel_1 = 0.0;
        } else {
            (*fp).self_vel.x = (*fp).facing_dir * 0.0001f32;
            (*fp).x74_self_accel.x = 0.0;
        }
        rot_y(fp, 0, M_PI_2);
    }
}

pub fn ftPr_SpecialNStart_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialNStart_Anim");
    unsafe { start_anim(gobj, ftPr_MS_SpecialNLoop as i32, true) }
}

pub fn ftPr_SpecialAirNStart_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNStart_Anim");
    unsafe { start_anim(gobj, ftPr_MS_SpecialAirNChargeLoop as i32, false) }
}

/// The charge step and spin: `x2C += xA8` (int += float), clamp at `xA4`; then the fused
/// `x14 += x34.x * (x2C * MTXDegToRad(xAC))`. `full_ms` < 0: the Full state (no state change).
unsafe fn charge_anim(gobj: G, full_ms: i32) {
    let fp = fpp(gobj);
    let da = da_of(fp);
    ftPr_SpecialS_8013DD54(gobj, false);
    sn!(fp).facing_dir = 0.0;
    sn!(fp).x2C = (sn!(fp).x2C as f32 + da.xA8) as i32;
    if sn!(fp).x2C as f32 >= da.xA4 {
        sn!(fp).x2C = da.xA4 as i32;
        if full_ms >= 0 {
            if sn!(fp).x30 == 0 {
                ext::ftCo_800BFFD0(fp, 5, 0);
            }
            sn!(fp).x30 = 1;
            st::Fighter_ChangeMotionState(gobj, full_ms, MF_FULL, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
        } else {
            sn!(fp).x30 = 1;
        }
    }
    let prod = sn!(fp).x2C as f32 * deg_to_rad(da.xAC);
    sn!(fp).x14 = fmadds(sn!(fp).x34.x, prod, sn!(fp).x14);
    ftPr_NormalizeAndSetRollAngle(gobj);
    rot_y(fp, FtPart_TopN as i32, M_PI_2);
}

pub fn ftPr_SpecialNLoop_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialNLoop_Anim");
    unsafe { charge_anim(gobj, ftPr_MS_SpecialNFull as i32) }
}

pub fn ftPr_SpecialNFull_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialNFull_Anim");
    unsafe { charge_anim(gobj, -1) }
}

pub fn ftPr_SpecialAirNChargeLoop_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNChargeLoop_Anim");
    unsafe { charge_anim(gobj, ftPr_MS_SpecialAirNChargeFull as i32) }
}

pub fn ftPr_SpecialAirNChargeFull_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNChargeFull_Anim");
    unsafe { charge_anim(gobj, -1) }
}

/// The wrap-around end test of the two Release Anims. `air_direct`: the air variant sets the TopN
/// rotation through the jobj inline (`ftPr_JObjSetRotationY`) in its `x0 <= 0` arms.
unsafe fn release_end(gobj: G, fp: *mut Fighter, old_angle: f32, delta: f32, air: bool) {
    ftPr_NormalizeAndSetRollAngle(gobj);
    sn!(fp).x0 -= 1;
    let set_rot = |fp: *mut Fighter| {
        if air {
            ext::HSD_JObjSetRotationY((*(*fp).parts.add(FtPart_TopN as usize)).joint as *mut _, M_PI_2 as f32);
        } else {
            rot_y(fp, FtPart_TopN as i32, M_PI_2);
        }
    };
    if sn!(fp).x0 <= 0 {
        let x14 = sn!(fp).x14 as f64;
        if M_PI_2 < x14 && x14 < M_PI_2 * 3.0 {
            if delta > 0.0 {
                if x14 > M_PI && (old_angle as f64) < M_PI {
                    sn!(fp).x0 = 0;
                    ftPr_SpecialS_8013DA24(gobj, air, 0x40012, 0.0);
                    return;
                }
                set_rot(fp);
                return;
            }
            if x14 < M_PI && old_angle as f64 > M_PI {
                sn!(fp).x0 = 0;
                ftPr_SpecialS_8013DA24(gobj, air, 0x40012, 0.0);
                return;
            }
            set_rot(fp);
            return;
        }
        set_rot(fp);
        return;
    }
    rot_y(fp, FtPart_TopN as i32, M_PI_2);
}

pub fn ftPr_SpecialNRelease_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialNRelease_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftPr_SpecialS_8013DD54(gobj, false);
        sn!(fp).facing_dir = 0.0;
        scale_anim_step(gobj, &mut scale);
        hit_capsule_toggle(gobj);
        ftPr_SpecialS_8013D8E4(gobj);
        let old_angle = sn!(fp).x14;
        let delta = deg_to_rad(sn!(fp).x2C as f32) * ((0.2f64 * da.x98 as f64 * sn!(fp).x34.x as f64) as f32);
        sn!(fp).x14 = old_angle + delta;
        release_end(gobj, fp, old_angle, delta, false);
    }
}

pub fn ftPr_SpecialAirNChargeRelease_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNChargeRelease_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftPr_SpecialS_8013DD54(gobj, false);
        sn!(fp).facing_dir = 0.0;
        // the inline reads `scale_base[frame + 4]` off 803D05C8: that is 803D05D8 (adjacent data)
        scale_anim_step(gobj, &mut scale);
        hit_capsule_toggle(gobj);
        ftPr_SpecialS_8013D8E4(gobj);
        let prev_angle = sn!(fp).x14;
        let delta = ((0.2f64 * da.x98 as f64 * sn!(fp).x34.x as f64) as f32) * (0.017453292f32 * sn!(fp).x2C as f32 * da.xBC);
        sn!(fp).x14 += delta;
        release_end(gobj, fp, prev_angle, delta, true);
    }
}

pub fn ftPr_SpecialNTurn_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialNTurn_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftPr_SpecialS_8013DD54(gobj, true);
        scale_anim_step(gobj, &mut scale);
        // f64 fused site: x14 = fma(0.2 * x6C, -x34.x, x14)
        let a = 0.2f64 * da.x6C as f64;
        let b = (-sn!(fp).x34.x) as f64;
        sn!(fp).x14 = a.mul_add(b, sn!(fp).x14 as f64) as f32;
        ftPr_NormalizeAndSetRollAngle(gobj);
        sn!(fp).x0 -= 1;
        if sn!(fp).x0 <= 0 {
            sn!(fp).x0 = 0;
            sn!(fp).x34.x = -sn!(fp).x34.x;
            ftPr_SpecialS_8013DA24(gobj, false, 0x40012, 0.0);
            return;
        }
        if melee_runtime::cabi::srem(sn!(fp).x24, da.x70) == 0 {
            let mut angle = atan2f(-(*fp).coll_data.floor.normal.x, (*fp).coll_data.floor.normal.y);
            ext::efSync_Spawn(
                0x3FF,
                gobj,
                &[
                    &mut (*fp).cur_pos as *mut Vec3 as *mut core::ffi::c_void,
                    &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void,
                    &mut angle as *mut f32 as *mut core::ffi::c_void,
                ],
            );
        }
        sn!(fp).x24 += 1;
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
    }
}

pub fn ftPr_SpecialNEnd_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        sn!(fp).facing_dir = 0.0;
        scale_anim_step(gobj, &mut scale);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftPr_SpecialS_8013D658(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

/// `x14 += xBC * (f32)(0.2 * x6C * -x34.x)`: one fused f32 site.
unsafe fn hit_spin(fp: *mut Fighter, da: &ftPurinAttributes) {
    let t = (0.2f64 * da.x6C as f64 * (-sn!(fp).x34.x) as f64) as f32;
    sn!(fp).x14 = fmadds(da.xBC, t, sn!(fp).x14);
}

pub fn ftPr_SpecialAirNStartTurn_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNStartTurn_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftPr_SpecialS_8013DD54(gobj, true);
        scale_anim_step(gobj, &mut scale);
        hit_spin(fp, da);
        ftPr_NormalizeAndSetRollAngle(gobj);
        sn!(fp).x0 -= 1;
        if sn!(fp).x0 <= 0 {
            sn!(fp).x0 = 0;
            sn!(fp).x34.x = -sn!(fp).x34.x;
            ftPr_SpecialS_8013DA24(gobj, false, 0x40012, 0.0);
            return;
        }
        sn!(fp).x24 += 1;
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
    }
}

pub fn ftPr_SpecialAirNEnd_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        sn!(fp).facing_dir = 0.0;
        scale_anim_step(gobj, &mut scale);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftPr_SpecialS_8013D658(gobj);
            if da.xD8 == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCo_80096900(gobj, 1, 0, 1, 1.0, da.xD8);
            }
        }
    }
}

pub fn ftPr_SpecialNHit_Anim(gobj: G) {
    melee_core::ct!("ftPr_SpecialNHit_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        scale_anim_step(gobj, &mut scale);
        hit_spin(fp, da);
        ftPr_NormalizeAndSetRollAngle(gobj);
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
    }
}

// ---------------------------------------------------------------------------------------------
// IASA
// ---------------------------------------------------------------------------------------------

pub fn ftPr_SpecialNStart_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialNStart_IASA"); }
pub fn ftPr_SpecialNTurn_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialNTurn_IASA"); }
pub fn ftPr_SpecialNEnd_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialNEnd_IASA"); }
pub fn ftPr_SpecialAirNStart_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialAirNStart_IASA"); }
pub fn ftPr_SpecialAirNChargeRelease_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialAirNChargeRelease_IASA"); }
pub fn ftPr_SpecialAirNStartTurn_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialAirNStartTurn_IASA"); }
pub fn ftPr_SpecialAirNEnd_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialAirNEnd_IASA"); }
pub fn ftPr_SpecialNHit_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialNHit_IASA"); }

/// B released while charging: go to the Release state (`air`: AirNChargeRelease, speed into self_vel.x).
unsafe fn release_iasa(gobj: G, air: bool) {
    let fp = fpp(gobj);
    let da = da_of(fp);
    if (*fp).input.held_buttons[0] & 0x200 == 0 {
        let ms = if air { ftPr_MS_SpecialAirNChargeRelease } else { ftPr_MS_SpecialNRelease };
        st::Fighter_ChangeMotionState(gobj, ms as i32, MF_RELEASE, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
        setup_purin_callbacks(gobj);
        st::ftAnim_8006EBA4(gobj);
        let v = sn!(fp).x34.x * (da.xC0 * (sn!(fp).x2C as f32 - da.xA0));
        if air {
            (*fp).self_vel.x = v;
        } else {
            (*fp).gr_vel = v;
        }
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
        ftPr_NormalizeAndSetRollAngle(gobj);
        fh::play_sfx(&mut *fp, 250073, 127, 64);
    }
}

pub fn ftPr_SpecialNLoop_IASA(gobj: G) {
    melee_core::ct!("ftPr_SpecialNLoop_IASA");
    unsafe { release_iasa(gobj, false) }
}

pub fn ftPr_SpecialNFull_IASA(gobj: G) {
    melee_core::ct!("ftPr_SpecialNFull_IASA");
    unsafe { release_iasa(gobj, false) }
}

pub fn ftPr_SpecialAirNChargeLoop_IASA(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNChargeLoop_IASA");
    unsafe { release_iasa(gobj, true) }
}

pub fn ftPr_SpecialAirNChargeFull_IASA(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNChargeFull_IASA");
    unsafe { release_iasa(gobj, true) }
}

pub fn ftPr_SpecialNRelease_IASA(gobj: G) {
    melee_core::ct!("ftPr_SpecialNRelease_IASA");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        if abs_f((*fp).input.lstick[0].x) > da.x68 {
            let dir = signf((*fp).input.lstick[0].x);
            if sn!(fp).x34.x != dir {
                ext::ftColl_8007AFF8(gobj);
                sn!(fp).facing_dir = dir;
                sn!(fp).x34.x = -dir;
                st::Fighter_ChangeMotionState(gobj, ftPr_MS_SpecialNTurn as i32, MF_FULL, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
                setup_purin_callbacks(gobj);
                sn!(fp).x10 = (*fp).gr_vel;
                sn!(fp).x1C = -0.05f32 * (*fp).gr_vel;
                sn!(fp).x24 = 0;
                ftPr_NormalizeAndSetRollAngle(gobj);
                fh::play_sfx(&mut *fp, 250073, 127, 64);
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Phys
// ---------------------------------------------------------------------------------------------

pub fn ftPr_SpecialNStart_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialNStart_Phys");
    unsafe {
        let fp = fpp(gobj);
        (*fp).xE4_ground_accel_1 = 0.0;
        (*fp).gr_vel = 0.0;
        (*fp).self_vel.x = (*fp).facing_dir * 0.0001f32;
        (*fp).self_vel.y = 0.0;
        (*fp).x74_self_accel.y = 0.0;
        (*fp).x74_self_accel.x = 0.0;
    }
}

pub fn ftPr_SpecialNLoop_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialNLoop_Phys");
    ftPr_SpecialNStart_Phys(gobj)
}

pub fn ftPr_SpecialNFull_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialNFull_Phys");
    ftPr_SpecialNStart_Phys(gobj)
}

/// `ABS(v) > max`: clamp to `max` with v's sign.
#[inline(always)]
fn clamp_mag(v: f32, max: f32) -> f32 {
    if abs_f(v) > max {
        if v < 0.0 {
            -max
        } else {
            max
        }
    } else {
        v
    }
}

pub fn ftPr_SpecialNRelease_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialNRelease_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let slope = (*fp).coll_data.floor.normal.x;
        let base_vel = sn!(fp).x34.x * (da.xC0 * (sn!(fp).x2C as f32 - da.xB8));
        let influence = da.xC8 * (base_vel * abs_f(slope));
        // both arms of the C's `x1C > 0` test are identical
        if (*fp).coll_data.floor.normal.x > 0.0 {
            (*fp).gr_vel = base_vel + influence;
        } else {
            (*fp).gr_vel = base_vel - influence;
        }
        (*fp).gr_vel = clamp_mag((*fp).gr_vel, da.x4C);
        (*fp).gr_vel = clamp_mag((*fp).gr_vel, da.x50);
        sn!(fp).x18 = abs_f((*fp).gr_vel);
        (*fp).x74_self_accel.y = 0.0;
        (*fp).self_vel.y = 0.0;
        ext::ftCommon_SetSelfMovementFromGroundedMovement(fp);
        sn!(fp).x2C = (sn!(fp).x2C as f32 - da.xB4) as i32;
        if (sn!(fp).x2C as f32) < da.xB8 {
            ftPr_SpecialS_8013DA24(gobj, false, 0x40012, 0.0);
        }
    }
}

pub fn ftPr_SpecialNTurn_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialNTurn_Phys");
    const MF1: u32 = (1 << 4) | (1 << 18) | (1 << 25);
    const MF2: u32 = (1 << 1) | (1 << 4) | (1 << 18) | (1 << 25);
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let scale = da.xC4 * ext::mpLib_800569EC((*fp).coll_data.floor.flags);
        let slope = abs_f((*fp).coll_data.floor.normal.x);
        let x1c = sn!(fp).x1C;
        let influence = da.xC8 * (x1c * slope);
        (*fp).gr_vel = turn_gr_vel((*fp).gr_vel, scale, x1c, influence, (*fp).coll_data.floor.normal.x > 0.0);
        (*fp).x74_self_accel.y = 0.0;
        (*fp).self_vel.y = 0.0;
        ext::ftCommon_SetSelfMovementFromGroundedMovement(fp);
        let leave = if sn!(fp).x10 > 0.0 {
            if (*fp).gr_vel < 0.0 && abs_f((*fp).gr_vel) >= abs_f(sn!(fp).x10 * da.xD0) { Some(MF1) } else { None }
        } else if (*fp).gr_vel > 0.0 && abs_f((*fp).gr_vel) >= abs_f(sn!(fp).x10 * da.xD0) {
            Some(MF2)
        } else {
            None
        };
        if let Some(mf) = leave {
            (*fp).x21EC = Some(cb_ftPr_SpecialS_8013D8B0);
            st::Fighter_ChangeMotionState(gobj, ftPr_MS_SpecialNRelease as i32, mf, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            setup_purin_callbacks(gobj);
            ftPr_SetFacingDir(fp);
        }
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
        ftPr_NormalizeAndSetRollAngle(gobj);
    }
}

pub fn ftPr_SpecialNEnd_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialNEnd_Phys");
    unsafe {
        let fp = &mut *fpp(gobj);
        let f = fp.co_attrs.ground_friction;
        co::calc_ground_accel_deaccel(fp, f);
        co::set_self_movement_from_grounded_movement(fp);
    }
}

/// `ftCommon_Fall(fp, da->x3C, da->x40)`.
unsafe fn fall_da(fp: *mut Fighter) {
    let da = da_of(fp);
    ext::ftCommon_Fall(fp, da.x3C, da.x40);
}

pub fn ftPr_SpecialAirNStart_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNStart_Phys");
    unsafe { fall_da(fpp(gobj)) }
}

pub fn ftPr_SpecialAirNChargeLoop_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNChargeLoop_Phys");
    unsafe { fall_da(fpp(gobj)) }
}

pub fn ftPr_SpecialAirNChargeFull_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNChargeFull_Phys");
    unsafe { fall_da(fpp(gobj)) }
}

pub fn ftPr_SpecialAirNEnd_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNEnd_Phys");
    unsafe { fall_da(fpp(gobj)) }
}

pub fn ftPr_SpecialAirNChargeRelease_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNChargeRelease_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        (*fp).self_vel.x = sn!(fp).x34.x * (da.xC0 * (sn!(fp).x2C as f32 - da.xB8));
        (*fp).self_vel.x = air_decel_floor((*fp).self_vel.x, da.x58, da.x5C);
        sn!(fp).x18 = abs_f((*fp).self_vel.x);
        fall_da(fp);
        sn!(fp).x2C = (sn!(fp).x2C as f32 - da.xB4) as i32;
        if (sn!(fp).x2C as f32) < da.xB8 {
            ftPr_SpecialS_8013DA24(gobj, true, 0x40012, 0.0);
        }
    }
}

pub fn ftPr_SpecialAirNStartTurn_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNStartTurn_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        (*fp).self_vel.x = air_decel_floor((*fp).self_vel.x, da.x58, da.x5C);
        fall_da(fp);
        rot_y(fp, FtPart_TopN as i32, M_PI_2);
        ftPr_NormalizeAndSetRollAngle(gobj);
    }
}

pub fn ftPr_SpecialNHit_Phys(gobj: G) {
    melee_core::ct!("ftPr_SpecialNHit_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        if (*fp).self_vel.y <= -da.x40 {
            co::calc_self_accel_drift(&mut *fp);
        }
        fall_da(fp);
    }
}

// ---------------------------------------------------------------------------------------------
// Coll
// ---------------------------------------------------------------------------------------------

pub fn ftPr_SpecialNStart_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialNStart_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            ext::ftCommon_8007D5D4(fp);
            let msid = if sn!(fp).x34.x == 1.0 { ftPr_MS_SpecialAirNStartR } else { ftPr_MS_SpecialAirNStartL };
            st::Fighter_ChangeMotionState(gobj, msid as i32, MF_COLL, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            setup_purin_callbacks(gobj);
        }
    }
}

/// Charge state left the ground / landed: same charge state on the other side.
unsafe fn charge_coll(gobj: G, ms: i32, to_air: bool) {
    let fp = fpp(gobj);
    if to_air {
        ext::ftCommon_8007D5D4(fp);
    } else {
        ext::ftCommon_8007D7FC(fp);
    }
    st::Fighter_ChangeMotionState(gobj, ms, MF_RELEASE, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
    setup_purin_callbacks(gobj);
    rot_y(fp, FtPart_TopN as i32, M_PI_2);
    ftPr_NormalizeAndSetRollAngle(gobj);
}

pub fn ftPr_SpecialNLoop_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialNLoop_Coll");
    unsafe {
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            charge_coll(gobj, ftPr_MS_SpecialAirNChargeLoop as i32, true);
        }
    }
}

pub fn ftPr_SpecialNFull_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialNFull_Coll");
    unsafe {
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            charge_coll(gobj, ftPr_MS_SpecialAirNChargeFull as i32, true);
        }
    }
}

/// The wall test of the two Release Colls (`wallBounceEffect` when hit).
unsafe fn wall_check(gobj: G, fp: *mut Fighter) -> bool {
    let (mask, dir) = if sn!(fp).x34.x == 1.0 { (0x3F, 1.0f32) } else { (0xFC0, -1.0f32) };
    let wall_hit = ((*fp).coll_data.env_flags & mask) != 0;
    if wall_hit {
        let (mut pos, mut angle) = (Vec3 { x: 0.0, y: 0.0, z: 0.0 }, 0.0f32);
        ftKb_PrWallBounceEffect(gobj, fp, dir, &mut pos, &mut angle);
    }
    wall_hit
}

#[inline(always)]
fn ecb_box() -> *mut ftCollisionBox {
    &raw const ftPr_Init_803D0610 as *mut ftCollisionBox
}

pub fn ftPr_SpecialNRelease_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialNRelease_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let coll = ext::ft_80082888(gobj, ecb_box());
        if wall_check(gobj, fp) {
            sn!(fp).x2C = (sn!(fp).x2C as f32 * da.xD4) as i32;
            if sn!(fp).x2C < 0 {
                sn!(fp).x2C = 0;
            }
            sn!(fp).x18 *= da.xD4;
            (*fp).gr_vel = -(*fp).gr_vel * da.xD4;
            (*fp).self_vel.x = -(*fp).self_vel.x * da.xD4;
            sn!(fp).x34.x = signf((*fp).gr_vel);
            rot_y(fp, FtPart_TopN as i32, M_PI_2);
        }
        if !coll {
            ext::ftCommon_8007D5D4(fp);
            st::Fighter_ChangeMotionState(gobj, ftPr_MS_SpecialAirNChargeRelease as i32, MF_ROLL_COLL, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            setup_purin_callbacks(gobj);
            ftPr_NormalizeAndSetRollAngle(gobj);
            rot_y(fp, FtPart_TopN as i32, M_PI_2);
            sn!(fp).x1C = da.x54;
        }
    }
}

pub fn ftPr_SpecialNTurn_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialNTurn_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let coll = if abs_f((*fp).gr_vel) > da.x74 { ext::ft_80082888(gobj, ecb_box()) } else { ext::ft_80082978(gobj, ecb_box()) };
        if !coll {
            ext::ftCommon_8007D5D4(fp);
            st::Fighter_ChangeMotionState(gobj, ftPr_MS_SpecialAirNStartTurn as i32, MF_COLL, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            setup_purin_callbacks(gobj);
        }
    }
}

pub fn ftPr_SpecialNEnd_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialNEnd_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            ext::ftCommon_8007D5D4(fp);
            ftPr_SpecialS_8013DA24(gobj, true, MF_COLL, (*fp).cur_anim_frame);
        }
    }
}

pub fn ftPr_SpecialAirNStart_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNStart_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            ext::ftCommon_8007D7FC(fp);
            let msid = if sn!(fp).x34.x == 1.0 { ftPr_MS_SpecialNStartR } else { ftPr_MS_SpecialNStartL };
            st::Fighter_ChangeMotionState(gobj, msid as i32, MF_COLL, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            setup_purin_callbacks(gobj);
        }
    }
}

pub fn ftPr_SpecialAirNChargeLoop_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNChargeLoop_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            charge_coll(gobj, ftPr_MS_SpecialNLoop as i32, false);
        }
    }
}

pub fn ftPr_SpecialAirNChargeFull_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNChargeFull_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            charge_coll(gobj, ftPr_MS_SpecialNFull as i32, false);
        }
    }
}

pub fn ftPr_SpecialAirNChargeRelease_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNChargeRelease_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        let coll = if sn!(fp).x34.x == 1.0 { ext::ft_8008239C(gobj, 1, ecb_box()) } else { ext::ft_8008239C(gobj, -1, ecb_box()) };
        if wall_check(gobj, fp) {
            sn!(fp).x2C = (sn!(fp).x2C as f32 * da.xD4) as i32;
            if sn!(fp).x2C < 0 {
                sn!(fp).x2C = 0;
            }
            sn!(fp).x18 *= da.xD4;
            (*fp).self_vel.x = -(*fp).self_vel.x * da.xD4;
            sn!(fp).x34.x = -sn!(fp).x34.x;
        }
        if coll {
            let vel_y = abs_f((*fp).self_vel.y * da.x78);
            (*fp).self_vel.y = vel_y;
            if vel_y < da.x7C {
                ext::ftCommon_8007D7FC(fp);
                st::Fighter_ChangeMotionState(gobj, ftPr_MS_SpecialNRelease as i32, MF_ROLL_COLL, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
                setup_purin_callbacks(gobj);
                let vel = sn!(fp).x18 * sn!(fp).x34.x;
                (*fp).gr_vel = vel;
                (*fp).self_vel.x = vel;
                (*fp).self_vel.z = 0.0;
                (*fp).self_vel.y = 0.0;
                ftPr_NormalizeAndSetRollAngle(gobj);
                rot_y(fp, FtPart_TopN as i32, M_PI_2);
                let mut land_angle = atan2f(-(*fp).coll_data.floor.normal.x, (*fp).coll_data.floor.normal.y);
                ext::efSync_Spawn(
                    0x3FF,
                    gobj,
                    &[
                        &mut (*fp).cur_pos as *mut Vec3 as *mut core::ffi::c_void,
                        &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void,
                        &mut land_angle as *mut f32 as *mut core::ffi::c_void,
                    ],
                );
                sn!(fp).x1C = da.x44;
            } else {
                if abs_f((*fp).input.lstick[0].x) > da.x68 {
                    sn!(fp).x34.x = signf((*fp).input.lstick[0].x);
                    let v = sn!(fp).x18 * sn!(fp).x34.x;
                    (*fp).gr_vel = v;
                    (*fp).self_vel.x = v;
                    rot_y(fp, FtPart_TopN as i32, M_PI_2);
                }
                ftPr_NormalizeAndSetRollAngle(gobj);
            }
            sn!(fp).x8 = 0;
            return;
        }
        if st::ftCliffCommon_80081298(gobj) != 0 {
            ftPr_SpecialS_8013D658(gobj);
            ext::ftCliffCommon_80081370(gobj);
        }
    }
}

pub fn ftPr_SpecialAirNStartTurn_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNStartTurn_Coll");
    unsafe {
        let fp = fpp(gobj);
        if ext::ft_800824A0(gobj, ecb_box()) {
            ext::ftCommon_8007D7FC(fp);
            st::Fighter_ChangeMotionState(gobj, ftPr_MS_SpecialNTurn as i32, MF_COLL, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
            setup_purin_callbacks(gobj);
            (*fp).gr_vel = (*fp).self_vel.x;
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
        }
    }
}

pub fn ftPr_SpecialAirNEnd_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirNEnd_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            ext::ftCommon_8007D7FC(fp);
            ftPr_SpecialS_8013DA24(gobj, false, MF_COLL, (*fp).cur_anim_frame);
        }
    }
}

pub fn ftPr_SpecialNHit_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialNHit_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = da_of(fp);
        if ext::ft_800824A0(gobj, ecb_box()) {
            ftPr_SpecialS_8013D658(gobj);
            ext::ftCommon_8007D7FC(fp);
            if da.xD8 == 0.0 {
                st::ft_8008A2BC(gobj);
            } else {
                st::ftCo_LandingFallSpecial_Enter(gobj, 0, da.xD8);
            }
        }
    }
}

/// `ftPr_SpecialN_8014222C` (`x21F8`): mirror the roll state on a turn.
pub fn ftPr_SpecialN_8014222C(gobj: G) {
    melee_core::ct!("ftPr_SpecialN_8014222C");
    unsafe {
        let fp = fpp(gobj);
        rot_y(fp, FtPart_TopN as i32, M_PI_2 * (*fp).facing_dir as f64);
        ftPr_MirrorSpecialN(fp);
    }
}
