//! Complete PC `src/melee/lb/lbshadow.c` (11 definitions, including three static tangents).
//! Native records come from melee_types; external systems use named `ext` hooks.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use core::ptr::{addr_of_mut, null_mut};
use melee_core::fp::{fmadds, fmsubs};
use crate::lbvector;
use melee_core::vec::Vec3 as CVec3;
use melee_types::*;
use super::lbext::__assert;

/// Native bindings for game globals and callees whose raw-record entrypoints are not linked.
pub mod ext {
    use super::*;
    use crate::tu::lbext::lb_hook;
    pub static mut HSD_GObjPLinkHead: *mut *mut HSD_GObj = null_mut();
    pub static mut HSD_GObjGXLinkHead: *mut *mut HSD_GObj = null_mut();
    pub static mut HSD_GObj_804D7814: *mut HSD_GObj = null_mut();
    pub static mut HSD_PerfCurrentStat: HSD_PerfStat = unsafe { core::mem::zeroed() };
    pub static mut HSD_identityMtx: [[f32; 4]; 3] = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
    pub use crate::tu::lbext::OSReport;
    pub use crate::tu::lbgx::ext::{GXBegin, GXLoadPosMtxImm, GXPosition3f32, GXSetCurrentMtx, GXSetVtxAttrFmt, GXSetVtxDesc};
    lb_hook!(H_GXEnd, GXEnd, () = || unsafe { crate::tu::sobjlib::GXEnd() });
    lb_hook!(H_GXSetScissor, GXSetScissor, (a: u32, b: u32, c: u32, d: u32));
    lb_hook!(H_HSD_SetupChannelAll, HSD_SetupChannelAll, (chan: *mut HSD_Chan));
    lb_hook!(H_HSD_ClearVtxDesc, HSD_ClearVtxDesc, () = || unsafe { crate::tu::pobj::HSD_ClearVtxDesc() });
    lb_hook!(H_HSD_StateSetCullMode, HSD_StateSetCullMode, (mode: i32));
    lb_hook!(H_HSD_CObjGetTop, HSD_CObjGetTop, (c: *mut HSD_CObj) -> f32);
    lb_hook!(H_HSD_CObjGetBottom, HSD_CObjGetBottom, (c: *mut HSD_CObj) -> f32);
    lb_hook!(H_HSD_CObjGetLeft, HSD_CObjGetLeft, (c: *mut HSD_CObj) -> f32);
    lb_hook!(H_HSD_CObjGetRight, HSD_CObjGetRight, (c: *mut HSD_CObj) -> f32);
    lb_hook!(H_HSD_CObjGetNear, HSD_CObjGetNear, (c: *mut HSD_CObj) -> f32 = |c| unsafe { crate::tu::cobj::HSD_CObjGetNear(c) });
    lb_hook!(H_HSD_CObjSetNear, HSD_CObjSetNear, (c: *mut HSD_CObj, v: f32) = |c, v| unsafe { crate::tu::cobj::HSD_CObjSetNear(c, v) });
    lb_hook!(H_HSD_CObjSetFar, HSD_CObjSetFar, (c: *mut HSD_CObj, v: f32) = |c, v| unsafe { crate::tu::cobj::HSD_CObjSetFar(c, v) });
    lb_hook!(H_HSD_CObjSetProjectionType, HSD_CObjSetProjectionType, (c: *mut HSD_CObj, t: u32));
    lb_hook!(H_HSD_CObjSetFlags, HSD_CObjSetFlags, (c: *mut HSD_CObj, f: u32) = |c, f| unsafe { crate::tu::cobj::HSD_CObjSetFlags(c, f) });
    lb_hook!(H_HSD_CObjSetEyePosition, HSD_CObjSetEyePosition, (c: *mut HSD_CObj, v: *mut Vec3));
    lb_hook!(H_HSD_CObjSetInterest, HSD_CObjSetInterest, (c: *mut HSD_CObj, v: *mut Vec3));
    lb_hook!(H_HSD_CObjSetUpVector, HSD_CObjSetUpVector, (c: *mut HSD_CObj, v: *mut Vec3));
    lb_hook!(H_HSD_CObjSetOrtho, HSD_CObjSetOrtho, (c: *mut HSD_CObj, top: f32, bottom: f32, left: f32, right: f32) = |c, top, bottom, left, right| unsafe { crate::tu::cobj::HSD_CObjSetOrtho(c, top, bottom, left, right) });
    lb_hook!(H_HSD_ShadowAlloc, HSD_ShadowAlloc, () -> *mut HSD_Shadow = || unsafe { crate::tu::shadow::HSD_ShadowAlloc() });
    lb_hook!(H_HSD_ShadowSetSize, HSD_ShadowSetSize, (s: *mut HSD_Shadow, w: u16, h: u16) = |s, w, h| unsafe { crate::tu::shadow::HSD_ShadowSetSize(s, w, h) });
    lb_hook!(H_HSD_ShadowAddObject, HSD_ShadowAddObject, (s: *mut HSD_Shadow, j: *mut HSD_JObj) = |s, j| unsafe { crate::tu::shadow::HSD_ShadowAddObject(s, j) });
    lb_hook!(H_HSD_ShadowDeleteObject, HSD_ShadowDeleteObject, (s: *mut HSD_Shadow, j: *mut HSD_JObj) = |s, j| unsafe { crate::tu::shadow::HSD_ShadowDeleteObject(s, j) });
    lb_hook!(H_HSD_ShadowSetActive, HSD_ShadowSetActive, (s: *mut HSD_Shadow, active: i32) = |s, active| unsafe { crate::tu::shadow::HSD_ShadowSetActive(s, active) });
    lb_hook!(H_HSD_ShadowRemove, HSD_ShadowRemove, (s: *mut HSD_Shadow) = |s| unsafe { crate::tu::shadow::HSD_ShadowRemove(s) });
    lb_hook!(H_HSD_ShadowInit, HSD_ShadowInit, (s: *mut HSD_Shadow) = |s| unsafe { crate::tu::shadow::HSD_ShadowInit(s) });
    lb_hook!(H_HSD_ShadowStartRender, HSD_ShadowStartRender, (s: *mut HSD_Shadow) = |s| unsafe { crate::tu::shadow::HSD_ShadowStartRender(s) });
    lb_hook!(H_HSD_ShadowEndRender, HSD_ShadowEndRender, (s: *mut HSD_Shadow) = |s| unsafe { crate::tu::shadow::HSD_ShadowEndRender(s) });
    lb_hook!(H_HSD_ShadowSetViewingRect, HSD_ShadowSetViewingRect, (s: *mut HSD_Shadow, top: f32, bottom: f32, left: f32, right: f32));
    lb_hook!(H_HSD_ViewingRectInit, HSD_ViewingRectInit, (r: *mut HSD_ViewingRect, pos: *mut Vec3, interest: *mut Vec3, up: *mut Vec3, perspective: i32) = |r, pos, interest, up, perspective| unsafe { crate::tu::shadow::HSD_ViewingRectInit(r, pos, interest, up, perspective) });
    lb_hook!(H_HSD_ViewingRectAddRect, HSD_ViewingRectAddRect, (r: *mut HSD_ViewingRect, pos: *mut Vec3, top: f32, bottom: f32, left: f32, right: f32) = |r, pos, top, bottom, left, right| unsafe { crate::tu::shadow::HSD_ViewingRectAddRect(r, pos, top, bottom, left, right) });
    lb_hook!(H_HSD_ViewingRectCheck, HSD_ViewingRectCheck, (r: *mut HSD_ViewingRect) -> i32 = |r| unsafe { crate::tu::shadow::HSD_ViewingRectCheck(r) });
    lb_hook!(H_HSD_StartRender, HSD_StartRender, (pass: i32));
    lb_hook!(H_HSD_Init_803755A8, HSD_Init_803755A8, () = || unsafe { crate::tu::initialize::HSD_Init_803755A8() });
    lb_hook!(H_HSD_LObjGetFlags, HSD_LObjGetFlags, (l: *mut HSD_LObj) -> u32);
    lb_hook!(H_HSD_LObjGetPosition, HSD_LObjGetPosition, (l: *mut HSD_LObj, v: *mut Vec3) -> i32);
    lb_hook!(H_HSD_LObjGetInterest, HSD_LObjGetInterest, (l: *mut HSD_LObj, v: *mut Vec3) -> i32);
    lb_hook!(H_ftLib_80086960, ftLib_80086960, (g: *mut HSD_GObj) -> bool);
    lb_hook!(H_ftLib_800872B0, ftLib_800872B0, (g: *mut HSD_GObj) -> *mut LbShadow);
    lb_hook!(H_ftLib_800872BC, ftLib_800872BC, (g: *mut HSD_GObj) -> bool);
    lb_hook!(H_ftDrawCommon_80081168, ftDrawCommon_80081168, ());
    lb_hook!(H_ftDrawCommon_80081200, ftDrawCommon_80081200, ());
    lb_hook!(H_Ground_801C0508, Ground_801C0508, () -> i32);
}

