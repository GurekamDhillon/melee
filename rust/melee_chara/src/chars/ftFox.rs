//! Fox (src/melee/ft/kinds/ftFox).

pub mod ftfoxspecialhi {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftFox_DatAttrs, Fighter};

    pub fn ftFx_SpecialHiHoldAir_Phys(fp: &mut Fighter) {
        let a: &ftFox_DatAttrs = da(fp);
        unsafe {
            if fp.mv.fx.SpecialHi.gravityDelay != 0 {
                fp.mv.fx.SpecialHi.gravityDelay -= 1;
            } else {
                co::fall(fp, a.x60_FOX_FIREFOX_FALL_ACCEL, fp.co_attrs.terminal_velocity);
            }
        }
        co::calc_self_accel_deaccel(fp, a.x5C_FOX_FIREFOX_AIR_MOMENTUM_PRESERVE_X);
    }
}

pub mod ftfoxspecials {
    use crate::chars::da;
    use melee_fighter::ftcommon as co;
    use melee_types::{ftFox_DatAttrs, ftFx_MS_SpecialAirSEnd, ftFx_MS_SpecialSStart, Fighter, Vec3};

    pub fn ftFx_SpecialAirSStart_Phys(fp: &mut Fighter) {
        let a: &ftFox_DatAttrs = da(fp);
        unsafe {
            if fp.mv.fx.SpecialS.gravityDelay != 0 {
                fp.mv.fx.SpecialS.gravityDelay -= 1;
            } else {
                co::fall(fp, a.x30_FOX_ILLUSION_UNK2, fp.co_attrs.terminal_velocity);
            }
        }
        co::calc_self_accel_deaccel(fp, a.x2C_FOX_ILLUSION_UNK1);
    }

    pub fn ftFx_SpecialS_CheckGhostRemove(fp: &Fighter) -> bool {
        let msid = fp.motion_id as i32;
        !(msid >= ftFx_MS_SpecialSStart as i32 && msid <= ftFx_MS_SpecialAirSEnd as i32)
    }

    /// `*ghostPos = fp->mv.fx.SpecialS.ghostEffectPos[index]` (index unchecked in C).
    pub fn ftFx_SpecialS_CopyGhostPosIndexed(fp: &Fighter, index: i32, ghost_pos: &mut Vec3) {
        *ghost_pos = unsafe { fp.mv.fx.SpecialS.ghostEffectPos[index as usize] };
    }

    pub fn ftFx_SpecialS_GetCmdVar2(fp: &Fighter) -> i32 {
        fp.cmd_vars[2] as i32
    }
}
