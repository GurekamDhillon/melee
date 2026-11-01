//! it_2725.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::itacc::{ip};

// ---- folded from itacc.rs (consolidation steps 21/22) ----
/// `it_80274C60_fp`.
pub unsafe fn it_80274C60_fp(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80274C60_fp");
    ip(gobj).xDC8_word.flags.set_xC(0);
}
