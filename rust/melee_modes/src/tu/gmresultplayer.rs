//! Port of src/melee/gm/gmresultplayer.c (TARGET_PC build): the player-card half of the VS results
//! scene: the per-player cards' emblem / name / rank / stock-icon pictures, the page and scroll
//! input of the stats pages, the scene's frame callback and the "how many players placed" helpers.
//! It also carries the PC-only m-ex / Geno art (the file's `gmresultplayer_geno_art.inc` and the
//! shared `pc/geno/geno_art_anim.inc` / `geno_art_text.h` statics).
//!
//! State is `lbl_8046DBE8` (the `ResultsData`, owned by `gmresult.rs`), the pad arrays, the
//! `lbl_8046E3AC` results state (owned by gm_1798.c, which is not ported: it lives here until it
//! is) and the file's own statics. Callees are `ext` stubs; those that `gmresult.rs` ports forward.
//!
//! `bool` is a 32-bit int in this build, so the `static bool` flags are `i32`. None of the code
//! has a fused multiply-add site (the float math is compares, casts and one `30.0 * n` that is only
//! converted to int).
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use melee_types::{
    CharScaleEntry, CharacterKind, Gm_PKind_Human, Gm_PKind_NA, HSD_AObj, HSD_Archive, HSD_CameraDescPerspective, HSD_DObj, HSD_GObj,
    HSD_ImageDesc, HSD_JObj, HSD_MatAnim, HSD_MatAnimJoint, HSD_PadStatus, HSD_ShapeAnimDObj, HSD_TObj, HSD_Tlut, HSD_TlutDesc, HSD_WObjDesc,
    MatchEnd, ResultsCharacterData, ResultsData, ResultsPlayerConfig, ResultsPlayerData, ResultsRenderFuncs, Vec3, lbl_8046E3AC_t,
    ALL_TYPE_MASK, AOBJ_ARG_AF, HSD_A_OP_CON, HSD_A_OP_KEY, HSD_A_OP_LIN, HSD_A_OP_SLP, HSD_A_OP_SPL, HSD_A_OP_SPL0, HSD_A_T_TIMG,
    GX_TF_I4, JOBJ_HIDDEN, JOBJ_TYPE, PROJ_PERSPECTIVE, TOBJ_MASK, TOBJ_TLUT_NONE,
};

use crate::gm::gmRst_IsMex;
pub use crate::gm_r7::gm_80177724;
use crate::oracle::Va;

/// `PAD_BUTTON_A/B/START`, `PAD_STICK_LEFT/RIGHT` (dolphin/pad.h).
const PAD_BUTTON_A: u32 = 1 << 8;
const PAD_BUTTON_B: u32 = 1 << 9;
const PAD_BUTTON_START: u32 = 1 << 12;
const PAD_STICK_LEFT: u32 = 1 << 18;
const PAD_STICK_RIGHT: u32 = 1 << 19;

pub mod ext {
    use super::*;
    crate::oext_fns! {
        fn gm_80160854(a0: u8, a1: u8, a2: u8, a3: u8) -> u32;
        fn gm_80168B34(ck: CharacterKind, fk: i32, costume: i32) -> f32;
        fn fn_80168E54(ckind: i8, ftkind: i8, team: u8, is_teams: u8) -> ();
        fn fn_80179854() -> ();
        fn fn_80179D3C(g: *mut HSD_GObj, a: i32) -> ();
        fn fn_80179D60(g: *mut HSD_GObj, a: i32) -> ();
        fn fn_80179D84(g: *mut HSD_GObj, a: i32) -> ();
        fn fn_80179DA8(g: *mut HSD_GObj, a: i32) -> ();
        fn fn_80179DCC(g: *mut HSD_GObj, a: i32) -> ();
        fn fn_80179E34(g: *mut HSD_GObj, a: i32) -> ();
        fn fn_80179E9C(g: *mut HSD_GObj, a: i32) -> ();
        fn fn_80179F04(g: *mut HSD_GObj, a: i32) -> ();
        fn gm_SetDbPauseFlag(bit: i32) -> ();
        fn gm_801A4B60() -> ();
        /// gmresult.c (ported: forwarded)
        fn fn_80174274() -> *mut MatchEnd = super::super::gmresult::fn_80174274;
        fn fn_80174338() -> () = super::super::gmresult::fn_80174338;
        fn fn_8017435C() -> () = super::super::gmresult::fn_8017435C;
        fn fn_80174380() -> () = super::super::gmresult::fn_80174380;
        /// `bool` is a 32-bit int in this build
        fn gm_WasMatchCanceled(outcome: u8) -> i32 = super::was_match_canceled;
        fn fn_801748EC(list: *mut c_void, a: i32, b: i32) -> *mut c_void = super::super::gmresult::fn_801748EC;
        fn fn_80174920(data: *mut ResultsPlayerData) -> u8;
        fn fn_80174A60(list: *mut super::super::gmresult::StatsList, slot: i32) -> i32 = super::super::gmresult::fn_80174A60;
        fn fn_80174B4C(data: *mut ResultsData, slot: i32) -> () = super::super::gmresult::fn_80174B4C;
        fn fn_80174FD0(jobj: *mut HSD_JObj, a: i32) -> () = super::super::gmresult::fn_80174FD0;
        fn fn_80175C5C() -> () = super::super::gmresult::fn_80175C5C;
        fn fn_80175D34() -> () = super::super::gmresult::fn_80175D34;
        fn fn_80175DC8(g: *mut HSD_GObj) -> ();
        fn HSD_AObjReqAnim(aobj: *mut HSD_AObj, frame: f32) -> ();
        fn HSD_AObjSetRate(aobj: *mut HSD_AObj, rate: f32) -> ();
        fn HSD_AObjSetEndFrame(aobj: *mut HSD_AObj, frame: f32) -> ();
        fn HSD_AObjSetCurrentFrame(aobj: *mut HSD_AObj, frame: f32) -> ();
        fn HSD_JObjGetDObj(jobj: *mut HSD_JObj) -> *mut HSD_DObj;
        fn HSD_JObjAnimAll(jobj: *mut HSD_JObj) -> ();
        fn HSD_JObjSetFlagsAll(jobj: *mut HSD_JObj, flags: u32) -> ();
        fn HSD_JObjClearFlagsAll(jobj: *mut HSD_JObj, flags: u32) -> ();
        fn HSD_JObjRemoveAnimAll(jobj: *mut HSD_JObj) -> ();
        fn HSD_TObjReqAnim(tobj: *mut HSD_TObj, startframe: f32) -> ();
        fn HSD_TObjAnim(tobj: *mut HSD_TObj) -> ();
        fn HSD_TlutLoadDesc(desc: *mut HSD_TlutDesc) -> *mut HSD_Tlut;
        fn HSD_TlutRemove(tlut: *mut HSD_Tlut) -> ();
        fn un_802FF128(a0: i32, a1: i32, a2: i32, a3: i32) -> ();
        fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) -> ();
        fn lb_8000BA0C(jobj: *mut HSD_JObj, a: f32) -> ();
        fn lbDObjSetRateAll(dobj: *mut HSD_DObj, a: f32) -> ();
        fn lbDObjReqAnimAll(dobj: *mut HSD_DObj, a: f32) -> ();
        fn lbGetJObjCurrFrame(jobj: *mut HSD_JObj) -> f32;
        fn memzero(mem: *mut c_void, size: i32) -> ();
        fn lbAudioAx_800237A8(sfx_id: i32, sfx_vol: i32, sfx_pan: i32) -> i32;
        fn mn_8022F3D8(jobj: *mut HSD_JObj, a: u8, mask: u32) -> ();
        fn HSD_DObjAddAnimAll(dobj: *mut HSD_DObj, matanim: *mut HSD_MatAnim, shape: *mut HSD_ShapeAnimDObj) -> ();
        fn HSD_DObjReqAnimAll(dobj: *mut HSD_DObj, startframe: f32) -> ();
        fn HSD_ArchiveGetPublicAddress(archive: *mut HSD_Archive, name: *const i8) -> *mut c_void;
        fn HSD_Free(ptr: *mut c_void) -> ();
        fn HSD_MemAlloc(size: i32) -> *mut c_void;
        fn lbArchive_LoadArchive(filename: *const i8) -> *mut HSD_Archive;
        fn lbArchive_80016EFC(archive: *mut HSD_Archive) -> ();
        fn lbLang_IsSavedLanguageUS() -> i32;
        fn lbDvd_8001819C(basename: *const i8) -> *mut HSD_Archive;
        fn Mex_PortCKindToExt(ck: i32) -> i32;
        fn Mex_InsigniaForExt(ext: i32) -> i32;
        fn Mex_ResultArtIsOwn(ck: i32) -> i32;
        fn Mex_FighterName(ext: i32) -> *const i8;
        fn UI_TextI4(text: *const i8, w: i32, h: i32, style: i32, dst: *mut c_void) -> i32;
        fn gm_MexStockMatAnim() -> *mut HSD_MatAnimJoint;
        fn gm_MexStockFrame(fk: i32, costume: i32) -> f32;
        fn Geno_DefineBaseCK(ck: i32) -> i32;
        fn Geno_DefineArtBaseCK(ck: i32) -> i32;
        fn Geno_DefineName(ck: i32, out: *mut i8, cap: i32) -> i32;
        fn Geno_DefineArtOpen(ck: i32, what: i32, costume: i32) -> i32;
        fn GxTex_Width(h: i32) -> i32;
        fn GxTex_Height(h: i32) -> i32;
        fn GxTex_Format(h: i32) -> i32;
        fn GxTex_ImageSize(h: i32) -> i32;
        fn GxTex_CopyImage(h: i32, dst: *mut c_void) -> ();
        fn GxTex_Close(h: i32) -> ();
        fn mnCharSel_PcArtOpen(archive: *mut HSD_Archive) -> i32;
        fn mnCharSel_PcArtClose() -> ();
        fn mnCharSel_PcArtPortrait(ck: i32, costume: i32, img: *mut *mut HSD_ImageDesc, tlut: *mut *mut HSD_Tlut) -> i32;
    }
    crate::ext_va! {
        /// os: `(fmt, ...)`
        fn OSReport(fmt: *const i8) va 0 -> ();
    }
    crate::ext_vk! {
        /// aobj.c `HSD_ForeachAnim(obj, type, mask, func, arg_type, ...)`; every call here passes one double.
        fn HSD_ForeachAnim(obj: *mut c_void, ty: u32, mask: u32, func: *mut c_void, arg_type: u32) vk "d" -> ();
        /// lb_00B0.c, variadic in C: `(jobj, out, id, -1)`.
        fn lb_80011E24(jobj: *mut HSD_JObj, out: *mut *mut HSD_JObj) vk "ii" -> i32;
    }
}

