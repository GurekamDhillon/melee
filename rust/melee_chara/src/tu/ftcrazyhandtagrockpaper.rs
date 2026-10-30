//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandtagrockpaper.c. No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandtagrockpaper.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

use melee_fighter::tu::ftbosslib as bl;

pub use crate::chars::r2::ftCrazyHand::ftcrazyhandtagrockpaper::fn_8015AAC8;
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandtagrockpaper::ftCh_TagRockPaper_IASA;

pub fn ftCh_TagRockPaper_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagRockPaper_Anim");
    if bl::ftBossLib_8015C2E0() || bl::ftBossLib_8015C358() {
        crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj);
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::Fighter_ChangeMotionState(gobj, 0x180, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftCh_TagRockPaper_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_TagRockPaper_Phys");}

pub fn ftCh_TagRockPaper_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_TagRockPaper_Coll");}
