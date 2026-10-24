//! Port of src/melee/gm/gmfrontend.c (TARGET_PC): the port's own frontend, a game mode with one
//! scene whose screens are tables of items (the match-setup rules, the online rooms' rows) and
//! whose chrome is drawn with raw GX quads and Melee's text system. This file is the .c file
//! itself: the toolkit tables and their accessors, the online-pick plumbing and icon capture, the
//! mode router, the layout / drawing helpers, the row text, and the scene's enter / frame / exit.
//!
//! The C file `#include`s fifteen `gmfrontend_*.inc` files (the layout player, the kit, the panel
//! art, the mouse, the menu tree, the Atlas adapters, the online lobby, the select screens and the
//! settings pages). They are separate units here: their functions are `ext` stubs, and the objects
//! gmfrontend.c touches in them (`fm`, `fk`, `fl`, `fp`, `fms`, `fte`, `fss`, `ff`, `kit`, ...) are
//! VIEWS holding only the members this file uses, in their real order. The state oracle compiles
//! the C with the same views (gen/tu/patches/gmfrontend.py), so the two agree field for field.
//! When those units are ported the views are replaced by their real records.
//!
//! Fused multiply-adds (the shipping build contracts `a + b * c`): `fe_arrow` (4), `fe_row_y` (in
//! gm.rs), `fe_ease` (in gm.rs), `fe_draw_panels` (the highlight row, two on the slider track),
//! `gm_Scene_Frontend_OnFrame` (the highlight glide) and `fe_load_step` (the progress bar).
//!
//! `bool` is a 32-bit int in this build, so `bool` members and results are `i32` in the views and
//! tables; the one `static` cache of `fe_enabled` / `fe_menus_on` is a registered global.
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, static_mut_refs, clippy::missing_safety_doc, clippy::too_many_arguments)]

use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null, null_mut};

use melee_core::fp::fmadds;
use melee_types::*;
#[allow(unused_imports)]
use std::vec::Vec;

use crate::gm::{fe_ease, fe_rgba, fe_row_y};
use crate::oracle::Va;
use crate::GXColor;

use super::hsd::{jobj_child, jobj_next, sfxBack, sfxForward, sfxMove};

// ---- constants of gmfrontend.h / gmfrontend.c ----------------------------------------------------

/// `GM_FRONTEND` / `GS_FRONTEND`: one past the retail terminators.
pub const GM_FRONTEND: u8 = GM_COUNT as u8 + 1;
pub const GS_FRONTEND: u8 = GS_COUNT as u8 + 1;

const FE_STR: usize = 80;
const FE_HELP_STR: usize = 512;
const FE_MAX_ROWS: usize = 7;
const FE_MAX_ITEMS: usize = 24;
const FE_GX_LINK: u8 = 14;
const FE_W: f32 = 640.0;
const FE_H: f32 = 480.0;
const FE_FADE_FRAMES: i32 = 10;
const FE_LOAD_MIN_FRAMES: i32 = 45;
const FE_LOAD_SETTLE_FRAMES: i32 = 20;
const FE_LOAD_CEILING_FRAMES: i32 = 8 * 60;
const FE_NP_MAX_CK: usize = 0x80;
const FE_FRAME_M: f32 = 8.0;
const FE_PANEL_X: f32 = 28.0;
const FE_PANEL_Y: f32 = 68.0;
const FE_PANEL_W: f32 = 584.0;
const FE_PANEL_H: f32 = 320.0;
const FE_ROW_X: f32 = 64.0;
const FE_ROW_Y: f32 = 114.0;
const FE_ROW_W: f32 = 512.0;
const FE_ROW_H: f32 = 32.0;
const FE_ROW_STEP: f32 = 34.0;
const FE_VALUE_X: f32 = FE_ROW_X + FE_ROW_W - 40.0;
const FE_BTN_W: f32 = 204.0;
const FE_BTN_H: f32 = 51.0;
const FE_BTN_X: f32 = FE_PANEL_X + FE_PANEL_W - FE_BTN_W - 18.0;
const FE_BTN_Y: f32 = FE_PANEL_Y + FE_PANEL_H - 26.0;
const FE_HINT_Y: f32 = 424.0;
const FE_HINT_A_X: f32 = 44.0;
const FE_HINT_LR_X: f32 = 196.0;
const FE_HINT_B_X: f32 = 372.0;

/// `FM_HUB` / `FM_LIST`, `FM_FADE`, `FSP_*` (the inc files' own enums).
const FM_LIST: u8 = 1;
const FM_FADE: i32 = 8;
const FSP_CONTROLS: usize = 2;
const FSP_MODS: i32 = 4;
const FSP_GAMEPLAY: usize = 5;
const FSP_COUNT: i32 = 6;
const SEL_MAIN_MODS: u8 = 0x45;

// MenuInput (mn/inlines.h)
const MENU_UP: u32 = 1 << 0;
const MENU_DOWN: u32 = 1 << 1;
const MENU_LEFT: u32 = 1 << 2;
const MENU_RIGHT: u32 = 1 << 3;
const MENU_CONFIRM: u32 = 1 << 4;
const MENU_BACK: u32 = 1 << 5;

// FE_NP_* (the online phases)
const FE_NP_IDLE: i32 = 0;
const FE_NP_WORKING: i32 = 1;

// FT_* (the legacy art pack's texture slots)
const FT_FRAME_TL: usize = 0;
const FT_PANEL: usize = 6;
const FT_ROW: usize = 7;
const FT_ROW_SEL: usize = 8;
const FT_BTN: usize = 9;
const FT_BTN_HOVER: usize = 10;
const FT_BTN_PRESS: usize = 11;
const FT_GLYPH_A: usize = 12;
const FT_GLYPH_B: usize = 13;
const FT_CURSOR: usize = 14;
const FT_COUNT: usize = 20;

// ---- records -------------------------------------------------------------------------------------

/// A 32-byte aligned byte array (`ATTRIBUTE_ALIGN(32)` members).
#[repr(C, align(32))]
#[derive(Clone, Copy)]
pub struct A32<const N: usize>(pub [u8; N]);

/// `FeNpIcon`: the stage icon kept off the disc.
#[repr(C)]
pub struct FeNpIcon {
    pub ok: i32,
    pub w: u16,
    pub h: u16,
    pub fmt: i32,
    pub tlut_fmt: i32,
    pub tlut_n: i32,
    pub data: A32<{ 64 * 1024 }>,
    pub lut: A32<512>,
    pub obj: GXTexObj,
    pub tlut: GXTlutObj,
}

/// `FeNpCharIcon`: one fighter's CSS icon.
#[repr(C)]
pub struct FeNpCharIcon {
    pub ok: i32,
    pub w: u16,
    pub h: u16,
    pub fmt: i32,
    pub tlut_fmt: i32,
    pub tlut_n: i32,
    pub data: A32<4096>,
    pub lut: A32<512>,
}

/// `FeTex`: one art-pack texture.
#[repr(C)]
pub struct FeTex {
    pub data: *mut c_void,
    pub lut: *mut c_void,
    pub w: u16,
    pub h: u16,
    pub ok: i32,
    pub obj: GXTexObj,
    pub tlut: GXTlutObj,
}

/// The scene state `fe`.
#[repr(C)]
pub struct FeState {
    pub screen: *const FrontendScreen,
    pub continue_to: u8,
    pub back_to: u8,
    pub canvas: i32,
    pub title: *mut HSD_Text,
    pub subtitle: *mut HSD_Text,
    pub help: *mut HSD_Text,
    pub hints: *mut HSD_Text,
    pub hint_change: *mut HSD_Text,
    pub hint_back: *mut HSD_Text,
    pub label: [*mut HSD_Text; FE_MAX_ROWS],
    pub value: [*mut HSD_Text; FE_MAX_ROWS],
    pub label_str: [[i8; FE_HELP_STR]; FE_MAX_ROWS],
    pub value_str: [[i8; FE_STR]; FE_MAX_ROWS],
    pub help_str: [i8; FE_HELP_STR],
    pub vis: [i32; FE_MAX_ITEMS],
    pub n_vis: i32,
    pub n_list: i32,
    pub has_button: i32,
    pub cursor: i32,
    pub scroll: i32,
    pub frames: i32,
    pub fade: i32,
    pub leaving: i32,
    pub hl_y: f32,
    pub help_item: i32,
    pub loading: i32,
    pub warmed: i32,
    pub load_base: i32,
    pub load_core: i32,
    pub load_settled: i32,
    pub load_frames: i32,
    pub progress: f32,
    pub percent: i32,
    pub reported: u8,
    pub next_menus: i32,
}

// ---- views of the state the inc files own (gen/tu/patches/gmfrontend.py declares the same) --------

/// `FePanelRect` (gmfrontend_panel.inc).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FePanelRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// `FeMenu` (gmfrontend_menus.inc).
#[repr(C)]
pub struct FeMenu {
    pub kind: u8,
    pub style: u8,
    pub hero: u8,
    pub back_kind: u8,
    pub back_sel: u8,
    pub port_all: u8,
    pub title: *const i8,
    pub items: *const c_void,
    pub n: i32,
    pub atlas_id: *const i8,
}

/// `FpTile` (gmfrontend_player.inc): the quads a tile draws.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FpTile {
    pub q_plate: i32,
    pub q_face: i32,
}

/// `fm`: the menu tree's scene state.
#[repr(C)]
pub struct FmView {
    pub active: i32,
    pub start_kind: u8,
    pub start_sel: u8,
    pub native_req: i32,
    pub native_kind: u8,
    pub native_sel: u8,
    pub menu: *const FeMenu,
    pub nvis: i32,
    pub fade: i32,
}
/// `fk`: the kit's list screen.
#[repr(C)]
pub struct FkView {
    pub on: i32,
    pub nrows: i32,
    pub tile: [i32; 12],
    pub scroll: i32,
}
/// `fl`: the room screens.
#[repr(C)]
pub struct FlView {
    pub on: i32,
}
/// `fp`: the layout player.
#[repr(C)]
pub struct FpView {
    pub ok: i32,
    pub frame: i32,
    pub tile: [FpTile; 16],
}
/// `fms`: the mouse.
#[repr(C)]
pub struct FmsView {
    pub moved: i32,
    pub click: i32,
    pub wheel: i32,
}
/// `fte`: text entry.
#[repr(C)]
pub struct FteView {
    pub on: i32,
}
/// `fss`: the Atlas settings pages.
#[repr(C)]
pub struct FssView {
    pub want_label: *const i8,
}
/// `ff` / `kit`: the kit's font and chrome loaders.
#[repr(C)]
pub struct StateView {
    pub state: i32,
}

