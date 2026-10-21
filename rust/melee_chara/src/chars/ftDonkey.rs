//! Donkey Kong (src/melee/ft/kinds/ftDonkey).

pub mod ftdonkeyspecialhi {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftDonkeyAttributes, Fighter};

    pub fn ftDk_SpecialAirHi_Phys(fp: &mut Fighter) {
        let a: &ftDonkeyAttributes = da(fp);
        let grav_mul = if fp.cmd_vars[0] != 0 { 1.0 } else { a.SpecialHi.x50_AERIAL_GRAVITY };
        co::fall(fp, grav_mul * fp.co_attrs.gravity, fp.co_attrs.terminal_velocity);
        co::calc_self_accel_drift_simple(
            fp,
            0.0,
            a.SpecialHi.x60_AERIAL_MOBILITY,
            a.SpecialHi.x58_AERIAL_HORIZONTAL_VELOCITY,
        );
    }
}

pub mod ftdonkeyspeciallw {
    use melee_types::Fighter;

    pub fn ftDk_SpecialLwLoop_IASA(fp: &mut Fighter) {
        if fp.input.pressed_buttons & 512 != 0 {
            unsafe { fp.mv.dk.speciallw.x0 = 1 };
        }
    }
}
