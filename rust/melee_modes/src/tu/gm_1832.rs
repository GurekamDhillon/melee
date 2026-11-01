//! src/melee/gm/gm_1832.c: the Classic / All-Star "VS intro" scene (character cards, the splash
//! screen made of pre-rendered character images, the team layout tables).
//!
//! The file's statics are `pub static mut` so the state oracle (tests/tu_gm_1832.rs against
//! gen/turef/tu_gm_1832_ref.c, the C TU compiled natively with shipping contraction) can seed and
//! compare them. This follows the `TARGET_PC` branches of the C (host character ids are mapped
//! through `gm_MexVanillaKind` before indexing the GmIntEz tables; the splash ortho matrix is a
//! 4x4). The shipping build contracts `a * b + c` into fused multiply-adds, which is `mul_add`
//! here (the splash tile positions and `fn_801855BC`).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::HSD_GObj as HSD_GObj_gm_r10;
#[allow(unused_imports)]
use melee_types::HSD_SObj as HSD_SObj_gm_r10;
use crate::tu_util::*;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_types::inline as mi;
use melee_types::*;

/// Callees outside this TU (hooks; unset they panic `TODO: <name> is not ported`).
pub mod ext {
    pub use crate::tu_ext::{
        efAsync_LoadSync, efLib_Init, fn_80160DE8, fn_80160F58, ftDemo_ObjAllocInit, gm_80160980,
        gm_8016895C, gm_80167858, gm_80168C5C, gm_80168F88, gm_801A4B60, gm_GetNumCostumesForCKind,
        gm_MexVanillaKind, lbArchive_80016DBC, lbAudioAx_800236B8, lbAudioAx_800236DC,
        lbAudioAx_800237A8, lbAudioAx_80023F28, lbLang_IsSavedLanguageUS, lb_800121FC,
        lb_80011AC4, lb_80011E24, lb_8000FCDC, mpColl_80041C78, Camera_800313E0, Camera_Init,
        GXBegin, GXClearVtxDesc, GXLoadPosMtxImm, GXPosition3f32, GXSetAlphaCompare,
        GXSetBlendMode, GXSetChanCtrl, GXSetColorUpdate, GXSetCullMode, GXSetCurrentMtx,
        GXSetNumChans, GXSetNumTevStages, GXSetNumTexGens, GXSetProjection, GXSetTevOp,
        GXSetTevOrder, GXSetVtxAttrFmt, GXSetVtxDesc, GXSetZMode, GObj_Create, GObj_SetupGXLink,
        GObj_SetupGXLinkMax, GetNameText, Ground_801C0378, HSD_AObjSetRate, HSD_CObjEndCurrent,
        HSD_CObjLoadDesc, HSD_CObjSetCurrent, HSD_FogLoadDesc, HSD_GObjFree, HSD_GObjGXLink_8039084C,
        HSD_GObjObject_80390A70, HSD_GObj_804D783C, HSD_GObj_80390ED0, HSD_GObj_803910D8,
        HSD_GObj_CameraKind, HSD_GObj_FogCallback, HSD_GObj_FogKind, HSD_GObj_JObjCallback,
        HSD_GObj_JObjKind, HSD_GObj_LObjCallback, HSD_GObj_LightKind, HSD_GObj_SetupProc,
        HSD_ImageDescCopyFromEFB, HSD_JObjAnimAll, HSD_JObjLoadJoint, HSD_JObjReqAnimAll,
        HSD_Randf, HSD_SObjLib_803A477C, HSD_SObjLib_803A49E0, HSD_SObjLib_803A54EC,
        HSD_SObjLib_803A55DC, HSD_SObjLib_804D7960, HSD_SisLib_803A5ACC, HSD_SisLib_803A611C,
        HSD_SisLib_803A62A0, HSD_SisLib_803A6368, HSD_SisLib_803A6754, HSD_SisLib_803A6B98, HSD_SisLib_803A70A0,
        HSD_SisLib_803A7548, HSD_StateInvalidate, HSD_TObjReqAnimAll, HSD_identityMtx,
        Item_80266FA8, Item_80266FCC, MTXOrtho, Player_80032768, Player_80032828, Player_80036CF0,
        Player_800366DC, Player_80036E20, Player_80036F34, Player_80037054, Player_GetEntity,
        Player_GetEntityAtIndex, Player_InitAllPlayers, Player_SetCostumeId,
        Player_SetFacingDirection, Player_SetFlagsBit5, Player_SetModelScale,
        Player_SetPlayerCharacter, Player_SetPlayerId, Player_SetSlottype, Stage_802251E8,
        Stage_8022524C,
    };
}

/// `MSL_TrigF_80400770` / `MSL_TrigF_80400774`: the two constants `fn_801855BC` returns for
/// negative / NaN input (the C declares them `extern float[]`; owner TU is elsewhere).
pub static mut MSL_TrigF_80400770: [f32; 1] = [0.0];
pub static mut MSL_TrigF_80400774: [f32; 1] = [0.0];

#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_1832_804735A8_t {
    pub x0: i32,
    pub x4: [*mut HSD_JObj; 13],
    pub x38: u16,
    pub x3A: u16,
    pub x3C: u16,
    pub x3E: u16,
}
melee_core::gc_layout!(gm_1832_804735A8_t, "lbl_804735A8", size 0x40, align 4, [
    x0 @ 0x0: 4 "int",
    x4 @ 0x4: 52 "HSD_JObj*[13]",
    x38 @ 0x38: 2 "u16",
    x3A @ 0x3A: 2 "u16",
    x3C @ 0x3C: 2 "u16",
    x3E @ 0x3E: 2 "u16",
]);

/// Splash screen state (the C's `lbl_804735E8`, GC +0x40..+0xE4).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_1832_804735E8_t {
    pub x40: [HSD_ImageDesc; 3],
    pub x88: [HSD_ImageDesc; 3],
    pub xD0: [u8; 10],
    pub pad_DA: [u8; 2],
    pub xDC: *mut HSD_GObj,
    pub xE0: u8,
    pub xE1: u8,
    pub xE2: u8,
    pub xE3: u8,
}
melee_core::gc_layout!(gm_1832_804735E8_t, "lbl_804735E8", size 0xA4, align 4, [
    x40 @ 0x0: 72 "HSD_ImageDesc[3]",
    x88 @ 0x48: 72 "HSD_ImageDesc[3]",
    xD0 @ 0x90: 10 "u8[10]",
    pad_DA @ 0x9A: 2 "u8[2]",
    xDC @ 0x9C: 4 "HSD_GObj*",
    xE0 @ 0xA0: 1 "u8",
    xE1 @ 0xA1: 1 "u8",
    xE2 @ 0xA2: 1 "u8",
    xE3 @ 0xA3: 1 "u8",
]);

