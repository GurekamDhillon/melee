//! src/melee/ft/kinds/ftLuigi/ftluigispecials.c: Green Missile (side B). Fused sites
//! (tools/rust-port/fma_sites.py): the charge damage `chargeFrames * slope + tilt` in the four
//! Anim callbacks that hit (`ftLg_SpecialS_Anim`, `AirS`, `SMisfire`, `AirSMisfire`), and the two
//! launch velocities in `ftLg_SpecialSFly_Enter` (`x28 * chargeFrames + x24`,
//! `0.5 * vel_y + vel_y * (...)`), all fmadds. Checked against the C by tests/tu_ftluigispecials.rs
//! (damage handed to ftColl_8007ABD0, SetVars, SFly_Enter velocities, AirS2 and End physics, over
//! recording hooks on both sides); the rest is smoke-tested.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::ftcommon as fc;
use melee_types::*;

/// Already ported (rounds 2, 3, 5 and the pure helper): re-exported under this TU.
pub use crate::chars::ftLuigi::ftluigispecials::ftLg_SpecialAirSStart_Phys;
pub use crate::chars::r2::ftLuigi::ftluigispecials::{
    ftLg_SpecialAirSEnd_Anim, ftLg_SpecialAirSHold_AirToGround, ftLg_SpecialAirSHold_Phys,
    ftLg_SpecialAirSLaunch_AirToGround, ftLg_SpecialAirSMisfire_AirToGround, ftLg_SpecialAirSMisfire_Phys,
    ftLg_SpecialAirSStart_AirToGround, ftLg_SpecialAirS_Phys, ftLg_SpecialSEnd_Anim, ftLg_SpecialSEnd_Coll,
    ftLg_SpecialSHold_GroundToAir, ftLg_SpecialSHold_Phys, ftLg_SpecialSLaunch_GroundToAir,
    ftLg_SpecialSMisfire_GroundToAir, ftLg_SpecialSMisfire_Phys, ftLg_SpecialSStart_GroundToAir, ftLg_SpecialS_Phys,
};
pub use crate::chars::r3::ftLuigi::ftluigispecials::{
    ftLg_SpecialAirSHold_Coll, ftLg_SpecialAirSMisfire_Coll, ftLg_SpecialAirSStart_Coll, ftLg_SpecialAirS_Coll,
    ftLg_SpecialSHold_Coll, ftLg_SpecialSMisfire_Coll, ftLg_SpecialSStart_Coll, ftLg_SpecialS_Coll,
};
pub use crate::chars::r5::ftLuigi::ftluigispecials::ftLuigi_SpecialS_RemoveGFX;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a native port behind them (the port runs when the hook is unset).
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
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing arguments as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    hook!(FTCOLL_8007ABD0, ftColl_8007ABD0, (hit: *mut HitCapsule, damage_amount: u32, gobj: *mut HSD_GObj));
    // `HSD_Randi(n)` (baselib/random.c).
    hook_or!(HSD_RANDI, HSD_Randi, melee_fighter::hooks::hsd_randi, (n: i32) -> i32);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

// ---- Fighter callback entries (extern "C") for the functions this TU stores in Fighter ----

/// `ftLg_SpecialS_SetVars` as a Fighter callback entry (`x21EC`).
pub unsafe extern "C" fn cb_ftLg_SpecialS_SetVars(g: *mut HSD_GObj) {
    ftLg_SpecialS_SetVars(g);
}
/// `ftLg_SpecialS_SetGFX` as a Fighter callback entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_ftLg_SpecialS_SetGFX(g: *mut HSD_GObj) {
    ftLg_SpecialS_SetGFX(g);
}
/// `ftLg_SpecialS_OnGiveDamage` as a Fighter callback entry (`deal_dmg_cb`).
pub unsafe extern "C" fn cb_ftLg_SpecialS_OnGiveDamage(g: *mut HSD_GObj) {
    ftLg_SpecialS_OnGiveDamage(g);
}
/// `ftCommon_8007F76C(gobj)` as a Fighter callback entry (`x21F8`).
pub unsafe extern "C" fn cb_ftCommon_8007F76C(g: *mut HSD_GObj) {
    melee_fighter::ftcommon2::ftcommon_8007f76c(&mut *fpp(g));
}

// ---- motion-state hand-over flags ----

