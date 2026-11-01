//! src/melee/ft/kinds/ftFox/ftfoxspecialhi.c: Fox / Falco Fire Fox / Fire Bird (up B).
//! Fused sites (tools/rust-port/fma_sites.py): the two in `ftFx_SpecialAirHi_Phys`
//! (`-((facing * (rev * cosf(r))) - vx)` = `-fmadds(facing, rev * cosf(r), -vx)`, and the y twin),
//! written with `melee_core::fp::fmadds`. sinf/cosf are the MSL ones (`melee_runtime::gekko`).
//! Smoke-tested by tests/tu_ftfoxspecialhi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da as da_g;
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
#[allow(unused_imports)]
use melee_types::{ftFox_DatAttrs};
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::ftcommon as fc;
use melee_runtime::gekko::{cosf, sinf};
use melee_runtime::lbtrigf::atan2f;
use melee_types::*;

/// Already ported (rounds 2/3/6/7/11/12): re-exported under this TU.







/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// Like `hook!`, but an unset hook runs the native port `$port` (a ported TU elsewhere).
    macro_rules! hook_or {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:expr) => {
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
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing pointers as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    hook_or!(FIGHTER_SETEFFECTHITLAGCALLBACKS, Fighter_SetEffectHitlagCallbacks, (fp: *mut Fighter),
        |fp: *mut Fighter| unsafe { melee_fighter::tu::ftCo_AttackS4::Fighter_SetEffectHitlagCallbacks(&mut *fp) });
    hook_or!(FTPARTS_GETBONEINDEX, ftParts_GetBoneIndex, (fp: *mut Fighter, part: i32) -> i32,
        |fp: *mut Fighter, part: i32| unsafe { melee_fighter::hooks::ftparts_get_bone_index(&*fp, part) });
    hook_or!(FTPARTSETROTX, ftPartSetRotX, (fp: *mut Fighter, part: i32, v: f32),
        |fp: *mut Fighter, part: i32, v: f32| unsafe { melee_fighter::tu::ftparts::ftPartSetRotX(&*fp, part, v) });
    hook_or!(FTCO_8009A134, ftCo_8009A134, (gobj: *mut HSD_GObj) -> bool, melee_fighter::tu::ftCo_Pass::ftCo_8009A134);
}

const HALF_PI32: f32 = 1.5707963705062866_f32;
/// `Collide_FloorMask` (mp/forward.h:111).
const Collide_FloorMask: i32 = Collide_FloorPush | Collide_FloorHug;
/// `Collide_CeilingMask` (mp/forward.h:107).
const Collide_CeilingMask: i32 = Collide_CeilingPush | Collide_CeilingHug;
/// `0.01745329238474369f` (f32).
const DEG_TO_RAD: f32 = 0.01745329238474369_f32;
/// `FTFOX_SPECIALHI_COLL_FLAG`.
const FTFOX_SPECIALHI_COLL_FLAG: u32 = (Ft_MF_KeepGfx
    | Ft_MF_SkipMatAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipColAnim
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftFox_DatAttrs {
    (*fp).dat_attrs as *const ftFox_DatAttrs
}

/// `lbVector_AngleXY(a, b)` (lb/lbvector.c, verified in melee_runtime) over the generated `Vec`.
#[inline(always)]
fn angle_xy(a: &Vec3, b: &Vec3) -> f32 {
    // SAFETY: both are `{ f32 x, y, z }` repr(C) records.
    unsafe {
        melee_runtime::lbvector::angle_xy(
            &*(a as *const Vec3 as *const melee_core::vec::Vec3),
            &*(b as *const Vec3 as *const melee_core::vec::Vec3),
        )
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

/// `ftCommon_8007F76C` as the `x21F8` callback entry.
pub unsafe extern "C" fn cb_ftCommon_8007F76C(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftCommon_8007F76C");
    melee_fighter::ftcommon2::ftcommon_8007f76c(&mut *fpp(g));
}
pub unsafe extern "C" fn cb_ftFx_SpecialHi_CreateLaunchGFX(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftFx_SpecialHi_CreateLaunchGFX");
    ftFx_SpecialHi_CreateLaunchGFX(g);
}
pub unsafe extern "C" fn cb_ftFx_SpecialHi_CreateChargeGFX(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftFx_SpecialHi_CreateChargeGFX");
    ftFx_SpecialHi_CreateChargeGFX(g);
}

pub fn ftFx_SpecialHi_CreateLaunchGFX(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHi_CreateLaunchGFX");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            let bone = ext::ftParts_GetBoneIndex(fp, FtPart_HipN as i32);
            ext::efSync_Spawn(1164, gobj, &[(*(*fp).parts.add(bone as usize)).joint as *mut core::ffi::c_void]);
            (*fp).set_x2219_b0(1);
        }
        ext::Fighter_SetEffectHitlagCallbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

pub fn ftFx_SpecialHi_CreateChargeGFX(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHi_CreateChargeGFX");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            let bone = ext::ftParts_GetBoneIndex(fp, FtPart_TransN as i32);
            ext::efSync_Spawn(1163, gobj, &[(*(*fp).parts.add(bone as usize)).joint as *mut core::ffi::c_void]);
            (*fp).set_x2219_b0(1);
        }
        ext::Fighter_SetEffectHitlagCallbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

pub fn ftFx_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).mv.fx.SpecialHi.gravityDelay = (*da).x54_FOX_FIREFOX_GRAVITY_DELAY as i32;
        (*fp).gr_vel /= (*da).x58_FOX_FIREFOX_VEL_X;

        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialHiHold as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);

        (*fp).accessory4_cb = Some(cb_ftFx_SpecialHi_CreateChargeGFX);
    }
}

