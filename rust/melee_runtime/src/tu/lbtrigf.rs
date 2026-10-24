//! Port of src/melee/lb/lbtrigf.c (TARGET_PC branch): the game's atanf / atan2f / acosf / asinf and
//! the file's statics `lb_sqrtf`, `lbTrigf_Fmadds`, `lbTrigf_Fnmsubs`. atanf/atan2f/acosf/asinf are
//! re-exports of `crate::lbtrigf` (ported earlier, now also checked here against the whole TU).
//! Checked against the C (shipping contraction) by tests/lbtu_lbtrigf.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

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

pub use crate::lbtrigf::{acosf, asinf, atan2f, atanf};

const NAN_BITS: u32 = 0x7FFF_FFFF; // MSL_TrigF_80400770[0]
const INF_BITS: u32 = 0x7F80_0000; // MSL_TrigF_80400774[0]

/// `lbTrigf_Fmadds`: `(float) ((double) a * (double) c + (double) b)`; the C build contracts the
/// double expression into one fma.
pub fn lbTrigf_Fmadds(a: f32, c: f32, b: f32) -> f32 {
    (a as f64).mul_add(c as f64, b as f64) as f32
}

/// `lbTrigf_Fnmsubs`: `(float) -((double) a * (double) c - (double) b)` (one fma with the sign folded in).
pub fn lbTrigf_Fnmsubs(a: f32, c: f32, b: f32) -> f32 {
    (-(a as f64).mul_add(c as f64, -(b as f64))) as f32
}

/// The float Newton step: `0.5f * g * (3.0f - g * g * v)`, the `3 - (g*g)*v` fused.
#[inline]
fn nr_step_f(g: f32, v: f32) -> f32 {
    (0.5f32 * g) * fmadds(-(g * g), v, 3.0)
}

/// `lb_sqrtf` (static): despite the name it returns the refined reciprocal square root.
pub fn lb_sqrtf(x: f32) -> f32 {
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
