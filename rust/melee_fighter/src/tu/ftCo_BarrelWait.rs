#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_BarrelWait.c (inside a stage barrel). Every
//! function of the TU; calls out of it go through `ext` (unported ones panic until rewired).
//! The ftCommon/inlines.h helpers come from `tu::ftCo_Barrel` (their callees hook there).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_BarrelWait.rs.

use crate::types::*;
use super::ftCo_Barrel::{ftCo_Barrel_ApplyKnockback, ftCo_ReleaseItemAndVictim};

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCo_8009EAF8_fn() -> HSD_GObjEvent = || todo!("TODO(port) &ftCo_8009EAF8 (ftdynamics.c)");
        ftCommon_8007E2FC(g: G) = |g| crate::ftcommon2::ftcommon_8007e2fc(super::fp(g));
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCommon_8007EFC0(fp: &mut Fighter, v: u32) = crate::ftcommon2::ftcommon_8007efc0;
        Ground_801C4DD0() -> bool = || todo!("TODO(port) Ground_801C4DD0 (ground.c)");
        Ground_801C4DA0(pos: *mut Vec3, out: *mut f32) -> bool = |_, _| todo!("TODO(port) Ground_801C4DA0 (ground.c)");
        mpColl_80043680(c: *mut CollData, v: *mut Vec3) = |_, _| todo!("TODO(port) mpColl_80043680 (mpcoll.c)");
        ftCamera_80076064(fp: *mut Fighter) = |_| todo!("TODO(port) ftCamera_80076064 (ftcamera.c)");
        lbColl_80008D30(hit: *mut HitCapsule, a: *mut lbColl_80008D30_arg1) = |hit: *mut HitCapsule, a: *mut lbColl_80008D30_arg1| { unsafe { let _ = melee_runtime::tu::lbcollision::lbColl_80008D30(hit as _, a as _); } };
        ftColl_800788D4(g: G) = |_| todo!("TODO(port) ftColl_800788D4 (ftcoll.c)");
        Fighter_UnkTakeDamage_8006CC30(fp: *mut Fighter, dmg: f32) = |_, _| todo!("TODO(port) Fighter_UnkTakeDamage_8006CC30 (fighter.c)");
        ftCo_8008DCE0(g: G, a: i32, facing: f32) = |g: G, a: i32, facing: f32| { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_8008DCE0(g as _, a as _, facing as _); } };
    }
}

/// static inline `setCamData`.
fn setCamData(c: &ftCommonData, cam: *mut CmSubject) {
    unsafe {
        (*cam).target_ext.h = c.x4DC;
        (*cam).target_ext.v = c.x4E4;
    }
}

pub fn ftCo_8009EB18(g: G) {
    ftCo_ReleaseItemAndVictim(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_BarrelWait as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    fp(g).death2_cb = ext::ftCo_8009EAF8_fn();
    fp(g).accessory1_cb = Some(ftCo_8009EC44);
    ext::ftCommon_8007E2FC(g);
    let f = fp(g);
    f.x1988 = 2;
    f.set_invisible(1);
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    setCamData(cd(), fp(g).x890_cameraBox);
    ext::ftCommon_8007EFC0(fp(g), 1);
}

pub fn ftCo_BarrelWait_Anim(_g: G) {}

pub fn ftCo_BarrelWait_IASA(g: G) {
    if fp(g).input.pressed_buttons & (HSD_PAD_A | HSD_PAD_B) as u32 != 0 {
        ext::Ground_801C4DD0();
    }
}

pub fn ftCo_BarrelWait_Phys(_g: G) {}

pub fn ftCo_BarrelWait_Coll(_g: G) {}

/// static `ftCo_8009EC44` (installed as `accessory1_cb`).
pub unsafe extern "C" fn ftCo_8009EC44(g: G) {
    let mut param: f32 = 0.0;
    ext::Ground_801C4DA0(&mut fp(g).cur_pos, &mut param);
}

pub fn ftCo_8009EC70(g: G, pos: *mut Vec3, arg2: *mut lbColl_80008D30_arg1, kb_angle: f32) {
    let f = fp(g);
    f.bury_timer_2 = cd().bury_timer_unk2;
    f.cur_pos = unsafe { *pos };
    ext::mpColl_80043680(&mut fp(g).coll_data, pos);
    ext::ftCamera_80076064(fp(g));
    let mut hit = SmallerHitCapsule::zeroed();
    let hp = &mut hit as *mut SmallerHitCapsule as *mut HitCapsule;
    ext::lbColl_80008D30(hp, arg2);
    ext::ftColl_800788D4(g);
    ftCo_Barrel_ApplyKnockback(fp(g), kb_angle, hp);
    ext::Fighter_UnkTakeDamage_8006CC30(fp(g), hit.damage);
    ext::ftCo_8008DCE0(g, 91, 0.0);
}
