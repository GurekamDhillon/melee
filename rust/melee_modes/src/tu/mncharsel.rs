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
use melee_runtime::cabi::{f2i32, srem};
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
        fn Frontend_CaptureCharIcon(a0: i32, a1: *mut HSD_JObj) -> ();
        fn Frontend_CaptureIcon(a0: i32, a1: *mut HSD_JObj) -> ();
        fn Frontend_OnlinePick() -> i32;
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
        fn GetNameText(a0: i32) -> *mut i8;
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
        fn HSD_AObjReqAnim(a0: *mut HSD_AObj, a1: f32) -> ();
        fn HSD_AObjStopAnim(a0: *mut HSD_AObj, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut HSD_ObjData)>) -> ();
        fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void;
        fn HSD_CObjLoadDesc(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj;
        fn HSD_DObjAddAnimAll(a0: *mut HSD_DObj, a1: *mut _HSD_MatAnim, a2: *mut HSD_ShapeAnimDObj) -> ();
        fn HSD_FogLoadDesc(a0: *mut HSD_FogDesc) -> *mut HSD_Fog;
        fn HSD_FogSet(a0: *mut HSD_Fog) -> ();
        fn HSD_Free(a0: *mut c_void) -> ();
        fn HSD_GObjGXLink_803909D8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> ();
        fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) -> ();
        fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) -> ();
        fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) -> ();
        fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) -> ();
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint) -> ();
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj) -> ();
        fn HSD_JObjClearFlags(a0: *mut HSD_JObj, a1: u32) -> ();
        fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) -> ();
        fn HSD_JObjGetDObj(a0: *mut HSD_JObj) -> *mut HSD_DObj;
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj;
        fn HSD_JObjRemoveAll(a0: *mut HSD_JObj) -> ();
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32) -> ();
        fn HSD_JObjSetFlags(a0: *mut HSD_JObj, a1: u32) -> ();
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) -> ();
        fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) -> ();
        fn HSD_LObjLoadDesc(a0: *mut HSD_LightDesc) -> *mut HSD_LObj;
        fn HSD_MObjSetAlpha(a0: *mut HSD_MObj, a1: f32) -> ();
        fn HSD_MemAlloc(a0: i32) -> *mut c_void;
        fn HSD_Randf() -> f32;
        fn HSD_Randi(a0: i32) -> i32;
        fn HSD_SObjLib_803A477C(a0: *mut HSD_GObj, a1: *mut HSD_SObjDesc, a2: u32, a3: u32, a4: u8, a5: u8) -> *mut HSD_SObj;
        fn HSD_SObjLib_803A49E0(a0: *mut HSD_GObj, a1: i32) -> ();
        fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text;
        fn HSD_SisLib_803A5E70() -> ();
        fn HSD_SisLib_803A5FBC() -> ();
        fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32;
        fn HSD_SisLib_803A62A0(a0: i32, a1: *mut i8, a2: *mut i8) -> ();
        fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) -> ();
        fn HSD_SisLib_803A6478(a0: *mut u8, a1: *mut u8) -> *mut u8;
        fn HSD_SisLib_803A6530(a0: i32, a1: i32, a2: i32) -> *mut u8;
        fn HSD_SisLib_803A660C(a0: i32, a1: i32, a2: i32) -> ();
        fn HSD_SisLib_803A6754(a0: i32, a1: i32) -> *mut HSD_Text;
        fn HSD_SisLib_803A746C(a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32) -> ();
        fn HSD_SisLib_803A74F0(a0: *mut HSD_Text, a1: i32, a2: *mut _GXColor) -> ();
        fn IsNameListFull() -> i32;
        fn MTXOrtho(a0: *mut c_void, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> ();
        fn Mex_CssCursorScaleEnabled() -> i32;
        fn Mex_CssIconCount() -> i32;
        fn Mex_CssIconTable() -> *mut c_void;
        fn Mex_CssTraceEnabled() -> i32;
        fn Mex_Enabled(a0: *const i8) -> i32;
        fn Mex_ExtToPortCKind(a0: i32) -> i32;
        fn Mex_InsigniaForExt(a0: i32) -> i32;
        fn Mex_MenuParamF(a0: i32) -> f32;
        fn Mex_PortCKindToExt(a0: i32) -> i32;
        fn Player_800325C8(a0: u32, a1: i32) -> i8;
        fn atan2f(a0: f32, a1: f32) -> f32;
        fn atanf(a0: f32) -> f32;
        fn cosf(a0: f32) -> f32;
        fn gmMainLib_8015D0D8(a0: u8) -> i32;
        fn gmMainLib_8015D194(a0: u8) -> *mut u8;
        fn gmMainLib_8015D200(a0: u8) -> i32;
        fn gmMainLib_8015D2BC(a0: u8) -> *mut u8;
        fn gmMainLib_8015D328(a0: u8) -> i32;
        fn gmMainLib_8015D3E4(a0: u8) -> *mut u8;
        fn gmMainLib_8015ECB0() -> i32;
        fn gmMainLib_GetGamePrefs() -> *mut GamePrefs;
        fn gmMainLib_GetGameRules() -> *mut GameRules;
        fn gm_80160980(a0: u8) -> *const i8;
        fn gm_80162BD8(a0: u8) -> i32;
        fn gm_80162C48() -> i32;
        fn gm_80162D1C() -> i32;
        fn gm_80162D6C() -> u8;
        fn gm_80162DD4(a0: u8) -> i32;
        fn gm_80162E44() -> i32;
        fn gm_80162F18() -> i32;
        fn gm_80162F68() -> u8;
        fn gm_80162FD0(a0: u8) -> i32;
        fn gm_80163040() -> i32;
        fn gm_80163114() -> i32;
        fn gm_80163164() -> u8;
        fn gm_801631CC(a0: u8) -> u32;
        fn gm_801631F0() -> u32;
        fn gm_80163274(a0: u8) -> u16;
        fn gm_8016332C(a0: u8) -> i32;
        fn gm_80163374(a0: u8, a1: *mut u8, a2: *mut u8, a3: *mut u8, a4: *mut u8) -> ();
        fn gm_801634D4(a0: *mut u8, a1: *mut u8, a2: *mut u8, a3: *mut u8) -> i32;
        fn gm_8016365C(a0: u8) -> i32;
        fn gm_80163690(a0: u8) -> i32;
        fn gm_801636D8(a0: u8, a1: *mut u8, a2: *mut u8, a3: *mut u8, a4: *mut u8) -> ();
        fn gm_80163838(a0: *mut u8, a1: *mut u8, a2: *mut u8, a3: *mut u8) -> i32;
        fn gm_801639C0(a0: u8) -> i32;
        fn gm_801639F4(a0: u8) -> i32;
        fn gm_80163A3C(a0: u8, a1: *mut u8, a2: *mut u8, a3: *mut u8, a4: *mut u8) -> ();
        fn gm_80163B9C(a0: *mut u8, a1: *mut u8, a2: *mut u8, a3: *mut u8) -> i32;
        fn gm_801685D4(a0: u8, a1: u8) -> i8;
        fn gm_80168C5C(a0: u32) -> ();
        fn gm_80169264(a0: u8) -> u8;
        fn gm_80169290(a0: u8) -> u8;
        fn gm_801692BC(a0: u8) -> u8;
        fn gm_801A4B60() -> ();
        fn gm_ChangeGameModeAfterCurrentScene(a0: i32) -> ();
        fn gm_Get15MinMultimanHighscore(a0: u8) -> u16;
        fn gm_Get15MinMultimanTotalHighscore() -> i32;
        fn gm_Get3MinMultimanHighscore(a0: u8) -> u16;
        fn gm_Get3MinMultimanTotalHighscore() -> i32;
        fn gm_GetCruelHighscore(a0: u8) -> i32;
        fn gm_GetCruelTotalHighscore() -> i32;
        fn gm_GetEndlessHighscore(a0: u8) -> i32;
        fn gm_GetEndlessTotalHighscore() -> i32;
        fn gm_GetNumCostumesForCKind(a0: u8) -> u8;
        fn gm_IsCKindUnlocked(a0: u8) -> i32;
        fn gm_MexStockFrame(a0: i32, a1: i32) -> f32;
        fn gm_MexStockMatAnim() -> *mut HSD_MatAnimJoint;
        fn gm_RumbleEnabledForPlayer(a0: i32, a1: i32) -> i32;
        fn lbArchive_80016EFC(a0: *mut HSD_Archive) -> ();
        fn lbArchive_LoadArchive(a0: *const i8) -> *mut HSD_Archive;
        fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32;
        fn lbAudioAx_80023870(a0: i32, a1: i32, a2: i32, a3: i32) -> i32;
        fn lbAudioAx_80023F28(a0: i32) -> i32;
        fn lbAudioAx_80024030(a0: i32) -> ();
        fn lbAudioAx_8002411C(a0: i32) -> i32;
        fn lbAudioAx_80026E84(a0: u32) -> u64;
        fn lbAudioAx_80026F2C(a0: u32) -> ();
        fn lbAudioAx_8002702C(a0: u32, a1: u64) -> ();
        fn lbAudioAx_80027168() -> ();
        fn lbAudioAx_80027648() -> ();
        fn lbCardGame_LoadArchive(a0: i32) -> ();
        fn lbCardNew_AllocWorkArea() -> ();
        fn lbDvd_8001819C(a0: *const i8) -> *mut HSD_Archive;
        fn lbDvd_80018254() -> ();
        fn lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState;
        fn lbLang_IsSavedLanguageJP() -> i32;
        fn lbLang_IsSavedLanguageUS() -> i32;
        fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec3, a2: *mut Vec3) -> ();
        fn lb_80014574(a0: u8, a1: i32, a2: i32, a3: i32) -> ();
        fn lb_800145F4() -> ();
        fn mnNameNew_EnterFromMnCharSel(a0: *mut HSD_Archive, a1: i32) -> ();
        fn mn_8022BA1C(a0: *mut HSD_GObj) -> ();
        fn mn_8022F138(a0: u16, a1: u16) -> ();
        fn mn_8022F218() -> i32;
        fn mn_8022F268() -> ();
        fn mn_80231804(a0: *mut HSD_Archive, a1: i32) -> ();
        fn sinf(a0: f32) -> f32;
    }
    crate::ext_va! {
        /// hsd_3A64.c `HSD_SisLib_803A70A0(text, kind, fmt, ...)`: set a text's string (printf-style).
        fn HSD_SisLib_803A70A0(text: *mut HSD_Text, kind: i32, fmt: *const i8) va 2 -> i32;
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
jget!(HSD_JObjGetTranslationX, translate.x, 580);
jget!(HSD_JObjGetTranslationY, translate.y, 586);
jget!(HSD_JObjGetTranslationZ, translate.z, 592);

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
pub unsafe fn fn_8025F0E0(gobj: *mut HSD_GObj) {
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

pub unsafe fn fn_8025FAC0(gobj: *mut HSD_GObj) {
    let jobj = get_jobj(gobj);

    if mnCharSel_804D6CEC % 40 == 0 {
        let child = if jobj.is_null() { null_mut() } else { (*jobj).child };
        ext::HSD_JObjReqAnimAll(child, 0.0f32);
    }
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe fn fn_8025FB2C(gobj: *mut HSD_GObj) {
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
pub unsafe fn mnCharSel_CursorThink(gobj: *mut HSD_GObj) {
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

