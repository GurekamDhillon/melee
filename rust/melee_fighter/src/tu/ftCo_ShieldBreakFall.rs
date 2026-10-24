#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ShieldBreakFall.c (TARGET_PC build). Every function of the TU;
//! calls out of it go through `ext` (unported ones panic until rewired). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_ShieldBreakFall.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[allow(dead_code)]
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}

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

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    ext_fns! {
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_ClampAirDrift(f: &mut Fighter) = crate::ftcommon::clamp_air_drift;
        ftCommon_8007EBAC(f: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ft_80084EEC(g: G) = crate::ft084e::ft_80084eec;
        ft_80082C74(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_80082c74;
    }
}

pub fn ftCo_80098D90(g: G) {
    ext::Fighter_ChangeMotionState(
        g,
        ftCo_MS_ShieldBreakFall as i32,
        (Ft_MF_KeepColAnimHitStatus | Ft_MF_SkipModel | Ft_MF_Unk06 | Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim) as u32,
        0.0,
        1.0,
        0.0,
        core::ptr::null_mut(),
    );
    ext::ftCommon_ClampAirDrift(fp(g));
    ext::ftCommon_8007EBAC(fp(g), 8, 0);
}

pub fn ftCo_ShieldBreakFall_Anim(_g: G) {}

pub fn ftCo_ShieldBreakFall_IASA(_g: G) {}

pub fn ftCo_ShieldBreakFall_Phys(g: G) {
    ext::ft_80084EEC(g);
}

pub fn ftCo_ShieldBreakFall_Coll(g: G) {
    ext::ft_80082C74(g, crate::tu::ftCo_ShieldBreakDown::ftCo_80098E3C);
}
