//! Native port of the leaf accessors of `src/melee/ft/ftlib.c` that item code calls. The functions only read
//! or write fields of the `Fighter` behind `gobj->user_data`, so they live here, below `melee_fighter`
//! (which owns the rest of ftlib.c), and `melee_items` TUs reach them without a dependency cycle.
//! Checked by tests/native_ftlib.rs against the C bodies (gen/native/ftlib_ref.c).
#![allow(non_snake_case, clippy::missing_safety_doc)]

use melee_types::*;

#[inline(always)]
unsafe fn GET_FIGHTER(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub unsafe fn ftLib_IsFighter(arg: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_IsFighter");
    !arg.is_null() && (*arg).classifier as i32 == HSD_GOBJ_CLASS_FIGHTER as i32
}

pub unsafe fn ftLib_GetPos(gobj: *mut HSD_GObj, pos: *mut Vec) {
    melee_core::ct!("ftLib_GetPos");
    *pos = (*GET_FIGHTER(gobj)).cur_pos;
}

pub unsafe fn ftLib_GetFacingDir(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_GetFacingDir");
    (*GET_FIGHTER(gobj)).facing_dir
}

pub unsafe fn ftLib_GetPartJObj(gobj: *mut HSD_GObj, part: u32) -> *mut HSD_JObj {
    melee_core::ct!("ftLib_GetPartJObj");
    (*(*GET_FIGHTER(gobj)).parts.offset(part as isize)).joint
}

pub unsafe fn ftLib_GetGroundAir(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_GetGroundAir");
    (*GET_FIGHTER(gobj)).ground_or_air as i32
}

pub unsafe fn ftLib_GetPlayerIndex(gobj: *mut HSD_GObj) -> u8 {
    melee_core::ct!("ftLib_GetPlayerIndex");
    (*GET_FIGHTER(gobj)).player_idx as u8
}

pub unsafe fn ftLib_GetTeam(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_GetTeam");
    (*GET_FIGHTER(gobj)).team as i32
}

pub unsafe fn ftLib_GetScale(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_GetScale");
    (*GET_FIGHTER(gobj)).x34_scale.y
}

pub unsafe fn ftLib_GetLStick(gobj: *mut HSD_GObj, x: *mut f32, y: *mut f32) {
    melee_core::ct!("ftLib_GetLStick");
    let fp = GET_FIGHTER(gobj);
    *x = (*fp).input.lstick[0].x;
    *y = (*fp).input.lstick[0].y;
}

pub unsafe fn ftLib_IsHoldingItem(gobj: *mut HSD_GObj, arg1: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftLib_IsHoldingItem");
    let fp = GET_FIGHTER(gobj);
    (*fp).item_gobj == arg1 || (*fp).x1978 == arg1
}

pub unsafe fn ftLib_GetHeldSpecialItem(gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    melee_core::ct!("ftLib_GetHeldSpecialItem");
    (*GET_FIGHTER(gobj)).x1984_heldItemSpec
}

pub unsafe fn ftLib_GetPosDelta(gobj: *mut HSD_GObj, v: *mut Vec) {
    melee_core::ct!("ftLib_GetPosDelta");
    *v = (*GET_FIGHTER(gobj)).pos_delta;
}

pub unsafe fn ftLib_GetMotionId(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_GetMotionId");
    (*GET_FIGHTER(gobj)).motion_id as i32
}

/// `ftCommon_GetModelScale` (ftcommon.c): `fp->x34_scale.y * fp->co_attrs.model_scaling`.
pub unsafe fn ftCommon_GetModelScale(fp: *mut Fighter) -> f32 {
    melee_core::ct!("ftCommon_GetModelScale");
    (*fp).x34_scale.y * (*fp).co_attrs.model_scaling
}

pub unsafe fn ftLib_GetModelScale(gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("ftLib_GetModelScale");
    ftCommon_GetModelScale(GET_FIGHTER(gobj))
}
