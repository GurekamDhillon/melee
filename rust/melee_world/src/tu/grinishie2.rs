//! src/melee/gr/grinishie2.c: Mushroom Kingdom II / "Inishie 2" (Gr_Kind_Inishie2): the two dropping
//! platforms (gobj kinds 3 / 4), the bonus-ball like object (kind 1) and the Birdo-like walker
//! (kinds 15 / 3). State is the `yakumono_param` static, the per-gobj `Ground`
//! (`u.inishie2` / `u.inishie22` / `u.inishie23`, three overlays of the same words), `stage_info`
//! and the callback / stage-data tables; every other callee is a TODO hook in [`ext`].
//! No float arithmetic beyond plain adds (`fma_sites.py`: none fused).
//! State-oracle checked against the C (tests/gro_grinishie2.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use super::grinl::{
    GET_GROUND, GET_JOBJ, HSD_JObjAddTranslationX, HSD_JObjAddTranslationY, HSD_JObjAddTranslationZ,
    HSD_JObjGetTranslation, HSD_JObjSetTranslate,
};
use super::grinl2::{Ground_InitMapCollAndAnim, Ground_StartMapAnim};
use core::ptr::null_mut;
use melee_types::*;

pub use crate::gr::trivial::grinishie2::{
    grInishie2_801FCCD4, grInishie2_801FCDF4, grInishie2_801FCF24, grInishie2_801FD21C, grInishie2_801FD64C,
    grInishie2_801FDED0, grInishie2_801FDFE8, grInishie2_801FDFF0,
};

