#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Lift.c (carrying a heavy item: LiftWait /
//! LiftWalk1-2 / LiftTurn). Every function of the TU; calls out go through `ext`. One fused site:
//! `MTXDegToRad(180 / x230) + ftPartGetRotZ(..)` contracts to an fmadds (melee_core::fp::fmadds).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_Lift.rs.

use crate::types::*;

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftCo_80094EA4(g: G) -> bool = |_| todo!("TODO(port) ftCo_80094EA4 (ftCo_ItemThrow.c)");
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80085088(g: G) = crate::ft084e::ft_80085088;
        Item_8026ABD8(item: *mut Item_GObj, pos: *mut Vec3, f: f32) = |_, _, _| todo!("TODO(port) Item_8026ABD8 (item.c)");
        ftCo_80090780(g: G) = |_| todo!("TODO(port) ftCo_80090780 (ftCo_DamageFall.c)");
        ftCo_Fall_Enter(g: G) = crate::hooks::ftco_fall_enter;
        ft_8008403C(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_8008403c;
        ftAnim_8006EBA4(g: G) = |g| unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g) };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftParts_GetBoneIndex(f: &Fighter, part: i32) -> i32 = crate::ftdmgfall::ftparts_get_bone_index;
        ftPartGetRotZ(f: &Fighter, part: i32) -> f32 = crate::tu::ftparts::ftPartGetRotZ;
        ftPartSetRotY(f: &Fighter, part: i32, v: f32) = crate::tu::ftparts::ftPartSetRotY;
        HSD_JObjSetMtxDirty(j: *mut HSD_JObj) = |_| todo!("TODO(group1 jobj) HSD_JObjSetMtxDirty (jobj.h inline)");
        itIsHeavy(item: *mut Item_GObj) -> bool = |i: *mut Item_GObj| unsafe { melee_items::itlib::it_is_heavy(&*((*i).user_data as *mut Item)) };
    }
}

/// `HSD_JObjSetRotationY` (jobj.h static inline).
pub fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
    let j = unsafe { &mut *jobj };
    assert!(j.flags & JOBJ_USE_QUATERNION as u32 == 0, "jobj.h: !(jobj->flags & JOBJ_USE_QUATERNION)");
    j.rotate.y = y;
    if j.flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        ext::HSD_JObjSetMtxDirty(jobj);
    }
}

/// `ftCo_800974C4` as the C `HSD_GObjEvent` stored in `take_dmg_cb`.
pub unsafe extern "C" fn ftCo_800974C4_cb(g: *mut HSD_GObj) { ftCo_800974C4(g) }