/// `struct gm_1832_8047368C_t` (0x24): the intro request (model scale kind, game type, the two
/// teams' character / costume ids).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_1832_8047368C_t {
    pub model_scale_kind: i32,
    pub game_type: i32,
    pub xEC: u8,
    pub xED: u8,
    pub xEE: u8,
    pub xEF: u8,
    pub xF0: u8,
    pub xF1: [u8; 3],
    pub xF4: [u8; 3],
    pub xF7: [u8; 3],
    pub xFA: [u8; 3],
    pub xFD: [u8; 3],
    pub x100: [u8; 3],
    pub pad_1F: [u8; 5],
}
melee_core::gc_layout!(gm_1832_8047368C_t, "gm_1832_8047368C_t", size 0x24, align 4, [
    model_scale_kind @ 0x0: 4 "s32",
    game_type @ 0x4: 4 "s32",
    xEC @ 0x8: 1 "u8",
    xED @ 0x9: 1 "u8",
    xEE @ 0xA: 1 "u8",
    xEF @ 0xB: 1 "u8",
    xF0 @ 0xC: 1 "u8",
    xF1 @ 0xD: 3 "u8[3]",
    xF4 @ 0x10: 3 "u8[3]",
    xF7 @ 0x13: 3 "u8[3]",
    xFA @ 0x16: 3 "u8[3]",
    xFD @ 0x19: 3 "u8[3]",
    x100 @ 0x1C: 3 "u8[3]",
    pad_1F @ 0x1F: 5 "u8[5]",
]);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClassicSlotVals {
    pub vals: [f32; 3],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClassicCharLayout {
    pub x00: f32,
    pub x04: f32,
    pub x08: Vec3,
    pub pad_14: [u8; 8],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClassicTeamEntry {
    pub x00: f32,
    pub x04: f32,
    pub x08: f32,
    pub pad_0C: [u8; 8],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClassicSplashRow {
    pub x00: [f32; 3],
    pub x0C: [f32; 3],
    pub x18: [f32; 3],
    pub x24: [f32; 3],
}
melee_core::gc_layout!(ClassicSlotVals, "ClassicSlotVals", size 0xC, align 4, [vals @ 0x0: 12 "f32[3]"]);
melee_core::gc_layout!(ClassicCharLayout, "ClassicCharLayout", size 0x1C, align 4, [
    x00 @ 0x0: 4 "f32",
    x04 @ 0x4: 4 "f32",
    x08 @ 0x8: 12 "Vec3",
    pad_14 @ 0x14: 8 "u8[8]",
]);
melee_core::gc_layout!(ClassicTeamEntry, "ClassicTeamEntry", size 0x14, align 4, [
    x00 @ 0x0: 4 "f32",
    x04 @ 0x4: 4 "f32",
    x08 @ 0x8: 4 "f32",
    pad_0C @ 0xC: 8 "u8[8]",
]);
melee_core::gc_layout!(ClassicSplashRow, "ClassicSplashRow", size 0x30, align 4, [
    x00 @ 0x0: 12 "f32[3]",
    x0C @ 0xC: 12 "f32[3]",
    x18 @ 0x18: 12 "f32[3]",
    x24 @ 0x24: 12 "f32[3]",
]);

/// The GmIntEz.dat table `gmIntroEasyTable` (no pointers: the same bytes native and on the GC).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClassicIntroTable {
    pub x00: [ClassicSlotVals; 2],
    pub x18: [ClassicSlotVals; 3],
    pub x3C: [ClassicSlotVals; 4],
    pub x6C: [ClassicCharLayout; 28],
    pub x37C: [ClassicTeamEntry; 25],
    pub pad_570: [u8; 0xC],
    pub x57C: [ClassicSplashRow; 3],
    pub pad_60C: [u8; 0x24],
    pub x630: [ClassicSlotVals; 3],
    pub x654: [ClassicSlotVals; 3],
    pub x678: [ClassicSlotVals; 4],
    pub x6A8: [ClassicCharLayout; 28],
}
melee_core::gc_layout!(ClassicIntroTable, "gmIntroEasyTable", size 0x9B8, align 4, [
    x00 @ 0x0: 24 "ClassicSlotVals[2]",
    x18 @ 0x18: 36 "ClassicSlotVals[3]",
    x3C @ 0x3C: 48 "ClassicSlotVals[4]",
    x6C @ 0x6C: 784 "ClassicCharLayout[28]",
    x37C @ 0x37C: 500 "ClassicTeamEntry[25]",
    pad_570 @ 0x570: 12 "u8[12]",
    x57C @ 0x57C: 144 "ClassicSplashRow[3]",
    pad_60C @ 0x60C: 36 "u8[36]",
    x630 @ 0x630: 36 "ClassicSlotVals[3]",
    x654 @ 0x654: 36 "ClassicSlotVals[3]",
    x678 @ 0x678: 48 "ClassicSlotVals[4]",
    x6A8 @ 0x6A8: 784 "ClassicCharLayout[28]",
]);

pub static mut lbl_804735A8: gm_1832_804735A8_t = unsafe { core::mem::zeroed() };
pub static mut lbl_804735E8: gm_1832_804735E8_t = unsafe { core::mem::zeroed() };
pub static mut lbl_8047368C: gm_1832_8047368C_t = unsafe { core::mem::zeroed() };
pub static mut lbl_804D65F0: *mut HSD_GObj = null_mut();
pub static mut lbl_804D65F4: *mut HSD_Archive = null_mut();
pub static mut lbl_804D65F8: *mut HSD_Archive = null_mut();
pub static mut lbl_804D65FC: *mut SceneDesc = null_mut();
pub static mut lbl_804D6600: *mut SceneDesc = null_mut();
pub static mut lbl_804D6604: *mut ClassicIntroTable = null_mut();
pub static mut lbl_804D6608: i32 = 0;

/// `ClassicProcArray lbl_803B7C28 / lbl_803B7C40`: the six per-slot proc callbacks (three per team).
pub static lbl_803B7C28: [HSD_GObjEvent; 6] = [
    Some(fn_801849E0),
    Some(fn_80184A04),
    Some(fn_80184A28),
    Some(fn_80184A4C),
    Some(fn_80184A70),
    Some(fn_80184A94),
];
pub static lbl_803B7C40: [HSD_GObjEvent; 6] = [
    Some(fn_801849E0),
    Some(fn_80184A04),
    Some(fn_80184A28),
    Some(fn_80184A4C),
    Some(fn_80184A70),
    Some(fn_80184A94),
];

pub static gm_1832_CharScale: [f32; 28] = [
    0.6, 0.35, 0.6, 0.5, 0.6, 0.35, 0.6, 0.6, 0.7, 0.6, 0.5, 0.6, 0.6, 0.6, 0.5, 0.5, 0.6, 0.5, 0.6,
    0.6, 0.5, 0.6, 0.6, 0.6, 0.6, 0.5, 0.5, 0.5,
];

const fn v3(x: f32, y: f32) -> Vec3 {
    Vec3 { x, y, z: 0.0 }
}
pub static gm_1832_CharOffset: [Vec3; 28] = [
    v3(0.0, -6.0), v3(0.0, -3.5), v3(-1.0, -3.5), v3(0.0, -3.5), v3(0.0, -1.0), v3(0.0, -3.0),
    v3(0.0, -5.0), v3(0.0, -3.5), v3(0.0, -3.5), v3(0.0, -3.5), v3(0.0, -3.0), v3(-0.5, -3.5),
    v3(-2.0, -6.0), v3(0.0, -2.5), v3(0.0, -2.5), v3(-1.0, -1.0), v3(0.0, -5.0), v3(0.0, -3.0),
    v3(-1.0, -6.0), v3(-1.0, -2.5), v3(-1.0, -3.5), v3(0.0, -3.5), v3(0.0, -4.0), v3(-1.0, -3.5),
    v3(0.0, -1.5), v3(0.0, -4.5), v3(0.0, -3.5), v3(0.0, -3.5),
];

pub static gm_1832_SamusOffset: Vec3 = Vec3 { x: 3.0, y: 4.0, z: -5.0 };

/// Shift-JIS "Ｔｅａｍ" (NUL padded).
pub static lbl_803D9414: [u8; 12] = [0x82, 0x73, 0x82, 0x85, 0x82, 0x81, 0x82, 0x8D, 0, 0, 0, 0];
/// Shift-JIS "混戦" ("free-for-all").
pub static lbl_804D40A0: [u8; 5] = [0x8C, 0x52, 0x92, 0x63, 0x00];
pub static lbl_803B7C58: [&core::ffi::CStr; 4] = [c"IrAls", c"IrEzTarg", c"IrEzTuki", c"IrEzFigG"];
pub static lbl_803D9444: &core::ffi::CStr = c"ScItrAllstar_scene_data";
pub static lbl_804D40B0: &core::ffi::CStr = c"IrRdMap";

/// `gm_IntroTeamRow` (TARGET_PC): the host character mapped into the 25-row GmIntEz team table.
unsafe fn gm_IntroTeamRow(ckind: i32) -> i32 {
    let row = ext::gm_MexVanillaKind(ckind, 1);
    if (row as u32) < 25 { row } else { CKind_Mario as i32 }
}
/// `INTRO_TABLE_CK`
#[inline]
unsafe fn table_ck(ck: u8) -> i32 {
    ext::gm_MexVanillaKind(ck as i32, 1)
}
/// `INTRO_TEAM_CK`
#[inline]
unsafe fn team_ck(ck: u8) -> i32 {
    gm_IntroTeamRow(ck as i32)
}

#[inline]
unsafe fn slot(arr: *const ClassicSlotVals, i: isize, k: i32) -> f32 {
    (*arr.offset(i)).vals[k as usize]
}
#[inline]
unsafe fn layout6c(i: i32) -> *const ClassicCharLayout {
    (&raw const (*lbl_804D6604).x6C as *const ClassicCharLayout).offset(i as isize)
}
#[inline]
unsafe fn layout6a8(i: i32) -> *const ClassicCharLayout {
    (&raw const (*lbl_804D6604).x6A8 as *const ClassicCharLayout).offset(i as isize)
}
#[inline]
unsafe fn team37c(i: i32) -> *const ClassicTeamEntry {
    (&raw const (*lbl_804D6604).x37C as *const ClassicTeamEntry).offset(i as isize)
}
#[inline]
unsafe fn splash(i: isize) -> *const ClassicSplashRow {
    (&raw const (*lbl_804D6604).x57C as *const ClassicSplashRow).offset(i)
}
#[inline]
unsafe fn vals_of(p: *const [ClassicSlotVals; 2]) -> *const ClassicSlotVals {
    p as *const ClassicSlotVals
}

/// The tail both layout functions share: copy the JObj's translation / scale to the entity's
/// second JObj for every player slot that owns `arg0`.
unsafe fn mirror_to_entity2(arg0: *mut HSD_GObj, jobj: *mut HSD_JObj, pos: &mut Vec3) {
    for i in 0..6i32 {
        if ext::Player_GetEntity(i) == arg0 {
            let entity2 = ext::Player_GetEntityAtIndex(i, 1);
            if !entity2.is_null() {
                let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                let jobj2 = (*entity2).hsd_obj as *mut HSD_JObj;
                mi::HSD_JObjGetTranslation(&*jobj, pos);
                mi::HSD_JObjSetTranslate(&mut *jobj2, pos, &mut Dirty);
                mi::HSD_JObjGetScale(&*jobj, &mut scale);
                mi::HSD_JObjSetScale(&mut *jobj2, &scale, &mut Dirty);
            }
        }
    }
}

pub unsafe fn fn_8018325C(arg0: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("fn_8018325C");
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    let st = &mut lbl_804735A8;
    let src = st.x4[0];
    let k = &lbl_8047368C;
    let a = arg1 as usize;

    mi::HSD_JObjGetTranslation(&*src, &mut pos);
    mi::HSD_JObjSetTranslate(&mut *jobj, &pos, &mut Dirty);

    match arg1 {
        0 => {}
        1 => {
            mi::HSD_JObjAddTranslationZ(&mut *jobj, -10.0, &mut Dirty);
            if st.x38 as u32 > 0x50 {
                let t = &mut *st.x4[2];
                mi::HSD_JObjSetTranslateX(t, -1.0, &mut Dirty);
                mi::HSD_JObjSetTranslateY(t, 0.0, &mut Dirty);
                mi::HSD_JObjSetTranslateZ(t, 5.0, &mut Dirty);
                mi::HSD_JObjSetScaleX(t, 2.8, &mut Dirty);
                mi::HSD_JObjSetScaleY(t, 2.8, &mut Dirty);
            }
        }
        2 => {
            mi::HSD_JObjAddTranslationZ(&mut *jobj, -30.0, &mut Dirty);
            if st.x38 as u32 > 0x5A {
                let t = &mut *st.x4[3];
                mi::HSD_JObjSetTranslateX(t, -0.5, &mut Dirty);
                mi::HSD_JObjSetTranslateY(t, 3.0, &mut Dirty);
                mi::HSD_JObjSetTranslateZ(t, 5.0, &mut Dirty);
                mi::HSD_JObjSetScaleX(t, 2.2, &mut Dirty);
                mi::HSD_JObjSetScaleY(t, 2.2, &mut Dirty);
            }
        }
        _ => {}
    }

    let tb = lbl_804D6604;
    if k.model_scale_kind == 2 {
        let ef = k.xEF as isize;
        let s630 = &raw const (*tb).x630 as *const ClassicSlotVals;
        let s654 = &raw const (*tb).x654 as *const ClassicSlotVals;
        let s678 = &raw const (*tb).x678 as *const ClassicSlotVals;
        mi::HSD_JObjAddTranslationX(
            &mut *jobj,
            (*layout6a8(table_ck(k.xF1[a]))).x00 - slot(s630, ef, arg1),
            &mut Dirty,
        );
        mi::HSD_JObjAddTranslationY(
            &mut *jobj,
            (*layout6a8(table_ck(k.xF1[a]))).x04 + slot(s654, ef, arg1),
            &mut Dirty,
        );
        mi::HSD_JObjSetScaleX(
            &mut *jobj,
            (*layout6a8(table_ck(k.xF1[a]))).x08.x * slot(s678, ef, arg1),
            &mut Dirty,
        );
        mi::HSD_JObjSetScaleY(
            &mut *jobj,
            (*layout6a8(table_ck(k.xF1[a]))).x08.y * slot(s678, ef, arg1),
            &mut Dirty,
        );
        mi::HSD_JObjSetScaleZ(
            &mut *jobj,
            (*layout6a8(table_ck(k.xF1[a]))).x08.z * slot(s678, ef, arg1),
            &mut Dirty,
        );
    } else {
        let ef = k.xEF as isize;
        let s00 = &raw const (*tb).x00 as *const ClassicSlotVals;
        let s18 = &raw const (*tb).x18 as *const ClassicSlotVals;
        let s3c = &raw const (*tb).x3C as *const ClassicSlotVals;
        mi::HSD_JObjAddTranslationX(
            &mut *jobj,
            (*layout6c(table_ck(k.xF1[a]))).x00 - slot(s00, ef - 1, arg1),
            &mut Dirty,
        );

        mi::HSD_JObjAddTranslationY(
            &mut *jobj,
            (*layout6c(table_ck(k.xF1[a]))).x04 + slot(s18, ef, arg1),
            &mut Dirty,
        );

        mi::HSD_JObjSetScaleX(
            &mut *jobj,
            (*layout6c(table_ck(k.xF1[a]))).x08.x * slot(s3c, ef, arg1),
            &mut Dirty,
        );
        mi::HSD_JObjSetScaleY(
            &mut *jobj,
            (*layout6c(table_ck(k.xF1[a]))).x08.y * slot(s3c, ef, arg1),
            &mut Dirty,
        );
        mi::HSD_JObjSetScaleZ(
            &mut *jobj,
            (*layout6c(table_ck(k.xF1[a]))).x08.z * slot(s3c, ef, arg1),
            &mut Dirty,
        );
    }

    mirror_to_entity2(arg0, jobj, &mut pos);
}

pub unsafe fn fn_80184138(arg0: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("fn_80184138");
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    let st = &lbl_804735A8;
    let src = st.x4[1];
    let k = &lbl_8047368C;
    let a = arg1 as usize;
    let tb = lbl_804D6604;
    let scl: f32;
    let xoff: f32;
    let yoff: f32;

    mi::HSD_JObjGetTranslation(&*src, &mut pos);
    mi::HSD_JObjSetTranslate(&mut *jobj, &pos, &mut Dirty);

    if k.model_scale_kind == 2 {
        scl = (*team37c(team_ck(k.xF4[a]))).x08;
    } else {
        scl = 1.0;
    }

    if k.model_scale_kind == 2 {
        xoff = (*team37c(team_ck(k.xF4[a]))).x00;
    } else {
        xoff = 0.0;
    }

    if k.model_scale_kind == 2 {
        yoff = (*team37c(team_ck(k.xF4[a]))).x04;
    } else {
        yoff = 0.0;
    }

    if arg1 != 0 {
        mi::HSD_JObjAddTranslationZ(&mut *jobj, (-arg1 * 0x14 + 0xA) as f32, &mut Dirty);
    }

    let s00 = &raw const (*tb).x00 as *const ClassicSlotVals;
    let s18 = &raw const (*tb).x18 as *const ClassicSlotVals;
    let s3c = &raw const (*tb).x3C as *const ClassicSlotVals;
    mi::HSD_JObjAddTranslationX(
        &mut *jobj,
        xoff + (-(*layout6c(table_ck(k.xF4[a]))).x00 + slot(s00, k.xF0 as isize - 1, arg1)),
        &mut Dirty,
    );

    mi::HSD_JObjAddTranslationY(
        &mut *jobj,
        yoff + ((*layout6c(table_ck(k.xF4[a]))).x04 + slot(s18, k.xF0 as isize, arg1)),
        &mut Dirty,
    );

    if k.model_scale_kind == 2 {
        mi::HSD_JObjSetScaleX(&mut *jobj, (*layout6c(table_ck(k.xF4[a]))).x08.x * scl, &mut Dirty);
        mi::HSD_JObjSetScaleY(&mut *jobj, (*layout6c(table_ck(k.xF4[a]))).x08.y * scl, &mut Dirty);
        mi::HSD_JObjSetScaleZ(&mut *jobj, (*layout6c(table_ck(k.xF4[a]))).x08.z * scl, &mut Dirty);
    } else {
        mi::HSD_JObjSetScaleX(
            &mut *jobj,
            (*layout6c(table_ck(k.xF4[a]))).x08.x * slot(s3c, k.xF0 as isize, arg1),
            &mut Dirty,
        );
        mi::HSD_JObjSetScaleY(
            &mut *jobj,
            (*layout6c(table_ck(k.xF4[a]))).x08.y * slot(s3c, k.xF0 as isize, arg1),
            &mut Dirty,
        );
        mi::HSD_JObjSetScaleZ(
            &mut *jobj,
            (*layout6c(table_ck(k.xF4[a]))).x08.z * slot(s3c, k.xF0 as isize, arg1),
            &mut Dirty,
        );
    }

    mirror_to_entity2(arg0, jobj, &mut pos);
}

pub unsafe extern "C" fn fn_801849E0(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801849E0");
    fn_8018325C(gobj, 0);
}
pub unsafe extern "C" fn fn_80184A04(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80184A04");
    fn_8018325C(gobj, 1);
}
pub unsafe extern "C" fn fn_80184A28(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80184A28");
    fn_8018325C(gobj, 2);
}
pub unsafe extern "C" fn fn_80184A4C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80184A4C");
    fn_80184138(gobj, 0);
}
pub unsafe extern "C" fn fn_80184A70(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80184A70");
    fn_80184138(gobj, 1);
}
pub unsafe extern "C" fn fn_80184A94(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80184A94");
    fn_80184138(gobj, 2);
}

#[inline]
unsafe fn tobj_aobj_rate0(jobj: *mut HSD_JObj, frame: f32) {
    let tobj = (*(*(*jobj).u.dobj).mobj).tobj;
    ext::HSD_TObjReqAnimAll(tobj, frame);
    ext::HSD_AObjSetRate((*(*(*(*jobj).u.dobj).mobj).tobj).aobj, 0.0);
}

pub unsafe extern "C" fn fn_80184AB8(arg0: *mut HSD_GObj) {
    melee_core::ct!("fn_80184AB8");
    let mut sp110: *mut HSD_JObj = null_mut();
    let mut sp10 = [0u8; 0x100];
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    let st = &mut lbl_804735A8;
    let k = &mut lbl_8047368C;

    ext::HSD_JObjAnimAll(jobj);

    if (st.x38 as u32) < 0x8C {
        st.x38 = (st.x38 as i32 + 1) as u16;
    } else {
        st.x0 = 1;
        return;
    }

    if k.model_scale_kind != 3 {
        if (k.xEE as u32) < 0xA {
            ext::lb_80011E24(jobj, &mut sp110, 0x12, -1);
            tobj_aobj_rate0(sp110, (k.xEE as i32 % 10) as f32);
            ext::lb_80011E24(jobj, &mut sp110, 0x13, -1);
            tobj_aobj_rate0(sp110, 10.0);
        } else {
            ext::lb_80011E24(jobj, &mut sp110, 0x12, -1);
            tobj_aobj_rate0(sp110, (k.xEE as i32 / 10) as f32);
            ext::lb_80011E24(jobj, &mut sp110, 0x13, -1);
            tobj_aobj_rate0(sp110, (k.xEE as i32 % 10) as f32);
        }

        match st.x38 as i32 {
            0xA => {
                ext::lbAudioAx_800237A8(0x9C4A, 0x7F, 0x40);
                return;
            }
            0x2D => {}
            0x46 => {
                if k.model_scale_kind == 4 || k.model_scale_kind == 1 {
                    lbl_804D6608 = ext::lbAudioAx_800237A8(0x7C863, 0x7F, 0x40);
                    return;
                }
                if k.model_scale_kind == 2 {
                    lbl_804D6608 = ext::lbAudioAx_800237A8(0x7C837, 0x7F, 0x40);
                    return;
                }
                if k.x100[0] != 0 {
                    lbl_804D6608 = ext::lbAudioAx_800237A8(0x7C847, 0x7F, 0x40);
                    return;
                }
            }
            0x64 => {
                if lbl_804D6608 != -1 {
                    ext::lbAudioAx_800236B8(lbl_804D6608);
                }
                ext::gm_80168C5C(k.xF4[0] as u32);
                return;
            }
            0x5 => {
                let tb = lbl_804D6604;
                let mut i = 0i32;
                while i < k.xEF as i32 {
                    if i != 0 || k.xED == 0x78 {
                        ext::fn_80160DE8(
                            st.x4[(7 + i) as usize],
                            k.xF1[i as usize],
                            0,
                            0,
                            (*splash(k.xEF as isize)).x18[i as usize],
                            (*splash(k.xEF as isize)).x24[i as usize],
                        );
                    } else {
                        ext::HSD_SisLib_803A70A0(st.x4[(7 + i) as usize] as *mut HSD_Text, 0, ext::GetNameText(k.xED as i32));
                        ext::HSD_SisLib_803A7548(st.x4[(7 + i) as usize] as *mut HSD_Text, 0, 1.0, 1.0);
                    }
                    i += 1;
                }
                let _ = tb;
                return;
            }
            0x6 => {
                if k.model_scale_kind == 4 {
                    let mut i = 0usize;
                    let mut name = ext::gm_80160980(k.xF4[0]);
                    if ext::lbLang_IsSavedLanguageUS() != 0 {
                        for &b in lbl_803D9414.iter() {
                            if b == 0 {
                                break;
                            }
                            sp10[i] = b;
                            i += 1;
                        }
                        sp10[i] = b' ';
                        i += 1;
                        while *name != 0 {
                            sp10[i] = *name as u8;
                            i += 1;
                            name = name.add(1);
                        }
                        sp10[i] = 0;
                    } else {
                        while *name != 0 {
                            sp10[i] = *name as u8;
                            i += 1;
                            name = name.add(1);
                        }
                        sp10[i] = b' ';
                        i += 1;
                        for &b in lbl_804D40A0.iter() {
                            if b == 0 {
                                break;
                            }
                            sp10[i] = b;
                            i += 1;
                        }
                        sp10[i] = 0;
                    }
                    ext::HSD_SisLib_803A70A0(st.x4[10] as *mut HSD_Text, 0, sp10.as_ptr() as *const c_char);
                    let row1 = splash(1);
                    ext::HSD_SisLib_803A7548(
                        st.x4[10] as *mut HSD_Text,
                        0,
                        0.8f32 * ((*row1).x18[0] * ext::fn_80160F58(k.xF4[0])),
                        (*row1).x24[0],
                    );
                    return;
                }
                let mut i = 0i32;
                while i < k.xF0 as i32 {
                    ext::fn_80160DE8(
                        st.x4[(10 + i) as usize],
                        k.xF4[i as usize],
                        0,
                        1,
                        (*splash(k.xF0 as isize)).x18[i as usize],
                        (*splash(k.xF0 as isize)).x24[i as usize],
                    );
                    i += 1;
                }
            }
            _ => {}
        }
    }
}

pub unsafe extern "C" fn fn_8018504C_cb(_gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8018504C_cb");
    fn_8018504C()
}

pub unsafe fn fn_8018504C() {
    melee_core::ct!("fn_8018504C");
    let st = &mut lbl_804735A8;
    let k = &lbl_8047368C;
    let new_var: i32 = 0x32;
    if (st.x3A as u32) < 0x31 {
        st.x3A = (st.x3A as i32 + 1) as u16;
    } else {
        st.x3A = 0;
    }
    if (st.x3C as u32) < 0x31 {
        st.x3C = (st.x3C as i32 + 1) as u16;
    }
    ext::HSD_JObjReqAnimAll(st.x4[5], (st.x3C as i32 + ((k.xEE as i32 * 0x32) - new_var)) as f32);
    ext::HSD_JObjAnimAll(st.x4[5]);
    ext::HSD_JObjReqAnimAll(st.x4[4], (st.x3A as i32 + ((k.xEE as i32 * 0x32) - new_var)) as f32);
    ext::HSD_JObjAnimAll(st.x4[4]);
    if st.x38 == 0x2D && k.model_scale_kind == 3 {
        match k.game_type {
            1 => {
                ext::lbAudioAx_800237A8(0x7C864, 0x7F, 0x40);
            }
            2 => {
                ext::lbAudioAx_800237A8(0x9C49, 0x7F, 0x40);
            }
            3 => {
                ext::lbAudioAx_800237A8(0x9C47, 0x7F, 0x40);
            }
            _ => {}
        }
    }
}

pub unsafe fn fn_801851C0() -> i32 {
    melee_core::ct!("fn_801851C0");
    let mut pos: Vec3;
    let mut result: i32 = 0;
    let mut i = 0i32;
    while i < lbl_804735E8.xE0 as i32 {
        result = i + 1;
        let k = &lbl_8047368C;
        ext::Player_SetPlayerCharacter(result, k.xF4[0] as i32);
        ext::Player_SetPlayerId(result, 0);
        ext::Player_SetSlottype(result, Gm_PKind_Demo as i32);
        ext::Player_SetFacingDirection(result, 0.0);
        ext::Player_SetCostumeId(result, i);
        pos = gm_1832_CharOffset[table_ck(k.xF4[0]) as usize];
        ext::Player_80032768(result, &mut pos);
        if k.xF4[0] == 0xE {
            pos.x += gm_1832_SamusOffset.x;
            pos.y += gm_1832_SamusOffset.y;
            pos.z += gm_1832_SamusOffset.z;
            ext::Player_80032828(result, 1, &mut pos);
        }
        ext::Player_SetModelScale(result, gm_1832_CharScale[table_ck(k.xF4[0]) as usize]);
        ext::Player_SetFlagsBit5(result, k.x100[0]);
        ext::Player_80037054(result, 6);
        i += 1;
    }

    result
}

pub unsafe extern "C" fn fn_801852FC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801852FC");
    let st = &mut lbl_804735E8;
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj) != 0 {
        let mut i = 0i32;
        while i < st.xE0 as i32 {
            let mut j = 0i32;
            while j < st.xE0 as i32 + 1 {
                if j == 0 || i + 1 != j {
                    ext::Player_800366DC(j, 0);
                } else {
                    ext::Player_800366DC(j, 1);
                }
                j += 1;
            }
            ext::Camera_800313E0(gobj, 1);
            ext::HSD_StateInvalidate(-1);
            ext::HSD_ImageDescCopyFromEFB(&mut st.x40[i as usize], 0x82, 0, 0, 0);
            ext::HSD_ImageDescCopyFromEFB(&mut st.x88[i as usize], 0x82, 0, 1, 1);
            i += 1;
        }
        ext::HSD_CObjEndCurrent();
    }
    st.xE1 = 1;
}

