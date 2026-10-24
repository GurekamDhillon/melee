//! HSD splines. Port of `src/sysdolphin/baselib/spline.c` (all 11 functions incl. statics).
//!
//! Fused sites as in the shipping build's IR (`fma_sites.py src/sysdolphin/baselib/spline.c`).
//! Checked bit for bit against the C by `tests/oracle_hsd.rs`.
//!
//! [`Spline`] is a borrowed view of an `HSD_Spline` (melee_types record, GC size 0x18): the
//! raw struct's pointers become slices. `cv` must hold every control point the C reads
//! (`numcv` for lerp/cardinal+1, `3*(numcv-1)+4` for bezier, `numcv+2` for B-spline).

use crate::cabi::f2i16;
use crate::gekko::sqrtf;
use crate::mtx::Vec3;
use melee_core::fp::fmadds;

/// Borrowed native view of `HSD_Spline`.
#[derive(Clone, Copy, Debug)]
pub struct Spline<'a> {
    /// 0 lerp, 1 bezier, 2 B-spline, 3 cardinal.
    pub ty: u8,
    pub numcv: i16,
    pub tension: f32,
    pub cv: &'a [Vec3],
    pub total_length: f32,
    pub seg_length: &'a [f32],
    pub seg_poly: &'a [[f32; 5]],
}

/// `splGetHelmite(fterm, time, p0, p1, d0, d1)`: Hermite basis evaluation.
pub fn get_helmite(fterm: f32, time: f32, p0: f32, p1: f32, d0: f32, d1: f32) -> f32 {
    let one_t2 = time * time;
    let t2 = fterm * fterm;
    let t2_t = one_t2 * fterm;
    let t3_t2 = t2 * (one_t2 * time);
    let two_t3_t3 = 2.0 * t3_t2 * fterm;
    let three_t2_t2 = 3.0 * one_t2 * t2;
    let inner = fmadds(p0, 1.0 + (two_t3_t3 - three_t2_t2), p1 * (-two_t3_t3 + three_t2_t2));
    let mid = fmadds(d0, time + ((t3_t2 - t2_t) - t2_t), inner);
    fmadds(d1, t3_t2 - t2_t, mid)
}

#[inline(always)]
fn weigh(cp: &[Vec3], w: [f32; 4]) -> Vec3 {
    let c = |f: fn(&Vec3) -> f32| {
        let mut s = fmadds(f(&cp[0]), w[0], f(&cp[1]) * w[1]);
        s = fmadds(f(&cp[2]), w[2], s);
        fmadds(f(&cp[3]), w[3], s)
    };
    Vec3 { x: c(|v| v.x), y: c(|v| v.y), z: c(|v| v.z) }
}

/// `splGetCardinalPoint` (static).
pub fn cardinal_point(cp: &[Vec3], tension: f32, u: f32) -> Vec3 {
    let u2 = u * u;
    let u3 = u2 * u;
    let car0 = tension * (fmadds(2.0, u2, -u3) - u);
    let car1 = fmadds(2.0 - tension, u3, (tension - 3.0) * u2) + 1.0;
    let car2 = fmadds(tension, u, fmadds(tension - 2.0, u3, fmadds(-2.0, tension, 3.0) * u2));
    let car3 = tension * (u3 - u2);
    weigh(cp, [car0, car1, car2, car3])
}

/// `splGetBSplinePoint` (static).
pub fn bspline_point(cp: &[Vec3], u: f32) -> Vec3 {
    let u2 = u * u;
    let u3 = u2 * u;
    let u_1 = 1.0 - u;
    let k = 1.0f32 / 6.0;
    let b0 = k * u_1 * u_1 * u_1;
    let b1 = k * (4.0 + fmadds(3.0, u3, -(6.0 * u2)));
    let b2 = k * fmadds(3.0, -u3 + u2 + u, 1.0);
    let b3 = k * u3;
    weigh(cp, [b0, b1, b2, b3])
}

/// `splGetBezierPoint` (static).
pub fn bezier_point(cp: &[Vec3], u: f32) -> Vec3 {
    let u_1 = 1.0 - u;
    let u2 = u * u;
    let u_12 = u_1 * u_1;
    weigh(cp, [u_12 * u_1, 3.0 * u * u_12, 3.0 * u2 * u_1, u2 * u])
}

