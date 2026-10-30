//! src/melee/ft/kinds/ftGameWatch/ftgamewatchspecials.c: Mr. Game & Watch's Judgement (side B).
//! No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftgamewatchspecials.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftgamewatch::{cb_ftGw_Init_OnDamage, ext as gw_ext};
use melee_types::*;

/// Already ported (rounds 2/4/12 and the item-check helper): re-exported under this TU.
pub use crate::chars::ftGameWatch::ftgamewatchspecials::ftGw_SpecialS_ItemCheckJudgementRemove;
pub use crate::chars::r12::ftGameWatch::ftgamewatchspecials::{ftGw_SpecialAirS_IASA, ftGw_SpecialS_IASA};
pub use crate::chars::r2::ftGameWatch::ftgamewatchspecials::{ftGw_SpecialAirS_Anim, ftGw_SpecialS_Anim};
pub use crate::chars::r4::ftGameWatch::ftgamewatchspecials::{
    ftGameWatch_SpecialS_SetCall, ftGameWatch_SpecialS_SetVars, ftGw_SpecialS_Phys,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftgamewatch::ext::{lb_8000B1CC, LB_8000B1CC};
    pub use crate::tu::ftgamewatchspecialn::ext::{HSD_Randi, HSD_RANDI};
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
    // itgamewatchjudge.c
    pub static mut IT_802C7774: Option<fn(f32, *mut HSD_GObj, *const Vec3, i32, i32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802C7774(facing: f32, owner: *mut HSD_GObj, pos: *const Vec3, part: i32, judge: i32) -> *mut HSD_GObj {
                match unsafe { IT_802C7774 } {
                    Some(f) => f(facing, owner, pos, part, judge),
                    None => { return unsafe { melee_items::tu::itgamewatchjudge::it_802C7774(facing as _, owner as _, pos as _, part as _, judge as _) as _ }; },
                }
            }
    pub static mut IT_802C7A84: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7A84(item: *mut HSD_GObj)  {
                match unsafe { IT_802C7A84 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchjudge::it_802C7A84(item as _); } return; },
                }
            }
    pub static mut IT_802C7AD0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7AD0(item: *mut HSD_GObj)  {
                match unsafe { IT_802C7AD0 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchjudge::it_802C7AD0(item as _); } return; },
                }
            }
    pub static mut IT_802C7AF0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7AF0(item: *mut HSD_GObj)  {
                match unsafe { IT_802C7AF0 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchjudge::it_802C7AF0(item as _); } return; },
                }
            }
    // it/items.c
    pub static mut IT_8028FAF4: Option<fn(*mut HSD_GObj, *const Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn it_8028FAF4(owner: *mut HSD_GObj, pos: *const Vec3)  {
                match unsafe { IT_8028FAF4 } {
                    Some(f) => f(owner, pos),
                    None => { unsafe { let _ = melee_items::tu::itfoods::it_8028FAF4(owner as _, pos as _); } return; },
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

/// `ftGw_SpecialS_ItemJudgementSetup` as a Fighter callback entry (accessory4_cb).
pub unsafe extern "C" fn cb_ftGw_SpecialS_ItemJudgementSetup(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_SpecialS_ItemJudgementSetup");
    ftGw_SpecialS_ItemJudgementSetup(g);
}
/// `ftGw_SpecialS_ItemJudgementEnterHitlag` as a Fighter callback entry (pre_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_SpecialS_ItemJudgementEnterHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_SpecialS_ItemJudgementEnterHitlag");
    ftGw_SpecialS_ItemJudgementEnterHitlag(g);
}
/// `ftGw_SpecialS_ItemJudgementExitHitlag` as a Fighter callback entry (post_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_SpecialS_ItemJudgementExitHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_SpecialS_ItemJudgementExitHitlag");
    ftGw_SpecialS_ItemJudgementExitHitlag(g);
}

/// Create the Judgement item (and the number-9 effect item).
pub fn ftGw_SpecialS_ItemJudgementSetup(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialS_ItemJudgementSetup");
    unsafe {
        let mut sp20 = Vec { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);
        if (*fpp(gobj)).cmd_vars[1] != 0 {
            (*fp).cmd_vars[1] = 0;
            let r_thumb = gw_ext::ftParts_GetBoneIndex(&*fp, FtPart_RThumbNb as i32);
            gw_ext::lb_8000B1CC((*(*fp).parts.add(r_thumb as usize)).joint, core::ptr::null(), &mut sp20);
            (*fp).u.gw.x2264_judgementGObj = ext::it_802C7774(
                (*fp).facing_dir,
                gobj,
                &sp20,
                gw_ext::ftParts_GetBoneIndex(&*fp, FtPart_RThumbNb as i32),
                (*fp).u.gw.x222C_judgeVar1,
            );
            if (*fp).u.gw.x222C_judgeVar1 == 6 {
                let sp14 = Vec { x: 0.0, y: 5.0, z: 0.0 };
                let l_thumb = gw_ext::ftParts_GetBoneIndex(&*fp, FtPart_LThumbNb as i32);
                gw_ext::lb_8000B1CC((*(*fp).parts.add(l_thumb as usize)).joint, &sp14, &mut sp20);
                ext::it_8028FAF4(gobj, &sp20);
            }
            if !(*fp).u.gw.x2264_judgementGObj.is_null() {
                (*fp).death2_cb = Some(cb_ftGw_Init_OnDamage);
                (*fp).take_dmg_cb = Some(cb_ftGw_Init_OnDamage);
            }
            (*fp).pre_hitlag_cb = Some(cb_ftGw_SpecialS_ItemJudgementEnterHitlag);
            (*fp).post_hitlag_cb = Some(cb_ftGw_SpecialS_ItemJudgementExitHitlag);
            (*fp).accessory4_cb = None;
        }
    }
}

/// Set Judgement flags + clear pointers.
pub fn ftGw_SpecialS_ItemJudgementSetFlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialS_ItemJudgementSetFlag");
    unsafe {
        let fp = fpp(gobj);
        ftGw_SpecialS_ItemJudgementExitHitlag(gobj);
        (*fp).u.gw.x2264_judgementGObj = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

/// Remove the Judgement item.
pub fn ftGw_SpecialS_ItemJudgementRemove(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialS_ItemJudgementRemove");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2264_judgementGObj.is_null() {
            ext::it_802C7A84((*fp).u.gw.x2264_judgementGObj);
            ftGw_SpecialS_ItemJudgementSetFlag(gobj);
        }
    }
}

