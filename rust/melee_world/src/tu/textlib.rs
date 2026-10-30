//! Port of src/melee/if/textlib.c: the DevText box API (create from the pool, cursor, colours,
//! scrolling `Print`). Callees of unported TUs go through `ext`. Checked against the C (whole TU,
//! shipping contraction) by tests/tu_textlib.rs.
//!
//! * `DevText_Printf` is C-variadic; stable Rust cannot define one, so it takes its variadic
//!   arguments as a slice of [`PrintfArg`] and formats them with [`c_format`] (the subset of
//!   printf the game uses: flags `-+ #0`, width, precision, `d i u x X o c s f % `), truncating to
//!   the C buffer's 63 characters like `vsnprintf(str, 64, ..)`.
//! * The two bits of a glyph's second byte are `DevTextGlyph { u8 color:2; u8 unk:6 }`; the
//!   GameCube build (and this port) keeps `color` in the TOP two bits, as `DevText_Draw` reads it.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ptr::null_mut;
use melee_types::*;

use super::textdraw::{devtext_drawlist, devtext_poolhead};
use super::textlib_1::un_804D6E44;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { panic!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn DevText_NumToStr(a0: i32, a1: *mut i8) = |a0: i32, a1: *mut i8| { unsafe { let _ = crate::tu::textdraw::DevText_NumToStr(a0 as _, a1 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// One variadic argument and the printf formatter moved down to `melee_runtime::cformat`
/// (OSReport, in another crate, formats with it too).
pub use melee_runtime::cformat::{c_format, PrintfArg};

/// `memzero(ptr, size)`: zero fill (the size is an `int`; the C converts it to `size_t`).
#[inline]
unsafe fn memzero(p: *mut u8, n: i32) {
    core::ptr::write_bytes(p, 0, n as u32 as usize);
}

/// `static inline DevText* find_by_id(char id)`: the draw-list entry with that id.
pub unsafe fn find_by_id(id: i8) -> *mut DevText {
    melee_core::ct!("find_by_id");
    let mut text = devtext_drawlist;
    while !text.is_null() {
        if (*text).id == id {
            return text;
        }
        text = (*text).next;
    }
    null_mut()
}

/// `DevText* DevText_Create(char id, int x, int y, int w, int h, char* buf)`: takes a box off the
/// free list (NULL when `id` is already listed); a failed allocation is the C `HSD_ASSERTREPORT`.
pub unsafe extern "C" fn DevText_Create(id: i8, x: i32, y: i32, w: i32, h: i32, buf: *mut i8) -> *mut DevText {
    melee_core::ct!("DevText_Create");
    let bg = _GXColor { r: 0x60, g: 0xD0, b: 0xB0, a: 0x70 };
    if !find_by_id(id).is_null() {
        return null_mut();
    }
    let mut text = devtext_poolhead[0];
    if !text.is_null() {
        devtext_poolhead[0] = (*text).next;
    } else {
        text = null_mut();
    }
    if text.is_null() {
        panic!("TW : Screen alloc Fail (textlib.c:309)");
    }
    let white = _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };
    let red = _GXColor { r: 0xFF, g: 0x80, b: 0x80, a: 0xFF };
    let green = _GXColor { r: 0x80, g: 0xFF, b: 0x80, a: 0xFF };
    let blue = _GXColor { r: 0x80, g: 0x80, b: 0xFF, a: 0xFF };
    let t = &mut *text;
    t.x = x as i16;
    t.y = y as i16;
    t.w = w as u8;
    t.h = h as u8;
    t.cursor_x = 0;
    t.cursor_y = 0;
    t.scale_x = 10.0;
    t.scale_y = 16.0;
    t.bg_color = bg;
    t.text_colors[0] = white;
    t.text_colors[1] = red;
    t.text_colors[2] = green;
    t.text_colors[3] = blue;
    t.id = id;
    t.line_width = 10;
    t.flags = DEVTEXT_FLAG_SHOWCURSOR as u8;
    t.set_unk(0);
    t.set_current_color(0);
    t.prev = null_mut();
    t.next = null_mut();
    t.buf = buf;
    memzero(buf as *mut u8, h.wrapping_mul(w.wrapping_mul(2)));
    text
}

/// `void DevText_EraseFirstLine(DevText*)`: scrolls the buffer up one line and clears the last.
pub unsafe extern "C" fn DevText_EraseFirstLine(text: *mut DevText) {
    melee_core::ct!("DevText_EraseFirstLine");
    let t = &*text;
    let mut start_of_line = t.buf;
    let line_length = t.w as i32 * 2;
    let mut line_number = 0;
    while line_number < t.h as i32 - 1 {
        core::ptr::copy(start_of_line.offset(line_length as isize), start_of_line, line_length as usize);
        start_of_line = start_of_line.offset(line_length as isize);
        line_number += 1;
    }
    memzero(start_of_line as *mut u8, line_length);
}

/// `static inline int DevText_Clamp(int val, int max)`
#[inline]
pub fn DevText_Clamp(val: i32, max: i32) -> i32 {
    melee_core::ct!("DevText_Clamp");
    if max <= val {
        max - 1
    } else if val < 0 {
        0
    } else {
        val
    }
}

/// `void DevText_SetCursorXY(DevText*, int, int)`: clamped to the box.
pub unsafe extern "C" fn DevText_SetCursorXY(text: *mut DevText, mut x: i32, mut y: i32) {
    melee_core::ct!("DevText_SetCursorXY");
    let t = &mut *text;
    if t.w as i32 <= x {
        x = t.w as i32 - 1;
    } else if x < 0 {
        x = 0;
    }
    t.cursor_x = x as u8;
    if t.h as i32 <= y {
        y = t.h as i32 - 1;
    } else if y < 0 {
        y = 0;
    }
    t.cursor_y = y as u8;
}

/// `void DevText_SetCursorX(DevText*, int)`
pub unsafe extern "C" fn DevText_SetCursorX(text: *mut DevText, x: i32) {
    melee_core::ct!("DevText_SetCursorX");
    (*text).cursor_x = DevText_Clamp(x, (*text).w as i32) as u8;
}

/// `void DevText_HideCursor(DevText*)`
pub unsafe extern "C" fn DevText_HideCursor(text: *mut DevText) {
    melee_core::ct!("DevText_HideCursor");
    (*text).flags &= !(1 << 4);
}

/// `void DevText_80302AC0(DevText*)`
pub unsafe extern "C" fn DevText_80302AC0(text: *mut DevText) {
    melee_core::ct!("DevText_80302AC0");
    (*text).flags |= 1 << 5;
}

/// `void DevText_ShowBackground(DevText*)`
pub unsafe extern "C" fn DevText_ShowBackground(text: *mut DevText) {
    melee_core::ct!("DevText_ShowBackground");
    (*text).flags &= !(1 << 6);
}

/// `void DevText_HideBackground(DevText*)`
pub unsafe extern "C" fn DevText_HideBackground(text: *mut DevText) {
    melee_core::ct!("DevText_HideBackground");
    (*text).flags |= 1 << 6;
}

/// `void DevText_ShowText(DevText*)`
pub unsafe extern "C" fn DevText_ShowText(text: *mut DevText) {
    melee_core::ct!("DevText_ShowText");
    (*text).flags &= !(1 << 7);
}

/// `void DevText_HideText(DevText*)`
pub unsafe extern "C" fn DevText_HideText(text: *mut DevText) {
    melee_core::ct!("DevText_HideText");
    (*text).flags |= 1 << 7;
}

/// `void DevText_SetScale(DevText*, f32, f32)`
pub unsafe extern "C" fn DevText_SetScale(text: *mut DevText, x: f32, y: f32) {
    melee_core::ct!("DevText_SetScale");
    (*text).scale_x = x;
    (*text).scale_y = y;
}

/// `void DevText_SetXY(DevText*, int, int)`: never closer than 8 to the screen's top left.
pub unsafe extern "C" fn DevText_SetXY(text: *mut DevText, mut x: i32, mut y: i32) {
    melee_core::ct!("DevText_SetXY");
    if x < 8 {
        x = 8;
    }
    if y < 8 {
        y = 8;
    }
    (*text).x = x as i16;
    (*text).y = y as i16;
}

/// `u8 DevText_StoreColorIndex(DevText*, u8)`: returns the previous index.
pub unsafe extern "C" fn DevText_StoreColorIndex(text: *mut DevText, index: u8) -> u8 {
    melee_core::ct!("DevText_StoreColorIndex");
    let old = (*text).current_color();
    (*text).set_current_color(index);
    old
}

/// `GXColor DevText_SetTextColor(DevText*, GXColor)`: sets the current index's colour.
pub unsafe extern "C" fn DevText_SetTextColor(text: *mut DevText, color: _GXColor) -> _GXColor {
    melee_core::ct!("DevText_SetTextColor");
    let index = (*text).current_color() as usize;
    let old = (*text).text_colors[index];
    (*text).text_colors[index] = color;
    old
}

/// `GXColor DevText_SetBGColor(DevText*, GXColor)`
pub unsafe extern "C" fn DevText_SetBGColor(text: *mut DevText, color: _GXColor) -> _GXColor {
    melee_core::ct!("DevText_SetBGColor");
    let old = (*text).bg_color;
    (*text).bg_color = color;
    old
}

/// `void DevText_Erase(DevText*)`
pub unsafe extern "C" fn DevText_Erase(text: *mut DevText) {
    melee_core::ct!("DevText_Erase");
    let t = &*text;
    memzero(t.buf as *mut u8, 2i32.wrapping_mul(t.w as i32).wrapping_mul(t.h as i32));
}

/// `static inline void DevText_AdvanceLine(DevText*)`
pub unsafe fn DevText_AdvanceLine(text: *mut DevText) {
    melee_core::ct!("DevText_AdvanceLine");
    (*text).cursor_x = 0;
    if ((*text).cursor_y as i32) < (*text).h as i32 - 1 {
        (*text).cursor_y += 1;
    } else {
        DevText_EraseFirstLine(text);
    }
}

/// `void DevText_Print(DevText*, char*)`: writes each character with the current colour, wrapping
/// at the right edge (unless NOWRAP) and on `'\n'`.
pub unsafe extern "C" fn DevText_Print(text: *mut DevText, str: *mut i8) {
    melee_core::ct!("DevText_Print");
    if !str.is_null() {
        let mut cur = str;
        while *cur != 0 {
            if *cur != b'\n' as i8 {
                let t = &mut *text;
                let index = t.cursor_x as i32 + t.cursor_y as i32 * t.w as i32;
                let g = t.buf.offset(index as isize * 2) as *mut u8;
                *g = *cur as u8;
                // DevTextGlyph.color is the top two bits of the second byte
                *g.add(1) = (*g.add(1) & 0x3F) | (t.current_color() << 6);
                if (t.cursor_x as i32) < t.w as i32 - 1 {
                    t.cursor_x += 1;
                } else if (t.flags as i32 & DEVTEXT_FLAG_NOWRAP) == 0 {
                    DevText_AdvanceLine(text);
                }
            } else {
                DevText_AdvanceLine(text);
            }
            cur = cur.add(1);
        }
    }
}

/// `void DevText_PrintInt(DevText*, int)`
pub unsafe extern "C" fn DevText_PrintInt(text: *mut DevText, num: i32) {
    melee_core::ct!("DevText_PrintInt");
    let mut str = [0i8; 16];
    ext::DevText_NumToStr(num, str.as_mut_ptr());
    DevText_Print(text, str.as_mut_ptr());
}

/// `void DevText_Printf(DevText*, char* format, ...)` with the variadic arguments as a slice.
pub unsafe fn DevText_Printf(text: *mut DevText, format: *const i8, args: &[PrintfArg]) {
    melee_core::ct!("DevText_Printf");
    let mut str = [0i8; 64];
    let f = c_format(format, args);
    let n = f.len().min(63);
    for i in 0..n {
        str[i] = f[i] as i8;
    }
    DevText_Print(text, str.as_mut_ptr());
}

/// `struct un_80304138_objalloc_t* un_80302DF0(void)`: the current menu.
pub unsafe extern "C" fn un_80302DF0() -> *mut un_80304138_objalloc_t {
    melee_core::ct!("un_80302DF0");
    un_804D6E44
}

/// `void un_80302DF8(struct un_80304138_objalloc_t*, soundtest_callback)`
pub unsafe extern "C" fn un_80302DF8(arg0: *mut un_80304138_objalloc_t, arg1: soundtest_callback) {
    melee_core::ct!("un_80302DF8");
    (*arg0).xC = arg1;
}
