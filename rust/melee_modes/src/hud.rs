
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

pub use crate::tu::textlib::DevText_Clamp_fp as DevText_Clamp;
pub use crate::tu::tydisplay::_tyDisplay_80318CB4_calc_dist_sq_fp as _tyDisplay_80318CB4_calc_dist_sq;
pub use crate::tu::textlib_1::adjust_fp as adjust;
pub use crate::tu::soundtest::fn_802FFCC8_fp as fn_802FFCC8;
pub use crate::tu::ifstock::ifStock_802F89F8_count_fp as ifStock_802F89F8_count;
pub use crate::tu::ifstock::ifStock_802F89F8_digit_fp as ifStock_802F89F8_digit;
pub use crate::tu::ifstock::ifStock_802F89F8_divisor_fp as ifStock_802F89F8_divisor;
