//! ftlib.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::ftlib::{fp, msid_in};

// ---- folded from ftlib.rs (consolidation steps 21/22) ----
/// `ftLib_80086644_fp`: the current position.
pub unsafe fn ftLib_80086644_fp(gobj: *mut HSD_GObj, pos: *mut Vec) {
    melee_core::ct!("ftLib_80086644_fp");
    *pos = fp(gobj).cur_pos;
}

/// `ftLib_80086684_fp`: the previous position.
pub unsafe fn ftLib_80086684_fp(gobj: *mut HSD_GObj, pos: *mut Vec) {
    melee_core::ct!("ftLib_80086684_fp");
    *pos = fp(gobj).prev_pos;
}

/// `ftLib_80086B74_fp`: the fighter's camera subject.
pub unsafe fn ftLib_80086B74_fp(gobj: *mut HSD_GObj) -> *mut CmSubject {
    melee_core::ct!("ftLib_80086B74_fp");
    fp(gobj).x890_cameraBox
}

/// `ftLib_80086B80_fp`: the vertical camera extent target of the subject.
pub unsafe fn ftLib_80086B80_fp(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_80086B80_fp");
    (*ftLib_80086B74_fp(gobj)).target_ext.v.z
}

/// `ftLib_80086B90_fp`: the subject's bone position.
pub unsafe fn ftLib_80086B90_fp(gobj: *mut HSD_GObj, v: *mut Vec) {
    melee_core::ct!("ftLib_80086B90_fp");
    *v = (*ftLib_80086B74_fp(gobj)).bone_pos;
}

/// `ftLib_80086BEC_fp`: the position delta of the frame.
pub unsafe fn ftLib_80086BEC_fp(gobj: *mut HSD_GObj, v: *mut Vec) {
    melee_core::ct!("ftLib_80086BEC_fp");
    *v = fp(gobj).pos_delta;
}

/// `ftLib_80086BE0_fp`: the player id.
pub unsafe fn ftLib_80086BE0_fp(gobj: *mut HSD_GObj) -> u8 {
    melee_core::ct!("ftLib_80086BE0_fp");
    fp(gobj).player_id
}

/// `ftLib_80086EC0_fp`.
pub unsafe fn ftLib_80086EC0_fp(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_80086EC0_fp");
    fp(gobj).x221C_b6() != 0
}

/// `ftLib_80086960_fp`: is `gobj` a fighter GObj.
pub unsafe fn ftLib_80086960_fp(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_80086960_fp");
    !gobj.is_null() && (*gobj).classifier as i32 == HSD_GOBJ_CLASS_FIGHTER
}

/// `ftLib_80086A4C_fp`.
pub unsafe fn ftLib_80086A4C_fp(gobj: *mut HSD_GObj, val: f32) {
    melee_core::ct!("ftLib_80086A4C_fp");
    fp(gobj).dmg.x1958 = val;
}

/// `ftLib_80086984_fp`: the fighter's collision data.
pub unsafe fn ftLib_80086984_fp(gobj: *mut HSD_GObj) -> *mut CollData {
    melee_core::ct!("ftLib_80086984_fp");
    &mut fp(gobj).coll_data
}

/// `ftLib_80086B64_fp`.
pub unsafe fn ftLib_80086B64_fp(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_80086B64_fp");
    fp(gobj).x221F_b0() != 0
}

/// `ftLib_8008701C_fp`.
pub unsafe fn ftLib_8008701C_fp(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_8008701C_fp");
    fp(gobj).x221F_b3() != 0
}

/// `ftLib_8008731C`.
pub unsafe fn ftLib_8008731C(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_8008731C");
    ftLib_8008701C_fp(gobj) as i32
}

/// `ftLib_800872B0_fp`: the fighter's shadow record.
pub unsafe fn ftLib_800872B0_fp(gobj: *mut HSD_GObj) -> *mut LbShadow {
    melee_core::ct!("ftLib_800872B0_fp");
    &mut fp(gobj).x20A4
}

/// `ftLib_800874BC_fp`: is it a sub fighter (Nana).
pub unsafe fn ftLib_800874BC_fp(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_800874BC_fp");
    fp(gobj).is_sub_fighter() as i32
}

/// `ftLib_8008732C_fp`: in a death state.
pub unsafe fn ftLib_8008732C_fp(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_8008732C_fp");
    msid_in(gobj, ftCo_MS_DeadDown, ftCo_MS_DeadUpFallHitCameraIce)
}

/// `ftLib_800873F4`: in the entry animation.
pub unsafe fn ftLib_800873F4(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_800873F4");
    msid_in(gobj, ftCo_MS_Entry, ftCo_MS_EntryEnd)
}
