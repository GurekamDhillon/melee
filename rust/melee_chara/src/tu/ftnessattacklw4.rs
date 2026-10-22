//! src/melee/ft/kinds/ftNess/ftnessattacklw4.c: Ness down smash (shares the Yo-Yo helpers of
//! ftnessattackhi4.c).
//! No fused sites in this file (tools/rust-port/fma_sites.py): the blend lives in
//! `ftNs_AttackHi4_YoyoSetHitPosUnk`. Smoke-tested through recording hooks by
//! tests/tu_ftnessattacklw4.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftnessattackhi4 as hi4;
use melee_types::*;

/// `Ft_MF_SkipItemVis`.
const MF_SKIP_ITEM_VIS: u32 = Ft_MF_SkipItemVis;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftNs_AttackLw4_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).set_allow_interrupt(0);
        (*fp).mv.ns.attacklw4.isChargeDisable = 0;
        hi4::ftNs_AttackHi4_YoyoSetVarAll(gobj);
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_AttackLw4 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        hi4::set_yoyo_callbacks(fp);
    }
}

pub fn ftNs_AttackLw4_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ns.attacklw4.yoyoCurrentFrame += 1;
        if !hi4::ftNs_AttackHi4_YoyoThink_IsRemove(gobj) {
            hi4::ftNs_AttackHi4_YoyoCheckTimedRehit(gobj);
            if (*fp).mv.ns.attacklw4.yoyoCurrentFrame == 13
                && (*fp).mv.ns.attacklw4.isChargeDisable == 0
                && hi4::ftNs_AttackHi4_YoyoCheckNoObstruct(gobj)
            {
                ftNs_AttackLw4Charge_Enter(gobj);
            }
            if st::ftAnim_IsFramesRemaining(gobj) == 0 {
                st::ft_8008A2BC(gobj);
            }
        }
    }
}

pub fn ftNs_AttackLw4_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).input.held_buttons[0] as i32 & HSD_PAD_A == 0 {
            (*fp).mv.ns.attacklw4.isChargeDisable = 1;
        }
        if (*fp).allow_interrupt() != 0 {
            st::ftCo_Wait_IASA(gobj);
        }
    }
}

pub fn ftNs_AttackLw4_Phys(gobj: *mut HSD_GObj) {
    st::ft_80084F3C(gobj);
    hi4::ftNs_AttackHi4_YoyoSetHitPos(gobj);
}

/// Shared `*_Coll` body: ground/air collision, and in the air the Yo-Yo is despawned.
fn coll_despawn_in_air(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ft_80084104(gobj);
        if (*fp).ground_or_air as i32 == GA_Air as i32 {
            hi4::ftNs_AttackHi4_YoyoItemDespawn(gobj);
        }
    }
}

pub fn ftNs_AttackLw4_Coll(gobj: *mut HSD_GObj) {
    coll_despawn_in_air(gobj);
}

pub fn ftNs_AttackLw4Charge_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let attrs = (*fp).dat_attrs as *const ftNessAttributes;
        (*fp).mv.ns.attacklw4.yoyoCurrentFrame += 1;
        hi4::ftNs_AttackHi4_YoyoSetUnkRate(gobj);
        hi4::ftNs_AttackHi4_YoyoCheckTimedRehit(gobj);
        if (*fp).mv.ns.attacklw4.yoyoCurrentFrame as f32 >= (*attrs).xAC_YOYO_CHARGE_DURATION {
            ftNs_AttackLw4Release_Enter(gobj);
        }
    }
}

pub fn ftNs_AttackLw4Charge_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).input.held_buttons[0] as i32 & HSD_PAD_A == 0 {
            ftNs_AttackLw4Release_Enter(gobj);
        }
    }
}

pub fn ftNs_AttackLw4Charge_Phys(gobj: *mut HSD_GObj) {
    st::ft_80084F3C(gobj);
}

pub fn ftNs_AttackLw4Charge_Coll(gobj: *mut HSD_GObj) {
    coll_despawn_in_air(gobj);
}

pub fn ftNs_AttackLw4Charge_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(
            gobj,
            ftNs_MS_AttackLw4Charge as i32,
            MF_SKIP_ITEM_VIS,
            12.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
        st::ftAnim_SetAnimRate(gobj, 0.0);
        hi4::ftNs_AttackHi4_YoyoApplySmash(gobj);
        hi4::set_yoyo_callbacks(fp);
    }
}

pub fn ftNs_AttackLw4Release_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ns.attacklw4.yoyoCurrentFrame += 1;
        if !hi4::ftNs_AttackHi4_YoyoThink_IsRemove(gobj) {
            hi4::ftNs_AttackHi4_YoyoCheckTimedRehit(gobj);
            if st::ftAnim_IsFramesRemaining(gobj) == 0 {
                st::ft_8008A2BC(gobj);
            }
        }
    }
}

pub fn ftNs_AttackLw4Release_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).allow_interrupt() != 0 {
            st::ftCo_Wait_IASA(gobj);
        }
    }
}

/// The hit-position blend factor of the first 5 Release frames:
/// `0.20000000298023224f * (frame - 14)` clamped to [0, 1].
#[inline(always)]
pub fn release_blend(frame: i32) -> f32 {
    let mut v = 0.20000000298023224_f32 * (frame as f32 - 14.0);
    if v >= 1.0 {
        v = 1.0;
    } else if v <= 0.0 {
        v = 0.0;
    }
    v
}

pub fn ftNs_AttackLw4Release_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ft_80084F3C(gobj);
        let cur = (*fp).mv.ns.attacklw4.yoyoCurrentFrame;
        if cur < 0x13 {
            hi4::ftNs_AttackHi4_YoyoSetHitPosUnk(gobj, release_blend(cur));
            return;
        }
        hi4::ftNs_AttackHi4_YoyoSetHitPos(gobj);
    }
}

pub fn ftNs_AttackLw4Release_Coll(gobj: *mut HSD_GObj) {
    coll_despawn_in_air(gobj);
}

pub fn ftNs_AttackLw4Release_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(
            gobj,
            ftNs_MS_AttackLw4Release as i32,
            MF_SKIP_ITEM_VIS,
            13.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
        hi4::ftNs_AttackHi4_YoyoSetChargeDamage(gobj);
        hi4::set_yoyo_callbacks(fp);
    }
}
