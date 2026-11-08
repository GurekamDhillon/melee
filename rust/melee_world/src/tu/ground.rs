//! Port of src/melee/gr/ground.c: the stage manager (stage table, per-stage init / load / start, the
//! Ground gobj factory, map collision joint bookkeeping, camera / blast zone setters, fog and light
//! scans, stage music selection, the SFX slot helpers) and the TARGET_PC additions (m-ex stage rows,
//! Break-the-Targets platform mod, stage slot helpers). Every function of the TU, native: pointers
//! into the game's structs, callees of other TUs through `ext` hooks. Checked against the C (whole
//! TU, shipping contraction, stateful oracle) by tests/tu_ground.rs; regenerate the C side with
//! `python rust/melee_world/gen/cref/run_gr.py ground`.
//!
//! Local records of the C file (`LightOverrideEntry`, the stage-DAT entry views, the stage-GObj
//! list nodes) are native `repr(C)` mirrors of what the C compiler lays out natively. Spins
//! (`while (1) {}` after a failed lookup) are panics.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code, unused_mut)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::vec::Vec3 as Vec3_ground;
#[allow(unused_imports)]
use melee_runtime::lbvector::euler_angles_from_onb;
#[allow(unused_imports)]
use crate::gr::ground::{GroundHooks, ground_4b50_angles, ground_5630_volume, ground_5694_pan};
#[allow(unused_imports)]
use crate::gr::bgm::{BGM_UNDEFINED, BgmPick, BgmUnlocks, RANDI_MAX as RANDI_MAX_bgm, StageParam as StageParam_bgm};
use core::ffi::c_void;
use core::mem::{offset_of, size_of};
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2i32;
use melee_runtime::gekko::sqrtf;
use melee_types::inline::*;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_80030A78() -> i32 = || unsafe { crate::tu::camera::Camera_80030A78() } as i32;
    fn Camera_SetBackgroundColor(a0: u8, a1: u8, a2: u8) = |a0: u8, a1: u8, a2: u8| unsafe { crate::tu::camera::Camera_SetBackgroundColor(a0, a1, a2) };
    fn Camera_SetModeToFixed() = || unsafe { crate::tu::camera::Camera_SetModeToFixed() };
    fn Camera_SetModeToStandard() = || unsafe { crate::tu::camera::Camera_SetModeToStandard() };
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::GObj_InitUserData(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _); } };
    fn HSD_AObjSetFlags(a0: *mut HSD_AObj, a1: u32) = |a0: *mut HSD_AObj, a1: u32| { unsafe { melee_runtime::tu::aobj::HSD_AObjSetFlags(a0 as _, a1 as _); } };
    fn HSD_AObjSetRate(a0: *mut HSD_AObj, a1: f32) = |a0: *mut HSD_AObj, a1: f32| { unsafe { melee_runtime::tu::aobj::HSD_AObjSetRate(a0 as _, a1 as _); } };
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj = || unsafe { melee_runtime::tu::cobj::HSD_CObjGetCurrent() as _ };
    fn HSD_CObjGetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { melee_runtime::tu::cobj_world::HSD_CObjGetEyePosition(a0 as _, a1 as _); } };
    fn HSD_CObjGetEyeVector(a0: *mut HSD_CObj, a1: *mut Vec) -> i32 = |a0: *mut HSD_CObj, a1: *mut Vec| unsafe { melee_runtime::tu::cobj_world::HSD_CObjGetEyeVector(a0 as _, a1 as _) as _ };
    fn HSD_FogLoadDesc(a0: *mut HSD_FogDesc) -> *mut HSD_Fog;
    fn HSD_FogSet(a0: *mut HSD_Fog);
    fn HSD_ForeachAnim(a0: *mut c_void, a1: i32, a2: u32, a3: *mut c_void, a4: i32, a5: f64, a6: u32);
    fn HSD_Free(a0: *mut c_void) = |a0: *mut c_void| { unsafe { melee_runtime::tu::memory::HSD_Free(a0 as _); } };
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObjProc_RemoveAllProcs(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjProc_RemoveAllProcs(a0 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjAddChild(a0: *mut HSD_JObj, a1: *mut HSD_JObj) = |a0: *mut HSD_JObj, a1: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjAddChild(a0 as _, a1 as _); } };
    fn HSD_JObjAddNext(a0: *mut HSD_JObj, a1: *mut HSD_JObj) = |a0: *mut HSD_JObj, a1: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjAddNext(a0 as _, a1 as _); } };
    fn HSD_JObjAlloc() -> *mut HSD_JObj = || unsafe { melee_runtime::tu::jobj::HSD_JObjAlloc() as _ };
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = |a0: *mut HSD_Joint| unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_LObjAnim(a0: *mut HSD_LObj) = |a0: *mut HSD_LObj| { unsafe { melee_runtime::tu::lobj::HSD_LObjAnim(a0 as _); } };
    fn HSD_LObjAnimAll(a0: *mut HSD_LObj) = |a0: *mut HSD_LObj| { unsafe { melee_runtime::tu::lobj::HSD_LObjAnimAll(a0 as _); } };
    fn HSD_LObjDeleteCurrentAll(a0: *mut HSD_LObj) = |a0: *mut HSD_LObj| { unsafe { melee_runtime::tu::lobj::HSD_LObjDeleteCurrentAll(a0 as _); } };
    fn HSD_LObjGetInterest(a0: *mut HSD_LObj, a1: *mut Vec) -> i32 = |a0: *mut HSD_LObj, a1: *mut Vec| unsafe { melee_runtime::tu::lobj::HSD_LObjGetInterest(a0 as _, a1 as _) as _ };
    fn HSD_LObjGetInterestWObj(a0: *mut HSD_LObj) -> *mut HSD_WObj = |a0: *mut HSD_LObj| unsafe { melee_runtime::tu::lobj::HSD_LObjGetInterestWObj(a0 as _) as _ };
    fn HSD_LObjGetPosition(a0: *mut HSD_LObj, a1: *mut Vec) -> i32 = |a0: *mut HSD_LObj, a1: *mut Vec| unsafe { melee_runtime::tu::lobj::HSD_LObjGetPosition(a0 as _, a1 as _) as _ };
    fn HSD_LObjGetPositionWObj(a0: *mut HSD_LObj) -> *mut HSD_WObj = |a0: *mut HSD_LObj| unsafe { melee_runtime::tu::lobj::HSD_LObjGetPositionWObj(a0 as _) as _ };
    fn HSD_LObjReqAnimAll(a0: *mut HSD_LObj, a1: f32) = |a0: *mut HSD_LObj, a1: f32| { unsafe { melee_runtime::tu::lobj::HSD_LObjReqAnimAll(a0 as _, a1 as _); } };
    fn HSD_LObjSetInterest(a0: *mut HSD_LObj, a1: *mut Vec) = |a0: *mut HSD_LObj, a1: *mut Vec| { unsafe { melee_runtime::tu::lobj::HSD_LObjSetInterest(a0 as _, a1 as _); } };
    fn HSD_LObjSetPosition(a0: *mut HSD_LObj, a1: *mut Vec) = |a0: *mut HSD_LObj, a1: *mut Vec| { unsafe { melee_runtime::tu::lobj::HSD_LObjSetPosition(a0 as _, a1 as _); } };
    fn HSD_LObjSetupInit(a0: *mut HSD_CObj) = |a0: *mut HSD_CObj| { unsafe { melee_runtime::tu::lobj::HSD_LObjSetupInit(a0 as _); } };
    fn HSD_LObj_803668EC(a0: *mut HSD_LObj) = |a0: *mut HSD_LObj| { unsafe { melee_runtime::tu::lobj::HSD_LObj_803668EC(a0 as _); } };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn HSD_StateInvalidate(a0: i32);
    fn HSD_WObjGetPosition(a0: *mut HSD_WObj, a1: *mut Vec) = |a0: *mut HSD_WObj, a1: *mut Vec| { unsafe { melee_runtime::tu::wobj::HSD_WObjGetPosition(a0 as _, a1 as _); } };
    fn HSD_WObjSetPosition(a0: *mut HSD_WObj, a1: *mut Vec) = |a0: *mut HSD_WObj, a1: *mut Vec| { unsafe { melee_runtime::tu::wobj::HSD_WObjSetPosition(a0 as _, a1 as _); } };
    fn Mex_Enabled(a0: *const i8) -> i32;
    fn Mex_GrBgmChance(a0: i32, a1: i32) -> i32;
    fn Mex_GrBgmCount(a0: i32) -> i32;
    fn Mex_GrBgmId(a0: i32, a1: i32) -> i32;
    fn Mex_GrCallback4() -> i32;
    fn Mex_GrCallbacks() -> *mut c_void;
    fn Mex_GrFile(a0: i32) -> *const i8;
    fn Mex_GrFlags2(a0: i32) -> u32;
    fn Mex_GrFunctionInit(a0: *mut c_void, a1: i32);
    fn Mex_GrInternalCount() -> i32;
    fn Mex_GrIsMex(a0: i32) -> i32;
    fn Mex_GrOnCheckShadowRender(a0: *mut c_void, a1: i32, a2: *mut c_void) -> i32;
    fn Mex_GrOnDemoInit(a0: i32);
    fn Mex_GrOnInit();
    fn Mex_GrOnLoad();
    fn Mex_GrOnStart();
    fn Mex_GrOnTouchLine(a0: i32) -> *mut c_void;
    fn Mex_GrRowCallbacks(a0: i32) -> *mut c_void;
    fn Mex_GrSelect(a0: i32);
    fn Mex_GrTrace() -> i32;
    fn Mex_GuestLr() -> u32;
    fn PSMTXIdentity(a0: *mut c_void) = |a0: *mut c_void| { unsafe { melee_runtime::tu::mtx_pc::raw::PSMTXIdentity(a0 as _); } };
    fn PSVECAdd(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) = |a0: *mut Vec, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::mtx_pc::raw::PSVECAdd(a0 as _, a1 as _, a2 as _); } };
    fn Player_GetEntity(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::pl::Player_GetEntity(a0 as _) as _ };
    fn Player_GetEntityAtIndex(a0: i32, a1: i32) -> *mut HSD_GObj = |a0: i32, a1: i32| unsafe { crate::pl::Player_GetEntityAtIndex(a0 as _, a1 as _) as _ };
    fn Player_GetPlayerCharacter(a0: i32) -> u32 = |a0: i32| unsafe { crate::pl::Player_GetPlayerCharacter(a0 as _) as _ };
    fn Player_LoadPlayerCoords(a0: i32, a1: *mut Vec) = |a0: i32, a1: *mut Vec| { unsafe { let _ = crate::pl::Player_LoadPlayerCoords(a0 as _, a1 as _); } };
    fn ScriptGame_RefsGroundBorn(a0: i32, a1: i32);
    fn ScriptGame_StagePrepare(a0: *mut MapCollData) -> *mut MapCollData;
    fn ScriptGame_StageReady();
    fn ScriptGame_StageSlotCollision(a0: i32) -> i32;
    fn ScriptGame_StageSlotGround(a0: i32) -> *mut HSD_GObj;
    fn ScriptGame_StageSlotRetailCaptureBegin();
    fn ScriptGame_StageSlotRetailCaptureEnd();
    fn ScriptGame_StageSlotRetailCaptureResume();
    fn ScriptGame_StageSlotRetailCollision();
    fn ScriptGame_StageSlotRetailLoading() -> i32;
    fn StageSkin_ApplyForLoad(a0: i32, a1: *const i8, a2: i32);
    fn Stage_80224FDC(a0: *mut Vec) -> i32 = |a0: *mut Vec| unsafe { crate::tu::stage::Stage_80224FDC(a0 as _) as _ };
    fn Stage_GetBlastZoneTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneTopOffset() as _ };
    fn Stage_UnkSetVec3TCam_Offset(a0: *mut Vec) = |a0: *mut Vec| { unsafe { crate::tu::stage::Stage_UnkSetVec3TCam_Offset(a0 as _); } };
    fn TTMod_ForCharacter(a0: i32) -> i32;
    fn TTMod_Platform(a0: i32, a1: i32, a2: *mut f32, a3: *mut f32, a4: *mut f32, a5: *mut f32, a6: *mut f32);
    fn TTMod_PlatformCount(a0: i32) -> i32;
    fn TTMod_Target(a0: i32, a1: i32, a2: *mut f32, a3: *mut f32, a4: *mut f32);
    fn TTMod_TargetCount(a0: i32) -> i32;
    fn Toy_803048C0(a0: i32) -> i32;
    fn Toy_80304A58(a0: i32);
    fn Toy_80305918(a0: i8, a1: i32, a2: i32);
    fn Toy_8031234C(a0: i32);
    fn Toy_803124BC();
    fn asinf(a0: f32) -> f32 = |a0: f32| unsafe { melee_runtime::lbtrigf::asinf(a0 as _) as _ };
    fn ftCo_8009F4A4();
    fn ftCo_800C06C0() = || { unsafe { let _ = crate::ftdevice::ftCo_800C06C0(); } };
    fn ftLib_GetPos(a0: *mut HSD_GObj, a1: *mut Vec) = |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_GetPos(a0 as _, a1 as _); } };
    fn ftLib_IsSleeping(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_IsSleeping(a0 as _) as _ };
    fn gm_80164600() -> i32;
    fn gm_80164ABC() -> i32;
    fn gm_8016B238() -> i32 = || unsafe { crate::gm::gm_8016B238() as _ };
    fn gm_801B6320() -> *mut c_void;
    fn gm_GetFrameCount() -> u32 = || unsafe { crate::gm::gm_GetFrameCount() as _ };
    fn gm_IsCKindUnlocked(a0: u8) -> i32;
    fn gm_IsCurrently1PMode() -> i32;
    fn grCorneria_801E2A6C() = || { unsafe { crate::tu::grcorneria::grCorneria_801E2A6C(); } };
    fn grDatFiles_801C6038(a0: *mut c_void, a1: i32, a2: i32) = |a0: *mut c_void, a1: i32, a2: i32| { unsafe { crate::tu::grdatfiles::grDatFiles_801C6038(a0 as _, a1 as _, a2 as _); } };
    fn grDatFiles_801C6288() = || { unsafe { crate::tu::grdatfiles::grDatFiles_801C6288(); } };
    fn grDatFiles_801C6330(a0: i32) -> *mut UnkArchiveStruct = |a0: i32| unsafe { crate::tu::grdatfiles::grDatFiles_801C6330(a0 as _) as _ };
    fn grDatFiles_GetArchive() -> *mut UnkArchiveStruct = || unsafe { crate::tu::grdatfiles::grDatFiles_GetArchive() as _ };
    fn grDatFiles_StageSlotInstall(a0: *mut HSD_Archive) = |a0: *mut HSD_Archive| { unsafe { crate::tu::grdatfiles::grDatFiles_StageSlotInstall(a0 as _); } };
    fn grDisplay_801C5F60(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { crate::tu::grdisplay::grDisplay_801C5F60(a0 as _, a1 as _); } };
    fn grIzumi_801CD2D4() = || { unsafe { crate::tu::grizumi::grIzumi_801CD2D4(); } };
    fn grKongo_801D8270(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grkongo::grKongo_801D8270(a0 as _); } };
    fn grKongo_801D828C(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grkongo::grKongo_801D828C(a0 as _); } };
    fn grMaterial_801C95C4(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grmaterial::grMaterial_801C95C4(a0 as _); } };
    fn grMaterial_801C9698(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grmaterial::grMaterial_801C9698(a0 as _); } };
    fn grOldKongo_802105AC(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::groldkongo::grOldKongo_802105AC(a0 as _); } };
    fn grOldKongo_802105C8(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::groldkongo::grOldKongo_802105C8(a0 as _); } };
    fn grStadium_801D39A0(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grpstadium::grStadium_801D39A0(a0 as _); } };
    fn grStadium_801D511C() = || { unsafe { crate::tu::grpstadium::grStadium_801D511C(); } };
    fn grZebes_801DA3F4(a0: *mut Vec) = |a0: *mut Vec| { unsafe { crate::tu::grzebes::grZebes_801DA3F4(a0 as _); } };
    fn it_8026B40C(a0: *mut Article, a1: i32) = |a0: *mut Article, a1: i32| { unsafe { let _ = crate::itacc::it_8026B40C(a0 as _, a1 as _); } };
    fn it_8026BE84(a0: *mut BobOmbRain) -> *mut HSD_GObj;
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj;
    fn it_802F2094(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32, a3: i32) -> *mut HSD_GObj;
    fn it_ResetStageArticleTable() = || { unsafe { let _ = crate::itacc::it_ResetStageArticleTable(); } };
    fn lbAudioAx_8002305C(a0: i32, a1: i32) -> i32 = |a0: i32, a1: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002305C(a0 as _, a1 as _) as _ };
    fn lbAudioAx_800236B8(a0: i32) -> i32 = |a0: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236B8(a0 as _) as _ };
    fn lbAudioAx_80023710(a0: i32) -> i32 = |a0: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023710(a0 as _) as _ };
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 = |a0: i32, a1: i32, a2: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0 as _, a1 as _, a2 as _) as _ };
    fn lbAudioAx_80023870(a0: i32, a1: i32, a2: i32, a3: i32) -> i32 = |a0: i32, a1: i32, a2: i32, a3: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023870(a0 as _, a1 as _, a2 as _, a3 as _) as _ };
    fn lbAudioAx_80024B1C(a0: i32, a1: i32) = |a0: i32, a1: i32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024B1C(a0 as _, a1 as _); } };
    fn lbAudioAx_80024B58(a0: i32, a1: i32) = |a0: i32, a1: i32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024B58(a0 as _, a1 as _); } };
    fn lbDvd_800178E8(a0: i32, a1: *const i8, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: u8, a8: i32) = |a0: i32, a1: *const i8, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: u8, a8: i32| { unsafe { melee_runtime::tu::lbdvd::lbDvd_800178E8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _, a8 as _); } };
    fn lbShadow_8000E9F0(a0: *mut Vec, a1: *mut HSD_Spline, a2: f32) = |a0: *mut Vec, a1: *mut HSD_Spline, a2: f32| { unsafe { melee_runtime::tu::lbshadow::lbShadow_8000E9F0(&mut *a0, &*a1, a2); } };
    fn lbVector_Add(a0: *mut Vec, a1: *mut Vec) -> *mut Vec = |a0: *mut Vec, a1: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_Add(a0 as _, a1 as _) as _ };
    fn lbVector_CrossprodNormalized(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec = |a0: *mut Vec, a1: *mut Vec, a2: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_CrossprodNormalized(a0 as _, a1 as _, a2 as _) as _ };
    fn lbVector_EulerAnglesFromONB(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec) -> *mut Vec = |a0: *mut Vec, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_EulerAnglesFromONB(a0 as _, a1 as _, a2 as _, a3 as _) as _ };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 = |a0: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_8000F9F8(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::lb_00F9::lb_8000F9F8(a0 as _); } };
    fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj = |a0: *mut *mut LightList| unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(a0 as _) as _ };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj = |a0: *mut HSD_CameraDescPerspective| unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0 as _) as _ };
    fn mpGetGroundCollJoint() -> *mut CollJoint = || unsafe { crate::tu::mplib::mpGetGroundCollJoint() as _ };
    fn mpGetGroundCollLine() -> *mut CollLine = || unsafe { crate::tu::mplib::mpGetGroundCollLine() as _ };
    fn mpJointListAdd(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpJointListAdd(a0 as _); } };
    fn mpLibLoad(a0: *mut MapCollData) = |a0: *mut MapCollData| { unsafe { crate::tu::mplib::mpLibLoad(a0 as _); } };
    fn mpLib_800552B0(a0: i32, a1: *mut HSD_JObj, a2: i32) = |a0: i32, a1: *mut HSD_JObj, a2: i32| { unsafe { crate::tu::mplib::mpLib_800552B0(a0 as _, a1 as _, a2 as _); } };
    fn mpLib_80055E9C(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80055E9C(a0 as _); } };
    fn mpLib_80057424(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057424(a0 as _); } };
    fn mpLib_80057BC0(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057BC0(a0 as _); } };
    fn mpLib_80058820() = || { unsafe { crate::tu::mplib::mpLib_80058820(); } };
    fn psInitDataBankLoad(a0: i32, a1: *const i32, a2: *const i32, a3: *const u32, a4: *const i32) = |a0: i32, a1: *const i32, a2: *const i32, a3: *const u32, a4: *const i32| { unsafe { melee_runtime::tu::particle::psInitDataBankLoad(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } };
    fn splGetSplinePoint(a0: *mut Vec, a1: *mut HSD_Spline, a2: f32) = |a0: *mut Vec, a1: *mut HSD_Spline, a2: f32| { unsafe { melee_runtime::tu::spline_world::splGetSplinePoint(a0 as _, a1 as _, a2 as _); } };
    fn tyDisplay_8031C2CC();
    fn tyDisplay_8031C2EC() -> i32;
    fn tyDisplay_8031C454(a0: i32) -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_AObjSetFlags(a0: *mut HSD_AObj, a1: u32) { super::HSD_AObjSetFlags(a0, a1) }
        pub unsafe extern "C" fn HSD_AObjSetRate(a0: *mut HSD_AObj, a1: f32) { super::HSD_AObjSetRate(a0, a1) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn grDisplay_801C5F60(a0: *mut HSD_GObj, a1: i32) { super::grDisplay_801C5F60(a0, a1) }
        pub unsafe extern "C" fn mpJointListAdd(a0: i32) { super::mpJointListAdd(a0) }
        pub unsafe extern "C" fn mpLib_80057424(a0: i32) { super::mpLib_80057424(a0) }
        pub unsafe extern "C" fn mpLib_80057BC0(a0: i32) { super::mpLib_80057BC0(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_FogKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_LightKind;
    pub use crate::tu::grbigblueroute::grBb_Route_StageData;
    pub use crate::tu::grbigblue::grBb_StageData;
    pub use crate::tu::grcorneria::grCn_StageData;
    pub use crate::tu::grcastle::grCs_StageData;
    pub use crate::tu::grfigure1::grEF1_StageData;
    pub use crate::tu::grfigure2::grEF2_StageData;
    pub use crate::tu::grfigure3::grEF3_StageData;
    pub use crate::tu::grfigureget::grFigureGet_StageData;
    pub use crate::tu::grfourside::grFs_StageData;
    pub use crate::tu::grflatzone::grFz_StageData;
    pub use crate::tu::grgreatbay::grGb_StageData;
    pub use crate::tu::grgarden::grGd_StageData;
    pub use crate::tu::grgreens::grGr_StageData;
    pub use crate::tu::grheal::grHeal_StageData;
    pub use crate::tu::grhomerun::grHr_StageData;
    pub use crate::tu::grinishie1::grI1_StageData;
    pub use crate::tu::grinishie2::grI2_StageData;
    pub use crate::tu::gricemt::grIm_StageData;
    pub use crate::tu::grizumi::grIz_StageData;
    pub use crate::tu::grkongo::grKg_StageData;
    pub use crate::tu::grkraid::grKr_StageData;
    pub use crate::tu::grmutecity::grMc_StageData;
    pub use crate::tu::grbattle::grNBa_StageData;
    pub use crate::tu::grkinokoroute::grNKr_StageData;
    pub use crate::tu::grlast::grNLa_StageData;
    pub use crate::tu::groldkongo::grOk_StageData;
    pub use crate::tu::groldpupupu::grOp_StageData;
    pub use crate::tu::gronett::grOt_StageData;
    pub use crate::tu::groldyoshi::grOy_StageData;
    pub use crate::tu::grpstadium::grPs_StageData;
    pub use crate::tu::grpura::grPu_StageData;
    pub use crate::tu::grpushon::grPushOn_StageData;
    pub use crate::tu::grrcruise::grRc_StageData;
    pub use crate::tu::grshrineroute::grSh_Route_StageData;
    pub use crate::tu::grshrine::grSh_StageData;
    pub use crate::tu::grstory::grSt_StageData;
    pub use crate::tu::grtclink::grTCLink_StageData;
    pub use crate::tu::grtcaptain::grTCa_StageData;
    pub use crate::tu::grtdonkey::grTDk_StageData;
    pub static mut grTDr_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTFc_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTFe_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTFx_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTGn_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTGw_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTIc_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTKb_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTKp_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTLg_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTLk_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTMewtwo_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTMr_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTMs_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTNs_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTPc_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTPe_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTPk_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTPr_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTSk_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTSs_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTYs_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTZd_StageData: StageData = unsafe { core::mem::zeroed() };
    pub static mut grTe_StageData: StageData = unsafe { core::mem::zeroed() };
    pub use crate::tu::grvenom::grVe_StageData;
    pub use crate::tu::gryorster::grYt_StageData;
    pub use crate::tu::grzebesroute::grZe_Route_StageData;
    pub use crate::tu::grzebes::grZe_StageData;
    pub use melee_runtime::shared::it_804D6D38;
    pub use crate::tu::mpcoll::mpColl_804D64AC;
}
// END GENERATED EXT

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, _j: &mut HSD_JObj) { unreachable!("HSD_JObjSetMtxDirty (out of line) is not called by this TU") }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
}

/// `GXCommandList.h` GX_DRAW_TRIANGLES; `mobj.h` RENDER_DIFFUSE_MAT = `1 << RENDER_DIFFUSE_SHIFT`.
const GX_DRAW_TRIANGLES: u8 = 0x90;
const RENDER_DIFFUSE_MAT: u32 = 1 << 0;
const GR_MEX_FIRST_NEW: usize = 71;
const GR_MEX_ROWS: usize = 89;
const BGM_Undefined: i32 = -1;
const RANDI_MAX: i32 = 100;
const Gr_CObj_Max: usize = 4;
const buffer_size: usize = 64;
const STAGE_DATAS_LEN: usize = 160;

/// gr/inlines.h-style `HSD_JObjGet*` on a possibly-null pointer.
#[inline(always)]
unsafe fn jchild(j: *mut HSD_JObj) -> *mut HSD_JObj { HSD_JObjGetChild(j.as_ref()) }
#[inline(always)]
unsafe fn jnext(j: *mut HSD_JObj) -> *mut HSD_JObj { HSD_JObjGetNext(j.as_ref()) }
#[inline(always)]
unsafe fn jparent(j: *mut HSD_JObj) -> *mut HSD_JObj { HSD_JObjGetParent(j.as_ref()) }

/// `random.h` `randi`.
#[inline(always)]
unsafe fn randi(max_val: i32) -> i32 {
    if max_val != 0 { ext::HSD_Randi(max_val) } else { 0 }
}

/// `StageInfo::xA0` as the `s32*` the C passes around (35 words: the field plus its padding).
#[inline(always)]
unsafe fn stage_info_xA0() -> *mut i32 {
    (&raw mut stage_info as *mut u8).add(offset_of!(StageInfo, xA0)) as *mut i32
}

/// `StageInfo stage_info` (49E6C8)
pub static mut stage_info: StageInfo = unsafe { core::mem::zeroed() };

/// `Ground_StageData` (3DFEA8)
pub static mut Ground_StageData: StageData = StageData {
    grkind: 0,
    callbacks: null_mut(),
    data1: null_mut(),
    on_init: Some(Ground_OnStart),
    on_demo_init: Some(Ground_801BFFAC),
    on_load: Some(Ground_OnStart),
    on_start: Some(Ground_OnStart),
    callback4: None,
    on_touch_line: None,
    on_check_shadow_render: None,
    flags2: 0,
    joints: null_mut(),
    joint_count: 0,
};

/// `static StageData* stage_datas[]` (160 rows: rows 111..159 are the m-ex rows, TARGET_PC)
pub static mut stage_datas: [*mut StageData; STAGE_DATAS_LEN] = unsafe {
    let mut a = [&raw mut ext::grTe_StageData; STAGE_DATAS_LEN];
    a[0] = &raw mut Ground_StageData;
    a[2] = &raw mut ext::grCs_StageData;
    a[3] = &raw mut ext::grRc_StageData;
    a[4] = &raw mut ext::grKg_StageData;
    a[5] = &raw mut ext::grGd_StageData;
    a[6] = &raw mut ext::grGb_StageData;
    a[7] = &raw mut ext::grSh_StageData;
    a[8] = &raw mut ext::grZe_StageData;
    a[9] = &raw mut ext::grKr_StageData;
    a[10] = &raw mut ext::grSt_StageData;
    a[11] = &raw mut ext::grYt_StageData;
    a[12] = &raw mut ext::grIz_StageData;
    a[13] = &raw mut ext::grGr_StageData;
    a[14] = &raw mut ext::grCn_StageData;
    a[15] = &raw mut ext::grVe_StageData;
    a[16] = &raw mut ext::grPs_StageData;
    a[17] = &raw mut ext::grPu_StageData;
    a[18] = &raw mut ext::grMc_StageData;
    a[19] = &raw mut ext::grBb_StageData;
    a[20] = &raw mut ext::grOt_StageData;
    a[21] = &raw mut ext::grFs_StageData;
    a[22] = &raw mut ext::grIm_StageData;
    a[23] = null_mut();
    a[24] = &raw mut ext::grI1_StageData;
    a[25] = &raw mut ext::grI2_StageData;
    a[26] = null_mut();
    a[27] = &raw mut ext::grFz_StageData;
    a[28] = &raw mut ext::grOp_StageData;
    a[29] = &raw mut ext::grOy_StageData;
    a[30] = &raw mut ext::grOk_StageData;
    a[31] = &raw mut ext::grNKr_StageData;
    a[32] = &raw mut ext::grSh_Route_StageData;
    a[33] = &raw mut ext::grZe_Route_StageData;
    a[34] = &raw mut ext::grBb_Route_StageData;
    a[36] = &raw mut ext::grNBa_StageData;
    a[37] = &raw mut ext::grNLa_StageData;
    a[38] = &raw mut ext::grFigureGet_StageData;
    a[39] = &raw mut ext::grPushOn_StageData;
    a[40] = &raw mut ext::grTMr_StageData;
    a[41] = &raw mut ext::grTCa_StageData;
    a[42] = &raw mut ext::grTCLink_StageData;
    a[43] = &raw mut ext::grTDk_StageData;
    a[44] = &raw mut ext::grTDr_StageData;
    a[45] = &raw mut ext::grTFc_StageData;
    a[46] = &raw mut ext::grTFx_StageData;
    a[47] = &raw mut ext::grTIc_StageData;
    a[48] = &raw mut ext::grTKb_StageData;
    a[49] = &raw mut ext::grTKp_StageData;
    a[50] = &raw mut ext::grTLk_StageData;
    a[51] = &raw mut ext::grTLg_StageData;
    a[52] = &raw mut ext::grTMs_StageData;
    a[53] = &raw mut ext::grTMewtwo_StageData;
    a[54] = &raw mut ext::grTNs_StageData;
    a[55] = &raw mut ext::grTPe_StageData;
    a[56] = &raw mut ext::grTPc_StageData;
    a[57] = &raw mut ext::grTPk_StageData;
    a[58] = &raw mut ext::grTPr_StageData;
    a[59] = &raw mut ext::grTSs_StageData;
    a[60] = &raw mut ext::grTSk_StageData;
    a[61] = &raw mut ext::grTYs_StageData;
    a[62] = &raw mut ext::grTZd_StageData;
    a[63] = &raw mut ext::grTGw_StageData;
    a[64] = &raw mut ext::grTFe_StageData;
    a[65] = &raw mut ext::grTGn_StageData;
    a[66] = &raw mut ext::grHeal_StageData;
    a[67] = &raw mut ext::grHr_StageData;
    a[68] = &raw mut ext::grEF1_StageData;
    a[69] = &raw mut ext::grEF2_StageData;
    a[70] = &raw mut ext::grEF3_StageData;
    a
};

pub static mut Ground_MexStageDatas: [StageData; GR_MEX_ROWS] = unsafe { core::mem::zeroed() };
/// function-local `static int moaned` of `Ground_MexStageCallbacks`
pub static mut Ground_MexStageCallbacks_moaned: i32 = -1;
/// function-local `static bool done` of `Ground_MexInitStages`
pub static mut Ground_MexInitStages_done: bool = false;

/// `static u8* Ground_804D6950` (the per-map "joints off" flags)
pub static mut Ground_804D6950: *mut u8 = null_mut();
/// `static s16 Ground_804D6954`
pub static mut Ground_804D6954: i16 = 0;

// ---------------------------------------------------------------------------------------------
// m-ex stage rows (TARGET_PC)

/// A SYNTHESISED ROW MAY HAVE NO StageCallbacks[]: "no table" is "nothing matched".
pub unsafe extern "C" fn Ground_MexStageCallbacks(grkind: i32) -> *mut StageCallbacks {
    melee_core::ct!("Ground_MexStageCallbacks");
    if stage_datas[grkind as usize].is_null() {
        return null_mut();
    }
    let cb = (*stage_datas[grkind as usize]).callbacks;
    if cb.is_null() && Ground_MexStageCallbacks_moaned != grkind {
        Ground_MexStageCallbacks_moaned = grkind;
        // OSReport("grfunction: internal stage %d has no StageCallbacks[] ...", grkind)
    }
    cb
}

pub unsafe extern "C" fn Ground_MexOnInit() {
    melee_core::ct!("Ground_MexOnInit");
    ext::Mex_GrOnInit();
}
pub unsafe extern "C" fn Ground_MexOnDemoInit(arg: i32) {
    melee_core::ct!("Ground_MexOnDemoInit");
    ext::Mex_GrOnDemoInit(arg);
}
pub unsafe extern "C" fn Ground_MexOnLoad() {
    melee_core::ct!("Ground_MexOnLoad");
    ext::Mex_GrOnLoad();
}
pub unsafe extern "C" fn Ground_MexOnStart() {
    melee_core::ct!("Ground_MexOnStart");
    ext::Mex_GrOnStart();
}
pub unsafe extern "C" fn Ground_MexCallback4() -> i32 {
    melee_core::ct!("Ground_MexCallback4");
    (ext::Mex_GrCallback4() != 0) as i32
}
pub unsafe extern "C" fn Ground_MexOnTouchLine(index: i32) -> *mut DynamicsDesc {
    melee_core::ct!("Ground_MexOnTouchLine");
    ext::Mex_GrOnTouchLine(index) as *mut DynamicsDesc
}
pub unsafe extern "C" fn Ground_MexOnCheckShadowRender(pos: *mut Vec3, arg1: i32, jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("Ground_MexOnCheckShadowRender");
    (ext::Mex_GrOnCheckShadowRender(pos as *mut c_void, arg1, jobj as *mut c_void) != 0) as i32
}

pub unsafe extern "C" fn Ground_MexInitStages() {
    melee_core::ct!("Ground_MexInitStages");
    if Ground_MexInitStages_done {
        return;
    }
    Ground_MexInitStages_done = true;
    // The target-test stages (grt*.c) and the test stage (grtest.c) build their StageData at run time (`stage_data()`):
    // the static table holds zeroed stand-ins for them until this first use (C: `&grTMr_StageData` ... in stage_datas[]).
    {
        let te = &raw mut ext::grTe_StageData;
        let real_te = crate::tu::grtest::stage_data();
        for row in stage_datas.iter_mut() {
            if *row == te {
                *row = real_te;
            }
        }
        stage_datas[40] = crate::tu::grtmario::stage_data();
        stage_datas[44] = crate::tu::grtdrmario::stage_data();
        stage_datas[45] = crate::tu::grtfalco::stage_data();
        stage_datas[46] = crate::tu::grtfox::stage_data();
        stage_datas[47] = crate::tu::grticeclimber::stage_data();
        stage_datas[48] = crate::tu::grtkirby::stage_data();
        stage_datas[49] = crate::tu::grtkoopa::stage_data();
        stage_datas[50] = crate::tu::grtlink::stage_data();
        stage_datas[51] = crate::tu::grtluigi::stage_data();
        stage_datas[52] = crate::tu::grtmars::stage_data();
        stage_datas[53] = crate::tu::grtmewtwo::stage_data();
        stage_datas[54] = crate::tu::grtness::stage_data();
        stage_datas[55] = crate::tu::grtpeach::stage_data();
        stage_datas[56] = crate::tu::grtpichu::stage_data();
        stage_datas[57] = crate::tu::grtpikachu::stage_data();
        stage_datas[58] = crate::tu::grtpurin::stage_data();
        stage_datas[59] = crate::tu::grtsamus::stage_data();
        stage_datas[60] = crate::tu::grtseak::stage_data();
        stage_datas[61] = crate::tu::grtyoshi::stage_data();
        stage_datas[62] = crate::tu::grtzelda::stage_data();
        stage_datas[63] = crate::tu::grtgamewatch::stage_data();
        stage_datas[64] = crate::tu::grtemblem::stage_data();
        stage_datas[65] = crate::tu::grtganon::stage_data();
    }
    let mut n = ext::Mex_GrInternalCount();
    // (the truncation notice is an OSReport)
    if n > STAGE_DATAS_LEN as i32 {
        n = STAGE_DATAS_LEN as i32;
    }
    if n > (GR_MEX_FIRST_NEW + GR_MEX_ROWS) as i32 {
        n = (GR_MEX_FIRST_NEW + GR_MEX_ROWS) as i32;
    }
    let mut k = GR_MEX_FIRST_NEW as i32;
    while k < n {
        if ext::Mex_GrIsMex(k) == 0 {
            k += 1;
            continue;
        }
        let sd = &raw mut Ground_MexStageDatas[k as usize - GR_MEX_FIRST_NEW];
        (*sd).grkind = k as GrKind;
        (*sd).callbacks = ext::Mex_GrRowCallbacks(k) as *mut StageCallbacks;
        (*sd).data1 = ext::Mex_GrFile(k) as *mut i8;
        (*sd).on_init = Some(Ground_MexOnInit);
        (*sd).on_demo_init = Some(Ground_MexOnDemoInit);
        (*sd).on_load = Some(Ground_MexOnLoad);
        (*sd).on_start = Some(Ground_MexOnStart);
        (*sd).callback4 = Some(Ground_MexCallback4);
        (*sd).on_touch_line = Some(Ground_MexOnTouchLine);
        (*sd).on_check_shadow_render = Some(Ground_MexOnCheckShadowRender);
        (*sd).flags2 = ext::Mex_GrFlags2(k);
        (*sd).joints = null_mut();
        (*sd).joint_count = 0;
        stage_datas[k as usize] = sd;
        k += 1;
    }
}

pub unsafe extern "C" fn Ground_OnStart() { melee_core::ct!("Ground_OnStart");}

pub unsafe extern "C" fn Ground_801BFFAC(_arg0: i32) { melee_core::ct!("Ground_801BFFAC");}

pub unsafe extern "C" fn Ground_801BFFB0() {
    melee_core::ct!("Ground_801BFFB0");
    ext::grDatFiles_801C6288();
    stage_info.flags = 0;
    stage_info.x6D0 = -1;
    stage_info.x6D2 = 0;
    stage_info.x6D4 = 0;
    stage_info.x6D8 = 0;
    stage_info.x6DC = 0;
    stage_info.x724 = -10000.0;
    stage_info.x12C = null_mut();
    stage_info.unk8C.set_b3(0);
    stage_info.unk8C.set_b2(1);
    stage_info.on_touch_line = None;
    stage_info.on_check_shadow_render = None;
    stage_info.unk8C.set_b4(0);
    stage_info.unk8C.set_b5(0);
    stage_info.unk8C.set_b6(0);
    stage_info.x714 = -1;
    stage_info.x90 = None;
    stage_info.unk8C.set_b7(0);
    stage_info.x720 = -1;
    stage_info.x94 = None;
    for i in 0..stage_info.map_gobjs.len() {
        stage_info.map_gobjs[i] = null_mut();
    }
    for i in 0..stage_info.x280.len() {
        stage_info.x280[i] = null_mut();
    }
    for i in 0..stage_info.x694.len() {
        stage_info.x694[i] = null_mut();
    }
    stage_info.x6A4 = null_mut();
    stage_info.cam_info.cam_bounds.left = -170.0;
    stage_info.cam_info.cam_bounds.right = 170.0;
    stage_info.cam_info.cam_bounds.top = 120.0;
    stage_info.cam_info.cam_bounds.bottom = -60.0;
    stage_info.cam_info.cam_x_offset = 0.0;
    stage_info.cam_info.cam_y_offset = 0.0;
    stage_info.cam_info.cam_vertical_tilt = 30.0;
    stage_info.cam_info.cam_pan_degrees = -10.0;
    stage_info.cam_info.x20 = 0.2_f64 as f32;
    stage_info.cam_info.x24 = 0.2_f64 as f32;
    stage_info.cam_info.cam_zoom_rate = 82.0;
    stage_info.cam_info.cam_max_depth = 1000.0;
    stage_info.blast_zone.left = -99999.0;
    stage_info.blast_zone.right = 99999.0;
    stage_info.blast_zone.top = 99999.0;
    stage_info.blast_zone.bottom = -99999.0;
    stage_info.grkind = -1i32 as GrKind;
    stage_info.x6D2 = 0;
    stage_info.x6D4 = 0;
    stage_info.x98 = -1;
    stage_info.x9C = 0;
    stage_info.x72C = null_mut();
    stage_info.x6E0 = 0.0;
    stage_info.x708 = 0;
    stage_info.x740 = 0;
    if !Ground_804D6950.is_null() {
        for i in 0..buffer_size {
            *Ground_804D6950.add(i) = 0;
        }
    }
}

pub unsafe extern "C" fn zeroBuffer() {
    melee_core::ct!("zeroBuffer");
    for i in 0..buffer_size {
        *Ground_804D6950.add(i) = 0;
    }
}

pub unsafe extern "C" fn Ground_801C0378(_arg0: i32) {
    melee_core::ct!("Ground_801C0378");
    ext::ftCo_800C06C0();
    Ground_804D6950 = ext::HSD_MemAlloc(buffer_size as i32) as *mut u8;
    zeroBuffer();
}

pub unsafe extern "C" fn mem_free(ptr: *mut c_void) {
    melee_core::ct!("mem_free");
    ext::HSD_Free(ptr);
}

pub unsafe extern "C" fn Ground_801C0498() -> f32 {
    melee_core::ct!("Ground_801C0498");
    if !stage_info.param.is_null() {
        (*stage_info.param).y
    } else {
        1.0
    }
}

pub unsafe extern "C" fn alloc_user_data_ground() -> *mut Ground {
    melee_core::ct!("alloc_user_data_ground");
    let gp = ext::HSD_MemAlloc(size_of::<Ground>() as i32) as *mut Ground;
    // TARGET_PC: Slippi's Init Stage Data clears this block
    if !gp.is_null() {
        core::ptr::write_bytes(gp as *mut u8, 0, size_of::<Ground>());
    }
    // (couldn t get user data(Ground) is an OSReport)
    gp
}

pub unsafe extern "C" fn Ground_SetParamY(y: f32) {
    melee_core::ct!("Ground_SetParamY");
    if !stage_info.param.is_null() {
        (*stage_info.param).y = y;
    } else {
        panic!("ground.c:521 HSD_ASSERT(521, 0)");
    }
}

pub unsafe extern "C" fn Ground_801C0508() -> i32 {
    melee_core::ct!("Ground_801C0508");
    let temp_r3 = stage_info.param;
    if !temp_r3.is_null() { (*temp_r3).x4 as i32 } else { 128 }
}

pub unsafe extern "C" fn Ground_801C052C(arg0: *mut _GXColor) {
    melee_core::ct!("Ground_801C052C");
    (*stage_info.param).xB8 = *arg0;
}
pub unsafe extern "C" fn Ground_801C0544(arg0: *mut _GXColor) {
    melee_core::ct!("Ground_801C0544");
    (*stage_info.param).xC0 = *arg0;
}
pub unsafe extern "C" fn Ground_801C055C(arg0: *mut _GXColor) {
    melee_core::ct!("Ground_801C055C");
    (*stage_info.param).xD0 = *arg0;
}
pub unsafe extern "C" fn Ground_801C0574(arg0: *mut _GXColor) {
    melee_core::ct!("Ground_801C0574");
    (*stage_info.param).xD8 = *arg0;
}
pub unsafe extern "C" fn Ground_801C058C(arg0: *mut _GXColor) {
    melee_core::ct!("Ground_801C058C");
    (*stage_info.param).xC4 = *arg0;
}
pub unsafe extern "C" fn Ground_801C05A4(arg0: *mut _GXColor) {
    melee_core::ct!("Ground_801C05A4");
    (*stage_info.param).xCC = *arg0;
}
pub unsafe extern "C" fn Ground_801C05BC(arg0: *mut _GXColor) {
    melee_core::ct!("Ground_801C05BC");
    (*stage_info.param).xD4 = *arg0;
}
pub unsafe extern "C" fn Ground_801C05D4(arg0: *mut _GXColor) {
    melee_core::ct!("Ground_801C05D4");
    (*stage_info.param).xBC = *arg0;
}
pub unsafe extern "C" fn Ground_801C05EC(arg0: *mut _GXColor) {
    melee_core::ct!("Ground_801C05EC");
    (*stage_info.param).xC8 = *arg0;
}
pub unsafe extern "C" fn Ground_801C0604() -> *mut _GXColor {
    melee_core::ct!("Ground_801C0604");
    &raw mut (*stage_info.param).xB8
}
pub unsafe extern "C" fn Ground_801C0618() -> *mut _GXColor {
    melee_core::ct!("Ground_801C0618");
    &raw mut (*stage_info.param).xC0
}
pub unsafe extern "C" fn Ground_801C062C() -> *mut _GXColor {
    melee_core::ct!("Ground_801C062C");
    &raw mut (*stage_info.param).xD0
}
pub unsafe extern "C" fn Ground_801C0640() -> *mut _GXColor {
    melee_core::ct!("Ground_801C0640");
    &raw mut (*stage_info.param).xD8
}
pub unsafe extern "C" fn Ground_801C0654() -> *mut _GXColor {
    melee_core::ct!("Ground_801C0654");
    &raw mut (*stage_info.param).xC4
}
pub unsafe extern "C" fn Ground_801C0668() -> *mut _GXColor {
    melee_core::ct!("Ground_801C0668");
    &raw mut (*stage_info.param).xCC
}
pub unsafe extern "C" fn Ground_801C067C() -> *mut _GXColor {
    melee_core::ct!("Ground_801C067C");
    &raw mut (*stage_info.param).xD4
}
pub unsafe extern "C" fn Ground_801C0690() -> *mut _GXColor {
    melee_core::ct!("Ground_801C0690");
    &raw mut (*stage_info.param).xBC
}
pub unsafe extern "C" fn Ground_801C06A4() -> *mut _GXColor {
    melee_core::ct!("Ground_801C06A4");
    &raw mut (*stage_info.param).xC8
}

pub unsafe extern "C" fn Ground_801C06B8(arg0: GrKind) {
    melee_core::ct!("Ground_801C06B8");
    Ground_MexInitStages();
    if stage_datas[arg0 as usize].is_null() {
        return;
    }
    ext::StageSkin_ApplyForLoad(arg0 as i32, (*stage_datas[arg0 as usize]).data1, 1);
    if !(*stage_datas[arg0 as usize]).data1.is_null() {
        ext::lbDvd_800178E8(4, (*stage_datas[arg0 as usize]).data1, 4, 4, 0, 1, 7, 16, 0);
    }
    match arg0 {
        Gr_Kind_Izumi => ext::grIzumi_801CD2D4(),
        Gr_Kind_PStadium => ext::grStadium_801D511C(),
        _ => {}
    }
}

pub unsafe extern "C" fn Ground_801C0754(pair: *mut StageIdPair) {
    melee_core::ct!("Ground_801C0754");
    Ground_MexInitStages();
    ext::StageSkin_ApplyForLoad(
        (*pair).grkind as i32,
        if !stage_datas[(*pair).grkind as usize].is_null() { (*stage_datas[(*pair).grkind as usize]).data1 } else { null_mut() },
        2,
    );
    ext::Mex_GrSelect((*pair).grkind as i32);
    if stage_datas[(*pair).grkind as usize].is_null() {
        return;
    }
    ext::ScriptGame_StageSlotRetailCaptureBegin();
    Ground_801BFFB0();
    stage_info.grkind = (*pair).grkind;
    let stage = stage_datas[(*pair).grkind as usize];
    let arg3 = if (*pair).stkind == St_Kind_Heal { 0 } else { 1 };
    ext::grDatFiles_801C6038((*stage).data1 as *mut c_void, 0, arg3);
    if ext::Mex_GrIsMex((*pair).grkind as i32) != 0 {
        let mex_arc = ext::grDatFiles_GetArchive();
        ext::Mex_GrFunctionInit(
            if !mex_arc.is_null() { (*mex_arc).unk0 as *mut c_void } else { null_mut() },
            (*pair).grkind as i32,
        );
        let mut cb = ext::Mex_GrCallbacks() as *mut StageCallbacks;
        if cb.is_null() {
            cb = ext::Mex_GrRowCallbacks((*pair).grkind as i32) as *mut StageCallbacks;
        }
        (*stage_datas[(*pair).grkind as usize]).callbacks = cb;
    }
    Ground_801C28CC(stage_info_xA0(), (*pair).stkind);
    stage_info.on_touch_line = (*stage).on_touch_line;
    stage_info.on_check_shadow_render = (*stage).on_check_shadow_render;
    Ground_801C5878();
    ext::ScriptGame_StageSlotRetailCaptureEnd();
}

pub unsafe extern "C" fn Ground_TTMod_BuildCollData(orig: *mut MapCollData, level: i32, nplat: i32) -> *mut MapCollData {
    melee_core::ct!("Ground_TTMod_BuildCollData");
    let ovc = (*orig).vert_count;
    let olc = (*orig).line_count;
    let ojc = (*orig).joint_count;
    let mut scale = Ground_801C0498();

    if scale <= 0.001f32 {
        scale = 1.0;
    }
    let inv = 1.0f32 / scale;

    let merged = ext::HSD_MemAlloc(size_of::<MapCollData>() as i32) as *mut MapCollData;
    let verts = ext::HSD_MemAlloc((ovc + nplat * 2) * size_of::<Vec2>() as i32) as *mut Vec2;
    let lines = ext::HSD_MemAlloc((olc + nplat) * size_of::<MapLine>() as i32) as *mut MapLine;
    let joints = ext::HSD_MemAlloc((ojc + nplat) * size_of::<MapJoint>() as i32) as *mut MapJoint;
    if merged.is_null() || verts.is_null() || lines.is_null() || joints.is_null() {
        return null_mut();
    }

    for i in 0..ovc as isize {
        *verts.offset(i) = *(*orig).verts.offset(i);
    }
    for i in 0..olc as isize {
        *lines.offset(i) = *(*orig).lines.offset(i);
    }
    for i in 0..ojc as isize {
        *joints.offset(i) = *(*orig).joints.offset(i);
    }

    for i in 0..nplat {
        let mut cx = 0.0f32;
        let mut cy = 0.0f32;
        let mut cz = 0.0f32;
        let mut w = 0.0f32;
        let mut d = 0.0f32;
        let vi = ovc + 2 * i;
        let li = olc + i;
        let ji = ojc + i;

        ext::TTMod_Platform(level, i, &mut cx, &mut cy, &mut cz, &mut w, &mut d);
        let cx_c = cx * inv;
        let cy_c = cy * inv;
        let hw_c = (0.5f32 * w) * inv;

        (*verts.offset(vi as isize)).x = cx_c - hw_c;
        (*verts.offset(vi as isize)).y = cy_c;
        (*verts.offset(vi as isize + 1)).x = cx_c + hw_c;
        (*verts.offset(vi as isize + 1)).y = cy_c;

        let l = lines.offset(li as isize);
        (*l).v0_idx = (vi + 0) as u16;
        (*l).v1_idx = (vi + 1) as u16;
        (*l).prev_id0 = -1;
        (*l).next_id0 = -1;
        (*l).prev_id1 = -1;
        (*l).next_id1 = -1;
        (*l).hi_flags = (CollLine_Floor as i32 | LINE_FLAG_PLATFORM) as u16;
        (*l).lo_flags = 0;

        let j = joints.offset(ji as isize);
        (*j).ranges[MapLineGroup_Floor as usize].start = li as i16;
        (*j).ranges[MapLineGroup_Floor as usize].count = 1;
        (*j).ranges[MapLineGroup_Ceiling as usize].start = 0;
        (*j).ranges[MapLineGroup_Ceiling as usize].count = 0;
        (*j).ranges[MapLineGroup_RightWall as usize].start = 0;
        (*j).ranges[MapLineGroup_RightWall as usize].count = 0;
        (*j).ranges[MapLineGroup_LeftWall as usize].start = 0;
        (*j).ranges[MapLineGroup_LeftWall as usize].count = 0;
        (*j).ranges[MapLineGroup_Dynamic as usize].start = 0;
        (*j).ranges[MapLineGroup_Dynamic as usize].count = 0;
        (*j).left_bound = cx_c - hw_c - 30.0f32;
        (*j).bottom_bound = cy_c - 30.0f32;
        (*j).right_bound = cx_c + hw_c + 30.0f32;
        (*j).top_bound = cy_c + 30.0f32;
        (*j).vtx_start = vi as i16;
        (*j).vtx_count = 2;
    }

    (*merged).verts = verts;
    (*merged).vert_count = ovc + nplat * 2;
    (*merged).lines = lines;
    (*merged).line_count = olc + nplat;
    for g in [MapLineGroup_Floor, MapLineGroup_Ceiling, MapLineGroup_RightWall, MapLineGroup_LeftWall, MapLineGroup_Dynamic] {
        (*merged).ranges[g as usize].start = (*orig).ranges[g as usize].start;
        (*merged).ranges[g as usize].count = (*orig).ranges[g as usize].count;
    }
    (*merged).joints = joints;
    (*merged).joint_count = ojc + nplat;
    (*merged).x2C = (*orig).x2C;

    merged
}

pub unsafe extern "C" fn Ground_TTMod_InitLines(merged: *mut MapCollData, nplat: i32) {
    melee_core::ct!("Ground_TTMod_InitLines");
    let gline = ext::mpGetGroundCollLine();
    let olc = (*merged).line_count - nplat;
    for i in 0..nplat {
        let idx = (olc + i) as isize;
        (*gline.offset(idx)).x0 = (*merged).lines.offset(idx);
        (*gline.offset(idx)).flags = (*(*merged).lines.offset(idx)).hi_flags as u32 | LINE_FLAG_ENABLED as u32;
    }
}

pub unsafe extern "C" fn Ground_801C0800(pair: *mut StageIdPair) {
    melee_core::ct!("Ground_801C0800");
    if stage_datas[(*pair).grkind as usize].is_null() {
        return;
    }
    ext::ScriptGame_StageSlotRetailCaptureResume();
    let stage_data = stage_datas[(*pair).grkind as usize];
    let p = stage_info.param;
    Ground_801C38D0((*p).x8 as f32, (*p).x14 as f32, (*p).x1C, (*p).x18);
    Ground_801C38EC((*p).x10 as f32, (*p).xC as f32);
    Ground_801C3970((*p).x28);
    Ground_801C3900((*p).x2E as f32, (*p).x30 as f32, (*p).x34 as f32, (*p).x38 as f32, (*p).x3C, (*p).x40, (*p).x44, (*p).x48);
    Ground_801C392C((*p).x50, (*p).x54, (*p).x58, (*p).x5C, (*p).x60, (*p).x64);
    Ground_801C3960((*p).x20);
    Ground_801C3950((*p).x24);
    {
        // Previous scene heap may already be reused: clear descriptors BEFORE registering this
        // stage's current itemdata.
        ext::it_ResetStageArticleTable();
        if !stage_info.itemdata.is_null() {
            let mut i = 0isize;
            while !(*stage_info.itemdata.offset(i)).is_null() {
                let d = *stage_info.itemdata.offset(i);
                ext::it_8026B40C((*d).unk4, (*d).unk0);
                i += 1;
            }
        }

        if !stage_info.ald_yaku_all.is_null() {
            let mut i = 1isize;
            while !(*stage_info.ald_yaku_all.offset(i)).is_null() {
                let a = *ext::it_804D6D38.offset((It_PKind_Random - It_Kind_Kuriboh) as isize);
                // the C indexes past the declared extent of x0_itemStateDesc (the table is as long as the item has states)
                (*(*(*a).xC_itemStates).x0_itemStateDesc.as_mut_ptr().offset(i)).xC_script = *stage_info.ald_yaku_all.offset(i) as *mut c_void;
                i += 1;
            }
        }
    }
    if !stage_info.map_ptcl.is_null() && !stage_info.map_texg.is_null() {
        ext::psInitDataBankLoad(0x1E, stage_info.map_ptcl as *const i32, stage_info.map_texg as *const i32, core::ptr::null(), core::ptr::null());
    }
    {
        let ckind = *(ext::gm_801B6320() as *mut i8) as i32;
        let mut level = -1;
        let mut nplat = 0;
        let mut merged: *mut MapCollData = null_mut();
        let mut coll = stage_info.coll_data;
        if stage_info.grkind >= Gr_Kind_TMario && stage_info.grkind <= Gr_Kind_TGanon {
            level = ext::TTMod_ForCharacter(ckind);
            nplat = if level >= 0 { ext::TTMod_PlatformCount(level) } else { 0 };
        }
        if nplat > 0 && !stage_info.coll_data.is_null() {
            merged = Ground_TTMod_BuildCollData(stage_info.coll_data, level, nplat);
            if !merged.is_null() {
                coll = merged;
            }
        }
        if ext::ScriptGame_StageSlotRetailLoading() != 0 {
            ext::ScriptGame_StageSlotRetailCollision();
        } else {
            coll = ext::ScriptGame_StagePrepare(coll);
            ext::mpLibLoad(coll);
            if !merged.is_null() {
                Ground_TTMod_InitLines(coll, nplat);
            }
            ext::ScriptGame_StageReady();
        }
    }
    ext::mpLib_80058820();
    Ground_801C1E94();
    Ground_801C466C();
    (*stage_data).on_init.unwrap()();
    // grtrace report: the C only counts map gobjs and OSReports (no calls besides the trace switch)
    let _ = ext::Mex_GrTrace();
    ext::ScriptGame_StageSlotRetailCaptureEnd();
}

pub unsafe extern "C" fn Ground_801C0A70(pos: *mut Vec3) -> i32 {
    melee_core::ct!("Ground_801C0A70");
    if ext::HSD_Randi(2) != 0 {
        let enabled_stages: [GrKind; 27] = [
            Gr_Kind_Castle,
            Gr_Kind_RCruise,
            Gr_Kind_Kongo,
            Gr_Kind_Garden,
            Gr_Kind_GreatBay,
            Gr_Kind_Shrine,
            Gr_Kind_Zebes,
            Gr_Kind_Kraid,
            Gr_Kind_Story,
            Gr_Kind_Yorster,
            Gr_Kind_Izumi,
            Gr_Kind_Greens,
            Gr_Kind_Corneria,
            Gr_Kind_Venom,
            Gr_Kind_PStadium,
            Gr_Kind_Pura,
            Gr_Kind_MuteCity,
            Gr_Kind_BigBlue,
            Gr_Kind_Onett,
            Gr_Kind_Fourside,
            Gr_Kind_Inishie1,
            Gr_Kind_Inishie2,
            Gr_Kind_OldPupupu,
            Gr_Kind_OldYoshi,
            Gr_Kind_OldKongo,
            Gr_Kind_Battle,
            Gr_Kind_Last,
        ];
        let nstages = enabled_stages.len();
        let kind = stage_info.grkind;
        let mut i = 0;
        while i < nstages {
            if kind == enabled_stages[i] {
                break;
            }
            i += 1;
        }
        if i != nstages {
            let player_slot = ext::HSD_Randi(4);
            if !ext::Player_GetEntity(player_slot).is_null() {
                ext::Player_LoadPlayerCoords(player_slot, pos);
                (*pos).y = -5.0f32 + ext::Stage_GetBlastZoneTopOffset();
                let xoff = ext::HSD_Randi(0x64) - 0x32;
                (*pos).x += xoff as f32;
                return 1;
            }
        }
    }
    (ext::Stage_80224FDC(pos) != 0) as i32
}

pub unsafe extern "C" fn Ground_801C0C2C(arg0: *mut HSD_GObj) {
    melee_core::ct!("Ground_801C0C2C");
    // mod host site (hooks-v1 0x801C0C2C, melee_core::modhook): one None check when no mod hooks it
    melee_core::mod_hook!(0x801C0C2C, [melee_core::modhook::Arg::Gobj(arg0 as _)], Ground_801C0C2C__orig(arg0));
    #[allow(unused_unsafe)] unsafe { Ground_801C0C2C__orig(arg0) }
}

unsafe extern "C" fn Ground_801C0C2C__orig(arg0: *mut HSD_GObj) {
    let mut sp50 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp44 = sp50;
    let mut sp38 = sp50;
    let mut sp2C = sp50;

    if stage_info.unk8C.b6() != 0 || stage_info.unk8C.b7() != 0 {
        let gobj = Ground_GetP1Fighter();
        if !gobj.is_null() && ext::ftLib_IsSleeping(gobj) == 0 {
            ext::ftLib_GetPos(gobj, &mut sp50);
            if stage_info.unk8C.b6() != 0 {
                let mut result = false;
                let x_max = stage_info.x70C;
                let y_max = stage_info.x710;
                let mut i = 0x99;
                while i < 0xB3 {
                    if Ground_801C2D24(i, &mut sp44) != 0 {
                        let mut pass_position_bounds = false;
                        let mut pass_y_min = false;
                        let mut pass_x_bounds = false;
                        let xpos = sp50.x - sp44.x;
                        let ypos = sp50.y - sp44.y;
                        if xpos > -x_max && xpos < x_max {
                            pass_x_bounds = true;
                        }
                        if pass_x_bounds && ypos > -y_max {
                            pass_y_min = true;
                        }
                        if pass_y_min && ypos < y_max {
                            pass_position_bounds = true;
                        }
                        if pass_position_bounds && (stage_info.x90.is_none() || stage_info.x90.unwrap()(&mut sp50, i) != 0) {
                            result = true;
                            break;
                        }
                    }
                    i += 1;
                }
                if result {
                    stage_info.x714 = i;
                } else {
                    stage_info.x714 = -1;
                }
                stage_info.unk8C.set_b6(0);
            }
            if stage_info.unk8C.b7() != 0 {
                let mut result = false;
                let x_max = stage_info.x718;
                let y_max = stage_info.x71C;
                let mut i = 0xBD;
                while i < 0xC7 {
                    if Ground_801C2D24(i, &mut sp38) != 0 {
                        let mut pass_position_bounds = false;
                        let mut pass_y_min = false;
                        let mut pass_x_bounds = false;
                        let xpos = sp50.x - sp38.x;
                        let ypos = sp50.y - sp38.y;
                        if xpos > -x_max && xpos < x_max {
                            pass_x_bounds = true;
                        }
                        if pass_x_bounds && ypos > -y_max {
                            pass_y_min = true;
                        }
                        if pass_y_min && ypos < y_max {
                            pass_position_bounds = true;
                        }
                        if pass_position_bounds && (stage_info.x94.is_none() || stage_info.x94.unwrap()(&mut sp50, i) != 0) {
                            result = true;
                            break;
                        }
                    }
                    i += 1;
                }
                if result {
                    stage_info.x720 = i;
                } else {
                    stage_info.x720 = -1;
                }
                stage_info.unk8C.set_b7(0);
            }
        }
    }
    if ext::gm_8016B238() != 0 {
        let current_frame = ext::gm_GetFrameCount() as i32;
        // `current_frame - stage_info.x9C` is int - u32: unsigned
        if current_frame > 0x4B0 && (current_frame as u32).wrapping_sub(stage_info.x9C) > 0x1E {
            stage_info.x9C = current_frame as u32;
            if Ground_801C0A70(&mut sp2C) != 0 {
                let mut sp_c: BobOmbRain = core::mem::zeroed();
                sp_c.x14 = 6;
                sp_c.x8_vec = sp2C;
                ext::it_8026BE84(&mut sp_c);
            }
        }
    }
}

pub unsafe extern "C" fn Ground_OnLoad(pair: *mut StageIdPair) {
    melee_core::ct!("Ground_OnLoad");
    if stage_datas[(*pair).grkind as usize].is_null() {
        return;
    }
    ext::ScriptGame_StageSlotRetailCaptureResume();
    (*stage_datas[(*pair).grkind as usize]).on_load.unwrap()();
    ext::ScriptGame_StageSlotRetailCaptureEnd();
}

/// The `{ void* unk0; s32 unk4; void (*unk8)(s32); }` node `Ground_801C0FB8` walks.
#[repr(C)]
struct StartNode {
    unk0: *mut c_void,
    unk4: i32,
    unk8: Option<unsafe extern "C" fn(i32)>,
}

/// The `{ void* unk0; HSD_GObj* unk4; HSD_GObjEvent unk8; }` node `Ground_801C10B8` allocates.
#[repr(C)]
struct QueueNode {
    unk0: *mut c_void,
    unk4: *mut HSD_GObj,
    unk8: HSD_GObjEvent,
}

pub unsafe extern "C" fn Ground_801C0FB8(pair: *mut StageIdPair) {
    melee_core::ct!("Ground_801C0FB8");
    // mod host site (hooks-v1 0x801C0FB8, melee_core::modhook): one None check when no mod hooks it
    melee_core::mod_hook!(0x801C0FB8, [melee_core::modhook::Arg::Ptr(pair as _)], Ground_801C0FB8__orig(pair));
    #[allow(unused_unsafe)] unsafe { Ground_801C0FB8__orig(pair) }
}

unsafe extern "C" fn Ground_801C0FB8__orig(pair: *mut StageIdPair) {
    if stage_datas[(*pair).grkind as usize].is_null() {
        return;
    }
    ext::ScriptGame_StageSlotRetailCaptureResume();
    (*stage_datas[(*pair).grkind as usize]).on_start.unwrap()();
    let mut cur = stage_info.x6A4 as *mut StartNode;
    while !cur.is_null() {
        let next = (*cur).unk0;
        ((*cur).unk8.unwrap())((*cur).unk4);
        ext::HSD_Free(cur as *mut c_void);
        cur = next as *mut StartNode;
    }
    stage_info.x6A4 = null_mut();
    ext::HSD_GObj_SetupProc(ext::GObj_Create(HSD_GOBJ_CLASS_STAGE as u16, 5, 0), Some(Ground_801C0C2C), 10);
    ext::ScriptGame_StageSlotRetailCaptureEnd();
}

pub unsafe extern "C" fn Ground_DemoInit(pair: *mut StageIdPair, arg1: i32) {
    melee_core::ct!("Ground_DemoInit");
    if stage_datas[(*pair).grkind as usize].is_null() {
        return;
    }
    ((*stage_datas[(*pair).grkind as usize]).on_demo_init.unwrap())(arg1);
}

pub unsafe extern "C" fn Ground_801C10B8(arg0: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    melee_core::ct!("Ground_801C10B8");
    let temp_r3 = ext::HSD_MemAlloc(size_of::<QueueNode>() as i32) as *mut QueueNode;
    if !temp_r3.is_null() {
        (*temp_r3).unk0 = stage_info.x6A4;
        (*temp_r3).unk4 = arg0;
        (*temp_r3).unk8 = arg1;
        stage_info.x6A4 = temp_r3 as *mut c_void;
    } else {
        // OSReport("%s:%d: assert\n"); OSPanic(__FILE__, 1120, "")
        panic!("ground.c:1120 OSPanic");
    }
}

pub unsafe extern "C" fn Ground_801C1154() { melee_core::ct!("Ground_801C1154");}

pub unsafe extern "C" fn Ground_801C1158() {
    melee_core::ct!("Ground_801C1158");
    match stage_info.grkind {
        Gr_Kind_PStadium => ext::grStadium_801D39A0(Ground_GetMapGObj(1)),
        Gr_Kind_Corneria | Gr_Kind_Venom => ext::grCorneria_801E2A6C(),
        _ => {}
    }
}

pub unsafe extern "C" fn Ground_801C11AC(gobj: *mut HSD_GObj) {
    melee_core::ct!("Ground_801C11AC");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let tmp = 1.0f32;
    let mut scale = Vec3 { x: tmp, y: tmp, z: tmp };
    assert!(!jobj.is_null(), "jobj.h:760 jobj");
    HSD_JObjSetScale(&mut *jobj, &scale, &mut Dirty);
}

pub unsafe extern "C" fn Ground_801C126C(node: *mut HSD_Joint, depth: *mut i32) -> *mut HSD_Joint {
    melee_core::ct!("Ground_801C126C");
    *depth -= 1;
    if *depth < 0 {
        return node;
    }
    if !(*node).child.is_null() {
        let result = Ground_801C126C((*node).child, depth);
        if !result.is_null() {
            return result;
        }
    }
    if !(*node).next.is_null() {
        let result = Ground_801C126C((*node).next, depth);
        if !result.is_null() {
            return result;
        }
    }
    null_mut()
}

pub unsafe extern "C" fn Ground_801C13D0(arg0: i32, depth: i32) -> *mut HSD_JObj {
    melee_core::ct!("Ground_801C13D0");
    let mut result: *mut HSD_JObj = null_mut();
    let archive = ext::grDatFiles_801C6330(arg0);
    if !archive.is_null() && arg0 < (*(*archive).unk4).unkC {
        let joint;
        if depth == 0 {
            joint = (*(*(*archive).unk4).unk8.offset(arg0 as isize)).unk0;
        } else {
            let mut tmp_depth = depth;
            joint = Ground_801C126C((*(*(*archive).unk4).unk8.offset(arg0 as isize)).unk0, &mut tmp_depth);
        }
        result = ext::HSD_JObjLoadJoint(joint);
    }
    result
}

/// `Ground_803B7E0C`: the identity joint `get_jobj_inline` loads with a uniform scale.
pub unsafe extern "C" fn get_jobj_inline(phi_f0: f32) -> *mut HSD_JObj {
    melee_core::ct!("get_jobj_inline");
    let mut sp14: HSD_Joint = core::mem::zeroed();
    sp14.scale = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
    sp14.scale.x = phi_f0;
    sp14.scale.y = phi_f0;
    sp14.scale.z = phi_f0;
    let jobj = ext::HSD_JObjLoadJoint(&mut sp14);
    if jobj.is_null() {
        // OSReport("%s:%d: couldn t get jobj\n", __FILE__, 0x4C4); while (1) {}
        panic!("ground.c:1220 couldn t get jobj");
    }
    jobj
}

pub unsafe extern "C" fn Ground_GetStageGObj(map_id: i32) -> *mut Ground_GObj {
    melee_core::ct!("Ground_GetStageGObj");
    let slot_gobj = ext::ScriptGame_StageSlotGround(map_id);
    if !slot_gobj.is_null() {
        return slot_gobj;
    }
    let stageinfo = &raw mut stage_info;
    let mut phi_f0: f32;
    let mut new_var: *mut HSD_JObj;

    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_STAGE as u16, 5, 0);
    if gobj.is_null() {
        // OSReport("couldn t get gobj!")
        return null_mut();
    }
    let gp = alloc_user_data_ground();
    if gp.is_null() {
        ext::HSD_GObjFree(gobj);
        return null_mut();
    }
    ext::GObj_InitUserData(gobj, 3, Some(mem_free), gp as *mut c_void);
    (*gp).map_id = map_id as GrKind;
    (*gp).gobj = gobj;
    ext::ScriptGame_RefsGroundBorn(map_id, gobj as usize as i32);
    (*gp).x10_flags.set_b0(0);
    (*gp).x10_flags.set_b1(0);
    (*gp).x10_flags.set_b2(1);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    (*gp).x1C_callback = None;
    (*gp).x10_flags.set_b5(0);
    (*gp).x10_flags.set_b6(0);
    (*gp).x11_flags.set_b012(0);
    (*gp).x10_flags.set_b7(0);
    (*gp).x18 = null_mut();
    (*gp).x10_flags.set_b3(0);
    for i in 0..(*gp).x20.len() {
        (*gp).x20[i] = -1;
    }

    ext::grMaterial_801C95C4(gobj);
    let mut archive = ext::grDatFiles_GetArchive();
    assert!(!archive.is_null(), "ground.c:1358 HSD_ASSERT(1358, archive)");

    if ext::Mex_GrTrace() != 0 {
        // OSReport("grtrace: GetStageGObj(%d) from %08x: ...", map_id, Mex_GuestLr(), ...)
        let _ = ext::Mex_GuestLr();
    }
    if map_id < (*(*archive).unk4).unkC {
        archive = ext::grDatFiles_801C6330(map_id);
        let temp_r24 = (*(*(*archive).unk4).unk8.offset(map_id as isize)).unk0;
        let temp_r23 = ext::HSD_JObjLoadJoint(temp_r24);
        Ground_801C34AC(map_id, temp_r23, temp_r24);
        if !(*stageinfo).param.is_null() {
            phi_f0 = (*(*stageinfo).param).y;
        } else {
            phi_f0 = 1.0;
        }
        new_var = get_jobj_inline(phi_f0);
        ext::HSD_JObjAddNext(temp_r23, new_var);
        if new_var.is_null() {
            ext::HSD_GObjFree(gobj);
            return null_mut();
        }
        let map_cb = Ground_MexStageCallbacks((*stageinfo).grkind as i32);
        if !map_cb.is_null()
            && (*map_cb.offset(map_id as isize)).anon_10.anon_0.flags_b2() == 1
            && !(*(*(*archive).unk4).unk8.offset(map_id as isize)).x10.is_null()
        {
            let temp_r23_2 = ext::GObj_Create(17, 19, 0);
            let temp_r27 = ext::lb_80013B14((*(*(*archive).unk4).unk8.offset(map_id as isize)).x10);
            let new_var2 = temp_r23_2;
            ext::HSD_GObjObject_80390A70(temp_r23_2, ext::HSD_GObj_CameraKind, temp_r27 as *mut c_void);
            ext::GObj_SetupGXLinkMax(new_var2, Some(ext::cb::grDisplay_801C5F60), 5);
            (*temp_r23_2).gxlink_prios = 8;
            (*gp).x18 = temp_r23_2;
            Ground_801C2BD4(temp_r27 as *mut c_void);
        }
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, new_var as *mut c_void);
        let mut phi_r24 = (*(*(*archive).unk4).unk8.offset(map_id as isize)).x30;
        let mut phi_r23 = (*(*(*archive).unk4).unk8.offset(map_id as isize)).x2C;
        while phi_r24 != 0 {
            let temp_r3_8 = Ground_801C3FA4(gobj, *phi_r23 as i32);
            if !temp_r3_8.is_null() {
                ext::lb_8000F9F8(temp_r3_8);
            }
            phi_r24 -= 1;
            phi_r23 = phi_r23.add(1);
        }
    } else {
        new_var = ext::HSD_JObjAlloc();
        if !new_var.is_null() {
            ext::PSMTXIdentity((*new_var).mtx.as_mut_ptr() as *mut c_void);
            (*new_var).scl = null_mut();
        }
        if !(*stageinfo).param.is_null() {
            phi_f0 = (*(*stageinfo).param).y;
        } else {
            phi_f0 = 1.0;
        }
        let temp_r3_11 = get_jobj_inline(phi_f0);
        ext::HSD_JObjAddNext(new_var, temp_r3_11);
        if temp_r3_11.is_null() {
            ext::HSD_GObjFree(gobj);
            return null_mut();
        }
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, temp_r3_11 as *mut c_void);
    }
    ext::HSD_GObj_SetupProc(gobj, Some(Ground_801C1CD0), 1);
    ext::HSD_GObj_SetupProc(gobj, Some(Ground_801C1D38), 4);
    Ground_801C2BBC(gobj, map_id);
    gobj
}

