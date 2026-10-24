//! src/melee/ft/kinds/ftKirby/ftkirbyspecialcaptain.c: Kirby with Captain Falcon's / Ganondorf's
//! hat, neutral B (Falcon Punch / Warlock Punch). No fused sites (tools/rust-port/fma_sites.py).
//! `ca_special_air_n_iasa` is checked bit for bit against the C (tests/tu_ftkirbyspecialcaptain.rs,
//! gen/turef/tu_ftkirbyspecialcaptain_ref.c); the callbacks are smoke-tested through recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyspecialcaptain::{ftKb_CaSpecialAirN_Anim, ftKb_CaSpecialN_Anim};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::HSD_GObj;
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
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing pointers as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
}

/// `Fighter_SetEffectHitlagCallbacks(fp)` (ft/inlines.h): efLib_PauseAll / efLib_ResumeAll as the hitlag
/// callbacks (the entries panic `TODO` when the game calls them, until efLib is ported).
#[inline(always)]
unsafe fn set_effect_hitlag_cbs(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn joint(fp: *mut Fighter, part: usize) -> *mut core::ffi::c_void {
    (*(*fp).parts.add(part)).joint as *mut core::ffi::c_void
}

/// `MTXDegToRad(a)` (`(a)*0.01745329252f`).
#[inline(always)]
fn deg_to_rad(a: f32) -> f32 {
    a * 0.01745329252_f32
}

/// `ftKb_Ca_transition_flags` (static const).
pub const ftKb_Ca_transition_flags: u32 = Ft_MF_KeepGfx
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipRumble
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipColAnim
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27;

/// `ftKb_SpecialNCa_800F99BC`: enter Falcon Punch (ground); Ganondorf's hat uses the Warlock Punch id.
pub fn ftKb_SpecialNCa_800F99BC(gobj: *mut HSD_GObj) {
    unsafe {
        let mut msid = ftKb_MS_CaSpecialN;
        let fp = fpp(gobj);
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).anon_2210.throw_flags = 0;
        if (*fp).u.kb.hat.kind != Ft_Kind_Captain {
            msid = ftKb_MS_GnSpecialN;
        }
        st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_effect_hitlag_cbs(fp);
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `ftKb_SpecialNCa_800F9A54`: the aerial entry.
pub fn ftKb_SpecialNCa_800F9A54(gobj: *mut HSD_GObj) {
    unsafe {
        let mut msid = ftKb_MS_CaSpecialAirN;
        let fp = fpp(gobj);
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).anon_2210.throw_flags = 0;
        if (*fp).u.kb.hat.kind != Ft_Kind_Captain {
            msid = ftKb_MS_GnSpecialAirN;
        }
        st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_effect_hitlag_cbs(fp);
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftKb_CaSpecialN_IASA(_gobj: *mut HSD_GObj) {}

/// `ftKb_CaSpecialAirN_IASA`: the fighter-only body is [`ca_special_air_n_iasa`].
pub fn ftKb_CaSpecialAirN_IASA(gobj: *mut HSD_GObj) {
    unsafe { ca_special_air_n_iasa(&mut *fpp(gobj)) }
}

/// Body of `ftKb_CaSpecialAirN_IASA`: aim the punch on the frame the script sets cmd_vars[0].
pub fn ca_special_air_n_iasa(fp: &mut Fighter) {
    let da: &ftKb_DatAttrs = unsafe { &*(fp.dat_attrs as *const ftKb_DatAttrs) };
    if fp.cmd_vars[0] != 0 {
        fp.cmd_vars[0] = 0;
        let stick_y = fp.input.lstick[0].y;
        let mut abs_y = if stick_y < 0.0 { -stick_y } else { stick_y };
        if abs_y > da.specialn_ca_y_axis_range {
            abs_y = da.specialn_ca_y_axis_range;
        }
        let mut range = abs_y - da.specialn_ca_x_axis_range;
        if range < 0.0 {
            range = 0.0;
        }
        if fp.input.lstick[0].y < 0.0 {
            range = -range;
        }
        let angle = deg_to_rad(
            (range * da.specialn_ca_angle_difference) / (da.specialn_ca_y_axis_range - da.specialn_ca_x_axis_range),
        );
        fp.self_vel.y = da.specialn_ca_forward_momentum * sinf(angle);
        fp.self_vel.x = da.specialn_ca_forward_momentum * (fp.facing_dir * cosf(angle));
    }
}

/// The throw_flags_b1 pulse shared by both Phys callbacks: first pulse spawns the punch effect
/// (id by hat kind) and sets x2219_b0, later ones call ftCommon_8007DB24.
unsafe fn punch_pulse(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    let throw_b1 = if (*fp).anon_2210.anon_0.throw_flags_b1() != 0 {
        (*fp).anon_2210.anon_0.set_throw_flags_b1(0);
        true
    } else {
        false
    };
    if throw_b1 {
        if (*fp).x2219_b0() == 0 {
            match (*fp).u.kb.hat.kind {
                Ft_Kind_Captain => {
                    ext::efSync_Spawn(1195, gobj, &[joint(fp, 0), joint(fp, 44)]);
                }
                Ft_Kind_Ganon => {
                    ext::efSync_Spawn(1203, gobj, &[joint(fp, 0), joint(fp, 44)]);
                }
                _ => {}
            }
            (*fp).set_x2219_b0(1);
        } else {
            st::ftCommon_8007DB24(gobj);
        }
    }
}

pub fn ftKb_CaSpecialN_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        punch_pulse(gobj, fp);
        st::ft_80084FA8(gobj);
    }
}

pub fn ftKb_CaSpecialAirN_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        punch_pulse(gobj, fp);
        match (*fp).cmd_vars[1] {
            0 => st::ft_80084EEC(gobj),
            1 => {
                (*fp).self_vel.y *= (*da).specialn_ca_additional_vertical_momentum;
                (*fp).self_vel.x *= (*da).specialn_ca_additional_vertical_momentum;
            }
            2 => st::ft_80084DB0(gobj),
            _ => {}
        }
    }
}

pub fn ftKb_CaSpecialN_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        if st::ft_800827A0(gobj) == 0 {
            let fp = fpp(gobj);
            st::ftCommon_8007D5D4(&mut *fp);
            let anim_frame = (*fp).cur_anim_frame;
            let mut msid = ftKb_MS_CaSpecialAirN;
            if (*fp).u.kb.hat.kind != Ft_Kind_Captain {
                msid = ftKb_MS_GnSpecialAirN;
            }
            st::Fighter_ChangeMotionState(gobj, msid as i32, ftKb_Ca_transition_flags, anim_frame, 1.0, 0.0, core::ptr::null_mut());
            set_effect_hitlag_cbs(fp);
            melee_fighter::ftcommon::clamp_air_drift(&mut *fp);
        }
    }
}

pub fn ftKb_CaSpecialAirN_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            let fp = fpp(gobj);
            st::ftCommon_8007D7FC(&mut *fp);
            let anim_frame = (*fp).cur_anim_frame;
            let mut msid = ftKb_MS_CaSpecialN;
            if (*fp).u.kb.hat.kind != Ft_Kind_Captain {
                msid = ftKb_MS_GnSpecialN;
            }
            st::Fighter_ChangeMotionState(gobj, msid as i32, ftKb_Ca_transition_flags, anim_frame, 1.0, 0.0, core::ptr::null_mut());
            set_effect_hitlag_cbs(fp);
        }
    }
}