/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use melee_types::*;
    pub use crate::tu::grinl::ext::{stage_info, HSD_JObjSetMtxDirtySub};
    pub use crate::tu::grinl2::ext::{
        grAnime_801C8138, grDisplay_801C5DB0, GObj_SetupGXLink, Ground_InitMapColl, Ground_UpdateMapColl,
        HSD_GObj_SetupProc, HSD_Randi, lb_800115F4,
    };
    hook_or!(CAMERA_800311CC, Camera_800311CC, (a0: f32), |a0: f32| crate::cm::camera::world().camera_800311cc(a0));
    hook_or!(CAMERA_800311DC, Camera_800311DC, (a0: f32), |a0: f32| crate::cm::camera::world().camera_800311dc(a0));
    hook_or!(GETMATCHTIMER, GetMatchTimer, (a0: *mut i32) -> i32, |a0: *mut i32| unsafe { crate::gm::GetMatchTimer(a0 as _) as _ });
    pub static mut GROUND_801C39C0: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C39C0() {
        match unsafe { GROUND_801C39C0 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::ground::Ground_801C39C0(); } return; } }
    }
    pub static mut GROUND_801C3BB4: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3BB4() {
        match unsafe { GROUND_801C3BB4 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::ground::Ground_801C3BB4(); } return; } }
    }
    pub static mut GROUND_801C3FA4: Option<fn(*mut HSD_GObj, i32) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3FA4(gobj: *mut HSD_GObj, depth: i32) -> *mut HSD_JObj {
        match unsafe { GROUND_801C3FA4 } { Some(f) => f(gobj, depth), None => { return unsafe { crate::tu::ground::Ground_801C3FA4(gobj as _, depth as _) as _ }; } }
    }
    pub static mut GROUND_801C4A08: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C4A08(gobj: *mut HSD_GObj) {
        match unsafe { GROUND_801C4A08 } { Some(f) => f(gobj), None => { unsafe { let _ = crate::tu::ground::Ground_801C4A08(gobj as _); } return; } }
    }
    pub static mut GROUND_801C4E70: Option<fn(*mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C4E70(a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj) {
        match unsafe { GROUND_801C4E70 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { unsafe { let _ = crate::tu::ground::Ground_801C4E70(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } return; } }
    }
    pub static mut GROUND_801C5A94: Option<fn() -> s32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5A94() -> s32 {
        match unsafe { GROUND_801C5A94 } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_801C5A94() as _ }; } }
    }
    pub static mut GROUND_GETSTAGEGOBJ: Option<fn(i32) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj {
        match unsafe { GROUND_GETSTAGEGOBJ } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ }; } }
    }
    pub static mut GROUND_GETYAKUMONOPARAM: Option<fn() -> *mut core::ffi::c_void> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetYakumonoParam() -> *mut core::ffi::c_void {
        match unsafe { GROUND_GETYAKUMONOPARAM } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ }; } }
    }
    hook_or!(OSREPORT, OSReport, (a0: *mut i8), |a0: *mut i8| unsafe { melee_runtime::tu::os_error::OSReport(a0 as _, &[]) });
    pub static mut STAGE_GETBLASTZONELEFTOFFSET: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Stage_GetBlastZoneLeftOffset() -> f32 {
        match unsafe { STAGE_GETBLASTZONELEFTOFFSET } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_GetBlastZoneLeftOffset() as _ }; } }
    }
    pub static mut STAGE_GETBLASTZONERIGHTOFFSET: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Stage_GetBlastZoneRightOffset() -> f32 {
        match unsafe { STAGE_GETBLASTZONERIGHTOFFSET } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_GetBlastZoneRightOffset() as _ }; } }
    }
    pub static mut STAGE_GETCAMBOUNDSLEFTOFFSET: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Stage_GetCamBoundsLeftOffset() -> f32 {
        match unsafe { STAGE_GETCAMBOUNDSLEFTOFFSET } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_GetCamBoundsLeftOffset() as _ }; } }
    }
    pub static mut STAGE_GETCAMBOUNDSRIGHTOFFSET: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Stage_GetCamBoundsRightOffset() -> f32 {
        match unsafe { STAGE_GETCAMBOUNDSRIGHTOFFSET } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_GetCamBoundsRightOffset() as _ }; } }
    }
    pub static mut GRANIME_801C7FF8: Option<fn(*mut HSD_GObj, i32, i32, i32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) {
        match unsafe { GRANIME_801C7FF8 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { unsafe { let _ = crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } return; } }
    }
    pub static mut GRANIME_801C83D0: Option<fn(*mut HSD_GObj, i32, enum_t) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: enum_t) -> i32 {
        match unsafe { GRANIME_801C83D0 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRZAKOGENERATOR_801CAE04: Option<fn(*mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj {
        match unsafe { GRZAKOGENERATOR_801CAE04 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ }; } }
    }
    hook_or!(IT_802ECD1C, it_802ECD1C, (a0: *mut HSD_GObj, a1: *mut Vec3), |a0: *mut HSD_GObj, a1: *mut Vec3| { unsafe { let _ = crate::itacc::it_802ECD1C(a0 as _, a1 as _); } });
    hook!(IT_802ECD3C, it_802ECD3C, (a0: *mut HSD_GObj, a1: *mut Vec3, a2: f32) -> *mut HSD_GObj);
    pub static mut LBAUDIOAX_80023F28: Option<fn(i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn lbAudioAx_80023F28(a0: i32) -> i32 {
        match unsafe { LBAUDIOAX_80023F28 } { Some(f) => f(a0), None => { return unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0 as _) as _ }; } }
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    pub static mut MPJOINTLISTADD: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpJointListAdd(joint_id: i32) {
        match unsafe { MPJOINTLISTADD } { Some(f) => f(joint_id), None => { unsafe { let _ = crate::tu::mplib::mpJointListAdd(joint_id as _); } return; } }
    }
    pub static mut MPLIB_80057BC0: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_80057BC0(joint_id: i32) {
        match unsafe { MPLIB_80057BC0 } { Some(f) => f(joint_id), None => { unsafe { let _ = crate::tu::mplib::mpLib_80057BC0(joint_id as _); } return; } }
    }
}

