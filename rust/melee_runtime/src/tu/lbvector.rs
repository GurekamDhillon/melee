//! Port of src/melee/lb/lbvector.c: the game's Vec3 helpers (normalize, angle, quintic sin/cos, rotation
//! about an axis, euler angles, the quadratic Bezier, the WorldToScreen projection) with the C names and
//! pointer signatures. The float code follows `crate::lbvector` (ported and checked earlier) re-applied on
//! pointers; functions that call other TUs (`acosf`, `asinf`, `atan2f`, `PSVECCrossProduct`, the CObj
//! getters, `GXProject`, ...) go through `ext` (a hook, else the port, else panics "TODO"). Every
//! contraction of the shipping build is explicit (`fmadds`). Checked against the C (whole TU, shipping
//! contraction) by tests/lbtu_lbvector.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_types::*;

use crate::gekko::{sqrtf, sqrtf_accurate};

// BEGIN GENERATED EXT (rust/melee_runtime/gen/lbtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code, unreachable_code)]
    use super::*;
    use core::ffi::c_void;
    pub mod hook {
        use super::*;
        pub static mut C_MTXLookAt: Option<fn(*mut [f32; 4], *mut Vec, *mut Vec, *mut Vec)> = None;
        pub static mut GXProject: Option<fn(f32, f32, f32, *mut [f32; 4], *mut f32, *mut f32, *mut f32, *mut f32, *mut f32)> = None;
        pub static mut HSD_CObjGetEyePosition: Option<fn(*mut HSD_CObj, *mut Vec)> = None;
        pub static mut HSD_CObjGetInterest: Option<fn(*mut HSD_CObj, *mut Vec)> = None;
        pub static mut HSD_CObjGetProjectionType: Option<fn(*mut HSD_CObj) -> i32> = None;
        pub static mut HSD_CObjGetUpVector: Option<fn(*mut HSD_CObj, *mut Vec) -> i32> = None;
        pub static mut HSD_CObjGetViewingMtxPtr: Option<fn(*mut HSD_CObj) -> *mut [f32; 4]> = None;
        pub static mut MTXOrtho: Option<fn(*mut [f32; 4], f32, f32, f32, f32, f32, f32)> = None;
        pub static mut MTXPerspective: Option<fn(*mut [f32; 4], f32, f32, f32, f32)> = None;
        pub static mut PSVECCrossProduct: Option<fn(*mut Vec, *mut Vec, *mut Vec)> = None;
        pub static mut acosf: Option<fn(f32) -> f32> = None;
        pub static mut asinf: Option<fn(f32) -> f32> = None;
        pub static mut atan2f: Option<fn(f32, f32) -> f32> = None;
    }
    pub fn C_MTXLookAt(a0: *mut [f32; 4], a1: *mut Vec, a2: *mut Vec, a3: *mut Vec) {
        match unsafe { hook::C_MTXLookAt } { Some(f) => f(a0, a1, a2, a3), None => { panic!("TODO: C_MTXLookAt is not ported") } }
    }
    pub fn GXProject(a0: f32, a1: f32, a2: f32, a3: *mut [f32; 4], a4: *mut f32, a5: *mut f32, a6: *mut f32, a7: *mut f32, a8: *mut f32) {
        match unsafe { hook::GXProject } { Some(f) => f(a0, a1, a2, a3, a4, a5, a6, a7, a8), None => { panic!("TODO: GXProject is not ported") } }
    }
    pub fn HSD_CObjGetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec) {
        match unsafe { hook::HSD_CObjGetEyePosition } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjGetEyePosition is not ported") } }
    }
    pub fn HSD_CObjGetInterest(a0: *mut HSD_CObj, a1: *mut Vec) {
        match unsafe { hook::HSD_CObjGetInterest } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjGetInterest is not ported") } }
    }
    pub fn HSD_CObjGetProjectionType(a0: *mut HSD_CObj) -> i32 {
        match unsafe { hook::HSD_CObjGetProjectionType } { Some(f) => f(a0), None => { { return unsafe { crate::tu::cobj::HSD_CObjGetProjectionType(a0 as _) as _ }; } } }
    }
    pub fn HSD_CObjGetUpVector(a0: *mut HSD_CObj, a1: *mut Vec) -> i32 {
        match unsafe { hook::HSD_CObjGetUpVector } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjGetUpVector is not ported") } }
    }
    pub fn HSD_CObjGetViewingMtxPtr(a0: *mut HSD_CObj) -> *mut [f32; 4] {
        match unsafe { hook::HSD_CObjGetViewingMtxPtr } { Some(f) => f(a0), None => { panic!("TODO: HSD_CObjGetViewingMtxPtr is not ported") } }
    }
    pub fn MTXOrtho(a0: *mut [f32; 4], a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) {
        match unsafe { hook::MTXOrtho } { Some(f) => f(a0, a1, a2, a3, a4, a5, a6), None => { panic!("TODO: MTXOrtho is not ported") } }
    }
    pub fn MTXPerspective(a0: *mut [f32; 4], a1: f32, a2: f32, a3: f32, a4: f32) {
        match unsafe { hook::MTXPerspective } { Some(f) => f(a0, a1, a2, a3, a4), None => { panic!("TODO: MTXPerspective is not ported") } }
    }
    pub fn PSVECCrossProduct(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) {
        match unsafe { hook::PSVECCrossProduct } { Some(f) => f(a0, a1, a2), None => { return unsafe { psvec_cross_default(a0, a1, a2) } } }
    }
    pub fn acosf(a0: f32) -> f32 {
        match unsafe { hook::acosf } { Some(f) => f(a0), None => { return unsafe { crate::lbtrigf::acosf(a0) } } }
    }
    pub fn asinf(a0: f32) -> f32 {
        match unsafe { hook::asinf } { Some(f) => f(a0), None => { return unsafe { crate::lbtrigf::asinf(a0) } } }
    }
    pub fn atan2f(a0: f32, a1: f32) -> f32 {
        match unsafe { hook::atan2f } { Some(f) => f(a0, a1), None => { return unsafe { crate::lbtrigf::atan2f(a0, a1) } } }
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

const M_PI: f64 = 3.14159265358979323846;
const M_TAU: f64 = 6.283185307179586;

/// `PSVECCrossProduct` (the dolphin SDK function), as the default of the ext hook.
pub unsafe fn psvec_cross_default(a: *const Vec, b: *const Vec, r: *mut Vec) {
    let (a, b) = (&*a, &*b);
    let ax = crate::mtx::Vec3 { x: a.x, y: a.y, z: a.z };
    let bx = crate::mtx::Vec3 { x: b.x, y: b.y, z: b.z };
    let c = crate::mtx::vec_cross(&ax, &bx);
    (*r).x = c.x;
    (*r).y = c.y;
    (*r).z = c.z;
}

#[inline]
fn sq3(x: f32, y: f32, z: f32) -> f32 {
    fmadds(z, z, fmadds(x, x, y * y))
}
#[inline]
fn dot3(a: &Vec, b: &Vec) -> f32 {
    fmadds(a.z, b.z, fmadds(a.x, b.x, a.y * b.y))
}
#[inline]
fn dot2(a: &Vec, b: &Vec) -> f32 {
    fmadds(a.x, b.x, a.y * b.y)
}

/// `lbVector_Len` (lbvector.h inline).
pub unsafe fn lbVector_Len(v: *const Vec) -> f32 {
    sqrtf(sq3((*v).x, (*v).y, (*v).z))
}

/// `lbVector_Len_xy` (lbvector.h inline).
pub unsafe fn lbVector_Len_xy(v: *const Vec) -> f32 {
    sqrtf(dot2(&*v, &*v))
}

/// `lbVector_Len_xy_accurate` (static inline).
pub unsafe fn lbVector_Len_xy_accurate(v: *const Vec) -> f32 {
    sqrtf_accurate(dot2(&*v, &*v))
}

pub unsafe fn lbVector_Normalize(vec: *mut Vec) -> f32 {
    let len = lbVector_Len(vec);
    if len == 0.0 {
        return 0.0;
    }
    let inv = 1.0f32 / len;
    (*vec).x *= inv;
    (*vec).y *= inv;
    (*vec).z *= inv;
    len
}

pub unsafe fn lbVector_NormalizeXY(a: *mut Vec) -> f32 {
    let len = sqrtf_accurate(dot2(&*a, &*a));
    if len == 0.0 {
        return 0.0;
    }
    let inv = 1.0f32 / len;
    (*a).x *= inv;
    (*a).y *= inv;
    len
}

pub unsafe fn lbVector_Add(a: *mut Vec, b: *mut Vec) -> *mut Vec {
    (*a).x += (*b).x;
    (*a).y += (*b).y;
    (*a).z += (*b).z;
    a
}

pub unsafe fn lbVector_Add_xy(a: *mut Vec, b: *mut Vec) -> *mut Vec {
    (*a).x += (*b).x;
    (*a).y += (*b).y;
    a
}

pub unsafe fn lbVector_Sub(a: *mut Vec, b: *mut Vec) -> *mut Vec {
    (*a).x -= (*b).x;
    (*a).y -= (*b).y;
    (*a).z -= (*b).z;
    a
}

pub unsafe fn lbVector_Diff(a: *mut Vec, b: *mut Vec, result: *mut Vec) -> *mut Vec {
    (*result).x = (*a).x - (*b).x;
    (*result).y = (*a).y - (*b).y;
    (*result).z = (*a).z - (*b).z;
    result
}

pub unsafe fn lbVector_CrossprodNormalized(a: *mut Vec, b: *mut Vec, result: *mut Vec) -> *mut Vec {
    ext::PSVECCrossProduct(a, b, result);
    lbVector_Normalize(result);
    result
}

fn clamp_cos(c: f32) -> f32 {
    let mut c = c;
    if c > 1.0 {
        c = 1.0;
    }
    if c < -1.0 {
        c = -1.0;
    }
    c
}

/// 8000D620: the angle between a and b.
pub unsafe fn lbVector_Angle(a: *mut Vec, b: *mut Vec) -> f32 {
    let lena_lenb = lbVector_Len(a) * lbVector_Len(b);
    if lena_lenb > 0.0000000001f32 {
        let cosine = dot3(&*a, &*b) / lena_lenb;
        return ext::acosf(clamp_cos(cosine));
    }
    0.0
}

/// 8000D790: the angle between a and b in the xy plane.
pub unsafe fn lbVector_AngleXY(a: *mut Vec, b: *mut Vec) -> f32 {
    let lena_lenb = lbVector_Len_xy_accurate(a) * lbVector_Len_xy_accurate(b);
    if lena_lenb != 0.0 {
        let cosine = dot2(&*a, &*b) / lena_lenb;
        return ext::acosf(clamp_cos(cosine));
    }
    0.0
}

#[inline]
fn quintic(a: f32) -> f32 {
    let inner = fmadds(0.9878619909286499f32, a, -(0.15527099370956421f32 * a * a * a));
    fmadds(0.0056429998949170113f32 * a * a * a * a, a, inner)
}

#[inline]
fn wrap(a: f32) -> f32 {
    if a as f64 > M_PI {
        (a as f64 - M_TAU) as f32
    } else if (a as f64) < -M_PI {
        (a as f64 + M_TAU) as f32
    } else {
        a
    }
}

/// `lbvector_sin` (static).
pub fn lbvector_sin(angle: f32) -> f32 {
    quintic(wrap(angle))
}

/// `lbvector_cos` (static).
pub fn lbvector_cos(angle: f32) -> f32 {
    let a = (angle as f64 + M_PI / 2.0) as f32;
    quintic(wrap(a))
}

/// 8000D8F4: rotates v by angle about the unit axis.
pub unsafe fn lbVector_RotateAboutUnitAxis(v: *mut Vec, axis: *mut Vec, angle: f32) {
    let (v, axis) = (&mut *v, &*axis);
    let len_yz = sqrtf(fmadds(axis.y, axis.y, axis.z * axis.z));
    let s = lbvector_sin(angle);
    let c = lbvector_cos(angle);
    let (mut uy, mut uz) = (0.0f32, 0.0f32);
    let (mut x, mut y, mut z);
    if len_yz > 0.0000000001f32 {
        uy = axis.z / len_yz;
        uz = axis.y / len_yz;
        x = v.x;
        y = fmadds(v.y, uy, -(v.z * uz));
        z = fmadds(v.y, uz, v.z * uy);
    } else {
        x = v.x;
        y = v.y;
        z = v.z;
    }
    let x2 = fmadds(x, len_yz, -(z * axis.x));
    let z2 = fmadds(x, axis.x, z * len_yz);
    let x3 = fmadds(x2, c, -(y * s));
    let y3 = fmadds(x2, s, y * c);
    x = fmadds(x3, len_yz, z2 * axis.x);
    y = y3;
    z = fmadds(-x3, axis.x, z2 * len_yz);
    if len_yz > 0.0000000001f32 {
        v.x = x;
        v.y = fmadds(y, uy, z * uz);
        v.z = fmadds(-y, uz, z * uy);
    } else {
        v.x = x;
        v.y = y;
        v.z = z;
    }
}

/// axis 1 = x, 2 = y, 4 = z; any other axis zeroes `v` (the C reads uninitialised locals, which the
/// build's `-ftrivial-auto-var-init=zero` makes 0).
pub unsafe fn lbVector_Rotate(v: *mut Vec, axis: i32, angle: f32) {
    let s = lbvector_sin(angle);
    let c = lbvector_cos(angle);
    let vv = &mut *v;
    let (x, y, z) = match axis {
        1 => (vv.x, fmadds(vv.y, c, -(vv.z * s)), fmadds(vv.y, s, vv.z * c)),
        2 => (fmadds(vv.x, c, vv.z * s), vv.y, fmadds(vv.z, c, -(vv.x * s))),
        4 => (fmadds(vv.x, c, -(vv.y * s)), fmadds(vv.x, s, vv.y * c), vv.z),
        _ => (0.0, 0.0, 0.0),
    };
    vv.x = x;
    vv.y = y;
    vv.z = z;
}

/// 8000DC6C: a -= 2<a,b>b in xy.
pub unsafe fn lbVector_Mirror(a: *mut Vec, unit_mirror_axis: *mut Vec) {
    let (a, m) = (&mut *a, &*unit_mirror_axis);
    let f = fmadds(m.x, a.x, m.y * a.y) * -2.0f32;
    a.x = fmadds(m.x, f, a.x);
    a.y = fmadds(m.y, f, a.y);
}

/// 8000DCA8
pub unsafe fn lbVector_CosAngle(a: *mut Vec, b: *mut Vec) -> f32 {
    dot2(&*a, &*b) / (sqrtf(dot2(&*a, &*a)) * sqrtf(dot2(&*b, &*b)))
}

/// 8000DDAC: a + f*(b-a).
pub unsafe fn lbVector_Lerp(a: *mut Vec, b: *mut Vec, result: *mut Vec, f: f32) -> *mut Vec {
    lbVector_Diff(b, a, result);
    (*result).x *= f;
    (*result).y *= f;
    (*result).z *= f;
    lbVector_Add(result, a);
    result
}

/// `Mtx m` is a `float[3][4]`.
pub unsafe fn lbVector_8000DE38(m: *mut [f32; 4], v: *mut Vec, c: f32) -> *mut Vec {
    let mut c = c;
    if c > 1.0 {
        c = 1.0;
    } else if c < 0.0 {
        c = 0.0;
    }
    let m = core::slice::from_raw_parts(m, 3);
    let comp = |p0: f32, p1: f32, p2: f32| {
        let var1 = fmadds(p2, 2.0, fmadds(p0, 2.0, -(p1 * 4.0)));
        let var2 = fmadds(p0, -3.0, p1 * 4.0) - p2;
        p0 + fmadds(var1 * c, c, var2 * c)
    };
    (*v).x = comp(m[0][0], m[0][3], m[1][2]);
    (*v).y = comp(m[0][1], m[1][0], m[1][3]);
    (*v).z = comp(m[0][2], m[1][1], m[2][0]);
    v
}

/// 8000DF0C
pub unsafe fn lbVector_EulerAnglesFromONB(result_angles: *mut Vec, a: *mut Vec, b: *mut Vec, c: *mut Vec) -> *mut Vec {
    let (a, b, c) = (&*a, &*b, &*c);
    let r = &mut *result_angles;
    if b.z == -1.0 || b.z == 1.0 {
        if b.z == -1.0 {
            r.y = 1.5707963705062866f32;
            r.x = ext::atan2f(c.x, c.y);
        } else {
            r.y = -1.5707963705062866f32;
            r.x = ext::atan2f(-c.x, c.y);
        }
        r.z = 0.0;
    } else {
        r.y = ext::asinf(-b.z);
        r.x = ext::atan2f(c.z, a.z);
        r.z = ext::atan2f(b.y, b.x);
    }
    result_angles
}

/// 8000DFF4
pub unsafe fn lbVector_EulerAnglesFromPartialONB(result_angles: *mut Vec, a: *mut Vec, c: *mut Vec) -> *mut Vec {
    let mut b = Vec { x: 0.0, y: 0.0, z: 0.0 };
    ext::PSVECCrossProduct(c, a, &mut b);
    lbVector_Normalize(&mut b);
    lbVector_EulerAnglesFromONB(result_angles, a, &mut b, c);
    result_angles
}

/// 8000E138
pub unsafe fn lbVector_ApplyEulerRotation(v: *mut Vec, angles: *mut Vec) -> *mut Vec {
    lbVector_Rotate(v, 1, (*angles).x);
    lbVector_Rotate(v, 2, (*angles).y);
    lbVector_Rotate(v, 4, (*angles).z);
    v
}

/// 8000E19C
pub fn lbVector_sqrtf_accurate(x: f32) -> f32 {
    sqrtf_accurate(x)
}

/// 8000E210: projects a world position to screen coordinates (`d != 0` rebuilds the view matrix with
/// `C_MTXLookAt` instead of using the camera's).
pub unsafe fn lbVector_WorldToScreen(cobj: *mut HSD_CObj, pos3d: *const Vec, screen_coords: *mut Vec, d: i32) -> *mut Vec {
    assert!(!pos3d.is_null(), "HSD_ASSERT 676 pos3d");
    assert!((*pos3d).x > -50000.0 && (*pos3d).x < 50000.0, "HSD_ASSERT 677");
    assert!((*pos3d).y > -50000.0 && (*pos3d).y < 50000.0, "HSD_ASSERT 678");
    assert!((*pos3d).z > -50000.0 && (*pos3d).z < 50000.0, "HSD_ASSERT 679");

    let mut proj_mtx: Mtx44 = [[0.0; 4]; 4];
    let mut projection = [0.0f32; 7];
    let mut viewport = [0.0f32; 6];
    let mut m: Mtx = [[0.0; 4]; 3];
    let mut up_vec = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut cam_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut target = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut point = *pos3d;
    let c = &*cobj;
    match ext::HSD_CObjGetProjectionType(cobj) {
        1 => {
            ext::MTXPerspective(proj_mtx.as_mut_ptr(), c.projection_param.perspective.fov, c.projection_param.perspective.aspect, c.near, c.far);
            projection[0] = 0.0;
            projection[1] = proj_mtx[0][0];
            projection[2] = proj_mtx[0][2];
            projection[3] = proj_mtx[1][1];
            projection[4] = proj_mtx[1][2];
            projection[5] = proj_mtx[2][2];
            projection[6] = proj_mtx[2][3];
        }
        3 => {
            ext::MTXOrtho(proj_mtx.as_mut_ptr(), c.projection_param.ortho.top, c.projection_param.ortho.bottom, c.projection_param.ortho.left, c.projection_param.ortho.right, c.near, c.far);
            projection[0] = 1.0;
            projection[1] = proj_mtx[0][0];
            projection[2] = proj_mtx[0][3];
            projection[3] = proj_mtx[1][1];
            projection[4] = proj_mtx[1][3];
            projection[5] = proj_mtx[2][2];
            projection[6] = proj_mtx[2][3];
        }
        _ => return core::ptr::null_mut(),
    }
    viewport[0] = c.viewport.xmin;
    viewport[1] = c.viewport.ymin;
    viewport[2] = c.viewport.xmax - c.viewport.xmin;
    viewport[3] = c.viewport.ymax - c.viewport.ymin;
    viewport[4] = 0.0;
    viewport[5] = 1.0;

    let mv_mtx: *mut [f32; 4];
    if d != 0 {
        ext::HSD_CObjGetEyePosition(cobj, &mut cam_pos);
        ext::HSD_CObjGetUpVector(cobj, &mut up_vec);
        ext::HSD_CObjGetInterest(cobj, &mut target);
        ext::C_MTXLookAt(m.as_mut_ptr(), &mut cam_pos, &mut up_vec, &mut target);
        mv_mtx = m.as_mut_ptr();
    } else {
        mv_mtx = ext::HSD_CObjGetViewingMtxPtr(cobj);
    }
    let mv = core::slice::from_raw_parts(mv_mtx, 3);
    let p = &*pos3d;
    let mut f1 = fmadds(mv[2][2], p.z, fmadds(mv[2][0], p.x, mv[2][1] * p.y)) + mv[2][3];
    if f1 > -0.01 {
        f1 = -f1 - 0.01;
        point.x = fmadds(mv[2][0], f1, point.x);
        point.y = fmadds(mv[2][1], f1, point.y);
        point.z = fmadds(mv[2][2], f1, point.z);
    }
    ext::GXProject(point.x, point.y, point.z, mv_mtx, projection.as_mut_ptr(), viewport.as_mut_ptr(),
        &mut (*screen_coords).x, &mut (*screen_coords).y, &mut (*screen_coords).z);
    screen_coords
}

/// 8000E530: the 3x3 euler matrix from angles.x/y/z (column 4 zero).
pub unsafe fn lbVector_CreateEulerMatrix(m: *mut [f32; 4], angles: *mut Quaternion) {
    let m = core::slice::from_raw_parts_mut(m, 3);
    let a = &*angles;
    let sx = lbvector_sin(a.x);
    let cx = lbvector_cos(a.x);
    let sy = lbvector_sin(a.y);
    let cy = lbvector_cos(a.y);
    let sz = lbvector_sin(a.z);
    let cz = lbvector_cos(a.z);
    let sxsy = sx * sy;
    let cxsy = cx * sy;
    m[0][0] = cy * cz;
    m[1][0] = cy * sz;
    m[2][0] = -sy;
    m[0][1] = fmadds(cz, sxsy, -(cx * sz));
    m[1][1] = fmadds(sz, sxsy, cx * cz);
    m[2][1] = sx * cy;
    m[0][2] = fmadds(cz, cxsy, sx * sz);
    m[1][2] = fmadds(sz, cxsy, -(sx * cz));
    m[2][2] = cx * cy;
    m[0][3] = 0.0;
    m[1][3] = 0.0;
    m[2][3] = 0.0;
}

/// 8000E838: project c onto the line a->b into d; returns |c - d|.
pub unsafe fn lbVector_8000E838(a: *mut Vec, b: *mut Vec, c: *mut Vec, d: *mut Vec) -> f32 {
    let mut b_a = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut c_a = Vec { x: 0.0, y: 0.0, z: 0.0 };
    lbVector_Diff(b, a, &mut b_a);
    let sqrlen_b_a = sq3(b_a.x, b_a.y, b_a.z);
    lbVector_Diff(c, a, &mut c_a);
    let too_small = sqrlen_b_a < 9.9999997473787516e-06f32 && sqrlen_b_a > -9.9999997473787516e-06f32;
    if too_small {
        *d = *a;
        lbVector_Len(&c_a)
    } else {
        let mut v3 = Vec { x: 0.0, y: 0.0, z: 0.0 };
        let f1 = dot3(&b_a, &c_a) / sqrlen_b_a;
        (*d).x = fmadds(b_a.x, f1, (*a).x);
        (*d).y = fmadds(b_a.y, f1, (*a).y);
        (*d).z = fmadds(b_a.z, f1, (*a).z);
        lbVector_Diff(c, d, &mut v3);
        lbVector_Len(&v3)
    }
}
