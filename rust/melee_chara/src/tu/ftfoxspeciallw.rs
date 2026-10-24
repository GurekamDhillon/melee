//! src/melee/ft/kinds/ftFox/ftfoxspeciallw.c: Fox / Falco Reflector / Fire Wing (down B).
//! One fused site, in `ftFx_SpecialLw_Turn` and `ftFox_SpecialLw_Turn_Inline`
//! (tools/rust-port/fma_sites.py): `-(MTXDegToRad(180 / x9C) - ftPartGetRotZ(fp, 0))` is
//! `-fmadds(180 / x9C, 0.01745329252f, -rotz)` (`melee_core::fp::fmadds`). Smoke-tested by
//! tests/tu_ftfoxspeciallw.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::ftcommon as fc;
use melee_types::*;

/// Already ported (rounds 2/3/5/6/12): re-exported under this TU.
pub use crate::chars::r12::ftFox::ftfoxspeciallw::{
    ftFx_SpecialAirLwEnd_IASA, ftFx_SpecialAirLwHit_IASA, ftFx_SpecialAirLwStart_IASA, ftFx_SpecialAirLwTurn_IASA,
    ftFx_SpecialLwEnd_IASA, ftFx_SpecialLwHit_IASA, ftFx_SpecialLwTurn_IASA,
};
pub use crate::chars::r2::ftFox::ftfoxspeciallw::{
    ftFx_SpecialAirLwEnd_Enter, ftFx_SpecialLwEnd_Enter, ftFx_SpecialLwEnd_GroundToAir, ftFx_SpecialLwEnd_Phys,
    ftFx_SpecialLwStart_GroundToAir, ftFx_SpecialLwStart_Phys,
};
pub use crate::chars::r3::ftFox::ftfoxspeciallw::{
    ftFox_SpecialLw_SetReflectVars, ftFx_SpecialLwEnd_Coll, ftFx_SpecialLwHit_Coll, ftFx_SpecialLwHit_GroundToAir,
    ftFx_SpecialLwHit_SetCall, ftFx_SpecialLwStart_Coll, ftFx_SpecialLwTurn_Coll, ftFx_SpecialLwTurn_GroundToAir,
};
pub use crate::chars::r5::ftFox::ftfoxspeciallw::{ftFx_SpecialAirLwEnd_Anim, ftFx_SpecialLwEnd_Anim};
pub use crate::chars::r6::ftFox::ftfoxspeciallw::{
    ftFx_SpecialAirLwEnd_AirToGround, ftFx_SpecialAirLwEnd_Coll, ftFx_SpecialAirLwHit_AirToGround,
    ftFx_SpecialAirLwHit_Coll, ftFx_SpecialAirLwStart_AirToGround, ftFx_SpecialAirLwStart_Coll,
    ftFx_SpecialAirLwTurn_Coll, ftFx_SpecialAirLwTurn_GroundToAir, ftFx_SpecialLwHit_Phys, ftFx_SpecialLwLoop_Phys,
    ftFx_SpecialLwTurn_Phys,
};

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
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut core::ffi::c_void]) -> *mut core::ffi::c_void);
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(joint: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(joint, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(joint as _, off as _, out as _); } return; },
                }
            }
    pub static mut LB_800119DC: Option<fn(*mut Vec3, i32, f32, f32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn lb_800119DC(pos: *mut Vec3, a: i32, b: f32, c: f32, d: f32)  {
                match unsafe { LB_800119DC } {
                    Some(f) => f(pos, a, b, c, d),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800119DC(pos as _, a as _, b as _, c as _, d as _); } return; },
                }
            }
    crate::port_hook!(FTCOLL_CREATEREFLECTHIT, ftColl_CreateReflectHit, crate::adapt::ftColl_CreateReflectHit, (gobj: *mut HSD_GObj, desc: *mut ReflectDesc, cb: HSD_GObjEvent));
    hook_or!(FIGHTER_SETEFFECTHITLAGCALLBACKS, Fighter_SetEffectHitlagCallbacks, (fp: *mut Fighter),
        |fp: *mut Fighter| unsafe { melee_fighter::tu::ftCo_AttackS4::Fighter_SetEffectHitlagCallbacks(&mut *fp) });
    hook_or!(FTPARTS_GETBONEINDEX, ftParts_GetBoneIndex, (fp: *mut Fighter, part: i32) -> i32,
        |fp: *mut Fighter, part: i32| unsafe { melee_fighter::hooks::ftparts_get_bone_index(&*fp, part) });
    hook_or!(FTPARTSETROTY, ftPartSetRotY, (fp: *mut Fighter, part: i32, v: f32),
        |fp: *mut Fighter, part: i32, v: f32| unsafe { melee_fighter::tu::ftparts::ftPartSetRotY(&*fp, part, v) });
    hook_or!(FTPARTGETROTZ, ftPartGetRotZ, (fp: *mut Fighter, part: i32) -> f32,
        |fp: *mut Fighter, part: i32| unsafe { melee_fighter::tu::ftparts::ftPartGetRotZ(&*fp, part) });
    hook_or!(FTCO_80099F1C, ftCo_80099F1C, (gobj: *mut HSD_GObj) -> bool, melee_fighter::tu::ftCo_Pass::ftCo_80099F1C);
    hook_or!(FTCO_8009A184, ftCo_8009A184, (gobj: *mut HSD_GObj, msid: i32, flags: u32, anim_start: f32),
        melee_fighter::tu::ftCo_Pass::ftCo_8009A184);
    hook_or!(FTCO_JUMP_CHECKINPUT, ftCo_Jump_CheckInput, (gobj: *mut HSD_GObj) -> bool, melee_fighter::tu::ftCo_Jump::ftCo_Jump_CheckInput);
    hook_or!(FTCO_800CB870, ftCo_800CB870, (gobj: *mut HSD_GObj) -> bool, melee_fighter::tu::ftCo_JumpAerial::ftCo_800CB870);
    hook_or!(FTCO_800C97A8, ftCo_800C97A8, (gobj: *mut HSD_GObj) -> bool, melee_fighter::tu::ftCo_Turn::ftCo_800C97A8);
}

