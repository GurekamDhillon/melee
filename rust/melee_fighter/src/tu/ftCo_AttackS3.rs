#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_AttackS3.c (forward tilt). Every function of
//! the TU; calls out of it go through `ext` (unported ones panic until rewired; ported ones
//! default to the port). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ftCo_AttackS3.rs.


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
        it_8026B30C(i: *mut Item_GObj) -> i32 = crate::hooks::it_8026b30c;
        ftCo_800957F4(g: G, msid: i32) = |_, _| todo!("TODO(port) ftCo_800957F4 (ftCo_ItemThrow.c)");
        ftCo_Attack_800CCF58(g: G, a: i32) = |_, _| todo!("TODO(port) ftCo_Attack_800CCF58 (ftswing.c)");
        ftCo_Attack_800CDD14(g: G) = |_| todo!("TODO(port) ftCo_Attack_800CDD14 (ft_0CDD.c)");
        ftpickupitem_80094790(g: G) -> bool = |_| todo!("TODO(port) ftpickupitem_80094790 (ftpickupitem.c)");
        ftData_80085FD4(fp: &mut Fighter, msid: i32) -> *mut ftData_80085FD4_ret = |_, _| todo!("TODO(port) ftData_80085FD4 (ftdata.c)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftAnim_8006EBA4(g: G) = |_| todo!("TODO(port) ftAnim_8006EBA4 (ftanim.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_Wait_IASA(g: G) = |_| todo!("TODO(port) ftCo_Wait_IASA (ftCo_Wait.c)");
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80084104(g: G) = |_| todo!("TODO(port) ft_80084104 (ft_081B.c)");
    }
}

/// `ftCo_AttackS3_CheckInput`.
pub fn ftCo_AttackS3_CheckInput(g: G) -> bool {
    let f = fp(g);
    if f.input.pressed_buttons & HSD_PAD_A as u32 != 0 && f.input.lstick[0].x * f.facing_dir >= cd().x98 {
        // `ABS(ftCo_GetLStickAngle(fp))`: the macro evaluates its argument twice.
        let abs_ang = if ext::ftCo_GetLStickAngle(f) < 0.0 { -ext::ftCo_GetLStickAngle(f) } else { ext::ftCo_GetLStickAngle(f) };
        if abs_ang < cd().x20_radians {
            if !f.item_gobj.is_null() {
                if f.input.held_buttons[0] & HSD_PAD_LR != 0 || ext::it_8026B30C(f.item_gobj) == 0 {
                    ext::ftCo_800957F4(g, ftCo_MS_LightThrowF as i32);
                    return true;
                }
                match ext::it_8026B30C(f.item_gobj) {
                    2 => {
                        ext::ftCo_Attack_800CCF58(g, 1);
                        return true;
                    }
                    3 => {
                        ext::ftCo_Attack_800CDD14(g);
                        return true;
                    }
                    _ => {}
                }
            }
            decideAngle(g);
            return true;
        }
    }
    false
}

fn has_anim(f: &mut Fighter, msid: ftCommon_MotionState) -> bool {
    unsafe { (*ext::ftData_80085FD4(f, msid as i32)).x8 != 0 }
}

/// static `decideAngle`.
pub fn decideAngle(g: G) {
    let f = fp(g);
    if !ext::ftpickupitem_80094790(g) {
        let a = ext::ftCo_GetLStickAngle(f);
        let c = cd();
        let msid = if a > c.x9C_radians && has_anim(f, ftCo_MS_AttackS3S) {
            ftCo_MS_AttackS3Hi
        } else if a > c.xA0_radians && has_anim(f, ftCo_MS_AttackS3LwS) {
            ftCo_MS_AttackS3HiS
        } else if a < c.xA8_radians && has_anim(f, ftCo_MS_AttackLw3) {
            ftCo_MS_AttackS3Lw
        } else if a < c.xA4_radians && has_anim(f, ftCo_MS_AttackHi3) {
            ftCo_MS_AttackS3LwS
        } else {
            ftCo_MS_AttackS3S
        };
        f.set_allow_interrupt(0);
        ext::Fighter_ChangeMotionState(g, msid as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(g);
    }
}

pub fn ftCo_AttackS3_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_AttackS3_IASA(g: G) {
    if fp(g).allow_interrupt() != 0 {
        ext::ftCo_Wait_IASA(g);
    }
}

pub fn ftCo_AttackS3_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_AttackS3_Coll(g: G) {
    ext::ft_80084104(g);
}
