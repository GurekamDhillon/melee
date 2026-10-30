//! Port of src/melee/cm/camera.c: the game camera (subjects, bounds, follow / zoom / quake, the
//! per-mode update callbacks, the pause and training cameras, the render pass, the TARGET_PC script
//! camera and the projection helpers). Generated from the C by `gen/mtu/c2rs.py` (libclang AST:
//! every implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds`,
//! checked against `tools/rust-port/fma_sites.py`); `gen/mtu/cm/hand/` holds the functions the
//! tool cannot structure. Callees of other TUs are `ext` hooks named like the C (unset: panic
//! "TODO", or the native port `gen/rewire/` wired). `bool` is MSL `int`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

/// MSL `__fpclassifyf` (math.h; FP_NAN 1, FP_INFINITE 2, FP_ZERO 3, FP_NORMAL 4, FP_SUBNORMAL 5)
fn __fpclassifyf(x: f32) -> i32 {
    let b = x.to_bits() as i32;
    match b & 0x7F800000 {
        0x7F800000 => if b & 0x007FFFFF != 0 { 1 } else { 2 },
        0 => if b & 0x007FFFFF != 0 { 5 } else { 3 },
        _ => 4,
    }
}

/// `lbVector_Add` / `lbVector_Diff` (lb/lbvector.c; ported in melee_runtime, checked against the C)
use melee_runtime::tu::lbvector::{lbVector_Add, lbVector_Diff};

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { panic!(concat!("TODO: ", stringify!($name), " is not ported")) };
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
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _); } };
    fn GenoFly_Any() -> i32;
    fn GenoFly_CameraSubject(a0: *mut c_void) -> i32;
    fn Ground_801C4368(a0: *mut f32, a1: *mut f32);
    fn Ground_801C4FAC(a0: *mut HSD_CObj);
    fn Ground_GetP1Fighter() -> *mut HSD_GObj = || unsafe { crate::tu::ground::Ground_GetP1Fighter() as _ };
    fn HSD_CObjEndCurrent() = || { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjEndCurrent(); } };
    fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32) = |a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjEraseScreen(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_CObjGetAspect(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetAuthoredAspect(a0: *mut HSD_CObj) -> f32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj_world::HSD_CObjGetAuthoredAspect(a0 as _) as _ };
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj = || unsafe { melee_runtime::tu::cobj::HSD_CObjGetCurrent() as _ };
    fn HSD_CObjGetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjGetEyePosition(a0 as _, a1 as _); } };
    fn HSD_CObjGetEyeVector(a0: *mut HSD_CObj, a1: *mut Vec) -> i32 = |a0: *mut HSD_CObj, a1: *mut Vec| unsafe { melee_runtime::tu::cobj_world::HSD_CObjGetEyeVector(a0 as _, a1 as _) as _ };
    fn HSD_CObjGetFar(a0: *mut HSD_CObj) -> f32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj::HSD_CObjGetFar(a0 as _) as _ };
    fn HSD_CObjGetFov(a0: *mut HSD_CObj) -> f32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj_world::HSD_CObjGetFov(a0 as _) as _ };
    fn HSD_CObjGetInterest(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjGetInterest(a0 as _, a1 as _); } };
    fn HSD_CObjGetScissor(a0: *mut HSD_CObj, a1: *mut _Scissor) = |a0: *mut HSD_CObj, a1: *mut _Scissor| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjGetScissor(a0 as _, a1 as _); } };
    fn HSD_CObjGetUpVector(a0: *mut HSD_CObj, a1: *mut Vec) -> i32;
    fn HSD_CObjGetViewingMtxPtr(a0: *mut HSD_CObj) -> *mut [f32; 4];
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0 as _) as _ };
    fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjSetEyePosition(a0 as _, a1 as _); } };
    fn HSD_CObjSetFar(a0: *mut HSD_CObj, a1: f32) = |a0: *mut HSD_CObj, a1: f32| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetFar(a0 as _, a1 as _); } };
    fn HSD_CObjSetFov(a0: *mut HSD_CObj, a1: f32);
    fn HSD_CObjSetInterest(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjSetInterest(a0 as _, a1 as _); } };
    fn HSD_CObjSetMtxDirty(a0: *mut HSD_CObj);
    fn HSD_CObjSetNear(a0: *mut HSD_CObj, a1: f32) = |a0: *mut HSD_CObj, a1: f32| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetNear(a0 as _, a1 as _); } };
    fn HSD_CObjSetRoll(a0: *mut HSD_CObj, a1: f32);
    fn HSD_CObjSetUpVector(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjSetupViewingMtx(a0: *mut HSD_CObj);
    fn HSD_CObjUpdateView(a0: *mut HSD_CObj);
    fn HSD_FogSet(a0: *mut HSD_Fog);
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32) = |a0: *mut HSD_GObj, a1: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObj_80390ED0(a0 as _, a1 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_LObjDeleteCurrentAll(a0: *mut HSD_LObj) = |a0: *mut HSD_LObj| { unsafe { let _ = melee_runtime::tu::lobj::HSD_LObjDeleteCurrentAll(a0 as _); } };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_Randf() -> f32 = || melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand());
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    fn Mex_Enabled(a0: *const i8) -> i32;
    fn MotionWorldDraw(a0: i32);
    fn PSVECCrossProduct(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) = |a0: *mut Vec, a1: *mut Vec, a2: *mut Vec| { unsafe { let _ = melee_runtime::tu::mtx_pc::raw::PSVECCrossProduct(a0 as _, a1 as _, a2 as _); } };
    fn Player_GetEntity(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::pl::Player_GetEntity(a0 as _) as _ };
    fn Player_GetPlayerCharacter(a0: i32) -> CharacterKind = |a0: i32| unsafe { crate::pl::Player_GetPlayerCharacter(a0 as _) as _ };
    fn Player_GetPlayerId(a0: i32) -> i32 = |a0: i32| unsafe { crate::pl::Player_GetPlayerId(a0 as _) as _ };
    fn ScriptGame_CameraFollowTarget(a0: i32, a1: i32, a2: *mut Vec) -> i32;
    fn ScriptGame_CameraFov(a0: f32) -> f32;
    fn ScriptGame_MapWide() -> i32;
    fn ScriptGame_WbDraw();
    fn Shader_PostDraw(a0: i32);
    fn Stage_80224CAC(a0: *mut Vec);
    fn Stage_GetBlastZoneBottomOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneBottomOffset() as _ };
    fn Stage_GetBlastZoneLeftOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneLeftOffset() as _ };
    fn Stage_GetBlastZoneRightOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneRightOffset() as _ };
    fn Stage_GetBlastZoneTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneTopOffset() as _ };
    fn Stage_GetCamAngleRadiansDown() -> f32;
    fn Stage_GetCamAngleRadiansLeft() -> f32;
    fn Stage_GetCamAngleRadiansRight() -> f32;
    fn Stage_GetCamAngleRadiansUp() -> f32;
    fn Stage_GetCamBoundsBottomOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsBottomOffset() as _ };
    fn Stage_GetCamBoundsLeftOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsLeftOffset() as _ };
    fn Stage_GetCamBoundsRightOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsRightOffset() as _ };
    fn Stage_GetCamBoundsTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsTopOffset() as _ };
    fn Stage_GetCamFixedFov() -> f32;
    fn Stage_GetCamInfoX20() -> f32;
    fn Stage_GetCamInfoX24() -> f32;
    fn Stage_GetCamMaxDepth() -> f32;
    fn Stage_GetCamPanAngleRadians() -> f32;
    fn Stage_GetCamTrackRatio() -> f32;
    fn Stage_GetCamTrackSmooth() -> f32;
    fn Stage_GetCamZoomRate() -> f32;
    fn Stage_GetPauseCamZPosInit() -> f32;
    fn Stage_GetPauseCamZPosMax() -> f32;
    fn Stage_GetPauseCamZPosMin() -> f32;
    fn Stage_SetVecToFixedCamPos(a0: *mut Vec);
    fn Stage_UnkSetVec3TCam_Offset(a0: *mut Vec) = |a0: *mut Vec| { unsafe { let _ = crate::tu::stage::Stage_UnkSetVec3TCam_Offset(a0 as _); } };
    fn Wb_RenderActive() -> i32;
    fn Wb_RenderCameraBits(a0: i32) -> i32;
    fn Wb_RenderDrawn();
    fn Wb_RenderOption(a0: i32, a1: i32) -> i32;
    fn atan2f(a0: f32, a1: f32) -> f32 = |a0: f32, a1: f32| unsafe { melee_runtime::lbtrigf::atan2f(a0 as _, a1 as _) as _ };
    fn atanf(a0: f32) -> f32 = |a0: f32| unsafe { melee_runtime::lbtrigf::atanf(a0 as _) as _ };
    fn diag_cam_translate(a0: i32, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32, a9: f32);
    fn diag_camera_consts(a0: f32, a1: f32, a2: f32, a3: f32);
    fn diag_camera_desc(a0: *const f32, a1: *const f32, a2: f32, a3: f32, a4: f32, a5: f32);
    fn diag_camera_subject(a0: f32, a1: f32, a2: f32, a3: f32, a4: f32);
    fn diag_camera_track(a0: *const f32, a1: *const f32);
    fn diag_game_camera(a0: *const f32, a1: *const f32, a2: *const f32);
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec) = |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_80086644(a0 as _, a1 as _); } };
    fn ftLib_80086B74(a0: *mut HSD_GObj) -> *mut CmSubject = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086B74(a0 as _) as _ };
    fn ftLib_8008701C(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_8008701C(a0 as _) as _ };
    fn ftLib_8008732C(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_8008732C(a0 as _) as _ };
    fn gm_GetButtonsPressed(a0: u8) -> u64;
    fn gm_GetButtonsTriggered(a0: u8) -> u64;
    fn gm_IsCurrently1PMode_inline() -> i32;
    fn grCastle_801D0FF0() -> f32;
    fn grCorneria_801E2FCC() -> f32;
    fn grGarden_80203624() -> f32;
    fn grHomeRun_8021EF10() -> f32;
    fn grKinokoRoute_802087B0() -> f32;
    fn grLib_801C9CEC(a0: CmQuakeKind) -> *mut HSD_GObj;
    fn grZebes_801DCCC8() -> f32;
    fn lbRefract_8002247C(a0: *mut HSD_CObj);
    fn lbRefract_80022560();
    fn lbShadow_8000F38C(a0: i32);
    fn lbVector_8000E838(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec) -> f32 = |a0: *mut Vec, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_8000E838(a0 as _, a1 as _, a2 as _, a3 as _) as _ };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 = |a0: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0 as _) as _ };
    fn lbVector_Rotate(a0: *mut Vec, a1: i32, a2: f32) = |a0: *mut Vec, a1: i32, a2: f32| { unsafe { let _ = melee_runtime::tu::lbvector::lbVector_Rotate(a0 as _, a1 as _, a2 as _); } };
    fn lbVector_RotateAboutUnitAxis(a0: *mut Vec, a1: *mut Vec, a2: f32) = |a0: *mut Vec, a1: *mut Vec, a2: f32| { unsafe { let _ = melee_runtime::tu::lbvector::lbVector_RotateAboutUnitAxis(a0 as _, a1 as _, a2 as _); } };
    fn lbVector_Sub(a0: *mut Vec, a1: *mut Vec) -> *mut Vec = |a0: *mut Vec, a1: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_Sub(a0 as _, a1 as _) as _ };
    fn lbVector_WorldToScreen(a0: *mut HSD_CObj, a1: *const Vec, a2: *mut Vec, a3: i32) -> *mut Vec = |a0: *mut HSD_CObj, a1: *const Vec, a2: *mut Vec, a3: i32| unsafe { melee_runtime::tu::lbvector::lbVector_WorldToScreen(a0 as _, a1 as _, a2 as _, a3 as _) as _ };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj = |a0: *mut HSD_CameraDescPerspective| unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0 as _) as _ };
    fn mpLib_8005A2DC();
    fn mpLib_DrawSpecialPoints();
    fn mpLib_DrawZones();
    fn tanf(a0: f32) -> f32 = |a0: f32| unsafe { melee_runtime::gekko::tanf(a0 as _) as _ };
    fn OSReport(a0: &str) = |a0: &str| melee_runtime::tu::os_error::OSReport_str(a0);
    }
    pub static mut HSD_GObj_CameraKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
    pub static mut gm_CStickSmashTargetTest: i32 = unsafe { core::mem::zeroed() };
    pub static mut stage_info: StageInfo = unsafe { core::mem::zeroed() };
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CmScriptPose {
    pub eye: Vec3,
    pub interest: Vec3,
    pub fov: f32,
    pub roll: f32,
}

melee_core::gc_layout!(CmScriptPose, "CmScriptPose", size 0x20, align 4, [
    eye @ 0x0: 12 "Vec3", interest @ 0xc: 12 "Vec3", fov @ 0x18: 4 "f32", roll @ 0x1c: 4 "f32",
]);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CmScriptKey {
    pub pose: CmScriptPose,
    pub frame: i32,
}

melee_core::gc_layout!(CmScriptKey, "CmScriptKey", size 0x24, align 4, [
    pose @ 0x0: 32 "CmScriptPose", frame @ 0x20: 4 "int",
]);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cm_script_t {
    pub state: i32,
    pub lift_bounds: i32,
    pub elapsed: i32,
    pub duration: i32,
    pub ease: i32,
    pub path_count: i32,
    pub path_frame: i32,
    pub path_building: i32,
    pub completion: i32,
    pub completion_kind: i32,
    pub follow_kind: i32,
    pub follow_id: i32,
    pub shake_left: i32,
    pub shake_total: i32,
    pub shake_tick: i32,
    pub shake_intensity: f32,
    pub saved_roll: f32,
    pub saved_far: f32,
    pub saved_up_vector: i32,
    pub saved_up: Vec3,
    pub follow_point: Vec3,
    pub follow_offset: Vec3,
    pub follow_eye_delta: Vec3,
    pub normal: CameraTransformState,
    pub pose: CmScriptPose,
    pub from: CmScriptPose,
    pub to: CmScriptPose,
    pub path: [CmScriptKey; 64],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct cm_largemap_t {
    pub primary: *mut HSD_CObj,
    pub secondary: *mut HSD_CObj,
    pub primary_far: f32,
    pub secondary_far: f32,
}

pub static mut cm_script: cm_script_t = unsafe { core::mem::zeroed() };
pub static mut cm_largemap: cm_largemap_t = unsafe { core::mem::zeroed() };
pub static mut game_camera: Camera = unsafe { core::mem::zeroed() };
pub static mut cm_80453004: CameraDebugMode = unsafe { core::mem::zeroed() };
pub static mut cm_803BCB18: CameraModeCallbacks = CameraModeCallbacks { callback: [Some(Camera_8002B3D4), Some(Camera_8002CDDC), Some(Camera_8002D318), Some(Camera_8002D85C), Some(Camera_8002DDC4), Some(Camera_8002C908), Some(Camera_8002E490), None, None] };
pub static mut cm_803BCB3C: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0f32, y: 40.241425f32, z: 300.241f32 }, robjdesc: null_mut() };
pub static mut cm_803BCB50: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0f32, y: 1e+01f32, z: 0.0f32 }, robjdesc: null_mut() };
pub static mut cm_803BCB64: HSD_CameraDescPerspective = HSD_CameraDescPerspective { class_name: null_mut(), flags: 0_u16, projection_type: 1_u16, viewport: HSD_RectS16 { xmin: 0_i16, xmax: 640_i16, ymin: 0_i16, ymax: 480_i16 }, scissor: Scissor { left: 0_u16, right: 640_u16, top: 0_u16, bottom: 480_u16 }, eyepos: core::ptr::addr_of_mut!(cm_803BCB3C), interest: core::ptr::addr_of_mut!(cm_803BCB50), roll: 0.0f32, up_vector: null_mut(), nnear: 0.1f32, ffar: 16384.0f32, fov: 3e+01f32, aspect: 1.2173333f32 };
pub static mut cm_803BCB9C: [f32; 5] = [0.0f32, 1.5f32, 1.32f32, 1.16f32, 1.0f32];
pub static cm_WorldForward: Vec = Vec { x: 0.0f32, y: 0.0f32, z: -1.0f32 };
pub static cm_803B73C4: Vec = Vec { x: 0.0f32, ..unsafe { core::mem::zeroed() } };
pub static cm_WorldUp: Vec = Vec { x: 0.0f32, y: 1.0f32, z: 0.0f32 };
pub static cm_803B73DC: Vec = Vec { x: 0.0f32, y: 1.0f32, z: 0.0f32 };
pub static mut cm_804D6468: *mut CmSubject = null_mut();
pub static mut cm_804D6464: *mut HSD_CObj = null_mut();
pub static mut cm_804D6460: *mut CmSubject = null_mut();
pub static mut cm_804D645C: *mut CmSubject = null_mut();
pub static mut cm_804D6458: *mut CmSubject = null_mut();
pub static mut lbl_803BCC70: [i8; 22] = [114, 97, 116, 101, 62, 48, 46, 48, 70, 38, 38, 114, 97, 116, 101, 60, 61, 49, 46, 48, 70, 0];
pub static mut lbl_803BCC88: [i8; 20] = [115, 110, 97, 112, 115, 104, 111, 116, 33, 32, 112, 116, 114, 61, 37, 48, 56, 120, 10, 0];
pub static mut gap_07_803BCC9C_data: [i8; 4] = [0, 0, 0, 0];
pub static mut cm_803BCCA0: CameraUnkGlobals = CameraUnkGlobals { x0: 83.0f32, x4: 1e+03f32, x8: -3e+01f32, xC: 5.0f32, x10: -7.0f32, x14: 17.5f32, x18: -17.5f32, x1C: 0.0f32, x20: 0.0682f32, x24: 6e+01f32, x28: 1.2e+02f32, x2C: 0.05f32, x30: 0.1f32, x34: 1.2e+02f32, x38: 9e+02f32, x3C: 0.15f32, x40: 38.0f32, x44: 0.1f32, x48: 0.1f32, x4C: 0.001f32, x50: 0.1f32, x54: 1.0f32, x58: 1.0f32, x5C: 0.6f32, x60: 0.6f32, x64: 0.05f32, x68: 0.1f32, x6C: 29.0f32, x70: 0.1f32, x74: 0.1f32, x78: 0.1f32, x7C: 0.1f32, x80: 0.5f32, x84: 0.5f32, x88: 0.4f32, x8C: -11.0f32, x90: 4e+02f32, x94: 0.2f32, x98: 4.0f32, x9C: 0.05f32, xA0: 1.0f32, xA4: -7.0f32, xA8: 7.0f32, xAC: 0.5f32, xB0: 0.5f32, xB4: 0.004f32, xB8: 0.2f32, xBC: 0.025f32, xC0: 0.2f32, xC4: 0.003f32, xC8: 0.2f32, xCC: 0.025f32, xD0: 0.2f32, xD4: 0.02f32, xD8: 1.0f32, xDC: 0.14f32, xE0: 1.2e+03f32, xE4: -0.2f32, xE8: 1.2f32, xEC: 0.0f32 };

/// `Camera_LargeMapRestore`
pub unsafe fn Camera_LargeMapRestore() {
    if !cm_largemap.primary.is_null() {
        ext::HSD_CObjSetFar(cm_largemap.primary, cm_largemap.primary_far);
    }
    if !cm_largemap.secondary.is_null() {
        ext::HSD_CObjSetFar(cm_largemap.secondary, cm_largemap.secondary_far);
    }
    cm_largemap.primary = { cm_largemap.secondary = null_mut(); cm_largemap.secondary };
}

/// `cm_script_float`
pub unsafe fn cm_script_float(mut bits: i32) -> f32 {
    #[derive(Clone, Copy)] #[repr(C)] union Local5 { i: i32, f: f32 }
    let mut u: Local5 = unsafe { core::mem::zeroed() };
    u.i = bits;
    return u.f;
}

/// `cm_script_bits`
pub unsafe fn cm_script_bits(mut value: f32) -> i32 {
    #[derive(Clone, Copy)] #[repr(C)] union Local5 { i: i32, f: f32 }
    let mut u: Local5 = unsafe { core::mem::zeroed() };
    u.f = value;
    return u.i;
}

/// `cm_fly_left`
pub unsafe fn cm_fly_left() -> f32 {
    return if (ext::GenoFly_Any() != 0) || ((cm_script.state != 0) && (cm_script.lift_bounds != 0)) { -1e+05f32 } else { ext::Stage_GetCamBoundsLeftOffset() };
}

/// `cm_fly_right`
pub unsafe fn cm_fly_right() -> f32 {
    return if (ext::GenoFly_Any() != 0) || ((cm_script.state != 0) && (cm_script.lift_bounds != 0)) { 1e+05f32 } else { ext::Stage_GetCamBoundsRightOffset() };
}

/// `cm_fly_top`
pub unsafe fn cm_fly_top() -> f32 {
    return if (ext::GenoFly_Any() != 0) || ((cm_script.state != 0) && (cm_script.lift_bounds != 0)) { 1e+05f32 } else { ext::Stage_GetCamBoundsTopOffset() };
}

/// `cm_fly_bottom`
pub unsafe fn cm_fly_bottom() -> f32 {
    return if (ext::GenoFly_Any() != 0) || ((cm_script.state != 0) && (cm_script.lift_bounds != 0)) { -1e+05f32 } else { ext::Stage_GetCamBoundsBottomOffset() };
}

/// `camera_sdata2_order`
pub unsafe fn camera_sdata2_order() {
    -f32::MAX;
    ();
    ();
    ();
    -1.0f32;
    ();
    ();
    ();
    ();
    ();
    ();
    -0.5f32;
    f32::MAX;
    4e+01f32;
    8e+01f32;
    5e+03f32;
    4.92e+03f32;
    1e+01f32;
    3.9e+02f32;
    ();
    ();
    1e+03f32;
    ();
    ();
    ();
    -0.001f32;
    3e+01f32;
    ();
    ();
    ();
    ();
    -0.999f32;
    ();
    2e+01f32;
    ();
    ();
    ();
    2e+03f32;
    -2e+03f32;
    -1.0f64;
    ();
    ();
    ();
    ();
    ();
}

/// `vec_len`
pub unsafe fn vec_len(mut offset: *mut Vec) -> f32 {
    return melee_runtime::gekko::sqrtf(fmadds((*offset).z, (*offset).z, fmadds((*offset).x, (*offset).x, (*offset).y * (*offset).y)));
}

/// `Camera_Init`
pub unsafe fn Camera_Init(mut n_subjects: i32) {
    let mut cam_box: *mut CmSubject = null_mut();
    let mut interest_pos: *mut Vec = null_mut();
    let mut eye_pos: *mut Vec = null_mut();
    let mut i: i32 = 0;
    camera_sdata2_order();
    core::ptr::write_bytes(((core::ptr::addr_of_mut!(cm_script) as *mut cm_script_t) as *mut c_void) as *mut u8, 0, (2580_u64 as i32) as usize);
    interest_pos = core::ptr::addr_of_mut!((*cm_803BCB64.interest).pos) as *mut Vec;
    game_camera.transform.interest = *interest_pos;
    game_camera.transform.target_interest = *interest_pos;
    eye_pos = core::ptr::addr_of_mut!((*cm_803BCB64.eyepos).pos) as *mut Vec;
    ext::diag_camera_desc(eye_pos as *const f32, interest_pos as *const f32, cm_803BCB64.fov, cm_803BCB64.nnear, cm_803BCB64.ffar, cm_803BCB64.aspect);
    game_camera.transform.position = *eye_pos;
    game_camera.transform.target_position = *eye_pos;
    game_camera.transform.target_fov = cm_803BCB64.fov;
    game_camera.transform.fov = cm_803BCB64.fov;
    game_camera.transform_copy.interest = *interest_pos;
    game_camera.transform_copy.target_interest = *interest_pos;
    game_camera.transform_copy.position = *eye_pos;
    game_camera.transform_copy.target_position = *eye_pos;
    game_camera.transform_copy.target_fov = cm_803BCB64.fov;
    game_camera.transform_copy.fov = cm_803BCB64.fov;
    game_camera.background_color.b = 0 as u8;
    game_camera.background_color.g = 0 as u8;
    game_camera.background_color.r = 0 as u8;
    game_camera.nearz = 0.1f32;
    game_camera.farz = 16384.0f32;
    game_camera.mode = (CAMERA_STANDARD as i32) as u32;
    core::ptr::write_bytes(((game_camera.quake_frames_left.as_mut_ptr()) as *mut c_void) as *mut u8, 0, (((((core::mem::offset_of!(Camera, x2B0) as *mut f32)) as usize as u32) - (((core::mem::offset_of!(Camera, quake_frames_left) as *mut [i32; 5])) as usize as u32)) as i32) as usize);
    game_camera.quake_scale = 1.0f32;
    game_camera.x2BC = 1.0f32;
    game_camera.x2C0 = -1.0f32;
    game_camera.set_x398_b0((0 as u8) as u8);
    game_camera.set_x398_b1((0 as u8) as u8);
    game_camera.set_x398_b2((0 as u8) as u8);
    game_camera.set_x398_b3((0 as u8) as u8);
    game_camera.set_x398_b4((0 as u8) as u8);
    game_camera.set_x398_b5((0 as u8) as u8);
    game_camera.set_x399_b0_b1((0 as u8) as u8);
    game_camera.set_x399_b2((0 as u8) as u8);
    cm_80453004.last_mode = game_camera.mode;
    game_camera.set_x399_b3((0 as u8) as u8);
    game_camera.set_x399_b4((0 as u8) as u8);
    game_camera.set_x399_b5((0 as u8) as u8);
    game_camera.set_x399_b6((0 as u8) as u8);
    core::ptr::write_bytes(((core::ptr::addr_of_mut!(game_camera.x380) as *mut [u8; 24]) as *mut c_void) as *mut u8, 0, (24_u64 as i32) as usize);
    game_camera.set_x399_b7((0 as u8) as u8);
    game_camera.set_x39A_b0((0 as u8) as u8);
    game_camera.set_x39A_b1((0 as u8) as u8);
    game_camera.set_x39A_b2((0 as u8) as u8);
    game_camera.gobj = null_mut();
    cam_box = ext::HSD_MemAlloc(((n_subjects as u64) * 120_u64) as i32) as *mut CmSubject;
    cm_804D6458 = cam_box;
    cm_804D645C = cam_box;
    core::ptr::write_bytes((cm_804D6458 as *mut c_void) as *mut u8, 0, (((n_subjects as u64) * 120_u64) as i32) as usize);
    {
        i = 0;
        'l42: while i < (n_subjects - 1) {
            (*cm_804D645C.offset(i as isize)).prev = core::ptr::addr_of_mut!(*cm_804D645C.offset((i + 1) as isize)) as *mut CmSubject;
            i = i + 1;
        }
    }
    (*cm_804D645C.offset(i as isize)).prev = null_mut();
    cm_804D6460 = null_mut();
    cm_804D6468 = null_mut();
}

/// `Camera_80028F5C`
pub unsafe fn Camera_80028F5C(mut subject: *mut CmSubject, mut state: CmSubjectState) {
    if !subject.is_null() {
        (*subject).state = state;
        (*subject).pos.z = 0.0f32;
        (*subject).pos.y = 0.0f32;
        (*subject).pos.x = 0.0f32;
        (*subject).bone_pos = (*subject).pos;
        (*subject).facing_dir = 0.0f32;
        (*subject).set_on_ledge((0 as u8) as u8);
        (*subject).set_force_inactive((0 as u8) as u8);
        (*subject).set_was_framed((0 as u8) as u8);
        (*subject).state_timer = 0 as i16;
        (*subject).ext.h.x = -1.0f32;
        (*subject).ext.h.y = 1.0f32;
        (*subject).ext.v.x = 1.0f32;
        (*subject).ext.v.y = -1.0f32;
        (*subject).ext.v.z = 1.0f32;
        (*subject).target_ext.h = (*subject).ext.h;
        (*subject).target_ext.v = (*subject).ext.v;
        (*subject).x54.x = 0.0f32;
        (*subject).x54.y = 0.0f32;
        (*subject).x54.z = 0.0f32;
        (*subject).x60.x = 0.0f32;
        (*subject).x60.y = 0.0f32;
        (*subject).x60.z = 0.0f32;
    }
}

pub use crate::tu::camera::Camera_80029020;

/// `Camera_80029044`
pub unsafe fn Camera_80029044(mut arg0: i32) -> *mut CmSubject {
    let mut subject: *mut CmSubject = cm_804D6458;
    if cm_804D6458.is_null() {
        ext::OSReport(&format!("couldn't get CmSubject struct.\n"));
        'l43: while 1 != 0 {
        }
    }
    cm_804D6458 = (*subject).prev;
    (*subject).next = null_mut();
    if !cm_804D6460.is_null() {
        (*cm_804D6468).next = subject;
    } else {
        cm_804D6460 = subject;
    }
    (*subject).prev = cm_804D6468;
    cm_804D6468 = subject;
    Camera_80028F5C(subject, arg0 as u32);
    return subject;
}

pub use crate::tu::camera::Camera_800290D4;

/// `Camera_80029124`
pub unsafe fn Camera_80029124(mut subject_pos: *mut Vec, mut distance: i32) -> u32 {
    let mut _PAD: [u8; 4] = [0; 4];
    let mut cam_bounds_flags: u32 = 0;
    let mut slope: f32 = 0.0;
    let mut intercept: f32 = 0.0;
    let mut bounds_left: f32 = 0.0;
    let mut bounds_right: f32 = 0.0;
    let mut bounds_top: f32 = 0.0;
    let mut bounds_bottom: f32 = 0.0;
    cam_bounds_flags = 0 as u32;
    ext::Ground_801C4368(core::ptr::addr_of_mut!(slope) as *mut f32, core::ptr::addr_of_mut!(intercept) as *mut f32);
    slope = ((slope as f64) + 1.0f64) as f32;
    bounds_left = cm_fly_left();
    bounds_right = cm_fly_right();
    bounds_top = cm_fly_top();
    if cm_fly_bottom() > slope {
        bounds_bottom = cm_fly_bottom();
    } else {
        bounds_bottom = slope;
    }
    if (*subject_pos).x < (bounds_left - (distance as f32)) {
        cam_bounds_flags = cam_bounds_flags | ((1 << 2) as u32);
    }
    if (*subject_pos).x > (bounds_right + (distance as f32)) {
        cam_bounds_flags = cam_bounds_flags | ((1 << 3) as u32);
    }
    if (*subject_pos).y > (bounds_top + (distance as f32)) {
        cam_bounds_flags = cam_bounds_flags | ((1 << 0) as u32);
    }
    if (*subject_pos).y < (bounds_bottom - (distance as f32)) {
        cam_bounds_flags = cam_bounds_flags | ((1 << 1) as u32);
    }
    return cam_bounds_flags;
}

