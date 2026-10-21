//! Round 7 (night 3 round 3): gm callers of round 6's `memzero`, verified bit for bit against
//! the C by `tests/oracle_r7.rs`.
use crate::gm_r4::fn_801771C0;
use crate::lb_r6::memzero;
use melee_types::{MatchEnd, ResultsData, ResultsMatchInfo};

fn bytes_of<T>(x: &mut T) -> &mut [u8] {
    // SAFETY: a plain-data record viewed as its bytes, as the C's memzero does.
    unsafe { core::slice::from_raw_parts_mut(x as *mut T as *mut u8, core::mem::size_of::<T>()) }
}

/// The anonymous record gm_1601.c `fn_80168A6C` reads through `arg0` (defined in the function).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Fn80168A6CSrc {
    pub x0: *mut *mut i32,
    pub x4: *mut i32,
    pub x8: i32,
    pub xC: i32,
}

melee_core::gc_layout!(Fn80168A6CSrc, "fn_80168A6C src", size 0x10, align 4, [
    x0 @ 0x0: 4 "s32**",
    x4 @ 0x4: 4 "s32*",
    x8 @ 0x8: 4 "s32",
    xC @ 0xC: 4 "s32",
]);

/// gm_1601.c `fn_80168A6C`: zero eight words of `arg1`, then copy `src->x0[idx][0..4]` (when
/// non-NULL) to words 0..3, `src->x4[0..2]` (when non-NULL) to 5..6, x8 to 4 and xC to 7.
///
/// # Safety
/// `src->x0` points at an array holding index `idx`; non-NULL entries and `x4` point at 4 and 2
/// readable words (as in the C).
pub unsafe fn fn_80168A6C(src: &Fn80168A6CSrc, arg1: &mut [i32; 8], idx: i32) {
    memzero(bytes_of(arg1), 0x20);
    let e = *src.x0.offset(idx as isize);
    if !e.is_null() {
        for k in 0..4 {
            arg1[k] = *e.add(k);
        }
    }
    if !src.x4.is_null() {
        arg1[5] = *src.x4;
        arg1[6] = *src.x4.add(1);
    }
    arg1[4] = src.x8;
    arg1[7] = src.xC;
}

/// gm_16A2.c `fn_80169574`: zero `size` bytes, then `buf[size] = -2`.
pub fn fn_80169574(size: isize, buf: &mut [i8]) {
    // SAFETY: i8 and u8 share layout.
    memzero(unsafe { &mut *(buf as *mut [i8] as *mut [u8]) }, size);
    buf[size as usize] = -2;
}

/// gmresult.c `gmResult_PcWinner`: run `fn_801771C0` on a zeroed scratch ResultsData pointing at
/// `me` and return its x6. The C's scratch is a function static, fully zeroed on every call, so
/// a local is equivalent.
pub fn gmResult_PcWinner(me: &mut MatchEnd) -> i32 {
    // SAFETY: ResultsData is plain data (integers, floats, raw pointers); all-zero is valid.
    let mut scratch: ResultsData = unsafe { core::mem::zeroed() };
    memzero(bytes_of(&mut scratch), core::mem::size_of::<ResultsData>() as isize);
    scratch.x94 = me;
    fn_801771C0(&mut scratch);
    scratch.x6 as i32
}

/// gmresultplayer.c `gm_80177724`: zero the record.
pub fn gm_80177724(arg0: &mut ResultsMatchInfo) {
    let n = core::mem::size_of::<ResultsMatchInfo>() as isize;
    memzero(bytes_of(arg0), n);
}

/// gm_17C0.c `gm_8017C984`: zero 0x74 bytes.
pub fn gm_8017C984(arg0: &mut [u8]) {
    memzero(arg0, 0x74);
}
