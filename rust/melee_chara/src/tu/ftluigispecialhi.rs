//! src/melee/ft/kinds/ftLuigi/ftluigispecialhi.c: Super Jump Punch (up B). No fused sites
//! (tools/rust-port/fma_sites.py: 0 fmuladd in every function). The stick-angle update of the
//! IASA is checked against the C by tests/tu_ftluigispecialhi.rs; the rest is smoke-tested
//! through recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_types::*;

/// Already ported (round 7): re-exported under this TU.


/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a native port behind them (the port runs when the hook is unset).
pub mod ext {
    use melee_types::*;
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
    hook_or!(FT_80083B68, ft_80083B68, melee_fighter::tu::ft_081b::ft_80083b68, (gobj: *mut HSD_GObj));
    hook_or!(FT_800831CC, ft_800831CC, melee_fighter::tu::ft_081b::ft_800831cc,
        (gobj: *mut HSD_GObj, coll: fn(*mut HSD_GObj, isize) -> bool, cb: fn(*mut HSD_GObj)));
    // Recording hook; unset, the native `melee_fighter::tu::ftparts` port runs.
    hook_or!(FTPARTSETROTY, ftPartSetRotY, ftpartsetroty_port, (fp: *mut Fighter, part: i32, v: f32));
    fn ftpartsetroty_port(fp: *mut Fighter, part: i32, v: f32) {
        melee_fighter::tu::ftparts::ftPartSetRotY(unsafe { &*fp }, part, v)
    }
    /// `ftCo_80096CC8` (ftCo_FallSpecial.c) with the Rust ABI the collision helper takes.
    pub fn ftCo_80096CC8(gobj: *mut HSD_GObj, line_id: isize) -> bool {
        unsafe { melee_fighter::tu::ftCo_FallSpecial::ftCo_80096CC8(gobj, line_id) }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `stickGetDir` (ft/inlines.h).
#[inline(always)]
fn stick_get_dir(x1: f32, x2: f32) -> f32 {
    if x1 < x2 {
        -x1
    } else {
        x1
    }
}

/// Luigi's aerial Super Jump Punch Motion State handler.
pub fn ftLg_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        let attrs = (*fp).dat_attrs as *const ftLuigiAttributes;
        (*fp).cmd_vars[0] = 0;
        (*fp).x2210.throw_flags = 0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x *= (*attrs).x64_LUIGI_SUPERJUMP_VEL_X;
        st::Fighter_ChangeMotionState(gobj, ftLg_MS_SpecialAirHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

/// Luigi's grounded Super Jump Punch Animation callback.
pub fn ftLg_SpecialHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialHi_Anim");
    unsafe {
        let attrs = (*fpp(gobj)).dat_attrs as *const ftLuigiAttributes;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_80096900(
                gobj,
                0,
                1,
                0,
                (*attrs).x50_LUIGI_SUPERJUMP_FREEFALL_MOBILITY,
                (*attrs).x54_LUIGI_SUPERJUMP_LANDING_LAG,
            );
        }
    }
}

/// Luigi's aerial Super Jump Punch Animation callback.
pub fn ftLg_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirHi_Anim");
    ftLg_SpecialHi_Anim(gobj)
}

/// First half of the IASA: the stick-driven momentum angle (`fp->lstick_angle`).
pub fn special_hi_iasa_lstick(fp: &mut Fighter) {
    melee_core::ct!("special_hi_iasa_lstick");
    let luigi_attrs: &ftLuigiAttributes = unsafe { &*(fp.dat_attrs as *const ftLuigiAttributes) };
    let stick_x = fp.input.lstick[0].x;
    let stick_range = stick_get_dir(fp.input.lstick[0].x, 0.0);
    if fp.cmd_vars[0] == 0 {
        let super_jump_stick_range = luigi_attrs.x5C_LUIGI_SUPERJUMP_MOMENTUM_STICK_RANGE;
        if stick_range > super_jump_stick_range {
            let temp_range: f32 = (luigi_attrs.x60_LUIGI_SUPERJUMP_ANGLE_DIFF as f64
                * ((stick_range - super_jump_stick_range) as f64 / (1.0f64 - super_jump_stick_range as f64)))
                as f32;
            let control: f32 =
                if stick_x > 0.0 { -(0.01745329238474369f32 * temp_range) } else { 0.01745329238474369f32 * temp_range };
            let control_2 = fp.lstick_angle;
            let mut stick_angle = control_2;
            if stick_angle < 0.0 {
                stick_angle = -stick_angle;
            }
            let stick_range_2 = if control < 0.0 { -control } else { control };
            if stick_range_2 > stick_angle {
                fp.lstick_angle = control;
            }
        }
    }
}

