//! Marth (src/melee/ft/kinds/ftMars).

pub mod ftmarsspecialn {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{Fighter, MarsAttributes};

    pub fn ftMs_SpecialAirNStart_Phys(fp: &mut Fighter) {
        let a: &MarsAttributes = da(fp);
        co::fall_basic(fp);
        co::calc_self_accel_deaccel(fp, a.specialn_start_friction);
    }

    pub fn ftMs_SpecialNStart_Phys(fp: &mut Fighter) {
        let a: &MarsAttributes = da(fp);
        co::calc_ground_accel_deaccel(fp, a.specialn_start_friction);
        co::set_self_movement_from_grounded_movement(fp);
    }
}
