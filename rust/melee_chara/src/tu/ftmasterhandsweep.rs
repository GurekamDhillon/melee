//! src/melee/ft/kinds/ftMasterHand/ftmasterhandsweep.c: Master Hand damage / sweep states.
//! No fused sites (plain f32 adds and subtracts). Smoke-tested by tests/tu_ftmasterhandsweep.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.
pub use crate::chars::r2::ftMasterHand::ftmasterhandsweep::{ftMh_MS_344_80151828, ftMh_MS_347_80151AC8, ftMh_WaitSweep_Phys};
pub use crate::chars::r8::ftMasterHand::ftmasterhandsweep::{ftMh_Damage_IASA, ftMh_SweepLoop_IASA, ftMh_WaitSweep_IASA};

/// Calls into TUs that are not ported yet (settable hooks; unset they panic).
pub mod ext {
    use melee_types::*;
    pub static mut MPFLOORGETLEFT: Option<fn(i32, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn mpFloorGetLeft(line: i32, out: *mut Vec3) {
        match unsafe { MPFLOORGETLEFT } {
            Some(f) => f(line, out),
            None => { unsafe { let _ = melee_world::tu::mplib::mpFloorGetLeft(line as _, out as _); } },
        }
    }
}

pub fn ftMh_Damage_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        (*fp).self_vel.y += (*da).x150;
        (*fp).self_vel.z += (*da).x158;
    }
}

pub fn ftMh_Damage_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftMh_WaitSweep_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftMh_MS_346_80151918(gobj);
    }
}

pub fn ftMh_WaitSweep_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftMh_MS_346_80151918(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        (*fp).mv.mh.unk0.xC.x = (*fp).cur_pos.x - (*da).x3C;
        (*fp).mv.mh.unk0.xC.y = (*da).x38;
        (*fp).mv.mh.unk0.xC.z = 0.0;
        st::Fighter_ChangeMotionState(gobj, 347 /* ftMh_MS_SweepLoop */, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftMh_SweepLoop_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            let mut pos: Vec3 = core::mem::zeroed();
            ext::mpFloorGetLeft(0, &mut pos);
            if (*fp).cur_pos.x < pos.x {
                ftMh_MS_347_80151AC8(gobj);
                return;
            }
            ftMh_MS_346_80151918(gobj);
        }
    }
}

pub fn ftMh_SweepLoop_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        let xc = (*fp).mv.mh.unk0.xC;
        melee_fighter::tu::ftbosslib::ftBossLib_8015BE40(gobj, &xc, &mut (*fp).mv.mh.unk0.x18, (*da).x2C, (*da).x28);
    }
}

pub fn ftMh_SweepLoop_Coll(_gobj: *mut HSD_GObj) {}
