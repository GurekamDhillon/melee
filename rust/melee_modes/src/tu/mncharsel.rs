//! Port of src/melee/mn/mncharsel.c (TARGET_PC build): the character-select screen. The icon /
//! door / tag tables, the HUD text of the mode, the per-door costume / team / handicap display,
//! the cursor logic, the CSS scene (enter / frame / exit) and the PC-only m-ex CSS support (the
//! `mexSelectChr` icon table, the portrait frame lookup, the `Mex_*` metadata) plus the PC art
//! readers (`mnCharSel_PcArt*`) that the results and frontend screens call.
//!
//! State is the file's statics (`mnCharSel_804D6CB0` ... , the icon / door / tag / `data2` tables,
//! all declared in `globals!` below). Callees of unported TUs are `ext` TODO stubs named like the C
//! function; variadic callees (`HSD_SisLib_803A70A0`, `HSD_ForeachAnim`, `lb_80011E24`, `OSReport`)
//! take their varargs as a `Va` slice.
//!
//! The fused multiply-add sites follow `tools/rust-port/fma_sites.py src/melee/mn/mncharsel.c`
//! (all f32 -> `fmadds`): 1 in `mnCharSel_8025D1C4`, 4 in `fn_8025F0E0`, 6 in `mnCharSel_CursorThink`,
//! 1 in `getStickDelta`, 2 in `updateGrabbedSlider`, 8 in `fn_80262648`, 4 in `fn_802633B0`, 1 in
//! `mnCharSel_802640A0`, 2 in `mnCharSel_MenuTexSetup`.
//!
//! `bool` is a 32-bit int in this build; the file's `static bool` flags are `i32`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_unsafe, unused_variables, unused_assignments, unused_mut, dead_code, clippy::missing_safety_doc, clippy::too_many_arguments)]

use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null, null_mut};
use std::vec::Vec;

use melee_core::fp::fmadds;
use melee_runtime::cabi::{f2i32, sdiv, srem};
use melee_runtime::gekko::{cosf, sinf};
use melee_runtime::lbtrigf::{atan2f, atanf};
use melee_types::*;

use super::hsd::{get_jobj, jobj_child, jobj_next, jobj_parent, sfxBack, sfxForward, sfxMove};
use crate::oracle::Va;

pub use crate::mn::{equalU8, getIconOffset, toFeet, toMeters};
pub use crate::mn_ptr::{loadStickValue, mnCharSel_MexPackRows};

pub mod ext {
    #![allow(unused_imports)]
    use super::*;
    crate::oext_fns! {
        fn Frontend_CaptureCharIcon(a0: i32, a1: *mut HSD_JObj) -> () = crate::tu::adapt::Frontend_CaptureCharIcon__0cb8dd;
        fn Frontend_CaptureIcon(a0: i32, a1: *mut HSD_JObj) -> () = crate::tu::adapt::Frontend_CaptureIcon__520054;
        fn Frontend_OnlinePick() -> i32 = crate::tu::adapt::Frontend_OnlinePick__100e6e;
        fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
        fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void) -> ();
        fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) -> ();
        fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32) -> ();
        fn GXLoadPosMtxImm(a0: *mut c_void, a1: u32) -> ();
        fn GXSetChanCtrl(a0: u32, a1: u8, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32) -> ();
        fn GXSetChanMatColor(a0: u32, a1: crate::GXColor) -> ();
        fn GXSetCurrentMtx(a0: u32) -> ();
        fn GXSetNumChans(a0: u8) -> ();
        fn GXSetProjection(a0: *mut c_void, a1: u32) -> ();
        fn GXSetScissor(a0: u32, a1: u32, a2: u32, a3: u32) -> ();
        fn GXSetTexCoordGen2(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32) -> ();
        fn GXSetViewport(a0: f32, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32) -> ();
        fn Geno_DefineArtBaseCK(a0: i32) -> i32;
        fn Geno_DefineBaseCK(a0: i32) -> i32;
        fn GetNameText(a0: i32) -> *mut i8 = crate::tu::adapt::GetNameText__a3637e;
        fn GxTex_Close(a0: i32) -> ();
        fn GxTex_CopyImage(a0: i32, a1: *mut c_void) -> ();
        fn GxTex_CopyTlut(a0: i32, a1: *mut c_void) -> ();
        fn GxTex_Format(a0: i32) -> i32;
        fn GxTex_Height(a0: i32) -> i32;
        fn GxTex_ImageSize(a0: i32) -> i32;
        fn GxTex_OpenEnv() -> i32;
        fn GxTex_TlutEntries(a0: i32) -> i32;
        fn GxTex_TlutFormat(a0: i32) -> i32;
        fn GxTex_TlutSize(a0: i32) -> i32;
        fn GxTex_Width(a0: i32) -> i32;
        fn HSD_AObjReqAnim(a0: *mut HSD_AObj, a1: f32) -> () = crate::tu::adapt::HSD_AObjReqAnim__f16456;
        fn HSD_AObjStopAnim(a0: *mut HSD_AObj, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut HSD_ObjData)>) -> ();
        fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void = crate::tu::adapt::HSD_ArchiveGetPublicAddress__1a942f;
        fn HSD_CObjLoadDesc(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj = crate::tu::adapt::HSD_CObjLoadDesc__66a547;
        fn HSD_DObjAddAnimAll(a0: *mut HSD_DObj, a1: *mut _HSD_MatAnim, a2: *mut HSD_ShapeAnimDObj) -> ();
        fn HSD_FogLoadDesc(a0: *mut HSD_FogDesc) -> *mut HSD_Fog;
        fn HSD_FogSet(a0: *mut HSD_Fog) -> ();
        fn HSD_Free(a0: *mut c_void) -> () = crate::tu::adapt::HSD_Free__3ee311;
        fn HSD_GObjGXLink_803909D8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> ();
        fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) -> ();
        fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) -> ();
        fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) -> () = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) -> () = crate::tu::adapt::HSD_GObj_LObjCallback__aaa236;
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint) -> ();
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj) -> ();
        fn HSD_JObjClearFlags(a0: *mut HSD_JObj, a1: u32) -> ();
        fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) -> ();
        fn HSD_JObjGetDObj(a0: *mut HSD_JObj) -> *mut HSD_DObj;
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_JObjRemoveAll(a0: *mut HSD_JObj) -> ();
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32) -> ();
        fn HSD_JObjSetFlags(a0: *mut HSD_JObj, a1: u32) -> ();
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) -> ();
        fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) -> ();
        fn HSD_LObjLoadDesc(a0: *mut HSD_LightDesc) -> *mut HSD_LObj;
        fn HSD_MObjSetAlpha(a0: *mut HSD_MObj, a1: f32) -> ();
        fn HSD_MemAlloc(a0: i32) -> *mut c_void = crate::tu::adapt::HSD_MemAlloc__c2b3bc;
        fn HSD_Randf() -> f32;
        fn HSD_Randi(a0: i32) -> i32;
        fn HSD_SObjLib_803A477C(a0: *mut HSD_GObj, a1: *mut HSD_SObjDesc, a2: u32, a3: u32, a4: u8, a5: u8) -> *mut HSD_SObj = crate::tu::adapt::HSD_SObjLib_803A477C__a0c4ae;
        fn HSD_SObjLib_803A49E0(a0: *mut HSD_GObj, a1: i32) -> () = crate::tu::adapt::HSD_SObjLib_803A49E0__00f645;
        fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A5ACC__27fcb1;
        fn HSD_SisLib_803A5E70() -> () = crate::tu::adapt::HSD_SisLib_803A5E70__5f9c7d;
        fn HSD_SisLib_803A5FBC() -> () = crate::tu::adapt::HSD_SisLib_803A5FBC__b031ed;
        fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 = crate::tu::adapt::HSD_SisLib_803A611C__de78e8;
        fn HSD_SisLib_803A62A0(a0: i32, a1: *mut i8, a2: *mut i8) -> () = crate::tu::adapt::HSD_SisLib_803A62A0__acd015;
        fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) -> () = crate::tu::adapt::HSD_SisLib_803A6368__2e8f0d;
        fn HSD_SisLib_803A6478(a0: *mut u8, a1: *mut u8) -> *mut u8 = crate::tu::adapt::HSD_SisLib_803A6478__1fa60b;
        fn HSD_SisLib_803A6530(a0: i32, a1: i32, a2: i32) -> *mut u8 = crate::tu::adapt::HSD_SisLib_803A6530__05d28b;
        fn HSD_SisLib_803A660C(a0: i32, a1: i32, a2: i32) -> () = crate::tu::adapt::HSD_SisLib_803A660C__64b186;
        fn HSD_SisLib_803A6754(a0: i32, a1: i32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A6754__02fb41;
        fn HSD_SisLib_803A746C(a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32) -> () = crate::tu::adapt::HSD_SisLib_803A746C__8945a6;
        fn HSD_SisLib_803A74F0(a0: *mut HSD_Text, a1: i32, a2: *mut _GXColor) -> () = crate::tu::adapt::HSD_SisLib_803A74F0__610e18;
        fn IsNameListFull() -> i32 = crate::tu::adapt::IsNameListFull__2029e6;
        fn MTXOrtho(a0: *mut c_void, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> () = crate::tu::adapt::MTXOrtho__214174;
        fn Mex_CssCursorScaleEnabled() -> i32;
        fn Mex_CssIconCount() -> i32;
        fn Mex_CssIconTable() -> *mut c_void;
        fn Mex_CssTraceEnabled() -> i32;
        fn Mex_Enabled(a0: *const i8) -> i32;
        fn Mex_ExtToPortCKind(a0: i32) -> i32;
        fn Mex_InsigniaForExt(a0: i32) -> i32;
        fn Mex_MenuParamF(a0: i32) -> f32;
        fn Mex_PortCKindToExt(a0: i32) -> i32;
        fn Player_800325C8(a0: u32, a1: i32) -> i8 = crate::tu::adapt::Player_800325C8__4ace7f;
        fn atan2f(a0: f32, a1: f32) -> f32 = crate::tu::adapt::atan2f__2f62eb;
        fn atanf(a0: f32) -> f32 = crate::tu::adapt::atanf__3acb52;
        fn cosf(a0: f32) -> f32 = crate::tu::adapt::cosf__b177e0;
        fn gmMainLib_8015D0D8(a0: u8) -> i32 = crate::tu::adapt::gmMainLib_8015D0D8__be2386;
        fn gmMainLib_8015D194(a0: u8) -> *mut u8 = crate::tu::adapt::gmMainLib_8015D194__9e9b6e;
        fn gmMainLib_8015D200(a0: u8) -> i32 = crate::tu::adapt::gmMainLib_8015D200__713e80;
        fn gmMainLib_8015D2BC(a0: u8) -> *mut u8 = crate::tu::adapt::gmMainLib_8015D2BC__88a357;
        fn gmMainLib_8015D328(a0: u8) -> i32 = crate::tu::adapt::gmMainLib_8015D328__645c26;
        fn gmMainLib_8015D3E4(a0: u8) -> *mut u8 = crate::tu::adapt::gmMainLib_8015D3E4__d26187;
        fn gmMainLib_8015ECB0() -> i32 = crate::tu::adapt::gmMainLib_8015ECB0__963013;
        fn gmMainLib_GetGamePrefs() -> *mut GamePrefs = crate::tu::adapt::gmMainLib_GetGamePrefs__12f16c;
        fn gmMainLib_GetGameRules() -> *mut GameRules = crate::tu::adapt::gmMainLib_GetGameRules__34f6cb;
        fn gm_80160980(a0: u8) -> *const i8 = crate::tu::adapt::gm_80160980__e502e7;
        fn gm_80162BD8(a0: u8) -> i32 = crate::tu::adapt::gm_80162BD8__6ed8f2;
        fn gm_80162C48() -> i32 = crate::tu::adapt::gm_80162C48__9ae6cd;
        fn gm_80162D1C() -> i32 = crate::tu::adapt::gm_80162D1C__47f299;
        fn gm_80162D6C() -> u8 = crate::tu::adapt::gm_80162D6C__172711;
        fn gm_80162DD4(a0: u8) -> i32 = crate::tu::adapt::gm_80162DD4__1c125a;
        fn gm_80162E44() -> i32 = crate::tu::adapt::gm_80162E44__1147bf;
        fn gm_80162F18() -> i32 = crate::tu::adapt::gm_80162F18__c03e80;
        fn gm_80162F68() -> u8 = crate::tu::adapt::gm_80162F68__4e093d;
        fn gm_80162FD0(a0: u8) -> i32 = crate::tu::adapt::gm_80162FD0__bab432;
        fn gm_80163040() -> i32 = crate::tu::adapt::gm_80163040__1eec31;
        fn gm_80163114() -> i32 = crate::tu::adapt::gm_80163114__6184d4;
        fn gm_80163164() -> u8 = crate::tu::adapt::gm_80163164__f46f0d;
        fn gm_801631CC(a0: u8) -> u32 = crate::tu::adapt::gm_801631CC__ca58c4;
        fn gm_801631F0() -> u32 = crate::tu::adapt::gm_801631F0__9c2074;
        fn gm_80163274(a0: u8) -> u16 = crate::tu::adapt::gm_80163274__6b2c4a;
        fn gm_8016332C(a0: u8) -> i32 = crate::tu::adapt::gm_8016332C__1fdd1c;
        fn gm_80163374(a0: u8, a1: *mut u8, a2: *mut u8, a3: *mut u8, a4: *mut u8) -> () = crate::tu::adapt::gm_80163374__16b21a;
        fn gm_801634D4(a0: *mut u8, a1: *mut u8, a2: *mut u8, a3: *mut u8) -> i32 = crate::tu::adapt::gm_801634D4__1f064c;
        fn gm_8016365C(a0: u8) -> i32 = crate::tu::adapt::gm_8016365C__ec3f11;
        fn gm_80163690(a0: u8) -> i32 = crate::tu::adapt::gm_80163690__c14dc4;
        fn gm_801636D8(a0: u8, a1: *mut u8, a2: *mut u8, a3: *mut u8, a4: *mut u8) -> () = crate::tu::adapt::gm_801636D8__faafb1;
        fn gm_80163838(a0: *mut u8, a1: *mut u8, a2: *mut u8, a3: *mut u8) -> i32 = crate::tu::adapt::gm_80163838__97f44a;
        fn gm_801639C0(a0: u8) -> i32 = crate::tu::adapt::gm_801639C0__cad04b;
        fn gm_801639F4(a0: u8) -> i32 = crate::tu::adapt::gm_801639F4__890fba;
        fn gm_80163A3C(a0: u8, a1: *mut u8, a2: *mut u8, a3: *mut u8, a4: *mut u8) -> () = crate::tu::adapt::gm_80163A3C__deadfc;
        fn gm_80163B9C(a0: *mut u8, a1: *mut u8, a2: *mut u8, a3: *mut u8) -> i32 = crate::tu::adapt::gm_80163B9C__06e310;
        fn gm_801685D4(a0: u8, a1: u8) -> i8 = crate::tu::adapt::gm_801685D4__a2b2f9;
        fn gm_80168C5C(a0: u32) -> () = crate::tu::adapt::gm_80168C5C__554ef2;
        fn gm_80169264(a0: u8) -> u8 = crate::tu::adapt::gm_80169264__5fece7;
        fn gm_80169290(a0: u8) -> u8 = crate::tu::adapt::gm_80169290__fa9c3b;
        fn gm_801692BC(a0: u8) -> u8 = crate::tu::adapt::gm_801692BC__f7a9a4;
        fn gm_801A4B60() -> () = crate::tu::adapt::gm_801A4B60__d469d9;
        fn gm_ChangeGameModeAfterCurrentScene(a0: i32) -> () = crate::tu::adapt::gm_ChangeGameModeAfterCurrentScene__2292c1;
        fn gm_Get15MinMultimanHighscore(a0: u8) -> u16 = crate::tu::adapt::gm_Get15MinMultimanHighscore__923dcd;
        fn gm_Get15MinMultimanTotalHighscore() -> i32 = crate::tu::adapt::gm_Get15MinMultimanTotalHighscore__174f15;
        fn gm_Get3MinMultimanHighscore(a0: u8) -> u16 = crate::tu::adapt::gm_Get3MinMultimanHighscore__9e8675;
        fn gm_Get3MinMultimanTotalHighscore() -> i32 = crate::tu::adapt::gm_Get3MinMultimanTotalHighscore__906f85;
        fn gm_GetCruelHighscore(a0: u8) -> i32 = crate::tu::adapt::gm_GetCruelHighscore__d78ce6;
        fn gm_GetCruelTotalHighscore() -> i32 = crate::tu::adapt::gm_GetCruelTotalHighscore__ef30e1;
        fn gm_GetEndlessHighscore(a0: u8) -> i32 = crate::tu::adapt::gm_GetEndlessHighscore__1fa523;
        fn gm_GetEndlessTotalHighscore() -> i32 = crate::tu::adapt::gm_GetEndlessTotalHighscore__6d2632;
        fn gm_GetNumCostumesForCKind(a0: u8) -> u8 = crate::tu::adapt::gm_GetNumCostumesForCKind__a594a2;
        fn gm_IsCKindUnlocked(a0: u8) -> i32 = crate::tu::adapt::gm_IsCKindUnlocked__b6c64b;
        fn gm_MexStockFrame(a0: i32, a1: i32) -> f32 = crate::tu::adapt::gm_MexStockFrame__77cf16;
        fn gm_MexStockMatAnim() -> *mut HSD_MatAnimJoint = crate::tu::adapt::gm_MexStockMatAnim__1be326;
        fn gm_RumbleEnabledForPlayer(a0: i32, a1: i32) -> i32 = crate::tu::adapt::gm_RumbleEnabledForPlayer__aa5085;
        fn lbArchive_80016EFC(a0: *mut HSD_Archive) -> () = crate::tu::adapt::lbArchive_80016EFC__e6851b;
        fn lbArchive_LoadArchive(a0: *const i8) -> *mut HSD_Archive = crate::tu::adapt::lbArchive_LoadArchive__471a59;
        fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 = crate::tu::adapt::lbAudioAx_800237A8__95b1fa;
        fn lbAudioAx_80023870(a0: i32, a1: i32, a2: i32, a3: i32) -> i32 = crate::tu::adapt::lbAudioAx_80023870__b1a847;
        fn lbAudioAx_80023F28(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_80023F28__582fc2;
        fn lbAudioAx_80024030(a0: i32) -> () = crate::tu::adapt::lbAudioAx_80024030__36458a;
        fn lbAudioAx_8002411C(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_8002411C__198044;
        fn lbAudioAx_80026E84(a0: u32) -> u64 = crate::tu::adapt::lbAudioAx_80026E84__5dcc27;
        fn lbAudioAx_80026F2C(a0: u32) -> () = crate::tu::adapt::lbAudioAx_80026F2C__c80d0b;
        fn lbAudioAx_8002702C(a0: u32, a1: u64) -> () = crate::tu::adapt::lbAudioAx_8002702C__208dc3;
        fn lbAudioAx_80027168() -> () = crate::tu::adapt::lbAudioAx_80027168__342616;
        fn lbAudioAx_80027648() -> () = crate::tu::adapt::lbAudioAx_80027648__4f40c8;
        fn lbCardGame_LoadArchive(a0: i32) -> () = crate::tu::adapt::lbCardGame_LoadArchive__e9acb0;
        fn lbCardNew_AllocWorkArea() -> () = crate::tu::adapt::lbCardNew_AllocWorkArea__a2c203;
        fn lbDvd_8001819C(a0: *const i8) -> *mut HSD_Archive = crate::tu::adapt::lbDvd_8001819C__ea477a;
        fn lbDvd_80018254() -> () = crate::tu::adapt::lbDvd_80018254__6be579;
        fn lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState = crate::tu::adapt::lbDvd_GetPreloadCacheScene__1ed990;
        fn lbLang_IsSavedLanguageJP() -> i32 = crate::tu::adapt::lbLang_IsSavedLanguageJP__e2b095;
        fn lbLang_IsSavedLanguageUS() -> i32 = crate::tu::adapt::lbLang_IsSavedLanguageUS__90e41e;
        fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec3, a2: *mut Vec3) -> () = crate::tu::adapt::lb_8000B1CC__69851d;
        fn lb_80014574(a0: u8, a1: i32, a2: i32, a3: i32) -> () = crate::tu::adapt::lb_80014574__7435d0;
        fn lb_800145F4() -> () = crate::tu::adapt::lb_800145F4__895731;
        fn mnNameNew_EnterFromMnCharSel(a0: *mut HSD_Archive, a1: i32) -> () = crate::tu::adapt::mnNameNew_EnterFromMnCharSel__8b7caf;
        fn mn_8022BA1C(a0: *mut HSD_GObj) -> () = crate::tu::adapt::mn_8022BA1C__3b39ba;
        fn mn_8022F138(a0: u16, a1: u16) -> () = crate::tu::adapt::mn_8022F138__462f12;
        fn mn_8022F218() -> i32 = crate::tu::adapt::mn_8022F218__a1ac6a;
        fn mn_8022F268() -> () = crate::tu::adapt::mn_8022F268__186f85;
        fn mn_80231804(a0: *mut HSD_Archive, a1: i32) -> () = crate::tu::adapt::mn_80231804__20ee22;
        fn sinf(a0: f32) -> f32 = crate::tu::adapt::sinf__245589;
    }
    crate::ext_va! {
        /// hsd_3A64.c `HSD_SisLib_803A70A0(text, kind, fmt, ...)`: set a text's string (printf-style).
        fn HSD_SisLib_803A70A0(text: *mut HSD_Text, kind: i32, fmt: *const i8) va 2 -> i32;
        /// hsd_3A64.c `HSD_SisLib_803A6B98(text, x, y, fmt, ...)`: append a string to a text (printf-style).
        fn HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const i8) va 3 -> i32;
        /// debug `OSReport(fmt, ...)`.
        fn OSReport(fmt: *const i8) va 0 -> ();
    }
    crate::ext_vk! {
        /// aobj.c `HSD_ForeachAnim(obj, type, mask, func, arg_type, ...)`: one double, or two ints.
        fn HSD_ForeachAnim(obj: *mut c_void, ty: u32, mask: u32, func: *mut c_void, arg_type: u32) vk "d|ii" -> ();
        /// lb_00B0.c `lb_80011E24(jobj, out, id, -1)`.
        fn lb_80011E24(jobj: *mut HSD_JObj, out: *mut *mut HSD_JObj) vk "ii" -> i32;
    }
}


// ------------------------------------------------------------------------------------------
// the file's own records
// ------------------------------------------------------------------------------------------

/// `MnSelectChrModels` (mncharsel.c).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MnSelectChrModels {
    pub background: StaticModelDesc,
    pub hand: StaticModelDesc,
    pub token: StaticModelDesc,
    pub menu: StaticModelDesc,
    pub press_start: StaticModelDesc,
    pub debug_camera: StaticModelDesc,
    pub regend_menu: StaticModelDesc,
    pub regend_options: StaticModelDesc,
    pub door: StaticModelDesc,
}

melee_core::gc_layout!(MnSelectChrModels, "MnSelectChrModels", size 0x90, align 4, [
    background @ 0x0: 0x10 "StaticModelDesc",
    hand @ 0x10: 0x10 "StaticModelDesc",
    token @ 0x20: 0x10 "StaticModelDesc",
    menu @ 0x30: 0x10 "StaticModelDesc",
    press_start @ 0x40: 0x10 "StaticModelDesc",
    debug_camera @ 0x50: 0x10 "StaticModelDesc",
    regend_menu @ 0x60: 0x10 "StaticModelDesc",
    regend_options @ 0x70: 0x10 "StaticModelDesc",
    door @ 0x80: 0x10 "StaticModelDesc",
]);

/// `MnSelectChrDataTable` (mncharsel.c).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MnSelectChrDataTable {
    pub cam: *mut HSD_CObjDesc,
    pub light0: *mut HSD_LightDesc,
    pub light1: *mut HSD_LightDesc,
    pub fog: *mut HSD_FogDesc,
    pub models: MnSelectChrModels,
}

melee_core::gc_layout!(MnSelectChrDataTable, "MnSelectChrDataTable", size 0xA0, align 4, [
    cam @ 0x0: 4 "HSD_CObjDesc*",
    light0 @ 0x4: 4 "HSD_LightDesc*",
    light1 @ 0x8: 4 "HSD_LightDesc*",
    fog @ 0xC: 4 "HSD_FogDesc*",
    models @ 0x10: 0x90 "MnSelectChrModels",
]);

/// `MexSelectChr` (mncharsel.c, TARGET_PC): m-ex's CSS descriptor, public symbol `mexSelectChr` of an
/// m-ex MnSlChr archive.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MexSelectChr {
    pub icon_joint: *mut HSD_Joint,
    pub icon_animjoint: *mut HSD_AnimJoint,
    pub icon_matanim: *mut HSD_MatAnimJoint,
    pub csp_matanim: *mut HSD_MatAnim,
    pub csp_stride: i32,
}

melee_core::gc_layout!(MexSelectChr, "MexSelectChr", size 0x14, align 4, [
    icon_joint @ 0x0: 4 "HSD_Joint*",
    icon_animjoint @ 0x4: 4 "HSD_AnimJoint*",
    icon_matanim @ 0x8: 4 "HSD_MatAnimJoint*",
    csp_matanim @ 0xC: 4 "HSD_MatAnim*",
    csp_stride @ 0x10: 4 "int",
]);

/// `struct CSSCharModel` (mncharsel.c).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CSSCharModel {
    pub gobj: *mut HSD_GObj,
    pub x4: u8,
    pub x5: u8,
    pub x6: u8,
    pub x7: u8,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub x14: f32,
}

melee_core::gc_layout!(CSSCharModel, "CSSCharModel", size 0x18, align 4, [
    gobj @ 0x0: 4 "HSD_GObj*",
    x4 @ 0x4: 1 "u8",
    x5 @ 0x5: 1 "u8",
    x6 @ 0x6: 1 "u8",
    x7 @ 0x7: 1 "u8",
    x8 @ 0x8: 4 "float",
    xC @ 0xC: 4 "float",
    x10 @ 0x10: 4 "float",
    x14 @ 0x14: 4 "float",
]);

/// The icons[] capacity (`CSS_ICON_MAX`, pc/platform/gw.h `GW_MEX_CSS_ICON_MAX`).
pub const CSS_ICON_MAX: usize = 128;
const SELKIND_COUNT_U8: u8 = SELKIND_COUNT as u8;

// Icon-row geometry (`ICONROWHT_*`, `ICONROWY_*`, `ICONBNDS_*`).
const ICONROWHT_TOP_TOP: f32 = 20.0;
const ICONROWHT_MID_TOP: f32 = 13.0;
const ICONROWHT_BTM_TOP: f32 = 6.0;
const ICONROWHT_BTM_BTM: f32 = -1.0;
const ICONROWY_TOP: f32 = 20.2;
const ICONROWY_BTM: f32 = 5.8;
const ICONBNDS_COL0_L: f32 = -30.0;
const ICONBNDS_COL1_L: f32 = -24.4;
const ICONBNDS_COL2_L: f32 = -17.4;
const ICONBNDS_COL3_L: f32 = -10.4;
const ICONBNDS_COL4_L: f32 = -3.4;
const ICONBNDS_COL5_L: f32 = 3.6;
const ICONBNDS_COL6_L: f32 = 10.6;
const ICONBNDS_COL7_L: f32 = 17.6;
const ICONBNDS_COL8_L: f32 = 24.4;
const ICONBNDS_COL8_R: f32 = 30.2;

/// One `CSSIcon` row: `{ hud, kind, state, 0, joint, joint, sfx, left, right, top, bottom }`.
const fn icon(hud: CSSIconHud, kind: CharacterKind, state: CSSIconState, joint: CSSIconJointId, sfx: i32, l: f32, r: f32, u: f32, d: f32) -> CSSIcon {
    CSSIcon {
        ft_hudindex: hud as u8,
        char_kind: kind as u8,
        state: state as u8,
        anim_timer: 0,
        joint_id_vs: joint as u8,
        joint_id_1p: joint as u8,
        sfx,
        bound_l: l,
        bound_r: r,
        bound_u: u,
        bound_d: d,
    }
}

const ICON_NONE: CSSIcon = CSSIcon { ft_hudindex: 0, char_kind: 0, state: 0, anim_timer: 0, joint_id_vs: 0, joint_id_1p: 0, sfx: 0, bound_l: 0.0, bound_r: 0.0, bound_u: 0.0, bound_d: 0.0 };

const fn icons_init() -> [CSSIcon; CSS_ICON_MAX + 1] {
    let mut t = [ICON_NONE; CSS_ICON_MAX + 1];
    let (tt, mt, bt) = (ICONROWHT_TOP_TOP, ICONROWHT_MID_TOP, ICONROWHT_BTM_TOP);
    let bb = ICONROWHT_BTM_BTM;
    // Top row
    t[0] = icon(ICONHUD_DRMARIO, CKind_DrMario, ICONSTATE_UNLOCKED, ICONJOINT_DRMARIO, 0xC5, ICONBNDS_COL0_L, ICONBNDS_COL1_L, tt, mt);
    t[1] = icon(ICONHUD_MARIO, CKind_Mario, ICONSTATE_TEMP, ICONJOINT_MARIO, 0xCD, ICONBNDS_COL1_L, ICONBNDS_COL2_L, tt, mt);
    t[2] = icon(ICONHUD_LUIGI, CKind_Luigi, ICONSTATE_TEMP, ICONJOINT_LUIGI, 0xCC, ICONBNDS_COL2_L, ICONBNDS_COL3_L, tt, mt);
    t[3] = icon(ICONHUD_KOOPA, CKind_Koopa, ICONSTATE_TEMP, ICONJOINT_KOOPA, 0xCA, ICONBNDS_COL3_L, ICONBNDS_COL4_L, tt, mt);
    t[4] = icon(ICONHUD_PEACH, CKind_Peach, ICONSTATE_TEMP, ICONJOINT_PEACH, 0xD1, ICONBNDS_COL4_L, ICONBNDS_COL5_L, tt, mt);
    t[5] = icon(ICONHUD_YOSHI, CKind_Yoshi, ICONSTATE_TEMP, ICONJOINT_YOSHI, 0xD7, ICONBNDS_COL5_L, ICONBNDS_COL6_L, tt, mt);
    t[6] = icon(ICONHUD_DONKEY, CKind_Donkey, ICONSTATE_TEMP, ICONJOINT_DONKEY, 0xC4, ICONBNDS_COL6_L, ICONBNDS_COL7_L, tt, mt);
    t[7] = icon(ICONHUD_CAPTAIN, CKind_Captain, ICONSTATE_TEMP, ICONJOINT_CAPTAIN, 0xC2, ICONBNDS_COL7_L, ICONBNDS_COL8_L, tt, mt);
    t[8] = icon(ICONHUD_GANON, CKind_Ganon, ICONSTATE_UNLOCKED, ICONJOINT_GANON, 0xD9, ICONBNDS_COL8_L, ICONBNDS_COL8_R, tt, mt);
    // Middle row
    t[9] = icon(ICONHUD_FALCO, CKind_Falco, ICONSTATE_UNLOCKED, ICONJOINT_FALCO, 0xC6, ICONBNDS_COL0_L, ICONBNDS_COL1_L, mt, bt);
    t[10] = icon(ICONHUD_FOX, CKind_Fox, ICONSTATE_TEMP, ICONJOINT_FOX, 0xC7, ICONBNDS_COL1_L, ICONBNDS_COL2_L, mt, bt);
    t[11] = icon(ICONHUD_NESS, CKind_Ness, ICONSTATE_TEMP, ICONJOINT_NESS, 0xD0, ICONBNDS_COL2_L, ICONBNDS_COL3_L, mt, bt);
    t[12] = icon(ICONHUD_POPONANA, CKind_PopoNana, ICONSTATE_TEMP, ICONJOINT_POPONANA, 0xC8, ICONBNDS_COL3_L, ICONBNDS_COL4_L, mt, bt);
    t[13] = icon(ICONHUD_KIRBY, CKind_Kirby, ICONSTATE_TEMP, ICONJOINT_KIRBY, 0xC9, ICONBNDS_COL4_L, ICONBNDS_COL5_L, mt, bt);
    t[14] = icon(ICONHUD_SAMUS, CKind_Samus, ICONSTATE_TEMP, ICONJOINT_SAMUS, 0xD5, ICONBNDS_COL5_L, ICONBNDS_COL6_L, mt, bt);
    t[15] = icon(ICONHUD_ZELDA, CKind_Zelda, ICONSTATE_TEMP, ICONJOINT_ZELDA, 0xD6, ICONBNDS_COL6_L, ICONBNDS_COL7_L, mt, bt);
    t[16] = icon(ICONHUD_LINK, CKind_Link, ICONSTATE_TEMP, ICONJOINT_LINK, 0xCB, ICONBNDS_COL7_L, ICONBNDS_COL8_L, mt, bt);
    t[17] = icon(ICONHUD_CLINK, CKind_CLink, ICONSTATE_UNLOCKED, ICONJOINT_CLINK, 0xC3, ICONBNDS_COL8_L, ICONBNDS_COL8_R, mt, bt);
    // Bottom row
    t[18] = icon(ICONHUD_PICHU, CKind_Pichu, ICONSTATE_UNLOCKED, ICONJOINT_PICHU, 0xD2, -23.4_f64 as f32, ICONBNDS_COL2_L, bt, bb);
    t[19] = icon(ICONHUD_PIKACHU, CKind_Pikachu, ICONSTATE_TEMP, ICONJOINT_PIKACHU, 0xD3, ICONBNDS_COL2_L, ICONBNDS_COL3_L, bt, bb);
    t[20] = icon(ICONHUD_PURIN, CKind_Purin, ICONSTATE_TEMP, ICONJOINT_PURIN, 0xD4, ICONBNDS_COL3_L, ICONBNDS_COL4_L, bt, bb);
    t[21] = icon(ICONHUD_MEWTWO, CKind_Mewtwo, ICONSTATE_TEMP, ICONJOINT_MEWTWO, 0xCF, ICONBNDS_COL4_L, ICONBNDS_COL5_L, bt, bb);
    t[22] = icon(ICONHUD_GAMEWATCH, CKind_GameWatch, ICONSTATE_TEMP, ICONJOINT_GAMEWATCH, 0xD8, ICONBNDS_COL5_L, ICONBNDS_COL6_L, bt, bb);
    t[23] = icon(ICONHUD_MARS, CKind_Mars, ICONSTATE_TEMP, ICONJOINT_MARS, 0xCE, ICONBNDS_COL6_L, ICONBNDS_COL7_L, bt, bb);
    t[24] = icon(ICONHUD_EMBLEM, CKind_Emblem, ICONSTATE_UNLOCKED, ICONJOINT_EMBLEM, 0xDA, ICONBNDS_COL7_L, 23.6_f64 as f32, bt, bb);
    t
}

const fn mode_info(a: u16, b: u16, sfx: i32) -> CSSModeInfo {
    CSSModeInfo { mode_ffa_frame: a, mode_teams_frame: b, enter_sfx: sfx }
}

/// A `CSSDoor` row of the door table: nine joints, eleven zero bytes, four bounds.
const fn door_row(j: [u8; 9], b: [f32; 4]) -> CSSDoor {
    CSSDoor {
        emblem_joint: j[0],
        costume_joint: j[1],
        team_joint: j[2],
        door_joint: j[3],
        bg_joint: j[4],
        player_indicator_joint: j[5],
        slidername_joint: j[6],
        cpuslider_joint: j[7],
        cpuslider2_joint: j[8],
        selected_since_load: 0,
        team: 0,
        p_kind: 0,
        p_kind_prev: 0,
        costume: 0,
        sel_icon: 0,
        sel_icon_prev: 0,
        dooranim_timer: 0,
        slideranim_timer: 0,
        is_hold_cpu_slider: 0,
        is_hold_handicap_slider: 0,
        togglebtn_left: b[0],
        togglebtn_right: b[1],
        teambtn_left: b[2],
        teambtn_right: b[3],
    }
}

const fn tag(a: u8, b: u8, c: u8, d: u8, e: u8) -> CSSTag {
    CSSTag { data: null_mut(), x4: a, list_joint: b, name_jointl: c, x7: d, kostar_text_joint: e, x9: 0, xa: 0, xb: 0 }
}

