//! src/melee/ft/kinds/ftGameWatch/ftgamewatchspecialhi.c: Mr. Game & Watch's Fire Rescue (up B).
//! One fused site (tools/rust-port/fma_sites.py): the Fire Rescue spawn height
//! `-((2.5f * scale) - y)`. Smoke-tested by tests/tu_ftgamewatchspecialhi.rs (the fused
//! expression is also checked bit-for-bit against the C there).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftgamewatch::{cb_ftGw_Init_OnDamage, ext as gw_ext};
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 3/5/13): re-exported under this TU.
pub use crate::chars::r13::ftGameWatch::ftgamewatchspecialhi::{ftGw_SpecialAirHi_Phys, ftGw_SpecialHi_Phys};
pub use crate::chars::r3::ftGameWatch::ftgamewatchspecialhi::ftGw_SpecialHi_ItemRescueSetNULL;
pub use crate::chars::r5::ftGameWatch::ftgamewatchspecialhi::ftGameWatch_SpecialHi_SetVars;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftgamewatch::ext::{lb_8000B1CC, LB_8000B1CC};
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
    // itgamewatchrescue.c
    pub static mut IT_802C8038: Option<fn(*mut HSD_GObj, *const Vec3, i32, i32, f32, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802C8038(owner: *mut HSD_GObj, pos: *const Vec3, part: i32, kind: i32, facing: f32, scale: f32) -> *mut HSD_GObj {
                match unsafe { IT_802C8038 } {
                    Some(f) => f(owner, pos, part, kind, facing, scale),
                    None => { return unsafe { melee_items::tu::itgamewatchrescue::it_802C8038(owner as _, pos as _, part as _, kind as _, facing as _, scale as _) as _ }; },
                }
            }
    pub static mut IT_802C8158: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C8158(item: *mut HSD_GObj)  {
                match unsafe { IT_802C8158 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchrescue::it_802C8158(item as _); } return; },
                }
            }
    pub static mut IT_802C81C8: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C81C8(item: *mut HSD_GObj)  {
                match unsafe { IT_802C81C8 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchrescue::it_802C81C8(item as _); } return; },
                }
            }
    pub static mut IT_802C81E8: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C81E8(item: *mut HSD_GObj)  {
                match unsafe { IT_802C81E8 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchrescue::it_802C81E8(item as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `stickGetDir` (ft/inlines.h).
#[inline(always)]
fn stick_get_dir(x1: f32, x2: f32) -> f32 {
    if x1 < x2 {
        -x1
    } else {
        x1
    }
}

/// `ftGw_SpecialHi_ItemRescueSetup` as a Fighter callback entry (accessory4_cb).
pub unsafe extern "C" fn cb_ftGw_SpecialHi_ItemRescueSetup(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_SpecialHi_ItemRescueSetup");
    ftGw_SpecialHi_ItemRescueSetup(g);
}
/// `ftGw_SpecialHi_ItemRescueEnterHitlag` as a Fighter callback entry (pre_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_SpecialHi_ItemRescueEnterHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_SpecialHi_ItemRescueEnterHitlag");
    ftGw_SpecialHi_ItemRescueEnterHitlag(g);
}
/// `ftGw_SpecialHi_ItemRescueExitHitlag` as a Fighter callback entry (post_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_SpecialHi_ItemRescueExitHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_SpecialHi_ItemRescueExitHitlag");
    ftGw_SpecialHi_ItemRescueExitHitlag(g);
}

/// Spawn height of the Fire Rescue item: `-((2.5f * scale) - y)`, fused by clang as one fmadds.
#[inline(always)]
pub fn rescue_spawn_y(scale: f32, y: f32) -> f32 {
    melee_core::ct!("rescue_spawn_y");
    -fmadds(2.5, scale, -y)
}

/// Create the Fire Rescue item.
pub fn ftGw_SpecialHi_ItemRescueSetup(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialHi_ItemRescueSetup");
    unsafe {
        let mut sp10 = Vec { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);
        if (*fp).u.gw.x226C_rescueGObj.is_null() {
            gw_ext::lb_8000B1CC((*(*fp).parts.add(FtPart_TopN as usize)).joint, core::ptr::null(), &mut sp10);
            sp10.y = rescue_spawn_y(gw_ext::ftCommon_GetModelScale(&*fp), sp10.y);
            let rescue_gobj = ext::it_802C8038(
                gobj,
                &sp10,
                FtPart_TopN as i32,
                (*fp).motion_id - ftGw_MS_SpecialHi as i32,
                (*fp).facing_dir,
                2.5,
            );
            (*fp).u.gw.x226C_rescueGObj = rescue_gobj;
            if !rescue_gobj.is_null() {
                (*fp).death2_cb = Some(cb_ftGw_Init_OnDamage);
                (*fp).take_dmg_cb = Some(cb_ftGw_Init_OnDamage);
            }
            (*fp).pre_hitlag_cb = Some(cb_ftGw_SpecialHi_ItemRescueEnterHitlag);
            (*fp).post_hitlag_cb = Some(cb_ftGw_SpecialHi_ItemRescueExitHitlag);
            (*fp).accessory4_cb = None;
        }
    }
}

/// Check whether G&W has left Fire Rescue (true = remove the item).
pub fn ftGw_SpecialHi_ItemCheckRescueRemove(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftGw_SpecialHi_ItemCheckRescueRemove");
    unsafe {
        let msid = (*fpp(gobj)).motion_id;
        !(msid == ftGw_MS_SpecialHi as FtMotionId || msid == ftGw_MS_SpecialAirHi as FtMotionId)
    }
}

/// Remove the Fire Rescue item.
pub fn ftGw_SpecialHi_ItemRescueRemove(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialHi_ItemRescueRemove");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x226C_rescueGObj.is_null() {
            ext::it_802C8158((*fp).u.gw.x226C_rescueGObj);
            ftGw_SpecialHi_ItemRescueSetNULL(gobj);
        }
    }
}

