//! Port of src/melee/gm/gmresult.c: the VS results scene (per-player stat pages, the KO/time/score
//! rows, the winner model and the scene's enter/exit callbacks).
//!
//! State is the file's statics (`lbl_8046DBE8`: the `ResultsData`, `lbl_8046E190`: the pair-mode
//! stat info) and the `MatchEnd` the scene was started with. Callees are `ext` stubs.
//!
//! The japanese strings are byte arrays holding the SJIS the game stores (`81 7C` = a dash, ...):
//! `lbl_804D3F8C`, `lbl_804D3FA0`, `lbl_804D3FA4` and `lbl_804D3FA8` are `u32` words in the C that
//! the big-endian build reads as text; here the bytes are written out in text order.
//!
//! Fused multiply-adds: `fn_80174B4C` (row y) and `fn_8017507C` (the three colour channels).
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gm_r4::fn_801771C0 as fn_801771C0_gm_r7;
#[allow(unused_imports)]
use crate::lb_r6::memzero as memzero_gm_r7;
#[allow(unused_imports)]
use crate::gm_r7::{bytes_of};
#[allow(unused_imports)]
use crate::gm_ptr::fn_80165418 as fn_80165418_gm_r4;
#[allow(unused_imports)]
use crate::gm_ptr::fn_801654A0 as fn_801654A0_gm_r4;
#[allow(unused_imports)]
use mt::MatchEnd as MatchEnd_gm_r4;
#[allow(unused_imports)]
use mt::ResultsData as ResultsData_gm_r4;
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::MatchEnd as MatchEnd_gm_ptr;
#[allow(unused_imports)]
use crate::gm::{OUTCOME_NO_CONTEST as OUTCOME_NO_CONTEST_gm, OUTCOME_RETRY as OUTCOME_RETRY_gm};
use core::ffi::c_void;
use core::mem::offset_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use melee_core::fp::fmadds;
use melee_types::{
    CharacterKind, DynamicModelDesc, GObj_RenderFunc, Gm_PKind_Human, Gm_PKind_NA, HSD_AObj, HSD_Archive, HSD_CObj, HSD_CObjDesc,
    HSD_GObj, HSD_GObjEvent, HSD_GObjProc, HSD_JObj, HSD_Joint, HSD_LObj, HSD_MObj, HSD_Text, HSD_TObj, LightList, MatchEnd, MatchPlayerData,
    ResultsData, ResultsMatchInfo, ResultsPlayerData, Vec3, HSD_GOBJ_CLASS_CAMERA, JOBJ_HIDDEN, MOBJ_MASK, OUTCOME_NO_CONTEST, OUTCOME_RETRY,
    ChKind_Cap, ChKind_Mex0, HSD_AnimJoint, HSD_MatAnimJoint, HSD_ShapeAnimJoint,
};

use super::gmregtyfall::{HSD_GObj_CameraKind, HSD_GObj_JObjKind, HSD_GObj_LightKind};
use super::hsd::{self, get_jobj, jobj_child, jobj_next};
use crate::oracle::Va;
use crate::GXColor;

/// gm_WasMatchCanceled (gm.rs, oracle-checked there): NO_CONTEST / RETRY.
pub use crate::gm::gm_WasMatchCanceled;

