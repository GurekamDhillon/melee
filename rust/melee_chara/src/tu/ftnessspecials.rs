//! src/melee/ft/kinds/ftNess/ftnessspecials.c: PK Fire (side B), grounded and aerial.
//! One fused site (tools/rust-port/fma_sites.py: `ftNs_SpecialS_ItemPKFireSpawn` 1 f32): the
//! spawn x offset `pos.x += spawn_x * facing_dir` is an `fmadds`. Smoke-tested through recording
//! hooks by tests/tu_ftnessspecials.rs (the spawn position and velocity are checked numerically).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::*;

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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *const Vec3, pos1: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, pos0, pos1),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; },
                }
            }
    pub static mut IT_802AA054: Option<fn(*mut HSD_GObj, *mut Vec3, *mut Vec3, f32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AA054(gobj: *mut HSD_GObj, pos: *mut Vec3, vel: *mut Vec3, dir: f32, rot: f32)  {
                match unsafe { IT_802AA054 } {
                    Some(f) => f(gobj, pos, vel, dir, rot),
                    None => { unsafe { let _ = melee_items::tu::itnesspkfire::it_802AA054(gobj as _, pos as _, vel as _, dir as _, rot as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftNs_SpecialS_ItemPKFireSpawn` as the Accessory4 callback entry.
pub unsafe extern "C" fn cb_ftNs_SpecialS_ItemPKFireSpawn(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftNs_SpecialS_ItemPKFireSpawn");
    ftNs_SpecialS_ItemPKFireSpawn(g);
}

/// Accessory4 callback (subaction event 0x60): spawns the PK Fire item once `throw_flags_b0` is set.
pub fn ftNs_SpecialS_ItemPKFireSpawn(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialS_ItemPKFireSpawn");
    unsafe {
        let fp = fpp(gobj);
        let ness_attr = (*fp).dat_attrs as *const ftNessAttributes;
        // `ftCheckThrowB0`-style test and clear of throw_flags_b0.
        let flag_result = if (*fp).anon_2210.anon_0.throw_flags_b0() != 0 {
            (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
            true
        } else {
            false
        };
        if flag_result {
            let mut item_bone_pos: Vec3 = core::mem::zeroed();
            let mut vel: Vec3 = core::mem::zeroed();
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_R2ndNa as usize)).joint, core::ptr::null(), &mut item_bone_pos);

            item_bone_pos.x = fmadds((*ness_attr).x30_PKFIRE_SPAWN_X, (*fp).facing_dir, item_bone_pos.x);
            item_bone_pos.y += (*ness_attr).x34_PKFIRE_SPAWN_Y;
            item_bone_pos.z = 0.0;

            let (launch, speed);
            if (*fp).ground_or_air == GA_Air {
                launch = (*ness_attr).x20_PKFIRE_AERIAL_LAUNCH_TRAJECTORY;
                speed = (*ness_attr).x24_PKFIRE_AERIAL_VELOCITY;
            } else {
                launch = (*ness_attr).x28_PKFIRE_GROUNDED_LAUNCH_TRAJECTORY;
                speed = (*ness_attr).x2C_PKFIRE_GROUNDED_VELOCITY;
            }

            vel.z = 0.0;
            vel.x = speed * cosf(launch) * (*fp).facing_dir;
            let launch_new = sinf(launch);
            vel.y = speed * launch_new;
            let rot = launch * (*fp).facing_dir;

            ext::it_802AA054(gobj, &mut item_bone_pos, &mut vel, (*fp).facing_dir, rot);
        }
    }
}

/// 0x80116C94: grounded PK Fire enter.
pub fn ftNs_SpecialS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialS_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialS as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(cb_ftNs_SpecialS_ItemPKFireSpawn);
    }
}

/// 0x80116D04: aerial PK Fire enter.
pub fn ftNs_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirS_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_SpecialAirS as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(cb_ftNs_SpecialS_ItemPKFireSpawn);
    }
}

pub fn ftNs_SpecialS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialS_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

pub fn ftNs_SpecialAirS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirS_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

pub fn ftNs_SpecialS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialS_Phys");
    st::ft_80084F3C(gobj);
}

pub fn ftNs_SpecialAirS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirS_Phys");
    st::ft_80084EEC(gobj);
}

pub fn ftNs_SpecialS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialS_Coll");
    if st::ft_800827A0(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

pub fn ftNs_SpecialAirS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNs_SpecialAirS_Coll");
    unsafe {
        let fp = fpp(gobj);
        let ness_attr = (*fp).dat_attrs as *const ftNessAttributes;
        if st::ft_80081D0C(gobj) != 0 {
            st::ftCo_LandingFallSpecial_Enter(gobj, 0, (*ness_attr).x38_PKFIRE_LANDING_LAG);
        }
    }
}
