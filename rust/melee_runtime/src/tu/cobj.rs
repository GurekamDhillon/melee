//! Port of src/sysdolphin/baselib/cobj.c, the whole TU.
//!
//! The camera math (eye / interest / up / roll, viewing matrix and its inverse, projection
//! getters and setters, `makeProjectionMtx`, `HSD_CObjUpdateView`, `HSD_CObjGetAuthoredAspect`,
//! the flag accessors) is already ported over the native `HsdCObj<R>` in [`crate::cobj`] and
//! re-exported here (snake_case names).
//!
//! The rest works on the GameCube-layout record `melee_types::HSD_CObj` through raw pointers,
//! because it touches the class system, the AObj and the GX state: the camera animation
//! (`HSD_CObjRemoveAnim*`, `AddAnim`, `CObjUpdateFunc`, `Anim`, `ReqAnim`), the GX camera setup
//! (`HSD_CObjEraseScreen`, `setup{Offscreen,Normal,TopHalf,BottomHalf}Camera`,
//! `HSD_CObjSetCurrent` / `EndCurrent` / `GetCurrent`, `HSD_CObjSetupViewingMtx`) and load / init /
//! class (`CObjLoad`, `HSD_CObjInit`, `LoadDesc`, `Alloc`, `CObjInit`, `CObjRelease`,
//! `CObjAmnesia`, `CObjInfoInit`, `SetDefaultClass`). The simple accessors those need are ported
//! again on the raw record under the C names. The tree-dependent camera calls (eye / interest
//! positions, roll, up vector: they walk the WObj's JObj target) are the hooks in [`ext`], as are
//! the other unported TUs' functions (AObj, WObj, GX, video, displayfunc's Z list, mtx alloc).
//!
//! Float rule: clang fuses the half-screen projection arithmetic
//! `h_scale * (fov - aspect) - fov` / `+ aspect` (f32, `fmadds`); the products by 2.0f are exact
//! so their contraction changes nothing. `DegToRad(a)` is `a * 0.01745329252F` and is applied to
//! the double `0.5 * fov` in the half-screen cameras (so it is a double product) and to `fov` in
//! `HSD_CObjEraseScreen` (`0.5F * DegToRad(fov)` is all f32). `tanf` is `gekko::tanf`.
//! `HSD_CObjSetupViewingMtx`'s TARGET_PC `diag_cobj_view` call only logs and is not ported.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

pub use crate::cobj::*;

use super::class::{hsdInitClassInfo, hsdIsDescendantOf, hsdNew, hsdSearchClassInfo};
use super::debug::{HSD_Panic, __assert};
use crate::cabi::f2u32;
use crate::gekko::tanf;
use crate::mtx::{self, Vec3};
use core::ffi::{c_char, c_void};
use core::ptr::{addr_of_mut, null_mut};
use melee_core::fp::fmadds;
use melee_types::{
    GXRenderModeObj, HSD_AObjDesc, HSD_CObj, HSD_CObjDesc, HSD_CObjInfo, HSD_CameraAnim, HSD_Class,
    HSD_ClassInfo, HSD_ObjData, HSD_WObj, HSD_WObjDesc, Mtx44, _HSD_RectF32, _HSD_RectS16, _HSD_RenderPass,
    _Scissor, _HSD_ClassInfoHead, _HSD_ObjInfo, HSD_RP_BOTTOMHALF, HSD_RP_OFFSCREEN, HSD_RP_SCREEN, HSD_RP_TOPHALF,
};

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    // aobj.c
    fn HSD_AObjRemove(aobj: *mut melee_types::HSD_AObj);
    fn HSD_AObjLoadDesc(desc: *mut HSD_AObjDesc) -> *mut melee_types::HSD_AObj;
    fn HSD_AObjInterpretAnim(aobj: *mut melee_types::HSD_AObj, obj: *mut c_void, update: unsafe extern "C" fn(*mut c_void, i32, *mut HSD_ObjData));
    fn HSD_AObjReqAnim(aobj: *mut melee_types::HSD_AObj, startframe: f32);
    // wobj.c
    fn HSD_WObjAlloc() -> *mut HSD_WObj;
    fn HSD_WObjUnref(wobj: *mut HSD_WObj);
    fn HSD_WObjInit(wobj: *mut HSD_WObj, desc: *mut HSD_WObjDesc);
    fn HSD_WObjRemoveAnim(wobj: *mut HSD_WObj);
    fn HSD_WObjAddAnim(wobj: *mut HSD_WObj, anim: *mut melee_types::HSD_WObjAnim);
    fn HSD_WObjInterpretAnim(wobj: *mut HSD_WObj);
    fn HSD_WObjReqAnim(wobj: *mut HSD_WObj, startframe: f32);
    fn HSD_WObjClearFlags(wobj: *mut HSD_WObj, flags: u32);
    // cobj.c on the native model (positions walk the WObj's JObj target)
    fn HSD_CObjGetEyePosition(cobj: *mut HSD_CObj) -> Vec3;
    fn HSD_CObjSetEyePosition(cobj: *mut HSD_CObj, v: Vec3);
    fn HSD_CObjGetInterest(cobj: *mut HSD_CObj) -> Vec3;
    fn HSD_CObjSetInterest(cobj: *mut HSD_CObj, v: Vec3);
    fn HSD_CObjGetUpVector(cobj: *mut HSD_CObj) -> Vec3;
    fn HSD_CObjSetUpVector(cobj: *mut HSD_CObj, v: Vec3);
    fn HSD_CObjSetRoll(cobj: *mut HSD_CObj, roll: f32);
    fn HSD_CObjSetFov(cobj: *mut HSD_CObj, fov: f32);
    // displayfunc.c
    fn HSD_EraseRect(top: f32, bottom: f32, left: f32, right: f32, z: f32, enable_color: i32, enable_alpha: i32, enable_depth: i32);
    fn _HSD_ZListClear();
    fn _HSD_ZListSort();
    fn _HSD_ZListDisp();
    // video.c / initialize.c
    fn HSD_VIGetRenderMode() -> *mut GXRenderModeObj;
    fn HSD_GetCurrentRenderPass() -> _HSD_RenderPass;
    fn VIGetNextField() -> u32;
    // GX
    fn GXSetViewport(left: f32, top: f32, width: f32, height: f32, nearz: f32, farz: f32);
    fn GXSetViewportJitter(left: f32, top: f32, width: f32, height: f32, nearz: f32, farz: f32, field: u32);
    fn GXSetScissor(left: u32, top: u32, width: u32, height: u32);
    fn GXSetProjection(mtx: &Mtx44, type_: u32);
    // memory / object.c / platform
    fn HSD_MtxFree(mtx: *mut melee_types::Mtx);
    fn hsdObj() -> *mut HSD_ClassInfo;
    fn View_ProjectionScaleBits() -> i32;
    }
}

