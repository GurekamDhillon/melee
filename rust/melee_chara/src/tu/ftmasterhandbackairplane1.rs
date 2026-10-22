//! src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane1.c: Master Hand back-airplane (start).
//! No float arithmetic (no fused sites). Smoke-tested by tests/tu_ftmasterhandbackairplane1.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8/10): re-exported under this TU.
pub use crate::chars::r10::ftMasterHand::ftmasterhandbackairplane1::ftMh_BackAirplane2_Anim;
pub use crate::chars::r2::ftMasterHand::ftmasterhandbackairplane1::ftMh_BackAirplane1_Phys;
pub use crate::chars::r8::ftMasterHand::ftmasterhandbackairplane1::ftMh_BackAirplane1_IASA;

/// `doAnim` (static): start BackAirplane2 at the other fighter's x and `da->x5C`.
fn doAnim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::Fighter_ChangeMotionState(gobj, 368, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        melee_fighter::tu::ftbosslib::ftBossLib_8015C208(gobj, &mut pos);
        (*fp).cur_pos.x = pos.x;
        (*fp).cur_pos.y = (*da).x5C;
        (*fp).self_vel.z = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = 0.0;
    }
}

pub fn ftMh_BackAirplane1_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        doAnim(gobj);
    }
}

pub fn ftMh_BackAirplane1_Coll(_gobj: *mut HSD_GObj) {}
