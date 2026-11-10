//! src/melee/gr/grtclink.c: the Young Link target-test stage.
//!
//! The constant/empty callbacks and the single-call forwarders already ported in
//! `gr::trivial` / `gr::forward` are re-exported; the rest is ported here. Callees outside this
//! TU are `tu_ext` hooks (unset: panic `TODO ... not ported`). State-oracle checked by
//! tests/tu_grtclink.rs against gen/turef/tu_grtclink_ref.c.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use super::gr_util::*;
use crate::tu_ext as ext;
use melee_types::*;

pub use crate::gr::forward::grtclink::{stageGObj1_GObjProc, stageGObj1_OnInit, stageGObj2_GObjProc, stageGObj2_OnInit};
pub unsafe extern "C" fn grTCLink_8021FF44(_unused: i32) { melee_core::ct!("grTCLink_8021FF44");}

pub unsafe extern "C" fn grTCLink_OnInit() {
    melee_core::ct!("grTCLink_OnInit");
    Ground_InitTargetStage(grTCLink_8021FFE8);
}

pub unsafe extern "C" fn grTclink_UnkStage0_OnLoad() { melee_core::ct!("grTclink_UnkStage0_OnLoad");}

pub unsafe extern "C" fn grTCLink_8021FFE8(id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grTCLink_8021FFE8");
    let callbacks = (&raw mut grTCLink_StageCallbacks as *mut StageCallbacks).add(id as usize);
    let gobj = ext::Ground_GetStageGObj(id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    } else {
        report_no_gobj(c"grtclink.c".as_ptr(), 195, id);
    }
    gobj
}

pub unsafe extern "C" fn grTCLink_802200D0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grTCLink_802200D0");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

pub unsafe extern "C" fn grTCLink_80220104(_gobj: *mut HSD_GObj) { melee_core::ct!("grTCLink_80220104");}

pub unsafe extern "C" fn grTCLink_80220108(_gobj: *mut HSD_GObj) { melee_core::ct!("grTCLink_80220108");}

pub unsafe extern "C" fn grTCLink_80220198(_gobj: *mut HSD_GObj) { melee_core::ct!("grTCLink_80220198");}

pub unsafe extern "C" fn grTCLink_80220214(_gobj: *mut HSD_GObj) { melee_core::ct!("grTCLink_80220214");}

pub static mut grTCLink_StageCallbacks: [StageCallbacks; 4] = [
    StageCallbacks {
        on_init: Some(grTCLink_802200D0),
        callback1: Some(grTCLink_802200FC),
        gobj_proc: Some(grTCLink_80220104),
        callback3: Some(grTCLink_80220108),
        x10: StageCallbacks_x10 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(stageGObj1_OnInit),
        callback1: Some(grTCLink_802201EC),
        gobj_proc: Some(stageGObj1_GObjProc),
        callback3: Some(grTCLink_80220214),
        x10: StageCallbacks_x10 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(stageGObj2_OnInit),
        callback1: Some(grTCLink_8022015C),
        gobj_proc: Some(stageGObj2_GObjProc),
        callback3: Some(grTCLink_80220198),
        x10: StageCallbacks_x10 { flags: u32::to_be((1 << 30) | (1 << 31)) },
    },
    StageCallbacks { on_init: None, callback1: None, gobj_proc: None, callback3: None, x10: StageCallbacks_x10 { flags: 0 } },
];

pub static mut grTCLink_StageData: StageData = StageData {
    grkind: Gr_Kind_TClink,
    callbacks: unsafe { &raw mut grTCLink_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrTCl.dat".as_ptr() as *mut i8,
    on_init: Some(grTCLink_OnInit),
    on_demo_init: Some(grTCLink_8021FF44),
    on_load: Some(grTclink_UnkStage0_OnLoad),
    on_start: Some(grTclink_UnkStage0_OnStart),
    callback4: Some(grTCLink_8021FFE0),
    on_touch_line: Some(grTCLink_80220218),
    on_check_shadow_render: Some(grTCLink_80220220),
    flags2: 1 << 0,
    joints: core::ptr::null_mut(),
    joint_count: 0,
};

// ---- folded from grtclink.rs (consolidation steps 21/22) ----
/// grTclink_UnkStage0_OnStart (gr/grtclink.c:67)
    pub unsafe extern "C" fn grTclink_UnkStage0_OnStart() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

// ---- folded from grtclink.rs (consolidation steps 21/22) ----
/// grTCLink_8021FFE0 (gr/grtclink.c:72)
    pub extern "C" fn grTCLink_8021FFE0() -> i32 { 0 }

/// grTCLink_802200FC (gr/grtclink.c:99)
    pub extern "C" fn grTCLink_802200FC(a0: *mut HSD_GObj) -> i32 { 0 }

/// grTCLink_8022015C (gr/grtclink.c:113)
    pub extern "C" fn grTCLink_8022015C(a0: *mut HSD_GObj) -> i32 { 0 }

/// grTCLink_802201EC (gr/grtclink.c:130)
    pub extern "C" fn grTCLink_802201EC(a0: *mut HSD_GObj) -> i32 { 0 }

/// grTCLink_80220218 (gr/grtclink.c:142)
    pub extern "C" fn grTCLink_80220218(a0: i32) -> *mut melee_types::lbColl_80008D30_arg1 { core::ptr::null_mut() }

/// grTCLink_80220220 (gr/grtclink.c:147)
    pub extern "C" fn grTCLink_80220220(a0: *mut Vec3, a1: i32, a2: *mut HSD_JObj) -> i32 { 1 }
