//! src/melee/ft/kinds/ftMars/ftmarsspeciallw.c: Counter (down B).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested by
//! tests/tu_ftmarsspeciallw.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 3, 13): re-exported under this TU.




/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, lb_800119DC, EFSYNC_SPAWN, LB_800119DC};
    pub use crate::tu::ftclink::ext::{ftColl_8007B1B8, FTCOLL_8007B1B8};
    pub use crate::tu::ftcrazyhandfingerbeam0::ext::{lb_8000B1CC, LB_8000B1CC};
    pub static mut FTCOLL_8007ABD0: Option<fn(*mut HitCapsule, u32, *mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftColl_8007ABD0(hit: *mut HitCapsule, damage: u32, gobj: *mut HSD_GObj) {
        match unsafe { FTCOLL_8007ABD0 } {
            Some(f) => f(hit, damage, gobj),
            None => crate::adapt::ftColl_8007ABD0(hit, damage, gobj),
        }
    }
}

/// `ftCommon_GroundAirColl_MF | Ft_MF_KeepColAnimHitStatus | Ft_MF_SkipHit`.
const ftMs_MF_SpecialLw_Coll: u32 = 0xC4C508C;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut MarsAttributes {
    (*fp).dat_attrs as *mut MarsAttributes
}

/// `ftMs_SpecialLw_80139140` as a Fighter callback entry (the shield-hit callback).
pub unsafe extern "C" fn cb_ftMs_SpecialLw_80139140(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMs_SpecialLw_80139140");
    ftMs_SpecialLw_80139140(g);
}

pub fn ftMs_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLw_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fp).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, 369, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cmd_vars[1] = 0;
        (*fp).mv.ms.speciallw.x0 = 0;
    }
}

pub fn ftMs_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirLw_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).self_vel.x /= (*da).x4C;
        (*fp).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, 371, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cmd_vars[1] = 0;
        (*fp).mv.ms.speciallw.x0 = 0;
    }
}

/// Shield-window handling both Anim callbacks share.
unsafe fn shield_window(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    let da = da(fp);
    if (*fp).cmd_vars[1] == 1 {
        (*fp).cmd_vars[1] = 2;
        ext::ftColl_8007B1B8(gobj, &mut (*da).x64 as *mut AbsorbDesc as *mut _, Some(cb_ftMs_SpecialLw_80139140));
        (*fp).anon_221b.set_x221B_b1(1);
        (*fp).shield_unk0 = (*da).x60;
        (*fp).shield_unk1 = (*da).x60;
    } else if (*fp).cmd_vars[1] == 0 {
        (*fp).anon_221b.set_x221B_b0(0);
    }
}

pub fn ftMs_SpecialLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLw_Anim");
    unsafe {
        let fp = fpp(gobj);
        shield_window(gobj, fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftMs_SpecialAirLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirLw_Anim");
    unsafe {
        let fp = fpp(gobj);
        shield_window(gobj, fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftMs_SpecialLw_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMs_SpecialLw_IASA");}

pub fn ftMs_SpecialAirLw_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMs_SpecialAirLw_IASA");}

pub fn ftMs_SpecialAirLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirLw_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        fc::fall(&mut *fp, (*da).x54, (*da).x58);
        fc::calc_self_accel_deaccel(&mut *fp, (*da).x50);
        st::ftColl_8007AEE0(gobj);
    }
}

pub fn ftMs_SpecialLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLw_Coll");
    if st::ft_800827A0(gobj) == 0 {
        ftMs_SpecialLw_80138D38(gobj);
    }
}

pub fn ftMs_SpecialAirLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirLw_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftMs_SpecialLw_80138DD0(gobj);
    }
}

/// Re-arm the shield window after a ground/air transition while it is open.
unsafe fn rearm_shield(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let da = da(fp);
    if (*fp).cmd_vars[1] == 2 {
        ext::ftColl_8007B1B8(gobj, &mut (*da).x64 as *mut AbsorbDesc as *mut _, Some(cb_ftMs_SpecialLw_80139140));
        (*fp).anon_221b.set_x221B_b1(1);
    }
}

pub fn ftMs_SpecialLw_80138D38(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLw_80138D38");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 371, ftMs_MF_SpecialLw_Coll);
        rearm_shield(gobj);
    }
}

pub fn ftMs_SpecialLw_80138DD0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLw_80138DD0");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 369, ftMs_MF_SpecialLw_Coll);
        rearm_shield(gobj);
    }
}

/// `ftMars_SpecialLwHit_ApplyDamage` (static inline): the Emblem (Roy) counter scales the live
/// hitboxes by the countered damage.
pub fn ftMars_SpecialLwHit_ApplyDamage(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMars_SpecialLwHit_ApplyDamage");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.ms.speciallw.x0 > 0 && fh::ftlib_get_kind(gobj) == Ft_Kind_Emblem {
            for i in 0..4 {
                if (*fp).x914[i].state == HitCapsule_Enabled {
                    ext::ftColl_8007ABD0(&mut (*fp).x914[i], (*fp).mv.ms.speciallw.x0 as u32, gobj);
                }
            }
        }
    }
}

