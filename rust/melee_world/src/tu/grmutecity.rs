//! Port of src/melee/gr/grmutecity.c (Mute City: the stage callbacks, the animation command table that
//! drives the track, the car field and the lights). Every function of the TU, native: pointers into the
//! game's structs, callees of other TUs through `ext` hooks. Checked against the C (whole TU, shipping
//! contraction, stateful oracle) by tests/tu_grmutecity.rs; regenerate the C side with
//! `python rust/melee_world/gen/cref/run_gr.py grmutecity`.
//!
//! `grMuteCity_801F1328` reads `arr[-1]` (the word in front of `grMc_8049F440`, which belongs to another
//! TU's .bss in the retail layout): modelled as the hidden word `grMc_8049F43C`, the oracle patches the C
//! the same way (spec `patch`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
use melee_core::fp::Fmadd as _;
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf, sqrtf};
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
    fn Camera_GetTransformPosition(a0: *mut Vec) = |a0: *mut Vec| { unsafe { crate::tu::camera::Camera_GetTransformPosition(a0); } };
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec) = |a0: u32, a1: *mut Vec| unsafe { crate::tu::camera::Camera_RequestQuake(a0 as _, a1 as _) };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Ground_801C0498() -> f32 = || unsafe { crate::tu::ground::Ground_801C0498() as _ };
    fn Ground_801C13D0(a0: i32, a1: i32) -> *mut HSD_JObj = |a0: i32, a1: i32| unsafe { crate::tu::ground::Ground_801C13D0(a0 as _, a1 as _) as _ };
    fn Ground_801C3880(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C3880(a0 as _); } };
    fn Ground_801C3890(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C3890(a0 as _); } };
    fn Ground_801C38A0(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C38A0(a0 as _); } };
    fn Ground_801C38AC(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C38AC(a0 as _); } };
    fn Ground_801C3980(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C3980(a0 as _); } };
    fn Ground_801C3990(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C3990(a0 as _); } };
    fn Ground_801C39A0(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C39A0(a0 as _); } };
    fn Ground_801C39B0(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C39B0(a0 as _); } };
    fn Ground_801C39C0() = || { unsafe { crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3F20(a0: *mut HSD_JObj) -> f32 = |a0: *mut HSD_JObj| unsafe { crate::tu::ground::Ground_801C3F20(a0 as _) as _ };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C498C() -> *mut HSD_GObj = || unsafe { crate::tu::ground::Ground_801C498C() as _ };
    fn Ground_801C53EC(a0: u32) = |a0: u32| { unsafe { crate::tu::ground::Ground_801C53EC(a0 as _); } };
    fn Ground_801C5440(a0: *mut Ground, a1: i32, a2: u32) = |a0: *mut Ground, a1: i32, a2: u32| { unsafe { crate::tu::ground::Ground_801C5440(a0 as _, a1 as _, a2 as _); } };
    fn Ground_801C5630(a0: *mut Ground, a1: i32, a2: f32) = |a0: *mut Ground, a1: i32, a2: f32| { unsafe { crate::tu::ground::Ground_801C5630(a0 as _, a1 as _, a2 as _); } };
    fn Ground_801C5AEC(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec) = |a0: *mut Vec, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec| { unsafe { crate::tu::ground::Ground_801C5AEC(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetMapGObj(a0 as _) as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjAddChild(a0: *mut HSD_JObj, a1: *mut HSD_JObj) = |a0: *mut HSD_JObj, a1: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjAddChild(a0 as _, a1 as _); } };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_JObjUnref(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj::HSD_JObjUnref(a0 as _); } };
    fn HSD_LObjClearFlags(a0: *mut HSD_LObj, a1: u32) = |a0: *mut HSD_LObj, a1: u32| { unsafe { melee_runtime::tu::lobj::HSD_LObjClearFlags(a0 as _, a1 as _); } };
    fn HSD_LObjGetColor(a0: *mut HSD_LObj, a1: *mut _GXColor) = |a0: *mut HSD_LObj, a1: *mut _GXColor| { unsafe { melee_runtime::tu::lobj::HSD_LObjGetColor(a0 as _, a1 as _); } };
    fn HSD_LObjGetFlags(a0: *mut HSD_LObj) -> u32 = |a0: *mut HSD_LObj| unsafe { melee_runtime::tu::lobj::HSD_LObjGetFlags(a0 as _) as _ };
    fn HSD_LObjSetColor(a0: *mut HSD_LObj, a1: _GXColor) = |a0: *mut HSD_LObj, a1: _GXColor| { unsafe { melee_runtime::tu::lobj::HSD_LObjSetColor(a0 as _, a1 as _); } };
    fn HSD_LObjSetDistAttn(a0: *mut HSD_LObj, a1: f32, a2: f32, a3: i32) = |a0: *mut HSD_LObj, a1: f32, a2: f32, a3: i32| { unsafe { melee_runtime::tu::lobj::HSD_LObjSetDistAttn(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_LObjSetFlags(a0: *mut HSD_LObj, a1: u32) = |a0: *mut HSD_LObj, a1: u32| { unsafe { melee_runtime::tu::lobj::HSD_LObjSetFlags(a0 as _, a1 as _); } };
    fn HSD_Randf() -> f32 = || melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand());
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn PSVECCrossProduct(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) = |a0: *mut Vec, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::mtx_pc::raw::PSVECCrossProduct(a0 as _, a1 as _, a2 as _); } };
    fn Stage_GetBlastZoneBottomOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneBottomOffset() as _ };
    fn Stage_GetBlastZoneLeftOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneLeftOffset() as _ };
    fn Stage_GetBlastZoneRightOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneRightOffset() as _ };
    fn Stage_GetBlastZoneTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneTopOffset() as _ };
    fn Stage_GetCamBoundsBottomOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsBottomOffset() as _ };
    fn Stage_GetCamBoundsLeftOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsLeftOffset() as _ };
    fn Stage_GetCamBoundsRightOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsRightOffset() as _ };
    fn Stage_GetCamBoundsTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsTopOffset() as _ };
    fn Stage_UnkSetVec3TCam_Offset(a0: *mut Vec) = |a0: *mut Vec| { unsafe { crate::tu::stage::Stage_UnkSetVec3TCam_Offset(a0 as _); } };
    fn atan2f(a0: f32, a1: f32) -> f32 = |a0: f32, a1: f32| unsafe { melee_runtime::lbtrigf::atan2f(a0 as _, a1 as _) as _ };
    fn grAnime_801C775C(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32, a4: f32) = |a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32, a4: f32| { unsafe { crate::tu::granime::grAnime_801C775C(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } };
    fn grAnime_801C7A04(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32) = |a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32| { unsafe { crate::tu::granime::grAnime_801C7A04(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) = |a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32| { unsafe { crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grDatFiles_801C6330(a0: i32) -> *mut UnkArchiveStruct = |a0: i32| unsafe { crate::tu::grdatfiles::grDatFiles_801C6330(a0 as _) as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: isize) = |a0: *mut HSD_GObj, a1: isize| { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grFZeroCar_801CAFBC(a0: *mut HSD_GObj, a1: *mut c_void, a2: i32, a3: i32) = |a0: *mut HSD_GObj, a1: *mut c_void, a2: i32, a3: i32| { unsafe { crate::tu::grfzerocar::grFZeroCar_801CAFBC(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn grLib_801C96E8(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::grlib::grLib_801C96E8(a0 as _) as _ };
    fn grLib_801C96F8(a0: i32, a1: i32, a2: *mut Vec) -> *mut HSD_Generator = |a0: i32, a1: i32, a2: *mut Vec| unsafe { crate::tu::grlib::grLib_801C96F8(a0 as _, a1 as _, a2 as _) as _ };
    fn grLib_801C9808(a0: i32, a1: i32, a2: *mut HSD_JObj) -> *mut HSD_Generator = |a0: i32, a1: i32, a2: *mut HSD_JObj| unsafe { crate::tu::grlib::grLib_801C9808(a0 as _, a1 as _, a2 as _) as _ };
    fn grLib_801C98A0(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { crate::tu::grlib::grLib_801C98A0(a0 as _); } };
    fn grMaterial_801C8CDC(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grmaterial::grMaterial_801C8CDC(a0 as _); } };
    fn grMaterial_801C8CFC(a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, a4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj = |a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, a4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>| unsafe { crate::tu::grmaterial::grMaterial_801C8CFC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ };
    fn grMaterial_801C8DE0(a0: *mut HSD_GObj, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32) = |a0: *mut HSD_GObj, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32| { unsafe { crate::tu::grmaterial::grMaterial_801C8DE0(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _); } };
    fn grMaterial_801C8E08(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grmaterial::grMaterial_801C8E08(a0 as _); } };
    fn grMaterial_801C94D8(a0: *mut c_void) = |a0: *mut c_void| { unsafe { crate::tu::grmaterial::grMaterial_801C94D8(a0 as _); } };
    fn grMaterial_801C95C4(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grmaterial::grMaterial_801C95C4(a0 as _); } };
    fn grMaterial_801C9604(a0: *mut HSD_GObj, a1: *mut c_void, a2: i32) = |a0: *mut HSD_GObj, a1: *mut c_void, a2: i32| unsafe { crate::tu::grmaterial::grMaterial_801C9604(a0 as _, a1 as _, a2 != 0) };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn lbAudioAx_80024D78(a0: i32) = |a0: i32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024D78(a0 as _); } };
    fn lbShadow_8000E9F0(a0: *mut Vec, a1: *mut HSD_Spline, a2: f32) = |a0: *mut Vec, a1: *mut HSD_Spline, a2: f32| { unsafe { melee_runtime::tu::lbshadow::lbShadow_8000E9F0(&mut *a0, &*a1, a2); } };
    fn lbVector_Add(a0: *mut Vec, a1: *mut Vec) -> *mut Vec = |a0: *mut Vec, a1: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_Add(a0 as _, a1 as _) as _ };
    fn lbVector_Diff(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec = |a0: *mut Vec, a1: *mut Vec, a2: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_Diff(a0 as _, a1 as _, a2 as _) as _ };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 = |a0: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0 as _) as _ };
    fn lbVector_Sub(a0: *mut Vec, a1: *mut Vec) -> *mut Vec = |a0: *mut Vec, a1: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_Sub(a0 as _, a1 as _) as _ };
    fn lb_8000B134(a0: *mut HSD_JObj) -> i32 = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::lb_00B0::lb_8000B134(a0 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn mpJointListAdd(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpJointListAdd(a0 as _); } };
    fn mpJointSetB10(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpJointSetB10(a0 as _); } };
    fn mpJointSetCb1(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) = |a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>| { unsafe { crate::tu::mplib::mpJointSetCb1(a0 as _, a1 as _, a2 as _); } };
    fn mpLib_80054D68(a0: i32, a1: u32) = |a0: i32, a1: u32| { unsafe { crate::tu::mplib::mpLib_80054D68(a0 as _, a1 as _); } };
    fn mpLib_80055E24(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80055E24(a0 as _); } };
    fn mpLib_8005667C(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_8005667C(a0 as _); } };
    fn mpLib_80057424(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057424(a0 as _); } };
    fn mpLib_80057BC0(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057BC0(a0 as _); } };
    fn mpLineSetPos(a0: i32, a1: f32, a2: f32, a3: f32, a4: f32) = |a0: i32, a1: f32, a2: f32, a3: f32, a4: f32| { unsafe { crate::tu::mplib::mpLineSetPos(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } };
    fn psAddGeneratorAppSRT_begin(a0: *mut HSD_Generator, a1: i32) -> *mut HSD_psAppSRT;
    fn splGetSplinePoint(a0: *mut Vec, a1: *mut HSD_Spline, a2: f32) = |a0: *mut Vec, a1: *mut HSD_Spline, a2: f32| { unsafe { melee_runtime::tu::spline_world::splGetSplinePoint(a0 as _, a1 as _, a2 as _); } };
    fn un_802FD604(a0: i32) = |a0: i32| { unsafe { crate::tu::ifhazard::un_802FD604(a0 as _); } };
    fn un_802FD65C() = || { unsafe { crate::tu::ifhazard::un_802FD65C(); } };
    fn un_802FD8C4(a0: *mut DynamicModelDesc) = |a0: *mut DynamicModelDesc| { unsafe { crate::tu::ifhazard::un_802FD8C4(a0 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: isize) { super::grDisplay_801C5DB0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
    pub use crate::tu::ground::stage_info;
}
// END GENERATED EXT

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, _j: &mut HSD_JObj) { unreachable!("HSD_JObjSetMtxDirty (out of line) is not called by this TU") }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
}

/// gr/inlines.h `Ground_StartMapAnim`.
unsafe fn Ground_StartMapAnim(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// `const Vec3 grMc_803B81B8`
pub static grMc_803B81B8: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// The word in front of `grMc_8049F440` (see the module comment) and the array itself.
pub static mut grMc_8049F43C: [i32; 31] = [0; 31];

#[inline(always)]
pub unsafe fn grMc_8049F440() -> *mut i32 {
    melee_core::ct!("grMc_8049F440");
    (&raw mut grMc_8049F43C as *mut i32).add(1)
}

/// `grMc_CarEntry`'s `x22_flags` (b0, b1: GC bit order, MSB first)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GrMcCarFlags {
    pub bf_0: u8,
}
impl GrMcCarFlags {
    #[inline] pub fn b0(&self) -> u8 { (self.bf_0 >> 7) & 1 }
    #[inline] pub fn set_b0(&mut self, v: u8) { self.bf_0 = (self.bf_0 & !0x80) | ((v & 1) << 7) }
    #[inline] pub fn b1(&self) -> u8 { (self.bf_0 >> 6) & 1 }
    #[inline] pub fn set_b1(&mut self, v: u8) { self.bf_0 = (self.bf_0 & !0x40) | ((v & 1) << 6) }
}

/// `grMc_CarEntry`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GrMcCarEntry {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub pos: Vec3,
    pub x20: u16,
    pub x22_flags: GrMcCarFlags,
    pub x23: u8,
    pub x24: *mut HSD_GObj,
    pub r#gen: *mut HSD_Generator,
}
melee_core::gc_layout!(GrMcCarEntry, "grMc_CarEntry", size 0x2C, align 4, [
    x0 @ 0x0: 4 "f32", x4 @ 0x4: 4 "f32", x8 @ 0x8: 4 "f32", xC @ 0xC: 4 "f32", x10 @ 0x10: 4 "f32",
    pos @ 0x14: 12 "Vec3", x20 @ 0x20: 2 "u16", x22_flags @ 0x22: 1 "struct", x23 @ 0x23: 1 "u8",
    x24 @ 0x24: 4 "Item_GObj *", r#gen @ 0x28: 4 "HSD_Generator *",
]);

