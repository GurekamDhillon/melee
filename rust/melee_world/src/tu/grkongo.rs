//! src/melee/gr/grkongo.c: Kongo Jungle (Gr_Kind_Kongo, also Jungle Japes' test / castle / shrine /
//! zebes / kraid / yorster variants): the swinging barrel cannon (`kongo3`), the rotating rope bridge
//! of 15 segments whose sag drives the map lines (`grKg_803E188C`), the stage-hazard rolls, the
//! two spline-riding platforms (`kongo2`) and the file statics (`yakumono_param`, `grKg_804D6984`,
//! the callback / stage-data tables). State is the per-gobj `Ground` (`u.kongo` / `u.kongo2` /
//! `u.kongo3`); every other callee is a TODO hook in [`ext`]. The constant-return / empty
//! callbacks that an earlier round ported are re-exported from `crate::gr::trivial::grkongo`.
//!
//! Float code fuses exactly where the shipping IR does (`tools/rust-port/fma_sites.py`): 801D577C
//! (3 f32), 801D6074 (1 f64), 801D637C (2 f64), 801D6668 (3 f64 Newton steps), 801D6AFC (4 f32),
//! 801D77E0 (1 f32), 801D7BBC (4 f32), fn_801D542C (1), fn_801D7700 (1), fn_801D8134 (3). Float to
//! int casts saturate to 32 bits first, then truncate to the field width (gwtool). `tanf` and
//! `__frsqrte` are the gekko ones (`melee_runtime::gekko`, `melee_core::fp`).
//! `grKongo_801D77E0` steps a pointer by GC +0x10 from the `kongo` record (xC4/xC8, then xD4/xD8);
//! the port names those fields.
//! State-oracle checked against the C (tests/gro_grkongo.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use super::grinl::{
    assert_, GET_GROUND, GET_JOBJ, HSD_JObjGetRotationZ, HSD_JObjGetTranslation, HSD_JObjSetRotation, HSD_JObjSetRotationX,
    HSD_JObjSetRotationZ, HSD_JObjSetTranslate, HSD_JObjSetTranslateX, ABS, M_TAU,
};
use super::grinl2::{rand_range, Ground_ClampSymmetric, Ground_UpdateWindAndMapColl, Ground_WrapAngle};
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_core::fp::{fmadds, frsqrte};
use melee_runtime::gekko::tanf;
use melee_types::*;

pub use crate::gr::trivial::grkongo::{
    grKongo_801D5338, grKongo_801D55D4, grKongo_801D5574, grKongo_801D5774, grKongo_801D5FD4, grKongo_801D6028,
    grKongo_801D6064, grKongo_801D6190, grKongo_801D6378, grKongo_801D64B4, grKongo_801D6518, grKongo_801D6660,
    grKongo_801D8444,
};

