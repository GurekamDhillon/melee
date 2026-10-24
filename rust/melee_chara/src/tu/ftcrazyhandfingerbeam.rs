//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c: `ftCh_Init_80158B3C`, the finger-beam
//! windup state. No float math.
#![allow(non_snake_case, unused_unsafe)]

use crate::stubs as st;
use melee_types::*;

/// `ftCh_Init_80158B3C`.
pub fn ftCh_Init_80158B3C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        st::Fighter_ChangeMotionState(gobj, 0x16A, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_SetAnimRate(gobj, (*da).xCC_pos.x);
        (*fp).mv.ch.unk0.x8 = (*da).xC4_pos.y as i32; // C float -> s32 truncation
    }
}
