#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Rebound.c (every function of the TU). Calls out of it
//! go through `ext`. Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_Rebound.rs.
use crate::types::*;

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_800804A0(fp: &mut Fighter, v: f32) = crate::ftcommon2::ftcommon_800804a0;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80083F88(g: G) = crate::tu::ft_081b::ft_80083f88;
    }
}

pub fn ftCo_80099D9C(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_ReboundStop as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    let x191c = f.dmg.x191C;
    let x0 = -f.dmg.facing_dir * melee_core::fp::fmadds(x191c, cd().x3D8, cd().x3DC);
    unsafe {
        f.mv.co.rebound.anim_speed = (f.co_attrs.clank_animation_length + 0.1) / x191c;
        f.mv.co.rebound.x0 = x0;
    }
    ext::ftCommon_800804A0(f, x0);
}

pub fn ftCo_ReboundStop_Anim(g: G) {
    ftCo_80099E44(g);
}

pub fn ftCo_80099E44(g: G) {
    let speed = unsafe { fp(g).mv.co.rebound.anim_speed };
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Rebound as i32, Ft_MF_None as u32, 0.0, speed, 0.0, core::ptr::null_mut());
}

pub fn ftCo_Rebound_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_Rebound_IASA(_g: G) {}

pub fn ftCo_Rebound_Phys(g: G) {
    let f = fp(g);
    if unsafe { f.mv.co.rebound.x0 } != 0.0 {
        unsafe { f.mv.co.rebound.x0 = 0.0 };
    } else {
        ext::ft_80084F3C(g);
    }
}

pub fn ftCo_Rebound_Coll(g: G) {
    ext::ft_80083F88(g);
}
