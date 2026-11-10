//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandrockcrush1.c: Crazy Hand rock-crush descent.
//! `--x0 > da->x48` is a float-vs-int compare in C (the countdown lives in a float). No fused
//! sites. Smoke-tested by tests/tu_ftcrazyhandrockcrush1.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftcrazyhandfingerbeam0::ext::ftCh_GrabUnk1_8015BC88;
use melee_types::*;

/// Already ported (rounds 3/8): re-exported under this TU.



pub fn ftCh_RockCrushDown_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_RockCrushDown_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_RockCrushDown_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_RockCrushDown_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        st::ft_80085134(gobj);
        (*fp).mv.ch.unk0.x0 -= 1.0;
        if (*fp).mv.ch.unk0.x0 > (*da).x48 as f32 || (*fp).mv.ch.unk0.x0 < 0.0 {
            (*fp).self_vel.x = 0.0;
        } else {
            melee_fighter::tu::ftbosslib::ftBossLib_8015C010(gobj, (*da).x44);
        }
        melee_fighter::tu::ftbosslib::ftBossLib_8015C190(gobj);
    }
}

pub fn ftCh_RockCrushDown_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_RockCrushDown_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `fn_80158144` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandrockcrush1.c).
pub fn fn_80158144(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80158144");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 357 /* ftMh_MS_RockCrushDown */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.ch.unk0.x0 = f32::from_bits(0x42D60000) /* 107 */;
    }
}

/// `ftCh_RockCrushDown_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandrockcrush1.c).
pub fn ftCh_RockCrushDown_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_RockCrushDown_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
