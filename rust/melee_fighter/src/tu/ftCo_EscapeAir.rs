#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_EscapeAir.c (airdodge), including the
//! TARGET_PC script caps check. Every function of the TU (the static inline `inlineA0` too);
//! calls out of it go through `ext` (`cosf`/`sinf` default to group 1's MSL port). No fused
//! float sites. Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_EscapeAir.rs.
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
        ScriptGame_CapsActionAllowed(fp: &mut Fighter, flag: i32) -> i32 = |_, _| todo!("TODO(pc) ScriptGame_CapsActionAllowed (script_game.c)");
        ftCo_800D705C(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D705C;
        ftCommon_8007D9D4(fp: &Fighter) -> f32 = crate::ftcommon3::ftcommon_8007d9d4;
        cosf(x: f32) -> f32 = melee_runtime::gekko::cosf;
        sinf(x: f32) -> f32 = melee_runtime::gekko::sinf;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftAnim_8006EBA4(g: G) = |_| todo!("TODO(port) ftAnim_8006EBA4 (ftanim.c)");
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_80096900(g: G, a1: i32, a2: i32, ai: i32, mobility: f32, landing_lag: f32) = crate::tu::ftCo_FallSpecial::ftCo_80096900;
        ftCo_80095328(g: G, b: *mut bool) -> bool = |_, _| todo!("TODO(port) ftCo_80095328 (ftCo_ItemThrow.c)");
        ftCo_800C3B10(g: G) -> bool = |_| todo!("TODO(port) ftCo_800C3B10 (ftCo_AirCatch.c)");
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_80082C74(g: G, cb: HSD_GObjEvent) = |_, _| todo!("TODO(port) ft_80082C74 (ft_081B.c)");
        ftCo_LandingFallSpecial_Enter(g: G, allow_interrupt: i32, landing_lag: f32) = |_, _, _| todo!("TODO(port) ftCo_LandingFallSpecial_Enter (ftCo_Landing.c)");
    }
}

/// `cmd_vars` index `cmd_skip_decay`.
const CMD_SKIP_DECAY: usize = 0;

pub fn ftCo_80099A58(g: G) -> bool {
    let f = fp(g);
    if ext::ScriptGame_CapsActionAllowed(f, 2) == 0 {
        return false;
    }
    if fp(g).input.pressed_buttons & (HSD_PAD_R | HSD_PAD_L) as u32 != 0 {
        ftCo_80099A9C(g, cd().x334);
        return true;
    }
    false
}

/// static inline `inlineA0`.
pub fn inlineA0(f: &mut Fighter) {
    let c = cd();
    unsafe { f.mv.co.escapeair.self_vel = f.self_vel };
    if abs_c(f.input.lstick[0].x) < c.escapeair_deadzone.x && abs_c(f.input.lstick[0].y) < c.escapeair_deadzone.y {
        f.self_vel.y = 0.0;
        f.self_vel.x = 0.0;
        return;
    }
    let angle = ext::ftCommon_8007D9D4(f);
    f.self_vel.x = c.escapeair_force * ext::cosf(angle);
    f.self_vel.y = c.escapeair_force * ext::sinf(angle);
}

pub fn ftCo_80099A9C(g: G, timer: i32) {
    ext::ftCo_800D705C(g);
    let f = fp(g);
    inlineA0(f);
    f.cmd_vars[CMD_SKIP_DECAY] = 0;
    unsafe { f.mv.co.escapeair.timer = timer };
    ext::Fighter_ChangeMotionState(g, ftCo_MS_EscapeAir, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007EBAC(fp(g), 23, 0);
}

pub fn ftCo_EscapeAir_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_80096900(g, 1, 1, 0, cd().x340, cd().x344);
    }
}

pub fn ftCo_EscapeAir_IASA(g: G) {
    let f = fp(g);
    if unsafe { f.mv.co.escapeair.timer } != 0 {
        let self_vel = unsafe { f.mv.co.escapeair.self_vel };
        let mut b = false;
        if ext::ftCo_80095328(g, &mut b) {
            if b {
                fp(g).self_vel = self_vel;
            }
            return;
        }
        ext::ftCo_800D705C(g);
        unsafe { fp(g).mv.co.escapeair.timer -= 1 };
    }
    if ext::ftCo_800C3B10(g) {
        fp(g).self_vel.x *= 0.1;
    }
}

pub fn ftCo_EscapeAir_Phys(g: G) {
    let f = fp(g);
    if f.cmd_vars[CMD_SKIP_DECAY] == 0 {
        let d = cd().escapeair_decay;
        f.self_vel.x *= d;
        f.self_vel.y *= d;
    } else {
        ext::ft_80084DB0(g);
    }
}

pub fn ftCo_EscapeAir_Coll(g: G) {
    ext::ft_80082C74(g, Some(ftCo_80099D70));
}

/// static `ftCo_80099D70`; passed as a callback, hence `extern "C"`.
pub extern "C" fn ftCo_80099D70(g: G) {
    ext::ftCo_LandingFallSpecial_Enter(g, 0, cd().x344);
}
