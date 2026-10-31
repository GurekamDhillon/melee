#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CaptureJump.c (jumping out of a grab). No
//! fused sites. Smoke-tested by tests/tu_ftCo_CaptureJump.rs.

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
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCo_800DC920(a: G, g: G) = crate::tu::ftCo_CaptureCut::ftCo_800DC920;
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
        ftCo_800CB870(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_JumpAerial::ftCo_800CB870(g as _) as _ } };
        ftCommon_Fall(f: &mut Fighter, gravity: f32, terminal: f32) = crate::ftcommon::fall;
        ftCommon_CalcSelfAccel_Drift(f: &mut Fighter) = crate::ftcommon::calc_self_accel_drift;
        ftCo_AirCatchHit_Coll(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ftco_air_catch_hit_coll(g as _); } };
    }
}

/// `fn_800DC070`: enter CaptureJump (the victim is released first).
pub fn fn_800DC070(g: G) {
    melee_core::ct!("fn_800DC070");
    let f = fp(g);
    let victim = f.victim_gobj;
    ext::ftCommon_8007D5D4(f);
    f.self_vel.x = -f.facing_dir * cd().x374;
    f.self_vel.y = cd().x378;
    f.mv.co.buryjump.x0 = 0.0;
    ext::ftCo_800DC920(victim, g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureJump as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

/// `ftCo_CaptureJump_Anim`.
pub fn ftCo_CaptureJump_Anim(g: G) {
    melee_core::ct!("ftCo_CaptureJump_Anim");
    let f = fp(g);
    unsafe { f.mv.co.buryjump.x0 += 1.0 };
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_Fall_Enter(g);
    }
}

/// `ftCo_CaptureJump_IASA`.
pub fn ftCo_CaptureJump_IASA(g: G) {
    melee_core::ct!("ftCo_CaptureJump_IASA");
    let f = fp(g);
    if unsafe { f.mv.ca.specials.grav } >= cd().x3B8 {
        if ext::ftCo_SpecialAir_CheckInput(g) { return; }
        if ext::ftCo_80095328(g, core::ptr::null_mut()) { return; }
        if ext::ftCo_800D7100(g) { return; }
        if ext::ftCo_800C3B10(g) { return; }
        if ext::ftCo_80099A58(g) { return; }
        if ext::ftCo_AttackAir_CheckItemThrowInput(g) { return; }
        if ext::ftCo_800D705C(g) { return; }
        if ext::ftCo_800CB870(g) { return; }
    }
}

/// `ftCo_CaptureJump_Phys`.
pub fn ftCo_CaptureJump_Phys(g: G) {
    melee_core::ct!("ftCo_CaptureJump_Phys");
    let f = fp(g);
    let (gr, tv) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
    ext::ftCommon_Fall(f, gr, tv);
    ext::ftCommon_CalcSelfAccel_Drift(f);
}

/// `ftCo_CaptureJump_Coll`.
pub fn ftCo_CaptureJump_Coll(g: G) {
    melee_core::ct!("ftCo_CaptureJump_Coll");
    ext::ftCo_AirCatchHit_Coll(g);
}
