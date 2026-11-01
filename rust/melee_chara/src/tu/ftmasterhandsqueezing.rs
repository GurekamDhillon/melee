//! src/melee/ft/kinds/ftMasterHand/ftmasterhandsqueezing.c: Master Hand squeezing / throw states.
//! No fused sites. Smoke-tested by tests/tu_ftmasterhandsqueezing.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::chars::r9::ftMasterHand::ftmasterhandbackdisappear::ftMh_MS_374_801546D8;
use crate::stubs as st;
use crate::tu::ftmasterhandslap::ext::ftMh_MS_389_80151018;
use crate::tu::ftmasterhandsqueeze::{ftMh_MS_378_80154A78, ftMh_UpdateBossMotion};
use melee_types::*;

/// Already ported (rounds 4/8): re-exported under this TU.



/// Calls into TUs that are not ported yet (settable hooks; unset they panic).
pub mod ext {
    use melee_types::*;
    pub static mut FTMH_CAPTUREWAITMASTERHAND_80155D6C: Option<fn(*mut HSD_GObj, i32)> = None;
    #[allow(non_snake_case)]
    pub fn ftMh_CaptureWaitMasterHand_80155D6C(gobj: *mut HSD_GObj, msid: i32) {
        match unsafe { FTMH_CAPTUREWAITMASTERHAND_80155D6C } {
            Some(f) => f(gobj, msid),
            None => { unsafe { crate::tu::ftmasterhandthrownmasterhand::ftMh_CaptureWaitMasterHand_80155D6C(gobj as _, msid as _); } },
        }
    }
}
/// ftCo_Throw / ftCo_Thrown calls (hooks with the native melee_fighter port as default for DE7C0).
pub use crate::tu::ftcrazyhandbackairplane2::ext::{ftCo_800DE2A8, ftCo_800DE7C0};

pub fn ftMh_Squeezing_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Squeezing_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            if (*fp).mv.mh.unk0.x20 == 1 {
                ftMh_MS_378_80154A78(gobj);
            } else {
                ftMh_MS_374_801546D8(gobj);
            }
        }
    }
}

pub fn ftMh_Squeezing_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Squeezing_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        ftMh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftMh_Squeezing_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Squeezing_Coll");}

pub fn ftMh_Throw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Throw_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[0] != 0 {
            (*fp).cmd_vars[0] = 0;
            ext::ftMh_CaptureWaitMasterHand_80155D6C((*fp).victim_gobj as *mut HSD_GObj, ftCo_MS_ThrownMasterHand as i32);
            if !(*fp).victim_gobj.is_null() {
                let victim = (*fp).victim_gobj as *mut HSD_GObj;
                let victim_fp = (*victim).user_data as *mut Fighter;
                st::ftCommon_8007E2F4(&mut *fp, 0);
                ftCo_800DE2A8(gobj, victim);
                (*victim_fp).dmg.facing_dir_1 *= -1.0;
                ftCo_800DE7C0(victim, core::ptr::null_mut(), false);
            }
            (*fp).mv.mh.unk0.x20 = 0;
        }
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftMh_MS_389_80151018(gobj);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_MS_376_80154E78` (src/melee/ft/kinds/ftMasterHand/ftmasterhandsqueezing.c).
pub fn ftMh_MS_376_80154E78(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_376_80154E78");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 379 /* ftMh_MS_Throw */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cmd_vars[0] = 0_u32;
    }
}

/// `ftMh_Squeezing_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandsqueezing.c).
pub fn ftMh_Squeezing_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Squeezing_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
