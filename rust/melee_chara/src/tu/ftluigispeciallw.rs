//! src/melee/ft/kinds/ftLuigi/ftluigispeciallw.c: Luigi Cyclone (down B). No fused sites
//! (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested through recording
//! hooks by tests/tu_ftluigispeciallw.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_types::*;

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

/// `Fighter_SetDamageCallback` (ft/inlines.h).
#[inline(always)]
unsafe fn set_damage_callback(gobj: *mut HSD_GObj, cb: HSD_GObjEvent) {
    let fp = fpp(gobj);
    (*fp).take_dmg_cb = cb;
    (*fp).death2_cb = cb;
}

/// `FTLUIGI_SPECIALLW_FLAG`; also the literal 0x0C4C508A of the Cyclone's ground -> air hand-over.
const FTLUIGI_SPECIALLW_FLAG: u32 = (Ft_MF_KeepGfx
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipColAnim
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

/// `ftLg_SpecialLw_UpdateRot` as a Fighter callback entry (`take_dmg_cb` / `death2_cb`).
pub unsafe extern "C" fn cb_ftLg_SpecialLw_UpdateRot(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftLg_SpecialLw_UpdateRot");
    ftLg_SpecialLw_UpdateRot(g);
}

/// Luigi Cyclone Rotation Update (0x801445C4).
pub fn ftLg_SpecialLw_UpdateRot(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialLw_UpdateRot");
    unsafe { ext::ftPartSetRotX(fpp(gobj), 0, 0.0) }
}

/// `ftLuigi_SpecialLw_SetVars` (static inline).
pub fn ftLuigi_SpecialLw_SetVars(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLuigi_SpecialLw_SetVars");
    unsafe {
        let fp = fpp(gobj);
        let luigi_attrs = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).cmd_vars[0] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).mv.lg.SpecialLw.groundVelX = 0.0;
        (*fp).mv.lg.SpecialLw.unk = (*luigi_attrs).x88_LUIGI_CYCLONE_UNK.wrapping_add(1);
        (*fp).mv.lg.SpecialLw.isUnkColl = 0;
    }
}

/// `ftLuigi_SpecialLw_SetGFX` (static inline).
pub fn ftLuigi_SpecialLw_SetGFX(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLuigi_SpecialLw_SetGFX");
    unsafe {
        let fp = fpp(gobj);
        let hsd_obj = (*gobj).hsd_obj;
        ext::efSync_Spawn(0x509, gobj, &[hsd_obj as *mut core::ffi::c_void]);
        (*fp).set_x2219_b0(1);
        set_effect_hitlag_callbacks(fp);
    }
}

/// Luigi's grounded Cyclone Motion State handler (0x801445F0).
pub fn ftLg_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialLw_Enter");
    unsafe {
        let fp = fpp(gobj);
        let luigi_attrs = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fpp(gobj)).cmd_vars[2] = 0;
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialAirLw as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).self_vel.y = (*luigi_attrs).x70_LUIGI_CYCLONE_TAP_MOMENTUM - (*luigi_attrs).x8C_LUIGI_CYCLONE_TAP_Y_VEL_MAX;
        fc::clamp_self_vel_x(&mut *fp, (*luigi_attrs).x78_LUIGI_CYCLONE_MOMENTUM_X_AIR);
        ftLuigi_SpecialLw_SetVars(gobj);
        set_damage_callback(gobj, Some(cb_ftLg_SpecialLw_UpdateRot));
        ftLuigi_SpecialLw_SetGFX(gobj);
        set_effect_hitlag_callbacks(fp);
    }
}

/// Luigi's aerial Cyclone Motion State handler (0x80144708).
pub fn ftLg_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirLw_Enter");
    unsafe {
        let fp = fpp(gobj);
        let luigi_attrs = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fpp(gobj)).cmd_vars[2] = 0;
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialAirLw as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        let cyclone_var: f32 =
            if (*fp).u.lg.x222C_cycloneCharge != 0 { 0.0 } else { (*luigi_attrs).x8C_LUIGI_CYCLONE_TAP_Y_VEL_MAX };
        (*fp).self_vel.y = (*luigi_attrs).x70_LUIGI_CYCLONE_TAP_MOMENTUM - cyclone_var;
        fc::clamp_self_vel_x(&mut *fp, (*luigi_attrs).x78_LUIGI_CYCLONE_MOMENTUM_X_AIR);
        ftLuigi_SpecialLw_SetVars(gobj);
        set_damage_callback(gobj, Some(cb_ftLg_SpecialLw_UpdateRot));
        ftLuigi_SpecialLw_SetGFX(gobj);
        set_effect_hitlag_callbacks(fp);
    }
}

