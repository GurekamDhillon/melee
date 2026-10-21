#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CargoJump.c. Every function of the TU; calls out of it go
//! through `ext` (unported ones panic until rewired). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_CargoJump.rs.
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
        ftCo_Jump_GetInput(g: G) -> u32 = |_| todo!("TODO(port) ftCo_Jump_GetInput (ftCo_Jump.c)");
        ftCo_8009B9C8(g: G, a: i32) = crate::tu::ftCo_CargoKneebend::ftCo_8009B9C8;
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftAnim_SetAnimRate(g: G, r: f32) = |_, _| todo!("TODO(port) ftAnim_SetAnimRate (ftanim.c)");
        ftCo_800CB110(g: G, b: bool, f: f32) = |_, _, _| todo!("TODO(port) ftCo_800CB110 (ftCo_Jump.c)");
        ftCo_8009C5A4(g: G, msid: i32) = |_, _| todo!("TODO(port) ftCo_8009C5A4 (ftCo_Shouldered.c)");
        ftCo_8009BF3C(g: G) -> bool = crate::tu::ftCo_CargoThrow::ftCo_8009BF3C;
        ftCo_Jump_Phys_Inner(g: G) = |_| todo!("TODO(port) ftCo_Jump_Phys_Inner (ftCo_Jump.c)");
        ftCo_CargoFall_Coll(g: G) = crate::tu::ftCo_CargoFall::ftCo_CargoFall_Coll;
    }
}

pub fn ftCo_8009BB1C(g: G) -> bool {
    let b = ext::ftCo_Jump_GetInput(g);
    if b != 0 {
        ext::ftCo_8009B9C8(g, b as i32);
        return true;
    }
    false
}

pub fn ftCo_8009BB64(g: G) {
    ext::ftCommon_8007D5D4(fp(g));
    let ms = x2cc(fp(g)).x4_motion_state;
    ext::Fighter_ChangeMotionState(g, ms + 7, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_SetAnimRate(g, 0.0);
    ext::ftCo_800CB110(g, true, 1.0);
    ext::ftCo_8009C5A4(fp(g).victim_gobj, ftCo_MS_ShoulderedWait as i32);
}

pub fn ftCo_CargoJump_Anim(_g: G) {}

pub fn ftCo_CargoJump_IASA(g: G) {
    if ext::ftCo_8009BF3C(g) {
        return;
    }
}

pub fn ftCo_CargoJump_Phys(g: G) {
    ext::ftCo_Jump_Phys_Inner(g);
}

pub fn ftCo_CargoJump_Coll(g: G) {
    ext::ftCo_CargoFall_Coll(g);
}
