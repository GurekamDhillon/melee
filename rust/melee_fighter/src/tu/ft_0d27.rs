//! src/melee/ft/ft_0D27.c: mushroom (Kinoko) scale helpers, capture release, velocity save /
//! restore. Fused site (`tools/rust-port/fma_sites.py`): fn_800D2A3C `(fast - mid) * scale + mid`
//! -> `fmadds`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use super::ft_0cd1::{ext as e, fp, G};
use crate::hooks as h;
use crate::types::*;
use melee_core::fp::fmadds;

/// Callees outside this TU that have no port yet. Each panics until rewired.
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    pub fn ftCo_800D105C(_g: G) { todo!("TODO(group3 ftCo) ftCo_800D105C") }
    pub fn ftCamera_80076064(_fp: &mut Fighter) { todo!("TODO(group3 ftcamera) ftCamera_80076064") }
    pub fn ft_80081C88(_g: G, _s: f32) { todo!("TODO(group3 ft_081B) ft_80081C88") }
    pub extern "C" fn ftCo_800968C8(_g: G) { todo!("TODO(group3 ftCo) ftCo_800968C8") }
    pub extern "C" fn ftCo_FallAerial_Enter(_g: G) { todo!("TODO(group3 ftCo) ftCo_FallAerial_Enter") }
    pub fn HSD_JObjLoadJoint(_j: *mut HSD_Joint) -> *mut HSD_JObj { todo!("TODO(group1 HSD) HSD_JObjLoadJoint") }
    pub fn HSD_JObjAddAnim(_j: *mut HSD_JObj, _a: *mut HSD_AnimJoint) { todo!("TODO(group1 HSD) HSD_JObjAddAnim") }
    pub static mut HSD_JOBJ_ANIM: Option<fn(*mut HSD_JObj)> = None;
    pub fn HSD_JObjAnim(j: *mut HSD_JObj) { if let Some(h) = unsafe { HSD_JOBJ_ANIM } { return h(j) } todo!("TODO(group1 HSD) HSD_JObjAnim") }
    pub fn it_80293660(_i: i32) -> *mut HSD_AnimJoint { todo!("TODO(group5 itkinoko) it_80293660") }
}

/// `lbl_803C6FF0`: the one-joint scale driver fn_800D299C loads.
pub static mut lbl_803C6FF0: HSD_Joint = HSD_Joint {
    class_name: core::ptr::null_mut(),
    flags: 8,
    child: core::ptr::null_mut(),
    next: core::ptr::null_mut(),
    u: unsafe { core::mem::zeroed() },
    rotation: Vec { x: 0.0, y: 0.0, z: 0.0 },
    scale: Vec { x: 1.0, y: 1.0, z: 1.0 },
    position: Vec { x: 0.0, y: 0.0, z: 0.0 },
    mtx: core::ptr::null_mut(),
    robjdesc: core::ptr::null_mut(),
};

/// `ftCo_800D2770(gobj, scale)`.
pub fn ftco_800d2770(g: G, arg8: f32) {
    h::ftlib_set_scale(g, arg8);
    ext::ftCo_800D105C(g);
    ext::ftCamera_80076064(fp(g));
    let s = fp(g).x34_scale.y;
    ext::ft_80081C88(g, s);
}

/// `ftCo_800D27C4`.
pub fn ftco_800d27c4(f: &Fighter) -> bool {
    if f.motion_id as i32 == ftCo_MS_DamageIce as i32 {
        return false;
    }
    !matches!(unsafe { f.x2070.anon_0 }.x2071_b0_3(), 12 | 13)
}

/// `fn_800D2818`.
pub fn fn_800d2818(f: &Fighter) -> bool {
    let m = f.motion_id as i32;
    if m == ftCo_MS_DamageIce as i32 {
        return false;
    }
    if m >= ftCo_MS_KinokoGiantStart as i32 && m <= ftCo_MS_KinokoSmallEndAir as i32 {
        return false;
    }
    let tmp = unsafe { f.x2070.anon_0 }.x2071_b0_3() as i32;
    !(tmp != 5 && (tmp >= 5 || tmp != 0) && unsafe { f.x2070.anon_0 }.x2071_b4() == 0)
}

/// `fn_800D2890(gobj, msid)`: pick the fall state to return to.
pub fn fn_800d2890(g: G, ms_id: i32) {
    let f = fp(g);
    let cb: unsafe extern "C" fn(G) = if f.kind == Ft_Kind_Peach && ms_id == 0x172 {
        ext::ftCo_800968C8
    } else if ms_id == ftCo_MS_FallAerial as i32 {
        ext::ftCo_FallAerial_Enter
    } else if ms_id == ftCo_MS_FallSpecial as i32 || ms_id == ftCo_MS_ItemParasolFallSpecial as i32 {
        ext::ftCo_800968C8
    } else {
        e::ftCo_Fall_Enter
    };
    unsafe { f.mv.co.unk_800D2890.x0 = Some(cb) };
}

