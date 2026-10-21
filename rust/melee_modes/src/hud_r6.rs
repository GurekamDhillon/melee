//! Round 6 (night 3 round 2): if leaf functions and callers of round 5 (textlib_1.c,
//! soundtest.c), verified bit for bit against the C by `tests/oracle_r6.rs`.
use crate::hud_r5::DevText_StrLen;
use melee_types::un_80304138_objalloc_t_x8 as Entry;

/// A NUL-terminated C string as the slice `DevText_StrLen` takes (NULL -> None).
///
/// # Safety
/// `p` is NULL or points at a NUL-terminated string.
unsafe fn cstr<'a>(p: *const i8) -> Option<&'a [u8]> {
    if p.is_null() {
        return None;
    }
    let mut n = 0usize;
    while *p.add(n) != 0 {
        n += 1;
    }
    Some(core::slice::from_raw_parts(p as *const u8, n + 1))
}

/// textlib_1.c `un_80302EA4`: the text width a menu of entries (ended by x0 == 9) needs:
/// label column + value column, or the widest header line (x0 0/1), plus one.
///
/// # Safety
/// `arg0` points at entries ending with x0 == 9; every x8, and xC[0..(int) x18] of each x0 == 2
/// entry, is NULL or a NUL-terminated string.
pub unsafe fn un_80302EA4(mut arg0: *const Entry) -> i32 {
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

/// textlib_1.c `un_80302FFC_maxlen` (static inline): widest label (x0 not 0/1) plus one, at least 1.
///
/// # Safety
/// As `un_80302EA4` (only x8 is read).
pub unsafe fn un_80302FFC_maxlen(mut thing: *const Entry) -> i32 {
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

/// soundtest.c `un_802FFCD0`: fill `arr[0..count]` with `arr[count]`.
pub fn un_802FFCD0(count: i32, arr: &mut [i32]) {
    let mut i = 0;
    while i < count {
        arr[i as usize] = arr[count as usize];
        i += 1;
    }
}
