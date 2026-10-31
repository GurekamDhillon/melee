//! Port of src/melee/gr/grpstadium.c (Pokemon Stadium: the stage callbacks, the transformation state
//! machine, the jumbotron display with its camera feeds and text). Every function of the TU, native:
//! pointers into the game's structs, callees of other TUs through `ext` hooks. Checked against the C
//! (whole TU, shipping contraction, stateful oracle) by tests/tu_grpstadium.rs; regenerate the C side with
//! `python rust/melee_world/gen/cref/run_gr.py grpstadium`.
//!
//! Deviations from a literal copy (the oracle patches the C the same way, spec `patch`):
//! * `grStadium_801D4548` case 2 writes `u.display.xD8 = NULL`, which on the GameCube layout is the
//!   4-byte `u.stadium.xD8` countdown (both views start at gp+D8); on the native layout the two views
//!   differ, so it is written as `u.stadium.xD8 = 0`.
//! * `grStadium_801D3BBC` reads `colors.entries[n].packed`, a `u32` over the four bytes of an alpha
//!   preset; the game is big-endian, so it is composed big-endian from the bytes.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2i32;
use melee_types::inline::*;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
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
    fn Camera_80029020() -> *mut CmSubject;
    fn Camera_80030A50() -> *mut HSD_GObj = || unsafe { crate::cm::camera::Camera_80030A50() as _ };
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec) = |a0: u32, a1: *mut Vec| unsafe { crate::cm::camera::Camera_RequestQuake(a0 as _, a1 as _) };
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::GObj_InitUserData(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _); } };
    fn GXGetTexBufferSize(a0: u16, a1: u16, a2: u32, a3: u8, a4: u8) -> u32;
    fn Ground_801C0498() -> f32 = || unsafe { crate::tu::ground::Ground_801C0498() as _ };
    fn Ground_801C10B8(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>| { unsafe { crate::tu::ground::Ground_801C10B8(a0 as _, a1 as _); } };
    fn Ground_801C3214(a0: i32) -> i32 = |a0: i32| unsafe { crate::tu::ground::Ground_801C3214(a0 as _) as _ };
    fn Ground_801C39C0() = || { unsafe { crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C4A08(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::ground::Ground_801C4A08(a0 as _); } };
    fn Ground_801C53EC(a0: u32) = |a0: u32| { unsafe { crate::tu::ground::Ground_801C53EC(a0 as _); } };
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetMapGObj(a0 as _) as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void = |a0: *mut HSD_Archive, a1: *const i8| unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0 as _, a1 as _) as _ };
    fn HSD_CObjEndCurrent() = || { unsafe { melee_runtime::tu::cobj::HSD_CObjEndCurrent(); } };
    fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32) = |a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32| { unsafe { melee_runtime::tu::cobj::HSD_CObjEraseScreen(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0 as _) as _ };
    fn HSD_CObjSetOrtho(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32) = |a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32| { unsafe { melee_runtime::tu::cobj::HSD_CObjSetOrtho(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } };
    fn HSD_FogSet(a0: *mut HSD_Fog);
    fn HSD_Free(a0: *mut c_void) = |a0: *mut c_void| { unsafe { melee_runtime::tu::memory::HSD_Free(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32) = |a0: *mut HSD_GObj, a1: u32| { unsafe { melee_runtime::tu::gobj_native::HSD_GObj_80390ED0(a0 as _, a1 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjClearFlags(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlags(a0 as _, a1 as _); } };
    fn HSD_JObjSetFlags(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlags(a0 as _, a1 as _); } };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_MObjClearFlags(a0: *mut HSD_MObj, a1: u32) = |a0: *mut HSD_MObj, a1: u32| { unsafe { melee_runtime::tu::mobj::HSD_MObjClearFlags(a0 as _, a1 as _); } };
    fn HSD_MObjCompileTev(a0: *mut HSD_MObj) = |a0: *mut HSD_MObj| { unsafe { melee_runtime::tu::mobj::HSD_MObjCompileTev(a0 as _); } };
    fn HSD_MObjSetDiffuseColor(a0: *mut HSD_MObj, a1: u8, a2: u8, a3: u8) = |a0: *mut HSD_MObj, a1: u8, a2: u8, a3: u8| { unsafe { melee_runtime::tu::mobj::HSD_MObjSetDiffuseColor(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_MObjSetFlags(a0: *mut HSD_MObj, a1: u32) = |a0: *mut HSD_MObj, a1: u32| { unsafe { melee_runtime::tu::mobj::HSD_MObjSetFlags(a0 as _, a1 as _); } };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_Randf() -> f32 = || melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand());
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text = |a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32| unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ };
    fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 = |a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32| unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A611C(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _) as _ };
    fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) = |a0: *mut HSD_Text, a1: i32| { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A6368(a0 as _, a1 as _); } };
    fn HSD_SisLib_803A6530(a0: i32, a1: i32, a2: i32) -> *mut u8 = |a0: i32, a1: i32, a2: i32| unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6530(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_SisLib_803A660C(a0: i32, a1: i32, a2: i32) = |a0: i32, a1: i32, a2: i32| { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A660C(a0 as _, a1 as _, a2 as _); } };
    fn HSD_SisLib_803A6754(a0: i32, a1: i32) -> *mut HSD_Text = |a0: i32, a1: i32| unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6754(a0 as _, a1 as _) as _ };
    fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const i8, a4: usize, a5: usize, a6: usize) -> i32 = |a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const i8, a4: usize, a5: usize, a6: usize| { unsafe { let s = String::from_utf8_lossy(&melee_runtime::cformat::c_format(a3, &[melee_runtime::cformat::PrintfArg::Int(a4 as i32), melee_runtime::cformat::PrintfArg::Int(a5 as i32), melee_runtime::cformat::PrintfArg::Int(a6 as i32)])).into_owned(); melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6B98(a0, a1, a2, if a3.is_null() { None } else { Some(&s) }) } };
    fn HSD_SisLib_803A7664(a0: *mut HSD_Text) = |a0: *mut HSD_Text| { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A7664(a0 as _); } };
    fn Player_8003219C(a0: i32) -> i32;
    fn Player_GetEntity(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::pl::Player_GetEntity(a0 as _) as _ };
    fn Player_GetPlayerSlotType(a0: i32) -> Gm_PKind = |a0: i32| unsafe { crate::pl::Player_GetPlayerSlotType(a0 as _) as _ };
    fn Player_GetTeam(a0: i32) -> i32 = |a0: i32| unsafe { crate::pl::Player_GetTeam(a0 as _) as _ };
    fn Slippi_FrozenStadium() -> i32;
    fn Stage_80225194() -> u32 = || unsafe { crate::tu::stage::Stage_80225194() as _ };
    fn ftLib_80086B90(a0: *mut HSD_GObj, a1: *mut Vec) = |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_80086B90(a0 as _, a1 as _); } };
    fn gm_80160854(a0: u8, a1: u8, a2: u8, a3: u8) -> u32;
    fn gm_80160968(a0: u32) -> _GXColor;
    fn gm_80160A60(a0: i32) -> *const i8;
    fn gm_801694A0(a0: *mut HSD_GObj) -> i32;
    fn gm_8016AEEC() -> u32 = || unsafe { crate::gm::gm_8016AEEC() as _ };
    fn gm_8016AF0C() -> u16 = || unsafe { crate::gm::gm_8016AF0C() as _ };
    fn gm_8016B168() -> i32 = || unsafe { crate::gm::gm_8016B168() as _ };
    fn gm_8016B184() -> i32 = || unsafe { crate::gm::gm_8016B184() as _ };
    fn gm_8016C6C0(a0: *mut HSD_GObj) -> i32;
    fn gm_8018841C() -> i32;
    fn gm_GetStartMeleeRules() -> *mut StartMeleeRules = || unsafe { crate::gm::gm_GetStartMeleeRules() as _ };
    fn grAnime_801C65B0(a0: *mut UnkArchiveStruct) = |a0: *mut UnkArchiveStruct| { unsafe { crate::tu::granime::grAnime_801C65B0(a0 as _); } };
    fn grAnime_801C77FC(a0: *mut HSD_GObj, a1: i32, a2: u32) = |a0: *mut HSD_GObj, a1: i32, a2: u32| { unsafe { crate::tu::granime::grAnime_801C77FC(a0 as _, a1 as _, a2 as _); } };
    fn grAnime_801C7A04(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32) = |a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32| { unsafe { crate::tu::granime::grAnime_801C7A04(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grDatFiles_801C6330(a0: i32) -> *mut UnkArchiveStruct = |a0: i32| unsafe { crate::tu::grdatfiles::grDatFiles_801C6330(a0 as _) as _ };
    fn grDatFiles_801C6478(a0: *mut c_void, a1: i32) -> *mut UnkArchiveStruct = |a0: *mut c_void, a1: i32| unsafe { crate::tu::grdatfiles::grDatFiles_801C6478(a0 as _, a1 as _) as _ };
    fn grDatFiles_GetArchive() -> *mut UnkArchiveStruct = || unsafe { crate::tu::grdatfiles::grDatFiles_GetArchive() as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grLib_801C96F8(a0: i32, a1: i32, a2: *mut Vec) -> *mut HSD_Generator = |a0: i32, a1: i32, a2: *mut Vec| unsafe { crate::tu::grlib::grLib_801C96F8(a0 as _, a1 as _, a2 as _) as _ };
    fn grLib_801C9808(a0: i32, a1: i32, a2: *mut HSD_JObj) -> *mut HSD_Generator = |a0: i32, a1: i32, a2: *mut HSD_JObj| unsafe { crate::tu::grlib::grLib_801C9808(a0 as _, a1 as _, a2 as _) as _ };
    fn grLib_801C9834(a0: *mut HSD_Generator) = |a0: *mut HSD_Generator| { unsafe { crate::tu::grlib::grLib_801C9834(a0 as _); } };
    fn grLib_801C9874(a0: *mut HSD_Generator) = |a0: *mut HSD_Generator| { unsafe { crate::tu::grlib::grLib_801C9874(a0 as _); } };
    fn grLib_801C9908(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { crate::tu::grlib::grLib_801C9908(a0 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn lbDvd_80017740(a0: i32, a1: i32, a2: i32, a3: i32, a4: u32, a5: i32, a6: i32, a7: u8, a8: i32) -> *mut c_void = |a0: i32, a1: i32, a2: i32, a3: i32, a4: u32, a5: i32, a6: i32, a7: u8, a8: i32| unsafe { melee_runtime::tu::lbdvd::lbDvd_80017740(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _, a8 as _) as _ };
    fn lbDvd_GetPreloadedArchive(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::lbdvd::lbDvd_GetPreloadedArchive(a0 as _) as _ };
    fn lbFile_80016580(a0: *const i8, a1: *mut c_void, a2: *mut u32, a3: Option<unsafe extern "C" fn(i32, u32, *mut c_void, i32)>, a4: u32);
    fn lbLang_IsSavedLanguageJP() -> i32 = || unsafe { melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageJP() as _ };
    fn lbVector_Add(a0: *mut Vec, a1: *mut Vec) -> *mut Vec = |a0: *mut Vec, a1: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_Add(a0 as _, a1 as _) as _ };
    fn lbVector_Diff(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec = |a0: *mut Vec, a1: *mut Vec, a2: *mut Vec| unsafe { melee_runtime::tu::lbvector::lbVector_Diff(a0 as _, a1 as _, a2 as _) as _ };
    fn lbVector_WorldToScreen(a0: *mut HSD_CObj, a1: *const Vec, a2: *mut Vec, a3: i32) -> *mut Vec = |a0: *mut HSD_CObj, a1: *const Vec, a2: *mut Vec, a3: i32| unsafe { melee_runtime::tu::lbvector::lbVector_WorldToScreen(a0 as _, a1 as _, a2 as _, a3 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn lb_800121FC(a0: *mut HSD_ImageDesc, a1: i32, a2: i32, a3: u32, a4: i16) -> *mut HSD_ImageDesc = |a0: *mut HSD_ImageDesc, a1: i32, a2: i32, a3: u32, a4: i16| unsafe { melee_runtime::tu::lbspdisplay::lb_800121FC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _) as _ };
    fn lb_800122C8(a0: *mut HSD_ImageDesc, a1: u16, a2: u16, a3: i32) = |a0: *mut HSD_ImageDesc, a1: u16, a2: u16, a3: i32| unsafe { melee_runtime::tu::lbspdisplay::lb_800122C8(a0 as _, a1 as _, a2 as _, a3 != 0) };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj = |a0: *mut HSD_CameraDescPerspective| unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0 as _) as _ };
    fn memzero(a0: *mut c_void, a1: i32) = |a0: *mut c_void, a1: i32| { unsafe { melee_runtime::tu::lb_00B0::memzero(a0 as _, a1 as _); } };
    fn mpJointListAdd(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpJointListAdd(a0 as _); } };
    fn mpLib_8005667C(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_8005667C(a0 as _); } };
    fn mpLib_80057528(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057528(a0 as _); } };
    fn mpLib_800575B0(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_800575B0(a0 as _); } };
    fn mpLib_80057BC0(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057BC0(a0 as _); } };
    fn mpLib_800581DC(a0: i32, a1: i32) = |a0: i32, a1: i32| { unsafe { crate::tu::mplib::mpLib_800581DC(a0 as _, a1 as _); } };
    fn mpLib_80058560() = || { unsafe { crate::tu::mplib::mpLib_80058560(); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn Camera_80030A50() -> *mut HSD_GObj { super::Camera_80030A50() }
        pub unsafe extern "C" fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj { super::Ground_GetMapGObj(a0) }
        pub unsafe extern "C" fn HSD_Free(a0: *mut c_void) { super::HSD_Free(a0) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) { super::grDisplay_801C5DB0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use melee_runtime::tu::sislib::HSD_SisLib_804D1124;
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

/// gr/inlines.h `Ground_UpdateWindAndMapColl`.
unsafe fn Ground_UpdateWindAndMapColl(gobj: *mut HSD_GObj) {
    ext::lb_800115F4();
    ext::Ground_UpdateMapColl(gobj);
}

/// random.h `randi`.
unsafe fn randi(max_val: i32) -> i32 {
    if max_val != 0 {
        return ext::HSD_Randi(max_val);
    }
    0
}

/// `union_type_dobj(o)`: `(o)->flags & (JOBJ_PTCL | JOBJ_SPLINE) ? false : true`
#[inline(always)]
unsafe fn union_type_dobj(o: *mut HSD_JObj) -> bool {
    (*o).flags & (JOBJ_PTCL | JOBJ_SPLINE) as u32 == 0
}

/// `static struct grPStadium_YakumonoParam`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GrPStadiumYakumonoParam {
    pub x0: i32,
    pub x4: i32,
    pub x8: i32,
    pub xC: i32,
    pub x10: i32,
    pub x14: i32,
    pub x18: i32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub x20: u32,
    pub x24: u32,
    pub x28: u32,
    pub x2C: u32,
    pub x30: u32,
    pub x34: u32,
    pub x38: u32,
    pub x3C: u32,
    pub x40: u32,
    pub x44: u32,
    pub x48: i16,
    pub x4A: i16,
    pub x4C: i16,
    pub x4E: i16,
    pub x50: i16,
}
melee_core::gc_layout!(GrPStadiumYakumonoParam, "grPStadium_YakumonoParam", size 0x54, align 4, [
    x0 @ 0x0: 4 "int", x4 @ 0x4: 4 "int", x8 @ 0x8: 4 "int", xC @ 0xC: 4 "int", x10 @ 0x10: 4 "int", x14 @ 0x14: 4 "int", x18 @ 0x18: 4 "int",
    r @ 0x1C: 1 "u8", g @ 0x1D: 1 "u8", b @ 0x1E: 1 "u8",
    x20 @ 0x20: 4 "u32", x24 @ 0x24: 4 "u32", x28 @ 0x28: 4 "u32", x2C @ 0x2C: 4 "u32", x30 @ 0x30: 4 "u32", x34 @ 0x34: 4 "u32",
    x38 @ 0x38: 4 "u32", x3C @ 0x3C: 4 "u32", x40 @ 0x40: 4 "u32", x44 @ 0x44: 4 "u32",
    x48 @ 0x48: 2 "s16", x4A @ 0x4A: 2 "s16", x4C @ 0x4C: 2 "s16", x4E @ 0x4E: 2 "s16", x50 @ 0x50: 2 "s16",
]);

/// `static ... * yakumono_param`
pub static mut yakumono_param: *mut GrPStadiumYakumonoParam = null_mut();

/// `GrJoint grPs_803E1248[]`
pub static mut grPs_803E1248: [GrJoint; 6] = [
    GrJoint { x: 1, y: 3, z: 0 },
    GrJoint { x: 2, y: 3, z: 0 },
    GrJoint { x: 3, y: 4, z: 0 },
    GrJoint { x: 4, y: 5, z: 0 },
    GrJoint { x: 5, y: 6, z: 0 },
    GrJoint { x: 7, y: 9, z: 0 },
];

const fn cb(on_init: HSD_GObjEvent, callback1: HSD_GObjPredicate, gobj_proc: HSD_GObjEvent, callback3: Option<unsafe extern "C" fn(*mut Ground_GObj)>, flags: u32) -> StageCallbacks {
    StageCallbacks { on_init, callback1, gobj_proc, callback3, anon_10: StageCallbacks_anon_135 { flags: u32::to_be(flags) } }
}

/// `static StageCallbacks grPs_StageCallbacks[]`
pub static mut grPs_StageCallbacks: [StageCallbacks; 10] = [
    cb(None, None, None, None, 0),
    cb(Some(grStadium_801D1290), Some(grStadium_801D1388), Some(grStadium_801D1390), Some(grStadium_801D13C4), 0),
    cb(Some(grStadium_801D13E0), Some(grStadium_801D1518), Some(grStadium_801D1520), Some(grStadium_801D156C), 0xC0000000),
    cb(Some(grStadium_801D1840), Some(grStadium_801D19D0), Some(stageGObj3_GObjProc), Some(grStadium_801D19F8), 0),
    cb(Some(grStadium_801D1648), Some(grStadium_801D16D4), Some(stageGObj4_GObjProc), Some(grStadium_801D16FC), 0),
    cb(Some(grStadium_801D1570), Some(grStadium_801D15FC), Some(stageGObj5_GObjProc), Some(grStadium_801D1624), 0),
    cb(Some(grStadium_801D1720), Some(grStadium_801D17E0), Some(stageGObj6_GObjProc), Some(grStadium_801D1808), 0),
    cb(Some(stageGObj7_8_OnInit), Some(grStadium_801D1E10), Some(grStadium_801D1E18), Some(grStadium_801D1E1C), 0),
    cb(Some(stageGObj7_8_OnInit), Some(grStadium_801D1E10), Some(grStadium_801D1E18), Some(grStadium_801D1E1C), 0),
    cb(Some(grStadium_801D1A38), Some(grStadium_801D1B40), Some(grStadium_801D1B48), Some(grStadium_801D1D84), 0),
];

/// `grPs_StageData`
pub static mut grPs_StageData: StageData = StageData {
    grkind: Gr_Kind_PStadium as _,
    callbacks: unsafe { &raw mut grPs_StageCallbacks as *mut StageCallbacks },
    data1: b"/GrPs\0".as_ptr() as *mut i8,
    on_init: Some(grStadium_OnInit),
    on_demo_init: Some(grStadium_OnDemoInit),
    on_load: Some(grStadium_OnLoad),
    on_start: Some(grStadium_OnStart),
    callback4: Some(grStadium_801D10F0),
    on_touch_line: Some(grStadium_OnTouchLine),
    on_check_shadow_render: Some(grStadium_OnCheckShadowRender),
    flags2: 1,
    joints: unsafe { &raw mut grPs_803E1248 as *mut GrJoint },
    joint_count: 6,
};

pub unsafe extern "C" fn grStadium_OnDemoInit(_unused: i32) { melee_core::ct!("grStadium_OnDemoInit");}

pub unsafe extern "C" fn grStadium_OnInit() {
    melee_core::ct!("grStadium_OnInit");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut GrPStadiumYakumonoParam;

    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);

    grStadium_801D10F8(0);
    grStadium_801D10F8(PsType_Display);
    grStadium_801D10F8(2);

    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();

    ext::mpLib_80057BC0(1);
    ext::mpLib_80057BC0(2);
    ext::mpLib_80057BC0(3);
    ext::mpLib_80057BC0(5);
    ext::mpLib_80057BC0(7);
    ext::mpLib_80057BC0(0);
    ext::mpLib_800581DC(6, 4);
}

pub unsafe extern "C" fn grStadium_OnLoad() { melee_core::ct!("grStadium_OnLoad");}

pub unsafe extern "C" fn grStadium_OnStart() {
    melee_core::ct!("grStadium_OnStart");
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe extern "C" fn grStadium_801D10F0() -> i32 {
    melee_core::ct!("grStadium_801D10F0");
    0
}

/// Creates a specific sub-type of Ground_GObj for Pokemon Stadium (e.g. id=1 for the jumbotron display)
pub unsafe extern "C" fn grStadium_801D10F8(id: u32) -> *mut HSD_GObj {
    melee_core::ct!("grStadium_801D10F8");
    let cb = (&raw mut grPs_StageCallbacks as *mut StageCallbacks).add(id as usize);
    let gobj = ext::Ground_GetStageGObj(id as i32);

    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        (*gp).x8_callback = None;
        (*gp).xC_callback = None;
        ext::GObj_SetupGXLink(gobj, Some(ext::cb::grDisplay_801C5DB0), 3, 0);
        if (*cb).callback3.is_some() {
            (*gp).x1C_callback = (*cb).callback3;
        }
        if let Some(f) = (*cb).on_init {
            f(gobj);
        }
        if (*cb).gobj_proc.is_some() {
            ext::HSD_GObj_SetupProc(gobj, (*cb).gobj_proc, 4);
        }
    }
    gobj
}

pub unsafe extern "C" fn fn_801D11E4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801D11E4");
    let gp = (*gobj).user_data as *mut Ground;
    let cam = ext::Camera_80029020();
    (*gp).u.display.xF4 = cam;

    if !cam.is_null() {
        let scale = ext::Ground_801C0498();
        (*cam).state = CmSubjectState_Inactive as _;
        (*cam).pos.x = 0.0;
        (*cam).pos.y = 30.0 * scale;
        (*cam).pos.z = 0.0;
        (*cam).target_ext.h.x = -25.0 * scale;
        (*cam).target_ext.h.y = 25.0 * scale;
        (*cam).target_ext.v.x = 10.0 * scale;
        (*cam).target_ext.v.y = -10.0 * scale;
    }
}

pub unsafe extern "C" fn grStadium_801D1290(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1290");
    let mut mobj: *mut HSD_MObj = null_mut();
    let gp = (*gobj).user_data as *mut Ground;

    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C77FC(gobj, 0, 7);
    (*gp).u.display.xD0 = ext::HSD_ArchiveGetPublicAddress(
        (*ext::grDatFiles_801C6330(1)).unk0,
        b"GrdPStadiumBG_OVDummy_mat6962_GrdPStadiumDummy_0_image_desc\0".as_ptr() as *const i8,
    ) as *mut HSD_ImageDesc;
    (*gp).u.display.xC8 = grStadium_801D3138(gobj, (*gp).u.display.xD0, &mut mobj);
    (*gp).u.display.xCC = mobj;
    grStadium_801D2278(gobj);
    ext::Ground_801C10B8(gobj, Some(fn_801D11E4));
    (*gp).x11_flags.set_b012(1);
    (*gobj).render_cb = Some(fn_801D5074);
}

pub unsafe extern "C" fn grStadium_801D1388(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grStadium_801D1388");
    0
}

pub unsafe extern "C" fn grStadium_801D1390(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1390");
    grStadium_801D1E20();
    grStadium_801D2344(gobj);
}

pub unsafe extern "C" fn grStadium_801D13C4(_gobj: *mut HSD_GObj) { melee_core::ct!("grStadium_801D13C4");}

pub unsafe extern "C" fn fn_801D13C8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801D13C8");
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).u.stadium.set_xC4_b0(0);
}

