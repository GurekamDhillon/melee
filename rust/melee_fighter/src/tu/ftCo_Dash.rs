#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Dash.c (TARGET_PC build). Checked against the
//! C by tests/tu_ftCo_Dash.rs.

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
        ftCo_Turn_Enter_Smash(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Turn::ftCo_Turn_Enter_Smash(g as _); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_800804A0(fp: &mut Fighter, v: f32) = crate::ftcommon2::ftcommon_800804a0;
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |_, _, _, _| todo!("TODO(port) ft_PlaySFX (ft_0881.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_SpecialS_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput(g as _) as _ } };
        ftCo_80094E54(fp: &mut Fighter) -> bool = |fp: &mut Fighter| -> bool { unsafe { crate::tu::ftCo_ItemThrow::ftCo_80094E54(fp as _) as _ } };
        ftCo_800957F4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { let _ = crate::tu::ftCo_ItemThrow::ftCo_800957F4(g as _, msid as _); } };
        ftCo_800D8A38(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Catch::ftCo_800D8A38(g as _) as _ } };
        ftCo_AttackS4_8008C114(g: G) -> bool = crate::tu::ftCo_AttackS4::ftCo_AttackS4_8008C114;
        ftCo_80099264(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Escape::ftCo_80099264(g as _) as _ } };
        ftCo_AttackDash_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackDash::ftCo_AttackDash_CheckInput;
        ftCo_AttackDash_SetMv0(g: G) = crate::tu::ftCo_AttackDash::ftCo_AttackDash_SetMv0;
        ftCo_80091AD8(g: G, n: i32) -> bool = |g: G, n: i32| -> bool { unsafe { crate::tu::ftCo_Guard::ftCo_80091AD8(g as _, n as _) as _ } };
        ftCo_80091A4C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Guard::ftCo_80091A4C(g as _) as _ } };
        ftCo_80091B9C(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Guard::ftCo_80091B9C(g as _); } };
        ftCo_800DE9D8(g: G) -> bool = crate::tu::ftCo_AppealS::ftCo_800DE9D8;
        fn_800CAF78(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Jump::fn_800CAF78(g as _) as _ } };
        fn_800CA5F0(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Run::fn_800CA5F0(g as _) as _ } };
        ft_GetGroundFrictionMultiplier(fp: &mut Fighter) -> f32 = |_| todo!("TODO(port) ft_GetGroundFrictionMultiplier (ft_081B.c)");
        ftCommon_CalcGroundAccel_DashRun(fp: &mut Fighter, accel: f32, target: f32, friction: f32) = crate::ftcommon::calc_ground_accel_dash_run;
        ftCommon_SetSelfMovementFromGroundedMovement(g: G) = |g| crate::ftcommon::set_self_movement_from_grounded_movement(super::fp(g));
        ft_800844EC(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_800844ec(g as _); } };
    }
}

pub fn ftCo_Dash_CheckInput(g: G) -> bool {
    if ext::ScriptGame_CapsActionAllowed(fp(g), 4) == 0 {
        return false;
    }
    let f = fp(g);
    let mut lsx_abs = f.input.lstick[0].x;
    if lsx_abs < 0.0 {
        lsx_abs = -lsx_abs;
    }
    if lsx_abs >= cd().dash_smash_stick_threshold && (f.active_timer.lstick.x as i32) < cd().dash_smash_window {
        if f.input.lstick[0].x * f.facing_dir < 0.0 {
            ext::ftCo_Turn_Enter_Smash(g);
        } else {
            ftCo_Dash_Enter(g, 1);
        }
        return true;
    }
    false
}

pub fn ftCo_Dash_Enter(g: G, arg1: i32) {
    fp(g).cmd_vars[0] = 0;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Dash as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    f.active_timer.lstick.x = 0xFE;
    let init_vel = f.facing_dir * f.co_attrs.dash_initial_velocity;
    let v = if f.gr_vel * f.facing_dir < 0.0 { init_vel } else { init_vel - f.gr_vel };
    unsafe { f.mv.co.dash.x0 = v };
    ext::ftCommon_800804A0(fp(g), v);
    let f = fp(g);
    unsafe { f.mv.co.dash.x4 = arg1 };
    if !f.x197C.is_null() {
        ext::ft_PlaySFX(f, 0x118, 0x7F, 0x40);
    }
}

