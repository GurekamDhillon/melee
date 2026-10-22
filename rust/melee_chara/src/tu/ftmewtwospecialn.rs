//! src/melee/ft/kinds/ftMewtwo/ftmewtwospecialn.c: Shadow Ball (neutral B). No fused sites
//! (tools/rust-port/fma_sites.py); the float conversions are the C's (`(float)(u32)` of the
//! charge, `(s32)` of the cycles via `cabi::f2i32`). Smoke-tested through recording hooks by
//! tests/tu_ftmewtwospecialn.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_runtime::cabi::f2i32;
use melee_runtime::lbtrigf::atan2f;
use melee_types::*;

/// Already ported (rounds 2/3/4/12): re-exported under this TU.
pub use crate::chars::r12::ftMewtwo::ftmewtwospecialn::{
    ftMt_SpecialAirNCancel_IASA, ftMt_SpecialAirNEnd_IASA, ftMt_SpecialAirNStart_IASA,
    ftMt_SpecialNCancel_IASA, ftMt_SpecialNEnd_IASA, ftMt_SpecialNStart_IASA,
};
pub use crate::chars::r2::ftMewtwo::ftmewtwospecialn::{
    ftMt_SpecialAirNCancel_Phys, ftMt_SpecialAirNEnd_Phys, ftMt_SpecialAirNLoopFull_Phys,
    ftMt_SpecialAirNLoop_Phys, ftMt_SpecialAirNStart_Phys, ftMt_SpecialNCancel_Phys,
    ftMt_SpecialNEnd_Phys, ftMt_SpecialNLoopFull_Phys, ftMt_SpecialNLoop_Phys,
    ftMt_SpecialNStart_Phys,
};
pub use crate::chars::r3::ftMewtwo::ftmewtwospecialn::ftMewtwo_SpecialN_SetCall;
pub use crate::chars::r4::ftMewtwo::ftmewtwospecialn::{
    ftMt_SpecialAirNCancel_Coll, ftMt_SpecialAirNEnd_Coll, ftMt_SpecialAirNLoopFull_Coll,
    ftMt_SpecialAirNLoop_Coll, ftMt_SpecialAirNStart_Coll, ftMt_SpecialNCancel_Coll,
    ftMt_SpecialNEnd_Coll, ftMt_SpecialNLoopFull_Coll, ftMt_SpecialNLoop_Coll,
    ftMt_SpecialNStart_Coll,
};

/// `FTMEWTWO_SPECIALN_ACTION_FLAG`: SkipMatAnim | KeepSfx | UpdateCmd | SkipItemVis | Unk19.
pub const FTMEWTWO_SPECIALN_ACTION_FLAG: u32 =
    (Ft_MF_SkipMatAnim | Ft_MF_KeepSfx | Ft_MF_UpdateCmd | Ft_MF_SkipItemVis | Ft_MF_Unk19) as u32;

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
    hook!(LB_8000B1CC, lb_8000B1CC, (joint: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3));
    hook!(IT_802C573C, it_802C573C, (item: *mut HSD_GObj));
    hook!(IT_802C5000, it_802C5000,
          (owner: *mut HSD_GObj, pos: *const Vec3, bone: i32, kind: i32, facing_dir: f32) -> *mut HSD_GObj);
    hook!(IT_802C53F0, it_802C53F0,
          (item: *mut HSD_GObj, pos: *const Vec3, angle: f32, charge: f32, cycles: f32));
    hook!(IT_802C519C, it_802C519C,
          (owner: *mut HSD_GObj, pos: *const Vec3, kind: i32, cycles: f32, angle: f32, facing_dir: f32));
    fn ftco_800bffd0_port(fp: *mut Fighter, a: i32, b: i32) -> bool {
        melee_fighter::tu::ftcolanim::ftco_800bffd0(unsafe { &mut *fp }, a, b)
    }
    hook_or!(FTCO_800BFFD0, ftCo_800BFFD0, ftco_800bffd0_port, (fp: *mut Fighter, a: i32, b: i32) -> bool);
    fn ftco_800bffac_port(fp: *mut Fighter) {
        melee_fighter::tu::ftcolanim::ftco_800bffac(unsafe { &mut *fp })
    }
    hook_or!(FTCO_800BFFAC, ftCo_800BFFAC, ftco_800bffac_port, (fp: *mut Fighter));
    fn play_sfx_port(fp: *mut Fighter, id: i32, vol: i32, pan: i32) {
        melee_fighter::hooks::play_sfx(unsafe { &mut *fp }, id, vol, pan)
    }
    hook_or!(FT_PLAYSFX, ft_PlaySFX, play_sfx_port, (fp: *mut Fighter, id: i32, vol: i32, pan: i32));
    fn ft_80088510_port(fp: *mut Fighter, id: i32, vol: i32, pan: i32) {
        melee_fighter::ft0881::ft_80088510(unsafe { &mut *fp }, id, vol as u8, pan as u8)
    }
    hook_or!(FT_80088510, ft_80088510, ft_80088510_port, (fp: *mut Fighter, id: i32, vol: i32, pan: i32));
    fn ftco_8009917c_port(gobj: *mut HSD_GObj) -> bool {
        melee_fighter::tu::ftCo_Escape::ftCo_8009917C(gobj)
    }
    hook_or!(FTCO_8009917C, ftCo_8009917C, ftco_8009917c_port, (gobj: *mut HSD_GObj) -> bool);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftMewtwoAttributes {
    (*fp).dat_attrs as *mut ftMewtwoAttributes
}