/// `ftCommon_GroundAirColl_MF` (ftCommon/forward.h).
const ftCommon_GroundAirColl_MF: u32 = (Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;
const ftFx_MF_SpecialLw_Coll: u32 = ftCommon_GroundAirColl_MF | Ft_MF_KeepGfx as u32;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftFox_DatAttrs {
    (*fp).dat_attrs as *const ftFox_DatAttrs
}

/// `fp->input.held_buttons[0] & HSD_PAD_B`.
#[inline(always)]
unsafe fn held_b(fp: *mut Fighter) -> bool {
    (*fp).input.held_buttons[0] as i32 & HSD_PAD_B != 0
}

pub unsafe extern "C" fn cb_ftFx_SpecialLw_CreateLoopGFX(g: *mut HSD_GObj) {
    ftFx_SpecialLw_CreateLoopGFX(g);
}
pub unsafe extern "C" fn cb_ftFx_SpecialLw_CreateStartGFX(g: *mut HSD_GObj) {
    ftFx_SpecialLw_CreateStartGFX(g);
}
pub unsafe extern "C" fn cb_ftFx_SpecialLw_CreateReflectGFX(g: *mut HSD_GObj) {
    ftFx_SpecialLw_CreateReflectGFX(g);
}
pub unsafe extern "C" fn cb_ftFx_SpecialLwHit_Enter(g: *mut HSD_GObj) {
    ftFx_SpecialLwHit_Enter(g);
}

/// Shared body of the three effect-spawning accessory callbacks.
unsafe fn create_gfx(gobj: *mut HSD_GObj, gfx_id: i32) {
    let fp = fpp(gobj);
    if (*fp).x2219_b0() == 0 {
        // fp->parts[FtPart_HipN].joint (no ftParts_GetBoneIndex remap in this TU)
        let joint = (*(*fp).parts.add(FtPart_HipN as usize)).joint;
        ext::efSync_Spawn(gfx_id, gobj, &[joint as *mut core::ffi::c_void]);
        (*fp).set_x2219_b0(1);
    }
    ext::Fighter_SetEffectHitlagCallbacks(fp);
    (*fp).accessory4_cb = None;
}

pub fn ftFx_SpecialLw_CreateLoopGFX(gobj: *mut HSD_GObj) {
    unsafe { create_gfx(gobj, 1160) }
}

/// Create Reflector Start GFX.
pub fn ftFx_SpecialLw_CreateStartGFX(gobj: *mut HSD_GObj) {
    unsafe { create_gfx(gobj, 1161) }
}

pub fn ftFx_SpecialLw_CreateReflectGFX(gobj: *mut HSD_GObj) {
    unsafe { create_gfx(gobj, 1162) }
}

/// `ftFox_SpecialLw_SetVars` (static inline).
pub fn ftFox_SpecialLw_SetVars(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).mv.fx.SpecialLw.releaseLag = (*da).x98_FOX_REFLECTOR_RELEASE_LAG as i32;
        (*fp).mv.fx.SpecialLw.isRelease = 0;
        (*fp).cmd_vars[1] = 4;
        (*fp).mv.fx.SpecialLw.gravityDelay = (*da).xA4_FOX_REFLECTOR_GRAVITY_DELAY;
        (*fp).accessory4_cb = Some(cb_ftFx_SpecialLw_CreateStartGFX);
    }
}