/// `cam_bound`
pub unsafe fn cam_bound(mut x: f32) -> i32 {
    return ((x > 0.65f32) || (x < 0.35f32)) as i32;
}

/// `Camera_8002928C`
pub unsafe fn Camera_8002928C(mut subject: *mut CmSubject) -> i32 {
    let mut left: f32 = 0.0;
    let mut right: f32 = 0.0;
    let mut bottom: f32 = 0.0;
    let mut top: f32 = 0.0;
    {
        if (ext::GenoFly_Any() != 0) && (!(ext::GenoFly_CameraSubject(subject as *mut c_void) != 0)) {
            return 0;
        }
    }
    if ((*subject).state != ((CmSubjectState_Inactive as i32) as u32)) && (!((*subject).force_inactive() != 0)) {
        if (*subject).state == ((CmSubjectState_Auto as i32) as u32) {
            if ((*subject).state_timer as i32) != 0 {
                (*subject).state_timer = (*subject).state_timer - 1;
                return 0;
            }
            left = cm_fly_left();
            right = cm_fly_right();
            top = cm_fly_top();
            bottom = cm_fly_bottom();
            if (cam_bound(((*subject).pos.x - left) / (right - left)) != 0) || (cam_bound(((*subject).pos.y - bottom) / (top - bottom)) != 0) {
                if (*subject).was_framed() != 0 {
                    (*subject).set_was_framed((0 as u8) as u8);
                    (*subject).state_timer = 600 as i16;
                }
                return 0;
            }
            (*subject).state_timer = 0 as i16;
        }
        (*subject).set_was_framed((1 as u8) as u8);
        return 1;
    }
    return 0;
}

/// `Camera_800293E0`
pub unsafe fn Camera_800293E0() {
    let mut curr: *mut CmSubject = null_mut();
    let mut temp_f0: f32 = 0.0;
    let mut temp_f1: f32 = 0.0;
    let mut distance: f32 = 0.0;
    {
        curr = cm_804D6468;
        'l44: while !curr.is_null() {
            if Camera_8002928C(curr) != 0 {
                temp_f1 = (*curr).target_ext.h.x;
                temp_f0 = (*curr).ext.h.x;
                distance = temp_f1 - temp_f0;
                if distance != 0.0f32 {
                    if distance > 0.5f32 {
                        (*curr).ext.h.x = (*curr).ext.h.x + 0.5f32;
                    } else if distance < (-0.5f32) {
                        (*curr).ext.h.x = (*curr).ext.h.x - 0.5f32;
                    } else {
                        (*curr).ext.h.x = temp_f1;
                    }
                }
                temp_f1 = (*curr).target_ext.h.y;
                temp_f0 = (*curr).ext.h.y;
                distance = temp_f1 - temp_f0;
                if distance != 0.0f32 {
                    if distance > 0.5f32 {
                        (*curr).ext.h.y = (*curr).ext.h.y + 0.5f32;
                    } else if distance < (-0.5f32) {
                        (*curr).ext.h.y = (*curr).ext.h.y - 0.5f32;
                    } else {
                        (*curr).ext.h.y = temp_f1;
                    }
                }
                temp_f1 = (*curr).target_ext.v.x;
                temp_f0 = (*curr).ext.v.x;
                distance = temp_f1 - temp_f0;
                if distance != 0.0f32 {
                    if distance > 0.5f32 {
                        (*curr).ext.v.x = (*curr).ext.v.x + 0.5f32;
                    } else if distance < (-0.5f32) {
                        (*curr).ext.v.x = (*curr).ext.v.x - 0.5f32;
                    } else {
                        (*curr).ext.v.x = temp_f1;
                    }
                }
                temp_f1 = (*curr).target_ext.v.y;
                temp_f0 = (*curr).ext.v.y;
                distance = temp_f1 - temp_f0;
                if distance != 0.0f32 {
                    if distance > 0.5f32 {
                        (*curr).ext.v.y = (*curr).ext.v.y + 0.5f32;
                    } else if distance < (-0.5f32) {
                        (*curr).ext.v.y = (*curr).ext.v.y - 0.5f32;
                    } else {
                        (*curr).ext.v.y = temp_f1;
                    }
                }
                temp_f1 = (*curr).target_ext.v.z;
                temp_f0 = (*curr).ext.v.z;
                distance = temp_f1 - temp_f0;
                if distance != 0.0f32 {
                    if distance > 0.5f32 {
                        (*curr).ext.v.z = (*curr).ext.v.z + 0.5f32;
                    } else if distance < (-0.5f32) {
                        (*curr).ext.v.z = temp_f0 - 0.5f32;
                    } else {
                        (*curr).ext.v.z = temp_f1;
                    }
                }
            }
            curr = (*curr).prev;
        }
    }
}

/// `Camera_8002958C`
pub unsafe fn Camera_8002958C(mut bounds: *mut CameraBounds, mut transform: *mut CameraTransformState) {
    let mut z_pos: f32 = 0.0;
    let mut z_factor: f32 = 0.0;
    let mut new_bounds: *mut CameraBounds = null_mut();
    let mut cam_bounds_flags: u32 = 0;
    let mut base_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut test_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut default_offset: Vec3 = unsafe { core::mem::zeroed() };
    let mut base_ground_intercept: f32 = 0.0;
    let mut base_ground: f32 = 0.0;
    let mut x_extent_intercept: f32 = 0.0;
    let mut x_extent_ground: f32 = 0.0;
    let mut alt_x_intercept: f32 = 0.0;
    let mut alt_x_ground: f32 = 0.0;
    let mut y_extent_intercept: f32 = 0.0;
    let mut y_extent_ground: f32 = 0.0;
    let mut alt_y_intercept: f32 = 0.0;
    let mut alt_y_ground: f32 = 0.0;
    let mut min_x: f32 = 0.0;
    let mut min_y: f32 = 0.0;
    let mut max_x: f32 = 0.0;
    let mut max_y: f32 = 0.0;
    let mut subject: *mut CmSubject = null_mut();
    let mut subject_count: i32 = 0;
    let mut tracking_weight: f32 = 0.0;
    let mut tracking_multiplier: f32 = 0.0;
    subject_count = 0;
    subject = cm_804D6468;
    {
        subject = cm_804D6468;
        'l45: while !subject.is_null() {
            if Camera_8002928C(subject) != 0 {
                subject_count = subject_count + 1;
            }
            subject = (*subject).prev;
        }
    }
    new_bounds = bounds;
    if subject_count != 0 {
        if (subject_count as u32) < (5 as u32) {
            tracking_weight = cm_803BCB9C[subject_count as usize];
        } else {
            tracking_weight = 1.0f32;
        }
        tracking_multiplier = tracking_weight * ext::Stage_GetCamTrackRatio();
        min_x = { min_y = f32::MAX; min_y };
        max_x = { max_y = -f32::MAX; max_y };
        subject_count = 0;
        subject = cm_804D6468;
        'l46: while !subject.is_null() {
            if Camera_8002928C(subject) != 0 {
                subject_count = subject_count + 1;
                base_pos = (*subject).pos;
                test_pos = (*subject).pos;
                cam_bounds_flags = Camera_80029124(core::ptr::addr_of_mut!(base_pos) as *mut Vec, 0);
                if cam_bounds_flags != (0 as u32) {
                    ext::Ground_801C4368(core::ptr::addr_of_mut!(base_ground) as *mut f32, core::ptr::addr_of_mut!(base_ground_intercept) as *mut f32);
                    base_ground = ((base_ground as f64) + 1.0f64) as f32;
                    if (cam_bounds_flags & ((1 << 2) as u32)) != 0 {
                        base_pos.x = cm_fly_left();
                    }
                    if (cam_bounds_flags & ((1 << 3) as u32)) != 0 {
                        base_pos.x = cm_fly_right();
                    }
                    if (cam_bounds_flags & ((1 << 0) as u32)) != 0 {
                        base_pos.y = cm_fly_top();
                    }
                    if (cam_bounds_flags & ((1 << 1) as u32)) != 0 {
                        base_pos.y = if cm_fly_bottom() > base_ground { cm_fly_bottom() } else { base_ground };
                    }
                }
                test_pos.x = fmadds((*subject).ext.h.x, tracking_multiplier, base_pos.x);
                cam_bounds_flags = Camera_80029124(core::ptr::addr_of_mut!(test_pos) as *mut Vec, 0);
                if cam_bounds_flags != (0 as u32) {
                    ext::Ground_801C4368(core::ptr::addr_of_mut!(x_extent_ground) as *mut f32, core::ptr::addr_of_mut!(x_extent_intercept) as *mut f32);
                    x_extent_ground = ((x_extent_ground as f64) + 1.0f64) as f32;
                    if (cam_bounds_flags & ((1 << 2) as u32)) != 0 {
                        test_pos.x = cm_fly_left();
                    }
                    if (cam_bounds_flags & ((1 << 3) as u32)) != 0 {
                        test_pos.x = cm_fly_right();
                    }
                    if (cam_bounds_flags & ((1 << 0) as u32)) != 0 {
                        test_pos.y = cm_fly_top();
                    }
                    if (cam_bounds_flags & ((1 << 1) as u32)) != 0 {
                        test_pos.y = if cm_fly_bottom() > x_extent_ground { cm_fly_bottom() } else { x_extent_ground };
                    }
                }
                if test_pos.x < min_x {
                    min_x = test_pos.x;
                }
                if test_pos.x > max_x {
                    max_x = test_pos.x;
                }
                test_pos.x = fmadds((*subject).ext.h.y, tracking_multiplier, base_pos.x);
                cam_bounds_flags = Camera_80029124(core::ptr::addr_of_mut!(test_pos) as *mut Vec, 0);
                if cam_bounds_flags != (0 as u32) {
                    ext::Ground_801C4368(core::ptr::addr_of_mut!(alt_x_ground) as *mut f32, core::ptr::addr_of_mut!(alt_x_intercept) as *mut f32);
                    alt_x_ground = ((alt_x_ground as f64) + 1.0f64) as f32;
                    if (cam_bounds_flags & ((1 << 2) as u32)) != 0 {
                        test_pos.x = cm_fly_left();
                    }
                    if (cam_bounds_flags & ((1 << 3) as u32)) != 0 {
                        test_pos.x = cm_fly_right();
                    }
                    if (cam_bounds_flags & ((1 << 0) as u32)) != 0 {
                        test_pos.y = cm_fly_top();
                    }
                    if (cam_bounds_flags & ((1 << 1) as u32)) != 0 {
                        test_pos.y = if cm_fly_bottom() > alt_x_ground { cm_fly_bottom() } else { alt_x_ground };
                    }
                }
                if test_pos.x < min_x {
                    min_x = test_pos.x;
                }
                if test_pos.x > max_x {
                    max_x = test_pos.x;
                }
                test_pos.y = fmadds((*subject).ext.v.y, tracking_multiplier, base_pos.y);
                cam_bounds_flags = Camera_80029124(core::ptr::addr_of_mut!(test_pos) as *mut Vec, 0);
                if cam_bounds_flags != (0 as u32) {
                    ext::Ground_801C4368(core::ptr::addr_of_mut!(y_extent_ground) as *mut f32, core::ptr::addr_of_mut!(y_extent_intercept) as *mut f32);
                    y_extent_ground = ((y_extent_ground as f64) + 1.0f64) as f32;
                    if (cam_bounds_flags & ((1 << 2) as u32)) != 0 {
                        test_pos.x = cm_fly_left();
                    }
                    if (cam_bounds_flags & ((1 << 3) as u32)) != 0 {
                        test_pos.x = cm_fly_right();
                    }
                    if (cam_bounds_flags & ((1 << 0) as u32)) != 0 {
                        test_pos.y = cm_fly_top();
                    }
                    if (cam_bounds_flags & ((1 << 1) as u32)) != 0 {
                        test_pos.y = if cm_fly_bottom() > y_extent_ground { cm_fly_bottom() } else { y_extent_ground };
                    }
                }
                if test_pos.y < min_y {
                    min_y = test_pos.y;
                }
                if test_pos.y > max_y {
                    max_y = test_pos.y;
                }
                test_pos.y = fmadds((*subject).ext.v.x, tracking_multiplier, base_pos.y);
                cam_bounds_flags = Camera_80029124(core::ptr::addr_of_mut!(test_pos) as *mut Vec, 0);
                if cam_bounds_flags != (0 as u32) {
                    ext::Ground_801C4368(core::ptr::addr_of_mut!(alt_y_ground) as *mut f32, core::ptr::addr_of_mut!(alt_y_intercept) as *mut f32);
                    alt_y_ground = ((alt_y_ground as f64) + 1.0f64) as f32;
                    if (cam_bounds_flags & ((1 << 2) as u32)) != 0 {
                        test_pos.x = cm_fly_left();
                    }
                    if (cam_bounds_flags & ((1 << 3) as u32)) != 0 {
                        test_pos.x = cm_fly_right();
                    }
                    if (cam_bounds_flags & ((1 << 0) as u32)) != 0 {
                        test_pos.y = cm_fly_top();
                    }
                    if (cam_bounds_flags & ((1 << 1) as u32)) != 0 {
                        test_pos.y = if cm_fly_bottom() > alt_y_ground { cm_fly_bottom() } else { alt_y_ground };
                    }
                }
                if test_pos.y < min_y {
                    min_y = test_pos.y;
                }
                if test_pos.y > max_y {
                    max_y = test_pos.y;
                }
            }
            subject = (*subject).prev;
        }
    }
    if subject_count == 0 {
        ext::Stage_UnkSetVec3TCam_Offset(core::ptr::addr_of_mut!(default_offset) as *mut Vec);
        min_x = default_offset.x - 4e+01f32;
        min_y = default_offset.y - 4e+01f32;
        max_x = 4e+01f32 + default_offset.x;
        max_y = 4e+01f32 + default_offset.y;
    }
    z_pos = if (*transform).position.z < (0 as f32) { -(*transform).position.z } else { (*transform).position.z };
    if z_pos < 8e+01f32 {
        z_factor = 0.0f32;
    } else if z_pos > 5e+03f32 {
        z_factor = 1.0f32;
    } else {
        z_factor = (z_pos - 8e+01f32) / 4.92e+03f32;
    }
    (*new_bounds).x_min = min_x;
    (*new_bounds).y_min = min_y - fmadds(3.9e+02f32, z_factor, 1e+01f32);
    (*new_bounds).x_max = max_x;
    (*new_bounds).y_max = max_y;
    (*new_bounds).total_subjects = subject_count;
    (*new_bounds).z_pos = z_pos;
}

/// `get_follow_speed`
pub unsafe fn get_follow_speed(mut temp_f4: f32, mut spread: f32, mut globals: *mut CameraUnkGlobals) -> f32 {
    if spread > temp_f4 {
        return (*globals).x30;
    } else if spread < (*globals).x34 {
        return (*globals).x2C;
    } else {
        return fmadds((spread - (*globals).x34) / (temp_f4 - (*globals).x34), (*globals).x30 - (*globals).x2C, (*globals).x2C);
    }
}

/// `get_delta`
pub unsafe fn get_delta(mut temp_f: f32) -> f32 {
    if temp_f > 0.0001f32 {
        return 1.0f32 / temp_f;
    } else {
        return 1e+03f32;
    }
}

/// `Camera_80029AAC`
pub unsafe fn Camera_80029AAC(mut bounds: *mut CameraBounds, mut transform: *mut CameraTransformState, mut speed: f32) {
    let mut globals: *mut CameraUnkGlobals = null_mut();
    let mut follow_speed: f32 = 0.0;
    let mut temp_f: f32 = 0.0;
    let mut temp_f4: f32 = 0.0;
    let mut temp_f3: f32 = 0.0;
    let mut delta: f32 = 0.0;
    let mut spread: f32 = 0.0;
    let mut lerp_factor: f32 = 0.0;
    let mut offset_x: f32 = 0.0;
    let mut offset_y: f32 = 0.0;
    if (*bounds).total_subjects != 0 {
        temp_f4 = (*bounds).x_max - (*bounds).x_min;
        temp_f3 = (*bounds).y_max - (*bounds).y_min;
        if temp_f4 > temp_f3 {
            spread = temp_f4;
        } else {
            spread = temp_f3;
        }
    } else {
        spread = 99999.0f32;
    }
    offset_x = (*transform).target_interest.x - (*transform).interest.x;
    offset_y = (*transform).target_interest.y - (*transform).interest.y;
    globals = core::ptr::addr_of_mut!(cm_803BCCA0) as *mut CameraUnkGlobals;
    follow_speed = get_follow_speed((*globals).x38, spread, globals);
    delta = get_delta(game_camera.x2BC);
    lerp_factor = (follow_speed * speed) * delta;
    if lerp_factor > 1.0f32 {
        lerp_factor = 1.0f32;
    } else if lerp_factor < 0.0001f32 {
        lerp_factor = 0.0001f32;
    }
    (*transform).interest.x = fmadds(offset_x, lerp_factor, (*transform).interest.x);
    (*transform).interest.y = fmadds(offset_y, lerp_factor, (*transform).interest.y);
}

/// `Camera_80029BC4`
pub unsafe fn Camera_80029BC4(mut bounds: *mut CameraBounds, mut transform: *mut CameraTransformState) {
    let mut cam_dist: f32 = ((*bounds).y_max - (*bounds).y_min) / ext::tanf((*transform).target_fov * 0.017453292f32);
    let mut x_dist: f32 = ((*bounds).x_max - (*bounds).x_min) / (cm_803BCB64.aspect * ext::tanf((*transform).target_fov * 0.017453292f32));
    if x_dist > cam_dist {
        cam_dist = x_dist;
    }
    if cam_dist < ext::Stage_GetCamZoomRate() {
        cam_dist = ext::Stage_GetCamZoomRate();
    }
    if cam_dist > ext::Stage_GetCamMaxDepth() {
        cam_dist = ext::Stage_GetCamMaxDepth();
    }
    (*bounds).z_pos = cam_dist;
}

/// `Camera_80029C88`
pub unsafe fn Camera_80029C88(mut unused: *mut CameraBounds, mut transform: *mut CameraTransformState, mut arg_scale: f32) {
    let mut dist: Vec3 = unsafe { core::mem::zeroed() };
    let mut scale: f32 = 0.0;
    let mut camera_speed: f32 = 0.0;
    dist.x = (*transform).target_position.x - (*transform).position.x;
    dist.y = (*transform).target_position.y - (*transform).position.y;
    dist.z = (*transform).target_position.z - (*transform).position.z;
    scale = cm_803BCCA0.x3C * arg_scale;
    if scale > 1.0f32 {
        scale = 1.0f32;
    }
    (*transform).position.x = fmadds(dist.x, scale, (*transform).position.x);
    (*transform).position.y = fmadds(dist.y, scale, (*transform).position.y);
    (*transform).position.z = fmadds(dist.z, scale, (*transform).position.z);
}

/// `get_y_bias`
pub unsafe fn get_y_bias(mut spread: f32) -> f32 {
    if spread > cm_803BCCA0.x28 {
        return cm_803BCCA0.x20;
    } else if spread < cm_803BCCA0.x24 {
        return cm_803BCCA0.x1C;
    } else {
        return fmadds(cm_803BCCA0.x20 - cm_803BCCA0.x1C, (spread - cm_803BCCA0.x24) / (cm_803BCCA0.x28 - cm_803BCCA0.x24), cm_803BCCA0.x1C);
    }
}

/// `Camera_80029CF8`
pub unsafe fn Camera_80029CF8(mut bounds: *mut CameraBounds, mut transform: *mut CameraTransformState) {
    let mut x_center: f32 = 0.0;
    let mut y_angle: f32 = 0.0;
    let mut angle: f32 = 0.0;
    let mut tan_r: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut spread: f32 = 0.0;
    let mut _pad: [u8; 12] = [0; 12];
    let mut y_off: f32 = 0.0;
    let mut base: f32 = 0.0;
    let mut fov_r: f32 = 0.0;
    let mut fov_u: f32 = 0.0;
    let mut fov_d: f32 = 0.0;
    let mut cam_dist: f32 = 0.0;
    let mut tan_u: f32 = 0.0;
    let mut x_off: f32 = 0.0;
    let mut tan_l: f32 = 0.0;
    let mut tan_d: f32 = 0.0;
    let mut dist_y: f32 = 0.0;
    let mut dist_x: f32 = 0.0;
    let mut sp24: Vec3 = unsafe { core::mem::zeroed() };
    let mut fov_l: f32 = 0.0;
    let mut t: f32 = 0.0;
    let mut sp14: Vec3 = unsafe { core::mem::zeroed() };
    let mut y_sum: f32 = 0.0;
    let mut scaled_tan: f32 = 0.0;
    ext::Stage_UnkSetVec3TCam_Offset(core::ptr::addr_of_mut!(sp24) as *mut Vec);
    dx = (*bounds).x_max - (*bounds).x_min;
    dy = (*bounds).y_max - (*bounds).y_min;
    if dx > dy {
        spread = dx;
    } else {
        spread = dy;
    }
    t = get_y_bias(spread);
    y_sum = ((*bounds).y_min - sp24.y) + ((*bounds).y_max - sp24.y);
    base = fmadds(y_sum, 0.5f32 - t, sp24.y);
    angle = -(((base + cm_803BCCA0.x8) * ext::Stage_GetCamInfoX24()) * 0.017453292f32);
    if angle > (cm_803BCCA0.xC * 0.017453292f32) {
        angle = cm_803BCCA0.xC * 0.017453292f32;
    }
    if angle < (cm_803BCCA0.x10 * 0.017453292f32) {
        angle = cm_803BCCA0.x10 * 0.017453292f32;
    }
    angle = angle + ext::Stage_GetCamPanAngleRadians();
    y_angle = angle;
    fov_u = fmadds(0.5f32, (*transform).fov * 0.017453292f32, angle);
    if fov_u < (9e+01f32 * 0.017453292f32) { (); } else { panic!("Camera_80029CF8: assertion failed"); }
    fov_d = fmadds(0.5f32, (*transform).fov * 0.017453292f32, -angle);
    if fov_d < (9e+01f32 * 0.017453292f32) { (); } else { panic!("Camera_80029CF8: assertion failed"); }
    tan_u = ext::tanf(fov_u);
    tan_d = ext::tanf(fov_d);
    dist_y = ((*bounds).y_max - (*bounds).y_min) / (tan_u + tan_d);
    let _ = cm_fly_bottom();
    let _ = cm_fly_top();
    y_off = dist_y * ext::tanf(y_angle);
    (*transform).target_interest.y = y_off + fmadds(-dist_y, tan_u, (*bounds).y_max);
    ext::Stage_UnkSetVec3TCam_Offset(core::ptr::addr_of_mut!(sp14) as *mut Vec);
    x_center = 0.5f32 * ((*bounds).x_min + (*bounds).x_max);
    angle = -(((x_center - sp14.x) * ext::Stage_GetCamInfoX20()) * 0.017453292f32);
    if angle > (cm_803BCCA0.x14 * 0.017453292f32) {
        angle = cm_803BCCA0.x14 * 0.017453292f32;
    }
    if angle < (cm_803BCCA0.x18 * 0.017453292f32) {
        angle = cm_803BCCA0.x18 * 0.017453292f32;
    }
    fov_r = fmadds(0.5f32, (*transform).fov * 0.017453292f32, -angle);
    if fov_r < (9e+01f32 * 0.017453292f32) { (); } else { panic!("Camera_80029CF8: assertion failed"); }
    fov_l = fmadds(0.5f32, (*transform).fov * 0.017453292f32, angle);
    if fov_l < (9e+01f32 * 0.017453292f32) { (); } else { panic!("Camera_80029CF8: assertion failed"); }
    scaled_tan = cm_803BCB64.aspect * ext::tanf(fov_r);
    tan_r = scaled_tan;
    tan_l = cm_803BCB64.aspect * ext::tanf(fov_l);
    dist_x = ((*bounds).x_max - (*bounds).x_min) / (tan_r + tan_l);
    let _ = cm_fly_left();
    let _ = cm_fly_right();
    x_off = cm_803BCB64.aspect * (dist_x * ext::tanf(angle));
    (*transform).target_interest.x = fmadds(-dist_x, tan_r, (*bounds).x_max) - x_off;
    (*transform).target_interest.z = 0.0f32;
    dist_y = if dist_y > dist_x { dist_y } else { dist_x };
    cam_dist = dist_y;
    if dist_y < ext::Stage_GetCamZoomRate() {
        cam_dist = ext::Stage_GetCamZoomRate();
    }
    if cam_dist > ext::Stage_GetCamMaxDepth() {
        cam_dist = ext::Stage_GetCamMaxDepth();
    }
    (*transform).target_position.x = (*transform).target_interest.x + x_off;
    (*transform).target_position.y = (*transform).target_interest.y - y_off;
    (*transform).target_position.z = (*transform).target_interest.z + cam_dist;
}

/// `Camera_ApplyQuake`
pub unsafe fn Camera_ApplyQuake(mut bounds: *mut CameraBounds, mut state: *mut CameraTransformState) {
    let mut viewport_x_scale: f32 = 0.0;
    let mut viewport_y_scale: f32 = 0.0;
    let mut depth_factor_y: f32 = 0.0;
    let mut depth_factor_x: f32 = 0.0;
    let mut half_view_height: f32 = 0.0;
    let mut input_x: f32 = 0.0;
    let mut input_y: f32 = 0.0;
    let mut depth_ratio: f32 = 0.0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local5 { callbacks: CameraModeCallbacks, interest: HSD_WObjDesc, eyepos: HSD_WObjDesc, desc: HSD_CameraDescPerspective }
    let mut data: *mut c_void /* CameraStaticData */ = (core::ptr::addr_of_mut!(cm_803BCB18) as *mut CameraModeCallbacks) as *mut c_void /* CameraStaticData */;
    input_x = game_camera.quake_offset.x * game_camera.quake_scale;
    input_y = game_camera.quake_offset.y * game_camera.quake_scale;
    input_x = input_x * 1e+01f32;
    input_y = input_y * 1e+01f32;
    if (ext::gm_IsCurrently1PMode_inline() != 0) && (!(ext::Mex_Enabled((c"enable_c_stick_always_1p_camera2".as_ptr() as *mut i8) as *const i8) != 0)) {
        input_x = input_x * cm_803BCCA0.xE8;
        input_y = input_y * cm_803BCCA0.xE8;
    }
    input_x = input_x * game_camera.x2BC;
    input_y = input_y * game_camera.x2BC;
    half_view_height = (*bounds).z_pos * ext::tanf(0.5f32 * (0.017453292f32 * (*state).fov));
    viewport_x_scale = cm_803BCB64.aspect * (half_view_height / (0.5f32 * (((cm_803BCB64.viewport.xmax as i32) - (cm_803BCB64.viewport.xmin as i32)) as f32)));
    viewport_y_scale = half_view_height / (0.5f32 * (((cm_803BCB64.viewport.ymax as i32) - (cm_803BCB64.viewport.ymin as i32)) as f32));
    depth_factor_y = ext::Stage_GetCamZoomRate();
    depth_factor_x = ext::Stage_GetCamMaxDepth() - depth_factor_y;
    if depth_factor_x < 0.001f32 {
        depth_ratio = 0.5f32;
    } else {
        depth_ratio = ((*bounds).z_pos - ext::Stage_GetCamZoomRate()) / depth_factor_x;
    }
    depth_factor_y = fmadds(depth_ratio, cm_803BCCA0.x5C - cm_803BCCA0.x54, cm_803BCCA0.x54);
    depth_factor_x = fmadds(depth_ratio, cm_803BCCA0.x60 - cm_803BCCA0.x58, cm_803BCCA0.x58);
    ext::diag_cam_translate(cm_803BCB64.viewport.xmin as i32, cm_803BCB64.viewport.xmax as i32, cm_803BCB64.viewport.ymin as i32, cm_803BCB64.viewport.ymax as i32, cm_803BCB64.aspect, (*state).fov, (*bounds).z_pos, half_view_height, depth_factor_x * (input_x * viewport_x_scale), depth_factor_y * (input_y * viewport_y_scale));
    Camera_80030DE4(depth_factor_x * (input_x * viewport_x_scale), depth_factor_y * (input_y * viewport_y_scale));
    game_camera.quake_offset.x = 0.0f32;
    game_camera.quake_offset.y = 0.0f32;
}

/// `Camera_SetQuakeOffset`
pub unsafe fn Camera_SetQuakeOffset(mut x: f32, mut y: f32) {
    game_camera.quake_offset.x = x;
    game_camera.quake_offset.y = y;
}

/// `Camera_UpdateQuakes`
pub unsafe fn Camera_UpdateQuakes(mut bounds: *mut CameraBounds) {
    let mut __: [u8; 16] = [0; 16];
    let mut quakes_remaining: i32 = 0;
    let mut i: i32 = 0;
    quakes_remaining = (-1_i32);
    {
        i = 0;
        'l47: while i < 16 {
            game_camera.quakes[1 as usize][i as usize] = game_camera.quakes[0 as usize][i as usize];
            game_camera.quakes[0 as usize][i as usize].kind = (QuakeKind_None as i32) as u32;
            i = i + 1;
        }
    }
    {
        i = 0;
        'l48: while i < (QuakeKind_Count as i32) {
            if game_camera.quake_frames_left[i as usize] != 0 {
                game_camera.quake_frames_left[i as usize] = game_camera.quake_frames_left[i as usize] - 1;
                quakes_remaining = i;
            }
            i = i + 1;
        }
    }
    if ((quakes_remaining != (-1_i32)) && (!game_camera.quake_gobj.is_null())) && (game_camera.quake_frames_left[(QuakeKind_Loop as i32) as usize] == 0) {
        ext::HSD_GObjFree(game_camera.quake_gobj);
        game_camera.quake_gobj = null_mut();
    }
}

