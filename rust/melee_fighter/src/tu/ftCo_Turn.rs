#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Turn.c (every function, TARGET_PC build: UCF
//! 0.73/0.84 dashback and the script caps gate). Calls out of the TU go through `ext` (unported
//! ones panic until rewired). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ftCo_Turn.rs.
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
        ftCo_SpecialS_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput(g as _) as _ } };
        ftCo_800D68C0(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D68C0;
        ftCo_Attack100_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput;
        ftCo_Catch_CheckInput(g: G) -> bool = crate::tu::ftCo_Catch::ftCo_Catch_CheckInput;
        ftCo_AttackS4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS4::ftCo_AttackS4_CheckInput;
        ftCo_AttackHi4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInput;
        ftCo_AttackLw4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput;
        ftCo_AttackS3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput;
        ftCo_AttackHi3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi3::ftCo_AttackHi3_CheckInput;
        ftCo_AttackLw3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw3::ftCo_AttackLw3_CheckInput;
        ftCo_Attack1_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack1::ftCo_Attack1_CheckInput;
        ftCo_80091A4C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Guard::ftCo_80091A4C(g as _) as _ } };
        ftCo_800DE9D8(g: G) -> bool = crate::tu::ftCo_AppealS::ftCo_800DE9D8;
        ftCo_Jump_CheckInput(g: G) -> bool = crate::tu::ftCo_Jump::ftCo_Jump_CheckInput;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g| unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g) };
        ftAnim_IsFramesRemaining(g: G) -> bool = |g| unsafe { crate::tu::ftanim::ftAnim_IsFramesRemaining(g) };
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_Dash_Enter(g: G, arg1: i32) = crate::tu::ftCo_Dash::ftCo_Dash_Enter;
        ft_80084F3C(g: G) = |a0| crate::tu::ft_084e::ft_80084f3c(a0);
        ft_80083F88(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80083f88(g as _); } };
        // TARGET_PC `FT_CAPS_ACTION_ALLOWED(fp, flag)` = `ScriptGame_CapsActionAllowed`.
        ScriptGame_CapsActionAllowed(fp: &Fighter, flag: i32) -> i32 = |_, _| todo!("TODO(port) ScriptGame_CapsActionAllowed (pc/gameworld/script_game.c)");
        Replay_UcfVersion(port: i32) -> i32 = |port: i32| -> i32 { unsafe { crate::platform::Replay_UcfVersion(port) } };
        Replay_RawStickBack(port: i32, which: i32, back: i32) -> i32 = |port: i32, which: i32, back: i32| -> i32 { unsafe { crate::platform::Replay_RawStickBack(port, which, back) } };
        ftUcf_PadRaw(port: i32, which_y: i32, back: i32) -> i32 = |port: i32, which_y: i32, back: i32| -> i32 { unsafe { crate::fighter::ftucf_pad_raw(port as _, which_y as _, back as _) } };
        Player_GetEntityAtIndex(slot: i32, index: i32) -> G = crate::player::get_entity_at_index;
    }
}

macro_rules! ret_if { ($g:expr; $($n:ident),*) => { $( if ext::$n($g) { return; } )* } }

pub fn ftCo_800C97A8(g: G) -> bool {
    melee_core::ct!("ftCo_800C97A8");
    let f = fp(g);
    f.input.lstick[0].x * f.facing_dir <= cd().x34
}

pub fn ftCo_Turn_CheckInput(g: G) -> bool {
    melee_core::ct!("ftCo_Turn_CheckInput");
    if ftCo_800C97A8(g) {
        ftCo_Turn_Enter_Basic(g);
        return true;
    }
    false
}

pub fn ftCo_Turn_Enter(g: G, msid: i32, flags: u32, arg3: f32, frames_to_turn: f32, anim_start: f32) {
    melee_core::ct!("ftCo_Turn_Enter");
    // mod host site (hooks-v1 0x800C9840, melee_core::modhook): one None check when no mod hooks it
    melee_core::mod_hook!(0x800C9840, [melee_core::modhook::Arg::Fighter(g as _), melee_core::modhook::Arg::I32(msid as i32), melee_core::modhook::Arg::I32(flags as i32), melee_core::modhook::Arg::F32(arg3), melee_core::modhook::Arg::F32(frames_to_turn), melee_core::modhook::Arg::F32(anim_start)], ftCo_Turn_Enter__orig(g, msid, flags, arg3, frames_to_turn, anim_start));
    #[allow(unused_unsafe)] unsafe { ftCo_Turn_Enter__orig(g, msid, flags, arg3, frames_to_turn, anim_start) }
}