const FILE: &str = "src/melee/lb/lbshadow.c";

/// lbshadow.c `splGetCardinalTangent` (14 fused sites).
pub fn splGetCardinalTangent(p: &mut Vec3, cp: &[Vec3], tension: f32, u: f32) {
    let u2 = u * u;
    let car0 = tension * (fmadds(-3.0, u2, 4.0 * u) - 1.0);
    let car1 = fmadds(3.0 * (2.0 - tension), u2, 2.0 * (tension - 3.0) * u);
    let car2 = tension + fmadds(3.0 * (tension - 2.0), u2, 2.0 * -fmadds(2.0, tension, -3.0) * u);
    let car3 = tension * fmadds(3.0, u2, -(2.0 * u));
    let c = [car0, car1, car2, car3];
    let comb = |f: fn(&Vec3) -> f32| {
        fmadds(f(&cp[3]), c[3], fmadds(f(&cp[2]), c[2], fmadds(f(&cp[0]), c[0], f(&cp[1]) * c[1])))
    };
    p.x = comb(|v| v.x);
    p.y = comb(|v| v.y);
    p.z = comb(|v| v.z);
}

/// lbshadow.c `splGetBSplineTangent` (11 fused sites).
pub fn splGetBSplineTangent(p: &mut Vec3, cp: &[Vec3], u: f32) {
    let u2 = u * u;
    let u_1 = 1.0 - u;
    let half = 0.5f32;
    let b0 = u_1 * (-half * u_1);
    let b1 = half * fmadds(3.0, u2, -(4.0 * u));
    let b2 = half * (1.0 + fmadds(-3.0, u2, 2.0 * u));
    let b3 = half * u2;
    let c = [b0, b1, b2, b3];
    let comb = |f: fn(&Vec3) -> f32| {
        fmadds(f(&cp[3]), c[3], fmadds(f(&cp[2]), c[2], fmadds(f(&cp[0]), c[0], f(&cp[1]) * c[1])))
    };
    p.x = comb(|v| v.x);
    p.y = comb(|v| v.y);
    p.z = comb(|v| v.z);
}

