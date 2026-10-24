//! src/melee/ft/kinds/ftKirby/ftkirbyspecialkoopa.c: Kirby's Bowser copy ability (fire breath).
//! Two fused sites in ftKb_SpecialNKp_800FA588 (`pos.x += scale.y * (x_off * facing)` and
//! `pos.y += y_off * scale.y`, both fmadds; tools/rust-port/fma_sites.py). Smoke-tested by
//! tests/tu_ftkirbyspecialkoopa.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_runtime::cabi::srem;
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyspecialkoopa::{
    ftKb_KpSpecialAirNEnd_Anim, ftKb_KpSpecialAirNEnd_Phys, ftKb_KpSpecialAirNStart_Phys,
    ftKb_KpSpecialAirN_Phys, ftKb_KpSpecialNEnd_Anim, ftKb_KpSpecialNEnd_Phys, ftKb_KpSpecialNStart_Phys,
    ftKb_KpSpecialN_Phys,
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
    #[allow(unused_macros)]
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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    crate::port_hook!(HSD_RANDI, HSD_Randi, crate::adapt::HSD_Randi, (max: i32) -> i32);
    pub static mut ITEM_8026AE60: Option<fn() -> u32> = None;
            #[allow(non_snake_case)]
            pub fn Item_8026AE60() -> u32 {
                match unsafe { ITEM_8026AE60 } {
                    Some(f) => f(),
                    None => { return unsafe { melee_items::tu::item::Item_8026AE60() as _ }; },
                }
            }
    pub static mut ITKOOPAFLAME_SPAWN: Option<fn(*mut HSD_GObj, *mut Vec3, f32, u32, i32, i32, i32, i32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn itKoopaFlame_Spawn(parent: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32, unk: u32, gfx: i32, base_speed: i32, scale: i32, kind: i32) -> *mut HSD_GObj {
                match unsafe { ITKOOPAFLAME_SPAWN } {
                    Some(f) => f(parent, pos, facing_dir, unk, gfx, base_speed, scale, kind),
                    None => { return unsafe { melee_items::tu::itkoopaflame::itKoopaFlame_Spawn(parent as _, pos as _, facing_dir as _, unk as _, gfx as _, base_speed as _, scale as _, kind as _) as _ }; },
                }
            }
    hook!(FT_PLAYSFX, ft_PlaySFX, (fp: *mut Fighter, sfx: i32, vol: u8, pan: u8));
    hook_or!(FT_80089824, ft_80089824, melee_fighter::ft0892::ft_80089824, (gobj: *mut HSD_GObj));
    hook!(FT_800892A0, ft_800892A0, (gobj: *mut HSD_GObj));
    hook!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, (kind: CmQuakeKind, pos: *mut Vec3));
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

/// `ftKb_Init_803CB540`: the flame direction table (96 entries).
pub static ftKb_Init_803CB540: [i32; 96] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
    0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2,
    2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
];

/// The Kirby-Bowser state: Bowser's id, or Giga Bowser's when the hat is Giga Bowser.
#[inline(always)]
unsafe fn ms_for(fp: *mut Fighter, kp: u32, gk: u32) -> i32 {
    if (*fp).u.kb.hat.kind as u32 == Ft_Kind_GKoops as u32 {
        gk as i32
    } else {
        kp as i32
    }
}

