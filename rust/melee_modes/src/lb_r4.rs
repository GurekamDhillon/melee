//! Round 4: lb callers of rounds 1-3 and of group 1's ported code (lbtrigf `acosf`, MSL
//! `sqrtf`). Verified bit for bit against the C (`tests/oracle_r4.rs`).
//! TODO(group1): these lb TUs are lb leftovers; move into melee_runtime if group 1 takes them.
//!
//! FMA (`tools/rust-port/fma_sites.py`): `lb_8000D148` has the shipping build's 5 fused sites,
//! each an `fmadds` below; nothing else here fuses.
#![allow(non_snake_case)]

use crate::lb::{absf, calcPan};
use melee_core::fp::fmadds;
use melee_runtime::gekko::sqrtf;
use melee_types as mt;
use mt::{CameraBlurData, DynamicsData, HSD_GObj, HSD_GObjEvent, HSD_JObj, Vec as Vec3};

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

pub use melee_runtime::tu::lb_00CE::lb_8000D148;

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

// Compatibility export: the implementation lives with its TU in runtime.
pub use melee_runtime::tu::lbshadow::lbShadow_8000E9F0;
