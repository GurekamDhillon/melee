//! More of `src/sysdolphin/baselib/cobj.c` on the GameCube-layout record `HSD_CObj` (raw
//! pointers, the C names and argument shapes), the getters / setters the stage and HUD TUs call:
//! flags, fov, aspect (TARGET_PC `view_auto` bookkeeping), perspective, scissor, viewportf,
//! projection type, and the eye / interest positions (the WObj position, assert on a null CObj)
//! with `HSD_CObjGetEyeVector`. The vector helpers are the verified ones of [`crate::cobj`] /
//! [`crate::mtx`]. Checked against the C by `tests/pure_cobj_world.rs`
//! (`python rust/melee_runtime/gen/pure/run_pure.py cobj_world`).
#![allow(non_snake_case, clippy::missing_safety_doc)]

use super::cobj::HSD_CObjUpdateView;
use super::debug::__assert;
use super::wobj::{HSD_WObjGetPosition, HSD_WObjSetPosition};
use crate::mtx::{self, Vec3};
use melee_types::{HSD_CObj, _HSD_RectF32, _Scissor, Vec};

const PROJ_PERSPECTIVE: u8 = 1;

unsafe fn assert_cobj(line: u32, cobj: *mut HSD_CObj) {
    if cobj.is_null() {
        __assert(c"cobj.c".as_ptr(), line, c"cobj".as_ptr());
    }
}

/// `HSD_CObjGetFlags`.
pub unsafe fn HSD_CObjGetFlags(cobj: *mut HSD_CObj) -> u32 {
    (*cobj).flags
}

/// `HSD_CObjGetFov`.
pub unsafe fn HSD_CObjGetFov(cobj: *mut HSD_CObj) -> f32 {
    if cobj.is_null() || (*cobj).projection_type != PROJ_PERSPECTIVE {
        return 0.0;
    }
    (*cobj).projection_param.perspective.fov
}

/// `HSD_CObjGetAspect` (TARGET_PC: refreshes an auto view first).
pub unsafe fn HSD_CObjGetAspect(cobj: *mut HSD_CObj) -> f32 {
    HSD_CObjUpdateView(cobj);
    if cobj.is_null() || (*cobj).projection_type != PROJ_PERSPECTIVE {
        return 0.0;
    }
    (*cobj).projection_param.perspective.aspect
}

/// `HSD_CObjGetAuthoredAspect`.
pub unsafe fn HSD_CObjGetAuthoredAspect(cobj: *mut HSD_CObj) -> f32 {
    if !cobj.is_null() && (*cobj).view_auto != 0 {
        return (*cobj).view_authored_aspect;
    }
    HSD_CObjGetAspect(cobj)
}

/// `HSD_CObjSetAspect`.
pub unsafe fn HSD_CObjSetAspect(cobj: *mut HSD_CObj, aspect: f32) {
    if cobj.is_null() || (*cobj).projection_type != PROJ_PERSPECTIVE {
        return;
    }
    (*cobj).projection_param.perspective.aspect = aspect;
    if (*cobj).view_auto != 0 {
        (*cobj).view_authored_aspect = aspect;
        HSD_CObjUpdateView(cobj);
    }
}

/// `HSD_CObjGetPerspective`: `*top = fov`, `*bottom = aspect` (either may be NULL).
pub unsafe fn HSD_CObjGetPerspective(cobj: *mut HSD_CObj, top: *mut f32, bottom: *mut f32) {
    HSD_CObjUpdateView(cobj);
    if cobj.is_null() || (*cobj).projection_type != PROJ_PERSPECTIVE {
        return;
    }
    if !top.is_null() {
        *top = (*cobj).projection_param.perspective.fov;
    }
    if !bottom.is_null() {
        *bottom = (*cobj).projection_param.perspective.aspect;
    }
}

/// `HSD_CObjGetScissor`.
pub unsafe fn HSD_CObjGetScissor(cobj: *mut HSD_CObj, scissor: *mut _Scissor) {
    if cobj.is_null() {
        return;
    }
    *scissor = (*cobj).scissor;
}

/// `HSD_CObjGetViewportf`.
pub unsafe fn HSD_CObjGetViewportf(cobj: *mut HSD_CObj, viewport: *mut _HSD_RectF32) {
    if cobj.is_null() {
        return;
    }
    *viewport = (*cobj).viewport;
}

/// `HSD_CObjSetViewportf`.
pub unsafe fn HSD_CObjSetViewportf(cobj: *mut HSD_CObj, viewport: *mut _HSD_RectF32) {
    if cobj.is_null() {
        return;
    }
    (*cobj).viewport = *viewport;
}

/// `HSD_CObjSetProjectionType`.
pub unsafe fn HSD_CObjSetProjectionType(cobj: *mut HSD_CObj, proj_type: u32) {
    if cobj.is_null() {
        return;
    }
    (*cobj).projection_type = proj_type as u8;
}

/// `HSD_CObjGetInterest`.
pub unsafe fn HSD_CObjGetInterest(cobj: *mut HSD_CObj, interest: *mut Vec) {
    assert_cobj(709, cobj);
    HSD_WObjGetPosition((*cobj).interest, interest);
}

/// `HSD_CObjSetInterest`.
pub unsafe fn HSD_CObjSetInterest(cobj: *mut HSD_CObj, interest: *mut Vec) {
    assert_cobj(721, cobj);
    HSD_WObjSetPosition((*cobj).interest, interest);
}

/// `HSD_CObjGetEyePosition`.
pub unsafe fn HSD_CObjGetEyePosition(cobj: *mut HSD_CObj, position: *mut Vec) {
    assert_cobj(733, cobj);
    HSD_WObjGetPosition((*cobj).eyepos, position);
}

/// `HSD_CObjSetEyePosition`.
pub unsafe fn HSD_CObjSetEyePosition(cobj: *mut HSD_CObj, position: *mut Vec) {
    assert_cobj(745, cobj);
    HSD_WObjSetPosition((*cobj).eyepos, position);
}

/// `HSD_CObjGetEyeVector`: 0 and the unit view direction, or -1 and (0, 0, -1).
pub unsafe fn HSD_CObjGetEyeVector(cobj: *mut HSD_CObj, eye: *mut Vec) -> i32 {
    if !cobj.is_null() && !(*cobj).eyepos.is_null() && !(*cobj).interest.is_null() && !eye.is_null() {
        let mut eyepos: Vec = core::mem::zeroed();
        let mut interest: Vec = core::mem::zeroed();
        HSD_CObjGetEyePosition(cobj, &mut eyepos);
        HSD_CObjGetInterest(cobj, &mut interest);
        let v3 = |v: &Vec| Vec3 { x: v.x, y: v.y, z: v.z };
        let d = mtx::vec_sub(&v3(&interest), &v3(&eyepos));
        (*eye).x = d.x;
        (*eye).y = d.y;
        (*eye).z = d.z;
        let mut out = d;
        if crate::cobj::vec_normalize_check(&d, &mut out) == 0 {
            (*eye).x = out.x;
            (*eye).y = out.y;
            (*eye).z = out.z;
            return 0;
        }
    }
    if !eye.is_null() {
        (*eye).x = 0.0;
        (*eye).y = 0.0;
        (*eye).z = -1.0;
    }
    -1
}