/// `ftMt_SpecialN_GetChargeLevel`: -1 unless Mewtwo is holding a Shadow Ball, else 0 with the
/// charge and the cycle count written out.
pub fn ftMt_SpecialN_GetChargeLevel(gobj: *mut HSD_GObj, charge_level: &mut i32, charge_cycles: &mut i32) -> i32 {
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            let a = da(fp);
            if (*fp).u.mt.x2230_shadowHeldGObj.is_null() {
                return -1;
            }
            *charge_level = (*fp).u.mt.x2234_shadowBallCharge;
            *charge_cycles = f2i32((*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES as f64);
            return 0;
        }
        -1
    }
}

pub fn ftMt_SpecialN_CheckShadowBallRemove(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            let msid = (*fp).motion_id as u32;
            match msid {
                ftMt_MS_SpecialNStart
                | ftMt_MS_SpecialNLoop
                | ftMt_MS_SpecialNLoopFull
                | ftMt_MS_SpecialNCancel
                | ftMt_MS_SpecialNEnd
                | ftMt_MS_SpecialAirNStart
                | ftMt_MS_SpecialAirNLoop
                | ftMt_MS_SpecialAirNLoopFull
                | ftMt_MS_SpecialAirNCancel
                | ftMt_MS_SpecialAirNEnd => {
                    if (*fp).x2070.anon_0.x2071_b6() != 0 {
                        return true;
                    }
                    return false;
                }
                _ => {}
            }
            return true;
        }
        true
    }
}

pub fn ftMt_SpecialN_CheckShadowBallCancel(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        if !gobj.is_null() {
            let msid = (*fpp(gobj)).motion_id as u32;
            match msid {
                ftMt_MS_SpecialNStart
                | ftMt_MS_SpecialNLoop
                | ftMt_MS_SpecialNLoopFull
                | ftMt_MS_SpecialNEnd
                | ftMt_MS_SpecialAirNStart
                | ftMt_MS_SpecialAirNLoop
                | ftMt_MS_SpecialAirNLoopFull
                | ftMt_MS_SpecialAirNEnd => false,
                _ => true,
            }
        } else {
            true
        }
    }
}

/// `ftMewtwo_SpecialN_ClearGObj` (static inline): drop the charge-effect handle.
pub fn ftMewtwo_SpecialN_ClearGObj(gobj: *mut HSD_GObj) {
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !(*fp).u.mt.x2238_shadowBallGObj.is_null() {
                st::efLib_DestroyAll(gobj);
                (*fp).u.mt.x2238_shadowBallGObj = core::ptr::null_mut();
            }
        }
    }
}

pub fn ftMt_SpecialN_SetNULL(gobj: *mut HSD_GObj) {
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !(*fp).u.mt.x2230_shadowHeldGObj.is_null() {
                (*fp).u.mt.x2230_shadowHeldGObj = core::ptr::null_mut();
            }
            ftMewtwo_SpecialN_ClearGObj(gobj);
        }
    }
}