/// `GrJoint grI2_803E4A60[]`.
pub static mut grI2_803E4A60: [GrJoint; 14] = [
    GrJoint { x: 0, y: 1, z: 1 },
    GrJoint { x: 12, y: 3, z: 1 },
    GrJoint { x: 13, y: 4, z: 1 },
    GrJoint { x: 14, y: 15, z: 0 },
    GrJoint { x: 3, y: 5, z: 0 },
    GrJoint { x: 4, y: 6, z: 0 },
    GrJoint { x: 5, y: 7, z: 0 },
    GrJoint { x: 6, y: 8, z: 0 },
    GrJoint { x: 7, y: 9, z: 0 },
    GrJoint { x: 8, y: 10, z: 0 },
    GrJoint { x: 9, y: 11, z: 0 },
    GrJoint { x: 10, y: 12, z: 0 },
    GrJoint { x: 11, y: 13, z: 0 },
    GrJoint { x: 2, y: 14, z: 0 },
];

macro_rules! cbs {
    ($a:expr, $b:expr, $c:expr, $d:expr, $flags:expr) => {
        StageCallbacks {
            on_init: Some($a),
            callback1: Some($b),
            gobj_proc: Some($c),
            callback3: Some($d),
            anon_10: StageCallbacks_anon_135 { flags: $flags },
        }
    };
}

const NO_CB: StageCallbacks = StageCallbacks {
    on_init: None,
    callback1: None,
    gobj_proc: None,
    callback3: None,
    anon_10: StageCallbacks_anon_135 { flags: 0 },
};

/// `StageCallbacks grI2_StageCallbacks[]` (16 entries; 5..14 are empty).
pub static mut grI2_StageCallbacks: [StageCallbacks; 16] = [
    cbs!(stageGObj0_OnInit, grInishie2_801FCDF4, grInishie2_801FCDFC, grInishie2_801FCE00, 0),
    cbs!(grInishie2_801FDE8C, grInishie2_801FDED0, grInishie2_801FDED8, grInishie2_801FDFE4, 0),
    cbs!(grInishie2_801FCE04, grInishie2_801FCF24, grInishie2_801FCFCC, grInishie2_801FD014, 0xC000_0000),
    cbs!(grInishie2_801FD198, grInishie2_801FD21C, grInishie2_801FD224, grInishie2_801FD364, 0),
    cbs!(grInishie2_801FD198, grInishie2_801FD21C, grInishie2_801FD224, grInishie2_801FD364, 0),
    NO_CB,
    NO_CB,
    NO_CB,
    NO_CB,
    NO_CB,
    NO_CB,
    NO_CB,
    NO_CB,
    NO_CB,
    NO_CB,
    cbs!(grInishie2_801FD4F0, grInishie2_801FD64C, grInishie2_801FD654, grInishie2_801FD740, 0),
];

/// `StageData grI2_StageData`.
pub static mut grI2_StageData: StageData = StageData {
    grkind: Gr_Kind_Inishie2,
    callbacks: unsafe { core::ptr::addr_of!(grI2_StageCallbacks) as *mut StageCallbacks },
    data1: b"/GrI2.dat\0".as_ptr() as *mut i8,
    on_init: Some(grInishie2_801FCBC4),
    on_demo_init: Some(grInishie2_801FCBC0),
    on_load: Some(grInishie2_801FCCAC),
    on_start: Some(grInishie2_801FCCB0),
    callback4: Some(grInishie2_801FCCD4),
    on_touch_line: Some(grInishie2_801FDFE8),
    on_check_shadow_render: Some(grInishie2_801FDFF0),
    flags2: 1 << 0,
    joints: unsafe { core::ptr::addr_of!(grI2_803E4A60) as *mut GrJoint },
    joint_count: 14,
};

/// `struct grInishie2_YakumonoParam` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grInishie2_YakumonoParam {
    pub unk0: i16,
    pub unk2: i16,
    pub unk4: i16,
    pub unk6: i16,
    pub unk8: i16,
    pub unk_a: i16,
    pub unk_c: i16,
    pub unk_e: i16,
    pub unk10: [i16; 2],
    pub unk14: [Vec3; 2],
    pub unk2c: f32,
    pub unk30: [Vec3; 2],
    pub unk48: i16,
    pub unk4a: i16,
}

/// `static struct grInishie2_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grInishie2_YakumonoParam = null_mut();