/// Luigi's grounded Cyclone Animation callback.
pub fn ftLg_SpecialLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialLw_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe { set_damage_callback(gobj, None) };
        st::ft_8008A2BC(gobj);
    }
}

/// Luigi's aerial Cyclone Animation callback.
pub fn ftLg_SpecialAirLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirLw_Anim");
    unsafe {
        let fp = fpp(gobj);
        let attrs = (*fp).dat_attrs as *const ftLuigiAttributes;
        if (*fp).cmd_vars[1] != 0 {
            (*fp).cmd_vars[1] = 0;
            (*fp).u.lg.x222C_cycloneCharge = 1;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            set_damage_callback(gobj, None);
            let landing_lag = (*attrs).x94_LUIGI_CYCLONE_LANDING_LAG;
            if landing_lag as f64 == 0.0 {
                st::ftCo_Fall_Enter(gobj);
                return;
            }
            st::ftCo_80096900(gobj, 1, 0, 1, 1.0, landing_lag as f32);
        }
    }
}

/// Luigi's grounded Cyclone IASA callback.
pub fn ftLg_SpecialLw_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialLw_IASA");}

/// Luigi's aerial Cyclone IASA callback.
pub fn ftLg_SpecialAirLw_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftLg_SpecialAirLw_IASA");}

/// `ftLuigi_SpecialLw_GroundToAir` (static inline): the grounded Cyclone is tapped into the air.
pub fn ftLuigi_SpecialLw_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLuigi_SpecialLw_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        let luigi_attrs = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).cmd_vars[2] = 0;
        st::ftCommon_8007D5D4(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 0x166, FTLUIGI_SPECIALLW_FLAG, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        fc::clamp_fall_speed(&mut *fp, (*luigi_attrs).x90_LUIGI_CYCLONE_TAP_GRAVITY);
        fc::clamp_self_vel_x(&mut *fp, (*luigi_attrs).x78_LUIGI_CYCLONE_MOMENTUM_X_AIR);
        set_effect_hitlag_callbacks(fp);
    }
}

/// Luigi's grounded Cyclone Physics callback (0x80144958).
pub fn ftLg_SpecialLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialLw_Phys");
    unsafe {
        let fp = fpp(gobj);
        let attrs = (*fp).dat_attrs as *const ftLuigiAttributes;
        let mut var2 = (*attrs).x74_LUIGI_CYCLONE_MOMENTUM_X_GROUND;
        if (*fp).cmd_vars[0] != 0 {
            (*fp).mv.lg.SpecialLw.groundVelX -= (*attrs).x84_LUIGI_CYCLONE_FRICTION_END;
            var2 += (*fp).mv.lg.SpecialLw.groundVelX;
            if var2 < 0.0 {
                var2 = 0.0;
            }
        }
        fc::calc_ground_accel_accel_to_lstick_x(&mut *fp, 0.0, (*attrs).x7C_LUIGI_CYCLONE_MOMENTUM_X_MUL_GROUND, var2);
        fc::set_self_movement_from_grounded_movement(&mut *fp);
        if (*fp).cmd_vars[2] != 0 && ((*fp).input.pressed_buttons as i32 & HSD_PAD_B) != 0 {
            (*fp).self_vel.y += (*attrs).x8C_LUIGI_CYCLONE_TAP_Y_VEL_MAX;
            ftLuigi_SpecialLw_GroundToAir(gobj);
        }
    }
}

