//! ftlib.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::ftlib::{fp, msid_in};

// ---- folded from ftlib.rs (consolidation steps 21/22) ----
/// `ftLib_GetPos_fp`: the current position.
pub unsafe fn ftLib_GetPos_fp(gobj: *mut HSD_GObj, pos: *mut Vec) {
    melee_core::ct!("ftLib_GetPos_fp");
    *pos = fp(gobj).cur_pos;
}

/// `ftLib_GetPrevPos_fp`: the previous position.
pub unsafe fn ftLib_GetPrevPos_fp(gobj: *mut HSD_GObj, pos: *mut Vec) {
    melee_core::ct!("ftLib_GetPrevPos_fp");
    *pos = fp(gobj).prev_pos;
}

/// `ftLib_GetCameraSubject_fp`: the fighter's camera subject.
pub unsafe fn ftLib_GetCameraSubject_fp(gobj: *mut HSD_GObj) -> *mut CmSubject {
    melee_core::ct!("ftLib_GetCameraSubject_fp");
    fp(gobj).x890_cameraBox
}

/// `ftLib_80086B80_fp`: the vertical camera extent target of the subject.
pub unsafe fn ftLib_80086B80_fp(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_80086B80_fp");
    (*ftLib_GetCameraSubject_fp(gobj)).target_ext.v.z
}

/// `ftLib_GetCameraSubjectBonePos_fp`: the subject's bone position.
pub unsafe fn ftLib_GetCameraSubjectBonePos_fp(gobj: *mut HSD_GObj, v: *mut Vec) {
    melee_core::ct!("ftLib_GetCameraSubjectBonePos_fp");
    *v = (*ftLib_GetCameraSubject_fp(gobj)).bone_pos;
}

/// `ftLib_GetPosDelta_fp`: the position delta of the frame.
pub unsafe fn ftLib_GetPosDelta_fp(gobj: *mut HSD_GObj, v: *mut Vec) {
    melee_core::ct!("ftLib_GetPosDelta_fp");
    *v = fp(gobj).pos_delta;
}

/// `ftLib_GetPlayerIndex_fp`: the player id.
pub unsafe fn ftLib_GetPlayerIndex_fp(gobj: *mut HSD_GObj) -> u8 {
    melee_core::ct!("ftLib_GetPlayerIndex_fp");
    fp(gobj).player_id
}

/// `ftLib_IsInHitstun_fp`.
pub unsafe fn ftLib_IsInHitstun_fp(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_IsInHitstun_fp");
    fp(gobj).x221C_b6() != 0
}

/// `ftLib_IsFighter_fp`: is `gobj` a fighter GObj.
pub unsafe fn ftLib_IsFighter_fp(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_IsFighter_fp");
    !gobj.is_null() && (*gobj).classifier as i32 == HSD_GOBJ_CLASS_FIGHTER
}

/// `ftLib_80086A4C_fp`.
pub unsafe fn ftLib_80086A4C_fp(gobj: *mut HSD_GObj, val: f32) {
    melee_core::ct!("ftLib_80086A4C_fp");
    fp(gobj).dmg.x1958 = val;
}

/// `ftLib_GetCollData_fp`: the fighter's collision data.
pub unsafe fn ftLib_GetCollData_fp(gobj: *mut HSD_GObj) -> *mut CollData {
    melee_core::ct!("ftLib_GetCollData_fp");
    &mut fp(gobj).coll_data
}

/// `ftLib_IsOffscreen_fp`.
pub unsafe fn ftLib_IsOffscreen_fp(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_IsOffscreen_fp");
    fp(gobj).x221F_b0() != 0
}

/// `ftLib_IsSleeping_fp`.
pub unsafe fn ftLib_IsSleeping_fp(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_IsSleeping_fp");
    fp(gobj).x221F_b3() != 0
}

/// `ftLib_IsSleeping_8008731C`.
pub unsafe fn ftLib_IsSleeping_8008731C(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_IsSleeping_8008731C");
    ftLib_IsSleeping_fp(gobj) as i32
}

/// `ftLib_GetShadow_fp`: the fighter's shadow record.
pub unsafe fn ftLib_GetShadow_fp(gobj: *mut HSD_GObj) -> *mut LbShadow {
    melee_core::ct!("ftLib_GetShadow_fp");
    &mut fp(gobj).x20A4
}

/// `ftLib_IsSubFighter_fp`: is it a sub fighter (Nana).
pub unsafe fn ftLib_IsSubFighter_fp(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_IsSubFighter_fp");
    fp(gobj).is_sub_fighter() as i32
}

/// `ftLib_IsDead_fp`: in a death state.
pub unsafe fn ftLib_IsDead_fp(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_IsDead_fp");
    msid_in(gobj, ftCo_MS_DeadDown, ftCo_MS_DeadUpFallHitCameraIce)
}

/// `ftLib_IsEntry`: in the entry animation.
pub unsafe fn ftLib_IsEntry(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_IsEntry");
    msid_in(gobj, ftCo_MS_Entry, ftCo_MS_EntryEnd)
}