pub fn ftKb_SpecialNKp_800FA588(gobj: G) {
    unsafe {
        let mut pos: Vec3 = core::mem::zeroed();
        let mut offset: Vec3 = core::mem::zeroed();
        let fp = fpp(gobj);
        let da = da(fp);
        let dirs = &ftKb_Init_803CB540;
        offset.x = 0.0;
        offset.y = 0.5;
        offset.z = 3.0;
        ext::lb_8000B1CC((*(*fp).parts.add(12)).joint, &offset, &mut pos);
        pos.x = fmadds((*fp).x34_scale.y, (*da).specialn_kp_breath_x_offset * (*fp).facing_dir, pos.x);
        pos.y = fmadds((*da).specialn_kp_breath_y_offset, (*fp).x34_scale.y, pos.y);

        let new_facing;
        {
            let fp2 = fpp(gobj);
            match (*fp2).mv.kb.specialn_kp.x0[2] {
                1 | 2 => {
                    new_facing = dirs[(ext::HSD_Randi(0x20) + 0x20) as usize];
                }
                _ => {
                    if dirs[ext::HSD_Randi(0x20) as usize] == 0 {
                        new_facing = dirs[(ext::HSD_Randi(0x20) + 0x20) as usize];
                    } else {
                        new_facing = dirs[(ext::HSD_Randi(0x20) + 0x40) as usize];
                    }
                }
            }
            (*fp2).mv.kb.specialn_kp.x0[2] = new_facing;
        }

        ext::itKoopaFlame_Spawn(
            gobj,
            &mut pos,
            (*fp).facing_dir,
            (*fp).mv.kb.specialn_kp.x0[1] as u32,
            new_facing,
            (*fp).u.kb.x84 as i32,
            (*fp).u.kb.x88 as i32,
            It_Kind_Kirby_KoopaFlame as i32,
        );

        if (*fp).mv.kb.specialn_kp.x14 == 0 {
            (*fp).mv.kb.specialn_kp.x0[1] = ext::Item_8026AE60() as i32;
            ext::ft_80089824(gobj);
            ext::ft_800892A0(gobj);
        }

        if srem((*fp).mv.kb.specialn_kp.x14, 3) == 0 {
            let f = ((*fp).u.kb.x88 - (*da).specialn_kp_lowest_charge_graphic_size)
                / ((*da).specialn_kp_flame_scale - (*da).specialn_kp_lowest_charge_graphic_size);
            if f < 0.3333 {
                ext::ft_PlaySFX(fp, 0x24A2B, 0x7F, 0x40);
            } else if f < 0.6666 {
                ext::ft_PlaySFX(fp, 0x24A28, 0x7F, 0x40);
            } else {
                ext::ft_PlaySFX(fp, 0x24A25, 0x7F, 0x40);
            }
        }

        (*fp).mv.kb.specialn_kp.x14 += 1;
        (*fp).mv.kb.specialn_kp.x14 = srem((*fp).mv.kb.specialn_kp.x14, 12);
    }
}

pub fn ftKb_SpecialNKp_800FA7D4(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).motion_id >= 449 || (*fp).motion_id < 443 {
            (*fp).u.kb.x84 = (*fp).u.kb.x84 + (*da).specialn_kp_fuel_recharge_rate;
            if (*fp).u.kb.x84 > (*da).specialn_kp_max_fuel {
                (*fp).u.kb.x84 = (*da).specialn_kp_max_fuel;
            }
            (*fp).u.kb.x88 = (*fp).u.kb.x88 + (*da).specialn_kp_flame_size_recharge_rate;
            if (*fp).u.kb.x88 > (*da).specialn_kp_flame_scale {
                (*fp).u.kb.x88 = (*da).specialn_kp_flame_scale;
            }
        }
    }
}

pub fn ftKb_SpecialNKp_800FA83C(gobj: G) -> i32 {
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                return (*da(fp)).specialn_kp_max_fuel as i32;
            }
        }
        1
    }
}

pub fn ftKb_SpecialNKp_800FA878(gobj: G) -> i32 {
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                return (*da(fp)).specialn_kp_flame_scale as i32;
            }
        }
        2
    }
}

unsafe fn start_vars(gobj: G, air: bool) {
    let fp = fpp(gobj);
    (*fp).mv.kb.specialn_kp.x0[0] = 0;
    (*fp).mv.kb.specialn_kp.x0[1] = ext::Item_8026AE60() as i32;
    (*fp).mv.kb.specialn_kp.x0[2] = -1;
    (*fp).mv.kb.specialn_kp.x10 = 1;
    (*fp).mv.kb.specialn_kp.x14 = 0;
    (*fp).mv.kb.specialn_kp.x18 = 0;
    (*fp).mv.kb.specialn_kp.x0[3] = 0;
    let _ = air;
}