pub fn ftCo_80096D9C(g: G) {
    unsafe { fp(g).mv.co.lift.x0 = 0 };
    ext::Fighter_ChangeMotionState(g, ftCo_MS_LiftWait as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(ftCo_800974C4_cb);
}

pub fn ftCo_LiftWait_Anim(_g: G) {}

pub fn ftCo_LiftWait_IASA(g: G) {
    if ext::ftCo_80094EA4(g) { return; }
    if ftCo_800970E0(g) { return; }
    if ftCo_80096EF8(g) {}
}

pub fn ftCo_LiftWait_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_80096E68(g: G) {
    let it = fp(g).item_gobj;
    if !it.is_null() {
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::Item_8026ABD8(it, &mut v, 0.0);
        ext::ftCo_80090780(g);
    } else {
        ext::ftCo_Fall_Enter(g);
    }
}

pub fn ftCo_LiftWait_Coll(g: G) {
    ext::ft_8008403C(g, ftCo_80096E68);
}

pub fn ftCo_80096EF8(g: G) -> bool {
    let f = fp(g);
    if f.input.lstick[0].x * f.facing_dir >= cd().x228 {
        ftCo_80096F48(g);
        return true;
    }
    false
}

pub fn ftCo_80096F48(g: G) {
    let f = fp(g);
    let msid = if unsafe { f.mv.co.lift.x0 } == 0 { ftCo_MS_LiftWalk1 } else { ftCo_MS_LiftWalk2 };
    unsafe { f.mv.co.lift.x0 ^= 1 };
    ext::Fighter_ChangeMotionState(g, msid as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    fp(g).take_dmg_cb = Some(ftCo_800974C4_cb);
}

pub fn ftCo_LiftWalk_Anim(g: G) {
    if !ext::ftAnim_IsFramesRemaining(g) && !ftCo_80096EF8(g) {
        ext::Fighter_ChangeMotionState(g, ftCo_MS_LiftWait as i32, Ft_MF_Unk24 as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        fp(g).take_dmg_cb = Some(ftCo_800974C4_cb);
    }
}

pub fn ftCo_LiftWalk_IASA(g: G) {
    if ext::ftCo_80094EA4(g) {}
}

pub fn ftCo_LiftWalk_Phys(g: G) {
    ext::ft_80085088(g);
}

pub fn ftCo_LiftWalk_Coll(g: G) {
    ext::ft_8008403C(g, ftCo_80096E68);
}

pub fn ftCo_800970E0(g: G) -> bool {
    let f = fp(g);
    if f.input.lstick[0].x * f.facing_dir <= cd().x22C {
        ftCo_80097130(g);
        return true;
    }
    false
}

pub fn ftCo_80097130(g: G) {
    let f = fp(g);
    unsafe {
        f.mv.co.lift.x4 = cd().x230;
        f.mv.co.lift.x8 = 0;
    }
    ext::Fighter_ChangeMotionState(g, ftCo_MS_LiftTurn as i32, Ft_MF_Unk24 as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    fp(g).take_dmg_cb = Some(ftCo_800974C4_cb);
    ftCo_LiftTurn_Anim(g);
}

pub fn ftCo_LiftTurn_Anim(g: G) {
    let part = ext::ftParts_GetBoneIndex(fp(g), FtPart_TransN as i32);
    let jobj = unsafe { (*fp(g).parts.offset(part as isize)).x4_jobj2 };
    unsafe { fp(g).mv.co.lift.x4 -= 1.0 };
    if fp(g).x2222_b6() == 0 {
        let deg = 180.0f32 / cd().x230;
        let rz = ext::ftPartGetRotZ(fp(g), part);
        let angle = melee_core::fp::fmadds(deg, 0.01745329252f32, rz);
        ext::ftPartSetRotY(fp(g), part, angle);
        HSD_JObjSetRotationY(jobj, angle);
    }
    if unsafe { fp(g).mv.co.lift.x8 } == 0 && unsafe { fp(g).mv.co.lift.x4 } <= 0.5f32 * cd().x230 {
        let r = ext::ftPartGetRotZ(fp(g), part);
        let f = fp(g);
        f.facing_dir = -f.facing_dir;
        let top = (core::f64::consts::FRAC_PI_2 * f.facing_dir as f64) as f32;
        ext::ftPartSetRotY(fp(g), FtPart_TopN as i32, top);
        ext::ftPartSetRotY(fp(g), part, -r);
        HSD_JObjSetRotationY(jobj, -r);
        let f = fp(g);
        f.gr_vel = -f.gr_vel;
        unsafe { f.mv.co.lift.x8 = 1 };
    }
    if unsafe { fp(g).mv.co.lift.x4 } <= 0.0 {
        ext::Fighter_ChangeMotionState(g, 174, Ft_MF_Unk24 as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        fp(g).take_dmg_cb = Some(ftCo_800974C4_cb);
    }
}

pub fn ftCo_LiftTurn_IASA(g: G) {
    if ext::ftCo_80094EA4(g) {}
}

pub fn ftCo_LiftTurn_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_LiftTurn_Coll(g: G) {
    ext::ft_8008403C(g, ftCo_80096E68);
}

pub fn ftCo_800974C4(g: G) {
    let it = fp(g).item_gobj;
    if !it.is_null() {
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::Item_8026ABD8(it, &mut v, 0.0);
    }
}

pub fn ftCo_8009750C(g: G) {
    let it = fp(g).item_gobj;
    if !it.is_null() && ext::itIsHeavy(it) {
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::Item_8026ABD8(it, &mut v, 0.0);
    }
}
