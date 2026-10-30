//! src/melee/ft/kinds/ftGameWatch/ftgamewatchspeciallw.c: Mr. Game & Watch's Oil Panic (down B).
//! No fused sites (tools/rust-port/fma_sites.py): the release damage is `(int * f32)` stored to a
//! u32 and then `(f32)u32 + f32` stored to a u32 again, with the shipping build's saturating
//! float->unsigned conversion ([`fptoui_ppc`]). The pure parts are checked against the C by tests/tu_ftgamewatch.rs;
//! the rest has a smoke test through recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftgamewatch::{cb_ftGw_Init_OnDamage, ext as gw_ext};
use melee_types::*;

/// Already ported (rounds 2/3/4/13 and the item-check helper): re-exported under this TU.
pub use crate::chars::ftGameWatch::ftgamewatchspeciallw::ftGw_SpecialLw_ItemCheckPanicRemove;
pub use crate::chars::r13::ftGameWatch::ftgamewatchspeciallw::ftGw_SpecialLw_Phys;
pub use crate::chars::r2::ftGameWatch::ftgamewatchspeciallw::{
    ftGw_SpecialAirLwCatch_AirToGround, ftGw_SpecialAirLwCatch_Phys, ftGw_SpecialAirLwShoot_Phys,
    ftGw_SpecialLwCatch_GroundToAir, ftGw_SpecialLwCatch_Phys, ftGw_SpecialLwShoot_Phys,
};
pub use crate::chars::r3::ftGameWatch::ftgamewatchspeciallw::{ftGw_SpecialAirLwCatch_Coll, ftGw_SpecialLwCatch_Coll};
pub use crate::chars::r4::ftGameWatch::ftgamewatchspeciallw::ftGameWatch_SpecialLw_SetVars;

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
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    // itgamewatchpanic.c
    pub static mut IT_802C7D60: Option<fn(*mut HSD_GObj, *const Vec3, i32, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802C7D60(owner: *mut HSD_GObj, pos: *const Vec3, kind: i32, facing: f32) -> *mut HSD_GObj {
                match unsafe { IT_802C7D60 } {
                    Some(f) => f(owner, pos, kind, facing),
                    None => { return unsafe { melee_items::tu::itgamewatchpanic::it_802C7D60(owner as _, pos as _, kind as _, facing as _) as _ }; },
                }
            }
    pub static mut IT_802C7E94: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7E94(item: *mut HSD_GObj)  {
                match unsafe { IT_802C7E94 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchpanic::it_802C7E94(item as _); } return; },
                }
            }
    pub static mut IT_802C7EE0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7EE0(item: *mut HSD_GObj)  {
                match unsafe { IT_802C7EE0 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchpanic::it_802C7EE0(item as _); } return; },
                }
            }
    pub static mut IT_802C7F00: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7F00(item: *mut HSD_GObj)  {
                match unsafe { IT_802C7F00 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchpanic::it_802C7F00(item as _); } return; },
                }
            }
    // ft/ftcoll.c
    crate::port_hook!(FTCOLL_CREATEABSORBHIT, ftColl_CreateAbsorbHit, crate::adapt::ftColl_CreateAbsorbHit, (gobj: *mut HSD_GObj, desc: *const AbsorbDesc));
    crate::port_hook!(FTCOLL_8007ABD0, ftColl_8007ABD0, crate::adapt::ftColl_8007ABD0, (hit: *mut HitCapsule, dmg: u32, gobj: *mut HSD_GObj));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// Ft_MF_KeepGfx | KeepColAnimHitStatus | SkipHit | SkipMatAnim | SkipColAnim | UpdateCmd |
