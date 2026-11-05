//! src/melee/ft/kinds/ftMars/ftmarsspecialn.c: Shield Breaker (neutral B).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function; the End-anim
//! damage is int arithmetic converted to float once). Smoke-tested by tests/tu_ftmarsspecialn.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da as da_g;
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
#[allow(unused_imports)]
use melee_types::{MarsAttributes};
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftmars::{cb_ftMs_SpecialN_8013666C, cb_ftMs_SpecialN_801365A8};
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 11 and the pure helpers): re-exported under this TU.



/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    pub use crate::tu::ftcaptainspecialn::ext::{lb_800119DC, LB_800119DC};
    pub use crate::tu::ftcrazyhandfingerbeam0::ext::{lb_8000B1CC, LB_8000B1CC};
    pub use crate::tu::ftmarsspeciallw::ext::{ftColl_8007ABD0, FTCOLL_8007ABD0};
}

/// `MTXDegToRad(60)`: `60 * 0.01745329252f` in float.
const DEG60: f32 = 60.0f32 * 0.01745329252f32;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const MarsAttributes {
    (*fp).dat_attrs as *const MarsAttributes
}
#[inline(always)]
unsafe fn hip_joint(fp: *mut Fighter) -> *mut HSD_JObj {
    let idx = fh::ftparts_get_bone_index(&*fp, FtPart_HipN as i32);
    (*(*fp).parts.offset(idx as isize)).joint
}

/// `ftMs_SpecialN_80136730` as a Fighter callback entry (`x21EC`).
pub unsafe extern "C" fn cb_ftMs_SpecialN_80136730(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMs_SpecialN_80136730");
    crate::tu::ftmars::ftMs_SpecialN_80136730(g);
}

pub fn ftMs_SpecialN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).x21EC = Some(cb_ftMs_SpecialN_80136730);
        (*fp).gr_vel /= (*da).specialn_friction;
        st::Fighter_ChangeMotionState(gobj, ftMs_MS_SpecialNStart as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftMs_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirN_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).x21EC = Some(cb_ftMs_SpecialN_80136730);
        (*fp).self_vel.x /= (*da).specialn_friction;
        if (*fp).self_vel.y <= 0.0 {
            (*fp).self_vel.y = 0.0;
        }
        st::Fighter_ChangeMotionState(gobj, ftMs_MS_SpecialAirNStart as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `doStartAnim` (static inline).
fn doStartAnim(gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj)) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        cb(gobj);
        unsafe {
            let fp = fpp(gobj);
            if melee_fighter::ft0892::template_kind(&*fp) == Ft_Kind_Mars as i32 {
                melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *fp, 99, 0);
            } else {
                melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *fp, 100, 0);
            }
        }
    }
}

pub fn ftMs_SpecialNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialNStart_Anim");
    doStartAnim(gobj, ftMs_SpecialN_80136E74);
}

pub fn ftMs_SpecialAirNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirNStart_Anim");
    doStartAnim(gobj, ftMs_SpecialN_80136EAC);
}

pub fn ftMs_SpecialNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMs_SpecialNStart_IASA");}

pub fn ftMs_SpecialAirNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMs_SpecialAirNStart_IASA");}

/// The pure physics live in `chars::ftMars::ftmarsspecialn` (taking the fighter); these are the
/// callback entries.
pub fn ftMs_SpecialNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialNStart_Phys");
    unsafe { crate::chars::ftMars::ftmarsspecialn::ftMs_SpecialNStart_Phys(&mut *fpp(gobj)) }
}

pub fn ftMs_SpecialAirNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirNStart_Phys");
    unsafe { crate::chars::ftMars::ftmarsspecialn::ftMs_SpecialAirNStart_Phys(&mut *fpp(gobj)) }
}

/// `doLoopAnim` (static inline): charge counter, a rumble-ish camera pulse every 30 frames, and
/// release once the charge time (`x0` seconds) is up.
fn doLoopAnim(gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj)) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        let mut pos: Vec3 = core::mem::zeroed();
        if (*fp).mv.ms.specialn.cur_frame % 30 == 0 {
            ext::lb_8000B1CC(hip_joint(fp), core::ptr::null(), &mut pos);
            ext::lb_800119DC(&mut pos, 10, 0.5f64 as f32, 0.05f64 as f32, DEG60);
        }
        (*fp).mv.ms.specialn.cur_frame = (*fp).mv.ms.specialn.cur_frame.wrapping_add(1);
        if (*fp).mv.ms.specialn.cur_frame > (*da).x0.wrapping_mul(30) {
            (*fp).cmd_vars[0] = 1;
            cb(gobj);
        }
    }
}

