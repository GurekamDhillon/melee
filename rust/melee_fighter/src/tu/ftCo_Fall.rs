#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Fall.c. Every function of the TU (the static
//! inline `ftCo_Fall_inline` too); calls out of it go through `ext`.
//! One fused site: ftCo_Fall_Anim_Inner `*mv_x4 += x448 * (var_f0 - *mv_x4)` -> `fmadds`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_Fall.rs.
use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    #[allow(unused_imports)]
    use super::*;
    macro_rules! ext_fns {
        ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
            pub mod hook {
                #[allow(unused_imports)] use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
            } )*
        };
    }
    ext_fns! {
        ftCo_80090780(g: G) = |_| todo!("TODO(port) ftCo_80090780 (ftCo_DamageFall.c)");
        ftCo_800C5240(g: G) -> bool = |_| todo!("TODO(port) ftCo_800C5240 (ftCo_HammerWait.c)");
        ftCo_800C5D34(g: G) = |_| todo!("TODO(port) ftCo_800C5D34 (ftCo_HammerFall.c)");
        ftLib_GetKind(g: G) -> u32 = |_| todo!("TODO(port) ftLib_GetKind (ftlib.c)");
        ftMh_MS_389_80151018(g: G) = |_| todo!("TODO(port) ftMh_MS_389_80151018 (ftmasterhand)");
        ftCh_GrabUnk1_8015BC88(g: G) = |_| todo!("TODO(port) ftCh_GrabUnk1_8015BC88 (ftcrazyhand)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftCommon_ClampAirDrift(fp: &mut Fighter) = crate::ftcommon::clamp_air_drift;
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        HSD_JObjAnimAll(j: *mut HSD_JObj) = |_| todo!("TODO(port) HSD_JObjAnimAll (jobj.c)");
        ftAnim_8006FE9C(fp: &mut Fighter, part: u32, a: f32, b: f32) = |_, _, _, _| todo!("TODO(port) ftAnim_8006FE9C (ftanim.c)");
        ftAnim_8006FF74(fp: &mut Fighter, part: u32) = |_, _| todo!("TODO(port) ftAnim_8006FF74 (ftanim.c)");
        ftAnim_8006EDD0(fp: &mut Fighter, smid: i32, frame: f32, rate: f32) = |_, _, _, _| todo!("TODO(port) ftAnim_8006EDD0 (ftanim.c)");
        ftCo_SpecialAir_CheckInput(g: G) -> bool = |_| todo!("TODO(port) ftCo_SpecialAir_CheckInput (ftCo_SpecialAir.c)");
        ftCo_80095328(g: G, b: *mut bool) -> bool = |_, _| todo!("TODO(port) ftCo_80095328 (ftCo_ItemThrow.c)");
        ftCo_800D7100(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D7100;
        ftCo_800C3B10(g: G) -> bool = |_| todo!("TODO(port) ftCo_800C3B10 (ftCo_AirCatch.c)");
        ftCo_80099A58(g: G) -> bool = crate::tu::ftCo_EscapeAir::ftCo_80099A58;
        ftCo_AttackAir_CheckItemThrowInput(g: G) -> bool = |_| todo!("TODO(port) ftCo_AttackAir_CheckItemThrowInput (ftCo_AttackAir.c)");
        ftCo_800D705C(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D705C;
        ftCo_800CEE70(g: G) -> bool = |_| todo!("TODO(port) ftCo_800CEE70 (ftCo_ItemParasolOpen.c)");
        ftPe_8011BA54(g: G) -> bool = |_| todo!("TODO(port) ftPe_8011BA54 (ftpeachfloat.c)");
        ftCo_800CB870(g: G) -> bool = |_| todo!("TODO(port) ftCo_800CB870 (ftCo_JumpAerial.c)");
        ftPe_8011BAD8(g: G) -> bool = |_| todo!("TODO(port) ftPe_8011BAD8 (ftpeachfloat.c)");
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_800831CC(g: G, a: unsafe extern "C" fn(G, i32) -> bool, b: HSD_GObjEvent) = |_, _, _| todo!("TODO(port) ft_800831CC (ft_081B.c)");
    }
    /// Address-taken only (passed to `ft_800831CC`); not called from this TU.
    pub extern "C" fn ft_80082B1C(_g: G) {
        todo!("TODO(port) ft_80082B1C (ft_081B.c)")
    }
}

/// static inline `ftCo_Fall_inline`.
pub fn ftCo_Fall_inline(g: G) -> bool {
    if fp(g).x2224_b2() != 0 {
        ext::ftCo_80090780(g);
        return true;
    }
    if ext::ftCo_800C5240(g) {
        ext::ftCo_800C5D34(g);
        return true;
    }
    false
}

pub fn ftCo_Fall_Enter(g: G) {
    if ext::ftLib_GetKind(g) == Ft_Kind_MasterH {
        ext::ftMh_MS_389_80151018(g);
        return;
    }
    if ext::ftLib_GetKind(g) == Ft_Kind_CrezyH {
        ext::ftCh_GrabUnk1_8015BC88(g);
        return;
    }
    if !ftCo_Fall_inline(g) {
        ext::Fighter_ChangeMotionState(g, ftCo_MS_Fall, Ft_MF_KeepFastFall, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftCommon_ClampAirDrift(fp(g));
        let f = fp(g);
        unsafe {
            f.mv.co.fall.smid = ftCo_SM_Fall;
            f.mv.co.fall.x4 = 0.0;
        }
        if f.ground_or_air == GA_Ground {
            ext::ftCommon_8007D5D4(f);
        }
    }
}

pub fn ftCo_Fall_Enter_YoshiEgg(g: G) {
    if !ftCo_Fall_inline(g) {
        ext::Fighter_ChangeMotionState(g, ftCo_MS_Fall, Ft_MF_Unk06, 0.0, 1.0, -1.0, core::ptr::null_mut());
        let f = fp(g);
        unsafe {
            f.mv.co.fall.smid = ftCo_SM_Fall;
            f.mv.co.fall.x4 = 0.0;
        }
    }
}

pub fn ftCo_Fall_Enter_YoshiEgg_Kirby(g: G) {
    ftCo_Fall_Enter_YoshiEgg(g);
}

pub fn ftCo_800CC988(g: G, arg1: f32) {
    if arg1 != 0.0 {
        ext::HSD_JObjAnimAll(fp(g).x8AC_animSkeleton);
        if arg1 != 1.0 {
            ext::ftAnim_8006FE9C(fp(g), FtPart_TransN, arg1, 1.0 - arg1);
        } else {
            ext::ftAnim_8006FF74(fp(g), FtPart_TransN);
        }
    }
}

pub fn ftCo_Fall_Anim(g: G) {
    let f = fp(g);
    if f.is_sandbag() == 0 {
        let p = unsafe { core::ptr::addr_of_mut!(f.mv.co.fall.x4) };
        ftCo_Fall_Anim_Inner(g, p, ftCo_SM_Fall, ftCo_SM_FallF, ftCo_SM_FallB);
        ftCo_800CC988(g, unsafe { fp(g).mv.co.fall.x4 });
    }
}

pub fn ftCo_Fall_IASA_Inner(g: G) -> bool {
    ext::ftCo_SpecialAir_CheckInput(g)
        || ext::ftCo_80095328(g, core::ptr::null_mut())
        || ext::ftCo_800D7100(g)
        || ext::ftCo_800C3B10(g)
        || ext::ftCo_80099A58(g)
        || ext::ftCo_AttackAir_CheckItemThrowInput(g)
        || ext::ftCo_800D705C(g)
        || ext::ftCo_800CEE70(g)
        || ext::ftPe_8011BA54(g)
        || ext::ftCo_800CB870(g)
        || ext::ftPe_8011BAD8(g)
}

/// `mv_x4` points into the fighter's motion vars (raw: it aliases `fp->mv`).
pub fn ftCo_Fall_Anim_Inner(g: G, mv_x4: *mut f32, neutral_smid: i32, forwards_smid: i32, backwards_smid: i32) {
    let f = fp(g);
    let c = cd();
    let mut frac = f.self_vel.x / f.co_attrs.air_drift_max;
    if frac > 1.0 {
        frac = 1.0;
    } else if frac < -1.0 {
        frac = -1.0;
    }
    let frac_abs = abs_c(frac);
    let (smid, var_f0) = if frac_abs > c.x444 {
        let s = if frac * f.facing_dir > 0.0 { forwards_smid } else { backwards_smid };
        (s, (frac_abs - c.x444) / (1.0 - c.x444))
    } else {
        (neutral_smid, 0.0)
    };
    unsafe {
        *mv_x4 = fmadds(c.x448, var_f0 - *mv_x4, *mv_x4);
        if *mv_x4 != 0.0 && smid != fp(g).mv.co.fall.smid {
            let f = fp(g);
            let frame = f.cur_anim_frame;
            ext::ftAnim_8006EDD0(f, smid, frame, 1.0);
            ext::HSD_JObjAnimAll(fp(g).x8AC_animSkeleton);
            let v = *mv_x4;
            if v != 1.0 {
                ext::ftAnim_8006FE9C(fp(g), FtPart_TransN, v, 1.0 - v);
            } else {
                ext::ftAnim_8006FF74(fp(g), FtPart_TransN);
            }
            fp(g).mv.co.fall.smid = smid;
        }
    }
}

pub fn ftCo_Fall_IASA(g: G) {
    ftCo_Fall_IASA_Inner(g);
}

pub fn ftCo_Fall_Phys(g: G) {
    ext::ft_80084DB0(g);
}

pub fn ftCo_Fall_Coll(g: G) {
    ext::ft_800831CC(g, crate::tu::ftCo_FallSpecial::ftCo_80096CC8, Some(ext::ft_80082B1C));
}
