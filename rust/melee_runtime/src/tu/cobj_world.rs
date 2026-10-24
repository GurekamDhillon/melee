//! More of `src/sysdolphin/baselib/cobj.c` on the GameCube-layout record `HSD_CObj` (raw
//! pointers, the C names and argument shapes), the getters / setters the stage and HUD TUs call:
//! flags, fov, aspect (TARGET_PC `view_auto` bookkeeping), perspective, scissor, viewportf,
//! projection type, and the eye / interest positions (the WObj position, assert on a null CObj)
//! with `HSD_CObjGetEyeVector`. The vector helpers are the verified ones of [`crate::cobj`] /
//! [`crate::mtx`]. Checked against the C by `tests/pure_cobj_world.rs`
//! (`python rust/melee_runtime/gen/pure/run_pure.py cobj_world`). Also the up-vector / roll pair (`HSD_CObjGetUpVector`,
//! `SetUpVector`, `SetRoll`, with the static `upvec2roll` / `roll2upvec`), the viewing matrix (`GetViewingMtxPtr`,
//! `GetInvViewingMtxPtr[Direct]`) and `SetScissorx4` / `SetViewportfx4` / `GetOrtho`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::cobj::HSD_CObjUpdateView;
use super::debug::__assert;
use super::wobj::{HSD_WObjGetPosition, HSD_WObjSetPosition};
use crate::mtx::{self, Vec3};
use melee_types::{HSD_CObj, _HSD_RectF32, _Scissor, Vec};

const PROJ_PERSPECTIVE: u8 = 1;
const PROJ_ORTHO: u8 = 3;
use core::ffi::c_void;
use crate::gekko::sqrtf;
use melee_core::fp::fmadds;

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

// --- up vector, roll, viewing matrix ------------------------------------------------------------------

#[inline]
fn n3(v: &Vec) -> Vec3 {
    Vec3 { x: v.x, y: v.y, z: v.z }
}
#[inline]
fn g3(v: Vec3) -> Vec {
    Vec { x: v.x, y: v.y, z: v.z }
}

/// util.h `vec_normalize_check` on raw vectors: -1 (`dst` untouched) if every component is within
/// `FLT_MIN` of 0, else `*dst = normalize(*src)` and 0.
unsafe fn vec_normalize_check(src: *const Vec, dst: *mut Vec) -> i32 {
    let mut d = n3(&*dst);
    let r = crate::cobj::vec_normalize_check(&n3(&*src), &mut d);
    if r == 0 {
        *dst = g3(d);
    }
    r
}

/// Callees of cobj.c in other TUs that this part reaches (the oracle supplies the C `HSD_MtxAlloc` pool).
pub mod ext {
    use melee_types::Mtx;
    /// `HSD_MtxAlloc` (mtx.c); unset runs the pooled port.
    pub static mut HSD_MtxAlloc: Option<unsafe fn() -> *mut Mtx> = None;
    pub unsafe fn mtx_alloc() -> *mut Mtx {
        match HSD_MtxAlloc {
            Some(f) => f(),
            None => crate::tu::mtx::HSD_MtxAlloc() as *mut Mtx,
        }
    }
}

const FLT_MIN_: f32 = crate::cobj::FLT_MIN;
const COBJ_UP_VECTOR_: u32 = 1;

/// `upvec2roll` (static).
unsafe fn upvec2roll(cobj: *mut HSD_CObj, up: *mut Vec) -> f32 {
    let mut eye: Vec = core::mem::zeroed();
    if HSD_CObjGetEyeVector(cobj, &mut eye) != 0 {
        return 0.0;
    }
    let mut dot = mtx::vec_dot(&n3(&*up), &n3(&eye)).abs();
    dot = 1.0 - dot;
    if dot < FLT_MIN_ {
        return 0.0;
    }
    let orig = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let uy = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    let vmtx = mtx::look_at(&orig, &uy, &n3(&eye));
    let v = mtx::mult_vec_sr(&vmtx, &n3(&*up));
    if crate::hsd_mtx::fabsf_bitwise(v.y) == 0.0 {
        if -v.x >= 0.0 { 1.5707963267948966f64 as f32 } else { -1.5707963267948966f64 as f32 }
    } else {
        crate::lbtrigf::atan2f(-v.x, v.y)
    }
}

