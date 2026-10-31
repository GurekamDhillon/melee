//! src/melee/ft/kinds/ftNess/ftnessspecialhi.c: Ness up B (PK Thunder / PK Thunder 2).
//!
//! Fused sites (tools/rust-port/fma_sites.py), all f32 [`fmadds`]: `pos.y += 5.0f * scale.y`
//! (CheckNessCollide, both cases), `(5.0f * scale.y) + cur_pos.y` (SpecialHi_Enter,
//! NessFloatMath_PKThunder2), `facing * atan2f(..) - (float) M_PI_2` (the model tilt of
//! SpecialHi_Enter / AirHi_Enter / AirHiHold_Anim / SpecialHi_Phys / ftNess_atan2) and
//! `-(decel * facing - gr_vel)` (SpecialHi_Phys). The `lbVector_Len_xy` / `sqrtf` sites live in
//! melee_runtime. Smoke-tested through recording hooks by tests/tu_ftnessspecialhi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::chars::{abs_c, caps_max_jumps};
use crate::stubs as st;
use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf};
use melee_runtime::lbtrigf::atan2f;
use melee_runtime::lbvector;
use melee_types::*;

/// Already ported (rounds 2 and 12): re-exported under this TU.
pub use crate::chars::r12::ftNess::ftnessspecialhi::{
    ftNs_SpecialAirHiEnd_IASA, ftNs_SpecialAirHiHold_IASA, ftNs_SpecialAirHiRebound_IASA,
    ftNs_SpecialAirHiStart_IASA, ftNs_SpecialAirHi_IASA, ftNs_SpecialHiEnd_IASA,
    ftNs_SpecialHiHold_IASA, ftNs_SpecialHi_IASA,
};
pub use crate::chars::r2::ftNess::ftnessspecialhi::{ftNs_SpecialHiEnd_Anim, ftNs_SpecialHiEnd_Phys};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    pub static mut FTPARTSETROTX: Option<fn(*mut Fighter, i32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn ftPartSetRotX(fp: *mut Fighter, part: i32, v: f32) {
        match unsafe { FTPARTSETROTX } {
            Some(f) => f(fp, part, v),
            None => { unsafe { let _ = melee_fighter::tu::ftparts::ftPartSetRotX(&*(fp as *const _), part as _, v as _); } },
        }
    }
    pub static mut EFLIB_DESTROYALL: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn efLib_DestroyAll(gobj: *mut HSD_GObj) {
        match unsafe { EFLIB_DESTROYALL } {
            Some(f) => f(gobj),
            None => { unsafe { let _ = melee_world::tu::efn::eflib::efLib_DestroyAll(gobj as _); } },
        }
    }
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing pointers as a slice.
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    pub static mut IT_802AB3F0: Option<fn(*mut HSD_GObj, *mut Vec3, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AB3F0(item: *mut HSD_GObj, out: *mut Vec3, which: i32)  {
                match unsafe { IT_802AB3F0 } {
                    Some(f) => f(item, out, which),
                    None => { unsafe { let _ = melee_items::tu::itnesspkthunderball::it_802AB3F0(item as _, out as _, which as _); } return; },
                }
            }
    pub static mut IT_802AB9C0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AB9C0(item: *mut HSD_GObj)  {
                match unsafe { IT_802AB9C0 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itnesspkthunderball::it_802AB9C0(item as _); } return; },
                }
            }
    pub static mut IT_802AB58C: Option<fn(*mut HSD_GObj, *mut Vec3, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802AB58C(owner: *mut HSD_GObj, pos: *mut Vec3, facing: f32) -> *mut HSD_GObj {
                match unsafe { IT_802AB58C } {
                    Some(f) => f(owner, pos, facing),
                    None => { return unsafe { melee_items::tu::itnesspkthunderball::it_802AB58C(owner as _, pos as _, facing as _) as _ }; },
                }
            }
    pub static mut IT_802AB568: Option<fn(*mut HSD_GObj) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802AB568(item: *mut HSD_GObj) -> *mut HSD_GObj {
                match unsafe { IT_802AB568 } {
                    Some(f) => f(item),
                    None => { return unsafe { melee_items::tu::itnesspkthunderball::it_802AB568(item as _) as _ }; },
                }
            }
    pub static mut FTCO_80097D40: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_80097D40(gobj: *mut HSD_GObj)  {
                match unsafe { FTCO_80097D40 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = melee_fighter::tu::ftCo_DownBound::ftCo_80097D40(gobj as _); } return; },
                }
            }
    pub static mut FTCO_800969D8: Option<fn(*mut HSD_GObj, i32, i32, i32, f32, f32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_800969D8(gobj: *mut HSD_GObj, a: i32, b: i32, allow_interrupt: i32, mobility: f32, landing_lag: f32, blend: f32)  {
                match unsafe { FTCO_800969D8 } {
                    Some(f) => f(gobj, a, b, allow_interrupt, mobility, landing_lag, blend),
                    None => { unsafe { let _ = melee_fighter::tu::ftCo_FallSpecial::ftCo_800969D8(gobj as _, a as _, b as _, allow_interrupt as _, mobility as _, landing_lag as _, blend as _); } return; },
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
}

/// `FTNESS_SPECIALHI_COLL_FLAG`.
pub const FTNESS_SPECIALHI_COLL_FLAG: u32 = Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27;

/// `FTNESS_JIBAKU_COLL_FLAG` (PK Thunder 2 self-hit).
pub const FTNESS_JIBAKU_COLL_FLAG: u32 = Ft_MF_KeepGfx
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27;

const Collide_CeilingMask: i32 = Collide_CeilingPush | Collide_CeilingHug;
/// `static const float vel_epsilon = 0.0001f`.
const vel_epsilon: f32 = 0.0001;
/// `(float) M_PI_2`.
const M_PI_2_F: f32 = core::f64::consts::FRAC_PI_2 as f32;
const M_TAU: f64 = core::f64::consts::TAU;
const M_PI: f64 = core::f64::consts::PI;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn attrs(fp: *mut Fighter) -> *const ftNessAttributes {
    (*fp).dat_attrs as *const ftNessAttributes
}

#[inline(always)]
fn null() -> *mut HSD_GObj {
    core::ptr::null_mut()
}

#[inline(always)]
unsafe fn joint(fp: *mut Fighter, part: usize) -> *mut HSD_JObj {
    (*(*fp).parts.add(part)).joint as *mut HSD_JObj
}

/// The shared `switch (motion_id) { case SpecialHiStart..SpecialAirHiRebound: ... }` test: the
/// nine PK Thunder motions (358..=366).
#[inline(always)]
pub fn in_special_hi(msid: i32) -> bool {
    melee_core::ct!("in_special_hi");
    msid >= ftNs_MS_SpecialHiStart as i32 && msid <= ftNs_MS_SpecialAirHiRebound as i32
}

/// The switch body shared by many functions: in a PK Thunder motion, destroy the thunder graphics.
unsafe fn destroy_gfx_if_special_hi(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    if in_special_hi((*fp).motion_id as i32) {
        ext::efLib_DestroyAll(gobj);
        (*fp).u.ns.pkthunder_gfx = 0;
    }
}

/// C-ABI entry for the damage/death callbacks (`ftNs_Init_OnDamage`).
unsafe fn on_damage_cb() -> HSD_GObjEvent {
    Some(crate::tu::ftnessattackhi4::cb_ftNs_Init_OnDamage)
}

/// `ftModelPartRotX(fp, 0, (facing * atan2f(vx, vy)) - (float) M_PI_2)`, with the fused subtract.
#[inline(always)]
pub fn tilt_rot_x(facing: f32, vx: f32, vy: f32) -> f32 {
    melee_core::ct!("tilt_rot_x");
    fmadds(facing, atan2f(vx, vy), -M_PI_2_F)
}

unsafe fn set_tilt(fp: *mut Fighter) {
    let v = tilt_rot_x((*fp).facing_dir, (*fp).self_vel.x, (*fp).self_vel.y);
    ext::ftPartSetRotX(fp, 0, v);
}

/// Removes GFX (`ftNs_SpecialHiStopGFX`).
pub fn ftNs_SpecialHiStopGFX(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHiStopGFX");
    unsafe {
        let fp = fpp(gobj);
        destroy_gfx_if_special_hi(gobj, fp);
    }
}

/// `check_distance` (static inline).
#[inline(always)]
pub fn check_distance(pos: &Vec3, pair: &Vec3) -> bool {
    melee_core::ct!("check_distance");
    abs_c(pos.x - pair.x) < 8.333333015441895_f32 && abs_c(pos.y - pair.y) < 12.333333015441895_f32
}

/// C `bool`: PK Thunder is touching Ness.
pub fn ftNs_SpecialHi_ItemPKThunder_CheckNessCollide(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftNs_SpecialHi_ItemPKThunder_CheckNessCollide");
    unsafe {
        let fp = fpp(gobj);
        let mut ret = false;
        if (*fp).u.ns.pkthunder_gobj.is_null() {
            return false;
        }
        match (*fp).mv.ns.specialhi.thunderColl {
            0 => {
                let mut pos = (*fp).cur_pos;
                pos.y = fmadds(5.0, (*fp).x34_scale.y, pos.y);
                let mut pair = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                ext::it_802AB3F0((*fp).u.ns.pkthunder_gobj, &mut pair, 0);
                if check_distance(&pos, &pair) {
                    (*fp).mv.ns.specialhi.thunderColl = 2;
                    ext::it_802AB3F0((*fp).u.ns.pkthunder_gobj, &mut (*fp).mv.ns.specialhi.collPos1, 0);
                    ext::it_802AB3F0((*fp).u.ns.pkthunder_gobj, &mut (*fp).mv.ns.specialhi.collPos2, 1);
                    ret = true;
                }
            }
            1 => {
                let mut pos2 = (*fp).cur_pos;
                pos2.y = fmadds(5.0, (*fp).x34_scale.y, pos2.y);
                let mut pair2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                ext::it_802AB3F0((*fp).u.ns.pkthunder_gobj, &mut pair2, 0);
                if !check_distance(&pos2, &pair2) {
                    (*fp).mv.ns.specialhi.thunderColl = 0;
                }
            }
            _ => {}
        }
        ret
    }
}

/// C `bool`: Ness is in SpecialHiHold / SpecialAirHiHold (the PK Thunder control loop).
pub fn ftNs_SpecialHi_CheckSpecialHiHold(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftNs_SpecialHi_CheckSpecialHiHold");
    unsafe {
        let fp = fpp(gobj);
        (*fp).motion_id as i32 == ftNs_MS_SpecialHiHold as i32 || (*fp).motion_id as i32 == ftNs_MS_SpecialAirHiHold as i32
    }
}

pub fn ftNs_SpecialHi_ItemPKThunderRemove(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHi_ItemPKThunderRemove");
    unsafe {
        let temp_fp = fpp(gobj);
        if !(*temp_fp).u.ns.pkthunder_gobj.is_null() {
            (*temp_fp).u.ns.pkthunder_gobj = null();
        }
        destroy_gfx_if_special_hi(gobj, temp_fp);
        (*temp_fp).death2_cb = None;
        (*temp_fp).take_dmg_cb = None;
        ext::ftPartSetRotX(temp_fp, 0, 0.0);
    }
}

pub fn ftNs_SpecialHi_TakeDamage(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHi_TakeDamage");
    unsafe {
        let temp_fp = fpp(gobj);
        if !(*temp_fp).u.ns.pkthunder_gobj.is_null() {
            ext::it_802AB9C0((*temp_fp).u.ns.pkthunder_gobj);
            (*temp_fp).u.ns.pkthunder_gobj = null();
        }
        destroy_gfx_if_special_hi(gobj, temp_fp);
        (*temp_fp).death2_cb = None;
        (*temp_fp).take_dmg_cb = None;
        ext::ftPartSetRotX(temp_fp, 0, 0.0);
    }
}

/// Run from PK Thunder's OnReflect callback: forget the thunder if Ness reflects his own.
pub fn ftNs_SpecialHi_ItemPKThunderCheckOwn(gobj: *mut HSD_GObj, thunder_gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHi_ItemPKThunderCheckOwn");
    unsafe {
        let temp_fp = fpp(gobj);
        let temp_thunder = (*temp_fp).u.ns.pkthunder_gobj;
        if !temp_thunder.is_null() && temp_thunder == thunder_gobj {
            (*temp_fp).u.ns.pkthunder_gobj = null();
        }
    }
}

/// Wrap `angle` (f32 holder, double math as in the C) into [0, TAU].
#[inline(always)]
fn wrap_tau(mut v: f32) -> f32 {
    while v < 0.0 {
        v = (v as f64 + M_TAU) as f32;
    }
    while v as f64 > M_TAU {
        v = (v as f64 - M_TAU) as f32;
    }
    v
}

/// `ftNs_SpecialAirHi_CollisionModVel` (static): bend the flight direction along a wall.
pub fn ftNs_SpecialAirHi_CollisionModVel(gobj: *mut HSD_GObj, coll_data: *mut CollData) {
    melee_core::ct!("ftNs_SpecialAirHi_CollisionModVel");
    unsafe {
        let fp = fpp(gobj);
        // `phi_f1` is uninitialised when no wall flag is set: zero under -ftrivial-auto-var-init.
        let mut phi_f1: f32 = 0.0;
        (*fp).mv.ns.specialhi.aerialVel = wrap_tau((*fp).mv.ns.specialhi.aerialVel);
        if (*coll_data).env_flags & Collide_LeftWallMask != 0 {
            phi_f1 = atan2f((*coll_data).left_facing_wall.normal.y, (*coll_data).left_facing_wall.normal.x);
            phi_f1 = wrap_tau(phi_f1);
            let phi_f3 = wrap_tau((M_PI + (*fp).mv.ns.specialhi.aerialVel as f64) as f32);
            if phi_f3 - phi_f1 < 0.0 {
                phi_f1 = (phi_f1 as f64 + M_PI / 2.0) as f32;
            } else {
                phi_f1 = (phi_f1 as f64 - M_PI / 2.0) as f32;
            }
        }
        if (*coll_data).env_flags & Collide_RightWallMask != 0 {
            phi_f1 = atan2f((*coll_data).right_facing_wall.normal.y, (*coll_data).right_facing_wall.normal.x);
            let phi_f3 = wrap_tau((M_PI + phi_f1 as f64) as f32);
            if (*fp).mv.ns.specialhi.aerialVel - phi_f3 < 0.0 {
                phi_f1 = (phi_f1 as f64 + M_PI / 2.0) as f32;
            } else {
                phi_f1 = (phi_f1 as f64 - M_PI / 2.0) as f32;
            }
        }
        let sp14 = melee_core::vec::Vec3 { x: 0.0, y: 0.0, z: 1.0 };
        lbvector::rotate_about_unit_axis(
            &mut *(&mut (*fp).self_vel as *mut Vec3 as *mut melee_core::vec::Vec3),
            &sp14,
            phi_f1 - (*fp).mv.ns.specialhi.aerialVel,
        );
        (*fp).mv.ns.specialhi.aerialVel = atan2f((*fp).self_vel.y, (*fp).self_vel.x);
    }
}

/// Reset of the PK Thunder motion variables every Start enter / ground-wall hit does.
unsafe fn reset_thunder_vars(fp: *mut Fighter) {
    let a = attrs(fp);
    (*fp).mv.ns.specialhi.thunderTimerLoop1 = (*a).x40_PK_THUNDER_LOOP1 as i32;
    (*fp).mv.ns.specialhi.thunderTimerLoop2 = (*a).x44_PK_THUNDER_LOOP2 as i32;
    (*fp).mv.ns.specialhi.gravityDelay = (*a).x48_PK_THUNDER_GRAVITY_DELAY as i32;
    (*fp).mv.ns.specialhi.fallAccel = 0.0;
    (*fp).mv.ns.specialhi.unkVector1.z = 0.0;
    (*fp).mv.ns.specialhi.unkVector1.y = 0.0;
    (*fp).mv.ns.specialhi.unkVector1.x = 0.0;
    (*fp).mv.ns.specialhi.unkVar4 = 0.0;
    (*fp).mv.ns.specialhi.unkVar3 = 0.0;
    (*fp).death2_cb = None;
    (*fp).take_dmg_cb = None;
    ext::ftPartSetRotX(fp, 0, 0.0);
}

/// Common body of the two Start enters, after `Fighter_ClearCmdVars`. The C writes `collPos1.x` /
/// `collPos2.x` three times each (y and z are untouched); that is kept.
unsafe fn start_enter(gobj: *mut HSD_GObj, msid: i32, air: bool) {
    let fp = fpp(gobj);
    let ness_attr = attrs(fp);
    st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, null());
    st::Fighter_ClearCmdVars(&mut *fp);
    reset_thunder_vars(fp);
    (*fp).mv.ns.specialhi.thunderColl = 1;
    (*fp).mv.ns.specialhi.gravityDelay = (*ness_attr).x48_PK_THUNDER_GRAVITY_DELAY as i32;
    (*fp).mv.ns.specialhi.jibakuGFX = 0;
    (*fp).mv.ns.specialhi.collPos1.x = 0.0;
    (*fp).mv.ns.specialhi.collPos2.x = 0.0;
    let phi_f0: f64 = if 1.0_f32 == (*fp).facing_dir { 0.0 } else { M_PI };
    (*fp).mv.ns.specialhi.aerialVel = phi_f0 as f32;
    (*fp).mv.ns.specialhi.facingDir = 1.0;
    (*fp).mv.ns.specialhi.unkVar = 0.0;
    if air {
        (*fp).self_vel.y = 0.0;
    }
    st::ftAnim_8006EBA4(gobj);
}

pub fn ftNs_SpecialHiStart_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHiStart_Enter");
    unsafe { start_enter(gobj, ftNs_MS_SpecialHiStart as i32, false) }
}

