#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_AttackS4.c (forward smash; plus the ft/inlines.h Fighter_SetEffectHitlagCallbacks it uses). Every function of the TU;
//! calls out of it go through `ext` (unported ones panic until rewired; ported ones default
//! to the port). Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_AttackS4.rs.


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
    macro_rules! t { ($n:literal) => { |_| todo!(concat!("TODO(port) ", $n)) }; }
    ext_fns! {
        ftCo_GetLStickAngle(fp: &Fighter) -> f32 = crate::ftcommon3::ftco_get_lstick_angle;
        ftCo_GetCStickAngle(fp: &Fighter) -> f32 = crate::ftcommon3::ftco_get_cstick_angle;
        ftCo_800DF1C8(fp: &Fighter) -> bool = crate::ft0df::ftco_800df1c8;
        ftCo_800DF21C(fp: &Fighter) -> bool = crate::ft0df::ftco_800df21c;
        it_8026B30C(i: *mut Item_GObj) -> i32 = crate::hooks::it_8026b30c;
        it_8026B594(i: *mut Item_GObj) -> bool = |a0| unsafe { melee_items::tu::it_26B1::it_8026B594(a0 as _) };
        ftCo_800957F4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { crate::tu::ftCo_ItemThrow::ftCo_800957F4(g as _, msid as _); } };
        ftCo_Attack_800CCF58(g: G, a: i32) = |g: G, a: i32| { unsafe { crate::tu::ftswing::ftCo_Attack_800CCF58(g as _, a as _); } };
        ftCo_Attack_800CDD14(g: G) = t!("ftCo_Attack_800CDD14 (ft_0CDD.c)");
        Mex_HasFSmashHook(kind: i32) -> bool = |kind: i32| -> bool { unsafe { crate::platform::Mex_HasFSmashHook(kind) } };
        Mex_OnFSmashDispatch(kind: i32, g: G, vanilla: *mut core::ffi::c_void) = |kind: i32, g: G, vanilla: *mut core::ffi::c_void| { unsafe { let _ = crate::platform::mex_dispatch(crate::platform::MEX_EVENT_ON_FSMASH, kind, g, vanilla); } };
        ftNs_AttackS4_Enter(g: G) = t!("ftNs_AttackS4_Enter (ftnessattacks4.c)");
        ftPe_AttackS4_Enter(g: G) = t!("ftPe_AttackS4_Enter (ftpeachattacks4.c)");
        ftGw_AttackS4_Enter(g: G) = t!("ftGw_AttackS4_Enter (ftgamewatchattacks4.c)");
        efLib_PauseAll_fn() -> HSD_GObjEvent = || todo!("TODO(port) &efLib_PauseAll (eflib.c)");
        efLib_ResumeAll_fn() -> HSD_GObjEvent = || todo!("TODO(port) &efLib_ResumeAll (eflib.c)");
        ftData_80085FD4(fp: &mut Fighter, msid: i32) -> *mut Fighter_WaitAnimData = |fp: &mut Fighter, msid: i32| -> *mut Fighter_WaitAnimData { unsafe { crate::tu::ftdata::ftData_80085FD4(fp as _, msid as _) as _ } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = t!("ftAnim_8006EBA4 (ftanim.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ft_80084FA8(g: G) = crate::ft084e::ft_80084fa8;
        ft_80084104(g: G) = t!("ft_80084104 (ft_081B.c)");
        ftCo_SpecialS_CheckInput(g: G) -> bool = t!("ftCo_SpecialS_CheckInput");
        ftCo_Attack100_CheckInput(g: G) -> bool = t!("ftCo_Attack100_CheckInput");
        ftCo_800D6824(g: G) -> bool = t!("ftCo_800D6824");
        ftCo_800D68C0(g: G) -> bool = t!("ftCo_800D68C0");
        ftCo_Catch_CheckInput(g: G) -> bool = t!("ftCo_Catch_CheckInput");
        ftCo_800CECE8(g: G) -> bool = t!("ftCo_800CECE8");
        ftCo_AttackHi4_CheckInput(g: G) -> bool = t!("ftCo_AttackHi4_CheckInput (ftCo_AttackHi4.c)");
        ftCo_AttackLw4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput;
        ftCo_AttackS3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput;
        ftCo_AttackHi3_CheckInput(g: G) -> bool = t!("ftCo_AttackHi3_CheckInput");
        ftCo_AttackLw3_CheckInput(g: G) -> bool = t!("ftCo_AttackLw3_CheckInput");
        ftCo_Attack1_CheckInput(g: G) -> bool = t!("ftCo_Attack1_CheckInput");
        ftCo_80091A4C(g: G) -> bool = t!("ftCo_80091A4C");
        ftCo_800DE9D8(g: G) -> bool = t!("ftCo_800DE9D8");
        ftCo_Jump_CheckInput(g: G) -> bool = t!("ftCo_Jump_CheckInput");
        ftCo_Dash_CheckInput(g: G) -> bool = t!("ftCo_Dash_CheckInput");
        ftCo_800D5FB0(g: G) -> bool = t!("ftCo_800D5FB0");
        ftCo_Turn_CheckInput(g: G) -> bool = t!("ftCo_Turn_CheckInput");
        ftCo_Walk_CheckInput(g: G) -> bool = t!("ftCo_Walk_CheckInput");
    }
}