use super::gmregclear::HSD_PadMasterStatus;
use super::gmresult::lbl_8046DBE8;


crate::globals! {
    /// `static s32 lbl_804D3FC8 = 1`
    rw pub lbl_804D3FC8: i32 = 1;
    /// `static HSD_Archive* gmRst_GenoChrArt`
    rw pub gmRst_GenoChrArt: *mut HSD_Archive = null_mut();
    /// `static bool gmRst_GenoChrTried` (a 32-bit int)
    rw pub gmRst_GenoChrTried: i32 = 0;
    /// baselib/controller.c: `HSD_PadStatus HSD_PadCopyStatus[4]`
    rw pub HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
    /// gm_1798.c: `lbl_8046E3AC_t lbl_8046E3AC` (that TU is not ported: the state lives here)
    rw pub lbl_8046E3AC: lbl_8046E3AC_t = unsafe { core::mem::zeroed() };
    /// `u32 gmResultPlayerColors[4]` (RGBA)
    ro pub gmResultPlayerColors: [u32; 4] = [0x013C59FF, 0x064E01FF, 0x54010BFF, 0x408080FF];
    /// `CharScaleEntry gmResultCharacterScaleData[]`
    ro pub gmResultCharacterScaleData: [CharScaleEntry; 32] = [
        CharScaleEntry { x0: [0.2f32, 0.0f32, 2.8f32, 0.0f32, 10.0f32, 13.5f32, 24.0f32, 7.0f32], x20: [3.6f32, 4.0f32, 3.8f32, 1.5f32] },
        CharScaleEntry { x0: [-3.0f32, 0.0f32, -0.8f32, 0.0f32, 12.0f32, 11.0f32, 12.0f32, 1.0f32], x20: [2.3f32, 2.0f32, 2.5f32, 1.0f32] },
        CharScaleEntry { x0: [3.5f32, -3.0f32, 1.0f32, 0.0f32, 17.0f32, 16.0f32, 18.0f32, 5.0f32], x20: [3.5f32, 3.3f32, 3.5f32, 1.5f32] },
        CharScaleEntry { x0: [2.5f32, -7.0f32, -2.0f32, -0.5f32, 3.5f32, -4.5f32, -7.0f32, 3.0f32], x20: [1.8f32, 0.9f32, 0.9f32, 1.3f32] },
        CharScaleEntry { x0: [0.0f32, -1.0f32, 0.0f32, 0.0f32, 0.0f32, 7.0f32, -4.0f32, 1.5f32], x20: [2.0f32, 3.2f32, 1.5f32, 1.2f32] },
        CharScaleEntry { x0: [0.0f32, -8.0f32, 5.0f32, 0.0f32, 20.0f32, 10.0f32, 13.0f32, 1.0f32], x20: [3.2f32, 3.6f32, 2.8f32, 1.1f32] },
        CharScaleEntry { x0: [-1.0f32, -4.5f32, 3.0f32, 1.0f32, 22.0f32, 22.0f32, 23.0f32, 6.0f32], x20: [3.6f32, 3.7f32, 3.7f32, 1.5f32] },
        CharScaleEntry { x0: [11.0f32, 2.0f32, 0.0f32, 0.0f32, -3.0f32, 11.0f32, 13.5f32, 5.0f32], x20: [3.0f32, 2.5f32, 3.0f32, 1.5f32] },
        CharScaleEntry { x0: [0.0f32, -3.0f32, 0.0f32, 0.1f32, 12.5f32, 10.0f32, 15.0f32, 4.0f32], x20: [3.0f32, 3.0f32, 4.0f32, 1.4f32] },
        CharScaleEntry { x0: [-2.0f32, 0.0f32, 3.0f32, 0.0f32, 20.0f32, 23.0f32, 23.0f32, 7.0f32], x20: [3.5f32, 4.0f32, 3.8f32, 1.6f32] },
        CharScaleEntry { x0: [-1.0f32, 4.0f32, -1.0f32, -0.6f32, 12.0f32, 13.5f32, 23.0f32, 3.5f32], x20: [2.8f32, 3.0f32, 3.0f32, 1.3f32] },
        CharScaleEntry { x0: [0.0f32, 0.0f32, 0.0f32, 0.0f32, 7.0f32, 4.0f32, 6.0f32, 4.0f32], x20: [3.0f32, 2.0f32, 2.5f32, 1.4f32] },
        CharScaleEntry { x0: [0.0f32, 0.0f32, 0.0f32, 0.0f32, 18.0f32, 18.0f32, 20.0f32, 7.0f32], x20: [3.5f32, 3.5f32, 4.0f32, 1.6f32] },
        CharScaleEntry { x0: [0.0f32, 2.0f32, -8.0f32, 0.0f32, 0.0f32, 9.0f32, -4.0f32, 1.5f32], x20: [1.5f32, 3.2f32, 2.2f32, 1.2f32] },
        CharScaleEntry { x0: [-8.0f32, -9.0f32, -5.0f32, -2.0f32, 10.0f32, 6.5f32, 0.8f32, 2.0f32], x20: [2.35f32, 3.0f32, 2.0f32, 1.3f32] },
        CharScaleEntry { x0: [0.0f32, 0.0f32, 0.0f32, 0.0f32, -2.0f32, 0.0f32, -7.0f32, 0.0f32], x20: [1.6f32, 2.8f32, 1.0f32, 1.1f32] },
        CharScaleEntry { x0: [0.0f32, 1.0f32, -2.0f32, 0.0f32, 18.0f32, 12.5f32, 25.0f32, 6.5f32], x20: [3.2f32, 3.8f32, 4.2f32, 1.5f32] },
        CharScaleEntry { x0: [3.0f32, -1.0f32, 2.5f32, 0.1f32, 13.0f32, 9.5f32, 13.0f32, 2.8f32], x20: [2.6f32, 2.2f32, 2.3f32, 1.2f32] },
        CharScaleEntry { x0: [-1.0f32, 1.0f32, 0.0f32, 0.2f32, 23.5f32, 22.0f32, 23.0f32, 8.0f32], x20: [4.0f32, 4.0f32, 4.0f32, 1.6f32] },
        CharScaleEntry { x0: [1.0f32, 0.0f32, 1.0f32, 0.0f32, 20.0f32, 25.0f32, 16.0f32, 7.0f32], x20: [3.5f32, 4.0f32, 4.0f32, 1.5f32] },
        CharScaleEntry { x0: [-11.0f32, -9.0f32, -3.0f32, 0.5f32, 9.0f32, 7.0f32, 24.0f32, 5.0f32], x20: [3.0f32, 3.5f32, 3.8f32, 1.5f32] },
        CharScaleEntry { x0: [1.0f32, -5.5f32, -0.5f32, 1.0f32, 18.0f32, 19.0f32, 20.0f32, 6.5f32], x20: [3.3f32, 4.0f32, 3.8f32, 1.5f32] },
        CharScaleEntry { x0: [0.0f32, 0.5f32, 0.0f32, 0.2f32, 10.0f32, 13.0f32, 14.0f32, 5.0f32], x20: [3.0f32, 3.0f32, 3.6f32, 1.5f32] },
        CharScaleEntry { x0: [-15.0f32, 3.5f32, -0.3f32, 0.0f32, 15.0f32, 23.0f32, 22.0f32, 5.5f32], x20: [4.0f32, 3.5f32, 4.0f32, 1.5f32] },
        CharScaleEntry { x0: [0.0f32, 2.0f32, 0.0f32, 0.0f32, -3.0f32, 1.0f32, 1.0f32, 0.5f32], x20: [1.3f32, 2.5f32, 2.0f32, 1.3f32] },
        CharScaleEntry { x0: [0.5f32, 0.0f32, 0.0f32, 0.0f32, 19.0f32, 27.0f32, 25.0f32, 7.5f32], x20: [4.1f32, 4.0f32, 4.0f32, 1.5f32] },
        CharScaleEntry { x0: [0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32], x20: [1.0f32, 1.0f32, 1.0f32, 1.0f32] },
        CharScaleEntry { x0: [0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32], x20: [1.0f32, 1.0f32, 1.0f32, 1.0f32] },
        CharScaleEntry { x0: [0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32], x20: [1.0f32, 1.0f32, 1.0f32, 1.0f32] },
        CharScaleEntry { x0: [0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32], x20: [1.0f32, 1.0f32, 1.0f32, 1.0f32] },
        CharScaleEntry { x0: [0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32], x20: [1.0f32, 1.0f32, 1.0f32, 1.0f32] },
        CharScaleEntry { x0: [0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32], x20: [1.0f32, 1.0f32, 1.0f32, 1.0f32] },
    ];
    /// `u32 gmResultX22F4Init[8]`
    ro pub gmResultX22F4Init: [u32; 8] = [0x00180000, 0x00000000, 0x00150015, 0x00000000, 0x00120012, 0x00120000, 0x000E000E, 0x000E000E];
    /// `u32 gmResultScoreTableInit[8]`
    ro pub gmResultScoreTableInit: [u32; 8] = [0x00000000, 0x00000000, 0xFFF2000E, 0x00000000, 0xFFEE0000, 0x00120000, 0xFFEAFFF9, 0x00070016];
    /// `ResultsCharacterData gmResultCharacterData`
    ro pub gmResultCharacterData: ResultsCharacterData = ResultsCharacterData {
        scale: [0.85f32, 0.8f32, 1.0f32, 1.0f32, 1.0f32, 0.7f32, 0.9f32, 1.0f32, 1.0f32, 0.88f32, 0.8f32, 1.0f32, 0.9f32, 1.0f32, 0.9f32, 1.0f32, 0.9f32, 0.9f32, 0.9f32, 0.9f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 0.79f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32],
        slot_off: [
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -3.5f32, -4.0f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -3.0f32, -2.3f32, -1.5f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -3.3f32, -3.5f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.0f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.4f32, -1.5f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.4f32, -1.2f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, -0.3f32, -0.6f32], [0.0f32, -2.7f32, -3.1f32, -3.5f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.9f32, -3.2f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.7f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, -0.2f32, -0.4f32], [0.0f32, -2.7f32, -3.1f32, -3.5f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.7f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.7f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.3f32, -2.9f32, -3.5f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.0f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.4f32, 0.9f32], [0.0f32, -3.3f32, -2.9f32, -2.0f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.0f32, -1.0f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.9f32, -3.3f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.1f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -3.5f32, -3.9f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -3.3f32, -3.9f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.5f32, -2.6f32, -2.7f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, -0.1f32, -0.3f32, -0.6f32], [0.0f32, -2.7f32, -2.9f32, -3.2f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.9f32, -3.1f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, -0.2f32, -0.4f32, -0.5f32], [0.0f32, -2.7f32, -3.0f32, -3.3f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.0f32, -0.7f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -3.3f32, -3.9f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.7f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.7f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.7f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.7f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.7f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
            [[0.0f32, 0.0f32, 0.0f32, 0.0f32], [0.0f32, -2.7f32, -2.7f32, -2.7f32], [0.0f32, 0.0f32, 0.0f32, 0.0f32]],
        ],
        x680: {
            let mut x = [[0u8; 16]; 27];
            x[0][0] = 0x17;
            x[1][0] = 0x29;
            x[2][0] = 0x27;
            x[3][0] = 0x2C;
            x[4][0] = 0x05;
            x[5][0] = 0x17;
            x[6][0] = 0x28;
            x[7][0] = 0x24;
            x[8][0] = 0x18;
            x[9][0] = 0x57;
            x[10][0] = 0x13;
            x[11][0] = 0x13;
            x[12][0] = 0x17;
            x[13][0] = 0x17;
            x[14][0] = 0x17;
            x[15][0] = 0x17;
            x[16][0] = 0x17;
            x[17][0] = 0x17;
            x[18][0] = 0x17;
            x[19][0] = 0x17;
            x[20][0] = 0x17;
            x[21][0] = 0x17;
            x[22][0] = 0x17;
            x[23][0] = 0x17;
            x[24][0] = 0x17;
            x[25][0] = 0x17;
            x[26][0] = 0x17;
            x
        },
        pad_830: [0; 96],
    };
    rp pub gmResultCameraEyeDesc: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 62.0 }, robjdesc: null_mut() };
    rp pub gmResultCameraInterestDesc: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, robjdesc: null_mut() };
    rp pub gmResultCameraDesc: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
        class_name: null_mut(),
        flags: 0,
        projection_type: PROJ_PERSPECTIVE as u16,
        viewport: melee_types::_HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
        scissor: melee_types::_Scissor { left: 0, right: 640, top: 0, bottom: 480 },
        eyepos: addr_of_mut!(gmResultCameraEyeDesc),
        interest: addr_of_mut!(gmResultCameraInterestDesc),
        roll: 0.0,
        up_vector: null_mut(),
        nnear: 1.0,
        ffar: 5000.0,
        fov: 19.999998,
        aspect: 1.216667,
    };
    ro pub gmResultMissingGObjMessage: [u8; 54] = *b"Error : model gobj dont't find at gmResultSetViewPos\n\0";
    ro pub gmResultSourceFileName: [u8; 17] = *b"gmresultplayer.c\0";
    ro pub gmResultMissingJObjMessage: [u8; 54] = *b"Error : model jobj dont't find at gmResultSetViewPos\n\0";
    /// `ResultsPlayerConfig const lbl_803B7B68`
    rp pub lbl_803B7B68: ResultsPlayerConfig = ResultsPlayerConfig {
        x00: [0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -100.0],
        x24: Vec3 { x: 0.0, y: 100.0, z: 62.0 },
        x30: Vec3 { x: 0.0, y: 100.0, z: 0.0 },
        x3C: ResultsRenderFuncs { funcs: [Some(ext::fn_80179DCC), Some(ext::fn_80179E34), Some(ext::fn_80179E9C), Some(ext::fn_80179F04)] },
        x4C: Vec3 { x: 0.0, y: 100.0, z: 62.0 },
        x58: Vec3 { x: 0.0, y: 100.0, z: 0.0 },
        x64: ResultsRenderFuncs { funcs: [Some(ext::fn_80179D3C), Some(ext::fn_80179D60), Some(ext::fn_80179D84), Some(ext::fn_80179DA8)] },
        x74: 0.0,
        x78: 0.0,
        x7C: -100.0,
        x80: 0.0,
        x84: 0.0,
        x88: -100.0,
        x8C: 0.75,
        x90: 0.48,
        x94: 0.4,
        x98: 0.307,
        x9C: 0.0,
    };
}

