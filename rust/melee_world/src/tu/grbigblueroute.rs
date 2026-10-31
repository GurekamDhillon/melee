//! src/melee/gr/grbigblueroute.c: Big Blue (F-Zero) "Route" stage, the race-car spawner and the
//! per-frame car motion (`grBigBlueRoute_8020CD20`). Fused sites (tools/rust-port/fma_sites.py), written out as
//! `fmadds`: `x20 = 5.2359877f * HSD_Randf() - 2.6179938f` (`SpawnRoute`, and case 0 of `8020CD20`) and
//! `frac = (1.0f - prog) * (xC - 0.5f) + 0.5f` (case 2). Native adaptations of GameCube layout tricks, all
//! numerically identical to the PPC build: the car table is `31 * sizeof(RouteEntry)` bytes (the C allocates the
//! GC size 0x554), `8020CD20` advances `jobj` through `jobj->next` (the C reads the pointer through an `int`
//! view at +8), and `8020DE48` indexes `unk8[2]` (the C adds the GC byte offset 0x68).
//! State-oracle checked against the C (tests/gro_grbigblueroute.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use super::grinl::*;
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_core::fp::fmadds;
use melee_types::*;

pub use crate::gr::trivial::grbigblueroute::{
    grBigBlueRoute_8020B9CC, grBigBlueRoute_8020BB00, grBigBlueRoute_8020BB58, grBigBlueRoute_8020BC24,
    grBigBlueRoute_8020BF30, grBigBlueRoute_8020C1D4, grBigBlueRoute_8020DF78,
};

