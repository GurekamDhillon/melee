//! src/melee/ft/kinds/ftLuigi/ftluigispecials.c: Green Missile (side B). Fused sites
//! (tools/rust-port/fma_sites.py): the charge damage `chargeFrames * slope + tilt` in the four
//! Anim callbacks that hit (`ftLg_SpecialS_Anim`, `AirS`, `SMisfire`, `AirSMisfire`), and the two
//! launch velocities in `ftLg_SpecialSFly_Enter` (`x28 * chargeFrames + x24`,
//! `0.5 * vel_y + vel_y * (...)`), all fmadds. Checked against the C by tests/tu_ftluigispecials.rs
//! (damage handed to ftColl_8007ABD0, SetVars, SFly_Enter velocities, AirS2 and End physics, over
//! recording hooks on both sides); the rest is smoke-tested.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da;
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
#[allow(unused_imports)]
use melee_types::{ftLuigiAttributes};
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::ftcommon as fc;
use melee_types::*;

/// Already ported (rounds 2, 3, 5 and the pure helper): re-exported under this TU.




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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
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
    crate::port_hook!(FTCOLL_8007ABD0, ftColl_8007ABD0, crate::adapt::ftColl_8007ABD0, (hit: *mut HitCapsule, damage_amount: u32, gobj: *mut HSD_GObj));
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
    melee_core::ct!("cb_ftLg_SpecialS_SetVars");
    ftLg_SpecialS_SetVars(g);
}
/// `ftLg_SpecialS_SetGFX` as a Fighter callback entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_ftLg_SpecialS_SetGFX(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftLg_SpecialS_SetGFX");
    ftLg_SpecialS_SetGFX(g);
}
/// `ftLg_SpecialS_OnGiveDamage` as a Fighter callback entry (`deal_dmg_cb`).
pub unsafe extern "C" fn cb_ftLg_SpecialS_OnGiveDamage(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftLg_SpecialS_OnGiveDamage");
    ftLg_SpecialS_OnGiveDamage(g);
}
/// `ftCommon_8007F76C(gobj)` as a Fighter callback entry (`x21F8`).
pub unsafe extern "C" fn cb_ftCommon_8007F76C(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftCommon_8007F76C");
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
    melee_core::ct!("ftLg_SpecialS_SetGFX");
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
    melee_core::ct!("ftLg_SpecialS_SetVars");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).cmd_vars[0] = 0;
        if ((*fp).active_sticky.lstick.x as f32) < (*sa).x4_LUIGI_GREENMISSILE_SMASH {
            (*fp).mv.lg.SpecialS.chargeFrames = melee_core::ppc::fctiwz_f32((*sa).x8_LUIGI_GREENMISSILE_CHARGE_RATE);
            (*fp).x2070.x0.set_count_thrown_items(1);
        } else {
            (*fp).mv.lg.SpecialS.chargeFrames = 0;
        }
        if ext::HSD_Randi(melee_core::ppc::fctiwz_f32((*sa).x44_LUIGI_GREENMISSILE_MISFIRE_CHANCE)) == 0 {
            (*fp).mv.lg.SpecialS.isMisfire = 1;
        } else {
            (*fp).mv.lg.SpecialS.isMisfire = 0;
        }
    }
}

/// Luigi's grounded Green Missile Motion State handler.
pub fn ftLg_SpecialS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialS_Enter");
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
    melee_core::ct!("ftLg_SpecialAirS_Enter");
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
    melee_core::ct!("ftLg_SpecialS_OnGiveDamage");
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
    melee_core::ct!("ftLg_SpecialSStart_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftLg_SpecialSHold_Enter(gobj);
    }
}

pub fn ftLg_SpecialAirSStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSStart_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftLg_SpecialAirSHold_Enter(gobj);
    }
}

pub fn ftLg_SpecialSStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialSStart_IASA");}

pub fn ftLg_SpecialAirSStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialAirSStart_IASA");}

pub fn ftLg_SpecialSStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSStart_Phys");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        fc::calc_ground_accel_deaccel(&mut *fp, (*sa).x1C_LUIGI_GREENMISSILE_UNK2);
        fc::set_self_movement_from_grounded_movement(&mut *fp);
    }
}

// ---- SHold ----

pub fn ftLg_SpecialSHold_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSHold_Anim");
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
    melee_core::ct!("ftLg_SpecialAirSHold_Anim");
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
    melee_core::ct!("ftLg_SpecialSHold_IASA");
    unsafe {
        let fp = fpp(gobj);
        if ((*fp).input.held_buttons[0] as i32 & HSD_PAD_B) == 0 {
            ftLg_SpecialSLaunch_Enter(gobj);
        }
    }
}

pub fn ftLg_SpecialAirSHold_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSHold_IASA");
    unsafe {
        let fp = fpp(gobj);
        if ((*fp).input.held_buttons[0] as i32 & HSD_PAD_B) == 0 {
            ftLg_SpecialAirSLaunch_Enter(gobj);
        }
    }
}

/// Luigi's grounded Green Missile Charge Motion State handler.
pub fn ftLg_SpecialSHold_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSHold_Enter");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialSHold as i32, Ft_MF_KeepSfx as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fpp(gobj)).accessory4_cb = Some(cb_ftLg_SpecialS_SetGFX);
    }
}

