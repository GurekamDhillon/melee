//! src/melee/gr/grtclink.c: the Young Link target-test stage.
//!
//! The constant/empty callbacks and the single-call forwarders already ported in
//! `gr::trivial` / `gr::forward` are re-exported; the rest is ported here. Callees outside this
//! TU are `tu_ext` hooks (unset: panic `TODO ... not ported`). State-oracle checked by
//! tests/tu_grtclink.rs against gen/turef/tu_grtclink_ref.c.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use super::gr_util::*;
use crate::tu_ext as ext;
use melee_types::*;

pub use crate::gr::forward::grtclink::{
    grTclink_UnkStage0_OnStart, stageGObj1_GObjProc, stageGObj1_OnInit, stageGObj2_GObjProc,
    stageGObj2_OnInit,
};
pub use crate::gr::trivial::grtclink::{
    grTCLink_8021FFE0, grTCLink_802200FC, grTCLink_8022015C, grTCLink_802201EC, grTCLink_80220218,
    grTCLink_80220220,
};

pub unsafe extern "C" fn grTCLink_8021FF44(_unused: i32) {}

pub unsafe extern "C" fn grTCLink_OnInit() {
    Ground_InitTargetStage(grTCLink_8021FFE8);
}

pub unsafe extern "C" fn grTclink_UnkStage0_OnLoad() {}

pub unsafe extern "C" fn grTCLink_8021FFE8(id: i32) -> *mut HSD_GObj {
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
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

pub unsafe extern "C" fn grTCLink_80220104(_gobj: *mut HSD_GObj) {}

pub unsafe extern "C" fn grTCLink_80220108(_gobj: *mut HSD_GObj) {}

pub unsafe extern "C" fn grTCLink_80220198(_gobj: *mut HSD_GObj) {}

pub unsafe extern "C" fn grTCLink_80220214(_gobj: *mut HSD_GObj) {}

pub static mut grTCLink_StageCallbacks: [StageCallbacks; 4] = [
    StageCallbacks {
        on_init: Some(grTCLink_802200D0),
        callback1: Some(grTCLink_802200FC),
        gobj_proc: Some(grTCLink_80220104),
        callback3: Some(grTCLink_80220108),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(stageGObj1_OnInit),
        callback1: Some(grTCLink_802201EC),
        gobj_proc: Some(stageGObj1_GObjProc),
        callback3: Some(grTCLink_80220214),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(stageGObj2_OnInit),
        callback1: Some(grTCLink_8022015C),
        gobj_proc: Some(stageGObj2_GObjProc),
        callback3: Some(grTCLink_80220198),
        anon_10: StageCallbacks_anon_135 { flags: (1 << 30) | (1 << 31) },
    },
    StageCallbacks { on_init: None, callback1: None, gobj_proc: None, callback3: None, anon_10: StageCallbacks_anon_135 { flags: 0 } },
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
