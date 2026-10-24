//! Native port of src/melee/ty/toy.c (TARGET_PC build): the Trophy Gallery / Lottery scene core.
//! Trophy unlock state (`Toy_804A284C`, the save's flags), the stat/sort tables loaded from
//! `TyDatai`, the 3D trophy viewer (cameras, lights, stand model, per-frame input proc), the two
//! debug viewers, the debug unlock editor and the scene enter/leave/readback functions.
//!
//! Every function of the TU is here. Callees in other TUs go through `ext` (an unset hook panics
//! "TODO: <name> is not ported"; names are the C names). Data defined by this TU is `pub static`
//! for `tydisplay` / `tyfigupon` / `tylist`.
//!
//! Native-layout notes. The C reaches several GameCube records through differently typed
//! overlays (`ToyGlobalsS_`, `TyCameraData_`, `tyLightData`, `ToyEntryData`, `TyCleanupObj`,
//! `tyDispData`, `ToyJObjNode`, ...) whose padding only lines up with 4-byte pointers. Here each
//! record is accessed through the one native struct that owns it (`ToyED8Data`, `Toy6E68`,
//! `ToyCameraControl`, `TyDisplayData`, `ToyListEntry`, the `HSD_SObj` list), using the field at
//! the same GameCube offset; allocation sizes are `size_of` of the native records. Pointer-holding
//! slots the C declares as integers are `usize` in melee_types.
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ty/toy.c`): the stick dead-zone remap
//! `-((0.2f*sign) - v)/0.8f`, `0.025f*dist - zoom`, `3.0f*stick - angle` and `x -= x*(0.025f*s)`
//! in the viewer procs (`_Toy_80309404`, `_Toy_8030B530`, `_Toy_8030E110`), and the distance
//! `_Toy_80309338` (f32 `x*x + acc`, f64 `3.0 - g*g*v`); each is marked `// fused`.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused_unsafe,
    static_mut_refs,
    unused_imports,
    unused_variables,
    unused_mut,
    unused_assignments,
    dead_code,
    clippy::all
)]

use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::inline::*;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn DevText_80302AC0(a0: *mut DevText) => unsafe { melee_world::hud::textlib::devtext_80302ac0(&mut *a0) };
    fn DevText_Create(a0: i8, a1: i32, a2: i32, a3: i32, a4: i32, a5: *mut i8) -> *mut DevText => unsafe { melee_world::tu::textlib::DevText_Create(a0, a1, a2, a3, a4, a5) };
    fn DevText_Erase(a0: *mut DevText) => unsafe { melee_world::tu::textlib::DevText_Erase(a0) };
    fn DevText_GetGObj() -> *mut HSD_GObj => unsafe { melee_world::tu::textdraw::DevText_GetGObj() };
    fn DevText_HideBackground(a0: *mut DevText) => unsafe { melee_world::tu::textlib::DevText_HideBackground(a0) };
    fn DevText_HideCursor(a0: *mut DevText) => unsafe { melee_world::tu::textlib::DevText_HideCursor(a0) };
    fn DevText_HideText(a0: *mut DevText) => unsafe { melee_world::tu::textlib::DevText_HideText(a0) };
    fn DevText_Print(a0: *mut DevText, a1: *mut i8) => unsafe { melee_world::tu::textlib::DevText_Print(a0, a1) };
    fn DevText_Remove(a0: *mut *mut DevText) => unsafe { melee_world::tu::textdraw::DevText_Remove(a0) };
    fn DevText_SetBGColor(a0: *mut DevText, a1: _GXColor) -> _GXColor => unsafe { melee_world::tu::textlib::DevText_SetBGColor(a0, a1) };
    fn DevText_SetCursorXY(a0: *mut DevText, a1: i32, a2: i32) => unsafe { melee_world::tu::textlib::DevText_SetCursorXY(a0, a1, a2) };
    fn DevText_SetScale(a0: *mut DevText, a1: f32, a2: f32) => unsafe { melee_world::tu::textlib::DevText_SetScale(a0, a1, a2) };
    fn DevText_SetTextColor(a0: *mut DevText, a1: _GXColor) -> _GXColor => unsafe { melee_world::tu::textlib::DevText_SetTextColor(a0, a1) };
    fn DevText_SetXY(a0: *mut DevText, a1: i32, a2: i32) => unsafe { melee_world::tu::textlib::DevText_SetXY(a0, a1, a2) };
    fn DevText_Show(a0: *mut HSD_GObj, a1: *mut DevText) => unsafe { melee_world::tu::textdraw::DevText_Show(a0, a1) };
    fn DevText_StoreColorIndex(a0: *mut DevText, a1: u8) -> u8 => unsafe { melee_world::tu::textlib::DevText_StoreColorIndex(a0, a1) };
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn GXBegin(a0: u32, a1: u32, a2: u16);
    fn GXColor4u8(a0: u8, a1: u8, a2: u8, a3: u8);
    fn GXLoadPosMtxImm(a0: *mut c_void, a1: u32);
    fn GXPosition3f32(a0: f32, a1: f32, a2: f32);
    fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void => unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0, a1) };
    fn HSD_CObjEndCurrent() => unsafe { melee_runtime::tu::cobj::HSD_CObjEndCurrent() };
    fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32) => unsafe { melee_runtime::tu::cobj::HSD_CObjEraseScreen(a0, a1, a2, a3) };
    fn HSD_CObjGetBottom(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj => unsafe { melee_runtime::tu::cobj::HSD_CObjGetCurrent() };
    fn HSD_CObjGetEyeDistance(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjGetEyeVector(a0: *mut HSD_CObj, a1: *mut Vec) -> i32;
    fn HSD_CObjGetInterest(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjGetLeft(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetLeftVector(a0: *mut HSD_CObj, a1: *mut Vec) -> i32;
    fn HSD_CObjGetRight(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetTop(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetUpVector(a0: *mut HSD_CObj, a1: *mut Vec) -> i32;
    fn HSD_CObjGetViewingMtx(a0: *mut HSD_CObj, a1: *mut c_void);
    fn HSD_CObjSetBottom(a0: *mut HSD_CObj, a1: f32);
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 => unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0) as _ };
    fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjSetFrustum(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32) => unsafe { melee_runtime::tu::cobj::HSD_CObjSetFrustum(a0, a1, a2, a3, a4) };
    fn HSD_CObjSetInterest(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjSetLeft(a0: *mut HSD_CObj, a1: f32);
    fn HSD_CObjSetRight(a0: *mut HSD_CObj, a1: f32);
    fn HSD_CObjSetTop(a0: *mut HSD_CObj, a1: f32);
    fn HSD_FogLoadDesc(a0: *mut HSD_FogDesc) -> *mut HSD_Fog;
    fn HSD_FogSet(a0: *mut HSD_Fog);
    fn HSD_Free(a0: *mut c_void) => unsafe { melee_runtime::tu::memory::HSD_Free(a0) };
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObjProc_RemoveAllProcs(a0: *mut HSD_GObj);
    fn HSD_GObjProc_RemoveProc(a0: *mut HSD_GObjProc);
    fn HSD_GObj_80390CD4(a0: *mut HSD_GObj);
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32);
    fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
    fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1) };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
    fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
    fn HSD_JObjAddChild(a0: *mut HSD_JObj, a1: *mut HSD_JObj);
    fn HSD_JObjAlloc() -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjAlloc() };
    fn HSD_JObjAnim(a0: *mut HSD_JObj);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjRemoveAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjReqAnim(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_LObjAddAnimAll(a0: *mut HSD_LObj, a1: *mut HSD_LightAnim) => unsafe { melee_runtime::tu::lobj::HSD_LObjAddAnimAll(a0, a1) };
    fn HSD_LObjAnimAll(a0: *mut HSD_LObj) => unsafe { melee_runtime::tu::lobj::HSD_LObjAnimAll(a0) };
    fn HSD_LObjGetInterest(a0: *mut HSD_LObj, a1: *mut Vec) -> i32 => unsafe { melee_runtime::tu::lobj::HSD_LObjGetInterest(a0, a1 as _) };
    fn HSD_LObjGetPosition(a0: *mut HSD_LObj, a1: *mut Vec) -> i32 => unsafe { melee_runtime::tu::lobj::HSD_LObjGetPosition(a0, a1 as _) };
    fn HSD_LObjLoadDesc(a0: *mut HSD_LightDesc) -> *mut HSD_LObj => unsafe { melee_runtime::tu::lobj::HSD_LObjLoadDesc(a0) };
    fn HSD_LObjReqAnimAll(a0: *mut HSD_LObj, a1: f32) => unsafe { melee_runtime::tu::lobj::HSD_LObjReqAnimAll(a0, a1) };
    fn HSD_LObjSetInterest(a0: *mut HSD_LObj, a1: *mut Vec) => unsafe { melee_runtime::tu::lobj::HSD_LObjSetInterest(a0, a1 as _) };
    fn HSD_LObjSetPosition(a0: *mut HSD_LObj, a1: *mut Vec) => unsafe { melee_runtime::tu::lobj::HSD_LObjSetPosition(a0, a1 as _) };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void => unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0) };
    fn HSD_PadRenewStatus() => unsafe { melee_runtime::tu::controller::HSD_PadRenewStatus() };
    fn HSD_Randf() -> f32;
    fn HSD_Randi(a0: i32) -> i32;
    fn HSD_SObjLib_803A477C(a0: *mut HSD_GObj, a1: *mut HSD_SObjDesc, a2: u32, a3: u32, a4: u8, a5: u8) -> *mut HSD_SObj => unsafe { melee_runtime::tu::sobjlib::HSD_SObjLib_803A477C(a0, a1, a2, a3, a4, a5) };
    fn HSD_SObjLib_803A49E0(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::sobjlib::HSD_SObjLib_803A49E0(a0, a1) };
    fn HSD_SObjLib_803A55DC(a0: *mut HSD_GObj, a1: u16, a2: u16, a3: i32) => unsafe { melee_runtime::tu::sobjlib::HSD_SObjLib_803A55DC(a0, a1, a2, a3) };
    fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    fn HSD_SetupRenderMode(a0: u32);
    fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text => unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a0, a1, a2, a3, a4, a5, a6) };
    fn HSD_SisLib_803A5E70() => unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5E70() };
    fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 => unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A611C(a0, a1, a2, a3, a4, a5, a6, a7) };
    fn HSD_SisLib_803A62A0(a0: i32, a1: *mut i8, a2: *mut i8) => unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A62A0(a0, a1 as _, a2 as _) };
    fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) => unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A6368(a0, a1) };
    fn HSD_StateInitDirect(a0: i32, a1: u32);
    fn HSD_StateSetLineWidth(a0: u8, a1: i32);
    fn MTXRotRad(a0: *mut c_void, a1: i8, a2: f32);
    fn Mex_Enabled(a0: *const i8) -> i32;
    fn PSMTXMultVecSR(a0: *mut c_void, a1: *mut Vec, a2: *mut Vec);
    fn PSVECAdd(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn PSVECScale(a0: *mut Vec, a1: *mut Vec, a2: f32);
    fn Ui_RetailHidden(a0: i32) -> i32;
    fn gmMainLib_8015ED5C() -> i32;
    fn gmMainLib_GetTrophyCategoryFlags() -> *mut c_void;
    fn gmMainLib_GetTrophyCount() -> *mut i16;
    fn gmMainLib_GetTrophyFlags() -> *mut c_void;
    fn gm_80164430(a0: u16) -> i32;
    fn gm_80164504(a0: u16);
    fn gm_801677E8(a0: i8);
    fn gm_80172C44(a0: u32);
    fn gm_801A4B60();
    fn gm_GetCurrentGameMode() -> u8;
    fn gm_IsCurrently1PMode() -> i32 => unsafe { crate::tu::itzako::gm_IsCurrently1PMode() as _ };
    fn lbArchive_80016EFC(a0: *mut HSD_Archive) => unsafe { melee_runtime::tu::lbarchive::lbArchive_80016EFC(a0) };
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0, a1, a2) };
    fn lbAudioAx_80023F28(a0: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0) };
    fn lbAudioAx_80024030(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0) };
    fn lbLang_IsSavedLanguageJP() -> i32 => unsafe { melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageJP() as _ };
    fn lbLang_IsSavedLanguageUS() -> i32 => unsafe { melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageUS() as _ };
    fn lbLang_IsSettingJP() -> i32 => unsafe { melee_runtime::tu::lblanguage::lbLang_IsSettingJP() as _ };
    fn lbLang_IsSettingUS() -> i32 => unsafe { melee_runtime::tu::lblanguage::lbLang_IsSettingUS() as _ };
    fn lbArchive_LoadSymbols(a0: *const i8, a1: &[*mut c_void]) -> *mut HSD_Archive => unsafe { melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(a0 as _, a1) };
    fn lbArchive_80016DBC(a0: *const i8, a1: *mut c_void, a2: *const i8, a3: i32) -> *mut HSD_Archive;
    fn DevText_Printf(a0: *mut DevText, a1: &str);
    fn lbVector_ApplyEulerRotation(a0: *mut Vec, a1: *mut Vec) -> *mut Vec => unsafe { melee_runtime::tu::lbvector::lbVector_ApplyEulerRotation(a0, a1) };
    fn lb_8000B09C(a0: *mut HSD_JObj) -> i32 => unsafe { melee_runtime::tu::lb_00B0::lb_8000B09C(a0) as _ };
    fn lb_8000D008(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::tu::lb_00CE::lb_8000D008(a0, a1) };
    fn lb_8001204C(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: *mut u16, a3: i32) -> i32 => unsafe { melee_runtime::tu::lbspdisplay::lb_8001204C(a0, a1, a2, a3) };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0) };
    fn memzero(a0: *mut c_void, a1: i32) => unsafe { melee_runtime::tu::lb_00B0::memzero(a0, a1) };
    fn mn_8022F218() -> i32;
    fn mn_8022F268();
    fn tyList_803147C4() => unsafe { crate::tu::tylist::tyList_803147C4() };
    fn tyList_PcActive() -> i32 => unsafe { crate::tu::tylist::tyList_PcActive() };
    fn tyList_PcSetActive(a0: i32) => unsafe { crate::tu::tylist::tyList_PcSetActive(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_803910D8(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_LObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn HSD_SObjLib_803A49E0(a0: *mut HSD_GObj, a1: i32) { super::HSD_SObjLib_803A49E0(a0, a1) }
    }
    /// `OSReport` (variadic in C): the text is formatted by the caller. Output only; unset = discard.
    pub static mut OSReport_hook: Option<fn(&str)> = None;
    pub fn OSReport(s: &str) {
        if let Some(f) = unsafe { OSReport_hook } {
            f(s)
        }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut DbLevel: DbLKind = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_CameraKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_FogKind: i8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_LightKind: i8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT


/// Retail UI element ids (`pc/platform/gw_ui_retail_ids.h`).
pub const AT_RE_TOY_PANEL: i32 = 9;
pub const AT_RE_TOY_INFO: i32 = 10;
pub const AT_RE_TOY_TEXT: i32 = 11;

/// `ToyAnim` etc. share the C `sizeof` of GameCube records only in the GC offset tables; the
/// native records allocated by this TU use `size_of`.
///
/// NUL-terminated C string literal as `*mut i8`.
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *mut i8
    };
}
pub(crate) use cstr;

/// The `%s` of a C string pointer, for `OSReport` text.
pub unsafe fn c_str(p: *const i8) -> String {
    if p.is_null() {
        return String::from("(null)");
    }
    core::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
}

/// `JObjDirty` host: the shipping `HSD_JObjSetMtxDirty` is `if (!HSD_JObjMtxIsDirty(j))
/// HSD_JObjSetMtxDirtySub(j)`; every call goes to the unported `jobj.c`.
pub struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, jobj: &mut HSD_JObj) {
        if !HSD_JObjMtxIsDirty(jobj) {
            ext::HSD_JObjSetMtxDirtySub(jobj);
        }
    }
    fn set_mtx_dirty_sub(&mut self, jobj: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

// ---------------------------------------------------------------------------------------------
// Data defined by toy.c
// ---------------------------------------------------------------------------------------------

/* 3B8844 */
static mut _Toy_803B8844: HSD_FogDesc = HSD_FogDesc {
    r#type: GX_FOG_LIN as u32,
    fogadjdesc: core::ptr::null_mut(),
    start: 0.0,
    end: 1.0,
    color: _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF },
};
/* 3B8858 */
static _Toy_803B8858: Vec3 = Vec3 { x: 0.0, y: 8.0, z: 0.0 };
/* 3B8864 */
static _Toy_803B8864: PosArrayFull = PosArrayFull {
    a: [
        PosArray { xy: [526, 176] },
        PosArray { xy: [512, 74] },
        PosArray { xy: [0, 74] },
        PosArray { xy: [491, 174] },
        PosArray { xy: [46, 423] },
        PosArray { xy: [66, 37] },
        PosArray { xy: [546, 37] },
    ],
};
/* 3B889C */
static _Toy_803B889C: PosArrayFull = PosArrayFull {
    a: [
        PosArray { xy: [526, 172] },
        PosArray { xy: [496, 74] },
        PosArray { xy: [0, 72] },
        PosArray { xy: [491, 174] },
        PosArray { xy: [46, 423] },
        PosArray { xy: [66, 37] },
        PosArray { xy: [546, 37] },
    ],
};
/* 3B88D4..3B8904 */
static _Toy_803B88D4: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static _Toy_803B88E0: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static _Toy_803B88EC: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static _Toy_803B88F8: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static _Toy_803B8904: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
/* 3B8910: `ToyEntry` rows {category id, value} (the value's low byte is the letter shown). */
static _Toy_803B8910: [(i32, i32); 9] = [(7, 65), (6, 66), (5, 67), (4, 68), (3, 69), (2, 70), (0, 73), (1, 88), (8, 83)];
/* 4A26B8 */
pub static mut _Toy_804A26B8: _Toy_804A26B8_t = unsafe { core::mem::zeroed() };
/* 4A26C4 */
static mut _Toy_devtext_buf_804A26C4: [u8; 0x8C] = [0; 0x8C];
/* 4A2750 */
static mut _Toy_devtext_buf_804A2750: [u8; 0xFC] = [0; 0xFC];
/* 4A284C */
pub static mut Toy_804A284C: [u16; 302] = [0; 302];
/* 4A2AA8 */
pub static mut Toy_804A2AA8: ToyAnimState = unsafe { core::mem::zeroed() };

/* 4D5A40 */
static _Toy_color_E2E2E2FF: GXColor = GXColor { r: 0xE2, g: 0xE2, b: 0xE2, a: 0xFF };
/* 4D5A44 */
static _Toy_color_FF8020FF: GXColor = GXColor { r: 0xFF, g: 0x80, b: 0x20, a: 0xFF };
/* 4DDCFC */
static _Toy_color_FFBA00FF: GXColor = GXColor { r: 0xFF, g: 0xBA, b: 0x00, a: 0xFF };
/* 4DDE0C */
static _Toy_color_4080D060_0: GXColor = GXColor { r: 0x40, g: 0x80, b: 0xD0, a: 0x60 };
/* 4DDE18 */
static _Toy_color_40B0D060_1: GXColor = GXColor { r: 0x40, g: 0x80, b: 0xD0, a: 0x60 };

/* 4D6EE0 */
pub static mut Toy_sbss_804D6EE0: *mut TyDisplayData = core::ptr::null_mut();
/* 4D6EDC */
pub static mut Toy_sbss_804D6EDC: *mut i16 = core::ptr::null_mut();
/* 4D6ED8 */
pub static mut Toy_sbss_804D6ED8: *mut ToyED8Data = core::ptr::null_mut();
/* 4D6ED4 */
pub static mut Toy_sbss_804D6ED4: *mut ToyCameraControl = core::ptr::null_mut();
/* 4D6ED0 */
pub static mut _Toy_sbss_804D6ED0: *mut HSD_Archive = core::ptr::null_mut();
/* 4D6ECC */
pub static mut _Toy_sbss_804D6ECC: *mut HSD_Archive = core::ptr::null_mut();
/* 4D6EC8 */
pub static mut Toy_sbss_804D6EC8: *mut HSD_Archive = core::ptr::null_mut();
/* 4D6EC4 */
pub static mut _Toy_sbss_804D6EC4: *mut TrophyData = core::ptr::null_mut();
/* 4D6EC0 */
pub static mut _Toy_sbss_804D6EC0: *mut TrophyData = core::ptr::null_mut();
/* 4D6EBC */
pub static mut _Toy_sbss_804D6EBC: *mut c_void = core::ptr::null_mut();
/* 4D6EB8 */
pub static mut _Toy_sbss_804D6EB8: *mut c_void = core::ptr::null_mut();
/* 4D6EB4 */
pub static mut _Toy_sbss_804D6EB4: *mut i16 = core::ptr::null_mut();
/* 4D6EB0 */
pub static mut Toy_sbss_804D6EB0: *mut TyDspEntry = core::ptr::null_mut();
/* 4D6EAC */
pub static mut Toy_sbss_804D6EAC: *mut TyDspEntry = core::ptr::null_mut();
/* 4D6EA8 */
pub static mut _Toy_sbss_804D6EA8: *mut ToyModelFile = core::ptr::null_mut();
/* 4D6EA4 */
pub static mut _Toy_sbss_804D6EA4: *mut ToyModelFile = core::ptr::null_mut();
/* 4D6EA2 */
pub static mut _Toy_sbss_804D6EA2: i8 = 0;
/* 4D6EA1 */
pub static mut _Toy_sbss_804D6EA1: u8 = 0;
/* 4D6EA0 */
pub static mut _Toy_sbss_804D6EA0: u8 = 0;
/* 4D6E9C */
pub static mut _Toy_sbss_804D6E9C: *mut DevText = core::ptr::null_mut();
/* 4D6E98 */
pub static mut _Toy_sbss_804D6E98: *mut DevText = core::ptr::null_mut();
/* 4D6E94..4D6E80 */
pub static mut _Toy_sbss_804D6E94: f32 = 0.0;
pub static mut _Toy_sbss_804D6E90: f32 = 0.0;
pub static mut _Toy_sbss_804D6E8C: f32 = 0.0;
pub static mut _Toy_sbss_804D6E88: f32 = 0.0;
pub static mut _Toy_sbss_804D6E84: f32 = 0.0;
pub static mut _Toy_sbss_804D6E80: f32 = 0.0;
/* 4D6E7C..4D6E70 */
pub static mut _Toy_sbss_804D6E7C: i32 = 0;
pub static mut _Toy_sbss_804D6E78: i32 = 0;
pub static mut _Toy_sbss_804D6E74: i32 = 0;
pub static mut _Toy_sbss_804D6E70: i32 = 0;
/* 4D6E6C */
pub static mut _Toy_sbss_804D6E6C: *mut TyViewData = core::ptr::null_mut();
/* 4D6E68 */
pub static mut _Toy_sbss_804D6E68: *mut Toy6E68 = core::ptr::null_mut();
/* 4D6E64 */
pub static mut _Toy_sbss_804D6E64: *mut TySortRow = core::ptr::null_mut();
/* 4D6E60 */
pub static mut _Toy_sbss_804D6E60: i32 = 0;
/* 4D6E5C */
pub static mut _Toy_sbss_804D6E5C: *mut ToyParamEditor = core::ptr::null_mut();
/* 4D6E58 */
pub static mut _Toy_sbss_804D6E58: i32 = 0;
/* 4D6E54 */
pub static mut _Toy_sbss_804D6E54: i32 = 0;
/* 4D6E50 */
pub static mut _Toy_sbss_804D6E50: i8 = 0;

/// Number of sort keys held per trophy (`TY_SORT_KEY_COUNT`).
const TY_SORT_KEY_COUNT: i32 = 3;

/* 3FDD18 */
static _Toy_str_TyLight_dat: [u8; 12] = *b"TyLight.dat\0";

/* 3FDDE4 */
static mut _Toy_803FDDE4: lbl_803FDDE4_t = lbl_803FDDE4_t {
    symbols: [
        lbl_803FDDE4_t_symbols { name: cstr!("ScToyLightMaster0_scene_lights"), empty: core::ptr::null_mut() },
        lbl_803FDDE4_t_symbols { name: cstr!("ScToyLightMaster1_scene_lights"), empty: core::ptr::null_mut() },
        lbl_803FDDE4_t_symbols { name: cstr!("ScToyLightMaster2_scene_lights"), empty: core::ptr::null_mut() },
        lbl_803FDDE4_t_symbols { name: cstr!("ScToyLightMaster4_scene_lights"), empty: core::ptr::null_mut() },
        lbl_803FDDE4_t_symbols { name: cstr!("ScToyLightMaster5_scene_lights"), empty: core::ptr::null_mut() },
        lbl_803FDDE4_t_symbols { name: cstr!("ScToyLightMaster6_scene_lights"), empty: core::ptr::null_mut() },
    ],
    values: [
        lbl_803FDDE4_t_values { index: 0, color: _GXColor { r: 0x00, g: 0x00, b: 0x19, a: 0xFF }, flag: 1 },
        lbl_803FDDE4_t_values { index: 1, color: _GXColor { r: 0x10, g: 0x20, b: 0x20, a: 0xFF }, flag: 1 },
        lbl_803FDDE4_t_values { index: 2, color: _GXColor { r: 0x26, g: 0x13, b: 0x30, a: 0xFF }, flag: 0 },
        lbl_803FDDE4_t_values { index: 3, color: _GXColor { r: 0x12, g: 0x12, b: 0x12, a: 0xFF }, flag: 1 },
        lbl_803FDDE4_t_values { index: 4, color: _GXColor { r: 0x00, g: 0x00, b: 0x19, a: 0xFF }, flag: 0 },
        lbl_803FDDE4_t_values { index: 5, color: _GXColor { r: 0x00, g: 0x00, b: 0x19, a: 0xFF }, flag: 0 },
    ],
};

/* 3FDEA0 */
pub static mut _Toy_803FDEA0: [*mut i8; 1] = [cstr!("ToyFigurePanel_Top_joint")];

/* 3FDEBC */
static mut _Toy_803FDEBC: [*mut i8; 6] = [
    cstr!("ToyFigureBack_Top_joint"),
    cstr!("ToyFigureBack_Top_joint"),
    core::ptr::null_mut(),
    cstr!("ToyFigureBack_Top_joint"),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
];

/* 3FDF3C */
static mut _Toy_803FDF3C: ModelNamesDesc = ModelNamesDesc {
    animjoint: cstr!("ToyFigurePanel_Top_animjoint"),
    matanim_joint: cstr!("ToyFigurePanel_Top_matanim_joint"),
    shapeanim_joint: cstr!("ToyFigurePanel_Top_shapeanim_joint"),
};

