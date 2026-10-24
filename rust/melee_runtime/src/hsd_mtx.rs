//! HSD matrix helpers. Port of `src/sysdolphin/baselib/mtx.c` (+ `fabsf_bitwise` from mtx.h).
//!
//! Fused sites follow the shipping build's IR exactly (`tools/rust-port/fma_sites.py
//! src/sysdolphin/baselib/mtx.c`): every `a*b +/- c` inside one expression is [`fmadds`].
//! Checked bit for bit against the C by `tests/oracle_hsd.rs`.
//!
//! Not ported here: `HSD_VecAlloc`/`HSD_VecFree`/`HSD_MtxAlloc`/`HSD_MtxFree` and the
//! `*AllocData` accessors are provided by `tu::mtx` over the native object pools.

use crate::gekko::{cosf, sinf, sqrtf};
use crate::lbtrigf::{asinf, atan2f};
use crate::mtx::{self, Mtx, Quaternion, Vec3};
use melee_core::fp::fmadds;

const EPSILON: f32 = 0.000_000_000_1;
const FLOAT_MIN: f32 = 1.175_494_3e-38;
const M_PI: f64 = std::f64::consts::PI;

/// `fabsf_bitwise`: clear the sign bit (NaN payloads kept).
#[inline]
pub fn fabsf_bitwise(v: f32) -> f32 {
    f32::from_bits(v.to_bits() & !0x8000_0000)
}

/// `HSD_CalcDeterminantMatrix3x4` (static inline in mtx.c; 5 fused sites).
pub fn determinant3x4(m: &Mtx) -> f32 {
    let mut d = fmadds(m[0][0] * m[1][1], m[2][2], m[0][1] * m[1][2] * m[2][0]);
    d = fmadds(m[0][2] * m[1][0], m[2][1], d);
    d = fmadds(-(m[2][0] * m[1][1]), m[0][2], d);
    d = fmadds(-(m[1][0] * m[0][1]), m[2][2], d);
    fmadds(-(m[0][0] * m[2][1]), m[1][2], d)
}

/// `a*b - c*d` as the C build computes it: `fmadds(a, b, -(c*d))`.
#[inline(always)]
fn d2(a: f32, b: f32, c: f32, d: f32) -> f32 {
    fmadds(a, b, -(c * d))
}

/// The 3x3 cofactor block shared by HSD_MtxInverse / InverseConcat / InverseTranspose,
/// in HSD_MtxInverse's `dest[r][c]` order (already multiplied by `det`).
fn cofactors(m: &Mtx, det: f32) -> [[f32; 3]; 3] {
    [
        [
            d2(m[1][1], m[2][2], m[2][1], m[1][2]) * det,
            -d2(m[0][1], m[2][2], m[2][1], m[0][2]) * det,
            d2(m[0][1], m[1][2], m[1][1], m[0][2]) * det,
        ],
        [
            -d2(m[1][0], m[2][2], m[2][0], m[1][2]) * det,
            d2(m[0][0], m[2][2], m[2][0], m[0][2]) * det,
            -d2(m[0][0], m[1][2], m[1][0], m[0][2]) * det,
        ],
        [
            d2(m[1][0], m[2][1], m[2][0], m[1][1]) * det,
            -d2(m[0][0], m[2][1], m[2][0], m[0][1]) * det,
            d2(m[0][0], m[1][1], m[1][0], m[0][1]) * det,
        ],
    ]
}

/// `-(c2*t2 - (-c0*t0 - c1*t1))`: the translation row of the inverse.
#[inline(always)]
fn inv_t(c0: f32, c1: f32, c2: f32, t0: f32, t1: f32, t2: f32) -> f32 {
    -fmadds(c2, t2, -fmadds(-c0, t0, -(c1 * t1)))
}

/// `HSD_MtxInverse(src, dest)` with `src != dest`. A near-singular input gives identity.
pub fn inverse(src: &Mtx) -> Mtx {
    let det = determinant3x4(src);
    if fabsf_bitwise(det) < EPSILON {
        return mtx::identity();
    }
    let c = cofactors(src, 1.0 / det);
    let mut d = [[0.0f32; 4]; 3];
    for r in 0..3 {
        d[r][..3].copy_from_slice(&c[r]);
    }
    for r in 0..3 {
        d[r][3] = inv_t(c[r][0], c[r][1], c[r][2], src[0][3], src[1][3], src[2][3]);
    }
    d
}