/// `roll2upvec` (static): 0 and `*up` written, or the nonzero `HSD_CObjGetEyeVector` result with `up` untouched.
unsafe fn roll2upvec(cobj: *mut HSD_CObj, up: *mut Vec, roll: f32) -> i32 {
    let mut eye: Vec = core::mem::zeroed();
    let res = HSD_CObjGetEyeVector(cobj, &mut eye);
    if res != 0 {
        return res;
    }
    let mut v0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if 1.0 - (eye.y.abs() as f64) < 0.0001 {
        v0.x = crate::gekko::sqrtf(fmadds(eye.y, eye.y, eye.z * eye.z));
        v0.y = eye.y * (-eye.x / v0.x);
        v0.z = eye.z * (-eye.x / v0.x);
    } else {
        v0.y = crate::gekko::sqrtf(fmadds(eye.x, eye.x, eye.z * eye.z));
        v0.x = eye.x * (-eye.y / v0.y);
        v0.z = eye.z * (-eye.y / v0.y);
    }
    let m = mtx::rot_axis_rad(&n3(&eye), -roll);
    let v1 = mtx::mult_vec_sr(&m, &v0);
    *up = g3(mtx::vec_normalize(&v1));
    0
}

/// `HSD_CObjGetUpVector`: 0 and the up vector, or -1 and (0, 1, 0).
pub unsafe fn HSD_CObjGetUpVector(cobj: *mut HSD_CObj, up: *mut Vec) -> i32 {
    if !cobj.is_null() && !up.is_null() {
        if (*cobj).flags & COBJ_UP_VECTOR_ != 0 {
            *up = (*cobj).u.up;
            return 0;
        }
        if roll2upvec(cobj, up, (*cobj).u.roll) == 0 {
            return 0;
        }
    }
    if !up.is_null() {
        (*up).x = 0.0;
        (*up).y = 1.0;
        (*up).z = 0.0;
    }
    -1
}

/// `HSD_CObjSetUpVector`.
pub unsafe fn HSD_CObjSetUpVector(cobj: *mut HSD_CObj, mut up: *mut Vec) {
    if cobj.is_null() || up.is_null() {
        return;
    }
    if (*cobj).flags & COBJ_UP_VECTOR_ != 0 {
        let mut v: Vec = core::mem::zeroed();
        let mut uy2 = Vec { x: 0.0, y: 1.0, z: 0.0 };
        up = if vec_normalize_check(up, &mut v) != 0 { &mut uy2 } else { &mut v };
        if (*cobj).u.up.x != (*up).x || (*cobj).u.up.y != (*up).y || (*cobj).u.up.z != (*up).z {
            super::cobj::HSD_CObjSetMtxDirty(cobj);
        }
        (*cobj).u.up = *up;
    } else {
        HSD_CObjSetRoll(cobj, upvec2roll(cobj, up));
    }
}

/// `HSD_CObjSetRoll`.
pub unsafe fn HSD_CObjSetRoll(cobj: *mut HSD_CObj, roll: f32) {
    if cobj.is_null() {
        return;
    }
    if (*cobj).flags & COBJ_UP_VECTOR_ != 0 {
        // `up` is an uninitialised local in the C; zero in the shipping build (-ftrivial-auto-var-init=zero).
        let mut up: Vec = core::mem::zeroed();
        roll2upvec(cobj, &mut up, roll);
        HSD_CObjSetUpVector(cobj, &mut up);
    } else {
        if (*cobj).u.roll != roll {
            (*cobj).flags |= 0xC000_0000;
        }
        (*cobj).u.roll = roll;
    }
}

/// `HSD_CObjGetViewingMtxPtrDirect` (static inline).
#[inline]
pub unsafe fn HSD_CObjGetViewingMtxPtrDirect(cobj: *mut HSD_CObj) -> *mut [f32; 4] {
    (&raw mut (*cobj).view_mtx).cast()
}