pub unsafe fn fn_80185408(_x: i32, arg8: f32, arg9: f32, arg_a: f32, arg_b: f32) {
    melee_core::ct!("fn_80185408");
    // retail used a 3x4 Mtx the Mtx44 write overran; the PC build uses a 4x4
    let mut sp1c = [[0.0f32; 4]; 4];
    ext::GXSetNumChans(1);
    ext::GXSetChanCtrl(GX_COLOR0A0 as i32, 0, GX_SRC_REG as i32, GX_SRC_VTX as i32, 0, GX_DF_NONE as i32, GX_AF_NONE as i32);
    ext::GXSetNumTexGens(0);
    ext::GXSetZMode(1, GX_LEQUAL as i32, 1);
    ext::GXSetNumTevStages(1);
    ext::GXSetTevOrder(GX_TEVSTAGE0 as i32, GX_TEXCOORD_NULL as i32, GX_TEXMAP_NULL as i32, GX_COLOR_NULL as i32);
    ext::GXSetTevOp(GX_TEVSTAGE0 as i32, GX_PASSCLR as i32);
    ext::GXSetCullMode(GX_CULL_NONE as i32);
    ext::GXSetAlphaCompare(GX_ALWAYS as i32, 0, GX_AOP_OR as i32, GX_ALWAYS as i32, 0);
    ext::GXSetBlendMode(GX_BM_NONE as i32, GX_BL_SRCALPHA as i32, GX_BL_INVSRCALPHA as i32, GX_LO_CLEAR as i32);
    ext::GXSetColorUpdate(0);
    ext::GXClearVtxDesc();
    ext::GXSetVtxAttrFmt(GX_VTXFMT0 as i32, GX_VA_POS as i32, GX_TEX_ST as i32, GX_RGBA6 as i32, 0);
    ext::GXSetVtxDesc(GX_VA_POS as i32, GX_DIRECT as i32);
    ext::GXLoadPosMtxImm(ext::HSD_identityMtx.as_ptr() as *const [f32; 4], 0);
    ext::GXSetCurrentMtx(0);
    ext::MTXOrtho(sp1c.as_mut_ptr(), 0.0, 480.0, 0.0, 640.0, 0.0, 5000.0);
    ext::GXSetProjection(sp1c.as_ptr(), GX_ORTHOGRAPHIC as i32);
    ext::GXBegin(GX_QUADS as i32, GX_VTXFMT0 as i32, 4);
    ext::GXPosition3f32(arg_a, arg8, -4932.0);
    ext::GXPosition3f32(arg_b, arg8, -4932.0);
    ext::GXPosition3f32(arg_b, arg9, -4932.0);
    ext::GXPosition3f32(arg_a, arg9, -4932.0);
    ext::GXSetColorUpdate(1);
    ext::HSD_StateInvalidate(-1);
}

