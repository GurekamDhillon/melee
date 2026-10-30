//! src/melee/ft/kinds/ftCaptain/ftcaptainspecials.c: Raptor Boost / Gerudo Dragon (side B).
//! No fused sites (tools/rust-port/fma_sites.py). The gravity math is checked against the C by
//! tests/tu_ftcaptainspecials.rs; the rest has a smoke test through recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 4, 6): re-exported under this TU.
pub use crate::chars::r2::ftCaptain::ftcaptainspecials::{
    ftCa_SpecialSStart_Anim, ftCa_SpecialSStart_Phys, ftCa_SpecialS_Phys,
};
pub use crate::chars::r4::ftCaptain::ftcaptainspecials::{onDetectAir, setCallbacks};
pub use crate::chars::r6::ftCaptain::ftcaptainspecials::{
    ftCa_SpecialAirSStart_Coll, ftCa_SpecialAirS_Coll, ftCa_SpecialS_RemoveGFX,
};

/// Calls into TUs that are not ported yet (same hooks as ftcaptainspecialn's).
pub mod ext {
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
}

/// `ftCa_SpecialS_OnDetect` as a Fighter callback entry (`hurtbox_detect_cb`).
pub unsafe extern "C" fn cb_ftCa_SpecialS_OnDetect(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftCa_SpecialS_OnDetect");
    ftCa_SpecialS_OnDetect(g);
}

/// Ft_MF_KeepGfx | SkipMatAnim | UpdateCmd | SkipColAnim | SkipItemVis | Unk19 |
/// SkipModelPartVis | SkipModelFlags | Unk27.
const transition_flags: u32 = (Ft_MF_KeepGfx
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
unsafe fn joint(fp: *mut Fighter, part: Fighter_Part) -> *mut core::ffi::c_void {
    (*(*fp).parts.add(part as usize)).joint as *mut core::ffi::c_void
}

#[inline(always)]
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// `resetCmdVarsGround` (static).
pub fn resetCmdVarsGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("resetCmdVarsGround");
    unsafe {
        let fp = fpp(gobj);
        (*fp).cmd_vars = [0; 4];
        st::ftCommon_8007D7FC(&mut *fp);
    }
}

/// `resetVel` (static inline).
pub fn resetVel(fp: &mut Fighter) {
    melee_core::ct!("resetVel");
    fp.self_vel.z = 0.0;
    fp.self_vel.y = 0.0;
    fp.self_vel.x = 0.0;
}

/// The start-up effect both entry paths spawn (`switch (ftLib_GetKind(gobj))`).
unsafe fn spawn_start_effect(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    match fh::ftlib_get_kind(gobj) {
        Ft_Kind_Captain => {
            ext::efSync_Spawn(1169, gobj, &[joint(fp, FtPart_HeadN)]);
            (*fp).u.ca.during_specials_start = 1;
        }
        Ft_Kind_Ganon => {
            ext::efSync_Spawn(1293, gobj, &[joint(fp, FtPart_L2ndNb)]);
            (*fp).u.ca.during_specials_start = 1;
        }
        _ => {}
    }
}

pub fn ftCa_SpecialS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialS_Enter");
    unsafe {
        let fp = fpp(gobj);
        resetCmdVarsGround(gobj);
        st::Fighter_ChangeMotionState(gobj, ftCa_MS_SpecialSStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        setCallbacks(gobj);
        st::ftAnim_8006EBA4(gobj);
        spawn_start_effect(gobj, fp);
        (*fp).u.ca.during_specials = 0;
        set_effect_hitlag_callbacks(fp);
        (*fp).hurtbox_detect_cb = Some(cb_ftCa_SpecialS_OnDetect);
        resetVel(&mut *fp);
        (*fp).gr_vel = 0.0;
    }
}