pub mod ext {
    use super::*;
    crate::oext_fns! {
        fn Controls_Menu(editor: i32) -> ();
        fn Frontend_NativeSelect() -> i32;
        fn Frontend_TrainingSelect() -> i32;
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32) -> ();
        fn GXBegin(ty: GXPrimitive, vtxfmt: GXVtxFmt, nverts: u16) -> ();
        fn GXClearVtxDesc() -> ();
        fn GXColor4u8(x: u8, y: u8, z: u8, w: u8) -> ();
        fn GXGetTexBufferSize(width: u16, height: u16, format: u32, mipmap: u8, max_lod: u8) -> u32;
        fn GXInitTexObj(obj: *mut GXTexObj, image_ptr: *mut c_void, width: u16, height: u16, format: GXTexFmt, wrap_s: GXTexWrapMode, wrap_t: GXTexWrapMode, mipmap: u8) -> ();
        fn GXInitTexObjCI(obj: *mut GXTexObj, image_ptr: *mut c_void, width: u16, height: u16, format: GXTexFmt, wrap_s: GXTexWrapMode, wrap_t: GXTexWrapMode, mipmap: u8, tlut_name: u32) -> ();
        fn GXInitTexObjLOD(obj: *mut GXTexObj, min_filt: GXTexFilter, mag_filt: GXTexFilter, min_lod: f32, max_lod: f32, lod_bias: f32, bias_clamp: GXBool, do_edge_lod: GXBool, max_aniso: GXAnisotropy) -> ();
        fn GXInitTlutObj(tlut_obj: *mut GXTlutObj, lut: *mut c_void, fmt: GXTlutFmt, n_entries: u16) -> ();
        fn GXLoadTexObj(obj: *mut GXTexObj, id: GXTexMapID) -> ();
        fn GXLoadTlut(tlut_obj: *mut GXTlutObj, tlut_name: u32) -> ();
        fn GXPosition2f32(x: f32, y: f32) -> ();
        fn GXSetAlphaCompare(comp0: GXCompare, ref0: u8, op: GXAlphaOp, comp1: GXCompare, ref1: u8) -> ();
        fn GXSetBlendMode(ty: GXBlendMode, src_factor: GXBlendFactor, dst_factor: GXBlendFactor, op: GXLogicOp) -> ();
        fn GXSetChanCtrl(chan: GXChannelID, enable: GXBool, amb_src: GXColorSrc, mat_src: GXColorSrc, light_mask: u32, diff_fn: GXDiffuseFn, attn_fn: GXAttnFn) -> ();
        fn GXSetCullMode(mode: GXCullMode) -> ();
        fn GXSetNumChans(nChans: u8) -> ();
        fn GXSetNumIndStages(nIndStages: u8) -> ();
        fn GXSetNumTevStages(nStages: u8) -> ();
        fn GXSetNumTexGens(nTexGens: u8) -> ();
        fn GXSetTevOp(id: GXTevStageID, mode: GXTevMode) -> ();
        fn GXSetTevOrder(stage: GXTevStageID, coord: GXTexCoordID, map: GXTexMapID, color: GXChannelID) -> ();
        fn GXSetTexCoordGen2(dst_coord: GXTexCoordID, func: GXTexGenType, src_param: GXTexGenSrc, mtx: u32, normalize: GXBool, pt_texmtx: u32) -> ();
        fn GXSetVtxAttrFmt(vtxfmt: GXVtxFmt, attr: GXAttr, cnt: GXCompCnt, ty: GXCompType, frac: u8) -> ();
        fn GXSetVtxDesc(attr: GXAttr, ty: GXAttrType) -> ();
        fn GXSetZMode(compare_enable: GXBool, func: GXCompare, update_enable: GXBool) -> ();
        fn GXTexCoord2f32(x: f32, y: f32) -> ();
        fn Gfx_LoadScreenEnabled() -> i32;
        fn Gfx_PipelinesCreated() -> i32;
        fn Gfx_PipelinesPending() -> i32;
        fn Gfx_PipelinesUrgent() -> i32;
        fn Gfx_SeedCoreCount() -> i32;
        fn Gfx_SeedCorePending() -> i32;
        fn Gfx_SeedPipelinesBuilt() -> i32;
        fn GxTex_Close(h: i32) -> ();
        fn GxTex_CopyImage(h: i32, dst: *mut c_void) -> ();
        fn GxTex_CopyTlut(h: i32, dst: *mut c_void) -> ();
        fn GxTex_Format(h: i32) -> i32;
        fn GxTex_Height(h: i32) -> i32;
        fn GxTex_ImageSize(h: i32) -> i32;
        fn GxTex_OpenUI(name: *const i8) -> i32;
        fn GxTex_TlutEntries(h: i32) -> i32;
        fn GxTex_TlutFormat(h: i32) -> i32;
        fn GxTex_TlutSize(h: i32) -> i32;
        fn GxTex_Width(h: i32) -> i32;
        fn HSD_JObjGetDObj(jobj: *mut HSD_JObj) -> *mut HSD_DObj;
        fn HSD_MemAlloc(size: i32) -> *mut c_void = crate::tu::adapt::HSD_MemAlloc__c2b3bc;
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) -> () = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
        fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 = crate::tu::adapt::HSD_SisLib_803A611C__de78e8;
        fn HSD_SisLib_803A6754(arg0: i32, arg1: i32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A6754__02fb41;
        fn HSD_SisLib_803A74F0(a0: *mut HSD_Text, arg1: i32, arg2: *mut GXColor) -> ();
        fn HSD_SisLib_SetWideCanvas(font_idx: i32, canvas: i32) -> () = crate::tu::adapt::HSD_SisLib_SetWideCanvas__9bb4fb;
        fn HSD_StateInvalidate(mask: i32) -> ();
        fn Netplay_EnvoyAvailable() -> i32;
        fn Netplay_EnvoyPref() -> i32;
        fn Netplay_FighterAvailable(ck: i32) -> i32;
        fn Netplay_FighterWhy(ck: i32, why: *mut i8, cap: i32) -> i32;
        fn Netplay_LobbyActive() -> i32;
        fn Netplay_LobbyChar(ck: i32, color: i32) -> ();
        fn Netplay_MenuBegin(host: i32, ck: i32, color: i32, stocks: i32, minutes: i32, delay: i32) -> i32;
        fn Netplay_MenuHasServer() -> i32;
        fn Netplay_MenuStatus(out: *mut i8, cap: i32) -> ();
        fn Netplay_RandomBegin(ck: i32, color: i32, stocks: i32, minutes: i32, delay: i32) -> i32;
        fn Netplay_SetEnvoy(on: i32) -> ();
        fn Netplay_SetStageMode(mode: i32) -> ();
        fn Netplay_SetTurbo(on: i32) -> ();
        fn Netplay_StageMode() -> i32;
        fn Netplay_TurboPref() -> i32;
        fn PcFrontendEnabled() -> i32;
        fn PcFrontendMenusEnabled() -> i32;
        fn SceneLaunch_SetText(text: *const i8) -> ();
        fn Settings_Int(key: *const i8, dflt: i32) -> i32;
        fn Settings_SetInt(key: *const i8, value: i32) -> ();
        fn Ui_CssKnown(match_type: i32) -> i32;
        fn Ui_Ready() -> i32;
        fn Ui_SelCanOpen() -> i32;
        fn Ui_SelNextScene(on: i32) -> ();
        fn View_AspectBits() -> i32;
        fn fa_room_end() -> ();
        fn fa_room_leaving() -> ();
        fn fa_room_on() -> i32;
        fn fas_active() -> i32;
        fn fcr_back() -> ();
        fn fcr_frame() -> i32;
        fn fe_is_controls_help(s: *const FrontendScreen) -> i32;
        fn fe_is_settings(s: *const FrontendScreen) -> i32;
        fn fe_panel_art_frame(bounds: *const FePanelRect, corner: f32, edge: f32, cuts: u32, filled: i32) -> ();
        fn fe_panel_art_image(tex: i32, bounds: *const FePanelRect, authored_w: f32, authored_h: f32, cx: f32, cy: f32, fallback: GXColor) -> ();
        fn fe_panel_rect(r: *mut FePanelRect, x: f32, y: f32, w: f32, h: f32) -> ();
        fn fe_panel_solid(x: f32, y: f32, w: f32, h: f32, colour: GXColor) -> ();
        fn fe_rules_resume() -> ();
        fn fe_settings_apply_online() -> ();
        fn ff_load() -> ();
        fn fk_build(animate_in: i32) -> ();
        fn fk_bump(dir: i32) -> ();
        fn fk_exit() -> ();
        fn fk_frame() -> ();
        fn fk_note_dir(dir: i32) -> ();
        fn fl_code_reset() -> ();
        fn fl_exit() -> ();
        fn fl_frame() -> ();
        fn fl_frame_anim() -> ();
        fn fl_lobby_notice(s: *const i8) -> ();
        fn fl_open(art: i32) -> ();
        fn fl_rejoin_reset() -> ();
        fn fm_leave_scene(mode: u8) -> ();
        fn fm_position_for(previous_mode: u8, kind: *mut u8, sel: *mut u8) -> ();
        fn fm_replaced(kind: i32) -> i32;
        fn fm_scene_enter() -> ();
        fn fm_scene_exit() -> ();
        fn fm_scene_frame() -> ();
        fn fms_in_quad(q: i32) -> i32;
        fn fms_menu_bits() -> u32;
        fn fms_poll() -> ();
        fn fp_draw() -> ();
        fn fp_evaluate(f: i32) -> ();
        fn fp_free_textures() -> ();
        fn fp_scroll_cue(scroll: i32, count: i32) -> ();
        fn fsm_fill() -> ();
        fn fss_close() -> ();
        fn fss_exit() -> ();
        fn fss_frame() -> i32;
        fn fss_intercepts(s: *const FrontendScreen) -> i32;
        fn fss_switch(s: *const FrontendScreen) -> ();
        fn fte_frame() -> i32;
        fn gmMainLib_GetGamePrefs() -> *mut GamePrefs = crate::tu::adapt::gmMainLib_GetGamePrefs__12f16c;
        fn gmMainLib_GetGameRules() -> *mut GameRules = crate::tu::adapt::gmMainLib_GetGameRules__34f6cb;
        fn gm_801A4B60() -> () = crate::tu::adapt::gm_801A4B60__d469d9;
        fn gm_ChangeGameModeAfterCurrentScene(pending_mode: i32) -> () = crate::tu::adapt::gm_ChangeGameModeAfterCurrentScene__2292c1;
        fn gm_GetGameModeStateEnterData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateEnterData__d31bdf;
        fn hsd_80391A04(scale_x: f32, scale_y: f32, line_width: i32) -> ();
        fn kit_load() -> ();
        fn lbCardGame_LoadArchive(a0: i32) -> () = crate::tu::adapt::lbCardGame_LoadArchive__e9acb0;
        fn lbCardNew_AllocWorkArea() -> () = crate::tu::adapt::lbCardNew_AllocWorkArea__a2c203;
        fn lbDvd_8001823C() -> () = crate::tu::adapt::lbDvd_8001823C__d87971;
        fn lbDvd_80018254() -> () = crate::tu::adapt::lbDvd_80018254__6be579;
        fn lbDvd_80018C6C() -> () = crate::tu::adapt::lbDvd_80018C6C__4e040c;
        fn mn_80229624(a0: u32) -> u32 = crate::tu::adapt::mn_80229624__84b7ce;
    }
    crate::ext_va! {
        /// os: `(fmt, ...)`
        fn OSReport(fmt: *const i8) va 0 -> ();
        /// sislib.c: `(text, x, y, fmt, ...)`
        fn HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const i8) va 3 -> i32;
    }
}

// ---- tables and state (statics of the C file; the Rp ones hold pointers) ---------------------------

const ITEM0: FrontendItem = unsafe { core::mem::zeroed() };
const SCREEN0: FrontendScreen = unsafe { core::mem::zeroed() };

macro_rules! cs {
    ($s:literal) => {
        $s.as_ptr()
    };
}

macro_rules! sp {
    ($v:ident) => {
        addr_of!($v) as *const *const i8
    };
}

