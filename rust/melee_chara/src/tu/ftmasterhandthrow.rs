//! src/melee/ft/kinds/ftMasterHand/ftmasterhandthrow.c. No float math, no fused sites.
//! Smoke-tested by tests/tu_ftmasterhandthrow.rs.
#![allow(non_snake_case)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use melee_types::*;

/// Already ported (rounds 2/4/8): re-exported under this TU.



pub fn ftMh_Throw_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Throw_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_Throw_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandthrow.c).
pub fn ftMh_Throw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Throw_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_Throw_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandthrow.c).
pub fn ftMh_Throw_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Throw_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_MS_379_80155014` (src/melee/ft/kinds/ftMasterHand/ftmasterhandthrow.c).
pub fn ftMh_MS_379_80155014(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_379_80155014");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 380 /* ftMh_MS_Slam */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cmd_vars[0] = 0_u32;
    }
}