/// lbshadow.c `splGetBezierTangent` (12 fused sites).
pub fn splGetBezierTangent(p: &mut Vec3, cp: &[Vec3], u: f32) {
    let u_1 = u - 1.0;
    let u2 = u * u;
    let bez0 = -3.0 * u_1 * u_1;
    let bez1 = 3.0 * fmadds(3.0, u2, fmadds(-4.0, u, 1.0));
    let bez2 = 3.0 * fmadds(2.0, u, -(3.0 * u2));
    let bez3 = 3.0 * u2;
    let c = [bez0, bez1, bez2, bez3];
    let comb = |f: fn(&Vec3) -> f32| {
        fmadds(f(&cp[3]), c[3], fmadds(f(&cp[2]), c[2], fmadds(f(&cp[0]), c[0], f(&cp[1]) * c[1])))
    };
    p.x = comb(|v| v.x);
    p.y = comb(|v| v.y);
    p.z = comb(|v| v.z);
}

/// lbshadow.c `lbShadow_8000E9F0`: the tangent of `spline` at `u` in [0, 1] (no-op outside),
/// by type: 0 linear, 1 Bezier, 2 B-spline, 3 cardinal; any other type leaves `p` alone.
pub fn lbShadow_8000E9F0(p: &mut Vec3, spline: &HSD_Spline, mut u: f32) {
    if u < 0.0 || u > 1.0 {
        return;
    }
    let orig_u = u;
    u *= (spline.numcv as i32 - 1) as f32;
    let mut idx = (u as i32) as i16; // gwtool saturates to i32, then truncates.
    u -= idx as f32;
    // SAFETY: cv holds numcv control points (3*numcv-2 for Bezier); the C indexes it unchecked.
    let cps = |at: isize| unsafe { core::slice::from_raw_parts(spline.cv.offset(at), 4) };
    match spline.r#type {
        0 => {
            if orig_u == 1.0 {
                idx = idx.wrapping_sub(1);
            }
            let cp = unsafe { core::slice::from_raw_parts(spline.cv.offset(idx as isize), 2) };
            p.x = cp[1].x - cp[0].x;
            p.y = cp[1].y - cp[0].y;
            p.z = cp[1].z - cp[0].z;
        }
        1 => splGetBezierTangent(p, cps(idx as isize * 3), u),
        2 => splGetBSplineTangent(p, cps(idx as isize), u),
        3 => splGetCardinalTangent(p, cps(idx as isize), spline.tension, u),
        _ => {}
    }
}

