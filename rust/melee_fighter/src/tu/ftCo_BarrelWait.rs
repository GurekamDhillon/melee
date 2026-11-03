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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCo_8009EAF8_fn() -> HSD_GObjEvent = || todo!("TODO(port) &ftCo_8009EAF8 (ftdynamics.c)");
        ftCommon_8007E2FC(g: G) = |a0| crate::ftcommon2::ftcommon_8007e2fc(unsafe { &mut *((*(a0 as *mut HSD_GObj)).user_data as *mut Fighter) });
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCommon_8007EFC0(fp: &mut Fighter, v: u32) = crate::ftcommon2::ftcommon_8007efc0;
        Ground_801C4DD0() -> bool = || unsafe { melee_world::tu::ground::Ground_801C4DD0() != 0 };
        Ground_801C4DA0(pos: *mut Vec3, out: *mut f32) -> bool = |a0, a1| unsafe { melee_world::tu::ground::Ground_801C4DA0(a0, a1) != 0 };
        // rp3 mpColl_80043680
        mpColl_80043680(c: *mut CollData, v: *mut Vec3) = |c: *mut CollData, v: *mut Vec3| unsafe { melee_world::tu::mpcoll::mpColl_80043680(c, v as _); };
        // rp3-end
        // rp3 ftCamera_80076064
        ftCamera_80076064(fp: *mut Fighter) = |fp: *mut Fighter| unsafe { crate::tu::ftcamera::ftcamera_80076064(&mut *(fp as *mut _)); };
        // rp3-end
        lbColl_80008D30(hit: *mut HitCapsule, a: *mut lbColl_80008D30_arg1) = |hit: *mut HitCapsule, a: *mut lbColl_80008D30_arg1| { unsafe { melee_runtime::tu::lbcollision::lbColl_80008D30(hit as _, a as _); } };
        ftColl_800788D4(g: G) = |g: G| { unsafe { crate::tu::ftcoll::ftcoll_800788d4(g as _); } };
        // rp3 Fighter_UnkTakeDamage_8006CC30
        Fighter_UnkTakeDamage_8006CC30(fp: *mut Fighter, dmg: f32) = |fp: *mut Fighter, dmg: f32| unsafe { crate::tu::fighter::Fighter_UnkTakeDamage_8006CC30(&mut *(fp as *mut _), dmg); };
        // rp3-end
        ftCo_8008DCE0(g: G, a: i32, facing: f32) = |g: G, a: i32, facing: f32| { unsafe { crate::tu::ftCo_Damage::ftCo_8008DCE0(g as _, a as _, facing as _); } };
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
    melee_core::ct!("ftCo_8009EB18");
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

pub fn ftCo_BarrelWait_Anim(_g: G) { melee_core::ct!("ftCo_BarrelWait_Anim");}

pub fn ftCo_BarrelWait_IASA(g: G) {
    melee_core::ct!("ftCo_BarrelWait_IASA");
    if fp(g).input.pressed_buttons & (HSD_PAD_A | HSD_PAD_B) as u32 != 0 {
        ext::Ground_801C4DD0();
    }
}

pub fn ftCo_BarrelWait_Phys(_g: G) { melee_core::ct!("ftCo_BarrelWait_Phys");}

pub fn ftCo_BarrelWait_Coll(_g: G) { melee_core::ct!("ftCo_BarrelWait_Coll");}

/// static `ftCo_8009EC44` (installed as `accessory1_cb`).
pub unsafe extern "C" fn ftCo_8009EC44(g: G) {
    melee_core::ct!("ftCo_8009EC44");
    let mut param: f32 = 0.0;
    ext::Ground_801C4DA0(&mut fp(g).cur_pos, &mut param);
}

pub fn ftCo_8009EC70(g: G, pos: *mut Vec3, arg2: *mut lbColl_80008D30_arg1, kb_angle: f32) {
    melee_core::ct!("ftCo_8009EC70");
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
