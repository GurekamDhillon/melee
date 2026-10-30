//! src/melee/ft/kinds/ftMasterHand/ftmasterhandcapturedamagemasterhand.c. No float arithmetic.
//! Smoke-tested by tests/tu_ftmasterhandcapturedamagemasterhand.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (round 7): re-exported under this TU.
pub use crate::chars::r7::ftMasterHand::ftmasterhandcapturedamagemasterhand::ftMh_CaptureDamageMasterHand_80155C94;

pub fn ftMh_CaptureDamageMasterHand_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureDamageMasterHand_IASA");}

pub fn ftMh_CaptureDamageMasterHand_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureDamageMasterHand_Phys");}

pub fn ftMh_CaptureDamageMasterHand_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureDamageMasterHand_Coll");}