const fn ko_star(x4: f32, joint: u8) -> CSSKOStar {
    CSSKOStar { text: null_mut(), x4, joint, joint2: 0, xc: 0, x10: 0, x14: 0, x18: 0, x1c: 0 }
}

crate::globals! {
    rw mnCharSel_804D50C8: [u8; 4] = [1, 2, 4, 8];
    rw mnCharSel_804D50CC: [u8; 4] = [1, 0, 0, 2];
    rw mnCharSel_804D50D0: [u8; 8] = [2, 0, 1, 0, 5, 3, 4, 0];
    rw mnCharSel_804D50D8: [u8; 8] = [2, 0, 8, 1, 7, 7, 7, 7];
    rw mnCharSel_804D50E0: [u8; 3] = [0, 1, 3];

    rw mnCharSel_804D6CB0: *mut CSSData = null_mut();
    rw css_data_table: *mut MnSelectChrDataTable = null_mut();
    rw mnCharSel_804D6CB8: *mut HSD_GObj = null_mut();
    rw mnCharSel_804D6CBC: *mut HSD_GObj = null_mut();
    rw mnCharSel_804D6CC0: *mut HSD_JObj = null_mut();
    rw mnCharSel_804D6CC4: *mut HSD_JObj = null_mut();
    rw mnCharSel_804D6CC8: *mut HSD_JObj = null_mut();
    rw mnCharSel_804D6CCC: *mut HSD_JObj = null_mut();
    rw mnCharSel_804D6CD0: *mut HSD_Archive = null_mut();
    rw mnCharSel_804D6CD4: *mut HSD_Archive = null_mut();
    rw css_models: *mut MnSelectChrModels = null_mut();
    rw mnCharSel_804D6CDC: *mut HSD_Text = null_mut();
    rw mnCharSel_804D6CE0: *mut HSD_Text = null_mut();
    rw mnCharSel_804D6CE4: *mut HSD_Text = null_mut();
    rw mnCharSel_804D6CE8: *mut HSD_Text = null_mut();
    rw mnCharSel_CStickText: *mut HSD_Text = null_mut();
    rw mnCharSel_804D6CEC: u32 = 0;
    rw mnCharSel_804D6CF0: i8 = 0;
    rw mnCharSel_804D6CF1: i8 = 0;
    rw mnCharSel_804D6CF2: u8 = 0;
    rw mnCharSel_804D6CF3: u8 = 0;
    rw mnCharSel_804D6CF4: i8 = 0;
    /// number of (open? valid?) CSS doors
    rw mnCharSel_804D6CF5: u8 = 0;
    /// `CSSPendingSceneChangeKind`
    rw mnCharSel_804D6CF6: u8 = 0;
    rw mnCharSel_804D6CF7: u8 = 0;
    rw mnCharSel_804D6CF8: u8 = 0;
    rw mnCharSel_804D6CF9: i8 = 0;

    /// `static u8 mnCharSel_IconCount = SELKIND_COUNT`
    rw mnCharSel_IconCount: u8 = SELKIND_COUNT_U8;
    /// NULL = the retail CSS
    rw mnCharSel_Mex: *mut MexSelectChr = null_mut();
    /// 1P stock dots (regend 53..57) use m-ex Stc_icns
    rw mnCharSel_StcAttached: i32 = 0;
    /// the joint tree icons[].joint_id_* resolve in
    rw mnCharSel_IconRoot: *mut HSD_JObj = null_mut();
    /// m-ex icon whose fighter the port lacks
    rw mnCharSel_IconUnavail: [u8; CSS_ICON_MAX] = [0; CSS_ICON_MAX];

    rw mnCharSel_803F0A48: CSSIconsData = CSSIconsData {
        gnw_name: [0x82, 0x6C, 0x82, 0x92, 0x81, 0x44, 0x82, 0x66, 0x82, 0x81, 0x82, 0x8D, 0x82, 0x85, 0x81, 0x95, 0x82, 0x76, 0x82, 0x81, 0x82, 0x94, 0x82, 0x83, 0x82, 0x88, 0x00, 0x00],
        mode_info: [
            mode_info(0x0000, 0x0001, 0x00007535),
            mode_info(0x0002, 0x000D, 0x0000753A),
            mode_info(0x0003, 0x000E, 0x0000753B),
            mode_info(0x0004, 0x000F, 0x0000753C),
            mode_info(0x0007, 0x0011, 0x0000753D),
            mode_info(0x0008, 0x0012, 0x0000753E),
            mode_info(0x0009, 0x0013, 0x0000753F),
            mode_info(0x0006, 0x0010, 0x00007540),
            mode_info(0x000A, 0x0014, 0x00007541),
            mode_info(0x000B, 0x0015, 0x00007535),
            mode_info(0x000C, 0x0016, 0x00007535),
            mode_info(0x0000, 0x0000, 0x00007534),
            mode_info(0x0001, 0x0000, 0x00007534),
            mode_info(0x0002, 0x0000, 0x00007533),
            mode_info(0x0003, 0x0000, 0x00007534),
            mode_info(0x0004, 0x0000, 0x0007C864),
            mode_info(0x0005, 0x0000, 0x00007531),
            mode_info(0x0008, 0x0000, 0x00007534),
            mode_info(0x0009, 0x0000, 0x00007534),
            mode_info(0x000A, 0x0000, 0x00007534),
            mode_info(0x000B, 0x0000, 0x00007534),
            mode_info(0x000C, 0x0000, 0x00007534),
            mode_info(0x000D, 0x0000, 0x00007534),
            mode_info(0x0007, 0x0000, 0x00007532),
        ],
    };

    rw icons: [CSSIcon; CSS_ICON_MAX + 1] = icons_init();

    rw mnCharSel_803F0DFC: CSSDoorsData = CSSDoorsData {
        doors: [
            door_row([0x2E, 0x33, 0x38, 0x85, 0x29, 0xA6, 0x3D, 0x41, 0x40], [-35.6_f64 as f32, -28.6_f64 as f32, -26.8_f64 as f32, -21.0]),
            door_row([0x2F, 0x34, 0x39, 0x8D, 0x2A, 0xA8, 0x43, 0x47, 0x46], [-19.4_f64 as f32, -13.4_f64 as f32, -11.4_f64 as f32, -6.0]),
            door_row([0x30, 0x35, 0x3A, 0x95, 0x2B, 0xAA, 0x49, 0x4D, 0x4C], [-4.2_f64 as f32, 2.2_f64 as f32, 3.5_f64 as f32, 9.4_f64 as f32]),
            door_row([0x31, 0x36, 0x3B, 0x9D, 0x2C, 0xAC, 0x4F, 0x53, 0x52], [11.0, 17.0, 19.0, 24.6_f64 as f32]),
        ],
    };

    rw mnCharSel_803F0E8C: [CSSTag; 4] = [
        tag(0x70, 0x73, 0x74, 0x72, 0x71),
        tag(0x75, 0x78, 0x79, 0x77, 0x76),
        tag(0x7A, 0x7D, 0x7E, 0x7C, 0x7B),
        tag(0x7F, 0x82, 0x83, 0x81, 0x80),
    ];

    rw mnCharSel_803F0EBC: CSSDoorsMisc = CSSDoorsMisc {
        xc0: 0, xc1: 0, xc2: 0, xc3: 0, xc4: 0x4A, name_list_joint: 0x4D, tag_box_joint: 0x4E, xc7: 0x4C,
        xc8: 0x4B, xc9: 0, xca: 0, xcb: 0, xcc: 0x2F, cpu_level: 0x01, cpu_level_shown: 0, scroll_flag: 0,
        cpu_slider_x: 0.0, xd3: null_mut(),
        cpudown_left: -10.9_f64 as f32, cpudown_right: -4.2_f64 as f32, cpuup_left: 12.5, cpuup_right: 19.6_f64 as f32,
        cpubtn_top: -6.8_f64 as f32, cpubtn_btm: -12.1_f64 as f32,
    };

    rw data2: CSSDoorsData2 = CSSDoorsData2 {
        xf0: [0x35, 0x39, 0x36, 0x38, 0x37],
        stocks: 0,
        xf6: 0,
        xf7: 0,
        xf8: -2.2_f64 as f32,
        xfc: 3.7_f64 as f32,
        x100: 13.7_f64 as f32,
        x104: 19.3_f64 as f32,
        x108: -12.4_f64 as f32,
        x10c: -16.6_f64 as f32,
        ko_stars: [ko_star(-22.5_f64 as f32, 0x57), ko_star(-7.1_f64 as f32, 0x5D), ko_star(8.3_f64 as f32, 0x63), ko_star(23.7_f64 as f32, 0x69)],
    };

    rw mnCharSel_804A0BC0: [*mut CSSCursorData; 4] = [null_mut(); 4];
    rw mnCharSel_804A0BD0: [*mut CSSCharModel; 4] = [null_mut(); 4];

    // ---- data defined in TUs that are not ported (they live here until they are) ----
    /// gobj.c `u8 HSD_GObj_CameraKind`
    rw HSD_GObj_CameraKind: u8 = 0;
    /// gobj.c `s8 HSD_GObj_FogKind`
    rw HSD_GObj_FogKind: i8 = 0;
    /// gobj.c `u8 HSD_GObj_JObjKind`
    rw HSD_GObj_JObjKind: u8 = 0;
    /// gobj.c `s8 HSD_GObj_LightKind`
    rw HSD_GObj_LightKind: i8 = 0;
    /// sobjlib.c `u8 HSD_SObjLib_804D7960`
    rw HSD_SObjLib_804D7960: u8 = 0;
    /// sislib.c `SIS* HSD_SisLib_804D1124[5]`
    rw HSD_SisLib_804D1124: [*mut SIS; 5] = [null_mut(); 5];
    /// menu camera descriptor
    rw MenMain_cam: *mut HSD_CObjDesc = null_mut();
    /// pc `gm_CStickSmashTargetTest`
    rw gm_CStickSmashTargetTest: i32 = 0;
}

crate::globals_ext! {
    rw HSD_PadCopyStatus = super::globals::HSD_PadCopyStatus: [HSD_PadStatus; 4];
}


// ------------------------------------------------------------------------------------------
// table accessors and the file's small helpers
// ------------------------------------------------------------------------------------------

/// `mnCharSel_803F0DFC.doors[i]`.
#[inline]
unsafe fn door(i: usize) -> *mut CSSDoor {
    (addr_of_mut!(mnCharSel_803F0DFC.doors) as *mut CSSDoor).add(i)
}
/// `icons[i]`.
#[inline]
unsafe fn icon_at(i: usize) -> *mut CSSIcon {
    (addr_of_mut!(icons) as *mut CSSIcon).add(i)
}
/// `mnCharSel_803F0E8C[i]`.
#[inline]
unsafe fn tag_at(i: usize) -> *mut CSSTag {
    (addr_of_mut!(mnCharSel_803F0E8C) as *mut CSSTag).add(i)
}
/// `data2.ko_stars[i]`.
#[inline]
unsafe fn ko_star_at(i: usize) -> *mut CSSKOStar {
    (addr_of_mut!(data2.ko_stars) as *mut CSSKOStar).add(i)
}
/// `css->vs.start.players[i]`.
#[inline]
unsafe fn player_at(css: *mut CSSData, i: usize) -> *mut PlayerInitData {
    (addr_of_mut!((*css).vs.start.players) as *mut PlayerInitData).add(i)
}
/// `HSD_PadCopyStatus[i]`.
#[inline]
unsafe fn pad(i: usize) -> *mut HSD_PadStatus {
    (addr_of_mut!(super::globals::HSD_PadCopyStatus) as *mut HSD_PadStatus).add(i)
}

/// `MNCS_NUM` / `MNCS_NUM_SK`: `(int) mnCharSel_IconCount`.
#[inline]
unsafe fn MNCS_NUM() -> i32 {
    mnCharSel_IconCount as i32
}
/// `MNCS_NUM_U`: `(unsigned int) mnCharSel_IconCount`.
#[inline]
unsafe fn MNCS_NUM_U() -> u32 {
    mnCharSel_IconCount as u32
}
/// `MNCS_ICON_ROOT(retail)`.
#[inline]
unsafe fn MNCS_ICON_ROOT(retail: *mut HSD_JObj) -> *mut HSD_JObj {
    if !mnCharSel_IconRoot.is_null() { mnCharSel_IconRoot } else { retail }
}
/// `MNCS_TRACE(...)`: the one-shot CSS diagnostic.
#[inline]
unsafe fn trace_on() -> bool {
    ext::Mex_CssTraceEnabled() != 0
}

/// `lb_80011E24(root, out, id, -1)`.
#[inline]
unsafe fn lb_find(root: *mut HSD_JObj, out: *mut *mut HSD_JObj, id: i32) -> i32 {
    ext::lb_80011E24(root, out, &[Va::I32(id), Va::I32(-1)])
}
/// `HSD_ForeachAnim(j, JOBJ_TYPE, mask, HSD_AObjReqAnim, AOBJ_ARG_AF, frame)`.
#[inline]
unsafe fn req_anim(j: *mut HSD_JObj, mask: u32, frame: f32) {
    ext::HSD_ForeachAnim(j.cast(), JOBJ_TYPE as u32, mask, ext::HSD_AObjReqAnim as *const () as *mut c_void, AOBJ_ARG_AF, &[Va::D(frame as f64)]);
}
/// `HSD_ForeachAnim(j, JOBJ_TYPE, mask, HSD_AObjStopAnim, AOBJ_ARG_AOV, 0, 0)`.
#[inline]
unsafe fn stop_anim(j: *mut HSD_JObj, mask: u32) {
    ext::HSD_ForeachAnim(j.cast(), JOBJ_TYPE as u32, mask, ext::HSD_AObjStopAnim as *const () as *mut c_void, AOBJ_ARG_AOV, &[Va::I32(0), Va::I32(0)]);
}
/// `HSD_SisLib_803A70A0(text, 0, fmt, ...)`.
#[inline]
unsafe fn text_set(text: *mut HSD_Text, fmt: *const i8, va: &[Va]) {
    ext::HSD_SisLib_803A70A0(text, 0, fmt, va);
}
/// `HSD_SisLib_803A70A0(text, 0, NULL)`.
#[inline]
unsafe fn text_clear(text: *mut HSD_Text) {
    ext::HSD_SisLib_803A70A0(text, 0, null(), &[]);
}
#[inline]
fn cstr(s: &'static core::ffi::CStr) -> *const i8 {
    s.as_ptr()
}

/// `HSD_JObjGetTranslationX/Y/Z`, `HSD_JObjGetScaleX/Y` (jobj.h, with their `HSD_ASSERT`).
macro_rules! jget {
    ($name:ident, $field:ident . $axis:ident, $line:literal) => {
        #[inline]
        unsafe fn $name(jobj: *mut HSD_JObj) -> f32 {
            super::hsd::hsd_assert(!jobj.is_null(), $line, c"jobj");
            (*jobj).$field.$axis
        }
    };
}
jget!(HSD_JObjGetTranslationX, translate.x, 582);
jget!(HSD_JObjGetTranslationY, translate.y, 588);
jget!(HSD_JObjGetTranslationZ, translate.z, 594);

/// `animateJoint`, `animateJointPadded` and `animateJointLeadingPad` (the three differ only in the
/// stack padding the original build matched): find joint `joint` below `root`, freeze its animation
/// on `frame` for the `mask` object types and return it.
unsafe fn animateJoint(root: *mut HSD_JObj, joint: u8, mask: u32, frame: f32) -> *mut HSD_JObj {
    let mut jobj: *mut HSD_JObj = null_mut();
    lb_find(root, &mut jobj, joint as i32);
    let cc0 = jobj;
    req_anim(cc0, mask, frame);
    ext::HSD_JObjAnimAll(jobj);
    let cc1 = jobj;
    stop_anim(cc1, mask);
    jobj
}
unsafe fn animateJointPadded(root: *mut HSD_JObj, joint: u8, mask: u32, frame: f32) -> *mut HSD_JObj {
    animateJoint(root, joint, mask, frame)
}
unsafe fn animateJointLeadingPad(root: *mut HSD_JObj, joint: u8, mask: u32, frame: f32) -> *mut HSD_JObj {
    animateJoint(root, joint, mask, frame)
}

