//! src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane2.c. No float arithmetic (no fused
//! sites). Smoke-tested by tests/tu_ftmasterhandbackairplane2.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 8/10): re-exported under this TU.
pub use crate::chars::r10::ftMasterHand::ftmasterhandbackairplane2::ftMh_MS_368_80153A64;
pub use crate::chars::r8::ftMasterHand::ftmasterhandbackairplane2::ftMh_BackAirplane2_IASA;

pub fn ftMh_BackAirplane2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackAirplane2_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        melee_fighter::tu::ftbosslib::ftBossLib_8015BF74(gobj, (*da).x58);
    }
}

pub fn ftMh_BackAirplane2_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_BackAirplane2_Coll");}
