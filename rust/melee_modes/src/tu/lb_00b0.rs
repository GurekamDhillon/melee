//! lb_00B0.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::lb_r8::{jobj_tree_any};
#[allow(unused_imports)]
use melee_types::HSD_JObj;

// ---- folded from lb_r6.rs (consolidation steps 21/22) ----
/// lb_00B0.c `lb_8000B074_fp`: the joint has an AObj that is not AOBJ_NO_ANIM.
pub fn lb_8000B074_fp(jobj: &HSD_JObj) -> bool {
    melee_core::ct!("lb_8000B074_fp");
    const AOBJ_NO_ANIM: u32 = 1 << 30;
    // SAFETY: `aobj` is NULL or a live AObj, as in the C.
    match unsafe { jobj.aobj.as_ref() } {
        Some(a) => a.flags & AOBJ_NO_ANIM == 0,
        None => false,
    }
}

/// lb_00B0.c `memzero_fp`: zero `size` bytes (`while (size--)`; size >= 0).
pub fn memzero_fp(mem: &mut [u8], size: isize) {
    melee_core::ct!("memzero_fp");
    for b in &mut mem[..size as usize] {
        *b = 0;
    }
}

// ---- folded from lb_r8.rs (consolidation steps 21/22) ----
/// lb_00B0.c `lb_8000B09C_fp`: some joint in the tree has an AObj without AOBJ_NO_ANIM.
///
/// # Safety
/// See `jobj_tree_any`.
pub unsafe fn lb_8000B09C_fp(jobj: *const HSD_JObj) -> bool {
    melee_core::ct!("lb_8000B09C_fp");
    jobj_tree_any(jobj, |f| f & (1 << 30) == 0)
}

/// lb_00B0.c `lb_8000B134_fp`: some joint in the tree has an AObj with AOBJ_REWINDED.
///
/// # Safety
/// See `jobj_tree_any`.
pub unsafe fn lb_8000B134_fp(jobj: *const HSD_JObj) -> bool {
    melee_core::ct!("lb_8000B134_fp");
    jobj_tree_any(jobj, |f| f & (1 << 26) != 0)
}
