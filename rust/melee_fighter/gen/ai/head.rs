//! src/melee/ft/kinds/ftCommon/ftCo_0A01.c: the CPU fighter AI (target selection, behaviour
//! dispatch, movement/recovery planning, item handling), everything `ftCo_800B3900` runs.
//! Generated from the C by `melee_world/gen/mtu/c2rs.py` (libclang AST: C integer / float semantics,
//! clang's -ffp-contract=on fusion); the goto functions are hand ports in `gen/ai/hand/`.
//! Regenerate with `bash rust/melee_fighter/gen/ai/regen.sh`. Calls out of the TU are `ext::` hooks
//! (unset: panic "TODO"); the melee crate's `vs_match/cpu.rs` binds them to the native sim.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use crate::types::*;
