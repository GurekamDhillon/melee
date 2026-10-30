//! src/melee/ft/kinds/ftPurin/ftpurinspeciallw.c: Jigglypuff's down-B (Rest). No float arithmetic,
//! so no fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftpurinspeciallw.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.
pub use crate::chars::r2::ftPurin::ftpurinspeciallw::{ftPr_SpecialAirLw_Phys, ftPr_SpecialLw_Phys};

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// The Enter body shared by the ground and air states.
unsafe fn enter(gobj: G, left_id: u32, right_id: u32) {
    let mut fp = fpp(gobj);
    if -1.0 == (*fp).facing_dir {
        st::Fighter_ChangeMotionState(gobj, left_id as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    } else {
        st::Fighter_ChangeMotionState(gobj, right_id as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
    st::ftAnim_8006EBA4(gobj);
    fp = fpp(gobj);
    (*fp).cmd_vars[0] = 0;
    (*fp).accessory4_cb = Some(crate::chars::cb::ftpurinspecialhi__ftPr_SpecialHi_8013CE7C);
}

pub fn ftPr_SpecialLw_Enter(gobj: G) {
    melee_core::ct!("ftPr_SpecialLw_Enter");
    unsafe { enter(gobj, ftPr_MS_SpecialLwL as u32, ftPr_MS_SpecialLwR as u32) }
}

pub fn ftPr_SpecialAirLw_Enter(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirLw_Enter");
    unsafe { enter(gobj, ftPr_MS_SpecialAirLwL as u32, ftPr_MS_SpecialAirLwR as u32) }
}

pub fn ftPr_SpecialLw_Anim(fighter_gobj: G) {
    melee_core::ct!("ftPr_SpecialLw_Anim");
    if st::ftAnim_IsFramesRemaining(fighter_gobj) == 0 {
        st::ft_8008A2BC(fighter_gobj);
    }
}

pub fn ftPr_SpecialAirLw_Anim(fighter_gobj: G) {
    melee_core::ct!("ftPr_SpecialAirLw_Anim");
    if st::ftAnim_IsFramesRemaining(fighter_gobj) == 0 {
        st::ftCo_Fall_Enter(fighter_gobj);
    }
}

pub fn ftPr_SpecialLw_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialLw_IASA");}
pub fn ftPr_SpecialAirLw_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialAirLw_IASA");}

pub fn ftPr_SpecialLw_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialLw_Coll");
    if st::ft_800827A0(gobj) == 0 {
        ftPr_SpecialLw_8013D104(gobj);
    }
}

pub fn ftPr_SpecialAirLw_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirLw_Coll");
    if st::ft_80081D0C(gobj) != GA_Ground as i32 {
        ftPr_SpecialLw_8013D19C(gobj);
    }
}

pub fn ftPr_SpecialLw_8013D104(gobj: G) {
    melee_core::ct!("ftPr_SpecialLw_8013D104");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        let msid = if -1.0 == (*fp).facing_dir { ftPr_MS_SpecialAirLwL } else { ftPr_MS_SpecialAirLwR };
        st::Fighter_ChangeMotionState(gobj, msid as i32, 0x0C4C508E, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftPr_SpecialLw_8013D19C(gobj: G) {
    melee_core::ct!("ftPr_SpecialLw_8013D19C");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D7FC(&mut *fp);
        let msid = if -1.0 == (*fp).facing_dir { ftPr_MS_SpecialLwL } else { ftPr_MS_SpecialLwR };
        st::Fighter_ChangeMotionState(gobj, msid as i32, 0x0C4C508E, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}
