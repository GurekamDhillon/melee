//! src/melee/ft/kinds/ftNana/ftnanaspecials.c: Nana's follow-the-leader callbacks (Ice Climbers
//! side-B and the Init helpers that reposition her).
//!
//! One fused site per `Coll_inline4` (`1.5f * (x0 * facing) - popo.x`, an fmadds with the outer
//! negation applied after), reproduced with [`fmadds`] (tools/rust-port/fma_sites.py). Smoke-tested
//! through recording hooks by tests/tu_ftnanaspecials.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

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
    hook!(PLAYER_GETENTITYATINDEX, Player_GetEntityAtIndex, (slot: i32, index: i32) -> *mut HSD_GObj);
    hook!(FT_800849EC, ft_800849EC, (popo: *mut Fighter, nana: *mut Fighter));
    hook!(FT_80082888, ft_80082888, (gobj: *mut HSD_GObj, cb: *mut ftCollisionBox) -> i32);
    hook!(FT_800824A0, ft_800824A0, (gobj: *mut HSD_GObj, cb: *mut ftCollisionBox) -> i32);
    hook!(FTPP_SPECIALS_8011F6FC, ftPp_SpecialS_8011F6FC, (gobj: *mut HSD_GObj) -> i32);
    hook!(SLIPPI_CODES, Slippi_Codes, () -> i32);
    hook!(SLIPPI_VERSION, Slippi_Version, () -> i32);
    hook!(FTPARTSETROTX, ftPartSetRotX, (fp: *mut Fighter, part: i32, v: f32));
    hook!(FTPARTSETROTY, ftPartSetRotY, (fp: *mut Fighter, part: i32, v: f32));
    hook!(FTPARTGETROTX, ftPartGetRotX, (fp: *mut Fighter, part: i32) -> f32);
    hook!(FTNN_INIT_80122FAC, ftNn_Init_80122FAC, (gobj: *mut HSD_GObj));
    /// C-ABI trampoline so `ftNn_Init_80122FAC` can be stored as a `take_dmg_cb`/`death2_cb`.
    pub unsafe extern "C" fn ftNn_Init_80122FAC_cb(gobj: *mut HSD_GObj) {
        ftNn_Init_80122FAC(gobj)
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftNn_Unk2_803CDD60`: the collision box Nana's follow code probes with.
pub static mut ftNn_Unk2_803CDD60: ftCollisionBox = ftCollisionBox {
    top: 12.0,
    bottom: 0.0,
    left: Vec2 { x: -6.0, y: 6.0 },
    right: Vec2 { x: 6.0, y: 6.0 },
};

/// `M_PI_2 * facing_dir` (double math, narrowed to the f32 argument).
#[inline(always)]
fn half_pi_facing(facing: f32) -> f32 {
    (core::f64::consts::FRAC_PI_2 * facing as f64) as f32
}

pub fn ftNn_Init_801238E4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftPartSetRotX(fp, 0, 0.0);
        st::Fighter_UnkSetFlag_8006CFBC(gobj);
        if !(*fp).x1A5C.is_null() {
            let fp2 = fpp((*fp).x1A5C);
            st::Fighter_UnkSetFlag_8006CFBC((*fp).x1A5C);
            (*fp2).x1A5C = core::ptr::null_mut();
        }
        (*fp).x1A5C = core::ptr::null_mut();
    }
}

/// Returns C `bool`.
pub fn ftNn_Init_80123954(nana_gobj: *mut HSD_GObj, pp_ga: GroundOrAir) -> bool {
    unsafe {
        if nana_gobj.is_null() {
            return true;
        }
        let nana_fp = fpp(nana_gobj);
        let popo_fp = fpp(ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0));
        let attrs = (*nana_fp).dat_attrs as *const ftIceClimberAttributes;
        match (*nana_fp).x2070.anon_0.x2071_b0_3() {
            1..=13 => {
                // TARGET_PC: Slippi's FreezeGlitchFix keeps Nana's grab link in the online codeset
                // and in console codesets from Slippi 2.x on.
                if !(ext::Slippi_Codes() == 2 || (ext::Slippi_Codes() == 1 && ext::Slippi_Version() >= 20000)) {
                    (*nana_fp).x1A5C = core::ptr::null_mut();
                }
                true
            }
            _ => {
                let dd = (*attrs).xD0;
                let dx = (*nana_fp).cur_pos.x - (*popo_fp).cur_pos.x;
                let dy = (*nana_fp).cur_pos.y - (*popo_fp).cur_pos.y;
                let dxsq = dx * dx;
                let dysq = dy * dy;
                // C: `dxsq + dysq < (s32) (scale.y * (dd * dd))`: the s32 is converted back to
                // float for the compare.
                if dxsq + dysq < ((*popo_fp).x34_scale.y * (dd * dd)) as i32 as f32 {
                    if pp_ga == GA_Air {
                        if pp_ga != (*nana_fp).ground_or_air {
                            st::ftCommon_8007D5D4(&mut *nana_fp);
                        }
                        ftNn_Init_80123BF0(nana_gobj);
                    } else {
                        if pp_ga != (*nana_fp).ground_or_air {
                            ext::ft_800849EC(popo_fp, nana_fp);
                            st::ftCommon_8007D7FC(&mut *nana_fp);
                        }
                        ftNn_Init_80123B3C(nana_gobj);
                    }
                    (*nana_fp).mv.pp.unk_80123954.x0 = 0;
                    (*nana_fp).cur_pos = (*popo_fp).cur_pos;
                    (*nana_fp).self_vel = (*popo_fp).self_vel;
                    (*nana_fp).gr_vel = (*popo_fp).gr_vel;
                    (*nana_fp).facing_dir = (*popo_fp).facing_dir;
                    ext::ftPartSetRotY(nana_fp, 0, half_pi_facing((*nana_fp).facing_dir));
                    (*nana_fp).x1A5C = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
                    false
                } else {
                    (*nana_fp).x1A5C = core::ptr::null_mut();
                    true
                }
            }
        }
    }
}

/// C `bool`: false while Nana is in one of the two follow motions (359/360).
pub fn ftNn_Init_80123B10(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        match (*fpp(gobj)).motion_id as i32 {
            359 | 360 => false,
            _ => true,
        }
    }
}

/// `ftNn_Init_80123B3C_inline` (static inline).
fn ftNn_Init_80123B3C_inline(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).take_dmg_cb = Some(ext::ftNn_Init_80122FAC_cb);
        (*fp).death2_cb = Some(ext::ftNn_Init_80122FAC_cb);
        melee_fighter::tu::ftCo_AttackS4::Fighter_SetEffectHitlagCallbacks(&mut *fp);
    }
}

/// Shared tail of the two follow-motion entries (359 ground / 360 air).
fn enter_follow(gobj: *mut HSD_GObj, msid: i32) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ClearCmdVars(&mut *fp);
        ftNn_Init_80123B3C_inline(gobj);
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftNn_Init_80123B3C_inline(gobj);
        st::ftAnim_8006EBA4(gobj);
        (*fp).x74_self_accel.y = 0.0;
        (*fp).self_vel.y = 0.0;
    }
}

pub fn ftNn_Init_80123B3C(gobj: *mut HSD_GObj) {
    enter_follow(gobj, 359);
}

pub fn ftNn_Init_80123BF0(gobj: *mut HSD_GObj) {
    enter_follow(gobj, 360);
}

/// `ftPp_SpecialS_0_Anim_inline` (static inline): drop Nana's callbacks and grab link.
fn ftPp_SpecialS_0_Anim_inline(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        (*nana_fp).take_dmg_cb = None;
        (*nana_fp).death2_cb = None;
        (*nana_fp).pre_hitlag_cb = None;
        (*nana_fp).post_hitlag_cb = None;
        (*nana_fp).x1A5C = core::ptr::null_mut();
    }
}

/// `ftPp_SpecialS_8011F964(gobj)` (ftPopo/ftpopospecials.c, already ported on the fighter).
fn ftPp_SpecialS_8011F964(gobj: *mut HSD_GObj) -> bool {
    unsafe { crate::chars::ftPopo::ftpopospecials::ftPp_SpecialS_8011F964(&*fpp(gobj)) }
}

pub fn ftPp_SpecialS_0_Anim(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        let popo_fp = fpp(popo_gobj);
        if !ftPp_SpecialS_8011F964(popo_gobj) {
            (*nana_fp).cur_anim_frame = (*popo_fp).cur_anim_frame;
            return;
        }
        ftNn_Init_801238E4(nana_gobj);
        ftPp_SpecialS_0_Anim_inline(nana_gobj);
        st::ft_8008A2BC(nana_gobj);
    }
}

pub fn ftPp_SpecialS_1_Anim(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let attrs = (*nana_fp).dat_attrs as *const ftIceClimberAttributes;
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        let popo_fp = fpp(popo_gobj);
        if !ftPp_SpecialS_8011F964(popo_gobj) {
            (*nana_fp).cur_anim_frame = (*popo_fp).cur_anim_frame;
            return;
        }
        ftNn_Init_801238E4(nana_gobj);
        ftPp_SpecialS_0_Anim_inline(nana_gobj);
        if 0.0_f64 == (*attrs).x12C as f64 {
            st::ftCo_Fall_Enter(nana_gobj);
        } else {
            st::ftCommon_8007D60C(&mut *nana_fp);
            st::ftCo_80096900(nana_gobj, 1, 0, 1, 1.0, (*attrs).x12C);
        }
    }
}

pub fn ftPp_SpecialS_0_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftPp_SpecialS_1_IASA(_gobj: *mut HSD_GObj) {}

/// Copy the leader's velocity/accel/facing and turn Nana to match (shared by both Phys callbacks).
fn copy_popo_motion(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_fp = fpp(ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0));
        (*nana_fp).self_vel = (*popo_fp).self_vel;
        (*nana_fp).x74_self_accel = (*popo_fp).x74_self_accel;
        (*nana_fp).gr_vel = (*popo_fp).gr_vel;
        (*nana_fp).xE4_ground_accel_1 = (*popo_fp).xE4_ground_accel_1;
        (*nana_fp).facing_dir = (*popo_fp).facing_dir;
        ext::ftPartSetRotY(nana_fp, 0, half_pi_facing((*nana_fp).facing_dir));
    }
}

pub fn ftPp_SpecialS_0_Phys(nana_gobj: *mut HSD_GObj) {
    copy_popo_motion(nana_gobj);
}

pub fn ftPp_SpecialS_1_Phys(nana_gobj: *mut HSD_GObj) {
    copy_popo_motion(nana_gobj);
}

/// `ftPp_SpecialS_0_Coll_inline` (static inline): copy the leader's X rotation.
fn ftPp_SpecialS_0_Coll_inline(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        let popo_fp = fpp(popo_gobj);
        let rot = ext::ftPartGetRotX(popo_fp, 0);
        ext::ftPartSetRotX(nana_fp, 0, rot);
    }
}

/// `ftPp_SpecialS_0_Coll_inline2` (static inline): re-arm Nana's damage/death/hitlag callbacks.
fn ftPp_SpecialS_0_Coll_inline2(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        (*nana_fp).take_dmg_cb = Some(ext::ftNn_Init_80122FAC_cb);
        (*nana_fp).death2_cb = Some(ext::ftNn_Init_80122FAC_cb);
        melee_fighter::tu::ftCo_AttackS4::Fighter_SetEffectHitlagCallbacks(&mut *nana_fp);
    }
}

/// `ftPp_SpecialS_{0,1}_Coll_inline3`: fall back to the other medium (`air`: 0 = ground->air,
/// 1 = air->ground; the air->ground form also zeroes `self_vel.y` before the motion change).
fn special_s_coll_inline3(gobj: *mut HSD_GObj, to_ground: bool) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ClearCmdVars(&mut *fp);
        ftNn_Init_80123B3C_inline(gobj);
        if to_ground {
            st::ftCommon_8007D7FC(&mut *fp);
            (*fp).self_vel.y = 0.0;
        } else {
            st::ftCommon_8007D5D4(&mut *fp);
        }
        st::Fighter_ChangeMotionState(
            gobj,
            if to_ground { 359 } else { 360 },
            0x0C4C528A,
            (*fp).cur_anim_frame,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        ftNn_Init_80123B3C_inline(gobj);
    }
}

/// `ftPp_SpecialS_{0,1}_Coll_inline4`: track the leader (on ground/air change switch motion), or
/// slide in behind them while they are in the attack (motions 360 air / 359 ground).
fn special_s_coll_inline4(
    nana_gobj: *mut HSD_GObj,
    popo_fp: *mut Fighter,
    nana_fp: *mut Fighter,
    popo_gobj: *mut HSD_GObj,
    ground: bool,
) {
    unsafe {
        if !ftPp_SpecialS_8011F964(popo_gobj) && ext::ftPp_SpecialS_8011F6FC(popo_gobj) == 0 {
            (*nana_fp).cur_pos = (*popo_fp).cur_pos;
            ext::ft_800849EC(popo_fp, nana_fp);
            if (*nana_fp).ground_or_air != (*popo_fp).ground_or_air {
                st::Fighter_ChangeMotionState(
                    nana_gobj,
                    if ground { 359 } else { 360 },
                    0x0C4C528A,
                    (*nana_fp).cur_anim_frame,
                    1.0,
                    0.0,
                    core::ptr::null_mut(),
                );
            }
            (*nana_fp).ground_or_air = (*popo_fp).ground_or_air;
        } else {
            (*nana_fp).mv.pp.unk_80123954.x0 += 1;
            if (*nana_fp).mv.pp.unk_80123954.x0 > 5 {
                (*nana_fp).mv.pp.unk_80123954.x0 = 5;
            }
            // -((1.5f * (x0 * facing)) - popo.x): one fmadds, then the outer negation.
            let t = (*nana_fp).mv.pp.unk_80123954.x0 as f32 * (*nana_fp).facing_dir;
            (*nana_fp).cur_pos.x = -fmadds(1.5, t, -(*popo_fp).cur_pos.x);
            if ground {
                if ext::ft_800824A0(nana_gobj, &mut ftNn_Unk2_803CDD60) != 0 {
                    special_s_coll_inline3(nana_gobj, true);
                }
            } else if ext::ft_80082888(nana_gobj, &mut ftNn_Unk2_803CDD60) == 0 {
                special_s_coll_inline3(nana_gobj, false);
            }
        }
    }
}

pub fn ftPp_SpecialS_0_Coll(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        let popo_fp = fpp(popo_gobj);
        special_s_coll_inline4(nana_gobj, popo_fp, nana_fp, popo_gobj, false);
        ftPp_SpecialS_0_Coll_inline(nana_gobj);
        ftPp_SpecialS_0_Coll_inline2(nana_gobj);
    }
}

pub fn ftPp_SpecialS_1_Coll(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        let popo_fp = fpp(popo_gobj);
        special_s_coll_inline4(nana_gobj, popo_fp, nana_fp, popo_gobj, true);
        ftPp_SpecialS_0_Coll_inline(nana_gobj);
        ftPp_SpecialS_0_Coll_inline2(nana_gobj);
    }
}
