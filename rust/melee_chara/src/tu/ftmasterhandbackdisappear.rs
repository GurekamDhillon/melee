//! src/melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c: BackDisappear, Wait1_1, Grab, Fail
//! and Cancel states.
//!
//! Fused sites (tools/rust-port/fma_sites.py): `my_lbVector_Len` `x*x + y*y + z*z` ->
//! fmadds(z, z, fmadds(x, x, y*y)); `my_sqrtf` is crate::pure::my_sqrtf (three f64 Newton
//! steps, fnmsub fused). The Wait1_1 Phys velocity is checked bit for bit against the C by
//! tests/tu_ftmasterhandbackdisappear.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
#[allow(unused_imports)]
use crate::tu::ftmasterhandwait11::ftMh_MS_375_80154C78;
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/8/9/10): re-exported under this TU.






/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    /// `ftMh_MS_388_80155A58(gobj, gobj2)` (ftmasterhandtagcancel.c).
    pub static mut FTMH_MS_388_80155A58: Option<fn(*mut HSD_GObj, *mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftMh_MS_388_80155A58(gobj: *mut HSD_GObj, gobj2: *mut HSD_GObj) {
        match unsafe { FTMH_MS_388_80155A58 } {
            Some(f) => f(gobj, gobj2),
            None => { unsafe { crate::tu::ftmasterhandtagcancel::ftMh_MS_388_80155A58(gobj as _, gobj2 as _); } return; },
        }
    }
    /// `ftMh_MS_388_80155A58` as the `grabbed_cb` (`HSD_GObjInteraction`) value.
    pub unsafe extern "C" fn cb_ftMh_MS_388_80155A58(gobj: *mut HSD_GObj, gobj2: *mut HSD_GObj) {
        ftMh_MS_388_80155A58(gobj, gobj2)
    }
    /// `ftMh_CaptureWaitMasterHand_80155D1C(gobj)` (ftmasterhandthrownmasterhand.c).
    pub static mut FTMH_CAPTUREWAITMASTERHAND_80155D1C: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftMh_CaptureWaitMasterHand_80155D1C(gobj: *mut HSD_GObj) {
        match unsafe { FTMH_CAPTUREWAITMASTERHAND_80155D1C } {
            Some(f) => f(gobj),
            None => { unsafe { crate::tu::ftmasterhandthrownmasterhand::ftMh_CaptureWaitMasterHand_80155D1C(gobj as _); } return; },
        }
    }
}
pub use crate::tu::ftmasterhandbackairplane3::ext::ftMh_MS_389_80151018;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftMh_UpdateBossMotion(gobj, fp, da)` (ftMasterHand/inlines.h, static inline).
pub fn ftMh_UpdateBossMotion(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *const ftMasterHand_SpecialAttrs) {
    melee_core::ct!("ftMh_UpdateBossMotion");
    unsafe {
        st::ft_80085134(gobj);
        let target = (*fp).mv.mh.unk0.xC;
        melee_fighter::tu::ftbosslib::ftBossLib_8015BE40(gobj, &target, &mut (*fp).mv.mh.unk0.x18, (*da).x2C, (*da).x28);
    }
}

/// `my_lbVector_Len` (static inline).
#[inline]
pub fn my_lbVector_Len(v: &Vec3) -> f32 {
    melee_core::ct!("my_lbVector_Len");
    crate::pure::my_sqrtf(fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y)))
}

/// The arithmetic of `ftMh_Wait1_1_Phys` after `ftBossLib_8015C208`: the (self_vel.x, self_vel.y)
/// chasing `target` (the other fighter's position) from `cur`, with the offsets/gain of `da`.
pub fn wait1_1_vel(target: &Vec3, cur: &Vec3, da: &ftMasterHand_SpecialAttrs) -> (f32, f32) {
    melee_core::ct!("wait1_1_vel");
    let mut pos = *target;
    pos.x += da.x108;
    pos.y += da.x10C;
    pos.z = 0.0;
    let mut vel = Vec3 { x: pos.x - cur.x, y: pos.y - cur.y, z: pos.z - cur.z };
    let len = my_lbVector_Len(&vel);
    if len < da.x2C {
        (vel.x, vel.y)
    } else {
        let mut w = melee_core::vec::Vec3 { x: vel.x, y: vel.y, z: vel.z };
        melee_runtime::lbvector::normalize(&mut w);
        vel = Vec3 { x: w.x, y: w.y, z: w.z };
        let speed = len * da.x28;
        vel.x *= speed;
        vel.y *= speed;
        (vel.x, vel.y)
    }
}

/// `ftMh_MS_375_80154A2C` as a Fighter callback entry (`grab_cb`).
pub unsafe extern "C" fn cb_ftMh_MS_375_80154A2C(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMh_MS_375_80154A2C");
    crate::chars::r9::ftMasterHand::ftmasterhandbackdisappear::ftMh_MS_375_80154A2C(g)
}

pub fn ftMh_BackDisappear_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackDisappear_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = fpp(gobj);
            if let Some(cb) = (*fp).mv.mh.unk0.x4 {
                cb(gobj);
            }
        }
    }
}

pub fn ftMh_BackDisappear_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackDisappear_IASA");
    unsafe {
        let fp = fpp(gobj);
        if st::Player_GetPlayerSlotType((*fp).player_id as i32) == 0 {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

pub fn ftMh_BackDisappear_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_BackDisappear_Coll");}

pub fn ftMh_MS_372_801542E0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_372_801542E0");
    unsafe {
        let fp = fpp(gobj);
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::Fighter_ChangeMotionState(gobj, 373, 0, (*fp).cur_anim_frame as f32, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_SetAnimRate(gobj, (*da).x110_pos.y);
        (*fp).mv.mh.unk0.x8 = melee_core::ppc::fctiwz_f32((*da).x110_pos.x);
    }
}

pub fn ftMh_Wait1_1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Wait1_1_Anim");
    unsafe {
        let fp = fpp(gobj);
        let timer = (*fp).mv.mh.unk0.x8.wrapping_sub(1);
        let is_zero = timer == 0;
        (*fp).mv.mh.unk0.x8 = timer;
        if is_zero {
            st::ftAnim_SetAnimRate(gobj, 1.0);
            ftMh_MS_373_801545A0(gobj);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_ChangeMotionState(gobj, 373, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
        }
    }
}

pub fn ftMh_Wait1_1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Wait1_1_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::ft_80085134(gobj);
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        melee_fighter::tu::ftbosslib::ftBossLib_8015C208(gobj, &mut pos);
        let cur = (*fp).cur_pos;
        let (vx, vy) = wait1_1_vel(&pos, &cur, &*da);
        (*fp).self_vel.x = vx;
        (*fp).self_vel.y = vy;
    }
}

pub fn ftMh_Wait1_1_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Wait1_1_Coll");}

pub fn ftMh_MS_373_801545A0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_373_801545A0");
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, 374, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        melee_fighter::ftcommon2::ftcommon_8007e2d0(
            &mut *fp,
            128,
            Some(cb_ftMh_MS_375_80154A2C),
            None,
            Some(ext::cb_ftMh_MS_388_80155A58),
        );
        (*fp).mv.mh.unk0.x20 = 0;
    }
}

pub fn ftMh_Grab_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Grab_IASA");
    unsafe {
        let fp = fpp(gobj);
        if st::Player_GetPlayerSlotType((*fp).player_id as i32) == 0 {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

pub fn ftMh_Grab_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Grab_Coll");}

pub fn ftMh_Fail_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Fail_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_Fail_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Fail_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        ftMh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftMh_Fail_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Fail_Coll");}

pub fn ftMh_Cancel_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Cancel_Anim");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.mh.unk0.x24 -= 1.0;
        if (*fp).mv.mh.unk0.x24 <= 0.0 && (*fp).cmd_vars[0] != 0 {
            ext::ftMh_CaptureWaitMasterHand_80155D1C((*fp).victim_gobj);
            (*fp).mv.mh.unk0.x20 = 0;
            (*fp).cmd_vars[0] = 0;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftMh_MS_389_80151018(gobj);
        }
    }
}

pub fn ftMh_Cancel_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Cancel_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        ftMh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftMh_Cancel_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Cancel_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.mh.unk0.x18 == 0.0 {
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            (*fp).self_vel.x = 0.0;
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_Grab_Anim` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c).
pub fn ftMh_Grab_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Grab_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            (*fp).self_vel.z = f32::from_bits(0x00000000) /* 0 */;
            (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
            (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0 */;
            crate::chars::r9::ftMasterHand::ftmasterhandwait11::ftMh_MS_378_80154CF8(gobj);
        }
    }
}

