//! Character attribute accessors the common fighter states call (`ftKb_SpecialN_*`, `ftKp_SpecialS_*`,
//! `ftYs_SpecialN_*`, `ftYs_Init_8012BAC0`): reads of the fighter's `dat_attrs` block / the loaded
//! character data, moved down from `melee_chara` so the fighter TUs need no upward dependency.
//! The bodies are the ones `melee_chara` checks against its C oracle.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::types::*;
use melee_core::fp::fmadds;

/// `(T*) fp->dat_attrs`: the character's special attributes block (owned by the fighter data).
#[inline(always)]
pub fn da<T>(fp: &Fighter) -> &'static T {
    unsafe { &*(fp.dat_attrs as *const T) }
}

/// Yoshi's loaded extra-attribute block (`gFtDataList[Ft_Kind_Yoshi]->ext_attr`).
unsafe fn ext_attr() -> &'static ftYoshiAttributes {
    &*((*crate::tu::ftdata::gFtDataList[Ft_Kind_Yoshi as usize]).ext_attr as *const ftYoshiAttributes)
}

pub fn ftKb_SpecialN_800F58AC(fp: &Fighter, victim_self_vel: &mut Vec3, victim_facing_dir: f32) -> f32 {
let a: &ftKb_DatAttrs = da(fp);
victim_self_vel.x = -victim_facing_dir * a.specialn_ground_spit_initial_horizontal_velocity;
victim_self_vel.z = 0.0;
victim_self_vel.y = 0.0;
a.specialn_spit_deceleration_rate
}
pub fn ftKb_SpecialN_800F597C(fp: &Fighter) -> i32 {
(fp.coll_data.floor.flags & 0x100) as i32
}
pub fn ftKb_SpecialN_800F5A38(fp: &Fighter, _victim_fp: &Fighter) -> bool {
let id = fp.motion_id as u32;
id == ftKb_MS_EatWait as u32 || id == ftKb_MS_EatFall as u32
}
pub fn ftKb_SpecialN_800F5A60(fp: &Fighter) -> f32 {
-fp.facing_dir
}
pub fn ftKb_SpecialN_800F5A88(fp: &Fighter) -> f32 {
da::<ftKb_DatAttrs>(fp).specialn_star_base_duration
}
pub fn ftKb_SpecialN_800F5AB0(fp: &Fighter) -> f32 {
da::<ftKb_DatAttrs>(fp).specialn_frames_in_swallow_star
}
pub fn ftKb_SpecialN_800F5AF0(fp: &Fighter) -> f32 {
da::<ftKb_DatAttrs>(fp).specialn_base_duration
}
pub fn ftKb_SpecialN_800F5B00(fp: &Fighter) -> f32 {
da::<ftKb_DatAttrs>(fp).specialn_duration_divisor
}
pub fn ftKb_SpecialN_800F5B10(fp: &Fighter) -> f32 {
da::<ftKb_DatAttrs>(fp).specialn_inhale_resistance
}
pub fn ftKb_SpecialN_800F5B20(fp: &Fighter, out: &mut Vec2) {
let a: &ftKb_DatAttrs = da(fp);
out.x = a.specialn_velocity_outer_grab_box;
out.y = a.specialn_velocity_inner_grab_box;
}
pub fn ftKb_SpecialN_800F5B3C(fp: &Fighter) -> f32 {
da::<ftKb_DatAttrs>(fp).specialn_gravity_of_inhaled
}
pub fn ftKb_SpecialN_800F5B4C(fp: &Fighter) -> f32 {
da::<ftKb_DatAttrs>(fp).specialn_z_offset_inhaled
}
pub fn ftKb_SpecialN_800F5B5C(fp: &Fighter, output: &mut Vec3) {
let a: &ftKb_DatAttrs = da(fp);
*output = fp.cur_pos;
output.x = fmadds(a.specialn_x_offset_inhaled, fp.facing_dir, output.x);
output.y += a.specialn_y_offset_inhaled;
}
pub fn ftKp_SpecialS_80132DC0(fp: &Fighter) -> f32 {
da::<ftKoopaAttributes>(fp).x4C
}
pub fn ftKp_SpecialS_80132DD0(fp: &Fighter) -> f32 {
da::<ftKoopaAttributes>(fp).x48
}
pub fn ftKp_SpecialS_80132DE0(fp: &Fighter) -> f32 {
da::<ftKoopaAttributes>(fp).x44
}
pub fn ftKp_SpecialS_80132DF0(fp: &Fighter) -> f32 {
da::<ftKoopaAttributes>(fp).x40
}
pub fn ftKp_SpecialS_80132E00(fp: &Fighter) -> f32 {
da::<ftKoopaAttributes>(fp).x3C
}
pub fn ftKp_SpecialS_80132E10(fp: &Fighter) -> f32 {
da::<ftKoopaAttributes>(fp).x34
}
pub fn ftKp_SpecialS_80132E20(fp: &Fighter) -> f32 {
da::<ftKoopaAttributes>(fp).x38
}
pub fn ftYs_Init_8012BAC0(fp: &Fighter) -> f32 {
da::<ftYoshiAttributes>(fp).x120
}
pub fn ftYs_SpecialN_8012CD40() -> f32 {
unsafe { ext_attr().x30 }
}
pub fn ftYs_SpecialN_8012CDB4() -> f32 {
unsafe {
let ea = ext_attr();
ea.x44 / ea.x18
}
}
pub fn ftYs_SpecialN_GetDatAttr18(fp: &Fighter) -> f32 {
da::<ftYoshiAttributes>(fp).x18
}
pub fn ftYs_SpecialN_GetDatAttr1C(fp: &Fighter) -> f32 {
da::<ftYoshiAttributes>(fp).x1C
}
pub fn ftYs_SpecialN_GetDatAttr20(fp: &Fighter) -> f32 {
da::<ftYoshiAttributes>(fp).x20
}
pub fn ftYs_SpecialN_GetDatAttr24(fp: &Fighter) -> f32 {
da::<ftYs_DatAttrs>(fp).x24
}
pub fn ftYs_SpecialN_GetExtAttr28() -> f32 {
unsafe { ext_attr().x28 }
}
pub fn ftYs_SpecialN_GetExtAttr2C() -> f32 {
unsafe { ext_attr().x2C }
}
pub fn ftYs_SpecialN_GetExtAttr34() -> f32 {
unsafe { ext_attr().x34 }
}
pub fn ftYs_SpecialN_GetExtAttr38() -> i32 {
unsafe { ext_attr().x38 as i32 }
}
pub fn ftYs_SpecialN_GetFacingDir(fp: &Fighter) -> f32 {
fp.facing_dir
}
pub fn ftYs_SpecialN_SetupItemVel(fp: &Fighter, out: &mut Vec3) {
let a: &ftYoshiAttributes = da(fp);
out.x = -fp.facing_dir * a.x10;
out.y = a.x14;
out.z = 0.0;
}
