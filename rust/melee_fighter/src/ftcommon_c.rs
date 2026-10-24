//! C-named entry points of src/melee/ft/ftcommon.c's movement helpers. The bodies live in
//! `crate::ftcommon` (checked bit for bit against the C by tests/ftcommon_ref.rs, which calls
//! these names); this module only gives them the C names and the C parameter shapes.
#![allow(non_snake_case)]
use crate::ftcommon as fc;
use crate::types::*;

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpg<'a>(g: G) -> &'a mut Fighter {
    &mut *((*g).user_data as *mut Fighter)
}

pub fn ftCommon_CalcGroundAccel_Deaccel(fp: &mut Fighter, friction: f32) { fc::calc_ground_accel_deaccel(fp, friction) }
pub fn ftCommon_CalcGroundAccel_DashRun(fp: &mut Fighter, accel: f32, target_vel: f32, friction: f32) { fc::calc_ground_accel_dash_run(fp, accel, target_vel, friction) }
pub fn ftCommon_CalcGroundAccel_AccelToVel(fp: &mut Fighter, accel: f32, target_vel: f32, friction: f32) { fc::calc_ground_accel_accel_to_vel(fp, accel, target_vel, friction) }
pub fn ftCommon_CalcGroundAccel_AccelToLStickX(fp: &mut Fighter, a: f32, b: f32, c: f32) { fc::calc_ground_accel_accel_to_lstick_x(fp, a, b, c) }
pub fn ftCommon_SetSelfMovementFromGroundedMovement(g: G) { unsafe { fc::set_self_movement_from_grounded_movement(fpg(g)) } }
pub fn ftCommon_SetSelfMovementFromGroundedMovement_NoFriction(g: G) { unsafe { fc::set_self_movement_from_grounded_movement_no_friction(fpg(g)) } }
pub fn ftCommon_ClampGroundVel(fp: &mut Fighter, max: f32) { fc::clamp_ground_vel(fp, max) }
pub fn ftCommon_ApplyGroundedKnockbackFriction(fp: &mut Fighter, f: f32) { fc::apply_grounded_knockback_friction(fp, f) }
pub fn ftCommon_SetGroundedKnockbackIfLanded(fp: &mut Fighter) { fc::set_grounded_knockback_if_landed(fp) }
pub fn ftCommon_SandbagKnockbackDeaccel(kb: f32, dec: f32) -> f32 { fc::sandbag_knockback_deaccel(kb, dec) }
pub fn ftCommon_ApplyShieldKnockbackFriction(fp: &mut Fighter, v: f32) { fc::apply_shield_knockback_friction(fp, v) }
pub fn ftCommon_CalcSelfAccel_Deaccel(fp: &mut Fighter, friction: f32) { fc::calc_self_accel_deaccel(fp, friction) }
pub fn ftCommon_CalcSelfAccel_DeaccelAir(fp: &mut Fighter) { fc::calc_self_accel_deaccel_air(fp) }
pub fn ftCommon_CalcSelfAccel_DeaccelQuickAir(fp: &mut Fighter) -> bool { fc::calc_self_accel_deaccel_quick_air(fp) }
pub fn ftCommon_CalcSelfAccel_DeaccelQuick(fp: &mut Fighter, max_vel: f32) -> bool { fc::calc_self_accel_deaccel_quick(fp, max_vel) }
pub fn ftCommon_CalcSelfAccel_AccelToVelClamped(fp: &mut Fighter, a: f32, b: f32, c: f32) { fc::calc_self_accel_accel_to_vel_clamped(fp, a, b, c) }
pub fn ftCommon_CalcSelfAccel_AccelToVelClampedFrom(fp: &mut Fighter, vel: f32, a: f32, b: f32, c: f32) { fc::calc_self_accel_accel_to_vel_clamped_from(fp, vel, a, b, c) }
pub fn ftCommon_CalcSelfAccel_Drift(fp: &mut Fighter) { fc::calc_self_accel_drift(fp) }
pub fn ftCommon_CalcSelfAccel_DriftFrom(fp: &mut Fighter, vel: f32) { fc::calc_self_accel_drift_from(fp, vel) }
pub fn ftCommon_CalcSelfAccel_AccelToVel(fp: &mut Fighter, a: f32, b: f32, c: f32) { fc::calc_self_accel_accel_to_vel(fp, a, b, c) }
pub fn ftCommon_CalcSelfAccel_DriftSimple(fp: &mut Fighter, a: f32, b: f32, c: f32) { fc::calc_self_accel_drift_simple(fp, a, b, c) }
pub fn ftCommon_CalcSelfAccel_DriftSimple_NoFriction(fp: &mut Fighter, a: f32, b: f32, c: f32) { fc::calc_self_accel_drift_simple_no_friction(fp, a, b, c) }
pub fn ftCommon_ClampSelfVelX(fp: &mut Fighter, max: f32) { fc::clamp_self_vel_x(fp, max) }
pub fn ftCommon_ClampAirDrift(fp: &mut Fighter) { fc::clamp_air_drift(fp) }
pub fn ftCommon_Fall(fp: &mut Fighter, gravity: f32, terminal_vel: f32) { fc::fall(fp, gravity, terminal_vel) }
pub fn ftCommon_FallBasic(fp: &mut Fighter) { fc::fall_basic(fp) }
pub fn ftCommon_FallFast(fp: &mut Fighter) { fc::fall_fast(fp) }
pub fn ftCommon_ClampFallSpeed(fp: &mut Fighter, v: f32) { fc::clamp_fall_speed(fp, v) }
pub fn ftCommon_Ascend(fp: &mut Fighter, accel: f32, max: f32) { fc::ascend(fp, accel, max) }
pub fn ftCommon_CheckFallFast(fp: &mut Fighter) -> bool { fc::check_fall_fast(fp) }
pub fn ftCommon_UpdateFacing(fp: &mut Fighter) { fc::update_facing(fp) }
pub fn ftCommon_8007DA24(fp: &mut Fighter) { fc::update_facing_outside_deadzone(fp) }
pub fn ftCommon_CalcHitlag(dmg: i32, msid: i32, mul: f32) -> f32 { fc::calc_hitlag(dmg, msid, mul) }
/// `ftCo_GetLStickAngle`: `atan2f(lstick.y, ABS(lstick.x))`.
pub fn ftCo_GetLStickAngle(fp: &Fighter) -> f32 { crate::ftcommon3::ftco_get_lstick_angle(fp) }