/// `get_stage_floor_height`
pub unsafe fn get_stage_floor_height(mut kind: GrKind) -> f32 {
    let mut height: f32 = -f32::MAX;
    's49: {
        match kind {
            2 => {
                height = ext::grCastle_801D0FF0();
            }
            14 => {
                height = ext::grCorneria_801E2FCC();
            }
            8 => {
                height = ext::grZebes_801DCCC8();
            }
            5 => {
                height = ext::grGarden_80203624();
            }
            31 => {
                height = ext::grKinokoRoute_802087B0();
            }
            67 => {
                height = ext::grHomeRun_8021EF10();
            }
            _ => {
            }
        }
    }
    return height;
}

/// `Camera_8002A4AC`
pub unsafe fn Camera_8002A4AC(mut gobj: *mut HSD_GObj) {
    let mut floor_height: f32 = 0.0;
    let mut pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut cobj: *mut HSD_CObj = null_mut();
    let mut transform: *mut CameraTransformState = core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState;
    cobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_CObj;
    's50: {
        match game_camera.mode {
            0 | 4 => {
                Camera_8002AF68(cobj, transform);
                Camera_8002AF68(cm_804D6464, core::ptr::addr_of_mut!(game_camera.transform_copy) as *mut CameraTransformState);
                ext::HSD_CObjSetNear(cobj, game_camera.nearz);
                ext::HSD_CObjSetFar(cobj, game_camera.farz);
            }
            5 => {
                Camera_8002AF68(cobj, transform);
                ext::HSD_CObjSetNear(cobj, game_camera.nearz);
                ext::HSD_CObjSetFar(cobj, game_camera.farz);
            }
            1 => {
                pos = (*transform).position;
                floor_height = get_stage_floor_height(ext::stage_info.grkind);
                if pos.y < floor_height {
                    floor_height = get_stage_floor_height(ext::stage_info.grkind);
                    pos.y = floor_height;
                }
                ext::HSD_CObjSetFov(cobj, (*transform).fov);
                ext::HSD_CObjSetInterest(cobj, core::ptr::addr_of_mut!((*transform).interest) as *mut Vec);
                ext::HSD_CObjSetEyePosition(cobj, core::ptr::addr_of_mut!(pos) as *mut Vec);
                ext::HSD_CObjSetNear(cobj, 0.1f32);
                ext::HSD_CObjSetFar(cobj, 16384.0f32);
            }
            2 | 3 | 6 => {
                ext::HSD_CObjSetFov(cobj, (*transform).fov);
                ext::HSD_CObjSetInterest(cobj, core::ptr::addr_of_mut!((*transform).interest) as *mut Vec);
                ext::HSD_CObjSetEyePosition(cobj, core::ptr::addr_of_mut!((*transform).position) as *mut Vec);
                ext::HSD_CObjSetNear(cobj, 0.1f32);
                ext::HSD_CObjSetFar(cobj, 16384.0f32);
            }
            7 => {
                ext::HSD_CObjSetFov(cobj, cm_80453004.follow_fov);
                ext::HSD_CObjSetInterest(cobj, core::ptr::addr_of_mut!(cm_80453004.follow_eye_pos) as *mut Vec);
                ext::HSD_CObjSetEyePosition(cobj, core::ptr::addr_of_mut!(cm_80453004.follow_int_pos) as *mut Vec);
            }
            8 => {
                ext::HSD_CObjSetFov(cobj, cm_80453004.free_fov);
                ext::HSD_CObjSetInterest(cobj, core::ptr::addr_of_mut!(cm_80453004.free_int_pos) as *mut Vec);
                ext::HSD_CObjSetEyePosition(cobj, core::ptr::addr_of_mut!(cm_80453004.free_eye_pos) as *mut Vec);
            }
            _ => {}
        }
    }
    if cm_script.state != 0 {
        ext::HSD_CObjSetFov(cobj, (*transform).fov);
        ext::HSD_CObjSetInterest(cobj, core::ptr::addr_of_mut!((*transform).interest) as *mut Vec);
        ext::HSD_CObjSetEyePosition(cobj, core::ptr::addr_of_mut!((*transform).position) as *mut Vec);
        ext::HSD_CObjSetRoll(cobj, cm_script.pose.roll);
        if cm_script.lift_bounds != 0 {
            ext::HSD_CObjSetFar(cobj, 2e+05f32);
        } else if (game_camera.mode == ((CAMERA_DEBUG_FOLLOW as i32) as u32)) || (game_camera.mode == ((CAMERA_DEBUG_FREE as i32) as u32)) {
            ext::HSD_CObjSetFar(cobj, cm_script.saved_far);
        }
    }
    {
        if ext::ScriptGame_MapWide() != 0 {
            if !(!cm_largemap.primary.is_null()) {
                cm_largemap.primary = cobj;
                cm_largemap.primary_far = ext::HSD_CObjGetFar(cobj);
                cm_largemap.secondary = cm_804D6464;
                if !cm_804D6464.is_null() {
                    cm_largemap.secondary_far = ext::HSD_CObjGetFar(cm_804D6464);
                }
            }
            ext::HSD_CObjSetFar(cobj, 2e+05f32);
            if !cm_804D6464.is_null() {
                ext::HSD_CObjSetFar(cm_804D6464, 2e+05f32);
            }
        }
    }
}

/// `Camera_8002A768`
pub unsafe fn Camera_8002A768(mut transform: *mut CameraTransformState, mut arg1: i32) {
    let mut dist: Vec3 = unsafe { core::mem::zeroed() };
    let mut forward: Vec3 = unsafe { core::mem::zeroed() };
    let mut top_left: Vec3 = unsafe { core::mem::zeroed() };
    let mut top_right: Vec3 = unsafe { core::mem::zeroed() };
    let mut bottom_right: Vec3 = unsafe { core::mem::zeroed() };
    let mut bottom_left: Vec3 = unsafe { core::mem::zeroed() };
    let mut cam_correction: Vec3 = unsafe { core::mem::zeroed() };
    let mut half_fov: f32 = 0.0;
    let mut temp_f0: f32 = 0.0;
    let mut temp_f1: f32 = 0.0;
    let mut pitch_angle: f32 = 0.0;
    let mut top_overlap: f32 = 0.0;
    let mut right_overlap: f32 = 0.0;
    let mut bottom_overlap: f32 = 0.0;
    let mut left_overlap: f32 = 0.0;
    let mut var_r31: i32 = 0;
    let mut var_r30: i32 = 0;
    var_r30 = { var_r31 = 0; var_r31 };
    half_fov = 0.5f32 * ((*transform).target_fov * 0.017453292f32);
    forward = cm_WorldForward;
    let _ = lbVector_Diff(core::ptr::addr_of_mut!((*transform).target_interest) as *mut Vec, core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec, core::ptr::addr_of_mut!(dist) as *mut Vec);
    let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(dist) as *mut Vec);
    pitch_angle = ext::atan2f(dist.y, -dist.z);
    temp_f0 = ext::atan2f(-dist.x, -dist.z);
    top_left = { top_right = { bottom_right = { bottom_left = forward; bottom_left }; bottom_right }; top_right };
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(top_left) as *mut Vec, 1, half_fov);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(top_left) as *mut Vec, 2, half_fov);
    top_left.x = top_left.x * cm_803BCB64.aspect;
    let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(top_left) as *mut Vec);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(top_left) as *mut Vec, 1, pitch_angle);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(top_left) as *mut Vec, 2, temp_f0);
    if top_left.z < (-0.001f32) {
        temp_f1 = (-(*transform).target_position.z) / top_left.z;
        top_left.x = top_left.x * temp_f1;
        top_left.y = top_left.y * temp_f1;
        top_left.z = top_left.z * temp_f1;
        let _ = lbVector_Add(core::ptr::addr_of_mut!(top_left) as *mut Vec, core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec);
    } else {
        var_r30 = var_r30 | 1;
    }
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(top_right) as *mut Vec, 1, half_fov);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(top_right) as *mut Vec, 2, -half_fov);
    top_right.x = top_right.x * cm_803BCB64.aspect;
    let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(top_right) as *mut Vec);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(top_right) as *mut Vec, 1, pitch_angle);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(top_right) as *mut Vec, 2, temp_f0);
    if top_right.z < (-0.001f32) {
        temp_f1 = (-(*transform).target_position.z) / top_right.z;
        top_right.x = top_right.x * temp_f1;
        top_right.y = top_right.y * temp_f1;
        top_right.z = top_right.z * temp_f1;
        let _ = lbVector_Add(core::ptr::addr_of_mut!(top_right) as *mut Vec, core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec);
    } else {
        var_r30 = var_r30 | 2;
    }
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(bottom_right) as *mut Vec, 1, -half_fov);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(bottom_right) as *mut Vec, 2, half_fov);
    bottom_right.x = bottom_right.x * cm_803BCB64.aspect;
    let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(bottom_right) as *mut Vec);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(bottom_right) as *mut Vec, 1, pitch_angle);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(bottom_right) as *mut Vec, 2, temp_f0);
    if bottom_right.z < (-0.001f32) {
        temp_f1 = (-(*transform).target_position.z) / bottom_right.z;
        bottom_right.x = bottom_right.x * temp_f1;
        bottom_right.y = bottom_right.y * temp_f1;
        bottom_right.z = bottom_right.z * temp_f1;
        let _ = lbVector_Add(core::ptr::addr_of_mut!(bottom_right) as *mut Vec, core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec);
    } else {
        var_r30 = var_r30 | 4;
    }
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(bottom_left) as *mut Vec, 1, -half_fov);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(bottom_left) as *mut Vec, 2, -half_fov);
    bottom_left.x = bottom_left.x * cm_803BCB64.aspect;
    let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(bottom_left) as *mut Vec);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(bottom_left) as *mut Vec, 1, pitch_angle);
    ext::lbVector_Rotate(core::ptr::addr_of_mut!(bottom_left) as *mut Vec, 2, temp_f0);
    if bottom_left.z < (-0.001f32) {
        temp_f1 = (-(*transform).target_position.z) / bottom_left.z;
        bottom_left.x = bottom_left.x * temp_f1;
        bottom_left.y = bottom_left.y * temp_f1;
        bottom_left.z = bottom_left.z * temp_f1;
        let _ = lbVector_Add(core::ptr::addr_of_mut!(bottom_left) as *mut Vec, core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec);
    } else {
        var_r30 = var_r30 | 8;
    }
    left_overlap = { bottom_overlap = { right_overlap = { top_overlap = 0.0f32; top_overlap }; right_overlap }; bottom_overlap };
    if arg1 != 0 {
        if !((var_r30 & 1) != 0) {
            if top_left.x < ext::Stage_GetBlastZoneLeftOffset() {
                temp_f0 = ext::Stage_GetBlastZoneLeftOffset() - top_left.x;
                if temp_f0 > left_overlap {
                    left_overlap = temp_f0;
                    var_r31 = var_r31 | 4;
                }
            }
            if top_left.y > ext::Stage_GetBlastZoneTopOffset() {
                temp_f1 = ext::Stage_GetBlastZoneTopOffset() - top_left.y;
                if temp_f1 < top_overlap {
                    top_overlap = temp_f1;
                    var_r31 = var_r31 | 1;
                }
            }
        }
        if !((var_r30 & 2) != 0) {
            if top_right.x > ext::Stage_GetBlastZoneRightOffset() {
                temp_f0 = ext::Stage_GetBlastZoneRightOffset() - top_right.x;
                if temp_f0 < right_overlap {
                    right_overlap = temp_f0;
                    var_r31 = var_r31 | 8;
                }
            }
            if top_right.y > ext::Stage_GetBlastZoneTopOffset() {
                temp_f0 = ext::Stage_GetBlastZoneTopOffset() - top_right.y;
                if temp_f0 < top_overlap {
                    top_overlap = temp_f0;
                    var_r31 = var_r31 | 1;
                }
            }
        }
        if !((var_r30 & 4) != 0) {
            if bottom_right.x < ext::Stage_GetBlastZoneLeftOffset() {
                temp_f0 = ext::Stage_GetBlastZoneLeftOffset() - bottom_right.x;
                if temp_f0 > left_overlap {
                    left_overlap = temp_f0;
                    var_r31 = var_r31 | 4;
                }
            }
            if bottom_right.y < ext::Stage_GetBlastZoneBottomOffset() {
                temp_f0 = ext::Stage_GetBlastZoneBottomOffset() - bottom_right.y;
                if temp_f0 > bottom_overlap {
                    bottom_overlap = temp_f0;
                    var_r31 = var_r31 | 2;
                }
            }
        }
        if !((var_r30 & 8) != 0) {
            if bottom_left.x > ext::Stage_GetBlastZoneRightOffset() {
                temp_f0 = ext::Stage_GetBlastZoneRightOffset() - bottom_left.x;
                if temp_f0 < right_overlap {
                    right_overlap = temp_f0;
                    var_r31 = var_r31 | 8;
                }
            }
            if bottom_left.y < ext::Stage_GetBlastZoneBottomOffset() {
                temp_f0 = ext::Stage_GetBlastZoneBottomOffset() - bottom_left.y;
                if temp_f0 > bottom_overlap {
                    bottom_overlap = temp_f0;
                    var_r31 = var_r31 | 2;
                }
            }
        }
    } else {
        if !((var_r30 & 1) != 0) {
            if top_left.x < cm_fly_left() {
                temp_f0 = cm_fly_left() - top_left.x;
                if temp_f0 > left_overlap {
                    left_overlap = temp_f0;
                    var_r31 = var_r31 | 4;
                }
            }
            if top_left.y > cm_fly_top() {
                temp_f1 = cm_fly_top() - top_left.y;
                if temp_f1 < top_overlap {
                    top_overlap = temp_f1;
                    var_r31 = var_r31 | 1;
                }
            }
        }
        if !((var_r30 & 2) != 0) {
            if top_right.x > cm_fly_right() {
                temp_f0 = cm_fly_right() - top_right.x;
                if temp_f0 < right_overlap {
                    right_overlap = temp_f0;
                    var_r31 = var_r31 | 8;
                }
            }
            if top_right.y > cm_fly_top() {
                temp_f0 = cm_fly_top() - top_right.y;
                if temp_f0 < top_overlap {
                    top_overlap = temp_f0;
                    var_r31 = var_r31 | 1;
                }
            }
        }
        if !((var_r30 & 4) != 0) {
            if bottom_right.x < cm_fly_left() {
                temp_f0 = cm_fly_left() - bottom_right.x;
                if temp_f0 > left_overlap {
                    left_overlap = temp_f0;
                    var_r31 = var_r31 | 4;
                }
            }
            if bottom_right.y < cm_fly_bottom() {
                temp_f0 = cm_fly_bottom() - bottom_right.y;
                if temp_f0 > bottom_overlap {
                    bottom_overlap = temp_f0;
                    var_r31 = var_r31 | 2;
                }
            }
        }
        if !((var_r30 & 8) != 0) {
            if bottom_left.x > cm_fly_right() {
                temp_f0 = cm_fly_right() - bottom_left.x;
                if temp_f0 < right_overlap {
                    right_overlap = temp_f0;
                    var_r31 = var_r31 | 8;
                }
            }
            if bottom_left.y < cm_fly_bottom() {
                temp_f0 = cm_fly_bottom() - bottom_left.y;
                if temp_f0 > bottom_overlap {
                    bottom_overlap = temp_f0;
                    var_r31 = var_r31 | 2;
                }
            }
        }
    }
    if var_r31 != 0 {
        cam_correction = cm_803B73C4;
        if ((var_r31 & 4) != 0) && ((var_r31 & 8) != 0) {
            cam_correction.x = 0.5f32 * (left_overlap + right_overlap);
        } else if (var_r31 & 4) != 0 {
            cam_correction.x = left_overlap;
        } else if (var_r31 & 8) != 0 {
            cam_correction.x = right_overlap;
        }
        if ((var_r31 & 1) != 0) && ((var_r31 & 2) != 0) {
            cam_correction.y = 0.5f32 * (bottom_overlap + top_overlap);
        } else if (var_r31 & 1) != 0 {
            cam_correction.y = top_overlap;
        } else if (var_r31 & 2) != 0 {
            cam_correction.y = bottom_overlap;
        }
        temp_f1 = 1.0f32;
        cam_correction.x = cam_correction.x * temp_f1;
        cam_correction.y = cam_correction.y * temp_f1;
        cam_correction.z = cam_correction.z * temp_f1;
        let _ = lbVector_Add(core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec, core::ptr::addr_of_mut!(cam_correction) as *mut Vec);
        let _ = lbVector_Add(core::ptr::addr_of_mut!((*transform).target_interest) as *mut Vec, core::ptr::addr_of_mut!(cam_correction) as *mut Vec);
    }
}

/// `Camera_8002AF68`
pub unsafe fn Camera_8002AF68(mut cobj: *mut HSD_CObj, mut transform: *mut CameraTransformState) {
    let mut _1: [u8; 4] = [0; 4];
    let mut vec: Vec3 = unsafe { core::mem::zeroed() };
    let mut eye_y_bound: f32 = 0.0;
    let mut _2: [u8; 4] = [0; 4];
    ext::HSD_CObjSetFov(cobj, (*transform).fov);
    ext::diag_game_camera((core::ptr::addr_of_mut!((*transform).interest) as *mut Vec) as *const f32, (core::ptr::addr_of_mut!((*transform).position) as *mut Vec) as *const f32, (core::ptr::addr_of_mut!(game_camera.translation) as *mut Vec2) as *const f32);
    ext::diag_camera_consts(cm_803BCCA0.x64, cm_803BCCA0.x6C, cm_803BCCA0.x70, cm_803BCCA0.x3C);
    ext::diag_camera_track((core::ptr::addr_of_mut!(game_camera.transform.interest) as *mut Vec) as *const f32, (core::ptr::addr_of_mut!(game_camera.transform.position) as *mut Vec) as *const f32);
    vec = (*transform).interest;
    vec.x = vec.x + game_camera.translation.x;
    vec.y = vec.y + game_camera.translation.y;
    ext::HSD_CObjSetInterest(cobj, core::ptr::addr_of_mut!(vec) as *mut Vec);
    vec = (*transform).position;
    vec.x = vec.x + game_camera.translation.x;
    vec.y = vec.y + game_camera.translation.y;
    eye_y_bound = -f32::MAX;
    's51: {
        match ext::stage_info.grkind {
            2 => {
                eye_y_bound = ext::grCastle_801D0FF0();
            }
            14 => {
                eye_y_bound = ext::grCorneria_801E2FCC();
            }
            8 => {
                eye_y_bound = ext::grZebes_801DCCC8();
            }
            5 => {
                eye_y_bound = ext::grGarden_80203624();
            }
            31 => {
                eye_y_bound = ext::grKinokoRoute_802087B0();
            }
            67 => {
                eye_y_bound = ext::grHomeRun_8021EF10();
            }
            _ => {
            }
        }
    }
    if vec.y < eye_y_bound {
        vec.y = eye_y_bound;
    }
    ext::HSD_CObjSetEyePosition(cobj, core::ptr::addr_of_mut!(vec) as *mut Vec);
}

/// `Camera_8002B0E0`
pub unsafe fn Camera_8002B0E0() {
    let mut var_f1: f32 = 0.0;
    let mut var_f2: f32 = 0.0;
    if (((ext::gm_IsCurrently1PMode_inline() != 0) && (game_camera.x2C0 > 0.0f32)) && (!(ext::gm_CStickSmashTargetTest != 0))) && (!(ext::Mex_Enabled((c"enable_c_stick_always_1p_camera".as_ptr() as *mut i8) as *const i8) != 0)) {
        {
            let mut idx: i32 = ext::Player_GetPlayerId(0) & 255;
            var_f1 = ext::HSD_PadCopyStatus[idx as usize].nml_subStickY;
        }
        var_f2 = var_f1;
        if var_f2 < 0.0f32 {
            var_f1 = -var_f1;
        }
        if var_f1 < 0.85f32 {
            var_f2 = 0.0f32;
            if (game_camera.x2BA as i32) > 0 {
                game_camera.x2BA = ((game_camera.x2BA as i32) - 1) as i16;
            } else {
                var_f2 = cm_803BCCA0.xE4;
            }
        } else {
            game_camera.x2BA = melee_runtime::cabi::f2i16(cm_803BCCA0.xE0 as f64);
        }
        game_camera.x2BC = fmadds(-var_f2, cm_803BCCA0.xD4, game_camera.x2BC);
        if game_camera.x2BC < cm_803BCCA0.xDC {
            game_camera.x2BC = cm_803BCCA0.xDC;
            return;
        }
        if game_camera.x2BC > cm_803BCCA0.xD8 {
            game_camera.x2BC = cm_803BCCA0.xD8;
        }
    }
}

/// `Camera_8002B1F8`
pub unsafe fn Camera_8002B1F8(mut transform: *mut CameraTransformState) {
    let mut vec: Vec3 = unsafe { core::mem::zeroed() };
    let mut temp_r3: *mut HSD_GObj = null_mut();
    let mut temp_r3_2: *mut HSD_GObj = null_mut();
    let mut subject: *mut CmSubject = null_mut();
    let mut temp_f1: f32 = 0.0;
    let mut temp_f1_2: f32 = 0.0;
    let mut temp_f31: f32 = 0.0;
    let mut temp_r31: *mut f32 = null_mut();
    temp_r31 = core::ptr::addr_of_mut!(game_camera.x2BC) as *mut f32;
    if 1.0f32 == (*temp_r31) {
        return;
    }
    if ((((({ { temp_r3 = ext::Player_GetEntity(0); temp_r3 }; !temp_r3.is_null() }) && ({ { subject = ext::ftLib_80086B74(temp_r3); subject }; !subject.is_null() })) && (Camera_8002928C(subject) != 0)) && (Camera_80029124(core::ptr::addr_of_mut!((*subject).bone_pos) as *mut Vec, 0) == (0 as u32))) && (!(ext::ftLib_8008732C(temp_r3) != 0))) || (((((ext::Player_GetPlayerCharacter(1) == ((ChKind_Sandbag as i32) as u32)) && ({ { temp_r3_2 = ext::Player_GetEntity(1); temp_r3_2 }; ((temp_r3_2.is_null()) as i32) == 0 })) && ({ { subject = ext::ftLib_80086B74(temp_r3_2); subject }; ((subject.is_null()) as i32) == 0 })) && (Camera_8002928C(subject) != 0)) && (Camera_80029124(core::ptr::addr_of_mut!((*subject).bone_pos) as *mut Vec, 0) == (0 as u32))) {
        let _ = lbVector_Diff(core::ptr::addr_of_mut!((*transform).target_interest) as *mut Vec, core::ptr::addr_of_mut!((*subject).bone_pos) as *mut Vec, core::ptr::addr_of_mut!(vec) as *mut Vec);
        temp_f1 = *temp_r31;
        temp_f1_2 = temp_f1 * temp_f1;
        vec.x = vec.x * temp_f1_2;
        vec.y = vec.y * temp_f1_2;
        vec.z = vec.z * temp_f1_2;
        let _ = lbVector_Add(core::ptr::addr_of_mut!(vec) as *mut Vec, core::ptr::addr_of_mut!((*subject).bone_pos) as *mut Vec);
        (*transform).target_interest = vec;
        let _ = lbVector_Diff(core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec, core::ptr::addr_of_mut!((*transform).target_interest) as *mut Vec, core::ptr::addr_of_mut!(vec) as *mut Vec);
        temp_f31 = game_camera.x2C0 * (*temp_r31);
        let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(vec) as *mut Vec);
        vec.x = vec.x * temp_f31;
        vec.y = vec.y * temp_f31;
        vec.z = vec.z * temp_f31;
        let _ = lbVector_Add(core::ptr::addr_of_mut!(vec) as *mut Vec, core::ptr::addr_of_mut!((*transform).target_interest) as *mut Vec);
        (*transform).target_position = vec;
    }
}

/// `fighter_z_out_of_range`
pub unsafe fn fighter_z_out_of_range(mut fighter_pos: *mut Vec) -> i32 {
    let mut gobj: *mut HSD_GObj = null_mut();
    if Camera_80030AF8() != 0 {
        gobj = ext::Ground_GetP1Fighter();
        if !gobj.is_null() {
            ext::ftLib_80086644(gobj, fighter_pos);
            if (if (*fighter_pos).z < (0 as f32) { -(*fighter_pos).z } else { (*fighter_pos).z }) > 3e+01f32 {
                return 1;
            }
        }
    }
    return 0;
}

/// `update_zoom_distance`
pub unsafe fn update_zoom_distance() {
    let mut target_dx: f32 = 0.0;
    let mut target_dy: f32 = 0.0;
    let mut target_dz: f32 = 0.0;
    let mut dy2: f32 = 0.0;
    let mut dz2: f32 = 0.0;
    let mut dx2: f32 = 0.0;
    let mut sum: f32 = 0.0;
    if game_camera.x2BC == 1.0f32 {
        target_dx = game_camera.transform.target_position.x - game_camera.transform.target_interest.x;
        target_dy = game_camera.transform.target_position.y - game_camera.transform.target_interest.y;
        target_dz = game_camera.transform.target_position.z;
        target_dz = target_dz - game_camera.transform.target_interest.z;
        dy2 = target_dy * target_dy;
        dz2 = target_dz * target_dz;
        dx2 = target_dx * target_dx;
        game_camera.x2C0 = melee_runtime::gekko::sqrtf({ sum = (dx2 + dy2) + dz2; sum });
    }
}

/// `update_transform`
pub unsafe fn update_transform(mut bounds: *mut CameraBounds, mut ts: *mut CameraTransformState) {
    let mut fighter_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut delta: f32 = 0.0;
    Camera_8002958C(bounds, ts);
    (*ts).target_fov = cm_803BCCA0.x40;
    {
        (*ts).target_fov = ext::ScriptGame_CameraFov((*ts).target_fov);
    }
    delta = (*ts).target_fov - (*ts).fov;
    (*ts).fov = fmadds(delta, cm_803BCCA0.x44, (*ts).fov);
    Camera_80029BC4(bounds, ts);
    if !(fighter_z_out_of_range(core::ptr::addr_of_mut!(fighter_pos) as *mut Vec) != 0) {
        Camera_80029CF8(bounds, ts);
        Camera_8002A768(ts, 0);
    }
}

/// `update_avg_bounds_width`
pub unsafe fn update_avg_bounds_width() {
    let mut left_off: f32 = 0.0;
    if (game_camera.x2B8 as i32) > 1000 {
        game_camera.x2B4 = game_camera.x2B0;
        game_camera.x2B8 = 1 as i16;
    }
    left_off = cm_fly_left();
    game_camera.x2B4 = game_camera.x2B4 + (cm_fly_right() - left_off);
    game_camera.x2B8 = ((game_camera.x2B8 as i32) + 1) as i16;
    game_camera.x2B0 = game_camera.x2B4 / (game_camera.x2B8 as f32);
}

/// `update_bounds`
pub unsafe fn update_bounds(mut bounds: *mut CameraBounds, mut bounds_copy: *mut CameraBounds) {
    Camera_8002B1F8(core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState);
    Camera_80029AAC(bounds, core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState, ext::Stage_GetCamTrackSmooth());
    Camera_80029C88(bounds, core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState, ext::Stage_GetCamTrackSmooth());
    Camera_80029AAC(bounds_copy, core::ptr::addr_of_mut!(game_camera.transform_copy) as *mut CameraTransformState, ext::Stage_GetCamTrackSmooth());
    Camera_80029C88(bounds_copy, core::ptr::addr_of_mut!(game_camera.transform_copy) as *mut CameraTransformState, ext::Stage_GetCamTrackSmooth());
    Camera_UpdateQuakes(bounds);
    Camera_ApplyQuake(bounds, core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState);
}

/// `Camera_8002B3D4`
pub unsafe extern "C" fn Camera_8002B3D4(mut arg0: *mut c_void) {
    let mut bounds: CameraBounds = unsafe { core::mem::zeroed() };
    let mut bounds_copy: CameraBounds = unsafe { core::mem::zeroed() };
    Camera_80030DF8();
    Camera_800293E0();
    Camera_8002B0E0();
    update_transform(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds, core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState);
    update_transform(core::ptr::addr_of_mut!(bounds_copy) as *mut CameraBounds, core::ptr::addr_of_mut!(game_camera.transform_copy) as *mut CameraTransformState);
    update_zoom_distance();
    update_bounds(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds, core::ptr::addr_of_mut!(bounds_copy) as *mut CameraBounds);
    update_avg_bounds_width();
}

/// `get_slot_pad`
pub unsafe fn get_slot_pad(mut arg0: u8) -> *mut HSD_PadStatus {
    return core::ptr::addr_of_mut!(ext::HSD_PadCopyStatus[arg0 as usize]) as *mut HSD_PadStatus;
}

/// `get_stick_x`
pub unsafe fn get_stick_x(mut arg0: *mut HSD_PadStatus) -> f32 {
    return (*arg0).nml_stickX;
}

/// `get_stick_y`
pub unsafe fn get_stick_y(mut arg0: *mut HSD_PadStatus) -> f32 {
    return (*arg0).nml_stickY;
}