macro_rules! hsd_assert {
    ($line:expr, $cond:expr, $text:literal) => {
        if !($cond) {
            __assert(c"cobj.c".as_ptr(), $line, concat!($text, "\0").as_ptr() as *const c_char);
        }
    };
}

const PROJ_PERSPECTIVE_: u8 = 1;
const PROJ_FRUSTUM_: u8 = 2;
const PROJ_ORTHO_: u8 = 3;
const GX_PERSPECTIVE_: u32 = 0;
const GX_ORTHOGRAPHIC_: u32 = 1;

static mut default_class: *mut HSD_ClassInfo = null_mut();
static mut current: *mut HSD_CObj = null_mut();

/// `hsdCObj`: the CObj class info (`info_init` is `CObjInfoInit`).
pub static mut hsdCObj: HSD_CObjInfo = HSD_CObjInfo {
    parent: _HSD_ObjInfo {
        parent: HSD_ClassInfo {
            head: _HSD_ClassInfoHead {
                info_init: Some(CObjInfoInit),
                flags: 0,
                library_name: null_mut(),
                class_name: null_mut(),
                obj_size: 0,
                info_size: 0,
                parent: null_mut(),
                next: null_mut(),
                child: null_mut(),
                nb_exist: 0,
                nb_peak: 0,
            },
            alloc: None,
            init: None,
            release: None,
            destroy: None,
            amnesia: None,
        },
    },
    load: None,
};

/// `DegToRad(a)` on an f32.
#[inline]
fn deg_to_rad_f(a: f32) -> f32 {
    a * 0.01745329252f32
}

/// `DegToRad(a)` on a double (`0.5 * fov`): `a * 0.01745329252F` promoted to double.
#[inline]
fn deg_to_rad_d(a: f64) -> f64 {
    a * (0.01745329252f32 as f64)
}

fn class_of(info: *mut HSD_CObjInfo) -> *mut HSD_ClassInfo {
    info as *mut HSD_ClassInfo
}

// --- raw-record accessors (cobj.c getters / setters used below) --------------------------

/// `HSD_CObjGetNear`.
pub unsafe fn HSD_CObjGetNear(cobj: *mut HSD_CObj) -> f32 {
    if cobj.is_null() { 0.0 } else { (*cobj).near }
}

/// `HSD_CObjSetNear`.
pub unsafe fn HSD_CObjSetNear(cobj: *mut HSD_CObj, v: f32) {
    if !cobj.is_null() {
        (*cobj).near = v;
    }
}

/// `HSD_CObjGetFar`.
pub unsafe fn HSD_CObjGetFar(cobj: *mut HSD_CObj) -> f32 {
    if cobj.is_null() { 0.0 } else { (*cobj).far }
}

