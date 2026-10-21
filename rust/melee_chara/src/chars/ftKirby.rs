//! Kirby (src/melee/ft/kinds/ftKirby), including the copied-ability TUs.

pub mod ftkirby {
    /// `ftKb_Init_UnkDemoCallbacks0(kind, out1, out2)` (the C's goto kept as a branch).
    pub fn ftKb_Init_UnkDemoCallbacks0(kind: i32, out1: &mut i32, out2: &mut i32) {
        if kind == 14 {
            *out2 = 17;
            *out1 = 17;
            return;
        }
        if kind >= 14 {
            return;
        }
        if kind < 11 {
            return;
        }
        *out1 = 14;
        *out2 = 16;
    }
}

pub mod ftkirbyspecialgamewatch {
    use melee_types::{ftKb_MS_GwSpecialAirN, ftKb_MS_GwSpecialN, Fighter};
    pub fn ftKb_SpecialNGw_8010D160(fp: &Fighter) -> bool {
        let m = fp.motion_id as i32;
        !(m >= ftKb_MS_GwSpecialN as i32 && m <= ftKb_MS_GwSpecialAirN as i32)
    }
}

pub mod ftkirbyspecialhi {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftKb_DatAttrs, Fighter};

    pub fn ftKb_SpecialAirHi3_Phys(fp: &mut Fighter) {
        let a: &ftKb_DatAttrs = da(fp);
        co::calc_self_accel_drift_simple_no_friction(
            fp,
            0.0,
            fp.co_attrs.air_drift_stick_mul * a.specialhi_horizontal_momentum,
            fp.co_attrs.air_drift_max,
        );
    }

    pub fn ftKb_SpecialHi3_Phys(fp: &mut Fighter) {
        let a: &ftKb_DatAttrs = da(fp);
        co::calc_ground_accel_accel_to_lstick_x(
            fp,
            0.0,
            fp.co_attrs.air_drift_stick_mul * a.specialhi_horizontal_momentum,
            fp.co_attrs.air_drift_max,
        );
        co::set_self_movement_from_grounded_movement(fp);
    }
}

pub mod ftkirbyspecialmars {
    use melee_types::Fighter;
    pub fn fn_8010B2E8(fp: &mut Fighter) {
        fp.cmd_vars[0] = 0;
        unsafe { fp.mv.ms.specialn.cur_frame = 0 as _ };
    }
}

pub mod ftkirbyspecialn {
    use crate::chars::da;
    use melee_core::fp::fmadds;
    use melee_types::{ftKb_DatAttrs, ftKb_MS_EatFall, ftKb_MS_EatWait, Fighter, Vec2, Vec3};

    /// Spit: the victim's launch velocity; returns the deceleration rate.
    pub fn ftKb_SpecialN_800F58AC(fp: &Fighter, victim_self_vel: &mut Vec3, victim_facing_dir: f32) -> f32 {
        let a: &ftKb_DatAttrs = da(fp);
        victim_self_vel.x = -victim_facing_dir * a.specialn_ground_spit_initial_horizontal_velocity;
        victim_self_vel.z = 0.0;
        victim_self_vel.y = 0.0;
        a.specialn_spit_deceleration_rate
    }

    /// Returns `coll_data.floor.flags & 0x100` (an int, not 0/1: MSL bool is int).
    pub fn ftKb_SpecialN_800F597C(fp: &Fighter) -> i32 {
        (fp.coll_data.floor.flags & 0x100) as i32
    }

    pub fn ftKb_SpecialN_800F5A04(fp: &mut Fighter, arg1: f32) {
        let a: &ftKb_DatAttrs = da(fp);
        if fp.ground_or_air == 0 {
            fp.gr_vel = arg1 * a.specialn_stop_momentum;
            return;
        }
        fp.self_vel.x = arg1 * a.specialn_stop_momentum;
    }

