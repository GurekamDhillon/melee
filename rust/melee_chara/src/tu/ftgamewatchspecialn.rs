//! src/melee/ft/kinds/ftGameWatch/ftgamewatchspecialn.c: Mr. Game & Watch's Chef (neutral B).
//! No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftgamewatchspecialn.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftgamewatch::ext as gw_ext;
use melee_types::*;

/// Already ported (rounds 2/4): re-exported under this TU.
pub use crate::chars::r2::ftGameWatch::ftgamewatchspecialn::{ftGw_SpecialAirN_Phys, ftGw_SpecialN_Phys};
pub use crate::chars::r4::ftGameWatch::ftgamewatchspecialn::ftGameWatch_SpecialN_SetVars;

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
    // baselib/random.c
    crate::port_hook!(HSD_RANDI, HSD_Randi, crate::adapt::HSD_Randi, (max: i32) -> i32);
    // itgamewatchchef.c
    pub static mut IT_802C837C: Option<fn(*mut HSD_GObj, *const Vec3, i32, i32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C837C(owner: *mut HSD_GObj, pos: *const Vec3, kind: i32, sausage: i32, facing: f32)  {
                match unsafe { IT_802C837C } {
                    Some(f) => f(owner, pos, kind, sausage, facing),
                    None => { unsafe { melee_items::tu::itgamewatchchef::it_802C837C(owner as _, pos as _, kind as _, sausage as _, facing as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// Ft_MF_KeepColAnimHitStatus | SkipHit | SkipModel | SkipMatAnim | SkipColAnim | UpdateCmd |
/// SkipItemVis | Unk19 | SkipModelPartVis | SkipModelFlags | Unk27.
const transition_flags: u32 = (Ft_MF_KeepColAnimHitStatus
    | Ft_MF_SkipHit
    | Ft_MF_SkipModel
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

/// `ftGw_SpecialN_CreateSausage` as a Fighter callback entry (accessory4_cb).
pub unsafe extern "C" fn cb_ftGw_SpecialN_CreateSausage(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_SpecialN_CreateSausage");
    ftGw_SpecialN_CreateSausage(g);
}

/// Spawn one Chef sausage (static in the C); picks a sausage kind that is neither of the last two.
pub fn ftGw_SpecialN_CreateSausage(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialN_CreateSausage");
    unsafe {
        let fp = fpp(gobj);
        let gaw_attrs = (*fp).dat_attrs as *const ftGameWatchAttributes;
        if (*fp).cmd_vars[0] != 0 {
            let mut var_r28: i32 = 0;
            (*fp).cmd_vars[0] = 0;
            if ((*fp).mv.gw.SpecialN.maxSausage as f32) < (*gaw_attrs).x1C_GAMEWATCH_CHEF_MAX {
                (*fp).mv.gw.SpecialN.maxSausage += 1;
                let vec1 = Vec { x: 2.5, y: 6.5, z: 0.0 };
                let mut vec0 = Vec { x: 0.0, y: 0.0, z: 0.0 };
                let bone = gw_ext::ftParts_GetBoneIndex(&*fp, FtPart_LThumbNb as i32);
                gw_ext::lb_8000B1CC((*(*fp).parts.add(bone as usize)).joint, &vec1, &mut vec0);
                let fp1 = fpp(gobj);
                let mut chef = ftGameWatchChef::zeroed();
                let mut n = 0usize;
                let mut i = 0;
                while i < 5 {
                    if i != (*fp1).u.gw.x2240_chefVar1 && i != (*fp1).u.gw.x2244_chefVar2 {
                        chef.sausageCount[n] = i;
                        n += 1;
                        var_r28 += 1;
                    }
                    i += 1;
                }
                let temp_r5 = chef.sausageCount[ext::HSD_Randi(var_r28) as usize];
                (*fp1).u.gw.x2244_chefVar2 = (*fp1).u.gw.x2240_chefVar1;
                (*fp1).u.gw.x2240_chefVar1 = temp_r5;
                ext::it_802C837C(gobj, &vec0, It_Kind_GameWatch_Chef as i32, temp_r5, (*fp).facing_dir);
            }
            (*fp).accessory4_cb = None;
        }
    }
}

/// Grounded Chef Action State handler.
pub fn ftGw_SpecialN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialN_Enter");
    unsafe {
        (*fpp(gobj)).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, ftGw_MS_SpecialN as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftGameWatch_SpecialN_SetVars(gobj);
    }
}

/// Aerial Chef Action State handler.
pub fn ftGw_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirN_Enter");
    unsafe {
        (*fpp(gobj)).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, ftGw_MS_SpecialAirN as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftGameWatch_SpecialN_SetVars(gobj);
    }
}

/// Grounded Chef Animation callback.
pub fn ftGw_SpecialN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialN_Anim");
    unsafe {
        let fp = fpp(gobj);
        let gaw_attrs = (*fp).dat_attrs as *const ftGameWatchAttributes;
        if (*fp).cmd_vars[2] != 0 {
            (*fp).cmd_vars[2] = 0;
            if ((*fp).mv.gw.SpecialN.maxSausage as f32) < (*gaw_attrs).x1C_GAMEWATCH_CHEF_MAX
                && (*fp).mv.gw.SpecialN.isChefLoopDisable == 0
            {
                ftGw_SpecialN_Loop(gobj, (*gaw_attrs).x18_GAMEWATCH_CHEF_LOOPFRAME);
            }
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

/// Aerial Chef Animation callback.
pub fn ftGw_SpecialAirN_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirN_Anim");
    unsafe {
        let fp = fpp(gobj);
        let gaw_attrs = (*fp).dat_attrs as *const ftGameWatchAttributes;
        if (*fp).cmd_vars[2] != 0 {
            (*fp).cmd_vars[2] = 0;
            if (*fp).mv.gw.SpecialN.isChefLoopDisable == 0
                && ((*fp).mv.gw.SpecialN.maxSausage as f32) < (*gaw_attrs).x1C_GAMEWATCH_CHEF_MAX
            {
                ftGw_SpecialAirN_Loop(gobj, (*gaw_attrs).x18_GAMEWATCH_CHEF_LOOPFRAME);
            }
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

/// Grounded Chef IASA callback.
pub fn ftGw_SpecialN_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialN_IASA");
    unsafe {
        let fp = fpp(gobj);
        let gaw_attrs = (*fp).dat_attrs as *const ftGameWatchAttributes;
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
            (*fp).mv.gw.SpecialN.isChefLoopDisable = 1;
        }
        if (*fp).cmd_vars[1] != 0
            && ((*fp).input.pressed_buttons & (HSD_PAD_B as u32)) != 0
            && ((*fp).mv.gw.SpecialN.maxSausage as f32) < (*gaw_attrs).x1C_GAMEWATCH_CHEF_MAX
        {
            ftGw_SpecialN_Loop(gobj, (*gaw_attrs).x18_GAMEWATCH_CHEF_LOOPFRAME);
        }
    }
}

/// Aerial Chef IASA callback.
pub fn ftGw_SpecialAirN_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirN_IASA");
    unsafe {
        let fp = fpp(gobj);
        let gaw_attrs = (*fp).dat_attrs as *const ftGameWatchAttributes;
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
            (*fp).mv.gw.SpecialN.isChefLoopDisable = 1;
        }
        if (*fp).cmd_vars[1] != 0
            && ((*fp).input.pressed_buttons & (HSD_PAD_B as u32)) != 0
            && ((*fp).mv.gw.SpecialN.maxSausage as f32) < (*gaw_attrs).x1C_GAMEWATCH_CHEF_MAX
        {
            ftGw_SpecialAirN_Loop(gobj, (*gaw_attrs).x18_GAMEWATCH_CHEF_LOOPFRAME);
        }
    }
}

/// Grounded Chef Collision callback.
pub fn ftGw_SpecialN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialN_Coll");
    if st::ft_800827A0(gobj) == 0 {
        ftGw_SpecialN_GroundToAir(gobj);
    }
}

/// Aerial Chef Collision callback.
pub fn ftGw_SpecialAirN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirN_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftGw_SpecialAirN_AirToGround(gobj);
    }
}

/// Ground -> air Chef Motion State handler.
pub fn ftGw_SpecialN_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialN_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftGw_MS_SpecialAirN as i32, transition_flags);
        (*fpp(gobj)).accessory4_cb = Some(cb_ftGw_SpecialN_CreateSausage);
    }
}