/// `ftMewtwo_SpecialN_RemoveHeldShadowBall` (static inline).
pub fn ftMewtwo_SpecialN_RemoveHeldShadowBall(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut fp2 = fp;
        if !gobj.is_null() {
            if !(*fp).u.mt.x2230_shadowHeldGObj.is_null() {
                ext::it_802C573C((*fp).u.mt.x2230_shadowHeldGObj);
                (*fp).u.mt.x2230_shadowHeldGObj = core::ptr::null_mut();
            }
            if !gobj.is_null() {
                fp2 = fpp(gobj);
                if !(*fp2).u.mt.x2238_shadowBallGObj.is_null() {
                    st::efLib_DestroyAll(gobj);
                    (*fp2).u.mt.x2238_shadowBallGObj = core::ptr::null_mut();
                }
            }
        }
        (*fp2).u.mt.x2234_shadowBallCharge = 0;
        ext::ftCo_800BFFAC(fp2);
        st::efLib_DestroyAll(gobj);
    }
}

pub fn ftMt_SpecialN_OnTakeDamage(gobj: *mut HSD_GObj) {
    if !gobj.is_null() {
        ftMewtwo_SpecialN_RemoveHeldShadowBall(gobj);
    }
}

pub fn ftMt_SpecialN_OnDeath(gobj: *mut HSD_GObj) {
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            let mut fp2 = fp;
            let a = da(fp);
            if !gobj.is_null() {
                if !(*fp).u.mt.x2230_shadowHeldGObj.is_null() {
                    ext::it_802C573C((*fp).u.mt.x2230_shadowHeldGObj);
                    (*fp).u.mt.x2230_shadowHeldGObj = core::ptr::null_mut();
                }
                if !gobj.is_null() {
                    fp2 = fpp(gobj);
                    if !(*fp2).u.mt.x2238_shadowBallGObj.is_null() {
                        st::efLib_DestroyAll(gobj);
                        (*fp2).u.mt.x2238_shadowBallGObj = core::ptr::null_mut();
                    }
                }
            }
            if ((*fp2).u.mt.x2234_shadowBallCharge as f32) != (*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES {
                (*fp2).u.mt.x2234_shadowBallCharge = 0;
                ext::ftCo_800BFFAC(fp2);
                st::efLib_DestroyAll(gobj);
            }
        }
    }
}

/// `ftMewtwo_SpecialN_GetPos` (static): the Shadow Ball's hold position, 2 units in front of the
/// right shoulder, z cleared.
pub fn ftMewtwo_SpecialN_GetPos(fp: *mut Fighter, sp38: &mut Vec3) {
    unsafe {
        let sp2c = Vec3 { x: 0.0, y: 0.0, z: 2.0 };
        ext::lb_8000B1CC((*(*fp).parts.add(FtPart_RShoulderN as usize)).joint, &sp2c, sp38);
        sp38.z = 0.0;
    }
}

/// `ftMewtwo_SpecialN_SetRecoil` (static).
pub fn ftMewtwo_SpecialN_SetRecoil(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if (*fp).motion_id as u32 == ftMt_MS_SpecialAirNEnd || (*fp).ground_or_air as i32 == GA_Air as i32 {
            (*fp).self_vel.x = (*fp).facing_dir
                * ((*a).x8_MEWTWO_SHADOWBALL_AIR_RECOIL_X * ((*fp).u.mt.x2234_shadowBallCharge as f32));
        }
        if (*fp).motion_id as u32 == ftMt_MS_SpecialNEnd || (*fp).ground_or_air as i32 == GA_Ground as i32 {
            (*fp).gr_vel = (*fp).facing_dir
                * ((*a).x4_MEWTWO_SHADOWBALL_GROUND_RECOIL_X * ((*fp).u.mt.x2234_shadowBallCharge as f32));
        }
    }
}