pub unsafe extern "C" fn grStadium_801D13E0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D13E0");
    let gr = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::Ground_InitMapColl(jobj, (*gr).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gr).map_id as i32, 0);
    let mobj = ext::lbDvd_GetPreloadedArchive(0x7D5);
    (*gr).u.stadium.xCC = mobj as *mut HSD_MObj;
    if mobj.is_null() {
        (*gr).u.stadium.xCC = ext::HSD_MemAlloc(0x50000) as *mut HSD_MObj;
    }
    (*gr).u.stadium.set_xC4_b1(0);
    (*gr).u.stadium.xD0 = null_mut();
    (*gr).u.stadium.xD4 = 0.0;
    (*gr).u.stadium.xDC = 0;
    (*gr).u.stadium.xDE = 5;
    (*gr).u.stadium.xE0 = -1;
    (*gr).u.stadium.xE2 = -1;
    (*gr).u.stadium.xE4 = grStadium_801D10F8((*gr).u.stadium.xDE as i32 as u32);
    (*gr).u.stadium.xE8 = null_mut();
    let mut var_r27 = (*yakumono_param).x4;
    let temp_r28 = (*yakumono_param).x0;
    if var_r27 > (*yakumono_param).x0 {
        var_r27 = temp_r28.wrapping_add(randi(var_r27.wrapping_sub(temp_r28)));
    } else if var_r27 < temp_r28 {
        var_r27 = var_r27.wrapping_add(randi(temp_r28.wrapping_sub(var_r27)));
    }
    (*gr).u.stadium.xD8 = var_r27;
    (*gr).x11_flags.set_b012(1);
    ext::mpLib_800575B0(0x55);
    ext::mpLib_800575B0(0x6F);
    ext::Ground_801C10B8(gobj, Some(fn_801D13C8));
    (*gr).u.stadium.set_xC4_b0(1);
}