/// `HSD_CObjSetFar`.
pub unsafe fn HSD_CObjSetFar(cobj: *mut HSD_CObj, v: f32) {
    if !cobj.is_null() {
        (*cobj).far = v;
    }
}

/// `HSD_CObjUpdateView` (TARGET_PC): an auto-view perspective camera's aspect follows the window.
pub unsafe fn HSD_CObjUpdateView(cobj: *mut HSD_CObj) {
    if !cobj.is_null() && (*cobj).view_auto != 0 && (*cobj).projection_type == PROJ_PERSPECTIVE_ {
        let scale = f32::from_bits(ext::View_ProjectionScaleBits() as u32);
        (*cobj).projection_param.perspective.aspect = (*cobj).view_authored_aspect * scale;
    }
}

/// `HSD_CObjGetProjectionType`.
pub unsafe fn HSD_CObjGetProjectionType(cobj: *mut HSD_CObj) -> i32 {
    HSD_CObjUpdateView(cobj);
    if cobj.is_null() {
        return 1;
    }
    (*cobj).projection_type as i32
}

/// `HSD_CObjSetViewport`.
pub unsafe fn HSD_CObjSetViewport(cobj: *mut HSD_CObj, viewport: *const _HSD_RectS16) {
    if cobj.is_null() {
        return;
    }
    (*cobj).viewport.xmin = (*viewport).xmin as f32;
    (*cobj).viewport.xmax = (*viewport).xmax as f32;
    (*cobj).viewport.ymin = (*viewport).ymin as f32;
    (*cobj).viewport.ymax = (*viewport).ymax as f32;
}

/// `HSD_CObjSetScissor`.
pub unsafe fn HSD_CObjSetScissor(cobj: *mut HSD_CObj, scissor: *const _Scissor) {
    if cobj.is_null() {
        return;
    }
    (*cobj).scissor = *scissor;
}

/// `HSD_CObjSetPerspective`.
pub unsafe fn HSD_CObjSetPerspective(cobj: *mut HSD_CObj, fov: f32, aspect: f32) {
    if cobj.is_null() {
        return;
    }
    (*cobj).projection_type = PROJ_PERSPECTIVE_;
    (*cobj).projection_param.perspective.fov = fov;
    (*cobj).projection_param.perspective.aspect = aspect;
    if (*cobj).view_auto != 0 {
        (*cobj).view_authored_aspect = aspect;
        HSD_CObjUpdateView(cobj);
    }
}

/// `HSD_CObjSetFrustum`.
pub unsafe fn HSD_CObjSetFrustum(cobj: *mut HSD_CObj, top: f32, bottom: f32, left: f32, right: f32) {
    if cobj.is_null() {
        return;
    }
    (*cobj).projection_type = PROJ_FRUSTUM_;
    (*cobj).projection_param.frustum.top = top;
    (*cobj).projection_param.frustum.bottom = bottom;
    (*cobj).projection_param.frustum.left = left;
    (*cobj).projection_param.frustum.right = right;
}

/// `HSD_CObjSetOrtho`.
pub unsafe fn HSD_CObjSetOrtho(cobj: *mut HSD_CObj, top: f32, bottom: f32, left: f32, right: f32) {
    if cobj.is_null() {
        return;
    }
    (*cobj).projection_type = PROJ_ORTHO_;
    (*cobj).projection_param.ortho.top = top;
    (*cobj).projection_param.ortho.bottom = bottom;
    (*cobj).projection_param.ortho.left = left;
    (*cobj).projection_param.ortho.right = right;
}

/// `HSD_CObjSetFlags`.
pub unsafe fn HSD_CObjSetFlags(cobj: *mut HSD_CObj, flags: u32) {
    if !cobj.is_null() {
        (*cobj).flags |= flags;
    }
}

/// `HSD_CObjClearFlags`.
pub unsafe fn HSD_CObjClearFlags(cobj: *mut HSD_CObj, flags: u32) {
    if !cobj.is_null() {
        (*cobj).flags &= !flags;
    }
}

/// `HSD_CObjSetMtxDirty`.
pub unsafe fn HSD_CObjSetMtxDirty(cobj: *mut HSD_CObj) {
    (*cobj).flags |= (1 << 30) | (1 << 31);
}

/// `HSD_CObjMtxIsDirty`.
pub unsafe fn HSD_CObjMtxIsDirty(cobj: *mut HSD_CObj) -> bool {
    ((*cobj).flags & (1 << 30)) != 0
        || (!(*cobj).eyepos.is_null() && ((*(*cobj).eyepos).flags & 2) != 0)
        || (!(*cobj).interest.is_null() && ((*(*cobj).interest).flags & 2) != 0)
}