fn ftCo_Turn_Enter__orig(g: G, msid: i32, flags: u32, arg3: f32, frames_to_turn: f32, anim_start: f32) {
    let f = fp(g);
    let neg = -f.facing_dir;
    let t = unsafe { &mut f.mv.co.turn };
    t.has_turned = 0;
    t.just_turned = 0;
    t.facing_after = neg;
    t.frames_to_turn = frames_to_turn;
    t.x8 = arg3;
    t.x1C = 0;
    ext::Fighter_ChangeMotionState(g, msid, flags, anim_start, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
}

pub fn ftCo_Turn_Enter_Basic(g: G) {
    melee_core::ct!("ftCo_Turn_Enter_Basic");
    let frames = fp(g).co_attrs.standing_turn_frames;
    ftCo_Turn_Enter(g, ftCo_MS_Turn as i32, Ft_MF_None, 0.0, frames, 0.0);
}

pub fn ftCo_Turn_Anim_Inner(g: G) {
    melee_core::ct!("ftCo_Turn_Anim_Inner");
    let f = fp(g);
    let t = unsafe { &mut f.mv.co.turn };
    if t.frames_to_turn > 0.0 {
        t.frames_to_turn -= 1.0;
        return;
    }
    if t.has_turned == 0 {
        t.has_turned = 1;
        t.just_turned = 1;
        f.facing_dir = -f.facing_dir;
    }
}

pub fn ftCo_Turn_Anim(g: G) {
    melee_core::ct!("ftCo_Turn_Anim");
    ftCo_Turn_Anim_Inner(g);
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

/// TARGET_PC `ftCo_Turn_Ucf073Dashback` (UCF 0.73 dashback for MELEE_SLP playback).
pub fn ftCo_Turn_Ucf073Dashback(f: &mut Fighter) {
    melee_core::ct!("ftCo_Turn_Ucf073Dashback");
    let port = f.player_idx as i32;
    if ext::Replay_UcfVersion(f.pad_port as i32) != 73 {
        return;
    }
    if (f.x3E4_fighterCmdScript.frame_count.to_bits() >> 16) != 0x4000 {
        return;
    }
    let fa = unsafe { f.mv.co.turn.facing_after };
    if !(f.input.lstick[0].x * fa >= cd().dash_smash_stick_threshold) {
        return;
    }
    if f.active_timer.lstick.x >= 2 || f.is_sub_fighter() != 0 {
        return;
    }
    let d = ext::Replay_RawStickBack(port, 0, 0).wrapping_sub(ext::Replay_RawStickBack(port, 0, 2));
    if d.wrapping_mul(d) > 0x15F9 {
        let t = unsafe { &mut f.mv.co.turn };
        t.just_turned = 1;
        t.has_turned = 1;
        // UCF 0.8 `UCF DB.asm` CHANGE_TO_SMASH_TURN: Popo only (kind 0xA). The code runs after the turn's store of the
        // new facing; this port runs before the negation below it, so the new facing is -facing_dir.
        if f.kind as i32 == 0xA {
            ucf_dashback_nana_entry(f, -f.facing_dir);
        }
    }
}

/// UCF dashback (0.8 and 0.84): after granting the smash turn, overwrite the entry of the sub fighter's (Nana's)
/// input queue that her CPU last stored from this fighter (`cpu.x444`, fp+0x1ECC) with the new facing and a full
/// stick toward it (0x7F right, 0x80 left), so Nana dashes back too.
fn ucf_dashback_nana_entry(f: &Fighter, facing: f32) {
    let g = ext::Player_GetEntityAtIndex(f.player_idx as i32, 1);
    if g.is_null() {
        return;
    }
    let sub = fp(g);
    let e = sub.cpu.x444;
    if e.is_null() {
        return;
    }
    unsafe {
        (*e).facing_dir = facing;
        (*e).lstick.x = if facing.to_bits() >> 31 != 0 { 0x80u8 as i8 } else { 0x7F };
    }
}

/// TARGET_PC `ftCo_Turn_Ucf084Dashback` (UCF 0.84 dashback).
pub fn ftCo_Turn_Ucf084Dashback(f: &mut Fighter) {
    melee_core::ct!("ftCo_Turn_Ucf084Dashback");
    let port = f.pad_port as i32;
    if ext::Replay_UcfVersion(port) != 84 || f.is_sub_fighter() != 0 {
        return;
    }
    if f.x3E4_fighterCmdScript.frame_count.to_bits() != 0x4000_0000 {
        return;
    }
    if f.facing_dir * f.input.lstick[0].x < cd().dash_smash_stick_threshold {
        return;
    }
    if f.active_timer.lstick.x > 1 {
        return;
    }
    let d = ext::ftUcf_PadRaw(port, 0, 0).wrapping_sub(ext::ftUcf_PadRaw(port, 0, 2));
    if d.wrapping_mul(d) > 5625 {
        let t = unsafe { &mut f.mv.co.turn };
        t.has_turned = 1;
        t.just_turned = 1;
        // UCF 0.84 `UCF Dashback.asm` (C2 800C9A44): any fighter with a sub fighter, facing already negated
        ucf_dashback_nana_entry(f, f.facing_dir);
    }
}

pub fn ftCo_Turn_IASA(g: G) {
    melee_core::ct!("ftCo_Turn_IASA");
    {
        let f = fp(g);
        let (just, has, x1c) = unsafe { (f.mv.co.turn.just_turned, f.mv.co.turn.has_turned, f.mv.co.turn.x1C) };
        if just != 0 {
            f.input.pressed_buttons |= x1c;
        }
        if has == 0 {
            ftCo_Turn_Ucf073Dashback(f);
            f.facing_dir = -f.facing_dir;
            ftCo_Turn_Ucf084Dashback(f);
        }
    }
    ret_if!(g; ftCo_SpecialS_CheckInput, ftCo_800D68C0, ftCo_Attack100_CheckInput, ftCo_Catch_CheckInput,
        ftCo_AttackS4_CheckInput, ftCo_AttackHi4_CheckInput, ftCo_AttackLw4_CheckInput, ftCo_AttackS3_CheckInput,
        ftCo_AttackHi3_CheckInput, ftCo_AttackLw3_CheckInput, ftCo_Attack1_CheckInput);
    {
        let f = fp(g);
        if unsafe { f.mv.co.turn.has_turned } == 0 {
            f.facing_dir = -f.facing_dir;
        }
    }
    ret_if!(g; ftCo_80091A4C, ftCo_800DE9D8, ftCo_Jump_CheckInput);
    fn_800C9C2C(g);
    let (just, x8) = unsafe { (fp(g).mv.co.turn.just_turned, fp(g).mv.co.turn.x8) };
    if just != 0 && x8 != 0.0 && ext::ScriptGame_CapsActionAllowed(fp(g), 4) != 0 {
        let f = fp(g);
        let fa = unsafe { f.mv.co.turn.facing_after };
        if f.input.lstick[0].x * fa >= cd().dash_smash_stick_threshold {
            ext::ftCo_Dash_Enter(g, 0);
        }
    }
    let f = fp(g);
    let pb = f.input.pressed_buttons;
    let t = unsafe { &mut f.mv.co.turn };
    if pb & HSD_PAD_A as u32 != 0 {
        t.x1C |= HSD_PAD_A as u32;
    }
    if pb & HSD_PAD_B as u32 != 0 {
        t.x1C |= HSD_PAD_B as u32;
    }
    if t.just_turned != 0 {
        t.just_turned = 0;
    }
}

pub fn ftCo_Turn_Phys(g: G) {
    melee_core::ct!("ftCo_Turn_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_Turn_Coll(g: G) {
    melee_core::ct!("ftCo_Turn_Coll");
    ext::ft_80083F88(g);
}

pub fn fn_800C9C2C(g: G) -> bool {
    melee_core::ct!("fn_800C9C2C");
    let f = fp(g);
    let fa = unsafe { f.mv.co.turn.facing_after };
    if f.input.lstick[0].x * fa >= cd().dash_smash_stick_threshold && (f.active_timer.lstick.x as i32) < cd().dash_smash_window {
        unsafe { f.mv.co.turn.x8 = fa };
        return true;
    }
    false
}

pub fn ftCo_Turn_Enter_Smash(g: G) {
    melee_core::ct!("ftCo_Turn_Enter_Smash");
    let f = fp(g);
    let facing = f.facing_dir;
    let neg = -f.facing_dir;
    let t = unsafe { &mut f.mv.co.turn };
    t.has_turned = 0;
    t.just_turned = 0;
    t.facing_after = neg;
    t.frames_to_turn = 0.0;
    t.x8 = facing;
    t.x1C = 0;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Turn as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
}