pub unsafe extern "C" fn grStadium_801D1518(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grStadium_801D1518");
    0
}

pub unsafe extern "C" fn grStadium_801D1520(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1520");
    let gp = (*gobj).user_data as *mut Ground;
    if (*gp).u.stadium.xC4_b0() == 0 {
        grStadium_801D4548(gobj);
    }
    Ground_UpdateWindAndMapColl(gobj);
}

pub unsafe extern "C" fn grStadium_801D156C(_gobj: *mut HSD_GObj) { melee_core::ct!("grStadium_801D156C");}

pub unsafe extern "C" fn grStadium_801D1570(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1570");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::Ground_801C3214((*gp).map_id as i32);
    ext::mpJointListAdd(4);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).u.stadium.set_xC4_b0(0);
    (*gp).u.stadium.set_xC4_b1(0);
    (*gp).x10_flags.set_b5(1);
}

pub unsafe extern "C" fn grStadium_801D15FC(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grStadium_801D15FC");
    0
}

pub unsafe extern "C" fn stageGObj5_GObjProc(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj5_GObjProc");
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grStadium_801D1624(_gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1624");
    ext::mpLib_80057BC0(4);
}

pub unsafe extern "C" fn grStadium_801D1648(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1648");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::Ground_801C3214((*gp).map_id as i32);
    ext::mpJointListAdd(3);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).u.stadium.set_xC4_b0(0);
    (*gp).u.stadium.set_xC4_b1(0);
    (*gp).x10_flags.set_b5(1);
}

pub unsafe extern "C" fn grStadium_801D16D4(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grStadium_801D16D4");
    0
}

pub unsafe extern "C" fn stageGObj4_GObjProc(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj4_GObjProc");
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grStadium_801D16FC(_gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D16FC");
    ext::mpLib_80057BC0(3);
}

/// `static const Vec3 grPs_803B7F68`
pub static grPs_803B7F68: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

pub unsafe extern "C" fn grStadium_801D1720(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1720");
    let mut sp14: Vec3;
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::Ground_801C3214((*gp).map_id as i32);
    ext::mpJointListAdd(5);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).u.stadium9.set_xC4_b0(0);
    (*gp).u.stadium9.set_xC4_b1(0);
    sp14 = grPs_803B7F68;
    (*gp).u.stadium9.xC8 = ext::grLib_801C96F8(0x7546, 0x1E, &mut sp14);
    (*gp).x10_flags.set_b5(1);
}

pub unsafe extern "C" fn grStadium_801D17E0(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grStadium_801D17E0");
    0
}

pub unsafe extern "C" fn stageGObj6_GObjProc(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj6_GObjProc");
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grStadium_801D1808(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1808");
    let gp = (*gobj).user_data as *mut Ground;
    if !(*gp).u.stadium9.xC8.is_null() {
        ext::grLib_801C9834((*gp).u.stadium9.xC8);
    }
    ext::mpLib_80057BC0(5);
}

pub unsafe extern "C" fn grStadium_801D1840(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1840");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::Ground_801C3214((*gp).map_id as i32);
    ext::mpJointListAdd(1);
    ext::mpJointListAdd(2);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).u.stadium9.set_xC4_b0(0);
    (*gp).u.stadium9.set_xC4_b1(0);

    let temp_r3 = ext::Ground_801C3FA4(gobj, 0x13);
    if !temp_r3.is_null() {
        ext::grLib_801C9808(0x753F, 0x1E, temp_r3);
    }
    let temp_r3_2 = ext::Ground_801C3FA4(gobj, 0x14);
    if !temp_r3_2.is_null() {
        ext::grLib_801C9808(0x7549, 0x1E, temp_r3_2);
    }
    let temp_r3_3 = ext::Ground_801C3FA4(gobj, 0x15);
    if !temp_r3_3.is_null() {
        ext::grLib_801C9808(0x7549, 0x1E, temp_r3_3);
    }
    let temp_r3_4 = ext::Ground_801C3FA4(gobj, 0x16);
    if !temp_r3_4.is_null() {
        ext::grLib_801C9808(0x753F, 0x1E, temp_r3_4);
    }
    let temp_r3_5 = ext::Ground_801C3FA4(gobj, 0x17);
    if !temp_r3_5.is_null() {
        ext::grLib_801C9808(0x753F, 0x1E, temp_r3_5);
    }
    let temp_r3_6 = ext::Ground_801C3FA4(gobj, 0x18);
    if !temp_r3_6.is_null() {
        ext::grLib_801C9808(0x7549, 0x1E, temp_r3_6);
    }
    let temp_r3_7 = ext::Ground_801C3FA4(gobj, 0x19);
    if !temp_r3_7.is_null() {
        ext::grLib_801C9808(0x753F, 0x1E, temp_r3_7);
    }
    (*gp).x10_flags.set_b5(1);
}

pub unsafe extern "C" fn grStadium_801D19D0(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grStadium_801D19D0");
    0
}

pub unsafe extern "C" fn stageGObj3_GObjProc(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj3_GObjProc");
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grStadium_801D19F8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D19F8");
    let jobj = ext::Ground_801C3FA4(gobj, 0x12);
    if !jobj.is_null() {
        ext::grLib_801C9908(jobj);
    }
    ext::mpLib_80057BC0(1);
    ext::mpLib_80057BC0(2);
}

/// `static const Vec3 grPs_803B7F74`
pub static grPs_803B7F74: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

pub unsafe extern "C" fn grStadium_801D1A38(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1A38");
    let mut sp14: Vec3;

    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::Ground_801C3214((*gp).map_id as i32);
    ext::mpJointListAdd(7);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).u.stadium9.xCC_gobj = grStadium_801D10F8(7);
    (*gp).u.stadium9.xD0_gobj = grStadium_801D10F8(8);
    ext::mpLib_80057BC0(0);
    (*gp).u.stadium9.set_xC4_b0(0);
    (*gp).u.stadium9.set_xC4_b1(0);
    sp14 = grPs_803B7F74;

    (*gp).u.stadium9.xC8 = ext::grLib_801C96F8(0x7544, 0x1E, &mut sp14);
    (*gp).u.stadium9.xD4_jobj = ext::Ground_801C3FA4(gobj, 7);
    ext::HSD_JObjSetFlagsAll(jobj, 8);
    (*gp).x10_flags.set_b5(1);
}

pub unsafe extern "C" fn grStadium_801D1B40(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grStadium_801D1B40");
    0
}

pub unsafe extern "C" fn grStadium_801D1B48(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1B48");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let gp = (*gobj).user_data as *mut Ground;
    let scale = HSD_JObjGetScaleY(&*jobj);

    if !(*gp).u.stadium9.xCC_gobj.is_null() {
        let jobj = (*(*gp).u.stadium9.xCC_gobj).hsd_obj as *mut HSD_JObj;
        HSD_JObjSetScaleY(&mut *jobj, scale, &mut Dirty);
    }

    if !(*gp).u.stadium9.xD0_gobj.is_null() {
        let jobj = (*(*gp).u.stadium9.xD0_gobj).hsd_obj as *mut HSD_JObj;
        HSD_JObjSetScaleY(&mut *jobj, scale, &mut Dirty);
    }

    HSD_JObjAddRotationZ(&mut *(*gp).u.stadium9.xD4_jobj, -0.5f32 * 0.01745329252f32, &mut Dirty);

    if (*gp).u.stadium9.xC4_b0() != 0 {
        (*gp).u.stadium9.set_xC4_b0(0);
        ext::mpJointListAdd(0);
    }
    if (*gp).u.stadium9.xC4_b1() != 0 {
        (*gp).u.stadium9.set_xC4_b1(0);
        ext::mpLib_80057BC0(0);
    }

    ext::Ground_UpdateMapColl(gobj);
    ext::mpLib_8005667C(0);
}

pub unsafe extern "C" fn grStadium_801D1D84(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1D84");
    let gp = (*gobj).user_data as *mut Ground;

    if !(*gp).u.stadium9.xC8.is_null() {
        ext::grLib_801C9874((*gp).u.stadium9.xC8);
    }
    if !(*gp).u.stadium9.xCC_gobj.is_null() {
        ext::Ground_801C4A08((*gp).u.stadium9.xCC_gobj);
    }
    if !(*gp).u.stadium9.xD0_gobj.is_null() {
        ext::Ground_801C4A08((*gp).u.stadium9.xD0_gobj);
    }
    ext::mpLib_80057BC0(7);
}

pub unsafe extern "C" fn stageGObj7_8_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj7_8_OnInit");
    Ground_StartMapAnim(gobj);
}

pub unsafe extern "C" fn grStadium_801D1E10(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grStadium_801D1E10");
    0
}

pub unsafe extern "C" fn grStadium_801D1E18(_gobj: *mut HSD_GObj) { melee_core::ct!("grStadium_801D1E18");}

pub unsafe extern "C" fn grStadium_801D1E1C(_gobj: *mut HSD_GObj) { melee_core::ct!("grStadium_801D1E1C");}

#[inline(always)]
pub unsafe fn mul_vec(v: *mut Vec3, m: f32) {
    melee_core::ct!("mul_vec");
    (*v).x *= m;
    (*v).y *= m;
    (*v).z *= m;
}

pub unsafe extern "C" fn grStadium_801D1E20() {
    melee_core::ct!("grStadium_801D1E20");
    if ext::HSD_Randi(200) == 0 {
        let mut sp_c = Vec3 { x: 0.0, y: -100.0, z: -660.0 };
        let a = ext::HSD_Randi(2);
        let b = ext::HSD_Randi(200);

        sp_c.x += ((a.wrapping_mul(2).wrapping_sub(1)).wrapping_mul(b.wrapping_add(100))) as f32;

        mul_vec(&mut sp_c, ext::Ground_801C0498());

        ext::grLib_801C96F8(0x7530, 0x1E, &mut sp_c);
    }
}

/// `ImageDescWrapper`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ImageDescWrapper {
    pub desc: HSD_ImageDesc,
    /// `u16 flag : 1` (GC bit order, MSB first)
    pub bf_flag: u16,
    pub x1A: u16,
    pub x1C: u16,
}
impl ImageDescWrapper {
    #[inline] pub fn flag(&self) -> u16 { (self.bf_flag >> 15) & 1 }
    #[inline] pub fn set_flag(&mut self, v: u16) { self.bf_flag = (self.bf_flag & 0x7FFF) | ((v & 1) << 15) }
}
melee_core::gc_layout!(ImageDescWrapper, "ImageDescWrapper", size 0x20, align 4, [
    desc @ 0x0: 24 "HSD_ImageDesc", flag @ 0x18: 2 "u16" bits 0 + 1, x1A @ 0x1A: 2 "u16", x1C @ 0x1C: 2 "u16",
]);

/// `GET_WRAPPER(x)`
#[inline(always)]
unsafe fn GET_WRAPPER(g: *mut HSD_GObj) -> *mut ImageDescWrapper {
    (*g).user_data as *mut ImageDescWrapper
}