const transition_flags0: u32 = (Ft_MF_KeepColAnimHitStatus
    | Ft_MF_SkipMatAnim
    | Ft_MF_KeepSfx
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipColAnim
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;
const transition_flags1: u32 = transition_flags0 | Ft_MF_KeepGfx as u32;
const transition_flags2: u32 = (Ft_MF_KeepGfx
    | Ft_MF_KeepColAnimHitStatus
    | Ft_MF_SkipMatAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipColAnim
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;
const transition_flags3: u32 = (Ft_MF_KeepGfx
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipColAnim
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

// ---- setup ----

/// Luigi's Green Missile GFX callback.
pub fn ftLg_SpecialS_SetGFX(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
    }
}

/// Luigi's Green Missile Setup (RNG + calculations).
pub fn ftLg_SpecialS_SetVars(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).cmd_vars[0] = 0;
        if ((*fp).active_sticky.lstick.x as f32) < (*sa).x4_LUIGI_GREENMISSILE_SMASH {
            (*fp).mv.lg.SpecialS.chargeFrames = (*sa).x8_LUIGI_GREENMISSILE_CHARGE_RATE as i32;
            (*fp).x2070.anon_0.set_count_thrown_items(1);
        } else {
            (*fp).mv.lg.SpecialS.chargeFrames = 0;
        }
        if ext::HSD_Randi((*sa).x44_LUIGI_GREENMISSILE_MISFIRE_CHANCE as i32) == 0 {
            (*fp).mv.lg.SpecialS.isMisfire = 1;
        } else {
            (*fp).mv.lg.SpecialS.isMisfire = 0;
        }
    }
}

/// Luigi's grounded Green Missile Motion State handler.
pub fn ftLg_SpecialS_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).x21EC = Some(cb_ftLg_SpecialS_SetVars);
        (*fp).gr_vel /= (*sa).x18_LUIGI_GREENMISSILE_TRACTION;
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialSStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

/// Luigi's aerial Green Missile Motion State handler.
pub fn ftLg_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).x21EC = Some(cb_ftLg_SpecialS_SetVars);
        (*fp).self_vel.x /= (*sa).x18_LUIGI_GREENMISSILE_TRACTION;
        (*fp).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialAirSStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

/// Luigi's Green Missile OnGiveDamage callback.
pub fn ftLg_SpecialS_OnGiveDamage(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).self_vel.x = 0.0;
        if (*fp).self_vel.y >= 0.0 {
            (*fp).self_vel.y = 0.0;
        }
        ftLg_SpecialAirSEnd_Enter(gobj);
    }
}

// ---- SStart ----

pub fn ftLg_SpecialSStart_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftLg_SpecialSHold_Enter(gobj);
    }
}

pub fn ftLg_SpecialAirSStart_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftLg_SpecialAirSHold_Enter(gobj);
    }
}

pub fn ftLg_SpecialSStart_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftLg_SpecialAirSStart_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftLg_SpecialSStart_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        fc::calc_ground_accel_deaccel(&mut *fp, (*sa).x1C_LUIGI_GREENMISSILE_UNK2);
        fc::set_self_movement_from_grounded_movement(&mut *fp);
    }
}

// ---- SHold ----

pub fn ftLg_SpecialSHold_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCommon_8007DB24(gobj);
            (*fp).accessory4_cb = Some(cb_ftLg_SpecialS_SetGFX);
        }
        (*fp).mv.lg.SpecialS.chargeFrames = (*fp).mv.lg.SpecialS.chargeFrames.wrapping_add(1);
        if (*fp).mv.lg.SpecialS.chargeFrames as f32 > (*sa).xC_LUIGI_GREENMISSILE_MAX_CHARGE_FRAMES {
            ftLg_SpecialSLaunch_Enter(gobj);
        }
    }
}

pub fn ftLg_SpecialAirSHold_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCommon_8007DB24(gobj);
            (*fp).accessory4_cb = Some(cb_ftLg_SpecialS_SetGFX);
        }
        (*fp).mv.lg.SpecialS.chargeFrames = (*fp).mv.lg.SpecialS.chargeFrames.wrapping_add(1);
        if (*fp).mv.lg.SpecialS.chargeFrames as f32 > (*sa).xC_LUIGI_GREENMISSILE_MAX_CHARGE_FRAMES {
            ftLg_SpecialAirSLaunch_Enter(gobj);
        }
    }
}

pub fn ftLg_SpecialSHold_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if ((*fp).input.held_buttons[0] as i32 & HSD_PAD_B) == 0 {
            ftLg_SpecialSLaunch_Enter(gobj);
        }
    }
}

pub fn ftLg_SpecialAirSHold_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if ((*fp).input.held_buttons[0] as i32 & HSD_PAD_B) == 0 {
            ftLg_SpecialAirSLaunch_Enter(gobj);
        }
    }
}

/// Luigi's grounded Green Missile Charge Motion State handler.
pub fn ftLg_SpecialSHold_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialSHold as i32, Ft_MF_KeepSfx as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fpp(gobj)).accessory4_cb = Some(cb_ftLg_SpecialS_SetGFX);
    }
}

