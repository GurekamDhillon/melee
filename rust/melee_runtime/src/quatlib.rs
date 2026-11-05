//! Quaternion helpers. Port of `src/sysdolphin/baselib/quatlib.c` (6/6 functions).
//!
//! Fused sites as in the shipping build's IR (`fma_sites.py src/sysdolphin/baselib/quatlib.c`).
//! Checked bit for bit against the C by `tests/oracle_hsd.rs`. All C versions return 0 except
//! `HSD_QuatLib_8037ECE0`'s degenerate-axis case (-1), which is `None` here.

use crate::gekko::{cosf, sinf, sqrtf};
use crate::lbtrigf::{acosf, atan2f};
use crate::mtx::{Mtx, Quaternion, Vec3};
use melee_core::fp::fmadds;

const M_PI_2: f64 = std::f64::consts::FRAC_PI_2;

#[inline(always)]
fn col_len(m: &Mtx, c: usize) -> f32 {
    // squares of matrix entries (memory): not fused on the console (notes/fp-audit.md)
    sqrtf(m[0][c] * m[0][c] + m[1][c] * m[1][c] + m[2][c] * m[2][c])
}

/// `MatToQuat`: rotation quaternion of `m` (columns normalised first).
pub fn mat_to_quat(m: &Mtx) -> Quaternion {
    let len = [col_len(m, 0), col_len(m, 1), col_len(m, 2)];
    let n = |r: usize, c: usize| m[r][c] / len[c];
    let s = n(0, 0) + n(1, 1) + n(2, 2);
    if s > 0.0 {
        let s = sqrtf(1.0 + s);
        let scale = 0.5 / s;
        Quaternion {
            w: 0.5 * s,
            x: scale * (n(2, 1) - n(1, 2)),
            y: scale * (n(0, 2) - n(2, 0)),
            z: scale * (n(1, 0) - n(0, 1)),
        }
    } else {
        const NXT: [usize; 3] = [1, 2, 0];
        let mut i = 0;
        if n(1, 1) > n(0, 0) {
            i = 1;
        }
        if n(2, 2) > n(i, i) {
            i = 2;
        }
        let j = NXT[i];
        let k = NXT[j];
        let s = sqrtf(1.0 + ((n(i, i) - n(j, j)) - n(k, k)));
        let scale = 0.5 / s;
        let mut q3 = [0.0f32; 3];
        q3[i] = 0.5 * s;
        let w = scale * (n(k, j) - n(j, k));
        q3[j] = scale * (n(j, i) + n(i, j));
        q3[k] = scale * (n(k, i) + n(i, k));
        Quaternion { x: q3[0], y: q3[1], z: q3[2], w }
    }
}

/// `HSD_QuatLib_8037EB28`: Euler angles from a rotation matrix.
pub fn mtx_to_euler(m: &Mtx) -> Vec3 {
    let len = sqrtf(m[0][0] * m[0][0] + m[1][0] * m[1][0]); // memory squares: not fused on the console
    if len as f64 > 1e-05 {
        Vec3 { x: atan2f(m[2][1], m[2][2]), y: atan2f(-m[2][0], len), z: atan2f(m[1][0], m[0][0]) }
    } else {
        Vec3 { x: atan2f(-m[1][2], m[1][1]), y: atan2f(-m[2][0], len), z: 0.0 }
    }
}

/// `HSD_QuatLib_8037EC4C(p, q, out)`: quaternion product.
pub fn mul(p: &Quaternion, q: &Quaternion) -> Quaternion {
    let x = fmadds(q.w, p.x, p.w * q.x) + fmadds(p.y, q.z, -(q.y * p.z));
    let y = fmadds(q.w, p.y, p.w * q.y) + fmadds(q.x, p.z, -(p.x * q.z));
    let z = fmadds(q.w, p.z, p.w * q.z) + fmadds(p.x, q.y, -(q.x * p.y));
    let w = fmadds(p.w, q.w, -fmadds(p.z, q.z, fmadds(p.x, q.x, p.y * q.y)));
    Quaternion { x, y, z, w }
}

/// `HSD_QuatLib_8037ECE0(axis, q, angle)`: axis-angle to quaternion; `None` (C: -1, `q`
/// untouched) when |axis| is below FLT_MIN.
pub fn from_axis_angle(axis: &Vec3, angle: f32) -> Option<Quaternion> {
    // axis->x * axis->x + ...: through a pointer, not fused on the console (notes/fp-audit.md)
    let len = sqrtf(axis.x * axis.x + axis.y * axis.y + axis.z * axis.z);
    if len.abs() < 1.175_494_4e-38 {
        return None;
    }
    let inv = 1.0 / len;
    let half = 0.5 * angle;
    let w = cosf(half);
    let s = sinf(half);
    Some(Quaternion { x: s * (inv * axis.x), y: s * (inv * axis.y), z: s * (inv * axis.z), w })
}

/// `EulerToQuat`.
pub fn euler_to_quat(e: &Vec3) -> Quaternion {
    let cx = cosf(0.5 * e.x);
    let cy = cosf(0.5 * e.y);
    let cz = cosf(0.5 * e.z);
    let sx = sinf(0.5 * e.x);
    let sy = sinf(0.5 * e.y);
    let sz = sinf(0.5 * e.z);
    let ss = sy * sz;
    let cc = cy * cz;
    Quaternion {
        w: fmadds(cx, cc, sx * ss),
        x: fmadds(sx, cc, -(cx * ss)),
        y: fmadds(cz, cx * sy, sz * (sx * cy)),
        z: fmadds(sz, cx * cy, -(cz * (sx * sy))),
    }
}

#[inline(always)]
fn blend(sp: f32, p: &Quaternion, sq: f32, q: &Quaternion) -> Quaternion {
    Quaternion {
        x: fmadds(sp, p.x, sq * q.x),
        y: fmadds(sp, p.y, sq * q.y),
        z: fmadds(sp, p.z, sq * q.z),
        w: fmadds(sp, p.w, sq * q.w),
    }
}

/// `HSD_QuatLib_8037EF28(p, q, out, t)`: slerp.
pub fn slerp(p: &Quaternion, q: &Quaternion, t: f32) -> Quaternion {
    let cosom = fmadds(p.w, q.w, fmadds(p.z, q.z, fmadds(p.x, q.x, p.y * q.y)));
    if (1.0 + cosom) > 1e-10 {
        let (sp, sq);
        if (1.0 - cosom) > 1e-10 {
            let theta = acosf(cosom);
            let sinom = sinf(theta);
            sp = sinf((1.0 - t) * theta) / sinom;
            sq = sinf(t * theta) / sinom;
        } else {
            sq = t;
            sp = (1.0 - t as f64) as f32;
        }
        blend(sp, p, sq, q)
    } else if t < 0.5 {
        let sp = sinf((M_PI_2 * fmadds(-2.0, t, 1.0) as f64) as f32);
        let sq = sinf((M_PI_2 * (2.0 * t) as f64) as f32);
        blend(sp, p, sq, q)
    } else {
        let t2 = 2.0 * (t - 0.5);
        let sp = sinf((M_PI_2 * (1.0 - t2) as f64) as f32);
        let sq = sinf((M_PI_2 * t2 as f64) as f32);
        blend(sp, p, sq, q)
    }
}
