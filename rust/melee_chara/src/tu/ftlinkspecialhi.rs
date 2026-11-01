//! src/melee/ft/kinds/ftLink/ftlinkspecialhi.c: Link's Spin Attack (up B).
//! No fused sites (tools/rust-port/fma_sites.py): the only float arithmetic is `ftLk_SpecialAirHi_Phys`
//! (checked by tests/chars_ref.rs) and `self_vel.x *= x34`. Smoke-tested by
//! tests/tu_ftlinkspecialhi.rs.
//!
//! `ftLk_SpecialAirHi_Coll` is written in the C as `Fighter* fp = GET_FIGHTER(gobj)->dat_attrs`
//! (a typo for the attribute block) and then reads `fp->facing_dir1`, which is the Fighter field
//! at GC +0x30: that is `ftLk_DatAttrs::x30` of the block (the same field `AirHi_Anim` uses).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da as da_g;
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
#[allow(unused_imports)]
use melee_types::{ftLk_DatAttrs, Vec3};
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 3, 4 and the hand-written `chars::ftLink`): re-exported under this TU.




/// Calls into TUs that are not ported yet (`efSync_Spawn` is shared with ftcaptainspecialn).
pub mod ext {
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn da<'a>(fp: *mut Fighter) -> &'a ftLk_DatAttrs {
    &*((*fp).dat_attrs as *const ftLk_DatAttrs)
}

#[inline(always)]
unsafe fn part_joint(fp: *mut Fighter, part: Fighter_Part) -> *mut core::ffi::c_void {
    let idx = fh::ftparts_get_bone_index(&*fp, part as i32);
    (*(*fp).parts.offset(idx as isize)).joint as *mut core::ffi::c_void
}

/// Ft_MF_KeepGfx | SkipHit | SkipMatAnim | SkipColAnim | UpdateCmd | SkipItemVis | Unk19 |
/// SkipModelPartVis | SkipModelFlags | Unk27.
const coll_mf: u32 = (Ft_MF_KeepGfx
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

/// `onAccessory4` (static): spawn the spin effect once, then clear the callback.
pub fn onAccessory4(gobj: *mut HSD_GObj) {
    melee_core::ct!("onAccessory4");
    unsafe {
        let fp = fpp(gobj);
        let jobj0 = part_joint(fp, FtPart_TransN);
        let jobj1 = if (*fp).kind == Ft_Kind_Link {
            (*(*fp).parts.add(FtPart_L2ndNa as usize)).joint as *mut core::ffi::c_void
        } else {
            (*(*fp).parts.add(FtPart_L3rdNa as usize)).joint as *mut core::ffi::c_void
        };
        if (*fp).x2219_b0() == 0 {
            if (*fp).ground_or_air == GA_Ground {
                ext::efSync_Spawn(1211, gobj, &[jobj0, jobj1]);
            } else {
                ext::efSync_Spawn(1212, gobj, &[jobj0, jobj1]);
            }
            (*fp).set_x2219_b0(1);
        }
        // Fighter_SetEffectHitlagCallbacks
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
        (*fp).accessory4_cb = None;
    }
}

/// `onAccessory4` as a Fighter callback entry.
pub unsafe extern "C" fn cb_onAccessory4(g: *mut HSD_GObj) {
    melee_core::ct!("cb_onAccessory4");
    onAccessory4(g);
}

pub fn ftLk_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        st::Fighter_ChangeMotionState(gobj, ftLk_MS_SpecialAirHi as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).self_vel.x *= a.x34;
        (*fp).self_vel.y = a.x40;
        (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as _;
        (*fp).accessory4_cb = Some(cb_onAccessory4);
    }
}

pub fn ftLk_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirHi_Anim");
    unsafe {
        let a = da(fpp(gobj));
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_80096900(gobj, 1, 1, 0, a.specialairhi_drift_stick_mul, a.x30);
        }
    }
}

pub fn ftLk_SpecialHi_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLk_SpecialHi_IASA");}

pub fn ftLk_SpecialAirHi_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLk_SpecialAirHi_IASA");}