/// `sethidden`.
unsafe fn sethidden(jobj: *mut HSD_JObj, hidden: i32) {
    if hidden != 0 {
        ext::HSD_JObjSetFlags(jobj, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjClearFlags(jobj, JOBJ_HIDDEN as u32);
    }
}

// ------------------------------------------------------------------------------------------
// HUD text
// ------------------------------------------------------------------------------------------

/// Writes the kerning of the digits of `arg1` (at most four) through `arg0` and returns the pointer
/// past the terminator slot.
pub unsafe fn mnCharSel_8025BC20(mut arg0: *mut TextKerning, mut arg1: u32) -> *mut TextKerning {
    let mut render_zeroes: u32 = 0;
    let kerning = (*HSD_SisLib_804D1124[0].add(41)).kerning;
    if arg1 >= 10000 {
        arg1 = 9999;
    }
    if arg1 >= 1000 {
        (*arg0).left = (*kerning.add((arg1 / 1000) as usize)).left;
        (*arg0).right = (*kerning.add((arg1 / 1000) as usize)).right;
        arg0 = arg0.add(1);
        arg1 %= 1000;
        render_zeroes = 1;
    }
    if arg1 >= 100 || render_zeroes != 0 {
        (*arg0).left = (*kerning.add((arg1 / 100) as usize)).left;
        (*arg0).right = (*kerning.add((arg1 / 100) as usize)).right;
        arg0 = arg0.add(1);
        arg1 %= 100;
        render_zeroes += 1;
    }
    if arg1 >= 10 || render_zeroes != 0 {
        (*arg0).left = (*kerning.add((arg1 / 10) as usize)).left;
        (*arg0).right = (*kerning.add((arg1 / 10) as usize)).right;
        arg0 = arg0.add(1);
        arg1 %= 10;
    }
    (*arg0).left = (*kerning.add(arg1 as usize)).left;
    (*arg0).right = (*kerning.add(arg1 as usize)).right;
    (*arg0.add(1)).left = 0;
    arg0 = arg0.add(1);
    arg0
}

/// Fills in the rules text (time / stock count and the mode name) of the CSS.
pub unsafe fn mnCharSel_8025BD30() {
    let kerning = (*HSD_SisLib_804D1124[0].add(37)).kerning;
    let match_type = (*mnCharSel_804D6CB0).match_type as u32;

    if match_type == VS_CAMERA {
        ext::HSD_SisLib_803A6530(0, 0x4A, 0x4F);
        return;
    }
    if match_type == VS_STAMINA {
        ext::HSD_SisLib_803A6530(0, 0x4A, 0x50);
        return;
    }
    if ext::lbLang_IsSavedLanguageJP() != 0 {
        if (*ext::gmMainLib_GetGameRules()).mode == 1 {
            mnCharSel_8025BC20(kerning, (*ext::gmMainLib_GetGameRules()).stock_count as u32);
        } else if (*ext::gmMainLib_GetGameRules()).time_limit != 0 {
            mnCharSel_8025BC20(kerning, (*ext::gmMainLib_GetGameRules()).time_limit as u32);
            ext::HSD_SisLib_803A660C(0, 0x4A, 0x53);
        } else {
            ext::HSD_SisLib_803A6530(0, 0x4A, 0x51);
        }
        match (*ext::gmMainLib_GetGameRules()).mode as u32 {
            Mode_Time => {
                ext::HSD_SisLib_803A660C(0, 0x4A, 0x4B);
            }
            Mode_Stock => {
                ext::HSD_SisLib_803A660C(0, 0x4A, 0x4C);
            }
            Mode_Coin => {
                ext::HSD_SisLib_803A660C(0, 0x4A, 0x4D);
            }
            Mode_Bonus => {
                ext::HSD_SisLib_803A660C(0, 0x4A, 0x4E);
            }
            _ => {}
        }
        // (every case returns; falling out of the switch reaches the literal-pool anchors below, which
        // are no code)
        return;
    } else {
        match (*ext::gmMainLib_GetGameRules()).mode as u32 {
            Mode_Time => {
                if (*ext::gmMainLib_GetGameRules()).time_limit != 0 {
                    mnCharSel_8025BC20(kerning, (*ext::gmMainLib_GetGameRules()).time_limit as u32);
                    ext::HSD_SisLib_803A660C(0, 0x4A, 0x53);
                } else {
                    ext::HSD_SisLib_803A6530(0, 0x4A, 0x51);
                }
                ext::HSD_SisLib_803A660C(0, 0x4A, 0x4B);
                return;
            }
            Mode_Stock => {
                mnCharSel_8025BC20(kerning, (*ext::gmMainLib_GetGameRules()).stock_count as u32);
                if ((*ext::gmMainLib_GetGameRules()).stock_count as u32) < 2 {
                    ext::HSD_SisLib_803A660C(0, 0x4A, 0x55);
                } else {
                    ext::HSD_SisLib_803A660C(0, 0x4A, 0x54);
                }
                ext::HSD_SisLib_803A660C(0, 0x4A, 0x4C);
                return;
            }
            Mode_Coin => {
                if (*ext::gmMainLib_GetGameRules()).time_limit != 0 {
                    mnCharSel_8025BC20(kerning, (*ext::gmMainLib_GetGameRules()).time_limit as u32);
                    ext::HSD_SisLib_803A660C(0, 0x4A, 0x53);
                } else {
                    ext::HSD_SisLib_803A6530(0, 0x4A, 0x51);
                }
                ext::HSD_SisLib_803A660C(0, 0x4A, 0x4D);
                return;
            }
            Mode_Bonus => {
                if (*ext::gmMainLib_GetGameRules()).time_limit != 0 {
                    mnCharSel_8025BC20(kerning, (*ext::gmMainLib_GetGameRules()).time_limit as u32);
                    ext::HSD_SisLib_803A660C(0, 0x4A, 0x53);
                } else {
                    ext::HSD_SisLib_803A6530(0, 0x4A, 0x51);
                }
                ext::HSD_SisLib_803A660C(0, 0x4A, 0x4E);
                return;
            }
            _ => {}
        }
    }
}

/// `getStickDelta`: the stick's dead-zone-reduced displacement (squared-magnitude minus 200, along
/// the stick's angle) of `port`.
pub unsafe fn getStickDelta(port: i32, dx: *mut f32, dy: *mut f32) {
    let p = pad((port as u8) as usize);
    let stick_x = loadStickValue(&(*p).stickX);
    let stick_y = (*p).stickY as f32;
    let mag_sq = fmadds(stick_x, stick_x, stick_y * stick_y);
    if mag_sq < 200.0f32 {
        *dy = 0.0f32;
        *dx = 0.0f32;
    } else {
        let adj = mag_sq - 200.0f32;
        let angle = atan2f(stick_x, stick_y);
        *dx = adj * sinf(angle);
        *dy = adj * cosf(angle);
    }
}

/// `drawTimeText`.
pub unsafe fn drawTimeText(x: *mut HSD_Text, y: *mut HSD_Text, hours: i32, minutes: i32, seconds: i32, microseconds: i32) {
    if hours == 0 {
        text_set(x, cstr(c"%02d:%02d"), &[Va::I32(minutes), Va::I32(seconds)]);
        text_set(y, cstr(c"%02d"), &[Va::I32(microseconds)]);
    } else if hours < 100 {
        text_set(x, cstr(c"%d:%02d:%02d"), &[Va::I32(hours), Va::I32(minutes), Va::I32(seconds)]);
        text_set(y, cstr(c"%02d"), &[Va::I32(microseconds)]);
    } else {
        text_set(x, cstr(c"99:59:59"), &[]);
        text_set(y, cstr(c"99"), &[]);
    }
}

pub unsafe fn getClassicHighscore(hud: u8) -> u32 {
    ext::gm_80162BD8(hud) as u32
}
pub unsafe fn getAdventureHighscore(hud: u8) -> u32 {
    ext::gm_80162DD4(hud) as u32
}
pub unsafe fn getAllStarHighscore(hud: u8) -> u32 {
    ext::gm_80162FD0(hud) as u32
}

/// `inline3`: freeze the HUD joint `i` of the CSS model on frame `x + 1` (material animations).
pub unsafe fn inline3(i: i32, x: f32) -> *mut HSD_JObj {
    let mut sp5C: *mut HSD_JObj = null_mut();
    let arc = mnCharSel_804D6CC0;
    let frame = x + 1.0f32;
    lb_find(arc, &mut sp5C, i);
    let cc0 = sp5C;
    req_anim(cc0, MOBJ_MASK, frame);
    ext::HSD_JObjAnimAll(sp5C);
    stop_anim(sp5C, MOBJ_MASK);
    sp5C
}

/// `show_kos(x, y)`.
unsafe fn show_kos(x: *mut HSD_Text, y: i32) {
    if ext::lbLang_IsSavedLanguageJP() != 0 {
        text_set(x, cstr(c"%d \x90\x6c\x94\xb2\x82\xab"), &[Va::I32(y)]);
    } else {
        text_set(x, cstr(c"%d \x82\x6a\x82\x6e\x82\x93"), &[Va::I32(y)]);
    }
}

/// Fills in the high-score / record texts and HUD of the current mode for fighter icon 0 (`arg0`
/// set: clear them).
pub unsafe fn mnCharSel_8025C020(arg0: i32) {
    let mut sp7C: *mut HSD_JObj = null_mut();
    let mut sp7B: u8 = 0;
    let mut sp7A: u8 = 0;
    let mut sp79: u8 = 0;
    let mut sp78: u8 = 0;

    let i = (*door(0)).sel_icon as i32;
    let hud_index = (*icon_at(i as usize)).ft_hudindex;
    if arg0 != 0 {
        if !mnCharSel_804D6CDC.is_null() {
            text_clear(mnCharSel_804D6CDC);
        }
        if !mnCharSel_804D6CE0.is_null() {
            text_clear(mnCharSel_804D6CE0);
        }
    }
    let ml = |x: u32| Va::I32(x as i32);
    match (*mnCharSel_804D6CB0).match_type as u32 {
        REG_CLASSIC => {
            sp7C = inline3(0x42, *ext::gmMainLib_8015D194(hud_index) as f32);
            if arg0 != 0 {
                ext::HSD_JObjSetFlags(sp7C, JOBJ_HIDDEN as u32);
            } else {
                text_set(mnCharSel_804D6CDC, cstr(c"%08d"), &[ml(getClassicHighscore(hud_index))]);
                if ext::gmMainLib_8015D0D8(hud_index) != 0 {
                    ext::HSD_JObjClearFlags(sp7C, JOBJ_HIDDEN as u32);
                }
            }
            text_set(mnCharSel_804D6CE4, cstr(c"%09d"), &[Va::I32(ext::gm_80162C48())]);
            if ext::gm_80162D1C() != 0 {
                sp7C = inline3(0x43, ext::gm_80162D6C() as f32);
                ext::HSD_JObjClearFlags(sp7C, JOBJ_HIDDEN as u32);
            }
        }
        REG_ADVENTURE => {
            sp7C = inline3(0x42, *ext::gmMainLib_8015D2BC(hud_index) as f32);
            if arg0 != 0 {
                ext::HSD_JObjSetFlags(sp7C, JOBJ_HIDDEN as u32);
            } else {
                text_set(mnCharSel_804D6CDC, cstr(c"%08d"), &[ml(getAdventureHighscore(hud_index))]);
                if ext::gmMainLib_8015D200(hud_index) != 0 {
                    ext::HSD_JObjClearFlags(sp7C, JOBJ_HIDDEN as u32);
                }
            }
            text_set(mnCharSel_804D6CE4, cstr(c"%09d"), &[Va::I32(ext::gm_80162E44())]);
            if ext::gm_80162F18() != 0 {
                sp7C = inline3(0x43, ext::gm_80162F68() as f32);
                ext::HSD_JObjClearFlags(sp7C, JOBJ_HIDDEN as u32);
            }
        }
        REG_ALLSTAR => {
            sp7C = inline3(0x42, *ext::gmMainLib_8015D3E4(hud_index) as f32);
            if arg0 != 0 {
                ext::HSD_JObjSetFlags(sp7C, JOBJ_HIDDEN as u32);
            } else {
                text_set(mnCharSel_804D6CDC, cstr(c"%08d"), &[ml(getAllStarHighscore(hud_index))]);
                if ext::gmMainLib_8015D328(hud_index) != 0 {
                    ext::HSD_JObjClearFlags(sp7C, JOBJ_HIDDEN as u32);
                }
            }
            text_set(mnCharSel_804D6CE4, cstr(c"%09d"), &[Va::I32(ext::gm_80163040())]);
            if ext::gm_80163114() != 0 {
                sp7C = inline3(0x43, ext::gm_80163164() as f32);
                ext::HSD_JObjClearFlags(sp7C, JOBJ_HIDDEN as u32);
            }
        }
        STADIUM_TARGET => {
            if arg0 == 0 {
                let target_count = ext::gm_8016332C(hud_index);
                if target_count as u32 != u32::MAX {
                    if ext::lbLang_IsSavedLanguageJP() != 0 {
                        text_set(mnCharSel_804D6CDC, cstr(c"%d \x8c\xc2"), &[Va::I32(target_count)]);
                    } else {
                        text_set(mnCharSel_804D6CDC, cstr(c"%d \x81\x40"), &[Va::I32(target_count)]);
                    }
                } else {
                    ext::gm_80163374(hud_index, &mut sp7B, &mut sp7A, &mut sp79, &mut sp78);
                    drawTimeText(mnCharSel_804D6CDC, mnCharSel_804D6CE0, sp7B as i32, sp7A as i32, sp79 as i32, sp78 as i32);
                }
            }
            lb_find(mnCharSel_804D6CC8, &mut sp7C, 3);
            if ext::gm_801634D4(&mut sp7B, &mut sp7A, &mut sp79, &mut sp78) != 0 {
                drawTimeText(mnCharSel_804D6CE4, mnCharSel_804D6CE8, sp7B as i32, sp7A as i32, sp79 as i32, sp78 as i32);
                ext::HSD_JObjClearFlagsAll(sp7C, JOBJ_HIDDEN as u32);
            } else {
                text_clear(mnCharSel_804D6CE4);
                text_clear(mnCharSel_804D6CE8);
                ext::HSD_JObjSetFlagsAll(sp7C, JOBJ_HIDDEN as u32);
            }
            if !mnCharSel_CStickText.is_null() {
                text_set(
                    mnCharSel_CStickText,
                    if gm_CStickSmashTargetTest != 0 { cstr(c"C-STICK: SMASH") } else { cstr(c"C-STICK: CAMERA") },
                    &[],
                );
            }
        }
        STADIUM_HOMERUN => {
            if arg0 == 0 {
                if ext::lbLang_IsSavedLanguageJP() != 0 {
                    text_set(mnCharSel_804D6CDC, cstr(c"%.1f"), &[Va::D(toMeters(ext::gm_801631CC(hud_index) as f32) as f64)]);
                } else {
                    text_set(mnCharSel_804D6CDC, cstr(c"%.1f"), &[Va::D(toFeet(ext::gm_801631CC(hud_index) as f32) as f64)]);
                }
            }
            if ext::lbLang_IsSavedLanguageJP() != 0 {
                text_set(
                    mnCharSel_804D6CE4,
                    cstr(c"%.1f"),
                    &[Va::D((f2i32((10.0f32 * (ext::gm_801631F0() as f32 / 100.0f32)) as f64) as f32 / 10.0f32) as f64)],
                );
            } else {
                text_set(
                    mnCharSel_804D6CE4,
                    cstr(c"%.1f"),
                    &[Va::D((f2i32((10.0f32 * (ext::gm_801631F0() as f32 / 30.4788f32)) as f64) as f32 / 10.0f32) as f64)],
                );
            }
        }
        TRAINING_MODE => {
            if arg0 == 0 {
                text_set(mnCharSel_804D6CDC, cstr(c"%d"), &[Va::I32(ext::gm_80163274(hud_index) as i32)]);
            }
        }
        STADIUM_MULTIMAN_10 => {
            if arg0 == 0 {
                if ext::gm_8016365C(hud_index) != 0 {
                    show_kos(mnCharSel_804D6CDC, ext::gm_80163690(hud_index));
                } else {
                    ext::gm_801636D8(hud_index, &mut sp7B, &mut sp7A, &mut sp79, &mut sp78);
                    drawTimeText(mnCharSel_804D6CDC, mnCharSel_804D6CE0, sp7B as i32, sp7A as i32, sp79 as i32, sp78 as i32);
                }
            }
            lb_find(mnCharSel_804D6CC8, &mut sp7C, 3);
            if ext::gm_80163838(&mut sp7B, &mut sp7A, &mut sp79, &mut sp78) != 0 {
                drawTimeText(mnCharSel_804D6CE4, mnCharSel_804D6CE8, sp7B as i32, sp7A as i32, sp79 as i32, sp78 as i32);
                ext::HSD_JObjClearFlagsAll(sp7C, JOBJ_HIDDEN as u32);
            } else {
                text_clear(mnCharSel_804D6CE4);
                text_clear(mnCharSel_804D6CE8);
                ext::HSD_JObjSetFlagsAll(sp7C, JOBJ_HIDDEN as u32);
            }
        }
        STADIUM_MULTIMAN_100 => {
            if arg0 == 0 {
                if ext::gm_801639C0(hud_index) != 0 {
                    show_kos(mnCharSel_804D6CDC, ext::gm_801639F4(hud_index));
                } else {
                    ext::gm_80163A3C(hud_index, &mut sp7B, &mut sp7A, &mut sp79, &mut sp78);
                    drawTimeText(mnCharSel_804D6CDC, mnCharSel_804D6CE0, sp7B as i32, sp7A as i32, sp79 as i32, sp78 as i32);
                }
            }
            lb_find(mnCharSel_804D6CC8, &mut sp7C, 3);
            if ext::gm_80163B9C(&mut sp7B, &mut sp7A, &mut sp79, &mut sp78) != 0 {
                drawTimeText(mnCharSel_804D6CE4, mnCharSel_804D6CE8, sp7B as i32, sp7A as i32, sp79 as i32, sp78 as i32);
                ext::HSD_JObjClearFlagsAll(sp7C, JOBJ_HIDDEN as u32);
            } else {
                text_clear(mnCharSel_804D6CE4);
                text_clear(mnCharSel_804D6CE8);
                ext::HSD_JObjSetFlagsAll(sp7C, JOBJ_HIDDEN as u32);
            }
        }
        STADIUM_3_MIN_MELEE => {
            if arg0 == 0 {
                if ext::lbLang_IsSavedLanguageJP() != 0 {
                    text_set(mnCharSel_804D6CDC, cstr(c"%d \x90\x6c\x94\xb2\x82\xab"), &[Va::I32(ext::gm_Get3MinMultimanHighscore(hud_index) as i32)]);
                } else {
                    text_set(mnCharSel_804D6CDC, cstr(c"%d \x82\x6a\x82\x6e\x82\x93"), &[Va::I32(ext::gm_Get3MinMultimanHighscore(hud_index) as i32)]);
                }
            }
            if ext::lbLang_IsSavedLanguageJP() != 0 {
                text_set(mnCharSel_804D6CE4, cstr(c"%d \x90\x6c\x94\xb2\x82\xab"), &[Va::I32(ext::gm_Get3MinMultimanTotalHighscore())]);
            } else {
                text_set(mnCharSel_804D6CE4, cstr(c"%d \x82\x6a\x82\x6e\x82\x93"), &[Va::I32(ext::gm_Get3MinMultimanTotalHighscore())]);
            }
        }
        STADIUM_15_MIN_MELEE => {
            if arg0 == 0 {
                if ext::lbLang_IsSavedLanguageJP() != 0 {
                    text_set(mnCharSel_804D6CDC, cstr(c"%d \x90\x6c\x94\xb2\x82\xab"), &[Va::I32(ext::gm_Get15MinMultimanHighscore(hud_index) as i32)]);
                } else {
                    text_set(mnCharSel_804D6CDC, cstr(c"%d \x82\x6a\x82\x6e\x82\x93"), &[Va::I32(ext::gm_Get15MinMultimanHighscore(hud_index) as i32)]);
                }
            }
            if ext::lbLang_IsSavedLanguageJP() != 0 {
                text_set(mnCharSel_804D6CE4, cstr(c"%d \x90\x6c\x94\xb2\x82\xab"), &[Va::I32(ext::gm_Get15MinMultimanTotalHighscore())]);
            } else {
                text_set(mnCharSel_804D6CE4, cstr(c"%d \x82\x6a\x82\x6e\x82\x93"), &[Va::I32(ext::gm_Get15MinMultimanTotalHighscore())]);
            }
        }
        STADIUM_ENDLESS_MELEE => {
            if arg0 == 0 {
                show_kos(mnCharSel_804D6CDC, ext::gm_GetEndlessHighscore(hud_index));
            }
            show_kos(mnCharSel_804D6CE4, ext::gm_GetEndlessTotalHighscore());
        }
        STADIUM_CRUEL_MELEE => {
            if arg0 == 0 {
                show_kos(mnCharSel_804D6CDC, ext::gm_GetCruelHighscore(hud_index));
            }
            show_kos(mnCharSel_804D6CE4, ext::gm_GetCruelTotalHighscore());
        }
        _ => {}
    }
}

/// The K.O. star counter of door `port` (`mode` 0: hide, 1: dim, otherwise bright).
pub unsafe fn mnCharSel_8025D1C4(port: i32, mode: i32) {
    let css = mnCharSel_804D6CB0;
    let mut sp10: *mut HSD_JObj = null_mut();

    let show_stars = matches!(
        (*css).match_type as u32,
        VS_MELEE | VS_SUDDEN_DEATH | VS_GIANT | VS_TINY | VS_INVISIBLE | VS_FIXED_CAMERA | VS_SINGLE_BUTTON | VS_LIGHTNING | VS_SLOWMO
    );

    if show_stars {
        let stars = ko_star_at(port as usize);
        let sis_buf: *mut u8 = addr_of_mut!((*ko_star_at(port as usize)).joint2);
        let star_count = *(*css).ko_counts.add(port as usize) as i32;
        let mut star_x = (*ko_star_at(port as usize)).x4;
        let text = (*stars).text;

        lb_find(mnCharSel_804D6CC0, &mut sp10, (*ko_star_at(port as usize)).joint as i32);

        if mode == 0 || star_count == 0 {
            (*text).hidden = 1;
            ext::HSD_JObjSetFlagsAll(sp10, JOBJ_HIDDEN as u32);
            return;
        }
        ext::HSD_JObjClearFlagsAll(sp10, JOBJ_HIDDEN as u32);
        if star_count < 6 {
            (*text).hidden = 1;
            super::hsd::HSD_JObjSetTranslateX(sp10, fmadds(0.9f32, star_count as f32, star_x) - 4.5f32);
            let child = if sp10.is_null() { null_mut() } else { (*sp10).child };
            sp10 = child;
            if star_count < 5 {
                ext::HSD_JObjSetFlags(sp10, JOBJ_HIDDEN as u32);
            }
            let next1 = if sp10.is_null() { null_mut() } else { (*sp10).next };
            sp10 = next1;
            if star_count < 4 {
                ext::HSD_JObjSetFlags(sp10, JOBJ_HIDDEN as u32);
            }
            let next2 = if sp10.is_null() { null_mut() } else { (*sp10).next };
            sp10 = next2;
            if star_count < 3 {
                ext::HSD_JObjSetFlags(sp10, JOBJ_HIDDEN as u32);
            }
            let next3 = if sp10.is_null() { null_mut() } else { (*sp10).next };
            sp10 = next3;
            if star_count < 2 {
                ext::HSD_JObjSetFlags(sp10, JOBJ_HIDDEN as u32);
            }
        } else {
            (*text).hidden = 0;
            if star_count >= 0x3E8 {
                star_x -= 3.0f32;
            } else if star_count >= 100 {
                star_x -= 1.5f32;
            }

            super::hsd::HSD_JObjSetTranslateX(sp10, star_x);

            (*text).pos_x = 0.6f32 + star_x;
            if mode == 1 {
                (*text).text_color.a = 0x50;
            } else {
                (*text).text_color.a = 0xA0;
            }
            {
                let tmp = ext::HSD_SisLib_803A6478(sis_buf, (*HSD_SisLib_804D1124[0].add(43)).kerning as *mut u8) as *mut TextKerning;
                mnCharSel_8025BC20(tmp, star_count as u32);
            }
            ext::HSD_SisLib_803A6368(text, 0x56);
            (*text).sis_buffer = sis_buf as *mut SIS;
            sp10 = jobj_child(sp10);
            ext::HSD_JObjSetFlags(sp10, JOBJ_HIDDEN as u32);
            sp10 = jobj_next(sp10);
            ext::HSD_JObjSetFlags(sp10, JOBJ_HIDDEN as u32);
            sp10 = jobj_next(jobj_next(sp10));
            ext::HSD_JObjSetFlags(sp10, JOBJ_HIDDEN as u32);
            sp10 = jobj_next(sp10);
            ext::HSD_JObjSetFlags(sp10, JOBJ_HIDDEN as u32);
        }
    }
}

/// Plays door `door`'s card frame (portrait, emblem, stock dots) on `frame`, shown or `hidden`.
pub unsafe fn mnCharSel_8025D5AC(door_idx: i32, mut frame: i32, hidden: i32) {
    if mnCharSel_804D6CF5 == 1 {
        if door_idx != 0 {
            let sp5C = animateJoint(mnCharSel_804D6CC4, 4, TOBJ_MASK, frame as f32);
            sethidden(sp5C, hidden);

            let sp58 = animateJoint(mnCharSel_804D6CC4, 6, TOBJ_MASK, frame as f32);
            sethidden(sp58, hidden);
            return;
        }
        let sp54 = animateJoint(mnCharSel_804D6CC0, 0x2B, TOBJ_MASK, frame as f32);
        sethidden(sp54, hidden);

        let sp50 = animateJoint(mnCharSel_804D6CC0, 0x2D, TOBJ_MASK, frame as f32);
        sethidden(sp50, hidden);
        if hidden != 0 {
            frame = 0xB9;
            // m-ex "CSS - Change Stock Frame Dot (2)": the empty-slot dot is Stc_icns frame 7 (the red
            // ring); 0xB9 there is some fighter's icon.
            if mnCharSel_StcAttached != 0 {
                frame = 7;
            }
        }
        for i in 0..5usize {
            let tmp = data2.xf0[i];
            animateJoint(mnCharSel_804D6CC0, tmp, TOBJ_MASK, frame as f32);
        }
        mnCharSel_8025C020(hidden);
        return;
    }

    let sp48 = animateJoint(mnCharSel_804D6CC0, (*door(door_idx as usize)).costume_joint, TOBJ_MASK, frame as f32);
    sethidden(sp48, hidden);

    let sp44 = animateJoint(mnCharSel_804D6CC0, (*door(door_idx as usize)).emblem_joint, TOBJ_MASK, frame as f32);
    sethidden(sp44, hidden);
}

/// `isDuplicateCostumeWith`.
pub unsafe fn isDuplicateCostumeWith(door_idx: i32, css: *mut CSSData, door_count: u8) -> bool {
    let base_door = door(door_idx as usize);
    let num_doors: i32 = if (*css).match_type as u32 == TRAINING_MODE { 2 } else { door_count as i32 };

    for j in 0..num_doors {
        let other_door = door(j as usize);
        if door_idx != j
            && (*other_door).p_kind != 3
            && ((*other_door).sel_icon as i32) < MNCS_NUM()
            && (*other_door).sel_icon == (*base_door).sel_icon
            && (*base_door).costume == (*other_door).costume
        {
            return true;
        }
    }
    false
}

/// `loadCSSValue`.
pub unsafe fn loadCSSValue(css: *const *mut CSSData) -> *mut CSSData {
    core::ptr::read_volatile(css)
}

/// `isDuplicateCostumeCached`: reads the base door's icon and costume once before the scan.
pub unsafe fn isDuplicateCostumeCached(door_idx: i32) -> bool {
    let base_door = door(door_idx as usize);
    let num_doors: i32 = if (*mnCharSel_804D6CB0).match_type as u32 == TRAINING_MODE { 2 } else { mnCharSel_804D6CF5 as i32 };

    let sel = (*base_door).sel_icon;
    let cost = (*base_door).costume;

    for j in 0..num_doors {
        let o = door(j as usize);
        if door_idx != j && (*o).p_kind != 3 && ((*o).sel_icon as i32) < MNCS_NUM() && equalU8((*o).sel_icon, sel) && cost == (*o).costume {
            return true;
        }
    }
    false
}

/// `isDuplicateCostumeExact`.
pub unsafe fn isDuplicateCostumeExact(door_idx: i32) -> bool {
    let num_doors: i32 = if (*mnCharSel_804D6CB0).match_type as u32 == TRAINING_MODE {
        2
    } else {
        let door_count: u8 = mnCharSel_804D6CF5;
        door_count as i32
    };

    let base_door = door(door_idx as usize);
    for j in 0..num_doors {
        let other_door = door(j as usize);
        if door_idx != j
            && (*other_door).p_kind != 3
            && ((*other_door).sel_icon as i32) < MNCS_NUM()
            && (*other_door).sel_icon == (*base_door).sel_icon
            && (*base_door).costume == (*other_door).costume
        {
            return true;
        }
    }
    false
}

/// `isDuplicateCostume`.
pub unsafe fn isDuplicateCostume(door_idx: i32) -> bool {
    let base_door = door(door_idx as usize);
    let num_doors: i32 = if (*mnCharSel_804D6CB0).match_type as u32 == TRAINING_MODE { 2 } else { mnCharSel_804D6CF5 as i32 };

    for j in 0..num_doors {
        let other_door = door(j as usize);
        if door_idx != j
            && (*other_door).p_kind != 3
            && ((*other_door).sel_icon as i32) < MNCS_NUM()
            && (*other_door).sel_icon == (*base_door).sel_icon
            && (*base_door).costume == (*other_door).costume
        {
            return true;
        }
    }
    false
}

/// Whether another open door already wears `door`'s fighter in `door`'s costume.
pub unsafe fn mnCharSel_8025DAA0(door_idx: i32) -> bool {
    let num_doors: i32 = if (*mnCharSel_804D6CB0).match_type as u32 == TRAINING_MODE { 2 } else { mnCharSel_804D6CF5 as i32 };

    for j in 0..num_doors {
        if door_idx != j
            && (*door(j as usize)).p_kind != 3
            && ((*door(j as usize)).sel_icon as i32) < MNCS_NUM()
            && (*door(j as usize)).sel_icon == (*door(door_idx as usize)).sel_icon
            && (*door(door_idx as usize)).costume == (*door(j as usize)).costume
        {
            return true;
        }
    }
    false
}

/// `pickUniqueCostume`.
pub unsafe fn pickUniqueCostume(door_idx: i32) {
    if mnCharSel_8025DAA0(door_idx) {
        let mut v: i32 = 0;
        loop {
            (*door(door_idx as usize)).costume = v as u8;
            if !mnCharSel_8025DAA0(door_idx) {
                break;
            }
            v += 1;
        }
    }
}

/// `getHandicapValue`.
pub unsafe fn getHandicapValue(port: i32) -> i32 {
    let hval: i32 = if (*ext::gmMainLib_GetGameRules()).handicap == 1 {
        (ext::gm_801685D4(port as u8, (*player_at(mnCharSel_804D6CB0, port as usize)).nametag) as u8) as i32
    } else {
        ((*player_at(mnCharSel_804D6CB0, port as usize)).handicap as u8) as i32
    };
    if hval != 0 { hval } else { 1 }
}

// ------------------------------------------------------------------------------------------
// m-ex portrait support (TARGET_PC)
// ------------------------------------------------------------------------------------------

/// The frame of one fighter's portrait inside mexSelectChr's single CSP material animation:
/// `ext + costume * csp_stride`, an ANIMATION FRAME (not an index into the animation's image
/// table), bounded by the animation's own `end_frame`.
unsafe fn mnCharSel_MexCspFrame(ext_kind: i32, mut costume: i32) -> i32 {
    let mut end: i32 = 0;

    if ext_kind < 0 || mnCharSel_Mex.is_null() {
        return 0;
    }
    let mut stride = (*mnCharSel_Mex).csp_stride;
    if stride < 0 {
        stride = 0;
    }
    if costume < 0 {
        costume = 0;
    }
    let mut frame = ext_kind.wrapping_add(costume.wrapping_mul(stride));
    if !(*mnCharSel_Mex).csp_matanim.is_null()
        && !(*(*mnCharSel_Mex).csp_matanim).texanim.is_null()
        && !(*(*(*mnCharSel_Mex).csp_matanim).texanim).aobjdesc.is_null()
    {
        end = f2i32((*(*(*(*mnCharSel_Mex).csp_matanim).texanim).aobjdesc).end_frame as f64);
    }
    if end > 0 && frame > end {
        // Past the last authored costume row: fall back to this fighter's costume 0 rather than to
        // frame 0, which is somebody else's portrait.
        if trace_on() {
            ext::OSReport(
                c"css: portrait ext %d costume %d -> frame %d is past the animation's last frame %d - using costume 0\n".as_ptr(),
                &[Va::I32(ext_kind), Va::I32(costume), Va::I32(frame), Va::I32(end)],
            );
        }
        frame = ext_kind;
        if frame > end {
            return 0;
        }
    }
    if trace_on() {
        ext::OSReport(
            c"css: portrait ext %d costume %d stride %d -> CSP frame %d of %d\n".as_ptr(),
            &[Va::I32(ext_kind), Va::I32(costume), Va::I32(stride), Va::I32(frame), Va::I32(end)],
        );
    }
    frame
}

unsafe fn mnCharSel_MexPortrait(door_idx: i32, icon_idx: i32, costume: i32, retail_frame: i32) {
    let ext_kind = ext::Mex_PortCKindToExt((*icon_at(icon_idx as usize)).char_kind as i32);
    let emblem = ext::Mex_InsigniaForExt(ext_kind);
    let mut j: *mut HSD_JObj;
    if ext_kind < 0 {
        mnCharSel_8025D5AC(door_idx, 0, 1);
        return;
    }
    if mnCharSel_804D6CF5 == 1 {
        // 1P / Training. Run the retail routine first - it also drives the stock-icon dots and the
        // rest of the card - then override the portrait and emblem with m-ex frames. The player's
        // card is the regend model (portrait 45, emblem 43); the Training CPU door is its own model
        // (portrait 6, emblem 4).
        let root = if door_idx != 0 { mnCharSel_804D6CC4 } else { mnCharSel_804D6CC0 };
        let cj: i32 = if door_idx != 0 { 6 } else { 0x2D };
        let ej: i32 = if door_idx != 0 { 4 } else { 0x2B };
        mnCharSel_8025D5AC(door_idx, retail_frame, 0);
        if root.is_null() {
            return;
        }
        if door_idx == 0 && mnCharSel_StcAttached != 0 {
            // m-ex "CSS - Costume Change Rewrite": the stock dots show this fighter's icon
            let sf = ext::gm_MexStockFrame(ext::Player_800325C8((*icon_at(icon_idx as usize)).char_kind as u32, 0) as i32, costume);
            for d in 0..5usize {
                animateJoint(mnCharSel_804D6CC0, data2.xf0[d], TOBJ_MASK, if sf >= 0.0f32 { sf } else { 7.0f32 });
            }
        }
        j = animateJoint(root, cj as u8, TOBJ_MASK, mnCharSel_MexCspFrame(ext_kind, costume) as f32);
        sethidden(j, 0);
        j = animateJoint(root, ej as u8, TOBJ_MASK, (if emblem >= 0 { emblem } else { 0 }) as f32);
        sethidden(j, (emblem < 0) as i32);
        return;
    }
    j = animateJoint(mnCharSel_804D6CC0, (*door(door_idx as usize)).costume_joint, TOBJ_MASK, mnCharSel_MexCspFrame(ext_kind, costume) as f32);
    sethidden(j, 0);
    j = animateJoint(mnCharSel_804D6CC0, (*door(door_idx as usize)).emblem_joint, TOBJ_MASK, (if emblem >= 0 { emblem } else { 0 }) as f32);
    sethidden(j, (emblem < 0) as i32);
}

/// Refreshes everything door `arg0` shows: name, costume, portrait, door / handicap / CPU
/// sliders, team colours and the tag text.
pub unsafe fn mnCharSel_8025DB34(arg0: u8) {
    let mut sp90: *mut HSD_JObj = null_mut();
    let mut anim_frame: f32;
    let mut joint: u8;
    let sel_icon: u8;
    let mut hud_idx: i32;
    let mut color: i32;
    let mut door_frame: f32;
    let a = arg0 as usize;
    sel_icon = (*door(a)).sel_icon;
    hud_idx = (*icon_at(sel_icon as usize)).ft_hudindex as i32;
    mnCharSel_8025D5AC(arg0 as i32, 0, 1);

    // Name display
    if (*(*tag_at(a)).data).use_tag == 0 && (sel_icon as i32) < MNCS_NUM() {
        (*(*(*tag_at(a)).data).text).default_kerning = 1;
        if ext::lbLang_IsSavedLanguageUS() != 0 && sel_icon as i32 == 0x16 {
            ext::HSD_SisLib_803A70A0((*(*tag_at(a)).data).text, 0, addr_of!(mnCharSel_803F0A48.gnw_name) as *const i8, &[]);
        } else {
            ext::HSD_SisLib_803A70A0((*(*tag_at(a)).data).text, 0, ext::gm_80160980((*icon_at(sel_icon as usize)).char_kind), &[]);
        }
    }

    // Costume duplicate check
    {
        let new_icon = (*door(a)).sel_icon;
        if new_icon != (*door(a)).sel_icon_prev {
            (*door(a)).sel_icon_prev = new_icon;
            {
                let mut costume_var: u8 = 0;
                loop {
                    (*door(a)).costume = costume_var;
                    if !isDuplicateCostumeExact(arg0 as i32) {
                        break;
                    }
                    costume_var = costume_var.wrapping_add(1);
                }
            }
            if mnCharSel_804D6CF6 != 3 && mnCharSel_804D6CF6 != 4 {
                sfxMove();
            }
        }
    }

    if mnCharSel_804D6CF5 == 1 {
        // 1P mode
        if arg0 != 0 {
            anim_frame = mnCharSel_804D50D8[1] as f32;
            animateJoint(mnCharSel_804D6CC4, 2, TOBJ_MASK, anim_frame);
            anim_frame = mnCharSel_804D50D8[1] as f32;
            animateJoint(mnCharSel_804D6CC4, 4, MOBJ_MASK, anim_frame);
        } else {
            anim_frame = mnCharSel_804D50D8[mnCharSel_804D6CF0 as usize] as f32;
            animateJoint(mnCharSel_804D6CC0, 0x29, TOBJ_MASK, anim_frame);
            anim_frame = mnCharSel_804D50D8[mnCharSel_804D6CF0 as usize] as f32;
            animateJoint(mnCharSel_804D6CC0, 0x2B, MOBJ_MASK, anim_frame);
        }
        if (*(*tag_at(a)).data).use_tag == 0 && (*door(a)).sel_icon as u32 >= MNCS_NUM_U() {
            (*(*(*tag_at(a)).data).text).hidden = 1;
        } else {
            (*(*(*tag_at(a)).data).text).hidden = 0;
        }
    } else {
        // VS mode
        joint = (*door(a)).door_joint;
        lb_find(mnCharSel_804D6CC0, &mut sp90, joint as i32);
        if (*door(a)).p_kind_prev == 3 {
            {
                let anim_jobj = sp90;
                req_anim(anim_jobj, ALL_TYPE_MASK, 0.0f32);
            }
            ext::HSD_JObjAnimAll(sp90);
            if (*door(a)).p_kind == 3 {
                stop_anim(sp90, ALL_TYPE_MASK);
            } else {
                mnCharSel_8025D1C4(arg0 as i32, 2);
                pickUniqueCostume(arg0 as i32);
                (*door(a)).dooranim_timer = 0x1E;
                if mnCharSel_804D6CF6 != 3 && mnCharSel_804D6CF6 != 4 {
                    ext::lbAudioAx_800237A8(0xB9, 0x7F, 0x40);
                }
            }
        } else {
            {
                let anim_jobj = sp90;
                req_anim(anim_jobj, ALL_TYPE_MASK, 40.0f32);
            }
            ext::HSD_JObjAnimAll(sp90);
            if (*door(a)).p_kind == 3 {
                mnCharSel_8025D1C4(arg0 as i32, 0);
                (*door(a)).dooranim_timer = 0x1E;
                if mnCharSel_804D6CF6 != 3 && mnCharSel_804D6CF6 != 4 {
                    ext::lbAudioAx_800237A8(0xBA, 0x7F, 0x40);
                }
            } else {
                stop_anim(sp90, ALL_TYPE_MASK);
            }
        }

        // Switch on p_kind for handicap/CPU
        if (*door(a)).p_kind != (*door(a)).p_kind_prev {
            joint = (*door(a)).slidername_joint;
            lb_find(mnCharSel_804D6CC0, &mut sp90, joint as i32);
            match (*door(a)).p_kind as i32 {
                2 => {}
                0 => {
                    if (*ext::gmMainLib_GetGameRules()).handicap != 0 {
                        if (*ext::gmMainLib_GetGameRules()).handicap == 2 {
                            door_frame = 0.0f32;
                        } else {
                            door_frame = 60.0f32;
                        }
                        {
                            let anim_jobj = sp90;
                            req_anim(anim_jobj, ALL_TYPE_MASK, door_frame);
                        }

                        anim_frame = (*player_at(mnCharSel_804D6CB0, a)).cpu_level as f32;
                        joint = (*door(a)).cpuslider_joint;
                        animateJointPadded(mnCharSel_804D6CC0, joint, TOBJ_MASK, anim_frame);
                    }
                }
                1 => {
                    if (*ext::gmMainLib_GetGameRules()).handicap != 0 {
                        if (*ext::gmMainLib_GetGameRules()).handicap == 2 {
                            door_frame = 20.0f32;
                        } else {
                            door_frame = 80.0f32;
                        }
                        {
                            let anim_jobj = sp90;
                            req_anim(anim_jobj, ALL_TYPE_MASK, door_frame);
                        }
                        {
                            let hval: i32;
                            let port = arg0 as i32;
                            if (*ext::gmMainLib_GetGameRules()).handicap == 1 {
                                hval = (ext::gm_801685D4(port as u8, (*player_at(mnCharSel_804D6CB0, port as usize)).nametag) as u8) as i32;
                            } else {
                                hval = ((*player_at(mnCharSel_804D6CB0, port as usize)).handicap as u8) as i32;
                            }
                            anim_frame = (if hval != 0 { hval } else { 1 }) as f32;
                        }
                        joint = (*door(a)).cpuslider_joint;
                        sp90 = animateJoint(mnCharSel_804D6CC0, joint, TOBJ_MASK, anim_frame);
                        anim_frame = 1.25f32 * (getHandicapValue(arg0 as i32) - 1) as f32;
                        super::hsd::HSD_JObjSetTranslateX(sp90, anim_frame);

                        anim_frame = (*player_at(mnCharSel_804D6CB0, a)).cpu_level as f32;
                        joint = (*door(a)).cpuslider2_joint;
                        animateJoint(mnCharSel_804D6CC0, joint, TOBJ_MASK, anim_frame);
                    } else {
                        {
                            let anim_jobj = sp90;
                            req_anim(anim_jobj, ALL_TYPE_MASK, 40.0f32);
                        }

                        anim_frame = (*player_at(mnCharSel_804D6CB0, a)).cpu_level as f32;
                        joint = (*door(a)).cpuslider_joint;
                        animateJoint(mnCharSel_804D6CC0, joint, TOBJ_MASK, anim_frame);
                    }
                }
                3 => {
                    if (*ext::gmMainLib_GetGameRules()).handicap != 0 {
                        if (*door(a)).p_kind != (*door(a)).p_kind_prev {
                            if (*ext::gmMainLib_GetGameRules()).handicap == 2 {
                                door_frame = 30.0f32;
                            } else {
                                door_frame = 90.0f32;
                            }
                            {
                                let anim_jobj = sp90;
                                req_anim(anim_jobj, ALL_TYPE_MASK, door_frame);
                            }
                        }
                        {
                            let hval: i32;
                            let port = arg0 as i32;
                            if (*ext::gmMainLib_GetGameRules()).handicap == 1 {
                                hval = (ext::gm_801685D4(port as u8, (*player_at(mnCharSel_804D6CB0, port as usize)).nametag) as u8) as i32;
                            } else {
                                hval = ((*player_at(mnCharSel_804D6CB0, port as usize)).handicap as u8) as i32;
                            }
                            anim_frame = (if hval != 0 { hval } else { 1 }) as f32;
                        }
                        joint = (*door(a)).cpuslider_joint;
                        sp90 = animateJoint(mnCharSel_804D6CC0, joint, TOBJ_MASK, anim_frame);
                        anim_frame = 1.25f32 * (getHandicapValue(arg0 as i32) - 1) as f32;
                        super::hsd::HSD_JObjSetTranslateX(sp90, anim_frame);

                        anim_frame = (*player_at(mnCharSel_804D6CB0, a)).cpu_level as f32;
                        joint = (*door(a)).cpuslider2_joint;
                        animateJoint(mnCharSel_804D6CC0, joint, TOBJ_MASK, anim_frame);
                    } else {
                        if (*door(a)).p_kind != (*door(a)).p_kind_prev {
                            {
                                let anim_jobj = sp90;
                                req_anim(anim_jobj, ALL_TYPE_MASK, 50.0f32);
                            }
                        }

                        anim_frame = (*player_at(mnCharSel_804D6CB0, a)).cpu_level as f32;
                        joint = (*door(a)).cpuslider_joint;
                        animateJoint(mnCharSel_804D6CC0, joint, TOBJ_MASK, anim_frame);
                    }
                }
                _ => {}
            }
            (*door(a)).slideranim_timer = 0xA;
        }

        // Update p_kind_prev
        (*door(a)).p_kind_prev = (*door(a)).p_kind;

        // Player indicator / status icon
        anim_frame = mnCharSel_804D50CC[(*door(a)).p_kind as usize] as f32;
        joint = (*door(a)).player_indicator_joint;
        animateJoint(mnCharSel_804D6CC0, joint, ALL_TYPE_MASK, anim_frame);

        // Team / color
        lb_find(mnCharSel_804D6CC0, &mut sp90, (*door(a)).team_joint as i32);
        if (*mnCharSel_804D6CB0).vs.start.rules.is_teams == 0 {
            // FFA mode
            ext::HSD_JObjSetFlags(sp90, JOBJ_HIDDEN as u32);
            let mut port_color_idx = arg0 as i32;
            if (*door(a)).p_kind != 0 {
                port_color_idx += 4;
            }

            anim_frame = mnCharSel_804D50D8[port_color_idx as usize] as f32;
            joint = (*door(a)).bg_joint;
            animateJoint(mnCharSel_804D6CC0, joint, TOBJ_MASK, anim_frame);

            anim_frame = mnCharSel_804D50D8[port_color_idx as usize] as f32;
            joint = (*door(a)).emblem_joint;
            animateJoint(mnCharSel_804D6CC0, joint, MOBJ_MASK, anim_frame);
        } else {
            // Teams mode
            ext::HSD_JObjClearFlags(sp90, JOBJ_HIDDEN as u32);
            let mut team_v: i32 = (*door(a)).team as i32;

            anim_frame = mnCharSel_804D50D0[team_v as usize] as f32;
            joint = (*door(a)).team_joint;
            animateJoint(mnCharSel_804D6CC0, joint, TOBJ_MASK, anim_frame);
            if (*door(a)).p_kind != 0 {
                team_v += 4;
            }
            {
                anim_frame = mnCharSel_804D50D0[team_v as usize] as f32;
                joint = (*door(a)).bg_joint;
                animateJoint(mnCharSel_804D6CC0, joint, TOBJ_MASK, anim_frame);

                anim_frame = mnCharSel_804D50D0[team_v as usize] as f32;
                joint = (*door(a)).emblem_joint;
                animateJointLeadingPad(mnCharSel_804D6CC0, joint, MOBJ_MASK, anim_frame);
            }
            (*player_at(mnCharSel_804D6CB0, a)).team = (*door(a)).team;
        }

        // Hide/show nametag text
        if (*door(a)).p_kind == 3 || ((*(*tag_at(a)).data).use_tag == 0 && (*door(a)).sel_icon as u32 >= MNCS_NUM_U()) {
            (*(*(*tag_at(a)).data).text).hidden = 1;
        } else {
            (*(*(*tag_at(a)).data).text).hidden = 0;
        }
    }

    // Final: costume color assignment
    {
        let final_icon = (*door(a)).sel_icon;
        if (final_icon as u32) < MNCS_NUM_U() {
            if mnCharSel_804D6CF5 == 1 || (*mnCharSel_804D6CB0).vs.start.rules.is_teams == 0 {
                color = (*door(a)).costume as i32;
            } else {
                let ckind = (*icon_at(final_icon as usize)).char_kind;
                color = match (*door(a)).team as i32 {
                    1 => ext::gm_801692BC(ckind) as i32,
                    2 => ext::gm_80169290(ckind) as i32,
                    _ => ext::gm_80169264(ckind) as i32,
                };
            }
            {
                let port: i32;
                if mnCharSel_804D6CF5 == 1 {
                    if arg0 as i32 != 0 {
                        port = mnCharSel_804D6CF1 as i32;
                    } else {
                        port = mnCharSel_804D6CF0 as i32;
                    }
                } else {
                    port = arg0 as i32;
                }
                (*player_at(mnCharSel_804D6CB0, port as usize)).color = color as u8;
            }
            hud_idx += color * 0x1E;
            if !mnCharSel_Mex.is_null() {
                mnCharSel_MexPortrait(arg0 as i32, final_icon as i32, color, hud_idx);
            } else {
                mnCharSel_8025D5AC(arg0 as i32, hud_idx, 0);
            }
        }
    }
}

/// Plays the mode name frame for `idx` and refreshes every door.
pub unsafe fn mnCharSel_8025EE8C(idx: u8) {
    let mut mode_frame: f32;
    let mut sp14: *mut HSD_JObj = null_mut();
    let mut result_jobj: *mut HSD_JObj = null_mut();
    let mut spC: *mut HSD_JObj = null_mut();
    let mi = (addr_of!(mnCharSel_803F0A48.mode_info) as *const CSSModeInfo).add(idx as usize);

    if mnCharSel_804D6CF5 == 1 {
        mode_frame = (*mi).mode_ffa_frame as f32;
        let cc0 = mnCharSel_804D6CC0;
        lb_find(cc0, &mut sp14, 0x24);
        req_anim(sp14, TOBJ_MASK, mode_frame);

        ext::HSD_JObjAnimAll(sp14);
        stop_anim(sp14, TOBJ_MASK);
        mnCharSel_8025DB34(0);
        if (*mnCharSel_804D6CB0).match_type as u32 == TRAINING_MODE {
            mnCharSel_8025DB34(1);
        }
        return;
    }
    if (*mnCharSel_804D6CB0).vs.start.rules.is_teams != 0 {
        mode_frame = (*mi).mode_teams_frame as f32;
        let cc0 = mnCharSel_804D6CC0;
        lb_find(cc0, &mut result_jobj, 36);
        req_anim(result_jobj, TOBJ_MASK, mode_frame);
        ext::HSD_JObjAnimAll(result_jobj);
        stop_anim(result_jobj, TOBJ_MASK);
    } else {
        mode_frame = (*mi).mode_ffa_frame as f32;
        let cc0 = mnCharSel_804D6CC0;
        lb_find(cc0, &mut spC, 36);
        req_anim(spC, TOBJ_MASK, mode_frame);
        ext::HSD_JObjAnimAll(spC);
        stop_anim(spC, TOBJ_MASK);
    }
    for i in 0..mnCharSel_804D6CF5 as i32 {
        mnCharSel_8025DB34(i as u8);
    }
}

/// `updateStockIcons`.
pub unsafe fn updateStockIcons(stock_data: *mut CSSDoorsData2, stock_jobj: *mut *mut HSD_JObj, mut i: i32) {
    lb_find(mnCharSel_804D6CC0, stock_jobj, (*stock_data).xf0[0] as i32);
    super::hsd::HSD_JObjSetTranslateX(*stock_jobj, 7.5f32);
    i = 1;
    while i < 5 {
        lb_find(mnCharSel_804D6CC0, stock_jobj, (*stock_data).xf0[i as usize] as i32);
        if ((*stock_data).stocks as i32) <= i {
            ext::HSD_JObjSetFlags(*stock_jobj, JOBJ_HIDDEN as u32);
        } else {
            ext::HSD_JObjClearFlags(*stock_jobj, JOBJ_HIDDEN as u32);
        }
        i += 1;
    }
}


// ------------------------------------------------------------------------------------------
// the CSS scene's per-frame callbacks
// ------------------------------------------------------------------------------------------

/// `HSD_JObjGetDObj(j)->mobj`, NULL-checked as the C does.
#[inline]
unsafe fn dobj_mobj(j: *mut HSD_JObj) -> *mut HSD_MObj {
    let dobj = ext::HSD_JObjGetDObj(j);
    if !dobj.is_null() { (*dobj).mobj } else { null_mut() }
}

/// The CSS background's per-frame update: icon and door animation timers, the handicap sliders,
/// the CPU level scroll and the portrait alpha.
pub unsafe extern "C" fn fn_8025F0E0(gobj: *mut HSD_GObj) {
    let jobj = get_jobj(gobj);
    let mut sp54: *mut HSD_JObj = null_mut();
    let mut sp4C: *mut HSD_JObj = null_mut();
    let mut sp48: *mut HSD_JObj = null_mut();
    let mut sp44: *mut HSD_JObj = null_mut();
    let mut sp40: *mut HSD_JObj = null_mut();
    let mut sp3C: *mut HSD_JObj = null_mut();
    let mut sp38: *mut HSD_JObj = null_mut();
    let mut sp34: *mut HSD_JObj = null_mut();
    let mut sp30: *mut HSD_JObj = null_mut();
    let mut sp2C: *mut HSD_JObj = null_mut();
    let mut i: i32;
    let mut timer: u8;
    let mut val: i32;
    let mut css_models_v: i32;
    let mut fval: f32;

    if mnCharSel_804D6CEC % 10 != 0 {
        if mnCharSel_804D6CF5 == 1 {
            lb_find(jobj, &mut sp54, 0x27);
        } else {
            lb_find(jobj, &mut sp54, 0x27);
        }
        req_anim(sp54, ALL_TYPE_MASK, 0.0f32);
    }

    i = 0;
    while i < MNCS_NUM() {
        let ic = icon_at(i as usize);
        timer = (*ic).anim_timer;
        if timer != 0 {
            timer = timer.wrapping_sub(1);
            (*ic).anim_timer = timer;
            if timer == 0 {
                if mnCharSel_804D6CF5 == 1 {
                    lb_find(MNCS_ICON_ROOT(jobj), &mut sp4C, (*ic).joint_id_1p as i32);
                    req_anim(sp4C, TOBJ_MASK, 0.0f32);
                    ext::HSD_JObjAnimAll(sp4C);
                    stop_anim(sp4C, TOBJ_MASK);
                } else {
                    lb_find(MNCS_ICON_ROOT(jobj), &mut sp48, (*ic).joint_id_vs as i32);
                    req_anim(sp48, TOBJ_MASK, 0.0f32);
                    ext::HSD_JObjAnimAll(sp48);
                    stop_anim(sp48, TOBJ_MASK);
                }
            }
        }
        i += 1;
    }

    if mnCharSel_804D6CF5 == 4 {
        let mut doors = door(0);
        i = 0;
        while i < mnCharSel_804D6CF5 as i32 {
            timer = (*doors).dooranim_timer;
            if timer != 0 {
                timer = timer.wrapping_sub(1);
                (*doors).dooranim_timer = timer;
                if timer == 0 {
                    lb_find(jobj, &mut sp54, (*doors).door_joint as i32);
                    stop_anim(sp54, ALL_TYPE_MASK);
                }
            }

            timer = (*doors).slideranim_timer;
            if timer != 0 {
                timer = timer.wrapping_sub(1);
                (*doors).slideranim_timer = timer;
                if timer == 0 {
                    let rules = ext::gmMainLib_GetGameRules();
                    if (*rules).handicap != 0 {
                        if (*doors).p_kind == 1 {
                            let rules = ext::gmMainLib_GetGameRules();
                            if (*rules).handicap == 1 {
                                css_models_v = 0x5A;
                            } else {
                                css_models_v = 0x1E;
                            }
                            {
                                let slidername_joint = (*doors).slidername_joint;
                                fval = css_models_v as f32;
                                lb_find(mnCharSel_804D6CC0, &mut sp44, slidername_joint as i32);
                            }
                            {
                                let anim_jobj = sp44;
                                req_anim(anim_jobj, ALL_TYPE_MASK, fval);
                            }
                            ext::HSD_JObjAnimAll(sp44);
                            stop_anim(sp44, ALL_TYPE_MASK);
                        } else {
                            let rules = ext::gmMainLib_GetGameRules();
                            let css_models_v2: i32 = if (*rules).handicap == 1 { 0x50 } else { 0x14 };
                            {
                                let slidername_joint = (*doors).slidername_joint;
                                fval = css_models_v2 as f32;
                                lb_find(mnCharSel_804D6CC0, &mut sp40, slidername_joint as i32);
                            }
                            {
                                let anim_jobj = sp40;
                                req_anim(anim_jobj, ALL_TYPE_MASK, fval);
                            }
                            ext::HSD_JObjAnimAll(sp40);
                            stop_anim(sp40, ALL_TYPE_MASK);
                        }

                        let rules = ext::gmMainLib_GetGameRules();
                        if (*rules).handicap == 1 {
                            val = (ext::gm_801685D4(i as u8, (*player_at(mnCharSel_804D6CB0, i as usize)).nametag) as u8) as i32;
                        } else {
                            val = ((*player_at(mnCharSel_804D6CB0, i as usize)).handicap as u8) as i32;
                        }
                        // switch forces MWCC beq/b double-branch (not bne)
                        if val == 0 {
                            val = 1;
                        }
                        {
                            let cpuslider_joint = (*doors).cpuslider_joint;
                            fval = val as f32;
                            lb_find(mnCharSel_804D6CC0, &mut sp3C, cpuslider_joint as i32);
                        }
                        {
                            let anim_jobj = sp3C;
                            req_anim(anim_jobj, TOBJ_MASK, fval);
                        }
                        ext::HSD_JObjAnimAll(sp3C);
                        stop_anim(sp3C, TOBJ_MASK);

                        {
                            let cpuslider2_joint = (*doors).cpuslider2_joint;
                            fval = (*player_at(mnCharSel_804D6CB0, i as usize)).cpu_level as f32;
                            lb_find(mnCharSel_804D6CC0, &mut sp38, cpuslider2_joint as i32);
                        }
                        {
                            let anim_jobj = sp38;
                            req_anim(anim_jobj, TOBJ_MASK, fval);
                        }
                        ext::HSD_JObjAnimAll(sp38);
                        stop_anim(sp38, TOBJ_MASK);
                    } else {
                        if (*doors).p_kind == 1 {
                            let slidername_joint = (*doors).slidername_joint;
                            lb_find(mnCharSel_804D6CC0, &mut sp34, slidername_joint as i32);
                            req_anim(sp34, ALL_TYPE_MASK, 50.0f32);
                            ext::HSD_JObjAnimAll(sp34);
                            stop_anim(sp34, ALL_TYPE_MASK);
                        } else {
                            let slidername_joint = (*doors).slidername_joint;
                            lb_find(mnCharSel_804D6CC0, &mut sp30, slidername_joint as i32);
                            req_anim(sp30, ALL_TYPE_MASK, 40.0f32);
                            ext::HSD_JObjAnimAll(sp30);
                            stop_anim(sp30, ALL_TYPE_MASK);
                        }
                        {
                            let cpuslider_joint = (*doors).cpuslider_joint;
                            fval = (*player_at(mnCharSel_804D6CB0, i as usize)).cpu_level as f32;
                            lb_find(mnCharSel_804D6CC0, &mut sp2C, cpuslider_joint as i32);
                        }
                        {
                            let anim_jobj = sp2C;
                            req_anim(anim_jobj, TOBJ_MASK, fval);
                        }
                        ext::HSD_JObjAnimAll(sp2C);
                        stop_anim(sp2C, TOBJ_MASK);
                    }
                }
            }

            doors = doors.add(1);
            i += 1;
        }
    } else {
        let misc = addr_of_mut!(mnCharSel_803F0EBC);
        if (*misc).scroll_flag != 0 {
            if (*misc).cpu_level != (*misc).cpu_level_shown {
                if (*misc).cpu_level < (*misc).cpu_level_shown {
                    (*misc).cpu_slider_x += 31.0f32;
                    if (*misc).cpu_slider_x >= fmadds(-248.0f32, (*misc).cpu_level as f32, 124.0f32) {
                        (*misc).cpu_level_shown = (*misc).cpu_level;
                    }
                } else {
                    (*misc).cpu_slider_x = (*misc).cpu_slider_x - 31.0f32;
                    if (*misc).cpu_slider_x <= fmadds(-248.0f32, (*misc).cpu_level as f32, 124.0f32) {
                        (*misc).cpu_level_shown = (*misc).cpu_level;
                    }
                }
            }
            if (*misc).cpu_level == (*misc).cpu_level_shown {
                (*misc).scroll_flag = 0;
                (*misc).cpu_slider_x = fmadds(-248.0f32, (*misc).cpu_level as f32, 124.0f32);
            }
            {
                for j in 0..5i32 {
                    ext::HSD_SisLib_803A746C((*misc).xd3, j, fmadds(248.0f32, j as f32, (*misc).cpu_slider_x), 0.0f32);
                }
            }
        }
    }

    if mnCharSel_804D6CF5 == 1 {
        lb_find(mnCharSel_804D6CC0, &mut sp54, 0x2D);
        if (*mnCharSel_804A0BD0[0]).x5 != 0 {
            ext::HSD_MObjSetAlpha(dobj_mobj(sp54), 0.5f32);
        } else {
            ext::HSD_MObjSetAlpha(dobj_mobj(sp54), 1.0f32);
        }
        if (*mnCharSel_804D6CB0).match_type == 0x17 {
            lb_find(mnCharSel_804D6CC4, &mut sp54, 6);
            if (*mnCharSel_804A0BD0[1]).x5 != 0 {
                ext::HSD_MObjSetAlpha(dobj_mobj(sp54), 0.5f32);
            } else {
                ext::HSD_MObjSetAlpha(dobj_mobj(sp54), 1.0f32);
            }
        }
    } else {
        i = 0;
        while i < mnCharSel_804D6CF5 as i32 {
            lb_find(mnCharSel_804D6CC0, &mut sp54, (*door(i as usize)).costume_joint as i32);
            if (*mnCharSel_804A0BD0[i as usize]).x5 != 0 {
                ext::HSD_MObjSetAlpha(dobj_mobj(sp54), 0.5f32);
            } else {
                ext::HSD_MObjSetAlpha(dobj_mobj(sp54), 1.0f32);
            }
            i += 1;
        }
    }
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe extern "C" fn fn_8025FAC0(gobj: *mut HSD_GObj) {
    let jobj = get_jobj(gobj);

    if mnCharSel_804D6CEC % 40 == 0 {
        let child = if jobj.is_null() { null_mut() } else { (*jobj).child };
        ext::HSD_JObjReqAnimAll(child, 0.0f32);
    }
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe extern "C" fn fn_8025FB2C(gobj: *mut HSD_GObj) {
    ext::HSD_JObjAnimAll(get_jobj(gobj));
}

/// `getPlayerForDoor`.
pub unsafe fn getPlayerForDoor(door_idx: u8) -> i32 {
    if mnCharSel_804D6CF5 == 1 {
        if door_idx != 0 {
            return mnCharSel_804D6CF1 as i32;
        }
        return mnCharSel_804D6CF0 as i32;
    }
    door_idx as i32
}

/// Picks a random unlocked icon for door `door_idx` (the Random button).
pub unsafe fn mnCharSel_8025FB50(door_idx: u8, arg1: i32) {
    let mut icon_idx: i32;
    let mut icon_jobj: *mut HSD_JObj = null_mut();

    loop {
        let temp = ext::HSD_Randi(MNCS_NUM());
        icon_idx = temp;
        let _icon_offset = getIconOffset(icon_idx as u32);
        if (*icon_at(icon_idx as usize)).state != 0 {
            break;
        }
    }

    (*player_at(mnCharSel_804D6CB0, getPlayerForDoor(door_idx) as usize)).ckind = (*icon_at(icon_idx as usize)).char_kind as i8;

    (*door(door_idx as usize)).sel_icon = icon_idx as u8;
    if (*door(door_idx as usize)).sel_icon != (*door(door_idx as usize)).sel_icon_prev {
        let mut costume: u8 = 0;
        loop {
            (*door(door_idx as usize)).costume = costume;
            if !isDuplicateCostumeExact(door_idx as i32) {
                break;
            }
            costume = costume.wrapping_add(1);
        }
    }

    let model = mnCharSel_804A0BD0[door_idx as usize];
    (*model).x5 = 0;
    (*model).x8 = 3.4f32 + (*icon_at(icon_idx as usize)).bound_l;
    (*model).xC = -3.0f32 + (*icon_at(icon_idx as usize)).bound_u;
    if arg1 != 0 {
        (*model).x10 = (*model).x8;
        (*model).x14 = (*model).xC;
    }
    ext::HSD_GObjGXLink_803909D8((*model).gobj, (*mnCharSel_804A0BC0[mnCharSel_804D6CF5 as usize - 1]).gobj);

    if mnCharSel_804D6CF5 == 1 {
        lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut icon_jobj, (*icon_at(icon_idx as usize)).joint_id_1p as i32);
    } else {
        lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut icon_jobj, (*icon_at(icon_idx as usize)).joint_id_vs as i32);
    }
    ext::HSD_ForeachAnim(icon_jobj.cast(), JOBJ_TYPE as u32, TOBJ_MASK, ext::HSD_AObjReqAnim as *const () as *mut c_void, AOBJ_ARG_AF, &[Va::D(10.0)]);

    (*icon_at(icon_idx as usize)).anim_timer = 0xC;

    {
        let sel = (*door(door_idx as usize)).sel_icon;
        ext::lbAudioAx_80023870((*icon_at(sel as usize)).sfx, 0x7F, 0x40, sel as i32 + 0x8A);
        ext::gm_80168C5C((*icon_at(sel as usize)).char_kind as u32);
    }
}


// ------------------------------------------------------------------------------------------
// token pick-up, costume change and the cursor
// ------------------------------------------------------------------------------------------

const PAD_A: u32 = HSD_PAD_A as u32;
const PAD_B: u32 = HSD_PAD_B as u32;
const PAD_X: u32 = HSD_PAD_X as u32;
const PAD_Y: u32 = HSD_PAD_Y as u32;
const PAD_L: u32 = HSD_PAD_L as u32;
const PAD_R: u32 = HSD_PAD_R as u32;

/// Puts door `door_idx`'s token back on the icon of the fighter its player holds; returns 0 when it
/// did, 1 when the player's fighter is not a playable one.
pub unsafe fn mnCharSel_8025FDEC(door_idx: u8) -> i32 {
    let mut sp10: *mut HSD_JObj = null_mut();
    let mut icon_idx: i32;

    let player: i32 = if mnCharSel_804D6CF5 == 1 {
        if door_idx != 0 { mnCharSel_804D6CF1 as i32 } else { mnCharSel_804D6CF0 as i32 }
    } else {
        door_idx as i32
    };

    let css = mnCharSel_804D6CB0;
    let c_kind: i8 = (*player_at(css, player as usize)).ckind;

    if (c_kind as i32) < CKind_Playable_Count as i32 {
        if c_kind as i32 != (*icon_at((*door(door_idx as usize)).sel_icon as usize)).char_kind as i32 {
            (*door(door_idx as usize)).costume = 0;
        }

        {
            let icon = icon_at(0);
            icon_idx = 0;
            while icon_idx < MNCS_NUM() {
                if (*player_at(css, player as usize)).ckind as i32 == (*icon.add(icon_idx as usize)).char_kind as i32 {
                    break;
                }
                icon_idx += 1;
            }
        }

        let model = mnCharSel_804A0BD0[door_idx as usize];
        (*model).x8 = 3.4f32 + (*icon_at(icon_idx as usize)).bound_l;
        (*model).xC = -3.0f32 + (*icon_at(icon_idx as usize)).bound_u;

        (*door(door_idx as usize)).sel_icon_prev = icon_idx as u8;
        (*door(door_idx as usize)).sel_icon = icon_idx as u8;

        ext::HSD_GObjGXLink_803909D8((*model).gobj, (*mnCharSel_804A0BC0[mnCharSel_804D6CF5 as usize - 1]).gobj);

        (*model).x5 = 0;

        if mnCharSel_804D6CF5 == 1 {
            lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut sp10, (*icon_at(icon_idx as usize)).joint_id_1p as i32);
        } else {
            lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut sp10, (*icon_at(icon_idx as usize)).joint_id_vs as i32);
        }
        ext::HSD_ForeachAnim(sp10.cast(), JOBJ_TYPE as u32, TOBJ_MASK, ext::HSD_AObjReqAnim as *const () as *mut c_void, AOBJ_ARG_AF, &[Va::D(10.0)]);

        (*icon_at(icon_idx as usize)).anim_timer = 0xC;

        {
            let door_i = door_idx as i32;
            if mnCharSel_8025DAA0(door_i) {
                let selected_door = door(door_i as usize);
                let mut costume: i32 = 0;
                loop {
                    (*selected_door).costume = costume as u8;
                    if !mnCharSel_8025DAA0(door_i) {
                        break;
                    }
                    costume += 1;
                }
            }
        }

        if mnCharSel_804D6CF6 != 3 && mnCharSel_804D6CF6 != 4 {
            let sel = (*door(door_idx as usize)).sel_icon as i32;
            ext::lbAudioAx_80023870((*icon_at(sel as usize)).sfx, 0x7F, 0x40, sel + 0x8A);
            ext::gm_80168C5C((*icon_at(sel as usize)).char_kind as u32);
        }
        return 0;
    }
    1
}

