//! src/melee/gr/grcastle.c: Peach's Castle stage and the "grCastle" scenery shared by Pure Palace / Shrine /
//! Garden / Mute City / Big Blue / Onett (pipes, satellite, bumper, wind platforms).
//!
//! Fused sites (tools/rust-port/fma_sites.py), written out: `fmadds` for the platform speeds
//! `wind * x20 + x24` / `wind * x2C + x30` (`801CDC44`; the dx*dx + dy*dy distance test of `801CFBD4` is a
//! square of computed expressions, which the console does not fuse: notes/fp-audit.md); a true f64 `mul_add` for
//! `0.3 * HSD_Randf() + 0.2` (`801D0BBC`, `801D0D24`).
//!
//! Native adaptations (same result as the PPC build): the game keeps pointers in `u32` fields
//! (`gp->u.castle11.xD8 = (u32) camera`): [`ptr_u32`] / [`u32_ptr`] (lossless below 4 GiB, which the oracle tests
//! guarantee). Union views that alias on the PPC but not in the native layout are replaced by one canonical `u32`
//! view per logical field: the satellite gobjs live in `castle12.xC4[3]` / `xD0` / `xD2` (the C writes `castle2`),
//! the satellite's follow target in `castle11.xD0` / `xD4` / `xD8` (the C uses `castle7`/`castle5` pointers), and the
//! item callbacks (`801D0550`..`801D08AC`) reach `castle10.state` / `x10C` instead of the fake `unkCastle` struct.
//! State-oracle checked against the C (tests/gro_grcastle.rs, gen/oracle/gr_oracle.py; the spec lists the C patches).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
use melee_core::fp::Fmadd as _;
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use super::grinl::*;
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_core::fp::fmadds;
use melee_types::*;

