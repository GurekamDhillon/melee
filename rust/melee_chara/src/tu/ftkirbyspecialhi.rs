//! src/melee/ft/kinds/ftKirby/ftkirbyspecialhi.c: Kirby's up-B (Final Cutter).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function). Smoke-tested
//! through recording hooks by tests/tu_ftkirbyspecialhi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/11): re-exported under this TU.
pub use crate::chars::r11::ftKirby::ftkirbyspecialhi::{
    ftKb_SpecialAirHi1_Anim, ftKb_SpecialAirHi1_Coll, ftKb_SpecialAirHi2_Anim, ftKb_SpecialHi1_Anim,
    ftKb_SpecialHi2_Anim,
};
pub use crate::chars::r2::ftKirby::ftkirbyspecialhi::{ftKb_SpecialAirHiEnd_Phys, ftKb_SpecialHi4_Phys};

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
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// Like `hook!`, but an unset hook falls back to an existing native port.
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
    pub static mut EFLIB_SETFLAGS: Option<fn(*mut HSD_GObj, i32)> = None;
    #[allow(non_snake_case)]
    pub fn efLib_SetFlags(gobj: *mut HSD_GObj, flags: i32) {
        match unsafe { EFLIB_SETFLAGS } {
            Some(f) => f(gobj, flags),
            None => { unsafe { let _ = melee_world::tu::efn::eflib::efLib_SetFlags(gobj as _, flags as _); } },
        }
    }
    pub static mut IT_8029BAB8: Option<fn(*mut HSD_GObj, *mut Vec3, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8029BAB8(gobj: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32)  {
                match unsafe { IT_8029BAB8 } {
                    Some(f) => f(gobj, pos, facing_dir),
                    None => { unsafe { let _ = melee_items::tu::itkirbycutterbeam::it_8029BAB8(gobj as _, pos as _, facing_dir as _); } return; },
                }
            }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    pub static mut FTCLIFFCOMMON_80081370: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCliffCommon_80081370(gobj: *mut HSD_GObj)  {
                match unsafe { FTCLIFFCOMMON_80081370 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = melee_fighter::tu::ftcliffcommon::ftcliffcommon_80081370(gobj as _); } return; },
                }
            }
    hook_or!(FT_80082578, ft_80082578, melee_fighter::tu::ft_081b::ft_80082578, (gobj: *mut HSD_GObj) -> bool);
    /// `efSync_Spawn(gfx_id, gobj, ...)` (variadic in C; `va` are the extra word arguments).
    pub static mut EFSYNC_SPAWN: Option<fn(i32, *mut HSD_GObj, &[usize]) -> *mut core::ffi::c_void> = None;
    pub fn efSync_Spawn(id: i32, gobj: *mut HSD_GObj, va: &[usize]) -> *mut core::ffi::c_void {
        match unsafe { EFSYNC_SPAWN } {
            Some(f) => f(id, gobj, va),
            None => unsafe { crate::adapt::efSync_Spawn_usize(id, gobj, va) },
        }
    }
    /// Recording hook; unset, the native `melee_fighter::tu::ftparts` port runs.
    pub static mut FTPARTSETROTX: Option<fn(*mut Fighter, i32, f32)> = None;
    pub fn ftPartSetRotX(fp: *mut Fighter, part: i32, v: f32) {
        match unsafe { FTPARTSETROTX } {
            Some(f) => f(fp, part, v),
            None => melee_fighter::tu::ftparts::ftPartSetRotX(unsafe { &*fp }, part, v),
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
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftKb_DatAttrs {
    (*fp).dat_attrs as *const ftKb_DatAttrs
}

/// `ftKb_MF_SpecialHi_Coll`.
const MF_SPECIALHI_COLL: u32 =
    (Ft_MF_KeepGfx | Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim | Ft_MF_SkipItemVis | Ft_MF_Unk19) as u32;

/// `ftKb_SpecialN_set_cbs` (ftKirby/inlines.h).
#[inline(always)]
unsafe fn set_cbs(fp: *mut Fighter) {
    (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
    (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
}
/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
unsafe fn set_hitlag_cbs(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// Accessory callback entry for `fn_800F21E8`.
pub unsafe extern "C" fn cb_fn_800F21E8(g: G) {
    melee_core::ct!("cb_fn_800F21E8");
    fn_800F21E8(g);
}

/// `facing_dir * atan2f(floor.normal.x, floor.normal.y)` (group 1's MSL atan2f).
#[inline(always)]
unsafe fn slope_rot(fp: *mut Fighter) -> f32 {
    (*fp).facing_dir
        * melee_runtime::lbtrigf::atan2f((*fp).coll_data.floor.normal.x, (*fp).coll_data.floor.normal.y)
}

pub fn fn_800F21E8(gobj: G) {
    melee_core::ct!("fn_800F21E8");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).mv.kb.specialhi.xC == 0 && (*fp).cmd_vars[2] != 0 {
            let mut pos: Vec3 = core::mem::zeroed();
            (*fp).cmd_vars[2] = 0;
            (*fp).mv.kb.specialhi.xC = 1;
            ext::lb_8000B1CC((*(*fp).parts).joint, core::ptr::null(), &mut pos);
            pos.x += if (*fp).facing_dir == 1.0 {
                (*da).specialhi_projectile_spawn_x
            } else {
                -(*da).specialhi_projectile_spawn_x
            };
            pos.y += (*da).specialhi_projectile_spawn_y;
            pos.z = 0.0;
            ext::efLib_SetFlags(gobj, 1);
            pos.y -= 1.0;
            ext::it_8029BAB8(gobj, &mut pos, (*fp).facing_dir);
        }
    }
}

pub fn ftKb_AttackDashAir_800F22D4(gobj: G) {
    melee_core::ct!("ftKb_AttackDashAir_800F22D4");
    unsafe { ext::ftPartSetRotX(fpp(gobj), 0, 0.0) }
}

unsafe fn enter(gobj: G, msid: u32) {
    let fp = fpp(gobj);
    st::Fighter_ClearCmdVars(&mut *fp);
    (*fp).mv.kb.specialhi.x0 = 0;
    (*fp).mv.kb.specialhi.x4 = 0;
    (*fp).mv.kb.specialhi.x8.i = 0;
    (*fp).mv.kb.specialhi.xC = 0;
    st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
    let fp = fpp(gobj);
    ext::efSync_Spawn(0x494, gobj, &[]);
    (*fp).set_x2219_b0(1);
    set_hitlag_cbs(fp);
}

pub fn ftKb_SpecialHi_Enter(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi_Enter");
    unsafe { enter(gobj, ftKb_MS_SpecialHi1 as u32) }
}

pub fn ftKb_SpecialAirHi_Enter(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirHi_Enter");
    unsafe { enter(gobj, ftKb_MS_SpecialAirHi1 as u32) }
}

pub fn ftKb_SpecialHi3_Anim(_gobj: G) { melee_core::ct!("ftKb_SpecialHi3_Anim");}

unsafe fn end_anim(gobj: G) {
    let fp = fpp(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftPartSetRotX(fpp(gobj), 0, 0.0);
        if (*fp).ground_or_air == GA_Ground {
            st::ft_8008A2BC(gobj);
        } else {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftKb_SpecialHi4_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi4_Anim");
    unsafe { end_anim(gobj) }
}

pub fn ftKb_SpecialAirHi3_Anim(_gobj: G) { melee_core::ct!("ftKb_SpecialAirHi3_Anim");}

pub fn ftKb_SpecialAirHiEnd_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirHiEnd_Anim");
    unsafe { end_anim(gobj) }
}

/// Shared body of the two reverse-up-B IASA callbacks (the C bodies are the same).
unsafe fn reverse_iasa(gobj: G) {
    let fp = fpp(gobj);
    let da = da(fp);
    if (*fp).cmd_vars[3] == 0 && (*fp).mv.kb.specialhi.x4 == 0 {
        let lstick_x = (*fp).input.lstick[0].x;
        let mag = if lstick_x < 0.0 { -lstick_x } else { lstick_x };
        if mag > (*da).specialhi_reverse_upb_stick_range {
            if ((*fp).facing_dir == 1.0 && lstick_x < 0.0) || ((*fp).facing_dir == -1.0 && lstick_x > 0.0) {
                (*fp).cmd_vars[3] = 1;
                (*fp).mv.kb.specialhi.x4 = 1;
                st::ftCommon_UpdateFacing(&mut *fp);
                ext::ftPartSetRotY(fp, 0, (core::f64::consts::FRAC_PI_2 * (*fp).facing_dir as f64) as f32);
            }
        }
    }
}

pub fn ftKb_SpecialHi1_IASA(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi1_IASA");
    unsafe { reverse_iasa(gobj) }
}
pub fn ftKb_SpecialHi2_IASA(_gobj: G) { melee_core::ct!("ftKb_SpecialHi2_IASA");}
pub fn ftKb_SpecialHi3_IASA(_gobj: G) { melee_core::ct!("ftKb_SpecialHi3_IASA");}
pub fn ftKb_SpecialHi4_IASA(_gobj: G) { melee_core::ct!("ftKb_SpecialHi4_IASA");}
pub fn ftKb_SpecialAirHi1_IASA(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirHi1_IASA");
    unsafe { reverse_iasa(gobj) }
}
pub fn ftKb_SpecialAirHi2_IASA(_gobj: G) { melee_core::ct!("ftKb_SpecialAirHi2_IASA");}
pub fn ftKb_SpecialAirHi3_IASA(_gobj: G) { melee_core::ct!("ftKb_SpecialAirHi3_IASA");}
pub fn ftKb_SpecialAirHiEnd_IASA(_gobj: G) { melee_core::ct!("ftKb_SpecialAirHiEnd_IASA");}

/// Ground drift shared by Hi1/Hi2 Phys after the gravity step.
unsafe fn ground_drift(fp: *mut Fighter) {
    let da = da(fp);
    let fp = &mut *fp;
    melee_fighter::ftcommon::calc_ground_accel_accel_to_lstick_x(
        fp,
        0.0,
        fp.co_attrs.air_drift_stick_mul * (*da).specialhi_horizontal_momentum,
        fp.co_attrs.air_drift_max,
    );
}

pub fn ftKb_SpecialHi1_Phys(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi1_Phys");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80084FA8(gobj);
        ground_drift(fp);
        melee_fighter::ftcommon::set_self_movement_from_grounded_movement(&mut *fp);
    }
}

pub fn ftKb_SpecialHi2_Phys(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi2_Phys");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80085134(gobj);
        ground_drift(fp);
        melee_fighter::ftcommon::set_self_movement_from_grounded_movement(&mut *fp);
    }
}

pub fn ftKb_SpecialHi3_Phys(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi3_Phys");
    unsafe {
        crate::chars::ftKirby::ftkirbyspecialhi::ftKb_SpecialHi3_Phys(&mut *fpp(gobj));
    }
}

/// Air drift shared by AirHi1/AirHi2 Phys.
unsafe fn air_hi_phys(gobj: G) {
    let fp = fpp(gobj);
    let da = da(fp);
    st::ft_80085134(gobj);
    if (*fp).self_vel.y > 0.0 {
        (*fp).self_vel.y *= (*da).specialhi_vertical_momentum;
    }
    let fp = &mut *fp;
    melee_fighter::ftcommon::calc_self_accel_drift_simple_no_friction(
        fp,
        0.0,
        fp.co_attrs.air_drift_stick_mul * (*da).specialhi_horizontal_momentum,
        fp.co_attrs.air_drift_max,
    );
}

pub fn ftKb_SpecialAirHi1_Phys(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirHi1_Phys");
    unsafe { air_hi_phys(gobj) }
}
pub fn ftKb_SpecialAirHi2_Phys(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirHi2_Phys");
    unsafe { air_hi_phys(gobj) }
}

pub fn ftKb_SpecialAirHi3_Phys(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirHi3_Phys");
    unsafe {
        crate::chars::ftKirby::ftkirbyspecialhi::ftKb_SpecialAirHi3_Phys(&mut *fpp(gobj));
    }
}

pub fn ftKb_SpecialHi1_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi1_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftKb_MS_SpecialAirHi1 as i32, MF_SPECIALHI_COLL);
            set_cbs(fp);
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            ext::ftPartSetRotX(fp, 0, 0.0);
        }
    }
}

