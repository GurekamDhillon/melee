#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_TurnRun.c (every function, TARGET_PC build:
//! the script caps gate and FT_SCRIPT_VALUE in getAccelAndTarget). Calls out of the TU go through
//! `ext` (unported ones panic until rewired). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_TurnRun.rs.
use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
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
        // TARGET_PC `FT_CAPS_ACTION_ALLOWED(fp, flag)` = `ScriptGame_CapsActionAllowed`.
        ScriptGame_CapsActionAllowed(fp: &Fighter, flag: i32) -> i32 = |_, _| todo!("TODO(port) ScriptGame_CapsActionAllowed (pc/gameworld/script_game.c)");
        // TARGET_PC `FT_SCRIPT_VALUE(fp, field, base)` = `ScriptGame_ModValue(fp->player_id, ...)`.
        ScriptGame_ModValue(slot: i32, field: i32, base: f32) -> f32 = |_, _, _| todo!("TODO(port) ScriptGame_ModValue (pc/gameworld/script_game.c)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |_, _, _, _| todo!("ft_PlaySFX: installed by melee_link");
        ftAnim_SetAnimRate(g: G, rate: f32) = |g, r| unsafe { crate::tu::ftanim::ftAnim_SetAnimRate(g, r) };
        ftAnim_IsFramesRemaining(g: G) -> bool = |g| unsafe { crate::tu::ftanim::ftAnim_IsFramesRemaining(g) };
        fn_800CA644(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Run::fn_800CA644(g as _) as _ } };
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        fn_800CAF78(g: G) -> bool = crate::tu::ftCo_Jump::fn_800CAF78;
        ftCommon_CalcGroundAccel_Deaccel(fp: &mut Fighter, friction: f32) = crate::ftcommon::calc_ground_accel_deaccel;
        ftCommon_SetSelfMovementFromGroundedMovement(g: G) = |_| todo!("ftCommon_SetSelfMovementFromGroundedMovement: installed by melee_link");
        ft_800827A0(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ft_081b::ft_800827a0(g as _) as _ } };
        ftCo_Fall_Enter(g: G) = crate::tu::ftCo_Fall::ftCo_Fall_Enter;
        ftCommon_8007E2FC(g: G) = |_| todo!("ftCommon_8007E2FC: installed by melee_link");
    }
}

pub fn fn_800C9CEC(g: G) -> bool {
    melee_core::ct!("fn_800C9CEC");
    if ext::ScriptGame_CapsActionAllowed(fp(g), 4) == 0 {
        return false;
    }
    let f = fp(g);
    if f.input.lstick[0].x * f.facing_dir <= cd().x38_someLStickXThreshold {
        let start = f.cur_anim_frame;
        ftCo_TurnRun_Enter(g, start);
        return true;
    }
    false
}

pub fn fn_800C9D40(g: G) -> bool {
    melee_core::ct!("fn_800C9D40");
    if ext::ScriptGame_CapsActionAllowed(fp(g), 4) == 0 {
        return false;
    }
    let f = fp(g);
    if f.input.lstick[0].x * f.facing_dir <= cd().x38_someLStickXThreshold {
        ftCo_TurnRun_Enter(g, 0.0);
        return true;
    }
    false
}

pub fn ftCo_TurnRun_Enter(g: G, anim_start: f32) {
    melee_core::ct!("ftCo_TurnRun_Enter");
    // mod host site (hooks-v1 0x800C9D94, melee_core::modhook): one None check when no mod hooks it
    melee_core::mod_hook!(0x800C9D94, [melee_core::modhook::Arg::Fighter(g as _), melee_core::modhook::Arg::F32(anim_start)], ftCo_TurnRun_Enter__orig(g, anim_start));
    #[allow(unused_unsafe)] unsafe { ftCo_TurnRun_Enter__orig(g, anim_start) }
}

