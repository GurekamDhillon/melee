//! src/melee/ft/kinds/ftKirby/ftkirbyspeciallw.c: Kirby's down-B (Stone) and the Stone slide
//! helpers shared with the up-B hat code. One fused site: ftKb_SpecialHi_800F3570
//! (`(normal_y_slide * sign) + pos_x_alias` is an fmadds; tools/rust-port/fma_sites.py). `ABS` is
//! `x < 0 ? -x : x` (keeps -0.0). Smoke-tested by tests/tu_ftkirbyspeciallw.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_runtime::cabi::srem;
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyspeciallw::{
    ftKb_SpecialAirLwEnd_Phys, ftKb_SpecialAirLwStart_Phys, ftKb_SpecialLw1_Phys, ftKb_SpecialLwEnd_Phys,
};

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
    crate::port_hook!(HSD_RANDI, HSD_Randi, crate::adapt::HSD_Randi, (max: i32) -> i32);
    hook_or!(FTCOMMON_8007F5CC, ftCommon_8007F5CC, melee_fighter::ftcommon3::ftcommon_8007f5cc,
        (gobj: *mut HSD_GObj, arg1: i32));
    hook_or!(FT_GETGROUNDFRICTIONMULTIPLIER, ft_GetGroundFrictionMultiplier,
        ft_get_ground_friction_multiplier_port, (fp: *mut Fighter) -> f32);
    fn ft_get_ground_friction_multiplier_port(fp: *mut Fighter) -> f32 {
        melee_fighter::tu::ft_081b::ft_get_ground_friction_multiplier(unsafe { &mut *fp })
    }
    pub static mut MPCHECKALLREMAP: Option<fn(*mut Vec3, *mut i32, *mut u32, *mut Vec3, i32, i32, f32, f32, f32, f32) -> bool> = None;
    #[allow(non_snake_case)]
    pub fn mpCheckAllRemap(pos_out: *mut Vec3, line_id_out: *mut i32, flags_out: *mut u32, normal_out: *mut Vec3, joint_id_skip: i32, joint_id_only: i32, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
        match unsafe { MPCHECKALLREMAP } {
            Some(f) => f(pos_out, line_id_out, flags_out, normal_out, joint_id_skip, joint_id_only, x0, y0, x1, y1),
            None => { unsafe { melee_world::tu::mplib::mpCheckAllRemap(pos_out as _, line_id_out as _, flags_out as _, normal_out as _, joint_id_skip as _, joint_id_only as _, x0 as _, y0 as _, x1 as _, y1 as _) != 0 } },
        }
    }
    crate::port_hook!(FT_PLAYSFX, ft_PlaySFX, crate::adapt::ft_PlaySFX, (fp: *mut Fighter, sfx: i32, vol: u8, pan: u8));
    crate::port_hook!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, crate::adapt::Camera_RequestQuake, (kind: CmQuakeKind, pos: *mut Vec3));
    pub static mut EFLIB_DESTROYALL: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn efLib_DestroyAll(gobj: *mut HSD_GObj) {
        match unsafe { EFLIB_DESTROYALL } {
            Some(f) => f(gobj),
            None => { unsafe { let _ = melee_world::tu::efn::eflib::efLib_DestroyAll(gobj as _); } },
        }
    }
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
/// C `ABS` on floats: `x < 0 ? -x : x` (so -0.0 stays -0.0).
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// `ftKb_Init_803CB490`: per-frame "stone is visible" flags (MSL `bool` is a 4-byte int).
pub static ftKb_Init_803CB490: [i32; 23] = [0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 1];

/// `ftKb_Init_803CB4EC`: the Stone costume-pick table and the unit-Y vector.
#[allow(non_camel_case_types)]
pub struct ftKb_Init_803CB4EC_t {
    pub ints: [i32; 6],
    pub vec: Vec3,
}
pub static ftKb_Init_803CB4EC: ftKb_Init_803CB4EC_t = ftKb_Init_803CB4EC_t {
    ints: [0, 2, 3, 4, 5, 6],
    vec: Vec3 { x: 0.0, y: 1.0, z: 0.0 },
};

