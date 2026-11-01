//! src/melee/ft/kinds/ftCaptain/ftcaptainspecialn.c: Falcon / Warlock Punch (neutral B).
//! No fused sites (tools/rust-port/fma_sites.py). `order_sdata2` is `MUST_MATCH`-only (not
//! built on PC). Pure parts checked against the C by tests/tu_ftcaptainspecialn.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter};
use crate::stubs as st;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::*;

/// Already ported (round 11): re-exported under this TU.


/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::{HSD_GObj, Vec3};
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
    pub static mut LB_800119DC: Option<fn(*mut Vec3, i32, f32, f32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn lb_800119DC(pos: *mut Vec3, a: i32, b: f32, c: f32, d: f32)  {
                match unsafe { LB_800119DC } {
                    Some(f) => f(pos, a, b, c, d),
                    None => { unsafe { melee_runtime::tu::lb_00F9::lb_800119DC(pos as _, a as _, b as _, c as _, d as _); } return; },
                }
            }
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing pointers as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn joint(fp: *mut Fighter, part: usize) -> *mut core::ffi::c_void {
    (*(*fp).parts.add(part)).joint as *mut core::ffi::c_void
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

/// `ftCaptain_SpecialN_CreateWindEffect` (static inline): Ganondorf's wind puffs.
pub fn ftCaptain_SpecialN_CreateWindEffect(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCaptain_SpecialN_CreateWindEffect");
    unsafe {
        let fp = fpp(gobj);
        let cur_frame = (*fp).cur_anim_frame as i32;
        let kind = melee_fighter::hooks::ftlib_get_kind(gobj);
        match kind {
            Ft_Kind_Captain => {}
            Ft_Kind_Ganon => {
                if cur_frame & 1 != 0 {
                    if (16..=50).contains(&cur_frame) {
                        ext::lb_800119DC(&mut (*fp).cur_pos, 2, 2.0, 2.0, 0.0);
                    } else if (51..=68).contains(&cur_frame) {
                        ext::lb_800119DC(&mut (*fp).cur_pos, 2, 4.0, 4.0, 0.0);
                    }
                }
            }
            _ => {}
        }
    }
}

/// `ftCaptain_SpecialN_GetAngleVel` (static): stick angle of the aerial punch, in radians.
pub fn ftCaptain_SpecialN_GetAngleVel(fp: &Fighter) -> f32 {
    melee_core::ct!("ftCaptain_SpecialN_GetAngleVel");
    let da: &ftCaptain_DatAttrs = unsafe { &*(fp.dat_attrs as *const ftCaptain_DatAttrs) };
    let mut stick_y = stick_get_dir(fp.input.lstick[0].y, 0.0);
    let max = da.specialn_stick_range_y_pos;
    if stick_y > max {
        stick_y = max;
    }
    let min = da.specialn_stick_range_y_neg;
    stick_y -= min;
    if stick_y < 0.0 {
        stick_y = 0.0;
    }
    if fp.input.lstick[0].y < 0.0 {
        stick_y = -stick_y;
    }
    let f = 1.0_f32 * 0.01745329252_f32;
    f * (stick_y * da.specialn_angle_diff / (max - min))
}

pub fn ftCa_SpecialN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialN_Anim");
    ftCaptain_SpecialN_CreateWindEffect(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

pub fn ftCa_SpecialAirN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirN_Anim");
    ftCaptain_SpecialN_CreateWindEffect(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

pub fn ftCa_SpecialN_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCa_SpecialN_IASA");}

/// `ftCa_SpecialAirN_IASA`: the fighter-only body is [`special_air_n_iasa`].
pub fn ftCa_SpecialAirN_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirN_IASA");
    unsafe { special_air_n_iasa(&mut *fpp(gobj)) }
}

/// Body of `ftCa_SpecialAirN_IASA`: aim the punch on the frame the script sets cmd_vars[0].
pub fn special_air_n_iasa(fp: &mut Fighter) {
    melee_core::ct!("special_air_n_iasa");
    let da: &ftCaptain_DatAttrs = unsafe { &*(fp.dat_attrs as *const ftCaptain_DatAttrs) };
    if fp.cmd_vars[0] != 0 {
        fp.cmd_vars[0] = 0;
        let vel = ftCaptain_SpecialN_GetAngleVel(fp);
        fp.self_vel.y = da.specialn_vel_x * sinf(vel);
        fp.self_vel.x = da.specialn_vel_x * (fp.facing_dir * cosf(vel));
    }
}

/// `doPhys` (static inline): the punch effect on the script's throw_flags_b1 pulse.
pub fn doPhys(gobj: *mut HSD_GObj) {
    melee_core::ct!("doPhys");
    unsafe {
        let fp = fpp(gobj);
        let throw_b1 = if (*fp).anon_2210.anon_0.throw_flags_b1() != 0 {
            (*fp).anon_2210.anon_0.set_throw_flags_b1(0);
            true
        } else {
            false
        };
        if throw_b1 {
            if (*fp).x2219_b0() == 0 {
                let kind = melee_fighter::hooks::ftlib_get_kind(gobj);
                match kind {
                    Ft_Kind_Captain => {
                        ext::efSync_Spawn(1167, gobj, &[joint(fp, FtPart_TopN as usize), joint(fp, 57)]);
                    }
                    Ft_Kind_Ganon => {
                        ext::efSync_Spawn(1291, gobj, &[joint(fp, FtPart_TopN as usize), joint(fp, 78)]);
                    }
                    _ => {}
                }
                (*fp).set_x2219_b0(1);
            } else {
                st::ftCommon_8007DB24(gobj);
            }
        }
    }
}

pub fn ftCa_SpecialN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialN_Phys");
    doPhys(gobj);
    st::ft_80084FA8(gobj);
}

pub fn ftCa_SpecialAirN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirN_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftCaptain_DatAttrs;
        doPhys(gobj);
        match (*fp).cmd_vars[1] {
            0 => st::ft_80084EEC(gobj),
            1 => {
                (*fp).self_vel.y *= (*da).specialn_vel_mul;
                (*fp).self_vel.x *= (*da).specialn_vel_mul;
            }
            2 => st::ft_80084DB0(gobj),
            _ => {}
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCa_SpecialN_Enter` (src/melee/ft/kinds/ftCaptain/ftcaptainspecialn.c).
pub fn ftCa_SpecialN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialN_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[1] = 0_u32;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).anon_2210.throw_flags = 0_u32;
        st::Fighter_ChangeMotionState(gobj, 347 /* ftCa_MS_SpecialN */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `ftCa_SpecialAirN_Enter` (src/melee/ft/kinds/ftCaptain/ftcaptainspecialn.c).
pub fn ftCa_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirN_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[1] = 0_u32;
        (*fp).cmd_vars[0] = 0_u32;
        (*fp).anon_2210.throw_flags = 0_u32;
        st::Fighter_ChangeMotionState(gobj, 348 /* ftCa_MS_SpecialAirN */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `ftCa_SpecialN_Coll` (src/melee/ft/kinds/ftCaptain/ftcaptainspecialn.c).
pub fn ftCa_SpecialN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialN_Coll");
    unsafe {
        if (st::ft_800827A0(gobj) == 0) {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 348 /* ftCa_MS_SpecialAirN */, 0xC4C5882u32 /* transition_flags */);
            (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
            (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
            st::ftCommon_ClampAirDrift(&mut *fp);
        }
    }
}

/// `ftCa_SpecialAirN_Coll` (src/melee/ft/kinds/ftCaptain/ftcaptainspecialn.c).
pub fn ftCa_SpecialAirN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCa_SpecialAirN_Coll");
    unsafe {
        if (st::ft_80081D0C(gobj) != 0) {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 347 /* ftCa_MS_SpecialN */, 0xC4C5882u32 /* transition_flags */);
            (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
            (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
        }
    }
}
