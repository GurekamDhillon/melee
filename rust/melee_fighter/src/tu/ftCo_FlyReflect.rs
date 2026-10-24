#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_FlyReflect.c (every function of the TU). Calls out of it
//! go through `ext`. Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_FlyReflect.rs.
use crate::types::*;
use melee_core::fp::fmadds;

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
        ftCo_800C23A0(g: G) -> bool = crate::tu::ftCo_PassiveCeil::ftCo_800C23A0;
        ftCo_800C1D38(g: G) -> bool = crate::tu::ftCo_PassiveWall::ftCo_800C1D38;
        ftKb_SpecialN_800F1F1C(g: G, ofs: &mut Vec3) = |_, _| todo!("TODO(port) ftKb_SpecialN_800F1F1C (ftKb_SpecialN.c)");
        // `efAsync_Spawn(gobj, queue, kind, gfx_id, jobj, ...)`: kind 5 passes `Vec3*` then `float*`.
        efAsync_Spawn(g: G, queue: *mut core::ffi::c_void, kind: u32, id: u32, jobj: *mut HSD_JObj, a1: *mut core::ffi::c_void, a2: *mut core::ffi::c_void) = |_, _, _, _, _, _, _| todo!("TODO(port) efAsync_Spawn (ef/efasync.c)");
        Camera_RequestQuake(kind: i32, pos: *mut Vec) = |_, _| todo!("TODO(port) Camera_RequestQuake (cm/camera.c)");
        atan2f(y: f32, x: f32) -> f32 = melee_runtime::lbtrigf::atan2f;
        lbVector_Add_xy(a: &mut Vec3, b: &Vec3) = |a: &mut Vec3, b: &Vec3| { a.x += b.x; a.y += b.y; };
        lbVector_Mirror(a: &mut Vec3, n: &Vec3) = |_, _| todo!("TODO(port) lbVector_Mirror (lb/lbvector.c)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ft_80081F2C(g: G) -> bool = crate::tu::ft_081b::ft_80081f2c;
        ft_80082084(g: G) -> bool = crate::tu::ft_081b::ft_80082084;
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ftColl_8007B760(g: G, v: i32) = |_, _| todo!("TODO(port) ftColl_8007B760 (ftcoll.c)");
        ftCo_80097630(fp: &mut Fighter, sfx_ids: &[i32; 4], threshold: f32) = crate::tu::ftCo_DownBound::ftCo_80097630;
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ftCo_80090184(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_80090184(g as _); } };
        ftCo_DamageFly_Anim(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_DamageFly_Anim(g as _); } };
        ftCo_DamageFly_IASA(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_DamageFly_IASA(g as _); } };
    }
}

/// Left/right wall reflect.
pub fn ftCo_800C15F4(g: G) -> bool {
    let threshold = cd().x1B0;
    let kb_vel_x = fp(g).x8c_kb_vel.x;
    if kb_vel_x < -threshold
        && fp(g).coll_data.env_flags & Collide_RightWallHug != 0
        && unsafe { fp(g).mv.co.damage.x19 } != ftCo_Surface_LeftWall as u8
    {
        let c = &fp(g).coll_data;
        let mut vec = Vec3 { x: c.ecb.left.x, y: c.ecb.left.y, z: 0.0 };
        ext::ftKb_SpecialN_800F1F1C(g, &mut vec);
        let normal = &fp(g).coll_data.right_facing_wall.normal as *const Vec3;
        ftCo_800C18A8(g, ftCo_MS_FlyReflectWall as i32, unsafe { &*normal }, &vec);
        unsafe { fp(g).mv.co.damage.x19 = ftCo_Surface_LeftWall as u8 };
        return true;
    }
    if kb_vel_x > threshold
        && fp(g).coll_data.env_flags & Collide_LeftWallHug != 0
        && unsafe { fp(g).mv.co.damage.x19 } != ftCo_Surface_RightWall as u8
    {
        let c = &fp(g).coll_data;
        let mut vec = Vec3 { x: c.ecb.right.x, y: c.ecb.right.y, z: 0.0 };
        ext::ftKb_SpecialN_800F1F1C(g, &mut vec);
        let normal = &fp(g).coll_data.left_facing_wall.normal as *const Vec3;
        ftCo_800C18A8(g, ftCo_MS_FlyReflectWall as i32, unsafe { &*normal }, &vec);
        unsafe { fp(g).mv.co.damage.x19 = ftCo_Surface_RightWall as u8 };
        return true;
    }
    false
}

/// Ceiling reflect.
pub fn ftCo_800C1718(g: G) -> bool {
    if fp(g).x8c_kb_vel.y > cd().x1B0
        && fp(g).coll_data.env_flags & Collide_CeilingHug != 0
        && unsafe { fp(g).mv.co.damage.x19 } != ftCo_Surface_Ceiling as u8
    {
        let mut vec = Vec3 { x: 0.0, y: fp(g).coll_data.ecb.top.y, z: 0.0 };
        ext::ftKb_SpecialN_800F1F1C(g, &mut vec);
        let normal = &fp(g).coll_data.ceiling.normal as *const Vec3;
        ftCo_800C18A8(g, ftCo_MS_FlyReflectCeil as i32, unsafe { &*normal }, &vec);
        unsafe { fp(g).mv.co.damage.x19 = ftCo_Surface_Ceiling as u8 };
        return true;
    }
    false
}

