//! Port of src/melee/cm/camera.c: the game camera (subjects, bounds, follow / zoom / quake, the
//! per-mode update callbacks, the pause and training cameras, the render pass, the TARGET_PC script
//! camera and the projection helpers). Generated from the C by `gen/mtu/c2rs.py` (libclang AST:
//! every implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds`,
//! checked against `tools/rust-port/fma_sites.py`); `gen/mtu/cm/hand/` holds the functions the
//! tool cannot structure. Callees of other TUs are `ext` hooks named like the C (unset: panic
//! "TODO", or the native port `gen/rewire/` wired). `bool` is MSL `int`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

/// MSL `__fpclassifyf` (math.h; FP_NAN 1, FP_INFINITE 2, FP_ZERO 3, FP_NORMAL 4, FP_SUBNORMAL 5)
fn __fpclassifyf(x: f32) -> i32 {
    let b = x.to_bits() as i32;
    match b & 0x7F800000 {
        0x7F800000 => if b & 0x007FFFFF != 0 { 1 } else { 2 },
        0 => if b & 0x007FFFFF != 0 { 5 } else { 3 },
        _ => 4,
    }
}

/// `lbVector_Add` / `lbVector_Diff` (lb/lbvector.c; ported in melee_runtime, checked against the C)
use melee_runtime::tu::lbvector::{lbVector_Add, lbVector_Diff};
