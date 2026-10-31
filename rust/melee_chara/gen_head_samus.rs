//! Mechanical native port of the C file named below, generated from the C by `rust/melee_world/gen/mtu/c2rs.py`
//! (libclang AST: every implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds`).
//! Callees of other TUs are `ext` hooks named like the C (unset: panic "TODO"; `melee_link` wires them to the
//! canonical TU function). `bool` is MSL `int`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;