/// The ledge/ground check argument: `facing_dir == 1 ? 1 : -1`.
#[inline(always)]
unsafe fn dir_arg(fp: *mut Fighter) -> i32 {
    if (*fp).facing_dir == 1.0 {
        1
    } else {
        -1
    }
}

/// Common landing branch of Hi2/Hi3 Coll (and the Air variants): switch to the end state `msid`.
unsafe fn land_on_ledge(gobj: G, fp: *mut Fighter, from_air: bool, msid: i32) {
    if from_air {
        st::ftCommon_8007D7FC(&mut *fp);
    } else {
        st::ftCommon_8007D5D4(&mut *fp);
    }
    st::Fighter_ChangeMotionState(gobj, msid, 0xC1082, 0.0, 1.0, 0.0, core::ptr::null_mut());
    set_cbs(fp);
    (*fp).accessory4_cb = Some(cb_fn_800F21E8);
    (*fp).self_vel.z = 0.0;
    (*fp).self_vel.y = 0.0;
    (*fp).self_vel.x = 0.0;
    (*fp).gr_vel = 0.0;
    ext::ftPartSetRotX(fp, 0, slope_rot(fp));
}

pub fn ftKb_SpecialHi2_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi2_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.kb.specialn_pe.facing_dir > 0x14 {
            let i = dir_arg(fp);
            if st::ft_CheckGroundAndLedge(gobj, i) != 0 {
                (*fp).mv.kb.specialn_pe.facing_dir += 1;
                land_on_ledge(gobj, fp, false, ftKb_MS_SpecialAirHi4 as i32);
                return;
            }
            if st::ftCliffCommon_80081298(gobj) != 0 {
                ext::ftCliffCommon_80081370(gobj);
            }
        } else {
            (*fp).mv.kb.specialn_pe.facing_dir += 1;
            st::ftCommon_8007D60C(&mut *fp);
            ext::ft_80082578(gobj);
        }
    }
}