/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::{stage_info, HSD_JObjSetMtxDirty, HSD_JObjSetMtxDirtySub, HSD_JObjSetupMatrixSub};
    hook!(CAMERA_80029044, Camera_80029044, (a0: i32) -> *mut CmSubject);
    hook!(CAMERA_800290D4, Camera_800290D4, (a0: *mut CmSubject));
    hook_or!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, (a0: CmQuakeKind, a1: *mut Vec), |a0: CmQuakeKind, a1: *mut Vec| unsafe { crate::tu::camera::Camera_RequestQuake(a0 as _, a1 as _) });
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
    pub static mut GROUND_801C53EC: Option<fn(u32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C53EC(a0: u32) {
        match unsafe { GROUND_801C53EC } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C53EC(a0 as _); } return; } }
    }
    pub static mut GROUND_801C5440: Option<fn(*mut Ground, s32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5440(gp: *mut Ground, i: s32, a2: u32) {
        match unsafe { GROUND_801C5440 } { Some(f) => f(gp, i, a2), None => { unsafe { crate::tu::ground::Ground_801C5440(gp as _, i as _, a2 as _); } return; } }
    }
    pub static mut GROUND_801C5544: Option<fn(*mut Ground, s32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5544(a0: *mut Ground, a1: s32) {
        match unsafe { GROUND_801C5544 } { Some(f) => f(a0, a1), None => { unsafe { crate::tu::ground::Ground_801C5544(a0 as _, a1 as _); } return; } }
    }
    pub static mut GROUND_801C5694: Option<fn(*mut Ground, s32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5694(gp: *mut Ground, i: s32, val: f32) {
        match unsafe { GROUND_801C5694 } { Some(f) => f(gp, i, val), None => { unsafe { crate::tu::ground::Ground_801C5694(gp as _, i as _, val as _); } return; } }
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
    pub static mut GROUND_INITMAPCOLL: Option<fn(*mut HSD_JObj, s32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_InitMapColl(jobj: *mut HSD_JObj, map_id: s32) -> i32 {
        match unsafe { GROUND_INITMAPCOLL } { Some(f) => f(jobj, map_id), None => { return unsafe { crate::tu::ground::Ground_InitMapColl(jobj as _, map_id as _) as _ }; } }
    }
    pub static mut GROUND_UPDATEMAPCOLL: Option<fn(*mut HSD_GObj) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 {
        match unsafe { GROUND_UPDATEMAPCOLL } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ }; } }
    }
    pub static mut HSD_ARCHIVEGETPUBLICADDRESS: Option<fn(*mut HSD_Archive, *const i8) -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void {
        match unsafe { HSD_ARCHIVEGETPUBLICADDRESS } { Some(f) => f(a0, a1), None => { return unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0 as _, a1 as _) as _ }; } }
    }
    hook_or!(HSD_GOBJ_SETUPPROC, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc, |a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ });
    hook_or!(HSD_JOBJCLEARFLAGS, HSD_JObjClearFlags, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlags(a0 as _, flags as _); } });
    hook_or!(HSD_JOBJCLEARFLAGSALL, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, flags as _); } });
    hook_or!(HSD_JOBJGETFLAGS, HSD_JObjGetFlags, (jobj: *mut HSD_JObj) -> u32, |jobj: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjGetFlags(jobj as _) as _ });
    hook_or!(HSD_JOBJSETFLAGS, HSD_JObjSetFlags, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlags(a0 as _, flags as _); } });
    hook_or!(HSD_JOBJSETFLAGSALL, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, flags as _); } });
    hook_or!(HSD_RANDF, HSD_Randf, () -> f32, || melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand()));
    hook_or!(HSD_RANDI, HSD_Randi, (max_val: s32) -> s32, |max_val: s32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), max_val));
    hook_or!(OSREPORT, OSReport, (a0: *const i8, va: &[usize]), |a0: *const i8, va: &[usize]| unsafe { melee_runtime::tu::os_error::OSReport(a0, va) });
    hook_or!(PSVECCROSSPRODUCT, PSVECCrossProduct, (vec1: *mut Vec, vec2: *mut Vec, dst: *mut Vec), |vec1: *mut Vec, vec2: *mut Vec, dst: *mut Vec| { unsafe { melee_runtime::tu::mtx_pc::raw::PSVECCrossProduct(vec1 as _, vec2 as _, dst as _); } });
    hook_or!(ATAN2F, atan2f, (y: f32, x: f32) -> f32, melee_runtime::lbtrigf::atan2f);
    hook_or!(FTCO_800C06E8, ftCo_800C06E8, (a0: *mut HSD_GObj, a1: i32, a2: *mut c_void), |a0: *mut HSD_GObj, a1: i32, a2: *mut c_void| { unsafe { crate::ftdevice::ftCo_800C06E8(a0 as _, a1 as _, core::mem::transmute(a2)); } });
    hook_or!(FTLIB_80086644, ftLib_80086644, (a0: *mut HSD_GObj, a1: *mut Vec), |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_80086644(a0 as _, a1 as _); } });
    hook_or!(FTLIB_80086684, ftLib_80086684, (a0: *mut HSD_GObj, a1: *mut Vec), |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_80086684(a0 as _, a1 as _); } });
    hook_or!(FTLIB_80086960, ftLib_80086960, (gobj: *mut HSD_GObj) -> i32, |gobj: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086960(gobj as _) as _ });
    hook_or!(FTLIB_80086A4C, ftLib_80086A4C, (a0: *mut HSD_GObj, a1: f32), |a0: *mut HSD_GObj, a1: f32| { unsafe { let _ = crate::ftlib::ftLib_80086A4C(a0 as _, a1 as _); } });
    hook_or!(FTLIB_80086B80, ftLib_80086B80, (a0: *mut HSD_GObj) -> f32, |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086B80(a0 as _) as _ });
    hook_or!(FTLIB_80086BEC, ftLib_80086BEC, (a0: *mut HSD_GObj, a1: *mut Vec), |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_80086BEC(a0 as _, a1 as _); } });
    hook_or!(GM_8016AE80, gm_8016AE80, () -> i32, || unsafe { crate::gm::gm_8016AE80() as _ });
    hook_or!(GM_8016B238, gm_8016B238, () -> i32, || unsafe { crate::gm::gm_8016B238() as _ });
    pub static mut GRANIME_801C8138: Option<fn(*mut HSD_GObj, enum_t, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C8138(a0: *mut HSD_GObj, a1: enum_t, a2: i32) {
        match unsafe { GRANIME_801C8138 } { Some(f) => f(a0, a1, a2), None => { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRANIME_801C83D0: Option<fn(*mut HSD_GObj, i32, enum_t) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: enum_t) -> i32 {
        match unsafe { GRANIME_801C83D0 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRANIME_801C86D4: Option<fn(s32, *mut HSD_GObj, s32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C86D4(a0: s32, a1: *mut HSD_GObj, a2: s32) {
        match unsafe { GRANIME_801C86D4 } { Some(f) => f(a0, a1, a2), None => { unsafe { crate::tu::granime::grAnime_801C86D4(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRDATFILES_GETARCHIVE: Option<fn() -> *mut UnkArchiveStruct> = None;
    #[allow(non_snake_case)]
    pub fn grDatFiles_GetArchive() -> *mut UnkArchiveStruct {
        match unsafe { GRDATFILES_GETARCHIVE } { Some(f) => f(), None => { return unsafe { crate::tu::grdatfiles::grDatFiles_GetArchive() as _ }; } }
    }
    pub static mut GRLIB_801C96F8: Option<fn(s32, s32, *mut Vec) -> *mut HSD_Generator> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C96F8(a0: s32, a1: s32, a2: *mut Vec) -> *mut HSD_Generator {
        match unsafe { GRLIB_801C96F8 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::grlib::grLib_801C96F8(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRLIB_801C9B20: Option<fn(*mut HSD_JObj, *mut DynamicsDesc, *mut DynamicsDesc)> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C9B20(a0: *mut HSD_JObj, a1: *mut DynamicsDesc, a2: *mut DynamicsDesc) {
        match unsafe { GRLIB_801C9B20 } { Some(f) => f(a0, a1, a2), None => { unsafe { crate::tu::grlib::grLib_801C9B20(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRLIB_801C9B6C: Option<fn(*mut c_void)> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C9B6C(a0: *mut c_void) {
        match unsafe { GRLIB_801C9B6C } { Some(f) => f(a0), None => { unsafe { crate::tu::grlib::grLib_801C9B6C(a0 as _); } return; } }
    }
    pub static mut GRLIB_801C9B8C: Option<fn(*mut c_void)> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C9B8C(a0: *mut c_void) {
        match unsafe { GRLIB_801C9B8C } { Some(f) => f(a0), None => { unsafe { crate::tu::grlib::grLib_801C9B8C(a0 as _); } return; } }
    }
    pub static mut GRMATERIAL_801C8CDC: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8CDC(a0: *mut HSD_GObj) {
        match unsafe { GRMATERIAL_801C8CDC } { Some(f) => f(a0), None => { unsafe { crate::tu::grmaterial::grMaterial_801C8CDC(a0 as _); } return; } }
    }
    pub static mut GRMATERIAL_801C8CFC: Option<fn(i32, i32, *mut Ground, *mut HSD_JObj, Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8CFC(a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, arg4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, arg5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, arg6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj {
        match unsafe { GRMATERIAL_801C8CFC } { Some(f) => f(a0, a1, a2, a3, arg4, arg5, arg6), None => { return unsafe { crate::tu::grmaterial::grMaterial_801C8CFC(a0 as _, a1 as _, a2 as _, a3 as _, arg4 as _, arg5 as _, arg6 as _) as _ }; } }
    }
    pub static mut GRMATERIAL_801C8DE0: Option<fn(*mut HSD_GObj, f32, f32, f32, f32, f32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8DE0(gobj: *mut HSD_GObj, arg8: f32, arg9: f32, argA: f32, argB: f32, argC: f32, argD: f32, argE: f32) {
        match unsafe { GRMATERIAL_801C8DE0 } { Some(f) => f(gobj, arg8, arg9, argA, argB, argC, argD, argE), None => { unsafe { crate::tu::grmaterial::grMaterial_801C8DE0(gobj as _, arg8 as _, arg9 as _, argA as _, argB as _, argC as _, argD as _, argE as _); } return; } }
    }
    pub static mut GRMATERIAL_801C8E08: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8E08(a0: *mut HSD_GObj) {
        match unsafe { GRMATERIAL_801C8E08 } { Some(f) => f(a0), None => { unsafe { crate::tu::grmaterial::grMaterial_801C8E08(a0 as _); } return; } }
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
    pub static mut GRZAKOGENERATOR_801CAE04: Option<fn(*mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj {
        match unsafe { GRZAKOGENERATOR_801CAE04 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ }; } }
    }
    hook_or!(ITGETKIND, itGetKind, (gobj: *mut HSD_GObj) -> ItemKind, |gobj: *mut HSD_GObj| unsafe { crate::itacc::itGetKind(gobj as _) as _ });
    hook!(IT_8026F7C8, it_8026F7C8, (a0: *mut Vec, a1: *mut Vec, a2: i32) -> *mut HSD_GObj);
    hook_or!(IT_80275414, it_80275414, (a0: *mut HSD_GObj), |a0: *mut HSD_GObj| { unsafe { let _ = crate::itacc::it_80275414(a0 as _); } });
    pub static mut LBVECTOR_ANGLE: Option<fn(*mut Vec, *mut Vec) -> f32> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Angle(a: *mut Vec, b: *mut Vec) -> f32 {
        match unsafe { LBVECTOR_ANGLE } { Some(f) => f(a, b), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Angle(a as _, b as _) as _ }; } }
    }
    pub static mut LBVECTOR_NORMALIZE: Option<fn(*mut Vec) -> f32> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Normalize(vec: *mut Vec) -> f32 {
        match unsafe { LBVECTOR_NORMALIZE } { Some(f) => f(vec), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(vec as _) as _ }; } }
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec, *mut Vec)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec, pos1: *mut Vec) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    pub static mut LB_800103B8: Option<fn(*mut Vec, *mut Vec) -> f32> = None;
    #[allow(non_snake_case)]
    pub fn lb_800103B8(a0: *mut Vec, a1: *mut Vec) -> f32 {
        match unsafe { LB_800103B8 } { Some(f) => f(a0, a1), None => { return unsafe { melee_runtime::tu::lb_00F9::lb_800103B8(a0 as _, a1 as _) as _ }; } }
    }
    pub static mut LB_800115F4: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn lb_800115F4() {
        match unsafe { LB_800115F4 } { Some(f) => f(), None => { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } return; } }
    }
    pub static mut LB_800117F4: Option<fn(*mut DynamicsDesc, *mut _GXColor, *mut _GXColor, i32, u32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn lb_800117F4(a0: *mut DynamicsDesc, a1: *mut _GXColor, a2: *mut _GXColor, a3: i32, a4: u32) -> i32 {
        match unsafe { LB_800117F4 } { Some(f) => f(a0, a1, a2, a3, a4), None => { return unsafe { melee_runtime::tu::lb_00F9::lb_800117F4(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _) as _ }; } }
    }
    pub static mut LB_800119DC: Option<fn(*mut Vec, i32, f32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn lb_800119DC(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32) {
        match unsafe { LB_800119DC } { Some(f) => f(a0, a1, a2, a3, a4), None => { unsafe { melee_runtime::tu::lb_00F9::lb_800119DC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } return; } }
    }
    pub static mut LB_80011A50: Option<fn(*mut Vec, i32, f32, f32, f32, f32, f32, f32, f32) -> *mut lb_80011A50_t> = None;
    #[allow(non_snake_case)]
    pub fn lb_80011A50(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32) -> *mut lb_80011A50_t {
        match unsafe { LB_80011A50 } { Some(f) => f(a0, a1, a2, a3, a4, a5, a6, a7, a8), None => { return unsafe { melee_runtime::tu::lb_00F9::lb_80011A50(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _, a8 as _) as _ }; } }
    }
    pub static mut MPJOINTLISTADD: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpJointListAdd(joint_id: i32) {
        match unsafe { MPJOINTLISTADD } { Some(f) => f(joint_id), None => { unsafe { crate::tu::mplib::mpJointListAdd(joint_id as _); } return; } }
    }
    pub static mut MPJOINTSETCB1: Option<fn(i32, *mut c_void, mpLib_JointCollisionCallback)> = None;
    #[allow(non_snake_case)]
    pub fn mpJointSetCb1(joint_id: i32, user_data: *mut c_void, cb: mpLib_JointCollisionCallback) {
        match unsafe { MPJOINTSETCB1 } { Some(f) => f(joint_id, user_data, cb), None => { unsafe { crate::tu::mplib::mpJointSetCb1(joint_id as _, user_data as _, cb as _); } return; } }
    }
    pub static mut MPLIB_80057424: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_80057424(joint_id: i32) {
        match unsafe { MPLIB_80057424 } { Some(f) => f(joint_id), None => { unsafe { crate::tu::mplib::mpLib_80057424(joint_id as _); } return; } }
    }
    pub static mut MPLIB_80057BC0: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_80057BC0(joint_id: i32) {
        match unsafe { MPLIB_80057BC0 } { Some(f) => f(joint_id), None => { unsafe { crate::tu::mplib::mpLib_80057BC0(joint_id as _); } return; } }
    }
    /// The head of the ground gobj link (`HSD_GObjPLinkHead[HSD_GOBJ_PLINK_GROUND]`, a global of gobj.c).
    hook_or!(HSD_GOBJPLINKHEAD, HSD_GObjPLinkHead, (link: i32) -> *mut HSD_GObj, |link: i32| unsafe { *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(link as usize) });
    /// `grDisplay_801C5DB0` is only ever passed as a render callback (its address is the argument).
    pub static mut GRDISPLAY_801C5DB0: Option<fn(*mut HSD_GObj, i32)> = None;
    /// Called by `grCastle_801D0520` and passed as a render callback (its address is the argument).
    pub unsafe extern "C" fn grDisplay_801C5DB0(gobj: *mut HSD_GObj, pass: i32) {
        match GRDISPLAY_801C5DB0 {
            Some(f) => f(gobj, pass),
            None => { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(gobj as _, pass as _); } return; },
        }
    }
}

/// `struct grCastleParams_Entry`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grCastleParams_Entry {
    pub x0: i16,
    pub pad_x2: [u8; 2],
    pub x4: f32,
    pub rot: Vec3,
}

/// `struct grCastle_YakumonoParam`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grCastle_YakumonoParam {
    pub x0: i16,
    pub x2: i16,
    pub x4: i16,
    pub x6: i16,
    pub x8: i16,
    pub xA: i16,
    pub xC: i16,
    pub xE: i16,
    pub x10: f32,
    pub x14: f32,
    pub x18: f32,
    pub pad_x1C: [u8; 4],
    pub x20: f32,
    pub x24: f32,
    pub x28: f32,
    pub x2C: f32,
    pub x30: f32,
    pub x34: f32,
    pub x38: f32,
    pub x3C: f32,
    pub x40: i16,
    pub x42: i16,
    pub x44: i16,
    pub pad_x46: [u8; 2],
    pub x48: f32,
    pub x4C: f32,
    pub x50: f32,
    pub x54: i16,
    pub pad_x56: [u8; 2],
    pub x58: i16,
    pub pad_x5A: [u8; 2],
    pub entries: [grCastleParams_Entry; 9],
    pub x110: f32,
    pub x114: *mut c_void,
    pub x118: f32,
    pub x11C: f32,
    pub x120: f32,
    pub x124: f32,
    pub pad_x128: [u8; 4],
    pub x12C: [i16; 4],
    pub x134: f32,
    pub x138: f32,
    pub x13C: f32,
    pub x140: f32,
}

/// `static struct grCastle_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grCastle_YakumonoParam = null_mut();
/// `static struct lb_80011A50_t* grCs_804D6974`.
pub static mut grCs_804D6974: *mut lb_80011A50_t = null_mut();
/// `static f32 grCs_804D45E0` / `static u8 grCs_804D45E4`.
pub static mut grCs_804D45E0: f32 = 0.017453292;
pub static mut grCs_804D45E4: u8 = 1;

pub static mut grCs_803E0FE8: [GrJoint; 2] = [GrJoint { x: 4, y: 6, z: 1 }, GrJoint { x: 5, y: 6, z: 4 }];

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

/// `StageCallbacks grCs_StageCallbacks[21]`.
pub static mut grCs_StageCallbacks: [StageCallbacks; 21] = [
    cbs!(),
    cbs!(grCastle_801CE8E8, grCastle_801CE9E0, grCastle_801CE9E8, grCastle_801CEAC8, 0),
    cbs!(grCastle_801CE7E8, grCastle_801CE858, grCastle_801CE860, grCastle_801CE8E4, 0),
    cbs!(grCastle_801CD658, grCastle_801CD8A0, grCastle_801CD8A8, grCastle_801CD960, 0xC0000000),
    cbs!(grCastle_801CD5BC, grCastle_801CD600, grCastle_801CD608, grCastle_801CD60C, 0),
    cbs!(grCastle_801CEACC, grCastle_801CEEFC, grCastle_801CEF04, grCastle_801CF0F0, 0),
    cbs!(grCastle_801CDA0C, grCastle_801CDC3C, grCastle_801CDC44, grCastle_801CDF50, 0),
    cbs!(grCastle_801CEACC, grCastle_801CEEFC, grCastle_801CEF04, grCastle_801CF0F0, 0),
    cbs!(grCastle_801CE260, grCastle_801CE3A4, grCastle_801CE578, grCastle_801CE7E4, 0),
    cbs!(grCastle_801CE260, grCastle_801CE3A4, grCastle_801CE578, grCastle_801CE7E4, 0),
    cbs!(grCastle_801CE260, grCastle_801CE3A4, grCastle_801CE578, grCastle_801CE7E4, 0),
    cbs!(grCastle_801CE260, grCastle_801CE3A4, grCastle_801CE578, grCastle_801CE7E4, 0),
    cbs!(grCastle_801CE260, grCastle_801CE3A4, grCastle_801CE578, grCastle_801CE7E4, 0),
    cbs!(grCastle_801CE260, grCastle_801CE3A4, grCastle_801CE578, grCastle_801CE7E4, 0),
    cbs!(grCastle_801CE260, grCastle_801CE3A4, grCastle_801CE578, grCastle_801CE7E4, 0),
    cbs!(grCastle_801CE260, grCastle_801CE3A4, grCastle_801CE578, grCastle_801CE7E4, 0),
    cbs!(grCastle_801CE260, grCastle_801CE3A4, grCastle_801CE578, grCastle_801CE7E4, 0),
    cbs!(grCastle_801CEACC, grCastle_801CEEFC, grCastle_801CEF04, grCastle_801CF0F0, 0),
    cbs!(grCastle_801CF0F4, grCastle_801CF300, grCastle_801CF308, grCastle_801CF74C, 0),
    cbs!(grCastle_801CF0F4, grCastle_801CF300, grCastle_801CF308, grCastle_801CF74C, 0),
    cbs!(grCastle_801CF0F4, grCastle_801CF300, grCastle_801CF308, grCastle_801CF74C, 0),
];

/// `char grCs_803E1198[] = "/GrCs.dat"`.
pub static mut grCs_803E1198: [u8; 10] = *b"/GrCs.dat\0";

/// `StageData grCs_StageData`.
pub static mut grCs_StageData: StageData = StageData {
    grkind: Gr_Kind_Castle,
    callbacks: core::ptr::addr_of!(grCs_StageCallbacks) as *mut StageCallbacks,
    data1: core::ptr::addr_of!(grCs_803E1198) as *mut i8,
    on_init: Some(grCastle_801CD37C),
    on_demo_init: Some(grCastle_801CD338),
    on_load: Some(grCastle_801CD4A0),
    on_start: Some(grCastle_801CD4A4),
    callback4: Some(grCastle_801CD4C8),
    on_touch_line: Some(grCastle_801D0B04),
    on_check_shadow_render: Some(grCastle_801D0B0C),
    flags2: 1 << 0,
    joints: core::ptr::addr_of!(grCs_803E0FE8) as *mut GrJoint,
    joint_count: 2,
};

/// `static const Vec3 grCs_803B7E9C`.
pub static grCs_803B7E9C: Vec3 = Vec3 { x: -257.0, y: 13.5, z: -252.0 };
/// `static const grCastle_DynEntries grCs_803B7EA8`: `{ depth, type }` x 4.
pub static grCs_803B7EA8: [[i16; 2]; 4] = [[66, 6], [76, 6], [85, 6], [94, 6]];
/// `static const Quaternion grCs_803B7EB8`.
pub static grCs_803B7EB8: Quaternion = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
/// `static const grCastle_BlinkTable grCs_803B7EC8` (read as bytes by `grCastle_801CF308`).
pub static grCs_803B7EC8: [i16; 19] = [
    0x201, 0x201, 0x201, 0x201, 0x201, 0x201, 0x101, 0x101, 0x101, 0x101, 0x101, 0x101, 0x101, 0x101, 0x101, 0x102, 0x102, 0x102, 0x1ff,
];
/// `static const grCastle_WeightTable grCs_803B7EF0`.
pub static grCs_803B7EF0: [i32; 3] = [100, 100, 100];
/// `static const grCastle_TargetTable grCs_803B7EFC`: `{ map_id, jobj_idx }` x 11.
pub static grCs_803B7EFC: [[i16; 2]; 11] = [[3, 16], [3, 17], [3, 14], [3, 15], [3, 18], [3, 19], [3, 21], [3, 22], [3, 20], [6, 2], [6, 5]];
/// `static const grCastle_CallbackTable grCs_803B7F28` (damage-received callbacks of the yaku items).
pub static grCs_803B7F28: [unsafe extern "C" fn(*mut c_void, *mut Ground); 5] =
    [grCastle_801D0550, grCastle_801D059C, grCastle_801D05E8, grCastle_801D0634, grCastle_801D0680];
/// `static const grCastle_CallbackTable2 grCs_803B7F3C` (touched callbacks).
pub static grCs_803B7F3C: [unsafe extern "C" fn(*mut c_void, *mut Ground, *mut Ground_GObj); 5] =
    [grCastle_801D06CC, grCastle_801D0744, grCastle_801D07BC, grCastle_801D0834, grCastle_801D08AC];
/// `static const grCastle_YOffsets grCs_803B7F50`.
pub static grCs_803B7F50: [f32; 6] = [4.0, 6.0, 7.0, 6.0, 4.0, -1.0];

#[inline(always)]
unsafe fn yp() -> &'static grCastle_YakumonoParam {
    &*yakumono_param
}

pub unsafe extern "C" fn grCastle_801CD338(_arg0: i32) {
    melee_core::ct!("grCastle_801CD338");
    let gobj = ext::Ground_GetMapGObj(6);
    if !gobj.is_null() {
        let jobj = ext::Ground_801C3FA4(gobj, 4);
        if !jobj.is_null() {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    }
}

pub unsafe extern "C" fn grCastle_801CD37C() {
    melee_core::ct!("grCastle_801CD37C");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grCastle_YakumonoParam;

    let si = ext::stage_info();
    (*si).unk8C.set_b4(0);
    (*si).unk8C.set_b5(1);

    grCastle_801CD4D0(0);
    grCastle_801CD4D0(4);
    grCastle_801CD4D0(3);
    grCastle_801CD4D0(6);

    for j in [0, 1, 2, 6, 7, 8, 9, 10, 11, 12, 13, 14] {
        ext::mpLib_80057BC0(j);
    }

    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    let mut vec = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    grCs_804D6974 = ext::lb_80011A50(&mut vec, -1, 0.5, 0.0, 1.0471976, -10000.0, 10000.0, 10000.0, -10000.0);
}

pub unsafe extern "C" fn grCastle_801CD4A0() { melee_core::ct!("grCastle_801CD4A0");}

pub unsafe extern "C" fn grCastle_801CD4A4() {
    melee_core::ct!("grCastle_801CD4A4");
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe extern "C" fn grCastle_801CD4C8() -> i32 {
    melee_core::ct!("grCastle_801CD4C8");
    0
}

pub unsafe fn grCastle_801CD4D0(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grCastle_801CD4D0");
    let callbacks: *mut StageCallbacks = &mut grCs_StageCallbacks[gobj_id as usize];
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
            &[b"grcastle.c\0".as_ptr() as usize, 297, gobj_id as u32 as usize],
        );
    }
    gobj
}

pub unsafe extern "C" fn grCastle_801CD5BC(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CD5BC");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grCastle_801CD600(_gobj: *mut Ground_GObj) -> i32 {
    melee_core::ct!("grCastle_801CD600");
    0
}
pub unsafe extern "C" fn grCastle_801CD608(_gobj: *mut Ground_GObj) { melee_core::ct!("grCastle_801CD608");}
pub unsafe extern "C" fn grCastle_801CD60C(_gobj: *mut Ground_GObj) { melee_core::ct!("grCastle_801CD60C");}

pub unsafe extern "C" fn grCastle_801CD610(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CD610");
    ext::ftCo_800C06E8(gobj, 6, grCastle_801D09B8 as *mut c_void);
    grCastle_801CF7B0(gobj);
    grCastle_801CDFD8(gobj);
}

pub unsafe extern "C" fn grCastle_801CD658(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CD658");
    let gp = GET_GROUND(gobj);

    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);

    let mut pos = grCs_803B7E9C;
    {
        let scale = ext::Ground_801C0498();
        pos.x *= scale;
        pos.y *= scale;
        pos.z *= scale;
    }
    ext::grLib_801C96F8(0x7536, 0x1E, &mut pos);

    for i in 0..12usize {
        (*gp).u.castle9.dynamics[i].data = null_mut();
    }

    let archive = ext::grDatFiles_GetArchive();
    if !archive.is_null() {
        let flag3 = ext::HSD_ArchiveGetPublicAddress((*archive).unk0, b"dynamicsdata_flag3\0".as_ptr() as *const i8) as *mut DynamicsDesc;
        if !flag3.is_null() {
            let flag4 = ext::HSD_ArchiveGetPublicAddress((*archive).unk0, b"dynamicsdata_flag4\0".as_ptr() as *const i8) as *mut DynamicsDesc;
            if !flag4.is_null() {
                let flag6 = ext::HSD_ArchiveGetPublicAddress((*archive).unk0, b"dynamicsdata_flag6\0".as_ptr() as *const i8) as *mut DynamicsDesc;
                if !flag6.is_null() {
                    let entries_s = grCs_803B7EA8;

                    for i in 0..4usize {
                        let jobj = ext::Ground_801C3FA4(gobj, entries_s[i][0] as i32);
                        if !jobj.is_null() {
                            if entries_s[i][1] == 3 {
                                ext::grLib_801C9B20(jobj, flag3, &mut (*gp).u.castle9.dynamics[i]);
                            } else if entries_s[i][1] == 4 {
                                ext::grLib_801C9B20(jobj, flag4, &mut (*gp).u.castle9.dynamics[i]);
                            } else if entries_s[i][1] == 6 {
                                ext::grLib_801C9B20(jobj, flag6, &mut (*gp).u.castle9.dynamics[i]);
                            } else {
                                (*gp).u.castle9.dynamics[i].data = null_mut();
                            }
                        } else {
                            (*gp).u.castle9.dynamics[i].data = null_mut();
                        }
                    }
                }
            }
        }
    }

    (*gobj).render_cb = Some(fn_801D0924);
    ext::Ground_801C10B8(gobj, Some(grCastle_801CD610));
    (*gp).u.castle9.xC4 = 0;
    (*gp).u.castle9.xC8 = 0;
    (*gp).u.castle9.xCC = 0;
    (*gp).u.castle9.set_xDE_b0(0);
    (*gp).x10_flags.set_b5(1);
}

pub unsafe extern "C" fn grCastle_801CD8A0(_gobj: *mut Ground_GObj) -> i32 {
    melee_core::ct!("grCastle_801CD8A0");
    0
}

pub unsafe extern "C" fn grCastle_801CD8A8(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CD8A8");
    let gp = GET_GROUND(gobj);

    grCastle_801CF868(gobj);
    grCastle_801CE19C(gobj);
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
    grCastle_801D0BBC();
    for i in 0..12usize {
        if !(*gp).u.castle3.x1C[i].data.is_null() {
            grCastle_801D0D84((*(*(*gp).u.castle3.x1C[i].data).desc.lb_unk0.jobj).parent);
        }
    }
    for i in 0..12usize {
        if !(*gp).u.castle3.x1C[i].data.is_null() {
            ext::grLib_801C9B8C(&mut (*gp).u.castle3.x1C[i] as *mut DynamicsDesc as *mut c_void);
            grCastle_801D0D24();
        }
    }
}

pub unsafe extern "C" fn grCastle_801CD960(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CD960");
    let gp = (*gobj).user_data as *mut Ground;
    for i in 0..12usize {
        ext::grLib_801C9B6C(&mut (*gp).u.castle3.x1C[i] as *mut DynamicsDesc as *mut c_void);
    }
}

pub unsafe extern "C" fn grCastle_801CD9B4(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CD9B4");
    let gp = GET_GROUND(gobj);
    ext::mpJointSetCb1(4, gp as *mut c_void, Some(grCastle_801CF750));
    ext::mpJointSetCb1(5, gp as *mut c_void, Some(grCastle_801CF750));
}

pub unsafe extern "C" fn grCastle_801CDA0C(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CDA0C");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::Ground_InitMapColl(jobj, (*gp).map_id as s32);
    (*gp).x10_flags.set_b5(1);

    (*gp).u.castle8.plat[0].jobj = ext::Ground_801C3FA4(gobj, 1);
    let mut val = 0.0f32;
    if yp().x18 != val {
        val = yp().x14;
    } else {
        val = yp().x10;
    }
    (*gp).u.castle8.plat[0].pos = val;
    (*gp).u.castle8.plat[0].state = 0;

    HSD_JObjSetTranslateY((*gp).u.castle8.plat[0].jobj, (*gp).u.castle8.plat[0].pos);
    ext::mpLib_80057424(4);

    (*gp).u.castle8.plat[0].timer = melee_core::ppc::fctiwz_f32(yp().x38) as i16;
    (*gp).u.castle8.plat[0].wind = 0.0;

    (*gp).u.castle8.plat[1].jobj = ext::Ground_801C3FA4(gobj, 4);
    val = 0.0;
    if yp().x18 != val {
        val = yp().x14;
    } else {
        val = yp().x10;
    }
    (*gp).u.castle8.plat[1].pos = val;
    (*gp).u.castle8.plat[1].state = 0;

    HSD_JObjSetTranslateY((*gp).u.castle8.plat[1].jobj, (*gp).u.castle8.plat[1].pos);
    ext::mpLib_80057424(5);

    (*gp).u.castle8.plat[1].timer = melee_core::ppc::fctiwz_f32(yp().x38) as i16;
    (*gp).u.castle8.plat[1].wind = 0.0;

    ext::Ground_801C10B8(gobj, Some(grCastle_801CD9B4));
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grCastle_801CDC3C(_gobj: *mut Ground_GObj) -> i32 {
    melee_core::ct!("grCastle_801CDC3C");
    0
}

pub unsafe extern "C" fn grCastle_801CDC44(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CDC44");
    let gp = GET_GROUND(gobj);
    let mut i = 0;

    // the C advances `gp` by sizeof(grCastle_Platform) bytes and reads `plat[0]` again: platform `i`
    loop {
        let pl = &mut (*gp).u.castle8.plat[i];
        match pl.state {
            0 => {
                if pl.wind > 0.0 {
                    let cnt = pl.timer;
                    pl.timer = cnt.wrapping_add(1);
                    if cnt as f32 > yp().x38 {
                        pl.state = 2;
                    }
                } else {
                    pl.timer = 0;
                    pl.state = 1;
                }
            }
            1 => {
                if pl.wind > 0.0 {
                    pl.timer = 0;
                    pl.state = 0;
                } else {
                    let cnt = pl.timer;
                    pl.timer = cnt.wrapping_add(1);
                    if cnt as f32 > yp().x3C {
                        pl.state = 3;
                    }
                }
            }
            2 => {
                let wind = pl.wind;
                if wind > 0.0 {
                    let mut move_speed = fmadds(wind, yp().x20, yp().x24);
                    if move_speed > yp().x28 {
                        move_speed = yp().x28;
                    }
                    let max_val = yp().x10;
                    let cur = pl.pos;
                    if (max_val - cur) < move_speed {
                        pl.pos = max_val;
                    } else {
                        pl.pos = cur + move_speed;
                    }
                    HSD_JObjSetTranslateY(pl.jobj, pl.pos);
                } else {
                    pl.timer = 0;
                    pl.state = 1;
                }
            }
            3 => {
                let wind = pl.wind;
                if wind > 0.0 {
                    pl.timer = 0;
                    pl.state = 0;
                } else {
                    let mut move_speed = fmadds(wind, yp().x2C, yp().x30);
                    if move_speed > yp().x34 {
                        move_speed = yp().x34;
                    }
                    if (pl.pos - yp().x14) < move_speed {
                        pl.pos = yp().x14;
                    } else {
                        pl.pos -= move_speed;
                    }
                    HSD_JObjSetTranslateY(pl.jobj, pl.pos);
                }
            }
            _ => {}
        }
        i += 1;
        pl.wind = 0.0;
        if i >= 2 {
            break;
        }
    }
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grCastle_801CDF50(_gobj: *mut Ground_GObj) { melee_core::ct!("grCastle_801CDF50");}

pub unsafe fn grCastle_801CDF54(vec: *mut Vec3) -> i32 {
    melee_core::ct!("grCastle_801CDF54");
    let mut i = 0;
    if (*ext::stage_info()).grkind == Gr_Kind_Castle {
        while i < 9 {
            let gobj = ext::Ground_GetMapGObj(i + 8);
            if !gobj.is_null() {
                let jobj = ext::Ground_801C3FA4(gobj, 0);
                ext::lb_8000B1CC(jobj, null_mut(), vec);
                return 1;
            }
            i += 1;
        }
    }
    0
}

pub unsafe fn grCastle_801CDFD8(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CDFD8");
    let gp = GET_GROUND(gobj);

    (*gp).u.castle9.set_xDE_b0(1);

    let range = yp().xA as i32;
    let rand_result = if range != 0 { ext::HSD_Randi(range) } else { 0 };

    (*gp).u.castle9.xD4 = (yp().x8 as i32 + rand_result) as i16;
    (*gp).u.castle9.xDC = -1;
    (*gp).u.castle9.xDA = -1;
    (*gp).u.castle9.xD8 = -1;
    (*gp).u.castle9.xD6 = 0;
}

pub unsafe fn grCastle_801CE054(gobj: *mut Ground_GObj) -> s32 {
    melee_core::ct!("grCastle_801CE054");
    let mut arr = [0i16; 9];
    let mut count: s32 = 0;
    let mut val: s32 = 0;
    let mut ptr = 0usize;
    let gp = (*gobj).user_data as *mut Ground;
    for _ctr in 0..3 {
        for _i in 0..3 {
            if val != (*gp).u.castle4.xD8 as s32 && val != (*gp).u.castle4.xDA as s32 && val != (*gp).u.castle4.xDC as s32 {
                arr[ptr] = val as i16;
                ptr += 1;
                count += 1;
            }
            val += 1;
        }
    }

    let idx = if count != 0 { ext::HSD_Randi(count) } else { 0 };

    let picked = arr[idx as usize] as u32;
    // (&gp->u.castle4.xD8)[xD6] = picked: the three shorts xD8, xDA, xDC
    let base = addr_of_mut!((*gp).u.castle4.xD8);
    *base.offset((*gp).u.castle4.xD6 as isize) = picked as i16;

    let mut d6 = (*gp).u.castle4.xD6 as s32;
    if 2 == d6 {
        d6 = 0;
    } else {
        d6 += 1;
    }
    (*gp).u.castle4.xD6 = d6 as i16;

    picked as s32
}

pub unsafe fn grCastle_801CE19C(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CE19C");
    let gp = GET_GROUND(gobj);
    if (*gp).u.castle9.xDE_b0() != 0 {
        let timer = (*gp).u.castle9.xD4;
        (*gp).u.castle9.xD4 = timer.wrapping_sub(1);
        if timer < 0 {
            let new_gobj = grCastle_801CD4D0(grCastle_801CE054(gobj) + 8);
            (*gp).u.castle9.set_xDE_b0(0);
            if !new_gobj.is_null() {
                let new_gp = (*new_gobj).user_data as *mut Ground;
                ext::Ground_801C5440(gp, 0, 0x53021);
                ext::Ground_801C5694(gp, 0, yp().entries[(*new_gp).u.castle5.xC6 as usize].x4);
                (*new_gp).u.castle11.xD4 = ptr_u32(gobj);
            }
        }
    }
}

pub unsafe extern "C" fn grCastle_801CE260(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CE260");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::Ground_InitMapColl(jobj, (*gp).map_id as s32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);

    (*gp).u.castle11.xC4.set_b0(0);
    (*gp).u.icemt.x2 = ((*gp).map_id as i32 - 8) as i16;
    (*gp).u.arwing.xCC = 0;
    (*gp).u.flatzone.xCA = yp().entries[(*gp).u.icemt.x2 as usize].x0;

    let gp2 = GET_GROUND(gobj);
    (*gp2).u.arwing.xD8 = ptr_u32(ext::Camera_80029044(2)) as i32;
    let subject: *mut CmSubject = u32_ptr((*gp2).u.arwing.xD8 as u32);
    if !subject.is_null() {
        (*subject).target_ext.h.x = yp().x118;
        (*subject).target_ext.h.y = yp().x11C;
        (*subject).target_ext.v.x = yp().x120;
        (*subject).target_ext.v.y = yp().x124;
    }

    ext::grMaterial_801C94D8(jobj as *mut c_void);
    (*gp).u.arwing.xD0 = ptr_u32(ext::grMaterial_801C8CFC(
        0,
        3,
        gp,
        ext::Ground_801C3FA4(gobj, 0),
        Some(core::mem::transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>(fn_801CE3A0)),
        None,
        None,
    ));
    ext::it_80275414(u32_ptr((*gp).u.arwing.xD0));
    ext::Ground_801C5440(gp, 0, 0x53025);
}

pub unsafe extern "C" fn fn_801CE3A0() { melee_core::ct!("fn_801CE3A0");}

pub unsafe extern "C" fn grCastle_801CE3A4(_gobj: *mut Ground_GObj) -> i32 {
    melee_core::ct!("grCastle_801CE3A4");
    0
}

pub unsafe fn grCastle_801CE3AC_dontinline(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CE3AC_dontinline");
    grCastle_801CE3AC(gobj);
}

pub unsafe fn grCastle_801CE3AC(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CE3AC");
    let gp = GET_GROUND(gobj);

    // castle5.xCC (the C reads it as a pointer): the canonical u32 slot is castle11.xCC
    if (*gp).u.castle11.xCC != 0 {
        let jobj = (*u32_ptr::<HSD_GObj>((*gp).u.castle11.xCC)).hsd_obj as *mut HSD_JObj;
        let mut rot = grCs_803B7EB8;
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

        offset.z = 0.0;
        offset.x = 0.0;
        offset.y = yp().x110;

        ext::lb_8000B1CC(ext::Ground_801C3FA4(gobj, 1), &mut offset, &mut pos);

        HSD_JObjSetTranslate(jobj, &pos);

        let e = &yp().entries[(*gp).u.castle5.xC6 as usize];
        rot.x = e.rot.x;
        rot.y = e.rot.y;
        rot.z = e.rot.z;

        HSD_JObjSetRotation(jobj, &rot);
    }
}

pub unsafe extern "C" fn grCastle_801CE578(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CE578");
    let mut gp = (*gobj).user_data as *mut Ground;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut jpos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    {
        let cam: *mut CmSubject = u32_ptr((*gp).u.castle11.xD8);
        if !cam.is_null() {
            ext::lb_8000B1CC(ext::Ground_801C3FA4(gobj, 0), null_mut(), &mut pos);
            (*cam).pos = pos;
            (*cam).bone_pos = pos;
        }
    }

    {
        let gp2 = (*gobj).user_data as *mut Ground;
        if (*gp2).u.castle11.xCC == 0 {
            let timer = (*gp2).u.castle11.xCA;
            (*gp2).u.castle11.xCA = timer.wrapping_sub(1);
            if timer < 0 {
                (*gp2).u.castle11.xCC = ptr_u32(grCastle_801CD4D0(2));
                ext::Ground_801C53EC(0x53026);
                grCastle_801CE3AC_dontinline(gobj);
            }
        }
    }

    if (*gp).u.castle11.xC4.b0() == 0 {
        gp = (*gobj).user_data as *mut Ground;
        if ext::grAnime_801C83D0(gobj, 0, 1) != 0 {
            (*gp).u.castle11.xC8 = yp().x58;
            (*gp).u.castle11.xC4.set_b0(1);
            ext::grMaterial_801C9604(gobj, yp().x114, 0);
            if (*gp).u.castle11.xCC != 0 {
                ext::Ground_801C4A08(u32_ptr((*gp).u.castle11.xCC));
            }
            ext::Ground_801C5544(gp, 0);
        }
    } else {
        gp = (*gobj).user_data as *mut Ground;
        let timer = (*gp).u.castle11.xC8;
        (*gp).u.castle11.xC8 = timer.wrapping_sub(1);
        if timer < 0 {
            if (*gp).u.castle11.xD0 != 0 {
                ext::grMaterial_801C8CDC(u32_ptr((*gp).u.castle11.xD0));
            }
            (*gp).u.castle11.xD0 = 0;

            {
                let newobj = grCastle_801CD4D0(1);
                if !newobj.is_null() {
                    let jobj = (*newobj).hsd_obj as *mut HSD_JObj;
                    ext::lb_8000B1CC(ext::Ground_801C3FA4(gobj, 0), null_mut(), &mut jpos);
                    HSD_JObjSetTranslateWithMtxDirty(jobj, &jpos);
                    ext::lb_800119DC(&mut jpos, 0xB4, 20.0, 0.1, 1.0471976);
                }
            }

            {
                let sat = (*u32_ptr::<HSD_GObj>((*gp).u.castle11.xD4)).user_data as *mut Ground;

                (*sat).u.castle9.set_xDE_b0(1);

                let range = yp().xE as s32;
                let rand = if range != 0 { ext::HSD_Randi(range) } else { 0 };
                (*sat).u.castle9.xD4 = (yp().xC as s32 + rand) as i16;
            }

            gp = (*gobj).user_data as *mut Ground;
            if (*gp).u.castle11.xD8 != 0 {
                ext::Camera_800290D4(u32_ptr((*gp).u.castle11.xD8));
                (*gp).u.castle11.xD8 = 0;
            }
            ext::Ground_801C4A08(gobj);
        }
    }
}

pub unsafe extern "C" fn grCastle_801CE7E4(_gobj: *mut Ground_GObj) { melee_core::ct!("grCastle_801CE7E4");}

pub unsafe extern "C" fn grCastle_801CE7E8(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CE7E8");
    let gp = GET_GROUND(gobj);
    ext::Ground_InitMapColl(GET_JOBJ(gobj), (*gp).map_id as s32);
    (*gp).u.castle.xC4 = 0;
    (*gp).u.castle.xC8 = yp().x12C[(*gp).u.castle.xC4 as usize];
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, (*gp).u.castle.xC4 as i32);
}

pub unsafe extern "C" fn grCastle_801CE858(_gobj: *mut Ground_GObj) -> i32 {
    melee_core::ct!("grCastle_801CE858");
    0
}

pub unsafe extern "C" fn grCastle_801CE860(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CE860");
    let gp = (*gobj).user_data as *mut Ground;

    if ((*gp).u.castle.xC4 as s32) < 3 {
        let timer = (*gp).u.castle.xC8;
        (*gp).u.castle.xC8 = timer.wrapping_sub(1);
        if timer < 0 {
            (*gp).u.castle.xC4 += 1;
            ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, (*gp).u.castle.xC4 as i32);
            (*gp).u.castle.xC8 = yp().x12C[(*gp).u.castle.xC4 as usize];
        }
    }
    ext::Camera_RequestQuake(QuakeKind_Loop, null_mut());
}

pub unsafe extern "C" fn grCastle_801CE8E4(_gobj: *mut Ground_GObj) { melee_core::ct!("grCastle_801CE8E4");}

pub unsafe extern "C" fn grCastle_801CE8E8(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CE8E8");
    let gp = GET_GROUND(gobj);

    ext::Ground_InitMapColl((*gobj).hsd_obj as *mut HSD_JObj, (*gp).map_id as s32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);

    let gp2 = GET_GROUND(gobj);
    (*gp2).u.arwing.xC8 = ptr_u32(ext::Camera_80029044(2));
    let subject: *mut CmSubject = u32_ptr((*gp2).u.arwing.xC8);
    if !subject.is_null() {
        (*subject).target_ext.h.x = yp().x134;
        (*subject).target_ext.h.y = yp().x138;
        (*subject).target_ext.v.x = yp().x13C;
        (*subject).target_ext.v.y = yp().x140;
    }
    (*gp).u.arwing.xC4 = ptr_u32(ext::grMaterial_801C8CFC(
        0,
        4,
        gp,
        ext::Ground_801C3FA4(gobj, 0),
        Some(core::mem::transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>(fn_801CE9DC)),
        None,
        None,
    ));
    ext::it_80275414(u32_ptr((*gp).u.arwing.xC4));
    ext::Ground_801C5440(gp, 0, 0x53024);
}

pub unsafe extern "C" fn fn_801CE9DC() { melee_core::ct!("fn_801CE9DC");}

pub unsafe extern "C" fn grCastle_801CE9E0(_gobj: *mut Ground_GObj) -> i32 {
    melee_core::ct!("grCastle_801CE9E0");
    0
}

pub unsafe extern "C" fn grCastle_801CE9E8(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CE9E8");
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = GET_GROUND(gobj);
    let subject: *mut CmSubject = u32_ptr((*gp).u.arwing.xC8);
    if !subject.is_null() {
        ext::lb_8000B1CC(ext::Ground_801C3FA4(gobj, 0), null_mut(), &mut pos);
        (*subject).pos = pos;
        (*subject).bone_pos = pos;
    }
    if ext::grAnime_801C83D0(gobj, 0, 1) != 0 {
        let mat: *mut HSD_GObj = u32_ptr((*gp).u.arwing.xC4);
        if !mat.is_null() {
            ext::grMaterial_801C8CDC(mat);
        }
        (*gp).u.arwing.xC4 = 0;
        {
            let gp2 = (*gobj).user_data as *mut Ground;
            let subj2: *mut CmSubject = u32_ptr((*gp2).u.arwing.xC8);
            if !subj2.is_null() {
                ext::Camera_800290D4(subj2);
                (*gp2).u.arwing.xC8 = 0;
            }
        }
        ext::Ground_801C4A08(gobj);
    }
}

pub unsafe extern "C" fn grCastle_801CEAC8(_gobj: *mut Ground_GObj) { melee_core::ct!("grCastle_801CEAC8");}

/// `static inline zero`.
pub unsafe fn zero(gp: *mut Ground, i: usize, zero: f32) {
    melee_core::ct!("zero");
    (*gp).u.castle10.jobjs[i] = null_mut();
    (*gp).u.castle10.effect_a[i] = null_mut();
    (*gp).u.castle10.effect_b[i] = null_mut();
    (*gp).u.castle10.x10C[i] = 0;
    (*gp).u.castle10.x120[i] = -1;
    (*gp).u.castle10.state[i] = 0;
    (*gp).u.castle10.idx[i] = 0;
    (*gp).u.castle10.baseY[i] = zero;
}

pub unsafe extern "C" fn grCastle_801CEACC(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CEACC");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::Ground_InitMapColl(jobj, (*gp).map_id as s32);
    (*gp).x10_flags.set_b5(1);
    (*gp).u.castle10.xC4 = 0;
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*gp).u.castle10.xC8 = 0;

    for i in 0..5usize {
        zero(gp, i, 0.0);
    }

    let c10 = &mut (*gp).u.castle10;
    match (*gp).map_id {
        Gr_Kind_Pura => {
            c10.jobjs[0] = ext::Ground_801C3FA4(gobj, 1);
            c10.effect_a[0] = ext::Ground_801C3FA4(gobj, 2);
            c10.effect_b[0] = ext::Ground_801C3FA4(gobj, 3);
            c10.baseY[0] = HSD_JObjGetTranslationY(c10.jobjs[0]);
            c10.x120[0] = 0xB;

            c10.jobjs[1] = ext::Ground_801C3FA4(gobj, 4);
            c10.effect_a[1] = ext::Ground_801C3FA4(gobj, 5);
            c10.effect_b[1] = ext::Ground_801C3FA4(gobj, 6);
            c10.baseY[1] = HSD_JObjGetTranslationY(c10.jobjs[1]);
            c10.x120[1] = 0xC;

            c10.jobjs[2] = ext::Ground_801C3FA4(gobj, 7);
            c10.effect_a[2] = null_mut();
            c10.effect_b[2] = null_mut();
            c10.x120[2] = 0xD;

            c10.jobjs[3] = ext::Ground_801C3FA4(gobj, 8);
            c10.effect_a[3] = null_mut();
            c10.effect_b[3] = null_mut();
            c10.x120[3] = 0xE;
        }
        Gr_Kind_Shrine => {
            c10.jobjs[0] = ext::Ground_801C3FA4(gobj, 1);
            c10.effect_a[0] = ext::Ground_801C3FA4(gobj, 2);
            c10.effect_b[0] = ext::Ground_801C3FA4(gobj, 3);
            c10.baseY[0] = HSD_JObjGetTranslationY(c10.jobjs[0]);
            c10.x120[0] = 6;

            c10.jobjs[1] = ext::Ground_801C3FA4(gobj, 4);
            c10.effect_a[1] = ext::Ground_801C3FA4(gobj, 5);
            c10.effect_b[1] = ext::Ground_801C3FA4(gobj, 6);
            c10.baseY[1] = HSD_JObjGetTranslationY(c10.jobjs[1]);
            c10.x120[1] = 7;

            ext::HSD_JObjSetFlagsAll(ext::Ground_801C3FA4(gobj, 7), JOBJ_HIDDEN as u32);
            c10.jobjs[2] = null_mut();
            c10.effect_a[2] = null_mut();
            c10.effect_b[2] = null_mut();
            ext::mpLib_80057BC0(8);

            c10.jobjs[3] = ext::Ground_801C3FA4(gobj, 0xA);
            c10.effect_a[3] = null_mut();
            c10.effect_b[3] = null_mut();
            c10.x120[3] = 9;

            c10.jobjs[4] = ext::Ground_801C3FA4(gobj, 0xB);
            c10.effect_a[4] = null_mut();
            c10.effect_b[4] = null_mut();
            c10.x120[4] = 0xA;
        }
        Gr_Kind_Garden => {
            c10.jobjs[0] = ext::Ground_801C3FA4(gobj, 1);
            c10.effect_a[0] = ext::Ground_801C3FA4(gobj, 2);
            c10.effect_b[0] = ext::Ground_801C3FA4(gobj, 3);
            c10.baseY[0] = HSD_JObjGetTranslationY(c10.jobjs[0]);
            c10.x120[0] = 0;

            c10.jobjs[1] = ext::Ground_801C3FA4(gobj, 4);
            c10.effect_a[1] = null_mut();
            c10.effect_b[1] = null_mut();
            c10.x120[1] = 1;

            c10.jobjs[2] = ext::Ground_801C3FA4(gobj, 5);
            c10.effect_a[2] = null_mut();
            c10.effect_b[2] = null_mut();
            c10.x120[2] = 2;
        }
        _ => {}
    }

    grCastle_801D0298(gobj, 0);
    ext::Ground_UpdateMapColl(gobj);
    (*gobj).render_cb = Some(grCastle_801D0520);
}

pub unsafe extern "C" fn grCastle_801CEEFC(_gobj: *mut Ground_GObj) -> i32 {
    melee_core::ct!("grCastle_801CEEFC");
    0
}

pub unsafe extern "C" fn grCastle_801CEF04(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CEF04");
    let gp = GET_GROUND(gobj);

    match (*gp).u.castle6.xC4 {
        0 => {}
        1 => {
            (*gp).u.castle6.xC8 = 0;
            let mut hi = yp().x42 as i32;
            let lo = yp().x40 as i32;

            if hi > lo {
                let range = hi - lo;
                hi = lo + if range != 0 { ext::HSD_Randi(range) } else { 0 };
            } else if hi < lo {
                let range = lo - hi;
                hi += if range != 0 { ext::HSD_Randi(range) } else { 0 };
            }
            (*gp).u.castle6.xCC = hi;
            if grCastle_801D0298(gobj, 1) != 0 {
                (*gp).u.castle6.xC4 = 2;
            }
            grCastle_801D02B8(gobj);
        }
        2 => {
            let temp = (*gp).u.castle6.xCC;
            (*gp).u.castle6.xCC = temp.wrapping_sub(1);
            if temp < 0 {
                (*gp).u.castle6.xC4 = 3;
                (*gp).u.castle6.xCC = 0;
                for i in 0..5usize {
                    if (*gp).u.castle10.x10C[i] != 0 {
                        ext::grMaterial_801C8CDC(u32_ptr((*gp).u.castle10.x10C[i]));
                        (*gp).u.castle10.x10C[i] = 0;
                    }
                }
            }
            grCastle_801D02B8(gobj);
        }
        3 => {
            let temp = (*gp).u.castle6.xCC;
            (*gp).u.castle6.xCC = temp.wrapping_add(1);
            if temp > yp().x44 as i32 {
                (*gp).u.castle6.xC4 = 4;
            } else {
                let val: s32 = if (*gp).u.castle6.xC8 != 0 { 0 } else { 1 };
                (*gp).u.castle6.xC8 = val as i16;
            }
            grCastle_801D02B8(gobj);
        }
        4 => {
            let val: s32 = if (*gp).u.castle6.xC8 != 0 { 0 } else { 1 };
            (*gp).u.castle6.xC8 = val as i16;
            if grCastle_801D0298(gobj, 0) != 0 {
                (*gp).u.castle6.xC8 = 0;
                (*gp).u.castle6.xC4 = 0;
            }
        }
        _ => {}
    }
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grCastle_801CF0F0(_gobj: *mut Ground_GObj) { melee_core::ct!("grCastle_801CF0F0");}

pub unsafe extern "C" fn grCastle_801CF0F4(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CF0F4");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::Ground_InitMapColl(jobj, (*gp).map_id as s32);
    (*gp).u.castle7.xC4 = 0;
    // castle7.xD8 / xD0 are the u32 slots castle11.xD8 / xD0
    (*gp).u.castle11.xD8 = 0;
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    let scale = 1.5f32 * ext::Ground_801C0498();
    HSD_JObjSetScaleX(jobj, scale);
    HSD_JObjSetScaleY(jobj, scale);
    HSD_JObjSetScaleZ(jobj, scale);

    match (*gp).map_id {
        Gr_Kind_Onett => {
            (*gp).u.castle11.xD0 = ptr_u32(grCastle_801CD4D0(0x11));
        }
        Gr_Kind_BigBlue => {
            (*gp).u.castle11.xD0 = ptr_u32(grCastle_801CD4D0(7));
        }
        Gr_Kind_MuteCity => {
            (*gp).u.castle11.xD0 = ptr_u32(grCastle_801CD4D0(5));
        }
        _ => {}
    }
}

pub unsafe extern "C" fn grCastle_801CF300(_gobj: *mut Ground_GObj) -> i32 {
    melee_core::ct!("grCastle_801CF300");
    0
}

pub unsafe extern "C" fn grCastle_801CF308(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CF308");
    let mut var_r6: s32 = 0;
    let user_data = (*gobj).user_data as *mut Ground;
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let tbl: [i16; 19] = grCs_803B7EC8;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut quat = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };

    let mut state = (*user_data).u.castle5.xC4;
    let mut fall = false;
    if state == 1 {
        (*gp).u.castle.xC8 = -1;
        (*gp).u.castle11.xCA = 0;
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        ext::grAnime_801C86D4((*gp).map_id as s32, gobj, 0);
        (*gp).u.castle5.xC4 = 2;
        state = 2;
        fall = true;
    }
    let _ = fall;
    if state == 2 {
        let delay = (*gp).u.castle11.xCA as s32;
        if delay > 0 {
            (*gp).u.castle11.xCA = (delay - 1) as i16;
        } else {
            (*gp).u.castle.xC8 = (*gp).u.castle.xC8.wrapping_add(1);
            if ext::HSD_JObjGetFlags(jobj) & 0x10 != 0 {
                ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
            } else {
                ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            }
            // ((s8*) &tbl)[xC8]: the table viewed as bytes (big-endian s16 pairs on the PPC)
            let idx = (*gp).u.castle.xC8 as isize;
            let val: i8 = {
                let word = tbl[(idx >> 1) as usize];
                if idx & 1 == 0 { (word >> 8) as i8 } else { word as i8 }
            };
            if val != -1 {
                (*gp).u.castle11.xCA = melee_core::ppc::fctiwz(2.0 * (val as f64)) as i16;
            } else {
                (*gp).u.castle5.xC4 = 3;
                (*gp).u.castle11.xD8 = ptr_u32(ext::grMaterial_801C8CFC(
                    0,
                    1,
                    gp,
                    jobj,
                    None,
                    Some(core::mem::transmute::<
                        unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj),
                        unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32),
                    >(fn_801CFAFC)),
                    Some(fn_801CFB68),
                ));
                ext::grMaterial_801C8DE0(u32_ptr((*gp).u.castle11.xD8), 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 4.0);
                ext::grMaterial_801C8E08(u32_ptr((*gp).u.castle11.xD8));
                // PPC/x86-32 `plat[0].state` is the s16 at gp+CC (castle11.xCC): the native castle8 view puts it at gp+D0
                (*gp).u.castle11.xCC = ((*gp).u.castle11.xCC & 0xFFFF_0000) | (yp().x54 as u16 as u32);
            }
        }
        var_r6 = 1;
    } else if state == 3 {
        var_r6 = 1;
    } else if state == 4 || state == 5 {
        if state == 4 {
            if (*gp).u.castle11.xD8 != 0 {
                ext::grMaterial_801C8CDC(u32_ptr((*gp).u.castle11.xD8));
                (*gp).u.castle11.xD8 = 0;
            }
            (*((*u32_ptr::<HSD_GObj>((*gp).u.castle11.xD0)).user_data as *mut Ground)).u.castle5.xC4 = 1;
            (*gp).u.castle5.xC4 = 5;
            ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
            ext::Ground_801C5440(gp, 0, 0x53027);
            ext::Camera_RequestQuake(QuakeKind_Small, null_mut());
        }
        if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
            let cnt = (*gp).u.castle11.xCC as u16 as i16;
            (*gp).u.castle11.xCC = ((*gp).u.castle11.xCC & 0xFFFF_0000) | (cnt.wrapping_sub(1) as u16 as u32);
            if cnt < 0 {
                ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            }
        }
        if (*((*u32_ptr::<HSD_GObj>((*gp).u.castle11.xD0)).user_data as *mut Ground)).u.castle5.xC4 == 0 {
            (*gp).u.castle5.xC4 = 0;
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
        var_r6 = 1;
    }

    if var_r6 != 0 {
        ext::lb_8000B1CC(u32_ptr((*gp).u.castle11.xD4), null_mut(), &mut pos);
        HSD_JObjSetTranslate(jobj, &pos);

        HSD_JObjGetRotation(u32_ptr((*gp).u.castle11.xD4), &mut quat);
        HSD_JObjSetRotation(jobj, &quat);
    }
}

pub unsafe extern "C" fn grCastle_801CF74C(_gobj: *mut Ground_GObj) { melee_core::ct!("grCastle_801CF74C");}

/// `mpLib_JointCollisionCallback` of the two wind platforms.
pub unsafe extern "C" fn grCastle_801CF750(user_data: *mut c_void, joint_id: i32, coll: *mut CollData, coll_x50: i32, _ground_kind: mpLib_GroundEnum, _delta_y: f32) {
    melee_core::ct!("grCastle_801CF750");
    let gp = user_data as *mut Ground;
    let idx = if joint_id == 4 { 0 } else { 1 };

    if (*coll).x34_flags.b1234() as s32 == 1 {
        (*gp).u.castle8.plat[idx].wind += coll_x50 as f32;
    }
}

pub unsafe fn grCastle_801CF7B0(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CF7B0");
    let gp = GET_GROUND(gobj);

    // castle2.xC4 / xC8 / xCC / xD0 / xD2 on the PPC are castle12.xC4[0..3] / xD0 / xD2 here
    (*gp).u.castle12.xC4[0] = ptr_u32(grCastle_801CD4D0(20));
    (*gp).u.castle12.xC4[1] = ptr_u32(grCastle_801CD4D0(19));
    (*gp).u.castle12.xC4[2] = ptr_u32(grCastle_801CD4D0(18));
    (*gp).u.castle12.xD0 = -1;

    let x2 = yp().x2 as s32;
    let x0 = yp().x0 as s32;
    (*gp).u.castle12.xD2 = (if x2 > x0 {
        x0 + if x2 - x0 != 0 { ext::HSD_Randi(x2 - x0) } else { 0 }
    } else if x2 < x0 {
        x2 + if x0 - x2 != 0 { ext::HSD_Randi(x0 - x2) } else { 0 }
    } else {
        x2
    }) as i16;
}

/// `static inline grCastle_ResetSatelliteTimer`.
pub unsafe fn grCastle_ResetSatelliteTimer(gp: *mut Ground) {
    melee_core::ct!("grCastle_ResetSatelliteTimer");
    let mut base = yp().x2 as s32;
    let range_end = yp().x0 as s32;

    if base > range_end {
        let diff = base - range_end;
        base = range_end + if diff != 0 { ext::HSD_Randi(diff) } else { 0 };
    } else if base < range_end {
        let diff = range_end - base;
        base += if diff != 0 { ext::HSD_Randi(diff) } else { 0 };
    }
    (*gp).u.castle12.xD2 = base as i16;
}

/// `static inline grCastle_PickSatellite`.
pub unsafe fn grCastle_PickSatellite(gp: *mut Ground, wp: *mut s32) {
    melee_core::ct!("grCastle_PickSatellite");
    let total0 = *wp + *wp.add(1);
    let total = total0 + *wp.add(2);
    let mut rand = if total != 0 { ext::HSD_Randi(total) } else { 0 };

    let mut slot = 0;
    while slot < 3 {
        rand -= *wp.add(slot);
        if rand < 0 {
            break;
        }
        slot += 1;
    }
    if slot == 3 {
        slot = 0;
    }
    (*gp).u.castle12.xD0 = slot as i16;

    let sat_gp = (*u32_ptr::<HSD_GObj>((*gp).u.castle12.xC4[(*gp).u.castle12.xD0 as usize])).user_data as *mut Ground;

    let targets = grCs_803B7EFC;
    let idx = ext::HSD_Randi(11) as usize;
    let mut entity = ext::HSD_GObjPLinkHead(HSD_GOBJ_PLINK_GROUND);
    let want = targets[idx][0] as s32;
    while !entity.is_null() {
        if (*((*entity).user_data as *mut Ground)).map_id as s32 == want {
            break;
        }
        entity = (*entity).next;
    }
    if !entity.is_null() {
        // castle7.xD4 is castle11.xD4 (a u32 slot)
        (*sat_gp).u.castle11.xD4 = ptr_u32(ext::Ground_801C3FA4(entity, targets[idx][1] as s32));
        (*sat_gp).u.castle7.xC4 = 1;
    }
}

pub unsafe extern "C" fn grCastle_801CF868(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801CF868");
    let gp = GET_GROUND(gobj);

    if ((*gp).u.castle12.xC4[0] != 0 || (*gp).u.castle12.xC4[1] != 0 || (*gp).u.castle12.xC4[2] != 0)
        && ((*gp).u.castle12.xD0 == -1 || {
            let sat: *mut HSD_GObj = u32_ptr((*gp).u.castle12.xC4[(*gp).u.castle12.xD0 as usize]);
            !sat.is_null() && !(*sat).user_data.is_null() && *(addr_of_mut!((*((*sat).user_data as *mut Ground)).u.castle2.xC4) as *mut i16) == 0
        })
    {
        (*gp).u.castle12.xD2 = (*gp).u.castle12.xD2.wrapping_sub(1);
        if (*gp).u.castle12.xD2 < 0 {
            grCastle_ResetSatelliteTimer(gp);
            let mut weights = grCs_803B7EF0;
            let cur_slot = (*gp).u.castle12.xD0;
            if cur_slot != -1 {
                weights[cur_slot as usize] /= yp().x6 as s32;
            }

            grCastle_PickSatellite(gp, weights.as_mut_ptr());
        }
    }
}

pub unsafe extern "C" fn fn_801CFAFC(_item: *mut Item_GObj, gp: *mut Ground, _pos: *mut Vec3, gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801CFAFC");
    *(addr_of_mut!((*gp).u.castle2.xC4) as *mut i16) = 4;
    if ext::ftLib_80086960(gobj) != 0 {
        ext::ftLib_80086A4C(gobj, yp().x4 as f32);
    }
}

pub unsafe extern "C" fn fn_801CFB68(_item_gobj: *mut Item_GObj, gp: *mut Ground, gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801CFB68");
    (*gp).u.pura.xC4 = 4;
    if ext::ftLib_80086960(gobj) != 0 {
        ext::ftLib_80086A4C(gobj, yp().x4 as f32);
    }
}

pub unsafe fn grCastle_801CFBD4(gobj: *mut Ground_GObj, arg1: s32) -> s32 {
    melee_core::ct!("grCastle_801CFBD4");
    let mut result: s32 = 1;
    let gp = (*gobj).user_data as *mut Ground;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut target_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    for i in 0..5usize {
        let jobj = (*gp).u.castle10.jobjs[i];
        if !jobj.is_null() {
            let eff_a = (*gp).u.castle10.effect_a[i];
            let eff_b = (*gp).u.castle10.effect_b[i];

            if arg1 != 0 {
                if ext::HSD_JObjGetFlags(jobj) & 0x10 != 0 {
                    let close: s32;

                    if grCastle_801CDF54(&mut target_pos) != 0 && {
                        ext::lb_8000B1CC(jobj, null_mut(), &mut pos);
                        let dx = pos.x - target_pos.x;
                        let dy = pos.y - target_pos.y;
                        // squares of computed expressions: not fused on the console (notes/fp-audit.md)
                        melee_runtime::gekko::sqrtf(dx * dx + dy * dy) < 40.0
                    } {
                        close = 1;
                    } else {
                        close = 0;
                    }

                    if close == 0 {
                        ext::HSD_JObjClearFlags(jobj, JOBJ_HIDDEN as u32);
                        if !eff_a.is_null() && !eff_b.is_null() {
                            if ext::gm_8016AE80() != -1 && ext::gm_8016B238() == 0 {
                                ext::HSD_JObjClearFlags(eff_a, JOBJ_HIDDEN as u32);
                                HSD_JObjSetScaleY(eff_a, 0.0001);
                                ext::HSD_JObjSetFlags(eff_b, JOBJ_HIDDEN as u32);
                                HSD_JObjSetScaleY(eff_b, 1.0);
                            } else {
                                ext::HSD_JObjSetFlags(eff_a, JOBJ_HIDDEN as u32);
                                HSD_JObjSetScaleY(eff_a, 1.0);
                                ext::HSD_JObjClearFlags(eff_b, JOBJ_HIDDEN as u32);
                                HSD_JObjSetScaleY(eff_b, 0.0001);
                            }
                        } else {
                            HSD_JObjSetScaleY(jobj, 0.0001);
                        }
                    }
                    (*gp).u.castle10.state[i] = 0;
                    result = 0;
                    (*gp).u.castle10.idx[i] = 0;
                } else {
                    let target: *mut HSD_JObj;

                    if !eff_a.is_null() && !eff_b.is_null() {
                        if ext::HSD_JObjGetFlags(eff_a) & 0x10 != 0 {
                            target = eff_b;
                        } else {
                            target = eff_a;
                        }
                    } else {
                        target = jobj;
                    }

                    let mut new_scale = HSD_JObjGetScaleY(target);
                    if new_scale < 1.0 {
                        new_scale += yp().x48;
                        if new_scale >= 1.0 {
                            new_scale = 1.0;
                            ext::mpJointListAdd((*gp).u.castle10.x120[i]);
                            if !eff_a.is_null() && ext::HSD_JObjGetFlags(eff_a) & 0x10 == 0 {
                                let cb1 = grCs_803B7F28;
                                let cb2 = grCs_803B7F3C;
                                (*gp).u.castle10.x10C[i] = ptr_u32(ext::grMaterial_801C8CFC(
                                    0,
                                    2,
                                    gp,
                                    target,
                                    None,
                                    Some(core::mem::transmute::<
                                        unsafe extern "C" fn(*mut c_void, *mut Ground),
                                        unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32),
                                    >(cb1[i])),
                                    Some(core::mem::transmute::<
                                        unsafe extern "C" fn(*mut c_void, *mut Ground, *mut Ground_GObj),
                                        unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj),
                                    >(cb2[i])),
                                ));
                                ext::grMaterial_801C8DE0(u32_ptr((*gp).u.castle10.x10C[i]), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 8.0);
                                ext::grMaterial_801C8E08(u32_ptr((*gp).u.castle10.x10C[i]));
                            }
                        }
                        HSD_JObjSetScaleY(target, new_scale);
                        result = 0;
                    }
                }
            } else {
                if ext::HSD_JObjGetFlags(jobj) & 0x10 == 0 {
                    let target: *mut HSD_JObj;

                    if !eff_a.is_null() && !eff_b.is_null() {
                        if ext::HSD_JObjGetFlags(eff_a) & 0x10 != 0 {
                            target = eff_b;
                        } else {
                            target = eff_a;
                        }
                    } else {
                        target = jobj;
                    }

                    let mut new_scale = HSD_JObjGetScaleY(target);
                    if new_scale > 0.0001 {
                        new_scale -= yp().x4C;
                        if new_scale <= 0.0001 {
                            new_scale = 0.0001;
                            ext::HSD_JObjSetFlags(jobj, JOBJ_HIDDEN as u32);
                            if !eff_a.is_null() {
                                ext::HSD_JObjSetFlags(eff_a, JOBJ_HIDDEN as u32);
                            }
                            if !eff_b.is_null() {
                                ext::HSD_JObjSetFlags(eff_b, JOBJ_HIDDEN as u32);
                            }
                        }
                        HSD_JObjSetScaleY(target, new_scale);
                        result = 0;
                    }
                }
                ext::mpLib_80057BC0((*gp).u.castle10.x120[i]);
            }
        }
    }

    result
}

pub unsafe fn grCastle_801D0298(gobj: *mut Ground_GObj, arg1: s32) -> i32 {
    melee_core::ct!("grCastle_801D0298");
    grCastle_801CFBD4(gobj, arg1)
}

pub unsafe fn grCastle_801D02B8(gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801D02B8");
    let gp = GET_GROUND(gobj);

    for i in 0..5usize {
        if (*gp).u.castle10.state[i] == 1 {
            let offsets = grCs_803B7F50;
            let yoff = offsets[(*gp).u.castle10.idx[i] as usize];

            if yoff != -1.0 {
                let new_y = (*gp).u.castle10.baseY[i] + yoff;
                HSD_JObjSetTranslateY((*gp).u.castle10.jobjs[i], new_y);

                if ((*gp).u.castle10.xC4 == 1 || ((*gp).u.castle10.xC4.wrapping_sub(2) as u16) <= 1) && (*gp).u.castle10.idx[i] == 3 {
                    if !(*gp).u.castle10.effect_a[i].is_null() {
                        let mut vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                        ext::lb_8000B1CC((*gp).u.castle10.effect_a[i], null_mut(), &mut pos);
                        pos.y += 5.0;
                        vel.z = 0.0;
                        vel.x = 0.0;
                        vel.y = yp().x50;
                        ext::it_8026F7C8(&mut pos, &mut vel, 1);
                        ext::HSD_JObjSetFlags((*gp).u.castle10.effect_a[i], JOBJ_HIDDEN as u32);
                        ext::HSD_JObjClearFlags((*gp).u.castle10.effect_b[i], JOBJ_HIDDEN as u32);
                    }
                }
            } else {
                HSD_JObjSetTranslateY((*gp).u.castle10.jobjs[i], (*gp).u.castle10.baseY[i]);
                (*gp).u.castle10.state[i] = 0;
            }
            (*gp).u.castle10.idx[i] = (*gp).u.castle10.idx[i].wrapping_add(1);
        }
    }
}

pub unsafe extern "C" fn grCastle_801D0520(gobj: *mut Ground_GObj, renderpass: i32) {
    melee_core::ct!("grCastle_801D0520");
    let gp = GET_GROUND(gobj);
    if (*gp).u.castle.xC8 == 0 {
        ext::grDisplay_801C5DB0(gobj, renderpass);
    }
}

/// `grCastle_801D0550_sub`: the yaku item at slot `i` was destroyed (the C reaches `Ground` through the fake
/// `unkCastle` struct; its `x134` / `x10C` are `castle10.state` / `x10C`).
unsafe fn grCastle_801D0550_sub(gp: *mut Ground, i: usize) {
    (*gp).u.castle10.state[i] = 1;
    ext::grMaterial_801C8CDC(u32_ptr((*gp).u.castle10.x10C[i]));
    (*gp).u.castle10.x10C[i] = 0;
    ext::Ground_801C53EC(0x53020);
}

pub unsafe extern "C" fn grCastle_801D0550(_arg0: *mut c_void, arg1: *mut Ground) {
    melee_core::ct!("grCastle_801D0550");
    grCastle_801D0550_sub(arg1, 0);
}
pub unsafe extern "C" fn grCastle_801D059C(_arg0: *mut c_void, arg1: *mut Ground) {
    melee_core::ct!("grCastle_801D059C");
    grCastle_801D0550_sub(arg1, 1);
}
pub unsafe extern "C" fn grCastle_801D05E8(_arg0: *mut c_void, arg1: *mut Ground) {
    melee_core::ct!("grCastle_801D05E8");
    grCastle_801D0550_sub(arg1, 2);
}
pub unsafe extern "C" fn grCastle_801D0634(_arg0: *mut c_void, arg1: *mut Ground) {
    melee_core::ct!("grCastle_801D0634");
    grCastle_801D0550_sub(arg1, 3);
}
pub unsafe extern "C" fn grCastle_801D0680(_arg0: *mut c_void, arg1: *mut Ground) {
    melee_core::ct!("grCastle_801D0680");
    grCastle_801D0550_sub(arg1, 4);
}

unsafe fn grCastle_801D06CC_sub(gp: *mut Ground, gobj: *mut Ground_GObj, i: usize) {
    if ext::ftLib_80086960(gobj) != 0 || ext::itGetKind(gobj) != It_PKind_Random {
        (*gp).u.castle10.state[i] = 1;
        ext::grMaterial_801C8CDC(u32_ptr((*gp).u.castle10.x10C[i]));
        (*gp).u.castle10.x10C[i] = 0;
        ext::Ground_801C53EC(0x53020);
    }
}

pub unsafe extern "C" fn grCastle_801D06CC(_arg0: *mut c_void, arg1: *mut Ground, gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801D06CC");
    grCastle_801D06CC_sub(arg1, gobj, 0);
}
pub unsafe extern "C" fn grCastle_801D0744(_arg0: *mut c_void, arg1: *mut Ground, gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801D0744");
    grCastle_801D06CC_sub(arg1, gobj, 1);
}
pub unsafe extern "C" fn grCastle_801D07BC(_arg0: *mut c_void, arg1: *mut Ground, gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801D07BC");
    grCastle_801D06CC_sub(arg1, gobj, 2);
}
pub unsafe extern "C" fn grCastle_801D0834(_arg0: *mut c_void, arg1: *mut Ground, gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801D0834");
    grCastle_801D06CC_sub(arg1, gobj, 3);
}
pub unsafe extern "C" fn grCastle_801D08AC(_arg0: *mut c_void, arg1: *mut Ground, gobj: *mut Ground_GObj) {
    melee_core::ct!("grCastle_801D08AC");
    grCastle_801D06CC_sub(arg1, gobj, 4);
}

pub unsafe extern "C" fn fn_801D0924(gobj: *mut HSD_GObj, renderpass: i32) {
    melee_core::ct!("fn_801D0924");
    let gp = (*gobj).user_data as *mut Ground;

    ext::grDisplay_801C5DB0(gobj, renderpass);

    if (*gp).x10_flags.b7() != 0 {
        let mut color1 = _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFE };
        let mut color2 = _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };

        for i in 0..12usize {
            if !(*gp).u.castle3.x1C[i].data.is_null() {
                ext::lb_800117F4(&mut (*gp).u.castle3.x1C[i], &mut color1, &mut color2, 999, renderpass as u32);
            }
        }
    }
}

pub unsafe extern "C" fn grCastle_801D09B8(_unused: *mut c_void, gobj: *mut HSD_GObj, arg2: *mut Vec3) -> i32 {
    melee_core::ct!("grCastle_801D09B8");
    let mut sp2c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp20 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let temp_f31 = 14.0f32 * ext::Ground_801C0498();
    ext::ftLib_80086644(gobj, &mut sp2c);
    ext::ftLib_80086684(gobj, &mut sp20);
    if sp2c.y < temp_f31 {
        if sp20.y > temp_f31 {
            let temp_f1 = ext::ftLib_80086B80(gobj) / 10.0;
            sp2c.y = temp_f31;
            grCastle_801D0A9C(&mut sp2c, temp_f1);
        }
        ext::ftLib_80086BEC(gobj, &mut sp14);
        if sp14.y < -0.5 {
            (*arg2).x = 0.0;
            (*arg2).y = 0.1;
            (*arg2).z = 0.0;
        } else {
            (*arg2).z = 0.0;
            (*arg2).y = 0.0;
            (*arg2).x = 0.0;
        }
        return 1;
    }
    0
}

pub unsafe fn grCastle_801D0A9C(arg0: *mut Vec3, arg8: f32) {
    melee_core::ct!("grCastle_801D0A9C");
    let gen = ext::grLib_801C96F8(0x7530, 0x1E, arg0);
    if !gen.is_null() {
        let srt = (*gen).appsrt;
        (*srt).scale.x *= arg8;
        (*srt).scale.y *= arg8;
        (*srt).scale.z *= arg8;
    }
}

pub unsafe extern "C" fn grCastle_801D0B04(_arg: i32) -> *mut DynamicsDesc {
    melee_core::ct!("grCastle_801D0B04");
    null_mut()
}

pub unsafe extern "C" fn grCastle_801D0B0C(v: *mut Vec3, _arg1: i32, jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grCastle_801D0B0C");
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut x = 192.0f32;
    let mut y = 137.0f32;

    x *= ext::Ground_801C0498();
    y *= ext::Ground_801C0498();
    if (*v).x > x && (*v).y < y {
        return 0;
    }
    ext::lb_8000B1CC(jobj, null_mut(), &mut pos);
    if (1.0f32 + (*v).y) > pos.y {
        return 1;
    }
    0
}

pub unsafe fn grCastle_801D0BBC() {
    melee_core::ct!("grCastle_801D0BBC");
    let mut angle = ext::atan2f((*grCs_804D6974).x4.z, (*grCs_804D6974).x4.x);
    angle += grCs_804D45E0;
    if (angle as f64) > 1.0471975430846214 {
        angle = 1.0471976;
    }
    if (angle as f64) < -1.0471975430846214 {
        angle = -1.0471976;
    }
    (*grCs_804D6974).x4.x = melee_runtime::gekko::cosf(angle);
    (*grCs_804D6974).x4.z = melee_runtime::gekko::sinf(angle);
    if ((*grCs_804D6974).unk_angle_int % 30) == 0 && (ext::HSD_Randf() as f64) > 0.5 {
        grCs_804D45E0 = -grCs_804D45E0;
    }
    if ((*grCs_804D6974).unk_angle_int % 300) == 0 {
        if grCs_804D45E4 != 0 {
            grCs_804D45E4 = 0;
        } else if (ext::HSD_Randf() as f64) > 0.5 {
            grCs_804D45E4 = 1;
        }
    }
    if grCs_804D45E4 != 0 {
        (*grCs_804D6974).unk_scale = 0.3f64.fmadd(ext::HSD_Randf() as f64, 0.2) as f32;
        return;
    }
    (*grCs_804D6974).unk_scale = (0.05f64 * ext::HSD_Randf() as f64) as f32;
}

pub unsafe fn grCastle_801D0D24() {
    melee_core::ct!("grCastle_801D0D24");
    if grCs_804D45E4 != 0 {
        (*grCs_804D6974).unk_scale = 0.3f64.fmadd(ext::HSD_Randf() as f64, 0.2) as f32;
    } else {
        (*grCs_804D6974).unk_scale = (0.05f64 * ext::HSD_Randf() as f64) as f32;
    }
}

pub unsafe fn grCastle_801D0D84(jobj: *mut HSD_JObj) {
    melee_core::ct!("grCastle_801D0D84");
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut dir = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut fwd = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut cross = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if jobj.is_null() {
        return;
    }

    HSD_JObjSetupMatrix(jobj);
    pos.x = (*jobj).mtx[0][3];
    pos.y = (*jobj).mtx[1][3];
    pos.z = (*jobj).mtx[2][3];

    let mut rot = (*jobj).rotate;
    let rot_y = rot.y;

    let wind_len = ext::lb_800103B8(&mut pos, &mut dir);
    dir.y = 0.0;
    let len = ext::lbVector_Normalize(&mut dir);

    fwd.x = melee_runtime::gekko::cosf(rot_y);
    fwd.y = 0.0;
    fwd.z = -melee_runtime::gekko::sinf(rot_y);

    let angle = ext::lbVector_Angle(&mut fwd, &mut dir);
    let near_zero = if angle < 0.00001 && angle > -0.00001 { 1 } else { 0 };
    if near_zero == 0 {
        let mut correction = len * (wind_len * melee_runtime::gekko::sinf(angle));
        if correction < 0.0 {
            correction = -correction;
        }
        ext::PSVECCrossProduct(&mut fwd, &mut dir, &mut cross);
        if (cross.y as f64) > 0.0 {
            rot.y = rot_y + correction;
        } else {
            rot.y = rot_y - correction;
        }
    }

    if (rot.y as f64) > 3.1415926292538643 {
        rot.y = ((rot.y as f64) - 6.2831852585077286) as f32;
    } else if (rot.y as f64) < -3.1415926292538643 {
        rot.y = ((rot.y as f64) + 6.2831852585077286) as f32;
    }

    HSD_JObjSetRotation(jobj, &rot);
    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_USE_QUATERNION as u32);
}

pub unsafe fn grCastle_801D0FF0() -> f32 {
    melee_core::ct!("grCastle_801D0FF0");
    50.0f32 * ext::Ground_801C0498()
}

// ---- folded from grcastle.rs (consolidation steps 21/22) ----
/// grCastle_801CD4A4_fp (gr/grcastle.c:333)
    pub unsafe extern "C" fn grCastle_801CD4A4_fp() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

// ---- folded from grcastle.rs (consolidation steps 21/22) ----
/// grCastle_801CD4C8_fp (gr/grcastle.c:338)
    pub extern "C" fn grCastle_801CD4C8_fp() -> i32 { 0 }

/// grCastle_801CD600_fp (gr/grcastle.c:383)
    pub extern "C" fn grCastle_801CD600_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grCastle_801CD8A0_fp (gr/grcastle.c:472)
    pub extern "C" fn grCastle_801CD8A0_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grCastle_801CDC3C_fp (gr/grcastle.c:565)
    pub extern "C" fn grCastle_801CDC3C_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grCastle_801CE3A4_fp (gr/grcastle.c:824)
    pub extern "C" fn grCastle_801CE3A4_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grCastle_801CE858_fp (gr/grcastle.c:976)
    pub extern "C" fn grCastle_801CE858_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grCastle_801CE9E0_fp (gr/grcastle.c:1027)
    pub extern "C" fn grCastle_801CE9E0_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grCastle_801CEEFC_fp (gr/grcastle.c:1179)
    pub extern "C" fn grCastle_801CEEFC_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grCastle_801CF300_fp (gr/grcastle.c:1298)
    pub extern "C" fn grCastle_801CF300_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grCastle_801D0B04_fp (gr/grcastle.c:1897)
    pub extern "C" fn grCastle_801D0B04_fp(a0: i32) -> *mut DynamicsDesc { core::ptr::null_mut() }