/// lbshadow.c `lbShadow_8000ED54`: allocates the fighter shadow (256x256 ortho camera, intensity
/// 0xC0, texture scale +-0.45), adds `jobj` and activates it, and clears the suppression flags.
///
/// # Safety
/// `lbshadow` must be valid for writes; `jobj` is handed to the shadow system.
pub unsafe fn lbShadow_8000ED54(lbshadow: *mut LbShadow, jobj: *mut HSD_JObj) {
    if lbshadow.is_null() {
        __assert(FILE, 0x36, "lbshadow");
    }
    let shadow = ext::HSD_ShadowAlloc();
    if !shadow.is_null() {
        ext::HSD_CObjSetProjectionType((*shadow).camera, PROJ_ORTHO as u32);
        ext::HSD_CObjSetNear((*shadow).camera, 0.001);
        ext::HSD_CObjSetFar((*shadow).camera, 5000.0);
        ext::HSD_CObjSetFlags((*shadow).camera, 1);
        ext::HSD_ShadowSetSize(shadow, 0x100, 0x100);
        (*shadow).intensity = 0xC0;
        (*shadow).scaleS = 0.45;
        (*shadow).scaleT = -0.45;
        ext::HSD_ShadowAddObject(shadow, jobj);
        ext::HSD_ShadowSetActive(shadow, 1);
    }
    (*lbshadow).set_x0_b0(0);
    (*lbshadow).set_x0_b1(0);
    (*lbshadow).set_x0_b3(0);
    (*lbshadow).set_x0_b4(0);
    (*lbshadow).set_x0_b5(0);
    (*lbshadow).shadow = shadow;
}

/// lbshadow.c `lbShadow_8000EE8C`: removes the shadow, if any.
///
/// # Safety
/// `lbshadow` must be valid for reads.
pub unsafe fn lbShadow_8000EE8C(lbshadow: *mut LbShadow) {
    if lbshadow.is_null() {
        __assert(FILE, 0x62, "lbshadow");
    }
    if !(*lbshadow).shadow.is_null() {
        ext::HSD_ShadowRemove((*lbshadow).shadow);
    }
}

fn cv(v: &Vec3) -> CVec3 {
    CVec3 { x: v.x, y: v.y, z: v.z }
}
fn tv(v: CVec3) -> Vec3 {
    Vec3 { x: v.x, y: v.y, z: v.z }
}

/// True when any of the six suppression bits of `lbs` is set.
unsafe fn any_flag(lbs: *mut LbShadow) -> bool {
    (*lbs).x0_b0() != 0 || (*lbs).x0_b1() != 0 || (*lbs).x0_b2() != 0 || (*lbs).x0_b3() != 0 || (*lbs).x0_b4() != 0 || (*lbs).x0_b5() != 0
}

