
//! Round 7 (night 3 round 3): gm callers of round 6's `memzero`, verified bit for bit against
//! the C by `tests/oracle_r7.rs`.
use crate::gm_r4::fn_801771C0;

use crate::lb_r6::memzero;

use melee_types::{MatchEnd, ResultsData, ResultsMatchInfo};

pub(crate) fn bytes_of<T>(x: &mut T) -> &mut [u8] {
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

pub use crate::tu::gm_1601::fn_80168A6C_fp as fn_80168A6C;
pub use crate::tu::gm_16A2::fn_80169574_fp as fn_80169574;
pub use crate::tu::gmresult::gmResult_PcWinner;
pub use crate::tu::gmresultplayer::gm_80177724_fp as gm_80177724;
pub use crate::tu::gm_17c0::gm_8017C984_fp as gm_8017C984;
