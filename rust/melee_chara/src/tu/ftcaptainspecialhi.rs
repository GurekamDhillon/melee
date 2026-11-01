//! src/melee/ft/kinds/ftCaptain/ftcaptainspecialhi.c: Falcon Dive / Dark Dive (up B).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested
//! through recording hooks by tests/tu_ftcaptainspecialhi.rs (Phys velocity math included).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::{caps_max_jumps, da as da_g};
#[allow(unused_imports)]
use melee_types::{ftCaptain_DatAttrs, Fighter};
#[allow(unused_imports)]
use melee_types::{HSD_GObj};
use crate::stubs as st;
use melee_fighter::ftcommon as fc;
use melee_fighter::ftcommon2 as fc2;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 5/6/7 and the pure helper): re-exported under this TU.





/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
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
    pub static mut FTCO_8009CA0C: Option<fn(*mut HSD_GObj, *mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_8009CA0C(gobj: *mut HSD_GObj, vic: *mut HSD_GObj)  {
                match unsafe { FTCO_8009CA0C } {
                    Some(f) => f(gobj, vic),
                    None => { unsafe { melee_fighter::tu::ftCo_CaptureCaptain::ftCo_8009CA0C(gobj as _, vic as _); } return; },
                }
            }
    pub static mut FT_80083B68: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ft_80083B68(gobj: *mut HSD_GObj)  {
                match unsafe { FT_80083B68 } {
                    Some(f) => f(gobj),
                    None => { unsafe { melee_fighter::tu::ft_081b::ft_80083b68(gobj as _); } return; },
                }
            }
    pub static mut FTCLIFFCOMMON_80081370: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCliffCommon_80081370(gobj: *mut HSD_GObj)  {
                match unsafe { FTCLIFFCOMMON_80081370 } {
                    Some(f) => f(gobj),
                    None => { unsafe { melee_fighter::tu::ftcliffcommon::ftcliffcommon_80081370(gobj as _); } return; },
                }
            }
    pub static mut FTCO_800DB368: Option<fn(*mut Fighter, *mut Fighter)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800DB368(vic_fp: *mut Fighter, fp: *mut Fighter) {
        match unsafe { FTCO_800DB368 } {
            Some(f) => f(vic_fp, fp),
            None => { unsafe { melee_fighter::tu::ftCo_CapturePulled::ftCo_800DB368(&mut *(vic_fp as *mut _), &mut *(fp as *mut _)); } },
        }
    }
    pub static mut FTCO_800DE2A8: Option<fn(*mut HSD_GObj, *mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_800DE2A8(gobj: *mut HSD_GObj, vic: *mut HSD_GObj)  {
                match unsafe { FTCO_800DE2A8 } {
                    Some(f) => f(gobj, vic),
                    None => { unsafe { melee_fighter::tu::ftCo_Throw::ftCo_800DE2A8(gobj as _, vic as _); } return; },
                }
            }
    pub static mut FTCO_800DE7C0: Option<fn(*mut HSD_GObj, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800DE7C0(gobj: *mut HSD_GObj, a: i32, b: i32) {
        match unsafe { FTCO_800DE7C0 } {
            Some(f) => f(gobj, a, b),
            None => { unsafe { melee_fighter::tu::ftCo_Thrown::ftCo_800DE7C0(gobj as _, a as _, (b != 0)); } },
        }
    }
    /// Recording hook; unset, the native `melee_fighter::tu::ftparts` port runs.
    pub static mut FTPARTSETROTY: Option<fn(*mut Fighter, i32, f32)> = None;
    pub fn ftPartSetRotY(fp: *mut Fighter, part: i32, v: f32) {
        match unsafe { FTPARTSETROTY } {
            Some(f) => f(fp, part, v),
            None => melee_fighter::tu::ftparts::ftPartSetRotY(unsafe { &*fp }, part, v),
        }
    }

    /// `ftCo_8009CA0C` as a grabbed (interaction) callback entry.
    pub unsafe extern "C" fn cb_ftCo_8009CA0C(g: *mut HSD_GObj, v: *mut HSD_GObj) {
        ftCo_8009CA0C(g, v);
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

pub unsafe extern "C" fn cb_ftCa_SpecialLw_800E49FC(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftCa_SpecialLw_800E49FC");
    ftCa_SpecialLw_800E49FC(&mut *fpp(g));
}
pub unsafe extern "C" fn cb_ftCa_SpecialLw_800E5128(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftCa_SpecialLw_800E5128");
    ftCa_SpecialLw_800E5128(g);
}
pub unsafe extern "C" fn cb_ftCa_SpecialLw_800E550C(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftCa_SpecialLw_800E550C");
    ftCa_SpecialLw_800E550C(g);
}

unsafe fn enter(gobj: *mut HSD_GObj, msid: i32) {
    let fp = fpp(gobj);
    (*fp).x21EC = Some(cb_ftCa_SpecialLw_800E49FC);
    st::Fighter_ChangeMotionState(gobj, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    fc2::ftcommon_8007e2d0(&mut *fp, 2, Some(cb_ftCa_SpecialLw_800E5128), None, Some(ext::cb_ftCo_8009CA0C));
    st::ftAnim_8006EBA4(gobj);
}

pub fn ftCa_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHi_Enter");
    unsafe { enter(gobj, ftCa_MS_SpecialHi as i32) }
}

/// Shared body of the IASA callbacks once `cmd_vars[0]` is set (`doAirIASA`).
pub fn doAirIASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("doAirIASA");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).cmd_vars[0] = 0;
        (*fp).mv.ca.specialhi.set_x2_b1(1);
        let mut lstick_x = (*fp).input.lstick[0].x;
        if lstick_x < 0.0 {
            lstick_x = -lstick_x;
        }
        if lstick_x > (*da).specialhi_input_var {
            st::ftCommon_UpdateFacing(&mut *fp);
            ext::ftPartSetRotY(fp, 0, (core::f64::consts::FRAC_PI_2 * (*fp).facing_dir as f64) as f32);
        }
    }
}

pub fn ftCa_SpecialHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHi_IASA");
    unsafe {
        if (*fpp(gobj)).cmd_vars[0] != 0 {
            doAirIASA(gobj);
        }
    }
}

