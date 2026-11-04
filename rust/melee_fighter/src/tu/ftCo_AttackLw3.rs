#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_AttackLw3.c (down tilt, with its re-chain on
//! A). Every function of the TU; calls out of it go through `ext` (unported ones panic until
//! rewired; ported ones default to the port). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_AttackLw3.rs.

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
        ftCo_80094E54(fp: &Fighter) -> bool = |a0| crate::tu::ftCo_ItemThrow::ftCo_80094E54(a0);
        ftCo_800957F4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { crate::tu::ftCo_ItemThrow::ftCo_800957F4(g as _, msid as _); } };
        ft_800892A0(g: G) = |a0| unsafe { crate::tu::ft_0881::ft_800892A0(a0 as _) };
        ft_80089824(g: G) = crate::ft0892::ft_80089824;
        ftGw_AttackLw3_Enter(g: G) = |_| todo!("TODO(port) ftGw_AttackLw3_Enter (ftgamewatch)");
        ftpickupitem_80094790(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftpickupitem::ftpickupitem_80094790(g as _) as _ } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_800D638C(g: G) = |g: G| { unsafe { crate::tu::ftCo_SquatWait::ftCo_800D638C(g as _); } };
        ftCo_AttackS4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS4::ftCo_AttackS4_CheckInput;
        ftCo_AttackHi4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInput;
        ftCo_AttackLw4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput;
        ftCo_AttackS3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput;
        ftCo_AttackHi3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi3::ftCo_AttackHi3_CheckInput;
        ftCo_Attack1_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack1::ftCo_Attack1_CheckInput;
        ftCo_Jump_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Jump::ftCo_Jump_CheckInput(g as _) as _ } };
        ftCo_Dash_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Dash::ftCo_Dash_CheckInput(g as _) as _ } };
        ftCo_Squat_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Squat::ftCo_Squat_CheckInput(g as _) as _ } };
        ftCo_Turn_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Turn::ftCo_Turn_CheckInput(g as _) as _ } };
        ftCo_Walk_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Walk::ftCo_Walk_CheckInput(g as _) as _ } };
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80084104(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80084104(g as _); } };
    }
}

/// `cmd_vars[cmd_unk0_bool]`.
const CMD_UNK0_BOOL: usize = 0;

fn stick_down(f: &Fighter) -> bool {
    f.input.lstick[0].y <= cd().xB0 && ext::ftCo_GetLStickAngle(f) < -cd().x20_radians
}

/// `ftCo_AttackLw3_CheckInput`.
pub fn ftCo_AttackLw3_CheckInput(g: G) -> bool {
    melee_core::ct!("ftCo_AttackLw3_CheckInput");
    let f = fp(g);
    if f.input.pressed_buttons & HSD_PAD_A as u32 != 0 && stick_down(f) {
        if !f.item_gobj.is_null() && ext::ftCo_80094E54(f) {
            ext::ftCo_800957F4(g, ftCo_MS_LightThrowLw as i32);
            return true;
        }
        decideFighter(g);
        return true;
    }
    false
}

/// static `callUnk` (the x21EC callback).
pub unsafe extern "C" fn callUnk(g: *mut HSD_GObj) {
    melee_core::ct!("callUnk");
    ext::ft_800892A0(g);
    ext::ft_80089824(g);
}

/// static `decideFighter`.
pub fn decideFighter(g: G) {
    melee_core::ct!("decideFighter");
    if fp(g).kind == Ft_Kind_GameWatch {
        ext::ftGw_AttackLw3_Enter(g);
    } else {
        doEnter(g);
    }
}

/// static `doEnter`.
pub fn doEnter(g: G) {
    melee_core::ct!("doEnter");
    let f = fp(g);
    if !ext::ftpickupitem_80094790(g) {
        f.cmd_vars[CMD_UNK0_BOOL] = 0;
        f.set_allow_interrupt(0);
        f.mv.co.attacklw3.x0 = 0;
        f.x21EC = Some(callUnk);
        ext::Fighter_ChangeMotionState(g, ftCo_MS_AttackLw3 as i32, Ft_MF_SkipAttackCount as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(g);
    }
}

/// static `wrapper` (fake in the decomp: just decideFighter).
pub fn wrapper(g: G) {
    melee_core::ct!("wrapper");
    decideFighter(g);
}

pub fn ftCo_AttackLw3_Anim(g: G) {
    melee_core::ct!("ftCo_AttackLw3_Anim");
    let f = fp(g);
    if f.cmd_vars[CMD_UNK0_BOOL] != 0 && unsafe { f.mv.co.attacklw3.x0 } != 0 {
        wrapper(g);
        return;
    }
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_800D638C(g);
    }
}

/// static `checkPadA`: re-chain when the script opened the window, else buffer the press.
pub fn checkPadA(g: G) -> bool {
    melee_core::ct!("checkPadA");
    let f = fp(g);
    if f.input.pressed_buttons & HSD_PAD_A as u32 != 0 {
        if f.cmd_vars[CMD_UNK0_BOOL] != 0 {
            decideFighter(g);
            return true;
        }
        f.mv.co.attacklw3.x0 = 1;
    }
    false
}

/// static `checkItemThrowInput` (same test as the CheckInput).
pub fn checkItemThrowInput(g: G) -> bool {
    melee_core::ct!("checkItemThrowInput");
    ftCo_AttackLw3_CheckInput(g)
}

pub fn ftCo_AttackLw3_IASA(g: G) {
    melee_core::ct!("ftCo_AttackLw3_IASA");
    let ai = fp(g).allow_interrupt() != 0;
    if ai {
        if ext::ftCo_AttackS4_CheckInput(g)
            || ext::ftCo_AttackHi4_CheckInput(g)
            || ext::ftCo_AttackLw4_CheckInput(g)
            || ext::ftCo_AttackS3_CheckInput(g)
            || ext::ftCo_AttackHi3_CheckInput(g)
        {
            return;
        }
    }
    if checkPadA(g) {
        return;
    }
    if fp(g).allow_interrupt() != 0 {
        let _ = checkItemThrowInput(g)
            || ext::ftCo_Attack1_CheckInput(g)
            || ext::ftCo_Jump_CheckInput(g)
            || ext::ftCo_Dash_CheckInput(g)
            || ext::ftCo_Squat_CheckInput(g)
            || ext::ftCo_Turn_CheckInput(g)
            || ext::ftCo_Walk_CheckInput(g);
    }
}

pub fn ftCo_AttackLw3_Phys(g: G) {
    melee_core::ct!("ftCo_AttackLw3_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_AttackLw3_Coll(g: G) {
    melee_core::ct!("ftCo_AttackLw3_Coll");
    ext::ft_80084104(g);
}