/// Apply hitlag to the Judgement item (static in the C).
pub fn ftGw_SpecialS_ItemJudgementEnterHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialS_ItemJudgementEnterHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2264_judgementGObj.is_null() {
            ext::it_802C7AD0((*fp).u.gw.x2264_judgementGObj);
        }
    }
}

/// Remove hitlag for the Judgement item (static in the C).
pub fn ftGw_SpecialS_ItemJudgementExitHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialS_ItemJudgementExitHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2264_judgementGObj.is_null() {
            ext::it_802C7AF0((*fp).u.gw.x2264_judgementGObj);
        }
    }
}

/// Roll the Judgement number (1..9, weighted, never the last two). Returns it.
///
/// The C leaves the unused tail of both 9-entry arrays and `result` uninitialised; the shipping
/// build compiles with `-ftrivial-auto-var-init=zero`, so they are zero here.
pub fn ftGw_SpecialS_GetRandomInt(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftGw_SpecialS_GetRandomInt");
    unsafe {
        let fp = fpp(gobj);
        let sa = (*fp).dat_attrs as *const ftGameWatchAttributes;
        let mut gw_judge0 = ftGameWatchJudge::zeroed();
        let mut gw_judge1 = ftGameWatchJudge::zeroed();
        let mut i: usize = 0;
        let mut rand: i32 = 0;
        let judge_max: i32 = 9;
        let mut judge: i32 = 0;
        while judge < judge_max {
            if judge != (*fp).u.gw.x222C_judgeVar1 && judge != (*fp).u.gw.x2230_judgeVar2 {
                gw_judge0.rollVar[i] = judge;
                rand = rand.wrapping_add((*sa).x34_GAMEWATCH_JUDGE_ROLL[judge as usize]);
                gw_judge1.rollVar[i] = rand;
                i += 1;
            }
            judge += 1;
        }
        rand = ext::HSD_Randi(rand);
        let mut result: i32 = 0;
        let mut i = 0;
        while i < gw_judge1.rollVar.len() {
            if rand < gw_judge1.rollVar[i] {
                result = gw_judge0.rollVar[i];
                break;
            }
            i += 1;
        }
        (*fp).u.gw.x2230_judgeVar2 = (*fp).u.gw.x222C_judgeVar1;
        (*fp).u.gw.x222C_judgeVar1 = result;
        result
    }
}

