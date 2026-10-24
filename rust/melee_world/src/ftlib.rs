//! The plain `Fighter` field accessors of `src/melee/ft/ftlib.c` that the stage, HUD and effect
//! TUs call with a fighter `HSD_GObj*` (`gobj->user_data` is the `Fighter`), in the C shape (raw
//! pointers, C names). They live here, in the lowest crate that needs them; `melee_fighter::ftlib`
//! (which depends on this crate and ports the other ~90 functions of the file) forwards its same-named
//! accessors to these, so there is a single port of each. The C dereferences without checks and so
//! do these: pass a live fighter gobj. No float arithmetic.
//! Checked by the fighter's `tests/ftlib_ref.rs` (gen/cref, ftlib.c verbatim), which runs on them.
#![allow(non_snake_case, clippy::missing_safety_doc)]

use melee_types::*;

#[inline(always)]
unsafe fn fp<'a>(gobj: *mut HSD_GObj) -> &'a mut Fighter {
    &mut *((*gobj).user_data as *mut Fighter)
}

/// `ftLib_80086644`: the current position.
pub unsafe fn ftLib_80086644(gobj: *mut HSD_GObj, pos: *mut Vec) {
    *pos = fp(gobj).cur_pos;
}

/// `ftLib_80086684`: the previous position.
pub unsafe fn ftLib_80086684(gobj: *mut HSD_GObj, pos: *mut Vec) {
    *pos = fp(gobj).prev_pos;
}

/// `ftLib_80086B74`: the fighter's camera subject.
pub unsafe fn ftLib_80086B74(gobj: *mut HSD_GObj) -> *mut CmSubject {
    fp(gobj).x890_cameraBox
}

/// `ftLib_80086B80`: the vertical camera extent target of the subject.
pub unsafe fn ftLib_80086B80(gobj: *mut HSD_GObj) -> f32 {
    (*ftLib_80086B74(gobj)).target_ext.v.z
}

/// `ftLib_80086B90`: the subject's bone position.
pub unsafe fn ftLib_80086B90(gobj: *mut HSD_GObj, v: *mut Vec) {
    *v = (*ftLib_80086B74(gobj)).bone_pos;
}

/// `ftLib_80086BEC`: the position delta of the frame.
pub unsafe fn ftLib_80086BEC(gobj: *mut HSD_GObj, v: *mut Vec) {
    *v = fp(gobj).pos_delta;
}

/// `ftLib_80086BE0`: the player id.
pub unsafe fn ftLib_80086BE0(gobj: *mut HSD_GObj) -> u8 {
    fp(gobj).player_id
}

/// `ftLib_80086EC0`.
pub unsafe fn ftLib_80086EC0(gobj: *mut HSD_GObj) -> bool {
    fp(gobj).x221C_b6() != 0
}

/// `ftLib_80086960`: is `gobj` a fighter GObj.
pub unsafe fn ftLib_80086960(gobj: *mut HSD_GObj) -> bool {
    !gobj.is_null() && (*gobj).classifier as i32 == HSD_GOBJ_CLASS_FIGHTER
}

/// `ftLib_80086A4C`.
pub unsafe fn ftLib_80086A4C(gobj: *mut HSD_GObj, val: f32) {
    fp(gobj).dmg.x1958 = val;
}

/// `ftLib_80086984`: the fighter's collision data.
pub unsafe fn ftLib_80086984(gobj: *mut HSD_GObj) -> *mut CollData {
    &mut fp(gobj).coll_data
}

/// `ftLib_80086B64`.
pub unsafe fn ftLib_80086B64(gobj: *mut HSD_GObj) -> bool {
    fp(gobj).x221F_b0() != 0
}

/// `ftLib_8008701C`.
pub unsafe fn ftLib_8008701C(gobj: *mut HSD_GObj) -> bool {
    fp(gobj).x221F_b3() != 0
}

/// `ftLib_8008731C`.
pub unsafe fn ftLib_8008731C(gobj: *mut HSD_GObj) -> i32 {
    ftLib_8008701C(gobj) as i32
}

/// `ftLib_800872B0`: the fighter's shadow record.
pub unsafe fn ftLib_800872B0(gobj: *mut HSD_GObj) -> *mut LbShadow {
    &mut fp(gobj).x20A4
}

/// `ftLib_800874BC`: is it a sub fighter (Nana).
pub unsafe fn ftLib_800874BC(gobj: *mut HSD_GObj) -> i32 {
    fp(gobj).is_sub_fighter() as i32
}

#[inline]
unsafe fn msid_in(gobj: *mut HSD_GObj, lo: i32, hi: i32) -> bool {
    let m = fp(gobj).motion_id as i32;
    m >= lo && m <= hi
}

/// `ftLib_8008732C`: in a death state.
pub unsafe fn ftLib_8008732C(gobj: *mut HSD_GObj) -> bool {
    msid_in(gobj, ftCo_MS_DeadDown, ftCo_MS_DeadUpFallHitCameraIce)
}

/// `ftLib_800873F4`: in the entry animation.
pub unsafe fn ftLib_800873F4(gobj: *mut HSD_GObj) -> bool {
    msid_in(gobj, ftCo_MS_Entry, ftCo_MS_EntryEnd)
}
