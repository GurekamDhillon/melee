#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Escape.c (rolls and spotdodge), including the
//! TARGET_PC UCF 0.73 / 0.84 shield-drop checks (the game's `bool` is `int`, kept as i32). Every function of the TU; calls out of it go
//! through `ext`. Fused sites (PC build, shipping contraction): ftCo_Ucf073RimAxis
//! `|v| * 80 - 9.95e-6` -> `fmsubs`; ftCo_Ucf073BlocksSpotdodge `y*y + x*x` -> `fmadds(y, y, x*x)`;
//! ftCo_Ucf084RimUnits double `|v| * 80.0 - c` -> f64 fma.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_Escape.rs.
use melee_core::fp::Fmadd as _;
use crate::types::*;
use melee_core::fp::{fmadds, fmsubs};

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    #[allow(unused_imports)]
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
        ftCo_800DF8B0(fp: &Fighter) -> bool = crate::ft0df::ftco_800df8b0;
        ftCo_800DF8E8(fp: &Fighter) -> bool = crate::ft0df::ftco_800df8e8;
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftParts_80074B0C(g: G, model_idx: i32, val: i32) = |g: G, model_idx: i32, val: i32| { unsafe { crate::tu::ftparts::ftParts_80074B0C(g as _, model_idx as _, val as _); } };
        ftYs_Init_8012BDA0(g: G) = |_| todo!("TODO(port) ftYs_Init_8012BDA0 (ftyoshi.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::tu::ft_08a1::ft_8008A2BC;
        ftSs_SpecialLw_8012AEBC(g: G) = |_| todo!("TODO(port) ftSs_SpecialLw_8012AEBC (ftsamusspeciallw1.c)");
        ftSs_SpecialLw_8012AF38(g: G) = |_| todo!("TODO(port) ftSs_SpecialLw_8012AF38 (ftsamusspeciallw1.c)");
        ftYs_Shield_8012CC1C(g: G) -> bool = |_| todo!("TODO(port) ftYs_Shield_8012CC1C (ftyoshiguard.c)");
        ftYs_Init_8012BE3C(g: G) = |_| todo!("TODO(port) ftYs_Init_8012BE3C (ftyoshi.c)");
        ftYs_Init_8012B8A4(g: G) = |_| todo!("TODO(port) ftYs_Init_8012B8A4 (ftyoshi.c)");
        ftCo_8009563C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_ItemThrow::ftCo_8009563C(g as _) as _ } };
        ft_80085004(g: G) = crate::ft084e::ft_80085004;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80084104(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80084104(g as _); } };
        ft_800847D0(g: G, b: *mut ftCollisionBox) = |g: G, b: *mut ftCollisionBox| { unsafe { crate::tu::ft_081b::ft_800847d0(g as _, b as _); } };
        Replay_UcfVersion(port: i32) -> i32 = |port: i32| -> i32 { unsafe { crate::platform::Replay_UcfVersion(port) } };
    }
}

/// static inline `inlineA1`.
fn inlineA1(f: &Fighter) -> bool {
    abs_c(f.input.lstick[0].x) >= cd().x31C && (f.active_timer.lstick.x as i32) < cd().x320
}

pub fn ftCo_8009917C(g: G) -> bool {
    melee_core::ct!("ftCo_8009917C");
    let f = fp(g);
    let stick_x = if inlineA1(f) {
        f.input.lstick[0].x
    } else if ext::ftCo_800DF8B0(f) {
        f.input.cstick[0].x
    } else {
        return false;
    };
    let msid = if stick_x * f.facing_dir >= 0.0 { ftCo_MS_EscapeF } else { ftCo_MS_EscapeB };
    ftCo_800992A8(g, msid, cd().x324);
    true
}

pub fn ftCo_80099264(g: G) -> bool {
    melee_core::ct!("ftCo_80099264");
    if fp(g).input.held_buttons[0] & HSD_PAD_LR != 0 {
        ftCo_800992A8(g, ftCo_MS_EscapeF, 0);
        return true;
    }
    false
}

/// static `ftCo_800992A8`.
pub fn ftCo_800992A8(g: G, msid: i32, arg2: i32) {
    melee_core::ct!("ftCo_800992A8");
    match fp(g).kind {
        Ft_Kind_Samus => ftCo_80099390(g, msid, arg2),
        Ft_Kind_Yoshi => ftCo_80099438(g, msid, arg2),
        _ => ftCo_80099314(g, msid, arg2),
    }
    ext::ftCommon_8007EBAC(fp(g), 23, 0);
}

/// static `ftCo_80099314`.
pub fn ftCo_80099314(g: G, msid: i32, arg2: i32) {
    melee_core::ct!("ftCo_80099314");
    fp(g).anon_2210.throw_flags = 0;
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    f.set_x221D_b5(1);
    unsafe { f.mv.co.escape.x0 = arg2 };
}

