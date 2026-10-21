#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_DownBound.c (knockdown bounce, DownWait).
//! Checked against the C by tests/tu_ftCo_DownBound.rs.

use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    macro_rules! ext_fns {
        ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
            pub mod hook {
                #[allow(unused_imports)] use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
            } )*
        };
    }
    ext_fns! {
        ftParts_GetBoneIndex(fp: &mut Fighter, part: i32) -> i32 = |f: &mut Fighter, p| crate::hooks::ftparts_get_bone_index(f, p);
        HSD_JObjSetupMatrixSub(j: *mut HSD_JObj) = |_| todo!("TODO(port) HSD_JObjSetupMatrixSub (baselib/jobj.c)");
        atan2f(y: f32, x: f32) -> f32 = melee_runtime::lbtrigf::atan2f;
        // `efAsync_Spawn(gobj, queue, kind, gfx_id, jobj, float* param)` (variadic in C).
        efAsync_Spawn(g: G, queue: *mut core::ffi::c_void, kind: u32, id: u32, jobj: *mut HSD_JObj, param: *mut f32) = |_, _, _, _, _, _| todo!("TODO(port) efAsync_Spawn (ef/efasync.c)");
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |_, _, _, _| todo!("TODO(port) ft_PlaySFX (ft_0881.c)");
        ft_80084C74(g: G, sfx: &mut [i32; 4], a2: &mut i32, a3: &mut i32) -> bool = |_, _, _, _| todo!("TODO(port) ft_80084C74 (ft_081B.c)");
        ftCo_8009F834(g: G, id: i32, part: i32, a: i32, b: i32, v1: &mut Vec3, v0: &mut Vec3, f: f32) = |_, _, _, _, _, _, _, _| todo!("TODO(port) ftCo_8009F834 (ftCo_09F7.c)");
        Camera_RequestQuake(kind: i32, pos: *mut Vec) = |_, _| todo!("TODO(port) Camera_RequestQuake (cm/camera.c)");
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCommon_SetGroundedKnockbackIfLanded(fp: &mut Fighter) = crate::ftcommon::set_grounded_knockback_if_landed;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftSb_Init_8014FBA4(g: G) = |_| todo!("TODO(port) ftSb_Init_8014FBA4 (ftSandbag_Init.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_80098400(g: G) -> bool = crate::tu::ftCo_Down::ftCo_80098400;
        ftCo_Down_CheckInput(g: G) -> bool = crate::tu::ftCo_Down::ftCo_Down_CheckInput;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80082708(g: G) -> i32 = |_| todo!("TODO(port) ft_80082708 (ft_081B.c)");
        ftCo_Fall_Enter(g: G) = crate::hooks::ftco_fall_enter;
        ftCo_800C7CA0(g: G) -> bool = |_| todo!("TODO(port) ftCo_800C7CA0 (ftCo_800C7CA0.c)");
        ftAnim_8006EBA4(g: G) = |_| todo!("TODO(port) ftAnim_8006EBA4 (ftanim.c)");
        ftCo_80098160(g: G, msid: i32) = |_, _| todo!("TODO(port) ftCo_80098160 (ftCo_DownStand.c)");
        ftCo_800984D4(g: G) -> bool = crate::tu::ftCo_DownAttack::ftCo_800984D4;
        ftCo_800980BC(g: G) -> bool = |_| todo!("TODO(port) ftCo_800980BC (ftCo_DownSpot.c)");
        ft_80083F88(g: G) = |_| todo!("TODO(port) ft_80083F88 (ft_081B.c)");
    }
}

/// `ftCo_DownBound_SfxIds`.
pub static ftCo_DownBound_SfxIds: [i32; 4] = [9, 10, 11, 12];

const MF_BOUND: u32 = (Ft_MF_SkipNametagVis | Ft_MF_KeepColAnimPartHitStatus) as u32;
const MF_WAIT: u32 = (Ft_MF_SkipModel | Ft_MF_SkipMatAnim | Ft_MF_SkipNametagVis | Ft_MF_KeepColAnimPartHitStatus) as u32;

/// static inline `HSD_JObjSetupMatrix` (baselib/jobj.h).
fn HSD_JObjSetupMatrix(j: *mut HSD_JObj) {
    if j.is_null() || !melee_types::inline::HSD_JObjMtxIsDirty(unsafe { &*j }) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(j);
}

fn hip_joint(g: G) -> *mut HSD_JObj {
    let idx = ext::ftParts_GetBoneIndex(fp(g), FtPart_HipN as i32);
    unsafe { (*fp(g).parts.offset(idx as isize)).joint }
}