/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::stage_info;
    hook_or!(CAMERA_8002F3AC, Camera_8002F3AC, (), || { unsafe { crate::tu::cmcamera::Camera_8002F3AC(); } });
    hook_or!(CAMERA_80030AE0, Camera_80030AE0, (a0: i32), |a0: i32| crate::cm::camera::world().camera_80030ae0(a0));
    hook_or!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, (a0: CmQuakeKind, a1: *mut Vec), |a0: CmQuakeKind, a1: *mut Vec| unsafe { crate::cm::camera::Camera_RequestQuake(a0 as _, a1 as _) });
    hook_or!(GOBJ_SETUPGXLINK, GObj_SetupGXLink, (gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32), |gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(gobj as _, render_cb as _, gx_link as _, priority as _); } });
    pub static mut GROUND_801C0498: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C0498() -> f32 {
        match unsafe { GROUND_801C0498 } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_801C0498() as _ }; } }
    }
    pub static mut GROUND_801C10B8: Option<fn(*mut HSD_GObj, HSD_GObjEvent)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C10B8(a0: *mut HSD_GObj, a1: HSD_GObjEvent) {
        match unsafe { GROUND_801C10B8 } { Some(f) => f(a0, a1), None => { unsafe { crate::tu::ground::Ground_801C10B8(a0 as _, a1 as _); } return; } }
    }
    pub static mut GROUND_801C247C: Option<fn(s32, s32) -> *mut HSD_Spline> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C247C(a0: s32, a1: s32) -> *mut HSD_Spline {
        match unsafe { GROUND_801C247C } { Some(f) => f(a0, a1), None => { return unsafe { crate::tu::ground::Ground_801C247C(a0 as _, a1 as _) as _ }; } }
    }
    pub static mut GROUND_801C2CF4: Option<fn(s32) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C2CF4(a0: s32) -> *mut HSD_JObj {
        match unsafe { GROUND_801C2CF4 } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_801C2CF4(a0 as _) as _ }; } }
    }
    pub static mut GROUND_801C2D24: Option<fn(enum_t, *mut Vec) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C2D24(a0: enum_t, a1: *mut Vec) -> i32 {
        match unsafe { GROUND_801C2D24 } { Some(f) => f(a0, a1), None => { return unsafe { crate::tu::ground::Ground_801C2D24(a0 as _, a1 as _) as _ }; } }
    }
    pub static mut GROUND_801C38BC: Option<fn(f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C38BC(x: f32, y: f32) {
        match unsafe { GROUND_801C38BC } { Some(f) => f(x, y), None => { unsafe { crate::tu::ground::Ground_801C38BC(x as _, y as _); } return; } }
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
    pub static mut GROUND_801C3D44: Option<fn(*mut c_void, f32, f32) -> s32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3D44(a0: *mut c_void, a1: f32, a2: f32) -> s32 {
        match unsafe { GROUND_801C3D44 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::ground::Ground_801C3D44(a0 as _, a1 as _, a2 as _) as _ }; } }
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
    pub static mut GROUND_801C4E70: Option<fn(*mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C4E70(a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj) {
        match unsafe { GROUND_801C4E70 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { unsafe { crate::tu::ground::Ground_801C4E70(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } return; } }
    }
    pub static mut GROUND_801C53EC: Option<fn(u32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C53EC(a0: u32) {
        match unsafe { GROUND_801C53EC } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C53EC(a0 as _); } return; } }
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
    pub static mut GROUND_801C5AEC: Option<fn(*mut Vec, *mut Vec, *mut Vec, *mut Vec)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5AEC(v: *mut Vec, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec) {
        match unsafe { GROUND_801C5AEC } { Some(f) => f(v, a1, a2, a3), None => { unsafe { crate::tu::ground::Ground_801C5AEC(v as _, a1 as _, a2 as _, a3 as _); } return; } }
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
    pub static mut HSD_FREE: Option<fn(*mut c_void)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_Free(ptr: *mut c_void) {
        match unsafe { HSD_FREE } { Some(f) => f(ptr), None => { unsafe { melee_runtime::tu::memory::HSD_Free(ptr as _); } return; } }
    }
    hook_or!(HSD_GOBJ_SETUPPROC, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc, |a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ });
    hook_or!(HSD_JOBJCLEARFLAGSALL, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, flags as _); } });
    hook_or!(HSD_JOBJREPARENT, HSD_JObjReparent, (jobj: *mut HSD_JObj, parent: *mut HSD_JObj) -> *mut HSD_JObj, |jobj: *mut HSD_JObj, parent: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjReparent(jobj as _, parent as _) as _ });
    hook_or!(HSD_JOBJSETFLAGSALL, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, flags as _); } });
    pub static mut HSD_MEMALLOC: Option<fn(ssize_t) -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn HSD_MemAlloc(size: ssize_t) -> *mut c_void {
        match unsafe { HSD_MEMALLOC } { Some(f) => f(size), None => { return unsafe { melee_runtime::tu::memory::HSD_MemAlloc(size as _) as _ }; } }
    }
    hook_or!(HSD_RANDF, HSD_Randf, () -> f32, || melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand()));
    hook_or!(HSD_RANDI, HSD_Randi, (max_val: s32) -> s32, |max_val: s32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), max_val));
    hook_or!(OSREPORT, OSReport, (a0: *const i8, va: &[usize]), |a0: *const i8, va: &[usize]| unsafe { melee_runtime::tu::os_error::OSReport(a0, va) });
    hook_or!(PSVECCROSSPRODUCT, PSVECCrossProduct, (vec1: *mut Vec, vec2: *mut Vec, dst: *mut Vec), |vec1: *mut Vec, vec2: *mut Vec, dst: *mut Vec| { unsafe { melee_runtime::tu::mtx_pc::raw::PSVECCrossProduct(vec1 as _, vec2 as _, dst as _); } });
    hook_or!(ATAN2F, atan2f, (y: f32, x: f32) -> f32, melee_runtime::lbtrigf::atan2f);
    hook_or!(FTLIB_80086644, ftLib_80086644, (a0: *mut HSD_GObj, a1: *mut Vec), |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_80086644(a0 as _, a1 as _); } });
    hook_or!(FTLIB_80086EC0, ftLib_80086EC0, (a0: *mut HSD_GObj) -> i32, |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086EC0(a0 as _) as _ });
    hook!(GM_8016B3D8, gm_8016B3D8, () -> i32);
    pub static mut GRANIME_801C77FC: Option<fn(*mut HSD_GObj, i32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C77FC(a0: *mut HSD_GObj, a1: i32, a2: u32) {
        match unsafe { GRANIME_801C77FC } { Some(f) => f(a0, a1, a2), None => { unsafe { crate::tu::granime::grAnime_801C77FC(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRANIME_801C8138: Option<fn(*mut HSD_GObj, enum_t, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C8138(a0: *mut HSD_GObj, a1: enum_t, a2: i32) {
        match unsafe { GRANIME_801C8138 } { Some(f) => f(a0, a1, a2), None => { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRDATFILES_801C6330: Option<fn(s32) -> *mut UnkArchiveStruct> = None;
    #[allow(non_snake_case)]
    pub fn grDatFiles_801C6330(a0: s32) -> *mut UnkArchiveStruct {
        match unsafe { GRDATFILES_801C6330 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grdatfiles::grDatFiles_801C6330(a0 as _) as _ }; } }
    }
    pub static mut GRFZEROCAR_801CAFBC: Option<fn(*mut HSD_GObj, *mut c_void, s32, s32)> = None;
    #[allow(non_snake_case)]
    pub fn grFZeroCar_801CAFBC(a0: *mut HSD_GObj, a1: *mut c_void, a2: s32, a3: s32) {
        match unsafe { GRFZEROCAR_801CAFBC } { Some(f) => f(a0, a1, a2, a3), None => { unsafe { crate::tu::grfzerocar::grFZeroCar_801CAFBC(a0 as _, a1 as _, a2 as _, a3 as _); } return; } }
    }
    pub static mut GRMATERIAL_801C8CFC: Option<fn(i32, i32, *mut Ground, *mut HSD_JObj, Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8CFC(a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, arg4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, arg5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, arg6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj {
        match unsafe { GRMATERIAL_801C8CFC } { Some(f) => f(a0, a1, a2, a3, arg4, arg5, arg6), None => { return unsafe { crate::tu::grmaterial::grMaterial_801C8CFC(a0 as _, a1 as _, a2 as _, a3 as _, arg4 as _, arg5 as _, arg6 as _) as _ }; } }
    }
    pub static mut GRMATERIAL_801C8E28: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8E28(a0: *mut HSD_GObj) {
        match unsafe { GRMATERIAL_801C8E28 } { Some(f) => f(a0), None => { unsafe { crate::tu::grmaterial::grMaterial_801C8E28(a0 as _); } return; } }
    }
    pub static mut GRZAKOGENERATOR_801CAE04: Option<fn(*mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj {
        match unsafe { GRZAKOGENERATOR_801CAE04 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ }; } }
    }
    pub static mut GRZAKOGENERATOR_801CAEB0: Option<fn(i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAEB0(a0: i32, a1: i32) {
        match unsafe { GRZAKOGENERATOR_801CAEB0 } { Some(f) => f(a0, a1), None => { unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAEB0(a0 as _, a1 as _); } return; } }
    }
    pub static mut LBSHADOW_8000E9F0: Option<fn(*mut Vec, *mut HSD_Spline, f32)> = None;
    #[allow(non_snake_case)]
    pub fn lbShadow_8000E9F0(a0: *mut Vec, a1: *mut HSD_Spline, a2: f32) {
        match unsafe { LBSHADOW_8000E9F0 } {
            Some(f) => f(a0, a1, a2),
            None => { unsafe { melee_runtime::tu::lbshadow::lbShadow_8000E9F0(&mut *(a0 as *mut _), &*(a1 as *const _), a2 as _); } },
        }
    }
    pub static mut LBVECTOR_ADD: Option<fn(*mut Vec, *mut Vec) -> *mut Vec> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Add(a: *mut Vec, b: *mut Vec) -> *mut Vec {
        match unsafe { LBVECTOR_ADD } { Some(f) => f(a, b), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Add(a as _, b as _) as _ }; } }
    }
    pub static mut LBVECTOR_DIFF: Option<fn(*mut Vec, *mut Vec, *mut Vec) -> *mut Vec> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Diff(a: *mut Vec, b: *mut Vec, result: *mut Vec) -> *mut Vec {
        match unsafe { LBVECTOR_DIFF } { Some(f) => f(a, b, result), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Diff(a as _, b as _, result as _) as _ }; } }
    }
    pub static mut LBVECTOR_NORMALIZE: Option<fn(*mut Vec) -> f32> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Normalize(vec: *mut Vec) -> f32 {
        match unsafe { LBVECTOR_NORMALIZE } { Some(f) => f(vec), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(vec as _) as _ }; } }
    }
    pub static mut LBVECTOR_ROTATEABOUTUNITAXIS: Option<fn(*mut Vec, *mut Vec, f32)> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_RotateAboutUnitAxis(v: *mut Vec, axis: *mut Vec, angle: f32) {
        match unsafe { LBVECTOR_ROTATEABOUTUNITAXIS } { Some(f) => f(v, axis, angle), None => { unsafe { melee_runtime::tu::lbvector::lbVector_RotateAboutUnitAxis(v as _, axis as _, angle as _); } return; } }
    }
    pub static mut LBVECTOR_SUB: Option<fn(*mut Vec, *mut Vec) -> *mut Vec> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Sub(a: *mut Vec, b: *mut Vec) -> *mut Vec {
        match unsafe { LBVECTOR_SUB } { Some(f) => f(a, b), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Sub(a as _, b as _) as _ }; } }
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec, *mut Vec)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec, pos1: *mut Vec) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    pub static mut LB_800115F4: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn lb_800115F4() {
        match unsafe { LB_800115F4 } { Some(f) => f(), None => { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } return; } }
    }
    pub static mut MEMZERO: Option<fn(*mut c_void, ssize_t)> = None;
    #[allow(non_snake_case)]
    pub fn memzero(mem: *mut c_void, size: ssize_t) {
        match unsafe { MEMZERO } { Some(f) => f(mem, size), None => { unsafe { melee_runtime::tu::lb_00B0::memzero(mem as _, size as _); } return; } }
    }
    hook_or!(SPLGETSPLINEPOINT, splGetSplinePoint, (a0: *mut Vec, a1: *mut HSD_Spline, a2: f32), |a0: *mut Vec, a1: *mut HSD_Spline, a2: f32| { unsafe { melee_runtime::tu::spline_world::splGetSplinePoint(a0 as _, a1 as _, a2 as _); } });
    pub static mut UN_802FD604: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn un_802FD604(a0: i32) {
        match unsafe { UN_802FD604 } { Some(f) => f(a0), None => { unsafe { crate::tu::ifhazard::un_802FD604(a0 as _); } return; } }
    }
    pub static mut UN_802FD65C: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn un_802FD65C() {
        match unsafe { UN_802FD65C } { Some(f) => f(), None => { unsafe { crate::tu::ifhazard::un_802FD65C(); } return; } }
    }
    pub static mut UN_802FD8E8: Option<fn(*mut DynamicModelDesc)> = None;
    #[allow(non_snake_case)]
    pub fn un_802FD8E8(a0: *mut DynamicModelDesc) {
        match unsafe { UN_802FD8E8 } { Some(f) => f(a0), None => { unsafe { crate::tu::ifhazard::un_802FD8E8(a0 as _); } return; } }
    }
    /// `grDisplay_801C5DB0` is only ever passed as a render callback (its address is the argument).
    pub unsafe extern "C" fn grDisplay_801C5DB0(_gobj: *mut HSD_GObj, _pass: i32) {
        { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(_gobj as _, _pass as _); } return; }
    }
}