pub fn ftFx_SpecialAirHiStart_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirHiStart_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).mv.fx.SpecialHi.gravityDelay = (*da).x54_FOX_FIREFOX_GRAVITY_DELAY as i32;
        (*fp).self_vel.x /= (*da).x58_FOX_FIREFOX_VEL_X;
        (*fp).self_vel.y = 0.0;

        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialHiHoldAir as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);

        (*fp).accessory4_cb = Some(cb_ftFx_SpecialHi_CreateChargeGFX);
    }
}

/// `ftFox_SpecialHi_RotateModel` (static).
pub fn ftFox_SpecialHi_RotateModel(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFox_SpecialHi_RotateModel");
    unsafe {
        let fp = fpp(gobj);
        let bone = ext::ftParts_GetBoneIndex(fp, FtPart_XRotN as i32);
        // (2 * (float) M_PI) - rotateModel
        ext::ftPartSetRotX(fp, bone, (2.0_f32 * (core::f64::consts::PI as f32)) - (*fp).mv.fx.SpecialHi.rotateModel);
    }
}

pub fn ftFx_SpecialHiHold_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiHold_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).ground_or_air == GA_Air {
                ftFx_SpecialAirHi_Enter(gobj);
                return;
            }
            ftFx_SpecialAirHi_AirToGround(gobj);
        }
    }
}

pub fn ftFx_SpecialHiHoldAir_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiHoldAir_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).ground_or_air == GA_Air {
                ftFx_SpecialAirHi_Enter(gobj);
                return;
            }
            ftFx_SpecialAirHi_AirToGround(gobj);
        }
    }
}

/// Gobj entry for the fighter-only port `chars::ftFox::ftfoxspecialhi::ftFx_SpecialHiHoldAir_Phys`.
pub fn ftFx_SpecialHiHoldAir_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiHoldAir_Phys");
    unsafe { crate::chars::ftFox::ftfoxspecialhi::ftFx_SpecialHiHoldAir_Phys(&mut *fpp(gobj)) }
}

pub fn ftFx_SpecialHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHi_Anim");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.fx.SpecialHi.travelFrames -= 1;
        if (*fp).mv.fx.SpecialHi.travelFrames <= 0 {
            if (*fp).ground_or_air == GA_Air {
                ftFx_SpecialHiLanding_GroundToAir(gobj);
                return;
            }
            ftFx_SpecialHiFall_AirToGround(gobj);
        }
    }
}

pub fn ftFx_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirHi_Anim");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.fx.SpecialHi.travelFrames -= 1;
        if (*fp).mv.fx.SpecialHi.travelFrames <= 0 {
            if (*fp).ground_or_air == GA_Air {
                ftFx_SpecialHiLanding_GroundToAir(gobj);
                return;
            }
            ftFx_SpecialHiFall_AirToGround(gobj);
        }
    }
}

pub fn ftFx_SpecialHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHi_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).mv.fx.SpecialHi.unk += 1;
        if (*fp).mv.fx.SpecialHi.unk as f32 >= (*da).x70_FOX_FIREFOX_DURATION_END {
            fc::calc_ground_accel_deaccel(&mut *fp, (*da).x78_FOX_FIREFOX_REVERSE_ACCEL);
        }
        fc::set_self_movement_from_grounded_movement(&mut *fp);
    }
}

