//! src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c: Falcon Kick / Wizard's Foot (down B).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested
//! through recording hooks by tests/tu_ftcaptainspeciallw.rs.
//!
//! Note: the round-11 `ftCa_SpecialLw_Enter` installs `chars::cb::ftcaptainspeciallw__
//! ftCa_SpecialHi_800E400C`, still a TODO panic; [`cb_ftCa_SpecialHi_800E400C`] here is the
//! native entry that a later pass should point it at.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::Fighter;
#[allow(unused_imports)]
use melee_types::{HSD_GObj};
use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 4, 5, 7, 11 and the pure helper): re-exported under this TU.







/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except
/// `ftPartSetRotZ`, which falls back to the native `melee_fighter::tu::ftparts` port.
pub mod ext {
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
    crate::port_hook!(EFASYNC_SPAWN, efAsync_Spawn, crate::adapt::efAsync_Spawn_ptr, (gobj: *mut HSD_GObj, q: *mut core::ffi::c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj, angle: *const f32));
    pub static mut FTCOMMON_8007E5AC: Option<fn(*mut Fighter)> = None;
    #[allow(non_snake_case)]
    pub fn ftCommon_8007E5AC(fp: *mut Fighter) {
        match unsafe { FTCOMMON_8007E5AC } {
            Some(f) => f(fp),
            None => { unsafe { melee_fighter::tu::ftcommon::ftcommon_8007e5ac(&mut *(fp as *mut _)); } },
        }
    }

    pub static mut FTPARTSETROTZ: Option<fn(*mut Fighter, i32, f32)> = None;
    pub fn ftPartSetRotZ(fp: *mut Fighter, part: i32, v: f32) {
        match unsafe { FTPARTSETROTZ } {
            Some(f) => f(fp, part, v),
            None => melee_fighter::tu::ftparts::ftPartSetRotZ(unsafe { &*fp }, part, v),
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftCaptain_DatAttrs {
    (*fp).dat_attrs as *const ftCaptain_DatAttrs
}
#[inline(always)]
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// `ftCa_SpecialHi_800E400C` as a Fighter callback entry (`deal_dmg_cb`).
pub unsafe extern "C" fn cb_ftCa_SpecialHi_800E400C(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftCa_SpecialHi_800E400C");
    ftCa_SpecialHi_800E400C(g);
}

/// `ftCa_Special_Inline_Check_Flag` (static inline): test and clear throw_flags_b1.
pub fn ftCa_Special_Inline_Check_Flag(fp: &mut Fighter) -> bool {
    melee_core::ct!("ftCa_Special_Inline_Check_Flag");
    unsafe {
        if fp.x2210.x0.throw_flags_b1() != 0 {
            fp.x2210.x0.set_throw_flags_b1(0);
            true
        } else {
            false
        }
    }
}

/// Flame effect on the kicking foot, once per move (then ftCommon_8007DB24 on later flags).
pub fn ftCa_SpecialHi_800E3EAC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHi_800E3EAC");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if ftCa_Special_Inline_Check_Flag(&mut *fp) {
            if (*fp).x2219_b0() == 0 {
                // Uninitialised in the C for other motions; gwtool zeroes locals.
                let mut sp1c: f32 = 0.0;
                let mut part: i32 = 0;
                if (*fp).motion_id == 0x165 {
                    sp1c = 0.0;
                    part = fh::ftparts_get_bone_index(&*fp, FtPart_RFootJA as i32);
                } else if (*fp).motion_id == 0x167 {
                    sp1c = 0.017453292f32 * (*da).speciallw_flame_particle_angle;
                    part = fh::ftparts_get_bone_index(&*fp, FtPart_LFootJA as i32);
                }
                let q = &mut (*fp).x60C as *mut _ as *mut core::ffi::c_void;
                let joint = (*(*fp).parts.offset(part as isize)).joint;
                match fh::ftlib_get_kind(gobj) {
                    Ft_Kind_Captain => ext::efAsync_Spawn(gobj, q, 3, 0x490, joint, &sp1c),
                    Ft_Kind_Ganon => ext::efAsync_Spawn(gobj, q, 3, 0x50C, joint, &sp1c),
                    _ => {}
                }
                (*fp).set_x2219_b0(1);
                return;
            }
            st::ftCommon_8007DB24(gobj);
        }
    }
}

/// On-hit callback: count hits and slow the kick down.
pub fn ftCa_SpecialHi_800E400C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHi_800E400C");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if ((*fp).mv.ca.speciallw.x0 as i32) <= (*da).speciallw_unk2 {
            (*fp).mv.ca.speciallw.x0 = (*fp).mv.ca.speciallw.x0.wrapping_add(1);
            (*fp).mv.ca.speciallw.friction *= (*da).speciallw_on_hit_spd_modifier;
        }
    }
}

