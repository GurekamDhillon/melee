//! src/melee/ft/kinds/ftNess/ftnessspeciallw.c: Ness down B (PSI Magnet).
//!
//! One fused site, `MTXDegToRad(180 / x78) - rotZ` in the Turn animation's `getAttrStuff`
//! (tools/rust-port/fma_sites.py), written with [`fmadds`]. Smoke-tested through recording hooks
//! by tests/tu_ftnessspeciallw.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da;
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
#[allow(unused_imports)]
use melee_types::{ftNessAttributes};
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2 and 5): re-exported under this TU.


/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    crate::port_hook!(EFASYNC_SPAWN, efAsync_Spawn, crate::adapt::efAsync_Spawn_plain, (gobj: *mut HSD_GObj, queue_head: *mut c_void, kind: u32, gfx_id: u32, jobj: *mut HSD_JObj));
    crate::port_hook!(FTCOLL_CREATEABSORBHIT, ftColl_CreateAbsorbHit, crate::adapt::ftColl_CreateAbsorbHit, (gobj: *mut HSD_GObj, desc: *mut AbsorbDesc));
    pub static mut FTPARTSETROTY: Option<fn(*mut Fighter, i32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn ftPartSetRotY(fp: *mut Fighter, part: i32, v: f32) {
        match unsafe { FTPARTSETROTY } {
            Some(f) => f(fp, part, v),
            None => { unsafe { melee_fighter::tu::ftparts::ftPartSetRotY(&*(fp as *const _), part as _, v as _); } },
        }
    }
    pub static mut FTPARTGETROTZ: Option<fn(*mut Fighter, i32) -> f32> = None;
    #[allow(non_snake_case)]
    pub fn ftPartGetRotZ(fp: *mut Fighter, part: i32) -> f32 {
        match unsafe { FTPARTGETROTZ } {
            Some(f) => f(fp, part),
            None => { unsafe { melee_fighter::tu::ftparts::ftPartGetRotZ(&*(fp as *const _), part as _) as _ } },
        }
    }
    pub static mut PLAYER_SETHPBYINDEX: Option<fn(i32, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn Player_SetHPByIndex(slot: i32, is_sub: i32, hp: i32) {
        match unsafe { PLAYER_SETHPBYINDEX } {
            Some(f) => f(slot, is_sub, hp),
            None => { unsafe { melee_fighter::tu::player::Player_SetHPByIndex(slot as _, is_sub as _, hp as _); } },
        }
    }
    pub static mut PL_80040B8C: Option<fn(i32, i32, i32) > = None;
            #[allow(non_snake_case)]
            pub fn pl_80040B8C(slot: i32, is_sub: i32, amount: i32)  {
                match unsafe { PL_80040B8C } {
                    Some(f) => f(slot, is_sub, amount),
                    None => { unsafe { melee_fighter::tu::plbonuslib::pl_80040B8C(slot as _, is_sub as _, amount as _); } return; },
                }
            }
}

/// `FTNESS_SPECIALLW_COLL_FLAG`.
pub const FTNESS_SPECIALLW_COLL_FLAG: u32 = Ft_MF_KeepGfx
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27;

/// `FTNESS_SPECIALLW_END_FLAG`.
pub const FTNESS_SPECIALLW_END_FLAG: u32 = Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn attrs(fp: *mut Fighter) -> *const ftNessAttributes {
    (*fp).dat_attrs as *const ftNessAttributes
}

#[inline(always)]
fn null() -> *mut HSD_GObj {
    core::ptr::null_mut()
}

/// Start of the move: reset the timers from the attributes (shared by both Start enters).
unsafe fn start_vars(fp: *mut Fighter) {
    let a = attrs(fp);
    // (s32) x74_PSI_MAGNET_RELEASE_LAG: float -> int truncation.
    (*fp).mv.ns.speciallw.releaseLag = (*a).x74_PSI_MAGNET_RELEASE_LAG as i32;
    (*fp).mv.ns.speciallw.isRelease = 0;
    (*fp).mv.ns.speciallw.gravityDelay = (*a).x84_PSI_MAGNET_FRAMES_BEFORE_GRAVITY;
    (*fp).mv.ns.speciallw.x10 = 0;
}

pub fn ftNs_SpecialLwStart_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwStart_Enter");
    unsafe {
        let fp = fpp(gobj);
        start_vars(fp);
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialLwStart as i32, 0, 0.0, 1.0, 0.0, null());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftNs_SpecialAirLwStart_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwStart_Enter");
    unsafe {
        let fp = fpp(gobj);
        let a = attrs(fp);
        start_vars(fp);
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x /= (*a).x88_PSI_MAGNET_MOMENTUM_PRESERVATION;
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialAirLwStart as i32, 0, 0.0, 1.0, 0.0, null());
        st::ftAnim_8006EBA4(gobj);
    }
}