/// Luigi's aerial Green Missile Charge Motion State handler.
pub fn ftLg_SpecialAirSHold_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSHold_Enter");
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
    melee_core::ct!("charge_damage");
    melee_core::ppc::fcvt_u32(fmadds(charge_frames as f32, slope, tilt))
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
    melee_core::ct!("ftLg_SpecialS_Anim");
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
    melee_core::ct!("ftLg_SpecialAirS_Anim");
    unsafe {
        let fp = fpp(gobj);
        anim_charge_damage(gobj, fp);
        if (*fp).cmd_vars[0] != 0 {
            ftLg_SpecialSFly_Enter(gobj);
        }
    }
}

pub fn ftLg_SpecialS_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialS_IASA");}

pub fn ftLg_SpecialAirS_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialAirS_IASA");}

/// `ftLuigi_SpecialS_Setup` (static inline).
pub fn ftLuigi_SpecialS_Setup(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLuigi_SpecialS_Setup");
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
    melee_core::ct!("ftLg_SpecialSLaunch_Enter");
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
    melee_core::ct!("ftLg_SpecialAirSLaunch_Enter");
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
    melee_core::ct!("ftLg_SpecialSMisfire_Anim");
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
    melee_core::ct!("ftLg_SpecialAirSMisfire_Anim");
    unsafe {
        let fp = fpp(gobj);
        anim_charge_damage(gobj, fp);
        if (*fp).cmd_vars[0] != 0 {
            ftLg_SpecialSFly_Enter(gobj);
        }
    }
}

pub fn ftLg_SpecialSMisfire_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialSMisfire_IASA");}

pub fn ftLg_SpecialAirSMisfire_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialAirSMisfire_IASA");}

/// Luigi's grounded Green Missile Misfire Motion State handler.
pub fn ftLg_SpecialSMisfire_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSMisfire_Enter");
    st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialSMisfire as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftLuigi_SpecialS_RemoveGFX(gobj);
    ftLuigi_SpecialS_Setup(gobj);
}

/// Luigi's aerial Green Missile Misfire Motion State handler.
pub fn ftLg_SpecialAirSMisfire_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSMisfire_Enter");
    st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialAirSMisfire as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftLuigi_SpecialS_RemoveGFX(gobj);
    ftLuigi_SpecialS_Setup(gobj);
}

// ---- SFly (S2) ----

pub fn ftLg_SpecialS2_Anim(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialS2_Anim");}

pub fn ftLg_SpecialAirS2_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirS2_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftLg_SpecialAirSEnd_Enter(gobj);
    }
}

pub fn ftLg_SpecialS2_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialS2_IASA");}

pub fn ftLg_SpecialAirS2_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialAirS2_IASA");}

pub fn ftLg_SpecialS2_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialS2_Phys");}

pub fn ftLg_SpecialAirS2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirS2_Phys");
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

pub fn ftLg_SpecialS2_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialS2_Coll");}

pub fn ftLg_SpecialAirS2_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirS2_Coll");
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
    melee_core::ct!("ftLg_SpecialSFly_Enter");
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

pub fn ftLg_SpecialSEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialSEnd_IASA");}

pub fn ftLg_SpecialAirSEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialAirSEnd_IASA");}

pub fn ftLg_SpecialSEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSEnd_Phys");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        fc::calc_ground_accel_deaccel(&mut *fp, (*sa).x3C_LUIGI_GREENMISSILE_X_DECEL);
        fc::set_self_movement_from_grounded_movement(&mut *fp);
    }
}

pub fn ftLg_SpecialAirSEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSEnd_Phys");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        let terminal = (*fp).co_attrs.terminal_velocity;
        fc::fall(&mut *fp, (*sa).x40_LUIGI_GREENMISSILE_GRAVITY_MUL, terminal);
        fc::calc_self_accel_deaccel(&mut *fp, (*sa).x3C_LUIGI_GREENMISSILE_X_DECEL);
    }
}

pub fn ftLg_SpecialAirSEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSEnd_Coll");
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
    melee_core::ct!("ftLg_SpecialSEnd_Enter");
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
    melee_core::ct!("ftLg_SpecialAirSEnd_Enter");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).cmd_vars[0] = 0;
        (*fp).self_vel.x /= (*sa).x38_LUIGI_GREENMISSILE_FRICTION_END;
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialAirSEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}

