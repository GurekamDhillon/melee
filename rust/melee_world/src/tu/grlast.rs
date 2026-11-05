//! src/melee/gr/grlast.c: Final Destination (Gr_Kind_Last): the map gobj (`u.map`) that sequences the
//! backdrop (17 states, each starting animations on up to six backdrop gobjs and fading the
//! background / fog colours), the rotating star gobj (`u.last`, with its particle generator) and the
//! plain animation / jobj-holder gobjs. State is the per-gobj `Ground`, the file statics
//! (`yakumono_param`, the callback / stage-data tables, the animation table `grNLa_803E8010`) and
//! `stage_info`; every other callee is a TODO hook in [`ext`]. The constant-return / empty callbacks
//! that an earlier round ported are re-exported from `crate::gr::trivial::grlast`.
//!
//! TARGET_PC code: `grLast_8021AAB0` saves / restores the RNG seed around the backdrop think when
//! Slippi online runs (`Slippi_Codes() == 2`; no Slippi in the native build yet, the hook defaults to
//! 0), `Ground_StageSlotProofCallbacks` and `Ground_StageSlotFDBackdrop` (grlast_stage_slots.inc).
//! Float code fuses exactly where the shipping IR does (`tools/rust-port/fma_sites.py`): the 2 sites
//! of `grLast_8021AC30` and the 4 of `grLast_8021ADD0`. Float to u8 casts saturate to 32 bits, then
//! truncate (gwtool).
//! State-oracle checked against the C (tests/gro_grlast.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use super::grinl::{
    assert_, GET_GROUND, GET_JOBJ, HSD_JObjSetRotationX, HSD_JObjSetRotationY, HSD_JObjSetScale, ABS, M_PI,
};
use super::grinl2::Ground_StartMapAnim;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;