/// Spawn the magnet shimmer once (`x2219_b0`) and re-arm the hitlag effect callbacks.
unsafe fn magnet_effect(fp: *mut Fighter, gobj: *mut HSD_GObj) {
    if (*fp).x2219_b0() == 0 {
        let joint = (*(*fp).parts.add(FtPart_L1stNb as usize)).joint as *mut HSD_JObj;
        ext::efAsync_Spawn(gobj, &mut (*fp).x60C as *mut *mut c_void as *mut c_void, 0, 1264, joint);
        (*fp).set_x2219_b0(1);
    }
    melee_fighter::tu::ftCo_AttackS4::Fighter_SetEffectHitlagCallbacks(&mut *fp);
}

/// Shared Start animation of both mediums: latch B release, then enter Hold.
unsafe fn start_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if (*fp).input.held_buttons[0] as i32 & HSD_PAD_B == 0 {
        (*fp).mv.ns.speciallw.isRelease = 1;
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        magnet_effect(fp, gobj);
        (*fp).mv.ns.speciallw.x10 = 0;
        if (*fp).ground_or_air as i32 == GA_Ground as i32 {
            ftNs_SpecialLwHold_Enter(gobj);
            return;
        }
        ftNs_SpecialAirLwHold_Enter(gobj);
    }
}

pub fn ftNs_SpecialLwStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwStart_Anim");
    unsafe { start_anim(gobj) }
}

pub fn ftNs_SpecialAirLwStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwStart_Anim");
    unsafe { start_anim(gobj) }
}

pub fn ftNs_SpecialLwStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialLwStart_IASA");}

pub fn ftNs_SpecialAirLwStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialAirLwStart_IASA");}

/// Shared aerial physics: gravity after the delay, quick air deceleration. `ftColl_8007AF10` is
/// called by every phase except Start and End.
unsafe fn air_phys(gobj: *mut HSD_GObj, coll: bool) {
    let fp = fpp(gobj);
    let a = attrs(fp);
    let gravity_timer = (*fp).mv.ns.speciallw.gravityDelay;
    if gravity_timer != 0 {
        (*fp).mv.ns.speciallw.gravityDelay = gravity_timer - 1;
    } else {
        st::ftCommon_Fall(&mut *fp, (*a).x8C_PSI_MAGNET_FALL_ACCEL, (*fp).co_attrs.terminal_velocity);
    }
    melee_fighter::ftcommon::calc_self_accel_deaccel_quick_air(&mut *fp);
    if coll {
        st::ftColl_8007AF10(gobj);
    }
}

pub fn ftNs_SpecialLwStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwStart_Coll");
    if st::ft_80082708(gobj) == 0 {
        ftNs_SpecialLwStart_GroundToAir(gobj);
    }
}

pub fn ftNs_SpecialAirLwStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwStart_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftNs_SpecialAirLwStart_AirToGround(gobj);
    }
}

pub fn ftNs_SpecialLwStart_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwStart_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftNs_MS_SpecialAirLwStart as i32, FTNESS_SPECIALLW_COLL_FLAG);
    }
}