/// Luigi's grounded Super Jump Punch IASA callback.
pub fn ftLg_SpecialHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialHi_IASA");
    unsafe {
        let fp = fpp(gobj);
        let luigi_attrs = (*fp).dat_attrs as *const ftLuigiAttributes;
        special_hi_iasa_lstick(&mut *fp);
        // ftCheckThrowB3
        if (*fp).x2210.x0.throw_flags_b3() != 0 {
            (*fp).x2210.x0.set_throw_flags_b3(0);
            let mut control = (*fp).input.lstick[0].x;
            if control < 0.0 {
                control = -control;
            }
            if control > (*luigi_attrs).x58_LUIGI_SUPERJUMP_REVERSE_STICK_RANGE {
                st::ftCommon_UpdateFacing(&mut *fp);
                ext::ftPartSetRotY(fp, 0, (core::f64::consts::FRAC_PI_2 * (*fp).facing_dir as f64) as f32);
            }
        }
    }
}

/// Luigi's aerial Super Jump Punch IASA callback.
pub fn ftLg_SpecialAirHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirHi_IASA");
    ftLg_SpecialHi_IASA(gobj)
}

/// Luigi's grounded Super Jump Punch Physics callback.
pub fn ftLg_SpecialHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialHi_Phys");
    unsafe {
        if (*fpp(gobj)).ground_or_air as i32 == GA_Air as i32 {
            st::ft_80085154(gobj);
            return;
        }
        st::ft_80084FA8(gobj);
    }
}

/// Luigi's aerial Super Jump Punch Physics callback.
pub fn ftLg_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirHi_Phys");
    unsafe {
        let fp = fpp(gobj);
        let luigi_attrs = (*fp).dat_attrs as *const ftLuigiAttributes;
        if (*fp).cmd_vars[0] != 0 {
            st::ft_80085154(gobj);
            (*fp).self_vel.x *= (*luigi_attrs).x6C_LUIGI_SUPERJUMP_VEL_Y;
            (*fp).self_vel.y *= (*luigi_attrs).x6C_LUIGI_SUPERJUMP_VEL_Y;
            (*fp).self_vel.z *= (*luigi_attrs).x6C_LUIGI_SUPERJUMP_VEL_Y;
            return;
        }
        let terminal = (*fp).co_attrs.terminal_velocity;
        fc::fall(&mut *fp, (*luigi_attrs).x68_LUIGI_SUPERJUMP_GRAVITY_START, terminal);
        fc::calc_self_accel_deaccel_quick_air(&mut *fp);
    }
}

/// Luigi's Super Jump Punch Landing check.
pub fn ftLg_SpecialHi_CheckLanding(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialHi_CheckLanding");
    unsafe {
        st::ftCo_LandingFallSpecial_Enter(
            gobj,
            0,
            (*((*fpp(gobj)).dat_attrs as *const ftLuigiAttributes)).x54_LUIGI_SUPERJUMP_LANDING_LAG,
        );
    }
}

/// Luigi's grounded Super Jump Punch Collision callback.
pub fn ftLg_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air as i32 == GA_Air as i32 {
            if (*fp).cmd_vars[0] == 0 || (*fp).self_vel.y >= 0.0 {
                ext::ft_80083B68(gobj);
                return;
            }
            ext::ft_800831CC(gobj, ext::ftCo_80096CC8, ftLg_SpecialHi_CheckLanding);
            return;
        }
        st::ft_80084104(gobj);
    }
}

/// Luigi's aerial Super Jump Punch Collision callback.
pub fn ftLg_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialAirHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air as i32 == GA_Air as i32 {
            if (*fp).cmd_vars[0] == 0 || (*fp).self_vel.y >= 0.0 {
                ext::ft_80083B68(gobj);
                return;
            }
            ext::ft_800831CC(gobj, ext::ftCo_80096CC8, ftLg_SpecialHi_CheckLanding);
            return;
        }
        st::ft_80084104(gobj);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftLg_SpecialHi_Enter` (src/melee/ft/kinds/ftLuigi/ftluigispecialhi.c).
pub fn ftLg_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftLg_SpecialHi_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).x2210.throw_flags = 0_u32;
        st::Fighter_ChangeMotionState(gobj, 355 /* ftLg_MS_SpecialHi */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}
