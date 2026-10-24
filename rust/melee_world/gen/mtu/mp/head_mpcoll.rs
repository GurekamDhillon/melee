//! Port of src/melee/mp/mpcoll.c: environment collision for fighters / items (the `CollData` update
//! loop: ECB sources, interpolation, floor / ceiling / wall passes, ledge grabs and pushout, the
//! per-substep callbacks). Generated from the C by `gen/mtu/c2rs.py` (see `mplib.rs`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

/// MSL `__fpclassifyf` / `__fpclassifyd` (math.h; FP_NAN 1, FP_INFINITE 2, FP_ZERO 3, FP_NORMAL 4, FP_SUBNORMAL 5)
fn __fpclassifyf(x: f32) -> i32 {
    let b = x.to_bits() as i32;
    match b & 0x7F800000 {
        0x7F800000 => if b & 0x007FFFFF != 0 { 1 } else { 2 },
        0 => if b & 0x007FFFFF != 0 { 5 } else { 3 },
        _ => 4,
    }
}

fn __fpclassifyd(x: f64) -> i32 {
    let hi = (x.to_bits() >> 32) as u32 as i32;
    let lo = x.to_bits() as u32;
    match hi & 0x7ff00000 {
        0x7ff00000 => if (hi & 0x000fffff) != 0 || lo != 0 { 1 } else { 2 },
        0 => if (hi & 0x000fffff) != 0 || lo != 0 { 5 } else { 3 },
        _ => 4,
    }
}
