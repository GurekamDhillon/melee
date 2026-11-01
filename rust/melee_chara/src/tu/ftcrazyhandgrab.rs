//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandgrab.c: Crazy Hand's grab. No float math.
//! Smoke-tested by tests/tu_ftcrazyhandgrab.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.



pub mod ext {
    use melee_types::*;
    crate::port_hook!(FTCH_INIT_8015A030, ftCh_Init_8015A030, crate::tu::ftcrazyhandcancel::ftCh_Init_8015A030, (gobj: *mut HSD_GObj));
}

pub fn ftCh_Grab_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Grab_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftCh_Init_8015A030(gobj);
    }
}

pub fn ftCh_Grab_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Grab_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_Init_80159F40` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandgrab.c).
pub fn ftCh_Init_80159F40(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80159F40");
    st::Fighter_ChangeMotionState(gobj, 374 /* ftMh_MS_Grab */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftCh_Grab_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandgrab.c).
pub fn ftCh_Grab_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Grab_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Grab_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandgrab.c).
pub fn ftCh_Grab_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Grab_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