/* 3FDFA8 */
static mut _Toy_803FDFA8: [ModelNamesDesc; 6] = [
    ModelNamesDesc {
        animjoint: cstr!("ToyFigureBack_Top_animjoint"),
        matanim_joint: cstr!("ToyFigureBack_Top_matanim_joint"),
        shapeanim_joint: cstr!("ToyFigureBack_Top_shapeanim_joint"),
    },
    ModelNamesDesc {
        animjoint: cstr!("ToyFigureBack_Top_animjoint"),
        matanim_joint: cstr!("ToyFigureBack_Top_matanim_joint"),
        shapeanim_joint: cstr!("ToyFigureBack_Top_shapeanim_joint"),
    },
    ModelNamesDesc { animjoint: core::ptr::null_mut(), matanim_joint: core::ptr::null_mut(), shapeanim_joint: core::ptr::null_mut() },
    ModelNamesDesc {
        animjoint: cstr!("ToyFigureBack_Top_animjoint"),
        matanim_joint: cstr!("ToyFigureBack_Top_matanim_joint"),
        shapeanim_joint: cstr!("ToyFigureBack_Top_shapeanim_joint"),
    },
    ModelNamesDesc { animjoint: core::ptr::null_mut(), matanim_joint: core::ptr::null_mut(), shapeanim_joint: core::ptr::null_mut() },
    ModelNamesDesc { animjoint: core::ptr::null_mut(), matanim_joint: core::ptr::null_mut(), shapeanim_joint: core::ptr::null_mut() },
];

/* 3FE038 */
static mut _Toy_803FE038: [*mut i8; 3] = [cstr!("ToyFigureBg3_sobjdesc"), cstr!("ToyFigureBg5_sobjdesc"), cstr!("ToyFigureBg6_sobjdesc")];

/* 3FE108 */
static mut _Toy_803FE108: [*mut i8; 7] = [
    cstr!("ToyCameraInfoBoad_sobjdesc"),
    cstr!("ToyCameraInfoNext_sobjdesc"),
    cstr!("ToyCameraInfoPrev_sobjdesc"),
    cstr!("ToyCameraInfoButton_sobjdesc"),
    cstr!("ToyCameraInfoReturn_sobjdesc"),
    cstr!("ToyCameraInfoL_sobjdesc"),
    cstr!("ToyCameraInfoR_sobjdesc"),
];

/* 3FE244 */
static mut _Toy_803FE244: [ModelNamesDesc; 2] = [
    ModelNamesDesc {
        animjoint: cstr!("ToyStandModel_TopN_ACTION_action1_animjoint"),
        matanim_joint: cstr!("ToyStandModel_TopN_ACTION_action1_matanim_joint"),
        shapeanim_joint: cstr!("ToyStandModel_TopN_ACTION_action1_shapeanim_joint"),
    },
    ModelNamesDesc {
        animjoint: cstr!("ToyStandModel_TopN_ACTION_action2_animjoint"),
        matanim_joint: cstr!("ToyStandModel_TopN_ACTION_action2_matanim_joint"),
        shapeanim_joint: cstr!("ToyStandModel_TopN_ACTION_action2_shapeanim_joint"),
    },
];

/* 3FE3F8 */
static mut _Toy_803FE3F8: [u16; 9] = [1, 7, 22, 20, 18, 21, 19, 17, 23];

/* 3FE5E8 */
pub static Toy_str_ScMenFigure_cam_int1_camera: [u8; 28] = *b"ScMenFigure_cam_int1_camera\0";

// ---------------------------------------------------------------------------------------------
// Trophy unlock queries
// ---------------------------------------------------------------------------------------------

/// `gm_IsCurrently1PMode() || gm_GetCurrentGameMode() == GM_TOY_LOTTERY`, the test that picks
/// the scene-local Toy array over the save's trophy data. (Same call order as the C.)
#[inline]
unsafe fn toy_scene_mode() -> bool {
    ext::gm_IsCurrently1PMode() != 0 || ext::gm_GetCurrentGameMode() as i32 == GM_TOY_LOTTERY as i32
}

/// `&Toy_804A284C[5]`: the scene-local trophy flags (`TOY_FLAGS`).
#[inline]
unsafe fn toy_flags() -> *mut u16 {
    (&raw mut Toy_804A284C as *mut u16).add(5)
}

/// `getTrophyFlags()`.
#[inline]
unsafe fn getTrophyFlags() -> *mut u16 {
    if toy_scene_mode() {
        toy_flags()
    } else {
        ext::gmMainLib_GetTrophyFlags() as *mut u16
    }
}

/// `Toy_804A284C[i]` as a `u16` pointer.
#[inline]
unsafe fn toy_u16(i: usize) -> *mut u16 {
    (&raw mut Toy_804A284C as *mut u16).add(i)
}

/// `(u8*) Toy_804A284C + off`.
#[inline]
unsafe fn toy_byte(off: usize) -> *mut u8 {
    (&raw mut Toy_804A284C as *mut u8).add(off)
}

/// `TOY_COUNT`: `(s16*) &Toy_804A284C[0x12C]`.
#[inline]
unsafe fn TOY_COUNT() -> *mut i16 {
    toy_u16(0x12C) as *mut i16
}

/// `(TyModeState*) Toy_804A284C`.
#[inline]
pub unsafe fn toy_mode_state() -> *mut TyModeState {
    &raw mut Toy_804A284C as *mut TyModeState
}

/// `HSD_ASSERT(line, 0)` failure.
#[cold]
fn hsd_assert_fail(line: u32) -> ! {
    panic!("toy.c:{line}: assertion failed")
}

/// All trophies unlocked?
pub unsafe fn un_80304470() -> bool {
    let mut sum = 0;
    for i in 0..8 {
        sum += Toy_80304B94(i);
    }
    let mut count = 0;
    for i in 0..TY_TROPHY_COUNT {
        if i != 0xE6 && i != 0xC9 && Toy_803048C0(i) != 0 {
            count += 1;
        }
    }
    if sum <= count {
        true
    } else {
        // Ported from m-ex (Unlock Everything / Unlock All Trophies / Spoof All Trophies as
        // Unlocked): opt-in with MELEE_MEX=unlock_all_trophies.
        if ext::Mex_Enabled(cstr!("unlock_all_trophies")) != 0 {
            return true;
        }
        false
    }
}

pub unsafe fn un_80304510() -> bool {
    let mut sum = 0;
    for i in 0..9 {
        sum += Toy_80304B94(i);
    }
    let mut count = 0;
    for i in 0..TY_TROPHY_COUNT {
        if Toy_803048C0(i) != 0 {
            count += 1;
        }
    }
    sum == count
}

unsafe fn all_unlocked(sp: &[i16]) -> bool {
    let mut count = 0usize;
    for &t in sp {
        if Toy_803048C0(t as i32) != 0 {
            count += 1;
        }
    }
    count == sp.len()
}

pub unsafe fn un_803045A0() -> bool {
    let sp: [i16; 26] = [0, 3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45, 48, 51, 54, 57, 60, 63, 66, 69, 72, 75];
    all_unlocked(&sp)
}

/// Duplicate of `un_803045A0` with different data.
pub unsafe fn un_80304690() -> bool {
    let sp: [i16; 26] = [1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 31, 34, 37, 40, 43, 46, 49, 52, 55, 58, 61, 64, 67, 70, 73, 76];
    all_unlocked(&sp)
}

/// Duplicate of `un_803045A0` with different data.
pub unsafe fn un_80304780() -> bool {
    let sp: [i16; 26] = [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38, 41, 44, 47, 50, 53, 56, 59, 62, 65, 68, 71, 74, 77];
    all_unlocked(&sp)
}

pub unsafe fn Toy_GetTrophyTotal() -> i32 {
    if ext::gm_IsCurrently1PMode() != 0 || ext::gm_GetCurrentGameMode() as i32 == GM_TOY_LOTTERY as i32 {
        *toy_u16(0x258 / 2) as i16 as i32
    } else {
        *ext::gmMainLib_GetTrophyCount() as i32
    }
}

/// `_Toy_GetTrophyTotal(toy)` (static inline; PC reads the real symbol).
pub unsafe fn _Toy_GetTrophyTotal() -> i32 {
    if ext::gm_IsCurrently1PMode() != 0 || ext::gm_GetCurrentGameMode() as i32 == GM_TOY_LOTTERY as i32 {
        return *toy_u16(0x12C) as i16 as i32;
    }
    *ext::gmMainLib_GetTrophyCount() as i32
}

pub unsafe fn Toy_803048C0(arg0: i32) -> i32 {
    // Ported from m-ex (Unlock Everything / Unlock All Trophies / Have 99 of Every Trophy):
    // opt-in with MELEE_MEX=have_99_trophies.
    if ext::Mex_Enabled(cstr!("have_99_trophies")) != 0 {
        return 99;
    }
    (*getTrophyFlags().offset(arg0 as isize) & 0xFF) as i32
}

pub unsafe fn Toy_80304924(arg0: i32) -> bool {
    *getTrophyFlags().offset(arg0 as isize) & 0x8000 != 0
}

pub unsafe fn Toy_80304988(arg0: i32) {
    let v = getTrophyFlags();
    *v.offset(arg0 as isize) = *v.offset(arg0 as isize) ^ 0x8000;
}

pub unsafe fn Toy_803049F4(arg0: i32) -> bool {
    *getTrophyFlags().offset(arg0 as isize) & 0x4000 != 0
}

pub unsafe fn Toy_80304A58(arg0: i32) {
    let v = getTrophyFlags();
    if Toy_803049F4(arg0) {
        *v.offset(arg0 as isize) = *v.offset(arg0 as isize) ^ 0x4000;
    }
}

pub unsafe fn Toy_80304B0C(arg0: i32) -> bool {
    let mut s: u16 = 0;
    let v: *mut u16;
    if toy_scene_mode() {
        s = *toy_u16(3) | *toy_u16(4);
        v = &mut s;
    } else {
        v = ext::gmMainLib_GetTrophyCategoryFlags() as *mut u16;
    }
    *v as i32 & (1 << arg0) != 0
}

pub unsafe fn Toy_80304B94(option: i32) -> i32 {
    // Every case calls lbLang_IsSettingJP() and yields the same value for both languages except 5.
    match option {
        7 => {
            ext::lbLang_IsSettingJP();
            0x4
        }
        6 => {
            ext::lbLang_IsSettingJP();
            0xA
        }
        5 => {
            if ext::lbLang_IsSettingJP() != 0 {
                0x11
            } else {
                0x10
            }
        }
        4 => {
            ext::lbLang_IsSettingJP();
            0x17
        }
        3 => {
            ext::lbLang_IsSettingJP();
            0xC
        }
        2 => {
            ext::lbLang_IsSettingJP();
            0x48
        }
        0 => {
            ext::lbLang_IsSettingJP();
            0x1B
        }
        1 => {
            ext::lbLang_IsSettingJP();
            0x7E
        }
        8 => {
            ext::lbLang_IsSettingJP();
            0x2
        }
        // The C leaves `res` unset for other values.
        _ => 0,
    }
}

pub unsafe fn Toy_80304CC8(arg0: i32) -> bool {
    let mut v = _Toy_sbss_804D6EB4;
    if ext::lbLang_IsSettingUS() != 0 {
        while *v != -1 {
            if *v as i32 == arg0 {
                return false;
            }
            v = v.add(1);
        }
    }
    true
}

pub unsafe fn _Toy_80304CC8_noinline(arg0: i32) -> bool {
    Toy_80304CC8(arg0)
}

pub unsafe fn _Toy_80304D30() -> i32 {
    let mut sp14 = [0i32; 9];
    let mut x: i32 = 0;

    if un_80304470() {
        return 0;
    }
    ext::memzero(sp14.as_mut_ptr() as *mut c_void, 36);
    let mut count = 0;
    for i in 0..TY_TROPHY_COUNT {
        if Toy_80304CC8(i) && Toy_803048C0(i) != 0 {
            x = Toy_803060BC(i, 6) as i32;
            sp14[x as usize] += 1;
            if x != 8 && x != 1 {
                count += 1;
            }
        }
    }
    let mut idk: i32 = 6;
    let mut qwe: usize = 6;
    while idk != 0 {
        if idk as u32 > 2 && sp14[qwe] != 0 && sp14[qwe] == Toy_80304B94(idk) {
            for i in 0..idk {
                if 1 < i as u32 && i != 3 && !Toy_80304B0C(i) {
                    *toy_byte(0) = 2;
                    Toy_80305918(i as i8, 0, 0);
                }
            }
            let mut i = idk + 1;
            while Toy_80304B94(i) == 0 {
                i += 1;
            }
            if 4 <= i && i <= 6 && !Toy_80304B0C(i) {
                *toy_byte(0) = 2;
                Toy_80305918(i as i8, 0, 0);
                break;
            }
            break;
        }
        idk -= 1;
        qwe = qwe.wrapping_sub(1);
    }
    let mut count2 = 0;
    for i in 0..8 {
        if x != 8 && x != 1 && Toy_80304B0C(i) {
            count2 += Toy_80304B94(i);
        }
    }
    count2 - count
}

/// Pick a trophy matching the lottery request. `-1` if none.
pub unsafe fn Toy_80305058(arg0: i32, arg1: i32, arg2: i32, farg0: f32) -> i32 {
    let mut obtained_arr = [0i32; TY_TROPHY_COUNT as usize];
    let mut new_arr = [0i32; TY_TROPHY_COUNT as usize];
    let default_flags = toy_flags();
    let mut new_count: i32 = 0;
    let mut obtained_count: i32 = 0;
    let mut total: i32 = 0;

    // The flags the C re-fetches at every site.
    #[inline]
    unsafe fn flags_now(default_flags: *mut u16) -> *mut u16 {
        if toy_scene_mode() {
            default_flags
        } else {
            ext::gmMainLib_GetTrophyFlags() as *mut u16
        }
    }

    for trophy in 0..TY_TROPHY_COUNT {
        let mut skip_list = _Toy_sbss_804D6EB4;
        let mut skip = 1;
        if ext::lbLang_IsSettingUS() != 0 {
            loop {
                let val = *skip_list;
                if val == -1 {
                    break;
                }
                if val as i32 == trophy {
                    skip = 0;
                    break;
                }
                skip_list = skip_list.add(1);
            }
        }
        if skip == 0 {
            continue;
        }

        let mut add_obtained = false;
        if arg0 == 0x63 {
            let flags = flags_now(default_flags);
            if *flags.offset(trophy as isize) & 0x4000 == 0 {
                if arg1 == 0x63 {
                    // nothing
                } else if arg1 as f32 == Toy_803060BC(trophy, 6) {
                    if arg2 != 0 {
                        let flags = flags_now(default_flags);
                        if *flags.offset(trophy as isize) & 0x4000 != 0 {
                            add_obtained = true;
                        }
                    } else {
                        add_obtained = true;
                    }
                }
            } else {
                add_obtained = true;
            }
        } else if arg0 as f32 != Toy_803060BC(trophy, 6) {
            if arg1 != 0x63 && arg1 as f32 == Toy_803060BC(trophy, 6) {
                if arg2 != 0 {
                    let flags = flags_now(default_flags);
                    if *flags.offset(trophy as isize) & 0x4000 != 0 {
                        add_obtained = true;
                    }
                } else {
                    add_obtained = true;
                }
            }
        } else if arg2 != 0 {
            let flags = flags_now(default_flags);
            if *flags.offset(trophy as isize) & 0x4000 != 0 {
                add_obtained = true;
            }
        } else {
            add_obtained = true;
        }

        if add_obtained {
            let flags = flags_now(default_flags);
            if (*flags.offset(trophy as isize) & 0xFF) != 0 {
                obtained_arr[obtained_count as usize] = trophy;
                obtained_count += 1;
            } else {
                new_arr[new_count as usize] = trophy;
                new_count += 1;
            }
            total += 1;
        }
    }

    if total != 0 {
        let use_new;
        if farg0 >= 100.0 || obtained_count == 0 {
            use_new = 1;
        } else {
            let rand = ext::HSD_Randf();
            let rand_int = ext::HSD_Randi(0x64);
            if (rand_int as f32) + rand < farg0 {
                use_new = 1;
            } else {
                use_new = 0;
            }
        }
        if use_new != 0 && new_count != 0 {
            let mut idx = ext::HSD_Randi(new_count);
            if idx > new_count {
                idx = 0;
            }
            return new_arr[idx as usize];
        }
        let mut idx = ext::HSD_Randi(obtained_count);
        if idx > obtained_count {
            idx = 0;
        }
        return obtained_arr[idx as usize];
    }
    -1
}

/// Whether trophy `id` is in the language's enabled set (the inlined skip-list scan of the C).
#[inline]
unsafe fn trophy_enabled_inline(id: i32) -> bool {
    let mut list = _Toy_sbss_804D6EB4;
    if ext::lbLang_IsSettingUS() != 0 {
        while *list != -1 {
            if *list as i32 == id {
                return false;
            }
            list = list.add(1);
        }
    }
    true
}

pub unsafe fn _Toy_803053C4(target_value: i32, mut count: i32, flag: i32) {
    let default_flags = toy_flags();
    if flag != 0 {
        while count != 0 {
            let mut trophy_id: i32 = 0;
            while trophy_id < TY_TROPHY_COUNT {
                if trophy_enabled_inline(trophy_id) {
                    if target_value as f32 == Toy_803060BC(trophy_id, 6) {
                        if ext::HSD_Randi(2) == 0 {
                            let ptr = if toy_scene_mode() { default_flags } else { ext::gmMainLib_GetTrophyFlags() as *mut u16 };
                            if (*ptr.offset(trophy_id as isize) & 0xFF) == 0 {
                                Toy_SetUnlockState(trophy_id as i16 as i32, ext::HSD_Randi(0xFE) + 1);
                                let ptr = if toy_scene_mode() { default_flags } else { ext::gmMainLib_GetTrophyFlags() as *mut u16 };
                                let ptr = ptr.offset(trophy_id as isize);
                                count -= 1;
                                *ptr ^= 0x8000;
                            }
                        }
                    }
                    if count == 0 {
                        break;
                    }
                }
                trophy_id += 1;
            }
        }
    } else {
        let mut i: i32 = 0;
        while i < TY_TROPHY_COUNT {
            if trophy_enabled_inline(i) && target_value as f32 == Toy_803060BC(i, 6) {
                Toy_SetUnlockState(i as i16 as i32, ext::HSD_Randi(0xFE) + 1);
                let ptr = if toy_scene_mode() { default_flags } else { ext::gmMainLib_GetTrophyFlags() as *mut u16 };
                let ptr = ptr.offset(i as isize);
                count -= 1;
                *ptr ^= 0x8000;
                if count == 0 {
                    return;
                }
            }
            i += 1;
        }
    }
}

/// Unlock `trophy_id` (first time: set the new bit and bump the count) and add `add_value`
/// (saturating at 0xFF) to its collected count.
pub unsafe fn Toy_SetUnlockState(trophy_id: i32, add_value: i32) {
    let trophy_table = toy_flags();
    let table_now = || -> *mut u16 {
        if toy_scene_mode() {
            trophy_table
        } else {
            ext::gmMainLib_GetTrophyFlags() as *mut u16
        }
    };

    let mut table = table_now();
    let idx = trophy_id as i16;
    let byte_offset = idx as isize * 2;
    // One line per unlock. TY_TROPHY_COUNT is the size of every per-trophy array in the save
    // and in Toy, so an id at or past it corrupts whatever follows.
    ext::OSReport(&format!(
        "ty: unlock trophy {} (add={}){}\n",
        idx as i32,
        add_value,
        if (idx as i32) < 0 || idx as i32 >= TY_TROPHY_COUNT { "  *** OUT OF RANGE ***" } else { "" }
    ));
    if (*((table as *mut u8).offset(byte_offset) as *mut u16) & 0xFF) == 0 {
        table = table_now();
        table = (table as *mut u8).offset(byte_offset) as *mut u16;
        let mut temp = *table;
        temp ^= 0x8000;
        *table = temp;
        let old_count: i32 = if toy_scene_mode() { *TOY_COUNT() as i32 } else { *ext::gmMainLib_GetTrophyCount() as i32 };
        let new_count = old_count + 1;
        if toy_scene_mode() {
            *TOY_COUNT() = new_count as i16;
        } else {
            *ext::gmMainLib_GetTrophyCount() = new_count as i16;
        }
    }
    table = table_now();
    if add_value + (*((table as *mut u8).offset(byte_offset) as *mut u16) & 0xFF) as i32 <= 0xFF {
        table = table_now();
        let new_val = add_value + (*((table as *mut u8).offset(byte_offset) as *mut u16) & 0xFF) as i32;
        table = table_now();
        let ptr = (table as *mut u8).offset(byte_offset) as *mut u16;
        *ptr = (new_val + (*ptr & 0xFF00) as i32) as u16;
    } else {
        table = table_now();
        let ptr = (table as *mut u8).offset(byte_offset) as *mut u16;
        *ptr = ((*ptr & 0xFF00) as i32 + 0xFF) as u16;
    }
    let mut state2: u16 = 0;
    let state_ptr: *mut u16;
    if toy_scene_mode() {
        state2 = *toy_u16(3) | *toy_u16(4);
        state_ptr = &mut state2;
    } else {
        state_ptr = ext::gmMainLib_GetTrophyCategoryFlags() as *mut u16;
    }
    let state_flag = if *state_ptr & 0x80 != 0 { 1 } else { 0 };
    if state_flag == 0 {
        let count: i32 = if toy_scene_mode() { *TOY_COUNT() as i32 } else { *ext::gmMainLib_GetTrophyCount() as i32 };
        if count >= 0xFA {
            *toy_byte(0) = 2;
            Toy_80305918(7, 0, 0);
        }
    }
    _Toy_80304D30();
    if trophy_id as i16 == 0xA5 && ext::gm_80164430(0x14) == 0 {
        ext::gm_80164504(0x14);
    }
    if Toy_803060BC(idx as i32, 6) as i32 == 1 {
        ext::gm_80172C44(idx as u32);
    }
}

pub unsafe fn Toy_80305918(arg0: i8, arg1: i32, arg2: i32) {
    if arg0 == 8 {
        return;
    }
    let temp_r26 = toy_flags();
    for var_r25 in 0..TY_TROPHY_COUNT {
        if !trophy_enabled_inline(var_r25) {
            continue;
        }
        if arg0 as f32 != Toy_803060BC(var_r25, 6) {
            continue;
        }
        if arg1 != 0 {
            let var_r22_2 = if toy_scene_mode() { temp_r26 } else { ext::gmMainLib_GetTrophyFlags() as *mut u16 };
            let var_r3 = if toy_scene_mode() { temp_r26 } else { ext::gmMainLib_GetTrophyFlags() as *mut u16 };
            if *var_r3.offset(var_r25 as isize) & 0x4000 != 0 {
                let temp_ptr = var_r22_2.offset(var_r25 as isize);
                let val = *temp_ptr;
                *temp_ptr = val ^ 0x4000;
            }
        } else {
            let var_r3 = if toy_scene_mode() { temp_r26 } else { ext::gmMainLib_GetTrophyFlags() as *mut u16 };
            *var_r3.offset(var_r25 as isize) |= 0x4000;
        }
    }
    if arg1 != 0 {
        if arg2 != 0 {
            let val4 = *toy_u16(4);
            let mask2 = 1i32 << arg0;
            if val4 as i32 & mask2 != 0 {
                *toy_u16(4) = (val4 as i32 ^ mask2) as u16;
            }
        } else {
            let ptr = if toy_scene_mode() { toy_u16(3) } else { ext::gmMainLib_GetTrophyCategoryFlags() as *mut u16 };
            let temp_val = *ptr;
            let mask = 1i32 << arg0;
            if temp_val as i32 & mask != 0 {
                *ptr = (temp_val as i32 ^ mask) as u16;
            }
        }
    } else if arg2 != 0 {
        let ptr19c = toy_u16(4);
        *ptr19c = (*ptr19c as i32 | (1i32 << arg0)) as u16;
    } else {
        let ptr = if toy_scene_mode() { toy_u16(3) } else { ext::gmMainLib_GetTrophyCategoryFlags() as *mut u16 };
        *ptr = (*ptr as i32 | (1i32 << arg0)) as u16;
    }
}

// ---------------------------------------------------------------------------------------------
// Pad readers
// ---------------------------------------------------------------------------------------------

#[inline]
unsafe fn pad(slot: u8) -> *mut HSD_PadStatus {
    (&raw mut ext::HSD_PadCopyStatus as *mut HSD_PadStatus).add(slot as usize)
}

pub unsafe fn Toy_80305B88() -> i32 {
    let mut button: u32 = 0;
    for i in 0..4 {
        button = (*pad(i as u8)).trigger;
        if button != 0 {
            ext::gm_801677E8(i as i8);
            break;
        }
    }
    button as i32
}

pub unsafe fn Toy_80305C44() -> i32 {
    let mut button: u32 = 0;
    for i in 0..4 {
        button = (*pad(i as u8)).button;
        if button != 0 {
            ext::gm_801677E8(i as i8);
            break;
        }
    }
    button as i32
}

pub unsafe fn Toy_80305D00() -> f32 {
    let mut ret: f32 = 0.0;
    for i in 0..4 {
        let mut val = (*pad(i as u8)).nml_stickX;
        ret = val;
        if val < 0.0 {
            val = -val;
        }
        if val > 0.1 {
            break;
        }
    }
    ret
}

#[inline]
fn abs_f(v: f32) -> f32 {
    if v < 0.0 { -v } else { v }
}

pub unsafe fn Toy_80305DB0() -> f32 {
    let mut ret: f32 = 0.0;
    for i in 0..4 {
        ret = (*pad(i as u8)).nml_stickY;
        if abs_f(ret) > 0.1 {
            ext::gm_801677E8(i as i8);
            break;
        }
    }
    ret
}

pub unsafe fn Toy_80305EB4() -> f32 {
    let mut ret: f32 = 0.0;
    for i in 0..4 {
        ret = (*pad(i as u8)).nml_subStickX;
        if abs_f(ret) > 0.1 {
            ext::gm_801677E8(i as i8);
            break;
        }
    }
    ret
}

pub unsafe fn Toy_80305FB8() -> f32 {
    let mut ret: f32 = 0.0;
    for i in 0..4 {
        ret = (*pad(i as u8)).nml_subStickY;
        if abs_f(ret) > 0.1 {
            ext::gm_801677E8(i as i8);
            break;
        }
    }
    ret
}

// ---------------------------------------------------------------------------------------------
// Trophy table readers
// ---------------------------------------------------------------------------------------------

