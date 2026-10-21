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

#[derive(Clone, Copy)]
struct Entry {
    base: i32,
    dec: i32,
}

const fn e(base: i32, dec: i32) -> Entry {
    Entry { base, dec }
}

#[rustfmt::skip]
static FRSQRTE_EXPECTED: [Entry; 32] = [
    e(0x1a7e800, -0x568), e(0x17cb800, -0x4f3), e(0x1552800, -0x48d),
    e(0x130c000, -0x435), e(0x10f2000, -0x3e7), e(0x0eff000, -0x3a2),
    e(0x0d2e000, -0x365), e(0x0b7c000, -0x32e), e(0x09e5000, -0x2fc),
    e(0x0867000, -0x2d0), e(0x06ff000, -0x2a8), e(0x05ab800, -0x283),
    e(0x046a000, -0x261), e(0x0339800, -0x243), e(0x0218800, -0x226),
    e(0x0105800, -0x20b), e(0x3ffa000, -0x7a4), e(0x3c29000, -0x700),
    e(0x38aa000, -0x670), e(0x3572000, -0x5f2), e(0x3279000, -0x584),
    e(0x2fb7000, -0x524), e(0x2d26000, -0x4cc), e(0x2ac0000, -0x47e),
    e(0x2881000, -0x43a), e(0x2665000, -0x3fa), e(0x2468000, -0x3c2),
    e(0x2287000, -0x38e), e(0x20c1000, -0x35e), e(0x1f12000, -0x332),
    e(0x1d79000, -0x30a), e(0x1bf4000, -0x2e6),
];

#[rustfmt::skip]
static FRES_EXPECTED: [Entry; 32] = [
    e(0x7ff800, 0x3e1), e(0x783800, 0x3a7), e(0x70ea00, 0x371),
    e(0x6a0800, 0x340), e(0x638800, 0x313), e(0x5d6200, 0x2ea),
    e(0x579000, 0x2c4), e(0x520800, 0x2a0), e(0x4cc800, 0x27f),
    e(0x47ca00, 0x261), e(0x430800, 0x245), e(0x3e8000, 0x22a),
    e(0x3a2c00, 0x212), e(0x360800, 0x1fb), e(0x321400, 0x1e5),
    e(0x2e4a00, 0x1d1), e(0x2aa800, 0x1be), e(0x272c00, 0x1ac),
    e(0x23d600, 0x19b), e(0x209e00, 0x18b), e(0x1d8800, 0x17c),
    e(0x1a9000, 0x16e), e(0x17ae00, 0x15b), e(0x14f800, 0x15b),
    e(0x124400, 0x143), e(0x0fbe00, 0x143), e(0x0d3800, 0x12d),
    e(0x0ade00, 0x12d), e(0x088400, 0x11a), e(0x065000, 0x11a),
    e(0x041c00, 0x108), e(0x020c00, 0x106),
];

const EXP_MASK: i64 = 0x7FF << 52;
const MANTISSA_MASK: i64 = (1 << 52) - 1;
const SIGN_BIT: u64 = 1 << 63;
const QUIET_BIT: u64 = 1 << 51;

#[inline]
fn make_quiet(v: f64) -> f64 {
    f64::from_bits(v.to_bits() | QUIET_BIT)
}
#[inline]
fn infinity(negative: bool) -> f64 {
    f64::from_bits(if negative { SIGN_BIT | EXP_MASK as u64 } else { EXP_MASK as u64 })
}
#[inline]
fn nan() -> f64 {
    f64::from_bits(EXP_MASK as u64 | QUIET_BIT)
}

/// Gekko `frsqrte`: the hardware reciprocal-square-root estimate (not correctly rounded).
pub fn frsqrte(val: f64) -> f64 {
    let mut integral = val.to_bits() as i64;
    let mut mantissa = integral & MANTISSA_MASK;
    let sign = integral & SIGN_BIT as i64;
    let mut exponent = integral & EXP_MASK;

    if mantissa == 0 && exponent == 0 {
        return infinity(sign != 0);
    }
    if exponent == EXP_MASK {
        if mantissa == 0 {
            return if sign != 0 { nan() } else { 0.0 };
        }
        return make_quiet(val);
    }
    if sign != 0 {
        return nan();
    }
    if exponent == 0 {
        loop {
            exponent -= 1 << 52;
            mantissa <<= 1;
            if mantissa & (1 << 52) != 0 {
                break;
            }
        }
        mantissa &= MANTISSA_MASK;
        exponent += 1 << 52;
    }

    let exponent_lsb = exponent & (1 << 52);
    exponent = ((0x3FF << 52) - ((exponent - (0x3FE << 52)) / 2)) & EXP_MASK;
    integral = sign | exponent;

    let i = ((exponent_lsb | mantissa) >> 37) as i32;
    let index = (i / 2048) as usize;
    let ent = FRSQRTE_EXPECTED[index];
    integral |= (ent.base.wrapping_add(ent.dec.wrapping_mul(i % 2048)) as i64) << 26;
    f64::from_bits(integral as u64)
}

/// Gekko `fres`: the hardware reciprocal estimate.
pub fn fres(val: f64) -> f64 {
    let mut integral = val.to_bits() as i64;
    let mantissa = integral & MANTISSA_MASK;
    let sign = integral & SIGN_BIT as i64;
    let mut exponent = integral & EXP_MASK;

    if mantissa == 0 && exponent == 0 {
        return infinity(sign != 0);
    }
    if exponent == EXP_MASK {
        if mantissa == 0 {
            return f64::from_bits(sign as u64);
        }
        return make_quiet(val);
    }
    if exponent < (895 << 52) {
        return f64::from_bits(sign as u64 | 0x47EF_FFFF_E000_0000);
    }
    if exponent >= (1149 << 52) {
        return f64::from_bits(sign as u64);
    }

    exponent = (0x7FD << 52) - exponent;
    let i = (mantissa >> 37) as i32;
    let index = (i / 1024) as usize;
    let ent = FRES_EXPECTED[index];
    integral = sign | exponent;
    integral |= ((ent.base - (ent.dec * (i % 1024) + 1) / 2) as i64) << 29;
    f64::from_bits(integral as u64)
}

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

fn sincos_reduce(x: f32) -> (f32, i32) {
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

fn sincos_even(ysq: f32) -> f32 {
    let p = &SINCOS_POLY;
    let mut z = gk_fmadds(p[0], ysq, p[2]);
    z = gk_fmadds(z, ysq, p[4]);
    z = gk_fmadds(z, ysq, p[6]);
    gk_fmadds(z, ysq, p[8])
}

fn sincos_odd_head(ysq: f32) -> f32 {
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