pub fn ftFx_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialLwStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
    ftFox_SpecialLw_SetVars(gobj);
}

pub fn ftFx_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x /= (*da).xA8_FOX_REFLECTOR_MOMENTUM_PRESERVE_X;

        st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialAirLwStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftFox_SpecialLw_SetVars(gobj);
    }
}

/// `ftFx_SpecialLwLoop_Enter` (static): grounded Reflector Loop.
pub fn ftFx_SpecialLwLoop_Enter(gobj: *mut HSD_GObj) {
    st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialLwLoop as i32, Ft_MF_KeepGfx as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftFx_SpecialLw_CreateReflectHit(gobj);
}

/// `ftFx_SpecialAirLwLoop_Enter` (static): aerial Reflector Loop.
pub fn ftFx_SpecialAirLwLoop_Enter(gobj: *mut HSD_GObj) {
    st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialAirLwLoop as i32, Ft_MF_KeepGfx as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftFx_SpecialLw_CreateReflectHit(gobj);
}

/// Start Anim (grounded and aerial are the same code in the C).
unsafe fn start_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if !held_b(fp) {
        (*fp).mv.fx.SpecialLw.isRelease = 1;
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        if (*fp).ground_or_air == GA_Ground {
            ftFx_SpecialLwLoop_Enter(gobj);
        } else {
            ftFx_SpecialAirLwLoop_Enter(gobj);
        }
        st::ftCommon_8007DB24(gobj);
        (*fp).accessory4_cb = Some(cb_ftFx_SpecialLw_CreateLoopGFX);
    }
}

pub fn ftFx_SpecialLwStart_Anim(gobj: *mut HSD_GObj) {
    unsafe { start_anim(gobj) }
}

pub fn ftFx_SpecialAirLwStart_Anim(gobj: *mut HSD_GObj) {
    unsafe { start_anim(gobj) }
}

pub fn ftFx_SpecialLwStart_IASA(gobj: *mut HSD_GObj) {
    if ftFx_SpecialLwStart_CheckPass(gobj) {
        return;
    }
}

pub fn ftFx_SpecialLwStart_CheckPass(gobj: *mut HSD_GObj) -> bool {
    if ext::ftCo_80099F1C(gobj) {
        ftFx_SpecialLwStart_Pass(gobj);
        return true;
    }
    false
}

pub fn ftFx_SpecialLwStart_Pass(gobj: *mut HSD_GObj) {
    unsafe {
        ext::ftCo_8009A184(gobj, ftFx_MS_SpecialAirLwStart as i32, ftFx_MF_SpecialLw_Coll, (*fpp(gobj)).cur_anim_frame);
        ftFx_SpecialLw_CreateReflectHit(gobj);
    }
}

