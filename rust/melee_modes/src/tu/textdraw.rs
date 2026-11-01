//! textdraw.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::DevText;

// ---- folded from hud_r5.rs (consolidation steps 21/22) ----
/// textdraw.c `DevText_StrLen_fp` (NULL -> 0).
pub fn DevText_StrLen_fp(s: Option<&[u8]>) -> i32 {
    melee_core::ct!("DevText_StrLen_fp");
    match s {
        Some(s) => {
            let mut n = 0i32;
            let mut i = 0;
            while s[i] != 0 {
                n = n.wrapping_add(1);
                i += 1;
            }
            n
        }
        None => 0,
    }
}

/// textdraw.c `DevText_NumToStr_fp`: decimal digits, '-' first for a negative; `-INT_MIN` wraps
/// (-fwrapv), so INT_MIN gives "-" followed by the C's negative-remainder characters.
pub fn DevText_NumToStr_fp(mut num: i32, s: &mut [u8]) {
    melee_core::ct!("DevText_NumToStr_fp");
    let mut st = 0usize;
    if num < 0 {
        s[0] = b'-';
        num = num.wrapping_neg();
        st = 1;
    }
    let mut end = st;
    let mut len = 0i32;
    loop {
        s[end] = ((num % 10) as u8).wrapping_add(b'0');
        num /= 10;
        end += 1;
        len += 1;
        if num == 0 {
            break;
        }
    }
    s[end] = 0;
    end -= 1;
    len >>= 1;
    let mut i = 0;
    while i != len {
        s.swap(st, end);
        end = end.wrapping_sub(1);
        st += 1;
        i += 1;
    }
}

/// textdraw.c `DevText_AddToList_fp`: insert `text` into the id-sorted list after the last entry
/// with id <= text's (the C never sets `next->prev`).
pub fn DevText_AddToList_fp(list: &mut *mut DevText, text: *mut DevText) {
    melee_core::ct!("DevText_AddToList_fp");
    // SAFETY: the list's nodes and `text` are live DevTexts, as in the C.
    unsafe {
        if !(*list).is_null() {
            let mut next = *list;
            let mut prev: *mut DevText = core::ptr::null_mut();
            while !next.is_null() {
                if (*text).id >= (*next).id {
                    prev = next;
                    next = (*next).next;
                } else {
                    break;
                }
            }
            (*text).next = next;
            if !prev.is_null() {
                (*prev).next = text;
                (*text).prev = prev;
            } else {
                (*text).prev = core::ptr::null_mut();
                *list = text;
            }
        } else {
            *list = text;
        }
    }
}