/// `struct grBigBlueRoute_YakumonoParam` (file-local).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grBigBlueRoute_YakumonoParam {
    pub x0: i32,
    pub x4: f32,
    pub pad_8: [u8; 0x20 - 0x8],
    pub x20: f32,
    pub pad_24: [u8; 0x3C - 0x24],
    pub x3C: f32,
    pub x40: f32,
    pub x44: f32,
    pub x48: f32,
    pub x4C: f32,
}

/// `struct grBigBlueRoute_8020DA9C_t`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grBigBlueRoute_8020DA9C_t {
    pub pad_0: [i8; 8],
    pub x8: i32,
}

/// `static struct grBigBlueRoute_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grBigBlueRoute_YakumonoParam = null_mut();

macro_rules! cbs {
    () => {
        StageCallbacks { on_init: None, callback1: None, gobj_proc: None, callback3: None, anon_10: StageCallbacks_anon_135 { flags: 0 } }
    };
    ($a:ident, $b:ident, $c:ident, $d:ident, $flags:expr) => {
        StageCallbacks {
            on_init: Some($a),
            callback1: Some($b),
            gobj_proc: Some($c),
            callback3: Some($d),
            anon_10: StageCallbacks_anon_135 { flags: u32::to_be($flags) },
        }
    };
}

/// `StageCallbacks grBb_Route_StageCallbacks[38]`.
pub static mut grBb_Route_StageCallbacks: [StageCallbacks; 38] = [
    cbs!(grBigBlueRoute_8020BABC, grBigBlueRoute_8020BB00, grBigBlueRoute_8020BB08, grBigBlueRoute_8020BB0C, 0),
    cbs!(grBigBlueRoute_8020BB68, grBigBlueRoute_8020BC24, grBigBlueRoute_8020BC2C, grBigBlueRoute_8020BC30, 0),
    cbs!(), cbs!(),
    cbs!(grBigBlueRoute_8020C140, grBigBlueRoute_8020C1D4, grBigBlueRoute_8020C1DC, grBigBlueRoute_8020C210, 0x80000000),
    cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(),
    cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(),
    cbs!(grBigBlueRoute_8020BC68, grBigBlueRoute_8020BF30, grBigBlueRoute_8020BF38, grBigBlueRoute_8020C13C, 0x40000000),
    cbs!(grBigBlueRoute_8020BB10, grBigBlueRoute_8020BB58, grBigBlueRoute_8020BB60, grBigBlueRoute_8020BB64, 0),
    cbs!(), cbs!(), cbs!(), cbs!(), cbs!(),
];

/// `StageData grBb_Route_StageData`.
pub static mut grBb_Route_StageData: StageData = StageData {
    grkind: Gr_Kind_BigBlueRoute,
    callbacks: core::ptr::addr_of!(grBb_Route_StageCallbacks) as *mut StageCallbacks,
    data1: b"/GrNBr.dat\0".as_ptr() as *mut i8,
    on_init: Some(grBigBlueRoute_8020B89C),
    on_demo_init: Some(grBigBlueRoute_8020B864),
    on_load: Some(grBigBlueRoute_8020B920),
    on_start: Some(grBigBlueRoute_8020B95C),
    callback4: Some(grBigBlueRoute_8020B9CC),
    on_touch_line: Some(grBigBlueRoute_8020DF78),
    on_check_shadow_render: Some(grBigBlueRoute_8020DF80),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

/// `static s16 grBb_Route_carJObjIds[30]`.
pub static mut grBb_Route_carJObjIds: [i16; 30] = [
    4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 3, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 34, 35, 36, 37,
];

/// `static const Vec3 grBb_Route_803B83E0`.
pub static mut grBb_Route_803B83E0: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };

pub unsafe extern "C" fn grBigBlueRoute_8020B864(_arg: i32) {
    melee_core::ct!("grBigBlueRoute_8020B864");
    let gobj = ext::Ground_GetMapGObj(0x1F);
    if !gobj.is_null() {
        ext::Ground_801C4A08(gobj);
    }
    grBigBlueRoute_8020B9D4(1);
}

pub unsafe extern "C" fn grBigBlueRoute_8020B89C() {
    melee_core::ct!("grBigBlueRoute_8020B89C");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grBigBlueRoute_YakumonoParam;
    let si = ext::stage_info();
    (*si).unk8C.set_b4(0);
    (*si).unk8C.set_b5(1);
    grBigBlueRoute_8020B9D4(0);
    grBigBlueRoute_8020B9D4(0x20);
    grBigBlueRoute_8020B9D4(0x1F);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Camera_80030AE0(1);
    ext::un_802FD8E8(grBigBlueRoute_8020DE48());
    ext::un_802FD65C();
}

