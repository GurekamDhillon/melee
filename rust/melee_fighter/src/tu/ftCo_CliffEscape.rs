#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CliffEscape.c (ledge roll). Every function
//! of the TU. Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_CliffEscape.rs.

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
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCo_CliffCatch_Phys(g: G) = crate::tu::ftcliffcommon::ftco_cliff_catch_phys;
        ftCo_CliffClimb_Anim(g: G) = crate::tu::ftCo_CliffClimb::ftCo_CliffClimb_Anim;
        ftCo_CliffClimb_Phys(g: G) = crate::tu::ftCo_CliffClimb::ftCo_CliffClimb_Phys;
        ftCo_CliffClimb_Coll(g: G) = crate::tu::ftCo_CliffClimb::ftCo_CliffClimb_Coll;
    }
}

pub fn ftCo_8009B040(g: G) {
    melee_core::ct!("ftCo_8009B040");
    let msid = if fp(g).dmg.x1830_percent < crate::hooks::common_data().x488 as f32 { ftCo_MS_CliffEscapeQuick } else { ftCo_MS_CliffEscapeSlow };
    // inlines.h `ftCo_Cliff_EnterState`
    ext::Fighter_ChangeMotionState(g, msid as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2F4(fp(g), 32);
    fp(g).set_x221D_b7(1);
    fp(g).set_x221D_b5(1);
    ext::ftCo_CliffCatch_Phys(g);
}

pub fn ftCo_CliffEscape_Anim(g: G) {
    melee_core::ct!("ftCo_CliffEscape_Anim");
    ext::ftCo_CliffClimb_Anim(g);
}

pub fn ftCo_CliffEscape_IASA(_g: G) { melee_core::ct!("ftCo_CliffEscape_IASA");}

pub fn ftCo_CliffEscape_Phys(g: G) {
    melee_core::ct!("ftCo_CliffEscape_Phys");
    ext::ftCo_CliffClimb_Phys(g);
}

pub fn ftCo_CliffEscape_Coll(g: G) {
    melee_core::ct!("ftCo_CliffEscape_Coll");
    ext::ftCo_CliffClimb_Coll(g);
}