fn ftCo_TurnRun_Enter__orig(g: G, anim_start: f32) {
    let f = fp(g);
    f.cmd_vars[1] = 0;
    unsafe { f.mv.co.turnrun.accel_mul = f.facing_dir };
    ext::Fighter_ChangeMotionState(g, ftCo_MS_TurnRun as i32, Ft_MF_SkipAnimVel, anim_start, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    unsafe { f.mv.co.turnrun.x14 = 0 };
    if !f.x197C.is_null() {
        ext::ft_PlaySFX(f, 0x119, 0x7F, 0x40);
    }
}

pub fn ftCo_TurnRun_Anim(g: G) {
    melee_core::ct!("ftCo_TurnRun_Anim");
    if fp(g).cmd_vars[1] != 0 {
        if unsafe { fp(g).mv.co.turnrun.x14 } == 0 {
            ext::ftAnim_SetAnimRate(g, 0.0);
            unsafe { fp(g).mv.co.turnrun.x14 = 1 };
        } else {
            let f = fp(g);
            if unsafe { f.mv.co.walk.middle_anim_frame } * f.gr_vel <= 0.01 {
                ext::ftAnim_SetAnimRate(g, 1.0);
                let f = fp(g);
                f.cmd_vars[1] = 0;
                f.facing_dir = -f.facing_dir;
            }
        }
    }
    if !ext::ftAnim_IsFramesRemaining(g) && !ext::fn_800CA644(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_TurnRun_IASA(g: G) {
    melee_core::ct!("ftCo_TurnRun_IASA");
    if ext::fn_800CAF78(g) {
        return;
    }
}

/// static inline `getAccelAndTarget` (ft/inlines.h), TARGET_PC: returns (accel, target_vel).
fn get_accel_and_target(f: &Fighter) -> (f32, f32) {
    let lsx = f.input.lstick[0].x;
    let mut accel = lsx * f.co_attrs.dash_accel_mul;
    accel += if lsx > 0.0 { f.co_attrs.dash_accel_base } else { -f.co_attrs.dash_accel_base };
    let mut target_vel = lsx * f.co_attrs.dash_max_velocity;
    accel = ext::ScriptGame_ModValue(f.player_id as i32, 2, accel);
    target_vel = ext::ScriptGame_ModValue(f.player_id as i32, 2, target_vel);
    (accel, target_vel)
}

pub fn ftCo_TurnRun_Phys(g: G) {
    melee_core::ct!("ftCo_TurnRun_Phys");
    let f = fp(g);
    let (mut accel, target_vel) = get_accel_and_target(f);
    let f = fp(g);
    let gf = f.co_attrs.ground_friction;
    let mul = cd().run_dash_turn_friction_multiplier;
    if target_vel == 0.0 {
        ext::ftCommon_CalcGroundAccel_Deaccel(f, gf * mul);
    } else if unsafe { f.mv.co.turnrun.accel_mul } * accel < 0.0 {
        if accel > 0.0 {
            if f.gr_vel + accel > target_vel {
                // `accel -= a * b`: one fmuladd -> fmadds.
                accel = fmadds(-gf, mul, accel);
                if f.gr_vel + accel < target_vel {
                    accel = target_vel - f.gr_vel;
                }
            }
        } else if f.gr_vel + accel < target_vel {
            accel = fmadds(gf, mul, accel);
            if f.gr_vel + accel > target_vel {
                accel = target_vel - f.gr_vel;
            }
        }
        f.xE4_ground_accel_1 = accel;
    } else {
        ext::ftCommon_CalcGroundAccel_Deaccel(f, gf * mul);
    }
    ext::ftCommon_SetSelfMovementFromGroundedMovement(g);
}

pub fn ftCo_TurnRun_Coll(g: G) {
    melee_core::ct!("ftCo_TurnRun_Coll");
    if !ext::ft_800827A0(g) {
        ext::ftCo_Fall_Enter(g);
        return;
    }
    let env = fp(g).coll_data.env_flags as i32;
    let var_r4 = !(env & Collide_LeftEdge == 0 && env & Collide_RightEdge == 0);
    if var_r4 {
        ext::ftCommon_8007E2FC(g);
    }
}
