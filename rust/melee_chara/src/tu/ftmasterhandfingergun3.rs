//! src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun3.c: Master Hand finger-gun end state.
//! No float arithmetic. Smoke-tested by tests/tu_ftmasterhandfingergun3.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.



/// Calls into TUs that are not ported yet. Each is a settable hook.
pub mod ext {
    pub use crate::tu::ftmasterhanddamage0::ext::ftMh_MS_389_80151018;
}

pub fn ftMh_FingerGun3_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerGun3_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_FingerGun3_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_FingerGun3_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_FingerGun3_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun3.c).
pub fn ftMh_FingerGun3_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerGun3_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_MS_366_80153820` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun3.c).
pub fn ftMh_MS_366_80153820(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_366_80153820");
    st::Fighter_ChangeMotionState(gobj, 367 /* ftMh_MS_BackAirplane1 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftMh_FingerGun3_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun3.c).
pub fn ftMh_FingerGun3_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerGun3_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
