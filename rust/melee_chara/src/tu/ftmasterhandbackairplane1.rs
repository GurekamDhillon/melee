//! src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane1.c: Master Hand back-airplane (start).
//! No float arithmetic (no fused sites). Smoke-tested by tests/tu_ftmasterhandbackairplane1.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8/10): re-exported under this TU.




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
    melee_core::ct!("ftMh_BackAirplane1_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        doAnim(gobj);
    }
}

pub fn ftMh_BackAirplane1_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_BackAirplane1_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_BackAirplane2_Anim` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane1.c).
pub fn ftMh_BackAirplane2_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackAirplane2_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
            crate::chars::r10::ftMasterHand::ftmasterhandbackairplane2::ftMh_MS_368_80153A64(gobj);
        }
    }
}

/// `ftMh_BackAirplane1_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane1.c).
pub fn ftMh_BackAirplane1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackAirplane1_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_BackAirplane1_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane1.c).
pub fn ftMh_BackAirplane1_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackAirplane1_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
