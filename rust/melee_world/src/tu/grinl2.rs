//! More src/melee/gr/inlines.h helpers the stage TUs share (`grinl` has the jobj accessors):
//! `Ground_StartMapAnim`, `Ground_InitMapCollAndAnim`, `Ground_UpdateWindAndMapColl`,
//! `Ground_SetupStageCallbacks`, `Ground_ClampSymmetric`, `Ground_WrapAngle`, `ZRANDI`, `rand_range`.
//! Their callees are the hooks below; every stage TU's `ext` re-exports the ones it uses.
//!
//! `Ground_SetupStageCallbacks` has a TARGET_PC branch that passes each callback through
//! `Mex_GrBind` (an m-ex guest address -> native thunk; identity for a native pointer). The native
//! port has no guest addresses, so the callbacks are used directly.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

use super::grinl::{GET_GROUND, M_TAU};
use melee_types::*;

pub mod ext {
    use crate::tu::hookmac::hook;
    use melee_types::*;
    hook!(GOBJ_SETUPGXLINK, GObj_SetupGXLink, (gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32));
    /// Hook behind [`grDisplay_801C5DB0`] for the TUs that call it directly (grizumi's render callbacks).
    pub static mut GRDISPLAY_801C5DB0: Option<fn(*mut HSD_GObj, i32)> = None;
    /// `grDisplay_801C5DB0`: an `extern "C"` function so it can be stored in `GObj_RenderFunc`
    /// (most TUs only pass it as a render callback); a direct call goes to the hook above.
    #[allow(non_snake_case)]
    pub unsafe extern "C" fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) {
        match GRDISPLAY_801C5DB0 {
            Some(f) => f(a0, a1),
            None => { unsafe { let _ = crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } return; },
        }
    }
    hook!(HSD_GOBJ_SETUPPROC, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc);
    pub static mut GRANIME_801C8138: Option<fn(*mut HSD_GObj, enum_t, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C8138(a0: *mut HSD_GObj, a1: enum_t, a2: i32) {
        match unsafe { GRANIME_801C8138 } { Some(f) => f(a0, a1, a2), None => { unsafe { let _ = crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GROUND_INITMAPCOLL: Option<fn(*mut HSD_JObj, s32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_InitMapColl(jobj: *mut HSD_JObj, map_id: s32) -> i32 {
        match unsafe { GROUND_INITMAPCOLL } { Some(f) => f(jobj, map_id), None => { return unsafe { crate::tu::ground::Ground_InitMapColl(jobj as _, map_id as _) as _ }; } }
    }
    pub static mut GROUND_UPDATEMAPCOLL: Option<fn(*mut HSD_GObj) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 {
        match unsafe { GROUND_UPDATEMAPCOLL } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ }; } }
    }
    pub static mut LB_800115F4: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn lb_800115F4() {
        match unsafe { LB_800115F4 } { Some(f) => f(), None => { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800115F4(); } return; } }
    }
    hook!(HSD_RANDI, HSD_Randi, (max_val: s32) -> s32);
}

/// `Ground_StartMapAnim`.
pub unsafe fn Ground_StartMapAnim(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
}

/// `Ground_InitMapCollAndAnim`.
pub unsafe fn Ground_InitMapCollAndAnim(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as s32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
}

/// `Ground_UpdateWindAndMapColl`.
pub unsafe fn Ground_UpdateWindAndMapColl(gobj: *mut HSD_GObj) {
    ext::lb_800115F4();
    ext::Ground_UpdateMapColl(gobj);
}

/// `Ground_SetupStageCallbacks`.
pub unsafe fn Ground_SetupStageCallbacks(gobj: *mut HSD_GObj, callbacks: *mut StageCallbacks) {
    let gp = GET_GROUND(gobj);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    ext::GObj_SetupGXLink(gobj, Some(ext::grDisplay_801C5DB0), 3, 0);
    if (*callbacks).callback3.is_some() {
        (*gp).x1C_callback = (*callbacks).callback3;
    }
    if let Some(f) = (*callbacks).on_init {
        f(gobj);
    }
    if (*callbacks).gobj_proc.is_some() {
        ext::HSD_GObj_SetupProc(gobj, (*callbacks).gobj_proc, 4);
    }
}

/// `Ground_ClampSymmetric`.
#[inline]
pub fn Ground_ClampSymmetric(value: f32, limit: f32, out: &mut f32) {
    let mut limit = limit;
    if value > limit {
        *out = limit;
    } else {
        limit = -limit;
        if value < limit {
            *out = limit;
        }
    }
}

/// `Ground_WrapAngle` (`M_TAU` is a double: the compare and the add are in f64 like the C).
#[inline]
pub fn Ground_WrapAngle(angle: &mut f32) {
    if (*angle as f64) > M_TAU {
        *angle = (*angle as f64 - M_TAU) as f32;
    } else if (*angle as f64) < -M_TAU {
        *angle = (*angle as f64 + M_TAU) as f32;
    }
}

/// `ZRANDI(n)`.
#[inline]
pub unsafe fn ZRANDI(n: i32) -> i32 {
    if n != 0 { ext::HSD_Randi(n) } else { 0 }
}

/// `rand_range(a, b)`.
pub unsafe fn rand_range(a: i32, b: i32) -> i32 {
    if a > b {
        b + ZRANDI(a - b)
    } else if a < b {
        a + ZRANDI(b - a)
    } else {
        a
    }
}
