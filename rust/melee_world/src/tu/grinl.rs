//! Header inlines the gr TUs share: `HSD_JObj*` accessors of sysdolphin/baselib/jobj.h, the
//! `GET_GROUND` / `GET_JOBJ` macros and the `Ground_*` inlines of src/melee/gr/inlines.h that more
//! than one stage file uses (compiled into each C TU, so each Rust TU calls these).
//!
//! The only callee is `HSD_JObjSetMtxDirtySub` (the body of the `HSD_JObjSetMtxDirty` macro). An
//! unset hook panics, like every `ext` TODO stub. `HSD_ASSERT(line, cond)` is `__assert(...)`, which
//! never returns: [`assert_`] panics.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

use core::ffi::c_void;
use melee_types::*;

pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use melee_types::*;
    hook_or!(HSD_JOBJSETMTXDIRTYSUB, HSD_JObjSetMtxDirtySub, (jobj: *mut HSD_JObj), |jobj: *mut HSD_JObj| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(jobj as _); } });
    // rp3 HSD_JObjSetMtxDirty
    hook_or!(HSD_JOBJSETMTXDIRTY, HSD_JObjSetMtxDirty, (jobj: *mut HSD_JObj), |jobj: *mut HSD_JObj| unsafe { crate::tu::grinl::HSD_JObjSetMtxDirty(jobj); });
    // rp3-end

    hook!(HSD_JOBJSETUPMATRIXSUB, HSD_JObjSetupMatrixSub, (jobj: *mut HSD_JObj));
    // The C global `stage_info` (gr/ground.c, 49E6C8): its address.
    hook!(STAGE_INFO, stage_info, () -> *mut StageInfo);
}

/// `HSD_ASSERT(line, cond)` (the string is what the C prints).
#[inline]
#[track_caller]
pub fn assert_(cond: bool, line: u32, what: &str) {
    if !cond {
        panic!("__assert: line {line}: {what}");
    }
}

pub const M_PI: f64 = 3.14159265358979323846;
pub const M_TAU: f64 = 6.283185307179586;

/// `GET_GROUND(gobj)`.
#[inline(always)]
pub unsafe fn GET_GROUND(g: *mut HSD_GObj) -> *mut Ground {
    (*g).user_data as *mut Ground
}
/// `GET_JOBJ(gobj)` (`HSD_GObjGetHSDObj`).
#[inline(always)]
pub unsafe fn GET_JOBJ(g: *mut HSD_GObj) -> *mut HSD_JObj {
    (*g).hsd_obj as *mut HSD_JObj
}

// ---------------------------------------------------------------- walking
#[inline]
pub unsafe fn HSD_JObjGetChild(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() { core::ptr::null_mut() } else { (*j).child }
}
#[inline]
pub unsafe fn HSD_JObjGetNext(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() { core::ptr::null_mut() } else { (*j).next }
}
#[inline]
pub unsafe fn HSD_JObjGetParent(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() { core::ptr::null_mut() } else { (*j).parent }
}

/// `HSD_JObjMtxIsDirty`.
#[inline]
pub unsafe fn HSD_JObjMtxIsDirty(j: *mut HSD_JObj) -> bool {
    assert_(!j.is_null(), 564, "jobj");
    (*j).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*j).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// The `HSD_JObjSetMtxDirty(jobj)` macro.
#[inline]
pub unsafe fn HSD_JObjSetMtxDirty(j: *mut HSD_JObj) {
    if !j.is_null() && !HSD_JObjMtxIsDirty(j) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

#[inline]
unsafe fn dirty_unless_indep(j: *mut HSD_JObj) {
    if (*j).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(j);
    }
}

