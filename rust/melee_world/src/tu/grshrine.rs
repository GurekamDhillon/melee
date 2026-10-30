//! src/melee/gr/grshrine.c: the Temple (Shrine) stage ("Yoshi's story shrine" 0x11).
//!
//! The constant/empty callbacks and the single-call forwarders already ported in `gr::trivial` /
//! `gr::forward` are re-exported; the rest is ported here. Callees outside this TU are `tu_ext`
//! hooks (unset: panic `TODO ... not ported`). State-oracle checked by tests/tu_grshrine.rs against
//! gen/turef/tu_grshrine_ref.c.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use super::gr_util::*;
use crate::tu_ext as ext;
use core::ffi::c_void;
use melee_types::*;

pub use crate::gr::forward::grshrine::{grShrine_UnkStage0_OnStart, stageGObj0_OnInit};
pub use crate::gr::trivial::grshrine::{
    grShrine_80201D18, grShrine_80201E34, grShrine_80201E88, grShrine_80201F14, grShrine_80201F44,
};

/// `static void* yakumono_param`.
pub static mut yakumono_param: *mut c_void = core::ptr::null_mut();

pub unsafe extern "C" fn grShrine_80201C60(_arg0: i32) { melee_core::ct!("grShrine_80201C60");}

pub unsafe extern "C" fn grShrine_80201C64() {
    melee_core::ct!("grShrine_80201C64");
    yakumono_param = ext::Ground_GetYakumonoParam();
    grShrine_80201D20(0);
    grShrine_80201D20(1);
    grShrine_80201D20(2);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    {
        let mut v = Vec3 { x: 0.5, y: 0.0, z: 0.0 };
        ext::lb_80011A50(
            &mut v,
            -1,
            0.5,
            0.0,
            (core::f64::consts::PI / 3.0) as f32,
            -10000.0,
            10000.0,
            10000.0,
            -10000.0,
        );
    }
}

pub unsafe extern "C" fn grShrine_UnkStage0_OnLoad() { melee_core::ct!("grShrine_UnkStage0_OnLoad");}

pub unsafe extern "C" fn grShrine_80201D20(arg0: i32) -> *mut HSD_GObj {
    melee_core::ct!("grShrine_80201D20");
    let callbacks = (&raw mut grSh_StageCallbacks as *mut StageCallbacks).add(arg0 as usize);
    let gobj = ext::Ground_GetStageGObj(arg0);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    } else {
        report_no_gobj(c"grshrine.c".as_ptr(), 205, arg0);
    }
    gobj
}

pub unsafe extern "C" fn grShrine_80201E3C(_arg0: *mut HSD_GObj) { melee_core::ct!("grShrine_80201E3C");}

pub unsafe extern "C" fn grShrine_80201E40(_arg0: *mut HSD_GObj) { melee_core::ct!("grShrine_80201E40");}

pub unsafe extern "C" fn grShrine_80201E44(gobj: *mut HSD_GObj) {
    melee_core::ct!("grShrine_80201E44");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grShrine_80201E90(_arg0: *mut HSD_GObj) { melee_core::ct!("grShrine_80201E90");}

pub unsafe extern "C" fn grShrine_80201E94(_arg0: *mut HSD_GObj) { melee_core::ct!("grShrine_80201E94");}

pub unsafe extern "C" fn grShrine_80201E98(_arg0: *mut HSD_GObj) { melee_core::ct!("grShrine_80201E98");}

pub unsafe extern "C" fn grShrine_80201E9C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grShrine_80201E9C");
    let gp = GET_GROUND(gobj);
    ext::Ground_InitMapColl((*gobj).hsd_obj as *mut HSD_JObj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::Ground_801C10B8(gobj, Some(grShrine_80201E98));
    ext::Ground_UpdateMapColl(gobj);
    (*gp).x10_flags.set_b5(1);
}

pub unsafe extern "C" fn grShrine_80201F1C(arg0: *mut HSD_GObj) {
    melee_core::ct!("grShrine_80201F1C");
    ext::Ground_UpdateMapColl(arg0);
    ext::lb_800115F4();
}

pub unsafe extern "C" fn grShrine_80201F40(_arg0: *mut HSD_GObj) { melee_core::ct!("grShrine_80201F40");}

pub unsafe extern "C" fn grShrine_80201F4C(a: *mut Vec3, _unused: i32, joint: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grShrine_80201F4C");
    let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(joint, core::ptr::null_mut(), &mut b);
    if (*a).y > b.y { 1 } else { 0 }
}

pub static mut grSh_StageCallbacks: [StageCallbacks; 3] = [
    StageCallbacks {
        on_init: Some(stageGObj0_OnInit),
        callback1: Some(grShrine_80201E34),
        gobj_proc: Some(grShrine_80201E3C),
        callback3: Some(grShrine_80201E40),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(grShrine_80201E44),
        callback1: Some(grShrine_80201E88),
        gobj_proc: Some(grShrine_80201E90),
        callback3: Some(grShrine_80201E94),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(grShrine_80201E9C),
        callback1: Some(grShrine_80201F14),
        gobj_proc: Some(grShrine_80201F1C),
        callback3: Some(grShrine_80201F40),
        anon_10: StageCallbacks_anon_135 { flags: (1 << 30) | (1 << 31) },
    },
];

pub static mut grSh_StageData: StageData = StageData {
    grkind: Gr_Kind_Shrine,
    callbacks: unsafe { &raw mut grSh_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrSh.dat".as_ptr() as *mut i8,
    on_init: Some(grShrine_80201C64),
    on_demo_init: Some(grShrine_80201C60),
    on_load: Some(grShrine_UnkStage0_OnLoad),
    on_start: Some(grShrine_UnkStage0_OnStart),
    callback4: Some(grShrine_80201D18),
    on_touch_line: Some(grShrine_80201F44),
    on_check_shadow_render: Some(grShrine_80201F4C),
    flags2: 1 << 0,
    joints: core::ptr::null_mut(),
    joint_count: 0,
};
