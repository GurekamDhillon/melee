//! src/melee/ft/kinds/ftCommon/ftCo_0A01.c: the CPU fighter AI (target selection, behaviour
//! dispatch, movement/recovery planning, item handling), everything `ftCo_800B3900` runs.
//! Generated from the C by `melee_world/gen/mtu/c2rs.py` (libclang AST: C integer / float semantics,
//! clang's -ffp-contract=on fusion); the goto functions are hand ports in `gen/ai/hand/`.
//! Regenerate with `bash rust/melee_fighter/gen/ai/regen.sh`. Calls out of the TU are `ext::` hooks
//! (unset: panic "TODO"); the melee crate's `vs_match/cpu.rs` binds them to the native sim.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use crate::types::*;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_8003118C(a0: *mut Vec, a1: f32) -> i32;
    fn Geno_DefineAiLike(a0: i32) -> i32;
    fn Ground_801C4368(a0: *mut f32, a1: *mut f32);
    fn Ground_801C4DA0(a0: *mut Vec, a1: *mut f32) -> i32;
    fn Ground_801C5794() -> i32;
    fn HSD_Randf() -> f32;
    fn Item_IsGrabbable(a0: *mut HSD_GObj) -> i32;
    fn PSMTXIdentity(a0: *mut [f32; 4]);
    fn Player_8003248C(a0: i32, a1: i32) -> Gm_PKind;
    fn ScriptGame_CapsMaxJumps(a0: *mut Fighter) -> i32;
    fn ScriptGame_CpuStandTick(a0: *mut Fighter) -> i32;
    fn ScriptGame_StageJointActive(a0: i32) -> i32;
    fn Stage_GetBlastZoneBottomOffset() -> f32;
    fn Stage_GetBlastZoneLeftOffset() -> f32;
    fn Stage_GetBlastZoneRightOffset() -> f32;
    fn Stage_GetBlastZoneTopOffset() -> f32;
    fn ftKb_MexKirbyLike(a0: i32) -> i32;
    fn ftLib_IsDead(a0: *mut HSD_GObj) -> i32;
    fn ftPp_SpecialLw_Enter(a0: *mut HSD_GObj);
    fn ft_80087A18(a0: *mut HSD_GObj) -> i32;
    fn ft_80087A80(a0: *mut HSD_GObj) -> i32;
    fn ftpickupitem_80094150(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn gm_8016B14C() -> i32;
    fn gm_8016B168() -> i32;
    fn gm_8016C6C0(a0: *mut HSD_GObj) -> i32;
    fn gm_8016C75C(a0: *mut HSD_GObj) -> i32;
    fn gm_GetCurrentGameMode() -> u8;
    fn grBigBlue_801EF844(a0: i32) -> i32;
    fn grCastle_801CDF54(a0: *mut Vec) -> i32;
    fn grCorneria_801E2D90(a0: i32) -> i32;
    fn grCorneria_801E2E50(a0: i32) -> i32;
    fn grGreatBay_801F66A4() -> i32;
    fn grInishie1_801FCAAC(a0: i32) -> i32;
    fn grLib_801C9E60(a0: *mut Vec) -> i32;
    fn grRCruise_80201988(a0: i32) -> i32;
    fn grVenom_80206D10(a0: i32) -> i32;
    fn it_8026C1B4(a0: *mut HSD_GObj) -> i32;
    fn lbColl_800083C4(a0: *mut HurtCapsule);
    fn lbColl_800096B4(a0: *mut [f32; 4], a1: Vec3, a2: Vec3, a3: *mut _GXColor, a4: *mut _GXColor, a5: f32);
    fn lbColl_80009DD4(a0: *mut Vec, a1: *mut Vec, a2: *mut _GXColor);
    fn lbVector_Angle(a0: *mut Vec, a1: *mut Vec) -> f32;
    fn lbVector_Normalize(a0: *mut Vec) -> f32;
    fn lb_8000D008(a0: f32, a1: f32) -> f32;
    fn mpCheckCeiling(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: i32, a9: i32) -> i32;
    fn mpCheckFloor(a0: f32, a1: f32, a2: f32, a3: f32, a4: f32, a5: *mut Vec, a6: *mut i32, a7: *mut u32, a8: *mut Vec, a9: i32, a10: i32, a11: i32, a12: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, a13: *mut HSD_GObj) -> i32;
    fn mpCheckLeftWall(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: i32, a9: i32) -> i32;
    fn mpCheckRightWall(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: i32, a9: i32) -> i32;
    fn mpGetSpeed(a0: i32, a1: *mut Vec, a2: *mut Vec) -> i32;
    fn mpIsland_8005AB54(a0: i32) -> *mut mp_UnkStruct0;
    fn mpIsland_8005AC14(a0: *mut Vec, a1: f32) -> *mut mp_UnkStruct0;
    fn mpIsland_8005AC8C(a0: *mut mp_UnkStruct0) -> i32;
    fn mpIsland_8005ACE8(a0: *mut mp_UnkStruct0, a1: *mut Vec, a2: *mut Vec);
    fn mpLineGetFlags(a0: i32) -> u32;
    }
    pub use crate::tu::fighter::Fighter_804D64FC;
    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
    pub use crate::tu::ft_3c61::ftCo_803C6594;
    pub use melee_world::tu::mpisland::mpIsland_80458E88;
    pub use melee_runtime::shared::p_ftCommonData;
    pub use melee_world::tu::ground::stage_info;
}

pub static mut ftCo_803C5A68: [i32; 35] = [1, 0, 0, 1, 0, 0, 0, 1, 7, 6, 5, 4, 4, 3, 1, 1, 2, 1, 5, 2, 2, 2, 3, 3, 3, 4, 0, 0, 8, 4, 1, 4, 4, 5, 5];

/// `ftCo_CpuFinishWithNeutralY`
unsafe fn ftCo_CpuFinishWithNeutralY(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_CpuFinishWithNeutralStick`
unsafe fn ftCo_CpuFinishWithNeutralStick(mut fp: *mut Fighter) {
    ftCo_CpuSetNeutralStick(fp);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_CpuReturnToPreviousBehavior`
unsafe fn ftCo_CpuReturnToPreviousBehavior(mut fp: *mut Fighter, mut data: *mut CpuFighter) {
    (*data).x18 = (*data).x1C;
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_CpuHoldUpForOneFrame`
unsafe fn ftCo_CpuHoldUpForOneFrame(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 127 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    ftCo_CpuFinishWithNeutralY(fp);
}

/// `ftCo_CpuTapA`
unsafe fn ftCo_CpuTapA(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressA as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseA as i32) as u8);
}

/// `ftCo_CpuRetapA`
unsafe fn ftCo_CpuRetapA(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseA as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    ftCo_CpuTapA(fp);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_CpuTapB`
unsafe fn ftCo_CpuTapB(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressB as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseB as i32) as u8);
}

/// `ftCo_CpuStartCharging`
unsafe fn ftCo_CpuStartCharging(mut fp: *mut Fighter) {
    ftCo_CpuSetNeutralStick(fp);
    ftCo_CpuTapB(fp);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_CpuTapR`
unsafe fn ftCo_CpuTapR(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressR as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseR as i32) as u8);
}

/// `ftCo_CpuRetapR`
unsafe fn ftCo_CpuRetapR(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseR as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    ftCo_CpuTapR(fp);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_CpuTapRAndWait`
unsafe fn ftCo_CpuTapRAndWait(mut fp: *mut Fighter) {
    ftCo_CpuTapR(fp);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
}

/// `ftCo_CpuDoubleJumpAfterDelay`
unsafe fn ftCo_CpuDoubleJumpAfterDelay(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 10 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 10 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_800A0148`
pub unsafe fn ftCo_800A0148(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A0148");
    let mut x1A88: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ((*fp).cur_pos.y + (*x1A88).x558) > ext::Stage_GetBlastZoneTopOffset() {
        ftCo_CpuFinishWithNeutralStick(fp);
    } else if ftCo_800A1CA8(fp) != 0 {
        ftCo_CpuSetNeutralStick(fp);
        ftCo_CpuDoubleJumpAfterDelay(fp);
    } else if ((if ((*x1A88).x54.x - (*fp).cur_pos.x) < (0 as f32) { -((*x1A88).x54.x - (*fp).cur_pos.x) } else { (*x1A88).x54.x - (*fp).cur_pos.x }) as f64) > 30.0f64 {
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickTowardDestination as i32) as u8, 127 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 10 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    } else {
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        ftCo_CpuSetNeutralStick(fp);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 10 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800A0384`
pub unsafe fn ftCo_800A0384(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A0384");
    if ((*fp).cur_pos.y + (*fp).cpu.x558) > ext::Stage_GetBlastZoneTopOffset() {
        ftCo_CpuFinishWithNeutralStick(fp);
    } else if ftCo_800A1CA8(fp) != 0 {
        ftCo_CpuSetNeutralStick(fp);
        ftCo_CpuDoubleJumpAfterDelay(fp);
    } else {
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXForward as i32) as u8, 127 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 10 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800A0508`
pub unsafe fn ftCo_800A0508(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A0508");
    ftCo_CpuSetNeutralStick(fp);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 30 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXForward as i32) as u8, 80 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 127 as u8);
    ftCo_CpuTapR(fp);
    ftCo_CpuFinishWithNeutralY(fp);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_800A05F4`
pub unsafe fn ftCo_800A05F4(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A05F4");
    if ((*fp).cur_pos.y + (*fp).cpu.x558) > ext::Stage_GetBlastZoneTopOffset() {
        ftCo_CpuFinishWithNeutralStick(fp);
    } else if ftCo_800A1CA8(fp) != 0 {
        ftCo_CpuSetNeutralStick(fp);
        ftCo_CpuDoubleJumpAfterDelay(fp);
    } else {
        ftCo_CpuSetNeutralStick(fp);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickTowardFighter as i32) as u8, 127 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 10 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800A0798`
pub unsafe fn ftCo_800A0798(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A0798");
    if ftCo_800A1CA8(fp) != 0 {
        ftCo_CpuSetNeutralStick(fp);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 10 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    } else {
        ftCo_CpuSetNeutralStick(fp);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardFighter as i32) as u8, 127 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 5 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800A08F0`
pub unsafe fn ftCo_800A08F0(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A08F0");
    ftCo_CpuSetNeutralStick(fp);
    if (((*fp).motion_id == ftCo_MS_SquatRv) || ((*fp).motion_id == ftCo_MS_Squat)) || ((*fp).motion_id == ftCo_MS_SquatWait) {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
        return;
    }
    if ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Nana as i32) {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, (-127_i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 5 as u8);
        ftCo_CpuSetNeutralStick(fp);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
        return;
    }
    's66: {
        match (*fp).cpu.level {
            0 => {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 45 as u8);
            }
            1 | 2 => {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 30 as u8);
            }
            3 | 4 => {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 15 as u8);
            }
            5 | 6 => {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 10 as u8);
            }
            7 | 8 => {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 5 as u8);
            }
            9 => {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
            }
            _ => {}
        }
    }
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, (-127_i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 5 as u8);
    ftCo_CpuSetNeutralStick(fp);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_800A0AF4`
pub unsafe fn ftCo_800A0AF4(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A0AF4");
    let mut rand: f32 = ext::HSD_Randf();
    if rand < 0.6f32 {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 80 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        ftCo_CpuFinishWithNeutralY(fp);
    } else if rand < 0.8f32 {
        ftCo_CpuRetapR(fp);
    } else if rand < 0.9f32 {
        ftCo_CpuRetapA(fp);
    } else {
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800A0C8C`
pub unsafe fn ftCo_800A0C8C(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A0C8C");
    super::ftcmdscript::ftCo_800B463C(fp, 127 as u8);
}

/// `ftCo_800A0CB0`
pub unsafe fn ftCo_800A0CB0(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A0CB0");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ((*fp).cpu.x7C % 600) == 0 {
        let mut x: f32 = ext::HSD_Randf();
        let mut rand: f32 = 0.0;
        x = (x * x) * x;
        rand = 1.0f32 - x;
        's67: {
            match ftAi_Kind((*fp).kind as i32) {
                3 | 5 => {
                    (*data).x56C = (9.0f64 * (rand as f64)) as f32;
                    return;
                }
                31 => {
                    (*data).x56C = (18.0f64 * (rand as f64)) as f32;
                    return;
                }
                11 => {
                    (*data).x56C = (1.0f64 * (rand as f64)) as f32;
                    return;
                }
                _ => {
                    (*data).x56C = (4.0f64 * (rand as f64)) as f32;
                    return;
                }
            }
        }
    }
}

/// `ftCo_800A0DA4`
pub unsafe fn ftCo_800A0DA4(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A0DA4");
    let mut i: i32 = 0;
    let mut hurt: *mut FighterHurtCapsule = null_mut();
    let mut temp_r28: *mut CpuFighter = null_mut();
    let mut scale: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut var_f31: f32 = 0.0;
    let mut var_f30: f32 = 0.0;
    let mut var_f29: f32 = 0.0;
    let mut dy: f32 = 0.0;
    temp_r28 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    var_f30 = { var_f31 = { var_f29 = 0.0f32; var_f29 }; var_f31 };
    {
        i = 0;
        'l68: while i < ((*fp).hurt_capsules_len as i32) {
            hurt = core::ptr::addr_of_mut!((*fp).hurt_capsules[i as usize]) as *mut FighterHurtCapsule;
            ext::lbColl_800083C4(core::ptr::addr_of_mut!((*hurt).capsule) as *mut HurtCapsule);
            dx = (*hurt).capsule.a_pos.x - (*fp).cur_pos.x;
            dy = (*hurt).capsule.a_pos.y - (*fp).cur_pos.y;
            scale = (*hurt).capsule.scale * (*fp).x34_scale.y;
            if var_f31 > (dx - scale) {
                var_f31 = dx - scale;
            }
            if var_f30 < (dx + scale) {
                var_f30 = dx + scale;
            }
            if var_f29 < (dy + scale) {
                var_f29 = dy + scale;
            }
            dx = (*hurt).capsule.b_pos.x - (*fp).cur_pos.x;
            dy = (*hurt).capsule.b_pos.y - (*fp).cur_pos.y;
            if var_f31 > (dx - scale) {
                var_f31 = dx - scale;
            }
            if var_f30 < (dx + scale) {
                var_f30 = dx + scale;
            }
            if var_f29 < (dy + scale) {
                var_f29 = dy + scale;
            }
            i = i + 1;
        }
    }
    if ((*fp).facing_dir as f64) > 0.0f64 {
        (*temp_r28).x55C = var_f30;
        (*temp_r28).x560 = -var_f31;
    } else {
        (*temp_r28).x55C = -var_f31;
        (*temp_r28).x560 = var_f30;
    }
    (*temp_r28).x564 = (0.5f64 * (((*temp_r28).x55C + (*temp_r28).x560) as f64)) as f32;
    (*temp_r28).x568 = var_f29;
}

/// `ftCo_800A0F00`
pub unsafe fn ftCo_800A0F00(mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftCo_800A0F00");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ext::ft_80087A18(gobj) != 0 {
        let mut result: i32 = ext::ft_80087A80(gobj);
        if result == 2 {
            return 1;
        }
        if result == 1 {
            if ((*data).x7C % 240) > 120 {
                return 1;
            }
            return 0;
        }
        return 0;
    }
    return 0;
}

/// `ftCo_800A0FB0`
pub unsafe fn ftCo_800A0FB0(mut vec_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut arg4: i32, mut arg5: i32, mut arg6: i32, mut arg7: f32, mut arg8: f32, mut arg9: f32, mut arg10: f32, mut arg11: f32) -> i32 {
    melee_core::ct!("ftCo_800A0FB0");
    *line_id_out = (-1_i32);
    {
        let mut ret: i32 = ext::mpCheckFloor(arg7, arg8, arg9, arg10, arg11, vec_out, line_id_out, flags_out, normal_out, arg4, arg5, arg6, None, null_mut());
        if (ret != 0) && (ftCo_800A1B38(*line_id_out) != 0) {
            return 0;
        }
        return ret;
    }
}

/// `ftCo_800A101C`
pub unsafe fn ftCo_800A101C(mut arg0: *mut Fighter, mut arg1: i32, mut arg2: i32, mut arg3: i32) {
    melee_core::ct!("ftCo_800A101C");
    let mut sp50: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp44: Vec3 = unsafe { core::mem::zeroed() };
    let mut var_r31: i32 = 0;
    let mut temp_r30: *mut CpuFighter = null_mut();
    let mut temp_r3_2: *mut Fighter = null_mut();
    let mut temp_f0: f32 = 0.0;
    let mut temp_f0_3: f32 = 0.0;
    let mut temp_f1: f32 = 0.0;
    let mut temp_f3_2: f32 = 0.0;
    let mut temp_r6: i32 = 0;
    let mut var_r0: i32 = 0;
    let mut i: i32 = 0;
    let mut sp34: i32 = 0;
    let mut sp30: u32 = 0;
    temp_r30 = core::ptr::addr_of_mut!((*arg0).cpu) as *mut CpuFighter;
    if ftAi_Kind((*arg0).kind as i32) == (Ft_Kind_Nana as i32) {
        (*temp_r30).set_xF9_b2((1 as u8) as u8);
        (*temp_r30).kind = 6 as u32;
        (*temp_r30).x3C = 15.0f32;
    } else {
        (*temp_r30).kind = arg1 as u32;
        (*temp_r30).x3C = 4e+01f32;
    }
    (*temp_r30).level = arg2;
    (*temp_r30).x14 = arg3;
    's69: {
        match (*temp_r30).kind {
            1 | 25 => {
                (*temp_r30).x18 = 12;
                (*temp_r30).x1C = 12;
                (*temp_r30).x20 = 0;
            }
            3 => {
                (*temp_r30).x18 = 1;
                (*temp_r30).x1C = 1;
                (*temp_r30).x20 = 11;
            }
            15 => {
                (*temp_r30).x18 = 0;
                (*temp_r30).x1C = 0;
                (*temp_r30).x20 = 0;
            }
            _ => {
                (*temp_r30).x18 = 1;
                (*temp_r30).x1C = 1;
                (*temp_r30).x20 = 10;
            }
        }
    }
    (*temp_r30).x24 = 300;
    (*temp_r30).x28 = 0;
    (*temp_r30).x2C = 5;
    (*temp_r30).x30 = 0;
    (*temp_r30).set_xF9_b0((0 as u8) as u8);
    (*temp_r30).x38 = 5.0f32;
    (*temp_r30).x40 = 5e+01f32;
    (*temp_r30).x44 = null_mut();
    (*temp_r30).x48 = null_mut();
    (*temp_r30).x50 = 0 as u32;
    (*temp_r30).x98 = (*arg0).cur_pos;
    if ftCo_800A0FB0(core::ptr::addr_of_mut!(sp50) as *mut Vec, core::ptr::addr_of_mut!(sp34) as *mut i32, core::ptr::addr_of_mut!(sp30) as *mut u32, core::ptr::addr_of_mut!(sp44) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), (*arg0).cur_pos.x, 1e+01f32 + (*arg0).cur_pos.y, (*arg0).cur_pos.x, (*arg0).cur_pos.y - 1e+03f32, 0.0f32) != 0 {
        (*temp_r30).x64.x = { (*temp_r30).x54.x = sp50.x; (*temp_r30).x54.x };
        (*temp_r30).x64.y = { (*temp_r30).x54.y = sp50.y; (*temp_r30).x54.y };
    } else {
        (*temp_r30).x64.x = { (*temp_r30).x54.x = (*arg0).cur_pos.x; (*temp_r30).x54.x };
        (*temp_r30).x64.y = { (*temp_r30).x54.y = (*arg0).cur_pos.y; (*temp_r30).x54.y };
    }
    (*temp_r30).x60 = 0;
    (*temp_r30).x74.y = 0.0f32;
    (*temp_r30).x74.x = 0.0f32;
    (*temp_r30).x6C.y = 0.0f32;
    (*temp_r30).x6C.x = 0.0f32;
    (*temp_r30).set_xF8_b0((0 as u8) as u8);
    (*temp_r30).set_xF8_b12((0 as u8) as u8);
    temp_r6 = melee_core::ppc::fctiwz(10.0f64 * (ext::HSD_Randf() as f64));
    (*temp_r30).x7C = temp_r6;
    (*temp_r30).x80 = temp_r6;
    (*temp_r30).x84 = 0;
    (*temp_r30).x90 = 0;
    (*temp_r30).x88 = 0;
    (*temp_r30).x8C = 0;
    (*temp_r30).csP = null_mut();
    (*temp_r30).command_duration = 0 as u32;
    (*temp_r30).write_pos = (*temp_r30).buffer.as_mut_ptr();
    (*temp_r30).xA4 = 0;
    (*temp_r30).xC8 = 0 as u8;
    (*temp_r30).xEC = 0 as u8;
    (*temp_r30).set_xF8_b5((0 as u8) as u8);
    (*temp_r30).set_xF8_b6((0 as u8) as u8);
    (*temp_r30).set_xF8_b7((0 as u8) as u8);
    (*temp_r30).set_xF9_b1((0 as u8) as u8);
    (*temp_r30).buttons = 0 as u32;
    (*temp_r30).lstick.x = 0 as i8;
    (*temp_r30).lstick.y = 0 as i8;
    (*temp_r30).cstick.x = 0 as i8;
    (*temp_r30).cstick.y = 0 as i8;
    (*temp_r30).rtrigger = 0 as u8;
    (*temp_r30).ltrigger = 0 as u8;
    (*temp_r30).set_xF9_b2((0 as u8) as u8);
    (*temp_r30).set_xF9_b3((0 as u8) as u8);
    (*temp_r30).set_xF9_b4((0 as u8) as u8);
    (*temp_r30).set_xF9_b6((0 as u8) as u8);
    (*temp_r30).set_xF9_b5((0 as u8) as u8);
    (*temp_r30).set_xF9_b7((0 as u8) as u8);
    (*temp_r30).set_xFA_b7((0 as u8) as u8);
    (*temp_r30).set_xFB_b0((0 as u8) as u8);
    (*temp_r30).x94 = 0;
    (*temp_r30).set_xFA_b1((0 as u8) as u8);
    (*temp_r30).set_xFA_b2((0 as u8) as u8);
    (*temp_r30).set_xFA_b34((1 as u8) as u8);
    (*temp_r30).set_xFA_b5((0 as u8) as u8);
    (*temp_r30).set_xFA_b6((0 as u8) as u8);
    (*temp_r30).x55C = 1.0f32;
    (*temp_r30).x560 = 1.0f32;
    (*temp_r30).x564 = 0.5f32 * ((*temp_r30).x55C + (*temp_r30).x560);
    (*temp_r30).x568 = 2.0f32;
    (*temp_r30).x444 = core::ptr::addr_of_mut!((*temp_r30).xFC[5 as usize]) as *mut Fighter_x1A88_xFC_t;
    (*temp_r30).x448 = (*temp_r30).xFC.as_mut_ptr();
    if ftAi_Kind((*arg0).kind as i32) == (Ft_Kind_Nana as i32) {
        temp_r3_2 = ftCo_800A589C(arg0);
        {
            i = 0;
            'l70: while i < 30 {
                (*temp_r30).xFC[i as usize].x0 = 0 as u32;
                (*temp_r30).xFC[i as usize].lstick.x = 0 as i8;
                (*temp_r30).xFC[i as usize].lstick.y = 0 as i8;
                (*temp_r30).xFC[i as usize].x4 = 0 as u8;
                (*temp_r30).xFC[i as usize].x5 = 0 as u8;
                (*temp_r30).xFC[i as usize].cur_pos = (*temp_r3_2).cur_pos;
                (*temp_r30).xFC[i as usize].facing_dir = (*temp_r3_2).facing_dir;
                i = i + 1;
            }
        }
    } else {
        {
            i = 0;
            'l71: while i < 30 {
                (*temp_r30).xFC[i as usize].x0 = 0 as u32;
                (*temp_r30).xFC[i as usize].lstick.x = 0 as i8;
                (*temp_r30).xFC[i as usize].lstick.y = 0 as i8;
                (*temp_r30).xFC[i as usize].x4 = 0 as u8;
                (*temp_r30).xFC[i as usize].x5 = 0 as u8;
                (*temp_r30).xFC[i as usize].cur_pos.x = 0.0f32;
                (*temp_r30).xFC[i as usize].cur_pos.y = 0.0f32;
                (*temp_r30).xFC[i as usize].cur_pos.z = 0.0f32;
                (*temp_r30).xFC[i as usize].facing_dir = 1.0f32;
                i = i + 1;
            }
        }
    }
    temp_f3_2 = (*arg0).co_attrs.gravity;
    temp_f1 = (*arg0).co_attrs.jump_v_initial_velocity * (*arg0).co_attrs.air_jump_v_multiplier;
    if (temp_f3_2 < 1e-05f32) && (temp_f3_2 > (-1e-05f32)) {
        var_r0 = 1;
    } else {
        var_r0 = 0;
    }
    if var_r0 != 0 {
        (*temp_r30).x558 = 1e+01f32;
    } else {
        (*temp_r30).x558 = (((temp_f1 * temp_f1) as f64) / (2.0f64 * (temp_f3_2 as f64))) as f32;
    }
    (*temp_r30).half_width = 1e+01f32;
    (*temp_r30).half_height = 1e+01f32;
    (*temp_r30).x570 = 1.0f32;
    super::ftcpuattack::ftCo_800B9704(arg0);
    's72: {
        match ftAi_Kind((*arg0).kind as i32) {
            3 | 5 => {
                (*temp_r30).x56C = 8.5f32;
            }
            31 => {
                (*temp_r30).x56C = 17.0f32;
            }
            11 => {
                (*temp_r30).x56C = 1.0f32;
            }
            _ => {
                (*temp_r30).x56C = 3.5f32;
            }
        }
    }
}

/// `convertStickAxis`
unsafe fn convertStickAxis(mut val: i8) -> f32 {
    let mut ret: f32 = if (val as i32) > 0 { ((val as i32) as f32) / 127.0f32 } else { ((val as i32) as f32) / 128.0f32 };
    return if (ret as f64) > 1.0f64 { 1.0f32 } else { if (ret as f64) < (-1.0f64) { -1.0f32 } else { ret } };
}

/// `ftCo_GetCpuLStickX`
pub unsafe fn ftCo_GetCpuLStickX(mut fp: *mut Fighter) -> f32 {
    melee_core::ct!("ftCo_GetCpuLStickX");
    return convertStickAxis((*fp).cpu.lstick.x);
}

/// `ftCo_GetCpuLStickY`
pub unsafe fn ftCo_GetCpuLStickY(mut fp: *mut Fighter) -> f32 {
    melee_core::ct!("ftCo_GetCpuLStickY");
    return convertStickAxis((*fp).cpu.lstick.y);
}

/// `ftCo_GetCpuLTrigger`
pub unsafe fn ftCo_GetCpuLTrigger(mut fp: *mut Fighter) -> f32 {
    melee_core::ct!("ftCo_GetCpuLTrigger");
    let mut ret: f32 = ((((*fp).cpu.ltrigger as i32) as f64) / 255.0f64) as f32;
    return if (ret as f64) > 1.0f64 { 1.0f32 } else { ret };
}

/// `ftCo_GetCpuRTrigger`
pub unsafe fn ftCo_GetCpuRTrigger(mut fp: *mut Fighter) -> f32 {
    melee_core::ct!("ftCo_GetCpuRTrigger");
    let mut ret: f32 = ((((*fp).cpu.rtrigger as i32) as f64) / 255.0f64) as f32;
    return if (ret as f64) > 1.0f64 { 1.0f32 } else { ret };
}

/// `ftCo_GetCpuButtons`
pub unsafe fn ftCo_GetCpuButtons(mut fp: *mut Fighter) -> u32 {
    melee_core::ct!("ftCo_GetCpuButtons");
    return (*fp).cpu.buttons;
}

/// `ftCo_GetCpuCStickX`
pub unsafe fn ftCo_GetCpuCStickX(mut fp: *mut Fighter) -> f32 {
    melee_core::ct!("ftCo_GetCpuCStickX");
    return convertStickAxis((*fp).cpu.cstick.x);
}

/// `ftCo_GetCpuCStickY`
pub unsafe fn ftCo_GetCpuCStickY(mut fp: *mut Fighter) -> f32 {
    melee_core::ct!("ftCo_GetCpuCStickY");
    return convertStickAxis((*fp).cpu.cstick.y);
}

/// `ftCo_800A1AB4`
pub unsafe fn ftCo_800A1AB4(mut fp0: *mut Fighter, mut fp1: *mut Fighter) -> f32 {
    melee_core::ct!("ftCo_800A1AB4");
    let mut dx: f32 = (*fp0).cur_pos.x - (*fp1).cur_pos.x;
    let mut dy: f32 = (*fp0).cur_pos.y - (*fp1).cur_pos.y;
    return melee_runtime::gekko::sqrtf(fmadds(dx, dx, dy * dy));
}

/// `ftCo_800A1B38_noinline`
pub unsafe fn ftCo_800A1B38_noinline(mut arg0: i32) -> i32 {
    melee_core::ct!("ftCo_800A1B38_noinline");
    return ftCo_800A1B38(arg0);
}

/// `ftCo_800A1B38`
pub unsafe fn ftCo_800A1B38(mut arg0: i32) -> i32 {
    melee_core::ct!("ftCo_800A1B38");
    if (((ext::grBigBlue_801EF844(arg0) != 0) || (ext::grInishie1_801FCAAC(arg0) != 0)) || (ext::grCorneria_801E2D90(arg0) != 0)) || (ext::grVenom_80206D10(arg0) != 0) {
        return 1;
    }
    return 0;
}

/// `ftCo_IsIgnoredFloor`
unsafe fn ftCo_IsIgnoredFloor(mut line_id: i32) -> i32 {
    return ftCo_800A1B38(line_id);
}

/// `ftCo_800A1BA8`
pub unsafe fn ftCo_800A1BA8(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A1BA8");
    let mut other_fp: *mut Fighter = (*fp).cpu.x44;
    if other_fp.is_null() {
        return 0;
    }
    {
        let mut coll_vec: Vec3 = unsafe { core::mem::zeroed() };
        let mut sp14: Vec3 = unsafe { core::mem::zeroed() };
        let mut sp10: i32 = 0;
        let mut spC: u32 = 0;
        if ((*fp).facing_dir as f64) > 0.0f64 {
            return ext::mpCheckRightWall((*fp).cur_pos.x, (*fp).cur_pos.y, (*other_fp).cur_pos.x, (*other_fp).cur_pos.y, core::ptr::addr_of_mut!(coll_vec) as *mut Vec, core::ptr::addr_of_mut!(sp10) as *mut i32, core::ptr::addr_of_mut!(spC) as *mut u32, core::ptr::addr_of_mut!(sp14) as *mut Vec, (-1_i32), (-1_i32));
        }
        return ext::mpCheckLeftWall((*fp).cur_pos.x, (*fp).cur_pos.y, (*other_fp).cur_pos.x, (*other_fp).cur_pos.y, core::ptr::addr_of_mut!(coll_vec) as *mut Vec, core::ptr::addr_of_mut!(sp10) as *mut i32, core::ptr::addr_of_mut!(spC) as *mut u32, core::ptr::addr_of_mut!(sp14) as *mut Vec, (-1_i32), (-1_i32));
    }
}

/// `ftCo_800A1C44`
pub unsafe fn ftCo_800A1C44(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A1C44");
    if (*fp).x2219_b1() != 0 {
        return 1;
    }
    if (*fp).x2164 != 0 {
        return 1;
    }
    if ((*fp).x2168 != 0) && ((*fp).x2338.x == 0) {
        return 1;
    }
    if (*fp).x221F_b3() != 0 {
        return 1;
    }
    return 0;
}

/// `ftCo_800A1CA8`
pub unsafe fn ftCo_800A1CA8(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A1CA8");
    return if (*fp).x2168 != 0 { 1 } else { 0 };
}

/// `ftCo_800A1CC4_inline0`
unsafe fn ftCo_800A1CC4_inline0(mut fp: *mut Fighter, mut var_r29: *mut ftCo_803C6594_t, mut temp_r3: *mut mp_UnkStruct0) -> i32 {
    if ext::mpIsland_8005AC14(core::ptr::addr_of_mut!((*var_r29).x0) as *mut Vec, -1e+01f32) == temp_r3 {
        if ((*var_r29).x44 as i32) != 0 {
            let mut temp_f1: f32 = (*fp).cur_pos.x;
            if (temp_f1 > (*var_r29).x48) && (temp_f1 < (*var_r29).x4C) {
                return 1;
            } else {
                return 0;
            }
        } else {
            return 1;
        }
    } else {
        return 0;
    }
}

/// `ftCo_800A1CC4_inline1`
unsafe fn ftCo_800A1CC4_inline1(mut fp: *mut Fighter, mut x: f32, mut y: f32) {
    (*fp).cpu.x60 = 300;
    (*fp).cpu.x64.x = (*fp).cpu.x54.x;
    (*fp).cpu.x64.y = (*fp).cpu.x54.y;
    (*fp).cpu.x54.x = x;
    (*fp).cpu.x54.y = y;
    (*fp).cpu.x38 = 5.0f32;
}

/// `ftCo_800A1CC4`
unsafe fn ftCo_800A1CC4(mut fp: *mut Fighter, mut var_r29: *mut ftCo_803C6594_t) {
    let mut data: *mut CpuFighter = null_mut();
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ((((!var_r29.is_null()) && ((*data).x60 == 0)) && ((*data).kind != (0 as u32))) && ((*fp).ground_or_air != ((GA_Air as i32) as u32))) && (!(ftCo_800A21FC(fp) != 0)) {
        let mut temp_r3: *mut mp_UnkStruct0 = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
        if !temp_r3.is_null() {
            let mut dy: f32 = (*data).x54.y - (*fp).cur_pos.y;
            let mut angle: f32 = ext::lb_8000D008(dy, if ((*data).x54.x - (*fp).cur_pos.x) < (0 as f32) { -((*data).x54.x - (*fp).cur_pos.x) } else { (*data).x54.x - (*fp).cur_pos.x });
            'l73: while !var_r29.is_null() {
                'c73: {
                    if !(ftCo_800A1CC4_inline0(fp, var_r29, temp_r3) != 0) {
                        break 'c73;
                    }
                    if angle > (*var_r29).x3C {
                        if (*var_r29).x14 >= 0.0f32 {
                            ftCo_800A1CC4_inline1(fp, (*var_r29).xC, (*var_r29).x10);
                            return;
                        }
                    } else if angle < (*var_r29).x40 {
                        if (*var_r29).x20 >= 0.0f32 {
                            ftCo_800A1CC4_inline1(fp, (*var_r29).x18, (*var_r29).x1C);
                            return;
                        }
                    } else if (*fp).facing_dir > 0.0f32 {
                        if (*var_r29).x2C >= 0.0f32 {
                            ftCo_800A1CC4_inline1(fp, (*var_r29).x24, (*var_r29).x28);
                            return;
                        }
                    } else {
                        if (*var_r29).x38 >= 0.0f32 {
                            ftCo_800A1CC4_inline1(fp, (*var_r29).x30, (*var_r29).x34);
                            return;
                        }
                    }
                }
                var_r29 = (*var_r29).next;
            }
        }
    }
}

/// `ftCo_800A1F3C_noinline`
pub unsafe fn ftCo_800A1F3C_noinline(mut fp: *mut Fighter, mut arg1: f32, mut arg2: f32, mut arg3: f32) {
    melee_core::ct!("ftCo_800A1F3C_noinline");
    ftCo_800A1F3C(fp, arg1, arg2, arg3);
}

/// `ftCo_800A1F3C_noinline2`
pub unsafe fn ftCo_800A1F3C_noinline2(mut fp: *mut Fighter, mut arg1: f32, mut arg2: f32, mut arg3: f32) {
    melee_core::ct!("ftCo_800A1F3C_noinline2");
    ftCo_800A1F3C_noinline(fp, arg1, arg2, arg3);
}

/// `ftCo_800A1F3C`
pub unsafe fn ftCo_800A1F3C(mut fp: *mut Fighter, mut arg1: f32, mut arg2: f32, mut arg3: f32) {
    melee_core::ct!("ftCo_800A1F3C");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*data).x60 == 0 {
        (*data).x54.x = arg1;
        (*data).x54.y = arg2;
        (*data).x38 = arg3;
        if std::env::var_os("MELEE_DBG_CPU").is_some() { eprintln!("x38 set1 {} p{} t {:?} x56C {} ", arg3, (*fp).player_id, ((*data).x54.x, (*data).x54.y), (*data).x56C); }
        ftCo_800A1CC4(fp, ext::ftCo_803C6594[ext::stage_info.grkind as usize]);
    }
}

/// `ftCo_ApplyStageEntry`
unsafe fn ftCo_ApplyStageEntry(mut fp: *mut Fighter, mut kind: GrKind) {
    ftCo_800A1CC4(fp, ext::ftCo_803C6594[kind as usize]);
}

/// `ftCo_800A75DC_set_target`
unsafe fn ftCo_800A75DC_set_target(mut fp: *mut Fighter, mut x60: *const i32, mut arg1: f32, mut arg2: f32, mut arg3: f32) {
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*x60) == 0 {
        (*data).x54.x = arg1;
        (*data).x54.y = arg2;
        (*data).x38 = arg3;
        if std::env::var_os("MELEE_DBG_CPU").is_some() { eprintln!("x38 set2 {} p{} t {:?}", arg3, (*fp).player_id, ((*data).x54.x, (*data).x54.y)); }
        ftCo_ApplyStageEntry(fp, ext::stage_info.grkind);
    }
}

/// `ftCo_800A1F98`
pub unsafe fn ftCo_800A1F98(mut x: i32, mut y: f32) -> i32 {
    melee_core::ct!("ftCo_800A1F98");
    let mut slope: f32 = 0.0;
    let mut intercept: f32 = 0.0;
    if ext::stage_info.grkind != ((Gr_Kind_Zebes as i32) as u32) {
        return 0;
    }
    ext::Ground_801C4368(core::ptr::addr_of_mut!(slope) as *mut f32, core::ptr::addr_of_mut!(intercept) as *mut f32);
    if y < slope {
        return 1;
    }
    if y < fmadds(intercept, x as f32, slope) {
        return 1;
    }
    return 0;
}

/// `ftCo_IsCpuControlled`
pub unsafe fn ftCo_IsCpuControlled(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_IsCpuControlled");
    if ext::Player_8003248C((*fp).player_id as i32, (*fp).is_sub_fighter() as i32) != ((Gm_PKind_Cpu as i32) as u32) {
        return 0;
    }
    if (*fp).cpu.kind == (5 as u32) {
        return 0;
    }
    return 1;
}

/// `ftCo_800A20A0`
pub unsafe fn ftCo_800A20A0(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A20A0");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (!(*fp).cpu.x44.is_null()) && ((*fp).ground_or_air == ((GA_Ground as i32) as u32)) {
        let mut other_fp: *mut Fighter = (*data).x44;
        if ftCo_800A1AB4(fp, other_fp) < (2.0f32 * (*(*ext::Fighter_804D64FC).x20.offset(ftAi_Kind((*fp).kind as i32) as isize))) {
            (*data).set_xF8_b6((1 as u8) as u8);
        } else {
            (*data).set_xF8_b6((0 as u8) as u8);
        }
    } else {
        (*data).set_xF8_b6((0 as u8) as u8);
    }
}

/// `ftCo_800A2170`
pub unsafe fn ftCo_800A2170(mut fp0: *mut Fighter, mut fp1: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A2170");
    let mut temp_r3: *mut mp_UnkStruct0 = null_mut();
    if (*fp0).ground_or_air == ((GA_Air as i32) as u32) {
        return 0;
    }
    if (*fp1).ground_or_air == ((GA_Air as i32) as u32) {
        return 0;
    }
    temp_r3 = ext::mpIsland_8005AB54((*fp0).coll_data.floor.index);
    if temp_r3.is_null() {
        return 0;
    }
    if ext::mpIsland_8005AB54((*fp1).coll_data.floor.index) == temp_r3 {
        return 1;
    }
    return 0;
}

/// `ftCo_800A21FC`
pub unsafe fn ftCo_800A21FC(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A21FC");
    let mut vec: Vec3 = unsafe { core::mem::zeroed() };
    let mut data0: *mut mp_UnkStruct0 = null_mut();
    let mut data1: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        return 0;
    }
    data0 = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
    if data0.is_null() {
        return 0;
    }
    vec.x = (*data1).x54.x;
    vec.y = (5.0f64 + ((*data1).x54.y as f64)) as f32;
    vec.z = 0.0f32;
    if ext::mpIsland_8005AC14(core::ptr::addr_of_mut!(vec) as *mut Vec, -1e+01f32) == data0 {
        return 1;
    }
    return 0;
}

/// `ftCo_800A229C`
#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe fn ftCo_800A229C(mut fp: *mut Fighter, mut arg1: *mut Vec) -> i32 {
    melee_core::ct!("ftCo_800A229C");
    let mut sp2C: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp20: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp1C: f32 = 0.0;
    let mut sp18: f32 = 0.0;
    let mut stage: u32 = 0;
    let mut w: f32 = 0.0;
    let mut h: f32 = 0.0;
    let mut bottom: f32 = 0.0;
    let mut frac: f32 = 0.0;
    let mut mag: f32 = 0.0;
    let mut var_r0: i32 = 0;
    if (ext::grGreatBay_801F66A4() != 0) && (((*fp).cur_pos.x as f64) > (-5.0f64)) {
        *arg1 = (*fp).cur_pos;
        return 2;
    }
    if ext::grCastle_801CDF54(arg1) != 0 {
        return 1;
    }
    if ext::stage_info.grkind != ((Gr_Kind_Zebes as i32) as u32) {
        var_r0 = 0;
    } else {
        ext::Ground_801C4368(core::ptr::addr_of_mut!(sp18) as *mut f32, core::ptr::addr_of_mut!(sp1C) as *mut f32);
        bottom = ext::Stage_GetBlastZoneBottomOffset();
        if sp18 > bottom {
            (*arg1).x = 0.5f32 * (ext::Stage_GetBlastZoneLeftOffset() + ext::Stage_GetBlastZoneRightOffset());
            (*arg1).y = bottom;
            (*arg1).z = 0.0f32;
            var_r0 = 1;
        } else {
            var_r0 = 0;
        }
    }
    if var_r0 != 0 {
        return 1;
    }
    stage = ext::stage_info.grkind;
    let _ = ext::grLib_801C9E60(core::ptr::addr_of_mut!(sp2C) as *mut Vec);
    'b74: {
        if stage == ((Gr_Kind_RCruise as i32) as u32) {
            w = ext::Stage_GetBlastZoneRightOffset() - ext::Stage_GetBlastZoneLeftOffset();
            'b75: {
                if (sp2C.x as f64) < 0.0f64 {
                    if (*fp).cur_pos.x < fmadds(0.4f32, w, ext::Stage_GetBlastZoneLeftOffset()) {
                        *arg1 = (*fp).cur_pos;
                        return 2;
                    }
                    break 'b75;
                }
                if (*fp).cur_pos.x > (-fmadds(0.4f32, w, -ext::Stage_GetBlastZoneRightOffset())) {
                    *arg1 = (*fp).cur_pos;
                    return 2;
                }
            }
            h = ext::Stage_GetBlastZoneTopOffset() - ext::Stage_GetBlastZoneBottomOffset();
            'b76: {
                if (sp2C.y as f64) > 0.0f64 {
                    if (*fp).cur_pos.y > (-fmadds(0.4f32, h, -ext::Stage_GetBlastZoneTopOffset())) {
                        *arg1 = (*fp).cur_pos;
                        return 2;
                    }
                    break 'b76;
                }
                if (*fp).cur_pos.y < fmadds(0.4f32, h, ext::Stage_GetBlastZoneBottomOffset()) {
                    *arg1 = (*fp).cur_pos;
                    return 2;
                }
            }
            if ((((*fp).cur_pos.x < fmadds(0.2f32, w, ext::Stage_GetBlastZoneLeftOffset())) || ((*fp).cur_pos.x > fmadds(-0.2f32, w, ext::Stage_GetBlastZoneRightOffset()))) || ((*fp).cur_pos.y > fmadds(-0.2f32, h, ext::Stage_GetBlastZoneTopOffset()))) || ((*fp).cur_pos.y < fmadds(0.2f32, h, ext::Stage_GetBlastZoneBottomOffset())) {
                *arg1 = (*fp).cur_pos;
                return 2;
            }
            break 'b74;
        }
        if stage == ((Gr_Kind_BigBlue as i32) as u32) {
            w = ext::Stage_GetBlastZoneRightOffset() - ext::Stage_GetBlastZoneLeftOffset();
            h = ext::Stage_GetBlastZoneTopOffset() - ext::Stage_GetBlastZoneBottomOffset();
            if ((((*fp).cur_pos.x < fmadds(0.2f32, w, ext::Stage_GetBlastZoneLeftOffset())) || ((*fp).cur_pos.x > fmadds(-0.2f32, w, ext::Stage_GetBlastZoneRightOffset()))) || ((*fp).cur_pos.y > fmadds(-0.2f32, h, ext::Stage_GetBlastZoneTopOffset()))) || ((*fp).cur_pos.y < fmadds(0.2f32, h, ext::Stage_GetBlastZoneBottomOffset())) {
                *arg1 = (*fp).cur_pos;
                return 2;
            }
            break 'b74;
        }
        if stage == ((Gr_Kind_Icemt as i32) as u32) {
            h = ext::Stage_GetBlastZoneTopOffset() - ext::Stage_GetBlastZoneBottomOffset();
            let _ = ext::grLib_801C9E60(core::ptr::addr_of_mut!(sp20) as *mut Vec);
            mag = if sp20.y < (0 as f32) { -sp20.y } else { sp20.y };
            frac = (0.4f64).mul_add(mag as f64, 0.4f64) as f32;
            if (sp20.y as f64) < 0.0f64 {
                if (*fp).cur_pos.y < fmadds(h, frac, ext::Stage_GetBlastZoneBottomOffset()) {
                    *arg1 = (*fp).cur_pos;
                    return 2;
                }
                break 'b74;
            }
            if (*fp).cur_pos.y > (-fmadds(h, frac, -ext::Stage_GetBlastZoneTopOffset())) {
                *arg1 = (*fp).cur_pos;
                return 2;
            }
        }
    }
    if ext::Camera_8003118C(core::ptr::addr_of_mut!((*fp).cur_pos) as *mut Vec, 0.0f32) == 0 {
        *arg1 = (*fp).cur_pos;
        return (-1_i32);
    }
    return 0;
}

/// `ftCo_800A2718`
pub unsafe fn ftCo_800A2718(mut arg0: *mut mp_UnkStruct0) -> i32 {
    melee_core::ct!("ftCo_800A2718");
    let mut island: *mut mp_UnkStruct0 = arg0;
    if arg0.is_null() {
        return 0;
    }
    {
        let mut cur: *mut HSD_GObj = null_mut();
        let mut cur_ip: *mut Item = null_mut();
        let mut stage: *mut u32 = null_mut();
        {
            cur = *ext::HSD_GObjPLinkHead.offset(9 as isize);
            'l77: while !cur.is_null() {
                'c77: {
                    cur_ip = (*cur).user_data as *mut Item;
                    if ext::it_8026C1B4(cur) == 0 {
                        break 'c77;
                    }
                    if !(ftCo_800A5944(cur_ip) != 0) {
                        break 'c77;
                    }
                    if island == ext::mpIsland_8005AB54((*cur_ip).x378_itemColl.floor.index) {
                        return 1;
                    }
                }
                cur = (*cur).next;
            }
        }
        stage = core::ptr::addr_of_mut!(ext::stage_info.grkind) as *mut u32;
        's78: {
            match *stage {
                10 => {
                    return ext::mpIsland_8005AC8C(island);
                }
                8 => {
                    {
                        let mut y: f32 = (*island).x14.y;
                        if ftCo_800A1F98(90, y) != 0 {
                            return 1;
                        }
                        y = (*island).x8.y;
                        if ftCo_800A1F98(90, y) != 0 {
                            return 1;
                        }
                        return 0;
                    }
                }
                20 => {
                    {
                        let mut ret: i32 = 0;
                        let mut ret2: i32 = 0;
                        let mut y: f32 = (*island).x14.y;
                        if (*stage) != ((Gr_Kind_Onett as i32) as u32) {
                            ret = 0;
                        } else if ((y as f64) <= 5.0f64) && (ext::Ground_801C5794() != 0) {
                            ret = 1;
                        } else {
                            ret = 0;
                        }
                        if ret != 0 {
                            return 1;
                        }
                        y = (*island).x8.y;
                        if (*stage) != ((Gr_Kind_Onett as i32) as u32) {
                            ret2 = 0;
                        } else if ((y as f64) <= 5.0f64) && (ext::Ground_801C5794() != 0) {
                            ret2 = 1;
                        } else {
                            ret2 = 0;
                        }
                        if ret2 != 0 {
                            return 1;
                        }
                        return 0;
                    }
                }
                _ => {
                    return 0;
                }
            }
        }
    }
}

/// `ftCo_800A2718_dontinline`
unsafe fn ftCo_800A2718_dontinline(mut arg0: *mut mp_UnkStruct0) -> i32 {
    return ftCo_800A2718(arg0);
}

/// `ftCo_800A2718_dontinline2`
unsafe fn ftCo_800A2718_dontinline2(mut arg0: *mut mp_UnkStruct0) -> i32 {
    return ftCo_800A2718_dontinline(arg0);
}

/// `ftCo_800A2718_dontinline3`
unsafe fn ftCo_800A2718_dontinline3(mut arg0: *mut mp_UnkStruct0) -> i32 {
    return ftCo_800A2718_dontinline2(arg0);
}

/// `ftCo_800A28D0`
pub unsafe fn ftCo_800A28D0(mut fp: *mut Fighter, mut arg1: f32) -> i32 {
    melee_core::ct!("ftCo_800A28D0");
    if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        return 0;
    }
    {
        let mut data: *mut mp_UnkStruct0 = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
        if data.is_null() {
            return 0;
        }
        {
            let mut x_offset: f32 = 0.0;
            if ((*fp).facing_dir as f64) > 0.0f64 {
                x_offset = if ((*data).x14.x - (*fp).cur_pos.x) < (0 as f32) { -((*data).x14.x - (*fp).cur_pos.x) } else { (*data).x14.x - (*fp).cur_pos.x };
            } else {
                x_offset = if ((*data).x8.x - (*fp).cur_pos.x) < (0 as f32) { -((*data).x8.x - (*fp).cur_pos.x) } else { (*data).x8.x - (*fp).cur_pos.x };
            }
            if (x_offset as f64) < (5.0f64 * (arg1 as f64)) {
                return 1;
            }
        }
    }
    return 0;
}

/// `ftCo_800A2998`
pub unsafe fn ftCo_800A2998(mut fp: *mut Fighter, mut arg1: f32) -> i32 {
    melee_core::ct!("ftCo_800A2998");
    if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        return 0;
    }
    {
        let mut data: *mut mp_UnkStruct0 = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
        if data.is_null() {
            return 0;
        }
        {
            let mut diff_x8: f32 = 0.0;
            let mut diff_x14: f32 = 0.0;
            diff_x14 = if ((*data).x14.x - (*fp).cur_pos.x) < (0 as f32) { -((*data).x14.x - (*fp).cur_pos.x) } else { (*data).x14.x - (*fp).cur_pos.x };
            diff_x8 = if ((*data).x8.x - (*fp).cur_pos.x) < (0 as f32) { -((*data).x8.x - (*fp).cur_pos.x) } else { (*data).x8.x - (*fp).cur_pos.x };
            if diff_x14 < diff_x8 {
                if (diff_x14 as f64) < (5.0f64 * (arg1 as f64)) {
                    return 1;
                }
                return 0;
            }
            if (diff_x8 as f64) < (5.0f64 * (arg1 as f64)) {
                return 1;
            }
        }
    }
    return 0;
}

/// `ftCo_800A2A70`
pub unsafe fn ftCo_800A2A70(mut fp: *mut Fighter, mut arg1: i32) -> f32 {
    melee_core::ct!("ftCo_800A2A70");
    if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        return -1.0f32;
    }
    {
        let mut data: *mut mp_UnkStruct0 = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
        if data.is_null() {
            return -1.0f32;
        }
        {
            let mut r: f32 = 0.0;
            if arg1 != 0 {
                let mut x: f32 = (*data).x8.x - (*fp).cur_pos.x;
                let mut y: f32 = (*data).x8.y - (*fp).cur_pos.y;
                r = melee_runtime::gekko::sqrtf(fmadds(x, x, y * y));
            } else {
                let mut x: f32 = (*data).x14.x - (*fp).cur_pos.x;
                let mut y: f32 = (*data).x14.y - (*fp).cur_pos.y;
                r = melee_runtime::gekko::sqrtf(fmadds(x, x, y * y));
            }
            return r;
        }
    }
}

/// `ftCo_800A2BD4`
pub unsafe fn ftCo_800A2BD4(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A2BD4");
    return if (((*fp).facing_dir * ((*fp).cpu.x54.x - (*fp).cur_pos.x)) as f64) >= 0.0f64 { 1 } else { 0 };
}

/// `ftCo_800A2C08`
pub unsafe fn ftCo_800A2C08(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A2C08");
    let mut other_fp: *mut Fighter = (*fp).cpu.x44;
    if other_fp.is_null() {
        return 1;
    }
    {
        let mut pos_x_diff: f32 = (*other_fp).cur_pos.x - (*fp).cur_pos.x;
        if (if pos_x_diff < (0 as f32) { -pos_x_diff } else { pos_x_diff }) < 1.0f32 {
            return 1;
        }
        if ((pos_x_diff * (*fp).facing_dir) as f64) >= 0.0f64 {
            return 1;
        }
        return 0;
    }
}

/// `ftCo_800A2C80_inline0`
unsafe fn ftCo_800A2C80_inline0(mut fp: *mut Fighter) -> i32 {
    let mut floor_id: i32 = (*fp).coll_data.floor.index;
    if (((ext::grBigBlue_801EF844(floor_id) != 0) || (ext::grInishie1_801FCAAC(floor_id) != 0)) || (ext::grCorneria_801E2D90(floor_id) != 0)) || (ext::grVenom_80206D10(floor_id) != 0) {
        return 1;
    }
    return 0;
}

/// `ftCo_800A2C80`
pub unsafe fn ftCo_800A2C80(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A2C80");
    let mut result: i32 = 0;
    let mut temp_data: *mut CpuFighter = null_mut();
    let mut blocked: i32 = 0;
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut dir: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut line: i32 = 0;
    let mut data: *mut CpuFighter = null_mut();
    let mut flags: u32 = 0;
    let mut ax: f32 = 0.0;
    let mut ay: f32 = 0.0;
    let mut ex: f32 = 0.0;
    let mut ey: f32 = 0.0;
    let mut mag: f32 = 0.0;
    let mut v: f32 = 0.0;
    let mut vy: f32 = 0.0;
    let mut x_sq: f32 = 0.0;
    let mut y_sq: f32 = 0.0;
    let mut is_small: i32 = 0;
    let mut oob: i32 = 0;
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*data).xFA_b6() != 0 {
        return 0;
    }
    if (*fp).ground_or_air == ((GA_Ground as i32) as u32) {
        return ftCo_800A2C80_inline0(fp);
    }
    if (*fp).motion_id == 244 {
        return 0;
    }
    dir = (*fp).pos_delta;
    v = dir.x;
    vy = dir.y;
    x_sq = v * v;
    y_sq = vy * vy;
    mag = x_sq + y_sq;
    if (mag < 1e-05f32) && (mag > (-1e-05f32)) {
        is_small = 1;
    } else {
        is_small = 0;
    }
    if is_small != 0 {
        return 0;
    }
    if (vy as f64) > 0.0f64 {
        return 0;
    }
    if v < 0.0f32 {
        v = -v;
    }
    if (ext::lb_8000D008(vy, v) as f64) > (-1.0471975430846214f64) {
        return 0;
    }
    if (*data).xFA_b5() != 0 {
        return 0;
    }
    if ext::stage_info.grkind == ((Gr_Kind_Inishie1 as i32) as u32) {
        if (*fp).cur_pos.y < 5.0f32 {
            return 1;
        }
    } else if (ext::stage_info.grkind == ((Gr_Kind_Fourside as i32) as u32)) && ((*fp).cur_pos.y < (-35.0f32)) {
        return 1;
    }
    let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(dir) as *mut Vec);
    {
        let mut bottom_x: f32 = (*fp).coll_data.cur_pos.x + (*fp).coll_data.ecb.bottom.x;
        let mut bottom_y: f32 = (*fp).coll_data.cur_pos.y + (*fp).coll_data.ecb.bottom.y;
        ax = bottom_x;
        ay = bottom_y;
    }
    ex = fmadds(1e+03f32, dir.x, ax);
    ey = fmadds(1e+03f32, dir.y, ay);
    line_id = (-1_i32);
    blocked = 0;
    result = ext::mpCheckFloor(ax, ay, ex, ey, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, blocked as usize as *mut HSD_GObj);
    if result != 0 {
        line = line_id;
        if (((ext::grBigBlue_801EF844(line) != 0) || (ext::grInishie1_801FCAAC(line) != 0)) || (ext::grCorneria_801E2D90(line) != 0)) || (ext::grVenom_80206D10(line) != 0) {
            blocked = 1;
        }
        if blocked != 0 {
            result = 0;
        }
    }
    if result != 0 {
        {
            let mut floor_x: f32 = 0.0;
            let mut floor_y: f32 = 0.0;
            floor_y = floor_pos.y;
            temp_data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
            floor_x = floor_pos.x;
            if (((floor_x < ((*fp).cpu.half_width + ext::Stage_GetBlastZoneLeftOffset())) || (floor_x > (ext::Stage_GetBlastZoneRightOffset() - (*temp_data).half_width))) || (floor_y < ((*temp_data).half_height + ext::Stage_GetBlastZoneBottomOffset()))) || (floor_y > (ext::Stage_GetBlastZoneTopOffset() - (*temp_data).half_height)) {
                oob = 1;
            } else {
                oob = 0;
            }
        }
        if oob == 0 {
            return 0;
        }
    }
    if ext::mpCheckLeftWall(ax, ay, ex, ey, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32)) != 0 {
        {
            let mut floor_x: f32 = 0.0;
            let mut floor_y: f32 = 0.0;
            floor_y = floor_pos.y;
            temp_data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
            floor_x = floor_pos.x;
            if (((floor_x < ((*fp).cpu.half_width + ext::Stage_GetBlastZoneLeftOffset())) || (floor_x > (ext::Stage_GetBlastZoneRightOffset() - (*temp_data).half_width))) || (floor_y < ((*temp_data).half_height + ext::Stage_GetBlastZoneBottomOffset()))) || (floor_y > (ext::Stage_GetBlastZoneTopOffset() - (*temp_data).half_height)) {
                oob = 1;
            } else {
                oob = 0;
            }
        }
        if oob == 0 {
            return 0;
        }
    }
    if ext::mpCheckRightWall(ax, ay, ex, ey, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32)) != 0 {
        {
            let mut floor_x: f32 = 0.0;
            let mut floor_y: f32 = 0.0;
            floor_y = floor_pos.y;
            temp_data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
            floor_x = floor_pos.x;
            if (((floor_x < ((*fp).cpu.half_width + ext::Stage_GetBlastZoneLeftOffset())) || (floor_x > (ext::Stage_GetBlastZoneRightOffset() - (*temp_data).half_width))) || (floor_y < ((*temp_data).half_height + ext::Stage_GetBlastZoneBottomOffset()))) || (floor_y > (ext::Stage_GetBlastZoneTopOffset() - (*temp_data).half_height)) {
                oob = 1;
            } else {
                oob = 0;
            }
        }
        if oob == 0 {
            return 0;
        }
    }
    return 1;
}

/// `ftCo_800A3134`
pub unsafe fn ftCo_800A3134(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A3134");
    's79: {
        match (*fp).motion_id {
            191 | 183 => {
                return 1;
            }
            192 | 184 => {
                return 2;
            }
            _ => {
                return 0;
            }
        }
    }
}

/// `ftCo_IsGrabbing_dontinline`
pub unsafe fn ftCo_IsGrabbing_dontinline(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_IsGrabbing_dontinline");
    return ftCo_IsGrabbing(fp);
}

/// `ftCo_IsGrabbing`
pub unsafe fn ftCo_IsGrabbing(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_IsGrabbing");
    if (*fp).motion_id == ftCo_MS_CatchWait {
        return 1;
    }
    if ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Donkey as i32) {
        if ((*fp).motion_id >= (ftDk_MS_ThrowFWait0 as i32)) && ((*fp).motion_id <= (ftDk_MS_ThrowFKneebend as i32)) {
            return 1;
        }
    }
    if (ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Koopa as i32)) || (ftAi_Kind((*fp).kind as i32) == (Ft_Kind_GKoops as i32)) {
        if (*fp).motion_id == (ftKp_MS_SpecialSHit0_1 as i32) {
            return 1;
        }
    }
    if (ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Kirby as i32)) || (((ftAi_Kind((*fp).kind as i32) >= (Ft_Kind_Mex0 as i32)) && (ftAi_Kind((*fp).kind as i32) < (Ft_Kind_Max as i32))) && (ext::ftKb_MexKirbyLike(ftAi_Kind((*fp).kind as i32)) != 0)) {
        if ((*fp).motion_id >= (ftKb_MS_SpecialNCapture0 as i32)) && ((*fp).motion_id <= (ftKb_MS_EatTurnAir as i32)) {
            return 1;
        }
    }
    return 0;
}

/// `ftCo_800A3200`
pub unsafe fn ftCo_800A3200(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A3200");
    's80: {
        match (*fp).motion_id {
            252 => {
                return 1;
            }
            253 => {
                return 2;
            }
            _ => {
                return 0;
            }
        }
    }
}

/// `ftCo_800A3234`
pub unsafe fn ftCo_800A3234(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A3234");
    let mut sp34: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp28: Vec3 = unsafe { core::mem::zeroed() };
    let mut var_r0: i32 = 0;
    let mut temp_r31: *mut CpuFighter = null_mut();
    temp_r31 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*fp).cpu.xFA_b6() != 0 {
        if (*fp).pos_delta.y < 0.1f32 {
            return 1;
        }
        return 0;
    }
    if (*fp).ground_or_air != ((GA_Air as i32) as u32) {
        return 0;
    }
    if ((*fp).pos_delta.y as f64) > 0.0f64 {
        return 0;
    }
    if (*fp).motion_id == 244 {
        return 0;
    }
    if ((ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Yoshi as i32)) && ((*fp).x34_scale.y > 1.0f32)) && ((*temp_r31).x7C < 900) {
        return 0;
    }
    if ext::lb_8000D008((*temp_r31).x54.y - (*fp).cur_pos.y, if ((*temp_r31).x54.x - (*fp).cur_pos.x) < (0 as f32) { -((*temp_r31).x54.x - (*fp).cur_pos.x) } else { (*temp_r31).x54.x - (*fp).cur_pos.x }) < (-1.3089969f32) {
        return 0;
    }
    if (*fp).facing_dir > 0.0f32 {
        if ((*fp).coll_data.env_flags & 63) != 0 {
            var_r0 = 1;
        } else {
            var_r0 = 0;
        }
    } else if ((*fp).coll_data.env_flags & 4032) != 0 {
        var_r0 = 1;
    } else {
        var_r0 = 0;
    }
    if var_r0 != 0 {
        return 1;
    }
    {
        let mut sp18: i32 = 0;
        let mut sp14: u32 = 0;
        let mut topn_x: f32 = (*fp).coll_data.last_pos.x;
        let mut topn_y: f32 = (*fp).coll_data.last_pos.y;
        let mut bottom_x: f32 = (*fp).coll_data.ecb.bottom.x;
        let mut bottom_y: f32 = (*fp).coll_data.ecb.bottom.y;
        let mut y: f32 = (*fp).coll_data.last_pos.y + bottom_y;
        if ftCo_800A0FB0(core::ptr::addr_of_mut!(sp34) as *mut Vec, core::ptr::addr_of_mut!(sp18) as *mut i32, core::ptr::addr_of_mut!(sp14) as *mut u32, core::ptr::addr_of_mut!(sp28) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), topn_x + bottom_x, topn_y + bottom_y, (*fp).coll_data.cur_pos.x + bottom_x, (*fp).coll_data.cur_pos.y + bottom_y, 0.0f32) != 0 {
            return 0;
        }
        if (*temp_r31).x54.y > y {
            return 1;
        }
    }
    return 0;
}

/// `inlineC0`
unsafe fn inlineC0(mut fp: *mut Fighter) -> i32 {
    if ((*fp).coll_data.env_flags & (63 | 4032)) != 0 {
        return 1;
    } else {
        return 0;
    }
}

/// `ftCo_800A3498`
pub unsafe fn ftCo_800A3498(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A3498");
    let mut temp_r4: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*fp).cpu.xFA_b5() != 0 {
        return 0;
    }
    if (*fp).ground_or_air != ((GA_Air as i32) as u32) {
        return 0;
    }
    if ((*fp).pos_delta.y as f64) > 0.0f64 {
        return 0;
    }
    if inlineC0(fp) != 0 {
        return 1;
    }
    if ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Luigi as i32) {
        if ((*fp).pos_delta.y as f64) < 0.0f64 {
            return 1;
        }
    } else if (*temp_r4).x54.y > ((*temp_r4).x568 + ((*fp).coll_data.cur_pos.y + (*fp).coll_data.ecb.bottom.y)) {
        return 1;
    }
    return 0;
}

/// `ftCo_800A3554`
pub unsafe fn ftCo_800A3554(mut fp: *mut Fighter, mut arg1: f32) -> i32 {
    melee_core::ct!("ftCo_800A3554");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        return 0;
    }
    if !(ftCo_800A21FC(fp) != 0) {
        return 0;
    }
    {
        let mut x: f32 = (*data).x54.x - (*fp).cur_pos.x;
        let mut y: f32 = (*data).x54.y - (*fp).cur_pos.y;
        if melee_runtime::gekko::sqrtf(fmadds(x, x, y * y)) < ((*data).x38 + arg1) {
            if (*data).x60 != 0 {
                (*data).x60 = 0;
                (*data).x54.x = (*data).x64.x;
                (*data).x54.y = (*data).x64.y;
                ftCo_800A1CC4(fp, ext::ftCo_803C6594[ext::stage_info.grkind as usize]);
                return 0;
            }
            return 1;
        }
        return 0;
    }
}

/// `ftCo_800A2170_it`
unsafe fn ftCo_800A2170_it(mut fp: *mut Fighter, mut ip: *mut Item) -> i32 {
    let mut data: *mut mp_UnkStruct0 = null_mut();
    if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        return 0;
    }
    if (*ip).ground_or_air == ((GA_Air as i32) as u32) {
        return 0;
    }
    data = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
    if data.is_null() {
        return 0;
    }
    if ext::mpIsland_8005AB54((*ip).x378_itemColl.floor.index) == data {
        return 1;
    }
    return 0;
}

/// `ftCo_800A3710`
pub unsafe fn ftCo_800A3710(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A3710");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut ip: *mut Item = (*data).x4C;
    if !((*data).xF9_b7() != 0) {
        return 0;
    }
    if ip.is_null() {
        return 0;
    }
    if !(ext::Item_IsGrabbable((*ip).entity) != 0) {
        (*data).x4C = null_mut();
        return 0;
    }
    if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        return 0;
    }
    if !(*fp).item_gobj.is_null() {
        return 0;
    }
    if !(ftCo_800A2170_it(fp, ip) != 0) {
        return 0;
    }
    if ext::ftpickupitem_80094150((*fp).gobj, (*ip).entity) != 0 {
        return 1;
    }
    return 0;
}

/// `ftCo_IsAlly`
pub unsafe fn ftCo_IsAlly(mut fp0: *mut Fighter, mut fp1: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_IsAlly");
    if fp0.is_null() {
        return 0;
    }
    if fp1.is_null() {
        return 0;
    }
    if ((*fp0).player_id as i32) == ((*fp1).player_id as i32) {
        return 1;
    }
    if ext::gm_8016B168() != 0 {
        if ((*fp0).team as i32) == ((*fp1).team as i32) {
            return 1;
        }
        return 0;
    }
    if ext::gm_8016B14C() != 0 {
        return 0;
    }
    if ((*fp0).team as i32) == ((*fp1).team as i32) {
        return 1;
    }
    return 0;
}

/// `ftCo_IsAlly_dontinline`
unsafe fn ftCo_IsAlly_dontinline(mut fp0: *mut Fighter, mut fp1: *mut Fighter) -> i32 {
    return ftCo_IsAlly(fp0, fp1);
}

/// `ftCo_800A3908_inline0`
unsafe fn ftCo_800A3908_inline0(mut fp: *mut Fighter, mut data: *mut CpuFighter, mut x: f32, mut y: f32) -> i32 {
    if (((x < ((*fp).cpu.half_width + ext::Stage_GetBlastZoneLeftOffset())) || (x > (ext::Stage_GetBlastZoneRightOffset() - (*data).half_width))) || (y < ((*data).half_height + ext::Stage_GetBlastZoneBottomOffset()))) || (y > (ext::Stage_GetBlastZoneTopOffset() - (*data).half_height)) {
        return 1;
    }
    return 0;
}

/// `ftCo_800A3908_inline1`
unsafe fn ftCo_800A3908_inline1(mut x: f32, mut y: f32, mut out_pos: *mut Vec, mut out_normal: *mut Vec, mut out_line: *mut i32, mut out_flags: *mut u32) -> i32 {
    let mut result: i32 = 0;
    let mut valid: i32 = 0;
    *out_line = (-1_i32);
    valid = 0;
    result = ext::mpCheckFloor(x, 5.0f32 + y, x, y - 5.0f32, 0.0f32, out_pos, out_line, out_flags, out_normal, (-1_i32), (-1_i32), (-1_i32), None, null_mut());
    if (result != 0) && (ftCo_800A1B38_noinline(*out_line) != 0) {
        ();
    } else {
        valid = result;
    }
    return valid;
}

/// `ftCo_GetTerminalVelocity`
unsafe fn ftCo_GetTerminalVelocity(mut fp: *mut Fighter) -> f32 {
    return (*fp).co_attrs.terminal_velocity;
}

/// `ftCo_800A3908`
pub unsafe fn ftCo_800A3908(mut fp: *mut Fighter, mut arg1: i32) -> i32 {
    melee_core::ct!("ftCo_800A3908");
    let mut ez: f32 = 0.0;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut dist: f32 = 0.0;
    let mut island_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut ex: f32 = 0.0;
    let mut ey: f32 = 0.0;
    let mut valid: i32 = 0;
    let mut grav: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut px: f32 = 0.0;
    let mut island: *mut mp_UnkStruct0 = null_mut();
    let mut ddy: f32 = 0.0;
    let mut t: i32 = 0;
    let mut frames: i32 = 0;
    let mut ok: i32 = 0;
    grav = (*fp).co_attrs.gravity;
    if (grav < 1e-05f32) && (grav > (-1e-05f32)) {
        ok = 1;
    } else {
        ok = 0;
    }
    if ok != 0 {
        frames = 1000;
    } else {
        frames = melee_core::ppc::fctiwz_f32((-((-(*fp).co_attrs.terminal_velocity) - (*fp).pos_delta.y)) / grav);
    }
    {
        island = ext::mpIsland_80458E88.next;
        'l81: while !island.is_null() {
            'c81: {
                if !(ext::ScriptGame_StageJointActive((*island).x28 as i32) != 0) {
                    break 'c81;
                }
                let mut data2: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
                island_pos = (*island).x14;
                ex = island_pos.x;
                {
                    let mut y: f32 = island_pos.y;
                    ey = y;
                }
                ez = island_pos.z;
                if (((ex < ((*fp).cpu.half_width + ext::Stage_GetBlastZoneLeftOffset())) || (ex > (ext::Stage_GetBlastZoneRightOffset() - (*data2).half_width))) || (ey < ((*data2).half_height + ext::Stage_GetBlastZoneBottomOffset()))) || (ey > (ext::Stage_GetBlastZoneTopOffset() - (*data2).half_height)) {
                    ok = 1;
                } else {
                    ok = 0;
                }
                if ok != 0 {
                    break 'c81;
                }
                if ext::grCorneria_801E2E50((*island).x4 as i32) != 0 {
                    break 'c81;
                }
                dx = (*fp).cur_pos.x - ex;
                if dx > 0.0f32 {
                    t = melee_core::ppc::fctiwz_f32(dx / (*fp).co_attrs.air_drift_max);
                    if frames <= 0 {
                        dist = fmadds((*fp).pos_delta.y, t as f32, (*fp).cur_pos.y);
                    } else if t < frames {
                        let mut sqrt_time_store: f32 = 0.0;
                        dist = (((*fp).cur_pos.y as f64) + (-0.5f64).mul_add(((*fp).co_attrs.gravity * melee_runtime::gekko::sqrtf(t as f32)) as f64, ((*fp).pos_delta.y * (t as f32)) as f64)) as f32;
                    } else {
                        let mut sqrt_terminal_store: f32 = 0.0;
                        dist = (((*fp).cur_pos.y as f64) + ((-0.5f64).mul_add(((*fp).co_attrs.gravity * melee_runtime::gekko::sqrtf(frames as f32)) as f64, ((*fp).pos_delta.y * (frames as f32)) as f64) - ((((t - frames) as f32) * ftCo_GetTerminalVelocity(fp)) as f64))) as f32;
                    }
                    if arg1 != 0 {
                        if !((dist + (*data).x558) < ey) {
                            let mut flags0: u32 = 0;
                            let mut line_id0: i32 = 0;
                            let mut floor_normal0: Vec3 = unsafe { core::mem::zeroed() };
                            let mut floor_pos0: Vec3 = unsafe { core::mem::zeroed() };
                            px = ((ex as f64) - 5.0f64) as f32;
                            valid = ftCo_800A3908_inline1(px, ey, core::ptr::addr_of_mut!(floor_pos0) as *mut Vec, core::ptr::addr_of_mut!(floor_normal0) as *mut Vec, core::ptr::addr_of_mut!(line_id0) as *mut i32, core::ptr::addr_of_mut!(flags0) as *mut u32);
                            if (valid != 0) && (!(ftCo_800A3908_inline0(fp, data2, px, ey) != 0)) {
                                ok = 1;
                            } else {
                                ok = 0;
                            }
                            if ok != 0 {
                                ftCo_800A75DC_set_target(fp, (core::ptr::addr_of_mut!((*fp).cpu.x60) as *mut i32) as *const i32, px, ey, 5.0f32);
                                ftCo_800A49B4(fp);
                                return 1;
                            }
                        }
                    } else {
                        let mut flags1: u32 = 0;
                        let mut line_id1: i32 = 0;
                        let mut floor_normal1: Vec3 = unsafe { core::mem::zeroed() };
                        let mut floor_pos1: Vec3 = unsafe { core::mem::zeroed() };
                        px = ((ex as f64) - 5.0f64) as f32;
                        ex = px - (*fp).cur_pos.x;
                        ddy = ey - (*fp).cur_pos.y;
                        valid = ftCo_800A3908_inline1(px, ey, core::ptr::addr_of_mut!(floor_pos1) as *mut Vec, core::ptr::addr_of_mut!(floor_normal1) as *mut Vec, core::ptr::addr_of_mut!(line_id1) as *mut i32, core::ptr::addr_of_mut!(flags1) as *mut u32);
                        if (valid != 0) && (!(ftCo_800A3908_inline0(fp, data2, px, ey) != 0)) {
                            ok = 1;
                        } else {
                            ok = 0;
                        }
                        if ok != 0 {
                            let mut sqrt_dist_store: f32 = 0.0;
                            dist = melee_runtime::gekko::sqrtf(fmadds(ex, ex, ddy * ddy));
                            if (*data).x5C > dist {
                                ftCo_800A75DC_set_target(fp, (core::ptr::addr_of_mut!((*fp).cpu.x60) as *mut i32) as *const i32, px, ey, 5.0f32);
                                ftCo_800A49B4(fp);
                            }
                        }
                    }
                }
            }
            island = (*island).next;
        }
    }
    return 0;
}

/// `ftCo_800A4038_inline0`
unsafe fn ftCo_800A4038_inline0(mut fp: *mut Fighter, mut data: *mut CpuFighter, mut x: f32, mut y: f32) -> i32 {
    if (((x < ((*fp).cpu.half_width + ext::Stage_GetBlastZoneLeftOffset())) || (x > (ext::Stage_GetBlastZoneRightOffset() - (*data).half_width))) || (y < ((*data).half_height + ext::Stage_GetBlastZoneBottomOffset()))) || (y > (ext::Stage_GetBlastZoneTopOffset() - (*data).half_height)) {
        return 1;
    }
    return 0;
}

/// `ftCo_800A4038_inline1`
unsafe fn ftCo_800A4038_inline1(mut x: f32, mut y: f32, mut arg_floor_pos: *mut Vec, mut arg_floor_normal: *mut Vec, mut arg_line_id: *mut i32, mut arg_flags: *mut u32) -> i32 {
    let mut result: i32 = 0;
    let mut valid: i32 = 0;
    *arg_line_id = (-1_i32);
    valid = 0;
    result = ext::mpCheckFloor(x, 5.0f32 + y, x, y - 5.0f32, 0.0f32, arg_floor_pos, arg_line_id, arg_flags, arg_floor_normal, (-1_i32), (-1_i32), (-1_i32), None, null_mut());
    if (result != 0) && (ftCo_800A1B38_noinline(*arg_line_id) != 0) {
        ();
    } else {
        valid = result;
    }
    return valid;
}

/// `ftCo_800A4038`
pub unsafe fn ftCo_800A4038(mut fp: *mut Fighter, mut arg1: i32) -> i32 {
    melee_core::ct!("ftCo_800A4038");
    let mut ez: f32 = 0.0;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut island: *mut mp_UnkStruct0 = null_mut();
    let mut island_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut ex: f32 = 0.0;
    let mut ey: f32 = 0.0;
    let mut valid: i32 = 0;
    let mut grav: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut px: f32 = 0.0;
    let mut dist: f32 = 0.0;
    let mut ddy: f32 = 0.0;
    let mut t: i32 = 0;
    let mut frames: i32 = 0;
    let mut ok: i32 = 0;
    grav = (*fp).co_attrs.gravity;
    if (grav < 1e-05f32) && (grav > (-1e-05f32)) {
        ok = 1;
    } else {
        ok = 0;
    }
    if ok != 0 {
        frames = 1000;
    } else {
        frames = melee_core::ppc::fctiwz_f32((-((-(*fp).co_attrs.terminal_velocity) - (*fp).pos_delta.y)) / grav);
    }
    {
        island = ext::mpIsland_80458E88.next;
        'l82: while !island.is_null() {
            'c82: {
                if !(ext::ScriptGame_StageJointActive((*island).x28 as i32) != 0) {
                    break 'c82;
                }
                let mut data2: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
                island_pos = (*island).x8;
                ex = island_pos.x;
                {
                    let mut y: f32 = island_pos.y;
                    ey = y;
                }
                ez = island_pos.z;
                if (((ex < ((*fp).cpu.half_width + ext::Stage_GetBlastZoneLeftOffset())) || (ex > (ext::Stage_GetBlastZoneRightOffset() - (*data2).half_width))) || (ey < ((*data2).half_height + ext::Stage_GetBlastZoneBottomOffset()))) || (ey > (ext::Stage_GetBlastZoneTopOffset() - (*data2).half_height)) {
                    ok = 1;
                } else {
                    ok = 0;
                }
                if ok != 0 {
                    break 'c82;
                }
                if ext::grCorneria_801E2E50((*island).x4 as i32) != 0 {
                    break 'c82;
                }
                dx = ex - (*fp).cur_pos.x;
                if dx > 0.0f32 {
                    t = melee_core::ppc::fctiwz_f32(dx / (*fp).co_attrs.air_drift_max);
                    if frames <= 0 {
                        dist = fmadds((*fp).pos_delta.y, t as f32, (*fp).cur_pos.y);
                    } else if t < frames {
                        let mut sqrt_time_store: f32 = 0.0;
                        dist = (((*fp).cur_pos.y as f64) + (-0.5f64).mul_add(((*fp).co_attrs.gravity * melee_runtime::gekko::sqrtf(t as f32)) as f64, ((*fp).pos_delta.y * (t as f32)) as f64)) as f32;
                    } else {
                        let mut sqrt_terminal_store: f32 = 0.0;
                        dist = (((*fp).cur_pos.y as f64) + ((-0.5f64).mul_add(((*fp).co_attrs.gravity * melee_runtime::gekko::sqrtf(frames as f32)) as f64, ((*fp).pos_delta.y * (frames as f32)) as f64) - ((((t - frames) as f32) * ftCo_GetTerminalVelocity(fp)) as f64))) as f32;
                    }
                    if arg1 != 0 {
                        if !((dist + (*data).x558) < ey) {
                            let mut flags: u32 = 0;
                            let mut line_id: i32 = 0;
                            let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
                            let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
                            px = (5.0f64 + (ex as f64)) as f32;
                            valid = ftCo_800A4038_inline1(px, ey, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32);
                            if (valid != 0) && (!(ftCo_800A4038_inline0(fp, data2, px, ey) != 0)) {
                                ok = 1;
                            } else {
                                ok = 0;
                            }
                            if ok != 0 {
                                ftCo_800A75DC_set_target(fp, (core::ptr::addr_of_mut!((*fp).cpu.x60) as *mut i32) as *const i32, px, ey, 5.0f32);
                                ftCo_800A49B4(fp);
                                return 1;
                            }
                        }
                    } else {
                        let mut flags: u32 = 0;
                        let mut line_id: i32 = 0;
                        let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
                        let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
                        px = (5.0f64 + (ex as f64)) as f32;
                        ex = px - (*fp).cur_pos.x;
                        ddy = ey - (*fp).cur_pos.y;
                        valid = ftCo_800A4038_inline1(px, ey, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32);
                        if (valid != 0) && (!(ftCo_800A4038_inline0(fp, data2, px, ey) != 0)) {
                            ok = 1;
                        } else {
                            ok = 0;
                        }
                        if ok != 0 {
                            let mut sqrt_dist_store: f32 = 0.0;
                            dist = melee_runtime::gekko::sqrtf(fmadds(ex, ex, ddy * ddy));
                            if (*data).x5C > dist {
                                ftCo_800A75DC_set_target(fp, (core::ptr::addr_of_mut!((*fp).cpu.x60) as *mut i32) as *const i32, px, ey, 5.0f32);
                                ftCo_800A49B4(fp);
                            }
                        }
                    }
                }
            }
            island = (*island).next;
        }
    }
    return 0;
}

/// `ftCo_800A4768_inline0`
unsafe fn ftCo_800A4768_inline0(mut fp: *mut Fighter, mut p: *mut Vec) -> i32 {
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ((((*p).x < ((*data).half_width + ext::Stage_GetBlastZoneLeftOffset())) || ((*p).x > (ext::Stage_GetBlastZoneRightOffset() - (*data).half_width))) || ((*p).y < ((*data).half_height + ext::Stage_GetBlastZoneBottomOffset()))) || ((*p).y > (ext::Stage_GetBlastZoneTopOffset() - (*data).half_height)) {
        return 1;
    }
    return 0;
}

/// `ftCo_800A4768`
pub unsafe fn ftCo_800A4768(mut fp: *mut Fighter, mut arg1: *mut Vec) -> i32 {
    melee_core::ct!("ftCo_800A4768");
    let mut pad: u32 = 0;
    let mut island: *mut mp_UnkStruct0 = null_mut();
    let mut best: f32 = 0.0;
    let mut pt: Vec3 = unsafe { core::mem::zeroed() };
    let mut dx: f32 = 0.0;
    let mut tmp_y: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut tmp_x: f32 = 0.0;
    let mut dist: f32 = 0.0;
    best = -1.0f32;
    {
        island = ext::mpIsland_80458E88.next;
        'l83: while !island.is_null() {
            'c83: {
                if !(ext::ScriptGame_StageJointActive((*island).x28 as i32) != 0) {
                    break 'c83;
                }
                pt = (*island).x14;
                tmp_y = pt.y;
                if !(ftCo_800A4768_inline0(fp, core::ptr::addr_of_mut!(pt) as *mut Vec) != 0) {
                    dx = (*fp).cur_pos.x - pt.x;
                    if dx > 0.0f32 {
                        dy = (*fp).cur_pos.y - pt.y;
                        dist = fmadds(dx, dx, dy * dy);
                        if ((best as f64) < 0.0f64) || (best > dist) {
                            best = dist;
                            tmp_x = ((pt.x as f64) - 5.0f64) as f32;
                            (*arg1).x = tmp_x;
                            (*arg1).y = tmp_y;
                            (*arg1).z = pt.z;
                        }
                    }
                }
                if !(ftCo_800A4768_inline0(fp, core::ptr::addr_of_mut!(pt) as *mut Vec) != 0) {
                    pt = (*island).x8;
                    dx = (*fp).cur_pos.x - pt.x;
                    if dx < 0.0f32 {
                        dy = (*fp).cur_pos.y - pt.y;
                        dist = fmadds(dx, dx, dy * dy);
                        if ((best as f64) < 0.0f64) || (best > dist) {
                            best = dist;
                            (*arg1).x = ((pt.x as f64) + 5.0f64) as f32;
                            (*arg1).y = pt.y;
                            (*arg1).z = pt.z;
                        }
                    }
                }
            }
            island = (*island).next;
        }
    }
    return 0;
}

/// `ftCo_800A49B4`
pub unsafe fn ftCo_800A49B4(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A49B4");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut dx: f32 = (*fp).cur_pos.x - (*data).x54.x;
    let mut dy: f32 = (*fp).cur_pos.y - (*data).x54.y;
    (*data).x5C = melee_runtime::gekko::sqrtf(fmadds(dx, dx, dy * dy));
}

/// `inlineD0`
unsafe fn inlineD0(mut fp: *mut Fighter, mut fp1: *mut Fighter) -> i32 {
    let mut y: f32 = 0.0;
    let mut x: f32 = 0.0;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    y = (*fp1).cur_pos.y;
    x = (*fp1).cur_pos.x;
    if (((x < (ext::Stage_GetBlastZoneLeftOffset() + (*data).half_width)) || (x > (ext::Stage_GetBlastZoneRightOffset() - (*data).half_width))) || (y < (ext::Stage_GetBlastZoneBottomOffset() + (*data).half_height))) || (y > (ext::Stage_GetBlastZoneTopOffset() - (*data).half_height)) {
        return 1;
    }
    return 0;
}

/// `inlineD0_it`
unsafe fn inlineD0_it(mut fp: *mut Fighter, mut it: *mut Item) -> i32 {
    let mut y: f32 = 0.0;
    let mut x: f32 = 0.0;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    y = (*it).pos.y;
    x = (*it).pos.x;
    if (((x < (ext::Stage_GetBlastZoneLeftOffset() + (*data).half_width)) || (x > (ext::Stage_GetBlastZoneRightOffset() - (*data).half_width))) || (y < (ext::Stage_GetBlastZoneBottomOffset() + (*data).half_height))) || (y > (ext::Stage_GetBlastZoneTopOffset() - (*data).half_height)) {
        return 1;
    }
    return 0;
}

/// `inlineD1`
unsafe fn inlineD1(mut fp: *mut Fighter) -> i32 {
    let mut gobj: *mut HSD_GObj = (*fp).gobj;
    if (((((*fp).x221F_b3() as i32) != 0) || (((*fp).x2224_b2() as i32) != 0)) || (ftCo_800A0F00(gobj) != 0)) || (ext::ftLib_IsDead(gobj) != 0) {
        return 1;
    } else {
        return 0;
    }
}

/// `ftCo_800A4A40`
pub unsafe fn ftCo_800A4A40(mut fp: *mut Fighter) -> *mut Fighter {
    melee_core::ct!("ftCo_800A4A40");
    let mut fp1: *mut Fighter = null_mut();
    let mut cur_fp: *mut Fighter = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut closest: f32 = 0.0;
    let mut distance: f32 = 0.0;
    if fp.is_null() {
        return null_mut();
    }
    cur_fp = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l84: while !cur.is_null() {
            'c84: {
                if (*fp).gobj == cur {
                    break 'c84;
                }
                fp1 = HSD_GObjGetUserData(cur) as *mut Fighter;
                if inlineD0(fp, fp1) != 0 {
                    break 'c84;
                }
                if inlineD1(fp1) != 0 {
                    break 'c84;
                }
                if cur_fp.is_null() {
                    cur_fp = fp1;
                    closest = ftCo_800A1AB4(fp, fp1);
                    break 'c84;
                }
                fp1 = HSD_GObjGetUserData(cur) as *mut Fighter;
                distance = ftCo_800A1AB4(fp, fp1);
                if closest > distance {
                    closest = distance;
                    cur_fp = fp1;
                }
            }
            cur = (*cur).next;
        }
    }
    return cur_fp;
}

/// `ftCo_800A4BEC`
pub unsafe fn ftCo_800A4BEC(mut fp: *mut Fighter) -> *mut Fighter {
    melee_core::ct!("ftCo_800A4BEC");
    let mut cur_fp: *mut Fighter = null_mut();
    let mut closest: *mut Fighter = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut best: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut dist: f32 = 0.0;
    if fp.is_null() {
        return null_mut();
    }
    closest = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l85: while !cur.is_null() {
            if (*fp).gobj != cur {
                cur_fp = HSD_GObjGetUserData(cur) as *mut Fighter;
                if (!(inlineD0(fp, cur_fp) != 0)) && (!(ftCo_IsAlly_dontinline(fp, cur_fp) != 0)) {
                    if !(ftCo_800A1C44(cur_fp) != 0) {
                        if !(inlineD1(cur_fp) != 0) {
                            if (*data).xF9_b0() != 0 {
                                if cur_fp == ((*data).x48 as *mut Fighter) {
                                    return cur_fp;
                                }
                            } else if closest.is_null() {
                                closest = cur_fp;
                                best = ftCo_800A1AB4(fp, cur_fp);
                            } else {
                                cur_fp = HSD_GObjGetUserData(cur) as *mut Fighter;
                                dist = ftCo_800A1AB4(fp, cur_fp);
                                if best > dist {
                                    best = dist;
                                    closest = cur_fp;
                                }
                            }
                        }
                    }
                }
            }
            cur = (*cur).next;
        }
    }
    if closest.is_null() {
        (*data).set_xF9_b0((0 as u8) as u8);
    } else {
        (*data).set_xF9_b0((1 as u8) as u8);
        dist = ext::HSD_Randf();
        (*data).x30 = melee_core::ppc::fctiwz_f32(1.2e+02f32 * (0.5f32 * (0.5f32 * dist)));
        (*data).x48 = closest as *mut c_void;
    }
    return closest;
}

/// `ftCo_800A4E8C`
pub unsafe fn ftCo_800A4E8C(mut fp: *mut Fighter, mut arg1: *mut Vec) -> *mut Fighter {
    melee_core::ct!("ftCo_800A4E8C");
    let mut cur_fp: *mut Fighter = null_mut();
    let mut closest_fp: *mut Fighter = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut closest: f32 = 0.0;
    let mut distance: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut dz: f32 = 0.0;
    if fp.is_null() {
        return null_mut();
    }
    closest_fp = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l86: while !cur.is_null() {
            if (*fp).gobj != cur {
                cur_fp = HSD_GObjGetUserData(cur) as *mut Fighter;
                if (!(inlineD0(fp, cur_fp) != 0)) && (!(ftCo_IsAlly_dontinline(fp, cur_fp) != 0)) {
                    if !(ftCo_800A1C44(cur_fp) != 0) {
                        if !(inlineD1(cur_fp) != 0) {
                            if closest_fp.is_null() {
                                closest_fp = cur_fp;
                                distance = { dx = (*cur_fp).cur_pos.x; dx } - (*arg1).x;
                                dy = (*cur_fp).cur_pos.y - (*arg1).y;
                                dx = distance;
                                dz = (*cur_fp).cur_pos.z - (*arg1).z;
                                closest = melee_runtime::gekko::sqrtf(fmadds(dz, dz, fmadds(dx, dx, dy * dy)));
                            } else {
                                dx = (*cur_fp).cur_pos.x - (*arg1).x;
                                dz = (*cur_fp).cur_pos.z - (*arg1).z;
                                dy = (*cur_fp).cur_pos.y - (*arg1).y;
                                distance = melee_runtime::gekko::sqrtf(fmadds(dz, dz, fmadds(dx, dx, dy * dy)));
                                if closest > distance {
                                    closest = distance;
                                    closest_fp = cur_fp;
                                }
                            }
                        }
                    }
                }
            }
            cur = (*cur).next;
        }
    }
    return closest_fp;
}

/// `ftCo_800A50D4`
pub unsafe fn ftCo_800A50D4(mut fp: *mut Fighter) -> *mut Fighter {
    melee_core::ct!("ftCo_800A50D4");
    let mut cur_fp: *mut Fighter = null_mut();
    let mut closest_fp: *mut Fighter = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut closest: f32 = 0.0;
    let mut distance: f32 = 0.0;
    if fp.is_null() {
        return null_mut();
    }
    closest_fp = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l87: while !cur.is_null() {
            'c87: {
                if (*fp).gobj == cur {
                    break 'c87;
                }
                cur_fp = HSD_GObjGetUserData(cur) as *mut Fighter;
                if inlineD0(fp, cur_fp) != 0 {
                    break 'c87;
                }
                if !(ftCo_IsAlly_dontinline(fp, cur_fp) != 0) {
                    break 'c87;
                }
                if inlineD1(cur_fp) != 0 {
                    break 'c87;
                }
                if closest_fp.is_null() {
                    closest_fp = cur_fp;
                    closest = ftCo_800A1AB4(fp, cur_fp);
                } else {
                    cur_fp = HSD_GObjGetUserData(cur) as *mut Fighter;
                    distance = ftCo_800A1AB4(fp, cur_fp);
                    if closest > distance {
                        closest = distance;
                        closest_fp = cur_fp;
                    }
                }
            }
            cur = (*cur).next;
        }
    }
    return closest_fp;
}

/// `ftCo_800A5294`
pub unsafe fn ftCo_800A5294(mut fp: *mut Fighter, mut player_id: i32) -> *mut Fighter {
    melee_core::ct!("ftCo_800A5294");
    if fp.is_null() {
        return null_mut();
    }
    {
        let mut cur_fp: *mut Fighter = null_mut();
        let mut cur: *mut HSD_GObj = null_mut();
        {
            cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
            'l88: while !cur.is_null() {
                if (*fp).gobj != cur {
                    cur_fp = HSD_GObjGetUserData(cur) as *mut Fighter;
                    if !(inlineD0(fp, cur_fp) != 0) {
                        if (!(inlineD1(cur_fp) != 0)) && (((*cur_fp).player_id as i32) == player_id) {
                            return cur_fp;
                        }
                    }
                }
                cur = (*cur).next;
            }
        }
    }
    return null_mut();
}

/// `ftCo_800A53DC`
pub unsafe fn ftCo_800A53DC(mut fp: *mut Fighter) -> *mut Fighter {
    melee_core::ct!("ftCo_800A53DC");
    let mut temp_r31: *mut CpuFighter = null_mut();
    let mut var_r30: *mut HSD_GObj = null_mut();
    let mut var_r30_2: *mut HSD_GObj = null_mut();
    let mut temp_r29: *mut Fighter = null_mut();
    let mut temp_r29_3: *mut Fighter = null_mut();
    let mut var_r28: *mut Fighter = null_mut();
    let mut var_r27: i32 = 0;
    let mut temp_r25: *mut HSD_GObj = null_mut();
    let mut temp_r25_2: *mut Fighter = null_mut();
    let mut temp_r25_3: *mut HSD_GObj = null_mut();
    let mut temp_r25_4: *mut Fighter = null_mut();
    let mut temp_f1_3: f32 = 0.0;
    let mut temp_f1_6: f32 = 0.0;
    let mut var_f31: f32 = 0.0;
    temp_r31 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if fp.is_null() {
        return null_mut();
    }
    var_r28 = null_mut();
    if (*temp_r31).xFA_b34() != 0 {
        {
            var_r30 = *ext::HSD_GObjPLinkHead.offset(8 as isize);
            'l89: while !var_r30.is_null() {
                'c89: {
                    if (*fp).gobj == var_r30 {
                        break 'c89;
                    }
                    temp_r29 = HSD_GObjGetUserData(var_r30) as *mut Fighter;
                    if inlineD0(fp, temp_r29) != 0 {
                        break 'c89;
                    }
                    if ftCo_IsAlly_dontinline(fp, temp_r29) != 0 {
                        break 'c89;
                    }
                    if ftCo_800A1C44(temp_r29) != 0 {
                        break 'c89;
                    }
                    temp_r25 = (*temp_r29).gobj;
                    if inlineD1(temp_r29) != 0 {
                        break 'c89;
                    }
                    if ftCo_IsCpuControlled(temp_r29) != 0 {
                        break 'c89;
                    }
                    if var_r28.is_null() {
                        var_r28 = temp_r29;
                        var_r27 = ext::gm_8016C6C0(var_r30);
                        var_f31 = ftCo_800A1AB4(fp, temp_r29);
                    } else {
                        let mut temp_r29_2: i32 = 0;
                        temp_r25_2 = HSD_GObjGetUserData(var_r30) as *mut Fighter;
                        temp_r29_2 = ext::gm_8016C6C0(var_r30);
                        temp_f1_3 = ftCo_800A1AB4(fp, temp_r25_2);
                        if var_r27 > temp_r29_2 {
                            var_f31 = temp_f1_3;
                            var_r27 = temp_r29_2;
                            var_r28 = temp_r25_2;
                        } else if (var_r27 == temp_r29_2) && (var_f31 > temp_f1_3) {
                            var_f31 = temp_f1_3;
                            var_r28 = temp_r25_2;
                        }
                    }
                }
                var_r30 = (*var_r30).next;
            }
        }
        if !var_r28.is_null() {
            return var_r28;
        }
    }
    {
        var_r30_2 = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l90: while !var_r30_2.is_null() {
            'c90: {
                if (*fp).gobj == var_r30_2 {
                    break 'c90;
                }
                temp_r29_3 = HSD_GObjGetUserData(var_r30_2) as *mut Fighter;
                if inlineD0(fp, temp_r29_3) != 0 {
                    break 'c90;
                }
                if ftCo_IsAlly_dontinline(fp, temp_r29_3) != 0 {
                    break 'c90;
                }
                if ftCo_800A1C44(temp_r29_3) != 0 {
                    break 'c90;
                }
                temp_r25_3 = (*temp_r29_3).gobj;
                if inlineD1(temp_r29_3) != 0 {
                    break 'c90;
                }
                if var_r28.is_null() {
                    var_r28 = temp_r29_3;
                    var_r27 = ext::gm_8016C6C0(var_r30_2);
                    var_f31 = ftCo_800A1AB4(fp, temp_r29_3);
                } else {
                    let mut temp_r29_4: i32 = 0;
                    temp_r25_4 = HSD_GObjGetUserData(var_r30_2) as *mut Fighter;
                    temp_r29_4 = ext::gm_8016C6C0(var_r30_2);
                    temp_f1_6 = ftCo_800A1AB4(fp, temp_r25_4);
                    if var_r27 > temp_r29_4 {
                        var_f31 = temp_f1_6;
                        var_r27 = temp_r29_4;
                        var_r28 = temp_r25_4;
                    } else if (var_r27 == temp_r29_4) && (var_f31 > temp_f1_6) {
                        var_f31 = temp_f1_6;
                        var_r28 = temp_r25_4;
                    }
                }
            }
            var_r30_2 = (*var_r30_2).next;
        }
    }
    return var_r28;
}

/// `ftCo_800A589C`
pub unsafe fn ftCo_800A589C(mut fp: *mut Fighter) -> *mut Fighter {
    melee_core::ct!("ftCo_800A589C");
    if fp.is_null() {
        return null_mut();
    }
    {
        let mut cur: *mut HSD_GObj = null_mut();
        {
            cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
            'l91: while !cur.is_null() {
                if (*fp).gobj != cur {
                    let mut cur_fp: *mut Fighter = HSD_GObjGetUserData(cur) as *mut Fighter;
                    if ((*fp).player_id as i32) == ((*cur_fp).player_id as i32) {
                        if (*cur_fp).x221F_b3() != 0 {
                            return null_mut();
                        }
                        return cur_fp;
                    }
                }
                cur = (*cur).next;
            }
        }
    }
    return null_mut();
}

/// `ftCo_800A5908`
pub unsafe fn ftCo_800A5908(mut ip: *mut Item) -> i32 {
    melee_core::ct!("ftCo_800A5908");
    if (*ip).kind == It_Kind_Heart {
        return 1;
    }
    if (*ip).kind == It_Kind_Tomato {
        return 1;
    }
    if (*ip).kind == It_Kind_Foods {
        return 1;
    }
    return 0;
}

/// `ftCo_800A5944`
pub unsafe fn ftCo_800A5944(mut ip: *mut Item) -> i32 {
    melee_core::ct!("ftCo_800A5944");
    if (*ip).kind == It_Kind_BombHei {
        return 1;
    }
    if (*ip).kind == It_Kind_G_Shell {
        return 1;
    }
    if (*ip).kind == It_Kind_R_Shell {
        return 1;
    }
    return 0;
}

/// `ftCo_800A5980_dontinline`
pub unsafe fn ftCo_800A5980_dontinline(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A5980_dontinline");
    return ftCo_800A5980(fp);
}

/// `ftCo_800A5980`
pub unsafe fn ftCo_800A5980(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A5980");
    if ((*fp).motion_id >= ftCo_MS_LandingFallSpecial) && ((*fp).motion_id < ftCo_MS_Attack100Start) {
        return 1;
    }
    if ((((*fp).motion_id == ftCo_MS_Furafura) || ((*fp).motion_id == ftCo_MS_Catch)) || ((*fp).motion_id == ftCo_MS_CatchPull)) || ((*fp).motion_id == ftCo_MS_CatchAttack) {
        return 1;
    }
    return 0;
}

/// `ftCo_800A59C0`
pub unsafe fn ftCo_800A59C0(mut ip: *mut Item) -> i32 {
    melee_core::ct!("ftCo_800A59C0");
    if ((*(*ip).xCC_item_attr).x0_78() as i32) == 3 {
        return 1;
    }
    return 0;
}

/// `ftCo_800A59E4`
pub unsafe fn ftCo_800A59E4(mut ip: *mut Item) -> i32 {
    melee_core::ct!("ftCo_800A59E4");
    if ip.is_null() {
        return 0;
    }
    if ((((((*ip).kind == It_Kind_Harisen) || ((*ip).kind == It_Kind_StarRod)) || ((*ip).kind == It_Kind_LipStick)) || ((*ip).kind == It_Kind_Bat)) || ((*ip).kind == It_Kind_Sword)) || ((*ip).kind == It_Kind_Parasol) {
        return 1;
    }
    return 0;
}

/// `ftCo_800A5A28`
pub unsafe fn ftCo_800A5A28(mut ip: *mut Item) -> i32 {
    melee_core::ct!("ftCo_800A5A28");
    's92: {
        match (*ip).kind {
            0 | 6 | 7 | 14 | 15 | 17 | 19 | 20 | 34 => {
                return 1;
            }
            _ => {
                return 0;
            }
        }
    }
}

/// `ftCo_800A5A5C`
pub unsafe fn ftCo_800A5A5C(mut ip: *mut Item) -> i32 {
    melee_core::ct!("ftCo_800A5A5C");
    's93: {
        match (*ip).kind {
            1 | 2 | 4 | 5 => {
                return 1;
            }
            _ => {
                return 0;
            }
        }
    }
}

/// `ftCo_800A5A90`
pub unsafe fn ftCo_800A5A90(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A5A90");
    if ((*fp).motion_id >= ftCo_MS_SwordSwing1) && ((*fp).motion_id <= ftCo_MS_StarRodSwingDash) {
        return 1;
    }
    if ((*fp).motion_id >= ftCo_MS_LGunShoot) && ((*fp).motion_id <= ftCo_MS_ItemScopeAirEndEmpty) {
        return 1;
    }
    return 0;
}

/// `ftCo_800A5ACC`
pub unsafe fn ftCo_800A5ACC(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A5ACC");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if !((*data).xF9_b6() != 0) {
        return 0;
    }
    's94: {
        match (*data).level {
            0 | 1 => {
                return 0;
            }
            2 => {
                if ((*data).x7C % 120) > 100 {
                    return 1;
                }
                return 0;
            }
            3 => {
                if ((*data).x7C % 120) > 80 {
                    return 1;
                }
                return 0;
            }
            4 => {
                if ((*data).x7C % 120) > 60 {
                    return 1;
                }
                return 0;
            }
            5 => {
                if ((*data).x7C % 120) > 40 {
                    return 1;
                }
                return 0;
            }
            6 => {
                if ((*data).x7C % 120) > 30 {
                    return 1;
                }
                return 0;
            }
            7 => {
                if ((*data).x7C % 120) > 20 {
                    return 1;
                }
                return 0;
            }
            8 => {
                if ((*data).x7C % 120) > 10 {
                    return 1;
                }
                return 0;
            }
            9 => {
                return 1;
            }
            _ => {}
        }
    }
    return 0;
}

/// `ftCo_800A5CE0`
pub unsafe fn ftCo_800A5CE0(mut fp: *mut Fighter) -> *mut Fighter {
    melee_core::ct!("ftCo_800A5CE0");
    let mut cur_fp: *mut Fighter = null_mut();
    let mut closest_enemy: *mut Fighter = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut var_r0_3: i32 = 0;
    let mut distance: f32 = 0.0;
    let mut closest: f32 = 0.0;
    if fp.is_null() {
        return null_mut();
    }
    if ftCo_800A1C44(fp) != 0 {
        return null_mut();
    }
    closest_enemy = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(8 as isize);
        'l95: while !cur.is_null() {
            'c95: {
                if (*fp).gobj == cur {
                    break 'c95;
                }
                cur_fp = HSD_GObjGetUserData(cur) as *mut Fighter;
                if inlineD1(cur_fp) != 0 {
                    break 'c95;
                }
                if (*cur_fp).x221D_b6() != 0 {
                    var_r0_3 = 1;
                } else if ((*cur_fp).x2168 != 0) && ((*cur_fp).x2338.x == 0) {
                    var_r0_3 = 1;
                } else {
                    var_r0_3 = 0;
                }
                if !(var_r0_3 != 0) {
                    break 'c95;
                }
                if ftCo_IsAlly_dontinline(fp, cur_fp) != 0 {
                    break 'c95;
                }
                if inlineD0(fp, cur_fp) != 0 {
                    break 'c95;
                }
                if closest_enemy.is_null() {
                    closest_enemy = cur_fp;
                    closest = ftCo_800A1AB4(fp, cur_fp);
                } else {
                    distance = ftCo_800A1AB4(fp, cur_fp);
                    if closest > distance {
                        closest = distance;
                        closest_enemy = cur_fp;
                    }
                }
            }
            cur = (*cur).next;
        }
    }
    return closest_enemy;
}

/// `ftCo_GetItemDistance`
unsafe fn ftCo_GetItemDistance(mut fp: *mut Fighter, mut ip: *mut Item) -> f32 {
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    if ip.is_null() {
        return 1e+04f32;
    }
    dx = (*fp).cur_pos.x - (*ip).pos.x;
    dy = (*fp).cur_pos.y - (*ip).pos.y;
    return melee_runtime::gekko::sqrtf(fmadds(dx, dx, dy * dy));
}

/// `ftCo_800A5F4C`
pub unsafe fn ftCo_800A5F4C(mut fp: *mut Fighter, mut arg1: ItemKind) -> *mut Item {
    melee_core::ct!("ftCo_800A5F4C");
    let mut cur_ip: *mut Item = null_mut();
    let mut closest_ip: *mut Item = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut data: *mut CpuFighter = null_mut();
    let mut closest: f32 = 0.0;
    let mut distance: f32 = 0.0;
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if fp.is_null() {
        return null_mut();
    }
    closest_ip = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(9 as isize);
        'l96: while !cur.is_null() {
            'c96: {
                cur_ip = HSD_GObjGetUserData(cur) as *mut Item;
                cur_ip = HSD_GObjGetUserData(cur) as *mut Item;
                if !(ext::Item_IsGrabbable(cur) != 0) {
                    break 'c96;
                }
                if (arg1 != It_Kind_L_Gun_Ray) && ((*cur_ip).kind != arg1) {
                    break 'c96;
                }
                if inlineD0_it(fp, cur_ip) != 0 {
                    break 'c96;
                }
                if (*cur_ip).kind >= It_Kind_L_Gun_Ray {
                    break 'c96;
                }
                if ftCo_803C5A68[(*cur_ip).kind as usize] < (*data).x2C {
                    break 'c96;
                }
                if closest_ip.is_null() {
                    closest_ip = cur_ip;
                    closest = ftCo_GetItemDistance(fp, cur_ip);
                    break 'c96;
                }
                if ftCo_803C5A68[(*cur_ip).kind as usize] < ftCo_803C5A68[(*closest_ip).kind as usize] {
                    break 'c96;
                }
                distance = ftCo_GetItemDistance(fp, cur_ip);
                if closest > distance {
                    closest = distance;
                    closest_ip = cur_ip;
                }
            }
            cur = HSD_GObjGetNext(cur);
        }
    }
    return closest_ip;
}

/// `ftCo_800A61D8`
pub unsafe fn ftCo_800A61D8(mut fp: *mut Fighter) -> *mut Item {
    melee_core::ct!("ftCo_800A61D8");
    let mut ip: *mut Item = null_mut();
    let mut closest: *mut Item = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut data: *mut CpuFighter = null_mut();
    let mut best: f32 = 0.0;
    let mut dist: f32 = 0.0;
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if fp.is_null() {
        return null_mut();
    }
    closest = null_mut();
    {
        cur = *ext::HSD_GObjPLinkHead.offset(9 as isize);
        'l97: while !cur.is_null() {
            'c97: {
                ip = HSD_GObjGetUserData(cur) as *mut Item;
                ip = HSD_GObjGetUserData(cur) as *mut Item;
                ip = HSD_GObjGetUserData(cur) as *mut Item;
                if !(ext::Item_IsGrabbable(cur) != 0) {
                    break 'c97;
                }
                if !(ftCo_800A5908(ip) != 0) {
                    break 'c97;
                }
                if inlineD0_it(fp, ip) != 0 {
                    break 'c97;
                }
                if (*ip).kind >= It_Kind_L_Gun_Ray {
                    break 'c97;
                }
                if ftCo_803C5A68[(*ip).kind as usize] < (*data).x2C {
                    break 'c97;
                }
                if closest.is_null() {
                    closest = ip;
                    best = ftCo_GetItemDistance(fp, ip);
                    break 'c97;
                }
                if ftCo_803C5A68[(*ip).kind as usize] >= ftCo_803C5A68[(*closest).kind as usize] {
                    dist = ftCo_GetItemDistance(fp, ip);
                    if best > dist {
                        best = dist;
                        closest = ip;
                    }
                }
            }
            cur = (*cur).next;
        }
    }
    return closest;
}

/// `ftCo_800A648C_inline1`
unsafe fn ftCo_800A648C_inline1(mut ip: *mut Item) -> i32 {
    if ((*ip).kind >= It_Kind_Kuriboh) && ((*ip).kind < It_Kind_Octarock_Stone) {
        return 1;
    }
    if (((*ip).kind == It_Kind_Nokonoko) || ((((*ip).kind - It_Kind_Patapata) as u32) <= (1 as u32))) || ((*ip).kind == It_Kind_Whitebea) {
        return 1;
    }
    return 0;
}

/// `ftCo_800A648C_inline2`
unsafe fn ftCo_800A648C_inline2() -> *mut HSD_GObj {
    return *ext::HSD_GObjPLinkHead.offset(9 as isize);
}

/// `ftCo_800A648C_inline3`
unsafe fn ftCo_800A648C_inline3(mut cur: *mut HSD_GObj) -> *mut HSD_GObj {
    return (*cur).next;
}

/// `ftCo_800A648C`
pub unsafe fn ftCo_800A648C(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800A648C");
    let mut ip: *mut Item = null_mut();
    let mut closest: *mut Item = null_mut();
    let mut cur: *mut HSD_GObj = null_mut();
    let mut best: f32 = 0.0;
    let mut dist: f32 = 0.0;
    if fp.is_null() {
        return 0;
    }
    closest = null_mut();
    {
        cur = ftCo_800A648C_inline2();
        'l98: while !cur.is_null() {
            ip = HSD_GObjGetUserData(cur) as *mut Item;
            if ftCo_800A648C_inline1(ip) != 0 {
                if !(inlineD0_it(fp, ip) != 0) {
                    if closest.is_null() {
                        closest = ip;
                        best = ftCo_GetItemDistance(fp, ip);
                    } else {
                        dist = ftCo_GetItemDistance(fp, ip);
                        if best > dist {
                            best = dist;
                            closest = ip;
                        }
                    }
                }
            }
            cur = ftCo_800A648C_inline3(cur);
        }
    }
    return closest as usize as i32;
}

/// `ftCo_800A6700_inline0`
unsafe fn ftCo_800A6700_inline0(mut fp: *mut Fighter, mut x: f32, mut y: f32) -> i32 {
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (((x < ((*data).half_width + ext::Stage_GetBlastZoneLeftOffset())) || (x > (ext::Stage_GetBlastZoneRightOffset() - (*data).half_width))) || (y < ((*data).half_height + ext::Stage_GetBlastZoneBottomOffset()))) || (y > (ext::Stage_GetBlastZoneTopOffset() - (*data).half_height)) {
        return 1;
    }
    return 0;
}

/// `ftCo_800A6700`
pub unsafe fn ftCo_800A6700(mut fp: *mut Fighter, mut arg1: *mut Vec, mut arg2: *mut Vec) -> i32 {
    melee_core::ct!("ftCo_800A6700");
    let mut island: *mut mp_UnkStruct0 = null_mut();
    let mut best: f32 = 0.0;
    let mut px: f32 = 0.0;
    let mut ay: f32 = 0.0;
    let mut ax: f32 = 0.0;
    let mut b: Vec3 = unsafe { core::mem::zeroed() };
    let mut a: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut dy: f32 = 0.0;
    let mut dist: f32 = 0.0;
    let mut result: i32 = 0;
    best = -1.0f32;
    {
        island = ext::mpIsland_80458E88.next;
        'l99: while !island.is_null() {
            'c99: {
                if !(ext::ScriptGame_StageJointActive((*island).x28 as i32) != 0) {
                    break 'c99;
                }
                if ftCo_800A2718(island) == 0 {
                    let mut line_id: i32 = 0;
                    let mut flags: u32 = 0;
                    let mut by: f32 = 0.0;
                    a = (*island).x8;
                    ax = a.x;
                    ay = a.y;
                    px = ((ax as f64) + 5.0f64) as f32;
                    result = ftCo_800A0FB0(core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), ((ax as f64) + 5.0f64) as f32, ((ay as f64) + 5.0f64) as f32, ((ax as f64) + 5.0f64) as f32, ((ay as f64) - 5.0f64) as f32, 0.0f32);
                    if result != 0 {
                        if !(ftCo_800A6700_inline0(fp, px, ay) != 0) {
                            px = px - (*arg1).x;
                            dy = ay - (*arg1).y;
                            dist = fmadds(px, px, dy * dy);
                            if dist > best {
                                best = dist;
                                (*arg2).x = ((ax as f64) + 5.0f64) as f32;
                                (*arg2).y = ay;
                                (*arg2).z = a.z;
                            }
                        }
                    }
                    b = (*island).x14;
                    ax = b.x;
                    by = b.y;
                    px = ((ax as f64) - 5.0f64) as f32;
                    result = ftCo_800A0FB0(core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), px, ((by as f64) + 5.0f64) as f32, px, ((by as f64) - 5.0f64) as f32, 0.0f32);
                    if result != 0 {
                        if !(ftCo_800A6700_inline0(fp, px, by) != 0) {
                            px = px - (*arg1).x;
                            dy = by - (*arg1).y;
                            dist = fmadds(px, px, dy * dy);
                            if dist > best {
                                best = dist;
                                (*arg2).x = ((ax as f64) - 5.0f64) as f32;
                                (*arg2).y = by;
                                (*arg2).z = b.z;
                            }
                        }
                    }
                }
            }
            island = (*island).next;
        }
    }
    if (best as f64) < 0.0f64 {
        return 0;
    }
    return 1;
}

/// `ftCo_800A6A98`
pub unsafe fn ftCo_800A6A98(mut fp: *mut Fighter, mut arg1: *mut Vec) -> i32 {
    melee_core::ct!("ftCo_800A6A98");
    let mut __: [u8; 4] = [0; 4];
    let mut dist: f32 = 0.0;
    let mut b: Vec3 = unsafe { core::mem::zeroed() };
    let mut a: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut flags: u32 = 0;
    let mut mx: f32 = 0.0;
    let mut my: f32 = 0.0;
    let mut result: i32 = 0;
    let mut blocked: i32 = 0;
    let mut line: i32 = 0;
    let mut fy: f32 = 0.0;
    let mut fx: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut island: *mut mp_UnkStruct0 = null_mut();
    let mut best: f32 = 0.0;
    best = -1.0f32;
    {
        island = ext::mpIsland_80458E88.next;
        'l100: while !island.is_null() {
            'c100: {
                if !(ext::ScriptGame_StageJointActive((*island).x28 as i32) != 0) {
                    break 'c100;
                }
                if ftCo_800A2718(island) == 0 {
                    a = (*island).x8;
                    b = (*island).x14;
                    blocked = 0;
                    line_id = (-1_i32);
                    mx = 0.5f32 * (b.x + a.x);
                    my = 0.5f32 * (b.y + a.y);
                    result = ext::mpCheckFloor(mx, 5.0f32 + my, mx, my - 2e+01f32, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, blocked as usize as *mut HSD_GObj);
                    if result != 0 {
                        line = line_id;
                        if (((ext::grBigBlue_801EF844(line) != 0) || (ext::grInishie1_801FCAAC(line) != 0)) || (ext::grCorneria_801E2D90(line) != 0)) || (ext::grVenom_80206D10(line) != 0) {
                            blocked = 1;
                        }
                        if blocked != 0 {
                            result = 0;
                        }
                    }
                    if ((result != 0) && (line_id != (-1_i32))) && ((ext::mpLineGetFlags(line_id) & ((1 << 8) as u32)) != 0) {
                        fy = floor_pos.y;
                        fx = floor_pos.x;
                        ();
                        if !(fy < ((*fp).cur_pos.y + (*data).x568)) {
                            if !(ftCo_800A6700_inline0(fp, fx, fy) != 0) {
                                dx = fx - (*fp).cur_pos.x;
                                dy = fy - (*fp).cur_pos.y;
                                dx = dx * dx;
                                dy = dy * dy;
                                dist = dx + dy;
                                if ((best as f64) < 0.0f64) || (dist < best) {
                                    (*arg1).x = fx;
                                    best = dist;
                                    (*arg1).y = fy;
                                    (*arg1).z = 0.0f32;
                                }
                            }
                        }
                    }
                }
            }
            island = (*island).next;
        }
    }
    if (best as f64) < 0.0f64 {
        return 0;
    }
    return 1;
}

/// `ftCo_800A6D2C_inline0`
unsafe fn ftCo_800A6D2C_inline0(mut mx: f32, mut my: f32, mut fpos: *mut Vec, mut lid: *mut i32, mut fl: *mut u32, mut r#fn: *mut Vec, mut blocked: i32) -> i32 {
    return ext::mpCheckFloor(mx, 5.0f32 + my, mx, my - 2e+01f32, 0.0f32, fpos, lid, fl, r#fn, (-1_i32), (-1_i32), (-1_i32), None, blocked as usize as *mut HSD_GObj);
}

/// `ftCo_800A6D2C`
pub unsafe fn ftCo_800A6D2C(mut fp: *mut Fighter, mut arg1: *mut Vec) -> i32 {
    melee_core::ct!("ftCo_800A6D2C");
    let mut dist: f32 = 0.0;
    let mut b: Vec3 = unsafe { core::mem::zeroed() };
    let mut a: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut flags: u32 = 0;
    let mut mx: f32 = 0.0;
    let mut my: f32 = 0.0;
    let mut result: i32 = 0;
    let mut cur_island: *mut mp_UnkStruct0 = null_mut();
    let mut blocked: i32 = 0;
    let mut line: i32 = 0;
    let mut fy: f32 = 0.0;
    let mut fx: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut island: *mut mp_UnkStruct0 = null_mut();
    let mut best: f32 = 0.0;
    best = -1.0f32;
    cur_island = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
    {
        island = ext::mpIsland_80458E88.next;
        'l101: while !island.is_null() {
            'c101: {
                if !(ext::ScriptGame_StageJointActive((*island).x28 as i32) != 0) {
                    break 'c101;
                }
                if (ftCo_800A2718(island) == 0) && (island != cur_island) {
                    a = (*island).x8;
                    b = (*island).x14;
                    let _ = ext::HSD_Randf();
                    blocked = 0;
                    line_id = (-1_i32);
                    mx = 0.5f32 * (b.x + a.x);
                    my = 0.5f32 * (b.y + a.y);
                    result = ftCo_800A6D2C_inline0(mx, my, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, blocked);
                    if result != 0 {
                        line = line_id;
                        if (((ext::grBigBlue_801EF844(line) != 0) || (ext::grInishie1_801FCAAC(line) != 0)) || (ext::grCorneria_801E2D90(line) != 0)) || (ext::grVenom_80206D10(line) != 0) {
                            blocked = 1;
                        }
                        if blocked != 0 {
                            result = 0;
                        }
                    }
                    if result != 0 {
                        fy = floor_pos.y;
                        fx = floor_pos.x;
                        ();
                        if !(fy >= ((*fp).cur_pos.y - (*data).x558)) {
                            if !(ftCo_800A6700_inline0(fp, fx, fy) != 0) {
                                dx = fx - (*fp).cur_pos.x;
                                dy = fy - (*fp).cur_pos.y;
                                dx = dx * dx;
                                dy = dy * dy;
                                dist = dx + dy;
                                if ((best as f64) < 0.0f64) || (dist < best) {
                                    (*arg1).x = fx;
                                    best = dist;
                                    (*arg1).y = fy;
                                    (*arg1).z = 0.0f32;
                                }
                            }
                        }
                    }
                }
            }
            island = (*island).next;
        }
    }
    if (best as f64) < 0.0f64 {
        return 0;
    }
    return 1;
}

/// `ftCo_800A6FC4`
pub unsafe fn ftCo_800A6FC4(mut fp: *mut Fighter, mut arg1: *mut Vec, mut arg2: *mut Vec) -> i32 {
    melee_core::ct!("ftCo_800A6FC4");
    let mut __: [u8; 8] = [0; 8];
    let mut b: Vec3 = unsafe { core::mem::zeroed() };
    let mut a: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut flags: u32 = 0;
    let mut dir: Vec3 = unsafe { core::mem::zeroed() };
    let mut island: *mut mp_UnkStruct0 = null_mut();
    let mut cur_island: *mut mp_UnkStruct0 = null_mut();
    let mut data: *mut CpuFighter = null_mut();
    let mut px: f32 = 0.0;
    let mut floor_y: f32 = 0.0;
    let mut fx: f32 = 0.0;
    let mut fy: f32 = 0.0;
    let mut mx: f32 = 0.0;
    let mut my: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut dist2: f32 = 0.0;
    let mut dist: f32 = 0.0;
    let mut best: f32 = 0.0;
    let mut line: i32 = 0;
    let mut blocked: i32 = 0;
    let mut result: i32 = 0;
    best = -1.0f32;
    cur_island = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
    let _ = ext::lbVector_Normalize(arg2);
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    {
        island = ext::mpIsland_80458E88.next;
        'l102: while !island.is_null() {
            'c102: {
                if !(ext::ScriptGame_StageJointActive((*island).x28 as i32) != 0) {
                    break 'c102;
                }
                if island == cur_island {
                    break 'c102;
                }
                b = (*island).x14;
                a = (*island).x8;
                let _ = ext::HSD_Randf();
                line_id = (-1_i32);
                blocked = 0;
                mx = 0.5f32 * (a.x + b.x);
                my = 0.5f32 * (a.y + b.y);
                result = ext::mpCheckFloor(mx, (10.0f64 + (my as f64)) as f32, mx, ((my as f64) - 10.0f64) as f32, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, blocked as usize as *mut HSD_GObj);
                if result != 0 {
                    line = line_id;
                    if (((ext::grBigBlue_801EF844(line) != 0) || (ext::grInishie1_801FCAAC(line) != 0)) || (ext::grCorneria_801E2D90(line) != 0)) || (ext::grVenom_80206D10(line) != 0) {
                        blocked = 1;
                    }
                    if blocked != 0 {
                        result = 0;
                    }
                }
                if result == 0 {
                    break 'c102;
                }
                fx = floor_pos.x;
                fy = floor_pos.y;
                if ext::grRCruise_80201988(line_id) != 0 {
                    break 'c102;
                }
                dir.x = fx - (*fp).cur_pos.x;
                dir.y = fy - (*fp).cur_pos.y;
                dir.z = 0.0f32;
                let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(dir) as *mut Vec);
                if ext::lbVector_Angle(arg2, core::ptr::addr_of_mut!(dir) as *mut Vec) > 0.7853982f32 {
                    break 'c102;
                }
                dx = fx - (*fp).cur_pos.x;
                dy = fy - (*fp).cur_pos.y;
                dx = dx * dx;
                dy = dy * dy;
                dist = dx + dy;
                if !(ftCo_800A6700_inline0(fp, fx, fy) != 0) {
                    if ((best as f64) < 0.0f64) || (dist < best) {
                        (*arg1).x = fx;
                        best = dist;
                        (*arg1).y = fy;
                        (*arg1).z = 0.0f32;
                    }
                }
                line_id = (-1_i32);
                line = 0;
                px = ((a.x as f64) + 5.0f64) as f32;
                blocked = ext::mpCheckFloor(px, ((a.y as f64) + 10.0f64) as f32, px, ((a.y as f64) - 10.0f64) as f32, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, line as usize as *mut HSD_GObj);
                if blocked != 0 {
                    result = line_id;
                    if (((ext::grBigBlue_801EF844(result) != 0) || (ext::grInishie1_801FCAAC(result) != 0)) || (ext::grCorneria_801E2D90(result) != 0)) || (ext::grVenom_80206D10(result) != 0) {
                        line = 1;
                    }
                    if line != 0 {
                        blocked = 0;
                    }
                }
                if blocked == 0 {
                    break 'c102;
                }
                fx = floor_pos.x;
                my = floor_pos.y;
                fy = my;
                dir.x = fx - (*fp).cur_pos.x;
                dir.y = fy - (*fp).cur_pos.y;
                dir.z = 0.0f32;
                let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(dir) as *mut Vec);
                if ext::lbVector_Angle(arg2, core::ptr::addr_of_mut!(dir) as *mut Vec) > 0.7853982f32 {
                    break 'c102;
                }
                dx = fx - (*fp).cur_pos.x;
                dy = fy - (*fp).cur_pos.y;
                dx = dx * dx;
                dy = dy * dy;
                dist2 = dx + dy;
                if !(ftCo_800A6700_inline0(fp, fx, fy) != 0) {
                    if dist2 < best {
                        (*arg1).x = fx;
                        best = dist2;
                        (*arg1).y = fy;
                        (*arg1).z = 0.0f32;
                    }
                }
                line_id = (-1_i32);
                line = 0;
                px = ((b.x as f64) - 5.0f64) as f32;
                blocked = ext::mpCheckFloor(px, ((b.y as f64) + 10.0f64) as f32, px, ((b.y as f64) - 10.0f64) as f32, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, line as usize as *mut HSD_GObj);
                if blocked != 0 {
                    result = line_id;
                    if (((ext::grBigBlue_801EF844(result) != 0) || (ext::grInishie1_801FCAAC(result) != 0)) || (ext::grCorneria_801E2D90(result) != 0)) || (ext::grVenom_80206D10(result) != 0) {
                        line = 1;
                    }
                    if line != 0 {
                        blocked = 0;
                    }
                }
                if blocked == 0 {
                    break 'c102;
                }
                fx = floor_pos.x;
                floor_y = floor_pos.y;
                fy = floor_y;
                dir.x = fx - (*fp).cur_pos.x;
                dir.y = fy - (*fp).cur_pos.y;
                dir.z = 0.0f32;
                let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(dir) as *mut Vec);
                if ext::lbVector_Angle(arg2, core::ptr::addr_of_mut!(dir) as *mut Vec) > 0.7853982f32 {
                    break 'c102;
                }
                dx = fx - (*fp).cur_pos.x;
                dy = fy - (*fp).cur_pos.y;
                dx = dx * dx;
                dy = dy * dy;
                dist = dx + dy;
                if !(ftCo_800A6700_inline0(fp, fx, fy) != 0) {
                    if dist < best {
                        (*arg1).x = fx;
                        best = dist;
                        (*arg1).y = fy;
                        (*arg1).z = 0.0f32;
                    }
                }
            }
            island = (*island).next;
        }
    }
    if (best as f64) < 0.0f64 {
        return 0;
    }
    return 1;
}

/// `ftCo_800A75DC_CheckFloor`
unsafe fn ftCo_800A75DC_CheckFloor(mut x0: f32, mut above: f32, mut x1: f32, mut below: f32, mut floor_pos: *mut Vec, mut line_id: *mut i32, mut flags: *mut u32, mut floor_normal: *mut Vec) -> i32 {
    let mut line: i32 = 0;
    let mut blocked: i32 = 0;
    let mut result: i32 = 0;
    blocked = 0;
    *line_id = (-1_i32);
    result = ext::mpCheckFloor(x0, above, x1, below, 0.0f32, floor_pos, line_id, flags, floor_normal, (-1_i32), (-1_i32), (-1_i32), None, blocked as usize as *mut HSD_GObj);
    if result != 0 {
        line = *line_id;
        if (((ext::grBigBlue_801EF844(line) != 0) || (ext::grInishie1_801FCAAC(line) != 0)) || (ext::grCorneria_801E2D90(line) != 0)) || (ext::grVenom_80206D10(line) != 0) {
            blocked = 1;
        }
        if blocked != 0 {
            result = 0;
        }
    }
    return result;
}

/// `ftCo_800A75DC`
pub unsafe fn ftCo_800A75DC(mut fp0: *mut Fighter, mut fp1: *mut Fighter) {
    melee_core::ct!("ftCo_800A75DC");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp0).cpu) as *mut CpuFighter;
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut flags: u32 = 0;
    let mut __: [u8; 24] = [0; 24];
    let mut sp18: Vec3 = unsafe { core::mem::zeroed() };
    let mut fx: f32 = 0.0;
    let mut fy: f32 = 0.0;
    let mut d: f32 = 0.0;
    if fp1.is_null() {
        return;
    }
    if (*fp1).ground_or_air == ((GA_Air as i32) as u32) {
        let mut result: i32 = 0;
        let mut blocked: i32 = 0;
        fx = (*fp1).cur_pos.x;
        {
            let mut fx2: f32 = fx;
            let mut below: f32 = 0.0;
            let mut above: f32 = 0.0;
            fy = (*fp1).cur_pos.y;
            blocked = 0;
            line_id = (-1_i32);
            below = fy - 1e+03f32;
            above = 1e+01f32 + fy;
            result = ext::mpCheckFloor(fx2, above, fx, below, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, blocked as usize as *mut HSD_GObj);
        }
        if result != 0 {
            let mut line: i32 = line_id;
            if (((ext::grBigBlue_801EF844(line) != 0) || (ext::grInishie1_801FCAAC(line) != 0)) || (ext::grCorneria_801E2D90(line) != 0)) || (ext::grVenom_80206D10(line) != 0) {
                blocked = 1;
            }
            if blocked != 0 {
                result = 0;
            }
        }
        if result != 0 {
            let mut x60: *mut i32 = null_mut();
            let mut island: *mut mp_UnkStruct0 = null_mut();
            island = ext::mpIsland_8005AB54(line_id);
            if ftCo_800A2718(island) == 0 {
                x60 = core::ptr::addr_of_mut!((*fp0).cpu.x60) as *mut i32;
                ftCo_800A1F3C(fp0, floor_pos.x, floor_pos.y, (*data).x56C + (*fp1).cpu.x564);
                if !island.is_null() {
                    d = (*island).x14.x - (*data).x54.x;
                    if d < 0.0f32 {
                        d = -d;
                    }
                    if (d as f64) < 5.0f64 {
                        ftCo_800A75DC_set_target(fp0, x60 as *const i32, (((*island).x14.x as f64) - 5.0f64) as f32, (*island).x14.y, (*data).x56C + (*fp1).cpu.x564);
                    } else {
                        d = if ((*island).x8.x - (*data).x54.x) < (0 as f32) { -((*island).x8.x - (*data).x54.x) } else { (*island).x8.x - (*data).x54.x };
                        if (d as f64) < 5.0f64 {
                            ftCo_800A1F3C(fp0, (5.0f64 + ((*island).x8.x as f64)) as f32, (*island).x8.y, (*data).x56C + (*fp1).cpu.x564);
                        }
                    }
                }
            }
        } else {
            let _ = ftCo_800A4768(fp1, core::ptr::addr_of_mut!(sp18) as *mut Vec);
            ftCo_800A1F3C(fp0, sp18.x, sp18.y, (*data).x56C + (*fp1).cpu.x564);
        }
    } else if ftCo_800A2718(ext::mpIsland_8005AB54((*fp1).coll_data.floor.index)) == 0 {
        let mut x60: *mut i32 = null_mut();
        let mut island: *mut mp_UnkStruct0 = null_mut();
        let mut fp0_island: *mut mp_UnkStruct0 = null_mut();
        let mut same_island: i32 = 0;
        x60 = core::ptr::addr_of_mut!((*fp0).cpu.x60) as *mut i32;
        ftCo_800A1F3C(fp0, (*fp1).cur_pos.x, (*fp1).cur_pos.y, (*data).x56C + (*fp1).cpu.x564);
        if (*fp0).ground_or_air == ((GA_Air as i32) as u32) {
            same_island = 0;
        } else if (*fp1).ground_or_air == ((GA_Air as i32) as u32) {
            same_island = 0;
        } else {
            fp0_island = ext::mpIsland_8005AB54((*fp0).coll_data.floor.index);
            if fp0_island.is_null() {
                same_island = 0;
            } else if ext::mpIsland_8005AB54((*fp1).coll_data.floor.index) == fp0_island {
                same_island = 1;
            } else {
                same_island = 0;
            }
        }
        if ((same_island == 0) && (!{ island = ext::mpIsland_8005AB54((*fp1).coll_data.floor.index); island }.is_null())) && ((((*data).x54.y - (*fp0).cur_pos.y) as f64) > 0.0f64) {
            if (((*data).x54.x - (*fp0).cur_pos.x) as f64) > 0.0f64 {
                if (*fp0).cur_pos.x < (*island).x8.x {
                    ftCo_800A75DC_set_target(fp0, x60 as *const i32, (5.0f64 + ((*island).x8.x as f64)) as f32, (*island).x8.y, (*data).x56C + (*fp1).cpu.x564);
                }
            } else {
                if (*fp0).cur_pos.x > (*island).x14.x {
                    ftCo_800A1F3C(fp0, (((*island).x14.x as f64) - 5.0f64) as f32, (*island).x14.y, (*data).x56C + (*fp1).cpu.x564);
                }
            }
        }
    }
}

/// `ftCo_800A7AAC`
pub unsafe fn ftCo_800A7AAC(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A7AAC");
    let mut partner: *mut Fighter = null_mut();
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut flags: u32 = 0;
    let mut __: [u8; 24] = [0; 24];
    let mut partner_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut d: f32 = 0.0;
    partner = ftCo_800A589C(fp);
    if partner.is_null() {
        return;
    }
    partner_pos = (*partner).cur_pos;
    if (*partner).ground_or_air == ((GA_Air as i32) as u32) {
        let mut result: i32 = 0;
        let mut island: *mut mp_UnkStruct0 = null_mut();
        let mut below: f32 = partner_pos.y - 1e+03f32;
        let mut above: f32 = 1e+01f32 + partner_pos.y;
        result = ftCo_800A75DC_CheckFloor(partner_pos.x, above, partner_pos.x, below, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec);
        if result != 0 {
            island = ext::mpIsland_8005AB54(line_id);
            if ftCo_800A2718(island) == 0 {
                ftCo_800A1F3C(fp, floor_pos.x, floor_pos.y, (*data).x56C + (*partner).cpu.x564);
                if !island.is_null() {
                    d = (*island).x14.x - (*data).x54.x;
                    if d < 0.0f32 {
                        d = -d;
                    }
                    if (d as f64) < 5.0f64 {
                        ftCo_800A1F3C(fp, (((*island).x14.x as f64) - 5.0f64) as f32, (*island).x14.y, (*data).x56C + (*partner).cpu.x564);
                    } else {
                        d = if ((*island).x8.x - (*data).x54.x) < (0 as f32) { -((*island).x8.x - (*data).x54.x) } else { (*island).x8.x - (*data).x54.x };
                        if (d as f64) < 5.0f64 {
                            ftCo_800A1F3C(fp, (5.0f64 + ((*island).x8.x as f64)) as f32, (*island).x8.y, (*data).x56C + (*partner).cpu.x564);
                        }
                    }
                }
            }
        }
    } else if ftCo_800A2718(ext::mpIsland_8005AB54((*partner).coll_data.floor.index)) == 0 {
        let mut blocked: i32 = 0;
        let mut line: i32 = 0;
        let mut result: i32 = 0;
        let mut fp_island: *mut mp_UnkStruct0 = null_mut();
        let mut island: *mut mp_UnkStruct0 = null_mut();
        let mut same_island: i32 = 0;
        let mut sub: i32 = 0;
        let mut below: f32 = ((partner_pos.y as f64) - 2.0f64) as f32;
        let mut above: f32 = (2.0f64 + (partner_pos.y as f64)) as f32;
        blocked = 0;
        line_id = (-1_i32);
        result = ext::mpCheckFloor(partner_pos.x, above, partner_pos.x, below, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, blocked as usize as *mut HSD_GObj);
        if result != 0 {
            line = line_id;
            if (((ext::grBigBlue_801EF844(line) != 0) || (ext::grInishie1_801FCAAC(line) != 0)) || (ext::grCorneria_801E2D90(line) != 0)) || (ext::grVenom_80206D10(line) != 0) {
                blocked = 1;
            }
            if blocked != 0 {
                result = 0;
            }
        }
        if result == 0 {
            ftCo_800A1F3C(fp, (*partner).cur_pos.x, (*partner).cur_pos.y, (*data).x56C + (*partner).cpu.x564);
        } else {
            ftCo_800A1F3C(fp, partner_pos.x, partner_pos.y, (*data).x56C + (*partner).cpu.x564);
        }
        if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
            same_island = 0;
        } else if (*partner).ground_or_air == ((GA_Air as i32) as u32) {
            same_island = 0;
        } else {
            fp_island = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
            if fp_island.is_null() {
                same_island = 0;
            } else if ext::mpIsland_8005AB54((*partner).coll_data.floor.index) == fp_island {
                same_island = 1;
            } else {
                same_island = 0;
            }
        }
        if same_island == 0 {
            's103: {
                match (*fp).motion_id {
                    252 => {
                        sub = 1;
                    }
                    253 => {
                        sub = 2;
                    }
                    _ => {
                        sub = 0;
                    }
                }
            }
            's104: {
                match sub {
                    0 => {
                        island = ext::mpIsland_8005AB54((*partner).coll_data.floor.index);
                        if (!island.is_null()) && ((((*data).x54.y - (*fp).cur_pos.y) as f64) > 0.0f64) {
                            if (((*data).x54.x - (*fp).cur_pos.x) as f64) > 0.0f64 {
                                if (*fp).cur_pos.x < (*island).x8.x {
                                    ftCo_800A1F3C(fp, (5.0f64 + ((*island).x8.x as f64)) as f32, (*island).x8.y, (*data).x56C + (*partner).cpu.x564);
                                }
                            } else if (*fp).cur_pos.x > (*island).x14.x {
                                ftCo_800A1F3C(fp, (((*island).x14.x as f64) - 5.0f64) as f32, (*island).x14.y, (*data).x56C + (*partner).cpu.x564);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// `ftCo_800A80E4`
pub unsafe fn ftCo_800A80E4(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A80E4");
    let mut other_fp: *mut Fighter = (*fp).cpu.x44;
    if (!other_fp.is_null()) && ((*fp).ground_or_air != ((GA_Air as i32) as u32)) {
        let mut dx: f32 = (*fp).cur_pos.x - (*other_fp).cur_pos.x;
        let mut dy: f32 = (*fp).cur_pos.y - (*other_fp).cur_pos.y;
        let mut vec: Vec3 = unsafe { core::mem::zeroed() };
        if (!((melee_runtime::gekko::sqrtf(fmadds(dx, dx, dy * dy)) as f64) > 50.0f64)) && (ftCo_800A6700(fp, core::ptr::addr_of_mut!((*other_fp).cur_pos) as *mut Vec, core::ptr::addr_of_mut!(vec) as *mut Vec) != 0) {
            ftCo_800A1F3C(fp, vec.x, vec.y, 5.0f32);
        }
    }
}

/// `ftCo_800A8210_inline0`
unsafe fn ftCo_800A8210_inline0(mut fp: *mut Fighter, mut out: *mut Vec) {
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut y: f32 = (*out).y;
    let mut x: f32 = (*out).x;
    if (*data).x60 == 0 {
        (*data).x54.x = x;
        (*data).x54.y = y;
        (*data).x38 = 5.0f32;
        ftCo_800A1CC4(fp, ext::ftCo_803C6594[ext::stage_info.grkind as usize]);
    }
}

/// `ftCo_800A8210`
pub unsafe fn ftCo_800A8210(mut fp: *mut Fighter, mut arg1: *mut Vec) -> i32 {
    melee_core::ct!("ftCo_800A8210");
    let mut out: Vec3 = unsafe { core::mem::zeroed() };
    let mut dir: Vec3 = unsafe { core::mem::zeroed() };
    let mut left: f32 = 0.0;
    let mut sum_x: f32 = 0.0;
    let mut cx: f32 = 0.0;
    let mut bottom: f32 = 0.0;
    let mut sum_y: f32 = 0.0;
    let mut cy: f32 = 0.0;
    if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        return 1;
    }
    if ext::stage_info.grkind == ((Gr_Kind_Icemt as i32) as u32) {
        let _ = ext::grLib_801C9E60(core::ptr::addr_of_mut!(dir) as *mut Vec);
        if (dir.y as f64) < 0.0f64 {
            if ftCo_800A6A98(fp, core::ptr::addr_of_mut!(out) as *mut Vec) != 0 {
                ftCo_800A8210_inline0(fp, core::ptr::addr_of_mut!(out) as *mut Vec);
                return 1;
            }
        } else {
            if ftCo_800A6D2C(fp, core::ptr::addr_of_mut!(out) as *mut Vec) != 0 {
                ftCo_800A8210_inline0(fp, core::ptr::addr_of_mut!(out) as *mut Vec);
                return 1;
            }
        }
        return 0;
    } else if ext::stage_info.grkind == ((Gr_Kind_RCruise as i32) as u32) {
        left = ext::Stage_GetBlastZoneLeftOffset();
        sum_x = ext::Stage_GetBlastZoneRightOffset();
        cx = 0.5f32 * { sum_x = sum_x + left; sum_x };
        bottom = ext::Stage_GetBlastZoneBottomOffset();
        sum_y = ext::Stage_GetBlastZoneTopOffset();
        cy = 0.5f32 * { sum_y = sum_y + bottom; sum_y };
        dir.x = cx - (*fp).cur_pos.x;
        dir.y = cy - (*fp).cur_pos.y;
        dir.z = 0.0f32;
        if ftCo_800A6FC4(fp, core::ptr::addr_of_mut!(out) as *mut Vec, core::ptr::addr_of_mut!(dir) as *mut Vec) != 0 {
            ftCo_800A8210_inline0(fp, core::ptr::addr_of_mut!(out) as *mut Vec);
            return 1;
        }
        let _ = ext::grLib_801C9E60(core::ptr::addr_of_mut!(dir) as *mut Vec);
        dir.x = -dir.x;
        dir.y = -dir.y;
        if ftCo_800A6FC4(fp, core::ptr::addr_of_mut!(out) as *mut Vec, core::ptr::addr_of_mut!(dir) as *mut Vec) != 0 {
            ftCo_800A8210_inline0(fp, core::ptr::addr_of_mut!(out) as *mut Vec);
            return 1;
        }
        return 0;
    } else if ext::stage_info.grkind == ((Gr_Kind_BigBlue as i32) as u32) {
        left = ext::Stage_GetBlastZoneLeftOffset();
        sum_x = ext::Stage_GetBlastZoneRightOffset();
        cx = 0.5f32 * { sum_x = sum_x + left; sum_x };
        bottom = ext::Stage_GetBlastZoneBottomOffset();
        sum_y = ext::Stage_GetBlastZoneTopOffset();
        cy = 0.5f32 * { sum_y = sum_y + bottom; sum_y };
        dir.x = cx - (*fp).cur_pos.x;
        dir.y = cy - (*fp).cur_pos.y;
        dir.z = 0.0f32;
        if ftCo_800A6FC4(fp, core::ptr::addr_of_mut!(out) as *mut Vec, core::ptr::addr_of_mut!(dir) as *mut Vec) != 0 {
            ftCo_800A8210_inline0(fp, core::ptr::addr_of_mut!(out) as *mut Vec);
            return 1;
        }
        let _ = ext::grLib_801C9E60(core::ptr::addr_of_mut!(dir) as *mut Vec);
        dir.x = -dir.x;
        dir.y = -dir.y;
        if ftCo_800A6FC4(fp, core::ptr::addr_of_mut!(out) as *mut Vec, core::ptr::addr_of_mut!(dir) as *mut Vec) != 0 {
            ftCo_800A8210_inline0(fp, core::ptr::addr_of_mut!(out) as *mut Vec);
            return 1;
        }
        return 0;
    } else {
        if ftCo_800A6700(fp, arg1, core::ptr::addr_of_mut!(out) as *mut Vec) != 0 {
            ftCo_800A8210_inline0(fp, core::ptr::addr_of_mut!(out) as *mut Vec);
            return 1;
        }
        return 0;
    }
}

/// `ftCo_800A866C`
pub unsafe fn ftCo_800A866C(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A866C");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut item: *mut Item = (*fp).cpu.x4C;
    let mut island: *mut mp_UnkStruct0 = null_mut();
    let mut fp_island: *mut mp_UnkStruct0 = null_mut();
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut flags: u32 = 0;
    let mut found: i32 = 0;
    let mut blocked: i32 = 0;
    let mut same_island: i32 = 0;
    let mut ix: f32 = 0.0;
    let mut iy: f32 = 0.0;
    if ((!item.is_null()) && ((*fp).ground_or_air != ((GA_Air as i32) as u32))) && ((*item).ground_or_air != ((GA_Air as i32) as u32)) {
        island = ext::mpIsland_8005AB54((*item).x378_itemColl.floor.index);
        if !(ftCo_800A2718_dontinline(island) != 0) {
            ix = (*item).pos.x;
            iy = (*item).pos.y;
            blocked = 0;
            line_id = (-1_i32);
            {
                let mut bottom: f32 = iy - 5e+01f32;
                let mut top: f32 = 1.0f32 + iy;
                let mut ix2: f32 = ix;
                found = ext::mpCheckFloor(ix2, top, ix, bottom, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, blocked as usize as *mut HSD_GObj);
            }
            if found != 0 {
                let mut line: i32 = line_id;
                if (((ext::grBigBlue_801EF844(line) != 0) || (ext::grInishie1_801FCAAC(line) != 0)) || (ext::grCorneria_801E2D90(line) != 0)) || (ext::grVenom_80206D10(line) != 0) {
                    blocked = 1;
                }
                if blocked != 0 {
                    found = 0;
                }
            }
            if found != 0 {
                let mut x: f32 = 0.0;
                let mut y: f32 = floor_pos.y;
                x = floor_pos.x;
                ftCo_800A1F3C(fp, x, y, 1.0f32);
            }
            if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
                same_island = 0;
            } else if (*item).ground_or_air == ((GA_Air as i32) as u32) {
                same_island = 0;
            } else {
                fp_island = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
                if fp_island.is_null() {
                    same_island = 0;
                } else if ext::mpIsland_8005AB54((*item).x378_itemColl.floor.index) == fp_island {
                    same_island = 1;
                } else {
                    same_island = 0;
                }
            }
            if ((same_island == 0) && (!island.is_null())) && ((((*data).x54.y - (*fp).cur_pos.y) as f64) > 0.0f64) {
                if (((*data).x54.x - (*fp).cur_pos.x) as f64) > 0.0f64 {
                    let mut x: f32 = (*island).x8.x;
                    if (*fp).cur_pos.x < x {
                        let mut y: f32 = (*island).x8.y;
                        x = (5.0f64 + (x as f64)) as f32;
                        ftCo_800A1F3C(fp, x, y, 1.0f32);
                    }
                } else {
                    let mut x: f32 = (*island).x14.x;
                    if (*fp).cur_pos.x > x {
                        let mut y: f32 = (*island).x14.y;
                        x = ((x as f64) - 5.0f64) as f32;
                        ftCo_800A1F3C(fp, x, y, 1.0f32);
                    }
                }
            }
        }
    }
}

/// `ftCo_800A8940_inline0`
unsafe fn ftCo_800A8940_inline0(mut result_in: i32, mut blocked_in: i32, mut line_id_ptr: *const i32) -> i32 {
    let mut line_id: i32 = 0;
    let mut blocked: i32 = blocked_in;
    let mut result: i32 = result_in;
    if result != 0 {
        line_id = *line_id_ptr;
        if (((ext::grBigBlue_801EF844(line_id) != 0) || (ext::grInishie1_801FCAAC(line_id) != 0)) || (ext::grCorneria_801E2D90(line_id) != 0)) || (ext::grVenom_80206D10(line_id) != 0) {
            blocked = 1;
        }
        if blocked != 0 {
            result = 0;
        }
    }
    return result;
}

/// `ftCo_800A8940`
pub unsafe fn ftCo_800A8940(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A8940");
    let mut island: *mut mp_UnkStruct0 = null_mut();
    let mut cur_island: *mut mp_UnkStruct0 = null_mut();
    let mut chosen: *mut mp_UnkStruct0 = null_mut();
    let mut total: f32 = 0.0;
    let mut accum: f32 = 0.0;
    let mut rnd: f32 = 0.0;
    let mut rnd2: f32 = 0.0;
    let mut __: [u8; 4] = [0; 4];
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut _2: [u8; 12] = [0; 12];
    let mut line_id: i32 = 0;
    let mut flags: u32 = 0;
    let mut px: f32 = 0.0;
    let mut py: f32 = 0.0;
    let mut width: f32 = 0.0;
    let mut result: i32 = 0;
    let mut blocked: i32 = 0;
    rnd = ext::HSD_Randf();
    cur_island = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
    total = 0.0f32;
    {
        island = ext::mpIsland_80458E88.next;
        'l105: while !island.is_null() {
            'c105: {
                if !(ext::ScriptGame_StageJointActive((*island).x28 as i32) != 0) {
                    break 'c105;
                }
                if (cur_island != island) && (ftCo_800A2718_dontinline3(island) == 0) {
                    if !(ftCo_800A6700_inline0(fp, (*island).x14.x, (*island).x14.y) != 0) {
                        if !(ftCo_800A6700_inline0(fp, (*island).x8.x, (*island).x8.y) != 0) {
                            total = total + ((*island).x14.x - (*island).x8.x);
                        }
                    }
                }
            }
            island = (*island).next;
        }
    }
    if (total < 1e-05f32) && (total > (-1e-05f32)) {
        result = 1;
    } else {
        result = 0;
    }
    if result != 0 {
        return;
    }
    accum = 0.0f32;
    total = (1.0f64 / (total as f64)) as f32;
    {
        island = ext::mpIsland_80458E88.next;
        'l106: while !island.is_null() {
            'c106: {
                if !(ext::ScriptGame_StageJointActive((*island).x28 as i32) != 0) {
                    break 'c106;
                }
                if cur_island != island {
                    if !(ftCo_800A6700_inline0(fp, (*island).x14.x, (*island).x14.y) != 0) {
                        if !(ftCo_800A6700_inline0(fp, (*island).x8.x, (*island).x8.y) != 0) {
                            accum = accum + ((*island).x14.x - (*island).x8.x);
                            if rnd < (accum * total) {
                                chosen = island;
                                break 'l106;
                            }
                        }
                    }
                }
            }
            island = (*island).next;
        }
    }
    rnd2 = ext::HSD_Randf();
    width = (*island).x14.x - (*island).x8.x;
    if (width as f64) > 10.0f64 {
        px = ((width as f64) - 10.0f64).mul_add(rnd2 as f64, 5.0f64 + ((*chosen).x8.x as f64)) as f32;
    } else {
        px = fmadds(width, rnd2, (*chosen).x8.x);
    }
    blocked = 0;
    py = fmadds(rnd2, (*island).x14.y - (*island).x8.y, (*chosen).x8.y);
    line_id = (-1_i32);
    {
        let mut bottom: f32 = py - 1e+02f32;
        let mut top: f32 = 5.0f32 + py;
        result = ext::mpCheckFloor(px, top, px, bottom, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, null_mut());
    }
    result = ftCo_800A8940_inline0(result, blocked, (core::ptr::addr_of_mut!(line_id) as *mut i32) as *const i32);
    if result != 0 {
        if !(ftCo_800A6700_inline0(fp, floor_pos.x, floor_pos.y) != 0) {
            let mut x: f32 = 0.0;
            let mut y: f32 = floor_pos.y;
            x = floor_pos.x;
            ftCo_800A1F3C(fp, x, y, 5.0f32);
        }
    }
}

/// `ftCo_800A8DE4_noinline`
pub unsafe fn ftCo_800A8DE4_noinline(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A8DE4_noinline");
    ftCo_800A8DE4(fp);
}

/// `ftCo_800A8DE4`
pub unsafe fn ftCo_800A8DE4(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A8DE4");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if !((*fp).cpu.xFA_b2() != 0) {
        (*data).set_xFA_b2((1 as u8) as u8);
        (*data).x5C = 1e+04f32;
        if ((*fp).facing_dir as f64) > 0.0f64 {
            if (!(ftCo_800A4038(fp, 1) != 0)) && (!(ftCo_800A3908(fp, 1) != 0)) {
                let _ = ftCo_800A4038(fp, 0);
            }
        } else if (!(ftCo_800A3908(fp, 1) != 0)) && (!(ftCo_800A4038(fp, 1) != 0)) {
            let _ = ftCo_800A3908(fp, 0);
        }
    }
}

/// `angle_x_units`
unsafe fn angle_x_units(mut angle: f32) -> i32 {
    return melee_core::ppc::fctiwz_f32(127.0f32 * melee_runtime::gekko::cosf(angle));
}

/// `angle_y_units`
unsafe fn angle_y_units(mut angle: f32) -> i32 {
    return melee_core::ppc::fctiwz_f32(127.0f32 * melee_runtime::gekko::sinf(angle));
}

/// `ftCo_CpuAimPkThunder`
unsafe fn ftCo_CpuAimPkThunder(mut fp: *mut Fighter, mut angle: f32, mut frames: i32) {
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, angle_x_units(angle) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, angle_y_units(angle) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, frames as u8);
}

/// `ftCo_CpuStartUpSpecial`
unsafe fn ftCo_CpuStartUpSpecial(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 127 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_PressBFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseB as i32) as u8);
}

/// `ftCo_800A8EB0`
pub unsafe fn ftCo_800A8EB0(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A8EB0");
    let mut angle: f32 = 0.0;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*fp).cpu.level < 9 {
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
        return;
    }
    ftCo_CpuStartUpSpecial(fp);
    if (((*data).x54.x - (*fp).cur_pos.x) as f64) > 0.0f64 {
        angle = 3.141592653589793f64 as f32;
        ftCo_CpuAimPkThunder(fp, angle, 20);
        angle = (-(3.141592653589793f64 / (2 as f64))) as f32;
        ftCo_CpuAimPkThunder(fp, angle, 35);
        angle = 0.0f32;
        ftCo_CpuAimPkThunder(fp, angle, 20);
        angle = (3.141592653589793f64 / (2 as f64)) as f32;
        ftCo_CpuAimPkThunder(fp, angle, 20);
    } else {
        angle = 0.0f32;
        ftCo_CpuAimPkThunder(fp, angle, 20);
        angle = (-(3.141592653589793f64 / (2 as f64))) as f32;
        ftCo_CpuAimPkThunder(fp, angle, 35);
        angle = 3.141592653589793f64 as f32;
        ftCo_CpuAimPkThunder(fp, angle, 20);
        angle = (3.141592653589793f64 / (2 as f64)) as f32;
        ftCo_CpuAimPkThunder(fp, angle, 20);
    }
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_800A92CC`
pub unsafe fn ftCo_800A92CC(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A92CC");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ((if ((*fp).cur_pos.x - (*data).x54.x) < (0 as f32) { -((*fp).cur_pos.x - (*data).x54.x) } else { (*fp).cur_pos.x - (*data).x54.x }) as f64) > 60.0f64 {
        ftCo_CpuStartUpSpecial(fp);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 88 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 88 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 15 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 15 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    } else {
        ftCo_CpuStartUpSpecial(fp);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 15 as u8);
        if (ext::HSD_Randf() as f64) > 0.5f64 {
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 88 as u8);
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 88 as u8);
        } else {
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
        }
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 15 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800A949C`
unsafe fn ftCo_800A949C(mut fp: *mut Fighter, mut unused: i32) {
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ((if ((*data).x54.x - (*fp).cur_pos.x) < (0 as f32) { -((*data).x54.x - (*fp).cur_pos.x) } else { (*data).x54.x - (*fp).cur_pos.x }) as f64) > 30.0f64 {
        ftCo_CpuSetNeutralStick(fp);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 127 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_PressBFor as i32) as u8, 40 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseB as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    } else {
        ftCo_CpuSetNeutralStick(fp);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 127 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_PressBFor as i32) as u8, 40 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseB as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    }
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_CpuRecoverDiagonally`
unsafe fn ftCo_CpuRecoverDiagonally(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 88 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 88 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_PressBFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseB as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_800A963C`
unsafe fn ftCo_800A963C(mut fp: *mut Fighter, mut unused: i32) {
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 88 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 88 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_PressBFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseB as i32) as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_800A96B8`
pub unsafe fn ftCo_800A96B8(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A96B8");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut sum: f32 = (*fp).coll_data.cur_pos.y + (*fp).coll_data.ecb.bottom.y;
    let mut tmp: i32 = if (*fp).cpu.x54.y > sum { 1 } else { 0 };
    's107: {
        match ftAi_Kind((*fp).kind as i32) {
            0 => {
                ftCo_CpuRecoverDiagonally(fp);
            }
            12 | 23 => {
                ftCo_800A92CC(fp);
            }
            1 | 22 => {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 88 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 88 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_PressBFor as i32) as u8, 1 as u8);
                super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseB as i32) as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 40 as u8);
                super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
            }
            14 => {
                super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
            }
            8 => {
                if ((*fp).cur_pos.y as f64) > (((*data).x54.y as f64) - 10.0f64) {
                    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
                } else {
                    ftCo_800A8EB0(fp);
                }
            }
            17 => {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_PressBFor as i32) as u8, 1 as u8);
                super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseB as i32) as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 40 as u8);
                super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
            }
            19 => {
                ftCo_800A949C(fp, tmp);
            }
            13 => {
                ftCo_800A963C(fp, tmp);
            }
            _ => {
                ftCo_CpuRecoverDiagonally(fp);
            }
        }
    }
}

/// `ftCo_IsNearlyZero`
unsafe fn ftCo_IsNearlyZero(mut x: f32) -> i32 {
    if (x < 1e-05f32) && (x > (-1e-05f32)) {
        return 1;
    }
    return 0;
}

/// `ftCo_800A9904`
pub unsafe fn ftCo_800A9904(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A9904");
    let mut _top: [u8; 8] = [0; 8];
    #[derive(Clone, Copy)] #[repr(C)] struct Local2 { pad: u32, flags: u32, line_id: i32, normal: Vec3, pos: Vec3 }
    let mut ceiling: Local2 = unsafe { core::mem::zeroed() };
    let mut sqrt_time_store: f32 = 0.0;
    let mut sqrt_terminal_store: f32 = 0.0;
    let mut __: [u8; 28] = [0; 28];
    let mut x_time: f32 = 0.0;
    let mut gravity: f32 = 0.0;
    let mut predicted_y: f32 = 0.0;
    let mut terminal_time: f32 = 0.0;
    let mut sqrt_time: f32 = 0.0;
    let mut sqrt_terminal: f32 = 0.0;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ftCo_800A3498(fp) != 0 {
        if ext::ScriptGame_CapsMaxJumps(fp) > ((*fp).x1968_jumpsUsed as i32) {
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickTowardDestination as i32) as u8, 127 as u8);
            super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
            super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
            super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
        } else {
            ftCo_800A96B8(fp);
        }
    } else if (*data).xFA_b5() != 0 {
        let mut grav_p: *mut f32 = null_mut();
        let mut dx: f32 = (*data).x54.x - (*fp).cur_pos.x;
        if ftCo_IsNearlyZero((*fp).pos_delta.x) != 0 {
            x_time = 1e+03f32;
        } else {
            x_time = dx / (*fp).pos_delta.x;
        }
        gravity = *{ grav_p = core::ptr::addr_of_mut!((*fp).co_attrs.gravity) as *mut f32; grav_p };
        if ftCo_IsNearlyZero(gravity) != 0 {
            terminal_time = 1e+03f32;
        } else {
            terminal_time = (-((-(*fp).co_attrs.terminal_velocity) - (*fp).pos_delta.y)) / gravity;
        }
        if terminal_time <= 0.0f32 {
            predicted_y = fmadds((*fp).pos_delta.y, x_time, (*fp).cur_pos.y);
        } else if x_time < terminal_time {
            {
                if x_time > 0.0f32 {
                    let mut guess: f64 = melee_runtime::gekko::frsqrte(x_time as f64);
                    guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(x_time as f64, 3.0f64);
                    guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(x_time as f64, 3.0f64);
                    guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(x_time as f64, 3.0f64);
                    sqrt_time_store = ((x_time as f64) * guess) as f32;
                    sqrt_time = *(core::ptr::addr_of_mut!(sqrt_time_store) as *mut f32);
                } else {
                    sqrt_time = x_time;
                }
            }
            predicted_y = (((*fp).cur_pos.y as f64) + (-0.5f64).mul_add(((*grav_p) * sqrt_time) as f64, ((*fp).pos_delta.y * x_time) as f64)) as f32;
        } else {
            {
                if terminal_time > 0.0f32 {
                    let mut guess: f64 = melee_runtime::gekko::frsqrte(terminal_time as f64);
                    guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(terminal_time as f64, 3.0f64);
                    guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(terminal_time as f64, 3.0f64);
                    guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(terminal_time as f64, 3.0f64);
                    sqrt_terminal_store = ((terminal_time as f64) * guess) as f32;
                    sqrt_terminal = *(core::ptr::addr_of_mut!(sqrt_terminal_store) as *mut f32);
                } else {
                    sqrt_terminal = terminal_time;
                }
            }
            predicted_y = (((*fp).cur_pos.y as f64) + ((-0.5f64).mul_add(((*grav_p) * sqrt_terminal) as f64, ((*fp).pos_delta.y * terminal_time) as f64) - (((x_time - terminal_time) * ftCo_GetTerminalVelocity(fp)) as f64))) as f32;
        }
        {
            let mut stick: i32 = melee_core::ppc::fctiwz_f32(fmadds(4.7000003f32, ((*data).level + 1) as f32, 8e+01f32));
            if ((x_time as f64) < 0.0f64) || (predicted_y < (*data).x54.y) {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, stick as u8);
            } else {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
            }
        }
        ftCo_CpuFinishWithNeutralY(fp);
    } else if ext::mpCheckCeiling((*fp).cur_pos.x, (*fp).cur_pos.y, (*data).x54.x, (*data).x54.y, core::ptr::addr_of_mut!(ceiling.pos) as *mut Vec, core::ptr::addr_of_mut!(ceiling.line_id) as *mut i32, core::ptr::addr_of_mut!(ceiling.flags) as *mut u32, core::ptr::addr_of_mut!(ceiling.normal) as *mut Vec, (-1_i32), (-1_i32)) != 0 {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 129 as u8);
        ftCo_CpuFinishWithNeutralY(fp);
    } else {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
        ftCo_CpuFinishWithNeutralY(fp);
    }
}

/// `ftCo_800A9CB4_inline0`
unsafe fn ftCo_800A9CB4_inline0(mut fp: *mut Fighter) -> i32 {
    if ext::stage_info.grkind == ((Gr_Kind_Shrine as i32) as u32) {
        if ((*fp).cur_pos.x > (-1.5e+02f32)) && ((*fp).cur_pos.x < (-9e+01f32)) {
            return 1;
        }
    } else if ext::stage_info.grkind == ((Gr_Kind_Fourside as i32) as u32) {
        return 2;
    }
    return 0;
}

/// `ftCo_800A9CB4_inline1`
unsafe fn ftCo_800A9CB4_inline1(mut fp: *mut Fighter, mut stage_pos: *mut Vec) -> i32 {
    if (*fp).cpu.xFA_b6() != 0 {
        return 0;
    }
    if (*fp).x34_scale.y < 1.0f32 {
        return 1;
    }
    if ext::stage_info.grkind == ((Gr_Kind_Icemt as i32) as u32) {
        let _ = ext::grLib_801C9E60(stage_pos);
        if ((*stage_pos).y as f64) < 0.0f64 {
            return 1;
        }
    } else if ext::stage_info.grkind == ((Gr_Kind_RCruise as i32) as u32) {
        return 1;
    }
    return 0;
}

/// `ftCo_CpuUseAvailableJump`
unsafe fn ftCo_CpuUseAvailableJump(mut fp: *mut Fighter, mut data: *mut CpuFighter) {
    if (*data).xFA_b6() != 0 {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXForward as i32) as u8, 127 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    } else if ((ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Yoshi as i32)) && (((*fp).x34_scale.y as f64) < 1.0f64)) && ((if ((*data).x54.x - (*fp).cur_pos.x) < (0 as f32) { -((*data).x54.x - (*fp).cur_pos.x) } else { (*data).x54.x - (*fp).cur_pos.x }) < 2e+01f32) {
        ftCo_800A0508(fp);
    } else {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickTowardDestination as i32) as u8, 127 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800A9CB4`
pub unsafe fn ftCo_800A9CB4(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800A9CB4");
    let mut __: [u8; 56] = [0; 56];
    let mut stage_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut unused_time_tmp: f32 = 0.0;
    let mut sqrt_x_time_store: f32 = 0.0;
    let mut sqrt_terminal_time_store: f32 = 0.0;
    let mut sp38_b: f32 = 0.0;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut grav_p: *mut f32 = null_mut();
    let mut temp_f1: f32 = 0.0;
    let mut gravity: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut x_time: f32 = 0.0;
    let mut x_delta_abs: f32 = 0.0;
    let mut y_pos: f32 = 0.0;
    let mut sqrt_x_time: f32 = 0.0;
    let mut sqrt_term_time: f32 = 0.0;
    let mut terminal_time: f32 = 0.0;
    let mut var_r0_2: i32 = 0;
    let mut stick_clamp: i32 = 0;
    if ftCo_800A1CA8(fp) != 0 {
        ftCo_CpuFinishWithNeutralStick(fp);
        return;
    }
    if ftCo_800A3234(fp) != 0 {
        if ext::ScriptGame_CapsMaxJumps(fp) > ((*fp).x1968_jumpsUsed as i32) {
            ftCo_CpuUseAvailableJump(fp, data);
            return;
        }
        if { var_r0_2 = ftCo_800A9CB4_inline0(fp); var_r0_2 } != 0 {
            if var_r0_2 == 1 {
                ftCo_CpuSetNeutralStick(fp);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 127 as u8);
                ftCo_CpuTapR(fp);
                ftCo_CpuFinishWithNeutralY(fp);
            } else {
                ftCo_CpuSetNeutralStick(fp);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXForward as i32) as u8, 80 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 127 as u8);
                ftCo_CpuTapR(fp);
                ftCo_CpuFinishWithNeutralY(fp);
            }
            return;
        }
        if ftCo_800A9CB4_inline1(fp, core::ptr::addr_of_mut!(stage_pos) as *mut Vec) != 0 {
            ftCo_800A96B8(fp);
            return;
        }
    }
    if (*data).xFA_b6() != 0 {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 129 as u8);
        ftCo_CpuFinishWithNeutralY(fp);
        return;
    }
    dx = (*data).x54.x - (*fp).cur_pos.x;
    x_delta_abs = if dx < (0 as f32) { -dx } else { dx };
    if !(ftCo_IsNearlyZero((*fp).pos_delta.x) != 0) {
        x_time = dx / (*fp).pos_delta.x;
    } else if (x_delta_abs as f64) > 0.0f64 {
        x_time = -1.0f32;
    } else {
        x_time = 0.0f32;
    }
    gravity = *{ grav_p = core::ptr::addr_of_mut!((*fp).co_attrs.gravity) as *mut f32; grav_p };
    if ftCo_IsNearlyZero(gravity) != 0 {
        terminal_time = 1e+03f32;
    } else {
        terminal_time = (-((-(*fp).co_attrs.terminal_velocity) - (*fp).pos_delta.y)) / gravity;
    }
    if terminal_time <= 0.0f32 {
        y_pos = fmadds((*fp).pos_delta.y, x_time, (*fp).cur_pos.y);
    } else if x_time < terminal_time {
        {
            if x_time > 0.0f32 {
                let mut guess: f64 = melee_runtime::gekko::frsqrte(x_time as f64);
                guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(x_time as f64, 3.0f64);
                guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(x_time as f64, 3.0f64);
                guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(x_time as f64, 3.0f64);
                sqrt_x_time_store = ((x_time as f64) * guess) as f32;
                sqrt_x_time = sqrt_x_time_store;
            } else {
                sqrt_x_time = x_time;
            }
        }
        y_pos = (((*fp).cur_pos.y as f64) + (-0.5f64).mul_add(((*grav_p) * sqrt_x_time) as f64, ((*fp).pos_delta.y * x_time) as f64)) as f32;
    } else {
        {
            if terminal_time > 0.0f32 {
                let mut guess: f64 = melee_runtime::gekko::frsqrte(terminal_time as f64);
                guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(terminal_time as f64, 3.0f64);
                guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(terminal_time as f64, 3.0f64);
                guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(terminal_time as f64, 3.0f64);
                sqrt_terminal_time_store = ((terminal_time as f64) * guess) as f32;
                sqrt_term_time = sqrt_terminal_time_store;
            } else {
                sqrt_term_time = terminal_time;
            }
        }
        sqrt_x_time = *grav_p;
        x_delta_abs = x_time - terminal_time;
        y_pos = (((*fp).cur_pos.y as f64) + ((-0.5f64).mul_add((sqrt_x_time * sqrt_term_time) as f64, ((*fp).pos_delta.y * terminal_time) as f64) - ((x_delta_abs * (*fp).co_attrs.terminal_velocity) as f64))) as f32;
    }
    if (x_time as f64) < 0.0f64 {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
    } else {
        if y_pos < (*data).x54.y {
            stick_clamp = (melee_core::ppc::fctiwz_f32((*data).x54.y - y_pos)) * 2;
            if stick_clamp > 32 {
                stick_clamp = 32;
            }
            super::ftcmdscript::ftCo_800B4778(fp, (CpuCmd_LstickXTowardDestinationClamped as i32) as u8, stick_clamp as u8, 127 as u8);
        } else {
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
        }
    }
    ftCo_CpuFinishWithNeutralY(fp);
}

/// `ftCo_800AA320`
pub unsafe fn ftCo_800AA320(mut fp: *mut Fighter, mut stick: *mut i32, mut clamp: *mut i32) {
    melee_core::ct!("ftCo_800AA320");
    if ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Nana as i32) {
        *stick = 64;
        *clamp = 127;
        return;
    }
    's108: {
        match (*fp).cpu.level {
            0 => {
                *stick = 1;
                *clamp = 67;
            }
            1 => {
                *stick = 2;
                *clamp = 72;
            }
            2 => {
                *stick = 2;
                *clamp = 77;
            }
            3 => {
                *stick = 2;
                *clamp = 82;
            }
            4 => {
                *stick = 2;
                *clamp = 87;
            }
            5 => {
                *stick = 2;
                *clamp = 92;
            }
            6 => {
                *stick = 4;
                *clamp = 97;
            }
            7 => {
                *stick = 4;
                *clamp = 107;
            }
            8 => {
                *stick = 8;
                *clamp = 117;
            }
            9 => {
                *stick = 8;
                *clamp = 127;
            }
            _ => {}
        }
    }
}

/// `isInTeeter`
unsafe fn isInTeeter(mut fp: *mut Fighter) -> i32 {
    if ((*fp).motion_id == ftCo_MS_Ottotto) || ((*fp).motion_id == ftCo_MS_OttottoWait) {
        return 1;
    }
    return 0;
}

/// `ftCo_CpuTurnAwayFromLedge`
unsafe fn ftCo_CpuTurnAwayFromLedge(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXForward as i32) as u8, 127 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_CpuTurnAround`
unsafe fn ftCo_CpuTurnAround(mut fp: *mut Fighter) {
    ftCo_CpuSetNeutralStick(fp);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXForward as i32) as u8, 176 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 10 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_800AA42C_inline0`
unsafe fn ftCo_800AA42C_inline0(mut clamp: f32, mut dist: f32, mut near: f32, mut far: f32) -> i32 {
    let mut raw: f32 = 0.0;
    if dist > far {
        return melee_core::ppc::fctiwz_f32(clamp);
    }
    if dist < near {
        return 0;
    }
    raw = 127.0f32 * (*ext::p_ftCommonData).horizontal_stick_deadzone;
    if raw > clamp {
        raw = clamp;
    }
    if raw < 8e+01f32 {
        raw = 8e+01f32;
    }
    {
        let mut scale: f32 = (dist - near) / (far - near);
        let mut tmp: f32 = fmadds(scale, clamp - raw, raw);
        let mut new_clamp: i32 = melee_core::ppc::fctiwz_f32(tmp);
        if (new_clamp as f32) > clamp {
            new_clamp = melee_core::ppc::fctiwz_f32(clamp);
        }
        return new_clamp;
    }
}

/// `ftCo_800AA42C`
pub unsafe fn ftCo_800AA42C(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AA42C");
    let mut data: *mut CpuFighter = null_mut();
    let mut near: f32 = 0.0;
    let mut dist: f32 = 0.0;
    let mut facing_dest: i32 = 0;
    let mut running_: i32 = 0;
    let mut stick: i32 = 0;
    let mut clamp: i32 = 0;
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    ftCo_800AA320(fp, core::ptr::addr_of_mut!(stick) as *mut i32, core::ptr::addr_of_mut!(clamp) as *mut i32);
    if isInTeeter(fp) != 0 {
        ftCo_CpuTurnAwayFromLedge(fp);
        return;
    }
    if (((*fp).facing_dir * ((*fp).cpu.x54.x - (*fp).cur_pos.x)) as f64) >= 0.0f64 {
        facing_dest = 1;
    } else {
        facing_dest = 0;
    }
    if !(facing_dest != 0) {
        ftCo_CpuTurnAround(fp);
        return;
    }
    {
        let mut dx: f32 = (*data).x54.x - (*fp).cur_pos.x;
        let mut dy: f32 = (*data).x54.y - (*fp).cur_pos.y;
        dist = melee_runtime::gekko::sqrtf(fmadds(dx, dx, dy * dy));
    }
    near = (*data).x38;
    if (dist - near) > (*data).x3C {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
        if (*data).xF8_b0() != 0 {
            if (((*fp).motion_id == ftCo_MS_Dash) || ((*fp).motion_id == ftCo_MS_Run)) || ((*fp).motion_id == ftCo_MS_TurnRun) {
                running_ = 1;
            } else {
                running_ = 0;
            }
            if running_ != 0 {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
            } else {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 0 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 2 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
            }
        } else {
            super::ftcmdscript::ftCo_800B4778(fp, (CpuCmd_LstickXTowardDestinationClamped as i32) as u8, stick as u8, clamp as u8);
        }
    } else if dist > near {
        let mut tmp: i32 = 0;
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
        tmp = ftCo_800AA42C_inline0(clamp as f32, dist, (*data).x38, (*data).x3C);
        super::ftcmdscript::ftCo_800B4778(fp, (CpuCmd_LstickXTowardDestinationClamped as i32) as u8, stick as u8, tmp as u8);
    } else {
        ftCo_CpuSetNeutralStick(fp);
    }
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_800AA844`
pub unsafe fn ftCo_800AA844(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AA844");
    let mut data: *mut CpuFighter = null_mut();
    let mut sp38: i32 = 0;
    let mut sp34: i32 = 0;
    let mut sp28: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp1C: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp18: i32 = 0;
    let mut sp14: u32 = 0;
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if isInTeeter(fp) != 0 {
        ftCo_CpuTurnAwayFromLedge(fp);
        return;
    }
    ftCo_800AA320(fp, core::ptr::addr_of_mut!(sp38) as *mut i32, core::ptr::addr_of_mut!(sp34) as *mut i32);
    if ftCo_800A28D0(fp, 1.0f32) != 0 {
        if (((ftCo_800A1CA8(fp) != 0) && ((*fp).coll_data.floor.index != (-1_i32))) && ((ext::mpLineGetFlags((*fp).coll_data.floor.index) & ((1 << 8) as u32)) != 0)) && (ftCo_800A0FB0(core::ptr::addr_of_mut!(sp28) as *mut Vec, core::ptr::addr_of_mut!(sp18) as *mut i32, core::ptr::addr_of_mut!(sp14) as *mut u32, core::ptr::addr_of_mut!(sp1C) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), (*fp).cur_pos.x, (*fp).cur_pos.y - 1e+01f32, (*fp).cur_pos.x, (*fp).cur_pos.y - 1e+03f32, 0.0f32) != 0) {
            ftCo_800A08F0(fp);
        } else if (*data).x54.y > (*fp).cur_pos.y {
            ftCo_800A0148(fp);
        } else if ((if ((*data).x54.x - (*fp).cur_pos.x) < (0 as f32) { -((*data).x54.x - (*fp).cur_pos.x) } else { (*data).x54.x - (*fp).cur_pos.x }) as f64) > 40.0f64 {
            ftCo_800A0148(fp);
        } else if inlineC0(fp) != 0 {
            ftCo_800A0148(fp);
        } else {
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
            super::ftcmdscript::ftCo_800B4778(fp, (CpuCmd_LstickForwardClamped as i32) as u8, sp38 as u8, sp34 as u8);
            super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
        }
        return;
    } else if inlineC0(fp) != 0 {
        ftCo_800A0148(fp);
    } else {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B4778(fp, (CpuCmd_LstickForwardClamped as i32) as u8, sp38 as u8, sp34 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800AABC8`
pub unsafe fn ftCo_800AABC8(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AABC8");
    if isInTeeter(fp) != 0 {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXForward as i32) as u8, 127 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    } else {
        let mut stick: i32 = 0;
        let mut clamp: i32 = 0;
        ftCo_800AA320(fp, core::ptr::addr_of_mut!(stick) as *mut i32, core::ptr::addr_of_mut!(clamp) as *mut i32);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B4778(fp, (CpuCmd_LstickForwardClamped as i32) as u8, stick as u8, clamp as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800AABC8_dontinline`
unsafe fn ftCo_800AABC8_dontinline(mut fp: *mut Fighter) {
    ftCo_800AABC8(fp);
}

/// `ftCo_800AACD0`
pub unsafe fn ftCo_800AACD0(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AACD0");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut temp_r3_2: *mut mp_UnkStruct0 = null_mut();
    let mut __: [u8; 8] = [0; 8];
    let mut stick: i32 = 0;
    let mut clamp: i32 = 0;
    let mut sp24: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp18: Vec3 = unsafe { core::mem::zeroed() };
    if isInTeeter(fp) != 0 {
        ftCo_CpuTurnAwayFromLedge(fp);
        return;
    }
    ftCo_800AA320(fp, core::ptr::addr_of_mut!(stick) as *mut i32, core::ptr::addr_of_mut!(clamp) as *mut i32);
    if ((*fp).coll_data.floor.index != (-1_i32)) && ((ext::mpLineGetFlags((*fp).coll_data.floor.index) & ((1 << 8) as u32)) != 0) {
        ftCo_800A08F0(fp);
        return;
    }
    temp_r3_2 = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
    if !temp_r3_2.is_null() {
        ext::mpIsland_8005ACE8(temp_r3_2, core::ptr::addr_of_mut!(sp24) as *mut Vec, core::ptr::addr_of_mut!(sp18) as *mut Vec);
        if (((*data).x54.x < sp24.x) || ((*data).x54.x > sp18.x)) && (!(ftCo_800A2BD4(fp) != 0)) {
            ftCo_CpuTurnAround(fp);
            return;
        }
    }
    if (ext::stage_info.grkind != ((Gr_Kind_Icemt as i32) as u32)) && (inlineC0(fp) != 0) {
        ftCo_800A0148(fp);
        return;
    }
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
    super::ftcmdscript::ftCo_800B4778(fp, (CpuCmd_LstickForwardClamped as i32) as u8, stick as u8, clamp as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_800AAF48`
pub unsafe fn ftCo_800AAF48(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800AAF48");
    let mut temp_r29: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut cur: *mut mp_UnkStruct0 = null_mut();
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut sp44: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp38: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp2C: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp28: i32 = 0;
    let mut sp24: u32 = 0;
    if (*temp_r29).x60 != 0 {
        return 0;
    }
    if ext::stage_info.grkind != ((Gr_Kind_Shrine as i32) as u32) {
        return 0;
    }
    if ((*fp).facing_dir as f64) > 0.0f64 {
        {
            cur = ext::mpIsland_80458E88.next;
            'l109: while !cur.is_null() {
                'c109: {
                    if !(ext::ScriptGame_StageJointActive((*cur).x28 as i32) != 0) {
                        break 'c109;
                    }
                    sp44 = (*cur).x8;
                    dx = sp44.x - (*fp).cur_pos.x;
                    dy = sp44.y - (*fp).cur_pos.y;
                    if dx < 0.0f32 {
                        break 'c109;
                    }
                    if dy < 0.0f32 {
                        break 'c109;
                    }
                    if ext::mpCheckCeiling((*fp).cur_pos.x, (*fp).cur_pos.y, sp44.x, sp44.y, core::ptr::addr_of_mut!(sp38) as *mut Vec, core::ptr::addr_of_mut!(sp28) as *mut i32, core::ptr::addr_of_mut!(sp24) as *mut u32, core::ptr::addr_of_mut!(sp2C) as *mut Vec, (-1_i32), (-1_i32)) != 0 {
                        break 'c109;
                    }
                    if ext::mpCheckRightWall((*fp).cur_pos.x, (*fp).cur_pos.y, sp44.x, sp44.y, core::ptr::addr_of_mut!(sp38) as *mut Vec, core::ptr::addr_of_mut!(sp28) as *mut i32, core::ptr::addr_of_mut!(sp24) as *mut u32, core::ptr::addr_of_mut!(sp2C) as *mut Vec, (-1_i32), (-1_i32)) != 0 {
                        break 'c109;
                    }
                    if (dy < fmadds(2.0f32, (*temp_r29).x558, (*temp_r29).x568)) && (dx < (5.0f32 * (*temp_r29).x564)) {
                        (*fp).cpu.x60 = 300;
                        (*fp).cpu.x64.x = (*fp).cpu.x54.x;
                        (*fp).cpu.x64.y = (*fp).cpu.x54.y;
                        (*fp).cpu.x54.x = sp44.x;
                        (*fp).cpu.x54.y = sp44.y;
                        (*fp).cpu.x38 = 5.0f32;
                        return 1;
                    }
                }
                cur = (*cur).next;
            }
        }
    } else {
        {
            cur = ext::mpIsland_80458E88.next;
            'l110: while !cur.is_null() {
                'c110: {
                    if !(ext::ScriptGame_StageJointActive((*cur).x28 as i32) != 0) {
                        break 'c110;
                    }
                    let mut spC: Vec3 = (*cur).x14;
                    dx = (*fp).cur_pos.x - spC.x;
                    dy = (*fp).cur_pos.y - spC.y;
                    if dx < 0.0f32 {
                        break 'c110;
                    }
                    if dy < 0.0f32 {
                        break 'c110;
                    }
                    if ext::mpCheckCeiling((*fp).cur_pos.x, (*fp).cur_pos.y, spC.x, spC.y, core::ptr::addr_of_mut!(sp38) as *mut Vec, core::ptr::addr_of_mut!(sp28) as *mut i32, core::ptr::addr_of_mut!(sp24) as *mut u32, core::ptr::addr_of_mut!(sp2C) as *mut Vec, (-1_i32), (-1_i32)) != 0 {
                        break 'c110;
                    }
                    if ext::mpCheckLeftWall((*fp).cur_pos.x, (*fp).cur_pos.y, spC.x, spC.y, core::ptr::addr_of_mut!(sp38) as *mut Vec, core::ptr::addr_of_mut!(sp28) as *mut i32, core::ptr::addr_of_mut!(sp24) as *mut u32, core::ptr::addr_of_mut!(sp2C) as *mut Vec, (-1_i32), (-1_i32)) != 0 {
                        break 'c110;
                    }
                    if (dy < fmadds(2.0f32, (*temp_r29).x558, (*temp_r29).x568)) && (dx < (5.0f32 * (*temp_r29).x564)) {
                        (*fp).cpu.x60 = 300;
                        (*fp).cpu.x64.x = (*fp).cpu.x54.x;
                        (*fp).cpu.x64.y = (*fp).cpu.x54.y;
                        (*fp).cpu.x54.x = spC.x;
                        (*fp).cpu.x54.y = spC.y;
                        (*fp).cpu.x38 = 5.0f32;
                        return 1;
                    }
                }
                cur = (*cur).next;
            }
        }
    }
    return 0;
}

/// `ftCo_800AB224`
pub unsafe fn ftCo_800AB224(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AB224");
    let mut temp_r31: *mut CpuFighter = null_mut();
    let mut temp_f1: f32 = 0.0;
    let mut var_f1_3: f32 = 0.0;
    let mut var_f2: f32 = 0.0;
    let mut var_f31: f32 = 0.0;
    let mut temp_r0: *mut Fighter = null_mut();
    let mut temp_r29: *mut mp_UnkStruct0 = null_mut();
    let mut temp_r28: *mut CpuFighter = null_mut();
    let mut var_r0_5: i32 = 0;
    let mut var_r0_6: i32 = 0;
    let mut __: [u8; 60] = [0; 60];
    let mut sp44: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp40: u32 = 0;
    let mut sp3C: i32 = 0;
    let mut sp30: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp24: Vec3 = unsafe { core::mem::zeroed() };
    temp_r31 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*temp_r31).xFA_b6() != 0 {
        if ((*fp).facing_dir as f64) > 0.0f64 {
            ftCo_CpuTurnAround(fp);
            return;
        }
        if !(ftCo_800A28D0(fp, 1.0f32) != 0) {
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 176 as u8);
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
            super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
            return;
        }
        ftCo_800A0384(fp);
        return;
    }
    let mut go49: i32 = 0;
    if (*temp_r31).xF9_b1() != 0 {
        temp_r0 = (*temp_r31).x44;
        if !temp_r0.is_null() {
            if ftCo_800A2C08(fp) != 0 {
                if ftCo_800A2170(fp, temp_r0) != 0 {
                    var_f1_3 = if ((*temp_r0).cur_pos.x - (*fp).cur_pos.x) < (0 as f32) { -((*temp_r0).cur_pos.x - (*fp).cur_pos.x) } else { (*temp_r0).cur_pos.x - (*fp).cur_pos.x };
                    if (var_f1_3 as f64) < 37.5f64 {
                        ftCo_800A0148(fp);
                        return;
                    }
                }
            }
        }
        if (((!((*temp_r31).xF8_b6() != 0)) || ((*fp).ground_or_air != ((GA_Ground as i32) as u32))) || (!(ext::HSD_Randf() < (0.05f32 * ((*temp_r31).level as f32))))) || ({ { (*temp_r31).set_xF8_b6((0 as u8) as u8); (*temp_r31).xF8_b6() }; !super::ftcpuattack::ftCo_800BA2E8(fp, temp_r0) }) {
            go49 = 1;
        }
    } else {
        go49 = 1;
    }
    if go49 != 0 {
        temp_r28 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
        if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
            var_r0_5 = 0;
        } else {
            temp_r29 = ext::mpIsland_8005AB54((*fp).coll_data.floor.index);
            if temp_r29.is_null() {
                var_r0_5 = 0;
            } else {
                sp44.x = (*temp_r28).x54.x;
                sp44.y = (5.0f64 + ((*temp_r28).x54.y as f64)) as f32;
                sp44.z = 0.0f32;
                if ext::mpIsland_8005AC14(core::ptr::addr_of_mut!(sp44) as *mut Vec, -1e+01f32) == temp_r29 {
                    var_r0_5 = 1;
                } else {
                    var_r0_5 = 0;
                }
            }
        }
        if var_r0_5 != 0 {
            ftCo_800AA42C(fp);
            return;
        }
        var_f2 = if ((*temp_r31).x54.x - (*fp).cur_pos.x) < (0 as f32) { -((*temp_r31).x54.x - (*fp).cur_pos.x) } else { (*temp_r31).x54.x - (*fp).cur_pos.x };
        temp_f1 = (*temp_r31).x54.y - (*fp).cur_pos.y;
        if (var_f2 < 1e-05f32) && (var_f2 > (-1e-05f32)) {
            var_r0_5 = 1;
        } else {
            var_r0_5 = 0;
        }
        if var_r0_5 != 0 {
            if (temp_f1 as f64) > 0.0f64 {
                var_f31 = (3.141592653589793f64 / (2 as f64)) as f32;
            } else {
                var_f31 = (-(3.141592653589793f64 / (2 as f64))) as f32;
            }
        } else {
            var_f31 = ext::lb_8000D008(temp_f1, var_f2);
        }
        if (var_f31 as f64) > 0.6108652334660292f64 {
            if ext::mpCheckCeiling((*fp).coll_data.cur_pos.x, (*fp).coll_data.cur_pos.y, (*fp).coll_data.cur_pos.x, (*fp).cpu.x54.y, core::ptr::addr_of_mut!(sp24) as *mut Vec, core::ptr::addr_of_mut!(sp3C) as *mut i32, core::ptr::addr_of_mut!(sp40) as *mut u32, core::ptr::addr_of_mut!(sp30) as *mut Vec, (-1_i32), (-1_i32)) != 0 {
                var_r0_6 = 1;
            } else {
                var_r0_6 = 0;
            }
            if var_r0_6 != 0 {
                if ftCo_800A28D0(fp, 1.0f32) != 0 {
                    ftCo_CpuTurnAround(fp);
                } else if ftCo_800AAF48(fp) != 0 {
                    ftCo_800A0148(fp);
                } else {
                    ftCo_800AABC8_dontinline(fp);
                }
            } else if ext::stage_info.grkind == ((Gr_Kind_Icemt as i32) as u32) {
                ftCo_800A0148(fp);
            } else if !(ftCo_800A2BD4(fp) != 0) {
                if (var_f31 as f64) < 1.3089969288557768f64 {
                    ftCo_CpuTurnAround(fp);
                } else {
                    ftCo_800AABC8_dontinline(fp);
                }
            } else {
                ftCo_800A0148(fp);
            }
        } else if (var_f31 as f64) > (-0.7853981573134661f64) {
            if !(ftCo_800A2BD4(fp) != 0) {
                ftCo_CpuTurnAround(fp);
            } else {
                ftCo_800AA844(fp);
            }
        } else {
            ftCo_800AACD0(fp);
        }
    }
}

/// `ftCo_800ABA34`
pub unsafe fn ftCo_800ABA34(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800ABA34");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*data).kind == (11 as u32) {
        if (*fp).item_gobj.is_null() {
            super::ftcmdscript::ftCo_800B4880(fp, 38);
            return;
        }
    } else if (*data).kind == (12 as u32) {
        if ((*data).x7C % 30) == 0 {
            super::ftcmdscript::ftCo_800B4880(fp, 38);
            return;
        }
    } else if (((*data).kind == (16 as u32)) && ((*fp).ground_or_air == ((GA_Ground as i32) as u32))) && (((*data).x7C % 300) == 0) {
        ftCo_800A05F4(fp);
        return;
    }
    if ((ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Zelda as i32)) || (ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Seak as i32))) && (((*data).xF8_b5() as i32) != 0) {
        super::ftcmdscript::ftCo_800B4880(fp, 38);
        (*data).set_xF8_b5((0 as u8) as u8);
    } else if (ftCo_800A5CE0(fp).is_null()) && ((*fp).motion_id == ftCo_MS_RebirthWait) {
        ftCo_800A08F0(fp);
    } else if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        ftCo_800A9CB4(fp);
    } else {
        ftCo_800AB224(fp);
    }
}

/// `ftCo_800ABBA8_blk155144r`
unsafe fn ftCo_800ABBA8_blk155144r(mut fp: *mut Fighter, mut target: *mut *mut Fighter) {
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    *target = (*data).x44;
}

/// `ftCo_800ABBA8`
#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe fn ftCo_800ABBA8(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800ABBA8");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut target: *mut Fighter = null_mut();
    let mut below: *mut Fighter = null_mut();
    let mut grav_ptr: *mut f32 = null_mut();
    let mut sp74: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp68: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut flags: u32 = 0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local2 { v: Vec3, __: [u8; 4] }
    let mut sp50: Local2 = unsafe { core::mem::zeroed() };
    let mut sqrt_gap: [u8; 12] = [0; 12];
    let mut sqrt_tmp: [f32; 4] = [0.0; 4];
    let mut result: i32 = 0;
    let mut blocked: i32 = 0;
    let mut ok: i32 = 0;
    let mut t: i32 = 0;
    let mut cx: f32 = 0.0;
    let mut cy: f32 = 0.0;
    let mut g: f32 = 0.0;
    let mut v: f32 = 0.0;
    let mut h: f32 = 0.0;
    let mut disc: f32 = 0.0;
    let mut vf0: f32 = 0.0;
    let mut vf5: f32 = 0.0;
    let mut land_y: f32 = 0.0;
    let mut tmp: f32 = 0.0;
    let mut dyy: f32 = 0.0;
    let mut dxx: f32 = 0.0;
    ftCo_800ABBA8_blk155144r(fp, core::ptr::addr_of_mut!(target) as *mut *mut Fighter);
    sqrt_gap.as_mut_ptr();
    if (*data).level < 5 {
        ok = 0;
    } else if (*fp).motion_id == 38 {
        ok = 1;
    } else {
        ok = 0;
    }
    if ok == 0 {
        (*data).x18 = (*data).x1C;
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseR as i32) as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
        return;
    }
    if (!target.is_null()) && (ftCo_800A6700(fp, core::ptr::addr_of_mut!((*target).cur_pos) as *mut Vec, core::ptr::addr_of_mut!(sp50.v) as *mut Vec) != 0) {
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        y = sp50.v.y;
        x = sp50.v.x;
        ftCo_800A1F3C(fp, x, y, 5.0f32);
    }
    if (*data).level > 7 {
        cx = (*fp).cur_pos.x;
        cy = (*fp).cur_pos.y;
        blocked = 0;
        line_id = (-1_i32);
        {
            let mut cx2: f32 = cx;
            let mut floor_y: f32 = ((cy as f64) - 1000.0f64) as f32;
            result = ext::mpCheckFloor(cx2, cy, cx, floor_y, 0.0f32, core::ptr::addr_of_mut!(sp74) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(sp68) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, blocked as usize as *mut HSD_GObj);
        }
        if result != 0 {
            let mut line: i32 = line_id;
            if (((ext::grBigBlue_801EF844(line) != 0) || (ext::grInishie1_801FCAAC(line) != 0)) || (ext::grCorneria_801E2D90(line) != 0)) || (ext::grVenom_80206D10(line) != 0) {
                blocked = 1;
            }
            if blocked != 0 {
                result = 0;
            }
        }
        if result != 0 {
            g = -(*fp).co_attrs.gravity;
            v = (*fp).pos_delta.y;
            h = sp74.y - ((*fp).coll_data.cur_pos.y + (*fp).coll_data.ecb.bottom.y);
            if (g < 1e-05f32) && (g > (-1e-05f32)) {
                ok = 1;
            } else {
                ok = 0;
            }
            if ok == 0 {
                disc = melee_runtime::gekko::sqrtf(if fmadds(2.0f32 * g, h, v * v) < (0 as f32) { -fmadds(2.0f32 * g, h, v * v) } else { fmadds(2.0f32 * g, h, v * v) });
                t = melee_core::ppc::fctiwz_f32(((-disc) - v) / g);
            } else {
                if (v < 1e-05f32) && (v > (-1e-05f32)) {
                    ok = 1;
                } else {
                    ok = 0;
                }
                if ok == 0 {
                    t = melee_core::ppc::fctiwz_f32(h / v);
                } else {
                    t = 100;
                }
            }
            if t < 10 {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
                super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressR as i32) as u8);
                super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
                return;
            }
        }
    }
    below = ftCo_800A4BEC(fp);
    if below.is_null() {
        ftCo_CpuReturnToPreviousBehavior(fp, data);
        return;
    }
    if (*below).cur_pos.y < (*fp).cur_pos.y {
        dyy = if ((*below).cur_pos.y - (*fp).cur_pos.y) < (0 as f32) { -((*below).cur_pos.y - (*fp).cur_pos.y) } else { (*below).cur_pos.y - (*fp).cur_pos.y };
        if dyy < ((*below).cpu.x568 + (*data).x568) {
            dxx = if ((*below).cur_pos.x - (*fp).cur_pos.x) < (0 as f32) { -((*below).cur_pos.x - (*fp).cur_pos.x) } else { (*below).cur_pos.x - (*fp).cur_pos.x };
            if (dxx as f64) < (5.0f64 + (((*below).cpu.x564 + (*data).x564) as f64)) {
                if ((*below).motion_id == 56) || ((*below).motion_id == 63) {
                    ok = 1;
                } else {
                    ok = 0;
                }
                if ok != 0 {
                    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseR as i32) as u8);
                    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
                    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressR as i32) as u8);
                    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
                    ftCo_CpuSetNeutralStick(fp);
                    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseR as i32) as u8);
                    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 50 as u8);
                    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 129 as u8);
                    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
                    ftCo_CpuFinishWithNeutralY(fp);
                    return;
                }
            }
        }
    }
    v = (*fp).pos_delta.x;
    dxx = (*data).x54.x - (*fp).cur_pos.x;
    if (v < 1e-05f32) && (v > (-1e-05f32)) {
        ok = 1;
    } else {
        ok = 0;
    }
    if ok != 0 {
        vf0 = 1e+03f32;
    } else {
        vf0 = dxx / v;
    }
    grav_ptr = core::ptr::addr_of_mut!((*fp).co_attrs.gravity) as *mut f32;
    ();
    g = *grav_ptr;
    if (g < 1e-05f32) && (g > (-1e-05f32)) {
        ok = 1;
    } else {
        ok = 0;
    }
    if ok != 0 {
        vf5 = 1e+03f32;
    } else {
        vf5 = (-((-(*fp).co_attrs.terminal_velocity) - (*fp).pos_delta.y)) / g;
    }
    if vf5 <= 0.0f32 {
        land_y = fmadds((*fp).pos_delta.y, vf0, (*fp).cur_pos.y);
    } else if vf0 < vf5 {
        tmp = melee_runtime::gekko::sqrtf(vf0);
        land_y = (((*fp).cur_pos.y as f64) + (-0.5f64).mul_add(((*grav_ptr) * tmp) as f64, ((*fp).pos_delta.y * vf0) as f64)) as f32;
    } else {
        tmp = melee_runtime::gekko::sqrtf(vf5);
        land_y = (((*fp).cur_pos.y as f64) + ((-0.5f64).mul_add(((*grav_ptr) * tmp) as f64, ((*fp).pos_delta.y * vf5) as f64) - (((vf0 - vf5) * ftCo_GetTerminalVelocity(fp)) as f64))) as f32;
    }
    if ((vf0 as f64) < 0.0f64) || (land_y < (*data).x54.y) {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_LstickXTowardDestination as i32) as u8, 127 as u8);
    } else {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
    }
    ftCo_CpuFinishWithNeutralY(fp);
}

/// `inlineG0`
unsafe fn inlineG0(mut fp: *mut Fighter) -> i32 {
    let mut temp_r0: i32 = (*fp).motion_id;
    let mut ret: i32 = 0;
    if temp_r0 == 224 {
        ret = 1;
    } else if temp_r0 == 227 {
        ret = 1;
    } else if (temp_r0 >= 266) && (temp_r0 <= 270) {
        ret = 1;
    } else {
        ret = 0;
    }
    return ret;
}

/// `ftCo_800AC30C_noinline`
pub unsafe fn ftCo_800AC30C_noinline(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AC30C_noinline");
    ftCo_800AC30C(fp);
}

/// `ftCo_800AC30C`
pub unsafe fn ftCo_800AC30C(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AC30C");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if !(inlineG0(fp) != 0) {
        ftCo_CpuReturnToPreviousBehavior(fp, data);
        return;
    }
    if (((*data).x7C % 3) == 0) && (!((((*data).level as f32) * 0.1f32) < ext::HSD_Randf())) {
        if ((*fp).cpu.lstick.x as i32) < 0 {
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 127 as u8);
        } else {
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, (-127_i32) as u8);
        }
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `inlineH0`
unsafe fn inlineH0(mut fp: *mut Fighter) -> i32 {
    if (*fp).motion_id == ftCo_MS_BarrelWait {
        return 1;
    }
    if (*fp).motion_id == ftCo_MS_Barrel {
        return 2;
    }
    return 0;
}

/// `inlineH1`
unsafe fn inlineH1(mut fp: *mut Fighter, mut data: *mut CpuFighter) {
    if (melee_runtime::cabi::srem((*data).x7C, (10 - (*data).level) * 5) == 0) && (ext::HSD_Randf() < 0.5f32) {
        ftCo_CpuTapA(fp);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    } else {
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800AC434`
pub unsafe fn ftCo_800AC434(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AC434");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut vec: Vec3 = unsafe { core::mem::zeroed() };
    let mut f: f32 = 0.0;
    let mut barrel_state: i32 = inlineH0(fp);
    if barrel_state == 0 {
        ftCo_CpuReturnToPreviousBehavior(fp, data);
        return;
    }
    if barrel_state == 1 {
        let _ = ext::Ground_801C4DA0(core::ptr::addr_of_mut!(vec) as *mut Vec, core::ptr::addr_of_mut!(f) as *mut f32);
    } else {
        inlineH1(fp, data);
        return;
    }
    if ((f as f64) >= (-0.26179938577115536f64)) && ((f as f64) <= 0.0f64) {
        ftCo_CpuTapA(fp);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    } else {
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800AC5A0`
pub unsafe fn ftCo_800AC5A0(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AC5A0");
    let mut data: *mut CpuFighter = null_mut();
    let mut var_r0: i32 = 0;
    let mut stick_x: i8 = 0;
    let mut stick_y: i8 = 0;
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if !((*fp).x221A_b3() != 0) {
        (*data).x18 = (*data).x1C;
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseR as i32) as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
        return;
    }
    if ((*data).x7C % 120) > (120 - ((*data).level * 12)) {
        var_r0 = 1;
    } else {
        var_r0 = 0;
    }
    if var_r0 != 0 {
        let mut kb_x: f32 = (*fp).x8c_kb_vel.x;
        let mut kb_y: f32 = (*fp).x8c_kb_vel.y;
        let mut kb_mag: f32 = 0.0;
        kb_mag = fmadds(kb_x, kb_x, kb_y * kb_y);
        kb_mag = melee_runtime::gekko::sqrtf(kb_mag);
        if !(ftCo_IsNearlyZero(kb_mag) != 0) {
            let mut x: f32 = kb_x * (1.0f32 / kb_mag);
            let mut y: f32 = kb_y * (1.0f32 / kb_mag);
            if kb_x > 0.0f32 {
                stick_y = melee_runtime::cabi::f2i8((127.0f32 * (-y)) as f64);
                stick_x = melee_runtime::cabi::f2i8((127.0f32 * x) as f64);
            } else {
                stick_y = melee_runtime::cabi::f2i8((127.0f32 * y) as f64);
                stick_x = melee_runtime::cabi::f2i8((127.0f32 * (-x)) as f64);
            }
        }
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, stick_y as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, stick_x as u8);
    } else {
        ftCo_CpuSetNeutralStick(fp);
    }
    if (*data).xFA_b1() != 0 {
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressR as i32) as u8);
    }
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
    super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
}

/// `ftCo_800AC7D4`
pub unsafe fn ftCo_800AC7D4(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AC7D4");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut temp_r3: *mut Fighter = null_mut();
    let mut temp_r0_2: i32 = 0;
    let mut var_r0: i32 = 0;
    var_r0 = ftCo_800A3134(fp);
    if var_r0 == 0 {
        ftCo_CpuReturnToPreviousBehavior(fp, data);
        return;
    }
    if var_r0 == 1 {
        ftCo_CpuFinishWithNeutralStick(fp);
        return;
    }
    temp_r0_2 = (*data).kind as i32;
    if (temp_r0_2 == 15) || (temp_r0_2 == 0) {
        ftCo_CpuHoldUpForOneFrame(fp);
        return;
    }
    temp_r3 = ftCo_800A4BEC(fp);
    if temp_r3.is_null() {
        ftCo_CpuHoldUpForOneFrame(fp);
    } else if (ftCo_800A1AB4(fp, temp_r3) as f64) < 15.0f64 {
        if ext::HSD_Randf() < (0.1f32 * ((*data).level as f32)) {
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
            super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
            ftCo_CpuRetapA(fp);
        } else {
            if ((*temp_r3).cur_pos.x - (*fp).cur_pos.x) < 0.0f32 {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 80 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 10 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
            } else {
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 176 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 10 as u8);
                super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
            }
            super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
        }
    } else {
        ftCo_CpuHoldUpForOneFrame(fp);
    }
}

/// `ftCo_800ACB44`
pub unsafe fn ftCo_800ACB44(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800ACB44");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut temp_r3: *mut Fighter = null_mut();
    let mut temp_f1: f32 = 0.0;
    let mut temp_f1_2: f32 = 0.0;
    let mut temp_f2: f32 = 0.0;
    let mut temp_r0_2: i32 = 0;
    let mut var_r0: i32 = 0;
    var_r0 = ftCo_800A3200(fp);
    if var_r0 == 1 {
        ftCo_CpuFinishWithNeutralStick(fp);
        return;
    }
    if var_r0 == 0 {
        ftCo_CpuReturnToPreviousBehavior(fp, data);
        return;
    }
    if ((*data).kind == (15 as u32)) || ((*data).kind == (0 as u32)) {
        ftCo_800A0AF4(fp);
        return;
    }
    temp_r3 = ftCo_800A4BEC(fp);
    if temp_r3.is_null() {
        ftCo_800A0AF4(fp);
        return;
    }
    temp_f1_2 = ftCo_800A1AB4(fp, temp_r3);
    if (((*data).xF9_b2() as i32) != 0) && ((temp_f1_2 as f64) < 30.0f64) {
        if (0.1f32 * ((*data).level as f32)) > ext::HSD_Randf() {
            ftCo_CpuRetapR(fp);
        } else {
            ftCo_CpuRetapA(fp);
        }
    } else {
        ftCo_800A0AF4(fp);
    }
}

/// `ftCo_800ACD5C`
pub unsafe fn ftCo_800ACD5C(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800ACD5C");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*data).xF9_b1() != 0 {
        let mut enemy: *mut Fighter = (*data).x44;
        if (!enemy.is_null()) && (((*data).xF8_b6() as i32) != 0) {
            (*data).set_xF8_b6((0 as u8) as u8);
            super::ftcpuattack::ftCo_800BA674(fp, enemy);
            return;
        }
    }
    if ((*data).x88 > 0) && ((ext::gm_GetCurrentGameMode() as i32) != (GM_TRAINING as i32)) {
        ftCo_CpuSetNeutralStick(fp);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressUp as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseUp as i32) as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
        (*data).x88 = 0;
        (*data).x8C = ext::gm_8016C75C((*fp).gobj);
        return;
    }
    if ((*data).kind == (11 as u32)) && ((*fp).item_gobj.is_null()) {
        super::ftcmdscript::ftCo_800B4880(fp, 38);
        return;
    }
    if ftCo_800A3554(fp, 1.0f32) == 0 {
        ftCo_CpuReturnToPreviousBehavior(fp, data);
        return;
    }
    if ((ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Zelda as i32)) || (ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Seak as i32))) && (((*data).xF8_b5() as i32) != 0) {
        super::ftcmdscript::ftCo_800B4880(fp, 38);
        (*data).set_xF8_b5((0 as u8) as u8);
        return;
    }
    if !(ftCo_800A2C08(fp) != 0) {
        ftCo_CpuTurnAround(fp);
        return;
    }
    if (*data).level < 5 {
        ftCo_CpuFinishWithNeutralStick(fp);
        return;
    }
    if ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Donkey as i32) {
        if ((*fp).motion_id != (ftDk_MS_SpecialNLoop as i32)) && ((*fp).u.dk.x222C == 0) {
            ftCo_CpuStartCharging(fp);
            return;
        }
    } else if ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Samus as i32) {
        if ((*fp).motion_id != (ftSs_MS_SpecialNHold as i32)) && ((*fp).u.ss.x2230 == 0) {
            ftCo_CpuStartCharging(fp);
            return;
        }
    } else if ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Mewtwo as i32) {
        let mut da: *mut _ftMewtwoAttributes = (*fp).dat_attrs as *mut _ftMewtwoAttributes;
        if ((*fp).u.mt.x2234_shadowBallCharge as f32) == (*da).x0_MEWTWO_SHADOWBALL_CHARGE_CYCLES {
            if ((*fp).motion_id == (ftMt_MS_SpecialNLoop as i32)) || ((*fp).motion_id == (ftMt_MS_SpecialNLoopFull as i32)) {
                ftCo_CpuTapR(fp);
                super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
                return;
            }
        } else if ((*fp).motion_id != (ftMt_MS_SpecialNLoop as i32)) && ((*fp).u.mt.x2234_shadowBallCharge == 0) {
            ftCo_CpuStartCharging(fp);
            return;
        }
    } else if (ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Kirby as i32)) || (((ftAi_Kind((*fp).kind as i32) >= (Ft_Kind_Mex0 as i32)) && (ftAi_Kind((*fp).kind as i32) < (Ft_Kind_Max as i32))) && (ext::ftKb_MexKirbyLike(ftAi_Kind((*fp).kind as i32)) != 0)) {
        if (*fp).u.kb.hat.kind == ((Ft_Kind_Donkey as i32) as u32) {
            if ((*fp).motion_id != (ftKb_MS_DkSpecialNLoop as i32)) && ((*fp).u.kb.xBC == 0) {
                ftCo_CpuStartCharging(fp);
                return;
            }
        } else if (*fp).u.kb.hat.kind == ((Ft_Kind_Samus as i32) as u32) {
            if ((*fp).motion_id != (ftKb_MS_SsSpecialNHold as i32)) && ((*fp).u.kb.xA8 == 0) {
                ftCo_CpuStartCharging(fp);
                return;
            }
        } else if (*fp).u.kb.hat.kind == ((Ft_Kind_Mewtwo as i32) as u32) {
            let mut da: *mut ftKb_DatAttrs = (*fp).dat_attrs as *mut ftKb_DatAttrs;
            if ((*fp).u.kb.x9C as f32) == (*da).specialn_mt_charge_time {
                if ((*fp).motion_id == (ftKb_MS_MtSpecialNLoop as i32)) || ((*fp).motion_id == (ftKb_MS_MtSpecialNLoopFull as i32)) {
                    ftCo_CpuTapR(fp);
                    return;
                }
            } else if ((*fp).motion_id != (ftKb_MS_MtSpecialNLoop as i32)) && ((*fp).u.kb.x9C == 0) {
                ftCo_CpuStartCharging(fp);
                return;
            }
        }
    }
    ftCo_CpuFinishWithNeutralStick(fp);
}

/// `ftCo_800AD42C`
pub unsafe fn ftCo_800AD42C(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AD42C");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if !(ftCo_800A3554(fp, 1.0f32) != 0) {
        ftCo_CpuReturnToPreviousBehavior(fp, data);
    } else if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        ftCo_CpuFinishWithNeutralStick(fp);
    } else {
        ftCo_CpuSetNeutralStick(fp);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 1 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_PressY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, ((*ext::p_ftCommonData).tap_jump_window + 1) as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_ReleaseY as i32) as u8);
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_WaitFor as i32) as u8, 5 as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `ftCo_800AD54C`
pub unsafe fn ftCo_800AD54C(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AD54C");
    let mut __: [u8; 8] = [0; 8];
    let mut stick: i32 = 0;
    let mut clamp: i32 = 0;
    ftCo_800AA320(fp, core::ptr::addr_of_mut!(stick) as *mut i32, core::ptr::addr_of_mut!(clamp) as *mut i32);
    if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
        super::ftcmdscript::ftCo_800B4778(fp, (CpuCmd_LstickForwardClamped as i32) as u8, stick as u8, clamp as u8);
        ftCo_CpuFinishWithNeutralY(fp);
    } else if ftCo_800A28D0(fp, 1.0f32) != 0 {
        ftCo_CpuTurnAround(fp);
    } else if inlineC0(fp) != 0 {
        if ext::HSD_Randf() < 0.5f32 {
            ftCo_800A0384(fp);
        } else {
            ftCo_CpuTurnAround(fp);
        }
    } else if !ext::mpIsland_8005AB54((*fp).coll_data.floor.index).is_null() {
        super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
        super::ftcmdscript::ftCo_800B4778(fp, (CpuCmd_LstickForwardClamped as i32) as u8, stick as u8, clamp as u8);
        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
    }
}

/// `is_capsule_egg_ball`
unsafe fn is_capsule_egg_ball(mut kind: ItemKind) -> i32 {
    if ((kind == It_Kind_Capsule) || (kind == It_Kind_Egg)) || (kind == It_Kind_M_Ball) {
        return 1;
    }
    return 0;
}

/// `ftCo_800AD7FC`
pub unsafe fn ftCo_800AD7FC(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AD7FC");
    let mut enemy: *mut Fighter = null_mut();
    let mut data: *mut CpuFighter = null_mut();
    let mut item: *mut Item = null_mut();
    let mut angle: f32 = 0.0;
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*fp).item_gobj.is_null() {
        ftCo_CpuReturnToPreviousBehavior(fp, data);
        return;
    }
    enemy = (*data).x44;
    item = HSD_GObjGetUserData((*fp).item_gobj) as *mut Item;
    if (!enemy.is_null()) && ((*data).kind == (29 as u32)) {
        if !(ftCo_800A2C08(fp) != 0) {
            ftCo_CpuTurnAround(fp);
        } else {
            angle = ext::lb_8000D008((*enemy).cur_pos.y - (*fp).cur_pos.y, if ((*enemy).cur_pos.x - (*fp).cur_pos.x) < (0 as f32) { -((*enemy).cur_pos.x - (*fp).cur_pos.x) } else { (*enemy).cur_pos.x - (*fp).cur_pos.x });
            if (angle as f64) > 1.3089969288557768f64 {
                super::ftcmdscript::ftCo_800B4880(fp, 43);
            } else if (angle as f64) < (-1.3089969288557768f64) {
                super::ftcmdscript::ftCo_800B4880(fp, 44);
            } else {
                super::ftcmdscript::ftCo_800B4880(fp, 42);
            }
            (*data).x18 = (*data).x1C;
        }
    } else {
        if !enemy.is_null() {
            let mut unused: f32 = ftCo_800A1AB4(fp, enemy);
            angle = ext::lb_8000D008((*enemy).cur_pos.y - (*fp).cur_pos.y, if ((*enemy).cur_pos.x - (*fp).cur_pos.x) < (0 as f32) { -((*enemy).cur_pos.x - (*fp).cur_pos.x) } else { (*enemy).cur_pos.x - (*fp).cur_pos.x });
            if (angle as f64) > 1.3089969288557768f64 {
                super::ftcmdscript::ftCo_800B4880(fp, 13);
            } else if (angle as f64) < (-1.3089969288557768f64) {
                super::ftcmdscript::ftCo_800B4880(fp, 14);
            } else {
                if !(ftCo_800A2C08(fp) != 0) {
                    if is_capsule_egg_ball((*item).kind) != 0 {
                        super::ftcmdscript::ftCo_800B4880(fp, 14);
                    } else {
                        super::ftcmdscript::ftCo_800B4880(fp, 40);
                    }
                } else if ((angle as f64) < 0.26179938577115536f64) && ((angle as f64) > (-0.26179938577115536f64)) {
                    super::ftcmdscript::ftCo_800B4880(fp, 15);
                } else {
                    if is_capsule_egg_ball((*item).kind) != 0 {
                        super::ftcmdscript::ftCo_800B4880(fp, 14);
                    } else {
                        super::ftcmdscript::ftCo_800B4880(fp, 40);
                    }
                }
            }
        } else {
            if is_capsule_egg_ball((*item).kind) != 0 {
                super::ftcmdscript::ftCo_800B4880(fp, 14);
            } else {
                super::ftcmdscript::ftCo_800B4880(fp, 40);
            }
        }
        (*data).x18 = (*data).x1C;
    }
}

/// `ftCo_800ADC28`
pub unsafe fn ftCo_800ADC28(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800ADC28");
    if (*fp).cpu.x18 == 10 {
        return;
    }
    if ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Donkey as i32) {
        if (*fp).motion_id == (ftDk_MS_SpecialNLoop as i32) {
            ftCo_CpuTapR(fp);
        }
    } else if ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Samus as i32) {
        if (*fp).motion_id == (ftSs_MS_SpecialNHold as i32) {
            ftCo_CpuTapRAndWait(fp);
        }
    } else if ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Mewtwo as i32) {
        if ((*fp).motion_id == (ftMt_MS_SpecialNLoop as i32)) || ((*fp).motion_id == (ftMt_MS_SpecialNLoopFull as i32)) {
            ftCo_CpuTapRAndWait(fp);
        }
    } else if (ftAi_Kind((*fp).kind as i32) == (Ft_Kind_Kirby as i32)) || (((ftAi_Kind((*fp).kind as i32) >= (Ft_Kind_Mex0 as i32)) && (ftAi_Kind((*fp).kind as i32) < (Ft_Kind_Max as i32))) && (ext::ftKb_MexKirbyLike(ftAi_Kind((*fp).kind as i32)) != 0)) {
        let mut kind: FighterKind = (*fp).u.kb.hat.kind;
        if (kind == ((Ft_Kind_Donkey as i32) as u32)) && ((*fp).motion_id == (ftKb_MS_DkSpecialNLoop as i32)) {
            ftCo_CpuTapRAndWait(fp);
        } else if (kind == ((Ft_Kind_Samus as i32) as u32)) && ((*fp).motion_id == (ftKb_MS_SsSpecialNHold as i32)) {
            ftCo_CpuTapRAndWait(fp);
        } else if (kind == ((Ft_Kind_Mewtwo as i32) as u32)) && (((*fp).motion_id == (ftKb_MS_MtSpecialNLoop as i32)) || ((*fp).motion_id == (ftKb_MS_MtSpecialNLoopFull as i32))) {
            ftCo_CpuTapRAndWait(fp);
        }
    }
}

/// `ftCo_ItemCheck`
unsafe fn ftCo_ItemCheck(mut ip: *mut Item) -> i32 {
    return ftCo_800A5980(ip as *mut Fighter);
}

/// `ftCo_800ADE48_inline0`
unsafe fn ftCo_800ADE48_inline0(mut y: f32) -> f32 {
    return y - 1e+03f32;
}

/// `ftCo_800ADE48_inline1`
unsafe fn ftCo_800ADE48_inline1(mut fp: *mut Fighter) -> *mut CpuFighter {
    return core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
}

/// `ftCo_800ADE48`
unsafe fn ftCo_800ADE48(fp: *mut Fighter) -> i32 {
    ftCo_800ADE48_r31(fp, true)
}

/// `ftCo_800ADE48` with the caller's r31 made explicit. On the GameCube the "level roll" branch of the 0x12 command
/// switch (fp->x221A_b3 set) leaves `switch_cmd` unassigned: it lives in r31, which the prologue saved but did not
/// clear, so the test reads the CALLER's r31. Every caller holds a pointer there (fp, the CPU block) except
/// ftCo_800B21C8's last call (0x800B2494), where r31 was set to 0 for its bit clears. Seen in Dolphin: a probe at
/// 0x800AE270 shows that branch with r31 = the CPU block pointer, and the console switching to command 0x12.
unsafe fn ftCo_800ADE48_r31(fp: *mut Fighter, caller_r31_nonzero: bool) -> i32 {
    let b = (*fp).cpu.x38.to_bits();
    let r = ftCo_800ADE48_impl(fp, caller_r31_nonzero);
    if std::env::var_os("MELEE_DBG_CPU").is_some() { eprintln!("    ADE48 ret {} x38 {:08X} -> {:08X} x54 {:?} x60 {}", r, b, (*fp).cpu.x38.to_bits(), ((*fp).cpu.x54.x, (*fp).cpu.x54.y), (*fp).cpu.x60); }
    r
}

unsafe fn ftCo_800ADE48_impl(mut fp: *mut Fighter, caller_r31_nonzero: bool) -> i32 {
    let mut switch_cmd: i32 = 0;
    let mut data: *mut CpuFighter = null_mut();
    let mut ip: *mut Item = null_mut();
    let mut flags: u32 = 0;
    let mut line_id: i32 = 0;
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut item_gobj: *mut HSD_GObj = null_mut();
    let mut data2: *mut CpuFighter = null_mut();
    let mut kind: ItemKind = 0;
    let mut found: i32 = 0;
    let mut result: i32 = 0;
    let mut in_bounds: i32 = 0;
    let mut motion_id: i32 = 0;
    let mut cur_cmd: i32 = 0;
    let mut five: f64 = 0.0;
    let mut x: f32 = 0.0;
    let mut x2: f32 = 0.0;
    let mut y: f32 = 0.0;
    let mut below: f32 = 0.0;
    let mut above: f32 = 0.0;
    let mut dy: f32 = 0.0;
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    found = 0;
    x = (*data).x54.x;
    x2 = x;
    y = (*data).x54.y;
    five = 5.0f64;
    below = ((y as f64) - five) as f32;
    above = (five + (y as f64)) as f32;
    line_id = (-1_i32);
    'l111: loop {
        result = ext::mpCheckFloor(x2, above, x, below, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, found as usize as *mut HSD_GObj);
        if (result != 0) && (ftCo_800A1B38_noinline(line_id) != 0) {
            ();
        } else {
            found = result;
        }
        if found != 0 {
            let mut data2: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
            y = (*data).x54.y;
            x = (*data).x54.x;
            if (((x < ((*fp).cpu.half_width + ext::Stage_GetBlastZoneLeftOffset())) || (x > (ext::Stage_GetBlastZoneRightOffset() - (*data2).half_width))) || (y < ((*data2).half_height + ext::Stage_GetBlastZoneBottomOffset()))) || (y > (ext::Stage_GetBlastZoneTopOffset() - (*data2).half_height)) {
                in_bounds = 1;
            } else {
                in_bounds = 0;
            }
            if in_bounds == 0 {
                break 'l111;
            }
        }
        (*data).set_xFA_b2((0 as u8) as u8);
        if (*fp).ground_or_air == ((GA_Air as i32) as u32) {
            x = (*fp).cur_pos.x;
            x2 = x;
            y = (*fp).cur_pos.y;
            line_id = (-1_i32);
            found = 0;
            result = ext::mpCheckFloor(x2, y, x, ftCo_800ADE48_inline0(y), 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, found as usize as *mut HSD_GObj);
            if (result != 0) && (ftCo_800A1B38_noinline(line_id) != 0) {
                ();
            } else {
                found = result;
            }
            if found != 0 {
                (*data).x54.x = floor_pos.x;
                (*data).x54.y = floor_pos.y;
            } else {
                (*data).x54.x = (*fp).cur_pos.x;
                (*data).x54.y = (*fp).cur_pos.y;
            }
        } else {
            (*data).x54.x = (*fp).cur_pos.x;
            (*data).x54.y = (*fp).cur_pos.y;
        }
        if !(0 != 0) { break 'l111; }
    }
    dy = (*fp).cur_pos.y - (*fp).cpu.x54.y;
    {
        let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
        let mut dx: f32 = 0.0;
        let mut dist: f32 = 0.0;
        dx = (*fp).cur_pos.x - (*data).x54.x;
        dist = melee_runtime::gekko::sqrtf(fmadds(dx, dx, dy * dy));
        (*data).x5C = dist;
    }
    if (*data).x88 > 0 {
        (*data).x88 = (*data).x88 - 1;
    }
    if (*data).x8C != ext::gm_8016C75C((*fp).gobj) {
        (*data).x88 = 300;
        (*data).x8C = ext::gm_8016C75C((*fp).gobj);
    }
    if (*data).kind == (14 as u32) {
        if ((*data).x7C % 120) == 0 {
            if (ext::HSD_Randf() as f64) > 0.5f64 {
                (*data).set_xF8_b5((1 as u8) as u8);
            } else {
                (*data).set_xF8_b5((0 as u8) as u8);
            }
        }
    } else if ((*data).x7C % 400) == 0 {
        if (ext::HSD_Randf() as f64) > 0.5f64 {
            (*data).set_xF8_b5((1 as u8) as u8);
        } else {
            (*data).set_xF8_b5((0 as u8) as u8);
        }
    }
    if !((*fp).anon_221b.x221B_b5() != 0) {
        (*data).x94 = 0;
    }
    if (*data).x18 != 18 {
        data2 = ftCo_800ADE48_inline1(fp);
        if ftAi_Kind((*fp).kind as i32) == (Ft_Kind_GKoops as i32) {
            switch_cmd = 0;
        } else if ((*data2).kind == (15 as u32)) || ((*data2).kind == (0 as u32)) {
            switch_cmd = 0;
        } else if !((*fp).x221A_b3() != 0) {
            switch_cmd = 0;
        } else {
            (*data2).set_xF9_b0((0 as u8) as u8);
            if (0.1f32 * ((*data2).level as f32)) > ext::HSD_Randf() {
                (*data2).set_xFA_b1((1 as u8) as u8);
            } else {
                (*data2).set_xFA_b1((0 as u8) as u8);
            }
            switch_cmd = caller_r31_nonzero as i32;
        }
        if switch_cmd != 0 {
            super::ftcmdscript::ftCo_800B4A78(fp);
            (*data).x18 = 18;
            return 1;
        }
    } else {
        return 1;
    }
    cur_cmd = (*data).x18;
    if cur_cmd != 17 {
        motion_id = (*fp).motion_id;
        switch_cmd = inlineH0(fp);
        if switch_cmd != 0 {
            super::ftcmdscript::ftCo_800B4A78(fp);
            (*data).x18 = 17;
            return 1;
        }
    } else {
        return 1;
    }
    if cur_cmd != 19 {
        if motion_id == 305 {
            switch_cmd = 1;
        } else if motion_id == 306 {
            switch_cmd = 2;
        } else {
            switch_cmd = 0;
        }
        if switch_cmd != 0 {
            super::ftcmdscript::ftCo_800B4A78(fp);
            (*data).x18 = 19;
            return 1;
        }
    } else {
        return 1;
    }
    if cur_cmd != 5 {
        's112: {
            match motion_id {
                191 | 183 => {
                    switch_cmd = 1;
                }
                192 | 184 => {
                    switch_cmd = 2;
                }
                _ => {
                    switch_cmd = 0;
                }
            }
        }
        if switch_cmd != 0 {
            super::ftcmdscript::ftCo_800B4A78(fp);
            (*data).x18 = 5;
            return 1;
        }
    } else {
        return 1;
    }
    if cur_cmd != 6 {
        's113: {
            match motion_id {
                252 => {
                    switch_cmd = 1;
                }
                253 => {
                    switch_cmd = 2;
                }
                _ => {
                    switch_cmd = 0;
                }
            }
        }
        if switch_cmd != 0 {
            super::ftcmdscript::ftCo_800B4A78(fp);
            (*data).x18 = 6;
            return 1;
        }
    } else {
        return 1;
    }
    if cur_cmd != 9 {
        if ftCo_IsGrabbing_dontinline(fp) != 0 {
            super::ftcmdscript::ftCo_800B4A78(fp);
            (*data).x18 = 9;
            return 1;
        }
    } else {
        return 1;
    }
    if (*data).x18 != 4 {
        if ftCo_800A2C80(fp) != 0 {
            super::ftcmdscript::ftCo_800B4A78(fp);
            (*data).x18 = 4;
            return 1;
        }
    } else {
        return 1;
    }
    cur_cmd = (*data).x18;
    if cur_cmd != 15 {
        if (*fp).cpu.level < 5 {
            switch_cmd = 0;
        } else if (*fp).motion_id == 38 {
            switch_cmd = 1;
        } else {
            switch_cmd = 0;
        }
        if switch_cmd != 0 {
            super::ftcmdscript::ftCo_800B4A78(fp);
            (*data).x18 = 15;
            return 1;
        }
    } else {
        return 1;
    }
    if cur_cmd != 16 {
        motion_id = (*fp).motion_id;
        if motion_id == 224 {
            switch_cmd = 1;
        } else if motion_id == 227 {
            switch_cmd = 1;
        } else if (motion_id >= 266) && (motion_id <= 270) {
            switch_cmd = 1;
        } else {
            switch_cmd = 0;
        }
        if switch_cmd != 0 {
            super::ftcmdscript::ftCo_800B4A78(fp);
            (*data).x18 = 16;
            return 1;
        }
    } else {
        return 1;
    }
    if cur_cmd != 7 {
        if ftCo_800A5ACC(fp) != 0 {
            let _ = super::ftcpuattack::ftCo_800BB9B4(fp);
            if (*data).xF8_b12() != 0 {
                super::ftcmdscript::ftCo_800B4A78(fp);
                (*data).x18 = 7;
                return 1;
            }
        }
    } else {
        let _ = super::ftcpuattack::ftCo_800BB9B4(fp);
        return 1;
    }
    if (*data).x18 != 3 {
        if super::ftcpuattack::ftCo_800B9CBC(fp) {
            super::ftcmdscript::ftCo_800B4A78(fp);
            (*data).x18 = 3;
            return 1;
        }
    } else {
        return 1;
    }
    if (*data).x18 != 2 {
        if super::ftcpuattack::ftCo_800B8A9C(fp) {
            super::ftcmdscript::ftCo_800B4A78(fp);
            (*data).x18 = 2;
            return 1;
        }
    } else {
        return 1;
    }
    if (*data).x18 != 8 {
        if { (*data).set_xF8_b34((super::ftcpuattack::ftCo_800B732C(fp) as u8) as u8); (*data).xF8_b34() } != 0 {
            (*data).x18 = 8;
            return 1;
        }
    } else {
        return 1;
    }
    if (*data).x18 != 13 {
        if ftCo_800A3710(fp) != 0 {
            (*data).x18 = 13;
            return 1;
        }
    } else {
        return 1;
    }
    if (*data).x18 != 14 {
        data2 = ftCo_800ADE48_inline1(fp);
        item_gobj = (*fp).item_gobj;
        if item_gobj.is_null() {
            found = 0;
        } else {
            ip = HSD_GObjGetUserData(item_gobj) as *mut Item;
            if (*data2).kind == (29 as u32) {
                found = 1;
            } else {
                kind = (*ip).kind;
                's114: {
                    match kind {
                        0 | 1 | 2 | 3 | 4 | 5 | 6 | 34 | 42 => {
                            found = 1;
                        }
                        _ => {
                            if ftCo_ItemCheck(ip) != 0 {
                                found = 1;
                            } else {
                                found = 0;
                            }
                        }
                    }
                }
            }
        }
        if found != 0 {
            (*data).x18 = 14;
            return 1;
        }
    } else {
        return 1;
    }
    if (*data).x18 != 10 {
        if ((*data).x20 != 0) && (ftCo_800A3554(fp, 0.0f32) != 0) {
            (*data).x18 = (*data).x20;
            return 1;
        }
    } else {
        return 1;
    }
    return 0;
}

/// `ftCo_CpuUpdateCommonItemTarget`
unsafe fn ftCo_CpuUpdateCommonItemTarget(mut fp: *mut Fighter) {
    let mut data: *mut CpuFighter = null_mut();
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (!(*fp).item_gobj.is_null()) && (!(ftCo_800A5908(HSD_GObjGetUserData((*fp).item_gobj) as *mut Item) != 0)) {
        (*data).x4C = null_mut();
    } else {
        if (*fp).x2168 != 0 {
            (*data).x4C = null_mut();
        } else {
            (*data).x4C = ftCo_800A5F4C(fp, It_Kind_L_Gun_Ray);
        }
    }
}

/// `ftCo_CpuUpdateSpecialItemTarget`
unsafe fn ftCo_CpuUpdateSpecialItemTarget(mut fp: *mut Fighter) {
    (*fp).cpu.x50 = ftCo_800A648C(fp) as u32;
}

/// `ftCo_CpuDataShouldAct`
unsafe fn ftCo_CpuDataShouldAct(mut data: *mut CpuFighter) -> i32 {
    if ((*data).x18 != (*data).x20) && ((*data).x18 != (*data).x1C) {
        (*data).x60 = 0;
    }
    if (*data).x18 == 4 {
        return 0;
    } else {
        (*data).set_xFA_b2((0 as u8) as u8);
        return 1;
    }
}

/// `ftCo_800AE7AC`
pub unsafe fn ftCo_800AE7AC(mut fp: *mut Fighter, mut arg1: *mut Vec, mut arg2: i32) {
    melee_core::ct!("ftCo_800AE7AC");
    let mut data0: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut data2: *mut CpuFighter = null_mut();
    let mut data3: *mut CpuFighter = null_mut();
    let mut do_act: i32 = 0;
    let mut __: [u8; 8] = [0; 8];
    let mut dir: Vec3 = unsafe { core::mem::zeroed() };
    let mut out: Vec3 = unsafe { core::mem::zeroed() };
    let mut left: f32 = 0.0;
    let mut sum_x: f32 = 0.0;
    let mut cx: f32 = 0.0;
    let mut bottom: f32 = 0.0;
    let mut sum_y: f32 = 0.0;
    let mut cy: f32 = 0.0;
    let mut dy: f32 = 0.0;
    (*data0).set_xF8_b0((1 as u8) as u8);
    (*data0).set_xF9_b3((0 as u8) as u8);
    (*data0).set_xF9_b5((0 as u8) as u8);
    (*data0).set_xF9_b7((0 as u8) as u8);
    (*data0).set_xF9_b1((0 as u8) as u8);
    if arg2 > 1 {
        (*data0).set_xF9_b2((0 as u8) as u8);
        (*data0).set_xF9_b4((0 as u8) as u8);
        (*data0).set_xF9_b6((0 as u8) as u8);
    } else {
        (*data0).set_xF9_b2((1 as u8) as u8);
        (*data0).set_xF9_b4((1 as u8) as u8);
        (*data0).set_xF9_b6((1 as u8) as u8);
    }
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    data2 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    do_act = ftCo_CpuDataShouldAct(data2);
    if do_act != 0 {
        if arg2 >= 0 {
            let _ = ftCo_800A8210(fp, arg1);
        } else {
            data3 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
            if (*fp).ground_or_air != ((GA_Air as i32) as u32) {
                left = ext::Stage_GetBlastZoneLeftOffset();
                sum_x = ext::Stage_GetBlastZoneRightOffset();
                cx = 0.5f32 * { sum_x = sum_x + left; sum_x };
                bottom = ext::Stage_GetBlastZoneBottomOffset();
                sum_y = ext::Stage_GetBlastZoneTopOffset();
                cy = 0.5f32 * { sum_y = sum_y + bottom; sum_y };
                dir.x = cx - (*fp).cur_pos.x;
                dir.y = cy - (*fp).cur_pos.y;
                dir.z = 0.0f32;
                if ftCo_800A6FC4(fp, core::ptr::addr_of_mut!(out) as *mut Vec, core::ptr::addr_of_mut!(dir) as *mut Vec) != 0 {
                    let mut x: f32 = 0.0;
                    let mut y: f32 = 0.0;
                    y = out.y;
                    x = out.x;
                    ftCo_800A1F3C(fp, x, y, 5.0f32);
                } else {
                    ftCo_800A75DC(fp, (*data3).x44);
                }
            }
        }
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_800AEA8C`
pub unsafe fn ftCo_800AEA8C(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AEA8C");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut data2: *mut CpuFighter = null_mut();
    let mut flags: [u32; 5] = [0; 5];
    let mut line_id: i32 = 0;
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut is_food: i32 = 0;
    let mut do_floor: i32 = 0;
    let mut found: i32 = 0;
    (*data).set_xF8_b0((0 as u8) as u8);
    (*data).set_xF9_b2(({ is_food = 1; is_food } as u8) as u8);
    (*data).set_xF9_b4((1 as u8) as u8);
    (*data).set_xF9_b3((0 as u8) as u8);
    (*data).set_xF9_b5((1 as u8) as u8);
    (*data).set_xF9_b6((1 as u8) as u8);
    (*data).set_xF9_b7((1 as u8) as u8);
    (*data).set_xF9_b1((0 as u8) as u8);
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    data2 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    do_floor = ftCo_CpuDataShouldAct(data2);
    if do_floor != 0 {
        let mut result: i32 = 0;
        let mut x: f32 = (*fp).cur_pos.x;
        let mut x2: f32 = x;
        let mut y: f32 = (*fp).cur_pos.y;
        let mut below: f32 = 0.0;
        let mut above: f32 = 0.0;
        found = 0;
        line_id = (-1_i32);
        below = y - 1e+03f32;
        above = 1e+01f32 + y;
        result = ext::mpCheckFloor(x2, above, x, below, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line_id) as *mut i32, flags.as_mut_ptr(), core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, found as usize as *mut HSD_GObj);
        'b115: {
            'b116: {
                if result == 0 {
                    break 'b116;
                }
                if ftCo_800A1B38_noinline(line_id) == 0 {
                    break 'b116;
                }
                ();
                break 'b115;
            }
            found = result;
        }
        if found != 0 {
            let mut floor_x: f32 = 0.0;
            let mut floor_y: f32 = 0.0;
            floor_y = floor_pos.y;
            floor_x = floor_pos.x;
            ftCo_800A1F3C(fp, floor_x, floor_y, 5.0f32);
        }
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_CpuShouldAct`
unsafe fn ftCo_CpuShouldAct(mut fp: *mut Fighter) -> i32 {
    let mut data: *mut CpuFighter = null_mut();
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ((*data).x18 != (*data).x20) && ((*data).x18 != (*data).x1C) {
        (*data).x60 = 0;
    }
    if (*data).x18 == 4 {
        return 0;
    } else {
        (*data).set_xFA_b2((0 as u8) as u8);
        return 1;
    }
}

/// `ftCo_CpuTargetAct`
unsafe fn ftCo_CpuTargetAct(mut fp: *mut Fighter) {
    ftCo_800A20A0(fp);
    if ftCo_CpuShouldAct(fp) != 0 {
        ftCo_800A80E4(fp);
    }
}

/// `ftCo_CpuSetTargetActive`
unsafe fn ftCo_CpuSetTargetActive(mut data: *mut CpuFighter) {
    (*data).set_xF8_b0((1 as u8) as u8);
}

/// `ftCo_CpuClearTargetModes`
unsafe fn ftCo_CpuClearTargetModes(mut data: *mut CpuFighter) {
    (*data).set_xF9_b2((0 as u8) as u8);
    (*data).set_xF9_b4((0 as u8) as u8);
    (*data).set_xF9_b3((0 as u8) as u8);
    (*data).set_xF9_b5((0 as u8) as u8);
    (*data).set_xF9_b6((0 as u8) as u8);
    (*data).set_xF9_b7((0 as u8) as u8);
}

/// `ftCo_CpuSetTargetReady`
unsafe fn ftCo_CpuSetTargetReady(mut data: *mut CpuFighter) {
    (*data).set_xF9_b1((1 as u8) as u8);
}

/// `ftCo_CpuSetTargetFlags`
unsafe fn ftCo_CpuSetTargetFlags(mut data: *mut CpuFighter) {
    ftCo_CpuSetTargetActive(data);
    ftCo_CpuClearTargetModes(data);
    ftCo_CpuSetTargetReady(data);
}

/// `ftCo_800AECF0`
pub unsafe fn ftCo_800AECF0(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AECF0");
    let mut data2: *mut CpuFighter = null_mut();
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut sp28: Vec3 = unsafe { core::mem::zeroed() };
    let mut cmd: i32 = 0;
    let mut target: *mut Fighter = null_mut();
    let mut do_act: i32 = 0;
    let mut is_food: i32 = 0;
    cmd = ftCo_800A229C(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec);
    if cmd != 0 {
        ftCo_800AE7AC(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec, cmd);
        return;
    }
    target = ftCo_800A5CE0(fp);
    (*data).x44 = target;
    if !target.is_null() {
        ftCo_CpuSetTargetFlags(data);
        ftCo_CpuTargetAct(fp);
        let _ = ftCo_800ADE48(fp);
        return;
    }
    (*data).set_xF8_b0((0 as u8) as u8);
    (*data).set_xF9_b2(({ is_food = 1; is_food } as u8) as u8);
    (*data).set_xF9_b4((1 as u8) as u8);
    (*data).set_xF9_b3((1 as u8) as u8);
    (*data).set_xF9_b5((1 as u8) as u8);
    (*data).set_xF9_b6((1 as u8) as u8);
    (*data).set_xF9_b7((1 as u8) as u8);
    (*data).set_xF9_b1((0 as u8) as u8);
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    data2 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ((*data2).x18 != (*data2).x20) && ((*data2).x18 != (*data2).x1C) {
        (*data2).x60 = 0;
    }
    if (*data2).x18 == 4 {
        do_act = 0;
    } else {
        (*data2).set_xFA_b2((0 as u8) as u8);
        do_act = 1;
    }
    if do_act != 0 {
        if !(*data).x4C.is_null() {
            ftCo_800A866C(fp);
        } else {
            ftCo_800A75DC(fp, (*data).x44);
        }
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_CpuInitNoTarget`
unsafe fn ftCo_CpuInitNoTarget(mut fp: *mut Fighter, mut fear_flag: i32) {
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut is_food: i32 = 0;
    (*data).set_xF8_b0((0 as u8) as u8);
    (*data).set_xF9_b2(({ is_food = 1; is_food } as u8) as u8);
    (*data).set_xF9_b4((1 as u8) as u8);
    (*data).set_xF9_b3((fear_flag as u8) as u8);
    (*data).set_xF9_b5((1 as u8) as u8);
    (*data).set_xF9_b6((fear_flag as u8) as u8);
    (*data).set_xF9_b7((1 as u8) as u8);
    (*data).set_xF9_b1((0 as u8) as u8);
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
}

/// `ftCo_CpuActOnNoTarget`
unsafe fn ftCo_CpuActOnNoTarget(mut fp: *mut Fighter, mut data: *mut CpuFighter) {
    let mut data2: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut attack_target: *mut Fighter = null_mut();
    let mut do_act: i32 = 0;
    data2 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ((*data2).x18 != (*data2).x20) && ((*data2).x18 != (*data2).x1C) {
        (*data2).x60 = 0;
    }
    if (*data2).x18 == 4 {
        do_act = 0;
    } else {
        (*data2).set_xFA_b2((0 as u8) as u8);
        do_act = 1;
    }
    if do_act != 0 {
        if !(*data).x4C.is_null() {
            ftCo_800A866C(fp);
        } else {
            attack_target = ftCo_800A53DC(fp);
            if attack_target.is_null() {
                attack_target = (*data).x44;
            }
            ftCo_800A75DC(fp, attack_target);
        }
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_800AEFB8`
pub unsafe fn ftCo_800AEFB8(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AEFB8");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut sp28: Vec3 = unsafe { core::mem::zeroed() };
    let mut cmd: i32 = 0;
    let mut target: *mut Fighter = null_mut();
    cmd = ftCo_800A229C(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec);
    if cmd != 0 {
        ftCo_800AE7AC(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec, cmd);
        return;
    }
    target = ftCo_800A5CE0(fp);
    (*data).x44 = target;
    if !target.is_null() {
        ftCo_CpuSetTargetFlags(data);
        ftCo_CpuTargetAct(fp);
        let _ = ftCo_800ADE48(fp);
        return;
    }
    ftCo_CpuInitNoTarget(fp, 1);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    ftCo_CpuActOnNoTarget(fp, data);
}

/// `ftCo_CpuUpdateTargetDistance`
unsafe fn ftCo_CpuUpdateTargetDistance(mut fp: *mut Fighter) {
    let mut data: *mut CpuFighter = null_mut();
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ftCo_800A1C44(fp) != 0 {
        (*data).set_xF8_b6((0 as u8) as u8);
    } else {
        if (!(*data).x44.is_null()) && ((*fp).ground_or_air == ((GA_Ground as i32) as u32)) {
            if ftCo_800A1AB4(fp, (*data).x44) < (*(*ext::Fighter_804D64FC).x20.offset(ftAi_Kind((*fp).kind as i32) as isize)) {
                (*data).set_xF8_b6((1 as u8) as u8);
            } else {
                (*data).set_xF8_b6((0 as u8) as u8);
            }
        } else {
            (*data).set_xF8_b6((0 as u8) as u8);
        }
    }
}

/// `ftCo_CpuUpdateFoodItemTarget`
unsafe fn ftCo_CpuUpdateFoodItemTarget(mut fp: *mut Fighter, mut is_food: *mut i32) {
    let mut item_gobj: *mut HSD_GObj = null_mut();
    let mut kind: ItemKind = 0;
    let mut data: *mut CpuFighter = null_mut();
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    item_gobj = (*fp).item_gobj;
    if !item_gobj.is_null() {
        kind = (*(HSD_GObjGetUserData(item_gobj) as *mut Item)).kind;
        if kind == It_Kind_Heart {
            *is_food = 1;
        } else if kind == It_Kind_Tomato {
            *is_food = 1;
        } else if kind == It_Kind_Foods {
            *is_food = 1;
        } else {
            *is_food = 0;
        }
        if (*is_food) == 0 {
            (*data).x4C = null_mut();
            return;
        }
    }
    if (*fp).x2168 != 0 {
        (*data).x4C = null_mut();
    } else {
        (*data).x4C = ftCo_800A5F4C(fp, It_Kind_L_Gun_Ray);
    }
}

/// `ftCo_CpuApproachFoodItemTarget`
unsafe fn ftCo_CpuApproachFoodItemTarget(mut fp: *mut Fighter, mut data: *mut CpuFighter, mut target_slot: *mut *mut Fighter, mut approach_pos: *mut Vec) {
    let mut nearby_fp: *mut Fighter = null_mut();
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut dist: f32 = 0.0;
    if ftCo_CpuShouldAct(fp) != 0 {
        if !(*data).x4C.is_null() {
            ftCo_800A866C(fp);
        } else {
            nearby_fp = *target_slot;
            if (!nearby_fp.is_null()) && ((*fp).ground_or_air != ((GA_Air as i32) as u32)) {
                dy = (*fp).cur_pos.y - (*nearby_fp).cur_pos.y;
                dx = (*fp).cur_pos.x - (*nearby_fp).cur_pos.x;
                dist = melee_runtime::gekko::sqrtf(fmadds(dx, dx, dy * dy));
                if (!((dist as f64) > 50.0f64)) && (ftCo_800A6700(fp, core::ptr::addr_of_mut!((*nearby_fp).cur_pos) as *mut Vec, approach_pos) != 0) {
                    ftCo_800A1F3C(fp, (*approach_pos).x, (*approach_pos).y, 5.0f32);
                }
            }
        }
    }
}

/// `ftCo_800AF290`
#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe fn ftCo_800AF290(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AF290");
    let mut sp54: Vec3 = unsafe { core::mem::zeroed() };
    let mut __: [u8; 24] = [0; 24];
    let mut sp30: Vec3 = unsafe { core::mem::zeroed() };
    let mut target: *mut Fighter = null_mut();
    let mut cmd: i32 = 0;
    let mut redirect: i32 = 0;
    let mut is_food: i32 = 0;
    let mut target_slot: *mut *mut Fighter = null_mut();
    let mut data: *mut CpuFighter = null_mut();
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    cmd = ftCo_800A229C(fp, core::ptr::addr_of_mut!(sp54) as *mut Vec);
    if cmd != 0 {
        ftCo_800AE7AC(fp, core::ptr::addr_of_mut!(sp54) as *mut Vec, cmd);
        return;
    }
    target = ftCo_800A5CE0(fp);
    (*data).x44 = target;
    if !target.is_null() {
        ftCo_CpuSetTargetFlags(data);
        ftCo_CpuTargetAct(fp);
        let _ = ftCo_800ADE48(fp);
        return;
    }
    if (*fp).x221D_b6() != 0 {
        redirect = 1;
    } else if ((*fp).x2168 != 0) && ((*fp).x2338.x == 0) {
        redirect = 1;
    } else {
        redirect = 0;
    }
    if redirect != 0 {
        ftCo_800AECF0(fp);
        return;
    }
    (*data).set_xF8_b0(({ is_food = 1; is_food } as u8) as u8);
    (*data).set_xF9_b2((1 as u8) as u8);
    (*data).set_xF9_b4((1 as u8) as u8);
    (*data).set_xF9_b3((0 as u8) as u8);
    (*data).set_xF9_b5((1 as u8) as u8);
    (*data).set_xF9_b6((1 as u8) as u8);
    (*data).set_xF9_b7((1 as u8) as u8);
    (*data).set_xF9_b1((0 as u8) as u8);
    target = ftCo_800A4BEC(fp);
    let __t117 = core::ptr::addr_of_mut!(*{ target_slot = core::ptr::addr_of_mut!((*fp).cpu.x44) as *mut *mut Fighter; target_slot }); *__t117 = target;
    ftCo_CpuUpdateFoodItemTarget(fp, core::ptr::addr_of_mut!(is_food) as *mut i32);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    ftCo_CpuUpdateTargetDistance(fp);
    target = (*data).x44;
    if !target.is_null() {
        if ftCo_800A1AB4(fp, target) > (*data).x40 {
            ftCo_800AEA8C(fp);
            return;
        }
    }
    ftCo_CpuApproachFoodItemTarget(fp, data, target_slot, core::ptr::addr_of_mut!(sp30) as *mut Vec);
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_800AF78C`
#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe fn ftCo_800AF78C(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AF78C");
    let mut sp4C: Vec3 = unsafe { core::mem::zeroed() };
    let mut __: [u8; 20] = [0; 20];
    let mut approach_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut target: *mut Fighter = null_mut();
    let mut cmd: i32 = 0;
    let mut should_escape: i32 = 0;
    let mut target_slot: *mut *mut Fighter = null_mut();
    let mut data: *mut CpuFighter = null_mut();
    let mut is_food: i32 = 0;
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    cmd = ftCo_800A229C(fp, core::ptr::addr_of_mut!(sp4C) as *mut Vec);
    if cmd != 0 {
        ftCo_800AE7AC(fp, core::ptr::addr_of_mut!(sp4C) as *mut Vec, cmd);
        return;
    }
    target = ftCo_800A5CE0(fp);
    (*data).x44 = target;
    if !target.is_null() {
        ftCo_CpuSetTargetFlags(data);
        ftCo_CpuTargetAct(fp);
        let _ = ftCo_800ADE48(fp);
        return;
    }
    if (*fp).x221D_b6() != 0 {
        should_escape = 1;
    } else if ((*fp).x2168 != 0) && ((*fp).x2338.x == 0) {
        should_escape = 1;
    } else {
        should_escape = 0;
    }
    if should_escape != 0 {
        ftCo_800AECF0(fp);
        return;
    }
    (*data).set_xF8_b0(({ is_food = 1; is_food } as u8) as u8);
    (*data).set_xF9_b2((1 as u8) as u8);
    (*data).set_xF9_b4((1 as u8) as u8);
    (*data).set_xF9_b3((0 as u8) as u8);
    (*data).set_xF9_b5((1 as u8) as u8);
    (*data).set_xF9_b6((1 as u8) as u8);
    (*data).set_xF9_b7((1 as u8) as u8);
    (*data).set_xF9_b1((0 as u8) as u8);
    let __t118 = core::ptr::addr_of_mut!(*{ target_slot = core::ptr::addr_of_mut!((*fp).cpu.x44) as *mut *mut Fighter; target_slot }); *__t118 = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateFoodItemTarget(fp, core::ptr::addr_of_mut!(is_food) as *mut i32);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    ftCo_CpuUpdateTargetDistance(fp);
    ftCo_CpuApproachFoodItemTarget(fp, data, target_slot, core::ptr::addr_of_mut!(approach_pos) as *mut Vec);
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_800AFC40`
pub unsafe fn ftCo_800AFC40(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800AFC40");
    let mut temp_r31: *mut CpuFighter = null_mut();
    let mut temp_r3_3: *mut Fighter = null_mut();
    let mut var_r0: i32 = 0;
    let mut temp_r4: *mut CpuFighter = null_mut();
    temp_r31 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if (*fp).x221D_b6() != 0 {
        var_r0 = 1;
    } else if ((*fp).x2168 != 0) && ((*fp).x2338.x == 0) {
        var_r0 = 1;
    } else {
        var_r0 = 0;
    }
    if var_r0 != 0 {
        ftCo_800AECF0(fp);
        return;
    }
    (*temp_r31).set_xF8_b0((1 as u8) as u8);
    (*temp_r31).set_xF9_b2((1 as u8) as u8);
    (*temp_r31).set_xF9_b4((1 as u8) as u8);
    (*temp_r31).set_xF9_b3((1 as u8) as u8);
    (*temp_r31).set_xF9_b5((1 as u8) as u8);
    (*temp_r31).set_xF9_b6((1 as u8) as u8);
    (*temp_r31).set_xF9_b7((0 as u8) as u8);
    (*temp_r31).set_xF9_b1((0 as u8) as u8);
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    temp_r4 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ftCo_CpuDataShouldAct(temp_r4) != 0 {
        temp_r3_3 = ftCo_800A50D4(fp);
        if !temp_r3_3.is_null() {
            ftCo_800A75DC(fp, temp_r3_3);
        } else {
            ftCo_800A75DC(fp, (*temp_r31).x44);
        }
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_CpuActOnPlayer`
unsafe fn ftCo_CpuActOnPlayer(mut fp: *mut Fighter, mut data: *mut CpuFighter, mut player_id: i32) {
    let mut target: *mut Fighter = ftCo_800A5294(fp, player_id);
    if !target.is_null() {
        ftCo_800A75DC(fp, target);
    } else {
        ftCo_800A75DC(fp, (*data).x44);
    }
}

/// `ftCo_800AFE3C`
pub unsafe fn ftCo_800AFE3C(mut fp: *mut Fighter, mut arg1: i32) {
    melee_core::ct!("ftCo_800AFE3C");
    let mut temp_r31: *mut CpuFighter = null_mut();
    let mut sp2C: Vec3 = unsafe { core::mem::zeroed() };
    let mut temp_r3_2: *mut Fighter = null_mut();
    let mut temp_r3: i32 = 0;
    temp_r31 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    temp_r3 = ftCo_800A229C(fp, core::ptr::addr_of_mut!(sp2C) as *mut Vec);
    if temp_r3 != 0 {
        ftCo_800AE7AC(fp, core::ptr::addr_of_mut!(sp2C) as *mut Vec, temp_r3);
        return;
    }
    temp_r3_2 = ftCo_800A5CE0(fp);
    (*temp_r31).x44 = temp_r3_2;
    if !temp_r3_2.is_null() {
        ftCo_CpuSetTargetFlags(temp_r31);
        ftCo_CpuTargetAct(fp);
        let _ = ftCo_800ADE48(fp);
        return;
    }
    (*temp_r31).set_xF8_b0((0 as u8) as u8);
    (*temp_r31).set_xF9_b2((1 as u8) as u8);
    (*temp_r31).set_xF9_b4((1 as u8) as u8);
    (*temp_r31).set_xF9_b3((1 as u8) as u8);
    (*temp_r31).set_xF9_b5((1 as u8) as u8);
    (*temp_r31).set_xF9_b6((1 as u8) as u8);
    (*temp_r31).set_xF9_b7((1 as u8) as u8);
    (*temp_r31).set_xF9_b1((0 as u8) as u8);
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    if ftCo_CpuShouldAct(fp) != 0 {
        ftCo_CpuActOnPlayer(fp, temp_r31, arg1);
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_800B00F8`
pub unsafe fn ftCo_800B00F8(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B00F8");
    let mut sp28: Vec3 = unsafe { core::mem::zeroed() };
    let mut temp_r3: *mut Fighter = null_mut();
    let mut temp_r3_4: i32 = 0;
    let mut var_r0_2: i32 = 0;
    let mut var_r0_3: i32 = 0;
    let mut var_r0_10: i32 = 0;
    let mut var_r0_4: i32 = 0;
    let mut var_r0_5: i32 = 0;
    let mut var_r0_6: i32 = 0;
    let mut var_r0_7: i32 = 0;
    let mut var_r0_8: i32 = 0;
    let mut var_r0_9: i32 = 0;
    let mut temp_r31: *mut CpuFighter = null_mut();
    let mut temp_r4: *mut CpuFighter = null_mut();
    let mut tmp: i32 = 0;
    temp_r31 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    tmp = ftCo_800A229C(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec);
    if tmp != 0 {
        ftCo_800AE7AC(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec, tmp);
        return;
    }
    temp_r3 = ftCo_800A5CE0(fp);
    (*temp_r31).x44 = temp_r3;
    if !temp_r3.is_null() {
        ftCo_CpuSetTargetFlags(temp_r31);
        ftCo_CpuTargetAct(fp);
        let _ = ftCo_800ADE48(fp);
        return;
    }
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
    if (!(*fp).item_gobj.is_null()) && (!(ftCo_800A5908(HSD_GObjGetUserData((*fp).item_gobj) as *mut Item) != 0)) {
        (*temp_r31).x4C = null_mut();
    } else {
        if (*fp).x2168 != 0 {
            (*temp_r31).x4C = null_mut();
        } else {
            (*temp_r31).x4C = ftCo_800A61D8(fp);
        }
    }
    ftCo_CpuUpdateSpecialItemTarget(fp);
    temp_r4 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    temp_r3_4 = (*fp).cpu.x18;
    if (temp_r3_4 != (*fp).cpu.x20) && (temp_r3_4 != (*temp_r4).x1C) {
        (*temp_r4).x60 = 0;
    }
    if (*temp_r4).x18 == 4 {
        var_r0_3 = 0;
    } else {
        (*temp_r4).set_xFA_b2((0 as u8) as u8);
        var_r0_3 = 1;
    }
    if var_r0_3 != 0 {
        if !(*temp_r31).x4C.is_null() {
            ftCo_800A866C(fp);
        } else if (((*temp_r31).x7C % 60) * 5) == 0 {
            if (ext::HSD_Randf() as f64) < 0.5f64 {
                ftCo_800A8940(fp);
            }
            if (ext::HSD_Randf() as f64) > 0.5f64 {
                var_r0_4 = 1;
            } else {
                var_r0_4 = 0;
            }
            (*temp_r31).set_xF8_b0((var_r0_4 as u8) as u8);
            if (ext::HSD_Randf() as f64) > 0.5f64 {
                var_r0_5 = 1;
            } else {
                var_r0_5 = 0;
            }
            (*temp_r31).set_xF9_b2((var_r0_5 as u8) as u8);
            if (ext::HSD_Randf() as f64) > 0.5f64 {
                var_r0_6 = 1;
            } else {
                var_r0_6 = 0;
            }
            (*temp_r31).set_xF9_b4((var_r0_6 as u8) as u8);
            if (ext::HSD_Randf() as f64) > 0.5f64 {
                var_r0_7 = 1;
            } else {
                var_r0_7 = 0;
            }
            (*temp_r31).set_xF9_b3((var_r0_7 as u8) as u8);
            if (ext::HSD_Randf() as f64) > 0.5f64 {
                var_r0_8 = 1;
            } else {
                var_r0_8 = 0;
            }
            (*temp_r31).set_xF9_b5((var_r0_8 as u8) as u8);
            if (ext::HSD_Randf() as f64) > 0.5f64 {
                var_r0_9 = 1;
            } else {
                var_r0_9 = 0;
            }
            (*temp_r31).set_xF9_b6((var_r0_9 as u8) as u8);
            if (ext::HSD_Randf() as f64) > 0.5f64 {
                var_r0_10 = 1;
            } else {
                var_r0_10 = 0;
            }
            (*temp_r31).set_xF9_b7((var_r0_10 as u8) as u8);
            (*temp_r31).set_xF9_b1((0 as u8) as u8);
        }
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_800B04DC`
pub unsafe fn ftCo_800B04DC(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B04DC");
    let mut target_slot: *mut *mut Fighter = null_mut();
    let mut data: *mut CpuFighter = null_mut();
    let mut item_gobj: *mut HSD_GObj = null_mut();
    let mut kind: ItemKind = 0;
    let mut target: *mut Fighter = null_mut();
    let mut dist: [f32; 1] = [0.0; 1];
    let mut is_food: i32 = 0;
    let mut __: [u8; 8] = [0; 8];
    let mut approach_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut unused: f32 = 0.0;
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    (*data).set_xF8_b0(({ is_food = 1; is_food } as u8) as u8);
    ftCo_CpuClearTargetModes(data);
    (*data).set_xF9_b1((1 as u8) as u8);
    target = ftCo_800A4BEC(fp);
    let __t119 = core::ptr::addr_of_mut!(*{ target_slot = core::ptr::addr_of_mut!((*fp).cpu.x44) as *mut *mut Fighter; target_slot }); *__t119 = target;
    let mut mfi: i32 = 0;
    item_gobj = (*fp).item_gobj;
    if !item_gobj.is_null() {
        kind = (*(HSD_GObjGetUserData(item_gobj) as *mut Item)).kind;
        if kind == It_Kind_Heart {
            is_food = 1;
        } else if kind == It_Kind_Tomato {
            is_food = 1;
        } else if kind == It_Kind_Foods {
            is_food = 1;
        } else {
            is_food = 0;
        }
        if is_food == 0 {
            (*data).x4C = null_mut();
        } else {
            mfi = 1;
        }
    } else {
        mfi = 1;
    }
    if mfi != 0 {
        if (*fp).x2168 != 0 {
            (*data).x4C = null_mut();
        } else {
            (*data).x4C = ftCo_800A61D8(fp);
        }
    }
    ftCo_CpuUpdateSpecialItemTarget(fp);
    if ftCo_CpuShouldAct(fp) != 0 {
        target = *target_slot;
        if (!target.is_null()) && ((*fp).ground_or_air != ((GA_Air as i32) as u32)) {
            let mut dy: f32 = (*fp).cur_pos.y - (*target).cur_pos.y;
            let mut dx: f32 = (*fp).cur_pos.x - (*target).cur_pos.x;
            dist[0 as usize] = melee_runtime::gekko::sqrtf(fmadds(dx, dx, dy * dy));
            if (!((dist[0 as usize] as f64) > 50.0f64)) && (ftCo_800A6700(fp, core::ptr::addr_of_mut!((*target).cur_pos) as *mut Vec, core::ptr::addr_of_mut!(approach_pos) as *mut Vec) != 0) {
                ftCo_800A1F3C(fp, approach_pos.x, approach_pos.y, 5.0f32);
            }
        }
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_CpuInitEnemyTarget`
unsafe fn ftCo_CpuInitEnemyTarget(mut fp: *mut Fighter, mut data: *mut CpuFighter) {
    (*data).set_xF8_b0((0 as u8) as u8);
    (*data).set_xF9_b2((1 as u8) as u8);
    (*data).set_xF9_b4((1 as u8) as u8);
    (*data).set_xF9_b3((0 as u8) as u8);
    (*data).set_xF9_b5((0 as u8) as u8);
    (*data).set_xF9_b6((0 as u8) as u8);
    (*data).set_xF9_b7((0 as u8) as u8);
    (*data).set_xF9_b1((0 as u8) as u8);
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
}

/// `inlineI3`
unsafe fn inlineI3(mut fp: *mut Fighter, mut data: *mut CpuFighter) {
    let mut fp_data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ftCo_CpuDataShouldAct(fp_data) != 0 {
        if (*data).xFB_b0() != 0 {
            ftCo_800A8DE4(fp);
        } else {
            ftCo_800A7AAC(fp);
        }
    }
}

/// `ftCo_800B0760`
pub unsafe fn ftCo_800B0760(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B0760");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    (*data).set_xF8_b0((1 as u8) as u8);
    (*data).set_xF9_b2((1 as u8) as u8);
    (*data).set_xF9_b4((1 as u8) as u8);
    (*data).set_xF9_b3((0 as u8) as u8);
    (*data).set_xF9_b5((0 as u8) as u8);
    (*data).set_xF9_b6((0 as u8) as u8);
    (*data).set_xF9_b7((0 as u8) as u8);
    (*data).set_xF9_b1((0 as u8) as u8);
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    inlineI3(fp, data);
    let _ = ftCo_800ADE48(fp);
}

/// `inlineM0`
unsafe fn inlineM0(mut x: f32) -> u8 {
    if x >= (0 as f32) {
        return melee_runtime::cabi::f2u8((127.0f32 * x) as f64);
    } else {
        return melee_runtime::cabi::f2u8((128.0f32 * x) as f64);
    }
}

/// `ftCo_800B0918`
pub unsafe fn ftCo_800B0918(mut fp0: *mut Fighter, mut fp1: *mut Fighter) {
    melee_core::ct!("ftCo_800B0918");
    let mut data: *mut CpuFighter = null_mut();
    data = core::ptr::addr_of_mut!((*fp1).cpu) as *mut CpuFighter;
    (*data).x444 = (*data).x444.offset(1);
    (*data).x448 = (*data).x448.offset(1);
    if (*data).x444 == (((*data).xFC.as_mut_ptr()).offset(30_u64 as isize)) {
        (*data).x444 = (*data).xFC.as_mut_ptr();
    }
    if (*data).x448 == (((*data).xFC.as_mut_ptr()).offset(30_u64 as isize)) {
        (*data).x448 = (*data).xFC.as_mut_ptr();
    }
    (*(*data).x444).lstick.x = inlineM0((*fp0).input.lstick[0 as usize].x) as i8;
    (*(*data).x444).lstick.y = inlineM0((*fp0).input.lstick[0 as usize].y) as i8;
    (*(*data).x444).cstick.x = inlineM0((*fp0).input.cstick[0 as usize].x) as i8;
    (*(*data).x444).cstick.y = inlineM0((*fp0).input.cstick[0 as usize].y) as i8;
    (*(*data).x444).x4 = melee_runtime::cabi::f2u8((*fp0).input.triggers[0 as usize] as f64);
    (*(*data).x444).x5 = melee_runtime::cabi::f2u8((*fp0).input.triggers[0 as usize] as f64);
    (*(*data).x444).x0 = (*fp0).input.held_buttons[0 as usize];
    (*(*data).x444).cur_pos = (*fp0).cur_pos;
    (*(*data).x444).facing_dir = (*fp0).facing_dir;
}

/// `inlineJ0`
unsafe fn inlineJ0(mut fp: *mut Fighter, mut nana_fp: *mut Fighter) -> i32 {
    if ((*fp).motion_id == ftCo_MS_FireFlowerShoot) || ((*fp).motion_id == ftCo_MS_FireFlowerShootAir) {
        return 0;
    }
    if ((*nana_fp).motion_id == (ftPp_MS_SpecialLw as i32)) && ((*fp).motion_id < ftCo_MS_Count) {
        return 1;
    } else {
        return 0;
    }
}

/// `ftCo_800B0AF4`
pub unsafe fn ftCo_800B0AF4(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B0AF4");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut nana_fp: *mut Fighter = ftCo_800A589C(fp);
    if (!nana_fp.is_null()) && (((*data).xFA_b7() as i32) != 0) {
        if inlineJ0(fp, nana_fp) != 0 {
            (*fp).facing_dir = -(*nana_fp).facing_dir;
            ext::ftPp_SpecialLw_Enter((*fp).gobj);
            return;
        }
        (*data).lstick.x = (*(*data).x448).lstick.x;
        (*data).lstick.y = (*(*data).x448).lstick.y;
        (*data).cstick.x = (*(*data).x448).cstick.x;
        (*data).cstick.y = (*(*data).x448).cstick.y;
        (*data).buttons = (*(*data).x448).x0;
        (*data).ltrigger = (*(*data).x448).x4;
        (*data).rtrigger = (*(*data).x448).x5;
        if (*fp).x2225_b3() != 0 {
            (*fp).cur_pos.x = (0.95f64).mul_add((*fp).cur_pos.x as f64, 0.05f64 * ((*(*data).x448).cur_pos.x as f64)) as f32;
            (*fp).cur_pos.y = (0.95f64).mul_add((*fp).cur_pos.y as f64, 0.05f64 * ((*(*data).x448).cur_pos.y as f64)) as f32;
            if ((*fp).motion_id != (ftPp_MS_SpecialLw as i32)) && ((*fp).motion_id != (ftPp_MS_SpecialAirLw as i32)) {
                (*fp).facing_dir = (*(*data).x448).facing_dir;
            }
        }
    }
}

/// `isInCaptureWait`
unsafe fn isInCaptureWait(mut fp: *mut Fighter) -> i32 {
    if (*fp).motion_id == ftCo_MS_CaptureWaitHi {
        return 1;
    } else if (*fp).motion_id == ftCo_MS_CaptureWaitLw {
        return 1;
    } else if ((*fp).motion_id >= ftCo_MS_ShoulderedWait) && ((*fp).motion_id <= ftCo_MS_ShoulderedTurn) {
        return 1;
    }
    return 0;
}

/// `ftCo_800B0CA8`
pub unsafe fn ftCo_800B0CA8(mut fp0: *mut Fighter, mut fp1: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800B0CA8");
    let mut temp_r5_2: *mut HSD_GObj = null_mut();
    let mut temp_r0: i32 = 0;
    let mut temp_r5: i32 = 0;
    let mut var_r0_4: i32 = 0;
    temp_r5 = (*fp1).motion_id;
    if ((ftCo_800A3200(fp1) != 0) || (isInCaptureWait(fp1) != 0)) || (isInTeeter(fp1) != 0) {
        return 0;
    }
    if (temp_r5 >= ftCo_MS_CaptureKirby) && (temp_r5 <= ftCo_MS_CaptureWaitKirby) {
        return 0;
    }
    if (temp_r5 >= ftCo_MS_Rebirth) && (temp_r5 <= ftCo_MS_RebirthWait) {
        return 0;
    }
    if (temp_r5 >= ftCo_MS_WarpStarJump) && (temp_r5 <= ftCo_MS_WarpStarFall) {
        return 0;
    }
    if (temp_r5 >= ftCo_MS_ItemParasolFall) && (temp_r5 <= ftCo_MS_ItemParasolDamageFall) {
        return 0;
    }
    temp_r5_2 = (*fp0).item_gobj;
    if !temp_r5_2.is_null() {
        temp_r0 = (*(HSD_GObjGetUserData(temp_r5_2) as *mut Item)).kind;
        's120: {
            match temp_r0 {
                1 | 2 | 4 | 5 => {
                    var_r0_4 = 1;
                }
                _ => {
                    var_r0_4 = 0;
                }
            }
        }
        if var_r0_4 != 0 {
            return 0;
        }
    }
    if ((((*fp1).ground_or_air == ((GA_Ground as i32) as u32)) && ((*fp0).ground_or_air == ((GA_Air as i32) as u32))) && ((*fp1).cur_pos.y > (*fp0).cur_pos.y)) && ((*fp0).pos_delta.y < 0.0f32) {
        return 0;
    }
    if (*fp0).x2225_b3() != 0 {
        return 1;
    }
    's121: {
        match (*fp0).motion_id {
            39 | 40 | 42 | 43 | 70 | 71 | 72 | 73 | 74 => {
                return 1;
            }
            _ => {}
        }
    }
    return 0;
}

/// `inlineK0`
unsafe fn inlineK0(mut fp0: *mut Fighter) -> i32 {
    if ((*fp0).motion_id >= 361) && ((*fp0).motion_id <= 366) {
        return 1;
    } else {
        return 0;
    }
}

/// `ftCo_800B0E98`
pub unsafe fn ftCo_800B0E98(mut fp0: *mut Fighter, mut fp1: *mut Fighter) -> i32 {
    melee_core::ct!("ftCo_800B0E98");
    let mut temp_r0: i32 = 0;
    if ftCo_800B0CA8(fp0, fp1) != 0 {
        let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp0).cpu) as *mut CpuFighter;
        if (*fp1).ground_or_air == ((GA_Air as i32) as u32) {
            return 0;
        }
        if (*fp0).ground_or_air == ((GA_Air as i32) as u32) {
            return 0;
        }
        if (((*data).xFB_b0() as i32) != 0) || (inlineK0(fp0) != 0) {
            return 0;
        }
        if (!(ftCo_CpuShouldAct(fp0) != 0)) && ((*fp0).motion_id != ftCo_MS_Wait) {
            return 0;
        }
        {
            let mut x: f32 = (*fp0).pos_delta.x - (*fp1).pos_delta.x;
            let mut y: f32 = (*fp0).pos_delta.y - (*fp1).pos_delta.y;
            let mut r: f32 = (*fp0).co_attrs.mid_walk_point;
            if fmadds(x, x, y * y) > (r * r) {
                return 0;
            }
        }
        {
            let mut x: f32 = (*fp0).cur_pos.x - (*fp1).cur_pos.x;
            let mut y: f32 = (*fp1).cur_pos.y - (*fp0).cur_pos.y;
            if (fmadds(x, x, y * y) as f64) < (25.0f64 * 25.0f64) {
                return 1;
            }
        }
    }
    return 0;
}

/// `isPopoUpB`
unsafe fn isPopoUpB(mut fp: *mut Fighter) -> i32 {
    if ((*fp).motion_id >= (ftPp_MS_SpecialHi_0 as i32)) && ((*fp).motion_id <= (ftPp_MS_SpecialHi_5 as i32)) {
        return 1;
    }
    return 0;
}

/// `ftCo_800B101C_inline0`
unsafe fn ftCo_800B101C_inline0(mut fp: *mut Fighter, mut other: *mut Fighter) -> i32 {
    let mut temp_r4: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if isPopoUpB(fp) != 0 {
        (*temp_r4).set_xFB_b0((1 as u8) as u8);
        return 1;
    } else {
        let mut dx: f32 = (*fp).cur_pos.x - (*other).cur_pos.x;
        let mut dy: f32 = (*other).cur_pos.y - (*fp).cur_pos.y;
        if (fmadds(dx, dx, dy * dy) as f64) > (25.0f64 * 25.0f64) {
            return 1;
        } else if ftCo_800B0CA8(fp, other) != 0 {
            return 0;
        } else {
            return 1;
        }
    }
}

/// `ftCo_CpuUpdateNanaFollow`
unsafe fn ftCo_CpuUpdateNanaFollow(mut fp: *mut Fighter, mut popo: *mut Fighter, mut data: *mut CpuFighter) {
    ftCo_800B0760(fp);
    if ftCo_800B0E98(fp, popo) != 0 {
        (*data).x18 = 0;
        (*data).set_xFA_b7((1 as u8) as u8);
        (*data).set_xF9_b2((0 as u8) as u8);
        (*data).set_xF9_b4((0 as u8) as u8);
    }
}

/// `ftCo_800B101C`
pub unsafe fn ftCo_800B101C(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B101C");
    let mut var_r4: *mut HSD_GObj = null_mut();
    let mut temp_f0: f32 = 0.0;
    let mut temp_f2: f32 = 0.0;
    let mut temp_f3: f32 = 0.0;
    let mut var_r0: i32 = 0;
    let mut var_r0_2: i32 = 0;
    let mut var_r3: i32 = 0;
    let mut temp_r31: *mut CpuFighter = null_mut();
    let mut var_r29: *mut Fighter = null_mut();
    temp_r31 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    (*temp_r31).set_xF9_b2((1 as u8) as u8);
    (*temp_r31).set_xF9_b4((1 as u8) as u8);
    var_r29 = ftCo_800A589C(fp);
    if var_r29.is_null() {
        ftCo_800AEFB8(fp);
        return;
    }
    ftCo_800B0918(var_r29, fp);
    temp_f0 = (*var_r29).dmg.x1830_percent / 2e+01f32;
    var_r0 = melee_core::ppc::fctiwz_f32(temp_f0);
    if (melee_core::ppc::fctiwz_f32(temp_f0)) > 9 {
        var_r0 = 9;
    }
    (*temp_r31).level = var_r0;
    if (*temp_r31).xFA_b7() != 0 {
        ftCo_800A7AAC(fp);
        if ftCo_800B101C_inline0(fp, var_r29) != 0 {
            (*temp_r31).set_xFA_b7((0 as u8) as u8);
            (*temp_r31).x18 = (*temp_r31).x1C;
            (*fp).cpu.lstick.x = 0 as i8;
            (*fp).cpu.lstick.y = 0 as i8;
            (*fp).cpu.buttons = 0 as u32;
            (*fp).cpu.ltrigger = 0 as u8;
            (*fp).cpu.rtrigger = 0 as u8;
        }
    } else {
        ftCo_CpuUpdateNanaFollow(fp, var_r29, temp_r31);
    }
    if (*fp).ground_or_air == ((GA_Ground as i32) as u32) {
        (*temp_r31).set_xFB_b0((0 as u8) as u8);
    }
}

/// `ftCo_800B126C`
pub unsafe fn ftCo_800B126C(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B126C");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    (*data).set_xF8_b0((1 as u8) as u8);
    (*data).set_xF9_b2((0 as u8) as u8);
    (*data).set_xF9_b4((0 as u8) as u8);
    (*data).set_xF9_b3((0 as u8) as u8);
    (*data).set_xF9_b5((1 as u8) as u8);
    (*data).set_xF9_b6((1 as u8) as u8);
    (*data).set_xF9_b7((1 as u8) as u8);
    (*data).x2C = 1;
    (*data).set_xF9_b1((0 as u8) as u8);
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    if ftCo_CpuShouldAct(fp) != 0 {
        if (!(*data).x4C.is_null()) && ((*fp).item_gobj.is_null()) {
            ftCo_800A866C(fp);
        } else if ((((*data).x7C % 60) * 5) == 0) && ((ext::HSD_Randf() as f64) < 0.5f64) {
            ftCo_800A8940(fp);
        }
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_800B1478`
pub unsafe fn ftCo_800B1478(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B1478");
    let mut data: *mut CpuFighter = null_mut();
    let mut target_slot: *mut *mut Fighter = null_mut();
    let mut target: *mut Fighter = null_mut();
    let mut attack_target: *mut Fighter = null_mut();
    let mut is_food: i32 = 0;
    data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    target = ftCo_800A5CE0(fp);
    let __t122 = core::ptr::addr_of_mut!(*{ target_slot = core::ptr::addr_of_mut!((*fp).cpu.x44) as *mut *mut Fighter; target_slot }); *__t122 = target;
    if !target.is_null() {
        ftCo_CpuSetTargetFlags(data);
        ftCo_CpuTargetAct(fp);
        let _ = ftCo_800ADE48(fp);
        return;
    }
    (*data).set_xF8_b0((1 as u8) as u8);
    (*data).set_xF9_b2(({ is_food = 1; is_food } as u8) as u8);
    (*data).set_xF9_b4((1 as u8) as u8);
    (*data).set_xF9_b3((1 as u8) as u8);
    (*data).set_xF9_b5((1 as u8) as u8);
    (*data).set_xF9_b6((1 as u8) as u8);
    (*data).set_xF9_b7((1 as u8) as u8);
    (*data).set_xF9_b1((0 as u8) as u8);
    *target_slot = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    ftCo_CpuUpdateTargetDistance(fp);
    if ftCo_CpuShouldAct(fp) != 0 {
        if !(*data).x4C.is_null() {
            ftCo_800A866C(fp);
        } else {
            attack_target = ftCo_800A53DC(fp);
            if attack_target.is_null() {
                attack_target = (*data).x44;
            }
            ftCo_800A75DC(fp, attack_target);
        }
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_800B17D0`
pub unsafe fn ftCo_800B17D0(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B17D0");
    let mut data2: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut sp28: Vec3 = unsafe { core::mem::zeroed() };
    let mut cmd: i32 = 0;
    let mut target: *mut Fighter = null_mut();
    let mut attack_target: *mut Fighter = null_mut();
    let mut do_act: i32 = 0;
    let mut is_food: i32 = 0;
    cmd = ftCo_800A229C(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec);
    if cmd != 0 {
        ftCo_800AE7AC(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec, cmd);
        return;
    }
    target = ftCo_800A5CE0(fp);
    (*data).x44 = target;
    if !target.is_null() {
        ftCo_CpuSetTargetFlags(data);
        ftCo_CpuTargetAct(fp);
        let _ = ftCo_800ADE48(fp);
        return;
    }
    (*data).set_xF8_b0((0 as u8) as u8);
    (*data).set_xF9_b2(({ is_food = 1; is_food } as u8) as u8);
    (*data).set_xF9_b4((1 as u8) as u8);
    (*data).set_xF9_b3((1 as u8) as u8);
    (*data).set_xF9_b5((1 as u8) as u8);
    (*data).set_xF9_b6((1 as u8) as u8);
    (*data).set_xF9_b7((1 as u8) as u8);
    (*data).x2C = is_food;
    (*data).set_xF9_b1((0 as u8) as u8);
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    data2 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    do_act = ftCo_CpuDataShouldAct(data2);
    if do_act != 0 {
        if (!(*data).x4C.is_null()) && ((*fp).item_gobj.is_null()) {
            ftCo_800A866C(fp);
        } else {
            attack_target = ftCo_800A53DC(fp);
            if attack_target.is_null() {
                attack_target = (*data).x44;
            }
            ftCo_800A75DC(fp, attack_target);
        }
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_800B1AB8`
pub unsafe fn ftCo_800B1AB8(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B1AB8");
    let mut data2: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut sp28: Vec3 = unsafe { core::mem::zeroed() };
    let mut cmd: i32 = 0;
    let mut target: *mut Fighter = null_mut();
    let mut attack_target: *mut Fighter = null_mut();
    let mut do_act: i32 = 0;
    let mut is_food: i32 = 0;
    cmd = ftCo_800A229C(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec);
    if cmd != 0 {
        ftCo_800AE7AC(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec, cmd);
        return;
    }
    target = ftCo_800A5CE0(fp);
    (*data).x44 = target;
    if !target.is_null() {
        ftCo_CpuSetTargetFlags(data);
        ftCo_CpuTargetAct(fp);
        let _ = ftCo_800ADE48(fp);
        return;
    }
    (*data).set_xF8_b0((0 as u8) as u8);
    (*data).set_xF9_b2(({ is_food = 1; is_food } as u8) as u8);
    (*data).set_xF9_b4((1 as u8) as u8);
    (*data).set_xF9_b3((1 as u8) as u8);
    (*data).set_xF9_b5((0 as u8) as u8);
    (*data).set_xF9_b6((1 as u8) as u8);
    (*data).set_xF9_b7((1 as u8) as u8);
    (*data).x2C = is_food;
    (*data).set_xF9_b1((0 as u8) as u8);
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    data2 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    do_act = ftCo_CpuDataShouldAct(data2);
    if do_act != 0 {
        if (!(*data).x4C.is_null()) && ((*fp).item_gobj.is_null()) {
            ftCo_800A866C(fp);
        } else {
            attack_target = ftCo_800A53DC(fp);
            if attack_target.is_null() {
                attack_target = (*data).x44;
            }
            ftCo_800A75DC(fp, attack_target);
        }
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_800B1DA0`
pub unsafe fn ftCo_800B1DA0(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B1DA0");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    ftCo_CpuInitEnemyTarget(fp, data);
    if ((ftCo_CpuDataShouldAct(data) != 0) && ((((*data).x7C % 60) * 5) == 0)) && ((ext::HSD_Randf() as f64) < 0.5f64) {
        ftCo_800A8940(fp);
    }
    let _ = ftCo_800ADE48(fp);
}

/// `ftCo_800B1DA0_noinline`
unsafe fn ftCo_800B1DA0_noinline(mut fp: *mut Fighter) {
    ftCo_800B1DA0(fp);
}

/// `ftCo_800B1DA0_noinline2`
unsafe fn ftCo_800B1DA0_noinline2(mut fp: *mut Fighter) {
    ftCo_800B1DA0_noinline(fp);
}

/// `ftCo_800B1EF0`
pub unsafe fn ftCo_800B1EF0(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B1EF0");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut sp28: Vec3 = unsafe { core::mem::zeroed() };
    let mut cmd: i32 = 0;
    let mut target: *mut Fighter = null_mut();
    cmd = ftCo_800A229C(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec);
    if cmd != 0 {
        ftCo_800AE7AC(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec, cmd);
        return;
    }
    target = ftCo_800A5CE0(fp);
    (*data).x44 = target;
    if !target.is_null() {
        ftCo_CpuSetTargetFlags(data);
        ftCo_CpuTargetAct(fp);
        let _ = ftCo_800ADE48(fp);
        return;
    }
    ftCo_CpuInitNoTarget(fp, 0);
    ftCo_CpuUpdateCommonItemTarget(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    ftCo_CpuActOnNoTarget(fp, data);
}

/// `ftCo_800B21C8`
pub unsafe fn ftCo_800B21C8(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B21C8");
    let mut data2: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut sp20: Vec3 = unsafe { core::mem::zeroed() };
    let mut cmd: i32 = 0;
    let mut target: *mut Fighter = null_mut();
    let mut attack_target: *mut Fighter = null_mut();
    let mut do_act: i32 = 0;
    cmd = ftCo_800A229C(fp, core::ptr::addr_of_mut!(sp20) as *mut Vec);
    if cmd != 0 {
        ftCo_800AE7AC(fp, core::ptr::addr_of_mut!(sp20) as *mut Vec, cmd);
        return;
    }
    target = ftCo_800A5CE0(fp);
    (*data).x44 = target;
    if !target.is_null() {
        ftCo_CpuSetTargetFlags(data);
        ftCo_CpuTargetAct(fp);
        let _ = ftCo_800ADE48(fp);
        return;
    }
    (*data).set_xF8_b0((0 as u8) as u8);
    (*data).set_xF9_b2((1 as u8) as u8);
    (*data).set_xF9_b4((1 as u8) as u8);
    (*data).set_xF9_b3((1 as u8) as u8);
    (*data).set_xF9_b5((1 as u8) as u8);
    (*data).set_xF9_b6((1 as u8) as u8);
    (*data).set_xF9_b7((0 as u8) as u8);
    (*data).set_xF9_b1((0 as u8) as u8);
    (*fp).cpu.x44 = ftCo_800A4BEC(fp);
    ftCo_CpuUpdateSpecialItemTarget(fp);
    ftCo_CpuUpdateTargetDistance(fp);
    data2 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    do_act = ftCo_CpuDataShouldAct(data2);
    if do_act != 0 {
        attack_target = ftCo_800A53DC(fp);
        if attack_target.is_null() {
            attack_target = (*data).x44;
        }
        ftCo_800A75DC(fp, attack_target);
    }
    // r31 is 0 here (`li r31, 0` at 0x800B22F8, used for the bit clears above)
    let _ = ftCo_800ADE48_r31(fp, false);
}

/// `ftCo_800B24B8`
pub unsafe fn ftCo_800B24B8(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B24B8");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut sp28: Vec3 = unsafe { core::mem::zeroed() };
    let mut cmd: i32 = 0;
    let mut target: *mut Fighter = null_mut();
    macro_rules! st { ($n:expr) => { if std::env::var_os("MELEE_DBG_CPU").is_some() { eprintln!("    B24B8 {} x38 {:08X}", $n, (*fp).cpu.x38.to_bits()); } } }
    st!("entry");
    cmd = ftCo_800A229C(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec);
    st!("229C");
    if cmd != 0 {
        st!("cmd!=0");
        ftCo_800AE7AC(fp, core::ptr::addr_of_mut!(sp28) as *mut Vec, cmd);
        st!("AE7AC");
        return;
    }
    target = ftCo_800A5CE0(fp);
    (*data).x44 = target;
    if !target.is_null() {
        ftCo_CpuSetTargetFlags(data);
        st!("flags");
        ftCo_CpuTargetAct(fp);
        st!("TargetAct");
        let _ = ftCo_800ADE48(fp);
        st!("ADE48");
        return;
    }
    st!("notarget");
    ftCo_CpuInitNoTarget(fp, 1);
    st!("InitNoTarget");
    ftCo_CpuUpdateCommonItemTarget(fp);
    st!("common");
    ftCo_CpuUpdateSpecialItemTarget(fp);
    st!("special");
    ftCo_CpuActOnNoTarget(fp, data);
    st!("ActOnNoTarget");
}

/// `ftCo_800B2790`
pub unsafe fn ftCo_800B2790(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B2790");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut cmd: i32 = 0;
    let mut line_id: i32 = 0;
    let mut on_ground: i32 = 0;
    let mut special_floor: i32 = 0;
    let mut b15: i32 = 0;
    if ((*data).csP.is_null()) && ((*data).command_duration == (0 as u32)) {
        {
            let mut fp2: *mut Fighter = fp;
            (*data).x80 = (*data).x80 + 1;
            (*data).set_xF8_b7((({ special_floor = 0; special_floor } & 1) as u8) as u8);
            super::ftcmdscript::ftCo_800B462C(fp2);
        }
        ftCo_800ADC28(fp);
        cmd = (*data).x18;
        {
            's123: {
                match cmd {
                    1 => {
                        ftCo_800ABA34(fp);
                    }
                    2 => {
                        super::ftcpuattack::ftCo_800B658C(fp);
                    }
                    3 => {
                        super::ftcpuattack::ftCo_800B9020(fp);
                    }
                    4 => {
                        ftCo_800A8DE4_noinline(fp);
                        if (*fp).ground_or_air == ((GA_Ground as i32) as u32) {
                            line_id = (*fp).coll_data.floor.index;
                            if (((ext::grBigBlue_801EF844(line_id) != 0) || (ext::grInishie1_801FCAAC(line_id) != 0)) || (ext::grCorneria_801E2D90(line_id) != 0)) || (ext::grVenom_80206D10(line_id) != 0) {
                                special_floor = 1;
                            }
                            if special_floor != 0 {
                                ftCo_800A0148(fp);
                            } else {
                                b15 = 1;
                            }
                        } else {
                            b15 = 1;
                        }
                        if b15 != 0 {
                            if (*fp).ground_or_air == ((GA_Ground as i32) as u32) {
                                on_ground = 1;
                            } else {
                                on_ground = 0;
                            }
                            if on_ground != 0 {
                                ftCo_CpuReturnToPreviousBehavior(fp, data);
                                (*data).set_xFA_b2((0 as u8) as u8);
                            } else {
                                ftCo_800A9904(fp);
                            }
                        }
                    }
                    5 => {
                        ftCo_800AC7D4(fp);
                    }
                    6 => {
                        ftCo_800ACB44(fp);
                    }
                    7 => {
                        super::ftcpuattack::ftCo_800BA9A0(fp);
                    }
                    8 => {
                        super::ftcpuattack::ftCo_800B7180(fp);
                    }
                    9 => {
                        super::ftcpuattack::ftCo_800B683C(fp);
                    }
                    10 => {
                        ftCo_800ACD5C(fp);
                    }
                    11 => {
                        ftCo_800AD42C(fp);
                    }
                    12 => {
                        ftCo_800AD54C(fp);
                    }
                    13 => {
                        if !(*fp).item_gobj.is_null() {
                            ftCo_CpuReturnToPreviousBehavior(fp, data);
                        } else {
                            ftCo_CpuSetNeutralStick(fp);
                            ftCo_CpuRetapA(fp);
                            (*data).x18 = (*data).x1C;
                        }
                    }
                    14 => {
                        ftCo_800AD7FC(fp);
                    }
                    15 => {
                        ftCo_800ABBA8(fp);
                    }
                    16 => {
                        ftCo_800AC30C_noinline(fp);
                    }
                    17 => {
                        ftCo_800AC434(fp);
                    }
                    19 => {
                        if (*fp).motion_id == 305 {
                            special_floor = 1;
                        } else if (*fp).motion_id == 306 {
                            special_floor = 2;
                        }
                        if special_floor == 0 {
                            ftCo_CpuReturnToPreviousBehavior(fp, data);
                        } else {
                            ftCo_CpuFinishWithNeutralStick(fp);
                        }
                    }
                    18 => {
                        ftCo_800AC5A0(fp);
                    }
                    0 => {
                        super::ftcmdscript::ftCo_800B4A78(fp);
                        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
                    }
                    _ => {
                        super::ftcmdscript::ftCo_800B463C(fp, (CpuCmd_Done as i32) as u8);
                    }
                }
            }
        }
        super::ftcmdscript::ftCo_800B49F4(fp);
    }
}

/// `ftCo_800B2AFC_IsIgnoredFloor`
unsafe fn ftCo_800B2AFC_IsIgnoredFloor(mut line: i32) -> i32 {
    return ftCo_IsIgnoredFloor(line);
}

/// `ftCo_800B2AFC`
pub unsafe fn ftCo_800B2AFC(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B2AFC");
    's124: {
        match (*fp).cpu.kind {
            0 => {
                {
                    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
                    let mut result: i32 = 0;
                    let mut found: i32 = 0;
                    let mut do_act: i32 = 0;
                    let mut x18: i32 = 0;
                    (*data).set_xF8_b0((0 as u8) as u8);
                    ftCo_CpuClearTargetModes(data);
                    (*data).set_xF9_b1((0 as u8) as u8);
                    x18 = (*fp).cpu.x18;
                    if (x18 != (*data).x20) && (x18 != (*data).x1C) {
                        (*data).x60 = 0;
                    }
                    if (*data).x18 == 4 {
                        do_act = 0;
                    } else {
                        (*data).set_xFA_b2((0 as u8) as u8);
                        do_act = 1;
                    }
                    if do_act != 0 {
                        let mut flags0: u32 = 0;
                        let mut line0: i32 = 0;
                        let mut floor_normal0: Vec3 = unsafe { core::mem::zeroed() };
                        let mut floor_pos0: Vec3 = unsafe { core::mem::zeroed() };
                        let mut x: f32 = (*fp).cur_pos.x;
                        let mut x2: f32 = x;
                        let mut y: f32 = (*fp).cur_pos.y;
                        let mut below: f32 = 0.0;
                        let mut above: f32 = 0.0;
                        found = 0;
                        below = y - 1e+03f32;
                        above = 1e+01f32 + y;
                        line0 = (-1_i32);
                        result = ext::mpCheckFloor(x2, above, x, below, 0.0f32, core::ptr::addr_of_mut!(floor_pos0) as *mut Vec, core::ptr::addr_of_mut!(line0) as *mut i32, core::ptr::addr_of_mut!(flags0) as *mut u32, core::ptr::addr_of_mut!(floor_normal0) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, found as usize as *mut HSD_GObj);
                        if result == 0 {
                            found = result;
                        } else {
                            's125: {
                                match ftCo_800B2AFC_IsIgnoredFloor(line0) {
                                    0 => {
                                        found = result;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if found != 0 {
                            let mut floor_x: f32 = 0.0;
                            let mut floor_y: f32 = 0.0;
                            floor_x = floor_pos0.x;
                            floor_y = floor_pos0.y;
                            ftCo_800A1F3C_noinline2(fp, floor_x, floor_y, 5.0f32);
                        }
                    }
                    if std::env::var_os("MELEE_DBG_CPU").is_some() { eprintln!("  B2AFC kind0 before ADE48 x38 {:08X}", (*fp).cpu.x38.to_bits()); }
                    let _ = ftCo_800ADE48(fp);
                    if std::env::var_os("MELEE_DBG_CPU").is_some() { eprintln!("  B2AFC kind0 after ADE48 x38 {:08X}", (*fp).cpu.x38.to_bits()); }
                    return;
                }
            }
            1 => {
                {
                    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
                    let mut found: i32 = 0;
                    let mut result: i32 = 0;
                    let mut do_act: i32 = 0;
                    let mut x18: i32 = 0;
                    (*data).set_xF8_b0((0 as u8) as u8);
                    ftCo_CpuClearTargetModes(data);
                    (*data).set_xF9_b1((0 as u8) as u8);
                    x18 = (*fp).cpu.x18;
                    if (x18 != (*data).x20) && (x18 != (*data).x1C) {
                        (*data).x60 = 0;
                    }
                    if (*data).x18 == 4 {
                        do_act = 0;
                    } else {
                        (*data).set_xFA_b2((0 as u8) as u8);
                        do_act = 1;
                    }
                    if do_act != 0 {
                        let mut flags1: u32 = 0;
                        let mut line1: i32 = 0;
                        let mut floor_normal1: Vec3 = unsafe { core::mem::zeroed() };
                        let mut floor_pos1: Vec3 = unsafe { core::mem::zeroed() };
                        let mut x: f32 = (*fp).cpu.x98.x;
                        let mut x2: f32 = x;
                        let mut y: f32 = (*fp).cpu.x98.y;
                        let mut below: f32 = 0.0;
                        let mut above: f32 = 0.0;
                        found = 0;
                        below = y - 1e+03f32;
                        above = 1e+01f32 + y;
                        line1 = (-1_i32);
                        result = ext::mpCheckFloor(x2, above, x, below, 0.0f32, core::ptr::addr_of_mut!(floor_pos1) as *mut Vec, core::ptr::addr_of_mut!(line1) as *mut i32, core::ptr::addr_of_mut!(flags1) as *mut u32, core::ptr::addr_of_mut!(floor_normal1) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, found as usize as *mut HSD_GObj);
                        if result == 0 {
                            found = result;
                        } else {
                            's126: {
                                match ftCo_800B2AFC_IsIgnoredFloor(line1) {
                                    0 => {
                                        found = result;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if found != 0 {
                            let mut floor_x: f32 = 0.0;
                            let mut floor_y: f32 = 0.0;
                            floor_x = floor_pos1.x;
                            floor_y = floor_pos1.y;
                            ftCo_800A1F3C_noinline2(fp, floor_x, floor_y, 5.0f32);
                        } else {
                            let mut cur_x: f32 = 0.0;
                            let mut cur_y: f32 = 0.0;
                            cur_x = (*fp).cur_pos.x;
                            cur_y = (*fp).cur_pos.y;
                            ftCo_800A1F3C_noinline2(fp, cur_x, cur_y, 5.0f32);
                        }
                    }
                    let _ = ftCo_800ADE48(fp);
                    return;
                }
            }
            2 => {
                ftCo_800B04DC(fp);
                return;
            }
            3 => {
                {
                    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
                    let mut found: i32 = 0;
                    let mut result: i32 = 0;
                    let mut do_act: i32 = 0;
                    let mut x18: i32 = 0;
                    (*data).set_xF8_b0((0 as u8) as u8);
                    ftCo_CpuClearTargetModes(data);
                    (*data).set_xF9_b1((0 as u8) as u8);
                    x18 = (*fp).cpu.x18;
                    if (x18 != (*data).x20) && (x18 != (*data).x1C) {
                        (*data).x60 = 0;
                    }
                    if (*data).x18 == 4 {
                        do_act = 0;
                    } else {
                        (*data).set_xFA_b2((0 as u8) as u8);
                        do_act = 1;
                    }
                    if do_act != 0 {
                        let mut flags3: u32 = 0;
                        let mut line3: i32 = 0;
                        let mut floor_normal3: Vec3 = unsafe { core::mem::zeroed() };
                        let mut floor_pos3: Vec3 = unsafe { core::mem::zeroed() };
                        let mut x: f32 = (*fp).cur_pos.x;
                        let mut x2: f32 = x;
                        let mut y: f32 = (*fp).cur_pos.y;
                        let mut below: f32 = 0.0;
                        let mut above: f32 = 0.0;
                        found = 0;
                        below = y - 1e+03f32;
                        above = 1e+01f32 + y;
                        line3 = (-1_i32);
                        result = ext::mpCheckFloor(x2, above, x, below, 0.0f32, core::ptr::addr_of_mut!(floor_pos3) as *mut Vec, core::ptr::addr_of_mut!(line3) as *mut i32, core::ptr::addr_of_mut!(flags3) as *mut u32, core::ptr::addr_of_mut!(floor_normal3) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, found as usize as *mut HSD_GObj);
                        if result == 0 {
                            found = result;
                        } else {
                            's127: {
                                match ftCo_800B2AFC_IsIgnoredFloor(line3) {
                                    0 => {
                                        found = result;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if found != 0 {
                            let mut floor_x: f32 = 0.0;
                            let mut floor_y: f32 = 0.0;
                            floor_x = floor_pos3.x;
                            floor_y = floor_pos3.y;
                            ftCo_800A1F3C_noinline2(fp, floor_x, floor_y, 5.0f32);
                        }
                    }
                    let _ = ftCo_800ADE48(fp);
                    return;
                }
            }
            6 => {
                ftCo_800B101C(fp);
                return;
            }
            4 => {
                ftCo_800B24B8(fp);
                return;
            }
            7 => {
                ftCo_800AF290(fp);
                return;
            }
            8 => {
                ftCo_800AECF0(fp);
                return;
            }
            9 => {
                ftCo_800B00F8(fp);
                return;
            }
            10 => {
                ftCo_800AFC40(fp);
                return;
            }
            11 => {
                ftCo_800B24B8(fp);
                return;
            }
            12 => {
                ftCo_800B24B8(fp);
                return;
            }
            13 => {
                ftCo_800B126C(fp);
                return;
            }
            14 => {
                ftCo_800B1478(fp);
                return;
            }
            15 => {
                (*fp).cpu.x18 = 0;
                return;
            }
            17 => {
                ftCo_800B17D0(fp);
                return;
            }
            18 => {
                ftCo_800AF78C(fp);
                return;
            }
            19 => {
                ftCo_800AFE3C(fp, 0);
                return;
            }
            20 => {
                ftCo_800AFE3C(fp, 1);
                return;
            }
            21 => {
                ftCo_800AFE3C(fp, 2);
                return;
            }
            22 => {
                ftCo_800AFE3C(fp, 3);
                return;
            }
            23 => {
                {
                    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
                    let mut do_act: i32 = 0;
                    let mut x18: i32 = 0;
                    ftCo_CpuInitEnemyTarget(fp, data);
                    x18 = (*data).x18;
                    if (x18 != (*data).x20) && (x18 != (*data).x1C) {
                        (*data).x60 = 0;
                    }
                    if (*data).x18 == 4 {
                        do_act = 0;
                    } else {
                        (*data).set_xFA_b2((0 as u8) as u8);
                        do_act = 1;
                    }
                    if do_act != 0 {
                        ftCo_800A75DC(fp, (*data).x44);
                    }
                    let _ = ftCo_800ADE48(fp);
                    return;
                }
            }
            24 => {
                ftCo_800B1DA0_noinline2(fp);
                return;
            }
            25 => {
                {
                    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
                    let mut do_act: i32 = 0;
                    let mut x18: i32 = 0;
                    ftCo_CpuInitEnemyTarget(fp, data);
                    x18 = (*data).x18;
                    if (x18 != (*data).x20) && (x18 != (*data).x1C) {
                        (*data).x60 = 0;
                    }
                    if (*data).x18 == 4 {
                        do_act = 0;
                    } else {
                        (*data).set_xFA_b2((0 as u8) as u8);
                        do_act = 1;
                    }
                    if do_act != 0 {
                        ftCo_800A75DC(fp, (*data).x44);
                    }
                    let _ = ftCo_800ADE48(fp);
                    return;
                }
            }
            26 => {
                {
                    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
                    let mut found: i32 = 0;
                    let mut result: i32 = 0;
                    let mut do_act: i32 = 0;
                    let mut x18: i32 = 0;
                    ftCo_CpuInitEnemyTarget(fp, data);
                    x18 = (*data).x18;
                    if (x18 != (*data).x20) && (x18 != (*data).x1C) {
                        (*data).x60 = 0;
                    }
                    if (*data).x18 == 4 {
                        do_act = 0;
                    } else {
                        (*data).set_xFA_b2((0 as u8) as u8);
                        do_act = 1;
                    }
                    if do_act != 0 {
                        let mut flags26: u32 = 0;
                        let mut line26: i32 = 0;
                        let mut floor_normal26: Vec3 = unsafe { core::mem::zeroed() };
                        let mut floor_pos26: Vec3 = unsafe { core::mem::zeroed() };
                        let mut x: f32 = (*fp).cur_pos.x;
                        let mut x2: f32 = x;
                        let mut y: f32 = (*fp).cur_pos.y;
                        let mut below: f32 = 0.0;
                        let mut above: f32 = 0.0;
                        found = 0;
                        below = y - 1e+03f32;
                        above = 1e+01f32 + y;
                        line26 = (-1_i32);
                        result = ext::mpCheckFloor(x2, above, x, below, 0.0f32, core::ptr::addr_of_mut!(floor_pos26) as *mut Vec, core::ptr::addr_of_mut!(line26) as *mut i32, core::ptr::addr_of_mut!(flags26) as *mut u32, core::ptr::addr_of_mut!(floor_normal26) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, found as usize as *mut HSD_GObj);
                        if result == 0 {
                            found = result;
                        } else {
                            's128: {
                                match ftCo_800B2AFC_IsIgnoredFloor(line26) {
                                    0 => {
                                        found = result;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if found != 0 {
                            let mut floor_x: f32 = 0.0;
                            let mut floor_y: f32 = 0.0;
                            floor_x = floor_pos26.x;
                            floor_y = floor_pos26.y;
                            ftCo_800A1F3C_noinline2(fp, floor_x, floor_y, 5.0f32);
                        }
                    }
                    let _ = ftCo_800ADE48(fp);
                    return;
                }
            }
            27 => {
                ftCo_800B1EF0(fp);
                return;
            }
            28 => {
                ftCo_800B21C8(fp);
                return;
            }
            29 => {
                ftCo_800B1AB8(fp);
                return;
            }
            _ => {
                ftCo_800B24B8(fp);
                return;
            }
        }
    }
}

/// `ftCo_800B33B0_IsIgnoredFloor`
unsafe fn ftCo_800B33B0_IsIgnoredFloor(mut line1: i32) -> i32 {
    return ftCo_800A1B38_noinline(line1);
}

/// `ftCo_CpuUpdateRecoveryScale`
unsafe fn ftCo_CpuUpdateRecoveryScale(mut fp: *mut Fighter, mut data: *mut CpuFighter, mut timer: *const i32) {
    if ((*timer) % 30) == 0 {
        let mut rand: f32 = ext::HSD_Randf();
        (*data).x570 = (0.05f64).mul_add(((*data).level + 1) as f64, 0.05f64 * (rand as f64)) as f32;
    }
    ftCo_800A0CB0(fp);
}

/// `ftCo_800B33B0`
pub unsafe fn ftCo_800B33B0(mut fp: *mut Fighter) {
    melee_core::ct!("ftCo_800B33B0");
    let mut data: *mut CpuFighter = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    let mut temp_data: *mut CpuFighter = null_mut();
    let mut temp_data2: *mut CpuFighter = null_mut();
    let mut timer: *mut i32 = core::ptr::addr_of_mut!((*data).x7C) as *mut i32;
    let mut floor_vel: Vec3 = unsafe { core::mem::zeroed() };
    let mut target_floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut target_floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut line2: i32 = 0;
    let mut flags2: u32 = 0;
    let mut fx: f32 = 0.0;
    let mut below: f32 = 0.0;
    let mut tmp: f32 = 0.0;
    let mut found: i32 = 0;
    let mut result: i32 = 0;
    let mut in_bounds: i32 = 0;
    let mut blocked: i32 = 0;
    let mut vx: f32 = 0.0;
    let mut vy: f32 = 0.0;
    let mut vmag: f32 = 0.0;
    let mut fy: f32 = 0.0;
    let mut sx: f32 = 0.0;
    let mut sy: f32 = 0.0;
    let mut flags1: u32 = 0;
    let mut line1: i32 = 0;
    let mut floor_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut flags3: u32 = 0;
    let mut line3: i32 = 0;
    let mut ceiling_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut ceiling_pos: Vec3 = unsafe { core::mem::zeroed() };
    if ((*timer) % 300) == 0 {
        if ext::HSD_Randf() < fmadds(0.04f32, (*data).level as f32, 0.3f32) {
            (*data).set_xFA_b34((1 as u8) as u8);
        } else {
            (*data).set_xFA_b34((0 as u8) as u8);
        }
    }
    fy = (*fp).cur_pos.y;
    found = 0;
    fx = (*fp).cur_pos.x;
    line1 = (-1_i32);
    {
        sy = fy;
        sx = fx;
        below = ((sy as f64) - 1000.0f64) as f32;
        tmp = (10.0f64 + (sy as f64)) as f32;
        {
            result = ext::mpCheckFloor(sx, tmp, sx, below, 0.0f32, core::ptr::addr_of_mut!(floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line1) as *mut i32, core::ptr::addr_of_mut!(flags1) as *mut u32, core::ptr::addr_of_mut!(floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, found as usize as *mut HSD_GObj);
        }
    }
    if result == 0 {
        found = result;
    } else {
        's129: {
            match ftCo_800B33B0_IsIgnoredFloor(line1) {
                0 => {
                    found = result;
                }
                _ => {}
            }
        }
    }
    if found != 0 {
        let mut floor_x: f32 = 0.0;
        let mut floor_y: f32 = 0.0;
        let mut temp_data: *mut CpuFighter = null_mut();
        let mut oob: i32 = 0;
        floor_y = floor_pos.y;
        temp_data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
        floor_x = floor_pos.x;
        if (((floor_x < ((*fp).cpu.half_width + ext::Stage_GetBlastZoneLeftOffset())) || (floor_x > (ext::Stage_GetBlastZoneRightOffset() - (*temp_data).half_width))) || (floor_y < ((*temp_data).half_height + ext::Stage_GetBlastZoneBottomOffset()))) || (floor_y > (ext::Stage_GetBlastZoneTopOffset() - (*temp_data).half_height)) {
            oob = 1;
        } else {
            oob = 0;
        }
        if oob != 0 {
            in_bounds = 0;
        } else {
            in_bounds = 1;
        }
    } else {
        in_bounds = 0;
    }
    if in_bounds != 0 {
        (*data).set_xFA_b5((1 as u8) as u8);
    } else {
        (*data).set_xFA_b5((0 as u8) as u8);
    }
    vy = (*fp).coll_data.cur_pos.y - (*fp).coll_data.last_pos.y;
    vx = (*fp).coll_data.cur_pos.x - (*fp).coll_data.last_pos.x;
    vmag = melee_runtime::gekko::sqrtf(fmadds(vx, vx, vy * vy));
    if (vmag as f64) < 0.01f64 {
        result = 1;
    } else {
        result = 0;
    }
    if result != 0 {
        (*data).x84 = (*data).x84 + 1;
    } else {
        (*data).x84 = 0;
    }
    if (*data).xF9_b0() != 0 {
        if (*data).x30 != 0 {
            (*data).x30 = (*data).x30 - 1;
        } else {
            (*data).set_xF9_b0((0 as u8) as u8);
        }
    } else {
        let mut rand: f32 = ext::HSD_Randf();
        (*data).x30 = melee_core::ppc::fctiwz_f32(1.2e+02f32 * (0.5f32 * (0.5f32 * rand)));
    }
    if (*data).x60 != 0 {
        (*data).x60 = (*data).x60 - 1;
        if (*data).x60 == 0 {
            (*data).x54.x = (*data).x64.x;
            (*data).x54.y = (*data).x64.y;
        }
    }
    sy = (*data).x54.y;
    blocked = 0;
    sx = (*data).x54.x;
    line2 = (-1_i32);
    below = ((sy as f64) - 2.0f64) as f32;
    tmp = (2.0f64 + (sy as f64)) as f32;
    fx = sx;
    result = ext::mpCheckFloor(fx, tmp, sx, below, 0.0f32, core::ptr::addr_of_mut!(target_floor_pos) as *mut Vec, core::ptr::addr_of_mut!(line2) as *mut i32, core::ptr::addr_of_mut!(flags2) as *mut u32, core::ptr::addr_of_mut!(target_floor_normal) as *mut Vec, (-1_i32), (-1_i32), (-1_i32), None, blocked as usize as *mut HSD_GObj);
    if result != 0 {
        let mut line: i32 = line2;
        if (((ext::grBigBlue_801EF844(line) != 0) || (ext::grInishie1_801FCAAC(line) != 0)) || (ext::grCorneria_801E2D90(line) != 0)) || (ext::grVenom_80206D10(line) != 0) {
            blocked = 1;
        }
        if blocked != 0 {
            result = 0;
        }
    }
    if result != 0 {
        let _ = ext::mpGetSpeed(line2, core::ptr::addr_of_mut!(target_floor_pos) as *mut Vec, core::ptr::addr_of_mut!(floor_vel) as *mut Vec);
        (*data).x54.x = (*data).x54.x + floor_vel.x;
        (*data).x54.y = (*data).x54.y + floor_vel.y;
    } else if ext::grLib_801C9E60(core::ptr::addr_of_mut!(floor_vel) as *mut Vec) != 0 {
        (*data).x54.x = (*data).x54.x + floor_vel.x;
        (*data).x54.y = (*data).x54.y + floor_vel.y;
    }
    temp_data = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    ftCo_CpuUpdateRecoveryScale(fp, temp_data, timer as *const i32);
    {
        temp_data2 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
        if (*fp).cpu.xFA_b6() != 0 {
            fx = (*fp).cur_pos.x;
            fy = (*fp).cur_pos.y;
            if ext::mpCheckCeiling(fx, fy, fx, 1e+03f32 + fy, core::ptr::addr_of_mut!(ceiling_pos) as *mut Vec, core::ptr::addr_of_mut!(line3) as *mut i32, core::ptr::addr_of_mut!(flags3) as *mut u32, core::ptr::addr_of_mut!(ceiling_normal) as *mut Vec, (-1_i32), (-1_i32)) == 0 {
                (*temp_data2).set_xFA_b6((0 as u8) as u8);
            }
        } else if ((*fp).ground_or_air == ((GA_Ground as i32) as u32)) && (ext::grCorneria_801E2E50((*fp).coll_data.floor.index) != 0) {
            (*temp_data2).set_xFA_b6((1 as u8) as u8);
        }
    }
}

/// `ftCo_800B3900`
pub unsafe fn ftCo_800B3900(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCo_800B3900");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    {
        if ext::ScriptGame_CpuStandTick(fp) != 0 {
            return;
        }
    }
    let dbg = std::env::var_os("MELEE_DBG_CPU").is_some();
    macro_rules! st { ($n:expr) => { if dbg { eprintln!("cpu p{} x7C {} {} x38 {:08X}", (*fp).player_id, (*fp).cpu.x7C, $n, (*fp).cpu.x38.to_bits()); } } }
    st!("start");
    ftCo_800B33B0(fp);
    st!("33B0");
    ftCo_800B2AFC(fp);
    st!("2AFC");
    ftCo_800B2790(fp);
    st!("2790");
    super::ftcmdscript::ftCo_800B3E04(fp);
    st!("3E04");
    ftCo_800B0AF4(fp);
    st!("0AF4");
    (*fp).cpu.x7C = (*fp).cpu.x7C + 1;
}

/// `ftCo_800B395C`
pub unsafe fn ftCo_800B395C(mut gobj: *mut HSD_GObj, mut arg1: i32) -> i32 {
    melee_core::ct!("ftCo_800B395C");
    let mut fp: *mut Fighter = null_mut();
    let mut temp_r0_2: i32 = 0;
    let mut temp_r30: *mut CpuFighter = null_mut();
    let mut sp84: GXColor = unsafe { core::mem::zeroed() };
    let mut sp80: GXColor = unsafe { core::mem::zeroed() };
    let mut sp50: [[f32; 4]; 3] = unsafe { core::mem::zeroed() };
    let mut sp44: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp38: Vec3 = unsafe { core::mem::zeroed() };
    fp = HSD_GObjGetUserData(gobj) as *mut Fighter;
    temp_r30 = core::ptr::addr_of_mut!((*fp).cpu) as *mut CpuFighter;
    if ext::Player_8003248C((*fp).player_id as i32, (*fp).is_sub_fighter() as i32) == ((Gm_PKind_Cpu as i32) as u32) {
        's130: {
            match (*temp_r30).x18 {
                2 | 3 => {
                    sp84.r = 255 as u8;
                    sp80.r = 255 as u8;
                    sp84.g = 0 as u8;
                    sp80.g = 0 as u8;
                    sp84.b = 0 as u8;
                    sp80.b = 0 as u8;
                    sp84.a = 96 as u8;
                    sp80.a = 96 as u8;
                }
                7 => {
                    sp84.r = 0 as u8;
                    sp80.r = 0 as u8;
                    sp84.g = 0 as u8;
                    sp80.g = 0 as u8;
                    sp84.b = 255 as u8;
                    sp80.b = 255 as u8;
                    sp84.a = 96 as u8;
                    sp80.a = 96 as u8;
                }
                4 => {
                    sp84.r = 255 as u8;
                    sp80.r = 255 as u8;
                    sp84.g = 255 as u8;
                    sp80.g = 255 as u8;
                    sp84.b = 255 as u8;
                    sp80.b = 255 as u8;
                    sp84.a = 255 as u8;
                    sp80.a = 255 as u8;
                }
                10 => {
                    sp84.r = 255 as u8;
                    sp80.r = 255 as u8;
                    sp84.g = 0 as u8;
                    sp80.g = 0 as u8;
                    sp84.b = 255 as u8;
                    sp80.b = 255 as u8;
                    sp84.a = 96 as u8;
                    sp80.a = 96 as u8;
                }
                1 => {
                    sp84.r = 0 as u8;
                    sp80.r = 0 as u8;
                    sp84.g = 255 as u8;
                    sp80.g = 255 as u8;
                    sp84.b = 0 as u8;
                    sp80.b = 0 as u8;
                    sp84.a = 96 as u8;
                    sp80.a = 96 as u8;
                }
                18 => {
                    sp84.r = 0 as u8;
                    sp80.r = 0 as u8;
                    sp84.g = 0 as u8;
                    sp80.g = 0 as u8;
                    sp84.b = 0 as u8;
                    sp80.b = 0 as u8;
                    sp84.a = 96 as u8;
                    sp80.a = 96 as u8;
                }
                _ => {
                    sp84.r = 255 as u8;
                    sp80.r = 255 as u8;
                    sp84.g = 255 as u8;
                    sp80.g = 255 as u8;
                    sp84.b = 0 as u8;
                    sp80.b = 0 as u8;
                    sp84.a = 96 as u8;
                    sp80.a = 96 as u8;
                }
            }
        }
        sp44.x = (*temp_r30).x54.x;
        sp44.y = (*temp_r30).x54.y;
        sp44.z = 0.0f32;
        ext::PSMTXIdentity(sp50.as_mut_ptr());
        ext::lbColl_800096B4(sp50.as_mut_ptr(), sp44, sp44, core::ptr::addr_of_mut!(sp84) as *mut _GXColor, core::ptr::addr_of_mut!(sp80) as *mut _GXColor, 5.0f32);
        sp84.r = 255 as u8;
        sp84.g = 255 as u8;
        sp84.b = 0 as u8;
        sp84.a = 96 as u8;
        if ((*fp).facing_dir as f64) > 0.0f64 {
            sp44.x = (*fp).cur_pos.x - (*temp_r30).x560;
            sp44.y = (*fp).cur_pos.y + (*temp_r30).x568;
            sp44.z = (*fp).cur_pos.z;
            sp38.x = (*fp).cur_pos.x + (*temp_r30).x55C;
            sp38.y = (*fp).cur_pos.y;
            sp38.z = (*fp).cur_pos.z;
        } else {
            sp44.x = (*fp).cur_pos.x - (*temp_r30).x55C;
            sp44.y = (*fp).cur_pos.y + (*temp_r30).x568;
            sp44.z = (*fp).cur_pos.z;
            sp38.x = (*fp).cur_pos.x + (*temp_r30).x560;
            sp38.y = (*fp).cur_pos.y;
            sp38.z = (*fp).cur_pos.z;
        }
        if (*temp_r30).xF8_b7() != 0 {
            temp_r0_2 = (*temp_r30).x7C % 3;
            if temp_r0_2 == 1 {
                sp84.r = 255 as u8;
                sp84.g = 255 as u8;
                sp84.b = 0 as u8;
                sp84.a = 255 as u8;
                ext::lbColl_80009DD4(core::ptr::addr_of_mut!(sp44) as *mut Vec, core::ptr::addr_of_mut!(sp38) as *mut Vec, core::ptr::addr_of_mut!(sp84) as *mut _GXColor);
            } else if temp_r0_2 == 2 {
                sp84.r = 0 as u8;
                sp84.g = 0 as u8;
                sp84.b = 0 as u8;
                sp84.a = 0 as u8;
                ext::lbColl_80009DD4(core::ptr::addr_of_mut!(sp44) as *mut Vec, core::ptr::addr_of_mut!(sp38) as *mut Vec, core::ptr::addr_of_mut!(sp84) as *mut _GXColor);
            } else {
                sp84.r = 255 as u8;
                sp84.g = 255 as u8;
                sp84.b = 255 as u8;
                sp84.a = 255 as u8;
                ext::lbColl_80009DD4(core::ptr::addr_of_mut!(sp44) as *mut Vec, core::ptr::addr_of_mut!(sp38) as *mut Vec, core::ptr::addr_of_mut!(sp84) as *mut _GXColor);
            }
        } else {
            ext::lbColl_80009DD4(core::ptr::addr_of_mut!(sp44) as *mut Vec, core::ptr::addr_of_mut!(sp38) as *mut Vec, core::ptr::addr_of_mut!(sp84) as *mut _GXColor);
        }
        if (*temp_r30).x18 == 2 {
            sp84.r = 255 as u8;
            sp84.g = 0 as u8;
            sp84.b = 0 as u8;
            sp84.a = 96 as u8;
            if ((*fp).facing_dir as f64) > 0.0f64 {
                sp44.x = (*fp).cur_pos.x + (*temp_r30).x6C.x;
                sp44.y = (*fp).cur_pos.y + (*temp_r30).x74.y;
                sp44.z = (*fp).cur_pos.z;
                sp38.x = (*fp).cur_pos.x + (*temp_r30).x74.x;
                sp38.y = (*fp).cur_pos.y + (*temp_r30).x6C.y;
                sp38.z = (*fp).cur_pos.z;
            } else {
                sp44.x = (*fp).cur_pos.x - (*temp_r30).x74.x;
                sp44.y = (*fp).cur_pos.y + (*temp_r30).x74.y;
                sp44.z = (*fp).cur_pos.z;
                sp38.x = (*fp).cur_pos.x - (*temp_r30).x6C.x;
                sp38.y = (*fp).cur_pos.y + (*temp_r30).x6C.y;
                sp38.z = (*fp).cur_pos.z;
            }
            ext::lbColl_80009DD4(core::ptr::addr_of_mut!(sp44) as *mut Vec, core::ptr::addr_of_mut!(sp38) as *mut Vec, core::ptr::addr_of_mut!(sp84) as *mut _GXColor);
        }
        return 1;
    }
    return 0;
}

/// `ftCo_CpuSetNeutralStick`
unsafe fn ftCo_CpuSetNeutralStick(mut fp: *mut Fighter) {
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickX as i32) as u8, 0 as u8);
    super::ftcmdscript::ftCo_800B46B8(fp, (CpuCmd_SetLstickY as i32) as u8, 0 as u8);
}

/// `ftAi_Kind`
unsafe fn ftAi_Kind(mut kind: i32) -> i32 {
    let mut like: i32 = 0;
    if kind < (Ft_Kind_Mex0 as i32) {
        return kind;
    }
    like = ext::Geno_DefineAiLike(kind);
    return if like >= 0 { like } else { kind };
}

/// `HSD_GObjGetUserData`
unsafe fn HSD_GObjGetUserData(mut gobj: *mut HSD_GObj) -> *mut c_void {
    return (*gobj).user_data;
}

/// `HSD_GObjGetNext`
unsafe fn HSD_GObjGetNext(mut gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    return (*gobj).next;
}

/// The hand-written CPU controller reads (stick / trigger / button accessors, the hurtbox probe).
pub mod ctl {
    #![allow(non_snake_case, clippy::missing_safety_doc, unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]

#[allow(unused_imports)]
use crate::tu::ftcpuattack::ftco_800b9704;
use melee_types::{Fighter, FighterKind, Ft_Kind_Donkey, Ft_Kind_GKoops, Ft_Kind_Koopa, Ft_Kind_Nana};
use crate::types::*;

type F = *mut Fighter;

/// `ftCo_800A101C(fp, kind, level, x14)`. `rand` is `HSD_Randf` (called twice, in C order:
/// once for `x7C`/`x80`, once inside `ftCo_800B9704`). `nana_partner` is `ftCo_800A589C(fp)`'s
/// position and facing (only read for Nana).
pub fn ftco_800a101c(
    fp: &mut Fighter,
    kind: i32,
    level: i32,
    x14: i32,
    floor: Option<(f32, f32)>,
    nana_partner: Option<(melee_types::Vec, f32)>,
    rand: &mut dyn FnMut() -> f32,
) {
    melee_core::ct!("ftco_800a101c");
    let ai_kind: FighterKind = fp.kind;
    let cur_pos = fp.cur_pos;
    {
        let c = &mut fp.cpu;
        if ai_kind == Ft_Kind_Nana {
            c.set_xF9_b2(1);
            c.kind = 6;
            c.x3C = 15.0;
        } else {
            c.kind = kind as u32;
            c.x3C = 40.0;
        }
        c.level = level;
        c.x14 = x14;
        match c.kind {
            1 | 25 => {
                c.x18 = 0xC;
                c.x1C = 0xC;
                c.x20 = 0;
            }
            3 => {
                c.x18 = 1;
                c.x1C = 1;
                c.x20 = 0xB;
            }
            15 => {
                c.x18 = 0;
                c.x1C = 0;
                c.x20 = 0;
            }
            _ => {
                c.x18 = 1;
                c.x1C = 1;
                c.x20 = 0xA;
            }
        }
        c.x24 = 0x12C;
        c.x28 = 0;
        c.x2C = 5;
        c.x30 = 0;
        c.set_xF9_b0(0);
        c.x38 = 5.0;
        c.x40 = 50.0;
        c.x44 = core::ptr::null_mut();
        c.x48 = core::ptr::null_mut();
        c.x50 = 0;
        c.x98 = cur_pos;
        let (fx, fy) = floor.unwrap_or((cur_pos.x, cur_pos.y));
        c.x54.x = fx;
        c.x64.x = fx;
        c.x54.y = fy;
        c.x64.y = fy;
        c.x60 = 0;
        c.x74.y = 0.0;
        c.x74.x = 0.0;
        c.x6C.y = 0.0;
        c.x6C.x = 0.0;
        c.set_xF8_b0(0);
        c.set_xF8_b12(0);
    }
    // `temp_r6 = 10.0 * HSD_Randf()`: a double product truncated to int.
    let r = rand();
    let t = melee_core::ppc::fctiwz(10.0f64 * r as f64);
    {
        let c = &mut fp.cpu;
        c.x7C = t;
        c.x80 = t;
        c.x84 = 0;
        c.x90 = 0;
        c.x88 = 0;
        c.x8C = 0;
        c.csP = core::ptr::null_mut();
        c.command_duration = 0;
        c.write_pos = c.buffer.as_mut_ptr();
        c.xA4 = 0;
        c.xC8 = 0;
        c.xEC = 0;
        c.set_xF8_b5(0);
        c.set_xF8_b6(0);
        c.set_xF8_b7(0);
        c.set_xF9_b1(0);
        c.buttons = 0;
        c.lstick.x = 0;
        c.lstick.y = 0;
        c.cstick.x = 0;
        c.cstick.y = 0;
        c.rtrigger = 0;
        c.ltrigger = 0;
        c.set_xF9_b2(0);
        c.set_xF9_b3(0);
        c.set_xF9_b4(0);
        c.set_xF9_b6(0);
        c.set_xF9_b5(0);
        c.set_xF9_b7(0);
        c.set_xFA_b7(0);
        c.set_xFB_b0(0);
        c.x94 = 0;
        c.set_xFA_b1(0);
        c.set_xFA_b2(0);
        c.set_xFA_b34(1);
        c.set_xFA_b5(0);
        c.set_xFA_b6(0);
        c.x55C = 1.0;
        c.x560 = 1.0;
        c.x564 = 0.5 * (c.x55C + c.x560);
        c.x568 = 2.0;
        c.x444 = &mut c.xFC[5];
        c.x448 = c.xFC.as_mut_ptr();
        for e in c.xFC.iter_mut() {
            e.x0 = 0;
            e.lstick.x = 0;
            e.lstick.y = 0;
            e.x4 = 0;
            e.x5 = 0;
            match (ai_kind == Ft_Kind_Nana, nana_partner) {
                (true, Some((pos, facing))) => {
                    e.cur_pos = pos;
                    e.facing_dir = facing;
                }
                (true, None) => panic!("ftCo_800A101C on Nana needs ftCo_800A589C's partner (TODO(group3 nana))"),
                _ => {
                    e.cur_pos.x = 0.0;
                    e.cur_pos.y = 0.0;
                    e.cur_pos.z = 0.0;
                    e.facing_dir = 1.0;
                }
            }
        }
    }
    let g = fp.co_attrs.gravity;
    let v = fp.co_attrs.jump_v_initial_velocity * fp.co_attrs.air_jump_v_multiplier;
    let zero_g = g < 0.00001 && g > -0.00001;
    fp.cpu.x558 = if zero_g { 10.0 } else { ((v * v) as f64 / (2.0 * g as f64)) as f32 };
    fp.cpu.half_width = 10.0;
    fp.cpu.half_height = 10.0;
    fp.cpu.x570 = 1.0;
    let r = rand();
    ftco_800b9704(fp, r);
    fp.cpu.x56C = match ai_kind {
        k if k == Ft_Kind_Donkey || k == Ft_Kind_Koopa => 8.5,
        k if k == Ft_Kind_GKoops => 17.0,
        k if k == Ft_Kind_Nana => 1.0,
        _ => 3.5,
    };
}

}
pub use ctl::ftco_800a101c;
