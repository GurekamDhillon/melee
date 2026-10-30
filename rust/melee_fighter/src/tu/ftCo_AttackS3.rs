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
        ftCo_800957F4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { let _ = crate::tu::ftCo_ItemThrow::ftCo_800957F4(g as _, msid as _); } };
        ftCo_Attack_800CCF58(g: G, a: i32) = |g: G, a: i32| { unsafe { let _ = crate::tu::ftswing::ftCo_Attack_800CCF58(g as _, a as _); } };
        ftCo_Attack_800CDD14(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_0cdd::ftco_attack_800cdd14(g as _); } };
        ftpickupitem_80094790(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftpickupitem::ftpickupitem_80094790(g as _) as _ } };
        ftData_80085FD4(fp: &mut Fighter, msid: i32) -> *mut ftData_80085FD4_ret = |fp: &mut Fighter, msid: i32| -> *mut ftData_80085FD4_ret { unsafe { crate::tu::ftdata::ftData_80085FD4(fp as _, msid as _) as _ } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_Wait_IASA(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Wait::ftCo_Wait_IASA(g as _); } };
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80084104(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80084104(g as _); } };
    }
}

/// `ftCo_AttackS3_CheckInput`.
pub fn ftCo_AttackS3_CheckInput(g: G) -> bool {
    melee_core::ct!("ftCo_AttackS3_CheckInput");
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
    melee_core::ct!("decideAngle");
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
    melee_core::ct!("ftCo_AttackS3_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_AttackS3_IASA(g: G) {
    melee_core::ct!("ftCo_AttackS3_IASA");
    if fp(g).allow_interrupt() != 0 {
        ext::ftCo_Wait_IASA(g);
    }
}

pub fn ftCo_AttackS3_Phys(g: G) {
    melee_core::ct!("ftCo_AttackS3_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_AttackS3_Coll(g: G) {
    melee_core::ct!("ftCo_AttackS3_Coll");
    ext::ft_80084104(g);
}
