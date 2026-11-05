//! Dolphin SDK matrix / vector routines. Port of `pc/gameworld/mtx_pc.c` (the PS* stand-ins for
//! the SDK's paired-single asm) and `libs/dolphin/src/dolphin/mtx/mtx44.c`.
//!
//! Every C expression that the real build contracts into `llvm.fmuladd` (clang
//! `-ffp-contract=on`, lowered by gwtool to [`fmadds`]) is written out as [`fmadds`] here; every
//! other operation keeps its C grouping. Contraction rule (clang `tryEmitFMulAdd`): in `A + B`
//! or `A - B`, if A is a multiply it is fused, else if B is; `X - a*b` is `fmadds(-a, b, X)` and
//! `a*b - X` is `fmadds(a, b, -X)`. Calls, casts and statement boundaries stop contraction.
//!
//! Aliasing: the C versions accept `dst == src`; here sources are borrowed and the result is
//! returned by value, which is what the C does through its temporary in the aliased case.

use crate::gekko::{cosf, frsqrte, sinf, sqrtf, tanf};
use melee_core::fp::{fmadds, force25};

pub use melee_core::vec::Vec3;

/// `Quaternion` (`{ f32 x, y, z, w; }`). TODO(move-to-core).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
melee_core::gc_layout!(Quaternion, "Quaternion", size 0x10, align 4, [
    x @ 0x0: 4 "f32",
    y @ 0x4: 4 "f32",
    z @ 0x8: 4 "f32",
    w @ 0xC: 4 "f32",
]);

/// `Mtx` = `f32[3][4]` (GC: 0x30 bytes, row-major, same element order natively).
pub type Mtx = [[f32; 4]; 3];
/// `Mtx44` = `f32[4][4]`.
pub type Mtx44 = [[f32; 4]; 4];

pub const IDENTITY: Mtx = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];

pub fn identity() -> Mtx {
    IDENTITY
}

/// `PSMTXConcat`: the retail paired-single dataflow (see mtx_pc.c).
pub fn concat(a: &Mtx, b: &Mtx) -> Mtx {
    let mut m = [[0.0f32; 4]; 3];
    for i in 0..3 {
        for j in 0..4 {
            let mut t = b[0][j] * a[i][0];
            t = fmadds(b[1][j], a[i][1], t);
            t = fmadds(b[2][j], a[i][2], t);
            if j == 3 {
                t = fmadds(1.0, a[i][3], t);
            } else if j == 2 {
                t = fmadds(0.0, a[i][3], t);
            }
            m[i][j] = t;
        }
    }
    m
}

/// `PSMTXTranspose`: 3x3 transposed, translation cleared.
pub fn transpose(s: &Mtx) -> Mtx {
    [
        [s[0][0], s[1][0], s[2][0], 0.0],
        [s[0][1], s[1][1], s[2][1], 0.0],
        [s[0][2], s[1][2], s[2][2], 0.0],
    ]
}

/// `a*b - c*d` as the C build computes it: the left product fused.
#[inline(always)]
fn diff2(a: f32, b: f32, c: f32, d: f32) -> f32 {
    fmadds(a, b, -(c * d))
}

/// `PSMTXInverse`. `None` when singular (the C leaves `inv` untouched and returns 0).
pub fn inverse(s: &Mtx) -> Option<Mtx> {
    // The retail paired-single PSMTXInverse (libs/dolphin/src/dolphin/mtx/mtx.c), not the C_MTX
    // reference: cofactors as ps_mul + ps_msub (one rounding each), det by ps_mul + 2 ps_madd,
    // 1/det as fres plus one Newton step (-(det*r*r - 2r), ps_nmsub), the translation column by
    // ps_mul + ps_madd + ps_nmadd. pc/gameworld/mtx_pc.c does the same, one statement per rounding.
    let msub = |a: f32, c: f32, b: f32| (a as f64 * c as f64 - b as f64) as f32;
    let c00 = msub(s[1][1], s[2][2], s[2][1] * s[1][2]);
    let c10 = msub(s[1][2], s[2][0], s[2][2] * s[1][0]);
    let c01 = msub(s[2][1], s[0][2], s[0][1] * s[2][2]);
    let c11 = msub(s[2][2], s[0][0], s[0][2] * s[2][0]);
    let c02 = msub(s[0][1], s[1][2], s[1][1] * s[0][2]);
    let c12 = msub(s[0][2], s[1][0], s[1][2] * s[0][0]);
    let c20 = msub(s[1][0], s[2][1], s[1][1] * s[2][0]);
    let c21 = msub(s[0][1], s[2][0], s[0][0] * s[2][1]);
    let c22 = msub(s[0][0], s[1][1], s[0][1] * s[1][0]);
    let mut det = s[0][0] * c00;
    det = fmadds(s[1][0], c01, det);
    det = fmadds(s[2][0], c02, det);
    if det == 0.0 {
        return None;
    }
    let r = melee_core::fp::fres(det as f64) as f32;
    let r2 = r + r;
    let rr = r * r;
    let i = -msub(det, rr, r2);
    let c = [[c00, c01, c02], [c10, c11, c12], [c20, c21, c22]];
    let mut m = [[0.0f32; 4]; 3];
    for r in 0..3 {
        for k in 0..3 {
            m[r][k] = c[r][k] * i;
        }
        let mut t = m[r][0] * s[0][3];
        t = fmadds(m[r][1], s[1][3], t);
        m[r][3] = -fmadds(m[r][2], s[2][3], t);
    }
    Some(m)
}