crate::globals! {
    rp pub fe_opt_mode: [*const i8; 4] = [cs!(c"Time"), cs!(c"Stock"), cs!(c"Coin"), cs!(c"Bonus")];
    rp pub fe_opt_handicap: [*const i8; 3] = [cs!(c"Off"), cs!(c"Auto"), cs!(c"On")];
    rp pub fe_opt_stage_sel: [*const i8; 5] = [cs!(c"On"), cs!(c"Random"), cs!(c"Ordered"), cs!(c"Turns"), cs!(c"Loser")];
    rp pub fe_opt_items: [*const i8; 6] = [cs!(c"None"), cs!(c"Very Low"), cs!(c"Low"), cs!(c"Medium"), cs!(c"High"), cs!(c"Very High")];
    rp pub fe_items_vs_setup: [FrontendItem; 10] = [
        FrontendItem { kind: FE_ACTION as u8, action: FE_DO_CONTINUE as u8, label: cs!(c"Continue to Character Select"), help: cs!(c"Pick fighters with these rules."), ..ITEM0 },
        FrontendItem { kind: FE_CHOICE as u8, action: 0, label: cs!(c"Mode"), help: cs!(c"How a match is won."), get: Some(fe_get_mode), set: Some(fe_set_mode), min: 0, max: 3, step: 1, options: sp!(fe_opt_mode), ..ITEM0 },
        FrontendItem { kind: FE_SLIDER as u8, action: 0, label: cs!(c"Stocks"), help: cs!(c"Lives each player starts with."), get: Some(fe_get_stocks), set: Some(fe_set_stocks), min: 1, max: 99, step: 1, options: null(), format: None, visible: Some(fe_is_stock), ..ITEM0 },
        FrontendItem { kind: FE_SLIDER as u8, action: 0, label: cs!(c"Time Limit"), help: cs!(c"0 plays without a clock."), get: Some(fe_get_time), set: Some(fe_set_time), min: 0, max: 99, step: 1, options: null(), format: Some(fe_fmt_time), ..ITEM0 },
        FrontendItem { kind: FE_CHOICE as u8, action: 0, label: cs!(c"Items"), help: cs!(c"How often items appear."), get: Some(fe_get_items), set: Some(fe_set_items), min: -1, max: 4, step: 1, options: sp!(fe_opt_items), ..ITEM0 },
        FrontendItem { kind: FE_SLIDER as u8, action: 0, label: cs!(c"Damage Ratio"), help: cs!(c"How far hits send fighters."), get: Some(fe_get_ratio), set: Some(fe_set_ratio), min: 5, max: 20, step: 1, options: null(), format: Some(fe_fmt_ratio), ..ITEM0 },
        FrontendItem { kind: FE_CHOICE as u8, action: 0, label: cs!(c"Handicap"), help: cs!(c"Auto evens out winners and losers."), get: Some(fe_get_handicap), set: Some(fe_set_handicap), min: 0, max: 2, step: 1, options: sp!(fe_opt_handicap), ..ITEM0 },
        FrontendItem { kind: FE_CHOICE as u8, action: 0, label: cs!(c"Stage Select"), help: cs!(c"How the stage is chosen."), get: Some(fe_get_stage_sel), set: Some(fe_set_stage_sel), min: 0, max: 4, step: 1, options: sp!(fe_opt_stage_sel), ..ITEM0 },
        FrontendItem { kind: FE_TOGGLE as u8, action: 0, label: cs!(c"Friendly Fire"), help: cs!(c"Team attacks hit teammates."), get: Some(fe_get_ff), set: Some(fe_set_ff), min: 0, max: 1, step: 1, ..ITEM0 },
        FrontendItem { kind: FE_TOGGLE as u8, action: 0, label: cs!(c"Pause"), help: cs!(c"Allow pausing during the match."), get: Some(fe_get_pause), set: Some(fe_set_pause), min: 0, max: 1, step: 1, ..ITEM0 },
    ];
    rp pub fe_screen_vs_setup: FrontendScreen = FrontendScreen { title: cs!(c"VS. MELEE"), subtitle: cs!(c"MATCH SETUP"), items: addr_of!(fe_items_vs_setup) as *const FrontendItem, n_items: 10, art: 0 };

    /// `fe_np_ck` ... the fighter this player last picked and the rules of rooms it hosts
    rw pub fe_np_ck: i32 = 2;
    rw pub fe_np_color: i32 = 0;
    rw pub fe_np_stocks: i32 = 4;
    rw pub fe_np_minutes: i32 = 8;
    rw pub fe_np_delay: i32 = 2;
    rw pub fe_np_phase: i32 = 0;
    rw pub fe_np_pick: i32 = 0;
    rw pub fe_np_icon: [FeNpIcon; 2] = unsafe { core::mem::zeroed() };
    rw pub fe_np_char_icon: [FeNpCharIcon; FE_NP_MAX_CK] = unsafe { core::mem::zeroed() };
    rw pub fe_np_scene: [i8; 160] = [0; 160];
    rw pub fe_ol_note: [i8; FE_HELP_STR] = [0; FE_HELP_STR];
    rw pub fe_ol_note_bad: i32 = 0;
    rw pub fe_ol_note_frames: i32 = 0;
    rp pub fe_np_stage_modes: [*const i8; 2] = [cs!(c"Competitive"), cs!(c"All Stages")];
    rp pub fe_np_turbo_modes: [*const i8; 2] = [cs!(c"Off"), cs!(c"On")];
    rp pub fe_np_envoy_modes: [*const i8; 2] = [cs!(c"Off"), cs!(c"On")];
    rp pub fe_items_online: [FrontendItem; 9] = [
        FrontendItem { kind: FE_ACTION as u8, action: FE_DO_CALL as u8, label: cs!(c"Host a Room"), help: cs!(c"Open a room and get a code to send your opponent."), call: Some(fe_ol_host), ..ITEM0 },
        FrontendItem { kind: FE_ACTION as u8, action: FE_DO_CALL as u8, label: cs!(c"Join a Room"), help: cs!(c"Type in the room code your opponent sent you."), call: Some(fe_ol_join), ..ITEM0 },
        FrontendItem { kind: FE_ACTION as u8, action: FE_DO_CALL as u8, label: cs!(c"Random Opponent"), help: cs!(c"Play whoever else is looking right now."), call: Some(fe_ol_random), ..ITEM0 },
        FrontendItem { kind: FE_CHOICE as u8, action: 0, label: cs!(c"Stage List"), help: cs!(c"Stages in rooms you host: the legal six, or every stage you both have."), get: Some(fe_np_get_stage_mode), set: Some(fe_np_set_stage_mode), min: 0, max: 1, step: 1, options: sp!(fe_np_stage_modes), ..ITEM0 },
        FrontendItem { kind: FE_CHOICE as u8, action: 0, label: cs!(c"Turbo"), help: cs!(c"Rooms you host: a connected hit can be cancelled into most moves."), get: Some(fe_np_get_turbo), set: Some(fe_np_set_turbo), min: 0, max: 1, step: 1, options: sp!(fe_np_turbo_modes), ..ITEM0 },
        FrontendItem { kind: FE_CHOICE as u8, action: 0, label: cs!(c"Envoy"), help: cs!(c"Rooms you host: a best-of set, an Envoy build each. Needs the Envoy mod."), get: Some(fe_np_get_envoy), set: Some(fe_np_set_envoy), min: 0, max: 1, step: 1, options: sp!(fe_np_envoy_modes), ..ITEM0 },
        FrontendItem { kind: FE_SLIDER as u8, action: 0, label: cs!(c"Stocks"), help: cs!(c"Stocks per game, in rooms you host."), get: Some(fe_np_get_stocks), set: Some(fe_np_set_stocks), min: 1, max: 9, step: 1, ..ITEM0 },
        FrontendItem { kind: FE_SLIDER as u8, action: 0, label: cs!(c"Time Limit"), help: cs!(c"Minutes per game, in rooms you host."), get: Some(fe_np_get_minutes), set: Some(fe_np_set_minutes), min: 1, max: 20, step: 1, options: null(), format: Some(fe_np_fmt_minutes), ..ITEM0 },
        FrontendItem { kind: FE_SLIDER as u8, action: 0, label: cs!(c"Input Delay"), help: cs!(c"Frames of delay in rooms you host. 2 suits most connections."), get: Some(fe_np_get_delay), set: Some(fe_np_set_delay), min: 0, max: 8, step: 1, options: null(), format: Some(fe_np_fmt_delay), ..ITEM0 },
    ];
    rp pub fe_screen_online: FrontendScreen = FrontendScreen { title: cs!(c"ONLINE"), subtitle: cs!(c"ONLINE PLAY"), items: addr_of!(fe_items_online) as *const FrontendItem, n_items: 9, art: 0 };
    rp pub fe_screen_code: FrontendScreen = FrontendScreen { title: cs!(c"ONLINE"), subtitle: cs!(c"JOIN ROOM"), items: null(), n_items: 0, art: 1 };
    rp pub fe_screen_wait: FrontendScreen = FrontendScreen { title: cs!(c"ONLINE"), subtitle: cs!(c"WAITING ROOM"), items: null(), n_items: 0, art: 2 };
    rp pub fe_screen_lobby: FrontendScreen = FrontendScreen { title: cs!(c"ONLINE"), subtitle: cs!(c"LOBBY"), items: null(), n_items: 0, art: 3 };
    rp pub fe_screen_loading: FrontendScreen = FrontendScreen { title: cs!(c"GET READY"), subtitle: cs!(c"LOADING"), items: null(), n_items: 0, art: 4 };
    rp pub fe_screen_css: FrontendScreen = FrontendScreen { title: cs!(c"VERSUS"), subtitle: cs!(c"CHARACTERS"), items: null(), n_items: 0, art: 5 };
    rp pub fe_screen_sss: FrontendScreen = FrontendScreen { title: cs!(c"VERSUS"), subtitle: cs!(c"STAGES"), items: null(), n_items: 0, art: 6 };
    rw pub fe_sel_css: *mut CSSData = null_mut();
    rw pub fe_sel_sss: *mut SSSData = null_mut();
    rw pub fe_sel_training: i32 = 0;
    rw pub fe_train_css: *mut c_void = null_mut();
    rw pub fe_train_sss: *mut c_void = null_mut();
    rw pub fe_ats_css: *mut c_void = null_mut();
    rw pub fe_ats_sss: *mut c_void = null_mut();
    rw pub fe_sel_mode_pending: *const i8 = null();
    rw pub fe_sel_mode: *const i8 = null();
    rp pub fe_screen_online_load: FrontendScreen = FrontendScreen { title: cs!(c"GET READY"), subtitle: cs!(c"LOADING"), items: null(), n_items: 0, art: 4 };
    /// `static const FrontendRule fe_rules[]`: `{ from, to, screen }` (3 bytes + a pointer)
    rp pub fe_rules: [FrontendRule; 1] = [FrontendRule { from: GM_MENU as u8, to: GM_VS as u8, screen: addr_of!(fe_screen_vs_setup) }];
    rw pub fe: FeState = unsafe { core::mem::zeroed() };
    rp pub fe_tex_names: [*const i8; FT_COUNT] = [
        cs!(c"frame_corner_tl"), cs!(c"frame_corner_tr"), cs!(c"frame_corner_bl"), cs!(c"frame_corner_br"),
        cs!(c"frame_edge_h"), cs!(c"frame_edge_v"), cs!(c"panel_bg"), cs!(c"row_ng"), cs!(c"row_sel"),
        cs!(c"btn_continue_ng"), cs!(c"btn_continue_hover"), cs!(c"btn_continue_press"),
        cs!(c"glyph_a"), cs!(c"glyph_b"), cs!(c"cursor_hand"),
        cs!(c"frame_fill"), cs!(c"frame_cut_corner_tl"), cs!(c"frame_cut_corner_tr"),
        cs!(c"frame_cut_corner_bl"), cs!(c"frame_cut_corner_br"),
    ];
    rw pub fe_tex: [FeTex; FT_COUNT] = unsafe { core::mem::zeroed() };
    rw pub fe_kit: i32 = 0;
    rw pub fm_back_kind: i32 = -1;
    rw pub fm_back_sel: i32 = 0;
    rw pub fm_back_to_online_item: i32 = 0;
    rw pub fe_enter_data: [u8; 4] = [0; 4];
    /// the two lazily cached host answers (function-local statics in the C; gen/tu/patches/gmfrontend.py)
    rw pub fe_enabled_cache: i32 = -1;
    rw pub fe_menus_on_cache: i32 = -1;
    rp pub gm_Mode_Frontend_States: [GameModeState; 2] = [
        GameModeState {
            id: 0,
            preload: lbDvdPreload_2 as u8,
            flags: 0,
            on_enter: Some(fe_state_enter),
            on_exit: None,
            info: GameSceneInfo { scene_kind: GS_FRONTEND, enter_data: unsafe { addr_of_mut!(fe_enter_data) as *mut c_void }, exit_data: null_mut() },
        },
        GameModeState { id: GM_GAMEMODESTATE_TERMINATE as u8, preload: 0, flags: 0, on_enter: None, on_exit: None, info: GameSceneInfo { scene_kind: 0, enter_data: null_mut(), exit_data: null_mut() } },
    ];

    // ---- extern state this file only declares (owned by units not ported yet) ----
    rw pub gmVsMelee_CssData: CSSData = unsafe { core::mem::zeroed() };
    rw pub gmVsMelee_SssData: SSSData = unsafe { core::mem::zeroed() };
    rw pub fm: FmView = unsafe { core::mem::zeroed() };
    rw pub fk: FkView = unsafe { core::mem::zeroed() };
    rw pub fl: FlView = unsafe { core::mem::zeroed() };
    rw pub fp: FpView = unsafe { core::mem::zeroed() };
    rw pub fms: FmsView = unsafe { core::mem::zeroed() };
    rw pub fte: FteView = unsafe { core::mem::zeroed() };
    rs pub fss: FssView = unsafe { core::mem::zeroed() };
    rw pub ff: StateView = unsafe { core::mem::zeroed() };
    rw pub kit: StateView = unsafe { core::mem::zeroed() };
    rw pub fa_mods_door: i32 = 0;
    rw pub fa_page_from_main: i32 = 0;
    rw pub fm_scroll: i32 = 0;
    rw pub fcr_port: i32 = 0;
    rp pub fe_screen_remap: FrontendScreen = SCREEN0;
    rp pub fe_screen_settings: [FrontendScreen; 6] = [SCREEN0; 6];
}

/// `FrontendRule`: `{ u8 from; u8 to; const FrontendScreen* screen; }`.
#[repr(C)]
pub struct FrontendRule {
    pub from: u8,
    pub to: u8,
    pub screen: *const FrontendScreen,
}

#[inline]
unsafe fn fe_() -> *mut FeState {
    addr_of_mut!(fe)
}

// ---- C string helpers (sprintf into a char buffer) ------------------------------------------------

/// `sprintf(out, "<bytes>")` for text with no conversions: the bytes and a NUL.
unsafe fn put(out: *mut i8, s: &str) {
    let b = s.as_bytes();
    core::ptr::copy_nonoverlapping(b.as_ptr(), out as *mut u8, b.len());
    *out.add(b.len()) = 0;
}

/// `sprintf(out, "%s", s)`.
unsafe fn put_cstr(out: *mut i8, s: *const i8) {
    let mut i = 0usize;
    while *s.add(i) != 0 {
        *out.add(i) = *s.add(i);
        i += 1;
    }
    *out.add(i) = 0;
}

// ---- the match-setup rules (GameRules / GamePrefs accessors) ---------------------------------------

pub unsafe extern "C" fn fe_get_mode() -> i32 {
    (*ext::gmMainLib_GetGameRules()).mode as i32
}
pub unsafe extern "C" fn fe_set_mode(v: i32) {
    (*ext::gmMainLib_GetGameRules()).mode = v as u8;
}
pub unsafe extern "C" fn fe_is_stock() -> i32 {
    ((*ext::gmMainLib_GetGameRules()).mode == 1) as i32
}
pub unsafe extern "C" fn fe_get_stocks() -> i32 {
    (*ext::gmMainLib_GetGameRules()).stock_count as i32
}
pub unsafe extern "C" fn fe_set_stocks(v: i32) {
    (*ext::gmMainLib_GetGameRules()).stock_count = v as u8;
}
pub unsafe extern "C" fn fe_get_time() -> i32 {
    let r = ext::gmMainLib_GetGameRules();
    if (*r).mode == 1 { (*r).stock_time_limit as i32 } else { (*r).time_limit as i32 }
}
pub unsafe extern "C" fn fe_set_time(v: i32) {
    let r = ext::gmMainLib_GetGameRules();
    if (*r).mode == 1 {
        (*r).stock_time_limit = v as u8;
    } else {
        (*r).time_limit = v as u8;
    }
}
pub unsafe extern "C" fn fe_fmt_time(v: i32, out: *mut i8) {
    if v == 0 {
        put(out, "None");
    } else {
        put(out, &std::format!("{} min", v));
    }
}
pub unsafe extern "C" fn fe_get_handicap() -> i32 {
    (*ext::gmMainLib_GetGameRules()).handicap as i32
}
pub unsafe extern "C" fn fe_set_handicap(v: i32) {
    (*ext::gmMainLib_GetGameRules()).handicap = v as u8;
}
pub unsafe extern "C" fn fe_get_ratio() -> i32 {
    (*ext::gmMainLib_GetGameRules()).damage_ratio as i32
}
pub unsafe extern "C" fn fe_set_ratio(v: i32) {
    (*ext::gmMainLib_GetGameRules()).damage_ratio = v as u8;
}
pub unsafe extern "C" fn fe_fmt_ratio(v: i32, out: *mut i8) {
    put(out, &std::format!("{}.{}x", v / 10, v % 10));
}
pub unsafe extern "C" fn fe_get_stage_sel() -> i32 {
    (*ext::gmMainLib_GetGameRules()).stage_sel as i32
}
pub unsafe extern "C" fn fe_set_stage_sel(v: i32) {
    (*ext::gmMainLib_GetGameRules()).stage_sel = v as u8;
}
pub unsafe extern "C" fn fe_get_items() -> i32 {
    (*ext::gmMainLib_GetGamePrefs()).item_freq as i8 as i32
}
pub unsafe extern "C" fn fe_set_items(v: i32) {
    (*ext::gmMainLib_GetGamePrefs()).item_freq = v as i8 as u8;
}
pub unsafe extern "C" fn fe_get_ff() -> i32 {
    ((*ext::gmMainLib_GetGameRules()).friendly_fire & 1) as i32
}
pub unsafe extern "C" fn fe_set_ff(v: i32) {
    let r = ext::gmMainLib_GetGameRules();
    (*r).friendly_fire = (((*r).friendly_fire as i32 & !1) | (v & 1)) as u8;
}
pub unsafe extern "C" fn fe_get_pause() -> i32 {
    ((*ext::gmMainLib_GetGameRules()).pause != 0) as i32
}
pub unsafe extern "C" fn fe_set_pause(v: i32) {
    (*ext::gmMainLib_GetGameRules()).pause = (v != 0) as u8;
}

// ---- ONLINE: rooms over the internet -----------------------------------------------------------------

pub unsafe extern "C" fn fe_np_get_stocks() -> i32 {
    fe_np_stocks
}
pub unsafe extern "C" fn fe_np_set_stocks(v: i32) {
    fe_np_stocks = v;
}
pub unsafe extern "C" fn fe_np_get_minutes() -> i32 {
    fe_np_minutes
}
pub unsafe extern "C" fn fe_np_set_minutes(v: i32) {
    fe_np_minutes = v;
}
pub unsafe extern "C" fn fe_np_fmt_minutes(v: i32, out: *mut i8) {
    put(out, &std::format!("{} min", v));
}
pub unsafe extern "C" fn fe_np_get_delay() -> i32 {
    fe_np_delay
}
pub unsafe extern "C" fn fe_np_set_delay(v: i32) {
    fe_np_delay = v;
}
pub unsafe extern "C" fn fe_np_fmt_delay(v: i32, out: *mut i8) {
    put(out, &std::format!("{} frames", v));
}

/// `Frontend_OnlinePick`.
pub unsafe fn Frontend_OnlinePick() -> i32 {
    fe_np_pick
}

/// Copy the biggest texture under `j` - the icon the player just confirmed.
pub unsafe fn fe_np_find_tex(mut j: *mut HSD_JObj, best: *mut *mut HSD_TObj, area: *mut i32, depth: i32) {
    // the chosen icon and its own subtree - not its siblings, which are the other icons
    while !j.is_null() && depth < 12 {
        let mut d = ext::HSD_JObjGetDObj(j);
        while !d.is_null() {
            let mut t = if !(*d).mobj.is_null() { (*(*d).mobj).tobj } else { null_mut() };
            while !t.is_null() {
                if !(*t).imagedesc.is_null() && !(*(*t).imagedesc).image_ptr.is_null() {
                    // colour beats intensity: an icon's plate/outline is an I4/IA4 mask, the
                    // portrait a colour (usually CI8) texture of the same size
                    let fmt = (*(*t).imagedesc).format as i32;
                    let a = (*(*t).imagedesc).width as i32 * (*(*t).imagedesc).height as i32 * if fmt >= 4 { 4 } else { 1 };
                    if a > *area {
                        *area = a;
                        *best = t;
                    }
                }
                t = (*t).next;
            }
            d = (*d).next;
        }
        fe_np_find_tex(jobj_child(j), best, area, depth + 1);
        j = if depth == 0 { null_mut() } else { jobj_next(j) };
    }
}

