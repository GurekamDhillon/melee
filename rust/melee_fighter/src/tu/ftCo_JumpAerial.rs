#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_JumpAerial.c (TARGET_PC build): air jump,
//! including the Ness / Yoshi / Peach / Mewtwo variants and the static inlines. Every function
//! of the TU; calls out go through `ext`. Checked against the C by tests/tu_ftCo_JumpAerial.rs.

use crate::types::*;
use melee_types::inline as ji;

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
        FighterControls_TapJumpOff(port: i32) -> i32 = |p| crate::tu::ftCo_Jump::ext::FighterControls_TapJumpOff(p);
        ScriptGame_ModJump(slot: i32, field: i32, base: f32) -> f32 = |s, f, b| crate::tu::ftCo_Jump::ext::ScriptGame_ModJump(s, f, b);
        ScriptGame_ModValue(slot: i32, field: i32, base: f32) -> f32 = crate::hooks::script_mod_value;
        // TARGET_PC `FT_CAPS_MAX_JUMPS(fp)` (pc/gameworld/script_fighter_caps.h).
        ScriptGame_CapsMaxJumps(fp: &Fighter) -> i32 = |_| todo!("TODO(port) ScriptGame_CapsMaxJumps (pc/gameworld/script_game.c)");
        Mex_OnDoubleJumpDispatch(kind: i32, g: G, vanilla: *mut core::ffi::c_void) = |kind: i32, g: G, vanilla: *mut core::ffi::c_void| { unsafe { let _ = crate::platform::mex_dispatch(crate::platform::MEX_EVENT_ON_DOUBLE_JUMP, kind, g, vanilla); } };
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |_| todo!("TODO(group1 jobj) HSD_JObjSetMtxDirtySub");
        ftCo_800C5240(g: G) -> bool = |g| crate::tu::ftCo_Jump::ext::ftCo_800C5240(g);
        ftCo_800D730C(g: G, arg1: bool) -> bool = crate::tu::ftCo_JumpAerialF1::ftCo_800D730C;
        ft_800D2D0C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ft_0d27::ft_800d2d0c(g as _) as _ } };
        ft_800D2E7C(g: G, v: &mut Vec3) = crate::tu::ftCo_ItemScrew::ft_800D2E7C;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ft_80088770(fp: &mut Fighter) = crate::ft0881::ft_80088770;
        ft_80088328(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |fp: &mut Fighter, id: i32, vol: u8, pan: u8| { unsafe { let _ = crate::ft0881::ft_80088328(fp as _, id as _, vol as _, pan as _); } };
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |fp: &mut Fighter, id: i32, vol: u8, pan: u8| { unsafe { let _ = crate::ft0877::play_sfx(fp, id as _, vol as _, pan as _); } };
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_FallAerial_Enter(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_FallAerial::ftCo_FallAerial_Enter(g as _); } };
        ftCo_SpecialAir_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialAir::ftCo_SpecialAir_CheckInput(g as _) as _ } };
        ftCo_80095328(g: G, out: *mut bool) -> bool = crate::tu::ftCo_ItemThrow::ftCo_80095328;
        ftCo_800D7100(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Attack100::ftCo_800D7100(g as _) as _ } };
        ftCo_800C3B10(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftco_aircatch::ftco_800c3b10(g as _) as _ } };
        ftCo_80099A58(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_EscapeAir::ftCo_80099A58(g as _) as _ } };
        ftCo_AttackAir_CheckItemThrowInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_AttackAir::ftCo_AttackAir_CheckItemThrowInput(g as _) as _ } };
        ftCo_800D705C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Attack100::ftCo_800D705C(g as _) as _ } };
        ftPe_8011BA54(g: G) -> bool = |_| todo!("TODO(port) ftPe_8011BA54 (ftpeachfloat.c)");
        ftPe_8011BAD8(g: G) -> bool = |_| todo!("TODO(port) ftPe_8011BAD8 (ftpeachfloat.c)");
        ftCo_800CEE70(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_ItemParasolOpen::ftCo_800CEE70(g as _) as _ } };
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ftCommon_CalcSelfAccel_DriftFrom(fp: &mut Fighter, v: f32) = |fp: &mut Fighter, v: f32| { unsafe { let _ = crate::ftcommon::calc_self_accel_drift_from(fp, v as _); } };
        ftCommon_CalcSelfAccel_Drift(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { let _ = crate::ftcommon::calc_self_accel_drift(fp); } };
        ft_800851D0(g: G) = crate::ft084e::ft_800851d0;
        ft_800835B0(g: G, a: fn(G, i32) -> bool, b: fn(G)) = |g: G, a: fn(G, i32) -> bool, b: fn(G)| { unsafe { let _ = crate::tu::ft_081b::ft_800835b0(g as _, a as _, b as _); } };
        ftCo_80096CC8(g: G, line: i32) -> bool = |g: G, line: i32| -> bool { unsafe { crate::tu::ftCo_FallSpecial::ftCo_80096CC8(g as _, line as _) as _ } };
        ft_80082B1C(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80082b1c(g as _); } };
    }
}