// ---------------------------------------------------------------- rotation
pub unsafe fn HSD_JObjSetRotation(j: *mut HSD_JObj, r: *const Quaternion) {
    assert_(!j.is_null(), 618, "jobj");
    assert_(!r.is_null(), 619, "rotate");
    (*j).rotate = *r;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjSetRotationX(j: *mut HSD_JObj, x: f32) {
    assert_(!j.is_null(), 639, "jobj");
    assert_((*j).flags & JOBJ_USE_QUATERNION as u32 == 0, 640, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*j).rotate.x = x;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjSetRotationY(j: *mut HSD_JObj, y: f32) {
    assert_(!j.is_null(), 660, "jobj");
    assert_((*j).flags & JOBJ_USE_QUATERNION as u32 == 0, 661, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*j).rotate.y = y;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjSetRotationZ(j: *mut HSD_JObj, z: f32) {
    assert_(!j.is_null(), 681, "jobj");
    assert_((*j).flags & JOBJ_USE_QUATERNION as u32 == 0, 682, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*j).rotate.z = z;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjGetRotation(j: *mut HSD_JObj, r: *mut Quaternion) {
    assert_(!j.is_null(), 699, "jobj");
    assert_(!r.is_null(), 700, "rotate");
    *r = (*j).rotate;
}
pub unsafe fn HSD_JObjGetRotationX(j: *mut HSD_JObj) -> f32 {
    assert_(!j.is_null(), 715, "jobj");
    (*j).rotate.x
}
pub unsafe fn HSD_JObjGetRotationY(j: *mut HSD_JObj) -> f32 {
    assert_(!j.is_null(), 730, "jobj");
    (*j).rotate.y
}
pub unsafe fn HSD_JObjGetRotationZ(j: *mut HSD_JObj) -> f32 {
    assert_(!j.is_null(), 745, "jobj");
    (*j).rotate.z
}

// ---------------------------------------------------------------- scale
pub unsafe fn HSD_JObjSetScale(j: *mut HSD_JObj, s: *const Vec3) {
    assert_(!j.is_null(), 760, "jobj");
    assert_(!s.is_null(), 761, "scale");
    (*j).scale = *s;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjSetScaleX(j: *mut HSD_JObj, x: f32) {
    assert_(!j.is_null(), 776, "jobj");
    (*j).scale.x = x;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjSetScaleY(j: *mut HSD_JObj, y: f32) {
    assert_(!j.is_null(), 791, "jobj");
    (*j).scale.y = y;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjSetScaleZ(j: *mut HSD_JObj, z: f32) {
    assert_(!j.is_null(), 806, "jobj");
    (*j).scale.z = z;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjGetScale(j: *mut HSD_JObj, s: *mut Vec3) {
    assert_(!j.is_null(), 823, "jobj");
    assert_(!s.is_null(), 824, "scale");
    *s = (*j).scale;
}
pub unsafe fn HSD_JObjGetScaleX(j: *mut HSD_JObj) -> f32 {
    assert_(!j.is_null(), 875, "jobj");
    (*j).scale.x
}
pub unsafe fn HSD_JObjGetScaleY(j: *mut HSD_JObj) -> f32 {
    assert_(!j.is_null(), 888, "jobj");
    (*j).scale.y
}
pub unsafe fn HSD_JObjGetScaleZ(j: *mut HSD_JObj) -> f32 {
    assert_(!j.is_null(), 901, "jobj");
    (*j).scale.z
}

// ---------------------------------------------------------------- translation
pub unsafe fn HSD_JObjSetTranslate(j: *mut HSD_JObj, t: *const Vec3) {
    assert_(!j.is_null(), 916, "jobj");
    assert_(!t.is_null(), 917, "translate");
    (*j).translate = *t;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjSetTranslateX(j: *mut HSD_JObj, x: f32) {
    assert_(!j.is_null(), 932, "jobj");
    (*j).translate.x = x;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjSetTranslateY(j: *mut HSD_JObj, y: f32) {
    assert_(!j.is_null(), 947, "jobj");
    (*j).translate.y = y;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjSetTranslateZ(j: *mut HSD_JObj, z: f32) {
    assert_(!j.is_null(), 962, "jobj");
    (*j).translate.z = z;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjGetTranslation(j: *mut HSD_JObj, t: *mut Vec3) {
    assert_(!j.is_null(), 977, "jobj");
    assert_(!t.is_null(), 978, "translate");
    *t = (*j).translate;
}
pub unsafe fn HSD_JObjGetTranslationX(j: *mut HSD_JObj) -> f32 {
    assert_(!j.is_null(), 990, "jobj");
    (*j).translate.x
}
pub unsafe fn HSD_JObjGetTranslationY(j: *mut HSD_JObj) -> f32 {
    assert_(!j.is_null(), 1003, "jobj");
    (*j).translate.y
}
pub unsafe fn HSD_JObjGetTranslationZ(j: *mut HSD_JObj) -> f32 {
    assert_(!j.is_null(), 1016, "jobj");
    (*j).translate.z
}

pub unsafe fn HSD_JObjGetTranslation2(j: *mut HSD_JObj, t: *mut Vec3) {
    assert_(!j.is_null(), 979, "jobj");
    *t = (*j).translate;
}

// ---------------------------------------------------------------- Add*
pub unsafe fn HSD_JObjAddRotationX(j: *mut HSD_JObj, x: f32) {
    assert_(!j.is_null(), 1029, "jobj");
    (*j).rotate.x += x;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjAddRotationY(j: *mut HSD_JObj, y: f32) {
    assert_(!j.is_null(), 1041, "jobj");
    (*j).rotate.y += y;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjAddRotationZ(j: *mut HSD_JObj, z: f32) {
    assert_(!j.is_null(), 1053, "jobj");
    (*j).rotate.z += z;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjAddScaleX(j: *mut HSD_JObj, x: f32) {
    assert_(!j.is_null(), 1065, "jobj");
    (*j).scale.x += x;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjAddScaleY(j: *mut HSD_JObj, y: f32) {
    assert_(!j.is_null(), 1077, "jobj");
    (*j).scale.y += y;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjAddScaleZ(j: *mut HSD_JObj, z: f32) {
    assert_(!j.is_null(), 1089, "jobj");
    (*j).scale.z += z;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjAddTranslationX(j: *mut HSD_JObj, x: f32) {
    assert_(!j.is_null(), 1102, "jobj");
    (*j).translate.x += x;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjAddTranslationY(j: *mut HSD_JObj, y: f32) {
    assert_(!j.is_null(), 1114, "jobj");
    (*j).translate.y += y;
    dirty_unless_indep(j);
}
pub unsafe fn HSD_JObjAddTranslationZ(j: *mut HSD_JObj, z: f32) {
    assert_(!j.is_null(), 1126, "jobj");
    (*j).translate.z += z;
    dirty_unless_indep(j);
}

/// A pointer stored in a `u32` field (`gp->u.castle11.xD8 = (u32) gobj`). Lossless only while the process lives below
/// 4 GiB, which the stage oracle tests arrange (build.rs); the game needs a handle table before it can run natively.
#[inline(always)]
pub fn ptr_u32<T>(p: *mut T) -> u32 {
    debug_assert!((p as usize) <= u32::MAX as usize, "pointer does not fit a u32 field");
    p as usize as u32
}
/// The inverse of [`ptr_u32`] (`(HSD_GObj*) gp->u.castle11.xD8`).
#[inline(always)]
pub fn u32_ptr<T>(v: u32) -> *mut T {
    v as usize as *mut T
}

/// `HSD_JObjSetTranslateWithMtxDirty`: like `HSD_JObjSetTranslate` but calls the out-of-line `HSD_JObjSetMtxDirty`.
pub unsafe fn HSD_JObjSetTranslateWithMtxDirty(j: *mut HSD_JObj, t: *const Vec3) {
    assert_(!j.is_null(), 916, "jobj");
    assert_(!t.is_null(), 917, "translate");
    (*j).translate = *t;
    if (*j).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        ext::HSD_JObjSetMtxDirty(j);
    }
}

/// `HSD_JObjSetupMatrix`.
pub unsafe fn HSD_JObjSetupMatrix(j: *mut HSD_JObj) {
    if j.is_null() || !HSD_JObjMtxIsDirty(j) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(j);
}

/// `ABS(x)` of Runtime/platform.h: `(x) < 0 ? -(x) : (x)`.
#[inline(always)]
pub fn ABS(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

#[allow(dead_code)]
pub(crate) type Gobj = *mut HSD_GObj;
#[allow(dead_code)]
pub(crate) type Vp = *mut c_void;