/// Luigi's aerial Cyclone Physics callback (0x80144A74C).
pub fn ftLg_SpecialAirLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirLw_Phys");
    unsafe {
        let fp = fpp(gobj);
        let attrs0 = (*fp).dat_attrs as *const ftLuigiAttributes;
        if (*fp).u.lg.x222C_cycloneCharge == 0
            && (*fp).cmd_vars[2] != 0
            && ((*fp).input.pressed_buttons as i32 & HSD_PAD_B) != 0
        {
            fc::ascend(&mut *fp, (*attrs0).x8C_LUIGI_CYCLONE_TAP_Y_VEL_MAX, (*attrs0).x90_LUIGI_CYCLONE_TAP_GRAVITY);
        }
        fc::fall_basic(&mut *fp);
        let mut spd_x = (*attrs0).x78_LUIGI_CYCLONE_MOMENTUM_X_AIR;
        let attrs1 = (*fp).dat_attrs as *const ftLuigiAttributes;
        if (*fp).cmd_vars[0] != 0 {
            (*fp).mv.lg.SpecialLw.groundVelX -= (*attrs1).x84_LUIGI_CYCLONE_FRICTION_END;
            spd_x += (*fp).mv.lg.SpecialLw.groundVelX;
            if spd_x < 0.0 {
                spd_x = 0.0;
            }
        }
        fc::calc_self_accel_drift_simple_no_friction(&mut *fp, 0.0, (*attrs0).x80_LUIGI_CYCLONE_MOMENTUM_X_MUL_AIR, spd_x);
    }
}

/// `ftLg_SpecialLw_CollisionBox` (static ftCollisionBox).
static mut ftLg_SpecialLw_CollisionBox: ftCollisionBox = ftCollisionBox {
    top: 12.0,
    bottom: 0.0,
    left: Vec2 { x: -6.0, y: 6.0 },
    right: Vec2 { x: 6.0, y: 6.0 },
};

/// `ftLuigi_SpecialLw_UnkAngle` (static inline): tilt the model to the floor normal.
pub fn ftLuigi_SpecialLw_UnkAngle(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLuigi_SpecialLw_UnkAngle");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[3] != 0 && (*fp).mv.lg.SpecialLw.isUnkColl != 0 {
            let ang = (*fp).facing_dir
                * melee_runtime::lbtrigf::atan2f((*fp).coll_data.floor.normal.x, (*fp).coll_data.floor.normal.y);
            ext::ftPartSetRotX(fp, 0, ang);
            return;
        }
        ext::ftPartSetRotX(fp, 0, 0.0);
    }
}

/// Luigi's grounded Cyclone Collision callback.
pub fn ftLg_SpecialLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialLw_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Ground {
            if !ext::ft_80082888(gobj, &mut ftLg_SpecialLw_CollisionBox) {
                ftLuigi_SpecialLw_GroundToAir(gobj);
                (*fp).mv.lg.SpecialLw.isUnkColl = 0;
            } else {
                (*fp).mv.lg.SpecialLw.isUnkColl = 1;
            }
        } else if !ext::ft_800824A0(gobj, &mut ftLg_SpecialLw_CollisionBox) {
            ftLuigi_SpecialLw_GroundToAir(gobj);
            (*fp).mv.lg.SpecialLw.isUnkColl = 0;
        } else {
            (*fp).mv.lg.SpecialLw.isUnkColl = 1;
        }
        ftLuigi_SpecialLw_UnkAngle(gobj);
    }
}

/// `ftLuigi_SpecialAirLw_AirToGround` (static inline): the airborne Cyclone lands.
pub fn ftLuigi_SpecialAirLw_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLuigi_SpecialAirLw_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        let luigi_attrs = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).cmd_vars[2] = 0;
        st::ftCommon_8007D7FC(&mut *fp);
        (*fp).self_vel.y = 0.0;
        (*fp).u.lg.x222C_cycloneCharge = 0;
        st::Fighter_ChangeMotionState(
            gobj,
            ftLg_MS_SpecialLw as i32,
            FTLUIGI_SPECIALLW_FLAG,
            (*fp).cur_anim_frame,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        fc::clamp_ground_vel(&mut *fp, (*luigi_attrs).x74_LUIGI_CYCLONE_MOMENTUM_X_GROUND);
        set_effect_hitlag_callbacks(fp);
    }
}

/// Luigi's aerial Cyclone Collision callback (0x80144CEC).
pub fn ftLg_SpecialAirLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirLw_Coll");
    unsafe {
        let fp = fpp(gobj);
        if ext::ft_800824A0(gobj, &mut ftLg_SpecialLw_CollisionBox) {
            ftLuigi_SpecialAirLw_AirToGround(gobj);
            (*fp).mv.lg.SpecialLw.isUnkColl = 1;
        } else {
            (*fp).mv.lg.SpecialLw.isUnkColl = 0;
        }
        ftLuigi_SpecialLw_UnkAngle(gobj);
    }
}
