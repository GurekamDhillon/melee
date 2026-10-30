#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Run.c (TARGET_PC build: every function of the TU). Calls out of it
//! go through `ext`. Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_Run.rs.
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
        // TARGET_PC `FT_CAPS_ACTION_ALLOWED(fp, flag)` = `ScriptGame_CapsActionAllowed`.
        ScriptGame_CapsActionAllowed(fp: &Fighter, flag: i32) -> i32 = |_, _| todo!("TODO(port) ScriptGame_CapsActionAllowed (pc/gameworld/script_game.c)");
        // TARGET_PC `FT_SCRIPT_VALUE(fp, field, base)` = `ScriptGame_ModValue(fp->player_id, ...)`.
        ScriptGame_ModValue(slot: i32, field: i32, base: f32) -> f32 = |_, _, _| todo!("TODO(port) ScriptGame_ModValue (pc/gameworld/script_game.c)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ft_GetGroundFrictionMultiplier(fp: &mut Fighter) -> f32 = |fp: &mut Fighter| -> f32 { unsafe { crate::tu::ft_081b::ft_get_ground_friction_multiplier(fp) } };
        ftAnim_SetAnimRate(g: G, r: f32) = |g: G, r: f32| { unsafe { let _ = crate::tu::ftanim::ftAnim_SetAnimRate(g as _, r as _); } };
        ftCo_SpecialS_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput(g as _) as _ } };
        ftCo_Attack100_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput;
        ftCo_800D6824(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D6824;
        ftCo_800D68C0(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D68C0;
        ftCo_800D8A38(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Catch::ftCo_800D8A38(g as _) as _ } };
        ftCo_AttackDash_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackDash::ftCo_AttackDash_CheckInput;
        ftCo_AttackDash_SetMv0(g: G) = crate::tu::ftCo_AttackDash::ftCo_AttackDash_SetMv0;
        ftCo_80091A4C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Guard::ftCo_80091A4C(g as _) as _ } };
        ftCo_80091B90(g: G, v: i32) = |g: G, v: i32| { unsafe { let _ = crate::tu::ftCo_Guard::ftCo_80091B90(g as _, v as _); } };
        ftCo_80091B9C(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Guard::ftCo_80091B9C(g as _); } };
        ftCo_800DE9D8(g: G) -> bool = crate::tu::ftCo_AppealS::ftCo_800DE9D8;
        fn_800CAF78(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Jump::fn_800CAF78(g as _) as _ } };
        fn_800C9D40(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_TurnRun::fn_800C9D40(g as _) as _ } };
        ftCo_RunBrake_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_RunBrake::ftCo_RunBrake_CheckInput(g as _) as _ } };
        ftCommon_CalcGroundAccel_DashRun(fp: &mut Fighter, accel: f32, target: f32, friction: f32) = crate::ftcommon::calc_ground_accel_dash_run;
        ftCommon_SetSelfMovementFromGroundedMovement(g: G) = |g| crate::ftcommon::set_self_movement_from_grounded_movement(super::fp(g));
        ft_800844EC(g: G) = crate::tu::ft_081b::ft_800844ec;
    }
}

pub fn fn_800CA5F0(g: G) -> bool {
    melee_core::ct!("fn_800CA5F0");
    if ext::ScriptGame_CapsActionAllowed(fp(g), 4) == 0 {
        return false;
    }
    let f = fp(g);
    if f.input.lstick[0].x * f.facing_dir >= cd().x58_someLStickXThreshold {
        ftCo_Run_Enter(g, 0.0);
        return true;
    }
    false
}

pub fn fn_800CA644(g: G) -> bool {
    melee_core::ct!("fn_800CA644");
    if ext::ScriptGame_CapsActionAllowed(fp(g), 4) == 0 {
        return false;
    }
    let f = fp(g);
    if f.input.lstick[0].x * f.facing_dir >= cd().x58_someLStickXThreshold {
        ftCo_Run_Enter(g, cd().x430);
        return true;
    }
    false
}

pub fn fn_800CA698(g: G) -> bool {
    melee_core::ct!("fn_800CA698");
    if ext::ScriptGame_CapsActionAllowed(fp(g), 4) == 0 {
        return false;
    }
    let f = fp(g);
    if f.input.lstick[0].x * f.facing_dir >= cd().x58_someLStickXThreshold {
        let (frame, speed) = (f.cur_anim_frame, f.frame_speed_mul);
        ftCo_Run_Enter_Full(g, 0.0, frame, speed);
        return true;
    }
    false
}

