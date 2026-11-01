//! textlib_1.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::hud_r5::DevText_StrLen;
#[allow(unused_imports)]
use melee_types::un_80304138_objalloc_t_x8 as Entry;
#[allow(unused_imports)]
use crate::hud_r6::{cstr};
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use crate::GXColor;

// ---- folded from hud.rs (consolidation steps 21/22) ----
/// textlib_1.c `adjust_fp`: halve r, g, b (int arithmetic, truncating), keep a.
pub fn adjust_fp(mut c: GXColor) -> GXColor {
    melee_core::ct!("adjust_fp");
    c.r = (c.r as i32 * 50 / 100) as u8;
    c.g = (c.g as i32 * 50 / 100) as u8;
    c.b = (c.b as i32 * 50 / 100) as u8;
    c
}

// ---- folded from hud_ptr.rs (consolidation steps 21/22) ----
/// textlib_1.c `un_80303FD4_first_fp`: index of the first entry with x0 != 0 before the 9
/// terminator, else 0.
pub fn un_80303FD4_first_fp(p: &[mt::un_80304138_objalloc_t_x8]) -> i32 {
    melee_core::ct!("un_80303FD4_first_fp");
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

/// textlib_1.c `un_80303FD4_count_fp`: entries before the 9 terminator.
pub fn un_80303FD4_count_fp(p: &[mt::un_80304138_objalloc_t_x8]) -> i32 {
    melee_core::ct!("un_80303FD4_count_fp");
    let mut n = 0usize;
    while p[n].x0 != 9 {
        n += 1;
    }
    n as i32
}

/// textlib_1.c `un_80304334_fp`.
pub fn un_80304334_fp(arg0: &mut mt::un_80304138_objalloc_t) {
    melee_core::ct!("un_80304334_fp");
    arg0.x1 |= 1 << 7;
}

// ---- folded from hud_r6.rs (consolidation steps 21/22) ----
/// textlib_1.c `un_80302EA4_fp`: the text width a menu of entries (ended by x0 == 9) needs:
/// label column + value column, or the widest header line (x0 0/1), plus one.
///
/// # Safety
/// `arg0` points at entries ending with x0 == 9; every x8, and xC[0..(int) x18] of each x0 == 2
/// entry, is NULL or a NUL-terminated string.
pub unsafe fn un_80302EA4_fp(mut arg0: *const Entry) -> i32 {
    melee_core::ct!("un_80302EA4_fp");
    let (mut x, mut y, mut z) = (1i32, 1i32, 1i32);
    while (*arg0).x0 != 9 {
        let e = &*arg0;
        let len = DevText_StrLen(cstr(e.x8)).wrapping_add(1);
        if (e.x0 as u32) <= 1 {
            if len > z {
                z = len;
            }
        } else if len > x {
            x = len;
        }
        if (e.x0 == 3 || e.x0 == 8) && y < 3 {
            y = 3;
        }
        if e.x0 == 4 && y < 10 {
            y = 10;
        }
        if e.x0 == 5 && y < 2 {
            y = 2;
        }
        if e.x0 == 6 && y < 4 {
            y = 4;
        }
        if e.x0 == 7 && y < 8 {
            y = 8;
        }
        if e.x0 == 2 {
            // (int) x18: float -> int truncation, saturating like gwtool's lowering
            let n = e.x18 as i32;
            let mut i = 0;
            while i < n {
                let len = DevText_StrLen(cstr(*e.xC.add(i as usize)));
                if len > y {
                    y = len;
                }
                i += 1;
            }
        }
        arg0 = arg0.add(1);
    }
    x = x.wrapping_add(y);
    if x > z { x.wrapping_add(1) } else { z.wrapping_add(1) }
}

/// textlib_1.c `un_80302FFC_maxlen_fp` (static inline): widest label (x0 not 0/1) plus one, at least 1.
///
/// # Safety
/// As `un_80302EA4_fp` (only x8 is read).
pub unsafe fn un_80302FFC_maxlen_fp(mut thing: *const Entry) -> i32 {
    melee_core::ct!("un_80302FFC_maxlen_fp");
    let mut cursor_x = 1i32;
    while (*thing).x0 != 9 {
        let e = &*thing;
        if e.x0 != 0 && e.x0 != 1 {
            let len = DevText_StrLen(cstr(e.x8));
            if len.wrapping_add(1) > cursor_x {
                cursor_x = len.wrapping_add(1);
            }
        }
        thing = thing.add(1);
    }
    cursor_x
}
