#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CaptureYoshi.c. Every function of the TU; calls out of it go
//! through `ext` (unported ones panic until rewired). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_CaptureYoshi.rs.
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
        ftCommon_8007DB58(g: G) = crate::ftcommon3::ftcommon_8007db58;
        ftCo_8009750C(g: G) = |g: G| { unsafe { crate::tu::ftCo_Lift::ftCo_8009750C(g as _); } };
        ftCo_800DD168(g: G) = |g: G| { unsafe { crate::tu::ftCo_Throw::ftCo_800DD168(g as _); } };
        // rp3 ftCo_800DB368
        ftCo_800DB368(victim: *mut Fighter, fp: *mut Fighter) = |victim: *mut Fighter, fp: *mut Fighter| unsafe { crate::tu::ftCo_CapturePulled::ftCo_800DB368(&mut *(victim as *mut _), &mut *(fp as *mut _)); };
        // rp3-end
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_800DB464(g: G) = |g: G| { unsafe { crate::tu::ftCo_CapturePulled::ftCo_800DB464(g as _); } };
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_8007E2FC(g: G) = |a0| crate::ftcommon2::ftcommon_8007e2fc(unsafe { &mut *((*(a0 as *mut HSD_GObj)).user_data as *mut Fighter) });
    }
}

/// `ftCo_800DB464` as the `accessory1_cb` value.
pub unsafe extern "C" fn ftCo_800DB464_cb(g: G) {
    melee_core::ct!("ftCo_800DB464_cb");
    ext::ftCo_800DB464(g)
}

/// `ftCo_ReleaseItemAndVictim` (ftCommon/inlines.h).
fn release_item_and_victim(g: G) {
    ext::ftCommon_8007DB58(g);
    ext::ftCo_8009750C(g);
    ext::ftCo_800DD168(g);
}

pub fn ftCo_800BBB8C(g: G, attacker_gobj: G) {
    melee_core::ct!("ftCo_800BBB8C");
    let attacker_fp = unsafe { (*attacker_gobj).user_data as *mut Fighter };
    release_item_and_victim(g);
    let f = fp(g);
    f.x1A5C = attacker_gobj;
    f.victim_gobj = attacker_gobj;
    f.anon_221b.set_x221B_b5(0);
    f.anon_221b.set_x221B_b7(0);
    f.facing_dir = -unsafe { (*attacker_fp).facing_dir };
    ext::ftCo_800DB368(attacker_fp, f as *mut Fighter);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureYoshi as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    fp(g).accessory1_cb = Some(ftCo_800DB464_cb);
    ext::ftCommon_8007D5D4(fp(g));
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2FC(g);
}

pub fn ftCo_CaptureYoshi_Anim(_g: G) { melee_core::ct!("ftCo_CaptureYoshi_Anim");}

pub fn ftCo_CaptureYoshi_IASA(_g: G) { melee_core::ct!("ftCo_CaptureYoshi_IASA");}

pub fn ftCo_CaptureYoshi_Phys(_g: G) { melee_core::ct!("ftCo_CaptureYoshi_Phys");}

pub fn ftCo_CaptureYoshi_Coll(_g: G) { melee_core::ct!("ftCo_CaptureYoshi_Coll");}
