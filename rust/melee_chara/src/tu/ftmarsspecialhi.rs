//! src/melee/ft/kinds/ftMars/ftmarsspecialhi.c: Dolphin Slash (up B).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function; the IASA angle is
//! f64 mul/div/sub with no add, so nothing fuses). Smoke-tested by tests/tu_ftmarsspecialhi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_types::*;

/// Already ported (rounds 5/7): re-exported under this TU.
pub use crate::chars::r5::ftMars::ftmarsspecialhi::ftMs_SpecialHi_80138884;
pub use crate::chars::r7::ftMars::ftmarsspecialhi::{ftMs_SpecialAirHi_Anim, ftMs_SpecialHi_Anim, ftMs_SpecialHi_Enter};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it falls back to the
/// native `melee_fighter` port where there is one.
pub mod ext {
    use melee_types::*;
    type G = *mut HSD_GObj;
    pub static mut FT_80083B68: Option<fn(G)> = None;
    pub fn ft_80083B68(g: G) {
        match unsafe { FT_80083B68 } {
            Some(f) => f(g),
            None => melee_fighter::tu::ft_081b::ft_80083b68(g),
        }
    }
    /// `ft_800831CC(gobj, &ftCo_80096CC8, cb)`: the line-id callback is always `ftCo_80096CC8`.
    pub static mut FT_800831CC: Option<fn(G, fn(G, i32) -> bool, fn(G))> = None;
    pub fn ft_800831CC(g: G, line_cb: fn(G, i32) -> bool, cb: fn(G)) {
        match unsafe { FT_800831CC } {
            Some(f) => f(g, line_cb, cb),
            None => melee_fighter::tu::ft_081b::ft_800831cc(g, line_cb, cb),
        }
    }
    pub fn ftCo_80096CC8(g: G, line_id: i32) -> bool {
        melee_fighter::tu::ftCo_FallSpecial::ftCo_80096CC8(g, line_id)
    }
    pub static mut FTPARTSETROTY: Option<fn(*mut Fighter, i32, f32)> = None;
    pub fn ftPartSetRotY(fp: *mut Fighter, part: i32, v: f32) {
        match unsafe { FTPARTSETROTY } {
            Some(f) => f(fp, part, v),
            None => melee_fighter::tu::ftparts::ftPartSetRotY(unsafe { &*fp }, part, v),
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const MarsAttributes {
    (*fp).dat_attrs as *const MarsAttributes
}

/// C `abs(x)` macro of this file: `x < 0.0f ? -x : x`.
#[inline(always)]
fn cabs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

pub fn ftMs_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).cmd_vars[2] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).anon_2210.throw_flags = 0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x *= (*da).x3C;
        st::Fighter_ChangeMotionState(gobj, 0x170, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

/// Shared body of both IASA callbacks (the two C bodies are identical).
unsafe fn iasa(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let da = da(fp);
    let abs_lstick_x = cabs((*fp).input.lstick[0].x);
    if (*fp).cmd_vars[0] == 0 && abs_lstick_x > (*da).x34 {
        // `da->x38 * ((abs_lstick_x - da->x34) / (1.0 - da->x34))`: the literal 1.0 is a double,
        // so the quotient and product are f64 and round to f32 once, at the assignment.
        let q = ((abs_lstick_x - (*da).x34) as f64) / (1.0f64 - (*da).x34 as f64);
        let mut temp_f1: f32 = ((*da).x38 as f64 * q) as f32;
        let deg_to_rad: f32 = 3.14159265359f32 / 180.0f32;
        temp_f1 = if (*fp).input.lstick[0].x > 0.0 { -(deg_to_rad * temp_f1) } else { deg_to_rad * temp_f1 };
        if cabs(temp_f1) > cabs((*fp).lstick_angle) {
            (*fp).lstick_angle = temp_f1;
        }
    }
    // ftCheckThrowB3: test and clear
    if (*fp).anon_2210.anon_0.throw_flags_b3() != 0 {
        (*fp).anon_2210.anon_0.set_throw_flags_b3(0);
        if cabs((*fp).input.lstick[0].x) > (*da).x30 {
            st::ftCommon_UpdateFacing(&mut *fp);
            ext::ftPartSetRotY(fp, 0, (1.5707963267948966f64 * (*fp).facing_dir as f64) as f32);
        }
    }
}

pub fn ftMs_SpecialHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialHi_IASA");
    unsafe { iasa(gobj) }
}

pub fn ftMs_SpecialAirHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirHi_IASA");
    unsafe { iasa(gobj) }
}

pub fn ftMs_SpecialHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialHi_Phys");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Air {
            if (*fp).cmd_vars[2] == 0 {
                st::ft_80085154(gobj);
                let num: i32 = if (*fp).self_vel.x < 0.0 { -1 } else { 1 };
                if (*fp).facing_dir != num as f32 {
                    (*fp).self_vel.x *= -1.0;
                }
                if (*fp).x6A4_transNOffset.y < 0.0 {
                    (*fp).cmd_vars[2] = 1;
                }
            } else {
                let da = da(fp);
                fc::fall(&mut *fp, (*da).x44, (*da).x48);
                let stick_mul = (*fp).co_attrs.air_drift_stick_mul * (*da).x28;
                let drift_max = (*fp).co_attrs.air_drift_max * (*da).x28;
                fc::calc_self_accel_drift_simple(&mut *fp, 0.0, stick_mul, drift_max);
            }
        } else {
            st::ft_80084FA8(gobj);
        }
    }
}

pub fn ftMs_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirHi_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[0] != 0 {
            if (*fp).cmd_vars[2] == 0 {
                st::ft_80085154(gobj);
                (*fp).self_vel.x *= (*da).x40;
                (*fp).self_vel.y *= (*da).x40;
                (*fp).self_vel.z *= (*da).x40;
                let num: i32 = if (*fp).self_vel.x < 0.0 { -1 } else { 1 };
                if (*fp).facing_dir != num as f32 {
                    (*fp).self_vel.x *= -1.0;
                }
                if (*fp).x6A4_transNOffset.y < 0.0 {
                    (*fp).cmd_vars[2] = 1;
                }
            } else {
                fc::fall(&mut *fp, (*da).x44, (*da).x48);
                let stick_mul = (*fp).co_attrs.air_drift_stick_mul * (*da).x28;
                let drift_max = (*fp).co_attrs.air_drift_max * (*da).x28;
                fc::calc_self_accel_drift_simple(&mut *fp, 0.0, stick_mul, drift_max);
            }
        } else {
            fc::fall(&mut *fp, (*fp).co_attrs.gravity, (*fp).co_attrs.terminal_velocity);
            fc::calc_self_accel_deaccel_quick_air(&mut *fp);
        }
    }
}

/// `ftMs_SpecialHi_Coll` / `ftMs_SpecialAirHi_Coll` share this body.
unsafe fn coll(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if (*fp).ground_or_air == GA_Air {
        if (*fp).cmd_vars[0] == 0 || (*fp).self_vel.y >= 0.0 {
            ext::ft_80083B68(gobj);
        } else if (*fp).cmd_vars[1] == 0 {
            (*fp).cmd_vars[1] = 1;
            ext::ft_80083B68(gobj);
        } else {
            ext::ft_800831CC(gobj, ext::ftCo_80096CC8, ftMs_SpecialHi_80138884);
        }
    } else {
        st::ft_80084104(gobj);
    }
}

pub fn ftMs_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialHi_Coll");
    unsafe { coll(gobj) }
}

pub fn ftMs_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMs_SpecialAirHi_Coll");
    unsafe { coll(gobj) }
}
