
//! Round 4: mn callers of the jobj accessors (melee_types' `inline.rs`). Verified bit for bit
//! against the C (`tests/oracle_r4.rs`). `mnDiagram3_GetPopupY` has the one fused site
//! `tools/rust-port/fma_sites.py` finds here.
#![allow(non_snake_case)]

use melee_core::fp::fmadds;

use melee_types as mt;

use mt::{Diagram3, HSD_JObj};

/// A jobj the C passes to `HSD_JObjGetTranslation*`, which asserts it is not NULL.
pub(crate) fn jobj<'a>(p: *mut HSD_JObj) -> &'a HSD_JObj {
    // SAFETY: the menu's joints are live while the menu is; NULL is the C's assert.
    unsafe { p.as_ref() }.expect("jobj (HSD_JObjGetTranslation assert)")
}

pub use crate::tu::mndiagram3::mnDiagram3_GetPopupSpacing;
pub use crate::tu::mndiagram3::mnDiagram3_GetPopupY;
pub use crate::tu::mndiagram3::mnDiagram3_GetRowSpacing;
pub use crate::tu::mnname::mnName_TextWidth_fp as mnName_TextWidth;
pub use crate::tu::mnvibration::mnVibration_GetCursorYSpacing_fp as mnVibration_GetCursorYSpacing;
