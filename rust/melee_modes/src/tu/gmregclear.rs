//! Port of src/melee/gm/gmregclear.c: the Regular Match / Classic "clear" results screen (score
//! tallies, bonus text, camera/light/model setup and the per-frame proc).
//!
//! All screen state lives in one static (`lbl_80472D28`). The C views it through two structs whose
//! fields alias by GameCube offset (4-byte pointers); here that is ONE struct, [`RegClear`], with
//! every name of both views (GC order, pointer runs contiguous), which is also what the oracle's
//! patched C uses (gen/tu/patches/gmregclear.py). The `s32*` cache that `fn_8017F47C` keeps beside
//! the 8 text-pointer slots is `xA4`.
//!
//! Fused multiply-adds (the shipping build's `fmuladd`): `fn_8017F2A4` row y, `fn_8017FE54`
//! event scale.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};

use melee_core::fp::fmadds;
use melee_types::{
    DynamicModelDesc, GObj_RenderFunc, Gm_PKind, HSD_AObj, HSD_Archive, HSD_CObj, HSD_CObjDesc, HSD_GObj, HSD_GObjEvent,
    HSD_GObjProc, HSD_ImageDesc, HSD_JObj, HSD_Joint, HSD_LObj, HSD_PadStatus, HSD_Text, LightList, MatchEnd, StartMeleeRules, Vec3,
    VsSceneController, Gm_PKind_Human, JOBJ_HIDDEN, OUTCOME_UNK_1P_BONUS_STAGE_END,
};

use super::gmregtyfall::{HSD_GObj_CameraKind, HSD_GObj_JObjKind, HSD_GObj_LightKind};
use super::hsd::{self, get_jobj};
use crate::oracle::Va;

pub const PAD_BUTTON_DOWN: u64 = 1 << 2;
pub const PAD_BUTTON_UP: u64 = 1 << 3;
pub const PAD_STICK_UP: u64 = 1 << 16;
pub const PAD_STICK_DOWN: u64 = 1 << 17;
pub const HSD_PAD_A: u32 = 1 << 8;
const GX_TF_RGB5A3: u32 = 5;

pub mod ext {
    use super::*;
    crate::oext_fns! {
        fn Camera_8002F7AC(a0: i8) -> ();
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32) -> ();
        fn GObj_SetupGXLinkMax(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, priority: u32) -> ();
        fn Ground_801C1DD4() -> i32 = crate::tu::adapt::Ground_801C1DD4__f2fb49;
        fn Ground_801C1DE4(a0: *mut i32, a1: *mut i32) -> () = crate::tu::adapt::Ground_801C1DE4__3e6526;
        fn HSD_AObjSetCurrentFrame(aobj: *mut HSD_AObj, frame: f32) -> () = crate::tu::adapt::HSD_AObjSetCurrentFrame__64f5bd;
        fn HSD_AObjSetRate(aobj: *mut HSD_AObj, rate: f32) -> () = crate::tu::adapt::HSD_AObjSetRate__1c2c90;
        fn HSD_CObjLoadDesc(desc: *mut HSD_CObjDesc) -> *mut HSD_CObj = crate::tu::adapt::HSD_CObjLoadDesc__66a547;
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void) -> ();
        fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) -> ();
        fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32) -> () = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn HSD_GObj_LObjCallback(gobj: *mut HSD_GObj, unused: i32) -> () = crate::tu::adapt::HSD_GObj_LObjCallback__aaa236;
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj) -> ();
        fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, flags: u32) -> ();
        fn HSD_JObjGetFlags(jobj: *mut HSD_JObj) -> u32;
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32) -> ();
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32) -> ();
        fn HSD_Randi(max_val: i32) -> i32;
        fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A5ACC__27fcb1;
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) -> () = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
        fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 = crate::tu::adapt::HSD_SisLib_803A611C__de78e8;
        fn HSD_SisLib_803A62A0(a0: i32, a1: *const i8, a2: *const i8) -> () = crate::tu::adapt::HSD_SisLib_803A62A0__58f49d;
        fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) -> () = crate::tu::adapt::HSD_SisLib_803A6368__2e8f0d;
        fn HSD_SisLib_803A6754(arg0: i32, arg1: i32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A6754__02fb41;
        fn HSD_SisLib_803A7548(a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32) -> () = crate::tu::adapt::HSD_SisLib_803A7548__d5953f;
        fn Player_GetPlayerId(slot: i32) -> i32 = crate::tu::adapt::Player_GetPlayerId__aaadbd;
        fn Player_GetPlayerSlotType(slot: i32) -> Gm_PKind = crate::tu::adapt::Player_GetPlayerSlotType__2d05e5;
        fn fn_801689E4(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32) -> () = crate::tu::adapt::fn_801689E4__f8906a;
        fn fn_80168A6C(a0: *mut c_void, a1: *mut c_void, a2: i32) -> ();
        fn fn_80168F2C(a0: i8) -> () = crate::tu::adapt::fn_80168F2C__69f670;
        fn fn_80168F7C() -> () = crate::tu::adapt::fn_80168F7C__c9139a;
        fn fn_8016F140(a0: i32) -> () = crate::tu::adapt::fn_8016F140__cc064d;
        fn fn_8016F160(a0: i32, a1: i32) -> i32 = crate::tu::adapt::fn_8016F160__106c7f;
        fn fn_8016F180(a0: i32) -> i32 = crate::tu::adapt::fn_8016F180__e8a49d;
        fn fn_8016F1F0(idx: i32) -> i32 = crate::tu::adapt::fn_8016F1F0__e5a15c;
        fn fn_8016F344(a0: *mut MatchEnd) -> () = crate::tu::adapt::fn_8016F344__d76861;
        fn fn_8016F39C(a0: *mut *mut HSD_Text, a1: *mut c_void, a2: u8, a3: u16, a4: u8, a5: u8) -> i32 = crate::tu::adapt::fn_8016F39C__3085d5;
        fn fn_8016F548(a0: *mut c_void, a1: u16, a2: u8, a3: u8) -> i32 = crate::tu::adapt::fn_8016F548__297f1e;
        fn fn_8016F740(a0: *mut c_void, a1: u16, a2: u8, a3: u8) -> i32 = crate::tu::adapt::fn_8016F740__946ab3;
        fn fn_8016F870(a0: *mut c_void, a1: u16, a2: u8, a3: u8) -> i32 = crate::tu::adapt::fn_8016F870__a28fe9;
        fn fn_8016F9A8(a0: *mut c_void, a1: u16, a2: u8, a3: u8) -> i32 = crate::tu::adapt::fn_8016F9A8__0f140d;
        fn fn_8016FAD4(a0: *mut MatchEnd, a1: i32, a2: i32, a3: u8) -> i32 = crate::tu::adapt::fn_8016FAD4__4d2bbb;
        fn fn_8016FFD4(a0: *mut MatchEnd, a1: i32, a2: u8) -> i32 = crate::tu::adapt::fn_8016FFD4__ba90b1;
        fn fn_801701C0(a0: *mut MatchEnd, a1: i32, a2: i32) -> i32 = crate::tu::adapt::fn_801701C0__332d21;
        fn fn_80171A88() -> i32 = crate::tu::adapt::fn_80171A88__4c9145;
        fn gmVs_GetSceneController() -> *mut VsSceneController = crate::tu::adapt::gmVs_GetSceneController__400cec;
        fn gm_8016B774() -> *mut MatchEnd = crate::tu::adapt::gm_8016B774__ba5c36;
        fn gm_801A36C0(a0: u8) -> u64 = crate::tu::adapt::gm_801A36C0__4faf6f;
        fn gm_GetButtonsTriggered(a0: u8) -> u64 = crate::tu::adapt::gm_GetButtonsTriggered__dca604;
        fn gm_GetStartMeleeRules() -> *mut StartMeleeRules = crate::tu::adapt::gm_GetStartMeleeRules__b73473;
        fn grPushOn_80219204(arg0: i32, out1: *mut i32, out2: *mut i32) -> () = crate::tu::adapt::grPushOn_80219204__ac5d9d;
        fn lbAudioAx_80023F28(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_80023F28__582fc2;
        fn lbLang_IsSavedLanguageUS() -> i32 = crate::tu::adapt::lbLang_IsSavedLanguageUS__90e41e;
        fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) -> () = crate::tu::adapt::lb_8000B1CC__69851d;
        fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj = crate::tu::adapt::lb_80011AC4__8bc38c;
        fn lb_8001204C(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: *const u16, a3: i32) -> i32 = crate::tu::adapt::lb_8001204C__7ca2e9;
        fn lb_800121FC(image_desc: *mut HSD_ImageDesc, width: i32, height: i32, format: u32, entry_num: i16) -> *mut HSD_ImageDesc = crate::tu::adapt::lb_800121FC__e48dc8;
        fn lb_800122C8(image_desc: *mut HSD_ImageDesc, origx: u16, origy: u16, clear: i32) -> () = crate::tu::adapt::lb_800122C8__a924f6;
        fn lb_800138CC(gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) -> () = crate::tu::adapt::lb_800138CC__37f797;
        fn lb_800138D8(gobj: *mut HSD_GObj, arg1: i8) -> () = crate::tu::adapt::lb_800138D8__383f48;
        fn lb_800138EC(a0: *mut HSD_ImageDesc, a1: GObj_RenderFunc, a2: i32, a3: i8, a4: f32, a5: f32, a6: f32, a7: f32) -> *mut HSD_GObj;
        fn memzero(mem: *mut c_void, size: isize) -> ();
        fn un_802FF128(arg0: i32, arg1: i32, arg2: i32, arg3: i32) -> () = crate::tu::adapt::un_802FF128__59ac96;
    }
    crate::ext_va! {
        /// sislib.c: `(text, x, y, fmt, ...)`
        fn HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const i8) va 3 -> i32;
        /// sislib.c: `(text, idx, fmt, ...)`
        fn HSD_SisLib_803A70A0(text: *mut HSD_Text, idx: i32, fmt: *const i8) va 2 -> i32;
        /// os: `(file, line, fmt, ...)`
        fn OSPanic(file: *const i8, line: i32, fmt: *const i8) va 2 -> ();
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
    /// lb_00B0.c, variadic in C: `(jobj, out, a, b)`; fixed arguments logged only.
    pub unsafe fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, _rest: &[i32]) -> i32 { unsafe { melee_runtime::tu::lbspdisplay::lb_80011E24(a0 as _, a1 as _, _rest as _) as _ } }
}