pub unsafe extern "C" fn grStadium_801D1EF8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D1EF8");
    let gp = (*gobj).user_data as *mut Ground;

    if (*gp).u.display.xF8_0() != 0 {
        let temp_r29: u32 = ((*gp).u.display.xE8 as i32 ^ (*gp).u.display.xE6 as i32) as u32;
        let xe6 = (*gp).u.display.xE6 as i32;
        let mut upd = |bit: u32, id: i32| {
            if temp_r29 & bit != 0 {
                let jobj = ext::Ground_801C3FA4(gobj, id);
                if !jobj.is_null() {
                    if xe6 & bit as i32 != 0 {
                        ext::HSD_JObjClearFlags(jobj, JOBJ_HIDDEN as u32);
                    } else {
                        ext::HSD_JObjSetFlags(jobj, JOBJ_HIDDEN as u32);
                    }
                }
            }
        };
        upd(1, 6);
        upd(2, 3);
        upd(8, 7);
        upd(0x10, 9);
        upd(4, 4);
        upd(0x20, 8);
        upd(0x80, 5);
        upd(0x40, 2);
        match (*gp).u.display.xE4 {
            0 | 1 | 9 | 10 | 11 | 12 | 13 | 14 | 17 => {
                let tobj = (*gp).u.display.xC8;
                if !tobj.is_null() {
                    if !(*gp).u.display.xD4.is_null() {
                        let wrapper = GET_WRAPPER((*gp).u.display.xD4);
                        (*tobj).imagedesc = &raw mut (*wrapper).desc;
                        grStadium_801D21E4(gobj, 0);
                    }
                }
            }
            7 => {
                let tobj = (*gp).u.display.xC8;
                if !tobj.is_null() {
                    if !(*gp).u.display.xD8.is_null() {
                        let wrapper = GET_WRAPPER((*gp).u.display.xD8);
                        (*tobj).imagedesc = &raw mut (*wrapper).desc;
                        grStadium_801D21E4(gobj, 1);
                    }
                }
            }
            8 => {
                let tobj = (*gp).u.display.xC8;
                if !tobj.is_null() {
                    if !(*gp).u.display.xDC.is_null() {
                        let wrapper = GET_WRAPPER((*gp).u.display.xDC);
                        (*tobj).imagedesc = &raw mut (*wrapper).desc;
                        grStadium_801D21E4(gobj, 1);
                    }
                }
            }
            _ => {}
        }
        (*gp).u.display.xE8 = (*gp).u.display.xE6;
    }
}

pub unsafe extern "C" fn grStadium_801D21E4(gobj: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("grStadium_801D21E4");
    let gp = (*gobj).user_data as *mut Ground;
    let mobj = (*gp).u.display.xCC;

    if mobj.is_null() {
        return;
    }

    if arg1 != 0 {
        ext::HSD_MObjClearFlags(mobj, 2);
        ext::HSD_MObjSetFlags(mobj, 1);
        ext::HSD_MObjSetDiffuseColor(mobj, (*yakumono_param).r, (*yakumono_param).g, (*yakumono_param).b);
    } else {
        ext::HSD_MObjSetFlags(mobj, 2);
        ext::HSD_MObjClearFlags(mobj, 1);
    }
    ext::HSD_MObjCompileTev(mobj);
}

pub unsafe extern "C" fn grStadium_801D2278(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D2278");
    let gp = (*gobj).user_data as *mut Ground;

    (*gp).u.display.xE4 = 1;
    (*gp).u.display.xE6 = 1;
    (*gp).u.display.xE8 = -1;
    (*gp).u.display.set_xF8_0(0);
    (*gp).u.display.set_xF8_1(0);
    (*gp).u.display.set_xF8_2(0);
    (*gp).u.display.xEA = -1;
    (*gp).u.display.xEE = 99;
    (*gp).u.display.xF2 = 0;
    (*gp).u.display.xD4 = grStadium_801D2BEC();
    (*gp).u.display.xD8 = grStadium_801D2D78();
    (*gp).u.display.xDC = grStadium_801D2E24();
    let wrapper = GET_WRAPPER((*gp).u.display.xDC);
    (*wrapper).x1A = 200;
    (*wrapper).x1C = 160;
    (*gp).u.display.xF4 = null_mut();
    grStadium_801D2528(gobj, 0, 0);
}

pub unsafe extern "C" fn grStadium_801D2344(g: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D2344");
    let gobj = g;
    let gp = (*gobj).user_data as *mut Ground;
    let temp_r0 = (*gp).u.display.xE4;
    match temp_r0 {
        0 => {
            grStadium_801D4194(gobj);
        }
        1 => {
            grStadium_801D39A0(gobj);
            let old = (*gp).u.display.xE0;
            (*gp).u.display.xE0 = old.wrapping_sub(1);
            if old < 0 {
                grStadium_801D2A60(gobj);
            }
        }
        9 => {
            grStadium_801D3A0C(gobj);
            let old = (*gp).u.display.xE0;
            (*gp).u.display.xE0 = old.wrapping_sub(1);
            if old < 0 {
                grStadium_801D2A60(gobj);
            }
        }
        14 => {
            grStadium_801D3BBC(gobj);
            let old = (*gp).u.display.xE0;
            (*gp).u.display.xE0 = old.wrapping_sub(1);
            if old < 0 {
                grStadium_801D2A60(gobj);
            }
        }
        10 | 11 | 12 | 13 => {
            grStadium_801D3F40(gobj, (*gp).u.display.xE4 as i32);
        }
        2 | 3 | 4 | 5 | 6 | 15 | 16 => {
            let old = (*gp).u.display.xE0;
            (*gp).u.display.xE0 = old.wrapping_sub(1);
            if old < 0 {
                grStadium_801D2A60(gobj);
            }
        }
        7 => {
            let old = (*gp).u.display.xE0;
            (*gp).u.display.xE0 = old.wrapping_sub(1);
            if old < 0 {
                grStadium_801D2A60(gobj);
                return;
            }
            let temp_r3_6 = GET_WRAPPER((*gp).u.display.xD8);
            (*temp_r3_6).set_flag(0);
        }
        8 => {
            let old = (*gp).u.display.xE0;
            (*gp).u.display.xE0 = old.wrapping_sub(1);
            if old < 0 || ext::Player_GetEntity((*gp).u.display.xEE as i32).is_null() || ext::Player_8003219C((*gp).u.display.xEE as i32) != 0 {
                grStadium_801D2A60(gobj);
                return;
            }
            let temp_r3_8 = GET_WRAPPER((*gp).u.display.xDC);
            (*temp_r3_8).set_flag(0);
            if (*gp).u.display.xDC.is_null()
                || ext::Player_GetEntity((*gp).u.display.xEE as i32).is_null()
                || ext::Player_8003219C((*gp).u.display.xEE as i32) != 0
                || grStadium_801D32D0(gobj) == 0
            {
                grStadium_801D2A60(gobj);
            }
        }
        _ => {}
    }
}

/// `TextWrapper`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TextWrapper {
    pub desc: HSD_ImageDesc,
    pub win_static_p: *mut HSD_Text,
    pub win_dynamic_p: *mut HSD_Text,
}
melee_core::gc_layout!(TextWrapper, "TextWrapper", size 0x20, align 4, [
    desc @ 0x0: 24 "HSD_ImageDesc", win_static_p @ 0x18: 4 "HSD_Text *", win_dynamic_p @ 0x1C: 4 "HSD_Text *",
]);

#[inline(always)]
pub unsafe fn randi_between_2(a: i32, b: i32) -> i32 {
    melee_core::ct!("randi_between_2");
    let mut result = b;
    if b > a {
        result = a.wrapping_add(randi(b.wrapping_sub(a)));
    } else if b < a {
        result = b.wrapping_add(randi(a.wrapping_sub(b)));
    }
    result
}

#[inline(always)]
pub unsafe fn randi_between(a: i32, b: i32) -> i32 {
    melee_core::ct!("randi_between");
    if b > a {
        return a.wrapping_add(randi(b.wrapping_sub(a)));
    } else if b < a {
        return b.wrapping_add(randi(a.wrapping_sub(b)));
    }
    b
}

pub unsafe extern "C" fn grStadium_801D2528(gobj: *mut HSD_GObj, mut arg1: i32, arg2: i32) {
    melee_core::ct!("grStadium_801D2528");
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;

    if ext::gm_8018841C() != 0 {
        match arg1 {
            1 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 => {
                arg1 = 7;
            }
            2 | 3 | 4 | 5 | 6 => {
                panic!("HSD_ASSERT(0x536, 0) grpstadium.c");
            }
            _ => {}
        }
    }
    (*gp).u.display.xEA = (*gp).u.display.xE4;
    (*gp).u.display.xE4 = arg1 as i16;
    match (*gp).u.display.xE4 {
        0 => {
            grStadium_801D4194(gobj);
            (*gp).u.display.xE6 = 0x40;
            (*gp).u.display.set_xF8_0(0);
            (*gp).u.display.set_xF8_1(0);
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Inactive as _;
            }
        }
        1 => {
            grStadium_801D39A0(gobj);
            (*gp).u.display.xE6 = 0x40;
            (*gp).u.display.xE0 = (*yp).x20 as i32;
            (*gp).u.display.set_xF8_0(0);
            (*gp).u.display.set_xF8_1(0);
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Inactive as _;
            }
        }
        9 => {
            grStadium_801D3A0C(gobj);
            (*gp).u.display.xE6 = 0x40;
            (*gp).u.display.xE0 = (*yp).x24 as i32;
            (*gp).u.display.set_xF8_0(0);
            (*gp).u.display.set_xF8_1(0);
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Inactive as _;
            }
        }
        14 => {
            grStadium_801D3BBC(gobj);
            (*gp).u.display.xE6 = 0x40;
            (*gp).u.display.xE0 = (*yp).x2C as i32;
            (*gp).u.display.set_xF8_0(0);
            (*gp).u.display.set_xF8_1(0);
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Inactive as _;
            }
        }
        10 | 11 | 12 | 13 => {
            grStadium_801D3F40(gobj, (*gp).u.display.xE4 as i32);
            (*gp).u.display.xE6 = 0x40;
            (*gp).u.display.set_xF8_0(0);
            (*gp).u.display.set_xF8_1(0);
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Inactive as _;
            }
        }
        15 => {
            (*gp).u.display.xE6 = 0x20;
            (*gp).u.display.xE0 = (*yp).x20 as i32;
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Inactive as _;
            }
        }
        16 => {
            (*gp).u.display.xE6 = 0x80;
            (*gp).u.display.xE0 = (*yp).x20 as i32;
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Inactive as _;
            }
        }
        7 => {
            (*gp).u.display.xE6 = 0x40;
            (*gp).u.display.xE0 = randi_between_2((*yp).x38 as i32, (*yp).x3C as i32);
            (*gp).u.display.set_xF8_0(0);
            (*gp).u.display.set_xF8_1(0);
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Inactive as _;
            }
        }
        8 => {
            (*gp).u.display.xE6 = 0x40;
            (*gp).u.display.xE0 = randi_between((*yp).x30 as i32, (*yp).x34 as i32);
            let n = (*gp).u.display.xEE;
            (*gp).u.display.xEE = n.wrapping_add(1);
            let mut i = 0;
            while (*gp).u.display.xEE != n {
                if (*gp).u.display.xEE >= 6 {
                    (*gp).u.display.xEE = 0;
                    i += 1;
                    if i > 2 {
                        (*gp).u.display.xEE = 0;
                        (*gp).u.display.xE0 = -1;
                        break;
                    }
                }
                if !ext::Player_GetEntity((*gp).u.display.xEE as i32).is_null() {
                    break;
                }
                (*gp).u.display.xEE = (*gp).u.display.xEE.wrapping_add(1);
            }
            (*gp).u.display.set_xF8_0(0);
            (*gp).u.display.set_xF8_1(0);
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Inactive as _;
            }
        }
        17 => {
            (*gp).u.display.xE6 = 0x40;
            let temp_r27_1 = (*(*gp).u.display.xD4).user_data as *mut TextWrapper;
            ext::HSD_SisLib_803A7664((*temp_r27_1).win_dynamic_p);
            ext::HSD_SisLib_803A6368((*temp_r27_1).win_static_p, 4);
            (*gp).u.display.set_xF8_0(0);
            (*gp).u.display.set_xF8_1(0);
        }
        2 => {
            (*gp).u.display.xE6 = 1;
            (*gp).u.display.xE0 = (*yp).x28 as i32;
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Active as _;
            }
        }
        3 => {
            (*gp).u.display.xE6 = 2;
            (*gp).u.display.xE0 = (*yp).x28 as i32;
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Active as _;
            }
        }
        4 => {
            (*gp).u.display.xE6 = 4;
            (*gp).u.display.xE0 = (*yp).x28 as i32;
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Active as _;
            }
        }
        5 => {
            (*gp).u.display.xE6 = 0x10;
            (*gp).u.display.xE0 = (*yp).x28 as i32;
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Active as _;
            }
        }
        6 => {
            (*gp).u.display.xE6 = 8;
            (*gp).u.display.xE0 = (*yp).x28 as i32;
            if !(*gp).u.display.xF4.is_null() {
                (*(*gp).u.display.xF4).state = CmSubjectState_Active as _;
            }
        }
        _ => {}
    }
    if arg2 != 0 {
        (*gp).u.display.xE0 = arg2;
    }
}

