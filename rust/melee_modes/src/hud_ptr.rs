
//! Round 3: `src/melee/if/` and `src/melee/ty/` leaf functions that take pointers to game
//! structs. Conventions and verification as in `gm_ptr.rs` (`tests/oracle_ptr.rs`).
#![allow(non_snake_case)]

use melee_types as mt;

use mt::{DevText, HSD_GObj, _GXColor};

/// soundtest.c `un_802FF884`.
pub fn un_802FF884(arg0: *mut i8) -> bool {
    // one port per C function: forwards to melee_world's TU port
    unsafe { melee_world::tu::soundtest::un_802FF884(arg0) != 0 }
}

/// textlib.c `un_80302DF8`.
pub fn un_80302DF8(arg0: &mut mt::un_80304138_objalloc_t, arg1: mt::soundtest_callback) {
    // one port per C function: forwards to melee_world's TU port
    unsafe { melee_world::tu::textlib::un_80302DF8(arg0, arg1) }
}

pub use crate::tu::textlib::DevText_80302AC0_fp as DevText_80302AC0;
pub use crate::tu::textlib::DevText_HideBackground_fp as DevText_HideBackground;
pub use crate::tu::textlib::DevText_HideCursor_fp as DevText_HideCursor;
pub use crate::tu::textlib::DevText_HideText_fp as DevText_HideText;
pub use crate::tu::textlib::DevText_SetBGColor_fp as DevText_SetBGColor;
pub use crate::tu::textlib::DevText_SetCursorXY_fp as DevText_SetCursorXY;
pub use crate::tu::textlib::DevText_SetScale_fp as DevText_SetScale;
pub use crate::tu::textlib::DevText_SetXY_fp as DevText_SetXY;
pub use crate::tu::textlib::DevText_ShowBackground_fp as DevText_ShowBackground;
pub use crate::tu::textlib::DevText_ShowText_fp as DevText_ShowText;
pub use crate::tu::ifstatus::ifStatus_GetStoredTranslationX_fp as ifStatus_GetStoredTranslationX;
pub use crate::tu::ifstock::ifStock_802F8298_step_anim_fp as ifStock_802F8298_step_anim;
pub use crate::tu::ifstock::ifStock_ClearGObjSlots_fp as ifStock_ClearGObjSlots;
pub use crate::tu::tydisplay::tyDisplay_SetGridSize_fp as tyDisplay_SetGridSize;
pub use crate::tu::tyfigupon::tyFigupon_GetBetCount;
pub use crate::tu::tyfigupon::tyFigupon_StoreDigits_fp as tyFigupon_StoreDigits;
pub use crate::tu::ifprize::un_802FE918_update_x3_fp as un_802FE918_update_x3;
pub use crate::tu::textlib_1::un_80303FD4_count_fp as un_80303FD4_count;
pub use crate::tu::textlib_1::un_80303FD4_first_fp as un_80303FD4_first;
pub use crate::tu::textlib_1::un_80304334_fp as un_80304334;
