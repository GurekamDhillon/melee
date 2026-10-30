//! src/melee/ft/kinds/ftMario/ftmariospecialhi.c: Super Jump Punch / Dr. Mario's Super Jump Punch
//! (up B). No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). The
//! stick-angle update of the IASA is checked against the C by tests/tu_ftmariospecialhi.rs; the
//! rest is smoke-tested through recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_types::*;

/// Already ported (rounds 7 and 13): re-exported under this TU.
pub use crate::chars::r13::ftMario::ftmariospecialhi::ftMr_SpecialHi_Phys;
pub use crate::chars::r5::ftMario::ftmariospecialhi::ftMr_SpecialHi_CheckLanding;
pub use crate::chars::r7::ftMario::ftmariospecialhi::ftMr_SpecialHi_Enter;

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
        (gobj: *mut HSD_GObj, coll: fn(*mut HSD_GObj, i32) -> bool, cb: fn(*mut HSD_GObj)));
    hook_or!(FTPARTSETROTY, ftPartSetRotY, ftpartsetroty_port, (fp: *mut Fighter, part: i32, v: f32));
    fn ftpartsetroty_port(fp: *mut Fighter, part: i32, v: f32) {
        melee_fighter::tu::ftparts::ftPartSetRotY(unsafe { &*fp }, part, v)
    }
    /// `ftCo_80096CC8` (ftCo_FallSpecial.c) with the Rust ABI the collision helper takes.
    pub fn ftCo_80096CC8(gobj: *mut HSD_GObj, line_id: i32) -> bool {
        unsafe { melee_fighter::tu::ftCo_FallSpecial::ftCo_80096CC8(gobj, line_id) }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// The C file's local `abs(x)` macro: `(x < 0 ? -x : x)`.
#[inline(always)]
fn abs_m(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

pub fn ftMr_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialAirHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        (*fp).cmd_vars[0] = 0;
        (*fp).anon_2210.throw_flags = 0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = (*fp).self_vel.x * (*sa).specialhi.vel_x;
        st::Fighter_ChangeMotionState(gobj, ftMr_MS_SpecialAirHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `ftMr_SpecialHi_Anim` (0x800E1B24): the end of the move enters FallSpecial.
pub fn ftMr_SpecialHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialHi_Anim");
    unsafe {
        let sa = (*fpp(gobj)).dat_attrs as *const ftMario_DatAttrs;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_80096900(gobj, 0, 1, 0, (*sa).specialhi.freefall_mobility, (*sa).specialhi.landing_lag);
        }
    }
}

/// `ftMr_SpecialAirHi_Anim` (0x800E1B84).
pub fn ftMr_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialAirHi_Anim");
    ftMr_SpecialHi_Anim(gobj)
}

/// First half of `ftMr_SpecialHi_IASA`: the stick-driven momentum angle (`fp->lstick_angle`).
pub fn special_hi_iasa_lstick(fp: &mut Fighter) {
    melee_core::ct!("special_hi_iasa_lstick");
    let sa: &ftMario_DatAttrs = unsafe { &*(fp.dat_attrs as *const ftMario_DatAttrs) };
    let lstick_x = abs_m(fp.input.lstick[0].x);
    let range = sa.specialhi.momentum_stick_range;
    if fp.cmd_vars[0] == 0 && lstick_x > range {
        // `angle_diff * ((lstick_x - range) / (1.0 - range))`: the `1.0` is a double.
        let deg: f32 = (sa.specialhi.angle_diff as f64 * ((lstick_x - range) as f64 / (1.0f64 - range as f64))) as f32;
        let rad: f32 = if fp.input.lstick[0].x > 0.0 { -(deg * 0.01745329252f32) } else { deg * 0.01745329252f32 };
        if abs_m(rad) > abs_m(fp.lstick_angle) {
            fp.lstick_angle = rad;
        }
    }
}

pub fn ftMr_SpecialHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialHi_IASA");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftMario_DatAttrs;
        special_hi_iasa_lstick(&mut *fp);
        // ftCheckThrowB3
        if (*fp).anon_2210.anon_0.throw_flags_b3() != 0 {
            (*fp).anon_2210.anon_0.set_throw_flags_b3(0);
            if abs_m((*fp).input.lstick[0].x) > (*sa).specialhi.reverse_stick_range {
                st::ftCommon_UpdateFacing(&mut *fp);
                ext::ftPartSetRotY(fp, 0, (core::f64::consts::FRAC_PI_2 * (*fp).facing_dir as f64) as f32);
            }
        }
    }
}

pub fn ftMr_SpecialAirHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialAirHi_IASA");
    ftMr_SpecialHi_IASA(gobj)
}

pub fn ftMr_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialAirHi_Phys");
    unsafe {
        let fp = &mut *fpp(gobj);
        let sa = &*(fp.dat_attrs as *const ftMario_DatAttrs);
        if fp.cmd_vars[0] != 0 {
            st::ft_80085154(gobj);
            let fp = &mut *fpp(gobj);
            fp.self_vel.x *= sa.specialhi.vel_mul;
            fp.self_vel.y *= sa.specialhi.vel_mul;
            fp.self_vel.z *= sa.specialhi.vel_mul;
        } else {
            let terminal = fp.co_attrs.terminal_velocity;
            fc::fall(fp, sa.specialhi.grav, terminal);
            fc::calc_self_accel_deaccel_quick_air(fp);
        }
    }
}

pub fn ftMr_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Air {
            if (*fp).cmd_vars[0] == 0 || (*fp).self_vel.y >= 0.0 {
                ext::ft_80083B68(gobj);
            } else {
                ext::ft_800831CC(gobj, ext::ftCo_80096CC8, ftMr_SpecialHi_CheckLanding_rs);
            }
        } else {
            st::ft_80084104(gobj);
        }
    }
}

/// `ftMr_SpecialHi_CheckLanding` with the Rust ABI the landing helper takes.
fn ftMr_SpecialHi_CheckLanding_rs(gobj: *mut HSD_GObj) {
    ftMr_SpecialHi_CheckLanding(gobj)
}

pub fn ftMr_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialAirHi_Coll");
    ftMr_SpecialHi_Coll(gobj)
}
