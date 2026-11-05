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
    melee_core::ct!("HSD_CObjGetFlags");
    (*cobj).flags
}

/// `HSD_CObjGetFov`.
pub unsafe fn HSD_CObjGetFov(cobj: *mut HSD_CObj) -> f32 {
    melee_core::ct!("HSD_CObjGetFov");
    if cobj.is_null() || (*cobj).projection_type != PROJ_PERSPECTIVE {
        return 0.0;
    }
    (*cobj).projection_param.perspective.fov
}

/// `HSD_CObjGetAspect` (TARGET_PC: refreshes an auto view first).
pub unsafe fn HSD_CObjGetAspect(cobj: *mut HSD_CObj) -> f32 {
    melee_core::ct!("HSD_CObjGetAspect");
    HSD_CObjUpdateView(cobj);
    if cobj.is_null() || (*cobj).projection_type != PROJ_PERSPECTIVE {
        return 0.0;
    }
    (*cobj).projection_param.perspective.aspect
}

/// `HSD_CObjGetAuthoredAspect`.
pub unsafe fn HSD_CObjGetAuthoredAspect(cobj: *mut HSD_CObj) -> f32 {
    melee_core::ct!("HSD_CObjGetAuthoredAspect");
    if !cobj.is_null() && (*cobj).view_auto != 0 {
        return (*cobj).view_authored_aspect;
    }
    HSD_CObjGetAspect(cobj)
}

/// `HSD_CObjSetAspect`.
pub unsafe fn HSD_CObjSetAspect(cobj: *mut HSD_CObj, aspect: f32) {
    melee_core::ct!("HSD_CObjSetAspect");
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
    melee_core::ct!("HSD_CObjGetPerspective");
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
    melee_core::ct!("HSD_CObjGetScissor");
    if cobj.is_null() {
        return;
    }
    *scissor = (*cobj).scissor;
}

/// `HSD_CObjGetViewportf`.
pub unsafe fn HSD_CObjGetViewportf(cobj: *mut HSD_CObj, viewport: *mut _HSD_RectF32) {
    melee_core::ct!("HSD_CObjGetViewportf");
    if cobj.is_null() {
        return;
    }
    *viewport = (*cobj).viewport;
}

/// `HSD_CObjSetViewportf`.
pub unsafe fn HSD_CObjSetViewportf(cobj: *mut HSD_CObj, viewport: *mut _HSD_RectF32) {
    melee_core::ct!("HSD_CObjSetViewportf");
    if cobj.is_null() {
        return;
    }
    (*cobj).viewport = *viewport;
}

/// `HSD_CObjSetProjectionType`.
pub unsafe fn HSD_CObjSetProjectionType(cobj: *mut HSD_CObj, proj_type: u32) {
    melee_core::ct!("HSD_CObjSetProjectionType");
    if cobj.is_null() {
        return;
    }
    (*cobj).projection_type = proj_type as u8;
}

/// `HSD_CObjGetInterest`.
pub unsafe fn HSD_CObjGetInterest(cobj: *mut HSD_CObj, interest: *mut Vec) {
    melee_core::ct!("HSD_CObjGetInterest");
    assert_cobj(709, cobj);
    cx::HSD_WObjGetPosition((*cobj).interest.cast(), interest.cast());
}

/// `HSD_CObjSetInterest`.
pub unsafe fn HSD_CObjSetInterest(cobj: *mut HSD_CObj, interest: *mut Vec) {
    melee_core::ct!("HSD_CObjSetInterest");
    assert_cobj(721, cobj);
    cx::HSD_WObjSetPosition((*cobj).interest.cast(), interest.cast());
}

/// `HSD_CObjGetEyePosition`.
pub unsafe fn HSD_CObjGetEyePosition(cobj: *mut HSD_CObj, position: *mut Vec) {
    melee_core::ct!("HSD_CObjGetEyePosition");
    assert_cobj(733, cobj);
    cx::HSD_WObjGetPosition((*cobj).eyepos.cast(), position.cast());
}

/// `HSD_CObjSetEyePosition`.
pub unsafe fn HSD_CObjSetEyePosition(cobj: *mut HSD_CObj, position: *mut Vec) {
    melee_core::ct!("HSD_CObjSetEyePosition");
    assert_cobj(745, cobj);
    cx::HSD_WObjSetPosition((*cobj).eyepos.cast(), position.cast());
}

/// util.h `vec_normalize_check` (inline): -1 if `src` is within `FLT_MIN` of 0 on every axis, else
/// `PSVECNormalize(src, dst)` and 0.
unsafe fn vec_normalize_check(src: *mut Vec, dst: *mut Vec) -> i32 {
    if src.is_null() || dst.is_null() {
        return -1;
    }
    if crate::hsd_mtx::fabsf_bitwise((*src).x) <= FLT_MIN_
        && crate::hsd_mtx::fabsf_bitwise((*src).y) <= FLT_MIN_
        && crate::hsd_mtx::fabsf_bitwise((*src).z) <= FLT_MIN_
    {
        return -1;
    }
    cx::PSVECNormalize(src, dst);
    0
}