/// static inline `ftCo_800C1718_inline` (same body as `ftCo_800C1718`).
fn ftCo_800C1718_inline(g: G) -> bool {
    ftCo_800C1718(g)
}

/// Wall check, then ceiling check.
pub fn ftCo_800C17CC(g: G) -> bool {
    if ftCo_800C15F4(g) {
        return true;
    }
    ftCo_800C1718(g)
}

pub fn ftCo_800C18A8(g: G, msid: i32, normal: &Vec3, offset: &Vec3) {
    let f = fp(g);
    let mut vec0 = Vec3 { x: f.cur_pos.x + offset.x, y: f.cur_pos.y + offset.y, z: f.cur_pos.z + offset.z };
    let mut param = ext::atan2f(-normal.x, normal.y);
    ext::efAsync_Spawn(
        g,
        &mut f.x60C as *mut _ as *mut core::ffi::c_void,
        5,
        (1 << 1) | (1 << 2) | (1 << 10),
        core::ptr::null_mut(),
        &mut vec0 as *mut Vec3 as *mut core::ffi::c_void,
        &mut param as *mut f32 as *mut core::ffi::c_void,
    );
    ext::Camera_RequestQuake(QuakeKind_Small as i32, &mut vec0 as *mut Vec3 as *mut Vec);
    let f = fp(g);
    let mut vec1 = f.self_vel;
    ext::lbVector_Add_xy(&mut vec1, &f.x8c_kb_vel);
    ext::lbVector_Mirror(&mut vec1, normal);
    let co_x1bc = cd().x1BC;
    vec1.x *= co_x1bc;
    vec1.y *= co_x1bc;
    f.x8c_kb_vel = vec1;
    f.self_vel.x = 0.0;
    f.self_vel.y = 0.0;
    f.self_vel.z = 0.0;
    f.facing_dir = if f.x8c_kb_vel.x < 0.0 { -1.0 } else { 1.0 };
    ext::Fighter_ChangeMotionState(
        g,
        msid,
        (Ft_MF_Unk06 | Ft_MF_SkipNametagVis | Ft_MF_KeepColAnimPartHitStatus | Ft_MF_SkipHitStun) as u32,
        0.0,
        1.0,
        0.0,
        core::ptr::null_mut(),
    );
    let f = fp(g);
    if msid == ftCo_MS_FlyReflectWall as i32 {
        // `-(z * -facing - (x + off))`: the inner a*b-c contracts to fmuladd(a, b, -c).
        f.cur_pos.x = -fmadds(f.x68C_transNPos.z, -f.facing_dir, -(f.cur_pos.x + offset.x));
        ext::ft_80081F2C(g);
    } else {
        f.cur_pos.y = f.x68C_transNPos.y + (f.cur_pos.y + offset.y);
        ext::ft_80082084(g);
    }
    unsafe { fp(g).mv.co.damage.x18 = cd().x1C0 as u8 };
    ext::ftCommon_8007EBAC(fp(g), 7, 0);
    ext::ftColl_8007B760(g, cd().x1B8);
    let f = fp(g);
    let vel_x = f.self_vel.x + f.x8c_kb_vel.x;
    let vel_y = f.self_vel.y + f.x8c_kb_vel.y;
    let mag = melee_runtime::gekko::sqrtf(fmadds(vel_x, vel_x, vel_y * vel_y));
    let w = f.co_attrs.weight;
    ext::ftCo_80097630(fp(g), &crate::tu::ftCo_DownBound::ftCo_DownBound_SfxIds, mag * w);
}

pub fn ftCo_FlyReflect_Anim(g: G) {
    let f = fp(g);
    unsafe {
        if f.mv.co.damage.x18 != 0 {
            f.mv.co.damage.x18 -= 1;
        }
    }
    ext::ftCo_DamageFly_Anim(g);
}

pub fn ftCo_FlyReflect_IASA(g: G) {
    ext::ftCo_DamageFly_IASA(g);
}

pub fn ftCo_FlyReflect_Phys(g: G) {
    ext::ft_80084DB0(g);
}

pub fn ftCo_FlyReflect_Coll(g: G) {
    if fp(g).motion_id == ftCo_MS_FlyReflectWall as _ {
        if ext::ft_80081F2C(g) {
            ext::ftCo_80090184(g);
            return;
        }
        if ext::ftCo_800C23A0(g) {
            return;
        }
        if ftCo_800C1718_inline(g) {
            return;
        }
        if unsafe { fp(g).mv.co.damage.x18 } == 0 {
            if ext::ftCo_800C1D38(g) {
                return;
            }
            if ftCo_800C15F4(g) || ftCo_800C1718(g) {
                return;
            }
        }
    } else {
        if ext::ft_80082084(g) {
            ext::ftCo_80090184(g);
            return;
        }
        if ext::ftCo_800C1D38(g) {
            return;
        }
        if ftCo_800C15F4(g) {
            return;
        }
    }
}