/// SkipItemVis | Unk19 | SkipModelPartVis | SkipModelFlags | Unk27.
const transition_flags0: u32 = (Ft_MF_KeepGfx
    | Ft_MF_KeepColAnimHitStatus
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

/// Ft_MF_KeepGfx | SkipHit | SkipMatAnim | SkipColAnim | UpdateCmd | SkipItemVis | Unk19 |
/// SkipModelPartVis | SkipModelFlags | Unk27.
const transition_flags1: u32 = (Ft_MF_KeepGfx
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

/// The frame of the loop animation at which the bucket re-fills (`anim_update_frame`).
const anim_update_frame: i32 = 38;

/// C `(u32) x` for an f32 in the shipping build: gwtool lowers float->int conversions to
/// saturating ones (NaN -> 0, negative -> 0, > u32::MAX -> u32::MAX), which is Rust's `as`.
#[inline(always)]
pub fn fptoui_ppc(x: f32) -> u32 {
    melee_core::ct!("fptoui_ppc");
    x as u32
}

/// The release damage of the Oil Panic: `cmd_vars[1] = (u32)(damage * mul); cmd_vars[1] =
/// (u32)((f32)cmd_vars[1] + add)`.
#[inline(always)]
pub fn panic_release_damage(panic_damage: i32, mul: f32, add: f32) -> u32 {
    melee_core::ct!("panic_release_damage");
    let v = fptoui_ppc(panic_damage as f32 * mul);
    fptoui_ppc(v as f32 + add)
}

/// `ftGw_SpecialLw_ItemPanicSetup` as a Fighter callback entry (accessory4_cb).
pub unsafe extern "C" fn cb_ftGw_SpecialLw_ItemPanicSetup(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_SpecialLw_ItemPanicSetup");
    ftGw_SpecialLw_ItemPanicSetup(g);
}
/// `ftGw_SpecialLw_ItemPanicEnterHitlag` as a Fighter callback entry (pre_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_SpecialLw_ItemPanicEnterHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_SpecialLw_ItemPanicEnterHitlag");
    ftGw_SpecialLw_ItemPanicEnterHitlag(g);
}
/// `ftGw_SpecialLw_ItemPanicExitHitlag` as a Fighter callback entry (post_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_SpecialLw_ItemPanicExitHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_SpecialLw_ItemPanicExitHitlag");
    ftGw_SpecialLw_ItemPanicExitHitlag(g);
}

/// Create the Oil Panic item.
pub fn ftGw_SpecialLw_ItemPanicSetup(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_ItemPanicSetup");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.gw.x2268_panicGObj.is_null() {
            let mut vec = Vec { x: 0.0, y: 0.0, z: 0.0 };
            gw_ext::lb_8000B1CC((*(*fp).parts.add(FtPart_TopN as usize)).joint, core::ptr::null(), &mut vec);
            (*fp).u.gw.x2268_panicGObj = ext::it_802C7D60(gobj, &vec, 0, (*fp).facing_dir);
        }
        if !(*fp).u.gw.x2268_panicGObj.is_null() {
            (*fp).death2_cb = Some(cb_ftGw_Init_OnDamage);
            (*fp).take_dmg_cb = Some(cb_ftGw_Init_OnDamage);
        }
        (*fp).pre_hitlag_cb = Some(cb_ftGw_SpecialLw_ItemPanicEnterHitlag);
        (*fp).post_hitlag_cb = Some(cb_ftGw_SpecialLw_ItemPanicExitHitlag);
        (*fp).accessory4_cb = None;
    }
}

/// Set Oil Panic flags + clear pointers.
pub fn ftGw_SpecialLw_ItemPanicSetFlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_ItemPanicSetFlag");
    unsafe {
        let fp = fpp(gobj);
        ftGw_SpecialLw_ItemPanicExitHitlag(gobj);
        (*fp).u.gw.x2268_panicGObj = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

/// Remove the Oil Panic item.
pub fn ftGw_SpecialLw_ItemPanicRemove(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_ItemPanicRemove");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2268_panicGObj.is_null() {
            ext::it_802C7E94((*fp).u.gw.x2268_panicGObj);
            ftGw_SpecialLw_ItemPanicSetFlag(gobj);
        }
    }
}

/// Apply hitlag to the Oil Panic item.
pub fn ftGw_SpecialLw_ItemPanicEnterHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_ItemPanicEnterHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2268_panicGObj.is_null() {
            ext::it_802C7EE0((*fp).u.gw.x2268_panicGObj);
        }
    }
}

/// Remove hitlag for the Oil Panic item.
pub fn ftGw_SpecialLw_ItemPanicExitHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_ItemPanicExitHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2268_panicGObj.is_null() {
            ext::it_802C7F00((*fp).u.gw.x2268_panicGObj);
        }
    }
}

/// Update the Oil Panic charge level indicator models.
pub fn ftGw_SpecialLw_UpdateBucketModel(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_UpdateBucketModel");
    unsafe {
        let fp = fpp(gobj);
        st::ftParts_80074B0C(gobj, 5, 2);
        let (a, b, c) = match (*fp).u.gw.x2238_panicCharge {
            x if x == ftGw_Panic_Empty as i32 => (-1, -1, -1),
            x if x == ftGw_Panic_Low as i32 => (0, -1, -1),
            x if x == ftGw_Panic_Mid as i32 => (0, 0, -1),
            // ftGw_Panic_Full and default
            _ => (0, 0, 0),
        };
        st::ftParts_80074B0C(gobj, 6, a);
        st::ftParts_80074B0C(gobj, 7, b);
        st::ftParts_80074B0C(gobj, 8, c);
    }
}

