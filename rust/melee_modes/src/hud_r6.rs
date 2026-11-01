
//! Round 6 (night 3 round 2): if leaf functions and callers of round 5 (textlib_1.c,
//! soundtest.c), verified bit for bit against the C by `tests/oracle_r6.rs`.
use crate::hud_r5::DevText_StrLen;

use melee_types::un_80304138_objalloc_t_x8 as Entry;

/// A NUL-terminated C string as the slice `DevText_StrLen` takes (NULL -> None).
///
/// # Safety
/// `p` is NULL or points at a NUL-terminated string.
pub(crate) unsafe fn cstr<'a>(p: *const i8) -> Option<&'a [u8]> {
    if p.is_null() {
        return None;
    }
    let mut n = 0usize;
    while *p.add(n) != 0 {
        n += 1;
    }
    Some(core::slice::from_raw_parts(p as *const u8, n + 1))
}

pub use crate::tu::soundtest::un_802FFCD0_fp as un_802FFCD0;
pub use crate::tu::textlib_1::un_80302EA4_fp as un_80302EA4;
pub use crate::tu::textlib_1::un_80302FFC_maxlen_fp as un_80302FFC_maxlen;
