//! Round 4: lb callers of rounds 1-3 and of group 1's ported code (lbtrigf `acosf`, MSL
//! `sqrtf`). Verified bit for bit against the C (`tests/oracle_r4.rs`).
//! TODO(group1): these lb TUs are lb leftovers; move into melee_runtime if group 1 takes them.
//!
//! FMA (`tools/rust-port/fma_sites.py`): `lb_8000D148` has the shipping build's 5 fused sites,
//! each an `fmadds` below; nothing else here fuses.
#![allow(non_snake_case)]

use crate::lb::{absf, calcPan};
use crate::lb_ptr::{splGetBSplineTangent, splGetBezierTangent, splGetCardinalTangent};
use melee_core::fp::fmadds;
use melee_runtime::gekko::sqrtf;
use melee_types as mt;
use mt::{CameraBlurData, DynamicsData, HSD_GObj, HSD_GObjEvent, HSD_JObj, HSD_Spline, Vec as Vec3};

/// `lbAudioAx_UserData` (lb/lbaudio_ax.static.h, 0x48 bytes on the GC). melee_types does not
/// scan `.static.h` headers. TODO(group0): generate it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct lbAudioAx_UserData {
    pub x0: i32,
    pub gobj: *mut HSD_GObj,
    pub owner: *mut HSD_GObj,
    pub xC: i32,
    pub x10: Option<unsafe extern "C" fn(*mut HSD_GObj) -> i32>,
    pub id: i32,
    pub start_vol: i32,
    pub end_vol: i32,
    pub vol: i32,
    pub pan_left: i32,
    pub pan_right: i32,
    pub pan: i32,
    pub voice_id: i32,
    pub current_frame: i32,
    pub end_frame: i32,
    pub x3C: f32,
    pub track: i32,
    /// MSL `bool` (int)
    pub x44: i32,
}

melee_core::gc_layout!(lbAudioAx_UserData, "lbAudioAx_UserData", size 0x48, align 4, [
    x0 @ 0x0: 4 "int",
    gobj @ 0x4: 4 "HSD_GObj*",
    owner @ 0x8: 4 "HSD_GObj*",
    xC @ 0xC: 4 "int",
    x10 @ 0x10: 4 "bool (*)(HSD_GObj*)",
    id @ 0x14: 4 "int",
    start_vol @ 0x18: 4 "int",
    end_vol @ 0x1C: 4 "int",
    vol @ 0x20: 4 "int",
    pan_left @ 0x24: 4 "int",
    pan_right @ 0x28: 4 "int",
    pan @ 0x2C: 4 "int",
    voice_id @ 0x30: 4 "int",
    current_frame @ 0x34: 4 "int",
    end_frame @ 0x38: 4 "int",
    x3C @ 0x3C: 4 "float",
    track @ 0x40: 4 "int",
    x44 @ 0x44: 4 "bool",
]);

/// `GET_SOUND(gobj)`: the gobj's user data as the sound object (may be NULL).
fn get_sound<'a>(gobj: &HSD_GObj) -> Option<&'a mut lbAudioAx_UserData> {
    // SAFETY: a sound gobj's user_data is its lbAudioAx_UserData (or NULL).
    unsafe { (mt::inline::HSD_GObjGetUserData(gobj) as *mut lbAudioAx_UserData).as_mut() }
}

fn pan_of(ud: &lbAudioAx_UserData) -> i32 {
    calcPan(ud.current_frame, ud.end_frame, ud.pan_left, ud.pan_right)
}

/// lbaudio_ax.c `fn_800253D8` (static): pan left->right when x3C == 1, else mirrored.
pub fn fn_800253D8(gobj: Option<&HSD_GObj>) -> bool {
    if let Some(ud) = gobj.and_then(get_sound) {
        ud.pan = if ud.x3C == 1.0 { pan_of(ud) } else { 0x7F - pan_of(ud) };
    }
    false
}

/// lbaudio_ax.c `fn_800256BC` (static): the same body as `fn_800253D8`.
pub fn fn_800256BC(gobj: Option<&HSD_GObj>) -> bool {
    if let Some(ud) = gobj.and_then(get_sound) {
        ud.pan = if ud.x3C == 1.0 { pan_of(ud) } else { 0x7F - pan_of(ud) };
    }
    false
}

