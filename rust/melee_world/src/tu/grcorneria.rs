//! src/melee/gr/grcorneria.c: Corneria (the Great Fox / Arwing battle stage), shared with Venom through the
//! "smash taunt" talk sequence (`smashTaunt_*`, `grCorneria_801E2*`).
//!
//! Fused sites (tools/rust-port/fma_sites.py, all f32 -> `fmadds`): the two ship speeds
//! `-35.0f * Ground_801C0498() - xD0` (`801DD9A8`), `a * randf + x0` (`801E1970`), the three
//! `50.0f * Ground_801C0498() - 250.0f` and the two `speed * t + -0.0025f * t * t` of `801E2228`, the two sites of
//! `801E2EEC` and `Ground_801C0498() * -35.0f - xD0` of `801E2FCC`. Everything else is unfused (the
//! `3200.0f * Ground_801C0498() / 2 + c` forms put a division between the multiply and the add).
//!
//! Native adaptations (same result as the PPC build): `gp->u.arwing.xDC` read off the ship's ground
//! (`801DDDA8`) is the ship's `corneria.offset_x` on the PPC (both at +0x18) but not in the native layout, so the
//! port reads `offset_x`; `grCn_803E214C[xF4][group_offset]` with `group_offset == -1` reads the previous row's
//! second element (C), written as a flat index.
//! State-oracle checked against the C (tests/gro_grcorneria.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use super::grinl::*;
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_core::fp::fmadds;
use melee_types::*;

/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::hook;
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::{stage_info, HSD_JObjSetMtxDirty, HSD_JObjSetMtxDirtySub, HSD_JObjSetupMatrixSub};
    hook!(CAMERA_8003010C, Camera_8003010C, () -> i32);
    hook!(CAMERA_80030154, Camera_80030154, () -> i32);
    hook!(CAMERA_80030178, Camera_80030178, () -> i32);
    hook!(CAMERA_GETTRANSFORMINTEREST, Camera_GetTransformInterest, (a0: *mut Vec));
    hook!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, (a0: CmQuakeKind, a1: *mut Vec));
    hook!(FTLIB_80086644, ftLib_80086644, (gobj: *mut HSD_GObj, pos: *mut Vec));
    hook!(GM_8017E7E0, gm_8017E7E0, () -> i32);
    hook!(GM_801883C0, gm_801883C0, () -> f32);
    hook!(GM_GETDBPAUSEFLAG, gm_GetDbPauseFlag, (bit: i32) -> i32);
    hook!(GOBJ_SETUPGXLINK, GObj_SetupGXLink, (gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32));
    pub static mut GRANIME_801C787C: Option<fn(*mut HSD_GObj, i32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C787C(a0: *mut HSD_GObj, a1: i32, a2: u32) {
        match unsafe { GRANIME_801C787C } { Some(f) => f(a0, a1, a2), None => { unsafe { let _ = crate::tu::granime::grAnime_801C787C(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRANIME_801C7980: Option<fn(*mut HSD_GObj, i32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C7980(a0: *mut HSD_GObj, a1: i32, a2: u32) {
        match unsafe { GRANIME_801C7980 } { Some(f) => f(a0, a1, a2), None => { unsafe { let _ = crate::tu::granime::grAnime_801C7980(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRANIME_801C7FF8: Option<fn(*mut HSD_GObj, i32, i32, i32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) {
        match unsafe { GRANIME_801C7FF8 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { unsafe { let _ = crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } return; } }
    }
    pub static mut GRANIME_801C8098: Option<fn(*mut HSD_GObj, i32, i32, i32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C8098(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) {
        match unsafe { GRANIME_801C8098 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { unsafe { let _ = crate::tu::granime::grAnime_801C8098(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } return; } }
    }
    pub static mut GRANIME_801C8138: Option<fn(*mut HSD_GObj, enum_t, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C8138(a0: *mut HSD_GObj, a1: enum_t, a2: i32) {
        match unsafe { GRANIME_801C8138 } { Some(f) => f(a0, a1, a2), None => { unsafe { let _ = crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRANIME_801C83D0: Option<fn(*mut HSD_GObj, i32, enum_t) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: enum_t) -> i32 {
        match unsafe { GRANIME_801C83D0 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRANIME_801C8780: Option<fn(*mut HSD_GObj, u32, u32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C8780(a0: *mut HSD_GObj, a1: u32, a2: u32, a3: f32, a4: f32) {
        match unsafe { GRANIME_801C8780 } { Some(f) => f(a0, a1, a2, a3, a4), None => { unsafe { let _ = crate::tu::granime::grAnime_801C8780(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } return; } }
    }
    pub static mut GRDATFILES_GETARCHIVE: Option<fn() -> *mut UnkArchiveStruct> = None;
    #[allow(non_snake_case)]
    pub fn grDatFiles_GetArchive() -> *mut UnkArchiveStruct {
        match unsafe { GRDATFILES_GETARCHIVE } { Some(f) => f(), None => { return unsafe { crate::tu::grdatfiles::grDatFiles_GetArchive() as _ }; } }
    }
    pub static mut GRLIB_801C96F8: Option<fn(i32, i32, *mut Vec) -> *mut HSD_Generator> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C96F8(a0: i32, a1: i32, a2: *mut Vec) -> *mut HSD_Generator {
        match unsafe { GRLIB_801C96F8 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::grlib::grLib_801C96F8(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRLIB_801C9808: Option<fn(i32, i32, *mut HSD_JObj) -> *mut HSD_Generator> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C9808(a0: i32, a1: i32, a2: *mut HSD_JObj) -> *mut HSD_Generator {
        match unsafe { GRLIB_801C9808 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::grlib::grLib_801C9808(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRMATERIAL_801C87D0: Option<fn(*mut HSD_JObj, u32)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C87D0(a0: *mut HSD_JObj, flags: u32) {
        match unsafe { GRMATERIAL_801C87D0 } { Some(f) => f(a0, flags), None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C87D0(a0 as _, flags as _); } return; } }
    }
    pub static mut GRMATERIAL_801C8858: Option<fn(*mut HSD_JObj, u32)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8858(a0: *mut HSD_JObj, flags: u32) {
        match unsafe { GRMATERIAL_801C8858 } { Some(f) => f(a0, flags), None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8858(a0 as _, flags as _); } return; } }
    }
    pub static mut GRMATERIAL_801C8A04: Option<fn(*mut HSD_JObj, u32)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8A04(a0: *mut HSD_JObj, flags: u32) {
        match unsafe { GRMATERIAL_801C8A04 } { Some(f) => f(a0, flags), None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8A04(a0 as _, flags as _); } return; } }
    }
    pub static mut GRMATERIAL_801C8CFC: Option<fn(i32, i32, *mut Ground, *mut HSD_JObj, Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8CFC(a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, arg4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, arg5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, arg6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj {
        match unsafe { GRMATERIAL_801C8CFC } { Some(f) => f(a0, a1, a2, a3, arg4, arg5, arg6), None => { return unsafe { crate::tu::grmaterial::grMaterial_801C8CFC(a0 as _, a1 as _, a2 as _, a3 as _, arg4 as _, arg5 as _, arg6 as _) as _ }; } }
    }
    pub static mut GRMATERIAL_801C8DE0: Option<fn(*mut HSD_GObj, f32, f32, f32, f32, f32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8DE0(gobj: *mut HSD_GObj, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32) {
        match unsafe { GRMATERIAL_801C8DE0 } { Some(f) => f(gobj, a1, a2, a3, a4, a5, a6, a7), None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8DE0(gobj as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _); } return; } }
    }
    pub static mut GRMATERIAL_801C8E08: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8E08(gobj: *mut HSD_GObj) {
        match unsafe { GRMATERIAL_801C8E08 } { Some(f) => f(gobj), None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8E08(gobj as _); } return; } }
    }
    pub static mut GRMATERIAL_801C8E28: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8E28(gobj: *mut HSD_GObj) {
        match unsafe { GRMATERIAL_801C8E28 } { Some(f) => f(gobj), None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8E28(gobj as _); } return; } }
    }
    hook!(GRMATERIAL_801C9604, grMaterial_801C9604, (bg: *mut HSD_GObj, a1: *mut c_void, a2: i32));
    pub static mut GRVENOM_80206BF0: Option<fn(i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grVenom_80206BF0(a0: i32) -> i32 {
        match unsafe { GRVENOM_80206BF0 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grvenom::grVenom_80206BF0(a0 as _) as _ }; } }
    }
    pub static mut GRVENOM_80206CB0: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn grVenom_80206CB0(a0: i32) {
        match unsafe { GRVENOM_80206CB0 } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::grvenom::grVenom_80206CB0(a0 as _); } return; } }
    }
    pub static mut GRZAKOGENERATOR_801CA394: Option<fn(*mut grZakoGenerator_Spawn, i32, grZakoGenerator_SpawnFunc, f32) -> *mut grZakoGenerator_Config> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CA394(a0: *mut grZakoGenerator_Spawn, a1: i32, a2: grZakoGenerator_SpawnFunc, a3: f32) -> *mut grZakoGenerator_Config {
        match unsafe { GRZAKOGENERATOR_801CA394 } { Some(f) => f(a0, a1, a2, a3), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CA394(a0 as _, a1 as _, a2 as _, a3 as _) as _ }; } }
    }
    pub static mut GRZAKOGENERATOR_801CA43C: Option<fn(*mut grZakoGenerator_Config, *mut HSD_JObj, f32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CA43C(a0: *mut grZakoGenerator_Config, a1: *mut HSD_JObj, a2: f32) -> i32 {
        match unsafe { GRZAKOGENERATOR_801CA43C } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CA43C(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRZAKOGENERATOR_801CAE04: Option<fn(*mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj {
        match unsafe { GRZAKOGENERATOR_801CAE04 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ }; } }
    }
    pub static mut GROUND_801C055C: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C055C(a0: *mut GXColor) {
        match unsafe { GROUND_801C055C } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C055C(a0 as _); } return; } }
    }
    pub static mut GROUND_801C0574: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C0574(a0: *mut GXColor) {
        match unsafe { GROUND_801C0574 } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C0574(a0 as _); } return; } }
    }
    pub static mut GROUND_801C058C: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C058C(a0: *mut GXColor) {
        match unsafe { GROUND_801C058C } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C058C(a0 as _); } return; } }
    }
    pub static mut GROUND_801C05A4: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C05A4(a0: *mut GXColor) {
        match unsafe { GROUND_801C05A4 } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C05A4(a0 as _); } return; } }
    }
    pub static mut GROUND_801C05BC: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C05BC(a0: *mut GXColor) {
        match unsafe { GROUND_801C05BC } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C05BC(a0 as _); } return; } }
    }
    pub static mut GROUND_801C05EC: Option<fn(*mut GXColor)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C05EC(a0: *mut GXColor) {
        match unsafe { GROUND_801C05EC } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::ground::Ground_801C05EC(a0 as _); } return; } }
    }
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
    pub static mut GROUND_801C11AC: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C11AC(gobj: *mut HSD_GObj) {
        match unsafe { GROUND_801C11AC } { Some(f) => f(gobj), None => { unsafe { let _ = crate::tu::ground::Ground_801C11AC(gobj as _); } return; } }
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
    pub static mut GROUND_801C4E70: Option<fn(*mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj, *mut HSD_JObj)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C4E70(a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj) {
        match unsafe { GROUND_801C4E70 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { unsafe { let _ = crate::tu::ground::Ground_801C4E70(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } return; } }
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
    pub static mut GROUND_801C5544: Option<fn(*mut Ground, s32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5544(a0: *mut Ground, a1: s32) {
        match unsafe { GROUND_801C5544 } { Some(f) => f(a0, a1), None => { unsafe { let _ = crate::tu::ground::Ground_801C5544(a0 as _, a1 as _); } return; } }
    }
    pub static mut GROUND_801C54DC: Option<fn(*mut Ground, s32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C54DC(gp: *mut Ground, i: s32) -> i32 {
        match unsafe { GROUND_801C54DC } { Some(f) => f(gp, i), None => { return unsafe { crate::tu::ground::Ground_801C54DC(gp as _, i as _) as _ }; } }
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
    hook!(HSD_GOBJGXLINK_80390908, HSD_GObjGXLink_80390908, (a0: *mut HSD_GObj, gx_link: u8, priority: u8));
    hook!(HSD_GOBJPLINKHEAD, HSD_GObjPLinkHead, (link: i32) -> *mut HSD_GObj);
    hook!(HSD_GOBJ_SETUPPROC, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc);
    hook!(HSD_JOBJANIMALL, HSD_JObjAnimAll, (a0: *mut HSD_JObj));
    hook!(HSD_JOBJCLEARFLAGS, HSD_JObjClearFlags, (a0: *mut HSD_JObj, flags: u32));
    hook!(HSD_JOBJCLEARFLAGSALL, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, flags: u32));
    hook!(HSD_JOBJREMOVEANIMALL, HSD_JObjRemoveAnimAll, (a0: *mut HSD_JObj));
    hook!(HSD_JOBJSETFLAGS, HSD_JObjSetFlags, (a0: *mut HSD_JObj, flags: u32));
    hook!(HSD_JOBJSETFLAGSALL, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, flags: u32));
    hook!(HSD_PADMASTERSTATUS, HSD_PadMasterStatus, () -> *mut HSD_PadStatus);
    hook!(HSD_RANDF, HSD_Randf, () -> f32);
    hook!(HSD_RANDI, HSD_Randi, (max_val: s32) -> s32);
    pub static mut HSD_SISLIB_803A5ACC: Option<fn(i32, i32, f32, f32, f32, f32, f32) -> *mut HSD_Text> = None;
    #[allow(non_snake_case)]
    pub fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text {
        match unsafe { HSD_SISLIB_803A5ACC } { Some(f) => f(a0, a1, a2, a3, a4, a5, a6), None => { return unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ }; } }
    }
    pub static mut HSD_SISLIB_803A5F50: Option<fn(s32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_SisLib_803A5F50(a0: s32) {
        match unsafe { HSD_SISLIB_803A5F50 } { Some(f) => f(a0), None => { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A5F50(a0 as _); } return; } }
    }
    pub static mut HSD_SISLIB_803A611C: Option<fn(i32, *mut HSD_GObj, u16, u8, u8, u8, u8, u32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 {
        match unsafe { HSD_SISLIB_803A611C } { Some(f) => f(a0, a1, a2, a3, a4, a5, a6, a7), None => { return unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A611C(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _) as _ }; } }
    }
    pub static mut HSD_SISLIB_803A6368: Option<fn(*mut HSD_Text, i32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) {
        match unsafe { HSD_SISLIB_803A6368 } { Some(f) => f(a0, a1), None => { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A6368(a0 as _, a1 as _); } return; } }
    }
    hook!(HSD_SISLIB_804D1124, HSD_SisLib_804D1124, () -> *mut *mut SIS);
    hook!(HSD_8039D580, hsd_8039D580, (a0: *mut HSD_JObj));
    pub static mut IFSTATUS_802F6898: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn ifStatus_802F6898() {
        match unsafe { IFSTATUS_802F6898 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::ifstatus::ifStatus_802F6898(); } return; } }
    }
    pub static mut IFSTATUS_802F68F0: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn ifStatus_802F68F0() {
        match unsafe { IFSTATUS_802F68F0 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::ifstatus::ifStatus_802F68F0(); } return; } }
    }
    hook!(ITEM_80268E5C, Item_80268E5C, (gobj: *mut HSD_GObj, msid: enum_t, flags: Item_StateChangeFlags));
    hook!(IT_802E72E0, it_802E72E0, (a0: *mut HSD_GObj, a1: *mut HSD_JObj, a2: s32, a3: f32, a4: f32) -> *mut HSD_GObj);
    hook!(IT_802E7654, it_802E7654, (a0: *mut HSD_GObj, a1: *mut HSD_JObj, a2: *mut Vec, a3: s32, a4: s32, a5: f32) -> *mut HSD_GObj);
    hook!(IT_802EAF34, it_802EAF34, (a0: *mut HSD_GObj, a1: *mut Vec, a2: i32) -> *mut HSD_GObj);
    pub static mut LBAUDIOAX_800237A8: Option<fn(enum_t, i32, i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn lbAudioAx_800237A8(sfx_id: enum_t, sfx_vol: i32, sfx_pan: i32) -> i32 {
        match unsafe { LBAUDIOAX_800237A8 } { Some(f) => f(sfx_id, sfx_vol, sfx_pan), None => { return unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(sfx_id as _, sfx_vol as _, sfx_pan as _) as _ }; } }
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec, *mut Vec)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec, pos1: *mut Vec) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    pub static mut LB_8000C2F8: Option<fn(*mut HSD_JObj, *mut HSD_JObj)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000C2F8(a0: *mut HSD_JObj, a1: *mut HSD_JObj) {
        match unsafe { LB_8000C2F8 } { Some(f) => f(a0, a1), None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000C2F8(a0 as _, a1 as _); } return; } }
    }
    pub static mut LB_800115F4: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn lb_800115F4() {
        match unsafe { LB_800115F4 } { Some(f) => f(), None => { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800115F4(); } return; } }
    }
    pub static mut LB_80011A50: Option<fn(*mut Vec, i32, f32, f32, f32, f32, f32, f32, f32) -> *mut lb_80011A50_t> = None;
    #[allow(non_snake_case)]
    pub fn lb_80011A50(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32) -> *mut lb_80011A50_t {
        match unsafe { LB_80011A50 } { Some(f) => f(a0, a1, a2, a3, a4, a5, a6, a7, a8), None => { return unsafe { melee_runtime::tu::lb_00F9::lb_80011A50(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _, a8 as _) as _ }; } }
    }
    pub static mut MEMZERO: Option<fn(*mut c_void, ssize_t)> = None;
    #[allow(non_snake_case)]
    pub fn memzero(mem: *mut c_void, size: ssize_t) {
        match unsafe { MEMZERO } { Some(f) => f(mem, size), None => { unsafe { let _ = melee_runtime::tu::lb_00B0::memzero(mem as _, size as _); } return; } }
    }
    pub static mut MPJOINTFROMLINE: Option<fn(i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn mpJointFromLine(line_id: i32) -> i32 {
        match unsafe { MPJOINTFROMLINE } { Some(f) => f(line_id), None => { return unsafe { crate::tu::mplib::mpJointFromLine(line_id as _) as _ }; } }
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
    hook!(OSREPORT, OSReport, (a0: *const i8, va: &[usize]));
    hook!(PLAYER_GETPLAYERSLOTTYPE, Player_GetPlayerSlotType, (slot: s32) -> Gm_PKind);
    hook!(PLAYER_LOADPLAYERCOORDS, Player_LoadPlayerCoords, (slot: s32, out_vec: *mut Vec));
    pub static mut STAGE_80225194: Option<fn() -> StKind> = None;
    #[allow(non_snake_case)]
    pub fn Stage_80225194() -> StKind {
        match unsafe { STAGE_80225194 } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_80225194() as _ }; } }
    }
    pub static mut STAGE_GETBLASTZONEBOTTOMOFFSET: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Stage_GetBlastZoneBottomOffset() -> f32 {
        match unsafe { STAGE_GETBLASTZONEBOTTOMOFFSET } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_GetBlastZoneBottomOffset() as _ }; } }
    }
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
    pub static mut STAGE_GETBLASTZONETOPOFFSET: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Stage_GetBlastZoneTopOffset() -> f32 {
        match unsafe { STAGE_GETBLASTZONETOPOFFSET } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_GetBlastZoneTopOffset() as _ }; } }
    }
    pub static mut UN_802FF570: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn un_802FF570() {
        match unsafe { UN_802FF570 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::if_2FF2::un_802FF570(); } return; } }
    }
    pub static mut UN_802FF620: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn un_802FF620() {
        match unsafe { UN_802FF620 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::if_2FF2::un_802FF620(); } return; } }
    }
    /// `grDisplay_801C5DB0` is passed as a render callback and called by the smash-taunt renderer.
    pub static mut GRDISPLAY_801C5DB0: Option<fn(*mut HSD_GObj, i32)> = None;
    pub unsafe extern "C" fn grDisplay_801C5DB0(gobj: *mut HSD_GObj, pass: i32) {
        match GRDISPLAY_801C5DB0 {
            Some(f) => f(gobj, pass),
            None => { unsafe { let _ = crate::tu::grdisplay::grDisplay_801C5DB0(gobj as _, pass as _); } return; },
        }
    }
}