/// Grounded Judgement Motion State handler.
pub fn ftGw_SpecialS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialS_Enter");
    unsafe {
        let fp = fpp(gobj);
        (*fpp(gobj)).self_vel.y = 0.0;
        ftGw_SpecialS_GetRandomInt(gobj);
        st::Fighter_ChangeMotionState(
            gobj,
            (*fp).u.gw.x222C_judgeVar1 + ftGw_MS_SpecialS1 as i32,
            0,
            0.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
        ftGameWatch_SpecialS_SetVars(gobj);
    }
}

/// Aerial Judgement Motion State handler.
pub fn ftGw_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirS_Enter");
    unsafe {
        let fp = fpp(gobj);
        let gaw_attrs = (*fp).dat_attrs as *const ftGameWatchAttributes;
        (*fpp(gobj)).self_vel.x /= (*gaw_attrs).x20_GAMEWATCH_JUDGE_MOMENTUM_PRESERVE;
        ftGw_SpecialS_GetRandomInt(gobj);
        st::Fighter_ChangeMotionState(
            gobj,
            (*fp).u.gw.x222C_judgeVar1 + ftGw_MS_SpecialAirS1 as i32,
            0,
            0.0,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        st::ftAnim_8006EBA4(gobj);
        ftGameWatch_SpecialS_SetVars(gobj);
    }
}

/// Aerial Judgement Physics callback.
pub fn ftGw_SpecialAirS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirS_Phys");
    unsafe {
        let fp = fpp(gobj);
        let ftcmd_var = (*fp).cmd_vars[0];
        let gaw_attrs = (*fp).dat_attrs as *const ftGameWatchAttributes;
        if ftcmd_var >= 1 {
            if ftcmd_var == 1 {
                (*fp).cmd_vars[0] = 2;
                if (*fp).u.gw.x2234 == 0 {
                    (*fp).u.gw.x2234 = 1;
                    (*fp).self_vel.y = (*gaw_attrs).x28_GAMEWATCH_JUDGE_VEL_Y;
                } else {
                    (*fp).self_vel.y = 0.0;
                }
            }
            st::ftCommon_Fall(
                &mut *fp,
                (*gaw_attrs).x2C_GAMEWATCH_JUDGE_FRICTION1,
                (*gaw_attrs).x30_GAMEWATCH_JUDGE_FRICTION2,
            );
        } else {
            melee_fighter::ftcommon::fall_basic(&mut *fp);
        }
        melee_fighter::ftcommon::calc_self_accel_deaccel(&mut *fp, (*gaw_attrs).x24_GAMEWATCH_JUDGE_MOMENTUM_MUL);
    }
}

/// Grounded Judgement Collision callback.
pub fn ftGw_SpecialS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialS_Coll");
    if st::ft_800827A0(gobj) == 0 {
        ftGw_SpecialS_GroundToAir(gobj);
    }
}

/// Aerial Judgement Collision callback.
pub fn ftGw_SpecialAirS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirS_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftGw_SpecialAirS_AirToGround(gobj);
    }
}

/// Ground -> air Judgement Motion State handler (static in the C).
pub fn ftGw_SpecialS_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialS_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        st::Fighter_ChangeMotionState(
            gobj,
            (*fp).u.gw.x222C_judgeVar1 + ftGw_MS_SpecialAirS1 as i32,
            transition_flags,
            (*fp).cur_anim_frame,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        if (*fp).cmd_vars[0] == 1 {
            (*fp).cmd_vars[0] = 2;
        }
        ftGameWatch_SpecialS_SetCall(gobj);
    }
}

/// Air -> ground Judgement Motion State handler (static in the C).
pub fn ftGw_SpecialAirS_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_SpecialAirS_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.gw.x2234 = 0;
        st::ftCommon_8007D7FC(&mut *fp);
        st::Fighter_ChangeMotionState(
            gobj,
            (*fp).u.gw.x222C_judgeVar1 + ftGw_MS_SpecialS1 as i32,
            transition_flags,
            (*fp).cur_anim_frame,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        ftGameWatch_SpecialS_SetCall(gobj);
    }
}
