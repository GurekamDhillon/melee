//! Peach (src/melee/ft/kinds/ftPeach).

pub mod ftpeachfloat {
    use melee_fighter::ftcommon as co;
    use melee_types::Fighter;

    pub fn ftPe_Float_Phys(fp: &mut Fighter) {
        co::calc_self_accel_drift(fp);
    }
}

pub mod ftpeachfloatfall {
    use melee_fighter::hooks::common_data;
    use melee_types::{ftPe_MS_FloatFallB, ftPe_MS_FloatFallF, ftPeach_MotionState, Fighter};

    /// `getFloatDir` (static): stick x times facing past `-p_ftCommonData->x78` -> forward.
    pub fn getFloatDir(fp: &Fighter) -> ftPeach_MotionState {
        if fp.input.lstick[0].x * fp.facing_dir > -common_data().x78 {
            ftPe_MS_FloatFallF
        } else {
            ftPe_MS_FloatFallB
        }
    }
}

pub mod ftpeachspecialhi {
    use melee_types::Fighter;

    /// `checkCmdVar2` (static): clears cmd_vars[2]; always returns false.
    pub fn checkCmdVar2(fp: &mut Fighter) -> bool {
        if fp.cmd_vars[2] == 0 {
            return false;
        }
        fp.cmd_vars[2] = 0;
        false
    }
}

pub mod ftpeachspecialn {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftPe_DatAttrs, ftPe_MS_SpecialAirNHit, ftPe_MS_SpecialN, Fighter};

    pub fn ftPe_SpecialAirNHit_Phys_fp(fp: &mut Fighter) {
        let a: &ftPe_DatAttrs = da(fp);
        co::fall(fp, a.xA0, a.xA4);
        co::calc_self_accel_deaccel(fp, a.x98);
    }

    pub fn ftPe_SpecialN_IsActive(fp: &Fighter) -> bool {
        let m = fp.motion_id as i32;
        !(m >= ftPe_MS_SpecialN as i32 && m <= ftPe_MS_SpecialAirNHit as i32)
    }
}

pub mod ftpeachspecials {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftPe_DatAttrs, Fighter};

    pub fn ftPe_SpecialAirSJump_Phys_fp(fp: &mut Fighter) {
        let a: &ftPe_DatAttrs = da(fp);
        if fp.cmd_vars[1] != 0 {
            co::calc_self_accel_deaccel(fp, a.x54);
        }
        let g = if fp.cmd_vars[1] != 0 { a.x58_gravity } else { a.x50_gravity };
        co::fall(fp, g, a.x5C_terminal_vel);
    }

    pub fn ftPe_SpecialSStart_Phys_fp(fp: &mut Fighter) {
        let a: &ftPe_DatAttrs = da(fp);
        co::calc_ground_accel_accel_to_vel(
            fp,
            a.specials_start_accel * fp.facing_dir,
            a.specials_start_vel_x * fp.facing_dir,
            0.0,
        );
        co::set_self_movement_from_grounded_movement(fp);
    }
}