/// `setupAirStart` (static inline).
pub fn setupAirStart(gobj: *mut HSD_GObj) {
    melee_core::ct!("setupAirStart");
    unsafe {
        let fp = fpp(gobj);
        (*fp).cmd_vars = [0; 4];
        st::Fighter_ChangeMotionState(gobj, ftCa_MS_SpecialAirSStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        setCallbacks(gobj);
        st::ftAnim_8006EBA4(gobj);
        spawn_start_effect(gobj, fp);
        (*fp).u.ca.during_specials = 0;
        set_effect_hitlag_callbacks(fp);
        (*fp).hurtbox_detect_cb = Some(cb_ftCa_SpecialS_OnDetect);
        resetVel(&mut *fp);
    }
}

pub fn ftCa_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirS_Enter");
    unsafe {
        let fp = fpp(gobj);
        setupAirStart(gobj);
        (*fp).mv.ca.specials.grav = 0.0;
        st::ftCommon_8007D60C(&mut *fp);
    }
}

/// `onDetectGround` (static).
pub fn onDetectGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("onDetectGround");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftCaptain_DatAttrs;
        st::ftCommon_8007D7FC(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, ftCa_MS_SpecialS as i32, transition_flags, 0.0, 1.0, 0.0, core::ptr::null_mut());
        setCallbacks(gobj);
        (*fp).self_vel.z = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).gr_vel *= (*sa).specials_gr_vel_x;
    }
}

#[inline]
unsafe fn on_detect_by_motion(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    match (*fp).motion_id {
        m if m == ftCa_MS_SpecialSStart as FtMotionId => onDetectGround(gobj),
        m if m == ftCa_MS_SpecialAirSStart as FtMotionId => onDetectAir(gobj),
        _ => {}
    }
}

pub fn ftCa_SpecialS_OnDetect(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialS_OnDetect");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] != 0 {
            let detected_gobj = (*fp).unk_gobj;
            if (*(*fp).unk_gobj).classifier as i32 == HSD_GOBJ_CLASS_FIGHTER {
                on_detect_by_motion(gobj, fp);
            } else if (*(*fp).unk_gobj).classifier as i32 == HSD_GOBJ_CLASS_ITEM {
                let k = |g| fh::it_get_kind(g);
                if k(detected_gobj) < It_Kind_BombHei as i32 {
                    on_detect_by_motion(gobj, fp);
                } else if (k(detected_gobj) >= It_Kind_Kuriboh as i32 && k(detected_gobj) < It_Kind_Octarock_Stone as i32)
                    || (k(detected_gobj) >= It_Kind_Old_Kuri as i32 && k(detected_gobj) < It_Kind_Arwing_Laser as i32)
                    || k(detected_gobj) == It_PKind_Random as i32
                {
                    on_detect_by_motion(gobj, fp);
                }
            }
        }
    }
}

/// The dash effect of `ftCa_SpecialS_Anim` / `ftCa_SpecialAirS_Anim` (ids differ per state).
unsafe fn dash_effect(gobj: *mut HSD_GObj, fp: *mut Fighter, captain: i32, ganon: i32) {
    if (*fp).u.ca.during_specials == 0 {
        let facing = &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void;
        match fh::ftlib_get_kind(gobj) {
            Ft_Kind_Captain => {
                ext::efSync_Spawn(captain, gobj, &[joint(fp, FtPart_TransN), facing]);
                (*fp).u.ca.during_specials = 1;
            }
            Ft_Kind_Ganon => {
                ext::efSync_Spawn(ganon, gobj, &[joint(fp, FtPart_TransN), facing]);
                (*fp).u.ca.during_specials = 1;
            }
            _ => {}
        }
        set_effect_hitlag_callbacks(fp);
    }
}