pub fn ftFx_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirHi_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).mv.fx.SpecialHi.unk += 1;
        if (*fp).mv.fx.SpecialHi.unk as f32 >= (*da).x70_FOX_FIREFOX_DURATION_END {
            let rot = (*fp).mv.fx.SpecialHi.rotateModel;
            let rev = (*da).x78_FOX_FIREFOX_REVERSE_ACCEL;
            // -((facing * (rev * cosf(rot))) - vx), the multiply-subtract fused (fmadds).
            (*fp).self_vel.x = -fmadds((*fp).facing_dir, rev * cosf(rot), -(*fp).self_vel.x);
            // -((rev * sinf(rot)) - vy)
            (*fp).self_vel.y = -fmadds(rev, sinf(rot), -(*fp).self_vel.y);
        }
    }
}

pub fn ftFx_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHi_Coll");
    unsafe {
        let fp = fpp(gobj);

        (*fp).mv.fx.SpecialHi.unk2 += 1;

        if st::ft_80082708(gobj) == 0 {
            ftFx_SpecialHi_GroundToAir(gobj);
            return;
        }
        if (*fp).coll_data.env_flags & Collide_FloorMask != 0 {
            (*fp).mv.fx.SpecialHi.rotateModel =
                atan2f(-(*fp).coll_data.floor.normal.x * (*fp).facing_dir, (*fp).coll_data.floor.normal.y);
            ftFox_SpecialHi_RotateModel(gobj);
        }
    }
}

/// `ftFox_SpecialHi_IsBound` (static inline).
pub fn ftFox_SpecialHi_IsBound(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftFox_SpecialHi_IsBound");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).mv.fx.SpecialHi.unk2 >= (*da).x6C_FOX_FIREFOX_BOUNCE_VAR {
            true
        } else if ext::ftCo_8009A134(gobj) {
            false
        } else {
            true
        }
    }
}

/// The `facingDir:` tail of `ftFx_SpecialAirHi_Coll`: face the travel direction, aim the model.
unsafe fn air_hi_coll_facing(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    let facing_dir = if (*fp).self_vel.x >= 0.0 { 1.0_f32 } else { -1.0_f32 };
    (*fp).facing_dir = facing_dir;
    (*fp).mv.fx.SpecialHi.rotateModel = atan2f((*fp).self_vel.y, (*fp).self_vel.x * (*fp).facing_dir);
    ftFox_SpecialHi_RotateModel(gobj);
}

pub fn ftFx_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        if st::ft_CheckGroundAndLedge(gobj, CLIFFCATCH_BOTH) != 0 && ftFox_SpecialHi_IsBound(gobj) {
            let env_flags = (*fp).coll_data.env_flags;
            if env_flags & Collide_FloorMask == 0
                || !(angle_xy(&(*fp).coll_data.floor.normal, &(*fp).self_vel)
                    < (DEG_TO_RAD * (90.0_f32 + (*da).x94_FOX_FIREFOX_BOUND_ANGLE)))
            {
                ftFx_SpecialHiBound_Enter(gobj);
            } else {
                air_hi_coll_facing(gobj, fp);
            }
            return;
        }

        if st::ftCliffCommon_80081298(gobj) == 0 {
            let env_flags = (*fp).coll_data.env_flags;
            let var = if env_flags & Collide_CeilingMask != 0 {
                angle_xy(&(*fp).coll_data.ceiling.normal, &(*fp).self_vel)
            } else if env_flags & Collide_LeftWallMask != 0 {
                angle_xy(&(*fp).coll_data.left_facing_wall.normal, &(*fp).self_vel)
            } else if env_flags & Collide_RightWallMask != 0 {
                angle_xy(&(*fp).coll_data.right_facing_wall.normal, &(*fp).self_vel)
            } else {
                return;
            };
            if var < (DEG_TO_RAD * (90.0_f32 + (*da).x94_FOX_FIREFOX_BOUND_ANGLE)) {
                air_hi_coll_facing(gobj, fp);
            }
        }
    }
}

