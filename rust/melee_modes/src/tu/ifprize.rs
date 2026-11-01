//! ifprize.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;

// ---- folded from hud_ptr.rs (consolidation steps 21/22) ----
/// ifprize.c `un_802FE918_update_x3_fp`. `new_x3` is uninitialised in the C when no `k`
/// matches; the shipping build zero-initialises locals (`-ftrivial-auto-var-init=zero`).
pub fn un_802FE918_update_x3_fp(x3_ptr: &mut u8, r: &mut i32) {
    melee_core::ct!("un_802FE918_update_x3_fp");
    let old_x3 = *x3_ptr as i32;
    let mut new_x3 = 0i32;
    for k in 0..3 {
        if k == old_x3 {
            *r = r.wrapping_add(1);
        } else if *r == k {
            new_x3 = *r;
            break;
        }
    }
    *x3_ptr = new_x3 as u8;
}
