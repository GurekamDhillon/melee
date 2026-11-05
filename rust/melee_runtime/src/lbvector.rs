
//! `src/melee/lb/lbvector.c` (+ the inlines in lbvector.h). Every contraction the C build makes
//! (clang `-ffp-contract=on`, gwtool fmadds) is explicit. `lbVector_WorldToScreen` is not ported
//! yet: it needs HSD_CObj and GXProject.

use crate::gekko::{sqrtf, sqrtf_accurate};

use crate::lbtrigf::{acosf, asinf, atan2f};

use crate::mtx::{vec_cross, Mtx, Quaternion, Vec3};

use melee_core::fp::fmadds;

pub(crate) const M_PI: f64 = 3.14159265358979323846;

pub(crate) const M_TAU: f64 = 6.283185307179586;

/// `x*x + y*y + z*z` as the C build computes it.
#[inline]
pub(crate) fn sq3(x: f32, y: f32, z: f32) -> f32 {
    fmadds(z, z, fmadds(x, x, y * y))
}

/// `x*x + y*y + z*z` with every product rounded: the console's form when the operands are memory
/// (`vec->x * vec->x`). MWCC does not fuse a square of a memory or computed expression; it does fuse
/// squares of variables (notes/fp-audit.md).
#[inline]
pub(crate) fn sq3_mem(x: f32, y: f32, z: f32) -> f32 {
    x * x + y * y + z * z
}

/// `x*x + y*y` with both products rounded (see [`sq3_mem`]).
#[inline]
pub(crate) fn sq2_mem(x: f32, y: f32) -> f32 {
    x * x + y * y
}

/// `a.x*b.x + a.y*b.y + a.z*b.z`.
#[inline]
pub(crate) fn dot3(a: &Vec3, b: &Vec3) -> f32 {
    fmadds(a.z, b.z, fmadds(a.x, b.x, a.y * b.y))
}

/// `a.x*b.x + a.y*b.y`.
#[inline]
pub(crate) fn dot2(a: &Vec3, b: &Vec3) -> f32 {
    fmadds(a.x, b.x, a.y * b.y)
}

/// `lbVector_Len` (lbvector.h) through a pointer: `vec->x * vec->x` is not fused on the console.
pub fn len(v: &Vec3) -> f32 {
    sqrtf(sq3_mem(v.x, v.y, v.z))
}

/// `lbVector_Len(&local)` where MWCC inlined it and kept the local struct in registers
/// (lbVector_8000E838): the squares fuse there.
pub fn len_local(v: &Vec3) -> f32 {
    sqrtf(sq3(v.x, v.y, v.z))
}

/// `lbVector_Len_xy` (lbvector.h) through a pointer: not fused (see [`len`]).
pub fn len_xy(v: &Vec3) -> f32 {
    sqrtf(sq2_mem(v.x, v.y))
}

pub(crate) fn len_xy_accurate(v: &Vec3) -> f32 {
    sqrtf_accurate(sq2_mem(v.x, v.y))
}

/// `lbVector_Normalize`: returns the length; a zero vector is left alone.
pub fn normalize(v: &mut Vec3) -> f32 {
    let l = len(v);
    if l == 0.0 {
        return 0.0;
    }
    let inv = 1.0f32 / l;
    v.x *= inv;
    v.y *= inv;
    v.z *= inv;
    l
}

/// `lbVector_NormalizeXY`.
pub fn normalize_xy(a: &mut Vec3) -> f32 {
    let l = sqrtf_accurate(sq2_mem(a.x, a.y));
    if l == 0.0 {
        return 0.0;
    }
    let inv = 1.0f32 / l;
    a.x *= inv;
    a.y *= inv;
    l
}

/// `lbVector_Add`.
pub fn add(a: &mut Vec3, b: &Vec3) {
    a.x += b.x;
    a.y += b.y;
    a.z += b.z;
}

/// `lbVector_Add_xy`.
pub fn add_xy(a: &mut Vec3, b: &Vec3) {
    a.x += b.x;
    a.y += b.y;
}

/// `lbVector_Sub`.
pub fn sub(a: &mut Vec3, b: &Vec3) {
    a.x -= b.x;
    a.y -= b.y;
    a.z -= b.z;
}

