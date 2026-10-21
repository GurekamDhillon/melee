#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ItemParasolFall.c (falling with the parasol
//! open). Every function of the TU; calls out of it go through `ext`. Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_ItemParasolFall.rs.

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftCo_SpecialAir_CheckInput(g: G) -> bool = |_| todo!("TODO(port) ftCo_SpecialAir_CheckInput (ftCo_SpecialAir.c)");
        ftCo_80095328(g: G, out: *mut bool) -> bool = |_, _| todo!("TODO(port) ftCo_80095328 (ftCo_JumpAerial.c)");
        ftCo_800C3B10(g: G) -> bool = crate::tu::ftco_aircatch::ftco_800c3b10;
        ftCo_80099A58(g: G) -> bool = |_| todo!("TODO(port) ftCo_80099A58 (ftCo_ItemThrow.c)");
        ftCo_AttackAir_CheckItemThrowInput(g: G) -> bool = crate::tu::ftCo_AttackAir::ftCo_AttackAir_CheckItemThrowInput;
        ft_80087988(g: G) -> bool = crate::ft0877::ft_80087988;
        ftCo_Fall_Enter(g: G) = |_| todo!("TODO(port) ftCo_Fall_Enter (ftCo_Fall.c)");
        ftCo_800CB870(g: G) -> bool = |_| todo!("TODO(port) ftCo_800CB870 (ftCo_AirCatch.c)");
        ftCo_ItemParasol_Phys(g: G) = crate::tu::ftCo_ItemParasolOpen::ftCo_ItemParasol_Phys;
        ft_8008370C(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_8008370c;
        ftCo_Landing_Enter_Basic(g: G) = |_| todo!("TODO(port) ftCo_Landing_Enter_Basic (ftCo_Landing.c)");
    }
}

pub fn ftCo_800CF280(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_ItemParasolFall as i32, Ft_MF_SkipHit | Ft_MF_SkipParasol, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

pub fn ftCo_ItemParasolFall_Anim(_g: G) {}

pub fn ftCo_ItemParasolFall_IASA(g: G) {
    if ext::ftCo_SpecialAir_CheckInput(g) { return; }
    if ext::ftCo_80095328(g, core::ptr::null_mut()) { return; }
    if ext::ftCo_800C3B10(g) { return; }
    if ext::ftCo_80099A58(g) { return; }
    if ext::ftCo_AttackAir_CheckItemThrowInput(g) { return; }
    if fp(g).input.lstick[0].y <= crate::hooks::common_data().close_parasol_threshold && ext::ft_80087988(g) {
        ext::ftCo_Fall_Enter(g);
    } else if ext::ftCo_800CB870(g) {
        return;
    }
}

pub fn ftCo_ItemParasolFall_Phys(g: G) {
    ext::ftCo_ItemParasol_Phys(g);
}

pub fn ftCo_ItemParasolFall_Coll(g: G) {
    ext::ft_8008370C(g, ext::ftCo_Landing_Enter_Basic);
}