pub fn ftMs_SpecialNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialNLoop_Anim");
    doLoopAnim(gobj, ftMs_SpecialN_80137354);
}

pub fn ftMs_SpecialAirNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirNLoop_Anim");
    doLoopAnim(gobj, ftMs_SpecialN_801373B8);
}

/// `doLoopIasa` (static inline): releasing B ends the charge.
fn doLoopIasa(gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj)) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
            (*fp).cmd_vars[0] = 0;
            cb(gobj);
        }
    }
}

pub fn ftMs_SpecialNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialNLoop_IASA");
    doLoopIasa(gobj, ftMs_SpecialN_80137354);
}

pub fn ftMs_SpecialAirNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirNLoop_IASA");
    doLoopIasa(gobj, ftMs_SpecialN_801373B8);
}

/// `inlineA0` (static inline): the release scales live hitboxes by the charge; frame 9 pulses.
fn inlineA0(gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj)) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[0] == 0 {
            for i in 0..4 {
                if (*fp).x914[i].state == HitCapsule_Enabled {
                    let dmg: i32 = (*da).x4.wrapping_add(((*fp).mv.ms.specialn.cur_frame / 30).wrapping_mul((*da).x8));
                    // int -> float -> u32 parameter (fctiwz on the GC)
                    ext::ftColl_8007ABD0(&mut (*fp).x914[i], melee_core::ppc::fctiwz_f32(dmg as f32) as u32, gobj);
                }
            }
        }
        if (*fp).cur_anim_frame == 9.0 {
            let mut position: Vec3 = core::mem::zeroed();
            ext::lb_8000B1CC(hip_joint(fp), core::ptr::null(), &mut position);
            ext::lb_800119DC(&mut position, 120, 0.9f32, 0.02f32, DEG60);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            cb(gobj);
        }
    }
}

pub fn ftMs_SpecialNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialNEnd_Anim");
    inlineA0(gobj, |g| st::ft_8008A2BC(g));
}

pub fn ftMs_SpecialAirNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirNEnd_Anim");
    inlineA0(gobj, |g| st::ftCo_Fall_Enter(g));
}

pub fn ftMs_SpecialNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMs_SpecialNEnd_IASA");}

pub fn ftMs_SpecialAirNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMs_SpecialAirNEnd_IASA");}

pub fn ftMs_SpecialNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialNEnd_Coll");
    if st::ft_80082708(gobj) == GA_Ground as i32 {
        ftMs_SpecialN_801371FC(gobj);
    }
}

pub fn ftMs_SpecialAirNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirNEnd_Coll");
    if st::ft_80081D0C(gobj) != GA_Ground as i32 {
        ftMs_SpecialN_801372A8(gobj);
    }
}

/// `ftMs_SpecialN_801371FC` / `801372A8` motion flags (raw bit mask in the C).
const MF_END: u32 = (1 << 1) | (1 << 2) | (1 << 3) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);

pub fn ftMs_SpecialN_801371FC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_801371FC");
    unsafe {
        let fp = fpp(gobj);
        let msid = if (*fp).cmd_vars[0] == 0 { ftMs_MS_SpecialAirNEnd0 } else { ftMs_MS_SpecialAirNEnd1 };
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, msid as i32, MF_END);
        if (*fp).x2219_b0() == 1 {
            (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
            (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
        }
    }
}

pub fn ftMs_SpecialN_801372A8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_801372A8");
    unsafe {
        let fp = fpp(gobj);
        let msid = if (*fp).cmd_vars[0] == 0 { ftMs_MS_SpecialNEnd0 } else { ftMs_MS_SpecialNEnd1 };
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, msid as i32, MF_END);
        if (*fp).x2219_b0() == 1 {
            (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
            (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
        }
    }
}

