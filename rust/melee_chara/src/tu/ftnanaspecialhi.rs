//! src/melee/ft/kinds/ftNana/ftnanaspecialhi.c: Nana follows Popo's up-B (Belay). Fused sites
//! (tools/rust-port/fma_sites.py): `ftNn_Init_801237F8` 2 (`cur_pos.x += 4*facing*scale`,
//! `cur_pos.y += 7*scale`) and `ftPp_SpecialHi_0_Phys` 1 (`-(8*facing - x)`), all `fmadds`.
//! Smoke-tested through recording hooks by tests/tu_ftnanaspecialhi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::*;

/// Already ported (round 3): re-exported under this TU.
pub use crate::chars::r3::ftNana::ftnanaspecialhi::{
    ftNn_Init_80123720, ftNn_Init_8012378C, ftNn_Init_801233F8, ftPp_SpecialHi_1_Coll,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics, except the
/// ones with a verified port in melee_fighter, which the unset hook calls.
pub mod ext {
    pub use crate::tu::ftnanaspecials::ext::{FTPARTSETROTY, FT_800849EC, PLAYER_GETENTITYATINDEX,
        ftPartSetRotY, ft_800849EC, lb_8000B1CC, Player_GetEntityAtIndex,
    };
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `M_PI_2 * facing_dir` (double math, narrowed at the f32 argument).
#[inline(always)]
fn pi_2_facing(facing: f32) -> f32 {
    (core::f64::consts::FRAC_PI_2 * facing as f64) as f32
}

/// `Collide_CeilingMask` (mp/forward.h): `Collide_CeilingPush | Collide_CeilingHug`.
const COLLIDE_CEILING_MASK: i32 = Collide_CeilingPush | Collide_CeilingHug;

/// `ftNn_Init_801230D0` (static): glue Nana to Popo's hand while he throws her.
pub fn ftNn_Init_801230D0(nana_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftNn_Init_801230D0");
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        let mut popo_vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut nana_vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        if !popo_gobj.is_null() {
            let popo_fp = fpp(popo_gobj);
            if ((*popo_fp).motion_id as i32) < 347 || (*popo_fp).motion_id as i32 > 352 {
                return false;
            }
            if (*nana_fp).facing_dir != (*popo_fp).facing_dir {
                (*nana_fp).facing_dir = (*popo_fp).facing_dir;
                ext::ftPartSetRotY(nana_fp, 0, pi_2_facing((*nana_fp).facing_dir));
            }
            ext::lb_8000B1CC(
                (*(*popo_fp).parts.add(FtPart_R4thNb as usize)).joint,
                core::ptr::null(),
                &mut popo_vec,
            );
            ext::lb_8000B1CC(
                (*(*nana_fp).parts.add(FtPart_XRotN as usize)).joint,
                core::ptr::null(),
                &mut nana_vec,
            );
            let x = (*nana_fp).cur_pos.x - nana_vec.x;
            let y = (*nana_fp).cur_pos.y - nana_vec.y;
            let z = (*nana_fp).cur_pos.z - nana_vec.z;
            (*nana_fp).cur_pos.x = popo_vec.x + x;
            (*nana_fp).cur_pos.y = popo_vec.y + y;
            (*nana_fp).cur_pos.z = popo_vec.z + z;
            if (*popo_fp).x2219_b5() != 0 {
                st::ftAnim_SetAnimRate(nana_gobj, 0.0);
            } else if (*nana_fp).frame_speed_mul != (*popo_fp).frame_speed_mul {
                st::ftAnim_SetAnimRate(nana_gobj, (*popo_fp).frame_speed_mul);
            }
        }
        true
    }
}

/// `fn_80123218` (static): the accessory4 callback; publishes Nana's hand position to Popo.
pub fn fn_80123218(nana_gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80123218");
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        if !popo_gobj.is_null() {
            let popo_fp = fpp(popo_gobj);
            let mut nana_vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::lb_8000B1CC(
                (*(*fpp(nana_gobj)).parts.add(FtPart_L4thNb as usize)).joint,
                core::ptr::null(),
                &mut nana_vec,
            );
            (*popo_fp).u.pp.x2240 = nana_vec;
        }
    }
}

/// `ftNn_Init_801232A4_inline` (static inline).
fn ftNn_Init_801232A4_inline(nana_gobj: *mut HSD_GObj) {
    unsafe {
        let nana_fp = fpp(nana_gobj);
        (*nana_fp).cmd_vars[0] = 0;
        (*nana_fp).set_x2222_b2(1);
    }
}

pub fn ftNn_Init_801232A4(nana_gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNn_Init_801232A4");
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        if !popo_gobj.is_null() {
            let popo_fp = fpp(popo_gobj);
            (*nana_fp).facing_dir = (*popo_fp).facing_dir;
        }
        if (*nana_fp).ground_or_air as i32 == GA_Ground as i32 {
            st::ftCommon_8007D60C(&mut *nana_fp);
        } else {
            (*nana_fp).x1968_jumpsUsed = crate::chars::caps_max_jumps(&*nana_fp) as _;
        }
        ftNn_Init_801233F8(nana_gobj);
        ftNn_Init_801232A4_inline(nana_gobj);
    }
}

pub fn ftPp_SpecialHi_0_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_SpecialHi_0_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftNn_Init_801237F8(gobj);
        }
    }
}