/// Square root through the Gekko `frsqrte` estimate and three Newton steps (the shipping build
/// contracts each `x * (t * t) - 3.0` into an `fma`).
pub unsafe fn fn_801855BC(arg8: f64) -> f64 {
    melee_core::ct!("fn_801855BC");
    if arg8 > 0.0 {
        let step = |t: f64| 0.5 * t * -(arg8.mul_add(t * t, -3.0));
        let temp_f2_2 = melee_core::fp::frsqrte(arg8);
        let temp_f2_3 = step(temp_f2_2);
        let temp_f2_4 = step(temp_f2_3);
        let temp_f2 = step(temp_f2_4);
        return arg8 * (0.5 * temp_f2 * -(arg8.mul_add(temp_f2 * temp_f2, -3.0)));
    }
    if 0.0 == arg8 {
        return 0.0;
    }
    if arg8 != 0.0 {
        return MSL_TrigF_80400770[0] as f64;
    }
    MSL_TrigF_80400774[0] as f64
}

pub unsafe extern "C" fn fn_8018564C(gobj: *mut HSD_GObj, _arg1: i32) {
    melee_core::ct!("fn_8018564C");
    fn_80185408(0xFC8000, 0.0, 480.0, 200.0, 640.0);
    ext::HSD_SObjLib_803A54EC(gobj, 7);
}

