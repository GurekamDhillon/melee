#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_HammerWalk.c (hammer item: walk state). Every
//! function of the TU; calls out of it go through `ext`. Checked against the C (whole TU,
//! shipping contraction) by tests/tu_ftCo_HammerWalk.rs.

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
        ftWalkCommon_800DFC70(g: G) -> bool = crate::tu::ftwalkcommon::ftwalkcommon_800dfc70;
        ftWalkCommon_800E0060(g: G) = crate::tu::ftwalkcommon::ftwalkcommon_800e0060;
        ftCo_CalcYScaledKnockback(a: f32, scale: f32, b: f32) -> f32 = crate::tu::ftchangeparam::ftco_calc_y_scaled_knockback;
        // `Fighter_804D6524->xC`.
        Fighter_804D6524_xC() -> f32 = || crate::tu::ftchangeparam::ext::d6524().xC;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCo_800C54C4(f: &Fighter) -> u32 = crate::tu::ftCo_HammerWait::ftCo_800C54C4;
        ftCo_800C548C(f: &Fighter) -> f32 = crate::tu::ftCo_HammerWait::ftCo_800C548C;
        ftCo_800C4E94(f: &mut Fighter) = crate::tu::ftCo_HammerWait::ftCo_800C4E94;
        ftCo_800C4F64(g: G) = crate::tu::ftCo_HammerWait::ftCo_800C4F64;
        ftCo_800C5A50(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_HammerJump::ftCo_800C5A50(g as _) as _ } };
        ftCo_800C60C8(g: G) -> bool = crate::tu::ftco_0c60::ftco_800c60c8;
        ftCo_800C4E4C(g: G) -> bool = crate::tu::ftCo_HammerWait::ftCo_800C4E4C;
        ftCo_HammerWait_Coll(g: G) = crate::tu::ftCo_HammerWait::ftCo_HammerWait_Coll;
    }
}

pub fn ftCo_800C55CC(g: G) -> bool {
    melee_core::ct!("ftCo_800C55CC");
    if ext::ftWalkCommon_800DFC70(g) {
        ftCo_800C5614(g);
        return true;
    }
    false
}

pub fn ftCo_800C5614(g: G) {
    melee_core::ct!("ftCo_800C5614");
    let f = fp(g);
    let mut accel_mul = 1.0f32;
    if f.x34_scale.y != 1.0 {
        accel_mul = ext::ftCo_CalcYScaledKnockback(1.0, f.x34_scale.y, ext::Fighter_804D6524_xC());
    }
    let f = fp(g);
    f.mv.co.walk.accel_mul = accel_mul;
    f.mv.co.walk.x0 = f.gr_vel;
    let flags = ext::ftCo_800C54C4(fp(g));
    let start = ext::ftCo_800C548C(fp(g));
    ext::Fighter_ChangeMotionState(g, ftCo_MS_HammerWalk as i32, flags, start, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCo_800C4E94(fp(g));
}

pub fn ftCo_HammerWalk_Anim(g: G) {
    melee_core::ct!("ftCo_HammerWalk_Anim");
    ext::ftCo_800C4F64(g);
}

pub fn ftCo_HammerWalk_IASA(g: G) {
    melee_core::ct!("ftCo_HammerWalk_IASA");
    if ext::ftCo_800C5A50(g) { return; }
    if ext::ftCo_800C60C8(g) { return; }
    if ext::ftCo_800C4E4C(g) { return; }
}

pub fn ftCo_HammerWalk_Phys(g: G) {
    melee_core::ct!("ftCo_HammerWalk_Phys");
    ext::ftWalkCommon_800E0060(g);
}

pub fn ftCo_HammerWalk_Coll(g: G) {
    melee_core::ct!("ftCo_HammerWalk_Coll");
    ext::ftCo_HammerWait_Coll(g);
}