/// `lbVector_Diff`: a - b.
pub fn diff(a: &Vec3, b: &Vec3) -> Vec3 {
    Vec3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}

/// `lbVector_CrossprodNormalized`.
pub fn crossprod_normalized(a: &Vec3, b: &Vec3) -> Vec3 {
    let mut r = vec_cross(a, b);
    normalize(&mut r);
    r
}

pub(crate) fn clamp_cos(c: f32) -> f32 {
    let mut c = c;
    if c > 1.0 {
        c = 1.0;
    }
    if c < -1.0 {
        c = -1.0;
    }
    c
}

/// `lbVector_Angle` (8000D620).
pub fn angle(a: &Vec3, b: &Vec3) -> f32 {
    let ll = len(a) * len(b);
    if ll > 0.0000000001f32 {
        return acosf(clamp_cos(dot3(a, b) / ll));
    }
    0.0
}

/// `lbVector_AngleXY` (8000D790).
pub fn angle_xy(a: &Vec3, b: &Vec3) -> f32 {
    let ll = len_xy_accurate(a) * len_xy_accurate(b);
    if ll != 0.0 {
        return acosf(clamp_cos(dot2(a, b) / ll));
    }
    0.0
}

/// The quintic `0.98786199f*a - 0.155271f*a^3 + 0.005643f*a^5`, fused as the C build does.
#[inline]
pub(crate) fn quintic(a: f32) -> f32 {
    let inner = fmadds(0.9878619909286499f32, a, -(0.15527099370956421f32 * a * a * a));
    fmadds(0.0056429998949170113f32 * a * a * a * a, a, inner)
}

/// Wrap into [-pi, pi] once; the comparisons and steps are in double (M_PI is a double).
#[inline]
pub(crate) fn wrap(a: f32) -> f32 {
    if a as f64 > M_PI {
        (a as f64 - M_TAU) as f32
    } else if (a as f64) < -M_PI {
        (a as f64 + M_TAU) as f32
    } else {
        a
    }
}