pub fn ftCa_SpecialHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHi_Phys");
    unsafe {
        let fp = &mut *fpp(gobj);
        let da = &*da(fp);
        fp.self_vel.x = fp.mv.ca.specialhi.vel.x;
        fp.self_vel.y = fp.mv.ca.specialhi.vel.y;
        fp.self_vel.z = 0.0;
        let drift_max = fp.co_attrs.air_drift_max;
        if !fc::calc_self_accel_deaccel_quick(fp, da.specialhi_horz_vel * drift_max) {
            let stick_mul = fp.co_attrs.air_drift_stick_mul * da.specialhi_air_friction_mul;
            let target = fp.co_attrs.air_drift_max * da.specialhi_horz_vel;
            fc::calc_self_accel_drift_simple_no_friction(fp, fh::common_data().x258, stick_mul, target);
        }
        fp.mv.ca.specialhi.vel.x = fp.x74_self_accel.x + fp.self_vel.x;
        fp.mv.ca.specialhi.vel.y = fp.x74_self_accel.y + fp.self_vel.y;
        st::ft_80085134(gobj);
        let fp = &mut *fpp(gobj);
        fp.x74_self_accel.y = 0.0;
        fp.x74_self_accel.x = 0.0;
        fp.self_vel.x = fp.self_vel.x + fp.mv.ca.specialhi.vel.x;
        fp.self_vel.y = fp.self_vel.y + fp.mv.ca.specialhi.vel.y;
    }
}

pub fn ftCa_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirHi_Enter");
    unsafe { enter(gobj, ftCa_MS_SpecialAirHi as i32) }
}

/// `doAirColl` (static).
pub fn doAirColl(gobj: *mut HSD_GObj) {
    melee_core::ct!("doAirColl");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ft_CheckGroundAndLedge(gobj, 0) != 0 {
            if (*fp).mv.ca.specialhi.x2_b1() != 0 {
                st::ftCo_LandingFallSpecial_Enter(gobj, 0, (*da).specialhi_landing_lag);
            } else {
                ext::ft_80083B68(gobj);
            }
        } else if (*fp).mv.ca.specialhi.x2_b1() != 0 && st::ftCliffCommon_80081298(gobj) != 0 {
            ext::ftCliffCommon_80081370(gobj);
        }
    }
}

pub fn ftCa_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Air {
            doAirColl(gobj);
        } else if st::ft_80082708(gobj) == 0 {
            st::ftCommon_8007D5D4(&mut *fp);
        }
    }
}

pub fn ftCa_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirHi_Anim");
    unsafe {
        let da = da(fpp(gobj));
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_80096900(gobj, 1, 1, 0, (*da).specialhi_freefall_air_spd_mul, (*da).specialhi_landing_lag);
        }
    }
}

pub fn ftCa_SpecialAirHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirHi_IASA");
    ftCa_SpecialHi_IASA(gobj)
}

pub fn ftCa_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirHi_Phys");
    ftCa_SpecialHi_Phys(gobj)
}

pub fn ftCa_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirHi_Coll");
    ftCa_SpecialHi_Coll(gobj)
}

/// Grab callback: enter SpecialHiCatch.
pub fn ftCa_SpecialLw_800E5128(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLw_800E5128");
    unsafe {
        let fp = fpp(gobj);
        let vic_fp = fpp((*fp).victim_gobj);
        st::Fighter_ChangeMotionState(gobj, ftCa_MS_SpecialHiCatch as i32, 2, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).set_x2222_b2(1);
        st::ftCommon_8007E2F4(&mut *fp, 511);
        st::ftCommon_8007E2FC(gobj);
        if (*vic_fp).ground_or_air == GA_Air {
            (*fp).anon_221b.set_x221B_b7(0);
        } else {
            ext::ftCo_800DB368(vic_fp, fp);
            (*fp).anon_221b.set_x221B_b7(1);
            (*fp).accessory4_cb = Some(cb_ftCa_SpecialLw_800E550C);
        }
    }
}

