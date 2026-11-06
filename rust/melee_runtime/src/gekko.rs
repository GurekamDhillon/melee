
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

use melee_core::fp::Fmadd as _;
use melee_core::fp::fmadds;

pub use melee_core::fp::{fres, frsqrte};

pub(crate) static SINCOS_ON_QUADRANT: [f32; 8] = [0.0, 1.0, 1.0, 0.0, 0.0, -1.0, -1.0, 0.0];

/// Double literals in a float array in C: decimal -> f64 -> f32, reproduced with `as f32`.
#[rustfmt::skip]
pub(crate) static SINCOS_POLY: [f32; 10] = [
    0.0000035287617_f64 as f32, 0.0000003089747_f64 as f32, -0.0003259365_f64 as f32,
    -0.00003657235_f64 as f32, 0.015854323_f64 as f32, 0.0024903931_f64 as f32,
    -0.30842513_f64 as f32, -0.08074551_f64 as f32, 1.0, 0.7853982_f64 as f32,
];

pub(crate) static FOUR_OVER_PI_M1: [f32; 4] = [0.25, 0.0232393741608, 1.70555722434e-7, 1.86736494323e-11];

pub(crate) const SINCOS_EPSILON: f32 = 3.45266983e-4;

pub fn sincos_reduce(x: f32) -> (f32, i32) {
    // `(2.0f / 3.14159265358979323846f) * x`: a float constant (folded in f32), f32 multiply.
    let z = (2.0f32 / 3.14159265358979323846f32) * x;
    // `(int)`: fctiwz (melee_core::ppc), as gwtool lowers it in the C build.
    let n = if x.to_bits() & 0x8000_0000 != 0 { melee_core::ppc::fctiwz_f32(z - 0.5) } else { melee_core::ppc::fctiwz_f32(z + 0.5) };
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
pub(crate) fn fabs_lt(y: f32) -> bool {
    (if y < 0.0 { -y } else { y }) < SINCOS_EPSILON
}

// ---- src/MSL/math_ppc.h ----------------------------------------------------------------------

/// One Newton-Raphson step of math_ppc.h: `0.5 * guess * (3.0 - guess * guess * x)`.
/// The C build contracts `3.0 - (guess*guess)*x` into ONE double fma (verified in the IR:
/// `fmuladd.f64(-(g*g), x, 3.0)`), which differs from the plain expression.
#[inline(always)]
pub(crate) fn nr_step(guess: f64, x: f64) -> f64 {
    (0.5 * guess) * (-(guess * guess)).fmadd(x, 3.0)
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
pub(crate) mod tests {
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

pub use crate::tu::gekko_fp::cosf;
pub use crate::tu::gekko_fp::gk_fmadds;
pub use crate::tu::gekko_fp::sinf;
pub use crate::tu::gekko_fp::tanf_fp as tanf;
