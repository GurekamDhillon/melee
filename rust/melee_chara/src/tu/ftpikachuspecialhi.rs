//! src/melee/ft/kinds/ftPikachu/ftpikachuspecialhi.c: Pikachu / Pichu Quick Attack (up special).
//! Fused sites (tools/rust-port/fma_sites.py), all f32 `fmuladd` -> fmadds(a, b, c):
//! * the stick magnitude `sqrtf(x*x + y*y)` = fmadds(x, x, y*y) in `ftPk_SpecialHi_80126C0C`,
//!   `ftPk_SpecialHi_80126E1C` and `ftPk_SpecialHi_80127064` (MSL `sqrtf` is `gekko::sqrtf`; its three
//!   f64 Newton steps are fused inside that port);
//! * the zip speed `x90 * stick_mag + x94` (once in the ground zip, twice, for cos and sin, in the
//!   air zip);
//! * the spark jitter `6 * rand - 3` / `10 * rand - 5` in `ftPk_Special(Air)HiStart1_Anim`;
//! * the rotation `facing_dir * atan2f(..) + offset` in `ftPk_SpecialHi_8012642C`,
//!   `ftPk_SpecialHiStart1_Coll` (x68) and `ftPk_SpecialHi_ChangeMotion_Unk03` (x68).
//! `xA0` and `xA8` are C `int` angles (`90.0f + xA0`, `MTXDegToRad(xA8)` convert to float);
//! `mv.pk.specialhi.x4/x18` are ints compared with float attributes. Smoke-tested through
//! recording hooks by tests/tu_ftpikachuspecialhi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::chars::caps_max_jumps;
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::ftcommon as co;
use melee_runtime::gekko::{cosf, sinf, sqrtf};
use melee_runtime::lbtrigf::atan2f;
use melee_runtime::lbvector::angle_xy as core_angle_xy;
use melee_types::*;

/// Already ported (rounds 2/3/4/11 and the original pass): re-exported under this TU.
pub use crate::chars::r11::ftPikachu::ftpikachuspecialhi::ftPk_SpecialAirHiStart0_Coll;
pub use crate::chars::r2::ftPikachu::ftpikachuspecialhi::{
    ftPk_SpecialHiStart0_Phys, ftPk_SpecialHi_ChangeMotion_Unk00, ftPk_SpecialHi_ChangeMotion_Unk01,
    ftPk_SpecialHi_ChangeMotion_Unk04,
};
pub use crate::chars::r3::ftPikachu::ftpikachuspecialhi::ftPk_SpecialHiStart0_Coll;
pub use crate::chars::r4::ftPikachu::ftpikachuspecialhi::ftPk_SpecialHiEnd_Phys;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// Like `hook!`, but an unset hook falls back to an existing native port.
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    hook!(HSD_RANDF, HSD_Randf, () -> f32);
    pub static mut HSD_JOBJSETSCALE: Option<fn(*mut HSD_JObj, *const Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *const Vec3)  {
                match unsafe { HSD_JOBJSETSCALE } {
                    Some(f) => f(jobj, scale),
                    None => { unsafe { let _ = crate::tu::ftinl::HSD_JObjSetScale(jobj as _, scale as _); } return; },
                }
            }
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing pointers as a slice.
    hook!(EFSYNC_SPAWN, efSync_Spawn,
        (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut core::ffi::c_void]) -> *mut core::ffi::c_void);
    hook_or!(FTCO_8009A134, ftCo_8009A134, melee_fighter::tu::ftCo_Pass::ftCo_8009A134,
        (gobj: *mut HSD_GObj) -> bool);
    hook_or!(FT_80082888, ft_80082888, melee_fighter::tu::ft_081b::ft_80082888,
        (gobj: *mut HSD_GObj, box_: *mut ftCollisionBox) -> bool);
    hook_or!(FT_8008239C, ft_8008239C, melee_fighter::tu::ft_081b::ft_8008239c,
        (gobj: *mut HSD_GObj, dir: i32, box_: *mut ftCollisionBox) -> bool);
    /// Recording hook; unset, the native `melee_fighter::tu::ftparts` port runs.
    pub static mut FTPARTSETROTX: Option<fn(*mut Fighter, i32, f32)> = None;
    pub fn ftPartSetRotX(fp: *mut Fighter, part: i32, v: f32) {
        match unsafe { FTPARTSETROTX } {
            Some(f) => f(fp, part, v),
            None => melee_fighter::tu::ftparts::ftPartSetRotX(unsafe { &*fp }, part, v),
        }
    }
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftPikachuAttributes {
    (*fp).dat_attrs as *mut ftPikachuAttributes
}

