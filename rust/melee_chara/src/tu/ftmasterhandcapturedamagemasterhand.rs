//! src/melee/ft/kinds/ftMasterHand/ftmasterhandcapturedamagemasterhand.c. No float arithmetic.
//! Smoke-tested by tests/tu_ftmasterhandcapturedamagemasterhand.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use melee_types::*;

/// Already ported (round 7): re-exported under this TU.


pub fn ftMh_CaptureDamageMasterHand_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureDamageMasterHand_IASA");}

pub fn ftMh_CaptureDamageMasterHand_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureDamageMasterHand_Phys");}

pub fn ftMh_CaptureDamageMasterHand_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_CaptureDamageMasterHand_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_CaptureDamageMasterHand_80155C94` (src/melee/ft/kinds/ftMasterHand/ftmasterhandcapturedamagemasterhand.c).
pub fn ftMh_CaptureDamageMasterHand_80155C94(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_CaptureDamageMasterHand_80155C94");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 329 /* ftCo_MS_CaptureWaitMasterHand */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).set_invisible(1);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (511) */ as i16));
        (*fp).set_x2220_b3(1);
        st::ftAnim_8006EBA4(gobj);
    }
}
