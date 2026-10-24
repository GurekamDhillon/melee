#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CaptureMewtwo.c (held by Mewtwo's throw, ground
//! and air). No fused sites. Smoke-tested by tests/tu_ftCo_CaptureMewtwo.rs.

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
        ftCo_ReleaseItemAndVictim(g: G) = crate::tu::ftco_800c7590::ftco_release_item_and_victim;
        ftCo_800DB368(att: &mut Fighter, f: &mut Fighter) = crate::tu::ftCo_CapturePulled::ftCo_800DB368;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_8007E2FC(g: G) = |g| crate::ftcommon2::ftcommon_8007e2fc(super::fp(g));
        ftCo_800BD0E8(g: G, msid: i32) = |g: G, msid: i32| { unsafe { let _ = crate::tu::ftCo_ThrownMewtwo::ftCo_800BD0E8(g as _, msid as _); } };
    }
}

/// The shared body of both entries (`doEnter`).
fn do_enter(g: G, vg: G, msid: i32, victim_msid: i32) {
    let vf = fp(vg);
    let f = fp(g);
    ext::ftCo_ReleaseItemAndVictim(g);
    f.x1A5C = vg;
    f.victim_gobj = vg;
    f.anon_221b.set_x221B_b5(0);
    f.anon_221b.set_x221B_b7(0);
    f.facing_dir = vf.facing_dir;
    ext::ftCo_800DB368(vf, f);
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, fp(g).victim_gobj);
    ext::ftCommon_8007D5D4(fp(g));
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2FC(g);
    ext::ftCo_800BD0E8(g, victim_msid);
}

/// `ftCo_800BCF18`.
pub fn ftCo_800BCF18(g: G, vg: G) {
    do_enter(g, vg, ftCo_MS_CaptureMewtwo as i32, ftCo_MS_ThrownMewtwo as i32);
}

/// `ftCo_800BD000`.
pub fn ftCo_800BD000(g: G, vg: G) {
    do_enter(g, vg, ftCo_MS_CaptureMewtwoAir as i32, ftCo_MS_ThrownMewtwoAir as i32);
}