/// `MAX_STICK_MAG`.
const MAX_STICK_MAG: f32 = 0.999;
/// `(float) M_PI_2`.
const HALF_PI: f32 = core::f32::consts::FRAC_PI_2;
/// `MTXDegToRad(a)` is `a * 0.01745329252f` (dolphin/mtx.h).
const DEG_TO_RAD: f32 = 0.01745329252;
/// `ftPk_MF_SpecialHiStart_Coll` / `ftPk_MF_SpecialHiMove_Coll` (ftPikachu/forward.h).
const MF_SPECIALHISTART_COLL: u32 = 0xC4C5084;
const MF_SPECIALHIMOVE_COLL: u32 = 0xC4C508A;
const Collide_CeilingMask: i32 = 0x2000 | 0x4000;
const Collide_FloorMask: i32 = 0x8000 | 0x10000;

/// `ftGetFacingDirInt` (ft/inlines.h).
#[inline(always)]
unsafe fn facing_dir_int(fp: *mut Fighter) -> i32 {
    if (*fp).facing_dir < 0.0 {
        -1
    } else {
        1
    }
}

/// `lbVector_AngleXY` on this crate's `Vec3` (same layout as `melee_core::vec::Vec3`).
#[inline(always)]
fn angle_xy(a: &Vec3, b: &Vec3) -> f32 {
    let c = |v: &Vec3| melee_core::vec::Vec3 { x: v.x, y: v.y, z: v.z };
    core_angle_xy(&c(a), &c(b))
}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
unsafe fn set_hitlag_cbs(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

#[inline(always)]
unsafe fn xrotn_joint(fp: *mut Fighter) -> *mut HSD_JObj {
    let part = melee_fighter::hooks::ftparts_get_bone_index(&*fp, FtPart_XRotN as i32);
    (*(*fp).parts.add(part as usize)).joint
}

#[inline(always)]
unsafe fn set_xrotn_rot(fp: *mut Fighter, angle: f32) {
    let part = melee_fighter::hooks::ftparts_get_bone_index(&*fp, FtPart_XRotN as i32);
    ext::ftPartSetRotX(fp, part, angle);
}

/// `HSD_JObjSetScale(XRotN, &(x, y, z))` with a copy of an attribute vector.
#[inline(always)]
unsafe fn set_xrotn_scale(fp: *mut Fighter, s: &Vec) {
    let scl = Vec3 { x: s.x, y: s.y, z: s.z };
    ext::HSD_JObjSetScale(xrotn_joint(fp), &scl);
}

/// `ftCommon_HandleTeleportCollisions` (ft/inlines.h): `on_collide` once per ceiling / left wall /
/// right wall whose normal is more than `90 + angle_clamp` degrees from the velocity.
unsafe fn handle_teleport_collisions(gobj: G, fp: *mut Fighter, angle_clamp: i32, on_collide: fn(G)) {
    let coll = &(*fp).coll_data as *const CollData;
    let limit = (90.0f32 + angle_clamp as f32) * DEG_TO_RAD;
    if (*coll).env_flags & Collide_CeilingMask != 0 && angle_xy(&(*coll).ceiling.normal, &(*fp).self_vel) > limit {
        on_collide(gobj);
    }
    let limit = (90.0f32 + angle_clamp as f32) * DEG_TO_RAD;
    if (*coll).env_flags & Collide_LeftWallMask != 0
        && angle_xy(&(*coll).left_facing_wall.normal, &(*fp).self_vel) > limit
    {
        on_collide(gobj);
    }
    let limit = (90.0f32 + angle_clamp as f32) * DEG_TO_RAD;
    if (*coll).env_flags & Collide_RightWallMask != 0
        && angle_xy(&(*coll).right_facing_wall.normal, &(*fp).self_vel) > limit
    {
        on_collide(gobj);
    }
}

/// Gobj entries for the ported `&mut Fighter` callbacks.
pub fn ftPk_SpecialAirHiEnd_Phys(gobj: G) {
    unsafe { crate::chars::ftPikachu::ftpikachuspecialhi::ftPk_SpecialAirHiEnd_Phys(&mut *fpp(gobj)) }
}
pub fn ftPk_SpecialAirHiStart0_Phys(gobj: G) {
    unsafe { crate::chars::ftPikachu::ftpikachuspecialhi::ftPk_SpecialAirHiStart0_Phys(&mut *fpp(gobj)) }
}
pub fn ftPk_SpecialHiStart1_Phys(gobj: G) {
    unsafe { crate::chars::ftPikachu::ftpikachuspecialhi::ftPk_SpecialHiStart1_Phys(&mut *fpp(gobj)) }
}
/// `ftPk_SpecialHi_UpdateVel`: points velocity toward the facing direction.
pub fn ftPk_SpecialHi_UpdateVel(gobj: G) {
    unsafe { crate::chars::ftPikachu::ftpikachuspecialhi::ftPk_SpecialHi_UpdateVel(&mut *fpp(gobj)) }
}
/// `ftPk_SpecialHi_UpdateVel` as the `x21F8` callback entry.
pub unsafe extern "C" fn cb_ftPk_SpecialHi_UpdateVel(g: G) {
    ftPk_SpecialHi_UpdateVel(g);
}

/// Enter body shared by ground (motion 353) and air (356).
unsafe fn enter(gobj: G, msid: i32) {
    let fp = fpp(gobj);
    let sa = da(fp);
    (*fp).cmd_vars[0] = 0;
    (*fp).mv.pk.specialhi.x0 = (*sa).x5C;
    (*fp).mv.pk.specialhi.x8 = 0;
    (*fp).mv.pk.specialhi.x18 = 0;
    (*fp).gr_vel = 0.0;
    (*fp).self_vel.y = 0.0;
    (*fp).self_vel.x = 0.0;
    st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

pub fn ftPk_SpecialHi_Enter(gobj: G) {
    unsafe { enter(gobj, 353) }
}

pub fn ftPk_SpecialAirHi_Enter(gobj: G) {
    unsafe { enter(gobj, 356) }
}

pub fn ftPk_SpecialHiStart0_Anim(gobj: G) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftPk_SpecialHi_80126C0C(gobj);
    }
}

