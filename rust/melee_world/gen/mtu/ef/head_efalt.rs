//! Mechanical native port of src/melee/ef/efalt.c (
//! TARGET_PC variant), generated from the C by `gen/mtu/c2rs.py` (libclang AST: every
//! implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds`). Verified against
//! the C compiled natively by the state oracle `tests/tu_ef.rs` (`python rust/melee_world/gen/mtu/mk_ef.py`).
//! Callees of other TUs are `ext` hooks named like the C (unset: panic "TODO", or the native port
//! `gen/rewire/` wired). `bool` is MSL `int`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;
