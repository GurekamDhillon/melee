//! Bit-exact Gekko estimate instructions and MSL trig. Port of `pc/gameworld/gekko_fp.c`.
//!
//! Every float expression mirrors what the C build computes, which is NOT what the C source
//! text says at face value: the real build compiles with clang's default `-ffp-contract=on`, so
//! a `a*b + c` inside one expression becomes `llvm.fmuladd`, and gwtool lowers that to
//! `(f32)((f64)a*(f64)b + (f64)c)` for floats ([`fmadds`]) and to a true `fma` for doubles
//! (see pc/tools/gwtool/gwtool.cpp `lowerFMulAdd`). Where a contraction happens in the C build it
//! is written out explicitly here. Checked against the C (contraction + gwtool lowering
//! reproduced) by `tests/oracle.rs`.
//!
//! Portions derived from Dolphin's FloatUtils.cpp (GPL-2.0-or-later), as the C file is.

use melee_core::fp::fmadds;

pub use melee_core::fp::{fres, frsqrte};

// ---- MSL sinf / cosf / tanf ----------------------------------------------------------------

/// gekko_fp.c `gk_fmadds`. In the C build its body `(double)a*(double)c + (double)b` is itself
/// contracted to a double `fma`; the float product is exact in double, so that equals
/// [`fmadds`] (f64 multiply, f64 add, then f32). Not `f32::mul_add`.
#[inline(always)]
pub fn gk_fmadds(a: f32, c: f32, b: f32) -> f32 {
    fmadds(a, c, b)
}

static SINCOS_ON_QUADRANT: [f32; 8] = [0.0, 1.0, 1.0, 0.0, 0.0, -1.0, -1.0, 0.0];
/// Double literals in a float array in C: decimal -> f64 -> f32, reproduced with `as f32`.
#[rustfmt::skip]
static SINCOS_POLY: [f32; 10] = [
    0.0000035287617_f64 as f32, 0.0000003089747_f64 as f32, -0.0003259365_f64 as f32,
    -0.00003657235_f64 as f32, 0.015854323_f64 as f32, 0.0024903931_f64 as f32,
    -0.30842513_f64 as f32, -0.08074551_f64 as f32, 1.0, 0.7853982_f64 as f32,
];
static FOUR_OVER_PI_M1: [f32; 4] = [0.25, 0.0232393741608, 1.70555722434e-7, 1.86736494323e-11];
const SINCOS_EPSILON: f32 = 3.45266983e-4;

pub fn sincos_reduce(x: f32) -> (f32, i32) {
    // `(2.0f / 3.14159265358979323846f) * x`: a float constant (folded in f32), f32 multiply.
    let z = (2.0f32 / 3.14159265358979323846f32) * x;
    // `(int)` saturates in the C build (gwtool: fptosi.sat, like fctiwz), as Rust `as` does.
    let n = if x.to_bits() & 0x8000_0000 != 0 { (z - 0.5) as i32 } else { (z + 0.5) as i32 };
    let mut y = x - n.wrapping_mul(2) as f32;
    y = gk_fmadds(FOUR_OVER_PI_M1[0], x, y);
    y = gk_fmadds(FOUR_OVER_PI_M1[1], x, y);
    y = gk_fmadds(FOUR_OVER_PI_M1[2], x, y);
    y = gk_fmadds(FOUR_OVER_PI_M1[3], x, y);
    (y, n & 3)
}

pub fn sincos_even(ysq: f32) -> f32 {
    let p = &SINCOS_POLY;
    let mut z = gk_fmadds(p[0], ysq, p[2]);
    z = gk_fmadds(z, ysq, p[4]);
    z = gk_fmadds(z, ysq, p[6]);
    gk_fmadds(z, ysq, p[8])
}

pub fn sincos_odd_head(ysq: f32) -> f32 {
    let p = &SINCOS_POLY;
    let mut z = gk_fmadds(p[1], ysq, p[3]);
    z = gk_fmadds(z, ysq, p[5]);
    gk_fmadds(z, ysq, p[7])
}