pub fn ftKb_SpecialHi3_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi3_Coll");
    unsafe {
        let fp = fpp(gobj);
        let i = dir_arg(fp);
        if st::ft_CheckGroundAndLedge(gobj, i) != 0 {
            land_on_ledge(gobj, fp, false, ftKb_MS_SpecialAirHi4 as i32);
            return;
        }
        if st::ftCliffCommon_80081298(gobj) != 0 {
            ext::ftCliffCommon_80081370(gobj);
        }
    }
}

pub fn ftKb_SpecialHi4_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi4_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftKb_MS_SpecialAirHi4 as i32, MF_SPECIALHI_COLL);
            set_cbs(fp);
            (*fp).accessory4_cb = Some(cb_fn_800F21E8);
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            ext::ftPartSetRotX(fp, 0, 0.0);
            return;
        }
        ext::ftPartSetRotX(fp, 0, slope_rot(fp));
    }
}

pub fn ftKb_SpecialAirHi2_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirHi2_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.kb.specialn_pe.facing_dir > 0x14 {
            let i = dir_arg(fp);
            if st::ft_CheckGroundAndLedge(gobj, i) != 0 {
                (*fp).mv.kb.specialn_pe.facing_dir += 1;
                land_on_ledge(gobj, fp, true, ftKb_MS_SpecialHi4 as i32);
                return;
            }
            if st::ftCliffCommon_80081298(gobj) != 0 {
                ext::ftCliffCommon_80081370(gobj);
            }
        } else {
            (*fp).mv.kb.specialn_pe.facing_dir += 1;
            st::ftCommon_8007D60C(&mut *fp);
            ext::ft_80082578(gobj);
        }
    }
}

pub fn ftKb_SpecialAirHi3_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirHi3_Coll");
    unsafe {
        let fp = fpp(gobj);
        let i = dir_arg(fp);
        if st::ft_CheckGroundAndLedge(gobj, i) != 0 {
            land_on_ledge(gobj, fp, true, ftKb_MS_SpecialHi4 as i32);
            return;
        }
        if st::ftCliffCommon_80081298(gobj) != 0 {
            ext::ftCliffCommon_80081370(gobj);
        }
    }
}

pub fn ftKb_SpecialAirHiEnd_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirHiEnd_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftKb_MS_SpecialHi4 as i32, MF_SPECIALHI_COLL);
            set_cbs(fp);
            (*fp).accessory4_cb = Some(cb_fn_800F21E8);
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            ext::ftPartSetRotX(fp, 0, slope_rot(fp));
            return;
        }
        ext::ftPartSetRotX(fp, 0, 0.0);
    }
}