pub unsafe extern "C" fn grStadium_801D2A60(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D2A60");
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;
    let mut var_r4: i32;

    if (*gp).u.display.xF2 >= (*yp).x50 {
        (*gp).u.display.xF2 = 0;
        var_r4 = 0xE;
    } else {
        loop {
            let mut val: f32 = ext::HSD_Randf()
                * (((*yp).x48 as i32 + (*yp).x4A as i32 + (*yp).x4C as i32 + (*yp).x4E as i32) as f32);
            val -= (*yp).x48 as f32;
            if val < 0.0 {
                var_r4 = 8;
            } else {
                val -= (*yp).x4C as f32;
                if val < 0.0 {
                    var_r4 = 7;
                } else {
                    val -= (*yp).x4A as f32;
                    if val < 0.0 {
                        var_r4 = 1;
                    } else {
                        val -= (*yp).x4E as f32;
                        if val < 0.0 {
                            var_r4 = 15;
                        } else {
                            var_r4 = 1;
                        }
                    }
                }
            }
            if !(var_r4 == (*gp).u.display.xE4 as i32 || var_r4 == (*gp).u.display.xEA as i32) {
                break;
            }
        }
    }
    (*gp).u.display.xF2 = (*gp).u.display.xF2.wrapping_add(1);
    grStadium_801D2528(gobj, var_r4, 0);
}

/// `static HSD_WObjDesc grPs_803E14D4`
pub static mut grPs_803E14D4: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 1.0 }, robjdesc: null_mut() };
/// `static HSD_WObjDesc grPs_803E14E8`
pub static mut grPs_803E14E8: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, robjdesc: null_mut() };

/// `static HSD_CameraDescPerspective grPs_803E14FC`
pub static mut grPs_803E14FC: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
    class_name: null_mut(),
    flags: 0,
    projection_type: 1,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 0xFA, ymin: 0, ymax: 0xA0 },
    scissor: _Scissor { left: 0, right: 0xFA, top: 0, bottom: 0xA0 },
    eyepos: unsafe { &raw mut grPs_803E14D4 },
    interest: unsafe { &raw mut grPs_803E14E8 },
    roll: 0.0,
    up_vector: null_mut(),
    nnear: 0.1,
    ffar: 32768.0,
    fov: 30.0,
    aspect: 1.3636,
};

pub unsafe extern "C" fn grStadium_801D2BEC() -> *mut HSD_GObj {
    melee_core::ct!("grStadium_801D2BEC");
    let gobj = ext::GObj_Create(0x11, 0x13, 0);
    let cobj = ext::lb_80013B14(&raw mut grPs_803E14FC);
    ext::HSD_CObjSetOrtho(cobj, 0.0, -160.0, 0.0, 250.0);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj, Some(fn_801D2ED0), 1);
    (*gobj).gxlink_prios = 2;
    let text = ext::HSD_MemAlloc(core::mem::size_of::<TextWrapper>() as i32) as *mut TextWrapper;
    ext::GObj_InitUserData(gobj, 3, Some(ext::cb::HSD_Free), text as *mut c_void);
    ext::memzero(text as *mut c_void, 0x18);
    ext::lb_800121FC(&raw mut (*text).desc, 0xFA, 0xA0, 4, 0x7D2);
    let archive = ext::grDatFiles_GetArchive();
    ext::HSD_SisLib_803A611C(1, gobj, 9, 0xD, 0, 1, 0, 1);
    ext::HSD_SisLib_804D1124[1] = ext::HSD_ArchiveGetPublicAddress((*archive).unk0, b"SIS_GrPStadiumData\0".as_ptr() as *const i8) as *mut SIS;
    (*text).win_static_p = ext::HSD_SisLib_803A5ACC(1, 0, 0.0, 0.0, 0.0, 250.0, 160.0);
    (*text).win_dynamic_p = ext::HSD_SisLib_803A6754(1, 0);
    (*(*text).win_dynamic_p).pos_z = 0.0;
    let hsd_text = (*text).win_dynamic_p;
    (*hsd_text).box_size_x = 250.0;
    (*hsd_text).box_size_y = 160.0;
    gobj
}

pub unsafe extern "C" fn grStadium_801D2D78() -> *mut HSD_GObj {
    melee_core::ct!("grStadium_801D2D78");
    let temp_r3 = ext::GObj_Create(0x11, 0x12, 0);
    ext::GObj_SetupGXLinkMax(temp_r3, Some(grStadium_801D2FD0), 3);
    let wrapper = ext::HSD_MemAlloc(0x1C) as *mut ImageDescWrapper;
    ext::GObj_InitUserData(temp_r3, 3, Some(ext::cb::HSD_Free), wrapper as *mut c_void);
    ext::memzero(&raw mut (*wrapper).desc as *mut c_void, core::mem::size_of::<HSD_ImageDesc>() as i32);
    ext::lb_800121FC(&raw mut (*wrapper).desc, 0x280, 0x196, 4, 0x7D3);
    (*wrapper).set_flag(1);
    temp_r3
}

pub unsafe extern "C" fn grStadium_801D2E24() -> *mut HSD_GObj {
    melee_core::ct!("grStadium_801D2E24");
    let gobj = ext::GObj_Create(0x11, 0x12, 0);
    ext::GObj_SetupGXLinkMax(gobj, Some(grStadium_801D3084), 3);
    let wrapper = ext::HSD_MemAlloc(core::mem::size_of::<ImageDescWrapper>() as i32) as *mut ImageDescWrapper;
    ext::GObj_InitUserData(gobj, 3, Some(ext::cb::HSD_Free), wrapper as *mut c_void);
    ext::memzero(&raw mut (*wrapper).desc as *mut c_void, core::mem::size_of::<HSD_ImageDesc>() as i32);
    ext::lb_800121FC(&raw mut (*wrapper).desc, 0x7C, 0x50, 4, 0x7D4);
    (*wrapper).set_flag(0);
    gobj
}

pub unsafe extern "C" fn fn_801D2ED0(gobj: *mut HSD_GObj, _unused: i32) {
    melee_core::ct!("fn_801D2ED0");
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    let wrapper = GET_WRAPPER(gobj);
    if ext::HSD_CObjSetCurrent(cobj) != 0 {
        ext::HSD_SetEraseColor(0, 0, 0, 1);
        ext::HSD_CObjEraseScreen(cobj, 1, 0, 0);
        ext::HSD_FogSet(null_mut());
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_CObjEndCurrent();
    }
    ext::lb_800122C8(&raw mut (*wrapper).desc, 0, 0, 1);
    let vision_gobj = ext::Ground_GetMapGObj(PsType_Display as i32);
    if vision_gobj.is_null() {
        panic!("HSD_ASSERT(0x6D2, vision_gobj) grpstadium.c");
    }
    let gp2 = (*vision_gobj).user_data as *mut Ground;
    if gp2.is_null() {
        panic!("HSD_ASSERT(0x6D3, gp2) grpstadium.c");
    }
    (*gp2).u.display.set_xF8_0(1);
}

pub unsafe extern "C" fn grStadium_801D2FD0(gobj: *mut HSD_GObj, _unused: i32) {
    melee_core::ct!("grStadium_801D2FD0");
    let wrapper = GET_WRAPPER(gobj);
    let copy = wrapper;
    if (*wrapper).flag() == 0 {
        ext::lb_800122C8(&raw mut (*copy).desc, 0, 36, 0);
        (*wrapper).set_flag(1);

        let vision_gobj = ext::Ground_GetMapGObj(PsType_Display as i32);
        if vision_gobj.is_null() {
            panic!("HSD_ASSERT(0x6EA, vision_gobj) grpstadium.c");
        }
        let gp2 = (*vision_gobj).user_data as *mut Ground;
        if gp2.is_null() {
            panic!("HSD_ASSERT(0x6EB, gp2) grpstadium.c");
        }
        (*gp2).u.display.set_xF8_0(1);
    }
}

pub unsafe extern "C" fn grStadium_801D3084(gobj: *mut HSD_GObj, _unused: i32) {
    melee_core::ct!("grStadium_801D3084");
    let wrapper = GET_WRAPPER(gobj);
    let new_var = wrapper;
    if (*new_var).flag() == 0 {
        ext::lb_800122C8(&raw mut (*new_var).desc, (*new_var).x1A, (*new_var).x1C, 0);
        (*new_var).set_flag(1);
        let vision_gobj = ext::Ground_GetMapGObj(PsType_Display as i32);
        if vision_gobj.is_null() {
            panic!("HSD_ASSERT(0x703, vision_gobj) grpstadium.c");
        }
        let gp2 = (*vision_gobj).user_data as *mut Ground;
        if gp2.is_null() {
            panic!("HSD_ASSERT(0x704, gp2) grpstadium.c");
        }
        (*gp2).u.display.set_xF8_0(1);
    }
}

pub unsafe extern "C" fn grStadium_801D3138(gobj: *mut HSD_GObj, desc: *mut HSD_ImageDesc, arg2: *mut *mut HSD_MObj) -> *mut HSD_TObj {
    melee_core::ct!("grStadium_801D3138");
    let mut jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    while !jobj.is_null() {
        if union_type_dobj(jobj) {
            let mut dobj = (*jobj).u.dobj;
            while !dobj.is_null() {
                let mobj = (*dobj).mobj;
                if !mobj.is_null() {
                    let mut tobj = (*mobj).tobj;
                    while !tobj.is_null() {
                        if (*tobj).imagedesc == desc {
                            *arg2 = mobj;
                            return tobj;
                        }
                        tobj = (*tobj).next;
                    }
                }
                dobj = (*dobj).next;
            }
        }

        if (*jobj).flags & JOBJ_INSTANCE as u32 == 0 && !HSD_JObjGetChild(Some(&*jobj)).is_null() {
            jobj = HSD_JObjGetChild(Some(&*jobj));
            continue;
        }
        if !HSD_JObjGetNext(Some(&*jobj)).is_null() {
            jobj = HSD_JObjGetNext(Some(&*jobj));
            continue;
        }
        loop {
            if HSD_JObjGetParent(Some(&*jobj)).is_null() {
                jobj = null_mut();
                break;
            }
            if !HSD_JObjGetNext(Some(&*HSD_JObjGetParent(Some(&*jobj)))).is_null() {
                jobj = HSD_JObjGetNext(Some(&*HSD_JObjGetParent(Some(&*jobj))));
                break;
            }
            jobj = HSD_JObjGetParent(Some(&*jobj));
        }
    }
    null_mut()
}

pub unsafe extern "C" fn grStadium_801D32D0(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grStadium_801D32D0");
    let mut sp28 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp1c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let mut var_r28 = false;
    let gp = (*gobj).user_data as *mut Ground;

    let cobj = (*ext::Camera_80030A50()).hsd_obj as *mut HSD_CObj;
    if !cobj.is_null() {
        let player_gobj = ext::Player_GetEntity((*gp).u.display.xEE as i32);
        if !player_gobj.is_null() && ext::Player_8003219C((*gp).u.display.xEE as i32) == 0 {
            let wrapper = (*(*gp).u.display.xDC).user_data as *mut ImageDescWrapper;
            if !wrapper.is_null() {
                ext::ftLib_80086B90(player_gobj, &mut sp28);
                ext::lbVector_WorldToScreen(cobj, &sp28, &mut sp1c, 0);
                sp1c.x -= 62.0f32;
                if sp1c.x < (*cobj).viewport.xmin {
                    sp1c.x = (*cobj).viewport.xmin;
                    var_r28 = true;
                } else if sp1c.x + 124.0f32 > (*cobj).viewport.xmax {
                    var_r28 = true;
                    sp1c.x = (*cobj).viewport.xmax - 124.0f32;
                }
                sp1c.y -= 40.0f32;
                if sp1c.y < (*cobj).viewport.ymin {
                    sp1c.y = (*cobj).viewport.ymin;
                    var_r28 = true;
                } else if sp1c.y + 80.0f32 > (*cobj).viewport.ymax {
                    var_r28 = true;
                    sp1c.y = (*cobj).viewport.ymax - 80.0f32;
                }
                (*wrapper).x1A = ((f2i32(sp1c.x as f64) >> 1).wrapping_mul(2)) as u16;
                (*wrapper).x1C = ((f2i32(sp1c.y as f64) >> 1).wrapping_mul(2)) as u16;

                if var_r28 {
                    return 0;
                } else {
                    return 1;
                }
            }
        }
    }
    0
}

