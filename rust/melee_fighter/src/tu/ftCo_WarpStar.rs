#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_WarpStar.c (every function, including the
//! static damage/landing callbacks). No fused float sites. Calls out of the TU go through `ext`
//! (unported ones panic until rewired). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ftCo_WarpStar.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn jobj(g: G) -> *mut HSD_JObj {
    unsafe { (*g).hsd_obj as *mut HSD_JObj }
}
#[inline(always)]
fn ws(f: &mut Fighter) -> &mut ftCommon_MotionVars_warpstar {
    unsafe { &mut f.mv.co.warpstar }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
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
        it_80294430(item: *mut Item_GObj, a: f32, b: f32) = |_, _, _| todo!("TODO(port) it_80294430 (itwstar.c)");
        it_80294364(item: *mut Item_GObj) -> *mut HSD_AnimJoint = |_| todo!("TODO(port) it_80294364 (itwstar.c)");
        it_802944AC(item: *mut Item_GObj, ecb: *mut ftCollisionBox) = |_, _| todo!("TODO(port) it_802944AC (itwstar.c)");
        it_802946B0(item: *mut Item_GObj) = |_| todo!("TODO(port) it_802946B0 (itwstar.c)");
        it_802947CC(item: *mut Item_GObj, pos: *mut Vec3) = |_, _| todo!("TODO(port) it_802947CC (itwstar.c)");
        Item_8026ABD8(item: *mut Item_GObj, pos: *mut Vec3, v: f32) = |_, _, _| todo!("TODO(port) Item_8026ABD8 (item.c)");
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftColl_8007B6A0(g: G) = |_| todo!("TODO(port) ftColl_8007B6A0 (ftcoll.c)");
        ftColl_8007B6EC(g: G) = |_| todo!("TODO(port) ftColl_8007B6EC (ftcoll.c)");
        ftCommon_8007E2FC(g: G) = |g| crate::ftcommon2::ftcommon_8007e2fc(super::fp(g));
        HSD_JObjAddAnim(j: *mut HSD_JObj, an: *mut HSD_AnimJoint, mat: *mut HSD_MatAnimJoint, sh: *mut HSD_ShapeAnimJoint) = |_, _, _, _| todo!("TODO(port) HSD_JObjAddAnim (jobj.c)");
        ftAnim_80070734(j: *mut HSD_JObj, frame: f32) = |_, _| todo!("TODO(port) ftAnim_80070734 (ftanim.c)");
        HSD_JObjAnim(j: *mut HSD_JObj) = |_| todo!("TODO(port) HSD_JObjAnim (jobj.c)");
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |_| todo!("TODO(port) HSD_JObjSetMtxDirtySub (jobj.c)");
        Stage_GetBlastZoneTopOffset() -> f32 = || todo!("TODO(port) Stage_GetBlastZoneTopOffset (stage.c)");
        ftCamera_800762F4(g: G) = |_| todo!("TODO(port) ftCamera_800762F4 (ftcamera.c)");
        mpColl_80043680(c: *mut CollData, pos: *mut Vec3) = |_, _| todo!("TODO(port) mpColl_80043680 (mpcoll.c)");
        ftCommon_Fall(f: &mut Fighter, a: f32, b: f32) = |_, _, _| todo!("TODO(port) ftCommon_Fall (ftcommon.c)");
        ftCommon_CalcSelfAccel_AccelToVelClampedFrom(f: &mut Fighter, vel: f32, accel: f32, target: f32, fr: f32) = crate::ftcommon::calc_self_accel_accel_to_vel_clamped_from;
        ft_80083E64(g: G, ecb: *mut ftCollisionBox, cb: HSD_GObjEvent) = |_, _, _| todo!("TODO(port) ft_80083E64 (ft_081B.c)");
        atan2f(y: f32, x: f32) -> f32 = melee_runtime::lbtrigf::atan2f;
        // `efAsync_Spawn(gobj, queue, kind, gfx_id, jobj, ...)` with (pos, param) as the varargs.
        efAsync_Spawn(g: G, queue: *mut core::ffi::c_void, kind: u32, id: u32, j: *mut HSD_JObj, pos: *mut Vec3, param: *mut f32) = |_, _, _, _, _, _, _| todo!("TODO(port) efAsync_Spawn (efasync.c)");
        Camera_RequestQuake(kind: i32, pos: *mut Vec3) = |_, _| todo!("TODO(port) Camera_RequestQuake (camera.c)");
        ftCommon_8007EBAC(f: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
    }
}

