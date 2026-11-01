//! src/melee/ft/kinds/ftMasterHand/ftmasterhandslap.c: Master Hand slap state. No fused sites.
//! Smoke-tested by tests/tu_ftmasterhandslap.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.



/// Calls into TUs that are not ported yet (settable hooks; unset they panic). The ftmasterhandwait12.c
/// function is shared by every Master Hand TU that returns to the wait state.
pub mod ext {
    use melee_types::*;
    pub static mut FTMH_MS_389_80151018: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftMh_MS_389_80151018(gobj: *mut HSD_GObj) {
        match unsafe { FTMH_MS_389_80151018 } {
            Some(f) => f(gobj),
            None => { unsafe { crate::tu::ftmasterhandwait12::ftMh_MS_389_80151018(gobj as _); } },
        }
    }
}

pub fn ftMh_Slap_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Slap_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_Slap_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Slap_Coll");}

pub fn ftMh_MS_349_80151CA8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_349_80151CA8");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::Fighter_ChangeMotionState(gobj, 353 /* ftMh_MS_WalkShoot */, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cur_pos.y = (*da).x50.x;
        (*fp).self_vel.x = 0.0;
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_Slap_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandslap.c).
pub fn ftMh_Slap_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Slap_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_Slap_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandslap.c).
pub fn ftMh_Slap_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Slap_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftMh_MS_349_80151CA8_fp` (src/melee/ft/kinds/ftMasterHand/ftmasterhandslap.c).
pub fn ftMh_MS_349_80151CA8_fp(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_349_80151CA8_fp");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 353 /* ftMh_MS_WalkShoot */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cur_pos.y = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x50.x;
        (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
    }
}
