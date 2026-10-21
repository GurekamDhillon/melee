//! Round 5: mn leaf functions over nested pointers, verified by `tests/oracle_r5.rs`.

/// mnnamenew.c `GetAutoNameCharacter`: `(*names)[char_idx]`.
pub fn GetAutoNameCharacter(names: &*const u8, char_idx: i32) -> u8 {
    // SAFETY: the C indexes the name string unchecked.
    unsafe { *names.offset(char_idx as isize) }
}