pub fn ftFx_SpecialAirHi_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirHi_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        let mut stick_y = (*fp).input.lstick[0].y;
        let da = da(fp);

        if stick_y < 0.0 {
            stick_y = -stick_y;
        }
        let temp_stick = (*fp).input.lstick[0].x;
        let stick_x = stick_get_dir((*fp).input.lstick[0].x, 0.0);

        if !((stick_x + stick_y) < (*da).x64_FOX_FIREFOX_DIRECTION_STICK_RANGE_MIN) {
            let sp20 = Vec3 { x: temp_stick, y: (*fp).input.lstick[0].y, z: 0.0 };

            if !(angle_xy(&(*fp).coll_data.floor.normal, &sp20) < HALF_PI32)
                && !ext::ftCo_8009A134(gobj)
            {
                st::ftCommon_UpdateFacing(&mut *fp);

                st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());

                let temp_attrs = (*fp).dat_attrs as *const ftFox_DatAttrs;
                (*fp).set_x2223_b4(1);

                (*fp).mv.fx.SpecialHi.travelFrames = (*temp_attrs).x68_FOX_FIREFOX_DURATION as i32;

                (*fp).mv.fx.SpecialHi.unk = 0;
                (*fp).mv.fx.SpecialHi.unk2 = 0;

                (*fp).gr_vel = (*da).x74_FOX_FIREFOX_SPEED * (*fp).facing_dir;

                (*fp).mv.fx.SpecialHi.rotateModel =
                    atan2f(-(*fp).coll_data.floor.normal.x * (*fp).facing_dir, (*fp).coll_data.floor.normal.y);

                ftFox_SpecialHi_RotateModel(gobj);
                (*fp).accessory4_cb = Some(cb_ftFx_SpecialHi_CreateLaunchGFX);
                (*fp).x21F8 = Some(cb_ftCommon_8007F76C);
                return;
            }
        }
        st::ftCommon_8007D60C(&mut *fp);
        ftFx_SpecialAirHi_Enter(gobj);
    }
}

pub fn ftFx_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        let stick_y = stick_get_dir((*fp).input.lstick[0].y, 0.0);
        let mut temp_stick = (*fp).input.lstick[0].x;
        let stick_x = stick_get_dir(temp_stick, 0.0);

        if (stick_x + stick_y) >= (*da).x64_FOX_FIREFOX_DIRECTION_STICK_RANGE_MIN {
            if temp_stick < 0.0 {
                temp_stick = -temp_stick;
            }
            if temp_stick > (*da).x88_FOX_FIREFOX_FACING_STICK_RANGE_MIN {
                st::ftCommon_UpdateFacing(&mut *fp);
            }
            (*fp).mv.fx.SpecialHi.rotateModel =
                atan2f((*fp).input.lstick[0].y, (*fp).input.lstick[0].x * (*fp).facing_dir);
        } else {
            (*fp).mv.fx.SpecialHi.rotateModel = HALF_PI32;
        }

        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialAirHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());

        let temp_attrs = (*fp).dat_attrs as *const ftFox_DatAttrs;
        (*fp).set_x2223_b4(1);

        (*fp).mv.fx.SpecialHi.travelFrames = (*temp_attrs).x68_FOX_FIREFOX_DURATION as i32;
        (*fp).mv.fx.SpecialHi.unk = 0;
        (*fp).mv.fx.SpecialHi.unk2 = 0;

        let rot = (*fp).mv.fx.SpecialHi.rotateModel;
        (*fp).self_vel.x = (*fp).facing_dir * ((*da).x74_FOX_FIREFOX_SPEED * cosf(rot));
        (*fp).self_vel.y = (*da).x74_FOX_FIREFOX_SPEED * sinf(rot);
        ftFox_SpecialHi_RotateModel(gobj);
        (*fp).x21F8 = Some(cb_ftCommon_8007F76C);
        (*fp).accessory4_cb = Some(cb_ftFx_SpecialHi_CreateLaunchGFX);
        (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as u8;
    }
}

pub fn ftFx_SpecialHiLanding_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiLanding_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        fc::calc_ground_accel_deaccel(&mut *fp, (*da).x7C_FOX_FIREFOX_GROUND_MOMENTUM_END);
        fc::set_self_movement_from_grounded_movement(&mut *fp);
    }
}

pub fn ftFx_SpecialHiFall_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiFall_Coll");
    if st::ft_CheckGroundAndLedge(gobj, CLIFFCATCH_BOTH) != 0 {
        ftFx_SpecialHiFall_Enter(gobj);
        return;
    }
    if st::ftCliffCommon_80081298(gobj) != 0 {
        return;
    }
}

