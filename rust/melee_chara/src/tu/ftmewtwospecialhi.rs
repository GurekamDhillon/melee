//! src/melee/ft/kinds/ftMewtwo/ftmewtwospecialhi.c: Teleport (up B). Fused sites
//! (tools/rust-port/fma_sites.py): `ftMt_SpecialHi_Enter` 1 and `ftMt_SpecialAirHi_Enter` 2, all
//! `x5C_MOMENTUM * stick + x60_MOMENTUM_ADD` = fmadds; the file's `sqrtf` is
//! `melee_runtime::gekko::sqrtf`. The aerial zoom entry is checked against an independent
//! double-precision fused multiply-add by tests/tu_ftmewtwospecialhi.rs; the rest is
//! smoke-tested through recording hooks (no compiled-C oracle).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::chars::cb;
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf, sqrtf};
use melee_runtime::lbtrigf::atan2f;
use melee_types::*;

/// `lbVector_AngleXY` over the generated `Vec` records.
fn angle_xy(a: &Vec, b: &Vec) -> f32 {
    let c = |v: &Vec| melee_core::vec::Vec3 { x: v.x, y: v.y, z: v.z };
    melee_runtime::lbvector::angle_xy(&c(a), &c(b))
}

/// Already ported (rounds 2/3/7/11 and the base module): re-exported under this TU.
pub use crate::chars::ftMewtwo::ftmewtwospecialhi::ftMt_SpecialHiLost_Phys;
pub use crate::chars::r11::ftMewtwo::ftmewtwospecialhi::{
    ftMt_SpecialAirHiStart_Coll, ftMt_SpecialAirHi_Coll,
};
pub use crate::chars::r2::ftMewtwo::ftmewtwospecialhi::{
    ftMt_SpecialHiLost_GroundToAir, ftMt_SpecialHiStart_Phys, ftMt_SpecialHi_Anim,
    ftMt_SpecialHi_Phys,
};
pub use crate::chars::r3::ftMewtwo::ftmewtwospecialhi::{
    ftMewtwo_SpecialHiLost_SetVars, ftMt_SpecialAirHiStart_AirToGround, ftMt_SpecialAirHi_AirToGround,
    ftMt_SpecialHiStart_Coll, ftMt_SpecialHiStart_GroundToAir, ftMt_SpecialHi_Coll,
    ftMt_SpecialHi_GroundToAir,
};
pub use crate::chars::r7::ftMewtwo::ftmewtwospecialhi::ftMt_SpecialAirHi_Anim;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a verified port in melee_fighter, which the unset hook calls.
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
    /// `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): shared with ftCaptain's TU.
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(joint: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(joint, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(joint as _, off as _, out as _); } return; },
                }
            }
    crate::port_hook!(FTCOLL_8007B62C, ftColl_8007B62C, crate::adapt::ftColl_8007B62C, (gobj: *mut HSD_GObj, arg: i32));
    crate::port_hook!(FTCOMMON_8007DA24, ftCommon_8007DA24, crate::adapt::ftCommon_8007DA24, (fp: *mut Fighter));
    fn ft_setfx_port(gobj: *mut HSD_GObj) {
        melee_fighter::tu::ftCo_AttackS4::Fighter_SetEffectHitlagCallbacks(unsafe {
            &mut *((*gobj).user_data as *mut Fighter)
        })
    }
    hook_or!(FIGHTER_SETEFFECTHITLAGCALLBACKS, Fighter_SetEffectHitlagCallbacks, ft_setfx_port, (gobj: *mut HSD_GObj));
    fn play_sfx_port(fp: *mut Fighter, id: i32, vol: i32, pan: i32) {
        melee_fighter::hooks::play_sfx(unsafe { &mut *fp }, id, vol, pan)
    }
    hook_or!(FT_PLAYSFX, ft_PlaySFX, play_sfx_port, (fp: *mut Fighter, id: i32, vol: i32, pan: i32));
    fn ftco_8009a134_port(gobj: *mut HSD_GObj) -> bool {
        melee_fighter::tu::ftCo_Pass::ftCo_8009A134(gobj)
    }
    hook_or!(FTCO_8009A134, ftCo_8009A134, ftco_8009a134_port, (gobj: *mut HSD_GObj) -> bool);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftMewtwoAttributes {
    (*fp).dat_attrs as *mut ftMewtwoAttributes
}

/// `ftGetFacingDirInt` (ft/inlines.h): the facing direction as -1 / +1.
#[inline(always)]
fn facing_dir_int(fp: *mut Fighter) -> i32 {
    if unsafe { (*fp).facing_dir } < 0.0 {
        -1
    } else {
        1
    }
}

