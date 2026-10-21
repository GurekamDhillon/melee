//! Ness (src/melee/ft/kinds/ftNess).

pub mod ftness {
    use crate::chars::da;
    use melee_types::{ftNessAttributes, Fighter};

    pub fn ftNs_Init_GetAbsorbHeal(fp: &Fighter) -> f32 {
        da::<ftNessAttributes>(fp).x94_PSI_MAGNET_HEAL_MUL
    }
}

pub mod ftnessattackhi4 {
    use crate::chars::da;
    use melee_types::{ftNessAttributes, Fighter};

    /// `(s32) xB4_YOYO_REHIT_RATE`: C float->int truncation. The shipping build's gwtool makes
    /// fptosi saturating (NaN -> 0), which is Rust's `as i32`.
    pub fn ftNs_AttackHi4_YoyoStartTimedRehit(fp: &mut Fighter) {
        let a: &ftNessAttributes = da(fp);
        unsafe { fp.mv.ns.attackhi4.yoyoRehitTimer = a.xB4_YOYO_REHIT_RATE as i32 };
    }
}

pub mod ftnessspeciallw {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftNessAttributes, Fighter};

    pub fn ftNs_SpecialAirLwStart_Phys(fp: &mut Fighter) {
        let a: &ftNessAttributes = da(fp);
        let gravity_timer = unsafe { fp.mv.ns.speciallw.gravityDelay } as i32;
        if gravity_timer != 0 {
            unsafe { fp.mv.ns.speciallw.gravityDelay = (gravity_timer - 1) as _ };
        } else {
            co::fall(fp, a.x8C_PSI_MAGNET_FALL_ACCEL, fp.co_attrs.terminal_velocity);
        }
        co::calc_self_accel_deaccel_quick_air(fp);
    }
}

pub mod ftnessspecialn {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftNessAttributes, Fighter};

    #[inline]
    fn pk_flash_air_phys(fp: &mut Fighter) {
        let a: &ftNessAttributes = da(fp);
        unsafe {
            if fp.mv.ns.specialn.falling_acceleration_delay != 0 {
                fp.mv.ns.specialn.falling_acceleration_delay -= 1;
            } else {
                co::fall(fp, a.x14_PKFLASH_FALL_ACCEL, fp.co_attrs.terminal_velocity);
            }
        }
        let air_friction = fp.co_attrs.aerial_friction;
        co::calc_self_accel_deaccel(fp, air_friction);
    }
    pub fn ftNs_SpecialAirNEnd_Phys(fp: &mut Fighter) {
        pk_flash_air_phys(fp)
    }
    pub fn ftNs_SpecialAirNRelease_Phys(fp: &mut Fighter) {
        pk_flash_air_phys(fp)
    }
    pub fn ftNs_SpecialAirNStart_Phys(fp: &mut Fighter) {
        pk_flash_air_phys(fp)
    }
}