pub unsafe extern "C" fn fn_8018569C(gobj: *mut HSD_GObj, _arg1: i32) {
    melee_core::ct!("fn_8018569C");
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj) != 0 {
        let mut i = 0i32;
        while i < lbl_804735E8.xE0 as i32 + 1 {
            if i == 0 {
                ext::Player_800366DC(i, 1);
            } else {
                ext::Player_800366DC(i, 0);
            }
            i += 1;
        }
        ext::Camera_800313E0(gobj, 1);
        (*gobj).gxlink_prios = 0x1001;
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_CObjEndCurrent();
    }
}

pub unsafe extern "C" fn fn_8018575C(gobj: *mut HSD_GObj, _arg1: i32) {
    melee_core::ct!("fn_8018575C");
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj) != 0 {
        ext::Camera_800313E0(gobj, 1);
        (*gobj).gxlink_prios = 0x1001;
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_CObjEndCurrent();
    }
}

/// Creates splash screen sprite objects from pre-rendered character images. Distributes 10
/// image tiles across a grid with random offsets.
pub unsafe extern "C" fn fn_801857C4(arg0: *mut HSD_GObj) {
    melee_core::ct!("fn_801857C4");
    let st = &mut lbl_804735E8;
    if st.xE1 != 0 {
        ext::HSD_GObjFree(lbl_804D65F0);
        let mut delay: u32 = 1;
        for i in 0..10usize {
            let idx = st.xD0[i] as usize;
            let mut desc = HSD_SObjDesc2 { desc: HSD_SObjDesc { image: null_mut(), tlut: null_mut() }, image2: null_mut() };
            desc.desc.image = &mut st.x40[idx];
            desc.desc.tlut = null_mut();
            desc.image2 = &mut st.x88[idx];
            let sobj = ext::HSD_SObjLib_803A477C(st.xDC, &mut desc.desc, 0, 0, 0x80, 1);
            let total_tiles: u32 = 10;
            let num_cols: u64 = fn_801855BC(total_tiles as f64) as i32 as i64 as u64;
            let row = (i as u8 / num_cols as u8) as i32;
            let nc = num_cols as u8;
            let px = 40.0f32.mul_add(
                ext::HSD_Randf(),
                (290.0f32 / (((nc as i32 + 9) / nc as i32) as u8) as f32)
                    .mul_add((i as u8 % nc) as f32, 280.0),
            );
            let py = 40.0f32.mul_add(
                ext::HSD_Randf(),
                (160.0f32 / nc as f32).mul_add(row as f32, -60.0),
            );
            (*sobj).x10 = px;
            (*sobj).x14 = py;
            (*sobj).x48 = delay;
            delay += 8;
            (*sobj).x40 |= 9;
        }
        ext::HSD_GObjFree(arg0);
    }
}