pub unsafe extern "C" fn grBigBlueRoute_8020B920() {
    melee_core::ct!("grBigBlueRoute_8020B920");
    let mut fighter_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let fighter = ext::Ground_GetP1Fighter();
    if !fighter.is_null() {
        ext::ftLib_80086644(fighter, &mut fighter_pos);
        grBigBlueRoute_8020DED4(&mut fighter_pos);
        ext::Camera_8002F3AC();
    }
}

pub unsafe extern "C" fn grBigBlueRoute_8020B95C() {
    melee_core::ct!("grBigBlueRoute_8020B95C");
    ext::grZakoGenerator_801CAE04(null_mut());
    if ext::gm_8016B3D8() == 0 {
        return;
    }

    let mut val = (*yakumono_param).x0;
    if val != 0 {
        val = ext::HSD_Randi((*yakumono_param).x0);
    } else {
        val = 0;
    }
    if val == 0 {
        ext::grZakoGenerator_801CAEB0(ext::Ground_801C5840(), ext::Ground_801C5940());
    }
}

pub unsafe fn grBigBlueRoute_8020B9D4(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grBigBlueRoute_8020B9D4");
    let callbacks: *mut StageCallbacks = &mut grBb_Route_StageCallbacks[gobj_id as usize];
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
        ext::OSReport(
            b"%s:%d: couldn t get gobj(id=%d)\n\0".as_ptr() as *const i8,
            &[b"src/melee/gr/grbigblueroute.c\0".as_ptr() as usize, 0x117, gobj_id as u32 as usize],
        );
    }
    gobj
}

pub unsafe extern "C" fn grBigBlueRoute_8020BABC(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020BABC");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grBigBlueRoute_8020BB08(_arg: *mut Ground_GObj) { melee_core::ct!("grBigBlueRoute_8020BB08");}
pub unsafe extern "C" fn grBigBlueRoute_8020BB0C(_arg: *mut Ground_GObj) { melee_core::ct!("grBigBlueRoute_8020BB0C");}

pub unsafe extern "C" fn grBigBlueRoute_8020BB10(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020BB10");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    ext::grAnime_801C77FC(gobj, 0, 7);
}

pub unsafe extern "C" fn grBigBlueRoute_8020BB60(_arg: *mut Ground_GObj) { melee_core::ct!("grBigBlueRoute_8020BB60");}
pub unsafe extern "C" fn grBigBlueRoute_8020BB64(_arg: *mut Ground_GObj) { melee_core::ct!("grBigBlueRoute_8020BB64");}

pub unsafe extern "C" fn grBigBlueRoute_8020BB68(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020BB68");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    ext::grAnime_801C77FC(gobj, 0, 7);
    let jobj6 = ext::Ground_801C3FA4(gobj, 7);
    let jobj5 = ext::Ground_801C3FA4(gobj, 8);
    let jobj4 = ext::Ground_801C3FA4(gobj, 4);
    let jobj3 = ext::Ground_801C3FA4(gobj, 5);
    let jobj2 = ext::Ground_801C3FA4(gobj, 3);
    let jobj = ext::Ground_801C3FA4(gobj, 6);
    ext::Ground_801C4E70(jobj, jobj2, jobj3, jobj4, jobj5, jobj6);
}

pub unsafe extern "C" fn grBigBlueRoute_8020BC2C(_arg: *mut Ground_GObj) { melee_core::ct!("grBigBlueRoute_8020BC2C");}
pub unsafe extern "C" fn grBigBlueRoute_8020BC30(_arg: *mut Ground_GObj) { melee_core::ct!("grBigBlueRoute_8020BC30");}

pub unsafe extern "C" fn grBigBlueRoute_8020BC34(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020BC34");
    let gp = GET_GROUND(gobj);
    (*gp).u.car.xC4 = grBigBlueRoute_8020B9D4(4);
}

pub unsafe extern "C" fn grBigBlueRoute_8020BC68(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020BC68");
    let mut origin = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = GET_GROUND(gobj);

    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    (*gp).x10_flags.set_b5(1);
    (*gp).u.bigblueroute2.xC8 = 0;

    if ext::Ground_801C2D24(148, &mut origin) != 0 {
        for i in 0..4usize {
            let mut reb0_jobj = ext::Ground_801C2CF4(127 + i as s32);
            (*gp).u.bigblueroute2.tracks[i].jobj = reb0_jobj;
            if !reb0_jobj.is_null() {
                reb0_jobj = (*gp).u.bigblueroute2.tracks[i].jobj;
                HSD_JObjGetTranslation(reb0_jobj, &mut (*gp).u.bigblueroute2.tracks[i].offset);
                ext::lbVector_Sub(&mut (*gp).u.bigblueroute2.tracks[i].offset, &mut origin);
            }
        }
    } else {
        (*gp).u.bigblueroute2.tracks[0].jobj = null_mut();
        (*gp).u.bigblueroute2.tracks[1].jobj = null_mut();
        (*gp).u.bigblueroute2.tracks[2].jobj = null_mut();
        (*gp).u.bigblueroute2.tracks[3].jobj = null_mut();
    }

    let reb0_jobj = ext::Ground_801C2CF4(4);
    assert_(!reb0_jobj.is_null(), 452, "reb0_jobj");
    HSD_JObjGetTranslation(reb0_jobj, &mut (*gp).u.bigblueroute2.xCC);
    ext::Ground_801C10B8(gobj, Some(grBigBlueRoute_8020BC34));
}

pub unsafe extern "C" fn grBigBlueRoute_8020BF38(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020BF38");
    let gp = GET_GROUND(gobj);
    let mut checkpoint = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut fighter_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let fighter = ext::Ground_GetP1Fighter();
    ext::Ground_801C3D44(null_mut(), 30.0, 4000.0);

    if !fighter.is_null() {
        ext::ftLib_80086644(fighter, &mut fighter_pos);

        if ext::ftLib_80086EC0(fighter) == 0 {
            grBigBlueRoute_8020DED4(&mut fighter_pos);

            for i in 0..3usize {
                if !(*gp).u.bigblueroute2.tracks[i].jobj.is_null() {
                    let mut jobj = (*gp).u.bigblueroute2.tracks[i].jobj;
                    HSD_JObjSetTranslateX(jobj, (*gp).u.bigblueroute2.tracks[i].offset.x + fighter_pos.x);

                    jobj = (*gp).u.bigblueroute2.tracks[i].jobj;
                    HSD_JObjSetTranslateY(jobj, (*gp).u.bigblueroute2.tracks[i].offset.y + fighter_pos.y);
                }
            }

            ext::Camera_80030AE0(1);
        }

        {
            let idx = (*gp).u.bigblueroute2.xC8 as i32 + 5;
            if (*gp).u.bigblueroute2.xC8 as i32 + 5 <= 7 {
                if ext::Ground_801C2D24(idx, &mut checkpoint) != 0 {
                    if fighter_pos.x > checkpoint.x {
                        (*gp).u.bigblueroute2.xC8 = ((*gp).u.bigblueroute2.xC8 as i32 + 1) as i16;
                    }
                }
            }
        }
        (*ext::stage_info()).x6DC = (*gp).u.bigblueroute2.xC8;
    }

    ext::lb_800115F4();
}

pub unsafe extern "C" fn grBigBlueRoute_8020C13C(_arg: *mut Ground_GObj) { melee_core::ct!("grBigBlueRoute_8020C13C");}

pub unsafe extern "C" fn grBigBlueRoute_8020C140(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020C140");
    let gp = (*gobj).user_data as *mut Ground;

    ext::grFZeroCar_801CAFBC(gobj, addr_of_mut!(grBb_Route_carJObjIds) as *mut c_void, 30, 0);
    (*gp).u.car.xCC = ext::Ground_801C247C(33, 1);
    (*gp).u.car.xD0 = ext::Ground_801C247C(33, 0);
    (*gp).u.car.xD4 = ext::Ground_801C247C(33, 2);
    grBigBlueRoute_8020C238(gobj);
    // ((UnkFlagStruct*) &gp->u.car.xC4)->b0 = 0
    (*(addr_of_mut!((*gp).u.car.xC4) as *mut UnkFlagStruct)).anon_0.set_b0(0);
}

pub unsafe extern "C" fn grBigBlueRoute_8020C1DC(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020C1DC");
    grBigBlueRoute_8020C85C(gobj);
    grBigBlueRoute_8020CD20(gobj);
}

pub unsafe extern "C" fn grBigBlueRoute_8020C210(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020C210");
    let gp = GET_GROUND(gobj);
    ext::HSD_Free((*gp).u.car.car_info);
}

/// Bytes of the car table: 31 `RouteEntry` records (the C allocates the GameCube size 0x554).
const CAR_BYTES: usize = 31 * core::mem::size_of::<RouteEntry>();

pub unsafe fn grBigBlueRoute_8020C238(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020C238");
    let root_jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let gp = (*gobj).user_data as *mut Ground;
    let mut spline_pt = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if !root_jobj.is_null() {
        let mut jobj = (*root_jobj).child;

        while !jobj.is_null() {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);

            let mut val = (*yakumono_param).x4;
            HSD_JObjSetScaleX(jobj, val);

            val = (*yakumono_param).x4;
            HSD_JObjSetScaleY(jobj, val);

            val = (*yakumono_param).x4;
            HSD_JObjSetScaleZ(jobj, val);

            jobj = (*jobj).next;
        }
    }

    (*gp).u.car.car_info = ext::HSD_MemAlloc(CAR_BYTES as ssize_t);
    assert_(!(*gp).u.car.car_info.is_null(), 0x2A2, "gp->u.car.car_info");
    ext::memzero((*gp).u.car.car_info, CAR_BYTES as ssize_t);

    (*gp).u.car.x10A = 0;
    (*gp).u.car.x108 = 0;

    ext::splGetSplinePoint(&mut (*gp).u.car.xD8, (*gp).u.car.xCC, 0.0);
    ext::splGetSplinePoint(&mut (*gp).u.car.xE4, (*gp).u.car.xCC, 1.0);

    ext::splGetSplinePoint(&mut (*gp).u.car.xF0, (*gp).u.car.xD0, 0.0);
    ext::splGetSplinePoint(&mut spline_pt, (*gp).u.car.xD4, 0.0);
    ext::lbVector_Add(&mut (*gp).u.car.xF0, &mut spline_pt);

    (*gp).u.car.xF0.x *= 0.5;
    (*gp).u.car.xF0.y *= 0.5;
    (*gp).u.car.xF0.z *= 0.5;

    ext::splGetSplinePoint(&mut (*gp).u.car.xFC, (*gp).u.car.xD0, 1.0);
    ext::splGetSplinePoint(&mut spline_pt, (*gp).u.car.xD4, 1.0);
    ext::lbVector_Add(&mut (*gp).u.car.xFC, &mut spline_pt);

    (*gp).u.car.xFC.x *= 0.5;
    (*gp).u.car.xFC.y *= 0.5;
    (*gp).u.car.xFC.z *= 0.5;
}

