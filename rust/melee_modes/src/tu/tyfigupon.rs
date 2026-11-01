//! tyfigupon.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;

// ---- folded from hud_ptr.rs (consolidation steps 21/22) ----
/// tyfigupon.c `tyFigupon_StoreDigits_fp`: decimal digits, least significant first.
pub fn tyFigupon_StoreDigits_fp(digits: &mut [i32], mut value: i32) {
    melee_core::ct!("tyFigupon_StoreDigits_fp");
    let mut i = 0;
    loop {
        digits[i] = value % 10;
        i += 1;
        value /= 10;
        if value <= 0 {
            break;
        }
    }
}

/// tyfigupon.c `tyFigupon_GetBetCount`.
pub fn tyFigupon_GetBetCount(ef4: &mt::un_804D6EF4_t) -> i32 {
    melee_core::ct!("tyFigupon_GetBetCount");
    ef4.x5E as i32
}