    pub fn ftKb_SpecialN_800F5A38(fp: &Fighter, _victim_fp: &Fighter) -> bool {
        let id = fp.motion_id as u32;
        id == ftKb_MS_EatWait as u32 || id == ftKb_MS_EatFall as u32
    }

    pub fn ftKb_SpecialN_800F5A60(fp: &Fighter) -> f32 {
        -fp.facing_dir
    }
    pub fn ftKb_SpecialN_800F5A88(fp: &Fighter) -> f32 {
        da::<ftKb_DatAttrs>(fp).specialn_star_base_duration
    }
    pub fn ftKb_SpecialN_800F5AB0(fp: &Fighter) -> f32 {
        da::<ftKb_DatAttrs>(fp).specialn_frames_in_swallow_star
    }
    pub fn ftKb_SpecialN_800F5AF0(fp: &Fighter) -> f32 {
        da::<ftKb_DatAttrs>(fp).specialn_base_duration
    }
    pub fn ftKb_SpecialN_800F5B00(fp: &Fighter) -> f32 {
        da::<ftKb_DatAttrs>(fp).specialn_duration_divisor
    }
    pub fn ftKb_SpecialN_800F5B10(fp: &Fighter) -> f32 {
        da::<ftKb_DatAttrs>(fp).specialn_inhale_resistance
    }
    pub fn ftKb_SpecialN_800F5B20(fp: &Fighter, out: &mut Vec2) {
        let a: &ftKb_DatAttrs = da(fp);
        out.x = a.specialn_velocity_outer_grab_box;
        out.y = a.specialn_velocity_inner_grab_box;
    }
    pub fn ftKb_SpecialN_800F5B3C(fp: &Fighter) -> f32 {
        da::<ftKb_DatAttrs>(fp).specialn_gravity_of_inhaled
    }
    pub fn ftKb_SpecialN_800F5B4C(fp: &Fighter) -> f32 {
        da::<ftKb_DatAttrs>(fp).specialn_z_offset_inhaled
    }

    /// Where an inhaled fighter sits. `output->x += off_x * facing_dir` is one fused site in
    /// the shipping build (fmadds); the y add is a plain f32 add.
    pub fn ftKb_SpecialN_800F5B5C(fp: &Fighter, output: &mut Vec3) {
        let a: &ftKb_DatAttrs = da(fp);
        *output = fp.cur_pos;
        output.x = fmadds(a.specialn_x_offset_inhaled, fp.facing_dir, output.x);
        output.y += a.specialn_y_offset_inhaled;
    }
}

pub mod ftkirbyspecialness {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftKb_DatAttrs, Fighter};

    pub fn ftKb_NsSpecialAirNStart_Phys(fp: &mut Fighter) {
        let a: &ftKb_DatAttrs = da(fp);
        unsafe {
            if fp.mv.ns.specialn.falling_acceleration_delay != 0 {
                fp.mv.ns.specialn.falling_acceleration_delay -= 1;
            } else {
                co::fall(fp, a.specialn_ns_gravity, fp.co_attrs.terminal_velocity);
            }
        }
        let air_friction = fp.co_attrs.aerial_friction;
        co::calc_self_accel_deaccel(fp, air_friction);
    }
}

pub mod ftkirbyspecialpeach {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftKb_DatAttrs, ftKb_MS_PeSpecialAirLwHit, ftKb_MS_PeSpecialLw, Fighter};

    pub fn ftKb_PeSpecialAirLwHit_Phys(fp: &mut Fighter) {
        let a: &ftKb_DatAttrs = da(fp);
        co::fall(fp, a.specialn_pe_fall_acceleration, a.specialn_pe_unk2);
        co::calc_self_accel_deaccel(fp, a.specialn_pe_air_horizontal_momentum_preservation);
    }

    pub fn ftKb_SpecialNPe_8010C4AC(fp: &Fighter) -> bool {
        let m = fp.motion_id as i32;
        !(m >= ftKb_MS_PeSpecialLw as i32 && m <= ftKb_MS_PeSpecialAirLwHit as i32)
    }
}

