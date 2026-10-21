//! Round 3: `src/melee/if/` and `src/melee/ty/` leaf functions that take pointers to game
//! structs. Conventions and verification as in `gm_ptr.rs` (`tests/oracle_ptr.rs`).
#![allow(non_snake_case)]

use melee_types as mt;
use mt::{DevText, HSD_GObj, _GXColor};

/// ifprize.c `un_802FE918_update_x3`. `new_x3` is uninitialised in the C when no `k`
/// matches; the shipping build zero-initialises locals (`-ftrivial-auto-var-init=zero`).
pub fn un_802FE918_update_x3(x3_ptr: &mut u8, r: &mut i32) {
    let old_x3 = *x3_ptr as i32;
    let mut new_x3 = 0i32;
    for k in 0..3 {
        if k == old_x3 {
            *r = r.wrapping_add(1);
        } else if *r == k {
            new_x3 = *r;
            break;
        }
    }
    *x3_ptr = new_x3 as u8;
}

/// ifstatus.c `ifStatus_GetStoredTranslationX`.
pub fn ifStatus_GetStoredTranslationX(state: &mt::IfDamageState, index: i32) -> f32 {
    state.translation_x[index as usize]
}

/// ifstock.c `ifStock_802F8298_step_anim`.
pub fn ifStock_802F8298_step_anim(anim: &mut u8) {
    if *anim < 10 {
        *anim += 1;
    }
}

/// ifstock.c `ifStock_ClearGObjSlots`.
pub fn ifStock_ClearGObjSlots(slots: &mut [*mut HSD_GObj], count: i32) {
    for s in &mut slots[..count.max(0) as usize] {
        *s = core::ptr::null_mut();
    }
}

/// soundtest.c `un_802FF884`.
pub fn un_802FF884(_arg0: *mut i8) -> bool {
    false
}

/// textlib.c `DevText_SetCursorXY`: clamp into the text grid.
pub fn DevText_SetCursorXY(text: &mut DevText, mut x: i32, mut y: i32) {
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

/// textlib.c `DevText_HideCursor`.
pub fn DevText_HideCursor(text: &mut DevText) {
    text.flags &= !(1 << 4);
}

/// textlib.c `DevText_80302AC0`.
pub fn DevText_80302AC0(text: &mut DevText) {
    text.flags |= 1 << 5;
}

/// textlib.c `DevText_ShowBackground`.
pub fn DevText_ShowBackground(text: &mut DevText) {
    text.flags &= !(1 << 6);
}

/// textlib.c `DevText_HideBackground`.
pub fn DevText_HideBackground(text: &mut DevText) {
    text.flags |= 1 << 6;
}

/// textlib.c `DevText_ShowText`.
pub fn DevText_ShowText(text: &mut DevText) {
    text.flags &= !(1 << 7);
}

/// textlib.c `DevText_HideText`.
pub fn DevText_HideText(text: &mut DevText) {
    text.flags |= 1 << 7;
}

/// textlib.c `DevText_SetScale`.
pub fn DevText_SetScale(text: &mut DevText, x: f32, y: f32) {
    text.scale_x = x;
    text.scale_y = y;
}

/// textlib.c `DevText_SetXY`: at least 8.
pub fn DevText_SetXY(text: &mut DevText, x: i32, y: i32) {
    text.x = x.max(8) as i16;
    text.y = y.max(8) as i16;
}

/// textlib.c `DevText_SetBGColor`: returns the old colour.
pub fn DevText_SetBGColor(text: &mut DevText, color: _GXColor) -> _GXColor {
    core::mem::replace(&mut text.bg_color, color)
}

/// textlib.c `un_80302DF8`.
pub fn un_80302DF8(arg0: &mut mt::un_80304138_objalloc_t, arg1: mt::soundtest_callback) {
    arg0.xC = arg1;
}

/// textlib_1.c `un_80303FD4_first`: index of the first entry with x0 != 0 before the 9
/// terminator, else 0.
pub fn un_80303FD4_first(p: &[mt::un_80304138_objalloc_t_x8]) -> i32 {
    let mut i = 0usize;
    loop {
        let v = p[i].x0;
        if v == 9 {
            return 0;
        }
        if v != 0 {
            return i as i32;
        }
        i += 1;
    }
}

/// textlib_1.c `un_80303FD4_count`: entries before the 9 terminator.
pub fn un_80303FD4_count(p: &[mt::un_80304138_objalloc_t_x8]) -> i32 {
    let mut n = 0usize;
    while p[n].x0 != 9 {
        n += 1;
    }
    n as i32
}

/// textlib_1.c `un_80304334`.
pub fn un_80304334(arg0: &mut mt::un_80304138_objalloc_t) {
    arg0.x1 |= 1 << 7;
}

/// tydisplay.c `tyDisplay_SetGridSize`: grow x75/x76 (columns capped at 0x14) until they
/// cover `x08` trophies; kinds 0 and 1 then add a border row and column.
pub fn tyDisplay_SetGridSize(cfg: &mut mt::TyDspConfig, grid: &mt::TyDspGrid) {
    let border = match grid.x00 {
        0 | 1 => true,
        2 | 3 => false,
        _ => return,
    };
    let count = cfg.x08;
    let mut toggle = 0;
    cfg.x75 = 1;
    cfg.x76 = 1;
    while (cfg.x75 as i8 as i32) * (cfg.x76 as i8 as i32) < count {
        toggle ^= 1;
        if toggle != 0 && (cfg.x75 as i8) < 0x14 {
            cfg.x75 = cfg.x75.wrapping_add(1);
        } else {
            cfg.x76 = cfg.x76.wrapping_add(1);
        }
    }
    if border {
        cfg.x75 = cfg.x75.wrapping_add(1);
        cfg.x76 = cfg.x76.wrapping_add(1);
    }
}

/// tyfigupon.c `tyFigupon_StoreDigits`: decimal digits, least significant first.
pub fn tyFigupon_StoreDigits(digits: &mut [i32], mut value: i32) {
    let mut i = 0;
    loop {
        digits[i] = value % 10;
        i += 1;
        value /= 10;
        if value <= 0 {
            break;
        }
    }
}

/// tyfigupon.c `tyFigupon_GetBetCount`.
pub fn tyFigupon_GetBetCount(ef4: &mt::un_804D6EF4_t) -> i32 {
    ef4.x5E as i32
}
