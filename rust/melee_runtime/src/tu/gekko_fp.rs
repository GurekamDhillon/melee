//! `pc/gameworld/gekko_fp.c`: reuse the bit-exact estimate and trigonometric ports.
//! The reduction returns `(value, quadrant)` instead of writing a C out-parameter.

pub use crate::gekko::{
    cosf, gk_fmadds, sincos_even as gk_sincos_even, sincos_odd_head as gk_sincos_odd_head,
    sincos_reduce as gk_sincos_reduce, sinf, tanf,
};
pub use melee_core::fp::{
    fres as __fres, frsqrte as __frsqrte, make_quiet as gekko_make_quiet, nan as gekko_nan,
};

/// The C union pun preserves the complete bit pattern, including signaling NaNs.
pub fn gekko_bits_to_double(bits: u64) -> f64 {
    f64::from_bits(bits)
}

/// The C union pun does no floating-point arithmetic.
pub fn gekko_double_to_bits(value: f64) -> u64 {
    value.to_bits()
}

pub fn gekko_infinity(negative: i32) -> f64 {
    melee_core::fp::infinity(negative != 0)
}
