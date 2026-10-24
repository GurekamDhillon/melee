//! Port of src/melee/mp/mpisland.c: the "island" segment lists the map collision keeps for floors
//! and ceilings (one `mp_UnkStruct0` per connected run of lines, used by the stage / item code to
//! find the platform under a point), and the TARGET_PC script-joint batch rebuild. Generated from
//! the C by `gen/mtu/c2rs.py` (see `mplib.rs`); `mpIsland_8005AE1C` is a hand-structured goto
//! state machine (`gen/mtu/hand/`). `mpIsland_TerrainPalette` is `{ terrain kind, GXColor }[20]`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;