#[inline(always)]
unsafe fn re_at(gp: *mut Ground, i: usize) -> *mut RouteEntry {
    ((*gp).u.car.car_info as *mut RouteEntry).add(i)
}

pub unsafe fn grBigBlueRoute_8020C530(arg0: *mut Ground_GObj) -> s32 {
    melee_core::ct!("grBigBlueRoute_8020C530");
    let gp = (*arg0).user_data as *mut Ground;
    let mut count: s32 = 0;

    for i in 0..30usize {
        if (*re_at(gp, i)).flags.b0() == 0 {
            count += 1;
        }
    }

    if count == 0 {
        return -1;
    }

    if count != 0 {
        count = ext::HSD_Randi(count);
    } else {
        count = 0;
    }

    for i in 0..30usize {
        if (*re_at(gp, i)).flags.b0() == 0 {
            count -= 1;
            if count < 0 {
                return i as s32;
            }
        }
    }
    assert_(false, 0x2E5, "0");
    unreachable!()
}

/// `static inline grBigBlueRoute_SpawnRoute`.
pub unsafe fn grBigBlueRoute_SpawnRoute(route_idx: s32, gp: *mut Ground, gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_SpawnRoute");
    let mut route_idx = route_idx;
    if route_idx == 0 {
        route_idx = 30;
        (*re_at(gp, 30)).flags.set_b1(1);
        (*gp).u.car.x10A = (*yakumono_param).x4C as i16;
    } else {
        route_idx = grBigBlueRoute_8020C530(gobj);

        (*re_at(gp, route_idx as usize)).flags.set_b1(0);

        let min_val = (*yakumono_param).x44 as s32;
        let max_val = (*yakumono_param).x48 as s32;
        let new_timer: s32;

        if max_val > min_val {
            let range = max_val - min_val;
            new_timer = min_val + if range != 0 { ext::HSD_Randi(range) } else { 0 };
        } else if max_val < min_val {
            let range = min_val - max_val;
            new_timer = max_val + if range != 0 { ext::HSD_Randi(range) } else { 0 };
        } else {
            new_timer = max_val;
        }
        (*gp).u.car.x10A = new_timer as i16;
    }

    if route_idx != -1 {
        let re = re_at(gp, route_idx as usize);
        (*re).flags.set_b0(1);
        (*re).flags.set_b6(0);
        (*re).flags.set_b2_5(1);

        (*re).x4 = 1.0;
        (*re).x8 = (*yakumono_param).x20;
        (*re).xC = 0.5;
        {
            let rand = ext::HSD_Randf();
            (*re).x10 = rand;
        }
        (*re).x14 = 0.0;
        (*re).x18 = (*yakumono_param).x3C;
        (*re).x1C = 0.0;
        {
            let rand = ext::HSD_Randf();
            (*re).x20 = fmadds(5.2359877, rand, -2.6179938);
        }

        if (*re).flags.b1() == 0 {
            let root = (*gobj).hsd_obj as *mut HSD_JObj;
            if !root.is_null() {
                let mut jobj = (*root).child;

                if !jobj.is_null() {
                    let mut i = 0;
                    while i < route_idx && !jobj.is_null() {
                        i += 1;
                        jobj = (*jobj).next;
                    }

                    assert_(!jobj.is_null(), 901, "jobj");
                    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

                    HSD_JObjSetRotationX(jobj, 0.0);
                    HSD_JObjSetRotationY(jobj, 0.0);
                    HSD_JObjSetRotationZ(jobj, 0.0);

                    {
                        let item = ext::grMaterial_801C8CFC(
                            0,
                            1,
                            gp,
                            jobj,
                            Some(core::mem::transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>(fn_8020DEAC)),
                            None,
                            None,
                        );
                        (*re).x28 = item as *mut c_void;
                        if !item.is_null() {
                            ext::grMaterial_801C8E28((*re).x28 as *mut HSD_GObj);
                        }
                    }
                }
            }
        }
    }

    (*gp).u.car.x108 += 1;
}