crate::globals_ext! {
    rw lbl_8046DBE8 = super::gmresult::lbl_8046DBE8: ResultsData;
    rw lbl_803D6878 = super::gmresult::lbl_803D6878: [super::gmresult::StatsList; 4];
    rw HSD_PadMasterStatus = super::gmregclear::HSD_PadMasterStatus: [HSD_PadStatus; 4];
}

#[inline]
unsafe fn data() -> *mut ResultsData {
    addr_of_mut!(lbl_8046DBE8)
}

#[inline]
unsafe fn copy_pad(i: usize) -> *mut HSD_PadStatus {
    (addr_of_mut!(HSD_PadCopyStatus) as *mut HSD_PadStatus).add(i)
}
#[inline]
unsafe fn master_pad(i: usize) -> *mut HSD_PadStatus {
    (addr_of_mut!(HSD_PadMasterStatus) as *mut HSD_PadStatus).add(i)
}

/// `HSD_ForeachAnim(jobj, JOBJ_TYPE, ALL_TYPE_MASK, HSD_AObjSetRate, AOBJ_ARG_AF, rate)`.
unsafe fn foreach_rate(jobj: *mut HSD_JObj, rate: f64) {
    ext::HSD_ForeachAnim(jobj.cast(), JOBJ_TYPE, ALL_TYPE_MASK, ext::HSD_AObjSetRate as *const () as *mut c_void, AOBJ_ARG_AF, &[Va::D(rate)]);
}
/// `HSD_ForeachAnim(jobj, JOBJ_TYPE, ALL_TYPE_MASK, HSD_AObjSetCurrentFrame, AOBJ_ARG_AF, frame)`.
unsafe fn foreach_frame(jobj: *mut HSD_JObj, frame: f64) {
    ext::HSD_ForeachAnim(
        jobj.cast(),
        JOBJ_TYPE,
        ALL_TYPE_MASK,
        ext::HSD_AObjSetCurrentFrame as *const () as *mut c_void,
        AOBJ_ARG_AF,
        &[Va::D(frame)],
    );
}

/// `inline0`: freeze a joint's animation on frame `f`.
pub unsafe fn inline0(jobj: *mut HSD_JObj, f: f32) {
    foreach_rate(jobj, 0.0);
    foreach_frame(jobj, f as f64);
    ext::HSD_JObjAnimAll(jobj);
}

// ------------------------------------------------------------------------------------------
// geno_art_anim.inc / geno_art_text.h
// ------------------------------------------------------------------------------------------

/// A shift by 32 or more gives 0, as on the PowerPC (x86 and Rust mask the count instead).
#[inline]
fn slw(x: u32, shift: i32) -> u32 {
    if shift as u32 >= 32 { 0 } else { x << shift }
}

/// One key value of an FObj stream (fobj.c's own reader, restated): the format is the top 3 bits
/// of `frac`, the fixed-point shift the low 5.
pub unsafe fn genoArt_FObjValue(p: *mut *mut u8, frac: u8) -> f32 {
    let s = *p;
    let scale = (1i32 << (frac & 0x1F)) as f32;
    match frac & 0xE0 {
        0x00 => {
            let v = ((*s as u32) << 24) | ((*s.add(1) as u32) << 16) | ((*s.add(2) as u32) << 8) | *s.add(3) as u32;
            *p = s.add(4);
            f32::from_bits(v)
        }
        0x20 => {
            *p = s.add(2);
            (((((*s as i32) << 8) | *s.add(1) as i32) as i16) as f32) / scale
        }
        0x40 => {
            *p = s.add(2);
            (((((*s as i32) << 8) | *s.add(1) as i32) as u16) as f32) / scale
        }
        0x60 => {
            *p = s.add(1);
            ((*s as i8) as f32) / scale
        }
        _ => {
            *p = s.add(1);
            (*s as f32) / scale
        }
    }
}

pub unsafe fn genoArt_FObjPacked(p: *mut *mut u8) -> u32 {
    let mut s = *p;
    let mut v = (*s & 0x7F) as u32;
    let mut shift = 7;
    loop {
        let more = *s & 0x80 != 0;
        s = s.add(1);
        if !more {
            break;
        }
        v |= slw((*s & 0x7F) as u32, shift);
        shift += 7;
    }
    *p = s;
    v
}

/// Walk the texture-image track (`HSD_A_T_TIMG`) of `tobj`'s animation. Returns true when a key
/// sits exactly on `frame` (a frame with no key of its own just holds the previous name), and
/// stores the largest image index any key selects in `*max_img` (-1 when there is no track).
pub unsafe fn genoArt_ImageKeyAt(tobj: *mut HSD_TObj, frame: i32, max_img: *mut i32) -> bool {
    let mut hit = false;
    *max_img = -1;
    if tobj.is_null() || (*tobj).aobj.is_null() {
        return false;
    }
    let mut fo = (*(*tobj).aobj).fobj;
    while !fo.is_null() {
        if (*fo).obj_type as i32 != HSD_A_T_TIMG || (*fo).ad_head.is_null() {
            fo = (*fo).next;
            continue;
        }
        let mut p = (*fo).ad_head;
        let end = p.add((*fo).length as usize);
        let mut t = (*fo).startframe as f32;
        while p < end {
            let mut b = *p;
            p = p.add(1);
            let op = (b & 0xF) as i32;
            let mut n = ((b >> 4) & 7) as i32;
            let mut shift = 3;
            while b & 0x80 != 0 {
                b = *p;
                p = p.add(1);
                n += slw((b & 0x7F) as u32, shift) as i32;
                shift += 7;
            }
            n += 1;
            while n > 0 {
                #[allow(unused_assignments)]
                let mut v = 0.0f32;
                let mut wait = 0u32;
                match op {
                    HSD_A_OP_CON | HSD_A_OP_LIN | HSD_A_OP_SPL0 => {
                        v = genoArt_FObjValue(&mut p, (*fo).frac_value);
                        wait = genoArt_FObjPacked(&mut p);
                    }
                    HSD_A_OP_SPL => {
                        v = genoArt_FObjValue(&mut p, (*fo).frac_value);
                        genoArt_FObjValue(&mut p, (*fo).frac_slope);
                        wait = genoArt_FObjPacked(&mut p);
                    }
                    HSD_A_OP_SLP => {
                        genoArt_FObjValue(&mut p, (*fo).frac_slope);
                        n -= 1;
                        continue;
                    }
                    HSD_A_OP_KEY => {
                        v = genoArt_FObjValue(&mut p, (*fo).frac_value);
                    }
                    _ => return hit,
                }
                if t as i32 == frame {
                    hit = true;
                }
                if v as i32 > *max_img {
                    *max_img = v as i32;
                }
                t += wait as f32;
                n -= 1;
            }
        }
        fo = (*fo).next;
    }
    hit
}

