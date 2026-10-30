//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandrockcrush0.c: Crazy Hand rock-crush rise.
//!
//! Fused sites (fma_sites.py): `my_lbVector_Len` `x*x + y*y + z*z` -> fmadds(z, z, fmadds(x, x,
//! y*y)); `my_sqrtf` is crate::pure::my_sqrtf (three f64 Newton steps, fnmsub fused). The Phys
//! velocity is checked bit for bit against the C by tests/tu_ftcrazyhandrockcrush0.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftcrazyhandfingerbeam0::ext::ftCh_GrabUnk1_8015BC88;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (round 8): re-exported under this TU.
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandrockcrush0::ftCh_RockCrushUp_IASA;

/// `my_lbVector_Len` (static inline).
#[inline]
pub fn my_lbVector_Len(v: &Vec3) -> f32 {
    melee_core::ct!("my_lbVector_Len");
    crate::pure::my_sqrtf(fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y)))
}

/// The arithmetic of `ftCh_RockCrushUp_Phys` after `ftBossLib_8015C208`: the (self_vel.x, self_vel.y)
/// chasing `target` (the other fighter's position) from `cur`, with the data offsets/gain of `da`.
pub fn rockcrush_up_vel(target: &Vec3, cur: &Vec3, da: &ftCrazyHand_DatAttrs) -> (f32, f32) {
    melee_core::ct!("rockcrush_up_vel");
    let mut pos = *target;
    pos.x += da.x5C;
    pos.y += da.x60;
    pos.z = 0.0;
    let mut vel = Vec3 { x: pos.x - cur.x, y: pos.y - cur.y, z: pos.z - cur.z };
    let len = my_lbVector_Len(&vel);
    if len < da.x14 {
        (vel.x, vel.y)
    } else {
        let mut w = melee_core::vec::Vec3 { x: vel.x, y: vel.y, z: vel.z };
        melee_runtime::lbvector::normalize(&mut w);
        vel = Vec3 { x: w.x, y: w.y, z: w.z };
        let speed = len * da.x10;
        vel.x *= speed;
        vel.y *= speed;
        (vel.x, vel.y)
    }
}

pub fn ftCh_Init_80157DF8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80157DF8");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        st::Fighter_ChangeMotionState(gobj, 0x163, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        let r = melee_fighter::hooks::hsd_randi((*da).x54.wrapping_sub((*da).x58));
        (*fp).mv.ch.unk13.x0 = (*da).x58.wrapping_add(r) as f32;
        (*fp).mv.ch.unk13.x4 = 0.0;
    }
}

pub fn ftCh_RockCrushUp_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_RockCrushUp_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            (*fp).mv.ch.unk13.x4 = 1.0;
            (*fp).mv.ch.unk13.x0 -= 1.0;
            if (*fp).mv.ch.unk13.x0 < 0.0 {
                (*fp).mv.ch.unk13.x4 = 0.0;
                (*fp).self_vel.x = 0.0;
                (*fp).self_vel.y = 0.0;
                st::Fighter_ChangeMotionState(gobj, 0x164, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                st::ftAnim_8006EBA4(gobj);
                melee_fighter::hooks::play_sfx(&mut *fp, 0x4E207, 127, 64);
            }
        }
    }
}

pub fn ftCh_RockCrushWait_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_RockCrushWait_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_RockCrushUp_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_RockCrushUp_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        st::ft_80085134(gobj);
        if (*fp).mv.ch.unk13.x4 != 0.0 {
            let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
            let mut pos: Vec3 = core::mem::zeroed();
            melee_fighter::tu::ftbosslib::ftBossLib_8015C208(gobj, &mut pos);
            let cur = (*fp).cur_pos;
            let (vx, vy) = rockcrush_up_vel(&pos, &cur, &*da);
            (*fp).self_vel.x = vx;
            (*fp).self_vel.y = vy;
        }
    }
}

pub fn ftCh_RockCrushUp_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_RockCrushUp_Coll");}