/// `PAD_TRIGGER_R` / `PAD_TRIGGER_L` (dolphin/pad.h).
pub const PAD_TRIGGER_R: u32 = 1 << 5;
pub const PAD_TRIGGER_L: u32 = 1 << 6;

/// `struct grCorneria_YakumonoParam`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grCorneria_YakumonoParam {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub x14: f32,
    pub x18: f32,
    pub x1C: f32,
    pub x20: f32,
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
    pub pad50: [u8; 0x18],
    pub x68: f32,
    pub pad6C: [u8; 0x4],
    pub x70: f32,
    pub x74: i32,
    pub x78: i32,
    pub x7C: i32,
    pub x80: i32,
    pub x84: *mut c_void,
    pub x88: f32,
}

#[inline(always)]
unsafe fn yp() -> &'static grCorneria_YakumonoParam {
    &*yakumono_param
}

/// `static struct grCorneria_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grCorneria_YakumonoParam = null_mut();
pub static mut grCn_804D69A4: u32 = 0;
pub static mut grCn_804D69A8: i32 = 0;
pub static mut grCn_804D69AC: i32 = 0; // C `bool` is a 4-byte int in the shipping build
pub static mut grCn_804D69B0: i32 = 0;
/// `int grCn_804D466C = -1`.
pub static mut grCn_804D466C: i32 = -1;

pub static mut grCn_803E1D38: [GrJoint; 8] = [
    GrJoint { x: 3, y: 3, z: 0 },
    GrJoint { x: 4, y: 3, z: 0 },
    GrJoint { x: 0, y: 13, z: 0 },
    GrJoint { x: 1, y: 14, z: 0 },
    GrJoint { x: 2, y: 15, z: 0 },
    GrJoint { x: 5, y: 16, z: 0 },
    GrJoint { x: 6, y: 17, z: 0 },
    GrJoint { x: 7, y: 18, z: 0 },
];

pub static mut arwing_gobjs: [*mut HSD_GObj; 3] = [null_mut(); 3];
pub static mut arwing_types: [i32; 3] = [0; 3];
pub static mut arwing_groups: [i32; 3] = [0; 3];

macro_rules! cbs {
    ($a:ident, $b:ident, $c:ident, $d:ident, $flags:expr) => {
        StageCallbacks {
            on_init: Some($a),
            callback1: Some($b),
            gobj_proc: Some($c),
            callback3: Some($d),
            anon_10: StageCallbacks_anon_135 { flags: $flags },
        }
    };
}

/// `StageCallbacks grCn_StageCallbacks[19]`.
pub static mut grCn_StageCallbacks: [StageCallbacks; 19] = [
    cbs!(grCorneria_801DD620, grCorneria_801DD64C, grCorneria_801DD654, grCorneria_801DD658, 0),
    cbs!(grCorneria_801DDAC4, grCorneria_801DDCE8, grCorneria_801DE024, grCorneria_801DE4BC, 0),
    cbs!(grCorneria_801DE8E4, grCorneria_801DEC00, grCorneria_801DED50, grCorneria_801DF8CC, 0),
    cbs!(grCorneria_801DD674, grCorneria_801DD9A0, grCorneria_801DD9A8, grCorneria_801DDAC0, 0x80000000),
    cbs!(grCorneria_801E0140, grCorneria_801E01A0, grCorneria_801E01A8, grCorneria_801E03C4, 0),
    cbs!(grCorneria_801E0DE4, grCorneria_801E0E0C, grCorneria_801E0E14, grCorneria_801E0E3C, 0x20000000),
    cbs!(stageGObj6_OnInit, grCorneria_801DFC1C, grCorneria_801DFC24, grCorneria_801DFC28, 0),
    cbs!(grCorneria_801E0C3C, grCorneria_801E0D28, grCorneria_801E0D30, grCorneria_801E0DE0, 0x40000000),
    cbs!(grCorneria_801DFC2C, grCorneria_801DFC90, grCorneria_801DFC98, grCorneria_801DFEB4, 0),
    cbs!(grCorneria_801DFEB8, grCorneria_801DFF18, grCorneria_801DFF20, grCorneria_801E013C, 0),
    cbs!(grCorneria_801DF8D0, grCorneria_801DFBC4, grCorneria_801DFBCC, grCorneria_801DFBEC, 0),
    cbs!(grCorneria_801E1054, grCorneria_801E1058, grCorneria_801E1060, grCorneria_801E12CC, 0),
    cbs!(grCorneria_801E0F30, grCorneria_801E0F64, grCorneria_801E0F6C, grCorneria_801E1030, 0),
    cbs!(grCorneria_Arwing_801DE4C0, grCorneria_801DE560, grCorneria_801DE568, grCorneria_801DE8E0, 0),
    cbs!(grCorneria_Arwing_801DE4C0, grCorneria_801DE560, grCorneria_801DE568, grCorneria_801DE8E0, 0),
    cbs!(grCorneria_Arwing_801DE4C0, grCorneria_801DE560, grCorneria_801DE568, grCorneria_801DE8E0, 0),
    cbs!(grCorneria_Arwing_801DE4C0, grCorneria_801DE560, grCorneria_801DE568, grCorneria_801DE8E0, 0),
    cbs!(grCorneria_Arwing_801DE4C0, grCorneria_801DE560, grCorneria_801DE568, grCorneria_801DE8E0, 0),
    cbs!(grCorneria_Arwing_801DE4C0, grCorneria_801DE560, grCorneria_801DE568, grCorneria_801DE8E0, 0),
];

/// `"/GrCn"`.
pub static mut grCn_str_GrCn: [u8; 6] = *b"/GrCn\0";

/// `StageData grCn_StageData`.
pub static mut grCn_StageData: StageData = StageData {
    grkind: Gr_Kind_Corneria,
    callbacks: core::ptr::addr_of!(grCn_StageCallbacks) as *mut StageCallbacks,
    data1: core::ptr::addr_of!(grCn_str_GrCn) as *mut i8,
    on_init: Some(grCorneria_801DD350),
    on_demo_init: Some(grCorneria_801DD2C0),
    on_load: Some(grCorneria_801DD478),
    on_start: Some(grCorneria_801DD508),
    callback4: Some(grCorneria_801DD52C),
    on_touch_line: Some(grCorneria_801E2EE4),
    on_check_shadow_render: Some(grCorneria_801E2EEC),
    flags2: 1 << 0,
    joints: core::ptr::addr_of!(grCn_803E1D38) as *mut GrJoint,
    joint_count: 8,
};

/// `grCorneria_801DCCFC`: reset the arwing slots and roll the first spawn timer.
pub unsafe extern "C" fn grCorneria_801DCCFC() {
    arwing_gobjs[0] = null_mut();
    arwing_types[0] = 0;
    arwing_groups[0] = 0;
    arwing_gobjs[1] = null_mut();
    arwing_types[1] = 0;
    arwing_groups[1] = 0;
    arwing_gobjs[2] = null_mut();
    arwing_types[2] = 0;
    arwing_groups[2] = 0;

    let stage = ext::Stage_80225194();
    match stage {
        0xE9 | 0x47 => grCn_804D69B0 = 1,
        _ => grCn_804D69B0 = 0,
    }

    if grCn_804D69B0 == 1 {
        grCn_804D69A8 = 0x78;
        return;
    }

    let imin = yp().x3C as s32;
    let mut imax = yp().x40 as s32;
    if imax > imin {
        let range = imax - imin;
        imax = imin + if range != 0 { ext::HSD_Randi(range) } else { 0 };
    } else if imax < imin {
        let range = imin - imax;
        imax += if range != 0 { ext::HSD_Randi(range) } else { 0 };
    }
    grCn_804D69A8 = imax;
}

/// `static inline grCn_RandRange`.
unsafe fn grCn_RandRange(mut imax: s32, imin: s32) -> s32 {
    if imax > imin {
        let range = imax - imin;
        imax = imin + if range != 0 { ext::HSD_Randi(range) } else { 0 };
    } else if imax < imin {
        let range = imin - imax;
        imax += if range != 0 { ext::HSD_Randi(range) } else { 0 };
    }
    imax
}

/// `static inline grCn_CheckFar`.
unsafe fn grCn_CheckFar(pos: *mut Vec3) -> i32 {
    let mut kind = grCorneria_801E0A74(addr_of_mut!((*pos).x));
    if kind == 4 || kind == 9 {
        (*pos).x -= 300.0f32;
        kind = grCorneria_801E0A74(addr_of_mut!((*pos).x));
        if kind == 4 || kind == 9 {
            return 0;
        }
    }
    1
}