/// `HSD_CObjGetEyePositionWObj`.
pub unsafe fn HSD_CObjGetEyePositionWObj(cobj: *mut HSD_CObj) -> *mut HSD_WObj {
    hsd_assert!(685, !cobj.is_null(), "cobj");
    (*cobj).eyepos
}

/// `HSD_CObjGetInterestWObj`.
pub unsafe fn HSD_CObjGetInterestWObj(cobj: *mut HSD_CObj) -> *mut HSD_WObj {
    hsd_assert!(661, !cobj.is_null(), "cobj");
    (*cobj).interest
}

// --- the TU's own functions ------------------------------------------------------------

/// `HSD_CObjEraseScreen`.
pub unsafe fn HSD_CObjEraseScreen(cobj: *mut HSD_CObj, enable_color: i32, enable_alpha: i32, enable_depth: i32) {
    if cobj.is_null() || (enable_color == 0 && enable_alpha == 0 && enable_depth == 0) {
        return;
    }

    let z_val: f32 = (0.5f64 * (HSD_CObjGetNear(cobj) + HSD_CObjGetFar(cobj)) as f64) as f32;
    let (mut left_res, mut right_res, mut top_res, mut bottom_res) = (0f32, 0f32, 0f32, 0f32);

    match HSD_CObjGetProjectionType(cobj) {
        1 => {
            top_res = z_val * tanf(0.5f32 * deg_to_rad_f((*cobj).projection_param.perspective.fov));
            bottom_res = -top_res;
            right_res = top_res * (*cobj).projection_param.perspective.aspect;
            left_res = -right_res;
        }
        2 => {
            let m_val = z_val / HSD_CObjGetNear(cobj);
            right_res = m_val * (*cobj).projection_param.frustum.right;
            left_res = m_val * (*cobj).projection_param.frustum.left;
            top_res = m_val * (*cobj).projection_param.frustum.top;
            bottom_res = m_val * (*cobj).projection_param.frustum.bottom;
        }
        3 => {
            right_res = (*cobj).projection_param.ortho.right;
            left_res = (*cobj).projection_param.ortho.left;
            top_res = (*cobj).projection_param.ortho.top;
            bottom_res = (*cobj).projection_param.ortho.bottom;
        }
        _ => {}
    }

    ext::HSD_EraseRect(top_res, bottom_res, left_res, right_res, -z_val, enable_color, enable_alpha, enable_depth);
}

/// `HSD_CObjRemoveAnimByFlags`.
pub unsafe fn HSD_CObjRemoveAnimByFlags(cobj: *mut HSD_CObj, _flags: u32) {
    if cobj.is_null() {
        return;
    }
    ext::HSD_AObjRemove((*cobj).aobj);
    (*cobj).aobj = null_mut();
    ext::HSD_WObjRemoveAnim(HSD_CObjGetEyePositionWObj(cobj));
    ext::HSD_WObjRemoveAnim(HSD_CObjGetInterestWObj(cobj));
}

/// `HSD_CObjRemoveAnim`.
pub unsafe fn HSD_CObjRemoveAnim(cobj: *mut HSD_CObj) {
    if cobj.is_null() {
        return;
    }
    HSD_CObjRemoveAnimByFlags(cobj, 0x7FF);
}

/// `HSD_CObjAddAnim`.
pub unsafe fn HSD_CObjAddAnim(cobj: *mut HSD_CObj, canim: *mut HSD_CameraAnim) {
    if cobj.is_null() || canim.is_null() {
        return;
    }
    if !(*cobj).aobj.is_null() {
        ext::HSD_AObjRemove((*cobj).aobj);
    }
    (*cobj).aobj = ext::HSD_AObjLoadDesc((*canim).aobjdesc);
    ext::HSD_WObjAddAnim(HSD_CObjGetEyePositionWObj(cobj), (*canim).eye_anim);
    ext::HSD_WObjAddAnim(HSD_CObjGetInterestWObj(cobj), (*canim).interest_anim);
}

/// `CObjUpdateFunc` (static): the AObj track writer (type 5-7 all write `x`, as in the C).
pub unsafe extern "C" fn CObjUpdateFunc(obj: *mut c_void, type_: i32, val: *mut HSD_ObjData) {
    let cobj = obj as *mut HSD_CObj;
    if cobj.is_null() {
        return;
    }
    let fv = (*val).fv;
    match type_ {
        1 => {
            let mut v = ext::HSD_CObjGetEyePosition(cobj);
            v.x = fv;
            ext::HSD_CObjSetEyePosition(cobj, v);
        }
        2 => {
            let mut v = ext::HSD_CObjGetEyePosition(cobj);
            v.y = fv;
            ext::HSD_CObjSetEyePosition(cobj, v);
        }
        3 => {
            let mut v = ext::HSD_CObjGetEyePosition(cobj);
            v.z = fv;
            ext::HSD_CObjSetEyePosition(cobj, v);
        }
        5 | 6 | 7 => {
            let mut v = ext::HSD_CObjGetInterest(cobj);
            v.x = fv;
            ext::HSD_CObjSetInterest(cobj, v);
        }
        9 => ext::HSD_CObjSetRoll(cobj, fv),
        10 => ext::HSD_CObjSetFov(cobj, fv),
        11 => HSD_CObjSetNear(cobj, fv),
        12 => HSD_CObjSetFar(cobj, fv),
        _ => {}
    }
}

