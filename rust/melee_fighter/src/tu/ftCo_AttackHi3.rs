#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_AttackHi3.c (up tilt). Every function of the
//! TU; calls out of it go through `ext` (unported ones panic until rewired; ported ones default
//! to the port). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ftCo_AttackHi3.rs.

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
        ftCo_GetLStickAngle(fp: &Fighter) -> f32 = crate::ftcommon3::ftco_get_lstick_angle;
        ftCo_80094E54(fp: &Fighter) -> bool = |_| todo!("TODO(port) ftCo_80094E54 (ftCo_ItemThrow.c)");
        ftCo_800957F4(g: G, msid: i32) = |_, _| todo!("TODO(port) ftCo_800957F4 (ftCo_ItemThrow.c)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftAnim_8006EBA4(g: G) = |_| todo!("TODO(port) ftAnim_8006EBA4 (ftanim.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_Wait_IASA(g: G) = |_| todo!("TODO(port) ftCo_Wait_IASA (ftCo_Wait.c)");
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80084104(g: G) = |_| todo!("TODO(port) ft_80084104 (ft_081B.c)");
    }
}

/// `ftCo_AttackHi3_CheckInput`.
pub fn ftCo_AttackHi3_CheckInput(g: G) -> bool {
    let f = fp(g);
    if f.input.pressed_buttons & HSD_PAD_A as u32 != 0
        && f.input.lstick[0].y >= cd().attackhi3_stick_threshold_y
        && ext::ftCo_GetLStickAngle(f) > cd().x20_radians
    {
        if !f.item_gobj.is_null() && ext::ftCo_80094E54(f) {
            ext::ftCo_800957F4(g, ftCo_MS_LightThrowHi as i32);
            return true;
        }
        doEnter(g);
        return true;
    }
    false
}

/// static `doEnter`.
pub fn doEnter(g: G) {
    let f = fp(g);
    f.set_allow_interrupt(0);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_AttackHi3 as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
}

pub fn ftCo_AttackHi3_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_AttackHi3_IASA(g: G) {
    if fp(g).allow_interrupt() != 0 {
        ext::ftCo_Wait_IASA(g);
    }
}

pub fn ftCo_AttackHi3_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_AttackHi3_Coll(g: G) {
    ext::ft_80084104(g);
}