/// Keep a copy of an image off the disc (the select screens' art). `which`: 0..FE_NP_MAX_CK-1 a
/// fighter's icon, -1 the chosen stage's icon.
pub unsafe fn fe_np_store(which: i32, im: *mut HSD_ImageDesc, tl: *mut HSD_Tlut) {
    let data: *mut u8;
    let lut: *mut u8;
    let cap: i32;
    let lcap: i32;
    let ok: *mut i32;
    let w: *mut u16;
    let h: *mut u16;
    let fmt: *mut i32;
    let tfmt: *mut i32;
    let tn: *mut i32;
    if im.is_null() || (*im).image_ptr.is_null() {
        return;
    }
    if which < 0 {
        let ic = addr_of_mut!(fe_np_icon[1]);
        data = (*ic).data.0.as_mut_ptr();
        cap = 64 * 1024;
        lut = (*ic).lut.0.as_mut_ptr();
        lcap = 512;
        ok = addr_of_mut!((*ic).ok);
        w = addr_of_mut!((*ic).w);
        h = addr_of_mut!((*ic).h);
        fmt = addr_of_mut!((*ic).fmt);
        tfmt = addr_of_mut!((*ic).tlut_fmt);
        tn = addr_of_mut!((*ic).tlut_n);
    } else if (which as usize) < FE_NP_MAX_CK {
        let ic = addr_of_mut!(fe_np_char_icon[which as usize]);
        data = (*ic).data.0.as_mut_ptr();
        cap = 4096;
        lut = (*ic).lut.0.as_mut_ptr();
        lcap = 512;
        ok = addr_of_mut!((*ic).ok);
        w = addr_of_mut!((*ic).w);
        h = addr_of_mut!((*ic).h);
        fmt = addr_of_mut!((*ic).fmt);
        tfmt = addr_of_mut!((*ic).tlut_fmt);
        tn = addr_of_mut!((*ic).tlut_n);
    } else {
        return;
    }
    let size = ext::GXGetTexBufferSize((*im).width, (*im).height, (*im).format as u32, 0, 0);
    if size == 0 || size as i32 > cap {
        return;
    }
    core::ptr::copy_nonoverlapping((*im).image_ptr as *const u8, data, size as usize);
    *w = (*im).width;
    *h = (*im).height;
    *fmt = (*im).format as i32;
    *tn = 0;
    if !tl.is_null() && !(*tl).lut.is_null() && (*tl).n_entries as i32 * 2 <= lcap {
        *tn = (*tl).n_entries as i32;
        *tfmt = (*tl).fmt as i32;
        core::ptr::copy_nonoverlapping((*tl).lut as *const u8, lut, *tn as usize * 2);
    }
    *ok = 1;
}

/// Copy the biggest texture under `root` into the icon slot `ic` (shared by the two capture calls).
unsafe fn capture_into(
    t: *mut HSD_TObj,
    data: *mut u8,
    cap: usize,
    lut: *mut u8,
    lutcap: usize,
    ok: *mut i32,
    w: *mut u16,
    h: *mut u16,
    fmt: *mut i32,
    tlut_n: *mut i32,
    tlut_fmt: *mut i32,
) {
    let id = (*t).imagedesc;
    let size = ext::GXGetTexBufferSize((*id).width, (*id).height, (*id).format as u32, 0, 0);
    if size == 0 || size as usize > cap {
        return;
    }
    core::ptr::copy_nonoverlapping((*id).image_ptr as *const u8, data, size as usize);
    *w = (*id).width;
    *h = (*id).height;
    *fmt = (*id).format as i32;
    *tlut_n = 0;
    {
        // an animated palette is tluttbl[tlut_no] (tobj.c)
        let tl = if (*t).tlut_no != TOBJ_TLUT_NONE && !(*t).tluttbl.is_null() {
            *(*t).tluttbl.offset((*t).tlut_no as isize)
        } else {
            (*t).tlut
        };
        if !tl.is_null() && !(*tl).lut.is_null() && (*tl).n_entries as i32 * 2 <= lutcap as i32 {
            *tlut_n = (*tl).n_entries as i32;
            *tlut_fmt = (*tl).fmt as i32;
            core::ptr::copy_nonoverlapping((*tl).lut as *const u8, lut, *tlut_n as usize * 2);
        }
    }
    *ok = 1;
}

/// `Frontend_CaptureCharIcon`: every fighter's CSS icon, captured on each CSS visit.
pub unsafe fn Frontend_CaptureCharIcon(ck: i32, root: *mut HSD_JObj) {
    if ck < 0 || ck >= FE_NP_MAX_CK as i32 || root.is_null() {
        return;
    }
    let ic = addr_of_mut!(fe_np_char_icon[ck as usize]);
    let mut t: *mut HSD_TObj = null_mut();
    let mut area = 0i32;
    fe_np_find_tex(root, &mut t, &mut area, 0);
    if t.is_null() {
        return;
    }
    // (the size test reads the image desc before anything is written)
    capture_into(
        t,
        (*ic).data.0.as_mut_ptr(),
        4096,
        (*ic).lut.0.as_mut_ptr(),
        512,
        addr_of_mut!((*ic).ok),
        addr_of_mut!((*ic).w),
        addr_of_mut!((*ic).h),
        addr_of_mut!((*ic).fmt),
        addr_of_mut!((*ic).tlut_n),
        addr_of_mut!((*ic).tlut_fmt),
    );
}

/// `Frontend_CaptureIcon`: the character (0) or the stage (1) icon the player just confirmed.
pub unsafe fn Frontend_CaptureIcon(which: i32, root: *mut HSD_JObj) {
    if which < 0 || which > 1 || root.is_null() {
        return;
    }
    let ic = addr_of_mut!(fe_np_icon[which as usize]);
    let mut t: *mut HSD_TObj = null_mut();
    let mut area = 0i32;
    fe_np_find_tex(root, &mut t, &mut area, 0);
    if t.is_null() {
        ext::OSReport(c"frontend: online pick - no texture under the chosen icon\n".as_ptr(), &[]);
        return;
    }
    capture_into(
        t,
        (*ic).data.0.as_mut_ptr(),
        64 * 1024,
        (*ic).lut.0.as_mut_ptr(),
        512,
        addr_of_mut!((*ic).ok),
        addr_of_mut!((*ic).w),
        addr_of_mut!((*ic).h),
        addr_of_mut!((*ic).fmt),
        addr_of_mut!((*ic).tlut_n),
        addr_of_mut!((*ic).tlut_fmt),
    );
}

/// `fe_np_pick_char`: open VS mode's CSS seeded with this player's last pick.
pub unsafe fn fe_np_pick_char() {
    put(addr_of_mut!(fe_np_scene) as *mut i8, &std::format!("mode=vs;at=css;p1=ck:{}/c{}/hu", fe_np_ck, fe_np_color));
    fe_np_open(1, addr_of!(fe_np_scene) as *const i8);
}

/// A notice over the description strip ("You left the room.").
pub unsafe fn fe_ol_notice(s: *const i8, bad: i32) {
    let note = addr_of_mut!(fe_ol_note) as *mut i8;
    let mut i = 0usize;
    while !s.is_null() && *s.add(i) != 0 && i < FE_HELP_STR - 1 {
        *note.add(i) = *s.add(i);
        i += 1;
    }
    *note.add(i) = 0;
    fe_ol_note_bad = bad;
    fe_ol_note_frames = 0;
}

pub unsafe extern "C" fn fe_ol_host() {
    if ext::Netplay_MenuBegin(1, fe_np_ck, fe_np_color, fe_np_stocks, fe_np_minutes, fe_np_delay) == 0 {
        fe_np_phase = FE_NP_WORKING;
        fe_switch_screen(addr_of!(fe_screen_wait));
    } else {
        let mut why = [0i8; FE_HELP_STR];
        ext::Netplay_MenuStatus(why.as_mut_ptr(), FE_HELP_STR as i32);
        fe_ol_notice(why.as_ptr(), 1);
        fe_np_phase = FE_NP_IDLE;
    }
}
pub unsafe extern "C" fn fe_np_get_stage_mode() -> i32 {
    ext::Netplay_StageMode()
}
pub unsafe extern "C" fn fe_np_set_stage_mode(v: i32) {
    ext::Netplay_SetStageMode(v);
}
pub unsafe extern "C" fn fe_np_get_turbo() -> i32 {
    ext::Netplay_TurboPref()
}
pub unsafe extern "C" fn fe_np_set_turbo(v: i32) {
    ext::Netplay_SetTurbo(v);
    ext::Settings_SetInt(c"turbo_online".as_ptr(), v);
    ext::OSReport(c"frontend: online Turbo rule -> %s%c".as_ptr(), &[Va::P(if v != 0 { c"on".as_ptr() } else { c"off".as_ptr() }.cast()), Va::I32(10)]);
}
pub unsafe extern "C" fn fe_np_get_envoy() -> i32 {
    ext::Netplay_EnvoyPref()
}
pub unsafe extern "C" fn fe_np_set_envoy(mut v: i32) {
    if v != 0 && ext::Netplay_EnvoyAvailable() == 0 {
        // no Envoy mod loaded here: a room that could never start its match
        ext::OSReport(c"frontend: online Envoy rules stay off - the Envoy mod is not loaded%c".as_ptr(), &[Va::I32(10)]);
        v = 0;
    }
    ext::Netplay_SetEnvoy(v);
    ext::Settings_SetInt(c"envoy_online".as_ptr(), v);
    ext::OSReport(c"frontend: online Envoy rules -> %s%c".as_ptr(), &[Va::P(if v != 0 { c"on".as_ptr() } else { c"off".as_ptr() }.cast()), Va::I32(10)]);
}
pub unsafe extern "C" fn fe_ol_random() {
    if ext::Netplay_RandomBegin(fe_np_ck, fe_np_color, fe_np_stocks, fe_np_minutes, fe_np_delay) == 0 {
        fe_np_phase = FE_NP_WORKING;
        fe_switch_screen(addr_of!(fe_screen_wait));
    } else {
        let mut why = [0i8; FE_HELP_STR];
        ext::Netplay_MenuStatus(why.as_mut_ptr(), FE_HELP_STR as i32);
        fe_ol_notice(why.as_ptr(), 1);
        fe_np_phase = FE_NP_IDLE;
    }
}
pub unsafe extern "C" fn fe_ol_join() {
    if ext::Netplay_MenuHasServer() == 0 {
        fe_ol_notice(c"Online play needs the server address (netplay_server.txt beside the game).".as_ptr(), 1);
        return;
    }
    ext::fl_code_reset();
    fe_switch_screen(addr_of!(fe_screen_code));
}

// ---- read-only view for the scripting engine ---------------------------------------------------------

pub unsafe fn Frontend_ScreenTitle() -> *const i8 {
    if !fe.screen.is_null() { (*fe.screen).title } else { c"".as_ptr() }
}
pub unsafe fn Frontend_ScreenSubtitle() -> *const i8 {
    if !fe.screen.is_null() { (*fe.screen).subtitle } else { c"".as_ptr() }
}
pub unsafe fn Frontend_Cursor() -> i32 {
    if !fe.screen.is_null() && fe.n_vis > 0 { fe.cursor } else { -1 }
}
pub unsafe fn Frontend_CursorLabel() -> *const i8 {
    if fe.screen.is_null() || fe.n_vis <= 0 || (*fe.screen).items.is_null() {
        return c"".as_ptr();
    }
    if fe.has_button != 0 && fe.cursor >= fe.n_list {
        return c"CONTINUE".as_ptr();
    }
    let idx = fe.vis[fe.cursor as usize];
    if idx >= 0 && idx < (*fe.screen).n_items {
        (*(*fe.screen).items.offset(idx as isize)).label
    } else {
        c"".as_ptr()
    }
}

/// Back from the CSS the lobby sent this player to: tell the room, and back to the lobby.
pub unsafe fn Frontend_OnlinePicked(which: i32, a: i32, b: i32) {
    if which == 1 && a >= 0 {
        fe_np_ck = a;
        fe_np_color = b;
    }
    let gone = if ext::Netplay_LobbyActive() != 0 { c"" } else { c" (the room is gone)" };
    ext::OSReport(c"frontend: online pick %d -> %d/%d%s\n".as_ptr(), &[Va::I32(which), Va::I32(a), Va::I32(b), Va::P(gone.as_ptr().cast())]);
    fe_np_pick = 0;
    ext::SceneLaunch_SetText(null());
    if ext::Netplay_LobbyActive() != 0 {
        if which == 1 && a >= 0 && ext::Netplay_FighterAvailable(a) == 0 {
            let mut why = [0i8; 200];
            // Geno slice 7: say which fighter and what differs
            if ext::Netplay_FighterWhy(a, why.as_mut_ptr(), 200) == 0 && why[0] != 0 {
                ext::fl_lobby_notice(why.as_ptr());
            } else {
                ext::fl_lobby_notice(c"Your opponent doesn't have that fighter - pick another.".as_ptr());
            }
        } else if which == 1 && a >= 0 {
            ext::Netplay_LobbyChar(a, b);
        }
        fe.screen = addr_of!(fe_screen_lobby);
    } else {
        // the room went away while this player was on the CSS
        fe.screen = addr_of!(fe_screen_online);
    }
    fe.continue_to = GM_VS as u8;
    fe.back_to = GM_MENU as u8;
    fe.next_menus = 0;
}

/// After an online match in a room: the results screen comes back to the lobby.
pub unsafe fn Frontend_BackToOnline() {
    fe_np_pick = 0;
    fe_np_phase = FE_NP_IDLE;
    ext::SceneLaunch_SetText(null());
    fe.screen = addr_of!(fe_screen_lobby);
    fe.continue_to = GM_VS as u8;
    fe.back_to = GM_MENU as u8;
    fe.next_menus = 0;
    ext::fl_rejoin_reset();
}

pub unsafe fn fe_np_open(which: i32, scene: *const i8) {
    fe_np_pick = which;
    ext::SceneLaunch_SetText(scene);
    fe.continue_to = GM_VS as u8;
    fe.leaving = 1; // fade out into VS mode, which opens at the CSS / SSS
}

// ---- how the other game modes reach the frontend ---------------------------------------------------

pub unsafe fn gmFrontend_SelectScene(state: *mut GameModeState, sss: i32) {
    if state.is_null() {
        return;
    }
    (*state).info.scene_kind = if ext::Frontend_NativeSelect() != 0 {
        if sss != 0 { GS_SSS as u8 } else { GS_CSS as u8 }
    } else {
        GS_FRONTEND
    };
}

