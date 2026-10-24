//! src/melee/gr/grkinokoroute.c: Mushroom Kingdom (Adventure) "Mushroom Kingdom route" stage
//! (Gr_Kind_KinokoRoute). State is the `yakumono_param` static, the per-gobj `Ground`
//! (`u.kinokoroute` / `u.kinokoroute2`), `stage_info` and the callback / stage-data tables; every
//! other callee is a hook in [`ext`] (a TODO stub, or the native port of the TU that owns it:
//! `grlib`, `grmaterial`).
//!
//! Float code: `grKinokoRoute_80207C88` has two fused sites (`SQ(x)+SQ(y)+SQ(z)`, the camera
//! distance) and calls MSL `sqrtf` (Newton-Raphson on `frsqrte`: `melee_runtime::gekko::sqrtf`).
//! `grKinokoRoute_802084B4` casts the item's user_data to a struct with a pointer at GC +0xDD8;
//! that is `Item.xDD4_itemVar.yaku.x4` (the item is a `yaku` item), and the map gobj's `Ground+0xC4`
//! flag byte is `u.kinokoroute2.flags_0`.
//! State-oracle checked against the C (tests/gro_grkinokoroute.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use super::grinl::{assert_, GET_GROUND, HSD_JObjGetTranslation, HSD_JObjSetTranslate, HSD_JObjSetTranslateX, HSD_JObjSetTranslateY};
use super::grinl2::Ground_UpdateWindAndMapColl;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::gekko::sqrtf;
use melee_core::fp::fmadds;
use melee_types::*;

pub use crate::gr::trivial::grkinokoroute::{
    grKinokoRoute_80207544, grKinokoRoute_802078E8, grKinokoRoute_80207ACC, grKinokoRoute_80207B20,
    grKinokoRoute_80207C80, grKinokoRoute_80208754,
};

