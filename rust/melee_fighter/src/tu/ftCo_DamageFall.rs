#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_DamageFall.c (tumble). The model-shift
//! leaves (ftCo_80090574, calcShift, ftCo_80090594, ftCo_80090690, ftCo_80090718, Anim, Phys)
//! were ported and verified in `crate::ftdmgfall` and are re-exported here; this file adds the
//! rest: ftCo_80090780, the TARGET_PC UCF 0.84 tumble test, DamageFall_IASA and DamageFall_Coll.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_DamageFall.rs.

use crate::types::*;

pub use crate::ftdmgfall::{
    calc_shift as calcShift, ftco_80090574 as ftCo_80090574, ftco_80090594 as ftCo_80090594,
    ftco_80090690 as ftCo_80090690, ftco_80090718 as ftCo_80090718,
    ftco_damagefall_anim as ftCo_DamageFall_Anim, ftco_damagefall_phys as ftCo_DamageFall_Phys,
};

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
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
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftGetParasolStatus(g: G) -> i32 = crate::ftcommon3::ft_get_parasol_status;
        ftCo_800CF4DC(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_ItemParasolDamageFall::ftCo_800CF4DC(g as _); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_ClampAirDrift(fp: &mut Fighter) = crate::ftcommon::clamp_air_drift;
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        Replay_UcfVersion(port: i32) -> i32 = |_| todo!("TODO(port) Replay_UcfVersion (pc replay)");
        ftUcf_PadRaw(port: i32, which_y: i32, back: i32) -> i32 = crate::fighter::ftucf_pad_raw;
        ftCo_800C5240(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C5240(g as _) as _ } };
        ftCo_SpecialAir_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialAir::ftCo_SpecialAir_CheckInput(g as _) as _ } };
        ftCo_80095328(g: G, out: *mut i32) -> bool = |g: G, out: *mut i32| -> bool { unsafe { crate::tu::ftCo_ItemThrow::ftCo_80095328(g as _, out as _) as _ } };
        ftCo_800D7100(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Attack100::ftCo_800D7100(g as _) as _ } };
        ftCo_800C3B10(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftco_aircatch::ftco_800c3b10(g as _) as _ } };
        ftCo_AttackAir_CheckItemThrowInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_AttackAir::ftCo_AttackAir_CheckItemThrowInput(g as _) as _ } };
        ftCo_800D705C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Attack100::ftCo_800D705C(g as _) as _ } };
        ftCo_800CB870(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_JumpAerial::ftCo_800CB870(g as _) as _ } };
        ftCo_Fall_Enter(g: G) = crate::hooks::ftco_fall_enter;
        ftCo_800C5DDC(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_HammerFall::ftCo_800C5DDC(g as _) as _ } };
        ftCo_800C5CD4(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_HammerFall::ftCo_800C5CD4(g as _) as _ } };
        ft_8008370C(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_8008370c;
        ftCo_80090984(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_DamageIce::ftCo_80090984(g as _); } };
    }
}

pub fn ftCo_80090780(g: G) {
    let f = fp(g);
    if f.ground_or_air == GA_Ground {
        ext::ftCommon_8007D5D4(f);
    }
    if ext::ftGetParasolStatus(g) != -1 {
        ext::ftCo_800CF4DC(g);
    } else {
        ext::Fighter_ChangeMotionState(g, 0x26, 0x18001, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftCommon_ClampAirDrift(fp(g));
        ext::ftCommon_8007EBAC(fp(g), 8, 0);
    }
}

#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// TARGET_PC static `ftCo_Ucf084Tumble`: UCF 0.84's tumble stick-out cancel window.
pub fn ftCo_Ucf084Tumble(f: &Fighter) -> bool {
    let port = f.x618_player_id as i32;
    let prev = f.input.lstick[1].x;
    if ext::Replay_UcfVersion(port) != 84 {
        return false;
    }
    if !(abs_c(prev) < cd().x210) {
        return false;
    }
    let d = ext::ftUcf_PadRaw(port, 0, 0).wrapping_sub(ext::ftUcf_PadRaw(port, 0, 2));
    d.wrapping_mul(d) > 5625
}

pub fn ftCo_DamageFall_IASA(g: G) {
    if !ext::ftCo_800C5240(g) {
        if ext::ftCo_SpecialAir_CheckInput(g) {
            return;
        }
        if ext::ftCo_80095328(g, core::ptr::null_mut()) {
            return;
        }
        if ext::ftCo_800D7100(g) {
            return;
        }
        if ext::ftCo_800C3B10(g) {
            return;
        }
        if ext::ftCo_AttackAir_CheckItemThrowInput(g) {
            return;
        }
        if ext::ftCo_800D705C(g) {
            return;
        }
        if ext::ftCo_800CB870(g) {
            return;
        }
        let f = fp(g);
        if abs_c(f.input.lstick[0].x) >= cd().x210
            && ((f.active_timer.lstick.x as i32) < cd().x214 || ftCo_Ucf084Tumble(f))
        {
            ext::ftCo_Fall_Enter(g);
            return;
        }
    }
    if ext::ftCo_800C5DDC(g) {
        return;
    }
    ext::ftCo_800C5CD4(g);
}

pub fn ftCo_DamageFall_Coll(g: G) {
    ext::ft_8008370C(g, ext::ftCo_80090984);
}