/// `ftCo_80097570`: true = the hip faces up.
pub fn ftCo_80097570(g: G) -> bool {
    let jobj = hip_joint(g);
    HSD_JObjSetupMatrix(jobj);
    let m = unsafe { &(*jobj).mtx };
    let f = if fp(g).x2226_b0() != 0 { m[1][2] } else { m[1][1] };
    f > 0.0
}

pub fn ftCo_80097630(f: &mut Fighter, sfx_ids: &[i32; 4], threshold: f32) {
    let c = cd();
    // The third arm repeats x1F4 in C, so sfx_ids[2] is never chosen.
    let id = if threshold >= c.x1F0 {
        sfx_ids[0]
    } else if threshold >= c.x1F4 {
        sfx_ids[1]
    } else if threshold >= c.x1F4 {
        sfx_ids[2]
    } else {
        sfx_ids[3]
    };
    ext::ft_PlaySFX(f, id, 127, 64);
}

/// static inline `ftCo_800976A4_inline`.
pub fn ftCo_800976A4_inline(f: &Fighter, dist: &mut f32) {
    *dist *= f.co_attrs.weight;
}

/// static `ftCo_800976A4`: landing effect, sounds and quake.
pub fn ftCo_800976A4(g: G) {
    let f = fp(g);
    let vel_x = f.self_vel.x + f.x8c_kb_vel.x;
    let vel_y = f.self_vel.y + f.x8c_kb_vel.y;
    // `SQ(vel_x) + SQ(vel_y)`: one fmuladd.
    let mut dist = melee_runtime::gekko::sqrtf(fmadds(vel_x, vel_x, vel_y * vel_y));
    ftCo_800976A4_inline(f, &mut dist);
    let mut ef_id = 1031;
    let mut sfx_ids = [0i32; 4];
    let mut arg2 = 1;
    let mut arg3 = 0;
    if ext::ft_80084C74(g, &mut sfx_ids, &mut arg2, &mut arg3) {
        if sfx_ids[0] != -1 {
            ftCo_80097630(fp(g), &sfx_ids, dist);
        }
        if arg3 != -1 {
            ef_id = arg3;
        }
    }
    let mut vec1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut vec0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::ftCo_8009F834(g, ef_id, 0, 0, 0, &mut vec1, &mut vec0, 0.0);
    if arg2 != 0 {
        ftCo_80097630(fp(g), &ftCo_DownBound_SfxIds, dist);
    }
    let pos: *mut Vec = &mut fp(g).cur_pos;
    ext::Camera_RequestQuake(QuakeKind_Large as i32, pos);
    ext::ftCommon_8007EBAC(fp(g), 9, 0);
}

/// static inline `ftCo_800978D4_inline`.
pub fn ftCo_800978D4_inline(g: G, param: &mut f32) {
    let f = fp(g);
    *param = ext::atan2f(-f.coll_data.floor.normal.x, f.coll_data.floor.normal.y);
    spawn_406(g, param);
    ftCo_800976A4(g);
}

fn spawn_406(g: G, param: &mut f32) {
    let f = fp(g);
    let q: *mut *mut core::ffi::c_void = &mut f.x60C;
    let top = unsafe { (*f.parts.offset(FtPart_TopN as isize)).joint };
    ext::efAsync_Spawn(g, q as *mut core::ffi::c_void, 4, 0x406, top, param);
}

pub fn ftCo_800978D4(g: G) {
    let mut param = 0.0f32;
    ftCo_800978D4_inline(g, &mut param);
}

