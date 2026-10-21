//! Bowser (src/melee/ft/kinds/ftKoopa).

pub mod ftkoopa {
    use crate::chars::da;
    use melee_types::{ftKoopaAttributes, Fighter};

    pub fn ftKp_SpecialS_80132DC0(fp: &Fighter) -> f32 {
        da::<ftKoopaAttributes>(fp).x4C
    }
    pub fn ftKp_SpecialS_80132DD0(fp: &Fighter) -> f32 {
        da::<ftKoopaAttributes>(fp).x48
    }
    pub fn ftKp_SpecialS_80132DE0(fp: &Fighter) -> f32 {
        da::<ftKoopaAttributes>(fp).x44
    }
    pub fn ftKp_SpecialS_80132DF0(fp: &Fighter) -> f32 {
        da::<ftKoopaAttributes>(fp).x40
    }
    pub fn ftKp_SpecialS_80132E00(fp: &Fighter) -> f32 {
        da::<ftKoopaAttributes>(fp).x3C
    }
    pub fn ftKp_SpecialS_80132E10(fp: &Fighter) -> f32 {
        da::<ftKoopaAttributes>(fp).x34
    }
    pub fn ftKp_SpecialS_80132E20(fp: &Fighter) -> f32 {
        da::<ftKoopaAttributes>(fp).x38
    }
}

pub mod ftkoopaspeciallw {
    use melee_types::Fighter;

    /// `ftKp_SpecialLw_Enter_inline` (static).
    pub fn ftKp_SpecialLw_Enter_inline(fp: &mut Fighter) {
        fp.cmd_vars[1] = 0;
        fp.cmd_vars[0] = 0;
        unsafe { fp.anon_2210.anon_0.set_throw_flags_b0(0) };
        fp.x6A4_transNOffset.y = 0.0;
        fp.set_x2223_b4(1);
    }
}

pub mod ftkoopaspecialn {
    use melee_types::Fighter;

    /// `ftKp_SpecialN_Anim_inline_1` (static).
    pub fn ftKp_SpecialN_Anim_inline_1(fp: &mut Fighter) {
        if fp.cur_anim_frame == 0.0 {
            unsafe {
                fp.mv.kp.specialn.x10 -= 1;
                if fp.mv.kp.specialn.x10 <= 0 {
                    fp.mv.kp.specialn.x10 = 0;
                }
            }
        }
    }
}
