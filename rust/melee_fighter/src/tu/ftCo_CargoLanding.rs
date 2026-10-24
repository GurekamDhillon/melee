#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CargoLanding.c. Every function of the TU; calls out of it go
//! through `ext` (unported ones panic until rewired). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_CargoLanding.rs.
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
        ftCo_Landing_Enter(g: G, msid: i32, allow_interrupt: bool, flags: u32, start: f32, speed: f32) = |g: G, msid: i32, allow_interrupt: bool, flags: u32, start: f32, speed: f32| { unsafe { let _ = crate::tu::ftCo_Landing::ftCo_Landing_Enter(g as _, msid as _, allow_interrupt as _, flags as _, start as _, speed as _); } };
        ftAnim_SetAnimRate(g: G, r: f32) = |g: G, r: f32| { unsafe { let _ = crate::tu::ftanim::ftAnim_SetAnimRate(g as _, r as _); } };
        ftCo_8009C5A4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { let _ = crate::tu::ftCo_Shouldered::ftCo_8009C5A4(g as _, msid as _); } };
        ftCo_8009B518(g: G) = crate::tu::ftCo_CargoWait::ftCo_8009B518;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ftCo_CargoWait_Coll(g: G) = crate::tu::ftCo_CargoWait::ftCo_CargoWait_Coll;
    }
}

/// `ftCo_8009BD4C` as the C `HSD_GObjEvent` handed to `ft_80082C74`.
pub unsafe extern "C" fn ftCo_8009BD4C_cb(g: G) {
    ftCo_8009BD4C(g)
}

pub fn ftCo_8009BD4C(g: G) {
    let f = fp(g);
    let lag = x2cc(f).cargo_hold.x28_LANDING_LAG;
    f.mv.ca.specialhi.vel.x = lag;
    let ms = x2cc(f).x4_motion_state;
    ext::ftCo_Landing_Enter(g, ms + 8, true, Ft_MF_None, 0.0, 1.0);
    ext::ftAnim_SetAnimRate(g, 0.0);
    ext::ftCo_8009C5A4(fp(g).victim_gobj, 0x10A);
}

pub fn ftCo_CargoLanding_Anim(g: G) {
    if unsafe { fp(g).mv.ca.specialhi.vel.x } <= 0.0 {
        ext::ftCo_8009B518(g);
    }
    unsafe { fp(g).mv.ca.specialhi.vel.x -= 1.0 };
}

pub fn ftCo_CargoLanding_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_CargoLanding_Coll(g: G) {
    ext::ftCo_CargoWait_Coll(g);
}
