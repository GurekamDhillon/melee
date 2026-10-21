#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ItemParasolDamageFall.c (tumble with the
//! parasol). Every function of the TU; calls out of it go through `ext`. Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_ItemParasolDamageFall.rs.

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
        ftCommon_ClampAirDrift(f: &mut Fighter) = crate::ftcommon::clamp_air_drift;
        ftCo_DamageFall_IASA(g: G) = |_| todo!("TODO(port) ftCo_DamageFall_IASA (ftCo_DamageFall.c)");
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_8008370C(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_8008370c;
        ftCo_80090984(g: G) = |_| todo!("TODO(port) ftCo_80090984 (ftCo_DamageIce.c)");
    }
}

pub fn ftCo_800CF4DC(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_ItemParasolDamageFall as i32, Ft_MF_KeepFastFall, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftCommon_ClampAirDrift(fp(g));
}

pub fn ftCo_ItemParasolDamageFall_Anim(_g: G) {}

pub fn ftCo_ItemParasolDamageFall_IASA(g: G) {
    ext::ftCo_DamageFall_IASA(g);
}

pub fn ftCo_ItemParasolDamageFall_Phys(g: G) {
    ext::ft_80084DB0(g);
}

pub fn ftCo_ItemParasolDamageFall_Coll(g: G) {
    ext::ft_8008370C(g, ext::ftCo_80090984);
}