/// lbshadow.c `lbShadow_8000EEE0`: activates the fighter shadow when no suppression bit is set
/// and `ftLib_800872BC` allows it, deactivates it otherwise.
///
/// # Safety
/// `gobj` must be a valid fighter GObj.
pub unsafe fn lbShadow_8000EEE0(gobj: *mut HSD_GObj) {
    if ext::ftLib_80086960(gobj) {
        let lbs = ext::ftLib_800872B0(gobj);
        if !lbs.is_null() {
            if !any_flag(lbs) && ext::ftLib_800872BC(gobj) {
                ext::HSD_ShadowSetActive((*lbs).shadow, 1);
            } else {
                ext::HSD_ShadowSetActive((*lbs).shadow, 0);
            }
        }
    }
}

/// lbshadow.c `lbShadow_8000EFEC`: clears every fighter's "too many shadows" bit, then sets it on
/// the 6th and later fighters that would otherwise cast one, refreshing each shadow's activity.
///
/// # Safety
/// The fighter player link (`ext::HSD_GObjPLinkHead`) must be valid.
pub unsafe fn lbShadow_8000EFEC() {
    let mut count = 0i32;
    let head = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
    let mut g = head;
    while !g.is_null() {
        let lbs = ext::ftLib_800872B0(g);
        if !lbs.is_null() {
            (*lbs).set_x0_b2(0);
        }
        g = (*g).next;
    }
    let mut cur = head;
    while !cur.is_null() {
        let lbs = ext::ftLib_800872B0(cur);
        if !lbs.is_null() {
            if !any_flag(lbs) {
                count += 1;
                if count > 5 {
                    (*lbs).set_x0_b2(1);
                }
            }
            lbShadow_8000EEE0(cur);
        }
        cur = (*cur).next;
    }
}

/// lbshadow.c `lbShadow_803BA0FC`: the channel `lbShadow_8000F214` sets up (black ambient, opaque
/// black material, register sources, no light).
fn shadow_chan() -> HSD_Chan {
    let mut c = HSD_Chan::zeroed();
    c.chan = GX_COLOR0A0;
    c.mat_color.a = 0xFF;
    c.amb_src = GX_SRC_REG;
    c.mat_src = GX_SRC_REG;
    c.light_mask = GX_LIGHT_NULL;
    c.diff_fn = GX_DF_CLAMP;
    c.attn_fn = GX_AF_NONE;
    c
}

/// lbshadow.c `lbShadow_8000F214`: clears the shadow texture with one black quad 1.2x the camera
/// ortho rectangle at 1.1x its near plane.
///
/// # Safety
/// `shadow` (with camera and texture) must be valid.
pub unsafe fn lbShadow_8000F214(shadow: *mut HSD_Shadow) {
    let cobj = (*shadow).camera;
    let mut chan = shadow_chan();
    ext::HSD_SetupChannelAll(&mut chan);
    let imagedesc = (*(*shadow).texture).imagedesc;
    ext::GXSetScissor(0, 0, (*imagedesc).width as u32, (*imagedesc).height as u32);
    ext::GXLoadPosMtxImm(addr_of_mut!(ext::HSD_identityMtx) as *const [f32; 4], GX_PNMTX0 as u32);
    ext::HSD_PerfCurrentStat.nb_mtx_load = ext::HSD_PerfCurrentStat.nb_mtx_load.wrapping_add(1);
    ext::GXSetCurrentMtx(GX_PNMTX0 as u32);
    ext::HSD_ClearVtxDesc();
    ext::GXSetVtxDesc(GX_VA_POS as i32, GX_DIRECT as i32);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0 as i32, GX_VA_POS as i32, GX_TEX_ST as i32, GX_RGBA6 as i32, 0);
    ext::HSD_StateSetCullMode(GX_CULL_BACK as i32);

    let mut y0 = ext::HSD_CObjGetTop(cobj);
    let mut y1 = ext::HSD_CObjGetBottom(cobj);
    let mut x0 = ext::HSD_CObjGetLeft(cobj);
    let mut x1 = ext::HSD_CObjGetRight(cobj);
    let mut z = ext::HSD_CObjGetNear(cobj);
    y0 *= 1.2;
    y1 *= 1.2;
    x0 *= 1.2;
    x1 *= 1.2;
    z *= -1.1;

    ext::GXBegin(GX_QUADS as i32, GX_VTXFMT0 as i32, 4);
    ext::GXPosition3f32(x0, y0, z);
    ext::GXPosition3f32(x1, y0, z);
    ext::GXPosition3f32(x1, y1, z);
    ext::GXPosition3f32(x0, y1, z);
    ext::GXEnd();
}