/// X / Y cycle door `door_idx`'s costume (skipping ones another door wears).
pub unsafe fn mnCharSel_CostumeChange(door_idx: i32, input: u32) {
    let d = door(door_idx as usize);
    let prev_costume = (*d).costume;

    if (*d).sel_icon as i32 >= MNCS_NUM() {
        return;
    }
    if (*d).sel_icon_prev as i32 >= MNCS_NUM() {
        return;
    }

    if input & PAD_X != 0 {
        loop {
            (*d).costume = srem((*d).costume as i32 + 1, ext::gm_GetNumCostumesForCKind((*icon_at((*d).sel_icon as usize)).char_kind) as i32) as u8;
            if !isDuplicateCostume(door_idx) {
                break;
            }
        }
    } else if input & PAD_Y != 0 {
        loop {
            if (*d).costume != 0 {
                (*d).costume = (*d).costume.wrapping_sub(1);
            } else {
                (*d).costume = (ext::gm_GetNumCostumesForCKind((*icon_at((*d).sel_icon as usize)).char_kind) as i32 - 1) as u8;
            }
            if !isDuplicateCostume(door_idx) {
                break;
            }
        }
    }
    if prev_costume != (*d).costume {
        mnCharSel_8025DB34(door_idx as u8);
        sfxMove();
    }
}

/// `updateCursorDisplay`: cursor state / colour animation and its position.
pub unsafe fn updateCursorDisplay(jobj: *mut HSD_JObj, cursor: *mut CSSCursorData, state_jobj: *mut *mut HSD_JObj, color_jobj: *mut *mut HSD_JObj) {
    if (*cursor).x5 != 1 {
        let y = (*cursor).x10;
        if y < 0.2f32 || y > 22.0f32 {
            (*cursor).x5 = 0;
        } else {
            (*cursor).x5 = 2;
        }
    }

    {
        let state = (*cursor).x5 as f32;
        lb_find(jobj, state_jobj, 2);
        let cc0 = *state_jobj;
        req_anim(cc0, TOBJ_MASK, state);
        ext::HSD_JObjAnimAll(*state_jobj);
        stop_anim(*state_jobj, TOBJ_MASK);
    }

    {
        let mut color_idx: i32;
        if mnCharSel_804D6CF5 == 1 {
            color_idx = mnCharSel_804D6CF0 as i32;
        } else if (*mnCharSel_804D6CB0).vs.start.rules.is_teams == 0 {
            color_idx = (*cursor).x4 as i32;
        } else {
            color_idx = mnCharSel_804D50E0[(*door((*cursor).x4 as usize)).team as usize] as i32;
        }

        {
            if (*cursor).x8 != 0 {
                let mut port = (*cursor).x4 as i32;
                if mnCharSel_804D6CF5 == 1 {
                    if port != 0 {
                        port = mnCharSel_804D6CF1 as i32;
                    } else {
                        port = mnCharSel_804D6CF0 as i32;
                    }
                }
                color_idx = (mnCharSel_804D6CEC & 3) as i32 + port * 4;
            } else {
                let mut port = (*cursor).x4 as i32;
                if mnCharSel_804D6CF5 == 1 {
                    if port != 0 {
                        port = mnCharSel_804D6CF1 as i32;
                    } else {
                        port = mnCharSel_804D6CF0 as i32;
                    }
                }
                color_idx = color_idx + port * 4;
            }

            lb_find(jobj, color_jobj, 3);
            {
                let anim_jobj = *color_jobj;
                req_anim(anim_jobj, TOBJ_MASK, color_idx as f32);
            }
            ext::HSD_JObjAnimAll(*color_jobj);
            stop_anim(*color_jobj, TOBJ_MASK);
        }
    }

    super::hsd::HSD_JObjSetTranslateX(jobj, (*cursor).xC);
    super::hsd::HSD_JObjSetTranslateY(jobj, (*cursor).x10);
    super::hsd::HSD_JObjSetTranslateZ(jobj, 0.0f32);
    ext::HSD_JObjAnimAll(jobj);
}

/// `updateGrabbedSlider`: drags a CPU-level (`is_handicap` false) or handicap slider along with the
/// cursor; A drops it.
pub unsafe fn updateGrabbedSlider(
    cursor: *mut CSSCursorData,
    door_p: *mut CSSDoor,
    door_idx: i32,
    trigger: u32,
    slider_jobj: *mut *mut HSD_JObj,
    pos: *mut Vec3,
    is_handicap: bool,
) {
    let base_x: f32;
    let anim_frame: f32;

    if is_handicap {
        lb_find(mnCharSel_804D6CC0, slider_jobj, (*door_p).cpuslider_joint as i32);
    } else {
        let rules = ext::gmMainLib_GetGameRules();
        if (*rules).handicap != 0 {
            lb_find(mnCharSel_804D6CC0, slider_jobj, (*door_p).cpuslider2_joint as i32);
        } else {
            lb_find(mnCharSel_804D6CC0, slider_jobj, (*door_p).cpuslider_joint as i32);
        }
    }
    ext::lb_8000B1CC(*slider_jobj, null_mut(), pos);

    base_x = HSD_JObjGetTranslationX(*slider_jobj) - (*pos).x;
    (*pos).x = ((*cursor).xC - -2.9f32) + base_x;
    if (*pos).x < 0.0f32 {
        (*pos).x = 0.0f32;
    }
    if (*pos).x > 10.0f32 {
        (*pos).x = 10.0f32;
    }

    if is_handicap {
        (*player_at(mnCharSel_804D6CB0, door_idx as usize)).handicap = (f2i32(fmadds(0.8f32, (*pos).x, 0.5f32) as f64) + 1) as i8;
        anim_frame = ((*player_at(mnCharSel_804D6CB0, door_idx as usize)).handicap as u8) as f32;
    } else {
        (*player_at(mnCharSel_804D6CB0, door_idx as usize)).cpu_level = (f2i32(fmadds(0.8f32, (*pos).x, 0.5f32) as f64) + 1) as u8;
        anim_frame = (*player_at(mnCharSel_804D6CB0, door_idx as usize)).cpu_level as f32;
    }

    {
        let anim_jobj = *slider_jobj;
        req_anim(anim_jobj, TOBJ_MASK, anim_frame);
    }
    ext::HSD_JObjAnimAll(*slider_jobj);
    stop_anim(*slider_jobj, TOBJ_MASK);
    super::hsd::HSD_JObjSetTranslateX(*slider_jobj, (*pos).x);
    (*cursor).xC = -2.9f32 + ((*pos).x - base_x);
    (*cursor).x10 = 1.7f32 + (*pos).y;

    if trigger & PAD_A != 0 {
        (*cursor).x5 = 2;
        if is_handicap {
            (*door_p).is_hold_handicap_slider = 0;
        } else {
            (*door_p).is_hold_cpu_slider = 0;
        }
        ext::lbAudioAx_800237A8(0xB7, 0x7F, 0x40);
    }
}

/// `cycleTeam`: advances the door's team colour when the cursor clicks its team box.
pub unsafe fn cycleTeam(cursor: *mut CSSCursorData, dp: *mut CSSDoor, di: i32) {
    let cx5 = (*cursor).xC;
    if cx5 > (*dp).teambtn_left && cx5 < (*dp).teambtn_right {
        let cy5 = (*cursor).x10;
        if (cy5 as f64) < -0.9999999046325683f64 && (cy5 as f64) > -5.800000095367432f64 {
            (*cursor).x10 = -3.4f32;
            (*dp).team = (((*dp).team as i32 + 1) % 3) as u8;
            (*player_at(mnCharSel_804D6CB0, di as usize)).team = (*dp).team;
            mnCharSel_8025DB34(di as u8);
            sfxMove();
        }
    }
}

