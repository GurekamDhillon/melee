//! lbdvd.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]

// ---- folded from lb.rs (consolidation steps 21/22) ----
/// lbdvd.c `same`.
pub fn same(a: i32, b: i32) -> i32 {
    melee_core::ct!("same");
    (a == b) as i32
}