/// `HSD_CObjAnim`.
pub unsafe fn HSD_CObjAnim(cobj: *mut HSD_CObj) {
    if cobj.is_null() {
        return;
    }
    ext::HSD_AObjInterpretAnim((*cobj).aobj, cobj as *mut c_void, CObjUpdateFunc);
    ext::HSD_WObjInterpretAnim((*cobj).eyepos);
    ext::HSD_WObjInterpretAnim((*cobj).interest);
}

/// `HSD_CObjReqAnim`.
pub unsafe fn HSD_CObjReqAnim(cobj: *mut HSD_CObj, startframe: f32) {
    if cobj.is_null() {
        return;
    }
    ext::HSD_AObjReqAnim((*cobj).aobj, startframe);
    ext::HSD_WObjReqAnim((*cobj).eyepos, startframe);
    ext::HSD_WObjReqAnim((*cobj).interest, startframe);
}

/// `makeProjectionMtx` on the raw record: the GX projection type and `mtx` (untouched, type 0, for
/// an unknown projection type).
pub unsafe fn makeProjectionMtx(cobj: *mut HSD_CObj, m: &mut Mtx44) -> u32 {
    HSD_CObjUpdateView(cobj);
    let c = &*cobj;
    match c.projection_type {
        PROJ_PERSPECTIVE_ => {
            *m = mtx::perspective(c.projection_param.perspective.fov, c.projection_param.perspective.aspect, c.near, c.far);
            GX_PERSPECTIVE_
        }
        PROJ_FRUSTUM_ => {
            let p = &c.projection_param.frustum;
            *m = mtx::frustum(p.top, p.bottom, p.left, p.right, c.near, c.far);
            GX_PERSPECTIVE_
        }
        PROJ_ORTHO_ => {
            let p = &c.projection_param.ortho;
            *m = mtx::ortho(p.top, p.bottom, p.left, p.right, c.near, c.far);
            GX_ORTHOGRAPHIC_
        }
        _ => 0,
    }
}

/// `setupOffscreenCamera` (static).
unsafe fn setupOffscreenCamera(cobj: *mut HSD_CObj) -> bool {
    let c = &*cobj;
    ext::GXSetViewport(
        c.viewport.xmin,
        c.viewport.ymin,
        c.viewport.xmax - c.viewport.xmin,
        c.viewport.ymax - c.viewport.ymin,
        0.0,
        1.0,
    );
    ext::GXSetScissor(
        c.scissor.left as u32,
        c.scissor.top as u32,
        (c.scissor.right as i32 - c.scissor.left as i32) as u32,
        (c.scissor.bottom as i32 - c.scissor.top as i32) as u32,
    );
    let mut m: Mtx44 = [[0.0; 4]; 4];
    let t = makeProjectionMtx(cobj, &mut m);
    ext::GXSetProjection(&m, t);
    true
}

/// `setupNormalCamera` (static).
unsafe fn setupNormalCamera(cobj: *mut HSD_CObj) -> bool {
    let c = &*cobj;
    let rmode = ext::HSD_VIGetRenderMode();

    let x_scale = (*rmode).fbWidth as f32 / (*rmode).viWidth as f32;
    let y_scale = (*rmode).efbHeight as f32 / (*rmode).viHeight as f32;

    let mut left = c.viewport.xmin * x_scale;
    let mut right = c.viewport.xmax * x_scale;
    let mut top = c.viewport.ymin * y_scale;
    let mut bottom = c.viewport.ymax * y_scale;

    let mut width = right - left;
    let mut height = bottom - top;

    if (*rmode).field_rendering != 0 {
        ext::GXSetViewportJitter(left, top, width, height, 0.0, 1.0, ext::VIGetNextField());
    } else {
        ext::GXSetViewport(left, top, width, height, 0.0, 1.0);
    }

    left = c.scissor.left as f32 * x_scale;
    right = c.scissor.right as f32 * x_scale;
    top = c.scissor.top as f32 * y_scale;
    bottom = c.scissor.bottom as f32 * y_scale;
    width = right - left;
    height = bottom - top;
    ext::GXSetScissor(f2u32(left as f64), f2u32(top as f64), f2u32(width as f64), f2u32(height as f64));

    let mut p: Mtx44 = [[0.0; 4]; 4];
    let projection_type = makeProjectionMtx(cobj, &mut p);
    ext::GXSetProjection(&p, projection_type);
    true
}

