#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CatchAttack.c (pummel). Every function of
//! the TU; calls out go through `ext`. Checked against the C by tests/tu_ftCo_CatchAttack.rs.

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        fn_800DA2B0(g: G) = crate::tu::ftCo_CatchWait::fn_800DA2B0;
        fn_800DA054(g: G) = crate::tu::ftCo_CatchPull::fn_800DA054;
        ftCommon_CalcGroundAccel_Deaccel(fp: &mut Fighter, friction: f32) = crate::ftcommon::calc_ground_accel_deaccel;
        ftCommon_SetSelfMovementFromGroundedMovement(g: G) = |g| crate::ftcommon::set_self_movement_from_grounded_movement(fp(g));
        ft_800841B8(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_800841b8(g as _, ev)); } };
        ftCo_800DC920(g: G, victim: G) = |g: G, victim: G| { unsafe { let _ = crate::tu::ftCo_CaptureCut::ftCo_800DC920(g as _, victim as _); } };
        ftCo_Fall_Enter(g: G) = crate::hooks::ftco_fall_enter;
    }
}

/// `fn_800DA678` as the C `HSD_GObjEvent` stored in `accessory1_cb`.
pub unsafe extern "C" fn fn_800DA678_cb(g: *mut HSD_GObj) { melee_core::ct!("fn_800DA678_cb"); fn_800DA678(g) }
/// `fn_800DA668` as the C `HSD_GObjEvent` stored in `take_dmg_cb`.
pub unsafe extern "C" fn fn_800DA668_cb(g: *mut HSD_GObj) { melee_core::ct!("fn_800DA668_cb"); fn_800DA668(g) }
/// `fn_800DA618` as the C `HSD_GObjEvent` handed to `ft_800841B8`.
pub unsafe extern "C" fn fn_800DA618_cb(g: *mut HSD_GObj) { melee_core::ct!("fn_800DA618_cb"); fn_800DA618(g) }

pub fn fn_800DA4FC(g: G) {
    melee_core::ct!("fn_800DA4FC");
    fp(g).gr_vel = 0.0;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CatchAttack as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.accessory1_cb = Some(fn_800DA678_cb);
    f.take_dmg_cb = Some(fn_800DA668_cb);
    ext::ftCommon_8007E2F4(f, 0x1FF);
}

pub fn ftCo_CatchAttack_Anim(g: G) {
    melee_core::ct!("ftCo_CatchAttack_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::fn_800DA2B0(g);
    }
}

pub fn ftCo_CatchAttack_IASA(_g: G) { melee_core::ct!("ftCo_CatchAttack_IASA");}

pub fn ftCo_CatchAttack_Phys(g: G) {
    melee_core::ct!("ftCo_CatchAttack_Phys");
    let f = fp(g);
    let fr = crate::hooks::common_data().x64 * f.co_attrs.ground_friction;
    ext::ftCommon_CalcGroundAccel_Deaccel(f, fr);
    ext::ftCommon_SetSelfMovementFromGroundedMovement(g);
}

pub fn ftCo_CatchAttack_Coll(g: G) {
    melee_core::ct!("ftCo_CatchAttack_Coll");
    ext::ft_800841B8(g, Some(fn_800DA618_cb));
}

pub fn fn_800DA618(g: G) {
    melee_core::ct!("fn_800DA618");
    let victim = fp(g).victim_gobj;
    ext::ftCo_800DC920(g, victim);
    ext::ftCo_Fall_Enter(g);
    ext::ftCo_Fall_Enter(victim);
}

pub fn fn_800DA668(g: G) {
    melee_core::ct!("fn_800DA668");
    fp(g).take_dmg_cb = None;
}

pub fn fn_800DA678(g: G) {
    melee_core::ct!("fn_800DA678");
    ext::fn_800DA054(g);
}
