//! gmmain_lib.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use core::ptr::addr_of_mut;
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::gmmain::{gmMainLib_804D3EE0, persistent_fighter, save};

// ---- folded from gmmain.rs (consolidation steps 21/22) ----
#[inline]
pub fn bitset_mask_fp(bit: u32) -> u32 {
    melee_core::ct!("bitset_mask_fp");
    1u32 << (bit % 32)
}

/// `gmMainLib_8015D8B0_fp`: clear a bit of the `x1B40` set.
pub unsafe fn gmMainLib_8015D8B0_fp(arg0: u32) {
    melee_core::ct!("gmMainLib_8015D8B0_fp");
    let words = &mut (*save()).x1B40;
    words[(arg0 / 32) as usize] &= !bitset_mask_fp(arg0);
}

/// `gmMainLib_8015CCF0_fp`.
pub unsafe fn gmMainLib_8015CCF0_fp() -> *mut u32 {
    melee_core::ct!("gmMainLib_8015CCF0_fp");
    addr_of_mut!((*save()).x1A48)
}

/// `gmMainLib_8015CDC8_fp`.
pub unsafe fn gmMainLib_8015CDC8_fp() -> *mut gmm_x0_528_t {
    melee_core::ct!("gmMainLib_8015CDC8_fp");
    addr_of_mut!((*gmMainLib_804D3EE0).vs.unk_51C)
}

/// `gmMainLib_8015CDD4_fp`.
pub unsafe fn gmMainLib_8015CDD4_fp() -> *mut gmm_x0_528_t {
    melee_core::ct!("gmMainLib_8015CDD4_fp");
    addr_of_mut!((*gmMainLib_804D3EE0).vs.unk_522)
}

/// `gmMainLib_8015CDE0_fp`.
pub unsafe fn gmMainLib_8015CDE0_fp() -> *mut gmm_x0_528_t {
    melee_core::ct!("gmMainLib_8015CDE0_fp");
    addr_of_mut!((*gmMainLib_804D3EE0).vs.unk_528)
}

/// `gmMainLib_8015D06C_fp`.
pub unsafe fn gmMainLib_8015D06C_fp(arg0: u8) -> *mut i32 {
    melee_core::ct!("gmMainLib_8015D06C_fp");
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x84)
}

/// `gmMainLib_8015D48C_fp`.
pub unsafe fn gmMainLib_8015D48C_fp(arg0: u8) -> bool {
    melee_core::ct!("gmMainLib_8015D48C_fp");
    (*persistent_fighter(arg0)).x7C.b0() != 0
}

/// `gmMainLib_8015D4E8_fp`.
pub unsafe fn gmMainLib_8015D4E8_fp(arg0: u8, arg1: i32) {
    melee_core::ct!("gmMainLib_8015D4E8_fp");
    (*persistent_fighter(arg0)).x7C.set_b0(arg1 as u16);
}

/// `gmMainLib_GetBonusMatchTotal_fp`.
pub unsafe fn gmMainLib_GetBonusMatchTotal_fp() -> *mut u32 {
    melee_core::ct!("gmMainLib_GetBonusMatchTotal_fp");
    addr_of_mut!((*save()).bonus_matches)
}

/// `gmMainLib_GetCoinMatchTotal_fp`.
pub unsafe fn gmMainLib_GetCoinMatchTotal_fp() -> *mut u32 {
    melee_core::ct!("gmMainLib_GetCoinMatchTotal_fp");
    addr_of_mut!((*save()).coin_matches)
}

/// `gmMainLib_GetKOTotal_fp`.
pub unsafe fn gmMainLib_GetKOTotal_fp() -> *mut i32 {
    melee_core::ct!("gmMainLib_GetKOTotal_fp");
    addr_of_mut!((*save()).x1A5C)
}

/// `gmMainLib_GetStockMatchTotal_fp`.
pub unsafe fn gmMainLib_GetStockMatchTotal_fp() -> *mut u32 {
    melee_core::ct!("gmMainLib_GetStockMatchTotal_fp");
    addr_of_mut!((*save()).stock_matches)
}

/// `gmMainLib_GetTimeMatchTotal_fp`.
pub unsafe fn gmMainLib_GetTimeMatchTotal_fp() -> *mut u32 {
    melee_core::ct!("gmMainLib_GetTimeMatchTotal_fp");
    addr_of_mut!((*save()).time_matches)
}