/// `setupTopHalfCamera` (static).
unsafe fn setupTopHalfCamera(cobj: *mut HSD_CObj) -> bool {
    let c = &*cobj;
    let rmode = ext::HSD_VIGetRenderMode();
    let efb = (*rmode).efbHeight as f32;

    if c.viewport.ymin >= efb {
        return false;
    }
    let mut left = c.viewport.xmin;
    let mut right = c.viewport.xmax;
    let mut top = c.viewport.ymin;
    let mut bottom = c.viewport.ymax;
    bottom = if bottom < efb { bottom } else { efb };
    let mut width = right - left;
    let mut height = bottom - top;
    ext::GXSetScissor(f2u32(left as f64), f2u32(top as f64), f2u32(width as f64), f2u32(height as f64));
    top = c.viewport.ymin;
    bottom = c.viewport.ymax;
    left = c.viewport.xmin;
    right = c.viewport.xmax;
    height = (if bottom < efb { bottom } else { efb }) - top;
    let h_scale = height / (bottom - top);
    width = right - left;

    ext::GXSetViewport(left, top, width, height, 0.0, 1.0);

    let fov = c.projection_param.perspective.fov;
    let aspect = c.projection_param.perspective.aspect;
    let mut p: Mtx44 = [[0.0; 4]; 4];
    let projection_type;
    match c.projection_type {
        PROJ_PERSPECTIVE_ => {
            projection_type = GX_PERSPECTIVE_;
            let t = c.near * tanf(deg_to_rad_d(0.5f64 * fov as f64) as f32);
            let w = t * aspect;
            let b = t * -(2.0f32 * h_scale - 1.0f32);
            p = mtx::frustum(t, b, -w, w, c.near, c.far);
        }
        PROJ_FRUSTUM_ => {
            projection_type = GX_PERSPECTIVE_;
            let bot = -fmadds(h_scale, fov - aspect, -fov);
            p = mtx::frustum(fov, bot, c.projection_param.frustum.left, c.projection_param.frustum.right, c.near, c.far);
        }
        PROJ_ORTHO_ => {
            projection_type = GX_ORTHOGRAPHIC_;
            let bot = -fmadds(h_scale, fov - aspect, -fov);
            p = mtx::ortho(fov, bot, c.projection_param.frustum.left, c.projection_param.frustum.right, c.near, c.far);
        }
        _ => projection_type = 0,
    }

    ext::GXSetProjection(&p, projection_type);
    true
}

/// `setupBottomHalfCamera` (static).
unsafe fn setupBottomHalfCamera(cobj: *mut HSD_CObj) -> bool {
    let c = &*cobj;
    let rmode = ext::HSD_VIGetRenderMode();

    let screen_top: u32 = ((*rmode).efbHeight as u32).wrapping_sub(8);
    let screen_top_f = screen_top as f32;

    if c.viewport.ymax < screen_top_f {
        return false;
    }

    let left = c.scissor.left as f32;
    let right = c.scissor.right as f32;
    let sc_top = if (c.scissor.top as u32) > screen_top { c.scissor.top as u32 } else { screen_top };
    let top = sc_top.wrapping_sub(screen_top) as f32;
    let bottom = (c.scissor.bottom as u32).wrapping_sub(screen_top) as f32;
    let width = right - left;
    let height = bottom - top;
    ext::GXSetScissor(f2u32(left as f64), f2u32(top as f64), f2u32(width as f64), f2u32(height as f64));

    let mut top = c.viewport.ymin;
    let left = c.viewport.xmin;
    let right = c.viewport.xmax;
    top = if top > screen_top_f { top } else { screen_top_f };
    let bottom = c.viewport.ymax;
    let width = right - left;
    let height = bottom - c.viewport.ymin;

    let t = top - screen_top_f;
    let h = (bottom - screen_top_f) - t;
    let hscale = h / height;
    ext::GXSetViewport(left, t, width, h, 0.0, 1.0);

    let fov = c.projection_param.perspective.fov;
    let aspect = c.projection_param.perspective.aspect;
    let mut p: Mtx44 = [[0.0; 4]; 4];
    let projection_type;
    match c.projection_type {
        PROJ_PERSPECTIVE_ => {
            projection_type = GX_PERSPECTIVE_;
            let b = c.near * tanf(deg_to_rad_d(0.5f64 * fov as f64) as f32);
            let w = b * aspect;
            let t = b * (2.0f32 * hscale + -1.0f32);
            p = mtx::frustum(t, -b, -w, w, c.near, c.far);
        }
        PROJ_FRUSTUM_ => {
            projection_type = GX_PERSPECTIVE_;
            let h = fmadds(hscale, fov - aspect, aspect);
            p = mtx::frustum(h, aspect, c.projection_param.frustum.left, c.projection_param.frustum.right, c.near, c.far);
        }
        PROJ_ORTHO_ => {
            projection_type = GX_ORTHOGRAPHIC_;
            let h = fmadds(hscale, fov - aspect, aspect);
            p = mtx::ortho(h, aspect, c.projection_param.frustum.left, c.projection_param.frustum.right, c.near, c.far);
        }
        _ => projection_type = 0,
    }

    ext::GXSetProjection(&p, projection_type);
    true
}

