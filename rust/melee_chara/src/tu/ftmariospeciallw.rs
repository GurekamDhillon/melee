//! src/melee/ft/kinds/ftMario/ftmariospeciallw.c: Mario Tornado / Dr. Tornado (down B).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested
//! through recording hooks by tests/tu_ftmariospeciallw.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_types::*;

/// Already ported (rounds 3 and 4): re-exported under this TU.
pub use crate::chars::r3::ftMario::ftmariospeciallw::{ftMr_SpecialLw_Anim, unsetCallbacks};
pub use crate::chars::r4::ftMario::ftmariospeciallw::setCmdVar2;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a native port behind them (the port runs when the hook is unset).
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
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing arguments as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    hook_or!(FT_800824A0, ft_800824A0, melee_fighter::tu::ft_081b::ft_800824a0, (gobj: *mut HSD_GObj, ecb: *mut ftCollisionBox) -> bool);
    hook_or!(FT_80082888, ft_80082888, melee_fighter::tu::ft_081b::ft_80082888, (gobj: *mut HSD_GObj, ecb: *mut ftCollisionBox) -> bool);
    // Recording hook; unset, the native `melee_fighter::tu::ftparts` port runs.
    hook_or!(FTPARTSETROTX, ftPartSetRotX, ftpartsetrotx_port, (fp: *mut Fighter, part: i32, v: f32));
    fn ftpartsetrotx_port(fp: *mut Fighter, part: i32, v: f32) {
        melee_fighter::tu::ftparts::ftPartSetRotX(unsafe { &*fp }, part, v)
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// `ftCommon_AirToGroundStateChange` / `GroundToAir...` flags (`transition_flags`).
const transition_flags: u32 = (Ft_MF_KeepGfx
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipColAnim
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

/// `updateRot` (static) as a Fighter callback entry (`take_dmg_cb` / `death2_cb`).
pub unsafe extern "C" fn cb_updateRot(g: *mut HSD_GObj) {
    updateRot(g);
}

/// `updateRot` (static): reset the Tornado spin on being hit.
pub fn updateRot(gobj: *mut HSD_GObj) {
    unsafe { ext::ftPartSetRotX(fpp(gobj), 0, 0.0) }
}

/// `setGfx` (static).
pub fn setGfx(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let hsd_obj_ptr = (*gobj).hsd_obj;
        ext::efSync_Spawn(0x47C, gobj, &[hsd_obj_ptr as *mut core::ffi::c_void]);
        (*fp).set_x2219_b0(1);
        set_effect_hitlag_callbacks(fp);
    }
}

/// `setCallbacks` (static).
pub fn setCallbacks(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).take_dmg_cb = Some(cb_updateRot);
        (*fp).death2_cb = Some(cb_updateRot);
    }
}

/// `doStartMotion` (static).
pub fn doStartMotion(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        (*fp).cmd_vars[0] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).mv.mr.SpecialLw.groundVelX = 0.0;
        (*fp).mv.mr.SpecialLw.unk = (*sa).speciallw.unk0.wrapping_add(1);
        (*fp).mv.mr.SpecialLw.isUnkColl = 0;
        setCallbacks(gobj);
        setGfx(gobj);
    }
}

pub fn ftMr_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        setCmdVar2(gobj);
        st::Fighter_ChangeMotionState(gobj, ftMr_MS_SpecialAirLw as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).self_vel.y = (*sa).speciallw.vel_y - (*sa).speciallw.tap_y_vel_max;
        fc::clamp_self_vel_x(&mut *fp, (*sa).speciallw.air_momentum_x);
        doStartMotion(gobj);
        set_effect_hitlag_callbacks(fp);
    }
}

pub fn ftMr_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        setCmdVar2(gobj);
        st::Fighter_ChangeMotionState(gobj, ftMr_MS_SpecialAirLw as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        let sub_val: f32 = if (*fp).u.mr.x2234_tornadoCharge != 0 { 0.0 } else { (*sa).speciallw.tap_y_vel_max };
        (*fp).self_vel.y = (*sa).speciallw.vel_y - sub_val;
        fc::clamp_self_vel_x(&mut *fp, (*sa).speciallw.air_momentum_x);
        doStartMotion(gobj);
        set_effect_hitlag_callbacks(fp);
    }
}

pub fn ftMr_SpecialAirLw_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        if (*fp).cmd_vars[1] != 0 {
            (*fp).cmd_vars[1] = 0;
            (*fp).u.mr.x2234_tornadoCharge = 1;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            unsetCallbacks(gobj);
            if 0.0f64 == (*sa).speciallw.landing_lag as f64 {
                st::ftCo_Fall_Enter(gobj);
                return;
            }
            st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*sa).speciallw.landing_lag as f32);
        }
    }
}

pub fn ftMr_SpecialLw_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftMr_SpecialAirLw_IASA(_gobj: *mut HSD_GObj) {}