pub fn ftNs_SpecialAirHiStart_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiStart_Enter");
    unsafe { start_enter(gobj, ftNs_MS_SpecialAirHiStart as i32, true) }
}

/// Grounded PK Thunder 2 enter: launch along the thunder, or fall back to the air / downed path.
pub fn ftNs_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        let ness_attr = attrs(fp);
        let mut block_stuff = false;
        let mut to_air = true;
        if (*fp).coll_data.floor.flags as i32 & LINE_FLAG_PLATFORM == 0 {
            let mut sp40 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            sp40.x = (*fp).cur_pos.x - (*fp).mv.ns.specialhi.collPos1.x;
            let temp_f1 = fmadds(5.0, (*fp).x34_scale.y, (*fp).cur_pos.y);
            sp40.y = temp_f1 - (*fp).mv.ns.specialhi.collPos1.y;
            sp40.z = 0.0;

            let normal = &*(&(*fp).coll_data.floor.normal as *const Vec3 as *const melee_core::vec::Vec3);
            let spv = &*(&sp40 as *const Vec3 as *const melee_core::vec::Vec3);
            let temp_f1_2 = lbvector::angle(normal, spv);

            if !(temp_f1_2 < M_PI_2_F) {
                to_air = false;
                // MTXDegToRad(90 + x60): a plain multiply.
                if !(temp_f1_2 > (90.0_f32 + (*ness_attr).x60_PK_THUNDER_2_KNOCKDOWN_ANGLE) * 0.01745329252_f32) {
                    (*fp).facing_dir = if sp40.x >= 0.0 { 1.0 } else { -1.0 };
                    (*fp).mv.ns.specialhi.facingDir = if sp40.y >= 0.0 { 1.0 } else { -1.0 };
                    (*fp).mv.ns.specialhi.aerialVel = atan2f(sp40.y, sp40.x);
                    st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialHi as i32, 0, 0.0, 1.0, 0.0, null());
                    (*fp).gr_vel = (*ness_attr).x54_PK_THUNDER_2_MOMENTUM * (*fp).facing_dir;
                    (*fp).mv.ns.specialhi.unkVar = (*attrs(fp)).x58_PK_THUNDER_2_UNK1;
                    set_tilt(fp);
                    (*fp).death2_cb = None;
                    (*fp).take_dmg_cb = None;
                    (*fp).x1968_jumpsUsed = caps_max_jumps(&*fp) as u8;
                    return;
                }
                block_stuff = true;
            }
        }
        if block_stuff {
            destroy_gfx_if_special_hi(gobj, fp);
            ext::ftPartSetRotX(fp, 0, 0.0);
            ext::ftCo_80097D40(gobj);
            return;
        }
        let _ = to_air;
        (*fp).x1968_jumpsUsed = caps_max_jumps(&*fp) as u8;
        st::ftCommon_8007D60C(&mut *fp);
        ftNs_SpecialAirHi_Enter(gobj);
    }
}

