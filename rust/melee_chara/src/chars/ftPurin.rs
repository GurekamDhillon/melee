//! Jigglypuff (src/melee/ft/kinds/ftPurin).

pub mod ftpurinspecialn {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftPurinAttributes, Fighter};

    #[inline]
    fn fall_da(fp: &mut Fighter) {
        let a: &ftPurinAttributes = da(fp);
        co::fall(fp, a.x3C, a.x40);
    }
    pub fn ftPr_SpecialAirNChargeFull_Phys(fp: &mut Fighter) {
        fall_da(fp)
    }
    pub fn ftPr_SpecialAirNChargeLoop_Phys(fp: &mut Fighter) {
        fall_da(fp)
    }
    pub fn ftPr_SpecialAirNEnd_Phys(fp: &mut Fighter) {
        fall_da(fp)
    }
    pub fn ftPr_SpecialAirNStart_Phys(fp: &mut Fighter) {
        fall_da(fp)
    }

    pub fn ftPr_SpecialNEnd_Phys(fp: &mut Fighter) {
        let f = fp.co_attrs.ground_friction;
        co::calc_ground_accel_deaccel(fp, f);
        co::set_self_movement_from_grounded_movement(fp);
    }

    pub fn ftPr_SpecialNHit_Phys(fp: &mut Fighter) {
        let a: &ftPurinAttributes = da(fp);
        if fp.self_vel.y <= -a.x40 {
            co::calc_self_accel_drift(fp);
        }
        co::fall(fp, a.x3C, a.x40);
    }

    pub fn ftPr_SpecialNStart_Phys(fp: &mut Fighter) {
        fp.xE4_ground_accel_1 = 0.0;
        fp.gr_vel = 0.0;
        fp.self_vel.x = fp.facing_dir * 0.0001f32;
        fp.self_vel.y = 0.0;
        fp.x74_self_accel.y = 0.0;
        fp.x74_self_accel.x = 0.0;
    }
}