/// The cursor token / hand of one port: reads the port's pad, moves the cursor, and performs the
/// click actions (pick / drop a token, costume change, door toggles, sliders, team box, name tag
/// list, stock and CPU level buttons, start / back).
pub unsafe extern "C" fn mnCharSel_CursorThink(gobj: *mut HSD_GObj) {
    let mut sp98: *mut HSD_JObj = null_mut();
    let mut sp88: Vec3 = core::mem::zeroed();
    let mut sp74: *mut HSD_JObj = null_mut();
    let mut sp54: *mut HSD_JObj = null_mut();
    let mut sp50: *mut HSD_JObj = null_mut();
    let mut cpu_door: i32;
    let mut a_press: u32;
    let mut current_tag: *mut CSSTagData;
    let mut state_jobj: *mut HSD_JObj = null_mut();
    let mut color_jobj: *mut HSD_JObj = null_mut();
    let mut next_port: u32;
    let jobj = get_jobj(gobj);
    let cursor = (*gobj).user_data as *mut CSSCursorData;
    let trigger: u32;
    let n_doors: i32;
    let buttons: u32;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut door_i: i32;

    if (*mnCharSel_804D6CB0).match_type as u32 == TRAINING_MODE {
        n_doors = 2;
    } else {
        n_doors = mnCharSel_804D6CF5 as i32;
    }

    'update_display: {
        if mnCharSel_804D6CF6 == 0 {
            (*cursor).x8 = 0;

            if mnCharSel_804D6CF5 == 1 {
                let port = (mnCharSel_804D6CF0 as u8) as i32;
                trigger = (*pad(port as usize)).trigger;
                buttons = (*pad(port as usize)).button;
                getStickDelta(port, &mut dx, &mut dy);
                if buttons & 0x200 != 0 {
                    if mnCharSel_804D6CF3 & mnCharSel_804D50C8[(*cursor).x4 as usize] != 0 {
                        let new_timer: u16 = (*cursor).xA.wrapping_add(1);
                        (*cursor).xA = new_timer;
                        if new_timer > 0x1E {
                            mnCharSel_804D6CF6 = 2;
                        }
                    } else {
                        (*cursor).xA = 0;
                    }
                } else {
                    mnCharSel_804D6CF3 |= mnCharSel_804D50C8[(*cursor).x4 as usize];
                    (*cursor).xA = 0;
                }
            } else {
                let port = (*cursor).x4 as i32;
                trigger = (*pad(port as usize)).trigger;
                buttons = (*pad(port as usize)).button;
                getStickDelta(port, &mut dx, &mut dy);

                if (*pad((*cursor).x4 as usize)).err != 0 {
                    if (*cursor).x5 != 3 {
                        (*cursor).xC = fmadds(15.0f32, (*cursor).x4 as f32, -31.0f32);
                        (*cursor).x10 = -21.5f32;
                        (*cursor).xA = 0;
                        if (*cursor).x5 == 1 {
                            door_i = (*cursor).x6 as i32;
                            if door_i as u32 <= 3 {
                                (*cursor).x5 = 2;
                                if mnCharSel_8025FDEC((door_i & 0xFF) as u8) != 0 {
                                    mnCharSel_8025FB50((door_i & 0xFF) as u8, 1);
                                }
                            } else if door_i as u32 <= 7 {
                                cpu_door = door_i - 4;
                                (*door(cpu_door as usize)).is_hold_cpu_slider = 0;
                            }
                        } else {
                            if (*door((*cursor).x4 as usize)).sel_icon as u32 >= MNCS_NUM_U()
                                && (*door((*cursor).x4 as usize)).p_kind != 3
                                && mnCharSel_8025FDEC((*cursor).x4) != 0
                            {
                                mnCharSel_8025FB50((*cursor).x4, 1);
                            }
                        }
                        if (*door((*cursor).x4 as usize)).p_kind == 0 {
                            (*door((*cursor).x4 as usize)).p_kind = 1;
                            (*player_at(mnCharSel_804D6CB0, (*cursor).x4 as usize)).slot_type = 1;
                            (*player_at(mnCharSel_804D6CB0, (*cursor).x4 as usize)).nametag = 0x78;
                            (*(*tag_at((*cursor).x4 as usize)).data).use_tag = 0;
                        }
                        mnCharSel_8025DB34((*cursor).x4);
                        if (*(*tag_at((*cursor).x4 as usize)).data).state != 0 {
                            (*(*tag_at((*cursor).x4 as usize)).data).state = 4;
                        }
                        (*door((*cursor).x4 as usize)).is_hold_handicap_slider = 0;
                    }
                    (*cursor).x5 = 3;
                    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
                    return;
                }

                if (*cursor).x5 == 3 {
                    (*cursor).x5 = 2;
                    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
                }

                if buttons & 0x200 != 0 {
                    if mnCharSel_804D6CF3 & mnCharSel_804D50C8[(*cursor).x4 as usize] != 0 {
                        let new_timer: u16 = (*cursor).xA.wrapping_add(1);
                        (*cursor).xA = new_timer;
                        if new_timer > 0x1E {
                            mnCharSel_804D6CF6 = 2;
                        }
                    } else {
                        (*cursor).xA = 0;
                    }
                } else {
                    mnCharSel_804D6CF3 |= mnCharSel_804D50C8[(*cursor).x4 as usize];
                    (*cursor).xA = 0;
                }
            }

            if (*mnCharSel_804D6CB0).match_type as u32 >= REG_CLASSIC && (trigger & (PAD_L | PAD_R)) != 0 {
                gm_CStickSmashTargetTest = (gm_CStickSmashTargetTest == 0) as i32;
                sfxMove();
            }

            (*cursor).xC = fmadds(0.0002f32, dx, (*cursor).xC);
            (*cursor).x10 = fmadds(0.0002f32, dy, (*cursor).x10);

            (*(*tag_at((*cursor).x4 as usize)).data).scroll_amt = 0.0f32;

            {
                current_tag = (*tag_at((*cursor).x4 as usize)).data;
                if (*current_tag).state != 0 {
                    if mnCharSel_804D6CF5 == 1 {
                        lb_find(mnCharSel_804D6CC0, &mut sp98, mnCharSel_803F0EBC.name_list_joint as i32);
                    } else {
                        lb_find(mnCharSel_804D6CC0, &mut sp98, (*tag_at((*cursor).x4 as usize)).list_joint as i32);
                    }
                    ext::lb_8000B1CC(sp98, null_mut(), &mut sp88);

                    if (0.8f32 + sp88.y) < (*cursor).x10 {
                        (*(*tag_at((*cursor).x4 as usize)).data).scroll_amt = (*cursor).x10 - (0.8f32 + sp88.y);
                        (*cursor).x10 = 0.8f32 + sp88.y;
                    }
                    if (-14.8f32 + sp88.y) > (*cursor).x10 {
                        (*(*tag_at((*cursor).x4 as usize)).data).scroll_amt = (*cursor).x10 - (-14.8f32 + sp88.y);
                        (*cursor).x10 = -14.8f32 + sp88.y;
                    }
                    if (4.3f32 + sp88.x) < (*cursor).xC {
                        (*cursor).xC = 4.3f32 + sp88.x;
                    }
                    if (-5.3f32 + sp88.x) > (*cursor).xC {
                        (*cursor).xC = -5.3f32 + sp88.x;
                    }
                } else {
                    if 25.0f32 < (*cursor).x10 {
                        (*cursor).x10 = 25.0f32;
                    }
                    if -22.0f32 > (*cursor).x10 {
                        (*cursor).x10 = -22.0f32;
                    }
                    if 26.0f32 < (*cursor).xC {
                        (*cursor).xC = 26.0f32;
                    }
                    if -35.0f32 > (*cursor).xC {
                        (*cursor).xC = -35.0f32;
                    }
                }
            }

            {
                let port: u8 = (*cursor).x4;
                let tag = (*tag_at(port as usize)).data;
                if (*tag).state == 0 {
                    if (*cursor).x5 == 1 {
                        door_i = (*cursor).x6 as i32;
                        match door_i {
                            0 | 1 | 2 | 3 => 'case: {
                                if (*cursor).x10 < 0.2f32 {
                                    if port as i32 != door_i || (port as i32 == door_i && (*door(door_i as usize)).p_kind == 3) {
                                        mnCharSel_8025FDEC(door_i as u8);
                                        ext::lbAudioAx_800237A8(0xB8, 0x7F, 0x40);
                                    } else {
                                        (*mnCharSel_804A0BD0[door_i as usize]).x5 = 0;
                                        (*door(door_i as usize)).sel_icon = MNCS_NUM() as u8;
                                        {
                                            let player_idx: i32;
                                            if mnCharSel_804D6CF5 == 1 {
                                                if door_i != 0 {
                                                    player_idx = mnCharSel_804D6CF1 as i32;
                                                } else {
                                                    player_idx = mnCharSel_804D6CF0 as i32;
                                                }
                                            } else {
                                                player_idx = door_i;
                                            }
                                            (*player_at(mnCharSel_804D6CB0, player_idx as usize)).ckind = ChKind_None as i8;
                                        }
                                    }
                                    mnCharSel_8025DB34(door_i as u8);
                                    (*cursor).x5 = 2;
                                } else {
                                    a_press = trigger & PAD_A;
                                    if a_press != 0 {
                                        let m = mnCharSel_804A0BD0[door_i as usize];
                                        if (*m).xC < 6.0f32
                                            && (*m).xC > -1.0f32
                                            && (((*m).x8 > -30.0f32 && (*m).x8 < -24.4f32) || ((*m).x8 > 24.4f32 && (*m).x8 < 30.2f32))
                                        {
                                            let mut icon_count: i32 = 0;
                                            while icon_count < MNCS_NUM() {
                                                if (*icon_at(icon_count as usize)).state < 2 {
                                                    break;
                                                }
                                                icon_count += 1;
                                            }
                                            if icon_count == MNCS_NUM() {
                                                mnCharSel_8025FB50(door_i as u8, 0);
                                                loop {
                                                    (*door(door_i as usize)).costume = ext::HSD_Randi(
                                                        ext::gm_GetNumCostumesForCKind((*icon_at((*door(door_i as usize)).sel_icon as usize)).char_kind) as i32,
                                                    ) as u8;
                                                    if !isDuplicateCostume(door_i) {
                                                        break;
                                                    }
                                                }
                                                mnCharSel_8025DB34(door_i as u8);
                                                (*cursor).x5 = 2;
                                                ext::lbAudioAx_800237A8(0xB8, 0x7F, 0x40);
                                                break 'case;
                                            }
                                        }
                                    }

                                    {
                                        let m2 = mnCharSel_804A0BD0[door_i as usize];
                                        let mut cancel_or_costume = false;
                                        let mut found = false;
                                        let mut i: i32 = 0;
                                        while i < MNCS_NUM() {
                                            let ic = icon_at(i as usize);
                                            if (*m2).x8 > (*ic).bound_l
                                                && (*m2).x8 < (*ic).bound_r
                                                && (*m2).xC < (*ic).bound_u
                                                && (*m2).xC > (*ic).bound_d
                                                && (*ic).state >= 1
                                            {
                                                (*door(door_i as usize)).sel_icon = i as u8;
                                                mnCharSel_8025DB34(door_i as u8);
                                                if trigger & PAD_A != 0 {
                                                    let player_idx: i32;
                                                    if mnCharSel_804D6CF5 == 1 {
                                                        if door_i != 0 {
                                                            player_idx = mnCharSel_804D6CF1 as i32;
                                                        } else {
                                                            player_idx = mnCharSel_804D6CF0 as i32;
                                                        }
                                                    } else {
                                                        player_idx = door_i;
                                                    }
                                                    (*player_at(mnCharSel_804D6CB0, player_idx as usize)).ckind =
                                                        (*icon_at((*door(door_i as usize)).sel_icon as usize)).char_kind as i8;
                                                    if mnCharSel_804D6CF5 == 1 {
                                                        lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut sp98, (*ic).joint_id_1p as i32);
                                                    } else {
                                                        lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut sp98, (*ic).joint_id_vs as i32);
                                                    }
                                                    ext::HSD_ForeachAnim(
                                                        sp98.cast(),
                                                        JOBJ_TYPE as u32,
                                                        TOBJ_MASK,
                                                        ext::HSD_AObjReqAnim as *const () as *mut c_void,
                                                        AOBJ_ARG_AF,
                                                        &[Va::D(10.0)],
                                                    );
                                                    (*ic).anim_timer = 0xC;
                                                    (*mnCharSel_804A0BD0[door_i as usize]).x5 = 0;
                                                    ext::HSD_GObjGXLink_803909D8(
                                                        (*mnCharSel_804A0BD0[door_i as usize]).gobj,
                                                        (*mnCharSel_804A0BC0[mnCharSel_804D6CF5 as usize - 1]).gobj,
                                                    );
                                                    (*door(door_i as usize)).selected_since_load = 1;
                                                    (*cursor).x5 = 2;
                                                    {
                                                        let sel = (*door((door_i as u8) as usize)).sel_icon as i32;
                                                        ext::lbAudioAx_80023870((*icon_at(sel as usize)).sfx, 0x7F, 0x40, sel + 0x8A);
                                                        ext::gm_80168C5C((*icon_at(sel as usize)).char_kind as u32);
                                                    }
                                                    ext::lbAudioAx_800237A8(0xB8, 0x7F, 0x40);
                                                    break 'case;
                                                }
                                                cancel_or_costume = true;
                                                found = true;
                                                break;
                                            }
                                            i += 1;
                                        }

                                        if !found {
                                            mnCharSel_8025D5AC(door_i, 0, 1);
                                            {
                                                let td = (*tag_at(door_i as usize)).data;
                                                if (*td).use_tag == 0 {
                                                    (*(*td).text).hidden = 1;
                                                }
                                            }
                                            (*door(door_i as usize)).sel_icon_prev = MNCS_NUM() as u8;
                                            if trigger & PAD_A != 0 {
                                                ext::lbAudioAx_80024030(3);
                                            } else {
                                                cancel_or_costume = true;
                                            }
                                        }
                                        if cancel_or_costume {
                                            if trigger & PAD_B != 0 {
                                                if mnCharSel_8025FDEC(door_i as u8) == 0 {
                                                    mnCharSel_8025DB34(door_i as u8);
                                                    (*cursor).x5 = 2;
                                                }
                                            } else {
                                                mnCharSel_CostumeChange(door_i, trigger);
                                            }
                                        }
                                    }
                                }
                            },
                            4 | 5 | 6 | 7 => {
                                let slider_door = door_i - 4;
                                updateGrabbedSlider(cursor, door(slider_door as usize), slider_door, trigger, &mut sp98, &mut sp88, false);
                            }
                            8 | 9 | 0xA | 0xB => {
                                let hc_door = door_i - 8;
                                updateGrabbedSlider(cursor, door(hc_door as usize), hc_door, trigger, &mut sp98, &mut sp88, true);
                            }
                            _ => {}
                        }
                        break 'update_display;
                    }

                    if (*cursor).xC > 17.3f32 && (*cursor).x10 > 22.0f32 {
                        (*cursor).x8 = 1;
                        if trigger & PAD_A != 0 {
                            mnCharSel_804D6CF6 = 2;
                            break 'update_display;
                        }
                    }

                    // `door_clicks` is reached by falling out of `try_costume_change`, by the jumps of the 1P
                    // block, and never by the team-toggle press.
                    let mut do_try_costume_change = false;
                    let mut do_door_clicks = false;

                    if mnCharSel_804D6CF5 == 1 {
                        if (*mnCharSel_804A0BD0[(*cursor).x4 as usize]).x5 == 0 {
                            mnCharSel_CostumeChange(0, trigger);
                        }

                        let mut cpu_level_widget = false;
                        if ((*mnCharSel_804D6CB0).match_type as u32) < 0xD {
                            let cy = (*cursor).x10;
                            if cy < data2.x108 && cy > data2.x10c {
                                let cx = (*cursor).xC;
                                if cx > data2.xf8 && cx < data2.xfc {
                                    (*cursor).x8 = 1;
                                    if (trigger & PAD_A) != 0 && data2.stocks > 1 {
                                        data2.stocks = data2.stocks.wrapping_sub(1);
                                        updateStockIcons(addr_of_mut!(data2), &mut sp54, 1);
                                        (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF0 as usize)).stocks = data2.stocks as i8;
                                        sfxMove();
                                    } else {
                                        cpu_level_widget = true;
                                    }
                                } else if cx > data2.x100 && cx < data2.x104 {
                                    (*cursor).x8 = 1;
                                    if (trigger & PAD_A) != 0 && data2.stocks < 5 {
                                        data2.stocks = data2.stocks.wrapping_add(1);
                                        updateStockIcons(addr_of_mut!(data2), &mut sp50, 1);
                                        (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF0 as usize)).stocks = data2.stocks as i8;
                                        sfxMove();
                                    } else {
                                        cpu_level_widget = true;
                                    }
                                } else {
                                    cpu_level_widget = true;
                                }
                            } else {
                                cpu_level_widget = true;
                            }
                        } else {
                            cpu_level_widget = true;
                        }
                        if cpu_level_widget {
                            let misc = addr_of_mut!(mnCharSel_803F0EBC);
                            if (*mnCharSel_804D6CB0).match_type as u32 <= 0xD {
                                let cy2 = (*cursor).x10;
                                if cy2 < (*misc).cpubtn_top && cy2 > (*misc).cpubtn_btm {
                                    let cx2 = (*cursor).xC;
                                    if cx2 > (*misc).cpudown_left && cx2 < (*misc).cpudown_right {
                                        (*cursor).x8 = 1;
                                        if trigger & PAD_A != 0 {
                                            let level = addr_of_mut!((*misc).cpu_level);
                                            let current_level = *level;
                                            if current_level != 0 {
                                                *level = current_level - 1;
                                                (*misc).scroll_flag = 1;
                                                (*player_at(mnCharSel_804D6CB0, ((mnCharSel_804D6CF0 as u8) as i8) as usize)).cpu_level = *level;
                                                sfxMove();
                                            } else {
                                                do_door_clicks = true;
                                            }
                                        } else {
                                            do_door_clicks = true;
                                        }
                                    } else if cx2 > (*misc).cpuup_left && cx2 < (*misc).cpuup_right {
                                        (*cursor).x8 = 1;
                                        if trigger & PAD_A != 0 {
                                            let level = addr_of_mut!((*misc).cpu_level);
                                            let current_level = *level;
                                            if current_level < 4 {
                                                *level = current_level + 1;
                                                (*misc).scroll_flag = 1;
                                                (*player_at(mnCharSel_804D6CB0, ((mnCharSel_804D6CF0 as u8) as i8) as usize)).cpu_level = *level;
                                                sfxMove();
                                            } else {
                                                do_door_clicks = true;
                                            }
                                        } else {
                                            do_door_clicks = true;
                                        }
                                    } else {
                                        do_door_clicks = true;
                                    }
                                } else {
                                    do_door_clicks = true;
                                }
                            } else {
                                do_door_clicks = true;
                            }
                        }
                        if !do_door_clicks {
                            break 'update_display;
                        }
                    } else {
                        if (*mnCharSel_804D6CB0).match_type != 2 {
                            let cx3 = (*cursor).xC;
                            if cx3 > -17.0f32 && cx3 < 15.0f32 && (*cursor).x10 > 22.0f32 {
                                (*cursor).x8 = 1;
                                if trigger & PAD_A != 0 {
                                    mnCharSel_804D6CF6 = 3;
                                    let mut loop_i: i32 = 0;
                                    while loop_i < mnCharSel_804D6CF5 as i32 {
                                        if (*mnCharSel_804A0BC0[loop_i as usize]).x5 == 1 && mnCharSel_8025FDEC(loop_i as u8) == 0 {
                                            mnCharSel_8025DB34(loop_i as u8);
                                            (*mnCharSel_804A0BC0[loop_i as usize]).x5 = 2;
                                        }
                                        loop_i += 1;
                                    }
                                    return;
                                }
                            }
                        }

                        if (*cursor).xC < -25.5f32 && (*cursor).x10 > 22.0f32 {
                            match (*mnCharSel_804D6CB0).match_type as u32 {
                                VS_MELEE | VS_CAMERA | VS_STAMINA | VS_SUDDEN_DEATH | VS_GIANT | VS_TINY | VS_INVISIBLE | VS_FIXED_CAMERA | VS_SINGLE_BUTTON
                                | VS_LIGHTNING | VS_SLOWMO => {
                                    (*cursor).x8 = 1;
                                    if trigger & PAD_A != 0 {
                                        sfxMove();
                                        {
                                            let is_teams = addr_of_mut!((*mnCharSel_804D6CB0).vs.start.rules.is_teams);
                                            *is_teams = (*is_teams + 1) & 1;
                                        }
                                        if (*mnCharSel_804D6CB0).vs.start.rules.is_teams == 0 {
                                            let mut k: i32 = 0;
                                            while k < mnCharSel_804D6CF5 as i32 {
                                                if isDuplicateCostumeCached(k) {
                                                    (*door(k as usize)).costume = 0;
                                                    let css_v = loadCSSValue(addr_of!(mnCharSel_804D6CB0));
                                                    loop {
                                                        if !isDuplicateCostumeWith(k, css_v, mnCharSel_804D6CF5) {
                                                            break;
                                                        }
                                                        (*door(k as usize)).costume = (*door(k as usize)).costume.wrapping_add(1);
                                                    }
                                                }
                                                k += 1;
                                            }
                                        }
                                        mnCharSel_8025EE8C((*mnCharSel_804D6CB0).match_type);
                                    } else {
                                        do_try_costume_change = true;
                                    }
                                }
                                _ => {
                                    do_try_costume_change = true;
                                }
                            }
                        } else {
                            do_try_costume_change = true;
                        }
                        if do_try_costume_change {
                            do_door_clicks = true;
                        }
                    }

                    if do_try_costume_change {
                        let cport2: u8 = (*cursor).x4;
                        if (*mnCharSel_804A0BD0[cport2 as usize]).x5 == 0
                            && (*mnCharSel_804D6CB0).vs.start.rules.is_teams == 0
                            && (*door(cport2 as usize)).p_kind != 3
                        {
                            mnCharSel_CostumeChange(cport2 as i32, trigger);
                        }
                    }
                    if do_door_clicks {
                        // door_clicks:
                        let a_press2 = trigger & PAD_A;
                        if a_press2 != 0 {
                            if mnCharSel_804D6CF5 != 1 {
                                door_i = 0;
                                while door_i < mnCharSel_804D6CF5 as i32 {
                                    let d = door_i as usize;

                                    if door_i == 3 && (*mnCharSel_804D6CB0).match_type == 1 {
                                        door_i += 1;
                                        continue;
                                    }
                                    if ((*door(d)).is_hold_cpu_slider | (*door(d)).is_hold_handicap_slider) == 0
                                        && (*mnCharSel_804A0BD0[d]).x5 == 0
                                        && (*mnCharSel_804A0BC0[d]).x5 != 1
                                    {
                                        let tag_data = (*tag_at(d)).data;
                                        if (*tag_data).state != 0 {
                                            door_i += 1;
                                            continue;
                                        }
                                        let cx4 = (*cursor).xC;
                                        if cx4 > (*door(d)).togglebtn_left && cx4 < (*door(d)).togglebtn_right {
                                            let cy4 = (*cursor).x10;
                                            // Retail nudges both toggle-box bounds outward by 1/10485760.
                                            if (cy4 as f64) < 0.20000009536743146f64 && (cy4 as f64) > -4.600000095367432f64 {
                                                (*cursor).x10 = -2.2f32;
                                                {
                                                    let new_kind: u8 = (*door(d)).p_kind.wrapping_add(1);
                                                    (*door(d)).p_kind = new_kind;
                                                    match new_kind as i32 {
                                                        3 => {}
                                                        2 => {
                                                            (*door(d)).p_kind = 3;
                                                        }
                                                        4 => {
                                                            if (*pad((door_i as u8) as usize)).err != 0 {
                                                                (*door(d)).p_kind = 1;
                                                            } else {
                                                                (*door(d)).p_kind = 0;
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                (*player_at(mnCharSel_804D6CB0, d)).slot_type = (*door(d)).p_kind;
                                                if (*door(d)).p_kind == 1 {
                                                    (*player_at(mnCharSel_804D6CB0, d)).nametag = 0x78;
                                                    (*(*tag_at(d)).data).use_tag = 0;
                                                    if (*door(d)).selected_since_load == 0 && (*cursor).x4 as i32 != door_i {
                                                        mnCharSel_8025FB50(door_i as u8, 1);
                                                    }
                                                }
                                                mnCharSel_8025DB34(door_i as u8);
                                                sfxMove();
                                                break;
                                            }
                                        }
                                    }

                                    if (*mnCharSel_804D6CB0).vs.start.rules.is_teams == 1 && (*door(d)).p_kind != 3 {
                                        cycleTeam(cursor, door(d), door_i);
                                    }
                                    if a_press2 != 0 {
                                        if (*door(d)).is_hold_cpu_slider == 0 && (*door(d)).p_kind == 1 {
                                            let rules2 = ext::gmMainLib_GetGameRules();
                                            if (*rules2).handicap != 0 {
                                                lb_find(mnCharSel_804D6CC0, &mut sp98, (*door(d)).cpuslider2_joint as i32);
                                            } else {
                                                lb_find(mnCharSel_804D6CC0, &mut sp98, (*door(d)).cpuslider_joint as i32);
                                            }
                                            ext::lb_8000B1CC(sp98, null_mut(), &mut sp88);
                                            {
                                                let dx = (*cursor).xC - (-2.9f32 + sp88.x);
                                                let dy = (*cursor).x10 - (1.7f32 + sp88.y);
                                                if fmadds(dx, dx, dy * dy) < 5.0f32 {
                                                    (*cursor).x5 = 1;
                                                    (*cursor).x6 = (door_i + 4) as u8;
                                                    (*cursor).xC = -2.9f32 + sp88.x;
                                                    (*cursor).x10 = 1.7f32 + sp88.y;
                                                    (*door(d)).is_hold_cpu_slider = 1;
                                                    ext::lbAudioAx_800237A8(0xB7, 0x7F, 0x40);
                                                }
                                            }
                                        }

                                        if (*door(d)).is_hold_handicap_slider == 0 {
                                            let rules3 = ext::gmMainLib_GetGameRules();
                                            if (*rules3).handicap == 2 {
                                                let pk = (*door(d)).p_kind;
                                                if pk != 3 && (pk == 1 || door_i == (*cursor).x4 as i32) {
                                                    lb_find(mnCharSel_804D6CC0, &mut sp98, (*door(d)).cpuslider_joint as i32);
                                                    ext::lb_8000B1CC(sp98, null_mut(), &mut sp88);
                                                    {
                                                        let hdx = (*cursor).xC - (-2.9f32 + sp88.x);
                                                        let hdy = (*cursor).x10 - (1.7f32 + sp88.y);
                                                        if fmadds(hdx, hdx, hdy * hdy) < 5.0f32 {
                                                            (*cursor).x5 = 1;
                                                            (*cursor).x6 = (door_i + 8) as u8;
                                                            (*cursor).xC = -2.9f32 + sp88.x;
                                                            (*cursor).x10 = 1.7f32 + sp88.y;
                                                            (*door(d)).is_hold_handicap_slider = 1;
                                                            ext::lbAudioAx_800237A8(0xB7, 0x7F, 0x40);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    door_i += 1;
                                }
                            }

                            {
                                if ((*cursor).x4 != 3 || (*mnCharSel_804D6CB0).match_type != 1) && (*door((*cursor).x4 as usize)).p_kind == 0 {
                                    if mnCharSel_804D6CF5 == 1 {
                                        lb_find(mnCharSel_804D6CC0, &mut sp98, mnCharSel_803F0EBC.tag_box_joint as i32);
                                    } else {
                                        lb_find(mnCharSel_804D6CC0, &mut sp98, (*tag_at(core::ptr::read_volatile(addr_of!((*cursor).x4)) as usize)).name_jointl as i32);
                                    }
                                    ext::lb_8000B1CC(sp98, null_mut(), &mut sp88);
                                    {
                                        if (*cursor).xC > (sp88.x - 4.7f32) && (*cursor).xC < (5.2f32 + sp88.x) {
                                            if (*cursor).x10 < (2.0f32 + sp88.y) && (*cursor).x10 > (sp88.y - 1.0f32) {
                                                if mnCharSel_804D6CF5 == 1 {
                                                    lb_find(mnCharSel_804D6CC0, &mut sp74, mnCharSel_803F0EBC.name_list_joint as i32);
                                                } else {
                                                    lb_find(mnCharSel_804D6CC0, &mut sp74, (*tag_at((*cursor).x4 as usize)).list_joint as i32);
                                                }
                                                ext::lb_8000B1CC(sp74, null_mut(), &mut sp88);
                                                (*cursor).xC = sp88.x;
                                                (*cursor).x10 = sp88.y - 5.0f32;
                                                (*(*tag_at((*cursor).x4 as usize)).data).state = 1;
                                            }
                                        }
                                    }
                                }
                            }

                            {
                                let mut closest: i32 = -1;
                                let mut closest_dist: f32 = 9.0f32;
                                let mut ci: i32 = 0;
                                while ci < n_doors {
                                    let cy7 = (*cursor).x10;
                                    if !(cy7 < 0.2f32) && !(cy7 > 22.0f32) {
                                        let pk2 = (*door(ci as usize)).p_kind;
                                        if pk2 != 3
                                            && ((*door(ci as usize)).sel_icon as u32) < MNCS_NUM_U()
                                            && (pk2 != 0 || (*cursor).x4 as i32 == ci)
                                        {
                                            let mc = mnCharSel_804A0BD0[ci as usize];
                                            if (*mc).x5 == 0 {
                                                let ddx = 3.8f32 + ((*cursor).xC - (*mc).x8);
                                                let ddy = -2.6f32 + (cy7 - (*mc).xC);
                                                let dist = fmadds(ddx, ddx, ddy * ddy);
                                                if dist < closest_dist {
                                                    closest_dist = dist;
                                                    closest = ci;
                                                }
                                            }
                                        }
                                    }
                                    ci += 1;
                                }

                                if closest >= 0 {
                                    let cport4: u8 = (*cursor).x4;
                                    let closest_door = (closest as u8) as i32;
                                    (*mnCharSel_804A0BD0[(closest as u8) as usize]).x5 = (cport4 as i32 + 1) as u8;
                                    (*mnCharSel_804A0BC0[cport4 as usize]).x5 = 1;
                                    (*mnCharSel_804A0BC0[cport4 as usize]).x6 = closest_door as u8;
                                    (*door(closest_door as usize)).sel_icon = 0xD;
                                    ext::HSD_GObjGXLink_803909D8((*mnCharSel_804A0BD0[(closest as u8) as usize]).gobj, (*mnCharSel_804A0BC0[cport4 as usize]).gobj);
                                    ext::lbAudioAx_800237A8(0xB7, 0x7F, 0x40);
                                    (*cursor).xC = (*mnCharSel_804A0BD0[closest as usize]).x8 - 2.7f32;
                                    (*cursor).x10 = (*mnCharSel_804A0BD0[closest as usize]).xC - -2.0f32;
                                }
                            }
                        } else if trigger & PAD_B != 0 {
                            let cport5: u8 = (*cursor).x4;
                            if (*door((*cursor).x4 as usize)).p_kind != 3 && ((*door((*cursor).x4 as usize)).sel_icon as u32) < MNCS_NUM_U() {
                                let cy8 = (*cursor).x10;
                                if cy8 > 0.2f32 && cy8 < 22.0f32 {
                                    if (*mnCharSel_804A0BD0[(*cursor).x4 as usize]).x5 == 0 {
                                        (*mnCharSel_804A0BD0[cport5 as usize]).x5 = (cport5 as i32 + 1) as u8;
                                        (*mnCharSel_804A0BC0[cport5 as usize]).x5 = 1;
                                        (*mnCharSel_804A0BC0[cport5 as usize]).x6 = cport5;
                                        (*door(cport5 as usize)).sel_icon = 0xD;
                                        ext::HSD_GObjGXLink_803909D8((*mnCharSel_804A0BD0[cport5 as usize]).gobj, (*mnCharSel_804A0BC0[cport5 as usize]).gobj);
                                        ext::lbAudioAx_800237A8(0xB7, 0x7F, 0x40);
                                    }
                                }
                            }
                        }

                        {
                            let cy9 = (*cursor).x10;
                            if cy9 > 0.2f32 && cy9 < 22.0f32 {
                                let cport6: u8 = (*cursor).x4;
                                if cport6 != 3 || (*mnCharSel_804D6CB0).match_type != 1 {
                                    if (*door(cport6 as usize)).p_kind == 3 {
                                        (*door(cport6 as usize)).p_kind = 0;
                                        (*player_at(mnCharSel_804D6CB0, (*cursor).x4 as usize)).slot_type = 0;
                                        mnCharSel_8025DB34((*cursor).x4);
                                        sfxMove();
                                    }

                                    {
                                        let cport7: u8 = (*cursor).x4;
                                        if (*door(cport7 as usize)).sel_icon as u32 >= MNCS_NUM_U() {
                                            next_port = cport7 as u32 + 1;
                                            (*mnCharSel_804A0BD0[cport7 as usize]).x5 = next_port as u8;
                                            (*mnCharSel_804A0BC0[cport7 as usize]).x5 = 1;
                                            (*mnCharSel_804A0BC0[cport7 as usize]).x6 = cport7;
                                            (*door(cport7 as usize)).sel_icon = 0xD;
                                            ext::HSD_GObjGXLink_803909D8((*mnCharSel_804A0BD0[cport7 as usize]).gobj, (*mnCharSel_804A0BC0[cport7 as usize]).gobj);
                                            ext::lbAudioAx_800237A8(0xB7, 0x7F, 0x40);
                                            {
                                                let val_x = 2.7f32 + (*cursor).xC;
                                                (*mnCharSel_804A0BD0[(*cursor).x4 as usize]).x10 = val_x;
                                                (*mnCharSel_804A0BD0[(*cursor).x4 as usize]).x8 = val_x;
                                            }
                                            {
                                                let val_y = -2.0f32 + (*cursor).x10;
                                                (*mnCharSel_804A0BD0[(*cursor).x4 as usize]).x14 = val_y;
                                                (*mnCharSel_804A0BD0[(*cursor).x4 as usize]).xC = val_y;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    updateCursorDisplay(jobj, cursor, &mut state_jobj, &mut color_jobj);
}


// ------------------------------------------------------------------------------------------
// token models, start button, name tag list
// ------------------------------------------------------------------------------------------

/// `getDoorCount`.
pub unsafe fn getDoorCount(css: *mut CSSData) -> i32 {
    if (*css).match_type as u32 == TRAINING_MODE {
        return 2;
    }
    mnCharSel_804D6CF5 as i32
}

/// `animateCharModel`: freeze the token model's joint 4 on `frame`.
pub unsafe fn animateCharModel(jobj: *mut HSD_JObj, frame: f32) {
    let mut child: *mut HSD_JObj = null_mut();

    lb_find(jobj, &mut child, 4);
    {
        let anim_jobj = child;
        req_anim(anim_jobj, TOBJ_MASK, frame);
    }
    ext::HSD_JObjAnimAll(child);
    stop_anim(child, TOBJ_MASK);
}

/// `atanf(dx / dy)` based heading used by the token physics: `(f32) ((dy < 0 ? 0 : pi) + atanf(dx/dy))`,
/// or +-pi/2 when `dy` is within 0.01 of zero.
#[inline]
unsafe fn heading(dx: f32, dy: f32) -> f32 {
    if dy < 0.01f32 && dy > -0.01f32 {
        (if dx < 0.0f32 { 1.5707963267948966f64 } else { -1.5707963267948966f64 }) as f32
    } else {
        ((if dy < 0.0f32 { 0.0f64 } else { 3.141592653589793f64 }) + atanf(dx / dy) as f64) as f32
    }
}

/// The per-frame callback of a fighter token (hand-held icon model): chooses its colour, keeps it
/// away from the other tokens, steers it onto its icon and eases the displayed position towards it.
pub unsafe extern "C" fn fn_80262648(gobj: *mut HSD_GObj) {
    let mut sp24: *mut HSD_JObj = null_mut();
    let model = (*gobj).user_data as *mut CSSCharModel;
    let jobj = get_jobj(gobj);
    let prev_port: u8 = (*model).x6;
    let css = mnCharSel_804D6CB0;
    let n_doors = getDoorCount(css);

    {
        let p_kind: u8;
        let door_i: u8 = (*model).x4;

        p_kind = (*door(door_i as usize)).p_kind;
        if p_kind == 3 || (*door(door_i as usize)).sel_icon as u32 >= MNCS_NUM_U() {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            return;
        }

        if mnCharSel_804D6CF5 == 1 {
            if door_i != 0 {
                (*model).x6 = 8;
            } else {
                (*model).x6 = mnCharSel_804D6CF0 as u8;
            }
        } else if (*css).vs.start.rules.is_teams == 0 {
            if p_kind == 1 {
                (*model).x6 = 8;
            } else {
                (*model).x6 = door_i;
            }
        } else if p_kind == 1 {
            (*model).x6 = mnCharSel_804D50E0[(*door(door_i as usize)).team as usize].wrapping_add(4);
        } else {
            (*model).x6 = mnCharSel_804D50E0[(*door(door_i as usize)).team as usize];
        }
    }

    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*model).x7 = (*model).x7.wrapping_add(1);

    {
        let port: u8 = (*model).x6;
        if prev_port != port || (*model).x7 as u32 > 0x27 {
            if (port as u32) < 4 {
                if mnCharSel_804D6CF5 == 1 {
                    animateCharModel(jobj, (mnCharSel_804D6CF0 as i32 * 4) as f32);
                } else {
                    animateCharModel(jobj, ((*model).x4 as i32 * 4) as f32);
                }
            } else {
                animateCharModel(jobj, 16.0f32);
            }
            lb_find(jobj, &mut sp24, 3);
            {
                let anim_jobj = sp24;
                req_anim(anim_jobj, MOBJ_MASK, ((*model).x6 as i32 * 0x28) as f32);
            }
            (*model).x7 = 0;
        }
    }

    {
        let status: u8 = (*model).x5;
        if status == 0 {
            let mut iter: i32 = 0;
            while iter < 0x14 {
                let mut bdp = addr_of_mut!(mnCharSel_804A0BD0) as *mut *mut CSSCharModel;
                let mut dp = door(0);
                let mut j: i32 = 0;

                while j < n_doors {
                    if j != (*model).x4 as i32 && (**bdp).x5 == 0 && (*dp).p_kind != 3 && (*dp).sel_icon as u32 >= MNCS_NUM_U() {
                        let mut dx: f32;
                        let mut dy: f32;
                        let angle: f32;
                        loop {
                            dx = fmadds(0.1f32, ext::HSD_Randf() - 0.5f32, (*model).x8 - (**bdp).x8);
                            if !(dx == 0.0f32) {
                                break;
                            }
                        }

                        loop {
                            let rand = ext::HSD_Randf();
                            dy = fmadds(0.1f32, rand - 0.5f32, (*model).xC - (**bdp).xC);
                            if !(dy == 0.0f32) {
                                break;
                            }
                        }

                        if fmadds(dx, dx, dy * dy) < 8.0f32 {
                            angle = heading(dx, dy);
                            (*model).x8 = -(fmadds(0.01f32, sinf(angle), -(*model).x8));
                            (*model).xC = -(fmadds(0.01f32, cosf(angle), -(*model).xC));
                        }
                    }
                    bdp = bdp.add(1);
                    dp = dp.add(1);
                    j += 1;
                }

                {
                    let sel = (*door((*model).x4 as usize)).sel_icon as usize;
                    let ic = icon_at(sel);
                    let mut x: f32;

                    x = (*model).x8;
                    if x < 1.5f32 + (*ic).bound_l {
                        (*model).x8 += 0.02f32;
                        if (*model).x8 < (*ic).bound_l {
                            (*model).x8 = (*ic).bound_l;
                        }
                    }

                    x = (*model).x8;
                    if x > (*ic).bound_r - 1.5f32 {
                        (*model).x8 = x - 0.02f32;
                        if (*model).x8 > (*ic).bound_r {
                            (*model).x8 = (*ic).bound_r;
                        }
                    }

                    x = (*model).xC;
                    if x > (*ic).bound_u - 1.5f32 {
                        (*model).xC = x - 0.02f32;
                        if (*model).xC > (*ic).bound_u {
                            (*model).xC = (*ic).bound_u;
                        }
                    }

                    x = (*model).xC;
                    if x < 1.5f32 + (*ic).bound_d {
                        (*model).xC += 0.02f32;
                        if (*model).xC < (*ic).bound_d {
                            (*model).xC = (*ic).bound_d;
                        }
                    }
                }
                iter += 1;
            }
        } else {
            (*model).x8 = 2.7f32 + (*mnCharSel_804A0BC0[status as usize - 1]).xC;
            (*model).xC = -2.0f32 + (*mnCharSel_804A0BC0[(*model).x5 as usize - 1]).x10;
        }
    }

    {
        let dx: f32;
        let tx: f32;
        let dy: f32;

        tx = (*model).x8;
        dx = (*model).x10 - tx;
        dy = (*model).x14 - (*model).xC;

        if fmadds(dx, dx, dy * dy) < 4.0f32 {
            (*model).x10 = tx;
            (*model).x14 = (*model).xC;
        } else {
            let angle = heading(dx, dy);
            (*model).x10 = fmadds(3.0f32, sinf(angle), (*model).x10);
            (*model).x14 = fmadds(3.0f32, cosf(angle), (*model).x14);
        }
    }

    super::hsd::HSD_JObjSetTranslateX(jobj, (*model).x10);
    super::hsd::HSD_JObjSetTranslateY(jobj, (*model).x14);
    super::hsd::HSD_JObjSetTranslateZ(jobj, 1.0f32);
    ext::HSD_JObjAnimAll(jobj);
}

/// The "Start" banner's per-frame callback: shows it when every open door has picked a fighter (and
/// the teams are valid), and handles the Start button.
pub unsafe extern "C" fn fn_80262F44(gobj: *mut HSD_GObj) {
    let jobj = get_jobj(gobj);
    let mut i: i32;
    let mut valid_count: i32 = 0;
    let mut hide = false;

    if (*mnCharSel_804D6CB0).match_type as u32 == VS_CAMERA {
        if (*pad(3)).err != 0 {
            ext::HSD_JObjClearFlagsAll(if mnCharSel_804D6CCC.is_null() { null_mut() } else { (*mnCharSel_804D6CCC).child }, JOBJ_HIDDEN as u32);
            hide = true;
        } else {
            ext::HSD_JObjSetFlagsAll(if mnCharSel_804D6CCC.is_null() { null_mut() } else { (*mnCharSel_804D6CCC).child }, JOBJ_HIDDEN as u32);
        }
    }

    if !hide {
        if mnCharSel_804D6CF5 == 1 {
            if (*mnCharSel_804A0BC0[0]).x5 == 1 || (*door(0)).sel_icon as i32 >= MNCS_NUM() {
                mnCharSel_804D6CF7 = 0;
            } else {
                mnCharSel_804D6CF7 = 1;
            }
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        } else {
            valid_count = 0;

            i = 0;
            while i < mnCharSel_804D6CF5 as i32 {
                if (*door(i as usize)).p_kind != 3 {
                    if (*door(i as usize)).sel_icon as i32 >= MNCS_NUM() {
                        hide = true;
                        break;
                    }
                    valid_count += 1;
                }
                i += 1;
            }

            let mut teams_ok = false;
            if !hide {
                // the online lobby sends one player here to pick a fighter: one is enough, and teams
                // (from the local rules) do not apply
                if ext::Frontend_OnlinePick() == 1 && valid_count == 1 {
                    teams_ok = true;
                } else if valid_count >= 2 {
                    if (*mnCharSel_804D6CB0).vs.start.rules.is_teams == 1 {
                        'search: {
                            i = 0;
                            while i < mnCharSel_804D6CF5 as i32 - 1 {
                                if (*door(i as usize)).p_kind == 3 {
                                    i += 1;
                                    continue;
                                }
                                let mut j = i;
                                while j < mnCharSel_804D6CF5 as i32 {
                                    if (*door(j as usize)).p_kind != 3 && (*door(i as usize)).team != (*door(j as usize)).team {
                                        teams_ok = true;
                                        break 'search;
                                    }
                                    j += 1;
                                }
                                i += 1;
                            }
                            hide = true;
                        }
                    } else {
                        teams_ok = true;
                    }
                } else {
                    hide = true;
                }
            }
            if teams_ok {
                let tag = addr_of_mut!(mnCharSel_803F0E8C) as *mut CSSTag;
                i = 0;
                while i < mnCharSel_804D6CF5 as i32 {
                    if (*mnCharSel_804A0BC0[i as usize]).x5 == 1 || (*(*tag.add(i as usize)).data).state != 0 {
                        hide = true;
                        break;
                    }
                    i += 1;
                }

                if !hide {
                    if mnCharSel_804D6CF7 == 0 {
                        ext::HSD_ForeachAnim(jobj.cast(), JOBJ_TYPE as u32, ALL_TYPE_MASK, ext::HSD_AObjReqAnim as *const () as *mut c_void, AOBJ_ARG_AF, &[Va::D(0.0)]);
                    } else if mnCharSel_804D6CF7 > 100 {
                        ext::HSD_ForeachAnim(jobj.cast(), JOBJ_TYPE as u32, ALL_TYPE_MASK, ext::HSD_AObjReqAnim as *const () as *mut c_void, AOBJ_ARG_AF, &[Va::D(10.0)]);
                        mnCharSel_804D6CF7 = 10;
                    }
                    mnCharSel_804D6CF7 = mnCharSel_804D6CF7.wrapping_add(1);
                    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
                    ext::HSD_JObjAnimAll(jobj);
                }
            }
        }
    }
    if hide {
        mnCharSel_804D6CF7 = 0;
        mnCharSel_804D6CF2 = 10;
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }

    {
        let mut p: u32;
        let mut trigger: u32;
        if mnCharSel_804D6CF5 == 1 {
            trigger = (*pad((mnCharSel_804D6CF0 as u8) as usize)).trigger;
        } else {
            p = 0;
            trigger = 0;
            i = 0;
            while i < mnCharSel_804D6CF5 as i32 {
                if (*mnCharSel_804A0BC0[i as usize]).x5 != 3 {
                    let tmp = p as u8;
                    trigger |= (*pad(tmp as usize)).trigger;
                }
                i += 1;
                p += 1;
            }
        }

        if mnCharSel_804D6CF2 == 0 && (trigger & HSD_PAD_START as u32) != 0 {
            if mnCharSel_804D6CF7 != 0 {
                mnCharSel_804D6CF6 = 1;
                mnCharSel_804D6CF2 = 0xFF;
                if mnCharSel_804D6CF5 == 1 {
                    if ext::gm_RumbleEnabledForPlayer(
                        mnCharSel_804D6CF0 as i32,
                        (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF0 as usize)).nametag as i32,
                    ) != 0
                    {
                        ext::lb_80014574(mnCharSel_804D6CF0 as u8, 0, 0xB, 0x1E);
                    }
                } else {
                    i = 0;
                    while i < mnCharSel_804D6CF5 as i32 {
                        if (*door(i as usize)).p_kind == 0
                            && ext::gm_RumbleEnabledForPlayer(i, (*player_at(mnCharSel_804D6CB0, i as usize)).nametag as i32) != 0
                        {
                            ext::lb_80014574(i as u8, 0, 0xB, 0x1E);
                        }
                        i += 1;
                    }
                }
            } else {
                ext::lbAudioAx_80024030(3);
            }
        }
    }
}

pub unsafe extern "C" fn fn_80263354(gobj: *mut HSD_GObj) {
    let jobj = get_jobj(gobj);
    if mnCharSel_804D6CEC % 200 == 0 {
        ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
    }
    ext::HSD_JObjAnimAll(jobj);
}

/// `static const GXColor mnCharSel_804DC560`.
pub const mnCharSel_804DC560: _GXColor = _GXColor { r: 255, g: 255, b: 255, a: 255 };
/// `static const GXColor mnCharSel_804DC564`.
pub const mnCharSel_804DC564: _GXColor = _GXColor { r: 100, g: 100, b: 100, a: 255 };

/// The "clear the K.O. count" decision of the nametag pick: modes 0 and 3..0xA reset it.
#[inline]
fn tag_clears_kos(match_type: u8) -> bool {
    if (match_type as i32) < 3 {
        (match_type as i32) == 0
    } else {
        (match_type as i32) < 0xB
    }
}

/// Nametag list think callback: handles scrolling through the nametag list.
pub unsafe extern "C" fn fn_802633B0(gobj: *mut HSD_GObj) {
    let mut list_jobj: *mut HSD_JObj = null_mut();
    let mut arrow_jobj: *mut HSD_JObj = null_mut();
    let mut list_origin_jobj: *mut HSD_JObj = null_mut();
    let mut list_origin: Vec3 = core::mem::zeroed();
    let mut handicap_slider_jobj: *mut HSD_JObj = null_mut();
    let page: i32;
    let new_page: i32;
    let mut num_entries: i32;
    let trigger: u32;

    let tag = (*gobj).user_data as *mut CSSTagData;
    let mut port: i32 = (*tag).port as i32;

    if mnCharSel_804D6CF5 == 1 {
        if port != 0 {
            port = mnCharSel_804D6CF1 as i32;
        } else {
            port = mnCharSel_804D6CF0 as i32;
        }
    }

    let white = mnCharSel_804DC560;
    let gray = mnCharSel_804DC564;
    num_entries = (*tag).next_tag as i32;

    if num_entries as u32 > 0x78 {
        num_entries -= 8;
    } else {
        num_entries -= 7;
    }

    if mnCharSel_804D6CF5 == 1 {
        lb_find(mnCharSel_804D6CC0, &mut list_jobj, mnCharSel_803F0EBC.xc4 as i32);
        trigger = (*pad((mnCharSel_804D6CF0 as u8) as usize)).trigger;
    } else {
        lb_find(mnCharSel_804D6CC0, &mut list_jobj, (*tag_at((*tag).port as usize)).x4 as i32);
        trigger = (*pad((*tag).port as usize)).trigger;
    }

    match (*tag).state as i32 {
        1 => {
            mnCharSel_8025D1C4((*tag).port as i32, 1);
            if ((*door((*tag).port as usize)).sel_icon as u32) < MNCS_NUM_U() {
                if ext::lbLang_IsSavedLanguageUS() != 0 && (*door((*tag).port as usize)).sel_icon == 0x16 {
                    ext::HSD_SisLib_803A70A0((*tag).name_ls, 0, addr_of!(mnCharSel_803F0A48.gnw_name) as *const i8, &[]);
                } else {
                    ext::HSD_SisLib_803A70A0(
                        (*tag).name_ls,
                        0,
                        ext::gm_80160980((*icon_at((*door((*tag).port as usize)).sel_icon as usize)).char_kind),
                        &[],
                    );
                }
            } else {
                ext::HSD_SisLib_803A70A0(
                    (*tag).name_ls,
                    0,
                    c"\x82\x6d\x82\x60\x82\x6c\x82\x64\x81\x40\x82\x62\x82\x60\x82\x6d\x82\x62\x82\x64\x82\x6b".as_ptr(),
                    &[],
                );
            }
            let mut gray_copy = gray;
            ext::HSD_SisLib_803A74F0((*tag).text, 0, &mut gray_copy);
            req_anim(list_jobj, ALL_TYPE_MASK, 0.0f32);
            (*tag).state = 2;
            (*tag).timer = 0;
            sfxMove();
            return;
        }
        2 => {
            let t: u32 = ((*tag).timer as u32).wrapping_add(1);
            (*tag).timer = t as i32;
            if t > 0xA {
                stop_anim(list_jobj, ALL_TYPE_MASK);
                (*(*tag).name_ls).hidden = 0;
                (*tag).state = 3;
            }
            return;
        }
        3 => 'case3: {
            let vel = (*tag).scroll_amt;
            if vel != 0.0f32 {
                (*tag).scroll_force = fmadds(10.0f32, vel, if vel < 0.0f32 { -1.0f32 } else { 1.0f32 });
            }
            if (*tag).scroll_force != 0.0f32 {
                let scroll_pos = (*tag).x8;
                let new_pos = scroll_pos + (*tag).scroll_force;
                page = f2i32((scroll_pos / 32.0f32) as f64);
                (*tag).x8 = new_pos;
                new_page = f2i32(((*tag).x8 / 32.0f32) as f64);
                if (*tag).x8 > 0.0f32 {
                    (*tag).x8 = 0.0f32;
                }
                if (*tag).next_tag > 7 {
                    if (*tag).x8 < -32.0f32 * num_entries as f32 {
                        (*tag).x8 = -32.0f32 * num_entries as f32;
                    }
                } else {
                    (*tag).x8 = 0.0f32;
                }
                if (*tag).scroll_amt == 0.0f32 {
                    if page < new_page {
                        (*tag).x8 = 32.0f32 * page as f32;
                        (*tag).scroll_force = 0.0f32;
                    } else if page > new_page {
                        (*tag).x8 = 32.0f32 * new_page as f32;
                        (*tag).scroll_force = 0.0f32;
                    }
                }
            }

            if mnCharSel_804D6CF5 == 1 {
                lb_find(mnCharSel_804D6CC0, &mut arrow_jobj, mnCharSel_803F0EBC.xc7 as i32);
            } else {
                lb_find(mnCharSel_804D6CC0, &mut arrow_jobj, (*tag_at((*tag).port as usize)).x7 as i32);
            }

            if (*tag).x8 == 0.0f32 {
                ext::HSD_JObjSetFlags(arrow_jobj, JOBJ_HIDDEN as u32);
            } else {
                ext::HSD_JObjClearFlags(arrow_jobj, JOBJ_HIDDEN as u32);
            }

            if mnCharSel_804D6CF5 == 1 {
                lb_find(mnCharSel_804D6CC0, &mut arrow_jobj, mnCharSel_803F0EBC.xc8 as i32);
            } else {
                lb_find(mnCharSel_804D6CC0, &mut arrow_jobj, (*tag_at((*tag).port as usize)).kostar_text_joint as i32);
            }

            if (*tag).next_tag > 7 {
                if (*tag).x8 == (-32.0f32 * num_entries as f32) {
                    ext::HSD_JObjSetFlags(arrow_jobj, JOBJ_HIDDEN as u32);
                } else {
                    ext::HSD_JObjClearFlags(arrow_jobj, JOBJ_HIDDEN as u32);
                }
            } else {
                ext::HSD_JObjSetFlags(arrow_jobj, JOBJ_HIDDEN as u32);
            }

            ext::HSD_SisLib_803A746C((*tag).name_ls, 0, 0.0f32, (*tag).x8);
            ext::HSD_SisLib_803A746C((*tag).name_ls, 1, 0.0f32, fmadds(32.0f32, (*tag).next_tag as f32, (*tag).x8));
            {
                let mut row_color_v: _GXColor = white;
                let mut used_row_color_v: _GXColor = gray;
                let page_off: i32 = sdiv(f2i32((*tag).x8 as f64), 32) + 1;
                let mut j: i32 = 0;
                while j < 9 {
                    let row_idx = j - page_off;
                    if row_idx < 0 {
                        ext::HSD_SisLib_803A70A0((*tag).name_ls, j + 2, null(), &[]);
                    } else {
                        ext::HSD_SisLib_803A70A0((*tag).name_ls, j + 2, ext::GetNameText((row_idx as u8) as i32), &[]);
                        row_color_v = white;
                        ext::HSD_SisLib_803A74F0((*tag).name_ls, j + 2, &mut row_color_v);
                        {
                            let mut p2: i32 = 0;
                            while p2 < mnCharSel_804D6CF5 as i32 {
                                if p2 != (*tag).port as i32 && (*player_at(mnCharSel_804D6CB0, p2 as usize)).nametag as i32 == row_idx {
                                    used_row_color_v = gray;
                                    ext::HSD_SisLib_803A74F0((*tag).name_ls, j + 2, &mut used_row_color_v);
                                    break;
                                }
                                p2 += 1;
                            }
                        }
                    }
                    ext::HSD_SisLib_803A746C(
                        (*tag).name_ls,
                        j + 2,
                        10.0f32,
                        fmadds(32.0f32, (j + 1) as f32, (srem(f2i32((*tag).x8 as f64), 32) - 0x20) as f32),
                    );
                    j += 1;
                }
            }

            'check_cancel: {
                if trigger & PAD_A != 0 {
                    if mnCharSel_804D6CF5 == 1 {
                        lb_find(mnCharSel_804D6CC0, &mut list_origin_jobj, mnCharSel_803F0EBC.name_list_joint as i32);
                    } else {
                        lb_find(mnCharSel_804D6CC0, &mut list_origin_jobj, (*tag_at((*tag).port as usize)).list_joint as i32);
                    }
                    ext::lb_8000B1CC(list_origin_jobj, null_mut(), &mut list_origin);
                    let cursor_row: f32 = fmadds(0.5f32, 0.8f32 + (list_origin.y - (*mnCharSel_804A0BC0[(*tag).port as usize]).x10), -((*tag).x8 / 32.0f32));
                    let row: i32 = f2i32(cursor_row as f64);

                    if f2i32(cursor_row as f64) == 0 {
                        if ext::lbLang_IsSavedLanguageUS() != 0 && (*door((*tag).port as usize)).sel_icon == 0x16 {
                            ext::HSD_SisLib_803A70A0((*tag).text, 0, addr_of!(mnCharSel_803F0A48.gnw_name) as *const i8, &[]);
                        } else {
                            ext::HSD_SisLib_803A70A0(
                                (*tag).text,
                                0,
                                ext::gm_80160980((*icon_at((*door((*tag).port as usize)).sel_icon as usize)).char_kind),
                                &[],
                            );
                        }
                        (*(*tag).text).default_kerning = 1;
                        if ((*door((*tag).port as usize)).sel_icon as u32) < MNCS_NUM_U() {
                            (*(*tag).text).hidden = 0;
                        } else {
                            (*(*tag).text).hidden = 1;
                        }
                        {
                            if (*player_at(mnCharSel_804D6CB0, port as usize)).nametag != 0x78 {
                                let match_type = (*mnCharSel_804D6CB0).match_type;
                                if tag_clears_kos(match_type) {
                                    *(*mnCharSel_804D6CB0).ko_counts.add((*tag).port as usize) = 0;
                                }
                            }
                        }
                        (*player_at(mnCharSel_804D6CB0, port as usize)).nametag = 0x78;
                        (*tag).use_tag = 0;
                        (*tag).state = 4;
                        sfxMove();
                        break 'check_cancel;
                    }

                    if (*tag).next_tag as i32 == row {
                        if ext::IsNameListFull() == 0 {
                            if mnCharSel_804D6CF5 == 1 {
                                mnCharSel_804D6CF9 = mnCharSel_804D6CF0;
                            } else {
                                mnCharSel_804D6CF9 = (*tag).port as i8;
                            }
                            mnCharSel_804D6CF6 = 4;
                            {
                                let mut k: i32 = 0;
                                while k < mnCharSel_804D6CF5 as i32 {
                                    if (*mnCharSel_804A0BC0[k as usize]).x5 == 1 && mnCharSel_8025FDEC(k as u8) == 0 {
                                        mnCharSel_8025DB34(k as u8);
                                        (*mnCharSel_804A0BC0[k as usize]).x5 = 2;
                                    }
                                    k += 1;
                                }
                            }
                            return;
                        }
                        break 'check_cancel;
                    }

                    if (*tag).next_tag as i32 > row {
                        {
                            let mut p: i32 = 0;
                            while p < mnCharSel_804D6CF5 as i32 {
                                if p != port && (*player_at(mnCharSel_804D6CB0, p as usize)).nametag as i32 == row - 1 {
                                    break 'check_cancel;
                                }
                                p += 1;
                            }
                            {
                                ext::HSD_SisLib_803A70A0((*tag).text, 0, ext::GetNameText(row - 1), &[]);
                                (*(*tag).text).default_kerning = 0;
                                (*(*tag).text).hidden = 0;
                                {
                                    if (*player_at(mnCharSel_804D6CB0, port as usize)).nametag as i32 != row - 1 {
                                        let match_type = (*mnCharSel_804D6CB0).match_type;
                                        if tag_clears_kos(match_type) {
                                            *(*mnCharSel_804D6CB0).ko_counts.add((*tag).port as usize) = 0;
                                        }
                                    }
                                }
                                (*player_at(mnCharSel_804D6CB0, port as usize)).nametag = (row - 1) as u8;
                                (*tag).use_tag = 1;
                                (*tag).state = 4;
                                sfxMove();
                            }
                        }
                    }
                }
            }
            // check_cancel:
            if trigger & PAD_B != 0 {
                (*tag).state = 4;
                sfxBack();
                return;
            }
            break 'case3;
        },
        4 => {
            req_anim(list_jobj, ALL_TYPE_MASK, 20.0f32);
            (*(*tag).name_ls).hidden = 1;
            if (*ext::gmMainLib_GetGameRules()).handicap != 0 && mnCharSel_804D6CF5 == 4 {
                {
                    let mut hval: i32;
                    let mut hval_f: f32;
                    hval = getHandicapValue((*tag).port as i32);
                    hval_f = hval as f32;
                    lb_find(mnCharSel_804D6CC0, &mut handicap_slider_jobj, (*door(port as usize)).cpuslider_joint as i32);
                    {
                        let anim_jobj = handicap_slider_jobj;
                        req_anim(anim_jobj, TOBJ_MASK, hval_f);
                    }
                    ext::HSD_JObjAnimAll(handicap_slider_jobj);
                    stop_anim(handicap_slider_jobj, TOBJ_MASK);
                    list_jobj = handicap_slider_jobj;
                    hval = getHandicapValue((*tag).port as i32);
                    hval_f = 1.25f32 * (hval - 1) as f32;
                    super::hsd::HSD_JObjSetTranslateX(list_jobj, hval_f);
                }
            }
            (*tag).state = 5;
            (*tag).timer = 0;
            return;
        }
        5 => {
            let t2: u32 = ((*tag).timer as u32).wrapping_add(1);
            (*tag).timer = t2 as i32;
            if t2 > 0xA {
                mnCharSel_8025D1C4((*tag).port as i32, 2);
                let mut white_copy = white;
                ext::HSD_SisLib_803A74F0((*tag).text, 0, &mut white_copy);
                (*tag).state = 0;
            }
        }
        _ => {}
    }
}

pub unsafe fn mnCharSel_80264070() {
    mnCharSel_804D6CF4 = 1;
}

pub unsafe extern "C" fn fn_8026407C(gobj: *mut HSD_GObj) {
    ext::HSD_FogSet((*gobj).hsd_obj as *mut HSD_Fog);
}

pub const mnCharSel_804DC580: _GXColor = _GXColor { r: 255, g: 255, b: 0, a: 255 };
pub const mnCharSel_804DC584: _GXColor = _GXColor { r: 20, g: 80, b: 160, a: 255 };
pub const mnCharSel_804DC588: _GXColor = _GXColor { r: 60, g: 140, b: 80, a: 255 };
pub const mnCharSel_804DC58C: _GXColor = _GXColor { r: 160, g: 160, b: 0, a: 255 };
pub const mnCharSel_804DC590: _GXColor = _GXColor { r: 180, g: 80, b: 0, a: 255 };
pub const mnCharSel_804DC594: _GXColor = _GXColor { r: 220, g: 0, b: 0, a: 255 };


// ------------------------------------------------------------------------------------------
// m-ex CSS (TARGET_PC)
// ------------------------------------------------------------------------------------------

/// Give one portrait slot m-ex's art: the portrait joint's DObj gets mexSelectChr's CSP material
/// animation, the emblem joint's SECOND DObj IfAll's Eblm_matanim_joint animation - the same shape
/// in every m-ex patch that does it (Replace CSS VS / SinglePlayer / Training Emblem).
unsafe fn mnCharSel_MexAttach(mex: *mut MexSelectChr, root: *mut HSD_JObj, csp_joint: i32, emblem_joint: i32) {
    let ifall: *mut HSD_Archive = ext::lbDvd_8001819C(c"IfAll".as_ptr());
    let eblm: *mut HSD_MatAnimJoint =
        if !ifall.is_null() { ext::HSD_ArchiveGetPublicAddress(ifall, c"Eblm_matanim_joint".as_ptr()) as *mut HSD_MatAnimJoint } else { null_mut() };
    let mut j: *mut HSD_JObj;
    if root.is_null() {
        return;
    }
    j = null_mut();
    lb_find(root, &mut j, csp_joint);
    let ok = |b: bool| -> Va { Va::P(if b { c"ok".as_ptr() } else { c"MISSING".as_ptr() }.cast()) };
    if trace_on() {
        ext::OSReport(
            c"mexcss: attach csp joint %d: jobj %s dobj %s matanim %s\n".as_ptr(),
            &[Va::I32(csp_joint), ok(!j.is_null()), ok(!j.is_null() && !(*j).u.dobj.is_null()), ok(!(*mex).csp_matanim.is_null())],
        );
    }
    if !j.is_null() && !(*j).u.dobj.is_null() && !(*mex).csp_matanim.is_null() {
        ext::HSD_DObjAddAnimAll((*j).u.dobj, (*mex).csp_matanim, null_mut());
    }
    j = null_mut();
    lb_find(root, &mut j, emblem_joint);
    if trace_on() {
        ext::OSReport(
            c"mexcss: attach eblm joint %d: jobj %s dobj %s next %s eblm %s\n".as_ptr(),
            &[
                Va::I32(emblem_joint),
                ok(!j.is_null()),
                ok(!j.is_null() && !(*j).u.dobj.is_null()),
                ok(!j.is_null() && !(*j).u.dobj.is_null() && !(*(*j).u.dobj).next.is_null()),
                ok(!eblm.is_null()),
            ],
        );
    }
    if !j.is_null() && !(*j).u.dobj.is_null() && !(*(*j).u.dobj).next.is_null() && !eblm.is_null() {
        ext::HSD_DObjAddAnimAll((*(*j).u.dobj).next, (*eblm).matanim, null_mut());
    }
}

// ---- PNG -> GX texture -> HSD scene ------------------------------------------------------------
//
// The smallest end-to-end proof that the port can put its own art on screen: read a `.gxtex`
// (MELEE_MENUTEX_DIR / MELEE_MENUTEX, opened by GxTex_OpenEnv), hand the bytes to HSD as an
// HSD_ImageDesc (+ HSD_Tlut for a colour-indexed format) and draw it as one textured HSD_SObj on
// the GXLinkMax list (priority 0xFF). Unset, GxTex_OpenEnv fails and this is a no-op.

static mut mnCharSel_MenuTexImage: HSD_ImageDesc = unsafe { core::mem::zeroed() };
static mut mnCharSel_MenuTexTlut: HSD_Tlut = unsafe { core::mem::zeroed() };
static mut mnCharSel_MenuTexDesc: HSD_SObjDesc = unsafe { core::mem::zeroed() };

unsafe extern "C" fn mnCharSel_MenuTexDraw(gobj: *mut HSD_GObj, pass: i32) {
    let mut proj: Mtx44 = [[0.0; 4]; 4];
    let mut view: Mtx = [[0.0; 4]; 3];

    // 640x480 in screen pixels with Y growing downwards, which is the space HSD_SObj's quad is
    // written in: it emits GXPosition2f32(x, -y). Near/far 0..2 with the view matrix one unit down
    // the -Z axis puts the quad inside the frustum.
    ext::GXSetViewport(0.0f32, 0.0f32, 640.0f32, 480.0f32, 0.0f32, 1.0f32);
    ext::GXSetScissor(0, 0, 640, 480);
    ext::MTXOrtho(proj.as_mut_ptr().cast(), 0.0f32, -480.0f32, 0.0f32, 640.0f32, 0.0f32, 2.0f32);
    ext::GXSetProjection(proj.as_mut_ptr().cast(), GX_ORTHOGRAPHIC);
    view[0][0] = 1.0f32;
    view[0][1] = 0.0f32;
    view[0][2] = 0.0f32;
    view[0][3] = 0.0f32;
    view[1][0] = 0.0f32;
    view[1][1] = 1.0f32;
    view[1][2] = 0.0f32;
    view[1][3] = 0.0f32;
    view[2][0] = 0.0f32;
    view[2][1] = 0.0f32;
    view[2][2] = 1.0f32;
    view[2][3] = -1.0f32;
    ext::GXLoadPosMtxImm(view.as_mut_ptr().cast(), GX_PNMTX0);
    ext::GXSetCurrentMtx(GX_PNMTX0);

    // HSD_SObj's draw sets its own TEV stages, vertex descriptor and blend mode but inherits the
    // texgen and the colour channel, so those are established here.
    ext::GXSetNumChans(1);
    ext::GXSetChanCtrl(GX_COLOR0A0, 0, GX_SRC_REG, GX_SRC_REG, GX_LIGHT_NULL, GX_DF_NONE, GX_AF_NONE);
    {
        let white = crate::GXColor { r: 255, g: 255, b: 255, a: 255 };
        ext::GXSetChanMatColor(GX_COLOR0A0, white);
    }
    ext::GXSetTexCoordGen2(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY, 0, GX_PTIDENTITY);

    ext::HSD_SObjLib_803A49E0(gobj, pass);
}

unsafe fn mnCharSel_MenuTexSetup() {
    let gobj: *mut HSD_GObj;
    let sobj: *mut HSD_SObj;
    let image: *mut c_void;
    let lut: *mut c_void;
    let handle: i32;
    let (w, h, fmt, tfmt, tents): (i32, i32, i32, i32, i32);
    let mut scale_shift: i32;
    let mut scale: f32;

    handle = ext::GxTex_OpenEnv();
    if handle < 0 {
        return;
    }
    w = ext::GxTex_Width(handle);
    h = ext::GxTex_Height(handle);
    fmt = ext::GxTex_Format(handle);
    tfmt = ext::GxTex_TlutFormat(handle);
    tents = ext::GxTex_TlutEntries(handle);

    // The texture data is copied into the game heap because that is what an HSD_ImageDesc points at
    // and what GXInitTexObj records; the loader's own copy is released immediately after.
    image = ext::HSD_MemAlloc(ext::GxTex_ImageSize(handle));
    if image.is_null() {
        ext::GxTex_Close(handle);
        return;
    }
    ext::GxTex_CopyImage(handle, image);

    mnCharSel_MenuTexImage.image_ptr = image;
    mnCharSel_MenuTexImage.width = w as u16;
    mnCharSel_MenuTexImage.height = h as u16;
    mnCharSel_MenuTexImage.format = fmt as _GXTexFmt;
    mnCharSel_MenuTexImage.mipmap = 0;
    mnCharSel_MenuTexImage.minLOD = 0.0f32;
    mnCharSel_MenuTexImage.maxLOD = 0.0f32;
    mnCharSel_MenuTexDesc.image = addr_of_mut!(mnCharSel_MenuTexImage);
    mnCharSel_MenuTexDesc.tlut = null_mut();

    if tfmt >= 0 {
        lut = ext::HSD_MemAlloc(ext::GxTex_TlutSize(handle));
        if lut.is_null() {
            ext::GxTex_Close(handle);
            return;
        }
        ext::GxTex_CopyTlut(handle, lut);
        mnCharSel_MenuTexTlut.lut = lut;
        mnCharSel_MenuTexTlut.fmt = tfmt as _GXTlutFmt;
        mnCharSel_MenuTexTlut.n_entries = tents as u16;
        // GX_TLUT0, because HSD_SObj's draw hardcodes GXLoadTlut(..., GX_TLUT0) for a CI texture.
        mnCharSel_MenuTexTlut.tlut_name = GX_TLUT0 as u32;
        mnCharSel_MenuTexDesc.tlut = addr_of_mut!(mnCharSel_MenuTexTlut);
    }
    ext::GxTex_Close(handle);

    gobj = ext::GObj_Create(0xE, 0xF, 0);
    ext::HSD_GObjObject_80390A70(gobj, HSD_SObjLib_804D7960, null_mut());
    ext::GObj_SetupGXLinkMax(gobj, Some(mnCharSel_MenuTexDraw), 0xFF);
    sobj = ext::HSD_SObjLib_803A477C(gobj, addr_of_mut!(mnCharSel_MenuTexDesc), GX_CLAMP as u32, GX_CLAMP as u32, 0x80, 0);

    // Fit inside 640x480 and centre it: the art is authored at 2x, so a 1024x512 panel is a 512x256
    // rectangle on a 640x480 screen.
    scale_shift = 0;
    while (w >> scale_shift) > 640 || (h >> scale_shift) > 480 {
        scale_shift += 1;
    }
    scale = 1.0f32;
    while scale_shift > 0 {
        scale_shift -= 1;
        scale *= 0.5f32;
    }
    (*sobj).x1C = scale;
    (*sobj).x20 = scale;
    (*sobj).x10 = fmadds(-(w as f32), scale, 640.0f32) * 0.5f32;
    (*sobj).x14 = fmadds(-(h as f32), scale, 480.0f32) * 0.5f32;
    // Integers only (see the C: the bridge's float-vararg path).
    ext::OSReport(
        c"menutex: %dx%d fmt %d tlut fmt %d entries %d -> quad at %d,%d size %dx%d\n".as_ptr(),
        &[
            Va::I32(w),
            Va::I32(h),
            Va::I32(fmt),
            Va::I32(tfmt),
            Va::I32(tents),
            Va::I32(f2i32((*sobj).x10 as f64)),
            Va::I32(f2i32((*sobj).x14 as f64)),
            Va::I32(f2i32((w as f32 * scale) as f64)),
            Va::I32(f2i32((h as f32 * scale) as f64)),
        ],
    );
}

/// Slot positions and bounds of the m-ex icon grid, captured before anything moves (the C's
/// function-local `static` arrays).
static mut MEX_POS: [Vec3; CSS_ICON_MAX] = unsafe { core::mem::zeroed() };
static mut MEX_BL: [f32; CSS_ICON_MAX] = [0.0; CSS_ICON_MAX];
static mut MEX_BR: [f32; CSS_ICON_MAX] = [0.0; CSS_ICON_MAX];
static mut MEX_BU: [f32; CSS_ICON_MAX] = [0.0; CSS_ICON_MAX];
static mut MEX_BD: [f32; CSS_ICON_MAX] = [0.0; CSS_ICON_MAX];
/// `static int captured` / `static CSSIcon retail_none` / `static u8 retail_vs[], retail_1p[]` of
/// `mnCharSel_MexSetup`.
static mut MEX_CAPTURED: i32 = 0;
static mut MEX_RETAIL_NONE: CSSIcon = ICON_NONE;
static mut MEX_RETAIL_VS: [u8; SELKIND_COUNT as usize] = [0; SELKIND_COUNT as usize];
static mut MEX_RETAIL_1P: [u8; SELKIND_COUNT as usize] = [0; SELKIND_COUNT as usize];

/// Ported from m-ex (https://github.com/akaneia/m-ex): asm/m-ex/CSS Expansion/ - the data-driven
/// CSS, re-expressed natively. When the MnSlChr archive carries `mexSelectChr` (Akaneia does) and
/// mexData has a CSS icon table: the icon table and count come from mexData, with each icon's m-ex
/// EXTERNAL id translated to the port's CharacterKind (an icon whose fighter the port does not have
/// is marked unavailable: hidden, never selectable); the icons are drawn by mexSelectChr's own icon
/// model, and every icon-joint lookup resolves in it (`MNCS_ICON_ROOT`); the retail menu model's 25
/// icon joints are hidden. Returns silently on retail, leaving the retail CSS untouched.
unsafe fn mnCharSel_MexSetup() {
    let mex: *mut MexSelectChr;
    let tbl: *mut CSSIcon;
    let gobj: *mut HSD_GObj;
    let root: *mut HSD_JObj;
    let mut j: *mut HSD_JObj;
    let n: i32;
    let mut i: i32;

    if MEX_CAPTURED == 0 {
        MEX_RETAIL_NONE = *icon_at(SELKIND_COUNT as usize);
        i = 0;
        while i < SELKIND_COUNT as i32 {
            MEX_RETAIL_VS[i as usize] = (*icon_at(i as usize)).joint_id_vs;
            MEX_RETAIL_1P[i as usize] = (*icon_at(i as usize)).joint_id_1p;
            i += 1;
        }
        MEX_CAPTURED = 1;
    }
    mnCharSel_Mex = null_mut();
    mnCharSel_IconRoot = null_mut();
    mnCharSel_StcAttached = 0;
    mnCharSel_IconCount = SELKIND_COUNT as u8;

    mex = ext::HSD_ArchiveGetPublicAddress(mnCharSel_804D6CD0, c"mexSelectChr".as_ptr()) as *mut MexSelectChr;
    n = if !mex.is_null() { ext::Mex_CssIconCount() } else { 0 };
    tbl = if n > 0 && n <= CSS_ICON_MAX as i32 { ext::Mex_CssIconTable() as *mut CSSIcon } else { null_mut() };
    if tbl.is_null() {
        return;
    }

    // hide the retail icons: the m-ex model replaces them
    i = 0;
    while i < SELKIND_COUNT as i32 {
        j = null_mut();
        lb_find(
            mnCharSel_804D6CC0,
            &mut j,
            (if mnCharSel_804D6CF5 == 1 { MEX_RETAIL_1P[i as usize] } else { MEX_RETAIL_VS[i as usize] }) as i32,
        );
        if !j.is_null() {
            ext::HSD_JObjSetFlagsAll(j, JOBJ_HIDDEN as u32);
        }
        i += 1;
    }

    i = 0;
    while i < n {
        let ck: i32;
        *icon_at(i as usize) = *tbl.add(i as usize);
        ck = ext::Mex_ExtToPortCKind((*icon_at(i as usize)).char_kind as i32);
        // not available = the port lacks the fighter, OR it is still locked on this save (locked
        // fighters are hidden, as on retail). Both are excluded from the packed grid below, so
        // neither leaves a hole.
        mnCharSel_IconUnavail[i as usize] = ((ck < 0) || ext::gm_IsCKindUnlocked(ck as u8) == 0) as u8;
        // Stub: m-ex fighters have no single-player STORY support in the port, so they are hidden in
        // the three story modes; they stay in Training and the Stadium modes.
        if ck >= ChKind_Mex0 as i32
            && ((*mnCharSel_804D6CB0).match_type as u32 == REG_CLASSIC
                || (*mnCharSel_804D6CB0).match_type as u32 == REG_ADVENTURE
                || (*mnCharSel_804D6CB0).match_type as u32 == REG_ALLSTAR)
        {
            mnCharSel_IconUnavail[i as usize] = 1;
        }
        if trace_on() {
            let ic = icon_at(i as usize);
            ext::OSReport(
                c"mexcss: icon %2d ext %2d -> CKind %3d  unavail %d  joint vs %2d 1p %2d  bounds [%d..%d, %d..%d]\n".as_ptr(),
                &[
                    Va::I32(i),
                    Va::I32((*ic).char_kind as i32),
                    Va::I32(ck),
                    Va::I32(mnCharSel_IconUnavail[i as usize] as i32),
                    Va::I32((*ic).joint_id_vs as i32),
                    Va::I32((*ic).joint_id_1p as i32),
                    Va::I32(f2i32((*ic).bound_l as f64)),
                    Va::I32(f2i32((*ic).bound_r as f64)),
                    Va::I32(f2i32((*ic).bound_d as f64)),
                    Va::I32(f2i32((*ic).bound_u as f64)),
                ],
            );
        }
        (*icon_at(i as usize)).char_kind = if ck < 0 { MEX_RETAIL_NONE.char_kind } else { ck as u8 };
        (*icon_at(i as usize)).anim_timer = 0;
        i += 1;
    }
    if trace_on() {
        ext::OSReport(
            c"mexcss: %d icons, mode %d (1 = 1P/Training, 4 = VS), match_type %d\n".as_ptr(),
            &[Va::I32(n), Va::I32(mnCharSel_804D6CF5 as i32), Va::I32((*mnCharSel_804D6CB0).match_type as i32)],
        );
    }
    *icon_at(n as usize) = MEX_RETAIL_NONE;
    mnCharSel_IconCount = n as u8;

    gobj = ext::GObj_Create(4, 5, 0x80);
    root = ext::HSD_JObjLoadJoint((*mex).icon_joint);
    ext::HSD_JObjAddAnimAll(root, (*mex).icon_animjoint, (*mex).icon_matanim, null_mut());
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, root.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 1, 0x80);
    ext::HSD_JObjReqAnimAll(root, 0.0f32);
    ext::HSD_JObjAnimAll(root);
    stop_anim(root, ALL_TYPE_MASK);

    // Pack the grid. Akaneia's layout has a slot for each of its fighters; the ones the port does not
    // have are hidden, which left holes in the grid. Keep Akaneia's slot positions and order, but
    // slide the available icons into consecutive slots, row by row, centring a partial last row. An
    // icon moves by moving its own joint (the joint carries that fighter's art) and taking the
    // slot's bounds (the cursor hit-tests bounds). Every slot's position and bounds are captured
    // BEFORE anything moves.
    {
        let mut row_start = [0i32; CSS_ICON_MAX];
        let mut row_cap = [0i32; CSS_ICON_MAX];
        let mut row_take = [0i32; CSS_ICON_MAX];
        let mut rows: i32 = 0;
        let mut avail = [0i32; CSS_ICON_MAX];
        let mut n_avail: i32 = 0;
        let mut placed: i32 = 0;
        let mut r: i32;
        let mut k: i32;
        i = 0;
        while i < n {
            let iu = i as usize;
            j = null_mut();
            lb_find(
                root,
                &mut j,
                (if mnCharSel_804D6CF5 == 1 { (*icon_at(iu)).joint_id_1p } else { (*icon_at(iu)).joint_id_vs }) as i32,
            );
            if !j.is_null() {
                super::hsd::HSD_JObjGetTranslation(j, addr_of_mut!(MEX_POS[iu]));
            }
            MEX_BL[iu] = (*icon_at(iu)).bound_l;
            MEX_BR[iu] = (*icon_at(iu)).bound_r;
            MEX_BU[iu] = (*icon_at(iu)).bound_u;
            MEX_BD[iu] = (*icon_at(iu)).bound_d;
            // a new row starts where the slot's top bound changes by more than rounding
            if i == 0 || MEX_BU[iu] - MEX_BU[iu - 1] > 1.0f32 || MEX_BU[iu - 1] - MEX_BU[iu] > 1.0f32 {
                row_start[rows as usize] = i;
                row_cap[rows as usize] = 0;
                rows += 1;
            }
            row_cap[rows as usize - 1] += 1;
            if mnCharSel_IconUnavail[iu] == 0 {
                avail[n_avail as usize] = i;
                n_avail += 1;
            }
            i += 1;
        }
        if trace_on() {
            ext::OSReport(c"mexcss: packing %d of %d icons over %d rows\n".as_ptr(), &[Va::I32(n_avail), Va::I32(n), Va::I32(rows)]);
        }
        r = 0;
        while r < rows {
            if trace_on() {
                ext::OSReport(
                    c"mexcss:   row %d: slots %d..%d (%d wide)\n".as_ptr(),
                    &[Va::I32(r), Va::I32(row_start[r as usize]), Va::I32(row_start[r as usize] + row_cap[r as usize] - 1), Va::I32(row_cap[r as usize])],
                );
            }
            r += 1;
        }
        // Each row's share (see mnCharSel_MexPackRows), then each row centred in its slots.
        {
            let total = mnCharSel_MexPackRows(&row_cap, rows, n_avail, &mut row_take);
            if total != n_avail {
                if trace_on() {
                    ext::OSReport(c"mexcss: only %d of %d icons fit the %d-row grid\n".as_ptr(), &[Va::I32(total), Va::I32(n_avail), Va::I32(rows)]);
                }
            }
        }
        r = 0;
        while r < rows && placed < n_avail {
            let take = row_take[r as usize];
            let off = (row_cap[r as usize] - take) / 2;
            k = 0;
            while k < take {
                let icon_i = avail[placed as usize];
                let slot = row_start[r as usize] + off + k;
                j = null_mut();
                lb_find(
                    root,
                    &mut j,
                    (if mnCharSel_804D6CF5 == 1 { (*icon_at(icon_i as usize)).joint_id_1p } else { (*icon_at(icon_i as usize)).joint_id_vs }) as i32,
                );
                if trace_on() {
                    ext::OSReport(
                        c"mexcss:   icon %2d -> slot %2d  joint %s\n".as_ptr(),
                        &[Va::I32(icon_i), Va::I32(slot), Va::P(if !j.is_null() { c"ok".as_ptr() } else { c"MISSING".as_ptr() }.cast())],
                    );
                }
                if !j.is_null() {
                    super::hsd::HSD_JObjSetTranslate(j, addr_of!(MEX_POS[slot as usize]));
                }
                (*icon_at(icon_i as usize)).bound_l = MEX_BL[slot as usize];
                (*icon_at(icon_i as usize)).bound_r = MEX_BR[slot as usize];
                (*icon_at(icon_i as usize)).bound_u = MEX_BU[slot as usize];
                (*icon_at(icon_i as usize)).bound_d = MEX_BD[slot as usize];
                k += 1;
                placed += 1;
            }
            r += 1;
        }
    }

    // Portraits and emblems (VS doors): as m-ex's "Replace CSS VS Emblem" does, give each door's
    // portrait joint's DObj the CSP material animation, and each emblem joint's SECOND DObj IfAll's
    // Eblm_matanim_joint animation. Frames are picked per selection in mnCharSel_MexPortrait.
    if mnCharSel_804D6CF5 != 1 {
        let mut d: i32 = 0;
        while d < 4 {
            mnCharSel_MexAttach(mex, mnCharSel_804D6CC0, (*door(d as usize)).costume_joint as i32, (*door(d as usize)).emblem_joint as i32);
            d += 1;
        }
    } else {
        // 1P / Training: the regend model's portrait is joint 45, its emblem joint 43. The Training
        // CPU door is a separate model, attached where it is built.
        let stc = ext::gm_MexStockMatAnim();
        mnCharSel_MexAttach(mex, mnCharSel_804D6CC0, 0x2D, 0x2B);
        // m-ex "Replace CSS SinglePlayer Stock And Emblem": the five stock dots use Stc_icns
        if !stc.is_null() {
            let mut d: usize = 0;
            while d < 5 {
                j = null_mut();
                lb_find(mnCharSel_804D6CC0, &mut j, data2.xf0[d] as i32);
                if !j.is_null() {
                    ext::HSD_JObjAddAnimAll(j, null_mut(), stc, null_mut());
                }
                d += 1;
            }
            mnCharSel_StcAttached = 1;
        }
    }

    mnCharSel_IconRoot = root;
    mnCharSel_Mex = mex;
}

/// Ported from m-ex (https://github.com/akaneia/m-ex): CSS Expansion/mexSelectChr/CursorScale -
/// Hand.asm and - Puck.asm. mexSelectChr's icon grid is denser than retail's 25 icons, so both
/// cursors are scaled by mexData's menu param 0 (Akaneia: 0.95). The hand's graphic hangs off a
/// child joint while the joint itself is the hot spot, so m-ex compensates by translating the child
/// by (1 - scale) * the hand's tip offset. The puck has no child and needs no shift. Retail (no
/// mexSelectChr) is untouched.
unsafe fn mnCharSel_MexScaleCursor(jobj: *mut HSD_JObj, is_hand: i32) {
    if mnCharSel_Mex.is_null() || jobj.is_null() {
        return;
    }
    // MELEE_CSS_CURSOR_SCALE=0 turns this off.
    if ext::Mex_CssCursorScaleEnabled() == 0 {
        return;
    }
    let s: f32 = ext::Mex_MenuParamF(0);
    if !(s > 0.0f32) || s == 1.0f32 {
        return;
    }
    let target = if is_hand != 0 { (*jobj).child } else { jobj };
    if target.is_null() {
        return;
    }
    (*target).scale.x *= s;
    (*target).scale.y *= s;
    (*target).scale.z *= s;
    if is_hand != 0 {
        (*target).translate.x = (1.0f32 - s) * 2.7f32;
        (*target).translate.y = (1.0f32 - s) * -2.0f32;
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}


// ------------------------------------------------------------------------------------------
// building the scene
// ------------------------------------------------------------------------------------------

/// `HSD_LObjSetNext` (lobj.h).
unsafe fn HSD_LObjSetNext(lobj: *mut HSD_LObj, next: *mut HSD_LObj) {
    if lobj.is_null() {
        super::hsd::ext::__assert(c"lobj.h".as_ptr(), 0x136, c"lobj".as_ptr());
    }
    (*lobj).next = next;
}

/// `text->pos_x/y/z = x, y, z`.
#[inline]
unsafe fn text_pos(text: *mut HSD_Text, x: f32, y: f32, z: f32) {
    (*text).pos_x = x;
    (*text).pos_y = y;
    (*text).pos_z = z;
}
/// `text->font_size.x/y = x, y`.
#[inline]
unsafe fn text_font(text: *mut HSD_Text, x: f32, y: f32) {
    (*text).font_size.x = x;
    (*text).font_size.y = y;
}
/// `HSD_SisLib_803A6B98(text, x, y, fmt)`.
#[inline]
unsafe fn text_add(text: *mut HSD_Text, x: f32, y: f32, fmt: *const i8) {
    ext::HSD_SisLib_803A6B98(text, x, y, fmt, &[]);
}
/// `HSD_JObjAddAnimAll(jobj, m.animjoint, m.matanim_joint, m.shapeanim_joint)` for a model.
#[inline]
unsafe fn add_model_anims(jobj: *mut HSD_JObj, m: *const StaticModelDesc) {
    ext::HSD_JObjAddAnimAll(jobj, (*m).animjoint, (*m).matanim_joint, (*m).shapeanim_joint);
}
/// The `HSD_ForeachAnim(j, JOBJ_TYPE, JOBJ_MASK, HSD_AObjReqAnim, AOBJ_ARG_AF, frame)` pair of the tag boxes.
#[inline]
unsafe fn req_anim_jobj_mask(j: *mut HSD_JObj, frame: f32) {
    req_anim(j, JOBJ_MASK, frame);
}
/// The "clear the K.O. counts" decision shared by the tag overflow and the stars: modes 0 and 3..0xA.
#[inline]
fn clears_kos(mt: u8) -> bool {
    tag_clears_kos(mt)
}

/// Builds the character select screen (cameras, light, fog, backgrounds, menus, icons, cursors,
/// tokens, name tags, the mode's high-score texts and the start banner) and returns the result of
/// `lbAudioAx_80023F28(gmMainLib_8015ECB0())`.
pub unsafe fn mnCharSel_802640A0() -> i32 {
    let mut sp108: *mut HSD_JObj = null_mut();
    let mut spEC: Vec3 = core::mem::zeroed();
    let spE8: _GXColor;
    let spE4: _GXColor;
    let spE0: _GXColor;
    let spDC: _GXColor;
    let hard_color: _GXColor;
    let spD4: _GXColor;
    let row_b: i32;
    let row_a: i32;
    let mut gobj: *mut HSD_GObj;
    let mut jobj: *mut HSD_JObj;
    let mut text: *mut HSD_Text = null_mut();
    let ctx: i32;
    let mut found: i32;
    let mut num_players: i32;
    let mut i: i32;
    let mut slot: i32;
    let mut icon: i32;
    let match_type: u8 = (*mnCharSel_804D6CB0).match_type;

    if match_type != 0 {
        ext::lbAudioAx_800237A8((*(addr_of!(mnCharSel_803F0A48.mode_info) as *const CSSModeInfo).add(match_type as usize)).enter_sfx, 0x7F, 0x40);
    } else {
        match (*ext::gmMainLib_GetGameRules()).mode as i32 {
            0 => {
                ext::lbAudioAx_800237A8(0x7535, 0x7F, 0x40);
            }
            1 => {
                ext::lbAudioAx_800237A8(0x7538, 0x7F, 0x40);
            }
            2 => {
                ext::lbAudioAx_800237A8(0x7537, 0x7F, 0x40);
            }
            3 => {
                ext::lbAudioAx_800237A8(0x7536, 0x7F, 0x40);
            }
            _ => {}
        }
    }

    mnCharSel_804D6CE0 = null_mut();
    mnCharSel_804D6CDC = null_mut();
    mnCharSel_804D6CE8 = null_mut();
    mnCharSel_804D6CE4 = null_mut();
    mnCharSel_CStickText = null_mut();
    mnCharSel_804D6CF2 = 0x1E;
    mnCharSel_804D6CF3 = 0;
    {
        let mt_p = addr_of_mut!((*mnCharSel_804D6CB0).match_type);

        if *mt_p as u32 >= 0xB {
            mnCharSel_804D6CF5 = 1;
            num_players = 1;
        } else {
            mnCharSel_804D6CF5 = 4;
            num_players = 4;
        }
        if *mt_p as u32 == TRAINING_MODE {
            num_players = 2;
        }
    }
    mnCharSel_804D6CF6 = 0;
    if mnCharSel_804D6CF5 == 1 && mnCharSel_804D6CF0 < 0 {
        mnCharSel_804D6CF0 = 0;
    }
    if mnCharSel_804D6CF0 == 0 {
        mnCharSel_804D6CF1 = 1;
    } else {
        mnCharSel_804D6CF1 = 0;
    }

    if (mnCharSel_804D6CF4 as u8) != 0 {
        mnCharSel_804D6CF4 = 0;
        i = 0;
        while i < num_players {
            (*player_at(mnCharSel_804D6CB0, i as usize)).handicap = 9;
            i += 1;
        }
    }

    mnCharSel_804D6CB8 = ext::GObj_Create(2, 3, 0x80);
    gobj = mnCharSel_804D6CB8;
    {
        MenMain_cam = (*css_data_table).cam;
        let cobj = ext::HSD_CObjLoadDesc(MenMain_cam);
        ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_CameraKind, cobj.cast());
    }
    ext::GObj_SetupGXLinkMax(gobj, Some(ext::HSD_GObj_803910D8), 0);
    (*gobj).gxlink_prios = 0x1F;
    ext::HSD_GObj_SetupProc(gobj, Some(ext::mn_8022BA1C), 5);
    ctx = ext::HSD_SisLib_803A611C(0, gobj, 7, 8, 0x80, 1, 0x80, 0);

    gobj = ext::GObj_Create(3, 4, 0x80);
    {
        let lobj0 = ext::HSD_LObjLoadDesc((*css_data_table).light0);
        let lobj1 = ext::HSD_LObjLoadDesc((*css_data_table).light1);
        HSD_LObjSetNext(lobj0, lobj1);
        ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_LightKind as u8, lobj0.cast());
    }
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_LObjCallback), 0, 0x80);

    gobj = ext::GObj_Create(0xE, 2, 0);
    {
        let fog = ext::HSD_FogLoadDesc((*css_data_table).fog);
        ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_FogKind as u8, fog.cast());
    }
    ext::GObj_SetupGXLink(
        gobj,
        Some(core::mem::transmute::<unsafe extern "C" fn(*mut HSD_GObj), unsafe extern "C" fn(*mut HSD_GObj, i32)>(fn_8026407C)),
        0,
        0x80,
    );

    gobj = ext::GObj_Create(4, 5, 0x80);
    jobj = ext::HSD_JObjLoadJoint((*css_models).background.joint);
    add_model_anims(jobj, addr_of!((*css_models).background));
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 1, 0x80);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_80263354), 4);
    ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
    ext::HSD_JObjAnimAll(jobj);

    mnCharSel_804D6CBC = ext::GObj_Create(4, 5, 0x80);
    if mnCharSel_804D6CF5 == 1 {
        mnCharSel_804D6CC0 = ext::HSD_JObjLoadJoint((*css_models).regend_menu.joint);
        add_model_anims(mnCharSel_804D6CC0, addr_of!((*css_models).regend_menu));
    } else {
        mnCharSel_804D6CC0 = ext::HSD_JObjLoadJoint((*css_models).menu.joint);
        add_model_anims(mnCharSel_804D6CC0, addr_of!((*css_models).menu));
    }
    {
        let obj_kind: u8 = HSD_GObj_JObjKind;
        ext::HSD_GObjObject_80390A70(mnCharSel_804D6CBC, obj_kind, mnCharSel_804D6CC0.cast());
    }
    ext::GObj_SetupGXLink(mnCharSel_804D6CBC, Some(ext::HSD_GObj_JObjCallback), 1, 0x80);
    ext::HSD_GObj_SetupProc(mnCharSel_804D6CBC, Some(fn_8025F0E0), 4);
    ext::HSD_JObjReqAnimAll(mnCharSel_804D6CC0, 0.0f32);
    ext::HSD_JObjAnimAll(mnCharSel_804D6CC0);
    stop_anim(mnCharSel_804D6CC0, ALL_TYPE_MASK);

    mnCharSel_MexSetup();
    mnCharSel_MenuTexSetup();
    // The retail Luigi-row relocation moves retail icon joints and rewrites two retail rows' bounds;
    // with an m-ex CSS the layout and bounds come from mexData instead.
    if mnCharSel_Mex.is_null() {
        if ext::gm_IsCKindUnlocked(CKind_Luigi as u8) == 0 {
            row_a = 2;
            row_b = 0x13;
        } else {
            row_a = 0x13;
            row_b = 2;
        }
        if mnCharSel_804D6CF5 == 1 {
            lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut sp108, (*icon_at(row_a as usize)).joint_id_1p as i32);
        } else {
            lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut sp108, (*icon_at(row_a as usize)).joint_id_vs as i32);
        }
        super::hsd::HSD_JObjSetTranslateY(sp108, ICONROWY_BTM);
        (*icon_at(row_a as usize)).bound_u = ICONROWHT_BTM_TOP;
        (*icon_at(row_a as usize)).bound_d = ICONROWHT_BTM_BTM;
        if mnCharSel_804D6CF5 == 1 {
            lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut sp108, (*icon_at(row_b as usize)).joint_id_1p as i32);
        } else {
            lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut sp108, (*icon_at(row_b as usize)).joint_id_vs as i32);
        }
        super::hsd::HSD_JObjSetTranslateY(sp108, ICONROWY_TOP);
        (*icon_at(row_b as usize)).bound_u = ICONROWHT_TOP_TOP;
        (*icon_at(row_b as usize)).bound_d = ICONROWHT_MID_TOP;
    }

    icon = 0;
    while icon < MNCS_NUM() {
        let ic = icon_at(icon as usize);
        (*ic).state = ext::gm_IsCKindUnlocked((*ic).char_kind) as u8;
        (*ic).anim_timer = 0;
        if mnCharSel_804D6CF5 == 1 {
            lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut sp108, (*ic).joint_id_1p as i32);
        } else {
            lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut sp108, (*ic).joint_id_vs as i32);
        }
        if !mnCharSel_Mex.is_null() {
            // The retail switch below keys off RETAIL icon indices (0, 8, 9... = the unlockable
            // fighters) and retail-only anims. With an m-ex layout: an icon whose fighter the port
            // lacks, or that is still locked on this save, is hidden and dead; the rest are
            // selectable.
            if mnCharSel_IconUnavail[icon as usize] != 0 || (*ic).state == 0 {
                (*ic).state = 0;
                if !sp108.is_null() {
                    ext::HSD_JObjSetFlagsAll(sp108, JOBJ_HIDDEN as u32);
                }
            } else {
                (*ic).state = 2;
            }
            if trace_on() {
                ext::OSReport(
                    c"mexcss: final icon %2d CKind %3d state %d %s joint %s\n".as_ptr(),
                    &[
                        Va::I32(icon),
                        Va::I32((*ic).char_kind as i32),
                        Va::I32((*ic).state as i32),
                        Va::P(if (*ic).state == 0 { c"HIDDEN ".as_ptr() } else { c"visible".as_ptr() }.cast()),
                        Va::P(if !sp108.is_null() { c"ok".as_ptr() } else { c"MISSING".as_ptr() }.cast()),
                    ],
                );
            }
            icon += 1;
            continue;
        }
        match icon {
            0 | 8 | 9 | 17 | 18 | 24 => {
                if (*ic).state == 0 {
                    ext::HSD_JObjSetFlags(sp108, JOBJ_HIDDEN as u32);
                } else {
                    sp108 = jobj_parent(sp108);
                    (*ic).state = 2;
                    {
                        let anim_jobj = sp108;
                        req_anim(anim_jobj, ALL_TYPE_MASK, 20.0f32);
                    }
                    ext::HSD_JObjAnimAll(sp108);
                }
            }
            _ => {
                if (*ic).state == 0 {
                    req_anim(sp108, ALL_TYPE_MASK, 30.0f32);
                    ext::HSD_JObjAnimAll(sp108);
                } else {
                    (*ic).state = 2;
                }
            }
        }
        icon += 1;
    }

    {
        let mt: u8 = (*mnCharSel_804D6CB0).match_type;
        if mt >= 0xF && mt <= 0x16 {
            gobj = ext::GObj_Create(4, 5, 0x80);
            mnCharSel_804D6CC8 = ext::HSD_JObjLoadJoint((*css_models).regend_options.joint);
            ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, mnCharSel_804D6CC8.cast());
            ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 1, 0x80);
            ext::HSD_GObj_SetupProc(gobj, Some(fn_8025FB2C), 4);
            add_model_anims(mnCharSel_804D6CC8, addr_of!((*css_models).regend_options));
            ext::HSD_JObjReqAnimAll(mnCharSel_804D6CC8, 0.0f32);
            stop_anim(mnCharSel_804D6CC8, ALL_TYPE_MASK);
        }
    }

    if (*mnCharSel_804D6CB0).match_type == 0x17 {
        let ck: u8;
        gobj = ext::GObj_Create(4, 5, 0x80);
        mnCharSel_804D6CC4 = ext::HSD_JObjLoadJoint((*css_models).door.joint);
        ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, mnCharSel_804D6CC4.cast());
        ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 1, 0x80);
        ext::HSD_GObj_SetupProc(gobj, Some(fn_8025FB2C), 4);
        add_model_anims(mnCharSel_804D6CC4, addr_of!((*css_models).door));
        // m-ex "Replace CSS Training Emblem": the CPU door's portrait is joint 6, emblem 4
        if !mnCharSel_Mex.is_null() {
            mnCharSel_MexAttach(mnCharSel_Mex, mnCharSel_804D6CC4, 6, 4);
        }
        ext::HSD_JObjReqAnimAll(mnCharSel_804D6CC4, 0.0f32);
        stop_anim(mnCharSel_804D6CC4, ALL_TYPE_MASK);
        ck = (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF1 as usize)).ckind as u8;
        // m-ex fighters' character kinds are past CKind_Playable_Count but playable
        if (((ck as i8) as i32 >= CKind_Playable_Count as i32) && (ck as u32) < ChKind_Mex0) || ext::gm_IsCKindUnlocked(ck) == 0 {
            loop {
                i = ext::HSD_Randi(MNCS_NUM());
                if (*icon_at(i as usize)).state != 0 {
                    break;
                }
            }
            let icon_off = getIconOffset(i as u32);
            (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF1 as usize)).ckind = (*icon_at(i as usize)).char_kind as i8;
            (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF1 as usize)).color =
                ext::HSD_Randi(ext::gm_GetNumCostumesForCKind((*icon_at(i as usize)).char_kind) as i32) as u8;
            let _ = icon_off;
        }
    }

    if (*mnCharSel_804D6CB0).match_type == 1 {
        gobj = ext::GObj_Create(4, 5, 0x80);
        mnCharSel_804D6CCC = ext::HSD_JObjLoadJoint((*css_models).debug_camera.joint);
        ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, mnCharSel_804D6CCC.cast());
        ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 4, 0x80);
        ext::HSD_GObj_SetupProc(gobj, Some(fn_8025FAC0), 4);
        add_model_anims(mnCharSel_804D6CCC, addr_of!((*css_models).debug_camera));
        ext::HSD_JObjReqAnimAll(mnCharSel_804D6CCC, 0.0f32);
    }

    i = mnCharSel_804D6CF5 as i32 - 1;
    while i >= 0 {
        let cursor_gobj = ext::GObj_Create(4, 5, 0x80);
        jobj = ext::HSD_JObjLoadJoint((*css_models).hand.joint);
        mnCharSel_MexScaleCursor(jobj, 1);
        let cursor = ext::HSD_MemAlloc(core::mem::size_of::<CSSCursorData>() as i32) as *mut CSSCursorData;
        ext::HSD_GObjObject_80390A70(cursor_gobj, HSD_GObj_JObjKind, jobj.cast());
        ext::GObj_SetupGXLink(cursor_gobj, Some(ext::HSD_GObj_JObjCallback), 3, 0x80);
        ext::HSD_GObj_SetupProc(cursor_gobj, Some(mnCharSel_CursorThink), 1);
        ext::GObj_InitUserData(cursor_gobj, 4, Some(ext::HSD_Free), cursor.cast());
        add_model_anims(jobj, addr_of!((*css_models).hand));
        ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
        ext::HSD_JObjAnimAll(jobj);
        stop_anim(jobj, ALL_TYPE_MASK);
        mnCharSel_804A0BC0[i as usize] = cursor;
        (*cursor).gobj = cursor_gobj;
        (*cursor).x4 = i as u8;
        (*cursor).x8 = 0;
        (*cursor).xA = 0;
        (*cursor).x7 = 0;
        (*cursor).x6 = 0;
        (*cursor).x5 = 2;
        (*cursor).xC = fmadds(15.0f32, i as f32, -31.0f32);
        (*cursor).x10 = -21.5f32;
        i -= 1;
    }

    i = 0;
    slot = 0;
    while i < num_players {
        {
            let model_gobj = ext::GObj_Create(4, 5, 0x80);
            jobj = ext::HSD_JObjLoadJoint((*css_models).token.joint);
            mnCharSel_MexScaleCursor(jobj, 0);
            {
                let player: i32;
                let model = ext::HSD_MemAlloc(core::mem::size_of::<CSSCharModel>() as i32) as *mut CSSCharModel;
                ext::HSD_GObjObject_80390A70(model_gobj, HSD_GObj_JObjKind, jobj.cast());
                ext::GObj_InitUserData(model_gobj, 4, Some(ext::HSD_Free), model.cast());
                ext::GObj_SetupGXLink(model_gobj, Some(ext::HSD_GObj_JObjCallback), 2, 0x80);
                ext::HSD_GObj_SetupProc(model_gobj, Some(fn_80262648), 2);
                add_model_anims(jobj, addr_of!((*css_models).token));
                ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
                {
                    let anim_jobj = jobj;
                    stop_anim(anim_jobj, TOBJ_MASK);
                }
                mnCharSel_804A0BD0[slot as usize] = model;
                (*model).gobj = model_gobj;
                (*model).x4 = i as u8;
                (*model).x5 = 0;
                (*model).x6 = 0xFF;
                (*model).x7 = 0;
                if mnCharSel_804D6CF5 == 1 {
                    if i != 0 {
                        player = mnCharSel_804D6CF1 as i32;
                    } else {
                        player = mnCharSel_804D6CF0 as i32;
                    }
                } else {
                    player = i;
                }
                found = 0;
                while found < MNCS_NUM() {
                    if (*player_at(mnCharSel_804D6CB0, player as usize)).ckind as i32 == (*icon_at(found as usize)).char_kind as i32
                        && ext::gm_IsCKindUnlocked((*player_at(mnCharSel_804D6CB0, player as usize)).ckind as u8) != 0
                    {
                        break;
                    }
                    found += 1;
                }
                if found >= MNCS_NUM() {
                    (*player_at(mnCharSel_804D6CB0, player as usize)).ckind = CKind_Playable_Count as i8;
                    let slot_type = addr_of_mut!((*player_at(mnCharSel_804D6CB0, player as usize)).slot_type);
                    if *slot_type == 1 {
                        *slot_type = 3;
                    }
                }
                (*door(slot as usize)).sel_icon = found as u8;
                (*model).x10 = 3.4f32 + (*icon_at(found as usize)).bound_l;
                (*model).x8 = (*model).x10;
                (*model).x14 = -3.0f32 + (*icon_at(found as usize)).bound_u;
                (*model).xC = (*model).x14;
            }
        }
        i += 1;
        slot += 1;
    }

    spE8 = mnCharSel_804DC580;
    i = 0;
    while i < num_players {
        let td: *mut CSSTagData;
        let player: i32;
        gobj = ext::GObj_Create(4, 5, 0x80);
        td = ext::HSD_MemAlloc(core::mem::size_of::<CSSTagData>() as i32) as *mut CSSTagData;
        (*tag_at(i as usize)).data = td;
        ext::GObj_InitUserData(gobj, 4, Some(ext::HSD_Free), td.cast());
        ext::HSD_GObj_SetupProc(gobj, Some(fn_802633B0), 4);
        (*td).use_tag = 0;
        (*td).timer = 0;
        (*td).state = 0;
        (*td).x8 = 0.0f32;
        (*td).scroll_amt = 0.0f32;
        (*td).scroll_force = 0.0f32;
        (*td).port = i as u8;
        (*td).text = ext::HSD_SisLib_803A6754(0, ctx);
        (*(*td).text).x4C = 1;
        (*(*td).text).default_fitting = 1;
        (*(*td).text).default_alignment = 1;
        text = (*td).text;
        text_font(text, 0.058f32, 0.055f32);
        if mnCharSel_804D6CF5 == 1 {
            if i != 0 {
                text = (*td).text;
                text_pos(text, 15.6f32, 20.2f32, 0.0f32);
            } else {
                let mut spA4: *mut HSD_JObj = null_mut();
                let tag_box_joint: u8 = mnCharSel_803F0EBC.tag_box_joint;
                lb_find(mnCharSel_804D6CC0, &mut spA4, tag_box_joint as i32);
                {
                    let anim_jobj = spA4;
                    req_anim_jobj_mask(anim_jobj, 2.0f32);
                }
                ext::HSD_JObjAnimAll(spA4);
                {
                    let anim_jobj = spA4;
                    stop_anim(anim_jobj, JOBJ_MASK);
                }
                sp108 = spA4;
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = (*td).text;
                text_pos(text, spEC.x - 0.1f32, 20.2f32, 0.0f32);
            }
            text = (*td).text;
            (*text).box_size_x = 200.0f32;
            (*text).box_size_y = 32.0f32;
            text_add((*td).text, 100.0f32, 0.0f32, c"\x81\x45\x81\x45\x81\x45\x81\x45".as_ptr());
        } else {
            let mut tag_name_jobj: *mut HSD_JObj = null_mut();
            let name_joint: u8 = (*tag_at(i as usize)).name_jointl;
            lb_find(mnCharSel_804D6CC0, &mut tag_name_jobj, name_joint as i32);
            {
                let anim_jobj = tag_name_jobj;
                req_anim_jobj_mask(anim_jobj, 2.0f32);
            }
            ext::HSD_JObjAnimAll(tag_name_jobj);
            {
                let anim_jobj = tag_name_jobj;
                stop_anim(anim_jobj, JOBJ_MASK);
            }
            sp108 = tag_name_jobj;
            ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
            {
                let sx = 0.5f32 + spEC.x;
                let sy = -0.4f32 - spEC.y;
                let sz = spEC.z;
                text = (*td).text;
                text_pos(text, sx, sy, sz);
            }
            text = (*td).text;
            (*text).box_size_x = 160.0f32;
            (*text).box_size_y = 32.0f32;
            text_add((*td).text, 80.0f32, 0.0f32, c"\x81\x45\x81\x45\x81\x45\x81\x45".as_ptr());
        }
        if mnCharSel_804D6CF5 == 1 {
            lb_find(mnCharSel_804D6CC0, &mut sp108, mnCharSel_803F0EBC.name_list_joint as i32);
        } else {
            lb_find(mnCharSel_804D6CC0, &mut sp108, (*tag_at(i as usize)).list_joint as i32);
            if i == 3 {
                super::hsd::HSD_JObjSetTranslateX(sp108, 16.7f32);
            }
        }
        ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
        (*td).name_ls = ext::HSD_SisLib_803A6754(0, ctx);
        (*(*td).name_ls).default_fitting = 1;
        text = (*td).name_ls;
        (*text).box_size_x = 154.0f32;
        (*text).box_size_y = 256.0f32;
        {
            let sy = (0.8f32 - spEC.y) - 1.0f32;
            let sx = spEC.x - 0.6f32;
            let sz = spEC.z;
            text = (*td).name_ls;
            text_pos(text, sx, sy, sz);
        }
        text = (*td).name_ls;
        text_font(text, 0.065f32, 0.065f32);
        (*(*td).name_ls).x4E = 1;
        (*(*td).name_ls).hidden = 1;
        text_add(
            (*td).name_ls,
            0.0f32,
            0.0f32,
            c"\x81\x45\x81\x45\x81\x45\x81\x45\x81\x45\x81\x45\x81\x45\x81\x45\x81\x45\x81\x45\x81\x45".as_ptr(),
        );
        let mut color = spE8;
        ext::HSD_SisLib_803A74F0((*td).name_ls, 0, &mut color);
        text_add((*td).name_ls, 0.0f32, 0.0f32, c"\x82\x6d\x82\x60\x82\x6c\x82\x64\x20\x82\x64\x82\x6d\x82\x73\x82\x71\x82\x78".as_ptr());
        let mut color2 = spE8;
        ext::HSD_SisLib_803A74F0((*td).name_ls, 1, &mut color2);
        found = 0;
        while found < 9 {
            text_add((*td).name_ls, 10.0f32, 0.0f32, c"\x81\x45\x81\x45\x81\x45\x81\x45".as_ptr());
            found += 1;
        }
        found = 0;
        while found < 0x78 {
            if ext::GetNameText((found as u8) as i32).is_null() {
                break;
            }
            found += 1;
        }
        (*td).next_tag = (found + 1) as u8;
        lb_find(mnCharSel_804D6CC0, &mut sp108, (*tag_at(i as usize)).x7 as i32);
        ext::HSD_JObjSetFlags(sp108, JOBJ_HIDDEN as u32);
        lb_find(mnCharSel_804D6CC0, &mut sp108, (*tag_at(i as usize)).kostar_text_joint as i32);
        if (*td).next_tag > 7 {
            ext::HSD_JObjClearFlags(sp108, JOBJ_HIDDEN as u32);
        } else {
            ext::HSD_JObjSetFlags(sp108, JOBJ_HIDDEN as u32);
        }
        if mnCharSel_804D6CF5 == 1 {
            if i != 0 {
                player = mnCharSel_804D6CF1 as i32;
            } else {
                player = mnCharSel_804D6CF0 as i32;
            }
        } else {
            player = i;
        }
        if player == mnCharSel_804D6CF9 as i32 && mnCharSel_804D6CF8 < (*td).next_tag {
            let mt: u8 = (*mnCharSel_804D6CB0).match_type;
            if clears_kos(mt) {
                *(*mnCharSel_804D6CB0).ko_counts.add(player as usize) = 0;
            }
            (*player_at(mnCharSel_804D6CB0, player as usize)).nametag = mnCharSel_804D6CF8.wrapping_sub(1);
            mnCharSel_804D6CF8 = (*td).next_tag;
        }
        if (*player_at(mnCharSel_804D6CB0, player as usize)).nametag != 0x78 {
            (*td).use_tag = 1;
            ext::HSD_SisLib_803A70A0((*td).text, 0, ext::GetNameText((*player_at(mnCharSel_804D6CB0, player as usize)).nametag as i32), &[]);
            (*(*td).text).default_kerning = 0;
        }
        i += 1;
    }

    if mnCharSel_804D6CF5 == 1 {
        let mut hide_extra = false;
        match (*mnCharSel_804D6CB0).match_type as u32 {
            REG_CLASSIC | REG_ADVENTURE | REG_ALLSTAR => {
                spE4 = mnCharSel_804DC584;
                spE0 = mnCharSel_804DC588;
                spDC = mnCharSel_804DC58C;
                hard_color = mnCharSel_804DC590;
                spD4 = mnCharSel_804DC594;
                {
                    let cpu_level: u8 = (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF0 as usize)).cpu_level;
                    mnCharSel_803F0EBC.cpu_level_shown = cpu_level;
                    mnCharSel_803F0EBC.cpu_level = cpu_level;
                }
                mnCharSel_803F0EBC.cpu_slider_x = 124.0f32;
                mnCharSel_803F0EBC.xd3 = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_803F0EBC.scroll_flag = 1;
                let xd3 = mnCharSel_803F0EBC.xd3;
                text = xd3;
                text_pos(text, -0.5f32, 9.0f32, 0.0f32);
                text = xd3;
                (*text).box_size_x = 248.0f32;
                (*text).box_size_y = 32.0f32;
                text = xd3;
                text_font(text, 0.08f32, 0.09f32);
                (*xd3).x4E = 1;
                (*xd3).default_kerning = 1;
                (*xd3).default_alignment = 1;
                (*xd3).x4C = 1;
                text_add(xd3, 0.0f32, 0.0f32, c"\x82\x75\x82\x64\x82\x71\x82\x78\x20\x82\x64\x82\x60\x82\x72\x82\x78".as_ptr());
                let mut color3 = spE4;
                ext::HSD_SisLib_803A74F0(xd3, 0, &mut color3);
                text_add(xd3, 0.0f32, 0.0f32, c"\x82\x64\x82\x60\x82\x72\x82\x78".as_ptr());
                let mut color4 = spE0;
                ext::HSD_SisLib_803A74F0(xd3, 1, &mut color4);
                text_add(xd3, 0.0f32, 0.0f32, c"\x82\x6d\x82\x6e\x82\x71\x82\x6c\x82\x60\x82\x6b".as_ptr());
                let mut color5 = spDC;
                ext::HSD_SisLib_803A74F0(xd3, 2, &mut color5);
                text_add(xd3, 0.0f32, 0.0f32, c"\x82\x67\x82\x60\x82\x71\x82\x63".as_ptr());
                let mut color6 = hard_color;
                ext::HSD_SisLib_803A74F0(xd3, 3, &mut color6);
                text_add(xd3, 0.0f32, 0.0f32, c"\x82\x75\x82\x64\x82\x71\x82\x78\x20\x82\x67\x82\x60\x82\x71\x82\x63".as_ptr());
                let mut color7 = spD4;
                ext::HSD_SisLib_803A74F0(xd3, 4, &mut color7);
                ext::HSD_GObjGXLink_803909D8((*xd3).entity, mnCharSel_804D6CBC);
                {
                    let css = mnCharSel_804D6CB0;
                    if (*css).match_type == 0xD {
                        lb_find(mnCharSel_804D6CC0, &mut sp108, 0x34);
                        ext::HSD_JObjSetFlagsAll(sp108, JOBJ_HIDDEN as u32);
                        lb_find(mnCharSel_804D6CC0, &mut sp108, 0x3D);
                        ext::HSD_JObjSetFlagsAll(sp108, JOBJ_HIDDEN as u32);
                        lb_find(mnCharSel_804D6CC0, &mut sp108, 0x45);
                        ext::HSD_JObjSetFlagsAll(sp108, JOBJ_HIDDEN as u32);
                        lb_find(mnCharSel_804D6CC0, &mut sp108, 0x46);
                        {
                            let hide_jobj = sp108;
                            ext::HSD_JObjSetFlagsAll(hide_jobj, JOBJ_HIDDEN as u32);
                        }
                    } else {
                        let mut sp70: *mut HSD_JObj = null_mut();
                        data2.stocks = (*player_at(css, mnCharSel_804D6CF0 as usize)).stocks as u8;
                        updateStockIcons(addr_of_mut!(data2), &mut sp70, 1);
                        num_players = data2.stocks as i32;
                        (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF0 as usize)).stocks = data2.stocks as i8;
                    }
                }
                lb_find(mnCharSel_804D6CC0, &mut sp108, 0x40);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CDC = text;
                {
                    let sx = 13.5f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.07f32, 0.055f32);
                (*text).default_alignment = 2;
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                lb_find(mnCharSel_804D6CC0, &mut sp108, 0x42);
                ext::HSD_JObjSetFlags(sp108, JOBJ_HIDDEN as u32);
                lb_find(mnCharSel_804D6CC0, &mut sp108, 0x41);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CE4 = text;
                {
                    let sx = 13.5f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.07f32, 0.055f32);
                (*text).default_alignment = 2;
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                {
                    let mut jobj43: *mut HSD_JObj = null_mut();
                    lb_find(mnCharSel_804D6CC0, &mut jobj43, 0x43);
                    req_anim(jobj43, TOBJ_MASK, 0.0f32);
                    ext::HSD_JObjAnimAll(jobj43);
                    stop_anim(jobj43, TOBJ_MASK);
                    sp108 = jobj43;
                }
                ext::HSD_JObjSetFlags(sp108, JOBJ_HIDDEN as u32);
            }
            STADIUM_TARGET => {
                lb_find(mnCharSel_804D6CC8, &mut sp108, 4);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CDC = text;
                {
                    let sx = 10.9f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                (*text).default_alignment = 2;
                text_font(text, 0.07f32, 0.055f32);
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CE0 = text;
                {
                    let sx = 11.6f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = 0.3f32 - spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.06f32, 0.045f32);
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                lb_find(mnCharSel_804D6CC8, &mut sp108, 5);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CE4 = text;
                {
                    let sx = 10.9f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.07f32, 0.055f32);
                (*text).default_alignment = 2;
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CE8 = text;
                {
                    let sx = 11.6f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = 0.3f32 - spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.06f32, 0.045f32);
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                lb_find(mnCharSel_804D6CC8, &mut sp108, 5);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_CStickText = text;
                {
                    let sx = 10.9f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y - 0.6f32;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.06f32, 0.045f32);
                (*text).default_alignment = 2;
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                lb_find(mnCharSel_804D6CC8, &mut sp108, 3);
                ext::HSD_JObjSetFlagsAll(sp108, JOBJ_HIDDEN as u32);
                hide_extra = true;
            }
            STADIUM_HOMERUN => {
                lb_find(mnCharSel_804D6CC8, &mut sp108, 4);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CDC = text;
                {
                    let sx = 11.0f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.07f32, 0.055f32);
                (*text).default_alignment = 2;
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                {
                    let sx = 12.0f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.07f32, 0.055f32);
                (*text).default_kerning = 1;
                if ext::lbLang_IsSavedLanguageJP() != 0 {
                    text_add(text, 0.0f32, 0.0f32, c"\x82\x8d".as_ptr());
                    text_add(text, 0.0f32, 35.0f32, c"\x82\x8d".as_ptr());
                } else {
                    text_add(text, 0.0f32, 0.0f32, c"\x82\x65\x82\x94\x81\x44".as_ptr());
                    text_add(text, 0.0f32, 35.0f32, c"\x82\x65\x82\x94\x81\x44".as_ptr());
                }
                lb_find(mnCharSel_804D6CC8, &mut sp108, 5);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CE4 = text;
                {
                    let sx = 11.0f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.07f32, 0.055f32);
                (*text).default_alignment = 2;
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                hide_extra = true;
            }
            STADIUM_MULTIMAN_10 | STADIUM_MULTIMAN_100 => {
                lb_find(mnCharSel_804D6CC8, &mut sp108, 4);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CDC = text;
                {
                    let sx = 10.9f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.07f32, 0.055f32);
                (*text).default_alignment = 2;
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CE0 = text;
                {
                    let sx = 11.6f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = 0.3f32 - spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.06f32, 0.045f32);
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                lb_find(mnCharSel_804D6CC8, &mut sp108, 5);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CE4 = text;
                {
                    let sx = 10.9f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.07f32, 0.055f32);
                (*text).default_alignment = 2;
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CE8 = text;
                {
                    let sx = 11.6f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = 0.3f32 - spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.06f32, 0.045f32);
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                lb_find(mnCharSel_804D6CC8, &mut sp108, 3);
                ext::HSD_JObjSetFlagsAll(sp108, JOBJ_HIDDEN as u32);
                hide_extra = true;
            }
            STADIUM_3_MIN_MELEE | STADIUM_15_MIN_MELEE | STADIUM_ENDLESS_MELEE | STADIUM_CRUEL_MELEE => {
                lb_find(mnCharSel_804D6CC8, &mut sp108, 4);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CDC = text;
                {
                    let sx = 16.0f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.07f32, 0.055f32);
                (*text).default_alignment = 2;
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                lb_find(mnCharSel_804D6CC8, &mut sp108, 5);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CE4 = text;
                {
                    let sx = 16.0f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.07f32, 0.055f32);
                (*text).default_alignment = 2;
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                hide_extra = true;
            }
            TRAINING_MODE => {
                lb_find(mnCharSel_804D6CC4, &mut sp108, 9);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A6754(0, ctx);
                mnCharSel_804D6CDC = text;
                {
                    let sx = 10.0f32 + spEC.x;
                    let sz = spEC.z;
                    let sy = -spEC.y;
                    text_pos(text, sx, sy, sz);
                }
                text_font(text, 0.13f32, 0.055f32);
                (*text).default_alignment = 2;
                ext::HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, null(), &[]);
                // fallthrough
                hide_extra = true;
            }
            EVENT_MATCH => {
                hide_extra = true;
            }
            _ => {}
        }
        if hide_extra {
            lb_find(mnCharSel_804D6CC0, &mut sp108, 0x2E);
            ext::HSD_JObjSetFlagsAll(sp108, JOBJ_HIDDEN as u32);
            lb_find(mnCharSel_804D6CC0, &mut sp108, 0x34);
            ext::HSD_JObjSetFlagsAll(sp108, JOBJ_HIDDEN as u32);
            lb_find(mnCharSel_804D6CC0, &mut sp108, 0x3A);
            ext::HSD_JObjSetFlagsAll(sp108, JOBJ_HIDDEN as u32);
            lb_find(mnCharSel_804D6CC0, &mut sp108, 0x44);
            ext::HSD_JObjSetFlagsAll(sp108, JOBJ_HIDDEN as u32);
        }
    } else {
        let mt: u8;
        mnCharSel_803F0EBC.scroll_flag = 0;
        text = ext::HSD_SisLib_803A5ACC(0, ctx, -12.0f32, -23.3f32, 0.0f32, 450.0f32, 32.0f32);
        (*text).default_alignment = 1;
        (*text).default_kerning = 1;
        (*text).default_fitting = 1;
        text_font(text, 0.07f32, 0.07f32);
        ext::HSD_SisLib_803A6368(text, 0x4A);
        mnCharSel_8025BD30();
        mt = (*mnCharSel_804D6CB0).match_type;
        if clears_kos(mt) {
            i = 0;
            while i < mnCharSel_804D6CF5 as i32 {
                lb_find(mnCharSel_804D6CC0, &mut sp108, (*ko_star_at(i as usize)).joint as i32);
                ext::lb_8000B1CC(sp108, null_mut(), &mut spEC);
                text = ext::HSD_SisLib_803A5ACC(0, ctx, spEC.x, -spEC.y - 0.9f32, spEC.z, 32.0f32, 32.0f32);
                (*ko_star_at(i as usize)).text = text;
                text_font(text, 0.07f32, 0.07f32);
                mnCharSel_8025D1C4(i, 0);
                i += 1;
            }
        } else {
            i = 0;
            while i < mnCharSel_804D6CF5 as i32 {
                lb_find(mnCharSel_804D6CC0, &mut sp108, (*ko_star_at(i as usize)).joint as i32);
                ext::HSD_JObjSetFlagsAll(sp108, JOBJ_HIDDEN as u32);
                i += 1;
            }
        }
    }

    gobj = ext::GObj_Create(4, 5, 0x80);
    jobj = ext::HSD_JObjLoadJoint((*css_models).press_start.joint);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 4, 0x80);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_80262F44), 3);
    add_model_anims(jobj, addr_of!((*css_models).press_start));
    ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
    stop_anim(jobj, ALL_TYPE_MASK);

    mnCharSel_804D6CF7 = 0;
    if mnCharSel_804D6CF5 == 1 {
        (*player_at(mnCharSel_804D6CB0, 3)).slot_type = 3;
        (*player_at(mnCharSel_804D6CB0, 2)).slot_type = 3;
        (*player_at(mnCharSel_804D6CB0, 1)).slot_type = 3;
        (*player_at(mnCharSel_804D6CB0, 0)).slot_type = 3;
        (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF0 as usize)).slot_type = 0;
        (*door(0)).p_kind = 0;
        'doors_done: {
            let css = mnCharSel_804D6CB0;
            (*door(0)).costume = (*player_at(css, mnCharSel_804D6CF0 as usize)).color;
            (*door(0)).sel_icon_prev = (*door(0)).sel_icon;
            if (*css).match_type != 0x17 {
                break 'doors_done;
            }
            (*player_at(css, mnCharSel_804D6CF1 as usize)).slot_type = 1;
            (*door(1)).p_kind = 1;
            (*door(1)).costume = (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF1 as usize)).color;
            (*door(1)).sel_icon_prev = (*door(1)).sel_icon;
        }
    } else {
        i = 0;
        while i < mnCharSel_804D6CF5 as i32 {
            let iu = i as usize;
            (*door(iu)).p_kind = (*player_at(mnCharSel_804D6CB0, iu)).slot_type;
            (*door(iu)).costume = (*player_at(mnCharSel_804D6CB0, iu)).color;
            (*door(iu)).p_kind_prev = 3;
            (*door(iu)).slideranim_timer = 0;
            (*door(iu)).dooranim_timer = 0;
            (*door(iu)).is_hold_handicap_slider = 0;
            (*door(iu)).is_hold_cpu_slider = 0;
            (*door(iu)).sel_icon_prev = (*door(iu)).sel_icon;
            let rules = ext::gmMainLib_GetGameRules();
            if (*rules).handicap != 0 {
                let mut hval: i32;
                lb_find(mnCharSel_804D6CC0, &mut sp108, (*door(iu)).cpuslider_joint as i32);
                if (*ext::gmMainLib_GetGameRules()).handicap == 1 {
                    hval = (ext::gm_801685D4(i as u8, (*player_at(mnCharSel_804D6CB0, iu)).nametag) as u8) as i32;
                } else {
                    hval = ((*player_at(mnCharSel_804D6CB0, iu)).handicap as u8) as i32;
                }
                // switch forces MWCC beq/b double-branch (not bne)
                if hval == 0 {
                    hval = 1;
                }
                {
                    let slider_x = 1.25f32 * (hval - 1) as f32;
                    super::hsd::HSD_JObjSetTranslateX(sp108, slider_x);
                }
                lb_find(mnCharSel_804D6CC0, &mut sp108, (*door(iu)).cpuslider2_joint as i32);
            } else {
                lb_find(mnCharSel_804D6CC0, &mut sp108, (*door(iu)).cpuslider_joint as i32);
            }
            {
                let cpu_level = addr_of_mut!((*player_at(mnCharSel_804D6CB0, iu)).cpu_level);
                if *cpu_level == 0 {
                    *cpu_level = 1;
                }
            }
            super::hsd::HSD_JObjSetTranslateX(sp108, 1.25f32 * ((*player_at(mnCharSel_804D6CB0, iu)).cpu_level as i32 - 1) as f32);
            i += 1;
        }
    }

    mnCharSel_8025EE8C((*mnCharSel_804D6CB0).match_type);
    ext::lbAudioAx_80023F28(ext::gmMainLib_8015ECB0())
}