/// Training's CSS / SSS states call this from their on_enter.
pub unsafe fn gmFrontend_TrainingSelect(state: *mut GameModeState, sss: i32) {
    if state.is_null() || ext::Frontend_TrainingSelect() == 0 {
        return;
    }
    if sss != 0 {
        fe_train_sss = ext::gm_GetGameModeStateEnterData(state);
    } else {
        fe_train_css = ext::gm_GetGameModeStateEnterData(state);
    }
    (*state).info.scene_kind = GS_FRONTEND;
}

/// A VS-machinery mode that wants the kit's select under its own name (Geno's LAB).
pub unsafe fn gmFrontend_ModeSelect(state: *mut GameModeState, _sss: i32, name: *const i8) {
    if state.is_null() {
        return;
    }
    fe_sel_mode_pending = name;
    (*state).info.scene_kind = GS_FRONTEND;
}

/// THE ONE ENTRY POINT of the Atlas character and stage select for the modes that have their own
/// CSS / SSS state.
pub unsafe fn gmFrontend_AtlasSelect(state: *mut GameModeState, sss: i32, name: *const i8) {
    if state.is_null() || ext::Ui_Ready() == 0 || ext::Ui_SelCanOpen() == 0 || ext::Frontend_NativeSelect() != 0 {
        return;
    }
    if sss == 0 {
        let css = ext::gm_GetGameModeStateEnterData(state) as *mut CSSData;
        if css.is_null() {
            return;
        }
        let mt = (*css).match_type as i32;
        if ext::Ui_CssKnown(mt) == 0 {
            return;
        }
        if ((0xB..=0xD).contains(&mt) || (0xF..=0x16).contains(&mt)) && ext::Settings_Int(c"atlas_select_solo".as_ptr(), 0) == 0 {
            return;
        }
        fe_ats_css = css.cast();
    } else {
        fe_ats_sss = ext::gm_GetGameModeStateEnterData(state);
    }
    if !name.is_null() {
        fe_sel_mode_pending = name; // a name another entry already queued is never wiped by a mode that has none
    }
    (*state).info.scene_kind = GS_FRONTEND;
}

pub unsafe fn gmFrontend_BeginLoading() {
    fe.screen = addr_of!(fe_screen_loading);
    fe.loading = 1;
    fe.next_menus = 0;
}

pub unsafe fn gmFrontend_TakeWarmed() -> i32 {
    let w = fe.warmed;
    fe.warmed = 0;
    w
}

pub unsafe fn fe_enabled() -> i32 {
    if fe_enabled_cache < 0 {
        fe_enabled_cache = ext::PcFrontendEnabled();
    }
    fe_enabled_cache
}

/// The menu tree replaces GM_MENU's list screens: MELEE_FRONTEND_MENUS (default on).
pub unsafe fn fe_menus_on() -> i32 {
    if fe_menus_on_cache < 0 {
        fe_menus_on_cache = (fe_enabled() != 0 && ext::PcFrontendMenusEnabled() != 0) as i32;
    }
    fe_menus_on_cache
}

pub unsafe fn fe_rules_enabled() -> i32 {
    fe_enabled()
}

/// Called by the top-level mode loop on every mode change: returns `to`, or GM_FRONTEND when a
/// frontend screen is placed between `from` and `to`.
pub unsafe fn gmFrontend_Route(from: u8, to: u8) -> u8 {
    if fe_np_pick != 0 && to == GM_MENU as u8 {
        fe_np_pick = 0; // left the CSS/SSS for the main menu: the online pick is abandoned
        ext::SceneLaunch_SetText(null());
    }
    if fe_enabled() == 0 {
        return to;
    }
    if fe_menus_on() != 0 {
        if to == GM_MENU as u8 {
            let mut kind = 0u8;
            let mut sel = 0u8;
            if fm_route_native() != 0 {
                return to; // the frontend asked GM_MENU for one of its native screens
            }
            // wherever GM_MENU would open, the frontend opens instead - unless that is a screen
            // it does not draw (the Event list after an event), which stays native
            if fm_route_position(if from == GM_FRONTEND { fe.reported } else { from }, &mut kind, &mut sel) != 0 {
                fm_route_to_menus(kind, sel);
                ext::OSReport(
                    c"frontend: mode %d -> GM_MENU becomes the menus (kind %d, sel %d)\n".as_ptr(),
                    &[Va::I32(from as i32), Va::I32(kind as i32), Va::I32(sel as i32)],
                );
                return GM_FRONTEND;
            }
            return to;
        }
        if to == GM_FRONTEND {
            return to; // a native screen backed out to a menu of ours, or MATCH SETUP
        }
    }
    if from == GM_FRONTEND {
        return to; // leaving a screen goes exactly where it chose
    }
    let rules = addr_of!(fe_rules) as *const FrontendRule;
    for i in 0..1usize {
        if (*rules.add(i)).from == from && (*rules.add(i)).to == to {
            fe.screen = (*rules.add(i)).screen;
            fe.continue_to = to;
            fe.back_to = from;
            fe.reported = to;
            fe.next_menus = 0;
            ext::OSReport(
                c"frontend: mode %d -> %d, showing \"%s\" first\n".as_ptr(),
                &[Va::I32(from as i32), Va::I32(to as i32), Va::P((*fe.screen).subtitle.cast())],
            );
            return GM_FRONTEND;
        }
    }
    to
}

pub unsafe fn gmFrontend_ReportedMode() -> u8 {
    fe.reported
}

// ---- layout and drawing -------------------------------------------------------------------------------

/// `fe_canvas_width`: the canvas height times the window's aspect (a float's bits from the host).
pub unsafe fn fe_canvas_width() -> f32 {
    let v = ext::View_AspectBits();
    FE_H * f32::from_bits(v as u32)
}
pub unsafe fn fe_canvas_left() -> f32 {
    (FE_W - fe_canvas_width()) * 0.5f32
}
pub unsafe fn fe_canvas_extra() -> f32 {
    let extra = -fe_canvas_left();
    if extra > 0.0f32 { extra } else { 0.0f32 }
}

fn ink() -> GXColor {
    fe_rgba(10, 14, 24, 255)
}
fn cobalt_dk() -> GXColor {
    fe_rgba(20, 38, 92, 255)
}
fn cobalt() -> GXColor {
    fe_rgba(30, 58, 140, 255)
}
fn cobalt_lt() -> GXColor {
    fe_rgba(47, 85, 184, 255)
}
fn gold() -> GXColor {
    fe_rgba(240, 180, 41, 255)
}
fn gold_lt() -> GXColor {
    fe_rgba(255, 215, 102, 255)
}
fn bone() -> GXColor {
    fe_rgba(242, 239, 228, 255)
}
fn help_color() -> GXColor {
    fe_rgba(150, 162, 184, 255)
}

/// A rectangle in 640x480 screen space, y down, with a vertical gradient. The canvas camera is
/// ortho with y running 0..-480, hence the negation.
pub unsafe fn fe_rect(x: f32, y: f32, w: f32, h: f32, top: GXColor, bottom: GXColor) {
    ext::GXBegin(GX_QUADS, GX_VTXFMT0, 4);
    ext::GXPosition2f32(x, -y);
    ext::GXColor4u8(top.r, top.g, top.b, top.a);
    ext::GXPosition2f32(x + w, -y);
    ext::GXColor4u8(top.r, top.g, top.b, top.a);
    ext::GXPosition2f32(x + w, -(y + h));
    ext::GXColor4u8(bottom.r, bottom.g, bottom.b, bottom.a);
    ext::GXPosition2f32(x, -(y + h));
    ext::GXColor4u8(bottom.r, bottom.g, bottom.b, bottom.a);
}

pub unsafe fn fe_solid(x: f32, y: f32, w: f32, h: f32, c: GXColor) {
    fe_rect(x, y, w, h, c, c);
}

/// A small arrowhead pointing left (dir < 0) or right, centred on (x, y). A quad with two
/// coincident corners is a triangle. `x + s * 0.5` and `y +- size * 0.6` are fused sites.
pub unsafe fn fe_arrow(x: f32, y: f32, size: f32, dir: i32, c: GXColor) {
    let s = if dir < 0 { -size } else { size };
    let tip = fmadds(s, 0.5f32, x);
    let s2 = if dir < 0 { -size } else { size };
    let base = fmadds(-s2, 0.5f32, x);
    ext::GXBegin(GX_QUADS, GX_VTXFMT0, 4);
    ext::GXPosition2f32(base, -fmadds(-size, 0.6f32, y));
    ext::GXColor4u8(c.r, c.g, c.b, c.a);
    ext::GXPosition2f32(tip, -y);
    ext::GXColor4u8(c.r, c.g, c.b, c.a);
    ext::GXPosition2f32(tip, -y);
    ext::GXColor4u8(c.r, c.g, c.b, c.a);
    ext::GXPosition2f32(base, -fmadds(size, 0.6f32, y));
    ext::GXColor4u8(c.r, c.g, c.b, c.a);
}

/// Load one art-pack texture into the scene heap; a missing one stays flat (`ok` = 0).
pub unsafe fn fe_tex_load(t: *mut FeTex, name: *const i8) {
    let h = ext::GxTex_OpenUI(name);
    (*t).ok = 0;
    (*t).lut = null_mut();
    if h < 0 {
        return;
    }
    (*t).data = ext::HSD_MemAlloc(ext::GxTex_ImageSize(h));
    let tfmt = ext::GxTex_TlutFormat(h);
    if tfmt >= 0 {
        (*t).lut = ext::HSD_MemAlloc(ext::GxTex_TlutSize(h));
    }
    if !(*t).data.is_null() && (tfmt < 0 || !(*t).lut.is_null()) {
        ext::GxTex_CopyImage(h, (*t).data);
        (*t).w = ext::GxTex_Width(h) as u16;
        (*t).h = ext::GxTex_Height(h) as u16;
        if tfmt >= 0 {
            // a colour-indexed texture: its palette loads into TLUT0 before each draw
            ext::GxTex_CopyTlut(h, (*t).lut);
            ext::GXInitTlutObj(addr_of_mut!((*t).tlut), (*t).lut, tfmt as GXTlutFmt, ext::GxTex_TlutEntries(h) as u16);
            ext::GXInitTexObjCI(addr_of_mut!((*t).obj), (*t).data, (*t).w, (*t).h, ext::GxTex_Format(h) as GXTexFmt, GX_CLAMP, GX_CLAMP, 0, GX_TLUT0 as u32);
        } else {
            ext::GXInitTexObj(addr_of_mut!((*t).obj), (*t).data, (*t).w, (*t).h, ext::GxTex_Format(h) as GXTexFmt, GX_CLAMP, GX_CLAMP, 0);
        }
        ext::GXInitTexObjLOD(addr_of_mut!((*t).obj), GX_LINEAR, GX_LINEAR, 0.0f32, 0.0f32, 0.0f32, 0, 0, GX_ANISO_1);
        (*t).ok = 1;
    }
    ext::GxTex_Close(h);
}

/// A textured quad with explicit texture coordinates (a flipped 9-slice edge swaps them), tinted
/// by `c`. It sets its own GX state and then invalidates HSD's state cache and restores the
/// untextured vertex-colour setup the rest of the panels draw with.
pub unsafe fn fe_tex_quad_uv(t: *mut FeTex, x: f32, y: f32, w: f32, h: f32, u0: f32, v0: f32, u1: f32, v1: f32, c: GXColor) {
    if (*t).ok == 0 {
        return;
    }
    ext::GXClearVtxDesc();
    ext::GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
    ext::GXSetVtxDesc(GX_VA_CLR0, GX_DIRECT);
    ext::GXSetVtxDesc(GX_VA_TEX0, GX_DIRECT);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XY, GX_F32, 0);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_TEX0, GX_TEX_ST, GX_F32, 0);
    ext::GXSetNumTexGens(1);
    // GXSetTexCoordGen (a header inline)
    ext::GXSetTexCoordGen2(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY as u32, 0, GX_PTIDENTITY as u32);
    ext::GXSetNumIndStages(0);
    ext::GXSetNumTevStages(1);
    ext::GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
    ext::GXSetTevOp(GX_TEVSTAGE0, GX_MODULATE);
    ext::GXSetNumChans(1);
    ext::GXSetChanCtrl(GX_COLOR0A0, 0, GX_SRC_REG, GX_SRC_VTX, GX_LIGHT_NULL as u32, GX_DF_NONE, GX_AF_NONE);
    ext::GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_NOOP);
    ext::GXSetAlphaCompare(GX_ALWAYS, 0, GX_AOP_AND, GX_ALWAYS, 0);
    ext::GXSetZMode(0, GX_ALWAYS, 0);
    ext::GXSetCullMode(GX_CULL_NONE);
    if !(*t).lut.is_null() {
        ext::GXLoadTlut(addr_of_mut!((*t).tlut), GX_TLUT0 as u32);
    }
    ext::GXLoadTexObj(addr_of_mut!((*t).obj), GX_TEXMAP0);
    ext::GXBegin(GX_QUADS, GX_VTXFMT0, 4);
    ext::GXPosition2f32(x, -y);
    ext::GXColor4u8(c.r, c.g, c.b, c.a);
    ext::GXTexCoord2f32(u0, v0);
    ext::GXPosition2f32(x + w, -y);
    ext::GXColor4u8(c.r, c.g, c.b, c.a);
    ext::GXTexCoord2f32(u1, v0);
    ext::GXPosition2f32(x + w, -(y + h));
    ext::GXColor4u8(c.r, c.g, c.b, c.a);
    ext::GXTexCoord2f32(u1, v1);
    ext::GXPosition2f32(x, -(y + h));
    ext::GXColor4u8(c.r, c.g, c.b, c.a);
    ext::GXTexCoord2f32(u0, v1);
    ext::HSD_StateInvalidate(-1);
    ext::hsd_80391A04(1.0f32, 1.0f32, 1);
}

pub unsafe fn fe_tex_quad(t: *mut FeTex, x: f32, y: f32, w: f32, h: f32, c: GXColor) {
    fe_tex_quad_uv(t, x, y, w, h, 0.0f32, 0.0f32, 1.0f32, 1.0f32, c);
}

/// Legacy panels, rows and buttons share the grid too. The source grid uses their old display
/// sizes, preserving UV placement at those sizes exactly.
pub unsafe fn fe_tex_or_solid(which: i32, x: f32, y: f32, w: f32, h: f32, fallback: GXColor) {
    let mut bounds = FePanelRect::default();
    let mut aw = FE_BTN_W;
    let mut ah = FE_BTN_H;
    let mut cx = FE_BTN_H * 0.5f32;
    let mut cy = 8.0f32;
    if which as usize == FT_PANEL {
        aw = FE_PANEL_W;
        ah = FE_PANEL_H;
        cx = 32.0;
        cy = 32.0;
    } else if which as usize == FT_ROW || which as usize == FT_ROW_SEL {
        aw = FE_ROW_W;
        ah = FE_ROW_H;
        cx = 32.0;
        cy = 8.0;
    }
    ext::fe_panel_rect(&mut bounds, x, y, w, h);
    ext::fe_panel_art_image(which, &bounds, aw, ah, cx, cy, fallback);
}