/// `Camera_8002B694`
pub unsafe fn Camera_8002B694(mut inputs: *mut CameraInputs, mut slot: i32) {
    let mut pad: *mut HSD_PadStatus = null_mut();
    let mut stick_x: f32 = 0.0;
    let mut stick_y: f32 = 0.0;
    let mut substick_x: f32 = 0.0;
    let mut substick_y: f32 = 0.0;
    let mut temp_x: f32 = 0.0;
    let mut temp_y: f32 = 0.0;
    let mut current_slot: i32 = 0;
    let mut i: i32 = 0;
    let mut idx: i32 = 0;
    let mut temp_ret: u64 = 0;
    if slot == 5 {
        (*inputs).stick_x = 0.0f32;
        (*inputs).stick_y = 0.0f32;
        (*inputs).substick_x = 0.0f32;
        (*inputs).substick_y = 0.0f32;
        (*inputs).buttons_triggered = 0 as u64;
        (*inputs).buttons_pressed = 0 as u64;
        return;
    }
    if slot == 4 {
        {
            current_slot = 0;
            'l52: while current_slot < 4 {
                pad = get_slot_pad(current_slot as u8);
                temp_x = get_stick_x(pad);
                temp_y = get_stick_y(pad);
                stick_x = temp_x;
                stick_y = temp_y;
                if temp_x < 0.0f32 {
                    temp_x = -temp_x;
                }
                if temp_x > 0.85f32 {
                    break 'l52;
                }
                if temp_y < 0.0f32 {
                    temp_y = -temp_y;
                }
                if temp_y > 0.85f32 {
                    break 'l52;
                }
                current_slot = current_slot + 1;
            }
        }
        if current_slot == 4 {
            stick_y = 0.0f32;
            stick_x = 0.0f32;
        }
        idx = 0;
        {
            i = 0;
            'l53: while i < 4 {
                pad = core::ptr::addr_of_mut!(ext::HSD_PadCopyStatus[idx as usize]) as *mut HSD_PadStatus;
                temp_x = (*pad).nml_subStickX;
                temp_y = (*pad).nml_subStickY;
                substick_x = temp_x;
                substick_y = temp_y;
                if temp_x < 0.0f32 {
                    temp_x = -temp_x;
                }
                if temp_x > 0.85f32 {
                    break 'l53;
                }
                if temp_y < 0.0f32 {
                    temp_y = -temp_y;
                }
                if temp_y > 0.85f32 {
                    break 'l53;
                }
                i = i + 1; idx = (i as u8) as i32;
            }
        }
        if idx == 4 {
            substick_y = 0.0f32;
            substick_x = 0.0f32;
        }
        (*inputs).stick_x = stick_x;
        (*inputs).stick_y = stick_y;
        (*inputs).substick_x = substick_x;
        (*inputs).substick_y = substick_y;
        temp_ret = ext::gm_GetButtonsPressed(4 as u8);
        (*inputs).buttons_pressed = temp_ret;
        temp_ret = ext::gm_GetButtonsTriggered(4 as u8);
        (*inputs).buttons_triggered = temp_ret;
        return;
    }
    pad = get_slot_pad(slot as u8);
    (*inputs).stick_x = (*pad).nml_stickX;
    (*inputs).stick_y = (*pad).nml_stickY;
    (*inputs).substick_x = (*pad).nml_subStickX;
    (*inputs).substick_y = (*pad).nml_subStickY;
    temp_ret = ext::gm_GetButtonsPressed(slot as u8);
    (*inputs).buttons_pressed = temp_ret;
    temp_ret = ext::gm_GetButtonsTriggered(slot as u8);
    (*inputs).buttons_triggered = temp_ret;
}

/// `Camera_8002BA00`
pub unsafe fn Camera_8002BA00(mut slot: i32, mut arg1: i32) -> i32 {
    if slot == 10 {
        if arg1 > 0 {
            slot = 0;
        } else if arg1 < 0 {
            slot = 5;
        }
    } else {
        slot = slot + arg1;
        if slot < 0 {
            slot = 10;
        } else if slot > 5 {
            slot = 10;
        }
    }
    if slot == 10 {
        return 10;
    }
    if !ext::Player_GetEntity(slot).is_null() {
        return slot;
    }
    return Camera_8002BA00(slot, arg1);
}

/// `Camera_8002BAA8`
pub unsafe fn Camera_8002BAA8(mut zoom_amt: f32) {
    let mut offset: Vec3 = unsafe { core::mem::zeroed() };
    let mut offset_len: f32 = 0.0;
    let mut dist: f32 = 0.0;
    offset = game_camera.pause_eye_offset;
    offset.x = offset.x * (-1.0f32);
    offset.y = offset.y * (-1.0f32);
    offset.z = offset.z * (-1.0f32);
    offset_len = vec_len(core::ptr::addr_of_mut!(offset) as *mut Vec);
    if offset_len < 1.0f32 {
        offset.y = 0.0f32;
        offset.x = 0.0f32;
        offset.z = 1.0f32;
        game_camera.pause_eye_distance = 1e+01f32;
    }
    game_camera.pause_eye_distance = fmadds(zoom_amt, fmadds(offset_len, cm_803BCCA0.x9C, cm_803BCCA0.xA0), game_camera.pause_eye_distance);
    if game_camera.pause_eye_distance < game_camera.x2D0.unk28 {
        game_camera.pause_eye_distance = game_camera.x2D0.unk28;
    } else if game_camera.pause_eye_distance > game_camera.x2D0.unk2C {
        game_camera.pause_eye_distance = game_camera.x2D0.unk2C;
    }
    dist = game_camera.pause_eye_distance;
    let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(offset) as *mut Vec);
    offset.x = offset.x * dist;
    offset.y = offset.y * dist;
    offset.z = offset.z * dist;
    offset.x = offset.x * (-1.0f32);
    offset.y = offset.y * (-1.0f32);
    offset.z = offset.z * (-1.0f32);
    game_camera.pause_eye_offset = offset;
}

/// `OrthonormalizeBasis`
pub unsafe fn OrthonormalizeBasis(mut forward: *mut Vec, mut up: *mut Vec, mut right: *mut Vec) {
    ext::PSVECCrossProduct(up, forward, right);
    let _ = ext::lbVector_Normalize(right);
    ext::PSVECCrossProduct(forward, right, up);
    let _ = ext::lbVector_Normalize(up);
}

/// `Camera_8002BC78`
pub unsafe fn Camera_8002BC78(mut forward: *mut Vec, mut up: *mut Vec, mut right: *mut Vec) -> i32 {
    let mut clamp_result: i32 = 0;
    if (*forward).y > 0.999f32 {
        clamp_result = 1;
        (*forward).y = 1.0f32;
        (*forward).z = 0.0f32;
        (*forward).x = 0.0f32;
        *up = game_camera.pause_up;
    } else if (*forward).y < (-0.999f32) {
        clamp_result = (-1_i32);
        (*forward).y = -1.0f32;
        (*forward).z = 0.0f32;
        (*forward).x = 0.0f32;
        *up = game_camera.pause_up;
    }
    OrthonormalizeBasis(forward, up, right);
    return clamp_result;
}

/// `Camera_8002BD88`
pub unsafe fn Camera_8002BD88(mut x: f32, mut y: f32) {
    let mut scale: f32 = 0.0;
    let mut view_dir: f32 = 0.0;
    let mut clamp_result: i32 = 0;
    let mut up: Vec3 = unsafe { core::mem::zeroed() };
    let mut forward: Vec3 = unsafe { core::mem::zeroed() };
    let mut right: Vec3 = unsafe { core::mem::zeroed() };
    let mut pos: Vec3 = unsafe { core::mem::zeroed() };
    up = cm_WorldUp;
    pos = game_camera.pause_eye_offset;
    forward = pos;
    forward.x = forward.x * (-1.0f32);
    forward.y = forward.y * (-1.0f32);
    forward.z = forward.z * (-1.0f32);
    view_dir = ext::lbVector_Normalize(core::ptr::addr_of_mut!(forward) as *mut Vec);
    if view_dir < 1.0f32 {
        forward.y = 0.0f32;
        forward.x = 0.0f32;
        forward.z = 1.0f32;
        game_camera.pause_eye_distance = 1e+01f32;
    }
    clamp_result = Camera_8002BC78(core::ptr::addr_of_mut!(forward) as *mut Vec, core::ptr::addr_of_mut!(up) as *mut Vec, core::ptr::addr_of_mut!(right) as *mut Vec);
    if clamp_result == 1 {
        if y < 0.0f32 {
            y = 0.0f32;
        }
        x = 0.0f32;
    } else if clamp_result == (-1_i32) {
        if y > 0.0f32 {
            y = 0.0f32;
        }
        x = 0.0f32;
    }
    OrthonormalizeBasis(core::ptr::addr_of_mut!(forward) as *mut Vec, core::ptr::addr_of_mut!(up) as *mut Vec, core::ptr::addr_of_mut!(right) as *mut Vec);
    game_camera.pause_up = up;
    if y != 0.0f32 {
        scale = y * fmadds(view_dir, cm_803BCCA0.xBC, cm_803BCCA0.xC0);
        up.x = up.x * scale;
        up.y = up.y * scale;
        up.z = up.z * scale;
        let _ = lbVector_Add(core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(up) as *mut Vec);
    }
    if x != 0.0f32 {
        scale = x * fmadds(view_dir, cm_803BCCA0.xBC, cm_803BCCA0.xC0);
        right.x = right.x * (-scale);
        right.y = right.y * (-scale);
        right.z = right.z * (-scale);
        let _ = lbVector_Add(core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(right) as *mut Vec);
    }
    scale = game_camera.pause_eye_distance;
    let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(pos) as *mut Vec);
    pos.x = pos.x * scale;
    pos.y = pos.y * scale;
    pos.z = pos.z * scale;
    game_camera.pause_eye_offset = pos;
}

/// `Camera_8002C010`
pub unsafe fn Camera_8002C010(mut farg0: f32, mut farg1: f32) {
    let mut up: Vec3 = unsafe { core::mem::zeroed() };
    let mut forward: Vec3 = unsafe { core::mem::zeroed() };
    let mut right: Vec3 = unsafe { core::mem::zeroed() };
    let mut eye_dist: f32 = 0.0;
    let mut scale: f32 = 0.0;
    let mut movement_scale: f32 = 0.0;
    up = cm_803B73DC;
    forward = game_camera.pause_eye_offset;
    forward.x = forward.x * (-1.0f32);
    forward.y = forward.y * (-1.0f32);
    forward.z = forward.z * (-1.0f32);
    eye_dist = ext::lbVector_Normalize(core::ptr::addr_of_mut!(forward) as *mut Vec);
    if eye_dist < 1.0f32 {
        eye_dist = 1.0f32;
        forward.y = 0.0f32;
        forward.x = 0.0f32;
        forward.z = 1.0f32;
        game_camera.pause_eye_distance = 1.0f32;
    }
    let _ = Camera_8002BC78(core::ptr::addr_of_mut!(forward) as *mut Vec, core::ptr::addr_of_mut!(up) as *mut Vec, core::ptr::addr_of_mut!(right) as *mut Vec);
    if farg1 != 0.0f32 {
        movement_scale = fmadds(eye_dist, cm_803BCCA0.xB4, cm_803BCCA0.xB8);
        scale = farg1 * movement_scale;
        up.x = up.x * scale;
        up.y = up.y * scale;
        up.z = up.z * scale;
        let _ = lbVector_Add(core::ptr::addr_of_mut!(game_camera.x314) as *mut Vec, core::ptr::addr_of_mut!(up) as *mut Vec);
    }
    if farg0 != 0.0f32 {
        scale = farg0 * fmadds(eye_dist, cm_803BCCA0.xB4, cm_803BCCA0.xB8);
        right.x = right.x * (-scale);
        right.y = right.y * (-scale);
        right.z = right.z * (-scale);
        let _ = lbVector_Add(core::ptr::addr_of_mut!(game_camera.x314) as *mut Vec, core::ptr::addr_of_mut!(right) as *mut Vec);
    }
}

/// `Camera_8002C1A8_inline`
pub unsafe fn Camera_8002C1A8_inline() {
    if lbVector_Len(core::ptr::addr_of_mut!(game_camera.pause_eye_offset) as *mut Vec) < 1.0f32 {
        game_camera.pause_eye_distance = 1.0f32;
    }
}

/// `Camera_8002C1A8`
pub unsafe fn Camera_8002C1A8() {
    let mut inputs: CameraInputs = unsafe { core::mem::zeroed() };
    let mut stick_x: f32 = 0.0;
    let mut stick_y: f32 = 0.0;
    let mut zoom_dir: f32 = 0.0;
    let mut x_move: f32 = 0.0;
    let mut y_move: f32 = 0.0;
    let mut substick_x: f32 = 0.0;
    let mut substick_y: f32 = 0.0;
    let mut substick_x_val: f32 = 0.0;
    let mut substick_y_val: f32 = 0.0;
    let mut scale: f32 = 0.0;
    let mut dir: i32 = 0;
    let mut slot: i8 = 0;
    if (game_camera.x305 as i32) == 5 {
        return;
    }
    Camera_8002B694(core::ptr::addr_of_mut!(inputs) as *mut CameraInputs, game_camera.x305 as i32);
    zoom_dir = { x_move = { y_move = { substick_x_val = { substick_y_val = 0.0f32; substick_y_val }; substick_x_val }; y_move }; x_move };
    stick_x = inputs.stick_x;
    stick_y = inputs.stick_y;
    substick_x = inputs.substick_x;
    substick_y = inputs.substick_y;
    dir = 0;
    {
        let mut triggered: u64 = inputs.buttons_triggered;
        let mut pressed: u64 = inputs.buttons_pressed;
        if (triggered & ((1 << 5) as u64)) != (0 as u64) {
            dir = 1;
        } else if (triggered & ((1 << 6) as u64)) != (0 as u64) {
            dir = (-1_i32);
        }
        if (pressed & ((1 << 3) as u64)) != (0 as u64) {
            y_move = 1.0f32;
        } else if (pressed & ((1 << 2) as u64)) != (0 as u64) {
            y_move = -1.0f32;
        }
        if (pressed & ((1 << 0) as u64)) != (0 as u64) {
            x_move = -1.0f32;
        } else if (pressed & ((1 << 1) as u64)) != (0 as u64) {
            x_move = 1.0f32;
        }
        if (pressed & ((1 << 10) as u64)) != (0 as u64) {
            zoom_dir = 1.0f32;
        } else if (pressed & ((1 << 11) as u64)) != (0 as u64) {
            zoom_dir = -1.0f32;
        }
        if (pressed & ((1 << 8) as u64)) != (0 as u64) {
            if ((if stick_x < (0 as f32) { -stick_x } else { stick_x }) as f64) > 0.125f64 {
                x_move = stick_x;
            }
            if ((if stick_y < (0 as f32) { -stick_y } else { stick_y }) as f64) > 0.125f64 {
                y_move = stick_y;
            }
            stick_y = 0.0f32;
        }
    }
    if ((if stick_y < (0 as f32) { -stick_y } else { stick_y }) as f64) > 0.125f64 {
        zoom_dir = -stick_y;
    }
    if ((if substick_x < (0 as f32) { -substick_x } else { substick_x }) as f64) > 0.125f64 {
        substick_x_val = substick_x;
    }
    if ((if substick_y < (0 as f32) { -substick_y } else { substick_y }) as f64) > 0.125f64 {
        substick_y_val = substick_y;
    }
    if dir != 0 {
        scale = fmadds(game_camera.x32C, cm_803BCCA0.x8C, cm_803BCCA0.x90);
        game_camera.x304 = Camera_8002BA00(game_camera.x304 as i32, dir) as i8;
        slot = game_camera.x304;
        game_camera.x314.x = { game_camera.x314.y = { game_camera.x314.z = 0.0f32; game_camera.x314.z }; game_camera.x314.y };
        game_camera.pause_eye_offset.x = 0.0f32;
        game_camera.pause_eye_offset.y = 5.0f32;
        game_camera.pause_eye_offset.z = 2e+01f32;
        game_camera.pause_up.x = 0.0f32;
        game_camera.pause_up.y = 1.0f32;
        game_camera.pause_up.z = 0.0f32;
        if (slot as i32) == 10 {
            game_camera.pause_eye_distance = 3.0f32 * scale;
        } else {
            game_camera.pause_eye_distance = scale;
        }
        Camera_8002BAA8(0.0f32);
    }
    if zoom_dir != 0.0f32 {
        Camera_8002BAA8(zoom_dir);
    }
    if (x_move != 0.0f32) || (y_move != 0.0f32) {
        if (game_camera.x304 as i32) == 10 {
            Camera_8002C1A8_inline();
            if y_move != 0.0f32 {
                game_camera.x314.y = game_camera.x314.y + y_move;
            }
            if x_move != 0.0f32 {
                game_camera.x314.x = game_camera.x314.x + x_move;
            }
        } else {
            Camera_8002C010(x_move, y_move);
        }
    }
    if (substick_x_val != 0.0f32) || (substick_y_val != 0.0f32) {
        Camera_8002BD88(substick_x_val, substick_y_val);
    }
}

/// `eye_offset_len`
pub unsafe fn eye_offset_len(mut offset: *mut Vec) -> f32 {
    return melee_runtime::gekko::sqrtf(fmadds((*offset).z, (*offset).z, fmadds((*offset).x, (*offset).x, (*offset).y * (*offset).y)));
}

/// `Camera_8002C5B4`
pub unsafe fn Camera_8002C5B4(mut arg0: *mut Camera_x2D0) {
    let mut len: f64 = 0.0;
    let mut temp_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut cross1: Vec3 = unsafe { core::mem::zeroed() };
    let mut cross2: Vec3 = unsafe { core::mem::zeroed() };
    let mut cam: *mut Camera = null_mut();
    let mut params: *mut Camera_x2D0 = null_mut();
    let mut cam_pos: *mut Vec = null_mut();
    let mut cam_offset: *mut Vec = null_mut();
    let mut eye_offset_z: *mut f32 = null_mut();
    let mut eye_offset: *mut Vec = null_mut();
    let mut eye_offset_y: *mut f32 = null_mut();
    let mut y_ptr: *mut f32 = null_mut();
    let mut z_ptr: *mut f32 = null_mut();
    let mut xz_dist: f32 = 0.0;
    let mut pitch: f32 = 0.0;
    let mut yaw: f32 = 0.0;
    let mut pos: f32 = 0.0;
    let mut limit: f32 = 0.0;
    cam = core::ptr::addr_of_mut!(game_camera) as *mut Camera;
    params = arg0;
    if (*params).callback.is_some() {
        (*params).callback.unwrap()(arg0);
    }
    limit = (*params).x_max;
    pos = (*{ cam_pos = core::ptr::addr_of_mut!((*cam).x308) as *mut Vec; cam_pos }).x;
    if pos > limit {
        (*cam_pos).x = limit;
    } else {
        limit = (*params).x_min;
        if pos < limit {
            (*cam_pos).x = limit;
        }
    }
    limit = (*params).y_max;
    pos = *{ y_ptr = core::ptr::addr_of_mut!((*cam).x308.y) as *mut f32; y_ptr };
    if pos > limit {
        *y_ptr = limit;
    } else {
        limit = (*params).y_min;
        if pos < limit {
            *y_ptr = limit;
        }
    }
    limit = (*params).z_max;
    pos = *{ z_ptr = core::ptr::addr_of_mut!((*cam).x308.z) as *mut f32; z_ptr };
    if pos > limit {
        *z_ptr = limit;
    } else {
        limit = (*params).z_min;
        if pos < limit {
            *z_ptr = limit;
        }
    }
    temp_pos = *{ cam_offset = core::ptr::addr_of_mut!((*cam).x314) as *mut Vec; cam_offset };
    let _ = lbVector_Add(core::ptr::addr_of_mut!(temp_pos) as *mut Vec, cam_pos);
    if temp_pos.x > (*params).x_max {
        temp_pos.x = (*params).x_max;
    } else if temp_pos.x < (*params).x_min {
        temp_pos.x = (*params).x_min;
    }
    if temp_pos.y > (*params).y_max {
        temp_pos.y = (*params).y_max;
    } else if temp_pos.y < (*params).y_min {
        temp_pos.y = (*params).y_min;
    }
    if temp_pos.z > (*params).z_max {
        temp_pos.z = (*params).z_max;
    } else if temp_pos.z < (*params).z_min {
        temp_pos.z = (*params).z_min;
    }
    let _ = ext::lbVector_Sub(core::ptr::addr_of_mut!(temp_pos) as *mut Vec, cam_pos);
    *cam_offset = temp_pos;
    eye_offset = core::ptr::addr_of_mut!((*cam).pause_eye_offset) as *mut Vec;
    eye_offset_y = core::ptr::addr_of_mut!((*cam).pause_eye_offset.y) as *mut f32;
    eye_offset_z = core::ptr::addr_of_mut!((*cam).pause_eye_offset.z) as *mut f32;
    len = eye_offset_len(eye_offset) as f64;
    xz_dist = melee_runtime::gekko::sqrtf(fmadds((*eye_offset).x, (*eye_offset).x, (*eye_offset_z) * (*eye_offset_z)));
    ext::PSVECCrossProduct(eye_offset, core::ptr::addr_of_mut!((*cam).pause_up) as *mut Vec, core::ptr::addr_of_mut!(cross1) as *mut Vec);
    let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(cross1) as *mut Vec);
    ext::PSVECCrossProduct(core::ptr::addr_of_mut!(cross1) as *mut Vec, eye_offset, core::ptr::addr_of_mut!(cross2) as *mut Vec);
    let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(cross2) as *mut Vec);
    pitch = ext::atan2f(*eye_offset_y, xz_dist);
    if pitch > (*params).angle_down {
        ext::lbVector_RotateAboutUnitAxis(eye_offset, core::ptr::addr_of_mut!(cross1) as *mut Vec, (*params).angle_down - pitch);
    } else if pitch < (-(*params).angle_up) {
        ext::lbVector_RotateAboutUnitAxis(eye_offset, core::ptr::addr_of_mut!(cross1) as *mut Vec, (-(*params).angle_up) - pitch);
    }
    yaw = ext::atan2f((*eye_offset).x, *eye_offset_z);
    if yaw > (*params).angle_right {
        ext::lbVector_RotateAboutUnitAxis(eye_offset, core::ptr::addr_of_mut!(cross2) as *mut Vec, (*params).angle_right - yaw);
    } else if yaw < (-(*params).angle_left) {
        ext::lbVector_RotateAboutUnitAxis(eye_offset, core::ptr::addr_of_mut!(cross2) as *mut Vec, (-(*params).angle_left) - yaw);
    }
}

/// `get_subject_pos`
pub unsafe fn get_subject_pos(mut pos: *mut Vec, mut slot_ptr: *const i8) -> i32 {
    let mut subject: *mut CmSubject = null_mut();
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut slot: i32 = 0;
    let mut valid: i32 = 0;
    slot = (*slot_ptr) as i32;
    valid = 1;
    if slot == 11 {
        valid = 0;
    } else if slot == 10 {
        ext::Stage_UnkSetVec3TCam_Offset(pos);
    } else {
        gobj = ext::Player_GetEntity(slot);
        if (!gobj.is_null()) && (!{ subject = ext::ftLib_80086B74(gobj); subject }.is_null()) {
            *pos = (*subject).bone_pos;
        } else {
            valid = 0;
        }
    }
    return valid;
}

/// `get_subject_pos_out`
pub unsafe fn get_subject_pos_out(mut pos: *mut Vec, mut slot_ptr: *const i8, mut valid_out: *mut i32) {
    let mut subject: *mut CmSubject = null_mut();
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut slot: i32 = 0;
    slot = (*slot_ptr) as i32;
    *valid_out = 1;
    if slot == 11 {
        *valid_out = 0;
    } else if slot == 10 {
        ext::Stage_UnkSetVec3TCam_Offset(pos);
    } else {
        gobj = ext::Player_GetEntity(slot);
        if (!gobj.is_null()) && (!{ subject = ext::ftLib_80086B74(gobj); subject }.is_null()) {
            *pos = (*subject).bone_pos;
        } else {
            *valid_out = 0;
        }
    }
}

/// `camera_cddc_select`
pub unsafe fn camera_cddc_select(mut slot_ptr: *mut i8) {
    let mut pos_ptr: *mut Vec = null_mut();
    let mut valid: i32 = 0;
    let mut gobj: *mut HSD_GObj = null_mut();
    if ((*slot_ptr) as i32) == 11 {
        return;
    }
    pos_ptr = core::ptr::addr_of_mut!(game_camera.x308) as *mut Vec;
    'l54: while (((((*slot_ptr) as i32) == 10) || ({ get_subject_pos_out(pos_ptr, slot_ptr as *const i8, core::ptr::addr_of_mut!(valid) as *mut i32); !(valid != 0) })) || ({ gobj = ext::Player_GetEntity((*slot_ptr) as i32); gobj }.is_null())) || (ext::ftLib_8008701C(gobj) != 0) {
        *slot_ptr = Camera_8002BA00((*slot_ptr) as i32, 1) as i8;
    }
}

/// `camera_cddc_target_interest`
pub unsafe fn camera_cddc_target_interest(mut transform: *mut CameraTransformState) -> *mut Vec {
    return core::ptr::addr_of_mut!((*transform).target_interest) as *mut Vec;
}

/// `track_subject`
pub unsafe fn track_subject(mut transform: *mut CameraTransformState, mut interest_diff: *mut Vec, mut eye_diff: *mut Vec) {
    let mut copy_src: *mut Vec = null_mut();
    let mut target_pos: *mut Vec = null_mut();
    let mut position_ptr: *mut Vec = null_mut();
    let mut coeff_ptr: *mut f32 = null_mut();
    let mut coeff: f32 = 0.0;
    Camera_8002C5B4(core::ptr::addr_of_mut!(game_camera.x2D0) as *mut Camera_x2D0);
    copy_src = core::ptr::addr_of_mut!(game_camera.transform.target_interest) as *mut Vec;
    *copy_src = game_camera.x308;
    let _ = lbVector_Add(camera_cddc_target_interest(transform), core::ptr::addr_of_mut!(game_camera.x314) as *mut Vec);
    game_camera.transform.target_position = *copy_src;
    target_pos = core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec;
    let _ = lbVector_Add(target_pos, core::ptr::addr_of_mut!(game_camera.pause_eye_offset) as *mut Vec);
    position_ptr = core::ptr::addr_of_mut!((*transform).position) as *mut Vec;
    let _ = lbVector_Diff(target_pos, position_ptr, eye_diff);
    coeff_ptr = core::ptr::addr_of_mut!(cm_803BCCA0.x84) as *mut f32;
    coeff = *coeff_ptr;
    (*eye_diff).x = (*eye_diff).x * coeff;
    (*eye_diff).y = (*eye_diff).y * coeff;
    (*eye_diff).z = (*eye_diff).z * coeff;
    let _ = lbVector_Add(position_ptr, eye_diff);
    let _ = lbVector_Diff(camera_cddc_target_interest(transform), core::ptr::addr_of_mut!((*transform).interest) as *mut Vec, interest_diff);
    coeff = *coeff_ptr;
    (*interest_diff).x = (*interest_diff).x * coeff;
    (*interest_diff).y = (*interest_diff).y * coeff;
    (*interest_diff).z = (*interest_diff).z * coeff;
    let _ = lbVector_Add(core::ptr::addr_of_mut!((*transform).interest) as *mut Vec, interest_diff);
}

/// `Camera_8002C908`
pub unsafe extern "C" fn Camera_8002C908(mut arg0: *mut c_void) {
    let mut bounds: CameraBounds = unsafe { core::mem::zeroed() };
    let mut pad: [u8; 24] = [0; 24];
    let mut eye_diff: Vec3 = unsafe { core::mem::zeroed() };
    let mut interest_diff: Vec3 = unsafe { core::mem::zeroed() };
    let mut transform: *mut CameraTransformState = null_mut();
    let mut slot_ptr: *mut i8 = null_mut();
    let mut subject_pos: *mut Vec = null_mut();
    let mut copy_src: *mut Vec = null_mut();
    let mut target_pos: *mut Vec = null_mut();
    let mut position_ptr: *mut Vec = null_mut();
    let mut coeff_ptr: *mut f32 = null_mut();
    let mut target_interest: *mut Vec = null_mut();
    let mut coeff: f32 = 0.0;
    let mut delta: f32 = 0.0;
    Camera_80030DF8();
    Camera_800293E0();
    transform = core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState;
    Camera_8002958C(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds, transform);
    Camera_8002C1A8();
    subject_pos = core::ptr::addr_of_mut!(game_camera.x308) as *mut Vec;
    slot_ptr = core::ptr::addr_of_mut!(game_camera.x304) as *mut i8;
    'l55: while !(get_subject_pos(subject_pos, slot_ptr as *const i8) != 0) {
        *slot_ptr = Camera_8002BA00((*slot_ptr) as i32, 1) as i8;
    }
    Camera_8002C5B4(core::ptr::addr_of_mut!(game_camera.x2D0) as *mut Camera_x2D0);
    target_interest = core::ptr::addr_of_mut!((*transform).target_interest) as *mut Vec;
    copy_src = core::ptr::addr_of_mut!(game_camera.transform.target_interest) as *mut Vec;
    *copy_src = *subject_pos;
    let _ = lbVector_Add(target_interest, core::ptr::addr_of_mut!(game_camera.x314) as *mut Vec);
    game_camera.transform.target_position = *copy_src;
    target_pos = core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec;
    let _ = lbVector_Add(target_pos, core::ptr::addr_of_mut!(game_camera.pause_eye_offset) as *mut Vec);
    position_ptr = core::ptr::addr_of_mut!((*transform).position) as *mut Vec;
    let _ = lbVector_Diff(target_pos, position_ptr, core::ptr::addr_of_mut!(eye_diff) as *mut Vec);
    coeff_ptr = core::ptr::addr_of_mut!(cm_803BCCA0.x84) as *mut f32;
    coeff = *coeff_ptr;
    eye_diff.x = eye_diff.x * coeff;
    eye_diff.y = eye_diff.y * coeff;
    eye_diff.z = eye_diff.z * coeff;
    let _ = lbVector_Add(position_ptr, core::ptr::addr_of_mut!(eye_diff) as *mut Vec);
    let _ = lbVector_Diff(target_interest, core::ptr::addr_of_mut!((*transform).interest) as *mut Vec, core::ptr::addr_of_mut!(interest_diff) as *mut Vec);
    coeff = *coeff_ptr;
    interest_diff.x = interest_diff.x * coeff;
    interest_diff.y = interest_diff.y * coeff;
    interest_diff.z = interest_diff.z * coeff;
    let _ = lbVector_Add(core::ptr::addr_of_mut!((*transform).interest) as *mut Vec, core::ptr::addr_of_mut!(interest_diff) as *mut Vec);
    game_camera.transform.target_fov = game_camera.x32C;
    delta = game_camera.transform.target_fov - game_camera.transform.fov;
    game_camera.transform.fov = fmadds(delta, cm_803BCCA0.x88, game_camera.transform.fov);
    Camera_UpdateQuakes(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds);
    Camera_ApplyQuake(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds, core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState);
}

