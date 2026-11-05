//! Gekko (GameCube CPU) floating-point helpers that differ from plain IEEE f32 arithmetic.
//!
//! Source of truth: `pc/gameworld/gekko_fp.c`. Its `__frsqrte`/`__fres` table estimates and the
//! sinf/cosf/tanf polynomials are ported in `melee_runtime` (group 1), not here.

/// Gekko `fmadds` as the C build models it (`gk_fmadds` in gekko_fp.c):
/// `(float)((double)a * (double)c + (double)b)`.
///
/// The f32 x f32 product is exact in f64; the sum rounds to f64 and then once more to f32.
/// This is NOT `a.mul_add(c, b)` (one rounding) and NOT `a * c + b` in f32 (two f32
/// roundings); both give different bits for some inputs.
#[inline(always)]
pub fn fmadds(a: f32, c: f32, b: f32) -> f32 {
    (a as f64 * c as f64 + b as f64) as f32
}

/// Gekko `fmsubs` under the same model: `(float)((double)a * (double)c - (double)b)`.
#[inline(always)]
pub fn fmsubs(a: f32, c: f32, b: f32) -> f32 {
    (a as f64 * c as f64 - b as f64) as f32
}

/// `fmsubs` as clang's `fmuladd(a, c, -b)` forms it (`a*c - b` where the C build fuses to a negated
/// addend): identical to [`fmsubs`] for every non-NaN input; a NaN `b` comes out with its sign flipped
/// (ft_0c31's oracle). Other TUs' oracles (itkinds10) pin the unflipped [`fmsubs`].
#[inline(always)]
pub fn fmsubs_neg(a: f32, c: f32, b: f32) -> f32 {
    (a as f64 * c as f64 + (-(b as f64))) as f32
}

/// Gekko `fnmsubs`: `-(a*c - b)`, the form MWCC emits for `b -= a*c` / `b - a*c`.
///
/// Same magnitude as `fmadds(-a, c, b)`, but the sign of an exact-zero result differs: `a*c == b`
/// gives -0 here and +0 there (round-to-nearest gives +0 for an exact cancellation, then the
/// negation flips it). The console keeps that -0 (Dolphin oracle, `x8c_kb_vel.y` in
/// Fighter_procUpdate; notes/fp-audit.md "sign of zero").
#[inline(always)]
pub fn fnmsubs(a: f32, c: f32, b: f32) -> f32 {
    -fmsubs(a, c, b)
}

/// Gekko `fnmadds`: `-(a*c + b)`, MWCC's form of `-(a*c) - b` / `-b - a*c`. An exact-zero `a*c + b`
/// gives -0, where `fmsubs(-a, c, b)` gives +0.
#[inline(always)]
pub fn fnmadds(a: f32, c: f32, b: f32) -> f32 {
    -fmadds(a, c, b)
}

// ---- Gekko estimate instructions (moved from melee_runtime::gekko; port of
// pc/gameworld/gekko_fp.c, table and algorithm from Dolphin's FloatUtils.cpp, GPL-2.0-or-later).
// Verified bit for bit against the C by melee_runtime/tests/oracle.rs and
// melee_chara's pure tests (220 inputs) and melee_world's mplib test.

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

/// Preserve a double's payload and sign while setting the Gekko quiet-NaN bit.
#[inline]
pub fn make_quiet(v: f64) -> f64 {
    f64::from_bits(v.to_bits() | QUIET_BIT)
}
/// Construct the signed infinity used by the Gekko estimate instructions.
#[inline]
pub fn infinity(negative: bool) -> f64 {
    f64::from_bits(if negative { SIGN_BIT | EXP_MASK as u64 } else { EXP_MASK as u64 })
}
/// Construct the canonical positive quiet NaN used by the Gekko estimates.
#[inline]
pub fn nan() -> f64 {
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

/// Dolphin's Force25Bit (frC operand of a single-precision multiply keeps 25 mantissa bits).
#[inline]
pub fn force25(d: f64) -> f64 {
    let u = d.to_bits();
    f64::from_bits((u & 0xFFFF_FFFF_F800_0000).wrapping_add(u & 0x800_0000))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmadds_is_double_rounded_not_fused() {
        // a*c = 1 + 2^-11 + 2^-24 exactly; + 2^-60 puts the exact sum just above an f32 tie.
        // Fused (one rounding) rounds up; the f64 step drops 2^-60, leaving an exact tie that
        // rounds to even (down). So mul_add is observably wrong here.
        let a = 1.0f32 + 2f32.powi(-12);
        let b = 2f32.powi(-60);
        assert_eq!(fmadds(a, a, b), 1.0 + 2f32.powi(-11));
        assert_eq!(a.mul_add(a, b), 1.0 + 2f32.powi(-11) + 2f32.powi(-23));
        assert_ne!(fmadds(a, a, b).to_bits(), a.mul_add(a, b).to_bits());
    }
}
