//! lbsnap.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::lb_r9::{Unk80433380_0};
#[allow(unused_imports)]
use melee_types as mt;
// ---- folded from lb.rs (consolidation steps 21/22) ----
/// lbsnap.c `RGB565_TO_RGB5A3_fp`.
pub fn RGB565_TO_RGB5A3_fp(pixel: u16) -> u16 {
    melee_core::ct!("RGB565_TO_RGB5A3_fp");
    let mut result = (pixel >> 1) & (0x7C00 | 0x03E0);
    result |= pixel & 0x001F;
    result | 0x8000
}

/// lbsnap.c `lbSnap_8001E204_fp`.
pub fn lbSnap_8001E204_fp() -> i32 {
    melee_core::ct!("lbSnap_8001E204_fp");
    0x3E840
}

/// lbsnap.c `lbSnap_8001E210_fp`.
pub fn lbSnap_8001E210_fp() -> i32 {
    melee_core::ct!("lbSnap_8001E210_fp");
    0x840
}

// ---- folded from lb_ptr.rs (consolidation steps 21/22) ----
/// lbsnap.c `lbSnap_ClearText_fp`: 0x21 NULs.
pub fn lbSnap_ClearText_fp(text: &mut [i8]) {
    melee_core::ct!("lbSnap_ClearText_fp");
    text[..0x21].fill(0);
}

// ---- folded from lb_r9.rs (consolidation steps 21/22) ----
/// lbsnap.c `lbSnap_GetSaveDataOffset_fp`: `xC` plus the offset of `x38` (0x38 on both the GC and
/// native layouts: the record holds no pointers).
pub fn lbSnap_GetSaveDataOffset_fp(snap: &Unk80433380_0) -> i32 {
    melee_core::ct!("lbSnap_GetSaveDataOffset_fp");
    snap.xC.wrapping_add(core::mem::offset_of!(Unk80433380_0, x38) as i32)
}