/// The screen's state: `struct lbl_80472D28_t` and `fn_8017FA1C_arg` merged (see module docs).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RegClear {
    pub x0: *mut HSD_GObj,
    /// x4..x28: ten consecutive JObj pointers (`lb_8001204C` fills them as an array)
    pub x4: *mut HSD_JObj,
    pub x8: *mut HSD_JObj,
    pub xC: *mut HSD_JObj,
    pub x10: *mut HSD_JObj,
    pub x14: *mut HSD_JObj,
    pub x18: *mut HSD_JObj,
    pub x1C: *mut HSD_JObj,
    pub x20: *mut HSD_JObj,
    pub x24: *mut HSD_JObj,
    pub x28: *mut HSD_JObj,
    pub x2C: *mut HSD_GObj,
    pub x30: HSD_ImageDesc,
    pub x48: *mut HSD_Archive,
    pub x4C: DynamicModelDesc,
    pub x5C: *mut c_void,
    pub x60: *mut c_void,
    pub x6C: *mut HSD_Text,
    pub x70: *mut HSD_Text,
    pub x74: *mut HSD_Text,
    pub x78: *mut HSD_Text,
    pub x7C: *mut HSD_Text,
    pub x80: *mut HSD_Text,
    /// x84..xA0: the eight text slots `fn_8017F2A4` fills (`HSD_Text*` array base = &x84)
    pub x84: *mut HSD_Text,
    pub x88: *mut HSD_Text,
    pub x8C: *mut HSD_Text,
    pub x90: *mut HSD_Text,
    pub x94: *mut HSD_Text,
    pub x98: *mut HSD_Text,
    pub x9C: *mut HSD_Text,
    pub xA0: *mut HSD_Text,
    /// the last value shown per text slot (`p[8]` of `fn_8017F47C`)
    pub xA4: [i32; 8],
    pub xC0: u16,
    pub pad_C2: u16,
    pub xC4: u32,
    pub xC8: u8,
    pub pad_C9: [u8; 3],
    pub xCC: i32,
    pub xD0: i32,
    pub xD4: i32,
    pub xD8: i32,
    pub xDC: i32,
    pub xE0: i32,
    pub xE4: i32,
    pub xE8: i32,
    pub xEC: i32,
    pub xF0: i32,
    pub xF4: i32,
    pub xF8: i32,
    pub xFC: i32,
    pub x100: i32,
    pub x104: i32,
    pub x108: i16,
    pub x10A: i16,
    pub x10C: f32,
    pub x110: u32,
    pub x114: u8,
    pub x115: u8,
    pub x116: u8,
    pub x117: u8,
    pub x118: u8,
    pub x119: u8,
    pub x11A: u8,
    pub x11B: u8,
    pub x11C: u8,
    pub x11D: u8,
    pub x11E: u8,
    pub x11F: u8,
}