/// Charge released on the ground: enter the End state (starting on frame 1) and arm the effect.
pub fn ftMs_SpecialN_80137354(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_80137354");
    unsafe {
        let fp = fpp(gobj);
        let msid = if (*fp).cmd_vars[0] == 0 { ftMs_MS_SpecialNEnd0 } else { ftMs_MS_SpecialNEnd1 };
        st::Fighter_ChangeMotionState(gobj, msid as i32, Ft_MF_None as u32, 1.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(cb_ftMs_SpecialN_801365A8);
    }
}

pub fn ftMs_SpecialN_801373B8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_801373B8");
    unsafe {
        let fp = fpp(gobj);
        let msid = if (*fp).cmd_vars[0] == 0 { ftMs_MS_SpecialAirNEnd0 } else { ftMs_MS_SpecialAirNEnd1 };
        st::Fighter_ChangeMotionState(gobj, msid as i32, Ft_MF_None as u32, 1.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(cb_ftMs_SpecialN_8013666C);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMs_SpecialNStart_Coll` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialNStart_Coll");
    unsafe {
        if (st::ft_80082708(gobj) == 0) {
            crate::chars::r11::ftMars::ftmarsspecialn::ftMs_SpecialN_80136A1C(gobj);
        }
    }
}

/// `ftMs_SpecialAirNStart_Coll` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialAirNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirNStart_Coll");
    unsafe {
        if (st::ft_80081D0C(gobj) != 0) {
            crate::chars::r11::ftMars::ftmarsspecialn::ftMs_SpecialN_80136A7C(gobj);
        }
    }
}

/// `ftMs_SpecialN_80136A1C` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialN_80136A1C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_80136A1C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 345 /* ftMs_MS_SpecialAirNStart */, 0xC4C5084u32 /* ((u32) ((1 << 2) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27))) */);
    }
}

/// `ftMs_SpecialN_80136A7C` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialN_80136A7C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_80136A7C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 341 /* ftMs_MS_SpecialNStart */, 0xC4C5084u32 /* ((u32) ((1 << 2) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27))) */);
    }
}

/// `ftMs_SpecialNLoop_Coll` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialNLoop_Coll");
    unsafe {
        if (st::ft_80082708(gobj) == 0 /* GA_Ground */) {
            crate::chars::r11::ftMars::ftmarsspecialn::ftMs_SpecialN_80136DB4(gobj);
        }
    }
}

/// `ftMs_SpecialAirNLoop_Coll` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialAirNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirNLoop_Coll");
    unsafe {
        if (st::ft_80081D0C(gobj) != 0 /* GA_Ground */) {
            crate::chars::r11::ftMars::ftmarsspecialn::ftMs_SpecialN_80136E14(gobj);
        }
    }
}

/// `ftMs_SpecialN_80136DB4` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialN_80136DB4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_80136DB4");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 346 /* ftMs_MS_SpecialAirNLoop */, 0xC4C5A86u32 /* ((u32) ((1 << 1) | (1 << 2) | (1 << 7) | (1 << 9) | (1 << 11) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27))) */);
    }
}

/// `ftMs_SpecialN_80136E14` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialN_80136E14(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_80136E14");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 342 /* ftMs_MS_SpecialNLoop */, 0xC4C5A86u32 /* ((u32) ((1 << 1) | (1 << 2) | (1 << 7) | (1 << 9) | (1 << 11) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27))) */);
    }
}

/// `ftMs_SpecialNLoop_Phys` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialNLoop_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftMs_SpecialAirNLoop_Phys` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialAirNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirNLoop_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftMs_SpecialN_80136E74` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialN_80136E74(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_80136E74");
    st::Fighter_ChangeMotionState(gobj, 342 /* ftMs_MS_SpecialNLoop */, 0x1200u32 /* Ft_MF_KeepSfx | Ft_MF_SkipColAnim */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
}

/// `ftMs_SpecialN_80136EAC` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialN_80136EAC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialN_80136EAC");
    st::Fighter_ChangeMotionState(gobj, 346 /* ftMs_MS_SpecialAirNLoop */, 0x1200u32 /* Ft_MF_KeepSfx | Ft_MF_SkipColAnim */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
}

/// `ftMs_SpecialNEnd_Phys` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialNEnd_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftMs_SpecialAirNEnd_Phys` (src/melee/ft/kinds/ftMars/ftmarsspecialn.c).
pub fn ftMs_SpecialAirNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirNEnd_Phys");
    st::ft_80084EEC(gobj);
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftMs_SpecialAirNStart_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftMs_SpecialAirNStart_Phys_fp");
    let a: &MarsAttributes = da_g(fp);
    co::fall_basic(fp);
    co::calc_self_accel_deaccel(fp, a.specialn_start_friction);
}

pub fn ftMs_SpecialNStart_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftMs_SpecialNStart_Phys_fp");
    let a: &MarsAttributes = da_g(fp);
    co::calc_ground_accel_deaccel(fp, a.specialn_start_friction);
    co::set_self_movement_from_grounded_movement(fp);
}
