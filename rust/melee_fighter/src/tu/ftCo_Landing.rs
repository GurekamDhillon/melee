#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Landing.c (landing lag). Every function of
//! the TU; calls out go through `ext`. No fused site (add then divide / add then compare).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_Landing.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
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
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        // TARGET_PC `ftGetTemplateKind` / `FTKB_IS_KIRBY` (inlines.h, ft/forward.h).
        Geno_DefineTemplateSkeleton(kind: i32) -> i32 = |kind: i32| -> i32 { unsafe { crate::platform::Geno_DefineTemplateSkeleton(kind) } };
        Geno_DefineBaseKind(kind: i32) -> i32 = |kind: i32| -> i32 { unsafe { crate::platform::Geno_DefineBaseKind(kind) } };
        ftKb_MexKirbyLike(kind: i32) -> i32 = |_| todo!("TODO(port) ftKb_MexKirbyLike (ftkirby.c)");
        ftPe_8011D598(g: G) = |_| todo!("TODO(port) ftPe_8011D598 (ftpeachspecialhi.c)");
        ftCo_800C5240(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C5240(g as _) as _ } };
        ftCo_HammerLanding_Enter(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_HammerLanding::ftCo_HammerLanding_Enter(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_SpecialS_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput(g as _) as _ } };
        ftCo_Attack100_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput;
        ftCo_800D6824(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D6824;
        ftCo_800D68C0(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D68C0;
        ftCo_Catch_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Catch::ftCo_Catch_CheckInput(g as _) as _ } };
        ftCo_AttackS4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS4::ftCo_AttackS4_CheckInput;
        ftCo_AttackHi4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInput;
        ftCo_AttackLw4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput;
        ftCo_AttackS3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput;
        ftCo_AttackHi3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi3::ftCo_AttackHi3_CheckInput;
        ftCo_AttackLw3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw3::ftCo_AttackLw3_CheckInput;
        ftCo_Attack1_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack1::ftCo_Attack1_CheckInput;
        ftCo_80091A4C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Guard::ftCo_80091A4C(g as _) as _ } };
        ftCo_800DE9D8(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_AppealS::ftCo_800DE9D8(g as _) as _ } };
        ftCo_Jump_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Jump::ftCo_Jump_CheckInput(g as _) as _ } };
        ftCo_Dash_CheckInput(g: G) -> bool = crate::tu::ftCo_Dash::ftCo_Dash_CheckInput;
        ftCo_SquatWait_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SquatWait::ftCo_SquatWait_CheckInput(g as _) as _ } };
        ftCo_Turn_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Turn::ftCo_Turn_CheckInput(g as _) as _ } };
        ftCo_Walk_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Walk::ftCo_Walk_CheckInput(g as _) as _ } };
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80084280(g: G) = crate::tu::ft_081b::ft_80084280;
    }
}

/// `ftGetTemplateKind(fp)` (inlines.h, TARGET_PC).
fn template_kind(f: &Fighter) -> i32 {
    let k = f.kind as i32;
    if k >= Ft_Kind_Mex0 as i32 && ext::Geno_DefineTemplateSkeleton(k) != 0 {
        return ext::Geno_DefineBaseKind(k);
    }
    k
}

/// `FTKB_CANON_KIND(ftGetTemplateKind(fp))`: the macro re-evaluates its argument at every use.
fn canon_template_kind(f: &Fighter) -> i32 {
    let is_kirby = template_kind(f) == Ft_Kind_Kirby as i32
        || (template_kind(f) >= Ft_Kind_Mex0 as i32
            && template_kind(f) < Ft_Kind_Max as i32
            && ext::ftKb_MexKirbyLike(template_kind(f)) != 0);
    if is_kirby { Ft_Kind_Kirby as i32 } else { template_kind(f) }
}

