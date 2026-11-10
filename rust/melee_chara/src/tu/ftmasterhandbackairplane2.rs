//! src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane2.c. No float arithmetic (no fused
//! sites). Smoke-tested by tests/tu_ftmasterhandbackairplane2.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use melee_types::*;

/// Already ported (rounds 8/10): re-exported under this TU.



pub fn ftMh_BackAirplane2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackAirplane2_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        melee_fighter::tu::ftbosslib::ftBossLib_8015BF74(gobj, (*da).x58);
    }
}

pub fn ftMh_BackAirplane2_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_BackAirplane2_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_MS_368_80153A64` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane2.c).
pub fn ftMh_MS_368_80153A64(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_368_80153A64");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 369 /* ftMh_MS_BackAirplane3 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cur_pos.x = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x60;
        (*fp).cur_pos.y = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x64;
        (*fp).self_vel.z = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0 */;
    }
}

/// `ftMh_BackAirplane2_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane2.c).
pub fn ftMh_BackAirplane2_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackAirplane2_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
