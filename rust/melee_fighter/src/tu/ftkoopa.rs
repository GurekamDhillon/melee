//! ftkoopa.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::types::*;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use crate::chardata::{da};

// ---- folded from chardata.rs (consolidation steps 21/22) ----
pub fn ftKp_SpecialS_80132DC0_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132DC0_fp");
da::<ftKoopaAttributes>(fp).x4C
}

pub fn ftKp_SpecialS_80132DD0_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132DD0_fp");
da::<ftKoopaAttributes>(fp).x48
}

pub fn ftKp_SpecialS_80132DE0_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132DE0_fp");
da::<ftKoopaAttributes>(fp).x44
}

pub fn ftKp_SpecialS_80132DF0_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132DF0_fp");
da::<ftKoopaAttributes>(fp).x40
}

pub fn ftKp_SpecialS_80132E00_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132E00_fp");
da::<ftKoopaAttributes>(fp).x3C
}

pub fn ftKp_SpecialS_80132E10_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132E10_fp");
da::<ftKoopaAttributes>(fp).x34
}

pub fn ftKp_SpecialS_80132E20_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKp_SpecialS_80132E20_fp");
da::<ftKoopaAttributes>(fp).x38
}