/// `ftMh_BackDisappear_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c).
pub fn ftMh_BackDisappear_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackDisappear_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_Grab_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c).
pub fn ftMh_Grab_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Grab_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_Wait1_1_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c).
pub fn ftMh_Wait1_1_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Wait1_1_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_Fail_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c).
pub fn ftMh_Fail_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Fail_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_Cancel_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c).
pub fn ftMh_Cancel_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Cancel_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_MS_374_801546D8` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c).
pub fn ftMh_MS_374_801546D8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_374_801546D8");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 381 /* ftMh_MS_Fail */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.mh.unk0.xC.x = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x30_pos2.x;
        (*fp).mv.mh.unk0.xC.y = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x30_pos2.y;
        (*fp).mv.mh.unk0.xC.z = f32::from_bits(0x00000000) /* 0 */;
    }
}

/// `ftMh_MS_381_8015483C` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c).
pub fn ftMh_MS_381_8015483C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_381_8015483C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 375 /* ftMh_MS_Cancel */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.mh.unk0.x24 = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x120;
        (*fp).cmd_vars[0] = 1_u32;
        (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.z = f32::from_bits(0x00000000) /* 0 */;
        (*fp).mv.mh.unk0.xC.x = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x30_pos2.x;
        (*fp).mv.mh.unk0.xC.y = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x30_pos2.y;
        (*fp).mv.mh.unk0.xC.z = f32::from_bits(0x00000000) /* 0 */;
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftMh_Cancel_Coll_fp(fp: &mut Fighter) {
    melee_core::ct!("ftMh_Cancel_Coll_fp");
    crate::chars::ftMasterHand::stop_if_x18_zero(fp)
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftMh_MS_375_80154A2C` (src/melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c).
pub fn ftMh_MS_375_80154A2C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_375_80154A2C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).self_vel.z = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0 */;
        (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
        (*fp).mv.mh.unk0.x20 = (1_i32 /* true */);
        (*fp).set_x221E_b6(0);
        crate::chars::r9::ftMasterHand::ftmasterhandwait11::ftMh_MS_375_80154C78(gobj);
    }
}
