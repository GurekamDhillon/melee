//! src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun.c: Master Hand finger-beam end and the
//! finger-gun states (the bullet-spawning accessory callback `ftMh_MS_365_801535B0`).
//!
//! Fused sites (fma_sites.py): `my_lbVector_Len` `x*x + y*y + z*z` -> fmadds(z, z, fmadds(x, x,
//! y*y)); `my_sqrtf` is crate::pure::my_sqrtf (three f64 Newton steps, fnmsub fused). The
//! FingerGun1 Phys velocity is checked bit for bit against the C by tests/tu_ftmasterhandfingergun.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.



/// Calls into TUs that are not ported yet. Each is a settable hook.
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftmasterhanddamage0::ext::{lb_8000B1CC, ftMh_MS_389_80151018};
    pub use crate::tu::ftmasterhandpoke::ext::ftBossLib_8015C208;
    // `it_802F0AE0` (itmasterhandbullet.c): `msid` is the C's `b` (bool as int).
    pub static mut IT_802F0AE0: Option<fn(*mut HSD_GObj, *mut Vec3, *mut Vec3, ItemKind, i32, f32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn it_802F0AE0(gobj: *mut HSD_GObj, prev_pos: *mut Vec3, pos: *mut Vec3, kind: ItemKind, msid: i32, facing_dir: f32, vel_x: f32, vel_y: f32) {
        match unsafe { IT_802F0AE0 } {
            Some(f) => f(gobj, prev_pos, pos, kind, msid, facing_dir, vel_x, vel_y),
            None => { unsafe { melee_items::tu::itmasterhandbullet::it_802F0AE0(gobj as _, prev_pos as _, pos as _, kind, msid as _, facing_dir as _, vel_x as _, vel_y as _); } },
        }
    }
    fn ftLib_80087120_port(gobj: *mut HSD_GObj) -> i32 {
        melee_fighter::ftlib::ftlib_80087120(gobj)
    }
    // Falls back to melee_fighter's port.
    crate::mh_hook_or!(FTLIB_80087120, ftLib_80087120, ftLib_80087120_port, (gobj: *mut HSD_GObj) -> i32);
}

/// `my_lbVector_Len` (static inline).
#[inline]
pub fn my_lbVector_Len(v: &Vec3) -> f32 {
    melee_core::ct!("my_lbVector_Len");
    crate::pure::my_sqrtf(v.x * v.x + v.y * v.y + v.z * v.z) // through a pointer: not fused on the console
}

/// The arithmetic of `ftMh_FingerGun1_Phys` after `ftBossLib_8015C208`: the (self_vel.x, self_vel.y)
/// chasing `target` from `cur`, with the data offsets `off_x/off_y` (xDC/xE0), the snap radius
/// `x2c` and the gain `x28` of the attributes.
pub fn fingergun1_vel(target: &Vec3, cur: &Vec3, off_x: f32, off_y: f32, x2c: f32, x28: f32) -> (f32, f32) {
    melee_core::ct!("fingergun1_vel");
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

pub fn ftMh_FingerBeamEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerBeamEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_FingerBeamEnd_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_FingerBeamEnd_Coll");}

pub fn ftMh_MS_363_801530A4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_363_801530A4");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::Fighter_ChangeMotionState(gobj, ftMh_MS_FingerGun1 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        // HSD_Randi(float) converts the f32 difference to s32; `xE4 + rand` is an f32 add.
        let rand = melee_fighter::hooks::hsd_randi(melee_core::ppc::fctiwz_f32((*da).xE8 - (*da).xE4));
        (*fp).mv.mh.unk0.x50 = (*da).xE4 + rand as f32;
        (*fp).cmd_vars[0] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[2] = 0;
    }
}

pub fn ftMh_FingerGun1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerGun1_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            (*fp).cmd_vars[2] = 1;
            let f = (*fp).mv.mh.unk0.x50 - 1.0;
            (*fp).mv.mh.unk0.x50 = f;
            if f < 0.0 {
                let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
                ftMh_MS_364_801533CC(gobj);
                if ext::ftLib_80087120(gobj) > (*da).xEC {
                    (*fp).mv.mh.unk0.x54 = (*da).xF0;
                } else {
                    (*fp).mv.mh.unk0.x54 = 1;
                }
                (*fp).cmd_vars[2] = 0;
            }
        }
    }
}