pub unsafe extern "C" fn ftCo_800C4858_cb(g: G) {
    ftCo_800C4858(g)
}
pub unsafe extern "C" fn ftCo_800C48AC_cb(g: G) {
    ftCo_800C48AC(g)
}
pub unsafe extern "C" fn ftCo_800C4C60_cb(g: G) {
    ftCo_800C4C60(g)
}

/// static inline `Fighter_SetDamageCallbacks` (ft/inlines.h).
fn set_damage_callbacks(f: &mut Fighter) {
    f.take_dmg_cb = Some(ftCo_800C4858_cb);
    f.death2_cb = Some(ftCo_800C48AC_cb);
}

pub fn ftCo_800C4724(g: G) {
    let j = jobj(g);
    let f = fp(g);
    ext::it_80294430(f.item_gobj, f.x2C4.y, f.co_attrs.warp_star_hitbox_scale);
    let f = fp(g);
    let facing = f.facing_dir;
    ws(f).facing_dir = facing;
    f.facing_dir = 0.0;
    ws(f).x1C = 120;
    let cp = f.cur_pos;
    ws(f).cur_pos = cp;
    ws(f).self_vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::ftCommon_8007D5D4(f);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_WarpStarJump as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftColl_8007B6A0(g);
    let f = fp(g);
    set_damage_callbacks(f);
    f.set_x2219_b2(1);
    f.set_x2222_b7(1);
    f.set_x221E_b2(1);
    ext::ftCommon_8007E2FC(g);
    let an = ext::it_80294364(fp(g).item_gobj);
    ext::HSD_JObjAddAnim(j, an, core::ptr::null_mut(), core::ptr::null_mut());
    ext::ftAnim_80070734(j, 0.0);
    ext::HSD_JObjAnim(j);
}

/// static inline `inlineA0`: restore facing and drop the star.
fn inline_a0(g: G) {
    ext::ftColl_8007B6EC(g);
    let f = fp(g);
    f.facing_dir = ws(f).facing_dir;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::Item_8026ABD8(f.item_gobj, &mut pos, 1.0);
}

pub fn ftCo_800C4858(g: G) {
    inline_a0(g);
}

pub fn ftCo_800C48AC(g: G) {
    inline_a0(g);
}

pub fn ftCo_WarpStarJump_Anim(g: G) {
    let f = fp(g);
    ws(f).x1C = ws(f).x1C.wrapping_sub(1);
    let y = f.cur_pos.y;
    if y > ext::Stage_GetBlastZoneTopOffset() || ws(fp(g)).x1C == 0 {
        ftCo_800C4A38(g);
    }
}

pub fn ftCo_WarpStarJump_IASA(_g: G) {}

pub fn ftCo_WarpStarJump_Phys(g: G) {
    let j = jobj(g);
    assert!(!j.is_null(), "jobj.h:979 jobj");
    let t = unsafe { (*j).translate };
    let f = fp(g);
    let prev = ws(f).self_vel;
    f.self_vel.x = t.x - prev.x;
    f.self_vel.y = t.y - prev.y;
    f.self_vel.z = t.z - prev.z;
    ws(f).self_vel = t;
}

pub fn ftCo_WarpStarJump_Coll(_g: G) {}

pub fn ftCo_WarpStarJump_Cam(g: G) {
    ext::ftCamera_800762F4(g);
}