/// `ftLk_SpecialAirHi_Phys(gobj)`: the fighter-only body is [`crate::chars::ftLink::ftlinkspecialhi`].
pub fn ftLk_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirHi_Phys");
    unsafe {
        crate::chars::ftLink::ftlinkspecialhi::ftLk_SpecialAirHi_Phys(&mut *fpp(gobj));
    }
}

/// `doColl` (static): fall off the ground into the air spin, keeping the animation frame.
fn doColl(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D60C(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, ftLk_MS_SpecialAirHi as i32, coll_mf, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftLk_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialHi_Coll");
    if st::ft_80082708(gobj) == 0 {
        doColl(gobj);
    }
}

pub fn ftLk_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialAirHi_Coll");
    unsafe {
        let a = da(fpp(gobj));
        if st::ft_CheckGroundAndLedge(gobj, 0) != 0 {
            st::ftCo_LandingFallSpecial_Enter(gobj, 0, a.x30);
        } else if st::ftCliffCommon_80081298(gobj) != 0 {
            return;
        }
    }
}

pub fn ftLk_SpecialHi_ProcessPartLThumbNb(gobj: *mut HSD_GObj) -> Fighter_Part {
    melee_core::ct!("ftLk_SpecialHi_ProcessPartLThumbNb");
    unsafe { fh::ftparts_get_bone_index(&*fpp(gobj), FtPart_LThumbNb as i32) as Fighter_Part }
}

/// `ftLk_SpecialHi_GetPosWithAdjustedY(gobj, pos)`: the fighter-only body is `chars::ftLink`'s.
pub fn ftLk_SpecialHi_GetPosWithAdjustedY(gobj: *mut HSD_GObj, pos: &mut Vec3) {
    melee_core::ct!("ftLk_SpecialHi_GetPosWithAdjustedY");
    unsafe {
        crate::chars::ftLink::ftlinkspecialhi::ftLk_SpecialHi_GetPosWithAdjustedY(&*fpp(gobj), pos);
    }
}

pub fn ftLk_SpecialHi_GetFv4(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLk_SpecialHi_GetFv4");
    unsafe { (*fpp(gobj)).u.lk.x4 }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftLk_SpecialHi_Anim` (src/melee/ft/kinds/ftLink/ftlinkspecialhi.c).
pub fn ftLk_SpecialHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialHi_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftLk_SpecialHi_Enter` (src/melee/ft/kinds/ftLink/ftlinkspecialhi.c).
pub fn ftLk_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialHi_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 356 /* ftLk_MS_SpecialHi */, 0x0u32 /* Ft_MF_None */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftlinkspecialhi__onAccessory4);
    }
}

/// `ftLk_SpecialHi_Phys` (src/melee/ft/kinds/ftLink/ftlinkspecialhi.c).
pub fn ftLk_SpecialHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLk_SpecialHi_Phys");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (((*fp).ground_or_air as i32) == 1i32 /* GA_Air */) {
            crate::chars::ftLink::ftlinkspecialhi::ftLk_SpecialAirHi_Phys(&mut *fp);
        } else {
            st::ft_80084F3C(gobj);
        }
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftLk_SpecialAirHi_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftLk_SpecialAirHi_Phys_fp");
    let a: &ftLk_DatAttrs = da_g(fp);
    co::fall(fp, fp.co_attrs.gravity * a.specialhi_grav_mul, fp.co_attrs.terminal_velocity);
    co::calc_self_accel_drift_simple(
        fp,
        0.0,
        fp.co_attrs.air_drift_stick_mul * a.specialairhi_drift_stick_mul,
        fp.co_attrs.air_drift_max * a.specialairhi_drift_max_mul,
    );
}

pub fn ftLk_SpecialHi_GetPosWithAdjustedY_fp(fp: &Fighter, pos: &mut Vec3) {
    melee_core::ct!("ftLk_SpecialHi_GetPosWithAdjustedY_fp");
    let a: &ftLk_DatAttrs = da_g(fp);
    *pos = fp.cur_pos;
    pos.y += a.specialhi_pos_y_offset;
}