pub fn ftPk_SpecialAirHiStart0_Anim(gobj: G) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftPk_SpecialHi_80126E1C(gobj);
    }
}

pub fn ftPk_SpecialHiStart0_IASA(_gobj: G) {}

pub fn ftPk_SpecialAirHiStart0_IASA(_gobj: G) {}

/// The spark effect of `Start1_Anim`: `jitter` is the C's `6` / `10` (ground / air); the offset is
/// `jitter * rand - jitter / 2`.
unsafe fn start1_anim(gobj: G, jitter: f32, minus: f32, motion_change: fn(G)) {
    let fp = fpp(gobj);
    (*fp).mv.pk.specialhi.x4 -= 1;
    if (*fp).mv.pk.specialhi.x4 <= 0 {
        motion_change(gobj);
        let fp = fpp(gobj);
        if (*fp).kind != Ft_Kind_Pichu {
            let mut vec: Vec3 = core::mem::zeroed();
            ext::lb_8000B1CC(xrotn_joint(fp), core::ptr::null(), &mut vec);
            ext::efSync_Spawn(1012, gobj, &[&mut vec as *mut Vec3 as *mut core::ffi::c_void]);
            (*fp).set_x2219_b0(1);
            set_hitlag_cbs(fp);
        }
    } else {
        let fp = fpp(gobj);
        if (*fp).kind != Ft_Kind_Pichu {
            let mut vec2: Vec3 = core::mem::zeroed();
            ext::lb_8000B1CC(xrotn_joint(fp), core::ptr::null(), &mut vec2);
            let mut tempf = ext::HSD_Randf();
            vec2.x += fmadds(jitter, tempf, -minus);
            tempf = ext::HSD_Randf();
            vec2.y += fmadds(jitter, tempf, -minus);
            ext::efSync_Spawn(1012, gobj, &[&mut vec2 as *mut Vec3 as *mut core::ffi::c_void]);
            (*fp).set_x2219_b0(1);
            set_hitlag_cbs(fp);
        }
    }
}

