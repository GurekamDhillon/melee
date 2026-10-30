#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_AttackAir.c (aerials). Every function of the
//! TU; calls out of it go through `ext` (unported ones panic until rewired; ported ones default
//! to the port). No fused site (`stick_x * facing_dir >= 0` is a lone product). Checked against
//! the C (whole TU, shipping contraction) by tests/tu_ftCo_AttackAir.rs.

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
        ftCo_800DF478(fp: &Fighter) -> bool = crate::ft0df::ftco_800df478;
        it_8026B30C(i: *mut Item_GObj) -> i32 = crate::hooks::it_8026b30c;
        ftCo_800CDDA0(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_0cdd::ftco_800cdda0(g as _); } };
        ftLk_AttackAir_Enter(g: G) = |_| todo!("TODO(port) ftLk_AttackAir_Enter (ftlink)");
        ftGw_AttackAirN_DecideAction(g: G) = |_| todo!("TODO(port) ftGw_AttackAirN_DecideAction (ftgamewatch)");
        ftCo_GetCStickAngle(fp: &Fighter) -> f32 = crate::ftcommon3::ftco_get_cstick_angle;
        ftCo_GetLStickAngle(fp: &Fighter) -> f32 = crate::ftcommon3::ftco_get_lstick_angle;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_Fall_Enter(g: G) = crate::hooks::ftco_fall_enter;
        ftCo_80095328(g: G, out: *mut i32) -> bool = |g: G, out: *mut i32| -> bool { unsafe { crate::tu::ftCo_ItemThrow::ftCo_80095328(g as _, out as _) as _ } };
        ftCo_800D7100(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D7100;
        ftCo_800C3B10(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftco_aircatch::ftco_800c3b10(g as _) as _ } };
        ftCo_800D705C(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D705C;
        ftCo_800CB870(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_JumpAerial::ftCo_800CB870(g as _) as _ } };
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_80082C74(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_80082c74(g as _, ev)); } };
    }
    /// `ftCo_LandingAir_EnterWithLag` (ftCo_LandingAir.c), passed as a callback: a TODO stub.
    pub unsafe extern "C" fn ftCo_LandingAir_EnterWithLag(_g: *mut HSD_GObj) {
        { unsafe { let _ = crate::tu::ftCo_LandingAir::ftCo_LandingAir_EnterWithLag(_g as _); } return; }
    }
}

/// `ABS(x)` (`x < 0 ? -x : x`).
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

fn throw_or_decide(g: G) -> bool {
    let f = fp(g);
    if pressed_a(f) || ext::ftCo_800DF478(f) {
        if (ftCo_AttackAir_GetMsidFromCStick(f) as u32).wrapping_sub(ftCo_MS_AttackAirN as u32) <= 1
            && !f.item_gobj.is_null()
            && ext::it_8026B30C(f.item_gobj) == 3
        {
            ext::ftCo_800CDDA0(g);
        } else {
            decideFighter(g);
        }
        return true;
    }
    false
}

#[inline(always)]
fn pressed_a(f: &Fighter) -> bool {
    f.input.pressed_buttons & HSD_PAD_A as u32 != 0
}

/// static `checkItemThrowInput`.
pub fn checkItemThrowInput(g: G) -> bool {
    melee_core::ct!("checkItemThrowInput");
    throw_or_decide(g)
}

/// `ftCo_AttackAir_CheckItemThrowInput` (same body as checkItemThrowInput).
pub fn ftCo_AttackAir_CheckItemThrowInput(g: G) -> bool {
    melee_core::ct!("ftCo_AttackAir_CheckItemThrowInput");
    throw_or_decide(g)
}

/// static `decideFighter`.
pub fn decideFighter(g: G) {
    melee_core::ct!("decideFighter");
    match fp(g).kind {
        Ft_Kind_Link | Ft_Kind_CLink => ext::ftLk_AttackAir_Enter(g),
        Ft_Kind_GameWatch => ext::ftGw_AttackAirN_DecideAction(g),
        _ => ftCo_AttackAir_EnterFromCStick(g),
    }
}