/// Grounded Oil Panic Start Motion State handler.
pub fn ftGw_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_Enter");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.gw.x2238_panicCharge >= ftGw_Panic_Full as i32 {
            ftGw_SpecialLwShoot_ReleaseOil(gobj);
            return;
        }
        (*fp).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, ftGw_MS_SpecialLw as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftGameWatch_SpecialLw_SetVars(gobj);
    }
}

/// Aerial Oil Panic Start Motion State handler.
pub fn ftGw_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirLw_Enter");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftGameWatchAttributes;
        if (*fp).u.gw.x2238_panicCharge >= ftGw_Panic_Full as i32 {
            ftGw_SpecialAirLwShoot_ReleaseOil(gobj);
            return;
        }
        (*fp).self_vel.x /= (*sa).x64_GAMEWATCH_PANIC_MOMENTUM_PRESERVE;
        (*fp).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, ftGw_MS_SpecialAirLw as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ftGameWatch_SpecialLw_SetVars(gobj);
    }
}

/// `ftGameWatch_SpecialLw_UpdateVars` (static inline): spawn the absorb hit on the first frame.
unsafe fn update_vars(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let ftcmd_var = (*fp).cmd_vars[0];
    let sa = (*fp).dat_attrs as *const ftGameWatchAttributes;
    if ftcmd_var == 1 {
        (*fp).cmd_vars[0] = 2;
        ext::ftColl_CreateAbsorbHit(gobj, &(*sa).x80_GAMEWATCH_PANIC_ABSORPTION);
    } else if ftcmd_var == 0 {
        (*fp).set_x2218_b6(0);
    }
    if (*fp).cmd_vars[0] != 0 {
        ftGw_SpecialLw_UpdateBucketModel(gobj);
    }
}

/// Grounded Oil Panic Loop Animation callback.
pub fn ftGw_SpecialLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_Anim");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cur_anim_frame == anim_update_frame as f32 && (*fp).mv.gw.SpecialLw.isRelease == 0 {
            ftGw_SpecialLw_UpdateAction(gobj, 5.0);
        }
        update_vars(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

/// Aerial Oil Panic Loop Animation callback.
pub fn ftGw_SpecialAirLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirLw_Anim");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cur_anim_frame == anim_update_frame as f32 && (*fp).mv.gw.SpecialLw.isRelease == 0 {
            ftGw_SpecialAirLw_UpdateAction(gobj, 5.0);
        }
        update_vars(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

/// The IASA body shared by the grounded and aerial loops: count down the turn lock, turn on a
/// full stick press, and latch the release when B is let go.
unsafe fn iasa(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let turn_frames = (*fp).mv.gw.SpecialLw.turnFrames;
    let sa = (*fp).dat_attrs as *const ftGameWatchAttributes;
    if turn_frames > 0 {
        (*fp).mv.gw.SpecialLw.turnFrames = turn_frames - 1;
    } else {
        let mut stick_range = (*fp).input.lstick[0].x;
        if stick_range < 0.0 {
            stick_range = -stick_range;
        }
        if stick_range > melee_fighter::hooks::common_data().horizontal_stick_deadzone {
            let facing_dir = (*fp).facing_dir;
            gw_ext::ftCommon_UpdateFacing(&mut *fp);
            if facing_dir != (*fp).facing_dir {
                (*fp).mv.gw.SpecialLw.turnFrames = (*sa).x7C_GAMEWATCH_PANIC_TURN_FRAMES as i32;
            }
        }
    }
    if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
        (*fp).mv.gw.SpecialLw.isRelease = 1;
    }
}

/// Grounded Oil Panic Loop IASA callback.
pub fn ftGw_SpecialLw_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_IASA");
    unsafe { iasa(gobj) }
}

/// Aerial Oil Panic Loop IASA callback.
pub fn ftGw_SpecialAirLw_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirLw_IASA");
    unsafe { iasa(gobj) }
}

/// Aerial Oil Panic Loop Physics callback.
pub fn ftGw_SpecialAirLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirLw_Phys");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftGameWatchAttributes;
        st::ftCommon_Fall(&mut *fp, (*sa).x6C_GAMEWATCH_PANIC_FALL_ACCEL, (*sa).x70_GAMEWATCH_PANIC_VEL_Y_MAX);
        melee_fighter::ftcommon::calc_self_accel_deaccel(&mut *fp, (*sa).x68_GAMEWATCH_PANIC_MOMENTUM_MUL);
        st::ftColl_8007AF10(gobj);
    }
}