pub fn ftKb_SpecialNKp_800FA8B4(gobj: G) {
    unsafe {
        let msid = ms_for(fpp(gobj), ftKb_MS_KpSpecialNStart as u32, ftKb_MS_GkSpecialNStart as u32);
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        start_vars(gobj, false);
    }
}

pub fn ftKb_SpecialNKp_800FA958(gobj: G) {
    unsafe {
        let msid = ms_for(fpp(gobj), ftKb_MS_KpSpecialAirNStart as u32, ftKb_MS_GkSpecialAirNStart as u32);
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        start_vars(gobj, true);
    }
}

pub fn ftKb_KpSpecialNStart_Anim(gobj: G) {
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let msid = ms_for(fpp(gobj), ftKb_MS_KpSpecialN as u32, ftKb_MS_GkSpecialN as u32);
            st::Fighter_ChangeMotionState(gobj, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        }
    }
}

/// Shared by KpSpecialN_Anim / KpSpecialAirN_Anim (identical bodies).
unsafe fn loop_anim(gobj: G) {
    {
        let fp = fpp(gobj);
        if (*fp).cur_anim_frame == 0.0 {
            (*fp).mv.kb.specialn_kp.x10 -= 1;
            if (*fp).mv.kb.specialn_kp.x10 <= 0 {
                (*fp).mv.kb.specialn_kp.x10 = 0;
            }
        }
    }
    {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).mv.kb.specialn_kp.x18 == 0 {
            ext::Camera_RequestQuake(QuakeKind_Small, &mut (*fp).cur_pos);
        }
        (*fp).mv.kb.specialn_kp.x18 += 1;
        (*fp).mv.kb.specialn_kp.x18 = srem((*fp).mv.kb.specialn_kp.x18, (*da).specialn_kp_screen_shake_frequency);
    }
}

pub fn ftKb_KpSpecialN_Anim(gobj: G) {
    unsafe { loop_anim(gobj) }
}

pub fn ftKb_KpSpecialAirNStart_Anim(gobj: G) {
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let msid = ms_for(fpp(gobj), ftKb_MS_KpSpecialAirN as u32, ftKb_MS_GkSpecialAirN as u32);
            st::Fighter_ChangeMotionState(gobj, msid, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        }
    }
}

pub fn ftKb_KpSpecialAirN_Anim(gobj: G) {
    unsafe { loop_anim(gobj) }
}

pub fn ftKb_KpSpecialNStart_IASA(_gobj: G) {}

/// Shared body of KpSpecialN_IASA / KpSpecialAirN_IASA; `end_kp`/`end_gk` are the end states.
unsafe fn flame_iasa(gobj: G, end_kp: u32, end_gk: u32) {
    let fp = fpp(gobj);
    let da = da(fp);
    if (*fp).mv.kb.specialn_kp.x0[3] >= (*da).specialn_kp_b_button_check_frequency as i32 {
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) != 0 {
            if (*fp).mv.kb.specialn_kp.x0[0] == 0 {
                ftKb_SpecialNKp_800FA588(gobj);
            }
        } else if (*fp).mv.kb.specialn_kp.x10 != 0 {
            if (*fp).mv.kb.specialn_kp.x0[0] == 0 {
                ftKb_SpecialNKp_800FA588(gobj);
            }
        } else {
            let msid = ms_for(fp, end_kp, end_gk);
            st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        }
    } else if (*fp).mv.kb.specialn_kp.x0[0] == 0 {
        ftKb_SpecialNKp_800FA588(gobj);
    }
    (*fp).mv.kb.specialn_kp.x0[0] += 1;
    if (*fp).mv.kb.specialn_kp.x0[0] >= 3 {
        (*fp).mv.kb.specialn_kp.x0[0] = 0;
    }
    (*fp).u.kb.x84 -= 1.0;
    if (*fp).u.kb.x84 < (*da).specialn_kp_spew_flame_velocity {
        (*fp).u.kb.x84 = (*da).specialn_kp_spew_flame_velocity;
    }
    (*fp).u.kb.x88 -= 1.0;
    if (*fp).u.kb.x88 < (*da).specialn_kp_lowest_charge_graphic_size {
        (*fp).u.kb.x88 = (*da).specialn_kp_lowest_charge_graphic_size;
    }
    (*fp).mv.kb.specialn_kp.x0[3] += 1;
    if (*fp).mv.kb.specialn_kp.x0[3] > (*da).specialn_kp_b_button_check_frequency as i32 {
        (*fp).mv.kb.specialn_kp.x0[3] = (*da).specialn_kp_b_button_check_frequency as i32;
    }
}

