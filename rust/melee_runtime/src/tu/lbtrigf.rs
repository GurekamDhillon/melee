//! Port of src/melee/lb/lbtrigf.c (TARGET_PC branch): the game's atanf / atan2f / acosf / asinf and
//! the file's statics `lb_sqrtf`, `lbTrigf_Fmadds`, `lbTrigf_Fnmsubs`. atanf/atan2f/acosf/asinf are
//! re-exports of `crate::lbtrigf` (ported earlier, now also checked here against the whole TU).
//! Checked against the C (shipping contraction) by tests/lbtu_lbtrigf.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
use melee_core::fp::Fmadd as _;
#[allow(unused_imports)]
use crate::gekko::frsqrte as frsqrte_lbtrigf;
#[allow(unused_imports)]
use crate::lbtrigf::{ATANF_LOOKUP, BITWISE_PI_2, INF, M_PI, M_PI_2, NAN, SIGN_BIT, fnmsubs, lk, nr_step_f as nr_step_f_lbtrigf};
use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_runtime/gen/lbtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code, unreachable_code)]
    use super::*;
    use core::ffi::c_void;
    pub mod hook {
        use super::*;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut MSL_TrigF_80400770: [f32; 1] = unsafe { core::mem::zeroed() };
    pub static mut MSL_TrigF_80400774: [f32; 1] = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

pub use crate::lbtrigf::{asinf, atan2f, atanf};
const NAN_BITS: u32 = 0x7FFF_FFFF; // MSL_TrigF_80400770[0]
const INF_BITS: u32 = 0x7F80_0000; // MSL_TrigF_80400774[0]

/// `lbTrigf_Fmadds`: `(float) ((double) a * (double) c + (double) b)`; the C build contracts the
/// double expression into one fma.
pub fn lbTrigf_Fmadds(a: f32, c: f32, b: f32) -> f32 {
    melee_core::ct!("lbTrigf_Fmadds");
    (a as f64).fmadd(c as f64, b as f64) as f32
}

/// `lbTrigf_Fnmsubs`: `(float) -((double) a * (double) c - (double) b)` (one fma with the sign folded in).
pub fn lbTrigf_Fnmsubs(a: f32, c: f32, b: f32) -> f32 {
    melee_core::ct!("lbTrigf_Fnmsubs");
    (-(a as f64).fmadd(c as f64, -(b as f64))) as f32
}

/// The float Newton step: `0.5f * g * (3.0f - g * g * v)`, the `3 - (g*g)*v` fused.
#[inline]
fn nr_step_f(g: f32, v: f32) -> f32 {
    (0.5f32 * g) * fmadds(-(g * g), v, 3.0)
}

/// `lb_sqrtf` (static): despite the name it returns the refined reciprocal square root.
pub fn lb_sqrtf(x: f32) -> f32 {
    melee_core::ct!("lb_sqrtf");
    if x > 0.0 {
        let mut guess = crate::gekko::frsqrte(x as f64) as f32;
        guess = nr_step_f(guess, x);
        guess = nr_step_f(guess, x);
        guess = nr_step_f(guess, x);
        return guess;
    }
    if x != 0.0 {
        return f32::from_bits(NAN_BITS);
    }
    f32::from_bits(INF_BITS)
}

// ---- folded from lbtrigf.rs (consolidation steps 21/22) ----
/// `atanf_fp` (TARGET_PC branch of lbtrigf.c).
pub fn atanf_fp(x: f32) -> f32 {
    melee_core::ct!("atanf_fp");
    let silver_ratio = 2.4142136573791504f32;
    let silver_ratio_conjugate = 0.4142135679721832f32;
    let mut lookup_index: i32 = -1;
    let mut x_ge_ratio = false;
    let sign_bit_x = x.to_bits() & SIGN_BIT;
    let x = f32::from_bits(x.to_bits() & !SIGN_BIT);
    let xb = x.to_bits() as i32;

    let mut result;
    if x >= silver_ratio {
        x_ge_ratio = true;
        result = 1.0f32 / x;
    } else if silver_ratio_conjugate < x {
        lookup_index = 0;
        match x.to_bits() & 0x7F80_0000 {
            0x3F00_0000 => {
                if !(xb < 0x3F08_D5B9) {
                    lookup_index = 1;
                }
                if !(xb < 0x3F52_1801) {
                    lookup_index += 1;
                }
            }
            0x3F80_0000 => {
                lookup_index = 2;
                if !(xb < 0x3F9B_F7EC) {
                    lookup_index = 3;
                }
                if !(xb < 0x3FEF_789E) {
                    lookup_index += 1;
                }
            }
            0x4000_0000 => lookup_index = 4,
            _ => {}
        }
        let offset_39 = lk(lookup_index, 39);
        let offset_33 = lk(lookup_index, 33);
        result = 1.0f32 / (offset_33 + (x + offset_39));
        result = fnmsubs(result, lk(lookup_index, 7), offset_33)
            + fnmsubs(result, lk(lookup_index, 13), offset_39);
    } else {
        result = x;
    }

    let rs = result * result;
    let a = &ATANF_LOOKUP;
    let mut poly = fmadds(rs, a[6], a[5]);
    poly = fmadds(rs, poly, a[4]);
    poly = fmadds(rs, poly, a[3]);
    poly = fmadds(rs, poly, a[2]);
    poly = fmadds(rs, poly, a[1]);
    result = fmadds(result * rs, poly, result);
    result += lk(lookup_index, 27);
    result += lk(lookup_index, 20);

    if x_ge_ratio {
        result -= M_PI_2 as f32;
        return if sign_bit_x != 0 { result } else { -result };
    }
    f32::from_bits(result.to_bits() | sign_bit_x)
}

/// `atan2f_fp`.
pub fn atan2f_fp(y: f32, x: f32) -> f32 {
    melee_core::ct!("atan2f_fp");
    let sx = x.to_bits() & SIGN_BIT;
    let sy = y.to_bits() & SIGN_BIT;
    if sx == sy {
        if sx != 0 {
            return if x == -0.0 { -M_PI_2 as f32 } else { atanf_fp(y / x) - M_PI as f32 };
        }
        return if x != 0.0 { atanf_fp(y / x) } else { M_PI_2 as f32 };
    }
    if x < 0.0 {
        return M_PI as f32 + atanf_fp(y / x);
    }
    if x != 0.0 {
        return atanf_fp(y / x);
    }
    f32::from_bits(sy.wrapping_add(BITWISE_PI_2))
}

/// `acosf`.
pub fn acosf(x: f32) -> f32 {
    melee_core::ct!("acosf");
    let mut result = fnmsubs(x, x, 1.0); // 1.0F - x * x, fused
    if result > 0.0 {
        let mut guess = frsqrte_lbtrigf(result as f64) as f32;
        guess = nr_step_f_lbtrigf(guess, result);
        guess = nr_step_f_lbtrigf(guess, result);
        guess = nr_step_f_lbtrigf(guess, result);
        result = guess;
    } else if result != 0.0 {
        result = f32::from_bits(NAN);
    } else {
        result = f32::from_bits(INF);
    }
    M_PI_2 as f32 - atanf_fp(x * result)
}

/// `lb_sqrtf_fp` (static): despite the name it returns the refined reciprocal square root.
pub fn lb_sqrtf_fp(x: f32) -> f32 {
    melee_core::ct!("lb_sqrtf_fp");
    if x > 0.0 {
        let mut guess = frsqrte_lbtrigf(x as f64) as f32;
        guess = nr_step_f_lbtrigf(guess, x);
        guess = nr_step_f_lbtrigf(guess, x);
        guess = nr_step_f_lbtrigf(guess, x);
        return guess;
    }
    if x != 0.0 {
        return f32::from_bits(NAN);
    }
    f32::from_bits(INF)
}

/// `asinf_fp`: `atanf_fp(x * lb_sqrtf_fp(-(x * x - 1.0f)))`, the inner `x*x - 1` fused.
pub fn asinf_fp(x: f32) -> f32 {
    melee_core::ct!("asinf_fp");
    atanf_fp(x * lb_sqrtf_fp(-fmadds(x, x, -1.0)))
}
