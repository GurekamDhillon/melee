//! Hand port of the movement helpers in src/melee/ft/ftcommon.c (lines 60-540, 728-755).
//!
//! Bit-identical to the C: every f32 expression keeps the C operation order, `ABS` is the C
//! macro (`x < 0 ? -x : x`, so `ABS(-0.0) == -0.0`, unlike `f32::abs`), and `!x` on a float is
//! `x == 0.0`. Functions that took `HSD_GObj*` take the fighter (`gobj->user_data`) directly.
//! Checked against the C functions themselves by tests/ftcommon_ref.rs.

use crate::hooks::{common_data, geno_fall_limit, ground_friction_mul, play_sfx, script_value};
use crate::types::{ftCo_MS_Squat, Fighter, GA_Ground};

/// `ABS(x)` from Runtime/platform.h.
#[inline(always)]
pub fn abs_c(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

pub fn calc_ground_accel_deaccel(fp: &mut Fighter, mut friction: f32) {
    if abs_c(friction) > abs_c(fp.gr_vel) {
        friction = -fp.gr_vel;
    } else if fp.gr_vel > 0.0 {
        friction = -friction;
    }
    fp.xE4_ground_accel_1 = friction;
}

pub fn calc_ground_accel_dash_run(fp: &mut Fighter, mut accel: f32, target_vel: f32, friction: f32) {
    if target_vel == 0.0 {
        calc_ground_accel_deaccel(fp, friction);
        return;
    }
    if !(fp.gr_vel * accel < 0.0) {
        if accel > 0.0 {
            if fp.gr_vel + accel > target_vel {
                accel = -friction;
                if fp.gr_vel + accel < target_vel {
                    accel = target_vel - fp.gr_vel;
                }
                if fp.gr_vel + accel > script_value(fp, 2, fp.co_attrs.ground_max_horizontal_velocity) {
                    accel = script_value(fp, 2, fp.co_attrs.ground_max_horizontal_velocity) - fp.gr_vel;
                }
            }
        } else if fp.gr_vel + accel < target_vel {
            accel = friction;
            if fp.gr_vel + accel > target_vel {
                accel = target_vel - fp.gr_vel;
            }
            if fp.gr_vel + accel < -script_value(fp, 2, fp.co_attrs.ground_max_horizontal_velocity) {
                accel = -script_value(fp, 2, fp.co_attrs.ground_max_horizontal_velocity) - fp.gr_vel;
            }
        }
    }
    fp.xE4_ground_accel_1 = accel;
}

pub fn calc_ground_accel_accel_to_vel(fp: &mut Fighter, mut accel: f32, target_vel: f32, _friction: f32) {
    if target_vel == 0.0 {
        accel = -fp.gr_vel;
    } else if !(fp.gr_vel * accel < 0.0) {
        if accel > 0.0 {
            if fp.gr_vel + accel > target_vel {
                accel = target_vel - fp.gr_vel;
            }
        } else if fp.gr_vel + accel < target_vel {
            accel = target_vel - fp.gr_vel;
        }
    }
    fp.xE4_ground_accel_1 = accel;
}

pub fn calc_ground_accel_accel_to_lstick_x(fp: &mut Fighter, threshold: f32, accel_max: f32, target_max: f32) {
    let lsx = fp.input.lstick[0].x;
    let (accel, target_vel);
    if abs_c(lsx) >= threshold {
        accel = lsx * accel_max;
        target_vel = lsx * target_max;
    } else {
        target_vel = 0.0;
        accel = 0.0;
    }
    calc_ground_accel_accel_to_vel(fp, accel, target_vel, lsx);
}

fn self_movement_from_grounded(fp: &mut Fighter) {
    let n = fp.coll_data.floor.normal;
    fp.x74_self_accel.x = n.y * fp.xE4_ground_accel_1;
    fp.x74_self_accel.y = -n.x * fp.xE4_ground_accel_1;
    fp.x74_self_accel.z = 0.0;
    fp.self_vel.x = n.y * fp.gr_vel;
    fp.self_vel.y = -n.x * fp.gr_vel;
    fp.self_vel.z = 0.0;
}

/// `ftCommon_SetSelfMovementFromGroundedMovement`.
pub fn set_self_movement_from_grounded_movement(fp: &mut Fighter) {
    let ground_friction = ground_friction_mul(fp);
    if ground_friction < 1.0 {
        fp.xE4_ground_accel_1 *= ground_friction;
    }
    self_movement_from_grounded(fp);
}

pub fn set_self_movement_from_grounded_movement_no_friction(fp: &mut Fighter) {
    self_movement_from_grounded(fp);
}

pub fn clamp_ground_vel(fp: &mut Fighter, max: f32) {
    if fp.gr_vel < -max {
        fp.gr_vel = -max;
    } else if fp.gr_vel > max {
        fp.gr_vel = max;
    }
}

pub fn apply_grounded_knockback_friction(fp: &mut Fighter, friction: f32) {
    if fp.xF0_ground_kb_vel < 0.0 {
        fp.xF0_ground_kb_vel = fp.xF0_ground_kb_vel + friction;
        if fp.xF0_ground_kb_vel > 0.0 {
            fp.xF0_ground_kb_vel = 0.0;
        }
    } else {
        fp.xF0_ground_kb_vel = fp.xF0_ground_kb_vel - friction;
        if fp.xF0_ground_kb_vel < 0.0 {
            fp.xF0_ground_kb_vel = 0.0;
        }
    }
}

pub fn set_grounded_knockback_if_landed(fp: &mut Fighter) {
    if fp.ground_or_air == GA_Ground && fp.xF0_ground_kb_vel == 0.0 {
        let n = fp.coll_data.floor.normal;
        let max = common_data().max_grounded_kb_on_landing;
        fp.xF0_ground_kb_vel = fp.x8c_kb_vel.x;
        if fp.xF0_ground_kb_vel > max {
            fp.xF0_ground_kb_vel = max;
        }
        if fp.xF0_ground_kb_vel < -max {
            fp.xF0_ground_kb_vel = -max;
        }
        fp.x8c_kb_vel.x = n.y * fp.xF0_ground_kb_vel;
        fp.x8c_kb_vel.y = -n.x * fp.xF0_ground_kb_vel;
    }
}

pub fn sandbag_knockback_deaccel(kb: f32, decrement: f32) -> f32 {
    let mut result = kb;
    if kb > 0.0 {
        result -= decrement;
        if result < 0.0 {
            return 0.0;
        }
    } else if kb < 0.0 {
        result += decrement;
        if result > 0.0 {
            return 0.0;
        }
    }
    result
}

pub fn apply_shield_knockback_friction(fp: &mut Fighter, val: f32) {
    if fp.xF4_ground_attacker_shield_kb_vel < 0.0 {
        fp.xF4_ground_attacker_shield_kb_vel += val;
        if fp.xF4_ground_attacker_shield_kb_vel > 0.0 {
            fp.xF4_ground_attacker_shield_kb_vel = 0.0;
        }
    } else {
        fp.xF4_ground_attacker_shield_kb_vel -= val;
        if fp.xF4_ground_attacker_shield_kb_vel < 0.0 {
            fp.xF4_ground_attacker_shield_kb_vel = 0.0;
        }
    }
}

pub fn calc_self_accel_deaccel(fp: &mut Fighter, mut friction: f32) {
    if abs_c(friction) >= abs_c(fp.self_vel.x) {
        friction = -fp.self_vel.x;
    } else if fp.self_vel.x > 0.0 {
        friction = -friction;
    }
    fp.x74_self_accel.x = friction;
}

pub fn calc_self_accel_deaccel_air(fp: &mut Fighter) {
    let mut result = fp.co_attrs.aerial_friction;
    let absvel = abs_c(fp.self_vel.x);
    if abs_c(result) >= absvel {
        result = -fp.self_vel.x;
    } else if fp.self_vel.x > 0.0 {
        result = -fp.co_attrs.aerial_friction;
    }
    fp.x74_self_accel.x = result;
}

fn deaccel_quick(fp: &mut Fighter, max_vel: f32) -> bool {
    let vel = fp.self_vel.x;
    if abs_c(vel) > max_vel {
        let oob = common_data().aerial_friction_oob;
        let mut accel = oob;
        if abs_c(accel) >= abs_c(vel) {
            accel = -vel;
        } else if vel > 0.0 {
            accel = -oob;
        }
        fp.x74_self_accel.x = accel;
        true
    } else {
        let mut accel = fp.co_attrs.aerial_friction;
        if abs_c(accel) >= abs_c(vel) {
            accel = -vel;
        } else if vel > 0.0 {
            accel = -fp.co_attrs.aerial_friction;
        }
        fp.x74_self_accel.x = accel;
        false
    }
}

pub fn calc_self_accel_deaccel_quick_air(fp: &mut Fighter) -> bool {
    let max = script_value(fp, 3, fp.co_attrs.air_drift_max);
    deaccel_quick(fp, max)
}

pub fn calc_self_accel_deaccel_quick(fp: &mut Fighter, max_vel: f32) -> bool {
    deaccel_quick(fp, max_vel)
}

pub fn calc_self_accel_accel_to_vel_clamped(fp: &mut Fighter, accel: f32, target_vel: f32, friction: f32) {
    let vel = fp.self_vel.x;
    calc_self_accel_accel_to_vel_clamped_from(fp, vel, accel, target_vel, friction);
}

pub fn calc_self_accel_accel_to_vel_clamped_from(fp: &mut Fighter, vel: f32, mut accel: f32, target_vel: f32, friction: f32) {
    if target_vel == 0.0 {
        calc_self_accel_deaccel(fp, friction);
        return;
    }
    if !(vel * accel < 0.0) {
        if accel > 0.0 {
            if vel + accel > target_vel {
                accel = -friction;
                if vel + accel < target_vel {
                    accel = target_vel - vel;
                }
                if vel + accel > script_value(fp, 3, fp.co_attrs.air_max_horizontal_velocity) {
                    accel = script_value(fp, 3, fp.co_attrs.air_max_horizontal_velocity) - vel;
                }
            }
        } else if vel + accel < target_vel {
            accel = friction;
            if vel + accel > target_vel {
                accel = target_vel - vel;
            }
            if vel + accel < -script_value(fp, 3, fp.co_attrs.air_max_horizontal_velocity) {
                accel = -script_value(fp, 3, fp.co_attrs.air_max_horizontal_velocity) - vel;
            }
        }
    }
    fp.x74_self_accel.x = accel;
}

pub fn calc_self_accel_drift(fp: &mut Fighter) {
    let vel = fp.self_vel.x;
    calc_self_accel_drift_from(fp, vel);
}

pub fn calc_self_accel_drift_from(fp: &mut Fighter, vel: f32) {
    let lsx = fp.input.lstick[0].x;
    let accel_scaling = lsx * fp.co_attrs.air_drift_stick_mul;
    let accel_flat = if lsx > 0.0 { fp.co_attrs.aerial_drift_base } else { -fp.co_attrs.aerial_drift_base };
    let target = lsx * script_value(fp, 3, fp.co_attrs.air_drift_max);
    let friction = fp.co_attrs.aerial_friction;
    calc_self_accel_accel_to_vel_clamped_from(fp, vel, accel_scaling + accel_flat, target, friction);
}

pub fn calc_self_accel_accel_to_vel(fp: &mut Fighter, mut accel: f32, target_vel: f32, _unused: f32) {
    if target_vel == 0.0 {
        accel = -fp.self_vel.x;
    } else if !(fp.self_vel.x * accel < 0.0) {
        if accel > 0.0 {
            if fp.self_vel.x + accel > target_vel {
                accel = target_vel - fp.self_vel.x;
            }
        } else if fp.self_vel.x + accel < target_vel {
            accel = target_vel - fp.self_vel.x;
        }
    }
    fp.x74_self_accel.x = accel;
}

pub fn calc_self_accel_drift_simple(fp: &mut Fighter, threshold: f32, accel_max: f32, target_max: f32) {
    let (accel, target);
    if abs_c(fp.input.lstick[0].x) >= threshold {
        accel = fp.input.lstick[0].x * accel_max;
        target = fp.input.lstick[0].x * target_max;
    } else {
        target = 0.0;
        accel = 0.0;
    }
    let friction = fp.co_attrs.aerial_friction;
    calc_self_accel_accel_to_vel_clamped(fp, accel, target, friction);
}

pub fn calc_self_accel_drift_simple_no_friction(fp: &mut Fighter, threshold: f32, accel_max: f32, target_max: f32) {
    let (accel, target_vel);
    if abs_c(fp.input.lstick[0].x) >= threshold {
        accel = fp.input.lstick[0].x * accel_max;
        target_vel = fp.input.lstick[0].x * target_max;
    } else {
        target_vel = 0.0;
        accel = 0.0;
    }
    calc_self_accel_accel_to_vel(fp, accel, target_vel, 0.0);
}

pub fn clamp_self_vel_x(fp: &mut Fighter, max: f32) {
    let vel = fp.self_vel.x;
    if vel < -max {
        fp.self_vel.x = -max;
    } else if vel > max {
        fp.self_vel.x = max;
    }
}

pub fn clamp_air_drift(fp: &mut Fighter) {
    let max = script_value(fp, 3, fp.co_attrs.air_drift_max);
    clamp_self_vel_x(fp, max);
}

pub fn fall(fp: &mut Fighter, gravity: f32, terminal_vel: f32) {
    let terminal_vel = geno_fall_limit(fp, terminal_vel);
    fp.self_vel.y -= gravity;
    if fp.self_vel.y < -terminal_vel {
        fp.self_vel.y = -terminal_vel;
    }
}

pub fn fall_basic(fp: &mut Fighter) {
    let g = fp.co_attrs.gravity;
    let t = script_value(fp, 8, fp.co_attrs.terminal_velocity);
    fall(fp, g, t);
}

pub fn fall_fast(fp: &mut Fighter) {
    fp.self_vel.y = -fp.co_attrs.fast_fall_velocity;
}

pub fn clamp_fall_speed(fp: &mut Fighter, val: f32) {
    if fp.self_vel.y > val {
        fp.self_vel.y = val;
    }
}

pub fn ascend(fp: &mut Fighter, accel: f32, max: f32) {
    fp.self_vel.y += accel;
    if fp.self_vel.y > max {
        fp.self_vel.y = max;
    }
}

pub fn check_fall_fast(fp: &mut Fighter) -> bool {
    let cd = common_data();
    if fp.fall_fast() == 0
        && fp.self_vel.y < 0.0
        && fp.input.lstick[0].y <= -cd.x88
        && (fp.active_timer.lstick.y as i32) < cd.x8C
    {
        fp.set_fall_fast(1);
        fp.active_timer.lstick.y = 0xFE;
        play_sfx(fp, 0x96, 0x7F, 0x40);
        return true;
    }
    false
}

pub fn update_facing(fp: &mut Fighter) {
    fp.facing_dir = if fp.input.lstick[0].x >= 0.0 { 1.0 } else { -1.0 };
}

/// `ftCommon_8007DA24`: face the stick when it is outside the horizontal deadzone.
pub fn update_facing_outside_deadzone(fp: &mut Fighter) {
    if abs_c(fp.input.lstick[0].x) > common_data().horizontal_stick_deadzone {
        update_facing(fp);
    }
}

/// C `(int) f`: PPC `fctiwz` saturates, and so does Rust's `as`; NaN differs (PPC gives
/// 0x80000000, Rust 0), which no hitlag input produces.
#[inline(always)]
fn trunc_i32(f: f32) -> i32 {
    f as i32
}

pub fn calc_hitlag(dmg: i32, msid: i32, mul: f32) -> f32 {
    let cd = common_data();
    let tmp = trunc_i32(dmg as f32 * cd.x198 + cd.x19C);
    let mut result = trunc_i32(tmp as f32 * mul) as f32;
    if (msid as u32).wrapping_sub(ftCo_MS_Squat as u32) <= 1 {
        result = trunc_i32(result * cd.x1A0) as f32;
    }
    result
}