pub fn ftNs_SpecialAirLwStart_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwStart_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftNs_MS_SpecialLwStart as i32, FTNESS_SPECIALLW_COLL_FLAG);
        st::ftCommon_ClampAirDrift(&mut *fp);
    }
}

/// Shared Hold animation: latch B release, count the minimum hold down, leave to End when both
/// hold, and play the hum every 40 frames.
unsafe fn hold_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if (*fp).input.held_buttons[0] as i32 & HSD_PAD_B == 0 {
        (*fp).mv.ns.speciallw.isRelease = 1;
    }
    if (*fp).mv.ns.speciallw.releaseLag > 0 {
        (*fp).mv.ns.speciallw.releaseLag -= 1;
    }
    if (*fp).mv.ns.speciallw.releaseLag <= 0 && (*fp).mv.ns.speciallw.isRelease != 0 {
        if (*fp).ground_or_air as i32 == GA_Ground as i32 {
            ftNs_SpecialLwEnd_Enter(gobj);
        } else {
            ftNs_SpecialAirLwEnd_Enter(gobj);
        }
    }
    magnet_hum(fpp(gobj));
}

/// `MagnetStateVarCalc`: tick the hum counter; at 0 play the loop sound and reload 40.
unsafe fn magnet_hum(fp: *mut Fighter) {
    (*fp).mv.ns.speciallw.x10 -= 1;
    if (*fp).mv.ns.speciallw.x10 <= 0 {
        melee_fighter::ft0881::ft_80088478(&mut *fp, 210081, 127, 64);
        (*fp).mv.ns.speciallw.x10 = 40;
    }
}

pub fn ftNs_SpecialLwHold_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwHold_Anim");
    unsafe { hold_anim(gobj) }
}

pub fn ftNs_SpecialAirLwHold_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwHold_Anim");
    unsafe { hold_anim(gobj) }
}

pub fn ftNs_SpecialLwHold_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialLwHold_IASA");}

pub fn ftNs_SpecialAirLwHold_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialAirLwHold_IASA");}

pub fn ftNs_SpecialLwHold_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwHold_Phys");
    st::ft_80084F3C(gobj);
    st::ftColl_8007AF10(gobj);
}

pub fn ftNs_SpecialAirLwHold_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwHold_Phys");
    unsafe { air_phys(gobj, true) }
}

pub fn ftNs_SpecialLwHold_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwHold_Coll");
    if st::ft_80082708(gobj) == 0 {
        ftNs_SpecialLwHold_GroundToAir(gobj);
    }
}

pub fn ftNs_SpecialAirLwHold_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwHold_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftNs_SpecialAirLwHold_AirToGround(gobj);
    }
}

/// Re-create the magnet's absorb hitbox after a motion change.
unsafe fn create_absorb_hit(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let a = (*fp).dat_attrs as *mut ftNessAttributes;
    ext::ftColl_CreateAbsorbHit(gobj, &mut (*a).x98_PSI_MAGNET_ABSORPTION);
}

pub fn ftNs_SpecialLwHold_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwHold_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftNs_MS_SpecialAirLwHold as i32, FTNESS_SPECIALLW_COLL_FLAG);
        create_absorb_hit(gobj);
    }
}

pub fn ftNs_SpecialAirLwHold_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwHold_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftNs_MS_SpecialLwHold as i32, FTNESS_SPECIALLW_COLL_FLAG);
        st::ftCommon_ClampAirDrift(&mut *fp);
        create_absorb_hit(gobj);
    }
}

pub fn ftNs_SpecialLwHold_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwHold_Enter");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialLwHold as i32, Ft_MF_KeepGfx, 0.0, 1.0, 0.0, null());
        create_absorb_hit(gobj);
    }
}

pub fn ftNs_SpecialAirLwHold_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwHold_Enter");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialAirLwHold as i32, Ft_MF_KeepGfx, 0.0, 1.0, 0.0, null());
        create_absorb_hit(gobj);
    }
}