/// `static const GXColor grPs_804DAF3C`
pub static grPs_804DAF3C: _GXColor = _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };

/// `static u8 grPs_8049F040[0x100]`: "Half the size of a #TextGlyphTexture; the stage's I4 glyph scratch."
pub static mut grPs_8049F040: [u8; 0x100] = [0; 0x100];

pub unsafe extern "C" fn grStadium_801D3460(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D3460");
    let mut sp48: _GXColor;
    let mut sp38: [*const i8; 4] = [core::ptr::null(); 4];
    let mut var_r28: usize;

    let gp = (*gobj).user_data as *mut Ground;
    let gp2 = (*(*gp).u.display.xD4).user_data as *mut TextWrapper;
    sp48 = grPs_804DAF3C;
    if (*gp2).win_static_p.is_null() {
        panic!("HSD_ASSERT(0x780, gp2->win_static_p) grpstadium.c");
    }
    if (*gp2).win_dynamic_p.is_null() {
        panic!("HSD_ASSERT(0x781, gp2->win_dynamic_p) grpstadium.c");
    }
    ext::HSD_SisLib_803A7664((*gp2).win_dynamic_p);
    let temp_r29 = (*ext::Ground_GetMapGObj(2)).user_data as *mut Ground;

    (*ext::HSD_SisLib_804D1124[1].add(2)).textures = (&raw mut grPs_8049F040) as *mut TextGlyphTexture;

    ext::HSD_SisLib_803A6530(1, 5, 6);
    let temp_r0 = (*temp_r29).u.stadium.xDE;

    if temp_r0 == 3 {
        ext::HSD_SisLib_803A660C(1, 5, 9);
    } else if temp_r0 == 9 {
        ext::HSD_SisLib_803A660C(1, 5, 0xA);
    } else if temp_r0 == 6 {
        ext::HSD_SisLib_803A660C(1, 5, 0xB);
    } else if temp_r0 == 4 {
        ext::HSD_SisLib_803A660C(1, 5, 0xC);
    } else {
        ext::HSD_SisLib_803A660C(1, 5, 8);
    }

    ext::HSD_SisLib_803A660C(1, 5, 7);
    ext::HSD_SisLib_803A6368((*gp2).win_static_p, 5);
    (*(*gp2).win_dynamic_p).default_kerning = 0;
    if ext::lbLang_IsSavedLanguageJP() != 0 {
        let temp_r3 = (*gp2).win_dynamic_p;
        (*temp_r3).x34.x = 0.3125f32;
        (*temp_r3).x34.y = 0.625f32;
    } else {
        let temp_r3_2 = (*gp2).win_dynamic_p;
        (*temp_r3_2).x34.x = 0.26f32;
        (*temp_r3_2).x34.y = 0.625f32;
    }
    sp48.r = 0xC0;
    sp48.g = 0xC0;
    var_r28 = 0;
    sp48.b = 0xFF;
    (*(*gp2).win_dynamic_p).text_color = sp48;
    (*(*gp2).win_dynamic_p).default_alignment = 1;

    for i in 0..4i32 {
        if ext::Player_GetPlayerSlotType(i) != Gm_PKind_NA {
            let character_name = ext::gm_80160A60(i);
            sp38[var_r28] = character_name;
            if !character_name.is_null() {
                var_r28 += 1;
            }
        }
    }

    (*(*gp2).win_dynamic_p).default_fitting = 1;
    let d = (*gp2).win_dynamic_p;
    if var_r28 == 1 {
        ext::HSD_SisLib_803A6B98(d, 124.0, 70.0, sp38[0], 0, 0, 0);
    } else if var_r28 == 2 {
        ext::HSD_SisLib_803A6B98(d, 62.0, 70.0, sp38[0], 0, 0, 0);
        ext::HSD_SisLib_803A6B98(d, 186.0, 70.0, sp38[1], 0, 0, 0);
    } else if var_r28 == 3 {
        ext::HSD_SisLib_803A6B98(d, 62.0, 60.0, sp38[0], 0, 0, 0);
        ext::HSD_SisLib_803A6B98(d, 186.0, 60.0, sp38[1], 0, 0, 0);
        ext::HSD_SisLib_803A6B98(d, 124.0, 80.0, sp38[2], 0, 0, 0);
    } else if var_r28 == 4 {
        ext::HSD_SisLib_803A6B98(d, 62.0, 60.0, sp38[0], 0, 0, 0);
        ext::HSD_SisLib_803A6B98(d, 186.0, 60.0, sp38[1], 0, 0, 0);
        ext::HSD_SisLib_803A6B98(d, 62.0, 80.0, sp38[2], 0, 0, 0);
        ext::HSD_SisLib_803A6B98(d, 186.0, 80.0, sp38[3], 0, 0, 0);
    }

    (*(*gp2).win_dynamic_p).default_fitting = 0;
    let rules = ext::gm_GetStartMeleeRules();
    let temp_r4 = (*gp2).win_dynamic_p;
    (*temp_r4).x34.x = 0.625f32;
    (*temp_r4).x34.y = 0.625f32;

    if ((*rules).time_limit == 0) | ((*rules).timer_enabled() == 0) {
        ext::HSD_SisLib_803A6B98((*gp2).win_dynamic_p, 125.0, 120.0, b"00:00 00\0".as_ptr() as *const i8, 0, 0, 0);
        return;
    }
    let temp_r29_2 = ext::gm_8016AEEC();
    let a = (temp_r29_2 / 60) as usize;
    let b = (temp_r29_2 % 60) as usize;
    let c = ext::gm_8016AF0C() as i32 as usize;
    ext::HSD_SisLib_803A6B98((*gp2).win_dynamic_p, 125.0, 120.0, b"%02d:%02d %02d\0".as_ptr() as *const i8, a, b, c);
}

/// `const GXColor grPs_804DAF60`
pub static grPs_804DAF60: _GXColor = _GXColor { r: 0xC0, g: 0xC0, b: 0xFF, a: 0xFF };

pub unsafe extern "C" fn grStadium_801D384C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D384C");
    let gp = (*gobj).user_data as *mut Ground;

    let temp_r3 = (*gp).u.display.xD4;
    let gp2 = (*temp_r3).user_data as *mut TextWrapper;
    let sp2c = grPs_804DAF60;
    let temp_r29 = ext::gm_801694A0(temp_r3);
    if (*gp2).win_static_p.is_null() {
        panic!("HSD_ASSERT(0x812, gp2->win_static_p) grpstadium.c");
    }
    if (*gp2).win_dynamic_p.is_null() {
        panic!("HSD_ASSERT(0x813, gp2->win_dynamic_p) grpstadium.c");
    }
    ext::HSD_SisLib_803A7664((*gp2).win_dynamic_p);
    if ext::lbLang_IsSavedLanguageJP() != 0 {
        ext::HSD_SisLib_803A6368((*gp2).win_static_p, 0xD);
        (*(*gp2).win_dynamic_p).default_kerning = 0;
        let temp_r3_2 = (*gp2).win_dynamic_p;
        (*temp_r3_2).x34.x = 1.5;
        (*temp_r3_2).x34.y = 1.5;
        let temp_r6 = (*gp2).win_dynamic_p;
        (*temp_r6).text_color = sp2c;
        (*(*gp2).win_dynamic_p).default_alignment = 1;
        ext::HSD_SisLib_803A6B98((*gp2).win_dynamic_p, 125.0, 52.0, b"%d\0".as_ptr() as *const i8, temp_r29 as u32 as usize, 0, 0);
    } else {
        ext::HSD_SisLib_803A6368((*gp2).win_static_p, 0xD);
        (*(*gp2).win_dynamic_p).default_kerning = 0;
        let temp_r3_3 = (*gp2).win_dynamic_p;
        (*temp_r3_3).x34.x = 1.5;
        (*temp_r3_3).x34.y = 1.5;
        (*(*gp2).win_dynamic_p).text_color = sp2c;
        ext::HSD_SisLib_803A6B98((*gp2).win_dynamic_p, -3.0, 52.0, b"%3d\0".as_ptr() as *const i8, temp_r29 as u32 as usize, 0, 0);
    }
}

pub unsafe extern "C" fn grStadium_801D39A0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D39A0");
    let gp = (*gobj).user_data as *mut Ground;
    if (*gp).u.display.xE4 == 1 {
        if ext::gm_8016B184() != 0 || (*gp).u.display.xF8_2() != 0 {
            grStadium_801D384C(gobj);
        } else {
            grStadium_801D3460(gobj);
        }
    }
}

/// `const GXColor grPs_804DAF70`
pub static grPs_804DAF70: _GXColor = _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };

pub unsafe extern "C" fn grStadium_801D3A0C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D3A0C");
    let mut sp24: _GXColor;
    let gp = (*gobj).user_data as *mut Ground;
    let tmp = (*gp).u.display.xD4;
    let gp2 = (*tmp).user_data as *mut TextWrapper;

    sp24 = grPs_804DAF70;

    if (*gp2).win_static_p.is_null() {
        panic!("HSD_ASSERT(0x848, gp2->win_static_p) grpstadium.c");
    }
    if (*gp2).win_dynamic_p.is_null() {
        panic!("HSD_ASSERT(0x849, gp2->win_dynamic_p) grpstadium.c");
    }
    ext::HSD_SisLib_803A7664((*gp2).win_dynamic_p);
    ext::HSD_SisLib_803A6368((*gp2).win_static_p, 3);
    (*(*gp2).win_dynamic_p).default_kerning = 1;
    sp24.r = 0xC0;
    sp24.g = 0xC0;
    sp24.b = 0xFF;
    (*(*gp2).win_dynamic_p).text_color = sp24;
    (*(*gp2).win_dynamic_p).default_alignment = 1;
    if (*gp).u.display.xF0 as i32 as u32 == Gm_PKind_Human {
        let temp_r3 = (*gp2).win_dynamic_p;
        (*temp_r3).x34.x = 0.625f32;
        (*temp_r3).x34.y = 1.5f32;
        ext::HSD_SisLib_803A6B98(
            (*gp2).win_dynamic_p,
            125.0,
            56.0,
            "Ｐｌａｙｅｒ %d Ｄｅｆｅａｔｅｄ\0".as_ptr() as *const i8,
            ((*gp).u.display.xEE as i32 + 1) as u32 as usize,
            0,
            0,
        );
    } else {
        let temp_r3 = (*gp2).win_dynamic_p;
        (*temp_r3).x34.x = 0.42f32;
        (*temp_r3).x34.y = 1.2f32;
        ext::HSD_SisLib_803A6B98((*gp2).win_dynamic_p, 125.0, 56.0, b"Computer Player Defeated\0".as_ptr() as *const i8, 0, 0, 0);
    }
}

pub unsafe extern "C" fn grStadium_801D3B4C(arg0: i32, slot_type: i32) {
    melee_core::ct!("grStadium_801D3B4C");
    if ext::stage_info.grkind == Gr_Kind_PStadium as _ {
        let gobj = ext::Ground_GetMapGObj(PsType_Display as i32);
        let gp = (*gobj).user_data as *mut Ground;
        (*gp).u.display.xEE = arg0 as i16;
        (*gp).u.display.xF0 = slot_type as i16;
        grStadium_801D2528(gobj, 9, 0);
    }
}

/// `StadiumAlphaPresets`: four `U8Vec4`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StadiumAlphaPresets {
    pub entries: [[u8; 4]; 4],
}

/// `const StadiumAlphaPresets grPs_803B7F8C`
pub static grPs_803B7F8C: StadiumAlphaPresets = StadiumAlphaPresets { entries: [[0, 0, 0, 0x50], [0, 0, 0, 0x40], [0, 0, 0, 0x30], [0, 0, 0, 0x20]] };

#[inline(always)]
pub unsafe fn grStadium_ScaleColor(value: u8) -> u8 {
    melee_core::ct!("grStadium_ScaleColor");
    let mut scaled = 1.25f32 * value as f32;
    if scaled > 255.0 {
        scaled = 255.0;
    }
    f2i32(scaled as f64) as u8
}