/// `test_random(val)`: `((val != 0) ? HSD_Randi(val) : 0)`.
#[inline(always)]
unsafe fn test_random(val: i32) -> i32 {
    if val != 0 { ext::HSD_Randi(val) } else { 0 }
}

pub unsafe extern "C" fn grInishie2_801FCBC0(_arg: i32) { melee_core::ct!("grInishie2_801FCBC0");}

pub unsafe extern "C" fn grInishie2_801FCBC4() {
    melee_core::ct!("grInishie2_801FCBC4");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grInishie2_YakumonoParam;

    let si = ext::stage_info();
    (*si).unk8C.set_b4(0);
    (*si).unk8C.set_b5(1);

    grInishie2_801FCCDC(0);
    grInishie2_801FCCDC(2);

    for id in [0, 0xC, 0xD, 0xE, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 2] {
        ext::mpLib_80057BC0(id);
    }

    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Camera_800311DC(1.0);
    ext::Camera_800311CC(4000.0);
}

pub unsafe extern "C" fn grInishie2_801FCCAC() { melee_core::ct!("grInishie2_801FCCAC");}

pub unsafe extern "C" fn grInishie2_801FCCB0() {
    melee_core::ct!("grInishie2_801FCCB0");
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe fn grInishie2_801FCCDC(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grInishie2_801FCCDC");
    let callbacks: *mut StageCallbacks = &mut grI2_StageCallbacks[gobj_id as usize];
    let gobj = ext::Ground_GetStageGObj(gobj_id);

    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
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
    } else {
        let mut msg = *b"%s:%d: couldn t get gobj(id=%d)\n\0";
        ext::OSReport(msg.as_mut_ptr() as *mut i8);
    }

    gobj
}

/// `static void stageGObj0_OnInit(Ground_GObj*)`.
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj0_OnInit");
    Ground_StartMapAnim(gobj);
}

pub unsafe extern "C" fn grInishie2_801FCDFC(_gobj: *mut HSD_GObj) { melee_core::ct!("grInishie2_801FCDFC");}

pub unsafe extern "C" fn grInishie2_801FCE00(_gobj: *mut HSD_GObj) { melee_core::ct!("grInishie2_801FCE00");}

pub unsafe extern "C" fn grInishie2_801FCE04(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FCE04");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C7FF8(gobj, 1, 7, 0, 0.0, 1.0);
    ext::grAnime_801C7FF8(gobj, 6, 7, 1, 0.0, 1.0);
    ext::grAnime_801C7FF8(gobj, 5, 7, 2, 0.0, 1.0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    grInishie2_801FD018(gobj);
    grInishie2_801FD368(gobj);
    grInishie2_801FD744(gobj);
    // Evaluation order of the six arguments: left to right (the shipping PPC build).
    let a0 = ext::Ground_801C3FA4(gobj, 9);
    let a1 = ext::Ground_801C3FA4(gobj, 0xC);
    let a2 = ext::Ground_801C3FA4(gobj, 8);
    let a3 = ext::Ground_801C3FA4(gobj, 0xB);
    let a4 = ext::Ground_801C3FA4(gobj, 0xA);
    let a5 = ext::Ground_801C3FA4(gobj, 0xD);
    ext::Ground_801C4E70(a0, a1, a2, a3, a4, a5);
    (*gp).u.inishie2.xC4_flags.set_b0(0);
}

pub unsafe fn grInishie2_801FCF2C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FCF2C");
    let gp = GET_GROUND(gobj);
    let mut time_remaining: u32 = 0;
    if (*gp).u.inishie2.xC4_flags.b0() == 0 {
        if ext::GetMatchTimer(&mut time_remaining as *mut u32 as *mut i32) != 0 {
            if time_remaining < 20 {
                let temp_r3 = ext::Ground_801C5A94();
                let var_r3 = if temp_r3 == 0x29 {
                    0x2A
                } else if temp_r3 == 0x2B {
                    0x2C
                } else {
                    -1
                };
                ext::lbAudioAx_80023F28(var_r3);
                (*gp).u.inishie2.xC4_flags.set_b0(1);
            }
        } else {
            (*gp).u.inishie2.xC4_flags.set_b0(1);
        }
    }
}

