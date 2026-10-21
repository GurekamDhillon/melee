//! Samus (src/melee/ft/kinds/ftSamus).

pub mod ftsamusspeciallw0 {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftSs_DatAttrs, Fighter};

    pub fn ftSs_SpecialAirLw_Phys(fp: &mut Fighter) {
        let a: &ftSs_DatAttrs = da(fp);
        co::fall_basic(fp);
        co::calc_self_accel_drift_simple(
            fp,
            0.0,
            fp.co_attrs.ground_to_air_jump_momentum_multiplier * a.x10,
            fp.co_attrs.jump_h_max_velocity * a.x10,
        );
    }

    /// The gobj may be NULL (then 1).
    pub fn ftSs_SpecialLw_80129158(fp: Option<&Fighter>) -> i32 {
        if let Some(fp) = fp {
            match fp.motion_id as i32 {
                0x157..=0x15C => return (unsafe { fp.x2070.anon_0.x2071_b6() } != 0) as i32,
                _ => return 1,
            }
        }
        1
    }

    /// The gobj may be NULL (then 1).
    pub fn ftSs_SpecialN_801291A8(fp: Option<&Fighter>) -> i32 {
        if let Some(fp) = fp {
            return match fp.motion_id as i32 {
                0x157 | 0x158 | 0x15A | 0x15B | 0x15C => 0,
                _ => 1,
            };
        }
        1
    }
}

pub mod ftsamusspeciallw1 {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftSs_DatAttrs, Fighter};

    pub fn ftSs_SpecialAirLwBomb_Phys(fp: &mut Fighter) {
        let a: &ftSs_DatAttrs = da(fp);
        co::fall_basic(fp);
        co::calc_self_accel_drift_simple_no_friction(
            fp,
            0.0,
            fp.co_attrs.air_drift_stick_mul * a.x68,
            fp.co_attrs.air_drift_max * a.x60,
        );
    }
}
