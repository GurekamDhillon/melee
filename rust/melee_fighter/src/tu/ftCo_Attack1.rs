#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Attack1.c (jab 1/2/3 chain). Every function
//! of the TU; calls out of it go through `ext` (unported ones panic until rewired; ported ones
//! default to the port). `hitlag_mul` doubles as the jab input window here (f32 countdown).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_Attack1.rs.

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
        it_8026B30C(i: *mut Item_GObj) -> i32 = crate::hooks::it_8026b30c;
        ftCo_800957F4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { let _ = crate::tu::ftCo_ItemThrow::ftCo_800957F4(g as _, msid as _); } };
        ftCo_Attack_800CCF58(g: G, kind: i32) = |g: G, kind: i32| { unsafe { let _ = crate::tu::ftswing::ftCo_Attack_800CCF58(g as _, kind as _); } };
        ftCo_Attack_800CDD14(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_0cdd::ftco_attack_800cdd14(g as _); } };
        ft_800892A0(g: G) = |_| todo!("TODO(port) ft_800892A0 (ft_0881.c)");
        ft_80089824(g: G) = crate::ft0892::ft_80089824;
        ftGw_Attack11_Enter(g: G) = |_| todo!("TODO(port) ftGw_Attack11_Enter (ftgamewatch)");
        ftpickupitem_80094790(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftpickupitem::ftpickupitem_80094790(g as _) as _ } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_AttackS4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS4::ftCo_AttackS4_CheckInput;
        ftCo_AttackHi4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInput;
        ftCo_AttackLw4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput;
        ftCo_AttackS3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput;
        ftCo_AttackHi3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi3::ftCo_AttackHi3_CheckInput;
        ftCo_AttackLw3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw3::ftCo_AttackLw3_CheckInput;
        ftCo_Attack_800D6A50(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack_800D6A50;
        ftCo_Jump_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Jump::ftCo_Jump_CheckInput(g as _) as _ } };
        ftCo_Dash_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Dash::ftCo_Dash_CheckInput(g as _) as _ } };
        ftCo_800D5FB0(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Squat::ftCo_800D5FB0(g as _) as _ } };
        ftCo_Turn_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Turn::ftCo_Turn_CheckInput(g as _) as _ } };
        ftCo_Walk_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Walk::ftCo_Walk_CheckInput(g as _) as _ } };
        ftCo_Wait_IASA(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Wait::ftCo_Wait_IASA(g as _); } };
        ft_80084FA8(g: G) = crate::ft084e::ft_80084fa8;
        ft_80084104(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80084104(g as _); } };
        ftGetTemplateKind(fp: &Fighter) -> i32 = crate::ft0892::template_kind;
    }
}

/// `ftCo_Attack1_CheckInput`.
pub fn ftCo_Attack1_CheckInput(g: G) -> bool {
    let f = fp(g);
    if f.input.pressed_buttons & HSD_PAD_A as u32 != 0 {
        if !f.item_gobj.is_null() {
            if ext::it_8026B30C(f.item_gobj) == 0 {
                ext::ftCo_800957F4(g, ftCo_MS_LightThrowF as i32);
                return true;
            }
            if f.input.held_buttons[0] & HSD_PAD_LR != 0 {
                ext::ftCo_800957F4(g, ftCo_MS_LightThrowDrop as i32);
                return true;
            }
            match ext::it_8026B30C(f.item_gobj) {
                2 => {
                    ext::ftCo_Attack_800CCF58(g, 0);
                    return true;
                }
                3 => {
                    ext::ftCo_Attack_800CDD14(g);
                    return true;
                }
                _ => {}
            }
        }
        if f.hitlag_mul > 0.0 && f.x2218_b1() != 0 {
            match f.unk_msid {
                44 => {
                    doAttack12(g);
                    return true;
                }
                45 => {
                    doAttack13(g);
                    return true;
                }
                _ => {}
            }
        } else {
            decideAttack11(g);
            return true;
        }
    }
    if f.hitlag_mul > 0.0 {
        f.hitlag_mul -= 1.0;
    }
    false
}

/// static `onPkPc21EC` (Pikachu/Pichu x21EC callback).
pub unsafe extern "C" fn onPkPc21EC(g: *mut HSD_GObj) {
    ext::ft_800892A0(g);
    ext::ft_80089824(g);
}

/// static `decideAttack11`.
pub fn decideAttack11(g: G) {
    if fp(g).kind == Ft_Kind_GameWatch {
        ext::ftGw_Attack11_Enter(g);
    } else {
        checkAttack11(g);
    }
}

/// static `getMotionFlags`: Pikachu/Pichu install onPkPc21EC and skip the attack count.
pub fn getMotionFlags(f: &mut Fighter) -> u32 {
    match f.kind {
        Ft_Kind_Pichu | Ft_Kind_Pikachu => {
            f.x21EC = Some(onPkPc21EC);
            Ft_MF_SkipAttackCount as u32
        }
        _ => Ft_MF_None as u32,
    }
}