pub unsafe extern "C" fn fn_801859C8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801859C8");
    let mut sobj = (*gobj).hsd_obj as *mut HSD_SObj;

    while !sobj.is_null() {
        if (*sobj).x48 != 0 {
            (*sobj).x48 -= 1;
            if (*sobj).x48 == 0 {
                (*sobj).x40 &= !1;
            }
        }
        sobj = (*sobj).next;
    }
}

pub unsafe fn fn_80185A0C() -> i32 {
    melee_core::ct!("fn_80185A0C");
    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    ext::HSD_SObjLib_803A55DC(gobj, 0x280, 0x1E0, 0xB);
    (*gobj).gxlink_prios = 0x20000;
    ext::HSD_GObjGXLink_8039084C(gobj);
    ext::GObj_SetupGXLinkMax(gobj, Some(fn_8018564C), 0xB);

    let gobj2 = ext::GObj_Create(0xE, 0xF, 0);
    ext::HSD_GObjObject_80390A70(gobj2, ext::HSD_SObjLib_804D7960, null_mut());
    ext::GObj_SetupGXLink(gobj2, Some(ext::HSD_SObjLib_803A49E0), 0x11, 0);
    let proc = ext::HSD_GObj_SetupProc(gobj2, Some(fn_801859C8), 0);
    (*proc).set_flags_3(ext::HSD_GObj_804D783C as u8);

    let count = ext::gm_GetNumCostumesForCKind(lbl_8047368C.xF4[0]);
    lbl_804735E8.xE0 = if (count as i32) > 3 { 3 } else { count };

    let st = &mut lbl_804735E8;
    st.xDC = gobj2;

    // fn_80185A0C_Tail
    st.xE1 = 0;
    let n = st.xE0 as usize;
    for i in 0..n {
        st.x40[i].image_ptr = null_mut();
        ext::lb_800121FC(&mut st.x40[i], 0x17C, 0x190, GX_TF_RGB5A3 as i32, 0);
        // `img[3]` is `x88[i]`
        st.x88[i].image_ptr = null_mut();
        ext::lb_800121FC(&mut st.x88[i], 0x17C, 0x190, GX_TF_Z24X8 as i32, 0);
    }

    for k in 0..10i32 {
        if (k / n as i32) % 2 != 0 {
            st.xD0[k as usize] = ((n as i32 - 1) - (k % n as i32)) as u8;
        } else {
            st.xD0[k as usize] = (k % n as i32) as u8;
        }
    }

    ext::HSD_GObj_SetupProc(ext::GObj_Create(0x13, 1, 0), Some(fn_801857C4), 0);

    let gobj3 = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::HSD_CObjLoadDesc((*(*lbl_804D6600).cameras).desc);
    ext::HSD_GObjObject_80390A70(gobj3, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    // `(GObj_RenderFunc) (Event) fn_801852FC`: called with a surplus argument
    let render: unsafe extern "C" fn(*mut HSD_GObj, i32) = core::mem::transmute(fn_801852FC as unsafe extern "C" fn(*mut HSD_GObj));
    ext::GObj_SetupGXLinkMax(gobj3, Some(render), 0);
    (*gobj3).gxlink_prios = 0x61;
    lbl_804D65F0 = gobj3;

    fn_801851C0()
}

pub unsafe fn fn_80185D64() {
    melee_core::ct!("fn_80185D64");
    ext::ftDemo_ObjAllocInit();
    ext::Player_InitAllPlayers();

    let k = &mut lbl_8047368C;
    let mut i = 0usize;
    while (i as i32) < k.xEF as i32 {
        let chr = k.xF1[i];
        if chr as i32 != ChKind_Max as i32 {
            ext::Player_80036E20(chr as i32, lbl_804D65F4, 1);
        }
        i += 1;
    }

    i = 0;
    while (i as i32) < k.xF0 as i32 {
        if k.xF4[i] as i32 == CKind_GKoops as i32 {
            k.xF4[i] = CKind_Koopa as u8;
        }
        let chr = k.xF4[i];
        if chr as i32 != ChKind_Max as i32 {
            ext::Player_80036E20(chr as i32, lbl_804D65F4, 1);
        }
        i += 1;
    }
}

pub unsafe fn fn_80185E34() -> i32 {
    melee_core::ct!("fn_80185E34");
    let local = lbl_803B7C28;
    let k = &lbl_8047368C;
    let mut player_slot = 0i32;

    let mut i = 0usize;
    while (i as i32) < k.xEF as i32 {
        // m-ex null-id shift: the external null id is ChKind_None
        if k.xF1[i] as i32 != ChKind_None as i32 {
            ext::Player_80036CF0(player_slot);
            ext::Player_SetPlayerCharacter(player_slot, k.xF1[i] as i32);
            ext::Player_SetCostumeId(player_slot, k.xF7[i] as i32);
            ext::Player_SetPlayerId(player_slot, 0);
            ext::Player_SetSlottype(player_slot, Gm_PKind_Demo as i32);
            ext::Player_SetFacingDirection(player_slot, 0.0);
            ext::Player_SetModelScale(player_slot, 1.0);
            ext::Player_SetFlagsBit5(player_slot, k.xFD[i]);
            ext::Player_80036F34(player_slot, 5);
            ext::HSD_GObj_SetupProc(ext::Player_GetEntity(player_slot), local[i], 0x16);
            player_slot += 1;
        }
        i += 1;
    }
    player_slot
}

pub unsafe fn fn_80185F5C(mut arg0: i32) {
    melee_core::ct!("fn_80185F5C");
    let local = lbl_803B7C40;
    let k = &lbl_8047368C;
    let mut i = 0usize;

    while (i as i32) < k.xF0 as i32 {
        // m-ex null-id shift: the external null id is ChKind_None
        if k.xF1[i] as i32 != ChKind_None as i32 {
            ext::Player_80036CF0(arg0);
            ext::Player_SetPlayerCharacter(arg0, k.xF4[i] as i32);
            ext::Player_SetCostumeId(arg0, k.xFA[i] as i32);
            ext::Player_SetPlayerId(arg0, 0);
            ext::Player_SetSlottype(arg0, Gm_PKind_Demo as i32);
            ext::Player_SetFacingDirection(arg0, 0.0);
            ext::Player_SetModelScale(arg0, 1.0);
            ext::Player_SetFlagsBit5(arg0, k.x100[i]);
            ext::Player_80036F34(arg0, 6);
            ext::HSD_GObj_SetupProc(ext::Player_GetEntity(arg0), local[i + 3], 0x16);
            arg0 += 1;
        }
        i += 1;
    }
}

pub unsafe fn fn_80186080() {
    melee_core::ct!("fn_80186080");
    let st = &mut lbl_804735A8;
    let k = &lbl_8047368C;
    let temp_r31 = ext::HSD_SisLib_803A611C(0, null_mut(), 9, 0x14, 0, 0xE, 0, 0x12);
    ext::HSD_SisLib_803A62A0(0, c"SdIntro.dat".as_ptr() as *mut c_char, c"SIS_IntroData".as_ptr() as *mut c_char);
    st.x4[6] = ext::HSD_SisLib_803A5ACC(0, temp_r31, 0.0, 0.0, 0.0, 640.0, 480.0) as *mut HSD_JObj;
    let text = st.x4[6] as *mut HSD_Text;
    if k.game_type == 1 {
        if ext::lbLang_IsSavedLanguageUS() != 0 {
            ext::HSD_SisLib_803A6368(text, 5);
            return;
        }
        ext::HSD_SisLib_803A6368(text, 2);
    } else if k.game_type == 3 {
        if ext::lbLang_IsSavedLanguageUS() != 0 {
            ext::HSD_SisLib_803A6368(text, 6);
            return;
        }
        ext::HSD_SisLib_803A6368(text, 3);
    } else if k.game_type == 2 {
        if ext::lbLang_IsSavedLanguageUS() != 0 {
            ext::HSD_SisLib_803A6368(text, 7);
            return;
        }
        ext::HSD_SisLib_803A6368(text, 4);
    }
}

pub unsafe fn fn_801861B8() {
    melee_core::ct!("fn_801861B8");
    let st = &mut lbl_804735A8;
    let k = &lbl_8047368C;

    ext::HSD_SisLib_803A611C(0, null_mut(), 9, 0x14, 0, 0xE, 0, 0x12);
    ext::HSD_SisLib_803A62A0(0, c"SdIntro.dat".as_ptr() as *mut c_char, c"SIS_IntroData".as_ptr() as *mut c_char);

    let mut i = 0i32;
    while i < k.xEF as i32 {
        let s = (7 + i) as usize;
        st.x4[s] = ext::HSD_SisLib_803A6754(0, 0) as *mut HSD_JObj;
        (*(st.x4[s] as *mut HSD_Text)).default_kerning = 1;
        let text = st.x4[s] as *mut HSD_Text;
        (*text).font_size.x = 1.0;
        (*text).font_size.y = 1.0;
        (*(st.x4[s] as *mut HSD_Text)).default_alignment = 1;
        let row = splash(k.xEF as isize);
        ext::HSD_SisLib_803A6B98(st.x4[s] as *mut HSD_Text, (*row).x00[i as usize], (*row).x0C[i as usize], c" ".as_ptr());
        ext::HSD_SisLib_803A7548(st.x4[s] as *mut HSD_Text, 0, (*row).x18[i as usize], (*row).x24[i as usize]);
        i += 1;
    }

    i = 0;
    while i < k.xF0 as i32 {
        let s = (10 + i) as usize;
        st.x4[s] = ext::HSD_SisLib_803A6754(0, 0) as *mut HSD_JObj;
        (*(st.x4[s] as *mut HSD_Text)).default_kerning = 1;
        let text = st.x4[s] as *mut HSD_Text;
        (*text).font_size.x = 1.0;
        (*text).font_size.y = 1.0;
        (*(st.x4[s] as *mut HSD_Text)).default_alignment = 1;
        if k.model_scale_kind == 4 {
            let r1 = splash(1);
            ext::HSD_SisLib_803A6B98(st.x4[s] as *mut HSD_Text, 340.0 + (*r1).x00[0], (*r1).x0C[0], c" ".as_ptr());
            ext::HSD_SisLib_803A7548(st.x4[s] as *mut HSD_Text, 0, 340.0 + (*r1).x18[0], (*r1).x24[0]);
            return;
        }
        let row = splash(k.xF0 as isize);
        ext::HSD_SisLib_803A6B98(
            st.x4[s] as *mut HSD_Text,
            340.0 + (*row).x00[i as usize],
            (*row).x0C[i as usize],
            c" ".as_ptr(),
        );
        ext::HSD_SisLib_803A7548(st.x4[s] as *mut HSD_Text, 0, 340.0 + (*row).x18[i as usize], (*row).x24[i as usize]);
        i += 1;
    }
}

pub unsafe fn fn_80186400() {
    melee_core::ct!("fn_80186400");
    let gobj = ext::GObj_Create(0xE, 0xF, 0);
    let models0 = *(*lbl_804D65FC).models;
    let jobj = ext::HSD_JObjLoadJoint((*models0).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_80184AB8), 0);
    ext::gm_8016895C(jobj, models0, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    if lbl_8047368C.model_scale_kind != 3 {
        let st = &mut lbl_804735A8;
        ext::lb_80011E24(jobj, &mut st.x4[0], 4, -1);
        ext::lb_80011E24(jobj, &mut st.x4[1], 5, -1);
        ext::lb_80011E24(jobj, &mut st.x4[2], 0x3A, -1);
        ext::lb_80011E24(jobj, &mut st.x4[3], 0x3B, -1);
        mi::HSD_JObjSetTranslateZ(&mut *st.x4[2], 10000.0, &mut Dirty);
        mi::HSD_JObjSetTranslateZ(&mut *st.x4[3], 10000.0, &mut Dirty);
    }
}

/// `gm_80186634_SetupLight`
unsafe fn setup_light() {
    let gobj = ext::GObj_Create(0xB, 3, 0);
    let list = ext::lb_80011AC4((*lbl_804D65FC).lights);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_LightKind as u8, list as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_LObjCallback), 0, 0);
}