/// The viewing-matrix rebuild shared by `HSD_CObjGetViewingMtxPtr` and `HSD_CObjGetInvViewingMtxPtr`.
unsafe fn rebuild_view_mtx(cobj: *mut HSD_CObj) {
    if (*cobj).flags & 2 == 0 && super::cobj::HSD_CObjMtxIsDirty(cobj) {
        let mut interest: Vec = core::mem::zeroed();
        let mut up_vec: Vec = core::mem::zeroed();
        let mut eyepos: Vec = core::mem::zeroed();
        HSD_CObjGetEyePosition(cobj, &mut eyepos);
        HSD_CObjGetUpVector(cobj, &mut up_vec);
        HSD_CObjGetInterest(cobj, &mut interest);
        (*cobj).view_mtx = mtx::look_at(&n3(&eyepos), &n3(&up_vec), &n3(&interest));
        super::wobj::HSD_WObjClearFlags((*cobj).eyepos, 2);
        super::wobj::HSD_WObjClearFlags((*cobj).interest, 2);
        super::cobj::HSD_CObjClearFlags(cobj, 0x4000_0000);
        super::cobj::HSD_CObjSetFlags(cobj, 0x8000_0000);
    }
}

/// `HSD_CObjGetViewingMtxPtr`.
pub unsafe fn HSD_CObjGetViewingMtxPtr(cobj: *mut HSD_CObj) -> *mut [f32; 4] {
    rebuild_view_mtx(cobj);
    HSD_CObjGetViewingMtxPtrDirect(cobj)
}

/// `HSD_CObjGetInvViewingMtxPtrDirect`: the inverse of the viewing matrix, (re)computed when bit 31 is set.
pub unsafe fn HSD_CObjGetInvViewingMtxPtrDirect(cobj: *mut HSD_CObj) -> *mut [f32; 4] {
    if (*cobj).flags & (1 << 31) != 0 {
        if (*cobj).proj_mtx.is_null() {
            (*cobj).proj_mtx = ext::mtx_alloc();
        }
        // PSMTXInverse leaves the destination untouched for a singular matrix.
        if let Some(inv) = mtx::inverse(&(*cobj).view_mtx) {
            *(*cobj).proj_mtx = inv;
        }
        super::cobj::HSD_CObjClearFlags(cobj, 1 << 31);
    }
    (*cobj).proj_mtx.cast()
}

/// `HSD_CObjGetInvViewingMtxPtr`.
pub unsafe fn HSD_CObjGetInvViewingMtxPtr(cobj: *mut HSD_CObj) -> *mut [f32; 4] {
    rebuild_view_mtx(cobj);
    HSD_CObjGetInvViewingMtxPtrDirect(cobj)
}

/// `HSD_CObjSetScissorx4`.
pub unsafe fn HSD_CObjSetScissorx4(cobj: *mut HSD_CObj, left: u16, right: u16, top: u16, bottom: u16) {
    if cobj.is_null() {
        return;
    }
    (*cobj).scissor.left = left;
    (*cobj).scissor.right = right;
    (*cobj).scissor.top = top;
    (*cobj).scissor.bottom = bottom;
}

/// `HSD_CObjSetViewportfx4`.
pub unsafe fn HSD_CObjSetViewportfx4(cobj: *mut HSD_CObj, left: f32, right: f32, top: f32, bottom: f32) {
    if cobj.is_null() {
        return;
    }
    (*cobj).viewport.xmin = left;
    (*cobj).viewport.xmax = right;
    (*cobj).viewport.ymin = top;
    (*cobj).viewport.ymax = bottom;
}

/// `HSD_CObjGetOrtho`.
pub unsafe fn HSD_CObjGetOrtho(cobj: *mut HSD_CObj, top: *mut f32, bottom: *mut f32, left: *mut f32, right: *mut f32) {
    if cobj.is_null() || (*cobj).projection_type != PROJ_ORTHO {
        return;
    }
    let o = &(*cobj).projection_param.ortho;
    if !top.is_null() {
        *top = o.top;
    }
    if !bottom.is_null() {
        *bottom = o.bottom;
    }
    if !left.is_null() {
        *left = o.left;
    }
    if !right.is_null() {
        *right = o.right;
    }
}