/// `grCorneria_801DCE1C`: arwing spawn scheduler.
pub unsafe extern "C" fn grCorneria_801DCE1C() {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut pos2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if ext::Stage_80225194() != 0x46 {
        if grCn_804D69B0 == 0 {
            if arwing_gobjs[0].is_null() {
                grCn_804D69A8 -= 1;
                if grCn_804D69A8 <= 0 {
                    grCorneria_801DDD4C(&mut pos);
                    let far = grCn_CheckFar(&mut pos);

                    let mut type_id = arwing_types[0];
                    while type_id == arwing_types[0] {
                        let var_r0 = if far == 1 { 9 } else { 13 };
                        let var_r3_2 = if var_r0 != 0 { ext::HSD_Randi(if far == 1 { 9 } else { 13 }) } else { 0 };
                        type_id = var_r3_2 + 1;
                    }

                    let group = if arwing_groups[0] == 4 {
                        1
                    } else if ext::HSD_Randf() > yp().x4C {
                        1
                    } else {
                        4
                    };

                    grCn_804D69A4 = 0;
                    arwing_types[0] = type_id;
                    arwing_groups[0] = group;
                    arwing_gobjs[grCn_804D69A4 as usize] = grCorneria_801DD534(1);
                }
            } else {
                grCn_804D69A8 = grCn_RandRange(yp().x48 as s32, yp().x44 as s32);
            }
        } else {
            let mut count: usize = 0;
            while count < 3 {
                if arwing_gobjs[count].is_null() {
                    break;
                }
                count += 1;
            }

            if count < 3 {
                grCn_804D69A8 -= 1;
                if grCn_804D69A8 <= 0 {
                    let mut has_near = false;
                    for i in 0..3usize {
                        if i != count {
                            match arwing_types[i] {
                                1..=9 => {
                                    if !arwing_gobjs[i].is_null() {
                                        has_near = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    if !has_near {
                        let mut type_id = arwing_types[count];
                        while type_id == arwing_types[0] || type_id == arwing_types[1] || type_id == arwing_types[2] {
                            type_id = ext::HSD_Randi(13) + 1;
                        }
                        grCn_804D69A4 = count as u32;
                        arwing_types[count] = type_id;
                        arwing_groups[count] = count as i32 + 1;
                        arwing_gobjs[grCn_804D69A4 as usize] = grCorneria_801DD534(1);
                    } else {
                        grCorneria_801DDD4C(&mut pos2);
                        if grCn_CheckFar(&mut pos2) == 0 {
                            let mut type_id = arwing_types[count];
                            while type_id == arwing_types[0] || type_id == arwing_types[1] || type_id == arwing_types[2] {
                                type_id = ext::HSD_Randi(4) + 10;
                            }
                            grCn_804D69A4 = count as u32;
                            arwing_types[count] = type_id;
                            arwing_groups[count] = count as i32 + 1;
                            arwing_gobjs[grCn_804D69A4 as usize] = grCorneria_801DD534(1);
                        }
                    }
                }
            } else {
                grCn_804D69A8 = 10;
            }
        }
    } else {
        arwing_gobjs[0] = null_mut();
        arwing_gobjs[1] = null_mut();
        arwing_gobjs[2] = null_mut();
    }
}

/// `grCorneria_801DD2C0` (on_demo_init).
pub unsafe extern "C" fn grCorneria_801DD2C0(_unused: i32) {
    let mut obj = ext::Ground_GetMapGObj(3);
    if !obj.is_null() {
        let jobj = GET_JOBJ(obj);
        if !jobj.is_null() {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    }
    grCorneria_801E0678();
    obj = grCorneria_801DD534(12);
    let rand = if ext::gm_8017E7E0() != 0 { ext::HSD_Randi(4) + 4 } else { ext::HSD_Randi(4) };
    grCorneria_801E0F34(obj, rand);
    grCn_804D69AC = 1;
}

/// `grCorneria_801DD350` (on_init).
pub unsafe extern "C" fn grCorneria_801DD350() {
    let light_pos_init = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grCorneria_YakumonoParam;
    let si = ext::stage_info();
    (*si).unk8C.set_b4(0);
    (*si).unk8C.set_b5(1);
    grCorneria_801DD534(7);
    grCorneria_801DD534(3);
    grCorneria_801DCCFC();
    (*(*grCorneria_801DD534(8)).user_data.cast::<Ground>()).u.corneria.xC4.value = 1;
    (*(*grCorneria_801DD534(9)).user_data.cast::<Ground>()).u.corneria.xC4.value = 1;
    (*(*grCorneria_801DD534(4)).user_data.cast::<Ground>()).u.corneria.xC4.value = 1;
    grCorneria_801DD534(0xB);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    let mut light_pos = light_pos_init;
    ext::lb_80011A50(&mut light_pos, -1, 1.0, 0.0, 1.0471976, -100000.0, 100000.0, 100000.0, -100000.0);
    grCn_804D69AC = 1;
    ext::mpLib_80057BC0(0);
    ext::mpLib_80057BC0(1);
    ext::mpLib_80057BC0(2);
    ext::mpLib_80057BC0(5);
    ext::mpLib_80057BC0(6);
    ext::mpLib_80057BC0(7);
}

/// `grCorneria_801DD478` (on_load).
pub unsafe extern "C" fn grCorneria_801DD478() {
    let mut gobj = ext::Ground_GetMapGObj(11);
    if !gobj.is_null() {
        ext::Ground_801C4A08(gobj);
    }
    gobj = ext::Ground_GetMapGObj(8);
    if !gobj.is_null() {
        ext::Ground_801C4A08(gobj);
    }
    gobj = ext::Ground_GetMapGObj(9);
    if !gobj.is_null() {
        ext::Ground_801C4A08(gobj);
    }
    gobj = ext::Ground_GetMapGObj(4);
    if !gobj.is_null() {
        ext::Ground_801C4A08(gobj);
    }
    gobj = grCorneria_801DD534(8);
    let gp = GET_GROUND(gobj);
    (*gp).u.corneria.xC5 = 0;
    grCn_804D69AC = 0;
    ext::Ground_801C53EC(0x55737);
}

/// `grCorneria_801DD508` (on_start).
pub unsafe extern "C" fn grCorneria_801DD508() {
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// `grCorneria_801DD52C` (callback4).
pub unsafe extern "C" fn grCorneria_801DD52C() -> i32 {
    0
}

/// `grCorneria_801DD534`: create stage gobj `arg0` and wire its callbacks.
pub unsafe fn grCorneria_801DD534(arg0: i32) -> *mut HSD_GObj {
    let callbacks: *mut StageCallbacks = &mut grCn_StageCallbacks[arg0 as usize];
    let gobj = ext::Ground_GetStageGObj(arg0);

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
            &[b"grcorneria.c\0".as_ptr() as usize, 0x26A, arg0 as u32 as usize],
        );
    }
    gobj
}

pub unsafe extern "C" fn grCorneria_801DD620(gobj: *mut Ground_GObj) {
    ext::grAnime_801C8138(gobj, (*GET_GROUND(gobj)).map_id as enum_t, 0);
}

pub unsafe extern "C" fn grCorneria_801DD64C(_arg: *mut Ground_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grCorneria_801DD654(_arg: *mut Ground_GObj) {}

pub unsafe extern "C" fn grCorneria_801DD658(_arg: *mut Ground_GObj) {}

pub unsafe extern "C" fn grCorneria_801DD65C(gobj: *mut Ground_GObj) {
    (*GET_GROUND(gobj)).u.corneria.xC4.flags.set_b0(0);
}

/// `static Vec3 grCn_803E1F70[5]` / `grCn_803E1FAC[5]`.
pub static mut grCn_803E1F70: [Vec3; 5] = [
    Vec3 { x: -274.14, y: 48.58, z: 0.0 },
    Vec3 { x: -200.81, y: -20.1, z: 0.0 },
    Vec3 { x: -59.31, y: -58.74, z: 0.0 },
    Vec3 { x: -40.19, y: -73.77, z: 0.0 },
    Vec3 { x: 78.44, y: -73.48, z: 0.0 },
];
pub static mut grCn_803E1FAC: [Vec3; 5] = [Vec3 { x: 0.0, y: 0.0, z: 0.0 }; 5];

/// `static grZakoGenerator_Spawn grCn_803E1FE8[1]` / `grCn_803E2000[2]` (six ints each in the C initialisers).
pub static mut grCn_803E1FE8: [grZakoGenerator_Spawn; 1] = [grZakoGenerator_Spawn {
    pos0: Vec3 { x: -59.0, y: -58.0, z: 18.0 },
    pos1: Vec3 { x: -22.0, y: -73.0, z: 0.0 },
}];
pub static mut grCn_803E2000: [grZakoGenerator_Spawn; 2] = [
    grZakoGenerator_Spawn { pos0: Vec3 { x: -81.0, y: -21.0, z: 28.0 }, pos1: Vec3 { x: 14.0, y: -59.0, z: 128.0 } },
    grZakoGenerator_Spawn { pos0: Vec3 { x: 14.0, y: -59.0, z: 128.0 }, pos1: Vec3 { x: 0.0, y: -75.0, z: 173.0 } },
];

pub static grCn_803E2030: [i32; 14] = [-1, 0, 1, 2, 3, 4, 5, 6, 6, 6, 7, 8, 9, 10];

pub static lbl_803E2068: [Vec3; 14] = [
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    Vec3 { x: -40.0, y: 10.0, z: 0.0 },
    Vec3 { x: 15.0, y: 5.0, z: 0.0 },
    Vec3 { x: 190.0, y: -10.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
];

pub static grCn_803E2110: [i32; 5] = [2, 2, 2, 2, 10];

/// `grCorneria_801DD674`: the Great Fox (ship) init.
pub unsafe extern "C" fn grCorneria_801DD674(ground_gobj: *mut Ground_GObj) {
    let gr = GET_GROUND(ground_gobj);
    ext::Ground_InitMapColl((*ground_gobj).hsd_obj as *mut HSD_JObj, (*gr).map_id as s32);
    (*gr).u.corneria.xC4.flags.set_b0(1);
    (*gr).u.corneria.xC4.flags.set_b1(0);
    (*gr).u.corneria.base_x = 0.0;
    (*gr).u.corneria.base_y = 250.0;
    (*gr).u.corneria.xD0 = 0.0;
    (*gr).u.corneria.offset_x = 0.0;
    (*gr).u.corneria.offset_y.val = 0.0;
    (*gr).u.corneria.xE4.x = 0.0;
    (*gr).u.corneria.xE4.y = 0.0;
    (*gr).u.corneria.xE4.z = 0.0;
    (*gr).u.corneria.xF0 = 0.0;
    (*gr).u.corneria.xF4 = 0.0;
    (*gr).u.corneria.xF8 = 0.0;
    (*gr).u.corneria.x104 = 0;
    (*gr).x10_flags.set_b5(1);
    ext::grAnime_801C8138(ground_gobj, (*gr).map_id as enum_t, 0);
    ext::grAnime_801C8780(ground_gobj, 3, 0, 0.0, 1.0);
    (*gr).u.corneria.x128 = grCorneria_801DD534(0);
    let jobj = (*ground_gobj).hsd_obj as *mut HSD_JObj;
    let translate = Vec3 {
        x: (*gr).u.corneria.base_x + (*gr).u.corneria.offset_x,
        y: (*gr).u.corneria.base_y + (*gr).u.corneria.offset_y.val,
        z: 0.0,
    };
    HSD_JObjSetTranslate(jobj, &translate);
    grCorneria_801E1878(ground_gobj);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    (*gr).u.corneria.x108 = 0;
    (*gr).u.corneria.x10C = 0;
    (*gr).u.corneria.x118 = 0;
    (*gr).u.corneria.x11A = 0;
    (*gr).u.corneria.x110 = 0;
    (*gr).u.corneria.x114 = 0.0;
    let j2 = ext::Ground_801C3FA4(ground_gobj, 2);
    (*gr).u.corneria.left_cannon = ext::grMaterial_801C8CFC(0, 0, gr, j2, Some(fn_801E12D0), Some(fn_801E12D4), None);
    ext::grMaterial_801C8DE0((*gr).u.corneria.left_cannon, 4.0, 0.0, 0.0, 16.0, 0.0, 0.0, 4.0);
    ext::grMaterial_801C8E08((*gr).u.corneria.left_cannon);
    let j3 = ext::Ground_801C3FA4(ground_gobj, 3);
    (*gr).u.corneria.right_cannon = ext::grMaterial_801C8CFC(0, 2, gr, j3, Some(fn_801E12D0), Some(fn_801E12D4), None);
    ext::grMaterial_801C8E08((*gr).u.corneria.right_cannon);
    ext::grMaterial_801C8DE0((*gr).u.corneria.right_cannon, 4.0, 0.0, 0.0, 16.0, 0.0, 0.0, 4.0);
    ext::HSD_JObjClearFlags(ext::Ground_801C3FA4(ground_gobj, 6), JOBJ_HIDDEN as u32);
    ext::HSD_JObjSetFlags(ext::Ground_801C3FA4(ground_gobj, 5), JOBJ_HIDDEN as u32);
    (*gr).u.corneria.xC8 = ext::grZakoGenerator_801CA394(addr_of_mut!(grCn_803E1FE8) as *mut grZakoGenerator_Spawn, 1, Some(grCorneria_801E2454), 0.3);
    (*gr).u.corneria.xCC = ext::grZakoGenerator_801CA394(addr_of_mut!(grCn_803E2000) as *mut grZakoGenerator_Spawn, 2, Some(grCorneria_801E2480), 0.3);
    ext::Ground_801C10B8(ground_gobj, Some(grCorneria_801DD65C));
    (*gr).x11_flags.set_b012(1);
    (*gr).u.corneria.x12C = ext::Ground_801C3FA4(ground_gobj, 8);
}

pub unsafe extern "C" fn grCorneria_801DD9A0(_arg: *mut Ground_GObj) -> i32 {
    0
}

/// `grCorneria_801DD9A8`: Great Fox per-frame update.
pub unsafe extern "C" fn grCorneria_801DD9A8(gobj: *mut Ground_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = GET_JOBJ(gobj);

    if (*gp).u.corneria.xC4.flags.b0() == 0 {
        grCorneria_801E1970(gobj);
        grCorneria_801E2228(gobj);

        let mut speed = fmadds(-35.0, ext::Ground_801C0498(), -(*gp).u.corneria.xD0);
        ext::grZakoGenerator_801CA43C((*gp).u.corneria.xC8, ext::Ground_801C3FA4(gobj, 0), speed);

        speed = fmadds(-35.0, ext::Ground_801C0498(), -(*gp).u.corneria.xD0);
        ext::grZakoGenerator_801CA43C((*gp).u.corneria.xCC, ext::Ground_801C3FA4(gobj, 0), speed);

        grCorneria_801E1348(gobj);
        grCorneria_801E0E40();
        grCorneria_801DCE1C();
    }

    for i in 0..5usize {
        ext::lb_8000B1CC(jobj, addr_of_mut!(grCn_803E1F70[i]), addr_of_mut!(grCn_803E1FAC[i]));
    }

    ext::Ground_UpdateMapColl(gobj);
    grCorneria_801E1878(gobj);
    grCorneria_801E2110();
    ext::lb_800115F4();
}

pub unsafe extern "C" fn grCorneria_801DDAC0(_arg: *mut Ground_GObj) {}

/// `grCorneria_801DDAC4`: an arwing joins the fight.
pub unsafe extern "C" fn grCorneria_801DDAC4(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    (*gp).u.arwing.xC8 = grCn_804D69A4;
    arwing_gobjs[grCn_804D69A4 as usize] = gobj;
    {
        let idx = arwing_groups[(*gp).u.arwing.xC8 as usize];
        let arwing = grCorneria_801DD534(grCn_803E2110[idx as usize]);
        if !arwing.is_null() {
            (*GET_GROUND(arwing)).x10_flags.set_b2(0);
            {
                let agp = GET_GROUND(arwing);
                if !agp.is_null() {
                    (*agp).u.arwing.xC8 = (*gp).u.arwing.xC8;
                }
            }
        }
    }
    let scale = ext::Ground_801C0498();
    HSD_JObjSetScaleX(jobj, scale * yp().x70);
    HSD_JObjSetScaleY(jobj, scale);
    HSD_JObjSetScaleZ(jobj, scale);
    (*gp).u.arwing.xD4 = 1;
    (*gp).u.arwing.xD8 = 0;
}

pub unsafe extern "C" fn grCorneria_801DDCE8(_arg: *mut Ground_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grCorneria_801DDCF0(vec: *mut Vec3) {
    let gobj = ext::Ground_GetMapGObj(3);
    let gp = GET_GROUND(gobj);
    (*vec).x = (*gp).u.corneria.offset_x - (*gp).u.corneria.xE4.x;
    (*vec).y = (*gp).u.corneria.offset_y.val - (*gp).u.corneria.xE4.y;
    (*vec).z = 0.0;
}

pub unsafe extern "C" fn grCorneria_801DDD4C(vec: *mut Vec3) {
    let gobj = ext::Ground_GetMapGObj(3);
    let gp = GET_GROUND(gobj);
    (*vec).x = (*gp).u.corneria.base_x + (*gp).u.corneria.offset_x;
    (*vec).y = (*gp).u.corneria.base_y + (*gp).u.corneria.offset_y.val;
    (*vec).z = 0.0;
}

pub unsafe extern "C" fn grCorneria_801DDDA8(gobj: *mut HSD_GObj, vec: *mut Vec3) {
    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        let gp2 = (*ext::Ground_GetMapGObj(3)).user_data as *mut Ground;
        let jobj = ext::Ground_801C3FA4(gobj, 4);
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC(jobj, null_mut(), &mut pos);
        let idx = arwing_types[(*gp).u.arwing.xC8 as usize] as usize;
        // the ship's ground is a `corneria` record: the C's `u.arwing.xDC` is its `offset_x` (+0x18 on the PPC)
        (*vec).x = (*gp2).u.corneria.offset_x + (-pos.z + lbl_803E2068[idx].x);
        (*vec).y = pos.y + lbl_803E2068[idx].y;
        (*vec).z = pos.x + lbl_803E2068[idx].z;
    } else {
        (*vec).z = 0.0;
        (*vec).y = 0.0;
        (*vec).x = 0.0;
    }

    if (*vec).y <= 100.0 {
        (*vec).y = 100.0;
    }
}

pub unsafe extern "C" fn grCorneria_801DDE88(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);

    let mut sp2C = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp20 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if !gobj.is_null() {
        let gp2 = (*gobj).user_data as *mut Ground;
        let ship = GET_GROUND(ext::Ground_GetMapGObj(3));
        ext::lb_8000B1CC(ext::Ground_801C3FA4(gobj, 4), null_mut(), &mut sp20);
        let idx = arwing_types[(*gp2).u.arwing.xC8 as usize] as usize;
        sp2C.x = (*ship).u.corneria.offset_x + (-sp20.z + lbl_803E2068[idx].x);
        sp2C.y = sp20.y + lbl_803E2068[idx].y;
        sp2C.z = sp20.x + lbl_803E2068[idx].z;
    } else {
        sp2C.z = 0.0;
        sp2C.y = 0.0;
        sp2C.x = 0.0;
    }
    if sp2C.y <= 100.0 {
        sp2C.y = 100.0;
    }
    match (*gp).u.arwing.xD8 {
        0 => {
            if grCorneria_801DEC08(&mut sp2C) == 0 {
                ext::lbAudioAx_800237A8(0x55730, 0x7F, 0x40);
                (*gp).u.arwing.xD8 = 1;
            }
        }
        1 => {
            if grCorneria_801DEC94(&mut sp2C) == 0 {
                (*gp).u.arwing.xD8 = 2;
                return;
            }
        }
        2 => {
            if grCorneria_801DEC94(&mut sp2C) == 1 {
                ext::lbAudioAx_800237A8(0x55732, 0x7F, 0x40);
                (*gp).u.arwing.xD8 = 3;
            }
        }
        _ => {}
    }
}

/// Bounds shared by `801DE024` and `801DED50`.
const grCn_ArwingMinX: f32 = -10.0;
const grCn_ArwingMaxX: f32 = 2000.0;

/// `static inline grCorneria_801DE024_inline`.
unsafe fn grCorneria_801DE024_inline(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    grCorneria_801DDDA8(gobj, &mut pos);
    match (*gp).u.arwing.xD8 {
        0 => {
            if grCn_ArwingMinX < pos.x && pos.x < grCn_ArwingMaxX {
                ext::lbAudioAx_800237A8(0x55731, 0x7F, 0x40);
                (*gp).u.arwing.xD8 = 1;
            }
        }
        1 => {
            if pos.x < grCn_ArwingMinX || grCn_ArwingMaxX < pos.x {
                ext::lbAudioAx_800237A8(0x55732, 0x7F, 0x40);
                (*gp).u.arwing.xD8 = 2;
            }
        }
        _ => {}
    }
}

/// `grCorneria_801DE024`: arwing per-frame update.
pub unsafe extern "C" fn grCorneria_801DE024(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    let mut translate = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let ship_gobj = ext::Ground_GetMapGObj(3);
    let ship_jobj = GET_JOBJ(ship_gobj);
    if !ship_jobj.is_null() {
        let y = HSD_JObjGetTranslationY(ship_jobj);
        HSD_JObjSetTranslateY(jobj, y);
        {
            let j1 = ext::Ground_801C3FA4(gobj, 1);
            let y1 = HSD_JObjGetTranslationY(j1) - 349.0;
            HSD_JObjSetTranslateY(j1, y1);
        }
    }
    {
        if arwing_gobjs[(*gp).u.arwing.xC8 as usize] == gobj {
            if (*gp).u.arwing.xD4 == 1 {
                (*gp).u.arwing.xD4 = 0;
                {
                    let idx = arwing_types[(*gp).u.arwing.xC8 as usize] as usize;
                    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, (grCn_803E2030[idx] != 0) as i32);
                }
                return;
            }
            match arwing_types[(*gp).u.arwing.xC8 as usize] {
                1..=9 => {
                    grCorneria_801DDE88(gobj);
                }
                10..=13 => {
                    HSD_JObjSetRotationY(jobj, -1.5707964);
                    grCorneria_801DE024_inline(gobj);
                    {
                        let mut vf = grCorneria_801E2EA0();
                        if vf > 0.0 {
                            vf = 0.0;
                        }
                        if vf < -150.0 {
                            vf = -150.0;
                        }
                        let j0 = ext::Ground_801C3FA4(gobj, 0);
                        HSD_JObjGetTranslation(j0, &mut translate);
                        translate.y = ABS(vf);
                        HSD_JObjSetTranslate(j0, &translate);
                    }
                }
                _ => {}
            }
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                arwing_gobjs[(*gp).u.arwing.xC8 as usize] = null_mut();
                ext::Ground_801C4A08(gobj);
            }
        } else {
            ext::Ground_801C4A08(gobj);
        }
    }
}

pub unsafe extern "C" fn grCorneria_801DE4BC(_arg: *mut Ground_GObj) {}

/// `static int grCn_803E214C[6][2]`, flat (row `k`, column `c` is `k * 2 + c`).
pub static grCn_803E214C: [i32; 12] = [-1, -1, 1, 0, 2, 2, 4, 4, 5, 5, -1, -1];
pub static grCn_803E217C: [i32; 5] = [0x0D, 0x0D, 0x0E, 0x0F, 0x10];
pub static grCn_803E2190: [i32; 5] = [0, 0, 1, 2, 5];

/// `grCorneria_Arwing_801DE4C0`: arwing article init.
pub unsafe extern "C" fn grCorneria_Arwing_801DE4C0(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::Ground_InitMapColl(jobj, (*gp).map_id as s32);
    (*gp).u.arwing.xC8 = grCn_804D69A4;
    (*gp).u.arwing.xD0 = 0;
    (*gp).u.arwing.xDC = 0.0;
    (*gp).u.arwing.xE0.z = 0.0;
    (*gp).u.arwing.xE0.y = 0.0;
    (*gp).u.arwing.xE0.x = 0.0;
    let group = arwing_groups[(*gp).u.arwing.xC8 as usize];
    let joint_id = grCn_803E2190[group as usize];
    ext::mpJointListAdd(joint_id);

    ext::Ground_InitMapColl(jobj, (*gp).map_id as s32);
}

pub unsafe extern "C" fn grCorneria_801DE560(_arg: *mut Ground_GObj) -> i32 {
    0
}

/// `static inline Ground_UpdateStarFoxArwingVisibility` (gr/inlines.h).
unsafe fn Ground_UpdateStarFoxArwingVisibility(gp: *mut Ground, jobj: *mut HSD_JObj, depth_limit: f32) {
    let angle = ABS((*gp).u.arwing.xE0.z);
    if angle < depth_limit {
        (*gp).u.arwing.xE0.z = 0.0;
        while ((*gp).u.arwing.xDC as f64) < -M_PI {
            (*gp).u.arwing.xDC = ((*gp).u.arwing.xDC as f64 + M_TAU) as f32;
        }
        while ((*gp).u.arwing.xDC as f64) > M_PI {
            (*gp).u.arwing.xDC = ((*gp).u.arwing.xDC as f64 - M_TAU) as f32;
        }
        if ABS((*gp).u.arwing.xDC) < 1.0471976 {
            ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        } else {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    } else {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
    HSD_JObjSetTranslate(jobj, addr_of_mut!((*gp).u.arwing.xE0));
}

/// `grCorneria_801DE568`: arwing article per-frame update.
pub unsafe extern "C" fn grCorneria_801DE568(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    if !arwing_gobjs[(*gp).u.arwing.xC8 as usize].is_null() {
        Ground_UpdateStarFoxArwingVisibility(gp, jobj, 10.0);
        {
            let scale = ext::Ground_801C0498();
            {
                let s = scale * yp().x70;
                HSD_JObjSetScaleX(jobj, s);
            }
            {
                let s = scale * yp().x70;
                HSD_JObjSetScaleY(jobj, s);
            }
            {
                let s = scale * yp().x70;
                HSD_JObjSetScaleZ(jobj, s);
            }
        }
        ext::Ground_UpdateMapColl(gobj);
    } else {
        ext::mpLib_80057BC0(grCn_803E2190[arwing_groups[(*gp).u.arwing.xC8 as usize] as usize]);
        ext::Ground_801C4A08(gobj);
    }
}

pub unsafe extern "C" fn grCorneria_801DE8E0(_arg: *mut Ground_GObj) {}

// ---- gr/inlines.h (star fox arwing helpers)
unsafe fn Ground_ClearStarFoxArwingGObjs(gp: *mut Ground) {
    (*gp).u.starfox.article_gobjs[3] = null_mut();
    (*gp).u.starfox.article_gobjs[2] = null_mut();
    (*gp).u.starfox.article_gobjs[1] = null_mut();
    (*gp).u.starfox.article_gobjs[0] = null_mut();
}

unsafe fn Ground_AnimateStarFoxArwing(gobj: *mut Ground_GObj) {
    ext::grAnime_801C7FF8(gobj, 0, 7, 1, 0.0, 1.0);
    ext::grAnime_801C8098(gobj, 2, 7, 3, 0.0, 1.0);
}

unsafe fn Ground_AnimateStarFoxArwingWithBackground(gobj: *mut Ground_GObj) {
    ext::grAnime_801C7FF8(gobj, 7, 7, 0, 0.0, 1.0);
    ext::grAnime_801C7FF8(gobj, 8, 7, 0, 0.0, 1.0);
    ext::grAnime_801C8098(gobj, 2, 7, 3, 0.0, 1.0);
}

unsafe fn Ground_LinkStarFoxArwing(gp: *mut Ground, linked_gobj: *mut HSD_GObj) {
    (*gp).u.starfox.linked_gobj = linked_gobj;
    if !linked_gobj.is_null() {
        let linked_gp = GET_GROUND((*gp).u.starfox.linked_gobj);
        if !linked_gp.is_null() {
            (*linked_gp).u.starfox.arwing_slot = (*gp).u.starfox.arwing_slot;
        }
    }
}

unsafe fn Ground_DisableStarFoxArwingGObjs(gp: *mut Ground) {
    (*gp).u.starfox.article_gobjs[0] = usize::MAX as *mut HSD_GObj;
    (*gp).u.starfox.article_gobjs[1] = usize::MAX as *mut HSD_GObj;
}

unsafe fn Ground_ResetStarFoxArwingState(gp: *mut Ground) {
    (*gp).u.starfox.xC4.flags.set_b0(0);
    (*gp).u.starfox.xD4 = 0;
    (*gp).u.starfox.xF0 = 0;
    (*gp).u.starfox.xF4 = 0;
}

unsafe fn Ground_AttachStarFoxArwingModel(gobj: *mut Ground_GObj, gp: *mut Ground, arwing_gobj: *mut Ground_GObj, joint_id: i32) {
    let a = ext::Ground_801C3FA4(gobj, 0);
    let b = ext::Ground_801C3FA4(arwing_gobj, joint_id);
    ext::lb_8000C2F8(a, b);
    (*gp).u.starfox.linked_gobj = null_mut();
}

/// `grCorneria_801DE8E4`: Star Fox article (with the background) init.
pub unsafe extern "C" fn grCorneria_801DE8E4(gobj: *mut Ground_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = GET_JOBJ(gobj);

    let scale = ext::Ground_801C0498();
    (*gp).u.corneria2.xC8 = grCn_804D69A4 as i32;
    Ground_ClearStarFoxArwingGObjs(gp);
    Ground_AnimateStarFoxArwingWithBackground(gobj);

    match arwing_types[(*gp).u.corneria2.xC8 as usize] {
        1..=9 => {
            let arwing = grCorneria_801DD534(grCn_803E217C[arwing_groups[(*gp).u.corneria2.xC8 as usize] as usize]);
            Ground_LinkStarFoxArwing(gp, arwing);
        }
        10..=13 => {
            Ground_AttachStarFoxArwingModel(gobj, gp, arwing_gobjs[(*gp).u.corneria2.xC8 as usize], 4);
        }
        _ => {
            Ground_DisableStarFoxArwingGObjs(gp);
        }
    }
    HSD_JObjSetScaleX(jobj, scale * yp().x70);
    HSD_JObjSetScaleY(jobj, scale * yp().x70);
    HSD_JObjSetScaleZ(jobj, scale * yp().x70);
    Ground_ResetStarFoxArwingState(gp);
    (*gp).u.corneria2.xF8 = yp().x68 as s32;
    (*gp).u.corneria2.xFC = 0;
    (*gp).u.corneria2.x100 = ext::HSD_Randi(2);
}

pub unsafe extern "C" fn grCorneria_801DEC00(_arg: *mut Ground_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grCorneria_801DEC08(pos: *mut Vec3) -> i32 {
    if (*pos).x > ext::Stage_GetBlastZoneRightOffset() {
        return 1;
    }
    if (*pos).x < ext::Stage_GetBlastZoneLeftOffset() {
        return 1;
    }
    if (*pos).y > ext::Stage_GetBlastZoneTopOffset() {
        return 1;
    }
    if (*pos).y < ext::Stage_GetBlastZoneBottomOffset() {
        return 1;
    }
    0
}

pub unsafe extern "C" fn grCorneria_801DEC94(pos: *mut Vec3) -> s32 {
    if (*pos).z > -30.0 {
        if (*pos).x > ext::Stage_GetBlastZoneRightOffset() - 30.0 {
            return 1;
        }
        if (*pos).x < ext::Stage_GetBlastZoneLeftOffset() + 30.0 {
            return 1;
        }
        if (*pos).y > ext::Stage_GetBlastZoneTopOffset() - 30.0 {
            return 1;
        }
        if (*pos).y < ext::Stage_GetBlastZoneBottomOffset() + 30.0 {
            return 1;
        }
    }
    0
}

pub static grCn_803E21B0: [i32; 5] = [1, 1, 1, 1, 1];

/// `static inline get_grp_off`.
unsafe fn get_grp_off(gobj: *mut Ground_GObj) -> s32 {
    let mut grp_off: s32 = -1;
    {
        let g = arwing_groups[(*GET_GROUND(gobj)).u.corneria2.xC8 as usize];
        match g {
            0 => {}
            1 | 2 | 3 => grp_off = 0,
            4 => grp_off = 1,
            _ => {}
        }
    }
    grp_off
}

/// `grCorneria_801DED50`: Star Fox article per-frame update (the Arwing's shots and rolls).
pub unsafe extern "C" fn grCorneria_801DED50(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if grCn_804D69AC != 0 {
        return;
    }

    let xc8 = (*gp).u.corneria2.xC8 as usize;
    if !arwing_gobjs[xc8].is_null() {
        if arwing_groups[xc8] == 4 {
            let arwing = ext::Ground_801C3FA4(gobj, 1);
            HSD_JObjSetRotationZ(arwing, 0.0);
        }
        match arwing_types[xc8] {
            1..=9 => {
                HSD_JObjSetRotationY(jobj, -1.5707964);
                match (*gp).u.corneria2.xF4 {
                    0 => {
                        if (*gp).u.corneria2.xF8 <= 0 {
                            let new_state = ext::HSD_Randi(5) + 1;
                            let mut group_offset: i32 = -1;
                            (*gp).u.corneria2.xF4 = new_state;
                            {
                                let group = arwing_groups[(*GET_GROUND(gobj)).u.corneria2.xC8 as usize];
                                match group {
                                    0 => {}
                                    1 | 2 | 3 => group_offset = 0,
                                    4 => group_offset = 1,
                                    _ => {}
                                }
                            }
                            {
                                // grCn_803E214C[xF4][group_offset]: `-1` reads the previous row's second element
                                let anim_arg = grCn_803E214C[((*gp).u.corneria2.xF4 as isize * 2 + group_offset as isize) as usize];
                                ext::grAnime_801C8098(gobj, grCn_803E21B0[arwing_groups[(*gp).u.corneria2.xC8 as usize] as usize], 7, anim_arg, 0.0, 1.0);
                            }
                        } else {
                            HSD_JObjSetRotationZ(
                                ext::Ground_801C3FA4(gobj, grCn_803E21B0[arwing_groups[(*gp).u.corneria2.xC8 as usize] as usize]),
                                0.0,
                            );
                        }
                        (*gp).u.corneria2.xF8 -= 1;
                    }
                    1..=4 => {
                        if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                            (*gp).u.corneria2.xF4 = 0;
                            (*gp).u.corneria2.xF8 = yp().x68 as s32;
                        }
                    }
                    _ => {}
                }
                {
                    let arwing = arwing_gobjs[(*gp).u.corneria2.xC8 as usize];
                    if !arwing.is_null() {
                        let mut arwing_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                        let arwing_gp = GET_GROUND(arwing);
                        let ship_gp = GET_GROUND(ext::Ground_GetMapGObj(3));
                        ext::lb_8000B1CC(ext::Ground_801C3FA4(arwing, 4), null_mut(), &mut arwing_pos);
                        {
                            let atype = arwing_types[(*arwing_gp).u.corneria2.xC8 as usize] as usize;
                            pos.x = (*ship_gp).u.corneria.offset_x + (-arwing_pos.z + lbl_803E2068[atype].x);
                            pos.y = arwing_pos.y + lbl_803E2068[atype].y;
                            pos.z = arwing_pos.x + lbl_803E2068[atype].z;
                        }
                    } else {
                        pos.z = 0.0;
                        pos.y = 0.0;
                        pos.x = 0.0;
                    }
                }
                if pos.y <= 100.0 {
                    pos.y = 100.0;
                }
                HSD_JObjSetTranslate(jobj, &pos);
                ext::lb_8000B1CC(
                    ext::Ground_801C3FA4(gobj, grCn_803E21B0[arwing_groups[(*gp).u.corneria2.xC8 as usize] as usize]),
                    null_mut(),
                    &mut pos,
                );
                {
                    {
                        let item = (*gp).u.corneria2.xDC;
                        if !item.is_null() {
                            let gr = (*item).user_data as *mut Ground;
                            if !gr.is_null() {
                                (*gr).u.arwing.xE0 = pos;
                            }
                        }
                    }
                    let rot_z = HSD_JObjGetRotationZ(ext::Ground_801C3FA4(
                        gobj,
                        grCn_803E21B0[arwing_groups[(*gp).u.corneria2.xC8 as usize] as usize],
                    ));
                    let mut angle = rot_z;
                    {
                        let item = (*gp).u.corneria2.xDC;
                        if !item.is_null() {
                            let gr = (*item).user_data as *mut Ground;
                            if !gr.is_null() {
                                (*gr).u.arwing.xDC = rot_z;
                            }
                        }
                    }
                    if grCn_804D69B0 == 0 {
                        if (*gp).u.corneria2.xF0 % yp().x74 == 0 && randi(yp().x78) == 0 {
                            (*gp).u.corneria2.xFC = 1;
                        } else {
                            (*gp).u.corneria2.xFC = 0;
                        }
                    } else if (*gp).u.corneria2.xF0 % yp().x7C == 0 && randi(yp().x80) == 0 {
                        (*gp).u.corneria2.xFC = 1;
                    } else {
                        (*gp).u.corneria2.xFC = 0;
                    }
                    if (*gp).u.corneria2.xFC != 0 {
                        (*gp).u.corneria2.xFC = 0;
                        if grCorneria_801DEC08(&mut pos) == 0 {
                            while (angle as f64) < -M_PI {
                                angle = (angle as f64 + M_TAU) as f32;
                            }
                            while (angle as f64) > M_PI {
                                angle = (angle as f64 - M_TAU) as f32;
                            }
                            {
                                let abs_a = ABS(angle);
                                if abs_a < 0.05235988 || ABS(angle) > 3.0892327 {
                                    ext::lbAudioAx_800237A8(0x55733, 0x7F, 0x40);
                                    if get_grp_off(gobj) == 1 {
                                        ext::it_802E72E0(gobj, ext::Ground_801C3FA4(gobj, 7), 2, -1.0, yp().x70);
                                    } else {
                                        ext::it_802E72E0(gobj, ext::Ground_801C3FA4(gobj, 5), 0, -1.0, yp().x70);
                                    }
                                    ext::grMaterial_801C9604(gobj, yp().x84, 0);
                                }
                            }
                        }
                    }
                }
            }
            10..=13 => {
                if grCn_804D69B0 == 0 {
                    if (*gp).u.corneria2.xF0 % yp().x74 == 0 && randi(yp().x78) == 0 {
                        (*gp).u.corneria2.xFC = 1;
                    } else {
                        (*gp).u.corneria2.xFC = 0;
                    }
                } else if (*gp).u.corneria2.xF0 % yp().x7C == 0 && randi(yp().x80) == 0 {
                    (*gp).u.corneria2.xFC = 1;
                } else {
                    (*gp).u.corneria2.xFC = 0;
                }
                {
                    let far_arwing = arwing_gobjs[(*gp).u.corneria2.xC8 as usize];
                    if !far_arwing.is_null() {
                        let mut arwing_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                        let arwing_gp = (*far_arwing).user_data as *mut Ground;
                        let ship_gp = GET_GROUND(ext::Ground_GetMapGObj(3));
                        ext::lb_8000B1CC(ext::Ground_801C3FA4(far_arwing, 4), null_mut(), &mut arwing_pos);
                        {
                            let atype = arwing_types[(*arwing_gp).u.corneria2.xC8 as usize] as usize;
                            pos.x = (*ship_gp).u.corneria.offset_x + (-arwing_pos.z + lbl_803E2068[atype].x);
                            pos.y = arwing_pos.y + lbl_803E2068[atype].y;
                            pos.z = arwing_pos.x + lbl_803E2068[atype].z;
                        }
                    } else {
                        pos.z = 0.0;
                        pos.y = 0.0;
                        pos.x = 0.0;
                    }
                }
                if pos.y <= 100.0 {
                    pos.y = 100.0;
                }
                if 500.0 < pos.x && pos.x < grCn_ArwingMaxX && (*gp).u.corneria2.xFC != 0 {
                    (*gp).u.corneria2.xFC = 0;
                    {
                        let mut player = ext::HSD_Randi(4);
                        for tries in 0..4 {
                            player = (player + tries) & 3;
                            if ext::Player_GetPlayerSlotType(player) != Gm_PKind_NA {
                                break;
                            }
                        }
                        ext::Player_LoadPlayerCoords(player, &mut pos);
                    }
                    pos.y += 5.0;
                    ext::lbAudioAx_800237A8(0x55734, 0x7F, 0x40);
                    {
                        if get_grp_off(gobj) == 1 {
                            ext::it_802E7654(gobj, ext::Ground_801C3FA4(gobj, 7), &mut pos, 3, 3, yp().x70);
                        } else {
                            if (*gp).u.corneria2.x100 != 0 {
                                ext::it_802E7654(gobj, ext::Ground_801C3FA4(gobj, 5), &mut pos, 1, 3, yp().x70);
                            } else {
                                ext::it_802E7654(gobj, ext::Ground_801C3FA4(gobj, 6), &mut pos, 1, 3, yp().x70);
                            }
                            (*gp).u.corneria2.x100 = ((*gp).u.corneria2.x100 + 1) & 1;
                        }
                    }
                    ext::grMaterial_801C9604(gobj, yp().x84, 0);
                }
            }
            _ => {}
        }
        (*gp).u.corneria2.xF0 += 1;
        return;
    }
    {
        let mut item = (*gp).u.corneria2.xE0;
        if !item.is_null() {
            ext::Ground_801C4A08(item);
        }
        item = (*gp).u.corneria2.xE4;
        if !item.is_null() {
            ext::Ground_801C4A08(item);
        }
        item = (*gp).u.corneria2.xE8;
        if !item.is_null() {
            ext::Ground_801C4A08(item);
        }
        item = (*gp).u.corneria2.xEC;
        if !item.is_null() {
            ext::Ground_801C4A08(item);
        }
        ext::Ground_801C4A08(gobj);
    }
}

/// `static inline randi` (baselib/random.h): `HSD_Randi` that accepts an empty range.
unsafe fn randi(max_val: s32) -> s32 {
    if max_val != 0 {
        return ext::HSD_Randi(max_val);
    }
    0
}

pub unsafe extern "C" fn grCorneria_801DF8CC(_arg: *mut Ground_GObj) {}

/// `grCorneria_801DF8D0`: Star Fox article init (no background).
pub unsafe extern "C" fn grCorneria_801DF8D0(gobj: *mut Ground_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = GET_JOBJ(gobj);

    let scale = ext::Ground_801C0498();
    (*gp).u.corneria2.xC8 = grCn_804D69A4 as i32;
    Ground_ClearStarFoxArwingGObjs(gp);
    Ground_AnimateStarFoxArwing(gobj);

    match arwing_types[(*gp).u.corneria2.xC8 as usize] {
        1..=9 => {
            let arwing = grCorneria_801DD534(grCn_803E217C[arwing_groups[(*gp).u.corneria2.xC8 as usize] as usize]);
            Ground_LinkStarFoxArwing(gp, arwing);
        }
        10..=13 => {
            Ground_AttachStarFoxArwingModel(gobj, gp, arwing_gobjs[(*gp).u.corneria2.xC8 as usize], 4);
        }
        _ => {
            Ground_DisableStarFoxArwingGObjs(gp);
        }
    }
    HSD_JObjSetScaleX(jobj, scale * yp().x70);
    HSD_JObjSetScaleY(jobj, scale * yp().x70);
    HSD_JObjSetScaleZ(jobj, scale * yp().x70);
    Ground_ResetStarFoxArwingState(gp);
    (*gp).u.corneria2.xF8 = yp().x68 as s32;
    (*gp).u.corneria2.xFC = 0;
}

pub unsafe extern "C" fn grCorneria_801DFBC4(_arg: *mut Ground_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grCorneria_801DFBCC(arg: *mut Ground_GObj) {
    grCorneria_801DED50(arg);
}

pub unsafe extern "C" fn grCorneria_801DFBEC(_arg: *mut Ground_GObj) {}

/// `static stageGObj6_OnInit`.
pub unsafe extern "C" fn stageGObj6_OnInit(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
}

pub unsafe extern "C" fn grCorneria_801DFC1C(_arg: *mut Ground_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grCorneria_801DFC24(_arg: *mut Ground_GObj) {}

pub unsafe extern "C" fn grCorneria_801DFC28(_arg: *mut Ground_GObj) {}

/// `grCorneria_801DFC2C`: background 8 init.
pub unsafe extern "C" fn grCorneria_801DFC2C(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);

    (*gp).u.corneria.xC6.value = 0;
    (*gp).u.corneria.offset_y.flags.set_b0(0);
    ext::memzero(addr_of_mut!((*gp).u.corneria.xC8) as *mut c_void, 0x18);
    (*gp).u.corneria.xC4.value = 0;
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grCorneria_801DFC90(_arg: *mut Ground_GObj) -> i32 {
    0
}

/// `grCorneria_801DFC98`: background 8 scroll.
pub unsafe extern "C" fn grCorneria_801DFC98(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    if (*gp).u.corneria.xC4.value == 0 && (*gp).u.corneria.offset_y.flags.b0() != 1 {
        HSD_JObjAddTranslationX(jobj, yp().x88);
        HSD_JObjSetTranslateY(jobj, -grCorneria_801E2EA0());

        match (*gp).u.corneria.xC5 as i8 {
            0 => {
                if HSD_JObjGetTranslationX(jobj) >= 3200.0f32 * ext::Ground_801C0498() / 2.0 + -1400.0 {
                    grCorneria_801E03C8(gobj, 9);
                    (*gp).u.corneria.xC5 = 1;
                    return;
                }
            }
            1 => {
                if HSD_JObjGetTranslationX(jobj) >= 3200.0f32 * ext::Ground_801C0498() / 2.0 + 1400.0 {
                    ext::Ground_801C4A08(gobj);
                }
            }
            _ => {}
        }
    }
}

pub unsafe extern "C" fn grCorneria_801DFEB4(_arg: *mut Ground_GObj) {}

/// `grCorneria_801DFEB8`: background 9 init.
pub unsafe extern "C" fn grCorneria_801DFEB8(gobj: *mut Ground_GObj) {
    let gp = (*gobj).user_data as *mut Ground;

    (*gp).u.corneria.xC6.flags.set_b0(0);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).u.corneria.xC4.value = 0;
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grCorneria_801DFF18(_arg: *mut Ground_GObj) -> i32 {
    0
}

/// `grCorneria_801DFF20`: background 9 scroll.
pub unsafe extern "C" fn grCorneria_801DFF20(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    if (*gp).u.corneria.xC4.value == 0 && (*gp).u.corneria.xC6.flags.b0() != 1 {
        HSD_JObjAddTranslationX(jobj, yp().x88);
        HSD_JObjSetTranslateY(jobj, -grCorneria_801E2EA0());

        match (*gp).u.corneria.xC5 as i8 {
            0 => {
                if HSD_JObjGetTranslationX(jobj) >= 3200.0f32 * ext::Ground_801C0498() / 2.0 + -1400.0 {
                    grCorneria_801E03C8(gobj, 4);
                    (*gp).u.corneria.xC5 = 1;
                    return;
                }
            }
            1 => {
                if HSD_JObjGetTranslationX(jobj) >= 3200.0f32 * ext::Ground_801C0498() / 2.0 + 1400.0 {
                    ext::Ground_801C4A08(gobj);
                }
            }
            _ => {}
        }
    }
}

pub unsafe extern "C" fn grCorneria_801E013C(_arg: *mut Ground_GObj) {}

/// `grCorneria_801E0140`: background 4 init.
pub unsafe extern "C" fn grCorneria_801E0140(gobj: *mut Ground_GObj) {
    let gp = (*gobj).user_data as *mut Ground;

    (*gp).u.corneria.xC6.flags.set_b0(0);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).u.corneria.xC4.value = 0;
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grCorneria_801E01A0(_arg: *mut Ground_GObj) -> i32 {
    0
}

/// `grCorneria_801E01A8`: background 4 scroll.
pub unsafe extern "C" fn grCorneria_801E01A8(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    if (*gp).u.corneria.xC4.value == 0 && (*gp).u.corneria.xC6.flags.b0() != 1 {
        HSD_JObjAddTranslationX(jobj, yp().x88);
        HSD_JObjSetTranslateY(jobj, -grCorneria_801E2EA0());

        match (*gp).u.corneria.xC5 as i8 {
            0 => {
                if HSD_JObjGetTranslationX(jobj) >= 4800.0f32 * ext::Ground_801C0498() / 2.0 + -1400.0 {
                    grCorneria_801E03C8(gobj, 8);
                    (*gp).u.corneria.xC5 = 1;
                    return;
                }
            }
            1 => {
                if HSD_JObjGetTranslationX(jobj) >= 4800.0f32 * ext::Ground_801C0498() / 2.0 + 1400.0 {
                    ext::Ground_801C4A08(gobj);
                }
            }
            _ => {}
        }
    }
}

pub unsafe extern "C" fn grCorneria_801E03C4(_arg: *mut Ground_GObj) {}

/// `grCorneria_801E03C8` (the C declares a fake `UNK_T` return and returns nothing): spawn the next background.
pub unsafe extern "C" fn grCorneria_801E03C8(gobj: *mut Ground_GObj, id: i32) {
    let jobj = GET_JOBJ(gobj);
    let x = HSD_JObjGetTranslationX(jobj);
    let new_gobj = grCorneria_801DD534(id);
    let gp = GET_GROUND(new_gobj);

    if id == 8 {
        let half = 3200.0f32 * ext::Ground_801C0498() / 2.0;
        HSD_JObjSetTranslateX((*new_gobj).hsd_obj as *mut HSD_JObj, x - (4800.0f32 * ext::Ground_801C0498() / 2.0 + half));
        (*gp).u.corneria.xC5 = 0;
    } else if id == 9 {
        let half = 3200.0f32 * ext::Ground_801C0498() / 2.0;
        HSD_JObjSetTranslateX(GET_JOBJ(new_gobj), x - (3200.0f32 * ext::Ground_801C0498() / 2.0 + half));
        (*gp).u.corneria.xC5 = 0;
    } else if id == 4 {
        let half = 4800.0f32 * ext::Ground_801C0498() / 2.0;
        HSD_JObjSetTranslateX(GET_JOBJ(new_gobj), x - (3200.0f32 * ext::Ground_801C0498() / 2.0 + half));
        (*gp).u.corneria.xC5 = 0;
    }
    if !new_gobj.is_null() {
        ext::HSD_GObjGXLink_80390908(new_gobj, 3, 0);
    }
}

/// `grCorneria_801E0678`: park the three backgrounds for the demo.
pub unsafe extern "C" fn grCorneria_801E0678() {
    let mut gobj = ext::Ground_GetMapGObj(0xB);
    if !gobj.is_null() {
        ext::Ground_801C4A08(gobj);
    }
    gobj = ext::Ground_GetMapGObj(8);
    HSD_JObjSetTranslateX((*gobj).hsd_obj as *mut HSD_JObj, 0.0);
    let mut gp = GET_GROUND(gobj);
    (*gp).u.corneria.offset_y.flags.set_b0(1);

    gobj = ext::Ground_GetMapGObj(9);
    gp = GET_GROUND(gobj);
    (*gp).u.corneria.xC6.flags.set_b0(1);
    {
        let half = 3200.0f32 * ext::Ground_801C0498() / 2.0;
        HSD_JObjSetTranslateX((*gobj).hsd_obj as *mut HSD_JObj, -(3200.0f32 * ext::Ground_801C0498() / 2.0 + half));
    }

    gobj = ext::Ground_GetMapGObj(4);
    gp = GET_GROUND(gobj);
    (*gp).u.corneria.xC6.flags.set_b0(1);
    {
        let half = 4800.0f32 * ext::Ground_801C0498() / 2.0;
        HSD_JObjSetTranslateX((*gobj).hsd_obj as *mut HSD_JObj, 3200.0f32 * ext::Ground_801C0498() / 2.0 + half);
    }
}

/// `grCorneria_801E08CC`: which background the camera is over (asserts when none).
pub unsafe extern "C" fn grCorneria_801E08CC() -> i32 {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::Camera_GetTransformInterest(&mut pos);
    let cam_x = pos.x;
    let mut gobj = ext::Ground_GetMapGObj(8);
    if !gobj.is_null() {
        let x = HSD_JObjGetTranslationX((*gobj).hsd_obj as *mut HSD_JObj);
        if cam_x > -(3200.0f32 * ext::Ground_801C0498() / 2.0 - x) && cam_x < 3200.0f32 * ext::Ground_801C0498() / 2.0 + x {
            return 8;
        }
    }
    gobj = ext::Ground_GetMapGObj(9);
    if !gobj.is_null() {
        let x = HSD_JObjGetTranslationX((*gobj).hsd_obj as *mut HSD_JObj);
        if cam_x > -(3200.0f32 * ext::Ground_801C0498() / 2.0 - x) && cam_x < 3200.0f32 * ext::Ground_801C0498() / 2.0 + x {
            return 9;
        }
    }
    gobj = ext::Ground_GetMapGObj(4);
    if !gobj.is_null() {
        let x = HSD_JObjGetTranslationX((*gobj).hsd_obj as *mut HSD_JObj);
        if cam_x > -(4800.0f32 * ext::Ground_801C0498() / 2.0 - x) && cam_x < 4800.0f32 * ext::Ground_801C0498() / 2.0 + x {
            return 4;
        }
    }
    assert_(false, 0x9AC, "0");
    -1
}

/// `grCorneria_801E0A74`: the same lookup for an x coordinate.
pub unsafe extern "C" fn grCorneria_801E0A74(arg0: *const f32) -> i32 {
    let mut gobj = ext::Ground_GetMapGObj(8);
    if !gobj.is_null() {
        let x = HSD_JObjGetTranslationX((*gobj).hsd_obj as *mut HSD_JObj);
        if *arg0 > x - 3200.0f32 * ext::Ground_801C0498() / 2.0 && *arg0 < x + 3200.0f32 * ext::Ground_801C0498() / 2.0 {
            return 8;
        }
    }
    gobj = ext::Ground_GetMapGObj(9);
    if !gobj.is_null() {
        let x = HSD_JObjGetTranslationX((*gobj).hsd_obj as *mut HSD_JObj);
        if *arg0 > x - 3200.0f32 * ext::Ground_801C0498() / 2.0 && *arg0 < x + 3200.0f32 * ext::Ground_801C0498() / 2.0 {
            return 9;
        }
    }
    gobj = ext::Ground_GetMapGObj(4);
    if !gobj.is_null() {
        let x = HSD_JObjGetTranslationX((*gobj).hsd_obj as *mut HSD_JObj);
        if *arg0 > x - 4800.0f32 * ext::Ground_801C0498() / 2.0 && *arg0 < x + 4800.0f32 * ext::Ground_801C0498() / 2.0 {
            return 4;
        }
    }
    ext::OSReport(b"grcorneria.c    grCorneriaGetPosMapKind2\n\0".as_ptr() as *const i8, &[]);
    assert_(false, 0x9CB, "0");
    -1
}

/// `grCorneria_801E0C3C`: Venom / Corneria backdrop (id 7) init.
pub unsafe extern "C" fn grCorneria_801E0C3C(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*GET_GROUND(gobj)).x10_flags.set_b2(0);
    let a = ext::Ground_801C3FA4(gobj, 5);
    let b = ext::Ground_801C3FA4(gobj, 2);
    let c = ext::Ground_801C3FA4(gobj, 4);
    let d = ext::Ground_801C3FA4(gobj, 3);
    let e = ext::Ground_801C3FA4(gobj, 7);
    let f = ext::Ground_801C3FA4(gobj, 6);
    ext::Ground_801C4E70(a, b, c, d, e, f);
    ext::grMaterial_801C8A04(jobj, JOBJ_UNK_B27 as u32);
    ext::grMaterial_801C8858(jobj, JOBJ_ROOT_XLU as u32);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grCorneria_801E0D28(_arg: *mut Ground_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grCorneria_801E0D30(gobj: *mut Ground_GObj) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if !jobj.is_null() {
        HSD_JObjSetTranslateY(jobj, -grCorneria_801E2EA0());
    }
}

pub unsafe extern "C" fn grCorneria_801E0DE0(_arg: *mut Ground_GObj) {}

pub unsafe extern "C" fn grCorneria_801E0DE4(gobj: *mut Ground_GObj) {
    let gr = GET_GROUND(gobj);
    smashTaunt_801E2550(gobj, addr_of_mut!((*gr).u.smashtaunt));
}

pub unsafe extern "C" fn grCorneria_801E0E0C(_arg: *mut Ground_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grCorneria_801E0E14(gobj: *mut Ground_GObj) {
    let gr = GET_GROUND(gobj);
    grCorneria_801E277C(gobj, addr_of_mut!((*gr).u.smashtaunt));
}

pub unsafe extern "C" fn grCorneria_801E0E3C(_arg: *mut Ground_GObj) {}

pub static grCn_803E21F0: [[i16; 2]; 5] = [[0, 0], [1, 6], [2, 7], [3, 8], [4, 9]];

/// `static grCn_Entry grCn_803E2204[20][5]` (talk table: `{ line, sis index, sound id }`).
pub static grCn_803E2204: [[[i32; 3]; 5]; 20] = [
    [ [1, 7, 350014], [4, 8, 350015], [2, 9, 350016], [0, 0, 0], [0, 0, 0] ],
    [ [2, 10, 350017], [1, 11, 350018], [4, 12, 350019], [0, 0, 0], [0, 0, 0] ],
    [ [4, 13, 350020], [1, 14, 350021], [2, 15, 350022], [0, 0, 0], [0, 0, 0] ],
    [ [2, 16, 350023], [1, 17, 350024], [4, 18, 350025], [0, 0, 0], [0, 0, 0] ],
    [ [1, 47, 100119], [4, 48, 100120], [3, 49, 100121], [0, 0, 0], [0, 0, 0] ],
    [ [3, 36, 100108], [1, 11, 350018], [4, 12, 350019], [0, 0, 0], [0, 0, 0] ],
    [ [4, 44, 100116], [1, 45, 100117], [3, 46, 100118], [0, 0, 0], [0, 0, 0] ],
    [ [4, 50, 100122], [1, 51, 100123], [3, 52, 100124], [0, 0, 0], [0, 0, 0] ],
    [ [4, 19, 110112], [2, 20, 110113], [4, 21, 110114], [1, 22, 110115], [0, 0, 0] ],
    [ [1, 23, 110116], [4, 24, 110117], [2, 25, 110118], [0, 0, 0], [0, 0, 0] ],
    [ [4, 26, 110119], [1, 27, 110120], [2, 28, 110121], [0, 0, 0], [0, 0, 0] ],
    [ [1, 29, 110122], [1, 30, 110123], [1, 31, 110124], [4, 32, 110125], [0, 0, 0] ],
    [ [2, 33, 110126], [1, 34, 110127], [4, 35, 110128], [0, 0, 0], [0, 0, 0] ],
    [ [3, 36, 100108], [4, 37, 100109], [1, 38, 100110], [1, 39, 100111], [0, 0, 0] ],
    [ [1, 40, 100112], [4, 41, 100113], [3, 42, 100114], [4, 43, 100115], [0, 0, 0] ],
    [ [4, 44, 100116], [1, 45, 100117], [3, 46, 100118], [0, 0, 0], [0, 0, 0] ],
    [ [1, 47, 100119], [4, 48, 100120], [3, 49, 100121], [0, 0, 0], [0, 0, 0] ],
    [ [4, 50, 100122], [1, 51, 100123], [3, 52, 100124], [0, 0, 0], [0, 0, 0] ],
    [ [4, 5, 350012], [0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0] ],
    [ [4, 6, 350013], [0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0] ],
];

/// `grCorneria_801E0E40`: debug talk-number selection with L/R.
pub unsafe extern "C" fn grCorneria_801E0E40() {
    if ext::Camera_80030178() == 0 && ext::Camera_80030154() == 0 {
        return;
    }

    let trigger = (*ext::HSD_PadMasterStatus().add(1)).trigger;
    if (trigger & (PAD_TRIGGER_L | PAD_TRIGGER_R)) == 0 {
        return;
    }

    if !ext::Ground_GetMapGObj(0xC).is_null() {
        return;
    }

    if trigger & PAD_TRIGGER_L != 0 {
        grCn_804D466C -= 1;
    } else {
        grCn_804D466C += 1;
    }

    if grCn_804D466C < 0 {
        grCn_804D466C = 0;
    }
    if (grCn_804D466C as u32) >= 0x14 {
        grCn_804D466C = 0x13;
    }

    ext::OSReport(b"talk no = %d\n\0".as_ptr() as *const i8, &[grCn_804D466C as u32 as usize]);
    let wgobj = grCorneria_801DD534(0xC);
    assert_(!wgobj.is_null(), 0xAC3, "wgobj");

    grCorneria_801E0F34(wgobj, grCn_804D466C);
}

pub unsafe extern "C" fn grCorneria_801E0F30(_arg: *mut Ground_GObj) {}

/// `grCorneria_801E0F34`: select the talk sequence `val`.
pub unsafe extern "C" fn grCorneria_801E0F34(gobj: *mut Ground_GObj, mut val: i32) {
    let gp = GET_GROUND(gobj);
    (*gp).u.arwing.xCC = 0;
    (*gp).u.arwing.xC8 = 0;
    if val < 0 {
        val = 0;
    }
    if (val as u32) >= 0x14 {
        val = 0x13;
    }
    (*gp).u.arwing.xC4 = val as u32;
}

pub unsafe extern "C" fn grCorneria_801E0F64(_arg: *mut Ground_GObj) -> i32 {
    0
}

/// `static inline Ground_UpdateStarFoxSequence` called with `(gobj, gp, 5, grCorneria_801DD534)`.
pub unsafe extern "C" fn grCorneria_801E0F6C(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::ifStatus_802F6898();
    ext::un_802FF570();

    if ext::Ground_GetMapGObj(5).is_null() {
        if grCorneria_801E2598((*gp).u.starfox.xC4.word, (*gp).u.starfox.arwing_slot as u32) != 0 {
            let old = (*gp).u.starfox.xCC;
            (*gp).u.starfox.xCC = old.wrapping_sub(1);
            if old < 0 {
                let sequence_gobj = grCorneria_801DD534(5);
                grCorneria_801E2738(
                    sequence_gobj,
                    addr_of_mut!((*GET_GROUND(sequence_gobj)).u) as *mut c_void,
                    (*gp).u.starfox.xC4.word,
                    (*gp).u.starfox.arwing_slot as u32,
                );
                (*gp).u.starfox.arwing_slot += 1;
                (*gp).u.starfox.xCC = 0;
            }
        } else {
            let old = (*gp).u.starfox.xCC;
            (*gp).u.starfox.xCC = old.wrapping_sub(1);
            if old < 0 {
                ext::Ground_801C4A08(gobj);
            }
        }
    }
}

pub unsafe extern "C" fn grCorneria_801E1030(_gobj: *mut Ground_GObj) {
    ext::ifStatus_802F68F0();
    ext::un_802FF620();
}

pub unsafe extern "C" fn grCorneria_801E1054(_arg: *mut Ground_GObj) {}

pub unsafe extern "C" fn grCorneria_801E1058(_arg: *mut Ground_GObj) -> i32 {
    0
}

/// `static inline Fake_HSD_JObjSetTranslateX`.
unsafe fn Fake_HSD_JObjSetTranslateX(jobj: *mut HSD_JObj, x: f32) {
    if jobj.is_null() {
        return;
    }
    (*jobj).translate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        crate::tu::grinl::HSD_JObjSetMtxDirty(jobj);
    }
}

/// `grCorneria_801E1060`: background scroll from the camera.
pub unsafe extern "C" fn grCorneria_801E1060(_gobj: *mut Ground_GObj) {
    let mut pos = ext::gm_801883C0();
    let total = 3200.0f32 * ext::Ground_801C0498();
    let a = 4800.0f32 * ext::Ground_801C0498() / 2.0;
    let b = 3200.0f32 * ext::Ground_801C0498() / 2.0;
    pos *= (a + (total + b)) / 2400.0;

    let mut bg = ext::Ground_GetMapGObj(8);
    if !bg.is_null() {
        Fake_HSD_JObjSetTranslateX((*bg).hsd_obj as *mut HSD_JObj, pos);
    }
    let c = 3200.0f32 * ext::Ground_801C0498() / 2.0;
    let d = 3200.0f32 * ext::Ground_801C0498() / 2.0;
    pos = pos - (c + d);

    bg = ext::Ground_GetMapGObj(9);
    if !bg.is_null() {
        Fake_HSD_JObjSetTranslateX((*bg).hsd_obj as *mut HSD_JObj, pos);
    }
    let e = 3200.0f32 * ext::Ground_801C0498() / 2.0;
    let f = 4800.0f32 * ext::Ground_801C0498() / 2.0;
    pos = pos - (e + f);

    bg = ext::Ground_GetMapGObj(4);
    if !bg.is_null() {
        Fake_HSD_JObjSetTranslateX((*bg).hsd_obj as *mut HSD_JObj, pos);
    }
}

pub unsafe extern "C" fn grCorneria_801E12CC(_arg: *mut Ground_GObj) {}

pub unsafe extern "C" fn fn_801E12D0(_gobj: *mut Item_GObj, _gr: *mut Ground) {}

/// `fn_801E12D4`: the Great Fox cannon takes a beam hit.
pub unsafe extern "C" fn fn_801E12D4(_gobj: *mut Item_GObj, gr: *mut Ground, _pos: *mut Vec3, _arg3: *mut HSD_GObj, damage: f32) {
    (*gr).u.corneria.x114 += damage;
    ext::OSReport(b"*** Beam Damage! %f\n\0".as_ptr() as *const i8, &[(damage as f64).to_bits() as usize]);
    if (*gr).u.corneria.x114 >= yp().x38 {
        (*gr).u.corneria.x108 = 4;
        ext::grMaterial_801C8E28((*gr).u.corneria.left_cannon);
        ext::grMaterial_801C8E28((*gr).u.corneria.right_cannon);
    }
}

/// `grCorneria_801E1348`: Great Fox cannon state machine.
pub unsafe extern "C" fn grCorneria_801E1348(gobj: *mut Ground_GObj) {
    let mut sp40 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut cannon_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut fighter_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = GET_GROUND(gobj);

    match (*gp).u.corneria.x108 {
        0 => {
            if (*gp).u.corneria.x110 == 0 {
                if ext::HSD_Randf() <= yp().x24 {
                    (*gp).u.corneria.x119 = 0;
                    (*gp).u.corneria.x118 = 1;
                    (*gp).u.corneria.x10C = yp().x28 as s32;
                    ext::Ground_801C53EC(0x55739);
                } else {
                    let imin = yp().x30 as s32;
                    let mut imax = yp().x34 as s32;
                    if imax > imin {
                        let range = imax - imin;
                        imax = imin + if range != 0 { ext::HSD_Randi(range) } else { 0 };
                    } else if imax < imin {
                        let range = imin - imax;
                        imax += if range != 0 { ext::HSD_Randi(range) } else { 0 };
                    }
                    (*gp).u.corneria.x119 = 1;
                    (*gp).u.corneria.x118 = imax as i8;
                    (*gp).u.corneria.x10C = yp().x2C as s32;
                    ext::Ground_801C53EC(0x5573A);
                }
                sp40.x = grCn_803E1FAC[1].x;
                sp40.y = grCn_803E1FAC[1].y;
                sp40.z = grCn_803E1FAC[1].z;
                ext::grLib_801C9808(0xBC, 0, ext::Ground_801C3FA4(gobj, 2));
                ext::grLib_801C9808(0xBC, 0, ext::Ground_801C3FA4(gobj, 3));
                ext::Item_80268E5C((*gp).u.corneria.left_cannon, 2, ITEM_ANIM_UPDATE as _);
                ext::Item_80268E5C((*gp).u.corneria.right_cannon, 2, ITEM_ANIM_UPDATE as _);
                ext::grMaterial_801C8E28((*gp).u.corneria.left_cannon);
                ext::grMaterial_801C8E28((*gp).u.corneria.right_cannon);
                (*gp).u.corneria.x108 = 1;
                return;
            }
            (*gp).u.corneria.x110 -= 1;
        }
        1 => {
            if (*gp).u.corneria.x10C == 0 {
                ext::hsd_8039D580(ext::Ground_801C3FA4(gobj, 2));
                ext::hsd_8039D580(ext::Ground_801C3FA4(gobj, 3));
                ext::Item_80268E5C((*gp).u.corneria.left_cannon, 0, ITEM_ANIM_UPDATE as _);
                ext::Item_80268E5C((*gp).u.corneria.right_cannon, 0, ITEM_ANIM_UPDATE as _);
                (*gp).u.corneria.x10C = yp().x20 as s32;
                (*gp).u.corneria.x108 = 2;
                return;
            }
            (*gp).u.corneria.x10C -= 1;
        }
        2 => {
            let mut found = 0;
            ext::lb_8000B1CC(ext::Ground_801C3FA4(gobj, 2), null_mut(), &mut cannon_pos);
            {
                let mut fighter = ext::HSD_GObjPLinkHead(HSD_GOBJ_PLINK_FIGHTER);
                while !fighter.is_null() {
                    ext::ftLib_80086644(fighter, &mut fighter_pos);
                    if ABS(cannon_pos.y - fighter_pos.y) <= 10.0 {
                        found = 1;
                        break;
                    }
                    fighter = (*fighter).next;
                }
            }
            if found != 0 {
                (*gp).u.corneria.x10C = 0;
            }
            if (*gp).u.corneria.x10C == 0 {
                (*gp).u.corneria.x108 = 3;
                ext::grMaterial_801C8E08((*gp).u.corneria.left_cannon);
                ext::grMaterial_801C8E08((*gp).u.corneria.right_cannon);
                return;
            }
            (*gp).u.corneria.x10C -= 1;
        }
        3 => {
            if (*gp).u.corneria.x118 != 0 {
                if (*gp).u.corneria.x110 == 0 {
                    let side = (*gp).u.corneria.x11A as s32;
                    sp40.x = 20.0 + grCn_803E1F70[1].x;
                    sp40.y = 2.0 + grCn_803E1F70[1].y;
                    let cannon;
                    if side != 0 {
                        sp40.z = 9.0;
                        cannon = ext::Ground_801C3FA4(gobj, 2);
                    } else {
                        cannon = ext::Ground_801C3FA4(gobj, 3);
                        sp40.z = -9.0;
                    }
                    ext::grLib_801C9808(0xC0, 0, cannon);
                    ext::it_802EAF34(gobj, &mut sp40, (*gp).u.corneria.x119 as i8 as i32);
                    ext::Ground_801C53EC(0x55735);
                    (*gp).u.corneria.x11A ^= 1;
                    (*gp).u.corneria.x110 = 0xA;
                    (*gp).u.corneria.x118 -= 1;
                    return;
                }
                (*gp).u.corneria.x110 -= 1;
                return;
            }
            {
                let rand_range = yp().x18 as s32;
                let range = yp().x18 as s32;
                (*gp).u.corneria.x110 = (yp().x1C + (if range != 0 { ext::HSD_Randi(rand_range) } else { 0 }) as f32) as s32;
            }
            (*gp).u.corneria.x10C = 0;
            (*gp).u.corneria.x108 = 0;
        }
        4 => {
            ext::grLib_801C9808(0xE3, 0, ext::Ground_801C3FA4(gobj, 2));
            ext::grLib_801C9808(0xE3, 0, ext::Ground_801C3FA4(gobj, 3));
            sp40.x = grCn_803E1FAC[1].x;
            sp40.y = grCn_803E1FAC[1].y;
            sp40.z = grCn_803E1FAC[1].z;
            ext::Camera_RequestQuake(QuakeKind_Large, &mut sp40);
            ext::Ground_801C53EC(0x76);
            ext::HSD_JObjSetFlags(ext::Ground_801C3FA4(gobj, 6), JOBJ_HIDDEN as u32);
            ext::HSD_JObjClearFlags(ext::Ground_801C3FA4(gobj, 5), JOBJ_HIDDEN as u32);
            ext::mpLib_80057BC0(4);
            (*gp).u.corneria.x108 = 5;
        }
        _ => {}
    }
}

/// `grCorneria_801E1878`: copy the ship's translation onto the cannon target.
pub unsafe extern "C" fn grCorneria_801E1878(gobj: *mut Ground_GObj) {
    let gr = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let target_jobj = (*(*gr).u.corneria.x128).hsd_obj as *mut HSD_JObj;
    {
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        HSD_JObjGetTranslation(jobj, &mut pos);
        HSD_JObjSetTranslate(target_jobj, &pos);
    }

    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

/// `grCorneria_801E1970`: Great Fox wobble.
pub unsafe extern "C" fn grCorneria_801E1970(gobj: *mut Ground_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    if (*gp).u.corneria.x104 <= 0 {
        let mut randf = ext::HSD_Randf();
        let a = yp().x4 - yp().x0;
        (*gp).u.corneria.x104 = fmadds(a, randf, yp().x0) as s32;
        randf = ext::HSD_Randf();
        (*gp).u.corneria.xF4 = yp().x8 * (2.0 * (randf - 0.5));
        randf = ext::HSD_Randf();
        (*gp).u.corneria.xF8 = yp().x8 * (2.0 * (randf - 0.5));
    }
    (*gp).u.corneria.x104 -= 1;
    (*gp).u.corneria.xE4.z += (*gp).u.corneria.xF4;
    (*gp).u.corneria.xF0 += (*gp).u.corneria.xF8;
    {
        let vel = (*gp).u.corneria.xE4.z;
        let max = yp().xC;
        if vel > max {
            (*gp).u.corneria.xE4.z = max;
        } else if vel < -max {
            (*gp).u.corneria.xE4.z = -max;
        }
    }
    {
        let vel = (*gp).u.corneria.xF0;
        let max = yp().xC;
        if vel > max {
            (*gp).u.corneria.xF0 = max;
        } else if vel < -max {
            (*gp).u.corneria.xF0 = -max;
        }
    }
    (*gp).u.corneria.xE4.x = (*gp).u.corneria.offset_x;
    (*gp).u.corneria.xE4.y = (*gp).u.corneria.offset_y.val;
    (*gp).u.corneria.offset_x += (*gp).u.corneria.xE4.z;
    (*gp).u.corneria.offset_y.val += (*gp).u.corneria.xF0;
    {
        let offset = (*gp).u.corneria.offset_x;
        let limit = yp().x10;
        if offset > limit {
            (*gp).u.corneria.xF4 = -yp().x8;
        } else if offset < -limit {
            (*gp).u.corneria.xF4 = yp().x8;
        }
    }
    {
        let offset = (*gp).u.corneria.offset_y.val;
        let limit = yp().x14;
        if offset > limit {
            (*gp).u.corneria.xF8 = -yp().x8;
        } else if offset < -limit {
            (*gp).u.corneria.xF8 = yp().x8;
        }
    }
    let pos = Vec3 {
        x: (*gp).u.corneria.base_x + (*gp).u.corneria.offset_x,
        y: (*gp).u.corneria.base_y + (*gp).u.corneria.offset_y.val,
        z: 0.0,
    };
    HSD_JObjSetTranslate(jobj, &pos);
}

/// `grCorneria_801E1BF0`: reset and return the Great Fox gobj (id 2).
pub unsafe extern "C" fn grCorneria_801E1BF0() -> *mut HSD_GObj {
    let gobj = grCorneria_801DD534(2);
    assert_(!gobj.is_null(), 0xC70, "gobj");
    let gp = GET_GROUND(gobj);
    assert_(!gp.is_null(), 0xC71, "gp");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    assert_(!jobj.is_null(), 0xC72, "jobj");
    ext::HSD_JObjRemoveAnimAll(jobj);
    HSD_JObjSetTranslateX(jobj, 0.0);
    HSD_JObjSetTranslateY(jobj, 0.0);
    HSD_JObjSetTranslateZ(jobj, 0.0);
    HSD_JObjSetRotationX(jobj, 0.0);
    HSD_JObjSetRotationY(jobj, 0.0);
    HSD_JObjSetRotationZ(jobj, 0.0);
    HSD_JObjSetScaleX(jobj, 1.0);
    HSD_JObjSetScaleY(jobj, 1.0);
    HSD_JObjSetScaleZ(jobj, 1.0);
    (*gp).u.corneria.xC4.flags.set_b0(1);
    gobj
}

pub static grCn_804DB218: GXColor = GXColor { r: 0xAE, g: 0xBA, b: 0xD6, a: 0xFF };
pub static grCn_804DB21C: GXColor = GXColor { r: 0xD7, g: 0xE4, b: 0xF6, a: 0xFF };
pub static grCn_804DB220: GXColor = GXColor { r: 0xE0, g: 0xEC, b: 0xFA, a: 0xFF };
pub static grCn_804DB224: GXColor = GXColor { r: 0x6A, g: 0x9E, b: 0xAD, a: 0xFF };
pub static grCn_804DB228: GXColor = GXColor { r: 0x4F, g: 0x94, b: 0xB5, a: 0xFF };
pub static grCn_804DB22C: GXColor = GXColor { r: 0x49, g: 0x6E, b: 0x3F, a: 0xFF };
pub static grCn_804DB230: GXColor = GXColor { r: 0x6A, g: 0x9E, b: 0xAD, a: 0xFF };
pub static grCn_804DB234: GXColor = GXColor { r: 0x4F, g: 0x94, b: 0xB5, a: 0xFF };
pub static grCn_804DB238: GXColor = GXColor { r: 0x00, g: 0x50, b: 0x91, a: 0xFF };

/// `grCorneria_801E2110`: sky / light colours of the current background.
pub unsafe extern "C" fn grCorneria_801E2110() {
    let id = grCorneria_801E08CC();

    if id == 8 {
        let mut c1 = grCn_804DB218;
        let mut c2 = grCn_804DB21C;
        let mut c3 = grCn_804DB220;
        ext::Ground_801C058C(&mut c2);
        ext::Ground_801C05EC(&mut c1);
        ext::Ground_801C05A4(&mut c2);
        ext::Ground_801C055C(&mut c3);
        ext::Ground_801C05BC(&mut c3);
        ext::Ground_801C0574(&mut c3);
    } else if id == 9 {
        let mut c4 = grCn_804DB224;
        let mut c5 = grCn_804DB228;
        let mut c6 = grCn_804DB22C;
        ext::Ground_801C058C(&mut c5);
        ext::Ground_801C05EC(&mut c4);
        ext::Ground_801C05A4(&mut c5);
        ext::Ground_801C055C(&mut c6);
        ext::Ground_801C05BC(&mut c6);
        ext::Ground_801C0574(&mut c6);
    } else if id == 4 {
        let mut c7 = grCn_804DB230;
        let mut c8 = grCn_804DB234;
        let mut c9 = grCn_804DB238;
        ext::Ground_801C058C(&mut c8);
        ext::Ground_801C05EC(&mut c7);
        ext::Ground_801C05A4(&mut c8);
        ext::Ground_801C055C(&mut c9);
        ext::Ground_801C05BC(&mut c9);
        ext::Ground_801C0574(&mut c9);
    }
}

/// `grCorneria_801E2228`: Great Fox rise / fall.
pub unsafe extern "C" fn grCorneria_801E2228(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);

    if (*gp).u.corneria.xC4.flags.b1() == 0 {
        if (*gp).u.corneria.xC4.flags.b2() == 0 && grCorneria_801E08CC() == 4 {
            if ext::HSD_Randi(3) == 0 {
                (*gp).u.corneria.xC4.flags.set_b1(1);
                (*gp).u.corneria.x100 = 0;
                (*gp).u.corneria.xFC = 0.0;
            }
            (*gp).u.corneria.xC4.flags.set_b2(1);
        }
        if grCorneria_801E08CC() == 9 {
            (*gp).u.corneria.xC4.flags.set_b2(0);
        }
    } else {
        let timer = (*gp).u.corneria.x100;
        (*gp).u.corneria.x100 = timer + 1;
        if timer < 0x384 {
            let speed = (*gp).u.corneria.xFC;
            let t = speed / 0.005;
            let lim = fmadds(50.0, ext::Ground_801C0498(), -250.0);
            if ((*gp).u.corneria.xD0 - lim) > fmadds(speed, t, (-0.0025 * t) * t) {
                (*gp).u.corneria.xFC += 0.005;
                if (*gp).u.corneria.xFC > 3.0 {
                    (*gp).u.corneria.xFC = 3.0;
                }
            } else {
                (*gp).u.corneria.xFC -= 0.005;
                if (*gp).u.corneria.xFC < 0.005 {
                    (*gp).u.corneria.xFC = 0.005;
                }
            }
            (*gp).u.corneria.xD0 -= (*gp).u.corneria.xFC;
            if (*gp).u.corneria.xD0 < fmadds(50.0, ext::Ground_801C0498(), -250.0) {
                (*gp).u.corneria.xD0 = fmadds(50.0, ext::Ground_801C0498(), -250.0);
            }
        } else {
            let speed = (*gp).u.corneria.xFC;
            let mut t = speed / 0.005;
            if -(*gp).u.corneria.xD0 > fmadds(speed, t, (-0.0025 * t) * t) {
                t = 0.005;
                (*gp).u.corneria.xFC = speed + t;
                if (*gp).u.corneria.xFC > 3.0 {
                    (*gp).u.corneria.xFC = 3.0;
                }
            } else {
                (*gp).u.corneria.xFC = speed - 0.005;
                if (*gp).u.corneria.xFC < 0.005 {
                    (*gp).u.corneria.xFC = 0.005;
                }
            }
            (*gp).u.corneria.xD0 += (*gp).u.corneria.xFC;
            if (*gp).u.corneria.xD0 > 0.0 {
                (*gp).u.corneria.xD0 = 0.0;
                (*gp).u.corneria.xC4.flags.set_b1(0);
            }
        }
    }
}

pub unsafe extern "C" fn grCorneria_801E2454(vec: *mut Vec3, _arg1: s32) -> *mut HSD_Generator {
    ext::grLib_801C96F8(0x7534, 0x1E, vec)
}

pub unsafe extern "C" fn grCorneria_801E2480(vec: *mut Vec3, _arg1: s32) -> *mut HSD_Generator {
    ext::grLib_801C96F8(0x7530, 0x1E, vec)
}

/// `smashTaunt_801E24AC`: the smash-taunt text hides while paused.
pub unsafe extern "C" fn smashTaunt_801E24AC(gobj: *mut Ground_GObj, renderpass: i32) {
    let gp = GET_GROUND(gobj);

    if ext::gm_GetDbPauseFlag(1) != 0 || ext::gm_GetDbPauseFlag(2) != 0 || ext::Camera_8003010C() != 0 {
        if (*gp).u.smashtaunt.text.is_null() {
            return;
        }
        (*(*gp).u.smashtaunt.text).hidden = 1;
        return;
    }
    if !(*gp).u.smashtaunt.text.is_null() {
        (*(*gp).u.smashtaunt.text).hidden = 0;
    }
    ext::grDisplay_801C5DB0(gobj, renderpass);
}

pub unsafe extern "C" fn smashTaunt_801E2550(gobj: *mut Ground_GObj, gv: *mut grSmashTaunt_GroundVars) {
    (*gv).text = null_mut();
    ext::Ground_801C11AC(gobj);
    ext::ifStatus_802F6898();
    ext::un_802FF570();
    (*gobj).render_cb = Some(smashTaunt_801E24AC);
}

/// `grCorneria_801E2598`: does talk line `arg1` of sequence `arg0` exist.
pub unsafe extern "C" fn grCorneria_801E2598(arg0: u32, arg1: u32) -> s32 {
    let val = grCn_803E2204[arg0 as usize][arg1 as usize][0];
    (val != 0) as s32
}

/// `static inline grCn_HideJointPair`.
unsafe fn grCn_HideJointPair(gobj: *mut HSD_GObj, joints: *const i16) {
    let j0 = ext::Ground_801C3FA4(gobj, *joints as i32);
    if !j0.is_null() {
        ext::HSD_JObjSetFlagsAll(j0, JOBJ_HIDDEN as u32);
    }
    {
        let j1 = ext::Ground_801C3FA4(gobj, *joints.add(1) as i32);
        if !j1.is_null() {
            ext::HSD_JObjSetFlagsAll(j1, JOBJ_HIDDEN as u32);
        }
    }
}

/// `static inline grCn_SetupSmashTaunt`.
unsafe fn grCn_SetupSmashTaunt(gv: *mut grSmashTaunt_GroundVars, line: i32, arg3: i32, arg4: i32, gobj: *mut HSD_GObj) {
    (*gv).line = line as i16;
    (*gv).sis_data_idx = arg3 as i16;
    (*gv).sound_id = arg4;
    let mut i: i32 = 0;
    let mut joints: *const i16 = grCn_803E21F0.as_ptr() as *const i16;
    let joint0 = grCn_803E21F0[(*gv).line as usize][0] as i32;
    let joint1 = grCn_803E21F0[(*gv).line as usize][1] as i32;
    loop {
        if i != 0 && (*gv).line as i32 != i {
            grCn_HideJointPair(gobj, joints);
        }
        i += 1;
        joints = joints.add(2);
        if i >= 5 {
            break;
        }
    }
    (*gv).jobj0 = ext::Ground_801C3FA4(gobj, joint0);
    (*gv).jobj1 = ext::Ground_801C3FA4(gobj, joint1);
    (*gv).jobj2 = ext::Ground_801C3FA4(gobj, 5);
    (*gv).joint_idx0 = joint0 as i16;
    (*gv).joint_idx1 = joint1 as i16;
    (*gv).joint_idx2 = 5;
    (*gv).state = 0;
    (*gv).timer = 0xA;
    ext::grAnime_801C8098(gobj, joint0, 7, 0, 0.0, 1.0);
    ext::grAnime_801C8098(gobj, joint1, 7, 0, 0.0, 1.0);
    ext::grAnime_801C8098(gobj, 5, 7, 0, 0.0, 1.0);
    ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
}

pub unsafe extern "C" fn grCorneria_801E25C4(gobj: *mut HSD_GObj, gv: *mut grSmashTaunt_GroundVars, line: i32, arg3: i32, arg4: i32) {
    grCn_SetupSmashTaunt(gv, line, arg3, arg4, gobj);
}

pub unsafe extern "C" fn grCorneria_801E2738(gobj: *mut HSD_GObj, ptr: *mut c_void, idx1: u32, idx2: u32) {
    let e = &grCn_803E2204[idx1 as usize][idx2 as usize];
    grCorneria_801E25C4(gobj, ptr as *mut grSmashTaunt_GroundVars, e[0], e[1], e[2]);
}

pub static grCn_804DB24C: GXColor = GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };

/// `grCorneria_801E277C`: smash-taunt text sequence.
pub unsafe extern "C" fn grCorneria_801E277C(gobj: *mut Ground_GObj, gv: *mut grSmashTaunt_GroundVars) {
    let gp = GET_GROUND(gobj);

    if !(*gv).jobj0.is_null() {
        ext::grMaterial_801C87D0((*gv).jobj0, JOBJ_UNK_B27 as u32);
    }
    if !(*gv).jobj1.is_null() {
        ext::grMaterial_801C87D0((*gv).jobj1, JOBJ_UNK_B27 as u32);
    }
    if !(*gv).jobj2.is_null() {
        ext::grMaterial_801C87D0((*gv).jobj2, JOBJ_UNK_B27 as u32);
    }
    match (*gv).state {
        0 => {
            let timer = (*gv).timer;
            (*gv).timer = timer.wrapping_sub(1);
            if timer == 0 {
                ext::Ground_801C53EC(0xB2);
            }
            if ext::grAnime_801C83D0(gobj, (*gv).joint_idx0 as i32, 2) != 0 {
                (*gv).state = 1;
                (*gv).timer = 0x3C;
                ext::grAnime_801C8098(gobj, (*gv).joint_idx0 as i32, 7, 1, 0.0, 1.0);
                ext::grAnime_801C8098(gobj, (*gv).joint_idx1 as i32, 7, 1, 0.0, 1.0);
                ext::grAnime_801C8098(gobj, (*gv).joint_idx2 as i32, 7, 1, 0.0, 1.0);
                ext::grAnime_801C787C(gobj, (*gv).joint_idx0 as i32, 7);
                ext::grAnime_801C787C(gobj, (*gv).joint_idx1 as i32, 7);
                ext::grAnime_801C787C(gobj, (*gv).joint_idx2 as i32, 7);
                let archive = ext::grDatFiles_GetArchive();
                let sp18 = grCn_804DB24C;
                ext::HSD_SisLib_803A611C(1, null_mut(), 9, 0xD, 0, 1, 0, 7);
                *ext::HSD_SisLib_804D1124().add(1) =
                    ext::HSD_ArchiveGetPublicAddress((*archive).unk0, b"SIS_GrCorneriaData\0".as_ptr() as *const i8) as *mut SIS;
                let text = ext::HSD_SisLib_803A5ACC(1, 0, 140.0, 380.0, 0.0, 288.0, 96.0);
                (*text).text_color = sp18;
                ext::HSD_SisLib_803A6368(text, (*gv).sis_data_idx as i32);
                (*gv).text = text;
                ext::Ground_801C5440(gp, 0, (*gv).sound_id as u32);
                return;
            }
        }
        1 => {
            (*gv).timer = (*gv).timer.wrapping_sub(1);
            if (*gv).timer < 0 && ext::Ground_801C54DC(gp, 0) == 0 {
                (*gv).state = 2;
                ext::grAnime_801C8098(gobj, (*gv).joint_idx0 as i32, 7, 2, 0.0, 1.0);
                ext::grAnime_801C8098(gobj, (*gv).joint_idx1 as i32, 7, 2, 0.0, 1.0);
                ext::grAnime_801C8098(gobj, (*gv).joint_idx2 as i32, 7, 2, 0.0, 1.0);
                ext::grAnime_801C7980(gobj, (*gv).joint_idx0 as i32, 7);
                ext::grAnime_801C7980(gobj, (*gv).joint_idx1 as i32, 7);
                ext::grAnime_801C7980(gobj, (*gv).joint_idx2 as i32, 7);
                ext::HSD_SisLib_803A5F50(1);
                return;
            }
        }
        2 => {
            if ext::grAnime_801C83D0(gobj, (*gv).joint_idx0 as i32, 2) != 0 {
                ext::Ground_801C4A08(gobj);
            }
        }
        _ => {}
    }
}

/// `grCorneria_801E2A6C`.
pub unsafe extern "C" fn grCorneria_801E2A6C() {
    let si = ext::stage_info();
    if (*si).grkind == Gr_Kind_Corneria {
        let gobj = ext::Ground_GetMapGObj(12);
        if gobj.is_null() {
            return;
        }
        let gobj = ext::Ground_GetMapGObj(5);
        if !gobj.is_null() {
            ext::Ground_801C5544(GET_GROUND(gobj), 0);
            ext::Ground_801C4A08(gobj);
        }
        ext::HSD_SisLib_803A5F50(1);
    } else if (*si).grkind == Gr_Kind_Venom {
        ext::grVenom_80206CB0(0);
    }
}

/// `grCorneria_801E2AF4`.
pub unsafe extern "C" fn grCorneria_801E2AF4() {
    let si = ext::stage_info();
    if (*si).grkind == Gr_Kind_Corneria {
        let gobj = ext::Ground_GetMapGObj(12);
        if gobj.is_null() {
            return;
        }
        ext::Ground_801C4A08(gobj);
        let gobj = ext::Ground_GetMapGObj(5);
        if !gobj.is_null() {
            ext::Ground_801C5544(GET_GROUND(gobj), 0);
            ext::Ground_801C4A08(gobj);
        }
        ext::HSD_SisLib_803A5F50(1);
    } else if (*si).grkind == Gr_Kind_Venom {
        ext::grVenom_80206CB0(1);
    }
}

/// `grCorneria_801E2B80`.
pub unsafe extern "C" fn grCorneria_801E2B80() -> i32 {
    let si = ext::stage_info();
    if (*si).grkind == Gr_Kind_Corneria {
        if !ext::Ground_GetMapGObj(12).is_null() {
            return 0;
        }
        let rand = ext::HSD_Randi(5) + 8;
        let wgobj = grCorneria_801DD534(12);
        assert_(!wgobj.is_null(), 3598, "wgobj");
        grCorneria_801E0F34(wgobj, rand);
        return 1;
    } else if (*si).grkind == Gr_Kind_Venom {
        return ext::grVenom_80206BF0(2);
    }
    1
}

/// `grCorneria_801E2C34`.
pub unsafe extern "C" fn grCorneria_801E2C34() -> i32 {
    let si = ext::stage_info();
    if (*si).grkind == Gr_Kind_Corneria {
        if !ext::Ground_GetMapGObj(12).is_null() {
            return 0;
        }
        let rand = ext::HSD_Randi(5) + 13;
        let wgobj = grCorneria_801DD534(12);
        assert_(!wgobj.is_null(), 3598, "wgobj");
        grCorneria_801E0F34(wgobj, rand);
        return 1;
    } else if (*si).grkind == Gr_Kind_Venom {
        return ext::grVenom_80206BF0(20);
    }
    1
}

/// `grCorneria_801E2CE8`.
pub unsafe extern "C" fn grCorneria_801E2CE8() -> i32 {
    let si = ext::stage_info();
    ((*si).grkind == Gr_Kind_Corneria || (*si).grkind == Gr_Kind_Venom) as i32
}

/// `grCorneria_801E2D14`.
pub unsafe extern "C" fn grCorneria_801E2D14() -> i32 {
    let si = ext::stage_info();
    if (*si).grkind == Gr_Kind_Corneria {
        let gobj = ext::Ground_GetMapGObj(12);
        return (!gobj.is_null()) as i32;
    } else if (*si).grkind == Gr_Kind_Venom {
        let gobj = ext::Ground_GetMapGObj(8);
        return (!gobj.is_null()) as i32;
    }
    0
}

/// `grCorneria_801E2D90`.
pub unsafe extern "C" fn grCorneria_801E2D90(line_id: enum_t) -> i32 {
    let si = ext::stage_info();
    if (*si).grkind == Gr_Kind_Corneria && line_id != -1 {
        let joint = ext::mpJointFromLine(line_id);
        let result = joint == 0 || joint == 1 || joint == 2 || joint == 5 || joint == 6 || joint == 7;
        if result {
            return 1;
        }
    }
    0
}

/// `grCorneria_801E2E50`.
pub unsafe extern "C" fn grCorneria_801E2E50(line_id: i32) -> i32 {
    let si = ext::stage_info();
    if (*si).grkind == Gr_Kind_Corneria && line_id != -1 {
        if ext::mpJointFromLine(line_id) == 4 {
            return 1;
        }
    }
    0
}

/// `grCorneria_801E2EA0`: the ship's vertical drift.
pub unsafe extern "C" fn grCorneria_801E2EA0() -> f32 {
    let gobj = ext::Ground_GetMapGObj(3);
    if !gobj.is_null() {
        let gp = GET_GROUND(gobj);
        if !gp.is_null() {
            return (*gp).u.corneria.xD0;
        }
    }
    0.0
}

/// `grCorneria_801E2EE4` (on_touch_line).
pub unsafe extern "C" fn grCorneria_801E2EE4(_arg: enum_t) -> *mut DynamicsDesc {
    null_mut()
}

/// `grCorneria_801E2EEC` (on_check_shadow_render).
pub unsafe extern "C" fn grCorneria_801E2EEC(v: *mut Vec3, _arg1: i32, jobj: *mut HSD_JObj) -> i32 {
    let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, null_mut(), &mut sp14);
    let temp_r3 = ext::Ground_GetMapGObj(3);
    if !temp_r3.is_null() {
        let temp_r3_2 = (*temp_r3).user_data as *mut Ground;
        if !temp_r3_2.is_null() && (*temp_r3_2).u.corneria.x12C == jobj {
            let temp_f31 = 106.0f32 * ext::Ground_801C0498();
            let temp_f31_2 = fmadds((*v).y - sp14.y, (107.0f32 * ext::Ground_801C0498()) / temp_f31, sp14.x);
            if (*v).x > -fmadds(107.0, ext::Ground_801C0498(), -temp_f31_2) {
                return 0;
            }
        }
    }
    if (*v).y > sp14.y {
        return 1;
    }
    0
}

/// `grCorneria_801E2FCC`.
pub unsafe extern "C" fn grCorneria_801E2FCC() -> f32 {
    let gobj = ext::Ground_GetMapGObj(3);
    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        if !gp.is_null() {
            return fmadds(ext::Ground_801C0498(), -35.0, -(*gp).u.corneria.xD0) + 5.0;
        }
    }
    -3.4028235e38
}