/// Gravity (after the delay) and quick air deceleration, shared by every aerial Reflector Phys.
unsafe fn air_lw_phys(fp: *mut Fighter) {
    let da = da(fp);
    if (*fp).mv.fx.SpecialLw.gravityDelay != 0 {
        (*fp).mv.fx.SpecialLw.gravityDelay -= 1;
    } else {
        fc::fall(&mut *fp, (*da).xAC_FOX_REFLECTOR_FALL_ACCEL, (*fp).co_attrs.terminal_velocity);
    }
    fc::calc_self_accel_deaccel_quick_air(&mut *fp);
}

pub fn ftFx_SpecialAirLwStart_Phys(gobj: *mut HSD_GObj) {
    unsafe { air_lw_phys(fpp(gobj)) }
}

/// Loop Anim (grounded and aerial are the same code in the C).
unsafe fn loop_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if !held_b(fp) {
        (*fp).mv.fx.SpecialLw.isRelease = 1;
    }
    if (*fp).mv.fx.SpecialLw.releaseLag > 0 {
        (*fp).mv.fx.SpecialLw.releaseLag -= 1;
    }
    if (*fp).mv.fx.SpecialLw.releaseLag <= 0 && (*fp).mv.fx.SpecialLw.isRelease != 0 {
        if (*fp).ground_or_air == GA_Ground {
            ftFx_SpecialLwEnd_Enter(gobj);
            return;
        }
        ftFx_SpecialAirLwEnd_Enter(gobj);
    }
}

pub fn ftFx_SpecialLwLoop_Anim(gobj: *mut HSD_GObj) {
    unsafe { loop_anim(gobj) }
}

pub fn ftFx_SpecialAirLwLoop_Anim(gobj: *mut HSD_GObj) {
    unsafe { loop_anim(gobj) }
}

pub fn ftFx_SpecialLwLoop_IASA(gobj: *mut HSD_GObj) {
    if !ftFx_SpecialLwTurn_Check(gobj) && !ext::ftCo_Jump_CheckInput(gobj) {
        if ftFx_SpecialLwLoop_CheckPass(gobj) {
            return;
        }
    }
}

pub fn ftFx_SpecialAirLwLoop_IASA(gobj: *mut HSD_GObj) {
    if !ftFx_SpecialLwTurn_Check(gobj) && ext::ftCo_800CB870(gobj) {
        return;
    }
}

/// `ftFx_SpecialLwLoop_CheckPass` (static): drop through a platform during the loop.
pub fn ftFx_SpecialLwLoop_CheckPass(gobj: *mut HSD_GObj) -> bool {
    if ext::ftCo_80099F1C(gobj) {
        ftFx_SpecialLwLoop_Pass(gobj);
        return true;
    }
    false
}

/// `ftFx_SpecialLwLoop_Pass` (static).
pub fn ftFx_SpecialLwLoop_Pass(gobj: *mut HSD_GObj) {
    unsafe {
        ext::ftCo_8009A184(gobj, ftFx_MS_SpecialAirLwLoop as i32, ftFx_MF_SpecialLw_Coll, (*fpp(gobj)).cur_anim_frame);
        ftFx_SpecialLw_CreateReflectHit(gobj);
    }
}

/// `ftFox_SpecialLw_InlinePhys` (static inline).
pub fn ftFox_SpecialLw_InlinePhys(gobj: *mut HSD_GObj) {
    unsafe { air_lw_phys(fpp(gobj)) }
}

pub fn ftFx_SpecialAirLwLoop_Phys(gobj: *mut HSD_GObj) {
    ftFox_SpecialLw_InlinePhys(gobj);
    st::ftColl_8007AEF8(gobj);
}

pub fn ftFx_SpecialLwLoop_Coll(gobj: *mut HSD_GObj) {
    if st::ft_80082708(gobj) == 0 {
        ftFx_SpecialLwLoop_GroundToAir(gobj);
    }
}

pub fn ftFx_SpecialAirLwLoop_Coll(gobj: *mut HSD_GObj) {
    if st::ft_80081D0C(gobj) != 0 {
        ftFx_SpecialAirLwLoop_AirToGround(gobj);
    }
}