pub fn ftCa_SpecialS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialS_Anim");
    unsafe {
        let fp = fpp(gobj);
        dash_effect(gobj, fp, 1170, 1294);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

/// Fall, or FallSpecial with `lag` landing lag (shared tail of the aerial Anim callbacks).
unsafe fn fall_or_special(gobj: *mut HSD_GObj, fp: *mut Fighter, lag: f32) {
    st::ftCommon_8007D60C(&mut *fp);
    if lag == 0.0 {
        st::ftCo_Fall_Enter(gobj);
    } else {
        st::ftCo_80096900(gobj, 1, 1, 0, 1.0, lag);
    }
}

pub fn ftCa_SpecialAirSStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirSStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftCaptain_DatAttrs;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            fall_or_special(gobj, fp, (*da).specials_miss_landing_lag);
        }
    }
}

pub fn ftCa_SpecialAirS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirS_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftCaptain_DatAttrs;
        dash_effect(gobj, fp, 1171, 1295);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            fall_or_special(gobj, fp, (*da).specials_hit_landing_lag);
        }
    }
}

pub fn ftCa_SpecialSStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCa_SpecialSStart_IASA");}
pub fn ftCa_SpecialS_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCa_SpecialS_IASA");}
pub fn ftCa_SpecialAirSStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCa_SpecialAirSStart_IASA");}
pub fn ftCa_SpecialAirS_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCa_SpecialAirS_IASA");}

/// Gravity step of the aerial states (the fighter-only half of the Phys callbacks).
pub fn specials_gravity(fp: &mut Fighter) {
    melee_core::ct!("specials_gravity");
    let da: &ftCaptain_DatAttrs = unsafe { &*(fp.dat_attrs as *const ftCaptain_DatAttrs) };
    unsafe {
        fp.mv.ca.specials.grav -= da.specials_grav;
        if fp.mv.ca.specials.grav < -da.specials_terminal_vel {
            fp.mv.ca.specials.grav = -da.specials_terminal_vel;
        }
        fp.self_vel.y = fp.mv.ca.specials.grav;
    }
}

pub fn ftCa_SpecialAirSStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirSStart_Phys");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80085134(gobj);
        if (*fp).cmd_vars[1] == 1 {
            specials_gravity(&mut *fp);
        }
    }
}

pub fn ftCa_SpecialAirS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirS_Phys");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80085134(gobj);
        specials_gravity(&mut *fp);
    }
}

/// Captain Falcon & Ganondorf's grounded Raptor Boost / Gerudo Dragon start collision.
pub fn ftCa_SpecialSStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialSStart_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftCaptain_DatAttrs;
        if (*fp).cmd_vars[2] == 0 {
            st::ft_80084104(gobj);
            return;
        }
        if st::ft_80082708(gobj) == 0 {
            st::efLib_DestroyAll(gobj);
            st::ftCommon_8007D60C(&mut *fp);
            if (*da).specials_miss_landing_lag == 0.0 {
                st::ftCo_Fall_Enter(gobj);
                return;
            }
            st::ftCommon_ClampAirDrift(&mut *fp);
            st::ftCo_80096900(gobj, 1, 1, 0, 1.0, (*da).specials_miss_landing_lag);
            return;
        }
        if (*fp).cmd_vars[0] == 1 {
            let env = (*fp).coll_data.env_flags as i32;
            if ((*fp).facing_dir == 1.0 && (env & Collide_LeftWallMask) != 0)
                || ((*fp).facing_dir == -1.0 && (env & Collide_RightWallMask) != 0)
            {
                st::efLib_DestroyAll(gobj);
                st::ft_8008A2BC(gobj);
            }
        }
    }
}

pub fn ftCa_SpecialS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialS_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftCaptain_DatAttrs;
        if st::ft_80082708(gobj) == 0 {
            st::efLib_DestroyAll(gobj);
            st::ftCommon_8007D60C(&mut *fp);
            if (*da).specials_hit_landing_lag == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCommon_ClampAirDrift(&mut *fp);
                st::ftCo_80096900(gobj, 1, 1, 0, 1.0, (*da).specials_hit_landing_lag);
            }
        }
    }
}