/// `ftKb_Init_800EE74C` / `ftKb_Init_800EE7B8` through `Fighter_SetDamageCallbacks`.
#[inline(always)]
unsafe fn set_damage_cbs(fp: *mut Fighter) {
    (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
    (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
}
/// `ftKb_SpecialN_set_cbs2` (ftKirby/inlines.h).
#[inline(always)]
unsafe fn set_cbs2(fp: *mut Fighter) {
    set_damage_cbs(fp)
}

/// Plain-float callbacks used as Fighter fields elsewhere (hit callbacks etc.) are not in this TU.
/// `ftKb_SpecialHi_800F331C`: step the stone fade-in / fade-out counter and set part visibility.
pub fn ftKb_SpecialHi_800F331C(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi_800F331C");
    unsafe {
        let fp = fpp(gobj);
        match (*fp).mv.kb.specialhi.x8.i {
            1 => (*fp).mv.kb.specialhi.x14 += 1,
            2 => (*fp).mv.kb.specialhi.x14 -= 1,
            _ => {}
        }
        if (*fp).mv.kb.specialhi.x14 >= 0 && (*fp).mv.kb.specialhi.x14 < 0x16 {
            if ftKb_Init_803CB490[(*fp).mv.kb.specialhi.x14 as usize] == 0 {
                st::ftParts_80074B0C(gobj, 0, 0);
                st::ftParts_80074B0C(gobj, 1, 0);
                ext::ftCommon_8007F5CC(gobj, 1);
                (*fp).set_x221E_b4(1);
                (*fp).set_x2225_b2(1);
            } else {
                st::ftParts_80074B0C(gobj, 0, ftKb_Init_803CB4EC.ints[(*fp).mv.kb.specialhi.x10.i as usize]);
                st::ftParts_80074B0C(gobj, 1, -1);
                ext::ftCommon_8007F5CC(gobj, 0);
                (*fp).set_x221E_b4(0);
                (*fp).set_x2225_b2(0);
            }
        } else {
            (*fp).mv.kb.specialhi.x8.i = 0;
        }
    }
}

pub fn ftKb_SpecialHi_800F346C(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi_800F346C");
    unsafe {
        let fp = fpp(gobj);
        let dat_attr = da(fp);
        match (*fp).cmd_vars[0] {
            1 => {
                if (*fp).mv.kb.specialhi.x8.i == 0 && (*fp).mv.kb.specialhi.xC == 0 {
                    (*fp).mv.kb.specialhi.x8.i = (*fp).cmd_vars[0] as i32;
                    let new_var = 1;
                    (*fp).dmg.x1834 = (*dat_attr).speciallw_hp as f32;
                    let v = srem((*fp).u.kb.hat.x4.wrapping_add(ext::HSD_Randi(4)), 5).wrapping_add(new_var);
                    (*fp).mv.kb.specialhi.x10.i = v;
                    (*fp).u.kb.hat.x4 = v;
                    (*fp).mv.kb.specialhi.x14 = 0;
                    (*fp).mv.kb.specialhi.xC = 1;
                }
            }
            2 => {
                if (*fp).mv.kb.specialhi.x8.i == 0 && (*fp).mv.kb.specialhi.xC == 1 {
                    (*fp).mv.kb.specialhi.x8.i = (*fp).cmd_vars[0] as i32;
                    (*fp).mv.kb.specialhi.x14 = 0x16;
                }
            }
            _ => {}
        }
        (*fp).cmd_vars[0] = 0;
    }
}

pub fn ftKb_SpecialHi_800F3570(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi_800F3570");
    unsafe {
        let fp = fpp(gobj);
        let p = &ftKb_Init_803CB4EC;
        let dat_attr = da(fp);
        let mut sp1c: Vec3 = core::mem::zeroed();

        (*fp).mv.kb.specialhi.x18 = (*fp).coll_data.floor.normal;
        (*fp).mv.kb.specialhi.xC4 = (*fp).facing_dir
            * melee_runtime::lbtrigf::atan2f((*fp).mv.kb.specialhi.x18.x, (*fp).mv.kb.specialhi.x18.y);
        if 0.0 != (*fp).mv.kb.specialhi.x18.x {
            let mut pos = (*fp).cur_pos.y;
            let pos_x_alias = (*fp).cur_pos.x;
            let slide_speed = (*dat_attr).speciallw_slide_max_speed;
            let mut top_y = pos;
            top_y = top_y + slide_speed;
            let floor_normal_x_sign = if (*fp).mv.kb.specialhi.x18.x > 0.0 { 1.0f32 } else { -1.0f32 };
            let bottom_y = pos - slide_speed;
            pos = (*fp).cur_pos.x;
            let normal_y_slide = (*fp).mv.kb.specialhi.x18.y * slide_speed;
            if !ext::mpCheckAllRemap(
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut sp1c,
                -1,
                -1,
                pos,
                top_y,
                fmadds(normal_y_slide, floor_normal_x_sign, pos_x_alias),
                bottom_y,
            ) {
                sp1c = p.vec;
            }
            if (*fp).mv.kb.specialhi.x18.x > 0.0 {
                if sp1c.x < 0.0 {
                    (*fp).mv.kb.specialhi.x18 = p.vec;
                    (*fp).mv.kb.specialhi.xC4 = 0.0;
                    (*fp).xE4_ground_accel_1 = 0.0;
                    (*fp).gr_vel = 0.0;
                }
            } else if sp1c.x > 0.0 {
                (*fp).mv.kb.specialhi.x18 = p.vec;
                (*fp).mv.kb.specialhi.xC4 = 0.0;
                (*fp).xE4_ground_accel_1 = 0.0;
                (*fp).gr_vel = 0.0;
            }
        }
    }
}

pub fn ftKb_SpecialHi_800F36DC(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi_800F36DC");
    unsafe {
        let fp = fpp(gobj);
        let p = &ftKb_Init_803CB4EC;
        let sl = &mut (*fp).mv.kb.speciallw;
        for i in 0..4 {
            sl.x24[i] = p.vec;
            sl.x54[i] = sl.x24[i];
            sl.x88[i] = 0.0;
            sl.x88[i + 4] = 0.0;
        }
        sl.x18 = p.vec;
        sl.x84 = 0.0;
    }
}

pub fn ftKb_SpecialHi_800F37EC(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi_800F37EC");
    unsafe {
        let fp = fpp(gobj);
        let dat_attr = da(fp);
        let sl = &mut (*fp).mv.kb.speciallw;
        let mut i = 3usize;
        while i > 0 {
            sl.x24[i] = sl.x24[i - 1];
            sl.x54[i] = sl.x54[i - 1];
            sl.x88[i] = sl.x88[i - 1];
            sl.x88[i + 4] = sl.x88[i + 3];
            i -= 1;
        }
        sl.x24[0] = sl.x18;
        sl.x88[0] = sl.x84;
        sl.x54[0].z = 0.0;
        sl.x54[0].y = 0.0;
        sl.x54[0].x = 0.0;
        sl.x88[4] = 0.0;
        for k in 0..4 {
            sl.x54[0].x += sl.x24[k].x;
            sl.x54[0].y += sl.x24[k].y;
            sl.x54[0].z += sl.x24[k].z;
            sl.x88[4] += sl.x88[k];
        }
        // `lbVector_Normalize` (group 1's port works on melee_core's Vec3).
        let mut n = melee_core::vec::Vec3 { x: sl.x54[0].x, y: sl.x54[0].y, z: sl.x54[0].z };
        melee_runtime::lbvector::normalize(&mut n);
        sl.x54[0].x = n.x;
        sl.x54[0].y = n.y;
        sl.x54[0].z = n.z;
        sl.x88[4] *= 0.25;
        sl.x88[8] = 1.0;
        let mut i: i32 = 3;
        while i > 0 {
            if abs_c(sl.x88[i as usize]) > (*dat_attr).speciallw_min_slant_angle_slide {
                let mut j: i32 = i;
                while j >= 0 {
                    let slide_angle = sl.x88[j as usize];
                    let abs_slide_angle = if slide_angle < 0.0 { -slide_angle } else { slide_angle };
                    if abs_slide_angle <= (*dat_attr).speciallw_min_slant_angle_slide {
                        sl.x88[8] = 0.9;
                        j = -1;
                        i = -1;
                    } else if sl.x88[i as usize] > 0.0 {
                        if slide_angle < 0.0 {
                            sl.x88[8] -= 0.25;
                            if sl.x88[8] < 0.0 {
                                sl.x88[8] = 0.0;
                            }
                        }
                    } else if slide_angle > 0.0 {
                        sl.x88[8] -= 0.25;
                        if sl.x88[8] < 0.0 {
                            sl.x88[8] = 0.0;
                        }
                    }
                    j -= 1;
                }
            }
            i -= 1;
        }
    }
}

pub fn ftKb_SpecialHi_800F3B28(gobj: G) {
    melee_core::ct!("ftKb_SpecialHi_800F3B28");
    unsafe {
        let fp = fpp(gobj);
        let dat_attr = da(fp);
        let var_f5;
        let var_f4;

        let mut var_f31 = (*fp).mv.kb.speciallw.x88[4];
        ext::ftPartSetRotX(fp, 0, var_f31);
        if abs_c(var_f31) < (*dat_attr).speciallw_min_slant_angle_slide {
            var_f31 = 0.0;
        }
        if var_f31 > (*dat_attr).speciallw_max_slant_angle_slide {
            var_f31 = (*dat_attr).speciallw_max_slant_angle_slide;
        }
        if var_f31 < -(*dat_attr).speciallw_max_slant_angle_slide {
            var_f31 = -(*dat_attr).speciallw_max_slant_angle_slide;
        }
        let temp_f1 = ext::ft_GetGroundFrictionMultiplier(fp);
        if abs_c(var_f31) < 0.017453292 {
            var_f5 = 0.0f32;
            let temp_f3 = (*fp).gr_vel;
            let temp_f2 = temp_f1 * ((*fp).mv.co.common.x58.x * (*fp).co_attrs.ground_friction);
            let var_f1_2 = abs_c((*fp).gr_vel);
            if abs_c(temp_f2) > var_f1_2 {
                var_f4 = -(*fp).gr_vel;
            } else if temp_f3 > 0.0 {
                var_f4 = -temp_f2;
            } else {
                var_f4 = temp_f2;
            }
        } else {
            var_f4 = 0.0f32;
            var_f5 = (1.0 + (1.0 - temp_f1)) * ((*fp).mv.co.common.x4C.z * (*dat_attr).speciallw_slide_acceleration);
        }
        (*fp).xE4_ground_accel_1 = (*fp).mv.kb.speciallw.x88[8] * (var_f5 + var_f4);
        (*fp).gr_vel *= (*fp).mv.kb.speciallw.x88[8];
        if (*fp).gr_vel < -(*dat_attr).speciallw_slide_max_speed {
            (*fp).gr_vel = -(*dat_attr).speciallw_slide_max_speed;
        } else if (*fp).gr_vel > (*dat_attr).speciallw_slide_max_speed {
            (*fp).gr_vel = (*dat_attr).speciallw_slide_max_speed;
        }
        if (*fp).gr_vel + (*fp).xE4_ground_accel_1 < -(*dat_attr).speciallw_slide_max_speed {
            (*fp).xE4_ground_accel_1 = -(*dat_attr).speciallw_slide_max_speed - (*fp).gr_vel;
        } else if (*fp).gr_vel + (*fp).xE4_ground_accel_1 > (*dat_attr).speciallw_slide_max_speed {
            (*fp).xE4_ground_accel_1 = (*dat_attr).speciallw_slide_max_speed - (*fp).gr_vel;
        }
        (*fp).x74_self_accel.x = (*fp).mv.co.common.x58.x * (*fp).xE4_ground_accel_1;
        (*fp).x74_self_accel.y = -(*fp).mv.co.common.x4C.z * (*fp).xE4_ground_accel_1;
        (*fp).x74_self_accel.z = 0.0;
        (*fp).self_vel.x = (*fp).mv.co.common.x58.x * (*fp).gr_vel;
        (*fp).self_vel.y = -(*fp).mv.co.common.x4C.z * (*fp).gr_vel;
        (*fp).self_vel.z = 0.0;
    }
}

unsafe fn enter(gobj: G, msid: u32) {
    let fp = fpp(gobj);
    let dat_attr = da(fp);
    let temp_r28 = (*(*fp).parts.add(FtPart_YRotN as usize)).joint;
    st::Fighter_ClearCmdVars(&mut *fp);
    (*fp).mv.kb.speciallw.x0 = (*dat_attr).speciallw_max_time_in_stone as i16;
    (*fp).mv.kb.speciallw.x2 = (*dat_attr).speciallw_min_time_in_stone as i16;
    (*fp).mv.kb.speciallw.x4 = 0;
    (*fp).mv.kb.speciallw.x6 = 0;
    (*fp).mv.kb.speciallw.x8 = 0;
    (*fp).mv.kb.speciallw.xC = 0;
    (*fp).mv.kb.speciallw.x10 = -1;
    (*fp).mv.kb.speciallw.x14 = -1;
    ftKb_SpecialHi_800F36DC(gobj);
    (*fp).dmg.x1834 = 0.0;
    (*fp).set_x221C_b4(0);
    let temp_r30 = (*fp).x221C_b4();
    st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    (*fp).set_x221C_b4(temp_r30);
    set_cbs2(fp);
    st::ftAnim_8006EBA4(gobj);
    ext::efSync_Spawn(0x49B, gobj, &[temp_r28 as usize]);
}

pub fn ftKb_SpecialLw_Enter(gobj: G) {
    melee_core::ct!("ftKb_SpecialLw_Enter");
    unsafe { enter(gobj, ftKb_MS_SpecialLw1 as u32) }
}

pub fn ftKb_SpecialAirLw_Enter(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirLw_Enter");
    unsafe { enter(gobj, ftKb_MS_SpecialAirLwStart as u32) }
}

/// Shared by the two "start" Anim callbacks: `next_msid` is the looping Stone state.
unsafe fn start_anim(gobj: G, next_msid: u32) {
    let fp = fpp(gobj);
    (*fp).set_x221C_b4(0);
    ftKb_SpecialHi_800F346C(gobj);
    ftKb_SpecialHi_800F331C(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        let temp_r30 = (*fp).x221C_b4();
        st::Fighter_ChangeMotionState(gobj, next_msid as i32, 0x04440012, 0.0, 0.0, 0.0, core::ptr::null_mut());
        (*fp).set_x221C_b4(temp_r30);
        st::ftAnim_SetAnimRate(gobj, 0.0);
    }
}

pub fn ftKb_SpecialLw1_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialLw1_Anim");
    unsafe { start_anim(gobj, ftKb_MS_SpecialLw as u32) }
}

/// `ftKbUnkInline`.
unsafe fn unk_inline(gobj: G, val: u16) {
    let fp = fpp(gobj);
    (*fp).set_x221C_b4(val);
    ftKb_SpecialHi_800F346C(gobj);
    ftKb_SpecialHi_800F331C(gobj);
}

pub fn ftKb_SpecialLw_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialLw_Anim");
    unsafe { unk_inline(gobj, 1) }
}