pub static mut grMc_8049F4B8: [GrMcCarEntry; 30] = unsafe { core::mem::zeroed() };

#[inline(always)]
unsafe fn ce(i: usize) -> *mut GrMcCarEntry {
    (&raw mut grMc_8049F4B8 as *mut GrMcCarEntry).add(i)
}

/// `GrJoint grMc_803E30B0[]`
pub static mut grMc_803E30B0: [GrJoint; 3] = [
    GrJoint { x: 6, y: 29, z: 6 },
    GrJoint { x: 7, y: 29, z: 8 },
    GrJoint { x: 8, y: 29, z: 9 },
];

const fn cb(on_init: HSD_GObjEvent, callback1: HSD_GObjPredicate, gobj_proc: HSD_GObjEvent, callback3: Option<unsafe extern "C" fn(*mut Ground_GObj)>, flags: u32) -> StageCallbacks {
    StageCallbacks { on_init, callback1, gobj_proc, callback3, x10: StageCallbacks_x10 { flags: u32::to_be(flags) } }
}

/// `grMc_StageCallbacks`
pub static mut grMc_StageCallbacks: [StageCallbacks; 39] = [
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(Some(grMuteCity_801F044C), Some(grMuteCity_801F04A8), Some(grMuteCity_801F04B0), Some(grMuteCity_801F04B4), 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(Some(grMuteCity_801F01B4), Some(grMuteCity_801F0288), Some(grMuteCity_801F0290), Some(grMuteCity_801F040C), 0),
    cb(Some(grMuteCity_801EFDF8), Some(grMuteCity_801F0118), Some(grMuteCity_801F0120), Some(grMuteCity_801F01B0), 0xC0000000),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(Some(stageGObj36_37_OnInit), Some(grMuteCity_801F043C), Some(grMuteCity_801F0444), Some(grMuteCity_801F0448), 0),
    cb(Some(stageGObj36_37_OnInit), Some(grMuteCity_801F043C), Some(grMuteCity_801F0444), Some(grMuteCity_801F0448), 0),
    cb(None, None, None, None, 0),
];

/// `grMc_StageData`
pub static mut grMc_StageData: StageData = StageData {
    grkind: Gr_Kind_MuteCity as _,
    callbacks: unsafe { &raw mut grMc_StageCallbacks as *mut StageCallbacks },
    data1: b"/GrMc.dat\0".as_ptr() as *mut i8,
    on_init: Some(grMuteCity_801EFC6C),
    on_demo_init: Some(grMuteCity_801EFC68),
    on_load: Some(grMuteCity_801EFCDC),
    on_start: Some(grMuteCity_801EFCE0),
    callback4: Some(grMuteCity_801EFD04),
    on_touch_line: Some(grMuteCity_801F2BBC),
    on_check_shadow_render: Some(grMuteCity_801F2C10),
    flags2: 1 << 0,
    joints: unsafe { &raw mut grMc_803E30B0 as *mut GrJoint },
    joint_count: 3,
};

/// `struct grMc_YakumonoParam`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GrMcYakumonoParam {
    pub x0: *mut c_void,
    pub x4: *mut c_void,
    pub x8: *mut DynamicsDesc,
    pub xC: *mut DynamicsDesc,
    pub pad10: [u8; 0x1C],
    pub x2C: f32,
    pub x30: f32,
    pub x34: f32,
    pub x38: f32,
    pub x3C: f32,
    pub x40: f32,
    pub x44: f32,
    pub x48: f32,
    pub x4C: f32,
}
melee_core::gc_layout!(GrMcYakumonoParam, "grMc_YakumonoParam", size 0x50, align 4, [
    x0 @ 0x0: 4 "void *", x4 @ 0x4: 4 "void *", x8 @ 0x8: 4 "DynamicsDesc *", xC @ 0xC: 4 "DynamicsDesc *",
    pad10 @ 0x10: 28 "u8[28]", x2C @ 0x2C: 4 "f32", x30 @ 0x30: 4 "f32", x34 @ 0x34: 4 "f32", x38 @ 0x38: 4 "f32",
    x3C @ 0x3C: 4 "f32", x40 @ 0x40: 4 "f32", x44 @ 0x44: 4 "f32", x48 @ 0x48: 4 "f32", x4C @ 0x4C: 4 "f32",
]);

/// `static struct grMc_YakumonoParam* yakumono_param`
pub static mut yakumono_param: *mut GrMcYakumonoParam = null_mut();

/// `static s32 grMc_804D69D4`
pub static mut grMc_804D69D4: i32 = 0;

pub unsafe extern "C" fn grMuteCity_801EFC68(_arg: i32) { melee_core::ct!("grMuteCity_801EFC68");}

pub unsafe extern "C" fn grMuteCity_801EFC6C() {
    melee_core::ct!("grMuteCity_801EFC6C");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut GrMcYakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(0);
    grMuteCity_801EFD0C(0);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    grMuteCity_801EFD0C(0x1E);
    ext::un_802FD8C4(grMuteCity_801F28A8());
    ext::un_802FD65C();
}

pub unsafe extern "C" fn grMuteCity_801EFCDC() { melee_core::ct!("grMuteCity_801EFCDC");}