pub fn ftCa_SpecialHiCatch_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHiCatch_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        doCatchAnim(gobj);
    }
}

pub fn ftCa_SpecialHiCatch_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCa_SpecialHiCatch_IASA");}
pub fn ftCa_SpecialHiCatch_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCa_SpecialHiCatch_Phys");}

pub fn ftCa_SpecialHiCatch_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHiCatch_Coll");
    unsafe {
        if (*fpp(gobj)).anon_221b.x221B_b7() == 0 {
            ext::ft_80083B68(gobj);
        }
    }
}

/// `doCatchAnim` (static): throw the caught victim.
pub fn doCatchAnim(gobj: *mut HSD_GObj) {
    melee_core::ct!("doCatchAnim");
    unsafe {
        let fp = fpp(gobj);
        let vic_gobj = (*fp).victim_gobj;
        (*fp).cmd_vars[0] = 0;
        (*fp).mv.ca.specialhi.set_x2_b0(0);
        (*fp).mv.ca.specialhi.vel.x = 0.0;
        (*fp).mv.ca.specialhi.vel.y = 0.0;
        st::Fighter_ChangeMotionState(
            gobj,
            ftCa_MS_SpecialHiThrow as i32,
            (Ft_MF_Unk19 | Ft_MF_KeepGfx) as u32,
            0.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftCommon_8007E2F4(&mut *fp, 0);
        ext::ftCo_800DE2A8(gobj, vic_gobj);
        ext::ftCo_800DE7C0(vic_gobj, 0, 0);
    }
}

pub fn ftCa_SpecialHiThrow0_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHiThrow0_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
        st::ftCommon_8007D60C(&mut *fp);
        if (*fp).cmd_vars[0] != 0 {
            (*fp).cmd_vars[0] = 0;
            (*fp).mv.ca.specialhi.set_x2_b0(1);
        }
    }
}

pub fn ftCa_SpecialHiThrow0_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCa_SpecialHiThrow0_IASA");}

pub fn ftCa_SpecialHiThrow0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHiThrow0_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).mv.ca.specialhi.x2_b0() != 0 {
            ftCa_SpecialHi_Phys(gobj);
            let vel_y = (*fp).self_vel.y - (*fp).mv.ca.specialhi.vel.y;
            let tv = (*fp).co_attrs.terminal_velocity;
            st::ftCommon_Fall(&mut *fp, (*da).specialhi_catch_grav, tv);
            (*fp).mv.ca.specialhi.vel.y = (*fp).self_vel.y - vel_y;
        } else {
            st::ft_80085134(gobj);
        }
    }
}

/// Accessory callback: stick to the victim position.
pub fn ftCa_SpecialLw_800E550C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialLw_800E550C");
    unsafe {
        let fp = fpp(gobj);
        let vic_fp = fpp((*fp).victim_gobj);
        (*fp).cur_pos = (*vic_fp).cur_pos;
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCa_SpecialLw_800E49FC` (static): Falcon Dive state reset.
pub fn ftCa_SpecialLw_800E49FC(fp: &mut Fighter) {
    melee_core::ct!("ftCa_SpecialLw_800E49FC");
    let da: &ftCaptain_DatAttrs = da_g(fp);
    fp.x1968_jumpsUsed = caps_max_jumps(fp) as u8;
    unsafe {
        fp.mv.ca.specialhi.x0 = da.specialhi_air_var as _;
    }
    fp.cmd_vars[0] = 0;
    fp.cmd_vars[1] = da.specialhi_unk2 as u32;
    unsafe {
        fp.mv.ca.specialhi.vel.x = 0.0;
        fp.mv.ca.specialhi.vel.y = 0.0;
        fp.mv.ca.specialhi.set_x2_b0(0);
        fp.mv.ca.specialhi.set_x2_b1(0);
    }
}

/// `ftCa_SpecialHiThrow0_Coll` (src/melee/ft/kinds/ftCaptain/ftcaptainspecialhi.c).
pub fn ftCa_SpecialHiThrow0_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHiThrow0_Coll");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ft_80081D0C(gobj) != 0) {
            st::ftCo_LandingFallSpecial_Enter(gobj, 0 /* false */, (*((*fp).dat_attrs as *const melee_types::ftCaptain_DatAttrs)).specialhi_landing_lag);
        }
    }
}

/// `ftCa_SpecialHiThrow1_Coll` (src/melee/ft/kinds/ftCaptain/ftcaptainspecialhi.c).
pub fn ftCa_SpecialHiThrow1_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHiThrow1_Coll");
    unsafe {
        st::ftCo_AirCatchHit_Coll(gobj);
    }
}

/// `ftCa_SpecialHi_Anim` (src/melee/ft/kinds/ftCaptain/ftcaptainspecialhi.c).
pub fn ftCa_SpecialHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialHi_Anim");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            st::ftCo_80096900(gobj, 1, 1, 0 /* false */, ((*((*fp).dat_attrs as *const melee_types::ftCaptain_DatAttrs)).specialhi_freefall_air_spd_mul as f32), ((*((*fp).dat_attrs as *const melee_types::ftCaptain_DatAttrs)).specialhi_landing_lag as f32));
        }
    }
}
