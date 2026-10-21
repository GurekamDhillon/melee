//! Fighter-independent helpers from the group-4 character TUs, ported by hand.
//!
//! Everything else in the character code reads or writes `Fighter` (group 3) and is not ported
//! yet; see docs/rust-port/group-4 fighter characters.md. Expected values for the tests come
//! from the C compiled natively (gen/ref_pure.c, gen/run_ref.sh -> src/pure_ref.rs).

/// Gekko `frsqrte` estimate, bit-exact (port of `__frsqrte` in pc/gameworld/gekko_fp.c,
/// itself from Dolphin's ApproximateReciprocalSquareRoot).
/// TODO(move-to-core): group 1 (melee_runtime) owns gekko_fp; replace with its version.
pub fn frsqrte(val: f64) -> f64 {
    const BASE_DEC: [(i64, i64); 32] = [
        (0x1a7e800, -0x568), (0x17cb800, -0x4f3), (0x1552800, -0x48d), (0x130c000, -0x435),
        (0x10f2000, -0x3e7), (0x0eff000, -0x3a2), (0x0d2e000, -0x365), (0x0b7c000, -0x32e),
        (0x09e5000, -0x2fc), (0x0867000, -0x2d0), (0x06ff000, -0x2a8), (0x05ab800, -0x283),
        (0x046a000, -0x261), (0x0339800, -0x243), (0x0218800, -0x226), (0x0105800, -0x20b),
        (0x3ffa000, -0x7a4), (0x3c29000, -0x700), (0x38aa000, -0x670), (0x3572000, -0x5f2),
        (0x3279000, -0x584), (0x2fb7000, -0x524), (0x2d26000, -0x4cc), (0x2ac0000, -0x47e),
        (0x2881000, -0x43a), (0x2665000, -0x3fa), (0x2468000, -0x3c2), (0x2287000, -0x38e),
        (0x20c1000, -0x35e), (0x1f12000, -0x332), (0x1d79000, -0x30a), (0x1bf4000, -0x2e6),
    ];
    const EXP_MASK: i64 = 0x7FF << 52;
    const MANT_MASK: i64 = (1 << 52) - 1;
    const SIGN: i64 = i64::MIN;
    const QUIET: u64 = 1 << 51;
    let nan = f64::from_bits(EXP_MASK as u64 | QUIET);

    let integral = val.to_bits() as i64;
    let mut mantissa = integral & MANT_MASK;
    let sign = integral & SIGN;
    let mut exponent = integral & EXP_MASK;

    if mantissa == 0 && exponent == 0 {
        return f64::from_bits(if sign != 0 { (SIGN | EXP_MASK) as u64 } else { EXP_MASK as u64 });
    }
    if exponent == EXP_MASK {
        if mantissa == 0 {
            return if sign != 0 { nan } else { 0.0 };
        }
        return f64::from_bits(val.to_bits() | QUIET);
    }
    if sign != 0 {
        return nan;
    }
    if exponent == 0 {
        loop {
            exponent -= 1 << 52;
            mantissa <<= 1;
            if mantissa & (1 << 52) != 0 {
                break;
            }
        }
        mantissa &= MANT_MASK;
        exponent += 1 << 52;
    }
    let exponent_lsb = exponent & (1 << 52);
    // C: signed 64-bit division truncating toward zero (Rust `/` on i64 does the same).
    exponent = ((0x3FF << 52) - ((exponent - (0x3FE << 52)) / 2)) & EXP_MASK;
    let mut out = sign | exponent;
    let i = ((exponent_lsb | mantissa) >> 37) as i32;
    let (base, dec) = BASE_DEC[(i / 2048) as usize];
    out |= (base + dec * (i % 2048) as i64) << 26;
    f64::from_bits(out as u64)
}

/// `my_sqrtf`, the static inline copied into ftCrazyHand (fingerbeam, rockcrush0),
/// ftMasterHand (backdisappear, fingergun, poke, walk) and ftSeak (specialhi): an frsqrte
/// estimate plus three f64 Newton-Raphson steps, then `(float)(x * guess)`.
/// The operation order is the C's; f64 throughout, one final rounding to f32.
pub fn my_sqrtf(x: f32) -> f32 {
    if x > 0.0 {
        let xd = x as f64;
        let mut guess = frsqrte(xd);
        guess = 0.5 * guess * (3.0 - guess * guess * xd);
        guess = 0.5 * guess * (3.0 - guess * guess * xd);
        guess = 0.5 * guess * (3.0 - guess * guess * xd);
        return (xd * guess) as f32;
    }
    x
}

/// `sumOfSquares` (ftSeak/ftseakspecials.c): two f32 products, then an f32 add.
pub fn sum_of_squares(a: f32, b: f32) -> f32 {
    let a = a * a;
    let c = b * b;
    a + c
}

/// `ftFox_AppealS_GetLR` (ftFox/ftfoxappeals.c): `x1 == x2 ? false : true` (C float compare,
/// so NaN != NaN gives true).
pub fn fox_appeal_s_get_lr(x1: f32, x2: f32) -> bool {
    x1 != x2
}

/// `returnStateVar` (ftNess/ftnessspeciallw.c): `(float) s32`, round-to-nearest.
pub fn ness_return_state_var(v: i32) -> f32 {
    v as f32
}

/// `perm_neg_inline` / `perm_mul_inline` (ftYoshi/ftyoshispecials.c).
pub fn yoshi_perm_neg(x: f32) -> f32 {
    -x
}
pub fn yoshi_perm_mul(a: f32, b: f32) -> f32 {
    a * b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pure_ref::{SQRT_CASES, SUMSQ_CASES};

    #[test]
    fn frsqrte_and_my_sqrtf_match_c() {
        for &(x, sq, est) in SQRT_CASES {
            let xf = f32::from_bits(x);
            assert_eq!(frsqrte(xf as f64).to_bits(), est, "frsqrte({x:#010x})");
            let got = my_sqrtf(xf).to_bits();
            // NaN input returns the same NaN (x > 0 is false); compare bits everywhere.
            assert_eq!(got, sq, "my_sqrtf({x:#010x})");
        }
    }

    #[test]
    fn sum_of_squares_matches_c() {
        for &(a, b, r) in SUMSQ_CASES {
            assert_eq!(sum_of_squares(f32::from_bits(a), f32::from_bits(b)).to_bits(), r);
        }
    }

    #[test]
    fn small_inlines() {
        assert!(!fox_appeal_s_get_lr(1.0, 1.0));
        assert!(fox_appeal_s_get_lr(1.0, -1.0));
        assert!(fox_appeal_s_get_lr(f32::NAN, f32::NAN));
        assert!(!fox_appeal_s_get_lr(0.0, -0.0));
        assert_eq!(ness_return_state_var(16_777_217).to_bits(), 16_777_216f32.to_bits());
        assert_eq!(yoshi_perm_neg(0.0).to_bits(), 0x8000_0000);
        assert_eq!(yoshi_perm_mul(3.0, 0.5), 1.5);
    }
}
