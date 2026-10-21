//! Popo / Ice Climbers (src/melee/ft/kinds/ftPopo).

pub mod ftpopospecialhi {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftIceClimberAttributes, Fighter};

    pub fn ftPp_SpecialAirHiThrow_1_Phys(fp: &mut Fighter) {
        let a: &ftIceClimberAttributes = da(fp);
        co::fall(fp, a.xA8, a.xAC);
        if fp.self_vel.y < 0.0 {
            co::calc_self_accel_deaccel_air(fp);
        }
    }
}

pub mod ftpopospecials {
    use crate::chars::{abs_c, da};
    use melee_types::{ftIceClimberAttributes, ftPp_MS_SpecialAirS2, ftPp_MS_SpecialS2, Fighter};

    pub fn ftPp_SpecialAirS1_IASA(fp: &mut Fighter) {
        let a: &ftIceClimberAttributes = da(fp);
        let x = fp.input.lstick[0].x;
        let v = if abs_c(x) >= a.x40 { x * a.x34 } else { 0.0 };
        unsafe { fp.mv.pp.specials.x1C = v };
    }

    pub fn ftPp_SpecialS1_IASA(fp: &mut Fighter) {
        let a: &ftIceClimberAttributes = da(fp);
        let x = fp.input.lstick[0].x;
        let v = if abs_c(x) >= a.x40 { x * a.x30 } else { 0.0 };
        unsafe { fp.mv.pp.specials.x1C = v };
    }

    pub fn ftPp_SpecialS_8011F964(fp: &Fighter) -> bool {
        let m = fp.motion_id as u32;
        !(m == ftPp_MS_SpecialS2 as u32 || m == ftPp_MS_SpecialAirS2 as u32)
    }
}
