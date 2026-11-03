#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Jump.c (TARGET_PC build): ground jump. Every
//! function of the TU; calls out go through `ext`. Checked against the C by tests/tu_ftCo_Jump.rs.

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

pub const HSD_PAD_XY: u32 = 0xC00;

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
        // TARGET_PC `FT_TAP_JUMP_OK(fp)` = `!FighterControls_TapJumpOff(fp->player_id)`.
        FighterControls_TapJumpOff(port: i32) -> i32 = |port: i32| -> i32 { unsafe { crate::platform::FighterControls_TapJumpOff(port) } };
        // TARGET_PC `FT_SCRIPT_JUMP(fp, field, base)` = `ScriptGame_ModJump(fp->player_id, ...)`.
        ScriptGame_ModJump(slot: i32, field: i32, base: f32) -> f32 = |_, _, _| todo!("TODO(port) ScriptGame_ModJump (pc/gameworld/script_game.c)");
        ftCo_800C5240(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C5240(g as _) as _ } };
        ftCo_800C5A50(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_HammerJump::ftCo_800C5A50(g as _) as _ } };
        ftCo_KneeBend_Enter(g: G, input: i32) = |g: G, input: i32| { unsafe { crate::tu::ftCo_KneeBend::ftCo_KneeBend_Enter(g as _, input as _); } };
        ftCo_800DF910(fp: &mut Fighter) -> bool = |a0| crate::tu::ft_0df1::ftco_800df910(a0);
        ft_800881D8(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |a0, a1, a2, a3| crate::tu::ft_0881::ft_800881d8(a0, a1, a2, a3);
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |a0, a1, a2, a3| crate::ft0877::play_sfx(a0, a1, a2, a3);
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_Fall_Enter(g: G) = |g: G| { unsafe { crate::tu::ftCo_Fall::ftCo_Fall_Enter(g as _); } };
        ftCo_SpecialAir_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialAir::ftCo_SpecialAir_CheckInput(g as _) as _ } };
        ftCo_80095328(g: G, out: *mut bool) -> bool = |g: G, out: *mut bool| -> bool { unsafe { crate::tu::ftCo_ItemThrow::ftCo_80095328(g as _, out as _) as _ } };
        ftCo_800D7100(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Attack100::ftCo_800D7100(g as _) as _ } };
        ftCo_800C3B10(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftco_aircatch::ftco_800c3b10(g as _) as _ } };
        ftCo_80099A58(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_EscapeAir::ftCo_80099A58(g as _) as _ } };
        ftCo_AttackAir_CheckItemThrowInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_AttackAir::ftCo_AttackAir_CheckItemThrowInput(g as _) as _ } };
        ftCo_800D705C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Attack100::ftCo_800D705C(g as _) as _ } };
        ftCo_800CEE70(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_ItemParasolOpen::ftCo_800CEE70(g as _) as _ } };
        ftPe_8011BA54(g: G) -> bool = |_| todo!("TODO(port) ftPe_8011BA54 (ftpeachfloat.c)");
        ftCo_800CB870(g: G) -> bool = crate::tu::ftCo_JumpAerial::ftCo_800CB870;
        ftPe_8011BAD8(g: G) -> bool = |_| todo!("TODO(port) ftPe_8011BAD8 (ftpeachfloat.c)");
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_800835B0(g: G, a: fn(G, i32) -> bool, b: fn(G)) = |g: G, a: fn(G, i32) -> bool, b: fn(G)| { unsafe { crate::tu::ft_081b::ft_800835b0(g as _, a as _, b as _); } };
        ftCo_80096CC8(g: G, line: i32) -> bool = |g: G, line: i32| -> bool { unsafe { crate::tu::ftCo_FallSpecial::ftCo_80096CC8(g as _, line as _) as _ } };
        ft_80082B1C(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80082b1c(g as _); } };
    }
}

#[inline(always)]
pub(crate) fn tap_jump_ok(f: &Fighter) -> bool {
    melee_core::ct!("tap_jump_ok");
    ext::FighterControls_TapJumpOff(f.player_id as i32) == 0
}

pub fn ftCo_Jump_GetInput(g: G) -> ftCo_JumpInput {
    melee_core::ct!("ftCo_Jump_GetInput");
    let f = fp(g);
    if tap_jump_ok(f)
        && f.input.lstick[0].y >= cd().tap_jump_threshold
        && (f.active_timer.lstick.y as i32) < cd().tap_jump_window
    {
        return JumpInput_LStick;
    }
    if f.input.pressed_buttons & HSD_PAD_XY != 0 {
        return JumpInput_XY;
    }
    JumpInput_None
}

pub fn ftCo_Jump_CheckInput(g: G) -> bool {
    melee_core::ct!("ftCo_Jump_CheckInput");
    if ext::ftCo_800C5240(g) {
        return ext::ftCo_800C5A50(g);
    }
    let ji = ftCo_Jump_GetInput(g);
    if ji != 0 {
        ext::ftCo_KneeBend_Enter(g, ji as i32);
        return true;
    }
    false
}

pub fn fn_800CAF78(g: G) -> bool {
    melee_core::ct!("fn_800CAF78");
    if ext::ftCo_800C5240(g) {
        return ext::ftCo_800C5A50(g);
    }
    let f = fp(g);
    if tap_jump_ok(f)
        && f.input.lstick[0].y >= cd().relaxed_tap_jump_threshold
        && (f.active_timer.lstick.y as i32) < cd().tap_jump_window
    {
        ext::ftCo_KneeBend_Enter(g, JumpInput_LStick as i32);
        return true;
    }
    if fp(g).input.pressed_buttons & HSD_PAD_XY != 0 {
        ext::ftCo_KneeBend_Enter(g, JumpInput_XY as i32);
        return true;
    }
    false
}