/// `stickGetDir` (ft/inlines.h).
#[inline(always)]
fn stick_get_dir(x1: f32, x2: f32) -> f32 {
    if x1 < x2 {
        -x1
    } else {
        x1
    }
}

/// `Collide_CeilingMask` (mp/forward.h): `Collide_CeilingPush | Collide_CeilingHug`.
const COLLIDE_CEILING_MASK: i32 = Collide_CeilingPush | Collide_CeilingHug;

/// `ftCommon_HandleTeleportCollisions` (ft/inlines.h, static inline): `on_collide(gobj)` once per
/// ceiling / left wall / right wall contact whose normal is more than `90 + angle_clamp` degrees
/// off the velocity.
pub fn ftCommon_HandleTeleportCollisions(
    gobj: *mut HSD_GObj,
    fp: *mut Fighter,
    angle_clamp: i32,
    on_collide: fn(*mut HSD_GObj),
) {
    unsafe {
        let coll = &(*fp).coll_data;
        let lim = (90.0f32 + angle_clamp as f32) * 0.01745329252f32;
        if (coll.env_flags & COLLIDE_CEILING_MASK) != 0
            && angle_xy(&coll.ceiling.normal, &(*fp).self_vel) > lim
        {
            on_collide(gobj);
        }
        if (coll.env_flags & Collide_LeftWallMask) != 0
            && angle_xy(&coll.left_facing_wall.normal, &(*fp).self_vel) > lim
        {
            on_collide(gobj);
        }
        if (coll.env_flags & Collide_RightWallMask) != 0
            && angle_xy(&coll.right_facing_wall.normal, &(*fp).self_vel) > lim
        {
            on_collide(gobj);
        }
    }
}

/// Create Teleport Start GFX.
pub fn ftMt_SpecialHi_CreateGFX(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ftMt_SpecialHi_SetStartGFX(gobj);
        (*fp).accessory4_cb = None;
    }
}

/// Setup Teleport Start GFX.
pub fn ftMt_SpecialHi_SetStartGFX(gobj: *mut HSD_GObj) {
    unsafe {
        let fp0 = fpp(gobj);
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        if (*fp0).x2219_b0() == 0 {
            ext::lb_8000B1CC((*(*fp0).parts.add(FtPart_WaistN as usize)).joint, core::ptr::null(), &mut pos);
            ext::efSync_Spawn(0x4E8, gobj, &[&mut pos as *mut Vec3 as *mut core::ffi::c_void]);
            (*fp0).set_x2219_b0(1);
        }
        ext::Fighter_SetEffectHitlagCallbacks(gobj);
    }
}

/// Setup Teleport End GFX.
pub fn ftMt_SpecialHi_SetEndGFX(gobj: *mut HSD_GObj) {
    unsafe {
        let fp0 = fpp(gobj);
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        if (*fp0).x2219_b0() == 0 {
            ext::lb_8000B1CC((*(*fp0).parts.add(FtPart_WaistN as usize)).joint, core::ptr::null(), &mut pos);
            (*fp0).set_x2219_b0(1);
        }
        ext::Fighter_SetEffectHitlagCallbacks(gobj);
        (*fp0).accessory4_cb = None;
    }
}

/// Grounded Teleport Start Motion State handler.
pub fn ftMt_SpecialHiStart_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp0 = fpp(gobj);
        (*fp0).gr_vel = 0.0;
        (*fp0).self_vel.y = 0.0;
        (*fp0).self_vel.x = 0.0;

        st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialHiStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);

        (*fp0).cmd_vars[0] = 0;
        (*fp0).mv.mt.SpecialHi.unk4 = 0;
        (*fp0).accessory4_cb = Some(cb::ftmewtwospecialhi__ftMt_SpecialHi_CreateGFX);
    }
}

/// Aerial Teleport Start Motion State handler.
pub fn ftMt_SpecialAirHiStart_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);

        (*fp).self_vel.x /= (*a).x40_MEWTWO_TELEPORT_VEL_DIV_X;
        (*fp).self_vel.y /= (*a).x44_MEWTWO_TELEPORT_VEL_DIV_Y;

        st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirHiStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);

        (*fp).cmd_vars[0] = 0;
        (*fp).mv.mt.SpecialHi.unk4 = 0;
        (*fp).accessory4_cb = Some(cb::ftmewtwospecialhi__ftMt_SpecialHi_CreateGFX);
    }
}

pub fn ftMt_SpecialHiStart_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftMt_SpecialHi_Enter(gobj);
        }
    }
}

pub fn ftMt_SpecialAirHiStart_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftMt_SpecialAirHi_Enter(gobj);
        }
    }
}