/// `Camera_8002CB0C`
pub unsafe fn Camera_8002CB0C(mut bounds: *mut CameraBounds) {
    let mut camera: *mut Camera = null_mut();
    let mut inputs: CameraInputs = unsafe { core::mem::zeroed() };
    let mut entity: *mut HSD_GObj = null_mut();
    let mut selected_slot: i32 = 0;
    let mut slot: *mut i8 = null_mut();
    let mut dir: i32 = 0;
    let mut x308_ptr: *mut Vec = null_mut();
    let mut x_val: f32 = 0.0;
    let mut y_val: f32 = 0.0;
    let mut stick_x: f32 = 0.0;
    let mut zoom_dir: f32 = 0.0;
    let mut stick_y: f32 = 0.0;
    let mut z_init: f32 = 0.0;
    let mut abs_f1: f32 = 0.0;
    camera = core::ptr::addr_of_mut!(game_camera) as *mut Camera;
    {
        let mut pauser_slot: i8 = (*camera).x2C5;
        if (pauser_slot as i32) == 5 {
            return;
        }
        Camera_8002B694(core::ptr::addr_of_mut!(inputs) as *mut CameraInputs, pauser_slot as i32);
    }
    zoom_dir = { x_val = { y_val = 0.0f32; y_val }; x_val };
    dir = 0;
    stick_x = inputs.stick_x;
    stick_y = inputs.stick_y;
    {
        let mut newly_pressed_btns: u64 = inputs.buttons_triggered;
        let mut pressed_btns: u64 = inputs.buttons_pressed;
        if (newly_pressed_btns & ((1 << 5) as u64)) != (0 as u64) {
            dir = 1;
        } else if (newly_pressed_btns & ((1 << 6) as u64)) != (0 as u64) {
            dir = (-1_i32);
        }
        if (pressed_btns & ((1 << 10) as u64)) != (0 as u64) {
            zoom_dir = 1.0f32;
        } else if (pressed_btns & ((1 << 11) as u64)) != (0 as u64) {
            zoom_dir = -1.0f32;
        }
    }
    abs_f1 = if stick_x < (0 as f32) { -stick_x } else { stick_x };
    if (abs_f1 as f64) > 0.125f64 {
        x_val = stick_x;
    }
    abs_f1 = if stick_y < (0 as f32) { -stick_y } else { stick_y };
    if (abs_f1 as f64) > 0.125f64 {
        y_val = stick_y;
    }
    if dir != 0 {
        slot = core::ptr::addr_of_mut!((*camera).x2C4) as *mut i8;
        *slot = Camera_8002BA00((*slot) as i32, dir) as i8;
        x308_ptr = core::ptr::addr_of_mut!((*camera).x308) as *mut Vec;
        'l56: while (((((*slot) as i32) == 10) || (!(get_subject_pos(x308_ptr, slot as *const i8) != 0))) || ({ entity = ext::Player_GetEntity((*slot) as i32); entity }.is_null())) || (ext::ftLib_8008701C(entity) != 0) {
            *slot = Camera_8002BA00((*slot) as i32, dir) as i8;
        }
        selected_slot = (*slot) as i32;
        z_init = ext::Stage_GetPauseCamZPosInit();
        (*camera).x314.x = { (*camera).x314.y = { (*camera).x314.z = 0.0f32; (*camera).x314.z }; (*camera).x314.y };
        (*camera).pause_eye_offset.x = 0.0f32;
        (*camera).pause_eye_offset.y = 5.0f32;
        (*camera).pause_eye_offset.z = 2e+01f32;
        (*camera).pause_up.x = 0.0f32;
        (*camera).pause_up.y = 1.0f32;
        (*camera).pause_up.z = 0.0f32;
        if selected_slot == 10 {
            (*camera).pause_eye_distance = 3.0f32 * z_init;
        } else {
            (*camera).pause_eye_distance = z_init;
        }
        Camera_8002BAA8(0.0f32);
    }
    if zoom_dir != 0.0f32 {
        Camera_8002BAA8(zoom_dir);
    }
    if !((x_val == 0.0f32) && (y_val == 0.0f32)) {
        Camera_8002BD88(x_val, y_val);
    }
}

/// `Camera_8002CDDC`
pub unsafe extern "C" fn Camera_8002CDDC(mut unused: *mut c_void) {
    let mut slot_ptr: *mut i8 = null_mut();
    let mut late_transform: *mut CameraTransformState = null_mut();
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut subject: *mut CmSubject = null_mut();
    let mut bounds: CameraBounds = unsafe { core::mem::zeroed() };
    let mut eye_diff: Vec3 = unsafe { core::mem::zeroed() };
    let mut interest_diff: Vec3 = unsafe { core::mem::zeroed() };
    let mut delta: f32 = 0.0;
    let mut bounds_copy: CameraBounds = unsafe { core::mem::zeroed() };
    let mut bounds2: CameraBounds = unsafe { core::mem::zeroed() };
    Camera_80030DF8();
    Camera_800293E0();
    Camera_8002958C(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds, core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState);
    slot_ptr = core::ptr::addr_of_mut!(game_camera.x2C4) as *mut i8;
    late_transform = core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState;
    camera_cddc_select(slot_ptr);
    Camera_8002CB0C(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds);
    if (((((((((*slot_ptr) as i32) != 10) && (((*slot_ptr) as i32) != 11)) && (!{ gobj = ext::Player_GetEntity((*slot_ptr) as i32); gobj }.is_null())) && (!{ subject = ext::ftLib_80086B74(gobj); subject }.is_null())) && (Camera_8002928C(subject) != 0)) && ((*subject).state == ((CmSubjectState_Active as i32) as u32))) && (Camera_80029124(core::ptr::addr_of_mut!((*subject).bone_pos) as *mut Vec, 0) == (0 as u32))) && ((if (*subject).bone_pos.z < (0 as f32) { -(*subject).bone_pos.z } else { (*subject).bone_pos.z }) < 3e+01f32) {
        track_subject(late_transform, core::ptr::addr_of_mut!(interest_diff) as *mut Vec, core::ptr::addr_of_mut!(eye_diff) as *mut Vec);
        game_camera.transform.target_fov = cm_803BCCA0.x6C;
        delta = game_camera.transform.target_fov - game_camera.transform.fov;
        game_camera.transform.fov = fmadds(delta, cm_803BCCA0.x70, game_camera.transform.fov);
        return;
    }
    Camera_80030DF8();
    Camera_800293E0();
    Camera_8002B0E0();
    update_transform(core::ptr::addr_of_mut!(bounds2) as *mut CameraBounds, core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState);
    update_transform(core::ptr::addr_of_mut!(bounds_copy) as *mut CameraBounds, core::ptr::addr_of_mut!(game_camera.transform_copy) as *mut CameraTransformState);
    update_zoom_distance();
    update_bounds(core::ptr::addr_of_mut!(bounds2) as *mut CameraBounds, core::ptr::addr_of_mut!(bounds_copy) as *mut CameraBounds);
    update_avg_bounds_width();
}

/// `compute_orbit_distance`
pub unsafe fn compute_orbit_distance(mut slot: i32) -> f32 {
    let mut distance: f32 = 0.0;
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut subject: *mut CmSubject = null_mut();
    if (((((slot != 10) && (slot != 11)) && (slot >= 0)) && (slot < 6)) && (!{ gobj = ext::Player_GetEntity(slot); gobj }.is_null())) && (!{ subject = ext::ftLib_80086B74(gobj); subject }.is_null()) {
        distance = (2.0f32 * (*subject).ext.v.z) / ext::tanf(game_camera.transform.target_fov * 0.017453292f32);
    } else {
        distance = 1e+03f32;
    }
    return distance;
}

/// `get_subject_x1C`
pub unsafe fn get_subject_x1C(mut subject: *mut CmSubject) -> *mut Vec {
    return core::ptr::addr_of_mut!((*subject).bone_pos) as *mut Vec;
}

/// `Camera_8002D318`
pub unsafe extern "C" fn Camera_8002D318(mut unused: *mut c_void) {
    let mut pos: *mut Vec = null_mut();
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut half_z: f32 = 0.0;
    let mut horiz_dist: f32 = 0.0;
    let mut distance: f32 = 0.0;
    let mut delta: f32 = 0.0;
    let mut smooth: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut dz: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut smooth2: f32 = 0.0;
    let mut dx2: f32 = 0.0;
    let mut dz2: f32 = 0.0;
    let mut dy2: f32 = 0.0;
    let mut _pad: [u8; 16] = [0; 16];
    let mut subject: *mut CmSubject = null_mut();
    let mut bounds_copy: CameraBounds = unsafe { core::mem::zeroed() };
    let mut bounds: CameraBounds = unsafe { core::mem::zeroed() };
    gobj = ext::Player_GetEntity(game_camera.x2C4 as i32);
    if (((((!gobj.is_null()) && (ext::ftLib_8008701C(gobj) == 0)) && (!{ subject = ext::ftLib_80086B74(gobj); subject }.is_null())) && (Camera_8002928C(subject) != 0)) && (Camera_80029124(core::ptr::addr_of_mut!((*subject).bone_pos) as *mut Vec, 0) == (0 as u32))) && ((if (*subject).bone_pos.z < (0 as f32) { -(*subject).bone_pos.z } else { (*subject).bone_pos.z }) < 1e+01f32) {
        Camera_80030DF8();
        if (!{ gobj = ext::Player_GetEntity(game_camera.x2C4 as i32); gobj }.is_null()) && (!{ subject = ext::ftLib_80086B74(gobj); subject }.is_null()) {
            pos = get_subject_x1C(subject);
            half_z = 0.5f32 * (*subject).ext.v.z;
            ext::diag_camera_subject((*pos).x, (*pos).y, (*pos).z, (*subject).ext.v.z, game_camera.yaw_offset);
            game_camera.transform.target_interest.x = -fmadds(half_z, melee_runtime::gekko::cosf(game_camera.yaw_offset), -(*pos).x);
            game_camera.transform.target_interest.y = (*pos).y;
            game_camera.transform.target_interest.z = fmadds(half_z, melee_runtime::gekko::sinf(game_camera.yaw_offset), (*pos).z);
            dx = game_camera.transform.target_interest.x - game_camera.transform.interest.x;
            smooth = cm_803BCCA0.x64;
            dy = game_camera.transform.target_interest.y - game_camera.transform.interest.y;
            dz = game_camera.transform.target_interest.z - game_camera.transform.interest.z;
            game_camera.transform.interest.x = fmadds(dx, smooth, game_camera.transform.interest.x);
            game_camera.transform.interest.y = fmadds(dy, smooth, game_camera.transform.interest.y);
            game_camera.transform.interest.z = fmadds(dz, smooth, game_camera.transform.interest.z);
        }
        game_camera.transform.target_fov = cm_803BCCA0.x6C;
        delta = game_camera.transform.target_fov - game_camera.transform.fov;
        game_camera.transform.fov = fmadds(delta, cm_803BCCA0.x70, game_camera.transform.fov);
        {
            let mut slot: i32 = game_camera.x2C4 as i32;
            if (((((slot != 10) && (slot != 11)) && (slot >= 0)) && (slot < 6)) && (!{ gobj = ext::Player_GetEntity(slot); gobj }.is_null())) && (!{ subject = ext::ftLib_80086B74(gobj); subject }.is_null()) {
                distance = (2.0f32 * (*subject).ext.v.z) / ext::tanf(game_camera.transform.target_fov * 0.017453292f32);
            } else {
                distance = 1e+03f32;
            }
        }
        horiz_dist = distance * melee_runtime::gekko::cosf(game_camera.pitch_offset);
        game_camera.transform.target_position.x = fmadds(horiz_dist, melee_runtime::gekko::sinf(game_camera.yaw_offset), game_camera.transform.target_interest.x);
        game_camera.transform.target_position.y = fmadds(distance, melee_runtime::gekko::sinf(game_camera.pitch_offset), game_camera.transform.target_interest.y);
        game_camera.transform.target_position.z = fmadds(horiz_dist, melee_runtime::gekko::cosf(game_camera.yaw_offset), game_camera.transform.target_interest.z);
        dx2 = game_camera.transform.target_position.x - game_camera.transform.position.x;
        smooth2 = cm_803BCCA0.x68;
        dy2 = game_camera.transform.target_position.y - game_camera.transform.position.y;
        dz2 = game_camera.transform.target_position.z - game_camera.transform.position.z;
        game_camera.transform.position.x = fmadds(dx2, smooth2, game_camera.transform.position.x);
        game_camera.transform.position.y = fmadds(dy2, smooth2, game_camera.transform.position.y);
        game_camera.transform.position.z = fmadds(dz2, smooth2, game_camera.transform.position.z);
        return;
    }
    Camera_80030DF8();
    Camera_800293E0();
    Camera_8002B0E0();
    update_transform(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds, core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState);
    update_transform(core::ptr::addr_of_mut!(bounds_copy) as *mut CameraBounds, core::ptr::addr_of_mut!(game_camera.transform_copy) as *mut CameraTransformState);
    update_zoom_distance();
    update_bounds(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds, core::ptr::addr_of_mut!(bounds_copy) as *mut CameraBounds);
    update_avg_bounds_width();
}

/// `Camera_8002D85C`
pub unsafe extern "C" fn Camera_8002D85C(mut unused: *mut c_void) {
    let mut subject: *mut CmSubject = null_mut();
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut subj_pos: *mut Vec = null_mut();
    let mut horiz_dist: f32 = 0.0;
    let mut delta: f32 = 0.0;
    let mut distance: f32 = 0.0;
    let mut smooth: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut dz: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut smooth2: f32 = 0.0;
    let mut dx2: f32 = 0.0;
    let mut dz2: f32 = 0.0;
    let mut dy2: f32 = 0.0;
    let mut _pad: [u8; 16] = [0; 16];
    let mut bounds2: CameraBounds = unsafe { core::mem::zeroed() };
    let mut bounds: CameraBounds = unsafe { core::mem::zeroed() };
    gobj = ext::Player_GetEntity(game_camera.x2C4 as i32);
    if (((!gobj.is_null()) && (!{ subject = ext::ftLib_80086B74(gobj); subject }.is_null())) && (Camera_80029124(core::ptr::addr_of_mut!((*subject).bone_pos) as *mut Vec, 0) == (0 as u32))) && ((if (*subject).bone_pos.z < (0 as f32) { -(*subject).bone_pos.z } else { (*subject).bone_pos.z }) < 3e+01f32) {
        Camera_80030DF8();
        gobj = ext::Player_GetEntity(game_camera.x2C4 as i32);
        if (!gobj.is_null()) && (!{ subject = ext::ftLib_80086B74(gobj); subject }.is_null()) {
            subj_pos = get_subject_x1C(subject);
            game_camera.transform.target_interest.x = (*subj_pos).x;
            game_camera.transform.target_interest.y = (*subj_pos).y;
            game_camera.transform.target_interest.z = (*subj_pos).z;
            dx = game_camera.transform.target_interest.x - game_camera.transform.interest.x;
            smooth = cm_803BCCA0.x64;
            dy = game_camera.transform.target_interest.y - game_camera.transform.interest.y;
            dz = game_camera.transform.target_interest.z - game_camera.transform.interest.z;
            game_camera.transform.interest.x = fmadds(dx, smooth, game_camera.transform.interest.x);
            game_camera.transform.interest.y = fmadds(dy, smooth, game_camera.transform.interest.y);
            game_camera.transform.interest.z = fmadds(dz, smooth, game_camera.transform.interest.z);
        }
        game_camera.transform.target_fov = cm_803BCCA0.x6C;
        delta = game_camera.transform.target_fov - game_camera.transform.fov;
        game_camera.transform.fov = fmadds(delta, cm_803BCCA0.x70, game_camera.transform.fov);
        {
            let mut slot: i32 = game_camera.x2C4 as i32;
            if (((((slot != 10) && (slot != 11)) && (slot >= 0)) && (slot < 6)) && (!{ gobj = ext::Player_GetEntity(slot); gobj }.is_null())) && (!{ subject = ext::ftLib_80086B74(gobj); subject }.is_null()) {
                distance = (2.0f32 * (*subject).ext.v.z) / ext::tanf(game_camera.transform.target_fov * 0.017453292f32);
            } else {
                distance = 1e+03f32;
            }
        }
        if game_camera.pitch_offset > ext::Stage_GetCamAngleRadiansUp() {
            game_camera.pitch_offset = ext::Stage_GetCamAngleRadiansUp();
        } else if game_camera.pitch_offset < (-ext::Stage_GetCamAngleRadiansDown()) {
            game_camera.pitch_offset = -ext::Stage_GetCamAngleRadiansDown();
        }
        if game_camera.yaw_offset > ext::Stage_GetCamAngleRadiansLeft() {
            game_camera.yaw_offset = ext::Stage_GetCamAngleRadiansLeft();
        } else if game_camera.yaw_offset < (-ext::Stage_GetCamAngleRadiansRight()) {
            game_camera.yaw_offset = -ext::Stage_GetCamAngleRadiansRight();
        }
        horiz_dist = distance * melee_runtime::gekko::cosf(game_camera.pitch_offset);
        game_camera.transform.target_position.x = fmadds(horiz_dist, melee_runtime::gekko::sinf(game_camera.yaw_offset), game_camera.transform.target_interest.x);
        game_camera.transform.target_position.y = fmadds(distance, melee_runtime::gekko::sinf(game_camera.pitch_offset), game_camera.transform.target_interest.y);
        game_camera.transform.target_position.z = fmadds(horiz_dist, melee_runtime::gekko::cosf(game_camera.yaw_offset), game_camera.transform.target_interest.z);
        dx2 = game_camera.transform.target_position.x - game_camera.transform.position.x;
        smooth2 = cm_803BCCA0.x68;
        dy2 = game_camera.transform.target_position.y - game_camera.transform.position.y;
        dz2 = game_camera.transform.target_position.z - game_camera.transform.position.z;
        game_camera.transform.position.x = fmadds(dx2, smooth2, game_camera.transform.position.x);
        game_camera.transform.position.y = fmadds(dy2, smooth2, game_camera.transform.position.y);
        game_camera.transform.position.z = fmadds(dz2, smooth2, game_camera.transform.position.z);
        return;
    }
    Camera_80030DF8();
    Camera_800293E0();
    Camera_8002B0E0();
    update_transform(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds, core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState);
    update_transform(core::ptr::addr_of_mut!(bounds2) as *mut CameraBounds, core::ptr::addr_of_mut!(game_camera.transform_copy) as *mut CameraTransformState);
    update_zoom_distance();
    update_bounds(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds, core::ptr::addr_of_mut!(bounds2) as *mut CameraBounds);
    update_avg_bounds_width();
}

/// `set_bounds_z`
pub unsafe fn set_bounds_z(mut bounds: *mut CameraBounds, mut interest: *mut Vec, mut position: *mut Vec) {
    let mut diff: Vec3 = unsafe { core::mem::zeroed() };
    let _ = lbVector_Diff(interest, position, core::ptr::addr_of_mut!(diff) as *mut Vec);
    (*bounds).z_pos = lbVector_Len(core::ptr::addr_of_mut!(diff) as *mut Vec);
}

/// `smooth_fixed_camera_interest`
pub unsafe fn smooth_fixed_camera_interest(mut transform_copy: *mut CameraTransformState, mut cam: *mut Camera, mut globals: *mut CameraUnkGlobals) {
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut dz: f32 = 0.0;
    ext::Stage_80224CAC(core::ptr::addr_of_mut!((*transform_copy).target_interest) as *mut Vec);
    dx = (*cam).transform_copy.target_interest.x - (*cam).transform_copy.interest.x;
    dy = (*cam).transform_copy.target_interest.y - (*cam).transform_copy.interest.y;
    dz = (*cam).transform_copy.target_interest.z - (*cam).transform_copy.interest.z;
    (*cam).transform_copy.interest.x = fmadds(dx, (*globals).x74, (*cam).transform_copy.interest.x);
    (*cam).transform_copy.interest.y = fmadds(dy, (*globals).x74, (*cam).transform_copy.interest.y);
    (*cam).transform_copy.interest.z = fmadds(dz, (*globals).x74, (*cam).transform_copy.interest.z);
}

/// `Camera_8002DDC4`
pub unsafe extern "C" fn Camera_8002DDC4(mut unused: *mut c_void) {
    let mut bounds: CameraBounds = unsafe { core::mem::zeroed() };
    let mut target_interest: *mut Vec = null_mut();
    let mut transform: *mut CameraTransformState = null_mut();
    let mut cam: *mut Camera = null_mut();
    let mut target_position: *mut Vec = null_mut();
    let mut transform_copy: *mut CameraTransformState = null_mut();
    let mut globals: *mut CameraUnkGlobals = null_mut();
    let mut _pad: [u8; 4] = [0; 4];
    let mut smooth: f32 = 0.0;
    let mut _pad1: [u8; 4] = [0; 4];
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut dz: f32 = 0.0;
    cam = core::ptr::addr_of_mut!(game_camera) as *mut Camera;
    Camera_80030DF8();
    transform = core::ptr::addr_of_mut!((*cam).transform) as *mut CameraTransformState;
    target_interest = core::ptr::addr_of_mut!((*transform).target_interest) as *mut Vec;
    ext::Stage_80224CAC(target_interest);
    globals = core::ptr::addr_of_mut!(cm_803BCCA0) as *mut CameraUnkGlobals;
    transform_copy = core::ptr::addr_of_mut!((*cam).transform_copy) as *mut CameraTransformState;
    {
        dx = (*cam).transform.target_interest.x - (*cam).transform.interest.x;
        smooth = (*globals).x74;
        dy = (*cam).transform.target_interest.y - (*cam).transform.interest.y;
        dz = (*cam).transform.target_interest.z - (*cam).transform.interest.z;
        (*cam).transform.interest.x = fmadds(dx, smooth, (*cam).transform.interest.x);
        (*cam).transform.interest.y = fmadds(dy, smooth, (*cam).transform.interest.y);
        (*cam).transform.interest.z = fmadds(dz, smooth, (*cam).transform.interest.z);
    }
    smooth_fixed_camera_interest(transform_copy, cam, globals);
    (*cam).transform.target_fov = ext::Stage_GetCamFixedFov();
    {
        let mut delta: f32 = 0.0;
        delta = (*cam).transform.target_fov - (*cam).transform.fov;
        (*cam).transform.fov = fmadds(delta, (*globals).x7C, (*cam).transform.fov);
    }
    (*cam).transform_copy.target_fov = ext::Stage_GetCamFixedFov();
    target_position = core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec;
    {
        let mut delta: f32 = 0.0;
        delta = (*cam).transform_copy.target_fov - (*cam).transform_copy.fov;
        (*cam).transform_copy.fov = fmadds(delta, (*globals).x7C, (*cam).transform_copy.fov);
    }
    ext::Stage_SetVecToFixedCamPos(target_position);
    {
        dx = (*cam).transform.target_position.x - (*cam).transform.position.x;
        smooth = (*globals).x78;
        dy = (*cam).transform.target_position.y - (*cam).transform.position.y;
        dz = (*cam).transform.target_position.z - (*cam).transform.position.z;
        (*cam).transform.position.x = fmadds(dx, smooth, (*cam).transform.position.x);
        (*cam).transform.position.y = fmadds(dy, smooth, (*cam).transform.position.y);
        (*cam).transform.position.z = fmadds(dz, smooth, (*cam).transform.position.z);
    }
    ext::Stage_SetVecToFixedCamPos(core::ptr::addr_of_mut!((*transform_copy).target_position) as *mut Vec);
    {
        smooth = (*globals).x78;
        dx = (*cam).transform_copy.target_position.x - (*cam).transform_copy.position.x;
        dy = (*cam).transform_copy.target_position.y - (*cam).transform_copy.position.y;
        dz = (*cam).transform_copy.target_position.z - (*cam).transform_copy.position.z;
        (*cam).transform_copy.position.x = fmadds(dx, smooth, (*cam).transform_copy.position.x);
        (*cam).transform_copy.position.y = fmadds(dy, smooth, (*cam).transform_copy.position.y);
        (*cam).transform_copy.position.z = fmadds(dz, smooth, (*cam).transform_copy.position.z);
    }
    set_bounds_z(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds, target_interest, target_position);
    Camera_UpdateQuakes(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds);
    Camera_ApplyQuake(core::ptr::addr_of_mut!(bounds) as *mut CameraBounds, core::ptr::addr_of_mut!((*cam).transform) as *mut CameraTransformState);
}

/// `Camera_8002DFE4`
pub unsafe fn Camera_8002DFE4(mut arg0: *mut Vec, mut interest: *mut Vec, mut transform: *mut CameraTransformState) -> i32 {
    let mut sp14: Vec3 = unsafe { core::mem::zeroed() };
    let mut var_f31: f32 = 0.0;
    let mut var_r31: i32 = 0;
    var_r31 = 0;
    's57: {
        match game_camera.x341_b5_b6() as i32 {
            0 => {
                var_r31 = 1;
            }
            1 => {
                if game_camera.x378.s32_v == game_camera.x37C.s32_v {
                    var_r31 = 1;
                } else {
                    var_f31 = (game_camera.x378.s32_v as f32) / (game_camera.x37C.s32_v as f32);
                }
            }
            2 => {
                if game_camera.x378.f32_v >= 1.0f32 {
                    var_r31 = 1;
                } else {
                    var_f31 = game_camera.x378.f32_v;
                }
            }
            _ => {}
        }
    }
    if var_r31 != 0 {
        (*transform).interest = *interest;
    } else {
        let _ = lbVector_Diff(interest, arg0, core::ptr::addr_of_mut!(sp14) as *mut Vec);
        sp14.x = sp14.x * var_f31;
        sp14.y = sp14.y * var_f31;
        sp14.z = sp14.z * var_f31;
        let _ = lbVector_Add(core::ptr::addr_of_mut!(sp14) as *mut Vec, arg0);
        (*transform).interest = sp14;
    }
    return var_r31;
}

/// `Camera_8002E158`
pub unsafe fn Camera_8002E158(mut arg0: *mut f32, mut farg0: f32, mut farg1: f32) -> i32 {
    let mut ret: i32 = 0;
    let mut var_f4: f32 = 0.0;
    ret = 0;
    's58: {
        match game_camera.x341_b5_b6() as i32 {
            0 => {
                ret = 1;
            }
            1 => {
                if game_camera.x378.s32_v == game_camera.x37C.s32_v {
                    ret = 1;
                } else {
                    var_f4 = (game_camera.x378.s32_v as f32) / (game_camera.x37C.s32_v as f32);
                }
            }
            2 => {
                if game_camera.x378.f32_v >= 1.0f32 {
                    ret = 1;
                } else {
                    var_f4 = game_camera.x378.f32_v;
                }
            }
            _ => {}
        }
    }
    if ret != 0 {
        *arg0 = farg1;
    } else {
        *arg0 = fmadds(var_f4, farg1 - farg0, farg0);
    }
    return ret;
}

/// `Camera_8002E234`
pub unsafe fn Camera_8002E234() -> i32 {
    let mut sp14: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp10: f32 = 0.0;
    let mut spC: f32 = 0.0;
    let mut sp8: f32 = 0.0;
    let mut temp_f31: f32 = 0.0;
    let mut ret: i32 = 0;
    ret = 0;
    's59: {
        match game_camera.x341_b3_b4() as i32 {
            0 | 1 | 3 => {
                ret = Camera_8002DFE4(core::ptr::addr_of_mut!(game_camera.x368) as *mut Vec, core::ptr::addr_of_mut!(game_camera.transform.target_position) as *mut Vec, (core::ptr::addr_of_mut!(game_camera.transform.position) as *mut Vec) as *mut CameraTransformState);
            }
            2 => {
                ret = 1;
                if game_camera.x35C.bits.b1() != 0 {
                    ret = ret & Camera_8002E158(core::ptr::addr_of_mut!(sp10) as *mut f32, game_camera.x368.y, game_camera.x35C.vec.y);
                }
                if game_camera.x35C.bits.b2() != 0 {
                    ret = ret & Camera_8002E158(core::ptr::addr_of_mut!(spC) as *mut f32, game_camera.x368.z, game_camera.x35C.vec.z);
                }
                if game_camera.x35C.bits.b0() != 0 {
                    ret = ret & Camera_8002E158(core::ptr::addr_of_mut!(sp8) as *mut f32, (*((core::ptr::addr_of_mut!(game_camera.x368) as *mut Vec) as *mut i16)) as f32, game_camera.x35C.bits.x2 as f32);
                }
                sp14.y = 0.0f32;
                sp14.x = 0.0f32;
                sp14.z = 1.0f32;
                ext::lbVector_Rotate(core::ptr::addr_of_mut!(sp14) as *mut Vec, 1, -game_camera.x35C.vec.y);
                ext::lbVector_Rotate(core::ptr::addr_of_mut!(sp14) as *mut Vec, 2, game_camera.x35C.vec.z);
                temp_f31 = game_camera.x35C.bits.x2 as f32;
                let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(sp14) as *mut Vec);
                sp14.x = sp14.x * temp_f31;
                sp14.y = sp14.y * temp_f31;
                sp14.z = sp14.z * temp_f31;
                let _ = lbVector_Add(core::ptr::addr_of_mut!(sp14) as *mut Vec, core::ptr::addr_of_mut!(game_camera.transform.interest) as *mut Vec);
                game_camera.transform.target_position = sp14;
                sp14.y = 0.0f32;
                sp14.x = 0.0f32;
                sp14.z = 1.0f32;
                ext::lbVector_Rotate(core::ptr::addr_of_mut!(sp14) as *mut Vec, 1, -sp10);
                ext::lbVector_Rotate(core::ptr::addr_of_mut!(sp14) as *mut Vec, 2, spC);
                temp_f31 = sp8;
                let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(sp14) as *mut Vec);
                sp14.x = sp14.x * temp_f31;
                sp14.y = sp14.y * temp_f31;
                sp14.z = sp14.z * temp_f31;
                let _ = lbVector_Add(core::ptr::addr_of_mut!(sp14) as *mut Vec, core::ptr::addr_of_mut!(game_camera.transform.interest) as *mut Vec);
                game_camera.transform.position = sp14;
            }
            _ => {}
        }
    }
    return ret;
}

