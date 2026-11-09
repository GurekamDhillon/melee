//! src/melee/gr/grbigblue.c: Big Blue (F-Zero) stage, the first half of the Big Blue stage pair (`grbigblueroute.rs` is the
//! route variant): the three hover-platform gobjs and their manager (`801E6904` / `801E6C60`), the car gobj
//! (`801E6364`, `801ECB50`..`801EF424`), the moving road (`801EB004`..`801EBAF8`), the ground probe `801EC58C` and the
//! per-frame helpers they share.
//!
//! Fused sites (tools/rust-port/fma_sites.py with -g: 44 f32 + 2 f64 outside the MSL `sqrtf`), written out as `fmadds`
//! / `fmsubs` / `mul_add`; see the comment at each site. The `sqrtf` of `fn_801E8560` is the MSL one
//! (`melee_runtime::gekko::sqrtf`).
//!
//! Native adaptations of GameCube layout tricks, all numerically identical to the PPC build:
//! * raw `(u8*) gp + 0xNN` accesses of the C (the Ground's `u` union starts at GC +0xC4; the car lane table at +0xD4
//!   with 0x40 stride; the `grBb_CarGround` / `grBigBlue_CarPhysics` overlays) are the named fields of the same bytes
//!   (`u.bigblue.platform.*`, `u.bigblue.car.lanes[i].*`, ...). The PPC reads a lane's `state` as
//!   `(byte >> 2) & 0x3F`, which is the MSB-first bitfield accessor of the generated record;
//! * `grBigBlue_801E8B84` walks the four lanes with `gp = (Ground*) &gp->x20[8]` (a 0x40 byte step on the PPC): the
//!   port indexes the lanes;
//! * the manager's `event_data[]` slots hold `(void*) 1` / jobj pointers that the C compares through `(int)`.
//! State-oracle checked against the C (tests/gro_grbigblue.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, unused_mut, unused_variables, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
use melee_core::fp::Fmadd as _;
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use super::grinl::*;
use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null_mut};
use melee_core::fp::{fmadds, fmsubs};
#[allow(unused_imports)]
use melee_core::fp::{fnmadds, fnmsubs};
use melee_runtime::gekko::{cosf, sinf, sqrtf};
use melee_types::*;