pub unsafe extern "C" fn grInishie2_801FCFCC(arg0: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FCFCC");
    grInishie2_801FD0B4(arg0);
    grInishie2_801FD3CC(arg0);
    grInishie2_801FD7A8(arg0);
    grInishie2_801FCF2C(arg0);
    ext::lb_800115F4();
}

pub unsafe extern "C" fn grInishie2_801FD014(_gobj: *mut HSD_GObj) { melee_core::ct!("grInishie2_801FD014");}

pub unsafe fn grInishie2_801FD018(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD018");
    let gp = (*gobj).user_data as *mut Ground;

    (*gp).u.inishie2.xC4_flags.set_b1(1);
    (*gp).u.inishie2.xC4_flags.set_b2(1);
    (*gp).u.inishie2.xC6 = ((*yakumono_param).unk0 as i32 + test_random((*yakumono_param).unk2 as i32)) as i16;
    (*gp).u.inishie2.xC8 = ((*yakumono_param).unk4 as i32 + test_random((*yakumono_param).unk6 as i32)) as i16;
}

pub unsafe fn grInishie2_801FD0B4(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD0B4");
    let gp = GET_GROUND(gobj);

    if (*gp).u.inishie2.xC4_flags.b1() != 0 {
        let timer = (*gp).u.inishie2.xC6;
        (*gp).u.inishie2.xC6 = timer.wrapping_sub(1);
        if timer < 0 {
            let new_gobj = grInishie2_801FCCDC(3);
            ext::mpLib_80057BC0(12);
            (*gp).u.inishie2.xC4_flags.set_b1(0);
            if !new_gobj.is_null() {
                let new_gp = GET_GROUND(new_gobj);
                (*new_gp).u.inishie22.xCC = gobj;
            }
        }
    }

    if (*gp).u.inishie2.xC4_flags.b2() != 0 {
        let timer = (*gp).u.inishie2.xC8;
        (*gp).u.inishie2.xC8 = timer.wrapping_sub(1);
        if timer < 0 {
            let new_gobj = grInishie2_801FCCDC(4);
            ext::mpLib_80057BC0(13);
            (*gp).u.inishie2.xC4_flags.set_b2(0);
            if !new_gobj.is_null() {
                let new_gp = GET_GROUND(new_gobj);
                (*new_gp).u.inishie22.xCC = gobj;
            }
        }
    }
}

pub unsafe extern "C" fn grInishie2_801FD198(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD198");
    let gp = GET_GROUND(gobj);

    Ground_InitMapCollAndAnim(gobj);

    (*gp).u.inishie2.xC4_flags.set_b0(0);

    (*gp).u.inishie2.xC8 = ((*gp).map_id as i32 - 3) as i16;
    (*gp).u.inishie2.xC6 = (*yakumono_param).unk10[(*gp).u.inishie2.xC8 as usize];
}

pub unsafe extern "C" fn grInishie2_801FD224(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD224");
    let mut gp = GET_GROUND(gobj);

    if (*gp).u.inishie2.xC4_flags.b0() == 0 {
        let counter = (*gp).u.inishie2.xC6;
        (*gp).u.inishie2.xC6 = counter.wrapping_sub(1);

        if counter < 0 {
            if (*gp).u.inishie2.xC8 == 0 {
                ext::mpJointListAdd(0xC);
            } else {
                ext::mpJointListAdd(0xD);
            }

            (*gp).u.inishie2.xC4_flags.set_b0(1);
        }
    }

    ext::Ground_UpdateMapColl(gobj);

    gp = GET_GROUND(gobj);
    if ext::grAnime_801C83D0(gobj, 0, 1) != 0 {
        let gp_2 = GET_GROUND((*gp).u.inishie22.xCC);
        if (*gp).u.inishie2.xC8 == 0 {
            (*gp_2).u.inishie2.xC4_flags.set_b1(1);
            (*gp_2).u.inishie2.xC6 = ((*yakumono_param).unk0 as i32 + test_random((*yakumono_param).unk2 as i32)) as i16;
        } else {
            (*gp_2).u.inishie2.xC4_flags.set_b2(1);
            (*gp_2).u.inishie2.xC8 = ((*yakumono_param).unk4 as i32 + test_random((*yakumono_param).unk6 as i32)) as i16;
        }

        ext::Ground_801C4A08(gobj);
    }
}

