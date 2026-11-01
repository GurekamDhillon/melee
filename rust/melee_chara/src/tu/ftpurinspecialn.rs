//! melee/ft/kinds/ftPurin/ftpurinspecialn.c: ports folded from the hand-written character modules (consolidation step 20).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da;
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
#[allow(unused_imports)]
use melee_types::{ftPurinAttributes, Fighter};

// ---- folded from chars blocks (consolidation step 20) ----
#[inline]
pub fn fall_da(fp: &mut Fighter) {
    melee_core::ct!("fall_da");
    let a: &ftPurinAttributes = da(fp);
    co::fall(fp, a.x3C, a.x40);
}

pub fn ftPr_SpecialAirNChargeFull_Phys(fp: &mut Fighter) {
    melee_core::ct!("ftPr_SpecialAirNChargeFull_Phys");
    fall_da(fp)
}

pub fn ftPr_SpecialAirNChargeLoop_Phys(fp: &mut Fighter) {
    melee_core::ct!("ftPr_SpecialAirNChargeLoop_Phys");
    fall_da(fp)
}

pub fn ftPr_SpecialAirNEnd_Phys(fp: &mut Fighter) {
    melee_core::ct!("ftPr_SpecialAirNEnd_Phys");
    fall_da(fp)
}

pub fn ftPr_SpecialAirNStart_Phys(fp: &mut Fighter) {
    melee_core::ct!("ftPr_SpecialAirNStart_Phys");
    fall_da(fp)
}

pub fn ftPr_SpecialNEnd_Phys(fp: &mut Fighter) {
    melee_core::ct!("ftPr_SpecialNEnd_Phys");
    let f = fp.co_attrs.ground_friction;
    co::calc_ground_accel_deaccel(fp, f);
    co::set_self_movement_from_grounded_movement(fp);
}

pub fn ftPr_SpecialNHit_Phys(fp: &mut Fighter) {
    melee_core::ct!("ftPr_SpecialNHit_Phys");
    let a: &ftPurinAttributes = da(fp);
    if fp.self_vel.y <= -a.x40 {
        co::calc_self_accel_drift(fp);
    }
    co::fall(fp, a.x3C, a.x40);
}

pub fn ftPr_SpecialNStart_Phys(fp: &mut Fighter) {
    melee_core::ct!("ftPr_SpecialNStart_Phys");
    fp.xE4_ground_accel_1 = 0.0;
    fp.gr_vel = 0.0;
    fp.self_vel.x = fp.facing_dir * 0.0001f32;
    fp.self_vel.y = 0.0;
    fp.x74_self_accel.y = 0.0;
    fp.x74_self_accel.x = 0.0;
}