pub unsafe extern "C" fn Ground_801C1A20(arg0: *mut HSD_Joint, arg1: i32) -> *mut HSD_GObj {
    melee_core::ct!("Ground_801C1A20");
    let temp_r30 = ext::GObj_Create(HSD_GOBJ_CLASS_STAGE as u16, 5, 0);
    if temp_r30.is_null() {
        return null_mut();
    }
    let gp = alloc_user_data_ground();
    if gp.is_null() {
        ext::HSD_GObjFree(temp_r30);
        return null_mut();
    }
    ext::GObj_InitUserData(temp_r30, 3, Some(mem_free), gp as *mut c_void);
    (*gp).map_id = arg1 as GrKind;
    (*gp).gobj = temp_r30;
    (*gp).x10_flags.set_b0(0);
    (*gp).x10_flags.set_b1(0);
    (*gp).x10_flags.set_b2(1);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    (*gp).x1C_callback = None;
    (*gp).x10_flags.set_b5(0);
    (*gp).x18 = null_mut();
    (*gp).x10_flags.set_b3(0);
    for i in 0..8 {
        (*gp).x20[i] = -1;
    }
    ext::grMaterial_801C95C4(temp_r30);
    let temp_r29 = ext::HSD_JObjLoadJoint(arg0);
    Ground_801C34AC(arg1, temp_r29, arg0);
    let temp_r3_4 = get_jobj_inline(Ground_801C0498());
    ext::HSD_JObjAddNext(temp_r29, temp_r3_4);
    if temp_r3_4.is_null() {
        ext::HSD_GObjFree(temp_r30);
        return null_mut();
    }
    ext::HSD_GObjObject_80390A70(temp_r30, ext::HSD_GObj_JObjKind, temp_r3_4 as *mut c_void);
    ext::HSD_GObj_SetupProc(temp_r30, Some(Ground_801C1CD0), 1);
    ext::HSD_GObj_SetupProc(temp_r30, Some(Ground_801C1D38), 4);
    temp_r30
}

