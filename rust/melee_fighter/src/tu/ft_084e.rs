//! ft_084E.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::ftcommon as fc;
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use crate::types::*;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use melee_runtime::gekko::{cosf, sinf};
#[allow(unused_imports)]
use crate::ft084e::{G, fall_or_fast, fp, x594_b0};

// ---- folded from ft084e.rs (consolidation steps 21/22) ----
/// `ft_80084E1C(gobj, threshold, drift_max, target_max)`.
pub fn ft_80084e1c(g: G, threshold: f32, drift_max: f32, target_max: f32) {
    melee_core::ct!("ft_80084e1c");
    let f = fp(g);
    fall_or_fast(f);
    let x = f.input.lstick[0].x;
    let (drift, target_vel) = if fc::abs_c(x) >= threshold { (x * drift_max, x * target_max) } else { (0.0, 0.0) };
    let fr = f.co_attrs.aerial_friction;
    fc::calc_self_accel_accel_to_vel_clamped(f, drift, target_vel, fr);
}

/// `ft_80084EEC`.
pub fn ft_80084eec(g: G) {
    melee_core::ct!("ft_80084eec");
    let f = fp(g);
    let (gr, t) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
    fc::fall(f, gr, t);
    let fr = f.co_attrs.aerial_friction;
    fc::calc_self_accel_deaccel(f, fr);
}

/// `ft_80084F3C`.
pub fn ft_80084f3c(g: G) {
    melee_core::ct!("ft_80084f3c");
    let f = fp(g);
    let mut friction = f.co_attrs.ground_friction;
    if fc::abs_c(f.gr_vel) > f.co_attrs.walk_max_vel {
        friction *= h::common_data().friction_when_above_walk_speed;
    }
    fc::calc_ground_accel_deaccel(f, friction);
    fc::set_self_movement_from_grounded_movement(f);
}

/// `ft_80084FA8`.
pub fn ft_80084fa8(g: G) {
    melee_core::ct!("ft_80084fa8");
    let f = fp(g);
    let mut v = f.gr_vel;
    let mut fr = f.co_attrs.ground_friction;
    if v < 0.0 {
        v = -v;
    }
    if v > f.co_attrs.walk_max_vel {
        fr *= h::common_data().friction_when_above_walk_speed;
    }
    let dir = f.facing_dir;
    ft_80085030(g, fr, dir);
}

/// `ft_80085004`.
pub fn ft_80085004(g: G) {
    melee_core::ct!("ft_80085004");
    let f = fp(g);
    let (fr, dir) = (f.co_attrs.ground_friction, f.facing_dir1);
    ft_80085030(g, fr, dir);
}

/// `ft_80085030(gobj, gr_friction, facing_dir)`.
pub fn ft_80085030(g: G, gr_friction: f32, facing_dir: f32) {
    melee_core::ct!("ft_80085030");
    let f = fp(g);
    if x594_b0(f) != 0 {
        f.xE4_ground_accel_1 = fmadds(f.x6A4_transNOffset.z, facing_dir, -f.gr_vel);
    } else {
        fc::calc_ground_accel_deaccel(f, gr_friction);
    }
    fc::set_self_movement_from_grounded_movement(f);
}

/// `ft_80085088`.
pub fn ft_80085088(g: G) {
    melee_core::ct!("ft_80085088");
    let f = fp(g);
    let (fr, dir) = (f.co_attrs.ground_friction, f.facing_dir);
    ft_800850e0(g, fr, dir);
}

/// `ft_800850B4`.
pub fn ft_800850b4(g: G) {
    melee_core::ct!("ft_800850b4");
    let f = fp(g);
    let (fr, dir) = (f.co_attrs.ground_friction, f.facing_dir1);
    ft_800850e0(g, fr, dir);
}

/// `ft_800850E0(gobj, friction, dir)`.
pub fn ft_800850e0(g: G, arg8: f32, arg9: f32) {
    melee_core::ct!("ft_800850e0");
    let f = fp(g);
    if x594_b0(f) != 0 {
        f.gr_vel = f.x6A4_transNOffset.z * arg9;
    } else {
        fc::calc_ground_accel_deaccel(f, arg8);
    }
    fc::set_self_movement_from_grounded_movement(f);
}

/// `ft_80085134`.
pub fn ft_80085134(g: G) {
    melee_core::ct!("ft_80085134");
    let f = fp(g);
    f.self_vel.x = f.x6A4_transNOffset.z * f.facing_dir;
    f.self_vel.y = f.x6A4_transNOffset.y;
}

/// `ft_80085154`: the animation velocity rotated by `lstick_angle` (MSL sinf/cosf).
pub fn ft_80085154(g: G) {
    melee_core::ct!("ft_80085154");
    let f = fp(g);
    let lx = cosf(f.lstick_angle);
    let ly = sinf(f.lstick_angle);
    let t0 = f.x6A4_transNOffset.y;
    let t3 = f.x6A4_transNOffset.z * f.facing_dir;
    f.self_vel.x = fmadds(t3, lx, -(t0 * ly));
    f.self_vel.y = fmadds(t3, ly, t0 * lx);
}

/// `ft_800851C0`.
pub fn ft_800851c0(g: G) {
    melee_core::ct!("ft_800851c0");
    let f = fp(g);
    f.self_vel.y = f.x6A4_transNOffset.y;
}

/// `ft_800851D0`: bunny-hood jump scale (`Fighter_804D6520->x14`) while `x197C` is set.
pub fn ft_800851d0(g: G) {
    melee_core::ct!("ft_800851d0");
    let f = fp(g);
    if !f.x197C.is_null() {
        f.self_vel.y = f.x6A4_transNOffset.y * h::fighter_804d6520().x14;
    } else {
        f.self_vel.y = f.x6A4_transNOffset.y;
    }
}

/// `ft_80085204`.
pub fn ft_80085204(g: G) {
    melee_core::ct!("ft_80085204");
    let f = fp(g);
    f.self_vel.x = f.x6A4_transNOffset.z * f.facing_dir;
}
