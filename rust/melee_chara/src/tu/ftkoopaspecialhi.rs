//! src/melee/ft/kinds/ftKoopa/ftkoopaspecialhi.c: Bowser Bomb (up B).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). `atan2f` is the
//! game's (melee_runtime lbtrigf, as ftCo_Damage uses). Smoke-tested through recording hooks by
//! tests/tu_ftkoopaspecialhi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftkoopaspecialn::ftKp_SpecialN_80135780;
use melee_fighter::ftcommon as fc;
use melee_types::*;

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
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing pointers as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    pub static mut FTPARTSETROTX: Option<fn(*mut Fighter, i32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn ftPartSetRotX(fp: *mut Fighter, part: i32, v: f32) {
        match unsafe { FTPARTSETROTX } {
            Some(f) => f(fp, part, v),
            None => { unsafe { melee_fighter::tu::ftparts::ftPartSetRotX(&*(fp as *const _), part as _, v as _); } },
        }
    }
    /// Recording hook; unset, the native `melee_fighter::tu::ftcliffcommon` port runs.
    pub static mut FTCLIFFCOMMON_80081370: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCliffCommon_80081370(gobj: *mut HSD_GObj) {
        match unsafe { FTCLIFFCOMMON_80081370 } {
            Some(f) => f(gobj),
            None => melee_fighter::tu::ftcliffcommon::ftcliffcommon_80081370(gobj),
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftKoopaAttributes {
    (*fp).dat_attrs as *mut ftKoopaAttributes
}

/// `Fighter_SetDamageCallback(gobj, ftKp_Init_80132B38)` (ft/inlines.h): take_dmg and death2.
#[inline(always)]
unsafe fn set_damage_callback(fp: *mut Fighter) {
    (*fp).take_dmg_cb = Some(cb_ftKp_Init_80132B38);
    (*fp).death2_cb = Some(cb_ftKp_Init_80132B38);
}

/// `ftKp_Init_80132B38` (ftkoopa.c): the empty damage callback, as a Fighter callback entry.
pub unsafe extern "C" fn cb_ftKp_Init_80132B38(_g: *mut HSD_GObj) { melee_core::ct!("cb_ftKp_Init_80132B38");}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// Tail both Enters share after the motion change (`capturekoopa.xC` .. `ftAnim_8006EBA4`).
unsafe fn enter_tail(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    (*fp).mv.co.capturekoopa.xC = 0.0;
    (*fp).mv.kp.specials.x10 = 0;
    ext::efSync_Spawn(0x4DA, gobj, &[(*(*fp).parts).joint as *mut _]);
    (*fp).set_x2219_b0(1);
    set_effect_hitlag_callbacks(fp);
    st::ftAnim_8006EBA4(gobj);
}

pub fn ftKp_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, 0x167, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_damage_callback(fp);
        st::Fighter_ClearCmdVars(&mut *fp);
        (*fp).self_vel.y = 0.0;
        (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as u8;
        enter_tail(gobj, fp);
    }
}

pub fn ftKp_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        st::Fighter_ChangeMotionState(gobj, 0x168, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_damage_callback(fp);
        st::Fighter_ClearCmdVars(&mut *fp);
        fc::clamp_ground_vel(&mut *fp, (*da).x60);
        (*fp).self_vel.y = (*da).x54;
        (*fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*fp) as u8;
        enter_tail(gobj, fp);
        ftKp_SpecialHi_Anim(gobj);
    }
}

pub fn ftKp_SpecialHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialHi_Anim");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.co.walk.middle_anim_frame += 1.0;
        ftKp_SpecialN_80135780(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKp_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirHi_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).mv.co.walk.middle_anim_frame += 1.0;
        ftKp_SpecialN_80135780(gobj);
        if (*fp).cmd_vars[2] == 2 {
            if (*fp).ground_or_air == GA_Air {
                st::ftCommon_8007D60C(&mut *fp);
                st::efLib_DestroyAll(gobj);
                (*fp).set_x2219_b0(0);
                (*fp).pre_hitlag_cb = None;
                (*fp).post_hitlag_cb = None;
                if (*da).x7C == 0.0 {
                    st::ftCo_Fall_Enter(gobj);
                } else {
                    st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*da).x7C);
                }
            }
        } else if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCommon_8007D60C(&mut *fp);
            if (*da).x7C == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*da).x7C);
            }
        }
    }
}

pub fn ftKp_SpecialHi_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKp_SpecialHi_IASA");}
pub fn ftKp_SpecialAirHi_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftKp_SpecialAirHi_IASA");}