pub fn ftMh_FingerGun1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerGun1_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        st::ft_80085134(gobj);
        if (*fp).cmd_vars[2] != 0 {
            let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
            let mut pos: Vec3 = core::mem::zeroed();
            ext::ftBossLib_8015C208(gobj, &mut pos);
            let cur = (*fp).cur_pos;
            let (vx, vy) = fingergun1_vel(&pos, &cur, (*da).xDC, (*da).xE0, (*da).x2C, (*da).x28);
            (*fp).self_vel.x = vx;
            (*fp).self_vel.y = vy;
        }
    }
}

pub fn ftMh_FingerGun1_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_FingerGun1_Coll");}

/// Enter FingerGun2 (also the body of the C's `lbl_8015346C_inline`).
pub fn ftMh_MS_364_801533CC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_364_801533CC");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::Fighter_ChangeMotionState(gobj, ftMh_MS_FingerGun2 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        if ext::ftLib_80087120(gobj) > (*da).xEC {
            st::ftAnim_SetAnimRate(gobj, (*da).xF4);
        }
        (*fp).self_vel.x = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).accessory4_cb = Some(ftMh_MS_365_801535B0);
    }
}

pub fn ftMh_FingerGun2_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerGun2_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            (*fp).mv.mh.unk0.x54 = (*fp).mv.mh.unk0.x54.wrapping_sub(1);
            if (*fp).mv.mh.unk0.x54 == 0 {
                st::ftAnim_SetAnimRate(gobj, 1.0);
                ftMh_MS_365_80153730(gobj);
            } else {
                ftMh_MS_364_801533CC(gobj);
            }
        }
    }
}

pub fn ftMh_FingerGun2_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_FingerGun2_Coll");}

/// Accessory-4 callback: spawns a bullet from each knee when the script raises its flag.
pub unsafe extern "C" fn ftMh_MS_365_801535B0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_365_801535B0");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        if (*fp).cmd_vars[0] != 0 {
            ftMh_MS_365_8015364C(gobj, (*(*fp).parts.add(FtPart_LKneeJ as usize)).joint, (*da).xF8, (*da).xFC);
            (*fp).cmd_vars[0] = 0;
        }
        if (*fp).cmd_vars[1] != 0 {
            ftMh_MS_365_8015364C(gobj, (*(*fp).parts.add(FtPart_RKneeJ as usize)).joint, (*da).x100, (*da).x104);
            (*fp).cmd_vars[1] = 0;
        }
    }
}

/// `ftMh_MS_365_8015364C` (static in the C): spawn one MasterHand bullet at `joint` + (arg2, arg3).
pub fn ftMh_MS_365_8015364C(gobj: *mut HSD_GObj, joint: *mut HSD_JObj, arg2: f32, arg3: f32) {
    melee_core::ct!("ftMh_MS_365_8015364C");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        let mut b = false;
        let mut vec0: Vec3 = core::mem::zeroed();
        ext::lb_8000B1CC(joint, core::ptr::null(), &mut vec0);
        let mut vec1 = vec0;
        vec0.x += arg2;
        vec0.y += arg3;
        if ext::ftLib_80087120(gobj) > (*da).xEC {
            b = true;
        }
        ext::it_802F0AE0(gobj, &mut vec0, &mut vec1, It_Kind_MasterHand_Bullet, b as i32, (*fp).facing_dir, (*da).xD4, (*da).xD8);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_FingerBeamEnd_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun.c).
pub fn ftMh_FingerBeamEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerBeamEnd_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_FingerGun2_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun.c).
pub fn ftMh_FingerGun2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerGun2_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_MS_365_80153730` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun.c).
pub fn ftMh_MS_365_80153730(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_365_80153730");
    st::Fighter_ChangeMotionState(gobj, 366 /* ftMh_MS_FingerGun3 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftMh_FingerBeamEnd_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun.c).
pub fn ftMh_FingerBeamEnd_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerBeamEnd_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_FingerGun1_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun.c).
pub fn ftMh_FingerGun1_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerGun1_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_FingerGun2_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun.c).
pub fn ftMh_FingerGun2_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerGun2_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