/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::stage_info;
    hook_or!(CAMERA_8003010C, Camera_8003010C, () -> i32, || unsafe { crate::tu::camera::Camera_8003010C() } as i32);
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
    pub static mut GROUND_801C2CF4: Option<fn(s32) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C2CF4(a0: s32) -> *mut HSD_JObj {
        match unsafe { GROUND_801C2CF4 } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_801C2CF4(a0 as _) as _ }; } }
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
    pub static mut GROUND_801C3FA4: Option<fn(*mut HSD_GObj, isize) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3FA4(gobj: *mut HSD_GObj, depth: i32) -> *mut HSD_JObj {
        match unsafe { GROUND_801C3FA4 } { Some(f) => f(gobj, depth as isize), None => { return unsafe { crate::tu::ground::Ground_801C3FA4(gobj as _, depth as _) as _ }; } }
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
    pub static mut GROUND_801C5440: Option<fn(*mut Ground, s32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5440(gp: *mut Ground, i: s32, a2: u32) {
        match unsafe { GROUND_801C5440 } { Some(f) => f(gp, i, a2), None => { unsafe { crate::tu::ground::Ground_801C5440(gp as _, i as _, a2 as _); } return; } }
    }
    pub static mut GROUND_801C5630: Option<fn(*mut Ground, s32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5630(gp: *mut Ground, i: s32, val: f32) {
        match unsafe { GROUND_801C5630 } { Some(f) => f(gp, i, val), None => { unsafe { crate::tu::ground::Ground_801C5630(gp as _, i as _, val as _); } return; } }
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
    hook_or!(ATAN2F, atan2f, (y: f32, x: f32) -> f32, melee_runtime::lbtrigf::atan2f);
    hook!(GM_GETDBPAUSEFLAG, gm_GetDbPauseFlag, (bit: i32) -> i32);
    pub static mut GRANIME_801C8138: Option<fn(*mut HSD_GObj, enum_t, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C8138(a0: *mut HSD_GObj, a1: enum_t, a2: i32) {
        match unsafe { GRANIME_801C8138 } { Some(f) => f(a0, a1, a2), None => { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRDISPLAY_801C5DB0: Option<fn(*mut HSD_GObj, isize)> = None;
    #[allow(non_snake_case)]
    pub fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: isize) {
        match unsafe { GRDISPLAY_801C5DB0 } { Some(f) => f(a0, a1), None => { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } return; } }
    }
    pub static mut GRFZEROCAR_801CAFBC: Option<fn(*mut HSD_GObj, *mut c_void, s32, s32)> = None;
    #[allow(non_snake_case)]
    pub fn grFZeroCar_801CAFBC(a0: *mut HSD_GObj, a1: *mut c_void, a2: s32, a3: s32) {
        match unsafe { GRFZEROCAR_801CAFBC } { Some(f) => f(a0, a1, a2, a3), None => { unsafe { crate::tu::grfzerocar::grFZeroCar_801CAFBC(a0 as _, a1 as _, a2 as _, a3 as _); } return; } }
    }
    pub static mut GRMATERIAL_801C8858: Option<fn(*mut HSD_JObj, u32)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8858(a0: *mut HSD_JObj, flags: u32) {
        match unsafe { GRMATERIAL_801C8858 } { Some(f) => f(a0, flags), None => { unsafe { crate::tu::grmaterial::grMaterial_801C8858(a0 as _, flags as _); } return; } }
    }
    pub static mut GRMATERIAL_801C8A04: Option<fn(*mut HSD_JObj, u32)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8A04(a0: *mut HSD_JObj, flags: u32) {
        match unsafe { GRMATERIAL_801C8A04 } { Some(f) => f(a0, flags), None => { unsafe { crate::tu::grmaterial::grMaterial_801C8A04(a0 as _, flags as _); } return; } }
    }
    pub static mut GRZAKOGENERATOR_801CAE04: Option<fn(*mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj {
        match unsafe { GRZAKOGENERATOR_801CAE04 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ }; } }
    }
    hook!(IT_8026BE84, it_8026BE84, (bobOmbRain: *mut BobOmbRain) -> *mut HSD_GObj);
    hook!(IT_8026D324, it_8026D324, (a0: ItemKind) -> i32);
    pub static mut LBVECTOR_ADD: Option<fn(*mut Vec, *mut Vec) -> *mut Vec> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Add(a: *mut Vec, b: *mut Vec) -> *mut Vec {
        match unsafe { LBVECTOR_ADD } { Some(f) => f(a, b), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Add(a as _, b as _) as _ }; } }
    }
    pub static mut LBVECTOR_APPLYEULERROTATION: Option<fn(*mut Vec, *mut Vec) -> *mut Vec> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_ApplyEulerRotation(v: *mut Vec, angles: *mut Vec) -> *mut Vec {
        match unsafe { LBVECTOR_APPLYEULERROTATION } { Some(f) => f(v, angles), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_ApplyEulerRotation(v as _, angles as _) as _ }; } }
    }
    pub static mut LBVECTOR_DIFF: Option<fn(*mut Vec, *mut Vec, *mut Vec) -> *mut Vec> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Diff(a: *mut Vec, b: *mut Vec, result: *mut Vec) -> *mut Vec {
        match unsafe { LBVECTOR_DIFF } { Some(f) => f(a, b, result), None => { return unsafe { melee_runtime::tu::lbvector::lbVector_Diff(a as _, b as _, result as _) as _ }; } }
    }
    pub static mut LBVECTOR_ROTATE: Option<fn(*mut Vec, i32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn lbVector_Rotate(v: *mut Vec, axis: i32, angle: f32) {
        match unsafe { LBVECTOR_ROTATE } { Some(f) => f(v, axis, angle), None => { unsafe { melee_runtime::tu::lbvector::lbVector_Rotate(v as _, axis as _, angle as _); } return; } }
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
    pub static mut LB_80011A50: Option<fn(*mut Vec, i32, f32, f32, f32, f32, f32, f32, f32) -> *mut lb_80011A50_t> = None;
    #[allow(non_snake_case)]
    pub fn lb_80011A50(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32) -> *mut lb_80011A50_t {
        match unsafe { LB_80011A50 } { Some(f) => f(a0, a1, a2, a3, a4, a5, a6, a7, a8), None => { return unsafe { melee_runtime::tu::lb_00F9::lb_80011A50(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _, a8 as _) as _ }; } }
    }
    pub static mut MEMZERO: Option<fn(*mut c_void, ssize_t)> = None;
    #[allow(non_snake_case)]
    pub fn memzero(mem: *mut c_void, size: ssize_t) {
        match unsafe { MEMZERO } { Some(f) => f(mem, size), None => { unsafe { melee_runtime::tu::lb_00B0::memzero(mem as _, size as _); } return; } }
    }
    pub static mut MPCHECKMULTIPLE: Option<fn(f32, f32, f32, f32, *mut Vec, *mut i32, *mut u32, *mut Vec, u32, i32, i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn mpCheckMultiple(x0: f32, y0: f32, x1: f32, y1: f32, pos_out: *mut Vec, line_id_out: *mut i32, flags_out: *mut u32, normal_out: *mut Vec, checks: u32, joint_id_skip: i32, joint_id_only: i32) -> i32 {
        match unsafe { MPCHECKMULTIPLE } { Some(f) => f(x0, y0, x1, y1, pos_out, line_id_out, flags_out, normal_out, checks, joint_id_skip, joint_id_only), None => { return unsafe { crate::tu::mplib::mpCheckMultiple(x0 as _, y0 as _, x1 as _, y1 as _, pos_out as _, line_id_out as _, flags_out as _, normal_out as _, checks as _, joint_id_skip as _, joint_id_only as _) as _ }; } }
    }
    pub static mut MPJOINTFROMLINE: Option<fn(i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn mpJointFromLine(line_id: i32) -> i32 {
        match unsafe { MPJOINTFROMLINE } { Some(f) => f(line_id), None => { return unsafe { crate::tu::mplib::mpJointFromLine(line_id as _) as _ }; } }
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
    pub static mut MPLIB_80058044: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_80058044(joint_id: i32) {
        match unsafe { MPLIB_80058044 } { Some(f) => f(joint_id), None => { unsafe { crate::tu::mplib::mpLib_80058044(joint_id as _); } return; } }
    }
    pub static mut MPLIB_80058560: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn mpLib_80058560() {
        match unsafe { MPLIB_80058560 } { Some(f) => f(), None => { unsafe { crate::tu::mplib::mpLib_80058560(); } return; } }
    }

    /// `grDisplay_801C5DB0` as a render callback: its address is what `GObj_SetupGXLink` receives.
    pub unsafe extern "C" fn grDisplay_801C5DB0_cb(gobj: *mut HSD_GObj, pass: isize) {
        grDisplay_801C5DB0(gobj, pass as isize)
    }
}
use ext::*;

/// `Ground_UpdateWindAndMapColl` (gr/inlines.h).
#[inline]
unsafe fn Ground_UpdateWindAndMapColl(gobj: *mut HSD_GObj) {
    lb_800115F4();
    Ground_UpdateMapColl(gobj);
}

/// `grBb_ItemKindList`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grBb_ItemKindList {
    pub kinds: [ItemKind; 5],
}

/// `grBb_LineIds`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grBb_LineIds {
    pub v: [s32; 32],
}

/// `grBb_TrackEntry`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grBb_TrackEntry {
    pub jobj_index: s16,
    pub start_index: s16,
    pub end_index: s16,
    pub pad: s16,
    pub delta: Vec3,
}

/// `grBb_YakumonoParam` (grbigblue.static.h).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grBb_YakumonoParam {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: s32,
    pub x14: s32,
    pub x18: s32,
    pub x1C: s32,
    pub x20: s32,
    pub x24: f32,
    pub x28: f32,
    pub x2C: f32,
    pub x30: f32,
    pub x34: f32,
    pub x38: f32,
    pub x3C: f32,
    pub x40: f32,
    pub x44: f32,
    pub x48: f32,
    pub x4C: f32,
    pub x50: f32,
    pub x54: f32,
    pub x58: f32,
    pub x5C: f32,
    pub x60: f32,
    pub pad64: [u8; 0x68 - 0x64],
    pub x68: f32,
    pub x6C: f32,
    pub x70: f32,
    pub x74: f32,
    pub x78: f32,
    pub x7C: f32,
    pub x80: f32,
    pub x84: f32,
    pub x88: s32,
    pub x8C: s32,
    pub x90: f32,
    pub x94: f32,
    pub x98: f32,
    pub x9C: f32,
    pub xA0: f32,
    pub xA4: f32,
    pub xA8: f32,
    pub xAC: f32,
    pub xB0: s32,
    pub xB4: s32,
    pub xB8: s32,
    pub xBC: f32,
    pub xC0: f32,
    pub xC4: f32,
    pub xC8: f32,
    pub xCC: f32,
    pub xD0: f32,
    pub xD4: f32,
    pub xD8: f32,
    pub xDC: s32,
    pub xE0: s32,
    pub xE4: f32,
    pub xE8: f32,
    pub xEC: f32,
    pub xF0: f32,
    pub xF4: f32,
    pub xF8: f32,
    pub xFC: f32,
    pub x100: f32,
    pub x104: f32,
    pub x108: f32,
    pub x10C: s32,
    pub x110: s32,
    pub pad114: [u8; 0x11C - 0x114],
    pub x11C: s32,
    pub x120: s32,
    pub x124: f32,
    pub x128: f32,
    pub x12C: f32,
    pub x130: f32,
    pub x134_translate: Vec3,
    pub x140_scale: f32,
}

/// `static grBb_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grBb_YakumonoParam = null_mut();

macro_rules! cbs {
    () => {
        StageCallbacks { on_init: None, callback1: None, gobj_proc: None, callback3: None, x10: StageCallbacks_x10 { flags: 0 } }
    };
    ($a:ident, $b:ident, $c:ident, $d:ident, $flags:expr) => {
        StageCallbacks {
            on_init: Some($a),
            callback1: Some($b),
            gobj_proc: Some($c),
            callback3: Some($d),
            x10: StageCallbacks_x10 { flags: u32::to_be($flags) },
        }
    };
}

/// `GrJoint grBb_803E2938[28]`.
pub static mut grBb_803E2938: [GrJoint; 28] = {
    const fn j(x: i16, y: i16, z: i16) -> GrJoint {
        GrJoint { x, y, z }
    }
    [
        j(36, 34, 10), j(37, 34, 10), j(38, 34, 7), j(39, 34, 7), j(40, 34, 7), j(41, 34, 14), j(42, 34, 14), j(43, 34, 14),
        j(44, 34, 15), j(45, 34, 15), j(46, 34, 15), j(47, 34, 16), j(48, 34, 16), j(49, 34, 16), j(52, 34, 32), j(53, 34, 32),
        j(54, 34, 32), j(55, 34, 35), j(56, 34, 35), j(57, 34, 35), j(58, 34, 38), j(59, 34, 38), j(60, 34, 38), j(61, 34, 41),
        j(62, 34, 41), j(63, 34, 41), j(64, 34, 44), j(65, 34, 44),
    ]
};

/// `StageCallbacks grBb_StageCallbacks[41]`.
pub static mut grBb_StageCallbacks: [StageCallbacks; 41] = [
    cbs!(grBigBlue_801E5AE4, grBigBlue_801E5B10, grBigBlue_801E5B18, grBigBlue_801E5B1C, 0),
    cbs!(grBigBlue_801E6298, grBigBlue_801E6354, grBigBlue_801E635C, grBigBlue_801E6360, 0x40000000),
    cbs!(grBigBlue_801E6200, grBigBlue_801E6288, grBigBlue_801E6290, grBigBlue_801E6294, 0),
    cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(),
    cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(), cbs!(),
    cbs!(grBigBlue_801E5B20, grBigBlue_801E6114, grBigBlue_801E611C, grBigBlue_801E6120, 0),
    cbs!(grBigBlue_801E6904, grBigBlue_801E6C58, grBigBlue_801E6C60, grBigBlue_801E855C, 0),
    cbs!(grBigBlue_801E6364, grBigBlue_801E687C, grBigBlue_801E6884, grBigBlue_801E68B8, 0),
    cbs!(grBigBlue_801E613C, grBigBlue_801E61BC, grBigBlue_801E61C4, grBigBlue_801E61FC, 0),
    cbs!(grBigBlue_801E8D64, grBigBlue_801E93D0, grBigBlue_801E93D8, grBigBlue_801E9F38, 0x80000000),
    cbs!(grBigBlue_801E9F3C, grBigBlue_801EA054, grBigBlue_801EA05C, grBigBlue_801EAB4C, 0),
    cbs!(), cbs!(), cbs!(), cbs!(),
];

/// `static const Vec3 grBb_803B8108`.
pub static mut grBb_803B8108: Vec3 = Vec3 { x: -1.0, y: 0.0, z: 0.0 };
/// `static const Vec3 grBb_803B8114`.
pub static mut grBb_803B8114: Vec3 = Vec3 { x: 56.0, y: 40.0, z: 24.0 };
/// `static const grBb_ItemKindList grBb_803B8120`.
pub static mut grBb_803B8120: grBb_ItemKindList = grBb_ItemKindList {
    kinds: [It_Kind_Sword, It_Kind_S_Scope, It_Kind_RabbitC, It_Kind_F_Flower, It_Kind_Kusudama],
};
/// `const grBb_LineIds grBb_803B8134`.
pub static mut grBb_803B8134: grBb_LineIds = grBb_LineIds {
    v: [
        33, 35, 38, 39, 40, 36, 37, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62,
        63, 64, 65,
    ],
};

/// `StageData grBb_StageData`.
pub static mut grBb_StageData: StageData = StageData {
    grkind: Gr_Kind_BigBlue,
    callbacks: unsafe { addr_of!(grBb_StageCallbacks) as *mut StageCallbacks },
    data1: b"/GrBb.dat\0".as_ptr() as *mut i8,
    on_init: Some(grBigBlue_801E57C0),
    on_demo_init: Some(grBigBlue_801E57BC),
    on_load: Some(grBigBlue_801E59C8),
    on_start: Some(grBigBlue_801E59CC),
    callback4: Some(grBigBlue_801E59F0),
    on_touch_line: Some(grBigBlue_801EFC0C),
    on_check_shadow_render: Some(grBigBlue_801EFC14),
    flags2: 1 << 0,
    joints: unsafe { addr_of!(grBb_803E2938) as *mut GrJoint },
    joint_count: 28,
};

/// `static s16 grBb_803E2D84[30]`.
pub static mut grBb_803E2D84: [s16; 30] = [
    0x0004, 0x0005, 0x0006, 0x0008, 0x0009, 0x000A, 0x000B, 0x000C, 0x000D, 0x000E, 0x0003, 0x000F, 0x0010, 0x0011, 0x0012,
    0x0013, 0x0014, 0x0015, 0x0016, 0x0017, 0x0018, 0x0019, 0x001A, 0x001B, 0x001C, 0x001D, 0x0025, 0x0026, 0x0027, 0x0028,
];
/// `static s16 grBb_803E2DC0[30]`.
pub static mut grBb_803E2DC0: [s16; 30] = [
    0x0001, 0x0002, 0x0003, 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009, 0x000A, 0x000B, 0x000C, 0x000D, 0x000E, 0x000F,
    0x0010, 0x0011, 0x0012, 0x0013, 0x0014, 0x0015, 0x0016, 0x0017, 0x0018, 0x0019, 0x001A, 0x001B, 0x001C, 0x001D, 0x001E,
];
/// `static s16 lbl_803E2DFC[30]`.
pub static mut lbl_803E2DFC: [s16; 30] = [
    0x0003, 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009, 0x000A, 0x000B, 0x000C, 0x000D, 0x000E, 0x000F, 0x0010, 0x0011,
    0x0012, 0x0013, 0x0014, 0x0015, 0x0016, 0x0017, 0x0018, 0x0019, 0x001A, 0x001B, 0x001C, 0x001D, 0x001E, 0x001F, 0x0020,
];
/// `u32 lbl_803E3010[4]`.
pub static mut lbl_803E3010: [u32; 4] = [0x0006DDD2, 0x0006DDD3, 0x0006DDD4, 0x0006DDD5];

/// `static grBb_TrackEntry grBb_TrackEntries[12]` (the `delta` members are written by `grBigBlue_801EB004`).
pub static mut grBb_TrackEntries: [grBb_TrackEntry; 12] = {
    const fn t(a: i16, b: i16, c: i16) -> grBb_TrackEntry {
        grBb_TrackEntry { jobj_index: a, start_index: b, end_index: c, pad: 0, delta: Vec3 { x: 0.0, y: 0.0, z: 0.0 } }
    }
    [
        t(4, 6, 5), t(19, 21, 20), t(7, 9, 8), t(10, 12, 11), t(44, 46, 45), t(1, 3, 2),
        t(32, 34, 33), t(35, 37, 36), t(38, 40, 39), t(41, 43, 42), t(29, 31, 30), t(13, 18, 17),
    ]
};

// ---------------------------------------------------------------------------------------------------- views
/// The `Ground.u.bigblue` record's anonymous struct (header word, `xC8`, `xCC`, `xD0`, `xD4[3]`, `data[3]`).
type BbS = grBigBlue_GroundVars_x0_x0;
/// Its header union (`x0_w`, the four bytes `x0..x3`, `x0_b1`, the lane bitfields).
type BbH = grBigBlue_GroundVars_x0_x0_x0;
type BbL = grBigBlue_GroundVars_x0_x0_x0_x0_2;

#[inline(always)]
unsafe fn bbs(gp: *mut Ground) -> *mut BbS {
    addr_of_mut!((*gp).u.bigblue.x0_u.x0_s)
}
#[inline(always)]
unsafe fn bbh(gp: *mut Ground) -> *mut BbH {
    addr_of_mut!((*gp).u.bigblue.x0_u.x0_s.x0_u)
}
/// `gp->u.bigblue.x0` (byte 0 of the union).
#[inline(always)]
unsafe fn bb_x0(gp: *mut Ground) -> *mut u8 {
    addr_of_mut!((*bbh(gp)).x0_s.x0)
}
/// `gp->u.bigblue.x1` (byte 1).
#[inline(always)]
unsafe fn bb_x1(gp: *mut Ground) -> *mut u8 {
    addr_of_mut!((*bbh(gp)).x0_s.x1)
}
/// The 32-bit bitfield view of the header (`b0`, `b1`, `b2`, `prev_lane`, ...).
#[inline(always)]
unsafe fn bb_lane(gp: *mut Ground) -> *mut BbL {
    addr_of_mut!((*bbh(gp)).x0_2)
}
/// `Ground_GetMapGObj(id)->user_data`.
#[inline(always)]
unsafe fn map_ground(id: i32) -> *mut Ground {
    (*Ground_GetMapGObj(id)).user_data as *mut Ground
}
/// `(int) ptr`: the C truncates a pointer to `int`.
#[inline(always)]
fn ptr_i32<T>(p: *mut T) -> i32 {
    p as usize as u32 as i32
}
/// `ZRANDI(n)` / `randi(n)`.
#[inline(always)]
unsafe fn randi(n: i32) -> i32 {
    if n != 0 { HSD_Randi(n) } else { 0 }
}
/// `rand_range(a, b)` of gr/inlines.h.
#[inline(always)]
unsafe fn rand_range(a: i32, b: i32) -> i32 {
    if a > b {
        b + randi(a - b)
    } else if a < b {
        a + randi(b - a)
    } else {
        a
    }
}
const F32_MAX: f32 = 3.4028235e38;
#[inline(always)]
fn sqrtf_(x: f32) -> f32 {
    sqrtf(x)
}

// ---------------------------------------------------------------------------------------------------- stage entry points
/// `grBigBlue_801E57BC` (on_demo_init).
pub unsafe extern "C" fn grBigBlue_801E57BC(_arg: i32) { melee_core::ct!("grBigBlue_801E57BC");}

/// `grBigBlue_801E57C0` (on_init).
pub unsafe extern "C" fn grBigBlue_801E57C0() {
    melee_core::ct!("grBigBlue_801E57C0");
    yakumono_param = Ground_GetYakumonoParam() as *mut grBb_YakumonoParam;
    let si = ext::stage_info();
    (*si).unk8C.set_b4(0);
    (*si).unk8C.set_b5(1);
    grBigBlue_801E59F8(0x1F);
    grBigBlue_801E59F8(0);
    grBigBlue_801E59F8(1);
    grBigBlue_801E59F8(2);
    grBigBlue_801E59F8(0x22);
    grBigBlue_801E59F8(0x21);
    HSD_JObjSetFlagsAll((*grBigBlue_801E59F8(0x20)).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
    grBigBlue_801E59F8(0x23);
    HSD_JObjSetFlagsAll((*grBigBlue_801E59F8(0x24)).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
    Ground_801C39C0();
    Ground_801C3BB4();
    let mut id = 0x21;
    while id <= 0x41 {
        mpLib_80058044(id);
        id += 1;
    }
    let mut direction = grBb_803B8108;
    lb_80011A50(&mut direction, -1, 1.0, 0.0, (M_PI / 3.0) as f32, -100000.0, 100000.0, 100000.0, -100000.0);
}

/// `grBigBlue_801E59C8` (on_load).
pub unsafe extern "C" fn grBigBlue_801E59C8() { melee_core::ct!("grBigBlue_801E59C8");}

/// `grBigBlue_801E59CC` (on_start).
pub unsafe extern "C" fn grBigBlue_801E59CC() {
    melee_core::ct!("grBigBlue_801E59CC");
    grZakoGenerator_801CAE04(null_mut());
}

/// `grBigBlue_801E59F8`: spawn one stage gobj's procs from `grBb_StageCallbacks[id]`.
pub unsafe extern "C" fn grBigBlue_801E59F8(id: s32) -> *mut HSD_GObj {
    melee_core::ct!("grBigBlue_801E59F8");
    let cbs = addr_of_mut!(grBb_StageCallbacks[id as usize]);
    let gobj = Ground_GetStageGObj(id);
    if !gobj.is_null() {
        let gp = GET_GROUND(gobj);
        (*gp).x8_callback = None;
        (*gp).xC_callback = None;

        GObj_SetupGXLink(gobj, Some(ext::grDisplay_801C5DB0_cb), 3, 0);
        if (*cbs).callback3.is_some() {
            (*gp).x1C_callback = (*cbs).callback3;
        }
        if let Some(on_init) = (*cbs).on_init {
            on_init(gobj);
        }
        if (*cbs).gobj_proc.is_some() {
            HSD_GObj_SetupProc(gobj, (*cbs).gobj_proc, 4);
        }
    } else {
        OSReport(
            b"%s:%d: couldn t get gobj(id=%d)\n\0".as_ptr() as *const i8,
            &[b"grbigblue.c\0".as_ptr() as usize, 0x17E, id as u32 as usize],
        );
    }
    gobj
}

/// `grBigBlue_801E5AE4`.
pub unsafe extern "C" fn grBigBlue_801E5AE4(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E5AE4");
    grAnime_801C8138(gobj, (*GET_GROUND(gobj)).map_id as enum_t, 0);
}

pub unsafe extern "C" fn grBigBlue_801E5B18(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801E5B18");}

pub unsafe extern "C" fn grBigBlue_801E5B1C(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801E5B1C");}

/// `grBigBlue_801E5B20` (gobj 31: the backdrop).
pub unsafe extern "C" fn grBigBlue_801E5B20(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E5B20");
    let mut jobj = GET_JOBJ(gobj);

    (*gobj).render_cb = Some(fn_801EFB9C);

    HSD_JObjSetTranslateX(jobj, (*yakumono_param).x134_translate.x);
    HSD_JObjSetTranslateY(jobj, (*yakumono_param).x134_translate.y);
    HSD_JObjSetTranslateZ(jobj, (*yakumono_param).x134_translate.z);
    HSD_JObjSetScaleX(jobj, (*yakumono_param).x140_scale);
    HSD_JObjSetScaleY(jobj, (*yakumono_param).x140_scale);
    HSD_JObjSetScaleZ(jobj, (*yakumono_param).x140_scale);

    jobj = HSD_JObjGetChild(jobj);
    if !jobj.is_null() {
        HSD_JObjSetTranslateX(jobj, 0.0);
        HSD_JObjSetTranslateY(jobj, 0.0);
        HSD_JObjSetTranslateZ(jobj, 0.0);

        jobj = HSD_JObjGetChild(jobj);
        if !jobj.is_null() {
            HSD_JObjSetTranslateX(jobj, 0.0);
            HSD_JObjSetTranslateY(jobj, 0.0);
            HSD_JObjSetTranslateZ(jobj, 0.0);
        }
    }
}

pub unsafe extern "C" fn grBigBlue_801E611C(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801E611C");}

pub unsafe extern "C" fn grBigBlue_801E6120(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801E6120");}

/// `fn_801E6124`.
pub unsafe extern "C" fn fn_801E6124(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801E6124");
    let g = GET_GROUND(gobj);
    (*bbh(g)).x0_1.set_x0_b1(0);
}

/// `grBigBlue_801E613C` (gobj 34: the moving road).
pub unsafe extern "C" fn grBigBlue_801E613C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E613C");
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    Ground_InitMapColl(jobj, (*gp).map_id as s32);
    grBigBlue_801EB004(gobj);
    (*bbh(gp)).x0_1.set_x0_b1(1);
    Ground_801C10B8(gobj, Some(fn_801E6124));
}

pub unsafe extern "C" fn grBigBlue_801E61C4(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E61C4");
    grBigBlue_801EBAF8(gobj);
    Ground_UpdateWindAndMapColl(gobj);
}

pub unsafe extern "C" fn grBigBlue_801E61FC(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801E61FC");}

/// `grBigBlue_801E6200`.
pub unsafe extern "C" fn grBigBlue_801E6200(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E6200");
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    let gp2 = GET_GROUND(gobj);
    (*gp2).x10_flags.set_b2(0);
    grMaterial_801C8A04(jobj, 0x08000000);
    grMaterial_801C8858(jobj, 0x20000000);
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grBigBlue_801E6290(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801E6290");}

pub unsafe extern "C" fn grBigBlue_801E6294(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801E6294");}

/// `grBigBlue_801E6298`.
pub unsafe extern "C" fn grBigBlue_801E6298(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E6298");
    let gp = GET_GROUND(gobj);
    grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    let jobj1 = Ground_801C3FA4(gobj, 6);
    let jobj2 = Ground_801C3FA4(gobj, 7);
    let jobj3 = Ground_801C3FA4(gobj, 3);
    let jobj4 = Ground_801C3FA4(gobj, 4);
    let jobj5 = Ground_801C3FA4(gobj, 2);
    Ground_801C4E70(Ground_801C3FA4(gobj, 5), jobj5, jobj4, jobj3, jobj2, jobj1);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grBigBlue_801E635C(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801E635C");}

pub unsafe extern "C" fn grBigBlue_801E6360(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801E6360");}

/// `grBigBlue_801E6364` (gobj 33: the car manager).
pub unsafe extern "C" fn grBigBlue_801E6364(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E6364");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let gp = GET_GROUND(gobj);
    let car = addr_of_mut!((*gp).u.bigblue.x0_u.car);

    Ground_InitMapColl(jobj, (*gp).map_id as s32);

    let mut scale = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
    HSD_JObjSetScale(jobj, &scale);

    // the C allocates the GC size (30 * 4 bytes); the native array holds 30 pointers
    (*car).collision_jobjs = HSD_MemAlloc((30 * core::mem::size_of::<*mut HSD_JObj>()) as ssize_t) as *mut *mut HSD_JObj;
    assert_(!(*car).collision_jobjs.is_null(), 774, "gp->u.carnull.coll_jobj");

    (*car).ranks = HSD_MemAlloc(30) as *mut u8;
    assert_(!(*car).ranks.is_null(), 776, "gp->u.carnull.rank");

    for i in 0..30usize {
        *(*car).collision_jobjs.add(i) = Ground_801C3FA4(gobj, grBb_803E2DC0[i] as i32);
    }

    let car_gobj = grBigBlue_801E59F8(4);
    assert_(!car_gobj.is_null(), 783, "car_gobj");
    grFZeroCar_801CAFBC(car_gobj, addr_of_mut!(grBb_803E2D84) as *mut c_void, 30, 1);

    let mut cur = (*car_gobj).hsd_obj as *mut HSD_JObj;
    let mut i = 0usize;
    if cur.is_null() {
        cur = null_mut();
    } else {
        cur = (*cur).child;
    }

    while i < 30 {
        let child = HSD_JObjGetChild(cur);
        let next = HSD_JObjGetNext(cur);

        HSD_JObjReparent(cur, *(*car).collision_jobjs.add(i));
        HSD_JObjSetRotationY(cur, core::f32::consts::FRAC_PI_2);

        let s = Ground_801C0498() * (*yakumono_param).xC;
        scale.x = s;
        scale.y = s;
        scale.z = s;

        HSD_JObjSetScale(*(*car).collision_jobjs.add(i), &scale);

        HSD_JObjGetScale(child, &mut scale);
        {
            let ratio = 1.0f32 / Ground_801C0498();
            scale.x *= ratio;
            scale.y *= ratio;
            scale.z *= ratio;
        }

        HSD_JObjSetScale(child, &scale);

        if i == 9 {
            HSD_JObjSetTranslateX(child, 10.0);
        }

        cur = next;
        i += 1;
    }

    Ground_801C4A08(car_gobj);
    grBigBlue_801EC6C0(gobj);

    {
        let mut max_val = (*yakumono_param).x10;
        let min_val = (*yakumono_param).x14;
        if max_val > min_val {
            let range = max_val - min_val;
            max_val = min_val + if range != 0 { HSD_Randi(range) } else { 0 };
        } else if max_val < min_val {
            let range = min_val - max_val;
            max_val += if range != 0 { HSD_Randi(range) } else { 0 };
        }

        (*car).spawn_timer = max_val as i16;
    }
}

pub unsafe extern "C" fn grBigBlue_801E6884(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E6884");
    grBigBlue_801EF424(gobj);
    Ground_UpdateMapColl(gobj);
}

/// `grBigBlue_801E68B8`.
pub unsafe extern "C" fn grBigBlue_801E68B8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E68B8");
    let gp = GET_GROUND(gobj);
    let car = addr_of_mut!((*gp).u.bigblue.x0_u.car);

    HSD_Free((*car).collision_jobjs as *mut c_void);
    (*car).collision_jobjs = null_mut();
    HSD_Free((*car).ranks as *mut c_void);
    (*car).ranks = null_mut();
}

/// `grBigBlue_801E6904` (gobj 32: the platform manager).
pub unsafe extern "C" fn grBigBlue_801E6904(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E6904");
    let gp = GET_GROUND(gobj);
    let mut jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let s = bbs(gp);

    Ground_InitMapColl(jobj, (*gp).map_id as s32);
    (*gp).x10_flags.set_b5(1);

    let mut scale = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
    HSD_JObjSetScale(jobj, &scale);

    grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);

    (*s).xD4[0] = Ground_801C3FA4(gobj, 1);
    (*s).xD4[1] = Ground_801C3FA4(gobj, 6);
    (*s).xD4[2] = Ground_801C3FA4(gobj, 11);

    (*s).data[0].index = 0;
    (*s).data[0].x1 = 0;
    (*s).data[0].x50 = 0;
    (*s).data[1].index = 1;
    (*s).data[1].x1 = 0;
    (*s).data[1].x50 = 0;
    (*s).data[2].index = 2;
    (*s).data[2].x1 = 0;
    (*s).data[2].x50 = 0;
    (*bbh(gp)).x0_w = 0;

    grBigBlue_801E8978(0, null_mut(), null_mut());

    let g = Ground_801C0498();
    scale.x = g;
    scale.y = g;
    scale.z = g;

    jobj = (*s).xD4[0];
    HSD_JObjSetScale(jobj, &scale);

    jobj = (*s).xD4[1];
    HSD_JObjSetScale(jobj, &scale);

    jobj = (*s).xD4[2];
    HSD_JObjSetScale(jobj, &scale);

    mpJointSetCb1(0, gp as *mut c_void, Some(fn_801E8560));
    mpJointSetCb1(1, gp as *mut c_void, Some(fn_801E8560));
    mpJointSetCb1(2, gp as *mut c_void, Some(fn_801E8560));
}

// ---------------------------------------------------------------------------------------------------- hover platforms
const Z3: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// `grBigBlue_LaneSpeed(idx)`: `((f32*) &grBb_803B8114)[idx] * Ground_801C0498()`.
#[inline(always)]
unsafe fn grBigBlue_LaneSpeed(idx: i32) -> f32 {
    let speeds = grBb_803B8114;
    *(&speeds as *const Vec3 as *const f32).offset(idx as isize) * Ground_801C0498()
}

/// The rotation-Z easing block of `grBigBlue_801E6C60` (it appears three times, with `target` = `data[i].xC.x`).
#[inline(always)]
unsafe fn bb_ease_rot_z(jobj: *mut HSD_JObj, target: f32) {
    if HSD_JObjGetRotationZ(jobj) < target {
        let delta = 0.017453292f32 * ((*yakumono_param).x9C * (target - HSD_JObjGetRotationZ(jobj)));
        HSD_JObjAddRotationZ(jobj, delta);
        if HSD_JObjGetRotationZ(jobj) >= target {
            HSD_JObjSetRotationZ(jobj, target);
        }
    } else {
        let delta = 0.017453292f32 * ((*yakumono_param).x9C * (target - HSD_JObjGetRotationZ(jobj)));
        HSD_JObjAddRotationZ(jobj, delta);
        if HSD_JObjGetRotationZ(jobj) <= target {
            HSD_JObjSetRotationZ(jobj, target);
        }
    }
}

/// `grBigBlue_801E6C60` (gobj 32: the manager, per frame): the three hover platforms' state machines.
pub unsafe extern "C" fn grBigBlue_801E6C60(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E6C60");
    let gp = GET_GROUND(gobj);
    let s = bbs(gp);
    let mut pos = Z3;

    for i in 0..3usize {
        let d = addr_of_mut!((*s).data[i]);
        let mut fwd = Z3;
        let mut neg_pos = Z3;
        let mut normal = Z3;
        let mut euler = Z3;
        let mut speed_val: f32;
        let mut speed3: f32;
        let mut probe_pos = Z3;
        let mut coll_y: f32 = 0.0;
        let idx = (*d).index as i8;
        let jobj = *(*s).xD4.as_ptr().offset(idx as isize);

        match (*d).x1 as i8 {
            0 => {
                if ptr_i32(grBigBlue_801E89DC(0)) != 0 {
                    let range = (*yakumono_param).x8C;
                    let rand_val = if range != 0 { HSD_Randi(range) } else { 0 };
                    (*d).x4 = (*yakumono_param).x88 + rand_val;

                    HSD_JObjGetTranslation(jobj, &mut pos);

                    (*d).x38 = pos;
                    (*d).x44.z = 0.0;
                    (*d).x44.y = 0.0;
                    (*d).x44.x = 0.0;
                    (*d).x18.z = 0.0;
                    (*d).x18.y = 0.0;
                    (*d).x18.x = 0.0;
                    (*d).x1 = 1;
                }
            }
            1 | 2 => {
                if (*d).x1 as i8 == 1 && ptr_i32(grBigBlue_801E89DC(0)) == 0 {
                    (*d).x1 = 0;
                } else if (*d).x4 <= 0 {
                    memzero(&mut pos as *mut Vec3 as *mut c_void, core::mem::size_of::<Vec3>() as ssize_t);
                    memzero(&mut neg_pos as *mut Vec3 as *mut c_void, core::mem::size_of::<Vec3>() as ssize_t);
                    pos.x = 10.0f32 + Stage_GetBlastZoneRightOffset();
                    neg_pos.x = -(10.0f32 + Stage_GetBlastZoneRightOffset());

                    let right_y = grBigBlue_801EC58C(&mut pos, null_mut(), 500.0);
                    let left_y = grBigBlue_801EC58C(&mut neg_pos, null_mut(), 500.0);

                    if right_y != -F32_MAX || left_y != -F32_MAX {
                        (*d).x8 = (*yakumono_param).x90;
                        let mut height_range = (*yakumono_param).x94 - (*yakumono_param).x90;
                        height_range = ABS(height_range);
                        let height_range_arg = melee_core::ppc::fctiwz_f32(height_range);
                        let height_rand = if height_range_arg != 0 { HSD_Randi(height_range_arg) } else { 0 };
                        (*d).x8 += height_rand as f32;
                        pos.y = right_y + (*d).x8;
                        neg_pos.y = left_y + (*d).x8;

                        if left_y == -F32_MAX {
                            (*d).x2 = (-1i32) as u8 as i8;
                        } else if right_y == -F32_MAX {
                            (*d).x2 = 1;
                        } else {
                            let mut diff = right_y - left_y;
                            diff = ABS(diff);
                            if diff < 80.0 {
                                let dir = if HSD_Randi(2) != 0 { 1 } else { -1 };
                                (*d).x2 = dir as i8;
                            } else {
                                let dir2 = if right_y < left_y { -1 } else { 1 };
                                (*d).x2 = dir2 as i8;
                            }
                        }

                        if ptr_i32(grBigBlue_801E89DC(1)) != 0 {
                            (*d).x2 = 1;
                            pos.x = -(10.0f32 + Stage_GetBlastZoneRightOffset());
                        } else if (*d).x2 as i32 == 1 {
                            pos = neg_pos;
                        }

                        let mut retries = 0;
                        let mut found;
                        loop {
                            found = grBigBlue_801E8794(jobj as *mut c_void, &mut pos, false, 2.0f32 * grBigBlue_LaneSpeed(0), 25.0) as i32;
                            if found == 0 {
                                found = grBigBlue_801EAB50(&mut pos, 0, 2.0f32 * grBigBlue_LaneSpeed(0), 25.0) as i32;
                            }
                            if found == 0 {
                                let bound = grBigBlue_801E8D04();
                                if pos.y <= bound {
                                    found = 1;
                                }
                            }
                            if found == 1 {
                                retries += 1;
                                pos.y += 25.0;
                                if retries < 3 {
                                    continue;
                                }
                            }
                            break;
                        }
                        if found != 0 {
                            pos.y = 10.0f32 + Stage_GetCamBoundsTopOffset();
                        }
                        if pos.y == -F32_MAX {
                            pos.y = 10.0f32 + Stage_GetCamBoundsTopOffset();
                        }

                        HSD_JObjSetTranslate(jobj, &pos);
                        (*d).xC.z = pos.y;

                        (*d).x38 = pos;
                        (*d).x44.x = (*yakumono_param).x98 * (*d).x2 as f32;
                        (*d).x44.z = 0.0;
                        (*d).x44.y = 0.0;
                        (*d).x18.z = 0.0;
                        (*d).x18.y = 0.0;
                        (*d).x18.x = 0.0;
                        (*d).x34 = 0;
                        (*d).x2C = 0;
                        (*d).x1 = 3;
                        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
                        (*d).x50 = 0;
                        {
                            let chance = (*yakumono_param).xB8;
                            let cr = if chance != 0 { HSD_Randi(chance) } else { 0 };
                            if cr == 0 {
                                grBigBlue_801E8A1C(i as i32);
                            }
                        }
                        let mgr = addr_of_mut!((*gp).u.bigblue.x0_u.manager);
                        (*mgr).flags = (*mgr).flags.wrapping_add(1);
                        if ptr_i32(grBigBlue_801E89DC(2)) == 0 {
                            let params = yakumono_param;
                            let cnt = (*mgr).flags;
                            if cnt as s32 >= (*params).x11C {
                                let max = (*params).x120;
                                if cnt as s32 <= max && (cnt as s32 == max || HSD_Randi(2) != 0) {
                                    grBigBlue_801E8978(2, 1usize as *mut c_void, jobj as *mut c_void);
                                }
                            }
                        }
                        if ptr_i32(grBigBlue_801E89DC(1)) == 0 {
                            let params = yakumono_param;
                            let cnt2 = (*mgr).flags;
                            if cnt2 as s32 >= (*params).xDC
                                && (*d).x2 as i32 == 1
                                && (cnt2 as s32 >= (*params).xE0 || HSD_Randi(2) != 0)
                            {
                                grBigBlue_801E8978(0, null_mut(), null_mut());
                                grBigBlue_801E8978(1, 1usize as *mut c_void, jobj as *mut c_void);
                                (*mgr).flags = 0;
                            }
                        }
                    }
                } else {
                    let mut active_count = 0;
                    for j in 0..3usize {
                        if jobj != (*s).xD4[j] {
                            let other_state = (*s).data[j].x1;
                            if other_state as i8 == 3 {
                                if ((*s).data[j].x2 as i32 == 1 && (*s).data[j].x38.x < Stage_GetCamBoundsRightOffset())
                                    || ((*s).data[j].x2 as i32 == -1 && (*s).data[j].x38.x > Stage_GetCamBoundsLeftOffset())
                                {
                                    active_count += 1;
                                }
                            } else if other_state as i8 == 1 && (*s).data[j].x4 == 0 {
                                active_count += 1;
                            }
                        }
                    }

                    if active_count <= 1 {
                        (*d).x4 = 0;
                    } else {
                        (*d).x4 -= 1;
                    }
                }
            }
            3 => {
                HSD_JObjGetTranslation(jobj, &mut pos);
                let surface_y = grBigBlue_801EC58C(&mut pos, &mut normal, 500.0);
                if surface_y == -F32_MAX {
                    normal.z = 0.0;
                    normal.x = 0.0;
                    normal.y = 1.0;
                }
                euler.y = 0.0;
                euler.x = 0.0;
                euler.z = atan2f(-normal.x, normal.y);

                speed_val = (*d).x2 as f32 * grBigBlue_LaneSpeed(idx as i32);
                fwd.x = speed_val;
                fwd.z = 0.0;
                fwd.y = 0.0;
                lbVector_ApplyEulerRotation(&mut fwd, &mut euler);
                lbVector_Add(&mut fwd, &mut pos);

                speed_val = (-((*d).x2 as i32)) as f32 * grBigBlue_LaneSpeed(idx as i32);
                neg_pos.x = speed_val;
                neg_pos.z = 0.0;
                neg_pos.y = 0.0;
                lbVector_ApplyEulerRotation(&mut neg_pos, &mut euler);
                lbVector_Add(&mut neg_pos, &mut pos);

                {
                    let sub_state = (*d).x2C;

                    if sub_state == 0 || (*d).x34 == 3 {
                        let prev_sub_state = (*d).x30;

                        if prev_sub_state == 0 && sub_state != 0 {
                            (*d).xC.y = pos.y;
                            (*d).x4 = (*yakumono_param).xB0;
                            (*d).x34 = 1;
                        }
                        (*d).xC.x = euler.z;
                        bb_ease_rot_z(jobj, (*d).xC.x);
                    } else {
                        if (*d).x30 == 0 {
                            (*d).xC.y = pos.y;
                            (*d).x4 = (*yakumono_param).xB0;
                            (*d).x34 = 1;
                        }
                        let dir_val: i32 = if (*d).x24 < (*d).x28 { 1 } else { -1 };
                        if (*d).x18.y != dir_val as f32 {
                            (*d).x4 = (*yakumono_param).xB0;
                            (*d).x34 = 1;
                        }
                        (*d).x18.y = dir_val as f32;

                        if (*d).x34 == 2 {
                            pos.y += (*d).x44.y;
                            (*d).xC.x = euler.z;
                            bb_ease_rot_z(jobj, (*d).xC.x);
                            if pos.y >= (*d).xC.y {
                                (*d).x44.y = 0.0;
                                (*d).x18.z = 0.0;
                                (*d).x18.y = 0.0;
                                (*d).x34 = 3;
                            }
                        } else if (*d).x4 <= 0 {
                            (*d).x44.y = (*yakumono_param).xAC;
                            (*d).x4 = (*yakumono_param).xB4;
                            (*d).x34 = 2;
                        } else {
                            if (*d).x24 < (*d).x28 {
                                (*d).x18.z = (*yakumono_param).xA0 * ((*d).x28 - (*d).x24);
                                if (*d).x18.z >= (*yakumono_param).xA4 {
                                    (*d).x18.z = (*yakumono_param).xA4;
                                }
                                pos.y -= (*yakumono_param).xA8;
                            } else {
                                (*d).x18.z = -(*yakumono_param).xA0 * ((*d).x24 - (*d).x28);
                                if (*d).x18.z <= -(*yakumono_param).xA4 {
                                    (*d).x18.z = -(*yakumono_param).xA4;
                                }
                                pos.y += (*yakumono_param).xA8;
                            }
                            {
                                let delta = 0.017453292f32 * (*d).x18.z;
                                HSD_JObjAddRotationZ(jobj, delta);
                            }
                            {
                                let angle_deg = 57.29578f32 * HSD_JObjGetRotationZ(jobj);
                                if pos.y < (surface_y + (*yakumono_param).x90) {
                                    if angle_deg > 30.0 {
                                        HSD_JObjSetRotationZ(jobj, 0.5235988f32);
                                    }
                                    if angle_deg < -30.0 {
                                        HSD_JObjSetRotationZ(jobj, -0.5235988f32);
                                    }
                                }
                                if angle_deg > 65.0 {
                                    HSD_JObjSetRotationZ(jobj, 1.1344640f32);
                                }
                                if angle_deg < -65.0 {
                                    HSD_JObjSetRotationZ(jobj, -1.1344640f32);
                                }
                            }
                            (*d).x4 -= 1;
                        }
                    }
                }

                {
                    let cam_top = Stage_GetCamBoundsTopOffset();
                    let cam_bot = Stage_GetCamBoundsBottomOffset();
                    let target_y;
                    let y_vel;

                    speed3 = grBigBlue_LaneSpeed(idx as i32);
                    {
                        let left_x = pos.x - (20.0f32 + speed3);
                        let right_x = pos.x + (20.0f32 + grBigBlue_LaneSpeed(idx as i32));
                        target_y = grBigBlue_801E8B84(cam_top, cam_bot, left_x, right_x);
                    }

                    probe_pos = pos;
                    let probe_y = grBigBlue_801EC58C(&mut probe_pos, null_mut(), 500.0);

                    let coll_result =
                        grBigBlue_801EACE8(jobj, &mut pos, &mut coll_y, 10.0f32 + grBigBlue_LaneSpeed(idx as i32), 16.5);

                    if coll_result == 0 || (coll_result == 1 && pos.y < coll_y) {
                        if target_y <= probe_y {
                            if probe_y == -F32_MAX {
                                (*d).xC.z = fwd.y;
                            } else {
                                (*d).xC.z = probe_y + (*d).x8;
                            }
                        } else {
                            (*d).xC.z = target_y + (*d).x8;
                        }
                    } else if coll_result == 1 {
                        (*d).xC.z = pos.y + (pos.y - coll_y);
                    }

                    if ABS(pos.y - (*d).xC.z) < 0.5 {
                        y_vel = 0.0f32;
                    } else if pos.y < (*d).xC.z {
                        let mut v = ((*d).xC.z - pos.y) / (*yakumono_param).xBC;
                        if v > (*yakumono_param).xC0 {
                            v = (*yakumono_param).xC0;
                        }
                        y_vel = v;
                    } else {
                        let mut v = ((*d).xC.z - pos.y) / (*yakumono_param).xC4;
                        if v < -(*yakumono_param).xC8 {
                            v = -(*yakumono_param).xC8;
                        }
                        y_vel = v;
                    }
                    pos.y += y_vel;
                }

                {
                    let x_vel = (*d).x44.x;
                    HSD_JObjAddTranslationX(jobj, x_vel);
                }
                HSD_JObjSetTranslateY(jobj, pos.y);

                (*d).x38 = pos;
                (*d).x30 = (*d).x2C;
                (*d).x2C = 0;
                (*d).x28 = 0.0;
                (*d).x24 = 0.0;

                if ((*d).x44.x < 0.0
                    && HSD_JObjGetTranslationX(jobj) < -((10.0f32 + Stage_GetBlastZoneRightOffset()) - 50.0f32))
                    || ((*d).x44.x > 0.0
                        && HSD_JObjGetTranslationX(jobj) > (10.0f32 + Stage_GetBlastZoneRightOffset()) - 50.0f32)
                {
                    HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
                    HSD_JObjSetRotationZ(jobj, 0.0);
                    (*d).x44.z = 0.0;
                    (*d).x44.y = 0.0;
                    (*d).x44.x = 0.0;
                    (*d).x18.z = 0.0;
                    (*d).x18.y = 0.0;
                    (*d).x18.x = 0.0;
                    (*d).xC.x = 0.0;
                    (*d).x34 = 0;
                    (*d).x2C = 0;
                    {
                        let range = (*yakumono_param).x8C;
                        (*d).x4 = ((*yakumono_param).x88 + if range != 0 { HSD_Randi(range) } else { 0 }) / 2;
                    }
                    (*d).x1 = 1;
                }
            }
            _ => {}
        }
    }

    Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grBigBlue_801E855C(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801E855C");}

/// `fn_801E8560` (mpLib_JointCollisionCallback of the three platforms): fused sites `dy*dy + dx*dx` and the two
/// `x24 += dist * (...)` / `x28 += ...` accumulations.
pub unsafe extern "C" fn fn_801E8560(
    user_data: *mut c_void,
    joint_id: i32,
    coll: *mut CollData,
    coll_x50: i32,
    ground_kind: mpLib_GroundEnum,
    delta_y: f32,
) {
    melee_core::ct!("fn_801E8560");
    let gp = user_data as *mut Ground;
    let s = bbs(gp);
    let mut coll_x50 = coll_x50;
    let mut joint_index: i32 = 0;
    let mut pos = Z3;

    if (*coll).x34_flags.b1234() as i32 != 1 && (*coll).x34_flags.b1234() as i32 != 3 {
        return;
    }

    if joint_id == 0 {
        joint_index = 0;
    }
    if joint_id == 1 {
        joint_index = 1;
    }
    if joint_id == 2 {
        joint_index = 2;
    }

    let jobj = *(*s).xD4.as_ptr().offset(joint_index as isize);

    if coll_x50 as f32 > 1000.0f32 {
        coll_x50 = 1000;
    }

    HSD_JObjGetTranslation2(jobj, &mut pos);

    let dist;
    {
        let dx = pos.x - (*coll).cur_pos.x;
        let dy = pos.y - (*coll).cur_pos.y;
        // fmuladd(dy, dy, dx * dx)
        dist = sqrtf_(fmadds(dy, dy, dx * dx));
    }

    if dist > 2.0 {
        let mut active_joint: i32 = 0;
        let mut i = 0;
        while i < 3 {
            if (*(*s).data.as_ptr().offset(i as isize)).index as i8 as i32 == joint_index {
                break;
            }
            active_joint = i + 1;
            i += 1;
        }

        let a = (*s).data.as_mut_ptr().offset(active_joint as isize);
        if pos.x < (*coll).cur_pos.x {
            // x24 += dist * (x50 / 1000): fmuladd(dist, ratio, x24)
            (*a).x24 = fmadds(dist, coll_x50 as f32 / 1000.0f32, (*a).x24);
        } else {
            (*a).x28 = fmadds(dist, coll_x50 as f32 / 1000.0f32, (*a).x28);
        }

        (*a).x2C += 1;
    }
}

// ---------------------------------------------------------------------------------------------------- manager helpers
/// `grBigBlue_801E8794`: is a platform of the manager within (`rangeX`, `rangeY`) of `pos`?
pub unsafe extern "C" fn grBigBlue_801E8794(
    exclude: *mut c_void,
    pos: *mut Vec3,
    check_secondary: bool,
    range_x: f32,
    range_y: f32,
) -> bool {
    melee_core::ct!("grBigBlue_801E8794");
    let gp = GET_GROUND(Ground_GetMapGObj(32));
    let s = bbs(gp);
    let mut result = false;

    for i in 0..3usize {
        if exclude == (*s).xD4[i] as *mut c_void {
            continue;
        }

        if (*s).data[i].x1 as i32 != 3 {
            continue;
        }

        let mut dist = HSD_JObjGetTranslationX((*s).xD4[i]) - (*pos).x;
        dist = ABS(dist);

        if dist < range_x {
            dist = HSD_JObjGetTranslationY((*s).xD4[i]) - (*pos).y;
            dist = ABS(dist);

            if dist < range_y {
                result = true;
                break;
            }
        }

        if check_secondary && (*s).data[i].x2 as i32 == -1 {
            result = true;
            break;
        }
    }

    result
}

/// `grBigBlue_801E8978`: `manager.event_data[index] = data` (and the `event_extra` slot, the 4 bytes before `data[]`).
pub unsafe extern "C" fn grBigBlue_801E8978(index: i32, data: *mut c_void, extra: *mut c_void) {
    melee_core::ct!("grBigBlue_801E8978");
    let gp = GET_GROUND(Ground_GetMapGObj(32));
    let s = bbs(gp);
    if !extra.is_null() {
        // GC +0xE0: a 4-byte slot (`pad_3` here); the native `manager.event_extra` would overlap `data[0]`
        *(addr_of_mut!((*s).pad_3) as *mut u32) = ptr_u32(extra);
    }
    *addr_of_mut!((*gp).u.bigblue.x0_u.manager.event_data).cast::<*mut c_void>().offset(index as isize) = data;
}

/// `grBigBlue_801E89DC`: `manager.event_data[arg]`.
pub unsafe extern "C" fn grBigBlue_801E89DC(arg: i32) -> *mut c_void {
    melee_core::ct!("grBigBlue_801E89DC");
    let gp = GET_GROUND(Ground_GetMapGObj(32));
    *addr_of_mut!((*gp).u.bigblue.x0_u.manager.event_data).cast::<*mut c_void>().offset(arg as isize)
}

/// `grBigBlue_801E8A1C`: drop a random item onto platform `idx`.
pub unsafe extern "C" fn grBigBlue_801E8A1C(idx: i32) {
    melee_core::ct!("grBigBlue_801E8A1C");
    let gp = GET_GROUND(Ground_GetMapGObj(32));
    let s = bbs(gp);
    let mut spawn = BobOmbRain::zeroed();
    let mut valid: [ItemKind; 5] = [0; 5];
    let platform = *(*s).xD4.as_ptr().offset(idx as isize);

    let candidates = grBb_803B8120;
    spawn.x0 = null_mut();
    spawn.x4 = null_mut();

    let mut valid_count = 0usize;
    let mut i = 0usize;
    loop {
        if it_8026D324(candidates.kinds[i]) != 0 {
            valid[valid_count] = candidates.kinds[i];
            valid_count += 1;
        }
        i += 1;
        if i >= 5 {
            break;
        }
    }

    if valid_count != 0 {
        spawn.x14 = valid[HSD_Randi(valid_count as i32) as usize];
        HSD_JObjGetTranslation2(platform, &mut spawn.x8_vec);

        if spawn.x14 == 4 {
            spawn.x8_vec.y += 8.0f32;
        } else {
            spawn.x8_vec.y += 5.0f32;
        }

        spawn.x1C.x0.set_b0(1);
        (*(*s).data.as_mut_ptr().offset(idx as isize)).x50 = ptr_i32(it_8026BE84(&mut spawn));
    }
}

/// `grBigBlue_801E8B84(right, left, bottom, top)`: highest car (lane `pos.y` in (left, right), `pos.x` in (bottom, top))
/// among the four lanes that are not in state 1. The C steps `gp = (Ground*) &gp->x20[8]` (+0x40 bytes on the PPC) from
/// lane to lane; the port indexes the lanes.
pub unsafe extern "C" fn grBigBlue_801E8B84(right: f32, left: f32, bottom: f32, top: f32) -> f32 {
    melee_core::ct!("grBigBlue_801E8B84");
    let gp = (*Ground_GetMapGObj(33)).user_data as *mut Ground;
    let car = addr_of_mut!((*gp).u.bigblue.x0_u.car);
    let mut result: f32 = -F32_MAX;

    for k in 0..4usize {
        let lane = addr_of_mut!((*car).lanes[k]);
        if (*lane).x0.x0.state() as u32 != 1 {
            if (*lane).pos.y < right && (*lane).pos.y > left {
                if (*lane).pos.x < top && (*lane).pos.x > bottom {
                    if (*lane).pos.y > result {
                        result = (*lane).pos.y;
                    }
                }
            }
        }
    }

    result
}

/// `grBigBlue_801E8D04`.
pub unsafe extern "C" fn grBigBlue_801E8D04() -> f32 {
    melee_core::ct!("grBigBlue_801E8D04");
    let val1 = Stage_GetCamBoundsRightOffset();
    let val2 = Stage_GetCamBoundsLeftOffset();
    let val3 = Stage_GetCamBoundsBottomOffset();
    let val4 = Stage_GetCamBoundsTopOffset();
    grBigBlue_801E8B84(val4, val3, val2, val1)
}

/// `*(s32*) ((u8*) manager + 0xCC)` / `+0xD0`: the low four bytes of `event_data[1]` / `event_data[2]`.
#[inline(always)]
unsafe fn mgr_xcc(gp: *mut Ground) -> *mut i32 {
    addr_of_mut!((*bbs(gp)).xCC) as *mut i32
}
#[inline(always)]
unsafe fn mgr_xd0(gp: *mut Ground) -> *mut i32 {
    addr_of_mut!((*bbs(gp)).xD0) as *mut i32
}
/// `*(s32*) ((u8*) manager + 0xC8)`: the low four bytes of `event_data[0]`.
#[inline(always)]
unsafe fn mgr_xc8(gp: *mut Ground) -> *mut i32 {
    addr_of_mut!((*bbs(gp)).xC8) as *mut i32
}

/// `grBigBlue_801E8D64` (gobj 35's on_init): the single flying platform.
pub unsafe extern "C" fn grBigBlue_801E8D64(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E8D64");
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut pos = Z3;
    let mut scale = Z3;
    let mut translate = Z3;
    let plat = addr_of_mut!((*gp).u.bigblue.x0_u.platform);

    Ground_InitMapColl(jobj, (*gp).map_id as s32);
    (*gp).x10_flags.set_b5(1);

    let g = Ground_801C0498();
    scale.x = g;
    scale.y = g;
    scale.z = g;
    HSD_JObjSetScale(jobj, &scale);

    {
        let other = Ground_GetMapGObj(32);
        let other_gp = (*other).user_data as *mut Ground;
        *mgr_xcc(other_gp) = 1;
    }

    let mut y_pos = grBigBlue_801EC58C(&mut pos, null_mut(), 500.0);
    if -F32_MAX == y_pos {
        y_pos = 0.0;
    }

    HSD_JObjSetTranslateX(jobj, 0.0);

    y_pos += (*yakumono_param).xCC;
    HSD_JObjSetTranslateY(jobj, y_pos);

    HSD_JObjSetTranslateZ(jobj, 0.0);

    (*plat).xD8 = 0.0;
    (*plat).xC8_timer = melee_core::ppc::fctiwz_f32((*yakumono_param).xD8);
    *bb_x0(gp) = 2;

    grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);

    HSD_JObjGetTranslation2(jobj, &mut pos);
    {
        let inv = 1.0f32 / Ground_801C0498();
        pos.x *= inv;
        pos.y *= inv;
        pos.z *= inv;
    }

    HSD_JObjGetTranslation2(Ground_801C2CF4(0), &mut translate);
    lbVector_Add(&mut translate, &mut pos);
    HSD_JObjSetTranslate(Ground_801C2CF4(0), &translate);

    HSD_JObjGetTranslation2(Ground_801C2CF4(1), &mut translate);
    lbVector_Add(&mut translate, &mut pos);
    HSD_JObjSetTranslate(Ground_801C2CF4(1), &translate);

    HSD_JObjGetTranslation2(Ground_801C2CF4(2), &mut translate);
    lbVector_Add(&mut translate, &mut pos);
    HSD_JObjSetTranslate(Ground_801C2CF4(2), &translate);

    HSD_JObjGetTranslation2(Ground_801C2CF4(3), &mut translate);
    lbVector_Add(&mut translate, &mut pos);
    HSD_JObjSetTranslate(Ground_801C2CF4(3), &translate);
}

/// `grBigBlue_CountCars`: the number of platforms of the manager that are not idle.
#[inline(always)]
unsafe fn grBigBlue_CountCars() -> s32 {
    let manager = GET_GROUND(Ground_GetMapGObj(32));
    let s = bbs(manager);
    let mut count = 0;
    for i in 0..3usize {
        if (*s).data[i].x1 as i8 != 0 {
            count += 1;
        }
    }
    count
}

/// `grBigBlue_801E93D8` (gobj 35, per frame): the flying platform. The C reads the `platform` view of the Ground through
/// `(u8*) gp + 0xNN`: `+0xC4` is `x0` (the state byte), `+0xC8` `xC8_timer`, `+0xCC` / `+0xD0` the float views of
/// `xCC_timer` / `xD0_timer` (target rotation and target height), `+0xD8` `xD8` (x speed). Fused sites: the two
/// `60.0f * scale + 30.0f` ranges.
pub unsafe extern "C" fn grBigBlue_801E93D8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E93D8");
    let mut pos = Z3;
    let mut fwd = Z3;
    let mut back = Z3;
    let mut normal = Z3;
    let mut euler = Z3;
    let mut check_pos = Z3;
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = GET_JOBJ(gobj);
    let plat = addr_of_mut!((*gp).u.bigblue.x0_u.platform);
    let f_cc = addr_of_mut!((*plat).xCC_timer) as *mut f32;
    let f_d0 = addr_of_mut!((*plat).xD0_timer) as *mut f32;

    HSD_JObjGetTranslation2(jobj, &mut pos);

    if grBigBlue_801EC58C(&mut pos, &mut normal, 500.0) == -F32_MAX {
        normal.z = 0.0;
        normal.x = 0.0;
        normal.y = 1.0;
    }

    euler.y = 0.0;
    euler.x = 0.0;
    euler.z = atan2f(-normal.x, normal.y);

    fwd.x = 50.0;
    fwd.z = 0.0;
    fwd.y = 0.0;
    lbVector_ApplyEulerRotation(&mut fwd, &mut euler);
    lbVector_Add(&mut fwd, &mut pos);

    back.x = -50.0;
    back.z = 0.0;
    back.y = 0.0;
    lbVector_ApplyEulerRotation(&mut back, &mut euler);
    lbVector_Add(&mut back, &mut pos);

    let state = *bb_x0(gp) as i8;

    match state {
        0 => {
            if *mgr_xcc(GET_GROUND(Ground_GetMapGObj(32))) != 0 {
                let count = grBigBlue_CountCars();

                if count <= 1 {
                    memzero(&mut pos as *mut Vec3 as *mut c_void, core::mem::size_of::<Vec3>() as ssize_t);
                    pos.x = Stage_GetBlastZoneLeftOffset() - 50.0f32;
                    let height = grBigBlue_801EC58C(&mut pos, null_mut(), 500.0);
                    if height != -F32_MAX {
                        pos.y = height + (*yakumono_param).xCC;
                        let speed = 140.0f32 * Ground_801C0498();
                        let mut collided = grBigBlue_801E8794(
                            jobj as *mut c_void,
                            &mut pos,
                            true,
                            2.0f32 * (60.0f32 * Ground_801C0498()),
                            speed,
                        ) as i32;
                        if collided == 0 {
                            collided = grBigBlue_801EAB50(&mut pos, 1, 2.0f32 * (60.0f32 * Ground_801C0498()), 25.0) as i32;
                        }
                        if collided == 0 {
                            let cam_right = Stage_GetCamBoundsRightOffset();
                            let cam_left = Stage_GetCamBoundsLeftOffset();
                            let cam_bot = Stage_GetCamBoundsBottomOffset();
                            if pos.y <= grBigBlue_801E8B84(Stage_GetCamBoundsTopOffset(), cam_bot, cam_left, cam_right) {
                                collided = 1;
                            }
                        }
                        if collided != 0 {
                            pos.y = 30.0f32 + Stage_GetCamBoundsTopOffset();
                        }
                        if pos.y == -F32_MAX {
                            OSReport(b"*** Not Set Position!(FFlyer)\n\0".as_ptr() as *const i8, &[]);
                            assert_(false, 1739, "0");
                        }
                        HSD_JObjSetTranslate(jobj, &pos);
                        *f_d0 = pos.y;
                        (*plat).xD8 = (*yakumono_param).xD0;
                        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
                        *bb_x0(gp) = 1;
                    }
                }
            }
        }
        1 | 2 | 3 => {
            match state {
                1 => {
                    if pos.x > 0.0 {
                        (*plat).xD8 = 0.0;
                        (*plat).xC8_timer = melee_core::ppc::fctiwz_f32((*yakumono_param).xD8);
                        *bb_x0(gp) = 2;
                    } else {
                        let velocity = 140.0f32 * Ground_801C0498();
                        let range_scale = Ground_801C0498();
                        // fmuladd(60, range_scale, 30)
                        let range = fmadds(60.0, range_scale, 30.0);
                        if grBigBlue_801E8794(jobj as *mut c_void, &mut pos, true, range, velocity)
                            || {
                                let a3 = fmadds(60.0, Ground_801C0498(), 30.0);
                                let next_velocity = 140.0f32 * Ground_801C0498();
                                grBigBlue_801EAB50(&mut pos, 1, a3, next_velocity)
                            }
                        {
                            (*plat).xD8 = 0.0;
                        } else {
                            (*plat).xD8 = (*yakumono_param).xD0;
                        }
                    }
                }
                2 => {
                    let timer = (*plat).xC8_timer;
                    if timer <= 0 {
                        (*plat).xD8 = (*yakumono_param).xD0;
                        *bb_x0(gp) = 3;
                        let mgp = GET_GROUND(Ground_GetMapGObj(32));
                        let ms = bbs(mgp);
                        let mut idx = 0usize;
                        let mut ctr = 3;
                        while ctr != 0 {
                            if (*ms).data[idx].x1 as i8 == 0 {
                                (*ms).data[idx].x1 = 2;
                                (*ms).data[idx].x4 = 0;
                                break;
                            }
                            idx += 1;
                            ctr -= 1;
                        }
                    } else {
                        (*plat).xC8_timer = timer - 1;
                    }
                }
                3 => {
                    if pos.x > (50.0f32 + Stage_GetBlastZoneRightOffset()) {
                        HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
                        (*plat).xD8 = 0.0;
                        {
                            let manager = GET_GROUND(Ground_GetMapGObj(32));
                            *mgr_xcc(manager) = 0;
                        }
                        {
                            let mgp2 = GET_GROUND(Ground_GetMapGObj(32));
                            if !jobj.is_null() {
                                *(addr_of_mut!((*bbs(mgp2)).pad_3) as *mut u32) = ptr_u32(jobj);
                            }
                            *mgr_xc8(mgp2) = 1;
                        }
                        *f_cc = 0.0;
                        HSD_JObjSetRotationZ(jobj, 0.0);
                        *bb_x0(gp) = 0;
                    }
                }
                _ => {}
            }
            {
                let target_z = euler.z;
                *f_cc = target_z;
                if HSD_JObjGetRotationZ(jobj) < *f_cc {
                    let delta = 0.017453292f32 * ((*yakumono_param).xD4 * (*f_cc - HSD_JObjGetRotationZ(jobj)));
                    HSD_JObjAddRotationZ(jobj, delta);
                    let target = *f_cc;
                    if HSD_JObjGetRotationZ(jobj) >= target {
                        HSD_JObjSetRotationZ(jobj, target);
                    }
                } else {
                    let delta = 0.017453292f32 * ((*yakumono_param).xD4 * (*f_cc - HSD_JObjGetRotationZ(jobj)));
                    HSD_JObjAddRotationZ(jobj, delta);
                    let target = *f_cc;
                    if HSD_JObjGetRotationZ(jobj) <= target {
                        HSD_JObjSetRotationZ(jobj, target);
                    }
                }

                let bound_y;
                {
                    let cam_right2 = Stage_GetCamBoundsRightOffset();
                    let cam_left2 = Stage_GetCamBoundsLeftOffset();
                    let cam_bot2 = Stage_GetCamBoundsBottomOffset();
                    bound_y = grBigBlue_801E8B84(Stage_GetCamBoundsTopOffset(), cam_bot2, cam_left2, cam_right2);
                }
                check_pos = pos;
                let check_h = grBigBlue_801EC58C(&mut check_pos, null_mut(), 500.0);
                if bound_y <= check_h {
                    if check_h == -F32_MAX {
                        *f_d0 = fwd.y;
                    } else {
                        *f_d0 = check_h + (*yakumono_param).xCC;
                    }
                } else {
                    *f_d0 = bound_y + (*yakumono_param).xCC;
                }

                let mut diff_y = pos.y - *f_d0;
                if diff_y < 0.0 {
                    diff_y = -diff_y;
                }
                let vy;
                if diff_y < 0.5 {
                    vy = 0.0f32;
                } else if pos.y < *f_d0 {
                    let mut v = (*f_d0 - pos.y) / (*yakumono_param).xE4;
                    if v > (*yakumono_param).xE8 {
                        v = (*yakumono_param).xE8;
                    }
                    vy = v;
                } else {
                    let mut v = (*f_d0 - pos.y) / (*yakumono_param).xEC;
                    if v < -(*yakumono_param).xF0 {
                        v = -(*yakumono_param).xF0;
                    }
                    vy = v;
                }
                pos.y += vy;

                {
                    let translate_x = (*plat).xD8;
                    HSD_JObjAddTranslationX(jobj, translate_x);
                }
                {
                    let translate_y = pos.y;
                    HSD_JObjSetTranslateY(jobj, translate_y);
                }
            }
        }
        _ => {}
    }
    Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grBigBlue_801E9F38(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801E9F38");}

/// `grBigBlue_801E9F3C` (gobj 36's on_init).
pub unsafe extern "C" fn grBigBlue_801E9F3C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801E9F3C");
    let jobj = GET_JOBJ(gobj);
    let gp = (*gobj).user_data as *mut Ground;

    Ground_InitMapColl(jobj, (*gp).map_id as s32);
    (*gp).x10_flags.set_b5(1);

    let v = Ground_801C0498();
    let v = Vec3 { x: v, y: v, z: v };

    HSD_JObjSetScale(jobj, &v);

    *bb_x0(gp) = 0;
    *mgr_xd0(GET_GROUND(Ground_GetMapGObj(32))) = 0;
    grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
}

/// `grBigBlue_801EA05C` (gobj 36, per frame): the second moving platform; `x0` is its state, `x1` its direction.
#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe extern "C" fn grBigBlue_801EA05C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801EA05C");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    let plat = addr_of_mut!((*gp).u.bigblue.x0_u.platform);
    let mut pos = Z3;
    let mut half_top = Z3;
    let mut half_bot = Z3;
    let mut normal = Z3;
    let mut euler = Z3;
    let mut surface_y: f32 = 0.0;

    HSD_JObjGetTranslation2(jobj, &mut pos);

    if grBigBlue_801EC58C(&mut pos, &mut normal, 500.0) == -F32_MAX {
        normal.z = 0.0;
        normal.x = 0.0;
        normal.y = 1.0;
    }

    euler.y = 0.0;
    euler.x = 0.0;
    euler.z = atan2f(-normal.x, normal.y);

    half_top.x = (68.0f32 * Ground_801C0498()) / 2.0;
    half_top.z = 0.0;
    half_top.y = 0.0;
    lbVector_ApplyEulerRotation(&mut half_top, &mut euler);
    lbVector_Add(&mut half_top, &mut pos);

    half_bot.x = -((68.0f32 * Ground_801C0498()) / 2.0);
    half_bot.z = 0.0;
    half_bot.y = 0.0;
    lbVector_ApplyEulerRotation(&mut half_bot, &mut euler);
    lbVector_Add(&mut half_bot, &mut pos);

    match *bb_x0(gp) as i8 {
        0 => {
            if *mgr_xd0(GET_GROUND(Ground_GetMapGObj(32))) != 0 {
                (*plat).xC8_timer = 0;
                (*plat).xD0_timer = 0;
                (*plat).xCC_timer = 0;
                *bb_x0(gp) = 1;
            }
        }
        1 => {
            let timer = (*plat).xC8_timer;
            if timer <= 0 {
                memzero(&mut pos as *mut Vec3 as *mut c_void, core::mem::size_of::<Vec3>() as ssize_t);
                memzero(&mut half_bot as *mut Vec3 as *mut c_void, core::mem::size_of::<Vec3>() as ssize_t);
                pos.x = Stage_GetBlastZoneRightOffset();
                half_bot.x = -Stage_GetBlastZoneRightOffset();

                let right_y = grBigBlue_801EC58C(&mut pos, null_mut(), 500.0);
                let left_y = grBigBlue_801EC58C(&mut half_bot, null_mut(), 500.0);

                (*plat).height_offset = (*yakumono_param).xF4;
                let r = randi(melee_core::ppc::fctiwz_f32(ABS((*yakumono_param).xF8 - (*yakumono_param).xF4)));
                (*plat).height_offset += r as f32;

                pos.y = right_y + (*plat).height_offset;
                half_bot.y = left_y + (*plat).height_offset;

                if -F32_MAX != right_y || -F32_MAX != left_y {
                    if left_y == -F32_MAX {
                        *bb_x1(gp) = (-1i32) as u8;
                    } else if -F32_MAX == right_y {
                        *bb_x1(gp) = 1;
                    } else {
                        let mut diff = right_y - left_y;
                        if diff < 0.0 {
                            diff = -diff;
                        }
                        if diff < 80.0 {
                            let dir = if HSD_Randi(2) != 0 { 1 } else { -1 };
                            *bb_x1(gp) = dir as i8 as u8;
                        } else {
                            let dir = if right_y < left_y { -1 } else { 1 };
                            *bb_x1(gp) = dir as i8 as u8;
                        }
                    }

                    if *bb_x1(gp) as i8 == 1 {
                        pos = half_bot;
                    }

                    let platform_h = 52.0f32 * Ground_801C0498();
                    let mut collision = grBigBlue_801E8794(
                        jobj as *mut c_void,
                        &mut pos,
                        false,
                        2.0f32 * (68.0f32 * Ground_801C0498()),
                        platform_h,
                    ) as i32;

                    if collision == 0 {
                        let cam_right = Stage_GetCamBoundsRightOffset();
                        let cam_left = Stage_GetCamBoundsLeftOffset();
                        let cam_bot = Stage_GetCamBoundsBottomOffset();
                        let cam_top = Stage_GetCamBoundsTopOffset();
                        let bounds_y = grBigBlue_801E8B84(cam_top, cam_bot, cam_left, cam_right);
                        if pos.y <= bounds_y {
                            collision = 1;
                        }
                    }

                    if collision == 0 {
                        if -F32_MAX == pos.y {
                            OSReport(b"*** Not Set Position!(Tyukei)\n\0".as_ptr() as *const i8, &[]);
                            assert_(false, 1994, "0");
                        }

                        HSD_JObjSetTranslate(jobj, &pos);

                        (*plat).target_y = pos.y;
                        (*plat).xEC = 0.0;
                        (*plat).velocity.z = 0.0;
                        (*plat).velocity.y = 0.0;

                        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
                        *bb_x0(gp) = 2;
                    }
                }
            } else {
                (*plat).xC8_timer = timer - 1;
            }
        }
        2 => {
            let mut pos2 = Z3;
            let mut y_check: f32 = 0.0;
            let cam_top = Stage_GetCamBoundsTopOffset();
            let cam_bot = Stage_GetCamBoundsBottomOffset();
            let left_x = pos.x - (68.0f32 * Ground_801C0498() / 2.0 + 20.0f32);
            let bounds_y = grBigBlue_801E8B84(cam_top, cam_bot, left_x, pos.x + (68.0f32 * Ground_801C0498() / 2.0 + 20.0f32));
            let vel_y;

            pos2 = pos;

            surface_y = grBigBlue_801EC58C(&mut pos2, null_mut(), 500.0);

            let ace_result = grBigBlue_801EACE8(
                jobj,
                &mut pos,
                &mut y_check,
                68.0f32 * Ground_801C0498() / 2.0 + 10.0f32,
                52.0f32 * Ground_801C0498() / 2.0 + 4.0f32,
            );

            if ace_result == 0 || (ace_result == 1 && pos.y < y_check) {
                if bounds_y <= surface_y {
                    if -F32_MAX == surface_y {
                        (*plat).target_y = half_top.y;
                    } else {
                        (*plat).target_y = surface_y + (*plat).height_offset;
                    }
                } else {
                    (*plat).target_y = bounds_y + (*plat).height_offset;
                }
            } else if ace_result == 1 {
                (*plat).target_y = pos.y + (pos.y - y_check);
            }

            {
                let mut diff = pos.y - (*plat).target_y;
                if diff < 0.0 {
                    diff = -diff;
                }
                if diff < 0.5 {
                    vel_y = 0.0f32;
                } else if pos.y < (*plat).target_y {
                    let mut v = ((*plat).target_y - pos.y) / (*yakumono_param).x124;
                    if v > (*yakumono_param).x128 {
                        v = (*yakumono_param).x128;
                    }
                    vel_y = v;
                } else {
                    let mut v = ((*plat).target_y - pos.y) / (*yakumono_param).x12C;
                    if v < -(*yakumono_param).x130 {
                        v = -(*yakumono_param).x130;
                    }
                    vel_y = v;
                }
            }

            pos.y += vel_y;
            {
                let translate_y = pos.y;
                HSD_JObjSetTranslateY(jobj, translate_y);
            }

            {
                let timer = (*plat).xCC_timer;
                if timer <= 0 {
                    let mut timer_range = (*yakumono_param).x110 - (*yakumono_param).x10C;
                    if timer_range < 0 {
                        timer_range = -timer_range;
                    }
                    (*plat).xCC_timer = randi(timer_range);
                    (*plat).xCC_timer += (*yakumono_param).x10C;

                    let speed_range = ((*yakumono_param).x100 - (*yakumono_param).xFC) / 0.1f32;
                    let mut speed = randi(melee_core::ppc::fctiwz_f32(speed_range)) as f32;
                    speed *= 0.1f32;
                    if speed == 0.0 {
                        speed = (*yakumono_param).xFC;
                    }
                    (*plat).velocity.y = speed * (*bb_x1(gp) as i8 as f32);
                } else {
                    (*plat).xCC_timer = timer - 1;
                }
            }

            {
                let timer = (*plat).xD0_timer;
                if timer <= 0 {
                    let mut timer_range = (*yakumono_param).x110 - (*yakumono_param).x10C;
                    if timer_range < 0 {
                        timer_range = -timer_range;
                    }
                    (*plat).xD0_timer = randi(timer_range);
                    (*plat).xD0_timer += (*yakumono_param).x10C;

                    let speed_range = ((*yakumono_param).x108 - (*yakumono_param).x104) / 0.1f32;
                    let mut speed = randi(melee_core::ppc::fctiwz_f32(speed_range)) as f32;
                    speed *= 0.1f32;
                    if speed == 0.0 {
                        speed = (*yakumono_param).x104;
                    }
                    if HSD_Randi(2) != 0 {
                        speed *= -1.0;
                    }
                    (*plat).velocity.z = speed;
                } else {
                    (*plat).xD0_timer = timer - 1;
                }
            }

            {
                let vel_x = (*plat).velocity.y;
                HSD_JObjAddTranslationX(jobj, vel_x);
            }
            {
                let vel_z = (*plat).velocity.z;
                HSD_JObjAddTranslationY(jobj, vel_z);
            }

            if (*bb_x1(gp) as i8 == -1 && HSD_JObjGetTranslationX(jobj) < -(Stage_GetBlastZoneRightOffset() - 50.0f32))
                || (*bb_x1(gp) as i8 == 1 && HSD_JObjGetTranslationX(jobj) > Stage_GetBlastZoneRightOffset() - 50.0f32)
            {
                HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
                (*plat).xEC = 0.0;
                (*plat).velocity.z = 0.0;
                (*plat).velocity.y = 0.0;
                *mgr_xd0(GET_GROUND(Ground_GetMapGObj(32))) = 0;
                *bb_x0(gp) = 0;
            }
        }
        _ => {}
    }

    Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grBigBlue_801EAB4C(_arg: *mut HSD_GObj) { melee_core::ct!("grBigBlue_801EAB4C");}

/// `grBigBlue_801EAB50`: is the manager's `x0 == 2` platform within range of `pos`?
pub unsafe extern "C" fn grBigBlue_801EAB50(pos: *mut Vec3, flag: s32, range_x: f32, range_y: f32) -> bool {
    melee_core::ct!("grBigBlue_801EAB50");
    let gobj = Ground_GetMapGObj(32);
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = GET_JOBJ(gobj);
    let mut result = false;

    if *bb_x0(gp) as i8 == 2 {
        let mut dist = HSD_JObjGetTranslationX(jobj) - (*pos).x;
        dist = ABS(dist);

        if dist < range_x {
            dist = HSD_JObjGetTranslationY(jobj) - (*pos).y;
            dist = ABS(dist);

            if dist < range_y {
                result = true;
            }
        }

        if flag != 0 {
            if *bb_x1(gp) as i8 == -1 {
                result = true;
            }
        }
    }

    result
}

/// `grBigBlue_801EACE8`: collision of a platform / the route platform with `point` ±(`half_range_x`, `half_range_y`).
#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe extern "C" fn grBigBlue_801EACE8(
    exclude: *mut HSD_JObj,
    point: *mut Vec3,
    out_y: *mut f32,
    half_range_x: f32,
    half_range_y: f32,
) -> s32 {
    melee_core::ct!("grBigBlue_801EACE8");
    let mut pos = Z3;
    let mut hw_left = Z3;
    let mut hw_right = Z3;
    let mut route_pos = Z3;

    let mut gobj = Ground_GetMapGObj(32);

    let left_bound = (*point).x - half_range_x;
    let right_bound = (*point).x + half_range_x;
    let top_bound = (*point).y + half_range_y;
    let bottom_bound = (*point).y - half_range_y;

    let mut best_in_range = F32_MAX;
    let mut best_above = -F32_MAX;

    let gp = (*gobj).user_data as *mut Ground;
    let s = bbs(gp);
    // `p_left++` / `p_right++` step through the x, y, z members of the Vec3
    let comp = |v: &Vec3, i: usize| -> f32 {
        match i {
            0 => v.x,
            1 => v.y,
            _ => v.z,
        }
    };

    for i in 0..3usize {
        let jobj = (*s).xD4[i];

        if exclude == jobj {
            continue;
        }

        if (*s).data[i].x1 as i32 != 3 {
            continue;
        }

        HSD_JObjGetTranslation2(jobj, &mut pos);

        hw_left = grBb_803B8114;
        let mut left_x = comp(&hw_left, i) * Ground_801C0498();
        left_x = pos.x - left_x;

        hw_right = grBb_803B8114;
        let mut right_x = comp(&hw_right, i) * Ground_801C0498();
        right_x = pos.x + right_x;

        if (right_x > left_bound && right_x < right_bound) || (left_x < right_bound && left_x > left_bound) {
            if pos.y > bottom_bound && pos.y < top_bound {
                let mut dist = (*point).y - pos.y;
                if dist < 0.0 {
                    dist = -dist;
                }
                if dist < best_in_range {
                    best_in_range = pos.y;
                }
            } else if (*point).y > pos.y && pos.y > best_above {
                best_above = pos.y;
            }
        }
    }

    // Route platform
    gobj = Ground_GetMapGObj(36);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let gp = (*gobj).user_data as *mut Ground;

    if exclude != jobj && *bb_x0(gp) as i32 == 2 {
        HSD_JObjGetTranslation2(jobj, &mut route_pos);

        let left_x = route_pos.x - 68.0f32 * Ground_801C0498() / 2.0;
        let right_x = route_pos.x + 68.0f32 * Ground_801C0498() / 2.0;

        if (right_x > left_bound && right_x < right_bound) || (left_x < right_bound && left_x > left_bound) {
            if route_pos.y > bottom_bound && route_pos.y < top_bound {
                let mut dist = (*point).y - route_pos.y;
                if dist < 0.0 {
                    dist = -dist;
                }
                if dist < best_in_range {
                    best_in_range = route_pos.y;
                }
            } else if (*point).y > route_pos.y && route_pos.y > best_above {
                best_above = route_pos.y;
            }
        }
    }

    if F32_MAX != best_in_range {
        *out_y = best_in_range;
        return 1;
    }
    if -F32_MAX != best_above {
        *out_y = best_above;
        return 2;
    }
    0
}

// ---------------------------------------------------------------------------------------------------- moving road
#[inline(always)]
unsafe fn track_entry(idx: u32) -> *mut grBb_TrackEntry {
    (addr_of_mut!(grBb_TrackEntries) as *mut grBb_TrackEntry).offset(idx as isize)
}

/// `grBigBlue_801EB004` (gobj 34's init): measure the twelve track pieces and place the first two.
pub unsafe extern "C" fn grBigBlue_801EB004(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801EB004");
    let stage_jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let gp = (*gobj).user_data as *mut Ground;
    let road = addr_of_mut!((*gp).u.bigblue.x0_u.road);
    let mut jobj: *mut HSD_JObj;
    let mut pos = Z3;
    let mut scale = Z3;
    let mut diff = Z3;
    let mut start_pos = Z3;
    let mut end_pos = Z3;
    HSD_JObjSetFlagsAll(stage_jobj, JOBJ_HIDDEN as u32);
    scale = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
    HSD_JObjSetScale(stage_jobj, &scale);

    let g = Ground_801C0498();
    scale = Vec3 { x: g, y: g, z: g };

    for i in 0..12u32 {
        let entry = track_entry(i);
        let mut start_jobj: *mut HSD_JObj = null_mut();
        let mut end_jobj: *mut HSD_JObj = null_mut();
        jobj = Ground_801C3FA4(gobj, (*entry).jobj_index as i32);

        if !jobj.is_null() {
            start_jobj = Ground_801C3FA4(gobj, (*entry).start_index as i32);
            if !start_jobj.is_null() {
                end_jobj = Ground_801C3FA4(gobj, (*entry).end_index as i32);
                if !end_jobj.is_null() {
                    HSD_JObjSetScale(jobj, &scale);

                    lb_8000B1CC(start_jobj, null_mut(), &mut start_pos);
                    lb_8000B1CC(end_jobj, null_mut(), &mut end_pos);
                    lbVector_Diff(&mut end_pos, &mut start_pos, &mut diff);

                    (*entry).delta.x = diff.x;
                    (*entry).delta.y = diff.y;
                    (*entry).delta.z = diff.z;
                    continue;
                }
            }
        }

        assert_(!jobj.is_null(), 2328, "jobj");
        assert_(!start_jobj.is_null(), 2329, "start_jobj");
        assert_(!end_jobj.is_null(), 2330, "end_jobj");
    }

    let lane = bb_lane(gp);
    (*lane).set_prev_lane(0xFFFF);

    (*lane).set_cur_lane(4);

    (*lane).set_next_lane(0);

    (*road).position.x = -1000.0f32 * Ground_801C0498();
    (*road).position.y = 10.0f32 * Ground_801C0498();

    {
        let fval = 0.0f32 * Ground_801C0498();

        (*road).position.z = fval;
        (*road).previous_position.x = fval;
        (*road).previous_position.y = fval;
        (*road).previous_position.z = fval;
        (*road).direction = 0;
        (*road).rotation = fval;

        (*lane).set_nibble_hi(0);
        (*lane).set_b1(0);
        (*lane).set_b2(0);
    }

    let mut idx = (*lane).cur_lane();
    jobj = Ground_801C3FA4(gobj, (*track_entry(idx)).jobj_index as i32);

    HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    pos.x = 0.0;
    pos.y = (*yakumono_param).x0 * Ground_801C0498();
    pos.z = 0.0;

    HSD_JObjSetTranslate(jobj, &pos);

    idx = (*lane).next_lane();
    jobj = Ground_801C3FA4(gobj, (*track_entry(idx)).jobj_index as i32);

    HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

    idx = (*lane).cur_lane();
    pos.x += (*track_entry(idx)).delta.x;
    pos.y += (*track_entry(idx)).delta.y;
    pos.z += (*track_entry(idx)).delta.z;

    HSD_JObjSetTranslate(jobj, &pos);
}

/// `grBigBlue_801EB4AC`: the road reached the end of its piece; pick the next one. Fused site: the reachability test's
/// `x8 * scale + position.y`.
pub unsafe extern "C" fn grBigBlue_801EB4AC(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801EB4AC");
    let gp = GET_GROUND(gobj);
    let road = addr_of_mut!((*gp).u.bigblue.x0_u.road);
    let lane = bb_lane(gp);
    let mut entry: *mut grBb_TrackEntry;
    let new_jobj: *mut HSD_JObj;
    let jobj: *mut HSD_JObj;
    let mut count: s32 = 0;
    let mut sp_pos = Z3;
    let delta_threshold: f32;
    let mut random_lane: s32;

    // Hide current lane's jobj
    HSD_JObjSetFlagsAll(
        Ground_801C3FA4(gobj, (*track_entry((*lane).cur_lane())).jobj_index as i32),
        JOBJ_HIDDEN as u32,
    );

    (*lane).set_prev_lane((*lane).cur_lane());

    (*lane).set_cur_lane((*lane).next_lane());

    jobj = Ground_801C3FA4(gobj, (*track_entry((*lane).next_lane())).jobj_index as i32);

    lb_8000B1CC(jobj, null_mut(), &mut sp_pos);
    (*road).position.x = sp_pos.x;

    HSD_JObjSetTranslateX(jobj, 0.0);

    (*road).position.z = sp_pos.z;

    HSD_JObjSetTranslateZ(jobj, 0.0);

    // Direction check
    {
        let direction = (*road).direction;

        if direction == 1 {
            if (*road).position.y < ((*yakumono_param).x8 * Ground_801C0498()) * 0.25f32 {
                (*road).direction = 0;
            }
        } else if direction == -1 {
            if (*road).position.y > 0.25f32 * -((*yakumono_param).x4 * Ground_801C0498()) {
                (*road).direction = 0;
            }
        } else {
            if (*road).position.y < -((*yakumono_param).x4 * Ground_801C0498()) {
                (*road).direction = -1;
            } else if (*road).position.y > (*yakumono_param).x8 * Ground_801C0498() {
                (*road).direction = 1;
            }
        }
    }

    // Random lane selection loop
    delta_threshold = 0.0;

    loop {
        let c = count;
        count += 1;
        if c > 1000 {
            assert_(false, 2414, "0");
        }

        random_lane = HSD_Randi(12);
        entry = track_entry(random_lane as u32);

        // If lane 11: check velocity requirement
        if random_lane == 11 {
            if (*road).position.y < -700.0 {
                continue;
            }
        }

        // Reject if same as current lane
        if (*lane).cur_lane() == random_lane as u32 {
            continue;
        }

        // Direction flag check
        if (*lane).b0() != 0 {
            if random_lane as u32 > 1 {
                if random_lane != 4 {
                    continue;
                }
            }
        } else {
            // Direction compatibility
            {
                let dir = (*road).direction;

                if dir == 1 {
                    if (*entry).delta.y < delta_threshold {
                        continue;
                    }
                }
                if dir == -1 {
                    if (*entry).delta.y > delta_threshold {
                        continue;
                    }
                }
            }

            // Reachability check
            let lhs = sp_pos.y + (*track_entry((*lane).cur_lane())).delta.y + (*entry).delta.y;
            // fmuladd(x8, scale, position.y)
            let rhs = fmadds((*yakumono_param).x8, Ground_801C0498(), (*road).position.y);
            if lhs < rhs {
                continue;
            }
        }
        break;
    }

    // Store new lane index
    (*lane).set_next_lane(random_lane as u32);

    // Save current jobj translate
    HSD_JObjGetTranslation2(jobj, &mut sp_pos);

    // Get new lane jobj and unhide
    new_jobj = Ground_801C3FA4(gobj, (*entry).jobj_index as i32);
    HSD_JObjClearFlagsAll(new_jobj, JOBJ_HIDDEN as u32);

    // Add current lane deltas
    sp_pos.x += (*track_entry((*lane).cur_lane())).delta.x;
    sp_pos.y += (*track_entry((*lane).cur_lane())).delta.y;
    sp_pos.z += (*track_entry((*lane).cur_lane())).delta.z;

    HSD_JObjSetTranslate(new_jobj, &sp_pos);

    // If lane 11: activate joints and reset bitfield
    if random_lane == 11 {
        mpJointListAdd(41);
        mpJointListAdd(42);
        mpJointListAdd(43);
        mpLib_80057BC0(44);
        mpLib_80057BC0(45);
        mpLib_80057BC0(46);
        mpLib_80057BC0(47);
        mpLib_80057BC0(48);
        mpLib_80057BC0(49);
        (*lane).set_nibble_hi(0);
    }

    // Collision zone activation
    match random_lane {
        0 => mpLib_80057424(35),
        1 => mpLib_80057424(50),
        6 => {
            mpLib_80057424(52);
            mpLib_80057424(53);
            mpLib_80057424(54);
        }
        7 => {
            mpLib_80057424(55);
            mpLib_80057424(56);
            mpLib_80057424(57);
        }
        8 => {
            mpLib_80057424(58);
            mpLib_80057424(59);
            mpLib_80057424(60);
        }
        9 => {
            mpLib_80057424(61);
            mpLib_80057424(62);
            mpLib_80057424(63);
        }
        10 => mpLib_80057424(51),
        5 => mpLib_80057424(33),
        2 => {
            mpLib_80057424(38);
            mpLib_80057424(39);
            mpLib_80057424(40);
        }
        3 => {
            mpLib_80057424(36);
            mpLib_80057424(37);
        }
        4 => {
            mpLib_80057424(64);
            mpLib_80057424(65);
        }
        11 => {
            for id in 41..=49 {
                mpLib_80057424(id);
            }
        }
        _ => {}
    }

    mpLib_80058560();
}

/// `grBigBlue_801EBAF8` (gobj 34, per frame): steer the road toward the ground under the stage center.
/// Fused sites (clang's `fmuladd` lowering): `3 * (x78 * s) - lateral`, `x78 * s - lateral`, `x6C * s - position.x` (all
/// negated by the enclosing `-(...)`), and the two f64 `vel.x*s + vel.y*c` / `vel.x*c - vel.y*s` (exact products, so a
/// single rounding either way).
pub unsafe extern "C" fn grBigBlue_801EBAF8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801EBAF8");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let gp = (*gobj).user_data as *mut Ground;
    let road = addr_of_mut!((*gp).u.bigblue.x0_u.road);
    let lane = bb_lane(gp);
    let mut bone_pos = Z3;
    let mut vel = Z3;
    let mut prev = Z3;
    let mut center = Z3;
    let mut normal_out = Z3;
    let mut target = Z3;
    let mut target_y: f32;
    let rot_z: f32;
    let mut angular_vel: f32;

    center.z = 0.0;
    center.x = 0.0;
    center.y = (*yakumono_param).x0 * Ground_801C0498();

    prev = (*road).position;

    let entry = track_entry((*lane).cur_lane());

    lb_8000B1CC(Ground_801C3FA4(gobj, (*entry).end_index as i32), null_mut(), &mut bone_pos);

    {
        let mut angle = (*road).rotation;
        if angle < 0.0 {
            angle = -angle;
        }
        if angle < 0.17453292f32 && bone_pos.x < -300.0f32 * Ground_801C0498() {
            grBigBlue_801EB4AC(gobj);
        }
    }

    {
        let mut angle = (*road).rotation;
        if angle < 0.0 {
            angle = -angle;
        }
        if angle < 0.17453292f32 {
            if bone_pos.x < 0.0 {
                if (*lane).b2() != 0 {
                    (*lane).set_b2(0);
                }
            } else {
                if (*lane).b2() == 0 && (*lane).cur_lane() == 0xB {
                    (*lane).set_b2(1);
                }
            }
        }
    }

    if (*lane).cur_lane() == 0xB {
        let state = (*lane).nibble_hi();
        if state == 0 {
            (*lane).set_nibble_hi(1);
        } else if state == 1 {
            mpJointListAdd(0x29);
            mpJointListAdd(0x2A);
            mpJointListAdd(0x2B);
            mpLib_80057BC0(0x2C);
            mpLib_80057BC0(0x2D);
            mpLib_80057BC0(0x2E);
            mpLib_80057BC0(0x2F);
            mpLib_80057BC0(0x30);
            mpLib_80057BC0(0x30);
            (*lane).set_nibble_hi(2);
        } else if state == 2 && (*road).rotation < -0.5235988f32 {
            mpJointListAdd(0x2C);
            mpJointListAdd(0x2D);
            mpJointListAdd(0x2E);
            (*lane).set_nibble_hi(3);
        } else if state == 3 {
            if (*road).rotation < -2.7925267f32 {
                mpLib_80057BC0(0x29);
                mpLib_80057BC0(0x2A);
                mpLib_80057BC0(0x2B);
                mpJointListAdd(0x2F);
                mpJointListAdd(0x30);
                mpJointListAdd(0x31);
                (*lane).set_nibble_hi(4);
            }
        }
    }

    if (*lane).b1() == 0 {
        target_y = grBigBlue_801EC58C(&mut center, &mut normal_out, 20.0f32 * Ground_801C0498());
    } else {
        target_y = grBigBlue_801EC58C(&mut center, &mut normal_out, 120.0f32 * Ground_801C0498());
    }

    if -F32_MAX != target_y && ((*lane).b1() == 0 || target_y > center.y) {
        let max_steer = (*yakumono_param).x70;
        if target_y > max_steer {
            target_y = max_steer;
        } else if target_y < -max_steer {
            target_y = -max_steer;
        }

        target.x = center.x;
        target.y = target_y;
        target.z = center.z;

        if (*lane).cur_lane() == 0xB && (*lane).b2() != 0 {
            rot_z = -atan2f(-normal_out.x, normal_out.y);
        } else {
            rot_z = -(*road).rotation;
        }

        (*lane).set_b1(0);
    } else {
        if (*lane).b1() == 0 {
            lbVector_Diff(&mut (*road).previous_position, &mut (*road).position, &mut (*road).drift);
            let drift_speed = (*yakumono_param).x7C * Ground_801C0498();
            {
                let s = sinf((*road).rotation);
                (*road).lateral_drift = s * drift_speed;
            }
            (*lane).set_b1(1);
        }

        if -F32_MAX != target_y {
            // -(3 * (x78 * s) - lateral)
            let inner = (*yakumono_param).x78 * Ground_801C0498();
            (*road).lateral_drift = -fmsubs(3.0, inner, (*road).lateral_drift);
        } else {
            // -(x78 * s - lateral)
            (*road).lateral_drift = -fmsubs((*yakumono_param).x78, Ground_801C0498(), (*road).lateral_drift);
        }

        target.x = center.x;
        target.y = center.y + (*road).lateral_drift;
        target.z = center.z;

        rot_z = -(*road).rotation;
    }

    angular_vel = rot_z;
    if rot_z > 0.0 {
        angular_vel = 0.0;
    } else {
        let mut abs_rot = rot_z;
        if abs_rot < 0.0 {
            abs_rot = -abs_rot;
        }
        if abs_rot > 0.0034906585f32 {
            angular_vel *= (*yakumono_param).x74;
        }
    }

    vel.x = (*road).position.x;
    vel.y = (*road).position.y;
    vel.z = (*road).position.z;
    lbVector_Sub(&mut vel, &mut target);

    if angular_vel != 0.0 {
        let s = sinf(angular_vel);
        let c = cosf(angular_vel);
        let new_y = (vel.x as f64).fmadd(s as f64, vel.y as f64 * c as f64);
        vel.x = (vel.x as f64).fmadd(c as f64, -(vel.y as f64 * s as f64)) as f32;
        vel.y = new_y as f32;
    }

    lbVector_Add(&mut vel, &mut center);

    (*road).position.x = vel.x;
    (*road).position.y = vel.y;
    (*road).position.z = vel.z;

    {
        let old_rot = (*road).rotation;
        (*road).rotation = old_rot + angular_vel;
        if (*lane).b2() == 0 && old_rot > 0.0 {
            if (*road).rotation <= 0.0 {
                (*road).rotation = 0.0;
            }
        }
    }

    {
        let rot = (*road).rotation;
        if rot as f64 > M_PI {
            (*road).rotation = (rot as f64 - M_TAU) as f32;
        } else if (rot as f64) < -M_PI {
            let mut wrapped = rot as f64;
            wrapped += M_TAU;
            (*road).rotation = wrapped as f32;
        }
    }

    // -(x6C * s - position.x)
    (*road).position.x = -fmsubs((*yakumono_param).x6C, Ground_801C0498(), (*road).position.x);

    {
        let lat_adj = ((*entry).delta.z / (*entry).delta.x) * ((*yakumono_param).x6C * Ground_801C0498()) * 0.6f32;

        if bone_pos.z > lat_adj {
            (*road).position.z -= lat_adj;
        } else if bone_pos.z < -lat_adj {
            (*road).position.z += lat_adj;
        } else {
            (*road).position.z -= bone_pos.z;
        }
    }

    HSD_JObjSetTranslateX(jobj, (*road).position.x);
    HSD_JObjSetTranslateY(jobj, (*road).position.y);
    HSD_JObjSetTranslateZ(jobj, (*road).position.z);

    HSD_JObjSetRotationZ(jobj, (*road).rotation);

    lbVector_Rotate(&mut vel, 4, -(*road).rotation);
    vel.z = 0.0;
    vel.x = 0.0;
    lbVector_Rotate(&mut vel, 4, (*road).rotation);

    {
        let car = Ground_GetMapGObj(2);
        if !car.is_null() {
            let car_jobj = (*car).hsd_obj as *mut HSD_JObj;
            if !car_jobj.is_null() {
                HSD_JObjSetTranslate(car_jobj, &vel);
                HSD_JObjSetRotationZ(car_jobj, (*road).rotation);
            }
        }
    }

    {
        let car = Ground_GetMapGObj(1);
        if !car.is_null() {
            let car_jobj = (*car).hsd_obj as *mut HSD_JObj;
            if !car_jobj.is_null() {
                HSD_JObjSetTranslate(car_jobj, &vel);
                HSD_JObjSetRotationZ(car_jobj, (*road).rotation);
            }
        }
    }

    (*road).previous_position = prev;
}

/// `grBigBlue_801EC58C`: the highest ground hit by a vertical probe through `pos` (±`half_height`), over the 32 lines.
pub unsafe extern "C" fn grBigBlue_801EC58C(pos: *mut Vec3, normal_out: *mut Vec3, half_height: f32) -> f32 {
    melee_core::ct!("grBigBlue_801EC58C");
    let mut hit_pos = Z3;
    let mut hit_normal = Z3;
    let local_ids = grBb_803B8134;
    let mut max_y: f32 = -F32_MAX;

    let x1 = (*pos).x;
    let x2 = (*pos).x;
    let top = (*pos).y + half_height;
    let bottom = (*pos).y - half_height;

    for i in 0..32usize {
        if mpCheckMultiple(x1, top, x2, bottom, &mut hit_pos, null_mut(), null_mut(), &mut hit_normal, 1, -1, local_ids.v[i]) != 0 {
            if max_y < hit_pos.y {
                max_y = hit_pos.y;
                if !normal_out.is_null() {
                    *normal_out = hit_normal;
                }
            }
        }
    }

    max_y
}

// ---------------------------------------------------------------------------------------------------- cars
type CarLane = grBigBlue_CarLane;

#[inline(always)]
unsafe fn car_of(gp: *mut Ground) -> *mut grBigBlue_CarVars {
    addr_of_mut!((*gp).u.bigblue.x0_u.car)
}
#[inline(always)]
unsafe fn lane_at(gp: *mut Ground, i: i32) -> *mut CarLane {
    (*car_of(gp)).lanes.as_mut_ptr().offset(i as isize)
}
/// `lanes[i].state` (the PPC reads it as `(byte >> 2) & 0x3F`).
#[inline(always)]
unsafe fn lane_state(gp: *mut Ground, i: i32) -> u32 {
    (*lane_at(gp, i)).x0.x0.state() as u32
}
#[inline(always)]
unsafe fn lane_set_state(gp: *mut Ground, i: i32, v: i32) {
    (*lane_at(gp, i)).x0.x0.set_state(v as u8);
}
#[inline(always)]
unsafe fn lane_direction(gp: *mut Ground, i: i32) -> u8 {
    (*lane_at(gp, i)).x0.x0.direction()
}
#[inline(always)]
unsafe fn lane_slot(gp: *mut Ground, i: i32) -> u16 {
    (*lane_at(gp, i)).x0.x0_1.collision_slot()
}
#[inline(always)]
unsafe fn lane_set_slot(gp: *mut Ground, i: i32, v: i32) {
    (*lane_at(gp, i)).x0.x0_1.set_collision_slot(v as u16);
}

/// `grBigBlue_801EC6C0_inline`: set up car `car_idx` on collision line `line_idx`. Fused sites: `lerp + 0.5 * -(...)`
/// (twice) and `x0 * scale + lerp`.
unsafe fn grBigBlue_801EC6C0_inline(gp: *mut Ground, car_idx: s32, line_idx: s32) {
    let car = car_of(gp);
    let lane = lane_at(gp, car_idx);

    lane_set_slot(gp, car_idx, line_idx);

    (*lane).x0.x0.set_direction(0);

    let mut params = yakumono_param;
    let mut scale = Ground_801C0498();
    let mut lerp = (*params).x1C as f32 * scale;
    scale = lerp / ((*params).x18 + 1) as f32;
    lerp = (car_idx + 1) as f32 * scale;
    scale = Ground_801C0498();
    params = yakumono_param;
    (*lane).pos.x = fmadds(0.5, -((*params).x1C as f32 * scale), lerp);

    scale = Ground_801C0498();
    params = yakumono_param;
    lerp = (*params).x2C * scale;

    scale = Ground_801C0498();
    params = yakumono_param;
    (*lane).pos.y = fmadds((*params).x0, scale, lerp);

    (*lane).pos.z = 0.0;

    params = yakumono_param;
    scale = Ground_801C0498();
    lerp = (*params).x1C as f32 * scale;
    scale = lerp / ((*params).x18 + 1) as f32;
    lerp = (car_idx + 1) as f32 * scale;
    scale = Ground_801C0498();
    params = yakumono_param;
    (*lane).target = fmadds(0.5, -((*params).x1C as f32 * scale), lerp);

    (*lane).delta = 0.0;

    (*lane).gravity = 0.0;
    (*lane).height = 0.0;
    (*lane).velocity = 0.0;
    (*lane).accel = 0.0;

    (*lane).rotation = (2.0f64 * M_PI * HSD_Randf() as f64) as f32;

    scale = Ground_801C0498();
    params = yakumono_param;
    (*lane).amplitude = (*params).x34 * scale;

    (*lane).angular_velocity = 0.0;

    params = yakumono_param;
    let hi = melee_core::ppc::fctiwz_f32((*params).x60);
    let mut lo = melee_core::ppc::fctiwz_f32((*params).x5C);
    if lo > hi {
        let diff = lo - hi;
        let random = if diff != 0 { HSD_Randi(diff) } else { 0 };
        lo = hi + random;
    } else if lo < hi {
        let diff = hi - lo;
        let random = if diff != 0 { HSD_Randi(diff) } else { 0 };
        lo += random;
    }
    (*lane).threshold = lo;

    Ground_801C5440(gp, car_idx, lbl_803E3010[HSD_Randi(4) as usize]);

    (*lane).alpha = 1.0;

    {
        let idx = line_idx as isize;
        HSD_JObjClearFlagsAll(*(*car).collision_jobjs.offset(idx), JOBJ_HIDDEN as u32);

        HSD_JObjSetTranslate(*(*car).collision_jobjs.offset(idx), &(*lane).pos);
    }

    *(*car).ranks.offset(line_idx as isize) = 1;
}

/// `grBigBlue_801EC6C0`: first-time setup of the 30 car lines and up to 4 cars.
pub unsafe extern "C" fn grBigBlue_801EC6C0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801EC6C0");
    let gp = (*gobj).user_data as *mut Ground;
    let car = car_of(gp);

    for i in 0..30usize {
        mpJointSetCb1(lbl_803E2DFC[i] as i32, gp as *mut c_void, Some(fn_801EF60C));
        HSD_JObjSetFlagsAll(*(*car).collision_jobjs.add(i), JOBJ_HIDDEN as u32);
        let val: u8 = if HSD_Randi(2) != 0 { 0 } else { 2 };
        *(*car).ranks.add(i) = val;
    }

    if (*yakumono_param).x18 == 0 {
        (*yakumono_param).x18 = 1;
    }

    for car_idx in 0..4 {
        if car_idx < (*yakumono_param).x18 {
            lane_set_state(gp, car_idx, 4);

            let mut i;
            let mut line_idx;
            loop {
                line_idx = HSD_Randi(30);
                i = 0;
                while i < car_idx {
                    if lane_slot(gp, i) as i32 == line_idx {
                        break;
                    }
                    i += 1;
                }
                if i == car_idx {
                    break;
                }
            }

            grBigBlue_801EC6C0_inline(gp, car_idx, line_idx);
        } else {
            lane_set_state(gp, car_idx, 1);
        }
    }
}

/// Is the lane in a state other than 1 / 7 / 8 (idle / driving off)?
#[inline(always)]
unsafe fn lane_active(gp: *mut Ground, i: i32) -> bool {
    let st = lane_state(gp, i);
    st != 1 && st != 7 && st != 8
}

/// `grBigBlue_801ECB50` (gobj 33, per frame): keep the free / reserved line balance, count the cars, and spawn one when
/// the timer runs out. The C's "find the closest car" block only stores `byte` back into the state byte (its `rlwimi`
/// is under `#ifdef MUST_MATCH`), so it has no effect in this build.
pub unsafe extern "C" fn grBigBlue_801ECB50(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801ECB50");
    let gp = GET_GROUND(gobj);
    let car = car_of(gp);
    let ground_gobj = gobj;
    let mut active_count: s32 = 0;
    let mut free_count: s32 = 0;
    let mut reserved_count: s32 = 0;

    // Count free (0) and reserved (2) lanes
    for i in 0..30usize {
        let val = *(*car).ranks.add(i);
        if val == 0 {
            free_count += 1;
        } else if val == 2 {
            reserved_count += 1;
        }
    }

    // Balance lanes
    if free_count == 0 {
        let mut pick = if reserved_count != 0 { HSD_Randi(reserved_count) } else { 0 };
        for j in 0..30usize {
            let p = (*car).ranks.add(j);
            if *p == 2 {
                pick -= 1;
                if pick < 0 {
                    *p = 0;
                    free_count += 1;
                    break;
                }
            }
        }
    } else if reserved_count == 0 {
        let mut pick = if free_count != 0 { HSD_Randi(free_count) } else { 0 };
        for j in 0..30usize {
            let p = (*car).ranks.add(j);
            if *p == 0 {
                pick -= 1;
                if pick < 0 {
                    *p = 2;
                    reserved_count += 1;
                    break;
                }
            }
        }
    }

    // Count active cars (not in state 1, 7, or 8)
    if lane_active(gp, 0) {
        active_count = 1;
    }
    if lane_active(gp, 1) {
        active_count += 1;
    }
    if lane_active(gp, 2) {
        active_count += 1;
    }
    if lane_active(gp, 3) {
        active_count += 1;
    }

    if active_count == 1 {
        (*car).spawn_timer = (*car).spawn_timer.wrapping_sub(1);
    }

    // Timer-based car spawn
    {
        let timer = (*car).spawn_timer;
        (*car).spawn_timer = (timer as i32 - 1) as i16;
        if timer < 0 {
            active_count = -1;
            if lane_state(gp, 0) == 1 {
                active_count = 0;
            } else if lane_state(gp, 1) == 1 {
                active_count = 1;
            } else if lane_state(gp, 2) == 1 {
                active_count = 2;
            } else if lane_state(gp, 3) == 1 {
                active_count = 3;
            }

            if active_count != -1 {
                let mut right_count = 0;
                let mut left_count = 0;
                let direction: s32;

                let st = lane_state(gp, 0);
                if st == 7 {
                    right_count = 1;
                } else if st == 8 {
                    left_count = 1;
                }
                for k in 1..4 {
                    let st = lane_state(gp, k);
                    if st == 7 {
                        right_count += 1;
                    } else if st == 8 {
                        left_count += 1;
                    }
                }

                if right_count == 0 && left_count == 0 {
                    direction = if HSD_Randi(2) != 0 { 1 } else { -1 };
                } else if right_count != 0 {
                    direction = -1;
                } else if left_count != 0 {
                    direction = 1;
                } else {
                    direction = 0;
                }

                if direction == 1 {
                    let mut pick = if free_count != 0 { HSD_Randi(free_count) } else { 0 };
                    for pos in 0..30usize {
                        if *(*car).ranks.add(pos) == 0 {
                            pick -= 1;
                            if pick < 0 {
                                if grBigBlue_801EE398(ground_gobj, active_count, 5) != 0 {
                                    (*car).spawn_timer = bb_spawn_timer() as i16;
                                }
                                return;
                            }
                        }
                    }
                } else if direction == -1 {
                    let mut pick = if reserved_count != 0 { HSD_Randi(reserved_count) } else { 0 };
                    for pos in 0..30usize {
                        if *(*car).ranks.add(pos) == 2 {
                            pick -= 1;
                            if pick < 0 {
                                if grBigBlue_801EE398(ground_gobj, active_count, 6) != 0 {
                                    (*car).spawn_timer = bb_spawn_timer() as i16;
                                }
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The spawn-timer roll shared by the two branches of `grBigBlue_801ECB50`: a value between `x10` and `x14`.
#[inline(always)]
unsafe fn bb_spawn_timer() -> s32 {
    let mut tmin = (*yakumono_param).x10;
    let tmax = (*yakumono_param).x14;
    if tmin > tmax {
        let diff = tmin - tmax;
        let random = if diff != 0 { HSD_Randi(diff) } else { 0 };
        tmin = tmax + random;
    } else if tmin < tmax {
        let diff = tmax - tmin;
        let random = if diff != 0 { HSD_Randi(diff) } else { 0 };
        tmin += random;
    }
    tmin
}

/// `grBigBlue_801ED694` (one car, per frame): lane ranking, speed, bounce, heading and grounding of lane `lane`.
/// Fused sites: see the per-statement comments (15 in all).
#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe extern "C" fn grBigBlue_801ED694(gobj: *mut HSD_GObj, lane: s32) {
    melee_core::ct!("grBigBlue_801ED694");
    let gp = (*gobj).user_data as *mut Ground;
    let car = car_of(gp);
    let mut sp_vec = Z3;
    let mut rank_factor: f32;
    let mut ground_y: f32 = 0.0;
    let mut heading_osc: f32;
    let heading_val: f32;
    let l = lane_at(gp, lane);

    // Check and handle lane status
    {
        let status = grBigBlue_801EDF44(gobj, lane);
        if status != 0 {
            grBigBlue_801EE398(gobj, lane, status);
        }
    }

    if !grBigBlue_801EEF00(gobj, lane) {
        return;
    }

    let jobj = *(*car).collision_jobjs.add(lane_slot(gp, lane) as usize);

    let mut f31_rot = HSD_JObjGetRotationZ(jobj);

    {
        let state = lane_state(gp, lane);
        let target: f32;

        if state != 7 && state != 8 {
            let mut active = 0;
            let mut behind = 0;

            for idx in 0..4 {
                if lane_active(gp, idx) {
                    active += 1;
                    if idx != lane && (*lane_at(gp, idx)).pos.x < (*l).pos.x {
                        behind += 1;
                    }
                }
            }

            // Compute ranking-based target
            rank_factor = (behind + 1) as f32 * ((*yakumono_param).x1C as f32 * Ground_801C0498() / (active + 1) as f32);
            // fmuladd(0.5, -(x1C * scale), rank_factor)
            target = fmadds(0.5, -((*yakumono_param).x1C as f32 * Ground_801C0498()), rank_factor);
        } else {
            target = (*l).target;
        }

        // Convergence step for position index
        {
            let current = (*l).target;
            let diff = target - current;

            if diff < -0.5 {
                (*l).target = current - 0.5;
            } else if diff > 0.5 {
                (*l).target += 0.5;
            } else {
                (*l).target = target;
            }
        }
    }

    // Velocity update: vel += accel
    {
        (*l).velocity += (*l).accel;

        // Clamp velocity to +-max_speed*scale
        {
            if (*l).velocity > (*yakumono_param).x24 * Ground_801C0498() {
                (*l).velocity = (*yakumono_param).x24 * Ground_801C0498();
            } else if (*l).velocity < -((*yakumono_param).x24 * Ground_801C0498()) {
                (*l).velocity = -((*yakumono_param).x24 * Ground_801C0498());
            }
        }

        // Position update
        {
            (*l).delta += (*l).velocity;
            (*l).pos.x = (*l).target + (*l).delta;
        }
    }

    // Height/bounce simulation
    if lane_direction(gp, lane) == 0 {
        // gravity += x48 * scale
        (*l).gravity = fmadds((*yakumono_param).x48, Ground_801C0498(), (*l).gravity);
        (*l).height += (*l).gravity;

        if (*l).height > 0.0 {
            (*l).height = 0.0;
            (*l).gravity = 0.0;
        } else {
            if (*l).height < -((*yakumono_param).x4C * Ground_801C0498()) {
                (*l).height = -((*yakumono_param).x4C * Ground_801C0498());
            }
        }
    } else {
        (*l).height = 0.0;
        (*l).gravity = 0.0;
    }

    // Heading direction from velocity
    if lane_direction(gp, lane) == 0 {
        let vel = (*l).velocity;

        if vel > 0.0 {
            ground_y = (*yakumono_param).x24 * Ground_801C0498();
            rank_factor = (*yakumono_param).x34 * Ground_801C0498();
            // fmuladd(x38, scale, -rank_factor)
            rank_factor = (*l).velocity * (fmsubs((*yakumono_param).x38, Ground_801C0498(), rank_factor) / ground_y);
            // fmuladd(x34, scale, rank_factor)
            heading_val = fmadds((*yakumono_param).x34, Ground_801C0498(), rank_factor);
        } else if vel < 0.0 {
            ground_y = (*yakumono_param).x24 * Ground_801C0498();
            rank_factor = (*yakumono_param).x34 * Ground_801C0498();
            rank_factor = (*l).velocity * (fmsubs((*yakumono_param).x30, Ground_801C0498(), rank_factor) / ground_y);
            heading_val = fmadds((*yakumono_param).x34, Ground_801C0498(), rank_factor);
        } else {
            heading_val = (*yakumono_param).x34 * Ground_801C0498();
        }
    } else {
        heading_val = 0.0;
    }

    // Heading target interpolation: amplitude += x3C * (heading_val - amplitude)
    (*l).amplitude = fmadds((*yakumono_param).x3C, heading_val - (*l).amplitude, (*l).amplitude);

    // Rotation angle update: rotation += 0.017453292 * x40
    (*l).rotation = fmadds(0.017453292f32, (*yakumono_param).x40, (*l).rotation);
    if (*l).rotation as f64 > M_TAU {
        (*l).rotation = ((*l).rotation as f64 - M_TAU) as f32;
    }

    heading_osc = (*l).amplitude * sinf((*l).rotation);

    // Compute ground reference level: (pos.y - x2C * scale) - height - heading_osc
    rank_factor = fnmsubs(((*yakumono_param).x2C), Ground_801C0498(), (*l).pos.y) - (*l).height - heading_osc;

    // Collision/grounding check
    'converge: {
        if lane_direction(gp, lane) != 0 {
            // Grounded path
            ground_y = grBigBlue_801EC58C(&mut (*l).pos, &mut sp_vec, 500.0f32);

            if -F32_MAX != ground_y && ground_y > rank_factor {
                if (*l).angular_velocity < 0.0 {
                    (*l).x0.x0.set_direction(0);
                    // goto heading_converge: skip collision/grounding
                    break 'converge;
                }
            }

            if -F32_MAX != ground_y {
                // angular_velocity -= 3 * (x80 * scale)
                let inner = (*yakumono_param).x80 * Ground_801C0498();
                (*l).angular_velocity = fnmsubs(3.0, inner, (*l).angular_velocity);
            } else {
                // angular_velocity -= x80 * scale
                (*l).angular_velocity = fnmsubs(((*yakumono_param).x80), Ground_801C0498(), (*l).angular_velocity);
            }

            // Lateral position += angular velocity
            (*l).pos.y += (*l).angular_velocity;

            // Track reference adjustment
            {
                let map_gobj = Ground_GetMapGObj(34);

                assert_(!map_gobj.is_null(), 3255, "map_gobj");
                let gp2 = (*map_gobj).user_data as *mut Ground;
                assert_(!gp2.is_null(), 3256, "gp2");
                let road = addr_of_mut!((*gp2).u.bigblue.x0_u.road);
                (*l).pos.y += (*road).position.y - (*road).previous_position.y;
            }
        } else {
            // Non-grounded path
            let s = Ground_801C0498();
            ground_y = grBigBlue_801EC58C(&mut (*l).pos, &mut sp_vec, 20.0f32 * s);

            if -F32_MAX != ground_y {
                // heading_osc + (height + (x2C * scale + ground_y))
                (*l).pos.y = heading_osc + ((*l).height + fmadds((*yakumono_param).x2C, Ground_801C0498(), ground_y));
            } else {
                (*l).x0.x0.set_direction(1);
                rank_factor = (*yakumono_param).x84 * Ground_801C0498();
                heading_osc = sinf(f31_rot);
                (*l).angular_velocity = heading_osc * rank_factor;
            }
        }
    }

    // heading_converge: update rotation, smooth toward target angle
    if -F32_MAX != ground_y && lane_direction(gp, lane) == 0 {
        let angle = atan2f(-sp_vec.x, sp_vec.y);
        // f31_rot += x50 * (angle - f31_rot)
        f31_rot = fmadds((*yakumono_param).x50, angle - f31_rot, f31_rot);
    } else {
        // f31_rot += x54 * (0.017453292 * x58 - f31_rot)
        let inner = fmsubs(0.017453292f32, (*yakumono_param).x58, f31_rot);
        f31_rot = fmadds((*yakumono_param).x54, inner, f31_rot);
    }

    // Update jobj transform
    HSD_JObjSetTranslate(jobj, &(*l).pos);
    HSD_JObjSetRotationZ(jobj, f31_rot);
}

/// `grBigBlue_801EDF44`: what should lane `index` do next? Returns the state to switch to (0 = none).
/// Fused sites: `blast += x68 * scale` / `blast -= x68 * scale`.
pub unsafe extern "C" fn grBigBlue_801EDF44(gobj: *mut HSD_GObj, index: s32) -> s32 {
    melee_core::ct!("grBigBlue_801EDF44");
    let mut result: s32 = 0;
    let gp = (*gobj).user_data as *mut Ground;
    let ln = lane_at(gp, index);

    match lane_state(gp, index) {
        1 => {}
        9 => {
            if 0.0f32 == (*ln).alpha {
                result = 1;
            }
        }
        7 => {
            let mut blast = Stage_GetBlastZoneRightOffset();
            let scale = Ground_801C0498();

            blast = fmadds((*yakumono_param).x68, scale, blast);
            if (*ln).pos.x > blast {
                if 0.0f32 != (*ln).alpha {
                    result = 9;
                } else {
                    result = 1;
                }
            }
        }
        8 => {
            let mut blast = Stage_GetBlastZoneLeftOffset();
            let scale = Ground_801C0498();

            blast = fnmsubs(((*yakumono_param).x68), scale, blast);
            if (*ln).pos.x < blast {
                if 0.0f32 != (*ln).alpha {
                    result = 9;
                } else {
                    result = 1;
                }
            }
        }
        2 | 3 | 4 | 5 | 6 | 10 => {
            let old = (*ln).threshold;
            (*ln).threshold = old.wrapping_sub(1);
            if old < 0 {
                while result == 0 {
                    let rnd = HSD_Randi(4);

                    match rnd {
                        0 | 1 | 2 => {
                            result = 4;
                        }
                        3 => {
                            let mut right_cnt = 0;
                            let mut left_cnt = 0;
                            let mut right_10 = 0;
                            let mut left_10 = 0;

                            for i in 0..4 {
                                if i == index {
                                    continue;
                                }

                                let st = lane_state(gp, i);

                                if st == 1 {
                                    continue;
                                }
                                if st == 7 {
                                    continue;
                                }
                                if st == 8 {
                                    continue;
                                }

                                if (*lane_at(gp, i)).pos.x > (*ln).pos.x {
                                    if st == 10 {
                                        right_10 += 1;
                                    } else {
                                        right_cnt += 1;
                                    }
                                } else if (*lane_at(gp, i)).pos.x < (*ln).pos.x {
                                    if st == 10 {
                                        left_10 += 1;
                                    } else {
                                        left_cnt += 1;
                                    }
                                }
                            }

                            if right_cnt != 0 {
                                if left_cnt != 0 {
                                    if right_10 != 0 {
                                        if left_10 == 0 {
                                            result = 8;
                                        }
                                    } else if left_10 != 0 {
                                        result = 7;
                                    } else {
                                        result = if HSD_Randi(2) != 0 { 7 } else { 8 };
                                    }
                                } else {
                                    result = 8;
                                }
                            } else if left_cnt != 0 {
                                result = 7;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Override: check other platforms heading toward self
            for i in 0..4 {
                if i == index {
                    continue;
                }

                let st = lane_state(gp, i);

                if st == 7 {
                    if (*lane_at(gp, i)).pos.x < (*ln).pos.x {
                        result = 7;
                        continue;
                    }
                }
                if st == 8 {
                    if (*lane_at(gp, i)).pos.x > (*ln).pos.x {
                        result = 8;
                    }
                }
            }

            // Final threshold check
            if (*ln).pos.y < -2000.0 {
                result = 1;
            }
        }
        _ => {}
    }

    result
}

/// `grBigBlue_801EE398` (+ its inlined body): switch lane `arg1` into state `arg2`. Fused sites: `x68 * scale + blast`,
/// `-(x68 * scale - blast)` and `x2C * scale + pos.y` (twice).
pub unsafe extern "C" fn grBigBlue_801EE398(gobj: *mut HSD_GObj, arg1: s32, arg2: s32) -> s32 {
    melee_core::ct!("grBigBlue_801EE398");
    let mut result: s32 = 0;
    let gp = (*gobj).user_data as *mut Ground;
    let car = car_of(gp);
    let ln = lane_at(gp, arg1);
    let mut pos = Z3;
    let mut slot: s32 = 0;

    match arg2 {
        1 => {
            HSD_JObjSetFlagsAll(*(*car).collision_jobjs.add(lane_slot(gp, arg1) as usize), JOBJ_HIDDEN as u32);

            if (*ln).pos.x > 0.0 {
                *(*car).ranks.add(lane_slot(gp, arg1) as usize) = 0;
            } else {
                *(*car).ranks.add(lane_slot(gp, arg1) as usize) = 2;
            }
            result = 1;
            lane_set_state(gp, arg1, arg2);
        }

        10 => {
            (*ln).threshold = 0x14;
            result = 1;
            lane_set_state(gp, arg1, arg2);
        }

        2 | 3 | 4 => {
            (*ln).threshold = rand_range(melee_core::ppc::fctiwz_f32((*yakumono_param).x5C), melee_core::ppc::fctiwz_f32((*yakumono_param).x60));
            result = 1;
            lane_set_state(gp, arg1, arg2);
        }

        5 | 6 => {
            if arg2 == 5 {
                let blast = Stage_GetBlastZoneRightOffset();
                // x68 * scale + blast
                pos.x = fmadds((*yakumono_param).x68, Ground_801C0498(), blast);
            } else {
                let blast = Stage_GetBlastZoneLeftOffset();
                // -(x68 * scale - blast)
                pos.x = -fmsubs((*yakumono_param).x68, Ground_801C0498(), blast);
            }
            pos.z = 0.0;
            pos.y = 0.0;
            pos.y = grBigBlue_801EC58C(&mut pos, null_mut(), 1000.0);

            if -F32_MAX != pos.y {
                let mut count = 0;
                for j in 0..30usize {
                    if *(*car).ranks.add(j) == 0 {
                        count += 1;
                    }
                }

                if count != 0 {
                    slot = 0;
                    let mut pick = randi(count);

                    while slot < 30 {
                        if *(*car).ranks.add(slot as usize) == 0 {
                            pick -= 1;
                            if pick < 0 {
                                break;
                            }
                        }
                        slot += 1;
                    }

                    lane_set_slot(gp, arg1, slot);
                    (*ln).x0.x0.set_direction(0);

                    (*ln).pos.x = pos.x;
                    // x2C * scale + pos.y
                    (*ln).pos.y = fmadds((*yakumono_param).x2C, Ground_801C0498(), pos.y);
                    (*ln).pos.z = 0.0;
                    (*ln).target = pos.x;
                    (*ln).delta = 0.0;

                    (*ln).gravity = 0.0;
                    (*ln).height = 0.0;
                    (*ln).velocity = 0.0;
                    (*ln).accel = 0.0;
                    (*ln).rotation = (M_TAU * HSD_Randf() as f64) as f32;
                    (*ln).amplitude = (*yakumono_param).x34 * Ground_801C0498();
                    (*ln).angular_velocity = 0.0;

                    (*ln).threshold = rand_range(melee_core::ppc::fctiwz_f32((*yakumono_param).x5C), melee_core::ppc::fctiwz_f32((*yakumono_param).x60));

                    *(*car).ranks.add(slot as usize) = 1;
                    (*ln).alpha = 0.0;

                    let sfx_id = lbl_803E3010[HSD_Randi(4) as usize];
                    Ground_801C5440(gp, slot, sfx_id);

                    Ground_801C5630(gp, arg1, (*ln).alpha);

                    {
                        HSD_JObjClearFlagsAll(*(*car).collision_jobjs.add(slot as usize), JOBJ_HIDDEN as u32);
                        let jobj = *(*car).collision_jobjs.add(slot as usize);
                        HSD_JObjSetTranslate(jobj, &(*ln).pos);
                    }

                    result = 1;
                    lane_set_state(gp, arg1, arg2);
                }
            }
        }

        7 | 8 => {
            (*ln).threshold = 0x3E8;
            result = 1;
            lane_set_state(gp, arg1, arg2);
        }

        9 => {
            lane_set_state(gp, arg1, arg2);
        }

        _ => {}
    }

    result
}

/// `grBigBlue_801EEF00`: lane `index`'s acceleration for this frame (false if the lane is idle). Fused sites: the
/// predicted-position expressions of state 4.
pub unsafe extern "C" fn grBigBlue_801EEF00(gobj: *mut HSD_GObj, index: s32) -> bool {
    melee_core::ct!("grBigBlue_801EEF00");
    let gp = (*gobj).user_data as *mut Ground;
    let c = lane_at(gp, index);

    match lane_state(gp, index) {
        1 => return false,

        10 => {
            let mut abs_speed;
            let mut scale;
            let norm;

            if (*c).pos.x > 0.0 {
                abs_speed = (*c).velocity;
                if abs_speed < 0.0 {
                    abs_speed = -abs_speed;
                }
                scale = Ground_801C0498();
                norm = abs_speed / ((*yakumono_param).x28 * scale);
                if (*c).threshold as f32 > norm {
                    scale = Ground_801C0498();
                    (*c).accel = -((*yakumono_param).x28 * scale);
                } else {
                    scale = Ground_801C0498();
                    (*c).accel = (*yakumono_param).x28 * scale;
                }
            } else {
                abs_speed = (*c).velocity;
                if abs_speed < 0.0 {
                    abs_speed = -abs_speed;
                }
                scale = Ground_801C0498();
                norm = abs_speed / ((*yakumono_param).x28 * scale);
                if (*c).threshold as f32 > norm {
                    scale = Ground_801C0498();
                    (*c).accel = (*yakumono_param).x28 * scale;
                } else {
                    scale = Ground_801C0498();
                    (*c).accel = -((*yakumono_param).x28 * scale);
                }
            }
        }

        2 => {
            let mut abs_speed = (*c).velocity;
            if abs_speed < 0.0 {
                abs_speed = -abs_speed;
            }
            let mut scale = Ground_801C0498();
            let norm = abs_speed / ((*yakumono_param).x28 * scale);
            let mut done = false;
            if (*c).threshold as f32 > norm {
                if (*c).pos.x < Stage_GetCamBoundsRightOffset() {
                    scale = Ground_801C0498();
                    (*c).accel = (*yakumono_param).x28 * scale;
                    done = true;
                }
            }
            if !done {
                scale = Ground_801C0498();
                (*c).accel = -((*yakumono_param).x28 * scale);
            }
        }

        3 => {
            let mut abs_speed = (*c).velocity;
            if abs_speed < 0.0 {
                abs_speed = -abs_speed;
            }
            let mut scale = Ground_801C0498();
            let norm = abs_speed / ((*yakumono_param).x28 * scale);
            let mut done = false;
            if (*c).threshold as f32 > norm {
                if (*c).pos.x > Stage_GetCamBoundsLeftOffset() {
                    scale = Ground_801C0498();
                    (*c).accel = -((*yakumono_param).x28 * scale);
                    done = true;
                }
            }
            if !done {
                scale = Ground_801C0498();
                (*c).accel = (*yakumono_param).x28 * scale;
            }
        }

        4 => {
            let target = (*c).target;
            let mut abs_speed = (*c).velocity;
            if abs_speed < 0.0 {
                abs_speed = -abs_speed;
            }
            let mut scale = Ground_801C0498();
            let total = (*yakumono_param).x28 * scale;
            let norm = abs_speed / total;

            if target > (*c).pos.x {
                if (*c).velocity < 0.0 {
                    scale = Ground_801C0498();
                    (*c).accel = (*yakumono_param).x28 * scale;
                } else {
                    scale = Ground_801C0498();
                    // velocity * norm + pos.x - norm * (0.5 * (x28 * scale) * norm)
                    let head = fmadds((*c).velocity, norm, (*c).pos.x);
                    let predicted = fnmsubs(norm, 0.5f32 * ((*yakumono_param).x28 * scale) * norm, head);

                    if predicted < target {
                        scale = Ground_801C0498();
                        (*c).accel = (*yakumono_param).x28 * scale;
                    } else {
                        scale = Ground_801C0498();
                        (*c).accel = -((*yakumono_param).x28 * scale);
                    }
                }
            } else {
                if (*c).velocity > 0.0 {
                    scale = Ground_801C0498();
                    (*c).accel = -((*yakumono_param).x28 * scale);
                } else {
                    scale = Ground_801C0498();
                    let head = fmadds((*c).velocity, norm, (*c).pos.x);
                    let predicted = fnmsubs(norm, 0.5f32 * -((*yakumono_param).x28 * scale) * norm, head);

                    if predicted < target {
                        scale = Ground_801C0498();
                        (*c).accel = (*yakumono_param).x28 * scale;
                    } else {
                        scale = Ground_801C0498();
                        (*c).accel = -((*yakumono_param).x28 * scale);
                    }
                }
            }
        }

        5 | 6 => {
            (*c).accel = 0.0;
            (*c).alpha += 1.0f32 / 60.0f32;
            if (*c).alpha > 1.0 {
                (*c).alpha = 1.0;
            }
            Ground_801C5630(gp, index, (*c).alpha);
        }

        7 => {
            let scale = Ground_801C0498();
            (*c).accel = (*yakumono_param).x28 * scale;
        }

        8 => {
            let scale = Ground_801C0498();
            (*c).accel = -((*yakumono_param).x28 * scale);
        }

        9 => {
            (*c).alpha -= 1.0f32 / 60.0f32;
            if (*c).alpha < 0.0 {
                (*c).alpha = 0.0;
            }
            Ground_801C5630(gp, index, (*c).alpha);
        }

        _ => {}
    }

    true
}

/// `grBigBlue_801EF424` (gobj 33, per frame): run the car logic, then push overlapping cars apart (up to 10 passes).
pub unsafe extern "C" fn grBigBlue_801EF424(gobj: *mut HSD_GObj) {
    melee_core::ct!("grBigBlue_801EF424");
    let gp = (*gobj).user_data as *mut Ground;

    grBigBlue_801ECB50(gobj);

    for init_i in 0..4 {
        grBigBlue_801ED694(gobj, init_i);
    }

    let zero: f32 = 0.0;
    let mut changed = 1;
    let mut k = 0;

    while k < 10 && changed != 0 {
        changed = 0;

        for i in 0..4 {
            if lane_state(gp, i) == 1 {
                continue;
            }

            for j in 0..4 {
                if i == j {
                    continue;
                }

                if lane_state(gp, j) == 1 {
                    continue;
                }

                let li = lane_at(gp, i);
                let lj = lane_at(gp, j);
                let mut diff = (*li).pos.x - (*lj).pos.x;

                let abs_diff = if diff < zero { -diff } else { diff };

                if abs_diff < Ground_801C0498() * (*yakumono_param).x20 as f32 {
                    let adjustment = if diff > zero {
                        Ground_801C0498() * (*yakumono_param).x20 as f32
                    } else {
                        -(Ground_801C0498() * (*yakumono_param).x20 as f32)
                    };

                    diff -= adjustment;
                    changed = 1;
                    diff *= 0.5;

                    (*li).delta -= diff;
                    (*li).pos.x -= diff;
                    (*lj).delta += diff;
                    (*lj).pos.x += diff;
                }
            }
        }

        k += 1;
    }
}

/// `fn_801EF60C` (mpLib_JointCollisionCallback of the 30 car lines): a hit on line `joint_id` kicks the car riding it.
pub unsafe extern "C" fn fn_801EF60C(
    user_data: *mut c_void,
    joint_id: i32,
    coll: *mut CollData,
    coll_x50: i32,
    ground_kind: mpLib_GroundEnum,
    delta_y: f32,
) {
    melee_core::ct!("fn_801EF60C");
    let gp = user_data as *mut Ground;
    let mut car_num: i32 = 0;

    if (*coll).x34_flags.b1234() as i32 != 1 {
        return;
    }
    if ground_kind != 1 {
        return;
    }
    let table = addr_of!(lbl_803E2DFC) as *const s16;

    while car_num < 30 {
        if joint_id == *table.offset(car_num as isize) as i32 {
            break;
        }
        car_num += 1;
    }

    assert_(car_num != 30, 0xED9, "car_num!=Gr_Fzero_Car_Max");

    let params = yakumono_param;

    for i in 0..4 {
        if lane_slot(gp, i) as i32 == car_num {
            (*lane_at(gp, i)).gravity = -(*coll).x50 * (*params).x44;
        }
    }
}

/// `grBigBlue_801EF7D8`.
pub unsafe extern "C" fn grBigBlue_801EF7D8(pos: *mut Vec3) {
    melee_core::ct!("grBigBlue_801EF7D8");
    let gobj = Ground_GetMapGObj(34);

    if !gobj.is_null() && !(*gobj).user_data.is_null() {
        (*pos).x = -10.0;
        (*pos).y = 0.0;
        (*pos).z = 0.0;
    } else {
        (*pos).z = 0.0;
        (*pos).y = 0.0;
        (*pos).x = 0.0;
    }
}

/// `grBigBlue_801EF844`: is `line_id` one of the Big Blue car / platform lines?
pub unsafe extern "C" fn grBigBlue_801EF844(line_id: enum_t) -> bool {
    melee_core::ct!("grBigBlue_801EF844");
    if (*ext::stage_info()).grkind == Gr_Kind_BigBlue && line_id != -1 {
        let joint = mpJointFromLine(line_id);

        let result = joint == 33 || (35..=65).contains(&joint);

        if result {
            return true;
        }
    }
    false
}

/// `fn_801EFB9C` (render callback of the backdrop).
pub unsafe extern "C" fn fn_801EFB9C(gobj: *mut HSD_GObj, pass: isize) {
    melee_core::ct!("fn_801EFB9C");
    if gm_GetDbPauseFlag(1) != 0 || gm_GetDbPauseFlag(2) != 0 || Camera_8003010C() != 0 {
        return;
    }
    grDisplay_801C5DB0(gobj, pass as isize);
}

/// `grBigBlue_801EFC14` (on_check_shadow_render).
pub unsafe extern "C" fn grBigBlue_801EFC14(a: *mut Vec3, _b: i32, jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grBigBlue_801EFC14");
    let mut vec = Z3;
    lb_8000B1CC(jobj, null_mut(), &mut vec);
    if (*a).y > vec.y { 1 } else { 0 }
}

// ---- folded from grbigblue.rs (consolidation steps 21/22) ----
/// grBigBlue_801E59CC_fp (gr/grbigblue.c:279)
    pub unsafe extern "C" fn grBigBlue_801E59CC_fp() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

// ---- folded from grbigblue.rs (consolidation steps 21/22) ----
/// grBigBlue_801E59F0 (gr/grbigblue.c:284)
    pub extern "C" fn grBigBlue_801E59F0() -> i32 { 0 }

/// grBigBlue_801E5B10 (gr/grbigblue.c:337)
    pub extern "C" fn grBigBlue_801E5B10(a0: *mut HSD_GObj) -> i32 { 0 }

/// grBigBlue_801E6114 (gr/grbigblue.c:375)
    pub extern "C" fn grBigBlue_801E6114(a0: *mut HSD_GObj) -> i32 { 0 }

/// grBigBlue_801E61BC (gr/grbigblue.c:403)
    pub extern "C" fn grBigBlue_801E61BC(a0: *mut HSD_GObj) -> i32 { 0 }

/// grBigBlue_801E6288 (gr/grbigblue.c:434)
    pub extern "C" fn grBigBlue_801E6288(a0: *mut HSD_GObj) -> i32 { 0 }

/// grBigBlue_801E6354 (gr/grbigblue.c:463)
    pub extern "C" fn grBigBlue_801E6354(a0: *mut HSD_GObj) -> i32 { 0 }

/// grBigBlue_801E687C (gr/grbigblue.c:581)
    pub extern "C" fn grBigBlue_801E687C(a0: *mut HSD_GObj) -> i32 { 0 }

/// grBigBlue_801E6C58 (gr/grbigblue.c:650)
    pub extern "C" fn grBigBlue_801E6C58(a0: *mut HSD_GObj) -> i32 { 0 }

/// grBigBlue_801E93D0 (gr/grbigblue.c:1534)
    pub extern "C" fn grBigBlue_801E93D0(a0: *mut HSD_GObj) -> i32 { 0 }

/// grBigBlue_801EA054 (gr/grbigblue.c:1848)
    pub extern "C" fn grBigBlue_801EA054(a0: *mut HSD_GObj) -> i32 { 0 }

/// grBigBlue_801EFC0C (gr/grbigblue.c:4576)
    pub extern "C" fn grBigBlue_801EFC0C(a0: i32) -> *mut melee_types::lbColl_80008D30_arg1 { core::ptr::null_mut() }
