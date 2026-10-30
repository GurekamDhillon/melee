//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandsweepwait.c. The only float op is the f32 add
//! `self_vel.x += da->x40` (no multiply: no fused sites).
//! Smoke-tested by tests/tu_ftcrazyhandsweepwait.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

pub use crate::chars::r2::ftCrazyHand::ftcrazyhandsweepwait::{fn_8015755C, ftCh_Init_801577F8, ftCh_SweepWait_Phys, ftCh_Walk2_Phys};
pub use crate::chars::r3::ftCrazyHand::ftcrazyhandsweepwait::{ftCh_Init_8015764C, ftCh_Slap_Anim, ftCh_SweepWait_Anim};
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandsweepwait::{ftCh_Slap_IASA, ftCh_SweepWait_IASA, ftCh_Walk2_IASA};

pub fn ftCh_SweepWait_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_SweepWait_Coll");}

pub fn ftCh_Slap_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Slap_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        st::ft_80085134(gobj);
        (*fp).self_vel.x += (*da).x40;
    }
}

/// `static void fn_801577B4(gobj)`: accessory4 callback of the slap.
pub unsafe extern "C" fn fn_801577B4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801577B4");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        if (*fp).cur_pos.x > (*da).x3C {
            (*fp).self_vel.x = 0.0;
            ftCh_Init_801577F8(gobj);
        }
    }
}

pub fn ftCh_Walk2_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Walk2_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_Slap_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Slap_Coll");}
