#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ItemParasolFallSpecial.c (helpless fall with
//! the parasol open). Every function of the TU; calls out of it go through `ext`. Checked against
//! the C (whole TU, shipping contraction) by tests/tu_ftCo_ItemParasolFallSpecial.rs.

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
        ft_80087988(g: G) -> bool = crate::ft0877::ft_80087988;
        ftCo_800968C8(g: G) = |g: G| { unsafe { crate::tu::ftCo_FallSpecial::ftCo_800968C8(g as _); } };
        ftCo_800CB870(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_JumpAerial::ftCo_800CB870(g as _) as _ } };
        ftCo_ItemParasol_Phys(g: G) = crate::tu::ftCo_ItemParasolOpen::ftCo_ItemParasol_Phys;
        ft_8008370C(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_8008370c;
        ftCo_LandingFallSpecial_Enter_Basic(g: G) = |g: G| { unsafe { crate::tu::ftCo_Landing::ftCo_LandingFallSpecial_Enter_Basic(g as _); } };
    }
}

pub fn ftCo_800CF3C8(g: G) {
    melee_core::ct!("ftCo_800CF3C8");
    let ms = crate::tu::ftCo_ItemParasolOpen::ftCo_ItemParasolGetFallMotionId(
        g, ftPe_MS_ItemParasolFall as i32, ftCo_MS_ItemParasolFallSpecial as i32);
    ext::Fighter_ChangeMotionState(g, ms, Ft_MF_SkipHit | Ft_MF_SkipParasol, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

pub fn ftCo_ItemParasolFallSpecial_Anim(_g: G) { melee_core::ct!("ftCo_ItemParasolFallSpecial_Anim");}

pub fn ftCo_ItemParasolFallSpecial_IASA(g: G) {
    melee_core::ct!("ftCo_ItemParasolFallSpecial_IASA");
    if fp(g).input.lstick[0].y <= crate::hooks::common_data().close_parasol_threshold && ext::ft_80087988(g) {
        ext::ftCo_800968C8(g);
    } else if ext::ftCo_800CB870(g) {
        return;
    }
}

pub fn ftCo_ItemParasolFallSpecial_Phys(g: G) {
    melee_core::ct!("ftCo_ItemParasolFallSpecial_Phys");
    ext::ftCo_ItemParasol_Phys(g);
}

pub fn ftCo_ItemParasolFallSpecial_Coll(g: G) {
    melee_core::ct!("ftCo_ItemParasolFallSpecial_Coll");
    ext::ft_8008370C(g, ext::ftCo_LandingFallSpecial_Enter_Basic);
}