/// `Camera_8002E490`
pub unsafe extern "C" fn Camera_8002E490(mut unused: *mut c_void) {
    let mut sp1C: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp10: Vec3 = unsafe { core::mem::zeroed() };
    let mut var_r29: i32 = 0;
    Camera_80030DF8();
    's60: {
        match game_camera.x341_b1_b2() as i32 {
            1 => {
                {
                    let mut gobj: *mut HSD_GObj = ext::Player_GetEntity(game_camera.x344.slot);
                    if !gobj.is_null() {
                        let mut subject: *mut CmSubject = ext::ftLib_80086B74(gobj);
                        if !subject.is_null() {
                            game_camera.transform.target_interest = (*subject).bone_pos;
                        }
                    }
                    break 's60;
                }
            }
            2 => {
                game_camera.transform.target_interest = game_camera.x344.vec;
            }
            3 => {
                if (game_camera.x344.cb.is_some()) && (game_camera.x344.cb.unwrap()(core::ptr::addr_of_mut!(sp1C) as *mut Vec) != 0) {
                    game_camera.transform.target_interest = sp1C;
                }
            }
            _ => {
            }
        }
    }
    's61: {
        match game_camera.x341_b3_b4() as i32 {
            1 => {
                game_camera.transform.target_position = game_camera.x35C.vec;
            }
            3 => {
                if (game_camera.x35C.cb.is_some()) && (game_camera.x35C.cb.unwrap()(core::ptr::addr_of_mut!(sp10) as *mut Vec) != 0) {
                    game_camera.transform.target_position = sp10;
                }
            }
            _ => {
            }
        }
    }
    var_r29 = 0;
    's62: {
        match game_camera.x341_b1_b2() as i32 {
            0 | 1 | 2 | 3 => {
                var_r29 = Camera_8002DFE4(core::ptr::addr_of_mut!(game_camera.x350) as *mut Vec, core::ptr::addr_of_mut!(game_camera.transform.target_interest) as *mut Vec, core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState);
            }
            _ => {}
        }
    }
    var_r29 = var_r29 & Camera_8002E234();
    var_r29 = var_r29 & Camera_8002E158(core::ptr::addr_of_mut!(game_camera.transform.fov) as *mut f32, game_camera.x374, game_camera.transform.target_fov);
    's63: {
        match game_camera.x341_b5_b6() as i32 {
            0 => {
            }
            1 => {
                if game_camera.x378.s32_v < game_camera.x37C.s32_v {
                    game_camera.x378.s32_v = game_camera.x378.s32_v + 1;
                }
            }
            2 => {
                game_camera.x378.f32_v = fmadds(1.0f32 - game_camera.x378.f32_v, game_camera.x37C.f32_v, game_camera.x378.f32_v);
                if game_camera.x378.f32_v > 0.999f32 {
                    game_camera.x378.f32_v = 1.0f32;
                }
            }
            _ => {}
        }
    }
    game_camera.set_x341_b7(((if var_r29 != 0 { 1 } else { 0 }) as u8) as u8);
}

/// `Camera_8002E6FC`
pub unsafe fn Camera_8002E6FC(mut arg0: i32) {
    let mut spC: Vec3 = unsafe { core::mem::zeroed() };
    if game_camera.mode != ((CAMERA_BOSS_INTRO as i32) as u32) {
        Camera_8002FE38();
    }
    game_camera.set_x341_b1_b2((1 as u8) as u8);
    game_camera.x344.slot = arg0;
    's64: {
        match game_camera.x341_b1_b2() as i32 {
            1 => {
                {
                    let mut gobj: *mut HSD_GObj = ext::Player_GetEntity(game_camera.x344.slot);
                    if !gobj.is_null() {
                        let mut subject: *mut CmSubject = ext::ftLib_80086B74(gobj);
                        if !subject.is_null() {
                            game_camera.transform.target_interest = (*subject).bone_pos;
                        }
                    }
                    break 's64;
                }
            }
            2 => {
                game_camera.transform.target_interest = game_camera.x344.vec;
            }
            3 => {
                {
                    if (game_camera.x344.cb.is_some()) && (game_camera.x344.cb.unwrap()(core::ptr::addr_of_mut!(spC) as *mut Vec) != 0) {
                        game_camera.transform.target_interest = spC;
                    }
                    break 's64;
                }
            }
            _ => {
            }
        }
    }
}

/// `Camera_8002E818`
pub unsafe fn Camera_8002E818(mut pos: *mut Vec) {
    let mut spC: Vec3 = unsafe { core::mem::zeroed() };
    if game_camera.mode != ((CAMERA_BOSS_INTRO as i32) as u32) {
        Camera_8002FE38();
    }
    game_camera.set_x341_b1_b2((2 as u8) as u8);
    game_camera.x344.vec = *pos;
    's65: {
        match game_camera.x341_b1_b2() as i32 {
            1 => {
                {
                    let mut gobj: *mut HSD_GObj = ext::Player_GetEntity(game_camera.x344.slot);
                    if !gobj.is_null() {
                        let mut subject: *mut CmSubject = ext::ftLib_80086B74(gobj);
                        if !subject.is_null() {
                            game_camera.transform.target_interest = (*subject).bone_pos;
                        }
                    }
                    break 's65;
                }
            }
            2 => {
                game_camera.transform.target_interest = game_camera.x344.vec;
            }
            3 => {
                {
                    if (game_camera.x344.cb.is_some()) && (game_camera.x344.cb.unwrap()(core::ptr::addr_of_mut!(spC) as *mut Vec) != 0) {
                        game_camera.transform.target_interest = spC;
                    }
                    break 's65;
                }
            }
            _ => {
            }
        }
    }
}

/// `Camera_8002E948`
pub unsafe fn Camera_8002E948(mut cb: Option<unsafe extern "C" fn(*mut Vec) -> i32>) {
    let mut spC: Vec3 = unsafe { core::mem::zeroed() };
    if game_camera.mode != ((CAMERA_BOSS_INTRO as i32) as u32) {
        Camera_8002FE38();
    }
    game_camera.set_x341_b1_b2((3 as u8) as u8);
    game_camera.x344.cb = cb;
    's66: {
        match game_camera.x341_b1_b2() as i32 {
            1 => {
                {
                    let mut gobj: *mut HSD_GObj = ext::Player_GetEntity(game_camera.x344.slot);
                    if !gobj.is_null() {
                        let mut subject: *mut CmSubject = ext::ftLib_80086B74(gobj);
                        if !subject.is_null() {
                            game_camera.transform.target_interest = (*subject).bone_pos;
                        }
                    }
                    break 's66;
                }
            }
            2 => {
                game_camera.transform.target_interest = game_camera.x344.vec;
            }
            3 => {
                {
                    if (game_camera.x344.cb.is_some()) && (game_camera.x344.cb.unwrap()(core::ptr::addr_of_mut!(spC) as *mut Vec) != 0) {
                        game_camera.transform.target_interest = spC;
                    }
                    break 's66;
                }
            }
            _ => {
            }
        }
    }
}

pub unsafe fn Camera_8002EA64(mut arg0: *mut Vec) {
    todo!("Camera_8002EA64: goto, hand port")
}

/// `Camera_8002EB5C`
pub unsafe fn Camera_8002EB5C(mut arg0: f32) {
    let mut spC: Vec3 = unsafe { core::mem::zeroed() };
    if game_camera.mode != ((CAMERA_BOSS_INTRO as i32) as u32) {
        Camera_8002FE38();
    }
    if (game_camera.x341_b3_b4() as i32) != 2 {
        game_camera.set_x341_b3_b4((2 as u8) as u8);
        game_camera.x35C.bits.set_b0((0 as u8) as u8);
        game_camera.x35C.bits.set_b2((0 as u8) as u8);
    }
    game_camera.x35C.vec.y = arg0;
    game_camera.x35C.bits.set_b1((1 as u8) as u8);
    's68: {
        match game_camera.x341_b3_b4() as i32 {
            1 => {
                game_camera.transform.target_position = game_camera.x35C.vec;
            }
            3 => {
                if (game_camera.x35C.cb.is_some()) && (game_camera.x35C.cb.unwrap()(core::ptr::addr_of_mut!(spC) as *mut Vec) != 0) {
                    game_camera.transform.target_position = spC;
                }
            }
            _ => {
            }
        }
    }
}

/// `Camera_8002EC7C`
pub unsafe fn Camera_8002EC7C(mut arg0: f32) {
    let mut spC: Vec3 = unsafe { core::mem::zeroed() };
    if game_camera.mode != ((CAMERA_BOSS_INTRO as i32) as u32) {
        Camera_8002FE38();
    }
    if (game_camera.x341_b3_b4() as i32) != 2 {
        game_camera.set_x341_b3_b4((2 as u8) as u8);
        game_camera.x35C.bits.set_b0((0 as u8) as u8);
        game_camera.x35C.bits.set_b1((0 as u8) as u8);
    }
    game_camera.x35C.vec.z = arg0;
    game_camera.x35C.bits.set_b2((1 as u8) as u8);
    's69: {
        match game_camera.x341_b3_b4() as i32 {
            1 => {
                game_camera.transform.target_position = game_camera.x35C.vec;
            }
            3 => {
                if (game_camera.x35C.cb.is_some()) && (game_camera.x35C.cb.unwrap()(core::ptr::addr_of_mut!(spC) as *mut Vec) != 0) {
                    game_camera.transform.target_position = spC;
                }
            }
            _ => {
            }
        }
    }
}

/// `Camera_8002ED9C`
pub unsafe fn Camera_8002ED9C(mut arg0: f32) {
    let mut spC: Vec3 = unsafe { core::mem::zeroed() };
    if game_camera.mode != ((CAMERA_BOSS_INTRO as i32) as u32) {
        Camera_8002FE38();
    }
    if (game_camera.x341_b3_b4() as i32) != 2 {
        game_camera.set_x341_b3_b4((2 as u8) as u8);
        game_camera.x35C.bits.set_b1((0 as u8) as u8);
        game_camera.x35C.bits.set_b2((0 as u8) as u8);
    }
    game_camera.x35C.bits.x2 = melee_runtime::cabi::f2i16(arg0 as f64);
    game_camera.x35C.bits.set_b0((1 as u8) as u8);
    's70: {
        match game_camera.x341_b3_b4() as i32 {
            1 => {
                game_camera.transform.target_position = game_camera.x35C.vec;
            }
            3 => {
                if (game_camera.x35C.cb.is_some()) && (game_camera.x35C.cb.unwrap()(core::ptr::addr_of_mut!(spC) as *mut Vec) != 0) {
                    game_camera.transform.target_position = spC;
                }
            }
            _ => {
            }
        }
    }
}

/// `Camera_8002EEC8`
pub unsafe fn Camera_8002EEC8(mut fov: f32) {
    if game_camera.mode != ((CAMERA_BOSS_INTRO as i32) as u32) {
        Camera_8002FE38();
    }
    game_camera.transform.target_fov = fov;
}

/// `Camera_8002EF14`
pub unsafe fn Camera_8002EF14() {
    let mut sp24: i32 = 0;
    let mut sp18: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp14: i32 = 0;
    let mut sp8: Vec3 = unsafe { core::mem::zeroed() };
    if game_camera.mode != ((CAMERA_BOSS_INTRO as i32) as u32) {
        Camera_8002FE38();
    }
    game_camera.set_x341_b5_b6((0 as u8) as u8);
    's71: {
        match game_camera.x341_b1_b2() as i32 {
            1 => {
                {
                    let mut gobj: *mut HSD_GObj = ext::Player_GetEntity(game_camera.x344.slot);
                    if !gobj.is_null() {
                        let mut subject: *mut CmSubject = ext::ftLib_80086B74(gobj);
                        if !subject.is_null() {
                            game_camera.transform.target_interest = (*subject).bone_pos;
                        }
                    }
                    break 's71;
                }
            }
            2 => {
                game_camera.transform.target_interest = game_camera.x344.vec;
            }
            3 => {
                if (game_camera.x344.cb.is_some()) && (game_camera.x344.cb.unwrap()(core::ptr::addr_of_mut!(sp18) as *mut Vec) != 0) {
                    game_camera.transform.target_interest = sp18;
                }
            }
            _ => {
            }
        }
    }
    's72: {
        match game_camera.x341_b1_b2() as i32 {
            0 | 1 | 2 | 3 => {
                let _ = Camera_8002DFE4(core::ptr::addr_of_mut!(game_camera.x350) as *mut Vec, core::ptr::addr_of_mut!(game_camera.transform.target_interest) as *mut Vec, core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState);
            }
            _ => {}
        }
    }
    's73: {
        match game_camera.x341_b3_b4() as i32 {
            1 => {
                game_camera.transform.target_position = game_camera.x35C.vec;
            }
            3 => {
                if (game_camera.x35C.cb.is_some()) && (game_camera.x35C.cb.unwrap()(core::ptr::addr_of_mut!(sp8) as *mut Vec) != 0) {
                    game_camera.transform.target_position = sp8;
                }
            }
            _ => {
            }
        }
    }
    let _ = Camera_8002E234();
    game_camera.transform.fov = game_camera.transform.target_fov;
}

/// `Camera_8002F0E4`
pub unsafe fn Camera_8002F0E4(mut arg0: i32) {
    let mut sp24: i32 = 0;
    let mut sp20: i32 = 0;
    let mut spC: Vec3 = unsafe { core::mem::zeroed() };
    let mut temp_f30: f32 = 0.0;
    let mut temp_f31: f32 = 0.0;
    if game_camera.mode != ((CAMERA_BOSS_INTRO as i32) as u32) {
        Camera_8002FE38();
    }
    game_camera.set_x341_b5_b6((1 as u8) as u8);
    game_camera.x378.s32_v = 0;
    game_camera.x37C.s32_v = arg0;
    's74: {
        match game_camera.x341_b1_b2() as i32 {
            0 | 1 | 2 | 3 => {
                game_camera.x350 = game_camera.transform.interest;
            }
            _ => {}
        }
    }
    's75: {
        match game_camera.x341_b3_b4() as i32 {
            0 | 1 | 3 => {
                game_camera.x368 = game_camera.transform.position;
            }
            2 => {
                {
                    let _ = lbVector_Diff(core::ptr::addr_of_mut!(game_camera.transform.position) as *mut Vec, core::ptr::addr_of_mut!(game_camera.transform.interest) as *mut Vec, core::ptr::addr_of_mut!(spC) as *mut Vec);
                    temp_f30 = ext::atan2f(spC.y, melee_runtime::gekko::sqrtf(fmadds(spC.x, spC.x, spC.z * spC.z)));
                    temp_f31 = ext::atan2f(spC.x, spC.z);
                    *((core::ptr::addr_of_mut!(game_camera.x368) as *mut Vec) as *mut i16) = melee_runtime::cabi::f2i16(melee_runtime::gekko::sqrtf(fmadds(spC.z, spC.z, fmadds(spC.x, spC.x, spC.y * spC.y))) as f64);
                    game_camera.x368.y = temp_f30;
                    game_camera.x368.z = temp_f31;
                    break 's75;
                }
            }
            _ => {}
        }
    }
    game_camera.x374 = game_camera.transform.fov;
}

/// `Camera_8002F260`
pub unsafe fn Camera_8002F260() -> i32 {
    return game_camera.x341_b7() as i32;
}

/// `Camera_8002F274`
pub unsafe fn Camera_8002F274() {
    let mut sp8: Vec3 = unsafe { core::mem::zeroed() };
    if game_camera.mode != ((CAMERA_BOSS_INTRO as i32) as u32) {
        Camera_8002FE38();
    }
    game_camera.set_x341_b3_b4((1 as u8) as u8);
    game_camera.x35C.vec = game_camera.transform.position;
    's76: {
        match game_camera.x341_b3_b4() as i32 {
            1 => {
                game_camera.transform.target_position = game_camera.x35C.vec;
            }
            3 => {
                if (game_camera.x35C.cb.is_some()) && (game_camera.x35C.cb.unwrap()(core::ptr::addr_of_mut!(sp8) as *mut Vec) != 0) {
                    game_camera.transform.target_position = sp8;
                }
            }
            _ => {
            }
        }
    }
}

/// `fn_8002F360`
pub unsafe extern "C" fn fn_8002F360(mut x: *mut HSD_GObj) {
    if cm_script.state != 0 {
        game_camera.transform = cm_script.normal;
    }
    if cm_803BCB18.callback[game_camera.mode as usize].is_some() {
        cm_803BCB18.callback[game_camera.mode as usize].unwrap()(x as *mut c_void);
    }
    cm_script_update();
}

/// `Camera_8002F3AC`
pub unsafe fn Camera_8002F3AC() {
    let mut gobj: *mut HSD_GObj = game_camera.gobj;
    if cm_803BCB18.callback[game_camera.mode as usize].is_some() {
        cm_803BCB18.callback[game_camera.mode as usize].unwrap()(gobj as *mut c_void);
    }
    game_camera.transform.position = game_camera.transform.target_position;
    game_camera.transform.interest = game_camera.transform.target_interest;
    game_camera.transform.fov = game_camera.transform.target_fov;
    game_camera.transform_copy.position = game_camera.transform_copy.target_position;
    game_camera.transform_copy.interest = game_camera.transform_copy.target_interest;
    game_camera.transform_copy.fov = game_camera.transform_copy.target_fov;
}

/// `Camera_SetModeToStandard`
pub unsafe fn Camera_SetModeToStandard() {
    game_camera.mode = (CAMERA_STANDARD as i32) as u32;
}

/// `Camera_SetBounds`
pub unsafe extern "C" fn Camera_SetBounds(mut arg0: *mut Quaternion) -> i32 {
    (*arg0).z = cm_fly_left();
    (*arg0).w = cm_fly_right();
    (*arg0).x = cm_fly_top();
    (*arg0).y = cm_fly_bottom();
    return 1;
}

/// `Camera_SetUpPauseCamera`
pub unsafe fn Camera_SetUpPauseCamera(mut pauserSlot: i8, mut pauserId: i8, mut arg2: i32) {
    let mut target_interest: *mut Vec = null_mut();
    let mut pause_eye_offset: *mut Vec = null_mut();
    let mut transform: *mut CameraTransformState = null_mut();
    let mut var_f31: f32 = 0.0;
    if (((pauserSlot as i32) < 0) || ((pauserSlot as i32) >= 6)) && (((((pauserSlot as i32) - 10) as u8) as i32) > 1) {
        pauserSlot = 0 as i8;
    }
    if (((pauserId as i32) < 0) || ((pauserId as i32) >= 4)) && (((((pauserId as i32) - 4) as u8) as i32) > 1) {
        pauserId = 4 as i8;
    }
    game_camera.mode = (CAMERA_PAUSE as i32) as u32;
    game_camera.x2C4 = pauserSlot;
    game_camera.x2C5 = pauserId;
    game_camera.x2D0.x_min = cm_fly_left();
    game_camera.x2D0.x_max = cm_fly_right();
    game_camera.x2D0.y_max = cm_fly_top();
    game_camera.x2D0.y_min = cm_fly_bottom();
    game_camera.x2D0.z_max = 2e+03f32;
    game_camera.x2D0.z_min = -2e+03f32;
    game_camera.x2D0.angle_up = ext::Stage_GetCamAngleRadiansUp();
    game_camera.x2D0.angle_down = ext::Stage_GetCamAngleRadiansDown();
    game_camera.x2D0.angle_right = ext::Stage_GetCamAngleRadiansRight();
    game_camera.x2D0.angle_left = ext::Stage_GetCamAngleRadiansLeft();
    game_camera.x2D0.unk28 = ext::Stage_GetPauseCamZPosMin();
    game_camera.x2D0.unk2C = ext::Stage_GetPauseCamZPosMax();
    game_camera.x2D0.callback = core::mem::transmute::<Option<unsafe extern "C" fn()>, Option<unsafe extern "C" fn(*mut Camera_x2D0)>>(core::mem::transmute::<Option<unsafe extern "C" fn(*mut Quaternion) -> i32>, Option<unsafe extern "C" fn()>>(Some(Camera_SetBounds)));
    's77: {
        match arg2 {
            0 => {
                var_f31 = ext::Stage_GetPauseCamZPosInit();
            }
            1 => {
                var_f31 = game_camera.x2D0.unk28;
            }
            _ => {
                if 0 != 0 { (); } else { panic!("Camera_SetUpPauseCamera: assertion failed"); }
            }
        }
    }
    let _ = compute_orbit_distance(game_camera.x2C4 as i32);
    {
        let mut slot: i8 = game_camera.x2C4;
        pause_eye_offset = core::ptr::addr_of_mut!(game_camera.pause_eye_offset) as *mut Vec;
        game_camera.x314.z = 0.0f32;
        game_camera.x314.y = 0.0f32;
        game_camera.x314.x = 0.0f32;
        (*pause_eye_offset).x = 0.0f32;
        (*pause_eye_offset).y = 5.0f32;
        (*pause_eye_offset).z = 2e+01f32;
        game_camera.pause_up.x = 0.0f32;
        game_camera.pause_up.y = 1.0f32;
        game_camera.pause_up.z = 0.0f32;
        if (slot as i32) == 10 {
            game_camera.pause_eye_distance = 3.0f32 * var_f31;
        } else {
            game_camera.pause_eye_distance = var_f31;
        }
        Camera_8002BAA8(0.0f32);
        Camera_8002BD88(0.0f32, 0.0f32);
        Camera_8002C010(0.0f32, 0.0f32);
        {
            transform = core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState;
            target_interest = core::ptr::addr_of_mut!(game_camera.transform.target_interest) as *mut Vec;
            *target_interest = game_camera.x308;
            let _ = lbVector_Add(core::ptr::addr_of_mut!((*transform).target_interest) as *mut Vec, core::ptr::addr_of_mut!(game_camera.x314) as *mut Vec);
            game_camera.transform.target_position = *target_interest;
            let _ = lbVector_Add(core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec, pause_eye_offset);
        }
    }
}

/// `Camera_SetUpPauseCameraWithDefaultZoom`
pub unsafe fn Camera_SetUpPauseCameraWithDefaultZoom(mut pauserSlot: i8, mut pauserId: i8) {
    Camera_SetUpPauseCamera(pauserSlot, pauserId, 0);
}

/// `Camera_8002F760`
pub unsafe fn Camera_8002F760(mut arg0: i8, mut arg1: i8) {
    Camera_SetUpPauseCamera(arg0, arg1, 1);
}

/// `Camera_8002F784`
pub unsafe fn Camera_8002F784(mut slot: i8, mut arg1: i8) {
    game_camera.mode = (CAMERA_TRAINING_MENU as i32) as u32;
    game_camera.x2C4 = slot;
    game_camera.x2C5 = arg1;
    game_camera.pitch_offset = 0.0f32;
    game_camera.yaw_offset = 0.0f32;
}

/// `Camera_8002F7AC`
pub unsafe fn Camera_8002F7AC(mut slot: i8) {
    let mut cam_box: *mut CmSubject = null_mut();
    let mut fighter_gobj: *mut HSD_GObj = null_mut();
    let mut direction: f32 = 0.0;
    let mut randf: f32 = 0.0;
    let mut offset_dir: f32 = 0.0;
    let mut rand_dir: f64 = 0.0;
    game_camera.mode = (CAMERA_CLEAR as i32) as u32;
    game_camera.x2C4 = slot;
    fighter_gobj = ext::Player_GetEntity(game_camera.x2C4 as i32);
    if !fighter_gobj.is_null() {
        cam_box = ext::ftLib_80086B74(fighter_gobj);
        if !cam_box.is_null() {
            direction = (*cam_box).facing_dir;
            if direction > 0.0f32 {
                offset_dir = 1.0f32;
            } else if direction < 0.0f32 {
                offset_dir = -1.0f32;
            } else {
                if ext::HSD_Randi(2) != 0 {
                    rand_dir = 1.0f64;
                } else {
                    rand_dir = -1.0f64;
                }
                offset_dir = rand_dir as f32;
            }
            if (4.0f32 * ext::HSD_Randf()) < 1.0f32 {
                offset_dir = -offset_dir;
            }
            if offset_dir > 0.0f32 {
                randf = ext::HSD_Randf();
                game_camera.yaw_offset = randf * ext::Stage_GetCamAngleRadiansLeft();
            } else {
                randf = ext::HSD_Randf();
                game_camera.yaw_offset = randf * (-ext::Stage_GetCamAngleRadiansRight());
            }
            randf = ext::HSD_Randf();
            game_camera.pitch_offset = (3.141592653589793f64 / (8 as f64)).mul_add(randf as f64, -(3.141592653589793f64 / (16 as f64))) as f32;
            return;
        }
    }
    game_camera.pitch_offset = (3.141592653589793f64 / (8 as f64)).mul_add(ext::HSD_Randf() as f64, -(3.141592653589793f64 / (16 as f64))) as f32;
    randf = ext::HSD_Randf();
    game_camera.yaw_offset = (6.283185307179586f64).mul_add(randf as f64, -3.141592653589793f64) as f32;
}

/// `Camera_SetModeToFixed`
pub unsafe fn Camera_SetModeToFixed() {
    game_camera.mode = (CAMERA_FIXED as i32) as u32;
}

/// `fn_8002F908`
pub unsafe extern "C" fn fn_8002F908(mut arg0: *mut _HSD_RectF32) -> i32 {
    let mut half_width: f32 = 0.0;
    let mut half_height: f32 = 0.0;
    let mut center_h: f32 = 0.0;
    let mut center_v: f32 = 0.0;
    center_h = (cm_fly_right() + cm_fly_left()) * 0.5f32;
    half_width = cm_803BCCA0.xAC * (0.5f32 * (cm_fly_right() - cm_fly_left()));
    (*arg0).ymax = center_h + half_width;
    (*arg0).ymin = center_h - half_width;
    center_v = 0.5f32 * (cm_fly_top() + cm_fly_bottom());
    half_height = cm_803BCCA0.xB0 * (0.5f32 * (cm_fly_top() - cm_fly_bottom()));
    (*arg0).xmin = center_v + half_height;
    (*arg0).xmax = center_v - half_height;
    return 1;
}

/// `Camera_8002F9E4`
pub unsafe fn Camera_8002F9E4(mut arg0: i8, mut arg1: i8) {
    let mut target_interest: *mut Vec = null_mut();
    let mut transform: *mut CameraTransformState = null_mut();
    let mut scale: f32 = 0.0;
    game_camera.mode = (CAMERA_FREE as i32) as u32;
    game_camera.x304 = Camera_8002BA00((arg0 as i32) - 1, 1) as i8;
    game_camera.x305 = arg1;
    game_camera.x32C = cm_803BCCA0.x40;
    game_camera.x2D0.x_min = cm_fly_left();
    game_camera.x2D0.x_max = cm_fly_right();
    game_camera.x2D0.y_max = cm_fly_top();
    game_camera.x2D0.y_min = cm_fly_bottom();
    game_camera.x2D0.z_max = cm_803BCCA0.xA8;
    game_camera.x2D0.z_min = cm_803BCCA0.xA4;
    game_camera.x2D0.angle_up = ext::Stage_GetCamAngleRadiansUp();
    game_camera.x2D0.angle_down = ext::Stage_GetCamAngleRadiansDown();
    game_camera.x2D0.angle_right = ext::Stage_GetCamAngleRadiansRight();
    game_camera.x2D0.angle_left = ext::Stage_GetCamAngleRadiansLeft();
    scale = fmadds(game_camera.x32C, cm_803BCCA0.x8C, cm_803BCCA0.x90);
    game_camera.x2D0.unk28 = scale * cm_803BCCA0.x94;
    game_camera.x2D0.unk2C = scale * cm_803BCCA0.x98;
    game_camera.x2D0.callback = core::mem::transmute::<Option<unsafe extern "C" fn()>, Option<unsafe extern "C" fn(*mut Camera_x2D0)>>(core::mem::transmute::<Option<unsafe extern "C" fn(*mut _HSD_RectF32) -> i32>, Option<unsafe extern "C" fn()>>(Some(fn_8002F908)));
    {
        let mut slot: i8 = game_camera.x304;
        game_camera.x314.x = { game_camera.x314.y = { game_camera.x314.z = 0.0f32; game_camera.x314.z }; game_camera.x314.y };
        game_camera.pause_eye_offset.x = 0.0f32;
        game_camera.pause_eye_offset.y = 5.0f32;
        game_camera.pause_eye_offset.z = 2e+01f32;
        game_camera.pause_up.x = 0.0f32;
        game_camera.pause_up.y = 1.0f32;
        game_camera.pause_up.z = 0.0f32;
        if (slot as i32) == 10 {
            game_camera.pause_eye_distance = 3.0f32 * scale;
        } else {
            game_camera.pause_eye_distance = scale;
        }
    }
    Camera_8002BAA8(0.0f32);
    Camera_8002BD88(0.0f32, 0.0f32);
    Camera_8002C010(0.0f32, 0.0f32);
    transform = core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState;
    target_interest = core::ptr::addr_of_mut!(game_camera.transform.target_interest) as *mut Vec;
    *target_interest = game_camera.x308;
    let _ = lbVector_Add(core::ptr::addr_of_mut!((*transform).target_interest) as *mut Vec, core::ptr::addr_of_mut!(game_camera.x314) as *mut Vec);
    game_camera.transform.target_position = *target_interest;
    let _ = lbVector_Add(core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec, core::ptr::addr_of_mut!(game_camera.pause_eye_offset) as *mut Vec);
}

