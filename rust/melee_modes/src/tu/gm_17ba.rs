//! Native port of src/melee/gm/gm_17BA.c: result-screen stat readers and the per-target name
//! strings. Verified against the C by tests/tu_gm_17ba.rs.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::gm_17ad::gmClampResultStat;
use crate::ext_fns;
use core::ffi::c_char;
use melee_types::*;

/// Callees in TUs that are not ported yet.
pub mod ext {
    use super::*;
    ext_fns! {
        fn GetPersistentNameData(a0: i32) -> *mut NameTagData;
        fn fn_80174274() -> *mut MatchEnd;
        fn fn_8017AD04(a0: i32, a1: i32) -> i32 = crate::gm::fn_8017AD04;
        fn lbLang_IsSavedLanguageUS() -> i32;
    }
}

const Gm_PKind_NA: u8 = 3;

pub static mut lbl_803D79D0: [*const c_char; 4] = [
    "１Ｐ\0".as_ptr() as *const c_char,
    "２Ｐ\0".as_ptr() as *const c_char,
    "３Ｐ\0".as_ptr() as *const c_char,
    "４Ｐ\0".as_ptr() as *const c_char,
];

pub static mut lbl_803D79E0: [*const c_char; 4] = [
    "Ｐ１\0".as_ptr() as *const c_char,
    "Ｐ２\0".as_ptr() as *const c_char,
    "Ｐ３\0".as_ptr() as *const c_char,
    "Ｐ４\0".as_ptr() as *const c_char,
];

pub unsafe fn fn_8017BACC(arg0: i32) -> i32 {
    let m = ext::fn_80174274();
    gmClampResultStat((*m).player_standings[arg0 as usize].x9C)
}

pub unsafe fn fn_8017BB30(arg0: i32) -> i32 {
    let m = ext::fn_80174274();
    gmClampResultStat((*m).player_standings[arg0 as usize].xA0 as i32)
}

/// `fn_8017BB94_inline(arg0, target)`: the name of player `target` for the kill list of `arg0`.
pub unsafe fn fn_8017BB94_inline(arg0: i32, target: usize) -> *const c_char {
    let temp_r3_2 = ext::fn_80174274();
    let var_r3: i32 = if arg0 == target as i32 {
        -1
    } else if (*temp_r3_2).player_standings[target].pkind != Gm_PKind_NA {
        ext::fn_8017AD04((*temp_r3_2).player_standings[arg0 as usize].kills[target] as i32, 999999)
    } else {
        -1
    };
    if var_r3 != -1 {
        let temp_r3 = (*ext::fn_80174274()).player_standings[target].x4 as u32;
        if temp_r3 == 0x78 {
            if ext::lbLang_IsSavedLanguageUS() != 0 {
                return lbl_803D79E0[target];
            } else {
                return lbl_803D79D0[target];
            }
        } else {
            let tmp = ext::GetPersistentNameData(temp_r3 as i32);
            return (*tmp).namedata.as_ptr();
        }
    }
    core::ptr::null()
}

pub unsafe fn fn_8017BB94(arg0: i32) -> *const c_char {
    fn_8017BB94_inline(arg0, 0)
}

pub unsafe fn fn_8017BC50(arg0: i32) -> *const c_char {
    fn_8017BB94_inline(arg0, 1)
}

pub unsafe fn fn_8017BD0C(arg0: i32) -> *const c_char {
    fn_8017BB94_inline(arg0, 2)
}

pub unsafe fn fn_8017BDC8(arg0: i32) -> *const c_char {
    fn_8017BB94_inline(arg0, 3)
}
