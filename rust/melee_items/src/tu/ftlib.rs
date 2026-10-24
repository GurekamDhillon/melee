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

pub unsafe fn ftLib_80086960(arg: *mut HSD_GObj) -> bool {
    !arg.is_null() && (*arg).classifier as i32 == HSD_GOBJ_CLASS_FIGHTER as i32
}

pub unsafe fn ftLib_80086644(gobj: *mut HSD_GObj, pos: *mut Vec) {
    *pos = (*GET_FIGHTER(gobj)).cur_pos;
}

pub unsafe fn ftLib_800865C0(gobj: *mut HSD_GObj) -> f32 {
    (*GET_FIGHTER(gobj)).facing_dir
}

pub unsafe fn ftLib_80086630(gobj: *mut HSD_GObj, part: u32) -> *mut HSD_JObj {
    (*(*GET_FIGHTER(gobj)).parts.offset(part as isize)).joint
}

pub unsafe fn ftLib_800865CC(gobj: *mut HSD_GObj) -> i32 {
    (*GET_FIGHTER(gobj)).ground_or_air as i32
}

pub unsafe fn ftLib_80086BE0(gobj: *mut HSD_GObj) -> u8 {
    (*GET_FIGHTER(gobj)).player_id as u8
}

pub unsafe fn ftLib_80086EB4(gobj: *mut HSD_GObj) -> i32 {
    (*GET_FIGHTER(gobj)).team as i32
}

pub unsafe fn ftLib_80086A0C(gobj: *mut HSD_GObj) -> f32 {
    (*GET_FIGHTER(gobj)).x34_scale.y
}

pub unsafe fn ftLib_800865D8(gobj: *mut HSD_GObj, x: *mut f32, y: *mut f32) {
    let fp = GET_FIGHTER(gobj);
    *x = (*fp).input.lstick[0].x;
    *y = (*fp).input.lstick[0].y;
}

pub unsafe fn ftLib_800867A0(gobj: *mut HSD_GObj, arg1: *mut HSD_GObj) -> bool {
    let fp = GET_FIGHTER(gobj);
    (*fp).item_gobj == arg1 || (*fp).x1978 == arg1
}

pub unsafe fn ftLib_800867CC(gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    (*GET_FIGHTER(gobj)).x1984_heldItemSpec
}

pub unsafe fn ftLib_80086BEC(gobj: *mut HSD_GObj, v: *mut Vec) {
    *v = (*GET_FIGHTER(gobj)).pos_delta;
}
