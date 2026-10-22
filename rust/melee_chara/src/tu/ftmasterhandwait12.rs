//! src/melee/ft/kinds/ftMasterHand/ftmasterhandwait12.c (+ the static inline
//! `ftMh_UpdateBossMotion` of ftMasterHand/inlines.h). No float math, no fused sites.
//! Smoke-tested by tests/tu_ftmasterhandwait12.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_fighter::tu::ftbosslib as bl;
use melee_types::*;

/// Already ported (rounds 3/4): re-exported under this TU.
pub use crate::chars::r3::ftMasterHand::ftmasterhandwait12::ftMh_Wait1_2_Anim;
pub use crate::chars::r4::ftMasterHand::ftmasterhandwait12::{ftMh_MS_389_80150C8C, ftMh_MS_389_80150D28};

/// `ftMh_MS_341_8014FFDC` (ftmasterhandwait10.c, ported in round 4) as a GObj callback.
unsafe extern "C" fn wait10_init_cb(gobj: *mut HSD_GObj) {
    crate::chars::r4::ftMasterHand::ftmasterhandwait10::ftMh_MS_341_8014FFDC(gobj)
}

/// `static inline ftMh_UpdateBossMotion(gobj, fp, da)` (inlines.h).
pub fn ftMh_UpdateBossMotion(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *const ftMasterHand_SpecialAttrs) {
    unsafe {
        st::ft_80085134(gobj);
        let pos = &*(&raw const (*fp).mv.mh.unk0.xC);
        let out = &mut *(&raw mut (*fp).mv.mh.unk0.x18);
        bl::ftBossLib_8015BE40(gobj, pos, out, (*da).x2C, (*da).x28);
    }
}

pub fn ftMh_MS_389_80150DC4(gobj: *mut HSD_GObj, cb: HSD_GObjEvent, pos: *const Vec3) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).u.mh.x2258 == ftMh_MS_Wait2_0 as i32 {
            if (*fp).u.mh.x2258 == ftMh_MS_Wait2_0 as i32 || (*fp).u.mh.x2258 == ftMh_MS_Wait2_1 as i32 {
                st::Fighter_ChangeMotionState(gobj, ftMh_MS_Wait2_1 as i32, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            } else {
                st::Fighter_ChangeMotionState(gobj, ftMh_MS_Wait2_1 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                st::ftAnim_8006EBA4(gobj);
            }
            (*fp).u.mh.x2258 = ftMh_MS_Wait2_1 as i32;
        } else {
            if (*fp).u.mh.x2258 == ftMh_MS_Wait1_0 as i32 || (*fp).u.mh.x2258 == ftMh_MS_Wait1_2 as i32 {
                st::Fighter_ChangeMotionState(gobj, ftMh_MS_Wait1_2 as i32, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            } else {
                st::Fighter_ChangeMotionState(gobj, ftMh_MS_Wait1_2 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                st::ftAnim_8006EBA4(gobj);
            }
            (*fp).u.mh.x2258 = ftMh_MS_Wait1_2 as i32;
        }
        (*fp).mv.mh.unk0.x4 = cb;
        (*fp).mv.mh.unk0.xC = *pos;
    }
}

pub fn ftMh_Wait1_2_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        ftMh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftMh_Wait1_2_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).mv.mh.unk0.x18 == 0.0 {
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            (*fp).self_vel.x = 0.0;
            if let Some(cb) = (*fp).mv.mh.unk0.x4 {
                cb(gobj);
            }
        }
    }
}

pub fn ftMh_MS_389_80151018(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        (*fp).mv.mh.unk0.x20 = 0;
        let pos = Vec3 { x: (*da).x30_pos2.x, y: (*da).x30_pos2.y, z: 0.0 };
        (*fp).u.mh.x2258 = ftMh_MS_Wait2_1 as i32;
        // The inlined body of ftMh_MS_389_80150DC4-style dispatch on x2258.
        let fp2 = (*gobj).user_data as *mut Fighter;
        if (*fp2).u.mh.x2258 == ftMh_MS_Wait2_0 as i32 {
            ftMh_MS_389_80150D28(gobj);
        } else {
            ftMh_MS_389_80150C8C(gobj);
        }
        (*fp2).mv.mh.unk0.x4 = Some(wait10_init_cb);
        (*fp2).mv.mh.unk0.xC = pos;
    }
}
