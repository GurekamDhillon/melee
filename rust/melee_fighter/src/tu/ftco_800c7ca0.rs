//! src/melee/ft/ftCo_800C7CA0.c: DownReflect (downed fighter bouncing off a wall) and the
//! metal-material helpers ftCo_800C8064 / ft_800C80A4.
//! Fused sites (tools/rust-port/fma_sites.py): fn_800C7DC4, one f32 fmuladd:
//! `x68C.z * -facing - (cur.x + off.x)` -> fmsubs.
#![allow(non_upper_case_globals, static_mut_refs, non_snake_case)]

use crate::hooks as h;
use crate::types::*;
use melee_core::fp::fmsubs;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Callees in TUs not ported yet. Each is a settable pointer; unset panics (TODO rewire).
pub mod ext {
    use super::G;
    use crate::types::*;
    macro_rules! stubs {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO stub: ", stringify!($name))),
                }
            }
        )*};
    }
    stubs! {
        FTKB_SPECIALN_800F1F1C ftKb_SpecialN_800F1F1C(g: G, off: &mut Vec3);
        EFASYNC_SPAWN efAsync_Spawn(g: G, x60c: *mut *mut core::ffi::c_void, a: i32, id: i32, jobj: *mut HSD_JObj, pos: &Vec3, ang: &f32);
        CAMERA_REQUEST_QUAKE Camera_RequestQuake(k: CmQuakeKind, pos: &Vec3);
        FIGHTER_CHANGE_MOTION_STATE Fighter_ChangeMotionState(g: G, ms: i32, flags: u32, a: f32, b: f32, c: f32, j: *mut HSD_JObj);
        FTCOLL_8007B760 ftColl_8007B760(g: G, v: i32);
        FTCO_80090780 ftCo_80090780(g: G);
        FTCO_80097D88 ftCo_80097D88(g: G);
        FT_HSD_JOBJ_LOAD_JOINT HSD_JObjLoadJoint(j: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
        FT_HSD_JOBJ_GET_DOBJ HSD_JObjGetDObj(j: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
        FT_800C80A4_BONE ft_800C80A4_bone(jobj: *mut HSD_JObj);
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $d:expr;)*) => {
            pub mod hook {
                use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(unreachable_code, unused_variables)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => $d,
                }
            } )*
        };
    }
    ext_fns! {
        fn atan2f(_a0: f32, _a1: f32) -> f32 = { unsafe { (melee_runtime::lbtrigf::atan2f(_a0 as _, _a1 as _)) as _ } };
        fn ftCo_80090574(_a0: *mut core::ffi::c_void) = { unsafe { let _ = crate::ftdmgfall::ftco_80090574(_a0 as _); } };
        fn ftCommon_8007D5D4(_a0: *mut core::ffi::c_void) = { unsafe { let _ = crate::ftcommon2::ftcommon_8007d5d4(&mut *(_a0 as *mut Fighter)); } };
        fn ftCommon_8007EBAC(_a0: *mut core::ffi::c_void, _a1: u32, _a2: u32) = { unsafe { let _ = crate::ftcommon2::ftcommon_8007ebac(&mut *(_a0 as *mut Fighter), _a1 as _, _a2 as _); } };
    }
}

/// `ft_804D6580`: metal material loaded by ftCo_800C8064.
pub static mut ft_804D6580: *mut HSD_MObj = core::ptr::null_mut();

/// `ftCo_800C7CA0`: enter DownReflect when knocked into a wall fast enough.
pub fn ftco_800c7ca0(g: G) -> bool {
    melee_core::ct!("ftco_800c7ca0");
    let f = fp(g);
    let threshold = h::common_data().x1B0;
    let vel_x = f.x8c_kb_vel.x;
    let flags = f.coll_data.env_flags;
    let dr = unsafe { f.mv.co.downreflect.x4 };
    if vel_x < -threshold && (flags & Collide_RightWallHug) != 0 && dr != 1 {
        let mut off = Vec3 { x: f.coll_data.ecb.left.x, y: f.coll_data.ecb.left.y, z: 0.0 };
        ext::ftKb_SpecialN_800F1F1C(g, &mut off);
        let n = fp(g).coll_data.right_facing_wall.normal;
        fn_800c7dc4(g, ftCo_MS_DownReflect as i32, &n, &off);
        fp(g).mv.co.downreflect.x4 = 1;
        return true;
    }
    if vel_x > threshold && (flags & Collide_LeftWallHug) != 0 && dr != 2 {
        let mut off = Vec3 { x: f.coll_data.ecb.right.x, y: f.coll_data.ecb.right.y, z: 0.0 };
        ext::ftKb_SpecialN_800F1F1C(g, &mut off);
        let n = fp(g).coll_data.left_facing_wall.normal;
        fn_800c7dc4(g, ftCo_MS_DownReflect as i32, &n, &off);
        fp(g).mv.co.downreflect.x4 = 2;
        return true;
    }
    false
}

