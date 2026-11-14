#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_KneeBend.c (jump squat). Every
//! function of the TU; calls out go through `ext`. Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_KneeBend.rs.

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
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_800D2D0C(g: G) -> bool = crate::tu::ft_0d27::ft_800d2d0c;
        ftCo_ItemScrew_Enter(g: G) = |g: G| { unsafe { crate::tu::ftCo_ItemScrew::ftCo_ItemScrew_Enter(g as _); } };
        ftCo_Jump_Enter(g: G) = |g: G| { unsafe { crate::tu::ftCo_Jump::ftCo_Jump_Enter(g as _); } };
        ftCo_Attack100_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput;
        ftCo_Catch_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Catch::ftCo_Catch_CheckInput(g as _) as _ } };
        ftCo_AttackHi4_CheckInputNoD0(g: G) -> bool = crate::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInputNoD0;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80083F88(g: G) = crate::tu::ft_081b::ft_80083f88;
    }
}

pub fn ftCo_KneeBend_Enter(g: G, jump_input: ftCo_JumpInput) {
    melee_core::ct!("ftCo_KneeBend_Enter");
    // mod host site (hooks-v1 0x800CB4E0, melee_core::modhook): one None check when no mod hooks it
    melee_core::mod_hook!(0x800CB4E0, [melee_core::modhook::Arg::Fighter(g as _), melee_core::modhook::Arg::I32(jump_input as i32)], ftCo_KneeBend_Enter__orig(g, jump_input));
    #[allow(unused_unsafe)] unsafe { ftCo_KneeBend_Enter__orig(g, jump_input) }
}

fn ftCo_KneeBend_Enter__orig(g: G, jump_input: ftCo_JumpInput) {
    let f = fp(g);
    unsafe {
        f.mv.co.kneebend.jump_input = jump_input;
        f.mv.co.kneebend.is_short_hop = 0;
    }
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KneeBend as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

pub fn ftCo_KneeBend_Anim(g: G) {
    melee_core::ct!("ftCo_KneeBend_Anim");
    let f = fp(g);
    if f.cur_anim_frame >= f.co_attrs.jump_startup_time || !ext::ftAnim_IsFramesRemaining(g) {
        if ext::ft_800D2D0C(g) {
            ext::ftCo_ItemScrew_Enter(g);
        } else {
            if let Some(h) = unsafe { crate::hooks::ENVOY_JUMP } {
                let short = unsafe { f.mv.co.kneebend.is_short_hop } != 0;
                h(f, short);
            }
            ext::ftCo_Jump_Enter(g);
        }
    }
}

pub fn ftCo_KneeBend_Check_ShortHop(g: G) {
    melee_core::ct!("ftCo_KneeBend_Check_ShortHop");
    let f = fp(g);
    let thr = crate::hooks::common_data().tap_jump_release_threshold;
    let ji = unsafe { f.mv.co.kneebend.jump_input };
    if (f.input.held_buttons[0] & (HSD_PAD_X | HSD_PAD_Y) as u32 == 0 && ji == JumpInput_XY)
        || (f.input.lstick[0].y < thr && ji == JumpInput_LStick)
        || (f.input.cstick[0].y < thr && ji == JumpInput_CStick)
    {
        unsafe { f.mv.co.kneebend.is_short_hop = 1 };
    }
}

pub fn ftCo_KneeBend_IASA(g: G) {
    melee_core::ct!("ftCo_KneeBend_IASA");
    if ext::ftCo_Attack100_CheckInput(g) { return; }
    if ext::ftCo_Catch_CheckInput(g) { return; }
    if ext::ftCo_AttackHi4_CheckInputNoD0(g) { return; }
    ftCo_KneeBend_Check_ShortHop(g);
}

pub fn ftCo_KneeBend_Phys(g: G) {
    melee_core::ct!("ftCo_KneeBend_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_KneeBend_Coll(g: G) {
    melee_core::ct!("ftCo_KneeBend_Coll");
    ext::ft_80083F88(g);
}