/// `RegClearEv`: the per-event user data of `fn_8017FE54` (pad, image desc, float).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RegClearEv {
    pub pad_0: [u8; 0x1C],
    pub x1C: *mut HSD_ImageDesc,
    pub x20: f32,
}

const ZERO_STATE: RegClear = unsafe { core::mem::zeroed() };

pub use melee_runtime::tu::controller::HSD_PadMasterStatus;

crate::globals! {
    /// `static struct lbl_80472D28_t lbl_80472D28`
    rw pub lbl_80472D28: RegClear = ZERO_STATE;
    /// `s32 lbl_804D65C0`
    rw pub lbl_804D65C0: i32 = 0;
    /// `u16 lbl_803D8B88[]`: joint ids `lb_8001204C` looks up
    ro pub lbl_803D8B88: [u16; 10] = [0x18, 0x16, 0x12, 0x3, 0x5, 0x4, 0x6, 0x1a, 0x19, 0x7];
    /// `static const Vec3 lbl_803B7C18`
    ro pub lbl_803B7C18: Vec3 = Vec3 { x: -41.0, y: -0.25, z: 0.0 };
}

crate::globals_ext! {
    rw HSD_GObj_CameraKind = super::gmregtyfall::HSD_GObj_CameraKind: u8;
    rw HSD_GObj_JObjKind = super::gmregtyfall::HSD_GObj_JObjKind: u8;
    rw HSD_GObj_LightKind = super::gmregtyfall::HSD_GObj_LightKind: i8;
}

#[inline]
unsafe fn st() -> *mut RegClear {
    addr_of_mut!(lbl_80472D28)
}

/// fn_8017F008: the bonus-mask byte of the current match kind.
pub unsafe fn fn_8017F008() -> i32 {
    melee_core::ct!("fn_8017F008");
    let r3 = ext::fn_80171A88();
    if r3 == 3 {
        4
    } else if r3 == 1 {
        1
    } else if r3 == 2 {
        8
    } else if r3 == 7 {
        0x20
    } else if r3 == 6 {
        0x80
    } else if r3 == 8 {
        0x40
    } else if r3 == 4 {
        0x10
    } else {
        0xFD
    }
}

/// `data->x118 != 0 || (data->x11A != 0 && data->x11B == 0) || p2->start.timer_counts_up` negated:
/// whether the time bonus counts.
unsafe fn time_bonus_on(state: *const RegClear, p2: *const VsSceneController) -> bool {
    !((*state).x118 != 0 || ((*state).x11A != 0 && (*state).x11B == 0) || (*p2).start.timer_counts_up() != 0)
}

/// fn_8017F09C: the time bonus.
pub unsafe fn fn_8017F09C() -> i32 {
    melee_core::ct!("fn_8017F09C");
    let data = st();
    let p = ext::gmVs_GetSceneController();
    if (*p).start.x4_6() != 0 {
        let p2 = ext::gmVs_GetSceneController();
        let flag = if time_bonus_on(data, p2) { 1 } else { 0 };
        if flag != 0 {
            let product = ((*p).state.timer_seconds as i32).wrapping_mul((*data).x108 as i32);
            let mut ret = product;
            if product < 0 {
                ret = 0;
            }
            return ret;
        }
    }
    0
}

/// fn_8017F14C: the KO bonus; `arg0` is the MatchEnd (`s32` at GC +0x98).
pub unsafe fn fn_8017F14C(arg0: *mut c_void) -> i32 {
    melee_core::ct!("fn_8017F14C");
    let data = st();
    if (*ext::gmVs_GetSceneController()).start.x4_7() != 0 {
        let val = *(arg0 as *const u8).add(0x98).cast::<i32>();
        let mut ret = val;
        if val > 9999 {
            ret = 9999;
        }
        return ret.wrapping_mul((*data).x10A as i32);
    }
    0
}

/// fn_8017F1B8: award the bonuses the player earned; returns the bonus total.
pub unsafe fn fn_8017F1B8() -> i32 {
    melee_core::ct!("fn_8017F1B8");
    for i in 0..256 {
        if ext::fn_8016F1F0(i) < 0xD7 {
            let mask = fn_8017F008();
            let output = ext::fn_8016F180(i) as u8 as i32;
            if output & mask != 0 && ext::fn_8016F160(0, i) as u32 != 0 {
                ext::fn_8016F140(i);
            }
        } else {
            let mask = fn_8017F008();
            let output = ext::fn_8016F180(i) as u8 as i32;
            if output & mask != 0 && ext::fn_801701C0(ext::gm_8016B774(), 0, i) as u32 != 0 {
                ext::fn_8016F140(i);
            }
        }
    }

    let mask = fn_8017F008() as u8;
    ext::fn_8016FFD4(ext::gm_8016B774(), mask as i32, 0)
}

/// fn_8017F294.
pub unsafe fn fn_8017F294() -> i32 {
    melee_core::ct!("fn_8017F294");
    (*st()).x104
}

/// fn_8017F2A4: build the 8 score texts at (`farg0`, `farg1`).
pub unsafe fn fn_8017F2A4(arg0: *mut *mut HSD_Text, farg0: f32, farg1: f32) -> i32 {
    melee_core::ct!("fn_8017F2A4");
    let temp = ext::HSD_SisLib_803A611C(3, null_mut(), 9, 0xD, 0, 0x14, 0, 0x13);
    if ext::lbLang_IsSavedLanguageUS() != 0 {
        ext::HSD_SisLib_803A62A0(3, c"SdDec.usd".as_ptr(), c"SIS_DecisionData".as_ptr());
    } else {
        ext::HSD_SisLib_803A62A0(3, c"SdDec.dat".as_ptr(), c"SIS_DecisionData".as_ptr());
    }
    *arg0 = ext::HSD_SisLib_803A6754(3, temp);
    let mut ptr = arg0;

    (**arg0).default_kerning = 1;
    let mut text = *arg0;
    (*text).x34.x = 0.56f32;
    (*text).x34.y = 0.6f32;
    (**arg0).default_alignment = 2;
    let x_end = 281.0f32 + farg0;
    for i in 0..7 {
        let y = fmadds(0.6f32, 32.0f32 * (i as f32), farg1);
        *ptr.add(1) = ext::HSD_SisLib_803A5ACC(3, 0, farg0, y, 0.0f32, 2.0f32, 2.0f32);
        text = *ptr.add(1);
        (*text).x34.x = 0.56f32;
        (*text).x34.y = 0.6f32;
        (**ptr.add(1)).default_kerning = 1;
        ext::HSD_SisLib_803A6B98(*arg0, x_end, -12.0f32 + y, c"          ".as_ptr(), &[]);
        ptr = ptr.add(1);
    }
    ext::HSD_SisLib_803A6B98(*arg0, x_end, 9.0f32 + (134.40001f32 + farg1), c"               ".as_ptr(), &[])
}