pub unsafe extern "C" fn grBigBlueRoute_8020C85C(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020C85C");
    let gp = GET_GROUND(gobj);

    if !(((*gp).u.car.x108 as f32) < 1.0f32 + (*yakumono_param).x40) {
        return;
    }

    let t = (*gp).u.car.x10A;
    (*gp).u.car.x10A = t.wrapping_sub(1);
    if t >= 0 {
        return;
    }

    grBigBlueRoute_SpawnRoute((*gp).u.car.x108 as s32, gp, gobj);
}

pub unsafe extern "C" fn grBigBlueRoute_8020CD20(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBigBlueRoute_8020CD20");
    let gp = GET_GROUND(gobj);
    let root_jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut rot = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut fighter_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c2_road = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c2_air = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c2_road_rot = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c3_air = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c3_road = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c3_air_rot = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c0_tangent = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c0_p1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c0_p0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c1_orient = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c1_side = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c1_up = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c1_tangent = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c2_road_tan = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c2_p1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c2_p0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c2_orient = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c2_side = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c2_up = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c2_tangent = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c3_orient = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c3_side = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c3_up = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c3_tangent = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c3_road_tan = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c3_p1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut c3_p0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut t: f32 = 0.0;
    let mut frac: f32 = 0.0;
    let _ = (&mut t, &mut frac);

    let fighter = ext::Ground_GetP1Fighter();
    if !fighter.is_null() {
        ext::ftLib_80086644(fighter, &mut fighter_pos);
    } else {
        return;
    }

    if root_jobj.is_null() {
        return;
    }
    let mut jobj = (*root_jobj).child;
    if jobj.is_null() {
        return;
    }

    let mut i: usize = 0;
    loop {
        let re = re_at(gp, i);
        if (*re).flags.b0() != 0 {
            (*re).x4 += (*re).x8;
            if (*re).x4 > 1.0 {
                (*re).x4 = 1.0;
            }
            (*re).x14 += (*re).x18;
            if (*re).x14 > 1.0 {
                (*re).x14 = 1.0;
            }

            match (*re).flags.b2_5() {
                0 => {
                    let t: f32;
                    let frac: f32;

                    if (*re).xC < (*re).x10 {
                        (*re).xC += 0.01;
                        if (*re).xC > (*re).x10 {
                            (*re).xC = (*re).x10;
                        }
                    } else if (*re).xC > (*re).x10 {
                        (*re).xC -= 0.01;
                        if (*re).xC < (*re).x10 {
                            (*re).xC = (*re).x10;
                        }
                    }

                    t = (*re).x4;
                    frac = (*re).xC;
                    ext::splGetSplinePoint(&mut c0_p0, (*gp).u.car.xD0, t);
                    ext::splGetSplinePoint(&mut c0_p1, (*gp).u.car.xD4, t);
                    ext::lbShadow_8000E9F0(&mut c0_tangent, (*gp).u.car.xD0, t);
                    ext::lbVector_Diff(&mut c0_p1, &mut c0_p0, &mut pos);
                    pos.x *= frac;
                    pos.y *= frac;
                    pos.z *= frac;
                    ext::lbVector_Add(&mut pos, &mut c0_p0);
                    rot.x = -ext::atan2f(c0_tangent.y, c0_tangent.x);
                    rot.y = 1.5707964;
                    rot.z = 0.0;

                    if (*re).x4 > 0.966 {
                        (*re).flags.set_b2_5(2);
                        (*re).x14 = 0.0;
                        (*re).x1C = 0.0;
                        (*re).x20 = fmadds(5.2359877, ext::HSD_Randf(), -2.6179938);
                        (*re).x24 = (*re).x4;
                    }
                }
                1 => {
                    let t: f32;
                    let angle: f32;

                    if (*re).x1C < (*re).x20 {
                        (*re).x1C += 0.017453292;
                        if (*re).x1C > (*re).x20 {
                            (*re).x1C = (*re).x20;
                        }
                    } else if (*re).x1C > (*re).x20 {
                        (*re).x1C -= 0.017453292;
                        if (*re).x1C < (*re).x20 {
                            (*re).x1C = (*re).x20;
                        }
                    }

                    t = (*re).x14;
                    angle = (*re).x1C;
                    c1_up = grBb_Route_803B83E0;
                    ext::splGetSplinePoint(&mut pos, (*gp).u.car.xCC, t);
                    ext::lbShadow_8000E9F0(&mut c1_tangent, (*gp).u.car.xCC, t);
                    grBigBlueRoute_8020DD64(&mut c1_tangent);
                    ext::PSVECCrossProduct(&mut c1_up, &mut c1_tangent, &mut c1_side);
                    ext::lbVector_RotateAboutUnitAxis(&mut c1_side, &mut c1_tangent, angle);
                    grBigBlueRoute_8020DD64(&mut c1_side);
                    ext::PSVECCrossProduct(&mut c1_tangent, &mut c1_side, &mut c1_up);
                    grBigBlueRoute_8020DD64(&mut c1_up);
                    ext::Ground_801C5AEC(&mut c1_orient, &mut c1_tangent, &mut c1_side, &mut c1_up);
                    rot = c1_orient;
                    {
                        let s = 45.0f32 * ext::Ground_801C0498();
                        c1_up.x *= s;
                        c1_up.y *= s;
                        c1_up.z *= s;
                    }
                    ext::lbVector_Add(&mut pos, &mut c1_up);

                    if (*re).x14 > 0.967 {
                        (*re).flags.set_b2_5(3);
                        (*re).x4 = 0.0;
                        (*re).xC = 0.5;
                        (*re).x10 = ext::HSD_Randf();
                        (*re).x24 = (*re).x14;
                        (*re).flags.set_b6(0);
                        if (*re).flags.b1() != 0 {
                            (*(addr_of_mut!((*gp).u.bigblue) as *mut UnkFlagStruct)).anon_0.set_b1(0);
                        }
                    }
                }
                2 => {
                    let t: f32;
                    let prog: f32;
                    let frac: f32;
                    let angle: f32;

                    t = (*re).x4;
                    prog = (t - (*re).x24) / (1.0 - (*re).x24);
                    frac = fmadds(1.0 - prog, (*re).xC - 0.5, 0.5);
                    ext::splGetSplinePoint(&mut c2_p0, (*gp).u.car.xD0, t);
                    ext::splGetSplinePoint(&mut c2_p1, (*gp).u.car.xD4, t);
                    ext::lbShadow_8000E9F0(&mut c2_road_tan, (*gp).u.car.xD0, t);
                    ext::lbVector_Diff(&mut c2_p1, &mut c2_p0, &mut c2_road);
                    c2_road.x *= frac;
                    c2_road.y *= frac;
                    c2_road.z *= frac;
                    ext::lbVector_Add(&mut c2_road, &mut c2_p0);
                    c2_road_rot.x = -ext::atan2f(c2_road_tan.y, c2_road_tan.x);
                    c2_road_rot.y = 1.5707964;
                    c2_road_rot.z = 0.0;

                    let t = (*re).x14;
                    angle = (*re).x1C;
                    c2_up = grBb_Route_803B83E0;
                    ext::splGetSplinePoint(&mut c2_air, (*gp).u.car.xCC, t);
                    ext::lbShadow_8000E9F0(&mut c2_tangent, (*gp).u.car.xCC, t);
                    grBigBlueRoute_8020DD64(&mut c2_tangent);
                    ext::PSVECCrossProduct(&mut c2_up, &mut c2_tangent, &mut c2_side);
                    ext::lbVector_RotateAboutUnitAxis(&mut c2_side, &mut c2_tangent, angle);
                    grBigBlueRoute_8020DD64(&mut c2_side);
                    ext::PSVECCrossProduct(&mut c2_tangent, &mut c2_side, &mut c2_up);
                    grBigBlueRoute_8020DD64(&mut c2_up);
                    ext::Ground_801C5AEC(&mut c2_orient, &mut c2_tangent, &mut c2_side, &mut c2_up);
                    {
                        let s = 45.0f32 * ext::Ground_801C0498();
                        c2_up.x *= s;
                        c2_up.y *= s;
                        c2_up.z *= s;
                    }
                    ext::lbVector_Add(&mut c2_air, &mut c2_up);
                    ext::lbVector_Diff(&mut c2_air, &mut c2_road, &mut pos);
                    pos.x *= prog;
                    pos.y *= prog;
                    pos.z *= prog;
                    ext::lbVector_Add(&mut pos, &mut c2_road);
                    rot = c2_road_rot;

                    if (*re).x4 == 1.0 {
                        (*re).flags.set_b2_5(1);
                    }
                }
                3 => {
                    let t: f32;
                    let prog: f32;
                    let angle: f32;
                    let road_t: f32;
                    let frac: f32;

                    t = (*re).x14;
                    prog = (t - (*re).x24) / (1.0 - (*re).x24);
                    angle = (1.0 - prog) * (*re).x1C;
                    c3_up = grBb_Route_803B83E0;
                    ext::splGetSplinePoint(&mut c3_air, (*gp).u.car.xCC, t);
                    ext::lbShadow_8000E9F0(&mut c3_tangent, (*gp).u.car.xCC, t);
                    grBigBlueRoute_8020DD64(&mut c3_tangent);
                    ext::PSVECCrossProduct(&mut c3_up, &mut c3_tangent, &mut c3_side);
                    ext::lbVector_RotateAboutUnitAxis(&mut c3_side, &mut c3_tangent, angle);
                    grBigBlueRoute_8020DD64(&mut c3_side);
                    ext::PSVECCrossProduct(&mut c3_tangent, &mut c3_side, &mut c3_up);
                    grBigBlueRoute_8020DD64(&mut c3_up);
                    ext::Ground_801C5AEC(&mut c3_orient, &mut c3_tangent, &mut c3_side, &mut c3_up);
                    c3_air_rot = c3_orient;
                    {
                        let s = 45.0f32 * ext::Ground_801C0498();
                        c3_up.x *= s;
                        c3_up.y *= s;
                        c3_up.z *= s;
                    }
                    ext::lbVector_Add(&mut c3_air, &mut c3_up);

                    road_t = (*re).x4;
                    frac = (*re).xC;
                    ext::splGetSplinePoint(&mut c3_p0, (*gp).u.car.xD0, road_t);
                    ext::splGetSplinePoint(&mut c3_p1, (*gp).u.car.xD4, road_t);
                    ext::lbShadow_8000E9F0(&mut c3_road_tan, (*gp).u.car.xD0, road_t);
                    ext::lbVector_Diff(&mut c3_p1, &mut c3_p0, &mut c3_road);
                    c3_road.x *= frac;
                    c3_road.y *= frac;
                    c3_road.z *= frac;
                    ext::lbVector_Add(&mut c3_road, &mut c3_p0);
                    ext::atan2f(c3_road_tan.y, c3_road_tan.x);
                    ext::lbVector_Diff(&mut c3_road, &mut c3_air, &mut pos);
                    pos.x *= prog;
                    pos.y *= prog;
                    pos.z *= prog;
                    ext::lbVector_Add(&mut pos, &mut c3_air);
                    rot = c3_air_rot;

                    if (*re).x14 == 1.0 {
                        (*re).flags.set_b2_5(0);
                    }
                }
                _ => {}
            }

            if (*re).flags.b1() == 0 {
                HSD_JObjSetTranslate(jobj, &pos);
                HSD_JObjSetRotationX(jobj, rot.x);
                HSD_JObjSetRotationY(jobj, rot.y);
                HSD_JObjSetRotationZ(jobj, rot.z);
            }

            let flag_view = addr_of_mut!((*gp).u.bigblue) as *mut UnkFlagStruct;
            if pos.x - 260.0 < fighter_pos.x
                && fighter_pos.x < 260.0 + pos.x
                && pos.z - 260.0 < fighter_pos.z
                && fighter_pos.z < 260.0 + pos.z
                && (*re).flags.b1() == 0
            {
                if (*flag_view).anon_0.b1() == 0 {
                    (*flag_view).anon_0.set_b1(1);
                    ext::Ground_801C53EC(0x77A16);
                }
            }

            if pos.x - 100.0 < fighter_pos.x
                && fighter_pos.x < 100.0 + pos.x
                && pos.z - 100.0 < fighter_pos.z
                && fighter_pos.z < 100.0 + pos.z
            {
                if (*re).flags.b1() != 0 {
                    if (*flag_view).anon_0.b0() == 0 {
                        ext::un_802FD604((*yakumono_param).x4C as s32);
                        ext::Ground_801C53EC(0x77A11);
                        (*flag_view).anon_0.set_b0(1);
                    }
                } else {
                    ext::Camera_RequestQuake(QuakeKind_Loop, null_mut());
                    if (*re).flags.b6() == 0 {
                        (*re).flags.set_b6(1);
                        if ext::HSD_Randi(100) < 40 {
                            match ext::HSD_Randi(3) {
                                0 => ext::Ground_801C53EC(0x77A15),
                                1 => ext::Ground_801C53EC(0x77A14),
                                2 => ext::Ground_801C53EC(0x77A13),
                                _ => {}
                            }
                        }
                    }
                }
            } else if (*re).flags.b1() != 0 {
                if (*flag_view).anon_0.b0() != 0 {
                    (*flag_view).anon_0.set_b0(0);
                }
            }
        }

        i += 1;
        // the C walks `jobj = (jobj != NULL) ? (HSD_JObj*) grBigBlueRoute_8020DA9C((struct *) jobj) : NULL`, i.e. the
        // `next` pointer read through an `int` view at +8
        jobj = if !jobj.is_null() { (*jobj).next } else { null_mut() };
        if i >= 31 {
            break;
        }
    }
}

