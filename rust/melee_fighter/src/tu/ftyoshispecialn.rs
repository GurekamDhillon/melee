//! ftyoshispecialn.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::types::*;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use crate::chardata::{da, ext_attr};

// ---- folded from chardata.rs (consolidation steps 21/22) ----
pub fn ftYs_SpecialN_8012CD40_fp() -> f32 {
    melee_core::ct!("ftYs_SpecialN_8012CD40_fp");
unsafe { ext_attr().x30 }
}

pub fn ftYs_SpecialN_8012CDB4_fp() -> f32 {
    melee_core::ct!("ftYs_SpecialN_8012CDB4_fp");
unsafe {
let ea = ext_attr();
ea.x44 / ea.x18
}
}

pub fn ftYs_SpecialN_GetDatAttr18_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetDatAttr18_fp");
da::<ftYoshiAttributes>(fp).x18
}

pub fn ftYs_SpecialN_GetDatAttr1C_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetDatAttr1C_fp");
da::<ftYoshiAttributes>(fp).x1C
}

pub fn ftYs_SpecialN_GetDatAttr20_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetDatAttr20_fp");
da::<ftYoshiAttributes>(fp).x20
}

pub fn ftYs_SpecialN_GetDatAttr24_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetDatAttr24_fp");
da::<ftYs_DatAttrs>(fp).x24
}

pub fn ftYs_SpecialN_GetExtAttr28_fp() -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetExtAttr28_fp");
unsafe { ext_attr().x28 }
}

pub fn ftYs_SpecialN_GetExtAttr2C_fp() -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetExtAttr2C_fp");
unsafe { ext_attr().x2C }
}

pub fn ftYs_SpecialN_GetExtAttr34_fp() -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetExtAttr34_fp");
unsafe { ext_attr().x34 }
}

pub fn ftYs_SpecialN_GetExtAttr38_fp() -> i32 {
    melee_core::ct!("ftYs_SpecialN_GetExtAttr38_fp");
unsafe { ext_attr().x38 as i32 }
}

pub fn ftYs_SpecialN_GetFacingDir_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftYs_SpecialN_GetFacingDir_fp");
fp.facing_dir
}

pub fn ftYs_SpecialN_SetupItemVel_fp(fp: &Fighter, out: &mut Vec3) {
    melee_core::ct!("ftYs_SpecialN_SetupItemVel_fp");
let a: &ftYoshiAttributes = da(fp);
out.x = -fp.facing_dir * a.x10;
out.y = a.x14;
out.z = 0.0;
}
