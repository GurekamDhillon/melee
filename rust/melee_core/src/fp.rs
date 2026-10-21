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