pub fn ftCo_800CB024(g: G) -> bool {
    melee_core::ct!("ftCo_800CB024");
    if ftCo_Jump_CheckInput(g) {
        return true;
    }
    if ext::ftCo_800DF910(fp(g)) {
        ext::ftCo_KneeBend_Enter(g, JumpInput_CStick as i32);
        return true;
    }
    false
}

pub fn ftCo_800CB110(g: G, arg1: bool, jump_mul: f32) {
    melee_core::ct!("ftCo_800CB110");
    let f = fp(g);
    f.mv.co.jump.x4 = 0;
    unsafe {
        f.mv.co.jump.jump_mul = jump_mul;
        f.self_vel.x *= f.co_attrs.ground_to_air_jump_momentum_multiplier * f.mv.co.jump.jump_mul;
    }
    f.self_vel.y *= cd().x438;
    f.self_vel.z = 0.0;
    unsafe {
        let h_init_v = f.input.lstick[0].x * f.co_attrs.jump_h_initial_velocity;
        let mut h_vel = f.mv.co.jump.jump_mul * h_init_v;
        let pid = f.player_id as i32;
        let v_init_v = if f.mv.co.jump.x0 != 0 {
            ext::ScriptGame_ModJump(pid, 5, f.co_attrs.hop_v_initial_velocity) * f.mv.co.jump.jump_mul
        } else {
            ext::ScriptGame_ModJump(pid, 5, f.co_attrs.jump_v_initial_velocity) * f.mv.co.jump.jump_mul
        };
        f.self_vel.y = v_init_v;
        h_vel = f.self_vel.x + h_vel;
        let h_max_vel = f.co_attrs.jump_h_max_velocity * f.mv.co.jump.jump_mul;
        let abs = if h_vel < 0.0 { -h_vel } else { h_vel };
        if abs > h_max_vel {
            h_vel = if h_vel < 0.0 { -h_max_vel } else { h_max_vel };
        }
        f.self_vel.x = h_vel;
    }
    f.active_timer.lstick.y = 0xFE;
    if arg1 {
        let id = unsafe { (*(*f.ft_data).x4C_sfx).x10 };
        ext::ft_800881D8(f, id, 127, 64);
        if !fp(g).x197C.is_null() {
            ext::ft_PlaySFX(fp(g), 0x11A, 127, 64);
        }
    }
}

pub fn ftCo_Jump_Enter(g: G) {
    melee_core::ct!("ftCo_Jump_Enter");
    // mod host site (hooks-v1 0x800CB250, melee_core::modhook): one None check when no mod hooks it
    melee_core::mod_hook!(0x800CB250, [melee_core::modhook::Arg::Fighter(g as _)], ftCo_Jump_Enter__orig(g));
    #[allow(unused_unsafe)] unsafe { ftCo_Jump_Enter__orig(g) }
}

fn ftCo_Jump_Enter__orig(g: G) {
    ext::ftCommon_8007D5D4(fp(g));
    let f = fp(g);
    let msid = if f.input.lstick[0].x * f.facing_dir > -cd().x78 { ftCo_MS_JumpF } else { ftCo_MS_JumpB };
    ext::Fighter_ChangeMotionState(g, msid as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftCo_800CB110(g, true, 1.0);
    fp(g).set_x2227_b0(1);
}

pub fn ftCo_Jump_Anim(g: G) {
    melee_core::ct!("ftCo_Jump_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_Fall_Enter(g);
    }
}

pub fn ftCo_Jump_IASA(g: G) {
    melee_core::ct!("ftCo_Jump_IASA");
    if ext::ftCo_SpecialAir_CheckInput(g) { return; }
    if ext::ftCo_80095328(g, core::ptr::null_mut()) { return; }
    if ext::ftCo_800D7100(g) { return; }
    if ext::ftCo_800C3B10(g) { return; }
    if ext::ftCo_80099A58(g) { return; }
    if ext::ftCo_AttackAir_CheckItemThrowInput(g) { return; }
    if ext::ftCo_800D705C(g) { return; }
    if unsafe { fp(g).mv.co.jump.x4 } != 0 && ext::ftCo_800CEE70(g) { return; }
    if unsafe { fp(g).mv.co.jump.x4 } != 0 && ext::ftPe_8011BA54(g) { return; }
    if ext::ftCo_800CB870(g) { return; }
    if unsafe { fp(g).mv.co.jump.x4 } == 0 { return; }
    if ext::ftPe_8011BAD8(g) { return; }
}

pub fn ftCo_Jump_Phys_Inner(g: G) {
    melee_core::ct!("ftCo_Jump_Phys_Inner");
    let f = fp(g);
    unsafe {
        if f.mv.co.jump.x4 == 0 {
            f.mv.co.jump.x4 = 1;
            return;
        }
    }
    ext::ft_80084DB0(g);
}

pub fn ftCo_Jump_Phys(g: G) {
    melee_core::ct!("ftCo_Jump_Phys");
    ftCo_Jump_Phys_Inner(g);
}

pub fn ftCo_Jump_Coll(g: G) {
    melee_core::ct!("ftCo_Jump_Coll");
    ext::ft_800835B0(g, ext::ftCo_80096CC8, ext::ft_80082B1C);
}