pub fn ftPk_SpecialHiStart1_Anim(gobj: G) {
    unsafe { start1_anim(gobj, 6.0, 3.0, ftPk_SpecialHi_MotionChangeUpdateVel_Unk0) }
}

pub fn ftPk_SpecialAirHiStart1_Anim(gobj: G) {
    unsafe { start1_anim(gobj, 10.0, 5.0, ftPk_SpecialHi_MotionChangeUpdateVel_Unk1) }
}

pub fn ftPk_SpecialHiStart1_IASA(_gobj: G) {}

pub fn ftPk_SpecialAirHiStart1_IASA(_gobj: G) {}

/// `ftPk_SpecialHi_8012642C`: aims the model along the velocity and clears the transN offsets.
pub fn ftPk_SpecialHi_8012642C(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        let tempf = fmadds(
            (*fp).facing_dir,
            atan2f((*fp).self_vel.x, (*fp).self_vel.y),
            (*pika_attr).x78 - HALF_PI,
        );
        set_xrotn_rot(fp, tempf);
        set_xrotn_scale(fp, &(*pika_attr).x7C_scale);

        let velocity_vec = (*fp).self_vel;
        st::ftCommon_8007E2FC(gobj);
        (*fp).self_vel = velocity_vec;

        (*fp).x6D8.x = 0.0;
        (*fp).x6D8.y = 0.0;
        (*fp).x6D8.z = 0.0;
        (*fp).x6C0.x = 0.0;
        (*fp).x6C0.y = 0.0;
        (*fp).x6C0.z = 0.0;
        (*fp).lstick_angle = 0.0;
        (*fp).x6A4_transNOffset.x = 0.0;
        (*fp).x6A4_transNOffset.y = 0.0;
        (*fp).x6A4_transNOffset.z = 0.0;
        (*fp).x68C_transNPos.x = 0.0;
        (*fp).x68C_transNPos.y = 0.0;
        (*fp).x68C_transNPos.z = 0.0;
    }
}

pub fn ftPk_SpecialAirHiStart1_Phys(gobj: G) {
    ftPk_SpecialHi_8012642C(gobj);
}

/// The wall test used by the Start1 / Move collisions.
#[inline(always)]
unsafe fn hit_wall(fp: *mut Fighter) -> bool {
    let env = (*fp).coll_data.env_flags;
    env & Collide_LeftWallMask != 0 || env & Collide_RightWallMask != 0
}

pub fn ftPk_SpecialHiStart1_Coll(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        if !ext::ft_80082888(gobj, &mut (*pika_attr).height_attributes) {
            if hit_wall(fp) {
                st::ftCommon_8007D60C(&mut *fp);
                ftPk_SpecialHi_MotionChangeUpdateVel_Unk1(gobj);
                return;
            }
            ftPk_SpecialHi_ChangeMotion_Unk02(gobj);
            return;
        }

        // Possibly an inline in the C.
        let fighter2 = fpp(gobj);
        let pika_attr = da(fighter2);
        if (*fighter2).coll_data.env_flags & Collide_FloorMask != 0 {
            let angle = atan2f((*fighter2).coll_data.floor.normal.x, (*fighter2).coll_data.floor.normal.y);
            let angle2 = fmadds((*fighter2).facing_dir, angle, (*pika_attr).x68);
            set_xrotn_rot(fighter2, angle2);
        }
        set_xrotn_scale(fighter2, &(*pika_attr).x6C_scale);

        if hit_wall(fp) {
            ftPk_SpecialHi_MotionChangeUpdateVel_Unk0(gobj);
        }
    }
}

/// `ftPikachu_GetBool` (static).
unsafe fn ftPikachu_GetBool(gobj: G) -> bool {
    let fighter2 = fpp(gobj);
    let pika_attr2 = da(fighter2);
    if (*fighter2).mv.pk.specialhi.x18 as f32 >= (*pika_attr2).x88 {
        true
    } else if ext::ftCo_8009A134(gobj) {
        false
    } else {
        true
    }
}