/// `ftCa_SpecialLw_Anim_inline` (static inline): `air` picks SpecialLwEndAir.
pub fn ftCa_SpecialLw_Anim_inline(gobj: *mut HSD_GObj, air: i32) {
    melee_core::ct!("ftCa_SpecialLw_Anim_inline");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        resetCmdAndThrow(&mut *fp);
        if air == 0 {
            st::ftCommon_8007D7FC(&mut *fp);
            st::Fighter_ChangeMotionState(gobj, ftCa_MS_SpecialLwEnd as i32, Ft_MF_None as u32, 0.0, (*da).speciallw_ground_lag_mul, 0.0, core::ptr::null_mut());
        } else {
            st::ftCommon_8007D5D4(&mut *fp);
            st::Fighter_ChangeMotionState(gobj, ftCa_MS_SpecialLwEndAir as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        }
        set_effect_hitlag_callbacks(fp);
    }
}

pub fn ftCa_SpecialLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLw_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftCa_SpecialLw_Anim_inline(gobj, ((*fp).ground_or_air != GA_Ground) as i32);
        }
    }
}

/// `ftCa_Special_Inline_Friction` (static inline).
pub fn ftCa_Special_Inline_Friction(fp: &mut Fighter) {
    melee_core::ct!("ftCa_Special_Inline_Friction");
    let friction = unsafe { fp.mv.ca.speciallw.friction };
    fp.self_vel.x *= friction;
    fp.self_vel.y *= friction;
}

pub fn ftCa_SpecialLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLw_Phys");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Ground {
            ext::ftCommon_8007E5AC(fp);
            st::ft_80085088(gobj);
        } else {
            ext::ftPartSetRotZ(fp, 0, 0.0);
            st::ft_80085134(gobj);
        }
        ftCa_Special_Inline_Friction(&mut *fp);
        ftCa_SpecialHi_800E3EAC(gobj);
    }
}

pub fn ftCa_SpecialLwEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLwEnd_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).ground_or_air == GA_Ground {
            ext::ftCommon_8007E5AC(fp);
            if (*fp).cmd_vars[2] != 0 {
                let fr = (*da).speciallw_ground_traction * (*fp).co_attrs.ground_friction;
                fc::calc_ground_accel_deaccel(&mut *fp, fr);
                fc::set_self_movement_from_grounded_movement(&mut *fp);
            } else {
                st::ft_80084F3C(gobj);
            }
        } else {
            ext::ftPartSetRotZ(fp, 0, 0.0);
            st::ft_80084EEC(gobj);
        }
        ftCa_Special_Inline_Friction(&mut *fp);
    }
}

pub fn ftCa_SpecialLwEndAir_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLwEndAir_Phys");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Ground {
            ext::ftCommon_8007E5AC(fp);
            st::ft_80085088(gobj);
            return;
        }
        ext::ftPartSetRotZ(fp, 0, 0.0);
        if (*fp).cmd_vars[0] != 0 {
            st::ft_80084EEC(gobj);
        } else {
            st::ft_80085134(gobj);
        }
    }
}

pub fn ftCa_SpecialAirLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirLw_Phys");
    st::ft_80085134(gobj);
    ftCa_SpecialHi_800E3EAC(gobj);
}

pub fn ftCa_SpecialAirLwEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirLwEnd_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[2] != 0 {
            let fr = (*da).speciallw_air_landing_traction * (*fp).co_attrs.ground_friction;
            fc::calc_ground_accel_deaccel(&mut *fp, fr);
            fc::set_self_movement_from_grounded_movement(&mut *fp);
        } else {
            st::ft_80084F3C(gobj);
        }
    }
}

pub fn ftCa_SpecialLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLw_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Ground {
            if st::ft_80082708(gobj) == 0 {
                st::ftCommon_8007D5D4(&mut *fp);
            }
        } else if st::ft_80081D0C(gobj) != 0 {
            st::ftCommon_8007D7FC(&mut *fp);
        }
        let env = (*fp).coll_data.env_flags as i32;
        if (*fp).cmd_vars[0] != 0
            && (((*fp).facing_dir == -1.0 && (env & Collide_RightWallHug as i32) != 0)
                || ((*fp).facing_dir == 1.0 && (env & Collide_LeftWallHug as i32) != 0))
        {
            ftCa_Special_Inline_SetFlags(gobj);
            (*fp).x2210.throw_flags = 0;
            st::ftCommon_8007D5D4(&mut *fp);
            st::Fighter_ChangeMotionState(gobj, ftCa_MS_SpecialHiThrow1 as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        }
    }
}

/// `doColl` (static): land into `msid` when air collision reports ground.
pub fn doColl(gobj: *mut HSD_GObj, msid: i32) {
    melee_core::ct!("doColl");
    if st::ft_80081D0C(gobj) != GA_Ground as i32 {
        unsafe {
            let fp = fpp(gobj);
            let da = da(fp);
            resetCmdAndThrow(&mut *fp);
            st::ftCommon_8007D7FC(&mut *fp);
            st::Fighter_ChangeMotionState(gobj, msid, Ft_MF_None as u32, 0.0, (*da).speciallw_landing_lag_mul, 0.0, core::ptr::null_mut());
        }
    }
}