/// geno_art_text.h `genoArt_Initials`: the first two words, or two letters of a single word.
pub unsafe fn genoArt_Initials(name: *const u8, out: *mut u8) {
    let mut n = 0usize;
    let mut start = true;
    let mut second = 0u8;
    let mut i = 0usize;
    while *name.add(i) != 0 {
        let mut c = *name.add(i);
        let alnum = c.is_ascii_uppercase() || c.is_ascii_lowercase() || c.is_ascii_digit();
        if c.is_ascii_lowercase() {
            c -= b'a' - b'A';
        }
        if alnum && start && n < 2 {
            *out.add(n) = c;
            n += 1;
        } else if alnum && n == 1 && second == 0 {
            second = c;
        }
        start = !alnum;
        i += 1;
    }
    if n == 1 && second != 0 {
        *out.add(n) = second;
        n += 1;
    }
    if n == 0 {
        *out.add(n) = b'?';
        n += 1;
    }
    *out.add(n) = 0;
}

// ------------------------------------------------------------------------------------------
// m-ex names and attachments
// ------------------------------------------------------------------------------------------

/// The results name frame of CharacterKind `ck` in the name strip `tobj`: retail's formula for the
/// retail cast, the external id for an m-ex fighter whose name the loaded GmRst really has, -1
/// for an m-ex fighter it does not (the caller draws the name then).
pub unsafe fn gmRst_NameFrame(tobj: *mut HSD_TObj, ck: i32, fk: i32) -> i32 {
    let mut max_img = 0i32;
    if !gmRst_IsMex(ck) {
        return ext::gm_80168B34(ck as CharacterKind, fk, 0) as i32;
    }
    let ext_ = ext::Mex_PortCKindToExt(ck);
    if ext_ < 0 || ext::Mex_ResultArtIsOwn(ck) == 0 || !genoArt_ImageKeyAt(tobj, ext_, &mut max_img) {
        return -1;
    }
    ext_
}

/// Give name strip `tobj` a texture of m-ex fighter `ck`'s own name, drawn from its MxDt name,
/// and point every image its animation can select at it. Scene-heap memory.
pub unsafe fn gmRst_DrawName(tobj: *mut HSD_TObj, ck: i32, style: i32) {
    static mut geno_name: [u8; 48] = [0; 48];
    let mut upper = [0u8; 48];
    let mut max_img = 0i32;
    if tobj.is_null() {
        return;
    }
    let cur = (*tobj).imagedesc;
    if cur.is_null() {
        return;
    }
    let w = (*cur).width as i32;
    let h = (*cur).height as i32;
    let hp = (h + 7) & !7;
    let ext_ = ext::Mex_PortCKindToExt(ck);
    let mut name: *const u8 = if ext_ >= 0 { ext::Mex_FighterName(ext_) as *const u8 } else { core::ptr::null() };
    if name.is_null() {
        // a Geno define is no m-ex slot (ext < 0): its name is the one its package declares
        if ext::Geno_DefineName(ck, addr_of_mut!(geno_name) as *mut i8, 48) != 0 {
            name = addr_of!(geno_name) as *const u8;
        }
    }
    let mut i = 0usize;
    while !name.is_null() && *name.add(i) != 0 && i < upper.len() - 1 {
        let c = *name.add(i);
        upper[i] = if c.is_ascii_lowercase() { c - b'a' + b'A' } else { c };
        i += 1;
    }
    upper[i] = 0;
    let desc = ext::HSD_MemAlloc(core::mem::size_of::<HSD_ImageDesc>() as i32) as *mut HSD_ImageDesc;
    let mut pix = ext::HSD_MemAlloc(w * hp / 2 + 32) as *mut u8;
    genoArt_ImageKeyAt(tobj, -1, &mut max_img);
    if max_img < 0 {
        max_img = 0;
    }
    let tbl = ext::HSD_MemAlloc((max_img + 2) * core::mem::size_of::<*mut HSD_ImageDesc>() as i32) as *mut *mut HSD_ImageDesc;
    if desc.is_null() || pix.is_null() || tbl.is_null() {
        return;
    }
    pix = ((pix as usize + 31) & !31usize) as *mut u8;
    ext::memzero(pix.cast(), w * hp / 2);
    if (*cur).format == GX_TF_I4 && upper[0] != 0 {
        ext::UI_TextI4(upper.as_ptr().cast(), w, h, style, pix.cast());
    }
    ext::OSReport(
        c"gw: results: ck %d (m-ex ext %d) has no name art of its own - drawn \"%s\" %dx%d\n".as_ptr(),
        &[Va::I32(ck), Va::I32(ext_), Va::P(upper.as_ptr().cast()), Va::I32(w), Va::I32(h)],
    );
    (*desc).image_ptr = pix.cast();
    (*desc).width = w as u16;
    (*desc).height = h as u16;
    (*desc).format = GX_TF_I4;
    (*desc).mipmap = 0;
    (*desc).minLOD = 0.0f32;
    (*desc).maxLOD = 0.0f32;
    i = 0;
    while i as i32 <= max_img {
        *tbl.add(i) = desc;
        i += 1;
    }
    *tbl.add(max_img as usize + 1) = null_mut();
    (*tobj).imagetbl = tbl;
    (*tobj).imagedesc = desc;
}

/// m-ex "Replace Results Stock and Emblem Matanim", per player: `jobj`'s DObj `dobj_index` takes
/// IfAll's `symbol` matanim. False when the disc's IfAll has no such symbol (a retail disc).
pub unsafe fn gmRst_MexAttach(jobj: *mut HSD_JObj, mut dobj_index: i32, stock: bool) -> bool {
    if jobj.is_null() {
        return false;
    }
    let mut dobj = (*jobj).u.dobj;
    if dobj.is_null() {
        return false;
    }
    let maj: *mut HSD_MatAnimJoint;
    if stock {
        maj = ext::gm_MexStockMatAnim();
    } else {
        let ifall = ext::lbDvd_8001819C(c"IfAll".as_ptr());
        maj = if !ifall.is_null() {
            ext::HSD_ArchiveGetPublicAddress(ifall, c"Eblm_matanim_joint".as_ptr()) as *mut HSD_MatAnimJoint
        } else {
            null_mut()
        };
    }
    if maj.is_null() || (*maj).matanim.is_null() {
        return false;
    }
    while dobj_index > 0 && !dobj.is_null() {
        dobj = (*dobj).next;
        dobj_index -= 1;
    }
    if dobj.is_null() {
        return false;
    }
    ext::HSD_DObjAddAnimAll(dobj, (*maj).matanim, null_mut());
    // a freshly added AObj does not run until requested; the caller then sets its frame
    ext::HSD_DObjReqAnimAll(dobj, 0.0f32);
    true
}

// ------------------------------------------------------------------------------------------
// gmresultplayer_geno_art.inc
// ------------------------------------------------------------------------------------------

/// The TObj of DObj number `index` of `jobj`.
pub unsafe fn gmRst_GenoTObjAt(jobj: *mut HSD_JObj, mut index: i32) -> *mut HSD_TObj {
    if jobj.is_null() {
        return null_mut();
    }
    let mut dobj = (*jobj).u.dobj;
    if dobj.is_null() {
        return null_mut();
    }
    while index > 0 && !dobj.is_null() {
        dobj = (*dobj).next;
        index -= 1;
    }
    if !dobj.is_null() && !(*dobj).mobj.is_null() {
        (*(*dobj).mobj).tobj
    } else {
        null_mut()
    }
}

/// As ifStock_ReqAnim: every texture-image key selects our scene-owned image.
pub unsafe fn gmRst_GenoSetImage(tobj: *mut HSD_TObj, desc: *mut HSD_ImageDesc) -> bool {
    let mut max_img = 0i32;
    if tobj.is_null() || desc.is_null() {
        return false;
    }
    genoArt_ImageKeyAt(tobj, -1, &mut max_img);
    if max_img < 0 {
        max_img = 0;
    }
    if max_img > 65535 {
        return false;
    }
    let tbl = ext::HSD_MemAlloc((max_img + 2) * core::mem::size_of::<*mut HSD_ImageDesc>() as i32) as *mut *mut HSD_ImageDesc;
    if tbl.is_null() {
        return false;
    }
    let mut i = 0usize;
    while i as i32 <= max_img {
        *tbl.add(i) = desc;
        i += 1;
    }
    *tbl.add(max_img as usize + 1) = null_mut();
    (*tobj).imagetbl = tbl;
    (*tobj).imagedesc = desc;
    true
}