/// The toolkit screens are drawn with the kit when its files are there.
pub unsafe fn fe_kit_available() -> i32 {
    ext::kit_load();
    ext::ff_load();
    (ff.state == 1 && kit.state == 1) as i32
}

pub unsafe fn fm_route_to_menus(kind: u8, sel: u8) {
    fm.start_kind = kind;
    fm.start_sel = sel;
    fe.next_menus = 1;
}

pub unsafe fn fm_route_native() -> i32 {
    fm.native_req
}

pub unsafe fn fm_route_position(previous: u8, kind: *mut u8, sel: *mut u8) -> i32 {
    ext::fm_position_for(previous, kind, sel);
    ext::fm_replaced(*kind as i32)
}

/// VS > Melee from the menus: the MATCH SETUP screen as its own GS_FRONTEND scene.
pub unsafe fn fe_match_setup_from_menus() {
    fe.screen = addr_of!(fe_screen_vs_setup);
    fe.continue_to = GM_VS as u8;
    fe.back_to = GM_MENU as u8;
    fe.reported = GM_MENU as u8;
    fm_back_to_online_item = 0;
    fe.next_menus = 0;
    ext::fm_leave_scene(GM_FRONTEND);
}

pub unsafe fn gmFrontend_NativeRequest(kind: *mut u8, sel: *mut u8) -> i32 {
    if fm.native_req == 0 {
        return 0;
    }
    *kind = fm.native_kind;
    *sel = fm.native_sel;
    1
}

pub unsafe fn gmFrontend_TakeNativeRequest(kind: *mut u8, sel: *mut u8) -> i32 {
    if gmFrontend_NativeRequest(kind, sel) == 0 {
        return 0;
    }
    fm.native_req = 0;
    1
}

pub unsafe fn gmFrontend_NativeReturn(kind: i32, sel: i32) -> i32 {
    if fe_menus_on() == 0 || ext::fm_replaced(kind) == 0 {
        return 0;
    }
    if ext::Ui_Ready() != 0 && kind == MENU_KIND_SETTINGS as i32 && sel == SEL_SETTINGS_LANG as i32 {
        fe_settings_resume_game(); // Atlas: back to the tabs, on GAME at Language
        return 1;
    }
    if ext::Ui_Ready() != 0 && kind == MENU_KIND_VS as i32 && sel == SEL_VS_RULES as i32 {
        ext::fe_rules_resume(); // Atlas: the game's own Rules screen backs out to the Atlas Rules screen
        return 1;
    }
    ext::OSReport(c"frontend: native screen backs out to (kind %d, sel %d) - back to the menus\n".as_ptr(), &[Va::I32(kind), Va::I32(sel)]);
    fm_route_to_menus(kind as u8, sel as u8);
    1
}

/// The state's on_enter: when this scene is the menu tree, what gmmenumode.c's onEnter does.
pub unsafe extern "C" fn fe_state_enter(_state: *mut GameModeState) {
    if fe.next_menus == 0 {
        return;
    }
    ext::lbCardNew_AllocWorkArea();
    ext::lbCardGame_LoadArchive(0);
    ext::lbDvd_80018C6C();
    ext::lbDvd_8001823C();
    ext::lbDvd_80018254();
}

/// How far row `slot` has slid in: rows arrive one after another.
pub unsafe fn fe_row_offset(slot: i32) -> f32 {
    let t = (fe.frames - 2 * slot) as f32 / 12.0f32;
    (1.0f32 - fe_ease(t)) * 48.0f32
}

pub unsafe fn fe_rows_shown() -> i32 {
    if fe.n_list < FE_MAX_ROWS as i32 { fe.n_list } else { FE_MAX_ROWS as i32 }
}

/// The 9-slice frame around the screen.
pub unsafe fn fe_draw_frame() {
    let l = fe_canvas_left() + FE_FRAME_M;
    let w = fe_canvas_width() - 2.0 * FE_FRAME_M;
    let bounds = FePanelRect { x: l, y: FE_FRAME_M, w, h: FE_H - 2.0 * FE_FRAME_M };
    ext::fe_panel_art_frame(&bounds, 64.0f32, 16.0f32, 0, 0);
}

pub unsafe extern "C" fn fe_draw_panels(_gobj: *mut HSD_GObj, pass: i32) {
    if pass != 0 {
        return;
    }
    if fm.active != 0 || fk.on != 0 || fl.on != 0 {
        ext::fp_draw();
        if fk.on != 0 {
            ext::fp_scroll_cue(fk.scroll, fk.nrows);
        } else if fm.active != 0 && (*fm.menu).style == FM_LIST {
            ext::fp_scroll_cue(fm_scroll, fm.nvis);
        }
        return;
    }
    if fe.screen.is_null() || (*fe.screen).art != 0 || ext::fas_active() != 0 {
        return; // a room / select screen has no rows, and the Atlas select draws everything itself
    }
    ext::hsd_80391A04(1.0f32, 1.0f32, 1);

    fe_solid(fe_canvas_left(), 0.0, fe_canvas_width(), FE_H, ink());
    fe_draw_frame();

    let mut pulse = (fe.frames % 60) as f32 / 60.0f32;
    pulse = if pulse < 0.5f32 { pulse * 2.0f32 } else { (1.0f32 - pulse) * 2.0f32 };

    fe_tex_or_solid(FT_PANEL as i32, FE_PANEL_X, FE_PANEL_Y, FE_PANEL_W, FE_PANEL_H, cobalt());

    let mut slot = 0;
    while slot < fe_rows_shown() {
        fe_tex_or_solid(FT_ROW as i32, FE_ROW_X + fe_row_offset(slot), fe_row_y(slot), FE_ROW_W, FE_ROW_H, cobalt_dk());
        slot += 1;
    }
    // the cursor's plate, drawn where it currently is on its way to the cursor
    if fe.cursor < fe.n_list {
        let y = fmadds(FE_ROW_STEP, fe.hl_y, FE_ROW_Y);
        let s0 = (fe.hl_y + 0.5f32) as i32;
        fe_tex_or_solid(FT_ROW_SEL as i32, FE_ROW_X + fe_row_offset(s0), y, FE_ROW_W, FE_ROW_H, cobalt_lt());
    }

    slot = 0;
    while slot < fe_rows_shown() {
        let idx = fe.vis[(fe.scroll + slot) as usize];
        let it = (*fe.screen).items.offset(idx as isize);
        let x = FE_ROW_X + fe_row_offset(slot);
        let y = fe_row_y(slot);
        let cy = y + 14.0f32; // the plate's middle
        let selected = (fe.scroll + slot) == fe.cursor;

        // change arrows on the selected value
        if selected && ((*it).kind == FE_CHOICE as u8 || (*it).kind == FE_SLIDER as u8) {
            fe_arrow((x + FE_ROW_W) - 28.0f32, cy, 10.0, 1, gold_lt());
            fe_arrow(if (*it).kind == FE_SLIDER as u8 { x + 226.0f32 } else { x + 318.0f32 }, cy, 10.0, -1, gold_lt());
        }

        // the value's own furniture: a slider track, a toggle pill - ink-outlined, hard stops
        if (*it).kind == FE_SLIDER as u8 && (*it).get.is_some() && (*it).max > (*it).min {
            let frac = ((*it).get.unwrap()() - (*it).min) as f32 / ((*it).max - (*it).min) as f32;
            let tx = x + 240.0f32;
            let tw = 150.0f32;
            let ty = cy - 3.0f32;
            ext::fe_panel_solid(tx - 2.0f32, ty - 2.0f32, tw + 4.0f32, 10.0, ink());
            ext::fe_panel_solid(tx, ty, tw * frac, 6.0, gold());
            ext::fe_panel_solid(fmadds(tw, frac, tx) - 5.0f32, ty - 6.0f32, 10.0, 18.0, ink());
            ext::fe_panel_solid(fmadds(tw, frac, tx) - 3.0f32, ty - 4.0f32, 6.0, 14.0, bone());
        } else if (*it).kind == FE_TOGGLE as u8 && (*it).get.is_some() {
            let on = ((*it).get.unwrap()() != 0) as i32;
            let px = x + 240.0f32;
            let py = cy - 9.0f32;
            ext::fe_panel_solid(px - 2.0f32, py - 2.0f32, 44.0, 22.0, ink());
            ext::fe_panel_solid(px, py, 40.0, 18.0, if on != 0 { gold() } else { cobalt_dk() });
            ext::fe_panel_solid(if on != 0 { px + 23.0f32 } else { px + 3.0f32 }, py + 3.0f32, 14.0, 12.0, bone());
        }
        slot += 1;
    }

    // CONTINUE, overlapping the panel's corner: hover while the cursor is on it, pressed as the
    // screen leaves through it; the hand points at it
    if fe.has_button != 0 {
        let on = (fe.cursor == fe.n_vis - 1) as i32;
        let which = if fe.leaving == 1 {
            FT_BTN_PRESS
        } else if on != 0 {
            FT_BTN_HOVER
        } else {
            FT_BTN
        };
        fe_tex_or_solid(which as i32, FE_BTN_X, FE_BTN_Y, FE_BTN_W, FE_BTN_H, if on != 0 { cobalt_lt() } else { cobalt() });
        if on != 0 && fe.leaving == 0 {
            let bob = 3.0f32 * pulse;
            fe_tex_quad(
                addr_of_mut!(fe_tex[FT_CURSOR]),
                FE_BTN_X + FE_BTN_W * 0.82f32,
                (FE_BTN_Y + FE_BTN_H * 0.48f32) + bob,
                32.0,
                32.0,
                fe_rgba(255, 255, 255, 255),
            );
        }
    }

    // footer hints: the controller's own buttons beside their labels
    fe_tex_quad(addr_of_mut!(fe_tex[FT_GLYPH_A]), FE_HINT_A_X, FE_HINT_Y, 24.0, 24.0, fe_rgba(255, 255, 255, 255));
    fe_arrow(FE_HINT_LR_X + 6.0, FE_HINT_Y + 12.0, 12.0, -1, gold());
    fe_arrow(FE_HINT_LR_X + 24.0, FE_HINT_Y + 12.0, 12.0, 1, gold());
    fe_tex_quad(addr_of_mut!(fe_tex[FT_GLYPH_B]), FE_HINT_B_X, FE_HINT_Y, 24.0, 24.0, fe_rgba(255, 255, 255, 255));

    // scroll marks, when the list runs past the rows shown
    if fe.scroll > 0 {
        ext::fe_panel_solid(FE_W * 0.5f32 - 12.0, FE_ROW_Y - 9.0, 24.0, 4.0, gold());
    }
    if fe.scroll + fe_rows_shown() < fe.n_list {
        ext::fe_panel_solid(FE_W * 0.5f32 - 12.0, fe_row_y(FE_MAX_ROWS as i32) + 1.0, 24.0, 4.0, gold());
    }
}

pub unsafe extern "C" fn fe_draw_fade(_gobj: *mut HSD_GObj, pass: i32) {
    if pass != 2 {
        return;
    }
    if fm.active != 0 {
        if fm.fade > 0 {
            ext::hsd_80391A04(1.0f32, 1.0f32, 1);
            fe_solid(fe_canvas_left(), 0.0, fe_canvas_width(), FE_H, fe_rgba(0, 0, 0, (255 * fm.fade / FM_FADE) as u8));
        }
        return;
    }
    if fe.fade <= 0 {
        return;
    }
    ext::hsd_80391A04(1.0f32, 1.0f32, 1);
    fe_solid(fe_canvas_left(), 0.0, fe_canvas_width(), FE_H, fe_rgba(0, 0, 0, (255 * fe.fade / FE_FADE_FRAMES) as u8));
}

// ---- text ---------------------------------------------------------------------------------------------

pub unsafe fn fe_text(x: f32, y: f32, scale: f32, align: u8, mut color: GXColor, s: *const i8) -> *mut HSD_Text {
    let t = ext::HSD_SisLib_803A6754(0, fe.canvas);
    if t.is_null() {
        return null_mut();
    }
    (*t).pos_x = x;
    (*t).pos_y = y;
    (*t).pos_z = 0.0f32;
    (*t).font_size.x = scale;
    (*t).font_size.y = scale;
    (*t).default_kerning = 1;
    (*t).default_alignment = align;
    let n = ext::HSD_SisLib_803A6B98(t, 0.0f32, 0.0f32, s, &[]);
    ext::HSD_SisLib_803A74F0(t, n, &mut color);
    t
}

/// Replace a text's string: only when it changes (`cur` keeps what is shown), as a fresh text
/// object with the old one's position, size and colour.
pub unsafe fn fe_set_text(pt: *mut *mut HSD_Text, cur: *mut i8, cap: i32, s: *const i8, color: GXColor) {
    let old = *pt;
    if old.is_null() {
        return;
    }
    let mut i = 0usize;
    while *cur.add(i) == *s.add(i) && *s.add(i) != 0 && (i as i32) < cap - 1 {
        i += 1;
    }
    if *cur.add(i) == *s.add(i) {
        return; // unchanged
    }
    let t = fe_text((*old).pos_x, (*old).pos_y, (*old).font_size.x, (*old).default_alignment, color, if *s != 0 { s } else { c" ".as_ptr() });
    if t.is_null() {
        return;
    }
    (*t).hidden = (*old).hidden;
    ext::HSD_SisLib_803A5CC4(old);
    *pt = t;
    i = 0;
    while *s.add(i) != 0 && (i as i32) < cap - 1 {
        *cur.add(i) = *s.add(i);
        i += 1;
    }
    *cur.add(i) = 0;
}

pub unsafe fn fe_value_string(it: *const FrontendItem, out: *mut i8) {
    *out = 0;
    if (*it).get.is_none() {
        if (*it).kind == FE_ACTION as u8 && (*it).format.is_some() {
            (*it).format.unwrap()(0, out);
        }
        return;
    }
    let v = (*it).get.unwrap()();
    if (*it).format.is_some() && ((*it).kind == FE_ACTION as u8 || (*it).kind == FE_CHOICE as u8) {
        (*it).format.unwrap()(v, out); // a status beside an action, or a choice shown its own way
        return;
    }
    match (*it).kind as u32 {
        FE_CHOICE => {
            if !(*it).options.is_null() && v >= (*it).min && v <= (*it).max {
                put_cstr(out, *(*it).options.offset((v - (*it).min) as isize));
            }
        }
        FE_SLIDER => {
            if (*it).format.is_some() {
                (*it).format.unwrap()(v, out);
            } else {
                put(out, &std::format!("{}", v));
            }
        }
        FE_TOGGLE => {
            put(out, if v != 0 { "On" } else { "Off" });
        }
        _ => {}
    }
}