/// `fn_800D290C`: release a grab both ways.
pub fn fn_800d290c(g: G) {
    use super::ft_0c8c::ext as k;
    crate::ftcommon3::ftcommon_8007db58(g);
    k::ftCo_8009750C(g);
    let victim = fp(g).victim_gobj;
    if !victim.is_null() {
        if fp(g).anon_221b.x221B_b5() != 0 {
            k::ftCo_CaptureCut_Enter(victim);
        } else {
            k::ftCo_CaptureCut_Enter(g);
            k::ftCo_800DA698(victim, false);
        }
        crate::ftcommon3::ftcommon_8007db58(victim);
    }
}

/// `fn_800D299C(gobj, idx, from, to)`.
pub fn fn_800d299c(g: G, idx: i32, arg8: f32, arg9: f32) {
    let f = fp(g);
    if f.x2184.is_null() {
        f.x2184 = ext::HSD_JObjLoadJoint(unsafe { &mut lbl_803C6FF0 });
    }
    let f = fp(g);
    unsafe {
        f.mv.co.walk.middle_anim_frame = arg8;
        f.mv.co.walk.fast_anim_frame = arg9;
    }
    ext::HSD_JObjAddAnim(f.x2184, ext::it_80293660(idx));
    e::ftAnim_80070734(fp(g).x2184, 0.0);
    ext::HSD_JObjAnim(fp(g).x2184);
}

/// The scale fn_800D2A3C sets: `(fast - mid) * scale + mid`, fused as in the shipping build.
pub fn lerp_scale(mid: f32, fast: f32, scale: f32) -> f32 {
    fmadds(fast - mid, scale, mid)
}

/// `fn_800D2A3C`: step the scale animation; false when it ended.
pub fn fn_800d2a3c(g: G) -> bool {
    ext::HSD_JObjAnim(fp(g).x2184);
    let f = fp(g);
    let scale = unsafe { (*f.x2184).scale.x };
    let (mid, fast) = unsafe { (f.mv.co.walk.middle_anim_frame, f.mv.co.walk.fast_anim_frame) };
    h::ftlib_set_scale(g, lerp_scale(mid, fast, scale));
    let flags = unsafe { (*(*fp(g).x2184).aobj).flags };
    flags & 0x4000_0000 == 0
}

/// `fn_800D2AD8`: sfx id 284 (the C passes the motion-state constant).
pub fn fn_800d2ad8(f: &mut Fighter) { h::play_sfx(f, ftCo_MS_CaptureDamageKoopaAir as i32, 0x7F, 0x40) }

/// `fn_800D2B04`: sfx id 285.
pub fn fn_800d2b04(f: &mut Fighter) { h::play_sfx(f, ftCo_MS_CaptureWaitKoopaAir as i32, 0x7F, 0x40) }

/// Test/rewire override for fn_800D2B30's `ftCommon_8007E2FC(fp->gobj)` (unset = the port).
pub static mut FTCOMMON_8007E2FC: Option<fn(G)> = None;

/// `fn_800D2B30`: save every velocity into `mv.co.common`, then zero them.
pub fn fn_800d2b30(g: G) {
    let f = fp(g);
    unsafe {
        let c = &mut f.mv.co.common;
        c.x14 = f.xE4_ground_accel_1;
        c.x18 = f.xE8_ground_accel_2;
        c.x28 = f.x74_self_accel;
        c.x1C = f.gr_vel;
        c.x34 = f.self_vel;
        c.x20 = f.xF0_ground_kb_vel;
        c.x40 = f.x8c_kb_vel;
        c.x24 = f.xF4_ground_attacker_shield_kb_vel;
        c.x4C = f.x98_atk_shield_kb;
        c.x58 = f.xD4_unk_vel;
    }
    let fg = fp(g).gobj;
    match unsafe { FTCOMMON_8007E2FC } { Some(h) => h(fg), None => crate::ftcommon2::ftcommon_8007e2fc(fp(fg)) }
}

/// `fn_800D2BF8`: restore the saved velocities.
pub fn fn_800d2bf8(g: G) {
    let f = fp(g);
    unsafe {
        let c = f.mv.co.common;
        f.xE4_ground_accel_1 = c.x14;
        f.xE8_ground_accel_2 = c.x18;
        f.x74_self_accel = c.x28;
        f.gr_vel = c.x1C;
        f.self_vel = c.x34;
        f.xF0_ground_kb_vel = c.x20;
        f.x8c_kb_vel = c.x40;
        f.xF4_ground_attacker_shield_kb_vel = c.x24;
        f.x98_atk_shield_kb = c.x4C;
        f.xD4_unk_vel = c.x58;
    }
}

/// `fn_800D2CA0`: landed; ground velocity from the saved self velocity.
pub fn fn_800d2ca0(f: &mut Fighter) {
    crate::ftcommon3::ftcommon_8007d7fc(f);
    unsafe { f.mv.co.common.x1C = f.mv.co.common.x34.x };
}

/// `fn_800D2CD4`: went airborne.
pub fn fn_800d2cd4(f: &mut Fighter) {
    crate::ftcommon2::ftcommon_8007d5d4(f);
    unsafe {
        f.mv.co.common.x1C = 0.0;
        f.mv.co.common.x28.y = 0.0;
    }
}

/// `ft_800D2D0C`: the held item is `It_Kind_ScBall`.
pub fn ft_800d2d0c(g: G) -> bool {
    let it = fp(g).item_gobj;
    !it.is_null() && h::it_get_kind(it) as ItemKind == It_Kind_ScBall
}
