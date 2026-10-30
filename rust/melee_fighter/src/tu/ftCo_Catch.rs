#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Catch.c (grab / dash grab start). Every
//! function of the TU, TARGET_PC path (fighter-caps NoGrab gate, m-ex onCatch dispatch); calls
//! out go through `ext`. Checked against the C by tests/tu_ftCo_Catch.rs.

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
        ftCo_800951D0(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_ItemThrow::ftCo_800951D0(g as _) as _ } };
        ftCo_80095254(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_ItemThrow::ftCo_80095254(g as _) as _ } };
        ftCo_800952DC(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_ItemThrow::ftCo_800952DC(g as _) as _ } };
        // TARGET_PC `FT_CAPS_ACTION_ALLOWED(fp, flag)` = `ScriptGame_CapsActionAllowed`.
        ScriptGame_CapsActionAllowed(fp: &Fighter, flag: i32) -> i32 = |_, _| todo!("TODO(port) ScriptGame_CapsActionAllowed (pc/gameworld/script_game.c)");
        fn_800D952C(g: G) -> bool = crate::tu::ftco_0d8e::fn_800d952c;
        fn_800D8EC8(g: G) -> bool = crate::tu::ftco_0d8e::fn_800d8ec8;
        fn_800D9228(g: G) -> bool = crate::tu::ftco_0d8e::fn_800d9228;
        fn_800D949C(g: G) = crate::tu::ftco_0d8e::fn_800d949c;
        fn_800D9558(g: G) -> bool = crate::tu::ftco_0d95::fn_800d9558;
        fn_800D9930(g: G) -> bool = crate::tu::ftco_0d95::fn_800d9930;
        fn_800D9C64(g: G) = crate::tu::ftco_0d95::fn_800d9c64;
        ftpickupitem_800948A8(g: G, item: *mut Item_GObj) = |g: G, item: *mut Item_GObj| { unsafe { let _ = crate::tu::ftpickupitem::ftpickupitem_800948A8(g as _, item as _); } };
        ftpickupitem_80094694(g: G, msid: i32, b: bool) = |g: G, msid: i32, b: bool| { unsafe { let _ = crate::tu::ftpickupitem::ftpickupitem_80094694(g as _, msid as _, b as _); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_8007E2D0(fp: &mut Fighter, arg1: i16, grab_cb: HSD_GObjEvent, unk_cb: HSD_GObjEvent, grabbed_cb: HSD_GObjInteraction) = crate::ftcommon2::ftcommon_8007e2d0;
        fn_800D9CE8(g: G) = crate::tu::ftCo_CatchPull::fn_800D9CE8;
        fn_800DAADC(g: G, other: G) = |g: G, other: G| { unsafe { let _ = crate::tu::ftCo_CapturePulled::fn_800DAADC(g as _, other as _); } };
        Mex_OnCatchDispatch(kind: i32, g: G, vanilla: *mut core::ffi::c_void) = |kind: i32, g: G, vanilla: *mut core::ffi::c_void| { unsafe { let _ = crate::platform::mex_dispatch(crate::platform::MEX_EVENT_ON_CATCH, kind, g, vanilla); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::tu::ft_08a1::ft_8008A2BC;
        ftCommon_CalcGroundAccel_Deaccel(fp: &mut Fighter, friction: f32) = crate::ftcommon::calc_ground_accel_deaccel;
        ftCommon_SetSelfMovementFromGroundedMovement(g: G) = |g| crate::ftcommon::set_self_movement_from_grounded_movement(fp(g));
        ft_80085030(g: G, friction: f32, facing_dir: f32) = crate::ft084e::ft_80085030;
        ft_800841B8(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_800841b8(g as _, ev)); } };
        ftCo_Fall_Enter(g: G) = crate::hooks::ftco_fall_enter;
    }
}

/// `fn_800D9CE8` (ftCo_CatchPull.c) as the C `HSD_GObjEvent` stored in `grab_cb`.
pub unsafe extern "C" fn fn_800D9CE8_cb(g: *mut HSD_GObj) { melee_core::ct!("fn_800D9CE8_cb"); ext::fn_800D9CE8(g) }
/// `fn_800D8BFC` as the C `HSD_GObjEvent`.
pub unsafe extern "C" fn fn_800D8BFC_cb(g: *mut HSD_GObj) { melee_core::ct!("fn_800D8BFC_cb"); fn_800D8BFC(g) }
/// `fn_800DAADC` (ftCo_Throw.c) as the C `HSD_GObjInteraction` stored in `grabbed_cb`.
pub unsafe extern "C" fn fn_800DAADC_cb(g: *mut HSD_GObj, o: *mut HSD_GObj) { melee_core::ct!("fn_800DAADC_cb"); ext::fn_800DAADC(g, o) }
/// `fn_800D8E30` as the C `HSD_GObjEvent` handed to `ft_800841B8`.
pub unsafe extern "C" fn fn_800D8E30_cb(g: *mut HSD_GObj) { melee_core::ct!("fn_800D8E30_cb"); fn_800D8E30(g) }

/// TARGET_PC `FT_CAPS_ACTION_ALLOWED(fp, 8)` (ScriptCaps_NoGrab).
fn caps_grab_allowed(g: G) -> bool {
    ext::ScriptGame_CapsActionAllowed(fp(g), 8) != 0
}

fn lr_held(f: &Fighter) -> bool {
    f.input.held_buttons[0] & HSD_PAD_LR != 0
}

fn a_pressed(f: &Fighter) -> bool {
    f.input.pressed_buttons & HSD_PAD_A as u32 != 0
}

