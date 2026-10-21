#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ItemParasolOpen.c (parasol opening), plus the
//! ftCommon/inlines.h helpers `ftCo_ItemParasolGetFallMotionId` / `ftCo_ItemParasol_Phys` used by
//! the parasol TUs. Every function of the TU; calls out of it go through `ext`. Checked against
//! the C (whole TU, shipping contraction) by tests/tu_ftCo_ItemParasolOpen.rs.

use crate::types::*;

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
        ftGetParasolStatus(g: G) -> i32 = crate::ftcommon3::ft_get_parasol_status;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftAnim_8006EBA4(g: G) = |_| todo!("TODO(port) ftAnim_8006EBA4 (ftanim.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_800CF3C8(g: G) = crate::tu::ftCo_ItemParasolFallSpecial::ftCo_800CF3C8;
        ftCo_800CF280(g: G) = crate::tu::ftCo_ItemParasolFall::ftCo_800CF280;
        ftCo_SpecialAir_CheckInput(g: G) -> bool = |_| todo!("TODO(port) ftCo_SpecialAir_CheckInput (ftCo_SpecialAir.c)");
        ftCo_80095328(g: G, out: *mut bool) -> bool = |_, _| todo!("TODO(port) ftCo_80095328 (ftCo_JumpAerial.c)");
        ftCo_800C3B10(g: G) -> bool = crate::tu::ftco_aircatch::ftco_800c3b10;
        ftCo_AttackAir_CheckItemThrowInput(g: G) -> bool = crate::tu::ftCo_AttackAir::ftCo_AttackAir_CheckItemThrowInput;
        ftCo_800CB870(g: G) -> bool = |_| todo!("TODO(port) ftCo_800CB870 (ftCo_AirCatch.c)");
        ftCommon_Fall(f: &mut Fighter, grav: f32, terminal: f32) = crate::ftcommon::fall;
        ftCommon_CalcSelfAccel_AccelToVelClamped(f: &mut Fighter, accel: f32, target: f32, friction: f32) = crate::ftcommon::calc_self_accel_accel_to_vel_clamped;
        ft_8008370C(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_8008370c;
        ftCo_Landing_Enter_Basic(g: G) = |_| todo!("TODO(port) ftCo_Landing_Enter_Basic (ftCo_Landing.c)");
    }
}

/// `ftCo_ItemParasolGetFallMotionId` (ftCommon/inlines.h).
pub fn ftCo_ItemParasolGetFallMotionId(g: G, pe_msid: i32, co_msid: i32) -> i32 {
    let f = fp(g);
    if f.kind == Ft_Kind_Peach && unsafe { !f.u.pe.parasol_gobj_0.is_null() } {
        pe_msid
    } else {
        co_msid
    }
}

/// `ftCo_ItemParasol_Phys` (ftCommon/inlines.h).
pub fn ftCo_ItemParasol_Phys(g: G) {
    ftCo_ItemParasolOpen_Phys(g);
}

pub fn ftCo_800CEE70(g: G) -> bool {
    let f = fp(g);
    if f.x2221_b5() != 0 {
        return false;
    }
    if (f.x2221_b7() != 0 || f.input.lstick[0].y >= crate::hooks::common_data().open_parasol_threshold)
        && f.self_vel.y <= 0.0
        && ext::ftGetParasolStatus(g) == 6
    {
        ft_800CEF08(g);
        return true;
    }
    false
}

/// `ft_800CEF08_inline1`.
fn inline1(f: &mut Fighter, x2104: i32) {
    f.set_x2221_b7(0);
    f.set_x2221_b4(1);
    if f.x2221_b6() != 0 {
        f.x2104 = x2104;
        f.set_x2221_b6(0);
    }
}

fn enter(g: G, blend: f32) {
    let f = fp(g);
    f.mv.co.parasol_open.prev_msid = f.motion_id;
    let ms = ftCo_ItemParasolGetFallMotionId(g, ftPe_MS_ItemParasolOpen as i32, ftCo_MS_ItemParasolOpen as i32);
    ext::Fighter_ChangeMotionState(g, ms, 0, 0.0, 1.0, blend, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    inline1(fp(g), crate::hooks::common_data().x59C as i32);
}

pub fn ft_800CEF08(g: G) {
    enter(g, 0.0);
}

pub fn ftCo_800CEFE0(g: G, _arg1: i32) {
    enter(g, 10.0);
}

pub fn ftCo_ItemParasolOpen_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        if unsafe { fp(g).mv.co.parasol_open.prev_msid } == ftCo_MS_FallSpecial as i32 {
            ext::ftCo_800CF3C8(g);
        } else {
            ext::ftCo_800CF280(g);
        }
    }
}

pub fn ftCo_ItemParasolOpen_IASA(g: G) {
    if unsafe { fp(g).mv.co.parasol_open.prev_msid } != ftCo_MS_FallSpecial as i32 {
        if ext::ftCo_SpecialAir_CheckInput(g) { return; }
        if ext::ftCo_80095328(g, core::ptr::null_mut()) { return; }
        if ext::ftCo_800C3B10(g) { return; }
        if ext::ftCo_AttackAir_CheckItemThrowInput(g) { return; }
    }
    if ext::ftCo_800CB870(g) { return; }
}

pub fn ftCo_ItemParasolOpen_Phys(g: G) {
    let f = fp(g);
    let cd = crate::hooks::common_data();
    let mul = cd.x590;
    let grav = f.co_attrs.gravity * mul;
    let terminal_vel = f.co_attrs.terminal_velocity * mul;
    ext::ftCommon_Fall(f, grav, terminal_vel);
    let f = fp(g);
    let x = f.input.lstick[0].x;
    let (drift, target_vel) = if x.abs() >= cd.x258 {
        (cd.x58C * (x * f.co_attrs.air_drift_stick_mul), cd.x58C * (x * f.co_attrs.air_drift_max))
    } else {
        (0.0, 0.0)
    };
    let friction = f.co_attrs.aerial_friction;
    ext::ftCommon_CalcSelfAccel_AccelToVelClamped(f, drift, target_vel, friction);
}

pub fn ftCo_ItemParasolOpen_Coll(g: G) {
    ext::ft_8008370C(g, ext::ftCo_Landing_Enter_Basic);
}