/// fn_8017F47C: refresh the score texts for the ranking window starting at `arg1`.
pub unsafe fn fn_8017F47C(arg0: *mut *mut HSD_Text, arg1: i32) -> i32 {
    melee_core::ct!("fn_8017F47C");
    let mut entry = arg1;
    let mut prev_idx = -999;
    let mut mask = fn_8017F008() as u8;
    ext::fn_8016F39C(arg0.add(1), ext::gm_8016B774().cast(), 7, arg1 as u16, mask, 0);

    let cache = arg0.add(8) as *mut i32;
    let mut i = 0usize;
    while i < 7 {
        mask = fn_8017F008() as u8;
        let idx = ext::fn_8016F548(ext::gm_8016B774().cast(), entry as u16, mask, 0);
        mask = fn_8017F008() as u8;
        let val = ext::fn_8016FAD4(ext::gm_8016B774(), idx as u32 as i32, mask as i32, 0);

        if prev_idx == idx || idx < 0 {
            break;
        }

        if *cache.add(i) != val {
            if val < 0 {
                ext::HSD_SisLib_803A70A0(*arg0, i as i32, c"%s%d".as_ptr(), &[Va::P(c"－".as_ptr().cast()), Va::I32(val.wrapping_neg())]);
            } else {
                ext::HSD_SisLib_803A70A0(*arg0, i as i32, c"%d".as_ptr(), &[Va::I32(val)]);
            }
            *cache.add(i) = val;
        }

        prev_idx = idx;
        entry = idx + 1;
        i += 1;
    }

    mask = fn_8017F008() as u8;
    let val = ext::fn_8016FFD4(ext::gm_8016B774(), mask as i32, 0);

    if val < 0 {
        ext::HSD_SisLib_803A70A0(*arg0, 7, c"%s%d".as_ptr(), &[Va::P(c"\x81\x7c".as_ptr().cast()), Va::I32(val.wrapping_neg())]);
    } else {
        ext::HSD_SisLib_803A70A0(*arg0, 7, c"%d".as_ptr(), &[Va::I32(val)]);
    }

    mask = fn_8017F008() as u8;
    ext::fn_8016F548(ext::gm_8016B774().cast(), arg1 as u16, mask, 0)
}


/// fn_8017F608: the per-frame refresh of the bonus labels.
pub unsafe fn fn_8017F608(arg0: *mut c_void) {
    melee_core::ct!("fn_8017F608");
    let p = arg0 as *mut RegClear;
    let mut sp4C = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp40: Vec3 = lbl_803B7C18;

    if (*p).x6C.is_null() {
        (*p).x6C = ext::HSD_SisLib_803A5ACC(0, 0, 0.0f32, 0.0f32, 0.0f32, 6.0f32, 300.0f32);
        ext::HSD_SisLib_803A6368((*p).x6C, 2);
    }
    if (*p).x70.is_null() {
        let text = ext::HSD_SisLib_803A5ACC(0, 0, 0.0f32, 0.0f32, 0.0f32, 6.0f32, 300.0f32);
        (*p).x70 = text;
        if (*p).x11A != 0 {
            ext::HSD_SisLib_803A6368((*p).x70, 4);
        } else {
            ext::HSD_SisLib_803A6368((*p).x70, 3);
        }
    }
    if (*p).xD0 != (*p).xD4 {
        (*p).xD8 = (*p).xD8.wrapping_add(1);
        let text = (*p).x74;
        if !text.is_null() {
            ext::HSD_SisLib_803A5CC4(text);
        }
        (*p).x74 = ext::HSD_SisLib_803A6754(0, 0);
        {
            let state = st();
            (*(*p).x74).pos_z = -10.0f32;
            (*(*p).x74).default_alignment = 2;
            let gm = ext::gmVs_GetSceneController();
            let show = if (*state).x118 != 0 || ((*state).x11A != 0 && (*state).x11B == 0) || (*gm).start.timer_counts_up() != 0 { 0 } else { 1 };
            if show == 0 {
                let str_ = ext::HSD_SisLib_803A6B98((*p).x74, 0.0f32, 0.0f32, c"\x81|".as_ptr(), &[]);
                ext::HSD_SisLib_803A7548((*p).x74, str_, 0.1f32, 0.065f32);
                (*p).xD4 = (*p).xD0;
            } else if (*p).xD8 < 0x3C {
                (*(*p).x74).default_kerning = 1;
                let str_ = ext::HSD_SisLib_803A6B98(
                    (*p).x74,
                    0.0f32,
                    0.0f32,
                    c"%dＸ%d".as_ptr(),
                    &[Va::I32((*p).xD0.wrapping_div((*p).x108 as i32)), Va::I32((*p).x108 as i32)],
                );
                ext::HSD_SisLib_803A7548((*p).x74, str_, 0.089999996f32, 0.065f32);
            } else {
                let str_ = ext::HSD_SisLib_803A6B98((*p).x74, 0.0f32, 0.0f32, c"%d".as_ptr(), &[Va::I32((*p).xD0)]);
                ext::HSD_SisLib_803A7548((*p).x74, str_, 0.1f32, 0.065f32);
                (*p).xD4 = (*p).xD0;
            }
        }
    }
    if (*p).x11A == 0 && (*p).xDC != (*p).xE0 {
        (*p).xE4 = (*p).xE4.wrapping_add(1);
        let text = (*p).x78;
        if !text.is_null() {
            ext::HSD_SisLib_803A5CC4(text);
        }
        (*p).x78 = ext::HSD_SisLib_803A6754(0, 0);
        (*(*p).x78).default_alignment = 2;
        if (*p).xE4 < 0x3C {
            (*(*p).x78).default_kerning = 1;
            let str_ = ext::HSD_SisLib_803A6B98(
                (*p).x78,
                0.0f32,
                0.0f32,
                c"%d\x82w%d".as_ptr(),
                &[Va::I32((*p).xDC.wrapping_div((*p).x10A as i32)), Va::I32((*p).x10A as i32)],
            );
            ext::HSD_SisLib_803A7548((*p).x78, str_, 0.1f32, 0.065f32);
        } else {
            let str_ = ext::HSD_SisLib_803A6B98((*p).x78, 0.0f32, 0.0f32, c"%d".as_ptr(), &[Va::I32((*p).xDC)]);
            ext::HSD_SisLib_803A7548((*p).x78, str_, 0.1f32, 0.065f32);
            (*p).xE0 = (*p).xDC;
        }
    }
    ext::lb_8000B1CC((*p).x10, &mut sp40, &mut sp4C);
    let mut text = (*p).x70;
    if !text.is_null() {
        let px = sp4C.x;
        let py = -sp4C.y;
        let pz = sp4C.z;
        (*text).pos_x = px;
        (*text).pos_y = py;
        (*text).pos_z = pz;
    }
    text = (*p).x6C;
    if !text.is_null() {
        let px = 0.1f32 + sp4C.x;
        let py = -sp4C.y - 6.0f32;
        let pz = sp4C.z;
        (*text).pos_x = px;
        (*text).pos_y = py;
        (*text).pos_z = pz;
    }
    text = (*p).x74;
    if !text.is_null() {
        let px = 6.5f32 + sp4C.x;
        let py = -sp4C.y - 33.5f32;
        let pz = sp4C.z;
        (*text).pos_x = px;
        (*text).pos_y = py;
        (*text).pos_z = pz;
    }
    text = (*p).x78;
    if !text.is_null() {
        let px = 6.5f32 + sp4C.x;
        let py = -sp4C.y - 27.5f32;
        let pz = sp4C.z;
        (*text).pos_x = px;
        (*text).pos_y = py;
        (*text).pos_z = pz;
    }
}

