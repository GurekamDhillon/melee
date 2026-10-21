//! HUD (`src/melee/if/`; `if` is a Rust keyword) and trophy (`src/melee/ty/`) leaf functions.
//! Hand ports, same names as the C, checked by `tests/oracle.rs`.
#![allow(non_snake_case)]

use melee_core::fp::fmadds;
use melee_runtime::cabi;

use crate::GXColor;

/// ifnametag.c `inlineA1`.
pub fn inlineA1(var_f31: f32) -> f32 {
    let q = cabi::f2i32(var_f31 as f64);
    if q >= 16 {
        28.0
    } else {
        ((q % 4) + 20) as f32
    }
}

/// ifstock.c `ifStock_802F89F8_count`: decimal digit count (0 for 0), capped by a 32-step loop.
pub fn ifStock_802F89F8_count(mut coins: i32) -> i32 {
    for i in 0..32 {
        if coins == 0 {
            return i;
        }
        coins /= 10;
    }
    0
}

/// ifstock.c `ifStock_802F89F8_divisor`: 10^position (wrapping), 1 for position <= 0.
pub fn ifStock_802F89F8_divisor(position: i32) -> i32 {
    let mut divisor: i32 = 1;
    if position != 0 {
        for _ in 0..position.max(0) {
            divisor = divisor.wrapping_mul(10);
        }
    }
    divisor
}

/// ifstock.c `ifStock_802F89F8_digit`: the position-th decimal digit (1-based) of coins.
/// position >= 33 makes the C divisor 0 (gwtool's div-by-zero rule; not exercised).
pub fn ifStock_802F89F8_digit(coins: i32, position: i32) -> i32 {
    if position.wrapping_sub(1) == 0 {
        return coins % 10;
    }
    let mut divisor: i32 = 1;
    for _ in 0..position.wrapping_sub(1).max(0) {
        divisor = divisor.wrapping_mul(10);
    }
    cabi::sdiv(coins, divisor) % 10
}

/// soundtest.c `fn_802FFCC8`.
pub fn fn_802FFCC8(_arg0: i32) -> bool {
    false
}

/// textlib.c `DevText_Clamp`.
pub fn DevText_Clamp(val: i32, max: i32) -> i32 {
    if max <= val {
        max.wrapping_sub(1)
    } else if val < 0 {
        0
    } else {
        val
    }
}

/// textlib_1.c `adjust`: halve r, g, b (int arithmetic, truncating), keep a.
pub fn adjust(mut c: GXColor) -> GXColor {
    c.r = (c.r as i32 * 50 / 100) as u8;
    c.g = (c.g as i32 * 50 / 100) as u8;
    c.b = (c.b as i32 * 50 / 100) as u8;
    c
}

/// tydisplay.c `_tyDisplay_80318CB4_calc_dist_sq`: `dx*dx + dz*dz`, fused as
/// fmuladd(dx, dx, dz*dz).
pub fn _tyDisplay_80318CB4_calc_dist_sq(dz: f32, dx: f32) -> f32 {
    fmadds(dx, dx, dz * dz)
}
