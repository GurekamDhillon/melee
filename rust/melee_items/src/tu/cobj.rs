//! Native, pointer-level port of the `HSD_CObj` accessors of `src/sysdolphin/baselib/cobj.c` that the item,
//! trophy and vi TUs call: projection getters/setters (fov, top/bottom/left/right) and the eye/interest
//! position and distance helpers. Arithmetic is the C's (`tanf` is the MSL one, `VECMag` the paired-single
//! dataflow), the same expressions as `melee_runtime::cobj` (checked there against the C); tests/native_cobj.rs
//! checks these pointer-level bodies against that model port.
#![allow(non_snake_case, clippy::missing_safety_doc)]

use melee_runtime::gekko::tanf;
use melee_runtime::mtx::{vec_mag, vec_sub};
use melee_runtime::tu::cobj::{HSD_CObjGetEyePositionWObj, HSD_CObjGetInterestWObj, HSD_CObjUpdateView};
use melee_runtime::tu::wobj::{HSD_WObjGetPosition, HSD_WObjSetPosition};
use melee_types::*;

const PROJ_PERSPECTIVE: u8 = 1;
const PROJ_FRUSTUM: u8 = 2;
const PROJ_ORTHO: u8 = 3;

#[inline(always)]
fn deg_to_rad(a: f32) -> f32 {
    a * 0.01745329252f32
}

#[inline(always)]
unsafe fn half_tan(c: *mut HSD_CObj) -> f32 {
    tanf(0.5f32 * deg_to_rad((*c).projection_param.perspective.fov))
}

pub use melee_runtime::tu::cobj_world::HSD_CObjGetFov;

pub use melee_runtime::tu::cobj::HSD_CObjSetFov;

pub unsafe fn HSD_CObjGetTop(cobj: *mut HSD_CObj) -> f32 {
    if cobj.is_null() {
        return 0.0;
    }
    match (*cobj).projection_type {
        PROJ_PERSPECTIVE => (*cobj).near * half_tan(cobj),
        PROJ_FRUSTUM | PROJ_ORTHO => (*cobj).projection_param.perspective.fov,
        _ => 0.0,
    }
}

pub unsafe fn HSD_CObjGetBottom(cobj: *mut HSD_CObj) -> f32 {
    if cobj.is_null() {
        return 0.0;
    }
    match (*cobj).projection_type {
        PROJ_PERSPECTIVE => -(*cobj).near * half_tan(cobj),
        PROJ_FRUSTUM => (*cobj).projection_param.frustum.bottom,
        PROJ_ORTHO => (*cobj).projection_param.ortho.bottom,
        _ => 0.0,
    }
}

pub unsafe fn HSD_CObjGetLeft(cobj: *mut HSD_CObj) -> f32 {
    HSD_CObjUpdateView(cobj);
    if cobj.is_null() {
        return 0.0;
    }
    match (*cobj).projection_type {
        PROJ_PERSPECTIVE => (*cobj).projection_param.perspective.aspect * (-(*cobj).near * half_tan(cobj)),
        PROJ_FRUSTUM => (*cobj).projection_param.frustum.left,
        PROJ_ORTHO => (*cobj).projection_param.ortho.left,
        _ => 0.0,
    }
}

pub unsafe fn HSD_CObjGetRight(cobj: *mut HSD_CObj) -> f32 {
    HSD_CObjUpdateView(cobj);
    if cobj.is_null() {
        return 0.0;
    }
    match (*cobj).projection_type {
        PROJ_PERSPECTIVE => (*cobj).projection_param.perspective.aspect * ((*cobj).near * half_tan(cobj)),
        PROJ_FRUSTUM => (*cobj).projection_param.frustum.right,
        PROJ_ORTHO => (*cobj).projection_param.ortho.right,
        _ => 0.0,
    }
}

pub unsafe fn HSD_CObjSetTop(cobj: *mut HSD_CObj, top: f32) {
    if cobj.is_null() {
        return;
    }
    match (*cobj).projection_type {
        PROJ_FRUSTUM => (*cobj).projection_param.frustum.top = top,
        PROJ_ORTHO => (*cobj).projection_param.ortho.top = top,
        _ => {}
    }
}

pub unsafe fn HSD_CObjSetBottom(cobj: *mut HSD_CObj, bottom: f32) {
    if cobj.is_null() {
        return;
    }
    match (*cobj).projection_type {
        PROJ_FRUSTUM => (*cobj).projection_param.frustum.bottom = bottom,
        PROJ_ORTHO => (*cobj).projection_param.ortho.bottom = bottom,
        _ => {}
    }
}

pub unsafe fn HSD_CObjSetLeft(cobj: *mut HSD_CObj, left: f32) {
    if cobj.is_null() {
        return;
    }
    match (*cobj).projection_type {
        PROJ_FRUSTUM => (*cobj).projection_param.frustum.left = left,
        PROJ_ORTHO => (*cobj).projection_param.ortho.left = left,
        _ => {}
    }
}

pub unsafe fn HSD_CObjSetRight(cobj: *mut HSD_CObj, right: f32) {
    if cobj.is_null() {
        return;
    }
    match (*cobj).projection_type {
        PROJ_FRUSTUM => (*cobj).projection_param.frustum.right = right,
        PROJ_ORTHO => (*cobj).projection_param.ortho.right = right,
        _ => {}
    }
}

pub use melee_runtime::tu::cobj_world::HSD_CObjGetInterest;

pub use melee_runtime::tu::cobj_world::HSD_CObjSetInterest;

pub use melee_runtime::tu::cobj_world::HSD_CObjGetEyePosition;

pub use melee_runtime::tu::cobj_world::HSD_CObjSetEyePosition;

pub unsafe fn HSD_CObjGetEyeDistance(cobj: *mut HSD_CObj) -> f32 {
    if cobj.is_null() {
        return 0.0;
    }
    assert!(!(*cobj).eyepos.is_null(), "HSD_ASSERT(807, cobj->eyepos)");
    assert!(!(*cobj).interest.is_null(), "HSD_ASSERT(808, cobj->interest)");
    let mut position: Vec = core::mem::zeroed();
    let mut interest: Vec = core::mem::zeroed();
    HSD_CObjGetEyePosition(cobj, &mut position);
    HSD_CObjGetInterest(cobj, &mut interest);
    let look = vec_sub(&*(&interest as *const Vec as *const _), &*(&position as *const Vec as *const _));
    vec_mag(&look)
}