pub unsafe extern "C" fn grInishie2_801FD364(_gobj: *mut HSD_GObj) { melee_core::ct!("grInishie2_801FD364");}

pub unsafe fn grInishie2_801FD368(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD368");
    let gp = (*gobj).user_data as *mut Ground;

    (*gp).u.inishie2.xC4_flags.set_b3(1);

    let rand_max = (*yakumono_param).unk_a as i32;
    let rand_val = test_random(rand_max);

    (*gp).u.inishie2.xCA = ((*yakumono_param).unk8 as i32 + rand_val) as i16;
}

pub unsafe fn grInishie2_801FD3CC(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD3CC");
    let gp = GET_GROUND(gobj);

    if (*gp).u.inishie2.xC4_flags.b3() != 0 {
        let counter = (*gp).u.inishie2.xCA;
        (*gp).u.inishie2.xCA = counter.wrapping_sub(1);
        if counter < 0 {
            let gobj_2 = grInishie2_801FCCDC(0xF);
            (*gp).u.inishie2.xC4_flags.set_b3(0);
            if !gobj_2.is_null() {
                let gp_2 = (*gobj_2).user_data as *mut Ground;
                (*gp_2).u.inishie22.xC8 = gobj;
            }
        }
    }
}

pub unsafe fn grInishie2_801FD448(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD448");
    let gp = GET_GROUND(gobj);
    let gp_2 = GET_GROUND((*gp).u.inishie22.xC8);

    (*gp_2).u.inishie2.xC4_flags.set_b3(1);

    let rand_max = (*yakumono_param).unk_a as i32;
    let rand_val = test_random(rand_max);

    (*gp_2).u.inishie2.xCA = ((*yakumono_param).unk8 as i32 + rand_val) as i16;

    ext::Ground_801C4A08(gobj);
}

pub unsafe fn grInishie2_801FD4CC(_gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD4CC");
    ext::mpLib_80057BC0(0xE);
}

/// probably chooses which side to spawn Birdo on
pub unsafe extern "C" fn grInishie2_801FD4F0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD4F0");
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let jobj = GET_JOBJ(gobj);
    let gp = GET_GROUND(gobj);
    let temp_r29 = jobj;
    let temp_r30 = jobj;
    let spawn_side = ext::HSD_Randi(2);
    ext::mpJointListAdd(0xE);
    if spawn_side == 0 {
        vec.x = (*yakumono_param).unk14[spawn_side as usize].x + ext::Stage_GetBlastZoneRightOffset();
    } else {
        vec.x = (*yakumono_param).unk14[spawn_side as usize].x + ext::Stage_GetBlastZoneLeftOffset();
    }
    vec.y = (*yakumono_param).unk14[spawn_side as usize].y;
    vec.z = (*yakumono_param).unk14[spawn_side as usize].z;
    HSD_JObjSetTranslate(temp_r29, &vec);
    // sign_inline(spawn_side): (spawn_side == 0) ? -1.0f : 1.0f
    (*gp).u.inishie22.xC4 = ext::it_802ECD3C(gobj, &mut vec, if spawn_side == 0 { -1.0 } else { 1.0 });
    ext::Ground_InitMapColl(temp_r30, (*gp).map_id as s32);
}