pub unsafe extern "C" fn Ground_801C1CD0(gobj: *mut HSD_GObj) {
    melee_core::ct!("Ground_801C1CD0");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let gp = (*gobj).user_data as *mut Ground;
    ext::HSD_JObjAnimAll(jobj);
    ext::grMaterial_801C9698(gobj);
    ext::mpColl_804D64AC += 1;
    if let Some(f) = (*gp).x8_callback {
        f(gobj);
    }
}

pub unsafe extern "C" fn Ground_801C1D38(gobj: *mut HSD_GObj) {
    melee_core::ct!("Ground_801C1D38");
    let gp = (*gobj).user_data as *mut Ground;
    if let Some(f) = (*gp).xC_callback {
        f(gobj);
    }
}

/// Reuse preloaded slot joints; preserve the retail callback priorities.
pub unsafe extern "C" fn Ground_StageSlotPrepare(gobj: *mut HSD_GObj, map_id: i32) {
    melee_core::ct!("Ground_StageSlotPrepare");
    let gp = (*gobj).user_data as *mut Ground;
    ext::HSD_GObjProc_RemoveAllProcs(gobj);
    core::ptr::write_bytes(gp as *mut u8, 0, size_of::<Ground>());
    (*gp).map_id = map_id as GrKind;
    (*gp).gobj = gobj;
    (*gp).x10_flags.set_b2(1);
    ext::ScriptGame_RefsGroundBorn(map_id, gobj as usize as i32);
    for i in 0..8 {
        (*gp).x20[i] = -1;
    }
    ext::grMaterial_801C95C4(gobj);
    ext::HSD_GObj_SetupProc(gobj, Some(Ground_801C1CD0), 1);
    ext::HSD_GObj_SetupProc(gobj, Some(Ground_801C1D38), 4);
    stage_info.map_gobjs[map_id as usize] = gobj;
}

