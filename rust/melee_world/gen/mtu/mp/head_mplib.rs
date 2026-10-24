//! Port of src/melee/mp/mplib.c: the map collision library (the ground collision vertex / line /
//! joint tables, line walking and the floor / ceiling / wall segment queries, joint list editing,
//! bounding checks and the debug drawing). Generated from the C by `gen/mtu/c2rs.py` (libclang AST:
//! every implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds` /
//! `mul_add`, checked against `tools/rust-port/fma_sites.py`), with the goto-state-machine functions
//! that the tool cannot structure in `gen/mtu/hand/`. `bool` is MSL `int`. Callees of unported TUs
//! are `ext` hooks named like the C (unset: panics "TODO"); the TARGET_PC script hooks
//! (`ScriptGame_*`) are ext hooks too.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

/// `LINEID_CHECK` (hand ports): `HSD_ASSERTREPORT(line, 0, "%s:%d:not found lineID=%d")`
unsafe fn lineid_check(line: u32, line_id: i32) {
    if line_id == -1 || line_id >= (*mpLib_804D64B4).line_count {
        panic!("src/melee/mp/mplib.c:{}:not found lineID={}", line, line_id);
    }
}