pub fn ftKp_SpecialHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialHi_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[0] == 0 {
            fc::calc_ground_accel_accel_to_lstick_x(&mut *fp, 0.0, (*da).x68, (*da).x60);
            fc::set_self_movement_from_grounded_movement(&mut *fp);
        } else {
            st::ft_80084F3C(gobj);
        }
    }
}

pub fn ftKp_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirHi_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[0] == 0 {
            st::ftCommon_Fall(&mut *fp, (*da).x58, (*da).x5C);
            fc::calc_self_accel_drift_simple(&mut *fp, 0.0, (*da).x6C, (*da).x64);
        } else {
            st::ft_80084DB0(gobj);
        }
    }
}

/// Shell tilt to the floor normal (`facing_dir * atan2f(n.x, n.y)`), or flat.
unsafe fn tilt_shell(fp: *mut Fighter) {
    if (*fp).cmd_vars[3] != 0 && (*fp).mv.kp.specials.x10 != 0 {
        let a = melee_runtime::lbtrigf::atan2f((*fp).coll_data.floor.normal.x, (*fp).coll_data.floor.normal.y);
        ext::ftPartSetRotX(fp, 0, (*fp).facing_dir * a);
    } else {
        ext::ftPartSetRotX(fp, 0, 0.0);
    }
}

/// `ftKp_SpecialHi_Coll_inline` (static).
pub fn ftKp_SpecialHi_Coll_inline(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialHi_Coll_inline");
    unsafe { tilt_shell(fpp(gobj)) }
}

pub fn ftKp_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            st::ftCommon_8007D60C(&mut *fp);
            st::Fighter_ChangeMotionState(gobj, 0x168, 0x0C4C5292, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            set_damage_callback(fp);
            fc::clamp_self_vel_x(&mut *fp, (*da).x64);
            set_effect_hitlag_callbacks(fp);
            (*fp).mv.kp.specials.x10 = 0;
        } else {
            (*fp).mv.kp.specials.x10 = 1;
        }
        ftKp_SpecialN_80135780(gobj);
        ftKp_SpecialHi_Coll_inline(gobj);
    }
}

/// `ftKp_SpecialAirHi_Coll_inline` (static inline): airborne -> grounded motion swap.
pub fn ftKp_SpecialAirHi_Coll_inline(gobj: *mut HSD_GObj, fp: *mut Fighter, da: *mut ftKoopaAttributes) {
    melee_core::ct!("ftKp_SpecialAirHi_Coll_inline");
    unsafe {
        st::ftCommon_8007D7FC(&mut *fp);
        // `da->x70 = da->x70;` is a no-op store; the range test below can never hold.
        if (*da).x70 < (*fp).mv.co.walk.middle_anim_frame && (*fp).mv.co.walk.middle_anim_frame < (*da).x70 {
            st::Fighter_ChangeMotionState(gobj, 0x167, 0x0C4C5292, (*fp).cur_anim_frame - (*da).x78, 1.0, 0.0, core::ptr::null_mut());
            set_damage_callback(fp);
        } else {
            st::Fighter_ChangeMotionState(gobj, 0x167, 0x0C4C5292, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
            set_damage_callback(fp);
        }
        fc::clamp_ground_vel(&mut *fp, (*da).x60);
    }
}

/// `ftKp_SpecialAirHi_Coll_inline_2` (static inline).
pub fn ftKp_SpecialAirHi_Coll_inline_2(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirHi_Coll_inline_2");
    unsafe {
        ftKp_SpecialN_80135780(gobj);
        tilt_shell(fpp(gobj));
    }
}

pub fn ftKp_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKp_SpecialAirHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).self_vel.y >= 0.0 {
            if st::ft_80081D0C(gobj) != GA_Ground as i32 {
                ftKp_SpecialAirHi_Coll_inline(gobj, fp, da);
                set_effect_hitlag_callbacks(fp);
                (*fp).mv.kp.specials.x10 = 1;
            } else {
                (*fp).mv.kp.specials.x10 = 0;
            }
            ftKp_SpecialAirHi_Coll_inline_2(gobj);
        } else if st::ft_CheckGroundAndLedge(gobj, 0) != 0 {
            ftKp_SpecialAirHi_Coll_inline(gobj, fp, da);
            (*fp).mv.kp.specials.x10 = 1;
            ftKp_SpecialAirHi_Coll_inline_2(gobj);
            set_effect_hitlag_callbacks(fp);
        } else {
            (*fp).mv.kp.specials.x10 = 0;
            if st::ftCliffCommon_80081298(gobj) != 0 {
                ext::ftCliffCommon_80081370(gobj);
            }
        }
    }
}
