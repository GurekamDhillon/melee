//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandgrab.c: Crazy Hand's grab. No float math.
//! Smoke-tested by tests/tu_ftcrazyhandgrab.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.
pub use crate::chars::r2::ftCrazyHand::ftcrazyhandgrab::{ftCh_Grab_Phys, ftCh_Init_80159F40};
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandgrab::ftCh_Grab_IASA;

pub mod ext {
    use melee_types::*;
    crate::port_hook!(FTCH_INIT_8015A030, ftCh_Init_8015A030, crate::tu::ftcrazyhandcancel::ftCh_Init_8015A030, (gobj: *mut HSD_GObj));
}

pub fn ftCh_Grab_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Grab_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftCh_Init_8015A030(gobj);
    }
}

pub fn ftCh_Grab_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Grab_Coll");}