pub fn ftPk_SpecialAirHiStart1_Coll(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        (*fp).mv.pk.specialhi.x18 += 1;
        if st::ft_CheckGroundAndLedge(gobj, facing_dir_int(fp)) != 0 {
            let bool0 = ftPikachu_GetBool(gobj);
            if bool0 {
                let tempf = angle_xy(&(*fp).coll_data.floor.normal, &(*fp).self_vel);
                if tempf > 0.017453292f32 * (90.0f32 + (*pika_attr).xA0 as f32) {
                    st::ftCommon_8007D7FC(&mut *fp);
                    ftPk_SpecialHi_MotionChangeUpdateVel_Unk0(gobj);
                    return;
                }
                ftPk_SpecialHi_ChangeMotion_Unk03(gobj);
                return;
            }
        }
        if st::ftCliffCommon_80081298(gobj) == 0 {
            handle_teleport_collisions(gobj, fp, (*pika_attr).xA0, ftPk_SpecialHi_MotionChangeUpdateVel_Unk1);
        }
    }
}

pub fn ftPk_SpecialHi_ChangeMotion_Unk02(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D60C(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 357, MF_SPECIALHIMOVE_COLL, (*fp).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());
        (*fp).set_x2223_b4(1);
        ftPk_SpecialHi_8012642C(gobj);
    }
}

pub fn ftPk_SpecialHi_ChangeMotion_Unk03(gobj: G) {
    unsafe {
        let fighter2 = fpp(gobj);
        st::ftCommon_8007D7FC(&mut *fighter2);
        st::Fighter_ChangeMotionState(gobj, 354, MF_SPECIALHIMOVE_COLL, (*fighter2).cur_anim_frame, 0.0, 0.0, core::ptr::null_mut());

        let fp = fpp(gobj);
        let pika_attr = da(fp);
        if (*fp).coll_data.env_flags & Collide_FloorMask != 0 {
            let angle = fmadds(
                (*fp).facing_dir,
                atan2f((*fp).coll_data.floor.normal.x, (*fp).coll_data.floor.normal.y),
                (*pika_attr).x68,
            );
            set_xrotn_rot(fp, angle);
        }
        set_xrotn_scale(fp, &(*pika_attr).x6C_scale);
    }
}

/// `get_max_and_fill_stack` (static inline): returns `MAX_STICK_MAG` (the stack fill is dead).
#[inline(always)]
fn get_max_and_fill_stack() -> f32 {
    MAX_STICK_MAG
}

/// The stick magnitude: `sqrtf(x*x + y*y)`, the product pair fused (see the module doc).
#[inline(always)]
unsafe fn stick_mag(fp: *mut Fighter) -> f32 {
    let x = (*fp).input.lstick[0].x;
    let y = (*fp).input.lstick[0].y;
    sqrtf(fmadds(x, x, y * y))
}

/// Grounded Quick Attack zip.
pub fn ftPk_SpecialHi_80126C0C(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let coll_data = &(*fp).coll_data as *const CollData;
        let pika_attr = da(fp);

        let mut stick_mag = stick_mag(fp);
        // cap the stick magnitude to MAX_STICK_MAG
        if stick_mag > MAX_STICK_MAG {
            stick_mag = get_max_and_fill_stack();
        }

        if !(stick_mag < (*pika_attr).x8C) {
            let lstick_direction =
                Vec3 { x: (*fp).input.lstick[0].x, y: (*fp).input.lstick[0].y, z: 0.0 };
            if !(angle_xy(&(*coll_data).floor.normal, &lstick_direction) < HALF_PI)
                && !ext::ftCo_8009A134(gobj)
            {
                st::ftCommon_UpdateFacing(&mut *fp);
                // store the stick to compare during the second-zip check
                (*fp).mv.pk.specialhi.x10.x = lstick_direction.x;
                (*fp).mv.pk.specialhi.x10.y = lstick_direction.y;

                let fighter2 = fpp(gobj);
                // zip duration
                (*fighter2).mv.pk.specialhi.x4 = (*da(fighter2)).x60;
                // lose the double jump(s)
                (*fighter2).x1968_jumpsUsed = caps_max_jumps(&*fighter2) as u8;

                // ground velocity = (zip_slope * stick_mag) + zip_intercept, flipped by facing
                (*fp).gr_vel = fmadds((*pika_attr).x90, stick_mag, (*pika_attr).x94);
                (*fp).gr_vel *= (*fp).facing_dir;

                // second zip
                if (*fp).mv.pk.specialhi.x8 != 0 {
                    (*fp).gr_vel *= (*pika_attr).x98;
                    st::Fighter_ChangeMotionState(gobj, 354, 2, 12.0, 1.0, 0.0, core::ptr::null_mut());
                    st::ftAnim_8006EBA4(gobj);
                }
                st::Fighter_ChangeMotionState(gobj, 354, 10, 13.0, 1.0, 0.0, core::ptr::null_mut());
                st::ftAnim_8006EBA4(gobj);
                st::ftAnim_SetAnimRate(gobj, 0.0);
                (*fp).set_x2223_b4(1);
                (*fp).x21F8 = Some(cb_ftPk_SpecialHi_UpdateVel);
                return;
            }
        }
        st::ftCommon_8007D60C(&mut *fp);
        ftPk_SpecialHi_80126E1C(gobj);
    }
}