/// `NessFloatMath_PKThunder2` (static inline): aim along the vector from PK Thunder's contact
/// point and launch.
pub fn NessFloatMath_PKThunder2(gobj: *mut HSD_GObj) {
    melee_core::ct!("NessFloatMath_PKThunder2");
    unsafe {
        let fp = fpp(gobj);
        let temp_f2 = (*fp).cur_pos.x - (*fp).mv.ns.specialhi.collPos1.x;
        let ness_attr = attrs(fp);
        let temp_f1 = fmadds(5.0, (*fp).x34_scale.y, (*fp).cur_pos.y) - (*fp).mv.ns.specialhi.collPos1.y;
        (*fp).facing_dir = if temp_f2 >= 0.0 { 1.0 } else { -1.0 };
        (*fp).mv.ns.specialhi.facingDir = if temp_f1 >= 0.0 { 1.0 } else { -1.0 };
        (*fp).mv.ns.specialhi.aerialVel = atan2f(temp_f1, temp_f2);
        (*fp).self_vel.x = (*ness_attr).x54_PK_THUNDER_2_MOMENTUM * cosf((*fp).mv.ns.specialhi.aerialVel);
        (*fp).self_vel.y = (*ness_attr).x54_PK_THUNDER_2_MOMENTUM * sinf((*fp).mv.ns.specialhi.aerialVel);
    }
}

