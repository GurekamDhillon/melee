//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackairplane3.c. No float math. Smoke-tested by
//! tests/tu_ftcrazyhandbackairplane3.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftcrazyhandbackairplane2::ext;
use melee_types::*;

/// Already ported (rounds 2/4/8): re-exported under this TU.



pub fn ftCh_BackAirplane3_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackAirplane3_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[0] != 0 {
            (*fp).cmd_vars[0] = 0;
            ext::ftCh_GrabUnk1_8015B850((*fp).victim_gobj, 0x153);
            if !(*fp).victim_gobj.is_null() {
                let victim = (*fp).victim_gobj;
                st::ftCommon_8007E2F4(&mut *fp, 0);
                ext::ftCo_800DE2A8(gobj, victim);
                ext::ftCo_800DE7C0(victim, core::ptr::null_mut(), false);
            }
            (*fp).mv.ch.unk0.x20 = 0;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ext::ftCh_GrabUnk1_8015BC88(gobj);
        }
    }
}

pub fn ftCh_BackAirplane3_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_BackAirplane3_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_BackAirplane3_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackairplane3.c).
pub fn ftCh_BackAirplane3_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackAirplane3_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_BackAirplane3_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackairplane3.c).
pub fn ftCh_BackAirplane3_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackAirplane3_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_Init_80159670` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackairplane3.c).
pub fn ftCh_Init_80159670(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80159670");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 369 /* 0x171 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cmd_vars[0] = 0_u32;
    }
}
