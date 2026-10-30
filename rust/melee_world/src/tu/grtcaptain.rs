//! src/melee/gr/grtcaptain.c: the Captain Falcon target-test stage.
//!
//! The constant/empty callbacks and the single-call forwarders already ported in
//! `gr::trivial` / `gr::forward` are re-exported; the rest is ported here. Callees outside this
//! TU are `tu_ext` hooks (unset: panic `TODO ... not ported`). State-oracle checked by
//! tests/tu_grtcaptain.rs against gen/turef/tu_grtcaptain_ref.c.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use super::gr_util::*;
use crate::tu_ext as ext;
use melee_types::*;

pub use crate::gr::forward::grtcaptain::{
    grTCaptain_OnStart, stageGObj0_OnInit, stageGObj1_GObjProc, stageGObj1_OnInit,
    stageGObj2_GObjProc, stageGObj2_OnInit,
};
pub use crate::gr::trivial::grtcaptain::{
    grTCaptain_8021FCFC, grTCaptain_8021FE18, grTCaptain_8021FE78, grTCaptain_8021FF08,
    grTCaptain_OnTouchLine,
};

pub unsafe extern "C" fn grTCaptain_OnDemoInit(_unused: i32) { melee_core::ct!("grTCaptain_OnDemoInit");}

pub unsafe extern "C" fn grTCaptain_OnInit() {
    melee_core::ct!("grTCaptain_OnInit");
    Ground_InitTargetStage(grTCaptain_8021FD04);
}

pub unsafe extern "C" fn grTCaptain_OnLoad() { melee_core::ct!("grTCaptain_OnLoad");}

pub unsafe extern "C" fn grTCaptain_8021FD04(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grTCaptain_8021FD04");
    let callbacks = (&raw mut grTCa_StageCallbacks as *mut StageCallbacks).add(gobj_id as usize);
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    } else {
        report_no_gobj(c"grtcaptain.c".as_ptr(), 215, gobj_id);
    }
    gobj
}

pub unsafe extern "C" fn grTCaptain_8021FE20(_arg0: *mut HSD_GObj) { melee_core::ct!("grTCaptain_8021FE20");}

pub unsafe extern "C" fn grTCaptain_8021FE24(_arg0: *mut HSD_GObj) { melee_core::ct!("grTCaptain_8021FE24");}

pub unsafe extern "C" fn grTCaptain_8021FEB4(_arg0: *mut HSD_GObj) { melee_core::ct!("grTCaptain_8021FEB4");}

pub unsafe extern "C" fn grTCaptain_8021FF30(_argo: *mut HSD_GObj) { melee_core::ct!("grTCaptain_8021FF30");}

pub unsafe extern "C" fn grTCaptain_OnCheckShadowRender(_arg0: *mut Vec3, _arg1: i32, _arg2: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grTCaptain_OnCheckShadowRender");
    1
}

pub static mut grTCa_StageCallbacks: [StageCallbacks; 4] = [
    StageCallbacks {
        on_init: Some(stageGObj0_OnInit),
        callback1: Some(grTCaptain_8021FE18),
        gobj_proc: Some(grTCaptain_8021FE20),
        callback3: Some(grTCaptain_8021FE24),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(stageGObj1_OnInit),
        callback1: Some(grTCaptain_8021FF08),
        gobj_proc: Some(stageGObj1_GObjProc),
        callback3: Some(grTCaptain_8021FF30),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(stageGObj2_OnInit),
        callback1: Some(grTCaptain_8021FE78),
        gobj_proc: Some(stageGObj2_GObjProc),
        callback3: Some(grTCaptain_8021FEB4),
        anon_10: StageCallbacks_anon_135 { flags: (1 << 30) | (1 << 31) },
    },
    StageCallbacks { on_init: None, callback1: None, gobj_proc: None, callback3: None, anon_10: StageCallbacks_anon_135 { flags: 0 } },
];

pub static mut grTCa_StageData: StageData = StageData {
    grkind: Gr_Kind_TCaptain,
    callbacks: unsafe { &raw mut grTCa_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrTCa.dat".as_ptr() as *mut i8,
    on_init: Some(grTCaptain_OnInit),
    on_demo_init: Some(grTCaptain_OnDemoInit),
    on_load: Some(grTCaptain_OnLoad),
    on_start: Some(grTCaptain_OnStart),
    callback4: Some(grTCaptain_8021FCFC),
    on_touch_line: Some(grTCaptain_OnTouchLine),
    on_check_shadow_render: Some(grTCaptain_OnCheckShadowRender),
    flags2: 1 << 0,
    joints: core::ptr::null_mut(),
    joint_count: 0,
};