/// `lbVector_RotateAboutUnitAxis` (8000D8F4).
pub fn rotate_about_unit_axis(v: &mut Vec3, axis: &Vec3, angle: f32) {
    let len_yz = sqrtf(sq2_mem(axis.y, axis.z));
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

/// `lbVector_Rotate`: axis 1 = x, 2 = y, 4 = z. Any other axis zeroes `v`: the C reads
/// uninitialised locals there, which the build's `-ftrivial-auto-var-init=zero` makes 0.
pub fn rotate(v: &mut Vec3, axis: i32, angle: f32) {
    let s = lbvector_sin(angle);
    let c = lbvector_cos(angle);
    let (x, y, z) = match axis {
        1 => (v.x, fmadds(v.y, c, -(v.z * s)), fmadds(v.y, s, v.z * c)),
        2 => (fmadds(v.x, c, v.z * s), v.y, fmadds(v.z, c, -(v.x * s))),
        4 => (fmadds(v.x, c, -(v.y * s)), fmadds(v.x, s, v.y * c), v.z),
        _ => (0.0, 0.0, 0.0),
    };
    v.x = x;
    v.y = y;
    v.z = z;
}

/// `lbVector_Mirror` (8000DC6C): a -= 2<a,b>b in xy.
pub fn mirror(a: &mut Vec3, m: &Vec3) {
    let f = fmadds(m.x, a.x, m.y * a.y) * -2.0f32;
    a.x = fmadds(m.x, f, a.x);
    a.y = fmadds(m.y, f, a.y);
}

/// `lbVector_CosAngle` (8000DCA8).
pub fn cos_angle(a: &Vec3, b: &Vec3) -> f32 {
    dot2(a, b) / (sqrtf(sq2_mem(a.x, a.y)) * sqrtf(sq2_mem(b.x, b.y)))
}

/// `lbVector_Lerp` (8000DDAC): a + f*(b-a).
pub fn lerp(a: &Vec3, b: &Vec3, f: f32) -> Vec3 {
    let mut r = diff(b, a);
    r.x *= f;
    r.y *= f;
    r.z *= f;
    add(&mut r, a);
    r
}

/// `lbVector_8000DE38`: a quadratic Bezier through three points packed in `m`.
pub fn bezier_8000de38(m: &Mtx, c: f32) -> Vec3 {
    let mut c = c;
    if c > 1.0 {
        c = 1.0;
    } else if c < 0.0 {
        c = 0.0;
    }
    let comp = |p0: f32, p1: f32, p2: f32| {
        let var1 = fmadds(p2, 2.0, fmadds(p0, 2.0, -(p1 * 4.0)));
        let var2 = fmadds(p0, -3.0, p1 * 4.0) - p2;
        p0 + fmadds(var1 * c, c, var2 * c)
    };
    Vec3 {
        x: comp(m[0][0], m[0][3], m[1][2]),
        y: comp(m[0][1], m[1][0], m[1][3]),
        z: comp(m[0][2], m[1][1], m[2][0]),
    }
}

/// `lbVector_EulerAnglesFromONB` (8000DF0C).
pub fn euler_angles_from_onb(a: &Vec3, b: &Vec3, c: &Vec3) -> Vec3 {
    let mut r = Vec3::default();
    if b.z == -1.0 || b.z == 1.0 {
        if b.z == -1.0 {
            r.y = 1.5707963705062866f32;
            r.x = atan2f(c.x, c.y);
        } else {
            r.y = -1.5707963705062866f32;
            r.x = atan2f(-c.x, c.y);
        }
        r.z = 0.0;
    } else {
        r.y = asinf(-b.z);
        r.x = atan2f(c.z, a.z);
        r.z = atan2f(b.y, b.x);
    }
    r
}

/// `lbVector_EulerAnglesFromPartialONB` (8000DFF4).
pub fn euler_angles_from_partial_onb(a: &Vec3, c: &Vec3) -> Vec3 {
    let mut b = vec_cross(c, a);
    normalize(&mut b);
    euler_angles_from_onb(a, &b, c)
}

/// `lbVector_ApplyEulerRotation` (8000E138).
pub fn apply_euler_rotation(v: &mut Vec3, angles: &Vec3) {
    rotate(v, 1, angles.x);
    rotate(v, 2, angles.y);
    rotate(v, 4, angles.z);
}

/// `lbVector_sqrtf_accurate` (8000E19C).
pub fn lb_sqrtf_accurate(x: f32) -> f32 {
    sqrtf_accurate(x)
}

/// `lbVector_CreateEulerMatrix` (8000E530). Takes the angles from a Quaternion's x, y, z.
pub fn create_euler_matrix(angles: &Quaternion) -> Mtx {
    let sx = lbvector_sin(angles.x);
    let cx = lbvector_cos(angles.x);
    let sy = lbvector_sin(angles.y);
    let cy = lbvector_cos(angles.y);
    let sz = lbvector_sin(angles.z);
    let cz = lbvector_cos(angles.z);
    let sxsy = sx * sy;
    let cxsy = cx * sy;
    [
        [cy * cz, fmadds(cz, sxsy, -(cx * sz)), fmadds(cz, cxsy, sx * sz), 0.0],
        [cy * sz, fmadds(sz, sxsy, cx * cz), fmadds(sz, cxsy, -(sx * cz)), 0.0],
        [-sy, sx * cy, cx * cy, 0.0],
    ]
}

/// `lbVector_8000E838`: project c onto the line a->b; returns (|c - d|, d).
pub fn project_8000e838(a: &Vec3, b: &Vec3, c: &Vec3) -> (f32, Vec3) {
    let b_a = diff(b, a);
    let sqrlen = sq3(b_a.x, b_a.y, b_a.z);
    let c_a = diff(c, a);
    if sqrlen < 9.9999997473787516e-06f32 && sqrlen > -9.9999997473787516e-06f32 {
        return (len_local(&c_a), *a);
    }
    let f1 = dot3(&b_a, &c_a) / sqrlen;
    let d = Vec3 {
        x: fmadds(b_a.x, f1, a.x),
        y: fmadds(b_a.y, f1, a.y),
        z: fmadds(b_a.z, f1, a.z),
    };
    (len_local(&diff(c, &d)), d)
}

pub use crate::tu::lbvector::lbvector_cos_fp as lbvector_cos;
pub use crate::tu::lbvector::lbvector_sin_fp as lbvector_sin;