/// PK Thunder 2 launch into the air (from SpecialHiHold or a failed ground launch).
pub fn ftNs_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHi_Enter");
    unsafe {
        NessFloatMath_PKThunder2(gobj);
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialAirHi as i32, 0, 0.0, 1.0, 0.0, null());
        let fp = fpp(gobj);
        (*fp).mv.ns.specialhi.unkVar = (*attrs(fp)).x58_PK_THUNDER_2_UNK1;
        set_tilt(fp);
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
        (*fp).x1968_jumpsUsed = caps_max_jumps(&*fp) as u8;
    }
}

/// Spawn the thunder item at Ness's hand if there is none, hook the damage callbacks and put up
/// the graphics (shared by both Start animations).
unsafe fn start_anim(gobj: *mut HSD_GObj, hold_msid: i32) {
    let mut fp = fpp(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::Fighter_ChangeMotionState(gobj, hold_msid, 0, 0.0, 1.0, 0.0, null());
        let fd2 = fpp(gobj);
        let mut pkt_ptr = (*fd2).u.ns.pkthunder_gobj;
        if pkt_ptr.is_null() {
            let mut pkt_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::lb_8000B1CC(joint(fd2, FtPart_L2ndNa as usize), core::ptr::null(), &mut pkt_pos);
            pkt_pos.z = 0.0;
            pkt_ptr = ext::it_802AB58C(gobj, &mut pkt_pos, (*fd2).facing_dir);
            (*fd2).u.ns.pkthunder_gobj = pkt_ptr;
            if !pkt_ptr.is_null() {
                (*fd2).death2_cb = on_damage_cb();
                (*fd2).take_dmg_cb = on_damage_cb();
            }
        }
        (*fp).x1968_jumpsUsed = caps_max_jumps(&*fp) as u8;
        fp = fpp(gobj);
        ftNs_SpecialHiStopGFX(gobj);
        ext::efSync_Spawn(1262, gobj, &[joint(fp, FtPart_HipN as usize) as *mut c_void]);
        (*fp).u.ns.pkthunder_gfx = 1;
    }
}