/// `x >= 0 ? +1 : -1` (NaN gives -1).
#[inline(always)]
fn stick_sign(x: f32) -> f32 {
    if x >= 0.0 { 1.0 } else { -1.0 }
}

/// static `checkLStick`: A with a dash-smash flick.
pub fn checkLStick(fp: &Fighter) -> bool {
    melee_core::ct!("checkLStick");
    let c = cd();
    fp.input.pressed_buttons & HSD_PAD_A as u32 != 0
        && crate::ftcommon::abs_c(fp.input.lstick[0].x) >= c.dash_smash_stick_threshold
        && (fp.active_timer.lstick.x as i32) < c.dash_smash_window
}

/// `ftCo_AttackS4_CheckInput`.
pub fn ftCo_AttackS4_CheckInput(g: G) -> bool {
    melee_core::ct!("ftCo_AttackS4_CheckInput");
    let f = fp(g);
    let (sx, ang) = if checkLStick(f) {
        (stick_sign(f.input.lstick[0].x), ext::ftCo_GetLStickAngle(f))
    } else if ext::ftCo_800DF1C8(f) {
        (stick_sign(f.input.cstick[0].x), ext::ftCo_GetCStickAngle(f))
    } else {
        return false;
    };
    if checkItemThrow(g, sx) {
        return true;
    }
    decideFighter(g, sx, ang);
    true
}

/// static `checkFacingDir`.
pub fn checkFacingDir(fp: &Fighter) -> bool {
    melee_core::ct!("checkFacingDir");
    fp.input.pressed_buttons & HSD_PAD_A as u32 != 0
        && fp.input.lstick[0].x * fp.facing_dir >= cd().dash_smash_stick_threshold
}

/// `ftCo_AttackS4_8008C114`.
pub fn ftCo_AttackS4_8008C114(g: G) -> bool {
    melee_core::ct!("ftCo_AttackS4_8008C114");
    let f = fp(g);
    let (sx, ang) = if checkFacingDir(f) {
        (f.facing_dir, ext::ftCo_GetLStickAngle(f))
    } else if ext::ftCo_800DF1C8(f) {
        (stick_sign(f.input.cstick[0].x), ext::ftCo_GetCStickAngle(f))
    } else {
        return false;
    };
    if checkItemThrow(g, sx) {
        return true;
    }
    decideFighter(g, sx, ang);
    true
}

/// static `checkItemThrow`.
pub fn checkItemThrow(g: G, sx: f32) -> bool {
    melee_core::ct!("checkItemThrow");
    let f = fp(g);
    if !f.item_gobj.is_null() {
        if f.input.held_buttons[0] & HSD_PAD_LR != 0
            || ext::it_8026B30C(f.item_gobj) == 0
            || (ext::it_8026B30C(f.item_gobj) == 3 && ext::it_8026B594(f.item_gobj))
            || ext::ftCo_800DF21C(f)
        {
            let ms = if sx * f.facing_dir >= 0.0 { ftCo_MS_LightThrowF4 } else { ftCo_MS_LightThrowB4 };
            ext::ftCo_800957F4(g, ms as i32);
            return true;
        }
        match ext::it_8026B30C(f.item_gobj) {
            2 => {
                f.facing_dir = sx;
                ext::ftCo_Attack_800CCF58(g, 2);
                return true;
            }
            3 => {
                f.facing_dir = sx;
                ext::ftCo_Attack_800CDD14(g);
                return true;
            }
            _ => {}
        }
    }
    false
}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
pub fn Fighter_SetEffectHitlagCallbacks(f: &mut Fighter) {
    melee_core::ct!("Fighter_SetEffectHitlagCallbacks");
    f.pre_hitlag_cb = ext::efLib_PauseAll_fn();
    f.post_hitlag_cb = ext::efLib_ResumeAll_fn();
}

