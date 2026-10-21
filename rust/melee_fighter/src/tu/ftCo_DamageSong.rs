#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_DamageSong.c (Jigglypuff Sing sleep).
//! Checked against the C by tests/tu_ftCo_DamageSong.rs.

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
        ftCommon_8007DB58(g: G) = |_| todo!("TODO(port) ftCommon_8007DB58 (ftcommon.c)");
        ftCo_8009750C(g: G) = |_| todo!("TODO(port) ftCo_8009750C (ftCo_CaptureCut.c)");
        ftCo_800DD168(g: G) = |_| todo!("TODO(port) ftCo_800DD168 (ftCo_Catch.c)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        Player_GetHandicap(slot: i32) -> i32 = |_| todo!("TODO(port) Player_GetHandicap (player.c)");
        Player_80033BB8(slot: i32) -> i32 = crate::player::player_80033bb8;
        ftCommon_InitGrab(fp: &mut Fighter, a: i32, timer: f32) = crate::ftcommon2::init_grab;
        ftCommon_GrabMash(fp: &mut Fighter, v: f32) -> bool = crate::ftcommon2::grab_mash;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80084280(g: G) = crate::tu::ft_081b::ft_80084280;
        ftCo_800C3598(g: G) = |_| todo!("TODO(port) ftCo_800C3598 (ftCo_DamageBind.c)");
    }
}

/// static inline `ftCo_ReleaseItemAndVictim` (ftCommon/inlines.h).
fn ftCo_ReleaseItemAndVictim(g: G) {
    ext::ftCommon_8007DB58(g);
    ext::ftCo_8009750C(g);
    ext::ftCo_800DD168(g);
}

/// static `inlineA0`: the sleep timer. Three fmuladd sites (clang contraction), evaluated
/// innermost first; Player_GetHandicap is called before Player_80033BB8.
pub fn inlineA0(f: &Fighter, arg1: bool) -> f32 {
    let c = cd();
    let hc = ext::Player_GetHandicap(f.player_id as i32);
    let t0 = fmadds(c.x628, c.x62C - hc as f32, c.x624);
    let bb = ext::Player_80033BB8(f.player_id as i32);
    let t1 = fmadds(c.x630, c.x634 - (bb.wrapping_add(1)) as f32, t0);
    let mut result = fmadds(f.dmg.x1830_percent, c.x638, t1);
    if arg1 {
        result *= c.x644;
    }
    result
}

pub fn ftCo_800C318C(g: G, arg1: bool) {
    ftCo_ReleaseItemAndVictim(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_DamageSong as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let t = inlineA0(fp(g), arg1);
    ext::ftCommon_InitGrab(fp(g), 0, t);
}

/// static inline `inlineB0`: mash out; true = woke up (entered DamageSongRv).
fn inlineB0(g: G) -> bool {
    let f = fp(g);
    f.grab_timer -= cd().x63C;
    ext::ftCommon_GrabMash(fp(g), cd().x640);
    if fp(g).grab_timer <= 0.0 {
        ftCo_800C3480(g);
        true
    } else {
        false
    }
}

pub fn ftCo_DamageSong_Anim(g: G) {
    if !inlineB0(g) && !ext::ftAnim_IsFramesRemaining(g) {
        ftCo_800C3390(g);
    }
}

pub fn ftCo_DamageSong_IASA(_g: G) {}

pub fn ftCo_DamageSong_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_DamageSong_Coll(g: G) {
    ext::ft_80084280(g);
}

/// static `ftCo_800C3390`.
pub fn ftCo_800C3390(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_DamageSongWait as i32, (Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim) as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

pub fn ftCo_DamageSongWait_Anim(g: G) {
    inlineB0(g);
}

pub fn ftCo_DamageSongWait_IASA(_g: G) {}

pub fn ftCo_DamageSongWait_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_DamageSongWait_Coll(g: G) {
    ext::ft_80084280(g);
}

/// static `ftCo_800C3480`.
pub fn ftCo_800C3480(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_DamageSongRv as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

pub fn ftCo_DamageSongRv_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_DamageSongRv_IASA(_g: G) {}

pub fn ftCo_DamageSongRv_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_DamageSongRv_Coll(g: G) {
    ext::ft_80084280(g);
}

pub fn ftCo_800C3538(g: G) -> bool {
    let f = fp(g);
    if f.x2222_b2() != 0 {
        return false;
    }
    match unsafe { f.x2070.anon_0.x2071_b0_3() } {
        9 | 10 | 11 => false,
        _ => {
            ext::ftCo_800C3598(g);
            true
        }
    }
}