/// Trophy parameter `field` (0..5 the six position/scale floats, 6 and 7 the type bytes, 8 x04).
pub unsafe fn Toy_803060BC(trophy_id: i32, field: i32) -> f32 {
    let mut lang_flag = 0;
    let mut found_jp = 0;
    let mut jp_ptr = _Toy_sbss_804D6EC0;
    // Search JP table
    while (*jp_ptr).id != -1 {
        if (*jp_ptr).id == trophy_id {
            found_jp = 1;
            break;
        }
        jp_ptr = jp_ptr.add(1);
    }
    if (ext::lbLang_IsSettingJP() != 0 && ext::lbLang_IsSavedLanguageUS() != 0)
        || (ext::lbLang_IsSettingUS() != 0 && ext::lbLang_IsSavedLanguageJP() != 0)
    {
        lang_flag = 1;
    }
    // Search US table
    let mut us_ptr = _Toy_sbss_804D6EC4;
    while (*us_ptr).id != -1 {
        if (*us_ptr).id == trophy_id {
            break;
        }
        us_ptr = us_ptr.add(1);
    }
    let pick = |jp: f32, us: f32| if lang_flag != 0 && found_jp != 0 { jp } else { us };
    match field {
        0 => pick((*jp_ptr).x08, (*us_ptr).x08),
        1 => pick((*jp_ptr).x0C, (*us_ptr).x0C),
        2 => pick((*jp_ptr).x10, (*us_ptr).x10),
        3 => pick((*jp_ptr).x14, (*us_ptr).x14),
        4 => pick((*jp_ptr).x18, (*us_ptr).x18),
        5 => pick((*jp_ptr).x1C, (*us_ptr).x1C),
        6 => (*us_ptr).x20 as f32,
        7 => (*us_ptr).x21 as f32,
        8 => (*us_ptr).x04 as f32,
        // The C has no return here.
        _ => 0.0,
    }
}

pub unsafe fn Toy_803062BC(trophy_id: i32) -> i16 {
    let mut table = Toy_sbss_804D6EDC;
    let mut i = 0;
    while i < TY_TROPHY_COUNT {
        if trophy_id == *table as i32 {
            break;
        }
        table = table.add(1);
        i += 1;
    }
    i as i16
}

pub unsafe fn _Toy_803062EC(arg0: i32, arg1: u32, farg0: f32) {
    let mut td = _Toy_sbss_804D6EC4;
    loop {
        let id = (*td).id;
        if id == -1 || id == arg0 {
            break;
        }
        td = td.add(1);
    }
    match arg1 {
        0 => (*td).x08 = farg0,
        1 => (*td).x0C = farg0,
        2 => (*td).x10 = farg0,
        3 => (*td).x14 = farg0,
        4 => (*td).x18 = farg0,
        5 => (*td).x1C = farg0,
        _ => {}
    }
    if !_Toy_sbss_804D6E9C.is_null() {
        ext::DevText_Erase(_Toy_sbss_804D6E9C);
        ext::DevText_SetCursorXY(_Toy_sbss_804D6E9C, 0, 0);
        let s = format!(
            "X   {:3.2}\nY   {:3.2}\nZ   {:3.2}\nMS  {:3.2}\nSS  {:3.2}\nMD  {:3.2}\0",
            (*td).x08 as f64,
            (*td).x0C as f64,
            (*td).x10 as f64,
            (*td).x14 as f64,
            (*td).x18 as f64,
            (*td).x1C as f64
        );
        ext::DevText_Print(_Toy_sbss_804D6E9C, s.as_ptr() as *mut i8);
    }
}

#[inline]
unsafe fn getDataiDatFilename() -> *mut i8 {
    if ext::lbLang_IsSavedLanguageJP() != 0 {
        cstr!("TyDatai.dat")
    } else {
        cstr!("TyDatai.usd")
    }
}

pub unsafe fn Toy_803063D4(arg0: i32, arg1: i32, arg2: i32) -> i32 {
    let mut result = arg0 + arg1 + 1;
    if ext::lbLang_IsSettingJP() != 0 && ext::lbLang_IsSavedLanguageUS() != 0 {
        let mut ptr = _Toy_sbss_804D6EB8 as *mut i16;
        let mut i = 0;
        while *ptr != -1 {
            if *ptr as i32 == arg0 {
                result = i + arg2 + 1;
                break;
            }
            ptr = ptr.add(1);
            i += 1;
        }
    } else if ext::lbLang_IsSettingUS() != 0 && ext::lbLang_IsSavedLanguageJP() != 0 {
        let mut ptr = _Toy_sbss_804D6EB8 as *mut i16;
        let mut i = 0;
        while *ptr != -1 {
            if *ptr as i32 == arg0 {
                result = i + arg2 + 1;
                break;
            }
            ptr = ptr.add(1);
            i += 1;
        }
    }
    result
}

pub unsafe fn _Toy_803064B8(arg0: i16, arg1: i8) -> i16 {
    let data = _Toy_sbss_804D6EBC as *mut ToyNameData;
    let d = &*data.offset(arg0 as isize);
    match arg1 {
        0 => d.x0,
        1 => d.x2,
        2 => {
            if ext::lbLang_IsSettingJP() != 0 {
                if ext::lbLang_IsSavedLanguageJP() != 0 {
                    return d.x4;
                }
                return d.x8;
            }
            if ext::lbLang_IsSavedLanguageUS() != 0 {
                return d.x4;
            }
            d.x8
        }
        3 => {
            if ext::lbLang_IsSettingJP() != 0 {
                if ext::lbLang_IsSavedLanguageJP() != 0 {
                    return d.x6;
                }
                return d.xA;
            }
            if ext::lbLang_IsSavedLanguageUS() != 0 {
                return d.x6;
            }
            d.xA
        }
        _ => arg0,
    }
}

pub unsafe fn _Toy_8030663C() {
    let ptr = toy_u16(5);
    let mut count: i32 = 0;
    let mut offset: isize = 0;
    let mut dst = _Toy_sbss_804D6E64;
    for i in 0..TY_TROPHY_COUNT {
        let src = if toy_scene_mode() { ptr } else { ext::gmMainLib_GetTrophyFlags() as *mut u16 };
        if (*((src as *mut u8).offset(offset) as *mut u16) & 0xFF) != 0 {
            (*dst).key[0] = _Toy_803064B8(i as i16, 0);
            count += 1;
            dst = dst.add(1);
        }
        offset += 2;
    }
    {
        let mut dst = _Toy_sbss_804D6E64;
        for j in 0..TY_TROPHY_COUNT {
            let mut src = _Toy_sbss_804D6E64;
            let mut k = 0;
            while k < count {
                if (*src).key[0] == _Toy_803064B8(j as i16, 1) {
                    (*dst).key[1] = (*src).key[0];
                    dst = dst.add(1);
                    break;
                }
                src = src.add(1);
                k += 1;
            }
        }
    }
    {
        let mut dst = _Toy_sbss_804D6E64;
        for j in 0..TY_TROPHY_COUNT {
            let mut src = _Toy_sbss_804D6E64;
            let mut k = 0;
            while k < count {
                if ext::lbLang_IsSavedLanguageJP() != 0 {
                    if (*src).key[0] == _Toy_803064B8(j as i16, 2) {
                        (*dst).key[2] = (*src).key[0];
                        dst = dst.add(1);
                        break;
                    }
                } else {
                    let tmp = src;
                    if (*tmp).key[0] == _Toy_803064B8(j as i16, 3) {
                        (*dst).key[2] = (*tmp).key[0];
                        dst = dst.add(1);
                        break;
                    }
                }
                src = src.add(1);
                k += 1;
            }
        }
    }
}

/// Copy sort key column `arg0` of the sort table into `Toy_sbss_804D6EDC` (`arg1 == 0`
/// ascending, otherwise descending).
pub unsafe fn Toy_803067BC(arg0: i32, arg1: i32) {
    let keys = (*_Toy_sbss_804D6E64).key.as_mut_ptr() as *mut i16;
    if arg1 == 0 {
        let src = keys.offset(arg0 as isize);
        let mut i = 0;
        while i < *ext::gmMainLib_GetTrophyCount() as i32 {
            *Toy_sbss_804D6EDC.offset(i as isize) = *src.offset((i * TY_SORT_KEY_COUNT) as isize);
            i += 1;
        }
        return;
    }
    let mut count = *ext::gmMainLib_GetTrophyCount() as i32;
    let mut src = keys.offset(arg0 as isize);
    let mut dest = Toy_sbss_804D6EDC.offset(count as isize);
    while count != 0 {
        count -= 1;
        *dest = *src;
        dest = dest.sub(1);
        src = src.add(TY_SORT_KEY_COUNT as usize);
    }
}

// ---------------------------------------------------------------------------------------------
// Scene render callbacks and anim helpers
// ---------------------------------------------------------------------------------------------

pub unsafe extern "C" fn Toy_803068E0(gobj: *mut HSD_GObj, _unused: i32) {
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj) != 0 {
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_CObjEndCurrent();
        ext::HSD_FogSet(core::ptr::null_mut());
    }
}

pub unsafe extern "C" fn Toy_80306930(gobj: *mut HSD_GObj, _unused: i32) {
    ext::HSD_FogSet((*gobj).hsd_obj as *mut HSD_Fog);
}

pub unsafe extern "C" fn Toy_80306954(gobj: *mut HSD_GObj, _unused: i32) {
    let state = Toy_sbss_804D6ED4;
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj) != 0 {
        if _Toy_sbss_804D6E50 == 0 {
            let light_idx = (*state).x10;
            let v = &_Toy_803FDDE4.values[light_idx as usize];
            if v.flag != 0 {
                ext::HSD_SetEraseColor(v.color.r, v.color.g, v.color.b, v.color.a);
                ext::HSD_CObjEraseScreen((*gobj).hsd_obj as *mut HSD_CObj, 1, 0, 0);
            }
        }
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_FogSet(core::ptr::null_mut());
        ext::HSD_CObjEndCurrent();
    }
}

pub unsafe extern "C" fn _Toy_80306A0C(arg0: *mut HSD_GObj, _unused: i32) {
    let fog = (*arg0).hsd_obj as *mut HSD_Fog;
    if _Toy_sbss_804D6E54 == 0 {
        ext::HSD_FogSet(core::ptr::null_mut());
    } else {
        ext::HSD_FogSet(fog);
    }
}