/// Grounded Oil Panic Loop Collision callback.
pub fn ftGw_SpecialLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_Coll");
    if st::ft_800827A0(gobj) == 0 {
        ftGw_SpecialLw_GroundToAir(gobj);
    }
}

/// Aerial Oil Panic Loop Collision callback.
pub fn ftGw_SpecialAirLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirLw_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftGw_SpecialAirLw_AirToGround(gobj);
    }
}

/// `ftGameWatch_SpecialLw_UpdateVarsColl` (static inline): the absorb hit is re-created on the
/// state change when it is already running (cmd_vars[0] == 2).
unsafe fn update_vars_coll(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let ftcmd_var = (*fp).cmd_vars[0];
    let sa = (*fp).dat_attrs as *const ftGameWatchAttributes;
    if ftcmd_var == 2 {
        ext::ftColl_CreateAbsorbHit(gobj, &(*sa).x80_GAMEWATCH_PANIC_ABSORPTION);
    } else if ftcmd_var == 0 {
        (*fp).set_x2218_b6(0);
    }
    if (*fp).cmd_vars[0] != 0 {
        ftGw_SpecialLw_UpdateBucketModel(gobj);
    }
}

/// Ground -> air Oil Panic Motion State handler.
pub fn ftGw_SpecialLw_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftGw_MS_SpecialAirLw as i32, transition_flags0);
        update_vars_coll(gobj);
    }
}

/// Air -> ground Oil Panic Motion State handler.
pub fn ftGw_SpecialAirLw_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirLw_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftGw_MS_SpecialLw as i32, transition_flags0);
        update_vars_coll(gobj);
    }
}

/// `ftGameWatch_SpecialLw_UpdateVarsAction` (static inline).
unsafe fn update_vars_action(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let sa = (*fp).dat_attrs as *const ftGameWatchAttributes;
    if (*fp).cmd_vars[0] >= 1 {
        (*fp).cmd_vars[0] = 2;
        ext::ftColl_CreateAbsorbHit(gobj, &(*sa).x80_GAMEWATCH_PANIC_ABSORPTION);
    }
    if (*fp).cmd_vars[0] != 0 {
        ftGw_SpecialLw_UpdateBucketModel(gobj);
    }
}

/// Restart the grounded loop at `anim_frame` (one frame early, as the C).
pub fn ftGw_SpecialLw_UpdateAction(gobj: *mut HSD_GObj, anim_frame: f32) {
    melee_core::ct!("ftGw_SpecialLw_UpdateAction");
    unsafe {
        st::Fighter_ChangeMotionState(
            gobj,
            ftGw_MS_SpecialLw as i32,
            transition_flags1,
            anim_frame - 1.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
        update_vars_action(gobj);
    }
}

/// Restart the aerial loop at `anim_frame` (one frame early, as the C).
pub fn ftGw_SpecialAirLw_UpdateAction(gobj: *mut HSD_GObj, anim_frame: f32) {
    melee_core::ct!("ftGw_SpecialAirLw_UpdateAction");
    unsafe {
        st::Fighter_ChangeMotionState(
            gobj,
            ftGw_MS_SpecialAirLw as i32,
            transition_flags1,
            anim_frame - 1.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
        update_vars_action(gobj);
    }
}

/// Grounded Oil Panic Fill Animation callback.
pub fn ftGw_SpecialLwCatch_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLwCatch_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) != 0 {
            return;
        }
        if (*fp).u.gw.x2238_panicCharge >= ftGw_Panic_Full as i32 {
            st::ft_8008A2BC(gobj);
            return;
        }
        st::Fighter_ChangeMotionState(
            gobj,
            ftGw_MS_SpecialLw as i32,
            transition_flags1,
            4.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
        update_vars_action(gobj);
    }
}

/// Aerial Oil Panic Fill Animation callback.
pub fn ftGw_SpecialAirLwCatch_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirLwCatch_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) != 0 {
            return;
        }
        if (*fp).u.gw.x2238_panicCharge >= ftGw_Panic_Full as i32 {
            st::ftCo_Fall_Enter(gobj);
            return;
        }
        st::Fighter_ChangeMotionState(
            gobj,
            ftGw_MS_SpecialAirLw as i32,
            transition_flags1,
            4.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
        update_vars_action(gobj);
    }
}

/// Grounded Oil Panic Fill IASA callback.
pub fn ftGw_SpecialLwCatch_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftGw_SpecialLwCatch_IASA");}

/// Aerial Oil Panic Fill IASA callback.
pub fn ftGw_SpecialAirLwCatch_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftGw_SpecialAirLwCatch_IASA");}

