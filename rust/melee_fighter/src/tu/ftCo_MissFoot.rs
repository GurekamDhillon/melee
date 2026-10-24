#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_MissFoot.c (ledge-miss stumble). Every
//! function of the TU; calls out go through `ext`. Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_MissFoot.rs.

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_ClampAirDrift(fp: &mut Fighter) = crate::ftcommon::clamp_air_drift;
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_80090780(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_DamageFall::ftCo_80090780(g as _); } };
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_80082F28(g: G) = crate::tu::ft_081b::ft_80082f28;
    }
}

pub fn ftCo_8009F39C(g: G) {
    fp(g).x8c_kb_vel.y = 0.0;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_MissFoot as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    ext::ftCommon_ClampAirDrift(f);
    if fp(g).ground_or_air == GA_Ground {
        ext::ftCommon_8007D5D4(fp(g));
    }
}

pub fn ftCo_MissFoot_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_80090780(g);
    }
}

pub fn ftCo_MissFoot_IASA(_g: G) {}

pub fn ftCo_MissFoot_Phys(g: G) {
    ext::ft_80084DB0(g);
}

pub fn ftCo_MissFoot_Coll(g: G) {
    ext::ft_80082F28(g);
}