pub fn ftMt_SpecialN_ReleaseShadowBall(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if (*fp).cmd_vars[1] == 1 && !(*fp).u.mt.x2230_shadowHeldGObj.is_null() {
            let mut sp38 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            (*fp).cmd_vars[1] = 2;
            ftMewtwo_SpecialN_GetPos(fp, &mut sp38);
            let shadow_ball_gobj = (*fp).item_gobj;
            let facing_dir: f64 = if 1.0f32 == (*fp).facing_dir { 0.0 } else { core::f64::consts::PI };
            ext::it_802C53F0(
                (*fp).u.mt.x2230_shadowHeldGObj,
                &sp38,
                facing_dir as f32,
                ((*fp).u.mt.x2234_shadowBallCharge as u32) as f32,
                (*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES,
            );
            ftMewtwo_SpecialN_SetRecoil(gobj);
            (*fp).u.mt.x2234_shadowBallCharge = 0;
            if !gobj.is_null() {
                let temp_fp = fpp(gobj);
                if !(*temp_fp).u.mt.x2230_shadowHeldGObj.is_null() {
                    (*temp_fp).u.mt.x2230_shadowHeldGObj = core::ptr::null_mut();
                }
                if !gobj.is_null() {
                    let fp2 = fpp(gobj);
                    if !(*fp2).u.mt.x2238_shadowBallGObj.is_null() {
                        st::efLib_DestroyAll(gobj);
                        (*fp2).u.mt.x2238_shadowBallGObj = core::ptr::null_mut();
                    }
                }
            }
            (*fp).item_gobj = shadow_ball_gobj;
            if ((*fp).u.mt.x2234_shadowBallCharge as f32) == (*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES {
                ext::ft_PlaySFX(fp, 200118, 127, 64);
                return;
            }
            ext::ft_PlaySFX(fp, 200115, 127, 64);
        }
    }
}

/// `shadowBallSFX` (function-local static table).
pub const shadowBallSFX: [u32; 4] = [200121, 200124, 200127, 200130];

pub fn ftMt_SpecialN_PlayChargeSFX(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if (*fp).cmd_vars[2] != 0 {
            let charge_level: f32 = if (*fp).u.mt.x2234_shadowBallCharge != 0 {
                ((*fp).u.mt.x2234_shadowBallCharge as f32) / (*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES
            } else {
                0.0
            };
            if (*fp).mv.mt.SpecialN.chargeLevel > 0.0 {
                if charge_level > 0.75 && (*fp).mv.mt.SpecialN.chargeLevel <= 0.75 {
                    (*fp).mv.mt.SpecialN.chargeLevel = charge_level;
                    ext::ft_80088510(fp, shadowBallSFX[3] as i32, 127, 64);
                    return;
                }
                if charge_level > 0.5 && (*fp).mv.mt.SpecialN.chargeLevel <= 0.5 {
                    (*fp).mv.mt.SpecialN.chargeLevel = charge_level;
                    ext::ft_80088510(fp, shadowBallSFX[2] as i32, 127, 64);
                    return;
                }
                if charge_level > 0.25 && (*fp).mv.mt.SpecialN.chargeLevel <= 0.25 {
                    (*fp).mv.mt.SpecialN.chargeLevel = charge_level;
                    ext::ft_80088510(fp, shadowBallSFX[1] as i32, 127, 64);
                    return;
                }
                if charge_level > 0.0 && (*fp).mv.mt.SpecialN.chargeLevel <= 0.0 {
                    (*fp).mv.mt.SpecialN.chargeLevel = charge_level;
                    ext::ft_80088510(fp, shadowBallSFX[0] as i32, 127, 64);
                }
            } else {
                (*fp).mv.mt.SpecialN.chargeLevel = 9.999999747378752e-5f32;
                ext::ft_80088510(fp, shadowBallSFX[0] as i32, 127, 64);
            }
        }
    }
}

/// `ftMewtwo_SpecialN_ChangeAction` (static inline), the grounded entry.
pub fn ftMewtwo_SpecialN_ChangeAction(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialNStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        let mut release_lag: i32 = 0;
        (*fp).cmd_vars[3] = 0;
        (*fp).cmd_vars[2] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[0] = 0;
        st::ftCommon_8007D7FC(&mut *fp);
        (*fp).self_vel.y = 0.0;
        ftMewtwo_SpecialN_SetCall(gobj);
        (*fp).mv.mt.SpecialN.isFull = 0;
        (*fp).mv.mt.SpecialN.x2344 = 0;
        if (*fp).u.mt.x2234_shadowBallCharge == 0 {
            release_lag = (*a).x10_MEWTWO_SHADOWBALL_RELEASE_LAG;
        }
        (*fp).mv.mt.SpecialN.releaseLag = release_lag;
        (*fp).mv.mt.SpecialN.chargeLevel = 0.0;
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftMt_SpecialN_Enter(gobj: *mut HSD_GObj) {
    ftMewtwo_SpecialN_ChangeAction(gobj);
}

/// `ftMewtwo_SpecialAirN_ChangeAction` (static inline), the aerial entry.
pub fn ftMewtwo_SpecialAirN_ChangeAction(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirNStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        let mut release_lag: i32 = 0;
        (*fp).cmd_vars[3] = 0;
        (*fp).cmd_vars[2] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[0] = 0;
        ftMewtwo_SpecialN_SetCall(gobj);
        (*fp).mv.mt.SpecialN.isFull = 0;
        (*fp).mv.mt.SpecialN.x2344 = 0;
        if (*fp).u.mt.x2234_shadowBallCharge == 0 {
            release_lag = (*a).x10_MEWTWO_SHADOWBALL_RELEASE_LAG;
        }
        (*fp).mv.mt.SpecialN.releaseLag = release_lag;
        (*fp).mv.mt.SpecialN.chargeLevel = 0.0;
        (*fp).self_vel.y *= 0.5;
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftMt_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    ftMewtwo_SpecialAirN_ChangeAction(gobj);
}

/// The held-Shadow-Ball spawn the Start / Loop animation callbacks share on the frame the script
/// sets cmd_vars[3] (`ftMewtwo_SpecialN_CreateHeldShadow` and the inlined copies in the Start
/// callbacks).
fn create_held_shadow(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    unsafe {
        if (*fp).cmd_vars[3] == 1 && (*fp).u.mt.x2230_shadowHeldGObj.is_null() {
            let sp2c = Vec3 { x: 0.0, y: 0.0, z: 2.0 };
            let mut sp20 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_RShoulderN as usize)).joint, &sp2c, &mut sp20);
            sp20.z = 0.0;
            let held = ext::it_802C5000(
                gobj,
                &sp20,
                FtPart_RShoulderN as i32,
                It_Kind_Mewtwo_ShadowBall as i32,
                (*fp).facing_dir,
            );
            (*fp).u.mt.x2230_shadowHeldGObj = held;
            if !held.is_null() {
                ftMewtwo_SpecialN_SetCall(gobj);
            } else {
                (*fp).u.mt.x2230_shadowHeldGObj = core::ptr::null_mut();
            }
        }
    }
}