pub unsafe extern "C" fn Ground_801C1D6C(arg0: u32) {
    melee_core::ct!("Ground_801C1D6C");
    stage_info.flags |= arg0;
}

pub unsafe extern "C" fn Ground_801C1D84() -> u32 {
    melee_core::ct!("Ground_801C1D84");
    stage_info.flags & 0x330
}

pub unsafe extern "C" fn Ground_801C1D98() -> u32 {
    melee_core::ct!("Ground_801C1D98");
    stage_info.flags & 0x20
}

pub unsafe extern "C" fn Ground_801C1DAC() -> u32 {
    melee_core::ct!("Ground_801C1DAC");
    stage_info.flags & 0x100
}

pub unsafe extern "C" fn Ground_801C1DC0() -> u32 {
    melee_core::ct!("Ground_801C1DC0");
    stage_info.flags & 0x80
}

pub unsafe extern "C" fn Ground_801C1DD4() -> i32 {
    melee_core::ct!("Ground_801C1DD4");
    stage_info.x6D0 as i32
}

pub unsafe extern "C" fn Ground_801C1DE4(arg0: *mut i32, arg1: *mut i32) {
    melee_core::ct!("Ground_801C1DE4");
    *arg0 = stage_info.x6D4 as i32;
    *arg1 = stage_info.x6D2 as i32;
}

pub unsafe extern "C" fn Ground_801C1E00(arg0: i32) {
    melee_core::ct!("Ground_801C1E00");
    stage_info.unk8C.set_b2(arg0 as u8);
}

pub unsafe extern "C" fn Ground_801C1E18() -> i32 {
    melee_core::ct!("Ground_801C1E18");
    stage_info.unk8C.b2() as i32
}

pub unsafe extern "C" fn Ground_801C1E2C(gobj: *mut HSD_GObj, code: i32) {
    melee_core::ct!("Ground_801C1E2C");
    if ext::Camera_80030A78() != 0 {
        return;
    }
    let stage_is_something = stage_info.unk8C.b2() != 0;
    if !stage_is_something {
        return;
    }
    let jobj = (*gobj).hsd_obj;
    if !jobj.is_null() {
        ext::HSD_FogSet((*gobj).hsd_obj as *mut HSD_Fog);
    }
}

pub unsafe extern "C" fn Ground_801C1E84() -> *mut HSD_GObj {
    melee_core::ct!("Ground_801C1E84");
    stage_info.x12C
}

/// `static inline HSD_FogDesc* foo(void)`: the first fog desc of a model flagged for fog.
pub unsafe extern "C" fn foo() -> *mut HSD_FogDesc {
    melee_core::ct!("foo");
    ext::grDatFiles_GetArchive();
    let archive = ext::grDatFiles_GetArchive();
    let kind = stage_info.grkind as i32;
    let temp_r29 = Ground_MexStageCallbacks(kind);
    if temp_r29.is_null() {
        return null_mut();
    }
    // A stage file that is only a loader has no model table
    if archive.is_null() || (*archive).unk4.is_null() {
        return null_mut();
    }
    let temp_r30 = (*(*archive).unk4).unkC;
    ext::grDatFiles_GetArchive();
    let mut i = 0;
    while i < temp_r30 {
        let phi_r29 = temp_r29.offset(i as isize);
        if (*phi_r29).anon_10.anon_0.flags_b1() == 1 {
            if ext::grDatFiles_801C6330(i).is_null() {
                i += 1;
                continue;
            }
            if ext::Mex_GrTrace() != 0 {
                // OSReport(..., grDatFiles_801C6330(i)->unk4->unk8[i].x1C)
                let _ = (*(*(*ext::grDatFiles_801C6330(i)).unk4).unk8.offset(i as isize)).x1C;
            }
            return (*(*(*ext::grDatFiles_801C6330(i)).unk4).unk8.offset(i as isize)).x1C;
        }
        i += 1;
    }
    let _ = ext::Mex_GrTrace();
    null_mut()
}

pub unsafe extern "C" fn Ground_801C1E94() {
    melee_core::ct!("Ground_801C1E94");
    let stageinfo = &raw mut stage_info;
    let phi_r0 = foo();
    if !phi_r0.is_null() {
        let temp_r30_2 = ext::GObj_Create(0xA, 0xB, 0);
        let temp_r29_2 = ext::HSD_FogLoadDesc(phi_r0);
        ext::HSD_GObjObject_80390A70(temp_r30_2, ext::HSD_GObj_FogKind as u8, temp_r29_2 as *mut c_void);
        ext::GObj_SetupGXLink(temp_r30_2, Some(Ground_801C1E2C), 0, 0);
        let temp_r3 = (*stageinfo).param;
        let phi_f1 = if !temp_r3.is_null() { (*temp_r3).y } else { 1.0f32 };
        (*temp_r29_2).start *= phi_f1;
        (*temp_r29_2).end *= phi_f1;
        ext::Camera_SetBackgroundColor((*temp_r29_2).color.r, (*temp_r29_2).color.g, (*temp_r29_2).color.b);
        (*stageinfo).x12C = temp_r30_2;
    } else {
        ext::Camera_SetBackgroundColor(0, 0, 0);
    }
}

pub unsafe extern "C" fn Ground_ApplyStageBackgroundColor() {
    melee_core::ct!("Ground_ApplyStageBackgroundColor");
    if !stage_info.x12C.is_null() {
        let fog = (*stage_info.x12C).hsd_obj as *mut HSD_Fog;
        if !fog.is_null() {
            ext::Camera_SetBackgroundColor((*fog).color.r, (*fog).color.g, (*fog).color.b);
            return;
        }
    }
    ext::Camera_SetBackgroundColor(0, 0, 0);
}

pub unsafe extern "C" fn Ground_801C205C(color: *mut _GXColor) {
    melee_core::ct!("Ground_801C205C");
    if !stage_info.x12C.is_null() && !color.is_null() {
        let fog = (*stage_info.x12C).hsd_obj as *mut HSD_Fog;
        if !fog.is_null() {
            (*fog).color = *color;
        }
    }
}

pub unsafe extern "C" fn Ground_801C2090(color: *mut _GXColor) -> i32 {
    melee_core::ct!("Ground_801C2090");
    if !stage_info.x12C.is_null() && !color.is_null() {
        let fog = (*stage_info.x12C).hsd_obj as *mut HSD_Fog;
        if !fog.is_null() {
            *color = (*fog).color;
            return 1;
        }
    }
    0
}

pub unsafe extern "C" fn Ground_801C20D0() -> f32 {
    melee_core::ct!("Ground_801C20D0");
    stage_info.cam_info.cam_vertical_tilt
}

pub unsafe extern "C" fn find_light_override(archive: *mut UnkArchiveStruct, desc: *mut HSD_LightDesc, b6: *mut i32, b7: *mut i32, b5: *mut i32) -> i32 {
    melee_core::ct!("find_light_override");
    let dat = (*archive).unk4;
    let count = (*dat).unk1C;

    if count != 0 {
        for i in 0..count as isize {
            let arr = (*dat).unk18 as *mut LightOverrideEntry;
            if (*arr.offset(i)).desc == desc {
                *b6 = (*arr.offset(i)).b() as i32;
                *b7 = (*arr.offset(i)).a() as i32;
                *b5 = (*arr.offset(i)).c() as i32;
                return 1;
            }
        }
    }
    0
}

pub unsafe extern "C" fn find_light_override_in_dat(dat: *mut UnkStageDat, array_dat: *mut UnkStageDat, desc: *mut HSD_LightDesc, b6: *mut i32, b7: *mut i32, b5: *mut i32) -> i32 {
    melee_core::ct!("find_light_override_in_dat");
    let count = (*dat).unk1C;

    if count != 0 {
        for i in 0..count as isize {
            let arr = (*array_dat).unk18 as *mut LightOverrideEntry;
            if (*arr.offset(i)).desc == desc {
                *b6 = (*arr.offset(i)).b() as i32;
                *b7 = (*arr.offset(i)).a() as i32;
                *b5 = (*arr.offset(i)).c() as i32;
                return 1;
            }
        }
    }
    0
}

pub unsafe extern "C" fn Ground_801C20E0(archive: *mut UnkArchiveStruct, lightset: *mut *mut LightList) -> *mut *mut LightList {
    melee_core::ct!("Ground_801C20E0");
    let mut b6 = 0i32;
    let mut b7 = 0i32;
    let mut b5 = 0i32;

    assert!(!lightset.is_null(), "ground.c:1907 HSD_ASSERT(1907, lightset)");
    assert!(!(*lightset).is_null(), "ground.c:1908 HSD_ASSERT(1908, *lightset)");

    let mut walker = lightset;
    let mut matched = false;
    while !(*walker).is_null() {
        if find_light_override(archive, (**walker).desc, &mut b6, &mut b7, &mut b5) != 0 && (b6 != 0 || b7 != 0 || b5 != 0) {
            matched = true;
            break;
        }
        walker = walker.add(1);
    }

    if !matched {
        return lightset;
    }

    let mut out = lightset;
    while !(*out).is_null() {
        let desc = (**out).desc;
        let flags = &raw mut (*desc).flags;
        if *flags & 3 != 0 {
            let dat = (*archive).unk4;
            if find_light_override_in_dat(dat, (*archive).unk4, desc, &mut b6, &mut b7, &mut b5) == 0 || (b6 == 0 && b7 == 0 && b5 == 0) {
                let mut clean = out;
                loop {
                    *clean = *clean.add(1);
                    if (*clean).is_null() {
                        break;
                    }
                    clean = clean.add(1);
                }
                out = out.wrapping_sub(1);
            } else {
                if b6 != 0 {
                    *flags |= 4;
                } else {
                    *flags &= !4;
                }
                if b7 != 0 {
                    (*(**out).desc).flags |= 8;
                } else {
                    (*(**out).desc).flags &= !8;
                }
                if b5 != 0 {
                    (*(**out).desc).flags |= 0x400;
                } else {
                    (*(**out).desc).flags &= !0x400;
                }
            }
        }
        out = out.wrapping_add(1);
    }
    lightset
}

pub unsafe extern "C" fn Ground_801C2374(lobj: *mut HSD_LObj) {
    melee_core::ct!("Ground_801C2374");
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let vec_mul = Ground_801C0498();
    let mut cur = lobj;
    while !cur.is_null() {
        if ext::HSD_LObjGetPosition(cur, &mut vec) != 0 {
            vec.x *= vec_mul;
            vec.y *= vec_mul;
            vec.z *= vec_mul;
            ext::HSD_LObjSetPosition(cur, &mut vec);
        }
        if ext::HSD_LObjGetInterest(cur, &mut vec) != 0 {
            vec.x *= vec_mul;
            vec.y *= vec_mul;
            vec.z *= vec_mul;
            ext::HSD_LObjSetInterest(cur, &mut vec);
        }
        cur = if cur.is_null() { null_mut() } else { (*cur).next };
    }
}

pub unsafe extern "C" fn Ground_801C247C(arg0: i32, arg1: i32) -> *mut HSD_Spline {
    melee_core::ct!("Ground_801C247C");
    let archive = ext::grDatFiles_801C6330(arg0);
    assert!(!archive.is_null(), "ground.c:2017 HSD_ASSERT(2017, archive)");
    if !(*archive).unk4.is_null() && arg1 < (*(*archive).unk4).unk14 {
        *(*(*archive).unk4).unk10.offset(arg1 as isize)
    } else {
        null_mut()
    }
}

