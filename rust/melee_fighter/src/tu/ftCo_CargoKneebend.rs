#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CargoKneebend.c. Every function of the TU; calls out of it go
//! through `ext` (unported ones panic until rewired). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_CargoKneebend.rs.
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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftAnim_SetAnimRate(g: G, r: f32) = |_, _| todo!("TODO(port) ftAnim_SetAnimRate (ftanim.c)");
        ftCo_8009C5A4(g: G, msid: i32) = |_, _| todo!("TODO(port) ftCo_8009C5A4 (ftCo_Shouldered.c)");
        ftCo_8009BB64(g: G) = crate::tu::ftCo_CargoJump::ftCo_8009BB64;
        ftCo_8009BF3C(g: G) -> bool = crate::tu::ftCo_CargoThrow::ftCo_8009BF3C;
        ftCo_KneeBend_Check_ShortHop(g: G) = |_| todo!("TODO(port) ftCo_KneeBend_Check_ShortHop (ftCo_KneeBend.c)");
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ftCo_CargoWait_Coll(g: G) = crate::tu::ftCo_CargoWait::ftCo_CargoWait_Coll;
    }
}

pub fn ftCo_8009B9C8(g: G, arg1: i32) {
    let f = fp(g);
    let lag = x2cc(f).cargo_hold.x24_JUMP_STARTUP_LAG;
    f.mv.co.cargokneebend.x4 = arg1;
    f.mv.co.cargokneebend.x0 = 0;
    f.mv.co.cargokneebend.x8 = lag;
    let ms = x2cc(f).x4_motion_state;
    ext::Fighter_ChangeMotionState(g, ms + 5, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_SetAnimRate(g, 0.0);
    ext::ftCo_8009C5A4(fp(g).victim_gobj, ftCo_MS_ShoulderedWait as i32);
}

pub fn ftCo_CargoKneebend_Anim(g: G) {
    if unsafe { fp(g).mv.ca.specialhi.vel.y } <= 0.0 {
        ext::ftCo_8009BB64(g);
    }
    unsafe { fp(g).mv.ca.specialhi.vel.y -= 1.0 };
}

pub fn ftCo_CargoKneebend_IASA(g: G) {
    if ext::ftCo_8009BF3C(g) {
        return;
    }
    ext::ftCo_KneeBend_Check_ShortHop(g);
}

pub fn ftCo_CargoKneebend_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_CargoKneebend_Coll(g: G) {
    ext::ftCo_CargoWait_Coll(g);
}