pub fn ftMt_SpecialNStart_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        create_held_shadow(gobj, fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).mv.mt.SpecialN.isFull == 1
                || ((*fp).u.mt.x2234_shadowBallCharge as f32) == (*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES
            {
                st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialNEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            } else {
                st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialNLoop as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                (*fp).mv.mt.SpecialN.x2348 = 0;
            }
            ftMewtwo_SpecialN_SetCall(gobj);
        }
    }
}

/// `ftMewtwo_SpecialN_CreateHeldShadow` (static inline), the version the Loop callbacks call.
pub fn ftMewtwo_SpecialN_CreateHeldShadow(gobj: *mut HSD_GObj) {
    unsafe { create_held_shadow(gobj, fpp(gobj)) }
}

/// The charge step both Loop animation callbacks run; `full_msid` / `full_flag` differ.
unsafe fn loop_anim(gobj: *mut HSD_GObj, full_msid: i32, full_x2348: i32) {
    let fp = fpp(gobj);
    let a = da(fp);
    let sp34 = Vec3 { x: 0.0, y: 7.0, z: 0.0 };
    let mut sp40 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    (*fp).mv.mt.SpecialN.releaseLag -= 1;
    if (*fp).mv.mt.SpecialN.releaseLag <= 0 {
        (*fp).mv.mt.SpecialN.releaseLag = 0;
        ftMewtwo_SpecialN_CreateHeldShadow(gobj);
        ftMt_SpecialN_PlayChargeSFX(gobj);
        if (*fp).mv.mt.SpecialN.x2348 == 0 {
            (*fp).mv.mt.SpecialN.x2344 += 1;
            if (*fp).mv.mt.SpecialN.x2344 > (*a).xC_MEWTWO_SHADOWBALL_CHARGE_ITERATIONS {
                (*fp).mv.mt.SpecialN.x2344 = 0;
                (*fp).u.mt.x2234_shadowBallCharge += 1;
                if ((*fp).u.mt.x2234_shadowBallCharge as f32) >= (*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES {
                    st::Fighter_ChangeMotionState(
                        gobj,
                        full_msid,
                        FTMEWTWO_SPECIALN_ACTION_FLAG,
                        (*fp).cur_anim_frame,
                        1.0,
                        0.0,
                        core::ptr::null_mut(),
                    );
                    (*fp).u.mt.x2234_shadowBallCharge = f2i32((*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES as f64);
                    (*fp).mv.mt.SpecialN.x2348 = full_x2348;
                    st::ftCommon_8007EBAC(&mut *fp, 12, 0);
                    ext::lb_8000B1CC((*(*fp).parts.add(FtPart_TopN as usize)).joint, &sp34, &mut sp40);
                    ext::efSync_Spawn(27, gobj, &[&mut sp40 as *mut Vec3 as *mut core::ffi::c_void]);
                    ext::ftCo_800BFFD0(fp, 92, 0);
                }
            }
        }
    }
}

pub fn ftMt_SpecialNLoop_Anim(gobj: *mut HSD_GObj) {
    unsafe { loop_anim(gobj, ftMt_MS_SpecialNLoopFull as i32, 1) }
}

pub fn ftMt_SpecialNLoopFull_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        (*fp).mv.mt.SpecialN.x2348 = 1;
        (*fp).u.mt.x2234_shadowBallCharge = f2i32((*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES as f64);
    }
}

/// `ftMewtwo_SpecialN_RemoveShadowBall2` (static inline).
pub fn ftMewtwo_SpecialN_RemoveShadowBall2(gobj: *mut HSD_GObj) {
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !(*fp).u.mt.x2230_shadowHeldGObj.is_null() {
                ext::it_802C573C((*fp).u.mt.x2230_shadowHeldGObj);
                (*fp).u.mt.x2230_shadowHeldGObj = core::ptr::null_mut();
            }
            ftMewtwo_SpecialN_ClearGObj(gobj);
        }
    }
}