/// Resolve the StKind of the row that actually exists (m-ex added stages have a wildcard row).
pub unsafe extern "C" fn Ground_MexParamStkind(stkind: StKind) -> StKind {
    melee_core::ct!("Ground_MexParamStkind");
    if stage_info.param.is_null() {
        return stkind;
    }
    let p = (*stage_info.param).stage_params;
    let count = (*stage_info.param).stage_param_count as isize;
    if p.is_null() || count <= 0 {
        return stkind;
    }
    for i in 0..count {
        if (*p.offset(i)).stkind == stkind {
            return stkind;
        }
    }
    for i in 0..count {
        if (*p.offset(i)).stkind == -1i32 as StKind {
            return (*p.offset(i)).stkind;
        }
    }
    if count == 1 {
        return (*p).stkind;
    }
    stkind
}

pub unsafe extern "C" fn Ground_801C24F8(stkind: StKind, arg1: u32, arg2: *mut i32) -> i32 {
    melee_core::ct!("Ground_801C24F8");
    let mut stkind = stkind;
    let mut arg1 = arg1;
    let phi_r30_0 = (*stage_info.param).stage_params;
    let mut bgm: i32 = BGM_Undefined;
    let mut result = false;
    stkind = Ground_MexParamStkind(stkind);
    let mut i = 0;
    while i < (*stage_info.param).stage_param_count {
        let phi_r30 = phi_r30_0.offset(i as isize);
        if (*phi_r30).stkind == stkind {
            if arg1 & 4 != 0 {
                let temp_r25 = arg1 & 0x40 != 0;
                match (*phi_r30).x14 {
                    0 => {
                        if temp_r25 {
                            arg1 |= 2;
                        } else {
                            arg1 |= 1;
                        }
                    }
                    1 => {
                        if ext::HSD_Randi(0xC) == 0 || temp_r25 {
                            arg1 |= 2;
                        } else {
                            arg1 |= 1;
                        }
                    }
                    8 => {
                        arg1 |= 1;
                    }
                    2 => {
                        if (*phi_r30).x16 as i32 > ext::HSD_Randi(RANDI_MAX) || temp_r25 {
                            arg1 |= 2;
                        } else {
                            arg1 |= 1;
                        }
                    }
                    3 => {
                        if ext::gm_IsCKindUnlocked(CKind_Mars as u8) != 0
                            && ((*phi_r30).x16 as i32 > ext::HSD_Randi(RANDI_MAX) || temp_r25)
                        {
                            arg1 |= 2;
                        } else {
                            arg1 |= 1;
                        }
                    }
                    4 => {
                        if ext::gm_IsCKindUnlocked(CKind_CLink as u8) != 0
                            && ((*phi_r30).x16 as i32 > ext::HSD_Randi(RANDI_MAX) || temp_r25)
                        {
                            arg1 |= 2;
                        } else {
                            arg1 |= 1;
                        }
                    }
                    5 => {
                        if ext::Toy_803048C0(0x11A) != 0 && ((*phi_r30).x16 as i32 > ext::HSD_Randi(RANDI_MAX) || temp_r25) {
                            arg1 |= 2;
                        } else {
                            arg1 |= 1;
                        }
                    }
                    6 => {
                        // m-ex "Stage Music 50 50" (opt-in): the alternate track always has a 50% chance
                        if ext::Mex_Enabled(b"stage_music_5050\0".as_ptr() as *const i8) != 0 {
                            if ext::gm_80164ABC() != 0 && (50 > ext::HSD_Randi(RANDI_MAX) || temp_r25) {
                                arg1 |= 2;
                            } else {
                                arg1 |= 1;
                            }
                        } else if ext::gm_80164ABC() != 0 && ((*phi_r30).x16 as i32 > ext::HSD_Randi(RANDI_MAX) || temp_r25) {
                            arg1 |= 2;
                        } else {
                            arg1 |= 1;
                        }
                    }
                    7 => {
                        if ext::gm_80164600() != 0 && ((*phi_r30).x16 as i32 > ext::HSD_Randi(RANDI_MAX) || temp_r25) {
                            arg1 |= 2;
                        } else {
                            arg1 |= 1;
                        }
                    }
                    _ => {}
                }
            }
            if arg1 & 0x10 != 0 {
                stage_info.unk8C.set_b0(0);
                if arg1 & 1 != 0 {
                    bgm = (*phi_r30).xC as i32;
                } else if arg1 & 2 != 0 {
                    if (*phi_r30).x10 != BGM_Undefined as u32 {
                        bgm = (*phi_r30).x10 as i32;
                        result = true;
                    } else {
                        bgm = (*phi_r30).xC as i32;
                    }
                }
            } else if arg1 & 0x20 != 0 {
                if arg1 & 1 != 0 {
                    bgm = (*phi_r30).xC as i32;
                    stage_info.unk8C.set_b0(0);
                } else if arg1 & 2 != 0 {
                    if (*phi_r30).x10 != BGM_Undefined as u32 {
                        bgm = (*phi_r30).x10 as i32;
                        stage_info.unk8C.set_b0(1);
                        result = true;
                    } else {
                        bgm = (*phi_r30).xC as i32;
                        stage_info.unk8C.set_b0(0);
                    }
                }
            } else if arg1 & 1 != 0 {
                bgm = (*phi_r30).x4;
                stage_info.unk8C.set_b0(0);
            } else if arg1 & 2 != 0 {
                if (*phi_r30).x8 as u32 != BGM_Undefined as u32 {
                    bgm = (*phi_r30).x8;
                    stage_info.unk8C.set_b0(1);
                    result = true;
                } else {
                    bgm = (*phi_r30).x4;
                    stage_info.unk8C.set_b0(0);
                }
            }
            break;
        }
        i += 1;
    }
    // An m-ex ADDED stage has no row in the vanilla BGM table: its playlist lives in mexData.
    if bgm == BGM_Undefined {
        let n = ext::Mex_GrBgmCount(stage_info.grkind as i32);
        if n > 0 {
            let mut total: i32 = 0;
            let mut roll: i32 = 0;
            let mut acc: i32 = 0;
            for i in 0..n {
                if ext::Mex_GrBgmId(stage_info.grkind as i32, i) > 0 {
                    total = total.wrapping_add(ext::Mex_GrBgmChance(stage_info.grkind as i32, i));
                }
            }
            if total > 0 {
                roll = ext::HSD_Randi(total);
            }
            for i in 0..n {
                let id = ext::Mex_GrBgmId(stage_info.grkind as i32, i);
                if id <= 0 {
                    continue;
                }
                acc = acc.wrapping_add(ext::Mex_GrBgmChance(stage_info.grkind as i32, i));
                if total <= 0 || roll < acc {
                    bgm = id;
                    break;
                }
            }
            // OSReport("Ground: m-ex stage grkind=%d picked bgm %d of %d entries\n")
        }
    }
    assert!(bgm != BGM_Undefined, "ground.c:2242 HSD_ASSERT(2242, bgm!=BGM_Undefined)");
    if bgm == -2 {
        let c = ext::Player_GetPlayerCharacter(0) as i32;
        *arg2 = ext::lbAudioAx_8002305C(c, ext::HSD_Randi(2));
    } else {
        *arg2 = bgm;
    }
    result as i32
}

pub unsafe extern "C" fn Ground_801C28AC(stkind: StKind, arg1: u32, arg2: *mut i32) -> i32 {
    melee_core::ct!("Ground_801C28AC");
    Ground_801C24F8(stkind, arg1, arg2)
}

/// Not reached on PC (the lookup falls back to zeros): the vanilla failure path, a spin.
pub unsafe extern "C" fn panicMissingStageParam(stkind: StKind, count: i32) -> ! {
    melee_core::ct!("panicMissingStageParam");
    panic!("ground.c:2310 not found stage param in DAT(grkind={} stkind={},num={})", stage_info.grkind, stkind, count);
}

pub unsafe extern "C" fn Ground_801C28CC(arg0: *mut i32, stkind: StKind) {
    melee_core::ct!("Ground_801C28CC");
    let mut param = (*stage_info.param).stage_params;
    let count = (*stage_info.param).stage_param_count as isize;
    let stkind = Ground_MexParamStkind(stkind);

    for _ in 0..count {
        if (*param).stkind == stkind {
            for j in 0..35usize {
                *arg0.add(j) = (*stage_info.param).x6A[j] as i32 * (*param).x1A[j] as i32;
            }
            return;
        }
        param = param.add(1);
    }

    // An m-ex ADDED stage has no row for its own StKind: take an explicit -1 wildcard row, or a lone row
    let mut fallback: *mut StageParam = null_mut();
    param = (*stage_info.param).stage_params;
    for _ in 0..count {
        if (*param).stkind == -1i32 as StKind {
            fallback = param;
            break;
        }
        param = param.add(1);
    }
    if fallback.is_null() && count == 1 {
        fallback = (*stage_info.param).stage_params;
    }
    if !fallback.is_null() {
        for j in 0..35usize {
            *arg0.add(j) = (*stage_info.param).x6A[j] as i32 * (*fallback).x1A[j] as i32;
        }
        return;
    }
    // OSReport("Ground_801C28CC: no stage param ...") and carry on with zeroed params
    for j in 0..35usize {
        *arg0.add(j) = 0;
    }
}

pub unsafe extern "C" fn Ground_801C2AD8() -> *mut i32 {
    melee_core::ct!("Ground_801C2AD8");
    stage_info_xA0()
}

pub unsafe extern "C" fn Ground_801C2AE8(stkind: StKind) -> f32 {
    melee_core::ct!("Ground_801C2AE8");
    let mut phi_r5 = (*stage_info.param).stage_params;
    let stkind = Ground_MexParamStkind(stkind);
    let mut i = 0;
    while i < (*stage_info.param).stage_param_count {
        if (*phi_r5).stkind == stkind {
            return (0.01f32 * (*stage_info.param).x68 as f32) * (0.01f32 * (*phi_r5).x18 as f32);
        }
        phi_r5 = phi_r5.add(1);
        i += 1;
    }
    // OSReport("%s:%d: not found stage param in DAT\n"); while (1) {}
    panic!("ground.c:2343 not found stage param in DAT");
}

pub unsafe extern "C" fn Ground_GetMapGObj(map_id: i32) -> *mut Ground_GObj {
    melee_core::ct!("Ground_GetMapGObj");
    stage_info.map_gobjs[map_id as usize]
}

pub unsafe extern "C" fn Ground_801C2BBC(map_gobj: *mut Ground_GObj, index: i32) {
    melee_core::ct!("Ground_801C2BBC");
    stage_info.map_gobjs[index as usize] = map_gobj;
}

pub unsafe extern "C" fn Ground_801C2BD4(arg0: *mut c_void) {
    melee_core::ct!("Ground_801C2BD4");
    for i in 0..Gr_CObj_Max {
        if stage_info.x694[i] == arg0 {
            return;
        }
    }
    let mut i = 0;
    while i < Gr_CObj_Max {
        if stage_info.x694[i].is_null() {
            stage_info.x694[i] = arg0;
            break;
        }
        i += 1;
    }
    assert!(i != Gr_CObj_Max, "ground.c:2381 HSD_ASSERT(2381, i!=Gr_CObj_Max)");
}

pub unsafe extern "C" fn Ground_801C2C8C(arg0: *mut c_void) -> i32 {
    melee_core::ct!("Ground_801C2C8C");
    for i in 0..4 {
        if stage_info.x694[i] == arg0 {
            return 1;
        }
    }
    0
}

pub unsafe extern "C" fn Ground_801C2CF4(i: i32) -> *mut HSD_JObj {
    melee_core::ct!("Ground_801C2CF4");
    stage_info.x280[i as usize]
}

pub unsafe extern "C" fn Ground_801C2D0C(i: i32, jobj: *mut HSD_JObj) {
    melee_core::ct!("Ground_801C2D0C");
    stage_info.x280[i as usize] = jobj;
}

pub unsafe extern "C" fn Ground_801C2D24(arg0: i32, arg1: *mut Vec3) -> i32 {
    melee_core::ct!("Ground_801C2D24");
    let mut sp20 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp14 = sp20;
    if arg0 == 8 {
        Ground_801C2D24(4, arg1);
        Ground_801C2D24(5, &mut sp20);
        ext::lbVector_Add(arg1, &mut sp20);
        (*arg1).x *= 0.5f32;
        (*arg1).y *= 0.5f32;
        (*arg1).z *= 0.5f32;
        return 1;
    }
    if arg0 == 9 {
        Ground_801C2D24(6, arg1);
        Ground_801C2D24(7, &mut sp14);
        ext::lbVector_Add(arg1, &mut sp14);
        (*arg1).x *= 0.5f32;
        (*arg1).y *= 0.5f32;
        (*arg1).z *= 0.5f32;
        return 1;
    }
    if !stage_info.x280[arg0 as usize].is_null() {
        ext::lb_8000B1CC(stage_info.x280[arg0 as usize], null_mut(), arg1);
        return 1;
    }
    if (arg0 as u32).wrapping_sub(1) <= 2 {
        return Ground_801C2D24(0, arg1);
    }
    if (arg0 as u32).wrapping_sub(5) <= 2 {
        return Ground_801C2D24(4, arg1);
    }
    if arg0 == 0x7F {
        if Ground_801C2D24(0x94, arg1) != 0 {
            (*arg1).y += 50.0f32;
            return 1;
        }
        return Ground_801C2D24(0, arg1);
    }
    if arg0 == 4 {
        ext::Stage_UnkSetVec3TCam_Offset(arg1);
        return 1;
    }
    0
}

pub unsafe extern "C" fn Ground_InitMapColl(jobj: *mut HSD_JObj, arg1: i32) -> i32 {
    melee_core::ct!("Ground_InitMapColl");
    if ext::ScriptGame_StageSlotCollision(arg1) != 0 {
        return 1;
    }
    let mut result = 0;
    let temp_r3 = ext::grDatFiles_801C6330(arg1);
    if stage_datas[stage_info.grkind as usize].is_null() {
        return 0;
    }
    if !temp_r3.is_null() {
        let mut cur = (*(*(*temp_r3).unk4).unk8.offset(arg1 as isize)).unk20;
        let max = (*(*(*temp_r3).unk4).unk8.offset(arg1 as isize)).unk24;
        for _ in 0..max {
            ext::mpLib_800552B0((*cur).x as i32, jobj, (*cur).z as i32);
            ext::mpLib_80055E9C((*cur).x as i32);
            ext::mpLib_80057424((*cur).x as i32);
            result = 1;
            cur = cur.add(1);
        }
    }
    let max = (*stage_datas[stage_info.grkind as usize]).joint_count as i32;
    let mut cur = (*stage_datas[stage_info.grkind as usize]).joints;
    for _ in 0..max {
        if (*cur).y as i32 == arg1 {
            ext::mpLib_800552B0((*cur).x as i32, jobj, (*cur).z as i32);
            ext::mpLib_80055E9C((*cur).x as i32);
            ext::mpLib_80057424((*cur).x as i32);
            result = 1;
        }
        cur = cur.add(1);
    }
    Ground_801C3214(arg1);
    result
}

pub unsafe extern "C" fn Ground_UpdateMapColl(arg0: *mut Ground_GObj) -> i32 {
    melee_core::ct!("Ground_UpdateMapColl");
    if ext::ScriptGame_StageSlotCollision((*((*arg0).user_data as *mut Ground)).map_id as i32) != 0 {
        return 1;
    }
    let gr = (*arg0).user_data as *mut Ground;
    let map_id = (*gr).map_id as i32;

    if *Ground_804D6950.offset(map_id as isize) == 0 {
        let mut result = 0;

        let temp_r3 = ext::mpGetGroundCollJoint();
        Ground_804D6954 = Ground_804D6954.wrapping_add(1);
        let stagedata = stage_datas[stage_info.grkind as usize];
        let count = (*stagedata).joint_count as i32;
        let mut vec = (*stagedata).joints;

        for _ in 0..count {
            if (*vec).y as i32 == map_id {
                ext::mpLib_80055E9C((*vec).x as i32);
                (*temp_r3.offset((*vec).x as isize)).xC = Ground_804D6954;
                result = 1;
            }
            vec = vec.add(1);
        }

        let archive = ext::grDatFiles_801C6330(map_id);
        if !archive.is_null() {
            let dat = (*(*archive).unk4).unk8.offset(map_id as isize);
            let mut vec = (*dat).unk20;
            let count = (*dat).unk24;

            for _ in 0..count {
                if Ground_804D6954 != (*temp_r3.offset((*vec).x as isize)).xC {
                    ext::mpLib_80055E9C((*vec).x as i32);
                    (*temp_r3.offset((*vec).x as isize)).xC = Ground_804D6954;
                    result = 1;
                }
                vec = vec.add(1);
            }
        }

        return result;
    }

    0
}

pub unsafe extern "C" fn Ground_801C3128(gobj_id: i32, arg1: Option<unsafe extern "C" fn(i32)>) -> i32 {
    melee_core::ct!("Ground_801C3128");
    ext::mpGetGroundCollJoint();
    let mut result = 0;
    let max = (*stage_datas[stage_info.grkind as usize]).joint_count as i32;
    let mut cur = (*stage_datas[stage_info.grkind as usize]).joints;
    for _ in 0..max {
        if (*cur).y as i32 == gobj_id {
            arg1.unwrap()((*cur).x as i32);
            result = 1;
        }
        cur = cur.add(1);
    }
    let tmp = ext::grDatFiles_801C6330(gobj_id);
    if !tmp.is_null() {
        let e = (*(*tmp).unk4).unk8.offset(gobj_id as isize);
        let mut cur = (*e).unk20;
        let max = (*e).unk24;
        for _ in 0..max {
            arg1.unwrap()((*cur).x as i32);
            result = 1;
            cur = cur.add(1);
        }
    }
    result
}

pub unsafe extern "C" fn Ground_801C3214(gobj_id: i32) -> i32 {
    melee_core::ct!("Ground_801C3214");
    if *Ground_804D6950.offset(gobj_id as isize) == 1 {
        *Ground_804D6950.offset(gobj_id as isize) = 0;
        return Ground_801C3128(gobj_id, Some(ext::cb::mpJointListAdd));
    }
    0
}

pub unsafe extern "C" fn Ground_801C3260(arg0: i32) -> i32 {
    melee_core::ct!("Ground_801C3260");
    if *Ground_804D6950.offset(arg0 as isize) == 0 {
        *Ground_804D6950.offset(arg0 as isize) = 1;
        return Ground_801C3128(arg0, Some(ext::cb::mpLib_80057BC0));
    }
    0
}

pub unsafe extern "C" fn Ground_801C32AC(gobj_id: i32) {
    melee_core::ct!("Ground_801C32AC");
    Ground_801C3128(gobj_id, Some(ext::cb::mpLib_80057424));
}

pub unsafe extern "C" fn Ground_801C32D4(arg0: i32, arg1: i32) -> i32 {
    melee_core::ct!("Ground_801C32D4");
    ext::mpGetGroundCollJoint();
    let max = (*stage_datas[stage_info.grkind as usize]).joint_count as i32;
    let mut cur = (*stage_datas[stage_info.grkind as usize]).joints;
    let mut result = -1;
    for _ in 0..max {
        if (*cur).y as i32 == arg0 && (*cur).z as i32 == arg1 {
            result = (*cur).x as i32;
        }
        cur = cur.add(1);
    }
    let tmp = ext::grDatFiles_801C6330(arg0);
    if !tmp.is_null() {
        let e = (*(*tmp).unk4).unk8.offset(arg0 as isize);
        let max = (*e).unk24;
        let mut cur = (*e).unk20;
        for _ in 0..max {
            if (*cur).z as i32 == arg1 {
                result = (*cur).x as i32;
            }
            cur = cur.add(1);
        }
    }
    result
}

pub unsafe extern "C" fn Ground_801C33C0(arg0: i32, arg1: i32) -> i32 {
    melee_core::ct!("Ground_801C33C0");
    ext::mpGetGroundCollJoint();
    let max = (*stage_datas[stage_info.grkind as usize]).joint_count as i32;
    let mut cur = (*stage_datas[stage_info.grkind as usize]).joints;
    let mut result = -1;
    for _ in 0..max {
        if (*cur).y as i32 == arg0 && (*cur).x as i32 == arg1 {
            result = (*cur).z as i32;
        }
        cur = cur.add(1);
    }
    let tmp = ext::grDatFiles_801C6330(arg0);
    if !tmp.is_null() {
        let e = (*(*tmp).unk4).unk8.offset(arg0 as isize);
        let max = (*e).unk24;
        let mut cur = (*e).unk20;
        for _ in 0..max {
            if (*cur).x as i32 == arg1 {
                result = (*cur).z as i32;
            }
            cur = cur.add(1);
        }
    }
    result
}

/// `{ void* joint; s16* pairs; s32 pair_count; }`, the native view of a stage-DAT joint-pair entry.
#[repr(C)]
struct JointPairEntry {
    joint: *mut c_void,
    pairs: *mut i16,
    pair_count: i32,
}

/// `{ void* joint; u8 x4_pad[8]; }`
#[repr(C)]
struct JointEntry {
    joint: *mut c_void,
    x4_pad: [u8; 8],
}

pub unsafe extern "C" fn Ground_801C34AC(map_id: i32, root: *mut HSD_JObj, joint: *mut HSD_Joint) {
    melee_core::ct!("Ground_801C34AC");
    let mut jobj = root;
    let mut prev_index: i32 = -1;
    let archive = ext::grDatFiles_801C6330(map_id);
    if archive.is_null() {
        return;
    }
    if root.is_null() || joint.is_null() {
        // OSReport("%s:%d:Error (root=%08x joint=%08x)\n")
        return;
    }
    let stage_dat = (*archive).unk4;
    let entry_count = (*stage_dat).unk4;
    if entry_count == 0 {
        return;
    }
    let mut i = 0;
    let mut entry = (*stage_dat).unk0 as *mut JointPairEntry;
    loop {
        if i < entry_count {
            if (*entry).joint == joint as *mut c_void {
                break;
            }
        } else {
            return;
        }
        entry = entry.add(1);
        i += 1;
    }
    let count = (*entry).pair_count;
    let mut pair = (*entry).pairs;
    let mut j = count;
    while j > 0 {
        let target = *pair as i32;
        if prev_index > target || prev_index == -1 {
            jobj = root;
            i = 0;
        } else {
            i = prev_index;
        }
        while !jobj.is_null() {
            if i == target {
                break;
            }
            if (*jobj).flags & JOBJ_INSTANCE as u32 == 0 && !jchild(jobj).is_null() {
                jobj = jchild(jobj);
            } else if !jnext(jobj).is_null() {
                jobj = jnext(jobj);
            } else {
                loop {
                    if jparent(jobj).is_null() {
                        jobj = null_mut();
                        break;
                    }
                    if !jnext(jparent(jobj)).is_null() {
                        jobj = jnext(jparent(jobj));
                        break;
                    }
                    jobj = jparent(jobj);
                }
            }
            i += 1;
        }
        prev_index = i;
        stage_info.x280[*pair.add(1) as usize] = jobj;
        pair = pair.add(2);
        j -= 1;
    }
}