unsafe fn end_anim(gobj: G) {
    let fp = fpp(gobj);
    let dat_attr = da(fp);
    unk_inline(gobj, 0);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        (*fp).dmg.x1834 = 0.0;
        (*fp).set_x221C_b4(0);
        if (*dat_attr).speciallw_freefall_toggle == 0.0 {
            st::ftCo_Fall_Enter(gobj);
            return;
        }
        st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*dat_attr).speciallw_freefall_toggle);
    }
}

pub fn ftKb_SpecialLwEnd_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialLwEnd_Anim");
    unsafe { end_anim(gobj) }
}

pub fn ftKb_SpecialAirLwStart_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirLwStart_Anim");
    unsafe { start_anim(gobj, ftKb_MS_SpecialAirLw as u32) }
}

pub fn ftKb_SpecialAirLw_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirLw_Anim");
    unsafe { unk_inline(gobj, 1) }
}

pub fn ftKb_SpecialAirLwEnd_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirLwEnd_Anim");
    unsafe { end_anim(gobj) }
}

pub fn ftKb_SpecialLw1_IASA(_gobj: G) { melee_core::ct!("ftKb_SpecialLw1_IASA");}

/// `fbKb_SpecialLw_IASA_Inline`.
unsafe fn iasa_inline(gobj: G) -> bool {
    let fp = fpp(gobj);
    let dat_attr = da(fp);
    if (*fp).mv.kb.speciallw.x0 <= 0 {
        true
    } else if (*fp).mv.kb.speciallw.x0 as i32
        <= (*dat_attr).speciallw_max_time_in_stone.wrapping_sub((*dat_attr).speciallw_min_time_in_stone) as i32
        && (*fp).input.pressed_buttons & 0x200 != 0
    {
        true
    } else {
        if (*fp).mv.kb.speciallw.x0 > 0 {
            (*fp).mv.kb.speciallw.x0 -= 1;
        }
        false
    }
}

