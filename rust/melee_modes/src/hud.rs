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

pub use melee_world::tu::ifstock::ifStock_802F89F8_count;

pub use melee_world::tu::ifstock::ifStock_802F89F8_divisor;

pub use melee_world::tu::ifstock::ifStock_802F89F8_digit;

/// soundtest.c `fn_802FFCC8`.
pub fn fn_802FFCC8(_arg0: i32) -> bool {
    false
}

pub use melee_world::tu::textlib::DevText_Clamp;

/// textlib_1.c `adjust`: halve r, g, b (int arithmetic, truncating), keep a.
pub fn adjust(mut c: GXColor) -> GXColor {
    c.r = (c.r as i32 * 50 / 100) as u8;
    c.g = (c.g as i32 * 50 / 100) as u8;
    c.b = (c.b as i32 * 50 / 100) as u8;
    c
}

pub use melee_items::tu::tydisplay::_tyDisplay_80318CB4_calc_dist_sq;