pub unsafe extern "C" fn Ground_801C36F4(map_id: i32, root: *mut HSD_JObj, joint: *mut c_void) {
    melee_core::ct!("Ground_801C36F4");
    let archive = ext::grDatFiles_801C6330(map_id);
    assert!(!archive.is_null(), "ground.c:2936 HSD_ASSERT(2936, archive)");
    if root.is_null() || joint.is_null() {
        // OSReport("%s:%d:Error (root=%08x joint=%08x)\n")
        return;
    }
    let stage_dat = (*archive).unk4;
    let entry_count = (*stage_dat).unk4;
    if entry_count == 0 {
        return;
    }
    let mut i = 0;
    let mut entry = (*stage_dat).unk0 as *mut JointEntry;
    loop {
        if i < entry_count {
            if (*entry).joint == joint {
                break;
            }
        } else {
            return;
        }
        entry = entry.add(1);
        i += 1;
    }

    for i in 0..0x57 * 3usize {
        let mut jobj = stage_info.x280[i];
        if !jobj.is_null() {
            while !(*jobj).parent.is_null() {
                jobj = (*jobj).parent;
            }
            if jobj == root {
                stage_info.x280[i] = null_mut();
            }
        }
    }
}

pub unsafe extern "C" fn Ground_801C3880(val: f32) {
    melee_core::ct!("Ground_801C3880");
    stage_info.cam_info.cam_bounds.top = val;
}

pub unsafe extern "C" fn Ground_801C3890(val: f32) {
    melee_core::ct!("Ground_801C3890");
    stage_info.cam_info.cam_bounds.bottom = val;
}

pub unsafe extern "C" fn Ground_801C38A0(val: f32) {
    melee_core::ct!("Ground_801C38A0");
    stage_info.cam_info.cam_bounds.left = val;
}

pub unsafe extern "C" fn Ground_801C38AC(val: f32) {
    melee_core::ct!("Ground_801C38AC");
    stage_info.cam_info.cam_bounds.right = val;
}

pub unsafe extern "C" fn Ground_801C38BC(x: f32, y: f32) {
    melee_core::ct!("Ground_801C38BC");
    stage_info.cam_info.cam_x_offset = x;
    stage_info.cam_info.cam_y_offset = y;
}

pub unsafe extern "C" fn Ground_801C38D0(tilt: f32, pan: f32, a: f32, b: f32) {
    melee_core::ct!("Ground_801C38D0");
    stage_info.cam_info.cam_vertical_tilt = tilt;
    stage_info.cam_info.cam_pan_degrees = pan;
    stage_info.cam_info.x20 = a;
    stage_info.cam_info.x24 = b;
}

pub unsafe extern "C" fn Ground_801C38EC(depth: f32, zoom: f32) {
    melee_core::ct!("Ground_801C38EC");
    stage_info.cam_info.cam_max_depth = depth;
    stage_info.cam_info.cam_zoom_rate = zoom;
}

pub unsafe extern "C" fn Ground_801C3900(arg8: f32, arg9: f32, arg_a: f32, arg_b: f32, up: f32, down: f32, left: f32, right: f32) {
    melee_core::ct!("Ground_801C3900");
    stage_info.cam_info.x3C = arg8;
    stage_info.cam_info.pausecam_zpos_min = arg9;
    stage_info.cam_info.pausecam_zpos_init = arg_a;
    stage_info.cam_info.pausecam_zpos_max = arg_b;
    stage_info.cam_info.cam_angle_up = up;
    stage_info.cam_info.cam_angle_down = down;
    stage_info.cam_info.cam_angle_left = left;
    stage_info.cam_info.cam_angle_right = right;
}

pub unsafe extern "C" fn Ground_801C392C(x: f32, y: f32, z: f32, fov: f32, vert: f32, horz: f32) {
    melee_core::ct!("Ground_801C392C");
    stage_info.cam_info.fixed_cam_pos.x = x;
    stage_info.cam_info.fixed_cam_pos.y = y;
    stage_info.cam_info.fixed_cam_pos.z = z;
    stage_info.cam_info.fixed_cam_fov = fov;
    stage_info.cam_info.fixed_cam_vert_angle = vert;
    stage_info.cam_info.fixed_cam_horz_angle = horz;
}

pub unsafe extern "C" fn Ground_801C3950(zoom: f32) {
    melee_core::ct!("Ground_801C3950");
    stage_info.cam_info.cam_fixed_zoom = zoom;
}

pub unsafe extern "C" fn Ground_801C3960(ratio: f32) {
    melee_core::ct!("Ground_801C3960");
    stage_info.cam_info.cam_track_ratio = ratio;
}

pub unsafe extern "C" fn Ground_801C3970(smooth: f32) {
    melee_core::ct!("Ground_801C3970");
    stage_info.cam_info.cam_track_smooth = smooth;
}

pub unsafe extern "C" fn Ground_801C3980(top: f32) {
    melee_core::ct!("Ground_801C3980");
    stage_info.blast_zone.top = top;
}

pub unsafe extern "C" fn Ground_801C3990(bottom: f32) {
    melee_core::ct!("Ground_801C3990");
    stage_info.blast_zone.bottom = bottom;
}

pub unsafe extern "C" fn Ground_801C39A0(left: f32) {
    melee_core::ct!("Ground_801C39A0");
    stage_info.blast_zone.left = left;
}

pub unsafe extern "C" fn Ground_801C39B0(right: f32) {
    melee_core::ct!("Ground_801C39B0");
    stage_info.blast_zone.right = right;
}

pub unsafe extern "C" fn Ground_801C39C0() {
    melee_core::ct!("Ground_801C39C0");
    let mut sp20 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp14 = sp20;
    let mut sp8 = sp20;
    let phi_f3: f32;
    let phi_f4: f32;
    let phi_f5: f32;
    let phi_f0: f32;
    let phi_f1: f32;
    let phi_f2: f32;
    if Ground_801C2D24(0x95, &mut sp20) != 0 && Ground_801C2D24(0x96, &mut sp14) != 0 && Ground_801C2D24(0x94, &mut sp8) != 0 {
        if sp20.x < sp14.x {
            phi_f3 = sp20.x - sp8.x;
            phi_f4 = sp14.x - sp8.x;
        } else {
            phi_f3 = sp14.x - sp8.x;
            phi_f4 = sp20.x - sp8.x;
        }
        if sp20.y < sp14.y {
            phi_f5 = sp20.y - sp8.y;
            phi_f0 = sp14.y - sp8.y;
        } else {
            phi_f5 = sp14.y - sp8.y;
            phi_f0 = sp20.y - sp8.y;
        }
        phi_f1 = sp8.x;
        phi_f2 = sp8.y;
    } else {
        // OSReport("use dummy CamRange ...!\n")
        match stage_info.grkind {
            Gr_Kind_Castle => {
                phi_f3 = -170.0;
                phi_f4 = 170.0;
                phi_f0 = 120.0;
                phi_f5 = -100.0;
                phi_f1 = 0.0;
                phi_f2 = 180.0;
            }
            Gr_Kind_Corneria => {
                phi_f3 = -470.0;
                phi_f4 = 470.0;
                phi_f0 = 120.0;
                phi_f5 = -60.0;
                phi_f1 = 0.0;
                phi_f2 = 300.0;
            }
            Gr_Kind_Unk26 => {
                phi_f3 = -470.0;
                phi_f4 = 470.0;
                phi_f0 = 180.0;
                phi_f5 = -60.0;
                phi_f1 = 0.0;
                phi_f2 = 130.0;
            }
            Gr_Kind_Inishie2 => {
                phi_f3 = -200.0;
                phi_f4 = 200.0;
                phi_f0 = 200.0;
                phi_f5 = -60.0;
                phi_f1 = 0.0;
                phi_f2 = 130.0;
            }
            Gr_Kind_RCruise => {
                phi_f1 = 0.0;
                phi_f3 = -200.0;
                phi_f2 = 0.0;
                phi_f4 = 200.0;
                phi_f0 = 150.0;
                phi_f5 = -160.0;
            }
            Gr_Kind_Yorster => {
                phi_f1 = 0.0;
                phi_f3 = -170.0;
                phi_f2 = 0.0;
                phi_f4 = 170.0;
                phi_f0 = 180.0;
                phi_f5 = -60.0;
            }
            Gr_Kind_MuteCity => {
                phi_f1 = 0.0;
                phi_f3 = -170.0;
                phi_f2 = 0.0;
                phi_f4 = 170.0;
                phi_f0 = 180.0;
                phi_f5 = 10.0;
            }
            _ => {
                phi_f1 = 0.0;
                phi_f3 = -170.0;
                phi_f2 = 0.0;
                phi_f4 = 170.0;
                phi_f0 = 120.0;
                phi_f5 = -60.0;
            }
        }
    }
    stage_info.cam_info.cam_bounds.top = phi_f0;
    stage_info.cam_info.cam_bounds.bottom = phi_f5;
    stage_info.cam_info.cam_bounds.left = phi_f3;
    stage_info.cam_info.cam_bounds.right = phi_f4;
    stage_info.cam_info.cam_x_offset = phi_f1;
    stage_info.cam_info.cam_y_offset = phi_f2;
}

pub unsafe extern "C" fn Ground_801C3BB4() {
    melee_core::ct!("Ground_801C3BB4");
    let mut sp1C = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp10 = sp1C;
    let lft: f32;
    let rgt: f32;
    let top: f32;
    let bot: f32;
    if Ground_801C2D24(0x97, &mut sp1C) != 0 && Ground_801C2D24(0x98, &mut sp10) != 0 {
        if sp1C.x < sp10.x {
            lft = sp1C.x - stage_info.cam_info.cam_x_offset;
            rgt = sp10.x - stage_info.cam_info.cam_x_offset;
        } else {
            lft = sp10.x - stage_info.cam_info.cam_x_offset;
            rgt = sp1C.x - stage_info.cam_info.cam_x_offset;
        }
        if sp1C.y < sp10.y {
            bot = sp1C.y - stage_info.cam_info.cam_y_offset;
            top = sp10.y - stage_info.cam_info.cam_y_offset;
        } else {
            bot = sp10.y - stage_info.cam_info.cam_y_offset;
            top = sp1C.y - stage_info.cam_info.cam_y_offset;
        }
    } else {
        // OSReport("use dummy DeadRange ...\n")
        match stage_info.grkind {
            Gr_Kind_Castle => {
                lft = -250.0;
                rgt = 250.0;
                top = 180.0;
                bot = -150.0;
            }
            Gr_Kind_Corneria => {
                lft = -550.0;
                rgt = 550.0;
                top = 200.0;
                bot = -100.0;
            }
            Gr_Kind_Unk26 => {
                lft = -600.0;
                rgt = 600.0;
                top = 200.0;
                bot = -100.0;
            }
            Gr_Kind_Shrine => {
                lft = -550.0;
                rgt = 550.0;
                top = 200.0;
                bot = -150.0;
            }
            Gr_Kind_Inishie2 => {
                lft = -300.0;
                rgt = 300.0;
                top = 300.0;
                bot = -160.0;
            }
            Gr_Kind_RCruise => {
                lft = -300.0;
                rgt = 300.0;
                top = 270.0;
                bot = -240.0;
            }
            Gr_Kind_Yorster => {
                lft = -300.0;
                rgt = 300.0;
                top = 210.0;
                bot = -240.0;
            }
            _ => {
                lft = -250.0;
                rgt = 250.0;
                top = 200.0;
                bot = -100.0;
            }
        }
    }
    stage_info.blast_zone.top = top;
    stage_info.blast_zone.bottom = bot;
    stage_info.blast_zone.left = lft;
    stage_info.blast_zone.right = rgt;
}

type StageProbe = Option<unsafe extern "C" fn(*mut Vec, i32) -> i32>;

pub unsafe extern "C" fn Ground_801C3D44(arg0: *mut c_void, arg8: f32, arg9: f32) -> i32 {
    melee_core::ct!("Ground_801C3D44");
    stage_info.unk8C.set_b6(1);
    stage_info.x70C = 0.5f32 * arg8;
    stage_info.x710 = 0.5f32 * arg9;
    stage_info.x90 = core::mem::transmute::<*mut c_void, StageProbe>(arg0);
    if stage_info.x714 != -1 {
        stage_info.flags |= 0x10;
        stage_info.x6D0 = stage_info.x714 as i16;
    } else {
        stage_info.flags &= 0xFFFFFFEF;
    }
    stage_info.x714
}

pub unsafe extern "C" fn Ground_801C3DB4(arg0: *mut c_void, arg8: f32, arg9: f32) -> i32 {
    melee_core::ct!("Ground_801C3DB4");
    stage_info.unk8C.set_b7(1);
    stage_info.x718 = 0.5f32 * arg8;
    stage_info.x71C = 0.5f32 * arg9;
    stage_info.x94 = core::mem::transmute::<*mut c_void, StageProbe>(arg0);
    if stage_info.x720 != -1 {
        stage_info.flags |= 0x40;
    } else {
        stage_info.flags &= 0xFFFFFFBF;
    }
    stage_info.x720
}

pub unsafe extern "C" fn Ground_801C3E18(jobj: *mut HSD_JObj) -> *mut HSD_AObj {
    melee_core::ct!("Ground_801C3E18");
    if jobj.is_null() {
        return null_mut();
    }
    if !(*jobj).aobj.is_null() {
        return (*jobj).aobj;
    }
    let result = Ground_801C3E18((*jobj).child);
    if !result.is_null() {
        return result;
    }
    let result = Ground_801C3E18((*jobj).next);
    if !result.is_null() {
        return result;
    }
    null_mut()
}

pub unsafe extern "C" fn Ground_801C3F20(arg0: *mut HSD_JObj) -> f32 {
    melee_core::ct!("Ground_801C3F20");
    let aobj = Ground_801C3E18(arg0);
    if !aobj.is_null() {
        return (*aobj).curr_frame;
    }
    0.0
}

pub unsafe extern "C" fn Ground_801C3FA4(gobj: *mut HSD_GObj, depth: i32) -> *mut HSD_JObj {
    melee_core::ct!("Ground_801C3FA4");
    let mut depth = depth;
    let mut jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if (*gobj).hsd_obj.is_null() {
        return null_mut();
    }
    jobj = jchild(jobj);
    if jobj.is_null() {
        return null_mut();
    }
    while !jobj.is_null() && depth != 0 {
        depth -= 1;
        if (*jobj).flags & 0x1000 == 0 && !jchild(jobj).is_null() {
            jobj = jchild(jobj);
            continue;
        }
        if !jnext(jobj).is_null() {
            jobj = jnext(jobj);
            continue;
        }
        loop {
            if jparent(jobj).is_null() {
                jobj = null_mut();
                break;
            }
            if !jnext(jparent(jobj)).is_null() {
                jobj = jnext(jparent(jobj));
                break;
            }
            jobj = jparent(jobj);
        }
    }
    jobj
}

pub unsafe extern "C" fn Ground_801C4100(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("Ground_801C4100");
    let mut jobj = jobj;
    if (*jobj).flags & JOBJ_INSTANCE as u32 == 0 && !jchild(jobj).is_null() {
        return jchild(jobj);
    }
    if !jnext(jobj).is_null() {
        return jnext(jobj);
    }
    loop {
        if jparent(jobj).is_null() {
            return null_mut();
        }
        if !jnext(jparent(jobj)).is_null() {
            return jnext(jparent(jobj));
        }
        jobj = jparent(jobj);
    }
}

/// `Ground_TTMod_BoxVerts`: the unit box the Break-the-Targets platform mod draws.
pub static mut Ground_TTMod_BoxVerts: [[f32; 3]; 8] = [
    [-0.5, -0.5, -0.5], [0.5, -0.5, -0.5],
    [0.5, 0.5, -0.5],   [-0.5, 0.5, -0.5],
    [-0.5, -0.5, 0.5],  [0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],    [-0.5, 0.5, 0.5],
];

pub static mut Ground_TTMod_BoxDisp: [u8; 64] = {
    let mut a = [0u8; 64];
    let d: [u8; 39] = [
        GX_DRAW_TRIANGLES, 0x00, 0x24,
        4, 5, 6, 4, 6, 7, 1, 0, 3, 1, 3, 2,
        3, 7, 6, 3, 6, 2, 0, 1, 5, 0, 5, 4,
        0, 4, 7, 0, 7, 3, 1, 6, 5, 1, 2, 6,
    ];
    let mut i = 0;
    while i < 39 {
        a[i] = d[i];
        i += 1;
    }
    a
};

pub static mut Ground_TTMod_BoxVtxDesc: [HSD_VtxDescList; 2] = unsafe {
    [
        HSD_VtxDescList {
            attr: GX_VA_POS,
            attr_type: GX_INDEX8,
            comp_cnt: GX_POS_XYZ,
            comp_type: GX_F32,
            frac: 0,
            stride: 12,
            vertex: &raw mut Ground_TTMod_BoxVerts as *mut c_void,
        },
        HSD_VtxDescList {
            attr: GX_VA_NULL,
            attr_type: GX_NONE,
            comp_cnt: GX_POS_XY,
            comp_type: GX_U8,
            frac: 0,
            stride: 0,
            vertex: null_mut(),
        },
    ]
};

pub static mut Ground_TTMod_BoxMat: HSD_Material = HSD_Material {
    ambient: _GXColor { r: 0x80, g: 0x80, b: 0x80, a: 0xFF },
    diffuse: _GXColor { r: 0xE0, g: 0xE0, b: 0xE0, a: 0xFF },
    specular: _GXColor { r: 0, g: 0, b: 0, a: 0 },
    alpha: 1.0,
    shininess: 0.0,
};

pub static mut Ground_TTMod_BoxPObjDesc: HSD_PObjDesc = unsafe {
    HSD_PObjDesc {
        class_name: null_mut(),
        next: null_mut(),
        verts: &raw mut Ground_TTMod_BoxVtxDesc as *mut HSD_VtxDescList,
        flags: 0,
        n_display: 2,
        display: &raw mut Ground_TTMod_BoxDisp as *mut u8,
        u: core::mem::zeroed(),
    }
};

pub static mut Ground_TTMod_BoxMObjDesc: HSD_MObjDesc = unsafe {
    HSD_MObjDesc {
        class_name: null_mut(),
        rendermode: RENDER_DIFFUSE_MAT,
        texdesc: null_mut(),
        mat: &raw mut Ground_TTMod_BoxMat,
        renderdesc: null_mut(),
        pedesc: null_mut(),
    }
};

pub static mut Ground_TTMod_BoxDObjDesc: HSD_DObjDesc = unsafe {
    HSD_DObjDesc {
        class_name: null_mut(),
        next: null_mut(),
        mobjdesc: &raw mut Ground_TTMod_BoxMObjDesc,
        pobjdesc: &raw mut Ground_TTMod_BoxPObjDesc,
    }
};

pub static mut Ground_TTMod_BoxJoint: HSD_Joint = unsafe {
    let mut j: HSD_Joint = core::mem::zeroed();
    j.flags = (JOBJ_JOINT | JOBJ_OPA) as u32;
    j.u.dobjdesc = &raw mut Ground_TTMod_BoxDObjDesc;
    j.scale = Vec { x: 1.0, y: 1.0, z: 1.0 };
    j
};

pub unsafe extern "C" fn Ground_TTMod_AddPlatformBoxes(level: i32) {
    melee_core::ct!("Ground_TTMod_AddPlatformBoxes");
    let mut scale = Ground_801C0498();

    let nplat = ext::TTMod_PlatformCount(level);
    if nplat <= 0 {
        return;
    }
    let gobj = Ground_GetMapGObj(0);
    if gobj.is_null() || (*gobj).hsd_obj.is_null() {
        return;
    }
    let root = jchild((*gobj).hsd_obj as *mut HSD_JObj);
    if root.is_null() {
        return;
    }
    if scale <= 0.001f32 {
        scale = 1.0;
    }
    let inv = 1.0f32 / scale;

    for i in 0..nplat {
        let mut cx = 0.0f32;
        let mut cy = 0.0f32;
        let mut cz = 0.0f32;
        let mut w = 0.0f32;
        let mut d = 0.0f32;
        let mut tr = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut sc = tr;

        ext::TTMod_Platform(level, i, &mut cx, &mut cy, &mut cz, &mut w, &mut d);
        let box_ = ext::HSD_JObjLoadJoint(&raw mut Ground_TTMod_BoxJoint);
        if box_.is_null() {
            continue;
        }
        sc.x = w * inv;
        sc.y = 5.0f32 * inv;
        sc.z = d * inv;
        HSD_JObjSetScale(&mut *box_, &sc, &mut Dirty);
        tr.x = cx * inv;
        tr.y = (cy - 2.5f32) * inv;
        tr.z = cz * inv;
        HSD_JObjSetTranslate(&mut *box_, &tr, &mut Dirty);
        ext::HSD_JObjAddChild(root, box_);
    }
}

pub unsafe extern "C" fn Ground_801C4210() -> i32 {
    melee_core::ct!("Ground_801C4210");
    let mut count: i32 = 0;
    {
        // Data-driven Break-the-Targets layout (mods/targettest/<name>.tt)
        let ckind = *(ext::gm_801B6320() as *mut i8) as i32;
        let level = ext::TTMod_ForCharacter(ckind);
        if level >= 0 {
            let n = ext::TTMod_TargetCount(level);
            let mut i = 0;
            while i < n {
                let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                ext::TTMod_Target(level, i, &mut pos.x, &mut pos.y, &mut pos.z);
                if !ext::it_8027B5B0(It_Kind_Mato as i32, &mut pos, null_mut(), null_mut(), 0).is_null() {
                    count += 1;
                }
                i += 1;
            }
            stage_info.x6D4 = count as i16;
            stage_info.x6D2 = count as i16;
            Ground_TTMod_AddPlatformBoxes(level);
            return count;
        }
    }
    let mut i = 199;
    while i < 220 {
        if !stage_info.x280[i].is_null()
            && !ext::it_8027B5B0(It_Kind_Mato as i32, null_mut(), stage_info.x280[i], null_mut(), 0).is_null()
        {
            count += 1;
        }
        i += 1;
    }
    stage_info.x6D4 = count as i16;
    stage_info.x6D2 = count as i16;
    count
}