pub fn ftCo_Landing_Enter(g: G, msid: i32, allow_interrupt: bool, flags: u32, anim_start: f32, anim_speed: f32) {
    melee_core::ct!("ftCo_Landing_Enter");
    ext::ftCommon_8007D7FC(fp(g));
    ext::Fighter_ChangeMotionState(g, msid, flags, anim_start, anim_speed, 0.0, core::ptr::null_mut());
    unsafe { fp(g).mv.co.landing.allow_interrupt = allow_interrupt as i32 };
    let k = canon_template_kind(fp(g));
    let f = fp(g);
    unsafe {
        match k as u32 {
            Ft_Kind_Mario | Ft_Kind_DrMario => {
                f.u.mr.x2234_tornadoCharge = 0;
                f.u.mr.x2238_isCapeBoost = 0;
            }
            Ft_Kind_Peach => {
                f.u.pe.specialairn_used = 0;
                if allow_interrupt {
                    ext::ftPe_8011D598(g);
                }
            }
            Ft_Kind_Mars | Ft_Kind_Emblem => f.u.ms.x222C = 0,
            Ft_Kind_GameWatch => f.u.gw.x2234 = 0,
            Ft_Kind_Popo | Ft_Kind_Nana => f.u.pp.x224C = 0,
            Ft_Kind_Kirby => {
                f.u.kb.xCC = 0;
                f.u.kb.xC4 = 0;
                f.u.kb.x64 = 0;
            }
            Ft_Kind_Mewtwo => f.u.mt.x223C_isConfusionBoost = 0,
            _ => {}
        }
    }
}

pub fn ftCo_Landing_Enter_Basic(g: G) {
    melee_core::ct!("ftCo_Landing_Enter_Basic");
    if ext::ftCo_800C5240(g) {
        ext::ftCo_HammerLanding_Enter(g);
    } else {
        ftCo_Landing_Enter(g, ftCo_MS_Landing as i32, true, Ft_MF_None as u32, 0.0, 1.0);
    }
}

pub fn ftCo_LandingFallSpecial_Enter_Basic(g: G) {
    melee_core::ct!("ftCo_LandingFallSpecial_Enter_Basic");
    if ext::ftCo_800C5240(g) {
        ext::ftCo_HammerLanding_Enter(g);
    } else {
        ftCo_Landing_Enter(g, ftCo_MS_LandingFallSpecial as i32, false, Ft_MF_None as u32, 0.0, 1.0);
    }
}

pub fn ftCo_LandingFallSpecial_Enter(g: G, allow_interrupt: bool, landing_lag: f32) {
    melee_core::ct!("ftCo_LandingFallSpecial_Enter");
    if ext::ftCo_800C5240(g) {
        ext::ftCo_HammerLanding_Enter(g);
    } else {
        let speed = (0.1f32 + fp(g).x2EC) / landing_lag;
        ftCo_Landing_Enter(g, ftCo_MS_LandingFallSpecial as i32, allow_interrupt, Ft_MF_None as u32, 0.0, speed);
    }
}

pub fn ftCo_Landing_Anim(g: G) {
    melee_core::ct!("ftCo_Landing_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_Landing_IASA(g: G) {
    melee_core::ct!("ftCo_Landing_IASA");
    if fp(g).cur_anim_frame < fp(g).co_attrs.normal_landing_lag { return; }
    if unsafe { fp(g).mv.co.landing.allow_interrupt } == 0 { return; }
    if ext::ftCo_SpecialS_CheckInput(g) { return; }
    if ext::ftCo_Attack100_CheckInput(g) { return; }
    if ext::ftCo_800D6824(g) { return; }
    if ext::ftCo_800D68C0(g) { return; }
    if ext::ftCo_Catch_CheckInput(g) { return; }
    if ext::ftCo_AttackS4_CheckInput(g) { return; }
    if ext::ftCo_AttackHi4_CheckInput(g) { return; }
    if ext::ftCo_AttackLw4_CheckInput(g) { return; }
    if ext::ftCo_AttackS3_CheckInput(g) { return; }
    if ext::ftCo_AttackHi3_CheckInput(g) { return; }
    if ext::ftCo_AttackLw3_CheckInput(g) { return; }
    if ext::ftCo_Attack1_CheckInput(g) { return; }
    if ext::ftCo_80091A4C(g) { return; }
    if ext::ftCo_800DE9D8(g) { return; }
    if ext::ftCo_Jump_CheckInput(g) { return; }
    if ext::ftCo_Dash_CheckInput(g) { return; }
    let f = fp(g);
    if f.cur_anim_frame < f.frame_speed_mul + f.co_attrs.normal_landing_lag && ext::ftCo_SquatWait_CheckInput(g) {
        return;
    }
    if ext::ftCo_Turn_CheckInput(g) { return; }
    if ext::ftCo_Walk_CheckInput(g) {}
}

pub fn ftCo_Landing_Phys(g: G) {
    melee_core::ct!("ftCo_Landing_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_Landing_Coll(g: G) {
    melee_core::ct!("ftCo_Landing_Coll");
    ext::ft_80084280(g);
}