pub fn ftCo_Run_Enter(g: G, arg0: f32) {
    melee_core::ct!("ftCo_Run_Enter");
    ftCo_Run_Enter_Full(g, arg0, 0.0, 1.0);
}

pub fn ftCo_Run_Enter_Full(g: G, arg0: f32, anim_start: f32, anim_speed: f32) {
    melee_core::ct!("ftCo_Run_Enter_Full");
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Run as i32, Ft_MF_None as u32, anim_start, anim_speed, 0.0, core::ptr::null_mut());
    let f = fp(g);
    unsafe {
        f.mv.co.run.x0 = arg0;
        f.mv.co.run.x4 = f.gr_vel;
    }
}

pub fn ftCo_Run_Anim(g: G) {
    melee_core::ct!("ftCo_Run_Anim");
    let vel = if ext::ft_GetGroundFrictionMultiplier(fp(g)) < 1.0 {
        unsafe { fp(g).mv.co.run.x4 }
    } else {
        fp(g).gr_vel
    };
    let f = fp(g);
    let anim_rate = if vel * f.facing_dir <= 0.0 {
        0.0
    } else {
        (if vel < 0.0 { -vel } else { vel }) / f.co_attrs.run_animation_scaling
    };
    ext::ftAnim_SetAnimRate(g, anim_rate);
    let f = fp(g);
    unsafe {
        if f.mv.co.run.x0 > 0.0 {
            f.mv.co.run.x0 -= 1.0;
        }
    }
}

pub fn ftCo_Run_IASA(g: G) {
    melee_core::ct!("ftCo_Run_IASA");
    if ext::ftCo_SpecialS_CheckInput(g)
        || ext::ftCo_Attack100_CheckInput(g)
        || ext::ftCo_800D6824(g)
        || ext::ftCo_800D68C0(g)
        || ext::ftCo_800D8A38(g)
    {
        return;
    }
    if ext::ftCo_AttackDash_CheckInput(g) {
        ext::ftCo_AttackDash_SetMv0(g);
        return;
    }
    if ext::ftCo_80091A4C(g) {
        ext::ftCo_80091B90(g, cd().x410);
        ext::ftCo_80091B9C(g);
        return;
    }
    if ext::ftCo_800DE9D8(g) || ext::fn_800CAF78(g) {
        return;
    }
    if unsafe { fp(g).mv.co.run.x0 } <= 0.0 && ext::fn_800C9D40(g) {
        return;
    }
    if !(unsafe { fp(g).mv.co.run.x0 } <= 0.0) {
        return;
    }
    ext::ftCo_RunBrake_CheckInput(g);
}

/// static inline `getAccelAndTarget` (ft/inlines.h), TARGET_PC: returns (accel, target_vel).
fn getAccelAndTarget(f: &mut Fighter) -> (f32, f32) {
    let lsx = f.input.lstick[0].x;
    let mut accel = lsx * f.co_attrs.dash_accel_mul;
    accel += if lsx > 0.0 { f.co_attrs.dash_accel_base } else { -f.co_attrs.dash_accel_base };
    let mut target_vel = lsx * f.co_attrs.dash_max_velocity;
    accel = ext::ScriptGame_ModValue(f.player_id as i32, 2, accel);
    target_vel = ext::ScriptGame_ModValue(f.player_id as i32, 2, target_vel);
    (accel, target_vel)
}

pub fn ftCo_Run_Phys(g: G) {
    melee_core::ct!("ftCo_Run_Phys");
    let (mut accel, target_vel) = getAccelAndTarget(fp(g));
    let f = fp(g);
    if target_vel != 0.0 {
        let gr_frac = f.gr_vel / target_vel;
        if gr_frac > 0.0 && gr_frac < 1.0 {
            accel *= (1.0 - gr_frac) * cd().run_accel_taper_gain;
        }
    }
    unsafe { f.mv.co.run.x4 = target_vel * cd().x440 };
    let fr = f.co_attrs.ground_friction * cd().run_dash_turn_friction_multiplier;
    ext::ftCommon_CalcGroundAccel_DashRun(f, accel, target_vel, fr);
    ext::ftCommon_SetSelfMovementFromGroundedMovement(g);
}

pub fn ftCo_Run_Coll(g: G) {
    melee_core::ct!("ftCo_Run_Coll");
    ext::ft_800844EC(g);
}