/// `ftLg_SpecialAirSStart_Phys(gobj)`: the retail motion-state callback; the body is the fighter-only port `chars::ftLuigi::ftluigispecials::ftLg_SpecialAirSStart_Phys_fp`.
pub fn ftLg_SpecialAirSStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSStart_Phys");
    unsafe { crate::chars::ftLuigi::ftluigispecials::ftLg_SpecialAirSStart_Phys_fp(&mut *((*gobj).user_data as *mut Fighter)) }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftLg_SpecialSStart_GroundToAir` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialSStart_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSStart_GroundToAir");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 349 /* ftLg_MS_SpecialAirSStart */, 0xC4C5284u32 /* transition_flags0 */);
}

/// `ftLg_SpecialAirSStart_AirToGround` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirSStart_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSStart_AirToGround");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 343 /* ftLg_MS_SpecialSStart */, 0xC4C5284u32 /* transition_flags0 */);
}

/// `ftLg_SpecialSHold_Phys` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialSHold_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSHold_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftLg_SpecialAirSHold_Phys` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirSHold_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSHold_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftLg_SpecialSHold_GroundToAir` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialSHold_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSHold_GroundToAir");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 350 /* ftLg_MS_SpecialAirSHold */, 0xC4C5286u32 /* transition_flags1 */);
}

/// `ftLg_SpecialAirSHold_AirToGround` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirSHold_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSHold_AirToGround");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 344 /* ftLg_MS_SpecialSHold */, 0xC4C5286u32 /* transition_flags1 */);
}

/// `ftLg_SpecialS_Phys` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialS_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftLg_SpecialAirS_Phys` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirS_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftLg_SpecialSLaunch_GroundToAir` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialSLaunch_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSLaunch_GroundToAir");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 353 /* ftLg_MS_SpecialAirS */, 0xC4C5086u32 /* transition_flags2 */);
}

/// `ftLg_SpecialAirSLaunch_AirToGround` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirSLaunch_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSLaunch_AirToGround");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 347 /* ftLg_MS_SpecialS */, 0xC4C5086u32 /* transition_flags2 */);
}

/// `ftLg_SpecialSMisfire_Phys` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialSMisfire_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSMisfire_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftLg_SpecialAirSMisfire_Phys` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirSMisfire_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSMisfire_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftLg_SpecialSMisfire_GroundToAir` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialSMisfire_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSMisfire_GroundToAir");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 354 /* ftLg_MS_SpecialAirSMisfire */, 0xC4C5086u32 /* transition_flags2 */);
}

/// `ftLg_SpecialAirSMisfire_AirToGround` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirSMisfire_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSMisfire_AirToGround");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 348 /* ftLg_MS_SpecialSMisfire */, 0xC4C5086u32 /* transition_flags2 */);
}

/// `ftLg_SpecialSEnd_Anim` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialSEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftLg_SpecialAirSEnd_Anim` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirSEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftLg_SpecialSEnd_Coll` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialSEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSEnd_Coll");
    if st::ft_80082708(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftLg_SpecialSStart_Coll` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialSStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSStart_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 {
            crate::chars::r2::ftLuigi::ftluigispecials::ftLg_SpecialSStart_GroundToAir(gobj);
        }
    }
}

/// `ftLg_SpecialAirSStart_Coll` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirSStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSStart_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != 0 {
            crate::chars::r2::ftLuigi::ftluigispecials::ftLg_SpecialAirSStart_AirToGround(gobj);
        }
    }
}

/// `ftLg_SpecialSHold_Coll` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialSHold_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSHold_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 {
            crate::chars::r2::ftLuigi::ftluigispecials::ftLg_SpecialSHold_GroundToAir(gobj);
        }
    }
}

/// `ftLg_SpecialAirSHold_Coll` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirSHold_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSHold_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != 0 {
            crate::chars::r2::ftLuigi::ftluigispecials::ftLg_SpecialAirSHold_AirToGround(gobj);
        }
    }
}

/// `ftLg_SpecialS_Coll` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialS_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 {
            crate::chars::r2::ftLuigi::ftluigispecials::ftLg_SpecialSLaunch_GroundToAir(gobj);
        }
    }
}

/// `ftLg_SpecialAirS_Coll` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirS_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != 0 {
            crate::chars::r2::ftLuigi::ftluigispecials::ftLg_SpecialAirSLaunch_AirToGround(gobj);
        }
    }
}

/// `ftLg_SpecialSMisfire_Coll` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialSMisfire_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialSMisfire_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 {
            crate::chars::r2::ftLuigi::ftluigispecials::ftLg_SpecialSMisfire_GroundToAir(gobj);
        }
    }
}

/// `ftLg_SpecialAirSMisfire_Coll` (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLg_SpecialAirSMisfire_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirSMisfire_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != 0 {
            crate::chars::r2::ftLuigi::ftluigispecials::ftLg_SpecialAirSMisfire_AirToGround(gobj);
        }
    }
}

/// `ftLuigi_SpecialS_RemoveGFX` (static) (src/melee/ft/kinds/ftLuigi/ftluigispecials.c).
pub fn ftLuigi_SpecialS_RemoveGFX(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLuigi_SpecialS_RemoveGFX");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[0] = 0_u32;
        st::ftCommon_8007DB24(gobj);
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftLg_SpecialAirSStart_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftLg_SpecialAirSStart_Phys_fp");
    let a: &ftLuigiAttributes = da(fp);
    if fp.cmd_vars[0] != 0 {
        co::fall(fp, a.x20_LUIGI_GREENMISSILE_FALLING_SPEED, fp.co_attrs.terminal_velocity);
    }
    co::calc_self_accel_deaccel(fp, a.x1C_LUIGI_GREENMISSILE_UNK2);
}