/// `getAttrStuff` (static inline): count the turn down, flip facing at the half-way frame and spin
/// the model about Y. `MTXDegToRad(a)` is `a * 0.01745329252f`; the subtraction after it fuses.
unsafe fn turn_step(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let a = attrs(fp);
    (*fp).mv.ns.speciallw.turnFrames += -1;
    if (*fp).cmd_vars[0] == 0 && ((*fp).mv.ns.speciallw.turnFrames as f32) <= (*a).x78_PSI_MAGNET_UNK1 {
        (*fp).cmd_vars[0] = 1;
        (*fp).facing_dir = -(*fp).facing_dir;
    }
    let rot_z = ext::ftPartGetRotZ(fp, 0);
    ext::ftPartSetRotY(fp, 0, -turn_rot_y(180.0 / (*a).x78_PSI_MAGNET_UNK1, rot_z));
}

/// `MTXDegToRad(deg) - rot_z` as clang fused it.
#[inline(always)]
pub fn turn_rot_y(deg: f32, rot_z: f32) -> f32 {
    melee_core::ct!("turn_rot_y");
    fmadds(deg, 0.01745329252_f32, -rot_z)
}

/// Shared Turn animation.
unsafe fn turn_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if (*fp).input.held_buttons[0] as i32 & HSD_PAD_B == 0 {
        (*fp).mv.ns.speciallw.isRelease = 1;
    }
    if (*fp).mv.ns.speciallw.releaseLag > 0 {
        (*fp).mv.ns.speciallw.releaseLag -= 1;
    }
    turn_step(gobj);
    if (*fp).mv.ns.speciallw.turnFrames <= 0 {
        ftNs_SpecialLwHold_GroundOrAir(gobj);
    }
}

pub fn ftNs_SpecialLwTurn_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwTurn_Anim");
    unsafe { turn_anim(gobj) }
}

pub fn ftNs_SpecialAirLwTurn_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwTurn_Anim");
    unsafe { turn_anim(gobj) }
}

pub fn ftNs_SpecialLwTurn_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialLwTurn_IASA");}

pub fn ftNs_SpecialAirLwTurn_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialAirLwTurn_IASA");}

pub fn ftNs_SpecialLwTurn_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwTurn_Phys");
    st::ft_80084F3C(gobj);
    st::ftColl_8007AF10(gobj);
}

pub fn ftNs_SpecialAirLwTurn_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwTurn_Phys");
    unsafe { air_phys(gobj, true) }
}

pub fn ftNs_SpecialLwTurn_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwTurn_Coll");
    if st::ft_80082708(gobj) == 0 {
        ftNs_SpecialLwTurn_GroundToAir(gobj);
    }
}

pub fn ftNs_SpecialAirLwTurn_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwTurn_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftNs_SpecialAirLwTurn_AirToGround(gobj);
    }
}

pub fn ftNs_SpecialLwTurn_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwTurn_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftNs_MS_SpecialAirLwTurn as i32, FTNESS_SPECIALLW_COLL_FLAG);
    }
}

pub fn ftNs_SpecialAirLwTurn_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwTurn_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftNs_MS_SpecialLwTurn as i32, FTNESS_SPECIALLW_COLL_FLAG);
        st::ftCommon_ClampAirDrift(&mut *fp);
    }
}

/// Decide grounded or aerial Hold after a Turn/Hit; returns C `bool` (false = went to End).
pub fn ftNs_SpecialLwHold_GroundOrAir(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftNs_SpecialLwHold_GroundOrAir");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.ns.speciallw.releaseLag <= 0 && (*fp).mv.ns.speciallw.isRelease != 0 {
            if (*fp).ground_or_air as i32 == GA_Ground as i32 {
                ftNs_SpecialLwEnd_Enter(gobj);
            } else {
                ftNs_SpecialAirLwEnd_Enter(gobj);
            }
            return false;
        }
        let msid = if (*fp).ground_or_air as i32 == GA_Ground as i32 {
            ftNs_MS_SpecialLwHold
        } else {
            ftNs_MS_SpecialAirLwHold
        };
        st::Fighter_ChangeMotionState(gobj, msid as i32, Ft_MF_KeepGfx, 0.0, 1.0, 0.0, null());
        create_absorb_hit(gobj);
        true
    }
}