/// `fn_8002FBA0`
pub unsafe extern "C" fn fn_8002FBA0(mut arg0: *mut _HSD_RectF32) -> i32 {
    let mut half_width: f32 = 0.0;
    let mut half_height: f32 = 0.0;
    let mut center_h: f32 = 0.0;
    let mut center_v: f32 = 0.0;
    center_h = (cm_fly_right() + cm_fly_left()) * 0.5f32;
    half_width = cm_803BCCA0.xAC * (0.5f32 * (cm_fly_right() - cm_fly_left()));
    (*arg0).ymax = center_h + half_width;
    (*arg0).ymin = center_h - half_width;
    center_v = 0.5f32 * (cm_fly_top() + cm_fly_bottom());
    half_height = cm_803BCCA0.xB0 * (0.5f32 * (cm_fly_top() - cm_fly_bottom()));
    (*arg0).xmin = center_v + half_height;
    (*arg0).xmax = center_v - half_height;
    return 1;
}

/// `Camera_8002FC7C`
pub unsafe fn Camera_8002FC7C(mut arg0: i8, mut arg1: i8) {
    let mut new_var: *mut CameraUnkGlobals = null_mut();
    let mut target_interest: *mut Vec = null_mut();
    let mut transform: *mut CameraTransformState = null_mut();
    let mut temp_f2: f32 = 0.0;
    let mut x304_ptr: *mut i8 = null_mut();
    let mut x304_check: i8 = 0;
    game_camera.mode = (CAMERA_FREE as i32) as u32;
    game_camera.x304 = Camera_8002BA00((arg0 as i32) - 1, 1) as i8;
    game_camera.x305 = arg1;
    game_camera.x32C = cm_803BCCA0.x40;
    game_camera.x2D0.x_min = cm_fly_left();
    game_camera.x2D0.x_max = cm_fly_right();
    game_camera.x2D0.y_max = cm_fly_top();
    game_camera.x2D0.y_min = cm_fly_bottom();
    game_camera.x2D0.z_max = (*{ new_var = core::ptr::addr_of_mut!(cm_803BCCA0) as *mut CameraUnkGlobals; new_var }).xA8;
    game_camera.x2D0.z_min = (*new_var).xA4;
    game_camera.x2D0.angle_up = ext::Stage_GetCamAngleRadiansUp();
    game_camera.x2D0.angle_down = ext::Stage_GetCamAngleRadiansDown();
    game_camera.x2D0.angle_right = ext::Stage_GetCamAngleRadiansRight();
    game_camera.x2D0.angle_left = ext::Stage_GetCamAngleRadiansLeft();
    temp_f2 = fmadds(game_camera.x32C, (*new_var).x8C, (*new_var).x90);
    game_camera.x2D0.unk28 = temp_f2 * (*new_var).x94;
    game_camera.x2D0.unk2C = temp_f2 * (*new_var).x98;
    game_camera.x2D0.callback = core::mem::transmute::<Option<unsafe extern "C" fn()>, Option<unsafe extern "C" fn(*mut Camera_x2D0)>>(core::mem::transmute::<Option<unsafe extern "C" fn(*mut _HSD_RectF32) -> i32>, Option<unsafe extern "C" fn()>>(Some(fn_8002FBA0)));
    x304_ptr = core::ptr::addr_of_mut!(game_camera.x304) as *mut i8;
    x304_check = *x304_ptr;
    game_camera.x314.z = 0.0f32;
    game_camera.x314.y = 0.0f32;
    game_camera.x314.x = 0.0f32;
    game_camera.pause_eye_offset.x = 0.0f32;
    game_camera.pause_eye_offset.y = 5.0f32;
    game_camera.pause_eye_offset.z = 2e+01f32;
    game_camera.pause_up.x = 0.0f32;
    game_camera.pause_up.y = 1.0f32;
    game_camera.pause_up.z = 0.0f32;
    if (x304_check as i32) == 10 {
        game_camera.pause_eye_distance = 3.0f32 * temp_f2;
    } else {
        game_camera.pause_eye_distance = temp_f2;
    }
    Camera_8002BAA8(0.0f32);
    Camera_8002BD88(0.0f32, 0.0f32);
    Camera_8002C010(0.0f32, 0.0f32);
    transform = core::ptr::addr_of_mut!(game_camera.transform) as *mut CameraTransformState;
    target_interest = core::ptr::addr_of_mut!(game_camera.transform.target_interest) as *mut Vec;
    *target_interest = game_camera.x308;
    let _ = lbVector_Add(core::ptr::addr_of_mut!((*transform).target_interest) as *mut Vec, core::ptr::addr_of_mut!(game_camera.x314) as *mut Vec);
    game_camera.transform.target_position = *target_interest;
    let _ = lbVector_Add(core::ptr::addr_of_mut!((*transform).target_position) as *mut Vec, core::ptr::addr_of_mut!(game_camera.pause_eye_offset) as *mut Vec);
}

/// `Camera_8002FE38`
pub unsafe fn Camera_8002FE38() {
    game_camera.mode = (CAMERA_BOSS_INTRO as i32) as u32;
    game_camera.set_x341_b1_b2((0 as u8) as u8);
    game_camera.set_x341_b3_b4((0 as u8) as u8);
    game_camera.set_x341_b5_b6((0 as u8) as u8);
    game_camera.set_x341_b7((0 as u8) as u8);
    game_camera.x350 = game_camera.transform.interest;
    game_camera.transform.target_interest = game_camera.x350;
    game_camera.x368 = game_camera.transform.position;
    game_camera.transform.target_position = game_camera.x368;
    game_camera.transform.target_fov = { game_camera.x374 = game_camera.transform.fov; game_camera.x374 };
}

/// `Camera_8002FEEC`
pub unsafe fn Camera_8002FEEC(mut arg0: i32) {
    let mut cobj: *mut HSD_CObj = null_mut();
    let mut r#box: *mut CmSubject = null_mut();
    let mut new_var: f32 = 0.0;
    let mut target: Vec3 = unsafe { core::mem::zeroed() };
    let mut eye: Vec3 = unsafe { core::mem::zeroed() };
    let mut fov: f32 = 0.0;
    let mut temp_f1_3: f32 = 0.0;
    let mut temp_f1: f32 = 0.0;
    let mut temp_f31: f32 = 0.0;
    let mut temp_f1_4: f32 = 0.0;
    if !ext::Player_GetEntity(arg0).is_null() {
        r#box = ext::ftLib_80086B74(ext::Player_GetEntity(arg0));
        if (!r#box.is_null()) && (game_camera.mode != ((CAMERA_DEBUG_FOLLOW as i32) as u32)) {
            if game_camera.mode <= ((CAMERA_PAUSE as i32) as u32) {
                cm_80453004.last_mode = game_camera.mode;
            }
            game_camera.mode = (CAMERA_DEBUG_FOLLOW as i32) as u32;
            cm_80453004.ply_slot = arg0;
            temp_f1 = ext::tanf(0.017453292f32 * game_camera.transform.target_fov);
            temp_f31 = (2.0f32 * (*r#box).ext.v.z) / temp_f1;
            cm_80453004.follow_int_offset.z = 0.0f32;
            cm_80453004.follow_int_offset.y = 0.0f32;
            cm_80453004.follow_int_offset.x = 0.0f32;
            cobj = HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj;
            ext::HSD_CObjGetInterest(cobj, core::ptr::addr_of_mut!(eye) as *mut Vec);
            ext::HSD_CObjGetEyePosition(cobj, core::ptr::addr_of_mut!(target) as *mut Vec);
            fov = ext::HSD_CObjGetFov(cobj);
            cm_80453004.follow_eye_pos = eye;
            cm_80453004.follow_int_pos = target;
            cm_80453004.follow_fov = fov;
            let _ = lbVector_Diff(core::ptr::addr_of_mut!(target) as *mut Vec, core::ptr::addr_of_mut!(eye) as *mut Vec, core::ptr::addr_of_mut!(cm_80453004.follow_eye_offset) as *mut Vec);
            temp_f1_3 = cm_80453004.follow_eye_offset.x;
            temp_f1 = temp_f1_3 * temp_f1_3;
            new_var = fmadds(cm_80453004.follow_eye_offset.z, cm_80453004.follow_eye_offset.z, fmadds(cm_80453004.follow_eye_offset.y, cm_80453004.follow_eye_offset.y, temp_f1));
            temp_f1_4 = temp_f31 / melee_runtime::gekko::sqrtf(new_var);
            cm_80453004.follow_eye_offset.x = cm_80453004.follow_eye_offset.x * temp_f1_4;
            cm_80453004.follow_eye_offset.y = cm_80453004.follow_eye_offset.y * temp_f1_4;
            cm_80453004.follow_eye_offset.z = cm_80453004.follow_eye_offset.z * temp_f1_4;
        }
    }
}

/// `Camera_8003006C`
pub unsafe fn Camera_8003006C() {
    let mut cobj: *mut HSD_CObj = null_mut();
    if game_camera.mode != ((CAMERA_DEBUG_FREE as i32) as u32) {
        if game_camera.mode <= ((CAMERA_PAUSE as i32) as u32) {
            cm_80453004.last_mode = game_camera.mode;
        }
        game_camera.mode = (CAMERA_DEBUG_FREE as i32) as u32;
        cobj = HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj;
        ext::HSD_CObjGetInterest(cobj, core::ptr::addr_of_mut!(cm_80453004.free_int_pos) as *mut Vec);
        ext::HSD_CObjGetEyePosition(cobj, core::ptr::addr_of_mut!(cm_80453004.free_eye_pos) as *mut Vec);
        cm_80453004.free_fov = ext::HSD_CObjGetFov(cobj);
    }
}

/// `Camera_800300F0`
pub unsafe fn Camera_800300F0() {
    game_camera.mode = cm_80453004.last_mode;
}

/// `Camera_8003010C`
pub unsafe fn Camera_8003010C() -> i32 {
    if (game_camera.mode as i32) == (CAMERA_FREE as i32) {
        return 1;
    }
    return 0;
}

/// `Camera_80030130`
pub unsafe fn Camera_80030130() -> i32 {
    if (game_camera.mode as i32) == (CAMERA_BOSS_INTRO as i32) {
        return 1;
    }
    return 0;
}

/// `Camera_80030154`
pub unsafe fn Camera_80030154() -> i32 {
    if (game_camera.mode as i32) == (CAMERA_DEBUG_FOLLOW as i32) {
        return 1;
    }
    return 0;
}

/// `Camera_80030178`
pub unsafe fn Camera_80030178() -> i32 {
    if (game_camera.mode as i32) == (CAMERA_DEBUG_FREE as i32) {
        return 1;
    }
    return 0;
}

/// `Camera_8003019C`
pub unsafe fn Camera_8003019C() -> *mut Vec {
    return core::ptr::addr_of_mut!((*ext::ftLib_80086B74(ext::Player_GetEntity(cm_80453004.ply_slot))).bone_pos) as *mut Vec;
}

/// `gxlink_prio8`
pub unsafe fn gxlink_prio8() -> i64 {
    let mut prio: i64 = 0;
    if ext::Wb_RenderActive() != 0 {
        return (if ext::Wb_RenderOption(WB_RENDER_STAGE as i32, 1) != 0 { 8 } else { 0 }) as i64;
    }
    if game_camera.x398_b2() != 0 {
        prio = 0 as i64;
    } else {
        prio = 8 as i64;
    }
    return prio;
}

/// `gxlink_prio1`
pub unsafe fn gxlink_prio1() -> i64 {
    let mut prio: i64 = 0;
    if game_camera.x398_b4() != 0 {
        prio = 0 as i64;
    } else {
        prio = 1 as i64;
    }
    return prio;
}

/// `gxlink_prio80`
pub unsafe fn gxlink_prio80() -> i64 {
    let mut prio: i64 = 0;
    if game_camera.x398_b3() != 0 {
        prio = 0 as i64;
    } else {
        prio = 128 as i64;
    }
    return prio;
}

/// `render_gxlink_pass`
pub unsafe fn render_gxlink_pass(mut gobj: *mut HSD_GObj, mut pass: i32, mut gxlink: i32) {
    let mut prio8: i64 = 0;
    Camera_800310A0(pass as u8);
    prio8 = gxlink_prio8();
    (*gobj).gxlink_prios = prio8 as u64;
    ext::HSD_GObj_80390ED0(gobj, gxlink as u32);
}

/// `fn_800301D0`
pub unsafe extern "C" fn fn_800301D0(mut gobj: *mut HSD_GObj, mut arg1: i32) {
    let mut prio8_a: i64 = 0;
    let mut prio1_a: i64 = 0;
    let mut cobj: *mut HSD_CObj = null_mut();
    let mut wb_saved: HSD_CObj = unsafe { core::mem::zeroed() };
    let mut wb_eye: HSD_WObj = unsafe { core::mem::zeroed() };
    let mut wb_interest: HSD_WObj = unsafe { core::mem::zeroed() };
    let mut wb_inverse: [[f32; 4]; 3] = unsafe { core::mem::zeroed() };
    let mut wb_camera: i32 = 0;
    cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    ext::lbRefract_8002247C(cobj);
    Camera_8002A4AC(gobj);
    wb_camera = (((ext::Wb_RenderActive() != 0) && (!(*cobj).eyepos.is_null())) && (!(*cobj).interest.is_null())) as i32;
    if wb_camera != 0 {
        let mut eye: Vec3 = unsafe { core::mem::zeroed() };
        let mut interest: Vec3 = unsafe { core::mem::zeroed() };
        wb_saved = *cobj;
        wb_eye = *(*cobj).eyepos;
        wb_interest = *(*cobj).interest;
        (*cobj).proj_mtx = core::ptr::addr_of_mut!(wb_inverse) as *mut [[f32; 4]; 3];
        ext::HSD_CObjSetMtxDirty(cobj);
        eye.x = cm_script_float(ext::Wb_RenderCameraBits(0));
        eye.y = cm_script_float(ext::Wb_RenderCameraBits(1));
        eye.z = cm_script_float(ext::Wb_RenderCameraBits(2));
        interest.x = cm_script_float(ext::Wb_RenderCameraBits(3));
        interest.y = cm_script_float(ext::Wb_RenderCameraBits(4));
        interest.z = cm_script_float(ext::Wb_RenderCameraBits(5));
        ext::HSD_CObjSetEyePosition(cobj, core::ptr::addr_of_mut!(eye) as *mut Vec);
        ext::HSD_CObjSetInterest(cobj, core::ptr::addr_of_mut!(interest) as *mut Vec);
        ext::HSD_CObjSetFov(cobj, cm_script_float(ext::Wb_RenderCameraBits(6)));
        ext::HSD_CObjSetRoll(cobj, cm_script_float(ext::Wb_RenderCameraBits(7)));
        ext::HSD_CObjSetNear(cobj, 0.1f32);
        ext::HSD_CObjSetFar(cobj, 16384.0f32);
    }
    ext::lbShadow_8000F38C(game_camera.x399_b7() as i32);
    if ext::HSD_CObjSetCurrent(cobj) != 0 {
        if !(game_camera.x398_b5() != 0) {
            ext::HSD_SetEraseColor(game_camera.background_color.r, game_camera.background_color.g, game_camera.background_color.b, 255 as u8);
        } else {
            ext::HSD_SetEraseColor(0 as u8, 0 as u8, 0 as u8, 255 as u8);
        }
        ext::HSD_CObjEraseScreen(cobj, 1, 0, 0);
        ext::Ground_801C4FAC(cobj);
        ext::HSD_LObjDeleteCurrentAll(null_mut());
        Camera_800310A0(2 as u8);
        prio8_a = gxlink_prio8();
        prio1_a = gxlink_prio1();
        (*gobj).gxlink_prios = (prio1_a | prio8_a) as u64;
        ext::HSD_GObj_80390ED0(gobj, 7 as u32);
        render_gxlink_pass(gobj, 1, 7);
        render_gxlink_pass(gobj, 0, 3);
        ext::lbRefract_80022560();
        Camera_800311EC(gobj, gxlink_prio1() as u64);
        Camera_80031074(0 as u8);
        prio8_a = gxlink_prio8();
        prio1_a = gxlink_prio1();
        (*gobj).gxlink_prios = (prio1_a | prio8_a) as u64;
        ext::HSD_GObj_80390ED0(gobj, 4 as u32);
        Camera_80031328(gobj, gxlink_prio1() as u64);
        ext::HSD_FogSet(null_mut());
        (*gobj).gxlink_prios = gxlink_prio80() as u64;
        ext::HSD_GObj_80390ED0(gobj, 7 as u32);
        render_gxlink_pass(gobj, 3, 7);
        {
            ext::MotionWorldDraw((core::ptr::addr_of_mut!((*ext::HSD_CObjGetCurrent()).view_mtx) as *mut [[f32; 4]; 3]) as usize as i32);
            ext::Shader_PostDraw(0);
        }
        if wb_camera != 0 {
            ext::ScriptGame_WbDraw();
            ext::Wb_RenderDrawn();
        } else if Camera_80030AC4() != 0 {
            if Camera_80030A78() != 0 {
                ext::mpLib_8005A2DC();
            }
            if game_camera.x399_b4() != 0 {
                ext::mpLib_DrawZones();
            }
            if Camera_80030BA8() != 0 {
                ext::mpLib_DrawSpecialPoints();
            }
        }
        ext::HSD_CObjEndCurrent();
    }
    if wb_camera != 0 {
        *cobj = wb_saved;
        *(*cobj).eyepos = wb_eye;
        *(*cobj).interest = wb_interest;
    }
}

/// `Camera_800304E0`
pub unsafe fn Camera_800304E0(mut gobj: *mut HSD_GObj) {
    let mut prio8: i64 = 0;
    let mut prio1: i64 = 0;
    let mut cobj: *mut HSD_CObj = null_mut();
    cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    if ext::HSD_CObjSetCurrent(cobj) != 0 {
        ext::HSD_SetEraseColor(game_camera.background_color.r, game_camera.background_color.g, game_camera.background_color.b, 1 as u8);
        ext::HSD_CObjEraseScreen(cobj, 1, 0, 0);
        ext::Ground_801C4FAC(cobj);
        ext::HSD_LObjDeleteCurrentAll(null_mut());
        Camera_800310A0(2 as u8);
        prio8 = gxlink_prio8();
        prio1 = gxlink_prio1();
        (*gobj).gxlink_prios = (prio1 | prio8) as u64;
        ext::HSD_GObj_80390ED0(gobj, 7 as u32);
        render_gxlink_pass(gobj, 1, 7);
        render_gxlink_pass(gobj, 0, 7);
        (*gobj).gxlink_prios = 112 as u64;
        ext::HSD_GObj_80390ED0(gobj, 7 as u32);
        ext::HSD_FogSet(null_mut());
        (*gobj).gxlink_prios = 128 as u64;
        ext::HSD_GObj_80390ED0(gobj, 7 as u32);
        ext::HSD_CObjEndCurrent();
    }
}

/// `Camera_Create`
pub unsafe fn Camera_Create() {
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut cobj: *mut HSD_CObj = null_mut();
    gobj = ext::GObj_Create(16 as u16, 18 as u8, 0 as u8);
    game_camera.gobj = gobj;
    cobj = ext::lb_80013B14(core::ptr::addr_of_mut!(cm_803BCB64) as *mut HSD_CameraDescPerspective);
    cm_804D6464 = ext::lb_80013B14(core::ptr::addr_of_mut!(cm_803BCB64) as *mut HSD_CameraDescPerspective);
    {
        let mut c: *mut HSD_CObj = cobj;
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, c as *mut c_void);
    }
    ext::GObj_SetupGXLinkMax(gobj, Some(fn_800301D0), 2 as u32);
    let _ = ext::HSD_GObj_SetupProc(gobj, Some(fn_8002F360), 18 as u8);
}

/// `Camera_80030730`
pub unsafe fn Camera_80030730(mut arg8: f32) {
    cm_803BCCA0.x40 = arg8;
}

/// `Camera_SetBackgroundColor`
pub unsafe fn Camera_SetBackgroundColor(mut r: u8, mut g: u8, mut b: u8) {
    game_camera.background_color.r = r;
    game_camera.background_color.g = g;
    game_camera.background_color.b = b;
}

/// `Camera_GetBackgroundColor`
pub unsafe fn Camera_GetBackgroundColor() -> GXColor {
    let mut color: GXColor = unsafe { core::mem::zeroed() };
    color.r = game_camera.background_color.r;
    color.g = game_camera.background_color.g;
    color.b = game_camera.background_color.b;
    return color;
}

/// `Camera_GetTransformPosition`
pub unsafe fn Camera_GetTransformPosition(mut arg0: *mut Vec) {
    *arg0 = game_camera.transform.position;
}

/// `Camera_GetTransformInterest`
pub unsafe fn Camera_GetTransformInterest(mut arg0: *mut Vec) {
    *arg0 = game_camera.transform.interest;
}

/// `project_ground_x`
pub unsafe fn project_ground_x(mut value: *mut f32, mut eye_pos: *mut Vec, mut dir_x: f32, mut dir_z: f32) {
    *value = -fmadds(dir_x, (*eye_pos).z / dir_z, -(*eye_pos).x);
}

/// `same_side`
pub unsafe fn same_side(mut fwd_z: f32, mut dir_z: f32) -> i32 {
    return (((fwd_z * dir_z) as f64) > 0.0f64) as i32;
}

/// `Camera_Bounds`
pub unsafe fn Camera_Bounds(mut left: *mut f32, mut center: *mut f32, mut right: *mut f32, mut logic: i32) -> i32 {
    let mut cobj: *mut HSD_CObj = null_mut();
    let mut half_fov: f32 = 0.0;
    let mut forward: Vec3 = unsafe { core::mem::zeroed() };
    let mut interest_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut eye_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut result: i32 = 0;
    let mut s: f32 = 0.0;
    let mut c: f32 = 0.0;
    let mut edge_x: f32 = 0.0;
    let mut edge_z: f32 = 0.0;
    let mut s2: f32 = 0.0;
    let mut c2: f32 = 0.0;
    let mut edge_x2: f32 = 0.0;
    let mut edge_z2: f32 = 0.0;
    cobj = HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj;
    if logic != 0 {
        half_fov = (0.5f32 * (ext::HSD_CObjGetFov(cobj) * 0.017453292f32)) * ext::HSD_CObjGetAuthoredAspect(cobj);
    } else {
        half_fov = ext::atanf(ext::tanf(0.5f32 * (ext::HSD_CObjGetFov(cobj) * 0.017453292f32)) * ext::HSD_CObjGetAspect(cobj));
    }
    result = 1;
    ext::HSD_CObjGetEyePosition(cobj, core::ptr::addr_of_mut!(eye_pos) as *mut Vec);
    let _ = ext::HSD_CObjGetEyeVector(cobj, core::ptr::addr_of_mut!(forward) as *mut Vec);
    ext::HSD_CObjGetInterest(cobj, core::ptr::addr_of_mut!(interest_pos) as *mut Vec);
    if (((if forward.x < (0 as f32) { -forward.x } else { forward.x }) as f64) > 0.0001f64) && (((if forward.z < (0 as f32) { -forward.z } else { forward.z }) as f64) > 0.0001f64) {
        forward.x = forward.x * (-1.0f32);
        forward.y = forward.y * (-1.0f32);
        forward.z = forward.z * (-1.0f32);
        project_ground_x(center, core::ptr::addr_of_mut!(eye_pos) as *mut Vec, forward.x, forward.z);
        s = melee_runtime::gekko::sinf(half_fov);
        c = melee_runtime::gekko::cosf(half_fov);
        edge_x = fmadds(forward.x, c, forward.z * s);
        edge_z = fmadds(forward.z, c, -(forward.x * s));
        if (((if edge_z < (0 as f32) { -edge_z } else { edge_z }) as f64) > 0.0001f64) && (same_side(forward.z, edge_z) != 0) {
            project_ground_x(left, core::ptr::addr_of_mut!(eye_pos) as *mut Vec, edge_x, edge_z);
        } else if edge_x > 0.0f32 {
            *left = if logic != 0 { 8.5070587e+37f32 } else { -8.5070587e+37f32 };
        } else {
            *left = if logic != 0 { -8.5070587e+37f32 } else { 8.5070587e+37f32 };
        }
        s2 = melee_runtime::gekko::sinf(-half_fov);
        c2 = melee_runtime::gekko::cosf(-half_fov);
        edge_x2 = fmadds(forward.x, c2, forward.z * s2);
        edge_z2 = fmadds(forward.z, c2, -(forward.x * s2));
        if (((if edge_z2 < (0 as f32) { -edge_z2 } else { edge_z2 }) as f64) > 0.0001f64) && (same_side(forward.z, edge_z2) != 0) {
            project_ground_x(right, core::ptr::addr_of_mut!(eye_pos) as *mut Vec, edge_x2, edge_z2);
        } else if edge_x2 > 0.0f32 {
            *right = if logic != 0 { 8.5070587e+37f32 } else { -8.5070587e+37f32 };
        } else {
            *right = if logic != 0 { -8.5070587e+37f32 } else { 8.5070587e+37f32 };
        }
    } else {
        *left = -8.5070587e+37f32;
        *right = 8.5070587e+37f32;
        *center = 0.0f32;
        result = 0;
    }
    return result;
}

/// `Camera_800307D0`
pub unsafe fn Camera_800307D0(mut left: *mut f32, mut center: *mut f32, mut right: *mut f32) -> i32 {
    return Camera_Bounds(left, center, right, 0);
}

/// `Camera_LogicBounds`
pub unsafe fn Camera_LogicBounds(mut left: *mut f32, mut center: *mut f32, mut right: *mut f32) -> i32 {
    return Camera_Bounds(left, center, right, 1);
}

/// `Camera_ForgetGameCamera`
pub unsafe fn Camera_ForgetGameCamera() {
    game_camera.gobj = null_mut();
    core::ptr::write_bytes(((core::ptr::addr_of_mut!(cm_script) as *mut cm_script_t) as *mut c_void) as *mut u8, 0, (2580_u64 as i32) as usize);
}

/// `cm_script_lerp`
pub unsafe fn cm_script_lerp(mut a: f32, mut b: f32, mut t: f32) -> f32 {
    return fmadds(b - a, t, a);
}

/// `cm_script_restore_up`
pub unsafe fn cm_script_restore_up() {
    if !game_camera.gobj.is_null() {
        let mut c: *mut HSD_CObj = HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj;
        if cm_script.saved_up_vector != 0 {
            ext::HSD_CObjSetUpVector(c, core::ptr::addr_of_mut!(cm_script.saved_up) as *mut Vec);
        } else {
            ext::HSD_CObjSetRoll(c, cm_script.saved_roll);
        }
        ext::HSD_CObjSetFar(c, cm_script.saved_far);
    }
}

/// `cm_script_blend`
pub unsafe fn cm_script_blend(mut a: CmScriptPose, mut b: CmScriptPose, mut t: f32) -> CmScriptPose {
    let mut p: CmScriptPose = unsafe { core::mem::zeroed() };
    p.eye.x = cm_script_lerp(a.eye.x, b.eye.x, t);
    p.eye.y = cm_script_lerp(a.eye.y, b.eye.y, t);
    p.eye.z = cm_script_lerp(a.eye.z, b.eye.z, t);
    p.interest.x = cm_script_lerp(a.interest.x, b.interest.x, t);
    p.interest.y = cm_script_lerp(a.interest.y, b.interest.y, t);
    p.interest.z = cm_script_lerp(a.interest.z, b.interest.z, t);
    p.fov = cm_script_lerp(a.fov, b.fov, t);
    p.roll = cm_script_lerp(a.roll, b.roll, t);
    return p;
}

/// `cm_script_ease`
pub unsafe fn cm_script_ease(mut t: f32, mut ease: i32) -> f32 {
    if ease == 1 {
        return t * t;
    }
    if ease == 2 {
        return t * (2.0f32 - t);
    }
    if ease == 3 {
        return if t < 0.5f32 { (2.0f32 * t) * t } else { fmadds(-(2.0f32 * (1.0f32 - t)), 1.0f32 - t, 1.0f32) };
    }
    return t;
}

/// `cm_script_update`
pub unsafe fn cm_script_update() {
    let mut normal: CmScriptPose = unsafe { core::mem::zeroed() };
    let mut target: Vec3 = unsafe { core::mem::zeroed() };
    let mut i: i32 = 0;
    let mut t: f32 = 0.0;
    if !(cm_script.state != 0) {
        return;
    }
    cm_script.normal = game_camera.transform;
    normal.eye = game_camera.transform.position;
    normal.interest = game_camera.transform.interest;
    normal.fov = game_camera.transform.fov;
    normal.roll = cm_script.saved_roll;
    if cm_script.state == 2 {
        cm_script.elapsed = cm_script.elapsed + 1;
        t = (cm_script.elapsed as f32) / (cm_script.duration as f32);
        if t >= 1.0f32 {
            cm_script.state = 0;
            cm_script.lift_bounds = 0;
            cm_script_restore_up();
            return;
        }
        cm_script.pose = cm_script_blend(cm_script.from, normal, t);
    } else if cm_script.follow_kind != 0 {
        target = cm_script.follow_point;
        if (cm_script.follow_kind == 1) || (ext::ScriptGame_CameraFollowTarget(cm_script.follow_kind - 1, cm_script.follow_id, core::ptr::addr_of_mut!(target) as *mut Vec) != 0) {
            target.x = target.x + cm_script.follow_offset.x;
            target.y = target.y + cm_script.follow_offset.y;
            target.z = target.z + cm_script.follow_offset.z;
            cm_script.pose.interest = target;
            cm_script.pose.eye.x = target.x + cm_script.follow_eye_delta.x;
            cm_script.pose.eye.y = target.y + cm_script.follow_eye_delta.y;
            cm_script.pose.eye.z = target.z + cm_script.follow_eye_delta.z;
        }
    } else if cm_script.path_count != 0 {
        cm_script.path_frame = cm_script.path_frame + 1;
        {
            i = 0;
            'l78: while (i < cm_script.path_count) && (cm_script.path[i as usize].frame < cm_script.path_frame) {
                i = i + 1;
            }
        }
        if i == cm_script.path_count {
            cm_script.pose = cm_script.path[(i - 1) as usize].pose;
            cm_script.path_count = 0;
            cm_script.completion = cm_script.completion + 1;
            cm_script.completion_kind = 2;
        } else if i == 0 {
            cm_script.pose = cm_script.path[0 as usize].pose;
        } else {
            let mut begin: i32 = cm_script.path[(i - 1) as usize].frame;
            t = ((cm_script.path_frame - begin) as f32) / ((cm_script.path[i as usize].frame - begin) as f32);
            cm_script.pose = cm_script_blend(cm_script.path[(i - 1) as usize].pose, cm_script.path[i as usize].pose, t);
            if (i == (cm_script.path_count - 1)) && (cm_script.path_frame == cm_script.path[i as usize].frame) {
                cm_script.path_count = 0;
                cm_script.completion = cm_script.completion + 1;
                cm_script.completion_kind = 2;
            }
        }
    } else if cm_script.duration != 0 {
        cm_script.elapsed = cm_script.elapsed + 1;
        t = (cm_script.elapsed as f32) / (cm_script.duration as f32);
        if t >= 1.0f32 {
            t = 1.0f32;
        }
        cm_script.pose = cm_script_blend(cm_script.from, cm_script.to, cm_script_ease(t, cm_script.ease));
        if cm_script.elapsed >= cm_script.duration {
            cm_script.duration = 0;
            cm_script.completion = cm_script.completion + 1;
            cm_script.completion_kind = 1;
        }
    }
    game_camera.transform.position = cm_script.pose.eye;
    game_camera.transform.interest = cm_script.pose.interest;
    game_camera.transform.fov = cm_script.pose.fov;
    if cm_script.shake_left > 0 {
        let mut amplitude: f32 = cm_script.shake_intensity * ((cm_script.shake_left as f32) / (cm_script.shake_total as f32));
        let mut sx: f32 = melee_runtime::gekko::sinf((cm_script.shake_tick as f32) * 2.4f32) * amplitude;
        let mut sy: f32 = melee_runtime::gekko::sinf((cm_script.shake_tick as f32) * 3.7f32) * amplitude;
        game_camera.transform.position.x = game_camera.transform.position.x + sx;
        game_camera.transform.position.y = game_camera.transform.position.y + sy;
        game_camera.transform.interest.x = game_camera.transform.interest.x + sx;
        game_camera.transform.interest.y = game_camera.transform.interest.y + sy;
        cm_script.shake_tick = cm_script.shake_tick + 1;
        cm_script.shake_left = cm_script.shake_left - 1;
    }
}