/// `ftFx_SpecialLwLoop_GroundToAir` (static).
pub fn ftFx_SpecialLwLoop_GroundToAir(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftFx_MS_SpecialAirLwLoop as i32, ftFx_MF_SpecialLw_Coll);
        ftFx_SpecialLw_CreateReflectHit(gobj);
    }
}

/// `ftFx_SpecialAirLwLoop_AirToGround` (static).
pub fn ftFx_SpecialAirLwLoop_AirToGround(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftFx_MS_SpecialLwLoop as i32, ftFx_MF_SpecialLw_Coll);
        st::ftCommon_ClampAirDrift(&mut *fp);
        ftFx_SpecialLw_CreateReflectHit(gobj);
    }
}

/// `ftFx_SpecialLw_CreateReflectHit` (static): create the reflect bubble.
pub fn ftFx_SpecialLw_CreateReflectHit(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *mut ftFox_DatAttrs;
        ext::ftColl_CreateReflectHit(gobj, &mut (*da).xB0_FOX_REFLECTOR_REFLECTION, Some(cb_ftFx_SpecialLwHit_Enter));
    }
}

/// Turn body shared by `ftFx_SpecialLw_Turn` and `ftFox_SpecialLw_Turn_Inline`.
unsafe fn turn_body(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let da = da(fp);

    (*fp).mv.fx.SpecialLw.turnFrames -= 1;
    if (*fp).cmd_vars[0] == 0 && (*fp).mv.fx.SpecialLw.turnFrames as f32 <= (*da).x9C_FOX_REFLECTOR_TURN_FRAMES {
        (*fp).cmd_vars[0] = 1;
        (*fp).facing_dir = -(*fp).facing_dir;
    }
    // -(MTXDegToRad(180 / x9C) - ftPartGetRotZ(fp, 0)): the multiply-subtract is one fmadds.
    let deg = 180.0_f32 / (*da).x9C_FOX_REFLECTOR_TURN_FRAMES;
    let rz = ext::ftPartGetRotZ(fp, 0);
    ext::ftPartSetRotY(fp, 0, -fmadds(deg, 0.01745329252_f32, -rz));
}

/// `ftFx_SpecialLw_Turn` (static): Reflector Turn function.
pub fn ftFx_SpecialLw_Turn(gobj: *mut HSD_GObj) {
    unsafe { turn_body(gobj) }
}

/// `ftFox_SpecialLw_Turn_Inline` (static inline).
pub fn ftFox_SpecialLw_Turn_Inline(gobj: *mut HSD_GObj) {
    unsafe { turn_body(gobj) }
}

/// Turn Anim (grounded and aerial are the same code in the C).
unsafe fn turn_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if !held_b(fp) {
        (*fp).mv.fx.SpecialLw.isRelease = 1;
    }
    if (*fp).mv.fx.SpecialLw.releaseLag > 0 {
        (*fp).mv.fx.SpecialLw.releaseLag -= 1;
    }
    ftFox_SpecialLw_Turn_Inline(gobj);
    if (*fp).mv.fx.SpecialLw.turnFrames <= 0 {
        ftFx_SpecialLwHit_Check(gobj);
    }
}

pub fn ftFx_SpecialLwTurn_Anim(gobj: *mut HSD_GObj) {
    unsafe { turn_anim(gobj) }
}

pub fn ftFx_SpecialAirLwTurn_Anim(gobj: *mut HSD_GObj) {
    unsafe { turn_anim(gobj) }
}

pub fn ftFx_SpecialAirLwTurn_Phys(gobj: *mut HSD_GObj) {
    unsafe { air_lw_phys(fpp(gobj)) };
    st::ftColl_8007AEF8(gobj);
}

/// `ftFox_SpecialLwTurn_SetVarAll` (static inline).
pub fn ftFox_SpecialLwTurn_SetVarAll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).set_reflecting(1);
        (*fp).reflect_hit_cb = Some(cb_ftFx_SpecialLwHit_Enter);
        (*fp).mv.fx.SpecialLw.turnFrames = (*da).x9C_FOX_REFLECTOR_TURN_FRAMES as i32;
        (*fp).cmd_vars[0] = 0;
        ftFx_SpecialLw_Turn(gobj);
    }
}

