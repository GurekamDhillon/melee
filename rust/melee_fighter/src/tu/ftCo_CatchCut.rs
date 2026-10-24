#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CatchCut.c (grab released). Every function
//! of the TU; calls out go through `ext`. Checked against the C by tests/tu_ftCo_CatchCut.rs.

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
        ftCo_CaptureCut_Enter(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_CaptureCut::ftCo_CaptureCut_Enter(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCommon_8007D92C(g: G) = crate::ftcommon3::ftcommon_8007d92c;
        ftCommon_CalcGroundAccel_Deaccel(fp: &mut Fighter, friction: f32) = crate::ftcommon::calc_ground_accel_deaccel;
        ftCommon_SetSelfMovementFromGroundedMovement(g: G) = |g| crate::ftcommon::set_self_movement_from_grounded_movement(fp(g));
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_80084104(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80084104(g as _); } };
        ft_8008370C(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_8008370c(g as _, ev)); } };
        ft_80082B1C(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80082b1c(g as _); } };
    }
}

/// `ft_80082B1C` (ft_081B.c) as the C `HSD_GObjEvent` handed to `ft_8008370C`.
pub unsafe extern "C" fn ft_80082B1C_cb(g: *mut HSD_GObj) { ext::ft_80082B1C(g) }

pub fn ftCo_800DA698(g: G, arg1: bool) {
    let f = fp(g);
    let cd = crate::hooks::common_data();
    if f.ground_or_air == GA_Air {
        f.self_vel.x = -f.facing_dir * cd.x374;
        f.self_vel.y = cd.x378;
    } else {
        f.gr_vel = -f.facing_dir * cd.x370;
    }
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CatchCut as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    if arg1 {
        ext::ftCo_CaptureCut_Enter(fp(g).victim_gobj);
    }
}

pub fn ftCo_CatchCut_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCommon_8007D92C(g);
    }
}

pub fn ftCo_CatchCut_IASA(_g: G) {}

pub fn ftCo_CatchCut_Phys(g: G) {
    let f = fp(g);
    if f.ground_or_air == GA_Ground {
        let fr = crate::hooks::common_data().x64 * f.co_attrs.ground_friction;
        ext::ftCommon_CalcGroundAccel_Deaccel(f, fr);
        ext::ftCommon_SetSelfMovementFromGroundedMovement(g);
        return;
    }
    ext::ft_80084DB0(g);
}

pub fn ftCo_CatchCut_Coll(g: G) {
    if fp(g).ground_or_air == GA_Ground {
        ext::ft_80084104(g);
        return;
    }
    ext::ft_8008370C(g, Some(ft_80082B1C_cb));
}