/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::{stage_info, HSD_JObjSetMtxDirtySub};
    pub use crate::tu::grinl2::ext::{
        grAnime_801C8138, grDisplay_801C5DB0, GObj_SetupGXLink, Ground_UpdateMapColl, HSD_GObj_SetupProc, HSD_Randi,
        lb_800115F4,
    };
    pub use crate::tu::grinl2::ext::Ground_InitMapColl;
    hook!(CAMERA_8002F3AC, Camera_8002F3AC, ());
    hook_or!(CAMERA_80030AE0, Camera_80030AE0, (a0: i32), |a0: i32| crate::cm::camera::world().camera_80030ae0(a0));
    hook_or!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, (a0: CmQuakeKind, a1: *mut Vec3), |a0: CmQuakeKind, a1: *mut Vec3| unsafe { crate::cm::camera::Camera_RequestQuake(a0 as _, a1 as _) });
    pub static mut GROUND_801C0498: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C0498() -> f32 {
        match unsafe { GROUND_801C0498 } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_801C0498() as _ }; } }
    }
    pub static mut GROUND_801C10B8: Option<fn(*mut HSD_GObj, HSD_GObjEvent)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C10B8(a0: *mut HSD_GObj, a1: HSD_GObjEvent) {
        match unsafe { GROUND_801C10B8 } { Some(f) => f(a0, a1), None => { unsafe { let _ = crate::tu::ground::Ground_801C10B8(a0 as _, a1 as _); } return; } }
    }
    pub static mut GROUND_801C2CF4: Option<fn(s32) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C2CF4(a0: s32) -> *mut HSD_JObj {
        match unsafe { GROUND_801C2CF4 } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_801C2CF4(a0 as _) as _ }; } }
    }
    pub static mut GROUND_801C2D24: Option<fn(enum_t, *mut Vec3) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C2D24(a0: enum_t, a1: *mut Vec3) -> i32 {
        match unsafe { GROUND_801C2D24 } { Some(f) => f(a0, a1), None => { return unsafe { crate::tu::ground::Ground_801C2D24(a0 as _, a1 as _) as _ }; } }
    }
    pub static mut GROUND_801C3880: Option<fn(f32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3880(a0: f32) {
        match unsafe { GROUND_801C3880 } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C3880(a0 as _); } return; } }
    }
    pub static mut GROUND_801C3890: Option<fn(f32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3890(a0: f32) {
        match unsafe { GROUND_801C3890 } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C3890(a0 as _); } return; } }
    }
    pub static mut GROUND_801C38A0: Option<fn(f32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C38A0(a0: f32) {
        match unsafe { GROUND_801C38A0 } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C38A0(a0 as _); } return; } }
    }
    pub static mut GROUND_801C38AC: Option<fn(f32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C38AC(a0: f32) {
        match unsafe { GROUND_801C38AC } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C38AC(a0 as _); } return; } }
    }
    pub static mut GROUND_801C38BC: Option<fn(f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C38BC(x: f32, y: f32) {
        match unsafe { GROUND_801C38BC } { Some(f) => f(x, y), None => { unsafe { let _ = crate::tu::ground::Ground_801C38BC(x as _, y as _); } return; } }
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
    pub static mut GROUND_801C3D44: Option<fn(*mut c_void, f32, f32) -> s32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3D44(a0: *mut c_void, a1: f32, a2: f32) -> s32 {
        match unsafe { GROUND_801C3D44 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::ground::Ground_801C3D44(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GROUND_801C3DB4: Option<fn(*mut c_void, f32, f32) -> s32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3DB4(a0: *mut c_void, a1: f32, a2: f32) -> s32 {
        match unsafe { GROUND_801C3DB4 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::ground::Ground_801C3DB4(a0 as _, a1 as _, a2 as _) as _ }; } }
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
    pub static mut GROUND_801C53EC: Option<fn(u32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C53EC(a0: u32) {
        match unsafe { GROUND_801C53EC } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C53EC(a0 as _); } return; } }
    }
    pub static mut GROUND_801C5414: Option<fn(i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5414(a0: i32, a1: i32) {
        match unsafe { GROUND_801C5414 } { Some(f) => f(a0, a1), None => { unsafe { let _ = crate::tu::ground::Ground_801C5414(a0 as _, a1 as _); } return; } }
    }
    pub static mut GROUND_801C5740: Option<fn(s32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5740(a0: s32) {
        match unsafe { GROUND_801C5740 } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C5740(a0 as _); } return; } }
    }
    pub static mut GROUND_801C5750: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5750() {
        match unsafe { GROUND_801C5750 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::ground::Ground_801C5750(); } return; } }
    }
    pub static mut GROUND_801C5764: Option<fn() -> s32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5764() -> s32 {
        match unsafe { GROUND_801C5764 } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_801C5764() as _ }; } }
    }
    pub static mut GROUND_801C5840: Option<fn() -> s32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5840() -> s32 {
        match unsafe { GROUND_801C5840 } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_801C5840() as _ }; } }
    }
    pub static mut GROUND_801C5940: Option<fn() -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5940() -> i32 {
        match unsafe { GROUND_801C5940 } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_801C5940() as _ }; } }
    }
    pub static mut GROUND_GETMAPGOBJ: Option<fn(i32) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetMapGObj(map_id: i32) -> *mut HSD_GObj {
        match unsafe { GROUND_GETMAPGOBJ } { Some(f) => f(map_id), None => { return unsafe { crate::tu::ground::Ground_GetMapGObj(map_id as _) as _ }; } }
    }
    pub static mut GROUND_GETP1FIGHTER: Option<fn() -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetP1Fighter() -> *mut HSD_GObj {
        match unsafe { GROUND_GETP1FIGHTER } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_GetP1Fighter() as _ }; } }
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
    hook_or!(HSD_JOBJCLEARFLAGSALL, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, flags as _); } });
    hook_or!(HSD_JOBJSETFLAGSALL, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, flags as _); } });
    hook_or!(OSREPORT, OSReport, (a0: *mut i8), |a0: *mut i8| unsafe { melee_runtime::tu::os_error::OSReport(a0 as _, &[]) });
    pub static mut STAGE_GETCAMBOUNDSBOTTOMOFFSET: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Stage_GetCamBoundsBottomOffset() -> f32 {
        match unsafe { STAGE_GETCAMBOUNDSBOTTOMOFFSET } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_GetCamBoundsBottomOffset() as _ }; } }
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
    pub static mut STAGE_GETCAMBOUNDSTOPOFFSET: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Stage_GetCamBoundsTopOffset() -> f32 {
        match unsafe { STAGE_GETCAMBOUNDSTOPOFFSET } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_GetCamBoundsTopOffset() as _ }; } }
    }
    pub static mut STAGE_UNKSETVEC3TCAM_OFFSET: Option<fn(*mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn Stage_UnkSetVec3TCam_Offset(a0: *mut Vec3) {
        match unsafe { STAGE_UNKSETVEC3TCAM_OFFSET } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::stage::Stage_UnkSetVec3TCam_Offset(a0 as _); } return; } }
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
    hook_or!(FTCO_800C07F8, ftCo_800C07F8, (a0: *mut HSD_GObj, a1: u32, a2: *mut c_void), |a0: *mut HSD_GObj, a1: u32, a2: *mut c_void| { unsafe { crate::ftdevice::ftCo_800C07F8(a0 as _, a1 as _, core::mem::transmute(a2)); } });
    hook_or!(FTLIB_80086644, ftLib_80086644, (a0: *mut HSD_GObj, a1: *mut Vec3), |a0: *mut HSD_GObj, a1: *mut Vec3| { unsafe { let _ = crate::ftlib::ftLib_80086644(a0 as _, a1 as _); } });
    hook_or!(FTLIB_80086684, ftLib_80086684, (a0: *mut HSD_GObj, a1: *mut Vec3), |a0: *mut HSD_GObj, a1: *mut Vec3| { unsafe { let _ = crate::ftlib::ftLib_80086684(a0 as _, a1 as _); } });
    hook_or!(FTLIB_80086B80, ftLib_80086B80, (a0: *mut HSD_GObj) -> f32, |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086B80(a0 as _) as _ });
    hook_or!(FTLIB_80086EC0, ftLib_80086EC0, (a0: *mut HSD_GObj) -> i32, |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086EC0(a0 as _) as _ });
    hook_or!(FTLIB_8008732C, ftLib_8008732C, (a0: *mut HSD_GObj) -> i32, |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_8008732C(a0 as _) as _ });
    hook!(GM_801674C4, gm_801674C4, (a0: s8, a1: u8, a2: s8, a3: s8, a4: GmRouteCallback));
    hook_or!(GRLIB_801C96F8, grLib_801C96F8, (a0: s32, a1: s32, a2: *mut Vec3) -> *mut HSD_Generator, |a0: s32, a1: s32, a2: *mut Vec3| unsafe { crate::tu::grlib::grLib_801C96F8(a0, a1, a2) });
    hook_or!(GRMATERIAL_801C8CDC, grMaterial_801C8CDC, (a0: *mut HSD_GObj), |a0: *mut HSD_GObj| unsafe { crate::tu::grmaterial::grMaterial_801C8CDC(a0) });
    hook_or!(GRMATERIAL_801C8CFC, grMaterial_801C8CFC, (a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, arg4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, arg5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32)>, arg6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj, |a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, arg4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, arg5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32)>, arg6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>| unsafe { crate::tu::grmaterial::grMaterial_801C8CFC(a0, a1, a2, a3, arg4, arg5, arg6) });
    hook_or!(GRMATERIAL_801C8DE0, grMaterial_801C8DE0, (gobj: *mut HSD_GObj, arg8: f32, arg9: f32, arg_a: f32, arg_b: f32, arg_c: f32, arg_d: f32, arg_e: f32), |gobj: *mut HSD_GObj, arg8: f32, arg9: f32, arg_a: f32, arg_b: f32, arg_c: f32, arg_d: f32, arg_e: f32| unsafe { crate::tu::grmaterial::grMaterial_801C8DE0(gobj, arg8, arg9, arg_a, arg_b, arg_c, arg_d, arg_e) });
    hook_or!(GRMATERIAL_801C8E08, grMaterial_801C8E08, (a0: *mut HSD_GObj), |a0: *mut HSD_GObj| unsafe { crate::tu::grmaterial::grMaterial_801C8E08(a0) });
    hook_or!(GRMATERIAL_801C8E28, grMaterial_801C8E28, (a0: *mut HSD_GObj), |a0: *mut HSD_GObj| unsafe { crate::tu::grmaterial::grMaterial_801C8E28(a0) });
    hook_or!(GRMATERIAL_801C8E68, grMaterial_801C8E68, (a0: *mut HSD_GObj, a1: GroundOrAir), |a0: *mut HSD_GObj, a1: GroundOrAir| unsafe { crate::tu::grmaterial::grMaterial_801C8E68(a0, a1) });
    pub static mut GRZAKOGENERATOR_801CADE0: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CADE0() {
        match unsafe { GRZAKOGENERATOR_801CADE0 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::grzakogenerator::grZakoGenerator_801CADE0(); } return; } }
    }
    pub static mut GRZAKOGENERATOR_801CAE04: Option<fn(*mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj {
        match unsafe { GRZAKOGENERATOR_801CAE04 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ }; } }
    }
    pub static mut GRZAKOGENERATOR_801CAEB0: Option<fn(i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAEB0(a0: i32, a1: i32) {
        match unsafe { GRZAKOGENERATOR_801CAEB0 } { Some(f) => f(a0, a1), None => { unsafe { let _ = crate::tu::grzakogenerator::grZakoGenerator_801CAEB0(a0 as _, a1 as _); } return; } }
    }
    pub static mut GRZAKOGENERATOR_801CAEF0: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAEF0(a0: i32) {
        match unsafe { GRZAKOGENERATOR_801CAEF0 } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::grzakogenerator::grZakoGenerator_801CAEF0(a0 as _); } return; } }
    }
    pub static mut GRZAKOGENERATOR_801CAF08: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAF08() {
        match unsafe { GRZAKOGENERATOR_801CAF08 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::grzakogenerator::grZakoGenerator_801CAF08(); } return; } }
    }
    hook_or!(ITGETKIND, itGetKind, (gobj: *mut HSD_GObj) -> ItemKind, |gobj: *mut HSD_GObj| unsafe { crate::itacc::itGetKind(gobj as _) as _ });
    pub static mut LBVECTOR_ADD: Option<fn(*mut Vec3, *mut Vec3) -> *mut Vec3> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Add(a: *mut Vec3, b: *mut Vec3) -> *mut Vec3 {
        match unsafe { LBVECTOR_ADD } { Some(f) => f(a, b), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Add(a as _, b as _) as _ }; } }
    }
    pub static mut LBVECTOR_DIFF: Option<fn(*mut Vec3, *mut Vec3, *mut Vec3) -> *mut Vec3> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Diff(a: *mut Vec3, b: *mut Vec3, result: *mut Vec3) -> *mut Vec3 {
        match unsafe { LBVECTOR_DIFF } { Some(f) => f(a, b, result), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Diff(a as _, b as _, result as _) as _ }; } }
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
    pub static mut MPLIB_80058560: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_80058560() {
        match unsafe { MPLIB_80058560 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::mplib::mpLib_80058560(); } return; } }
    }
    /// `HSD_GObj** HSD_GObjPLinkHead` (the array of per-p_link list heads).
    pub static mut HSD_GObjPLinkHead: *mut *mut HSD_GObj = core::ptr::null_mut();
}

