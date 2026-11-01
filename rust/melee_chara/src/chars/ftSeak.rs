//! Sheik (src/melee/ft/kinds/ftSeak).

pub mod ftseakspecialhi {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftSeakAttributes, Fighter};

    pub fn ftSk_SpecialAirHiStart_0_Phys(fp: &mut Fighter) {
        let a: &ftSeakAttributes = da(fp);
        co::fall(fp, a.x30, a.x34);
        co::calc_self_accel_drift(fp);
    }

    pub fn ftSk_SpecialAirHi_Phys(fp: &mut Fighter) {
        let a: &ftSeakAttributes = da(fp);
        if fp.cmd_vars[0] != 0 {
            co::fall_basic(fp);
            co::clamp_self_vel_x(fp, a.x4C * fp.co_attrs.air_drift_max);
            return;
        }
        let vel_y = fp.self_vel.y;
        fp.self_vel.y = vel_y - vel_y / 10.0;
        co::calc_self_accel_deaccel_air(fp);
    }

    pub fn ftSk_SpecialHiStart_1_Phys(fp: &mut Fighter) {
        co::set_self_movement_from_grounded_movement(fp);
    }
}

pub mod ftseakspeciallw {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftSeakAttributes, Fighter};

    pub fn ftSk_SpecialAirLw2_Phys_fp(fp: &mut Fighter) {
        let a: &ftSeakAttributes = da(fp);
        co::fall(fp, a.x68, a.x6C);
        co::calc_self_accel_deaccel_air(fp);
    }

    pub fn ftSk_SpecialAirLw_Phys_fp(fp: &mut Fighter) {
        let a: &ftSeakAttributes = da(fp);
        co::fall(fp, a.x68, a.x6C);
        co::calc_self_accel_deaccel_air(fp);
    }
}

pub mod ftseakspecials {
    use melee_fighter::ftcommon as co;
    use melee_types::Fighter;

    pub fn ftSk_SpecialAirSStart_Phys_fp(fp: &mut Fighter) {
        if fp.cmd_vars[0] != 0 {
            co::fall(fp, fp.co_attrs.gravity, fp.co_attrs.terminal_velocity);
        }
        let f = fp.co_attrs.aerial_friction;
        co::calc_self_accel_deaccel(fp, f);
    }

    pub fn ftSk_SpecialS_80110F58(fp: &Fighter) -> f32 {
        fp.input.lstick[0].x
    }

    pub fn ftSk_SpecialS_80110F64(fp: &Fighter) -> f32 {
        fp.input.lstick[0].y
    }

    /// Zero the position and last position of the 4 hitboxes.
    pub fn ftSk_SpecialS_ZeroHitboxPositions(fp: &mut Fighter) {
        for hit in fp.x914.iter_mut() {
            hit.x58.x = 0.0;
            hit.x58.y = 0.0;
            hit.x58.z = 0.0;
            hit.x4C.x = 0.0;
            hit.x4C.y = 0.0;
            hit.x4C.z = 0.0;
        }
    }
}
