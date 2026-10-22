//! The HSD header inlines (src/sysdolphin/baselib/jobj.h, gobj.h) that every gm TU compiles in.
//!
//! In C these are `static inline` (or macros) so they are not callees: their bodies - including
//! the `HSD_ASSERT` calls and the `HSD_JObjSetMtxDirty` macro's own dirty test - run inside the
//! caller. They are ported once here, calling the same out-of-line functions the C does
//! (`HSD_JObjSetMtxDirtySub`, `HSD_JObjSetMtxDirty`, `__assert`) through `ext`.
//!
//! Bare `HSD_JObjSetMtxDirty(jobj)` in the C is the macro (null + dirty test, then
//! `HSD_JObjSetMtxDirtySub`); `(HSD_JObjSetMtxDirty)(jobj)` is the real function. The `...WithMtxDirty`
//! variants use the latter. `HSD_ASSERT`'s line argument is the header line in the real build
//! (`__LINE__`); the asserting path is never taken by well-formed state, so the macro argument is
//! passed.
#![allow(non_snake_case, clippy::missing_safety_doc)]

use core::ffi::c_void;
use melee_types::{HSD_GObj, HSD_JObj, Quaternion, Vec3, JOBJ_MTX_DIRTY, JOBJ_MTX_INDEP_SRT, JOBJ_USER_DEF_MTX, JOBJ_USE_QUATERNION};

pub mod ext {
    use super::*;
    crate::ext_fns! {
        /// jobj.c
        fn HSD_JObjSetMtxDirtySub(jobj: *mut HSD_JObj) -> ();
        /// jobj.c (the real function behind the parenthesised macro name)
        fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) -> ();
        /// debug.c
        fn __assert(file: *const i8, line: u32, msg: *const i8) -> ();
        // ---- gr/inlines.h: Ground_InitScene's callees
        fn Camera_Init(a0: i32) -> ();
        fn lb_8000FCDC() -> ();
        fn mpColl_80041C78() -> ();
        fn Ground_801C0378(a0: i32) -> ();
        fn Stage_802251E8(stkind: u32, a1: *mut i32) -> ();
        // ---- mn/inlines.h: the menu sounds
        fn lbAudioAx_80024030(a0: i32) -> ();
    }
}

/// mn/inlines.h `sfxBack` / `sfxForward` / `sfxMove`.
pub unsafe fn sfxBack() {
    ext::lbAudioAx_80024030(0);
}
pub unsafe fn sfxForward() {
    ext::lbAudioAx_80024030(1);
}
pub unsafe fn sfxMove() {
    ext::lbAudioAx_80024030(2);
}

/// gr/inlines.h `Ground_InitScene`.
pub unsafe fn Ground_InitScene() {
    ext::Camera_Init(6);
    ext::lb_8000FCDC();
    ext::mpColl_80041C78();
    ext::Ground_801C0378(0x40);
}

/// gr/inlines.h `Stage_InitScene`.
pub unsafe fn Stage_InitScene(stkind: u32, arg1: *mut i32) {
    Ground_InitScene();
    ext::Stage_802251E8(stkind, arg1);
}

/// `HSD_ASSERT(line, cond)`.
#[inline]
pub unsafe fn hsd_assert(cond: bool, line: u32, what: &'static core::ffi::CStr) {
    if !cond {
        ext::__assert(c"jobj.h".as_ptr(), line, what.as_ptr());
    }
}

/// `GET_JOBJ(gobj)` / `GET_COBJ` / `GET_LOBJ`: `HSD_GObjGetHSDObj(gobj)`.
#[inline]
pub unsafe fn get_jobj(g: *mut HSD_GObj) -> *mut HSD_JObj {
    (*g).hsd_obj.cast()
}
#[inline]
pub unsafe fn get_hsd_obj(g: *mut HSD_GObj) -> *mut c_void {
    (*g).hsd_obj
}
/// `HSD_JObjGetChild`.
#[inline]
pub unsafe fn jobj_child(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() { core::ptr::null_mut() } else { (*j).child }
}
/// `HSD_JObjGetNext`.
#[inline]
pub unsafe fn jobj_next(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() { core::ptr::null_mut() } else { (*j).next }
}
/// `HSD_JObjGetParent`.
#[inline]
pub unsafe fn jobj_parent(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() { core::ptr::null_mut() } else { (*j).parent }
}

/// `HSD_JObjMtxIsDirty`.
pub unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    hsd_assert(!jobj.is_null(), 564, c"jobj");
    let mut result = false;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0 {
        result = true;
    }
    result
}

/// The bare `HSD_JObjSetMtxDirty(jobj)` macro.
pub unsafe fn set_mtx_dirty_macro(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `HSD_JObjSetupMatrix`-style dirty hook used by the setters: the macro or the function.
#[inline]
unsafe fn dirty(jobj: *mut HSD_JObj, with_fn: bool) {
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        if with_fn {
            ext::HSD_JObjSetMtxDirty(jobj);
        } else {
            set_mtx_dirty_macro(jobj);
        }
    }
}