/// Aerial Quick Attack zip.
pub fn ftPk_SpecialHi_80126E1C(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);

        let temp_stick_mag = stick_mag(fp);
        let mut final_stick_mag = temp_stick_mag;
        // cap the stick magnitude to MAX_STICK_MAG
        if temp_stick_mag > MAX_STICK_MAG {
            final_stick_mag = get_max_and_fill_stack();
        }

        let some_angle: f32;
        if final_stick_mag > (*pika_attr).x8C {
            if co::abs_c((*fp).input.lstick[0].x) > 0.001 {
                st::ftCommon_UpdateFacing(&mut *fp);
            }
            // zip angle = atan2(stick_y, stick_x * facing_direction)
            some_angle = atan2f((*fp).input.lstick[0].y, (*fp).input.lstick[0].x * (*fp).facing_dir);
            (*fp).mv.pk.specialhi.x10.x = (*fp).input.lstick[0].x;
            (*fp).mv.pk.specialhi.x10.y = (*fp).input.lstick[0].y;
        } else {
            // face the stick when it is outside the horizontal deadzone (`ftCommon_8007DA24`)
            co::update_facing_outside_deadzone(&mut *fp);
            // use the max magnitude and 90 degrees
            final_stick_mag = MAX_STICK_MAG;
            some_angle = HALF_PI;
            (*fp).mv.pk.specialhi.x10.x = 0.0;
            (*fp).mv.pk.specialhi.x10.y = MAX_STICK_MAG;
        }

        let fighter2 = fpp(gobj);
        // zip duration
        (*fighter2).mv.pk.specialhi.x4 = (*da(fighter2)).x60;
        // lose the double jump(s)
        (*fighter2).x1968_jumpsUsed = caps_max_jumps(&*fighter2) as u8;

        // speed = zip_slope * stick_mag + zip_intercept; x flips with the facing direction
        let temp_f2_2 = fmadds((*pika_attr).x90, final_stick_mag, (*pika_attr).x94) * cosf(some_angle);
        (*fp).self_vel.x = (*fp).facing_dir * temp_f2_2;
        (*fp).self_vel.y = fmadds((*pika_attr).x90, final_stick_mag, (*pika_attr).x94) * sinf(some_angle);

        // second zip
        if (*fp).mv.pk.specialhi.x8 != 0 {
            (*fp).self_vel.x *= (*pika_attr).x98;
            (*fp).self_vel.y *= (*pika_attr).x98;
            st::Fighter_ChangeMotionState(gobj, 357, 2, 12.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
        }
        st::Fighter_ChangeMotionState(gobj, 357, 10, 13.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        st::ftAnim_SetAnimRate(gobj, 0.0);
        (*fp).set_x2223_b4(1);
        // move toward the facing direction
        (*fp).x21F8 = Some(cb_ftPk_SpecialHi_UpdateVel);
    }
}

/// `return_and_fill_stack` (static inline): returns 0 (the stack fill is dead).
#[inline(always)]
fn return_and_fill_stack() -> bool {
    false
}