/// Give `tobj` the package art of Geno define `ck` (`what`: 1 CSP, 2 stock, 3 emblem, 4 results).
pub unsafe fn gmRst_GenoApplyArt(tobj: *mut HSD_TObj, ck: i32, what: i32, costume: i32) -> bool {
    if tobj.is_null() {
        return false;
    }
    let h = ext::Geno_DefineArtOpen(ck, what, costume);
    if h < 0 {
        return false;
    }
    let w = ext::GxTex_Width(h);
    let ht = ext::GxTex_Height(h);
    let size = ext::GxTex_ImageSize(h);
    let fmt = ext::GxTex_Format(h);
    if fmt >= 8 || w < 1 || ht < 1 || size < 1 {
        ext::OSReport(c"geno: results art of ck %d (kind %d): unusable format %d\n".as_ptr(), &[Va::I32(ck), Va::I32(what), Va::I32(fmt)]);
        ext::GxTex_Close(h);
        return false;
    }
    let allocation = ext::HSD_MemAlloc(size + 32) as *mut u8;
    let desc = ext::HSD_MemAlloc(core::mem::size_of::<HSD_ImageDesc>() as i32) as *mut HSD_ImageDesc;
    if allocation.is_null() || desc.is_null() {
        if !allocation.is_null() {
            ext::HSD_Free(allocation.cast());
        }
        if !desc.is_null() {
            ext::HSD_Free(desc.cast());
        }
        ext::GxTex_Close(h);
        return false;
    }
    let pix = ((allocation as usize + 31) & !31usize) as *mut u8;
    ext::GxTex_CopyImage(h, pix.cast());
    ext::GxTex_Close(h);
    (*desc).image_ptr = pix.cast();
    (*desc).width = w as u16;
    (*desc).height = ht as u16;
    (*desc).format = fmt as _;
    (*desc).mipmap = 0;
    (*desc).minLOD = 0.0f32;
    (*desc).maxLOD = 0.0f32;
    if !gmRst_GenoSetImage(tobj, desc) {
        ext::HSD_Free(allocation.cast());
        ext::HSD_Free(desc.cast());
        return false;
    }
    let what_s: *const i8 = if what == 2 {
        c"stock icon".as_ptr()
    } else if what == 3 {
        c"emblem".as_ptr()
    } else {
        c"card picture".as_ptr()
    };
    ext::OSReport(
        c"geno: results %s of ck %d: %dx%d format %d from its package\n".as_ptr(),
        &[Va::P(what_s.cast()), Va::I32(ck), Va::I32(w), Va::I32(ht), Va::I32(fmt)],
    );
    true
}

/// No visual donor: draw the define's initials with the existing I4 name-strip renderer.
pub unsafe fn gmRst_GenoLetters(tobj: *mut HSD_TObj, ck: i32) -> bool {
    let mut name = [0u8; 48];
    let mut abbr = [0u8; 3];
    if tobj.is_null() {
        return false;
    }
    let cur = (*tobj).imagedesc;
    if cur.is_null() || ext::Geno_DefineName(ck, name.as_mut_ptr().cast(), 48) == 0 {
        return false;
    }
    genoArt_Initials(name.as_ptr(), abbr.as_mut_ptr());
    let w = (*cur).width as i32;
    let h = (*cur).height as i32;
    if w < 1 || h < 1 || w > 1024 || h > 1024 {
        return false;
    }
    let bytes = ((w + 7) / 8) * ((h + 7) / 8) * 32;
    let allocation = ext::HSD_MemAlloc(bytes + 32) as *mut u8;
    let desc = ext::HSD_MemAlloc(core::mem::size_of::<HSD_ImageDesc>() as i32) as *mut HSD_ImageDesc;
    if allocation.is_null() || desc.is_null() {
        if !allocation.is_null() {
            ext::HSD_Free(allocation.cast());
        }
        if !desc.is_null() {
            ext::HSD_Free(desc.cast());
        }
        return false;
    }
    let pix = ((allocation as usize + 31) & !31usize) as *mut u8;
    core::ptr::write_bytes(pix, 0, bytes as usize);
    ext::UI_TextI4(abbr.as_ptr().cast(), w, h, 0, pix.cast());
    (*desc).image_ptr = pix.cast();
    (*desc).width = w as u16;
    (*desc).height = h as u16;
    (*desc).format = GX_TF_I4;
    (*desc).mipmap = 0;
    (*desc).minLOD = 0.0f32;
    (*desc).maxLOD = 0.0f32;
    if !gmRst_GenoSetImage(tobj, desc) {
        ext::HSD_Free(allocation.cast());
        ext::HSD_Free(desc.cast());
        return false;
    }
    ext::OSReport(c"geno: results letters of ck %d: %s\n".as_ptr(), &[Va::I32(ck), Va::P(abbr.as_ptr().cast())]);
    true
}

/// `gmRst_GenoArtReset`: forget the owned CSS archive.
pub unsafe fn gmRst_GenoArtReset() {
    gmRst_GenoChrArt = null_mut();
    gmRst_GenoChrTried = 0;
}

/// `gmRst_GenoArtClose`: close and free the owned CSS archive.
pub unsafe fn gmRst_GenoArtClose() {
    if !gmRst_GenoChrArt.is_null() {
        ext::mnCharSel_PcArtClose();
        ext::lbArchive_80016EFC(gmRst_GenoChrArt);
    }
    gmRst_GenoArtReset();
}

/// Own the disc archive for the whole results scene and put the donor fighter's CSS portrait on
/// `tobj` (palette copied: `TObjRelease` owns its TLUT objects).
pub unsafe fn gmRst_GenoRetailPortrait(tobj: *mut HSD_TObj, base: i32, costume: i32) -> bool {
    let mut image: *mut HSD_ImageDesc = null_mut();
    let mut palette: *mut HSD_Tlut = null_mut();
    let mut copy: *mut HSD_Tlut = null_mut();
    let mut desc: HSD_TlutDesc = core::mem::zeroed();
    if tobj.is_null() || base < 0 {
        return false;
    }
    if gmRst_GenoChrTried == 0 {
        gmRst_GenoChrTried = 1;
        gmRst_GenoChrArt = ext::lbArchive_LoadArchive(if ext::lbLang_IsSavedLanguageUS() != 0 {
            c"MnSlChr.usd".as_ptr()
        } else {
            c"MnSlChr.dat".as_ptr()
        });
        ext::mnCharSel_PcArtOpen(gmRst_GenoChrArt);
    }
    if gmRst_GenoChrArt.is_null()
        || ext::mnCharSel_PcArtPortrait(base, costume, &mut image, &mut palette) == 0
        || image.is_null()
        || (*image).image_ptr.is_null()
    {
        return false;
    }
    if (*image).format == 8 || (*image).format == 9 || (*image).format == 10 {
        if palette.is_null() || (*palette).lut.is_null() {
            return false;
        }
        desc.lut = (*palette).lut;
        desc.fmt = (*palette).fmt;
        desc.tlut_name = (*palette).tlut_name;
        desc.n_entries = (*palette).n_entries;
        copy = ext::HSD_TlutLoadDesc(&mut desc);
        if copy.is_null() {
            return false;
        }
    }
    if !gmRst_GenoSetImage(tobj, image) {
        ext::HSD_TlutRemove(copy);
        return false;
    }
    // TObjRelease owns its TLUT objects: never borrow the CSS TObj's object.
    ext::HSD_TlutRemove((*tobj).tlut);
    if !(*tobj).tluttbl.is_null() {
        let mut i = 0usize;
        while !(*(*tobj).tluttbl.add(i)).is_null() {
            ext::HSD_TlutRemove(*(*tobj).tluttbl.add(i));
            i += 1;
        }
        ext::HSD_Free((*tobj).tluttbl.cast());
    }
    (*tobj).tlut = copy;
    (*tobj).tluttbl = null_mut();
    (*tobj).tlut_no = TOBJ_TLUT_NONE;
    true
}

/// Returns true for a Geno define, even if it has no loadable package art.
pub unsafe fn gmRst_GenoCard(card: *mut HSD_JObj, ck: i32, costume: i32) -> bool {
    if ext::Geno_DefineBaseCK(ck) < 0 {
        return false;
    }
    let base = ext::Geno_DefineArtBaseCK(ck);
    ext::HSD_JObjClearFlagsAll(card, JOBJ_HIDDEN as u32);
    inline0(card, if base >= 0 { ext::gm_80168B34(base as CharacterKind, 0, 0) } else { 0.0f32 });
    let picture = gmRst_GenoTObjAt(card, 1);
    let own = gmRst_GenoApplyArt(picture, ck, 4, costume);
    if !own {
        if gmRst_GenoRetailPortrait(picture, base, costume) {
            ext::OSReport(c"geno: results card of ck %d: retail ck %d\n".as_ptr(), &[Va::I32(ck), Va::I32(base)]);
        } else if !gmRst_GenoLetters(picture, ck) {
            ext::HSD_JObjSetFlagsAll(card, JOBJ_HIDDEN as u32);
        }
    }
    gmRst_GenoApplyArt(gmRst_GenoTObjAt(card, 0), ck, 3, 0);
    true
}

// ------------------------------------------------------------------------------------------
// the scene
// ------------------------------------------------------------------------------------------

/// fn_80177748: show each present player's card emblem and result banner.
pub unsafe fn fn_80177748() {
    let d = data();
    let temp_r3 = ext::fn_80174274();
    let mut var_r24: i32 = 0;

    for i in 0..4usize {
        let ps = addr_of_mut!((*temp_r3).player_standings[i]);
        if (*ps).pkind != Gm_PKind_NA as u8 {
            let ckind = (*ps).ckind as i32;
            let jobjs = addr_of_mut!((*d).player_data[i].jobjs);
            let j0 = (*jobjs)[0];
            ext::HSD_JObjClearFlagsAll(j0, JOBJ_HIDDEN as u32);
            if gmRst_GenoCard(j0, ckind, (*ps).anon_3.anon_0.x3_b0() as i32) {
                // Geno handled package art, retail fallback or initials.
            } else if gmRst_IsMex(ckind) {
                // m-ex: IfAll's emblem atlas at insignia[external id]; no emblem at all rather
                // than another fighter's when the disc has no atlas or no entry.
                let emblem = ext::Mex_InsigniaForExt(ext::Mex_PortCKindToExt(ckind));
                if emblem >= 0 && gmRst_MexAttach((*jobjs)[0], 0, false) {
                    inline0((*jobjs)[0], emblem as f32);
                } else {
                    ext::HSD_JObjSetFlagsAll((*jobjs)[0], JOBJ_HIDDEN as u32);
                }
            } else {
                inline0((*jobjs)[0], ext::gm_80168B34(ckind as CharacterKind, 0, 0));
            }
            ext::HSD_JObjClearFlagsAll((*jobjs)[4], JOBJ_HIDDEN as u32);
            if ext::gm_WasMatchCanceled((*temp_r3).outcome) != 0 {
                var_r24 = 4;
            } else if (*temp_r3).is_teams == 0 {
                if (*ps).is_small_loser == 0 {
                    var_r24 = 5;
                } else {
                    var_r24 = (*ps).is_big_loser as i32;
                }
            } else if (*temp_r3).is_teams == 1 {
                if (*d).x6 as i32 == i as i32 {
                    var_r24 = 5;
                } else {
                    var_r24 = (*temp_r3).team_standings[(*ps).team as usize].is_big_loser as i32;
                }
            }
            inline0((*jobjs)[4], var_r24 as f32);
        }
    }
}