pub fn ftCo_800C4A38(g: G) {
    let f = fp(g);
    let ecb = &mut ws(f).ecb as *mut ftCollisionBox;
    ext::it_802944AC(f.item_gobj, ecb);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_WarpStarFall as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_x2222_b7(1);
    set_damage_callbacks(f);
    f.set_x221E_b2(1);
    ext::ftCommon_8007E2FC(g);
    let f = fp(g);
    f.cur_pos.x = ws(f).cur_pos.x;
    f.cur_pos.y = ext::Stage_GetBlastZoneTopOffset();
    f.cur_pos.z = 0.0;
    let rot = Quaternion { x: core::f64::consts::FRAC_PI_2 as f32, y: 0.0, z: 0.0, w: 0.0 };
    // HSD_JObjSetRotation (jobj.h inline).
    let j = jobj(g);
    assert!(!j.is_null(), "jobj.h:618 jobj");
    let jr = unsafe { &mut *j };
    jr.rotate = rot;
    if jr.flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !melee_types::inline::HSD_JObjMtxIsDirty(jr) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
    let f = fp(g);
    ext::it_802946B0(f.item_gobj);
    let f = fp(g);
    ext::mpColl_80043680(&mut f.coll_data, &mut f.cur_pos);
}

pub fn ftCo_WarpStarFall_Anim(_g: G) {}
pub fn ftCo_WarpStarFall_IASA(_g: G) {}

pub fn ftCo_WarpStarFall_Phys(g: G) {
    let c = cd();
    ext::ftCommon_Fall(fp(g), c.x694, c.x698);
    let f = fp(g);
    let lsx = f.input.lstick[0].x;
    let accel_scaling = lsx * c.warpstarfall_drift_scaling;
    let accel_flat = if lsx > 0.0 { c.warpstarfall_drift_flat } else { -c.warpstarfall_drift_flat };
    let accel = accel_scaling + accel_flat;
    let target_vel = lsx * c.warpstarfall_drift_max;
    let v = f.self_vel.x;
    ext::ftCommon_CalcSelfAccel_AccelToVelClampedFrom(f, v, accel, target_vel, c.x6A8);
}

pub fn ftCo_WarpStarFall_Coll(g: G) {
    let ecb = &mut ws(fp(g)).ecb as *mut ftCollisionBox;
    ext::ft_80083E64(g, ecb, Some(ftCo_800C4C60_cb));
}

pub fn ftCo_800C4C60(g: G) {
    let f = fp(g);
    f.facing_dir = ws(f).facing_dir;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_JumpB as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftColl_8007B6EC(g);
    let f = fp(g);
    let c = cd();
    let jm;
    unsafe {
        f.mv.co.jump.x0 = 0;
        f.mv.co.jump.x4 = 0;
        f.mv.co.jump.jump_mul = 1.0;
        jm = f.mv.co.jump.jump_mul;
    }
    f.self_vel.x *= f.co_attrs.ground_to_air_jump_momentum_multiplier * jm;
    f.self_vel.y *= c.x438;
    f.self_vel.z = 0.0;
    {
        let h_init_v = -f.facing_dir * c.x78 * f.co_attrs.jump_h_initial_velocity;
        let mut h_vel = jm * h_init_v;
        let v_init_v = f.co_attrs.jump_v_initial_velocity * jm;
        f.self_vel.y = v_init_v;
        h_vel = f.self_vel.x + h_vel;
        let h_max_vel = f.co_attrs.jump_h_max_velocity * jm;
        let abs = if h_vel < 0.0 { -h_vel } else { h_vel };
        if abs > h_max_vel {
            h_vel = if h_vel < 0.0 { -h_max_vel } else { h_max_vel };
        }
        f.self_vel.x = h_vel;
    }
    ext::it_802947CC(f.item_gobj, &mut f.cur_pos);
    let f = fp(g);
    let mut vec = Vec3 { x: 0.0, y: f.coll_data.ecb.bottom.y, z: 0.0 };
    vec.x += f.cur_pos.x;
    vec.y += f.cur_pos.y;
    vec.z += f.cur_pos.z;
    let mut param = ext::atan2f(-f.coll_data.floor.normal.x, f.coll_data.floor.normal.y);
    let q = &mut fp(g).x60C as *mut *mut core::ffi::c_void as *mut core::ffi::c_void;
    ext::efAsync_Spawn(g, q, 5, 1030, core::ptr::null_mut(), &mut vec, &mut param);
    ext::Camera_RequestQuake(QuakeKind_Large as i32, &mut vec);
    ext::ftCommon_8007EBAC(fp(g), 14, 0);
}

pub fn ftCo_WarpStarFall_Cam(g: G) {
    ext::ftCamera_800762F4(g);
}