pub unsafe extern "C" fn grMuteCity_801EFCE0() {
    melee_core::ct!("grMuteCity_801EFCE0");
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe extern "C" fn grMuteCity_801EFD04() -> i32 {
    melee_core::ct!("grMuteCity_801EFD04");
    0
}

pub unsafe extern "C" fn grMuteCity_801EFD0C(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grMuteCity_801EFD0C");
    let callbacks = (&raw mut grMc_StageCallbacks as *mut StageCallbacks).add(gobj_id as usize);
    let gobj = ext::Ground_GetStageGObj(gobj_id);

    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        (*gp).x8_callback = None;
        (*gp).xC_callback = None;
        ext::GObj_SetupGXLink(gobj, Some(ext::cb::grDisplay_801C5DB0), 3, 0);
        if (*callbacks).callback3.is_some() {
            (*gp).x1C_callback = (*callbacks).callback3;
        }
        if let Some(f) = (*callbacks).on_init {
            f(gobj);
        }
        if (*callbacks).gobj_proc.is_some() {
            ext::HSD_GObj_SetupProc(gobj, (*callbacks).gobj_proc, 4);
        }
    }

    gobj
}

pub unsafe extern "C" fn grMuteCity_801EFDF8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grMuteCity_801EFDF8");
    let mut sp1c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = (*gobj).user_data as *mut Ground;

    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C775C(gobj, 0, 7, 0.0, 3600.0);
    (*gp).u.mutecity.xC4 = 0;
    (*gp).u.mutecity.xC6 = 0;
    (*gp).u.mutecity.xD2 = 0;
    (*gp).u.mutecity.xD8 = 0.932775f32;
    (*gp).u.mutecity.xD4 = 0.932775f32;
    (*gp).u.mutecity.xD0_flags.set_b0(0);
    (*gp).u.mutecity.xDC = ext::Ground_801C3FA4(gobj, 5);
    (*gp).u.mutecity.xE0 = ext::Ground_801C3FA4(gobj, 9);

    if (*(*gp).u.mutecity.xE0).u.spline.is_null() {
        panic!("HSD_ASSERT(351, 0) grmutecity.c");
    }

    if (*(*gp).u.mutecity.xDC).u.spline.is_null() {
        panic!("HSD_ASSERT(355, 0) grmutecity.c");
    }

    (*gp).u.mutecity.xD0_flags.set_b23(2);
    (*gp).u.mutecity.xE4.z = 0.0;
    (*gp).u.mutecity.xE4.y = 0.0;
    (*gp).u.mutecity.xE4.x = 0.0;
    (*gp).u.mutecity.xF0.z = 0.0;
    (*gp).u.mutecity.xF0.y = 0.0;
    (*gp).u.mutecity.xF0.x = 0.0;
    (*gp).u.mutecity.xFC = ext::Ground_801C3FA4(gobj, 0xC);
    (*gp).u.mutecity.x100 = ext::Ground_801C3FA4(gobj, 0xD);
    (*gp).u.mutecity.x104 = ext::Ground_801C3FA4(gobj, 0xE);
    (*gp).u.mutecity.x108 = ext::Ground_801C3FA4(gobj, 0xF);
    (*gp).u.mutecity.x10C = ext::Ground_801C3FA4(gobj, 0x10);
    (*gp).x10_flags.set_b5(1);
    ext::Stage_UnkSetVec3TCam_Offset(&mut sp1c);
    (*gp).u.mutecity.x114 = ext::Stage_GetCamBoundsTopOffset() - sp1c.y;
    (*gp).u.mutecity.x118 = ext::Stage_GetCamBoundsBottomOffset() - sp1c.y;
    (*gp).u.mutecity.x11C = ext::Stage_GetCamBoundsLeftOffset() - sp1c.x;
    (*gp).u.mutecity.x120 = ext::Stage_GetCamBoundsRightOffset() - sp1c.x;
    (*gp).u.mutecity.x124 = ext::Stage_GetBlastZoneTopOffset() - sp1c.y;
    (*gp).u.mutecity.x128 = ext::Stage_GetBlastZoneBottomOffset() - sp1c.y;
    (*gp).u.mutecity.x12C = ext::Stage_GetBlastZoneLeftOffset() - sp1c.x;
    (*gp).u.mutecity.x130 = ext::Stage_GetBlastZoneRightOffset() - sp1c.x;
    (*gp).u.mutecity.xC8 = grMuteCity_801EFD0C(0x1D);
    (*gp).u.mutecity.xCC = grMuteCity_801EFD0C(2);
    grMuteCity_801F1A34((*gp).u.mutecity.xCC, gobj);
    ext::grMaterial_801C94D8(ext::Ground_801C3FA4(gobj, 0xC) as *mut c_void);
    ext::grMaterial_801C94D8(ext::Ground_801C3FA4(gobj, 0xC) as *mut c_void);
    ext::grMaterial_801C94D8(ext::Ground_801C3FA4(gobj, 0xD) as *mut c_void);
    ext::grMaterial_801C94D8(ext::Ground_801C3FA4(gobj, 0xE) as *mut c_void);
    ext::grMaterial_801C94D8(ext::Ground_801C3FA4(gobj, 0xF) as *mut c_void);
    ext::grMaterial_801C94D8(ext::Ground_801C3FA4(gobj, 0x10) as *mut c_void);
    ext::mpJointSetB10(4);
    ext::mpJointSetCb1(4, gp as *mut c_void, Some(fn_801F2B58));
    let lgobj = ext::Ground_801C498C();
    (*gp).u.mutecity.x110 = null_mut();
    if !lgobj.is_null() {
        let mut lobj = (*lgobj).hsd_obj as *mut HSD_LObj;
        if !lobj.is_null() {
            while !lobj.is_null() {
                if ((*lobj).flags as u32 & 3) == LOBJ_POINT as u32 {
                    (*gp).u.mutecity.x110 = lobj;
                    ext::HSD_LObjSetFlags((*gp).u.mutecity.x110, LOBJ_HIDDEN as u32);
                }
                lobj = HSD_LObjGetNext(if lobj.is_null() { None } else { Some(&*lobj) });
            }
        }
    }
}

static light_ref_br: f32 = 40000.0;
static light_ref_dist: f32 = 0.99;
static grMc_804D46CC: i32 = 0x1;

pub unsafe extern "C" fn grMuteCity_801F0118(_arg: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grMuteCity_801F0118");
    0
}

pub unsafe extern "C" fn grMuteCity_801F0120(gobj: *mut HSD_GObj) {
    melee_core::ct!("grMuteCity_801F0120");
    let ground = (*gobj).user_data as *mut Ground;
    if !(*ground).u.mutecity.x110.is_null() {
        ext::HSD_LObjSetDistAttn((*ground).u.mutecity.x110, light_ref_br, light_ref_dist, grMc_804D46CC);
    }
    grMuteCity_801F04B8(gobj);
    grMuteCity_801F0948(gobj);
    if (*ground).u.mutecity.xD0_flags.b0() != 0 {
        grMuteCity_801F0D20(gobj);
    }
    grMuteCity_801F1328();
    grMuteCity_801F1A34((*ground).u.mutecity.xCC, gobj);
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
}

pub unsafe extern "C" fn grMuteCity_801F01B0(_arg: *mut HSD_GObj) { melee_core::ct!("grMuteCity_801F01B0");}

pub unsafe extern "C" fn grMuteCity_801F01B4(gobj: *mut HSD_GObj) {
    melee_core::ct!("grMuteCity_801F01B4");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grMaterial_801C94D8(jobj as *mut c_void);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C775C(gobj, 0, 7, 0.0, 3600.0);
    ext::grAnime_801C7FF8(gobj, 6, 7, 1, 0.0, 1.0);
    (*gp).x10_flags.set_b5(1);
    (*gp).u.mutecity2.xC8 = ext::Ground_801C3FA4(gobj, 6);
    (*gp).u.mutecity2.xCC = 0.0;
    (*gp).u.mutecity2.xD0 = 0.0;
    (*gp).u.mutecity2.xC4_flags.set_b0(0);
}

pub unsafe extern "C" fn grMuteCity_801F0288(_arg: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grMuteCity_801F0288");
    0
}

#[inline(always)]
pub unsafe fn grMuteCity_801F0290_get_target(gp: *mut Ground) -> f32 {
    melee_core::ct!("grMuteCity_801F0290_get_target");
    (*gp).u.mutecity2.xCC
}

pub unsafe extern "C" fn grMuteCity_801F0290(gobj: *mut HSD_GObj) {
    melee_core::ct!("grMuteCity_801F0290");
    let gp = (*gobj).user_data as *mut Ground;
    let target = grMuteCity_801F0290_get_target(gp);
    let current = (*gp).u.mutecity2.xD0;
    let diff = target - current;

    if diff > 0.0 {
        if diff < 0.001 {
            (*gp).u.mutecity2.xD0 = target;
        } else {
            (*gp).u.mutecity2.xD0 += 0.001;
        }
    } else if diff < 0.0 {
        if diff > -0.001 {
            (*gp).u.mutecity2.xD0 = target;
        } else {
            (*gp).u.mutecity2.xD0 = current - 0.001;
        }
    } else {
        (*gp).u.mutecity2.xD0 = target;
    }

    {
        let rot_z = HSD_JObjGetRotationZ(&*(*gp).u.mutecity2.xC8);
        HSD_JObjSetRotationZ(&mut *(*gp).u.mutecity2.xC8, rot_z * (*gp).u.mutecity2.xD0, &mut Dirty);
    }
    grMuteCity_801F290C(gobj);
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grMuteCity_801F040C(_arg: *mut HSD_GObj) { melee_core::ct!("grMuteCity_801F040C");}

pub unsafe extern "C" fn stageGObj36_37_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj36_37_OnInit");
    Ground_StartMapAnim(gobj);
}

pub unsafe extern "C" fn grMuteCity_801F043C(_arg: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grMuteCity_801F043C");
    0
}

pub unsafe extern "C" fn grMuteCity_801F0444(_arg: *mut HSD_GObj) { melee_core::ct!("grMuteCity_801F0444");}

pub unsafe extern "C" fn grMuteCity_801F0448(_arg: *mut HSD_GObj) { melee_core::ct!("grMuteCity_801F0448");}

/// `s16 grMc_803E34A4[30]`
pub static mut grMc_803E34A4: [i16; 30] = [
    2, 3, 4, 6, 7, 8, 9, 10, 11, 12,
    1, 13, 14, 15, 16, 17, 18, 19, 20, 21,
    22, 23, 24, 25, 26, 27, 32, 33, 34, 35,
];