/// fn_80177920: any pad press on the results card moves the scene on (x1 = 3).
pub unsafe fn fn_80177920(gobj: *mut HSD_GObj) {
    let d = data();
    let end = ext::fn_80174274();
    let mut human_controller_count = 0;

    for i in 0..4usize {
        if (*end).player_standings[i].pkind == Gm_PKind_Human as u8 && (*master_pad(i)).err == 0 {
            human_controller_count += 1;
            if (*copy_pad(i)).trigger != 0 {
                (*d).x1 = 3;
                break;
            }
        }
    }

    if human_controller_count == 0 {
        for i in 0..4usize {
            if (*master_pad(i)).err == 0 && (*copy_pad(i)).trigger != 0 {
                (*d).x1 = 3;
                break;
            }
        }
    }
    if (*d).x1 == 3 {
        ext::lb_8000BA0C((*gobj).hsd_obj.cast(), 1.0f32);
        fn_80177748();
    }
}

/// `pagePrev` (static inline).
pub unsafe fn pagePrev(slot: i32) -> bool {
    let d = data();
    let pd = addr_of_mut!((*d).player_data[slot as usize]);
    if (*pd).page != 0 {
        (*pd).page -= 1;
        (*pd).scroll_offset = 0.0f32;
        ext::fn_80174380();
        true
    } else {
        false
    }
}

/// `pageNext` (static inline).
pub unsafe fn pageNext(slot: i32) -> bool {
    let d = data();
    let pd = addr_of_mut!((*d).player_data[slot as usize]);
    if ((*pd).page as i32) < (*d).num_pages as i32 - 1 {
        (*pd).page += 1;
        (*pd).scroll_offset = 0.0f32;
        ext::fn_80174380();
        true
    } else {
        false
    }
}

const scroll_speed: f32 = 0.2f32;

/// `scrollDown` (static inline).
pub unsafe fn scrollDown(slot: i32, mut amount: f32) -> bool {
    let arg = slot;
    let d = data();
    let pd = addr_of_mut!((*d).player_data[slot as usize]);
    amount *= scroll_speed;

    let list = ext::fn_801748EC(addr_of_mut!(super::gmresult::lbl_803D6878).cast(), (*pd).page as i32, slot | arg);
    let scroll_max = ext::fn_80174A60(list.cast(), slot) - 10;
    if (*pd).scroll_offset < scroll_max as f32 {
        (*pd).scroll_offset += amount;
        if (*pd).scroll_offset > scroll_max as f32 {
            (*pd).scroll_offset = scroll_max as f32;
        }
        true
    } else {
        false
    }
}

/// `scrollUp` (static inline).
pub unsafe fn scrollUp(slot: i32, mut amount: f32) -> bool {
    let d = data();
    let pd = addr_of_mut!((*d).player_data[slot as usize]);
    amount *= scroll_speed;

    if (*pd).scroll_offset > 0.0f32 {
        (*pd).scroll_offset -= amount;
        if (*pd).scroll_offset < 0.0f32 {
            (*pd).scroll_offset = 0.0f32;
        }
        true
    } else {
        false
    }
}

/// Handles result screen controller input for a given player slot: pressing triggers moves to the
/// next/previous page, and moving the stick up/down scrolls within the current page. True if the
/// input was handled and the screen needs updating.
pub unsafe fn fn_80177B7C(slot: i32) -> bool {
    const scroll_deadzone: f32 = 0.3f32;
    let pad = copy_pad(slot as u8 as usize);
    let mut stick_y = (*pad).nml_stickY;
    let abs_stick_y = f32::from_bits(stick_y.to_bits() & !0x8000_0000u32);
    let mut result = false;

    if abs_stick_y < scroll_deadzone {
        stick_y = 0.0f32;
    }
    let trigger = (*pad).trigger;
    if trigger & (PAD_BUTTON_B | PAD_STICK_LEFT) != 0 {
        result = pagePrev(slot);
    } else if trigger & (PAD_BUTTON_A | PAD_STICK_RIGHT) != 0 {
        result = pageNext(slot);
    } else if stick_y < 0.0f32 {
        result = scrollDown(slot, -stick_y);
    } else if stick_y > 0.0f32 {
        result = scrollUp(slot, stick_y);
    }
    result
}

/// fn_80177DD0: a slot's stats page input (a pad: `fn_80177B7C`; no pad: the pages scroll and
/// turn by themselves).
pub unsafe fn fn_80177DD0(slot: i32) -> bool {
    let d = data();
    let unk: *mut c_void = addr_of_mut!(super::gmresult::lbl_803D6878).cast();
    let pd = addr_of_mut!((*d).player_data[slot as usize]);

    let mut result = false;
    if (*copy_pad(slot as usize)).err == 0 {
        return fn_80177B7C(slot);
    }
    let list = ext::fn_801748EC(unk, (*pd).page as i32, slot);
    let mut scroll_max = ext::fn_80174A60(list.cast(), slot) - 10;
    if scroll_max < 0 {
        scroll_max = 0;
    }
    match (*pd).page {
        0 => {
            let old = (*pd).x2;
            (*pd).x2 = old.wrapping_add(1);
            if old > 180 {
                (*pd).x2 = 0;
                (*pd).scroll_offset = 0.0f32;
                (*pd).page = 1;
                result = true;
            } else {
                (*pd).scroll_offset += 0.05f32;
                if (*pd).scroll_offset > scroll_max as f32 {
                    (*pd).scroll_offset = scroll_max as f32;
                } else {
                    result = true;
                }
            }
        }
        1 => {
            let old = (*pd).x2;
            (*pd).x2 = old.wrapping_add(1);
            if old > 1000 {
                if ((*pd).page as i32) < (*d).num_pages as i32 - 1 {
                    (*pd).x2 = 0;
                    (*pd).scroll_offset = 0.0f32;
                    (*pd).page = 2;
                    result = true;
                } else {
                    (*pd).x2 = (*pd).x2.wrapping_sub(1);
                }
            } else {
                (*pd).scroll_offset += 0.05f32;
                if (*pd).scroll_offset > scroll_max as f32 {
                    (*pd).scroll_offset = scroll_max as f32;
                } else {
                    result = true;
                }
            }
        }
        2 => {
            if (*pd).x2 < 1000 {
                (*pd).scroll_offset += 0.05f32;
                if (*pd).scroll_offset > scroll_max as f32 {
                    (*pd).scroll_offset = scroll_max as f32;
                } else {
                    result = true;
                }
                (*pd).x2 = (*pd).x2.wrapping_add(1);
            }
        }
        _ => {}
    }
    result
}

/// fn_80178050: the cards' per-frame input / animation state machine (x0_23 phases).
pub unsafe fn fn_80178050(arg0: *mut HSD_GObj) {
    let d = data();
    let jobj: *mut HSD_JObj = (*arg0).hsd_obj.cast();
    let match_end = ext::fn_80174274();
    let mut var_r24: i32 = 0;

    if ext::lbGetJObjCurrFrame(jobj) >= 50.0f32 && (*d).x0_23() == 0 {
        ext::lb_8000BA0C(jobj, 0.0f32);
        (*d).set_x0_23(1);
        lbl_804D3FC8 = 1;
        for j in 0..4usize {
            if (*match_end).player_standings[j].pkind == 0 {
                lbl_804D3FC8 = 0;
                break;
            }
        }
        if lbl_804D3FC8 != 0 {
            (*d).x3 = 0x14;
        } else {
            (*d).x3 = 0x0A;
        }
    }

    ext::fn_80175D34();

    let phase = (*d).x0_23() as u32;
    if phase != 0 {
        if phase == 1 {
            (*d).set_x0_23(2);
            for k in 0..4usize {
                match (*match_end).player_standings[k].pkind as i32 {
                    2 => {}
                    3 => (*d).player_data[k].set_x0_0(1),
                    0 => ext::fn_80174B4C(d, k as i32),
                    1 => {
                        ext::fn_80174B4C(d, k as i32);
                        (*d).player_data[k].set_x0_0(1);
                    }
                    _ => {}
                }
            }
        }

        for k2 in 0..4usize {
            let slot = (*match_end).player_standings[k2].pkind;
            let pd = addr_of_mut!((*d).player_data[k2]);
            if slot == 0 {
                if (*pd).x0_0() == 0 {
                    if fn_80177B7C(k2 as i32) {
                        ext::fn_80174B4C(d, k2 as i32);
                    }
                    if (*copy_pad(k2)).err != 0 || (*copy_pad(k2)).trigger & PAD_BUTTON_START != 0 {
                        (*pd).set_x0_0(1);
                        ext::fn_80174920(pd);
                        ext::lbDObjSetRateAll(ext::HSD_JObjGetDObj((*pd).jobjs[6]), 1.0f32);
                        ext::lbDObjReqAnimAll(ext::HSD_JObjGetDObj((*pd).jobjs[6]), 50.0f32);
                        ext::HSD_AObjSetRate((*(*pd).jobjs[1]).aobj, 1.0f32);
                        ext::HSD_AObjReqAnim((*(*pd).jobjs[1]).aobj, 50.0f32);
                        ext::HSD_JObjAnimAll((*pd).jobjs[6]);
                        ext::HSD_JObjAnimAll((*pd).jobjs[1]);
                        ext::HSD_JObjSetFlagsAll((*pd).jobjs[0], JOBJ_HIDDEN as u32);
                        ext::HSD_JObjSetFlagsAll((*pd).jobjs[4], JOBJ_HIDDEN as u32);
                        (*pd).set_x0_1(0);
                        (*pd).set_x0_2(0);
                        (*pd).set_x0_3(0);
                        (*pd).set_x0_4(0);
                        ext::fn_80174338();
                    }
                } else if (*copy_pad(k2)).trigger & PAD_BUTTON_START != 0 {
                    ext::fn_80174920(pd);
                    (*pd).set_x0_0(0);
                    ext::fn_80174B4C(d, k2 as i32);
                    ext::lbDObjSetRateAll(ext::HSD_JObjGetDObj((*pd).jobjs[6]), 0.0f32);
                    ext::lbDObjReqAnimAll(ext::HSD_JObjGetDObj((*pd).jobjs[6]), 50.0f32);
                    ext::HSD_AObjSetRate((*(*pd).jobjs[1]).aobj, 0.0f32);
                    ext::HSD_AObjReqAnim((*(*pd).jobjs[1]).aobj, 50.0f32);
                    ext::HSD_JObjAnimAll((*pd).jobjs[6]);
                    ext::HSD_JObjAnimAll((*pd).jobjs[1]);
                    ext::HSD_JObjClearFlagsAll((*pd).jobjs[0], JOBJ_HIDDEN as u32);
                    ext::HSD_JObjClearFlagsAll((*pd).jobjs[4], JOBJ_HIDDEN as u32);
                    ext::fn_8017435C();
                }
            } else if slot == 1 {
                if fn_80177DD0(k2 as i32) {
                    ext::fn_80174B4C(d, k2 as i32);
                }
                if lbl_804D3FC8 != 0 {
                    (*pd).set_x0_0(0);
                    if (*copy_pad(k2)).err == 0 && (*copy_pad(k2)).trigger & PAD_BUTTON_START != 0 {
                        ext::fn_80174338();
                        var_r24 = 1;
                    }
                }
            } else if slot == 3 && lbl_804D3FC8 != 0 && (*copy_pad(k2)).err == 0 && (*copy_pad(k2)).trigger & PAD_BUTTON_START != 0 {
                ext::fn_80174338();
                var_r24 = 1;
            }
        }

        for k3 in 0..4usize {
            if (*match_end).player_standings[k3].pkind != 3 {
                let pd = addr_of_mut!((*d).player_data[k3]);
                if (*pd).x0_1() == 0 {
                    ext::HSD_JObjSetFlagsAll((*pd).jobjs[8], JOBJ_HIDDEN as u32);
                } else {
                    ext::HSD_JObjClearFlagsAll((*pd).jobjs[8], JOBJ_HIDDEN as u32);
                }
                if (*pd).x0_2() == 0 {
                    ext::HSD_JObjSetFlagsAll((*pd).jobjs[9], JOBJ_HIDDEN as u32);
                } else {
                    ext::HSD_JObjClearFlagsAll((*pd).jobjs[9], JOBJ_HIDDEN as u32);
                }
                if (*pd).x0_3() == 0 {
                    ext::HSD_JObjSetFlagsAll((*pd).jobjs[0xA], JOBJ_HIDDEN as u32);
                } else {
                    ext::HSD_JObjClearFlagsAll((*pd).jobjs[0xA], JOBJ_HIDDEN as u32);
                }
                if (*pd).x0_4() == 0 {
                    ext::HSD_JObjSetFlagsAll((*pd).jobjs[0xB], JOBJ_HIDDEN as u32);
                } else {
                    ext::HSD_JObjClearFlagsAll((*pd).jobjs[0xB], JOBJ_HIDDEN as u32);
                }
            }
        }

        let mut all_done = 0;
        for i in 0..4usize {
            if (*d).player_data[i].x0_0() == 0 {
                all_done = 1;
                break;
            }
        }
        if all_done == 0 {
            var_r24 = 1;
        }
        if var_r24 != 0 {
            (*d).x1 = 4;
        }
    }
}

