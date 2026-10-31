//! src/melee/ft/kinds/ftPeach/ftpeachspecials.c: Peach's side special (Peach Bomber).
//! Two fused sites, both in `ftPe_SpecialSStart_Anim` (tools/rust-port/fma_sites.py):
//! `cur_pos.x += -4 * facing_dir * scale.y` = fmadds(-4*facing_dir, scale.y, x) and
//! `cur_pos.y += 3.5f * scale.y` = fmadds(3.5, scale.y, y). Everything else is unfused.
//! `doPostEnd` and `doPostEnd_SmallerStack` are the same code (they differ only in the dead stack
//! pad the matching build needs). Smoke-tested by tests/tu_ftpeachspecials.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/3 and the original pass): re-exported under this TU.
pub use crate::chars::ftPeach::ftpeachspecials::{ftPe_SpecialAirSJump_Phys, ftPe_SpecialSStart_Phys};
pub use crate::chars::r2::ftPeach::ftpeachspecials::{
    enterAirEnd, enterEnd, enterStart, ftPe_SpecialAirSEnd_Anim, ftPe_SpecialAirSEnd_Phys,
    ftPe_SpecialAirSStart_Phys, ftPe_SpecialSEnd_Anim, ftPe_SpecialSEnd_Phys,
};
pub use crate::chars::r3::ftPeach::ftpeachspecials::{
    enterAirStart, ftPe_SpecialAirSEnd_Coll, ftPe_SpecialSEnd_Coll,
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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    pub static mut IT_802BD158: Option<fn(*mut HSD_GObj, *mut Vec3, i32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BD158(gobj: *mut HSD_GObj, pos: *mut Vec3, smash: i32, facing_dir: f32)  {
                match unsafe { IT_802BD158 } {
                    Some(f) => f(gobj, pos, smash, facing_dir),
                    None => { unsafe { melee_items::tu::itpeachexplode::it_802BD158(gobj as _, pos as _, smash as _, facing_dir as _); } return; },
                }
            }
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftPe_DatAttrs {
    (*fp).dat_attrs as *mut ftPe_DatAttrs
}

/// `Collide_CeilingMask` (mp/forward.h): CeilingPush | CeilingHug.
const Collide_CeilingMask: i32 = 0x2000 | 0x4000;

/// Facing into a wall: right wall while facing left, or left wall while facing right.
#[inline(always)]
unsafe fn wall_ahead(fp: *mut Fighter) -> bool {
    let env_flags = (*fp).coll_data.env_flags;
    (env_flags & Collide_RightWallMask != 0 && (*fp).facing_dir == -1.0)
        || (env_flags & Collide_LeftWallMask != 0 && (*fp).facing_dir == 1.0)
}

/// `start_mf` / `end_mf` (static const MotionFlags).
pub const start_mf: u32 = Ft_MF_KeepColAnimHitStatus
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27;
pub const end_mf: u32 = Ft_MF_SkipColAnim | Ft_MF_UpdateCmd;

/// Callback entries stored into Fighter (C function pointers).
pub unsafe extern "C" fn cb_reset(g: G) {
    melee_core::ct!("cb_reset");
    reset(g);
}
pub unsafe extern "C" fn cb_doAirEnd0(g: G) {
    melee_core::ct!("cb_doAirEnd0");
    doAirEnd0(g);
}
/// `ftCommon_8007F76C` / `ftCommon_8007F7B4` as Fighter callback entries.
pub unsafe extern "C" fn cb_ftCommon_8007F76C(g: G) {
    melee_core::ct!("cb_ftCommon_8007F76C");
    melee_fighter::ftcommon2::ftcommon_8007f76c(&mut *fpp(g));
}
pub unsafe extern "C" fn cb_ftCommon_8007F7B4(g: G) {
    melee_core::ct!("cb_ftCommon_8007F7B4");
    melee_fighter::ftcommon2::ftcommon_8007f7b4(&mut *fpp(g));
}

/// `reset` (static): the `x21EC` hook set by the Enter functions.
pub fn reset(gobj: G) {
    melee_core::ct!("reset");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        st::Fighter_ClearCmdVars(&mut *fp);
        (*fp).self_vel.x = 0.0;
        (*fp).x6A4_transNOffset.x = 0.0;
        if ((*fp).active_sticky.lstick.x as i32) < (*da).x30 {
            (*fp).x2070.anon_0.set_count_thrown_items(1);
            (*fp).mv.pe.specials.x0 = 1;
        } else {
            (*fp).mv.pe.specials.x0 = 0;
        }
    }
}

pub fn ftPe_SpecialS_Enter(gobj: G) {
    melee_core::ct!("ftPe_SpecialS_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).x21EC = Some(cb_reset);
        (*fp).self_vel.y = 0.0;
        (*fp).gr_vel = (*da).x34 * (*fp).facing_dir;
        st::Fighter_ChangeMotionState(gobj, ftPe_MS_SpecialSStart as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftPe_SpecialAirS_Enter(gobj: G) {
    melee_core::ct!("ftPe_SpecialAirS_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).x21EC = Some(cb_reset);
        (*fp).self_vel.y = (*da).x40;
        st::Fighter_ChangeMotionState(gobj, ftPe_MS_SpecialAirSStart as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `doAirEnd0` (static): the `hurtbox_detect_cb` of the air jump.
pub fn doAirEnd0(gobj: G) {
    melee_core::ct!("doAirEnd0");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x221C_b5() != 0 {
            // `GET_FIGHTER(NULL)` reads guest address 0: NULL.
            let other_fp: *mut Fighter =
                if (*fp).unk_gobj.is_null() { core::ptr::null_mut() } else { fpp((*fp).unk_gobj) };
            if !other_fp.is_null() {
                match (*other_fp).kind as u32 {
                    k if k == Ft_Kind_Mars as u32 || k == Ft_Kind_Emblem as u32 => {
                        let msid = (*other_fp).motion_id;
                        if msid != 369 && msid != 371 {
                            return;
                        }
                    }
                    k if k == Ft_Kind_Peach as u32 => {
                        let msid = (*other_fp).motion_id;
                        if msid != 365 && msid != 367 {
                            return;
                        }
                    }
                    _ => return,
                }
            }
        }
        (*fp).self_vel.x = 0.0;
        if (*fp).self_vel.y >= 0.0 {
            (*fp).self_vel.y = 0.0;
        }
        (*fp).cmd_vars[2] = 1;
        enterAirEndSmash(gobj);
    }
}

pub fn ftPe_SpecialSStart_Anim(gobj: G) {
    melee_core::ct!("ftPe_SpecialSStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) != 0 {
            return;
        }
        if (*fp).cmd_vars[0] != 0 {
            enterEndSmash(gobj);
        } else {
            st::ftCommon_8007D5D4(&mut *fp);
            (*fp).cur_pos.x = fmadds(-4.0 * (*fp).facing_dir, (*fp).x34_scale.y, (*fp).cur_pos.x);
            (*fp).cur_pos.y = fmadds(3.5, (*fp).x34_scale.y, (*fp).cur_pos.y);
            enterAirJump(gobj);
        }
    }
}

pub fn ftPe_SpecialAirSStart_Anim(gobj: G) {
    melee_core::ct!("ftPe_SpecialAirSStart_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).cmd_vars[0] != 0 {
                enterAirEndSmash(gobj);
            } else {
                enterAirJump(gobj);
            }
        }
    }
}

pub fn ftPe_SpecialSStart_IASA(_gobj: G) { melee_core::ct!("ftPe_SpecialSStart_IASA");}

pub fn ftPe_SpecialAirSStart_IASA(_gobj: G) { melee_core::ct!("ftPe_SpecialAirSStart_IASA");}

pub fn ftPe_SpecialSStart_Coll(gobj: G) {
    melee_core::ct!("ftPe_SpecialSStart_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_800827A0(gobj) == 0 {
            enterAirStart(gobj);
        }
        if wall_ahead(fp) {
            (*fp).gr_vel = 0.0;
            (*fp).cmd_vars[0] = 1;
        }
    }
}

pub fn ftPe_SpecialAirSStart_Coll(gobj: G) {
    melee_core::ct!("ftPe_SpecialAirSStart_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != 0 {
            enterStart(gobj);
        }
        if (*fp).coll_data.env_flags & Collide_CeilingMask != 0 {
            (*fp).self_vel.y = 0.0;
            (*fp).cmd_vars[0] = 1;
        }
        if wall_ahead(fp) {
            (*fp).self_vel.x = 0.0;
            (*fp).cmd_vars[0] = 1;
        }
    }
}

pub fn ftPe_SpecialAirSJump_Anim(gobj: G) {
    melee_core::ct!("ftPe_SpecialAirSJump_Anim");
    unsafe {
        if (*fpp(gobj)).cmd_vars[3] != 0 || st::ftAnim_IsFramesRemaining(gobj) == 0 {
            enterAirEndSmash(gobj);
        }
    }
}

pub fn ftPe_SpecialAirSJump_IASA(_gobj: G) { melee_core::ct!("ftPe_SpecialAirSJump_IASA");}

pub fn ftPe_SpecialAirSJump_Coll(gobj: G) {
    melee_core::ct!("ftPe_SpecialAirSJump_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != 0 {
            st::ftCommon_8007D7FC(&mut *fp);
            enterEndSmash(gobj);
        }
        if wall_ahead(fp) {
            (*fp).cmd_vars[2] = 1;
            enterAirEndSmash(gobj);
        }
    }
}

