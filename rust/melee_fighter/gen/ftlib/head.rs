//! src/melee/ft/ftlib.c: the fighter library (kind/part/position accessors, fighter-list walks, damage and
//! control helpers every other fighter, item and camera TU calls). Generated from the C by
//! `melee_world/gen/mtu/c2rs.py` (libclang AST: C integer / float semantics, clang's -ffp-contract=on fusion).
//! Regenerate with `bash rust/melee_fighter/gen/ftlib/regen.sh`. Calls out of the TU are `ext::` hooks
//! (linked by `melee_link`, unset: panic "TODO"). The hand port `crate::ftlib` (Rust-typed, oracle-checked by
//! tests/ftlib_ref.rs) stays for the legacy `vs_match`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use crate::types::*;