/// static `ftCo_8009794C`.
pub fn ftCo_8009794C(g: G) {
    if fp(g).ground_or_air == GA_Air {
        ext::ftCommon_8007D7FC(fp(g));
    }
    let mut b = ftCo_80097570(g);
    if fp(g).x2226_b1() != 0 {
        b = !b;
    }
    let msid = if b { ftCo_MS_DownBoundU } else { ftCo_MS_DownBoundD };
    ext::Fighter_ChangeMotionState(g, msid as i32, MF_BOUND, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let mut param = 0.0f32;
    ftCo_800978D4_inline(g, &mut param);
    let f = fp(g);
    f.x67C = 255;
    f.x67D = 255;
    ext::ftCommon_8007E2F4(fp(g), 511);
    ext::ftCommon_SetGroundedKnockbackIfLanded(fp(g));
}

#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// static `ftCo_80097AF4`: the Sandbag's bounce.
pub fn ftCo_80097AF4(g: G) {
    let jobj = hip_joint(g);
    HSD_JObjSetupMatrix(jobj);
    let m = unsafe { &(*jobj).mtx };
    let (rot0, rot1) = if fp(g).x2226_b0() != 0 { (m[0][2], m[1][2]) } else { (m[0][1], m[1][1]) };
    if abs_c(rot0) < abs_c(rot1) {
        if fp(g).ground_or_air == GA_Air {
            ext::ftCommon_8007D7FC(fp(g));
        }
        let mut b = ftCo_80097570(g);
        if fp(g).x2226_b1() != 0 {
            b = !b;
        }
        let msid = if b { ftCo_MS_DownBoundU } else { ftCo_MS_DownBoundD };
        ext::Fighter_ChangeMotionState(g, msid as i32, MF_BOUND, 0.0, 1.0, 0.0, core::ptr::null_mut());
        let fpa = fp(g);
        let mut param_a = ext::atan2f(-fpa.coll_data.floor.normal.x, fpa.coll_data.floor.normal.y);
        spawn_406(g, &mut param_a);
        ftCo_800976A4(g);
        let f = fp(g);
        f.x67C = 255;
        f.x67D = 255;
        ext::ftCommon_8007E2F4(fp(g), 511);
        ext::ftCommon_SetGroundedKnockbackIfLanded(fp(g));
    } else {
        let mut param_e = 0.0f32;
        ftCo_800978D4_inline(g, &mut param_e);
        let pos: *mut Vec = &mut fp(g).cur_pos;
        ext::Camera_RequestQuake(QuakeKind_Large as i32, pos);
        if fp(g).facing_dir * rot0 > 0.0 {
            ext::ft_8008A2BC(g);
        } else {
            ext::ftSb_Init_8014FBA4(g);
        }
        ext::ftCommon_SetGroundedKnockbackIfLanded(fp(g));
    }
}

/// static `inlineA1`.
pub fn inlineA1(g: G) {
    if fp(g).is_sandbag() != 0 {
        ftCo_80097AF4(g);
    } else {
        ftCo_8009794C(g);
    }
}

pub fn ftCo_80097D40(g: G) {
    inlineA1(g);
    unsafe { fp(g).mv.co.downspot.x4 = 0 };
}

pub fn ftCo_80097D88(g: G) {
    inlineA1(g);
}

pub fn ftCo_DownBound_Anim(g: G) {
    if ext::ftAnim_IsFramesRemaining(g) {
        return;
    }
    if ext::ftCo_80098400(g) {
        return;
    }
    if ext::ftCo_Down_CheckInput(g) {
        return;
    }
    ftCo_80097E8C(g);
}

pub fn ftCo_DownBound_IASA(_g: G) {}

pub fn ftCo_DownBound_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_DownBound_Coll(g: G) {
    if ext::ft_80082708(g) == GA_Ground as i32 {
        ext::ftCo_Fall_Enter(g);
    } else {
        ext::ftCo_800C7CA0(g);
    }
}

pub fn ftCo_80097E8C(g: G) {
    if fp(g).ground_or_air == GA_Air {
        ext::ftCommon_8007D7FC(fp(g));
    }
    let f = fp(g);
    let msid = if f.motion_id == ftCo_MS_DownBoundU as _ || f.motion_id == ftCo_MS_DownSpotU as _ {
        ftCo_MS_DownWaitU
    } else {
        ftCo_MS_DownWaitD
    };
    unsafe { f.mv.co.downwait.x0 = cd().x424 };
    ext::Fighter_ChangeMotionState(g, msid as i32, MF_WAIT, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2F4(fp(g), 1);
}

pub fn ftCo_80097F38(g: G) {
    let msid = if fp(g).motion_id == ftCo_MS_DownDamageU as _ { ftCo_MS_DownWaitU } else { ftCo_MS_DownWaitD };
    ext::Fighter_ChangeMotionState(g, msid as i32, MF_WAIT, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2F4(fp(g), 1);
}

pub fn ftCo_DownWait_Anim(g: G) {
    let f = fp(g);
    if f.x2224_b2() == 0 {
        unsafe { f.mv.co.downwait.x0 -= 1.0 };
    }
    if unsafe { f.mv.co.downwait.x0 } <= 0.0 {
        let msid = if f.motion_id == ftCo_MS_DownWaitU as _ { ftCo_MS_DownStandU } else { ftCo_MS_DownStandD };
        ext::ftCo_80098160(g, msid as i32);
    }
}

pub fn ftCo_DownWait_IASA(g: G) {
    if ext::ftCo_800984D4(g) {
        return;
    }
    if ext::ftCo_Down_CheckInput(g) {
        return;
    }
    ext::ftCo_800980BC(g);
}

pub fn ftCo_DownWait_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_DownWait_Coll(g: G) {
    ext::ft_80083F88(g);
}