/// `enterAirJump` (static): launch into the jump arc.
pub fn enterAirJump(gobj: G) {
    melee_core::ct!("enterAirJump");
    unsafe {
        let mut fp = fpp(gobj);
        let da = da(fp);
        if (*fp).mv.pe.specials.x0 != 0 {
            (*fp).self_vel.x = (*da).specials_smash_vel_x * (*fp).facing_dir;
        } else {
            (*fp).self_vel.x = (*da).specials_vel_x * (*fp).facing_dir;
        }
        (*fp).self_vel.y = (*da).specials_vel_y;
        st::Fighter_ChangeMotionState(gobj, ftPe_MS_SpecialAirSJump as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        fp = fpp(gobj);
        (*fp).x21F8 = Some(cb_ftCommon_8007F76C);
        (*fp).hurtbox_detect_cb = Some(cb_doAirEnd0);
    }
}

pub fn ftPe_SpecialSEnd_IASA(_gobj: G) { melee_core::ct!("ftPe_SpecialSEnd_IASA");}

pub fn ftPe_SpecialAirSEnd_IASA(_gobj: G) { melee_core::ct!("ftPe_SpecialAirSEnd_IASA");}

/// `doPostEnd` (static): the bomber's impact burst when the script flagged it.
pub fn doPostEnd(gobj: G) {
    melee_core::ct!("doPostEnd");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[2] != 0 {
            let mut vec: Vec3 = core::mem::zeroed();
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_HipN as usize)).joint, core::ptr::null(), &mut vec);
            vec.z = 0.0;
            let facing_dir = (*fp).facing_dir;
            ext::it_802BD158(gobj, &mut vec, (*fp).mv.pe.specials.x0, facing_dir);
            (*fp).self_vel.x = (*da).specials_end_vel_x * (*fp).facing_dir;
            (*fp).self_vel.y = (*da).specials_end_vel_y;
        }
        (*fp).x21F8 = Some(cb_ftCommon_8007F7B4);
    }
}

/// `doPostEnd_SmallerStack` (static): identical to [`doPostEnd`].
pub fn doPostEnd_SmallerStack(gobj: G) {
    melee_core::ct!("doPostEnd_SmallerStack");
    doPostEnd(gobj);
}

/// `enterEndSmash` (static).
pub fn enterEndSmash(gobj: G) {
    melee_core::ct!("enterEndSmash");
    st::Fighter_ChangeMotionState(gobj, ftPe_MS_SpecialSEnd as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    doPostEnd(gobj);
}

pub fn enterAirEndSmash(gobj: G) {
    melee_core::ct!("enterAirEndSmash");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).self_vel.x /= (*da).x68;
        (*fp).self_vel.y /= (*da).x6C;
        if (*fp).cmd_vars[2] != 0 {
            st::Fighter_ChangeMotionState(gobj, ftPe_MS_SpecialAirSEnd_1 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        } else {
            st::Fighter_ChangeMotionState(gobj, ftPe_MS_SpecialAirSEnd_0 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        }
        doPostEnd_SmallerStack(gobj);
    }
}