/// `gm_80186634_SetupCamera`
unsafe fn setup_camera() {
    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::HSD_CObjLoadDesc((*(*lbl_804D65FC).cameras).desc);
    let cobj2 = ext::HSD_CObjLoadDesc((*(*lbl_804D6600).cameras).desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj, Some(ext::HSD_GObj_803910D8), 8);
    (*gobj).gxlink_prios = 0x801;
    let gobj2 = ext::GObj_Create(0x13, 0x15, 0);
    ext::HSD_GObjObject_80390A70(gobj2, ext::HSD_GObj_CameraKind, cobj2 as *mut c_void);
    if lbl_8047368C.model_scale_kind == 4 {
        ext::GObj_SetupGXLinkMax(gobj2, Some(fn_8018569C), 8);
    } else {
        ext::GObj_SetupGXLinkMax(gobj2, Some(fn_8018575C), 8);
    }
}

/// `gm_80186634_SetupModel`
unsafe fn setup_model() {
    let gobj = ext::GObj_Create(0xE, 0xF, 0);
    let models0 = *(*lbl_804D6600).models;
    let jobj = ext::HSD_JObjLoadJoint((*models0).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xC, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_8018504C_cb), 0x11);
    ext::gm_8016895C(jobj, models0, 0);
    ext::HSD_JObjReqAnimAll(jobj, ((lbl_8047368C.xEE as i32 - 1) * 0x32) as f32);
    ext::HSD_JObjAnimAll(jobj);
    let st = &mut lbl_804735A8;
    ext::lb_80011E24(jobj, &mut st.x4[4], 0xE, -1);
    ext::lb_80011E24(jobj, &mut st.x4[5], 1, -1);
}

