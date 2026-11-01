//! ifstock.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use mt::HSD_MObj;
#[allow(unused_imports)]
use mt::_GXColor;
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::HSD_GObj;
#[allow(unused_imports)]
use melee_runtime::cabi;

// ---- folded from hud.rs (consolidation steps 21/22) ----
/// ifstock.c `ifStock_802F89F8_count_fp`: decimal digit count (0 for 0), capped by a 32-step loop.
pub fn ifStock_802F89F8_count_fp(mut coins: i32) -> i32 {
    melee_core::ct!("ifStock_802F89F8_count_fp");
    for i in 0..32 {
        if coins == 0 {
            return i;
        }
        coins /= 10;
    }
    0
}

/// ifstock.c `ifStock_802F89F8_divisor_fp`: 10^position (wrapping), 1 for position <= 0.
pub fn ifStock_802F89F8_divisor_fp(position: i32) -> i32 {
    melee_core::ct!("ifStock_802F89F8_divisor_fp");
    let mut divisor: i32 = 1;
    if position != 0 {
        for _ in 0..position.max(0) {
            divisor = divisor.wrapping_mul(10);
        }
    }
    divisor
}

/// ifstock.c `ifStock_802F89F8_digit_fp`: the position-th decimal digit (1-based) of coins.
/// position >= 33 makes the C divisor 0 (gwtool's div-by-zero rule; not exercised).
pub fn ifStock_802F89F8_digit_fp(coins: i32, position: i32) -> i32 {
    melee_core::ct!("ifStock_802F89F8_digit_fp");
    if position.wrapping_sub(1) == 0 {
        return coins % 10;
    }
    let mut divisor: i32 = 1;
    for _ in 0..position.wrapping_sub(1).max(0) {
        divisor = divisor.wrapping_mul(10);
    }
    cabi::sdiv(coins, divisor) % 10
}

// ---- folded from hud_ptr.rs (consolidation steps 21/22) ----
/// ifstock.c `ifStock_802F8298_step_anim_fp`.
pub fn ifStock_802F8298_step_anim_fp(anim: &mut u8) {
    melee_core::ct!("ifStock_802F8298_step_anim_fp");
    if *anim < 10 {
        *anim += 1;
    }
}

/// ifstock.c `ifStock_ClearGObjSlots_fp`.
pub fn ifStock_ClearGObjSlots_fp(slots: &mut [*mut HSD_GObj], count: i32) {
    melee_core::ct!("ifStock_ClearGObjSlots_fp");
    for s in &mut slots[..count.max(0) as usize] {
        *s = core::ptr::null_mut();
    }
}

// ---- folded from hud_r5.rs (consolidation steps 21/22) ----
/// ifstock.c `copyColor_fp`: the material's diffuse r, g, b from `arg` (alpha kept).
pub fn copyColor_fp(q: &mut HSD_MObj, arg: &_GXColor) {
    melee_core::ct!("copyColor_fp");
    // SAFETY: the C dereferences `q->mat` unchecked.
    let m = unsafe { &mut *q.mat };
    m.diffuse.r = arg.r;
    m.diffuse.g = arg.g;
    m.diffuse.b = arg.b;
}