/// The file-local anonymous `static struct { int x0; grZakoGenerator_SpawnDesc x4; }* yakumono_param`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grNKr_YakumonoParam {
    pub x0: i32,
    pub x4: grZakoGenerator_SpawnDesc,
}

/// `typedef struct grNKr_Depths { s16 x[51]; }`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grNKr_Depths {
    pub x: [i16; 51],
}

/// `static struct grNKr_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grNKr_YakumonoParam = null_mut();

/// `static const Vec3 grNKr_803B82E8`.
pub static grNKr_803B82E8: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// `static const grNKr_Depths grNKr_803B82F4`.
pub static grNKr_803B82F4: grNKr_Depths = {
    let mut x = [0i16; 51];
    let mut i = 0;
    while i < 51 {
        x[i] = i as i16 + 1;
        i += 1;
    }
    grNKr_Depths { x }
};

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

/// `StageCallbacks grNKr_StageCallbacks[4]`.
pub static mut grNKr_StageCallbacks: [StageCallbacks; 4] = [
    cbs!(grKinokoRoute_80207634, grKinokoRoute_802078E8, grKinokoRoute_802078F0, grKinokoRoute_80207A94, 0),
    cbs!(grKinokoRoute_80207ADC, grKinokoRoute_80207B20, grKinokoRoute_80207B28, grKinokoRoute_80207B2C, 0),
    cbs!(grKinokoRoute_80207A98, grKinokoRoute_80207ACC, grKinokoRoute_80207AD4, grKinokoRoute_80207AD8, 0),
    cbs!(grKinokoRoute_80207B5C, grKinokoRoute_80207C80, grKinokoRoute_80207C88, grKinokoRoute_80208368, (1 << 30) | (1 << 31)),
];

