#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_DamageBind.c (bound/stuck state). Every
//! function of the TU, plus inlines.h `ftCo_ReleaseItemAndVictim`. Calls out go through `ext`.
//! Fused sites (shipping contraction): static commonCall's grab timer
//! `pct * released + (x65C * (x660 - hcap) + x658 + x664 * (pressed - (n + 1)))` is three nested
//! `fmadds`. Checked against the C (whole TU) by tests/tu_ftCo_DamageBind.rs.

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
        ftCommon_8007DB58(g: G) = crate::ftcommon3::ftcommon_8007db58;
        ftCo_8009750C(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Lift::ftCo_8009750C(g as _); } };
        ftCo_800DD168(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Throw::ftCo_800DD168(g as _); } };
        ftCo_80090780(g: G) = crate::tu::ftCo_DamageFall::ftCo_80090780;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        Player_GetHandicap(slot: i32) -> i32 = crate::player::get_handicap;
        Player_80033BB8(slot: i32) -> i32 = crate::player::player_80033bb8;
        ftCommon_InitGrab(fp: &mut Fighter, arg1: i32, timer: f32) = |fp: &mut Fighter, arg1: i32, timer: f32| { unsafe { let _ = crate::ftcommon2::init_grab(fp, arg1 as _, timer as _); } };
        ftCommon_GrabMash(fp: &mut Fighter, v: f32) -> bool = |fp: &mut Fighter, v: f32| -> bool { unsafe { crate::ftcommon2::grab_mash(fp, v as _) } };
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80084280(g: G) = crate::tu::ft_081b::ft_80084280;
    }
}

pub fn ftCo_800C44CC(g: G) -> bool {
    melee_core::ct!("ftCo_800C44CC");
    let f = fp(g);
    if f.dmg.x1860_element == HitElement_Disable as u32 && f.is_sandbag() == 0 {
        match unsafe { f.x2070.anon_0.x2071_b0_3() } {
            5 | 9 | 11 | 12 => return false,
            _ => {
                ftCo_800C4550(g);
                return true;
            }
        }
    }
    false
}

/// static `commonCall`.
pub fn commonCall(f: &mut Fighter) {
    melee_core::ct!("commonCall");
    let c = cd();
    let hcap = ext::Player_GetHandicap(f.player_id as i32) as f32;
    let n = (ext::Player_80033BB8(f.player_id as i32) + 1) as f32;
    let inner = fmadds(c.x65C, c.x660 - hcap, c.x658);
    let inner = fmadds(c.x664, c.pressed_inputs - n, inner);
    let timer = fmadds(f.dmg.x1830_percent, c.released_inputs, inner);
    ext::ftCommon_InitGrab(f, 0, timer);
}

pub fn ftCo_800C4550(g: G) {
    melee_core::ct!("ftCo_800C4550");
    // inlines.h `ftCo_ReleaseItemAndVictim`
    ext::ftCommon_8007DB58(g);
    ext::ftCo_8009750C(g);
    ext::ftCo_800DD168(g);
    if fp(g).ground_or_air == GA_Air {
        ext::ftCo_80090780(g);
    } else {
        ext::Fighter_ChangeMotionState(g, ftCo_MS_DamageBind as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        commonCall(fp(g));
    }
}

pub fn ftCo_DamageBind_Anim(g: G) {
    melee_core::ct!("ftCo_DamageBind_Anim");
    let f = fp(g);
    f.grab_timer -= cd().x670;
    ext::ftCommon_GrabMash(f, cd().x674);
    if fp(g).grab_timer <= 0.0 {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_DamageBind_IASA(_g: G) { melee_core::ct!("ftCo_DamageBind_IASA");}

pub fn ftCo_DamageBind_Phys(g: G) {
    melee_core::ct!("ftCo_DamageBind_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_DamageBind_Coll(g: G) {
    melee_core::ct!("ftCo_DamageBind_Coll");
    ext::ft_80084280(g);
}