pub fn ftFx_SpecialHiFall_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiFall_Enter");
    unsafe {
        st::ftCommon_8007D7FC(&mut *fpp(gobj));
        st::Fighter_ChangeMotionState(
            gobj,
            ftFx_MS_SpecialHiLanding as i32,
            (Ft_MF_SkipColAnim | Ft_MF_UpdateCmd) as u32,
            13.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftFx_SpecialHiFall_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiFall_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007DB24(gobj);
        if (*fp).ground_or_air == GA_Air {
            st::ftCommon_8007D7FC(&mut *fp);
        }
        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialHiLanding as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).x21F8 = Some(cb_ftCommon_8007F76C);
    }
}

pub fn ftFx_SpecialHiLanding_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiLanding_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007DB24(gobj);
        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialHiFall as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).x21F8 = Some(cb_ftCommon_8007F76C);
    }
}

pub fn ftFx_SpecialHiBound_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiBound_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        if (*fp).cmd_vars[0] != 0 && (*fp).ground_or_air == GA_Air {
            st::ftCo_80096900(
                gobj,
                1,
                0,
                1,
                (*da).x8C_FOX_FIREFOX_FREEFALL_MOBILITY,
                (*da).x90_FOX_FIREFOX_LANDING_LAG,
            );
            (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as u8;
            return;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).ground_or_air == GA_Air {
                st::ftCo_80096900(
                    gobj,
                    1,
                    0,
                    1,
                    (*da).x8C_FOX_FIREFOX_FREEFALL_MOBILITY,
                    (*da).x90_FOX_FIREFOX_LANDING_LAG,
                );
                (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as u8;
                return;
            }
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftFx_SpecialHiBound_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiBound_Phys");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Air {
            st::ft_800851C0(gobj);
            fc::calc_self_accel_deaccel_quick_air(&mut *fp);
            return;
        }
        st::ft_80084F3C(gobj);
    }
}

/// `ftFox_SpecialHiBound_SetVars` (static inline): the rebound spark, aimed along the floor.
pub fn ftFox_SpecialHiBound_SetVars(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFox_SpecialHiBound_SetVars");
    unsafe {
        let fp = fpp(gobj);
        let mut f: f32 = if (*fp).coll_data.env_flags & Collide_FloorMask != 0 {
            -atan2f((*fp).coll_data.floor.normal.x, (*fp).coll_data.floor.normal.y)
        } else {
            0.0
        };
        ext::efSync_Spawn(
            1030,
            gobj,
            &[
                &mut (*fp).cur_pos as *mut Vec3 as *mut core::ffi::c_void,
                &mut f as *mut f32 as *mut core::ffi::c_void,
            ],
        );
        (*fp).set_x2219_b0(1);
        ext::Fighter_SetEffectHitlagCallbacks(fp);
    }
}