/// static `ftCo_80099390` (Samus: the bomb-jump roll).
pub fn ftCo_80099390(g: G, msid: i32, arg2: i32) {
    melee_core::ct!("ftCo_80099390");
    let f = fp(g);
    f.cmd_vars[0] = 0;
    unsafe { f.mv.co.escape.x4 = 0 };
    ftCo_80099314(g, msid, arg2);
    let f = fp(g);
    f.anim_cb = Some(ftCo_80099564);
    f.coll_cb = Some(ftCo_80099754);
}

/// static `ftCo_80099438` (Yoshi).
pub fn ftCo_80099438(g: G, msid: i32, arg2: i32) {
    melee_core::ct!("ftCo_80099438");
    ftCo_80099314(g, msid, arg2);
    ext::ftParts_80074B0C(g, 0, 1);
    ext::ftYs_Init_8012BDA0(g);
    fp(g).anim_cb = Some(ftCo_80099644);
}

/// inlines.h `ftCheckThrowB3`.
fn ftCheckThrowB3(f: &mut Fighter) -> bool {
    unsafe {
        if f.anon_2210.anon_0.throw_flags_b3() != 0 {
            f.anon_2210.anon_0.set_throw_flags_b3(0);
            true
        } else {
            false
        }
    }
}

pub fn ftCo_Escape_Anim(g: G) {
    melee_core::ct!("ftCo_Escape_Anim");
    let f = fp(g);
    if ftCheckThrowB3(f) {
        f.facing_dir = -f.facing_dir;
    }
    if !ext::ftAnim_IsFramesRemaining(g) {
        fp(g).gr_vel = 0.0;
        ext::ft_8008A2BC(g);
    }
}

/// static `ftCo_80099564`; an `anim_cb`, hence `extern "C"`.
pub extern "C" fn ftCo_80099564(g: G) {
    melee_core::ct!("ftCo_80099564");
    let f = fp(g);
    if f.cmd_vars[0] != 0 && unsafe { f.mv.co.escape.x4 } == 0 {
        ext::ftSs_SpecialLw_8012AEBC(g);
        unsafe { fp(g).mv.co.escape.x4 = 1 };
    }
    let f = fp(g);
    if f.cmd_vars[0] == 0 && unsafe { f.mv.co.escape.x4 } != 0 {
        ext::ftSs_SpecialLw_8012AF38(g);
        unsafe { fp(g).mv.co.escape.x4 = 0 };
    }
    ftCo_Escape_Anim(g);
}

/// static `ftCo_80099644`; an `anim_cb`, hence `extern "C"`.
pub extern "C" fn ftCo_80099644(g: G) {
    melee_core::ct!("ftCo_80099644");
    let f = fp(g);
    if ftCheckThrowB3(f) {
        f.facing_dir = -f.facing_dir;
    }
    if !ext::ftAnim_IsFramesRemaining(g) {
        fp(g).gr_vel = 0.0;
        if ext::ftYs_Shield_8012CC1C(g) {
            return;
        }
        ext::ftYs_Init_8012BE3C(g);
        ext::ft_8008A2BC(g);
    }
    ext::ftYs_Init_8012B8A4(g);
}

pub fn ftCo_Escape_IASA(g: G) {
    melee_core::ct!("ftCo_Escape_IASA");
    ext::ftCo_8009563C(g);
}

pub fn ftCo_Escape_Phys(g: G) {
    melee_core::ct!("ftCo_Escape_Phys");
    ext::ft_80085004(g);
}

pub fn ftCo_Escape_Coll(g: G) {
    melee_core::ct!("ftCo_Escape_Coll");
    ext::ft_80084104(g);
}

/// static `ftCo_80099754`; a `coll_cb`, hence `extern "C"`.
pub extern "C" fn ftCo_80099754(g: G) {
    melee_core::ct!("ftCo_80099754");
    let f = fp(g);
    if f.cmd_vars[0] != 0 {
        let da = f.dat_attrs as *mut ftSs_DatAttrs;
        let b = unsafe { core::ptr::addr_of_mut!((*da).height_attributes) };
        ext::ft_800847D0(g, b);
    } else {
        ext::ft_80084104(g);
    }
}

/// static inline `inlineB0`.
fn inlineB0(f: &Fighter) -> bool {
    f.input.lstick[0].y <= cd().x314 && (f.active_timer.lstick.y as i32) < cd().x318
}

/// TARGET_PC static `ftCo_Ucf073RimAxis`.
pub fn ftCo_Ucf073RimAxis(v: f32) -> f32 {
    melee_core::ct!("ftCo_Ucf073RimAxis");
    let t = fmsubs(if v < 0.0 { -v } else { v }, 80.0, 9.953975677490234e-06);
    ((melee_core::ppc::fctiwz_f32(t)).wrapping_add(2)) as f32 / 80.0
}

