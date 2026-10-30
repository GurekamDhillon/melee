//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandrockcrush1.c: Crazy Hand rock-crush descent.
//! `--x0 > da->x48` is a float-vs-int compare in C (the countdown lives in a float). No fused
//! sites. Smoke-tested by tests/tu_ftcrazyhandrockcrush1.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftcrazyhandfingerbeam0::ext::ftCh_GrabUnk1_8015BC88;
use melee_types::*;

/// Already ported (rounds 3/8): re-exported under this TU.
pub use crate::chars::r3::ftCrazyHand::ftcrazyhandrockcrush1::fn_80158144;
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandrockcrush1::ftCh_RockCrushDown_IASA;

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