pub unsafe fn grBigBlueRoute_8020DA9C(desc: *mut grBigBlueRoute_8020DA9C_t) -> i32 {
    melee_core::ct!("grBigBlueRoute_8020DA9C");
    if desc.is_null() {
        return 0;
    }
    (*desc).x8
}

pub unsafe fn grBigBlueRoute_8020DAB4(jobjs: *mut *mut HSD_JObj, scale: f32, count: i32) {
    melee_core::ct!("grBigBlueRoute_8020DAB4");
    let mut countdown: i32 = 30;

    if jobjs.is_null() {
        return;
    }

    let gobj = grBigBlueRoute_8020B9D4(4);
    assert_(!gobj.is_null(), 1414, "gobj");
    {
        let gp = GET_GROUND(gobj);
        assert_(!gp.is_null(), 1415, "gp");
    }

    let mut arr = jobjs;
    let mut i = 0;
    while i < count {
        if (*arr).is_null() {
            break;
        }

        let mut random_idx = if countdown != 0 { ext::HSD_Randi(countdown) } else { 0 };

        let mut jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        assert_(!jobj.is_null(), 1434, "jobj");

        jobj = HSD_JObjGetChild(jobj);

        while random_idx != 0 {
            assert_(!jobj.is_null(), 1437, "jobj");
            random_idx -= 1;
            jobj = HSD_JObjGetNext(jobj);
        }

        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

        HSD_JObjSetScaleX(jobj, scale);
        HSD_JObjSetScaleY(jobj, scale);
        HSD_JObjSetScaleZ(jobj, scale);

        ext::HSD_JObjReparent(jobj, *arr);

        countdown -= 1;
        arr = arr.add(1);
        i += 1;
    }

    ext::Ground_801C4A08(gobj);
}

