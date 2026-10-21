//! Link (src/melee/ft/kinds/ftLink).

pub mod ftlinkspecialhi {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftLk_DatAttrs, Fighter, Vec3};

    pub fn ftLk_SpecialAirHi_Phys(fp: &mut Fighter) {
        let a: &ftLk_DatAttrs = da(fp);
        co::fall(fp, fp.co_attrs.gravity * a.specialhi_grav_mul, fp.co_attrs.terminal_velocity);
        co::calc_self_accel_drift_simple(
            fp,
            0.0,
            fp.co_attrs.air_drift_stick_mul * a.specialairhi_drift_stick_mul,
            fp.co_attrs.air_drift_max * a.specialairhi_drift_max_mul,
        );
    }

    pub fn ftLk_SpecialHi_GetPosWithAdjustedY(fp: &Fighter, pos: &mut Vec3) {
        let a: &ftLk_DatAttrs = da(fp);
        *pos = fp.cur_pos;
        pos.y += a.specialhi_pos_y_offset;
    }
}

pub mod ftlinkspecialn {
    use melee_types::*;

    pub fn ftLk_SpecialN_IsActiveAnd2071b6(fp: &Fighter) -> bool {
        let m = fp.motion_id as u32;
        let active = [
            ftLk_MS_SpecialNStart,
            ftLk_MS_SpecialNLoop,
            ftLk_MS_SpecialNEnd,
            ftLk_MS_SpecialAirNStart,
            ftLk_MS_SpecialAirNLoop,
            ftLk_MS_SpecialAirNEnd,
        ];
        if active.iter().any(|&s| s as u32 == m) && unsafe { fp.x2070.anon_0.x2071_b6() } == 0 {
            return false;
        }
        true
    }
}

pub mod ftlinkspecials {
    use melee_types::Fighter;
    pub fn ftLk_SepcialS_Get2219b5(fp: &Fighter) -> bool {
        fp.x2219_b5() != 0
    }
}