pub unsafe fn Toy_80306A48(
    jobj: *mut HSD_JObj,
    symbol1: *const i8,
    symbol2: *const i8,
    symbol3: *const i8,
    archive: *mut HSD_Archive,
    frame: i32,
) {
    let anim = if !symbol1.is_null() { ext::HSD_ArchiveGetPublicAddress(archive, symbol1) } else { core::ptr::null_mut() };
    let matanim = if !symbol2.is_null() { ext::HSD_ArchiveGetPublicAddress(archive, symbol2) } else { core::ptr::null_mut() };
    let shapeanim = if !symbol3.is_null() { ext::HSD_ArchiveGetPublicAddress(archive, symbol3) } else { core::ptr::null_mut() };
    ext::HSD_JObjAddAnimAll(jobj, anim as *mut HSD_AnimJoint, matanim as *mut HSD_MatAnimJoint, shapeanim as *mut HSD_ShapeAnimJoint);
    ext::HSD_JObjReqAnimAll(jobj, frame as f32);
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe fn Toy_80306B18(gobj: *mut HSD_GObj, anim_frame: i32, val1: i32, val2: i32) -> *mut HSD_GObjProc {
    let mut proc_ = core::ptr::null_mut();
    let tp = (*gobj).user_data as *mut Toy;
    if !tp.is_null() {
        ext::HSD_JObjClearFlagsAll((*gobj).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
        ext::HSD_JObjReqAnimAll((*gobj).hsd_obj as *mut HSD_JObj, anim_frame as f32);
        (*tp).x8 = val1;
        (*tp).x4 = val2;
        proc_ = ext::HSD_GObj_SetupProc(gobj, Some(Toy_80306BB8), 0);
        ext::HSD_GObj_80390CD4(gobj);
    }
    proc_
}

pub unsafe extern "C" fn Toy_80306BB8(gobj: *mut HSD_GObj) {
    let tp = (*gobj).user_data as *mut Toy;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if !tp.is_null() {
        let old = (*tp).x8;
        (*tp).x8 = old.wrapping_sub(1);
        if old != 0 {
            ext::HSD_JObjAnimAll(jobj);
        } else {
            (*tp).x8 = 0;
            if (*tp).x4 != 0 {
                ext::HSD_JObjClearFlagsAll((*gobj).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
            }
            ext::HSD_GObjProc_RemoveProc(ext::HSD_GObj_CurrentInvokedProc);
        }
    } else {
        if ext::lb_8000B09C(jobj) == 0 {
            ext::HSD_JObjReqAnimAll(jobj, 0.0);
        }
        ext::HSD_JObjAnimAll(jobj);
    }
}

pub unsafe extern "C" fn _Toy_80306C5C(arg0: *mut HSD_GObj) {
    let data = Toy_sbss_804D6ED4;
    let gobj = (*data).x04;
    let mut lobj = (*gobj).hsd_obj as *mut HSD_LObj;
    let mut i = 0usize;
    while !lobj.is_null() {
        ext::HSD_LObjSetPosition(lobj, &mut (*data).positions[i]);
        ext::HSD_LObjSetInterest(lobj, &mut (*data).interests[i]);
        i += 1;
        lobj = HSD_LObjGetNext(lobj.as_ref());
    }
    ext::HSD_LObjAnimAll((*arg0).hsd_obj as *mut HSD_LObj);
}

pub unsafe extern "C" fn Toy_RemoveUserData(ptr: *mut c_void) {
    ext::HSD_Free(ptr);
}

pub unsafe fn Toy_80306D14() {
    let state = toy_mode_state();
    if (*state).x0 == 1 {
        ext::lbAudioAx_800237A8(0xAA, 0x7F, 0x40);
    } else if (*state).x0 == 2 {
        ext::lbAudioAx_800237A8(0xAB, 0x7F, 0x40);
    }
}

pub unsafe fn Toy_80306D70(arg0: i32) {
    let mut sp14: *mut *mut LightList = core::ptr::null_mut();
    // THE LIGHT TABLE IS NOT WHERE THE CAST SAYS IT IS (PC): retail reaches this table by
    // aliasing the string symbol preceding it in .data; here the real symbol is named.
    let idx = _Toy_803FDDE4.values[arg0 as usize].index;
    let sym = _Toy_803FDDE4.symbols[idx as usize].name;
    ext::OSReport(&format!("ty: light set {} -> idx {} {}\n", arg0, idx, if !sym.is_null() { c_str(sym) } else { String::from("(null)") }));
    let data = Toy_sbss_804D6ED4;
    if !(*data).archive.is_null() && !(*data).x04.is_null() {
        ext::HSD_GObjProc_RemoveAllProcs((*data).x04);
        ext::HSD_GObjFree((*data).x04);
        (*data).x04 = core::ptr::null_mut();
        sp14 = ext::HSD_ArchiveGetPublicAddress((*data).archive, sym) as *mut *mut LightList;
    } else {
        (*data).archive =
            ext::lbArchive_80016DBC(_Toy_str_TyLight_dat.as_ptr() as *const i8, (&raw mut sp14) as *mut c_void, sym, 0);
    }
    if !sp14.is_null() {
        let mut sp_c: i32 = 0;
        (*data).x04 = ext::GObj_Create(2, 1, 0);
        let lobj = Toy_LoadLObjList(sp14, &mut sp_c);
        let kind = ext::HSD_GObj_LightKind as u8;
        ext::HSD_GObjObject_80390A70((*data).x04, kind, lobj as *mut c_void);
        ext::GObj_SetupGXLink((*data).x04, Some(ext::cb::HSD_GObj_LObjCallback), 0x37, 0);
        if sp_c != 0 {
            ext::HSD_GObj_SetupProc((*data).x04, Some(_Toy_80306C5C), 0);
            ext::HSD_GObj_80390CD4((*data).x04);
        }
    } else {
        ext::OSReport(&format!("*** Can not Load Light Label({})\n", c_str(sym)));
        hsd_assert_fail(2253);
    }
}

pub unsafe fn Toy_LoadLObjList(list: *mut *mut LightList, has_anim: *mut i32) -> *mut HSD_LObj {
    let mut list = list;
    let mut prev: *mut HSD_LObj = core::ptr::null_mut();
    let mut first: *mut HSD_LObj = core::ptr::null_mut();
    let base = Toy_sbss_804D6ED4;
    let mut idx = 0usize;
    if !has_anim.is_null() {
        *has_anim = 0;
    }
    while !(*list).is_null() {
        let lobj = ext::HSD_LObjLoadDesc((**list).desc);
        if !lobj.is_null() {
            let anim_flag = &mut (*base).has_position_anim[idx] as *mut i8;
            let anims = (**list).anims;
            *anim_flag = 0;
            if !anims.is_null() && !(*anims).is_null() {
                if !has_anim.is_null() {
                    *has_anim = 1;
                }
                ext::HSD_LObjAddAnimAll(lobj, *anims);
                ext::HSD_LObjReqAnimAll(lobj, 0.0);
                if !(**anims).position_anim.is_null() {
                    *anim_flag = 1;
                }
            }
            ext::HSD_LObjGetPosition(lobj, &mut (*base).positions[idx]);
            ext::HSD_LObjGetInterest(lobj, &mut (*base).interests[idx]);
            idx += 1;
        }
        if !prev.is_null() {
            HSD_LObjSetNext(prev.as_mut(), lobj);
        } else {
            first = lobj;
        }
        prev = lobj;
        list = list.add(1);
    }
    first
}

pub unsafe fn _Toy_80307018() {
    let ptr1 = Toy_sbss_804D6ED8;
    let ptr2 = Toy_sbss_804D6ED4 as *mut tyUnkStruct;
    if (*ptr1).archive.is_null() {
        ext::OSReport("*** BG data aren't being loaded!\n");
        hsd_assert_fail(2322);
    }
    let lights = ext::HSD_ArchiveGetPublicAddress((*ptr1).archive, cstr!("ScMenFigure_scene_lights")) as *mut *mut LightList;
    if !lights.is_null() {
        (*ptr2).x0 = ext::GObj_Create(2, 3, 0);
        let obj = Toy_LoadLObjList(lights, core::ptr::null_mut());
        let kind = ext::HSD_GObj_LightKind as u8;
        ext::HSD_GObjObject_80390A70((*ptr2).x0, kind, obj as *mut c_void);
        ext::GObj_SetupGXLink((*ptr2).x0, Some(ext::cb::HSD_GObj_LObjCallback), 0x36, 0);
        if _Toy_sbss_804D6E50 != 0 {
            let mut fog_desc = _Toy_803B8844;
            let fog = ext::HSD_FogLoadDesc(&mut fog_desc);
            (*ptr2).x8 = ext::GObj_Create(3, 4, 0);
            let kind = ext::HSD_GObj_FogKind as u8;
            ext::HSD_GObjObject_80390A70((*ptr2).x8, kind, fog as *mut c_void);
            ext::GObj_SetupGXLink((*ptr2).x8, Some(_Toy_80306A0C), 0x35, 0);
        }
    }
}

/// `data` is `Toy_sbss_804D6E68` viewed as the camera record (`TyCameraData_`); the two
/// declarations name the same GameCube block, so this reads it through `Toy6E68`.
pub unsafe fn _Toy_8030715C(cstick_x: f32, cstick_y: f32) {
    let mut euler = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut light_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut interest = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut new_interest = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut mtx: Mtx = core::mem::zeroed();
    let mut up_vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut left_vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut angles = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let data = _Toy_sbss_804D6E68;
    let data2 = Toy_sbss_804D6ED4;
    let cobj = (*((*data).x8 as *mut HSD_GObj)).hsd_obj as *mut HSD_CObj;
    ext::HSD_CObjGetInterest(cobj, &mut interest);
    ext::HSD_CObjGetInterest(cobj, &mut new_interest);
    ext::HSD_CObjGetUpVector(cobj, &mut up_vec);
    ext::PSVECScale(&mut up_vec, &mut up_vec, -cstick_y);
    ext::PSVECAdd(&mut up_vec, &mut new_interest, &mut new_interest);
    ext::HSD_CObjGetLeftVector(cobj, &mut left_vec);
    ext::PSVECScale(&mut left_vec, &mut left_vec, cstick_x);
    ext::PSVECAdd(&mut left_vec, &mut new_interest, &mut new_interest);
    if new_interest.x <= -3000.0
        || new_interest.y <= -3000.0
        || new_interest.z <= -3000.0
        || new_interest.x >= 3000.0
        || new_interest.y >= 3000.0
        || new_interest.z >= 3000.0
    {
        return;
    }
    ext::HSD_CObjSetInterest(cobj, &mut new_interest);
    euler.x = 0.0;
    euler.y = 0.0;
    euler.z = (*data).x20;
    ext::MTXRotRad(mtx.as_mut_ptr() as *mut c_void, b'x' as i8, 0.017453292f32 * (*data).x18);
    ext::PSMTXMultVecSR(mtx.as_mut_ptr() as *mut c_void, &mut euler, &mut euler);
    ext::MTXRotRad(mtx.as_mut_ptr() as *mut c_void, b'y' as i8, 0.017453292f32 * -(*data).x1C);
    ext::PSMTXMultVecSR(mtx.as_mut_ptr() as *mut c_void, &mut euler, &mut euler);
    euler.x += new_interest.x;
    euler.y += new_interest.y;
    euler.z += new_interest.z;
    ext::HSD_CObjSetEyePosition(cobj, &mut euler);
    let mut i = 0usize;
    let mut lobj = (*(*data2).x04).hsd_obj as *mut HSD_LObj;
    while !lobj.is_null() {
        let flag_ptr = &mut (*data2).has_position_anim[i] as *mut i8;
        angles.x = 0.017453292f32 * (*data2).x14;
        angles.y = 0.017453292f32 * -(*data2).x18;
        angles.z = 0.0;
        if *flag_ptr != 0 {
            ext::HSD_LObjGetPosition(lobj, &mut light_pos);
        } else {
            light_pos = (*data2).positions[i];
        }
        if *flag_ptr != 0 {
            (*data2).positions[i] = light_pos;
        }
        ext::lbVector_ApplyEulerRotation(&mut light_pos, &mut angles);
        ext::HSD_LObjSetPosition(lobj, &mut light_pos);
        if ext::HSD_LObjGetInterest(lobj, &mut light_pos) != 0 {
            if *flag_ptr != 0 {
                (*data2).interests[i] = light_pos;
            } else {
                light_pos = (*data2).interests[i];
            }
            ext::lbVector_ApplyEulerRotation(&mut light_pos, &mut angles);
            ext::HSD_LObjSetInterest(lobj, &mut light_pos);
        }
        i += 1;
        lobj = HSD_LObjGetNext(lobj.as_ref());
    }
}

// ---------------------------------------------------------------------------------------------
// Scene panels, backgrounds, stand models
// ---------------------------------------------------------------------------------------------

/// `Toy_AddPanelAnims` (static inline).
#[inline]
unsafe fn Toy_AddPanelAnims(
    jobj: *mut HSD_JObj,
    shapanim: *mut HSD_ShapeAnimJoint,
    matanim: *mut HSD_MatAnimJoint,
    anim: *mut HSD_AnimJoint,
) {
    ext::HSD_JObjAddAnimAll(jobj, anim, matanim, shapanim);
}

/// `Toy_RenderPanelGuarded` (TARGET_PC): the Atlas retail mask hides the Gallery panel.
pub unsafe extern "C" fn Toy_RenderPanelGuarded(gobj: *mut HSD_GObj, pass: i32) {
    if ext::tyList_PcActive() == 0 && ext::Ui_RetailHidden(AT_RE_TOY_PANEL) != 0 {
        return;
    }
    ext::HSD_GObj_JObjCallback(gobj, pass);
}

/// `Toy_RenderInfoGuarded` (TARGET_PC): the Atlas retail mask hides the Gallery info frame.
pub unsafe extern "C" fn Toy_RenderInfoGuarded(gobj: *mut HSD_GObj, pass: i32) {
    if ext::tyList_PcActive() == 0 && ext::Ui_RetailHidden(AT_RE_TOY_INFO) != 0 {
        return;
    }
    ext::HSD_SObjLib_803A49E0(gobj, pass);
}

/// `ToyED8Data*` viewed as the `GObj` that `ToyDataX8` describes: its `x28` is `hsd_obj`, the
/// `SObj` list (`ToyDataJObj` / `ToyJObjNode`: `x4` = next, `x40` = flags).
#[inline]
unsafe fn ed8_x8_sobj() -> *mut HSD_SObj {
    (*((*Toy_sbss_804D6ED8).x8 as *mut HSD_GObj)).hsd_obj as *mut HSD_SObj
}

pub unsafe fn Toy_80307470(arg0: i32) {
    let tg = Toy_sbss_804D6ED8;
    if (*tg).archive.is_null() {
        ext::OSReport("*** BG data aren't being loaded!\n");
        hsd_assert_fail(2481);
    }
    // `tg->x0` (the panel gobj) is the first slot of the ED8 record.
    if !(*tg).x0.is_null() {
        ext::HSD_GObjFree((*tg).x0 as *mut HSD_GObj);
        (*tg).x0 = core::ptr::null_mut();
    }
    // `.data` aliasing of the retail build resolved to the real table (see Toy_80306D70).
    let label = &raw mut _Toy_803FDEA0[arg0 as usize];
    let joint = ext::HSD_ArchiveGetPublicAddress((*tg).archive, *label) as *mut HSD_Joint;
    if !joint.is_null() {
        let gobj = ext::GObj_Create(9, 9, 0);
        (*tg).x0 = gobj as *mut *mut HSD_JObj;
        let loaded_jobj = ext::HSD_JObjLoadJoint(joint);
        let names = &_Toy_803FDF3C;
        let anim = ext::HSD_ArchiveGetPublicAddress((*tg).archive, names.animjoint) as *mut HSD_AnimJoint;
        let matanim = ext::HSD_ArchiveGetPublicAddress((*tg).archive, names.matanim_joint) as *mut HSD_MatAnimJoint;
        Toy_AddPanelAnims(
            loaded_jobj,
            ext::HSD_ArchiveGetPublicAddress((*tg).archive, names.shapeanim_joint) as *mut HSD_ShapeAnimJoint,
            matanim,
            anim,
        );
        ext::HSD_JObjReqAnimAll(loaded_jobj, 0.0);
        let kind = ext::HSD_GObj_JObjKind;
        ext::HSD_GObjObject_80390A70(gobj, kind, loaded_jobj as *mut c_void);
        ext::GObj_SetupGXLink(gobj, Some(Toy_RenderPanelGuarded), 0x3C, 0);
        // `lb_8001204C(jobj, (HSD_JObj**) &tg->x10, ids, 9)` fills nine consecutive pointer slots
        // from GameCube +0x10; the native record keeps `jobjs[0..3]` (+0x18..) and `x30` (+0x30).
        let mut out: [*mut HSD_JObj; 9] = [core::ptr::null_mut(); 9];
        ext::lb_8001204C(loaded_jobj, out.as_mut_ptr(), _Toy_803FE3F8.as_ptr() as *mut u16, 9);
        (*tg).jobjs = [out[2], out[3], out[4]];
        (*tg).x30 = out[8];
        Toy_803083D8((*tg).x30, 0x3E7);
        return;
    }
    ext::OSReport(&format!("*** Can not Load Panel Label({})\n", c_str(*label)));
    hsd_assert_fail(2534);
}

pub unsafe fn _Toy_803075E8(arg0: i32) {
    let td = Toy_sbss_804D6ED8;
    if (*td).archive.is_null() {
        ext::OSReport("*** BG data aren't being loaded!\n");
        hsd_assert_fail(2625);
    }
    if !(*td).gobj.is_null() {
        ext::HSD_GObjProc_RemoveAllProcs((*td).gobj);
        ext::HSD_GObjFree((*td).gobj);
        (*td).gobj = core::ptr::null_mut();
    }
    if (*td).x54 != 0 {
        let s0 = ed8_x8_sobj();
        (*s0).x40 = 9;
        (*(*s0).next).x40 = 9;
        (*(*(*s0).next).next).x40 = 9;
    }
    let ptr = &raw mut _Toy_803FDEBC[arg0 as usize];
    if !(*ptr).is_null() {
        let joint = ext::HSD_ArchiveGetPublicAddress((*td).archive, *ptr) as *mut HSD_Joint;
        if !joint.is_null() {
            (*td).gobj = ext::GObj_Create(4, 7, 0);
            let jobj = ext::HSD_JObjLoadJoint(joint);
            let kind = ext::HSD_GObj_JObjKind;
            ext::HSD_GObjObject_80390A70((*td).gobj, kind, jobj as *mut c_void);
            ext::GObj_SetupGXLink((*td).gobj, Some(ext::cb::HSD_GObj_JObjCallback), 0x33, 0);
            let names = &_Toy_803FDFA8[arg0 as usize];
            let joint = ext::HSD_ArchiveGetPublicAddress((*td).archive, names.animjoint);
            let data = ext::HSD_ArchiveGetPublicAddress((*td).archive, names.matanim_joint);
            let shapanim = ext::HSD_ArchiveGetPublicAddress((*td).archive, names.shapeanim_joint);
            if !joint.is_null() || !data.is_null() || !shapanim.is_null() {
                ext::HSD_JObjAddAnimAll(
                    jobj,
                    joint as *mut HSD_AnimJoint,
                    data as *mut HSD_MatAnimJoint,
                    shapanim as *mut HSD_ShapeAnimJoint,
                );
                ext::HSD_JObjReqAnimAll(jobj, 0.0);
                ext::HSD_JObjAnimAll(jobj);
                ext::HSD_GObj_SetupProc((*td).gobj, Some(Toy_80306BB8), 0);
                ext::HSD_GObj_80390CD4((*td).gobj);
            }
        } else {
            ext::OSReport(&format!("*** Can not Load Back Label({})\n", c_str(*ptr)));
            hsd_assert_fail(2677);
        }
    } else if (*td).x54 != 0 {
        let mut tdjobj = ed8_x8_sobj();
        match arg0 {
            2 => {}
            4 => tdjobj = (*tdjobj).next,
            5 => tdjobj = (*(*tdjobj).next).next,
            _ => {}
        }
        (*tdjobj).x40 = 8;
    }
}

pub unsafe fn _Toy_80307828(arg0: i32) {
    let data = _Toy_sbss_804D6E68;
    let cobj = (*((*data).x8 as *mut HSD_GObj)).hsd_obj as *mut HSD_CObj;
    let data2 = Toy_sbss_804D6ED4;
    let mut interest = _Toy_803B8858;
    if arg0 == 0 {
        (*data).x58 = 0;
        (*data).x1C = 0.0;
        (*data).x18 = 0.0;
        (*data2).x18 = 0.0;
        (*data2).x14 = 0.0;
    } else {
        (*data).x18 = 0.0;
        (*data2).x14 = 0.0;
    }
    (*data).x20 = 38.0;
    (*data).x2C = 0.0;
    (*data).x28 = 0.0;
    (*data).x24 = 0.0;
    ext::HSD_CObjSetInterest(cobj, &mut interest);
}

pub unsafe fn _Toy_803078E4() {
    // `Toy_sbss_804D6ED8` as `tyLightData`: x0C = ED8.gobj2, x58 = ED8.x58.
    let data = Toy_sbss_804D6ED8;
    let mut syms: [*mut c_void; 7] = [core::ptr::null_mut(); 7];
    let pos_en = _Toy_803B8864;
    let pos_jp = _Toy_803B889C;
    if (*data).x58.is_null() {
        let filename = if ext::lbLang_IsSavedLanguageJP() != 0 { cstr!("TyMnInfo.dat") } else { cstr!("TyMnInfo.usd") };
        let s = syms.as_mut_ptr();
        let names = &raw const _Toy_803FE108;
        let n = |i: usize| (*names)[i] as *mut c_void;
        (*data).x58 = ext::lbArchive_LoadSymbols(
            filename,
            &[
                s.add(0) as *mut c_void, n(0),
                s.add(1) as *mut c_void, n(1),
                s.add(2) as *mut c_void, n(2),
                s.add(3) as *mut c_void, n(3),
                s.add(4) as *mut c_void, n(4),
                s.add(5) as *mut c_void, n(5),
                s.add(6) as *mut c_void, n(6),
            ],
        ) as *mut c_void;
        (*data).gobj2 = ext::GObj_Create(5, 6, 0);
        ext::GObj_SetupGXLink((*data).gobj2, Some(Toy_RenderInfoGuarded), 0x38, 0);
        for i in 0..7usize {
            let sobj = ext::HSD_SObjLib_803A477C((*data).gobj2, syms[i] as *mut HSD_SObjDesc, 0, 0, 0x80, 0);
            if !sobj.is_null() {
                if ext::lbLang_IsSavedLanguageJP() != 0 {
                    (*sobj).x10 = pos_jp.a[i].xy[0] as f32;
                    (*sobj).x14 = pos_jp.a[i].xy[1] as f32;
                } else {
                    (*sobj).x10 = pos_en.a[i].xy[0] as f32;
                    (*sobj).x14 = pos_en.a[i].xy[1] as f32;
                }
            }
        }
    }
}

pub unsafe fn _Toy_80307BA0(parent_jobj: *mut HSD_JObj, arg1: i16) -> *mut HSD_JObj {
    let state = &raw mut Toy_804A2AA8;
    let mut joint_data: *mut c_void = core::ptr::null_mut();
    if Toy_sbss_804D6EC8.is_null() {
        Toy_sbss_804D6EC8 = ext::lbArchive_LoadSymbols(
            cstr!("TyStand.dat"),
            &[(&raw mut joint_data) as *mut c_void, cstr!("ToyStandModel_TopN_joint") as *mut c_void],
        );
    } else {
        joint_data = ext::HSD_ArchiveGetPublicAddress(Toy_sbss_804D6EC8, cstr!("ToyStandModel_TopN_joint"));
    }
    let jobj = ext::HSD_JObjLoadJoint(joint_data as *mut HSD_Joint);
    ext::HSD_JObjAddChild(parent_jobj, jobj);
    {
        let idx: i32 = if !(*state).jobj[0].is_null() { 1 } else { 0 };
        (*state).x0E = idx as i8;
        (*state).jobj[(*state).x0E as usize] = jobj;
    }
    HSD_JObjAddTranslationY(&mut *jobj, 0.25, &mut Dirty);
    (*state).x11 = 2;
    (*state).x10 = 2;
    (*state).x0F = 0;
    _Toy_80307F64(2, 0);
    let scale_val = Toy_803060BC(arg1 as i32, 4);
    HSD_JObjSetScaleX(&mut *jobj, scale_val, &mut Dirty);
    HSD_JObjSetScaleY(&mut *jobj, scale_val, &mut Dirty);
    HSD_JObjSetScaleZ(&mut *jobj, scale_val, &mut Dirty);
    jobj
}

pub unsafe extern "C" fn Toy_80307E84(gobj: *mut HSD_GObj) {
    let state = &raw mut Toy_804A2AA8;
    let idx = (*state).x0E;
    let x0f_val = (*state).x0F;
    let jobj0 = (*state).jobj[idx as usize];
    let jobj1 = (*state).jobj[(idx ^ 1) as usize];
    if x0f_val <= 0 {
        if (*state).x10 == 1 {
            ext::HSD_JObjSetFlagsAll(jobj0, JOBJ_HIDDEN as u32);
            ext::HSD_JObjSetFlagsAll(jobj1, JOBJ_HIDDEN as u32);
        }
        (*state).x10 = 0;
        ext::HSD_JObjRemoveAnimAll(jobj0);
        ext::HSD_JObjRemoveAnimAll(jobj1);
        ext::HSD_GObjProc_RemoveAllProcs(gobj);
    } else {
        (*state).x0F = (*state).x0F - 1;
        ext::HSD_JObjAnimAll(jobj0);
        ext::HSD_JObjAnimAll(jobj1);
    }
}

pub unsafe fn _Toy_80307F64(arg0: i32, arg1: i32) {
    let state = &raw mut Toy_804A2AA8;
    let idx = (*state).x0E;
    let jobj1 = (*state).jobj[idx as usize];
    let jobj2 = (*state).jobj[(idx ^ 1) as usize];
    if (*state).x0F == 0 {
        if arg1 != 0 {
            if arg0 != (*state).x11 as i32 {
                ext::HSD_JObjRemoveAnimAll(jobj1);
                ext::HSD_JObjRemoveAnimAll(jobj2);
                (*state).x11 = arg0 as i8;
                (*state).x10 = arg0 as i8;
                if arg0 == 1 {
                    Toy_80306A48(jobj1, core::ptr::null(), _Toy_803FE244[0].matanim_joint, core::ptr::null(), Toy_sbss_804D6EC8, 0);
                    Toy_80306A48(jobj2, core::ptr::null(), _Toy_803FE244[0].matanim_joint, core::ptr::null(), Toy_sbss_804D6EC8, 0);
                } else {
                    Toy_80306A48(jobj1, core::ptr::null(), _Toy_803FE244[1].matanim_joint, core::ptr::null(), Toy_sbss_804D6EC8, 0);
                    Toy_80306A48(jobj2, core::ptr::null(), _Toy_803FE244[1].matanim_joint, core::ptr::null(), Toy_sbss_804D6EC8, 0);
                }
                (*state).x0F = 0xA;
                ext::HSD_GObj_SetupProc((*state).gobj, Some(Toy_80307E84), 0);
                ext::HSD_GObj_80390CD4((*state).gobj);
            }
        } else {
            if arg0 == 1 {
                Toy_80306A48(jobj1, core::ptr::null(), _Toy_803FE244[0].matanim_joint, core::ptr::null(), Toy_sbss_804D6EC8, 10);
                Toy_80306A48(jobj2, core::ptr::null(), _Toy_803FE244[0].matanim_joint, core::ptr::null(), Toy_sbss_804D6EC8, 10);
            } else {
                Toy_80306A48(jobj1, core::ptr::null(), _Toy_803FE244[1].matanim_joint, core::ptr::null(), Toy_sbss_804D6EC8, 10);
                Toy_80306A48(jobj2, core::ptr::null(), _Toy_803FE244[1].matanim_joint, core::ptr::null(), Toy_sbss_804D6EC8, 10);
            }
            ext::HSD_JObjRemoveAnimAll(jobj1);
            ext::HSD_JObjRemoveAnimAll(jobj2);
            (*state).x0F = 0;
        }
    }
}

pub unsafe fn Toy_8030813C(trophy_id: i32) -> *mut ToyModelFile {
    let id = trophy_id;
    let mut found = 0;
    let mut ptr: *mut ToyModelFile = core::ptr::null_mut();
    if ext::lbLang_IsSettingUS() != 0 {
        ptr = _Toy_sbss_804D6EA4;
        let cur = (*ptr).trophy_id;
        if cur == id {
            found = 1;
        } else {
            ptr = ptr.add(1);
            if (*ptr).trophy_id == id {
                found = 1;
            } else {
                ptr = ptr.add(1);
                if (*ptr).trophy_id == id {
                    found = 1;
                } else {
                    ptr = ptr.add(1);
                    if (*ptr).trophy_id == id {
                        found = 1;
                    } else {
                        ptr = ptr.add(1);
                        if (*ptr).trophy_id == id {
                            found = 1;
                        } else {
                            ptr = ptr.add(1);
                        }
                    }
                }
            }
        }
    }
    if found == 0 {
        ptr = _Toy_sbss_804D6EA8;
        let mut i = TY_TROPHY_COUNT;
        while i != 0 {
            if (*ptr).trophy_id == id {
                found = 1;
                break;
            }
            ptr = ptr.add(1);
            i -= 1;
        }
    }
    if found == 0 {
        ext::OSReport(&format!("**** Not Found Toy Model!({})\n", id));
        hsd_assert_fail(2979);
    }
    ptr
}

/// `setupTrophyEntry` (static inline).
#[inline]
unsafe fn setupTrophyEntry(entry: *mut ToyListEntry, trophy_idx: i16) {
    let result = Toy_8030813C(trophy_idx as i32);
    if !(*entry).archive.is_null() {
        ext::lbArchive_80016EFC((*entry).archive);
        (*entry).archive = core::ptr::null_mut();
    }
    (*entry).archive_name = (*result).archive_name.as_mut_ptr();
    (*entry).symbol_name = (*result).symbol_name.as_mut_ptr();
    (*entry).trophy_id = trophy_idx;
}

pub unsafe fn Toy_80308250(arg0: *mut ToyListEntry, arg1: i16, arg2: i32) {
    let mut sym: *mut c_void = core::ptr::null_mut();
    setupTrophyEntry(arg0, arg1);
    if arg2 == 0 {
        (*arg0).archive =
            ext::lbArchive_LoadSymbols((*arg0).archive_name, &[(&raw mut sym) as *mut c_void, (*arg0).symbol_name as *mut c_void]);
    }
}

pub unsafe fn Toy_803082F8(idx: i16) -> i32 {
    Toy_803063D4(Toy_80308354(idx) as i16 as i32, 2, 0x128)
}

pub unsafe fn Toy_80308328(idx: i32) -> i32 {
    let id = idx as i16;
    Toy_803063D4(id as i32, 2, 0x128)
}

pub unsafe fn Toy_80308354(idx: i16) -> i32 {
    let target = *Toy_sbss_804D6EDC.offset(idx as isize) as i32;
    let mut entry = _Toy_sbss_804D6EC4;
    let mut i = 0;
    while i < TY_TROPHY_COUNT {
        if target == (*entry).id {
            break;
        }
        entry = entry.add(1);
        i += 1;
    }
    if i == TY_TROPHY_COUNT {
        ext::OSReport(&format!("*** Error : Not Found Model Name!(To Idx {})\n", target));
        hsd_assert_fail(3114);
    }
    target
}

pub unsafe fn Toy_803083D8(jobj: *mut HSD_JObj, arg1: i32) {
    if arg1 == 0x3E6 {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    if arg1 == 0x3E7 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    let temp_r31 = Toy_803060BC(arg1, 8) as i32;
    if temp_r31 == 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    if temp_r31 == 1 {
        ext::HSD_JObjReqAnim(jobj, 0.0);
    } else {
        ext::HSD_JObjReqAnim(jobj, 1.0);
    }
    ext::HSD_JObjAnim(jobj);
}

/// Name/description/series text objects of the viewer (`Toy_sbss_804D6EE0` as `tyDispData`:
/// the four `HSD_Text*` at +0x144.. are the native record's `texts`).
pub unsafe fn _Toy_803084A0(arg0: i32) {
    let id = arg0;
    let display = Toy_sbss_804D6EE0;
    let color = _Toy_color_FFBA00FF;
    if (*display).texts[0].is_null() {
        (*display).texts[0] = ext::HSD_SisLib_803A5ACC(0, _Toy_sbss_804D6E70, 0.9, -10.9, 0.0, 384.0, 64.0);
        let one = 1;
        let text = (*display).texts[0];
        (*text).default_fitting = one;
        (*text).default_alignment = one;
        (*text).text_color = color;
        if ext::lbLang_IsSavedLanguageJP() != 0 {
            let text = (*display).texts[0];
            (*text).font_size.x = 0.044;
            (*text).font_size.y = 0.103;
            (*text).x34.x = 2.0;
            (*text).x34.y = 1.0;
        } else {
            let text = (*display).texts[0];
            (*text).font_size.x = 0.044;
            (*text).font_size.y = 0.103;
            (*(*display).texts[0]).default_kerning = one;
            (*text).x34.x = 1.75;
            (*text).x34.y = 1.0;
        }
    }
    ext::HSD_SisLib_803A6368((*display).texts[0], Toy_803063D4(id, 2, 0x128));
    if (*display).texts[1].is_null() {
        if ext::lbLang_IsSavedLanguageJP() != 0 {
            (*display).texts[1] = ext::HSD_SisLib_803A5ACC(3, _Toy_sbss_804D6E74, 0.2, -5.1, 17.2, 384.0, 384.0);
            let text = (*display).texts[1];
            (*text).font_size.x = 0.035;
            (*text).font_size.y = 0.034;
        } else {
            (*display).texts[1] = ext::HSD_SisLib_803A5ACC(3, _Toy_sbss_804D6E74, 0.2, -5.0, 17.2, 384.0, 384.0);
            let text = (*display).texts[1];
            (*text).font_size.x = 0.034;
            (*text).font_size.y = 0.033;
            (*text).x34.x = 0.7;
            (*text).x34.y = 0.7;
        }
        let text = (*display).texts[1];
        (*text).default_fitting = 1;
        (*text).default_kerning = 1;
    }
    ext::HSD_SisLib_803A6368((*display).texts[1], Toy_803063D4(id, 2, 0x374));
    if (*display).texts[2].is_null() {
        if ext::lbLang_IsSavedLanguageJP() != 0 {
            (*display).texts[2] = ext::HSD_SisLib_803A5ACC(3, _Toy_sbss_804D6E78, 0.7, 7.9, 17.2, 384.0, 64.0);
            (*display).texts[3] = ext::HSD_SisLib_803A5ACC(3, _Toy_sbss_804D6E7C, 0.7, 9.0, 17.2, 384.0, 64.0);
            let text = (*display).texts[2];
            (*text).font_size.x = 0.03;
            (*text).font_size.y = 0.03;
            let text = (*display).texts[3];
            (*text).font_size.x = 0.03;
            (*text).font_size.y = 0.03;
        } else {
            (*display).texts[2] = ext::HSD_SisLib_803A5ACC(3, _Toy_sbss_804D6E78, 0.7, 7.9, 17.2, 384.0, 64.0);
            (*display).texts[3] = ext::HSD_SisLib_803A5ACC(3, _Toy_sbss_804D6E7C, 0.7, 9.0, 17.2, 384.0, 64.0);
            let text = (*display).texts[2];
            (*text).font_size.x = 0.03;
            (*text).font_size.y = 0.03;
            let text = (*display).texts[3];
            (*text).font_size.x = 0.03;
            (*text).font_size.y = 0.03;
        }
        (*(*display).texts[2]).default_kerning = 1;
        (*(*display).texts[3]).default_kerning = 1;
        let text = (*display).texts[2];
        (*text).x34.x = 1.0;
        (*text).x34.y = 1.0;
        let text = (*display).texts[3];
        (*text).x34.x = 1.0;
        (*text).x34.y = 1.0;
        (*(*display).texts[2]).default_fitting = 1;
        (*(*display).texts[3]).default_fitting = 1;
        (*(*display).texts[2]).default_alignment = 0;
        (*(*display).texts[3]).default_alignment = 0;
    }
    ext::HSD_SisLib_803A6368((*display).texts[2], Toy_803063D4(id, 0x128, 0x37A));
    ext::HSD_SisLib_803A6368((*display).texts[3], Toy_803063D4(id, 0x24E, 0x380));
    // The four text objects are made once (above) and only re-pointed afterwards; Atlas draws
    // the name, description and series itself, so the retail objects are hidden.
    if ext::Ui_RetailHidden(AT_RE_TOY_TEXT) != 0 {
        (*(*display).texts[0]).hidden = 1;
        (*(*display).texts[1]).hidden = 1;
        (*(*display).texts[2]).hidden = 1;
        (*(*display).texts[3]).hidden = 1;
    }
}

/// Show the 3D model of a list entry (`arg0` is a `ToyListEntry`).
pub unsafe fn Toy_803087F4(arg0: *mut ToyListEntry) -> *mut HSD_GObj {
    let entry = arg0;
    let anim = &raw mut Toy_804A2AA8;
    let mut sp_c: *mut c_void = core::ptr::null_mut();
    if (*entry).archive.is_null() {
        let trophy_id = (*entry).trophy_id;
        let model_name = Toy_8030813C(trophy_id as i32);
        if !(*entry).archive.is_null() {
            ext::lbArchive_80016EFC((*entry).archive);
            (*entry).archive = core::ptr::null_mut();
        }
        (*entry).archive_name = (*model_name).archive_name.as_mut_ptr();
        (*entry).symbol_name = (*model_name).symbol_name.as_mut_ptr();
        (*entry).trophy_id = trophy_id;
        (*entry).archive =
            ext::lbArchive_LoadSymbols((*entry).archive_name, &[(&raw mut sp_c) as *mut c_void, (*entry).symbol_name as *mut c_void]);
    }
    let joint = ext::HSD_ArchiveGetPublicAddress((*entry).archive, (*entry).symbol_name) as *mut HSD_Joint;
    if joint.is_null() {
        ext::OSReport("*** Cann't Show Model!\n");
        hsd_assert_fail(3358);
    }
    if !(*anim).gobj.is_null() {
        ext::HSD_GObjFree((*anim).gobj);
        (*anim).gobj = core::ptr::null_mut();
        (*anim).jobj[1] = core::ptr::null_mut();
        (*anim).jobj[0] = core::ptr::null_mut();
    }
    (*anim).gobj = ext::GObj_Create(6, 7, 0);
    (*anim).xC = (*entry).trophy_id;
    let parent_jobj = ext::HSD_JObjAlloc();
    _Toy_80307BA0(parent_jobj, (*anim).xC);
    let trophy_jobj = ext::HSD_JObjLoadJoint(joint);
    ext::HSD_JObjAddChild(parent_jobj, trophy_jobj);
    let kind = ext::HSD_GObj_JObjKind;
    ext::HSD_GObjObject_80390A70((*anim).gobj, kind, parent_jobj as *mut c_void);
    ext::GObj_SetupGXLink((*anim).gobj, Some(ext::cb::HSD_GObj_JObjCallback), 0x39, 0);
    let xc = (*anim).xC as i32;
    HSD_JObjSetTranslateX(&mut *trophy_jobj, Toy_803060BC(xc, 0), &mut Dirty);
    HSD_JObjSetTranslateY(&mut *trophy_jobj, Toy_803060BC(xc, 1), &mut Dirty);
    HSD_JObjSetTranslateZ(&mut *trophy_jobj, Toy_803060BC(xc, 2), &mut Dirty);
    let scale = Toy_803060BC(xc, 3);
    HSD_JObjSetScaleX(&mut *trophy_jobj, scale, &mut Dirty);
    HSD_JObjSetScaleY(&mut *trophy_jobj, scale, &mut Dirty);
    HSD_JObjSetScaleZ(&mut *trophy_jobj, scale, &mut Dirty);
    let rot = Toy_803060BC(xc, 5) * 0.01745329252f32;
    HSD_JObjSetRotationY(&mut *trophy_jobj, rot, &mut Dirty);
    if !_Toy_sbss_804D6E9C.is_null() {
        ext::DevText_Erase(_Toy_sbss_804D6E9C);
        ext::DevText_SetCursorXY(_Toy_sbss_804D6E9C, 0, 0);
        let s = format!(
            "X   {:3.2}\nY   {:3.2}\nZ   {:3.2}\nMS  {:3.2}\nSS  {:3.2}\nMD  {:3.2}\0",
            Toy_803060BC(xc, 0) as f64,
            Toy_803060BC(xc, 1) as f64,
            Toy_803060BC(xc, 2) as f64,
            Toy_803060BC(xc, 3) as f64,
            Toy_803060BC(xc, 4) as f64,
            Toy_803060BC(xc, 5) as f64
        );
        ext::DevText_Print(_Toy_sbss_804D6E9C, s.as_ptr() as *mut i8);
    }
    {
        let child = _Toy_80307BA0(parent_jobj, (*anim).xC);
        ext::HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32);
        (*anim).x0E = 0;
    }
    (*anim).gobj
}

pub unsafe fn _Toy_80308DC8(cobj: *mut HSD_CObj) {
    let mut interest = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut eye_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let temp_r31 = &raw mut Toy_804A2AA8;
    let temp_r30 = _Toy_sbss_804D6E68;
    ext::HSD_CObjGetInterest(cobj, &mut interest);
    ext::HSD_CObjGetEyePosition(cobj, &mut eye_pos);
    let temp_f1 = (*temp_r30).x18;
    if temp_f1 >= 25.0 || (temp_f1 >= -3.0 && _Toy_80309338(&mut eye_pos, core::ptr::null_mut()) <= 15.0) {
        _Toy_80307F64(1, 1);
        return;
    }
    _Toy_80307F64(2, 1);
    if (*temp_r30).x18 < 10.0 {
        if eye_pos.y < 0.0 {
            (*temp_r31).x0E = 1;
            ext::HSD_JObjClearFlagsAll((*temp_r31).jobj[1], JOBJ_HIDDEN as u32);
            ext::HSD_JObjSetFlagsAll((*temp_r31).jobj[0], JOBJ_HIDDEN as u32);
        } else {
            (*temp_r31).x0E = 0;
            ext::HSD_JObjClearFlagsAll((*temp_r31).jobj[0], JOBJ_HIDDEN as u32);
            ext::HSD_JObjSetFlagsAll((*temp_r31).jobj[1], JOBJ_HIDDEN as u32);
        }
    } else {
        (*temp_r31).x0E = 1;
        ext::HSD_JObjClearFlagsAll((*temp_r31).jobj[1], JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlagsAll((*temp_r31).jobj[0], JOBJ_HIDDEN as u32);
    }
}

pub unsafe fn _Toy_80308F04(cobj: *mut HSD_CObj) {
    let mut interest = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    // `(Toy26B8*) &_Toy_804A26B8`: only `x0` (a Vec3 over the first 12 bytes) is used.
    let data = &raw mut _Toy_804A26B8 as *mut Toy26B8;
    let state = _Toy_sbss_804D6E68;
    let jobj_ptr = (*(*(&raw mut Toy_804A2AA8)).gobj).hsd_obj as *mut HSD_JObj;
    let top = ext::HSD_CObjGetTop(cobj);
    let bottom = ext::HSD_CObjGetBottom(cobj);
    let right = ext::HSD_CObjGetRight(cobj);
    let left = ext::HSD_CObjGetLeft(cobj);
    HSD_JObjGetScaleY(&*jobj_ptr);
    if (*state).x61 == 1 {
        if ((*state).x5C as f32) < 10.0 {
            _Toy_sbss_804D6E80 = _Toy_sbss_804D6E80 - 0.0013187003;
            if top != _Toy_sbss_804D6E80 {
                let mut val = _Toy_sbss_804D6E80;
                if val < 0.036397 {
                    val = 0.036397;
                }
                ext::HSD_CObjSetTop(cobj, val);
            }
            _Toy_sbss_804D6E84 = _Toy_sbss_804D6E84 - 0.00008119978;
            if bottom != _Toy_sbss_804D6E84 {
                let mut val = _Toy_sbss_804D6E84;
                if val < -0.036397 {
                    val = -0.036397;
                }
                ext::HSD_CObjSetBottom(cobj, val);
            }
            _Toy_sbss_804D6E88 = _Toy_sbss_804D6E88 - 0.0032531999;
            if right != _Toy_sbss_804D6E88 {
                let mut val = _Toy_sbss_804D6E88;
                if val < 0.044307 {
                    val = 0.044307;
                }
                ext::HSD_CObjSetRight(cobj, val);
            }
            _Toy_sbss_804D6E8C = _Toy_sbss_804D6E8C - 0.0017468001;
            if left != _Toy_sbss_804D6E8C {
                ext::HSD_CObjSetLeft(cobj, _Toy_sbss_804D6E8C);
            }
            (*state).x5C = (*state).x5C + 1;
        } else {
            ext::HSD_CObjSetTop(cobj, 0.036397);
            ext::HSD_CObjSetBottom(cobj, -0.036397);
            ext::HSD_CObjSetRight(cobj, 0.044307);
            ext::HSD_CObjSetLeft(cobj, -0.044307);
            let mut jobj = (*(*Toy_sbss_804D6ED8).gobj2).hsd_obj as *mut HSD_SObj;
            if _Toy_sbss_804D6E58 != 0 {
                while !jobj.is_null() {
                    (*jobj).x40 = 9;
                    jobj = (*jobj).next;
                }
            } else {
                while !jobj.is_null() {
                    (*jobj).x40 = 8;
                    jobj = (*jobj).next;
                }
            }
            (*state).x61 = 2;
            (*state).x5C = 0;
            (*state).x60 = 4;
            ext::HSD_PadRenewStatus();
        }
    } else if ((*state).x5C as f32) < 10.0 {
        (*state).x20 = (*state).x20 - _Toy_sbss_804D6E90;
        (*state).x18 = (*state).x18 - _Toy_sbss_804D6E94;
        ext::HSD_CObjGetInterest(cobj, &mut interest);
        interest.x = interest.x + (*data).x0.x;
        interest.y = interest.y + (*data).x0.y;
        interest.z = interest.z + (*data).x0.z;
        ext::HSD_CObjSetInterest(cobj, &mut interest);
        _Toy_80308DC8(cobj);
        _Toy_sbss_804D6E80 += 0.0013187003;
        if top != _Toy_sbss_804D6E80 {
            let mut val = _Toy_sbss_804D6E80;
            if val > 0.049584 {
                val = 0.049584;
            }
            ext::HSD_CObjSetTop(cobj, val);
        }
        _Toy_sbss_804D6E84 += 0.00008119978;
        if bottom != _Toy_sbss_804D6E84 {
            let mut val = _Toy_sbss_804D6E84;
            if val > -0.035585 {
                val = -0.035585;
            }
            ext::HSD_CObjSetBottom(cobj, val);
        }
        _Toy_sbss_804D6E88 += 0.0032531999;
        if right != _Toy_sbss_804D6E88 {
            let mut val = _Toy_sbss_804D6E88;
            if val > 0.076839 {
                val = 0.076839;
            }
            ext::HSD_CObjSetRight(cobj, val);
        }
        _Toy_sbss_804D6E8C += 0.0017468001;
        if left != _Toy_sbss_804D6E8C {
            ext::HSD_CObjSetLeft(cobj, _Toy_sbss_804D6E8C);
        }
        (*state).x5C = (*state).x5C + 1;
    } else {
        (*state).x61 = 0;
        _Toy_80307828(1);
        ext::HSD_CObjSetTop(cobj, 0.049584);
        ext::HSD_CObjSetBottom(cobj, -0.035585);
        ext::HSD_CObjSetRight(cobj, 0.076839);
        ext::HSD_CObjSetLeft(cobj, -0.026839);
        _Toy_803102C4(1);
        (*state).x5C = 0;
        (*state).x58 = 0x95E;
        (*state).x60 = 4;
        ext::HSD_PadRenewStatus();
    }
}

/// Distance between two points (`NULL` = origin) with the Gekko `frsqrte` Newton-Raphson
/// square root. Shipping contraction: `x*x + acc` fuses (f32), `3.0 - g*g*v` fuses (f64).
pub unsafe fn _Toy_80309338(arg0: *mut Vec3, arg1: *mut Vec3) -> f32 {
    let mut sp14 = _Toy_803B88D4;
    let mut var_r3 = arg0;
    let mut var_r4 = arg1;
    if var_r3.is_null() {
        var_r3 = &mut sp14;
    }
    if var_r4.is_null() {
        var_r4 = &mut sp14;
    }
    let mut var_f1 = (*var_r3).y - (*var_r4).y;
    let temp_f4 = (*var_r3).x - (*var_r4).x;
    let temp_f2 = (*var_r3).z - (*var_r4).z;
    var_f1 = var_f1 * var_f1;
    var_f1 = fmadds(temp_f4, temp_f4, var_f1); // fused
    var_f1 = fmadds(temp_f2, temp_f2, var_f1); // fused
    if var_f1 > 0.0 {
        let v = var_f1 as f64;
        let mut guess = melee_core::fp::frsqrte(v);
        guess = 0.5 * guess * (-(guess * guess)).mul_add(v, 3.0); // fused (f64)
        guess = 0.5 * guess * (-(guess * guess)).mul_add(v, 3.0); // fused (f64)
        guess = 0.5 * guess * (-(guess * guess)).mul_add(v, 3.0); // fused (f64)
        let sp10 = (v * guess) as f32;
        var_f1 = sp10;
    }
    var_f1
}

// ---------------------------------------------------------------------------------------------
// The trophy viewer per-frame proc
// ---------------------------------------------------------------------------------------------

#[inline]
unsafe fn sfxBack() {
    ext::lbAudioAx_80024030(0);
}
#[inline]
unsafe fn sfxForward() {
    ext::lbAudioAx_80024030(1);
}
#[inline]
unsafe fn sfxMove() {
    ext::lbAudioAx_80024030(2);
}

/// `_Toy_ReadTrigger` (static inline): first pad with a trigger edge.
#[inline]
unsafe fn _Toy_ReadTrigger(out: &mut u32) {
    for i in 0..4 {
        *out = (*pad(i as u8)).trigger;
        if *out != 0 {
            ext::gm_801677E8(i as i8);
            break;
        }
    }
}

/// The stick dead-zone remap `(f32)(-((0.2f * sign) - v) / 0.8f)` with the shipping fused
/// multiply-subtract (`fmuladd(0.2, sign, -v)`).
#[inline]
fn stick_remap(tmp: f32) -> f32 {
    if tmp > -0.2 && tmp < 0.2 {
        0.0
    } else {
        let sign: i32 = if tmp > 0.0 { 1 } else { -1 };
        -fmadds(0.2, sign as f32, -tmp) / 0.8 // fused
    }
}

/// Number of trophies currently in the viewer's list (`TOY_TROPHY_COUNT_OF` or the save's).
#[inline]
unsafe fn trophy_count_now() -> i32 {
    if toy_scene_mode() {
        *TOY_COUNT() as i32
    } else {
        *ext::gmMainLib_GetTrophyCount() as i32
    }
}

/// The trophy flags for the current mode (the scene array or the save's).
#[inline]
unsafe fn flags_now() -> *mut u16 {
    if toy_scene_mode() {
        toy_flags()
    } else {
        ext::gmMainLib_GetTrophyFlags() as *mut u16
    }
}

/// Toggle the "new" bit (0x8000) of the currently selected trophy, if set.
#[inline]
unsafe fn clear_new_bit(idx: i16) {
    let mut flags = flags_now();
    if *flags.offset(idx as isize) & 0x8000 != 0 {
        flags = flags_now();
        flags = flags.offset(idx as isize);
        *flags ^= 0x8000;
    }
}

/// Per-frame input for the Trophy Gallery's 3D viewer (rotate, zoom, pan, cycle trophies).
pub unsafe extern "C" fn _Toy_80309404(gobj: *mut HSD_GObj) {
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    let base = &raw mut _Toy_804A26B8 as *mut Toy26B8;
    let ed8 = Toy_sbss_804D6ED8;
    let state = _Toy_sbss_804D6E68;
    let anim = &raw mut Toy_804A2AA8;
    let ed4 = Toy_sbss_804D6ED4;
    let mut zoom_update: f32 = 0.0;
    let mut movement_update: f32 = 0.0;
    let mut rotate_update: f32 = 0.0;
    let mut trigger: u32;
    let mut button: u32 = 0;
    let mut tmp: f32;
    let mut unused_eye = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut eye_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut transition_interest: Vec3;
    let mut transition_eye: Vec3;
    let mut sym_out: *mut c_void = core::ptr::null_mut();

    ext::HSD_CObjGetEyePosition(cobj, &mut eye_pos);
    let dist = _Toy_80309338(&mut eye_pos, core::ptr::null_mut());
    {
        let mut val: f32 = 0.0;
        for i in 0..4 {
            val = (*pad(i as u8)).nml_stickX;
            let abs = if val < 0.0 { -val } else { val };
            if abs > 0.1 {
                break;
            }
        }
        (*state).x30 = val;
    }
    (*state).x34 = Toy_80305DB0();
    (*state).x54 = 0.0;
    (*state).x50 = 0.0;
    (*state).x30 = stick_remap((*state).x30);
    (*state).x34 = stick_remap((*state).x34);
    (*state).x40 = Toy_80305EB4();
    (*state).x44 = Toy_80305FB8();
    (*state).x40 = stick_remap((*state).x40);
    (*state).x44 = stick_remap((*state).x44);
    {
        let cooldown = (*state).x60 as u8;
        if cooldown as i8 != 0 {
            (*state).x60 = cooldown.wrapping_sub(1) as i8;
            _Toy_8030715C(0.0, 0.0);
            return;
        }
    }
    if ext::mn_8022F218() != 0 {
        sfxBack();
        ext::HSD_GObjProc_RemoveProc(ext::HSD_GObj_CurrentInvokedProc);
        Toy_80310660(1);
        ext::HSD_GObj_80390CD4(gobj);
        ext::mn_8022F268();
        *toy_byte(4) = 1; // TOY_X198
        return;
    }
    match (*state).x61 {
        0 => {
            trigger = Toy_80305B88() as u32;
            if trigger & HSD_PAD_B as u32 != 0 {
                sfxBack();
                Toy_80310660(1);
                ext::HSD_GObj_80390CD4(gobj);
                ext::tyList_803147C4();
                return;
            }
            if trophy_count_now() == 0 {
                return;
            }
            let mut reset_idle_timer = false;
            if (*state).x40 + (*state).x44 != 0.0 {
                reset_idle_timer = true;
            } else {
                trigger = Toy_80305B88() as u32;
                if trigger & (HSD_PAD_DPADLEFT | HSD_PAD_DPADRIGHT | HSD_PAD_DPADDOWN | HSD_PAD_DPADUP) as u32 != 0 {
                    reset_idle_timer = true;
                }
            }
            if reset_idle_timer {
                (*state).x58 = 0;
            } else {
                if (*state).x58 as f32 > 2400.0 {
                    (*state).x24 += 0.01;
                    if (*state).x24 > 1.0 {
                        (*state).x24 = 1.0;
                    }
                    (*state).x1C = (*state).x1C + (*state).x24;
                    tmp = (*state).x1C;
                    if tmp < -360.0 {
                        (*state).x1C += 360.0;
                    }
                    tmp = (*state).x1C;
                    if tmp > 360.0 {
                        (*state).x1C = tmp - 360.0;
                    }
                    (*ed4).x18 = (*state).x1C;
                }
                (*state).x58 = (*state).x58 + 1;
            }
            if trophy_count_now() == 1 && (*state).x30 + (*state).x34 != 0.0 {
                (*state).x34 = 0.0;
                (*state).x30 = 0.0;
            }
            trigger = Toy_80305B88() as u32;
            if trigger & (HSD_PAD_A | HSD_PAD_X | HSD_PAD_Y) as u32 != 0 {
                sfxForward();
                _Toy_sbss_804D6E80 = ext::HSD_CObjGetTop(cobj);
                _Toy_sbss_804D6E84 = ext::HSD_CObjGetBottom(cobj);
                _Toy_sbss_804D6E88 = ext::HSD_CObjGetRight(cobj);
                _Toy_sbss_804D6E8C = ext::HSD_CObjGetLeft(cobj);
                (*state).x5C = 0;
                (*((*state).x0 as *mut HSD_GObj)).gxlink_prios = 0x0048000000000000u64;
                (*((*state).x4 as *mut HSD_GObj)).gxlink_prios = 0;
                (*((*state).xC as *mut HSD_GObj)).gxlink_prios = 0;
                (*state).x61 = 1;
                _Toy_8030715C(0.0, 0.0);
                return;
            }
            rotate_update = 1.0;
        }
        1 | 3 => {
            _Toy_80308F04(cobj);
            rotate_update = 0.0;
        }
        2 => {
            trigger = Toy_80305B88() as u32;
            if trigger & HSD_PAD_A as u32 != 0 {
                _Toy_sbss_804D6E58 ^= 1;
                let mut jobj_node = (*(*ed8).gobj2).hsd_obj as *mut HSD_SObj;
                if _Toy_sbss_804D6E58 != 0 {
                    while !jobj_node.is_null() {
                        (*jobj_node).x40 = 9;
                        jobj_node = (*jobj_node).next;
                    }
                } else {
                    while !jobj_node.is_null() {
                        (*jobj_node).x40 = 8;
                        jobj_node = (*jobj_node).next;
                    }
                }
            }
            trigger = Toy_80305B88() as u32;
            if trigger & HSD_PAD_B as u32 != 0 || (*state).x5C as f32 > 7200.0 {
                sfxBack();
                _Toy_sbss_804D6E80 = ext::HSD_CObjGetTop(cobj);
                _Toy_sbss_804D6E84 = ext::HSD_CObjGetBottom(cobj);
                _Toy_sbss_804D6E88 = ext::HSD_CObjGetRight(cobj);
                _Toy_sbss_804D6E8C = ext::HSD_CObjGetLeft(cobj);
                (*((*state).x0 as *mut HSD_GObj)).gxlink_prios = 0x5048000000000000u64;
                (*((*state).x4 as *mut HSD_GObj)).gxlink_prios = 0x8000000000000000u64;
                (*((*state).xC as *mut HSD_GObj)).gxlink_prios = 0x4000000000000000u64;
                let mut jobj_node = (*(*ed8).gobj2).hsd_obj as *mut HSD_SObj;
                while !jobj_node.is_null() {
                    (*jobj_node).x40 = 9;
                    jobj_node = (*jobj_node).next;
                }
                (*state).x5C = 0;
                transition_eye = _Toy_803B88E0;
                transition_interest = _Toy_803B88EC;
                ext::HSD_CObjGetEyePosition(cobj, &mut unused_eye);
                transition_eye.y = 8.0;
                ext::HSD_CObjGetInterest(cobj, &mut transition_interest);
                (*base).x0.x = (transition_eye.x - transition_interest.x) / 10.0;
                (*base).x0.y = (transition_eye.y - transition_interest.y) / 10.0;
                (*base).x0.z = (transition_eye.z - transition_interest.z) / 10.0;
                _Toy_sbss_804D6E90 = ((*state).x20 - 38.0) / 10.0;
                _Toy_sbss_804D6E94 = (*state).x18 / 10.0;
                (*state).x61 = 3;
            }
            trigger = Toy_80305B88() as u32;
            tmp = (*state).x44 + ((*state).x40 + ((*state).x30 + (*state).x34));
            if tmp + (trigger as f32) == 0.0 {
                (*state).x5C = (*state).x5C + 1;
            } else {
                (*state).x5C = 0;
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_DPADLEFT as u32 != 0 {
                (*state).x50 = -0.3f32 * (dist / 38.0);
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_DPADRIGHT as u32 != 0 {
                (*state).x50 = 0.3f32 * (dist / 38.0);
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_DPADUP as u32 != 0 {
                (*state).x54 = 0.3f32 * (dist / 38.0);
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_DPADDOWN as u32 != 0 {
                (*state).x54 = -0.3f32 * (dist / 38.0);
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_A as u32 != 0 {
                tmp = (*state).x30;
                if tmp != 0.0 && tmp < 0.0 {
                    (*state).x50 = 0.3f32 * tmp * (dist / 38.0);
                }
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_A as u32 != 0 {
                tmp = (*state).x30;
                if tmp != 0.0 && tmp > 0.0 {
                    (*state).x50 = 0.3f32 * tmp * (dist / 38.0);
                }
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_A as u32 != 0 {
                tmp = (*state).x34;
                if tmp != 0.0 && tmp > 0.0 {
                    (*state).x54 = 0.3f32 * tmp * (dist / 38.0);
                }
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_A as u32 != 0 {
                tmp = (*state).x34;
                if tmp != 0.0 && tmp < 0.0 {
                    (*state).x54 = 0.3f32 * tmp * (dist / 38.0);
                }
            }
            if (*state).x50 != 0.0 || (*state).x54 != 0.0 || (Toy_80305C44() as u32 & HSD_PAD_A as u32) != 0 {
                _Toy_803102C4(0);
            } else {
                _Toy_803102C4(1);
            }
            if (*state).x50 != 0.0 || (*state).x54 != 0.0 {
                movement_update = 1.0;
            }
            if movement_update == 0.0 {
                button = Toy_80305C44() as u32;
                if button & HSD_PAD_Y as u32 != 0 {
                    (*state).x20 = -fmadds(0.025, dist, -(*state).x20); // fused
                    zoom_update = 1.0;
                } else {
                    button = Toy_80305C44() as u32;
                    if button & HSD_PAD_X as u32 != 0 {
                        (*state).x20 = -fmadds(-0.025, dist, -(*state).x20); // fused
                        zoom_update = 1.0;
                    } else {
                        tmp = (*state).x34;
                        if tmp != 0.0 {
                            (*state).x20 = -fmadds(dist, 0.025f32 * tmp, -(*state).x20); // fused
                            zoom_update = 1.0;
                        }
                    }
                }
                if (*state).x20 < 5.0 {
                    (*state).x20 = 5.0;
                }
                if (*state).x20 > 250.0 {
                    (*state).x20 = 250.0;
                }
            }
            rotate_update = 1.0;
        }
        _ => {}
    }
    if rotate_update != 0.0 {
        (*state).x1C = -fmadds(3.0, (*state).x40, -(*state).x1C); // fused
        (*state).x18 = -fmadds(3.0, (*state).x44, -(*state).x18); // fused
        if (*state).x18 < -89.0 {
            (*state).x18 = -89.0;
        }
        if (*state).x18 > 89.0 {
            (*state).x18 = 89.0;
        }
        tmp = (*state).x1C;
        if tmp < -360.0 {
            (*state).x1C += 360.0;
        }
        tmp = (*state).x1C;
        if tmp > 360.0 {
            (*state).x1C = tmp - 360.0;
        }
        (*ed4).x18 = (*state).x1C;
        (*ed4).x14 = (*state).x18;
        if (*state).x40 + (*state).x44 != 0.0 {
            (*state).x24 = 0.0;
        }
        if movement_update == 0.0 && zoom_update == 0.0 && trophy_count_now() > 1 {
            tmp = (*state).x30;
            if tmp < 0.0 {
                tmp = -tmp;
            }
            let mut do_cycle = true;
            if !(tmp > 0.9) {
                trigger = Toy_80305B88() as u32;
                if trigger & (HSD_PAD_L | HSD_PAD_R) as u32 == 0 {
                    do_cycle = false;
                }
            }
            if do_cycle {
                let display = Toy_sbss_804D6EE0;
                if (*state).x30 < 0.0 || (Toy_80305B88() & 0x441) != 0 {
                    sfxMove();
                    (*display).selectedIdx = (*display).selectedIdx - 1;
                    if (*display).selectedIdx < 0 {
                        let total = trophy_count_now();
                        (*display).selectedIdx = (total - 1) as i16;
                    }
                    let total = trophy_count_now();
                    if total > 3 {
                        if ((*display).selectedIdx as i32 - 1) < 0 {
                            let mut list_idx = trophy_count_now();
                            list_idx += (*display).selectedIdx as i32;
                            let keys = Toy_sbss_804D6EDC;
                            let mut entry = (*display).first_entry;
                            entry = (*entry).prev;
                            let idx = list_idx - 1;
                            let tid = *keys.offset(idx as isize);
                            setupTrophyEntry(entry, tid);
                            (*entry).archive = ext::lbArchive_LoadSymbols(
                                (*entry).archive_name,
                                &[(&raw mut sym_out) as *mut c_void, (*entry).symbol_name as *mut c_void],
                            );
                        } else {
                            let tid = *Toy_sbss_804D6EDC.offset((*display).selectedIdx as isize - 1);
                            let entry = (*(*display).first_entry).prev;
                            setupTrophyEntry(entry, tid);
                            (*entry).archive = ext::lbArchive_LoadSymbols(
                                (*entry).archive_name,
                                &[(&raw mut sym_out) as *mut c_void, (*entry).symbol_name as *mut c_void],
                            );
                        }
                        {
                            let ma = (*(*display).last_entry).archive;
                            if !ma.is_null() {
                                ext::lbArchive_80016EFC(ma);
                                (*(*display).last_entry).archive = core::ptr::null_mut();
                            }
                        }
                        (*display).selected_entry = (*(*display).selected_entry).prev;
                        (*display).first_entry = (*(*display).first_entry).prev;
                        (*display).last_entry = (*(*display).last_entry).prev;
                    } else {
                        (*display).selected_entry = (*(*display).selected_entry).prev;
                    }
                } else {
                    let mut go_next = true;
                    if !((*state).x30 > 0.0) {
                        let mut btn3: u32 = 0;
                        _Toy_ReadTrigger(&mut btn3);
                        if btn3 & 0x822 == 0 {
                            go_next = false;
                        }
                    }
                    if go_next {
                        sfxMove();
                        (*display).selectedIdx = (*display).selectedIdx + 1;
                        let total = trophy_count_now();
                        if (*display).selectedIdx as i32 >= total {
                            (*display).selectedIdx = 0;
                        }
                        let total = trophy_count_now();
                        if total > 3 {
                            if ((*display).selectedIdx as i32 + 1) >= _Toy_GetTrophyTotal() {
                                let cnt2 = trophy_count_now();
                                let lk = (*display).selectedIdx as i32 - cnt2;
                                let tid = *Toy_sbss_804D6EDC.offset(lk as isize + 1);
                                let entry = (*(*display).last_entry).next;
                                setupTrophyEntry(entry, tid);
                                (*entry).archive = ext::lbArchive_LoadSymbols(
                                    (*entry).archive_name,
                                    &[(&raw mut sym_out) as *mut c_void, (*entry).symbol_name as *mut c_void],
                                );
                            } else {
                                let tid = *Toy_sbss_804D6EDC.offset((*display).selectedIdx as isize + 1);
                                let entry = (*(*display).last_entry).next;
                                setupTrophyEntry(entry, tid);
                                (*entry).archive = ext::lbArchive_LoadSymbols(
                                    (*entry).archive_name,
                                    &[(&raw mut sym_out) as *mut c_void, (*entry).symbol_name as *mut c_void],
                                );
                            }
                            {
                                let ma = (*(*display).first_entry).archive;
                                if !ma.is_null() {
                                    ext::lbArchive_80016EFC(ma);
                                    (*(*display).first_entry).archive = core::ptr::null_mut();
                                }
                            }
                            (*display).selected_entry = (*(*display).selected_entry).next;
                            (*display).first_entry = (*(*display).first_entry).next;
                            (*display).last_entry = (*(*display).last_entry).next;
                        } else {
                            (*display).selected_entry = (*(*display).selected_entry).next;
                        }
                    }
                }
                if true {
                    // (the labelled `next_done:` join point of the C)
                    _Toy_80307828(0);
                    _Toy_8030715C(0.0, 0.0);
                    (*state).x58 = 0x95E;
                    Toy_803087F4((*display).selected_entry);
                    {
                        let idx = *Toy_sbss_804D6EDC.offset((*display).selectedIdx as isize);
                        let flags = flags_now();
                        if *flags.offset(idx as isize) & 0x8000 != 0 {
                            let selected_trophy = *Toy_sbss_804D6EDC.offset((*display).selectedIdx as isize);
                            let idx = selected_trophy;
                            let mut flags = flags_now();
                            flags = flags.offset(idx as isize);
                            *flags ^= 0x8000;
                        }
                    }
                    _Toy_803084A0((*(*display).selected_entry).trophy_id as i32);
                    Toy_803083D8((*ed8).x30, (*(*display).selected_entry).trophy_id as i32);
                    (*state).x60 = 0x14;
                }
            }
        }
        {
            let mut btn: u32 = 0;
            _Toy_ReadTrigger(&mut btn);
            if btn & HSD_PAD_START as u32 != 0 {
                sfxMove();
                (*state).x58 = 0;
                (*ed4).x10 = (*ed4).x10 + 1;
                if (*ed4).x10 == 6 {
                    (*ed4).x10 = 0;
                }
                Toy_80306D70((*ed4).x10);
                _Toy_803075E8((*ed4).x10);
            }
        }
        {
            let mut btn2: u32 = 0;
            _Toy_ReadTrigger(&mut btn2);
            if btn2 & HSD_PAD_Z as u32 != 0 {
                _Toy_80307828(0);
                let camera = Toy_sbss_804D6ED4;
                (*camera).x10 = 0;
                Toy_80306D70((*camera).x10);
                _Toy_803075E8((*camera).x10);
                (*anim).x11 = 2;
                (*anim).x10 = 2;
                (*anim).x0F = 0;
                (*anim).x0E = 0;
                _Toy_80307F64(2, 0);
            }
        }
        _Toy_80308DC8(cobj);
    }
    _Toy_8030715C((*state).x50, (*state).x54);
    (*state).x38 = (*state).x30;
    (*state).x3C = (*state).x34;
    (*state).x48 = (*state).x40;
    (*state).x4C = (*state).x44;
    let _ = (&mut sym_out, &mut button);
}

// ---------------------------------------------------------------------------------------------
// Debug viewers (trophy / lighting editor), only entered through the debug-ROM pad chords
// ---------------------------------------------------------------------------------------------

#[inline]
fn abs_bits(v: f32) -> f32 {
    f32::from_bits(v.to_bits() & !0x8000_0000)
}

/// Advance the trophy list one step backwards (stick left / L): the shared body of the cycle
/// code in the viewer procs. `extra_hint` is unused by the C versions (they differ only in what
/// they do after the call).
unsafe fn cycle_prev(display: *mut TyDisplayData, sym_out: &mut *mut c_void) {
    sfxMove();
    (*display).selectedIdx = (*display).selectedIdx - 1;
    if (*display).selectedIdx < 0 {
        let total = trophy_count_now();
        (*display).selectedIdx = (total - 1) as i16;
    }
    let total = trophy_count_now();
    if total > 3 {
        if ((*display).selectedIdx as i32 - 1) < 0 {
            let mut lk = trophy_count_now();
            lk += (*display).selectedIdx as i32;
            let keys = Toy_sbss_804D6EDC;
            let mut entry = (*display).first_entry;
            entry = (*entry).prev;
            let idx = lk - 1;
            let tid = *keys.offset(idx as isize);
            setupTrophyEntry(entry, tid);
            (*entry).archive = ext::lbArchive_LoadSymbols(
                (*entry).archive_name,
                &[(sym_out as *mut *mut c_void) as *mut c_void, (*entry).symbol_name as *mut c_void],
            );
        } else {
            let tid = *Toy_sbss_804D6EDC.offset((*display).selectedIdx as isize - 1);
            let entry = (*(*display).first_entry).prev;
            setupTrophyEntry(entry, tid);
            (*entry).archive = ext::lbArchive_LoadSymbols(
                (*entry).archive_name,
                &[(sym_out as *mut *mut c_void) as *mut c_void, (*entry).symbol_name as *mut c_void],
            );
        }
        if !(*(*display).last_entry).archive.is_null() {
            ext::lbArchive_80016EFC((*(*display).last_entry).archive);
            (*(*display).last_entry).archive = core::ptr::null_mut();
        }
        (*display).selected_entry = (*(*display).selected_entry).prev;
        (*display).first_entry = (*(*display).first_entry).prev;
        (*display).last_entry = (*(*display).last_entry).prev;
    } else {
        (*display).selected_entry = (*(*display).selected_entry).prev;
    }
}

/// The forward counterpart of [`cycle_prev`] (stick right / R).
unsafe fn cycle_next(display: *mut TyDisplayData, sym_out: &mut *mut c_void) {
    sfxMove();
    (*display).selectedIdx = (*display).selectedIdx + 1;
    let total = trophy_count_now();
    if (*display).selectedIdx as i32 >= total {
        (*display).selectedIdx = 0;
    }
    let total = trophy_count_now();
    if total > 3 {
        let cnt = _Toy_GetTrophyTotal();
        if ((*display).selectedIdx as i32 + 1) >= cnt {
            let cnt2 = trophy_count_now();
            let lk = (*display).selectedIdx as i32 - cnt2;
            let tid = *Toy_sbss_804D6EDC.offset(lk as isize + 1);
            let entry = (*(*display).last_entry).next;
            setupTrophyEntry(entry, tid);
            (*entry).archive = ext::lbArchive_LoadSymbols(
                (*entry).archive_name,
                &[(sym_out as *mut *mut c_void) as *mut c_void, (*entry).symbol_name as *mut c_void],
            );
        } else {
            let tid = *Toy_sbss_804D6EDC.offset((*display).selectedIdx as isize + 1);
            let entry = (*(*display).last_entry).next;
            setupTrophyEntry(entry, tid);
            (*entry).archive = ext::lbArchive_LoadSymbols(
                (*entry).archive_name,
                &[(sym_out as *mut *mut c_void) as *mut c_void, (*entry).symbol_name as *mut c_void],
            );
        }
        if !(*(*display).first_entry).archive.is_null() {
            ext::lbArchive_80016EFC((*(*display).first_entry).archive);
            (*(*display).first_entry).archive = core::ptr::null_mut();
        }
        (*display).selected_entry = (*(*display).selected_entry).next;
        (*display).first_entry = (*(*display).first_entry).next;
        (*display).last_entry = (*(*display).last_entry).next;
    } else {
        (*display).selected_entry = (*(*display).selected_entry).next;
    }
}

/// Trophy / lighting debug viewer (per-frame proc of the camera gobj when the debug chord is held).
pub unsafe extern "C" fn _Toy_8030B530(arg0: *mut HSD_GObj) {
    let gobj = arg0;
    let _base = &raw mut _Toy_804A26B8 as *mut Toy26B8;
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    let state = _Toy_sbss_804D6E68;
    let _ed8 = Toy_sbss_804D6ED8;
    let ed4 = Toy_sbss_804D6ED4;
    let anim = &raw mut Toy_804A2AA8;
    let mut sym_out: *mut c_void = core::ptr::null_mut();

    if (*anim).gobj.is_null() {
        return;
    }
    let jobj_child = (*((*(*anim).gobj).hsd_obj as *mut HSD_JObj)).child;
    let jobj_next = (*jobj_child).next;

    let mut stick_x: f32 = 0.0;
    for i in 0..4 {
        stick_x = (*pad(i as u8)).nml_stickX;
        let abs = if stick_x < 0.0 { -stick_x } else { stick_x };
        if abs > 0.1 {
            break;
        }
    }
    let stick_y = Toy_80305DB0();
    let mut adj_x = stick_remap(stick_x);
    let mut adj_y = stick_remap(stick_y);
    let substick_x = Toy_80305EB4();
    let substick_y = Toy_80305FB8();
    let adj_sx = stick_remap(substick_x);
    let adj_sy = stick_remap(substick_y);
    {
        let cooldown = (*state).x60 as u8;
        if cooldown as i8 != 0 {
            (*state).x60 = cooldown.wrapping_sub(1) as i8;
            return;
        }
    }
    let mut trigger = Toy_80305B88() as u32;
    if trigger & HSD_PAD_Z as u32 != 0 {
        sfxBack();
        if (*state).x61 == 0 {
            Toy_80310660(1);
            ext::HSD_GObj_80390CD4(arg0);
            ext::tyList_803147C4();
            return;
        }
    }
    {
        let mode = (*state).x61;
        if mode == 1 {
            _Toy_80308F04(cobj);
            return;
        }
        if mode == 3 {
            _Toy_80308F04(cobj);
            return;
        }
    }
    trigger = Toy_80305B88() as u32;
    if trigger & HSD_PAD_X as u32 != 0 {
        _Toy_sbss_804D6E80 = ext::HSD_CObjGetTop(cobj);
        _Toy_sbss_804D6E84 = ext::HSD_CObjGetBottom(cobj);
        _Toy_sbss_804D6E88 = ext::HSD_CObjGetRight(cobj);
        _Toy_sbss_804D6E8C = ext::HSD_CObjGetLeft(cobj);
        ext::OSReport(&format!("*** Pad = {}\n", Toy_80305B88()));
        (*state).x5C = 0;
        {
            let mode = (*state).x61;
            if mode == 0 {
                (*((*state).x0 as *mut HSD_GObj)).gxlink_prios = 0x0048000000000000u64;
                (*((*state).x4 as *mut HSD_GObj)).gxlink_prios = 0;
                (*((*state).xC as *mut HSD_GObj)).gxlink_prios = 0;
                (*state).x61 = 1;
                (*state).x5C = 0;
                return;
            }
            if mode == 2 {
                (*state).x5C = 0;
                _Toy_80307828(0);
                (*((*state).x0 as *mut HSD_GObj)).gxlink_prios = 0x5048000000000000u64;
                (*((*state).x4 as *mut HSD_GObj)).gxlink_prios = 0x8000000000000000u64;
                (*((*state).xC as *mut HSD_GObj)).gxlink_prios = 0x4000000000000000u64;
                (*state).x5C = 0;
                (*state).x61 = 3;
                sfxBack();
                return;
            }
        }
    }
    let mut button = Toy_80305C44() as u32;
    if button & HSD_PAD_X as u32 != 0 {
        let mut btn2 = Toy_80305C44() as u32;
        if btn2 & 2 != 0 {
            adj_x = 0.01;
            (*state).x60 = 8;
        }
        btn2 = Toy_80305C44() as u32;
        if btn2 & 1 != 0 {
            adj_x = -0.01;
            (*state).x60 = 8;
        }
        HSD_JObjAddRotationY(&mut *jobj_next, 0.017453292f32 * adj_x, &mut Dirty);
        _Toy_803062EC((*anim).xC as i32, 5, 57.29578f32 * HSD_JObjGetRotationY(&*jobj_next));
        {
            let abs_y = if adj_y < 0.0 { -adj_y } else { adj_y };
            if abs_y > 0.8 {
                let dist = (*state).x20;
                (*state).x20 = -fmadds(dist, 0.025f32 * adj_y, -dist); // fused
                if (*state).x20 < 5.0 {
                    (*state).x20 = 5.0;
                }
                if (*state).x20 > 250.0 {
                    (*state).x20 = 250.0;
                }
            }
        }
    } else {
        let btn3 = Toy_80305C44() as u32;
        if btn3 & 0x100 != 0 {
            let mut btn4 = Toy_80305C44() as u32;
            if btn4 & 2 != 0 {
                adj_x = 1.0;
                (*state).x60 = 8;
            }
            btn4 = Toy_80305C44() as u32;
            if btn4 & 1 != 0 {
                adj_x = -1.0;
                (*state).x60 = 8;
            }
            btn4 = Toy_80305C44() as u32;
            if btn4 & 8 != 0 {
                adj_y = 1.0;
                (*state).x60 = 8;
            }
            btn4 = Toy_80305C44() as u32;
            if btn4 & 4 != 0 {
                adj_y = -1.0;
                (*state).x60 = 8;
            }
            let angle = ext::lb_8000D008(adj_y, adj_x);
            if (*state).x18 < -25.0 {
                if abs_bits(adj_x) > 0.8 {
                    let dx = 0.01f32 * cosf(angle);
                    HSD_JObjAddTranslationX(&mut *jobj_next, dx, &mut Dirty);
                    _Toy_803062EC((*anim).xC as i32, 0, HSD_JObjGetTranslationX(&*jobj_next));
                }
                if abs_bits(adj_y) > 0.8 {
                    let sin = sinf(angle);
                    let dz = 0.01f32 * -sin;
                    HSD_JObjAddTranslationZ(&mut *jobj_next, dz, &mut Dirty);
                    _Toy_803062EC((*anim).xC as i32, 2, HSD_JObjGetTranslationZ(&*jobj_next));
                }
            } else {
                if abs_bits(adj_x) > 0.8 {
                    let dx = 0.01f32 * cosf(angle);
                    HSD_JObjAddTranslationX(&mut *jobj_next, dx, &mut Dirty);
                    _Toy_803062EC((*anim).xC as i32, 0, HSD_JObjGetTranslationX(&*jobj_next));
                }
                if abs_bits(adj_y) > 0.8 {
                    let dy = 0.01f32 * sinf(angle);
                    HSD_JObjAddTranslationY(&mut *jobj_next, dy, &mut Dirty);
                    _Toy_803062EC((*anim).xC as i32, 1, HSD_JObjGetTranslationY(&*jobj_next));
                }
            }
        } else {
            let btn5 = Toy_80305C44() as u32;
            if btn5 & 0x200 != 0 {
                if adj_y > 0.8 || (Toy_80305C44() & 8) != 0 {
                    HSD_JObjAddScaleX(&mut *jobj_next, 0.01, &mut Dirty);
                    HSD_JObjAddScaleY(&mut *jobj_next, 0.01, &mut Dirty);
                    HSD_JObjAddScaleZ(&mut *jobj_next, 0.01, &mut Dirty);
                    _Toy_803062EC((*anim).xC as i32, 3, HSD_JObjGetScaleX(&*jobj_next));
                    if (Toy_80305C44() & 8) != 0 {
                        (*state).x60 = 8;
                    }
                } else if adj_y < -0.8 || (Toy_80305C44() & 4) != 0 {
                    HSD_JObjAddScaleX(&mut *jobj_next, -0.01, &mut Dirty);
                    HSD_JObjAddScaleY(&mut *jobj_next, -0.01, &mut Dirty);
                    HSD_JObjAddScaleZ(&mut *jobj_next, -0.01, &mut Dirty);
                    _Toy_803062EC((*anim).xC as i32, 3, HSD_JObjGetScaleX(&*jobj_next));
                    if (Toy_80305C44() & 4) != 0 {
                        (*state).x60 = 8;
                    }
                } else if adj_x > 0.8 || (Toy_80305C44() & 2) != 0 {
                    HSD_JObjAddScaleX(&mut *jobj_child, 0.01, &mut Dirty);
                    HSD_JObjAddScaleZ(&mut *jobj_child, 0.01, &mut Dirty);
                    let n2 = (*jobj_next).next;
                    HSD_JObjAddScaleX(&mut *n2, 0.01, &mut Dirty);
                    HSD_JObjAddScaleZ(&mut *n2, 0.01, &mut Dirty);
                    _Toy_803062EC((*anim).xC as i32, 4, HSD_JObjGetScaleX(&*n2));
                    if (Toy_80305C44() & 2) != 0 {
                        (*state).x60 = 8;
                    }
                } else if adj_x < -0.8 || (Toy_80305C44() & 1) != 0 {
                    HSD_JObjAddScaleX(&mut *jobj_child, -0.01, &mut Dirty);
                    HSD_JObjAddScaleZ(&mut *jobj_child, -0.01, &mut Dirty);
                    let n2 = (*jobj_next).next;
                    HSD_JObjAddScaleX(&mut *n2, -0.01, &mut Dirty);
                    HSD_JObjAddScaleZ(&mut *n2, -0.01, &mut Dirty);
                    _Toy_803062EC((*anim).xC as i32, 4, HSD_JObjGetScaleX(&*n2));
                    if (Toy_80305C44() & 1) != 0 {
                        (*state).x60 = 8;
                    }
                }
            } else {
                if (*state).x61 == 0 {
                    let mut do_mode_switch = false;
                    let bm = Toy_80305C44() as u32;
                    if bm != 8 {
                        let bm2 = Toy_80305C44() as u32;
                        if bm2 != 4 {
                            let bm3 = Toy_80305C44() as u32;
                            if bm3 != 1 {
                                let bm4 = Toy_80305C44() as u32;
                                if bm4 == 2 {
                                    do_mode_switch = true;
                                }
                            } else {
                                do_mode_switch = true;
                            }
                        } else {
                            do_mode_switch = true;
                        }
                    } else {
                        do_mode_switch = true;
                    }
                    if do_mode_switch {
                        match _Toy_sbss_804D6E60 {
                            0 | 1 | 2 => {
                                ext::lbLang_IsSavedLanguageJP();
                            }
                            _ => {}
                        }
                    }
                }
                (*state).x1C = -fmadds(3.0, adj_x, -(*state).x1C); // fused
                (*state).x18 = -fmadds(3.0, adj_y, -(*state).x18); // fused
                if (*state).x18 < -89.0 {
                    (*state).x18 = -89.0;
                }
                if (*state).x18 > 89.0 {
                    (*state).x18 = 89.0;
                }
                {
                    let yaw = (*state).x1C;
                    if yaw < -360.0 {
                        (*state).x1C += 360.0;
                    }
                }
                {
                    let yaw = (*state).x1C;
                    if yaw > 360.0 {
                        (*state).x1C = yaw - 360.0;
                    }
                }
                (*ed4).x18 = (*state).x1C;
                (*ed4).x14 = (*state).x18;
            }
        }
    }
    {
        let moved_x = 0.3f32 * adj_sx;
        let moved_y = 0.3f32 * adj_sy;
        trigger = Toy_80305B88() as u32;
        if trigger & HSD_PAD_START as u32 != 0 {
            (*state).x58 = 0;
            (*ed4).x10 = (*ed4).x10 + 1;
            if (*ed4).x10 == 6 {
                (*ed4).x10 = 0;
            }
            Toy_80306D70((*ed4).x10);
            _Toy_803075E8((*ed4).x10);
        }
        _Toy_80308DC8(cobj);
        trigger = Toy_80305B88() as u32;
        if trigger & (HSD_PAD_L | HSD_PAD_R) as u32 != 0 {
            let display = Toy_sbss_804D6EE0;
            let trig2 = Toy_80305B88() as u32;
            if trig2 & 0x441 != 0 {
                cycle_prev(display, &mut sym_out);
            } else {
                cycle_next(display, &mut sym_out);
            }
            _Toy_80307828(0);
            _Toy_8030715C(0.0, 0.0);
            (*state).x58 = 0x95E;
            Toy_803087F4((*display).selected_entry);
            _Toy_803084A0((*(*display).selected_entry).trophy_id as i32);
            Toy_803083D8((*Toy_sbss_804D6ED8).x30, (*(*display).selected_entry).trophy_id as i32);
        }
        trigger = Toy_80305B88() as u32;
        if trigger & HSD_PAD_START as u32 != 0 {
            _Toy_80307828(0);
        }
        {
            let mut top = ext::HSD_CObjGetTop(cobj);
            let mut bottom = ext::HSD_CObjGetBottom(cobj);
            let right = ext::HSD_CObjGetRight(cobj);
            let left = ext::HSD_CObjGetLeft(cobj);
            let p1 = &raw mut ext::HSD_PadCopyStatus as *mut HSD_PadStatus;
            let p1 = p1.add(1);
            if (*p1).trigger & HSD_PAD_START as u32 != 0 {
                ext::OSReport(&format!("top = {:.6}, bottom = {:.6}, right = {:.6}, left = {:.6}\n", top as f64, bottom as f64, right as f64, left as f64));
                return;
            }
            if (*p1).button & HSD_PAD_DPADUP as u32 != 0 {
                top += 0.001;
                bottom += 0.001;
                ext::HSD_CObjSetTop(cobj, top);
                ext::HSD_CObjSetBottom(cobj, bottom);
                ext::OSReport(&format!("top = {:.6}, bottom = {:.6}\n", top as f64, bottom as f64));
                return;
            }
            if (*p1).button & HSD_PAD_DPADDOWN as u32 != 0 {
                top -= 0.001;
                bottom -= 0.001;
                ext::HSD_CObjSetTop(cobj, top);
                ext::HSD_CObjSetBottom(cobj, bottom);
                ext::OSReport(&format!("top = {:.6}, bottom = {:.6}\n", top as f64, bottom as f64));
                return;
            }
            if (*p1).button & HSD_PAD_DPADLEFT as u32 != 0 {
                let mut nr = ext::HSD_CObjGetRight(cobj);
                let mut nl = ext::HSD_CObjGetLeft(cobj);
                nr -= 0.001;
                nl -= 0.001;
                ext::HSD_CObjSetRight(cobj, nr);
                ext::HSD_CObjSetLeft(cobj, nl);
                ext::OSReport(&format!("right = {:.6}, left = {:.6}\n", nr as f64, nl as f64));
                return;
            }
            if (*p1).button & HSD_PAD_DPADRIGHT as u32 != 0 {
                let mut nr = ext::HSD_CObjGetRight(cobj);
                let mut nl = ext::HSD_CObjGetLeft(cobj);
                nr += 0.001;
                nl += 0.001;
                ext::HSD_CObjSetRight(cobj, nr);
                ext::HSD_CObjSetLeft(cobj, nl);
                ext::OSReport(&format!("right = {:.6}, left = {:.6}\n", nr as f64, nl as f64));
                return;
            }
            _Toy_8030715C(moved_x, moved_y);
        }
    }
    let _ = (&mut button, &mut sym_out);
}

/// Lighting debug camera proc (`MELEE` debug chord with L held).
pub unsafe extern "C" fn _Toy_8030E110(arg0: *mut HSD_GObj) {
    let _base = &raw mut _Toy_804A26B8 as *mut Toy26B8;
    let anim = &raw mut Toy_804A2AA8;
    let state = _Toy_sbss_804D6E68;
    let ed4 = Toy_sbss_804D6ED4;
    let cobj = (*arg0).hsd_obj as *mut HSD_CObj;
    let mut moved_y: f32 = 0.0;
    let mut moved_x: f32 = 0.0;
    let mut sym_out: *mut c_void = core::ptr::null_mut();
    let mut sp134: Vec3;
    let mut sp128: Vec3;
    let mut sp140 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut tmp: f32;
    let mut trigger: u32;
    let mut button: u32;

    {
        let mut val: f32 = 0.0;
        for i in 0..4 {
            val = (*pad(i as u8)).nml_stickX;
            let abs = if val < 0.0 { -val } else { val };
            if abs > 0.1 {
                break;
            }
        }
        (*state).x30 = val;
    }
    (*state).x34 = Toy_80305DB0();
    (*state).x54 = 0.0;
    (*state).x50 = 0.0;
    (*state).x30 = stick_remap((*state).x30);
    (*state).x34 = stick_remap((*state).x34);
    (*state).x40 = Toy_80305EB4();
    (*state).x44 = Toy_80305FB8();
    (*state).x40 = stick_remap((*state).x40);
    (*state).x44 = stick_remap((*state).x44);
    {
        let cd = (*state).x60 as u8;
        if cd as i8 != 0 {
            (*state).x60 = cd.wrapping_sub(1) as i8;
            _Toy_8030715C(0.0, 0.0);
            return;
        }
    }

    // `case_default:` body, shared by every path of the C `switch`.
    let mut run_default = true;
    match (*state).x61 {
        0 => {
            trigger = Toy_80305B88() as u32;
            if trigger & HSD_PAD_B as u32 != 0 {
                sfxBack();
                Toy_80310660(1);
                ext::HSD_GObj_80390CD4(arg0);
                ext::tyList_803147C4();
                return;
            }
            {
                let tc = trophy_count_now();
                if tc == 0 {
                    return;
                }
                trigger = Toy_80305B88() as u32;
                if trigger & (HSD_PAD_A | HSD_PAD_X | HSD_PAD_Y) as u32 != 0 {
                    sfxForward();
                    _Toy_sbss_804D6E80 = ext::HSD_CObjGetTop(cobj);
                    _Toy_sbss_804D6E84 = ext::HSD_CObjGetBottom(cobj);
                    _Toy_sbss_804D6E88 = ext::HSD_CObjGetRight(cobj);
                    _Toy_sbss_804D6E8C = ext::HSD_CObjGetLeft(cobj);
                    (*state).x5C = 0;
                    (*state).x61 = 1;
                    _Toy_8030715C(0.0, 0.0);
                    return;
                }
            }
        }
        1 | 3 => {
            _Toy_80308F04(cobj);
        }
        2 => {
            trigger = Toy_80305B88() as u32;
            if trigger & HSD_PAD_B as u32 != 0 || (*state).x5C as f32 > 7200.0 {
                sfxBack();
                _Toy_sbss_804D6E80 = ext::HSD_CObjGetTop(cobj);
                _Toy_sbss_804D6E84 = ext::HSD_CObjGetBottom(cobj);
                _Toy_sbss_804D6E88 = ext::HSD_CObjGetRight(cobj);
                _Toy_sbss_804D6E8C = ext::HSD_CObjGetLeft(cobj);
                let mut jobj_node = (*(*Toy_sbss_804D6ED8).gobj2).hsd_obj as *mut HSD_SObj;
                while !jobj_node.is_null() {
                    (*jobj_node).x40 = 9;
                    jobj_node = (*jobj_node).next;
                }
                (*state).x5C = 0;
                sp134 = _Toy_803B88F8;
                sp128 = _Toy_803B8904;
                ext::HSD_CObjGetEyePosition(cobj, &mut sp140);
                sp134.y = 8.0;
                ext::HSD_CObjGetInterest(cobj, &mut sp128);
                let base = &raw mut _Toy_804A26B8 as *mut Toy26B8;
                (*base).x0.x = (sp134.x - sp128.x) / 10.0;
                (*base).x0.y = (sp134.y - sp128.y) / 10.0;
                (*base).x0.z = (sp134.z - sp128.z) / 10.0;
                _Toy_sbss_804D6E90 = ((*state).x20 - 38.0) / 10.0;
                _Toy_sbss_804D6E94 = (*state).x18 / 10.0;
                (*state).x61 = 3;
            }
            trigger = Toy_80305B88() as u32;
            tmp = (*state).x44 + ((*state).x40 + ((*state).x30 + (*state).x34));
            if tmp + (trigger as f32) == 0.0 {
                (*state).x5C = (*state).x5C + 1;
            } else {
                (*state).x5C = 0;
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_A as u32 != 0 {
                let sx = (*state).x30;
                if sx != 0.0 && sx < 0.0 {
                    (*state).x50 = 0.3f32 * sx * ((*state).x20 / 38.0);
                }
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_A as u32 != 0 {
                let sx = (*state).x30;
                if sx != 0.0 && sx > 0.0 {
                    (*state).x50 = 0.3f32 * sx * ((*state).x20 / 38.0);
                }
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_A as u32 != 0 {
                let sy = (*state).x34;
                if sy != 0.0 && sy > 0.0 {
                    (*state).x54 = 0.3f32 * sy * ((*state).x20 / 38.0);
                }
            }
            button = Toy_80305C44() as u32;
            if button & HSD_PAD_A as u32 != 0 {
                let sy = (*state).x34;
                if sy != 0.0 && sy < 0.0 {
                    (*state).x54 = 0.3f32 * sy * ((*state).x20 / 38.0);
                }
            }
            if (*state).x50 != 0.0 || (*state).x54 != 0.0 || (Toy_80305C44() as u32 & HSD_PAD_A as u32) != 0 {
                _Toy_803102C4(0);
            } else {
                _Toy_803102C4(1);
            }
            if (*state).x50 != 0.0 || (*state).x54 != 0.0 {
                moved_x = 1.0;
            }
            if moved_x == 0.0 {
                let sy = (*state).x34;
                if sy != 0.0 {
                    (*state).x20 = fmadds(-(*state).x20, 0.025f32 * sy, (*state).x20); // fused
                    moved_y = 1.0;
                }
                if (*state).x20 < 5.0 {
                    (*state).x20 = 5.0;
                }
                if (*state).x20 > 250.0 {
                    (*state).x20 = 250.0;
                }
            }
        }
        _ => {}
    }
    let _ = &mut run_default;

    // case_default:
    trigger = Toy_80305B88() as u32;
    if trigger & HSD_PAD_Y as u32 != 0 {
        _Toy_sbss_804D6E54 ^= 1;
    }
    (*state).x1C = -fmadds(3.0, (*state).x40, -(*state).x1C); // fused
    (*state).x18 = -fmadds(3.0, (*state).x44, -(*state).x18); // fused
    if (*state).x18 < -89.0 {
        (*state).x18 = -89.0;
    }
    if (*state).x18 > 89.0 {
        (*state).x18 = 89.0;
    }
    tmp = (*state).x1C;
    if tmp < -360.0 {
        (*state).x1C += 360.0;
    }
    tmp = (*state).x1C;
    if tmp > 360.0 {
        (*state).x1C = tmp - 360.0;
    }
    (*ed4).x18 = (*state).x1C;
    (*ed4).x14 = (*state).x18;
    if (*state).x40 + (*state).x44 != 0.0 {
        (*state).x24 = 0.0;
    }
    if moved_x == 0.0 && moved_y == 0.0 {
        trigger = Toy_80305B88() as u32;
        if trigger & (HSD_PAD_L | HSD_PAD_R) as u32 != 0 {
            let display = Toy_sbss_804D6EE0;
            if trophy_count_now() == 1 {
                (*state).x58 = 0x961;
                return;
            }
            if (*state).x30 < 0.0 || (Toy_80305B88() & 0x441) != 0 {
                cycle_prev(display, &mut sym_out);
            } else if (*state).x30 > 0.0 || (Toy_80305B88() & 0x822) != 0 {
                cycle_next(display, &mut sym_out);
            }
            _Toy_80307828(0);
            (*state).x58 = 0x95E;
            Toy_803087F4((*display).selected_entry);
            (*state).x60 = 0x14;
        }
    }
    // after_trophy_cycle:
    trigger = Toy_80305B88() as u32;
    if trigger & HSD_PAD_START as u32 != 0 {
        sfxMove();
        (*state).x58 = 0;
        (*ed4).x10 = (*ed4).x10 + 1;
        if (*ed4).x10 == 6 {
            (*ed4).x10 = 0;
        }
        Toy_80306D70((*ed4).x10);
        _Toy_803075E8((*ed4).x10);
    }
    trigger = Toy_80305B88() as u32;
    if trigger & HSD_PAD_Z as u32 != 0 {
        _Toy_80307828(0);
        let ed4_2 = Toy_sbss_804D6ED4;
        (*ed4_2).x10 = 0;
        Toy_80306D70((*ed4_2).x10);
        _Toy_803075E8((*ed4_2).x10);
        (*anim).x11 = 0;
        (*anim).x10 = 0;
        (*anim).x0E = 1;
    }
    {
        ext::HSD_JObjSetFlagsAll((*anim).jobj[0], JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlagsAll((*anim).jobj[1], JOBJ_HIDDEN as u32);
    }
    _Toy_8030715C((*state).x50, (*state).x54);
    (*state).x38 = (*state).x30;
    (*state).x3C = (*state).x34;
    (*state).x48 = (*state).x40;
    (*state).x4C = (*state).x44;
    let _ = sym_out;
}

// ---------------------------------------------------------------------------------------------
// Scene setup: cameras, lights, list, debug windows, mode entry/exit
// ---------------------------------------------------------------------------------------------

/* 3FE604 */
static mut _Toy_803FE604: HSD_WObjDesc = HSD_WObjDesc { class_name: core::ptr::null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 38.0 }, robjdesc: core::ptr::null_mut() };
/* 3FE618 */
static mut _Toy_803FE618: HSD_WObjDesc = HSD_WObjDesc { class_name: core::ptr::null_mut(), pos: Vec3 { x: 8.0, y: 4.0, z: 0.0 }, robjdesc: core::ptr::null_mut() };
/* 3FE62C */
static mut _Toy_803FE62C: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
    class_name: core::ptr::null_mut(),
    flags: 0,
    projection_type: PROJ_PERSPECTIVE as u16,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
    scissor: _Scissor { left: 0, right: 640, top: 0, bottom: 480 },
    eyepos: &raw mut _Toy_803FE604,
    interest: &raw mut _Toy_803FE618,
    roll: 0.0,
    up_vector: core::ptr::null_mut(),
    nnear: 0.1,
    ffar: 32768.0,
    fov: 40.0,
    aspect: 913.0f32 / 750.0f32,
};
/* 3FE664 */
static mut _Toy_803FE664: HSD_WObjDesc = HSD_WObjDesc { class_name: core::ptr::null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 50.0 }, robjdesc: core::ptr::null_mut() };
/* 3FE678 */
static mut _Toy_803FE678: HSD_WObjDesc = HSD_WObjDesc { class_name: core::ptr::null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, robjdesc: core::ptr::null_mut() };
/* 3FE68C */
static mut _Toy_803FE68C: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
    class_name: core::ptr::null_mut(),
    flags: 0,
    projection_type: PROJ_PERSPECTIVE as u16,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
    scissor: _Scissor { left: 0, right: 640, top: 0, bottom: 480 },
    eyepos: &raw mut _Toy_803FE664,
    interest: &raw mut _Toy_803FE678,
    roll: 0.0,
    up_vector: core::ptr::null_mut(),
    nnear: 0.1,
    ffar: 32768.0,
    fov: 40.0,
    aspect: 913.0f32 / 750.0f32,
};

/// Build the scene's cameras (state record slots 0..5 of `Toy_sbss_804D6E68`), the screen
/// quads, and attach the per-frame viewer proc.
pub unsafe fn _Toy_8030FA50() {
    let mut eye = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut mtx: Mtx = core::mem::zeroed();
    let ed8 = Toy_sbss_804D6ED8;
    // The C treats the record as `void*[]`: slot n at GameCube +4n. 0,1,2,3 = x0,x4,x8,xC; 4,5 = x10[0..2].
    let state = _Toy_sbss_804D6E68;
    let cam_desc = ext::HSD_ArchiveGetPublicAddress((*ed8).archive, Toy_str_ScMenFigure_cam_int1_camera.as_ptr() as *const i8)
        as *mut HSD_CameraDescPerspective;
    macro_rules! slot {
        (0) => { (*state).x0 };
        (1) => { (*state).x4 };
        (2) => { (*state).x8 };
        (3) => { (*state).xC };
        (4) => { (*state).x10[0] };
        (5) => { (*state).x10[1] };
    }
    macro_rules! gobj_of {
        ($n:tt) => { slot!($n) as *mut HSD_GObj };
    }

    /* Screen camera (offset 0x14) */
    slot!(5) = ext::GObj_Create(1, 2, 0) as _;
    ext::HSD_SObjLib_803A55DC(gobj_of!(5), 0x280, 0x1E0, 0);
    (*gobj_of!(5)).gxlink_prios = 0x0004000000000000u64;
    /* Main CObj (offset 0x00) */
    (*state).x0 = ext::GObj_Create(1, 2, 0) as *mut ToyTransitionObj;
    let cobj = ext::lb_80013B14(cam_desc);
    ext::HSD_GObjObject_80390A70(gobj_of!(0), ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj_of!(0), Some(Toy_80306954), 0);
    (*gobj_of!(0)).gxlink_prios = 0x1048000000000000u64;
    /* Screen2 (offset 0x10) */
    slot!(4) = ext::GObj_Create(1, 2, 0) as _;
    ext::HSD_SObjLib_803A55DC(gobj_of!(4), 0x280, 0x1E0, 0);
    (*gobj_of!(4)).gxlink_prios = 0x0100000000000000u64;
    /* Frustum camera (offset 0x08) */
    slot!(2) = ext::GObj_Create(1, 2, 0) as _;
    let cobj = ext::lb_80013B14(&raw mut _Toy_803FE62C);
    let frustum_top = 0.049584f32;
    let frustum_bottom = -0.035585f32;
    let frustum_far = 0.076839f64 as f32;
    let frustum_near = -0.026839f32;
    ext::HSD_CObjSetFrustum(cobj, frustum_top, frustum_bottom, frustum_near, frustum_far);
    ext::HSD_GObjObject_80390A70(gobj_of!(2), ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj_of!(2), Some(Toy_803068E0), 0);
    (*gobj_of!(2)).gxlink_prios = 0x0E80000000000000u64;
    if _Toy_sbss_804D6EA2 != 0 {
        _Toy_sbss_804D6E60 = 0;
        ext::DevText_SetXY(_Toy_sbss_804D6E9C, 0x21C, 0x82);
        ext::HSD_GObj_SetupProc(gobj_of!(2), Some(_Toy_8030B530), 0);
    } else if _Toy_sbss_804D6E50 != 0 {
        ext::HSD_GObj_SetupProc(gobj_of!(2), Some(_Toy_8030E110), 0);
    } else {
        ext::HSD_GObj_SetupProc(gobj_of!(2), Some(_Toy_80309404), 0);
    }
    ext::HSD_GObj_80390CD4(gobj_of!(2));
    (*state).x61 = 0;
    (*state).x60 = 4;
    _Toy_80307828(0);
    (*_Toy_sbss_804D6E68).x58 = 0x95E;
    /* Camera2 (offset 0x04) */
    slot!(1) = ext::GObj_Create(1, 2, 0) as _;
    let cobj = ext::lb_80013B14(&raw mut _Toy_803FE68C);
    ext::HSD_GObjObject_80390A70(gobj_of!(1), ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj_of!(1), Some(Toy_803068E0), 0);
    (*gobj_of!(1)).gxlink_prios = 0x8000000000000000u64;
    _Toy_sbss_804D6E70 = ext::HSD_SisLib_803A611C(0, gobj_of!(1), 0xB, 0xC, 0, 0x3F, 0, 0);
    ext::HSD_CObjGetEyePosition(cobj, &mut eye);
    eye.y = 0.0;
    eye.x = 0.0;
    eye.z = ext::HSD_CObjGetEyeDistance(cobj);
    ext::MTXRotRad(mtx.as_mut_ptr() as *mut c_void, b'y' as i8, 0.57595867f32);
    ext::PSMTXMultVecSR(mtx.as_mut_ptr() as *mut c_void, &mut eye, &mut eye);
    ext::HSD_CObjSetEyePosition(cobj, &mut eye);
    /* Light camera (offset 0x0C) */
    slot!(3) = ext::GObj_Create(1, 2, 0) as _;
    let cobj = ext::lb_80013B14(cam_desc);
    ext::HSD_GObjObject_80390A70(gobj_of!(3), ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj_of!(3), Some(ext::cb::HSD_GObj_803910D8), 0);
    (*gobj_of!(3)).gxlink_prios = 0x4000000000000000u64;
    _Toy_sbss_804D6E74 = ext::HSD_SisLib_803A611C(3, gobj_of!(3), 0xC, 0xB, 0, 0x3E, 0, 0);
    _Toy_sbss_804D6E78 = ext::HSD_SisLib_803A611C(3, gobj_of!(3), 0xD, 0xB, 0, 0x3E, 0, 0);
    _Toy_sbss_804D6E7C = ext::HSD_SisLib_803A611C(3, gobj_of!(3), 0xE, 0xB, 0, 0x3E, 0, 0);
    if _Toy_sbss_804D6E50 != 0 {
        (*gobj_of!(0)).gxlink_prios = 0;
        (*gobj_of!(1)).gxlink_prios = 0;
        (*gobj_of!(3)).gxlink_prios = 0;
        (*gobj_of!(4)).gxlink_prios = 0;
        (*gobj_of!(5)).gxlink_prios = 0;
        (*gobj_of!(2)).gxlink_prios = 0x02A0000000000000u64;
    }
}

/// `_Toy_8030FE48_link_entries`: wire the display entries into a circular list.
#[inline]
unsafe fn _Toy_8030FE48_link_entries(data: *mut TyDisplayData, entry_count: i32) {
    let last = entry_count - 1;
    let mut entry = (*data).entries.as_mut_ptr();
    let last_entry = &raw mut (*data).entries[last as usize];
    for i in 0..entry_count {
        if i == 0 {
            (*entry).prev = last_entry;
        } else {
            (*entry).prev = &raw mut (*data).entries[(i - 1) as usize];
        }
        if i == last {
            (*entry).next = (*data).entries.as_mut_ptr();
        } else {
            (*entry).next = &raw mut (*data).entries[(i + 1) as usize];
        }
        entry = entry.add(1);
    }
}

pub unsafe fn _Toy_8030FE48(data: *mut TyDisplayData, arg1: i32) {
    let sel = toy_u16(0x12A) as *mut i16; // TOY_SEL
    let mut sort_idx: i32 = 0;
    let mut sym: *mut c_void = core::ptr::null_mut();
    let mut sym2: *mut c_void = core::ptr::null_mut();

    // _Toy_8030FE48_init_sort_key: &keys[sort_mode] (sort_mode = TOY_X195).
    let keys = (*_Toy_sbss_804D6E64).key.as_mut_ptr();
    let sort_mode = *(toy_byte(1) as *mut i8) as i32;
    let mut ptr = keys.offset(sort_mode as isize);

    (*data).selectedIdx = *sel;
    while sort_idx < _Toy_GetTrophyTotal() {
        if *(toy_u16(0x12B) as *mut i16) == *ptr {
            (*data).selectedIdx = sort_idx as i16;
            *sel = (*data).selectedIdx;
            break;
        }
        ptr = ptr.add(TY_SORT_KEY_COUNT as usize);
        sort_idx += 1;
    }
    let count = _Toy_GetTrophyTotal();
    let mut entry_count = count;
    if count > 3 {
        (*data).visible_count = 3;
        entry_count = 0xD;
    } else {
        (*data).visible_count = count as i8;
    }
    if arg1 == 0 {
        _Toy_8030FE48_link_entries(data, entry_count);
    }
    for start in 0..0xDusize {
        (*data).entries[start].archive = core::ptr::null_mut();
    }
    if (*data).visible_count as i32 == entry_count {
        let mut start: i32 = 0;
        while start < (*data).visible_count as i32 {
            let e = &raw mut (*data).entries[start as usize];
            if start == (*data).selectedIdx as i32 {
                setupTrophyEntry(e, *Toy_sbss_804D6EDC.offset(start as isize));
                (*e).archive = ext::lbArchive_LoadSymbols(
                    (*e).archive_name,
                    &[(&raw mut sym) as *mut c_void, (*e).symbol_name as *mut c_void],
                );
            } else {
                setupTrophyEntry(e, *Toy_sbss_804D6EDC.offset(start as isize));
            }
            start += 1;
        }
        (*data).selected_entry = &raw mut (*data).entries[(*data).selectedIdx as usize];
        (*data).first_entry = (*data).entries.as_mut_ptr();
        (*data).last_entry = &raw mut (*data).entries[((*data).visible_count - 1) as usize];
        return;
    }
    {
        let prev_idx = (*data).selectedIdx as i32 - 1;
        let mut start_idx: i32;
        if prev_idx < 0 {
            let tcount = trophy_count_now();
            start_idx = tcount + (*data).selectedIdx as i32;
            start_idx -= 1;
        } else {
            start_idx = prev_idx;
        }
        (*data).selected_entry = (*data).entries.as_mut_ptr();
        let mut loop_count: i32 = 0;
        (*data).first_entry = (*(*data).selected_entry).prev;
        (*data).last_entry = (*(*data).selected_entry).next;
        let mut cur = (*data).first_entry;
        while loop_count < (*data).visible_count as i32 {
            let tcount2 = trophy_count_now();
            if start_idx >= tcount2 {
                start_idx = 0;
            }
            if start_idx == (*data).selectedIdx as i32 {
                setupTrophyEntry(cur, *Toy_sbss_804D6EDC.offset(start_idx as isize));
                (*cur).archive = ext::lbArchive_LoadSymbols(
                    (*cur).archive_name,
                    &[(&raw mut sym2) as *mut c_void, (*cur).symbol_name as *mut c_void],
                );
            } else {
                setupTrophyEntry(cur, *Toy_sbss_804D6EDC.offset(start_idx as isize));
            }
            cur = (*cur).next;
            start_idx += 1;
            loop_count += 1;
        }
    }
}

pub unsafe fn _Toy_803102C4(arg0: i8) {
    (*_Toy_sbss_804D6E6C).x4 = arg0;
}

pub unsafe fn Toy_803102D0() {
    if _Toy_sbss_804D6ECC.is_null() {
        _Toy_sbss_804D6ECC = ext::lbArchive_LoadSymbols(
            cstr!("TyDataf.dat"),
            &[
                (&raw mut _Toy_sbss_804D6EA8) as *mut c_void,
                cstr!("tyModelFileTbl") as *mut c_void,
                (&raw mut _Toy_sbss_804D6EA4) as *mut c_void,
                cstr!("tyModelFileUsTbl") as *mut c_void,
            ],
        );
    }
}

/// `toy_toggle_flag` (static inline): clear the "new" bit of the list's selected trophy.
#[inline]
unsafe fn toy_toggle_flag() {
    let tg6 = Toy_sbss_804D6EE0;
    let idx = *Toy_sbss_804D6EDC.offset((*tg6).selectedIdx as isize);
    let mut flags = flags_now();
    if *flags.offset(idx as isize) & 0x8000 != 0 {
        let tg6 = Toy_sbss_804D6EE0;
        let idx2 = *Toy_sbss_804D6EDC.offset((*tg6).selectedIdx as isize);
        flags = flags_now();
        flags = flags.offset(idx2 as isize);
        *flags ^= 0x8000;
    }
}

/// `toy_make_gobj` (static inline): the axis-gizmo gobj.
#[inline]
unsafe fn toy_make_gobj() {
    let tg3 = _Toy_sbss_804D6E6C;
    (*tg3).gobj = ext::GObj_Create(6, 7, 0);
    ext::GObj_SetupGXLink((*tg3).gobj, Some(_Toy_80312050), 0x39, 0);
    (*tg3).x4 = 1;
}

/// `toy_sobj_loop` (static inline): the three background sobjs.
#[inline]
unsafe fn toy_sobj_loop(tg2: *mut ToyED8Data, syms: &[*mut c_void; 3]) {
    let two = 2.0f32;
    let one = 1;
    for i in 0..3 {
        let sobj = ext::HSD_SObjLib_803A477C((*tg2).x8 as *mut HSD_GObj, syms[i] as *mut HSD_SObjDesc, 0, 0, 0x80, 0);
        (*sobj).x1C = two;
        (*sobj).x20 = two;
        (*sobj).x40 = one;
    }
}

pub unsafe fn Toy_80310324() {
    let tg = Toy_sbss_804D6ED8;
    let mut sym: [*mut c_void; 1] = [core::ptr::null_mut()];
    let mut syms: [*mut c_void; 3] = [core::ptr::null_mut(); 3];
    _Toy_8030663C();
    Toy_803067BC(*(toy_byte(1) as *mut i8) as i32, *(toy_byte(2) as *mut i8) as i32);
    if (*tg).archive.is_null() {
        // Retail writes the symbol through `sym + 4`, past a one-element local; here it is
        // written inside the array and discarded (only the returned archive is kept).
        (*tg).archive = ext::lbArchive_LoadSymbols(
            if ext::lbLang_IsSavedLanguageJP() != 0 { cstr!("TyMnView.dat") } else { cstr!("TyMnView.usd") },
            &[sym.as_mut_ptr() as *mut c_void, _Toy_803FDEA0[0] as *mut c_void],
        );
    }
    ext::memzero(_Toy_sbss_804D6E68 as *mut c_void, core::mem::size_of::<Toy6E68>() as i32);
    _Toy_8030FA50();
    ext::memzero(Toy_sbss_804D6ED4 as *mut c_void, core::mem::size_of::<ToyCameraControl>() as i32);
    Toy_80306D70(0);
    _Toy_80307018();
    let tg2 = Toy_sbss_804D6ED8;
    if (*tg2).x54 == 0 {
        (*tg2).x54 = ext::lbArchive_LoadSymbols(
            cstr!("TyMnBg.dat"),
            &[
                (&raw mut syms[0]) as *mut c_void, _Toy_803FE038[0] as *mut c_void,
                (&raw mut syms[1]) as *mut c_void, _Toy_803FE038[1] as *mut c_void,
                (&raw mut syms[2]) as *mut c_void, _Toy_803FE038[2] as *mut c_void,
            ],
        ) as usize;
        (*tg2).x8 = ext::GObj_Create(4, 5, 0) as *mut ToyDataX8;
        ext::GObj_SetupGXLink((*tg2).x8 as *mut HSD_GObj, Some(ext::cb::HSD_SObjLib_803A49E0), 0x32, 0);
        toy_sobj_loop(tg2, &syms);
    }
    _Toy_803075E8(0);
    Toy_80307470(0);
    _Toy_803078E4();
    let mut sobj = (*(*Toy_sbss_804D6ED8).gobj2).hsd_obj as *mut HSD_SObj;
    while !sobj.is_null() {
        (*sobj).x40 = 9;
        sobj = (*sobj).next;
    }
    let var_r0 = trophy_count_now();
    if var_r0 != 0 {
        ext::memzero(&raw mut Toy_804A2AA8 as *mut c_void, core::mem::size_of::<ToyAnimState>() as i32);
        _Toy_8030FE48(Toy_sbss_804D6EE0, 0);
        let tg6 = Toy_sbss_804D6EE0;
        Toy_803087F4((*tg6).selected_entry);
        toy_toggle_flag();
        let tg6 = Toy_sbss_804D6EE0;
        let sub = (*tg6).selected_entry;
        _Toy_803084A0((*sub).trophy_id as i32);
        let tg6 = Toy_sbss_804D6EE0;
        let sub = (*tg6).selected_entry;
        Toy_803083D8((*tg).x30, (*sub).trophy_id as i32);
        toy_make_gobj();
    }
    _Toy_80307828(0);
    let tg5 = _Toy_sbss_804D6E68;
    (*tg5).x58 = 0x95E;
    _Toy_8030715C(0.0, 0.0);
    let tg4 = Toy_sbss_804D6ED4;
    (*tg4).x10 = 0;
    Toy_80306D70((*tg4).x10);
    _Toy_803075E8((*tg4).x10);
    ext::HSD_PadRenewStatus();
}

/// Tear the viewer down (`arg0 != 0` also releases every archive and gobj).
pub unsafe fn Toy_80310660(arg0: i32) {
    let mut arg = arg0;
    let ty27 = &raw mut Toy_804A2AA8;
    let ty31 = _Toy_sbss_804D6E68;
    let ty28 = Toy_sbss_804D6ED4;
    let ty26 = _Toy_sbss_804D6E6C;
    let ty25 = Toy_sbss_804D6EE0;
    let ty30 = Toy_sbss_804D6ED8;
    let mut idx: i32 = trophy_count_now();
    if idx != 0 {
        idx = *Toy_sbss_804D6EDC.offset((*ty25).selectedIdx as isize) as i32;
        let mut ptr = flags_now();
        if *ptr.offset(idx as isize) & 0x8000 != 0 {
            idx = *Toy_sbss_804D6EDC.offset((*ty25).selectedIdx as isize) as i32;
            ptr = flags_now();
            ptr = ptr.offset(idx as isize);
            *ptr ^= 0x8000;
        }
        *(toy_u16(0x12A) as *mut i16) = (*ty25).selectedIdx; // TOY_SEL
        *(toy_u16(0x12B) as *mut i16) = (*(*ty25).selected_entry).trophy_id; // TOY_SELID
    }
    if arg != 0 {
        ext::HSD_SisLib_803A5E70();
        (*ty25).texts[3] = core::ptr::null_mut();
        (*ty25).texts[2] = core::ptr::null_mut();
        (*ty25).texts[1] = core::ptr::null_mut();
        (*ty25).texts[0] = core::ptr::null_mut();
        let idx = trophy_count_now();
        if idx != 0 {
            arg = 0;
            for count in 0..0xDusize {
                let lp = &raw mut (*ty25).entries[count];
                if !(*lp).archive.is_null() {
                    ext::lbArchive_80016EFC((*lp).archive);
                    (*lp).archive = arg as usize as *mut HSD_Archive;
                }
            }
        }
        if !Toy_sbss_804D6EC8.is_null() {
            ext::lbArchive_80016EFC(Toy_sbss_804D6EC8);
            Toy_sbss_804D6EC8 = core::ptr::null_mut();
        }
        if !(*ty28).archive.is_null() {
            ext::lbArchive_80016EFC((*ty28).archive);
            (*ty28).archive = core::ptr::null_mut();
        }
        if !(*ty30).x58.is_null() {
            ext::lbArchive_80016EFC((*ty30).x58 as *mut HSD_Archive);
            arg = 0;
            (*ty30).x58 = arg as usize as *mut c_void;
            if !(*ty30).gobj2.is_null() {
                ext::HSD_GObjFree((*ty30).gobj2);
                (*ty30).gobj2 = arg as usize as *mut HSD_GObj;
            }
        }
        if !(*ty27).gobj.is_null() {
            ext::HSD_GObjFree((*ty27).gobj);
            (*ty27).gobj = core::ptr::null_mut();
            (*ty27).jobj[1] = core::ptr::null_mut();
            (*ty27).jobj[0] = core::ptr::null_mut();
        }
        if !(*ty26).gobj.is_null() {
            ext::HSD_GObjFree((*ty26).gobj);
            (*ty26).gobj = core::ptr::null_mut();
        }
        if !(*ty28).x00.is_null() {
            ext::HSD_GObjFree((*ty28).x00);
            (*ty28).x00 = core::ptr::null_mut();
            (*ty28).x10 = 0;
        }
        if !(*ty28).x04.is_null() {
            ext::HSD_GObjProc_RemoveAllProcs((*ty28).x04);
            ext::HSD_GObjFree((*ty28).x04);
            (*ty28).x04 = core::ptr::null_mut();
        }
        if !(*ty28).x08.is_null() {
            ext::HSD_GObjFree((*ty28).x08);
            (*ty28).x08 = core::ptr::null_mut();
            ext::HSD_FogSet(core::ptr::null_mut());
        }
        if !(*ty30).gobj2.is_null() {
            ext::HSD_GObjFree((*ty30).gobj2);
            (*ty30).gobj2 = core::ptr::null_mut();
        }
        // `ty31[0..6]` are the six gobj slots of the camera record.
        macro_rules! free_slot {
            ($slot:expr) => {
                if !$slot.is_null() {
                    ext::HSD_GObjFree($slot as *mut HSD_GObj);
                    $slot = core::ptr::null_mut();
                }
            };
        }
        free_slot!((*ty31).x0);
        free_slot!((*ty31).x4);
        free_slot!((*ty31).x8);
        free_slot!((*ty31).xC);
        free_slot!((*ty31).x10[0]);
        free_slot!((*ty31).x10[1]);
    }
}

/// Debug-window row for one category (`Rea_<c> have/total`).
pub unsafe fn _Toy_803109A0(arg0: i32, arg1: i32, arg2: i32) {
    let table = &_Toy_803B8910;
    let mut idx = 0usize;
    while idx < 9 {
        if arg0 == table[idx].0 {
            break;
        }
        idx += 1;
    }
    if arg2 != 0 {
        ext::DevText_StoreColorIndex(_Toy_sbss_804D6E98, 1);
    } else {
        ext::DevText_StoreColorIndex(_Toy_sbss_804D6E98, 0);
    }
    // `table.entries[idx].value_byte`: the low byte of the entry's value (the C union's first byte).
    let c = (table[idx.min(8)].1 as u8) as char;
    let ret = Toy_80304B94(arg0);
    let s = if arg0 == 8 {
        format!("Rea_{} {:3}/{:3}", c, arg1, ret)
    } else {
        format!("Rea_{} {:3}/{:3}\n", c, arg1, ret)
    };
    ext::DevText_Printf(_Toy_sbss_804D6E98, &s);
}

/// Debug "unlock editor" window proc.
pub unsafe extern "C" fn _Toy_80310B48(gobj: *mut HSD_GObj) {
    let mut changed = 0;
    let editor = _Toy_sbss_804D6E5C;
    let mut stick_x: f32 = 0.0;
    let mut stick_y: f32 = 0.0;
    let dir_x: f32;
    let dir_y: f32;

    for i in 0..4 {
        stick_x = (*pad(i as u8)).nml_stickX;
        let abs_val = if stick_x < 0.0 { -stick_x } else { stick_x };
        if abs_val > 0.1 {
            break;
        }
    }
    for i in 0..4 {
        stick_y = (*pad(i as u8)).nml_stickY;
        let abs_val = if stick_y < 0.0 { -stick_y } else { stick_y };
        if abs_val > 0.1 {
            ext::gm_801677E8(i as i8);
            break;
        }
    }
    if stick_x < -0.6 {
        dir_x = 1.0;
        dir_y = 0.0;
    } else if stick_x > 0.6 {
        dir_x = -1.0;
        dir_y = 0.0;
    } else if stick_y < -0.6 {
        dir_x = 0.0;
        dir_y = 1.0;
    } else if stick_y > 0.6 {
        dir_x = 0.0;
        dir_y = -1.0;
    } else {
        dir_x = 0.0;
        dir_y = 0.0;
        (*editor).repeat_delay = 0;
    }
    if (*editor).repeat_delay as i8 != 0 {
        (*editor).repeat_delay = ((*editor).repeat_delay).wrapping_sub(1);
        return;
    }
    let mut buttons = Toy_80305B88() as u32;
    if buttons & HSD_PAD_B as u32 != 0 {
        sfxBack();
        ext::HSD_GObjFree(gobj);
        (*editor).gobj = core::ptr::null_mut();
        (*toy_mode_state()).x4 = 1;
        return;
    }
    buttons = Toy_80305B88() as u32;
    if buttons & (HSD_PAD_A | HSD_PAD_START) as u32 != 0 {
        sfxForward();
        Toy_80311960();
        for slot in 0..9usize {
            if (*editor).values[slot] != 0 {
                Toy_80305918(slot as i8, 0, 0);
                buttons = Toy_80305B88() as u32;
                if buttons & HSD_PAD_START as u32 != 0 {
                    _Toy_803053C4(slot as i32, (*editor).values[slot] as i32, 0);
                } else {
                    _Toy_803053C4(slot as i32, (*editor).values[slot] as i32, 1);
                }
            } else if slot == 2 {
                Toy_80305918(slot as i8, 0, 0);
            }
        }
        Toy_80305918(0, 1, 0);
        Toy_80305918(1, 1, 0);
        Toy_80305918(8, 1, 0);
        Toy_80305918(3, 1, 0);
        ext::DevText_HideBackground(_Toy_sbss_804D6E98);
        ext::DevText_HideText(_Toy_sbss_804D6E98);
        Toy_80310324();
        ext::HSD_GObjFree(gobj);
        (*editor).gobj = core::ptr::null_mut();
        return;
    }
    if dir_x > 0.0 || (Toy_80305C44() as u32 & HSD_PAD_Y as u32) != 0 || (Toy_80305B88() as u32 & HSD_PAD_DPADLEFT as u32) != 0 {
        sfxMove();
        let s = (*editor).selected_slot as i8 as usize;
        (*editor).values[s] = ((*editor).values[s] as i32 + 1) as i16;
        if (*editor).values[s] as i32 > Toy_80304B94((*editor).selected_slot as i8 as i32) {
            let selected_slot = (*editor).selected_slot;
            (*editor).values[s] = Toy_80304B94(selected_slot as i8 as i32) as i16;
        }
        (*editor).repeat_delay = 4;
        changed = 1;
    }
    if dir_x < 0.0 || (Toy_80305C44() as u32 & HSD_PAD_X as u32) != 0 || (Toy_80305B88() as u32 & HSD_PAD_DPADRIGHT as u32) != 0 {
        sfxMove();
        let s = (*editor).selected_slot as i8 as usize;
        (*editor).values[s] = ((*editor).values[s] as i32 - 1) as i16;
        if (*editor).values[s] < 0 {
            (*editor).values[s] = 0;
        }
        (*editor).repeat_delay = 4;
        changed = 1;
    }
    if dir_x == 0.0 && dir_y != 0.0 {
        sfxMove();
        (*editor).selected_slot = (((*editor).selected_slot as i8 as f32 + dir_y) as i32) as u8;
        if ((*editor).selected_slot as i8) < 0 {
            (*editor).selected_slot = 0;
        }
        if (*editor).selected_slot as i8 > 9 {
            (*editor).selected_slot = 9;
        }
        (*editor).repeat_delay = 8;
        changed = 1;
    }
    if changed != 0 {
        ext::DevText_Erase(_Toy_sbss_804D6E98);
        ext::DevText_SetCursorXY(_Toy_sbss_804D6E98, 0, 0);
        for i in 0..9i32 {
            if i == (*editor).selected_slot as i8 as i32 {
                _Toy_803109A0(i, (*editor).values[i as usize] as i32, 1);
            } else {
                _Toy_803109A0(i, (*editor).values[i as usize] as i32, 0);
            }
        }
    }
}

/// `showDevText` (static inline).
#[inline]
unsafe fn showDevText(gobj: *mut HSD_GObj, text: *mut *mut DevText) {
    ext::DevText_Show(gobj, *text);
    ext::DevText_HideCursor(*text);
    ext::DevText_80302AC0(*text);
}

pub unsafe fn _Toy_803114E8() {
    _Toy_sbss_804D6E5C = ext::HSD_MemAlloc(core::mem::size_of::<ToyParamEditor>() as i32) as *mut ToyParamEditor;
    let data = _Toy_sbss_804D6E5C;
    _Toy_sbss_804D6E98 = ext::DevText_Create(1, 0x28, 0x28, 0xE, 9, _Toy_devtext_buf_804A2750.as_mut_ptr() as *mut i8);
    if !_Toy_sbss_804D6E98.is_null() {
        let gobj = ext::DevText_GetGObj();
        let color = _Toy_color_4080D060_0;
        showDevText(gobj, &raw mut _Toy_sbss_804D6E98);
        ext::DevText_SetBGColor(_Toy_sbss_804D6E98, color);
        ext::DevText_SetScale(_Toy_sbss_804D6E98, 12.0, 18.0);
        ext::DevText_Erase(_Toy_sbss_804D6E98);
        ext::DevText_SetCursorXY(_Toy_sbss_804D6E98, 0, 0);
        ext::DevText_StoreColorIndex(_Toy_sbss_804D6E98, 0);
        ext::DevText_SetTextColor(_Toy_sbss_804D6E98, _Toy_color_E2E2E2FF);
        ext::DevText_StoreColorIndex(_Toy_sbss_804D6E98, 1);
        ext::DevText_SetTextColor(_Toy_sbss_804D6E98, _Toy_color_FF8020FF);
        for i in 0..9 {
            if i == 0 {
                _Toy_803109A0(i, 0, 1);
            } else {
                _Toy_803109A0(i, 0, 0);
            }
        }
        ext::memzero(data as *mut c_void, core::mem::size_of::<ToyParamEditor>() as i32);
        (*data).gobj = ext::GObj_Create(0, 0, 0);
        ext::HSD_GObj_SetupProc((*data).gobj, Some(_Toy_80310B48), 0);
        ext::HSD_GObj_80390CD4((*data).gobj);
    } else {
        ext::OSReport("*** Couldn't Open Debug Window2!\n");
    }
}

/// Unlock every trophy in the save (debug).
pub unsafe fn Toy_80311680() {
    let mut var_r29 = ext::gmMainLib_GetTrophyFlags() as *mut u16;
    let temp_r31 = ext::gmMainLib_GetTrophyCategoryFlags() as *mut i16;
    Toy_80311960();
    for var_r28 in 0..TY_TROPHY_COUNT {
        if trophy_enabled_inline(var_r28) {
            let temp_r3 = ext::gmMainLib_GetTrophyCount();
            let mut temp_r5 = *temp_r3;
            temp_r5 = temp_r5.wrapping_add(1);
            *temp_r3 = temp_r5;
            *var_r29 = 1;
            let temp_r0_2 = Toy_803060BC(var_r28, 6) as i32;
            if temp_r0_2 as u32 > 1 && temp_r0_2 != 8 && temp_r0_2 != 3 {
                *var_r29 |= 0x4000;
            }
        }
        var_r29 = var_r29.add(1);
    }
    *temp_r31 = 0xF4;
    _Toy_sbss_804D6EA1 = 1;
}

pub unsafe fn _Toy_80311788() {
    _Toy_sbss_804D6E9C = ext::DevText_Create(1, 0x21C, 0x82, 0xA, 7, _Toy_devtext_buf_804A26C4.as_mut_ptr() as *mut i8);
    if !_Toy_sbss_804D6E9C.is_null() {
        let gobj = ext::DevText_GetGObj();
        let color = _Toy_color_40B0D060_1;
        showDevText(gobj, &raw mut _Toy_sbss_804D6E9C);
        ext::DevText_SetBGColor(_Toy_sbss_804D6E9C, color);
        ext::DevText_SetScale(_Toy_sbss_804D6E9C, 12.0, 18.0);
        ext::DevText_Erase(_Toy_sbss_804D6E9C);
        ext::DevText_SetCursorXY(_Toy_sbss_804D6E9C, 0, 0);
        let sel = |_: ()| *Toy_sbss_804D6EDC.offset((*Toy_sbss_804D6EE0).selectedIdx as isize) as i32;
        let f27 = Toy_803060BC(sel(()), 5);
        let f28 = Toy_803060BC(sel(()), 4);
        let f29 = Toy_803060BC(sel(()), 3);
        let f30 = Toy_803060BC(sel(()), 2);
        let f31 = Toy_803060BC(sel(()), 1);
        let s = format!(
            "X   {:3.2}\nY   {:3.2}\nZ   {:3.2}\nMS  {:3.2}\nSS  {:3.2}\nMD  {:3.2}\0",
            Toy_803060BC(sel(()), 0) as f64,
            f31 as f64,
            f30 as f64,
            f29 as f64,
            f28 as f64,
            f27 as f64
        );
        ext::DevText_Print(_Toy_sbss_804D6E9C, s.as_ptr() as *mut i8);
    } else {
        ext::OSReport("*** Couldn't Open Debug Window!\n");
    }
}

/// Clear all trophy data (save and scene array).
pub unsafe fn Toy_80311960() {
    let save_data = ext::gmMainLib_GetTrophyFlags() as *mut u16;
    let save_data2 = ext::gmMainLib_GetTrophyCategoryFlags() as *mut i16;
    for i in 0..TY_TROPHY_COUNT as isize {
        *save_data.offset(i) = 0;
        *toy_flags().offset(i) = 0;
    }
    *save_data2 = 0;
    *toy_u16(4) = 0; // TOY_X19C
    *toy_u16(3) = 0; // TOY_X19A
    *(toy_u16(0x12A) as *mut i16) = 0; // TOY_SEL
    *(toy_u16(0x12B) as *mut i16) = 0; // TOY_SELID
    *toy_byte(3) = 0; // TOY_X197
    *ext::gmMainLib_GetTrophyCount() = 0;
    *TOY_COUNT() = 0;
}

pub unsafe fn Toy_Scene_OnEnter(_arg0: *mut c_void) {
    _Toy_sbss_804D6EA2 = 0;
    _Toy_sbss_804D6E50 = 0;
    _Toy_sbss_804D6EA1 = 0;
    ext::tyList_PcSetActive(0); // a Gallery entry starts in the viewer
    if ext::DbLevel as i32 >= DbLKind_DebugRom as i32 {
        /* Check Z button */
        let mut buttons = Toy_80305C44() as u32;
        _Toy_sbss_804D6E50 = if buttons & HSD_PAD_L as u32 != 0 { 1 } else { 0 };
        /* Check Start button */
        buttons = Toy_80305C44() as u32;
        _Toy_sbss_804D6EA2 = if buttons & HSD_PAD_Z as u32 != 0 { 1 } else { 0 };
        /* Check D-pad down */
        buttons = Toy_80305C44() as u32;
        if buttons & HSD_PAD_START as u32 != 0 {
            _Toy_sbss_804D6EA0 = 0;
        }
        if _Toy_sbss_804D6EA0 as i8 != 0 {
            _Toy_sbss_804D6EA0 = _Toy_sbss_804D6EA0.wrapping_add(1);
        }
        /* Check D-pad left */
        buttons = Toy_80305C44() as u32;
        if buttons & HSD_PAD_R as u32 != 0 {
            _Toy_sbss_804D6EA0 = 1;
        }
    }
    _Toy_sbss_804D6E68 = ext::HSD_MemAlloc(core::mem::size_of::<Toy6E68>() as i32) as *mut Toy6E68;
    Toy_sbss_804D6ED8 = ext::HSD_MemAlloc(core::mem::size_of::<ToyED8Data>() as i32) as *mut ToyED8Data;
    Toy_sbss_804D6ED4 = ext::HSD_MemAlloc(core::mem::size_of::<ToyCameraControl>() as i32) as *mut ToyCameraControl;
    Toy_sbss_804D6EDC = ext::HSD_MemAlloc(core::mem::size_of::<i16>() as i32 * TY_TROPHY_COUNT) as *mut i16;
    _Toy_sbss_804D6E64 = ext::HSD_MemAlloc(core::mem::size_of::<TySortRow>() as i32 * TY_TROPHY_COUNT) as *mut TySortRow;
    Toy_sbss_804D6EE0 = ext::HSD_MemAlloc(core::mem::size_of::<TyDisplayData>() as i32) as *mut TyDisplayData;
    _Toy_sbss_804D6E6C = ext::HSD_MemAlloc(core::mem::size_of::<TyViewData>() as i32) as *mut TyViewData;
    ext::memzero(_Toy_sbss_804D6E68 as *mut c_void, core::mem::size_of::<Toy6E68>() as i32);
    ext::memzero(Toy_sbss_804D6ED8 as *mut c_void, core::mem::size_of::<ToyED8Data>() as i32);
    ext::memzero(Toy_sbss_804D6ED4 as *mut c_void, core::mem::size_of::<ToyCameraControl>() as i32);
    ext::memzero(Toy_sbss_804D6EDC as *mut c_void, core::mem::size_of::<i16>() as i32 * TY_TROPHY_COUNT);
    ext::memzero(_Toy_sbss_804D6E64 as *mut c_void, core::mem::size_of::<TySortRow>() as i32 * TY_TROPHY_COUNT);
    ext::memzero(Toy_sbss_804D6EE0 as *mut c_void, core::mem::size_of::<TyDisplayData>() as i32);
    // `memzero(_Toy_sbss_804D6E6C, 8)`: the whole 8-byte GameCube record.
    ext::memzero(_Toy_sbss_804D6E6C as *mut c_void, core::mem::size_of::<TyViewData>() as i32);
    Toy_8031263C();
    /* Validate saved selection index */
    let selp = toy_u16(0x12A) as *mut i16; // TOY_SEL
    let mut reset_selection = false;
    if *selp < 0 {
        reset_selection = true;
    } else {
        let count = trophy_count_now();
        if *selp as i32 > count {
            reset_selection = true;
        }
    }
    if reset_selection {
        *selp = 0;
    }
    *(toy_byte(1) as *mut i8) = 0; // TOY_X195
    *(toy_byte(2) as *mut i8) = 0; // TOY_X196
    /* Set up SIS font tables based on language */
    if ext::lbLang_IsSavedLanguageJP() != 0 {
        ext::HSD_SisLib_803A62A0(0, cstr!("SdToy.dat"), cstr!("SIS_ToyData"));
        ext::HSD_SisLib_803A62A0(3, cstr!("SdToyExp.dat"), cstr!("SIS_ToyDataExp"));
    } else {
        ext::HSD_SisLib_803A62A0(0, cstr!("SdToy.usd"), cstr!("SIS_ToyData_E"));
        ext::HSD_SisLib_803A62A0(3, cstr!("SdToyExp.usd"), cstr!("SIS_ToyDataExp_E"));
    }
    /* Enter debug or normal mode */
    if _Toy_sbss_804D6EA2 != 0 {
        _Toy_80311788();
    }
    if _Toy_sbss_804D6EA0 as i32 == 1 {
        _Toy_803114E8();
        return;
    }
    if ext::gmMainLib_8015ED5C() != -1 {
        ext::lbAudioAx_80023F28(ext::gmMainLib_8015ED5C());
    }
    Toy_80310324();
}

pub unsafe fn _Toy_80311F5C() {
    let p1 = Toy_sbss_804D6ED8;
    let p2 = _Toy_sbss_804D6E68;
    if !(*p1).archive.is_null() {
        (*p1).archive = core::ptr::null_mut();
    }
    if !(*p1).x0.is_null() {
        (*p1).x0 = core::ptr::null_mut();
    }
    if !(*p2).x0.is_null() {
        (*p2).x0 = core::ptr::null_mut();
    }
    if _Toy_sbss_804D6EA2 != 0 && !_Toy_sbss_804D6E9C.is_null() {
        ext::DevText_Remove(&raw mut _Toy_sbss_804D6E9C);
        _Toy_sbss_804D6E9C = core::ptr::null_mut();
    }
    if !_Toy_sbss_804D6E98.is_null() {
        ext::DevText_Remove(&raw mut _Toy_sbss_804D6E98);
        _Toy_sbss_804D6E98 = core::ptr::null_mut();
    }
    if !_Toy_sbss_804D6E5C.is_null() {
        ext::HSD_Free(_Toy_sbss_804D6E5C as *mut c_void);
        _Toy_sbss_804D6E5C = core::ptr::null_mut();
    }
}

pub unsafe fn Toy_Scene_OnFrame() {
    let state = toy_mode_state();
    if (*state).x4 != 0 {
        _Toy_80311F5C();
        ext::gm_801A4B60();
    }
}

/// Axis gizmo render callback (red/green/blue lines through the camera interest).
pub unsafe extern "C" fn _Toy_80312050(_gobj: *mut HSD_GObj, _code: i32) {
    let data = _Toy_sbss_804D6E6C;
    let cobj = ext::HSD_CObjGetCurrent();
    if (*data).x4 == 0 {
        let mut interest = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut endpoint = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut view_mtx: Mtx = core::mem::zeroed();
        let mut up = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut left = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut eye = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut scaled = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::HSD_CObjGetInterest(cobj, &mut interest);
        ext::HSD_CObjGetLeftVector(cobj, &mut left);
        ext::HSD_CObjGetUpVector(cobj, &mut up);
        ext::HSD_CObjGetEyeVector(cobj, &mut eye);
        ext::HSD_StateInitDirect(0, 2);
        ext::HSD_SetupRenderMode(2);
        ext::HSD_CObjGetViewingMtx(cobj, view_mtx.as_mut_ptr() as *mut c_void);
        ext::GXLoadPosMtxImm(view_mtx.as_mut_ptr() as *mut c_void, 0);
        ext::HSD_StateSetLineWidth(6, 5);
        ext::GXBegin(GX_LINES as u32, GX_VTXFMT0 as u32, 6);
        let color_ff: u8 = 0xFF;
        let color_00: u8 = 0;
        let mut axis = |v: &mut Vec3, s: f32, r: u8, g: u8, b: u8| {
            ext::PSVECScale(v, &mut scaled, s);
            ext::PSVECAdd(&mut scaled, &mut interest, &mut endpoint);
            ext::GXPosition3f32(endpoint.x, endpoint.y, endpoint.z);
            ext::GXColor4u8(r, g, b, color_ff);
        };
        axis(&mut left, -3.25, color_ff, color_00, color_00);
        axis(&mut left, 3.25, color_ff, color_00, color_00);
        axis(&mut up, -3.25, color_00, color_ff, color_00);
        axis(&mut up, 3.25, color_00, color_ff, color_00);
        axis(&mut eye, -3.25, color_00, color_00, color_ff);
        axis(&mut eye, 3.25, color_00, color_00, color_ff);
    }
}

pub unsafe fn Toy_Mode_OnInit() {
    ext::memzero(toy_byte(0) as *mut c_void, 0x25A); // TOY_X194
    _Toy_sbss_804D6EA1 = 0;
    let target_ptr: *mut u16 = if toy_scene_mode() { toy_u16(3) } else { ext::gmMainLib_GetTrophyCategoryFlags() as *mut u16 };
    *target_ptr |= 4;
    *toy_byte(0) = 1;
}

/// Copy trophy data between the save (`arg0 == 0` to the scene array) and the scene array.
pub unsafe fn Toy_8031234C(arg0: i32) {
    let save_data = ext::gmMainLib_GetTrophyFlags() as *mut u16;
    let state_data = ext::gmMainLib_GetTrophyCategoryFlags() as *mut u16;
    if arg0 != 0 {
        let mut dst_ptr = save_data;
        let mut src_ptr = toy_u16(0);
        let mut i = TY_TROPHY_COUNT;
        while i != 0 {
            let flags = *src_ptr.add(5);
            if flags & 0x8000 != 0 {
                *dst_ptr |= 0x8000;
            }
            *dst_ptr = ((*src_ptr.add(5) & 0xFF) as u32 + (*dst_ptr & 0xFF00) as u32) as u16;
            src_ptr = src_ptr.add(1);
            dst_ptr = dst_ptr.add(1);
            i -= 1;
        }
        *state_data = *toy_u16(3); // TOY_X19A
        for category in 0..9i32 {
            if category as u32 > 1 && category != 8 && category != 3 && (*state_data as i32 & (1 << category)) != 0 {
                let mut ptr = save_data;
                for j in 0..TY_TROPHY_COUNT {
                    let result = Toy_803060BC(j, 6);
                    if category as f32 == result {
                        *ptr |= 0x4000;
                    }
                    ptr = ptr.add(1);
                }
            }
        }
        *ext::gmMainLib_GetTrophyCount() = *TOY_COUNT();
    } else {
        *toy_u16(3) = *state_data;
        *toy_u16(4) = 0;
        core::ptr::copy_nonoverlapping(save_data as *const u8, toy_flags() as *mut u8, 0x24A);
        *TOY_COUNT() = *ext::gmMainLib_GetTrophyCount();
    }
}

/// `loadTrophyMetadata` (static inline).
#[inline]
unsafe fn loadTrophyMetadata() {
    if _Toy_sbss_804D6ED0.is_null() {
        _Toy_sbss_804D6ED0 = ext::lbArchive_LoadSymbols(
            getDataiDatFilename(),
            &[
                (&raw mut _Toy_sbss_804D6EC4) as *mut c_void, cstr!("tyInitModelTbl") as *mut c_void,
                (&raw mut _Toy_sbss_804D6EC0) as *mut c_void, cstr!("tyInitModelDTbl") as *mut c_void,
                (&raw mut _Toy_sbss_804D6EBC) as *mut c_void, cstr!("tyModelSortTbl") as *mut c_void,
                (&raw mut _Toy_sbss_804D6EB8) as *mut c_void, cstr!("tyExpDifferentTbl") as *mut c_void,
                (&raw mut _Toy_sbss_804D6EB4) as *mut c_void, cstr!("tyNoGetUsTbl") as *mut c_void,
                (&raw mut Toy_sbss_804D6EB0) as *mut c_void, cstr!("tyDisplayModelTbl") as *mut c_void,
                (&raw mut Toy_sbss_804D6EAC) as *mut c_void, cstr!("tyDisplayModelUsTbl") as *mut c_void,
            ],
        );
    }
}

pub unsafe fn Toy_803124BC() {
    let mut table1 = ext::gmMainLib_GetTrophyFlags() as *mut u16;
    let table2 = ext::gmMainLib_GetTrophyCategoryFlags() as *mut u16;
    loadTrophyMetadata();
    for i in 0..TY_TROPHY_COUNT {
        let skip = trophy_enabled_inline(i);
        if skip && Toy_803060BC(i, 6) as i32 == 2 {
            *table1 |= 0x4000;
        }
        table1 = table1.add(1);
    }
    *table2 |= 4;
    *toy_u16(3) |= 4;
}

pub unsafe fn Toy_8031263C() {
    (*toy_mode_state()).x4 = 0;
    let mut table1 = ext::gmMainLib_GetTrophyFlags() as *mut u16;
    let table2 = ext::gmMainLib_GetTrophyCategoryFlags() as *mut u16;
    loadTrophyMetadata();
    for i in 0..TY_TROPHY_COUNT {
        if _Toy_80304CC8_noinline(i) && Toy_803060BC(i, 6) as i32 == 2 {
            *table1 |= 0x4000;
        }
        table1 = table1.add(1);
    }
    *table2 |= 4;
    *toy_u16(3) |= 4;
    Toy_803102D0();
    Toy_8031234C(0);
}

pub unsafe fn Toy_803127D4() {
    _Toy_sbss_804D6ED0 = core::ptr::null_mut();
    _Toy_sbss_804D6EC4 = core::ptr::null_mut();
    _Toy_sbss_804D6EC0 = core::ptr::null_mut();
    _Toy_sbss_804D6EBC = core::ptr::null_mut();
    _Toy_sbss_804D6EB8 = core::ptr::null_mut();
    _Toy_sbss_804D6EB4 = core::ptr::null_mut();
    Toy_sbss_804D6EB0 = core::ptr::null_mut();
    Toy_sbss_804D6EAC = core::ptr::null_mut();
    _Toy_sbss_804D6ECC = core::ptr::null_mut();
    _Toy_sbss_804D6EA8 = core::ptr::null_mut();
    _Toy_sbss_804D6EA4 = core::ptr::null_mut();
    Toy_sbss_804D6EC8 = core::ptr::null_mut();
    ext::memzero(&raw mut Toy_804A2AA8 as *mut c_void, core::mem::size_of::<ToyAnimState>() as i32);
}

/// Atlas step 10 readbacks for the framed scenes: 0 the trophy count, 1 the selected list index,
/// 2 the selected trophy's id, 3 the viewer's state (0 shelf, 1 and 3 moving, 2 examining),
/// 4 retail's exit request, 5 the full-screen trophy list is up. -1 when the state is not there.
pub unsafe fn Toy_PcReadback(what: i32) -> i32 {
    let display = Toy_sbss_804D6EE0;
    let mode = _Toy_sbss_804D6E68;
    match what {
        0 => Toy_GetTrophyTotal(),
        1 => {
            if !display.is_null() {
                (*display).selectedIdx as i32
            } else {
                -1
            }
        }
        2 => {
            if !display.is_null() && !(*display).selected_entry.is_null() {
                (*(*display).selected_entry).trophy_id as i32
            } else {
                -1
            }
        }
        3 => {
            if !mode.is_null() {
                (*mode).x61 as i32
            } else {
                -1
            }
        }
        4 => (*toy_mode_state()).x4 as i32,
        5 => ext::tyList_PcActive(),
        _ => -1,
    }
}