/// jobj.h's `HSD_JObjSetMtxDirty` macro / out-of-line dirty marks.
struct Dirty;
impl ji::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !ji::HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

#[inline(always)]
pub(crate) fn tap_jump_ok(f: &Fighter) -> bool {
    ext::FighterControls_TapJumpOff(f.player_id as i32) == 0
}
#[inline(always)]
pub(crate) fn caps_max_jumps(f: &Fighter) -> i32 {
    ext::ScriptGame_CapsMaxJumps(f)
}

/// Yoshi's flip: count down, rotate the root joint by -180/arg1 degrees, turn round at half.
pub fn ft_800CB6EC(f: &mut Fighter, arg1: i32) {
    unsafe {
        if f.mv.co.jumpaerial.x0 != 0 {
            f.mv.co.jumpaerial.x0 -= 1;
            let j = (*f.parts).joint;
            ji::HSD_JObjAddRotationY(&mut *j, -((180.0f32 / arg1 as f32) * 0.01745329252f32), &mut Dirty);
            if f.mv.co.jumpaerial.x0 == melee_runtime::cabi::sdiv(arg1, 2) {
                f.facing_dir = -f.facing_dir;
            }
        }
    }
}

pub fn ft_did_jump(f: &mut Fighter, arg1: bool) -> bool {
    if (f.x1968_jumpsUsed as i32) < caps_max_jumps(f)
        && ((tap_jump_ok(f)
            && f.input.lstick[0].y >= cd().tap_jump_threshold
            && (f.active_timer.lstick.y as i32) < cd().tap_jump_window)
            || f.input.pressed_buttons & 0xC00 != 0)
        && !(arg1 && (f.x68A as i32) < cd().x1C)
    {
        return true;
    }
    false
}

pub fn ftCo_800CB870(g: G) -> bool {
    if ext::ftCo_800C5240(g) {
        return false;
    }
    if fp(g).can_multijump() != 0 {
        return ext::ftCo_800D730C(g, false);
    }
    ftCo_JumpAerial_CheckInput(g, false)
}

pub fn ftCo_800CB8E0(g: G) -> bool {
    if ext::ftCo_800C5240(g) {
        return false;
    }
    if fp(g).can_multijump() != 0 {
        return ext::ftCo_800D730C(g, true);
    }
    ftCo_JumpAerial_CheckInput(g, true)
}

pub fn ftCo_JumpAerial_CheckInput(g: G, arg1: bool) -> bool {
    if ft_did_jump(fp(g), arg1) {
        if ext::ft_800D2D0C(g) {
            let f = fp(g);
            let x = f.input.lstick[0].x * f.co_attrs.air_jump_h_multiplier;
            let y = f.co_attrs.jump_v_initial_velocity
                * ext::ScriptGame_ModJump(f.player_id as i32, 6, f.co_attrs.air_jump_v_multiplier);
            let mut v = Vec3 { x, y, z: 0.0 };
            ext::ft_800D2E7C(g, &mut v);
            return true;
        }
        match fp(g).kind {
            Ft_Kind_Ness => ftNs_JumpAerial_Enter(g),
            Ft_Kind_Yoshi => ftYs_JumpAerial_Enter(g),
            Ft_Kind_Peach => ftPe_JumpAerial_Enter(g),
            Ft_Kind_Mewtwo => ftMt_JumpAerial_Enter(g),
            _ => ftCo_JumpAerial_Enter_Basic(g),
        }
        return true;
    }
    false
}

