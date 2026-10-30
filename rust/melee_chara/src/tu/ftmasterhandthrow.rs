//! src/melee/ft/kinds/ftMasterHand/ftmasterhandthrow.c. No float math, no fused sites.
//! Smoke-tested by tests/tu_ftmasterhandthrow.rs.
#![allow(non_snake_case)]

use melee_types::*;

/// Already ported (rounds 2/4/8): re-exported under this TU.
pub use crate::chars::r2::ftMasterHand::ftmasterhandthrow::ftMh_Throw_Phys;
pub use crate::chars::r4::ftMasterHand::ftmasterhandthrow::ftMh_MS_379_80155014;
pub use crate::chars::r8::ftMasterHand::ftmasterhandthrow::ftMh_Throw_IASA;

pub fn ftMh_Throw_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Throw_Coll");}
