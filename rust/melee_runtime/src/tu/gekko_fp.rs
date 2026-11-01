//! `pc/gameworld/gekko_fp.c`: reuse the bit-exact estimate and trigonometric ports.
//! The reduction returns `(value, quadrant)` instead of writing a C out-parameter.

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use crate::gekko::{SINCOS_ON_QUADRANT, SINCOS_POLY, fabs_lt, sincos_even as sincos_even_gekko, sincos_odd_head as sincos_odd_head_gekko, sincos_reduce as sincos_reduce_gekko};
pub use crate::gekko::{sincos_even as gk_sincos_even, sincos_odd_head as gk_sincos_odd_head, sincos_reduce as gk_sincos_reduce, tanf};pub use melee_core::fp::{make_quiet as gekko_make_quiet, nan as gekko_nan};

/// `__frsqrte`: the Gekko reciprocal-square-root estimate (checked against the C by
/// `tests/pure_gekko_fp.rs`).
pub fn __frsqrte(val: f64) -> f64 {
    melee_core::ct!("__frsqrte");
    melee_core::fp::frsqrte(val)
}

/// `__fres`: the Gekko reciprocal estimate.
pub fn __fres(val: f64) -> f64 {
    melee_core::ct!("__fres");
    melee_core::fp::fres(val)
}

/// The C union pun preserves the complete bit pattern, including signaling NaNs.
pub fn gekko_bits_to_double(bits: u64) -> f64 {
    melee_core::ct!("gekko_bits_to_double");
    f64::from_bits(bits)
}

/// The C union pun does no floating-point arithmetic.
pub fn gekko_double_to_bits(value: f64) -> u64 {
    melee_core::ct!("gekko_double_to_bits");
    value.to_bits()
}

pub fn gekko_infinity(negative: i32) -> f64 {
    melee_core::ct!("gekko_infinity");
    melee_core::fp::infinity(negative != 0)
}

// ---- folded from gekko.rs (consolidation steps 21/22) ----
// ---- MSL sinf / cosf / tanf_fp ----------------------------------------------------------------

/// gekko_fp.c `gk_fmadds`. In the C build its body `(double)a*(double)c + (double)b` is itself
/// contracted to a double `fma`; the float product is exact in double, so that equals
/// [`fmadds`] (f64 multiply, f64 add, then f32). Not `f32::mul_add`.
#[inline(always)]
pub fn gk_fmadds(a: f32, c: f32, b: f32) -> f32 {
    melee_core::ct!("gk_fmadds");
    fmadds(a, c, b)
}

/// MSL `sinf` as the retail binary computes it.
pub fn sinf(x: f32) -> f32 {
    melee_core::ct!("sinf");
    let (y, n) = sincos_reduce_gekko(x);
    let n = n as usize;
    if fabs_lt(y) {
        let n = n << 1;
        return gk_fmadds(SINCOS_POLY[9], SINCOS_ON_QUADRANT[n + 1] * y, SINCOS_ON_QUADRANT[n]);
    }
    let ysq = y * y;
    if n & 1 != 0 {
        return sincos_even_gekko(ysq) * SINCOS_ON_QUADRANT[n << 1];
    }
    let n = n << 1;
    (gk_fmadds(sincos_odd_head_gekko(ysq), ysq, SINCOS_POLY[9]) * y) * SINCOS_ON_QUADRANT[n + 1]
}

/// MSL `cosf` as the retail binary computes it.
pub fn cosf(x: f32) -> f32 {
    melee_core::ct!("cosf");
    let (y, n) = sincos_reduce_gekko(x);
    let n = n as usize;
    if fabs_lt(y) {
        let n = n << 1;
        // fnmsubs, a double fma in the C build; the product of two floats is exact in double.
        return (SINCOS_ON_QUADRANT[n + 1] as f64 - y as f64 * SINCOS_ON_QUADRANT[n] as f64) as f32;
    }
    let ysq = y * y;
    if n & 1 != 0 {
        let n = n << 1;
        return (y * -gk_fmadds(sincos_odd_head_gekko(ysq), ysq, SINCOS_POLY[9])) * SINCOS_ON_QUADRANT[n];
    }
    sincos_even_gekko(ysq) * SINCOS_ON_QUADRANT[(n << 1) + 1]
}

/// MSL `tanf_fp`.
pub fn tanf_fp(x: f32) -> f32 {
    melee_core::ct!("tanf_fp");
    sinf(x) / cosf(x)
}