/// Put the visible items into the row slots: labels, values, positions (rows slide in).
pub unsafe fn fe_refresh_rows() {
    let mut buf = [0i8; FE_STR];
    for slot in 0..FE_MAX_ROWS {
        let l = fe.label[slot];
        let v = fe.value[slot];
        if l.is_null() || v.is_null() {
            continue;
        }
        if slot as i32 >= fe_rows_shown() {
            (*l).hidden = 1;
            (*v).hidden = 1;
            continue;
        }
        {
            let it = (*fe.screen).items.offset(fe.vis[(fe.scroll + slot as i32) as usize] as isize);
            let dx = fe_row_offset(slot as i32);
            fe_set_text(addr_of_mut!(fe.label[slot]), fe.label_str[slot].as_mut_ptr(), FE_HELP_STR as i32, (*it).label, fe_rgba(242, 239, 228, 255));
            fe_value_string(it, buf.as_mut_ptr());
            fe_set_text(addr_of_mut!(fe.value[slot]), fe.value_str[slot].as_mut_ptr(), FE_STR as i32, buf.as_ptr(), gold_lt());
            let l = fe.label[slot];
            let v = fe.value[slot];
            (*l).hidden = 0;
            (*l).pos_x = (FE_ROW_X + 32.0f32) + dx;
            (*l).pos_y = fe_row_y(slot as i32) + 4.0f32;
            (*v).hidden = (buf[0] == 0) as u8;
            (*v).pos_x = FE_VALUE_X + dx;
            (*v).pos_y = fe_row_y(slot as i32) + 4.0f32;
        }
    }
    if fe.n_vis > 0 {
        let idx = fe.vis[fe.cursor as usize];
        let mut h = (*(*fe.screen).items.offset(idx as isize)).help;
        if (fe.screen == addr_of!(fe_screen_online) || ext::fe_is_settings(fe.screen) != 0) && fe_ol_note[0] != 0 {
            h = addr_of!(fe_ol_note) as *const i8;
        }
        fe_set_text(addr_of_mut!(fe.help), fe.help_str.as_mut_ptr(), FE_HELP_STR as i32, if !h.is_null() { h } else { c"".as_ptr() }, help_color());
    }
}

/// Recompute which items are visible (a value can hide or show others), keeping the cursor on
/// the same item where it can.
pub unsafe fn fe_rebuild_visible() {
    let keep = if fe.n_vis > 0 { fe.vis[fe.cursor as usize] } else { -1 };
    let mut button = -1i32;
    fe.n_list = 0;
    let mut i = 0i32;
    while i < (*fe.screen).n_items && fe.n_list < FE_MAX_ITEMS as i32 - 1 {
        let it = (*fe.screen).items.offset(i as isize);
        if (*it).visible.is_some() && (*it).visible.unwrap()() == 0 {
            i += 1;
            continue;
        }
        if (*it).kind == FE_ACTION as u8 && (*it).action == FE_DO_CONTINUE as u8 && fe_kit == 0 {
            button = i; // drawn as the CONTINUE button, not a row (on the kit: the first row)
        } else {
            fe.vis[fe.n_list as usize] = i;
            fe.n_list += 1;
        }
        i += 1;
    }
    fe.n_vis = fe.n_list;
    fe.has_button = (button >= 0) as i32;
    if fe.has_button != 0 {
        fe.vis[fe.n_vis as usize] = button;
        fe.n_vis += 1;
    }
    // the same item where it is still visible, else the nearest list row above it
    fe.cursor = 0;
    i = 0;
    while i < fe.n_vis {
        if fe.vis[i as usize] == keep {
            fe.cursor = i;
            break;
        }
        if i < fe.n_list && fe.vis[i as usize] <= keep {
            fe.cursor = i;
        }
        i += 1;
    }
    if fe.cursor < fe.n_list {
        if fe.cursor < fe.scroll {
            fe.scroll = fe.cursor;
        }
        if fe.cursor >= fe.scroll + FE_MAX_ROWS as i32 {
            fe.scroll = fe.cursor - FE_MAX_ROWS as i32 + 1;
        }
    }
    if fe.scroll > fe.n_list - fe_rows_shown() {
        fe.scroll = fe.n_list - fe_rows_shown();
    }
    if fe.scroll < 0 {
        fe.scroll = 0;
    }
}

// ---- the scene ----------------------------------------------------------------------------------------

/// The text canvas and the two GX-link objects every frontend scene opens with.
unsafe fn open_canvas() {
    fe.canvas = ext::HSD_SisLib_803A611C(0, null_mut(), 0x13, 0x14, 0, FE_GX_LINK, 10, 0);
    ext::HSD_SisLib_SetWideCanvas(0, fe.canvas);
    let mut gobj = ext::GObj_Create(0xE, 0xF, 0);
    if !gobj.is_null() {
        ext::GObj_SetupGXLink(gobj, Some(fe_draw_panels), FE_GX_LINK, 0);
    }
    gobj = ext::GObj_Create(0xE, 0xF, 0);
    if !gobj.is_null() {
        ext::GObj_SetupGXLink(gobj, Some(fe_draw_fade), FE_GX_LINK, 20);
    }
}

pub unsafe extern "C" fn gm_Scene_Frontend_OnEnter(enter_data: *mut c_void) {
    fe.cursor = 0;
    fe.scroll = 0;
    fe.frames = 0;
    fe.fade = FE_FADE_FRAMES;
    fe.leaving = 0;
    fe.hl_y = 0.0f32;
    fe.n_vis = 0;
    fe.help_item = -1;
    let css_data = addr_of_mut!(gmVsMelee_CssData) as *mut c_void;
    let sss_data = addr_of_mut!(gmVsMelee_SssData) as *mut c_void;
    if !enter_data.is_null() && (enter_data == css_data || enter_data == sss_data) {
        // VS mode's CSS / SSS state runs this scene in place of the native screen
        fe.screen = if enter_data == css_data { addr_of!(fe_screen_css) } else { addr_of!(fe_screen_sss) };
        fe.next_menus = 0;
        fe.loading = 0;
        fe_sel_css = css_data.cast();
        fe_sel_sss = sss_data.cast();
        fe_sel_training = 0;
        fe_sel_mode = fe_sel_mode_pending; // a named mode's select, once
        fe_sel_mode_pending = null();
    } else if !enter_data.is_null() && (enter_data == fe_ats_css || enter_data == fe_ats_sss) {
        // a mode's own CSS / SSS state (Classic, Event, Stadium), routed here by gmFrontend_AtlasSelect
        fe.screen = if enter_data == fe_ats_css { addr_of!(fe_screen_css) } else { addr_of!(fe_screen_sss) };
        fe.next_menus = 0;
        fe.loading = 0;
        fe_sel_css = fe_ats_css.cast();
        fe_sel_sss = if !fe_ats_sss.is_null() { fe_ats_sss.cast() } else { sss_data.cast() };
        fe_sel_training = 0;
        fe_sel_mode = fe_sel_mode_pending;
        fe_sel_mode_pending = null();
    } else if !enter_data.is_null() && (enter_data == fe_train_css || enter_data == fe_train_sss) {
        // Training's CSS / SSS state, routed here by gmFrontend_TrainingSelect
        fe.screen = if enter_data == fe_train_css { addr_of!(fe_screen_css) } else { addr_of!(fe_screen_sss) };
        fe.next_menus = 0;
        fe.loading = 0;
        fe_sel_css = fe_train_css.cast();
        fe_sel_sss = fe_train_sss.cast();
        fe_sel_training = 1;
    }
    // The registrations are used up by now: a later scene must never match a stale one.
    fe_ats_css = null_mut();
    fe_ats_sss = null_mut();
    if fe.next_menus != 0 {
        fe.next_menus = 0;
        open_canvas();
        ext::fm_scene_enter();
        return;
    }
    fm.active = 0;
    if fe.screen.is_null() {
        return;
    }
    fe_kit = (fe.loading == 0 && fe_kit_available() != 0) as i32;
    if (*fe.screen).art != 0 {
        // a room screen: its own model, drawn on the panels' link
        open_canvas();
        // this runs in the scene's init, BEFORE Script_SceneBegin
        ext::Ui_SelNextScene(1);
        ext::fl_open((*fe.screen).art);
        ext::Ui_SelNextScene(0);
        return;
    }
    fe_rebuild_visible();
    if fe.has_button != 0 {
        fe.cursor = fe.n_vis - 1; // A straight away continues, as the menus do
    }

    // Only this kit canvas opts into the wide ortho; panels draw below the text, the fade above it.
    open_canvas();
    if fe_kit != 0 {
        ext::fk_build(1);
        return;
    }
    {
        let mut n = 0;
        for i in 0..FT_COUNT {
            fe_tex_load(addr_of_mut!(fe_tex[i]), fe_tex_names[i]);
            n += fe_tex[i].ok;
        }
        ext::OSReport(
            c"frontend: %d of %d art elements loaded%s\n".as_ptr(),
            &[Va::I32(n), Va::I32(FT_COUNT as i32), Va::P(if (n as usize) < FT_COUNT { c" - the missing ones draw flat" } else { c"" }.as_ptr().cast())],
        );
    }

    fe.title = fe_text(44.0, 24.0, 0.9f32, 0, bone(), (*fe.screen).title);
    fe.subtitle = fe_text(FE_PANEL_X + 44.0, FE_PANEL_Y + 7.0, 0.55f32, 0, ink(), (*fe.screen).subtitle); // on the panel's gold tab
    for slot in 0..FE_MAX_ROWS {
        fe.label[slot] = fe_text(FE_ROW_X + 32.0, fe_row_y(slot as i32) + 4.0, 0.56f32, 0, bone(), c" ".as_ptr());
        fe.value[slot] = fe_text(FE_VALUE_X, fe_row_y(slot as i32) + 4.0, 0.56f32, 2, gold_lt(), c" ".as_ptr());
        fe.label_str[slot][0] = b' ' as i8;
        fe.label_str[slot][1] = 0;
        fe.value_str[slot][0] = b' ' as i8;
        fe.value_str[slot][1] = 0;
    }
    fe.help = fe_text(44.0, (FE_PANEL_Y + FE_PANEL_H) + 8.0, 0.5f32, 0, help_color(), c" ".as_ptr());
    fe.help_str[0] = b' ' as i8;
    fe.help_str[1] = 0;
    fe.hints = fe_text(FE_HINT_A_X + 32.0, FE_HINT_Y + 2.0, 0.5f32, 0, bone(), c"Select".as_ptr());
    fe.hint_change = fe_text(FE_HINT_LR_X + 40.0, FE_HINT_Y + 2.0, 0.5f32, 0, bone(), c"Change".as_ptr());
    fe.hint_back = fe_text(FE_HINT_B_X + 32.0, FE_HINT_Y + 2.0, 0.5f32, 0, bone(), c"Back".as_ptr());
    fe_refresh_rows();
}

/// Show another screen in the same scene: the rows, title and subtitle follow.
pub unsafe fn fe_switch_screen(s: *const FrontendScreen) {
    if ext::fss_intercepts(s) != 0 {
        ext::fss_switch(s); // the Atlas settings are up: fss_frame moves the host to this screen
        return;
    }
    if (*s).art != 0 || (!fe.screen.is_null() && (*fe.screen).art != 0) {
        // to or from a room screen: the model is rebuilt either way
        let from = fe.screen;
        fe.screen = s;
        fe.cursor = 0;
        fe.scroll = 0;
        fe.n_vis = 0;
        if (*s).art != 0 {
            ext::fss_close(); // the Atlas ONLINE PLAY screen goes before a room screen asks the host for its own slot
            if from.is_null() || (*from).art == 0 {
                ext::fk_exit(); // the rows' model goes; the room screen builds its own
                fe_kit = 0;
            }
            ext::fl_open((*s).art);
            return;
        }
        ext::fa_room_end(); // leaving a room screen for the rows: the host's room screen goes with it
        fl.on = 0;
        ext::fp_free_textures();
        fp.ok = 0;
        fe_kit = fe_kit_available();
        fe_rebuild_visible();
        if fe_kit != 0 {
            ext::fk_build(1);
        }
        return;
    }
    fe.screen = s;
    fe.cursor = 0;
    fe.scroll = 0;
    fe.hl_y = 0.0f32;
    fe.n_vis = 0;
    fe_rebuild_visible();
    if fe_kit != 0 {
        ext::fk_build(1); // the same scene, the next screen sliding in
        return;
    }
    if !fe.title.is_null() {
        ext::HSD_SisLib_803A5CC4(fe.title);
    }
    if !fe.subtitle.is_null() {
        ext::HSD_SisLib_803A5CC4(fe.subtitle);
    }
    fe.title = fe_text(44.0, 24.0, 0.9f32, 0, bone(), (*s).title);
    fe.subtitle = fe_text(FE_PANEL_X + 44.0, FE_PANEL_Y + 7.0, 0.55f32, 0, ink(), (*s).subtitle);
    fe_refresh_rows();
}

/// VERSUS > ONLINE from the menus: the ONLINE screen as its own GS_FRONTEND scene.
pub unsafe fn fe_online_from_menus() {
    fe.screen = addr_of!(fe_screen_online);
    fe.continue_to = GM_VS as u8;
    fe.back_to = GM_MENU as u8;
    fe.reported = GM_MENU as u8;
    fe.next_menus = 0;
    fe_np_phase = FE_NP_IDLE;
    fm_back_to_online_item = 1;
    ext::fe_settings_apply_online(); // the saved input delay and stage list
    ext::fm_leave_scene(GM_FRONTEND);
}

/// SETTINGS > a page from the menus: that page as its own GS_FRONTEND scene.
pub unsafe fn fe_settings_from_menus(page: i32) {
    if page < 0 || page >= FSP_COUNT {
        return;
    }
    if page == FSP_MODS {
        ext::fsm_fill();
    }
    fa_mods_door = (ext::Ui_Ready() != 0 && fa_page_from_main != 0 && page == FSP_MODS) as i32; // the main menu's MODS row
    ext::fe_settings_apply_online();
    fe_ol_note[0] = 0; // a notice from the last page stays behind
    fe.screen = addr_of!(fe_screen_settings[page as usize]);
    fe.continue_to = GM_MENU as u8;
    fe.back_to = GM_MENU as u8;
    fe.reported = GM_MENU as u8;
    fe.next_menus = 0;
    fm_back_kind = MENU_KIND_SETTINGS as i32;
    fm_back_sel = 0x41 + page;
    if ext::Ui_Ready() != 0 {
        // Atlas: the pages are the tabs of one screen, entered from the main menu's SETTINGS row (or
        // its MODS row, the first boot, the Language return): B returns to that main-menu item.
        fm_back_kind = MENU_KIND_MAIN as i32;
        fm_back_sel = SEL_MAIN_SETTINGS as i32;
        if fa_page_from_main != 0 && page == FSP_MODS {
            fm_back_sel = SEL_MAIN_MODS as i32; // the Atlas main menu's MODS row
        }
    }
    fa_page_from_main = 0;
    ext::fm_leave_scene(GM_FRONTEND);
}

