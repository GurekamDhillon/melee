//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackdisappear.c. Float math: `xF0` int->float, a
//! `-1` countdown and compares; no `a*b+c` shape, so no fused sites. Smoke-tested by
//! tests/tu_ftcrazyhandbackdisappear.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (round 8): re-exported under this TU.


pub fn fn_80159908(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80159908");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        st::Fighter_ChangeMotionState(gobj, 0x174, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.ch.unk0.x0 = (*da).xF0 as f32;
        melee_fighter::tu::ftbosslib::ftBossLib_8015C208(gobj, &mut pos);
        (*fp).cur_pos.x = pos.x;
        (*fp).cur_pos.y = (*da).xEC;
        (*fp).self_vel.z = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = 0.0;
    }
}

pub fn ftCh_BackDisappear_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackDisappear_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            (*fp).self_vel.x = 0.0;
            crate::tu::ftcrazyhandbackairplane2::ext::ftCh_GrabUnk1_8015BC88(gobj);
        }
    }
}

pub fn ftCh_BackDisappear_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackDisappear_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).mv.ch.unk0.x0 -= 1.0;
        if (*fp).mv.ch.unk0.x0 > 0.0 {
            let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
            melee_fighter::tu::ftbosslib::ftBossLib_8015BF74(gobj, (*da).xDC);
        } else {
            (*fp).self_vel.x = 0.0;
        }
    }
}

pub fn ftCh_BackDisappear_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_BackDisappear_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_BackDisappear_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackdisappear.c).
pub fn ftCh_BackDisappear_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackDisappear_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
