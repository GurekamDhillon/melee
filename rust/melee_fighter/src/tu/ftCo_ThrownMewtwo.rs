#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ThrownMewtwo.c (every function). Calls out of
//! the TU go through `ext` (unported ones panic until rewired). Checked against the C (whole TU,
//! shipping contraction) by tests/tu_ftCo_ThrownMewtwo.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

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

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    ext_fns! {
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_800DE508(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Thrown::ftCo_800DE508(g as _); } };
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftAnim_8006EBA4(g: G) = |g| unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g) };
    }
}

/// `ftCo_800DE508` as the `accessory1_cb` value.
pub unsafe extern "C" fn ftCo_800DE508_cb(g: G) {
    ext::ftCo_800DE508(g)
}

pub fn ftCo_800BD0E8(g: G, msid: i32) {
    let f = fp(g);
    f.facing_dir = fp(f.victim_gobj).facing_dir;
    unsafe { f.mv.co.capturekoopa.x0 = 0 };
    let flags = if f.x2222_b6() != 0 { Ft_MF_FreezeState } else { Ft_MF_None };
    let victim = f.victim_gobj;
    ext::Fighter_ChangeMotionState(g, msid, flags, 0.0, 1.0, 0.0, victim);
    fp(g).accessory1_cb = Some(ftCo_800DE508_cb);
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    ext::ftAnim_8006EBA4(g);
}

pub fn ftCo_ThrownMewtwo_Anim(_g: G) {}
pub fn ftCo_ThrownMewtwo_IASA(_g: G) {}
pub fn ftCo_ThrownMewtwo_Phys(_g: G) {}
pub fn ftCo_ThrownMewtwo_Coll(_g: G) {}
pub fn ftCo_ThrownMewtwoAir_Anim(_g: G) {}
pub fn ftCo_ThrownMewtwoAir_IASA(_g: G) {}
pub fn ftCo_ThrownMewtwoAir_Phys(_g: G) {}
pub fn ftCo_ThrownMewtwoAir_Coll(_g: G) {}