pub fn ftNs_SpecialHiStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHiStart_Anim");
    unsafe { start_anim(gobj, ftNs_MS_SpecialHiHold as i32) }
}

pub fn ftNs_SpecialAirHiStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiStart_Anim");
    unsafe { start_anim(gobj, ftNs_MS_SpecialAirHiHold as i32) }
}

/// Shared Hold loop: tick the two timers, end the move when the thunder is gone and both ran out,
/// or launch when PK Thunder reaches Ness (`air`: aerial launch inline, else `ftNs_SpecialHi_Enter`).
unsafe fn hold_anim(gobj: *mut HSD_GObj, end_msid: i32, air: bool) {
    let fp = fpp(gobj);
    if (*fp).mv.ns.specialhi.thunderTimerLoop1 > 0 {
        (*fp).mv.ns.specialhi.thunderTimerLoop1 -= 1;
    }
    if (*fp).u.ns.pkthunder_gobj.is_null() && (*fp).mv.ns.specialhi.thunderTimerLoop2 > 0 {
        (*fp).mv.ns.specialhi.thunderTimerLoop2 -= 1;
    }
    if (*fp).u.ns.pkthunder_gobj.is_null() {
        if (*fp).mv.ns.specialhi.thunderTimerLoop1 <= 0 && (*fp).mv.ns.specialhi.thunderTimerLoop2 <= 0 {
            st::Fighter_ChangeMotionState(gobj, end_msid, 0, 0.0, 1.0, 0.0, null());
            destroy_gfx_if_special_hi(gobj, fpp(gobj));
        }
    } else if ext::it_802AB568((*fp).u.ns.pkthunder_gobj) == gobj {
        if ftNs_SpecialHi_ItemPKThunder_CheckNessCollide(gobj) {
            if air {
                ftNs_SpecialAirHi_Enter(gobj);
            } else {
                ftNs_SpecialHi_Enter(gobj);
            }
        }
    } else {
        (*fp).u.ns.pkthunder_gobj = null();
        st::Fighter_ChangeMotionState(gobj, end_msid, 0, 0.0, 1.0, 0.0, null());
        destroy_gfx_if_special_hi(gobj, fpp(gobj));
    }
}

pub fn ftNs_SpecialHiHold_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHiHold_Anim");
    unsafe { hold_anim(gobj, ftNs_MS_SpecialHiEnd as i32, false) }
}

pub fn ftNs_SpecialAirHiHold_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiHold_Anim");
    unsafe { hold_anim(gobj, ftNs_MS_SpecialAirHiEnd as i32, true) }
}

/// `jibakuGFX++`; on the first frame swap the thunder graphics for PK Thunder 2's.
unsafe fn jibaku_gfx(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    (*fp).mv.ns.specialhi.jibakuGFX += 1;
    if (*fp).mv.ns.specialhi.jibakuGFX == 1 {
        let fp0 = fpp(gobj);
        ftNs_SpecialHiStopGFX(gobj);
        ext::efSync_Spawn(1263, gobj, &[joint(fp0, FtPart_HipN as usize) as *mut c_void]);
        (*fp0).u.ns.pkthunder_gfx = 1;
    }
}