/// lbaudio_ax.c `fn_80025B44` (static).
pub fn fn_80025B44(gobj: Option<&HSD_GObj>) -> bool {
    if let Some(ud) = gobj.and_then(get_sound) {
        ud.pan = pan_of(ud);
    }
    false
}

/// lbaudio_ax.c `fn_80025CBC` (static).
pub fn fn_80025CBC(gobj: Option<&HSD_GObj>) -> bool {
    if let Some(ud) = gobj.and_then(get_sound) {
        ud.pan = 0x7F - pan_of(ud);
    }
    false
}

/// lbaudio_ax.c `fn_800259EC` (static): the first time only, `fn_800253D8`.
pub fn fn_800259EC(gobj: Option<&HSD_GObj>) -> bool {
    if let Some(g) = gobj {
        if let Some(ud) = get_sound(g) {
            if ud.x44 == 0 {
                ud.x44 = 1;
                fn_800253D8(Some(g));
            }
        }
    }
    false
}

/// lbaudio_ax.c `fn_80025A98` (static): the same body as `fn_800259EC`.
pub fn fn_80025A98(gobj: Option<&HSD_GObj>) -> bool {
    if let Some(g) = gobj {
        if let Some(ud) = get_sound(g) {
            if ud.x44 == 0 {
                ud.x44 = 1;
                fn_800253D8(Some(g));
            }
        }
    }
    false
}

/// lbaudio_ax.c `lbAudioAx_800264E4`: the sound's voice id, -1 without one.
pub fn lbAudioAx_800264E4(gobj: Option<&HSD_GObj>) -> i32 {
    match gobj {
        Some(g) => get_sound(g).map_or(-1, |ud| ud.voice_id),
        None => -1,
    }
}

/// lbspdisplay.c `lb_800138CC`: set the camera blur's callback.
pub fn lb_800138CC(gobj: &HSD_GObj, blur_callback: HSD_GObjEvent) {
    // SAFETY: the blur gobj's user data is its CameraBlurData (the C does not check it).
    let data = unsafe { &mut *(mt::inline::HSD_GObjGetUserData(gobj) as *mut CameraBlurData) };
    data.callback = blur_callback;
}

/// lbspdisplay.c `lb_800138D8`: blur mode 1 with the given size.
pub fn lb_800138D8(gobj: &HSD_GObj, size: i8) {
    // SAFETY: as in `lb_800138CC`.
    let data = unsafe { &mut *(mt::inline::HSD_GObjGetUserData(gobj) as *mut CameraBlurData) };
    data.mode = 1;
    data.blur_size = size as u8;
}

/// lbspdisplay.c `lb_8001204C`: for each index in `indices[..count]`, the joint at that
/// depth-first position under `root` (instances' children skipped), or NULL past the end;
/// returns how many were found. Ascending indices continue the walk from the previous one.
pub fn lb_8001204C(root: Option<&HSD_JObj>, result: Option<&mut [*mut HSD_JObj]>, indices: Option<&[u16]>,
                   count: i32) -> i32 {
    let (Some(root), Some(out), Some(idx)) = (root, result, indices) else { return 0 };
    if count == 0 {
        return 0;
    }
    let root = root as *const HSD_JObj as *mut HSD_JObj;
    // SAFETY (all derefs below): the joints form a well-linked tree (child/next/parent).
    let get = |p: *mut HSD_JObj| unsafe { p.as_ref() };
    let mut found = 0;
    let mut prev: i32 = -1;
    let mut jobj: *mut HSD_JObj = core::ptr::null_mut();
    let mut cur: i32;
    let mut k = 0usize;
    let mut i = count;
    while i > 0 {
        let target = idx[k];
        if prev > target as i32 || prev == -1 {
            jobj = root;
            cur = 0;
        } else {
            cur = prev;
        }
        while let Some(j) = get(jobj) {
            if cur == target as i32 {
                break;
            }
            let mut saved = jobj;
            let next_node;
            if j.flags & mt::JOBJ_INSTANCE as u32 == 0 && !mt::inline::HSD_JObjGetChild(Some(j)).is_null() {
                next_node = mt::inline::HSD_JObjGetChild(Some(j));
            } else if !mt::inline::HSD_JObjGetNext(Some(j)).is_null() {
                next_node = mt::inline::HSD_JObjGetNext(Some(j));
            } else {
                loop {
                    let parent = mt::inline::HSD_JObjGetParent(get(saved));
                    if parent.is_null() {
                        next_node = core::ptr::null_mut();
                        break;
                    }
                    let pn = mt::inline::HSD_JObjGetNext(get(parent));
                    if !pn.is_null() {
                        saved = pn;
                        next_node = saved;
                        break;
                    }
                    saved = parent;
                }
            }
            jobj = next_node;
            cur += 1;
        }
        out[k] = jobj;
        prev = cur;
        if !jobj.is_null() {
            found += 1;
        }
        k += 1;
        i -= 1;
    }
    found
}

