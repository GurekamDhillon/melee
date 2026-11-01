//! ftkirbyspecialn.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::types::*;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use crate::chardata::{da};

// ---- folded from chardata.rs (consolidation steps 21/22) ----
pub fn ftKb_SpecialN_800F58AC_fp(fp: &Fighter, victim_self_vel: &mut Vec3, victim_facing_dir: f32) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F58AC_fp");
let a: &ftKb_DatAttrs = da(fp);
victim_self_vel.x = -victim_facing_dir * a.specialn_ground_spit_initial_horizontal_velocity;
victim_self_vel.z = 0.0;
victim_self_vel.y = 0.0;
a.specialn_spit_deceleration_rate
}

pub fn ftKb_SpecialN_800F597C_fp(fp: &Fighter) -> i32 {
    melee_core::ct!("ftKb_SpecialN_800F597C_fp");
(fp.coll_data.floor.flags & 0x100) as i32
}

pub fn ftKb_SpecialN_800F5A38_fp(fp: &Fighter, _victim_fp: &Fighter) -> bool {
    melee_core::ct!("ftKb_SpecialN_800F5A38_fp");
let id = fp.motion_id as u32;
id == ftKb_MS_EatWait as u32 || id == ftKb_MS_EatFall as u32
}

pub fn ftKb_SpecialN_800F5A60_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5A60_fp");
-fp.facing_dir
}

pub fn ftKb_SpecialN_800F5A88_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5A88_fp");
da::<ftKb_DatAttrs>(fp).specialn_star_base_duration
}

pub fn ftKb_SpecialN_800F5AB0_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5AB0_fp");
da::<ftKb_DatAttrs>(fp).specialn_frames_in_swallow_star
}

pub fn ftKb_SpecialN_800F5AF0_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5AF0_fp");
da::<ftKb_DatAttrs>(fp).specialn_base_duration
}

pub fn ftKb_SpecialN_800F5B00_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B00_fp");
da::<ftKb_DatAttrs>(fp).specialn_duration_divisor
}

pub fn ftKb_SpecialN_800F5B10_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B10_fp");
da::<ftKb_DatAttrs>(fp).specialn_inhale_resistance
}

pub fn ftKb_SpecialN_800F5B20_fp(fp: &Fighter, out: &mut Vec2) {
    melee_core::ct!("ftKb_SpecialN_800F5B20_fp");
let a: &ftKb_DatAttrs = da(fp);
out.x = a.specialn_velocity_outer_grab_box;
out.y = a.specialn_velocity_inner_grab_box;
}

pub fn ftKb_SpecialN_800F5B3C_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B3C_fp");
da::<ftKb_DatAttrs>(fp).specialn_gravity_of_inhaled
}

pub fn ftKb_SpecialN_800F5B4C_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B4C_fp");
da::<ftKb_DatAttrs>(fp).specialn_z_offset_inhaled
}

pub fn ftKb_SpecialN_800F5B5C_fp(fp: &Fighter, output: &mut Vec3) {
    melee_core::ct!("ftKb_SpecialN_800F5B5C_fp");
let a: &ftKb_DatAttrs = da(fp);
*output = fp.cur_pos;
output.x = fmadds(a.specialn_x_offset_inhaled, fp.facing_dir, output.x);
output.y += a.specialn_y_offset_inhaled;
}