/// `HSD_MtxInverse(m, m)`: the aliased call. The C reads `src[r][3]` after it has already
/// written `dest[0..r][3]`, so later rows see the new translation values; reproduced here.
pub fn inverse_in_place(m: &mut Mtx) {
    let det = determinant3x4(m);
    if fabsf_bitwise(det) < EPSILON {
        *m = mtx::identity();
        return;
    }
    let c = cofactors(m, 1.0 / det);
    for r in 0..3 {
        m[r][..3].copy_from_slice(&c[r]);
    }
    for r in 0..3 {
        m[r][3] = inv_t(c[r][0], c[r][1], c[r][2], m[0][3], m[1][3], m[2][3]);
    }
}

/// `HSD_MtxInverseConcat(inv, src, dest)`: `inverse(inv) * src` (any aliasing; the C uses a
/// temporary when aliased, so the result is the same). Near-singular `inv` gives `src`.
pub fn inverse_concat(inv: &Mtx, src: &Mtx) -> Mtx {
    let det = determinant3x4(inv);
    if fabsf_bitwise(det) < EPSILON {
        return *src;
    }
    let c = cofactors(inv, 1.0 / det);
    // temp1..temp9 in the C: row r = (c[r][0], c[r][1], c[r][2]).
    let t: [f32; 3] =
        core::array::from_fn(|r| inv_t(c[r][0], c[r][1], c[r][2], inv[0][3], inv[1][3], inv[2][3]));
    let mut d = [[0.0f32; 4]; 3];
    for r in 0..3 {
        for k in 0..4 {
            let v = fmadds(c[r][2], src[2][k], fmadds(c[r][0], src[0][k], c[r][1] * src[1][k]));
            d[r][k] = if k == 3 { v + t[r] } else { v };
        }
    }
    d
}

/// `HSD_MtxInverseTranspose(src, dest)`. Near-singular input is returned unchanged.
pub fn inverse_transpose(src: &Mtx) -> Mtx {
    let det = determinant3x4(src);
    if fabsf_bitwise(det) < EPSILON {
        return *src;
    }
    let c = cofactors(src, 1.0 / det);
    let mut d = [[0.0f32; 4]; 3];
    for r in 0..3 {
        for k in 0..3 {
            d[k][r] = c[r][k];
        }
    }
    d
}

/// `calcVal` (static inline in mtx.c).
pub fn calc_val(x: f32, y: f32) -> f32 {
    if fabsf_bitwise(x) <= FLOAT_MIN {
        if y >= 0.0 {
            (M_PI / 2.0) as f32
        } else {
            (-M_PI / 2.0) as f32
        }
    } else {
        atan2f(y, x)
    }
}

#[inline(always)]
fn col_len(m: &Mtx, c: usize) -> f32 {
    sqrtf(fmadds(m[2][c], m[2][c], fmadds(m[0][c], m[0][c], m[1][c] * m[1][c])))
}

/// `HSD_MtxGetRotation`: Euler angles (x, y, z) of `m`.
pub fn get_rotation(m: &Mtx) -> Vec3 {
    let l0 = col_len(m, 0);
    if !(l0 < FLOAT_MIN) {
        let l1 = col_len(m, 1);
        if !(l1 < FLOAT_MIN) {
            let l2 = col_len(m, 2);
            if !(l2 < FLOAT_MIN) {
                let t1 = -m[2][0] / l0;
                let y = if t1 >= 1.0 {
                    (M_PI / 2.0) as f32
                } else if t1 <= -1.0 {
                    (-M_PI / 2.0) as f32
                } else {
                    asinf(t1)
                };
                if cosf(y) >= FLOAT_MIN {
                    let t2 = m[2][2] / l2;
                    let t3 = m[2][1] / l1;
                    return Vec3 { x: calc_val(t2, t3), y, z: calc_val(m[0][0], m[1][0]) };
                }
                return Vec3 { x: calc_val(m[1][1], m[0][1]), y, z: 0.0 };
            }
        }
    }
    Vec3 { x: 0.0, y: 0.0, z: 0.0 }
}

/// `HSD_MtxGetTranslate`.
pub fn get_translate(m: &Mtx) -> Vec3 {
    Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] }
}

