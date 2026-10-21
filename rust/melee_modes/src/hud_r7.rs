//! Round 7 (night 3 round 3): textlib.c caller of round 6's `memzero`, verified bit for bit
//! against the C by `tests/oracle_r7.rs`.
use crate::lb_r6::memzero;
use melee_types::DevText;

/// textlib.c `DevText_Erase`: zero the 2*w*h-byte character buffer.
///
/// # Safety
/// `text.buf` points at least 2*w*h writable bytes.
pub unsafe fn DevText_Erase(text: &mut DevText) {
    let n = 2 * text.w as i32 * text.h as i32;
    memzero(core::slice::from_raw_parts_mut(text.buf as *mut u8, n as usize), n as isize);
}