pub fn ftFx_SpecialHiBound_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiBound_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialHiBound as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).x21F8 = Some(cb_ftCommon_8007F76C);
        (*fp).self_vel.x *= (*da).x84_FOX_FIREFOX_BOUND_VEL_X;
        (*fp).cmd_vars[0] = 0;
        ftFox_SpecialHiBound_SetVars(gobj);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftFx_SpecialHiHoldAir_Coll` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiHoldAir_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiHoldAir_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_CheckGroundAndLedge(gobj, (if (*fp).facing_dir < 0.0f32 { -1i32 } else { 1i32 })) != 0) {
            crate::chars::r6::ftFox::ftfoxspecialhi::ftFx_SpecialHiHoldAir_AirToGround(gobj);
            return;
        }
        if (st::ftCliffCommon_80081298(gobj) != 0) {
            return;
        }
    }
}

/// `ftFx_SpecialHiBound_Coll` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiBound_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiBound_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (((*fp).ground_or_air as i32) == 1i32 /* GA_Air */) {
            if (st::ft_CheckGroundAndLedge(gobj, (if (*fp).facing_dir < 0.0f32 { -1i32 } else { 1i32 })) != 0) {
                st::ftCommon_8007D7FC(&mut *fp);
                return;
            }
            if (st::ftCliffCommon_80081298(gobj) != 0) {
                return;
            }
        } else {
            st::ft_80084104(gobj);
        }
    }
}

/// `ftFx_SpecialHiHold_IASA` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiHold_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiHold_IASA");
    unsafe {
        return;
    }
}

/// `ftFx_SpecialHiHoldAir_IASA` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiHoldAir_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiHoldAir_IASA");
    unsafe {
        return;
    }
}

/// `ftFx_SpecialHi_IASA` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHi_IASA");
    unsafe {
        return;
    }
}

/// `ftFx_SpecialAirHi_IASA` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialAirHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialAirHi_IASA");
    unsafe {
        return;
    }
}

/// `ftFx_SpecialHiLanding_IASA` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiLanding_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiLanding_IASA");
    unsafe {
        return;
    }
}

/// `ftFx_SpecialHiFall_IASA` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiFall_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiFall_IASA");
    unsafe {
        return;
    }
}

/// `ftFx_SpecialHiBound_IASA` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiBound_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiBound_IASA");
    unsafe {
        return;
    }
}

/// `ftFx_SpecialHiHold_Phys` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiHold_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiHold_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftFx_SpecialHiLanding_Anim` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiLanding_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiLanding_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftFx_SpecialHiFall_Phys` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiFall_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiFall_Phys");
    st::ft_80084DB0(gobj);
}

/// `ftFx_SpecialHiHold_Coll` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiHold_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiHold_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 /* false */ {
            crate::chars::r3::ftFox::ftfoxspecialhi::ftFx_SpecialHiHold_GroundToAir(gobj);
        }
    }
}

/// `ftFx_SpecialHiHold_GroundToAir` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiHold_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiHold_GroundToAir");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_8007D60C(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 354 /* ftFx_MS_SpecialHiHoldAir */, 0xC4C5082u32 /* FTFOX_SPECIALHI_COLL_FLAG */, (*fp).cur_anim_frame, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(crate::chars::cb::ftfoxspecialhi__ftFx_SpecialHi_CreateChargeGFX);
    }
}

/// `ftFx_SpecialHi_GroundToAir` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHi_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHi_GroundToAir");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_8007D60C(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 356 /* ftFx_MS_SpecialAirHi */, 0xC4C508Au32 /* (Ft_MF_SkipHit | FTFOX_SPECIALHI_COLL_FLAG) */, (*fp).cur_anim_frame, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).set_x2223_b4(1);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftfoxspecialhi__ftFx_SpecialHi_CreateLaunchGFX);
    }
}

/// `ftFx_SpecialHiHoldAir_AirToGround` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiHoldAir_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiHoldAir_AirToGround");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 353 /* ftFx_MS_SpecialHiHold */, 0xC4C5082u32 /* FTFOX_SPECIALHI_COLL_FLAG */);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftfoxspecialhi__ftFx_SpecialHi_CreateChargeGFX);
        st::ftCommon_ClampAirDrift(&mut *fp);
    }
}

/// `ftFx_SpecialHiFall_Anim` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiFall_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiFall_Anim");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            st::ftCo_80096900(gobj, 1, 0, 1 /* true */, ((*((*fp).dat_attrs as *const melee_types::ftFox_DatAttrs)).x8C_FOX_FIREFOX_FREEFALL_MOBILITY as f32), ((*((*fp).dat_attrs as *const melee_types::ftFox_DatAttrs)).x90_FOX_FIREFOX_LANDING_LAG as f32));
        }
    }
}

/// `ftFx_SpecialHiLanding_Coll` (src/melee/ft/kinds/ftFox/ftfoxspecialhi.c).
pub fn ftFx_SpecialHiLanding_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftFx_SpecialHiLanding_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80082708(gobj) == 0 /* false */) {
            st::ftCo_80096900(gobj, 1, 0, 1 /* true */, ((*((*fp).dat_attrs as *const melee_types::ftFox_DatAttrs)).x8C_FOX_FIREFOX_FREEFALL_MOBILITY as f32), ((*((*fp).dat_attrs as *const melee_types::ftFox_DatAttrs)).x90_FOX_FIREFOX_LANDING_LAG as f32));
        }
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftFx_SpecialHiHoldAir_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftFx_SpecialHiHoldAir_Phys_fp");
    let a: &ftFox_DatAttrs = da_g(fp);
    unsafe {
        if fp.mv.fx.SpecialHi.gravityDelay != 0 {
            fp.mv.fx.SpecialHi.gravityDelay -= 1;
        } else {
            co::fall(fp, a.x60_FOX_FIREFOX_FALL_ACCEL, fp.co_attrs.terminal_velocity);
        }
    }
    co::calc_self_accel_deaccel(fp, a.x5C_FOX_FIREFOX_AIR_MOMENTUM_PRESERVE_X);
}