pub unsafe extern "C" fn Ground_801C42AC() {
    melee_core::ct!("Ground_801C42AC");
    let mut sp8: BobOmbRain = core::mem::zeroed();
    for i in 0xFC..0x104usize {
        let jobj = stage_info.x280[i];
        if !jobj.is_null() {
            sp8.x14 = 0x14;
            sp8.x4 = jobj;
            sp8.x1C.anon_0.set_b0(1);
            ext::it_8026BE84(&mut sp8);
        }
    }
}

pub unsafe extern "C" fn Ground_801C4338() {
    melee_core::ct!("Ground_801C4338");
    stage_info.x6D4 = stage_info.x6D4.wrapping_sub(1);
    if stage_info.x6D4 == 0 {
        stage_info.flags |= 0x20;
    }
}

pub unsafe extern "C" fn Ground_801C4368(slope: *mut f32, intercept: *mut f32) {
    melee_core::ct!("Ground_801C4368");
    *slope = stage_info.x724;
    *intercept = stage_info.x724 - stage_info.x728;
}

pub unsafe extern "C" fn Ground_801C438C(val: f32) {
    melee_core::ct!("Ground_801C438C");
    stage_info.x728 = stage_info.x724;
    stage_info.x724 = val;
}

pub unsafe extern "C" fn Ground_801C43A4(arg0: *mut c_void) {
    melee_core::ct!("Ground_801C43A4");
    ext::grZebes_801DA3F4(arg0 as *mut Vec);
}

pub unsafe extern "C" fn Ground_801C43C4(arg0: *mut c_void) -> i32 {
    melee_core::ct!("Ground_801C43C4");
    let tmp = (*ext::grDatFiles_GetArchive()).unk4;
    let mut phi_r4 = (*tmp).unk20;
    let max = (*tmp).unk24;
    if !arg0.is_null() && max != 0 {
        let mut i = 0;
        while i != max {
            if (*phi_r4).unk0 as *mut c_void == arg0 {
                return if (*phi_r4).flag() != 0 { 1 } else { 0 };
            }
            i += 1;
            phi_r4 = phi_r4.add(1);
        }
        panic!("ground.c:3652 HSD_ASSERT(3652, 0)");
    }
    0
}

pub unsafe extern "C" fn Ground_801C445C(lobj: *mut HSD_LObj) {
    melee_core::ct!("Ground_801C445C");
    let mut pos0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut pos1 = pos0;
    let mut pos2 = pos0;
    let mut pos3 = pos0;
    if lobj.is_null() {
        return;
    }
    let mut cur = lobj;
    while !cur.is_null() {
        let mut wobj = ext::HSD_LObjGetPositionWObj(lobj);
        if !wobj.is_null() {
            ext::HSD_WObjGetPosition(wobj, &mut pos0);
        }
        wobj = ext::HSD_LObjGetInterestWObj(lobj);
        if !wobj.is_null() {
            ext::HSD_WObjGetPosition(wobj, &mut pos2);
        }
        ext::HSD_LObjAnim(cur);
        wobj = ext::HSD_LObjGetPositionWObj(lobj);
        if !wobj.is_null() {
            ext::HSD_WObjGetPosition(wobj, &mut pos1);
            if pos0.x != pos1.x || pos0.y != pos1.y || pos0.z != pos1.z {
                let pos_mul = if !stage_info.param.is_null() { (*stage_info.param).y } else { 1.0f32 };
                pos1.x *= pos_mul;
                pos1.y *= pos_mul;
                pos1.z *= pos_mul;
                ext::HSD_WObjSetPosition(wobj, &mut pos1);
            }
        }
        wobj = ext::HSD_LObjGetInterestWObj(lobj);
        if !wobj.is_null() {
            ext::HSD_WObjGetPosition(wobj, &mut pos3);
            if pos2.x != pos3.x || pos2.y != pos3.y || pos2.z != pos3.z {
                let pos_mul = if !stage_info.param.is_null() { (*stage_info.param).y } else { 1.0f32 };
                pos3.x *= pos_mul;
                pos3.y *= pos_mul;
                pos3.z *= pos_mul;
                ext::HSD_WObjSetPosition(wobj, &mut pos3);
            }
        }
        cur = (*cur).next;
    }
}

pub unsafe extern "C" fn Ground_801C461C(gobj: *mut HSD_GObj) {
    melee_core::ct!("Ground_801C461C");
    Ground_801C445C((*gobj).hsd_obj as *mut HSD_LObj);
}

pub unsafe extern "C" fn Ground_801C4640(gobj: *mut HSD_GObj, _unused: i32) {
    melee_core::ct!("Ground_801C4640");
    ext::HSD_LObj_803668EC((*gobj).hsd_obj as *mut HSD_LObj);
    ext::HSD_LObjSetupInit(ext::HSD_CObjGetCurrent());
}

/// `Ground_803E065C` (3E065C)
pub static mut Ground_803E065C: [HSD_LightAnim; 1] = unsafe { core::mem::zeroed() };
/// `Ground_803E066C` (3E066C)
pub static mut Ground_803E066C: HSD_WObjDesc = HSD_WObjDesc {
    class_name: null_mut(),
    pos: Vec { x: 0.57, y: 0.57, z: 0.57 },
    robjdesc: null_mut(),
};
/// `Ground_804D4500` (4D4500)
pub static mut Ground_804D4500: [*mut HSD_LightAnim; 2] = unsafe { [&raw mut Ground_803E065C as *mut HSD_LightAnim, null_mut()] };
/// `Ground_804D4508` (4D4508)
pub static mut Ground_804D4508: f32 = 16.0;
/// `Ground_803E0680` (3E0680)
pub static mut Ground_803E0680: HSD_LightDesc = unsafe {
    let mut d: HSD_LightDesc = core::mem::zeroed();
    d.flags = (1 << 0) | (1 << 2) | (1 << 3);
    d.attnflags = 0;
    d.color = _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };
    d.position = &raw mut Ground_803E066C;
    d.u.p = &raw mut Ground_804D4508 as *mut c_void;
    d
};
/// `Ground_803E069C` (3E069C)
pub static mut Ground_803E069C: [HSD_LightAnim; 1] = unsafe { core::mem::zeroed() };
/// `Ground_803E06AC` (3E06AC)
pub static mut Ground_803E06AC: HSD_LightDesc = unsafe {
    let mut d: HSD_LightDesc = core::mem::zeroed();
    d.flags = 1 << 2;
    d.attnflags = 0;
    d.color = _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };
    d
};
/// `Ground_804D450C` (4D450C)
pub static mut Ground_804D450C: LightList = unsafe {
    LightList { desc: &raw mut Ground_803E0680, anims: &raw mut Ground_804D4500 as *mut *mut HSD_LightAnim }
};
/// `Ground_804D4514` (4D4514)
pub static mut Ground_804D4514: [*mut HSD_LightAnim; 2] = unsafe { [&raw mut Ground_803E069C as *mut HSD_LightAnim, null_mut()] };
/// `Ground_804D451C` (4D451C)
pub static mut Ground_804D451C: LightList = unsafe {
    LightList { desc: &raw mut Ground_803E06AC, anims: &raw mut Ground_804D4514 as *mut *mut HSD_LightAnim }
};
/// `Ground_803E06C8` (3E06C8)
pub static mut Ground_803E06C8: [*mut LightList; 3] = unsafe { [&raw mut Ground_804D451C, &raw mut Ground_804D450C, null_mut()] };

pub unsafe extern "C" fn Ground_801C466C() {
    melee_core::ct!("Ground_801C466C");
    let mut callbacks: *mut StageCallbacks;
    let mut archive: *mut UnkArchiveStruct;
    let mut selected: *mut *mut LightList;

    archive = ext::grDatFiles_GetArchive();
    callbacks = Ground_MexStageCallbacks(stage_info.grkind as i32);
    // a loader stub or damaged stage file: no model table
    let mut count = if !archive.is_null() && !(*archive).unk4.is_null() { (*(*archive).unk4).unkC } else { 0 };
    if callbacks.is_null() {
        count = 0;
    }
    archive = ext::grDatFiles_GetArchive();
    let _ = ext::Mex_GrTrace();
    let mut i = 0;
    selected = null_mut();
    let mut found = false;
    while i < count {
        if (*callbacks).anon_10.anon_0.flags_b0() == 1 {
            archive = ext::grDatFiles_801C6330(i);
            if archive.is_null() {
                callbacks = callbacks.add(1);
                i += 1;
                continue;
            }
            selected = Ground_801C20E0(archive, (*(*(*archive).unk4).unk8.offset(i as isize)).x18);
            let _ = ext::Mex_GrTrace();
            found = true;
            break;
        }
        callbacks = callbacks.add(1);
        i += 1;
    }
    if !found {
        selected = null_mut();
        let _ = ext::Mex_GrTrace();
    }
    // light_selected:
    let lights: *mut *mut LightList = if selected.is_null() { &raw mut Ground_803E06C8 as *mut *mut LightList } else { selected };
    if ext::Mex_GrTrace() != 0 {
        // OSReport("grtrace: lightset[%d]=%p\n") for each entry up to the first NULL: no calls
    }
    let temp_r3 = ext::GObj_Create(0xD, 3, 0);
    if temp_r3.is_null() {
        // OSReport("couldn t get gobj"); while (true) {}
        panic!("ground.c:3759 couldn t get gobj");
    }
    let temp_r3_2 = ext::lb_80011AC4(lights);
    if temp_r3_2.is_null() {
        panic!("ground.c:3761 couldn t get lobj");
    }
    ext::HSD_GObjObject_80390A70(temp_r3, ext::HSD_GObj_LightKind as u8, temp_r3_2 as *mut c_void);
    ext::GObj_SetupGXLink(temp_r3, Some(Ground_801C4640), 0, 0);
    let mut var_r27 = temp_r3_2;
    let var_f31 = if !stage_info.param.is_null() { (*stage_info.param).y } else { 1.0f32 };
    let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    while !var_r27.is_null() {
        if ext::HSD_LObjGetPosition(var_r27, &mut sp10) != 0 {
            sp10.x *= var_f31;
            sp10.y *= var_f31;
            sp10.z *= var_f31;
            ext::HSD_LObjSetPosition(var_r27, &mut sp10);
        }
        if ext::HSD_LObjGetInterest(var_r27, &mut sp10) != 0 {
            sp10.x *= var_f31;
            sp10.y *= var_f31;
            sp10.z *= var_f31;
            ext::HSD_LObjSetInterest(var_r27, &mut sp10);
        }
        var_r27 = if var_r27.is_null() { null_mut() } else { (*var_r27).next };
    }
    ext::HSD_LObjReqAnimAll(temp_r3_2, 0.0f32);
    ext::HSD_ForeachAnim(
        temp_r3_2 as *mut c_void,
        LOBJ_TYPE as i32,
        ALL_TYPE_MASK as u32,
        ext::cb::HSD_AObjSetRate as *mut c_void,
        AOBJ_ARG_AF as i32,
        1.0f64,
        0,
    );
    let mut var_r27_2 = lights;
    let mut var_r26_2 = temp_r3_2;
    if !(**lights).anims.is_null() {
        let callback = ext::cb::HSD_AObjSetFlags as *mut c_void;
        while !var_r26_2.is_null() {
            if Ground_801C43C4(*(**var_r27_2).anims as *mut c_void) != 0 {
                if !(*var_r26_2).aobj.is_null() {
                    ext::HSD_AObjSetFlags((*var_r26_2).aobj, AOBJ_LOOP as u32);
                }
                if !(*var_r26_2).position.is_null() {
                    ext::HSD_ForeachAnim(
                        (*var_r26_2).position as *mut c_void,
                        WOBJ_TYPE as i32,
                        ALL_TYPE_MASK as u32,
                        callback,
                        AOBJ_ARG_AU as i32,
                        0.0f64,
                        AOBJ_LOOP as u32,
                    );
                }
                if !(*var_r26_2).interest.is_null() {
                    ext::HSD_ForeachAnim(
                        (*var_r26_2).interest as *mut c_void,
                        WOBJ_TYPE as i32,
                        ALL_TYPE_MASK as u32,
                        callback,
                        AOBJ_ARG_AU as i32,
                        0.0f64,
                        AOBJ_LOOP as u32,
                    );
                }
            }
            var_r26_2 = if var_r26_2.is_null() { null_mut() } else { (*var_r26_2).next };
            var_r27_2 = var_r27_2.add(1);
        }
    }
    ext::HSD_LObjAnimAll(temp_r3_2);
    ext::HSD_GObj_SetupProc(temp_r3, Some(Ground_801C461C), 0);
}

pub unsafe extern "C" fn Ground_801C498C() -> *mut HSD_GObj {
    melee_core::ct!("Ground_801C498C");
    let mut gobj = *ext::HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_LIGHT as isize);
    while !gobj.is_null() {
        if (*gobj).classifier as i32 == HSD_GOBJ_CLASS_GROUND {
            break;
        }
        gobj = (*gobj).next;
    }
    gobj
}

pub unsafe extern "C" fn Ground_801C49B4() -> *mut *mut LightList {
    melee_core::ct!("Ground_801C49B4");
    let archive = ext::grDatFiles_GetArchive();
    if !stage_info.map_plit.is_null() {
        return Ground_801C20E0(archive, stage_info.map_plit);
    }
    &raw mut Ground_803E06C8 as *mut *mut LightList
}

pub unsafe extern "C" fn Ground_GetYakumonoParam() -> *mut c_void {
    melee_core::ct!("Ground_GetYakumonoParam");
    // grtrace report: only the trace switch is a call
    let _ = ext::Mex_GrTrace();
    stage_info.yakumono_param
}

pub unsafe extern "C" fn removeStageGObj(gobj: *mut HSD_GObj) {
    melee_core::ct!("removeStageGObj");
    let obj = (*gobj).hsd_obj;
    for i in 0..4 {
        if stage_info.x694[i] == obj {
            stage_info.x694[i] = null_mut();
            break;
        }
    }
}

/// Stage destroy ground gobj
pub unsafe extern "C" fn Ground_801C4A08(gobj: *mut HSD_GObj) {
    melee_core::ct!("Ground_801C4A08");
    if gobj.is_null() {
        return;
    }
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if !gp.is_null() {
        let map_id = (*gp).map_id as i32;
        if let Some(f) = (*gp).x1C_callback {
            f(gobj);
        }
        if stage_info.map_gobjs[map_id as usize] == gobj {
            stage_info.map_gobjs[map_id as usize] = null_mut();
        }
        Ground_801C55AC(gp);
        if !(*gp).x18.is_null() {
            removeStageGObj((*gp).x18);
            ext::HSD_GObjFree((*gp).x18);
        }
        if !(*gobj).hsd_obj.is_null() && *Ground_804D6950.offset(map_id as isize) == 0 {
            *Ground_804D6950.offset(map_id as isize) = 1;
            Ground_801C3128(map_id, Some(ext::cb::mpLib_80057BC0));
        }
        let archive = ext::grDatFiles_801C6330((*gp).map_id as i32);
        if !archive.is_null() {
            Ground_801C36F4((*gp).map_id as i32, jobj, (*(*(*archive).unk4).unk8.offset(map_id as isize)).unk0 as *mut c_void);
        }
    }
    ext::HSD_GObjFree(gobj);
}

pub unsafe extern "C" fn Ground_801C4B50(spline: *mut HSD_Spline, arg1: *mut Vec3, result: *mut Vec3, arg8: f32) {
    melee_core::ct!("Ground_801C4B50");
    let mut vec0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut vec1 = vec0;
    let mut vec2 = vec0;
    const M_PI: f64 = 3.14159265358979323846;
    ext::splGetSplinePoint(arg1, spline, arg8);
    ext::lbShadow_8000E9F0(&mut vec0, spline, arg8);
    ext::lbVector_Normalize(&mut vec0);
    let mut y0 = vec0.y;
    if vec0.y < 0.0f32 {
        y0 = -vec0.y;
    }
    // `0.9L`: long double is a double on the GameCube
    if y0 as f64 > 0.9f64 {
        vec2.x = 0.0;
        vec2.y = 0.0;
        vec2.z = 1.0;
    } else {
        vec2.x = 0.0;
        vec2.z = 0.0;
        vec2.y = 1.0;
    }
    ext::lbVector_CrossprodNormalized(&mut vec0, &mut vec2, &mut vec1);
    ext::lbVector_CrossprodNormalized(&mut vec1, &mut vec0, &mut vec2);
    let result_y = ext::asinf(vec1.z);
    let mut z1 = vec1.z;
    if vec1.z < 0.0f32 {
        z1 = -vec1.z;
    }
    let result_x: f32;
    let result_z: f32;
    if z1 >= 0.99999f32 {
        let mut phi_f31 = ext::asinf(-vec0.y);
        if vec0.x * melee_runtime::gekko::cosf(phi_f31) * melee_runtime::gekko::sinf(result_y) < 0.0f32 {
            phi_f31 = (M_PI - phi_f31 as f64) as f32;
        }
        result_x = phi_f31;
        result_z = 0.0;
    } else {
        let mut rx = ext::asinf(vec2.z / melee_runtime::gekko::cosf(result_y));
        if vec0.z * melee_runtime::gekko::cosf(rx) * melee_runtime::gekko::cosf(result_y) < 0.0f32 {
            rx = (M_PI - rx as f64) as f32;
        }
        let mut rz = ext::asinf(-vec1.y / melee_runtime::gekko::cosf(result_y));
        if -vec1.x * melee_runtime::gekko::cosf(result_y) * melee_runtime::gekko::cosf(rz) < 0.0f32 {
            rz = (M_PI - rz as f64) as f32;
        }
        result_x = rx;
        result_z = rz;
    }
    (*result).x = result_x;
    (*result).y = result_y;
    (*result).z = result_z;
}

pub unsafe extern "C" fn Ground_801C4D70(arg0: *mut HSD_GObj, arg1: *mut Vec3, arg8: f32) -> i32 {
    melee_core::ct!("Ground_801C4D70");
    stage_info.x72C = arg0;
    stage_info.x730 = *arg1;
    stage_info.x73C = arg8;
    1
}

pub unsafe extern "C" fn Ground_801C4DA0(arg0: *mut Vec3, arg1: *mut f32) -> i32 {
    melee_core::ct!("Ground_801C4DA0");
    *arg0 = stage_info.x730;
    *arg1 = stage_info.x73C;
    1
}

pub unsafe extern "C" fn Ground_801C4DD0() -> i32 {
    melee_core::ct!("Ground_801C4DD0");
    let stkind = stage_info.grkind;
    if stkind == Gr_Kind_Kongo {
        ext::grKongo_801D8270(stage_info.x72C);
    } else if stkind == Gr_Kind_OldKongo {
        ext::grOldKongo_802105AC(stage_info.x72C);
    }
    1
}

pub unsafe extern "C" fn Ground_801C4E20() -> i32 {
    melee_core::ct!("Ground_801C4E20");
    let stkind = stage_info.grkind;
    if stkind == Gr_Kind_Kongo {
        ext::grKongo_801D828C(stage_info.x72C);
    } else if stkind == Gr_Kind_OldKongo {
        ext::grOldKongo_802105C8(stage_info.x72C);
    }
    1
}

pub unsafe extern "C" fn Ground_801C4E70(arg0: *mut HSD_JObj, arg1: *mut HSD_JObj, arg2: *mut HSD_JObj, arg3: *mut HSD_JObj, arg4: *mut HSD_JObj, arg5: *mut HSD_JObj) {
    melee_core::ct!("Ground_801C4E70");
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    stage_info.unk8C.set_b3(1);
    ext::lb_8000B1CC(arg0, null_mut(), &mut vec);
    stage_info.x130 = vec;
    ext::lb_8000B1CC(arg1, null_mut(), &mut vec);
    stage_info.x13C = vec;
    ext::lb_8000B1CC(arg2, null_mut(), &mut vec);
    stage_info.x148 = vec;
    ext::lb_8000B1CC(arg3, null_mut(), &mut vec);
    stage_info.x154 = vec;
    ext::lb_8000B1CC(arg4, null_mut(), &mut vec);
    stage_info.x160 = vec;
    ext::lb_8000B1CC(arg5, null_mut(), &mut vec);
    stage_info.x16C = vec;
}

