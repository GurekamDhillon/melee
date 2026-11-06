//! Native port of src/melee/gm/gm_17AD.c: the result-screen stat readers (clamped copies of
//! fields of `fn_80174274()`'s MatchEnd). Verified against the C by tests/tu_gm_17ad.rs.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gm::{clamp_sym};
use crate::ext_fns;
use melee_types::*;

// Already ported (gm.rs); the C name is the same.
pub use crate::gm::{fn_8017AD04, fn_8017AD28, fn_8017AD48, fn_8017AD78};

/// Callees in TUs that are not ported yet.
pub mod ext {
    use super::*;
    ext_fns! {
        fn fn_80174274() -> *mut MatchEnd = crate::tu::adapt::fn_80174274__726c5b;
        fn lbLang_IsSavedLanguageUS() -> i32 = crate::tu::adapt::lbLang_IsSavedLanguageUS__90e41e;
    }
}

const Gm_PKind_NA: u8 = 3;

/// inlines.h `gmClampResultStat`.
#[inline]
pub fn gmClampResultStat(value: i32) -> i32 {
    melee_core::ct!("gmClampResultStat");
    if value > 999999 {
        999999
    } else if value < -999999 {
        -999999
    } else {
        value
    }
}

#[inline]
unsafe fn standings(slot: i32) -> *mut MatchPlayerData {
    let me = ext::fn_80174274();
    core::ptr::addr_of_mut!((*me).player_standings[slot as usize])
}

/// `gmClampResultStat(me->player_standings[slot].<field>)` readers.
macro_rules! clamped_stat {
    ($($name:ident => $field:ident),* $(,)?) => {$(
        pub unsafe fn $name(slot: i32) -> i32 {
            let me = ext::fn_80174274();
            gmClampResultStat((*me).player_standings[slot as usize].$field as i32)
        }
    )*};
}

clamped_stat! {
    fn_8017ADA8 => x1C,
    fn_8017AE0C => x20,
    fn_8017B21C => self_destructs,
    fn_8017B280 => x40,
    fn_8017B2E4 => x44,
    fn_8017B348 => x48,
    fn_8017B3AC => x4C,
    fn_8017B4D0 => x6C,
    fn_8017B534 => x70,
    fn_8017B598 => x34,
    fn_8017B5FC => x74,
    fn_8017B660 => x78,
    fn_8017B6C4 => x7C,
    fn_8017B728 => x84,
    fn_8017B78C => x88,
    fn_8017B7F0 => x8C,
    fn_8017B854 => x80,
    fn_8017B8B8 => x90,
}

/// Kills the player `slot` scored on player `$t`: -1 for the player itself or an absent target.
macro_rules! kills_on {
    ($($name:ident => $t:literal),* $(,)?) => {$(
        pub unsafe fn $name(arg0: i32) -> i32 {
            let me = ext::fn_80174274();
            if arg0 == $t {
                return -1;
            }
            if (*me).player_standings[$t].pkind != Gm_PKind_NA {
                return fn_8017AD04((*me).player_standings[arg0 as usize].kills[$t] as i32, 999999);
            }
            -1
        }
    )*};
}

kills_on! {
    fn_8017AE70 => 0,
    fn_8017AED8 => 1,
    fn_8017AF40 => 2,
    fn_8017AFA8 => 3,
}

/// Kills player `$t` scored on `arg0`.
macro_rules! kills_by {
    ($($name:ident => $t:literal),* $(,)?) => {$(
        pub unsafe fn $name(arg0: i32) -> i32 {
            let me = ext::fn_80174274();
            if arg0 == $t {
                return -1;
            }
            if (*me).player_standings[$t].pkind != Gm_PKind_NA {
                return fn_8017AD04((*me).player_standings[$t].kills[arg0 as usize] as i32, 999999);
            }
            -1
        }
    )*};
}

kills_by! {
    fn_8017B07C => 0,
    fn_8017B0E4 => 1,
    fn_8017B14C => 2,
    fn_8017B1B4 => 3,
}

pub unsafe fn fn_8017B010(arg0: i32) -> i32 {
    melee_core::ct!("fn_8017B010");
    let p = standings(arg0);
    gmClampResultStat((*p).x24.wrapping_sub((*p).self_destructs as i32))
}

pub unsafe fn fn_8017B410(arg0: i32) -> i32 {
    melee_core::ct!("fn_8017B410");
    let p = standings(arg0);
    let var_r3: i32 = if ext::lbLang_IsSavedLanguageUS() != 0 {
        // (u32)(f32)(x5C / 30.4788f)
        melee_core::ppc::fcvt_u32(((*p).x5C as f32 / 30.4788f32)) as i32
    } else {
        ((*p).x5C / 100) as i32
    };
    gmClampResultStat(var_r3)
}

pub unsafe fn fn_8017B91C(arg0: i32) -> i32 {
    melee_core::ct!("fn_8017B91C");
    let p = standings(arg0);
    let var_r3: u32 = if ext::lbLang_IsSavedLanguageUS() != 0 {
        // (u32)(f32)((f32)x94 / 30.4788): the divisor is a double literal
        melee_core::ppc::fcvt_u32((((*p).x94 as f32 as f64 / 30.4788f64) as f32))
    } else {
        melee_core::ppc::fcvt_u32(((*p).x94 as f32 / 100.0f32))
    };
    gmClampResultStat(var_r3 as i32)
}

pub unsafe fn fn_8017B9F4(arg0: i32) -> i32 {
    melee_core::ct!("fn_8017B9F4");
    let p = standings(arg0);
    let var_r3: u32 = if ext::lbLang_IsSavedLanguageUS() != 0 {
        melee_core::ppc::fcvt_u32((((*p).x98 as f32 as f64 / 30.4788f64) as f32))
    } else {
        melee_core::ppc::fcvt_u32(((*p).x98 as f32 / 100.0f32))
    };
    gmClampResultStat(var_r3 as i32)
}

// ---- folded from gm.rs (consolidation steps 21/22) ----
/// gm_17AD.c `fn_8017AD04_fp`: clamp to [-arg1, arg1] (upper bound tested first).
pub fn fn_8017AD04_fp(arg0: i32, arg1: i32) -> i32 {
    melee_core::ct!("fn_8017AD04_fp");
    if arg0 > arg1 {
        return arg1;
    }
    let neg = arg1.wrapping_neg();
    if arg0 < neg {
        return neg;
    }
    arg0
}

/// gm_17AD.c `fn_8017AD28_fp`.
pub fn fn_8017AD28_fp(arg0: i32) -> i32 {
    melee_core::ct!("fn_8017AD28_fp");
    clamp_sym(arg0, 9999)
}

/// gm_17AD.c `fn_8017AD48_fp`.
pub fn fn_8017AD48_fp(arg0: i32) -> i32 {
    melee_core::ct!("fn_8017AD48_fp");
    clamp_sym(arg0, 9999999)
}

/// gm_17AD.c `fn_8017AD78_fp`.
pub fn fn_8017AD78_fp(arg0: i32) -> i32 {
    melee_core::ct!("fn_8017AD78_fp");
    clamp_sym(arg0, 99999)
}
