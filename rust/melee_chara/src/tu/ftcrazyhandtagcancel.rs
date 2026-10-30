//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandtagcancel.c (+ the static inline
//! `ftCh_UpdateBossMotion` of ftCrazyHand/inlines.h). No fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandtagcancel.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_fighter::tu::ftbosslib as bl;
use melee_types::*;

pub use crate::chars::r4::ftCrazyHand::ftcrazyhandtagcancel::{ftCh_GrabUnk1_8015B8FC, ftCh_GrabUnk1_8015B998};

/// `ftCh_Init_80156198` as a GObj callback (ftcrazyhandwait10.c, already ported).
unsafe extern "C" fn wait10_init_cb(gobj: *mut HSD_GObj) {
    crate::chars::r4::ftCrazyHand::ftcrazyhandwait10::ftCh_Init_80156198(gobj)
}

pub fn ftCh_GrabUnk1_8015BA34(gobj: *mut HSD_GObj, cb: HSD_GObjEvent, pos: *mut Vec3) {
    melee_core::ct!("ftCh_GrabUnk1_8015BA34");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).u.mh.x2258 == ftMh_MS_Wait2_0 as i32 {
            if (*fp).u.mh.x2258 == ftMh_MS_Wait2_0 as i32 || (*fp).u.mh.x2258 == 0x185 {
                st::Fighter_ChangeMotionState(gobj, 0x185, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            } else {
                st::Fighter_ChangeMotionState(gobj, 0x185, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                st::ftAnim_8006EBA4(gobj);
            }
            (*fp).u.mh.x2258 = 0x185;
        } else {
            if (*fp).u.mh.x2258 == ftMh_MS_Wait1_0 as i32 || (*fp).u.mh.x2258 == 0x184 {
                st::Fighter_ChangeMotionState(gobj, 0x184, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            } else {
                st::Fighter_ChangeMotionState(gobj, 0x184, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                st::ftAnim_8006EBA4(gobj);
            }
            (*fp).u.mh.x2258 = 0x184;
        }
        (*fp).mv.ch.unk0.x4 = cb;
        (*fp).mv.ch.unk0.xC = *pos;
    }
}

pub fn ftCh_TagCancel_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagCancel_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            (*fp).u.mh.x2258 = ftMh_MS_TagCancel as i32;
            st::Fighter_ChangeMotionState(gobj, ftMh_MS_TagCancel as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
        }
    }
}

/// `static inline ftCh_UpdateBossMotion(gobj, fp, da)` (inlines.h).
pub fn ftCh_UpdateBossMotion(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *const ftCrazyHand_DatAttrs) {
    melee_core::ct!("ftCh_UpdateBossMotion");
    unsafe {
        st::ft_80085134(gobj);
        let pos = &*(&raw const (*fp).mv.ch.unk0.xC);
        let out = &mut *(&raw mut (*fp).mv.ch.unk0.x18);
        bl::ftBossLib_8015BE40(gobj, pos, out, (*da).x14, (*da).x10);
    }
}

pub fn ftCh_TagCancel_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagCancel_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        ftCh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftCh_TagCancel_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_TagCancel_Coll");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).mv.ch.unk0.x18 == 0.0 {
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            (*fp).self_vel.x = 0.0;
            if let Some(cb) = (*fp).mv.ch.unk0.x4 {
                cb(gobj);
            }
        }
    }
}

pub fn ftCh_GrabUnk1_8015BC88(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_GrabUnk1_8015BC88");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        (*fp).mv.ch.unk0.x20 = 0;
        let pos = Vec3 { x: (*da).x18, y: (*da).x1C, z: 0.0 };
        (*fp).u.mh.x2258 = 0x184;
        if (*fp).u.mh.x2258 == 0x156 {
            ftCh_GrabUnk1_8015B998(gobj);
        } else {
            ftCh_GrabUnk1_8015B8FC(gobj);
        }
        (*fp).mv.ch.unk0.x4 = Some(wait10_init_cb);
        (*fp).mv.ch.unk0.xC = pos;
    }
}