/// fn_8017FA1C: count the total up towards its target; returns 1 while the screen is live.
pub unsafe fn fn_8017FA1C(arg0: *mut c_void) -> i32 {
    melee_core::ct!("fn_8017FA1C");
    let p = arg0 as *mut RegClear;
    let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if ext::HSD_JObjGetFlags((*p).xC) & 0x10 != 0 {
        if !(*p).x80.is_null() {
            ext::HSD_SisLib_803A5CC4((*p).x80);
        }
        (*p).x80 = null_mut();
        (*p).x100 = (*p).xFC.wrapping_sub(1);
        return 0;
    }
    let mut abs_step = (*p).x104;
    let step = lbl_804D65C0;
    let target_val = abs_step;
    let mut diff = target_val.wrapping_sub((*p).xFC);

    if step < 0 {
        abs_step = step.wrapping_neg();
    } else {
        abs_step = step;
    }

    if diff < 0 {
        diff = diff.wrapping_neg();
    }

    if diff <= abs_step {
        (*p).xFC = target_val;
        (*p).x115 = 1;
    } else {
        (*p).xFC = (*p).xFC.wrapping_add(step);
        (*p).x115 = 0;
    }

    if (*p).xFC != (*p).x100 {
        if !(*p).x80.is_null() {
            ext::HSD_SisLib_803A5CC4((*p).x80);
        }
        (*p).x80 = ext::HSD_SisLib_803A6754(0, 0);
        (*(*p).x80).default_alignment = 2;
        let str_ = ext::HSD_SisLib_803A6B98((*p).x80, 0.0f32, 0.0f32, c"%d".as_ptr(), &[Va::I32((*p).xFC)]);
        ext::HSD_SisLib_803A7548((*p).x80, str_, 0.1f32, 0.08f32);
        (*p).x100 = (*p).xFC;
    }

    if !(*p).x80.is_null() {
        ext::lb_8000B1CC((*p).xC, null_mut(), &mut sp14);
        let py = -sp14.y - 30.6f32;
        let px = 16.0f32 + sp14.x;
        let pz = sp14.z;
        let text = (*p).x80;
        (*text).pos_x = px;
        (*text).pos_y = py;
        (*text).pos_z = pz;
    }

    1
}

/// fn_8017FBA4: the total's pop-in text.
pub unsafe fn fn_8017FBA4(arg0: *mut c_void) {
    melee_core::ct!("fn_8017FBA4");
    let p = arg0 as *mut RegClear;
    let mut sp24 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if ext::HSD_JObjGetFlags((*p).x8) & 0x10 != 0 {
        if !(*p).x7C.is_null() {
            ext::HSD_SisLib_803A5CC4((*p).x7C);
        }
        (*p).x7C = null_mut();
        (*p).xF4 = (*p).xF0.wrapping_sub(1);
        return;
    }

    if (*p).xF0 != (*p).xF4 {
        if !(*p).x7C.is_null() {
            ext::HSD_SisLib_803A5CC4((*p).x7C);
        }
        (*p).x7C = ext::HSD_SisLib_803A6754(0, 0);
        let tmp = ext::HSD_SisLib_803A6B98((*p).x7C, 0.0f32, 0.0f32, c"%d".as_ptr(), &[Va::I32((*p).xF0)]);
        ext::HSD_SisLib_803A7548((*p).x7C, tmp, 0.1f32, 0.065f32);
        (*p).xF4 = (*p).xF0;
    }

    let sx = hsd::HSD_JObjGetScaleX((*(*p).x8).parent);
    let sy = hsd::HSD_JObjGetScaleY((*(*p).x8).parent);

    if !(*p).x7C.is_null() {
        ext::HSD_SisLib_803A5CC4((*p).x7C);
    }
    (*p).x7C = ext::HSD_SisLib_803A6754(0, 0);
    (*(*p).x7C).default_alignment = 2;
    (*p).xF8 = (*p).xF8.wrapping_add(1);

    if (*p).xF8 < 0xC {
        let s = ext::HSD_SisLib_803A6B98((*p).x7C, 0.0f32, 0.0f32, c"%s %d".as_ptr(), &[Va::P(c"\x81{".as_ptr().cast()), Va::I32((*p).xCC)]);
        ext::HSD_SisLib_803A7548((*p).x7C, s, 0.1f32 * sx, 0.065f32 * sy);
    } else if (*p).xF8 < 0x20 {
        let s = ext::HSD_SisLib_803A6B98(
            (*p).x7C,
            0.0f32,
            0.0f32,
            c"%s %d".as_ptr(),
            &[Va::P(c"\x81{".as_ptr().cast()), Va::I32((*p).xDC.wrapping_add((*p).xD0))],
        );
        ext::HSD_SisLib_803A7548((*p).x7C, s, 0.1f32 * sx, 0.065f32 * sy);
    } else {
        let str_;
        if (*p).xF0 >= 0 {
            str_ = ext::HSD_SisLib_803A6B98((*p).x7C, 0.0f32, 0.0f32, c"%s %d".as_ptr(), &[Va::P(c"\x81{".as_ptr().cast()), Va::I32((*p).xF0)]);
        } else {
            // the C passes xF0 twice; "%d" consumes only the first
            str_ = ext::HSD_SisLib_803A6B98((*p).x7C, 0.0f32, 0.0f32, c"%d".as_ptr(), &[Va::I32((*p).xF0)]);
        }
        ext::HSD_SisLib_803A7548((*p).x7C, str_, 0.1f32 * sx, 0.065f32 * sy);
    }

    if !(*p).x7C.is_null() {
        ext::lb_8000B1CC((*p).x8, null_mut(), &mut sp24);
        let py = -sp24.y - 31.0f32;
        let px = 8.5f32 + sp24.x;
        let pz = sp24.z;
        let text = (*p).x7C;
        (*text).pos_x = px;
        (*text).pos_y = py;
        (*text).pos_z = pz;
    }
}

