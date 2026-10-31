#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_RunBrake.c (TARGET_PC build). Every function of the TU;
//! calls out of it go through `ext` (unported ones panic until rewired). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_RunBrake.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[allow(dead_code)]
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
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

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    ext_fns! {
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |fp: &mut Fighter, id: i32, vol: u8, pan: u8| { unsafe { let _ = crate::ft0877::play_sfx(fp, id as _, vol as _, pan as _); } };
        ftAnim_SetAnimRate(g: G, rate: f32) = |g: G, rate: f32| { unsafe { crate::tu::ftanim::ftAnim_SetAnimRate(g as _, rate as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        fn_800CAF78(g: G) -> bool = crate::tu::ftCo_Jump::fn_800CAF78;
        fn_800C9CEC(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_TurnRun::fn_800C9CEC(g as _) as _ } };
        ftCo_800D5FB0(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Squat::ftCo_800D5FB0(g as _) as _ } };
        ftCommon_CalcGroundAccel_Deaccel(f: &mut Fighter, friction: f32) = crate::ftcommon::calc_ground_accel_deaccel;
        ftCommon_SetSelfMovementFromGroundedMovement(g: G) = |g| crate::ftcommon::set_self_movement_from_grounded_movement(super::fp(g));
        ft_80084280(g: G) = crate::tu::ft_081b::ft_80084280;
    }
}

pub fn ftCo_RunBrake_CheckInput(g: G) -> bool {
    melee_core::ct!("ftCo_RunBrake_CheckInput");
    let mut lsx_abs = fp(g).input.lstick[0].x;
    if lsx_abs < 0.0 {
        lsx_abs = -lsx_abs;
    }
    if lsx_abs < cd().x58_someLStickXThreshold {
        ftCo_RunBrake_Enter(g);
        return true;
    }
    false
}

pub fn ftCo_RunBrake_Enter(g: G) {
    melee_core::ct!("ftCo_RunBrake_Enter");
    let f = fp(g);
    f.cmd_vars[0] = 0;
    f.cmd_vars[1] = 0;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_RunBrake as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    unsafe {
        f.mv.co.runbrake.x0 = 0;
        f.mv.co.runbrake.frames = f.co_attrs.max_run_brake_frames;
    }
    if !f.x197C.is_null() {
        ext::ft_PlaySFX(f, 0x119, 0x7F, 0x40);
    }
}

#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

pub fn ftCo_RunBrake_Anim(g: G) {
    melee_core::ct!("ftCo_RunBrake_Anim");
    if fp(g).cmd_vars[1] != 0 {
        let maybe_max_vel = cd().x42C;
        if unsafe { fp(g).mv.co.runbrake.x0 } == 0 {
            if abs_c(fp(g).gr_vel) >= maybe_max_vel {
                ext::ftAnim_SetAnimRate(g, 0.0);
                unsafe { fp(g).mv.co.runbrake.x0 = 1 };
            }
        } else if abs_c(fp(g).gr_vel) <= maybe_max_vel {
            ext::ftAnim_SetAnimRate(g, 1.0);
            fp(g).cmd_vars[1] = 0;
        }
    }
    let f = fp(g);
    unsafe {
        if f.mv.co.runbrake.frames != 0.0 {
            f.mv.co.runbrake.frames -= 1.0;
            if f.mv.co.runbrake.frames < 0.0 {
                f.mv.co.runbrake.frames = 0.0;
            }
        }
    }
    if !(ext::ftAnim_IsFramesRemaining(g) && unsafe { fp(g).mv.co.runbrake.frames } != 0.0) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_RunBrake_IASA(g: G) {
    melee_core::ct!("ftCo_RunBrake_IASA");
    if ext::fn_800CAF78(g) {
        return;
    }
    if fp(g).cmd_vars[0] != 0 && ext::fn_800C9CEC(g) {
        return;
    }
    if ext::ftCo_800D5FB0(g) {
        return;
    }
}

pub fn ftCo_RunBrake_Phys(g: G) {
    melee_core::ct!("ftCo_RunBrake_Phys");
    let f = fp(g);
    let friction = cd().run_dash_turn_friction_multiplier * f.co_attrs.ground_friction;
    ext::ftCommon_CalcGroundAccel_Deaccel(f, friction);
    ext::ftCommon_SetSelfMovementFromGroundedMovement(g);
}

pub fn ftCo_RunBrake_Coll(g: G) {
    melee_core::ct!("ftCo_RunBrake_Coll");
    ext::ft_80084280(g);
}