pub fn ftKb_SpecialLw_IASA(gobj: G) {
    melee_core::ct!("ftKb_SpecialLw_IASA");
    unsafe {
        let fp = fpp(gobj);
        let temp_r30 = (*(*fp).parts.add(FtPart_YRotN as usize)).joint;
        if iasa_inline(gobj) {
            st::ftCommon_8007D5D4(&mut *fp);
            let temp_r28 = (*fp).x221C_b4();
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_SpecialLwEnd as i32, 0x04440012, 0.0, 1.0, 0.0, core::ptr::null_mut());
            (*fp).set_x221C_b4(temp_r28);
            set_cbs2(fp);
            ext::efSync_Spawn(0x49C, gobj, &[temp_r30 as usize]);
        }
    }
}

pub fn ftKb_SpecialLwEnd_IASA(_gobj: G) { melee_core::ct!("ftKb_SpecialLwEnd_IASA");}
pub fn ftKb_SpecialAirLwStart_IASA(_gobj: G) { melee_core::ct!("ftKb_SpecialAirLwStart_IASA");}

pub fn ftKb_SpecialAirLw_IASA(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirLw_IASA");
    unsafe {
        let fp = fpp(gobj);
        let temp_r31 = (*(*fp).parts.add(FtPart_YRotN as usize)).joint;
        if iasa_inline(gobj) {
            let temp_r28 = (*fp).x221C_b4();
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_SpecialAirLwEnd as i32, 0x04440012, 0.0, 1.0, 0.0, core::ptr::null_mut());
            (*fp).set_x221C_b4(temp_r28);
            set_cbs2(fp);
            ext::efSync_Spawn(0x49C, gobj, &[temp_r31 as usize]);
        }
    }
}

