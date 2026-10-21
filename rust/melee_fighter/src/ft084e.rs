//! Hand port, round 12: all 14 functions of src/melee/ft/ft_084E.c (shared physics callbacks:
//! air fall + drift, ground friction, animation-driven `x6A4_transNOffset` velocities) and
//! two more from the tail of ft_081B.c (`ft_80084CB0` ECB box copy, `ft_80084DB0` fall + drift),
//! and ft_08A1.c's stick test (`ft_8008A1FC`, `ft_8008A244` + its static inline).
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ft_084E.c`): ft_80085030 one f32
//! fmuladd (`z * dir - gr_vel`), ft_80085154 two (`a*b - c*d`, `a*b + c*d`: clang contracts the
//! left product, the right one is rounded to f32 first); all -> `fmadds`. The rest have none.
//! Checked against the C (cut verbatim, shipping contraction) by tests/ft084e_ref.rs.

use crate::ftcommon as fc;
use crate::hooks as h;
use crate::types::*;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf};

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn x594_b0(f: &Fighter) -> u8 {
    unsafe { f.anon_594.anon_0.x594_b0() }
}

/// fall-fast check, then fast fall or `ftCommon_Fall(gravity, terminal_velocity)`.
fn fall_or_fast(f: &mut Fighter) {
    fc::check_fall_fast(f);
    if f.fall_fast() != 0 {
        fc::fall_fast(f);
    } else {
        let (g, t) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
        fc::fall(f, g, t);
    }
}

/// `ft_80084E1C(gobj, threshold, drift_max, target_max)`.
pub fn ft_80084e1c(g: G, threshold: f32, drift_max: f32, target_max: f32) {
    let f = fp(g);
    fall_or_fast(f);
    let x = f.input.lstick[0].x;
    let (drift, target_vel) = if fc::abs_c(x) >= threshold { (x * drift_max, x * target_max) } else { (0.0, 0.0) };
    let fr = f.co_attrs.aerial_friction;
    fc::calc_self_accel_accel_to_vel_clamped(f, drift, target_vel, fr);
}

/// `ft_80084EEC`.
pub fn ft_80084eec(g: G) {
    let f = fp(g);
    let (gr, t) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
    fc::fall(f, gr, t);
    let fr = f.co_attrs.aerial_friction;
    fc::calc_self_accel_deaccel(f, fr);
}

/// `ft_80084F3C`.
pub fn ft_80084f3c(g: G) {
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
    let f = fp(g);
    let (fr, dir) = (f.co_attrs.ground_friction, f.facing_dir1);
    ft_80085030(g, fr, dir);
}

/// `ft_80085030(gobj, gr_friction, facing_dir)`.
pub fn ft_80085030(g: G, gr_friction: f32, facing_dir: f32) {
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
    let f = fp(g);
    let (fr, dir) = (f.co_attrs.ground_friction, f.facing_dir);
    ft_800850e0(g, fr, dir);
}

/// `ft_800850B4`.
pub fn ft_800850b4(g: G) {
    let f = fp(g);
    let (fr, dir) = (f.co_attrs.ground_friction, f.facing_dir1);
    ft_800850e0(g, fr, dir);
}

/// `ft_800850E0(gobj, friction, dir)`.
pub fn ft_800850e0(g: G, arg8: f32, arg9: f32) {
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
    let f = fp(g);
    f.self_vel.x = f.x6A4_transNOffset.z * f.facing_dir;
    f.self_vel.y = f.x6A4_transNOffset.y;
}

/// `ft_80085154`: the animation velocity rotated by `lstick_angle` (MSL sinf/cosf).
pub fn ft_80085154(g: G) {
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
    let f = fp(g);
    f.self_vel.y = f.x6A4_transNOffset.y;
}

/// `ft_800851D0`: bunny-hood jump scale (`Fighter_804D6520->x14`) while `x197C` is set.
pub fn ft_800851d0(g: G) {
    let f = fp(g);
    if !f.x197C.is_null() {
        f.self_vel.y = f.x6A4_transNOffset.y * h::fighter_804d6520().x14;
    } else {
        f.self_vel.y = f.x6A4_transNOffset.y;
    }
}

/// `ft_80085204`.
pub fn ft_80085204(g: G) {
    let f = fp(g);
    f.self_vel.x = f.x6A4_transNOffset.z * f.facing_dir;
}

/// `ft_80084CB0(fp, box)` (ft_081B.c).
pub fn ft_80084cb0(f: &Fighter, b: &mut ftCollisionBox) {
    let e = &f.coll_data.ecb;
    b.top = e.top.y;
    b.bottom = e.bottom.y;
    b.right.x = e.right.x;
    b.right.y = e.right.y;
    b.left.x = e.left.x;
    b.left.y = e.left.y;
}

/// `ft_80084DB0` (ft_081B.c).
pub fn ft_80084db0(g: G) {
    let f = fp(g);
    fall_or_fast(f);
    fc::calc_self_accel_drift(f);
}

// ---- ft_08A1.c: the walk/turn stick test ---------------------------------------------------

/// `ft_8008A1FC` (ft_08A1.c): the stick points against the facing or is under the walk
/// threshold. No fused site (a product compared with 0).
pub fn ft_8008a1fc(g: G) -> bool {
    let f = fp(g);
    f.input.lstick[0].x * f.facing_dir < 0.0 || fc::abs_c(f.input.lstick[0].x) < h::common_data().walk_stick_threshold
}

/// `ft_8008A244_inline` (static inline in ft_08A1.c): the same test as [`ft_8008a1fc`].
#[inline]
pub fn ft_8008a244_inline(g: G) -> bool {
    let f = fp(g);
    let v = f.input.lstick[0].x;
    v * f.facing_dir < 0.0 || fc::abs_c(v) < h::common_data().walk_stick_threshold
}

/// `ft_8008A244` (ft_08A1.c): on the test, `ft_8008A2BC` (enter Wait; a hook, not ported).
pub fn ft_8008a244(g: G) -> bool {
    if ft_8008a244_inline(g) {
        h::ft_8008a2bc(g);
        return true;
    }
    false
}