/// `doPhys` (static): the grounded Tornado is tapped into the air.
pub fn doPhys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        (*fp).cmd_vars[2] = 0;
        st::ftCommon_8007D5D4(&mut *fp);
        st::Fighter_ChangeMotionState(
            gobj,
            ftMr_MS_SpecialAirLw as i32,
            transition_flags,
            (*fp).cur_anim_frame,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        fc::clamp_fall_speed(&mut *fp, (*sa).speciallw.tap_grav);
        fc::clamp_self_vel_x(&mut *fp, (*sa).speciallw.air_momentum_x);
        set_effect_hitlag_callbacks(fp);
    }
}

pub fn ftMr_SpecialLw_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp0 = fpp(gobj);
        let sa = (*fp0).dat_attrs as *const ftMario_DatAttrs;
        let mut vel_y = (*sa).speciallw.momentum_x;
        if (*fp0).cmd_vars[0] != 0 {
            (*fp0).mv.mr.SpecialLw.groundVelX = (*fp0).mv.mr.SpecialLw.groundVelX - (*sa).speciallw.friction_end;
            vel_y += (*fp0).mv.mr.SpecialLw.groundVelX;
            if vel_y < 0.0 {
                vel_y = 0.0;
            }
        }
        fc::calc_ground_accel_accel_to_lstick_x(&mut *fp0, 0.0, (*sa).speciallw.momentum_x_mul, vel_y);
        fc::set_self_movement_from_grounded_movement(&mut *fp0);
        if (*fp0).cmd_vars[2] != 0 && ((*fp0).input.pressed_buttons as i32 & HSD_PAD_B) != 0 {
            vel_y = (*fp0).self_vel.y;
            (*fp0).self_vel.y = vel_y + (*sa).speciallw.tap_y_vel_max;
            doPhys(gobj);
        }
    }
}

pub fn ftMr_SpecialAirLw_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        if (*fp).u.mr.x2234_tornadoCharge == 0
            && (*fp).cmd_vars[2] != 0
            && ((*fp).input.pressed_buttons as i32 & HSD_PAD_B) != 0
        {
            fc::ascend(&mut *fp, (*sa).speciallw.tap_y_vel_max, (*sa).speciallw.tap_grav);
        }
        fc::fall_basic(&mut *fp);
        let mut flt_var = (*sa).speciallw.air_momentum_x;
        let sa_2 = (*fp).dat_attrs as *const ftMario_DatAttrs;
        if (*fp).cmd_vars[0] != 0 {
            (*fp).mv.mr.SpecialLw.groundVelX = (*fp).mv.mr.SpecialLw.groundVelX - (*sa_2).speciallw.friction_end;
            flt_var += (*fp).mv.mr.SpecialLw.groundVelX;
            if flt_var < 0.0 {
                flt_var = 0.0;
            }
        }
        fc::calc_self_accel_drift_simple_no_friction(&mut *fp, 0.0, (*sa).speciallw.air_momentum_x_mul, flt_var);
    }
}

/// `doColl` (static): tilt the model to the floor normal while the Tornado is on it.
pub fn doColl(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[3] != 0 && (*fp).mv.mr.SpecialLw.isUnkColl != 0 {
            let ang = (*fp).facing_dir
                * melee_runtime::lbtrigf::atan2f((*fp).coll_data.floor.normal.x, (*fp).coll_data.floor.normal.y);
            ext::ftPartSetRotX(fp, 0, ang);
        } else {
            ext::ftPartSetRotX(fp, 0, 0.0);
        }
    }
}

/// `coll_box` (static ftCollisionBox).
static mut coll_box: ftCollisionBox = ftCollisionBox {
    top: 12.0,
    bottom: 0.0,
    left: Vec2 { x: -6.0, y: 6.0 },
    right: Vec2 { x: 6.0, y: 6.0 },
};

pub fn ftMr_SpecialLw_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Ground {
            if !ext::ft_80082888(gobj, &mut coll_box) {
                doPhys(gobj);
                (*fp).mv.mr.SpecialLw.isUnkColl = 0;
            } else {
                (*fp).mv.mr.SpecialLw.isUnkColl = 1;
            }
        } else if !ext::ft_800824A0(gobj, &mut coll_box) {
            doPhys(gobj);
            (*fp).mv.mr.SpecialLw.isUnkColl = 0;
        } else {
            (*fp).mv.mr.SpecialLw.isUnkColl = 1;
        }
        doColl(gobj);
    }
}

/// `doAirCollIfUnk` (static): the airborne Tornado lands.
pub fn doAirCollIfUnk(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        (*fp).cmd_vars[2] = 0;
        st::ftCommon_8007D7FC(&mut *fp);
        (*fp).self_vel.y = 0.0;
        (*fp).u.mr.x2234_tornadoCharge = 0;
        st::Fighter_ChangeMotionState(
            gobj,
            ftMr_MS_SpecialLw as i32,
            transition_flags,
            (*fp).cur_anim_frame,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        fc::clamp_ground_vel(&mut *fp, (*sa).speciallw.momentum_x);
        set_effect_hitlag_callbacks(fp);
    }
}

pub fn ftMr_SpecialAirLw_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if ext::ft_800824A0(gobj, &mut coll_box) {
            doAirCollIfUnk(gobj);
            (*fp).mv.mr.SpecialLw.isUnkColl = 1;
        } else {
            (*fp).mv.mr.SpecialLw.isUnkColl = 0;
        }
        doColl(gobj);
    }
}