/// `HSD_MtxGetScale`: Gram-Schmidt column lengths, negated when the basis is left-handed.
pub fn get_scale(m: &Mtx) -> Vec3 {
    let mut v1 = Vec3 { x: m[0][0], y: m[1][0], z: m[2][0] };
    let mut out = Vec3 { x: mtx::vec_mag(&v1), y: 0.0, z: 0.0 };
    v1 = mtx::vec_normalize(&v1);
    let mut v2 = Vec3 { x: m[0][1], y: m[1][1], z: m[2][1] };
    let v4 = mtx::vec_scale(&v1, mtx::vec_dot(&v1, &v2));
    v2 = mtx::vec_sub(&v2, &v4);
    out.y = mtx::vec_mag(&v2);
    v2 = mtx::vec_normalize(&v2);
    let mut v3 = Vec3 { x: m[0][2], y: m[1][2], z: m[2][2] };
    let v4 = mtx::vec_scale(&v2, mtx::vec_dot(&v2, &v3));
    v3 = mtx::vec_sub(&v3, &v4);
    let v4 = mtx::vec_scale(&v1, mtx::vec_dot(&v1, &v3));
    v3 = mtx::vec_sub(&v3, &v4);
    out.z = mtx::vec_mag(&v3);
    v3 = mtx::vec_normalize(&v3);
    let v4 = mtx::vec_cross(&v2, &v3);
    if (mtx::vec_dot(&v1, &v4) as f64) < 0.0 {
        out.x = (out.x as f64 * -1.0) as f32;
        out.y = (out.y as f64 * -1.0) as f32;
        out.z = (out.z as f64 * -1.0) as f32;
    }
    out
}

/// `HSD_MkRotationMtx`: rotation matrix from Euler angles (translation zero).
pub fn mk_rotation_mtx(r: &Vec3) -> Mtx {
    let (sx, cx) = (sinf(r.x), cosf(r.x));
    let (sy, cy) = (sinf(r.y), cosf(r.y));
    let (sz, cz) = (sinf(r.z), cosf(r.z));
    let t1 = sx * sy;
    let t2 = cx * sy;
    [
        [cy * cz, d2(cz, t1, cx, sz), fmadds(cz, t2, sx * sz), 0.0],
        [cy * sz, fmadds(sz, t1, cx * cz), d2(sz, t2, sx, cz), 0.0],
        [-sy, sx * cy, cx * cy, 0.0],
    ]
}

/// `HSD_MtxQuat` (= `PSMTXQuat`).
pub fn mtx_quat(q: &Quaternion) -> Mtx {
    mtx::quat(q)
}

/// `HSD_MtxSRT(m, scale, rotate, translate, parent_scale)`.
pub fn srt(s: &Vec3, r: &Vec3, t: &Vec3, ps: Option<&Vec3>) -> Mtx {
    let (sx, cx) = (sinf(r.x), cosf(r.x));
    let (sy, cy) = (sinf(r.y), cosf(r.y));
    let (sz, cz) = (sinf(r.z), cosf(r.z));
    let (x2, mut x1, mut x0) = (s.x, s.x, s.x);
    let (mut y2, y1, mut y0) = (s.y, s.y, s.y);
    let (mut z2, mut z1, z0) = (s.z, s.z, s.z);
    if let Some(p) = ps {
        let t1 = (1.0 / p.x as f64) as f32;
        let t2 = (1.0 / p.y as f64) as f32;
        let t3 = (1.0 / p.z as f64) as f32;
        y2 *= p.y * t1;
        z2 *= p.z * t1;
        x1 *= p.x * t2;
        z1 *= p.z * t2;
        x0 *= p.x * t3;
        y0 *= p.y * t3;
    }
    [
        [cz * (x2 * cy), y2 * d2(cz, sx * sy, cx, sz), z2 * fmadds(cz, cx * sy, sx * sz), t.x],
        [sz * (x1 * cy), y1 * fmadds(sz, sx * sy, cx * cz), z1 * d2(sz, cx * sy, sx, cz), t.y],
        [-x0 * sy, cy * (y0 * sx), cy * (z0 * cx), t.z],
    ]
}

/// `HSD_MtxSRTQuat(m, scale, rotate, translate, parent_scale)`.
pub fn srt_quat(s: &Vec3, q: &Quaternion, t: &Vec3, ps: Option<&Vec3>) -> Mtx {
    let mut m = mtx::scale(s.x, s.y, s.z);
    if let Some(p) = ps {
        m = mtx::concat(&mtx::scale(p.x, p.y, p.z), &m);
    }
    m = mtx::concat(&mtx::quat(q), &m);
    if let Some(p) = ps {
        let tmp = mtx::scale(
            (1.0 / p.x as f64) as f32,
            (1.0 / p.y as f64) as f32,
            (1.0 / p.z as f64) as f32,
        );
        m = mtx::concat(&tmp, &m);
    }
    mtx::concat(&mtx::trans(t.x, t.y, t.z), &m)
}

/// `HSD_MtxScaledAdd(a, b, dest, s)`: `dest = b + s*a`, element-wise (fused).
pub fn scaled_add(a: &Mtx, b: &Mtx, s: f32) -> Mtx {
    core::array::from_fn(|r| core::array::from_fn(|c| fmadds(s, a[r][c], b[r][c])))
}
