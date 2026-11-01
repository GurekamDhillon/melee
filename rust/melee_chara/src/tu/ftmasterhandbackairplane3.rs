//! src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane3.c (airplane-3 and BackPunch states).
//! Float math: `x0` int->float and a `-1` countdown; no multiply-add shape, so no fused sites.
//! Smoke-tested by tests/tu_ftmasterhandbackairplane3.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.



/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    /// `ftMh_MS_389_80151018(gobj)` (ftmasterhandwait12.c).
    pub static mut FTMH_MS_389_80151018: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftMh_MS_389_80151018(gobj: *mut HSD_GObj) {
        match unsafe { FTMH_MS_389_80151018 } {
            Some(f) => f(gobj),
            None => { unsafe { crate::tu::ftmasterhandwait12::ftMh_MS_389_80151018(gobj as _); } return; },
        }
    }
}

pub fn ftMh_BackAirplane3_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackAirplane3_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_BackAirplane3_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_BackAirplane3_Coll");}

pub fn ftMh_MS_369_80153B90(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_369_80153B90");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        st::Fighter_ChangeMotionState(gobj, 370, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.mh.unk0.x0 = (*da).x6C as f32;
        melee_fighter::tu::ftbosslib::ftBossLib_8015C208(gobj, &mut pos);
        (*fp).cur_pos.x = pos.x;
        (*fp).cur_pos.y = (*da).x68;
        (*fp).self_vel.z = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = 0.0;
    }
}

pub fn ftMh_BackPunch_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackPunch_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            (*fp).self_vel.x = 0.0;
            ext::ftMh_MS_389_80151018(gobj);
        }
    }
}

pub fn ftMh_BackPunch_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackPunch_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).mv.mh.unk0.x0 -= 1.0;
        if (*fp).mv.mh.unk0.x0 > 0.0 {
            let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
            melee_fighter::tu::ftbosslib::ftBossLib_8015BF74(gobj, (*da).x58);
        } else {
            (*fp).self_vel.x = 0.0;
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_BackAirplane3_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane3.c).
pub fn ftMh_BackAirplane3_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackAirplane3_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_BackAirplane3_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane3.c).
pub fn ftMh_BackAirplane3_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackAirplane3_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_BackPunch_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackairplane3.c).
pub fn ftMh_BackPunch_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackPunch_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