/// `Camera_ScriptDetach`
pub unsafe fn Camera_ScriptDetach() -> i32 {
    let mut c: *mut HSD_CObj = null_mut();
    if game_camera.gobj.is_null() {
        return 0;
    }
    if cm_script.state != 0 {
        return 1;
    }
    c = HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj;
    ext::HSD_CObjGetEyePosition(c, core::ptr::addr_of_mut!(cm_script.pose.eye) as *mut Vec);
    ext::HSD_CObjGetInterest(c, core::ptr::addr_of_mut!(cm_script.pose.interest) as *mut Vec);
    cm_script.pose.fov = ext::HSD_CObjGetFov(c);
    cm_script.saved_roll = if ((*c).flags & (1 as u32)) != 0 { 0.0f32 } else { (*c).u.roll };
    cm_script.saved_far = ext::HSD_CObjGetFar(c);
    cm_script.saved_up_vector = (((*c).flags & (1 as u32)) != (0 as u32)) as i32;
    let _ = ext::HSD_CObjGetUpVector(c, core::ptr::addr_of_mut!(cm_script.saved_up) as *mut Vec);
    cm_script.pose.roll = cm_script.saved_roll;
    cm_script.normal = game_camera.transform;
    cm_script.state = 1;
    cm_script.lift_bounds = 0;
    cm_script.duration = { cm_script.path_count = { cm_script.follow_kind = { cm_script.path_building = 0; cm_script.path_building }; cm_script.follow_kind }; cm_script.path_count };
    return 1;
}

/// `Camera_ScriptAttach`
pub unsafe fn Camera_ScriptAttach(mut frames: i32) {
    if !(cm_script.state != 0) {
        return;
    }
    cm_script.duration = { cm_script.path_count = { cm_script.follow_kind = { cm_script.path_building = 0; cm_script.path_building }; cm_script.follow_kind }; cm_script.path_count };
    cm_script.shake_left = { cm_script.lift_bounds = 0; cm_script.lift_bounds };
    if frames <= 0 {
        cm_script.state = 0;
        game_camera.transform = cm_script.normal;
        cm_script_restore_up();
        if !game_camera.gobj.is_null() {
            Camera_8002A4AC(game_camera.gobj);
            let _ = ext::HSD_CObjGetViewingMtxPtr(HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj);
        }
    } else {
        cm_script.state = 2;
        cm_script.from = cm_script.pose;
        cm_script.duration = frames;
        cm_script.elapsed = 0;
    }
}

/// `Camera_ScriptReset`
pub unsafe fn Camera_ScriptReset() {
    Camera_ScriptAttach(0);
}

/// `Camera_ScriptState`
pub unsafe fn Camera_ScriptState() -> i32 {
    return cm_script.state;
}

/// `Camera_ScriptMode`
pub unsafe fn Camera_ScriptMode() -> i32 {
    return game_camera.mode as i32;
}

/// `Camera_ScriptCompletion`
pub unsafe fn Camera_ScriptCompletion() -> i32 {
    return cm_script.completion;
}

/// `Camera_ScriptCompletionKind`
pub unsafe fn Camera_ScriptCompletionKind() -> i32 {
    return cm_script.completion_kind;
}

/// `Camera_ScriptGetBits`
pub unsafe fn Camera_ScriptGetBits(mut field: i32) -> i32 {
    let mut p: CmScriptPose = unsafe { core::mem::zeroed() };
    let mut c: *mut HSD_CObj = null_mut();
    if game_camera.gobj.is_null() {
        return 0;
    }
    if cm_script.state != 0 {
        p = cm_script.pose;
    } else {
        c = HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj;
        ext::HSD_CObjGetEyePosition(c, core::ptr::addr_of_mut!(p.eye) as *mut Vec);
        ext::HSD_CObjGetInterest(c, core::ptr::addr_of_mut!(p.interest) as *mut Vec);
        p.fov = ext::HSD_CObjGetFov(c);
        p.roll = if ((*c).flags & (1 as u32)) != 0 { 0.0f32 } else { (*c).u.roll };
    }
    's79: {
        match field {
            0 => {
                return cm_script_bits(p.eye.x);
            }
            1 => {
                return cm_script_bits(p.eye.y);
            }
            2 => {
                return cm_script_bits(p.eye.z);
            }
            3 => {
                return cm_script_bits(p.interest.x);
            }
            4 => {
                return cm_script_bits(p.interest.y);
            }
            5 => {
                return cm_script_bits(p.interest.z);
            }
            6 => {
                return cm_script_bits(p.fov);
            }
            7 => {
                return cm_script_bits(p.roll);
            }
            _ => {}
        }
    }
    return 0;
}

/// `Camera_ScriptSet`
pub unsafe fn Camera_ScriptSet(mut field: i32, mut bits: i32) {
    let mut v: f32 = cm_script_float(bits);
    if cm_script.state != 1 {
        return;
    }
    cm_script.duration = { cm_script.follow_kind = 0; cm_script.follow_kind };
    if !(cm_script.path_building != 0) {
        cm_script.path_count = 0;
    }
    's80: {
        match field {
            0 => {
                cm_script.pose.eye.x = v;
            }
            1 => {
                cm_script.pose.eye.y = v;
            }
            2 => {
                cm_script.pose.eye.z = v;
            }
            3 => {
                cm_script.pose.interest.x = v;
            }
            4 => {
                cm_script.pose.interest.y = v;
            }
            5 => {
                cm_script.pose.interest.z = v;
            }
            6 => {
                cm_script.pose.fov = v;
            }
            7 => {
                cm_script.pose.roll = v;
            }
            _ => {}
        }
    }
}

/// `Camera_ScriptMove`
pub unsafe fn Camera_ScriptMove(mut frames: i32, mut ease: i32) {
    if (cm_script.state != 1) || (frames < 1) {
        return;
    }
    cm_script.to = cm_script.pose;
    cm_script.pose = cm_script.from;
    cm_script.duration = frames;
    cm_script.elapsed = 0;
    cm_script.ease = ease;
    cm_script.path_count = { cm_script.follow_kind = { cm_script.path_building = 0; cm_script.path_building }; cm_script.follow_kind };
}

/// `Camera_ScriptMoveBegin`
pub unsafe fn Camera_ScriptMoveBegin() {
    cm_script.from = cm_script.pose;
}

/// `Camera_ScriptPathClear`
pub unsafe fn Camera_ScriptPathClear() {
    cm_script.path_count = { cm_script.duration = { cm_script.follow_kind = 0; cm_script.follow_kind }; cm_script.duration };
    cm_script.path_building = 1;
}

/// `Camera_ScriptPathKey`
pub unsafe fn Camera_ScriptPathKey(mut frame: i32) -> i32 {
    if (((cm_script.state != 1) || (cm_script.path_count >= 64)) || (frame < 0)) || ((cm_script.path_count != 0) && (frame <= cm_script.path[(cm_script.path_count - 1) as usize].frame)) {
        return 0;
    }
    cm_script.path[cm_script.path_count as usize].pose = cm_script.pose;
    cm_script.path[cm_script.path_count as usize].frame = frame;
    cm_script.path_count = cm_script.path_count + 1;
    return 1;
}

/// `Camera_ScriptPathStart`
pub unsafe fn Camera_ScriptPathStart() {
    if (cm_script.state != 1) || (cm_script.path_count < 2) {
        return;
    }
    cm_script.path_frame = 0;
    cm_script.pose = cm_script.path[0 as usize].pose;
    cm_script.path_building = 0;
    cm_script.duration = { cm_script.follow_kind = 0; cm_script.follow_kind };
}

/// `Camera_ScriptFollow`
pub unsafe fn Camera_ScriptFollow(mut kind: i32, mut id: i32, mut px: i32, mut py: i32, mut pz: i32, mut ox: i32, mut oy: i32, mut oz: i32) -> i32 {
    let mut target: Vec3 = unsafe { core::mem::zeroed() };
    if cm_script.state != 1 {
        return 0;
    }
    target.x = cm_script_float(px);
    target.y = cm_script_float(py);
    target.z = cm_script_float(pz);
    if (kind != 0) && (!(ext::ScriptGame_CameraFollowTarget(kind, id, core::ptr::addr_of_mut!(target) as *mut Vec) != 0)) {
        return 0;
    }
    cm_script.follow_point.x = cm_script_float(px);
    cm_script.follow_point.y = cm_script_float(py);
    cm_script.follow_point.z = cm_script_float(pz);
    cm_script.follow_offset.x = cm_script_float(ox);
    cm_script.follow_offset.y = cm_script_float(oy);
    cm_script.follow_offset.z = cm_script_float(oz);
    cm_script.follow_eye_delta.x = cm_script.pose.eye.x - cm_script.pose.interest.x;
    cm_script.follow_eye_delta.y = cm_script.pose.eye.y - cm_script.pose.interest.y;
    cm_script.follow_eye_delta.z = cm_script.pose.eye.z - cm_script.pose.interest.z;
    cm_script.follow_kind = kind + 1;
    cm_script.follow_id = id;
    cm_script.duration = { cm_script.path_count = 0; cm_script.path_count };
    return 1;
}

/// `Camera_ScriptShake`
pub unsafe fn Camera_ScriptShake(mut intensity_bits: i32, mut frames: i32) {
    cm_script.shake_intensity = cm_script_float(intensity_bits);
    cm_script.shake_left = { cm_script.shake_total = frames; cm_script.shake_total };
    cm_script.shake_tick = 1;
}

/// `Camera_ScriptLiftBounds`
pub unsafe fn Camera_ScriptLiftBounds(mut lift: i32) {
    cm_script.lift_bounds = ((cm_script.state == 1) && (lift != 0)) as i32;
}

/// `Camera_RefreshViewingMtx`
pub unsafe fn Camera_RefreshViewingMtx() {
    if !game_camera.gobj.is_null() {
        Camera_8002A4AC(game_camera.gobj);
        let _ = ext::HSD_CObjGetViewingMtxPtr(HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj);
    }
}

/// `Camera_ScriptCommit`
pub unsafe fn Camera_ScriptCommit() {
    if (cm_script.state != 1) || (game_camera.gobj.is_null()) {
        return;
    }
    game_camera.transform.position = cm_script.pose.eye;
    game_camera.transform.interest = cm_script.pose.interest;
    game_camera.transform.fov = cm_script.pose.fov;
    Camera_RefreshViewingMtx();
}

/// `Camera_80030A50`
pub unsafe fn Camera_80030A50() -> *mut HSD_GObj {
    if !game_camera.gobj.is_null() {
        ext::HSD_CObjUpdateView(HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj);
    }
    return game_camera.gobj;
}

/// `Camera_80030A60`
pub unsafe fn Camera_80030A60(mut arg0: i32) {
    game_camera.set_x399_b3((arg0 as u8) as u8);
}

/// `Camera_80030A78`
pub unsafe fn Camera_80030A78() -> i32 {
    return game_camera.x399_b3() as i32;
}

/// `Camera_80030A8C`
pub unsafe fn Camera_80030A8C(mut arg0: i32) {
    game_camera.set_x399_b4((arg0 as u8) as u8);
}

/// `Camera_SetStageVisible`
pub unsafe fn Camera_SetStageVisible(mut arg0: i32) {
    let mut temp: i32 = 0;
    game_camera.set_x399_b5(((({ temp = 0; temp } == arg0) as i32) as u8) as u8);
}

/// `Camera_80030AC4`
pub unsafe fn Camera_80030AC4() -> i32 {
    let mut temp: i32 = 0;
    return ({ temp = 0; temp } == (game_camera.x399_b5() as i32)) as i32;
}

/// `Camera_80030AE0`
pub unsafe fn Camera_80030AE0(mut arg0: i32) {
    game_camera.set_x399_b2((arg0 as u8) as u8);
}

/// `Camera_80030AF8`
pub unsafe fn Camera_80030AF8() -> i32 {
    return game_camera.x399_b2() as i32;
}

/// `Camera_80030B0C`
pub unsafe fn Camera_80030B0C(mut arg0: i32) {
    game_camera.set_x399_b7((arg0 as u8) as u8);
}

/// `Camera_80030B24`
pub unsafe fn Camera_80030B24() -> i32 {
    return game_camera.x399_b7() as i32;
}

/// `Camera_80030B38`
pub unsafe fn Camera_80030B38(mut arg0: i32) {
    game_camera.set_x39A_b0((arg0 as u8) as u8);
}

/// `Camera_80030B50`
pub unsafe fn Camera_80030B50() -> i32 {
    return game_camera.x39A_b0() as i32;
}

/// `Camera_80030B64`
pub unsafe fn Camera_80030B64(mut arg0: i32) {
    game_camera.set_x39A_b1((arg0 as u8) as u8);
}

/// `Camera_80030B7C`
pub unsafe fn Camera_80030B7C() -> i32 {
    return game_camera.x39A_b1() as i32;
}

/// `Camera_80030B90`
pub unsafe fn Camera_80030B90(mut arg0: i32) {
    game_camera.set_x39A_b2((arg0 as u8) as u8);
}

/// `Camera_80030BA8`
pub unsafe fn Camera_80030BA8() -> i32 {
    return game_camera.x39A_b2() as i32;
}

/// `Camera_ProjectToScreen`
pub unsafe fn Camera_ProjectToScreen(mut cobj: *mut HSD_CObj, mut arg0: *mut Vec, mut arg1: *mut S32Vec2) -> i32 {
    let mut point: Vec3 = unsafe { core::mem::zeroed() };
    let mut scissor: Scissor = unsafe { core::mem::zeroed() };
    let mut px: i32 = 0;
    let mut py: i32 = 0;
    if ext::lbVector_WorldToScreen(cobj, arg0 as *const Vec, core::ptr::addr_of_mut!(point) as *mut Vec, 1).is_null() {
        return 0;
    }
    if (((point.x > 2.1474836e+09f32) || (point.x < (-2.1474836e+09f32))) || (point.y > 2.1474836e+09f32)) || (point.y < (-2.1474836e+09f32)) {
        return 0;
    }
    px = point.x as i32;
    py = point.y as i32;
    if !arg1.is_null() {
        (*arg1).x = px;
        (*arg1).y = py;
    }
    ext::HSD_CObjGetScissor(cobj, core::ptr::addr_of_mut!(scissor) as *mut _Scissor);
    if (((px < (scissor.left as i32)) || (px >= (scissor.right as i32))) || (py < (scissor.top as i32))) || (py >= (scissor.bottom as i32)) {
        return 0;
    }
    return 1;
}

/// `Camera_80030BBC`
pub unsafe fn Camera_80030BBC(mut arg0: *mut Vec, mut arg1: *mut S32Vec2) -> i32 {
    return Camera_ProjectToScreen(HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj, arg0, arg1);
}

/// `Camera_LogicToScreen`
pub unsafe fn Camera_LogicToScreen(mut subject: *mut CmSubject, mut point: *mut S32Vec2) -> i32 {
    let mut source: *mut HSD_CObj = HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj;
    let mut camera: HSD_CObj = *source;
    camera.view_auto = 0 as u8;
    camera.projection_param.perspective.aspect = ext::HSD_CObjGetAuthoredAspect(source);
    return Camera_ProjectToScreen(core::ptr::addr_of_mut!(camera) as *mut HSD_CObj, core::ptr::addr_of_mut!((*subject).bone_pos) as *mut Vec, point);
}

/// `Camera_80030CD8`
pub unsafe fn Camera_80030CD8(mut arg0: *mut CmSubject, mut arg1: *mut S32Vec2) -> i32 {
    return Camera_80030BBC(core::ptr::addr_of_mut!((*arg0).bone_pos) as *mut Vec, arg1);
}

/// `Camera_80030CFC`
pub unsafe fn Camera_80030CFC(mut cam_box: *mut CmSubject, mut tolerance: f32) -> i32 {
    let mut cobj: *mut HSD_CObj = null_mut();
    let mut eye_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut interest: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp38: Vec3 = unsafe { core::mem::zeroed() };
    let mut _PAD: [u8; 12] = [0; 12];
    let mut sp20: Vec3 = unsafe { core::mem::zeroed() };
    let mut range: f32 = 0.0;
    cobj = HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj;
    ext::HSD_CObjGetEyePosition(cobj, core::ptr::addr_of_mut!(eye_pos) as *mut Vec);
    ext::HSD_CObjGetInterest(cobj, core::ptr::addr_of_mut!(interest) as *mut Vec);
    if ext::lbVector_8000E838(core::ptr::addr_of_mut!(interest) as *mut Vec, core::ptr::addr_of_mut!(eye_pos) as *mut Vec, core::ptr::addr_of_mut!((*cam_box).bone_pos) as *mut Vec, core::ptr::addr_of_mut!(sp38) as *mut Vec) < ((*cam_box).ext.v.z + tolerance) {
        return 1;
    }
    let _ = lbVector_Diff(core::ptr::addr_of_mut!(sp38) as *mut Vec, core::ptr::addr_of_mut!((*cam_box).bone_pos) as *mut Vec, core::ptr::addr_of_mut!(sp20) as *mut Vec);
    let _ = ext::lbVector_Normalize(core::ptr::addr_of_mut!(sp20) as *mut Vec);
    range = (*cam_box).ext.v.z + tolerance;
    sp20.x = sp20.x * range;
    sp20.y = sp20.y * range;
    sp20.z = sp20.z * range;
    let _ = lbVector_Add(core::ptr::addr_of_mut!(sp20) as *mut Vec, core::ptr::addr_of_mut!((*cam_box).bone_pos) as *mut Vec);
    return Camera_80030BBC(core::ptr::addr_of_mut!(sp20) as *mut Vec, null_mut());
}

/// `Camera_80030DE4`
pub unsafe fn Camera_80030DE4(mut arg8: f32, mut arg9: f32) {
    game_camera.translation.x = arg8;
    game_camera.translation.y = arg9;
}

/// `Camera_80030DF8`
pub unsafe fn Camera_80030DF8() {
    game_camera.translation.x = { game_camera.translation.y = 0 as f32; game_camera.translation.y };
}

/// `Camera_80030E10`
pub unsafe fn Camera_80030E10() -> f32 {
    if (game_camera.x2B8 as i32) < 1 {
        return 1e+04f32;
    }
    return game_camera.x2B0;
}

/// `Camera_SetQuakeScale`
pub unsafe fn Camera_SetQuakeScale(mut scale: f32) {
    game_camera.quake_scale = scale;
}

/// `Camera_RequestQuake`
pub unsafe fn Camera_RequestQuake(mut kind: CmQuakeKind, mut pos: *mut Vec) {
    let mut pquake: *mut *mut HSD_GObj = null_mut();
    let mut quake_length: i32 = 0;
    's81: {
        match kind {
            1 => {
                pquake = core::ptr::addr_of_mut!(game_camera.quake_gobj) as *mut *mut HSD_GObj;
                if game_camera.quake_gobj.is_null() {
                    *pquake = ext::grLib_801C9CEC(kind);
                }
                quake_length = 10;
            }
            2 => {
                let _ = ext::grLib_801C9CEC(kind);
                quake_length = 22;
            }
            3 => {
                let _ = ext::grLib_801C9CEC(kind);
                quake_length = 22;
            }
            4 => {
                let _ = ext::grLib_801C9CEC(kind);
                quake_length = 22;
            }
            _ => {
            }
        }
    }
    game_camera.quake_frames_left[kind as usize] = quake_length;
    {
        let mut i: i32 = 0;
        {
            i = 0;
            'l82: while i < 16 {
                if game_camera.quakes[0 as usize][i as usize].kind == ((QuakeKind_None as i32) as u32) {
                    game_camera.quakes[0 as usize][i as usize].kind = kind;
                    if !pos.is_null() {
                        game_camera.quakes[0 as usize][i as usize].epicenter = *pos;
                    } else {
                        game_camera.quakes[0 as usize][i as usize].epicenter.z = 0.0f32;
                        game_camera.quakes[0 as usize][i as usize].epicenter.y = 0.0f32;
                        game_camera.quakes[0 as usize][i as usize].epicenter.x = 0.0f32;
                    }
                }
                i = i + 1;
            }
        }
    }
}

/// `Camera_StopQuake`
pub unsafe fn Camera_StopQuake(mut kind: CmQuakeKind) {
    game_camera.quake_frames_left[kind as usize] = 0;
}

/// `Camera_PCMainCObj`
pub unsafe fn Camera_PCMainCObj() -> *mut HSD_CObj {
    return if !game_camera.gobj.is_null() { HSD_GObjGetHSDObj(game_camera.gobj) as *mut HSD_CObj } else { null_mut() };
}

/// `Camera_80031060`
pub unsafe fn Camera_80031060() -> i32 {
    return game_camera.x398_b6_b7() as i32;
}

/// `Camera_80031074`
pub unsafe fn Camera_80031074(mut arg0: u8) {
    game_camera.set_x398_b6_b7((arg0) as u8);
}

/// `Camera_8003108C`
pub unsafe fn Camera_8003108C() -> i32 {
    return game_camera.x399_b0_b1() as i32;
}

/// `Camera_800310A0`
pub unsafe fn Camera_800310A0(mut arg0: u8) {
    game_camera.set_x399_b0_b1((arg0) as u8);
}

/// `Camera_800310B8`
pub unsafe fn Camera_800310B8() -> *mut HSD_CObj {
    ext::HSD_CObjSetMtxDirty(cm_804D6464);
    ext::HSD_CObjSetupViewingMtx(cm_804D6464);
    return cm_804D6464;
}

/// `Camera_800310E8`
pub unsafe fn Camera_800310E8() {
    game_camera.set_x398_b0((0 as u8) as u8);
    game_camera.set_x398_b1((0 as u8) as u8);
    game_camera.set_x398_b2((0 as u8) as u8);
    game_camera.set_x398_b3((0 as u8) as u8);
    game_camera.set_x398_b4((0 as u8) as u8);
    game_camera.set_x398_b5((0 as u8) as u8);
}

/// `Camera_80031144`
pub unsafe fn Camera_80031144() -> f32 {
    return game_camera.x2BC;
}

/// `Camera_80031154`
pub unsafe fn Camera_80031154(mut arg0: *mut Vec) -> i32 {
    if Camera_80029124(arg0, 0) == (0 as u32) {
        return 1;
    }
    return 0;
}

/// `Camera_8003118C`
pub unsafe fn Camera_8003118C(mut arg0: *mut Vec, mut arg1: f32) -> i32 {
    if Camera_80029124(arg0, arg1 as i32) == (0 as u32) {
        return 1;
    }
    return 0;
}

/// `Camera_800311CC`
pub unsafe fn Camera_800311CC(mut arg8: f32) {
    game_camera.farz = arg8;
}

/// `Camera_StageSlotClipGet`
pub unsafe fn Camera_StageSlotClipGet(mut nearz: *mut f32, mut farz: *mut f32) {
    *nearz = game_camera.nearz;
    *farz = game_camera.farz;
}

/// `Camera_800311DC`
pub unsafe fn Camera_800311DC(mut arg8: f32) {
    game_camera.nearz = arg8;
}

/// `inline_cam_gx_b0`
pub unsafe fn inline_cam_gx_b0() -> i64 {
    if game_camera.x398_b0() != 0 {
        return 0 as i64;
    }
    return 32 as i64;
}

/// `inline_cam_gx_b1`
pub unsafe fn inline_cam_gx_b1() -> i64 {
    if game_camera.x398_b1() != 0 {
        return 0 as i64;
    }
    return 64 as i64;
}

/// `inline_cam_gx_b4`
pub unsafe fn inline_cam_gx_b4() -> i64 {
    if game_camera.x398_b4() != 0 {
        return 0 as i64;
    }
    return 16 as i64;
}

/// `Camera_800311EC`
pub unsafe fn Camera_800311EC(mut gobj: *mut HSD_GObj, mut prios: u64) {
    let mut a: i64 = 0;
    let mut b: i64 = 0;
    game_camera.set_x398_b6_b7((1 as u8) as u8);
    a = inline_cam_gx_b1();
    b = inline_cam_gx_b0();
    b = (prios | (b as u64)) as i64;
    (*gobj).gxlink_prios = (b | a) as u64;
    ext::HSD_GObj_80390ED0(gobj, 3 as u32);
    game_camera.set_x398_b6_b7((0 as u8) as u8);
    a = inline_cam_gx_b0();
    b = inline_cam_gx_b4();
    prios = inline_cam_gx_b1() as u64;
    (*gobj).gxlink_prios = prios | ((b | a) as u64);
    ext::HSD_GObj_80390ED0(gobj, 3 as u32);
}

/// `Camera_80031328`
pub unsafe fn Camera_80031328(mut gobj: *mut HSD_GObj, mut prios: u64) {
    let mut a: i64 = 0;
    let mut b: i64 = 0;
    let mut c: u64 = 0;
    game_camera.set_x398_b6_b7((0 as u8) as u8);
    a = inline_cam_gx_b0();
    b = inline_cam_gx_b4();
    c = inline_cam_gx_b1() as u64;
    b = (prios | (b as u64)) as i64;
    (*gobj).gxlink_prios = ((b | a) as u64) | c;
    ext::HSD_GObj_80390ED0(gobj, 4 as u32);
}

/// `Camera_800313E0`
pub unsafe fn Camera_800313E0(mut gobj: *mut HSD_GObj, mut prios: u64) {
    let mut a: i64 = 0;
    let mut b: i64 = 0;
    let mut c: u64 = 0;
    game_camera.set_x398_b6_b7((1 as u8) as u8);
    a = inline_cam_gx_b1();
    b = inline_cam_gx_b0();
    b = (prios | (b as u64)) as i64;
    (*gobj).gxlink_prios = (b | a) as u64;
    ext::HSD_GObj_80390ED0(gobj, 3 as u32);
    game_camera.set_x398_b6_b7((0 as u8) as u8);
    a = inline_cam_gx_b0();
    b = inline_cam_gx_b4();
    c = inline_cam_gx_b1() as u64;
    (*gobj).gxlink_prios = c | ((b | a) as u64);
    ext::HSD_GObj_80390ED0(gobj, 3 as u32);
    game_camera.set_x398_b6_b7((0 as u8) as u8);
    a = inline_cam_gx_b0();
    b = inline_cam_gx_b4();
    c = inline_cam_gx_b1() as u64;
    b = (prios | (b as u64)) as i64;
    (*gobj).gxlink_prios = ((b | a) as u64) | c;
    ext::HSD_GObj_80390ED0(gobj, 4 as u32);
}

/// `HSD_GObjGetHSDObj`
pub unsafe fn HSD_GObjGetHSDObj(mut gobj: *mut HSD_GObj) -> *mut c_void {
    return (*gobj).hsd_obj;
}

/// `lbVector_Len`
pub unsafe fn lbVector_Len(mut vec: *mut Vec) -> f32 {
    return melee_runtime::gekko::sqrtf(fmadds((*vec).z, (*vec).z, fmadds((*vec).x, (*vec).x, (*vec).y * (*vec).y)));
}