pub fn ftMt_SpecialHiStart_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftMt_SpecialAirHiStart_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftMt_SpecialAirHiStart_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp0 = fpp(gobj);
        let a = da(fp0);
        st::ftCommon_Fall(&mut *fp0, (*a).x48_MEWTWO_TELEPORT_GRAVITY, (*a).x4C_MEWTWO_TELEPORT_TERMINAL_VELOCITY);
        melee_fighter::ftcommon::calc_self_accel_deaccel_air(&mut *fp0);
    }
}

pub fn ftMt_SpecialHiLost_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.mt.SpecialHi.travelFrames -= 1;
        if (*fp).mv.mt.SpecialHi.travelFrames <= 0 {
            ftMt_SpecialHiLost_Enter(gobj);
        }
    }
}

pub fn ftMt_SpecialAirHiLost_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.mt.SpecialHi.travelFrames -= 1;
        if (*fp).mv.mt.SpecialHi.travelFrames <= 0 {
            ftMt_SpecialAirHiLost_Enter(gobj);
        }
    }
}

pub fn ftMt_SpecialHiLost_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftMt_SpecialAirHiLost_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftMt_SpecialAirHiLost_Phys(_gobj: *mut HSD_GObj) {}

pub fn ftMt_SpecialHiLost_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp0 = fpp(gobj);
        if st::ft_80082708(gobj) == 0 {
            let env_flags = (*fp0).coll_data.env_flags;
            if env_flags & Collide_LeftWallMask != 0 || env_flags & Collide_RightWallMask != 0 {
                st::ftCommon_8007D60C(&mut *fp0);
                ftMt_SpecialAirHiLost_Enter(gobj);
                return;
            } else {
                ftMt_SpecialHi_GroundToAir(gobj);
            }
        } else {
            let env_flags = (*fp0).coll_data.env_flags;
            if env_flags & Collide_LeftWallMask != 0 || env_flags & Collide_RightWallMask != 0 {
                ftMt_SpecialHiLost_Enter(gobj);
            }
        }
    }
}

/// `ftMewtwo_SpecialHi_CheckTimer` (static).
pub fn ftMewtwo_SpecialHi_CheckTimer(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if (*fp).mv.mt.SpecialHi.unk4 as f32 >= (*a).x54_MEWTWO_TELEPORT_UNK2 {
            return true;
        }
        if ext::ftCo_8009A134(gobj) {
            return false;
        }
        true
    }
}

pub fn ftMt_SpecialAirHiLost_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp0 = fpp(gobj);
        let a = da(fp0);
        (*fp0).mv.mt.SpecialHi.unk4 += 1;

        if st::ft_CheckGroundAndLedge(gobj, facing_dir_int(fp0)) != 0 {
            if ftMewtwo_SpecialHi_CheckTimer(gobj) {
                ftMt_SpecialAirHi_AirToGround(gobj);
                return;
            }
        }

        if st::ftCliffCommon_80081298(gobj) != 0 {
            return;
        }

        ftCommon_HandleTeleportCollisions(gobj, fp0, (*a).x68_MEWTWO_TELEPORT_ANGLE_CLAMP, ftMt_SpecialAirHiLost_Enter);
    }
}

/// `ftMewtwo_SpecialHi_SetVars` (static inline): the Teleport-zoom entry bookkeeping.
pub fn ftMewtwo_SpecialHi_SetVars(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);

        (*fp).mv.mt.SpecialHi.travelFrames = (*a).x50_MEWTWO_TELEPORT_DURATION;

        (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as u8;
        (*fp).set_x2223_b4(1);

        ext::ftColl_8007B62C(gobj, 2);

        (*fp).set_invisible(1);

        ext::ft_PlaySFX(fp, 0x30DA1, 127, 64);
    }
}

/// Grounded Teleport Zoom Motion State handler.
pub fn ftMt_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);

        let mut stick_x = (*fp).input.lstick[0].x;
        let mut stick_y = (*fp).input.lstick[0].y;
        stick_x *= stick_x;
        stick_y *= stick_y;

        let mut sqrt_stick = sqrtf(stick_x + stick_y);
        if sqrt_stick > 1.0 {
            sqrt_stick = 1.0;
        }

        if !(sqrt_stick < (*a).x58_MEWTWO_TELEPORT_STICK_RANGE_MIN) {
            let stick_vec = Vec3 { x: (*fp).input.lstick[0].x, y: (*fp).input.lstick[0].y, z: 0.0 };

            if !(angle_xy(&(*fp).coll_data.floor.normal, &stick_vec)
                < core::f64::consts::FRAC_PI_2 as f32)
                && !ext::ftCo_8009A134(gobj)
            {
                st::ftCommon_UpdateFacing(&mut *fp);

                let vel = atan2f((*fp).input.lstick[0].y, (*fp).input.lstick[0].x * (*fp).facing_dir);

                (*fp).mv.mt.SpecialHi.stickX = stick_vec.x;
                (*fp).mv.mt.SpecialHi.stickY = stick_vec.y;
                (*fp).gr_vel = (*fp).facing_dir
                    * (fmadds((*a).x5C_MEWTWO_TELEPORT_MOMENTUM, sqrt_stick, (*a).x60_MEWTWO_TELEPORT_MOMENTUM_ADD)
                        * cosf(vel));

                st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialHiLost as i32, 0, 35.0, 1.0, 0.0, core::ptr::null_mut());
                st::ftAnim_8006EBA4(gobj);
                st::ftAnim_SetAnimRate(gobj, 0.0);
                ftMewtwo_SpecialHi_SetVars(gobj);
                return;
            }
        }

        st::ftCommon_8007D60C(&mut *fp);
        ftMt_SpecialAirHi_Enter(gobj);
    }
}