pub unsafe extern "C" fn grMuteCity_801F044C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grMuteCity_801F044C");
    let gp = (*gobj).user_data as *mut Ground;
    ext::grFZeroCar_801CAFBC(gobj, (&raw mut grMc_803E34A4) as *mut c_void, 0x1C, 0);
    grMuteCity_801F0F4C(gobj);
    (*gp).u.mutecity.xC4 = 0;
}

pub unsafe extern "C" fn grMuteCity_801F04A8(_arg: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grMuteCity_801F04A8");
    0
}

pub unsafe extern "C" fn grMuteCity_801F04B0(_arg: *mut HSD_GObj) { melee_core::ct!("grMuteCity_801F04B0");}

pub unsafe extern "C" fn grMuteCity_801F04B4(_arg: *mut HSD_GObj) { melee_core::ct!("grMuteCity_801F04B4");}

/// `grMc_CmdEntry`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GrMcCmdEntry {
    pub frame: i32,
    pub cmd: i16,
    pub param: i16,
    pub fval: f32,
}
melee_core::gc_layout!(GrMcCmdEntry, "grMc_CmdEntry", size 0xC, align 4, [
    frame @ 0x0: 4 "s32", cmd @ 0x4: 2 "s16", param @ 0x6: 2 "s16", fval @ 0x8: 4 "f32",
]);

/// `grMc_803E34E0`
pub static mut grMc_803E34E0: [GrMcCmdEntry; 132] = [
    GrMcCmdEntry { frame: -1, cmd: 11, param: 2, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 11, param: 3, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 11, param: 6, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 11, param: 4, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 11, param: 7, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 4, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 4, param: 1, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 4, param: 6, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 4, param: 2, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 4, param: 5, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 4, param: 7, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 4, param: 8, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 10, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 9, param: 0, fval: 0.932775 }, GrMcCmdEntry { frame: -1, cmd: 18, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 7, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 17, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 13, param: 0, fval: 90.0 },
    GrMcCmdEntry { frame: 180, cmd: 17, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 384, cmd: 19, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 6, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 10, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 475, cmd: 2, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 475, cmd: 1, param: 340, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 20, param: 120, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 1, param: 60, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 3, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: 530, cmd: 21, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 567, cmd: 18, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 7, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: 708, cmd: 26, param: 1, fval: 0.0 }, GrMcCmdEntry { frame: 800, cmd: 19, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 6, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 10, param: 1, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 12, param: 2, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 5, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 5, param: 7, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 5, param: 8, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 15, param: 0, fval: -270.0 },
    GrMcCmdEntry { frame: -1, cmd: 16, param: 0, fval: 270.0 }, GrMcCmdEntry { frame: 890, cmd: 2, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 890, cmd: 1, param: 340, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 20, param: 120, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 1, param: 60, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 3, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: 950, cmd: 21, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 980, cmd: 18, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 4, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 4, param: 7, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 4, param: 8, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 11, param: 2, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 7, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 17, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 1000, cmd: 13, param: 0, fval: 40.0 },
    GrMcCmdEntry { frame: -1, cmd: 14, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 15, param: 0, fval: -50.0 }, GrMcCmdEntry { frame: -1, cmd: 16, param: 0, fval: 50.0 },
    GrMcCmdEntry { frame: 1080, cmd: 23, param: 1, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 24, param: 1, fval: 0.0 }, GrMcCmdEntry { frame: 708, cmd: 26, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: 1130, cmd: 19, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 23, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 24, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 6, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 17, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 14, param: 0, fval: -30.0 },
    GrMcCmdEntry { frame: -1, cmd: 10, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 12, param: 7, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 5, param: 5, fval: 0.0 },
    GrMcCmdEntry { frame: 1223, cmd: 2, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 1223, cmd: 1, param: 340, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 20, param: 120, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 1, param: 60, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 3, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 1280, cmd: 21, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: 1312, cmd: 18, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 26, param: 2, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 11, param: 7, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 4, param: 5, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 17, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 7, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: 1404, cmd: 19, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 6, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 10, param: 2, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 12, param: 3, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 5, param: 1, fval: 0.0 }, GrMcCmdEntry { frame: 1495, cmd: 2, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 25, param: 1, fval: 0.0 }, GrMcCmdEntry { frame: 1495, cmd: 1, param: 340, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 20, param: 120, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 1, param: 60, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 3, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 1550, cmd: 21, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: 1585, cmd: 18, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 7, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 25, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 11, param: 3, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 4, param: 1, fval: 0.0 }, GrMcCmdEntry { frame: 2045, cmd: 19, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 6, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 10, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 12, param: 6, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 5, param: 6, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 22, param: 0, fval: 1.0 }, GrMcCmdEntry { frame: 2135, cmd: 2, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: 2135, cmd: 1, param: 340, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 20, param: 120, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 1, param: 60, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 3, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 22, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 2190, cmd: 21, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: 2225, cmd: 18, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 7, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 11, param: 6, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 4, param: 6, fval: 0.0 }, GrMcCmdEntry { frame: 2273, cmd: 8, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 3290, cmd: 9, param: 0, fval: 0.86445 },
    GrMcCmdEntry { frame: 3300, cmd: 13, param: 0, fval: 90.0 }, GrMcCmdEntry { frame: 3390, cmd: 19, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 6, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 10, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 12, param: 4, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 5, param: 2, fval: 0.0 },
    GrMcCmdEntry { frame: 3510, cmd: 2, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 3510, cmd: 1, param: 340, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 20, param: 120, fval: 0.0 },
    GrMcCmdEntry { frame: -1, cmd: 1, param: 60, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 3, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 3570, cmd: 21, param: 0, fval: 0.0 },
    GrMcCmdEntry { frame: 3599, cmd: 18, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: -1, cmd: 7, param: 0, fval: 0.0 }, GrMcCmdEntry { frame: 9999, cmd: 0, param: 0, fval: 0.0 },
];

pub unsafe extern "C" fn grMuteCity_801F04B8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grMuteCity_801F04B8");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let gp = (*gobj).user_data as *mut Ground;

    let frame = ext::Ground_801C3F20(jobj);

    if (*gp).u.mutecity.xC6 > 0 {
        (*gp).u.mutecity.xC6 = (*gp).u.mutecity.xC6.wrapping_sub(1);
        return;
    }

    if ext::lb_8000B134(ext::Ground_801C3FA4(gobj, 0x1A)) != 0 {
        (*gp).u.mutecity.xC4 = 0;
        ext::HSD_JObjReqAnimAll((*(*gp).u.mutecity.xC8).hsd_obj as *mut HSD_JObj, 1.0);
    }

    let table = &raw mut grMc_803E34E0 as *mut GrMcCmdEntry;
    while frame >= (*table.add((*gp).u.mutecity.xC4 as usize)).frame as f32 {
        let entry = table.add((*gp).u.mutecity.xC4 as usize);
        let cmd = (*entry).cmd as i32;
        let param = (*entry).param as i32;
        let fval = (*entry).fval;

        match cmd {
            1 => {
                (*gp).u.mutecity.xC6 = (*entry).param;
            }
            2 => {
                ext::grAnime_801C7A04(gobj, 0, 1, 0.0);
                ext::grAnime_801C7A04((*gp).u.mutecity.xC8, 0, 7, 0.0);
                ext::grAnime_801C7A04(gobj, 0x11, 7, 1.0);
                ext::grAnime_801C7A04((*gp).u.mutecity.xC8, 6, 7, 1.0);
            }
            3 => {
                ext::grAnime_801C7A04(gobj, 0, 1, 1.0);
                ext::grAnime_801C7A04((*gp).u.mutecity.xC8, 0, 7, 1.0);
            }
            4 => {
                ext::mpLib_80057BC0((*entry).param as i32);
            }
            5 => {
                ext::mpJointListAdd((*entry).param as i32);
            }
            6 => {
                grMc_804D69D4 = 0;
            }
            7 => {
                grMc_804D69D4 = 1;
            }
            8 => {
                ext::mpLib_80057BC0(4);
                (*gp).u.mutecity.xD0_flags.set_b0(0);
            }
            18 => {
                (*gp).u.mutecity.xD0_flags.set_b1(1);
            }
            19 => {
                (*gp).u.mutecity.xD0_flags.set_b1(0);
            }
            9 => {
                ext::mpJointListAdd(4);
                (*gp).u.mutecity.xD0_flags.set_b0(1);
                (*gp).u.mutecity.xD4 = fval;
                (*gp).u.mutecity.xD8 = fval;
            }
            11 => {
                let j = ext::Ground_801C3FA4((*gp).u.mutecity.xC8, (*entry).param as i32);
                if !j.is_null() {
                    ext::HSD_JObjSetFlagsAll(j, JOBJ_HIDDEN as u32);
                }
            }
            12 => {
                let j = ext::Ground_801C3FA4((*gp).u.mutecity.xC8, (*entry).param as i32);
                if !j.is_null() {
                    ext::HSD_JObjClearFlagsAll(j, JOBJ_HIDDEN as u32);
                }
            }
            13 => {
                ext::Ground_801C3880(fval * ext::Ground_801C0498());
                if fval > (*gp).u.mutecity.x114 {
                    ext::Ground_801C3980(fval + ((*gp).u.mutecity.x124 - (*gp).u.mutecity.x114));
                }
            }
            14 => {
                ext::Ground_801C3890(fval * ext::Ground_801C0498());
                if fval < (*gp).u.mutecity.x118 {
                    ext::Ground_801C3990(fval + ((*gp).u.mutecity.x128 - (*gp).u.mutecity.x118));
                }
            }
            15 => {
                ext::Ground_801C38A0(fval * ext::Ground_801C0498());
                if fval < (*gp).u.mutecity.x11C {
                    ext::Ground_801C39A0(fval + ((*gp).u.mutecity.x12C - (*gp).u.mutecity.x11C));
                }
            }
            16 => {
                ext::Ground_801C38AC(fval * ext::Ground_801C0498());
                if fval > (*gp).u.mutecity.x120 {
                    ext::Ground_801C39B0(fval + ((*gp).u.mutecity.x130 - (*gp).u.mutecity.x120));
                }
            }
            17 => {
                ext::Ground_801C3880((*gp).u.mutecity.x114);
                ext::Ground_801C3890((*gp).u.mutecity.x118);
                ext::Ground_801C38A0((*gp).u.mutecity.x11C);
                ext::Ground_801C38AC((*gp).u.mutecity.x120);
                ext::Ground_801C3980((*gp).u.mutecity.x124);
                ext::Ground_801C3990((*gp).u.mutecity.x128);
                ext::Ground_801C39A0((*gp).u.mutecity.x12C);
                ext::Ground_801C39B0((*gp).u.mutecity.x130);
            }
            21 => {
                ext::grMaterial_801C9604(gobj, (*yakumono_param).x0, 0);
            }
            20 => {
                ext::un_802FD604((*entry).param as i32);
                ext::Ground_801C53EC(0x5CC6A);
            }
            22 => {
                let g2 = (*(*gp).u.mutecity.xC8).user_data as *mut Ground;
                (*g2).u.mutecity2.xCC = fval;
            }
            23 => {
                if param != 0 {
                    ext::lbAudioAx_80024D78(2);
                } else {
                    ext::lbAudioAx_80024D78(1);
                }
            }
            25 => {
                if param != 0 {
                    ext::mpLib_80054D68(0x32, 0xC);
                    ext::mpLib_80054D68(0x34, 0xC);
                } else {
                    ext::mpLib_80054D68(0x32, 0);
                    ext::mpLib_80054D68(0x34, 0);
                }
            }
            24 => {
                let bg_gobj = ext::Ground_GetMapGObj(0x1D);
                if !bg_gobj.is_null() {
                    if param != 0 {
                        ext::grMaterial_801C9604(bg_gobj, (*yakumono_param).x4, 0);
                        if !(*gp).u.mutecity.x110.is_null() {
                            ext::HSD_LObjClearFlags((*gp).u.mutecity.x110, LOBJ_HIDDEN as u32);
                        }
                    } else {
                        ext::grMaterial_801C95C4(bg_gobj);
                        if !(*gp).u.mutecity.x110.is_null() {
                            ext::HSD_LObjSetFlags((*gp).u.mutecity.x110, LOBJ_HIDDEN as u32);
                        }
                    }
                }
            }
            26 => {
                (*gp).u.mutecity.xD0_flags.set_b23(param as u8);
            }
            _ => {}
        }

        (*gp).u.mutecity.xC4 = (*gp).u.mutecity.xC4.wrapping_add(1);
        if ((*gp).u.mutecity.xC4 as i32 as u32) == 0x84 {
            (*gp).u.mutecity.xC4 = 0;
        }
        if cmd == 1 {
            break;
        }
    }
}

