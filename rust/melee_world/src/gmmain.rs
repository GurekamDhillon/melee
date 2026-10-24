//! The save-data / VS-state accessors of `src/melee/gm/gmmain_lib.c` over the global record
//! pointer `gmMainLib_804D3EE0` (`struct gmm_x0*`): the match / KO totals, the three VS record
//! windows, the persistent-fighter words. They live here, in the lowest crate that needs them (the
//! stage and HUD TUs read the totals); `melee_modes::tu::gmmain_lib` (which owns the record itself,
//! sets the pointer at boot and ports the rest of the file) re-exports the pointer and these
//! functions. Checked against the verbatim C definitions over a random record by
//! `tests/pure_gmmain.rs` (`python rust/melee_runtime/gen/pure/run_pure.py gmmain`).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use core::ptr::addr_of_mut;
use melee_types::*;

/// `struct gmm_x0* gmMainLib_804D3EE0` (C: `= &gmMainLib_8045A6C0`; set at boot by `melee_modes`).
pub static mut gmMainLib_804D3EE0: *mut gmm_x0 = core::ptr::null_mut();

#[inline]
pub unsafe fn save() -> *mut GmSaveData {
    addr_of_mut!((*gmMainLib_804D3EE0).thing.save_data)
}

#[inline]
pub unsafe fn persistent_fighter(selkind: u8) -> *mut FighterData {
    (addr_of_mut!((*save()).x1F2C) as *mut FighterData).add(selkind as usize)
}

#[inline]
fn bitset_mask(bit: u32) -> u32 {
    1u32 << (bit % 32)
}

/// `gmMainLib_8015D8B0`: clear a bit of the `x1B40` set.
pub unsafe fn gmMainLib_8015D8B0(arg0: u32) {
    let words = &mut (*save()).x1B40;
    words[(arg0 / 32) as usize] &= !bitset_mask(arg0);
}

/// `gmMainLib_8015CCF0`.
pub unsafe fn gmMainLib_8015CCF0() -> *mut u32 {
    addr_of_mut!((*save()).x1A48)
}

/// `gmMainLib_8015CDC8`.
pub unsafe fn gmMainLib_8015CDC8() -> *mut gmm_x0_528_t {
    addr_of_mut!((*gmMainLib_804D3EE0).vs.unk_51C)
}

/// `gmMainLib_8015CDD4`.
pub unsafe fn gmMainLib_8015CDD4() -> *mut gmm_x0_528_t {
    addr_of_mut!((*gmMainLib_804D3EE0).vs.unk_522)
}

/// `gmMainLib_8015CDE0`.
pub unsafe fn gmMainLib_8015CDE0() -> *mut gmm_x0_528_t {
    addr_of_mut!((*gmMainLib_804D3EE0).vs.unk_528)
}

/// `gmMainLib_8015D06C`.
pub unsafe fn gmMainLib_8015D06C(arg0: u8) -> *mut i32 {
    addr_of_mut!((*persistent_fighter(arg0)).x7C.x84)
}

/// `gmMainLib_8015D48C`.
pub unsafe fn gmMainLib_8015D48C(arg0: u8) -> bool {
    (*persistent_fighter(arg0)).x7C.b0() != 0
}

/// `gmMainLib_8015D4E8`.
pub unsafe fn gmMainLib_8015D4E8(arg0: u8, arg1: i32) {
    (*persistent_fighter(arg0)).x7C.set_b0(arg1 as u16);
}

/// `gmMainLib_GetBonusMatchTotal`.
pub unsafe fn gmMainLib_GetBonusMatchTotal() -> *mut u32 {
    addr_of_mut!((*save()).bonus_matches)
}

/// `gmMainLib_GetCoinMatchTotal`.
pub unsafe fn gmMainLib_GetCoinMatchTotal() -> *mut u32 {
    addr_of_mut!((*save()).coin_matches)
}

/// `gmMainLib_GetKOTotal`.
pub unsafe fn gmMainLib_GetKOTotal() -> *mut i32 {
    addr_of_mut!((*save()).x1A5C)
}

/// `gmMainLib_GetStockMatchTotal`.
pub unsafe fn gmMainLib_GetStockMatchTotal() -> *mut u32 {
    addr_of_mut!((*save()).stock_matches)
}

/// `gmMainLib_GetTimeMatchTotal`.
pub unsafe fn gmMainLib_GetTimeMatchTotal() -> *mut u32 {
    addr_of_mut!((*save()).time_matches)
}