#[inline(always)]
fn abs_f(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe extern "C" fn Ground_801C4FAC(cobj: *mut HSD_CObj) {
    melee_core::ct!("Ground_801C4FAC");
    let mut sp74 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp68;
    let mut sp5C;
    let mut sp50;
    let mut sp44;
    let mut sp38 = sp74;
    let mut sp2C = sp74;
    let mut sp20 = sp74;

    if stage_info.unk8C.b3() != 0 {
        ext::HSD_CObjGetEyeVector(cobj, &mut sp74);
        sp68 = stage_info.x130;
        sp5C = stage_info.x13C;
        if sp74.x < 0.0 {
            sp50 = stage_info.x148;
            sp44 = stage_info.x154;
        } else {
            sp50 = stage_info.x160;
            sp44 = stage_info.x16C;
        }
        if sp74.z < 0.0 {
            // x*x + z*z: the LHS multiply is fused
            // sp74 lives in memory (HSD_CObjGetEyeVector(&sp74)): squares not fused on the console
            let xz_inv_len = 1.0f32 / sqrtf(sp74.x * sp74.x + sp74.z * sp74.z);
            let xz_x_weight = xz_inv_len * abs_f(sp74.x);
            let xz_z_weight = xz_inv_len * abs_f(sp74.z);
            sp50.x *= xz_x_weight;
            sp50.y *= xz_x_weight;
            sp50.z *= xz_x_weight;

            sp44.x *= xz_x_weight;
            sp44.y *= xz_x_weight;
            sp44.z *= xz_x_weight;

            sp68.x *= xz_z_weight;
            sp68.y *= xz_z_weight;
            sp68.z *= xz_z_weight;

            sp5C.x *= xz_z_weight;
            sp5C.y *= xz_z_weight;
            sp5C.z *= xz_z_weight;
            ext::PSVECAdd(&mut sp68, &mut sp50, &mut sp38);
            ext::PSVECAdd(&mut sp5C, &mut sp44, &mut sp2C);
        } else {
            sp38 = sp50;
            sp2C = sp44;
        }
        ext::HSD_CObjGetEyePosition(cobj, &mut sp20);
        if !stage_info.x12C.is_null() {
            let fog = (*stage_info.x12C).hsd_obj as *mut HSD_Fog;
            if !fog.is_null() {
                let mut dx = sp38.x - sp20.x;
                let dy = sp38.y - sp20.y;
                let mut dz = sp38.z;
                dz -= sp20.z;
                let mut dx2 = dx * dx;
                let mut dy2 = dy * dy;
                let mut dz2 = dz * dz;
                let mut phi_f31 = sqrtf(dx2 + dy2 + dz2);
                dx = sp2C.x - sp20.x;
                dz = sp2C.z;
                dz -= sp20.z;
                dx2 = dx * dx;
                dy2 = (sp2C.y - sp20.y) * (sp2C.y - sp20.y);
                dz2 = dz * dz;
                let mut phi_f30 = sqrtf(dx2 + dy2 + dz2);
                if phi_f30 < 10.0 {
                    phi_f30 = 10.0;
                }
                if phi_f31 < 5.0 {
                    phi_f31 = 5.0;
                }
                if phi_f31 > phi_f30 {
                    phi_f30 = 1.0f32 + phi_f31;
                }
                (*fog).start = phi_f31;
                (*fog).end = phi_f30;
            }
        }
    }
}

pub unsafe extern "C" fn Ground_801C53EC(arg0: u32) {
    melee_core::ct!("Ground_801C53EC");
    ext::lbAudioAx_800237A8(arg0 as i32, 0x7F, 0x40);
}

pub unsafe extern "C" fn Ground_801C5414(arg0: i32, arg1: i32) {
    melee_core::ct!("Ground_801C5414");
    ext::lbAudioAx_80023870(arg0, 0x7F, 0x40, arg1);
}

pub unsafe extern "C" fn Ground_801C5440(gp: *mut Ground, i: i32, arg2: u32) {
    melee_core::ct!("Ground_801C5440");
    if i < 0 || i >= 8 {
        return;
    }
    if gp.is_null() {
        return;
    }
    if arg2 == 540000 {
        return;
    }
    if arg2 != 540001 {
        if (*gp).x20[i as usize] != -1 {
            let tmp_gp = gp;
            ext::lbAudioAx_800236B8((*tmp_gp).x20[i as usize]);
        }
        (*gp).x20[i as usize] = ext::lbAudioAx_800237A8(arg2 as i32, 0x7F, 0x40);
    } else {
        Ground_801C5544(gp, i);
    }
}

pub unsafe extern "C" fn Ground_801C54DC(gp: *mut Ground, i: i32) -> i32 {
    melee_core::ct!("Ground_801C54DC");
    if i < 0 || i >= 8 {
        return 0;
    }
    if !gp.is_null() && (*gp).x20[i as usize] != -1 && ext::lbAudioAx_80023710((*gp).x20[i as usize]) != 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn Ground_801C5544(gp: *mut Ground, i: i32) {
    melee_core::ct!("Ground_801C5544");
    if i < 0 || i >= 8 {
        return;
    }
    if gp.is_null() {
        return;
    }
    if (*gp).x20[i as usize] != -1 {
        let tmp_gp = gp;
        ext::lbAudioAx_800236B8((*tmp_gp).x20[i as usize]);
    }
    (*gp).x20[i as usize] = -1;
}

pub unsafe extern "C" fn Ground_801C55AC(gp: *mut Ground) {
    melee_core::ct!("Ground_801C55AC");
    if !gp.is_null() {
        for i in 0..8 {
            Ground_801C5544(gp, i);
        }
    }
}

pub unsafe extern "C" fn Ground_801C5630(gp: *mut Ground, i: i32, val: f32) {
    melee_core::ct!("Ground_801C5630");
    if i < 0 || i >= 8 {
        return;
    }
    if !gp.is_null() && (*gp).x20[i as usize] != -1 {
        ext::lbAudioAx_80024B58((*gp).x20[i as usize], f2i32((127.0f32 * val) as f64));
    }
}

pub unsafe extern "C" fn Ground_801C5694(gp: *mut Ground, i: i32, val: f32) {
    melee_core::ct!("Ground_801C5694");
    if i < 0 || i >= 8 {
        return;
    }
    if !gp.is_null() && (*gp).x20[i as usize] != -1 {
        ext::lbAudioAx_80024B1C((*gp).x20[i as usize], f2i32((63.5f32 * (val + 1.0f32)) as f64));
    }
}

pub unsafe extern "C" fn Ground_801C5700(i: i32) -> *mut DynamicsDesc {
    melee_core::ct!("Ground_801C5700");
    if let Some(f) = stage_info.on_touch_line {
        return f(i);
    }
    null_mut()
}

pub unsafe extern "C" fn Ground_801C5740(arg0: i32) {
    melee_core::ct!("Ground_801C5740");
    stage_info.x6D8 = arg0;
}

pub unsafe extern "C" fn Ground_801C5750() {
    melee_core::ct!("Ground_801C5750");
    stage_info.x6D8 = 0;
}

pub unsafe extern "C" fn Ground_801C5764() -> i32 {
    melee_core::ct!("Ground_801C5764");
    stage_info.x6D8
}

pub unsafe extern "C" fn Ground_801C5774() -> i32 {
    melee_core::ct!("Ground_801C5774");
    stage_info.x6DC as i32
}

pub unsafe extern "C" fn Ground_801C5784(arg0: i32) {
    melee_core::ct!("Ground_801C5784");
    stage_info.x740 = arg0;
}

pub unsafe extern "C" fn Ground_801C5794() -> i32 {
    melee_core::ct!("Ground_801C5794");
    stage_info.x740
}

pub unsafe extern "C" fn Ground_GetP1Fighter() -> *mut Fighter_GObj {
    melee_core::ct!("Ground_GetP1Fighter");
    ext::Player_GetEntity(0)
}

pub unsafe extern "C" fn Ground_GetP1Fighter2() -> *mut Fighter_GObj {
    melee_core::ct!("Ground_GetP1Fighter2");
    ext::Player_GetEntityAtIndex(0, 1)
}

/// `arg0` is unused by the implementation; the game's only caller materializes an explicit 0.
pub unsafe extern "C" fn Ground_801C57F0(_arg0: i32) -> f32 {
    melee_core::ct!("Ground_801C57F0");
    stage_info.x6E0
}

pub unsafe extern "C" fn Ground_EnableMatchCamera() {
    melee_core::ct!("Ground_EnableMatchCamera");
    if (*stage_info.param).x4C_fixed_cam != 0 {
        ext::Camera_SetModeToFixed();
    } else {
        ext::Camera_SetModeToStandard();
    }
}

pub unsafe extern "C" fn Ground_801C5840() -> i32 {
    melee_core::ct!("Ground_801C5840");
    let i = ext::HSD_Randi(1);
    stage_info.x6E4[i as usize]
}

pub unsafe extern "C" fn initSinglePlayerDisplay(stageinfo: *mut StageInfo) {
    melee_core::ct!("initSinglePlayerDisplay");
    let display_id = ext::tyDisplay_8031C2EC();
    ext::tyDisplay_8031C454(display_id);
    (*stageinfo).x6E4[0] = display_id;
}

pub unsafe extern "C" fn Ground_801C5878() {
    melee_core::ct!("Ground_801C5878");
    ext::tyDisplay_8031C2CC();
    if ext::gm_IsCurrently1PMode() != 0 {
        initSinglePlayerDisplay(&raw mut stage_info);
    } else {
        stage_info.x6E4[0] = -1;
    }
}

pub unsafe extern "C" fn Ground_801C58E0(arg0: i32, arg1: i32) -> *mut Item_GObj {
    melee_core::ct!("Ground_801C58E0");
    let tmp = arg0;
    let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    Ground_801C2D24(arg1, &mut sp10);
    let result = ext::it_802F2094(null_mut(), &mut sp10, tmp, 0);
    ext::Toy_80304A58(tmp);
    result
}

/// `{ u8 x0_pad[4]; struct { s16 a, b; }* unk4; s32 unk8; }`, a stage-DAT entry as the C lays it out.
#[repr(C)]
struct PairListEntry {
    x0_pad: [u8; 4],
    unk4: *mut [i16; 2],
    unk8: i32,
}

pub unsafe extern "C" fn Ground_801C5940() -> i32 {
    melee_core::ct!("Ground_801C5940");
    let mut vals = [0i32; 32];
    let vals_count = 32usize;
    let archive = ext::grDatFiles_GetArchive();
    let mut out_idx: usize = 0;
    if (*(*archive).unk4).unk4 == 0 {
        return -1;
    }
    let mut phi_r8 = (*(*archive).unk4).unk0 as *mut PairListEntry;
    let mut i = 0;
    while i < (*(*archive).unk4).unk4 {
        let max = (*phi_r8).unk8;
        let mut j = 0;
        while j < max {
            let val = (*(*phi_r8).unk4.offset(j as isize))[1] as i32;
            if val >= 220 && val < 252 && out_idx < vals_count {
                vals[out_idx] = val;
                out_idx += 1;
            }
            j += 1;
        }
        i += 1;
        phi_r8 = phi_r8.add(1);
    }
    if out_idx == 0 {
        return -1;
    }
    vals[randi(out_idx as i32) as usize]
}

pub unsafe extern "C" fn Ground_801C5A28() {
    melee_core::ct!("Ground_801C5A28");
    ext::Toy_803124BC();
    ext::Toy_8031234C(0);
    ext::Toy_80305918(0, 0, 1);
}

pub unsafe extern "C" fn Ground_801C5A60() {
    melee_core::ct!("Ground_801C5A60");
    ext::Toy_8031234C(1);
}

pub unsafe extern "C" fn Ground_801C5A84(arg0: i32) {
    melee_core::ct!("Ground_801C5A84");
    stage_info.x98 = arg0;
}

pub unsafe extern "C" fn Ground_801C5A94() -> i32 {
    melee_core::ct!("Ground_801C5A94");
    stage_info.x98
}

pub unsafe extern "C" fn Ground_801C5AA4(arg0: i32) {
    melee_core::ct!("Ground_801C5AA4");
    stage_info.unk8C.set_b1(arg0 as u8);
}

pub unsafe extern "C" fn Ground_801C5ABC() -> i32 {
    melee_core::ct!("Ground_801C5ABC");
    stage_info.unk8C.b1() as i32
}

pub unsafe extern "C" fn Ground_801C5AD0(i: i32) -> u32 {
    melee_core::ct!("Ground_801C5AD0");
    (*stage_datas[i as usize]).flags2
}

pub unsafe extern "C" fn Ground_801C5AEC(v: *mut Vec3, arg1: *mut Vec3, arg2: *mut Vec3, arg3: *mut Vec3) {
    melee_core::ct!("Ground_801C5AEC");
    ext::lbVector_EulerAnglesFromONB(v, arg1, arg2, arg3);
    if !(abs_f((*v).x) < 30000.0) {
        (*v).x = 0.0;
    }
    if !(abs_f((*v).y) < 30000.0) {
        (*v).y = 0.0;
    }
    if !(abs_f((*v).z) < 30000.0) {
        (*v).z = 0.0;
    }
}

// ---------------------------------------------------------------------------------------------
// ground_stage_slots.inc / ground_stage_slot_lighting.inc / ground_stage_slot_retail.inc

pub unsafe extern "C" fn Ground_StageSlotDataFile(kind: i32) -> *const i8 {
    melee_core::ct!("Ground_StageSlotDataFile");
    if kind < 0 || kind as u32 as usize >= STAGE_DATAS_LEN || stage_datas[kind as usize].is_null() {
        return core::ptr::null();
    }
    (*stage_datas[kind as usize]).data1
}

/// The audited FD group-0 callbacks' joint table (the C's `STAGE_SLOT_FIXTURE` build leaves it out).
pub unsafe extern "C" fn Ground_StageSlotJoints(kind: i32, count: *mut u32) -> *const GrJoint {
    melee_core::ct!("Ground_StageSlotJoints");
    *count = 0;
    if kind < 0 || kind as u32 as usize >= STAGE_DATAS_LEN || stage_datas[kind as usize].is_null() {
        return core::ptr::null();
    }
    *count = (*stage_datas[kind as usize]).joint_count as u32;
    (*stage_datas[kind as usize]).joints
}

pub unsafe extern "C" fn Ground_StageSlotFighterLightingClear() {
    melee_core::ct!("Ground_StageSlotFighterLightingClear");
    ext::HSD_LObjDeleteCurrentAll(null_mut());
    ext::HSD_FogSet(null_mut());
    ext::HSD_StateInvalidate(-1);
    if ext::HSD_GObjPLinkHead.is_null() {
        return;
    }
    let mut gobj = *ext::HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_LIGHT as isize);
    while !gobj.is_null() {
        let next = (*gobj).next;
        // Exact retail identity, not all lights on this link
        if (*gobj).classifier == 12 && (*gobj).obj_kind as i8 == ext::HSD_GObj_LightKind {
            ext::HSD_GObjFree(gobj);
        }
        gobj = next;
    }
}

pub unsafe extern "C" fn Ground_StageSlotFighterLightingRebuild() {
    melee_core::ct!("Ground_StageSlotFighterLightingRebuild");
    ext::ScriptGame_StageSlotRetailCaptureResume();
    ext::ftCo_8009F4A4();
    ext::ScriptGame_StageSlotRetailCaptureEnd();
    ext::HSD_StateInvalidate(-1);
    // OSReport("stage switch: destination fighter lighting rebuilt\n")
}

pub unsafe extern "C" fn Ground_StageSlotRetailInstall(pair: *mut StageIdPair, archive: *mut HSD_Archive) {
    melee_core::ct!("Ground_StageSlotRetailInstall");
    let stage = stage_datas[(*pair).grkind as usize];
    Ground_801BFFB0();
    stage_info.grkind = (*pair).grkind;
    ext::grDatFiles_StageSlotInstall(archive);
    Ground_801C28CC(stage_info_xA0(), (*pair).stkind);
    stage_info.on_touch_line = (*stage).on_touch_line;
    stage_info.on_check_shadow_render = (*stage).on_check_shadow_render;
    Ground_801C5878();
    Ground_801C0800(pair);
    Ground_StageSlotFighterLightingRebuild();
    Ground_OnLoad(pair);
    Ground_801C0FB8(pair);
}

// ---- folded from bgm.rs (consolidation steps 21/22) ----
/// `Ground_801C24F8(stkind, arg1, &bgm)`. `randi` is `HSD_Randi`.
pub fn ground_801c24f8(rows: &[StageParam_bgm], stkind: i32, mut arg1: u32, unlocks: &dyn BgmUnlocks, randi: &mut dyn FnMut(i32) -> i32) -> BgmPick {
    melee_core::ct!("ground_801c24f8");
    let mut bgm = BGM_UNDEFINED;
    let mut result = false;
    let mut alt_flag = None;
    for p in rows {
        if p.stkind != stkind {
            continue;
        }
        if arg1 & 4 != 0 {
            let forced = arg1 & 0x40 != 0;
            // `x16 > HSD_Randi(RANDI_MAX_bgm) || forced`: the draw happens even when forced.
            let roll = |randi: &mut dyn FnMut(i32) -> i32| p.x16 as i32 > randi(RANDI_MAX_bgm) || forced;
            let alt = match p.x14 {
                0 => Some(forced),
                1 => Some(randi(0xC) == 0 || forced),
                8 => Some(false),
                2 => Some(roll(randi)),
                3 => Some(unlocks.mars() && roll(randi)),
                4 => Some(unlocks.clink() && roll(randi)),
                5 => Some(unlocks.toy_11a() && roll(randi)),
                6 => Some(unlocks.all_characters() && roll(randi)),
                7 => Some(unlocks.gm_80164600() && roll(randi)),
                _ => None,
            };
            match alt {
                Some(true) => arg1 |= 2,
                Some(false) => arg1 |= 1,
                None => {}
            }
        }
        if arg1 & 0x10 != 0 {
            alt_flag = Some(false);
            if arg1 & 1 != 0 {
                bgm = p.x_c as i32;
            } else if arg1 & 2 != 0 {
                if p.x10 != BGM_UNDEFINED as u32 {
                    bgm = p.x10 as i32;
                    result = true;
                } else {
                    bgm = p.x_c as i32;
                }
            }
        } else if arg1 & 0x20 != 0 {
            if arg1 & 1 != 0 {
                bgm = p.x_c as i32;
                alt_flag = Some(false);
            } else if arg1 & 2 != 0 {
                if p.x10 != BGM_UNDEFINED as u32 {
                    bgm = p.x10 as i32;
                    alt_flag = Some(true);
                    result = true;
                } else {
                    bgm = p.x_c as i32;
                    alt_flag = Some(false);
                }
            }
        } else if arg1 & 1 != 0 {
            bgm = p.x4;
            alt_flag = Some(false);
        } else if arg1 & 2 != 0 {
            if p.x8 as u32 != BGM_UNDEFINED as u32 {
                bgm = p.x8;
                alt_flag = Some(true);
                result = true;
            } else {
                bgm = p.x4;
                alt_flag = Some(false);
            }
        }
        break;
    }
    assert!(bgm != BGM_UNDEFINED, "Ground_801C24F8: no music row for stkind {stkind:#x} (m-ex playlist fallback not ported)");
    if bgm == -2 {
        panic!("Ground_801C24F8: per-character track lbAudioAx_8002305C (TODO(group1 lbaudio))");
    }
    BgmPick { result, bgm, alt_flag }
}

// ---- folded from ground.rs (consolidation steps 21/22) ----
/// Ground_801C498C: the first gobj on the light plink list (`HSD_GObjPLinkHead[3]`, passed in)
/// whose classifier is HSD_GOBJ_CLASS_GROUND, or null.
///
/// # Safety
/// `head` is null or a well-formed `next`-linked list of live GObjs.
pub unsafe fn ground_801c498c(head: *mut HSD_GObj) -> *mut HSD_GObj {
    melee_core::ct!("ground_801c498c");
    let mut g = head;
    while !g.is_null() {
        if (*g).classifier as i32 == HSD_GOBJ_CLASS_GROUND {
            break;
        }
        g = (*g).next;
    }
    g
}

/// Ground_801C1D38 (static): call the Ground's `xC_callback` when set.
///
/// # Safety
/// `gobj` is a live Ground GObj (the C reads `user_data` unchecked).
pub unsafe fn ground_801c1d38(gobj: *mut HSD_GObj) {
    melee_core::ct!("ground_801c1d38");
    let gp = (*gobj).user_data as *mut Ground;
    if let Some(cb) = (*gp).xC_callback {
        cb(gobj);
    }
}

/// Ground_801C53EC
pub fn ground_801c53ec(h: &mut dyn GroundHooks, sfx: u32) { melee_core::ct!("ground_801c53ec"); h.lbaudioax_800237a8(sfx as i32, 0x7F, 0x40); }

/// Ground_801C5414
pub fn ground_801c5414(h: &mut dyn GroundHooks, a: i32, b: i32) { melee_core::ct!("ground_801c5414"); h.lbaudioax_80023870(a, 0x7F, 0x40, b); }

/// Ground_801C5A28
pub fn ground_801c5a28(h: &mut dyn GroundHooks) {
    melee_core::ct!("ground_801c5a28");
    h.toy_803124bc();
    h.toy_8031234c(0);
    h.toy_80305918(0, 0, 1);
}

/// Ground_801C5A60
pub fn ground_801c5a60(h: &mut dyn GroundHooks) { melee_core::ct!("ground_801c5a60"); h.toy_8031234c(1); }

/// Ground_801C5440
pub fn ground_801c5440(h: &mut dyn GroundHooks, gp: Option<&mut Ground>, i: i32, sfx: u32) {
    melee_core::ct!("ground_801c5440");
    if !(0..8).contains(&i) {
        return;
    }
    let Some(gp) = gp else { return };
    if sfx == 540000 {
        return;
    }
    if sfx != 540001 {
        if gp.x20[i as usize] != -1 {
            h.lbaudioax_800236b8(gp.x20[i as usize]);
        }
        gp.x20[i as usize] = h.lbaudioax_800237a8(sfx as i32, 0x7F, 0x40);
    } else {
        ground_801c5544(h, Some(gp), i);
    }
}

/// Ground_801C54DC
pub fn ground_801c54dc(h: &mut dyn GroundHooks, gp: Option<&Ground>, i: i32) -> bool {
    melee_core::ct!("ground_801c54dc");
    if !(0..8).contains(&i) {
        return false;
    }
    matches!(gp, Some(g) if g.x20[i as usize] != -1 && h.lbaudioax_80023710(g.x20[i as usize]))
}

/// Ground_801C5544
pub fn ground_801c5544(h: &mut dyn GroundHooks, gp: Option<&mut Ground>, i: i32) {
    melee_core::ct!("ground_801c5544");
    if !(0..8).contains(&i) {
        return;
    }
    let Some(gp) = gp else { return };
    if gp.x20[i as usize] != -1 {
        h.lbaudioax_800236b8(gp.x20[i as usize]);
    }
    gp.x20[i as usize] = -1;
}

/// Ground_801C55AC (static)
pub fn ground_801c55ac(h: &mut dyn GroundHooks, gp: Option<&mut Ground>) {
    melee_core::ct!("ground_801c55ac");
    if let Some(gp) = gp {
        for i in 0..8 {
            ground_801c5544(h, Some(gp), i);
        }
    }
}

/// Ground_801C5630
pub fn ground_801c5630(h: &mut dyn GroundHooks, gp: Option<&Ground>, i: i32, val: f32) {
    melee_core::ct!("ground_801c5630");
    if !(0..8).contains(&i) {
        return;
    }
    if let Some(g) = gp {
        if g.x20[i as usize] != -1 {
            h.lbaudioax_80024b58(g.x20[i as usize], ground_5630_volume(val));
        }
    }
}

/// Ground_801C5694
pub fn ground_801c5694(h: &mut dyn GroundHooks, gp: Option<&Ground>, i: i32, val: f32) {
    melee_core::ct!("ground_801c5694");
    if !(0..8).contains(&i) {
        return;
    }
    if let Some(g) = gp {
        if g.x20[i as usize] != -1 {
            h.lbaudioax_80024b1c(g.x20[i as usize], ground_5694_pan(val));
        }
    }
}

/// Ground_801C4B50: returns (*arg1 = spline point, *result = angles).
pub fn ground_801c4b50(h: &mut dyn GroundHooks, spline: *mut HSD_Spline, t: f32) -> (Vec3_ground, Vec3_ground) {
    melee_core::ct!("ground_801c4b50");
    let p = h.spl_get_spline_point(spline, t);
    let tangent = h.lbshadow_8000e9f0(spline, t);
    (p, ground_4b50_angles(tangent))
}

/// Ground_801C5AEC
pub fn ground_801c5aec(a: &Vec3_ground, b: &Vec3_ground, c: &Vec3_ground) -> Vec3_ground {
    melee_core::ct!("ground_801c5aec");
    let mut v = euler_angles_from_onb(a, b, c);
    let ok = |x: f32| (if x < 0.0 { -x } else { x }) < 30000.0;
    if !ok(v.x) {
        v.x = 0.0;
    }
    if !ok(v.y) {
        v.y = 0.0;
    }
    if !ok(v.z) {
        v.z = 0.0;
    }
    v
}