pub fn ftKb_KpSpecialN_IASA(gobj: G) {
    unsafe { flame_iasa(gobj, ftKb_MS_KpSpecialNEnd as u32, ftKb_MS_GkSpecialNEnd as u32) }
}

pub fn ftKb_KpSpecialNEnd_IASA(_gobj: G) {}
pub fn ftKb_KpSpecialAirNStart_IASA(_gobj: G) {}

pub fn ftKb_KpSpecialAirN_IASA(gobj: G) {
    unsafe { flame_iasa(gobj, ftKb_MS_KpSpecialAirNEnd as u32, ftKb_MS_GkSpecialAirNEnd as u32) }
}

pub fn ftKb_KpSpecialAirNEnd_IASA(_gobj: G) {}

/// Ground -> air Coll body: `kp`/`gk` are the air state ids, `flags` the motion flags.
unsafe fn coll_to_air(gobj: G, kp: u32, gk: u32, flags: u32) {
    let fp = fpp(gobj);
    if st::ft_80082708(gobj) == GA_Ground as i32 {
        st::ftCommon_8007D5D4(&mut *fp);
        let anim_frame = (*fp).cur_anim_frame;
        let msid = ms_for(fpp(gobj), kp, gk);
        st::Fighter_ChangeMotionState(gobj, msid, flags, anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

/// Air -> ground Coll body.
unsafe fn coll_to_ground(gobj: G, kp: u32, gk: u32, flags: u32) {
    let fp = fpp(gobj);
    if st::ft_80081D0C(gobj) != GA_Ground as i32 {
        st::ftCommon_8007D7FC(&mut *fp);
        let anim_frame = (*fp).cur_anim_frame;
        let msid = ms_for(fpp(gobj), kp, gk);
        st::Fighter_ChangeMotionState(gobj, msid, flags, anim_frame, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftKb_KpSpecialNStart_Coll(gobj: G) {
    unsafe { coll_to_air(gobj, ftKb_MS_KpSpecialAirNStart as u32, ftKb_MS_GkSpecialAirNStart as u32, 0x0C4C5080) }
}
pub fn ftKb_KpSpecialN_Coll(gobj: G) {
    unsafe { coll_to_air(gobj, ftKb_MS_KpSpecialAirN as u32, ftKb_MS_GkSpecialAirN as u32, 0x0C4C5880) }
}
pub fn ftKb_KpSpecialNEnd_Coll(gobj: G) {
    unsafe { coll_to_air(gobj, ftKb_MS_KpSpecialAirNEnd as u32, ftKb_MS_GkSpecialAirNEnd as u32, 0x0C4C5080) }
}
pub fn ftKb_KpSpecialAirNStart_Coll(gobj: G) {
    unsafe { coll_to_ground(gobj, ftKb_MS_KpSpecialN as u32, ftKb_MS_GkSpecialN as u32, 0x0C4C5080) }
}
pub fn ftKb_KpSpecialAirN_Coll(gobj: G) {
    unsafe { coll_to_ground(gobj, ftKb_MS_KpSpecialN as u32, ftKb_MS_GkSpecialN as u32, 0x0C4C5880) }
}
pub fn ftKb_KpSpecialAirNEnd_Coll(gobj: G) {
    unsafe { coll_to_ground(gobj, ftKb_MS_KpSpecialNEnd as u32, ftKb_MS_GkSpecialNEnd as u32, 0x0C4C5080) }
}
