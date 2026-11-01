//! src/melee/ft/kinds/ftMasterHand/ftmasterhandsqueeze.c: Master Hand squeeze state.
//! Also holds `ftMh_UpdateBossMotion` (static inline of ftMasterHand/inlines.h). No fused sites.
//! Smoke-tested by tests/tu_ftmasterhandsqueeze.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
#[allow(unused_imports)]
#[allow(unused_imports)]
use crate::tu::ftmasterhandcapturemasterhand::ftMh_CaptureMasterHand_80155B80;
use crate::tu::ftmasterhandthrow::ftMh_MS_379_80155014;
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftmasterhandsqueezing::ftMh_MS_376_80154E78;
use melee_types::*;

/// Already ported (rounds 4/7/8): re-exported under this TU.




/// `ftMh_UpdateBossMotion` (ftMasterHand/inlines.h, static inline).
pub unsafe fn ftMh_UpdateBossMotion(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *const ftMasterHand_SpecialAttrs) {
    melee_core::ct!("ftMh_UpdateBossMotion");
    unsafe {
        st::ft_80085134(gobj);
        let xc = (*fp).mv.mh.unk0.xC;
        melee_fighter::tu::ftbosslib::ftBossLib_8015BE40(gobj, &xc, &mut (*fp).mv.mh.unk0.x18, (*da).x2C, (*da).x28);
    }
}

pub fn ftMh_MS_378_80154A78(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_378_80154A78");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        (*fp).cmd_vars[1] = 0;
        st::Fighter_ChangeMotionState(gobj, 378 /* ftMh_MS_Squeeze */, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).set_x2222_b2(1);
        st::ftCommon_8007E2F4(&mut *fp, 511);
        st::ftCommon_8007E2FC(gobj);
        ftMh_CaptureMasterHand_80155B80((*fp).victim_gobj as *mut HSD_GObj);
        (*fp).mv.mh.unk0.xC.x = (*da).x118_pos.x;
        (*fp).mv.mh.unk0.xC.y = (*da).x118_pos.y;
        (*fp).mv.mh.unk0.xC.z = 0.0;
    }
}

pub fn ftMh_Squeeze_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Squeeze_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[1] != 0 {
            melee_fighter::tu::ftbosslib::ftBossLib_8015C5F8(gobj);
            (*fp).cmd_vars[1] = 0;
        }
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            if (*fp).u.mh.x2250 == 379 /* ftMh_MS_Throw */ {
                ftMh_MS_376_80154E78(gobj);
            } else {
                ftMh_MS_379_80155014(gobj);
            }
        }
    }
}

pub fn ftMh_Squeeze_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Squeeze_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        ftMh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftMh_Squeeze_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Squeeze_Coll");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).mv.mh.unk0.x18 == 0.0 {
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            (*fp).self_vel.x = 0.0;
        }
    }
}





/// `ftMh_Squeeze_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandsqueeze.c).
    pub fn ftMh_Squeeze_IASA(gobj: *mut HSD_GObj) {
        unsafe {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
                st::ftBossLib_8015BD20(gobj);
            }
        }
    }

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftMh_Squeeze_Coll_fp(fp: &mut Fighter) {
    melee_core::ct!("ftMh_Squeeze_Coll_fp");
    crate::chars::ftMasterHand::stop_if_x18_zero(fp)
}