pub unsafe extern "C" fn grInishie2_801FD654(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD654");
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let jobj = GET_JOBJ(gobj);
    let gp = GET_GROUND(gobj);
    if !(*gp).u.inishie22.xC4.is_null() {
        ext::it_802ECD1C((*gp).u.inishie22.xC4, &mut vec);
        vec.y += (*yakumono_param).unk2c;
        HSD_JObjSetTranslate(jobj, &vec);
    }
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grInishie2_801FD740(_gobj: *mut HSD_GObj) { melee_core::ct!("grInishie2_801FD740");}

pub unsafe fn grInishie2_801FD744(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD744");
    let gp = GET_GROUND(gobj);

    (*gp).u.inishie2.xC4_flags.set_b4(1);

    (*gp).u.inishie2.xCC = ((*yakumono_param).unk_c as i32 + test_random((*yakumono_param).unk_e as i32)) as i16;
}

pub unsafe fn grInishie2_801FD7A8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD7A8");
    let gp = GET_GROUND(gobj);

    if (*gp).u.inishie2.xC4_flags.b4() != 0 {
        let counter = (*gp).u.inishie2.xCC;
        (*gp).u.inishie2.xCC = counter.wrapping_sub(1);

        if counter < 0 {
            let gobj_2 = grInishie2_801FCCDC(1);
            (*gp).u.inishie2.xC4_flags.set_b4(0);

            if !gobj_2.is_null() {
                (*GET_GROUND(gobj_2)).u.inishie22.xC4 = gobj;
            }
        }
    }
}

pub unsafe fn grInishie2_801FD824(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD824");
    let mut spawn_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let gp = GET_GROUND(gobj);
    let spawn_side = ext::HSD_Randi(2);

    ext::mpJointListAdd(0);

    if spawn_side == 0 {
        spawn_pos.x = (*yakumono_param).unk30[spawn_side as usize].x + ext::Stage_GetBlastZoneRightOffset();
    } else {
        spawn_pos.x = (*yakumono_param).unk30[spawn_side as usize].x + ext::Stage_GetBlastZoneLeftOffset();
    }

    spawn_pos.y = (*yakumono_param).unk30[spawn_side as usize].y;
    spawn_pos.z = (*yakumono_param).unk30[spawn_side as usize].z;

    if spawn_side == 0 {
        (*gp).u.inishie23.xC8_flags.set_b3(1);
    } else {
        (*gp).u.inishie23.xC8_flags.set_b3(0);
    }

    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, (*gp).u.inishie23.xC8_flags.b3() as i32);

    (*gp).u.inishie23.xC8_flags.set_b1(1);
    (*gp).u.inishie23.xC8_flags.set_b0(0);
    (*gp).u.inishie23.xC8_flags.set_b2(0);

    (*gp).u.inishie2.xCA = 0;
    (*gp).u.inishie2.xD8 = spawn_pos;

    HSD_JObjSetTranslate(ext::Ground_801C3FA4(gobj, 0), &spawn_pos);
}

/// 0 = inside the camera bounds, 1 = left of them, 2 = right of them.
#[inline(always)]
unsafe fn cam_side(temp_f: f32) -> i32 {
    if temp_f > ext::Stage_GetCamBoundsRightOffset() {
        2
    } else if temp_f < ext::Stage_GetCamBoundsLeftOffset() {
        1
    } else {
        0
    }
}

