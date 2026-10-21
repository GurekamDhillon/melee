//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueezing0.c. No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandsqueezing0.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

pub use crate::chars::r10::ftCrazyHand::ftcrazyhandsqueezing0::ftCh_Init_8015A184;
pub use crate::chars::r2::ftCrazyHand::ftcrazyhandsqueezing0::ftCh_Squeezing0_Phys;
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandsqueezing0::ftCh_Squeezing0_IASA;

pub fn ftCh_Squeezing0_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_Squeezing0_Coll(_gobj: *mut HSD_GObj) {}