/// `fn_800C7DC4`: bounce: effect, quake, reflected knockback, motion change, position snap.
/// `normal` is passed by value-copy (C passes `&coll->..._wall.normal`; no callee before the
/// read writes it).
pub fn fn_800c7dc4(g: G, motion_state: i32, normal: &Vec3, offset: &Vec3) {
    melee_core::ct!("fn_800c7dc4");
    let f = fp(g);
    ext::ftCommon_8007D5D4(crate::mtu_ptr::ptr(&(f)));
    let spawn = Vec3 {
        x: f.cur_pos.x + offset.x,
        y: f.cur_pos.y + offset.y,
        z: f.cur_pos.z + offset.z,
    };
    let angle = (ext::atan2f((-normal.x) as f32, (normal.y) as f32) as ::core::ffi::c_float);
    ext::efAsync_Spawn(g, &mut fp(g).x60C, 5, 0x406, core::ptr::null_mut(), &spawn, &angle);
    ext::Camera_RequestQuake(QuakeKind_Small, &spawn);
    let f = fp(g);
    let mag = f.xF0_ground_kb_vel;
    let mut vel = *normal;
    vel.x *= mag;
    vel.y *= mag;
    vel.z *= mag;
    let mul = h::common_data().x1BC;
    vel.x *= mul;
    vel.y *= mul;
    f.x8c_kb_vel = vel;
    f.self_vel.z = 0.0;
    f.self_vel.y = 0.0;
    f.self_vel.x = 0.0;
    f.facing_dir = if f.x8c_kb_vel.x < 0.0 { -1.0 } else { 1.0 };
    ext::Fighter_ChangeMotionState(g, motion_state, 0x18040, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    if f.coll_data.env_flags & Collide_RightWallHug != 0 {
        f.cur_pos.x = -fmsubs(f.x68C_transNPos.z, -f.facing_dir, f.cur_pos.x + offset.x);
    } else {
        f.cur_pos.y = f.x68C_transNPos.y + (f.cur_pos.y + offset.y);
    }
    ext::ftCo_80090574(crate::mtu_ptr::ptr(&(g)));
    fp(g).dmg.x18A8 = mag;
    ext::ftCommon_8007EBAC(crate::mtu_ptr::ptr(&(fp(g))), (7) as u32, (0) as u32);
    ext::ftColl_8007B760(g, h::common_data().x1B8);
}

/// `ftCo_DownReflect_Anim`.
pub fn ftco_downreflect_anim(g: G) {
    melee_core::ct!("ftco_downreflect_anim");
    if !h::ftanim_is_frames_remaining(g) {
        ext::ftCo_80090780(g);
    }
}

/// `ftCo_DownReflect_IASA`: empty.
pub fn ftco_downreflect_iasa(_g: G) { melee_core::ct!("ftco_downreflect_iasa");}

/// `ftCo_DownReflect_Phys`.
pub fn ftco_downreflect_phys(g: G) {
    melee_core::ct!("ftco_downreflect_phys");
    h::ft_80084db0(g);
}

/// `ftCo_DownReflect_Coll`. C: `if (ft_80081DD4(gobj))`; the hook returns unit, so the
/// branch result comes from the stubbed predicate.
pub fn ftco_downreflect_coll(g: G) {
    melee_core::ct!("ftco_downreflect_coll");
    if ext_ft_80081dd4(g) {
        ext::ftCo_80097D88(g);
    }
}
/// `ft_80081DD4` returning its bool (crate hook models it as unit). TODO rewire.
pub static mut FT_80081DD4_BOOL: Option<fn(G) -> bool> = None;
fn ext_ft_80081dd4(g: G) -> bool {
    match unsafe { FT_80081DD4_BOOL } {
        Some(f) => f(g),
        None => panic!("TODO stub: ft_80081DD4"),
    }
}

/// `ftCo_800C8064`: `ft_804D6580 = first DObj's mobj of HSD_JObjLoadJoint(Fighter_804D6504)`
/// (HSD calls behind one stub).
pub fn ftco_800c8064() {
    melee_core::ct!("ftco_800c8064");
    unsafe {
        let dobj = ext::HSD_JObjGetDObj(ext::HSD_JObjLoadJoint(crate::tu::fighter::Fighter_804D6504 as *mut core::ffi::c_void)) as *mut HSD_DObj;
        ft_804D6580 = if !dobj.is_null() { (*dobj).mobj } else { core::ptr::null_mut() };
    }
}

/// `ft_800C80A4`: for each visible part bone, set lighting/texgen flags and clear the DObjs'
/// flag bits (HSD JObj/DObj edits behind one per-bone stub).
pub fn ft_800c80a4(f: &mut Fighter) {
    melee_core::ct!("ft_800c80a4");
    let tbl = unsafe { *h::ft_parts_table().add(f.kind as usize) };
    let n = unsafe { (*tbl).parts_num } as usize;
    for i in 0..n {
        let bone = unsafe { &mut *f.parts.add(i) };
        if unsafe { bone.anon_8.anon_0.flags_b1() } != 0 {
            ext::ft_800C80A4_bone(bone.joint);
        }
    }
}
