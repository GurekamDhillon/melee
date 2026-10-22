//! src/melee/ft/kinds/ftMasterHand/ftmasterhandslap.c: Master Hand slap state. No fused sites.
//! Smoke-tested by tests/tu_ftmasterhandslap.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.
pub use crate::chars::r2::ftMasterHand::ftmasterhandslap::ftMh_Slap_Phys;
pub use crate::chars::r8::ftMasterHand::ftmasterhandslap::ftMh_Slap_IASA;

/// Calls into TUs that are not ported yet (settable hooks; unset they panic). The ftmasterhandwait12.c
/// function is shared by every Master Hand TU that returns to the wait state.
pub mod ext {
    use melee_types::*;
    crate::ch_hook!(FTMH_MS_389_80151018, ftMh_MS_389_80151018, (gobj: *mut HSD_GObj));
}

pub fn ftMh_Slap_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_Slap_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftMh_MS_349_80151CA8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::Fighter_ChangeMotionState(gobj, 353 /* ftMh_MS_WalkShoot */, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cur_pos.y = (*da).x50.x;
        (*fp).self_vel.x = 0.0;
    }
}