/// Shared Hit (absorb) animation: when the motion ends return to Hold (or End), re-arm the effect.
unsafe fn hit_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if (*fp).input.held_buttons[0] as i32 & HSD_PAD_B == 0 {
        (*fp).mv.ns.speciallw.isRelease = 1;
    }
    if (*fp).mv.ns.speciallw.releaseLag > 0 {
        (*fp).mv.ns.speciallw.releaseLag -= 1;
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 && ftNs_SpecialLwHold_GroundOrAir(gobj) {
        st::ftCommon_8007DB24(gobj);
        magnet_effect(fpp(gobj), gobj);
    }
    magnet_hum(fpp(gobj));
}

pub fn ftNs_SpecialLwHit_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwHit_Anim");
    unsafe { hit_anim(gobj) }
}

pub fn ftNs_SpecialAirLwHit_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwHit_Anim");
    unsafe { hit_anim(gobj) }
}

pub fn ftNs_SpecialLwHit_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialLwHit_IASA");}

pub fn ftNs_SpecialAirLwHit_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialAirLwHit_IASA");}

pub fn ftNs_SpecialLwHit_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwHit_Phys");
    st::ft_80084F3C(gobj);
    st::ftColl_8007AF10(gobj);
}

pub fn ftNs_SpecialAirLwHit_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwHit_Phys");
    unsafe { air_phys(gobj, true) }
}

pub fn ftNs_SpecialLwHit_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwHit_Coll");
    if st::ft_80082708(gobj) == 0 {
        ftNs_SpecialLwHit_GroundToAir(gobj);
    }
}

pub fn ftNs_SpecialAirLwHit_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwHit_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftNs_SpecialAirLwHit_AirToGround(gobj);
    }
}

pub fn ftNs_SpecialLwHit_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwHit_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftNs_MS_SpecialAirLwHit as i32, FTNESS_SPECIALLW_COLL_FLAG);
        create_absorb_hit(gobj);
    }
}

pub fn ftNs_SpecialAirLwHit_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwHit_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftNs_MS_SpecialLwHit as i32, FTNESS_SPECIALLW_COLL_FLAG);
        st::ftCommon_ClampAirDrift(&mut *fp);
        create_absorb_hit(gobj);
    }
}

/// PSI Magnet `OnAbsorb` hook: heal Ness by the absorbed damage, face the hit and restart Hit.
pub fn ftNs_AbsorbThink_DecideAction(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_AbsorbThink_DecideAction");
    unsafe {
        let fp = fpp(gobj);
        let sa = attrs(fp);

        // (int) (damageTaken * heal): float -> int truncation, then back to float.
        let heal = crate::chars::ftNess::ftness::ftNs_Init_GetAbsorbHeal(&*fp);
        let mut heal_amount = ((*fp).AbsorbAttr.x1A44_damageTaken as f32 * heal) as i32 as f32;

        (*fp).dmg.x1830_percent -= heal_amount;
        let cur_percent = (*fp).dmg.x1830_percent;
        if (*fp).dmg.x1830_percent < 0.0 {
            heal_amount = cur_percent + heal_amount;
            (*fp).dmg.x1830_percent = 0.0;
        }

        ext::Player_SetHPByIndex((*fp).player_id as i32, (*fp).is_sub_fighter() as i32, (*fp).dmg.x1830_percent as i32);
        ext::pl_80040B8C((*fp).player_id as i32, (*fp).is_sub_fighter() as i32, heal_amount as i32);

        (*fp).facing_dir = (*fp).AbsorbAttr.x1A40_absorbHitDirection;
        let msid = (*fp).motion_id as i32;

        if (msid != ftNs_MS_SpecialLwHit as i32 && msid != ftNs_MS_SpecialAirLwHit as i32)
            || !((*fp).cur_anim_frame <= (*sa).x7C_PSI_MAGNET_UNK2)
        {
            let msid = if (*fp).ground_or_air as i32 == GA_Ground as i32 {
                ftNs_MS_SpecialLwHit
            } else {
                ftNs_MS_SpecialAirLwHit
            };
            st::Fighter_ChangeMotionState(gobj, msid as i32, 2, 0.0, 1.0, 0.0, null());
            create_absorb_hit(gobj);
        }
    }
}