macro_rules! scalar_setter {
    ($($name:ident, $name_w:ident, $field:ident . $axis:ident, $line:literal, $quat:literal;)*) => {$(
        pub unsafe fn $name(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert(!jobj.is_null(), $line, c"jobj");
            if $quat { hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, $line + 1, c"!(jobj->flags & JOBJ_USE_QUATERNION)"); }
            (*jobj).$field.$axis = v;
            dirty(jobj, false);
        }
        pub unsafe fn $name_w(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert(!jobj.is_null(), $line, c"jobj");
            if $quat { hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, $line + 1, c"!(jobj->flags & JOBJ_USE_QUATERNION)"); }
            (*jobj).$field.$axis = v;
            dirty(jobj, true);
        }
    )*};
}
scalar_setter! {
    HSD_JObjSetRotationX, HSD_JObjSetRotationXWithMtxDirty, rotate.x, 639, true;
    HSD_JObjSetRotationY, HSD_JObjSetRotationYWithMtxDirty, rotate.y, 660, true;
    HSD_JObjSetRotationZ, HSD_JObjSetRotationZWithMtxDirty, rotate.z, 681, true;
    HSD_JObjSetScaleX, HSD_JObjSetScaleXWithMtxDirty, scale.x, 776, false;
    HSD_JObjSetScaleY, HSD_JObjSetScaleYWithMtxDirty, scale.y, 791, false;
    HSD_JObjSetScaleZ, HSD_JObjSetScaleZWithMtxDirty, scale.z, 806, false;
    HSD_JObjSetTranslateX, HSD_JObjSetTranslateXWithMtxDirty, translate.x, 932, false;
    HSD_JObjSetTranslateY, HSD_JObjSetTranslateYWithMtxDirty, translate.y, 947, false;
    HSD_JObjSetTranslateZ, HSD_JObjSetTranslateZWithMtxDirty, translate.z, 962, false;
}

/// `HSD_JObjSetRotation` / `...WithMtxDirty`.
pub unsafe fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, rotate: *const Quaternion) {
    hsd_assert(!jobj.is_null(), 618, c"jobj");
    hsd_assert(!rotate.is_null(), 619, c"rotate");
    (*jobj).rotate = *rotate;
    dirty(jobj, false);
}
pub unsafe fn HSD_JObjSetRotationWithMtxDirty(jobj: *mut HSD_JObj, rotate: *const Quaternion) {
    hsd_assert(!jobj.is_null(), 618, c"jobj");
    hsd_assert(!rotate.is_null(), 619, c"rotate");
    (*jobj).rotate = *rotate;
    dirty(jobj, true);
}
/// `HSD_JObjSetScale` / `...WithMtxDirty`.
pub unsafe fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *const Vec3) {
    hsd_assert(!jobj.is_null(), 760, c"jobj");
    hsd_assert(!scale.is_null(), 761, c"scale");
    (*jobj).scale = *scale;
    dirty(jobj, false);
}
pub unsafe fn HSD_JObjSetScaleWithMtxDirty(jobj: *mut HSD_JObj, scale: *const Vec3) {
    hsd_assert(!jobj.is_null(), 760, c"jobj");
    hsd_assert(!scale.is_null(), 761, c"scale");
    (*jobj).scale = *scale;
    dirty(jobj, true);
}
/// `HSD_JObjSetTranslate` / `...WithMtxDirty`.
pub unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *const Vec3) {
    hsd_assert(!jobj.is_null(), 916, c"jobj");
    hsd_assert(!translate.is_null(), 917, c"translate");
    (*jobj).translate = *translate;
    dirty(jobj, false);
}
pub unsafe fn HSD_JObjSetTranslateWithMtxDirty(jobj: *mut HSD_JObj, translate: *const Vec3) {
    hsd_assert(!jobj.is_null(), 916, c"jobj");
    hsd_assert(!translate.is_null(), 917, c"translate");
    (*jobj).translate = *translate;
    dirty(jobj, true);
}

/// Getters (`HSD_JObjGetRotationX/Y/Z`, `...ScaleX/Y/Z`, `...TranslationX/Y/Z`).
macro_rules! scalar_getter {
    ($($name:ident, $field:ident . $axis:ident, $line:literal;)*) => {$(
        pub unsafe fn $name(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(!jobj.is_null(), $line, c"jobj");
            (*jobj).$field.$axis
        }
    )*};
}
scalar_getter! {
    HSD_JObjGetRotationX, rotate.x, 715;
    HSD_JObjGetRotationY, rotate.y, 730;
    HSD_JObjGetRotationZ, rotate.z, 745;
    HSD_JObjGetScaleX, scale.x, 875;
    HSD_JObjGetScaleY, scale.y, 888;
    HSD_JObjGetScaleZ, scale.z, 901;
}