/// `ftCo_AttackAir_GetMsidFromCStick`.
pub fn ftCo_AttackAir_GetMsidFromCStick(f: &Fighter) -> i32 {
    melee_core::ct!("ftCo_AttackAir_GetMsidFromCStick");
    let (x, y, angle);
    if ext::ftCo_800DF478(f) {
        x = f.input.cstick[0].x;
        y = f.input.cstick[0].y;
        angle = ext::ftCo_GetCStickAngle(f);
    } else {
        x = f.input.lstick[0].x;
        y = f.input.lstick[0].y;
        angle = ext::ftCo_GetLStickAngle(f);
    }
    let c = cd();
    if abs_c(x) < c.xDC && abs_c(y) < c.xE0 {
        return ftCo_MS_AttackAirN as i32;
    }
    if angle > c.x20_radians {
        return ftCo_MS_AttackAirHi as i32;
    }
    if angle < -c.x20_radians {
        return ftCo_MS_AttackAirLw as i32;
    }
    if x * f.facing_dir >= 0.0 {
        return ftCo_MS_AttackAirF as i32;
    }
    ftCo_MS_AttackAirB as i32
}

pub fn ftCo_AttackAir_EnterFromCStick(g: G) {
    melee_core::ct!("ftCo_AttackAir_EnterFromCStick");
    let msid = ftCo_AttackAir_GetMsidFromCStick(fp(g));
    ftCo_AttackAir_EnterFromMsid(g, msid);
}

pub fn ftCo_AttackAir_EnterFromMsid(g: G, msid: i32) {
    melee_core::ct!("ftCo_AttackAir_EnterFromMsid");
    let f = fp(g);
    f.set_allow_interrupt(0);
    f.cmd_vars[0] = 0;
    f.anon_2210.throw_flags = 0;
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_KeepFastFall as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
}

/// `ftCheckThrowB3` (ft/inlines.h): test and clear throw_flags_b3.
fn check_throw_b3(f: &mut Fighter) -> bool {
    unsafe {
        if f.anon_2210.anon_0.throw_flags_b3() != 0 {
            f.anon_2210.anon_0.set_throw_flags_b3(0);
            true
        } else {
            false
        }
    }
}

pub fn ftCo_AttackAir_Anim(g: G) {
    melee_core::ct!("ftCo_AttackAir_Anim");
    let f = fp(g);
    if check_throw_b3(f) {
        f.facing_dir = -f.facing_dir;
    }
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_Fall_Enter(g);
    }
}

/// The `DO_IASA` macro shared by the five aerial IASA callbacks.
fn do_iasa(g: G) {
    if fp(g).allow_interrupt() != 0 {
        let _ = ext::ftCo_80095328(g, core::ptr::null_mut())
            || ext::ftCo_800D7100(g)
            || ext::ftCo_800C3B10(g)
            || checkItemThrowInput(g)
            || ext::ftCo_800D705C(g)
            || ext::ftCo_800CB870(g);
    }
}

pub fn ftCo_AttackAirN_IASA(g: G) {
    melee_core::ct!("ftCo_AttackAirN_IASA");
    do_iasa(g);
}
pub fn ftCo_AttackAirF_IASA(g: G) {
    melee_core::ct!("ftCo_AttackAirF_IASA");
    do_iasa(g);
}
pub fn ftCo_AttackAirB_IASA(g: G) {
    melee_core::ct!("ftCo_AttackAirB_IASA");
    do_iasa(g);
}
pub fn ftCo_AttackAirHi_IASA(g: G) {
    melee_core::ct!("ftCo_AttackAirHi_IASA");
    do_iasa(g);
}
pub fn ftCo_AttackAirLw_IASA(g: G) {
    melee_core::ct!("ftCo_AttackAirLw_IASA");
    do_iasa(g);
}

pub fn ftCo_AttackAir_Phys(g: G) {
    melee_core::ct!("ftCo_AttackAir_Phys");
    ext::ft_80084DB0(g);
}

pub fn ftCo_AttackAir_Coll(g: G) {
    melee_core::ct!("ftCo_AttackAir_Coll");
    ext::ft_80082C74(g, Some(ext::ftCo_LandingAir_EnterWithLag));
}
