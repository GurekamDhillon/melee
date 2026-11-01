//! soundtest.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
// ---- folded from hud.rs (consolidation steps 21/22) ----
/// soundtest.c `fn_802FFCC8_fp`.
pub fn fn_802FFCC8_fp(_arg0: i32) -> bool {
    melee_core::ct!("fn_802FFCC8_fp");
    false
}

// ---- folded from hud_r6.rs (consolidation steps 21/22) ----
/// soundtest.c `un_802FFCD0_fp`: fill `arr[0..count]` with `arr[count]`.
pub fn un_802FFCD0_fp(count: i32, arr: &mut [i32]) {
    melee_core::ct!("un_802FFCD0_fp");
    let mut i = 0;
    while i < count {
        arr[i as usize] = arr[count as usize];
        i += 1;
    }
}