/// Luigi's aerial Green Missile Charge Motion State handler.
pub fn ftLg_SpecialAirSHold_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialAirSHold as i32, Ft_MF_KeepSfx as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fpp(gobj)).accessory4_cb = Some(cb_ftLg_SpecialS_SetGFX);
    }
}

// ---- SLaunch (the S states) ----

/// The charge damage handed to `ftColl_8007ABD0`: `chargeFrames * slope + tilt` (a fused site),
/// converted to `u32` like the C's argument.
#[inline(always)]
pub fn charge_damage(charge_frames: i32, slope: f32, tilt: f32) -> u32 {
    fmadds(charge_frames as f32, slope, tilt) as u32
}

/// Shared hit-damage update of the Launch / Misfire Anim callbacks.
unsafe fn anim_charge_damage(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
    if (*fp).mv.lg.SpecialS.isMisfire == 0 && (*fp).x914[0].state == HitCapsule_Enabled {
        ext::ftColl_8007ABD0(
            &mut (*fp).x914[0],
            charge_damage(
                (*fp).mv.lg.SpecialS.chargeFrames,
                (*sa).x14_LUIGI_GREENMISSILE_DAMAGE_SLOPE,
                (*sa).x10_LUIGI_GREENMISSILE_DAMAGE_TILT,
            ),
            gobj,
        );
    }
}

/// Luigi's grounded Green Missile Launch Animation callback.
pub fn ftLg_SpecialS_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        anim_charge_damage(gobj, fp);
        if (*fp).cmd_vars[0] != 0 {
            st::ftCommon_8007D5D4(&mut *fp);
            ftLg_SpecialSFly_Enter(gobj);
        }
    }
}

/// Luigi's aerial Green Missile Launch Animation callback.
pub fn ftLg_SpecialAirS_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        anim_charge_damage(gobj, fp);
        if (*fp).cmd_vars[0] != 0 {
            ftLg_SpecialSFly_Enter(gobj);
        }
    }
}

pub fn ftLg_SpecialS_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftLg_SpecialAirS_IASA(_gobj: *mut HSD_GObj) {}

/// `ftLuigi_SpecialS_Setup` (static inline).
pub fn ftLuigi_SpecialS_Setup(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            ext::efSync_Spawn(
                0x50A,
                gobj,
                &[(*(*fp).parts.offset(FtPart_HipN as isize)).joint as *mut core::ffi::c_void],
            );
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

/// Luigi's grounded Green Missile Launch Motion State handler.
pub fn ftLg_SpecialSLaunch_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.lg.SpecialS.isMisfire != 0 {
            ftLg_SpecialSMisfire_Enter(gobj);
            return;
        }
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialS as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftLuigi_SpecialS_RemoveGFX(gobj);
        ftLuigi_SpecialS_Setup(gobj);
    }
}

/// Luigi's aerial Green Missile Launch Motion State handler.
pub fn ftLg_SpecialAirSLaunch_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.lg.SpecialS.isMisfire != 0 {
            ftLg_SpecialAirSMisfire_Enter(gobj);
            return;
        }
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialAirS as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftLuigi_SpecialS_RemoveGFX(gobj);
        ftLuigi_SpecialS_Setup(gobj);
    }
}

// ---- SMisfire ----

pub fn ftLg_SpecialSMisfire_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        anim_charge_damage(gobj, fp);
        if (*fp).cmd_vars[0] != 0 {
            st::ftCommon_8007D5D4(&mut *fp);
            ftLg_SpecialSFly_Enter(gobj);
        }
    }
}

pub fn ftLg_SpecialAirSMisfire_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        anim_charge_damage(gobj, fp);
        if (*fp).cmd_vars[0] != 0 {
            ftLg_SpecialSFly_Enter(gobj);
        }
    }
}

pub fn ftLg_SpecialSMisfire_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftLg_SpecialAirSMisfire_IASA(_gobj: *mut HSD_GObj) {}

/// Luigi's grounded Green Missile Misfire Motion State handler.
pub fn ftLg_SpecialSMisfire_Enter(gobj: *mut HSD_GObj) {
    st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialSMisfire as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftLuigi_SpecialS_RemoveGFX(gobj);
    ftLuigi_SpecialS_Setup(gobj);
}

/// Luigi's aerial Green Missile Misfire Motion State handler.
pub fn ftLg_SpecialAirSMisfire_Enter(gobj: *mut HSD_GObj) {
    st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialAirSMisfire as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftLuigi_SpecialS_RemoveGFX(gobj);
    ftLuigi_SpecialS_Setup(gobj);
}

// ---- SFly (S2) ----

pub fn ftLg_SpecialS2_Anim(_gobj: *mut HSD_GObj) {}

pub fn ftLg_SpecialAirS2_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftLg_SpecialAirSEnd_Enter(gobj);
    }
}