// ------------------------------------------------------------------------------------------
// the scene
// ------------------------------------------------------------------------------------------

/// Scene entry: loads the MnSlChr / MnExtAll archives and the SIS text file, then builds the screen.
pub unsafe extern "C" fn mnCharSel_Scene_OnEnter(arg0: *mut c_void) {
    ext::lbCardNew_AllocWorkArea();
    ext::lbCardGame_LoadArchive(0);
    mnCharSel_804D6CB0 = arg0 as *mut CSSData;

    mnCharSel_804D6CF0 = ((*mnCharSel_804D6CB0).unk_0x0 as i32 - 1) as i8;

    mnCharSel_804D6CF8 = 0;
    while (mnCharSel_804D6CF8 as i32) < 0x78 {
        if ext::GetNameText(mnCharSel_804D6CF8 as i32).is_null() {
            break;
        }
        mnCharSel_804D6CF8 = mnCharSel_804D6CF8.wrapping_add(1);
    }
    mnCharSel_804D6CF9 = -1;
    mnCharSel_804D6CF8 = mnCharSel_804D6CF8.wrapping_add(1);

    (*door(0)).selected_since_load = 0;
    (*door(1)).selected_since_load = 0;
    (*door(2)).selected_since_load = 0;
    (*door(3)).selected_since_load = 0;

    ext::lbAudioAx_80026F2C(0x12);

    ext::lbAudioAx_8002702C(2, 8);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
    if ext::lbLang_IsSavedLanguageJP() != 0 {
        mnCharSel_804D6CD0 = ext::lbArchive_LoadArchive(c"MnSlChr.dat".as_ptr());
        mnCharSel_804D6CD4 = ext::lbArchive_LoadArchive(c"MnExtAll.dat".as_ptr());
    } else {
        mnCharSel_804D6CD0 = ext::lbArchive_LoadArchive(c"MnSlChr.usd".as_ptr());
        mnCharSel_804D6CD4 = ext::lbArchive_LoadArchive(c"MnExtAll.usd".as_ptr());
    }
    css_data_table = ext::HSD_ArchiveGetPublicAddress(mnCharSel_804D6CD0, c"MnSelectChrDataTable".as_ptr()) as *mut MnSelectChrDataTable;
    css_models = addr_of_mut!((*css_data_table).models);
    if ext::lbLang_IsSavedLanguageJP() != 0 {
        ext::HSD_SisLib_803A62A0(0, c"SdSlChr.dat".as_ptr() as *mut i8, c"SIS_SelCharData".as_ptr() as *mut i8);
    } else {
        ext::HSD_SisLib_803A62A0(0, c"SdSlChr.usd".as_ptr() as *mut i8, c"SIS_SelCharData".as_ptr() as *mut i8);
    }
    mnCharSel_804D6CF4 = 0;
    mnCharSel_802640A0();
}