/// fn_8017FE54: image-event proc: refresh the capture and fade it in with the frame counter.
pub unsafe extern "C" fn fn_8017FE54(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8017FE54");
    let ev = (*gobj).user_data as *mut RegClearEv;
    let state = st();

    ext::lb_800122C8((*ev).x1C, 0, 0, 1);
    ext::lb_800138D8((*state).x2C, ((120.0f32 * (*state).x10C) as i32).wrapping_add(1) as i8);

    (*ev).x20 = fmadds(0.0225f32, (*state).x110 as f32, -0.175f32);

    if (*ev).x20 < 0.05f32 {
        (*ev).x20 = 0.0f32;
    }
    if (*ev).x20 > 1.0f32 {
        (*ev).x20 = 1.0f32;
    }
}

/// fn_8017FF1C: the screen's per-frame proc.
pub unsafe extern "C" fn fn_8017FF1C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8017FF1C");
    let state = st();
    let arg = state as *mut c_void;
    let mut sp28: *mut HSD_JObj = null_mut();

    let jobj: *mut HSD_JObj = (*gobj).hsd_obj.cast();
    ext::HSD_JObjAnimAll(jobj);

    if (*state).x118 == 0 {
        fn_8017F608(arg);
    }

    let result = fn_8017FA1C(arg);
    fn_8017FBA4(arg);

    if (*state).x117 != 0 && (*state).x110 > 0x29 {
        (*state).xC0 = fn_8017F47C(addr_of_mut!((*state).x84), (*state).xC0 as i32) as u16;

        let mut mask = fn_8017F008() as u8;
        if ext::fn_8016F9A8(ext::gm_8016B774().cast(), (*state).xC0, mask, 0) > 7 {
            (*state).x11F = 0;
        } else {
            (*state).x11F = 1;
        }

        mask = fn_8017F008() as u8;
        if ext::fn_8016F870(ext::gm_8016B774().cast(), (*state).xC0, mask, 0) < 0 {
            (*state).x11E = 1;
        } else {
            (*state).x11E = 0;
        }

        if (*state).x110 % 30 == 0 && (*state).xC8 == 0 {
            mask = fn_8017F008() as u8;
            if ext::fn_8016F9A8(ext::gm_8016B774().cast(), (*state).xC0, mask, 0) > 7 {
                (*state).xC0 = (*state).xC0.wrapping_add(1);
            } else {
                (*state).xC8 = 1;
                (*state).xC4 = (*state).x110;
            }
        }

        {
            let mut buttons = ext::gm_GetButtonsTriggered(ext::Player_GetPlayerId(0) as u8);
            let mut repeat = ext::gm_801A36C0(ext::Player_GetPlayerId(0) as u8);
            if ((repeat | buttons) & (PAD_BUTTON_DOWN | PAD_STICK_DOWN)) | ((repeat | buttons) & 0) != 0 {
                mask = fn_8017F008() as u8;
                if ext::fn_8016F740(ext::gm_8016B774().cast(), (*state).xC0, mask, 0) > 0 {
                    mask = fn_8017F008() as u8;
                    if ext::fn_8016F9A8(ext::gm_8016B774().cast(), (*state).xC0, mask, 0) > 7 {
                        mask = fn_8017F008() as u8;
                        (*state).xC0 = ext::fn_8016F740(ext::gm_8016B774().cast(), (*state).xC0, mask, 0) as u16;
                        (*state).xC8 = 1;
                        (*state).xC4 = (*state).x110;
                    }
                }
            } else {
                buttons = ext::gm_GetButtonsTriggered(ext::Player_GetPlayerId(0) as u8);
                repeat = ext::gm_801A36C0(ext::Player_GetPlayerId(0) as u8);
                if ((repeat | buttons) & (PAD_BUTTON_UP | PAD_STICK_UP)) | ((repeat | buttons) & 0) != 0 {
                    let g = ext::gm_8016B774().cast();
                    let c = (*state).xC0;
                    let m = fn_8017F008() as u8;
                    if ext::fn_8016F870(g, c, m, 0) >= 0 {
                        let g = ext::gm_8016B774().cast();
                        let c = (*state).xC0;
                        let m = fn_8017F008() as u8;
                        (*state).xC0 = ext::fn_8016F870(g, c, m, 0) as u16;
                        (*state).xC8 = 1;
                        (*state).xC4 = (*state).x110;
                    }
                }
            }
        }
    }

    if (*state).x11A != 0
        && (*state).x110 > 0x14
        && (*state).x11D < (*state).x11C
        && ((0.5f32 * ((*state).x110.wrapping_sub(0x14) as f32)) as i32) > ((*state).x11D as i32)
    {
        ext::lb_80011E24(jobj, &mut sp28, &[(*state).x11D as i32 + 7, -1]);
        ext::HSD_JObjClearFlagsAll(sp28, JOBJ_HIDDEN as u32);
        (*state).x11D = (*state).x11D.wrapping_add(1);
    }

    if (*state).x115 == 0 && result != 0 {
        ext::fn_80168F2C(0);
    }

    for i in 0..6 {
        if ext::Player_GetPlayerSlotType(i) == Gm_PKind_Human
            && (*addr_of_mut!(HSD_PadMasterStatus))[ext::Player_GetPlayerId(i) as u8 as usize].trigger & HSD_PAD_A != 0
        {
            (*state).xFC = (*state).x104;
            (*state).x115 = 1;
            break;
        }
    }

    if (*state).x110 > 0x3E {
        for i in 0..6 {
            if ext::Player_GetPlayerSlotType(i) == Gm_PKind_Human
                && (*addr_of_mut!(HSD_PadMasterStatus))[ext::Player_GetPlayerId(i) as u8 as usize].trigger & 0x1000 != 0
            {
                (*state).xFC = (*state).x104;
                (*state).x116 = 1;
                hsd::sfxForward();
                break;
            }
        }
    }

    if (*state).x11E != 0 {
        ext::HSD_JObjSetFlagsAll((*state).x20, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjClearFlagsAll((*state).x20, JOBJ_HIDDEN as u32);
    }

    if (*state).x11F != 0 {
        ext::HSD_JObjSetFlagsAll((*state).x24, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjClearFlagsAll((*state).x24, JOBJ_HIDDEN as u32);
    }

    if (*state).x10C < 1.0f32 {
        (*state).x10C += 0.05f32;
    } else {
        (*state).x10C = 1.0f32;
    }

    if (*state).x110.wrapping_add(0x10000) != 0xFFFF {
        (*state).x110 = (*state).x110.wrapping_add(1);
    }
}

/// fn_801803FC: create the result model GObj and prime the labels.
pub unsafe fn fn_801803FC(arg0: *mut c_void) -> i32 {
    melee_core::ct!("fn_801803FC");
    let p = arg0 as *mut RegClear;
    let mut sp10: *mut HSD_JObj = null_mut();
    let state = st();
    let mdl = addr_of_mut!((*p).x4C);

    let gobj = ext::GObj_Create(0xE, 0xE, 0);
    (*p).x0 = gobj;
    if gobj.is_null() {
        // C: HSD_JObjAnimAll(GET_JOBJ(gobj)) on the NULL gobj (never reached with a real allocator)
        ext::HSD_JObjAnimAll(get_jobj(gobj));
        ext::OSReport(c"Error : gobj don't get (gmRegClearAddModel)\n".as_ptr(), &[]);
        ext::OSPanic(c"gmregclear.c".as_ptr(), 0x42C, c"".as_ptr(), &[]);
    }
    let jobj = ext::HSD_JObjLoadJoint((*mdl).joint);
    if jobj.is_null() {
        ext::OSReport(c"Error : jobj don't get (gmRegClearAddModel)\n".as_ptr(), &[]);
        ext::OSPanic(c"gmregclear.c".as_ptr(), 0x432, c"".as_ptr(), &[]);
    }
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_8017FF1C), 0x11);
    ext::fn_801689E4(jobj, mdl, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
    ext::HSD_JObjAnimAll(jobj);
    {
        ext::lb_8001204C(jobj, addr_of_mut!((*p).x4), lbl_803D8B88.as_ptr(), 0xA);
    }
    if (*state).x117 == 0 {
        ext::HSD_JObjSetFlagsAll((*p).x14, JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlagsAll((*p).x20, JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlagsAll((*p).x24, JOBJ_HIDDEN as u32);
    }
    if (*p).x11A != 0 {
        for i in 0..0xAi32 {
            ext::lb_80011E24(jobj, &mut sp10, &[i + 7, -1]);
            ext::HSD_JObjSetFlagsAll(sp10, JOBJ_HIDDEN as u32);
        }
        ext::HSD_JObjSetFlagsAll((*p).x10, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjSetFlagsAll((*p).x1C, JOBJ_HIDDEN as u32);
        if (*p).x118 != 0 {
            ext::HSD_JObjSetFlagsAll((*p).x18, JOBJ_HIDDEN as u32);
        }
    }
    let temp = (*p).x4;
    let frame = if (*p).x114 != 0 { 1.0f32 } else { 0.0f32 };
    let aobj = (*(*(*(*temp).u.dobj).mobj).tobj).aobj;
    ext::HSD_AObjSetCurrentFrame(aobj, frame);
    ext::HSD_AObjSetRate((*(*(*(*temp).u.dobj).mobj).tobj).aobj, 0.0f32);
    if (*p).x118 == 0 {
        fn_8017F608(arg0);
    }
    fn_8017FBA4(arg0);
    fn_8017FA1C(arg0)
}

/// `fn_80180630_CreateCameraGObj` (static inline).
pub unsafe fn fn_80180630_CreateCameraGObj() -> *mut HSD_GObj {
    melee_core::ct!("fn_80180630_CreateCameraGObj");
    ext::GObj_Create(0xE, 0xE, 0)
}

/// `fn_80180630_LoadLightList` (static inline).
pub unsafe fn fn_80180630_LoadLightList(state: *mut RegClear) -> *mut c_void {
    melee_core::ct!("fn_80180630_LoadLightList");
    ext::lb_80011AC4((*state).x5C.cast()).cast()
}

/// `fn_80180630_LoadCameraDesc` (static inline).
pub unsafe fn fn_80180630_LoadCameraDesc(state: *mut RegClear) -> *mut c_void {
    melee_core::ct!("fn_80180630_LoadCameraDesc");
    ext::HSD_CObjLoadDesc((*state).x60.cast()).cast()
}

/// `fn_80180630_GetModelDesc` (static inline).
pub unsafe fn fn_80180630_GetModelDesc(state: *mut RegClear) -> *mut DynamicModelDesc {
    melee_core::ct!("fn_80180630_GetModelDesc");
    addr_of_mut!((*state).x4C)
}

/// `fn_80180630_SetupSisLib` (static inline).
pub unsafe fn fn_80180630_SetupSisLib(cam_gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80180630_SetupSisLib");
    ext::HSD_SisLib_803A611C(0, cam_gobj, 9, 0xD, 0, 0xE, 0, 0x13);
    if ext::lbLang_IsSavedLanguageUS() != 0 {
        ext::HSD_SisLib_803A62A0(0, c"SdClr.usd".as_ptr(), c"SIS_ClearData".as_ptr());
    } else {
        ext::HSD_SisLib_803A62A0(0, c"SdClr.dat".as_ptr(), c"SIS_ClearData".as_ptr());
    }
}

/// `fn_80180630_CreateLightAndCamera` (static inline).
pub unsafe fn fn_80180630_CreateLightAndCamera(state: *mut RegClear, cam_gobj: *mut *mut HSD_GObj) {
    melee_core::ct!("fn_80180630_CreateLightAndCamera");
    let light_gobj = ext::GObj_Create(0xB, 3, 0);
    ext::HSD_GObjObject_80390A70(light_gobj, HSD_GObj_LightKind as u8, fn_80180630_LoadLightList(state));
    ext::GObj_SetupGXLink(light_gobj, Some(ext::HSD_GObj_LObjCallback), 0xA, 0);

    *cam_gobj = fn_80180630_CreateCameraGObj();
}

/// `fn_80180630_GetX118` (static inline).
pub unsafe fn fn_80180630_GetX118(state: *const RegClear) -> u8 {
    melee_core::ct!("fn_80180630_GetX118");
    (*state).x118
}

/// fn_80180630: set up the clear screen. `arg0`/`arg1` are the score and bonus-points base,
/// `arg2` the mode (1 = 1P event, 2 = ..., 3 = bonus stage), `arg3` (bool = int) the 1P flag.
pub unsafe fn fn_80180630(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: *mut MatchEnd) {
    melee_core::ct!("fn_80180630");
    let mut sp64: i32 = 0;
    let mut sp60: i32 = 0;
    let mut special_score_value: i32 = 0;
    let mut coin_count: i32 = 0;
    let mut cam_gobj: *mut HSD_GObj = null_mut();
    let mut special_score: i32 = 0;

    let mut coins: u16 = (*arg4).player_standings[0].xE;
    let state = st();
    ext::memzero(state.cast(), core::mem::size_of::<RegClear>() as isize);
    (*state).xD4 = -1;
    (*state).xD8 = 0;
    (*state).xE0 = -1;
    (*state).xE4 = 0;
    (*state).xEC = -1;
    (*state).xF4 = -1;
    (*state).x100 = -1;
    (*state).x11E = 1;
    (*state).x11F = 1;
    (*state).x108 = 0x64;
    (*state).x10A = 0x64;
    (*state).x114 = arg3 as u8;

    match arg2 {
        1 => {
            ext::Ground_801C1DE4(&mut sp60, &mut sp64);
            (*state).x11A = 1;
            (*state).x11C = sp64.wrapping_sub(sp60) as u8;
            (*state).x108 = 0xC8;
            if sp60 == 0 {
                (*state).x11B = 1;
            }
        }
        3 => {
            let temp = ext::gmVs_GetSceneController();
            (*state).x118 = 1;
            if (*temp).state.match_result as u32 == OUTCOME_UNK_1P_BONUS_STAGE_END as u32 {
                ext::grPushOn_80219204(ext::Ground_801C1DD4(), &mut special_score_value, &mut coin_count);
                special_score = special_score_value;
                // Keep the 64-bit round trip for register allocation.
                coins = (coin_count as i64) as i32 as u16;
                (*state).x108 = 0x1F4;
            }
        }
        2 => {
            (*state).x119 = 1;
        }
        _ => {}
    }

    {
        let tmp = ext::gm_8016B774();
        ext::fn_8016F344(tmp);
    }

    let var_r0: u8;
    if (*state).x11A == 0 && (*state).x118 == 0 && (*state).x119 == 0 && {
        let mask = fn_8017F008() as u8;
        ext::fn_8016F9A8(ext::gm_8016B774().cast(), 0, mask, 0) != 0
    } {
        var_r0 = 1;
    } else {
        var_r0 = 0;
    }
    (*state).x117 = var_r0;

    if (*state).x117 != 0 {
        (*state).xC8 = 0;
    } else {
        (*state).xC8 = 1;
    }

    if (*state).x118 == 0 {
        (*state).xD0 = fn_8017F09C();
        (*state).xDC = fn_8017F14C(arg4.cast());
    }

    if (*state).x117 != 0 {
        (*state).xE8 = fn_8017F1B8();
    }

    (*state).xF0 = (*state).xE8.wrapping_add((*state).xDC.wrapping_add((*state).xD0).wrapping_add(special_score));
    (*state).xFC = arg0.wrapping_add(arg1);
    (*state).xCC = arg1;

    let total = arg0.wrapping_add((*state).xCC).wrapping_add((*state).xF0);
    let mut var_r4 = total;
    if total > 999999999 {
        var_r4 = 999999999;
    } else if var_r4 < 0 {
        var_r4 = 0;
    }
    (*state).x104 = var_r4;
    lbl_804D65C0 = var_r4.wrapping_sub(arg0.wrapping_add(arg1)).wrapping_div(10);

    {
        let mut scene_data: *mut c_void = null_mut();

        let archive = ext::lbArchive_80016DBC(
            c"GmRegClr".as_ptr(),
            addr_of_mut!(scene_data).cast(),
            &[c"ScGamRegClear_scene_data".as_ptr().cast(), null_mut::<c_void>() as *const c_void],
        );
        (*state).x48 = archive;
        if scene_data.is_null() {
            ext::OSReport(c"Error : Cannot open archive file (File Name : %s).".as_ptr(), &[Va::P(c"GmRegClr".as_ptr().cast())]);
        }
        ext::fn_80168A6C(scene_data, fn_80180630_GetModelDesc(state).cast(), 0);
    }

    fn_80180630_CreateLightAndCamera(state, &mut cam_gobj);
    ext::HSD_GObjObject_80390A70(cam_gobj, HSD_GObj_CameraKind, fn_80180630_LoadCameraDesc(state));
    ext::GObj_SetupGXLinkMax(cam_gobj, Some(ext::HSD_GObj_803910D8), 8);
    (*cam_gobj).gxlink_prios = 0x4C00;

    fn_80180630_SetupSisLib(cam_gobj);

    fn_801803FC(state.cast());
    ext::fn_80168F7C();

    let var_r3 = if ext::HSD_Randi(2) != 0 { 0xA } else { 0xB };
    ext::lbAudioAx_80023F28(var_r3);

    ext::Camera_8002F7AC(0);
    ext::lb_800121FC(addr_of_mut!((*state).x30), 0x280, 0x1E0, GX_TF_RGB5A3, 0);
    (*state).x2C = ext::lb_800138EC(addr_of_mut!((*state).x30), None, 2, 0x32, 0.0f32, 0.0f32, 1.0f32, 1.0f32);
    ext::lb_800138D8((*state).x2C, 1);
    ext::lb_800138CC((*state).x2C, Some(fn_8017FE54));

    if (*ext::gm_GetStartMeleeRules()).x1_1() != 0 && coins != 0 {
        if fn_80180630_GetX118(state) == 0 {
            ext::un_802FF128(0x5A, 0x1AE, coins as i32, 5);
        } else {
            ext::un_802FF128(0x86, 0xC8, coins as i32, 5);
        }
    }

    (*arg4).player_standings[0].xE = coins;
    fn_8017F2A4(addr_of_mut!((*state).x84), 264.0f32, 211.0f32);
}

/// fn_80180AC0: whether the player skipped the screen (x116 == 1).
pub unsafe fn fn_80180AC0() -> i32 {
    melee_core::ct!("fn_80180AC0");
    if (*st()).x116 == 1 {
        return 1;
    }
    0
}