/// lbshadow.c `lbShadow_8000F38C(arg0)`: finds the stage key light, derives the shadow camera
/// (position, interest, up vector) from it, and re-renders every fighter shadow texture
/// (clearing it first when `arg0 != 0`).
///
/// # Safety
/// The fighter player link, the light GX link and every fighter must be valid.
pub unsafe fn lbShadow_8000F38C(arg0: i32) {
    let mut rect = HSD_ViewingRect::zeroed();
    let mut light_pos = Vec3::zeroed();
    let mut light_dir = Vec3::zeroed();
    let mut up_vec = Vec3::zeroed();
    let mut eye_pos = Vec3::zeroed();
    let mut interest_pos = Vec3::zeroed();
    let mut no_light = false;
    let plink = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);

    let mut g = plink;
    while !g.is_null() {
        let lbs = ext::ftLib_800872B0(g);
        if !lbs.is_null() {
            (*lbs).set_x0_b2(0);
        }
        g = (*g).next;
    }

    let mut lobj: *mut HSD_LObj = null_mut();
    let mut fallback: *mut HSD_LObj = null_mut();
    let mut lgobj = *ext::HSD_GObjGXLinkHead.add(4);
    while !lgobj.is_null() {
        let mut next_gx = (*lgobj).next_gx;
        lobj = (*lgobj).hsd_obj as *mut HSD_LObj;
        while !lobj.is_null() {
            if (*lobj).flags & 3 != 0 {
                fallback = lobj;
            }
            if ext::HSD_LObjGetFlags(lobj) & 0x400 != 0 {
                next_gx = null_mut();
                break;
            }
            lobj = (*lobj).next;
        }
        lgobj = next_gx;
    }
    if lobj.is_null() && !fallback.is_null() {
        lobj = fallback;
    }
    if lobj.is_null() {
        __assert(FILE, 0x181, "lobj");
    }

    if ext::HSD_LObjGetPosition(lobj, &mut light_pos) == 0 {
        ext::OSReport("coudn t get light position ...\n", &[]);
        __assert(FILE, 0x184, "0");
    }

    if ext::HSD_LObjGetInterest(lobj, &mut light_dir) != 0 {
        let mut lp = cv(&light_pos);
        lbvector::sub(&mut lp, &cv(&light_dir));
        light_pos = tv(lp);
    }
    light_dir = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let light_vec = tv(lbvector::diff(&cv(&light_pos), &cv(&light_dir)));
    let mut dist = fmadds(light_vec.z, light_vec.z, fmadds(light_vec.x, light_vec.x, light_vec.y * light_vec.y));
    dist = crate::gekko::sqrtf(dist);

    if dist < 0.001 {
        no_light = true;
        light_pos.x = 0.0;
        light_dir.x = 0.0;
        up_vec.x = 0.0;
        light_pos.y = 0.0;
        light_dir.y = 0.0;
        up_vec.y = 1.0;
        light_pos.z = 1.0;
        light_dir.z = 0.0;
        up_vec.z = 0.0;
    } else {
        let xz_sq = fmadds(light_vec.x, light_vec.x, light_vec.z * light_vec.z);
        if xz_sq > 0.0000010000001 {
            up_vec.z = 0.0;
            up_vec.x = 0.0;
            up_vec.y = 1.0;
            let mut norm_dir = tv(lbvector::diff(&cv(&light_dir), &cv(&light_pos)));
            let mut nd = cv(&norm_dir);
            let nlen = lbvector::normalize(&mut nd);
            norm_dir = tv(nd);
            if nlen < 100.0 {
                light_pos.x = -fmsubs(100.0, norm_dir.x, light_dir.x);
                light_pos.y = -fmsubs(100.0, norm_dir.y, light_dir.y);
                light_pos.z = -fmsubs(100.0, norm_dir.z, light_dir.z);
            }
            let right_vec = lbvector::crossprod_normalized(&cv(&up_vec), &cv(&norm_dir));
            up_vec = tv(lbvector::crossprod_normalized(&cv(&norm_dir), &right_vec));
        } else {
            up_vec.y = 0.0;
            up_vec.x = 0.0;
            up_vec.z = -1.0;
        }
    }

    ext::ftDrawCommon_80081200();

    let mut gobj = plink;
    while !gobj.is_null() {
        let fp = (*gobj).user_data as *mut Fighter;
        let cm = (*fp).x890_cameraBox;

        (*fp).x20A4.set_x0_b0(no_light as u8);

        if !(*fp).x20A4.shadow.is_null() {
            let mut added = false;
            ext::HSD_ShadowDeleteObject((*fp).x20A4.shadow, null_mut());
            if (*fp).x21FC_flag.anon_0.b7() != 0 {
                if (*fp).invisible() == 0
                    && (*fp).x221E_b5() == 0
                    && (!(*fp).x5AC.xC[1].is_null() || (*fp).kind > Ft_Kind_Sandbag)
                {
                    ext::HSD_ShadowAddObject((*fp).x20A4.shadow, (*gobj).hsd_obj as *mut HSD_JObj);
                    added = true;
                }
                if !(*fp).x20A0_accessory.is_null() {
                    ext::HSD_ShadowAddObject((*fp).x20A4.shadow, (*fp).x20A0_accessory);
                    added = true;
                }
            }
            (*fp).x20A4.set_x0_b3(if added { 0 } else { 1 });
        }

        if ext::ftLib_80086960(gobj) {
            let lbs = ext::ftLib_800872B0(gobj);
            if !lbs.is_null() {
                if !any_flag(lbs) && ext::ftLib_800872BC(gobj) {
                    ext::HSD_ShadowSetActive((*lbs).shadow, 1);
                } else {
                    ext::HSD_ShadowSetActive((*lbs).shadow, 0);
                }
            }
        }

        if !(*fp).x20A4.shadow.is_null() && !cm.is_null() && (*fp).x20A4.x0_b3() == 0 {
            let intensity = ext::Ground_801C0508() as u8;
            let shadow2 = (*fp).x20A4.shadow;
            (*shadow2).intensity = intensity;

            eye_pos = Vec3 { x: (*cm).bone_pos.x + light_pos.x, y: (*cm).bone_pos.y + light_pos.y, z: (*cm).bone_pos.z + light_pos.z };
            interest_pos = Vec3 { x: (*cm).bone_pos.x + light_dir.x, y: (*cm).bone_pos.y + light_dir.y, z: (*cm).bone_pos.z + light_dir.z };
            ext::HSD_CObjSetEyePosition((*shadow2).camera, &mut eye_pos);
            ext::HSD_CObjSetInterest((*shadow2).camera, &mut interest_pos);
            ext::HSD_CObjSetUpVector((*shadow2).camera, &mut up_vec);

            let mut cam_pos = (*cm).bone_pos;
            ext::HSD_ViewingRectInit(&mut rect, &mut eye_pos, &mut interest_pos, &mut up_vec, 0);
            let mut i = 0;
            while i < 0x14 {
                let scale = (*cm).target_ext.v.z;
                let top = 1.2 * scale;
                let bot = 1.2 * -scale;
                ext::HSD_ViewingRectAddRect(&mut rect, &mut cam_pos, top, bot, bot, top);
                if ext::HSD_ViewingRectCheck(&mut rect) != 0 {
                    break;
                }
                i += 1;
            }
            if i < 0x14 {
                ext::HSD_ShadowSetViewingRect(shadow2, rect.top, rect.bottom, rect.left, rect.right);
            } else {
                ext::HSD_CObjSetOrtho((*shadow2).camera, 128.0, -128.0, -128.0, 128.0);
            }

            ext::HSD_ShadowInit(shadow2);
            ext::HSD_StartRender(HSD_RP_OFFSCREEN as i32);
            ext::HSD_GObj_804D7814 = gobj;
            ext::HSD_ShadowStartRender(shadow2);
            if arg0 != 0 {
                lbShadow_8000F214(shadow2);
            }
            ext::HSD_ShadowEndRender(shadow2);
            ext::HSD_Init_803755A8();
        }
        gobj = (*gobj).next;
    }

    ext::ftDrawCommon_80081168();
}