/// Scene frame: keeps the preload cache current, handles leaving the scene (back / start / name
/// entry) and the per-state transitions.
pub unsafe extern "C" fn mnCharSel_Scene_OnFrame() {
    let num_slots: i32;
    let mut slot_type: u8;
    let cache: *mut GameCache;
    let mut i: i32;

    mnCharSel_804D6CEC = mnCharSel_804D6CEC.wrapping_add(1);
    if mnCharSel_804D6CF6 <= 1 {
        cache = addr_of_mut!((*ext::lbDvd_GetPreloadCacheScene()).game_cache);
        let entries = addr_of_mut!((*cache).entries) as *mut PreloadCacheSceneEntry;
        if mnCharSel_804D6CF5 == 1 {
            slot_type = (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF0 as usize)).slot_type;
            if !(slot_type != 0 && slot_type != 1) && (*mnCharSel_804A0BD0[0]).x5 == 0 {
                (*entries).char_id = (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF0 as usize)).ckind as i32;
                (*entries).color = (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF0 as usize)).color;
            } else {
                (*entries).char_id = ChKind_None as i32;
            }
            slot_type = (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF1 as usize)).slot_type;
            if !(slot_type != 0 && slot_type != 1) {
                if (*mnCharSel_804A0BD0[1]).x5 == 0 {
                    (*entries.add(1)).char_id = (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF1 as usize)).ckind as i32;
                    (*entries.add(1)).color = (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF1 as usize)).color;
                } else {
                    (*entries.add(1)).char_id = ChKind_None as i32;
                }
            }
        } else {
            if (*mnCharSel_804D6CB0).match_type as u32 == VS_CAMERA {
                num_slots = 3;
            } else {
                num_slots = 4;
            }

            i = 0;
            while i < num_slots {
                slot_type = (*player_at(mnCharSel_804D6CB0, i as usize)).slot_type;
                if !(slot_type != 0 && slot_type != 1) && (*mnCharSel_804A0BD0[i as usize]).x5 == 0 {
                    (*entries.add(i as usize)).char_id = (*player_at(mnCharSel_804D6CB0, i as usize)).ckind as i32;
                    (*entries.add(i as usize)).color = (*player_at(mnCharSel_804D6CB0, i as usize)).color;
                } else {
                    (*entries.add(i as usize)).char_id = ChKind_None as i32;
                }
                i += 1;
            }
        }
        ext::lbDvd_80018254();
    }
    if ext::mn_8022F218() != 0 {
        sfxBack();
        ext::lb_800145F4();
        ext::mn_8022F138(1, 8);
        ext::HSD_SisLib_803A5E70();
        ext::mn_8022F268();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        ext::gm_801A4B60();
        return;
    }
    if mnCharSel_804D6CF2 != 0 {
        mnCharSel_804D6CF2 -= 1;
    }
    match mnCharSel_804D6CF6 {
        1 => {
            i = 0;
            while i < 4 {
                if (*door(i as usize)).p_kind != 3 && (*mnCharSel_804A0BD0[0]).x5 != 0 {
                    mnCharSel_804D6CF6 = 0;
                }
                i += 1;
            }
            if mnCharSel_804D6CF6 != 0 {
                // ONLINE PLAY is picking: keep the chosen fighter's icon for its row
                {
                    // every fighter's icon, on every confirm: the loading screen and the online lobby
                    // show them after the CSS art is freed
                    let mut k: i32 = 0;
                    while k < MNCS_NUM() {
                        let mut aj: *mut HSD_JObj = null_mut();
                        lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut aj, (*icon_at(k as usize)).joint_id_vs as i32);
                        ext::Frontend_CaptureCharIcon((*icon_at(k as usize)).char_kind as i32, aj);
                        k += 1;
                    }
                }
                if ext::Frontend_OnlinePick() == 1 {
                    let mut p: i32 = 0;
                    while p < 4 {
                        let pl = player_at(mnCharSel_804D6CB0, p as usize);
                        if (*pl).slot_type as u32 != Gm_PKind_Human || (*pl).ckind as i32 == ChKind_None as i32 {
                            p += 1;
                            continue;
                        }
                        let mut k: i32 = 0;
                        while k < MNCS_NUM() {
                            if (*icon_at(k as usize)).char_kind as i32 == (*pl).ckind as i32 {
                                let mut ij: *mut HSD_JObj = null_mut();
                                lb_find(MNCS_ICON_ROOT(mnCharSel_804D6CC0), &mut ij, (*icon_at(k as usize)).joint_id_vs as i32);
                                ext::Frontend_CaptureIcon(0, ij);
                                break;
                            }
                            k += 1;
                        }
                        break;
                    }
                }
                ext::gm_801A4B60();
                if mnCharSel_804D6CF5 == 4 {
                    ext::lbAudioAx_8002411C(0x147);
                }
                // Ported from m-ex (https://github.com/akaneia/m-ex): asm/gameplay/Disable FoD in
                // Doubles.asm, @ 0x80266CE0. On the CSS->SSS transition, when the random stage select
                // mask is the default all-legal set (0xE70000B0) or that set with only Fountain of
                // Dreams toggled off (bit 0x20), force FoD off in teams and on in singles. Opt-in:
                // MELEE_MEX=disable_fod_in_doubles.
                if ext::Mex_Enabled(c"disable_fod_in_doubles".as_ptr()) != 0 {
                    let prefs = ext::gmMainLib_GetGamePrefs();
                    let mut mask: u32 = (*prefs).stage_mask;
                    let diff: u32 = mask ^ 0xE70000B0;
                    if diff == 0 || diff == 0x20 {
                        if (*mnCharSel_804D6CB0).vs.start.rules.is_teams == 1 {
                            mask &= !0x20u32;
                        } else {
                            mask |= 0x20u32;
                        }
                        (*prefs).stage_mask = mask;
                    }
                }
                sfxForward();
            }
        }
        2 => {
            ext::gm_801A4B60();
            sfxBack();
        }
        3 => {
            ext::mn_8022F138(1, 8);
            ext::HSD_SisLib_803A5E70();
            ext::mn_80231804(mnCharSel_804D6CD4, 1);
            mnCharSel_804D6CF6 = 5;
            sfxForward();
        }
        4 => {
            ext::mn_8022F138(1, 8);
            ext::HSD_SisLib_803A5E70();
            ext::mnNameNew_EnterFromMnCharSel(mnCharSel_804D6CD4, mnCharSel_804D6CF9 as i32);
            mnCharSel_804D6CF6 = 5;
            sfxForward();
        }
        _ => {}
    }
}

