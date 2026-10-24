//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandcancel.c: `ftCh_Init_8015A030`, the state after a
//! cancelled grab. No float math.
#![allow(non_snake_case, unused_unsafe)]

use crate::stubs as st;
use melee_types::*;

/// `ftCh_Init_8015A030`.
pub fn ftCh_Init_8015A030(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        st::Fighter_ChangeMotionState(gobj, 0x177, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        {
            let mut pos: Vec3 = core::mem::zeroed();
            melee_fighter::tu::ftbosslib::ftBossLib_8015C208(gobj, &mut pos);
            (*fp).cur_pos.x = pos.x;
        }
        (*fp).cur_pos.y = (*da).xE0;
        (*fp).self_vel.x = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.z = 0.0;
    }
}
