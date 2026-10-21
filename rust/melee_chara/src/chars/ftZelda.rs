//! Zelda (src/melee/ft/kinds/ftZelda).

pub mod ftzeldaspecialhi {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftZelda_DatAttrs, Fighter};

    pub fn ftZd_SpecialAirHiStart_0_Phys(fp: &mut Fighter) {
        let a: &ftZelda_DatAttrs = da(fp);
        co::fall(fp, a.x40, a.x44);
        co::calc_self_accel_deaccel_air(fp);
    }

    pub fn ftZd_SpecialAirHi_Phys(fp: &mut Fighter) {
        let a: &ftZelda_DatAttrs = da(fp);
        if fp.cmd_vars[0] != 0 {
            co::fall_basic(fp);
            co::clamp_self_vel_x(fp, a.x5C * fp.co_attrs.air_drift_max);
            return;
        }
        // `vel_y - vel_y / 10` with an int 10: converted to 10.0f, all f32.
        let vel_y = fp.self_vel.y;
        fp.self_vel.y = vel_y - vel_y / 10.0;
        co::calc_self_accel_deaccel_air(fp);
    }

    pub fn ftZd_SpecialHiStart_1_Phys(fp: &mut Fighter) {
        co::set_self_movement_from_grounded_movement(fp);
    }
}

pub mod ftzeldaspeciallw {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftZelda_DatAttrs, Fighter};

    pub fn ftZd_SpecialAirLw2_Phys(fp: &mut Fighter) {
        let a: &ftZelda_DatAttrs = da(fp);
        co::fall(fp, a.x78, a.x7C);
        co::calc_self_accel_deaccel_air(fp);
    }

    pub fn ftZd_SpecialAirLw_Phys(fp: &mut Fighter) {
        let a: &ftZelda_DatAttrs = da(fp);
        co::fall(fp, a.x78, a.x7C);
        co::calc_self_accel_deaccel_air(fp);
    }
}

pub mod ftzeldaspecials {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftZelda_DatAttrs, Fighter};

    #[inline]
    fn air_s_phys(fp: &mut Fighter) {
        let temp_r3 = unsafe { fp.mv.zd.specials.x8 } as i32;
        let a: &ftZelda_DatAttrs = da(fp);
        if temp_r3 != 0 {
            unsafe { fp.mv.zd.specials.x8 = (temp_r3 - 1) as _ };
        } else {
            co::fall(fp, a.x2C, fp.co_attrs.terminal_velocity);
        }
        let aerial_friction = fp.co_attrs.aerial_friction;
        co::calc_self_accel_deaccel(fp, aerial_friction);
    }
    pub fn ftZd_SpecialAirSEnd_Phys(fp: &mut Fighter) {
        air_s_phys(fp)
    }
    pub fn ftZd_SpecialAirSLoop_Phys(fp: &mut Fighter) {
        air_s_phys(fp)
    }
    pub fn ftZd_SpecialAirSStart_Phys(fp: &mut Fighter) {
        air_s_phys(fp)
    }
}
