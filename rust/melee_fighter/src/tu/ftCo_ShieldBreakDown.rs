#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ShieldBreakDown.c (TARGET_PC build). Every function of the TU;
//! calls out of it go through `ext` (unported ones panic until rewired). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_ShieldBreakDown.rs.
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
        ftCommon_8007D7FC(f: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftCo_80097570(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_DownBound::ftCo_80097570(g as _) as _ } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_800978D4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_DownBound::ftCo_800978D4(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_80098F3C(g: G) = crate::tu::ftCo_ShieldBreakStand::ftCo_80098F3C;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80083F88(g: G) = crate::tu::ft_081b::ft_80083f88;
    }
}

pub fn ftCo_80098E3C(g: G) {
    melee_core::ct!("ftCo_80098E3C");
    let f = fp(g);
    if f.ground_or_air == GA_Air as _ {
        ext::ftCommon_8007D7FC(f);
    }
    let msid = if ext::ftCo_80097570(g) { ftCo_MS_ShieldBreakDownU } else { ftCo_MS_ShieldBreakDownD };
    ext::Fighter_ChangeMotionState(
        g,
        msid as i32,
        (Ft_MF_KeepColAnimHitStatus | Ft_MF_SkipModel | Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim) as u32,
        0.0,
        1.0,
        0.0,
        core::ptr::null_mut(),
    );
    ext::ftCo_800978D4(g);
}

pub fn ftCo_ShieldBreakDown_Anim(g: G) {
    melee_core::ct!("ftCo_ShieldBreakDown_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_80098F3C(g);
    }
}

pub fn ftCo_ShieldBreakDown_IASA(_g: G) { melee_core::ct!("ftCo_ShieldBreakDown_IASA");}

pub fn ftCo_ShieldBreakDown_Phys(g: G) {
    melee_core::ct!("ftCo_ShieldBreakDown_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_ShieldBreakDown_Coll(g: G) {
    melee_core::ct!("ftCo_ShieldBreakDown_Coll");
    ext::ft_80083F88(g);
}
