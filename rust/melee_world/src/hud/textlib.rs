//! Night 3 round 6: the DevText buffer/state functions of src/melee/if/textlib.c, plus
//! textlib_1.c's `adjust`. Checked against the C by `tests/textlib_r6_vs_c.rs`
//! (`cref/gen_textlib_r6_ref.sh`). No float arithmetic, so no fused sites.
//!
//! The record is `melee_types::DevText` (GC offsets in its metadata). Its `buf` pointer is not
//! followed here: the glyph buffer is passed as a slice (`2 * w * h` bytes, two per cell). A
//! cell is the C's `DevTextGlyph { u8 chr; u8 color:2; u8 unk:6; }` as the native (PC,
//! little-endian clang) build lays it out: byte 0 the character, byte 1 bits 0-1 the colour,
//! bits 2-7 left untouched. Out-of-range cursor indices panic (the C is UB there).
use melee_types::{DevText, GXColor};

use super::leaves::{devtext_clamp, devtext_num_to_str};

pub const DEVTEXT_FLAG_HIDETEXT: u8 = 0x80;
pub const DEVTEXT_FLAG_HIDEBACKGROUND: u8 = 0x40;
pub const DEVTEXT_FLAG_NOWRAP: u8 = 0x20;
pub const DEVTEXT_FLAG_SHOWCURSOR: u8 = 0x10;

/// DevText_EraseFirstLine: scrolls the buffer up one line and clears the last.
/// With `h == 0` the C clears the first line (loop bound `h - 1` is -1).
pub fn devtext_erase_first_line(text: &DevText, buf: &mut [u8]) {
    let line_length = text.w as i32 * 2;
    let len = line_length as usize;
    let mut start = 0usize;
    let mut line_number = 0i32;
    while line_number < text.h as i32 - 1 {
        buf.copy_within(start + len..start + 2 * len, start);
        start += len;
        line_number += 1;
    }
    buf[start..start + len].fill(0);
}

/// DevText_SetCursorXY. Stores the clamped `int` truncated to u8 (w == 0 gives 255).
pub fn devtext_set_cursor_xy(text: &mut DevText, mut x: i32, mut y: i32) {
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

/// DevText_SetCursorX.
pub fn devtext_set_cursor_x(text: &mut DevText, x: i32) {
    text.cursor_x = devtext_clamp(x, text.w as i32) as u8;
}

/// DevText_HideCursor.
pub fn devtext_hide_cursor(text: &mut DevText) {
    text.flags &= !DEVTEXT_FLAG_SHOWCURSOR;
}

/// DevText_80302AC0 (sets NOWRAP).
pub fn devtext_80302ac0(text: &mut DevText) {
    text.flags |= DEVTEXT_FLAG_NOWRAP;
}

/// DevText_ShowBackground.
pub fn devtext_show_background(text: &mut DevText) {
    text.flags &= !DEVTEXT_FLAG_HIDEBACKGROUND;
}

/// DevText_HideBackground.
pub fn devtext_hide_background(text: &mut DevText) {
    text.flags |= DEVTEXT_FLAG_HIDEBACKGROUND;
}

/// DevText_ShowText.
pub fn devtext_show_text(text: &mut DevText) {
    text.flags &= !DEVTEXT_FLAG_HIDETEXT;
}

/// DevText_HideText.
pub fn devtext_hide_text(text: &mut DevText) {
    text.flags |= DEVTEXT_FLAG_HIDETEXT;
}

/// DevText_SetScale.
pub fn devtext_set_scale(text: &mut DevText, x: f32, y: f32) {
    text.scale_x = x;
    text.scale_y = y;
}

/// DevText_SetXY: both at least 8, stored truncated to s16.
pub fn devtext_set_xy(text: &mut DevText, mut x: i32, mut y: i32) {
    if x < 8 {
        x = 8;
    }
    if y < 8 {
        y = 8;
    }
    text.x = x as i16;
    text.y = y as i16;
}

/// DevText_StoreColorIndex: returns the old 2-bit colour index; stores `index & 3`.
pub fn devtext_store_color_index(text: &mut DevText, index: u8) -> u8 {
    let old = text.current_color();
    text.set_current_color(index);
    old
}

/// DevText_SetTextColor: swaps the colour at the current index.
pub fn devtext_set_text_color(text: &mut DevText, color: GXColor) -> GXColor {
    let index = text.current_color() as usize;
    core::mem::replace(&mut text.text_colors[index], color)
}

/// DevText_SetBGColor.
pub fn devtext_set_bg_color(text: &mut DevText, color: GXColor) -> GXColor {
    core::mem::replace(&mut text.bg_color, color)
}

/// DevText_Erase: clears `2 * w * h` bytes.
pub fn devtext_erase(text: &DevText, buf: &mut [u8]) {
    let n = 2 * text.w as usize * text.h as usize;
    buf[..n].fill(0);
}

/// DevText_AdvanceLine (static inline): next line, or scroll when on the last.
pub fn devtext_advance_line(text: &mut DevText, buf: &mut [u8]) {
    text.cursor_x = 0;
    if (text.cursor_y as i32) < text.h as i32 - 1 {
        text.cursor_y = text.cursor_y.wrapping_add(1);
    } else {
        devtext_erase_first_line(text, buf);
    }
}

/// DevText_Print. `None` is the C's NULL; the string ends at the first NUL (or slice end).
pub fn devtext_print(text: &mut DevText, buf: &mut [u8], s: Option<&[u8]>) {
    let Some(s) = s else { return };
    for &c in s.iter().take_while(|&&c| c != 0) {
        if c != b'\n' {
            let index = (text.cursor_x as i32 + text.cursor_y as i32 * text.w as i32) as usize;
            buf[index * 2] = c;
            buf[index * 2 + 1] = (buf[index * 2 + 1] & !3) | text.current_color();
            if (text.cursor_x as i32) < text.w as i32 - 1 {
                text.cursor_x += 1;
            } else if text.flags & DEVTEXT_FLAG_NOWRAP == 0 {
                devtext_advance_line(text, buf);
            }
        } else {
            devtext_advance_line(text, buf);
        }
    }
}

/// DevText_PrintInt.
pub fn devtext_print_int(text: &mut DevText, buf: &mut [u8], num: i32) {
    let mut s = [0u8; 16];
    devtext_num_to_str(num, &mut s);
    devtext_print(text, buf, Some(&s));
}

/// textlib_1.c `adjust` (static inline): each of r, g, b to `c * 50 / 100`; alpha kept.
pub fn textlib1_adjust(mut c: GXColor) -> GXColor {
    c.r = (c.r as i32 * 50 / 100) as u8;
    c.g = (c.g as i32 * 50 / 100) as u8;
    c.b = (c.b as i32 * 50 / 100) as u8;
    c
}