/// `stick_epsilon` (static float const = 0.001): the double literal rounded to f32.
const stick_epsilon: f32 = 0.001;

/// Aerial Teleport Zoom Motion State handler.
pub fn ftMt_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);

        let mut stick_x = (*fp).input.lstick[0].x;
        let mut stick_y = (*fp).input.lstick[0].y;
        stick_x *= stick_x;
        stick_y *= stick_y;

        let mut sqrt_stick = sqrtf(stick_x + stick_y);
        if sqrt_stick > 1.0 {
            sqrt_stick = 1.0;
        }

        let float_var: f32;
        if sqrt_stick > (*a).x58_MEWTWO_TELEPORT_STICK_RANGE_MIN {
            stick_x = stick_get_dir((*fp).input.lstick[0].x, 0.0);

            if stick_x > stick_epsilon {
                st::ftCommon_UpdateFacing(&mut *fp);
            }

            float_var = atan2f((*fp).input.lstick[0].y, (*fp).input.lstick[0].x * (*fp).facing_dir);
            (*fp).mv.mt.SpecialHi.stickX = (*fp).input.lstick[0].x;
            (*fp).mv.mt.SpecialHi.stickY = (*fp).input.lstick[0].y;
        } else {
            ext::ftCommon_8007DA24(fp);
            float_var = core::f64::consts::FRAC_PI_2 as f32;
            (*fp).mv.mt.SpecialHi.stickX = 0.0;
            (*fp).mv.mt.SpecialHi.stickY = 1.0;
            sqrt_stick = 1.0;
        }

        (*fp).self_vel.x = (*fp).facing_dir
            * (fmadds((*a).x5C_MEWTWO_TELEPORT_MOMENTUM, sqrt_stick, (*a).x60_MEWTWO_TELEPORT_MOMENTUM_ADD)
                * cosf(float_var));

        (*fp).self_vel.y =
            fmadds((*a).x5C_MEWTWO_TELEPORT_MOMENTUM, sqrt_stick, (*a).x60_MEWTWO_TELEPORT_MOMENTUM_ADD)
                * sinf(float_var);

        st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirHiLost as i32, 0, 35.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        st::ftAnim_SetAnimRate(gobj, 0.0);
        ftMewtwo_SpecialHi_SetVars(gobj);
    }
}

pub fn ftMt_SpecialHi_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftMt_SpecialAirHi_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftMt_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);

        if (*fp).cmd_vars[0] != 0 {
            melee_fighter::ftcommon::fall_basic(&mut *fp);
            melee_fighter::ftcommon::clamp_self_vel_x(
                &mut *fp,
                (*a).x64_MEWTWO_TELEPORT_DRIFT * (*fp).co_attrs.air_drift_max,
            );
        } else {
            let vel_y = (*fp).self_vel.y;
            (*fp).self_vel.y = vel_y - vel_y / 10.0;
            melee_fighter::ftcommon::calc_self_accel_deaccel_air(&mut *fp);
        }
    }
}

/// Grounded Teleport End Motion State handler.
pub fn ftMt_SpecialHiLost_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);

        st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);

        ftMewtwo_SpecialHiLost_SetVars(gobj);

        (*fp).gr_vel = (*fp).mv.mt.SpecialHi.groundVelX * (*a).x6C_MEWTWO_TELEPORT_MOMENTUM_END_MUL;
    }
}

/// Aerial Teleport End Motion State handler.
pub fn ftMt_SpecialAirHiLost_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);

        st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);

        ftMewtwo_SpecialHiLost_SetVars(gobj);

        (*fp).self_vel.x = (*fp).mv.mt.SpecialHi.velX * (*a).x6C_MEWTWO_TELEPORT_MOMENTUM_END_MUL;
        (*fp).self_vel.y = (*fp).mv.mt.SpecialHi.velY * (*a).x6C_MEWTWO_TELEPORT_MOMENTUM_END_MUL;
    }
}
