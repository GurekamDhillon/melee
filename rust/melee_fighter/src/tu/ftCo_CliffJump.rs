#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CliffJump.c (ledge jump). Every function of
//! the TU. One fused site: ftCo_8009B2F8 `self_vel.x += facing * h_vel` -> `fmadds`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_CliffJump.rs.

use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCo_CliffCatch_Phys(g: G) = crate::tu::ftcliffcommon::ftco_cliff_catch_phys;
        ftCo_Jump_GetInput(g: G) -> i32 = |g: G| -> i32 { unsafe { crate::tu::ftCo_Jump::ftCo_Jump_GetInput(g as _) as _ } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_CliffClimb_Coll(g: G) = crate::tu::ftCo_CliffClimb::ftCo_CliffClimb_Coll;
        ftCo_Fall_Enter(g: G) = crate::hooks::ftco_fall_enter;
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_800835B0(g: G, a: fn(G, i32) -> bool, b: fn(G)) = |g: G, a: fn(G, i32) -> bool, b: fn(G)| { unsafe { let _ = crate::tu::ft_081b::ft_800835b0(g as _, a as _, b as _); } };
        ftCo_80096CC8(g: G, line: i32) -> bool = |g: G, line: i32| -> bool { unsafe { crate::tu::ftCo_FallSpecial::ftCo_80096CC8(g as _, line as _) as _ } };
        ft_80082B1C(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80082b1c(g as _); } };
    }
}

pub fn ftCo_8009B170(g: G) -> bool {
    if ext::ftCo_Jump_GetInput(g) != 0 {
        ftCo_8009B1B8(g);
        return true;
    }
    false
}

/// static `ftCo_8009B1B8`.
pub fn ftCo_8009B1B8(g: G) {
    let msid = if fp(g).dmg.x1830_percent < crate::hooks::common_data().x488 as f32 { ftCo_MS_CliffJumpQuick1 } else { ftCo_MS_CliffJumpSlow1 };
    // inlines.h `ftCo_Cliff_EnterState`
    ext::Fighter_ChangeMotionState(g, msid as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2F4(fp(g), 32);
    fp(g).set_x221D_b7(1);
    ext::ftCo_CliffCatch_Phys(g);
}

pub fn ftCo_CliffJump1_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ftCo_8009B2F8(g);
    }
}

pub fn ftCo_CliffJump1_IASA(_g: G) {}

pub fn ftCo_CliffJump1_Phys(g: G) {
    ext::ftCo_CliffCatch_Phys(g);
}

pub fn ftCo_CliffJump1_Coll(g: G) {
    ext::ftCo_CliffClimb_Coll(g);
}

fn jump2_msid(g: G) -> i32 {
    (if fp(g).motion_id == ftCo_MS_CliffJumpQuick1 as i32 { ftCo_MS_CliffJumpQuick2 } else { ftCo_MS_CliffJumpSlow2 }) as i32
}

/// static `ftCo_8009B2F8`.
pub fn ftCo_8009B2F8(g: G) {
    let msid = jump2_msid(g);
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    f.mv.co.cliffjump.x0 = 0;
    f.self_vel.x = fmadds(f.facing_dir, f.co_attrs.ledge_jump_horizontal_velocity, f.self_vel.x);
    f.self_vel.y = f.co_attrs.ledge_jump_vertical_velocity;
}

pub fn ftCo_8009B390(g: G, force_mul: f32) {
    let msid = jump2_msid(g);
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    f.mv.co.cliffjump.x0 = 0;
    f.self_vel.y = f.co_attrs.ledge_jump_vertical_velocity * force_mul;
}

pub fn ftCo_CliffJump2_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_Fall_Enter(g);
    }
}

pub fn ftCo_CliffJump2_IASA(_g: G) {}

pub fn ftCo_CliffJump2_Phys(g: G) {
    let f = fp(g);
    if unsafe { f.mv.co.cliffjump.x0 } == 0 {
        f.mv.co.cliffjump.x0 = 1;
    } else {
        ext::ft_80084DB0(g);
    }
}

pub fn ftCo_CliffJump2_Coll(g: G) {
    ext::ft_800835B0(g, ext::ftCo_80096CC8, ext::ft_80082B1C);
}