/// `PSMTXTrans`.
pub fn trans(x: f32, y: f32, z: f32) -> Mtx {
    [[1.0, 0.0, 0.0, x], [0.0, 1.0, 0.0, y], [0.0, 0.0, 1.0, z]]
}

/// `PSMTXScale`.
pub fn scale(x: f32, y: f32, z: f32) -> Mtx {
    [[x, 0.0, 0.0, 0.0], [0.0, y, 0.0, 0.0], [0.0, 0.0, z, 0.0]]
}

/// `PSMTXRotAxisRad`.
pub fn rot_axis_rad(axis: &Vec3, rad: f32) -> Mtx {
    let s = sinf(rad);
    let c = cosf(rad);
    let t = 1.0f32 - c;
    let v = vec_normalize(axis);
    let (x, y, z) = (v.x, v.y, v.z);
    [
        [fmadds(t, x * x, c), fmadds(y, t * x, -(s * z)), fmadds(z, t * x, s * y), 0.0],
        [fmadds(y, t * x, s * z), fmadds(t, y * y, c), fmadds(z, t * y, -(s * x)), 0.0],
        [fmadds(z, t * x, -(s * y)), fmadds(z, t * y, s * x), fmadds(t, z * z, c), 0.0],
    ]
}

/// `MTXRotTrig` body. An unknown axis leaves `m` unchanged, as the retail routine does.
pub fn rot_trig(m: &mut Mtx, axis: u8, sin_a: f32, cos_a: f32) {
    match axis {
        b'x' | b'X' => {
            *m = [[1.0, 0.0, 0.0, 0.0], [0.0, cos_a, -sin_a, 0.0], [0.0, sin_a, cos_a, 0.0]];
        }
        b'y' | b'Y' => {
            *m = [[cos_a, 0.0, sin_a, 0.0], [0.0, 1.0, 0.0, 0.0], [-sin_a, 0.0, cos_a, 0.0]];
        }
        b'z' | b'Z' => {
            *m = [[cos_a, -sin_a, 0.0, 0.0], [sin_a, cos_a, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
        }
        _ => {}
    }
}

/// `MTXRotRad`.
pub fn rot_rad(m: &mut Mtx, axis: u8, rad: f32) {
    let s = sinf(rad);
    let c = cosf(rad);
    rot_trig(m, axis, s, c);
}

/// `PSMTXQuat`.
pub fn quat(q: &Quaternion) -> Mtx {
    // 2.0f / ((w*w) + ((z*z) + ((x*x) + (y*y))))
    let s = 2.0f32 / fmadds(q.w, q.w, fmadds(q.z, q.z, fmadds(q.x, q.x, q.y * q.y)));
    let xs = q.x * s;
    let ys = q.y * s;
    let zs = q.z * s;
    let wx = q.w * xs;
    let wy = q.w * ys;
    let wz = q.w * zs;
    let xx = q.x * xs;
    let xy = q.x * ys;
    let xz = q.x * zs;
    let yy = q.y * ys;
    let yz = q.y * zs;
    let zz = q.z * zs;
    [
        [1.0 - (yy + zz), xy - wz, xz + wy, 0.0],
        [xy + wz, 1.0 - (xx + zz), yz - wx, 0.0],
        [xz - wy, yz + wx, 1.0 - (xx + yy), 0.0],
    ]
}

/// `PSMTXMultVec`: x' = fma(m02, z, m00 x) + (m03 + m01 y), per row. Takes the first 3 rows of a
/// `Mtx` or `Mtx44` alike.
pub fn mult_vec(m: &[[f32; 4]], v: &Vec3) -> Vec3 {
    let (x, y, z) = (v.x, v.y, v.z);
    let row = |r: usize| {
        let p0 = m[r][0] * x;
        let p1 = m[r][1] * y;
        let s0 = fmadds(m[r][2], z, p0);
        let s1 = m[r][3] + p1;
        s0 + s1
    };
    Vec3 { x: row(0), y: row(1), z: row(2) }
}

/// `PSMTXMultVecSR`: x' = fma(m02, z, m00 x + m01 y).
pub fn mult_vec_sr(m: &[[f32; 4]], v: &Vec3) -> Vec3 {
    let (x, y, z) = (v.x, v.y, v.z);
    let row = |r: usize| {
        let s = m[r][0] * x + m[r][1] * y;
        fmadds(m[r][2], z, s)
    };
    Vec3 { x: row(0), y: row(1), z: row(2) }
}

pub fn vec_add(a: &Vec3, b: &Vec3) -> Vec3 {
    Vec3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z }
}

pub fn vec_sub(a: &Vec3, b: &Vec3) -> Vec3 {
    Vec3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}

pub fn vec_scale(v: &Vec3, s: f32) -> Vec3 {
    Vec3 { x: v.x * s, y: v.y * s, z: v.z * s }
}

/// `PSVECNormalize`: one Newton step on the hardware estimate; NOT correctly rounded, and a
/// zero vector yields non-finite output exactly as on the console.
pub fn vec_normalize(v: &Vec3) -> Vec3 {
    let (x, y, z) = (v.x, v.y, v.z);
    let xx = x * x;
    let yy = y * y;
    // Both double expressions are fma in the C build; the float*float products are exact in
    // double, so the plain double expression gives the same bits.
    let xz = (z as f64 * z as f64 + xx as f64) as f32;
    let sqsum = xz + yy;
    let est = frsqrte(sqsum as f64);
    let n0 = (est * force25(est)) as f32;
    let n1 = (est * 0.5) as f32;
    let n2 = (-(n0 as f64 * sqsum as f64 - 3.0)) as f32;
    let rsqrt = n2 * n1;
    Vec3 { x: x * rsqrt, y: y * rsqrt, z: z * rsqrt }
}

/// `PSVECMag`, the retail paired-single code: sqsum = (z*z + x*x) + y*y (ps_madd, ps_sum0), then
/// sqsum * one Newton step of frsqrte (fsel keeps 0 for a zero vector). Not correctly rounded.
pub fn vec_mag(v: &Vec3) -> f32 {
    let xx = v.x * v.x;
    let yy = v.y * v.y;
    let xz = (v.z as f64 * v.z as f64 + xx as f64) as f32;
    let sqsum = xz + yy;
    let est = frsqrte(sqsum as f64);
    let n0 = (est * force25(est)) as f32;
    let n1 = (est * 0.5) as f32;
    let n2 = (-(n0 as f64 * sqsum as f64 - 3.0)) as f32;
    let mut r = n2 * n1;
    if !(r >= 0.0) {
        r = sqsum;
    }
    sqsum * r
}

/// `PSVECDotProduct`, the retail paired-single code: fma(x1, x2, y1*y2) (ps_madd) + z1*z2 (ps_sum0).
pub fn vec_dot(a: &Vec3, b: &Vec3) -> f32 {
    fmadds(a.x, b.x, a.y * b.y) + a.z * b.z
}

/// `PSVECCrossProduct`, the retail paired-single code (ps_msub, then ps_neg for y and z): which
/// product is rounded on its own differs from the C reference in y and z.
pub fn vec_cross(a: &Vec3, b: &Vec3) -> Vec3 {
    // a*c - p with the product p rounded first, written as a subtraction (as the C's mtx_msub) so a
    // NaN p keeps its sign the way the C build's subsd does.
    let msub = |a: f32, c: f32, p: f32| (a as f64 * c as f64 - p as f64) as f32;
    Vec3 {
        x: msub(a.y, b.z, b.y * a.z),
        y: -msub(a.x, b.z, b.x * a.z),
        z: -msub(a.y, b.x, b.y * a.x),
    }
}

/// `C_MTXLookAt`.
pub fn look_at(cam_pos: &Vec3, cam_up: &Vec3, target: &Vec3) -> Mtx {
    let look = vec_normalize(&vec_sub(cam_pos, target));
    let right = vec_normalize(&vec_cross(cam_up, &look));
    let up = vec_cross(&look, &right);
    // The Dolphin SDK's C is built with MWCC -fp_contract off (configure.py DolphinLib): no fusion.
    let d = |v: &Vec3| -((cam_pos.z * v.z) + ((cam_pos.x * v.x) + (cam_pos.y * v.y)));
    [
        [right.x, right.y, right.z, d(&right)],
        [up.x, up.y, up.z, d(&up)],
        [look.x, look.y, look.z, d(&look)],
    ]
}

/// `MTXLightFrustum`.
#[allow(clippy::too_many_arguments)]
pub fn light_frustum(
    t: f32, b: f32, l: f32, r: f32, n: f32, scale_s: f32, scale_t: f32, trans_s: f32, trans_t: f32,
) -> Mtx {
    let mut m = [[0.0f32; 4]; 3];
    let tmp = 1.0f32 / (r - l);
    m[0][0] = scale_s * (2.0 * n * tmp);
    m[0][2] = fmadds(scale_s, tmp * (r + l), -trans_s);
    let tmp = 1.0f32 / (t - b);
    m[1][1] = scale_t * (2.0 * n * tmp);
    m[1][2] = fmadds(scale_t, tmp * (t + b), -trans_t);
    m[2][2] = -1.0;
    m
}

/// `MTXLightPerspective`.
pub fn light_perspective(
    fov_y: f32, aspect: f32, scale_s: f32, scale_t: f32, trans_s: f32, trans_t: f32,
) -> Mtx {
    let mut m = [[0.0f32; 4]; 3];
    let angle = (0.5f32 * fov_y) * 0.017453293f32;
    let cot = 1.0f32 / tanf(angle);
    m[0][0] = scale_s * (cot / aspect);
    m[0][2] = -trans_s;
    m[1][1] = cot * scale_t;
    m[1][2] = -trans_t;
    m[2][2] = -1.0;
    m
}

/// `MTXLightOrtho`.
#[allow(clippy::too_many_arguments)]
pub fn light_ortho(
    t: f32, b: f32, l: f32, r: f32, scale_s: f32, scale_t: f32, trans_s: f32, trans_t: f32,
) -> Mtx {
    let mut m = [[0.0f32; 4]; 3];
    let tmp = 1.0f32 / (r - l);
    m[0][0] = 2.0 * tmp * scale_s;
    m[0][3] = fmadds(scale_s, tmp * -(r + l), trans_s);
    let tmp = 1.0f32 / (t - b);
    m[1][1] = 2.0 * tmp * scale_t;
    m[1][3] = fmadds(scale_t, tmp * -(t + b), trans_t);
    m[2][3] = 1.0;
    m
}

/// `MTXFrustum` (mtx44.c).
pub fn frustum(t: f32, b: f32, l: f32, r: f32, n: f32, f: f32) -> Mtx44 {
    let mut m = [[0.0f32; 4]; 4];
    let tmp = 1.0f32 / (r - l);
    m[0][0] = 2.0 * n * tmp;
    m[0][2] = tmp * (r + l);
    let tmp = 1.0f32 / (t - b);
    m[1][1] = 2.0 * n * tmp;
    m[1][2] = tmp * (t + b);
    let tmp = 1.0f32 / (f - n);
    m[2][2] = -n * tmp;
    m[2][3] = tmp * -(f * n);
    m[3][2] = -1.0;
    m
}

/// `MTXPerspective` (mtx44.c).
pub fn perspective(fov_y: f32, aspect: f32, n: f32, f: f32) -> Mtx44 {
    let mut m = [[0.0f32; 4]; 4];
    let angle = (0.5f32 * fov_y) * 0.017453293f32;
    let cot = 1.0f32 / tanf(angle);
    m[0][0] = cot / aspect;
    m[1][1] = cot;
    let tmp = 1.0f32 / (f - n);
    m[2][2] = -n * tmp;
    m[2][3] = tmp * -(f * n);
    m[3][2] = -1.0;
    m
}

/// `MTXOrtho` (mtx44.c).
pub fn ortho(t: f32, b: f32, l: f32, r: f32, n: f32, f: f32) -> Mtx44 {
    let mut m = [[0.0f32; 4]; 4];
    let tmp = 1.0f32 / (r - l);
    m[0][0] = 2.0 * tmp;
    m[0][3] = tmp * -(r + l);
    let tmp = 1.0f32 / (t - b);
    m[1][1] = 2.0 * tmp;
    m[1][3] = tmp * -(t + b);
    let tmp = 1.0f32 / (f - n);
    m[2][2] = -1.0 * tmp;
    m[2][3] = -f * tmp;
    m[3][3] = 1.0;
    m
}