/// lb_00CE.c `lb_8000D148`: is point 2 within `threshold` of the segment point0..point1
/// (distance to the line, then the endpoint cases)? Five fused sites, as the shipping build.
pub fn lb_8000D148(point0_x: f32, point0_y: f32, point1_x: f32, point1_y: f32, point2_x: f32, point2_y: f32,
                   threshold: f32) -> i32 {
    let mut dist_01;
    let mut var_f0;
    {
        let diff_01_x = point1_x - point0_x;
        let diff_01_y = point0_y - point1_y;
        let mut cross = point0_y * point1_x;
        dist_01 = fmadds(diff_01_x, diff_01_x, diff_01_y * diff_01_y);
        cross = fmadds(point0_x, point1_y, -cross);
        if dist_01 < 0.00001 {
            return 0;
        }
        dist_01 = sqrtf(dist_01);
        var_f0 = cross + fmadds(diff_01_x, point2_x, diff_01_y * point2_y);
        if var_f0 < 0.0 {
            var_f0 = -var_f0;
        }
    }
    if var_f0 / dist_01 <= threshold {
        let dx0 = point0_x - point2_x;
        let dy0 = point0_y - point2_y;
        let dist_squared_02 = fmadds(dx0, dx0, dy0 * dy0);
        let threshold_squared = threshold * threshold;
        let dx1 = point1_x - point2_x;
        let dy1 = point1_y - point2_y;
        let dist_squared_12 = fmadds(dx1, dx1, dy1 * dy1);
        if dist_squared_02 < threshold_squared {
            if dist_squared_12 > threshold_squared {
                return 1;
            }
            if dist_squared_12 < threshold_squared {
                return 0;
            }
            return 1;
        }
        if dist_squared_02 > threshold_squared {
            if dist_squared_12 > threshold_squared {
                if (point0_x > point2_x && point1_x < point2_x)
                    || (point0_x < point2_x && point1_x > point2_x)
                    || (point0_y > point2_y && point1_y < point2_y)
                    || (point0_y < point2_y && point1_y > point2_y)
                {
                    return 1;
                }
                return 0;
            }
            return 1;
        }
        return 1;
    }
    0
}

/// lb_00F9.c `groundHeight` (static inline).
pub fn groundHeight(data: &DynamicsData, floor_point: &Vec3) -> f32 {
    // SAFETY: plain-data union; the C reads this member.
    let y = unsafe { data.desc.lb_unk0.unk_2C.y };
    absf(y - floor_point.y)
}

/// lb_020A.c `calc_acos` (static inline): group 1's `acosf`.
pub fn calc_acos(value: f32) -> f32 {
    melee_runtime::lbtrigf::acosf(value)
}

/// lbshadow.c `lbShadow_8000E9F0`: the tangent of `spline` at `u` in [0, 1] (no-op outside),
/// by type: 0 linear, 1 Bezier, 2 B-spline, 3 cardinal; any other type leaves `p` alone.
pub fn lbShadow_8000E9F0(p: &mut Vec3, spline: &HSD_Spline, mut u: f32) {
    if u < 0.0 || u > 1.0 {
        return;
    }
    let orig_u = u;
    u *= (spline.numcv as i32 - 1) as f32;
    let mut idx = u as i16; // (s16) u: u is in [0, numcv - 1] here
    u -= idx as f32;
    // SAFETY: cv holds numcv control points (3*numcv-2 for Bezier); the C indexes it unchecked.
    let cps = |at: isize| unsafe { core::slice::from_raw_parts(spline.cv.offset(at), 4) };
    match spline.r#type {
        0 => {
            if orig_u == 1.0 {
                idx -= 1;
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