pub unsafe extern "C" fn grStadium_801D3BBC(arg0: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D3BBC");
    // union { StadiumColor entries[5]; struct { StadiumColor head; StadiumAlphaPresets presets; } s; }
    let mut colors: [[u8; 4]; 5] = [[0; 4]; 5];
    let mut player_num: i32 = 0;
    let ground = (*arg0).user_data as *mut Ground;
    let text_gobj = (*ground).u.display.xD4;
    let gp2 = (*text_gobj).user_data as *mut TextWrapper;
    if (*gp2).win_static_p.is_null() {
        panic!("HSD_ASSERT(0x884, gp2->win_static_p) grpstadium.c");
    }
    if (*gp2).win_dynamic_p.is_null() {
        panic!("HSD_ASSERT(0x885, gp2->win_dynamic_p) grpstadium.c");
    }
    for player_index in 0..6i32 {
        if ext::Player_GetPlayerSlotType(player_index) != Gm_PKind_NA && ext::Player_8003219C(player_index) == 0 {
            let player_gobj = ext::Player_GetEntity(player_index);
            if !player_gobj.is_null() && ext::gm_8016C6C0(player_gobj) == 0 {
                player_num += 1;
            }
        }
    }
    ext::HSD_SisLib_803A7664((*gp2).win_dynamic_p);

    if player_num == 1 {
        ext::HSD_SisLib_803A6368((*gp2).win_static_p, 0xE);
    } else if player_num == 2 {
        ext::HSD_SisLib_803A6368((*gp2).win_static_p, 0xF);
    } else if player_num == 3 {
        ext::HSD_SisLib_803A6368((*gp2).win_static_p, 0x10);
    } else if player_num == 4 {
        ext::HSD_SisLib_803A6368((*gp2).win_static_p, 0x11);
    } else {
        ext::HSD_SisLib_803A6368((*gp2).win_static_p, 3);
        return;
    }

    (*(*gp2).win_dynamic_p).default_kerning = 0;
    (*(*gp2).win_dynamic_p).default_alignment = 1;
    (*(*gp2).win_dynamic_p).default_fitting = 1;
    if ext::lbLang_IsSavedLanguageJP() != 0 {
        let japanese_text = (*gp2).win_dynamic_p;
        (*japanese_text).x34.x = 1.0;
        (*japanese_text).x34.y = 1.0;
    } else {
        let dynamic_text = (*gp2).win_dynamic_p;
        (*dynamic_text).x34.x = 0.8f32;
        (*dynamic_text).x34.y = 1.0;
        (*(*gp2).win_dynamic_p).default_kerning = 1;
    }
    colors[1] = grPs_803B7F8C.entries[0];
    colors[2] = grPs_803B7F8C.entries[1];
    colors[3] = grPs_803B7F8C.entries[2];
    colors[4] = grPs_803B7F8C.entries[3];
    // `entries[n].packed` is a u32 over the four bytes: big-endian on the GameCube
    let e = colors[player_num as usize];
    let vertical_offset: i32 = u32::from_be_bytes(e) as i32;
    let mut text_offset: i32 = 0;
    player_num = 0;
    while player_num < 6 {
        if ext::Player_GetPlayerSlotType(player_num) != Gm_PKind_NA && ext::Player_8003219C(player_num) == 0 {
            let current_player_gobj = ext::Player_GetEntity(player_num);
            if !current_player_gobj.is_null() && ext::gm_8016C6C0(current_player_gobj) == 0 {
                let c = ext::gm_80160968(ext::gm_80160854(
                    player_num as u8,
                    ext::Player_GetTeam(player_num) as u8,
                    ext::gm_8016B168() as u8,
                    ext::Player_GetPlayerSlotType(player_num) as u8,
                ));
                colors[0] = [c.r, c.g, c.b, c.a];
                colors[0][0] = grStadium_ScaleColor(colors[0][0]);
                colors[0][1] = grStadium_ScaleColor(colors[0][1]);
                colors[0][2] = grStadium_ScaleColor(colors[0][2]);
                colors[0][3] = 0xFF;
                (*(*gp2).win_dynamic_p).text_color = _GXColor { r: colors[0][0], g: colors[0][1], b: colors[0][2], a: colors[0][3] };
                let character_name = ext::gm_80160A60(player_num);
                if !character_name.is_null() {
                    ext::HSD_SisLib_803A6B98(
                        (*gp2).win_dynamic_p,
                        125.0,
                        vertical_offset.wrapping_add(text_offset) as f32,
                        b"%s\0".as_ptr() as *const i8,
                        character_name as usize,
                        0,
                        0,
                    );
                }
                text_offset = text_offset.wrapping_add(0x20);
            }
        }
        player_num += 1;
    }
}

pub unsafe extern "C" fn grStadium_801D3F40(arg0: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("grStadium_801D3F40");
    let gp = (*arg0).user_data as *mut Ground;
    let text_gobj = (*gp).u.display.xD4;
    let gp2 = (*text_gobj).user_data as *mut TextWrapper;

    if (*gp2).win_static_p.is_null() {
        panic!("HSD_ASSERT(0x8E8, gp2->win_static_p) grpstadium.c");
    }
    if (*gp2).win_dynamic_p.is_null() {
        panic!("HSD_ASSERT(0x8E9, gp2->win_dynamic_p) grpstadium.c");
    }
    ext::HSD_SisLib_803A7664((*gp2).win_dynamic_p);
    match arg1 {
        10 => ext::HSD_SisLib_803A6368((*gp2).win_static_p, 0x12),
        11 => ext::HSD_SisLib_803A6368((*gp2).win_static_p, 0x13),
        12 => ext::HSD_SisLib_803A6368((*gp2).win_static_p, 0x14),
        13 => ext::HSD_SisLib_803A6368((*gp2).win_static_p, 0x15),
        _ => ext::HSD_SisLib_803A6368((*gp2).win_static_p, 3),
    }
}

pub unsafe extern "C" fn grStadium_801D4040() {
    melee_core::ct!("grStadium_801D4040");
    if ext::stage_info.grkind == Gr_Kind_PStadium as _ {
        grStadium_801D2528(ext::Ground_GetMapGObj(PsType_Display as i32), 0xB, 0);
    }
}

pub unsafe extern "C" fn grStadium_801D4084() {
    melee_core::ct!("grStadium_801D4084");
    if ext::stage_info.grkind == Gr_Kind_PStadium as _ {
        grStadium_801D2528(ext::Ground_GetMapGObj(PsType_Display as i32), 0xA, 0);
    }
}

pub unsafe extern "C" fn grStadium_801D40C8() {
    melee_core::ct!("grStadium_801D40C8");
    if ext::stage_info.grkind == Gr_Kind_PStadium as _ {
        grStadium_801D2528(ext::Ground_GetMapGObj(PsType_Display as i32), 0xC, 0);
    }
}

pub unsafe extern "C" fn grStadium_801D410C() {
    melee_core::ct!("grStadium_801D410C");
    if ext::stage_info.grkind == Gr_Kind_PStadium as _ {
        grStadium_801D2528(ext::Ground_GetMapGObj(PsType_Display as i32), 0xD, 0);
    }
}

pub unsafe extern "C" fn grStadium_801D4150() {
    melee_core::ct!("grStadium_801D4150");
    if ext::stage_info.grkind == Gr_Kind_PStadium as _ {
        grStadium_801D2528(ext::Ground_GetMapGObj(PsType_Display as i32), 1, 0);
    }
}

pub unsafe extern "C" fn grStadium_801D4194(arg0: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D4194");
    let gp = (*arg0).user_data as *mut Ground;
    let text_gobj = (*gp).u.display.xD4;
    let gp2 = (*text_gobj).user_data as *mut TextWrapper;

    if (*gp2).win_static_p.is_null() {
        panic!("HSD_ASSERT(0x950, gp2->win_static_p) grpstadium.c");
    }
    if (*gp2).win_dynamic_p.is_null() {
        panic!("HSD_ASSERT(0x951, gp2->win_dynamic_p) grpstadium.c");
    }
    ext::HSD_SisLib_803A7664((*gp2).win_dynamic_p);
    ext::HSD_SisLib_803A6368((*gp2).win_static_p, 3);
}

/// HSD_DevComCallback
pub unsafe extern "C" fn fn_801D4220(_dcreq: i32, _args: u32, _buf: *mut c_void, cancelflag: i32) {
    melee_core::ct!("fn_801D4220");
    if cancelflag != 0 {
        panic!("HSD_ASSERT(0x969, !cancelflag) grpstadium.c");
    }
    let map_gobj = ext::Ground_GetMapGObj(2);
    if map_gobj.is_null() {
        panic!("HSD_ASSERT(0x96A, map_gobj) grpstadium.c");
    }
    let gp = (*map_gobj).user_data as *mut Ground;
    if gp.is_null() {
        panic!("HSD_ASSERT(0x96B, gp) grpstadium.c");
    }
    (*gp).u.stadium.set_xC4_b1(0);
}

pub unsafe extern "C" fn grStadium_801D42B8() -> i32 {
    melee_core::ct!("grStadium_801D42B8");
    let map_gobj = ext::Ground_GetMapGObj(2);
    if map_gobj.is_null() {
        panic!("HSD_ASSERT(0x978, map_gobj) grpstadium.c");
    }
    let gp = grStadium_801D4354(map_gobj);
    if gp.is_null() {
        panic!("HSD_ASSERT(0x979, gp) grpstadium.c");
    }
    if (*gp).u.stadium.xC4_b1() != 0 {
        return 0;
    }
    (*gp).u.stadium.xD0 = ext::grDatFiles_801C6478((*gp).u.stadium.xCC as *mut c_void, (*gp).u.stadium.xC8 as i32);
    1
}

pub unsafe extern "C" fn grStadium_801D4354(gobj: *mut HSD_GObj) -> *mut Ground {
    melee_core::ct!("grStadium_801D4354");
    (*gobj).user_data as *mut Ground
}

/// `char* datfiles[]`
pub static mut datfiles: [*const i8; 4] = [
    b"GrPs1.dat\0".as_ptr() as *const i8,
    b"GrPs2.dat\0".as_ptr() as *const i8,
    b"GrPs3.dat\0".as_ptr() as *const i8,
    b"GrPs4.dat\0".as_ptr() as *const i8,
];

/// `struct VecScalar`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VecScalar {
    pub vec: Vec3,
    pub scalar: f32,
}
melee_core::gc_layout!(VecScalar, "VecScalar", size 0x10, align 4, [vec @ 0x0: 12 "Vec3", scalar @ 0xC: 4 "float"]);

const fn vs(x: f32, y: f32, z: f32, s: f32) -> VecScalar {
    VecScalar { vec: Vec3 { x, y, z }, scalar: s }
}

/// `static struct VecScalar lbl_803E1630[]`
pub static mut lbl_803E1630: [VecScalar; 9] = [
    vs(-50.0, 5.0, 50.0, 100.0),
    vs(50.0, 5.0, 50.0, 28.28),
    vs(70.0, 5.0, 30.0, 53.85),
    vs(70.0, 5.0, -30.0, 5.0),
    vs(50.0, 5.0, -50.0, 100.0),
    vs(-50.0, 5.0, -50.0, 28.28),
    vs(-70.0, 5.0, -30.0, 60.0),
    vs(-70.0, 5.0, 30.0, 28.28),
    vs(-50.0, 5.0, 50.0, 0.0),
];

pub unsafe extern "C" fn grStadium_801D435C(arg0: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D435C");
    let gp = (*arg0).user_data as *mut Ground;
    let mut sp_c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let tbl = &raw mut lbl_803E1630 as *mut VecScalar;

    let part0 = (*gp).u.stadium.xD4;
    if part0 == 0.0 {
        let mut cur = tbl;
        for _i in 0..9 {
            let part = (*cur).scalar;
            cur = cur.add(1);
            let total = (*gp).u.stadium.xD4;
            (*gp).u.stadium.xD4 = total - -part;
        }
    }
    for _var_r28 in 0..5 {
        let mut var_r4: usize = 0;
        let mut var_f31 = (*gp).u.stadium.xD4 * ext::HSD_Randf();

        while var_f31 > (*tbl.add(var_r4)).scalar && var_r4 < 8 {
            var_f31 -= (*tbl.add(var_r4)).scalar;
            var_r4 += 1;
        }
        ext::lbVector_Diff(&raw mut (*tbl.add(var_r4 + 1)).vec, &raw mut (*tbl.add(var_r4)).vec, &mut sp_c);

        let temp_f1_2 = var_f31 / (*tbl.add(var_r4)).scalar;
        sp_c.x *= temp_f1_2;
        sp_c.y *= temp_f1_2;
        sp_c.z *= temp_f1_2;
        ext::lbVector_Add(&mut sp_c, &raw mut (*tbl.add(var_r4)).vec);
        let temp_f1_3 = ext::Ground_801C0498();
        sp_c.x *= temp_f1_3;
        sp_c.y *= temp_f1_3;
        sp_c.z *= temp_f1_3;
        ext::grLib_801C96F8(0x7548, 0x1E, &mut sp_c);
    }
}

