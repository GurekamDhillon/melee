//! Helpers shared by the per-TU ports of `src/melee/gr` (game_tus.txt 585-592): the static inline
//! functions of gr/inlines.h and gr/ground.h that the stage TUs expand, with every callee outside
//! the TU going through the `tu_ext` hooks like the C goes through the linker.
#![allow(non_snake_case, static_mut_refs)]

use crate::tu_ext as ext;
use core::ffi::{c_char, c_void};
use melee_types::*;

/// `GET_GROUND(gobj)` = `HSD_GObjGetUserData`.
#[inline(always)]
pub unsafe fn GET_GROUND(gobj: *mut HSD_GObj) -> *mut Ground {
    (*gobj).user_data as *mut Ground
}

/// `GET_JOBJ(gobj)` = `HSD_GObjGetHSDObj`.
#[inline(always)]
pub unsafe fn GET_JOBJ(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

/// `Ground_StartMapAnim`: reset the map model and start animation set 0.
pub unsafe fn Ground_StartMapAnim(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// `Ground_InitMapCollAndAnim`.
pub unsafe fn Ground_InitMapCollAndAnim(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// `Ground_UpdateWindAndMapColl`.
pub unsafe fn Ground_UpdateWindAndMapColl(gobj: *mut HSD_GObj) {
    ext::lb_800115F4();
    ext::Ground_UpdateMapColl(gobj);
}

/// `Ground_SetupStageCallbacks` (TARGET_PC form: every callback goes through `Mex_GrBind`).
pub unsafe fn Ground_SetupStageCallbacks(gobj: *mut HSD_GObj, callbacks: *mut StageCallbacks) {
    let gp = GET_GROUND(gobj);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    ext::GObj_SetupGXLink(gobj, Some(ext::grDisplay_801C5DB0), 3, 0);
    if (*callbacks).callback3.is_some() {
        let r = ext::Mex_GrBind(core::mem::transmute::<_, *mut c_void>((*callbacks).callback3));
        (*gp).x1C_callback = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(r);
    }
    if (*callbacks).on_init.is_some() {
        let r = ext::Mex_GrBind(core::mem::transmute::<_, *mut c_void>((*callbacks).on_init));
        let f = core::mem::transmute::<*mut c_void, unsafe extern "C" fn(*mut HSD_GObj)>(r);
        f(gobj);
    }
    if (*callbacks).gobj_proc.is_some() {
        let r = ext::Mex_GrBind(core::mem::transmute::<_, *mut c_void>((*callbacks).gobj_proc));
        ext::HSD_GObj_SetupProc(gobj, core::mem::transmute::<*mut c_void, HSD_GObjEvent>(r), 4);
    }
}

/// `Ground_InitTargetStage(create_gobj)` (target-test stages).
pub unsafe fn Ground_InitTargetStage(create_gobj: unsafe extern "C" fn(i32) -> *mut HSD_GObj) {
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    create_gobj(0);
    create_gobj(1);
    create_gobj(2);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Ground_801C4210();
    ext::Ground_801C42AC();
}

/// `OSReport("%s:%d: couldn t get gobj(id=%d)\n", __FILE__, line, id)`, the message every
/// per-stage `...Create` function prints when `Ground_GetStageGObj` returns NULL.
pub unsafe fn report_no_gobj(file: *const c_char, line: i64, id: i32) {
    ext::OSReport(
        c"%s:%d: couldn t get gobj(id=%d)\n".as_ptr(),
        &[ext::OsArg::S(file), ext::OsArg::I(line), ext::OsArg::I(id as i64)],
    );
}

/// `randi` (sysdolphin/baselib/random.h): `HSD_Randi` that accepts an empty range.
#[inline]
pub unsafe fn randi(max_val: i32) -> i32 {
    if max_val != 0 {
        return ext::HSD_Randi(max_val);
    }
    0
}