/// Check to enter grounded or aerial Oil Panic Fill (the absorb think).
pub fn ftGw_SpecialLw_AbsorbThink_DecideAction(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLw_AbsorbThink_DecideAction");
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.gw.x2238_panicCharge += (*fp).AbsorbAttr.x1A48_hitsTaken;
        (*fp).u.gw.x223C_panicDamage += (*fp).AbsorbAttr.x1A44_damageTaken;
        if (*fp).u.gw.x2238_panicCharge >= ftGw_Panic_Full as i32 {
            melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *fp, 5, 0);
        }
        let msid = if (*fp).ground_or_air == GA_Ground {
            ftGw_MS_SpecialLwCatch
        } else {
            ftGw_MS_SpecialAirLwCatch
        };
        st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftGw_SpecialLw_UpdateBucketModel(gobj);
    }
}

/// `ftGameWatch_SpecialLwShoot_ApplyDamage` (static inline): hand the stored damage to every
/// enabled hit capsule.
unsafe fn shoot_apply_damage(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let mut i = 0;
    while i < 4 {
        if (*fp).x914[i].state == HitCapsule_Enabled {
            ext::ftColl_8007ABD0(&mut (*fp).x914[i], (*fp).cmd_vars[1], gobj);
        }
        i += 1;
    }
}

/// Grounded Oil Panic Release Animation callback.
pub fn ftGw_SpecialLwShoot_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLwShoot_Anim");
    unsafe {
        shoot_apply_damage(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

/// Aerial Oil Panic Release Animation callback.
pub fn ftGw_SpecialAirLwShoot_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirLwShoot_Anim");
    unsafe {
        shoot_apply_damage(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

/// Grounded Oil Panic Release IASA callback.
pub fn ftGw_SpecialLwShoot_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftGw_SpecialLwShoot_IASA");}

/// Aerial Oil Panic Release IASA callback.
pub fn ftGw_SpecialAirLwShoot_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftGw_SpecialAirLwShoot_IASA");}

/// Grounded Oil Panic Release Collision callback.
pub fn ftGw_SpecialLwShoot_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLwShoot_Coll");
    if st::ft_80082708(gobj) == 0 {
        ftGw_SpecialLwShoot_GroundToAir(gobj);
    }
}

/// Aerial Oil Panic Release Collision callback.
pub fn ftGw_SpecialAirLwShoot_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirLwShoot_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftGw_SpecialAirLwShoot_AirToGround(gobj);
    }
}

/// Ground -> air Oil Panic Release Motion State handler.
pub fn ftGw_SpecialLwShoot_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLwShoot_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftGw_MS_SpecialAirLwShoot as i32, transition_flags0);
        ftGw_SpecialLw_UpdateBucketModel(gobj);
        (*fp).accessory4_cb = Some(cb_ftGw_SpecialLw_ItemPanicSetup);
    }
}

/// Air -> ground Oil Panic Release Motion State handler.
pub fn ftGw_SpecialAirLwShoot_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirLwShoot_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftGw_MS_SpecialLwShoot as i32, transition_flags0);
        ftGw_SpecialLw_UpdateBucketModel(gobj);
        (*fp).accessory4_cb = Some(cb_ftGw_SpecialLw_ItemPanicSetup);
    }
}

/// Shared tail of both ReleaseOil functions: bank the damage, empty the bucket.
unsafe fn release_oil_tail(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    let sa = (*fp).dat_attrs as *const ftGameWatchAttributes;
    (*fp).cmd_vars[1] = panic_release_damage(
        (*fp).u.gw.x223C_panicDamage,
        (*sa).x78_GAMEWATCH_PANIC_DAMAGE_MUL,
        (*sa).x74_GAMEWATCH_PANIC_DAMAGE_ADD,
    );
    (*fp).u.gw.x2238_panicCharge = ftGw_Panic_Empty as i32;
    (*fp).u.gw.x223C_panicDamage = 0;
    ftGw_SpecialLw_UpdateBucketModel(gobj);
    (*fp).accessory4_cb = Some(cb_ftGw_SpecialLw_ItemPanicSetup);
}

/// Enter SpecialLwShoot and calculate the damage.
pub fn ftGw_SpecialLwShoot_ReleaseOil(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialLwShoot_ReleaseOil");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, ftGw_MS_SpecialLwShoot as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        release_oil_tail(gobj);
    }
}

/// Enter SpecialAirLwShoot and calculate the damage.
pub fn ftGw_SpecialAirLwShoot_ReleaseOil(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirLwShoot_ReleaseOil");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, ftGw_MS_SpecialAirLwShoot as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        release_oil_tail(gobj);
    }
}