/// Whether to perform a second zip: the stick is held and differs enough from the first zip's.
pub fn ftPk_SpecialHi_80127064(gobj: G) -> bool {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);

        let stick_mag = stick_mag(fp);
        if stick_mag < (*pika_attr).x8C {
            return return_and_fill_stack();
        }

        if (*fp).mv.pk.specialhi.x8 == 0 {
            let vec1 = Vec3 { x: (*fp).input.lstick[0].x, y: (*fp).input.lstick[0].y, z: 0.0 };
            let vec2 = Vec3 { x: (*fp).mv.pk.specialhi.x10.x, y: (*fp).mv.pk.specialhi.x10.y, z: 0.0 };
            let tempf = angle_xy(&vec2, &vec1);
            return tempf > (*pika_attr).xA8 as f32 * DEG_TO_RAD;
        }
        false
    }
}

pub fn ftPk_SpecialHiEnd_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] == 1 {
            if ftPk_SpecialHi_80127064(gobj) {
                (*fp).cmd_vars[0] = 0;
                (*fp).mv.pk.specialhi.x8 = 1;
                ftPk_SpecialHi_80126C0C(gobj);
                return;
            }
            (*fp).cmd_vars[0] = 2;
            return;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftPk_SpecialAirHiEnd_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        if (*fp).cmd_vars[0] == 1 {
            if ftPk_SpecialHi_80127064(gobj) {
                (*fp).cmd_vars[0] = 0;
                (*fp).mv.pk.specialhi.x8 = 1;
                ftPk_SpecialHi_80126E1C(gobj);
            } else {
                (*fp).cmd_vars[0] = 2;
            }
        } else if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_80096900(gobj, 1, 0, 1, (*sa).xAC, (*sa).xB0);
        }
    }
}

pub fn ftPk_SpecialHiEnd_IASA(_gobj: G) {}

pub fn ftPk_SpecialAirHiEnd_IASA(_gobj: G) {}

pub fn ftPk_SpecialHiEnd_Coll(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        if !ext::ft_80082888(gobj, &mut (*pika_attr).height_attributes) {
            ftPk_SpecialHi_ChangeMotion_Unk04(gobj);
        }
    }
}

pub fn ftPk_SpecialAirHiEnd_Coll(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        if ext::ft_8008239C(gobj, (*fp).facing_dir as i32, &mut (*pika_attr).height_attributes) {
            st::ftCo_LandingFallSpecial_Enter(gobj, 0, (*pika_attr).xB0);
        } else if st::ftCliffCommon_80081298(gobj) == 0 {
            // weird control flow in the C: both arms fall off the end
            return;
        }
    }
}

pub fn ftPk_SpecialHi_MotionChangeUpdateVel_Unk0(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        (*fp).mv.pk.specialhi.x1C.x = (*fp).self_vel.x;
        (*fp).mv.pk.specialhi.x1C.y = (*fp).self_vel.y;
        (*fp).mv.pk.specialhi.x24 = (*fp).gr_vel;

        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = 0.0;
        (*fp).gr_vel = 0.0;
        (*fp).gr_vel = (*fp).mv.pk.specialhi.x24 * (*pika_attr).xA4;
        st::Fighter_ChangeMotionState(gobj, 355, 2, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).x21F8 = Some(cb_ftPk_SpecialHi_UpdateVel);
    }
}

pub fn ftPk_SpecialHi_MotionChangeUpdateVel_Unk1(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let pika_attr = da(fp);
        (*fp).mv.pk.specialhi.x1C.x = (*fp).self_vel.x;
        (*fp).mv.pk.specialhi.x1C.y = (*fp).self_vel.y;
        (*fp).mv.pk.specialhi.x24 = (*fp).gr_vel;

        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = 0.0;
        (*fp).gr_vel = 0.0;
        (*fp).self_vel.x = (*fp).mv.pk.specialhi.x1C.x * (*pika_attr).xA4;
        (*fp).self_vel.y = (*fp).mv.pk.specialhi.x1C.y * (*pika_attr).xA4;
        st::Fighter_ChangeMotionState(gobj, 358, 2, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).x21F8 = Some(cb_ftPk_SpecialHi_UpdateVel);
    }
}
