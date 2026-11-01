//! controller.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::mp::types::*;

// ---- folded from check.rs (consolidation steps 21/22) ----
#[inline]
pub fn sq_fp(x: f32) -> f32 {
    melee_core::ct!("sq_fp");
    x * x
}

// ---- folded from lines.rs (consolidation steps 21/22) ----
#[inline]
pub fn sq_fp2(x: f32) -> f32 {
    melee_core::ct!("sq_fp2");
    x * x
}
