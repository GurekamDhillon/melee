//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackcrush.c. No float math. Smoke-tested by
//! tests/tu_ftcrazyhandbackcrush.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.
pub use crate::chars::r2::ftCrazyHand::ftcrazyhandbackcrush::ftCh_BackCrush_Phys;
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandbackcrush::ftCh_BackCrush_IASA;

pub fn ftCh_Init_801597F0(gobj: *mut HSD_GObj, cb: HSD_GObjEvent) {
    melee_core::ct!("ftCh_Init_801597F0");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 0x173, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.ch.unk0.x4 = cb;
    }
}

pub fn ftCh_BackCrush_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackCrush_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            if let Some(cb) = (*fp).mv.ch.unk0.x4 {
                cb(gobj);
            }
        }
    }
}

pub fn ftCh_BackCrush_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_BackCrush_Coll");}