pub unsafe extern "C" fn grMuteCity_801F0948(_arg: *mut HSD_GObj) { melee_core::ct!("grMuteCity_801F0948");}

#[inline(always)]
pub unsafe fn grMc_SplineEvalWrapped(result: *mut Vec3, spline: *mut HSD_Spline, t: f32) {
    melee_core::ct!("grMc_SplineEvalWrapped");
    let mut a = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let frac: f32;

    if t > 0.93f32 {
        frac = t - 0.93f32;
    } else if t < 0.01f32 {
        frac = 0.06999999f32 + t;
    } else {
        ext::splGetSplinePoint(result, spline, t);
        return;
    }
    ext::splGetSplinePoint(&mut a, spline, 0.93f32);
    ext::splGetSplinePoint(&mut b, spline, 0.01f32);
    ext::lbVector_Sub(&mut b, &mut a);
    let scale = frac / 0.07999999f32;
    b.x *= scale;
    b.y *= scale;
    b.z *= scale;
    ext::lbVector_Add(&mut b, &mut a);
    *result = b;
}

#[inline(always)]
pub unsafe fn grMc_SplineTangentWrapped(result: *mut Vec3, spline: *mut HSD_Spline, t: f32) {
    melee_core::ct!("grMc_SplineTangentWrapped");
    let mut a = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let frac: f32;

    if t > 0.93f32 {
        frac = t - 0.93f32;
    } else if t < 0.01f32 {
        frac = 0.06999999f32 + t;
    } else {
        ext::lbShadow_8000E9F0(result, spline, t);
        return;
    }
    ext::lbShadow_8000E9F0(&mut a, spline, 0.93f32);
    ext::lbShadow_8000E9F0(&mut b, spline, 0.01f32);
    ext::lbVector_Sub(&mut b, &mut a);
    let scale = frac / 0.07999999f32;
    b.x *= scale;
    b.y *= scale;
    b.z *= scale;
    ext::lbVector_Add(&mut b, &mut a);
    *result = b;
}

pub unsafe extern "C" fn grMuteCity_801F094C(arg0: *mut HSD_JObj, arg1: *mut HSD_Spline, arg2: *mut Vec3, mut var_f29: f32) -> f32 {
    melee_core::ct!("grMuteCity_801F094C");
    let mut sp78 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp6c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp60 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    grMc_SplineEvalWrapped(&mut sp78, arg1, var_f29);
    ext::lb_8000B1CC(arg0, &mut sp78, &mut sp6c);

    let mut var_f1 = var_f29 - 0.0004f32;
    if (var_f1 as f64) < 0.0 {
        var_f1 += 1.0f32;
    }

    grMc_SplineEvalWrapped(&mut sp78, arg1, var_f1);
    ext::lb_8000B1CC(arg0, &mut sp78, &mut sp60);

    let var_f30: f32 = if sp6c.z > sp60.z { 0.0004f32 } else { -0.0004f32 };
    let var_f31 = -var_f30;

    while (sp60.z > 0.0 && sp6c.z > 0.0) || (sp60.z < 0.0 && sp6c.z < 0.0) {
        let var_f0 = if sp6c.z < 0.0 { var_f30 } else { var_f31 };
        var_f29 += var_f0;
        if var_f29 > 1.0 {
            var_f29 -= 1.0;
        } else if var_f29 < 0.0 {
            var_f29 += 1.0;
        }
        sp60 = sp6c;
        grMc_SplineEvalWrapped(&mut sp78, arg1, var_f29);
        ext::lb_8000B1CC(arg0, &mut sp78, &mut sp6c);
    }

    *arg2 = sp6c;
    var_f29
}

pub unsafe extern "C" fn grMuteCity_801F0D20(gobj: *mut HSD_GObj) {
    melee_core::ct!("grMuteCity_801F0D20");
    let mut sp28 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp1c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = (*gobj).user_data as *mut Ground;

    let jobj = (*gp).u.mutecity.xE0;
    (*gp).u.mutecity.xD8 = grMuteCity_801F094C(jobj, (*jobj).u.spline, &mut sp28, (*gp).u.mutecity.xD8);
    let jobj = (*gp).u.mutecity.xDC;
    (*gp).u.mutecity.xD4 = grMuteCity_801F094C(jobj, (*jobj).u.spline, &mut sp1c, (*gp).u.mutecity.xD4);
    ext::mpLib_80057424(4);
    let mode = (*gp).u.mutecity.xD0_flags.b23() as u32;

    if mode == 1 {
        ext::mpLineSetPos(0x31, sp28.x, 30.0f32 + sp28.y, sp28.x, sp28.y);
        ext::mpLineSetPos(0x35, sp1c.x, sp1c.y, sp1c.x, 30.0f32 + sp1c.y);
    } else if mode == 2 {
        let angle = ext::atan2f(sp1c.y - sp28.y, sp1c.x - sp28.x);
        let angle_back = (3.1415927f32 + angle) - 0.2617994f32;
        let angle_fwd = 0.2617994f32 + angle;
        let a2 = fmadds(10.0f32, cosf(angle_back), sp28.x);
        let a3 = fmadds(10.0f32, sinf(angle_back), sp28.y);
        ext::mpLineSetPos(0x31, a2, a3, sp28.x, sp28.y);
        let a3 = fmadds(10.0f32, cosf(angle_fwd), sp1c.x);
        let a4 = fmadds(10.0f32, sinf(angle_fwd), sp1c.y);
        ext::mpLineSetPos(0x35, sp1c.x, sp1c.y, a3, a4);
    } else {
        ext::mpLineSetPos(0x31, sp28.x - 5.0f32, sp28.y, sp28.x, sp28.y);
        ext::mpLineSetPos(0x35, sp1c.x, sp1c.y, 5.0f32 + sp1c.x, sp1c.y);
    }

    (*gp).u.mutecity.xE4 = sp28;
    (*gp).u.mutecity.xF0 = sp1c;
    ext::lbVector_Diff(&mut sp1c, &mut sp28, &mut sp10);
    ext::mpLineSetPos(
        0x33,
        fmadds(0.35f32, sp10.x, sp28.x),
        fmadds(0.35f32, sp10.y, sp28.y),
        fmadds(0.65f32, sp10.x, sp28.x),
        fmadds(0.65f32, sp10.y, sp28.y),
    );
    ext::mpLib_80055E24(4);
    ext::mpLib_8005667C(4);
}

/// `grMc_TrackInitData`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GrMcTrackInitData {
    pub pos: f32,
    pub speed: f32,
}
melee_core::gc_layout!(GrMcTrackInitData, "grMc_TrackInitData", size 0x8, align 4, [
    pos @ 0x0: 4 "f32", speed @ 0x4: 4 "f32",
]);