/// Callees in TUs that are not ported yet. An unset hook panics (`Slippi_Codes` defaults to 0).
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::{stage_info, HSD_JObjSetMtxDirtySub};
    pub use crate::tu::grinl2::ext::{
        grAnime_801C8138, grDisplay_801C5DB0, GObj_SetupGXLink, Ground_InitMapColl, Ground_UpdateMapColl,
        HSD_GObj_SetupProc, HSD_Randi, lb_800115F4,
    };
    hook_or!(CAMERA_800311CC, Camera_800311CC, (a0: f32), |a0: f32| unsafe { crate::tu::camera::Camera_800311CC(a0) });
    hook_or!(CAMERA_800311DC, Camera_800311DC, (a0: f32), |a0: f32| unsafe { crate::tu::camera::Camera_800311DC(a0) });
    hook_or!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, (a0: CmQuakeKind, a1: *mut Vec3), |a0: CmQuakeKind, a1: *mut Vec3| unsafe { crate::tu::camera::Camera_RequestQuake(a0 as _, a1 as _) });
    hook_or!(CAMERA_SETBACKGROUNDCOLOR, Camera_SetBackgroundColor, (r: u8, g: u8, b: u8), |r: u8, g: u8, b: u8| unsafe { crate::tu::camera::Camera_SetBackgroundColor(r, g, b) });
    pub static mut GROUND_801C052C: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C052C(a0: *mut GXColor) {
        match unsafe { GROUND_801C052C } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C052C(a0 as _); } return; } }
    }
    pub static mut GROUND_801C0544: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C0544(a0: *mut GXColor) {
        match unsafe { GROUND_801C0544 } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C0544(a0 as _); } return; } }
    }
    pub static mut GROUND_801C055C: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C055C(a0: *mut GXColor) {
        match unsafe { GROUND_801C055C } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C055C(a0 as _); } return; } }
    }
    pub static mut GROUND_801C0574: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C0574(a0: *mut GXColor) {
        match unsafe { GROUND_801C0574 } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C0574(a0 as _); } return; } }
    }
    pub static mut GROUND_801C058C: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C058C(a0: *mut GXColor) {
        match unsafe { GROUND_801C058C } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C058C(a0 as _); } return; } }
    }
    pub static mut GROUND_801C05A4: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C05A4(a0: *mut GXColor) {
        match unsafe { GROUND_801C05A4 } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C05A4(a0 as _); } return; } }
    }
    pub static mut GROUND_801C05BC: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C05BC(a0: *mut GXColor) {
        match unsafe { GROUND_801C05BC } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C05BC(a0 as _); } return; } }
    }
    pub static mut GROUND_801C05D4: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C05D4(a0: *mut GXColor) {
        match unsafe { GROUND_801C05D4 } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C05D4(a0 as _); } return; } }
    }
    pub static mut GROUND_801C05EC: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C05EC(a0: *mut GXColor) {
        match unsafe { GROUND_801C05EC } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C05EC(a0 as _); } return; } }
    }
    pub static mut GROUND_801C10B8: Option<fn(*mut HSD_GObj, HSD_GObjEvent)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C10B8(a0: *mut HSD_GObj, a1: HSD_GObjEvent) {
        match unsafe { GROUND_801C10B8 } { Some(f) => f(a0, a1), None => { unsafe { crate::tu::ground::Ground_801C10B8(a0 as _, a1 as _); } return; } }
    }
    pub static mut GROUND_801C1E00: Option<fn(s32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C1E00(a0: s32) {
        match unsafe { GROUND_801C1E00 } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C1E00(a0 as _); } return; } }
    }
    pub static mut GROUND_801C205C: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C205C(color: *mut GXColor) {
        match unsafe { GROUND_801C205C } { Some(f) => f(color), None => { unsafe { crate::tu::ground::Ground_801C205C(color as _); } return; } }
    }
    pub static mut GROUND_801C2090: Option<fn(*mut GXColor) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C2090(color: *mut GXColor) -> i32 {
        match unsafe { GROUND_801C2090 } { Some(f) => f(color), None => { return unsafe { crate::tu::ground::Ground_801C2090(color as _) as _ }; } }
    }
    pub static mut GROUND_801C39C0: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C39C0() {
        match unsafe { GROUND_801C39C0 } { Some(f) => f(), None => { unsafe { crate::tu::ground::Ground_801C39C0(); } return; } }
    }
    pub static mut GROUND_801C3BB4: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3BB4() {
        match unsafe { GROUND_801C3BB4 } { Some(f) => f(), None => { unsafe { crate::tu::ground::Ground_801C3BB4(); } return; } }
    }
    pub static mut GROUND_801C3FA4: Option<fn(*mut HSD_GObj, i32) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3FA4(gobj: *mut HSD_GObj, depth: i32) -> *mut HSD_JObj {
        match unsafe { GROUND_801C3FA4 } { Some(f) => f(gobj, depth), None => { return unsafe { crate::tu::ground::Ground_801C3FA4(gobj as _, depth as _) as _ }; } }
    }
    pub static mut GROUND_801C4A08: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C4A08(gobj: *mut HSD_GObj) {
        match unsafe { GROUND_801C4A08 } { Some(f) => f(gobj), None => { unsafe { crate::tu::ground::Ground_801C4A08(gobj as _); } return; } }
    }
    pub static mut GROUND_GETMAPGOBJ: Option<fn(i32) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetMapGObj(map_id: i32) -> *mut HSD_GObj {
        match unsafe { GROUND_GETMAPGOBJ } { Some(f) => f(map_id), None => { return unsafe { crate::tu::ground::Ground_GetMapGObj(map_id as _) as _ }; } }
    }
    pub static mut GROUND_GETSTAGEGOBJ: Option<fn(i32) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj {
        match unsafe { GROUND_GETSTAGEGOBJ } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ }; } }
    }
    pub static mut GROUND_GETYAKUMONOPARAM: Option<fn() -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetYakumonoParam() -> *mut c_void {
        match unsafe { GROUND_GETYAKUMONOPARAM } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ }; } }
    }
    hook_or!(HSD_GOBJGXLINK_803909D8, HSD_GObjGXLink_803909D8, (a0: *mut HSD_GObj, a1: *mut HSD_GObj), |a0: *mut HSD_GObj, a1: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjGXLink_803909D8(a0 as _, a1 as _); } });
    hook_or!(HSD_RANDF, HSD_Randf, () -> f32, || melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand()));
    hook_or!(OSREPORT, OSReport, (a0: *mut i8), |a0: *mut i8| unsafe { melee_runtime::tu::os_error::OSReport(a0 as _, &[]) });
    /// `extern int Slippi_Codes(void)` (TARGET_PC): no Slippi online in the native build, so 0.
    hook_or!(SLIPPI_CODES, Slippi_Codes, () -> i32, || 0);
    pub static mut STAGE_80225194: Option<fn() -> StKind> = None;
    #[allow(non_snake_case)]
    pub fn Stage_80225194() -> StKind {
        match unsafe { STAGE_80225194 } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_80225194() as _ }; } }
    }
    hook!(GM_8016ECE8, gm_8016ECE8, () -> f32);
    pub static mut GRANIME_801C7980: Option<fn(*mut HSD_GObj, i32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C7980(a0: *mut HSD_GObj, a1: i32, a2: u32) {
        match unsafe { GRANIME_801C7980 } { Some(f) => f(a0, a1, a2), None => { unsafe { crate::tu::granime::grAnime_801C7980(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRANIME_801C7A94: Option<fn(*mut HSD_GObj, i32, u32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C7A94(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32) {
        match unsafe { GRANIME_801C7A94 } { Some(f) => f(a0, a1, a2, a3), None => { unsafe { crate::tu::granime::grAnime_801C7A94(a0 as _, a1 as _, a2 as _, a3 as _); } return; } }
    }
    pub static mut GRANIME_801C7FF8: Option<fn(*mut HSD_GObj, i32, i32, i32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) {
        match unsafe { GRANIME_801C7FF8 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { unsafe { crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } return; } }
    }
    pub static mut GRANIME_801C8098: Option<fn(*mut HSD_GObj, i32, i32, i32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C8098(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) {
        match unsafe { GRANIME_801C8098 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { unsafe { crate::tu::granime::grAnime_801C8098(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } return; } }
    }
    pub static mut GRANIME_801C8318: Option<fn(*mut HSD_GObj, i32, u32) -> *mut HSD_AObj> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C8318(a0: *mut HSD_GObj, a1: i32, a2: u32) -> *mut HSD_AObj {
        match unsafe { GRANIME_801C8318 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::granime::grAnime_801C8318(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRANIME_801C83D0: Option<fn(*mut HSD_GObj, i32, enum_t) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: enum_t) -> i32 {
        match unsafe { GRANIME_801C83D0 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRANIME_801C84A4: Option<fn(*mut HSD_GObj, s32, s32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C84A4(a0: *mut HSD_GObj, a1: s32, a2: s32) -> i32 {
        match unsafe { GRANIME_801C84A4 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::granime::grAnime_801C84A4(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRLIB_801C96E8: Option<fn(*mut HSD_GObj) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C96E8(a0: *mut HSD_GObj) -> i32 {
        match unsafe { GRLIB_801C96E8 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grlib::grLib_801C96E8(a0 as _) as _ }; } }
    }
    pub static mut GRLIB_801C96F8: Option<fn(s32, s32, *mut Vec3) -> *mut HSD_Generator> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C96F8(a0: s32, a1: s32, a2: *mut Vec3) -> *mut HSD_Generator {
        match unsafe { GRLIB_801C96F8 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::grlib::grLib_801C96F8(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRLIB_801C9834: Option<fn(*mut HSD_Generator)> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C9834(a0: *mut HSD_Generator) {
        match unsafe { GRLIB_801C9834 } { Some(f) => f(a0), None => { unsafe { crate::tu::grlib::grLib_801C9834(a0 as _); } return; } }
    }
    pub static mut GRLIB_801C9854: Option<fn(*mut HSD_JObj)> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C9854(a0: *mut HSD_JObj) {
        match unsafe { GRLIB_801C9854 } { Some(f) => f(a0), None => { unsafe { crate::tu::grlib::grLib_801C9854(a0 as _); } return; } }
    }
    pub static mut GRMATERIAL_801C94D8: Option<fn(*mut c_void)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C94D8(a0: *mut c_void) {
        match unsafe { GRMATERIAL_801C94D8 } { Some(f) => f(a0), None => { unsafe { crate::tu::grmaterial::grMaterial_801C94D8(a0 as _); } return; } }
    }
    pub static mut GRMATERIAL_801C9604: Option<fn(*mut HSD_GObj, *mut c_void, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C9604(bg: *mut HSD_GObj, a1: *mut c_void, a2: i32) {
        match unsafe { GRMATERIAL_801C9604 } {
            Some(f) => f(bg, a1, a2),
            None => { unsafe { crate::tu::grmaterial::grMaterial_801C9604(bg as _, a1 as _, (a2 != 0)); } },
        }
    }
    pub static mut GRMATERIAL_801C9698: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C9698(a0: *mut HSD_GObj) {
        match unsafe { GRMATERIAL_801C9698 } { Some(f) => f(a0), None => { unsafe { crate::tu::grmaterial::grMaterial_801C9698(a0 as _); } return; } }
    }
    pub static mut GRZAKOGENERATOR_801CAE04: Option<fn(*mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj {
        match unsafe { GRZAKOGENERATOR_801CAE04 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ }; } }
    }
    pub static mut LBVECTOR_EULERANGLESFROMPARTIALONB: Option<fn(*mut Vec3, *mut Vec3, *mut Vec3) -> *mut Vec3> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_EulerAnglesFromPartialONB(result_angles: *mut Vec3, a: *mut Vec3, c: *mut Vec3) -> *mut Vec3 {
        match unsafe { LBVECTOR_EULERANGLESFROMPARTIALONB } { Some(f) => f(result_angles, a, c), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_EulerAnglesFromPartialONB(result_angles as _, a as _, c as _) as _ }; } }
    }
    pub static mut LBVECTOR_NORMALIZE: Option<fn(*mut Vec3) -> f32> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Normalize(vec: *mut Vec3) -> f32 {
        match unsafe { LBVECTOR_NORMALIZE } { Some(f) => f(vec), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(vec as _) as _ }; } }
    }
    pub static mut LBVECTOR_SUB: Option<fn(*mut Vec3, *mut Vec3) -> *mut Vec3> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Sub(a: *mut Vec3, b: *mut Vec3) -> *mut Vec3 {
        match unsafe { LBVECTOR_SUB } { Some(f) => f(a, b), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Sub(a as _, b as _) as _ }; } }
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    /// `u32* HSD_RandSeedPtr` (baselib/random.c).
    pub use melee_runtime::tu::random::global::HSD_RandSeedPtr;
}

/// `static struct grLast_YakumonoParam { void* x0; void* x4; void* x8; void* xC; }* yakumono_param`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grLast_YakumonoParam {
    pub x0: *mut c_void,
    pub x4: *mut c_void,
    pub x8: *mut c_void,
    pub xC: *mut c_void,
}

pub static mut yakumono_param: *mut grLast_YakumonoParam = null_mut();

/// `SDATA GrJoint grLast_804D4968[]`.
pub static mut grLast_804D4968: [GrJoint; 1] = [GrJoint { x: 0, y: 3, z: 0 }];

/// `Vec3 const grLast_803B8480 / 848C / 8498`.
pub static grLast_803B8480: Vec3 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
pub static grLast_803B848C: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
pub static grLast_803B8498: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

/// `static s16 grNLa_803E8010[][4]`.
pub static mut grNLa_803E8010: [[i16; 4]; 6] = [
    [4, 1, 2, 0],
    [5, 1, 2, 4],
    [6, 1, 2, 6],
    [7, 1, 2, 8],
    [8, 1, 2, 10],
    [9, 0, -1, 0],
];

pub static grNLa_804DBBD4: GXColor = GXColor { r: 0xAA, g: 0xCA, b: 0xFF, a: 0x00 };
pub static grNLa_804DBBD8: GXColor = GXColor { r: 0xC8, g: 0xC8, b: 0xC8, a: 0xFF };
pub static grNLa_804DBBDC: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };
pub static grNLa_804DBBE0: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };

macro_rules! cbs {
    ($a:expr, $b:expr, $c:expr, $d:expr, $flags:expr) => {
        StageCallbacks {
            on_init: Some($a),
            callback1: Some($b),
            gobj_proc: Some($c),
            callback3: Some($d),
            anon_10: StageCallbacks_anon_135 { flags: u32::to_be($flags) },
        }
    };
}

/// `static StageCallbacks grLast_StageCallbacks[]`.
pub static mut grLast_StageCallbacks: [StageCallbacks; 10] = [
    cbs!(stageGObj0_OnInit, grLast_8021A90C, grLast_8021A914, grLast_8021A918, 0),
    cbs!(grLast_8021A91C, grLast_8021A960, grLast_8021A968, grLast_8021A96C, 0),
    cbs!(stageGObj2_OnInit, grLast_8021A99C, grLast_8021A9A4, grLast_8021A9A8, 0),
    cbs!(grLast_8021A9C4, grLast_8021AAA8, grLast_8021AAB0, grLast_8021AB30, 0x8000_0000),
    cbs!(grLast_8021AB34, grLast_8021AB78, grLast_8021AB80, grLast_8021AB84, 0x4000_0000),
    cbs!(grLast_8021AB88, grLast_8021ABCC, grLast_8021ABD4, grLast_8021ABD8, 0),
    cbs!(grLast_8021ABDC, grLast_8021AC20, grLast_8021AC28, grLast_8021AC2C, 0),
    cbs!(grLast_8021AC30, grLast_8021ADC8, grLast_8021ADD0, grLast_8021B23C, 0),
    cbs!(grLast_8021B240, grLast_8021B284, grLast_8021B28C, grLast_8021B290, 0),
    cbs!(grLast_8021B294, grLast_8021B2D8, grLast_8021B2E0, grLast_8021B2E4, 0),
];

/// `StageData grNLa_StageData`.
pub static mut grNLa_StageData: StageData = StageData {
    grkind: Gr_Kind_Last,
    callbacks: unsafe { core::ptr::addr_of!(grLast_StageCallbacks) as *mut StageCallbacks },
    data1: b"/GrNLa.dat\0".as_ptr() as *mut i8,
    on_init: Some(grLast_OnInit),
    on_demo_init: Some(grLast_OnDemoInit),
    on_load: Some(grLast_OnLoad),
    on_start: Some(grLast_OnStart),
    callback4: Some(grLast_8021A7EC),
    on_touch_line: Some(grLast_OnTouchLine),
    on_check_shadow_render: Some(grLast_OnCheckShadowRender),
    flags2: 1 << 0,
    joints: unsafe { core::ptr::addr_of!(grLast_804D4968) as *mut GrJoint },
    joint_count: 1,
};

pub unsafe extern "C" fn grLast_OnDemoInit(arg0: enum_t) {
    melee_core::ct!("grLast_OnDemoInit");
    if arg0 != 26 {
        return;
    }
    for id in [4, 5, 6, 7, 8, 9] {
        let gobj = ext::Ground_GetMapGObj(id);
        if !gobj.is_null() {
            ext::Ground_801C4A08(gobj);
        }
    }
    let mut gobj = ext::Ground_GetMapGObj(1);
    assert_(!gobj.is_null(), 80, "gobj");

    gobj = ext::Ground_GetMapGObj(3);
    assert_(!gobj.is_null(), 83, "gobj");

    let gp = GET_GROUND(gobj);
    assert_(!gp.is_null(), 84, "gp");
    (*gp).u.map.set_xC4_b1(1);
}

pub unsafe extern "C" fn grLast_OnInit() {
    melee_core::ct!("grLast_OnInit");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grLast_YakumonoParam;
    let si = ext::stage_info();
    (*si).unk8C.set_b4(1);
    (*si).unk8C.set_b5(1);
    grLast_8021A7F4(0);
    grLast_8021A7F4(1);
    grLast_8021A7F4(2);
    grLast_8021A7F4(3);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Camera_800311DC(1.0);
    ext::Camera_800311CC(30000.0);
}

pub unsafe extern "C" fn grLast_OnLoad() { melee_core::ct!("grLast_OnLoad");}

pub unsafe extern "C" fn grLast_OnStart() {
    melee_core::ct!("grLast_OnStart");
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe fn grLast_8021A7F4(id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grLast_8021A7F4");
    let cb: *mut StageCallbacks = &mut grLast_StageCallbacks[id as usize];
    let gobj = ext::Ground_GetStageGObj(id);
    if !gobj.is_null() {
        let gp = GET_GROUND(gobj);
        (*gp).x8_callback = None;
        (*gp).xC_callback = None;
        ext::GObj_SetupGXLink(gobj, Some(ext::grDisplay_801C5DB0), 3, 0);
        if (*cb).callback3.is_some() {
            (*gp).x1C_callback = (*cb).callback3;
        }
        if let Some(f) = (*cb).on_init {
            f(gobj);
        }
        if (*cb).gobj_proc.is_some() {
            ext::HSD_GObj_SetupProc(gobj, (*cb).gobj_proc, 4);
        }
    } else {
        let mut msg = *b"%s:%d: couldn t get gobj(id=%d)\n\0";
        ext::OSReport(msg.as_mut_ptr() as *mut i8);
    }
    gobj
}

pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj0_OnInit");
    Ground_StartMapAnim(gobj);
}

pub unsafe extern "C" fn grLast_8021A914(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021A914");}

pub unsafe extern "C" fn grLast_8021A918(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021A918");}

pub unsafe extern "C" fn grLast_8021A91C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021A91C");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grLast_8021A968(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021A968");}

pub unsafe extern "C" fn grLast_8021A96C(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021A96C");}

pub unsafe extern "C" fn stageGObj2_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj2_OnInit");
    Ground_StartMapAnim(gobj);
}

pub unsafe extern "C" fn grLast_8021A9A4(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021A9A4");}

pub unsafe extern "C" fn grLast_8021A9A8(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021A9A8");}

pub unsafe extern "C" fn grLast_8021A9AC(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021A9AC");
    let gp = GET_GROUND(gobj);
    (*gp).u.map.set_xC4_b0(0);
}

pub unsafe extern "C" fn grLast_8021A9C4(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021A9C4");
    let gp = (*gobj).user_data as *mut Ground;

    ext::Ground_InitMapColl((*gobj).hsd_obj as *mut HSD_JObj, (*gp).map_id as s32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).u.map.set_xC4_b0(1);
    for i in 0..6usize {
        (*gp).u.map.lv_gobj[i] = null_mut();
    }
    (*gp).u.map.set_xC4_b2_25(0);
    (*gp).u.map.set_xC4_b26(0);
    (*gp).u.map.set_xC4_b27(1);
    grLast_8021B920(gobj, 1);
    (*gp).x11_flags.set_b012(1);
    (*gp).u.map.set_xC4_b1(0);
    ext::Ground_801C10B8(gobj, Some(grLast_8021A9AC));
}

pub unsafe extern "C" fn grLast_8021AAB0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021AAB0");
    let gp = GET_GROUND(gobj);
    if (*gp).u.map.xC4_b1() == 0 && (*gp).u.map.xC4_b0() == 0 {
        // TARGET_PC: Slippi online wraps the backdrop think in a save / restore of the RNG seed
        let fd_seed = *ext::HSD_RandSeedPtr;
        // Slippi "Frozen Stages" (Frozen All/Core/2.asm @ 8021AAE4): the backdrop think is skipped, FD holds still
        if !melee_core::codes::on(melee_core::codes::FROZEN_STAGES) {
            grLast_8021B2E8(gobj);
        }
        if ext::Slippi_Codes() == 2 {
            *ext::HSD_RandSeedPtr = fd_seed;
        }
        if (*gp).u.map.xC4_b26() != 0 {
            let tmp = grLast_8021B5C4(gobj);
            if tmp != 0 {
                grLast_8021B920(gobj, tmp);
            }
        }
    }
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
}

pub unsafe extern "C" fn grLast_8021AB30(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021AB30");}

pub unsafe extern "C" fn grLast_8021AB34(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021AB34");
    let gp = GET_GROUND(gobj);
    ext::grMaterial_801C94D8(GET_JOBJ(gobj) as *mut c_void);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grLast_8021AB80(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021AB80");}

pub unsafe extern "C" fn grLast_8021AB84(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021AB84");}

pub unsafe extern "C" fn grLast_8021AB88(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021AB88");
    let gp = GET_GROUND(gobj);
    ext::grMaterial_801C94D8(GET_JOBJ(gobj) as *mut c_void);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grLast_8021ABD4(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021ABD4");}

pub unsafe extern "C" fn grLast_8021ABD8(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021ABD8");}

pub unsafe extern "C" fn grLast_8021ABDC(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021ABDC");
    let gp = GET_GROUND(gobj);
    ext::grMaterial_801C94D8(GET_JOBJ(gobj) as *mut c_void);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grLast_8021AC28(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021AC28");}

pub unsafe extern "C" fn grLast_8021AC2C(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021AC2C");}

/// `(M_PI / 180) / 250`, `/ 1000`, `/ 20`, `* 15`, `* 10` (double constants stored as float).
const C_804DBB80: f32 = ((M_PI / 180.0) / 250.0) as f32;
const C_804DBB7C: f32 = ((M_PI / 180.0) / 1000.0) as f32;
const C_804DBB8C: f32 = (M_PI / 180.0 / 20.0) as f32;
const C_804DBB90: f32 = (-(M_PI / 180.0) / 20.0) as f32;
const C_804DBB94: f32 = (M_PI / 180.0 * 15.0) as f32;
const C_804DBB98: f32 = (-(M_PI / 180.0) * 15.0) as f32;
const C_804DBB9C: f32 = (M_PI / 180.0 * 10.0) as f32;
const C_804DBBA0: f32 = (-(M_PI / 180.0) * 10.0) as f32;

pub unsafe extern "C" fn grLast_8021AC30(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021AC30");
    let gp = GET_GROUND(gobj);

    ext::grMaterial_801C94D8(GET_JOBJ(gobj) as *mut c_void);
    (*gp).u.last.xC4 = 0.0;
    (*gp).u.last.xC8 = 0.0;
    (*gp).u.last.xCC = 0.0;
    (*gp).u.last.xD0 = 0.0;
    (*gp).u.last.xD4 = fmadds(ext::HSD_Randf(), C_804DBB80, C_804DBB7C);
    (*gp).u.last.xD4 *= if ext::HSD_Randi(2) != 0 { 1.0 } else { -1.0 };
    (*gp).u.last.xD8 = fmadds(ext::HSD_Randf(), C_804DBB80, C_804DBB7C);
    (*gp).u.last.xD8 *= if ext::HSD_Randi(2) != 0 { 1.0 } else { -1.0 };
    (*gp).u.last.xDC = 0.0;
    (*gp).u.last.xE0 = null_mut();

    {
        let jobj = ext::Ground_801C3FA4(gobj, 5);
        let sp10 = grLast_803B8480;
        HSD_JObjSetScale(jobj, &sp10);
        (*gp).x11_flags.set_b012(2);
    }
}

pub unsafe extern "C" fn grLast_8021ADD0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021ADD0");
    let mut sp38 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp2c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp20;
    let mut sp14;

    let gp = GET_GROUND(gobj);
    let mut jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    (*gp).u.last.xCC += (*gp).u.last.xD4;
    (*gp).u.last.xCC *= 0.99f32;
    if (*gp).u.last.xCC > C_804DBB8C {
        (*gp).u.last.xCC = C_804DBB8C;
    } else if (*gp).u.last.xCC < C_804DBB90 {
        (*gp).u.last.xCC = C_804DBB90;
    }
    (*gp).u.last.xD0 += (*gp).u.last.xD8;
    (*gp).u.last.xD0 *= 0.99f32;
    if (*gp).u.last.xD0 > C_804DBB8C {
        (*gp).u.last.xD0 = C_804DBB8C;
    } else if (*gp).u.last.xD0 < C_804DBB90 {
        (*gp).u.last.xD0 = C_804DBB90;
    }
    (*gp).u.last.xC4 += (*gp).u.last.xCC;
    if (*gp).u.last.xC4 > C_804DBB94 {
        (*gp).u.last.xC4 = C_804DBB94;
        (*gp).u.last.xCC = -ABS((*gp).u.last.xCC);
        let randf = ext::HSD_Randf();
        (*gp).u.last.xD4 = fmadds(C_804DBB80, -randf, -C_804DBB7C);
    } else if (*gp).u.last.xC4 < C_804DBB98 {
        (*gp).u.last.xC4 = C_804DBB98;
        (*gp).u.last.xCC = ABS((*gp).u.last.xCC);
        (*gp).u.last.xD4 = fmadds(C_804DBB80, ext::HSD_Randf(), C_804DBB7C);
    }
    HSD_JObjSetRotationX(jobj, (*gp).u.last.xC4 * (*gp).u.last.xDC);
    (*gp).u.last.xC8 += (*gp).u.last.xD0;
    if (*gp).u.last.xC8 > C_804DBB9C {
        (*gp).u.last.xC8 = C_804DBB9C;
        (*gp).u.last.xD0 = -ABS((*gp).u.last.xD0);
        let randf = ext::HSD_Randf();
        (*gp).u.last.xD8 = fmadds(C_804DBB80, -randf, -C_804DBB7C);
    } else if (*gp).u.last.xC8 < C_804DBBA0 {
        (*gp).u.last.xC8 = C_804DBBA0;
        (*gp).u.last.xD0 = ABS((*gp).u.last.xD0);
        (*gp).u.last.xD8 = fmadds(C_804DBB80, ext::HSD_Randf(), C_804DBB7C);
    }
    HSD_JObjSetRotationY(jobj, (*gp).u.last.xC8 * (*gp).u.last.xDC);
    if !(*gp).u.last.xE0.is_null() && !(*(*gp).u.last.xE0).appsrt.is_null() {
        sp20 = grLast_803B848C;
        sp14 = grLast_803B8498;
        let p20: *mut Vec3 = &mut sp20;
        let p14: *mut Vec3 = &mut sp14;

        jobj = ext::Ground_801C3FA4(gobj, 5);
        ext::lb_8000B1CC(jobj, null_mut(), &mut sp38);
        ext::lb_8000B1CC(jobj, p20, p20);
        ext::lbVector_Sub(p20, &mut sp38);
        ext::lbVector_Normalize(p20);
        ext::lb_8000B1CC(jobj, p14, p14);
        ext::lbVector_Sub(p14, &mut sp38);
        ext::lbVector_Normalize(p14);
        ext::lbVector_EulerAnglesFromPartialONB(&mut sp2c, p14, p20);
        let a = (*(*gp).u.last.xE0).appsrt;
        (*a).translate = sp38;
        (*a).rot.x = sp2c.x;
        (*a).rot.y = sp2c.y;
        (*a).rot.z = sp2c.z;
        (*a).status = 0;
    }
}

pub unsafe extern "C" fn grLast_8021B23C(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021B23C");}

pub unsafe extern "C" fn grLast_8021B240(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021B240");
    let gp = GET_GROUND(gobj);
    ext::grMaterial_801C94D8(GET_JOBJ(gobj) as *mut c_void);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grLast_8021B28C(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021B28C");}

pub unsafe extern "C" fn grLast_8021B290(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021B290");}

pub unsafe extern "C" fn grLast_8021B294(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021B294");
    let gp = GET_GROUND(gobj);
    ext::grMaterial_801C94D8(GET_JOBJ(gobj) as *mut c_void);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grLast_8021B2E0(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021B2E0");}

pub unsafe extern "C" fn grLast_8021B2E4(_gobj: *mut HSD_GObj) { melee_core::ct!("grLast_8021B2E4");}

pub unsafe fn grLast_8021B2E8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021B2E8");
    let mut sp30 = GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut sp2c = GXColor { r: 0, g: 0, b: 0, a: 0 };

    let gp = GET_GROUND(gobj);
    if ext::Stage_80225194() == 0xB0 || ext::Stage_80225194() == 0xFB {
        let temp_f1 = ext::gm_8016ECE8();
        match (*gp).u.map.xC4_b2_25() {
            1 => {
                if temp_f1 < 0.7f32 {
                    (*gp).u.map.set_xC4_b26(1);
                }
            }
            3 => {
                if temp_f1 < 0.6f32 {
                    (*gp).u.map.set_xC4_b26(1);
                }
            }
            5 => {
                if temp_f1 < 0.5f32 {
                    (*gp).u.map.set_xC4_b26(1);
                }
            }
            7 => {
                if temp_f1 < 0.4f32 {
                    (*gp).u.map.set_xC4_b26(1);
                }
            }
            9 => {
                if temp_f1 < 0.3f32 {
                    (*gp).u.map.set_xC4_b26(1);
                }
            }
            12 => {
                if temp_f1 < 0.2f32 {
                    (*gp).u.map.set_xC4_b26(1);
                }
            }
            _ => {}
        }
    } else if (*gp).u.map.xC8 > 1800.0f32 {
        (*gp).u.map.set_xC4_b26(1);
    }
    (*gp).u.map.xC8 += 1.0f32;
    match (*gp).u.map.xC4_b2_25() {
        9 => {
            let temp_gp = GET_GROUND((*gp).u.map.lv_gobj[3]);
            (*temp_gp).u.last.xDC += 1.0f32 / 180.0f32;
            if (*temp_gp).u.last.xDC > 1.0 {
                (*temp_gp).u.last.xDC = 1.0;
            }
        }
        10 => {
            let temp_gp = GET_GROUND((*gp).u.map.lv_gobj[3]);
            (*temp_gp).u.last.xDC -= 1.0f32 / 58.0f32;
            if (*temp_gp).u.last.xDC < 0.0 {
                (*temp_gp).u.last.xDC = 0.0;
            }
            ext::Camera_RequestQuake(QuakeKind_Loop, null_mut());
        }
        11 => {
            ext::Camera_RequestQuake(QuakeKind_Loop, null_mut());
        }
        15 => {
            if (*gp).u.map.xC4_b27() != 0 {
                ext::Ground_801C2090(&mut sp2c);
                loop {
                    sp30.r = (ext::HSD_Randi(2) * 0x1E) as u8;
                    sp30.g = (ext::HSD_Randi(2) * 0x1E) as u8;
                    sp30.b = (ext::HSD_Randi(2) * 0x1E) as u8;
                    if !((sp30.r == 0x1E && sp30.g == 0x1E && sp30.b == 0x1E)
                        || (sp30.r == sp2c.r && sp30.g == sp2c.g && sp30.g == sp2c.g))
                    {
                        break;
                    }
                }
                let mut tmp = ext::HSD_Randi(0x3C);
                tmp += 0x3C;
                grLast_8021C40C(gobj, sp30, tmp as f32);
            }
        }
        _ => {}
    }
    grLast_8021C500(gobj);
}

/// `static inline bool animate_all(Ground* gp)`.
#[inline]
unsafe fn animate_all(gp: *mut Ground) -> bool {
    for i in 0..5usize {
        if ext::grAnime_801C83D0((*gp).u.map.lv_gobj[i], grNLa_803E8010[i][1] as i32, 7) == 0 {
            return false;
        }
    }
    true
}

pub unsafe fn grLast_8021B5C4(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grLast_8021B5C4");
    let mut result = 0;
    let gp = GET_GROUND(gobj);
    match (*gp).u.map.xC4_b2_25() {
        1 => result = 2,
        2 => {
            if animate_all(gp) {
                result = 3;
            }
        }
        3 => result = 4,
        4 => {
            if animate_all(gp) {
                result = 5;
            }
        }
        5 => result = 6,
        6 => {
            if animate_all(gp) {
                result = 7;
            }
        }
        7 => result = 8,
        8 => {
            if animate_all(gp) {
                result = 9;
            }
        }
        9 => {
            let aobj = ext::grAnime_801C8318((*gp).u.map.lv_gobj[3], grNLa_803E8010[3][2] as i32, 7);
            assert_(!aobj.is_null(), 0x3F4, "aobj");
            if (*aobj).curr_frame == 100.0 {
                result = 10;
            }
        }
        10 => {
            if ext::grAnime_801C84A4((*gp).u.map.lv_gobj[3], grNLa_803E8010[3][2] as i32, 7) != 0 {
                result = 11;
            }
        }
        11 => {
            if animate_all(gp) {
                result = 12;
            }
        }
        12 => result = 13,
        13 => {
            if (*gp).u.map.xC4_b27() != 0 && ext::grLib_801C96E8((*gp).u.map.lv_gobj[4]) != 0 {
                result = 14;
            }
        }
        14 => {
            if ext::grLib_801C96E8((*gp).u.map.lv_gobj[5]) != 0 {
                result = 15;
            }
        }
        15 => result = 16,
        16 => {
            if (*gp).u.map.xC4_b27() != 0 && ext::grLib_801C96E8((*gp).u.map.lv_gobj[5]) != 0 {
                result = 17;
            }
        }
        17 => {
            if ext::grLib_801C96E8((*gp).u.map.lv_gobj[0]) != 0 {
                result = 1;
            }
        }
        _ => {}
    }
    result
}

/// `static inline void do_anime(Ground_GObj* gobj, int i, int n)`.
#[inline]
unsafe fn do_anime(gobj: *mut HSD_GObj, i: usize, n: i32) {
    if grNLa_803E8010[i][2] != -1 {
        ext::grAnime_801C8098(gobj, grNLa_803E8010[i][1] as i32, 7, n, 0.0, 1.0);
        ext::grAnime_801C7980(gobj, grNLa_803E8010[i][1] as i32, 7);
    } else {
        ext::grAnime_801C7FF8(gobj, grNLa_803E8010[i][1] as i32, 7, n, 0.0, 1.0);
    }
}

/// `static inline void grlib_inline(Ground* gp)`.
#[inline]
unsafe fn grlib_inline(gp: *mut Ground) {
    ext::grLib_801C9834((*gp).u.last.xE0);
    (*gp).u.last.xE0 = null_mut();
}

pub unsafe fn grLast_8021B920(gobj: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("grLast_8021B920");
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;
    let mut sp40 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    (*gp).u.map.xC8 = 0.0;
    (*gp).u.map.set_xC4_b2_25(arg1 as u32);
    match (*gp).u.map.xC4_b2_25() {
        1 => {
            for i in 0..5usize {
                if (*gp).u.map.lv_gobj[i].is_null() {
                    (*gp).u.map.lv_gobj[i] = grLast_8021A7F4(grNLa_803E8010[i][0] as i32);
                    assert_(!(*gp).u.map.lv_gobj[i].is_null(), 1099, "gp->u.map.lv_gobj[i]");
                    ext::HSD_GObjGXLink_803909D8((*gp).u.map.lv_gobj[i], gobj);
                    if grNLa_803E8010[i][2] != -1 {
                        ext::grAnime_801C7FF8(
                            (*gp).u.map.lv_gobj[i],
                            grNLa_803E8010[i][2] as i32,
                            7,
                            grNLa_803E8010[i][3] as i32,
                            0.0,
                            1.0,
                        );
                    }
                    do_anime((*gp).u.map.lv_gobj[i], i, 0);
                }
            }
            ext::grAnime_801C8098(ext::Ground_GetMapGObj(4), 0x10, 7, 0, 0.0, 1.0);
            (*gp).u.map.set_xC4_b26(0);
            grLast_8021C640();
        }
        2 => {
            for i in 0..5usize {
                do_anime((*gp).u.map.lv_gobj[i], i, 1);
            }
            ext::grAnime_801C7A94(ext::Ground_GetMapGObj(4), 0x10, 7, 0.0);
            ext::grLib_801C9854(GET_JOBJ(ext::Ground_GetMapGObj(4)));
            (*gp).u.map.set_xC4_b26(1);
        }
        3 => {
            for i in 0..5usize {
                do_anime((*gp).u.map.lv_gobj[i], i, 2);
            }
            (*gp).u.map.set_xC4_b26(1);
        }
        4 => {
            for i in 0..5usize {
                do_anime((*gp).u.map.lv_gobj[i], i, 3);
            }
            (*gp).u.map.set_xC4_b26(1);
        }
        5 => {
            for i in 0..5usize {
                do_anime((*gp).u.map.lv_gobj[i], i, 4);
            }
            (*gp).u.map.set_xC4_b26(0);
        }
        6 => {
            for i in 0..5usize {
                do_anime((*gp).u.map.lv_gobj[i], i, 5);
            }
            (*gp).u.map.set_xC4_b26(1);
        }
        7 => {
            for i in 0..5usize {
                do_anime((*gp).u.map.lv_gobj[i], i, 6);
            }
            (*gp).u.map.set_xC4_b26(0);
        }
        8 => {
            for i in 0..5usize {
                do_anime((*gp).u.map.lv_gobj[i], i, 7);
            }
            (*gp).u.map.set_xC4_b26(1);
        }
        9 => {
            for i in 0..5usize {
                do_anime((*gp).u.map.lv_gobj[i], i, 8);
            }

            let temp_r28_10 = GET_GROUND(ext::Ground_GetMapGObj(7));
            ext::lb_8000B1CC(ext::Ground_801C3FA4(ext::Ground_GetMapGObj(7), 5), null_mut(), &mut sp40);
            (*temp_r28_10).u.last.xE0 = ext::grLib_801C96F8(0x7531, 0x1E, &mut sp40);

            (*gp).u.map.set_xC4_b26(0);
        }
        10 => {
            grLast_8021C40C(gobj, grNLa_804DBBD4, 120.0);
            ext::grAnime_801C7A94(ext::Ground_GetMapGObj(7), 5, 7, 0.0);

            grlib_inline(GET_GROUND(ext::Ground_GetMapGObj(7)));

            (*gp).u.map.set_xC4_b26(1);
            grLast_8021C6AC();
        }
        11 => {
            for i in 0..5usize {
                do_anime((*gp).u.map.lv_gobj[i], i, 9);
            }
            (*gp).u.map.set_xC4_b26(1);
        }
        12 => {
            for i in 0..5usize {
                do_anime((*gp).u.map.lv_gobj[i], i, 10);
            }
            (*gp).u.map.set_xC4_b26(0);
        }
        13 => {
            grLast_8021C40C(gobj, grNLa_804DBBD8, 120.0);
            for i in 0..5usize {
                ext::grMaterial_801C9604((*gp).u.map.lv_gobj[i], (*yp).x0, 0);
                ext::grMaterial_801C9698((*gp).u.map.lv_gobj[i]);
            }
            (*gp).u.map.set_xC4_b26(1);
        }
        14 => {
            grLast_8021C40C(gobj, grNLa_804DBBDC, 60.0);
            for i in 0..5usize {
                ext::Ground_801C4A08((*gp).u.map.lv_gobj[i]);
                (*gp).u.map.lv_gobj[i] = null_mut();
            }
            (*gp).u.map.lv_gobj[5] = grLast_8021A7F4(grNLa_803E8010[5][0] as i32);
            assert_(!(*gp).u.map.lv_gobj[5].is_null(), 0x4D2, "gp->u.map.lv_gobj[5]");
            ext::HSD_GObjGXLink_803909D8((*gp).u.map.lv_gobj[5], gobj);
            do_anime((*gp).u.map.lv_gobj[5], 5, 0);
            ext::grMaterial_801C9604((*gp).u.map.lv_gobj[5], (*yp).x4, 0);
            ext::grMaterial_801C9698((*gp).u.map.lv_gobj[5]);
            ext::Ground_801C1E00(0);
            (*gp).u.map.set_xC4_b26(1);
            grLast_8021C640();
        }
        15 => {
            (*gp).u.map.set_xC4_b26(0);
        }
        16 => {
            grLast_8021C40C(gobj, grNLa_804DBBE0, 60.0);
            ext::grMaterial_801C9604((*gp).u.map.lv_gobj[5], (*yp).x8, 0);
            ext::grMaterial_801C9698((*gp).u.map.lv_gobj[5]);
            (*gp).u.map.set_xC4_b26(1);
        }
        17 => {
            for i in 0..5usize {
                (*gp).u.map.lv_gobj[i] = grLast_8021A7F4(grNLa_803E8010[i][0] as i32);
                assert_(!(*gp).u.map.lv_gobj[i].is_null(), 0x4F0, "gp->u.map.lv_gobj[i]");
                ext::HSD_GObjGXLink_803909D8((*gp).u.map.lv_gobj[i], gobj);
                if grNLa_803E8010[i][2] != -1 {
                    ext::grAnime_801C7FF8(
                        (*gp).u.map.lv_gobj[i],
                        grNLa_803E8010[i][2] as i32,
                        7,
                        grNLa_803E8010[i][3] as i32,
                        0.0,
                        1.0,
                    );
                }
                do_anime((*gp).u.map.lv_gobj[i], i, 0);
                ext::grMaterial_801C9604((*gp).u.map.lv_gobj[i], (*yp).xC, 0);
                ext::grMaterial_801C9698((*gp).u.map.lv_gobj[i]);
            }

            ext::Ground_801C4A08((*gp).u.map.lv_gobj[5]);
            (*gp).u.map.lv_gobj[5] = null_mut();
            ext::Ground_801C1E00(1);
            (*gp).u.map.set_xC4_b26(1);
        }
        _ => {}
    }
}

pub unsafe fn grLast_8021C40C(gobj: *mut HSD_GObj, color: GXColor, arg2: f32) {
    melee_core::ct!("grLast_8021C40C");
    let mut sp18 = GXColor { r: 0, g: 0, b: 0, a: 0 };
    let gp = GET_GROUND(gobj);
    ext::Ground_801C2090(&mut sp18);
    (*gp).u.map.xE4 = color.r as f32;
    (*gp).u.map.xE8 = color.g as f32;
    (*gp).u.map.xEC = color.b as f32;
    (*gp).u.map.xF4 = sp18.r as f32;
    (*gp).u.map.xF8 = sp18.g as f32;
    (*gp).u.map.xFC = sp18.b as f32;
    (*gp).u.map.x104 = arg2;
    (*gp).u.map.set_xC4_b27(0);
}

/// `(u8) (float)`: gwtool saturates to 32 bits, then truncates.
#[inline(always)]
fn f2u8(x: f32) -> u8 {
    (melee_core::ppc::fctiwz_f32(x)) as u8
}

pub unsafe fn grLast_8021C500(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLast_8021C500");
    let mut sp10 = GXColor { r: 0, g: 0, b: 0, a: 0 };
    let gp = GET_GROUND(gobj);

    if (*gp).u.map.xC4_b27() == 0 {
        (*gp).u.map.x104 -= 1.0;
        if (*gp).u.map.x104 > 0.0 {
            (*gp).u.map.xF4 += ((*gp).u.map.xE4 - (*gp).u.map.xF4) / (*gp).u.map.x104;
            (*gp).u.map.xF8 += ((*gp).u.map.xE8 - (*gp).u.map.xF8) / (*gp).u.map.x104;
            (*gp).u.map.xFC += ((*gp).u.map.xEC - (*gp).u.map.xFC) / (*gp).u.map.x104;
            sp10.r = f2u8((*gp).u.map.xF4);
            sp10.g = f2u8((*gp).u.map.xF8);
            sp10.b = f2u8((*gp).u.map.xFC);
        } else {
            sp10.r = f2u8((*gp).u.map.xE4);
            sp10.g = f2u8((*gp).u.map.xE8);
            sp10.b = f2u8((*gp).u.map.xEC);
            (*gp).u.map.set_xC4_b27(1);
        }
        ext::Ground_801C205C(&mut sp10);
        ext::Camera_SetBackgroundColor(sp10.r, sp10.g, sp10.b);
    }
}

pub unsafe fn grLast_8021C640() {
    melee_core::ct!("grLast_8021C640");
    let mut sp8 = GXColor { r: 0x0C, g: 0x06, b: 0x28, a: 0xFF };
    ext::Ground_801C052C(&mut sp8);
    ext::Ground_801C05D4(&mut sp8);
    ext::Ground_801C0544(&mut sp8);
    ext::Ground_801C058C(&mut sp8);
    ext::Ground_801C05EC(&mut sp8);
    ext::Ground_801C05A4(&mut sp8);
    ext::Ground_801C055C(&mut sp8);
    ext::Ground_801C05BC(&mut sp8);
    ext::Ground_801C0574(&mut sp8);
}

pub unsafe fn grLast_8021C6AC() {
    melee_core::ct!("grLast_8021C6AC");
    let mut sp1c = GXColor { r: 0x8C, g: 0xB4, b: 0xBE, a: 0xFF };
    let mut sp18 = GXColor { r: 0x96, g: 0xB4, b: 0xBE, a: 0xFF };
    let mut sp14 = GXColor { r: 0x6E, g: 0x78, b: 0x8C, a: 0xFF };
    let mut sp10 = GXColor { r: 0x00, g: 0x00, b: 0x00, a: 0xFF };
    let mut spc = GXColor { r: 0x73, g: 0xA0, b: 0x91, a: 0xFF };
    let mut sp8 = GXColor { r: 0x87, g: 0xA5, b: 0xA0, a: 0xFF };
    ext::Ground_801C052C(&mut sp1c);
    ext::Ground_801C05D4(&mut sp18);
    ext::Ground_801C0544(&mut sp1c);
    ext::Ground_801C058C(&mut sp14);
    ext::Ground_801C05EC(&mut sp10);
    ext::Ground_801C05A4(&mut sp14);
    ext::Ground_801C055C(&mut spc);
    ext::Ground_801C05BC(&mut sp8);
    ext::Ground_801C0574(&mut spc);
}

/// TARGET_PC (grlast_stage_slots.inc): FD's group-0 callback pair, never an m-ex-overridden table.
pub unsafe fn Ground_StageSlotProofCallbacks() -> *mut StageCallbacks {
    melee_core::ct!("Ground_StageSlotProofCallbacks");
    &mut grLast_StageCallbacks[0]
}

/// TARGET_PC (grlast_stage_slots.inc): case 1 of `grLast_8021B920` without its generator / camera
/// globals / RNG / backdrop-cycle callbacks.
pub unsafe fn Ground_StageSlotFDBackdrop() {
    melee_core::ct!("Ground_StageSlotFDBackdrop");
    for i in 0..5usize {
        let g = ext::Ground_GetMapGObj(grNLa_803E8010[i][0] as i32);
        if g.is_null() {
            continue;
        }
        ext::grAnime_801C7FF8(g, grNLa_803E8010[i][2] as i32, 7, grNLa_803E8010[i][3] as i32, 0.0, 1.0);
        do_anime(g, i, 0);
    }
    if !ext::Ground_GetMapGObj(4).is_null() {
        ext::grAnime_801C8098(ext::Ground_GetMapGObj(4), 0x10, 7, 0, 0.0, 1.0);
    }
}

// ---- folded from grlast.rs (consolidation steps 21/22) ----
/// grLast_OnStart_fp (gr/grlast.c:251)
    pub unsafe extern "C" fn grLast_OnStart_fp() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

// ---- folded from grlast.rs (consolidation steps 21/22) ----
/// grLast_8021A7EC (gr/grlast.c:256)
    pub extern "C" fn grLast_8021A7EC() -> i32 { 0 }

/// grLast_8021A90C (gr/grlast.c:294)
    pub extern "C" fn grLast_8021A90C(a0: *mut HSD_GObj) -> i32 { 0 }

/// grLast_8021A960 (gr/grlast.c:310)
    pub extern "C" fn grLast_8021A960(a0: *mut HSD_GObj) -> i32 { 0 }

/// grLast_8021A99C (gr/grlast.c:324)
    pub extern "C" fn grLast_8021A99C(a0: *mut HSD_GObj) -> i32 { 0 }

/// grLast_8021AAA8 (gr/grlast.c:359)
    pub extern "C" fn grLast_8021AAA8(a0: *mut HSD_GObj) -> i32 { 0 }

/// grLast_8021AB78 (gr/grlast.c:402)
    pub extern "C" fn grLast_8021AB78(a0: *mut HSD_GObj) -> i32 { 0 }

/// grLast_8021ABCC (gr/grlast.c:418)
    pub extern "C" fn grLast_8021ABCC(a0: *mut HSD_GObj) -> i32 { 0 }

/// grLast_8021AC20 (gr/grlast.c:434)
    pub extern "C" fn grLast_8021AC20(a0: *mut HSD_GObj) -> i32 { 0 }

/// grLast_8021ADC8 (gr/grlast.c:470)
    pub extern "C" fn grLast_8021ADC8(a0: *mut HSD_GObj) -> i32 { 0 }

/// grLast_8021B284 (gr/grlast.c:563)
    pub extern "C" fn grLast_8021B284(a0: *mut HSD_GObj) -> i32 { 0 }

/// grLast_8021B2D8 (gr/grlast.c:579)
    pub extern "C" fn grLast_8021B2D8(a0: *mut HSD_GObj) -> i32 { 0 }

/// grLast_OnTouchLine (gr/grlast.c:1042)
    pub extern "C" fn grLast_OnTouchLine(a0: i32) -> *mut DynamicsDesc { core::ptr::null_mut() }

/// grLast_OnCheckShadowRender (gr/grlast.c:1047)
    pub extern "C" fn grLast_OnCheckShadowRender(a0: *mut Vec3, a1: i32, a2: *mut HSD_JObj) -> i32 { 1 }