/// The tail of `fn_801785B0` for one match kind: drop three score-banner animations, keep the
/// fourth (`keep`) at frame `|sd_penalty|`, then rewind it and let its texture run.
unsafe fn fn_801785B0_kind(jobj: *mut HSD_JObj, remove: [i32; 3], keep: i32) {
    let mut child: *mut HSD_JObj = null_mut();
    for id in remove {
        ext::lb_80011E24(jobj, &mut child, &[Va::I32(id), Va::I32(-1)]);
        ext::HSD_JObjRemoveAnimAll(child);
    }
    ext::lb_80011E24(jobj, &mut child, &[Va::I32(keep), Va::I32(-1)]);
    let node = child;
    let raw = (*ext::fn_80174274()).sd_penalty as i32;
    let frame_val = if raw < 0 { -raw } else { raw };
    let fv = frame_val as f32;
    foreach_rate(node, 0.0);
    foreach_frame(node, fv as f64);
    ext::HSD_JObjAnimAll(node);
    foreach_frame(node, 0.0);
    foreach_rate(node, 1.0);
    ext::mn_8022F3D8(node, 1, TOBJ_MASK);
}

/// fn_801785B0: the end-of-match banner (which of the four score words shows).
pub unsafe fn fn_801785B0(gobj: *mut HSD_GObj) {
    let jobj: *mut HSD_JObj = (*gobj).hsd_obj.cast();
    let match_end = ext::fn_80174274();
    let mode = (*match_end).match_kind;

    if mode == 2 {
        fn_801785B0_kind(jobj, [0x15, 0x11, 0x10], 0xC);
    } else if mode == 1 {
        fn_801785B0_kind(jobj, [0x15, 0xC, 0x10], 0x11);
    } else if mode == 3 {
        fn_801785B0_kind(jobj, [0x11, 0xC, 0x15], 0x10);
    } else {
        fn_801785B0_kind(jobj, [0x11, 0xC, 0x10], 0x15);
    }
}

/// The winner-name banner's frame range for texture `tex_id` (shared by `fn_80178BB4`'s branches).
unsafe fn banner_frames(aobj: *mut HSD_AObj, tex_id: i32) {
    if tex_id < 0x19 {
        ext::HSD_AObjSetCurrentFrame(aobj, 0.0f32);
        ext::HSD_AObjSetEndFrame(aobj, 29.0f32);
    } else {
        let start = (30.0f32 * (tex_id - 0xB4) as f32) as i32;
        ext::HSD_AObjSetCurrentFrame(aobj, start as f32);
        ext::HSD_AObjSetEndFrame(aobj, (30.0f32 + start as f32) - 1.0f32);
    }
}