pub fn ftKb_SpecialAirLwEnd_IASA(_gobj: G) { melee_core::ct!("ftKb_SpecialAirLwEnd_IASA");}

pub fn ftKb_SpecialLw_Phys(gobj: G) {
    melee_core::ct!("ftKb_SpecialLw_Phys");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80084FA8(gobj);
        (*fp).self_vel.y = 0.0;
        ftKb_SpecialHi_800F3B28(gobj);
    }
}

pub fn ftKb_SpecialAirLw_Phys(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirLw_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        st::ft_80085134(gobj);
        (*fp).self_vel.y = -(*da).speciallw_gravity;
    }
}

pub fn ftKb_SpecialLw1_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialLw1_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            ftKb_SpecialHi_800F36DC(gobj);
            (*fp).self_vel.x = 0.0;
            st::ftCommon_8007D5D4(&mut *fp);
            let temp = (*fp).x221C_b4();
            let af = (*fp).cur_anim_frame;
            st::Fighter_ChangeMotionState(gobj, 0x18C, 0x0C4C5092, af, 1.0, 0.0, core::ptr::null_mut());
            (*fp).set_x221C_b4(temp);
            set_damage_cbs(fpp(gobj));
            ext::ftPartSetRotX(fp, 0, 0.0);
        } else {
            ftKb_SpecialHi_800F3570(gobj);
            ftKb_SpecialHi_800F37EC(gobj);
        }
    }
}