/// `decideFighter` (TARGET_PC: an m-ex onFSmash hook replaces the per-kind switch).
pub fn decideFighter(g: G, sx: f32, ang: f32) {
    melee_core::ct!("decideFighter");
    let f = fp(g);
    f.facing_dir = sx;
    if ext::Mex_HasFSmashHook(f.kind as i32) {
        ext::Mex_OnFSmashDispatch(f.kind as i32, g, core::ptr::null_mut());
        return;
    }
    let k = f.kind as i32;
    if k == Ft_Kind_Ness as i32 {
        ext::ftNs_AttackS4_Enter(g);
    } else if k == Ft_Kind_Peach as i32 {
        ext::ftPe_AttackS4_Enter(g);
    } else if k == Ft_Kind_GameWatch as i32 {
        ext::ftGw_AttackS4_Enter(g);
    } else if k == Ft_Kind_Pikachu as i32 || k == Ft_Kind_Pichu as i32 {
        doEnter(g, ang);
        Fighter_SetEffectHitlagCallbacks(fp(g));
    } else {
        doEnter(g, ang);
    }
}

fn has_anim(f: &mut Fighter, msid: ftCommon_MotionState) -> bool {
    unsafe { (*ext::ftData_80085FD4(f, msid as i32)).x8 != 0 }
}

/// static `doEnter`.
pub fn doEnter(g: G, a: f32) {
    melee_core::ct!("doEnter");
    let f = fp(g);
    let c = cd();
    let msid = if a > c.xB8_radians && has_anim(f, ftCo_MS_AttackS4S) {
        ftCo_MS_AttackS4Hi
    } else if a > c.xBC_radians && has_anim(f, ftCo_MS_AttackS4LwS) {
        ftCo_MS_AttackS4HiS
    } else if a < c.xC4_radians && has_anim(f, ftCo_MS_AttackLw4) {
        ftCo_MS_AttackS4Lw
    } else if a < c.xC0_radians && has_anim(f, ftCo_MS_AttackHi4) {
        ftCo_MS_AttackS4LwS
    } else {
        ftCo_MS_AttackS4S
    };
    f.set_allow_interrupt(0);
    f.cmd_vars[0] = 0;
    f.x2210.throw_flags = 0;
    ext::Fighter_ChangeMotionState(g, msid as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
}

pub fn ftCo_AttackS4_Anim(g: G) {
    melee_core::ct!("ftCo_AttackS4_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

/// `ftCo_AttackS4_IASA`: the RETURN_IF chain.
pub fn ftCo_AttackS4_IASA(g: G) {
    melee_core::ct!("ftCo_AttackS4_IASA");
    if fp(g).allow_interrupt() != 0
        && (ext::ftCo_SpecialS_CheckInput(g)
            || ext::ftCo_Attack100_CheckInput(g)
            || ext::ftCo_800D6824(g)
            || ext::ftCo_800D68C0(g)
            || ext::ftCo_Catch_CheckInput(g))
    {
        return;
    }
    if ext::ftCo_800CECE8(g) {
        return;
    }
    if fp(g).allow_interrupt() != 0 {
        let _ = ftCo_AttackS4_CheckInput(g)
            || ext::ftCo_AttackHi4_CheckInput(g)
            || ext::ftCo_AttackLw4_CheckInput(g)
            || ext::ftCo_AttackS3_CheckInput(g)
            || ext::ftCo_AttackHi3_CheckInput(g)
            || ext::ftCo_AttackLw3_CheckInput(g)
            || ext::ftCo_Attack1_CheckInput(g)
            || ext::ftCo_80091A4C(g)
            || ext::ftCo_800DE9D8(g)
            || ext::ftCo_Jump_CheckInput(g)
            || ext::ftCo_Dash_CheckInput(g)
            || ext::ftCo_800D5FB0(g)
            || ext::ftCo_Turn_CheckInput(g)
            || ext::ftCo_Walk_CheckInput(g);
    }
}

pub fn ftCo_AttackS4_Phys(g: G) {
    melee_core::ct!("ftCo_AttackS4_Phys");
    ext::ft_80084FA8(g);
}

pub fn ftCo_AttackS4_Coll(g: G) {
    melee_core::ct!("ftCo_AttackS4_Coll");
    ext::ft_80084104(g);
}
