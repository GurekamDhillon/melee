//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweep.c. No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandsweep.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

pub use crate::chars::r2::ftCrazyHand::ftcrazyhandsweep::{fn_8015746C, ftCh_SweepLoop_Phys};
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandsweep::ftCh_SweepLoop_IASA;

pub fn ftCh_SweepLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_SweepLoop_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_SweepLoop_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_SweepLoop_Coll");}