/// `grMc_803E3B7C`
pub static mut grMc_803E3B7C: [GrMcTrackInitData; 30] = [
    GrMcTrackInitData { pos: 0.9999, speed: 0.37 }, GrMcTrackInitData { pos: 0.9999, speed: 0.87 }, GrMcTrackInitData { pos: 0.9988, speed: 0.37 },
    GrMcTrackInitData { pos: 0.9988, speed: 0.87 }, GrMcTrackInitData { pos: 0.9977, speed: 0.37 }, GrMcTrackInitData { pos: 0.9977, speed: 0.87 },
    GrMcTrackInitData { pos: 0.9966, speed: 0.37 }, GrMcTrackInitData { pos: 0.9966, speed: 0.87 }, GrMcTrackInitData { pos: 0.9955, speed: 0.37 },
    GrMcTrackInitData { pos: 0.9955, speed: 0.87 }, GrMcTrackInitData { pos: 0.9944, speed: 0.37 }, GrMcTrackInitData { pos: 0.9944, speed: 0.87 },
    GrMcTrackInitData { pos: 0.9933, speed: 0.37 }, GrMcTrackInitData { pos: 0.9933, speed: 0.87 }, GrMcTrackInitData { pos: 0.9922, speed: 0.37 },
    GrMcTrackInitData { pos: 0.9922, speed: 0.87 }, GrMcTrackInitData { pos: 0.9911, speed: 0.37 }, GrMcTrackInitData { pos: 0.9911, speed: 0.87 },
    GrMcTrackInitData { pos: 0.9900, speed: 0.37 }, GrMcTrackInitData { pos: 0.9900, speed: 0.87 }, GrMcTrackInitData { pos: 0.9889, speed: 0.37 },
    GrMcTrackInitData { pos: 0.9889, speed: 0.87 }, GrMcTrackInitData { pos: 0.9878, speed: 0.37 }, GrMcTrackInitData { pos: 0.9878, speed: 0.87 },
    GrMcTrackInitData { pos: 0.9867, speed: 0.37 }, GrMcTrackInitData { pos: 0.9867, speed: 0.87 }, GrMcTrackInitData { pos: 0.9856, speed: 0.37 },
    GrMcTrackInitData { pos: 0.9856, speed: 0.87 }, GrMcTrackInitData { pos: 0.9845, speed: 0.37 }, GrMcTrackInitData { pos: 0.9845, speed: 0.87 },
];

pub unsafe extern "C" fn grMuteCity_801F0F4C(_gobj: *mut HSD_GObj) {
    melee_core::ct!("grMuteCity_801F0F4C");
    for i in 0..30usize {
        let mut pos = grMc_803E3B7C[i].pos;
        if (pos as f64) > 1.0 {
            pos -= 1.0;
        } else if (pos as f64) < 0.0 {
            pos += 1.0;
        }
        (*ce(i)).x0 = pos;
        (*ce(i)).x4 = pos;
        (*ce(i)).xC = grMc_803E3B7C[i].speed;
        (*ce(i)).x8 = 0.0;
        (*ce(i)).x10 = 0.0;
        (*ce(i)).x20 = 0;
        (*ce(i)).x22_flags.set_b0(0);
        (*ce(i)).x22_flags.set_b1(0);
        (*ce(i)).x24 = null_mut();
        (*ce(i)).r#gen = null_mut();
        *grMc_8049F440().add(i) = i as i32;
    }
}