pub fn ftCo_Catch_CheckInput(g: G) -> bool {
    melee_core::ct!("ftCo_Catch_CheckInput");
    if ext::ftCo_800951D0(g) {
        return true;
    }
    if !caps_grab_allowed(g) || !fn_800D8E94(g) || !ext::fn_800D952C(g) {
        return false;
    }
    let f = fp(g);
    if lr_held(f) && a_pressed(f) {
        ftCo_800D8C54(g, ftCo_MS_Catch as i32);
        return true;
    }
    false
}

pub fn ftCo_800D8A38(g: G) -> bool {
    melee_core::ct!("ftCo_800D8A38");
    if ext::ftCo_80095254(g) {
        return true;
    }
    if !caps_grab_allowed(g) || !fn_800D8E94(g) || !ext::fn_800D952C(g) {
        return false;
    }
    let f = fp(g);
    if lr_held(f) && a_pressed(f) {
        ftCo_800D8C54(g, ftCo_MS_CatchDash as i32);
        return true;
    }
    false
}

pub fn ftCo_800D8AE0(g: G) -> bool {
    melee_core::ct!("ftCo_800D8AE0");
    if ext::ftCo_800952DC(g) {
        return true;
    }
    if !caps_grab_allowed(g) || !fn_800D8E94(g) || !ext::fn_800D952C(g) {
        return false;
    }
    let f = fp(g);
    unsafe {
        if lr_held(f) && f.mv.co.common.x0 != 0 {
            ftCo_800D8C54(g, ftCo_MS_CatchDash as i32);
            return true;
        }
        if f.mv.co.common.x0 != 0 {
            f.mv.co.common.x0 -= 1;
        }
    }
    false
}

pub fn ftCo_800D8B9C(g: G) -> bool {
    melee_core::ct!("ftCo_800D8B9C");
    if !caps_grab_allowed(g) {
        return false;
    }
    let f = fp(g);
    unsafe {
        if a_pressed(f) && f.mv.co.guard.x24 != 0 {
            ftCo_800D8C54(g, ftCo_MS_CatchDash as i32);
            return true;
        }
        if f.mv.co.guard.x24 != 0 {
            f.mv.co.guard.x24 -= 1;
        }
    }
    false
}

/// `fn_800D8BFC` (static in the C): the grab callback that picks up the targeted item.
pub fn fn_800D8BFC(g: G) {
    melee_core::ct!("fn_800D8BFC");
    let item = fp(g).target_item_gobj;
    ext::ftpickupitem_800948A8(g, item);
    let f = fp(g);
    f.x1A64 = core::ptr::null_mut();
    f.target_item_gobj = core::ptr::null_mut();
    ext::ftpickupitem_80094694(g, 0x5D, true);
}

pub fn ftCo_800D8C54(g: G, msid: i32) {
    melee_core::ct!("ftCo_800D8C54");
    let f = fp(g);
    f.x74_self_accel.z = 0.0;
    f.x74_self_accel.y = 0.0;
    f.x74_self_accel.x = 0.0;
    f.mv.co.catch_.x0 = 0.0;
    ext::Fighter_ChangeMotionState(g, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftCommon_8007E2D0(fp(g), 1, Some(fn_800D9CE8_cb), Some(fn_800D8BFC_cb), Some(fn_800DAADC_cb));
    // m-ex onCatch (slot 44), at the epilogue (TARGET_PC).
    ext::Mex_OnCatchDispatch(fp(g).kind as i32, g, core::ptr::null_mut());
}

pub fn ftCo_Catch_Anim(g: G) {
    melee_core::ct!("ftCo_Catch_Anim");
    if ext::fn_800D8EC8(g) {
        return;
    }
    if ext::fn_800D9558(g) {
        return;
    }
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_CatchDash_Anim(g: G) {
    melee_core::ct!("ftCo_CatchDash_Anim");
    if !ext::fn_800D9228(g) && !ext::fn_800D9930(g) && !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_Catch_IASA(_g: G) { melee_core::ct!("ftCo_Catch_IASA");}

pub fn ftCo_CatchDash_IASA(_g: G) { melee_core::ct!("ftCo_CatchDash_IASA");}

pub fn ftCo_Catch_Phys(g: G) {
    melee_core::ct!("ftCo_Catch_Phys");
    let f = fp(g);
    let fr = crate::hooks::common_data().x64 * f.co_attrs.ground_friction;
    ext::ftCommon_CalcGroundAccel_Deaccel(f, fr);
    ext::ftCommon_SetSelfMovementFromGroundedMovement(g);
}

pub fn ftCo_CatchDash_Phys(g: G) {
    melee_core::ct!("ftCo_CatchDash_Phys");
    let f = fp(g);
    let fr = crate::hooks::common_data().x64 * f.co_attrs.ground_friction;
    ext::ft_80085030(g, fr, f.facing_dir);
}

pub fn ftCo_Catch_Coll(g: G) {
    melee_core::ct!("ftCo_Catch_Coll");
    ext::ft_800841B8(g, Some(fn_800D8E30_cb));
}

pub fn fn_800D8E30(g: G) {
    melee_core::ct!("fn_800D8E30");
    ext::fn_800D949C(g);
    ext::fn_800D9C64(g);
    ext::ftCo_Fall_Enter(g);
}

pub fn ftCo_CatchDash_Coll(g: G) {
    melee_core::ct!("ftCo_CatchDash_Coll");
    ext::ft_800841B8(g, Some(fn_800D8E30_cb));
}

pub fn fn_800D8E94(g: G) -> bool {
    melee_core::ct!("fn_800D8E94");
    let f = fp(g);
    let kind = f.kind;
    if (kind == Ft_Kind_Link || kind == Ft_Kind_CLink) && unsafe { !f.u.lk.xC.is_null() } {
        return false;
    }
    true
}
