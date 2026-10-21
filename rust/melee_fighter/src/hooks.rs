//! Calls out of group 3 into code it does not own: other groups (sfx, items...) and the PC
//! port's add-ons (`FT_SCRIPT_VALUE` / `ScriptGame_ModValue`, Geno). Each is a settable
//! function pointer whose default is the vanilla behaviour, so a vanilla match needs none set.
//!
//! Single-threaded game state, like the C globals they replace: set them before the match.
#![allow(static_mut_refs)]

use crate::types::{ftCommonData, Fighter};

/// `ScriptGame_ModValue(player_id, field, base)` behind `FT_SCRIPT_VALUE` (ft/forward.h).
pub static mut SCRIPT_MOD_VALUE: Option<fn(player_id: i32, field: i32, base: f32) -> f32> = None;
/// `Geno_FallLimit(fp, terminal)` (ftCommon_Fall, TARGET_PC).
pub static mut GENO_FALL_LIMIT: Option<fn(fp: &Fighter, terminal: f32) -> f32> = None;
/// `ft_PlaySFX(fp, id, volume, pan)` (ft/ft_0877.c; sound is group 1).
pub static mut PLAY_SFX: Option<fn(fp: &mut Fighter, id: i32, vol: i32, pan: i32)> = None;
/// `ft_GetGroundFrictionMultiplier(fp)` (ft/ft_0877.c, reads the stage's floor material: group 2).
pub static mut GROUND_FRICTION_MUL: Option<fn(fp: &Fighter) -> f32> = None;

/// `p_ftCommonData` (fighter.h): PlCo's common fighter constants, loaded from the DAT.
/// TODO(move-to-core): owned by the fighter loader once ftdata is ported.
pub static mut P_FT_COMMON_DATA: *const ftCommonData = core::ptr::null();

/// `FT_SCRIPT_VALUE(fp, field, base)`.
#[inline]
pub fn script_value(fp: &Fighter, field: i32, base: f32) -> f32 {
    match unsafe { SCRIPT_MOD_VALUE } {
        Some(f) => f(fp.player_id as i32, field, base),
        None => base,
    }
}

#[inline]
pub fn geno_fall_limit(fp: &Fighter, terminal: f32) -> f32 {
    match unsafe { GENO_FALL_LIMIT } {
        Some(f) => f(fp, terminal),
        None => terminal,
    }
}

#[inline]
pub fn play_sfx(fp: &mut Fighter, id: i32, vol: i32, pan: i32) {
    if let Some(f) = unsafe { PLAY_SFX } {
        f(fp, id, vol, pan)
    }
}

#[inline]
pub fn ground_friction_mul(fp: &Fighter) -> f32 {
    match unsafe { GROUND_FRICTION_MUL } {
        Some(f) => f(fp),
        None => 1.0,
    }
}

/// `*p_ftCommonData`. Panics if the common data has not been installed.
#[inline]
pub fn common_data() -> &'static ftCommonData {
    let p = unsafe { P_FT_COMMON_DATA };
    assert!(!p.is_null(), "p_ftCommonData not loaded");
    unsafe { &*p }
}