pub fn ftKb_SpecialLw_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialLw_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            ftKb_SpecialHi_800F36DC(gobj);
            (*fp).self_vel.x = 0.0;
            st::ftCommon_8007D5D4(&mut *fp);
            let temp = (*fp).x221C_b4();
            st::Fighter_ChangeMotionState(gobj, ftKb_MS_SpecialAirLw as i32, 0x0C4C1092, 0.0, 0.0, 0.0, core::ptr::null_mut());
            (*fp).set_x221C_b4(temp);
            st::ftAnim_SetAnimRate(gobj, 0.0);
            set_damage_cbs(fpp(gobj));
            ext::ftPartSetRotX(fp, 0, 0.0);
        } else {
            ftKb_SpecialHi_800F3570(gobj);
            ftKb_SpecialHi_800F37EC(gobj);
        }
    }
}

pub fn ftKb_SpecialLwEnd_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialLwEnd_Coll");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80081D0C(gobj);
        ftKb_SpecialHi_800F36DC(gobj);
        ext::ftPartSetRotX(fp, 0, 0.0);
    }
}

/// `ftKb_SpecialLw_SetStoneVecs` (static inline).
unsafe fn set_stone_vecs(gobj: G, fp: *mut Fighter, da: *const ftKb_DatAttrs) {
    let fp3 = fpp(gobj);
    let mut angle = (*fp).mv.kb.specialhi.xC4;
    let x18 = (*fp).mv.kb.speciallw.x18;
    {
        let sl = &mut (*fp3).mv.kb.speciallw;
        sl.x24[0].x = x18.x;
        sl.x54[0].x = x18.x;
        sl.x24[0].y = x18.y;
        sl.x54[0].y = x18.y;
        sl.x24[0].z = x18.z;
        sl.x54[0].z = x18.z;
        sl.x88[0] = angle;
        sl.x88[4] = angle;
        for i in 1..4 {
            sl.x24[i].x = x18.x;
            sl.x54[i].x = x18.x;
            sl.x24[i].y = x18.y;
            sl.x54[i].y = x18.y;
            sl.x24[i].z = x18.z;
            sl.x54[i].z = x18.z;
            sl.x88[i] = angle;
            sl.x88[i + 4] = angle;
        }
        sl.x18.x = x18.x;
        sl.x18.y = x18.y;
        sl.x18.z = x18.z;
    }
    (*fp3).mv.kb.specialhi.xC4 = angle;
    angle = (*fp).mv.kb.specialhi.xC4;
    if angle < 0.0 {
        angle = -angle;
    }
    if angle >= (*da).speciallw_min_slant_angle_slide {
        (*fp).gr_vel = (*fp).mv.co.common.x18 * (*da).speciallw_slide_max_speed;
    }
}

