//! itparasol.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;

// ---- folded from tu_util.rs (consolidation steps 21/22) ----
/// `HSD_JObjGetChild`: NULL-safe.
#[inline]
pub fn jobj_child_fp(j: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("jobj_child_fp");
    if j.is_null() { core::ptr::null_mut() } else { unsafe { (*j).child } }
}
