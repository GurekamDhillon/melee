//! Round 4: mn callers of the jobj accessors (melee_types' `inline.rs`). Verified bit for bit
//! against the C (`tests/oracle_r4.rs`). `mnDiagram3_GetPopupY` has the one fused site
//! `tools/rust-port/fma_sites.py` finds here.
#![allow(non_snake_case)]

use melee_core::fp::fmadds;
use melee_types as mt;
use mt::{Diagram3, HSD_JObj};

/// A jobj the C passes to `HSD_JObjGetTranslation*`, which asserts it is not NULL.
fn jobj<'a>(p: *mut HSD_JObj) -> &'a HSD_JObj {
    // SAFETY: the menu's joints are live while the menu is; NULL is the C's assert.
    unsafe { p.as_ref() }.expect("jobj (HSD_JObjGetTranslation assert)")
}

/// mndiagram3.c `mnDiagram3_GetPopupSpacing` (static inline).
pub fn mnDiagram3_GetPopupSpacing(popup_row: &HSD_JObj, data: &Diagram3) -> f32 {
    mt::inline::HSD_JObjGetTranslationY(jobj(data.jobjs[9])) - mt::inline::HSD_JObjGetTranslationY(popup_row)
}

/// mndiagram3.c `mnDiagram3_GetPopupY` (static inline): `row_spacing * index + y`, fused.
pub fn mnDiagram3_GetPopupY(popup_row: &HSD_JObj, row_spacing: f32, index: u8) -> f32 {
    fmadds(row_spacing, index as f32, mt::inline::HSD_JObjGetTranslationY(popup_row))
}

/// mndiagram3.c `mnDiagram3_GetRowSpacing` (static inline).
pub fn mnDiagram3_GetRowSpacing(data: &Diagram3) -> f32 {
    mt::inline::HSD_JObjGetTranslationY(jobj(data.jobjs[9])) - mt::inline::HSD_JObjGetTranslationY(jobj(data.jobjs[8]))
}

/// mnname.c `mnName_TextWidth` (static inline).
pub fn mnName_TextWidth(start: f32, end: &HSD_JObj) -> f32 {
    mt::inline::HSD_JObjGetTranslationX(end) - start
}

/// mnvibration.c `mnVibration_GetCursorYSpacing` (static inline).
pub fn mnVibration_GetCursorYSpacing(base_y: f32, jobj: &HSD_JObj) -> f32 {
    mt::inline::HSD_JObjGetTranslationY(jobj) - base_y
}