/// `HSD_CObjGetEyeVector`: 0 and the unit view direction, or -1 and (0, 0, -1).
pub unsafe fn HSD_CObjGetEyeVector(cobj: *mut HSD_CObj, eye: *mut Vec) -> i32 {
    melee_core::ct!("HSD_CObjGetEyeVector");
    if !cobj.is_null() && !(*cobj).eyepos.is_null() && !(*cobj).interest.is_null() && !eye.is_null() {
        let mut eyepos: Vec = core::mem::zeroed();
        let mut interest: Vec = core::mem::zeroed();
        HSD_CObjGetEyePosition(cobj, &mut eyepos);
        HSD_CObjGetInterest(cobj, &mut interest);
        cx::PSVECSubtract(&mut interest, &mut eyepos, eye);
        if vec_normalize_check(eye, eye) == 0 {
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

/// Callees of cobj.c in other TUs that this part reaches (the oracle supplies the C `HSD_MtxAlloc` pool).
pub mod ext {
    use melee_types::Mtx;
    /// `HSD_MtxAlloc` (mtx.c); unset runs the pooled port.
    pub static mut HSD_MtxAlloc: Option<unsafe fn() -> *mut Mtx> = None;
    pub unsafe fn mtx_alloc() -> *mut Mtx {
        match HSD_MtxAlloc {
            Some(f) => f(),
            None => super::cx::HSD_MtxAlloc() as *mut Mtx,
        }
    }
}
use super::cobj::ext as cx;

const FLT_MIN_: f32 = crate::cobj::FLT_MIN;
const COBJ_UP_VECTOR_: u32 = 1;

static mut ORIG: Vec = Vec { x: 0.0, y: 0.0, z: 0.0 };
static mut UY: Vec = Vec { x: 0.0, y: 1.0, z: 0.0 };

/// `upvec2roll` (static).
unsafe fn upvec2roll(cobj: *mut HSD_CObj, up: *mut Vec) -> f32 {
    let mut eye: Vec = core::mem::zeroed();
    let mut v: Vec = core::mem::zeroed();
    let mut vmtx = [[0.0f32; 4]; 3];
    let mut dot;
    if HSD_CObjGetEyeVector(cobj, &mut eye) != 0 {
        dot = 0.0;
    } else {
        dot = cx::fabsf(cx::PSVECDotProduct(up, &mut eye));
        dot = 1.0 - dot;
        if dot < FLT_MIN_ {
            dot = 0.0;
        } else {
            cx::C_MTXLookAt(vmtx.as_mut_ptr().cast(), &raw mut ORIG, &raw mut UY, &mut eye);
            cx::PSMTXMultVecSR(vmtx.as_mut_ptr().cast(), up, &mut v);
            if crate::hsd_mtx::fabsf_bitwise(v.y) == 0.0 {
                dot = if -v.x >= 0.0 { 1.5707963267948966f64 as f32 } else { -1.5707963267948966f64 as f32 };
            } else {
                dot = cx::atan2f(-v.x, v.y);
            }
        }
    }
    dot
}

/// `roll2upvec` (static): 0 and `*up` written, or the nonzero `HSD_CObjGetEyeVector` result with `up` untouched.
unsafe fn roll2upvec(cobj: *mut HSD_CObj, up: *mut Vec, roll: f32) -> i32 {
    let mut eye: Vec = core::mem::zeroed();
    let res = HSD_CObjGetEyeVector(cobj, &mut eye);
    if res != 0 {
        return res;
    }
    let mut v0: Vec = core::mem::zeroed();
    let mut v1: Vec = core::mem::zeroed();
    let mut m = [[0.0f32; 4]; 3];
    if 1.0 - (eye.y.abs() as f64) < 0.0001 {
        v0.x = crate::gekko::sqrtf(eye.y * eye.y + eye.z * eye.z); // eye is in memory: not fused on the console
        v0.y = eye.y * (-eye.x / v0.x);
        v0.z = eye.z * (-eye.x / v0.x);
    } else {
        v0.y = crate::gekko::sqrtf(eye.x * eye.x + eye.z * eye.z);
        v0.x = eye.x * (-eye.y / v0.y);
        v0.z = eye.z * (-eye.y / v0.y);
    }
    cx::PSMTXRotAxisRad(m.as_mut_ptr().cast(), &mut eye, -roll);
    cx::PSMTXMultVecSR(m.as_mut_ptr().cast(), &mut v0, &mut v1);
    cx::PSVECNormalize(&mut v1, up);
    0
}

/// `HSD_CObjGetUpVector`: 0 and the up vector, or -1 and (0, 1, 0).
pub unsafe fn HSD_CObjGetUpVector(cobj: *mut HSD_CObj, up: *mut Vec) -> i32 {
    melee_core::ct!("HSD_CObjGetUpVector");
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

static mut UY2: Vec = Vec { x: 0.0, y: 1.0, z: 0.0 };

/// `HSD_CObjSetUpVector`.
pub unsafe fn HSD_CObjSetUpVector(cobj: *mut HSD_CObj, mut up: *mut Vec) {
    melee_core::ct!("HSD_CObjSetUpVector");
    if cobj.is_null() || up.is_null() {
        return;
    }
    if (*cobj).flags & COBJ_UP_VECTOR_ != 0 {
        let mut v: Vec = core::mem::zeroed();
        up = if vec_normalize_check(up, &mut v) != 0 { &raw mut UY2 } else { &mut v };
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
    melee_core::ct!("HSD_CObjSetRoll");
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
        cx::C_MTXLookAt((&raw mut (*cobj).view_mtx).cast(), &mut eyepos, &mut up_vec, &mut interest);
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
        cx::PSMTXInverse((&raw mut (*cobj).view_mtx).cast(), (*cobj).proj_mtx.cast());
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
    melee_core::ct!("HSD_CObjSetScissorx4");
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
    melee_core::ct!("HSD_CObjSetViewportfx4");
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
    melee_core::ct!("HSD_CObjGetOrtho");
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