/// static `checkAttack11`: enter jab 1.
pub fn checkAttack11(g: G) {
    let f = fp(g);
    if !ext::ftpickupitem_80094790(g) {
        f.set_allow_interrupt(0);
        f.set_x2218_b1(0);
        let flags = getMotionFlags(f);
        ext::Fighter_ChangeMotionState(g, ftCo_MS_Attack11 as i32, flags, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(g);
        let f = fp(g);
        f.hitlag_mul = f.co_attrs.jab_2_input_window;
        f.unk_msid = ftCo_MS_Attack11 as i32;
        f.set_x2218_b2(0);
        f.mv.co.attack1.x0 = 0;
        f.x1A54 = 0;
    }
}

pub fn ftCo_Attack11_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

fn tilt_smash_checks(g: G) -> bool {
    ext::ftCo_AttackS4_CheckInput(g)
        || ext::ftCo_AttackHi4_CheckInput(g)
        || ext::ftCo_AttackLw4_CheckInput(g)
        || ext::ftCo_AttackS3_CheckInput(g)
        || ext::ftCo_AttackHi3_CheckInput(g)
        || ext::ftCo_AttackLw3_CheckInput(g)
}

fn movement_checks(g: G) -> bool {
    ext::ftCo_Jump_CheckInput(g)
        || ext::ftCo_Dash_CheckInput(g)
        || ext::ftCo_800D5FB0(g)
        || ext::ftCo_Turn_CheckInput(g)
        || ext::ftCo_Walk_CheckInput(g)
}

pub fn ftCo_Attack11_IASA(g: G) {
    if fp(g).allow_interrupt() != 0 && tilt_smash_checks(g) {
        return;
    }
    if ext::ftCo_Attack_800D6A50(g) || checkAttack12(g) {
        return;
    }
    if fp(g).allow_interrupt() != 0 {
        movement_checks(g);
    }
}

pub fn ftCo_Attack11_Phys(g: G) {
    ext::ft_80084FA8(g);
}

pub fn ftCo_Attack11_Coll(g: G) {
    ext::ft_80084104(g);
}

/// static `doAttack12Rapid`.
pub fn doAttack12Rapid(g: G) {
    if fp(g).kind == Ft_Kind_GameWatch {
        ext::ftGw_Attack11_Enter(g);
    } else {
        checkAttack11(g);
    }
}

/// static `doAttack12Normal`: enter jab 2.
pub fn doAttack12Normal(g: G) {
    let f = fp(g);
    if !ext::ftpickupitem_80094790(g) {
        f.set_allow_interrupt(0);
        f.set_x2218_b1(0);
        ext::Fighter_ChangeMotionState(g, ftCo_MS_Attack12 as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        let f = fp(g);
        f.hitlag_mul = f.co_attrs.jab_3_input_window;
        f.unk_msid = ftCo_MS_Attack12 as i32;
        f.mv.co.attack1.x0 = 0;
    }
}

/// static `doAttack12`.
pub fn doAttack12(g: G) {
    match fp(g).kind {
        Ft_Kind_Pikachu | Ft_Kind_Pichu => doAttack12Rapid(g),
        _ => doAttack12Normal(g),
    }
}

/// Count the window down, buffering an A press into `mv.co.attack1.x0`.
fn window_tick(f: &mut Fighter) {
    if f.hitlag_mul > 0.0 {
        f.hitlag_mul -= 1.0;
        if f.input.pressed_buttons & HSD_PAD_A as u32 != 0 {
            f.mv.co.attack1.x0 = 1;
        }
    }
}

/// `checkAttack12`.
pub fn checkAttack12(g: G) -> bool {
    let f = fp(g);
    window_tick(f);
    if unsafe { f.mv.co.attack1.x0 } != 0 && f.x2218_b1() != 0 {
        doAttack12(g);
        return true;
    }
    false
}

pub fn ftCo_Attack12_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_Attack12_IASA(g: G) {
    if fp(g).allow_interrupt() != 0 && tilt_smash_checks(g) {
        return;
    }
    if ext::ftCo_Attack_800D6A50(g) || checkAttack13(g) {
        return;
    }
    if fp(g).allow_interrupt() != 0 {
        movement_checks(g);
    }
}

/// static `doAttack13`: enter jab 3 (Marth re-enters jab 1).
pub fn doAttack13(g: G) {
    let f = fp(g);
    if ext::ftGetTemplateKind(f) == Ft_Kind_Mars as i32 {
        doAttack12Rapid(g);
        return;
    }
    if !ext::ftpickupitem_80094790(g) {
        f.set_allow_interrupt(0);
        f.set_x2218_b1(0);
        ext::Fighter_ChangeMotionState(g, ftCo_MS_Attack13 as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}

/// `checkAttack13`.
pub fn checkAttack13(g: G) -> bool {
    let f = fp(g);
    window_tick(f);
    if unsafe { f.mv.co.attack1.x0 } != 0 && f.x2218_b1() != 0 {
        doAttack13(g);
        return true;
    }
    false
}

pub fn ftCo_Attack13_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_Attack13_IASA(g: G) {
    if !ext::ftCo_Attack_800D6A50(g) && fp(g).allow_interrupt() != 0 {
        ext::ftCo_Wait_IASA(g);
    }
}
