//! itkyasarin.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::itacc::{ip};

// ---- folded from itacc.rs (consolidation steps 21/22) ----
/// `it_802ECD1C_fp` (Kyasarin).
pub unsafe fn it_802ECD1C_fp(gobj: *mut HSD_GObj, v: *mut Vec) {
    melee_core::ct!("it_802ECD1C_fp");
    *v = ip(gobj).pos;
}