pub fn ftMt_SpecialNCancel_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        ftMewtwo_SpecialN_RemoveShadowBall2(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftMt_SpecialNEnd_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        ftMt_SpecialN_ReleaseShadowBall(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftMt_SpecialAirNStart_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        create_held_shadow(gobj, fp);
        (*fp).mv.mt.SpecialN.isFull = 0;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).mv.mt.SpecialN.isFull == 1
                || ((*fp).u.mt.x2234_shadowBallCharge as f32) == (*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES
            {
                st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirNEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            } else {
                st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirNLoop as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                (*fp).mv.mt.SpecialN.x2348 = 0;
            }
            ftMewtwo_SpecialN_SetCall(gobj);
        }
    }
}

pub fn ftMt_SpecialAirNLoop_Anim(gobj: *mut HSD_GObj) {
    unsafe { loop_anim(gobj, ftMt_MS_SpecialAirNLoopFull as i32, 1) }
}

pub fn ftMt_SpecialAirNLoopFull_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        (*fp).mv.mt.SpecialN.x2348 = 0;
        (*fp).u.mt.x2234_shadowBallCharge = f2i32((*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES as f64);
    }
}

pub fn ftMt_SpecialAirNCancel_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        ftMewtwo_SpecialN_RemoveShadowBall2(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftMt_SpecialAirNEnd_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        ftMt_SpecialN_ReleaseShadowBall(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if 0.0f32 == (*a).x14_MEWTWO_SHADOWBALL_LANDING_LAG {
                st::ftCo_Fall_Enter(gobj);
                return;
            }
            st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*a).x14_MEWTWO_SHADOWBALL_LANDING_LAG);
        }
    }
}

