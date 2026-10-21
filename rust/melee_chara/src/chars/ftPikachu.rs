//! Pikachu (src/melee/ft/kinds/ftPikachu).

pub mod ftpikachuspecialhi {
    use crate::chars::{abs_c, da};
    use melee_fighter::ftcommon as co;
    use melee_types::{ftPikachuAttributes, Fighter};

    pub fn ftPk_SpecialAirHiEnd_Phys(fp: &mut Fighter) {
        let sa: &ftPikachuAttributes = da(fp);
        if fp.cmd_vars[0] != 0 {
            co::fall_basic(fp);
            co::clamp_self_vel_x(fp, sa.x9C * fp.co_attrs.air_drift_max);
        } else {
            fp.self_vel.y -= fp.self_vel.y / 9.0;
            co::calc_self_accel_deaccel_air(fp);
        }
    }

    pub fn ftPk_SpecialAirHiStart0_Phys(fp: &mut Fighter) {
        let sa: &ftPikachuAttributes = da(fp);
        unsafe {
            if fp.mv.pk.specialhi.x0 != 0 {
                fp.mv.pk.specialhi.x0 -= 1;
            } else {
                co::fall(fp, sa.x64, fp.co_attrs.terminal_velocity);
            }
        }
        co::calc_self_accel_deaccel_quick_air(fp);
    }

    pub fn ftPk_SpecialHiStart1_Phys(fp: &mut Fighter) {
        co::set_self_movement_from_grounded_movement(fp);
    }

    pub fn ftPk_SpecialHi_UpdateVel(fp: &mut Fighter) {
        fp.gr_vel = fp.facing_dir * abs_c(fp.gr_vel);
        fp.self_vel.x = fp.facing_dir * abs_c(fp.self_vel.x);
        unsafe {
            fp.mv.pk.specialhi.x10.x = fp.facing_dir * abs_c(fp.mv.pk.specialhi.x10.x);
        }
    }
}

pub mod ftpikachuspeciallw {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftPikachuAttributes, Fighter};

    pub fn ftPk_SpecialAirLwLoop1_Phys(fp: &mut Fighter) {
        let a: &ftPikachuAttributes = da(fp);
        let pika_b8 = a.xB8;
        let terminal_velocity = fp.co_attrs.terminal_velocity;
        co::fall(fp, pika_b8, terminal_velocity);
        co::calc_self_accel_deaccel_quick_air(fp);
    }

    pub fn ftPk_SpecialLw_SetState_Unk0(fp: &mut Fighter) {
        unsafe { fp.mv.pk.specialhi.x4 = 3 };
    }

    pub fn ftPk_SpecialLw_SetState_Unk1(fp: &mut Fighter) {
        unsafe { fp.mv.pk.specialhi.x4 = 0 };
    }
}

pub mod ftpikachuspecials {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftPikachuAttributes, Fighter};

    pub fn ftPk_SpecialAirS1_Phys(fp: &mut Fighter) {
        let a: &ftPikachuAttributes = da(fp);
        if fp.cmd_vars[0] != 0 {
            co::fall(fp, a.x58, a.x4C);
        } else {
            co::fall(fp, a.x48, a.x4C);
        }
        if fp.cmd_vars[0] != 0 {
            co::calc_self_accel_deaccel(fp, a.x54);
        }
    }

    pub fn ftPk_SpecialAirSEnd_Phys(fp: &mut Fighter) {
        let sa: &ftPikachuAttributes = da(fp);
        co::fall(fp, sa.x58, fp.co_attrs.terminal_velocity);
        co::calc_self_accel_deaccel(fp, sa.x54);
    }

    pub fn ftPk_SpecialSEnd_Phys(fp: &mut Fighter) {
        let sa: &ftPikachuAttributes = da(fp);
        co::calc_ground_accel_deaccel(fp, sa.x54);
        co::set_self_movement_from_grounded_movement(fp);
    }
}