pub mod ftkirbyspecialpurin {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftCommonData, ftKb_DatAttrs, Fighter};

    /// The four aerial Rollout Phys callbacks read `dat_attrs` as `ftCommonData` (x2BC/x2C0),
    /// exactly as the C does.
    #[inline]
    fn fall_cd(fp: &mut Fighter) {
        let cd: &ftCommonData = da(fp);
        co::fall(fp, cd.x2BC, cd.x2C0);
    }
    pub fn ftKb_PrSpecialAirNEndR_Phys(fp: &mut Fighter) {
        fall_cd(fp)
    }
    pub fn ftKb_PrSpecialAirNFull_Phys(fp: &mut Fighter) {
        fall_cd(fp)
    }
    pub fn ftKb_PrSpecialAirNLoop_Phys(fp: &mut Fighter) {
        fall_cd(fp)
    }
    pub fn ftKb_PrSpecialAirNStart_Phys(fp: &mut Fighter) {
        fall_cd(fp)
    }

    pub fn ftKb_PrSpecialNEnd_Phys(fp: &mut Fighter) {
        let f = fp.co_attrs.ground_friction;
        co::calc_ground_accel_deaccel(fp, f);
        co::set_self_movement_from_grounded_movement(fp);
    }

    pub fn ftKb_PrSpecialNHit_Phys(fp: &mut Fighter) {
        let a: &ftKb_DatAttrs = da(fp);
        if fp.self_vel.y <= -a.specialn_pr_bounciness {
            co::calc_self_accel_drift(fp);
        }
        co::fall(fp, a.specialn_pr_air_height_offset_at_start, a.specialn_pr_bounciness);
    }

    pub fn ftKb_PrSpecialNStart_Phys(fp: &mut Fighter) {
        fp.xE4_ground_accel_1 = 0.0;
        fp.gr_vel = 0.0;
        fp.self_vel.x = 0.0001f32 * fp.facing_dir;
        fp.self_vel.y = 0.0;
        fp.x74_self_accel.y = 0.0;
        fp.x74_self_accel.x = 0.0;
    }
}

pub mod ftkirbyspecials {
    use melee_types::{Fighter, Vec3};
    pub fn ftKb_SpecialN_800F5800(fp: &Fighter, vec: &mut Vec3) {
        *vec = fp.cur_pos;
    }
}

pub mod ftkirbyspecialyoshi {
    use crate::chars::da;
    use melee_types::{ftKb_DatAttrs, Fighter, Vec3};

    pub fn ftKb_SpecialNYs_80109260(fp: &Fighter, vec: &mut Vec3) {
        let a: &ftKb_DatAttrs = da(fp);
        vec.x = -fp.facing_dir * a.specialn_ys_initial_horizontal_momentum;
        vec.y = a.specialn_ys_initial_vertical_momentum;
        vec.z = 0.0;
    }
    pub fn ftKb_SpecialNYs_80109290(fp: &Fighter) -> f32 {
        fp.facing_dir
    }
    pub fn ftKb_SpecialNYs_8010929C(fp: &Fighter) -> f32 {
        da::<ftKb_DatAttrs>(fp).specialn_ys_damage_multiplier
    }
    pub fn ftKb_SpecialNYs_801092AC(fp: &Fighter) -> f32 {
        da::<ftKb_DatAttrs>(fp).specialn_ys_unk
    }
    pub fn ftKb_SpecialNYs_801092BC(fp: &Fighter) -> f32 {
        da::<ftKb_DatAttrs>(fp).specialn_ys_growth_time
    }
    pub fn ftKb_SpecialNYs_801092CC(fp: &Fighter) -> f32 {
        da::<ftKb_DatAttrs>(fp).specialn_ys_base_duration
    }
}
