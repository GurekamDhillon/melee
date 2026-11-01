//! ftyoshi.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::types::*;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use crate::chardata::{da};

// ---- folded from chardata.rs (consolidation steps 21/22) ----
pub fn ftYs_Init_8012BAC0_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftYs_Init_8012BAC0_fp");
da::<ftYoshiAttributes>(fp).x120
}