pub unsafe fn grBigBlueRoute_8020DD64(v: *mut Vec3) -> f32 {
    melee_core::ct!("grBigBlueRoute_8020DD64");
    let mag = ext::lbVector_Normalize(v);
    let x = (*v).x;

    if x > 1.0 {
        (*v).x = 1.0;
        (*v).z = 0.0;
        (*v).y = 0.0;
    } else if x < -1.0 {
        (*v).x = -1.0;
        (*v).z = 0.0;
        (*v).y = 0.0;
    } else if (*v).y > 1.0 {
        (*v).y = 1.0;
        (*v).z = 0.0;
        (*v).x = 0.0;
    } else if (*v).y < -1.0 {
        (*v).y = -1.0;
        (*v).z = 0.0;
        (*v).x = 0.0;
    } else if (*v).z > 1.0 {
        (*v).z = 1.0;
        (*v).y = 0.0;
        (*v).x = 0.0;
    } else if (*v).z < -1.0 {
        (*v).z = -1.0;
        (*v).y = 0.0;
        (*v).x = 0.0;
    }

    mag
}

pub unsafe fn grBigBlueRoute_8020DE48() -> *mut DynamicModelDesc {
    melee_core::ct!("grBigBlueRoute_8020DE48");
    let archive = ext::grDatFiles_801C6330(0x2);
    assert_(!archive.is_null(), 1495, "archive");
    let dat = (*archive).unk4;
    if !dat.is_null() {
        // the C adds the GameCube byte offset 0x68 (= 2 * sizeof(UnkStageDat_x8_t) there): element 2
        return (*dat).unk8.add(2) as *mut DynamicModelDesc;
    }
    null_mut()
}

pub unsafe extern "C" fn fn_8020DEAC() {
    melee_core::ct!("fn_8020DEAC");
    ext::Ground_801C53EC(0x77A12);
}

/// Clamp camera position to stage bounds.
pub unsafe extern "C" fn grBigBlueRoute_8020DED4(pos: *mut Vec3) {
    melee_core::ct!("grBigBlueRoute_8020DED4");
    let mut x = (*pos).x;
    let mut y = (*pos).y;

    if x < -1140.0f32 * ext::Ground_801C0498() {
        x = -1140.0f32 * ext::Ground_801C0498();
    }

    if y < 20.0f32 * ext::Ground_801C0498() {
        y = 20.0f32 * ext::Ground_801C0498();
    } else if y > 250.0f32 * ext::Ground_801C0498() {
        y = 250.0f32 * ext::Ground_801C0498();
    }

    ext::Ground_801C38BC(x, y);
}

pub unsafe extern "C" fn grBigBlueRoute_8020DF80(a: *mut Vec3, _arg: i32, joint: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grBigBlueRoute_8020DF80");
    let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    ext::lb_8000B1CC(joint, null_mut(), &mut b);

    if (*a).y > b.y + -3.0f32 {
        1
    } else {
        0
    }
}