pub unsafe extern "C" fn grStadium_801D4548(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStadium_801D4548");
    let temp_r31 = (*gobj).user_data as *mut Ground;
    let temp_f31 = ext::Ground_801C0498();
    let yp = yakumono_param;
    if ext::gm_8018841C() != 0 {
        return;
    }

    if ext::Stage_80225194() == 0xF0 {
        let temp_r3 = ext::Ground_GetMapGObj(PsType_Display as i32);
        if !temp_r3.is_null() {
            let temp_r3_2 = (*temp_r3).user_data as *mut Ground;
            if !temp_r3_2.is_null() {
                if !(*temp_r3_2).u.display.xF4.is_null() {
                    (*(*temp_r3_2).u.display.xF4).state = CmSubjectState_Active as _;
                }
            }
        }
        return;
    }

    match (*temp_r31).u.stadium.xDC {
        0 => {
            let temp_r3_4 = (*temp_r31).u.stadium.xD8;
            (*temp_r31).u.stadium.xD8 = temp_r3_4.wrapping_sub(1);
            // TARGET_PC: Slippi's Frozen PS skips the whole transformation decision
            if ext::Slippi_FrozenStadium() != 0 {
                return;
            }
            if temp_r3_4 < 0 {
                let var_r4: i32;
                if (*temp_r31).u.stadium.xDE == 5 {
                    let sp60: [i32; 4] = [3, 4, 6, 9];
                    let mut v;
                    loop {
                        let idx = ext::HSD_Randi(4);
                        v = sp60[idx as usize];
                        if (*temp_r31).u.stadium.xE2 as i32 != v {
                            break;
                        }
                    }
                    var_r4 = v;
                } else {
                    var_r4 = 5;
                }
                (*temp_r31).u.stadium.xE2 = (*temp_r31).u.stadium.xE0;
                (*temp_r31).u.stadium.xE0 = (*temp_r31).u.stadium.xDE;
                (*temp_r31).u.stadium.xDE = var_r4 as i16;

                let var_r29: usize;
                if var_r4 == 5 {
                    (*temp_r31).u.stadium.xDC = 2;
                    return;
                } else if var_r4 == 3 {
                    var_r29 = 0;
                } else if var_r4 == 4 {
                    var_r29 = 1;
                } else if var_r4 == 9 {
                    var_r29 = 2;
                } else if var_r4 == 6 {
                    var_r29 = 3;
                } else {
                    panic!("HSD_ASSERT(0xA44, 0) grpstadium.c");
                }
                ext::grAnime_801C65B0((*temp_r31).u.stadium.xD0);
                (*temp_r31).u.stadium.xD0 = null_mut();
                let map_gobj = ext::Ground_GetMapGObj(2);
                if map_gobj.is_null() {
                    panic!("HSD_ASSERT(0x99A, map_gobj) grpstadium.c");
                }
                let gp = (*map_gobj).user_data as *mut Ground;
                if gp.is_null() {
                    panic!("HSD_ASSERT(0x99B, gp) grpstadium.c");
                }
                (*gp).u.stadium.set_xC4_b1(1);
                ext::lbFile_80016580(datfiles[var_r29], (*gp).u.stadium.xCC as *mut c_void, &raw mut (*gp).u.stadium.xC8 as *mut u32, Some(fn_801D4220), 0);
                (*temp_r31).u.stadium.xDC = 1;
                return;
            }
        }
        1 => {
            if grStadium_801D42B8() != 0 {
                (*temp_r31).u.stadium.xDC = 2;
                return;
            }
        }
        2 => {
            (*temp_r31).u.stadium.xD8 = 0;
            let temp_r3_6 = ext::Ground_GetMapGObj(1);
            if !temp_r3_6.is_null() {
                let temp_r0_2 = (*temp_r31).u.stadium.xDE;

                let var_r29: i32;
                if temp_r0_2 == 3 {
                    var_r29 = 3;
                } else if temp_r0_2 == 4 {
                    var_r29 = 4;
                } else if temp_r0_2 == 9 {
                    var_r29 = 5;
                } else if temp_r0_2 == 6 {
                    var_r29 = 6;
                } else if temp_r0_2 == 5 {
                    var_r29 = 2;
                } else {
                    panic!("HSD_ASSERT(0xA67, 0) grpstadium.c");
                }

                grStadium_801D2528(temp_r3_6, var_r29, 0);
            }
            (*temp_r31).u.stadium.xDC = 3;
            return;
        }
        3 => {
            let old = (*temp_r31).u.stadium.xD8;
            (*temp_r31).u.stadium.xD8 = old.wrapping_add(1);
            if old > (*yp).x10 {
                let temp_r6 = (*(*temp_r31).u.stadium.xE4).user_data as *mut Ground;
                (*temp_r6).u.stadium.set_xC4_b1(1);
                (*temp_r31).u.stadium.xDC = 4;
                (*temp_r31).u.stadium.xD8 = 0;
                ext::Ground_801C53EC(0x75300);
                ext::Ground_801C53EC(0x75301);
                return;
            }
        }
        4 => {
            let temp_r27_2 = (*(*temp_r31).u.stadium.xE4).hsd_obj as *mut HSD_JObj;
            let temp_r30 = temp_r27_2;
            let mut temp_f29 = HSD_JObjGetScaleY(&*temp_r30);
            temp_f29 = fmadds(-temp_f31, 0.95f32 / (*yp).x14 as f32, temp_f29);
            let temp_f30 = 0.05f32 * temp_f31;
            if temp_f29 > temp_f30 {
                HSD_JObjSetScaleY(&mut *temp_r30, temp_f29, &mut Dirty);
            } else {
                HSD_JObjSetScaleY(&mut *temp_r30, 0.05f32, &mut Dirty);
                let temp_r4_4 = (*temp_r31).u.stadium.xD8;
                (*temp_r31).u.stadium.xD8 = temp_r4_4.wrapping_add(1);
                if temp_r4_4 > (*yp).x18 {
                    ext::grAnime_801C7A04((*temp_r31).u.stadium.xE4, 0, 7, 0.0);
                    let temp_r3_7 = grStadium_801D10F8((*temp_r31).u.stadium.xDE as i32 as u32);
                    let temp_r27_3 = (*temp_r3_7).hsd_obj as *mut HSD_JObj;
                    HSD_JObjSetScaleY(&mut *temp_r27_3, temp_f30, &mut Dirty);
                    HSD_JObjSetTranslateY(&mut *temp_r27_3, -10.0, &mut Dirty);
                    (*temp_r31).u.stadium.xE8 = temp_r3_7;
                    (*temp_r31).u.stadium.xD8 = 0;
                    (*temp_r31).u.stadium.xDC = 5;
                    ext::mpLib_80057528(0x55);
                    ext::mpLib_80057528(0x6F);
                }
            }
            grStadium_801D435C(gobj);
            ext::Camera_RequestQuake(QuakeKind_Loop as u32, null_mut());
            return;
        }
        5 => {
            (*temp_r31).u.stadium.xD8 = (*temp_r31).u.stadium.xD8.wrapping_add(1);
            let temp_r0_3 = (*temp_r31).u.stadium.xD8;
            if temp_r0_3 <= (*yp).x14 {
                let temp_r30 = (*(*temp_r31).u.stadium.xE8).hsd_obj as *mut HSD_JObj;
                let tmp: f32 = 0.95f32 * temp_r0_3 as f32 / (*yp).x14 as f32 + 0.05f32;
                HSD_JObjSetScaleY(&mut *temp_r30, tmp * temp_f31, &mut Dirty);
                let temp_r4_8 = (*temp_r31).u.stadium.xD8;
                let temp_r0_4 = (*yp).x14 / 2;
                let var_f29: f32 = if temp_r4_8 < temp_r0_4 {
                    -10.0f32 * temp_f31 * (1.0f32 - (temp_r4_8 as f32 / temp_r0_4 as f32))
                } else {
                    0.0
                };
                HSD_JObjSetTranslateY(&mut *temp_r30, var_f29, &mut Dirty);
                let temp_r27_4 = (*(*temp_r31).u.stadium.xE4).hsd_obj as *mut HSD_JObj;
                let temp_r5_2 = (*yp).x14;
                let temp_r3_8 = (*temp_r31).u.stadium.xD8;
                let temp_r0_5 = temp_r5_2 / 2;
                let var_f30: f32 = if temp_r3_8 > temp_r0_5 {
                    -10.0f32 * temp_f31 * (1.0f32 - (temp_r5_2.wrapping_sub(temp_r3_8) as f32 / temp_r5_2.wrapping_sub(temp_r0_5) as f32))
                } else {
                    0.0
                };
                HSD_JObjSetTranslateY(&mut *temp_r27_4, var_f30, &mut Dirty);
            } else {
                let temp_r27_5 = (*(*temp_r31).u.stadium.xE8).hsd_obj as *mut HSD_JObj;
                HSD_JObjSetScaleY(&mut *temp_r27_5, temp_f31, &mut Dirty);
                HSD_JObjSetTranslateY(&mut *temp_r27_5, 0.0, &mut Dirty);
                (*temp_r31).u.stadium.xDC = 6;
                ext::Ground_801C4A08((*temp_r31).u.stadium.xE4);
                (*temp_r31).u.stadium.xE4 = (*temp_r31).u.stadium.xE8;
                (*temp_r31).u.stadium.xE8 = null_mut();
            }
            grStadium_801D435C(gobj);
            ext::Camera_RequestQuake(QuakeKind_Loop as u32, null_mut());
            return;
        }
        6 => {
            let temp_r3_9 = (*(*temp_r31).u.stadium.xE4).user_data as *mut Ground;
            (*temp_r3_9).u.stadium.set_xC4_b0(1);
            ext::mpLib_80058560();
            if (*temp_r31).u.stadium.xDE == 5 {
                (*temp_r31).u.stadium.xD8 = randi_between_2((*yp).x0, (*yp).x4);
                ext::grAnime_801C65B0((*temp_r31).u.stadium.xD0);
                (*temp_r31).u.stadium.xD0 = null_mut();
                ext::mpLib_800575B0(0x55);
                ext::mpLib_800575B0(0x6F);
            } else {
                (*temp_r31).u.stadium.xD8 = randi_between_2((*yp).x8, (*yp).xC);
            }
            (*temp_r31).u.stadium.xDC = 0;
        }
        _ => {}
    }
}

/// Check if the display is focused on a specific player
pub unsafe extern "C" fn grStadium_801D4FF8(pl_slot: i32) -> i32 {
    melee_core::ct!("grStadium_801D4FF8");
    if ext::stage_info.grkind == Gr_Kind_PStadium as _ {
        let gobj = ext::Ground_GetMapGObj(PsType_Display as i32);
        if !gobj.is_null() {
            let gp = (*gobj).user_data as *mut Ground;
            if !gp.is_null() && (*gp).u.display.xE4 == 8 && (*gp).u.display.xEE as i32 == pl_slot {
                return 1;
            }
        }
    }
    0
}

pub unsafe extern "C" fn fn_801D5074(gobj: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("fn_801D5074");
    grStadium_801D1EF8(gobj);
    ext::grDisplay_801C5DB0(gobj, arg1);
}

pub unsafe extern "C" fn grStadium_OnTouchLine(_unused: i32) -> *mut DynamicsDesc {
    melee_core::ct!("grStadium_OnTouchLine");
    null_mut()
}

pub unsafe extern "C" fn grStadium_OnCheckShadowRender(arg0: *mut Vec3, _arg1: i32, jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grStadium_OnCheckShadowRender");
    let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, null_mut(), &mut sp14);
    if (*arg0).y + 1.0f32 > sp14.y {
        1
    } else {
        0
    }
}

pub unsafe extern "C" fn grStadium_801D511C() {
    melee_core::ct!("grStadium_801D511C");
    let mut size = ext::GXGetTexBufferSize(0xFA, 0xA0, 4, 0, 0);
    ext::lbDvd_80017740(0, 0x7D2, 4, 4, (size.wrapping_add(31)) & !31u32, 0, 7, 0x10, 0);

    size = ext::GXGetTexBufferSize(0x280, 0x196, 4, 0, 0);
    ext::lbDvd_80017740(0, 0x7D3, 4, 4, (size.wrapping_add(31)) & !31u32, 0, 7, 0x10, 0);

    size = ext::GXGetTexBufferSize(0x7C, 0x50, 4, 0, 0);
    ext::lbDvd_80017740(0, 0x7D4, 4, 4, (size.wrapping_add(31)) & !31u32, 0, 7, 0x10, 0);

    ext::lbDvd_80017740(0, 0x7D5, 4, 4, 0x50000, 0, 7, 0x10, 0);
}
