#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_AttackDash.c (dash attack). Every function of
//! the TU; calls out of it go through `ext` (unported ones panic until rewired; ported ones
//! default to the port). No fused site (`x50 * ground_friction` is a lone product). Checked
//! against the C (whole TU, shipping contraction) by tests/tu_ftCo_AttackDash.rs.

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
        it_8026B30C(i: *mut Item_GObj) -> i32 = crate::hooks::it_8026b30c;
        ftCo_800957F4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { let _ = crate::tu::ftCo_ItemThrow::ftCo_800957F4(g as _, msid as _); } };
        ftCo_Attack_800CCF58(g: G, kind: i32) = |g: G, kind: i32| { unsafe { let _ = crate::tu::ftswing::ftCo_Attack_800CCF58(g as _, kind as _); } };
        ftKb_MexKirbyLike(kind: i32) -> i32 = |_| todo!("TODO(port) ftKb_MexKirbyLike (ftkirby.c)");
        ftKb_SpecialN_800F1F68(g: G) = |_| todo!("TODO(port) ftKb_SpecialN_800F1F68 (ftkirbyspecialn.c)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_800D8AE0(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Catch::ftCo_800D8AE0(g as _) as _ } };
        ftCo_Wait_IASA(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Wait::ftCo_Wait_IASA(g as _); } };
        ft_80085030(g: G, friction: f32, facing_dir: f32) = crate::ft084e::ft_80085030;
        ft_80084104(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80084104(g as _); } };
    }
}

/// `FTKB_IS_KIRBY(kind)` (ft/forward.h, TARGET_PC: m-ex Kirby clones count).
fn is_kirby(kind: FighterKind) -> bool {
    kind as i32 == Ft_Kind_Kirby as i32
        || ((kind as i32) >= Ft_Kind_Mex0 as i32 && (kind as i32) < Ft_Kind_Max as i32 && ext::ftKb_MexKirbyLike(kind as i32) != 0)
}

/// `ftCo_AttackDash_CheckInput`.
pub fn ftCo_AttackDash_CheckInput(g: G) -> bool {
    let f = fp(g);
    if f.input.pressed_buttons & HSD_PAD_A as u32 != 0 {
        if !f.item_gobj.is_null() {
            if f.input.held_buttons[0] & HSD_PAD_LR != 0 || ext::it_8026B30C(f.item_gobj) == 0 {
                ext::ftCo_800957F4(g, ftCo_MS_LightThrowDash as i32);
                return true;
            }
            if ext::it_8026B30C(f.item_gobj) == 2 {
                ext::ftCo_Attack_800CCF58(g, 4);
                return true;
            }
        }
        decideFighter(g);
        return true;
    }
    false
}

/// static `decideFighter`: `switch (FTKB_CANON_KIND(kind))`.
pub fn decideFighter(g: G) {
    if is_kirby(fp(g).kind) {
        ext::ftKb_SpecialN_800F1F68(g);
    } else {
        doEnter(g);
    }
}

/// static `doEnter`.
pub fn doEnter(g: G) {
    let f = fp(g);
    f.set_allow_interrupt(0);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_AttackDash as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    f.mv.co.attackdash.x0 = 0;
}

pub fn ftCo_AttackDash_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

/// `ftCo_AttackDash_SetMv0`: `mv.co.attackdash.x0 = p_ftCommonData->x68` (float -> int).
pub fn ftCo_AttackDash_SetMv0(g: G) {
    fp(g).mv.co.attackdash.x0 = cd().x68 as i32;
}

pub fn ftCo_AttackDash_IASA(g: G) {
    if !ext::ftCo_800D8AE0(g) && fp(g).allow_interrupt() != 0 {
        ext::ftCo_Wait_IASA(g);
    }
}

pub fn ftCo_AttackDash_Phys(g: G) {
    let f = fp(g);
    let fr = cd().x50 * f.co_attrs.ground_friction;
    let dir = f.facing_dir;
    ext::ft_80085030(g, fr, dir);
}

pub fn ftCo_AttackDash_Coll(g: G) {
    ext::ft_80084104(g);
}