/// `HSD_CObjSetupViewingMtx`.
pub unsafe fn HSD_CObjSetupViewingMtx(cobj: *mut HSD_CObj) {
    if (*cobj).flags & 2 == 0 && HSD_CObjMtxIsDirty(cobj) {
        let eyepos = ext::HSD_CObjGetEyePosition(cobj);
        let up_vec = ext::HSD_CObjGetUpVector(cobj);
        let interest = ext::HSD_CObjGetInterest(cobj);
        (*cobj).view_mtx = mtx::look_at(&eyepos, &up_vec, &interest);
        ext::HSD_WObjClearFlags((*cobj).eyepos, 2);
        ext::HSD_WObjClearFlags((*cobj).interest, 2);
        HSD_CObjClearFlags(cobj, 0x4000_0000);
        HSD_CObjSetFlags(cobj, 0x8000_0000);
    }
}

/// `HSD_CObjSetCurrent`.
pub unsafe fn HSD_CObjSetCurrent(cobj: *mut HSD_CObj) -> bool {
    HSD_CObjUpdateView(cobj);
    if cobj.is_null() {
        return false;
    }
    let render_pass = ext::HSD_GetCurrentRenderPass();
    ext::_HSD_ZListClear();
    current = cobj;
    let result = match render_pass {
        HSD_RP_OFFSCREEN => setupOffscreenCamera(cobj),
        HSD_RP_SCREEN => setupNormalCamera(cobj),
        HSD_RP_TOPHALF => setupTopHalfCamera(cobj),
        HSD_RP_BOTTOMHALF => setupBottomHalfCamera(cobj),
        _ => {
            HSD_Panic(c"cobj.c".as_ptr(), 624, c"unkown type of render pass.\n".as_ptr());
            return false;
        }
    };
    if !result {
        false
    } else {
        HSD_CObjSetupViewingMtx(cobj);
        true
    }
}

/// `HSD_CObjEndCurrent`.
pub fn HSD_CObjEndCurrent() {
    ext::_HSD_ZListSort();
    ext::_HSD_ZListDisp();
}

/// `HSD_CObjGetCurrent`.
pub unsafe fn HSD_CObjGetCurrent() -> *mut HSD_CObj {
    current
}

/// `HSD_CObjSetDefaultClass`.
pub unsafe fn HSD_CObjSetDefaultClass(info: *mut HSD_ClassInfo) {
    if !info.is_null() {
        // The line number is made up in the C too (the function is not in the original binary).
        hsd_assert!(1946, hsdIsDescendantOf(info as *mut c_void, class_of(addr_of_mut!(hsdCObj)) as *mut c_void), "hsdIsDescendantOf(info, &hsdCObj)");
    }
    default_class = info;
}

/// `HSD_CObjAlloc`.
pub unsafe fn HSD_CObjAlloc() -> *mut HSD_CObj {
    let info = if !default_class.is_null() { default_class } else { class_of(addr_of_mut!(hsdCObj)) };
    let cobj = hsdNew(info) as *mut HSD_CObj;
    hsd_assert!(1954, !cobj.is_null(), "cobj");
    cobj
}

/// `CObjResetFlags` (static inline).
unsafe fn CObjResetFlags(cobj: *mut HSD_CObj, flags: u32) {
    if cobj.is_null() {
        return;
    }
    (*cobj).flags = ((*cobj).flags & 0xC000_0000) | flags;
}