pub fn ftLg_SpecialS2_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftLg_SpecialAirS2_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftLg_SpecialS2_Phys(_gobj: *mut HSD_GObj) {}

pub fn ftLg_SpecialAirS2_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        if (*fp).cmd_vars[0] != 0 {
            fc::fall(&mut *fp, (*sa).x40_LUIGI_GREENMISSILE_GRAVITY_MUL, (*sa).x34_LUIGI_GREENMISSILE_GRAVITY_START);
        } else {
            fc::fall(&mut *fp, (*sa).x30_LUIGI_GREENMISSILE_MUL_Y, (*sa).x34_LUIGI_GREENMISSILE_GRAVITY_START);
        }
        if (*fp).cmd_vars[0] != 0 {
            fc::calc_self_accel_deaccel(&mut *fp, (*sa).x3C_LUIGI_GREENMISSILE_X_DECEL);
        }
    }
}

pub fn ftLg_SpecialS2_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftLg_SpecialAirS2_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp0 = fpp(gobj);
        if st::ft_80081D0C(gobj) != 0 {
            st::ftCommon_8007D7FC(&mut *fp0);
            ftLg_SpecialSEnd_Enter(gobj);
        }
        let env_flags = (*fp0).coll_data.env_flags as u32;
        if env_flags & Collide_LeftWallMask as u32 != 0 || env_flags & Collide_RightWallMask as u32 != 0 {
            ftLg_SpecialAirSEnd_Enter(gobj);
        }
    }
}

/// Luigi's Green Missile Fly Setup.
pub fn ftLg_SpecialSFly_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).cmd_vars[0] = 0;
        if (*fp).mv.lg.SpecialS.isMisfire != 0 {
            (*fp).self_vel.x = (*sa).x48_LUIGI_GREENMISSILE_MISFIRE_VEL_X;
        } else {
            (*fp).self_vel.x = fmadds(
                (*sa).x28_LUIGI_GREENMISSILE_MUL_X,
                (*fp).mv.lg.SpecialS.chargeFrames as f32,
                (*sa).x24_LUIGI_GREENMISSILE_VEL_X,
            );
        }
        (*fp).self_vel.x *= (*fp).facing_dir;
        if (*fp).mv.lg.SpecialS.isMisfire != 0 {
            (*fp).self_vel.y = (*sa).x4C_LUIGI_GREENMISSILE_MISFIRE_VEL_Y;
        } else {
            let vel_y = (*sa).x2C_LUIGI_GREENMISSILE_VEL_Y;
            (*fp).self_vel.y = fmadds(
                0.5,
                vel_y,
                vel_y * (0.5f32 * (*fp).mv.lg.SpecialS.chargeFrames as f32 / (*sa).xC_LUIGI_GREENMISSILE_MAX_CHARGE_FRAMES),
            );
        }
        st::Fighter_ChangeMotionState(
            gobj,
            ftLg_MS_SpecialAirS2 as i32,
            transition_flags3,
            (*fp).cur_anim_frame,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        (*fp).x21F8 = Some(cb_ftCommon_8007F76C);
        (*fp).deal_dmg_cb = Some(cb_ftLg_SpecialS_OnGiveDamage);
    }
}

// ---- SEnd ----

pub fn ftLg_SpecialSEnd_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftLg_SpecialAirSEnd_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftLg_SpecialSEnd_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        fc::calc_ground_accel_deaccel(&mut *fp, (*sa).x3C_LUIGI_GREENMISSILE_X_DECEL);
        fc::set_self_movement_from_grounded_movement(&mut *fp);
    }
}

pub fn ftLg_SpecialAirSEnd_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        let terminal = (*fp).co_attrs.terminal_velocity;
        fc::fall(&mut *fp, (*sa).x40_LUIGI_GREENMISSILE_GRAVITY_MUL, terminal);
        fc::calc_self_accel_deaccel(&mut *fp, (*sa).x3C_LUIGI_GREENMISSILE_X_DECEL);
    }
}

pub fn ftLg_SpecialAirSEnd_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != 0 {
            st::ftCommon_8007D7FC(&mut *fp);
            ftLg_SpecialSEnd_Enter(gobj);
        }
    }
}

/// Luigi's Green Missile End air -> ground Motion State handler.
pub fn ftLg_SpecialSEnd_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).cmd_vars[0] = 0;
        (*fp).gr_vel /= (*sa).x38_LUIGI_GREENMISSILE_FRICTION_END;
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialSEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}

/// Luigi's Green Missile End ground -> air Motion State handler.
pub fn ftLg_SpecialAirSEnd_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).cmd_vars[0] = 0;
        (*fp).self_vel.x /= (*sa).x38_LUIGI_GREENMISSILE_FRICTION_END;
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialAirSEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}
