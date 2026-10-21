//! Fighter-independent helpers from the group-4 character TUs, ported by hand.
//!
//! Everything else in the character code reads or writes `Fighter` (group 3) and is not ported
//! yet; see docs/rust-port/group-4 fighter characters.md. Expected values for the tests come
//! from the C compiled natively (gen/ref_pure.c, gen/run_ref.sh -> src/pure_ref.rs).

/// Gekko `frsqrte` (now in melee_core::fp; this crate's tests still check it on 220 inputs).
pub use melee_core::fp::frsqrte;

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