pub fn ftNs_SpecialHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHi_Anim");
    unsafe {
        let temp_fp = fpp(gobj);
        jibaku_gfx(gobj, temp_fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialHiEnd as i32, 0, 0.0, 1.0, 0.0, null());
            destroy_gfx_if_special_hi(gobj, fpp(gobj));
        }
    }
}

/// Leave the move into a landing-lag free-fall (or plain Fall without lag).
unsafe fn leave_to_fall(gobj: *mut HSD_GObj, ness_attr: *const ftNessAttributes) {
    if (*ness_attr).x70_PK_THUNDER_2_LANDING_LAG == 0.0 {
        st::ftCo_Fall_Enter(gobj);
        return;
    }
    ext::ftCo_800969D8(
        gobj,
        1,
        0,
        1,
        1.0,
        (*ness_attr).x70_PK_THUNDER_2_LANDING_LAG,
        (*ness_attr).x6C_PK_THUNDER_2_FREEFALL_ANIM_BLEND,
    );
}

pub fn ftNs_SpecialAirHiEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        let ness_attr = attrs(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            (*fp).x1968_jumpsUsed = caps_max_jumps(&*fp) as u8;
            st::ftCommon_8007D60C(&mut *fp);
            leave_to_fall(gobj, ness_attr);
        }
    }
}

pub fn ftNs_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHi_Anim");
    unsafe {
        let fp1 = fpp(gobj);
        let ness_attr = attrs(fp1);
        jibaku_gfx(gobj, fp1);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let temp_f1 = (*fp1).mv.ns.specialhi.fallAccel;
            let mut phi_f1 = temp_f1;
            if temp_f1 < 0.0 {
                phi_f1 = -temp_f1;
            }
            (*fp1).self_vel.y = -phi_f1;
            st::ftCommon_8007D60C(&mut *fp1);
            leave_to_fall(gobj, ness_attr);
        }
    }
}

pub fn ftNs_SpecialAirHiRebound_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiRebound_Anim");
    unsafe {
        let fp = fpp(gobj);
        let ness_attr = attrs(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCommon_8007D60C(&mut *fp);
            leave_to_fall(gobj, ness_attr);
        }
    }
}

pub fn ftNs_SpecialHiStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftNs_SpecialHiStart_IASA");}

/// `ThunderPhysTimer` (static inline): count the gravity delay down.
unsafe fn thunder_phys_timer(gobj: *mut HSD_GObj) {
    let temp_fp = fpp(gobj);
    let t = (*temp_fp).mv.ns.specialhi.gravityDelay;
    if t != 0 {
        (*temp_fp).mv.ns.specialhi.gravityDelay = t - 1;
    }
}

pub fn ftNs_SpecialHiStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHiStart_Phys");
    unsafe { thunder_phys_timer(gobj) };
    st::ft_80084F3C(gobj);
}

pub fn ftNs_SpecialHiHold_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHiHold_Phys");
    st::ft_80084F3C(gobj);
}

/// Grounded PK Thunder 2 physics: decelerate the ground speed and keep the vertical speed from
/// reversing, then tilt the model.
pub fn ftNs_SpecialHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHi_Phys");
    unsafe {
        let fp0 = fpp(gobj);
        let ground_vel = (*fp0).gr_vel;
        let vel_y = (*fp0).self_vel.y;
        let ness_attr = attrs(fp0);

        (*fp0).gr_vel = -fmadds((*ness_attr).x5C_PK_THUNDER_2_DECELERATION_RATE, (*fp0).facing_dir, -ground_vel);

        if (*fp0).facing_dir == 1.0 {
            if (*fp0).gr_vel <= vel_epsilon {
                (*fp0).gr_vel = ground_vel;
            }
        } else if (*fp0).gr_vel >= -vel_epsilon {
            (*fp0).gr_vel = ground_vel;
        }

        if (*fp0).mv.ns.specialhi.facingDir == 1.0 {
            if (*fp0).self_vel.y <= vel_epsilon {
                (*fp0).self_vel.y = vel_y;
            }
        } else if (*fp0).self_vel.y >= -vel_epsilon {
            (*fp0).self_vel.y = vel_y;
        }

        (*fp0).mv.ns.specialhi.unkVector1 = (*fp0).self_vel;
        melee_fighter::ftcommon::set_self_movement_from_grounded_movement(&mut *fp0);
        set_tilt(fpp(gobj));
    }
}

/// Shared aerial physics of Start / Hold / End.
unsafe fn air_thunder_phys(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let gravity_timer = (*fp).mv.ns.specialhi.gravityDelay;
    let ness_attr = attrs(fp);
    if gravity_timer != 0 {
        (*fp).mv.ns.specialhi.gravityDelay = gravity_timer - 1;
    } else {
        st::ftCommon_Fall(&mut *fp, (*ness_attr).x50_PK_THUNDER_FALL_ACCEL, (*fp).co_attrs.terminal_velocity);
    }
    let friction = (*fp).co_attrs.aerial_friction;
    melee_fighter::ftcommon::calc_self_accel_deaccel(&mut *fp, friction);
}

pub fn ftNs_SpecialAirHiStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiStart_Phys");
    unsafe { air_thunder_phys(gobj) }
}

pub fn ftNs_SpecialAirHiHold_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiHold_Phys");
    unsafe { air_thunder_phys(gobj) }
}

pub fn ftNs_SpecialAirHiEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiEnd_Phys");
    unsafe { air_thunder_phys(gobj) }
}

/// `ftNess_atan2` (static inline): tilt the model along the velocity.
pub fn ftNess_atan2(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNess_atan2");
    unsafe { set_tilt(fpp(gobj)) }
}