/// `StageData grNKr_StageData`.
pub static mut grNKr_StageData: StageData = StageData {
    grkind: Gr_Kind_KinokoRoute,
    callbacks: unsafe { core::ptr::addr_of!(grNKr_StageCallbacks) as *mut StageCallbacks },
    data1: b"/GrNKr.dat\0".as_ptr() as *mut i8,
    on_init: Some(grKinokoRoute_80207420),
    on_demo_init: Some(grKinokoRoute_8020741C),
    on_load: Some(grKinokoRoute_80207490),
    on_start: Some(grKinokoRoute_802074D8),
    callback4: Some(grKinokoRoute_80207544),
    on_touch_line: Some(grKinokoRoute_80208754),
    on_check_shadow_render: Some(grKinokoRoute_8020875C),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

pub unsafe extern "C" fn grKinokoRoute_8020741C(_arg: i32) {}

pub unsafe extern "C" fn grKinokoRoute_80207420() {
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grNKr_YakumonoParam;
    let si = ext::stage_info();
    (*si).unk8C.set_b4(0);
    (*si).unk8C.set_b5(1);
    grKinokoRoute_8020754C(2);
    grKinokoRoute_8020754C(0);
    grKinokoRoute_8020754C(1);
    grKinokoRoute_8020754C(3);
}

pub unsafe extern "C" fn grKinokoRoute_80207490() {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gobj = ext::Ground_GetP1Fighter();
    if !gobj.is_null() {
        ext::ftLib_80086644(gobj, &mut pos);
        ext::Ground_801C38BC(pos.x, 20.0_f32 + pos.y);
        ext::Camera_8002F3AC();
    }
}

pub unsafe extern "C" fn grKinokoRoute_802074D8() {
    ext::grZakoGenerator_801CAE04(&mut (*yakumono_param).x4);
    let mut val = (*yakumono_param).x0;
    if val != 0 {
        val = ext::HSD_Randi((*yakumono_param).x0);
    } else {
        val = 0;
    }
    if val == 0 {
        ext::grZakoGenerator_801CAEB0(ext::Ground_801C5840(), ext::Ground_801C5940());
    }
    ext::grZakoGenerator_801CADE0();
}

pub unsafe fn grKinokoRoute_8020754C(gobj_id: i32) -> *mut HSD_GObj {
    let callbacks: *mut StageCallbacks = &mut grNKr_StageCallbacks[gobj_id as usize];
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

pub unsafe extern "C" fn grKinokoRoute_80207634(gobj: *mut HSD_GObj) {
    let mut origin = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = (*gobj).user_data as *mut Ground;
    let mut cam_offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let jobj = ext::Ground_801C2CF4(0x94);
    if !jobj.is_null() {
        HSD_JObjGetTranslation(jobj, &mut origin);

        for (i, id) in [0x7F, 0x80, 0x81, 0x82].into_iter().enumerate() {
            (*gp).u.kinokoroute.entries[i].jobj = ext::Ground_801C2CF4(id);
            if !(*gp).u.kinokoroute.entries[i].jobj.is_null() {
                HSD_JObjGetTranslation((*gp).u.kinokoroute.entries[i].jobj, &mut (*gp).u.kinokoroute.entries[i].pos);
                ext::lbVector_Sub(&mut (*gp).u.kinokoroute.entries[i].pos, &mut origin);
            }
        }
    } else {
        (*gp).u.kinokoroute.entries[0].jobj = null_mut();
        (*gp).u.kinokoroute.entries[1].jobj = null_mut();
        (*gp).u.kinokoroute.entries[2].jobj = null_mut();
        (*gp).u.kinokoroute.entries[3].jobj = null_mut();
    }

    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Stage_UnkSetVec3TCam_Offset(&mut cam_offset);
    ext::Ground_801C3880(1.5_f32 * (ext::Stage_GetCamBoundsTopOffset() - cam_offset.y));
    ext::Ground_801C3890(1.5_f32 * (ext::Stage_GetCamBoundsBottomOffset() - cam_offset.y));
    ext::Ground_801C38A0(1.5_f32 * (ext::Stage_GetCamBoundsLeftOffset() - cam_offset.x));
    ext::Ground_801C38AC(1.5_f32 * (ext::Stage_GetCamBoundsRightOffset() - cam_offset.x));
}

pub unsafe extern "C" fn grKinokoRoute_802078F0(gobj: *mut HSD_GObj) {
    // `cursor` walks the Entry array through a Ground*/Entry* union; entry i is entries[i].
    let gp = (*gobj).user_data as *mut Ground;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let fighter = ext::Ground_GetP1Fighter();
    if !fighter.is_null() {
        ext::ftLib_80086644(fighter, &mut pos);
    } else {
        return;
    }

    let mut scale = ext::Ground_801C0498();
    if scale > 0.0001_f32 {
        scale = 1.0_f32 / scale;
    } else {
        scale = 1.0_f32;
    }

    for i in 0..4usize {
        let e = &mut (*gp).u.kinokoroute.entries[i];
        if !e.jobj.is_null() {
            HSD_JObjSetTranslateX(e.jobj, scale * (e.pos.x + pos.x));
            HSD_JObjSetTranslateY(e.jobj, scale * (e.pos.y + pos.y));
        }
    }
}

pub unsafe extern "C" fn grKinokoRoute_80207A94(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKinokoRoute_80207A98(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

pub unsafe extern "C" fn grKinokoRoute_80207AD4(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKinokoRoute_80207AD8(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKinokoRoute_80207ADC(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grKinokoRoute_80207B28(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKinokoRoute_80207B2C(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grKinokoRoute_80207B30(gobj: *mut HSD_GObj) {
    ext::ftCo_800C07F8(gobj, 6, grKinokoRoute_80208660 as *mut c_void);
}

pub unsafe extern "C" fn grKinokoRoute_80207B5C(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);

    ext::Ground_InitMapColl((*gobj).hsd_obj as *mut HSD_JObj, (*gp).map_id as s32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    (*gp).x10_flags.set_b5(1);
    (*gp).u.kinokoroute2.phase = 0;
    (*gp).u.kinokoroute2.zone_idx = 0;
    (*gp).u.kinokoroute2.cam_timer = 0;
    (*gp).u.kinokoroute2.spawn_idx = -1;
    ext::Ground_801C10B8(gobj, Some(grKinokoRoute_80207B30));
    grKinokoRoute_80208564(gobj);
    (*gp).u.kinokoroute2.set_flags_0(1);
    let reb0_jobj = ext::Ground_801C2CF4(4);
    assert_(!reb0_jobj.is_null(), 467, "reb0_jobj");
    HSD_JObjGetTranslation(reb0_jobj, &mut (*gp).u.kinokoroute2.reb0_pos);
}

pub unsafe extern "C" fn grKinokoRoute_80207C88(gobj: *mut HSD_GObj) {
    let mut fighter_pos: Vec3;
    let mut zone_point = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut cam_target = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut cam_offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = GET_GROUND(gobj);

    fighter_pos = grNKr_803B82E8;
    let scale = ext::Ground_801C0498();
    let fighter = ext::Ground_GetP1Fighter();
    if !fighter.is_null() {
        ext::ftLib_80086644(fighter, &mut fighter_pos);
    } else {
        return;
    }
    if (*gp).u.kinokoroute2.phase != 2 {
        if ext::ftLib_80086EC0(fighter) != 0 {
            ext::Stage_UnkSetVec3TCam_Offset(&mut cam_target);
            (*gp).u.kinokoroute2.cam_timer = 0x3C;
        } else if ext::ftLib_8008732C(fighter) != 0 {
            ext::Stage_UnkSetVec3TCam_Offset(&mut cam_target);
            (*gp).u.kinokoroute2.cam_timer = 0;
        } else {
            if fighter_pos.y < 20.0_f32 * scale {
                fighter_pos.y = 20.0_f32 * scale;
            } else if fighter_pos.y > 250.0_f32 * scale {
                fighter_pos.y = 250.0_f32 * scale;
            }
            if (*gp).u.kinokoroute2.spawn_idx != -1 {
                let mut spawn_point = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                ext::Ground_801C2D24((*gp).u.kinokoroute2.spawn_idx as enum_t, &mut spawn_point);
                if fighter_pos.x < spawn_point.x {
                    fighter_pos.x = spawn_point.x;
                }
            } else {
                let mut boundary_point = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                ext::Ground_801C2D24(0, &mut boundary_point);
                if fighter_pos.x < boundary_point.x {
                    fighter_pos.x = boundary_point.x;
                }
                ext::Ground_801C2D24(0xBD, &mut boundary_point);
                if fighter_pos.x > boundary_point.x {
                    fighter_pos.x = boundary_point.x;
                }
            }
            cam_target = fighter_pos;
            ext::Camera_80030AE0(1);
        }
        ext::Ground_801C3D44(null_mut(), 30.0_f32, 10000.0_f32);
        if (*gp).u.kinokoroute2.phase < 2 {
            let spawn_idx = ext::Ground_801C3DB4(null_mut(), 60.0_f32, 10000.0_f32) as i16;
            (*gp).u.kinokoroute2.spawn_idx = spawn_idx;
            if spawn_idx != -1 {
                (*gp).u.kinokoroute2.phase = 2;
                (*gp).u.kinokoroute2.cam_timer = 0x3C;
                ext::Ground_801C5750();
                {
                    let route_id: u8 = 0xB3;
                    ext::gm_801674C4(0x11, 0xA, 3, route_id as i8, Some(grKinokoRoute_80208480));
                }
                (*gp).u.kinokoroute2.zone_idx = 2;
                let mut ground_gobj = ext::Ground_GetMapGObj(0);
                if !ground_gobj.is_null() {
                    ext::Ground_801C4A08(ground_gobj);
                }
                ground_gobj = ext::Ground_GetMapGObj(2);
                if !ground_gobj.is_null() {
                    ext::Ground_801C4A08(ground_gobj);
                }
                grKinokoRoute_8020754C(0);
                grKinokoRoute_8020754C(2);
                ext::Ground_801C38BC(cam_target.x, cam_target.y);
                ext::grZakoGenerator_801CAF08();
                (*gp).u.kinokoroute2.cam_timer = 0x3C;
            }
        }
        {
            let zone_idx: s32 = (*gp).u.kinokoroute2.zone_idx as s32 + 5;
            if 7 > zone_idx && ext::Ground_801C2D24(zone_idx as enum_t, &mut zone_point) != 0 && fighter_pos.x > zone_point.x {
                (*gp).u.kinokoroute2.zone_idx += 1;
            }
        }
        (*ext::stage_info()).x6DC = (*gp).u.kinokoroute2.zone_idx as _;
    } else {
        let mut translate = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::Ground_801C2D24((*gp).u.kinokoroute2.spawn_idx as enum_t, &mut fighter_pos);
        fighter_pos.y += 30.0_f32;
        cam_target = fighter_pos;
        ext::Camera_80030AE0(0);
        (*ext::stage_info()).x6DC = 0;
        let mut reb0_jobj = ext::Ground_801C2CF4(4);
        let eve_jobj = ext::Ground_801C2CF4(0xBD);
        assert_(!reb0_jobj.is_null() && !eve_jobj.is_null(), 617, "reb0_jobj&&eve_jobj");
        HSD_JObjGetTranslation(eve_jobj, &mut translate);
        translate.y += 50.0_f32;
        HSD_JObjSetTranslate(reb0_jobj, &translate);
        if (*gp).u.kinokoroute2.cam_timer == 0 {
            grKinokoRoute_8020836C(gobj, 0);
        }
        if ext::Ground_801C5764() == 1 {
            (*gp).u.kinokoroute2.phase = 3;
            (*gp).u.kinokoroute2.cam_timer = 0x3C;
            grKinokoRoute_8020836C(gobj, 1);
            ext::grZakoGenerator_801CAEF0(1);
            reb0_jobj = ext::Ground_801C2CF4(4);
            assert_(!reb0_jobj.is_null(), 654, "reb0_jobj");
            HSD_JObjSetTranslate(reb0_jobj, &(*gp).u.kinokoroute2.reb0_pos);
            let mut ground_gobj = ext::Ground_GetMapGObj(0);
            if !ground_gobj.is_null() {
                ext::Ground_801C4A08(ground_gobj);
            }
            ground_gobj = ext::Ground_GetMapGObj(2);
            if !ground_gobj.is_null() {
                ext::Ground_801C4A08(ground_gobj);
            }
            grKinokoRoute_8020754C(2);
            grKinokoRoute_8020754C(0);
            ext::Ground_801C38BC(cam_target.x, cam_target.y);
        }
    }

    ext::Stage_UnkSetVec3TCam_Offset(&mut cam_offset);
    if (*gp).u.kinokoroute2.cam_timer == 0 {
        let mut diff = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lbVector_Diff(&mut cam_offset, &mut cam_target, &mut diff);
        // SQ(x) + SQ(y) + SQ(z): two fused multiply-adds, (x*x + y*y) first.
        let len2 = fmadds(diff.z, diff.z, fmadds(diff.x, diff.x, diff.y * diff.y));
        if sqrtf(len2) > 10.0_f32 {
            ext::lbVector_Normalize(&mut diff);
            diff.x *= 10.0_f32;
            diff.y *= 10.0_f32;
            diff.z *= 10.0_f32;
            ext::Ground_801C38BC(cam_target.x + diff.x, cam_target.y + diff.y);
        }
    } else {
        let step = 1.0_f32 / (*gp).u.kinokoroute2.cam_timer as f32;
        ext::lbVector_Sub(&mut cam_target, &mut cam_offset);
        cam_target.x *= step;
        cam_target.y *= step;
        cam_target.z *= step;
        ext::lbVector_Add(&mut cam_offset, &mut cam_target);
        ext::Ground_801C38BC(cam_offset.x, cam_offset.y);
        (*gp).u.kinokoroute2.cam_timer -= 1;
    }

    Ground_UpdateWindAndMapColl(gobj);
    if (*gp).u.kinokoroute2.flags_0() != 0 {
        ext::mpLib_80058560();
        (*gp).u.kinokoroute2.set_flags_0(0);
    }
}

pub unsafe extern "C" fn grKinokoRoute_80208368(_arg: *mut HSD_GObj) {}

pub unsafe fn grKinokoRoute_8020836C(gobj: *mut HSD_GObj, arg1: i32) {
    let jobj = ext::Ground_801C3FA4(gobj, 0x53);

    if jobj.is_null() {
        return;
    }

    if arg1 != 0 {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        ext::mpJointListAdd(0x3C);
        ext::mpJointListAdd(0x33);

        let mut cur = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
        while !cur.is_null() {
            if ext::itGetKind(cur) == It_PKind_Random {
                ext::grMaterial_801C8E08(cur);
            }
            cur = (*cur).next;
        }

        ext::mpJointListAdd(0x0C);
        ext::mpJointListAdd(0x0D);
        ext::mpJointListAdd(0x0E);
        ext::mpJointListAdd(0x0F);
    } else {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        ext::mpLib_80057BC0(0x3C);
        ext::mpLib_80057BC0(0x33);

        let mut cur = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
        while !cur.is_null() {
            if ext::itGetKind(cur) == It_PKind_Random {
                ext::grMaterial_801C8E28(cur);
            }
            cur = (*cur).next;
        }

        ext::mpLib_80057BC0(0x0C);
        ext::mpLib_80057BC0(0x0D);
        ext::mpLib_80057BC0(0x0E);
        ext::mpLib_80057BC0(0x0F);
    }
}

pub unsafe extern "C" fn grKinokoRoute_80208480(arg: i32) -> i32 {
    if arg == 1 {
        ext::Ground_801C5740(arg);
        return 1;
    }
    0
}

/// Item callback: `gobj` is the item gobj; its `user_data` is cast to a struct whose `jobj` is at
/// GC +0xDD8, `Item.xDD4_itemVar.yaku.x4`.
pub unsafe extern "C" fn grKinokoRoute_802084B4(gobj: *mut HSD_GObj) {
    let mut sp_vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let ip = (*gobj).user_data as *mut Item;
    let jobj = (*ip).xDD4_itemVar.yaku.anon_4.x4 as *mut HSD_JObj;

    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);

    let gobj2 = ext::Ground_GetMapGObj(3);
    if !gobj2.is_null() {
        let gp2 = (*gobj2).user_data as *mut Ground;
        if !gp2.is_null() {
            // ((UnkFlagStruct*) ((u8*) gp2 + 0xC4))->b0 = 1: the first bit of `Ground.u` (GC +0xC4)
            (*gp2).u.kinokoroute2.set_flags_0(1);
        }
    }

    ext::lb_8000B1CC(jobj, null_mut(), &mut sp_vec);
    ext::efSync_Spawn(0x442, gobj, &mut sp_vec);
    ext::Camera_RequestQuake(QuakeKind_Small, null_mut());
    ext::Ground_801C5414(0x136, 0xBA);
    ext::grMaterial_801C8CDC(gobj);
}

pub unsafe fn grKinokoRoute_80208564(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let depths = grNKr_803B82F4;

    for i in 0..0x33usize {
        let jobj = ext::Ground_801C3FA4(gobj, depths.x[i] as i32);
        let cb = core::mem::transmute::<
            unsafe extern "C" fn(*mut HSD_GObj),
            unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32),
        >(grKinokoRoute_802084B4);
        let item = ext::grMaterial_801C8CFC(8, 0, gp, jobj, None, Some(cb), None);
        if !item.is_null() {
            ext::grMaterial_801C8DE0(item, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 7.0);
            ext::grMaterial_801C8E08(item);
            ext::grMaterial_801C8E68(item, GA_Ground);
        }
    }
}

pub unsafe extern "C" fn grKinokoRoute_80208660(_unused: i32, gobj: *mut HSD_GObj) -> i32 {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    ext::ftLib_80086644(gobj, &mut pos);
    ext::ftLib_80086684(gobj, &mut vel);

    if pos.y < 5.5_f32 && vel.y > 5.5_f32 {
        let scale = ext::ftLib_80086B80(gobj) / 10.0_f32;
        pos.y = 5.5_f32;
        grKinokoRoute_802086EC(&mut pos, scale);
        ext::Ground_801C53EC(0x77A10);
    }

    0
}

pub unsafe fn grKinokoRoute_802086EC(arg0: *mut Vec3, arg8: f32) {
    let gen = ext::grLib_801C96F8(0x7530, 0x1E, arg0);
    if !gen.is_null() {
        let srt = (*gen).appsrt;
        (*srt).scale.x *= arg8;
        (*srt).scale.y *= arg8;
        (*srt).scale.z *= arg8;
    }
}

pub unsafe extern "C" fn grKinokoRoute_8020875C(a: *mut Vec3, _b: i32, jobj: *mut HSD_JObj) -> i32 {
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, null_mut(), &mut vec);
    if (*a).y > vec.y {
        1
    } else {
        0
    }
}

pub unsafe fn grKinokoRoute_802087B0() -> f32 {
    10.0_f32
}

#[allow(dead_code)]
type _Unused = c_void;