#[inline]
fn fabs_lt(y: f32) -> bool {
    (if y < 0.0 { -y } else { y }) < SINCOS_EPSILON
}

/// MSL `sinf` as the retail binary computes it.
pub fn sinf(x: f32) -> f32 {
    let (y, n) = sincos_reduce(x);
    let n = n as usize;
    if fabs_lt(y) {
        let n = n << 1;
        return gk_fmadds(SINCOS_POLY[9], SINCOS_ON_QUADRANT[n + 1] * y, SINCOS_ON_QUADRANT[n]);
    }
    let ysq = y * y;
    if n & 1 != 0 {
        return sincos_even(ysq) * SINCOS_ON_QUADRANT[n << 1];
    }
    let n = n << 1;
    (gk_fmadds(sincos_odd_head(ysq), ysq, SINCOS_POLY[9]) * y) * SINCOS_ON_QUADRANT[n + 1]
}

/// MSL `cosf` as the retail binary computes it.
pub fn cosf(x: f32) -> f32 {
    let (y, n) = sincos_reduce(x);
    let n = n as usize;
    if fabs_lt(y) {
        let n = n << 1;
        // fnmsubs, a double fma in the C build; the product of two floats is exact in double.
        return (SINCOS_ON_QUADRANT[n + 1] as f64 - y as f64 * SINCOS_ON_QUADRANT[n] as f64) as f32;
    }
    let ysq = y * y;
    if n & 1 != 0 {
        let n = n << 1;
        return (y * -gk_fmadds(sincos_odd_head(ysq), ysq, SINCOS_POLY[9])) * SINCOS_ON_QUADRANT[n];
    }
    sincos_even(ysq) * SINCOS_ON_QUADRANT[(n << 1) + 1]
}

/// MSL `tanf`.
pub fn tanf(x: f32) -> f32 {
    sinf(x) / cosf(x)
}

// ---- src/MSL/math_ppc.h ----------------------------------------------------------------------

/// One Newton-Raphson step of math_ppc.h: `0.5 * guess * (3.0 - guess * guess * x)`.
/// The C build contracts `3.0 - (guess*guess)*x` into ONE double fma (verified in the IR:
/// `fmuladd.f64(-(g*g), x, 3.0)`), which differs from the plain expression.
#[inline(always)]
fn nr_step(guess: f64, x: f64) -> f64 {
    (0.5 * guess) * (-(guess * guess)).mul_add(x, 3.0)
}

/// MSL `sqrtf` (math_ppc.h): three Newton-Raphson steps on [`frsqrte`].
pub fn sqrtf(x: f32) -> f32 {
    if x > 0.0 {
        let xd = x as f64;
        let mut guess = frsqrte(xd);
        guess = nr_step(guess, xd);
        guess = nr_step(guess, xd);
        guess = nr_step(guess, xd);
        return (xd * guess) as f32;
    }
    x
}

/// MSL `sqrtf_accurate` (math_ppc.h): four Newton-Raphson steps.
pub fn sqrtf_accurate(x: f32) -> f32 {
    if x > 0.0 {
        let xd = x as f64;
        let mut guess = frsqrte(xd);
        guess = nr_step(guess, xd);
        guess = nr_step(guess, xd);
        guess = nr_step(guess, xd);
        guess = nr_step(guess, xd);
        return (xd * guess) as f32;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frsqrte_specials() {
        assert_eq!(frsqrte(0.0), f64::INFINITY);
        assert_eq!(frsqrte(-0.0), f64::NEG_INFINITY);
        assert_eq!(frsqrte(f64::INFINITY), 0.0);
        assert!(frsqrte(-1.0).is_nan());
    }

    #[test]
    fn sqrt_is_not_std() {
        assert_eq!(sqrtf(4.0), 2.0);
        assert_eq!(sqrtf(-3.0), -3.0);
    }
}