/// `gm_80186634_SetupFog`
unsafe fn setup_fog() {
    let gobj = ext::GObj_Create(0xE, 0xF, 0);
    let fog = ext::HSD_FogLoadDesc((*(*lbl_804D65FC).fogs).desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_FogCallback), 0xB, 0);
}

pub unsafe fn fn_80186634(_arg0: *mut c_void) {
    melee_core::ct!("fn_80186634");
    ext::lbArchive_80016DBC(
        c"GmIntEz.dat".as_ptr(),
        &[
            &raw mut lbl_804D6604 as *mut c_void,
            c"gmIntroEasyTable".as_ptr() as *mut c_void,
        ],
    );
    ext::Camera_Init(0xC);
    ext::lb_8000FCDC();
    ext::mpColl_80041C78();
    ext::Ground_801C0378(0x40);
    ext::Stage_802251E8(St_Kind_Dummy as i32, null_mut());
    ext::Item_80266FA8();
    ext::Item_80266FCC();
    ext::Stage_8022524C();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);

    lbl_804D65F4 = ext::lbArchive_80016DBC(
        lbl_803B7C58[lbl_8047368C.game_type as usize].as_ptr(),
        &[
            &raw mut lbl_804D65FC as *mut c_void,
            lbl_803D9444.as_ptr() as *mut c_void,
        ],
    );
    lbl_804D65F8 = ext::lbArchive_80016DBC(
        lbl_804D40B0.as_ptr(),
        &[
            &raw mut lbl_804D6600 as *mut c_void,
            lbl_803D9444.as_ptr() as *mut c_void,
        ],
    );

    setup_light();

    if lbl_8047368C.model_scale_kind != 3 {
        lbl_8047368C.game_type = 0;
    }

    match lbl_8047368C.model_scale_kind {
        3 => {
            setup_camera();
            fn_80186400();
            setup_model();
            setup_fog();
            fn_80186080();
        }
        0 | 1 | 2 => {
            setup_camera();
            fn_80186400();
            setup_model();
            setup_fog();
            fn_801861B8();
            fn_80185D64();
            fn_80185F5C(fn_80185E34());
        }
        4 => {
            setup_camera();
            fn_80186400();
            setup_model();
            setup_fog();
            fn_801861B8();
            fn_80185D64();
            fn_80185A0C();
            fn_80185E34();
        }
        _ => {}
    }

    ext::lbAudioAx_80023F28(0x2D);
}

pub unsafe fn gm_Scene_IntroEasy_OnFrame() {
    melee_core::ct!("gm_Scene_IntroEasy_OnFrame");
    if lbl_804735A8.x0 != 0 {
        ext::lbAudioAx_800236DC();
        ext::gm_801A4B60();
    }
}

/// Classic Mode intro scene enter data (0x20 bytes); `x10`/`x11` are the first two bytes of the
/// word at 0x10.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClassicModeEnterData {
    pub x0: i32,
    pub x4: i32,
    pub x8: i32,
    pub xC: i32,
    pub x10_bytes: [u8; 4],
    pub x14: i32,
    pub x18: i32,
    pub x1C: i32,
}
melee_core::gc_layout!(ClassicModeEnterData, "ClassicModeEnterData", size 0x20, align 4, [
    x0 @ 0x0: 4 "int",
    x4 @ 0x4: 4 "int",
    x8 @ 0x8: 4 "int",
    xC @ 0xC: 4 "int",
    x10_bytes @ 0x10: 4 "int",
    x14 @ 0x14: 4 "int",
    x18 @ 0x18: 4 "int",
    x1C @ 0x1C: 4 "int",
]);

pub unsafe fn gm_Scene_IntroEasy_OnEnter(arg0_: *mut c_void) {
    melee_core::ct!("gm_Scene_IntroEasy_OnEnter");
    let arg0 = arg0_ as *const ClassicModeEnterData;

    if (*arg0).x10_bytes[0] == 0x1A || (*arg0).x10_bytes[1] == 0x1A {
        lbl_804735A8.x0 = 1;
        return;
    }

    lbl_804735A8.x0 = 0;
    lbl_804735A8.x38 = 0;
    lbl_804735A8.x3A = 0;
    lbl_804735A8.x3C = 0;
    // `*(ClassicModeEnterData*) &lbl_8047368C.model_scale_kind = *arg0;` (the first 0x20 bytes)
    core::ptr::copy_nonoverlapping(arg0 as *const u8, &raw mut lbl_8047368C as *mut u8, 0x20);
    fn_80186634(arg0_);
    lbl_804D6608 = -1;
    ext::gm_80167858(lbl_8047368C.xEC as i8 as i32, lbl_8047368C.xED as i32, 0xB, 0x2D);
    ext::gm_80168F88();
}

// ---- folded from gm_r10.rs (consolidation steps 21/22) ----
/// gm_1832.c `fn_801859C8_fp`: counts every SObj's `x48` timer down, clearing bit 0 of `x40`
/// when it reaches 0.
///
/// # Safety
/// `gobj.hsd_obj` must be NULL or a valid NULL-terminated HSD_SObj_gm_r10 chain.
pub unsafe fn fn_801859C8_fp(gobj: &HSD_GObj_gm_r10) {
    melee_core::ct!("fn_801859C8_fp");
    let mut sobj = gobj.hsd_obj as *mut HSD_SObj_gm_r10;
    while let Some(s) = sobj.as_mut() {
        if s.x48 != 0 {
            s.x48 = s.x48.wrapping_sub(1);
            if s.x48 == 0 {
                s.x40 &= !1;
            }
        }
        sobj = s.next;
    }
}
