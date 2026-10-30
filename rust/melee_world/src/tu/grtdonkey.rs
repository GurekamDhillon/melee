//! src/melee/gr/grtdonkey.c: the Donkey Kong target-test stage.
//!
//! The constant/empty callbacks and the single-call forwarders already ported in
//! `gr::trivial` / `gr::forward` are re-exported; the rest is ported here. Callees outside this
//! TU are `tu_ext` hooks (unset: panic `TODO ... not ported`). State-oracle checked by
//! tests/tu_grtdonkey.rs against gen/turef/tu_grtdonkey_ref.c.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use super::gr_util::*;
use crate::tu_ext as ext;
use melee_types::*;

pub use crate::gr::forward::grtdonkey::{
    grTdonkey_UnkStage0_OnStart, stageGObj0_OnInit, stageGObj1_GObjProc, stageGObj1_OnInit,
    stageGObj2_GObjProc, stageGObj2_OnInit,
};
pub use crate::gr::trivial::grtdonkey::{
    grTDonkey_802202C4, grTDonkey_802203E0, grTDonkey_80220440, grTDonkey_802204D0,
    grTDonkey_802204FC, grTDonkey_80220504,
};

pub unsafe extern "C" fn grTDonkey_80220228(_arg0: i32) { melee_core::ct!("grTDonkey_80220228");}

pub unsafe extern "C" fn grTDonkey_OnInit() {
    melee_core::ct!("grTDonkey_OnInit");
    Ground_InitTargetStage(grTDonkey_802202CC);
}

pub unsafe extern "C" fn grTdonkey_UnkStage0_OnLoad() { melee_core::ct!("grTdonkey_UnkStage0_OnLoad");}

pub unsafe extern "C" fn grTDonkey_802202CC(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grTDonkey_802202CC");
    let callbacks = (&raw mut grTDk_StageCallbacks as *mut StageCallbacks).add(gobj_id as usize);
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    } else {
        report_no_gobj(c"grtdonkey.c".as_ptr(), 0xc3, gobj_id);
    }
    gobj
}

pub unsafe extern "C" fn grTDonkey_802203E8(_arg0: *mut HSD_GObj) { melee_core::ct!("grTDonkey_802203E8");}

pub unsafe extern "C" fn grTDonkey_802203EC(_arg0: *mut HSD_GObj) { melee_core::ct!("grTDonkey_802203EC");}

pub unsafe extern "C" fn grTDonkey_8022047C(_arg0: *mut HSD_GObj) { melee_core::ct!("grTDonkey_8022047C");}

pub unsafe extern "C" fn grTDonkey_802204F8(_arg0: *mut HSD_GObj) { melee_core::ct!("grTDonkey_802204F8");}

pub static mut grTDk_StageCallbacks: [StageCallbacks; 4] = [
    StageCallbacks {
        on_init: Some(stageGObj0_OnInit),
        callback1: Some(grTDonkey_802203E0),
        gobj_proc: Some(grTDonkey_802203E8),
        callback3: Some(grTDonkey_802203EC),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(stageGObj1_OnInit),
        callback1: Some(grTDonkey_802204D0),
        gobj_proc: Some(stageGObj1_GObjProc),
        callback3: Some(grTDonkey_802204F8),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(stageGObj2_OnInit),
        callback1: Some(grTDonkey_80220440),
        gobj_proc: Some(stageGObj2_GObjProc),
        callback3: Some(grTDonkey_8022047C),
        anon_10: StageCallbacks_anon_135 { flags: (1 << 30) | (1 << 31) },
    },
    StageCallbacks { on_init: None, callback1: None, gobj_proc: None, callback3: None, anon_10: StageCallbacks_anon_135 { flags: 0 } },
];

pub static mut grTDk_StageData: StageData = StageData {
    grkind: Gr_Kind_TDonkey,
    callbacks: unsafe { &raw mut grTDk_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrTDk.dat".as_ptr() as *mut i8,
    on_init: Some(grTDonkey_OnInit),
    on_demo_init: Some(grTDonkey_80220228),
    on_load: Some(grTdonkey_UnkStage0_OnLoad),
    on_start: Some(grTdonkey_UnkStage0_OnStart),
    callback4: Some(grTDonkey_802202C4),
    on_touch_line: Some(grTDonkey_802204FC),
    on_check_shadow_render: Some(grTDonkey_80220504),
    flags2: 1 << 0,
    joints: core::ptr::null_mut(),
    joint_count: 0,
};