/// Apply hitlag to the Fire Rescue item.
pub fn ftGw_SpecialHi_ItemRescueEnterHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialHi_ItemRescueEnterHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x226C_rescueGObj.is_null() {
            ext::it_802C81C8((*fp).u.gw.x226C_rescueGObj);
        }
    }
}

/// Remove hitlag for the Fire Rescue item.
pub fn ftGw_SpecialHi_ItemRescueExitHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialHi_ItemRescueExitHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x226C_rescueGObj.is_null() {
            ext::it_802C81E8((*fp).u.gw.x226C_rescueGObj);
        }
    }
}

/// Grounded Fire Rescue Motion State handler.
pub fn ftGw_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fp).x74_self_accel.y = 0.0;
        (*fp).self_vel.y = 0.0;
        st::ftCommon_8007D60C(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, ftGw_MS_SpecialHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftGameWatch_SpecialHi_SetVars(gobj);
        st::ftAnim_8006EBA4(gobj);
        melee_fighter::ft0881::ft_80088510(&mut *fp, 290066, 127, 64);
    }
}

/// Aerial Fire Rescue Motion State handler.
pub fn ftGw_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirHi_Enter");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D60C(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, ftGw_MS_SpecialAirHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftGameWatch_SpecialHi_SetVars(gobj);
        st::ftAnim_8006EBA4(gobj);
        melee_fighter::ft0881::ft_80088510(&mut *fp, 290066, 127, 64);
    }
}

/// Grounded Fire Rescue Animation callback.
pub fn ftGw_SpecialHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialHi_Anim");
    ftGw_SpecialAirHi_Anim(gobj);
}

/// Aerial Fire Rescue Animation callback.
pub fn ftGw_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirHi_Anim");
    unsafe {
        let gaw_attrs = (*fpp(gobj)).dat_attrs as *const ftGameWatchAttributes;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if 0.0 == (*gaw_attrs).x60_GAMEWATCH_RESCUE_LANDING {
                st::ftCo_Fall_Enter(gobj);
                return;
            }
            st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*gaw_attrs).x60_GAMEWATCH_RESCUE_LANDING);
        }
    }
}

/// Grounded Fire Rescue IASA callback.
pub fn ftGw_SpecialHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialHi_IASA");
    ftGw_SpecialAirHi_IASA(gobj);
}

/// Aerial Fire Rescue IASA callback: aim the rescue once the stick passes the threshold.
pub fn ftGw_SpecialAirHi_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirHi_IASA");
    unsafe {
        let fp = fpp(gobj);
        let gaw_attrs = (*fp).dat_attrs as *const ftGameWatchAttributes;
        if (*fp).cmd_vars[0] == 0 {
            let stick_x = stick_get_dir((*fp).input.lstick[0].x, 0.0);
            if stick_x > (*gaw_attrs).x58_GAMEWATCH_RESCUE_STICK_RANGE {
                let stick_range_threshold = stick_x - (*gaw_attrs).x58_GAMEWATCH_RESCUE_STICK_RANGE;
                let facing_dir: f32 = if (*fp).input.lstick[0].x > 0.0 { 1.0 } else { -1.0 };
                let temp = stick_range_threshold * facing_dir;
                let angle = temp * (*gaw_attrs).x5C_GAMEWATCH_RESCUE_ANGLE_UNK;
                gw_ext::ftCommon_UpdateFacing(&mut *fp);
                // `(M_PI / 2) * fp->facing_dir`: double multiply, rounded to the f32 parameter.
                gw_ext::ftPartSetRotY(
                    &*fp,
                    0,
                    (core::f64::consts::FRAC_PI_2 * (*fp).facing_dir as f64) as f32,
                );
                (*fp).lstick_angle = -angle;
                (*fp).cmd_vars[0] = 1;
            }
        }
    }
}

/// Grounded Fire Rescue Collision callback.
pub fn ftGw_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialHi_Coll");
    ftGw_SpecialAirHi_Coll(gobj);
}

/// Aerial Fire Rescue Collision callback.
pub fn ftGw_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        let gaw_attrs = (*fp).dat_attrs as *const ftGameWatchAttributes;
        if (*fp).cur_anim_frame > 4.0 {
            if (*fp).self_vel.y >= 0.0 {
                if st::ft_80081D0C(gobj) != 0 {
                    ftGw_SpecialHi_ItemRescueRemove(gobj);
                    st::ftCommon_8007D7FC(&mut *fp);
                    st::ftCo_LandingFallSpecial_Enter(gobj, 0, (*gaw_attrs).x60_GAMEWATCH_RESCUE_LANDING);
                    (*fp).death2_cb = None;
                    (*fp).take_dmg_cb = None;
                }
            } else {
                let ledge_grab_dir = if 1.0 == (*fp).facing_dir { 1 } else { -1 };
                if st::ft_CheckGroundAndLedge(gobj, ledge_grab_dir) != 0 {
                    st::ftCo_LandingFallSpecial_Enter(gobj, 0, (*gaw_attrs).x60_GAMEWATCH_RESCUE_LANDING);
                } else if st::ftCliffCommon_80081298(gobj) != 0 {
                    melee_fighter::tu::ftcliffcommon::ftcliffcommon_80081370(gobj);
                }
            }
        }
    }
}