pub fn ftNs_SpecialLwEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCommon_8007DB24(gobj);
        st::ftCommon_8007D92C(gobj);
    }
}

pub fn ftNs_SpecialAirLwEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCommon_8007DB24(gobj);
        st::ftCommon_8007D92C(gobj);
    }
}

pub fn ftNs_SpecialLwEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialLwEnd_IASA");}

pub fn ftNs_SpecialAirLwEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialAirLwEnd_IASA");}

pub fn ftNs_SpecialLwEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwEnd_Phys");
    st::ft_80084F3C(gobj);
}

pub fn ftNs_SpecialAirLwEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwEnd_Phys");
    unsafe { air_phys(gobj, false) }
}

pub fn ftNs_SpecialLwEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwEnd_Coll");
    if st::ft_80082708(gobj) == 0 {
        ftNs_SpecialLwEnd_GroundToAir(gobj);
    }
}

pub fn ftNs_SpecialAirLwEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwEnd_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftNs_SpecialAirLwEnd_AirToGround(gobj);
    }
}

pub fn ftNs_SpecialLwEnd_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwEnd_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftNs_MS_SpecialAirLwEnd as i32, FTNESS_SPECIALLW_END_FLAG);
    }
}

pub fn ftNs_SpecialAirLwEnd_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwEnd_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftNs_MS_SpecialLwEnd as i32, FTNESS_SPECIALLW_END_FLAG);
        st::ftCommon_ClampAirDrift(&mut *fp);
    }
}

pub fn ftNs_SpecialLwEnd_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwEnd_Enter");
    st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialLwEnd as i32, 0, 0.0, 1.0, 0.0, null());
}

pub fn ftNs_SpecialAirLwEnd_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwEnd_Enter");
    st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialAirLwEnd as i32, 0, 0.0, 1.0, 0.0, null());
}

/// `ftNs_SpecialAirLwStart_Phys(gobj)`: the retail motion-state callback; the body is the fighter-only port `chars::ftNess::ftnessspeciallw::ftNs_SpecialAirLwStart_Phys_fp`.
pub fn ftNs_SpecialAirLwStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirLwStart_Phys");
    unsafe { crate::chars::ftNess::ftnessspeciallw::ftNs_SpecialAirLwStart_Phys_fp(&mut *((*gobj).user_data as *mut Fighter)) }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftNs_SpecialLwStart_Phys` (src/melee/ft/kinds/ftNess/ftnessspeciallw.c).
pub fn ftNs_SpecialLwStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialLwStart_Phys");
    st::ft_80084F3C(gobj);
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftNs_SpecialAirLwStart_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftNs_SpecialAirLwStart_Phys_fp");
    let a: &ftNessAttributes = da(fp);
    let gravity_timer = unsafe { fp.mv.ns.speciallw.gravityDelay } as i32;
    if gravity_timer != 0 {
        unsafe { fp.mv.ns.speciallw.gravityDelay = (gravity_timer - 1) as _ };
    } else {
        co::fall(fp, a.x8C_PSI_MAGNET_FALL_ACCEL, fp.co_attrs.terminal_velocity);
    }
    co::calc_self_accel_deaccel_quick_air(fp);
}
