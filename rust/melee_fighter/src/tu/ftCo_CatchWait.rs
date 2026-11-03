#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CatchWait.c (holding a grabbed fighter).
//! Every function of the TU; calls out go through `ext`. Checked against the C by
//! tests/tu_ftCo_CatchWait.rs.

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        efAsync_Spawn(g: G, queue: *mut core::ffi::c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj) = |a0, a1, a2, a3, a4| unsafe { let _ = melee_world::tu::efn::efasync::efAsync_Spawn(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, &[]); };
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        fn_800DB6C8(g: G) = |g: G| { unsafe { crate::tu::ftCo_CaptureWait::fn_800DB6C8(g as _); } };
        fn_800DA054(g: G) = crate::tu::ftCo_CatchPull::fn_800DA054;
        fn_800DA4FC(g: G) = crate::tu::ftCo_CatchAttack::fn_800DA4FC;
        ftCo_800DD1E4(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Throw::ftCo_800DD1E4(g as _) as _ } };
        it_802A7AAC(g: G) = |g: G| { unsafe { melee_items::tu::itlinkhookshot::it_802A7AAC(g as _); } };
        it_802BAC3C(g: G) = |g: G| { unsafe { melee_items::tu::itsamusgrapple::it_802BAC3C(g as _); } };
        ftCommon_CalcGroundAccel_Deaccel(fp: &mut Fighter, friction: f32) = crate::ftcommon::calc_ground_accel_deaccel;
        ftCommon_SetSelfMovementFromGroundedMovement(g: G) = |a0| crate::tu::ftcommon::ftCommon_SetSelfMovementFromGroundedMovement_fp(a0);
        ft_800841B8(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_800841b8(g as _, ev)); } };
        ftCo_800DC920(g: G, victim: G) = |g: G, victim: G| { unsafe { crate::tu::ftCo_CaptureCut::ftCo_800DC920(g as _, victim as _); } };
        ftCo_Fall_Enter(g: G) = crate::hooks::ftco_fall_enter;
    }
}

/// `fn_800DA4A0` as the C `HSD_GObjEvent` stored in `accessory1_cb`.
pub unsafe extern "C" fn fn_800DA4A0_cb(g: *mut HSD_GObj) { melee_core::ct!("fn_800DA4A0_cb"); fn_800DA4A0(g) }
/// `fn_800DA490` as the C `HSD_GObjEvent` stored in `take_dmg_cb`.
pub unsafe extern "C" fn fn_800DA490_cb(g: *mut HSD_GObj) { melee_core::ct!("fn_800DA490_cb"); fn_800DA490(g) }
/// `fn_800DA440` as the C `HSD_GObjEvent` handed to `ft_800841B8`.
pub unsafe extern "C" fn fn_800DA440_cb(g: *mut HSD_GObj) { melee_core::ct!("fn_800DA440_cb"); fn_800DA440(g) }

/// `fn_800DA1D8`: pull finished, enter CatchWait holding the victim.
pub fn fn_800DA1D8(g: G) {
    melee_core::ct!("fn_800DA1D8");
    fp(g).gr_vel = 0.0;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CatchWait as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.accessory1_cb = Some(fn_800DA4A0_cb);
    f.take_dmg_cb = Some(fn_800DA490_cb);
    f.anon_221b.set_x221B_b7(0);
    let jobj = unsafe { (*f.parts.offset((*(*f.ft_data).x8).x11 as isize)).joint };
    f.mv.co.capturedamage.x18 = jobj;
    let q = &mut f.x60C as *mut _ as *mut core::ffi::c_void;
    ext::efAsync_Spawn(g, q, 1, 0x41D, jobj);
    ext::ftCommon_8007EBAC(fp(g), 3, 0);
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    ext::fn_800DB6C8(fp(g).victim_gobj);
}

pub fn fn_800DA2B0(g: G) {
    melee_core::ct!("fn_800DA2B0");
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CatchWait as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.accessory1_cb = Some(fn_800DA4A0_cb);
    f.take_dmg_cb = Some(fn_800DA490_cb);
    ext::ftCommon_8007E2F4(f, 0x1FF);
}

pub fn ftCo_CatchWait_Anim(_g: G) { melee_core::ct!("ftCo_CatchWait_Anim");}

fn release_tether(g: G) {
    let kind = fp(g).kind;
    if kind == Ft_Kind_Link || kind == Ft_Kind_CLink {
        ext::it_802A7AAC(g);
    } else if kind == Ft_Kind_Samus {
        ext::it_802BAC3C(g);
    }
}

pub fn ftCo_CatchWait_IASA(g: G) {
    melee_core::ct!("ftCo_CatchWait_IASA");
    if fn_800DA4C0(g) {
        release_tether(g);
    } else if ext::ftCo_800DD1E4(g) {
        release_tether(g);
    }
}

pub fn ftCo_CatchWait_Phys(g: G) {
    melee_core::ct!("ftCo_CatchWait_Phys");
    let f = fp(g);
    let fr = crate::hooks::common_data().x64 * f.co_attrs.ground_friction;
    ext::ftCommon_CalcGroundAccel_Deaccel(f, fr);
    ext::ftCommon_SetSelfMovementFromGroundedMovement(g);
}

pub fn ftCo_CatchWait_Coll(g: G) {
    melee_core::ct!("ftCo_CatchWait_Coll");
    ext::ft_800841B8(g, Some(fn_800DA440_cb));
}

pub fn fn_800DA440(g: G) {
    melee_core::ct!("fn_800DA440");
    let victim = fp(g).victim_gobj;
    ext::ftCo_800DC920(g, victim);
    ext::ftCo_Fall_Enter(g);
    ext::ftCo_Fall_Enter(victim);
}

pub fn fn_800DA490(g: G) {
    melee_core::ct!("fn_800DA490");
    fp(g).take_dmg_cb = None;
}

pub fn fn_800DA4A0(g: G) {
    melee_core::ct!("fn_800DA4A0");
    ext::fn_800DA054(g);
}

pub fn fn_800DA4C0(g: G) -> bool {
    melee_core::ct!("fn_800DA4C0");
    if fp(g).input.pressed_buttons & 0x100 != 0 {
        ext::fn_800DA4FC(g);
        return true;
    }
    false
}