/// Aerial PK Thunder 2 physics.
pub fn ftNs_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHi_Phys");
    unsafe {
        let fp = fpp(gobj);
        let ness_attr = attrs(fp);

        let len = lbvector::len_xy(&*(&(*fp).self_vel as *const Vec3 as *const melee_core::vec::Vec3));
        let phi_f1 = abs_c(len);
        let temp_f2 = phi_f1 - (*ness_attr).x5C_PK_THUNDER_2_DECELERATION_RATE;
        let mut phi_f31 = temp_f2;
        if temp_f2 <= vel_epsilon {
            phi_f31 = phi_f1;
        }

        (*fp).self_vel.x = phi_f31 * cosf((*fp).mv.ns.specialhi.aerialVel);
        (*fp).self_vel.y = phi_f31 * sinf((*fp).mv.ns.specialhi.aerialVel);
        ftNess_atan2(gobj);
        (*fp).mv.ns.specialhi.unkVector1 = (*fp).self_vel;
        if (*fp).cmd_vars[0] == 1 {
            (*fp).mv.ns.specialhi.fallAccel -= (*ness_attr).x50_PK_THUNDER_FALL_ACCEL;
            if (*fp).mv.ns.specialhi.fallAccel < -(*ness_attr).x54_PK_THUNDER_2_MOMENTUM {
                (*fp).mv.ns.specialhi.fallAccel = -(*ness_attr).x54_PK_THUNDER_2_MOMENTUM;
            }
            (*fp).cur_pos.y += (*fp).mv.ns.specialhi.fallAccel;
        }
    }
}

pub fn ftNs_SpecialAirHiRebound_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiRebound_Phys");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_Fall(&mut *fp, (*fp).co_attrs.gravity, (*fp).co_attrs.terminal_velocity);
        let friction = (*fp).co_attrs.aerial_friction;
        melee_fighter::ftcommon::calc_self_accel_deaccel(&mut *fp, friction);
    }
}

/// Shared grounded `*_Coll` of Start / Hold / End: leave the ground into the matching air motion.
unsafe fn ground_coll_to_air(gobj: *mut HSD_GObj, air_msid: i32) {
    let fp = fpp(gobj);
    if st::ft_80082708(gobj) == 0 {
        st::ftCommon_8007D60C(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, air_msid, FTNESS_SPECIALHI_COLL_FLAG, (*fp).cur_anim_frame, 1.0, 0.0, null());
    }
}

pub fn ftNs_SpecialHiStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHiStart_Coll");
    unsafe { ground_coll_to_air(gobj, ftNs_MS_SpecialAirHiStart as i32) }
}

pub fn ftNs_SpecialHiHold_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHiHold_Coll");
    unsafe { ground_coll_to_air(gobj, ftNs_MS_SpecialAirHiHold as i32) }
}

pub fn ftNs_SpecialHiEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHiEnd_Coll");
    unsafe { ground_coll_to_air(gobj, ftNs_MS_SpecialAirHiEnd as i32) }
}

/// Grounded PK Thunder 2 collision.
pub fn ftNs_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialHi_Coll");
    unsafe {
        let fp0 = fpp(gobj);
        if st::ft_80082708(gobj) == 0 {
            let env_flags = (*fp0).coll_data.env_flags;
            if env_flags & Collide_LeftWallMask != 0 || env_flags & Collide_RightWallMask != 0 {
                st::ftCommon_8007D60C(&mut *fp0);
                let fp1 = fpp(gobj);
                reset_thunder_vars(fp1);
                st::Fighter_ChangeMotionState(
                    gobj,
                    ftNs_MS_SpecialAirHiEnd as i32,
                    FTNESS_SPECIALHI_COLL_FLAG,
                    (*fp0).cur_anim_frame,
                    1.0,
                    0.0,
                    null(),
                );
                return;
            }
            st::ftCommon_8007D60C(&mut *fp0);
            st::Fighter_ChangeMotionState(
                gobj,
                ftNs_MS_SpecialAirHi as i32,
                FTNESS_JIBAKU_COLL_FLAG,
                (*fp0).cur_anim_frame,
                1.0,
                0.0,
                null(),
            );
            return;
        }

        let env_flags = (*fp0).coll_data.env_flags;
        if env_flags & Collide_CeilingMask != 0
            || env_flags & Collide_LeftWallMask != 0
            || env_flags & Collide_RightWallMask != 0
        {
            (*fp0).gr_vel = 0.0;
            destroy_gfx_if_special_hi(gobj, fpp(gobj));
            ext::ftPartSetRotX(fp0, 0, 0.0);
            ext::ftCo_80097D40(gobj);
            return;
        }

        let (ny, nx) = ((*fp0).coll_data.floor.normal.y, (*fp0).coll_data.floor.normal.x);
        if ny > 0.0 {
            let angle: f64 = if (*fp0).facing_dir == 1.0 { -core::f64::consts::FRAC_PI_2 } else { core::f64::consts::FRAC_PI_2 };
            (*fp0).mv.ns.specialhi.aerialVel = (angle + atan2f(ny, nx) as f64) as f32;
            return;
        }
        let angle: f64 = if (*fp0).facing_dir == 1.0 { core::f64::consts::FRAC_PI_2 } else { -core::f64::consts::FRAC_PI_2 };
        (*fp0).mv.ns.specialhi.aerialVel = (angle + atan2f(ny, nx) as f64) as f32;
    }
}

/// Shared aerial `*_Coll` of Start / Hold / End: landing returns to the matching grounded motion.
unsafe fn air_coll_to_ground(gobj: *mut HSD_GObj, ground_msid: i32) {
    let fp = fpp(gobj);
    if st::ft_80081D0C(gobj) != 0 {
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ground_msid, FTNESS_SPECIALHI_COLL_FLAG);
    }
}