pub unsafe extern "C" fn grMuteCity_801F106C(i: i32) {
    melee_core::ct!("grMuteCity_801F106C");
    let e = ce(i as usize);
    let flags16 = (*e).x20;
    let yp = yakumono_param;

    if (*e).x22_flags.b0() == 0 {
        if flags16 & 1 != 0 {
            if flags16 & 8 != 0 {
                (*e).x8 -= (*yp).x4C;
            } else {
                let rnd = ext::HSD_Randi(4);
                match rnd {
                    3 => {}
                    0 => {
                        (*e).x8 -= (*yp).x4C;
                    }
                    1 | 2 => {
                        if flags16 & 4 != 0 {
                            (*e).xC += (*yp).x44;
                            if ((*e).xC as f64) > 0.9 {
                                (*e).xC = 0.9f32;
                            }
                        } else {
                            (*e).xC -= (*yp).x44;
                            if ((*e).xC as f64) < 0.1 {
                                (*e).xC = 0.1f32;
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else {
            let params = yakumono_param;
            (*e).x8 += (*params).x40;
            if (*e).xC > (0.7f32 + (*params).x48) {
                (*e).xC -= (*params).x48;
            } else if (*e).xC < (0.3f32 - (*params).x48) {
                (*e).xC += (*params).x48;
            }
        }
        if flags16 & 8 != 0 {
            if flags16 & 4 != 0 {
                (*e).xC += (*yakumono_param).x44;
                if ((*e).xC as f64) > 1.0 {
                    (*e).xC = 1.0f32;
                }
            } else {
                (*e).xC -= (*yakumono_param).x44;
                if ((*e).xC as f64) < 0.0 {
                    (*e).xC = 0.0f32;
                }
            }
        }
        let max_x8 = if flags16 & 0x10 != 0 {
            (*yakumono_param).x38
        } else if flags16 & 0x20 != 0 {
            (*yakumono_param).x3C
        } else {
            (*yakumono_param).x34
        };
        if (*e).x8 > max_x8 {
            (*e).x8 = max_x8;
            return;
        }
        if ((*e).x8 as f64) < 0.0 {
            (*e).x8 = 0.0f32;
        }
    } else if ((*e).x4 > 0.827f32) && ((*e).x4 < 0.914f32) {
        if (*e).x8 < 0.001f32 {
            (*e).x8 = 0.001f32;
        }
    } else {
        (*e).x8 *= 0.95f32;
        if (*e).x8 < 0.00001f32 {
            (*e).x8 = 0.0f32;
        }
    }
}

#[inline(always)]
pub unsafe fn grMc_DistanceSquared(a: *const Vec3, b: *const Vec3) -> f32 {
    melee_core::ct!("grMc_DistanceSquared");
    let dx = (*a).x - (*b).x;
    let dy = (*a).y - (*b).y;
    let dz = (*a).z - (*b).z;
    // SQ(a->x - b->x) + ...: squares of computed expressions, not fused on the console (notes/fp-audit.md)
    dx * dx + dy * dy + dz * dz
}

pub unsafe extern "C" fn grMuteCity_801F1328() {
    melee_core::ct!("grMuteCity_801F1328");
    let arr = grMc_8049F440();

    for i in 1..30isize {
        let mut j = i;
        while j >= 0 {
            let temp = *arr.offset(j);
            if (*ce(*arr.offset(j) as usize)).x0 > (*ce(*arr.offset(j - 1) as usize)).x0 {
                *arr.offset(j) = *arr.offset(j - 1);
                *arr.offset(j - 1) = temp;
            } else {
                break;
            }
            j -= 1;
        }
    }

    for i in 0..30isize {
        (*ce(*arr.offset(i) as usize)).x20 = 0;
    }

    for i in 0..30isize {
        if (*ce(*arr.offset(i) as usize)).x22_flags.b0() != 0 {
            continue;
        }
        for j in 1..30isize {
            let sum = i + j;
            let mut idx = sum;

            if sum >= 30 {
                idx -= 30;
            }

            if (*ce(*arr.offset(idx) as usize)).x22_flags.b0() != 0 {
                continue;
            }

            let other_pos = (*ce(*arr.offset(idx) as usize)).x4;
            let my_pos = (*ce(*arr.offset(i) as usize)).x4;
            let dist = if my_pos > other_pos { my_pos - other_pos } else { 1.0f32 + (my_pos - other_pos) };

            if (dist as f64) > 0.002 {
                break;
            }
            (*ce(*arr.offset(idx) as usize)).x20 |= 1;

            if grMc_DistanceSquared(&(*ce(*arr.offset(i) as usize)).pos, &(*ce(*arr.offset(idx) as usize)).pos) < 900.0f32 {
                (*ce(*arr.offset(i) as usize)).x20 |= 8;
                (*ce(*arr.offset(idx) as usize)).x20 |= 8;
            }

            if (*ce(*arr.offset(i) as usize)).xC > (*ce(*arr.offset(idx) as usize)).xC {
                (*ce(*arr.offset(i) as usize)).x20 |= 4;
                (*ce(*arr.offset(idx) as usize)).x20 |= 2;
            } else {
                (*ce(*arr.offset(i) as usize)).x20 |= 2;
                (*ce(*arr.offset(idx) as usize)).x20 |= 4;
            }
        }
    }

    for i in 0..5isize {
        (*ce(*arr.offset(i) as usize)).x20 |= 0x10;
    }
    for i in 25..30isize {
        (*ce(*arr.offset(i) as usize)).x20 |= 0x20;
    }
    for i in 0..30 {
        grMuteCity_801F106C(i);
    }
}

macro_rules! car_age_fns {
    ($($name:ident = $idx:literal),* $(,)?) => {
        $(
            pub unsafe extern "C" fn $name(_item_gobj: *mut HSD_GObj, _gp: *mut Ground, _pos: *mut Vec3, _gobj: *mut HSD_GObj, arg: f32) {
                (*ce($idx)).x10 += arg;
            }
        )*
    };
}

car_age_fns! {
    grMuteCity_801F173C = 0, grMuteCity_801F1754 = 1, grMuteCity_801F176C = 2, grMuteCity_801F1784 = 3,
    grMuteCity_801F179C = 4, grMuteCity_801F17B4 = 5, grMuteCity_801F17CC = 6, grMuteCity_801F17E4 = 7,
    grMuteCity_801F17FC = 8, grMuteCity_801F1814 = 9, grMuteCity_801F182C = 10, grMuteCity_801F1844 = 11,
    grMuteCity_801F185C = 12, grMuteCity_801F1874 = 13, grMuteCity_801F188C = 14, grMuteCity_801F18A4 = 15,
    grMuteCity_801F18BC = 16, grMuteCity_801F18D4 = 17, grMuteCity_801F18EC = 18, grMuteCity_801F1904 = 19,
    grMuteCity_801F191C = 20, grMuteCity_801F1934 = 21, grMuteCity_801F194C = 22, grMuteCity_801F1964 = 23,
    grMuteCity_801F197C = 24, grMuteCity_801F1994 = 25, grMuteCity_801F19AC = 26, grMuteCity_801F19C4 = 27,
    grMuteCity_801F19DC = 28, grMuteCity_801F19F4 = 29,
}

pub unsafe extern "C" fn grMuteCity_801F1A0C(_gobj: *mut HSD_GObj, _gp: *mut Ground) {
    melee_core::ct!("grMuteCity_801F1A0C");
    ext::Ground_801C53EC(0x5CC60);
}

/// `grMc_803E3C6C`
pub static mut grMc_803E3C6C: [Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32)>; 30] = [
    Some(grMuteCity_801F173C), Some(grMuteCity_801F1754), Some(grMuteCity_801F176C),
    Some(grMuteCity_801F1784), Some(grMuteCity_801F179C), Some(grMuteCity_801F17B4),
    Some(grMuteCity_801F17CC), Some(grMuteCity_801F17E4), Some(grMuteCity_801F17FC),
    Some(grMuteCity_801F1814), Some(grMuteCity_801F182C), Some(grMuteCity_801F1844),
    Some(grMuteCity_801F185C), Some(grMuteCity_801F1874), Some(grMuteCity_801F188C),
    Some(grMuteCity_801F18A4), Some(grMuteCity_801F18BC), Some(grMuteCity_801F18D4),
    Some(grMuteCity_801F18EC), Some(grMuteCity_801F1904), Some(grMuteCity_801F191C),
    Some(grMuteCity_801F1934), Some(grMuteCity_801F194C), Some(grMuteCity_801F1964),
    Some(grMuteCity_801F197C), Some(grMuteCity_801F1994), Some(grMuteCity_801F19AC),
    Some(grMuteCity_801F19C4), Some(grMuteCity_801F19DC), Some(grMuteCity_801F19F4),
];

#[inline(always)]
pub unsafe fn grMc_GetTrackMidpoint(gp: *mut Ground) -> f32 {
    melee_core::ct!("grMc_GetTrackMidpoint");
    0.5f32 * ((*gp).u.mutecity.xD4 + (*gp).u.mutecity.xD8)
}

#[inline(always)]
pub unsafe fn grMc_WreckAge() -> f32 {
    melee_core::ct!("grMc_WreckAge");
    (*yakumono_param).x30
}

#[inline(always)]
pub unsafe fn grMc_SmokeAge() -> f32 {
    melee_core::ct!("grMc_SmokeAge");
    (*yakumono_param).x2C
}

#[inline(always)]
pub unsafe fn grMc_RandomWreckOffset() -> f32 {
    melee_core::ct!("grMc_RandomWreckOffset");
    fmadds(1.0471976f32, ext::HSD_Randf(), -0.5235988f32)
}

#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe extern "C" fn grMuteCity_801F1A34(arg0: *mut HSD_GObj, arg1: *mut HSD_GObj) {
    melee_core::ct!("grMuteCity_801F1A34");
    let mut car_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp_e8 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp_d8 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp_cc = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp_c0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp_b4 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp_a8 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp_9c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp_90 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut spawn_pos: Vec3;

    let mut sound_count: i32 = 0;
    let car_gp = (*arg0).user_data as *mut Ground;
    let gp = (*arg1).user_data as *mut Ground;
    let car_jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    ext::Ground_801C0498();
    let track_mid = grMc_GetTrackMidpoint(gp);
    ext::Camera_GetTransformPosition(&mut sp_e8);

    let mut jobj: *mut HSD_JObj = if car_jobj.is_null() { null_mut() } else { (*car_jobj).child };

    {
        let timer = (*car_gp).u.mutecity.xC4 as i32;
        if timer != 0 {
            (*car_gp).u.mutecity.xC4 = (timer - 1) as i16;
        }
    }

    let mut car_idx: usize = 0;

    loop {
        let e = ce(car_idx);
        (*e).x4 += (*e).x8;
        (*e).x0 += (*e).x8;
        {
            let p = (*e).x4;
            if p > 1.0 {
                (*e).x4 = p - 1.0;
            }
        }

        let mut spline_t = (*e).x4;

        grMc_SplineEvalWrapped(&mut sp_d8, (*(*gp).u.mutecity.xE0).u.spline, spline_t);
        ext::lb_8000B1CC((*gp).u.mutecity.xE0, &mut sp_d8, &mut sp_cc);

        grMc_SplineEvalWrapped(&mut sp_d8, (*(*gp).u.mutecity.xDC).u.spline, spline_t);
        ext::lb_8000B1CC((*gp).u.mutecity.xDC, &mut sp_d8, &mut sp_c0);

        ext::lbVector_Diff(&mut sp_c0, &mut sp_cc, &mut sp_b4);
        ext::lbVector_Normalize(&mut sp_b4);

        if spline_t > 0.827f32 && spline_t < 0.914f32 {
            spline_t = 0.0;
        }

        grMc_SplineTangentWrapped(&mut sp_a8, (*(*gp).u.mutecity.xE0).u.spline, spline_t);
        ext::lb_8000B1CC((*gp).u.mutecity.xE0, &mut sp_a8, &mut sp_a8);
        ext::lb_8000B1CC((*gp).u.mutecity.xE0, null_mut(), &mut sp_d8);
        ext::lbVector_Sub(&mut sp_a8, &mut sp_d8);
        ext::lbVector_Normalize(&mut sp_a8);

        grMc_SplineTangentWrapped(&mut sp_9c, (*(*gp).u.mutecity.xDC).u.spline, spline_t);
        ext::lb_8000B1CC((*gp).u.mutecity.xDC, &mut sp_9c, &mut sp_9c);
        ext::lb_8000B1CC((*gp).u.mutecity.xDC, null_mut(), &mut sp_d8);
        ext::lbVector_Sub(&mut sp_9c, &mut sp_d8);
        ext::lbVector_Normalize(&mut sp_9c);

        ext::lbVector_Add(&mut sp_a8, &mut sp_9c);
        ext::lbVector_Normalize(&mut sp_a8);
        ext::PSVECCrossProduct(&mut sp_a8, &mut sp_b4, &mut sp_90);
        ext::lbVector_Normalize(&mut sp_90);
        ext::PSVECCrossProduct(&mut sp_b4, &mut sp_90, &mut sp_a8);
        ext::lbVector_Normalize(&mut sp_a8);

        let xc = (*e).xC;
        car_pos.x = 3.0f64.fmadd(sp_90.x as f64, fmadds(sp_c0.x - sp_cc.x, xc, sp_cc.x) as f64) as f32;
        car_pos.y = 3.0f64.fmadd(sp_90.y as f64, fmadds(sp_c0.y - sp_cc.y, xc, sp_cc.y) as f64) as f32;
        car_pos.z = 3.0f64.fmadd(sp_90.z as f64, fmadds(sp_c0.z - sp_cc.z, xc, sp_cc.z) as f64) as f32;
        (*e).pos = car_pos;

        HSD_JObjSetTranslate(&mut *jobj, &car_pos, &mut Dirty);

        ext::Ground_801C5AEC(&mut sp_d8, &mut sp_a8, &mut sp_b4, &mut sp_90);
        HSD_JObjSetRotationX(&mut *jobj, sp_d8.x, &mut Dirty);
        HSD_JObjSetRotationY(&mut *jobj, sp_d8.y, &mut Dirty);
        HSD_JObjSetRotationZ(&mut *jobj, sp_d8.z, &mut Dirty);

        {
            let flags20 = (*e).x20;
            if (flags20 & 8 != 0) && (flags20 & 2 != 0) && (*car_gp).u.mutecity.xC4 == 0 {
                ext::grLib_801C96F8(0x11A, 0, &mut car_pos);
                if sound_count < 1 {
                    if car_pos.z < 0.0 {
                        // car_pos's address goes to grLib_801C96F8: squares not fused on the console
                        spline_t = car_pos.z * car_pos.z + (car_pos.x * car_pos.x + car_pos.y * car_pos.y);
                        let distance = sqrtf(spline_t);
                        spline_t = distance;
                        if spline_t < 300.0 {
                            match ext::HSD_Randi(3) {
                                0 => ext::Ground_801C5440(car_gp, 0, 0x5CC69),
                                1 => ext::Ground_801C5440(car_gp, 0, 0x5CC68),
                                2 => ext::Ground_801C5440(car_gp, 0, 0x5CC67),
                                _ => {}
                            }
                            ext::Ground_801C5630(car_gp, 0, 1.0f32 - (spline_t / 300.0f32));
                            sound_count += 1;
                        }
                    }
                    (*car_gp).u.mutecity.xC4 = 7;
                }
            }
        }

        {
            let age = (*e).x10;
            if age > grMc_WreckAge() {
                if (*e).x22_flags.b0() == 0 {
                    ext::grLib_801C98A0(jobj);
                    (*e).r#gen = null_mut();
                    ext::grLib_801C96F8(0xE3, 0, &mut car_pos);
                    ext::HSD_JObjUnref(HSD_JObjGetChild(Some(&*jobj)));
                    {
                        let new_jobj = ext::Ground_801C13D0(5, 0);
                        if !new_jobj.is_null() {
                            spawn_pos = grMc_803B81B8;
                            ext::HSD_JObjAddChild(jobj, new_jobj);
                            HSD_JObjSetTranslateY(&mut *new_jobj, grMc_RandomWreckOffset(), &mut Dirty);
                            HSD_JObjSetTranslate(&mut *new_jobj, &spawn_pos, &mut Dirty);
                        }
                    }
                    if !(*e).x24.is_null() {
                        ext::grMaterial_801C8CDC((*e).x24);
                        (*e).x24 = null_mut();
                    }
                    (*e).x22_flags.set_b0(1);
                }
            } else if age > grMc_SmokeAge() && (*e).r#gen.is_null() {
                (*e).r#gen = grMuteCity_801F2AB0(0x116, jobj);
            }
        }

        if (*e).x22_flags.b0() == 0 {
            let rel_t = (*e).x4;
            let dist_t = if track_mid < rel_t { (1.0f32 + track_mid) - rel_t } else { track_mid - rel_t };

            if dist_t < 0.2 && dist_t > -0.2 {
                if -200.0 < car_pos.x && car_pos.x < 200.0 && -200.0 < car_pos.y && car_pos.y < 200.0 && -1350.0 < car_pos.z && car_pos.z < 50.0 {
                    ext::Camera_RequestQuake(QuakeKind_Loop as u32, null_mut());
                    if (*e).x22_flags.b1() == 0 {
                        if (*gp).u.mutecity.xD0_flags.b1() != 0 {
                            match ext::HSD_Randi(3) {
                                0 => ext::Ground_801C53EC(0x5CC66),
                                1 => ext::Ground_801C53EC(0x5CC65),
                                2 => ext::Ground_801C53EC(0x5CC64),
                                _ => {}
                            }
                        } else {
                            match ext::HSD_Randi(3) {
                                0 => ext::Ground_801C53EC(0x5CC63),
                                1 => ext::Ground_801C53EC(0x5CC62),
                                2 => ext::Ground_801C53EC(0x5CC61),
                                _ => {}
                            }
                        }
                        (*e).x22_flags.set_b1(1);
                    }
                } else if (*e).x22_flags.b1() != 0 {
                    (*e).x22_flags.set_b1(0);
                }
            }

            if -200.0 < car_pos.x && car_pos.x < 200.0 && -200.0 < car_pos.y && car_pos.y < 200.0 && -100.0 < car_pos.z && car_pos.z < 50.0 {
                if (*e).x22_flags.b0() == 0 && (*e).x24.is_null() {
                    let item_gobj = ext::grMaterial_801C8CFC(0, 2, car_gp, jobj, Some(grMuteCity_801F1A0C), grMc_803E3C6C[car_idx], None);
                    if !item_gobj.is_null() {
                        ext::grMaterial_801C8DE0(item_gobj, 0.0, 0.0, -12.0, 0.0, 0.0, 2.0, 15.0);
                        ext::grMaterial_801C8E08(item_gobj);
                        (*e).x24 = item_gobj;
                    }
                }
            } else if !(*e).x24.is_null() {
                ext::grMaterial_801C8CDC((*e).x24);
                (*e).x24 = null_mut();
            }
        } else if car_pos.z > sp_e8.z || car_pos.z > 5000.0 || car_pos.z < -1500.0 {
            if !(*e).r#gen.is_null() {
                ext::grLib_801C98A0(jobj);
                (*e).r#gen = null_mut();
            }
        } else if (*e).r#gen.is_null() {
            (*e).r#gen = grMuteCity_801F2AB0(0x119, jobj);
        }

        car_idx += 1;
        jobj = if jobj.is_null() { null_mut() } else { (*jobj).next };
        if car_idx >= 0x1E {
            break;
        }
    }
}

pub unsafe extern "C" fn grMuteCity_801F28A8() -> *mut DynamicModelDesc {
    melee_core::ct!("grMuteCity_801F28A8");
    let archive = ext::grDatFiles_801C6330(38);
    if archive.is_null() {
        panic!("HSD_ASSERT(2135, archive) grmutecity.c");
    }
    let dat = (*archive).unk4;
    if !dat.is_null() {
        return (*dat).unk8.add(38) as *mut DynamicModelDesc;
    }
    null_mut()
}

/// `grMc_StackPadArg`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GrMcStackPadArg {
    pub x: [u32; 4],
}