/// Scene exit: frees the archives and, when leaving for a match, preloads the chosen fighters'
/// sound banks.
pub unsafe extern "C" fn mnCharSel_Scene_OnExit(unused: *mut c_void) {
    let num_slots: i32;
    let mut tmp: u64;
    let mut i: i32;
    let mut type_: u8;

    ext::HSD_SisLib_803A5FBC();
    if !mnCharSel_804D6CD0.is_null() {
        ext::lbArchive_80016EFC(mnCharSel_804D6CD0);
        mnCharSel_804D6CD0 = null_mut();
    }
    if !mnCharSel_804D6CD4.is_null() {
        ext::lbArchive_80016EFC(mnCharSel_804D6CD4);
        mnCharSel_804D6CD4 = null_mut();
    }
    (*mnCharSel_804D6CB0).pending_scene_change = mnCharSel_804D6CF6;
    if mnCharSel_804D6CF6 != 0 {
        return;
    }

    tmp = 0;
    if mnCharSel_804D6CF5 == 1 {
        type_ = (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF0 as usize)).slot_type;
        if (type_ as u32 == Gm_PKind_Human || type_ as u32 == Gm_PKind_Cpu) && (*mnCharSel_804A0BD0[0]).x5 == 0 {
            tmp |= ext::lbAudioAx_80026E84((*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF0 as usize)).ckind as i32 as u32);
        }
        type_ = (*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF1 as usize)).slot_type;
        if (type_ as u32 == Gm_PKind_Human || type_ as u32 == Gm_PKind_Cpu) && (*mnCharSel_804A0BD0[1]).x5 == 0 {
            tmp |= ext::lbAudioAx_80026E84((*player_at(mnCharSel_804D6CB0, mnCharSel_804D6CF1 as usize)).ckind as i32 as u32);
        }
    } else {
        num_slots = if (*mnCharSel_804D6CB0).match_type as u32 == VS_CAMERA { 3 } else { 4 };
        i = 0;
        while i < num_slots {
            type_ = (*player_at(mnCharSel_804D6CB0, i as usize)).slot_type;
            if (type_ as u32 == Gm_PKind_Human || type_ as u32 == Gm_PKind_Cpu) && (*mnCharSel_804A0BD0[i as usize]).x5 == 0 {
                tmp |= ext::lbAudioAx_80026E84((*player_at(mnCharSel_804D6CB0, i as usize)).ckind as i32 as u32);
            }
            i += 1;
        }
    }

    ext::lbAudioAx_80026F2C(0x14);
    ext::lbAudioAx_8002702C(4, tmp);
    ext::lbAudioAx_80027168();
}

// ------------------------------------------------------------------------------------------
// the port's own character select (gmfrontend_select.inc)
//
// The frontend draws its CSS itself, with the disc's own art: this loads the icon model and the
// portrait joint out of the MnSlChr archive the frontend opened, the same models this screen uses
// (retail's menu model, or m-ex's mexSelectChr icon model + CSP animation), animates them with no
// GObj - nothing is drawn - and hands back the texture objects the animation selected. The frontend
// only borrows their image and palette pointers, which live in the archive.
// ------------------------------------------------------------------------------------------

/// The menu model: retail icons, the portrait joint.
static mut mnCharSel_PcMenu: *mut HSD_JObj = null_mut();
/// Where icon joints resolve (m-ex's icon model or the menu).
static mut mnCharSel_PcIcons: *mut HSD_JObj = null_mut();
static mut mnCharSel_PcMex: *mut MexSelectChr = null_mut();
static mut mnCharSel_PcTable: *mut CSSIcon = null_mut();
/// Retail's cursor hand and its coin (token), animated by frame only, never drawn.
static mut mnCharSel_PcHand: *mut HSD_JObj = null_mut();
static mut mnCharSel_PcToken: *mut HSD_JObj = null_mut();
static mut mnCharSel_PcCount: i32 = 0;

/// Release the models before the caller releases their backing archive.
pub unsafe fn mnCharSel_PcArtClose() {
    if !mnCharSel_PcIcons.is_null() && mnCharSel_PcIcons != mnCharSel_PcMenu {
        ext::HSD_JObjRemoveAll(mnCharSel_PcIcons);
    }
    if !mnCharSel_PcMenu.is_null() {
        ext::HSD_JObjRemoveAll(mnCharSel_PcMenu);
    }
    if !mnCharSel_PcHand.is_null() {
        ext::HSD_JObjRemoveAll(mnCharSel_PcHand);
    }
    if !mnCharSel_PcToken.is_null() {
        ext::HSD_JObjRemoveAll(mnCharSel_PcToken);
    }
    mnCharSel_PcHand = null_mut();
    mnCharSel_PcToken = null_mut();
    mnCharSel_PcMenu = null_mut();
    mnCharSel_PcIcons = null_mut();
    mnCharSel_PcMex = null_mut();
    mnCharSel_PcTable = null_mut();
    mnCharSel_PcCount = 0;
}

/// The biggest colour texture under `j` (its own subtree) - an icon's art, not its I4 plate.
unsafe fn mnCharSel_PcBestTex(mut j: *mut HSD_JObj, best: *mut *mut HSD_TObj, area: *mut i32, depth: i32) {
    while !j.is_null() && depth < 12 {
        let mut d = ext::HSD_JObjGetDObj(j);
        while !d.is_null() {
            let mut t: *mut HSD_TObj = if !(*d).mobj.is_null() { (*(*d).mobj).tobj } else { null_mut() };
            while !t.is_null() {
                if !(*t).imagedesc.is_null() && !(*(*t).imagedesc).image_ptr.is_null() {
                    let fmt = (*(*t).imagedesc).format as i32;
                    let a = (*(*t).imagedesc).width as i32 * (*(*t).imagedesc).height as i32 * (if fmt >= 4 { 4 } else { 1 });
                    if a > *area {
                        *area = a;
                        *best = t;
                    }
                }
                t = (*t).next;
            }
            d = (*d).next;
        }
        mnCharSel_PcBestTex(jobj_child(j), best, area, depth + 1);
        j = if depth == 0 { null_mut() } else { jobj_next(j) };
    }
}

/// The first texture of joint `part`'s own meshes (not its children).
unsafe fn mnCharSel_PcOwnTex(j: *mut HSD_JObj) -> *mut HSD_TObj {
    let mut d = if !j.is_null() { ext::HSD_JObjGetDObj(j) } else { null_mut() };
    while !d.is_null() {
        let mut t: *mut HSD_TObj = if !(*d).mobj.is_null() { (*(*d).mobj).tobj } else { null_mut() };
        while !t.is_null() {
            if !(*t).imagedesc.is_null() && !(*(*t).imagedesc).image_ptr.is_null() {
                return t;
            }
            t = (*t).next;
        }
        d = (*d).next;
    }
    null_mut()
}

/// The last part's TEV colour 0 (retail's hand: the navy its shading blends from).
static mut mnCharSel_PcCursorLo: u32 = 0;
pub unsafe fn mnCharSel_PcArtCursorLo() -> u32 {
    mnCharSel_PcCursorLo
}

/// Retail's cursor art. `which` 0: the hand (part 2 its pose, frame = retail's x5 state; part 3 its
/// port tag, frame = colour + port * 4), 1: the coin (part 4, frame = port * 4, 16 CPU). The image
/// and palette at that frame, and the joint's translation and scale (retail units) so the caller
/// can place the parts as retail does. 0 when the disc has no such art.
pub unsafe fn mnCharSel_PcArtCursor(
    which: i32,
    part: i32,
    frame: f32,
    img: *mut *mut HSD_ImageDesc,
    tlut: *mut *mut HSD_Tlut,
    geo: *mut f32,
    rgba: *mut u32,
) -> i32 {
    let root = if which == 0 { mnCharSel_PcHand } else { mnCharSel_PcToken };
    let j: *mut HSD_JObj;
    let t: *mut HSD_TObj;
    *img = null_mut();
    *tlut = null_mut();
    if root.is_null() {
        return 0;
    }
    j = animateJoint(root, part as u8, TOBJ_MASK | MOBJ_MASK, frame);
    t = mnCharSel_PcOwnTex(j);
    if !rgba.is_null() {
        // retail's colour at that frame: the texture's TEV constant (the port tags, the coin), else
        // the material's diffuse
        let d = if !j.is_null() { ext::HSD_JObjGetDObj(j) } else { null_mut() };
        *rgba = 0xFFFFFFFFu32;
        let mut body = d;
        while !body.is_null() && !(*body).mobj.is_null() && !(*(*body).mobj).tobj.is_null() {
            body = (*body).next;
        }
        if which == 1 && part == 3 && !body.is_null() && !(*body).mobj.is_null() && !(*(*body).mobj).mat.is_null() {
            // the coin's untextured body: its animated colour
            let c = (*(*(*body).mobj).mat).diffuse;
            *rgba = ((c.r as u32) << 24) | ((c.g as u32) << 16) | ((c.b as u32) << 8) | 0xFFu32;
        } else if !t.is_null() && !(*t).tev.is_null() {
            let k = (*(*t).tev).konst;
            let l = (*(*t).tev).tev0;
            mnCharSel_PcCursorLo = ((l.r as u32) << 24) | ((l.g as u32) << 16) | ((l.b as u32) << 8) | 0xFFu32;
            *rgba = ((k.r as u32) << 24) | ((k.g as u32) << 16) | ((k.b as u32) << 8) | 0xFFu32;
        } else if !d.is_null() && !(*d).mobj.is_null() && !(*(*d).mobj).mat.is_null() {
            let c = (*(*(*d).mobj).mat).diffuse;
            let a: f32 = (*(*(*d).mobj).mat).alpha;
            *rgba = ((c.r as u32) << 24)
                | ((c.g as u32) << 16)
                | ((c.b as u32) << 8)
                | (if a <= 0.0f32 {
                    0
                } else if a >= 1.0f32 {
                    255
                } else {
                    f2i32((a * 255.0f32) as f64)
                }) as u32;
        }
    }
    mnCharSel_PcImage(t, img, tlut);
    if !geo.is_null() && !j.is_null() {
        *geo.add(0) = HSD_JObjGetTranslationX(j);
        *geo.add(1) = HSD_JObjGetTranslationY(j);
        *geo.add(2) = super::hsd::HSD_JObjGetScaleX(j);
        *geo.add(3) = super::hsd::HSD_JObjGetScaleY(j);
    }
    (!t.is_null()) as i32
}

/// Once per open: what the hand and coin models hold (the log line is the map for
/// `mnCharSel_PcArtCursor`).
unsafe fn mnCharSel_PcCursorProbe() {
    let mut w: i32 = 0;
    while w < 2 {
        let root = if w == 0 { mnCharSel_PcHand } else { mnCharSel_PcToken };
        let mut k: i32 = 0;
        while !root.is_null() && k < 8 {
            let mut j: *mut HSD_JObj = null_mut();
            lb_find(root, &mut j, k);
            if j.is_null() {
                break;
            }
            let t = mnCharSel_PcOwnTex(j);
            ext::OSReport(
                c"frontend: css cursor art %s part %d: t (%.2f %.2f %.2f) s (%.2f %.2f) tex %dx%d fmt %d\n".as_ptr(),
                &[
                    Va::P(if w == 0 { c"hand".as_ptr() } else { c"coin".as_ptr() }.cast()),
                    Va::I32(k),
                    Va::D(HSD_JObjGetTranslationX(j) as f64),
                    Va::D(HSD_JObjGetTranslationY(j) as f64),
                    Va::D(HSD_JObjGetTranslationZ(j) as f64),
                    Va::D(super::hsd::HSD_JObjGetScaleX(j) as f64),
                    Va::D(super::hsd::HSD_JObjGetScaleY(j) as f64),
                    Va::I32(if !t.is_null() { (*(*t).imagedesc).width as i32 } else { 0 }),
                    Va::I32(if !t.is_null() { (*(*t).imagedesc).height as i32 } else { 0 }),
                    Va::I32(if !t.is_null() { (*(*t).imagedesc).format as i32 } else { -1 }),
                ],
            );
            k += 1;
        }
        w += 1;
    }
}

/// Open the art: `archive` is MnSlChr, loaded by the caller for the scene. Returns how many icons
/// the roster has (retail 25, m-ex mexData's count), 0 on failure.
pub unsafe fn mnCharSel_PcArtOpen(archive: *mut HSD_Archive) -> i32 {
    let tbl: *mut MnSelectChrDataTable;
    let m: *mut MnSelectChrModels;
    let n: i32;
    mnCharSel_PcMenu = null_mut();
    mnCharSel_PcIcons = null_mut();
    mnCharSel_PcMex = null_mut();
    mnCharSel_PcTable = icon_at(0);
    mnCharSel_PcCount = 0;
    if archive.is_null() {
        return 0;
    }
    tbl = ext::HSD_ArchiveGetPublicAddress(archive, c"MnSelectChrDataTable".as_ptr()) as *mut MnSelectChrDataTable;
    if tbl.is_null() {
        return 0;
    }
    m = addr_of_mut!((*tbl).models);
    mnCharSel_PcMenu = ext::HSD_JObjLoadJoint((*m).menu.joint);
    add_model_anims(mnCharSel_PcMenu, addr_of!((*m).menu));
    ext::HSD_JObjReqAnimAll(mnCharSel_PcMenu, 0.0f32);
    ext::HSD_JObjAnimAll(mnCharSel_PcMenu);
    mnCharSel_PcIcons = mnCharSel_PcMenu;
    mnCharSel_PcHand = if !(*m).hand.joint.is_null() { ext::HSD_JObjLoadJoint((*m).hand.joint) } else { null_mut() };
    if !mnCharSel_PcHand.is_null() {
        add_model_anims(mnCharSel_PcHand, addr_of!((*m).hand));
        ext::HSD_JObjReqAnimAll(mnCharSel_PcHand, 0.0f32);
        ext::HSD_JObjAnimAll(mnCharSel_PcHand);
    }
    mnCharSel_PcToken = if !(*m).token.joint.is_null() { ext::HSD_JObjLoadJoint((*m).token.joint) } else { null_mut() };
    if !mnCharSel_PcToken.is_null() {
        add_model_anims(mnCharSel_PcToken, addr_of!((*m).token));
        ext::HSD_JObjReqAnimAll(mnCharSel_PcToken, 0.0f32);
        ext::HSD_JObjAnimAll(mnCharSel_PcToken);
    }
    mnCharSel_PcCursorProbe();
    mnCharSel_PcCount = SELKIND_COUNT as i32; // retail: icons[] as authored (never rewritten there)

    mnCharSel_PcMex = ext::HSD_ArchiveGetPublicAddress(archive, c"mexSelectChr".as_ptr()) as *mut MexSelectChr;
    n = if !mnCharSel_PcMex.is_null() { ext::Mex_CssIconCount() } else { 0 };
    if n > 0 && n <= CSS_ICON_MAX as i32 && !ext::Mex_CssIconTable().is_null() {
        mnCharSel_PcTable = ext::Mex_CssIconTable() as *mut CSSIcon; // char_kind is the EXTERNAL id here
        mnCharSel_PcCount = n;
        mnCharSel_PcIcons = ext::HSD_JObjLoadJoint((*mnCharSel_PcMex).icon_joint);
        ext::HSD_JObjAddAnimAll(mnCharSel_PcIcons, (*mnCharSel_PcMex).icon_animjoint, (*mnCharSel_PcMex).icon_matanim, null_mut());
        ext::HSD_JObjReqAnimAll(mnCharSel_PcIcons, 0.0f32);
        ext::HSD_JObjAnimAll(mnCharSel_PcIcons);
        {
            let mut j: *mut HSD_JObj = null_mut();
            lb_find(mnCharSel_PcMenu, &mut j, (*door(0)).costume_joint as i32);
            if !j.is_null() && !(*j).u.dobj.is_null() && !(*mnCharSel_PcMex).csp_matanim.is_null() {
                ext::HSD_DObjAddAnimAll((*j).u.dobj, (*mnCharSel_PcMex).csp_matanim, null_mut());
            }
        }
    } else {
        mnCharSel_PcMex = null_mut();
    }
    mnCharSel_PcCount
}

/// A texture object's current image and palette (an animated palette is tluttbl[tlut_no]).
unsafe fn mnCharSel_PcImage(t: *mut HSD_TObj, img: *mut *mut HSD_ImageDesc, tlut: *mut *mut HSD_Tlut) {
    *img = if !t.is_null() { (*t).imagedesc } else { null_mut() };
    *tlut = if t.is_null() {
        null_mut()
    } else if (*t).tlut_no != TOBJ_TLUT_NONE && !(*t).tluttbl.is_null() {
        *(*t).tluttbl.add((*t).tlut_no as usize)
    } else {
        (*t).tlut
    };
}

/// Icon `i` of the roster: its CharacterKind (-1 = the port has no such fighter), its m-ex external
/// id (-1 retail), and the texture showing it. Returns 0 past the end.
pub unsafe fn mnCharSel_PcArtIcon(i: i32, ck: *mut i32, ext_id: *mut i32, img: *mut *mut HSD_ImageDesc, tlut: *mut *mut HSD_Tlut) -> i32 {
    let row: *mut CSSIcon;
    let mut j: *mut HSD_JObj = null_mut();
    let mut t: *mut HSD_TObj = null_mut();
    let mut area: i32 = 0;
    if i < 0 || i >= mnCharSel_PcCount || mnCharSel_PcIcons.is_null() {
        return 0;
    }
    row = mnCharSel_PcTable.add(i as usize);
    if !mnCharSel_PcMex.is_null() {
        *ext_id = (*row).char_kind as i32;
        *ck = ext::Mex_ExtToPortCKind((*row).char_kind as i32);
    } else {
        *ext_id = -1;
        *ck = (*row).char_kind as i32;
    }
    lb_find(mnCharSel_PcIcons, &mut j, (*row).joint_id_vs as i32);
    if !j.is_null() {
        mnCharSel_PcBestTex(j, &mut t, &mut area, 0);
    }
    mnCharSel_PcImage(t, img, tlut);
    1
}

/// The portrait (CSP) of fighter `ck` in `costume`: the portrait joint animated to its frame.
pub unsafe fn mnCharSel_PcArtPortrait(mut ck: i32, mut costume: i32, img: *mut *mut HSD_ImageDesc, tlut: *mut *mut HSD_Tlut) -> i32 {
    if ext::Geno_DefineBaseCK(ck) >= 0 {
        ck = ext::Geno_DefineArtBaseCK(ck);
    }
    let j: *mut HSD_JObj;
    let mut t: *mut HSD_TObj = null_mut();
    let mut frame: i32 = -1;
    let mut area: i32 = 0;
    *img = null_mut();
    *tlut = null_mut();
    if mnCharSel_PcMenu.is_null() || ck < 0 {
        return 0;
    }
    if costume < 0 {
        costume = 0;
    }
    if !mnCharSel_PcMex.is_null() {
        let saved = mnCharSel_Mex;
        let ext_kind = ext::Mex_PortCKindToExt(ck);
        if ext_kind < 0 {
            return 0;
        }
        mnCharSel_Mex = mnCharSel_PcMex; // the frame rule reads the CSS's descriptor
        frame = mnCharSel_MexCspFrame(ext_kind, costume);
        mnCharSel_Mex = saved;
    } else {
        // retail's CSS has no Sheik: she shares Zelda's slot, so her card shows Zelda's
        let want: i32 = if ck == CKind_Seak as i32 { CKind_Zelda as i32 } else { ck };
        let mut i: i32 = 0;
        while i < SELKIND_COUNT as i32 {
            if (*icon_at(i as usize)).char_kind as i32 == want {
                // The retail CSP atlas has six rows; extra skins use its default.
                frame = (*icon_at(i as usize)).ft_hudindex as i32 + (if costume < 6 { costume } else { 0 }) * 0x1E;
                break;
            }
            i += 1;
        }
        if frame < 0 {
            return 0;
        }
    }
    j = animateJoint(mnCharSel_PcMenu, (*door(0)).costume_joint, TOBJ_MASK, frame as f32);
    if !j.is_null() {
        mnCharSel_PcBestTex(j, &mut t, &mut area, 0);
    }
    mnCharSel_PcImage(t, img, tlut);
    (!t.is_null()) as i32
}