pub fn ftKb_SpecialAirLwStart_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirLwStart_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ft_80081D0C(gobj) == GA_Air as i32 {
            ftKb_SpecialHi_800F3570(gobj);
            ftKb_SpecialHi_800F37EC(gobj);
            (*fp).self_vel.y = 0.0;
            st::ftCommon_8007D7FC(&mut *fp);
            let temp = (*fp).x221C_b4();
            let af = (*fp).cur_anim_frame;
            st::Fighter_ChangeMotionState(gobj, 0x189, 0x0C4C5092, af, 1.0, 0.0, core::ptr::null_mut());
            (*fp).set_x221C_b4(temp);
            set_damage_cbs(fpp(gobj));
            ext::ft_PlaySFX(fp, 0x222E7, 0x7F, 0x40);
            ext::Camera_RequestQuake(QuakeKind_Large, &mut (*fp).cur_pos);
            st::ftCommon_8007EBAC(&mut *fp, 0xE, 0x14);
            set_stone_vecs(gobj, fp, da);
        } else {
            ftKb_SpecialHi_800F36DC(gobj);
            ext::ftPartSetRotX(fp, 0, 0.0);
        }
    }
}

pub fn ftKb_SpecialAirLw_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirLw_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ft_80081D0C(gobj) == GA_Air as i32 {
            ftKb_SpecialHi_800F3570(gobj);
            ftKb_SpecialHi_800F37EC(gobj);
            (*fp).self_vel.y = 0.0;
            st::ftCommon_8007D7FC(&mut *fp);
            let temp = (*fp).x221C_b4();
            st::Fighter_ChangeMotionState(gobj, 0x18A, 0x0C4C5092, 0.0, 0.0, 0.0, core::ptr::null_mut());
            (*fp).set_x221C_b4(temp);
            st::ftAnim_SetAnimRate(gobj, 0.0);
            set_damage_cbs(fpp(gobj));
            ext::ft_PlaySFX(fp, 0x222E7, 0x7F, 0x40);
            ext::Camera_RequestQuake(QuakeKind_Large, &mut (*fp).cur_pos);
            st::ftCommon_8007EBAC(&mut *fp, 0xE, 0x14);
            set_stone_vecs(gobj, fp, da);
        } else {
            ftKb_SpecialHi_800F36DC(gobj);
            ext::ftPartSetRotX(fp, 0, 0.0);
        }
    }
}

pub fn ftKb_SpecialAirLwEnd_Coll(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirLwEnd_Coll");
    unsafe {
        let fp = fpp(gobj);
        st::ft_80081D0C(gobj);
        ftKb_SpecialHi_800F36DC(gobj);
        ext::ftPartSetRotX(fp, 0, 0.0);
    }
}

pub fn ftKb_SpecialAirLw_800F5318(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirLw_800F5318");
    unsafe {
        let fp = fpp(gobj);
        ext::ftCommon_8007F5CC(gobj, 1);
        (*fp).set_x221E_b4(1);
        (*fp).set_x2225_b2(1);
        (*fp).dmg.x1834 = 0.0;
        (*fp).set_x221C_b4(0);
        ext::ftPartSetRotX(fp, 0, 0.0);
        ext::efLib_DestroyAll(gobj);
    }
}
