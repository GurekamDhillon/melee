//! textlib.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::lb_r6::memzero;
#[allow(unused_imports)]
use melee_types::DevText as DevText_hud_r7;
#[allow(unused_imports)]
use mt::_GXColor;
#[allow(unused_imports)]
use crate::hud::DevText_Clamp;
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::DevText;

// ---- folded from gm_r4.rs (consolidation steps 21/22) ----
/// textlib.c `DevText_SetCursorX_fp`.
pub fn DevText_SetCursorX_fp(text: &mut DevText, x: i32) {
    melee_core::ct!("DevText_SetCursorX_fp");
    text.cursor_x = DevText_Clamp(x, text.w as i32) as u8;
}

// ---- folded from hud.rs (consolidation steps 21/22) ----
/// textlib.c `DevText_Clamp_fp`.
pub fn DevText_Clamp_fp(val: i32, max: i32) -> i32 {
    melee_core::ct!("DevText_Clamp_fp");
    if max <= val {
        max.wrapping_sub(1)
    } else if val < 0 {
        0
    } else {
        val
    }
}

// ---- folded from hud_ptr.rs (consolidation steps 21/22) ----
/// textlib.c `DevText_SetCursorXY_fp`: clamp into the text grid.
pub fn DevText_SetCursorXY_fp(text: &mut DevText, mut x: i32, mut y: i32) {
    melee_core::ct!("DevText_SetCursorXY_fp");
    if text.w as i32 <= x {
        x = text.w as i32 - 1;
    } else if x < 0 {
        x = 0;
    }
    text.cursor_x = x as u8;
    if text.h as i32 <= y {
        y = text.h as i32 - 1;
    } else if y < 0 {
        y = 0;
    }
    text.cursor_y = y as u8;
}

/// textlib.c `DevText_HideCursor_fp`.
pub fn DevText_HideCursor_fp(text: &mut DevText) {
    melee_core::ct!("DevText_HideCursor_fp");
    text.flags &= !(1 << 4);
}

/// textlib.c `DevText_80302AC0_fp`.
pub fn DevText_80302AC0_fp(text: &mut DevText) {
    melee_core::ct!("DevText_80302AC0_fp");
    text.flags |= 1 << 5;
}

/// textlib.c `DevText_ShowBackground_fp`.
pub fn DevText_ShowBackground_fp(text: &mut DevText) {
    melee_core::ct!("DevText_ShowBackground_fp");
    text.flags &= !(1 << 6);
}

/// textlib.c `DevText_HideBackground_fp`.
pub fn DevText_HideBackground_fp(text: &mut DevText) {
    melee_core::ct!("DevText_HideBackground_fp");
    text.flags |= 1 << 6;
}

/// textlib.c `DevText_ShowText_fp`.
pub fn DevText_ShowText_fp(text: &mut DevText) {
    melee_core::ct!("DevText_ShowText_fp");
    text.flags &= !(1 << 7);
}

/// textlib.c `DevText_HideText_fp`.
pub fn DevText_HideText_fp(text: &mut DevText) {
    melee_core::ct!("DevText_HideText_fp");
    text.flags |= 1 << 7;
}

/// textlib.c `DevText_SetScale_fp`.
pub fn DevText_SetScale_fp(text: &mut DevText, x: f32, y: f32) {
    melee_core::ct!("DevText_SetScale_fp");
    text.scale_x = x;
    text.scale_y = y;
}

/// textlib.c `DevText_SetXY_fp`: at least 8.
pub fn DevText_SetXY_fp(text: &mut DevText, x: i32, y: i32) {
    melee_core::ct!("DevText_SetXY_fp");
    text.x = x.max(8) as i16;
    text.y = y.max(8) as i16;
}

/// textlib.c `DevText_SetBGColor_fp`: returns the old colour.
pub fn DevText_SetBGColor_fp(text: &mut DevText, color: _GXColor) -> _GXColor {
    melee_core::ct!("DevText_SetBGColor_fp");
    core::mem::replace(&mut text.bg_color, color)
}

// ---- folded from hud_r7.rs (consolidation steps 21/22) ----
/// textlib.c `DevText_Erase_fp`: zero the 2*w*h-byte character buffer.
///
/// # Safety
/// `text.buf` points at least 2*w*h writable bytes.
pub unsafe fn DevText_Erase_fp(text: &mut DevText_hud_r7) {
    melee_core::ct!("DevText_Erase_fp");
    let n = 2 * text.w as i32 * text.h as i32;
    memzero(core::slice::from_raw_parts_mut(text.buf as *mut u8, n as usize), n as isize);
}
