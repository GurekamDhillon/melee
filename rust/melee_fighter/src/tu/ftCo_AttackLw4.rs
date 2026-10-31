#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_AttackLw4.c (down smash). Every function of
//! the TU; calls out of it go through `ext` (unported ones panic until rewired; ported ones
//! default to the port). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ftCo_AttackLw4.rs.

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
        ftCo_800DF3A8(fp: &Fighter) -> bool = crate::ft0df::ftco_800df3a8;
        ftCo_80094E54(fp: &Fighter) -> bool = |_| todo!("TODO(port) ftCo_80094E54 (ftCo_ItemThrow.c)");
        ftCo_800DF3DC(fp: &Fighter) -> bool = crate::ft0df::ftco_800df3dc;
        ftCo_800957F4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { crate::tu::ftCo_ItemThrow::ftCo_800957F4(g as _, msid as _); } };
        ftNs_AttackLw4_Enter(g: G) = |_| todo!("TODO(port) ftNs_AttackLw4_Enter (ftnessattacklw4.c)");
        Mex_OnDSmashDispatch(kind: i32, g: G, vanilla: *mut core::ffi::c_void) = |kind: i32, g: G, vanilla: *mut core::ffi::c_void| { unsafe { let _ = crate::platform::mex_dispatch(crate::platform::MEX_EVENT_ON_DSMASH, kind, g, vanilla); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_Wait_IASA(g: G) = |g: G| { unsafe { crate::tu::ftCo_Wait::ftCo_Wait_IASA(g as _); } };
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80084104(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80084104(g as _); } };
    }
}

/// static `checkLStick(fp)`: A pressed with the stick flicked down.
pub fn checkLStick(fp: &Fighter) -> bool {
    melee_core::ct!("checkLStick");
    fp.input.pressed_buttons & HSD_PAD_A as u32 != 0
        && fp.input.lstick[0].y <= cd().xD4
        && (fp.active_timer.lstick.y as f32) < cd().xD8
}

/// `ftCo_AttackLw4_CheckInput`.
pub fn ftCo_AttackLw4_CheckInput(g: G) -> bool {
    melee_core::ct!("ftCo_AttackLw4_CheckInput");
    let f = fp(g);
    if checkLStick(f) || ext::ftCo_800DF3A8(f) {
        if !f.item_gobj.is_null() && (ext::ftCo_80094E54(f) || ext::ftCo_800DF3DC(f)) {
            ext::ftCo_800957F4(g, ftCo_MS_LightThrowLw4 as i32);
            return true;
        }
        if f.kind as i32 == Ft_Kind_Ness as i32 {
            ext::ftNs_AttackLw4_Enter(g);
        } else {
            doEnter(g);
        }
        return true;
    }
    false
}

/// static `doEnter`.
pub fn doEnter(g: G) {
    melee_core::ct!("doEnter");
    let f = fp(g);
    ext::Mex_OnDSmashDispatch(f.kind as i32, g, core::ptr::null_mut());
    f.set_allow_interrupt(0);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_AttackLw4 as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
}

pub fn ftCo_AttackLw4_Anim(g: G) {
    melee_core::ct!("ftCo_AttackLw4_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_AttackLw4_IASA(g: G) {
    melee_core::ct!("ftCo_AttackLw4_IASA");
    if fp(g).allow_interrupt() != 0 {
        ext::ftCo_Wait_IASA(g);
    }
}

pub fn ftCo_AttackLw4_Phys(g: G) {
    melee_core::ct!("ftCo_AttackLw4_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_AttackLw4_Coll(g: G) {
    melee_core::ct!("ftCo_AttackLw4_Coll");
    ext::ft_80084104(g);
}
