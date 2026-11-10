//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueezing0.c. No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandsqueezing0.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;





pub fn ftCh_Squeezing0_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Squeezing0_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_Squeezing0_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Squeezing0_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_Init_8015A184` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueezing0.c).
pub fn ftCh_Init_8015A184(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_8015A184");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 376 /* 0x178 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cur_pos.x = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs)).xE4;
        (*fp).cur_pos.y = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs)).xE8;
        (*fp).self_vel.z = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0 */;
    }
}

/// `ftCh_Squeezing0_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueezing0.c).
pub fn ftCh_Squeezing0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Squeezing0_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Squeezing0_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsqueezing0.c).
pub fn ftCh_Squeezing0_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Squeezing0_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