pub fn ftCa_SpecialAirLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirLw_Coll");
    doColl(gobj, ftCa_MS_SpecialAirLwEnd as i32);
}

pub fn ftCa_SpecialAirLwEndAir_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirLwEndAir_Coll");
    doColl(gobj, ftCa_MS_SpecialAirLwEnd as i32);
}

// ---- folded from chars (consolidation step 20/21) ----
/// `resetCmdAndThrow` (static).
pub fn resetCmdAndThrow(fp: &mut Fighter) {
    melee_core::ct!("resetCmdAndThrow");
    fp.cmd_vars[2] = 0;
    fp.cmd_vars[1] = 0;
    fp.cmd_vars[0] = 0;
    fp.x2210.throw_flags = 0;
}

/// `ftCa_SpecialLw_Enter` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLw_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[2] = 0_u32;
        (*fp).cmd_vars[1] = 0_u32;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).x2210.throw_flags = 0_u32;
        (*fp).mv.ca.speciallw.x0 = (0_u16 /* 0.0F */);
        (*fp).mv.ca.speciallw.friction = f32::from_bits(0x3F800000) /* 1.0F */;
        st::Fighter_ChangeMotionState(gobj, 357 /* ftCa_MS_SpecialLw */, 0x0u32 /* Ft_MF_None */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).deal_dmg_cb = Some(crate::chars::cb::ftcaptainspeciallw__ftCa_SpecialHi_800E400C);
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
    }
}

/// `ftCa_SpecialAirLw_Enter` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirLw_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[2] = 0_u32;
        (*fp).cmd_vars[1] = 0_u32;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).x2210.throw_flags = 0_u32;
        st::Fighter_ChangeMotionState(gobj, 359 /* ftCa_MS_SpecialAirLw */, 0x0u32 /* Ft_MF_None */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
    }
}

/// `ftCa_SpecialAirLwEnd_Anim` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialAirLwEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirLwEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftCa_SpecialAirLwEndAir_Anim` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialAirLwEndAir_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirLwEndAir_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftCa_SpecialHiThrow1_Anim` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialHiThrow1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHiThrow1_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftCa_SpecialAirLwEndAir_Phys` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialAirLwEndAir_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirLwEndAir_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftCa_SpecialHiThrow1_Phys` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialHiThrow1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHiThrow1_Phys");
    st::ft_80085134(gobj);
}

/// `ftCa_Special_Inline_SetFlags` (static) (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_Special_Inline_SetFlags(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_Special_Inline_SetFlags");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[2] = 0_u32;
        (*fp).cmd_vars[1] = 0_u32;
        (*fp).cmd_vars[0] = 0_u32;
    }
}

/// `ftCa_SpecialLwEnd_Coll` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialLwEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLwEnd_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (((*fp).ground_or_air as i32) == 0i32 /* GA_Ground */) {
            if (((*fp).cmd_vars[1] as u32) != 0u32 /* 0 */) {
                if (st::ft_800827A0(gobj) == 0) {
                    st::ftCommon_8007D5D4(&mut *fp);
                }
            } else {
                if (st::ft_80082708(gobj) == 0) {
                    st::ftCommon_8007D5D4(&mut *fp);
                }
            }
        } else {
            if (st::ft_80081D0C(gobj) != 0) {
                st::ftCommon_8007D7FC(&mut *fp);
            }
        }
    }
}

/// `ftCa_SpecialLwEndAir_Coll` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialLwEndAir_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLwEndAir_Coll");
    unsafe {
        crate::chars::r4::ftCaptain::ftcaptainspeciallw::ftCa_SpecialLwEnd_Coll(gobj);
    }
}

/// `ftCa_SpecialLwEnd_Anim` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialLwEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLwEnd_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            st::ftCommon_8007D92C(gobj);
        }
    }
}

/// `ftCa_SpecialLwEndAir_Anim` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialLwEndAir_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLwEndAir_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            st::ftCommon_8007D92C(gobj);
        }
    }
}

/// `ftCa_SpecialAirLwEnd_Coll` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialAirLwEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirLwEnd_Coll");
    unsafe {
        st::ft_80084104(gobj);
    }
}

/// `ftCa_SpecialAirLw_Anim_inline` (static) (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialAirLw_Anim_inline(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirLw_Anim_inline");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[2] = 0_u32;
        (*fp).cmd_vars[1] = 0_u32;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).x2210.throw_flags = 0_u32;
    }
}

/// `ftCa_SpecialAirLw_Anim` (src/melee/ft/kinds/ftCaptain/ftcaptainspeciallw.c).
pub fn ftCa_SpecialAirLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirLw_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            crate::chars::r7::ftCaptain::ftcaptainspeciallw::ftCa_SpecialAirLw_Anim_inline(gobj);
            st::ftCommon_8007D5D4(&mut *fp);
            st::Fighter_ChangeMotionState(gobj, 361 /* ftCa_MS_SpecialAirLwEndAir */, 0x0u32 /* Ft_MF_None */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        }
    }
}
