
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

pub use crate::tu::gmmain_lib::bitset_mask_fp as bitset_mask;
pub use crate::tu::gmmain_lib::gmMainLib_8015CCF0_fp as gmMainLib_8015CCF0;
pub use crate::tu::gmmain_lib::gmMainLib_8015CDC8_fp as gmMainLib_8015CDC8;
pub use crate::tu::gmmain_lib::gmMainLib_8015CDD4_fp as gmMainLib_8015CDD4;
pub use crate::tu::gmmain_lib::gmMainLib_8015CDE0_fp as gmMainLib_8015CDE0;
pub use crate::tu::gmmain_lib::gmMainLib_8015D06C_fp as gmMainLib_8015D06C;
pub use crate::tu::gmmain_lib::gmMainLib_8015D48C_fp as gmMainLib_8015D48C;
pub use crate::tu::gmmain_lib::gmMainLib_8015D4E8_fp as gmMainLib_8015D4E8;
pub use crate::tu::gmmain_lib::gmMainLib_8015D8B0_fp as gmMainLib_8015D8B0;
pub use crate::tu::gmmain_lib::gmMainLib_GetBonusMatchTotal_fp as gmMainLib_GetBonusMatchTotal;
pub use crate::tu::gmmain_lib::gmMainLib_GetCoinMatchTotal_fp as gmMainLib_GetCoinMatchTotal;
pub use crate::tu::gmmain_lib::gmMainLib_GetKOTotal_fp as gmMainLib_GetKOTotal;
pub use crate::tu::gmmain_lib::gmMainLib_GetStockMatchTotal_fp as gmMainLib_GetStockMatchTotal;
pub use crate::tu::gmmain_lib::gmMainLib_GetTimeMatchTotal_fp as gmMainLib_GetTimeMatchTotal;