/// `splGetSplinePoint(p, spline, u)`. `None` where the C leaves `*p` untouched
/// (u outside [0, 1], or an unknown spline type).
pub fn get_spline_point(s: &Spline, u: f32) -> Option<Vec3> {
    if u < 0.0 || u > 1.0 {
        return None;
    }
    if u < 1.0 {
        let mut t = u * (s.numcv as i32 - 1) as f32;
        let idx = f2i16(t as f64) as isize;
        t -= idx as f32;
        let at = |i: isize| &s.cv[i as usize..];
        match s.ty {
            0 => {
                let cp = at(idx);
                Some(Vec3 {
                    x: fmadds(t, cp[1].x - cp[0].x, cp[0].x),
                    y: fmadds(t, cp[1].y - cp[0].y, cp[0].y),
                    z: fmadds(t, cp[1].z - cp[0].z, cp[0].z),
                })
            }
            1 => Some(bezier_point(at(idx * 3), t)),
            2 => Some(bspline_point(at(idx), t)),
            3 => Some(cardinal_point(at(idx), s.tension, t)),
            _ => None,
        }
    } else {
        // `idx` is s16 in the C: numcv - 1 truncated to 16 bits.
        let idx = (s.numcv as i32 - 1) as i16 as isize;
        match s.ty {
            0 => Some(s.cv[idx as usize]),
            1 => Some(s.cv[(idx * 3) as usize]),
            2 => Some(bspline_point(&s.cv[(idx - 1) as usize..], 1.0)),
            3 => Some(s.cv[(idx + 1) as usize]),
            _ => None,
        }
    }
}

/// `splArcLengthPolynomial` (static).
pub fn arc_length_polynomial(c: &[f32; 5], t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let mut r = fmadds(c[0], t4, c[1] * t3);
    r = fmadds(c[2], t2, r);
    r = fmadds(c[3], t, r) + c[4];
    if r < 0.0 && r > -0.001 {
        r = 0.0;
    }
    sqrtf(r)
}

/// `spl_IterateSimpsonsMiddle` (static inline).
pub fn simpsons_middle(c: &[f32; 5], dx: f32, mut t: f32) -> f32 {
    let mut acc = 0.0f32;
    for i in 2..=8 {
        let w = if i & 1 == 0 { 4.0 } else { 2.0 };
        acc = fmadds(w, arc_length_polynomial(c, t), acc);
        t += dx;
    }
    acc
}

/// `spl_GetCoeffs` (static inline): the selected segment's coefficient array.
pub fn get_coeffs<'a>(spl: &Spline<'a>, idx: i32) -> &'a [f32; 5] {
    &spl.seg_poly[idx as usize]
}

/// `spl_GetArcLengthDx` (static): Simpson's eight equal subintervals.
pub fn get_arc_length_dx(start: f32, midpoint: f32) -> f32 {
    (midpoint - start) / 8.0
}

/// `splArcLengthGetParameter(spl, s)`: spline parameter at normalised arc length `s`.
pub fn arc_length_get_parameter(spl: &Spline, arg1: f32) -> f32 {
    let mut start = 0.0f32;
    let mut end = 1.0f32;
    if arg1 <= 0.0 {
        return start;
    }
    if arg1 >= 1.0 {
        return end;
    }
    let mut idx: i32 = 0;
    while spl.seg_length[(idx + 1) as usize] < arg1 {
        idx += 1;
    }
    let sl = |i: i32| spl.seg_length[i as usize];
    // Uninitialised in the C for an unknown type; gwtool's build zero-initialises locals.
    let mut result = 0.0f32;
    match spl.ty {
        0 => result = (arg1 - sl(idx)) / (sl(idx + 1) - sl(idx)),
        1..=3 => {
            let mut rem = spl.total_length * (arg1 - sl(idx));
            let c = get_coeffs(spl, idx);
            loop {
                let d = start - end;
                let a = if d < 0.0 { -d } else { d };
                if !(a >= 0.00001) {
                    break;
                }
                result = (start + end) / 2.0;
                let dx = get_arc_length_dx(start, result);
                let middle = simpsons_middle(c, dx, start + dx);
                let simpsons = dx
                    * (middle + arc_length_polynomial(c, start) + arc_length_polynomial(c, result))
                    / 3.0;
                if rem < 0.00001 + simpsons {
                    end = result;
                } else {
                    start = result;
                    rem -= simpsons;
                }
            }
        }
        _ => {}
    }
    (result + idx as f32) / (spl.numcv as f32 - 1.0)
}

/// `splArcLengthPoint(p, spline, s)`.
pub fn arc_length_point(spl: &Spline, s: f32) -> Option<Vec3> {
    get_spline_point(spl, arc_length_get_parameter(spl, s))
}
