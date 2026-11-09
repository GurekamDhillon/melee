//! src/melee/ft/kinds/ftMasterHand/ftmasterhandpoke.c: Master Hand poke states.
//!
//! Fused sites (fma_sites.py): `my_lbVector_Len` `x*x + y*y + z*z` -> fmadds(z, z, fmadds(x, x,
//! y*y)); `my_sqrtf` is crate::pure::my_sqrtf (three f64 Newton steps, fnmsub fused). The Phys
//! velocity is checked bit for bit against the C by tests/tu_ftmasterhandpoke.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (round 8): re-exported under this TU.


/// Calls into TUs that are not ported yet. Each is a settable hook.
pub mod ext {
    pub use crate::tu::ftmasterhanddamage0::ext::ftMh_MS_389_80151018;
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FTBOSSLIB_8015C208, ftBossLib_8015C208, ftBossLib_8015C208_port, (gobj: *mut melee_types::HSD_GObj, out: *mut melee_types::Vec3));
    fn ftBossLib_8015C208_port(gobj: *mut melee_types::HSD_GObj, out: *mut melee_types::Vec3) {
        melee_fighter::tu::ftbosslib::ftBossLib_8015C208(gobj, unsafe { &mut *out })
    }
}

/// `my_lbVector_Len` (static inline).
#[inline]
pub fn my_lbVector_Len(v: &Vec3) -> f32 {
    melee_core::ct!("my_lbVector_Len");
    crate::pure::my_sqrtf(v.x * v.x + v.y * v.y + v.z * v.z) // through a pointer: not fused on the console
}

/// The arithmetic of `ftMh_Poke1_Phys` after `ftBossLib_8015C208`: the (self_vel.x, self_vel.y)
/// chasing `target` from `cur`, with the data offsets `off_x/off_y` (x98/x9C), the snap radius
/// `x2c` and the gain `x28` of the attributes.
pub fn poke1_vel(target: &Vec3, cur: &Vec3, off_x: f32, off_y: f32, x2c: f32, x28: f32) -> (f32, f32) {
    melee_core::ct!("poke1_vel");
    let mut pos = *target;
    pos.x += off_x;
    pos.y += off_y;
    pos.z = 0.0;
    let mut vel = Vec3 { x: pos.x - cur.x, y: pos.y - cur.y, z: pos.z - cur.z };
    let len = my_lbVector_Len(&vel);
    if len < x2c {
        (vel.x, vel.y)
    } else {
        let mut w = melee_core::vec::Vec3 { x: vel.x, y: vel.y, z: vel.z };
        melee_runtime::lbvector::normalize(&mut w);
        vel = Vec3 { x: w.x, y: w.y, z: w.z };
        let speed = len * x28;
        vel.x *= speed;
        vel.y *= speed;
        (vel.x, vel.y)
    }
}

pub fn ftMh_PaperCrush_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_PaperCrush_Coll");}

pub fn ftMh_MS_358_80152880(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_358_80152880");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::Fighter_ChangeMotionState(gobj, ftMh_MS_Poke1 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        let r = melee_fighter::hooks::hsd_randi((*da).x90.wrapping_sub((*da).x94));
        (*fp).mv.mh.unk13.x0 = (*da).x94.wrapping_add(r) as f32;
        (*fp).mv.mh.unk13.x4 = 0.0;
    }
}

pub fn ftMh_Poke1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Poke1_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            (*fp).mv.mh.unk13.x4 = 1.0;
            (*fp).mv.mh.unk13.x0 -= 1.0;
            if (*fp).mv.mh.unk13.x0 < 0.0 {
                (*fp).mv.mh.unk13.x4 = 0.0;
                (*fp).self_vel.x = 0.0;
                (*fp).self_vel.y = 0.0;
                st::Fighter_ChangeMotionState(gobj, ftMh_MS_Poke2 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
                st::ftAnim_8006EBA4(gobj);
                melee_fighter::hooks::play_sfx(&mut *fp, 320007, 127, 64);
            }
        }
    }
}

pub fn ftMh_Poke2_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Poke2_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_Poke1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Poke1_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        st::ft_80085134(gobj);
        if (*fp).mv.mh.unk13.x4 != 0.0 {
            let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
            let mut pos: Vec3 = core::mem::zeroed();
            ext::ftBossLib_8015C208(gobj, &mut pos);
            let cur = (*fp).cur_pos;
            let (vx, vy) = poke1_vel(&pos, &cur, (*da).x98, (*da).x9C, (*da).x2C, (*da).x28);
            (*fp).self_vel.x = vx;
            (*fp).self_vel.y = vy;
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_Poke1_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandpoke.c).
pub fn ftMh_Poke1_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Poke1_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