/// static inline `ft_SetVec`.
#[inline(always)]
pub fn ft_SetVec(dst: &mut Vec3, src: &Vec3) {
    dst.x = src.x;
    dst.y = src.y;
    dst.z = src.z;
}

/// static inline `ft_JumpAerial_Sound`.
#[inline(always)]
pub fn ft_JumpAerial_Sound(f: &mut Fighter, kind: FighterKind) {
    if kind != Ft_Kind_Mewtwo && (kind >= 0x10 || kind != Ft_Kind_Ness) && !f.x197C.is_null() {
        ext::ft_PlaySFX(f, 0x11B, 127, 64);
    }
}

/// static inline `ft_JumpAerial_MarkAir` (TARGET_PC): keeps a script's extra jumps used.
#[inline(always)]
pub fn ft_JumpAerial_MarkAir(f: &mut Fighter) {
    let used = f.x1968_jumpsUsed;
    ext::ftCommon_8007D5D4(f);
    if used > 1 {
        f.x1968_jumpsUsed = used;
    }
}

pub fn ftCo_800CBAC4(g: G, msid: i32, vel: &Vec3, arg3: bool) {
    ext::Mex_OnDoubleJumpDispatch(fp(g).kind as i32, g, core::ptr::null_mut());
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_SkipNametagVis, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_x2221_b7(1);
    ft_SetVec(&mut f.self_vel, vel);
    if arg3 {
        f.active_timer.lstick.y = 0xFE;
    }
    f.x1968_jumpsUsed = f.x1968_jumpsUsed.wrapping_add(1);
    ext::ft_80088770(f);
    let f = fp(g);
    let id = unsafe { (*(*f.ft_data).x4C_sfx).x14 };
    ext::ft_80088328(f, id, 127, 64);
    let f = fp(g);
    let k = f.kind;
    ft_JumpAerial_Sound(f, k);
}

#[inline(always)]
fn fwd_msid(f: &Fighter) -> i32 {
    (if f.input.lstick[0].x * f.facing_dir > -cd().x78 { ftCo_MS_JumpAerialF } else { ftCo_MS_JumpAerialB }) as i32
}

pub fn ftCo_JumpAerial_Enter_Basic(g: G) {
    ft_JumpAerial_MarkAir(fp(g));
    let f = fp(g);
    f.cmd_vars[0] = 1;
    let msid = fwd_msid(f);
    let x = f.input.lstick[0].x * f.co_attrs.air_jump_h_multiplier;
    let y = f.co_attrs.jump_v_initial_velocity
        * ext::ScriptGame_ModJump(f.player_id as i32, 6, f.co_attrs.air_jump_v_multiplier);
    let v = Vec3 { x, y, z: 0.0 };
    ftCo_800CBAC4(g, msid, &v, true);
}

pub fn ftNs_JumpAerial_Enter(g: G) {
    let v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ft_JumpAerial_MarkAir(fp(g));
    let f = fp(g);
    f.cmd_vars[0] = 1;
    let msid = fwd_msid(f);
    ftCo_800CBAC4(g, msid, &v, true);
    let f = fp(g);
    f.phys_cb = Some(ftNs_JumpAerial_Phys_Cb_c);
    unsafe {
        f.mv.co.jumpaerial.init_h_vel = f.input.lstick[0].x * f.co_attrs.air_jump_h_multiplier;
    }
}

pub fn ftYs_JumpAerial_Enter(g: G) {
    let ys = fp(g).dat_attrs as *const ftYoshiAttributes;
    ft_JumpAerial_MarkAir(fp(g));
    let f = fp(g);
    f.cmd_vars[0] = 1;
    let v = Vec3 { x: f.input.lstick[0].x * f.co_attrs.air_jump_h_multiplier, y: 0.0, z: 0.0 };
    ftCo_800CBAC4(g, ftCo_MS_JumpAerialF as i32, &v, true);
    let f = fp(g);
    f.phys_cb = Some(ftCo_JumpAerial_Phys_Cb_c);
    f.anim_cb = Some(ftYs_JumpAerial_Anim_Cb_c);
    unsafe {
        f.dmg.armor1 = (*ys).x8;
        if f.input.lstick[0].x * f.facing_dir < -(*ys).x4 {
            f.mv.co.jumpaerial.x0 = (*ys).x0;
        } else {
            f.mv.co.jumpaerial.x0 = 0;
        }
        let a = (*ys).x0;
        ft_800CB6EC(f, a);
    }
}