pub fn ftMs_SpecialLwHit_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLwHit_Anim");
    ftMars_SpecialLwHit_ApplyDamage(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

pub fn ftMs_SpecialAirLwHit_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirLwHit_Anim");
    ftMars_SpecialLwHit_ApplyDamage(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

pub fn ftMs_SpecialLwHit_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMs_SpecialLwHit_IASA");}

pub fn ftMs_SpecialAirLwHit_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMs_SpecialAirLwHit_IASA");}

/// `ftMs_SpecialLw_80139140_inline` (static inline): the counter effect once, then drop the
/// hitlag and accessory callbacks.
pub fn ftMs_SpecialLw_80139140_inline(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLw_80139140_inline");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            let gfx = match fh::ftlib_get_kind(gobj) {
                Ft_Kind_Mars => Some(1265),
                Ft_Kind_Emblem => Some(1296),
                _ => None,
            };
            if let Some(gfx) = gfx {
                let idx = fh::ftparts_get_bone_index(&*fp, FtPart_RShoulderN as i32);
                let joint = (*(*fp).parts.offset(idx as isize)).joint as *mut core::ffi::c_void;
                let facing = &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void;
                ext::efSync_Spawn(gfx, gobj, &[joint, facing]);
            }
            (*fp).set_x2219_b0(1);
        }
        (*fp).pre_hitlag_cb = None;
        (*fp).post_hitlag_cb = None;
        (*fp).accessory4_cb = None;
    }
}

/// Shield-hit callback: take the countered damage, spawn the effect and enter the hit state.
pub fn ftMs_SpecialLw_80139140(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLw_80139140");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).facing_dir = (*fp).specialn_facing_dir;
        let temp_r0: i32 = (*fp).x19A4;
        if temp_r0 > 0 {
            (*fp).mv.ms.speciallw.x0 = melee_core::ppc::fctiwz_f32(temp_r0 as f32 * (*da).x5C);
        }
        let idx = fh::ftparts_get_bone_index(&*fp, FtPart_HipN as i32);
        let mut sp18: Vec3 = core::mem::zeroed();
        ext::lb_8000B1CC((*(*fp).parts.offset(idx as isize)).joint, core::ptr::null(), &mut sp18);
        ext::lb_800119DC(&mut sp18, 120, 0.9f64 as f32, 0.02f64 as f32, (core::f64::consts::PI / 3.0) as f32);
        let msid: i32 = if (*fp).ground_or_air == GA_Ground { 370 } else { 372 };
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftMs_SpecialLw_80139140_inline(gobj);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMs_SpecialLw_Phys` (src/melee/ft/kinds/ftMars/ftmarsspeciallw.c).
pub fn ftMs_SpecialLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLw_Phys");
    unsafe {
        st::ft_80084F3C(gobj);
        st::ftColl_8007AEE0(gobj);
    }
}

/// `ftMs_SpecialLwHit_Phys` (src/melee/ft/kinds/ftMars/ftmarsspeciallw.c).
pub fn ftMs_SpecialLwHit_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLwHit_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftMs_SpecialAirLwHit_Phys` (src/melee/ft/kinds/ftMars/ftmarsspeciallw.c).
pub fn ftMs_SpecialAirLwHit_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirLwHit_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftMs_SpecialLw_80139080` (src/melee/ft/kinds/ftMars/ftmarsspeciallw.c).
pub fn ftMs_SpecialLw_80139080(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLw_80139080");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 372, 0xC4C508Eu32 /* ftMs_MF_SpecialLwHit_Coll */);
}

/// `ftMs_SpecialLw_801390E0` (src/melee/ft/kinds/ftMars/ftmarsspeciallw.c).
pub fn ftMs_SpecialLw_801390E0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLw_801390E0");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 370, 0xC4C508Eu32 /* ftMs_MF_SpecialLwHit_Coll */);
}

/// `ftMs_SpecialLwHit_Coll` (src/melee/ft/kinds/ftMars/ftmarsspeciallw.c).
pub fn ftMs_SpecialLwHit_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialLwHit_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 {
            crate::chars::r2::ftMars::ftmarsspeciallw::ftMs_SpecialLw_80139080(gobj);
        }
    }
}

/// `ftMs_SpecialAirLwHit_Coll` (src/melee/ft/kinds/ftMars/ftmarsspeciallw.c).
pub fn ftMs_SpecialAirLwHit_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirLwHit_Coll");
    unsafe {
        if st::ft_80081D0C(gobj) != 0 {
            crate::chars::r2::ftMars::ftmarsspeciallw::ftMs_SpecialLw_801390E0(gobj);
        }
    }
}