pub fn ftNs_SpecialAirHiStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiStart_Coll");
    unsafe { air_coll_to_ground(gobj, ftNs_MS_SpecialHiStart as i32) }
}

pub fn ftNs_SpecialAirHiHold_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiHold_Coll");
    unsafe { air_coll_to_ground(gobj, ftNs_MS_SpecialHiHold as i32) }
}

pub fn ftNs_SpecialAirHiEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiEnd_Coll");
    unsafe { air_coll_to_ground(gobj, ftNs_MS_SpecialHiEnd as i32) }
}

/// The wall-hug angle test `lbVector_Angle(normal, vel) > 0.01745329238474369f * (90 + x64)`.
#[inline(always)]
unsafe fn steep(normal: *const Vec3, fp: *mut Fighter, ness_attr: *const ftNessAttributes) -> bool {
    lbvector::angle(
        &*(normal as *const melee_core::vec::Vec3),
        &*(&(*fp).self_vel as *const Vec3 as *const melee_core::vec::Vec3),
    ) > 0.01745329238474369_f32 * (90.0_f32 + (*ness_attr).x64_PK_THUNDER_2_WALLHUG_ANGLE)
}

/// Bounce off a wall/ceiling into the Rebound motion (shared by the three branches of AirHi_Coll).
unsafe fn rebound(gobj: *mut HSD_GObj, fp: *mut Fighter, normal: *const Vec3) {
    lbvector::mirror(
        &mut *(&mut (*fp).self_vel as *mut Vec3 as *mut melee_core::vec::Vec3),
        &*(normal as *const melee_core::vec::Vec3),
    );
    (*fp).self_vel.x *= 0.5;
    (*fp).self_vel.y *= 0.5;
    // lbVector_sqrtf_accurate(x*x + y*y): result unused.
    melee_fighter::ftcommon::clamp_self_vel_x(&mut *fp, (*fp).co_attrs.air_drift_max);
    (*fp).facing_dir = if (*fp).self_vel.x >= 0.0 { 1.0 } else { -1.0 };
    ftNs_SpecialHiStopGFX(gobj);
    st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialAirHiRebound as i32, Ft_MF_KeepGfx, 0.0, 1.0, 0.0, null());
    st::ftAnim_8006EBA4(gobj);
    let mut sp = Vec3 { x: atan2f(-(*normal).x, (*normal).y), y: 0.0, z: 0.0 };
    ext::efSync_Spawn(
        1030,
        gobj,
        &[&mut (*fp).cur_pos as *mut Vec3 as *mut c_void, &mut sp as *mut Vec3 as *mut c_void],
    );
}

/// Aerial PK Thunder 2 collision.
pub fn ftNs_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        let ness_attr = attrs(fp);
        (*fp).self_vel = (*fp).mv.ns.specialhi.unkVector1;

        let ledge_grab_dir = if 1.0_f32 == (*fp).facing_dir { 1 } else { -1 };
        if st::ft_CheckGroundAndLedge(gobj, ledge_grab_dir) != 0 {
            if steep(&(*fp).coll_data.floor.normal, fp, ness_attr) {
                (*fp).self_vel.z = 0.0;
                (*fp).self_vel.y = 0.0;
                (*fp).self_vel.x = 0.0;
                destroy_gfx_if_special_hi(gobj, fpp(gobj));
                ext::ftPartSetRotX(fp, 0, 0.0);
                ext::ftCo_80097D40(gobj);
                return;
            }
            st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftNs_MS_SpecialHi as i32, FTNESS_JIBAKU_COLL_FLAG);
            return;
        }

        if st::ftCliffCommon_80081298(gobj) != 0 {
            ext::ftCliffCommon_80081370(gobj);
            return;
        }

        let ecb_flag = (*fp).coll_data.env_flags;
        if ecb_flag & Collide_CeilingMask != 0 {
            if steep(&(*fp).coll_data.ceiling.normal, fp, ness_attr) {
                rebound(gobj, fp, &(*fp).coll_data.ceiling.normal);
            }
        } else {
            if ecb_flag & Collide_LeftWallMask != 0 {
                if steep(&(*fp).coll_data.left_facing_wall.normal, fp, ness_attr) {
                    rebound(gobj, fp, &(*fp).coll_data.left_facing_wall.normal);
                    return;
                }
                ftNs_SpecialAirHi_CollisionModVel(gobj, &mut (*fp).coll_data);
                return;
            }
            if ecb_flag & Collide_RightWallMask != 0 {
                if steep(&(*fp).coll_data.right_facing_wall.normal, fp, ness_attr) {
                    rebound(gobj, fp, &(*fp).coll_data.right_facing_wall.normal);
                    return;
                }
                ftNs_SpecialAirHi_CollisionModVel(gobj, &mut (*fp).coll_data);
            }
        }
    }
}

/// Wall-rebound collision: stop on ground or ledge, else grab the ledge.
pub fn ftNs_SpecialAirHiRebound_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirHiRebound_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_CheckGroundAndLedge(gobj, 0) == 1 {
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            (*fp).self_vel.x = 0.0;
            destroy_gfx_if_special_hi(gobj, fpp(gobj));
            ext::ftPartSetRotX(fp, 0, 0.0);
            ext::ftCo_80097D40(gobj);
            return;
        }
        if st::ftCliffCommon_80081298(gobj) != 0 {
            ext::ftCliffCommon_80081370(gobj);
        }
    }
}
