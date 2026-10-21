//! Round 5: if/ty leaf functions over nested pointers (textdraw.c, ifstock.c, toy.c), verified
//! bit for bit against the C by `tests/oracle_r5.rs`.
use melee_types as mt;
use mt::{DevText, HSD_MObj, TyDisplayData, _GXColor};

/// textdraw.c `DevText_StrLen` (NULL -> 0).
pub fn DevText_StrLen(s: Option<&[u8]>) -> i32 {
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

/// textdraw.c `DevText_NumToStr`: decimal digits, '-' first for a negative; `-INT_MIN` wraps
/// (-fwrapv), so INT_MIN gives "-" followed by the C's negative-remainder characters.
pub fn DevText_NumToStr(mut num: i32, s: &mut [u8]) {
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

/// textdraw.c `DevText_AddToList`: insert `text` into the id-sorted list after the last entry
/// with id <= text's (the C never sets `next->prev`).
pub fn DevText_AddToList(list: &mut *mut DevText, text: *mut DevText) {
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

/// ifstock.c `copyColor`: the material's diffuse r, g, b from `arg` (alpha kept).
pub fn copyColor(q: &mut HSD_MObj, arg: &_GXColor) {
    // SAFETY: the C dereferences `q->mat` unchecked.
    let m = unsafe { &mut *q.mat };
    m.diffuse.r = arg.r;
    m.diffuse.g = arg.g;
    m.diffuse.b = arg.b;
}

/// toy.c `_Toy_8030FE48_link_entries`: link the first `entry_count` entries into a ring.
pub fn _Toy_8030FE48_link_entries(data: &mut TyDisplayData, entry_count: i32) {
    let base = data.entries.as_mut_ptr();
    let last = entry_count.wrapping_sub(1);
    let last_entry = base.wrapping_offset(last as isize);
    for i in 0..entry_count {
        let e = &mut data.entries[i as usize];
        e.prev = if i == 0 { last_entry } else { base.wrapping_offset(i as isize - 1) };
        e.next = if i == last { base } else { base.wrapping_offset(i as isize + 1) };
    }
}
