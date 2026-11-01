//! Yoshi (src/melee/ft/kinds/ftYoshi).

pub mod ftyoshi {
    use crate::chars::da;
    use melee_types::{ftYoshiAttributes, Fighter};

    pub fn ftYs_Init_8012BAC0(fp: &Fighter) -> f32 {
        da::<ftYoshiAttributes>(fp).x120
    }
}

pub mod ftyoshispecialn {
    use crate::chars::da;
    use melee_types::{ftYoshiAttributes, ftYs_DatAttrs, Fighter, Vec3};

    pub fn ftYs_SpecialN_GetDatAttr18(fp: &Fighter) -> f32 {
        da::<ftYoshiAttributes>(fp).x18
    }
    pub fn ftYs_SpecialN_GetDatAttr1C(fp: &Fighter) -> f32 {
        da::<ftYoshiAttributes>(fp).x1C
    }
    pub fn ftYs_SpecialN_GetDatAttr20(fp: &Fighter) -> f32 {
        da::<ftYoshiAttributes>(fp).x20
    }
    pub fn ftYs_SpecialN_GetDatAttr24(fp: &Fighter) -> f32 {
        da::<ftYs_DatAttrs>(fp).x24
    }
    pub fn ftYs_SpecialN_GetFacingDir(fp: &Fighter) -> f32 {
        fp.facing_dir
    }
    pub fn ftYs_SpecialN_SetupItemVel(fp: &Fighter, out: &mut Vec3) {
        let a: &ftYoshiAttributes = da(fp);
        out.x = -fp.facing_dir * a.x10;
        out.y = a.x14;
        out.z = 0.0;
    }
}

pub mod ftyoshispecials {
    use crate::chars::{abs_c, da};
    use melee_fighter::ftcommon as co;
    use melee_types::{ftYoshiAttributes, Fighter};

    /// `fn_8012EFF4` (the Egg Roll deal-damage callback): bleed speed off x1C.
    pub fn fn_8012EFF4(fp: &mut Fighter) {
        let a: &ftYoshiAttributes = da(fp);
        unsafe {
            fp.mv.ys.specials.xC = 0 as _;
        }
        let motion = fp.motion_id as i32;
        if motion != 0x16a {
            if motion >= 0x16a {
                return;
            }
            if motion != 0x165 {
                return;
            }
        }
        unsafe {
            let s = &mut fp.mv.ys.specials;
            s.x0 -= a.x50;
            let temp_f2 = if fp.ground_or_air == 1 { a.x8C } else { a.x7C };
            let s = &mut fp.mv.ys.specials;
            if abs_c(s.x1C) > temp_f2 {
                if s.x1C > 0.0 {
                    s.x1C -= a.xCC;
                    if s.x1C < temp_f2 {
                        s.x1C = temp_f2;
                    }
                } else {
                    s.x1C += a.xCC;
                    if s.x1C > -temp_f2 {
                        s.x1C = -temp_f2;
                    }
                }
            } else if s.x1C == 0.0 {
                s.x1C = fp.facing_dir * temp_f2;
            }
        }
    }

    pub fn ftYs_SpecialAirSEnd_Phys(fp: &mut Fighter) {
        let f = fp.co_attrs.ground_friction;
        co::calc_ground_accel_deaccel(fp, f);
        co::set_self_movement_from_grounded_movement(fp);
    }

    pub fn ftYs_SpecialAirSLanding_IASA_fp(fp: &mut Fighter) {
        unsafe { fp.mv.ys.specials.x30 = 0 as _ };
    }

    pub fn ftYs_SpecialAirSLanding_Phys(fp: &mut Fighter) {
        let a: &ftYoshiAttributes = da(fp);
        unsafe { fp.mv.ys.specials.x30 = 0 as _ };
        co::fall(fp, a.specials_start_gravity, a.specials_start_terminal_vel);
    }

    #[inline]
    fn air_s_loop_phys(fp: &mut Fighter) {
        let a: &ftYoshiAttributes = da(fp);
        unsafe { fp.mv.ys.specials.x30 = 0 as _ };
        let temp = fp.input.lstick[0].x * a.xE0;
        fp.self_vel.x += temp;
        let var_f0 = if fp.self_vel.x < 0.0 { -fp.self_vel.x } else { fp.self_vel.x };
        let mut var_f2 = a.x90;
        if var_f0 > var_f2 {
            if fp.self_vel.x < 0.0 {
                var_f2 = -var_f2;
            }
            fp.self_vel.x = var_f2;
        }
        co::fall(fp, a.specials_start_gravity, a.specials_start_terminal_vel);
    }
    pub fn ftYs_SpecialAirSLoop_2_Phys(fp: &mut Fighter) {
        air_s_loop_phys(fp)
    }
    pub fn ftYs_SpecialAirSLoop_3_Phys(fp: &mut Fighter) {
        air_s_loop_phys(fp)
    }

    pub fn ftYs_SpecialAirSStart_0_Phys_fp(fp: &mut Fighter) {
        fp.xE4_ground_accel_1 = 0.0;
        fp.gr_vel = 0.0;
        fp.self_vel.y = 0.0;
        fp.self_vel.x = 0.0;
        fp.x74_self_accel.y = 0.0;
        fp.x74_self_accel.x = 0.0;
    }

    pub fn ftYs_SpecialAirSStart_1_IASA_fp(fp: &mut Fighter) {
        unsafe { fp.mv.ys.specials.x30 = 0 as _ };
    }

    pub fn ftYs_SpecialAirSStart_1_Phys(fp: &mut Fighter) {
        let a: &ftYoshiAttributes = da(fp);
        unsafe { fp.mv.ys.specials.x30 = 0 as _ };
        co::fall(fp, a.specials_start_gravity, a.specials_start_terminal_vel);
    }
}
