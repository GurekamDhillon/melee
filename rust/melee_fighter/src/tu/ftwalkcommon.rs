//! Hand port of src/melee/ft/ftwalkcommon.c (all 8 definitions: GetWalkType, its `_fake`
//! inline twin, 800DFC70, 800DFCA4, 800DFDDC, 800DFEC8, getWalkAccel, 800E0060).
//!
//! Fused sites (`tools/rust-port/fma_sites.py`): one, in 800DFEC8
//! (`cur_anim_frame - float_result * quotient`, f32 -> f64 product and difference, one f32
//! rounding). Checked against the C (cut verbatim, shipping contraction) by
//! tests/tu_ftwalkcommon.rs.
#![allow(static_mut_refs)]

use crate::ftcommon as fc;
use crate::hooks;
use crate::types::*;

pub type G = *mut HSD_GObj;

/// Calls out of this TU whose owners are not ported (or not wired) yet. Each is a settable fn
/// pointer; reaching one unset panics with a TODO naming the C function.
pub mod ext {
    use super::G;
    use crate::types::*;
    macro_rules! ext {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " not ported")),
                }
            }
        )*};
    }
    ext! {
        FIGHTER_CHANGE_MOTION_STATE Fighter_ChangeMotionState(g: G, msid: i32, flags: MotionFlags, anim_start: f32, rate: f32, blend: f32, obj: *mut HSD_GObj);
        FTANIM_8006EBA4 ftAnim_8006EBA4(g: G);
        FTANIM_8006F484 ftAnim_8006F484(g: G) -> f32;
    }
    /// `ftAnim_SetAnimRate`: shared hook in `crate::hooks`.
    #[allow(non_snake_case)]
    pub fn ftAnim_SetAnimRate(g: G, rate: f32) {
        crate::hooks::ftanim_set_anim_rate(g, rate)
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn walk(f: &mut Fighter) -> &mut ftCommon_MotionVars_walk {
    unsafe { &mut f.mv.co.walk }
}

fn walk_type_of(f: &mut Fighter, walk_vel: f32) -> FtWalkType {
    let cd = hooks::common_data();
    let mul = walk(f).accel_mul;
    if walk_vel >= mul * (cd.walk_fast_stick_threshold * f.co_attrs.walk_max_vel) {
        FtWalkType_Fast
    } else if walk_vel >= mul * (cd.walk_middle_animation_stick_threshold * f.co_attrs.walk_max_vel) {
        FtWalkType_Middle
    } else {
        FtWalkType_Slow
    }
}

/// `ftWalkCommon_GetWalkType`.
pub fn ftwalkcommon_get_walk_type(g: G) -> FtWalkType {
    melee_core::ct!("ftwalkcommon_get_walk_type");
    let f = fp(g);
    let v = fc::abs_c(f.gr_vel);
    walk_type_of(f, v)
}

/// `ftWalkCommon_GetWalkType_800DFBF8_fake` (static inline; same arithmetic).
pub fn ftwalkcommon_get_walk_type_800dfbf8_fake(g: G) -> FtWalkType {
    melee_core::ct!("ftwalkcommon_get_walk_type_800dfbf8_fake");
    ftwalkcommon_get_walk_type(g)
}

/// `ftWalkCommon_800DFC70`.
pub fn ftwalkcommon_800dfc70(g: G) -> bool {
    melee_core::ct!("ftwalkcommon_800dfc70");
    let f = fp(g);
    f.input.lstick[0].x * f.facing_dir >= hooks::common_data().walk_stick_threshold
}

/// `ftWalkCommon_800DFCA4`.
#[allow(clippy::too_many_arguments)]
pub fn ftwalkcommon_800dfca4(
    g: G, msid: FtMotionId, ms_flags: MotionFlags, anim_start: f32, slow_anim_frame: f32,
    middle_anim_frame: f32, fast_anim_frame: f32, slow_anim_rate: f32, middle_anim_rate: f32,
    fast_anim_rate: f32, accel_mul: f32,
) {
    melee_core::ct!("ftwalkcommon_800dfca4");
    walk(fp(g)).accel_mul = accel_mul;
    let wt = ftwalkcommon_get_walk_type_800dfbf8_fake(g);
    let new_msid = msid.wrapping_add(wt as i32);
    ext::Fighter_ChangeMotionState(g, new_msid, ms_flags, anim_start, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    let gv = f.gr_vel;
    let w = walk(f);
    w.x0 = gv;
    w.msid = msid;
    w.slow_anim_frame = slow_anim_frame;
    w.middle_anim_frame = middle_anim_frame;
    w.fast_anim_frame = fast_anim_frame;
    w.slow_anim_rate = slow_anim_rate;
    w.middle_anim_rate = middle_anim_rate;
    w.fast_anim_rate = fast_anim_rate;
}

/// `ftWalkCommon_800DFDDC`.
pub fn ftwalkcommon_800dfddc(g: G) {
    melee_core::ct!("ftwalkcommon_800dfddc");
    let f = fp(g);
    let mut mv_x0 = if hooks::ground_friction_mul(f) < 1.0 { walk(f).x0 } else { f.gr_vel };
    let mut anim_rate = 0.0f32; // uninitialised in C on an unknown walk type: zero-init build
    if mv_x0 * f.facing_dir <= 0.0 {
        anim_rate = 0.0;
    } else {
        mv_x0 = fc::abs_c(mv_x0);
        let mid = f.motion_id as i32;
        let w = walk(f);
        match mid.wrapping_sub(w.msid as i32) {
            0 /* FtWalkType_Slow */ => anim_rate = mv_x0 / w.slow_anim_rate,
            1 /* FtWalkType_Middle */ => anim_rate = mv_x0 / w.middle_anim_rate,
            2 /* FtWalkType_Fast */ => anim_rate = mv_x0 / w.fast_anim_rate,
            _ => {}
        }
    }
    ext::ftAnim_SetAnimRate(g, anim_rate);
}

/// `ftWalkCommon_800DFEC8`.
pub fn ftwalkcommon_800dfec8(g: G, arg_cb: fn(G, f32)) {
    melee_core::ct!("ftwalkcommon_800dfec8");
    let wt = ftwalkcommon_get_walk_type_800dfbf8_fake(g);
    let f = fp(g);
    let base = walk(f).msid as i32;
    let sum = base.wrapping_add(wt as i32);
    if sum != f.motion_id as i32 {
        let w = *walk(f);
        let frame = match sum.wrapping_sub(base) {
            0 /* FtWalkType_Slow */ => w.slow_anim_frame,
            1 /* FtWalkType_Middle */ => w.middle_anim_frame,
            2 /* FtWalkType_Fast */ => w.fast_anim_frame,
            _ => panic!("couldn't get walk frame (ftwalkcommon.c:71)"),
        };
        let fr = ext::ftAnim_8006F484(g);
        let f = fp(g);
        let init = f.cur_anim_frame;
        let quotient = (init / fr) as i32;
        // fused: cur_anim_frame - float_result * (float) quotient
        let adj = (f.cur_anim_frame as f64 - fr as f64 * (quotient as f32) as f64) as f32;
        let fin = (frame * (adj / fr)) as i32;
        arg_cb(g, fin as f32);
    }
}

/// `getWalkAccel` (static).
fn get_walk_accel(f: &Fighter, mul: f32) -> f32 {
    if f.input.lstick[0].x > 0.0 {
        mul * f.co_attrs.walk_accel_base
    } else {
        mul * -f.co_attrs.walk_accel_base
    }
}

/// `ftWalkCommon_800E0060`.
pub fn ftwalkcommon_800e0060(g: G) {
    melee_core::ct!("ftwalkcommon_800e0060");
    let f = fp(g);
    let cd = hooks::common_data();
    let accel_mul = walk(f).accel_mul;
    let lx = f.input.lstick[0].x;
    let mut accel = lx * f.co_attrs.walk_accel_mul * accel_mul;
    accel += get_walk_accel(f, accel_mul);
    let target_vel = lx * f.co_attrs.walk_max_vel * accel_mul;
    if target_vel != 0.0 {
        let mult = f.gr_vel / target_vel;
        if mult > 0.0 && mult < 1.0 {
            accel *= (1.0 - mult) * cd.walk_accel_taper_gain;
        }
    }
    walk(f).x0 = target_vel * cd.x440;
    let gf = f.co_attrs.ground_friction;
    fc::calc_ground_accel_dash_run(f, accel, target_vel, gf);
    fc::set_self_movement_from_grounded_movement(f);
}
