#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CargoFall.c. Every function of the TU; calls out of it go
//! through `ext` (unported ones panic until rewired). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_CargoFall.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
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

#[inline(always)]
fn x2cc<'a>(f: &Fighter) -> &'a _ftDonkeyAttributes {
    unsafe { &*f.x2CC }
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    ext_fns! {
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_SetAnimRate(g: G, r: f32) = |g: G, r: f32| { unsafe { crate::tu::ftanim::ftAnim_SetAnimRate(g as _, r as _); } };
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCo_8009C5A4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { crate::tu::ftCo_Shouldered::ftCo_8009C5A4(g as _, msid as _); } };
        ftCo_8009BF3C(g: G) -> bool = crate::tu::ftCo_CargoThrow::ftCo_8009BF3C;
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_80082C74(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_80082c74(g as _, ev)); } };
    }
}

pub fn ftCo_8009BC58(g: G) {
    melee_core::ct!("ftCo_8009BC58");
    let ms = x2cc(fp(g)).x4_motion_state;
    ext::Fighter_ChangeMotionState(g, ms + 6, Ft_MF_KeepFastFall, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_SetAnimRate(g, 0.0);
    if fp(g).ground_or_air == GA_Ground {
        ext::ftCommon_8007D5D4(fp(g));
    }
    ext::ftCo_8009C5A4(fp(g).victim_gobj, ftCo_MS_ShoulderedWait as i32);
}

pub fn ftCo_CargoFall_Anim(_g: G) { melee_core::ct!("ftCo_CargoFall_Anim");}

pub fn ftCo_CargoFall_IASA(g: G) {
    melee_core::ct!("ftCo_CargoFall_IASA");
    if ext::ftCo_8009BF3C(g) {
        return;
    }
}

pub fn ftCo_CargoFall_Phys(g: G) {
    melee_core::ct!("ftCo_CargoFall_Phys");
    ext::ft_80084DB0(g);
}

pub fn ftCo_CargoFall_Coll(g: G) {
    melee_core::ct!("ftCo_CargoFall_Coll");
    ext::ft_80082C74(g, Some(crate::tu::ftCo_CargoLanding::ftCo_8009BD4C_cb));
}