/// "Fake function: exists only to reproduce the retail stack layout."
pub unsafe extern "C" fn grMc_StackPad(_arg: GrMcStackPadArg) { melee_core::ct!("grMc_StackPad");}

pub unsafe extern "C" fn grMuteCity_801F290C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grMuteCity_801F290C");
    let gp = (*gobj).user_data as *mut Ground;

    if ext::grLib_801C96E8(gobj) != 0 {
        if (*gp).u.mutecity2.xC4_flags.b0() != 0 {
            let mut lgobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_LIGHT as usize);
            while !lgobj.is_null() {
                if HSD_GObjGetClassifier(&*lgobj) == 0xC {
                    break;
                }
                lgobj = (*lgobj).next;
            }
            if !lgobj.is_null() {
                let mut lobj = (*lgobj).hsd_obj as *mut HSD_LObj;
                let mut i = 0usize;
                while !lobj.is_null() {
                    if ext::HSD_LObjGetFlags(lobj) & 4 != 0 {
                        ext::HSD_LObjSetColor(lobj, (*gp).u.mutecity2.saved_colors[i]);
                        i += 1;
                        if i >= 4 {
                            break;
                        }
                    }
                    lobj = HSD_LObjGetNext(Some(&*lobj));
                }
            }
            (*gp).u.mutecity2.xC4_flags.set_b0(0);
        }
    } else {
        let mut lgobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_LIGHT as usize);
        while !lgobj.is_null() {
            if HSD_GObjGetClassifier(&*lgobj) == 0xC {
                break;
            }
            lgobj = (*lgobj).next;
        }
        if !lgobj.is_null() {
            let mut j = 0usize;
            let mut lobj2 = (*lgobj).hsd_obj as *mut HSD_LObj;
            while !lobj2.is_null() {
                if ext::HSD_LObjGetFlags(lobj2) & 4 != 0 {
                    if (*gp).u.mutecity2.xC4_flags.b0() == 0 {
                        ext::HSD_LObjGetColor(lobj2, &mut (*gp).u.mutecity2.saved_colors[j]);
                    }
                    ext::HSD_LObjSetColor(lobj2, (*gp).color_overlay.x2C_hex);
                    j += 1;
                    if j >= 4 {
                        break;
                    }
                }
                lobj2 = if lobj2.is_null() { null_mut() } else { (*lobj2).next };
            }
        }
        (*gp).u.mutecity2.xC4_flags.set_b0(1);
    }
    grMc_StackPad(*((*gp).u.mutecity2.saved_colors.as_ptr() as *const GrMcStackPadArg));
}

/// "Retail never sets the return value" (the PC build returns the generator or NULL).
pub unsafe extern "C" fn grMuteCity_801F2AB0(arg0: i32, jobj: *mut HSD_JObj) -> *mut HSD_Generator {
    melee_core::ct!("grMuteCity_801F2AB0");
    let gen = ext::grLib_801C9808(arg0, arg0 / 1000, jobj);

    if !gen.is_null() {
        let mut appsrt = (*gen).appsrt;
        if appsrt.is_null() {
            appsrt = ext::psAddGeneratorAppSRT_begin(gen, 0);
        }
        if !appsrt.is_null() {
            (*appsrt).xA2 = 0;
            let s = ext::Ground_801C0498();
            (*appsrt).scale.z = s;
            (*appsrt).scale.y = s;
            (*appsrt).scale.x = s;
            (*gen).r#type &= !(PSAPPSRT_UNK_B09 | PSAPPSRT_UNK_B10) as u16;
            (*gen).r#type |= PSAPPSRT_UNK_B11 as u16;
            (*appsrt).gp = gen;
            return gen;
        }
    }
    null_mut()
}

/// `mpLib_JointCollisionCallback`
pub unsafe extern "C" fn fn_801F2B58(user_data: *mut c_void, joint_id: i32, coll: *mut CollData, coll_x50: i32, ground_kind: u32, delta_y: f32) {
    melee_core::ct!("fn_801F2B58");
    let b1234 = (*coll).x34_flags.b1234() as i32;

    if grMc_804D69D4 != 1 {
        return;
    }

    if ((b1234 - 2) as u32) > 1 && b1234 != 5 {
        return;
    }

    if ground_kind != mpLib_GroundEnum_Unk2 as u32 {
        return;
    }

    if (*coll).x0_gobj.is_null() {
        return;
    }

    if (*(*coll).x0_gobj).p_link != 9 {
        return;
    }

    if (*coll).x0_gobj.is_null() {
        return;
    }

    (*coll).x34_flags.set_b7(1);
}

pub unsafe extern "C" fn grMuteCity_801F2BBC(arg0: i32) -> *mut melee_types::lbColl_80008D30_arg1 {
    melee_core::ct!("grMuteCity_801F2BBC");
    if grMc_804D69D4 == 1 {
        if arg0 == 0x31 {
            return (*yakumono_param).x8 as *mut lbColl_80008D30_arg1;
        }
        if arg0 == 0x35 {
            return (*yakumono_param).x8 as *mut lbColl_80008D30_arg1;
        }
        if ((arg0 - 0x32) as u32) <= 2 {
            return (*yakumono_param).xC as *mut melee_types::lbColl_80008D30_arg1;
        }
    }
    null_mut()
}

pub unsafe extern "C" fn grMuteCity_801F2C10(pos: *mut Vec3, arg: i32, jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grMuteCity_801F2C10");
    let gobj = ext::Ground_GetMapGObj(0x1E);
    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        if !gp.is_null() {
            if (*gp).u.mutecity.xFC == jobj
                || (*gp).u.mutecity.x100 == jobj
                || (*gp).u.mutecity.x104 == jobj
                || (*gp).u.mutecity.x108 == jobj
                || (*gp).u.mutecity.x10C == jobj
            {
                let x = (*pos).x;
                let x_min = (*gp).u.mutecity.xE4.x;
                if x < x_min {
                    return 0;
                }
                let x_max = (*gp).u.mutecity.xF0.x;
                if x > x_max {
                    return 0;
                }
                let slope = ((*gp).u.mutecity.xF0.y - (*gp).u.mutecity.xE4.y) / (x_max - x_min);
                let y = fmadds(slope, x - x_min, (*gp).u.mutecity.xE4.y);
                if (*pos).y > y - 3.0f32 {
                    return 1;
                }
                return 0;
            }
        }
    }
    1
}

// ---- folded from grmutecity.rs (consolidation steps 21/22) ----
/// grMuteCity_801EFCE0_fp (gr/grmutecity.c:398)
    pub unsafe extern "C" fn grMuteCity_801EFCE0_fp() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

/// stageGObj36_37_OnInit_fp (gr/grmutecity.c:601)
    pub unsafe extern "C" fn stageGObj36_37_OnInit_fp(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grmutecity.rs (consolidation steps 21/22) ----
/// grMuteCity_801EFD04_fp (gr/grmutecity.c:403)
    pub extern "C" fn grMuteCity_801EFD04_fp() -> i32 { 0 }

/// grMuteCity_801F0118_fp (gr/grmutecity.c:513)
    pub extern "C" fn grMuteCity_801F0118_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grMuteCity_801F0288_fp (gr/grmutecity.c:558)
    pub extern "C" fn grMuteCity_801F0288_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grMuteCity_801F043C_fp (gr/grmutecity.c:606)
    pub extern "C" fn grMuteCity_801F043C_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grMuteCity_801F04A8_fp (gr/grmutecity.c:630)
    pub extern "C" fn grMuteCity_801F04A8_fp(a0: *mut HSD_GObj) -> i32 { 0 }