/// Start a Reflector turn when the stick flips; true if it did.
pub fn ftFx_SpecialLwTurn_Check(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        if ext::ftCo_800C97A8(gobj) {
            if (*fp).ground_or_air == GA_Ground {
                st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialLwTurn as i32, Ft_MF_KeepGfx as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
                ftFox_SpecialLwTurn_SetVarAll(gobj);
            } else {
                st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialAirLwTurn as i32, Ft_MF_KeepGfx as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
                ftFox_SpecialLwTurn_SetVarAll(gobj);
            }
            (*fp).accessory4_cb = Some(cb_ftFx_SpecialLw_CreateLoopGFX);
            return true;
        }
        false
    }
}

/// `ftFox_SpecialLwHit_CreateReflectInline` (static inline).
pub fn ftFox_SpecialLwHit_CreateReflectInline(gobj: *mut HSD_GObj) {
    ftFx_SpecialLw_CreateReflectHit(gobj);
}

/// After a hit: end the move if the button was released, else back to the loop.
pub fn ftFx_SpecialLwHit_Check(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.fx.SpecialLw.releaseLag <= 0 && (*fp).mv.fx.SpecialLw.isRelease != 0 {
            if (*fp).ground_or_air == GA_Ground {
                ftFx_SpecialLwEnd_Enter(gobj);
            } else {
                ftFx_SpecialAirLwEnd_Enter(gobj);
            }
            return false;
        }
        if (*fp).ground_or_air == GA_Ground {
            st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialLwLoop as i32, Ft_MF_KeepGfx as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
            ftFox_SpecialLwHit_CreateReflectInline(gobj);
        } else {
            st::Fighter_ChangeMotionState(gobj, ftFx_MS_SpecialAirLwLoop as i32, Ft_MF_KeepGfx as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
            ftFox_SpecialLwHit_CreateReflectInline(gobj);
        }
        true
    }
}

/// Hit Anim (grounded and aerial are the same code in the C).
unsafe fn hit_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if !held_b(fp) {
        (*fp).mv.fx.SpecialLw.isRelease = 1;
    }
    if (*fp).mv.fx.SpecialLw.releaseLag > 0 {
        (*fp).mv.fx.SpecialLw.releaseLag -= 1;
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 && ftFx_SpecialLwHit_Check(gobj) {
        st::ftCommon_8007DB24(gobj);
        (*fp).accessory4_cb = Some(cb_ftFx_SpecialLw_CreateLoopGFX);
    }
}

pub fn ftFx_SpecialLwHit_Anim(gobj: *mut HSD_GObj) {
    unsafe { hit_anim(gobj) }
}

pub fn ftFx_SpecialAirLwHit_Anim(gobj: *mut HSD_GObj) {
    unsafe { hit_anim(gobj) }
}

pub fn ftFx_SpecialAirLwHit_Phys(gobj: *mut HSD_GObj) {
    unsafe { air_lw_phys(fpp(gobj)) };
    st::ftColl_8007AEF8(gobj);
}

/// Reflect Hit motion state handler (the `reflect_hit_cb`).
pub fn ftFx_SpecialLwHit_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

        (*fp).facing_dir = (*fp).ReflectAttr.x1A2C_reflectHitDirection;

        let bone = ext::ftParts_GetBoneIndex(fp, FtPart_HipN as i32);
        ext::lb_8000B1CC((*(*fp).parts.add(bone as usize)).joint, core::ptr::null(), &mut sp14);

        ext::lb_800119DC(&mut sp14, 120, 3.0, 0.1_f64 as f32, (core::f64::consts::PI / 3.0) as f32);

        let msid = if (*fp).ground_or_air == GA_Ground {
            ftFx_MS_SpecialLwHit
        } else {
            ftFx_MS_SpecialAirLwHit
        };

        st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftFx_SpecialLwHit_SetCall(gobj);

        (*fp).accessory4_cb = Some(cb_ftFx_SpecialLw_CreateReflectGFX);
    }
}

pub fn ftFx_SpecialAirLwEnd_Phys(gobj: *mut HSD_GObj) {
    unsafe { air_lw_phys(fpp(gobj)) }
}