pub fn ftCo_Dash_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

/// `block_42` of `ftCo_Dash_IASA`: true = return without applying friction.
fn block_42(g: G) -> bool {
    if !ext::ftCo_800DE9D8(g) {
        if ext::fn_800CAF78(g) {
            return true;
        }
        if fp(g).cmd_vars[0] == 0 {
            return true;
        }
        ext::fn_800CA5F0(g);
        return true;
    }
    false
}

pub fn ftCo_Dash_IASA(g: G) {
    let x4 = unsafe { fp(g).mv.co.dash.x4 };
    if x4 != 0 && fp(g).cur_anim_frame <= cd().x44 {
        if !ext::ftCo_SpecialS_CheckInput(g) {
            if !fp(g).item_gobj.is_null() && ext::ftCo_80094E54(fp(g)) {
                ext::ftCo_800957F4(g, ftCo_MS_LightThrowF4 as i32);
            } else if ext::ftCo_800D8A38(g) {
                return;
            } else if !ext::ftCo_AttackS4_8008C114(g) {
                if !(fp(g).cur_anim_frame <= cd().x48 && ext::ftCo_80099264(g)) {
                    if block_42(g) {
                        return;
                    }
                }
            }
        }
    } else if fp(g).cur_anim_frame <= cd().x4C {
        if !ext::ftCo_SpecialS_CheckInput(g) {
            if ext::ftCo_800D8A38(g) {
                return;
            }
            if ext::ftCo_AttackDash_CheckInput(g) {
                ext::ftCo_AttackDash_SetMv0(g);
                return;
            }
            let f = fp(g);
            if !(f.input.lstick[0].x * f.facing_dir < 0.0) || !ftCo_Dash_CheckInput(g) {
                let n = melee_runtime::cabi::f2i32((cd().x4C - fp(g).cur_anim_frame) as f64);
                if !ext::ftCo_80091AD8(g, n) {
                    if block_42(g) {
                        return;
                    }
                }
            }
        }
    } else {
        if ext::ftCo_800D8A38(g) {
            return;
        }
        if !ftCo_Dash_CheckInput(g) {
            if ext::ftCo_80091A4C(g) {
                ext::ftCo_80091B9C(g);
            } else if block_42(g) {
                return;
            }
        }
    }

    let friction = ext::ft_GetGroundFrictionMultiplier(fp(g));
    let f = fp(g);
    let temp_f0 = f.gr_vel * cd().x54;
    // `gr_vel += -temp_f0 * friction` is one fmuladd -> fmadds.
    f.gr_vel = melee_core::fp::fmadds(-temp_f0, friction, f.gr_vel);
}

/// static inline `getAccelAndTarget` (ft/inlines.h), TARGET_PC: returns (accel, target_vel).
pub fn getAccelAndTarget(f: &mut Fighter) -> (f32, f32) {
    let lsx = f.input.lstick[0].x;
    let mut accel = lsx * f.co_attrs.dash_accel_mul;
    accel += if lsx > 0.0 { f.co_attrs.dash_accel_base } else { -f.co_attrs.dash_accel_base };
    let mut target_vel = lsx * f.co_attrs.dash_max_velocity;
    accel = ext::ScriptGame_ModValue(f.player_id as i32, 2, accel);
    target_vel = ext::ScriptGame_ModValue(f.player_id as i32, 2, target_vel);
    (accel, target_vel)
}

pub fn ftCo_Dash_Phys(g: G) {
    let f = fp(g);
    if unsafe { f.mv.co.dash.x0 } != 0.0 {
        unsafe { f.mv.co.dash.x0 = 0.0 };
    } else {
        let (accel, target_vel) = getAccelAndTarget(f);
        let f = fp(g);
        let fr = f.co_attrs.ground_friction * cd().run_dash_turn_friction_multiplier;
        ext::ftCommon_CalcGroundAccel_DashRun(f, accel, target_vel, fr);
    }
    ext::ftCommon_SetSelfMovementFromGroundedMovement(g);
}

pub fn ftCo_Dash_Coll(g: G) {
    ext::ft_800844EC(g);
}