/// The game's Language screen backs out to the tabbed settings, on the GAME tab at Language.
pub unsafe fn fe_settings_resume_game() {
    ext::OSReport(c"frontend: the Language screen backs out to SETTINGS > GAME, on Language\n".as_ptr(), &[]);
    ext::fe_settings_apply_online();
    fe_ol_note[0] = 0;
    fe.screen = addr_of!(fe_screen_settings[FSP_GAMEPLAY]);
    fe.continue_to = GM_MENU as u8;
    fe.back_to = GM_MENU as u8;
    fe.reported = GM_MENU as u8;
    fe.next_menus = 0;
    fm_back_kind = MENU_KIND_MAIN as i32;
    fm_back_sel = SEL_MAIN_SETTINGS as i32;
    fa_page_from_main = 0;
    fss.want_label = c"Language".as_ptr();
}

pub unsafe fn fe_change(it: *const FrontendItem, dir: i32) {
    if (*it).get.is_none() || (*it).set.is_none() {
        return;
    }
    let mut v = (*it).get.unwrap()();
    let n = (*it).max - (*it).min + 1;
    match (*it).kind as u32 {
        FE_CHOICE => {
            v = (*it).min + ((v - (*it).min + dir) % n + n) % n; // wraps
        }
        FE_SLIDER => {
            v += dir * if (*it).step > 0 { (*it).step } else { 1 };
            v = if v < (*it).min { (*it).min } else if v > (*it).max { (*it).max } else { v }; // clamps
        }
        FE_TOGGLE => {
            v = (v == 0) as i32;
        }
        _ => return,
    }
    if fe_kit != 0 {
        ext::fk_note_dir(dir);
        if (*it).kind == FE_SLIDER as u8 && v == (*it).get.unwrap()() {
            ext::fk_bump(dir); // held at an end: the kit's "can't go further"
        }
    }
    (*it).set.unwrap()(v);
    sfxMove();
    fe_rebuild_visible(); // a value can show or hide other rows
}

/// The warm-up behind the loading screen: start counting from what is built now.
pub unsafe fn fe_load_begin() {
    fe.load_frames = 0;
    fe.load_base = ext::Gfx_PipelinesCreated();
    {
        fe.load_core = ext::Gfx_SeedCoreCount();
        let seed = ext::Gfx_SeedPipelinesBuilt();
        let pending = ext::Gfx_PipelinesPending();
        ext::OSReport(
            c"frontend: loading screen - %d pipelines built so far (%d from the seed), %d pending, seed core %d\n".as_ptr(),
            &[Va::I32(fe.load_base), Va::I32(seed), Va::I32(pending), Va::I32(fe.load_core)],
        );
    }
    fe.load_settled = 0;
    fe.progress = 0.0f32;
    fe.percent = -1;
}

/// One frame of the warm-up: the progress bar's value. True when it is time to go.
pub unsafe fn fe_load_step() -> i32 {
    let target: f32;
    fe.load_frames += 1;
    let pending = ext::Gfx_PipelinesUrgent();
    let done = ext::Gfx_PipelinesCreated();
    if ext::Gfx_SeedCorePending() > 0 && fe.load_core > 0 {
        target = 1.0f32 - ext::Gfx_SeedCorePending() as f32 / fe.load_core as f32;
        fe.load_settled = 0;
    } else if pending > 0 && fe.load_core == 0 {
        target = (done - fe.load_base) as f32 / (done - fe.load_base + pending) as f32;
        fe.load_settled = 0;
    } else {
        target = 1.0f32;
        fe.load_settled += 1;
    }
    if target > fe.progress {
        fe.progress = fmadds(target - fe.progress, 0.2f32, fe.progress);
    }
    if fe.load_settled > 0 && fe.progress > 0.995f32 {
        fe.progress = 1.0f32;
    }
    fe.percent = (fe.progress * 100.0f32) as i32;

    if ext::Gfx_LoadScreenEnabled() == 0
        || fe.load_frames >= FE_LOAD_CEILING_FRAMES
        || (fe.load_frames >= FE_LOAD_MIN_FRAMES && fe.load_settled >= FE_LOAD_SETTLE_FRAMES)
    {
        ext::OSReport(
            c"frontend: loading screen done after %d frames (%d pipelines built since boot, %d while here, %d still pending, core target %d, %s)\n".as_ptr(),
            &[
                Va::I32(fe.load_frames),
                Va::I32(done),
                Va::I32(done - fe.load_base),
                Va::I32(pending),
                Va::I32(fe.load_core),
                Va::P(if fe.load_settled > 0 { c"warm" } else { c"ceiling" }.as_ptr().cast()),
            ],
        );
        return 1;
    }
    0
}

/// VS mode's loading state: the warm-up, then the state's exit picks the match.
pub unsafe fn fe_loading_frame() {
    if fe.leaving != 0 {
        fe.fade += 1;
        if fe.fade >= FE_FADE_FRAMES {
            fe.warmed = 1;
            ext::gm_801A4B60(); // the state's on_exit picks the match
        }
        ext::fl_frame_anim();
        return;
    }
    if fe.fade > 0 {
        fe.fade -= 1;
    }
    if fe_load_step() != 0 {
        fe.leaving = 1;
    }
    ext::fl_frame();
}

/// The toolkit row under the pointer (the kit's rows as drawn), or -1.
pub unsafe fn fe_mouse_row() -> i32 {
    if fe_kit == 0 || fk.on == 0 {
        return -1;
    }
    let mut r = 0i32;
    while r < fk.nrows && r < fe.n_vis {
        let t = addr_of!(fp.tile[fk.tile[r as usize] as usize]);
        if ext::fms_in_quad((*t).q_face) != 0 || ext::fms_in_quad((*t).q_plate) != 0 {
            return r;
        }
        r += 1;
    }
    -1
}

pub unsafe extern "C" fn gm_Scene_Frontend_OnFrame() {
    ext::Controls_Menu(if fe.screen == addr_of!(fe_screen_remap) { fcr_port } else { -1 });
    ext::fms_poll();
    if fm.active != 0 {
        ext::fm_scene_frame();
        return;
    }
    fe.frames += 1;
    if fe.loading != 0 {
        fe_loading_frame();
        return;
    }
    if fe.screen.is_null() {
        ext::gm_ChangeGameModeAfterCurrentScene(fe.continue_to as i32);
        ext::gm_801A4B60();
        return;
    }

    // fades: in on arrival, out before leaving
    if fe.leaving != 0 {
        fe.fade += 1;
        if fe.fade >= FE_FADE_FRAMES {
            // continuing, the next mode sees GM_MENU before it, as from the native menu; backing
            // out, the menus position themselves by the mode this screen stood in for
            fe.reported = if fe.leaving == 1 && fe_menus_on() != 0 { GM_MENU as u8 } else { fe.continue_to };
            ext::gm_ChangeGameModeAfterCurrentScene(if fe.leaving == 1 { fe.continue_to as i32 } else { fe.back_to as i32 });
            ext::gm_801A4B60();
        }
        if (*fe.screen).art != 0 {
            if ext::fa_room_on() != 0 {
                ext::fa_room_leaving(); // under Atlas no frontend model was built: only the host's fade
                return;
            }
            fp.frame += 1;
            ext::fp_evaluate(fp.frame);
            return;
        }
        fe_refresh_rows();
        if fe_kit != 0 {
            ext::fk_frame();
        }
        return;
    }
    if fe.fade > 0 {
        fe.fade -= 1;
    }

    if fe.cursor < fe.n_list {
        fe.hl_y = fmadds((fe.cursor - fe.scroll) as f32 - fe.hl_y, 0.35f32, fe.hl_y);
    }

    // the Atlas settings pages. After the Controls_Menu heartbeat above, never before it.
    if ext::fss_frame() != 0 {
        return;
    }

    if (*fe.screen).art != 0 {
        ext::fl_frame(); // a room screen: its own input and model
        return;
    }
    if fe.screen == addr_of!(fe_screen_remap) && ext::fcr_frame() != 0 {
        fe_refresh_rows();
        if fe_kit != 0 {
            ext::fk_frame();
        }
        return;
    }
    if fte.on != 0 {
        // a text setting is being typed: the keys are text until it ends
        ext::fte_frame();
        fe_refresh_rows();
        if fe_kit != 0 {
            ext::fk_frame();
        }
        return;
    }
    if (fe.screen == addr_of!(fe_screen_online) || ext::fe_is_settings(fe.screen) != 0) && fe_ol_note[0] != 0 && {
        fe_ol_note_frames += 1;
        fe_ol_note_frames > 300
    } {
        fe_ol_note[0] = 0; // a notice stays five seconds
    }

    if fe.frames >= 4 && fe.n_vis > 0 {
        // let the button that brought us here go
        let hov = fe_mouse_row();
        let mut inp = ext::mn_80229624(4) | ext::fms_menu_bits();
        // the mouse: pointing moves the cursor, a click is A on the row, the wheel steps the
        // hovered row's value (a choice, slider or toggle) or scrolls the list
        if hov >= 0 && hov != fe.cursor && (fms.moved != 0 || fms.click != 0) && inp & MENU_CONFIRM == 0 {
            fe.cursor = hov;
            sfxMove();
        }
        if fms.click != 0 && hov >= 0 && hov == fe.cursor {
            inp |= MENU_CONFIRM;
        }
        if fms.wheel != 0 {
            let w = (*fe.screen).items.offset(fe.vis[fe.cursor as usize] as isize);
            if hov == fe.cursor && (*w).kind != FE_ACTION as u8 && (*w).set.is_some() {
                inp |= if fms.wheel > 0 { MENU_RIGHT } else { MENU_LEFT };
            } else {
                let mut to = fe.cursor - fms.wheel;
                to = if to < 0 { 0 } else if to >= fe.n_vis { fe.n_vis - 1 } else { to };
                if to != fe.cursor {
                    fe.cursor = to;
                    sfxMove();
                }
            }
        }
        let it = (*fe.screen).items.offset(fe.vis[fe.cursor as usize] as isize);
        if inp & MENU_UP != 0 {
            fe.cursor = (fe.cursor + fe.n_vis - 1) % fe.n_vis;
            sfxMove();
        } else if inp & MENU_DOWN != 0 {
            fe.cursor = (fe.cursor + 1) % fe.n_vis;
            sfxMove();
        } else if inp & MENU_LEFT != 0 {
            fe_change(it, -1);
        } else if inp & MENU_RIGHT != 0 {
            fe_change(it, 1);
        } else if inp & MENU_CONFIRM != 0 {
            if (*it).enabled.is_some() && (*it).enabled.unwrap()() == 0 {
                sfxBack(); // greyed out: says why in its help line
                if fe_kit != 0 {
                    ext::fk_bump(1);
                }
            } else if (*it).kind == FE_ACTION as u8 {
                if (*it).action == FE_DO_CALL as u8 {
                    sfxForward();
                    if (*it).call.is_some() {
                        (*it).call.unwrap()();
                    }
                    fe_rebuild_visible();
                } else if (*it).action == FE_DO_CONTINUE as u8 {
                    sfxForward();
                    fe.leaving = 1;
                } else {
                    sfxBack();
                    fe.leaving = 2;
                }
            } else if (*it).call.is_some() {
                sfxForward();
                (*it).call.unwrap()(); // e.g. the code field: A moves to the next slot
                fe_rebuild_visible();
            } else {
                fe_change(it, 1);
            }
        } else if inp & MENU_BACK != 0 {
            sfxBack();
            if fe.screen == addr_of!(fe_screen_remap) {
                ext::fcr_back();
            } else if ext::fe_is_controls_help(fe.screen) != 0 {
                fe_switch_screen(addr_of!(fe_screen_settings[FSP_CONTROLS]));
            } else {
                fe.leaving = 2; // ONLINE backs out to the VERSUS menu; MATCH SETUP to where it came from
            }
        }
        if fe.cursor >= fe.n_list {
            // on the button: the plate waits on the row the cursor will come back to
        } else if fe.cursor < fe.scroll {
            fe.scroll = fe.cursor;
        } else if fe.cursor >= fe.scroll + FE_MAX_ROWS as i32 {
            fe.scroll = fe.cursor - FE_MAX_ROWS as i32 + 1;
        }
    }
    fe_refresh_rows();
    if fe_kit != 0 {
        ext::fk_frame();
    }
}

pub unsafe extern "C" fn gm_Scene_Frontend_OnExit(_exit_data: *mut c_void) {
    ext::fss_exit(); // the host closes the settings screen with the scene (Ui_SceneExit)
    if fm.active != 0 {
        ext::fm_scene_exit();
        return;
    }
    ext::fk_exit();
    ext::fl_exit();
    fe_kit = 0;
    for slot in 0..FE_MAX_ROWS {
        if !fe.label[slot].is_null() {
            ext::HSD_SisLib_803A5CC4(fe.label[slot]);
        }
        if !fe.value[slot].is_null() {
            ext::HSD_SisLib_803A5CC4(fe.value[slot]);
        }
        fe.label[slot] = null_mut();
        fe.value[slot] = null_mut();
    }
    if !fe.title.is_null() {
        ext::HSD_SisLib_803A5CC4(fe.title);
    }
    if !fe.subtitle.is_null() {
        ext::HSD_SisLib_803A5CC4(fe.subtitle);
    }
    if !fe.help.is_null() {
        ext::HSD_SisLib_803A5CC4(fe.help);
    }
    if !fe.hints.is_null() {
        ext::HSD_SisLib_803A5CC4(fe.hints);
    }
    if !fe.hint_change.is_null() {
        ext::HSD_SisLib_803A5CC4(fe.hint_change);
    }
    if !fe.hint_back.is_null() {
        ext::HSD_SisLib_803A5CC4(fe.hint_back);
    }
    fe.title = null_mut();
    fe.subtitle = null_mut();
    fe.help = null_mut();
    fe.hints = null_mut();
    fe.hint_change = null_mut();
    fe.hint_back = null_mut();
    for i in 0..FT_COUNT {
        fe_tex[i].ok = 0; // the scene heap goes
    }
    fe.loading = 0;
}

/// `fe_np_zero` (gm.rs ports and verifies it; the leaf `fe_*` helpers `fe_rgba`, `fe_ease`, `fe_row_y` too).
pub use crate::gm::fe_np_zero;