/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::{stage_info, HSD_JObjSetMtxDirtySub};
    pub use crate::tu::grinl2::ext::{
        grAnime_801C8138, grDisplay_801C5DB0, GObj_SetupGXLink, Ground_InitMapColl, Ground_UpdateMapColl,
        HSD_GObj_SetupProc, HSD_Randi, lb_800115F4,
    };
    pub static mut GROUND_801C10B8: Option<fn(*mut HSD_GObj, HSD_GObjEvent)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C10B8(a0: *mut HSD_GObj, a1: HSD_GObjEvent) {
        match unsafe { GROUND_801C10B8 } { Some(f) => f(a0, a1), None => { unsafe { let _ = crate::tu::ground::Ground_801C10B8(a0 as _, a1 as _); } return; } }
    }
    pub static mut GROUND_801C247C: Option<fn(s32, s32) -> *mut HSD_Spline> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C247C(a0: s32, a1: s32) -> *mut HSD_Spline {
        match unsafe { GROUND_801C247C } { Some(f) => f(a0, a1), None => { return unsafe { crate::tu::ground::Ground_801C247C(a0 as _, a1 as _) as _ }; } }
    }
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
    pub static mut GROUND_801C4B50: Option<fn(*mut HSD_Spline, *mut Vec3, *mut Vec3, f32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C4B50(spline: *mut HSD_Spline, arg1: *mut Vec3, result: *mut Vec3, arg8: f32) {
        match unsafe { GROUND_801C4B50 } { Some(f) => f(spline, arg1, result, arg8), None => { unsafe { let _ = crate::tu::ground::Ground_801C4B50(spline as _, arg1 as _, result as _, arg8 as _); } return; } }
    }
    pub static mut GROUND_801C4D70: Option<fn(*mut HSD_GObj, *mut Vec3, f32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C4D70(a0: *mut HSD_GObj, a1: *mut Vec3, a2: f32) -> i32 {
        match unsafe { GROUND_801C4D70 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::ground::Ground_801C4D70(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GROUND_801C4DA0: Option<fn(*mut Vec3, *mut f32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C4DA0(a0: *mut Vec3, a1: *mut f32) -> i32 {
        match unsafe { GROUND_801C4DA0 } { Some(f) => f(a0, a1), None => { return unsafe { crate::tu::ground::Ground_801C4DA0(a0 as _, a1 as _) as _ }; } }
    }
    pub static mut GROUND_801C53EC: Option<fn(u32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C53EC(a0: u32) {
        match unsafe { GROUND_801C53EC } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C53EC(a0 as _); } return; } }
    }
    pub static mut GROUND_801C5440: Option<fn(*mut Ground, s32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5440(gp: *mut Ground, i: s32, a2: u32) {
        match unsafe { GROUND_801C5440 } { Some(f) => f(gp, i, a2), None => { unsafe { let _ = crate::tu::ground::Ground_801C5440(gp as _, i as _, a2 as _); } return; } }
    }
    pub static mut GROUND_801C5630: Option<fn(*mut Ground, s32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5630(gp: *mut Ground, i: s32, val: f32) {
        match unsafe { GROUND_801C5630 } { Some(f) => f(gp, i, val), None => { unsafe { let _ = crate::tu::ground::Ground_801C5630(gp as _, i as _, val as _); } return; } }
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
    hook!(HSD_RANDF, HSD_Randf, () -> f32);
    hook_or!(OSREPORT, OSReport, (a0: *mut i8), |a0: *mut i8| unsafe { melee_runtime::tu::os_error::OSReport(a0 as _, &[]) });
    pub static mut STAGE_80225194: Option<fn() -> StKind> = None;
    #[allow(non_snake_case)]
    pub fn Stage_80225194() -> StKind {
        match unsafe { STAGE_80225194 } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_80225194() as _ }; } }
    }
    /// `efSync_Spawn(gfx_id, arg_gobj, ...)`: variadic; here its one trailing `Vec3*`.
    pub static mut EFSYNC_SPAWN: Option<fn(s32, *mut HSD_GObj, *mut Vec3) -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn efSync_Spawn(gfx_id: s32, arg_gobj: *mut HSD_GObj, pos: *mut Vec3) -> *mut c_void {
        match unsafe { EFSYNC_SPAWN } {
            Some(f) => f(gfx_id, arg_gobj, pos),
            None => { unsafe { crate::tu::efsync::efSync_Spawn(gfx_id as _, arg_gobj as _, &mut *(pos as *mut _)) as _ } },
        }
    }
    hook!(FTCO_8009EC70, ftCo_8009EC70, (gobj: *mut HSD_GObj, pos: *mut Vec3, arg2: *mut lbColl_80008D30_arg1, kb_angle: f32));
    hook!(FTCO_800C0764, ftCo_800C0764, (a0: *mut HSD_GObj, a1: u32, a2: *mut c_void));
    hook_or!(FTLIB_80086644, ftLib_80086644, (a0: *mut HSD_GObj, a1: *mut Vec3), |a0: *mut HSD_GObj, a1: *mut Vec3| { unsafe { let _ = crate::ftlib::ftLib_80086644(a0 as _, a1 as _); } });
    hook!(FTLIB_80086C18, ftLib_80086C18, (a0: *mut HSD_GObj, a1: s32, a2: s32));
    hook_or!(GM_8016B238, gm_8016B238, () -> i32, || unsafe { crate::gm::gm_8016B238() as _ });
    pub static mut GRANIME_801C78FC: Option<fn(*mut HSD_GObj, i32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C78FC(a0: *mut HSD_GObj, a1: i32, a2: u32) {
        match unsafe { GRANIME_801C78FC } { Some(f) => f(a0, a1, a2), None => { unsafe { let _ = crate::tu::granime::grAnime_801C78FC(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRANIME_801C7A04: Option<fn(*mut HSD_GObj, i32, u32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C7A04(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32) {
        match unsafe { GRANIME_801C7A04 } { Some(f) => f(a0, a1, a2, a3), None => { unsafe { let _ = crate::tu::granime::grAnime_801C7A04(a0 as _, a1 as _, a2 as _, a3 as _); } return; } }
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
    pub static mut GRMATERIAL_801C94D8: Option<fn(*mut c_void)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C94D8(a0: *mut c_void) {
        match unsafe { GRMATERIAL_801C94D8 } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C94D8(a0 as _); } return; } }
    }
    pub static mut GRMATERIAL_801C95C4: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C95C4(a0: *mut HSD_GObj) {
        match unsafe { GRMATERIAL_801C95C4 } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C95C4(a0 as _); } return; } }
    }
    pub static mut GRMATERIAL_801C9604: Option<fn(*mut HSD_GObj, *mut c_void, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C9604(bg: *mut HSD_GObj, a1: *mut c_void, a2: i32) {
        match unsafe { GRMATERIAL_801C9604 } {
            Some(f) => f(bg, a1, a2),
            None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C9604(bg as _, a1 as _, (a2 != 0)); } },
        }
    }
    pub static mut GRZAKOGENERATOR_801CAE04: Option<fn(*mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj {
        match unsafe { GRZAKOGENERATOR_801CAE04 } {
            Some(f) => f(a0),
            None => { unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ } },
        }
    }
    hook!(ITGETKIND, itGetKind, (gobj: *mut HSD_GObj) -> ItemKind);
    hook!(IT_8026B294, it_8026B294, (gobj: *mut HSD_GObj, pos: *mut Vec3));
    hook!(IT_8026D324, it_8026D324, (a0: ItemKind) -> i32);
    hook!(IT_80286088, it_80286088, (a0: *mut HSD_GObj) -> *mut HSD_GObj);
    hook!(IT_802E18B4, it_802E18B4, (gobj: *mut HSD_GObj) -> *mut HSD_GObj);
    hook!(IT_802E20D8, it_802E20D8, (a0: *mut HSD_GObj));
    hook!(IT_802E2330, it_802E2330, (a0: *mut HSD_GObj, a1: *mut Vec3, a2: *mut lbColl_80008D30_arg1, a3: f32));
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    pub static mut MPJOINTCLEARCB1: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpJointClearCb1(joint_id: i32) {
        match unsafe { MPJOINTCLEARCB1 } { Some(f) => f(joint_id), None => { unsafe { let _ = crate::tu::mplib::mpJointClearCb1(joint_id as _); } return; } }
    }
    pub static mut MPJOINTLISTADD: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpJointListAdd(joint_id: i32) {
        match unsafe { MPJOINTLISTADD } { Some(f) => f(joint_id), None => { unsafe { let _ = crate::tu::mplib::mpJointListAdd(joint_id as _); } return; } }
    }
    pub static mut MPJOINTSETB10: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpJointSetB10(joint_id: i32) {
        match unsafe { MPJOINTSETB10 } { Some(f) => f(joint_id), None => { unsafe { let _ = crate::tu::mplib::mpJointSetB10(joint_id as _); } return; } }
    }
    pub static mut MPJOINTSETCB1: Option<fn(i32, *mut c_void, mpLib_JointCollisionCallback)> = None;
    #[allow(non_snake_case)]
    pub fn mpJointSetCb1(joint_id: i32, user_data: *mut c_void, cb: mpLib_JointCollisionCallback) {
        match unsafe { MPJOINTSETCB1 } { Some(f) => f(joint_id, user_data, cb), None => { unsafe { let _ = crate::tu::mplib::mpJointSetCb1(joint_id as _, user_data as _, cb as _); } return; } }
    }
    pub static mut MPLIB_8005667C: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_8005667C(joint_id: i32) {
        match unsafe { MPLIB_8005667C } { Some(f) => f(joint_id), None => { unsafe { let _ = crate::tu::mplib::mpLib_8005667C(joint_id as _); } return; } }
    }
    pub static mut MPLIB_80056758: Option<fn(i32, f32, f32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_80056758(line_id: i32, x0: f32, y0: f32, x1: f32, y1: f32) {
        match unsafe { MPLIB_80056758 } { Some(f) => f(line_id, x0, y0, x1, y1), None => { unsafe { let _ = crate::tu::mplib::mpLib_80056758(line_id as _, x0 as _, y0 as _, x1 as _, y1 as _); } return; } }
    }
    pub static mut MPLIB_80057424: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_80057424(joint_id: i32) {
        match unsafe { MPLIB_80057424 } { Some(f) => f(joint_id), None => { unsafe { let _ = crate::tu::mplib::mpLib_80057424(joint_id as _); } return; } }
    }
    pub static mut MPLIB_80057BC0: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_80057BC0(joint_id: i32) {
        match unsafe { MPLIB_80057BC0 } { Some(f) => f(joint_id), None => { unsafe { let _ = crate::tu::mplib::mpLib_80057BC0(joint_id as _); } return; } }
    }
    pub static mut MPVTXGETPOS: Option<fn(i32, *mut f32, *mut f32)> = None;
    #[allow(non_snake_case)]
    pub fn mpVtxGetPos(vtx_id: i32, x_out: *mut f32, y_out: *mut f32) {
        match unsafe { MPVTXGETPOS } { Some(f) => f(vtx_id, x_out, y_out), None => { unsafe { let _ = crate::tu::mplib::mpVtxGetPos(vtx_id as _, x_out as _, y_out as _); } return; } }
    }
    hook!(SPLARCLENGTHPOINT, splArcLengthPoint, (a0: *mut Vec3, a1: *mut HSD_Spline, a2: f32));
    /// `HSD_GObj** HSD_GObjPLinkHead`.
    pub static mut HSD_GObjPLinkHead: *mut *mut HSD_GObj = core::ptr::null_mut();
}

/// `struct grKongo_YakumonoParam` (grkongo.h; native layout like the C compiled natively).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grKongo_YakumonoParam {
    pub unk0: f32,
    pub unk4: f32,
    pub unk8: f32,
    pub unkC: f32,
    pub unk10: f32,
    pub unk14: f32,
    pub unk18: f32,
    pub unk1C: f32,
    pub unk20: f32,
    pub unk24: f32,
    pub unk28: f32,
    pub unk2C: f32,
    pub unk30: f32,
    pub unk34: f32,
    pub unk38: f32,
    pub unk3C: f32,
    pub unk40: f32,
    pub unk44: i16,
    pub unk46: i16,
    pub unk48: i16,
    pub unk4A: i16,
    pub unk4C: i16,
    pub unk4E: i16,
    pub unk50: i16,
    pub unk52: i16,
    pub unk54: f32,
    pub unk58: f32,
    pub unk5C: f32,
    pub unk60: f32,
    pub unk64: i32,
    pub unk68: i32,
    pub unk6C: f32,
    pub unk70: f32,
    pub unk74: f32,
    pub unk78: f32,
    pub unk7C: f32,
    pub unk80: f32,
    pub unk84: *mut c_void,
    pub unk88: f32,
    pub unk8C: f32,
    pub unk90: f32,
    pub unk94: f32,
    pub unk98: f32,
    pub unk9C: f32,
    pub unkA0: f32,
    pub unkA4: f32,
    pub unkA8: f32,
    pub unkAC: f32,
    pub unkB0: f32,
    pub unkB4: f32,
    pub unkB8: f32,
}

/// `grKg_SplineChoice` / `grKg_SplineChoiceList` (grkongo.h).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grKg_SplineChoice {
    pub file: s32,
    pub spline: s32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grKg_SplineChoiceList {
    pub pairs: [grKg_SplineChoice; 3],
    pub terminator: s32,
}

/// `struct _struct_grKg_804D6984`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grKg_804D6984_t {
    pub unk0: *mut HSD_JObj,
    pub unk4: *mut HSD_JObj,
}

/// `struct _struct_grKg_803E188C_0x18`: one rope-bridge segment (jobj, rest angle, angle, velocity, sag).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grKg_803E188C_t {
    pub unk0: i16,
    pub unk2: i16,
    pub unk4: *mut HSD_JObj,
    pub unk8: f32,
    pub unkC: f32,
    pub unk10: f32,
    pub unk14: f32,
}

/// `static struct grKongo_YakumonoParam* yakumono_param` (grkongo.h, one copy per TU).
pub static mut yakumono_param: *mut grKongo_YakumonoParam = null_mut();

/// `GrJoint grKg_803E16E0[]`.
pub static mut grKg_803E16E0: [GrJoint; 6] = [
    GrJoint { x: 2, y: 10, z: 19 },
    GrJoint { x: 3, y: 10, z: 22 },
    GrJoint { x: 5, y: 10, z: 43 },
    GrJoint { x: 6, y: 10, z: 44 },
    GrJoint { x: 0, y: 1, z: 0 },
    GrJoint { x: 1, y: 2, z: 2 },
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

/// `StageCallbacks grKg_StageCallbacks[12]`.
pub static mut grKg_StageCallbacks: [StageCallbacks; 12] = [
    StageCallbacks {
        on_init: None,
        callback1: None,
        gobj_proc: None,
        callback3: None,
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    cbs!(grKongo_801D6074, grKongo_801D6190, grKongo_801D6198, grKongo_801D6378, 0),
    cbs!(grKongo_801D6074, grKongo_801D6190, grKongo_801D6198, grKongo_801D6378, 0),
    cbs!(grKongo_801D6038, grKongo_801D6064, grKongo_801D606C, grKongo_801D6070, 0),
    cbs!(grKongo_801D5FA8, grKongo_801D5FD4, grKongo_801D5FDC, grKongo_801D5FE0, 0),
    cbs!(grKongo_801D55D8, grKongo_801D5774, grKongo_801D577C, grKongo_801D5FA4, 0),
    cbs!(grKongo_801D5FE4, grKongo_801D6028, grKongo_801D6030, grKongo_801D6034, 0),
    cbs!(grKongo_801D637C, grKongo_801D64B4, grKongo_801D64BC, grKongo_801D6518, 0),
    cbs!(grKongo_801D637C, grKongo_801D64B4, grKongo_801D64BC, grKongo_801D6518, 0),
    cbs!(grKongo_801D637C, grKongo_801D64B4, grKongo_801D64BC, grKongo_801D6518, 0),
    cbs!(grKongo_801D5490, grKongo_801D5574, grKongo_801D557C, grKongo_801D55D4, (1u32 << 30) | (1u32 << 31)),
    cbs!(grKongo_801D651C, grKongo_801D6660, grKongo_801D6668, grKongo_801D69AC, 0),
];

/// `StageData grKg_StageData`.
pub static mut grKg_StageData: StageData = StageData {
    grkind: Gr_Kind_Kongo,
    callbacks: unsafe { core::ptr::addr_of!(grKg_StageCallbacks) as *mut StageCallbacks },
    data1: b"/GrKg.dat\0".as_ptr() as *mut i8,
    on_init: Some(grKongo_801D523C),
    on_demo_init: Some(grKongo_801D5238),
    on_load: Some(grKongo_801D52F8),
    on_start: Some(grKongo_801D52FC),
    callback4: Some(grKongo_801D5338),
    on_touch_line: Some(grKongo_801D8444),
    on_check_shadow_render: Some(grKongo_801D844C),
    flags2: 1 << 0,
    joints: unsafe { core::ptr::addr_of!(grKg_803E16E0) as *mut GrJoint },
    joint_count: 6,
};

/// `static const lbColl_80008D30_arg1 grKg_803B7FB0`.
pub static grKg_803B7FB0: lbColl_80008D30_arg1 = lbColl_80008D30_arg1 {
    state: HitCapsule_Enabled,
    damage: 1,
    kb_angle: 361,
    unkC: 0,
    unk10: 0,
    unk14: 180,
    element: 0,
    sfx_severity: 0,
    sfx_kind: 0,
};

/// `const grKg_SplineChoiceList grKg_803B7FD4`.
pub static grKg_803B7FD4: grKg_SplineChoiceList = grKg_SplineChoiceList {
    pairs: [grKg_SplineChoice { file: 7, spline: 4 }, grKg_SplineChoice { file: 8, spline: 5 }, grKg_SplineChoice { file: 9, spline: 6 }],
    terminator: 0,
};

/// `struct _struct_grKg_804D6984 grKg_804D6984`.
pub static mut grKg_804D6984: grKg_804D6984_t = grKg_804D6984_t { unk0: null_mut(), unk4: null_mut() };

/// `static struct _struct_grKg_803E188C_0x18 grKg_803E188C[0xF]`.
pub static mut grKg_803E188C: [grKg_803E188C_t; 15] = [
    seg(8, 0.06981317),
    seg(9, 0.0),
    seg(0xA, -0.05235988),
    seg(0x17, -0.10471976),
    seg(0x18, -0.13962634),
    seg(0x19, -0.15707964),
    seg(0x1A, -0.20943952),
    seg(0x1B, -0.23561944),
    seg(0x1C, -0.18325958),
    seg(0x1D, -0.1308997),
    seg(0x1E, -0.07853982),
    seg(0x1F, -0.02617994),
    seg(0x20, 0.0),
    seg(7, 0.05235988),
    seg(0x2D, 0.10471976),
];

const fn seg(unk0: i16, unk8: f32) -> grKg_803E188C_t {
    grKg_803E188C_t { unk0, unk2: 0, unk4: null_mut(), unk8, unkC: 0.0, unk10: 0.0, unk14: 0.0 }
}

/// `MTXDegToRad(a)` (macro in mtx.h): `(a) * 0.01745329252f`.
#[inline(always)]
fn deg2rad(a: f32) -> f32 {
    a * 0.01745329252f32
}

/// `(s16) (float)`: gwtool saturates to 32 bits, then truncates.
#[inline(always)]
fn f2s16(x: f32) -> i16 {
    (x as i32) as i16
}

pub unsafe extern "C" fn grKongo_801D5238(_arg: i32) {}

pub unsafe extern "C" fn grKongo_801D523C() {
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grKongo_YakumonoParam;
    let si = ext::stage_info();
    (*si).unk8C.set_b4(0);
    (*si).unk8C.set_b5(1);
    grKongo_801D5340(0);
    grKongo_801D5340(10);
    grKongo_801D5340(5);
    grKongo_801D5340(3);
    grKongo_801D5340(6);
    grKongo_801D5340(4);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::mpLib_80057BC0(0);
    ext::mpLib_80057BC0(1);
    if ext::Stage_80225194() != 0x3D && ext::gm_8016B238() == 0 {
        ext::Ground_801C53EC(0x5A551);
    }
}

pub unsafe extern "C" fn grKongo_801D52F8() {}

pub unsafe extern "C" fn grKongo_801D52FC() {
    let gobj = ext::Ground_GetMapGObj(5);
    ext::ftCo_800C0764(gobj, 2, fn_801D8134 as usize as *mut c_void);
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe fn grKongo_801D5340(gobj_id: i32) -> *mut HSD_GObj {
    let callbacks: *mut StageCallbacks = &mut grKg_StageCallbacks[gobj_id as usize];
    let gobj = ext::Ground_GetStageGObj(gobj_id);

    if !gobj.is_null() {
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
    } else {
        let mut msg = *b"%s:%d: couldn t get gobj(id=%d)\n\0";
        ext::OSReport(msg.as_mut_ptr() as *mut i8);
    }

    gobj
}

/// `void fn_801D542C(HSD_GObj*)`: the barrel's (Ground_801C10B8) callback.
pub unsafe extern "C" fn fn_801D542C(arg0: *mut HSD_GObj) {
    let gp = (*arg0).user_data as *mut Ground;
    ext::mpJointSetCb1(4, gp as *mut c_void, Some(fn_801D7700));
    let rand = ext::HSD_Randf();
    let min = (*yakumono_param).unk0;
    let range = (*yakumono_param).unk4 - min;
    (*gp).u.kongo.xE4 = f2s16(fmadds(range, rand, min));
}

pub unsafe extern "C" fn grKongo_801D5490(arg0: *mut HSD_GObj) {
    let gp = (*arg0).user_data as *mut Ground;
    ext::Ground_InitMapColl((*arg0).hsd_obj as *mut HSD_JObj, (*gp).map_id as s32);
    ext::grAnime_801C8138(arg0, (*gp).map_id as enum_t, 0);
    (*gp).x10_flags.set_b5(1);
    (*gp).u.kongo.xE4 = -1;
    (*gp).u.kongo.xE6 = -1;
    (*gp).u.kongo.xC4 = 0.0;
    (*gp).u.kongo.xC8 = 0.0;
    (*gp).u.kongo3.xCC = ext::Ground_801C3FA4(arg0, 0x13);
    (*gp).u.kongo.u.taru.keep = ext::Ground_801C3FA4(arg0, 0xD) as *mut c_void;
    (*gp).u.kongo.xD4 = 0.0;
    (*gp).u.kongo.xD8 = 0.0;
    (*gp).u.kongo.xDC = ext::Ground_801C3FA4(arg0, 0x2B);
    (*gp).u.kongo.xE0 = ext::Ground_801C3FA4(arg0, 0x28);
    grKongo_801D69B0(arg0);
    ext::Ground_801C10B8(arg0, Some(fn_801D542C));
    ext::Ground_UpdateMapColl(arg0);
}

pub unsafe extern "C" fn grKongo_801D557C(arg0: *mut HSD_GObj) {
    grKongo_801D7134(arg0, 0);
    grKongo_801D77E0(arg0, 0);
    grKongo_801D7BBC(arg0);
    Ground_UpdateWindAndMapColl(arg0);
    ext::mpLib_8005667C(4);
}

pub unsafe extern "C" fn grKongo_801D55D8(arg0: *mut HSD_GObj) {
    let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = (*arg0).user_data as *mut Ground;
    let temp_r28 = (*arg0).hsd_obj;
    ext::grAnime_801C8138(arg0, (*gp).map_id as enum_t, 0);
    ext::grMaterial_801C94D8(temp_r28);
    (*gp).u.kongo3.xE4 = 1.0;
    (*gp).u.kongo.xE8 = 1.0;
    (*gp).u.kongo.xD4 = 0.0;
    (*gp).u.kongo.xD8 = 0.0;
    (*gp).u.kongo3.xC4 = 0;
    (*gp).u.kongo3.xC6 = 0;
    (*gp).u.kongo3.xC8 = 2;
    (*gp).u.kongo.u.taru.keep = null_mut();
    (*gp).u.kongo3.xDC = 0.0;
    (*gp).u.kongo3.xE0 = 0.0;
    (*gp).u.kongo.xE8 = (*yakumono_param).unk60;
    (*gp).u.kongo2.xCC = rand_range((*yakumono_param).unk30 as i32, (*yakumono_param).unk2C as i32) as i16;
    (*gp).u.kongo2.xCE = rand_range((*yakumono_param).unk68, (*yakumono_param).unk64) as i16;
    ext::lb_8000B1CC(ext::Ground_801C3FA4(arg0, 1), null_mut(), &mut sp14);
    ext::Ground_801C4D70(arg0, &mut sp14, (*gp).u.kongo.xD8);
}

pub unsafe extern "C" fn grKongo_801D577C(arg0: *mut HSD_GObj) {
    let gp = (*arg0).user_data as *mut Ground;
    let jobj = ext::Ground_801C3FA4(arg0, 1);
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let yp = yakumono_param;

    match (*gp).u.kongo3.xC4 {
        2 | 3 => {
            let angular_vel = (*gp).u.kongo3.xE0;
            let limit_angle = 0.5f32 * (angular_vel * (angular_vel / deg2rad((*yp).unk34)));
            let mut angle_delta: f32 = 0.0;
            if angular_vel > 0.0 {
                angle_delta = (*gp).u.kongo3.xD4 - (*gp).u.kongo3.xD8;
            } else if angular_vel < 0.0 {
                angle_delta = (*gp).u.kongo3.xD8 - (*gp).u.kongo3.xD4;
            } else {
                assert_(false, 505, "0");
            }
            if angle_delta < 0.0 {
                angle_delta = (angle_delta as f64 + M_TAU) as f32;
            }
            if angle_delta < limit_angle || angle_delta < ABS((*gp).u.kongo3.xE0) {
                if (*gp).u.kongo3.xC4 == 3 {
                    (*gp).u.kongo3.xC4 = 0;
                }
            } else if (*gp).u.kongo3.xC4 == 2 {
                (*gp).u.kongo3.xC4 = 3;
            }
        }
        0 => {
            if (*gp).u.kongo3.xE0 > 0.0 {
                if (*gp).u.kongo3.xE0 < deg2rad((*yp).unk34) {
                    (*gp).u.kongo3.xE0 = 0.0;
                    (*gp).u.kongo3.xD8 = (*gp).u.kongo3.xD4;
                } else {
                    (*gp).u.kongo3.xE0 = fmadds(-(*yp).unk34, 0.01745329252f32, (*gp).u.kongo3.xE0);
                }
            } else if (*gp).u.kongo3.xE0 < 0.0 {
                if (*gp).u.kongo3.xE0 > -deg2rad((*yp).unk34) {
                    (*gp).u.kongo3.xE0 = 0.0;
                    (*gp).u.kongo3.xD8 = (*gp).u.kongo3.xD4;
                } else {
                    (*gp).u.kongo3.xE0 = fmadds((*yp).unk34, 0.01745329252f32, (*gp).u.kongo3.xE0);
                }
            }
            (*gp).u.kongo2.xCC = (*gp).u.kongo2.xCC.wrapping_sub(1);
            if (*gp).u.kongo2.xCC < 0 {
                (*gp).u.kongo3.xC4 = 1;
                let spin_step = if ext::HSD_Randi(2) != 0 { deg2rad((*yp).unk34) } else { -deg2rad((*yp).unk34) };
                (*gp).u.kongo3.xDC = spin_step;
                (*gp).u.kongo2.xCC = rand_range((*yp).unk40 as i32, (*yp).unk3C as i32) as i16;
            }
        }
        1 => {
            (*gp).u.kongo3.xE0 += (*gp).u.kongo3.xDC;
            let v = (*gp).u.kongo3.xE0;
            Ground_ClampSymmetric(v, deg2rad((*yp).unk38), &mut (*gp).u.kongo3.xE0);
            let val2 = (*gp).u.kongo2.xCC;
            (*gp).u.kongo2.xCC = val2.wrapping_sub(1);
            if val2 < 0 {
                (*gp).u.kongo3.xC4 = 2;
                (*gp).u.kongo2.xCC = rand_range((*yp).unk30 as i32, (*yp).unk2C as i32) as i16;
                (*gp).u.kongo3.xD4 = grKongo_801D8314();
            }
        }
        _ => {}
    }
    (*gp).u.kongo3.xD8 += (*gp).u.kongo3.xE0;
    Ground_WrapAngle(&mut (*gp).u.kongo3.xD8);
    HSD_JObjSetRotationZ(jobj, (*gp).u.kongo.xD8);
    ext::lb_8000B1CC(jobj, null_mut(), &mut vec);
    ext::Ground_801C4D70(arg0, &mut vec, (*gp).u.kongo3.xD8);
    match (*gp).u.kongo3.xC8 {
        0 => {
            let val = (*gp).u.kongo2.xCE;
            (*gp).u.kongo2.xCE = (*gp).u.kongo2.xCE.wrapping_sub(1);
            if val < 0 {
                (*gp).u.kongo3.xC8 = 1;
            }
        }
        1 => {
            (*gp).u.kongo2.xE8 += (*yp).unk5C;
            if (*gp).u.kongo2.xE8 > (*yp).unk60 {
                (*gp).u.kongo2.xE8 = (*yp).unk60;
                (*gp).u.kongo2.xCE = rand_range((*yp).unk68, (*yp).unk64) as i16;
                (*gp).u.kongo3.xC8 = 2;
            }
        }
        2 => {
            let val2 = (*gp).u.kongo2.xCE;
            (*gp).u.kongo2.xCE = val2.wrapping_sub(1);
            if val2 < 0 {
                (*gp).u.kongo3.xC8 = 3;
            }
        }
        3 => {
            (*gp).u.kongo2.xE8 -= (*yp).unk5C;
            if (*gp).u.kongo2.xE8 < 0.0 {
                (*gp).u.kongo2.xE8 = 0.0;
                (*gp).u.kongo2.xCE = rand_range((*yp).unk58.to_bits() as i32, (*yp).unk54.to_bits() as i32) as i16;
                (*gp).u.kongo3.xC8 = 0;
            }
        }
        _ => {}
    }
    ext::grAnime_801C7A04(arg0, 0, 7, (*gp).u.kongo3.xE8);

    // switch (xC6): case 1 falls through into case 2 (also with a NULL `keep`)
    let mut state = (*gp).u.kongo3.xC6;
    if state == 0 {
        let item_gobj = grKongo_801D8078(arg0);
        if !item_gobj.is_null() {
            ext::it_802E20D8(item_gobj);
            let rand_val = ext::HSD_Randf();
            let diff = (*yp).unk24 - (*yp).unk20;
            (*gp).u.kongo3.xCA = f2s16(fmadds(diff, rand_val, (*yp).unk20));
            (*gp).u.kongo.u.taru.keep = item_gobj as *mut c_void;
            (*gp).u.kongo3.xC6 = 1;
            ext::Ground_801C5440(gp, 0, 0x129);
            ext::grMaterial_801C9604(arg0, (*yp).unk84, 0);
        }
        return;
    }
    if state == 1 {
        if (*gp).u.kongo.u.taru.keep.is_null() {
            (*gp).u.kongo3.xC6 = 0;
        } else {
            let v = (*gp).u.kongo3.xCA;
            (*gp).u.kongo3.xCA = v.wrapping_sub(1);
            if v >= 0 {
                return;
            }
            (*gp).u.kongo3.xC6 = 2;
        }
        state = 2;
    }
    if state == 2 {
        let mut hit = grKg_803B7FB0;
        hit.state = HitCapsule_Enabled;
        hit.damage = (*yp).unk6C.to_bits();
        hit.kb_angle = (*yp).unk70.to_bits() as i32;
        hit.unkC = (*yp).unk74.to_bits();
        hit.unk10 = (*yp).unk78.to_bits();
        hit.unk14 = (*yp).unk7C.to_bits();
        hit.element = (*yp).unk80.to_bits();
        let mut angle = (1.5707963267948966f64 + (*gp).u.kongo3.xD8 as f64) as f32;
        if angle < 0.0 {
            angle = (angle as f64 + M_TAU) as f32;
        } else if angle as f64 > M_TAU {
            angle = (angle as f64 - M_TAU) as f32;
        }
        let kept_gobj = (*gp).u.kongo.u.taru.keep as *mut HSD_GObj;
        angle = angle * 57.29578f32;
        if (*kept_gobj).p_link as i32 == HSD_GOBJ_PLINK_FIGHTER {
            ext::ftCo_8009EC70(kept_gobj, &mut vec, &mut hit, angle);
        } else if (*kept_gobj).p_link as i32 == HSD_GOBJ_PLINK_ITEM {
            ext::it_802E2330(kept_gobj, &mut vec, &mut hit, angle);
        }
        (*gp).u.kongo3.xC6 = 3;
        ext::Ground_801C5440(gp, 0, 0x12A);
        ext::grMaterial_801C95C4(arg0);
        return;
    }
    if state == 3 && grKongo_801D8078(arg0).is_null() {
        (*gp).u.kongo3.xC6 = 0;
    }
}

pub unsafe extern "C" fn grKongo_801D5FA4(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKongo_801D5FA8(arg0: *mut HSD_GObj) {
    let gp = (*arg0).user_data as *mut Ground;
    ext::grAnime_801C8138(arg0, (*gp).map_id as enum_t, 0);
}

pub unsafe extern "C" fn grKongo_801D5FDC(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKongo_801D5FE0(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKongo_801D5FE4(arg0: *mut HSD_GObj) {
    let gp = (*arg0).user_data as *mut Ground;
    ext::grAnime_801C8138(arg0, (*gp).map_id as enum_t, 0);
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grKongo_801D6030(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKongo_801D6034(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKongo_801D6038(arg0: *mut HSD_GObj) {
    let gp = (*arg0).user_data as *mut Ground;
    ext::grAnime_801C8138(arg0, (*gp).map_id as enum_t, 0);
}

pub unsafe extern "C" fn grKongo_801D606C(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKongo_801D6070(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKongo_801D6074(arg0: *mut HSD_GObj) {
    let gp = (*arg0).user_data as *mut Ground;
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    ext::grAnime_801C8138(arg0, (*gp).map_id as enum_t, 0);
    (*gp).u.kongo3.xC4 = 0;
    (*gp).u.kongo3.xC6 = 0;
    (*gp).u.kongo3.xC8 = 0;
    (*gp).u.kongo3.xCA = 0;
    (*gp).u.kongo2.xD0 = -99999.0;
    (*gp).u.kongo2.xD4 = 3.4028235e38;
    if (*gp).map_id == Gr_Kind_Test {
        (*gp).u.kongo.xCC = 45.0f64.mul_add(ext::HSD_Randf() as f64, -15.0) as f32;
    } else {
        (*gp).u.kongo.xCC = (9.0f64 * ext::HSD_Randf() as f64) as f32;
    }
    HSD_JObjSetTranslateX(jobj, (*gp).u.kongo.xCC);
}

pub unsafe extern "C" fn grKongo_801D6198(arg0: *mut HSD_GObj) {
    let gp = (*arg0).user_data as *mut Ground;
    match (*gp).u.kongo3.xC4 {
        0 => {
            if ext::grAnime_801C83D0(arg0, 0, 1) != 0 {
                ext::Ground_InitMapColl(GET_JOBJ(arg0), (*gp).map_id as s32);
                ext::Ground_UpdateMapColl(arg0);
                if (*gp).map_id == Gr_Kind_Test {
                    ext::mpJointListAdd(0);
                    ext::mpJointSetCb1(0, gp as *mut c_void, Some(fn_801D7E60));
                } else {
                    ext::mpJointListAdd(1);
                    ext::mpJointSetCb1(1, gp as *mut c_void, Some(fn_801D7E60));
                }
                (*gp).u.kongo3.xC4 = 1;
            }
        }
        1 => {
            (*gp).u.kongo3.xCA = (*gp).u.kongo3.xCA.wrapping_add(1);
            if (*gp).u.kongo3.xC8 == (*gp).u.kongo3.xC6 {
                (*gp).u.kongo3.xC6 = 0;
                (*gp).u.kongo3.xC8 = 0;
            } else {
                (*gp).u.kongo3.xC8 = (*gp).u.kongo3.xC6;
            }
            if ((*gp).u.kongo3.xC6 as f32) > (*yakumono_param).unk88
                || ((*gp).u.kongo3.xCA as f32) > (*yakumono_param).unk8C
                || grKongo_801D7F78(arg0)
            {
                ext::grAnime_801C8138(arg0, (*gp).map_id as enum_t, 1);
                if (*gp).map_id == Gr_Kind_Test {
                    ext::mpLib_80057BC0(0);
                    ext::mpJointClearCb1(0);
                } else {
                    ext::mpLib_80057BC0(1);
                    ext::mpJointClearCb1(1);
                }
                (*gp).u.kongo3.xC4 = 2;
            }
        }
        2 => {
            if ext::grAnime_801C83D0(arg0, 0, 1) != 0 {
                ext::Ground_801C4A08(arg0);
            }
        }
        _ => {}
    }
}

pub unsafe extern "C" fn grKongo_801D637C(arg0: *mut HSD_GObj) {
    let gp = (*arg0).user_data as *mut Ground;
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    ext::grAnime_801C8138(arg0, (*gp).map_id as enum_t, 0);
    ext::grAnime_801C78FC(arg0, 0, 7);
    if (*gp).map_id == Gr_Kind_Shrine {
        (*gp).u.kongo.xCC = 45.0f64.mul_add(ext::HSD_Randf() as f64, -15.0) as f32;
    } else if (*gp).map_id == Gr_Kind_Zebes {
        (*gp).u.kongo.xCC = 20.0f64.mul_add(ext::HSD_Randf() as f64, -10.0) as f32;
    } else {
        (*gp).u.kongo.xCC = (9.0f64 * ext::HSD_Randf() as f64) as f32;
    }
    HSD_JObjSetTranslateX(jobj, (*gp).u.kongo.xCC);
}

pub unsafe extern "C" fn grKongo_801D64BC(arg0: *mut HSD_GObj) {
    if ext::grAnime_801C83D0(arg0, 0, 1) != 0 || ext::grAnime_801C84A4(arg0, 0, 1) != 0 {
        ext::Ground_801C4A08(arg0);
    }
}

pub unsafe extern "C" fn grKongo_801D651C(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut sp2c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let list = grKg_803B7FD4.pairs;
    let i = ext::HSD_Randi(3);
    (*gp).u.kongo2.xC4 = ext::Ground_801C247C(list[i as usize].file, list[i as usize].spline);
    (*gp).u.kongo2.xC8 = 0.0;
    (*gp).u.kongo2.xD0 = 0.0;
    (*gp).u.kongo.xCC = 0.0;
    (*gp).u.kongo2.xD8 = -99999.0;
    (*gp).u.kongo2.xDC = 3.4028235e38;
    ext::splArcLengthPoint(&mut sp2c, (*gp).u.kongo2.xC4, (*gp).u.kongo.xCC);
    HSD_JObjSetTranslate(jobj, &sp2c);
}

pub unsafe extern "C" fn grKongo_801D6668(arg0: *mut HSD_GObj) {
    let mut sp5c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp50 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp44 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp34 = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    let mut sp28 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp1c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let gp = (*arg0).user_data as *mut Ground;
    let mut step: f32 = 0.001f32;
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;

    if ((*gp).u.kongo.xCC + step) as f64 <= 1.0 {
        ext::splArcLengthPoint(&mut sp28, (*gp).u.kongo2.xC4, (*gp).u.kongo.xCC);
        ext::splArcLengthPoint(&mut sp1c, (*gp).u.kongo2.xC4, (*gp).u.kongo.xCC + step);
        let mut dx = sp28.x - sp1c.x;
        let mut dz = sp28.z;
        dz -= sp1c.z;
        let mut dy = sp28.y - sp1c.y;
        dx *= dx;
        dy *= dy;
        dz *= dz;
        let mut dist = dx + dy;
        dist = dz + dist;
        if dist > 0.0 {
            let d = dist as f64;
            let mut guess = frsqrte(d);
            // 0.5 * guess * (3.0 - (guess * guess * dist)): the subtract fuses the product (one f64 fma)
            guess = (0.5 * guess) * (-(guess * guess)).mul_add(d, 3.0);
            guess = (0.5 * guess) * (-(guess * guess)).mul_add(d, 3.0);
            guess = (0.5 * guess) * (-(guess * guess)).mul_add(d, 3.0);
            let length = (d * guess) as f32;
            dist = length;
        }
        if dist as f64 > 0.0 {
            step *= (*yakumono_param).unk14 / dist;
            (*gp).u.kongo.xCC += step;
            if (*gp).u.kongo.xCC as f64 > 1.0 {
                (*gp).u.kongo.xCC = 1.0;
            }
        }
    }
    let dx = (*gp).u.kongo.xCC;
    ext::Ground_801C4B50((*gp).u.kongo2.xC4, &mut sp5c, &mut sp50, dx);
    HSD_JObjGetTranslation(jobj, &mut sp44);
    HSD_JObjSetTranslate(jobj, &sp5c);
    sp34.x = sp50.x;
    sp34.y = sp50.y;
    sp34.z = sp50.z;
    sp34.w = 1.0;
    HSD_JObjSetRotation(jobj, &sp34);
    if (*gp).u.kongo.xD8 < sp5c.y {
        (*gp).u.kongo.xD8 = sp5c.y;
    }
    if (*gp).u.kongo2.xDC > sp5c.z {
        (*gp).u.kongo2.xDC = sp5c.z;
    }
    if ((*gp).u.kongo.xD8 - sp44.y) < 5.0 && ((*gp).u.kongo.xD8 - sp5c.y) > 5.0 {
        ext::Ground_801C5440(gp, 0, 0x5A550);
        ext::Ground_801C5630(gp, 0, 1.0 - (sp5c.z / (*gp).u.kongo2.xDC));
    }
}

pub unsafe extern "C" fn grKongo_801D69AC(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKongo_801D69B0(gobj: *mut HSD_GObj) {
    for i in 0..15usize {
        grKg_803E188C[i].unk4 = ext::Ground_801C3FA4(gobj, grKg_803E188C[i].unk0 as s32);
        grKg_803E188C[i].unkC = grKg_803E188C[i].unk8;
        let rot_x = grKg_803E188C[i].unkC;
        HSD_JObjSetRotationX(grKg_803E188C[i].unk4, rot_x);
    }
    grKg_804D6984.unk0 = ext::Ground_801C3FA4(gobj, 0xB);
    grKg_804D6984.unk4 = ext::Ground_801C3FA4(gobj, 0x21);
    grKongo_801D7134(gobj, 1);
    grKongo_801D77E0(gobj, 1);
    ext::mpJointSetB10(4);
}

/// `static inline void rad_compare_b(f32 a, f32 b, f32* ret)`.
#[inline]
fn rad_compare_b(a: f32, b: f32, ret: &mut f32) {
    let limit = deg2rad(b);
    let b = if a > limit {
        -limit
    } else if a < -limit {
        limit
    } else {
        0.0
    };
    *ret = (0.99f64 * (b + *ret) as f64) as f32;
}

pub unsafe extern "C" fn grKongo_801D6AFC() {
    let yp = yakumono_param;
    let values = [0.0f32; 15];
    for i in 0..15usize {
        let a = grKg_803E188C[i].unkC - grKg_803E188C[i].unk8;
        rad_compare_b(a, (*yp).unk94, &mut grKg_803E188C[i].unk10);
    }
    for i in 0..15usize {
        grKg_803E188C[i].unk10 += values[i];
    }
    for i in 0..15usize {
        let v = grKg_803E188C[i].unk10;
        Ground_ClampSymmetric(v, deg2rad((*yp).unk98), &mut grKg_803E188C[i].unk10);
    }
    for i in 0..15usize {
        grKg_803E188C[i].unkC += grKg_803E188C[i].unk10;
        let v = grKg_803E188C[i].unkC;
        Ground_ClampSymmetric(v, deg2rad((*yp).unk90), &mut grKg_803E188C[i].unkC);
    }
    let mut sp8 = [0.0f32; 15];
    for i in 0..15usize {
        if grKg_803E188C[i].unk2 == 0 {
            if i != 0 && grKg_803E188C[i - 1].unk2 == 0 {
                let input = grKg_803E188C[i - 1].unkC - grKg_803E188C[i].unkC;
                let limit = deg2rad((*yp).unk9C);
                if input > limit {
                    sp8[i] = fmadds((*yp).unkA0, input - limit, sp8[i]);
                } else if input < -limit {
                    sp8[i] = fmadds((*yp).unkA0, input + limit, sp8[i]);
                }
            }
            if i as u32 != 14 && grKg_803E188C[i + 1].unk2 == 0 {
                let input = grKg_803E188C[i + 1].unkC - grKg_803E188C[i].unkC;
                let limit = deg2rad((*yp).unk9C);
                if input > limit {
                    sp8[i] = fmadds((*yp).unkA0, input - limit, sp8[i]);
                } else if input < -limit {
                    sp8[i] = fmadds((*yp).unkA0, input + limit, sp8[i]);
                }
            }
        }
    }
    for i in 0..15usize {
        if sp8[i] != 0.0 {
            grKg_803E188C[i].unkC += sp8[i];
        }
    }
}

/// `static inline f32 grKongo_calc_angle(s32 index)`.
#[inline]
unsafe fn grKongo_calc_angle(index: usize) -> f32 {
    let t = &grKg_803E188C;
    let yp = yakumono_param;
    let mut angle = (0.7853981633974483f64
        * (0.5f64
            * (((t[index].unk14 - t[index - 1].unk14) / 6.0f32) + ((t[index + 1].unk14 - t[index].unk14) / 6.0f32)) as f64))
        as f32;
    if angle > deg2rad((*yp).unkAC) {
        angle = deg2rad((*yp).unkAC);
    } else if angle < -deg2rad((*yp).unkAC) {
        angle = -deg2rad((*yp).unkAC);
    }
    angle
}

pub unsafe extern "C" fn grKongo_801D7134(gobj: *mut HSD_GObj, _arg1: i32) {
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;
    grKongo_801D6AFC();

    for i in 0..15usize {
        grKg_803E188C[i].unk14 = (37.8f64 * tanf(-grKg_803E188C[i].unkC) as f64) as f32;
    }

    let mut displacement = grKg_803E188C[2].unkC * (deg2rad((*yp).unkA8) / deg2rad((*yp).unk90) - 1.0f32);
    let mut angle = grKongo_calc_angle(2);
    HSD_JObjSetRotationX(grKg_804D6984.unk0, displacement);
    let mut old_angle = HSD_JObjGetRotationZ(grKg_804D6984.unk0);
    HSD_JObjSetRotationZ(grKg_804D6984.unk0, angle);
    if ABS(angle - old_angle) > deg2rad((*yp).unkB0) {
        (*gp).u.kongo.xC8 = -(angle - old_angle);
    }

    displacement = grKg_803E188C[12].unkC * ((deg2rad((*yp).unkA8) / deg2rad((*yp).unk90)) - 1.0f32);
    angle = grKongo_calc_angle(12);
    HSD_JObjSetRotationX(grKg_804D6984.unk4, displacement);
    old_angle = HSD_JObjGetRotationZ(grKg_804D6984.unk4);
    HSD_JObjSetRotationZ(grKg_804D6984.unk4, angle);
    if ABS(angle - old_angle) > deg2rad((*yp).unkB0) {
        (*gp).u.kongo.xD8 = -(angle - old_angle);
    }

    for index in 0..15usize {
        HSD_JObjSetRotationX(grKg_803E188C[index].unk4, grKg_803E188C[index].unkC);
    }

    ext::mpLib_80057424(4);
    let mut line_id: i32 = 0x28;
    for i in 0..15usize {
        let temp = grKg_803E188C[i].unk14;
        let id = line_id;
        ext::mpLib_80056758(id, 0.0, temp, 0.0, temp);
        if i == 0 {
            ext::mpLib_80056758(id - 1, 0.0, temp, 0.0, temp);
        } else if i == 14 {
            ext::mpLib_80056758(id + 1, 0.0, temp, 0.0, temp);
        }
        line_id += 2;
    }
}

/// `static void fn_801D7700(...)`: `mpLib_JointCollisionCallback` of the bridge joint.
pub unsafe extern "C" fn fn_801D7700(
    _user_data: *mut c_void,
    _joint_id: i32,
    coll: *mut CollData,
    _coll_x50: i32,
    ground_kind: mpLib_GroundEnum,
    _delta_y: f32,
) {
    let cur = &(*coll).cur_pos;
    let ty = (*coll).x34_flags.b1234() as i32;
    let (mut x1, mut y1, mut x2, mut y2) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);

    if ty == 1 || ty == 3 {
        if ground_kind == mpLib_GroundEnum_Unk1 {
            ext::mpVtxGetPos(0x1D, &mut x1, &mut y1);
            ext::mpVtxGetPos(0x1A, &mut x2, &mut y2);

            let segment_size = (x2 - x1) / 15.0f32;
            let ratio = (cur.x - x1) / segment_size;
            let mut idx2 = ratio as i32;
            let idx = ratio as i32;

            if idx < 0 {
                idx2 = 0;
            } else if idx2 as u32 >= 15 {
                idx2 = 14;
            }

            let t = &mut grKg_803E188C[idx2 as usize];
            t.unk10 = fmadds(0.017453292f32, (*yakumono_param).unkA4, t.unk10);
        }
    }
}

pub unsafe extern "C" fn grKongo_801D77E0(gobj: *mut HSD_GObj, arg1: i32) {
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;

    if arg1 != 0 {
        (*gp).u.kongo.xC4 = 0.0;
        (*gp).u.kongo.xC8 = 0.0;
        (*gp).u.kongo.xD4 = 0.0;
        (*gp).u.kongo.xD8 = 0.0;
    } else {
        // two (angle, velocity) pairs: GC gp+C4/C8, then gp+D4/D8 (the C steps a Ground* by 0x10)
        let pairs: [(*mut f32, *mut f32); 2] = [
            (addr_of_mut!((*gp).u.kongo.xC4), addr_of_mut!((*gp).u.kongo.xC8)),
            (addr_of_mut!((*gp).u.kongo.xD4), addr_of_mut!((*gp).u.kongo.xD8)),
        ];
        for (a, v) in pairs {
            let step: f32;
            if *a > 0.0 {
                if *v > 0.0 {
                    step = 0.017453292f32 * (*yp).unkB4;
                } else {
                    step = (0.017453292f32 * (*yp).unkB4) / 2.0f32;
                }
                *v -= step;
            } else if *a < 0.0 {
                if *v < 0.0 {
                    step = 0.017453292f32 * (*yp).unkB4;
                } else {
                    step = (0.017453292f32 * (*yp).unkB4) / 2.0f32;
                }
                *v += step;
            }
            *a += *v;
            let limit = 0.017453292f32 * fmadds(-0.017453292f32, (*yp).unkAC, (*yp).unkB8);
            if *a > limit {
                *a = limit;
                *v = 0.0;
            } else if *a < -limit {
                *a = -limit;
                *v = 0.0;
            } else if ABS(*a) < 0.017453292f32 * (*yp).unkB4 {
                if ABS(*v) < 0.017453292f32 * (*yp).unkB4 {
                    *a = 0.0;
                    *v = 0.0;
                }
            }
        }
    }
    HSD_JObjSetRotationZ((*gp).u.kongo3.xCC, (*gp).u.kongo.xC4);
    HSD_JObjSetRotationZ((*gp).u.kongo3.xD0, (*gp).u.kongo.xC4);
    HSD_JObjSetRotationZ((*gp).u.kongo.xDC, (*gp).u.kongo.xD4);
    HSD_JObjSetRotationZ((*gp).u.kongo.xE0, (*gp).u.kongo.xD4);
}

pub unsafe extern "C" fn grKongo_801D7BBC(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;
    let mut gobj_id: i32 = 0;

    if (*gp).u.kongo.xE4 > 0 {
        (*gp).u.kongo.xE4 -= 1;
    }
    if (*gp).u.kongo.xE4 == -1 {
        return;
    }
    if (*gp).u.kongo.xE4 != 0 {
        return;
    }

    let mut event_kind: i32 = (ext::gm_8016B238() == 0 && ext::it_8026D324(It_Kind_Box) != 0) as i32;
    let mut roll: f32 = if event_kind != 0 { (*yp).unkC } else { 0.0 };
    {
        let rand = ext::HSD_Randf();
        roll = rand
            * fmadds(
                (*yp).unk10,
                1.0f32 - (*yp).unk18,
                fmadds((*yp).unk10, (*yp).unk18, (*yp).unk8 + roll),
            );
    }
    {
        let r = ext::HSD_Randf();
        let delta = (*yp).unk4 - (*yp).unk0;
        (*gp).u.kongo.xE4 = f2s16(fmadds(delta, r, (*yp).unk0));
    }
    roll -= (*yp).unk8;
    if roll < 0.0 {
        event_kind = 1;
    } else {
        let mut flag = 0;
        if ext::gm_8016B238() == 0 && ext::it_8026D324(It_Kind_Box) != 0 {
            flag = 1;
        }
        let box_weight = if flag != 0 { (*yp).unkC } else { 0.0 };
        roll -= box_weight;
        let zero = 0.0f32;
        if roll < zero {
            event_kind = 2;
        } else {
            let base_weight = (*yp).unk10;
            let box_rate = (*yp).unk18;
            let prod = base_weight * box_rate;
            roll -= prod;
            if roll < zero {
                gobj_id = if ext::HSD_Randi(2) != 0 { 1 } else { 2 };
                event_kind = 3;
            } else {
                roll = fmadds(-base_weight, 1.0f32 - box_rate, roll);
                if roll < zero {
                    let rand_kind = ext::HSD_Randi(3);
                    gobj_id = if rand_kind == 0 {
                        7
                    } else if rand_kind == 1 {
                        8
                    } else {
                        9
                    };
                    event_kind = 3;
                } else {
                    return;
                }
            }
        }
    }

    if (*gp).u.kongo.xE6 as i32 == event_kind && ext::HSD_Randf() > (*yp).unk1C {
        return;
    }
    (*gp).u.kongo.xE6 = event_kind as i16;
    match event_kind {
        1 => {
            let other_gobj = grKongo_801D5340(11);
            if !other_gobj.is_null() && ext::it_802E18B4(other_gobj).is_null() {
                ext::Ground_801C4A08(other_gobj);
            }
        }
        2 => {
            let other_gobj = grKongo_801D5340(11);
            if !other_gobj.is_null() && ext::it_80286088(other_gobj).is_null() {
                ext::Ground_801C4A08(other_gobj);
            }
        }
        3 => {
            grKongo_801D5340(gobj_id);
        }
        _ => {}
    }
}

/// `static void fn_801D7E60(...)`: `mpLib_JointCollisionCallback` of the platform joint.
pub unsafe extern "C" fn fn_801D7E60(
    user_data: *mut c_void,
    _joint_id: i32,
    _coll: *mut CollData,
    _coll_x50: i32,
    ground_kind: mpLib_GroundEnum,
    _delta_y: f32,
) {
    let gp = user_data as *mut Ground;
    if ground_kind == mpLib_GroundEnum_Unk2 {
        (*gp).u.kongo3.xC6 = ((*gp).u.kongo3.xC8 as i32 + 1) as i16;
    }
}

pub unsafe fn grKongo_801D7E78(gobj: *mut HSD_GObj, pos: *mut Vec3) -> *mut Vec3 {
    let gp = (*gobj).user_data as *mut Ground;
    if !gp.is_null() {
        let m = (*gp).map_id;
        if m == Gr_Kind_Test || m == Gr_Kind_Castle || m == Gr_Kind_Shrine || m == Gr_Kind_Zebes || m == Gr_Kind_Kraid {
            let jobj = ext::Ground_801C3FA4(gobj, 2);
            if !jobj.is_null() {
                ext::lb_8000B1CC(jobj, null_mut(), pos);
            } else {
                return null_mut();
            }
        } else if m == Gr_Kind_Yorster {
            let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
            if !jobj.is_null() {
                HSD_JObjGetTranslation(jobj, pos);
            } else {
                return null_mut();
            }
        } else {
            return null_mut();
        }
    } else {
        return null_mut();
    }
    pos
}

pub unsafe fn grKongo_801D7F78(gobj: *mut HSD_GObj) -> bool {
    let mut pos1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut pos2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if grKongo_801D7E78(gobj, &mut pos1).is_null() {
        return false;
    }

    let mut cur = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_GROUND as usize);
    while !cur.is_null() {
        if cur != gobj && !grKongo_801D7E78(cur, &mut pos2).is_null() {
            let dz = pos2.z - pos1.z;
            if dz < -100.0f32 && dz > -150.0f32 {
                let dx = pos2.x - pos1.x;
                if dx * dx < 6400.0f32 {
                    return true;
                }
            }
        }
        cur = (*cur).next;
    }

    false
}

pub unsafe extern "C" fn grKongo_801D8058(arg: *mut HSD_GObj) {
    ext::Ground_801C4A08(arg);
}

pub unsafe fn grKongo_801D8078(_gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut unk: f32 = 0.0;
    let mut item_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::Ground_801C4DA0(&mut pos, &mut unk);

    let mut cur = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !cur.is_null() {
        if ext::itGetKind(cur) == It_Kind_Klap {
            ext::it_8026B294(cur, &mut item_pos);

            let dx = pos.x - item_pos.x;
            let dz = pos.z - item_pos.z;
            let dy = pos.y - item_pos.y;
            let dx2 = dx * dx;
            let dy2 = dy * dy;
            let dz2 = dz * dz;
            let r = (*yakumono_param).unk28;

            if dx2 + dy2 + dz2 < r * r {
                return cur;
            }
        }
        cur = (*cur).next;
    }

    null_mut()
}

/// `static int fn_801D8134(HSD_GObj*, HSD_GObj*)`: the ftCo_800C0764 grab-range callback.
pub unsafe extern "C" fn fn_801D8134(arg0: *mut HSD_GObj, arg1: *mut HSD_GObj) -> i32 {
    let gp = GET_GROUND(arg0);
    let mut pos_gnd = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut pos_ft = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut unk: f32 = 0.0;
    let yp = yakumono_param;

    if (*gp).u.kongo3.xC6 == 0 {
        ext::Ground_801C4DA0(&mut pos_gnd, &mut unk);
        ext::ftLib_80086644(arg1, &mut pos_ft);

        let a = pos_gnd.x - pos_ft.x;
        let b = pos_gnd.y - pos_ft.y;
        let c = pos_gnd.z - pos_ft.z;
        // a*a + b*b + c*c: LHS multiply fused with b*b, then c*c fused onto the sum
        let dist2 = fmadds(c, c, fmadds(a, a, b * b));
        if dist2 < (*yp).unk28 * (*yp).unk28 {
            let rand_val = ext::HSD_Randf();
            let diff = (*yp).unk24 - (*yp).unk20;
            (*gp).u.kongo3.xCA = f2s16(fmadds(diff, rand_val, (*yp).unk20));
            (*gp).u.kongo3.xD0 = arg1 as *mut HSD_JObj;
            (*gp).u.kongo3.xC6 = 1;
            ext::Ground_801C5440(gp, 0, 0x129);
            ext::grMaterial_801C9604(arg0, (*yp).unk84, 0);
            ext::efSync_Spawn(0x405, arg0, &mut pos_ft);
            ext::ftLib_80086C18(arg1, 0xD, 0x1E);
            return 1;
        }
    }
    0
}

pub unsafe extern "C" fn grKongo_801D8270(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    if (*gp).u.inishie2.xC6 == 1 {
        (*gp).u.inishie2.xC6 = 2;
    }
}

pub unsafe extern "C" fn grKongo_801D828C(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    if (*gp).u.kongo3.xC6 != 1 {
        return;
    }
    assert_(!(*gp).u.kongo.u.taru.keep.is_null(), 1719, "gp->u.taru.keep");
    if (*((*gp).u.kongo.u.taru.keep as *mut HSD_GObj)).p_link as i32 == HSD_GOBJ_PLINK_FIGHTER {
        (*gp).u.kongo3.xC6 = 0;
        (*gp).u.kongo.u.taru.keep = null_mut();
        ext::grMaterial_801C95C4(gobj);
    }
}

pub unsafe fn grKongo_801D8314() -> f32 {
    let yp = yakumono_param;
    let temp_r3: i32 = (*yp).unk44 as i32
        + (*yp).unk46 as i32
        + (*yp).unk48 as i32
        + (*yp).unk4A as i32
        + (*yp).unk4C as i32
        + (*yp).unk4E as i32
        + (*yp).unk50 as i32
        + (*yp).unk52 as i32;
    let var_r3 = if temp_r3 != 0 { ext::HSD_Randi(temp_r3) } else { 0 };
    let temp_r3_2 = var_r3 - (*yp).unk44 as i32;
    if temp_r3_2 < 0 {
        return 2.3561945f32;
    }
    let temp_r3_3 = temp_r3_2 - (*yp).unk46 as i32;
    if temp_r3_3 < 0 {
        return 1.5707964f32;
    }
    let temp_r3_4 = temp_r3_3 - (*yp).unk48 as i32;
    if temp_r3_4 < 0 {
        return 0.7853982f32;
    }
    let temp_r3_5 = temp_r3_4 - (*yp).unk4A as i32;
    if temp_r3_5 < 0 {
        return 0.0f32;
    }
    let temp_r3_6 = temp_r3_5 - (*yp).unk4C as i32;
    if temp_r3_6 < 0 {
        return -0.7853982f32;
    }
    let temp_r3_7 = temp_r3_6 - (*yp).unk4E as i32;
    if temp_r3_7 < 0 {
        return -1.5707964f32;
    }
    let temp_r3_8 = temp_r3_7 - (*yp).unk50 as i32;
    if temp_r3_8 < 0 {
        return -2.3561945f32;
    }
    let temp_r3_8 = temp_r3_8 - (*yp).unk52 as i32;
    if temp_r3_8 < 0 {
        return -3.1415927f32;
    }
    assert_(false, 1753, "0");
    0.0
}

pub unsafe extern "C" fn grKongo_801D844C(a: *mut Vec3, _b: i32, jobj: *mut HSD_JObj) -> i32 {
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, null_mut(), &mut vec);
    if (*a).y > vec.y {
        1
    } else {
        0
    }
}