pub mod ext {
    use super::*;
    crate::oext_fns! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32) -> ();
        fn GObj_SetupGXLinkMax(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, priority: u32) -> ();
        fn GXSetScissor(left: u32, top: u32, wd: u32, ht: u32) -> ();
        fn HSD_AObjSetCurrentFrame(aobj: *mut HSD_AObj, frame: f32) -> () = crate::tu::adapt::HSD_AObjSetCurrentFrame__64f5bd;
        fn HSD_AObjSetRate(aobj: *mut HSD_AObj, rate: f32) -> () = crate::tu::adapt::HSD_AObjSetRate__1c2c90;
        fn HSD_CObjLoadDesc(desc: *mut HSD_CObjDesc) -> *mut HSD_CObj = crate::tu::adapt::HSD_CObjLoadDesc__66a547;
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void) -> ();
        fn HSD_GObj_80390EB8(i: i32) -> u32 = crate::tu::adapt::HSD_GObj_80390EB8__5ed1a9;
        fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: isize) -> ();
        fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: isize) -> () = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn HSD_GObj_LObjCallback(gobj: *mut HSD_GObj, unused: isize) -> () = crate::tu::adapt::HSD_GObj_LObjCallback__aaa236;
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj) -> ();
        fn HSD_JObjDispAll(jobj: *mut HSD_JObj, vmtx: *mut c_void, flags: u32, rendermode: u32) -> ();
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32) -> ();
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32) -> ();
        fn HSD_MObjAnim(mobj: *mut HSD_MObj) -> ();
        fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A5ACC__27fcb1;
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) -> () = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
        fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 = crate::tu::adapt::HSD_SisLib_803A611C__de78e8;
        fn HSD_SisLib_803A62A0(a0: i32, a1: *const i8, a2: *const i8) -> () = crate::tu::adapt::HSD_SisLib_803A62A0__58f49d;
        fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) -> () = crate::tu::adapt::HSD_SisLib_803A6368__2e8f0d;
        fn HSD_SisLib_803A6754(arg0: i32, arg1: i32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A6754__02fb41;
        fn HSD_SisLib_803A74F0(a0: *mut HSD_Text, arg1: i32, arg2: *mut GXColor) -> ();
        fn HSD_SisLib_803A7548(a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32) -> () = crate::tu::adapt::HSD_SisLib_803A7548__d5953f;
        fn HSD_TObjAnim(tobj: *mut HSD_TObj) -> () = crate::tu::adapt::HSD_TObjAnim__0f2a76;
        fn Mex_InsigniaForExt(ext: i32) -> i32;
        fn Mex_PortCKindToExt(ck: i32) -> i32;
        fn Player_GetTeam(slot: i32) -> i32 = crate::tu::adapt::Player_GetTeam__f1d095;
        fn fn_80160400(a0: CharacterKind) -> u32 = crate::tu::adapt::fn_80160400__597b46;
        fn fn_80165418(a0: *mut MatchEnd) -> i32 = crate::tu::adapt::fn_80165418__e6c527;
        fn fn_801654A0(a0: *mut MatchEnd) -> i32 = crate::tu::adapt::fn_801654A0__ecc03d;
        fn fn_8016F140(a0: i32) -> () = crate::tu::adapt::fn_8016F140__cc064d;
        fn fn_8016F1F0(idx: i32) -> i32 = crate::tu::adapt::fn_8016F1F0__e5a15c;
        fn fn_8016F280(a0: i32) -> i32 = crate::tu::adapt::fn_8016F280__c29e83;
        fn fn_801701AC() -> () = crate::tu::adapt::fn_801701AC__5445dd;
        fn fn_801701B8() -> i32 = crate::tu::adapt::fn_801701B8__58bc4c;
        fn fn_801785B0(a0: *mut HSD_GObj) -> () = super::super::gmresultplayer::fn_801785B0;
        fn fn_80179350(a0: *mut HSD_GObj) -> () = super::super::gmresultplayer::fn_80179350;
        fn fn_80179F6C(idx: i32, value: *mut HSD_GObj) -> () = crate::tu::adapt::fn_80179F6C__0d22a2;
        fn fn_80179F84(a0: *mut HSD_JObj) -> () = crate::tu::adapt::fn_80179F84__bc4b92;
        fn fn_8017A004() -> () = crate::tu::adapt::fn_8017A004__2fa472;
        fn fn_8017A318(a0: i32) -> *mut HSD_GObj = crate::tu::adapt::fn_8017A318__402e70;
        fn fn_8017A67C(c_kind: CharacterKind, a1: i32, a2: i32) -> *mut HSD_GObj = crate::tu::adapt::fn_8017A67C__2be55f;
        fn fn_8017A9B4(a0: i32) -> () = crate::tu::adapt::fn_8017A9B4__17b847;
        fn fn_8017AA78(a0: *const u8) -> () = crate::tu::adapt::fn_8017AA78__72317a;
        fn fn_8017AD28(a0: i32) -> i32 = crate::tu::adapt::fn_8017AD28__3065ff;
        fn fn_8017AD48(a0: i32) -> i32 = crate::tu::adapt::fn_8017AD48__fef659;
        fn fn_8017AD78(a0: i32) -> i32 = crate::tu::adapt::fn_8017AD78__e2265e;
        fn fn_8017ADA8(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017ADA8;
        fn fn_8017AE0C(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017AE0C;
        fn fn_8017AE70(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017AE70;
        fn fn_8017AED8(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017AED8;
        fn fn_8017AF40(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017AF40;
        fn fn_8017AFA8(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017AFA8;
        fn fn_8017B010(a0: i32) -> i32 = crate::tu::adapt::fn_8017B010__57e88d;
        fn fn_8017B07C(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B07C;
        fn fn_8017B0E4(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B0E4;
        fn fn_8017B14C(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B14C;
        fn fn_8017B1B4(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B1B4;
        fn fn_8017B21C(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B21C;
        fn fn_8017B280(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B280;
        fn fn_8017B2E4(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B2E4;
        fn fn_8017B348(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B348;
        fn fn_8017B3AC(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B3AC;
        fn fn_8017B410(a0: i32) -> i32 = crate::tu::adapt::fn_8017B410__473c57;
        fn fn_8017B4D0(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B4D0;
        fn fn_8017B534(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B534;
        fn fn_8017B598(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B598;
        fn fn_8017B5FC(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B5FC;
        fn fn_8017B660(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B660;
        fn fn_8017B6C4(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B6C4;
        fn fn_8017B728(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B728;
        fn fn_8017B78C(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B78C;
        fn fn_8017B7F0(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B7F0;
        fn fn_8017B854(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B854;
        fn fn_8017B8B8(a0: i32) -> i32 = crate::tu::gm_17ad::fn_8017B8B8;
        fn fn_8017B91C(a0: i32) -> i32 = crate::tu::adapt::fn_8017B91C__85de94;
        fn fn_8017B9F4(a0: i32) -> i32 = crate::tu::adapt::fn_8017B9F4__6ec3d8;
        fn fn_8017BACC(a0: i32) -> i32 = crate::tu::adapt::fn_8017BACC__5e81e2;
        fn fn_8017BB30(a0: i32) -> i32 = crate::tu::adapt::fn_8017BB30__5566ea;
        fn fn_8017BB94(a0: i32) -> *const i8 = crate::tu::adapt::fn_8017BB94__2bf7c2;
        fn fn_8017BC50(a0: i32) -> *const i8 = crate::tu::adapt::fn_8017BC50__843ede;
        fn fn_8017BD0C(a0: i32) -> *const i8 = crate::tu::adapt::fn_8017BD0C__f30e74;
        fn fn_8017BDC8(a0: i32) -> *const i8 = crate::tu::adapt::fn_8017BDC8__877377;
        fn gmRst_GenoArtClose() -> () = super::super::gmresultplayer::gmRst_GenoArtClose;
        fn gmRst_GenoArtReset() -> () = super::super::gmresultplayer::gmRst_GenoArtReset;
        fn gm_80160854(a0: u8, a1: u8, a2: u8, a3: u8) -> u32 = crate::tu::adapt::gm_80160854__387c80;
        fn gm_80160968(a0: u32) -> GXColor;
        fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, idx: i32) -> () = crate::tu::adapt::gm_8016895C__f7ee90;
        fn lbAudioAx_80023F28(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_80023F28__582fc2;
        fn lbLang_IsSavedLanguageUS() -> i32 = crate::tu::adapt::lbLang_IsSavedLanguageUS__90e41e;
        fn lbVector_WorldToScreen(cobj: *mut HSD_CObj, pos3d: *const Vec3, screenCoords: *mut Vec3, d: i32) -> *mut Vec3 = crate::tu::adapt::lbVector_WorldToScreen__35d0d5;
        fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) -> () = crate::tu::adapt::lb_8000B1CC__69851d;
        fn lb_8000C07C(a0: *mut HSD_JObj, i: i32, a2: *mut *mut HSD_AnimJoint, a3: *mut *mut HSD_MatAnimJoint, a4: *mut *mut HSD_ShapeAnimJoint) -> () = crate::tu::adapt::lb_8000C07C__443874;
        fn lb_8000C0E8(jobj: *mut HSD_JObj, i: i32, a2: *mut DynamicModelDesc) -> () = crate::tu::adapt::lb_8000C0E8__cceb94;
        fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj = crate::tu::adapt::lb_80011AC4__8bc38c;
        fn lb_80014574(a0: u8, a1: i32, a2: i32, a3: i32) -> () = crate::tu::adapt::lb_80014574__7435d0;
        fn memzero(mem: *mut c_void, size: isize) -> ();
        fn mn_8022F3D8(a0: *mut HSD_JObj, a1: u8, a2: u32) -> () = crate::tu::adapt::mn_8022F3D8__5703af;
        fn un_802FF1B4() -> () = crate::tu::adapt::un_802FF1B4__2022ae;
    }
    crate::ext_va! {
        /// sislib.c: `(text, x, y, fmt, ...)`
        fn HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const i8) va 3 -> i32;
        /// os: `(fmt, ...)`
        fn OSReport(fmt: *const i8) va 0 -> ();
    }
    /// lbarchive.c, variadic in C: `(filename, symbols, name1, out1, ..., NULL)`; the recorder logs
    /// the two fixed arguments only.
    pub unsafe fn lbArchive_80016DBC(filename: *const i8, symbols: *mut c_void, _rest: &[*const c_void]) -> *mut HSD_Archive {
        use crate::oracle::{arg, ext_call, OVal, K};
        // the recorder first; else lbarchive.c (melee_runtime), `rest` after the symbols pointer being the (name, out, ..., NULL) list
        ext_call("lbArchive_80016DBC", &[arg(filename), arg(symbols)], K::P).map(<*mut HSD_Archive as OVal>::from_bits).unwrap_or_else(|| {
            let all: std::vec::Vec<*mut c_void> = core::iter::once(symbols).chain(_rest.iter().map(|p| *p as *mut c_void)).collect();
            melee_runtime::tu::lbarchive::lbArchive_80016DBC(filename as _, &all) as _
        })
    }
    /// lb_00B0.c, variadic in C: `(jobj, out, id, -1)`; fixed arguments logged only.
    pub unsafe fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, _rest: &[i32]) -> i32 {
        use crate::oracle::{arg, ext_call, OVal, K};
        // the recorder first (the state oracle logs this call), else lbspdisplay.c (melee_runtime)
        ext_call("lb_80011E24", &[arg(a0), arg(a1)], K::I(32))
            .map(<i32 as OVal>::from_bits)
            .unwrap_or_else(|| unsafe { melee_runtime::tu::lbspdisplay::lb_80011E24(a0 as _, a1 as _, _rest as _) as _ })
    }
}

/// `StatsEntry`: one row of a stats page.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StatsEntry {
    pub value: i16,
    pub pad_2: [u8; 2],
    pub check: Option<unsafe extern "C" fn(i32) -> i32>,
    pub get: Option<unsafe extern "C" fn(i32) -> *const i8>,
    pub xC: *const c_void,
}

/// `StatsList`: a page of rows.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StatsList {
    pub mode: u8,
    pub count: u8,
    pub pad_2: [u8; 2],
    pub entries: *mut StatsEntry,
}

/// `ResultsStatsInfo`: the pair-mode page descriptor.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ResultsStatsInfo {
    pub x0: u8,
    pub x1: u8,
    pub pad_2: [u8; 2],
    pub x4: i32,
}

/// `lbl_803B7B18`: franchise-emblem lookup and the title text's translate.
#[repr(C)]
pub struct EmblemTable {
    pub lookup: [[u8; 2]; 34],
    pub translate: Vec3,
}

macro_rules! se {
    ($v:expr, $chk:expr, $get:expr, $xc:expr) => {
        StatsEntry { value: $v, pad_2: [0; 2], check: $chk, get: $get, xC: $xc }
    };
}
const NIL: *const c_void = core::ptr::null();

const ZERO_INFO: ResultsStatsInfo = ResultsStatsInfo { x0: 0, x1: 0, pad_2: [0; 2], x4: 0 };

crate::globals! {
    /// `struct ResultsData lbl_8046DBE8`
    rw pub lbl_8046DBE8: ResultsData = unsafe { core::mem::zeroed() };
    /// `static ResultsStatsInfo lbl_8046E190[4]`
    rw pub lbl_8046E190: [ResultsStatsInfo; 4] = [ZERO_INFO; 4];
    /// `HSD_Archive* lbl_804D65B8`
    rw pub lbl_804D65B8: *mut HSD_Archive = null_mut();
    /// `static u32 lbl_804D3F8C[2] = { 0x817C817C, 0x817C0000 }` as the bytes the big-endian build stores
    ro pub lbl_804D3F8C: [u8; 8] = [0x81, 0x7C, 0x81, 0x7C, 0x81, 0x7C, 0x00, 0x00];
    /// `u32 lbl_804D3FA0 = 0x817C0000` (text bytes)
    ro pub lbl_804D3FA0: [u8; 4] = [0x81, 0x7C, 0x00, 0x00];
    /// `u32 lbl_804D3FA4 = 0x817B0000` (text bytes)
    ro pub lbl_804D3FA4: [u8; 4] = [0x81, 0x7B, 0x00, 0x00];
    /// the union `{ u32 words[2]; char text[8]; }` = { 0x817C8146, 0x817C0000 }
    ro pub lbl_804D3FA8: [u8; 8] = [0x81, 0x7C, 0x81, 0x46, 0x81, 0x7C, 0x00, 0x00];
    /// `static const struct { u8 lookup[34][2]; Vec3 translate; } lbl_803B7B18`
    ro pub lbl_803B7B18: EmblemTable = EmblemTable {
        lookup: [
            [0x00, 0x0], [0x01, 0x1], [0x02, 0x2], [0x03, 0x3], [0x04, 0x5], [0x05, 0x6], [0x06, 0xD], [0x07, 0x6], [0x08, 0x6],
            [0x09, 0x7], [0x0A, 0x9], [0x0B, 0x8], [0x0C, 0x6], [0x0D, 0x9], [0x0E, 0x4], [0x0F, 0x9], [0x10, 0xA], [0x11, 0xC],
            [0x12, 0xD], [0x13, 0xD], [0x14, 0x2], [0x15, 0xD], [0x16, 0x6], [0x17, 0x7], [0x18, 0x9], [0x19, 0xD], [0, 0], [0, 0],
            [0, 0], [0, 0], [0, 0], [0, 0], [0, 0], [0, 0],
        ],
        translate: Vec3 { x: 0.0, y: 4.0, z: 0.0 },
    };
    /// `static StatsEntry lbl_803D6488[]`
    rp pub lbl_803D6488: [StatsEntry; 13] = [
        se!(0x08, None, None, NIL),
        se!(0x08, None, None, NIL),
        se!(-1, Some(ext::fn_8017AE70), Some(ext::fn_8017BB94), c"P".as_ptr().cast()),
        se!(-1, Some(ext::fn_8017AED8), Some(ext::fn_8017BC50), NIL),
        se!(-1, Some(ext::fn_8017AF40), Some(ext::fn_8017BD0C), NIL),
        se!(-1, Some(ext::fn_8017AFA8), Some(ext::fn_8017BDC8), NIL),
        se!(0x09, None, None, NIL),
        se!(-1, Some(ext::fn_8017B07C), Some(ext::fn_8017BB94), c"%".as_ptr().cast()),
        se!(-1, Some(ext::fn_8017B0E4), Some(ext::fn_8017BC50), NIL),
        se!(-1, Some(ext::fn_8017B14C), Some(ext::fn_8017BD0C), NIL),
        se!(-1, Some(ext::fn_8017B1B4), Some(ext::fn_8017BDC8), NIL),
        se!(0x0C, Some(ext::fn_8017B21C), None, NIL),
        se!(0x0C, None, None, NIL),
    ];
    /// `static StatsEntry lbl_803D6558[]`
    rp pub lbl_803D6558: [StatsEntry; 48] = [
        se!(0x0A, None, None, NIL),
        se!(0x0A, None, None, NIL),
        se!(0x0B, Some(ext::fn_8017AE0C), None, NIL),
        se!(0x0E, None, None, NIL),
        se!(0x0F, Some(ext::fn_8017B010), None, NIL),
        se!(0x0C, None, None, NIL),
        se!(0x0D, Some(ext::fn_8017B21C), None, NIL),
        se!(0x10, None, None, NIL),
        se!(0x11, Some(ext::fn_8017B280), None, NIL),
        se!(0x12, None, None, NIL),
        se!(0x13, Some(ext::fn_8017B2E4), None, NIL),
        se!(0x14, None, None, NIL),
        se!(0x15, Some(ext::fn_8017B348), None, NIL),
        se!(0x16, None, None, NIL),
        se!(0x17, Some(ext::fn_8017B3AC), None, NIL),
        se!(0x18, None, None, NIL),
        se!(0x19, Some(ext::fn_8017B410), None, NIL),
        se!(0x1A, None, None, NIL),
        se!(0x1B, Some(ext::fn_8017B4D0), None, NIL),
        se!(0x1C, None, None, NIL),
        se!(0x1D, Some(ext::fn_8017B534), None, NIL),
        se!(0x1E, None, None, NIL),
        se!(0x1F, Some(ext::fn_8017B598), None, NIL),
        se!(0x20, None, None, NIL),
        se!(0x21, Some(ext::fn_8017B5FC), None, NIL),
        se!(0x22, None, None, NIL),
        se!(0x23, Some(ext::fn_8017B660), None, NIL),
        se!(0x24, None, None, NIL),
        se!(0x25, Some(ext::fn_8017B6C4), None, NIL),
        se!(0x26, None, None, NIL),
        se!(0x27, Some(ext::fn_8017B728), None, NIL),
        se!(0x28, None, None, NIL),
        se!(0x29, Some(ext::fn_8017B78C), None, NIL),
        se!(0x2A, None, None, NIL),
        se!(0x2B, Some(ext::fn_8017B7F0), None, NIL),
        se!(0x2C, None, None, NIL),
        se!(0x2D, Some(ext::fn_8017B854), None, NIL),
        se!(0x2E, None, None, NIL),
        se!(0x2F, Some(ext::fn_8017B8B8), None, NIL),
        se!(0x30, None, None, NIL),
        se!(0x31, Some(ext::fn_8017B91C), None, NIL),
        se!(0x32, None, None, NIL),
        se!(0x33, Some(ext::fn_8017B9F4), None, NIL),
        se!(0x34, None, None, NIL),
        se!(0x35, Some(ext::fn_8017BACC), None, NIL),
        se!(0x36, None, None, NIL),
        se!(0x37, Some(ext::fn_8017BB30), None, NIL),
        se!(0x36, None, None, NIL),
    ];
    /// `static StatsEntry lbl_803D6858[]`
    rp pub lbl_803D6858: [StatsEntry; 2] = [se!(0x08, None, None, NIL), se!(0x08, None, None, NIL)];
    /// `StatsList lbl_803D6878[]`: the pages (entries point at the tables above)
    rp pub lbl_803D6878: [StatsList; 4] = unsafe {
        [
            StatsList { mode: 0, count: 0x0D, pad_2: [0; 2], entries: addr_of!(lbl_803D6488) as *mut StatsEntry },
            StatsList { mode: 1, count: 0x30, pad_2: [0; 2], entries: addr_of!(lbl_803D6558) as *mut StatsEntry },
            StatsList { mode: 2, count: 0x02, pad_2: [0; 2], entries: addr_of!(lbl_803D6858) as *mut StatsEntry },
            StatsList { mode: 3, count: 0x00, pad_2: [0; 2], entries: null_mut() },
        ]
    };
}

crate::globals_ext! {
    rw HSD_GObj_CameraKind = super::gmregtyfall::HSD_GObj_CameraKind: u8;
    rw HSD_GObj_JObjKind = super::gmregtyfall::HSD_GObj_JObjKind: u8;
    rw HSD_GObj_LightKind = super::gmregtyfall::HSD_GObj_LightKind: i8;
}

#[inline]
unsafe fn data() -> *mut ResultsData {
    addr_of_mut!(lbl_8046DBE8)
}

/// fn_80174274.
pub unsafe fn fn_80174274() -> *mut MatchEnd {
    melee_core::ct!("fn_80174274");
    (*data()).x94
}

/// fn_80174284: how many of the player's 256 flags are set (reporting each to the game for humans).
pub unsafe fn fn_80174284(slot: u8) -> i32 {
    melee_core::ct!("fn_80174284");
    let tmp = (*(*data()).x94).x44C.as_mut_ptr();
    let mut count = 0;
    let do_call = (*(*data()).x94).player_standings[slot as usize].pkind == Gm_PKind_Human as u8;

    for i in 0..0x100usize {
        if (*tmp.add(slot as usize)).x0[i] != 0 {
            if do_call {
                ext::fn_8016F140(i as i32);
            }
            count += 1;
        }
    }
    count
}

/// fn_80174284_noinline.
pub unsafe fn fn_80174284_noinline(slot: u8) -> i32 {
    melee_core::ct!("fn_80174284_noinline");
    fn_80174284(slot)
}

/// fn_80174338: forward sound.
pub unsafe fn fn_80174338() {
    melee_core::ct!("fn_80174338");
    hsd::sfxForward();
}
/// fn_8017435C: back sound.
pub unsafe fn fn_8017435C() {
    melee_core::ct!("fn_8017435C");
    hsd::sfxBack();
}
/// fn_80174380: move sound.
pub unsafe fn fn_80174380() {
    melee_core::ct!("fn_80174380");
    hsd::sfxMove();
}

/// fn_801743C4: whether the stats row is shown for `slot`.
pub unsafe fn fn_801743C4(slot: i32, entry: *mut StatsEntry) -> i32 {
    melee_core::ct!("fn_801743C4");
    if (*entry).value >= 0 {
        return 1;
    }
    if let Some(check) = (*entry).check {
        if check(slot) == -1 {
            return 0;
        }
    }
    match (*entry).get {
        Some(get) => {
            if get(slot).is_null() {
                return 0;
            }
        }
        None => return 0,
    }
    1
}

/// gmResultFindNth (static inline; gm_ptr.rs has the oracle-checked slice version).
pub unsafe fn gmResultFindNth(values: *const u8, mut n: i32) -> i32 {
    melee_core::ct!("gmResultFindNth");
    let mut i = 0;
    while i < 256 {
        if *values.add(i as usize) != 0 {
            n = n.wrapping_sub(1);
            if n == 0 {
                break;
            }
        }
        i += 1;
    }
    i
}

/// gmResultFormatLabel (static inline).
pub unsafe fn gmResultFormatLabel(slot: i32, entry: *mut StatsEntry, zero: f32, neg30: f32, text: *mut HSD_Text) -> i32 {
    melee_core::ct!("gmResultFormatLabel");
    let s = ((*entry).get.unwrap())(slot);
    ext::HSD_SisLib_803A6B98(text, zero, neg30, c"%s".as_ptr(), &[Va::P(s.cast())])
}

/// fn_80174468: fill one row's three texts.
pub unsafe fn fn_80174468(slot: i32, text1: *mut HSD_Text, text2: *mut HSD_Text, text3: *mut HSD_Text, list: *mut StatsList, entry_idx: i32) {
    melee_core::ct!("fn_80174468");
    let const_zero = 0.0f32;
    let const_neg30 = -30.0f32;
    let mut label_id: i32 = -1;
    let mut value_id: i32 = -1;
    let entry = (*list).entries.offset(entry_idx as isize);

    if (*list).mode != 2 {
        if (*entry).value >= 0 {
            ext::HSD_SisLib_803A6368(text1, (*entry).value as i32);
        }
    } else {
        // Mode 2: special handling for pairs
        if (entry_idx & 1) == 1 && entry_idx < (*list).count as i32 {
            let tmp = (*(*data()).x94).x44C.as_mut_ptr();
            let mut loop_n = entry_idx / 2;
            let mut loop_i = 0usize;
            loop_n += 1;
            let loop_ptr = (*tmp.add(slot as u8 as usize)).x0.as_ptr();
            while loop_i < 256 {
                if *loop_ptr.add(loop_i) != 0 {
                    loop_n -= 1;
                    if loop_n == 0 {
                        break;
                    }
                }
                loop_i += 1;
            }
            let mut idx = ext::fn_8016F1F0(loop_i as i32);
            idx = (ext::fn_8016F280(idx) as u16 as i32) + 60;
            ext::HSD_SisLib_803A6368(text1, idx);
        }
    }

    if (*list).mode == 2 || (*entry).check.is_some() {
        if (*list).mode != 2 {
            let result = ((*entry).check.unwrap())(slot);
            if result < 0 {
                value_id = ext::HSD_SisLib_803A6B98(text3, const_zero, const_neg30, c"%s".as_ptr(), &[Va::P(addr_of!(lbl_804D3F8C).cast())]);
            } else {
                value_id = ext::HSD_SisLib_803A6B98(text3, const_zero, const_neg30, c"%d".as_ptr(), &[Va::I32(result)]);
            }
        } else if (entry_idx & 1) == 0 {
            let pair_idx = (entry_idx / 2) - 1;
            if 0 <= pair_idx {
                let tmp = (*(*data()).x94).x44C.as_mut_ptr();
                let i = gmResultFindNth((*tmp.add(slot as u8 as usize)).x0.as_ptr(), pair_idx + 1);
                let stat_value = (*tmp.add(slot as u8 as usize)).x104[i as usize];
                if stat_value < 0 {
                    value_id = ext::HSD_SisLib_803A6B98(
                        text3,
                        const_zero,
                        const_neg30,
                        c"%s%d".as_ptr(),
                        &[Va::P(addr_of!(lbl_804D3FA0).cast()), Va::I32(stat_value.wrapping_neg())],
                    );
                } else {
                    value_id = ext::HSD_SisLib_803A6B98(text3, const_zero, const_neg30, c"%d".as_ptr(), &[Va::I32(stat_value)]);
                }
            } else {
                value_id = ext::HSD_SisLib_803A6B98(text3, const_zero, const_neg30, c"%s".as_ptr(), &[Va::P(addr_of!(lbl_804D3FA0).cast())]);
            }
        }
        if (*list).mode != 2 && (*entry).get.is_some() {
            label_id = gmResultFormatLabel(slot, entry, const_zero, const_neg30, text2);
        }
    } else if (*list).mode != 2 {
        if let Some(get) = (*entry).get {
            let s = get(slot);
            label_id = ext::HSD_SisLib_803A6B98(text2, const_zero, const_neg30, c"%s".as_ptr(), &[Va::P(s.cast())]);
        }
    }

    if label_id >= 0 {
        ext::HSD_SisLib_803A7548(text2, label_id, 0.05f32, 0.0546875f32);
    }
    if value_id >= 0 {
        ext::HSD_SisLib_803A7548(text3, value_id, 0.059f32, 0.0546875f32);
    }
}

/// fn_801748EC: the stats list for `mode` (searching `list_` for it), or the pair info for `idx`.
pub unsafe fn fn_801748EC(list_: *mut c_void, mode: i32, idx: i32) -> *mut c_void {
    melee_core::ct!("fn_801748EC");
    let mut list = list_ as *mut ResultsStatsInfo;
    if mode != 2 {
        while (*list).x0 as i32 != mode {
            list = list.add(1);
        }
        return list.cast();
    }
    addr_of_mut!(lbl_8046E190[idx as usize]).cast()
}

/// fn_80174920: free the player's stats texts. (The C declares `u8` and returns nothing.)
pub unsafe fn fn_80174920(data: *mut ResultsPlayerData) {
    melee_core::ct!("fn_80174920");
    for i in 0..10usize {
        for row in 0..3usize {
            if !(*data).stats_text[row][i].is_null() {
                ext::HSD_SisLib_803A5CC4((*data).stats_text[row][i]);
                (*data).stats_text[row][i] = null_mut();
            }
        }
    }
}

/// fn_801749B8: clip the stats rows to the panel (render callback of the stats texts).
pub unsafe extern "C" fn fn_801749B8(_unused: *mut HSD_GObj) {
    melee_core::ct!("fn_801749B8");
    let mut sp24 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp18 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut spC = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if !(*data()).cobj.is_null() {
        ext::lbVector_WorldToScreen((*data()).cobj, addr_of!((*data()).player_data[0].stats_position).cast(), &mut sp24, 0);
        let new_var2 = data();
        ext::lb_8000B1CC((*new_var2).player_data[0].jobjs[0xD], null_mut(), &mut sp18);
        ext::lbVector_WorldToScreen((*new_var2).cobj, &sp18, &mut spC, 0);
        let new_var4 = (melee_core::ppc::fctiwz_f32(sp24.y)) as u16;
        let temp_r4: u32 = (new_var4 as u32) & 0xFFFC;
        let new_var3 = melee_core::ppc::fctiwz_f32(spC.y);
        let new_var = (new_var3 & 0xFFDC) as u16;
        ext::GXSetScissor(0, temp_r4, 0x27F, (new_var as u32).wrapping_sub(temp_r4));
    }
}

/// fn_80174A60: how many rows of `list` show for `slot`.
pub unsafe fn fn_80174A60(list: *mut StatsList, slot: i32) -> i32 {
    melee_core::ct!("fn_80174A60");
    let mut count = 0;
    if (*list).mode != 2 {
        let mut i = 0;
        while i < (*list).count as i32 {
            if fn_801743C4(slot, (*list).entries.offset(i as isize)) != 0 {
                count += 1;
            }
            i += 1;
        }
    } else {
        count = (*list).count as i32;
    }
    count
}

/// `fn_80174B4C_blk14829` (static inline): pick the player's data and clear its stats texts.
pub unsafe fn fn_80174B4C_blk14829(data: *mut ResultsData, slot: i32, pdata: *mut *mut ResultsPlayerData, list: *mut *mut StatsList) {
    melee_core::ct!("fn_80174B4C_blk14829");
    *pdata = addr_of_mut!((*data).player_data[slot as usize]);
    *list = addr_of_mut!(lbl_803D6878).cast();

    for i in 0..10usize {
        for row in 0..3usize {
            if !(**pdata).stats_text[row][i].is_null() {
                ext::HSD_SisLib_803A5CC4((**pdata).stats_text[row][i]);
                (**pdata).stats_text[row][i] = null_mut();
            }
        }
    }
}

/// fn_80174B4C: (re)build the stats rows of `slot`'s page at its scroll offset. The row y step
/// `pos.y += 1.75f * (offset - (start - 1))` is a fused multiply-add.
pub unsafe fn fn_80174B4C(data: *mut ResultsData, slot: i32) {
    melee_core::ct!("fn_80174B4C");
    let mut pdata: *mut ResultsPlayerData = null_mut();
    let mut list_base: *mut StatsList = null_mut();

    fn_80174B4C_blk14829(data, slot, &mut pdata, &mut list_base);
    let mut list = list_base;

    // Look up stats list
    if (*pdata).page as i32 != 2 {
        while (*list).mode as i32 != (*pdata).page as i32 {
            list = list.add(1);
        }
    } else {
        list = addr_of_mut!(lbl_8046E190[slot as usize]).cast();
    }

    let mut count = 0;
    let mut pos: Vec3 = (*pdata).stats_position;

    let offset = (*pdata).scroll_offset;
    let start_entry = (melee_core::ppc::fctiwz_f32(10.0f32 * offset)).wrapping_div(10);
    pos.y = fmadds(1.75f32, offset - ((start_entry - 1) as f32), pos.y);

    // Set flags
    (*pdata).set_x0_3(1);
    (*pdata).set_x0_4(1);

    if (*pdata).page == 0 {
        (*pdata).set_x0_3(0);
    }
    if (*pdata).page as i32 >= (*data).num_pages as i32 - 1 {
        (*pdata).set_x0_4(0);
    }

    // Count visible entries
    let visible_count = fn_80174A60(list, 0);

    // Set more flags based on scroll position
    if (*pdata).scroll_offset < (visible_count - 10) as f32 {
        (*pdata).set_x0_2(1);
    } else {
        (*pdata).set_x0_2(0);
    }

    let mut entry_offset = start_entry * 0x10;

    if (*pdata).scroll_offset > 0.0f32 {
        (*pdata).set_x0_1(1);
    } else {
        (*pdata).set_x0_1(0);
    }

    // Create text objects for visible entries
    let render_callback: Option<unsafe extern "C" fn(*mut c_void)> =
        Some(core::mem::transmute::<unsafe extern "C" fn(*mut HSD_GObj), unsafe extern "C" fn(*mut c_void)>(fn_801749B8));
    let mut entry_idx = start_entry;
    while count < 10 {
        if (*list).count as i32 <= entry_idx + 1 {
            break;
        }
        if (*list).mode != 2 && fn_801743C4(slot, (*list).entries.offset(entry_idx as isize)) == 0 {
            entry_idx += 1;
            entry_offset += 0x10;
            continue;
        }

        let c = count as usize;
        (*pdata).stats_text[0][c] = ext::HSD_SisLib_803A5ACC(0, 0, pos.x, -pos.y, pos.z, 11.0f32, 10.0f32);
        let mut text = (*pdata).stats_text[0][c];
        (*text).default_fitting = 1;
        text = (*pdata).stats_text[0][c];
        (*text).x34.x = 0.05f32;
        (*text).x34.y = 0.0546875f32;
        text = (*pdata).stats_text[0][c];
        (*text).render_callback = render_callback;

        (*pdata).stats_text[1][c] = ext::HSD_SisLib_803A6754(0, 0);
        text = (*pdata).stats_text[1][c];
        (*text).pos_x = pos.x;
        (*text).pos_y = -pos.y;
        (*text).pos_z = pos.z;
        text = (*pdata).stats_text[1][c];
        (*text).render_callback = render_callback;

        (*pdata).stats_text[2][c] = ext::HSD_SisLib_803A6754(0, 0);
        text = (*pdata).stats_text[2][c];
        (*text).pos_x = 11.0f32 + pos.x;
        (*text).pos_y = -pos.y;
        (*text).pos_z = pos.z;
        text = (*pdata).stats_text[2][c];
        (*text).render_callback = render_callback;
        text = core::ptr::read_volatile(addr_of!((*pdata).stats_text[2][c]));
        (*text).default_alignment = 2;

        pos.y -= 1.75f32;
        fn_80174468(slot, (*pdata).stats_text[0][c], (*pdata).stats_text[1][c], (*pdata).stats_text[2][c], list, entry_idx);
        entry_idx += 1;
        entry_offset += 0x10;
        count += 1;
    }
    let _ = entry_offset;
}

/// fn_80174FD0: set the texture frame of a joint's material.
pub unsafe fn fn_80174FD0(jobj: *mut HSD_JObj, arg1: i32) {
    melee_core::ct!("fn_80174FD0");
    let tobj = (*(*(*jobj).u.dobj).mobj).tobj;
    ext::HSD_AObjSetCurrentFrame((*tobj).aobj, arg1 as f32);
    ext::HSD_AObjSetRate((*tobj).aobj, 0.0f32);
    ext::HSD_TObjAnim(tobj);
}

/// fn_80175038: render callback of the panel model.
pub unsafe extern "C" fn fn_80175038(gobj: *mut HSD_GObj, flag: isize) {
    melee_core::ct!("fn_80175038");
    ext::HSD_JObjDispAll(get_jobj(gobj), null_mut(), ext::HSD_GObj_80390EB8(flag as i32), 0);
}

/// fn_8017507C: the player's row colour (team colour lightened, or white).
pub unsafe fn fn_8017507C(slot: i32) -> GXColor {
    melee_core::ct!("fn_8017507C");
    let mut color = GXColor { r: 0, g: 0, b: 0, a: 0 };
    let me = (*data()).x94;
    if (*me).is_teams == 1 {
        if (*me).player_standings[slot as usize].pkind == Gm_PKind_NA as u8 {
            color = ext::gm_80160968(4);
        } else {
            match (*me).player_standings[slot as usize].team {
                0 => color = ext::gm_80160968(0),
                1 => color = ext::gm_80160968(1),
                2 => color = ext::gm_80160968(3),
                3 => color = ext::gm_80160968(2),
                _ => color = ext::gm_80160968(4),
            }
        }

        let mut current_color = fmadds(0.9f32, color.r as f32, 100.0f32);
        if current_color > 255.0f32 {
            current_color = 255.0f32;
        }
        color.r = melee_core::ppc::fctiwz_f32(current_color) as u8;

        current_color = fmadds(0.9f32, color.g as f32, 100.0f32);
        if current_color > 255.0f32 {
            current_color = 255.0f32;
        }
        color.g = ((melee_core::ppc::fctiwz_f32(current_color)) as i8) as u8;

        current_color = fmadds(0.9f32, color.b as f32, 100.0f32);
        if current_color > 255.0f32 {
            current_color = 255.0f32;
        }
        color.b = ((melee_core::ppc::fctiwz_f32(current_color)) as i8) as u8;
    } else {
        color.r = 0xFF;
        color.g = 0xFF;
        color.b = 0xFF;
        color.a = 0xFF;
    }
    color
}

/// Show a KO-count row: `fmt`/`arg` text at (0, y) with the colour. Returns nothing.
unsafe fn ko_row(text: *mut HSD_Text, y: f32, fmt: &core::ffi::CStr, va: &[Va], color: GXColor) {
    let line_id = ext::HSD_SisLib_803A6B98(text, 0.0f32, y, fmt.as_ptr(), va);
    ext::HSD_SisLib_803A7548(text, line_id, 0.11f32, 0.08f32);
    let mut c = color;
    ext::HSD_SisLib_803A74F0(text, line_id, &mut c);
}

/// fn_80175240: the KO-count column of `slot`.
pub unsafe fn fn_80175240(slot: i32) {
    melee_core::ct!("fn_80175240");
    let d = data();
    let me = (*d).x94;
    if (*me).match_kind != 3 {
        // maybe x4C is an array of Vec2's
        let first_row_offset = 1.14f32 * ((*d).x4C[4].y - (*d).x4C[0].y);
        let second_row_offset = 1.12f32 * ((*d).x4C[5].y - (*d).x4C[4].y);
        let mut color = fn_8017507C(slot);
        let s = slot as usize;
        (*d).player_data[s].ko_count = ext::HSD_SisLib_803A6754(0, 0);
        let pos_z = (*d).x4C[s].z;
        let ko_count = (*d).player_data[s].ko_count;
        let pos_y = -(*d).x4C[s].y - 30.0f32;
        let pos_x = (*d).x4C[s].x;
        (*ko_count).pos_x = pos_x;
        (*ko_count).pos_y = pos_y;
        (*ko_count).pos_z = pos_z;
        (*(*d).player_data[s].ko_count).default_alignment = 1;
        (*(*d).player_data[s].ko_count).default_kerning = 1;
        if (*me).player_standings[s].pkind != Gm_PKind_NA as u8 {
            let mut first_value = ext::fn_8017AE0C(slot);
            if first_value > 0x3E7 {
                first_value = 0x3E7;
            }
            let t = (*d).player_data[s].ko_count;
            ko_row(t, 0.0f32, c"%d", &[Va::I32(first_value)], color);
            let mut value = ext::fn_8017B010(slot);
            if value > 0x3E7 {
                value = 0x3E7;
            }
            ko_row(t, -first_row_offset, c"%d", &[Va::I32(value)], color);
            value = ext::fn_8017B21C(slot);
            if value > 0x3E7 {
                value = 0x3E7;
            }
            ko_row(t, -first_row_offset - second_row_offset, c"%d", &[Va::I32(value)], color);
            return;
        }
        color.r = 0xA0;
        color.g = 0xA0;
        color.b = 0xA0;
        let t = (*d).player_data[s].ko_count;
        let dash = Va::P(addr_of!(lbl_804D3FA0).cast());
        ko_row(t, 0.0f32, c"%s", &[dash], color);
        ko_row(t, -first_row_offset, c"%s", &[dash], color);
        ko_row(t, -first_row_offset - second_row_offset, c"%s", &[dash], color);
    }
}

/// fn_8017556C: the score/points text for `slot` (KO-count style: signed with +/-).
pub unsafe fn fn_8017556C(slot: i32) {
    melee_core::ct!("fn_8017556C");
    let d = data();
    let me = (*d).x94;
    let s = slot as usize;
    let mut sp10 = fn_8017507C(slot);
    let var_r28;
    if (*me).player_standings[s].pkind != Gm_PKind_NA as u8 {
        let mut var_r6 = ext::fn_8017AD28((*me).player_standings[s].score);
        if var_r6 < 0 {
            if var_r6 < 0 {
                var_r6 = var_r6.wrapping_neg();
            }
            var_r28 = ext::HSD_SisLib_803A6B98(
                (*d).player_data[s].ko_time,
                0.0f32,
                -30.0f32,
                c"%s%d".as_ptr(),
                &[Va::P(addr_of!(lbl_804D3FA0).cast()), Va::I32(var_r6)],
            );
        } else if var_r6 > 0 {
            var_r28 = ext::HSD_SisLib_803A6B98(
                (*d).player_data[s].ko_time,
                0.0f32,
                -30.0f32,
                c"%s%d".as_ptr(),
                &[Va::P(addr_of!(lbl_804D3FA4).cast()), Va::I32(var_r6)],
            );
        } else {
            var_r28 = ext::HSD_SisLib_803A6B98((*d).player_data[s].ko_time, 0.0f32, -30.0f32, c"%d".as_ptr(), &[Va::I32(var_r6)]);
        }
    } else {
        sp10.r = 0xA0;
        sp10.g = 0xA0;
        sp10.b = 0xA0;
        var_r28 = ext::HSD_SisLib_803A6B98((*d).player_data[s].ko_time, 0.0f32, -30.0f32, c"%s".as_ptr(), &[Va::P(addr_of!(lbl_804D3FA0).cast())]);
    }
    ext::HSD_SisLib_803A7548((*d).player_data[s].ko_time, var_r28, 0.11f32, 0.08f32);
    let mut sp_c = sp10;
    ext::HSD_SisLib_803A74F0((*d).player_data[s].ko_time, var_r28, &mut sp_c);
}

/// `matchWasSkipped` (static inline).
pub unsafe fn matchWasSkipped(me: *mut MatchEnd) -> i32 {
    melee_core::ct!("matchWasSkipped");
    ((*me).outcome == OUTCOME_NO_CONTEST as u8 || (*me).outcome == OUTCOME_RETRY as u8) as i32
}

/// fn_801756E0: the points text of a time-match row.
pub unsafe fn fn_801756E0(slot: i32) {
    melee_core::ct!("fn_801756E0");
    let d = data();
    let me = (*d).x94;
    let s = slot as usize;
    let mut color = fn_8017507C(slot);
    let line_num;
    if (*me).player_standings[s].pkind != Gm_PKind_NA as u8 && matchWasSkipped(me) == 0 {
        let mut score = ext::fn_8017AD48((*me).player_standings[s].score);
        if score < 0 {
            if score < 0 {
                score = score.wrapping_neg();
            }
            line_num = ext::HSD_SisLib_803A6B98(
                (*d).player_data[s].ko_time,
                0.0f32,
                -30.0f32,
                c"%s%d".as_ptr(),
                &[Va::P(addr_of!(lbl_804D3FA0).cast()), Va::I32(score)],
            );
        } else if 0 < score {
            line_num = ext::HSD_SisLib_803A6B98(
                (*d).player_data[s].ko_time,
                0.0f32,
                -30.0f32,
                c"%s%d".as_ptr(),
                &[Va::P(addr_of!(lbl_804D3FA4).cast()), Va::I32(score)],
            );
        } else {
            line_num = ext::HSD_SisLib_803A6B98((*d).player_data[s].ko_time, 0.0f32, -30.0f32, c"%d".as_ptr(), &[Va::I32(score)]);
        }
    } else {
        color.r = 0xA0;
        color.g = 0xA0;
        color.b = 0xA0;
        line_num = ext::HSD_SisLib_803A6B98((*d).player_data[s].ko_time, 0.0f32, -30.0f32, c"%s".as_ptr(), &[Va::P(addr_of!(lbl_804D3FA0).cast())]);
    }
    ext::HSD_SisLib_803A7548((*d).player_data[s].ko_time, line_num, 0.08f32, 0.08f32);
    let mut color_copy = color;
    ext::HSD_SisLib_803A74F0((*d).player_data[s].ko_time, line_num, &mut color_copy);
}

/// fn_80175880: the time text of a stock-match row (a dash for the winner).
pub unsafe fn fn_80175880(slot: i32) {
    melee_core::ct!("fn_80175880");
    let d = data();
    let me = (*d).x94;
    let s = slot as usize;
    let mut color = fn_8017507C(slot);
    let line_num;
    if (*me).player_standings[s].pkind != Gm_PKind_NA as u8 {
        let x6 = (*d).x6 as usize;
        if (*d).x6 as i32 == slot
            || ((*me).is_teams == 1
                && (*me).team_standings[(*me).player_standings[s].team as usize].is_big_loser == 0
                && ((*me).player_standings[s].stocks > 0
                    || ((*me).player_standings[s].x28 == (*me).frame_count && (*me).player_standings[x6].stocks == 0)))
            || (gm_WasMatchCanceled((*me).outcome)
                && ((*me).player_standings[s].stocks > 0 || (*me).player_standings[s].score == (*me).player_standings[x6].score))
        {
            line_num = ext::HSD_SisLib_803A6B98((*d).player_data[s].ko_time, 0.0f32, -30.0f32, addr_of!(lbl_804D3FA8).cast(), &[]);
        } else {
            let seconds = (*me).player_standings[s].x28 / 60;
            let mut display_seconds = seconds;
            if seconds > 5999 {
                display_seconds = 5999;
            }
            let minutes = display_seconds / 60;
            line_num = ext::HSD_SisLib_803A6B98(
                (*d).player_data[s].ko_time,
                0.0f32,
                -30.0f32,
                c"%d\x81\x46%02d".as_ptr(),
                &[Va::I32(minutes as i32), Va::I32((display_seconds % 60) as i32)],
            );
        }
    } else {
        color.r = 0xA0;
        color.g = 0xA0;
        color.b = 0xA0;
        line_num = ext::HSD_SisLib_803A6B98((*d).player_data[s].ko_time, 0.0f32, -30.0f32, c"%s".as_ptr(), &[Va::P(addr_of!(lbl_804D3FA0).cast())]);
    }

    ext::HSD_SisLib_803A7548((*d).player_data[s].ko_time, line_num, 0.11f32, 0.08f32);
    let mut color_copy = color;
    ext::HSD_SisLib_803A74F0((*d).player_data[s].ko_time, line_num, &mut color_copy);
}

/// `fn_80175A94_get_match_end` (static inline).
pub unsafe fn fn_80175A94_get_match_end() -> *mut MatchEnd {
    melee_core::ct!("fn_80175A94_get_match_end");
    (*data()).x94
}

/// fn_80175A94: create `slot`'s time/points text at `position` and fill it per match kind.
pub unsafe fn fn_80175A94(slot: i32, position: *mut Vec3) {
    melee_core::ct!("fn_80175A94");
    let player = slot as u32 as usize;
    let mut sp24 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut me = (*data()).x94;
    let x_pos;
    if (*me).match_kind == 3 {
        ext::lb_8000B1CC((*data()).player_data[player].jobjs[12], null_mut(), &mut sp24);
        x_pos = 5.5f32 + sp24.x;
    } else {
        x_pos = (*data()).x4C[player].x;
    }

    let new_var = data();
    (*new_var).player_data[player].ko_time = ext::HSD_SisLib_803A6754(0, 0);
    let new_var3 = (*position).z;
    let new_var2 = (*new_var).player_data[player].ko_time;
    let new_var4 = -(*position).y;
    (*new_var2).pos_x = x_pos;
    (*new_var2).pos_y = new_var4;
    (*new_var2).pos_z = new_var3;
    (*(*new_var).player_data[player].ko_time).default_alignment = 1;
    (*(*new_var).player_data[player].ko_time).default_kerning = 1;

    if (*me).match_kind == 2 {
        me = fn_80175A94_get_match_end();
        let mut sp14 = fn_8017507C(player as i32);
        let line;
        if (*me).player_standings[player].pkind != 3 {
            let v = ext::fn_8017AD78(ext::fn_8017ADA8(player as i32));
            line = ext::HSD_SisLib_803A6B98((*new_var).player_data[player].ko_time, 0.0f32, -30.0f32, c"%d".as_ptr(), &[Va::I32(v)]);
        } else {
            sp14.r = 0xA0;
            sp14.g = 0xA0;
            sp14.b = 0xA0;
            line = ext::HSD_SisLib_803A6B98((*new_var).player_data[player].ko_time, 0.0f32, -30.0f32, c"%s".as_ptr(), &[Va::P(addr_of!(lbl_804D3FA0).cast())]);
        }
        ext::HSD_SisLib_803A7548((*new_var).player_data[player].ko_time, line, 0.09f32, 0.08f32);
        let mut sp18 = sp14;
        ext::HSD_SisLib_803A74F0((*new_var).player_data[player].ko_time, line, &mut sp18);
    } else if (*me).match_kind == 1 {
        fn_80175880(player as i32);
    } else if (*me).match_kind == 3 {
        fn_801756E0(player as i32);
    } else {
        fn_8017556C(player as i32);
    }
}

/// fn_80175C5C: lay out the four players' text columns.
pub unsafe fn fn_80175C5C() {
    melee_core::ct!("fn_80175C5C");
    let mut position = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let d = data();

    ext::lb_8000B1CC((*d).x24, null_mut(), &mut position);
    for i in 0..6usize {
        ext::lb_8000B1CC((*d).x34[i], null_mut(), addr_of_mut!((*d).x4C[i]));
    }

    (*d).xC = (*d).x4C[0].y - position.y;
    ext::HSD_SisLib_803A6754(0, 0);
    ext::HSD_SisLib_803A6754(0, 0);
    for i in 0..4 {
        fn_80175A94(i, &mut position);
        fn_80175240(i);
    }
}

/// fn_80175D34: drop the KO-count texts and re-seat the time texts.
pub unsafe fn fn_80175D34() {
    melee_core::ct!("fn_80175D34");
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let d = data();
    ext::lb_8000B1CC((*d).x24, null_mut(), &mut pos);

    for i in 0..4usize {
        if !(*d).player_data[i].ko_count.is_null() {
            ext::HSD_SisLib_803A5CC4((*d).player_data[i].ko_count);
            (*d).player_data[i].ko_count = null_mut();
        }

        let ko_time = (*d).player_data[i].ko_time;
        if !ko_time.is_null() {
            (*ko_time).pos_y = -pos.y;
        }
    }
}

/// The joint ids `fn_80175DC8` looks up per player slot, in the order the C does it
/// (`jobjs[0]`, `[1]`, `[2]`, `[3]`, `[5]`, `[4]`, `[6]`, ... `[14]`): `(jobjs index, ids per slot)`.
const PLAYER_JOINTS: [(usize, [i32; 4]); 15] = [
    (0, [0x42, 0x43, 0x44, 0x45]),
    (1, [0x1D, 0x25, 0x2D, 0x35]),
    (2, [0x1E, 0x26, 0x2E, 0x36]),
    (3, [0x3D, 0x3E, 0x3F, 0x40]),
    (5, [0x21, 0x29, 0x31, 0x39]),
    (4, [0x46, 0x49, 0x4C, 0x4F]),
    (6, [0x52, 0x53, 0x54, 0x55]),
    (7, [0x19, 0x1A, 0x1B, 0x1C]),
    (8, [0x6C, 0x6E, 0x70, 0x72]),
    (9, [0x6B, 0x6D, 0x6F, 0x71]),
    (10, [0x23, 0x2B, 0x33, 0x3B]),
    (11, [0x24, 0x2C, 0x34, 0x3C]),
    (12, [0x56, 0x57, 0x58, 0x59]),
    (13, [0x5A, 0x5B, 0x5C, 0x5D]),
    (14, [0x5E, 0x5F, 0x60, 0x61]),
];

/// fn_80175DC8: build the panel model and cache its joints.
pub unsafe extern "C" fn fn_80175DC8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80175DC8");
    let mut sp108: *mut HSD_JObj = null_mut();
    let mut sp104: *mut HSD_JObj;
    let mut sp100: *mut HSD_JObj;
    let mut sp_fc: *mut HSD_JObj;
    let mut sp78 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp6c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let d = data();
    let me = (*d).x94;
    let jobj;
    {
        let model = *(*(*d).pnlsce).models;
        jobj = ext::HSD_JObjLoadJoint((*model).joint);
        ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
        ext::GObj_SetupGXLink(gobj, Some(fn_80175038), 11, 0);
        ext::lb_8000C07C(jobj, 0, (*model).anims, (*model).matanims, (*model).shapeanims);
    }
    ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
    ext::HSD_JObjAnimAll(jobj);

    ext::lb_80011E24(jobj, &mut sp108, &[0x41, -1]);
    ext::fn_80179F84(sp108);
    sp104 = jobj;
    ext::lb_80011E24(jobj, &mut sp104, &[0x68, -1]);
    (*d).x24 = sp104;
    sp100 = jobj;
    ext::lb_80011E24(jobj, &mut sp100, &[0x69, -1]);
    (*d).x28 = sp100;

    let mut jobj_iter = addr_of_mut!((*d).x34) as *mut *mut HSD_JObj;
    for i in 0..6i32 {
        ext::lb_80011E24(jobj, jobj_iter, &[i + 0x62, -1]);
        jobj_iter = jobj_iter.add(1);
    }

    sp_fc = jobj;
    ext::lb_80011E24(jobj, &mut sp_fc, &[0xA, -1]);
    (*d).x30 = sp_fc;

    for j in 0..4usize {
        (*d).player_data[j].set_x0_0(0);
        for &(slot_idx, ids) in PLAYER_JOINTS.iter() {
            let mut pj: [*mut HSD_JObj; 2] = [null_mut(); 2];
            ext::lb_80011E24(jobj, pj.as_mut_ptr(), &[ids[j], -1]);
            (*d).player_data[j].jobjs[slot_idx] = pj[0];
        }
        ext::lb_8000B1CC((*d).player_data[j].jobjs[12], null_mut(), addr_of_mut!((*d).player_data[j].stats_position));
    }

    ext::fn_801785B0(gobj);
    let kind = (*me).match_kind;
    if kind <= 3 {
        let title_text = (*d).x2C;
        let title_jobj = (*d).x28;
        sp78 = lbl_803B7B18.translate;
        if !title_text.is_null() {
            ext::HSD_SisLib_803A5CC4(title_text);
        }
        ext::lb_8000B1CC(title_jobj, &mut sp78, &mut sp6c);
        let t = ext::HSD_SisLib_803A5ACC(0, 0, sp6c.x, -sp6c.y, sp6c.z, 12.0f32, 300.0f32);
        ext::HSD_SisLib_803A6368(t, kind as i32 + 2);
    }
}

/// fn_80176A6C: the panel camera and the SIS text bank.
pub unsafe fn fn_80176A6C() {
    melee_core::ct!("fn_80176A6C");
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_CAMERA as u16, 20, 0);
    if gobj.is_null() {
        ext::OSReport(c"Error : gobj dont't get (gmResultAddPanelCamera)\n".as_ptr(), &[]);
        hsd::hsd_assert(false, 1461, c"0");
    }

    let cobj = ext::HSD_CObjLoadDesc((*(*(*data()).pnlsce).cameras).desc);
    if cobj.is_null() {
        ext::OSReport(c"Error : cobj dont't get (gmResultAddPanelCamera)\n".as_ptr(), &[]);
        hsd::hsd_assert(false, 1467, c"0");
    }

    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_CameraKind, cobj.cast());
    ext::GObj_SetupGXLinkMax(gobj, Some(ext::HSD_GObj_803910D8), 8);
    if (*data()).x0_5() != 0 {
        (*gobj).gxlink_prios = 1;
    } else {
        (*gobj).gxlink_prios = 0x4D81;
    }

    ext::HSD_SisLib_803A611C(0, gobj, 9, 0xD, 0, 0xE, 0, 0x13);
    if ext::lbLang_IsSavedLanguageUS() != 0 {
        ext::HSD_SisLib_803A62A0(0, c"SdRst.usd".as_ptr(), c"SIS_ResultData".as_ptr());
    } else {
        ext::HSD_SisLib_803A62A0(0, c"SdRst.dat".as_ptr(), c"SIS_ResultData".as_ptr());
    }
    (*data()).cobj = cobj;
}

/// fn_80176BCC.
pub unsafe extern "C" fn fn_80176BCC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80176BCC");
    ext::HSD_JObjAnimAll(get_jobj(gobj));
}

/// `fn_80176BF0_inline` (static inline): the series-emblem index of a character kind (-1 = none).
pub unsafe fn fn_80176BF0_inline(arg1: u8) -> i32 {
    melee_core::ct!("fn_80176BF0_inline");
    // (TARGET_PC) m-ex characters are indexed through the m-ex insignia table
    if (arg1 as u32) >= ChKind_Mex0 && (arg1 as u32) < ChKind_Cap {
        return ext::Mex_InsigniaForExt(ext::Mex_PortCKindToExt(arg1 as i32));
    }
    for i in 0..33usize {
        if arg1 == lbl_803B7B18.lookup[i][0] {
            return lbl_803B7B18.lookup[i][1] as i32;
        }
    }
    -1
}

/// fn_80176BF0: hide all emblem joints but one; returns the shown one.
pub unsafe fn fn_80176BF0(arg0: *mut HSD_JObj, arg1: u8, arg2: i32) -> *mut HSD_JObj {
    melee_core::ct!("fn_80176BF0");
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut var_r31 = 0;
    let mut var_r30: *mut HSD_JObj = null_mut();
    ext::lb_80011E24(arg0, &mut jobj, &[5, -1]);
    let mut var_r29 = if arg2 != 0 { 11 } else { fn_80176BF0_inline(arg1) };
    {
        // (TARGET_PC) no emblem model at that index: 11
        let mut n = 0;
        let mut c = jobj_child(jobj);
        while !c.is_null() {
            n += 1;
            c = jobj_next(c);
        }
        if var_r29 < 0 || var_r29 >= n {
            var_r29 = 11;
        }
    }
    jobj = jobj_child(jobj);
    while !jobj.is_null() {
        if var_r31 != var_r29 {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        } else {
            var_r30 = jobj;
        }
        var_r31 += 1;
        jobj = jobj_next(jobj);
    }
    var_r30
}

/// fn_80176D18.
pub unsafe extern "C" fn fn_80176D18(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80176D18");
    ext::HSD_JObjAnimAll(get_jobj(gobj));
}

/// fn_80176D3C: put the result emblem models (winner / loser / draw) on the four podium spots.
pub unsafe fn fn_80176D3C(positions: *mut Vec3) {
    melee_core::ct!("fn_80176D3C");
    let d = data();
    let me = (*d).x94;
    let mut pos = positions;
    let mut models: [*mut DynamicModelDesc; 3] = [null_mut(); 3];
    models[0] = *(*(*d).flmsce).models.add(3);
    models[1] = *(*(*d).flmsce).models.add(2);
    models[2] = *(*(*d).flmsce).models.add(1);

    for i in 0..4usize {
        // (the C walks me_iter by one MatchPlayerData per loop; player_standings[0] of it is [i])
        let cur = (*me).player_standings[i];
        if cur.pkind != Gm_PKind_NA as u8 {
            let mut winner: i32 = if (*me).is_teams == 0 {
                cur.is_big_loser as i32
            } else {
                (*me).team_standings[cur.team as usize].is_big_loser as i32
            };

            if winner > 3 {
                winner = 3;
            }

            let mut skip = false;
            if (*me).outcome == OUTCOME_NO_CONTEST as u8 {
                winner = 1;
            } else if winner == 0 {
                skip = true;
            }

            if !skip {
                let gobj = ext::GObj_Create(14, 15, 0);
                let model = models.as_mut_ptr().add(winner as usize).sub(1);
                let jobj = ext::HSD_JObjLoadJoint((**model).joint);
                ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
                ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 11, 0);

                hsd::HSD_JObjSetTranslate(jobj, pos);

                ext::gm_8016895C(jobj, *model, 0);
                ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
                ext::HSD_JObjAnimAll(jobj);
                ext::HSD_GObj_SetupProc(gobj, Some(fn_80176D18), 1);
                ext::fn_80179F6C(i as i32, gobj);
            }
        }
        pos = pos.add(1);
    }
}

/// fn_80176F60: the winner emblem model, the result-title materials and the podium models.
pub unsafe fn fn_80176F60() {
    melee_core::ct!("fn_80176F60");
    let d = data();
    let mut sp8 = [Vec3 { x: 0.0, y: 0.0, z: 0.0 }; 4];

    let temp_r30 = (*d).x94;
    let temp_r27 = *(*(*d).flmsce).models;
    let temp_r29 = ext::GObj_Create(0xE, 0xF, 0);
    let mut jobj = ext::HSD_JObjLoadJoint((*temp_r27).joint);
    ext::HSD_GObjObject_80390A70(temp_r29, HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(temp_r29, Some(fn_80175038), 0xB, 0);
    ext::lb_8000C0E8(jobj, 0, temp_r27);
    ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
    (*d).x20 = fn_80176BF0(jobj, (*temp_r30).player_standings[(*d).x6 as usize].ckind as u8, gm_WasMatchCanceled((*temp_r30).outcome) as i32);
    let aobj = (*(*(*(*d).x20).u.dobj).mobj).aobj;
    let tmp = ext::gm_80160854(
        (*d).x6,
        ext::Player_GetTeam((*d).x6 as i32) as u8,
        ((*temp_r30).is_teams == 1) as u8,
        (*temp_r30).player_standings[(*d).x4 as usize].pkind,
    ) as u8;
    let new_var = aobj;
    ext::HSD_AObjSetCurrentFrame(new_var, 1.0f32 + tmp as f32);
    ext::HSD_AObjSetRate(aobj, 0.0f32);
    ext::HSD_MObjAnim((*(*(*d).x20).u.dobj).mobj);
    ext::HSD_AObjSetCurrentFrame(aobj, 0.0f32);
    ext::HSD_AObjSetRate(aobj, 1.0f32);
    ext::mn_8022F3D8((*d).x20, 4, MOBJ_MASK);
    ext::mn_8022F3D8((*d).x20, 5, MOBJ_MASK);
    ext::mn_8022F3D8((*d).x20, 6, MOBJ_MASK);
    ext::HSD_GObj_SetupProc(temp_r29, Some(fn_80176BCC), 0);
    if jobj.is_null() {
        jobj = null_mut();
    } else {
        jobj = (*jobj).child;
    }

    for i in 0..4usize {
        hsd::HSD_JObjGetTranslation(jobj, &mut sp8[i]);
        if jobj.is_null() {
            jobj = null_mut();
        } else {
            jobj = (*jobj).next;
        }
    }
    fn_80176D3C(sp8.as_mut_ptr());
}

/// fn_801771C0: the winner (x4), the winning team (x5) and the team's best player (x6).
pub unsafe fn fn_801771C0(data: *mut ResultsData) {
    melee_core::ct!("fn_801771C0");
    let me = (*data).x94;
    (*data).x4 = ext::fn_80165418(me) as u8;
    (*data).x5 = ext::fn_801654A0(me) as u8;

    if (*me).is_teams == 1 {
        for i in 0..4usize {
            if (*me).player_standings[i].pkind != Gm_PKind_NA as u8 && (*me).player_standings[i].team == (*data).x5 {
                let mut result = i as i32;
                for j in 0..4usize {
                    if (*me).player_standings[j].pkind != Gm_PKind_NA as u8
                        && (*me).player_standings[j].team == (*data).x5
                        && i != j
                        && (*me).player_standings[i].is_small_loser > (*me).player_standings[j].is_small_loser
                    {
                        result = -1;
                        break;
                    }
                }
                if result >= 0 {
                    (*data).x6 = result as u8;
                    break;
                }
            }
        }
    } else {
        (*data).x6 = (*data).x4;
    }
}

/// gmResult_PcWinner (gm_r7.rs; checked there against the C with `fn_801771C0` over local state).

/// `gmResultLoadArchive` (static inline).
pub unsafe fn gmResultLoadArchive(data: *mut ResultsData) {
    melee_core::ct!("gmResultLoadArchive");
    lbl_804D65B8 = ext::lbArchive_80016DBC(
        c"GmRst".as_ptr(),
        addr_of_mut!((*data).pnlsce).cast(),
        &[
            c"pnlsce".as_ptr().cast(),
            addr_of_mut!((*data).flmsce).cast::<c_void>() as *const c_void,
            c"flmsce".as_ptr().cast(),
            null_mut::<c_void>() as *const c_void,
        ],
    );
    if (*data).pnlsce.is_null() {
        ext::OSReport(c"Error : Cannot read archive file (File Name : %s).".as_ptr(), &[Va::P(c"GmRst".as_ptr().cast())]);
    }
    if (*data).flmsce.is_null() {
        ext::OSReport(c"Error : Cannot read archive file (File Name : %s).".as_ptr(), &[Va::P(c"GmRst".as_ptr().cast())]);
    }
}

/// `gmResultReportLightGObj` (static inline).
pub unsafe fn gmResultReportLightGObj() {
    melee_core::ct!("gmResultReportLightGObj");
    ext::OSReport(c"Error : gobj dont't get (gmResultAddLight)\n".as_ptr(), &[]);
    hsd::hsd_assert(false, 1753, c"0");
}
/// `gmResultReportLightLObj` (static inline).
pub unsafe fn gmResultReportLightLObj() {
    melee_core::ct!("gmResultReportLightLObj");
    ext::OSReport(c"Error : lobj dont't get (gmResultAddLight)\n".as_ptr(), &[]);
    hsd::hsd_assert(false, 1759, c"0");
}
/// `gmResultReportModelGObj` (static inline).
pub unsafe fn gmResultReportModelGObj() {
    melee_core::ct!("gmResultReportModelGObj");
    ext::OSReport(c"Error : gobj dont't get (gmResultAddModel)\n".as_ptr(), &[]);
    hsd::hsd_assert(false, 1765, c"0");
}

/// gm_Scene_Results_OnEnter: reset the scene state, pick the page count and build the scene.
pub unsafe extern "C" fn gm_Scene_Results_OnEnter(arg0_: *mut c_void) {
    melee_core::ct!("gm_Scene_Results_OnEnter");
    let arg0 = arg0_ as *mut ResultsMatchInfo;
    let d = data();

    ext::gmRst_GenoArtReset();
    ext::memzero(d.cast(), core::mem::size_of::<ResultsData>() as isize);
    (*d).x1 = 0;
    (*d).set_x0_0(0);
    (*d).x94 = addr_of_mut!((*arg0).match_end);
    (*d).set_x0_4((*arg0).x0_0() as u8);
    (*d).set_x0_5((*arg0).x0_1() as u8);

    let match_end = (*d).x94;
    if gm_WasMatchCanceled((*(*d).x94).outcome) {
        (*d).num_pages = 2;
    } else {
        (*d).num_pages = 3;
        if (*match_end).match_kind == 3 {
            (*d).player_data[0].page = 2;
            (*d).player_data[1].page = 2;
            (*d).player_data[2].page = 2;
            (*d).player_data[3].page = 2;
        }
    }
    if ext::fn_801701B8() == 0 {
        for i in 0..4usize {
            lbl_8046E190[i].x0 = 2;
            lbl_8046E190[i].x1 = (fn_80174284_noinline(i as u8) * 2 + 2) as u8;
        }
    } else {
        for i in 0..4usize {
            lbl_8046E190[i].x0 = 2;
            lbl_8046E190[i].x1 = 0;
        }
    }
    fn_801771C0(d);
    if (*match_end).player_standings[(*d).x6 as usize].pkind == Gm_PKind_Human as u8 {
        if !gm_WasMatchCanceled((*match_end).outcome) && (*match_end).player_standings[(*d).x6 as usize].x3_u.x0.x3_b6() != 0 {
            ext::lb_80014574((*d).x6, 3, 0x20, 0);
        }
    }
    ext::un_802FF1B4();
    gmResultLoadArchive(d);
    fn_80176A6C();
    let light_gobj = ext::GObj_Create(0xB, 3, 0);
    if light_gobj.is_null() {
        gmResultReportLightGObj();
    }
    let lobj = ext::lb_80011AC4((*(*d).pnlsce).lights);
    if lobj.is_null() {
        gmResultReportLightLObj();
    }
    ext::HSD_GObjObject_80390A70(light_gobj, HSD_GObj_LightKind as u8, lobj.cast());
    ext::GObj_SetupGXLink(light_gobj, Some(ext::HSD_GObj_LObjCallback), 0xA, 0);
    let model_gobj = ext::GObj_Create(0xE, 0xF, 0);
    (*d).x18 = model_gobj;
    if model_gobj.is_null() {
        gmResultReportModelGObj();
    }
    ext::HSD_GObj_SetupProc(model_gobj, Some(ext::fn_80179350), 0);
    fn_80176F60();
    ext::fn_8017AA78(addr_of!((*arg0).x1));
    ext::fn_8017A004();
    if !gm_WasMatchCanceled((*match_end).outcome) {
        ext::lbAudioAx_80023F28(ext::fn_80160400((*match_end).player_standings[(*d).x6 as usize].ckind as CharacterKind) as i32);
    }

    for i in 0..4usize {
        let ps = addr_of_mut!((*match_end).player_standings[i]);
        if (*ps).pkind != Gm_PKind_NA as u8 {
            ext::fn_8017A9B4(i as i32);
            (*d).player_data[i].fighter_gobj = ext::fn_8017A67C((*ps).ckind as CharacterKind, (*ps).x3_u.x0.x3_b0() as i32, i as i32);
            (*d).player_data[i].camera = ext::fn_8017A318(i as i32);
        }
    }
}

/// gm_Scene_Results_OnExit.
pub unsafe extern "C" fn gm_Scene_Results_OnExit(_unused: *mut c_void) {
    melee_core::ct!("gm_Scene_Results_OnExit");
    ext::gmRst_GenoArtClose();
    ext::fn_801701AC();
}

#[allow(dead_code)]
const _OFFS: usize = offset_of!(ResultsData, x34);
#[allow(dead_code)]
fn _unused(_: MatchPlayerData, _: ResultsPlayerData) {}

// ---- folded from gm.rs (consolidation steps 21/22) ----
/// gmresult.c `gm_WasMatchCanceled_fp`.
pub fn gm_WasMatchCanceled_fp(outcome: u8) -> bool {
    melee_core::ct!("gm_WasMatchCanceled_fp");
    outcome == OUTCOME_NO_CONTEST_gm || outcome == OUTCOME_RETRY_gm
}

// ---- folded from gm_ptr.rs (consolidation steps 21/22) ----
/// gmresult.c `gmResultFindNth_fp`: index of the n-th nonzero value (256 if there is none).
pub fn gmResultFindNth_fp(values: &[u8], mut n: i32) -> i32 {
    melee_core::ct!("gmResultFindNth_fp");
    let mut i = 0;
    while i < 256 {
        if values[i as usize] != 0 {
            n = n.wrapping_sub(1);
            if n == 0 {
                break;
            }
        }
        i += 1;
    }
    i
}

/// gmresult.c `matchWasSkipped_fp`.
pub fn matchWasSkipped_fp(me: &MatchEnd_gm_ptr) -> bool {
    melee_core::ct!("matchWasSkipped_fp");
    me.outcome == mt::OUTCOME_NO_CONTEST as u8 || me.outcome == mt::OUTCOME_RETRY as u8
}

// ---- folded from gm_r4.rs (consolidation steps 21/22) ----
/// gmresult.c `fn_801771C0_fp`: winner (x4), winning team (x5) and, in teams, the first player of
/// that team no teammate beats on `is_small_loser` (x6).
pub fn fn_801771C0_fp(data: &mut ResultsData_gm_r4) {
    melee_core::ct!("fn_801771C0_fp");
    // SAFETY: x94 is the results' MatchEnd_gm_r4, set before this runs (the C dereferences it unchecked).
    let me = unsafe { &*data.x94 };
    data.x4 = fn_80165418_gm_r4(me) as u8;
    data.x5 = fn_801654A0_gm_r4(me) as u8;
    if me.is_teams == 1 {
        let ps = &me.player_standings;
        let na = mt::Gm_PKind_NA as i32;
        for i in 0..4usize {
            if ps[i].pkind as i32 != na && ps[i].team == data.x5 {
                let mut result = i as i32;
                for j in 0..4usize {
                    if ps[j].pkind as i32 != na && ps[j].team == data.x5 && i != j
                        && ps[i].is_small_loser > ps[j].is_small_loser
                    {
                        result = -1;
                        break;
                    }
                }
                if result >= 0 {
                    data.x6 = result as u8;
                    break;
                }
            }
        }
    } else {
        data.x6 = data.x4;
    }
}

// ---- folded from gm_r7.rs (consolidation steps 21/22) ----
/// gmresult.c `gmResult_PcWinner`: run `fn_801771C0_gm_r7` on a zeroed scratch ResultsData pointing at
/// `me` and return its x6. The C's scratch is a function static, fully zeroed on every call, so
/// a local is equivalent.
pub fn gmResult_PcWinner(me: &mut MatchEnd) -> i32 {
    melee_core::ct!("gmResult_PcWinner");
    // SAFETY: ResultsData is plain data (integers, floats, raw pointers); all-zero is valid.
    let mut scratch: ResultsData = unsafe { core::mem::zeroed() };
    memzero_gm_r7(bytes_of(&mut scratch), core::mem::size_of::<ResultsData>() as isize);
    scratch.x94 = me;
    fn_801771C0_gm_r7(&mut scratch);
    scratch.x6 as i32
}