/// `fn_80178BB4_init_players` (static inline): set every card's pictures; counts KOs.
pub unsafe fn fn_80178BB4_init_players(d: *mut ResultsData, match_end: *mut MatchEnd, i: *mut i32, ko_count: *mut i32) {
    while *i < 6 {
        let ii = *i as usize;
        let pd = addr_of_mut!((*d).player_data[ii]);
        let ps = addr_of_mut!((*match_end).player_standings[ii]);
        ext::HSD_JObjSetFlagsAll((*pd).jobjs[0], JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlagsAll((*pd).jobjs[4], JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlagsAll((*pd).jobjs[8], JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlagsAll((*pd).jobjs[9], JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlagsAll((*pd).jobjs[10], JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlagsAll((*pd).jobjs[11], JOBJ_HIDDEN as u32);
        ext::lbDObjSetRateAll(ext::HSD_JObjGetDObj((*pd).jobjs[6]), 0.0f32);

        let slot_type = (*ps).pkind;

        if slot_type == 0 {
            *ko_count += (*ps).xE as i32;
        }

        if slot_type != 3 {
            let ckind = (*ps).ckind as i32;
            let cid = (*ps).ftkind as i32;
            let is_big_loser = (*ps).is_big_loser;

            if ext::gm_WasMatchCanceled((*match_end).outcome) == 0 && (*match_end).is_teams == 0 && is_big_loser as i32 == 0 {
                let d2 = data();
                let tobj = (*(*(*(*(*d2).x30).u.dobj).next).mobj).tobj;
                let aobj = (*tobj).aobj;
                let mut tex_id = gmRst_NameFrame(tobj, ckind, cid);
                let drawn = tex_id < 0;
                if drawn {
                    tex_id = 0; // any frame: gmRst_DrawName points them all at the name
                }
                ext::HSD_TObjReqAnim(tobj, tex_id as f32);
                ext::HSD_TObjAnim(tobj);
                if drawn {
                    gmRst_DrawName(tobj, ckind, 1);
                }
                // m-ex RstFixYellowName: an m-ex name past frame 25 is still a normal winner,
                // not one of retail's special-banner frames.
                if tex_id < 0x19 || gmRst_IsMex(ckind) {
                    ext::HSD_AObjSetCurrentFrame(aobj, 0.0f32);
                    ext::HSD_AObjSetEndFrame(aobj, 29.0f32);
                } else {
                    let start = (30.0f32 * (tex_id - 0xB4) as f32) as i32;
                    ext::HSD_AObjSetCurrentFrame(aobj, start as f32);
                    ext::HSD_AObjSetEndFrame(aobj, (30.0f32 + start as f32) - 1.0f32);
                }
                ext::mn_8022F3D8((*d2).x30, 1, TOBJ_MASK);
            }

            {
                let name_jobj = (*pd).jobjs[5];
                let name_tobj = (*(*(*name_jobj).u.dobj).mobj).tobj;
                let name_frame = gmRst_NameFrame(name_tobj, ckind, cid);
                ext::fn_80174FD0(name_jobj, if name_frame >= 0 { name_frame } else { 0 });
                if name_frame < 0 {
                    gmRst_DrawName(name_tobj, ckind, 0);
                }
            }

            {
                let rank_val = ext::gm_80160854(*i as u8, (*ps).team, ((*match_end).is_teams == 1) as u8, (*ps).pkind);
                let rank_aobj = (*(*(*(*pd).jobjs[2]).u.dobj).mobj).aobj;
                ext::HSD_AObjSetCurrentFrame(rank_aobj, 1.0f32 + (rank_val as u8) as f32);
                ext::HSD_AObjSetRate(rank_aobj, 0.0f32);
            }

            if (*match_end).match_kind != 3 {
                let mut taunt_frame = ext::gm_80168B34((*ps).ckind as CharacterKind, (*ps).ftkind as i32, (*ps).anon_3.anon_0.x3_b0() as i32);
                let taunt_jobj = (*pd).jobjs[7];
                // m-ex: the stock icon from IfAll's Stc_icns (the second DObj, as m-ex attaches
                // it); none rather than another fighter's without one.
                let mut no_stock = false;
                let geno_stock = ext::Geno_DefineBaseCK(ckind) >= 0;
                let geno_base = ext::Geno_DefineArtBaseCK(ckind);
                if geno_stock {
                    // Establish the retail fallback before trying a package.
                    taunt_frame = if geno_base >= 0 {
                        ext::gm_80168B34(geno_base as CharacterKind, 0, (*ps).anon_3.anon_0.x3_b0() as i32)
                    } else {
                        0.0f32
                    };
                } else if gmRst_IsMex(ckind) {
                    let mex = ext::gm_MexStockFrame((*ps).ftkind as i32, (*ps).anon_3.anon_0.x3_b0() as i32);
                    if mex >= 0.0f32 && gmRst_MexAttach(taunt_jobj, 1, true) {
                        taunt_frame = mex;
                    } else {
                        no_stock = true;
                    }
                }
                foreach_rate(taunt_jobj, 0.0);
                foreach_frame(taunt_jobj, taunt_frame as f64);
                ext::HSD_JObjAnimAll(taunt_jobj);
                ext::HSD_AObjSetRate((*(*pd).jobjs[7]).aobj, 1.0f32);
                ext::HSD_AObjSetCurrentFrame((*(*pd).jobjs[7]).aobj, 0.0f32);
                if geno_stock && !gmRst_GenoApplyArt(gmRst_GenoTObjAt(taunt_jobj, 1), ckind, 2, (*ps).anon_3.anon_0.x3_b0() as i32) {
                    if geno_base >= 0 {
                        ext::OSReport(c"geno: results stock of ck %d: retail ck %d\n".as_ptr(), &[Va::I32(ckind), Va::I32(geno_base)]);
                    } else {
                        no_stock = !gmRst_GenoLetters(gmRst_GenoTObjAt(taunt_jobj, 1), ckind);
                    }
                }
                if no_stock {
                    ext::HSD_JObjSetFlagsAll(taunt_jobj, JOBJ_HIDDEN as u32);
                }
            } else {
                let taunt_jobj = (*pd).jobjs[7];
                foreach_rate(taunt_jobj, 0.0);
                foreach_frame(taunt_jobj, 0.0);
                ext::HSD_JObjAnimAll(taunt_jobj);
                ext::HSD_JObjSetFlagsAll((*pd).jobjs[7], JOBJ_HIDDEN as u32);
            }

            ext::HSD_JObjRemoveAnimAll((*pd).jobjs[3]);
            ext::HSD_JObjSetFlagsAll((*pd).jobjs[3], JOBJ_HIDDEN as u32);
        } else {
            ext::HSD_JObjSetFlagsAll((*pd).jobjs[1], JOBJ_HIDDEN as u32);
            ext::HSD_JObjRemoveAnimAll((*pd).jobjs[1]);
            ext::HSD_JObjSetFlagsAll((*pd).jobjs[5], JOBJ_HIDDEN as u32);
            ext::HSD_JObjSetFlagsAll((*pd).jobjs[7], JOBJ_HIDDEN as u32);
        }
        *i += 1;
    }
}

/// fn_80178BB4: results card setup (the scene's first frame).
pub unsafe fn fn_80178BB4(gobj: *mut HSD_GObj) {
    let d = data();
    let match_end = ext::fn_80174274();
    let mut ko_count: i32 = 0;

    ext::fn_80179854();
    ext::fn_80175DC8(gobj);
    ext::fn_80175C5C();

    if ext::gm_WasMatchCanceled((*match_end).outcome) != 0 {
        let anim_n: i32 = 0xB8;
        let tobj = (*(*(*(*(*d).x30).u.dobj).next).mobj).tobj;
        let aobj = (*tobj).aobj;
        ext::HSD_TObjReqAnim(tobj, anim_n as f32);
        ext::HSD_TObjAnim((*(*(*(*(*d).x30).u.dobj).next).mobj).tobj);
        let start = (30.0f32 * (anim_n - 0xB4) as f32) as i32;
        ext::HSD_AObjSetCurrentFrame(aobj, start as f32);
        ext::HSD_AObjSetEndFrame(aobj, (30.0f32 + start as f32) - 1.0f32);
        ext::mn_8022F3D8((*d).x30, 1, TOBJ_MASK);
    } else if (*match_end).is_teams == 1 {
        let tex_id: i32 = (*d).x5 as i32 + 0xB5;
        let tobj = (*(*(*(*(*d).x30).u.dobj).next).mobj).tobj;
        let aobj = (*tobj).aobj;
        ext::HSD_TObjReqAnim(tobj, tex_id as f32);
        ext::HSD_TObjAnim((*(*(*(*(*d).x30).u.dobj).next).mobj).tobj);
        banner_frames(aobj, tex_id);
        ext::mn_8022F3D8((*d).x30, 1, TOBJ_MASK);
    }

    let mut i: i32 = 0;
    fn_80178BB4_init_players(d, match_end, &mut i, &mut ko_count);

    if ko_count > 0 {
        ext::un_802FF128(0x6C, 0x78, ko_count, 5);
    }

    (*d).x1 = 1;
}

/// fn_801791E4: should the results scene leave the card state (a start press on a cancelled
/// match, or 160 frames in).
pub unsafe fn fn_801791E4() -> bool {
    let d = data();
    let end = ext::fn_80174274();

    if ext::gm_WasMatchCanceled((*end).outcome) != 0 {
        for i in 0..4usize {
            if (*end).player_standings[i].pkind == Gm_PKind_Human as u8
                && (*master_pad(i)).err == 0
                && (*copy_pad(i)).trigger & PAD_BUTTON_START != 0
            {
                return true;
            }
        }
    }

    if (*d).x8 as u32 == 0xA0 {
        return true;
    }
    false
}

/// `fn_80179350_inline` (static inline).
pub unsafe fn fn_80179350_inline() -> i32 {
    fn_801791E4() as i32
}

/// `fn_80179350_update` (static inline): sounds by frame, then the state machine of `x1`.
pub unsafe fn fn_80179350_update(d: *mut ResultsData, match_end: *mut MatchEnd, arg0: *mut HSD_GObj) {
    if (*d).x8 as u32 == 0 && (*d).x0_4() != 0 {
        ext::gm_SetDbPauseFlag(0);
    }

    if (*d).x8 as u32 == 0xA2 {
        if ext::gm_WasMatchCanceled((*match_end).outcome) == 0 {
            ext::lbAudioAx_800237A8(0xB5, 0x7F, 0x40);
        }
    } else if (*d).x8 as u32 == 0x2 {
        if ext::gm_WasMatchCanceled((*match_end).outcome) == 0 {
            ext::lbAudioAx_800237A8(0xC355, 0x7F, 0x40);
            ext::lbAudioAx_800237A8(0x144, 0x7F, 0x40);
        } else {
            ext::lbAudioAx_800237A8(0x148, 0x7F, 0x40);
        }
    } else if (*d).x8 as u32 == 0x9A {
        if ext::gm_WasMatchCanceled((*match_end).outcome) != 0 {
            ext::lbAudioAx_800237A8(0xC350, 0x7F, 0x40);
        } else {
            let w = (*d).x6 as usize;
            ext::fn_80168E54(
                (*match_end).player_standings[w].ckind,
                (*match_end).player_standings[w].ftkind,
                (*match_end).player_standings[w].team,
                ((*match_end).is_teams == 1) as u8,
            );
        }
    }

    if (*d).x1 < 1 {
        if fn_80179350_inline() != 0 {
            fn_80178BB4(arg0);
        }
    } else {
        ext::HSD_JObjAnimAll((*arg0).hsd_obj.cast());
        match (*d).x1 {
            1 => {
                let d2 = data();
                let jobj: *mut HSD_JObj = (*arg0).hsd_obj.cast();
                let frame = ext::lbGetJObjCurrFrame(jobj);
                if frame >= 10.0f32 && (*d2).x0_1() == 0 {
                    fn_80177748();
                    (*d2).set_x0_1(1);
                }
                if frame >= 40.0f32 {
                    ext::lb_8000BA0C(jobj, 0.0f32);
                    (*d2).x1 = 2;
                }
            }
            2 => fn_80177920(arg0),
            3 => fn_80178050(arg0),
            4 => {
                if (*d).x3 != 0 {
                    (*d).x3 = (*d).x3.wrapping_sub(1);
                } else {
                    ext::gm_801A4B60();
                }
            }
            _ => {}
        }
    }
}

/// fn_80179350: the results scene's per-frame callback.
pub unsafe fn fn_80179350(arg0: *mut HSD_GObj) {
    let d = data();
    let match_end = ext::fn_80174274();
    for i in 0..6usize {
        if !(*d).player_data[i].jobjs[12].is_null() {
            ext::lb_8000B1CC((*d).player_data[i].jobjs[12], null_mut(), addr_of_mut!((*d).player_data[i].stats_position));
        }
    }
    fn_80179350_update(d, match_end, arg0);

    if ((*d).x8 as u32) < u32::MAX {
        (*d).x8 = (*d).x8.wrapping_add(1);
    }
}

/// fn_801795D4: how many players placed other than last (0 when nobody did).
pub unsafe fn fn_801795D4() -> i32 {
    let match_end = addr_of_mut!((*addr_of_mut!(lbl_8046E3AC)).match_end);
    let mut count = 0;

    for i in 0..4usize {
        let lookup: i32 = if (*match_end).is_teams == 0 {
            (*match_end).player_standings[i].is_big_loser as i32
        } else {
            let idx = (*match_end).player_standings[i].team as usize;
            (*match_end).team_standings[idx].is_big_loser as i32
        };

        if (*match_end).player_standings[i].pkind != 3 && lookup == 0 {
            count += 1;
        }
    }

    if count != 0 {
        return count - 1;
    }
    count
}

/// fn_801796F0: the placing index of player `arg0` among those who placed.
pub unsafe fn fn_801796F0(arg0: i32) -> i32 {
    let match_end = addr_of_mut!((*addr_of_mut!(lbl_8046E3AC)).match_end);
    let mut count = 0;

    for i in 0..4usize {
        let lookup: i32 = if (*match_end).is_teams == 0 {
            (*match_end).player_standings[i].is_big_loser as i32
        } else {
            let idx = (*match_end).player_standings[i].team as usize;
            (*match_end).team_standings[idx].is_big_loser as i32
        };

        if (*match_end).player_standings[i].pkind != 3 && lookup == 0 {
            if arg0 == i as i32 {
                return count;
            }
            count += 1;
        }
    }
    count
}

/// gmresult.c `gm_WasMatchCanceled` with the C `bool` (a 32-bit int) result.
unsafe fn was_match_canceled(outcome: u8) -> i32 {
    super::gmresult::gm_WasMatchCanceled(outcome) as i32
}