fn pe_mt_enter(g: G) {
    ft_JumpAerial_MarkAir(fp(g));
    let f = fp(g);
    f.cmd_vars[0] = 1;
    let msid = fwd_msid(f);
    let v = Vec3 { x: f.input.lstick[0].x * f.co_attrs.air_jump_h_multiplier, y: 0.0, z: 0.0 };
    ftCo_800CBAC4(g, msid, &v, true);
    fp(g).phys_cb = Some(ftCo_JumpAerial_Phys_Cb_c);
}

pub fn ftPe_JumpAerial_Enter(g: G) {
    pe_mt_enter(g);
}

pub fn ftMt_JumpAerial_Enter(g: G) {
    pe_mt_enter(g);
}

pub fn ftCo_JumpAerial_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_FallAerial_Enter(g);
    }
}

pub fn ftYs_JumpAerial_Anim_Cb(g: G) {
    let ys = fp(g).dat_attrs as *const ftYoshiAttributes;
    ftCo_JumpAerial_Anim(g);
    let a = unsafe { (*ys).x0 };
    ft_800CB6EC(fp(g), a);
}
pub unsafe extern "C" fn ftYs_JumpAerial_Anim_Cb_c(g: G) {
    ftYs_JumpAerial_Anim_Cb(g)
}

pub fn ftCo_JumpAerial_IASA(g: G) {
    if ext::ftCo_SpecialAir_CheckInput(g) { return; }
    if ext::ftCo_80095328(g, core::ptr::null_mut()) { return; }
    if ext::ftCo_800D7100(g) { return; }
    if ext::ftCo_800C3B10(g) { return; }
    if ext::ftCo_80099A58(g) { return; }
    if ext::ftCo_AttackAir_CheckItemThrowInput(g) { return; }
    if ext::ftCo_800D705C(g) { return; }
    if ext::ftPe_8011BA54(g) { return; }
    if ftCo_800CB870(g) { return; }
    if fp(g).cmd_vars[0] != 0 && ext::ftPe_8011BAD8(g) { return; }
    if fp(g).cmd_vars[0] == 0 { return; }
    if ext::ftCo_800CEE70(g) { return; }
}

pub fn ftCo_JumpAerial_Phys(g: G) {
    ext::ft_80084DB0(g);
}

pub fn ftNs_JumpAerial_Phys_Cb(g: G) {
    let f = fp(g);
    let v = unsafe { f.mv.co.jumpaerial.init_h_vel };
    ext::ftCommon_CalcSelfAccel_DriftFrom(f, v);
    let f = fp(g);
    unsafe {
        f.mv.co.jumpaerial.init_h_vel += f.x74_self_accel.x;
        f.x74_self_accel.x = 0.0;
        // `transN.z * facing + init_h_vel` is one fmuladd -> fmadds.
        f.self_vel.x = melee_core::fp::fmadds(f.x6A4_transNOffset.z, f.facing_dir, f.mv.co.jumpaerial.init_h_vel);
    }
    ext::ft_800851D0(g);
    let f = fp(g);
    f.self_vel.y = ext::ScriptGame_ModValue(f.player_id as i32, 6, f.self_vel.y);
}
pub unsafe extern "C" fn ftNs_JumpAerial_Phys_Cb_c(g: G) {
    ftNs_JumpAerial_Phys_Cb(g)
}

pub fn ftCo_JumpAerial_Phys_Cb(g: G) {
    ext::ftCommon_CalcSelfAccel_Drift(fp(g));
    ext::ft_800851D0(g);
    let f = fp(g);
    f.self_vel.y = ext::ScriptGame_ModValue(f.player_id as i32, 6, f.self_vel.y);
}
pub unsafe extern "C" fn ftCo_JumpAerial_Phys_Cb_c(g: G) {
    ftCo_JumpAerial_Phys_Cb(g)
}

pub fn ftCo_JumpAerial_Coll(g: G) {
    ext::ft_800835B0(g, ext::ftCo_80096CC8, ext::ft_80082B1C);
}