/// Air -> ground Chef Motion State handler.
pub fn ftGw_SpecialAirN_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirN_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftGw_MS_SpecialN as i32, transition_flags);
        (*fpp(gobj)).accessory4_cb = Some(cb_ftGw_SpecialN_CreateSausage);
    }
}

/// Re-enter the Chef loop at `anim_frame` (grounded).
pub fn ftGw_SpecialN_Loop(gobj: *mut HSD_GObj, anim_frame: f32) {
    melee_core::ct!("ftGw_SpecialN_Loop");
    unsafe {
        st::Fighter_ChangeMotionState(
            gobj,
            ftGw_MS_SpecialN as i32,
            transition_flags,
            anim_frame - 1.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
        let fp = fpp(gobj);
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[2] = 0;
        (*fp).mv.gw.SpecialN.isChefLoopDisable = 0;
        (*fp).accessory4_cb = Some(cb_ftGw_SpecialN_CreateSausage);
    }
}

/// Re-enter the Chef loop at `anim_frame` (aerial).
pub fn ftGw_SpecialAirN_Loop(gobj: *mut HSD_GObj, anim_frame: f32) {
    melee_core::ct!("ftGw_SpecialAirN_Loop");
    unsafe {
        st::Fighter_ChangeMotionState(
            gobj,
            ftGw_MS_SpecialAirN as i32,
            transition_flags,
            anim_frame - 1.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
        let fp = fpp(gobj);
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[2] = 0;
        (*fp).mv.gw.SpecialN.isChefLoopDisable = 0;
        (*fp).accessory4_cb = Some(cb_ftGw_SpecialN_CreateSausage);
    }
}