/// TARGET_PC static `ftCo_Ucf073BlocksSpotdodge`.
pub fn ftCo_Ucf073BlocksSpotdodge(f: &Fighter) -> bool {
    melee_core::ct!("ftCo_Ucf073BlocksSpotdodge");
    if ext::Replay_UcfVersion(f.x618_player_id as i32) != 73 {
        return false;
    }
    let c = cd();
    if !(f.input.cstick[0].y > c.x314) {
        return false;
    }
    let x = ftCo_Ucf073RimAxis(f.input.lstick[0].x);
    let y = ftCo_Ucf073RimAxis(f.input.lstick[0].y);
    if !(fmadds(y, y, x * x) >= 1.0) {
        return false;
    }
    if f.active_timer.lstick.x <= 3 {
        return false;
    }
    if -c.walk_fast_stick_threshold >= f.input.lstick[0].y {
        return false;
    }
    true
}

/// TARGET_PC static `ftCo_Ucf084RimUnits`.
pub fn ftCo_Ucf084RimUnits(v: f32) -> i32 {
    melee_core::ct!("ftCo_Ucf084RimUnits");
    let a = if v < 0.0 { -v } else { v };
    let t = (a as f64).fmadd(80.0, -(9.99999974738e-05f32 as f64)) as f32;
    (melee_core::ppc::fctiwz_f32(t)).wrapping_add(2)
}

/// TARGET_PC static `ftCo_Ucf084BlocksSpotdodge`.
pub fn ftCo_Ucf084BlocksSpotdodge(f: &Fighter) -> bool {
    melee_core::ct!("ftCo_Ucf084BlocksSpotdodge");
    if ext::Replay_UcfVersion(f.x618_player_id as i32) != 84 {
        return false;
    }
    let c = cd();
    if !(f.input.cstick[0].y > c.x314) {
        return false;
    }
    if (f.active_timer.lstick.x as i32) < c.x320 {
        return false;
    }
    if !(f.input.lstick[0].y > -0.8) {
        return false;
    }
    if f.coll_data.floor.index == -1 || f.coll_data.floor.flags & 0x100 == 0 {
        return false;
    }
    let ry = ftCo_Ucf084RimUnits(f.input.lstick[0].y);
    let rx = ftCo_Ucf084RimUnits(f.input.lstick[0].x);
    ry.wrapping_mul(ry).wrapping_add(rx.wrapping_mul(rx)) > 6400
}

pub fn ftCo_80099794(g: G) -> bool {
    melee_core::ct!("ftCo_80099794");
    let f = fp(g);
    if f.input.held_buttons[0] & HSD_PAD_LR != 0 && inlineB0(f) {
        if ftCo_Ucf073BlocksSpotdodge(f) || ftCo_Ucf084BlocksSpotdodge(f) {
            return false;
        }
        ftCo_80099894(g);
        return true;
    }
    false
}

pub fn ftCo_8009980C(g: G) -> bool {
    melee_core::ct!("ftCo_8009980C");
    let f = fp(g);
    if inlineB0(f) || ext::ftCo_800DF8E8(f) {
        if ftCo_Ucf073BlocksSpotdodge(f) || ftCo_Ucf084BlocksSpotdodge(f) {
            return false;
        }
        ftCo_80099894(g);
        return true;
    }
    false
}

/// static `ftCo_80099894`.
pub fn ftCo_80099894(g: G) {
    melee_core::ct!("ftCo_80099894");
    match fp(g).kind {
        Ft_Kind_Yoshi => ftCo_80099954(g),
        _ => ftCo_800998EC(g),
    }
    ext::ftCommon_8007EBAC(fp(g), 23, 0);
}

/// static `ftCo_800998EC`.
pub fn ftCo_800998EC(g: G) {
    melee_core::ct!("ftCo_800998EC");
    ext::Fighter_ChangeMotionState(g, ftCo_MS_EscapeN, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    fp(g).set_x221D_b5(1);
}

/// static `ftCo_80099954`.
pub fn ftCo_80099954(g: G) {
    melee_core::ct!("ftCo_80099954");
    if fp(g).x5F4_arr[0].idx == 1 {
        ext::ftYs_Init_8012BE3C(g);
    }
    ftCo_800998EC(g);
}

pub fn ftCo_EscapeN_Anim(g: G) {
    melee_core::ct!("ftCo_EscapeN_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_EscapeN_IASA(_g: G) { melee_core::ct!("ftCo_EscapeN_IASA");}

pub fn ftCo_EscapeN_Phys(g: G) {
    melee_core::ct!("ftCo_EscapeN_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_EscapeN_Coll(g: G) {
    melee_core::ct!("ftCo_EscapeN_Coll");
    ext::ft_80084104(g);
}