pub fn ftPp_SpecialHi_3_Anim(_gobj: *mut HSD_GObj) { melee_core::ct!("ftPp_SpecialHi_3_Anim");}
pub fn ftPp_SpecialHi_0_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftPp_SpecialHi_0_IASA");}
pub fn ftPp_SpecialHi_3_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftPp_SpecialHi_3_IASA");}

pub fn ftPp_SpecialHi_0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_SpecialHi_0_Phys");
    unsafe {
        let fp = fpp(gobj);
        if !ftNn_Init_801230D0(gobj) {
            // `-((8.0f * facing) - x)`: the multiply-subtract contracts.
            (*fp).cur_pos.x = -fmadds(8.0, (*fp).facing_dir, -(*fp).cur_pos.x);
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftPp_SpecialHi_3_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftPp_SpecialHi_3_Phys");}
pub fn ftPp_SpecialHi_0_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftPp_SpecialHi_0_Coll");}
pub fn ftPp_SpecialHi_3_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftPp_SpecialHi_3_Coll");}

pub fn ftPp_SpecialHi_1_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_SpecialHi_1_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftPp_SpecialHi_4_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_SpecialHi_4_Anim");
    unsafe {
        let fp = fpp(gobj);
        let attrs = (*fp).dat_attrs as *const ftIceClimberAttributes;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_80096900(gobj, 0, 1, 0, (*attrs).x130, (*attrs).x134);
        }
    }
}

pub fn ftPp_SpecialHi_1_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftPp_SpecialHi_1_IASA");}
pub fn ftPp_SpecialHi_4_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftPp_SpecialHi_4_IASA");}

pub fn ftPp_SpecialHi_1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_SpecialHi_1_Phys");
    st::ft_80084F3C(gobj);
}

pub fn ftPp_SpecialHi_4_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_SpecialHi_4_Phys");
    unsafe {
        let fp = fpp(gobj);
        let attrs = (*fp).dat_attrs as *const ftIceClimberAttributes;
        st::ftCommon_Fall(&mut *fp, (*attrs).x144, (*attrs).x148);
        if crate::chars::abs_c((*fp).input.lstick[0].x) > (*attrs).x138 {
            let co = (*fp).co_attrs;
            melee_fighter::ftcommon::calc_self_accel_drift_simple(
                &mut *fp,
                0.0,
                co.air_drift_stick_mul * (*attrs).xB0,
                co.air_drift_max * (*attrs).xB4,
            );
        } else if (*fp).self_vel.y < 0.0 {
            melee_fighter::ftcommon::calc_self_accel_deaccel_air(&mut *fp);
        }
    }
}

pub fn ftPp_SpecialHi_4_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_SpecialHi_4_Coll");
    unsafe {
        let fp = fpp(gobj);
        let attrs = (*fp).dat_attrs as *const ftIceClimberAttributes;
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            ftNn_Init_8012378C(gobj);
            return;
        }
        let env = (*fp).coll_data.env_flags as i32;
        if (env & Collide_LeftWallMask) != 0 && (*fp).self_vel.x > 0.0 {
            (*fp).self_vel.x *= -1.0 * (*attrs).x14C;
            (*fp).facing_dir *= -1.0;
            ext::ftPartSetRotY(fp, 0, pi_2_facing((*fp).facing_dir));
            return;
        }
        if (env & Collide_RightWallMask) != 0 && (*fp).self_vel.x < 0.0 {
            (*fp).self_vel.x *= -1.0 * (*attrs).x14C;
            (*fp).facing_dir *= -1.0;
            ext::ftPartSetRotY(fp, 0, pi_2_facing((*fp).facing_dir));
            return;
        }
        if (env & COLLIDE_CEILING_MASK) != 0 {
            (*fp).self_vel.y = 0.0;
        }
    }
}

pub fn ftNn_Init_801237F8(nana_gobj: *mut HSD_GObj) {
    melee_core::ct!("ftNn_Init_801237F8");
    unsafe {
        let nana_fp = fpp(nana_gobj);
        let attrs = (*nana_fp).dat_attrs as *const ftIceClimberAttributes;
        let popo_gobj = ext::Player_GetEntityAtIndex((*nana_fp).player_id as i32, 0);
        if !popo_gobj.is_null() {
            let popo_fp = fpp(popo_gobj);
            ext::ft_800849EC(popo_fp, nana_fp);
        }
        (*nana_fp).self_vel.x = (*nana_fp).facing_dir * ((*attrs).x13C * cosf((*attrs).x140));
        (*nana_fp).self_vel.y = (*attrs).x13C * sinf((*attrs).x140);
        let scale = (*nana_fp).x34_scale.y;
        // `cur_pos.x += 4.0f * facing * scale` / `cur_pos.y += 7.0f * scale`: both contract.
        (*nana_fp).cur_pos.x = fmadds(4.0 * (*nana_fp).facing_dir, scale, (*nana_fp).cur_pos.x);
        (*nana_fp).cur_pos.y = fmadds(7.0, scale, (*nana_fp).cur_pos.y);
        st::Fighter_ChangeMotionState(nana_gobj, 365, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*nana_fp).accessory4_cb = Some(crate::chars::cb::ftnanaspecialhi__fn_80123218);
    }
}