/// `CObjLoad` (static): the class's `load` method.
pub unsafe extern "C" fn CObjLoad(cobj: *mut HSD_CObj, desc: *mut HSD_CObjDesc) -> i32 {
    let up = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    let d = &(*desc).common;
    (*cobj).flags = d.flags as u32;
    CObjResetFlags(cobj, d.flags as u32);
    HSD_CObjSetViewport(cobj, &d.viewport);
    HSD_CObjSetScissor(cobj, &d.scissor);
    ext::HSD_WObjInit((*cobj).eyepos, d.eyepos);
    ext::HSD_WObjInit((*cobj).interest, d.interest);
    HSD_CObjSetNear(cobj, d.nnear);
    HSD_CObjSetFar(cobj, d.ffar);
    if d.flags & 1 != 0 {
        if !d.up_vector.is_null() {
            let u = &*d.up_vector;
            ext::HSD_CObjSetUpVector(cobj, Vec3 { x: u.x, y: u.y, z: u.z });
        } else {
            ext::HSD_CObjSetUpVector(cobj, up);
        }
    } else {
        ext::HSD_CObjSetRoll(cobj, d.roll);
    }
    match d.projection_type {
        1 => {
            let p = &(*desc).perspective;
            HSD_CObjSetPerspective(cobj, p.fov, p.aspect);
            // TARGET_PC Hor+ generalisation: remember the authored aspect and follow the window.
            (*cobj).view_authored_aspect = p.aspect;
            (*cobj).view_auto = 1;
            HSD_CObjUpdateView(cobj);
        }
        3 => {
            let o = &(*desc).ortho;
            HSD_CObjSetOrtho(cobj, o.top, o.bottom, o.left, o.right);
        }
        2 => {
            let f = &(*desc).frustum;
            HSD_CObjSetFrustum(cobj, f.top, f.bottom, f.left, f.right);
        }
        _ => hsd_assert!(0x7D0, false, "0"),
    }
    0
}

/// `HSD_CObjInit`.
pub unsafe fn HSD_CObjInit(cobj: *mut HSD_CObj, desc: *mut HSD_CObjDesc) {
    if cobj.is_null() || desc.is_null() {
        return;
    }
    CObjLoad(cobj, desc);
}

/// `HSD_CObjLoadDesc`.
pub unsafe fn HSD_CObjLoadDesc(desc: *mut HSD_CObjDesc) -> *mut HSD_CObj {
    if desc.is_null() {
        return null_mut();
    }
    let cobj: *mut HSD_CObj;
    let mut info: *mut HSD_ClassInfo = null_mut();
    if (*desc).class_name.is_null() || {
        info = hsdSearchClassInfo((*desc).class_name as *const c_char);
        info.is_null()
    } {
        cobj = HSD_CObjAlloc();
    } else {
        cobj = hsdNew(info) as *mut HSD_CObj;
        hsd_assert!(0x7F7, !cobj.is_null(), "cobj");
    }
    let m = (*((*cobj).parent.parent.class_info as *mut HSD_CObjInfo)).load;
    (m.unwrap())(cobj, desc);
    cobj
}

/// `CObjInit` (static): the class's `init` method.
unsafe extern "C" fn CObjInit(o: *mut HSD_Class) -> i32 {
    let parent = hsdCObj.parent.parent.head.parent;
    let status = ((*parent).init.unwrap())(o);
    if status < 0 {
        return status;
    }
    let cobj = o as *mut HSD_CObj;
    (*cobj).view_auto = 0;
    (*cobj).view_authored_aspect = 0.0;
    if !cobj.is_null() {
        HSD_CObjSetMtxDirty(cobj);
    }
    (*cobj).eyepos = ext::HSD_WObjAlloc();
    (*cobj).interest = ext::HSD_WObjAlloc();
    0
}

/// `CObjRelease`: the class's `release` method.
pub unsafe extern "C" fn CObjRelease(o: *mut HSD_Class) {
    let cobj = o as *mut HSD_CObj;
    ext::HSD_AObjRemove((*cobj).aobj);
    ext::HSD_WObjUnref(HSD_CObjGetEyePositionWObj(cobj));
    ext::HSD_WObjUnref(HSD_CObjGetInterestWObj(cobj));
    if !(*cobj).proj_mtx.is_null() {
        ext::HSD_MtxFree((*cobj).proj_mtx);
    }
    let parent = hsdCObj.parent.parent.head.parent;
    ((*parent).release.unwrap())(o);
}

/// `CObjAmnesia` (static): the class's `amnesia` method.
unsafe extern "C" fn CObjAmnesia(info: *mut HSD_ClassInfo) {
    if info == default_class {
        default_class = null_mut();
    }
    if info == class_of(addr_of_mut!(hsdCObj)) {
        current = null_mut();
    }
    let parent = hsdCObj.parent.parent.head.parent;
    ((*parent).amnesia.unwrap())(info);
}

/// `CObjInfoInit` (static).
unsafe extern "C" fn CObjInfoInit() {
    hsdInitClassInfo(
        class_of(addr_of_mut!(hsdCObj)),
        ext::hsdObj(),
        c"sysdolphin_base_library".as_ptr() as *mut c_char,
        c"hsd_cobj".as_ptr() as *mut c_char,
        core::mem::size_of::<HSD_CObjInfo>() as i32,
        core::mem::size_of::<HSD_CObj>() as i32,
    );
    let ci = class_of(addr_of_mut!(hsdCObj));
    (*ci).init = Some(CObjInit);
    (*ci).release = Some(CObjRelease);
    (*ci).amnesia = Some(CObjAmnesia);
    hsdCObj.load = Some(CObjLoad);
}