pub fn ftMt_SpecialNLoop_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if ext::ftCo_8009917C(gobj) {
            ftMewtwo_SpecialN_RemoveShadowBall2(gobj);
        } else {
            let recent_input = (*fp).input.pressed_buttons;
            if (recent_input & HSD_PAD_A as u32) != 0 && (*fp).mv.mt.SpecialN.releaseLag <= 0 {
                st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialNEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                ftMewtwo_SpecialN_SetCall(gobj);
                return;
            }
            if (recent_input & HSD_PAD_B as u32) != 0 {
                if (*fp).mv.mt.SpecialN.releaseLag <= 0 {
                    st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialNEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                    ftMewtwo_SpecialN_SetCall(gobj);
                }
            } else if (recent_input & HSD_PAD_LR) != 0 {
                st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialNCancel as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                ftMewtwo_SpecialN_RemoveShadowBall2(gobj);
                ftMewtwo_SpecialN_SetCall(gobj);
            }
        }
    }
}

pub fn ftMt_SpecialNLoopFull_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if ext::ftCo_8009917C(gobj) {
            ftMewtwo_SpecialN_RemoveShadowBall2(gobj);
        } else {
            let recent_input = (*fp).input.pressed_buttons;
            if (recent_input & HSD_PAD_A as u32) != 0 {
                st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialNEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                ftMewtwo_SpecialN_SetCall(gobj);
                return;
            }
            if (recent_input & HSD_PAD_B as u32) != 0 {
                st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialNEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                ftMewtwo_SpecialN_SetCall(gobj);
            } else if (recent_input & HSD_PAD_LR) != 0 {
                st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialNCancel as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                ftMewtwo_SpecialN_RemoveShadowBall2(gobj);
                ftMewtwo_SpecialN_SetCall(gobj);
            }
        }
    }
}

pub fn ftMt_SpecialAirNLoop_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let recent_input = (*fp).input.pressed_buttons;
        if (recent_input & HSD_PAD_A as u32) != 0 && (*fp).mv.mt.SpecialN.releaseLag <= 0 {
            st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirNEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            ftMewtwo_SpecialN_SetCall(gobj);
            return;
        }
        if (recent_input & HSD_PAD_B as u32) != 0 {
            if (*fp).mv.mt.SpecialN.releaseLag <= 0 {
                st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirNEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                ftMewtwo_SpecialN_SetCall(gobj);
            }
        } else if (recent_input & HSD_PAD_LR) != 0 {
            st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirNCancel as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            ftMewtwo_SpecialN_RemoveShadowBall2(gobj);
            ftMewtwo_SpecialN_SetCall(gobj);
        }
    }
}

pub fn ftMt_SpecialAirNLoopFull_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let recent_input = (*fp).input.pressed_buttons;
        if (recent_input & HSD_PAD_A as u32) != 0 {
            st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirNEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            ftMewtwo_SpecialN_SetCall(gobj);
            return;
        }
        if (recent_input & HSD_PAD_B as u32) != 0 {
            st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirNEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            ftMewtwo_SpecialN_SetCall(gobj);
        } else if (recent_input & HSD_PAD_LR) != 0 {
            st::Fighter_ChangeMotionState(gobj, ftMt_MS_SpecialAirNCancel as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            ftMewtwo_SpecialN_RemoveShadowBall2(gobj);
            ftMewtwo_SpecialN_SetCall(gobj);
        }
    }
}

/// `ftMewtwo_SpecialN_LaunchShadowBall` (static inline).
pub fn ftMewtwo_SpecialN_LaunchShadowBall(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).kind as i32 == Ft_Kind_Mewtwo as i32 {
            let a = da(fp);
            if (*fp).cmd_vars[3] == 1 {
                let mut sp20 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                let mut sp2c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                (*fp).cmd_vars[3] = 0;
                ext::lb_8000B1CC((*(*fp).parts.add(FtPart_RShoulderN as usize)).joint, core::ptr::null(), &mut sp20);
                ext::lb_8000B1CC((*(*fp).parts.add(FtPart_LHandNb as usize)).joint, core::ptr::null(), &mut sp2c);
                sp2c.z = 0.0;
                sp20.z = 0.0;
                ext::it_802C519C(
                    gobj,
                    &sp20,
                    It_Kind_Mewtwo_ShadowBall as i32,
                    (*a).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES,
                    atan2f(sp20.y - sp2c.y, sp20.x - sp2c.x),
                    (*fp).facing_dir,
                );
                ext::ft_PlaySFX(fp, 200115, 127, 64);
            }
        }
    }
}

pub fn ftMt_SpecialN_Shoot(gobj: *mut HSD_GObj) {
    ftMewtwo_SpecialN_LaunchShadowBall(gobj);
}
