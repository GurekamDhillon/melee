//! src/melee/ft/kinds/ftMasterHand/ftmasterhandsweepwait.c: Master Hand sweep-wait state.
//! No fused sites. Smoke-tested by tests/tu_ftmasterhandsweepwait.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use crate::tu::ftmasterhandslap::ext::ftMh_MS_389_80151018;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.
pub use crate::chars::r2::ftMasterHand::ftmasterhandsweepwait::{ftMh_MS_348_80151BB8, ftMh_SweepWait_Phys};
pub use crate::chars::r8::ftMasterHand::ftmasterhandsweepwait::ftMh_SweepWait_IASA;

pub fn ftMh_SweepWait_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_SweepWait_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_SweepWait_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_SweepWait_Coll");}