pub unsafe fn grInishie2_801FD9EC(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FD9EC");
    let mut gp = (*gobj).user_data as *mut Ground;

    if ext::grAnime_801C83D0(gobj, 1, 1) != 0 {
        if (*gp).u.inishie23.xC8_flags.b1() != 0 {
            let temp_f = (*GET_GROUND(gobj)).u.inishie23.xD8.x + (*GET_GROUND(gobj)).u.inishie23.xCC.x;

            let var_r0 = cam_side(temp_f);

            if var_r0 == 0 {
                (*gp).u.inishie23.xC8_flags.set_b1(0);
            }
        } else {
            if (*gp).u.inishie23.xC8_flags.b2() == 0 {
                if (*gp).u.inishie23.xCA < 2 {
                    let b3 = (*gp).u.inishie23.xC8_flags.b3();
                    (*gp).u.inishie23.xC8_flags.set_b3(b3 ^ 1);
                } else {
                    let temp_f = (*GET_GROUND(gobj)).u.inishie23.xD8.x + (*GET_GROUND(gobj)).u.inishie23.xCC.x;

                    let var_r3 = cam_side(temp_f);

                    if var_r3 != 0 {
                        (*gp).u.inishie23.xC8_flags.set_b2(1);
                        (*gp).u.inishie23.xC8_flags.set_b3(var_r3 as u8);
                        (*gp).u.inishie23.xC8_flags.set_b0(1);
                    } else {
                        let temp_r0 = ((*yakumono_param).unk48 as i32 + (*yakumono_param).unk4a as i32) as i16;

                        let var_r3_2 = test_random(temp_r0 as i32);

                        if var_r3_2 >= (*yakumono_param).unk48 as i32 {
                            let b3 = (*gp).u.inishie23.xC8_flags.b3();
                            (*gp).u.inishie23.xC8_flags.set_b3(b3 ^ 1);
                        }
                    }
                }
            }
            (*gp).u.inishie23.xCA = (*gp).u.inishie23.xCA.wrapping_add(1);
        }

        ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, (*gp).u.inishie23.xC8_flags.b3() as i32);

        gp = GET_GROUND(gobj);
        let jobj = ext::Ground_801C3FA4(gobj, 0);
        HSD_JObjSetTranslate(jobj, &(*gp).u.inishie23.xD8);
        HSD_JObjAddTranslationX(jobj, (*gp).u.inishie23.xCC.x);
        HSD_JObjAddTranslationY(jobj, (*gp).u.inishie23.xCC.y);
        HSD_JObjAddTranslationZ(jobj, (*gp).u.inishie23.xCC.z);
        HSD_JObjGetTranslation(jobj, &mut (*gp).u.inishie23.xD8);

        return;
    }

    gp = GET_GROUND(gobj);
    let jobj = ext::Ground_801C3FA4(gobj, 1);
    HSD_JObjGetTranslation(jobj, &mut (*gp).u.inishie23.xCC);
}

pub unsafe extern "C" fn grInishie2_801FDE8C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FDE8C");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    grInishie2_801FD824(gobj);
    ext::Ground_InitMapColl(jobj, (*gp).map_id as s32);
}

/// `static inline bool checkBlastZone(Vec3*)`.
unsafe fn checkBlastZone(pos: *mut Vec3) -> bool {
    if (*pos).x > ext::Stage_GetBlastZoneRightOffset() {
        true
    } else if (*pos).x < ext::Stage_GetBlastZoneLeftOffset() {
        true
    } else {
        false
    }
}

/// `static inline void grInishie2_801FDED8_inline(HSD_GObj*)`.
unsafe fn grInishie2_801FDED8_inline(gobj: *mut HSD_GObj) {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = GET_GROUND(gobj);

    ext::lb_8000B1CC(ext::Ground_801C3FA4(gobj, 0), null_mut(), &mut pos);

    if checkBlastZone(&mut pos) && (*gp).u.inishie23.xC8_flags.b0() != 0 {
        let gobj_2 = (*gp).u.inishie22.xC4;
        let gp_2 = GET_GROUND(gobj_2);
        (*gp_2).u.inishie2.xC4_flags.set_b4(1);
        (*gp_2).u.inishie2.xCC = ((*yakumono_param).unk_c as i32 + grInishie2_801FDFB4((*yakumono_param).unk_e as i32)) as i16;
        ext::Ground_801C4A08(gobj);
    }
}

pub unsafe extern "C" fn grInishie2_801FDED8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie2_801FDED8");
    ext::Ground_UpdateMapColl(gobj);
    grInishie2_801FD9EC(gobj);

    grInishie2_801FDED8_inline(gobj);
}

pub unsafe fn grInishie2_801FDFB4(arg0: s32) -> s32 {
    melee_core::ct!("grInishie2_801FDFB4");
    if arg0 != 0 {
        return ext::HSD_Randi(arg0);
    }
    0
}

pub unsafe extern "C" fn grInishie2_801FDFE4(_gobj: *mut HSD_GObj) { melee_core::ct!("grInishie2_801FDFE4");}
