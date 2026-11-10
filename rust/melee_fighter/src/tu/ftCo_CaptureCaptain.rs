#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CaptureCaptain.c (caught by the Falcon/Ganon
//! side special). Every function of the TU; calls out of it go through `ext` (unported ones
//! panic until rewired). `ftCo_ReleaseItemAndVictim` (ftCommon/inlines.h) comes from
//! `tu::ftCo_Barrel`. Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ftCo_CaptureCaptain.rs.

use crate::types::*;
use super::ftCo_Barrel::ftCo_ReleaseItemAndVictim;

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
        // rp3 ftCo_800DB368
        ftCo_800DB368(vic: *mut Fighter, fp: *mut Fighter) = |vic: *mut Fighter, fp: *mut Fighter| unsafe { crate::tu::ftCo_CapturePulled::ftCo_800DB368(&mut *(vic as *mut _), &mut *(fp as *mut _)); };
        // rp3-end
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_800DB464_fn() -> HSD_GObjEvent = || todo!("TODO(port) &ftCo_800DB464 (ftCo_Attack100.c)");
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_8007E2FC(g: G) = |a0| crate::tu::ftcommon::ftcommon_8007e2fc(unsafe { &mut *((*(a0 as *mut HSD_GObj)).user_data as *mut Fighter) });
        ft_80083B68(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80083b68(g as _); } };
    }
}

pub fn ftCo_8009CA0C(g: G, vic: G) {
    melee_core::ct!("ftCo_8009CA0C");
    let vf = unsafe { (*vic).user_data as *mut Fighter };
    ftCo_ReleaseItemAndVictim(g);
    let f = fp(g);
    f.x1A5C = vic;
    f.victim_gobj = vic;
    f.x221B.set_x221B_b5(0);
    f.facing_dir = -unsafe { (*vf).facing_dir };
    if f.ground_or_air as i32 == GA_Air as i32 {
        ext::ftCo_800DB368(vf, f);
        fp(g).x221B.set_x221B_b7(0);
    } else {
        f.x221B.set_x221B_b7(1);
    }
    let other = fp(g).victim_gobj;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureCaptain as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, other);
    if fp(g).x221B.x221B_b7() == 0 {
        fp(g).accessory1_cb = ext::ftCo_800DB464_fn();
    }
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2FC(g);
}

pub fn ftCo_CaptureCaptain_Anim(_g: G) { melee_core::ct!("ftCo_CaptureCaptain_Anim");}

pub fn ftCo_CaptureCaptain_IASA(_g: G) { melee_core::ct!("ftCo_CaptureCaptain_IASA");}

pub fn ftCo_CaptureCaptain_Phys(_g: G) { melee_core::ct!("ftCo_CaptureCaptain_Phys");}

pub fn ftCo_CaptureCaptain_Coll(g: G) {
    melee_core::ct!("ftCo_CaptureCaptain_Coll");
    if fp(g).x2226_b2() == 0 {
        ext::ft_80083B68(g);
    }
}
