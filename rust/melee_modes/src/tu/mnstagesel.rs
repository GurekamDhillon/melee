//! src/melee/mn/mnstagesel.c: the stage-select screen: the icon grid (`mnStageSel_803F06D0`),
//! hover / cursor animation, the Random picker, the preview models, and the scene entry /
//! per-frame / exit procs. Includes the TARGET_PC m-ex stage-select expansion (icon table
//! read from `mexData`, paging with L/R, the added icon models) and the PC stage-art helpers
//! (`mnStageSel_PcArt*`).
//! Callees in TUs that are not ported yet are `ext` stubs named like the C function.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_mut, unused_variables, unused_assignments, clippy::missing_safety_doc)]

use crate::mn_inl::*;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

/// `MexMapPages` (mnstagesel.c): the MnSlMap `mexMapData` paging block.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MexMapPages {
    pub jobj: *mut HSD_JObj,
    pub count: i32,
    pub current: i32,
    pub unk_C: i32,
    pub anims: *mut *mut HSD_AnimJoint,
}

melee_core::gc_layout!(MexMapPages, "MexMapPages", size 0x14, align 4, [
    jobj @ 0x0: 4 "HSD_JObj*",
    count @ 0x4: 4 "int",
    current @ 0x8: 4 "int",
    unk_C @ 0xC: 4 "int",
    anims @ 0x10: 4 "HSD_AnimJoint**",
]);

/// `MexMapData` (mnstagesel.c): the MnSlMap archive's `mexMapData` public symbol.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MexMapData {
    pub icon_joint: *mut HSD_Joint,
    pub icon_animjoint: *mut HSD_AnimJoint,
    pub icon_matanim: *mut HSD_MatAnimJoint,
    pub pos_joint: *mut HSD_Joint,
    pub pos_animjoint: *mut HSD_AnimJoint,
    pub stagename_matanim: *mut HSD_MatAnimJoint,
    pub pages: *mut MexMapPages,
}

melee_core::gc_layout!(MexMapData, "MexMapData", size 0x1C, align 4, [
    icon_joint @ 0x0: 4 "HSD_Joint*",
    icon_animjoint @ 0x4: 4 "HSD_AnimJoint*",
    icon_matanim @ 0x8: 4 "HSD_MatAnimJoint*",
    pos_joint @ 0xC: 4 "HSD_Joint*",
    pos_animjoint @ 0x10: 4 "HSD_AnimJoint*",
    stagename_matanim @ 0x14: 4 "HSD_MatAnimJoint*",
    pages @ 0x18: 4 "MexMapPages*",
]);

/// `MexSssIcon` (mnstagesel.c): m-ex's own icon row as it sits in MxDt.dat (0x20 bytes).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MexSssIcon {
    pub runtime_joint: u32,
    pub runtime_cooldown: i32,
    pub type_0: u8,
    pub preview_id: u8,
    pub random_id: u8,
    pub pad: u8,
    pub cursor_w: f32,
    pub cursor_h: f32,
    pub cursor_scale_x: f32,
    pub cursor_scale_y: f32,
    pub stkind: i32,
}

melee_core::gc_layout!(MexSssIcon, "MexSssIcon", size 0x20, align 4, [
    runtime_joint @ 0x0: 4 "u32",
    runtime_cooldown @ 0x4: 4 "s32",
    type_0 @ 0x8: 1 "u8",
    preview_id @ 0x9: 1 "u8",
    random_id @ 0xA: 1 "u8",
    pad @ 0xB: 1 "u8",
    cursor_w @ 0xC: 4 "f32",
    cursor_h @ 0x10: 4 "f32",
    cursor_scale_x @ 0x14: 4 "f32",
    cursor_scale_y @ 0x18: 4 "f32",
    stkind @ 0x1C: 4 "s32",
]);

/// The function-local `struct StageSelUserData { int x0; }` of `fn_8025A560`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StageSelUserData_0 {
    pub x0: i32,
}

melee_core::gc_layout!(StageSelUserData_0, "StageSelUserData_0", size 4, align 4, [
    x0 @ 0x0: 4 "int",
]);

/// The function-local `struct foo { int x0, x4; }` of `mnStageSel_Scene_OnEnter`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct foo {
    pub x0: i32,
    pub x4: i32,
}

melee_core::gc_layout!(foo, "foo", size 8, align 4, [
    x0 @ 0x0: 4 "int",
    x4 @ 0x4: 4 "int",
]);

/// The anonymous `struct { u32 x0; u32 x4; }` user data of `fn_8025A090`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct C2RustUnnamed_15 {
    pub x0: u32,
    pub x4: u32,
}

melee_core::gc_layout!(C2RustUnnamed_15, "C2RustUnnamed_15", size 8, align 4, [
    x0 @ 0x0: 4 "u32",
    x4 @ 0x4: 4 "u32",
]);

/// The anonymous struct `mnStageSel_Scene_OnEnter` casts the MnSlMap public symbols to
/// (camera / light / fog descriptors, then the model table).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct C2RustUnnamed_16 {
    pub unk0: *mut HSD_CObjDesc,
    pub unk4: *mut HSD_LightDesc,
    pub unk8: *mut HSD_LightDesc,
    pub unkC: *mut HSD_FogDesc,
    pub x10: mnStageSel_804D6C98_t,
}

melee_core::gc_layout!(C2RustUnnamed_16, "C2RustUnnamed_16", size 0xD0, align 4, [
    unk0 @ 0x0: 4 "HSD_CObjDesc*",
    unk4 @ 0x4: 4 "HSD_LightDesc*",
    unk8 @ 0x8: 4 "HSD_LightDesc*",
    unkC @ 0xC: 4 "HSD_FogDesc*",
    x10 @ 0x10: 192 "mnStageSel_804D6C98_t",
]);

/// The anonymous struct `mnStageSel_PcArtOpen` casts the archive's symbol to.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct C2RustUnnamed_18 {
    pub unk0: *mut c_void,
    pub unk4: *mut c_void,
    pub unk8: *mut c_void,
    pub unkC: *mut c_void,
    pub x10: mnStageSel_804D6C98_t,
}

melee_core::gc_layout!(C2RustUnnamed_18, "C2RustUnnamed_18", size 0xD0, align 4, [
    unk0 @ 0x0: 4 "void*",
    unk4 @ 0x4: 4 "void*",
    unk8 @ 0x8: 4 "void*",
    unkC @ 0xC: 4 "void*",
    x10 @ 0x10: 192 "mnStageSel_804D6C98_t",
]);

pub mod ext {
    use super::*;
    use crate::ext_fns;
    ext_fns! {
        extern "C" fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32);
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        extern "C" fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32);
        extern "C" fn HSD_GObj_LObjCallback(gobj: *mut HSD_GObj, unused: i32);
        fn lbAudioAx_80023F28(a0: i32) -> i32;
        extern "C" fn mn_8022BA1C(a0: *mut HSD_GObj);
        extern "C" fn mn_8022EAE0(a0: *mut HSD_GObj);
        fn mn_8022F218() -> i32;
        fn mn_8022F268();
        extern "C" fn HSD_AObjStopAnim(aobj: *mut HSD_AObj, obj: *mut c_void, func: HSD_ObjUpdateFunc);
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
        fn HSD_JObjGetDObj(jobj: *mut HSD_JObj) -> *mut HSD_DObj;
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
        fn HSD_JObjSetFlags(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjPrependRObj(jobj: *mut HSD_JObj, robj: *mut HSD_RObj);
        fn HSD_JObjReqAnimAllByFlags(jobj: *mut HSD_JObj, flags: u32, frame: f32);
        fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const c_char) -> *mut c_void;
        fn HSD_CObjLoadDesc(desc: *mut HSD_CObjDesc) -> *mut HSD_CObj;
        fn gm_80164330(a0: i32) -> i32;
        fn gm_80164430(a0: u16) -> i32;
        fn gm_ChangeGameModeAfterCurrentScene(pending_mode: i32);
        fn gm_801A4B60();
        fn gmMainLib_8015ECB0() -> i32;
        fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3);
        fn lb_8000C1C0(a0: *mut HSD_JObj, constraint: *mut HSD_JObj);
        fn lb_8000C2F8(a0: *mut HSD_JObj, constraint: *mut HSD_JObj);
        fn lb_800145F4();
        fn lbArchive_LoadArchive(filename: *const c_char) -> *mut HSD_Archive;
        fn lbArchive_80016EFC(a0: *mut HSD_Archive);
        fn lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState;
        fn lbDvd_80018254();
        fn lbLang_IsSavedLanguageUS() -> i32;
        fn HSD_FogSet(a0: *mut HSD_Fog);
        fn HSD_FogLoadDesc(desc: *mut HSD_FogDesc) -> *mut HSD_Fog;
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32);
        fn GObj_SetupGXLinkMax(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, priority: u32);
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void);
        fn HSD_GObjFree(a0: *mut HSD_GObj);
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void) -> ()>, data: *mut c_void, );
        fn HSD_LObjLoadDesc(ldesc: *mut HSD_LightDesc) -> *mut HSD_LObj;
        extern "C" fn HSD_Free(ptr: *mut c_void);
        fn HSD_MemAlloc(size: isize) -> *mut c_void;
        fn HSD_Randi(max_val: i32) -> i32;
        fn HSD_RObjAlloc() -> *mut HSD_RObj;
        fn HSD_RObjSetFlags(robj: *mut HSD_RObj, flags: u32);
        fn HSD_RObjSetConstraintObj(robj: *mut HSD_RObj, obj: *mut c_void);
        fn Mex_SssIconCount() -> i32;
        extern "C" fn Mex_SssTable() -> *mut c_void;
        fn Mex_GrKindForExt(ext: i32) -> i32;
        fn Mex_GrFile(grkind: i32) -> *const c_char;
    }
    pub static mut HSD_GObj_FogKind: i8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_LightKind: i8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_CameraKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut MenMain_cam: *mut HSD_CObjDesc = unsafe { core::mem::zeroed() };
    pub static mut HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
}
use ext::*;

pub const NUM_STAGES_RETAIL: i32 = 29;
pub const SSS_ICON_COUNT_RETAIL: i32 = NUM_STAGES_RETAIL
    + 1 as i32;
pub const SSS_ICON_MAX: i32 = 256;
static mut mnStageSel_NumStages: i32 = NUM_STAGES_RETAIL;
static mut mnStageSel_IconCount: i32 = SSS_ICON_COUNT_RETAIL;
static mut mnStageSel_MexActive: i32 = 0;
pub const SSS_EXT_MEX_FIRST: i32 = 288;
pub const SSS_RANDOM_ID_COUNT: i32 = 30;
pub const MAX_ITER: i32 = 100000;
pub static mut mnStageSel_803F06D0: [stagelistinfo; 256] = [
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0 as u8,
        xA: 0 as u8,
        stkind: 0x4 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x1 as u8,
        xA: 0xc as u8,
        stkind: 0xb as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x2 as u8,
        xA: 0x1 as u8,
        stkind: 0x5 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x3 as u8,
        xA: 0xd as u8,
        stkind: 0xc as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x4 as u8,
        xA: 0x2 as u8,
        stkind: 0xd as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x5 as u8,
        xA: 0xe as u8,
        stkind: 0xe as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x8 as u8,
        xA: 0x4 as u8,
        stkind: 0x8 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x9 as u8,
        xA: 0x10 as u8,
        stkind: 0x10 as i32,
        xC: 2.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0xa as u8,
        xA: 0x5 as u8,
        stkind: 0x2 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0xb as u8,
        xA: 0x11 as u8,
        stkind: 0x11 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0xc as u8,
        xA: 0x6 as u8,
        stkind: 0x7 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0xd as u8,
        xA: 0x12 as u8,
        stkind: 0x16 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x6 as u8,
        xA: 0x3 as u8,
        stkind: 0x6 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x7 as u8,
        xA: 0xf as u8,
        stkind: 0xf as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x12 as u8,
        xA: 0x9 as u8,
        stkind: 0x9 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x13 as u8,
        xA: 0x15 as u8,
        stkind: 0x12 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x10 as u8,
        xA: 0x8 as u8,
        stkind: 0xa as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x11 as u8,
        xA: 0x14 as u8,
        stkind: 0x18 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0xe as u8,
        xA: 0x7 as u8,
        stkind: 0x3 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0xf as u8,
        xA: 0x13 as u8,
        stkind: 0x17 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x14 as u8,
        xA: 0xb as u8,
        stkind: 0x13 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x15 as u8,
        xA: 0x16 as u8,
        stkind: 0x14 as i32,
        xC: 3.1_f32,
        x10: 2.7_f32,
        x14: 1.0_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x16 as u8,
        xA: 0xa as u8,
        stkind: 0x19 as i32,
        xC: 3.1_f32,
        x10: 2.9_f32,
        x14: 1.0_f32,
        x18: 1.1_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x17 as u8,
        xA: 0x17 as u8,
        stkind: 0x1b as i32,
        xC: 3.1_f32,
        x10: 2.9_f32,
        x14: 1.0_f32,
        x18: 1.1_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x18 as u8,
        xA: 0x18 as u8,
        stkind: 0x1f as i32,
        xC: 2.9_f32,
        x10: 2.1_f32,
        x14: 0.8_f32,
        x18: 0.8_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x19 as u8,
        xA: 0x19 as u8,
        stkind: 0x20 as i32,
        xC: 2.9_f32,
        x10: 2.1_f32,
        x14: 0.8_f32,
        x18: 0.8_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x1a as u8,
        xA: 0x1a as u8,
        stkind: 0x1c as i32,
        xC: 2.9_f32,
        x10: 2.1_f32,
        x14: 0.8_f32,
        x18: 0.8_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x1b as u8,
        xA: 0x1b as u8,
        stkind: 0x1d as i32,
        xC: 2.9_f32,
        x10: 2.1_f32,
        x14: 0.8_f32,
        x18: 0.8_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x1c as u8,
        xA: 0x1c as u8,
        stkind: 0x1e as i32,
        xC: 2.9_f32,
        x10: 2.1_f32,
        x14: 0.8_f32,
        x18: 0.8_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0 as i32,
        x8: 0x2 as u8,
        x9: 0x1d as u8,
        xA: 0 as u8,
        stkind: 0 as i32,
        xC: 3.6_f32,
        x10: 2.7_f32,
        x14: 1.2_f32,
        x18: 1.0_f32,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
    stagelistinfo {
        x0: null_mut(),
        x4: 0,
        x8: 0,
        x9: 0,
        xA: 0,
        stkind: 0,
        xC: 0.,
        x10: 0.,
        x14: 0.,
        x18: 0.,
    },
];
static mut mnStageSel_804D50A0: i8 = -1 as i8;
static mut sss_data: *mut SSSData = null_mut();
static mut mnStageSel_804D6C94: *mut HSD_Archive = null_mut();
static mut mnStageSel_804D6C98: *mut mnStageSel_804D6C98_t = core::ptr::null::<
    mnStageSel_804D6C98_t,
>() as *mut mnStageSel_804D6C98_t;
static mut mnStageSel_804D6C9C: *mut HSD_GObj = null_mut();
static mut mnStageSel_804D6CA0: u32 = 0;
static mut mnStageSel_804D6CA4: u32 = 0;
static mut mnStageSel_804D6CA8: i32 = 0;
static mut mnStageSel_804D6CAC: i8 = 0;
static mut mnStageSel_804D6CAD: i8 = 0;
static mut mnStageSel_804D6CAE: i32 = 0;
static mut mnStageSel_804D6CAF: u8 = 0;
unsafe extern "C" fn HSD_LObjSetNext(mut lobj: *mut HSD_LObj, mut next: *mut HSD_LObj) {
    if lobj.is_null() {
        __assert("src/sysdolphin/baselib/lobj.h", 194, "lobj");
    };
    (*lobj).next = next;
}
pub const TOBJ_TLUT_NONE: i32 = -1;
static mut mnStageSel_MexMap: *mut MexMapData = null_mut();
unsafe extern "C" fn sss_random_ok(mut i: i32) -> i32 {
    if mnStageSel_MexActive != 0 {
        if (mnStageSel_803F06D0[i as usize].x8 as i32)
            < 2
            || mnStageSel_803F06D0[i as usize].x8 as i32
                == 3
        {
            return 0;
        }
        if mnStageSel_803F06D0[i as usize].stkind >= SSS_EXT_MEX_FIRST {
            return 0;
        }
        if mnStageSel_803F06D0[i as usize].xA as i32
            >= SSS_RANDOM_ID_COUNT
        {
            return 0;
        }
    }
    return (gm_80164330(mnStageSel_803F06D0[i as usize].xA as i32)
        != 0) as i32;
}
unsafe extern "C" fn sss_has_preview(mut i: i32) -> i32 {
    if mnStageSel_MexActive == 0 {
        return (i < mnStageSel_NumStages) as i32;
    }
    return (i >= 0 && i < mnStageSel_IconCount
        && mnStageSel_803F06D0[i as usize].x8 as i32
            >= 2
        && mnStageSel_803F06D0[i as usize].x8 as i32
            != 3
        && mnStageSel_803F06D0[i as usize].x9 as i32
            != 0xff) as i32;
}
unsafe extern "C" fn mnStageSel_MexSetup(mut archive: *mut HSD_Archive) {
    let mut tbl: *mut MexSssIcon = null_mut();
    let mut n: i32 = 0;
    let mut i: i32 = 0;
    mnStageSel_MexMap = null_mut();
    mnStageSel_MexActive = 0;
    mnStageSel_NumStages = NUM_STAGES_RETAIL;
    mnStageSel_IconCount = SSS_ICON_COUNT_RETAIL;
    mnStageSel_MexMap = HSD_ArchiveGetPublicAddress(
        archive,
        c"mexMapData".as_ptr(),
    ) as *mut MexMapData;
    if mnStageSel_MexMap.is_null() || (*mnStageSel_MexMap).icon_joint.is_null()
        || (*mnStageSel_MexMap).pos_joint.is_null()
    {
        mnStageSel_MexMap = null_mut();
        return;
    }
    n = Mex_SssIconCount();
    tbl = (if n > SSS_ICON_COUNT_RETAIL && n <= SSS_ICON_MAX {
        Mex_SssTable()
    } else {
        null_mut()
    }) as *mut MexSssIcon;
    if tbl.is_null() {
        mnStageSel_MexMap = null_mut();
        return;
    }
    i = 0;
    while i < n {
        let mut icon: *mut stagelistinfo = (&raw mut mnStageSel_803F06D0
            as *mut stagelistinfo)
            .offset(i as isize) as *mut stagelistinfo;
        (*icon).x0 = null_mut();
        (*icon).x4 = 0;
        (*icon).x8 = (*tbl.offset(i as isize)).type_0;
        (*icon).x9 = (*tbl.offset(i as isize)).preview_id;
        (*icon).xA = (*tbl.offset(i as isize)).random_id;
        (*icon).stkind = (*tbl.offset(i as isize)).stkind;
        (*icon).xC = (*tbl.offset(i as isize)).cursor_w;
        (*icon).x10 = (*tbl.offset(i as isize)).cursor_h;
        (*icon).x14 = (*tbl.offset(i as isize)).cursor_scale_x;
        (*icon).x18 = (*tbl.offset(i as isize)).cursor_scale_y;
        i += 1;
    }
    mnStageSel_IconCount = n;
    mnStageSel_NumStages = n;
    mnStageSel_MexActive = 1;
}
unsafe extern "C" fn mnStageSel_MexScanUnlocked() {
    let mut i: i32 = 0;
    i = 0;
    while i < mnStageSel_IconCount {
        let mut icon: *mut stagelistinfo = (&raw mut mnStageSel_803F06D0
            as *mut stagelistinfo)
            .offset(i as isize) as *mut stagelistinfo;
        let mut grkind: i32 = 0;
        if !((*icon).x8 as i32 == 0
            || (*icon).x8 as i32 == 3)
        {
            grkind = if (*icon).stkind > 0 {
                Mex_GrKindForExt((*icon).stkind as i32)
            } else {
                -1
            };
            if grkind < 0
                || (*icon).stkind >= SSS_EXT_MEX_FIRST && Mex_GrFile(grkind).is_null()
            {
                (*icon).x8 = 1;
            } else {
                (*icon).x8 = (if gm_80164430((*icon).stkind as u16) != 0 {
                    2 as i32
                } else {
                    1 as i32
                }) as u8;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn mnStageSel_MexInitIcons() {
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut root: *mut HSD_JObj = null_mut();
    let mut pos: *mut HSD_JObj = null_mut();
    let mut i: i32 = 0;
    mnStageSel_MexScanUnlocked();
    gobj = GObj_Create(4, 5, 0x80);
    root = HSD_JObjLoadJoint((*mnStageSel_MexMap).pos_joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, root as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x82,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(mn_8022EAE0),
        4,
    );
    HSD_JObjAddAnimAll(
        root,
        (*mnStageSel_MexMap).pos_animjoint,
        null_mut(),
        null_mut(),
    );
    HSD_JObjReqAnimAll(root, 0.0_f32);
    HSD_JObjAnimAll(root);
    if !(*mnStageSel_MexMap).pages.is_null() {
        (*(*mnStageSel_MexMap).pages).jobj = root;
        (*(*mnStageSel_MexMap).pages).current = 0;
    }
    pos = (*root).child as *mut HSD_JObj;
    i = 0;
    while i < mnStageSel_IconCount && !pos.is_null() {
        let mut icon: *mut stagelistinfo = (&raw mut mnStageSel_803F06D0
            as *mut stagelistinfo)
            .offset(i as isize) as *mut stagelistinfo;
        let mut jobj: *mut HSD_JObj = null_mut();
        let mut g: *mut HSD_GObj = GObj_Create(4, 5, 0x80);
        jobj = HSD_JObjLoadJoint(
            if (*icon).x8 as i32 == 3 {
                (*mnStageSel_804D6C98).x10.joint as *mut HSD_Joint
            } else {
                (*mnStageSel_MexMap).icon_joint
            },
        );
        HSD_GObjObject_80390A70(g, HSD_GObj_JObjKind, jobj as *mut c_void);
        GObj_SetupGXLink(
            g,
            Some(
                HSD_GObj_JObjCallback
                   ,
            ),
            4,
            0x83,
        );
        HSD_GObj_SetupProc(
            g,
            Some(mn_8022EAE0),
            3,
        );
        HSD_JObjAddAnimAll(
            jobj,
            (*mnStageSel_MexMap).icon_animjoint,
            (*mnStageSel_MexMap).icon_matanim,
            null_mut(),
        );
        jobj = HSD_GObjGetHSDObj(g) as *mut HSD_JObj;
        lb_8000C2F8(jobj, pos);
        let mut robj: *mut HSD_RObj = HSD_RObjAlloc();
        HSD_RObjSetFlags(robj, 0x90000008);
        HSD_RObjSetConstraintObj(robj, pos as *mut c_void);
        HSD_JObjPrependRObj(jobj, robj);
        (*icon).x0 = jobj;
        if (*icon).x8 as i32 == 0 {
            HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        } else {
            do_anim(
                jobj,
                if (*icon).x8 as i32 == 1 {
                    1 as i32
                } else {
                    i + 2 as i32
                },
            );
        }
        i += 1;
        pos = (*pos).next as *mut HSD_JObj;
    }
    if i < mnStageSel_IconCount {
        while i < mnStageSel_IconCount {
            mnStageSel_803F06D0[i as usize].x0 = null_mut();
            mnStageSel_803F06D0[i as usize].x8 = 0;
            i += 1;
        }
    }
}
unsafe extern "C" fn mnStageSel_MexChangePage() -> i32 {
    let mut pages: *mut MexMapPages = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut page: i32 = 0;
    if mnStageSel_MexActive == 0 || mnStageSel_MexMap.is_null() {
        return 0;
    }
    pages = (*mnStageSel_MexMap).pages;
    if pages.is_null() || (*pages).count <= 1
        || (*pages).anims.is_null()
    {
        return 0;
    }
    jobj = (*pages).jobj;
    if jobj.is_null() {
        return 0;
    }
    if mnStageSel_804D6CA0 & 0x60 as u32 == 0 {
        return 0;
    }
    page = (*pages).current;
    if mnStageSel_804D6CA0 & 0x20 as u32 != 0 {
        page = if (page + 1 as i32) < (*pages).count {
            page + 1 as i32
        } else {
            0 as i32
        };
    } else {
        page = if page > 0 {
            page - 1 as i32
        } else {
            (*pages).count - 1 as i32
        };
    }
    (*pages).current = page;
    lbAudioAx_80024030(2 as i32);
    HSD_JObjAddAnimAll(
        jobj,
        *(*pages).anims.offset(page as isize),
        null_mut(),
        null_mut(),
    );
    HSD_JObjReqAnimAll(jobj, 0.0_f32);
    HSD_JObjAnimAll(jobj);
    mnStageSel_804D6CAE = mnStageSel_IconCount as i32;
    mnStageSel_804D6CA4 = 15;
    return 1;
}
pub unsafe extern "C" fn mnStageSel_802599EC() -> i32 {
    let mut var_r0: i32 = 0;
    let mut iter: i32 = 0;
    let mut var_r29: i32 = 1;
    let mut i: i32 = 0;
    i = 0;
    while i < mnStageSel_NumStages {
        if mnStageSel_803F06D0[i as usize].x4 >= 0
            && sss_random_ok(i) != 0
        {
            break;
        }
        i += 1;
    }
    if i == mnStageSel_NumStages {
        i = 0;
        while i < mnStageSel_NumStages {
            mnStageSel_803F06D0[i as usize].x4 = 0;
            i += 1;
        }
    }
    iter = 0;
    while var_r29 != 0 && iter < MAX_ITER {
        i = 0;
        while i < mnStageSel_NumStages {
            if mnStageSel_803F06D0[i as usize].x4 > 0 {
                mnStageSel_803F06D0[i as usize].x4 -= 1;
            }
            i += 1;
        }
        i = 0;
        while i < mnStageSel_NumStages {
            if mnStageSel_803F06D0[i as usize].x4 == 0
                && sss_random_ok(i) != 0
            {
                var_r29 = 0;
            }
            i += 1;
        }
        iter += 1;
    }
    iter = 0;
    while iter < MAX_ITER {
        let mut tmp: i32 = HSD_Randi(mnStageSel_NumStages as i32)
            as i32;
        i = tmp;
        if mnStageSel_803F06D0[i as usize].x4 == 0 {
            if sss_random_ok(i) != 0 {
                break;
            }
        }
        iter += 1;
    }
    if iter >= MAX_ITER {
        i = 0;
    }
    mnStageSel_803F06D0[i as usize].x4 = -1;
    if mnStageSel_MexActive == 0 && i < 22 {
        if i & 1 as i32 != 0 {
            var_r0 = i - 1 as i32;
        } else {
            var_r0 = i + 1 as i32;
        }
        if mnStageSel_803F06D0[var_r0 as usize].x4 >= 0 {
            mnStageSel_803F06D0[var_r0 as usize].x4 = 3;
        }
    }
    return i;
}
pub unsafe extern "C" fn mnStageSel_80259C28() {
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut gobj: *mut HSD_GObj = null_mut();
    if mnStageSel_804D6CA4 != 0 {
        return;
    }
    let mut hovered: i32 = (mnStageSel_804D6CAE >= 0
        && mnStageSel_804D6CAE < mnStageSel_IconCount) as i32;
    let mut type_0: i32 = if hovered != 0 {
        mnStageSel_803F06D0[mnStageSel_804D6CAE as usize].x8 as i32
    } else {
        0 as i32
    };
    let mut random: i32 = (hovered == 0 || type_0 == 3
        || mnStageSel_MexActive == 0 && mnStageSel_804D6CAE == mnStageSel_NumStages)
        as i32;
    if mnStageSel_804D6CA0
        & (if hovered != 0 {
            0x1100 as i32
        } else {
            0x1000 as i32
        }) as u32 == 0
    {
        return;
    }
    if random != 0 {
        mnStageSel_804D6CAE = mnStageSel_802599EC() as i32;
    } else if type_0 < 2 {
        lbAudioAx_80024030(3 as i32);
        return;
    }
    gobj = GObj_Create(HSD_GOBJ_CLASS_FIGHTER as u16, 5, 0x80);
    jobj = HSD_JObjLoadJoint((*mnStageSel_804D6C98).xB0);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x87,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(mn_8022EAE0),
        0,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*mnStageSel_804D6C98).xB4,
        (*mnStageSel_804D6C98).xB8,
        (*mnStageSel_804D6C98).xBC,
    );
    HSD_JObjReqAnimAll((*gobj).hsd_obj as *mut HSD_JObj, 0.0_f32);
    HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
    mnStageSel_804D6CAF = 1;
    mnStageSel_804D6CA4 = 0x1e;
    sfxForward();
}
pub unsafe extern "C" fn fn_80259D84(mut gobj: *mut HSD_GObj) {
    let mut temp_r31: *mut StageSelUserData = HSD_GObjGetUserData(gobj)
        as *mut StageSelUserData;
    let mut jobj: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    match (*temp_r31).x2 as i32 {
        0 => {
            if mnStageSel_804D6CAF as i32 == 0 {
                mnStageSel_80259C28();
            }
            (*temp_r31).x4 += 1;
            if (*temp_r31).x4 as u32 >= 9 {
                HSD_ForeachAnim(jobj as *mut c_void, JOBJ_TYPE, ALL_TYPE_MASK, HSD_AObjStopAnim as usize as *mut c_void, AOBJ_ARG_AOV, &[(0) as usize, (0) as usize]);
                (*temp_r31).x2 = 1;
            }
        }
        1 => {
            if mnStageSel_804D6CAF as i32 == 0 {
                mnStageSel_80259C28();
            }
            if (*temp_r31).x0 as i32 != mnStageSel_804D6CAE {
                if ((*temp_r31).x0 as i32) < mnStageSel_IconCount
                    && mnStageSel_803F06D0[(*temp_r31).x0 as usize].x8
                        as i32 >= 2
                {
                    HSD_JObjReqAnimAllByFlags(jobj, 1, 10.0_f32);
                }
                sfxMove();
                (*temp_r31).x4 = 0;
                (*temp_r31).x2 = 2;
                mnStageSel_80259ED8(mnStageSel_804D6CAE as i32);
            }
        }
        2 => {
            (*temp_r31).x4 += 1;
            if (*temp_r31).x4 as u32 > 0xa {
                HSD_GObjFree(gobj);
                (*temp_r31).x2 = (*temp_r31).x2.wrapping_add(1);
            }
        }
        _ => {}
    };
}
unsafe extern "C" fn do_anim(mut jobj: *mut HSD_JObj, mut frame: i32) {
    HSD_JObjReqAnimAll(jobj, 0.0_f32);
    HSD_JObjReqAnimAllByFlags(jobj, 0x10, frame as f32);
    HSD_JObjAnimAll(jobj);
    HSD_ForeachAnim(jobj as *mut c_void, JOBJ_TYPE, TOBJ_MASK, HSD_AObjStopAnim as usize as *mut c_void, AOBJ_ARG_AOV, &[(0) as usize, (0) as usize]);
}
pub unsafe extern "C" fn mnStageSel_80259ED8(mut id: i32) {
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut temp_r3_2: *mut StageSelUserData = core::ptr::null_mut::<
        StageSelUserData,
    >();
    gobj = GObj_Create(HSD_GOBJ_CLASS_FIGHTER as u16, 5, 0x80);
    jobj = HSD_JObjLoadJoint((*mnStageSel_804D6C98).x30.joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x84,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(mn_8022EAE0),
        1,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*mnStageSel_804D6C98).x30.animjoint,
        if !mnStageSel_MexMap.is_null() {
            (*mnStageSel_MexMap).stagename_matanim
        } else {
            (*mnStageSel_804D6C98).x30.matanim_joint
        },
        (*mnStageSel_804D6C98).x30.shapeanim_joint,
    );
    jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    temp_r3_2 = HSD_MemAlloc(core::mem::size_of::<StageSelUserData>() as isize)
        as *mut StageSelUserData;
    GObj_InitUserData(
        gobj,
        4,
        Some(HSD_Free),
        temp_r3_2 as *mut c_void,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(fn_80259D84),
        1,
    );
    (*temp_r3_2).x0 = id as u16;
    (*temp_r3_2).x4 = 0;
    (*temp_r3_2).x2 = 0;
    if id < mnStageSel_IconCount
        && mnStageSel_803F06D0[id as usize].x8 as i32
            >= 2
    {
        do_anim(
            jobj,
            (if !mnStageSel_MexMap.is_null() {
                id as f32
            } else {
                20.0_f32
                    * mnStageSel_803F06D0[id as usize].x9 as i32
                        as f32
            }) as i32,
        );
    }
}
pub unsafe extern "C" fn fn_8025A090(mut gobj: *mut HSD_GObj) {
    let mut var_r3: u32 = 0;
    let mut temp_r30: *mut C2RustUnnamed_15 = core::ptr::null_mut::<
        C2RustUnnamed_15,
    >();
    let mut jobj: *mut HSD_JObj = null_mut();
    jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    temp_r30 = HSD_GObjGetUserData(gobj) as *mut C2RustUnnamed_15;
    var_r3 = mnStageSel_804D6CAE as u32;
    if (mnStageSel_803F06D0[mnStageSel_804D6CAE as usize].x8 as i32)
        < 2
    {
        var_r3 = mnStageSel_IconCount as u32;
    }
    if (*temp_r30).x0 != var_r3 {
        (*temp_r30).x0 = var_r3;
        (*temp_r30).x4 = 0;
        if sss_has_preview(var_r3 as i32) != 0 {
            HSD_JObjReqAnimAll(
                jobj,
                50.0_f32
                    * mnStageSel_803F06D0[var_r3 as usize].x9 as i32
                        as f32,
            );
            HSD_JObjAnimAll(jobj);
            HSD_ForeachAnim(jobj as *mut c_void, JOBJ_TYPE, ALL_TYPE_MASK, HSD_AObjStopAnim as usize as *mut c_void, AOBJ_ARG_AOV, &[(0) as usize, (0) as usize]);
            HSD_JObjSetTranslateX(jobj, 0.0_f32);
        } else {
            HSD_JObjSetTranslateX(jobj, 100.0_f32);
        }
    }
    if (*temp_r30).x4 < 0x5a {
        (*temp_r30).x4 = (*temp_r30).x4.wrapping_add(1);
        if (*temp_r30).x4 == 0x14 {
            HSD_JObjReqAnimAll(
                jobj,
                50.0_f32
                    * mnStageSel_803F06D0[(*temp_r30).x0 as usize].x9
                        as i32 as f32,
            );
        }
        if (*temp_r30).x4 == 0x45 {
            HSD_ForeachAnim(jobj as *mut c_void, JOBJ_TYPE, ALL_TYPE_MASK, HSD_AObjStopAnim as usize as *mut c_void, AOBJ_ARG_AOV, &[(0) as usize, (0) as usize]);
        }
    } else if mnStageSel_804D6CAF != 0 {
        mnStageSel_804D6CAF = 2;
    }
}
pub unsafe extern "C" fn fn_8025A310(mut gobj: *mut HSD_GObj) {
    let mut sp1C: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut sp10: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut temp_f1: f32 = 0.;
    let mut temp_f2: f32 = 0.;
    let mut i: i32 = 0;
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut unused: u32 = 0;
    jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if mnStageSel_804D6CAF as i32 != 0 {
        HSD_JObjSetFlags(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    HSD_JObjGetTranslation(jobj, &raw mut sp1C);
    sp1C.x = 0.03_f32 * mnStageSel_804D6CAC as i32 as f32 + sp1C.x;
    if -27.0_f32 > sp1C.x {
        sp1C.x = -27.0_f32 as f32;
    }
    if 27.0_f32 < sp1C.x {
        sp1C.x = 27.0_f32 as f32;
    }
    sp1C.y = 0.03_f32 * mnStageSel_804D6CAD as i32 as f32 + sp1C.y;
    if -19.0_f32 > sp1C.y {
        sp1C.y = -19.0_f32 as f32;
    }
    if 19.0_f32 < sp1C.y {
        sp1C.y = 19.0_f32 as f32;
    }
    HSD_JObjSetTranslate(jobj, &raw mut sp1C);
    lb_8000B1CC(jobj, null_mut(), &raw mut sp1C);
    i = 0;
    while i < mnStageSel_IconCount {
        if mnStageSel_803F06D0[i as usize].x8 as i32
            != 0
        {
            lb_8000B1CC(
                mnStageSel_803F06D0[i as usize].x0,
                null_mut(),
                &raw mut sp10,
            );
            temp_f2 = sp10.x;
            temp_f1 = mnStageSel_803F06D0[i as usize].xC;
            if temp_f2 - temp_f1 < sp1C.x && temp_f2 + temp_f1 > sp1C.x {
                if sp10.y - mnStageSel_803F06D0[i as usize].x10 < sp1C.y
                    && sp10.y + mnStageSel_803F06D0[i as usize].x10 > sp1C.y
                {
                    mnStageSel_804D6CAE = i as i32;
                    return;
                }
            }
        }
        i += 1;
    }
}
pub unsafe extern "C" fn fn_8025A560(mut gobj: *mut HSD_GObj) {
    let mut temp_r30: *mut StageSelUserData_0 = core::ptr::null_mut::<
        StageSelUserData_0,
    >();
    let mut sp10: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut jobj: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    temp_r30 = HSD_GObjGetUserData(gobj) as *mut StageSelUserData_0;
    if mnStageSel_MexActive == 0 && mnStageSel_804D6CAE < 0x16
        && mnStageSel_804D6CAE as i32 & 1 as i32 != 0
        && mnStageSel_803F06D0[mnStageSel_804D6CAE as usize].x8 as i32
            == 0
    {
        HSD_JObjSetTranslateX(jobj, 100.0_f32);
    } else if mnStageSel_804D6CAE < mnStageSel_IconCount {
        lb_8000B1CC(
            mnStageSel_803F06D0[mnStageSel_804D6CAE as usize].x0,
            null_mut(),
            &raw mut sp10,
        );
        HSD_JObjSetTranslateX(jobj, sp10.x);
        HSD_JObjSetTranslateY(jobj, sp10.y);
        HSD_JObjSetScaleX(jobj, mnStageSel_803F06D0[mnStageSel_804D6CAE as usize].x14);
        HSD_JObjSetScaleY(jobj, mnStageSel_803F06D0[mnStageSel_804D6CAE as usize].x18);
    } else {
        HSD_JObjSetTranslateX(jobj, 100.0_f32);
    }
    if mnStageSel_804D6CAF as i32 != 0 {
        HSD_JObjReqAnimAll(jobj, 0.0_f32);
        HSD_JObjAnimAll(jobj);
        return;
    }
    (*temp_r30).x0 += 1;
    if (*temp_r30).x0 >= 10 {
        (*temp_r30).x0 = 0;
        HSD_JObjReqAnimAll(jobj, 0.0_f32);
        HSD_JObjAnimAll(jobj);
    }
}
pub unsafe extern "C" fn fn_8025A91C(mut gobj: *mut HSD_GObj) {
    let mut jobj: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    mnStageSel_804D6CA8 += 1;
    if mnStageSel_804D6CA8 >= 0xfa {
        mnStageSel_804D6CA8 = 0 as i32;
        HSD_JObjReqAnimAll(jobj, 0.0_f32);
        HSD_JObjAnimAll(jobj);
    }
}
pub unsafe extern "C" fn fn_8025A974(
    mut gobj: *mut HSD_GObj,
    mut unused: i32,
) {
    HSD_FogSet((*gobj).hsd_obj as *mut HSD_Fog);
}
static mut mnStageSel_803B8550: Vec3 = Vec3 {
    x: 0 as i32 as f32,
    y: -13.0_f32,
    z: 0 as i32 as f32,
};
unsafe extern "C" fn make_stage_icon(mut out: *mut *mut HSD_JObj) {
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    gobj = GObj_Create(4, 5, 0x80);
    jobj = HSD_JObjLoadJoint((*mnStageSel_804D6C98).x40.joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x83,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(mn_8022EAE0),
        3,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*mnStageSel_804D6C98).x40.animjoint,
        (*mnStageSel_804D6C98).x40.matanim_joint,
        (*mnStageSel_804D6C98).x40.shapeanim_joint,
    );
    *out = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
}
unsafe extern "C" fn attach_menu_model(mut gobj: *mut HSD_GObj) {
    let mut jobj: *mut HSD_JObj = null_mut();
    jobj = HSD_JObjLoadJoint((*mnStageSel_804D6C98).xA0.joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x80,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(mn_8022EAE0),
        0,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*mnStageSel_804D6C98).xA0.animjoint,
        (*mnStageSel_804D6C98).xA0.matanim_joint,
        (*mnStageSel_804D6C98).xA0.shapeanim_joint,
    );
}
unsafe extern "C" fn make_bg_model(mut out: *mut *mut HSD_JObj) {
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    gobj = GObj_Create(4, 5, 0x80);
    jobj = HSD_JObjLoadJoint((*mnStageSel_804D6C98).x50.joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x82,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(mn_8022EAE0),
        0,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*mnStageSel_804D6C98).x50.animjoint,
        (*mnStageSel_804D6C98).x50.matanim_joint,
        (*mnStageSel_804D6C98).x50.shapeanim_joint,
    );
    *out = (*gobj).hsd_obj as *mut HSD_JObj;
}
unsafe extern "C" fn make_icon_root(mut icons: *mut *mut HSD_JObj) {
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    gobj = GObj_Create(4, 5, 0x80);
    jobj = HSD_JObjLoadJoint((*mnStageSel_804D6C98).x90.joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x82,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(mn_8022EAE0),
        4,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*mnStageSel_804D6C98).x90.animjoint,
        (*mnStageSel_804D6C98).x90.matanim_joint,
        (*mnStageSel_804D6C98).x90.shapeanim_joint,
    );
    let ref mut fresh0 = *icons.offset(0 as i32 as isize);
    *fresh0 = (*(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj)).child as *mut HSD_JObj;
    HSD_JObjReqAnimAll(*icons.offset(0 as i32 as isize), 0.0_f32);
    HSD_JObjAnimAll(*icons.offset(0 as i32 as isize));
}
unsafe extern "C" fn get_jobj(mut gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    let mut jobj: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    return jobj;
}
pub unsafe extern "C" fn mnStageSel_Scene_OnEnter(mut arg0: *mut c_void) {
    let mut spDC: [*mut HSD_JObj; 19] = [null_mut(); 19];
    let mut spCC: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut i: i32 = 0;
    let mut temp_r3: *mut C2RustUnnamed_16 = null_mut();
    sss_data = arg0 as *mut SSSData;
    if ((*sss_data).force_stage_id as i32) < 0 {
        if lbLang_IsSavedLanguageUS() != 0 {
            mnStageSel_804D6C94 = lbArchive_LoadArchive(
                c"MnSlMap.usd".as_ptr(),
            );
        } else {
            mnStageSel_804D6C94 = lbArchive_LoadArchive(
                c"MnSlMap.dat".as_ptr(),
            );
        }
        temp_r3 = HSD_ArchiveGetPublicAddress(
            mnStageSel_804D6C94,
            c"MnSelectStageDataTable".as_ptr(),
        ) as *mut C2RustUnnamed_16;
        MenMain_cam = (*temp_r3).unk0;
        mnStageSel_804D6C98 = &raw mut (*temp_r3).x10 as *mut mnStageSel_804D6C98_t;
        mnStageSel_MexSetup(mnStageSel_804D6C94);
        mnStageSel_804D6CAF = 0;
        mnStageSel_804D6CA0 = 0;
        mnStageSel_804D6CAC = 0;
        mnStageSel_804D6CAD = 0;
        mnStageSel_804D6CAE = mnStageSel_IconCount as i32;
        mnStageSel_804D50A0 = ((*sss_data).unk_stage as i32
            - 1 as i32) as i8;
        mnStageSel_804D6CA4 = 0x14;
        mnStageSel_804D6C9C = GObj_Create(2, 3, 0x80);
        let mut gobj: *mut HSD_GObj = mnStageSel_804D6C9C;
        let mut cobj: *mut HSD_CObj = HSD_CObjLoadDesc(MenMain_cam);
        HSD_GObjObject_80390A70(
            gobj,
            HSD_GObj_CameraKind,
            cobj as *mut c_void,
        );
        GObj_SetupGXLinkMax(
            gobj,
            Some(
                HSD_GObj_803910D8
                   ,
            ),
            0,
        );
        (*gobj).gxlink_prios = 0x11;
        HSD_GObj_SetupProc(
            gobj,
            Some(mn_8022BA1C),
            5,
        );
        let mut gobj_0: *mut HSD_GObj = null_mut();
        let mut lobj1: *mut HSD_LObj = null_mut();
        let mut lobj2: *mut HSD_LObj = null_mut();
        gobj_0 = GObj_Create(3, 4, 0x80);
        lobj1 = HSD_LObjLoadDesc((*temp_r3).unk4);
        lobj2 = HSD_LObjLoadDesc((*temp_r3).unk8);
        HSD_LObjSetNext(lobj1, lobj2);
        HSD_GObjObject_80390A70(
            gobj_0,
            HSD_GObj_LightKind as u8,
            lobj1 as *mut c_void,
        );
        GObj_SetupGXLink(
            gobj_0,
            Some(
                HSD_GObj_LObjCallback
                   ,
            ),
            0,
            0x80,
        );
        let mut gobj_1: *mut HSD_GObj = GObj_Create(
            0xe,
            0xf,
            0,
        );
        let mut fog: *mut HSD_Fog = HSD_FogLoadDesc((*temp_r3).unkC);
        HSD_GObjObject_80390A70(
            gobj_1,
            HSD_GObj_FogKind as u8,
            fog as *mut c_void,
        );
        GObj_SetupGXLink(
            gobj_1,
            Some(
                fn_8025A974
                   ,
            ),
            0,
            0x80,
        );
        let mut jobj2: *mut HSD_JObj = null_mut();
        let mut gobj_2: *mut HSD_GObj = null_mut();
        gobj_2 = GObj_Create(4, 5, 0x80);
        attach_menu_model(gobj_2);
        let mut g: *mut HSD_GObj = gobj_2;
        jobj2 = HSD_GObjGetHSDObj(g) as *mut HSD_JObj;
        HSD_GObj_SetupProc(
            g,
            Some(fn_8025A91C),
            0,
        );
        HSD_JObjReqAnimAll(jobj2, 0.0_f32);
        HSD_JObjAnimAll(jobj2);
        let mut temp_r22_4: *mut HSD_JObj = null_mut();
        make_bg_model(&raw mut temp_r22_4);
        HSD_JObjReqAnimAll(temp_r22_4, 0.0_f32);
        HSD_JObjAnimAll(temp_r22_4);
        if mnStageSel_MexActive != 0 {
            mnStageSel_MexInitIcons();
        } else {
            make_icon_root(&raw mut spDC as *mut *mut HSD_JObj);
            i = 0;
            while i < 0x12 {
                spDC[(i + 1 as i32) as usize] = (*spDC[i as usize]).next
                    as *mut HSD_JObj;
                HSD_JObjReqAnimAll(spDC[(i + 1 as i32) as usize], 0.0_f32);
                HSD_JObjAnimAll(spDC[(i + 1 as i32) as usize]);
                i += 1;
            }
            i = 0;
            while i < mnStageSel_NumStages {
                mnStageSel_803F06D0[i as usize].x8 = (if gm_80164430(
                    mnStageSel_803F06D0[i as usize].stkind as u16,
                ) != 0
                {
                    2 as i32
                } else {
                    1 as i32
                }) as u8;
                i += 1;
            }
            i = 0;
            while i <= 0xa {
                let mut temp_r22_6: *mut HSD_JObj = null_mut();
                let mut jobj: *mut HSD_JObj = null_mut();
                make_stage_icon(&raw mut temp_r22_6);
                jobj = temp_r22_6;
                lb_8000C1C0(jobj, spDC[i as usize]);
                mnStageSel_803F06D0[(i * 2 as i32) as usize].x0 = (*(*temp_r22_6)
                    .child)
                    .next as *mut HSD_JObj;
                match mnStageSel_803F06D0[(i * 2 as i32) as usize].x8
                    as i32
                {
                    0 => {
                        HSD_JObjSetFlags(
                            mnStageSel_803F06D0[(i * 2 as i32) as usize]
                                .x0,
                            JOBJ_HIDDEN as u32,
                        );
                    }
                    1 => {
                        HSD_JObjReqAnimAllByFlags(
                            mnStageSel_803F06D0[(i * 2 as i32) as usize]
                                .x0,
                            0x10,
                            1.0_f32,
                        );
                    }
                    _ => {
                        HSD_JObjReqAnimAllByFlags(
                            mnStageSel_803F06D0[(i * 2 as i32) as usize]
                                .x0,
                            0x10,
                            (mnStageSel_803F06D0[(i * 2 as i32) as usize]
                                .x9 as i32 / 2 as i32
                                + 2 as i32) as f32,
                        );
                    }
                }
                mnStageSel_803F06D0[(i * 2 as i32
                        + 1 as i32) as usize]
                    .x0 = (*temp_r22_6).child as *mut HSD_JObj;
                match mnStageSel_803F06D0[(i * 2 as i32
                        + 1 as i32) as usize]
                    .x8 as i32
                {
                    0 => {
                        HSD_JObjSetFlags(
                            mnStageSel_803F06D0[(i * 2 as i32
                                    + 1 as i32) as usize]
                                .x0,
                            JOBJ_HIDDEN as u32,
                        );
                    }
                    1 => {
                        HSD_JObjReqAnimAllByFlags(
                            mnStageSel_803F06D0[(i * 2 as i32
                                    + 1 as i32) as usize]
                                .x0,
                            0x10,
                            1.0_f32,
                        );
                    }
                    _ => {
                        HSD_JObjReqAnimAllByFlags(
                            mnStageSel_803F06D0[(i * 2 as i32
                                    + 1 as i32) as usize]
                                .x0,
                            0x10,
                            (mnStageSel_803F06D0[(i * 2 as i32
                                    + 1 as i32) as usize]
                                .x9 as i32 / 2 as i32
                                + 2 as i32) as f32,
                        );
                    }
                }
                HSD_JObjAnimAll(jobj);
                HSD_ForeachAnim(jobj as *mut c_void, JOBJ_TYPE, TOBJ_MASK, HSD_AObjStopAnim as usize as *mut c_void, AOBJ_ARG_AOV, &[(0) as usize, (0) as usize]);
                i += 1;
            }
            i = 0xb;
            while i <= 0xf {
                let mut jobj_0: *mut HSD_JObj = null_mut();
                let mut gobj_3: *mut HSD_GObj = GObj_Create(
                    4,
                    5,
                    0x80,
                );
                let mut temp_r22_7: i32 = 0;
                let mut temp_r23_3: *mut HSD_JObj = null_mut();
                jobj_0 = HSD_JObjLoadJoint(
                    (*mnStageSel_804D6C98).x20.joint as *mut HSD_Joint,
                );
                HSD_GObjObject_80390A70(
                    gobj_3,
                    HSD_GObj_JObjKind,
                    jobj_0 as *mut c_void,
                );
                GObj_SetupGXLink(
                    gobj_3,
                    Some(
                        HSD_GObj_JObjCallback
                           ,
                    ),
                    4,
                    0x83,
                );
                HSD_GObj_SetupProc(
                    gobj_3,
                    Some(mn_8022EAE0),
                    3,
                );
                HSD_JObjAddAnimAll(
                    jobj_0,
                    (*mnStageSel_804D6C98).x20.animjoint,
                    (*mnStageSel_804D6C98).x20.matanim_joint,
                    (*mnStageSel_804D6C98).x20.shapeanim_joint,
                );
                temp_r23_3 = (*gobj_3).hsd_obj as *mut HSD_JObj;
                lb_8000C1C0(temp_r23_3, spDC[i as usize]);
                mnStageSel_803F06D0[(i + 13 as i32) as usize].x0 = temp_r23_3;
                let mut current_block_89: u64;
                match mnStageSel_803F06D0[(i + 13 as i32) as usize].x8
                    as i32
                {
                    1 => {
                        mnStageSel_803F06D0[(i + 13 as i32) as usize]
                            .x8 = 0;
                        current_block_89 = 7080652995821737316;
                    }
                    0 => {
                        current_block_89 = 7080652995821737316;
                    }
                    _ => {
                        temp_r22_7 = (mnStageSel_803F06D0[(i + 13 as i32)
                                as usize]
                            .x9 as i32 - 0x16 as i32)
                            as i32;
                        do_anim(temp_r23_3, temp_r22_7 as i32);
                        current_block_89 = 17019156190352891614;
                    }
                }
                match current_block_89 {
                    7080652995821737316 => {
                        HSD_JObjSetFlagsAll(temp_r23_3, JOBJ_HIDDEN as u32);
                    }
                    _ => {}
                }
                i += 1;
            }
            let mut jobj_1: *mut HSD_JObj = null_mut();
            let mut gobj_4: *mut HSD_GObj = GObj_Create(
                4,
                5,
                0x80,
            );
            let mut temp_r22_8: *mut HSD_JObj = null_mut();
            jobj_1 = HSD_JObjLoadJoint(
                (*mnStageSel_804D6C98).x10.joint as *mut HSD_Joint,
            );
            HSD_GObjObject_80390A70(
                gobj_4,
                HSD_GObj_JObjKind,
                jobj_1 as *mut c_void,
            );
            GObj_SetupGXLink(
                gobj_4,
                Some(
                    HSD_GObj_JObjCallback
                       ,
                ),
                4,
                0x83,
            );
            HSD_GObj_SetupProc(
                gobj_4,
                Some(mn_8022EAE0),
                3,
            );
            HSD_JObjAddAnimAll(
                jobj_1,
                (*mnStageSel_804D6C98).x10.animjoint,
                (*mnStageSel_804D6C98).x10.matanim_joint,
                (*mnStageSel_804D6C98).x10.shapeanim_joint,
            );
            temp_r22_8 = (*gobj_4).hsd_obj as *mut HSD_JObj;
            lb_8000C1C0(temp_r22_8, spDC[0x10 as i32 as usize]);
            do_anim(temp_r22_8, 2);
            mnStageSel_803F06D0[mnStageSel_NumStages as usize].x0 = temp_r22_8;
            i = 0x11;
            while i <= 0x12 {
                let mut jobj_2: *mut HSD_JObj = null_mut();
                let mut animjoint: *mut HSD_AnimJoint = core::ptr::null_mut::<
                    HSD_AnimJoint,
                >();
                let mut gobj_5: *mut HSD_GObj = GObj_Create(
                    4,
                    5,
                    0x80,
                );
                let mut temp_r22_9: i32 = 0;
                let mut temp_r23_6: *mut HSD_JObj = null_mut();
                jobj_2 = HSD_JObjLoadJoint(
                    (*mnStageSel_804D6C98).x0.joint as *mut HSD_Joint,
                );
                HSD_GObjObject_80390A70(
                    gobj_5,
                    HSD_GObj_JObjKind,
                    jobj_2 as *mut c_void,
                );
                GObj_SetupGXLink(
                    gobj_5,
                    Some(
                        HSD_GObj_JObjCallback
                           ,
                    ),
                    4,
                    0x83,
                );
                HSD_GObj_SetupProc(
                    gobj_5,
                    Some(mn_8022EAE0),
                    3,
                );
                animjoint = (*mnStageSel_804D6C98).x0.animjoint;
                HSD_JObjAddAnimAll(
                    jobj_2,
                    animjoint,
                    (*mnStageSel_804D6C98).x0.matanim_joint,
                    (*mnStageSel_804D6C98).x0.shapeanim_joint,
                );
                temp_r23_6 = (*gobj_5).hsd_obj as *mut HSD_JObj;
                lb_8000C1C0(temp_r23_6, spDC[i as usize]);
                mnStageSel_803F06D0[(i + 5 as i32) as usize].x0 = temp_r23_6;
                let mut current_block_116: u64;
                match mnStageSel_803F06D0[(i + 5 as i32) as usize].x8
                    as i32
                {
                    1 => {
                        mnStageSel_803F06D0[(i + 5 as i32) as usize].x8 = 0
                            as u8;
                        current_block_116 = 16603926312747396594;
                    }
                    0 => {
                        current_block_116 = 16603926312747396594;
                    }
                    _ => {
                        temp_r22_9 = (mnStageSel_803F06D0[(i + 5 as i32)
                                as usize]
                            .x9 as i32 - 0x14 as i32)
                            as i32;
                        do_anim(temp_r23_6, temp_r22_9 as i32);
                        current_block_116 = 6074735043880891984;
                    }
                }
                match current_block_116 {
                    16603926312747396594 => {
                        HSD_JObjSetFlagsAll(temp_r23_6, JOBJ_HIDDEN as u32);
                    }
                    _ => {}
                }
                i += 1;
            }
        }
        let mut jobj_3: *mut HSD_JObj = null_mut();
        let mut gobj_6: *mut HSD_GObj = null_mut();
        gobj_6 = GObj_Create(4, 5, 0x80);
        jobj_3 = HSD_JObjLoadJoint((*mnStageSel_804D6C98).x80.joint as *mut HSD_Joint);
        HSD_GObjObject_80390A70(
            gobj_6,
            HSD_GObj_JObjKind,
            jobj_3 as *mut c_void,
        );
        GObj_SetupGXLink(
            gobj_6,
            Some(
                HSD_GObj_JObjCallback
                   ,
            ),
            4,
            0x86,
        );
        HSD_GObj_SetupProc(
            gobj_6,
            Some(mn_8022EAE0),
            2,
        );
        HSD_JObjAddAnimAll(
            jobj_3,
            (*mnStageSel_804D6C98).x80.animjoint,
            (*mnStageSel_804D6C98).x80.matanim_joint,
            (*mnStageSel_804D6C98).x80.shapeanim_joint,
        );
        let mut jobj2_0: *mut HSD_JObj = null_mut();
        let mut g_0: *mut HSD_GObj = null_mut();
        jobj_3 = get_jobj(gobj_6);
        spCC = mnStageSel_803B8550;
        g_0 = gobj_6;
        jobj2_0 = jobj_3;
        HSD_GObj_SetupProc(
            g_0,
            Some(fn_8025A310),
            2,
        );
        do_anim(
            jobj2_0,
            mnStageSel_804D50A0 as i32 + 1 as i32,
        );
        HSD_JObjSetTranslate(jobj2_0, &raw mut spCC);
        let mut jobj_4: *mut HSD_JObj = null_mut();
        let mut gobj_7: *mut HSD_GObj = null_mut();
        gobj_7 = GObj_Create(HSD_GOBJ_CLASS_FIGHTER as u16, 5, 0x80);
        jobj_4 = HSD_JObjLoadJoint((*mnStageSel_804D6C98).x30.joint as *mut HSD_Joint);
        HSD_GObjObject_80390A70(
            gobj_7,
            HSD_GObj_JObjKind,
            jobj_4 as *mut c_void,
        );
        GObj_SetupGXLink(
            gobj_7,
            Some(
                HSD_GObj_JObjCallback
                   ,
            ),
            4,
            0x84,
        );
        HSD_GObj_SetupProc(
            gobj_7,
            Some(mn_8022EAE0),
            1,
        );
        HSD_JObjAddAnimAll(
            jobj_4,
            (*mnStageSel_804D6C98).x30.animjoint,
            (*mnStageSel_804D6C98).x30.matanim_joint,
            (*mnStageSel_804D6C98).x30.shapeanim_joint,
        );
        let mut userdata: *mut StageSelUserData = core::ptr::null_mut::<
            StageSelUserData,
        >();
        let mut jobj_5: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj_7) as *mut HSD_JObj;
        userdata = HSD_MemAlloc(core::mem::size_of::<StageSelUserData>() as isize)
            as *mut StageSelUserData;
        GObj_InitUserData(
            gobj_7,
            4,
            Some(HSD_Free),
            userdata as *mut c_void,
        );
        HSD_GObj_SetupProc(
            gobj_7,
            Some(fn_80259D84),
            1,
        );
        (*userdata).x0 = mnStageSel_IconCount as u16;
        (*userdata).x4 = 0;
        (*userdata).x2 = 0;
        let mut jobj_6: *mut HSD_JObj = null_mut();
        let mut gobj_8: *mut HSD_GObj = GObj_Create(4, 5, 0x80);
        jobj_6 = HSD_JObjLoadJoint((*mnStageSel_804D6C98).x60.joint as *mut HSD_Joint);
        HSD_GObjObject_80390A70(
            gobj_8,
            HSD_GObj_JObjKind,
            jobj_6 as *mut c_void,
        );
        GObj_SetupGXLink(
            gobj_8,
            Some(
                HSD_GObj_JObjCallback
                   ,
            ),
            4,
            0x81,
        );
        HSD_GObj_SetupProc(
            gobj_8,
            Some(mn_8022EAE0),
            1,
        );
        HSD_JObjAddAnimAll(
            jobj_6,
            (*mnStageSel_804D6C98).x60.animjoint,
            (*mnStageSel_804D6C98).x60.matanim_joint,
            (*mnStageSel_804D6C98).x60.shapeanim_joint,
        );
        let mut g_1: *mut HSD_GObj = null_mut();
        let mut jobj_7: *mut HSD_JObj = null_mut();
        let mut temp_r3_14: *mut foo = null_mut();
        g_1 = gobj_8;
        jobj_7 = HSD_GObjGetHSDObj(g_1) as *mut HSD_JObj;
        temp_r3_14 = HSD_MemAlloc(core::mem::size_of::<foo>() as isize) as *mut foo;
        GObj_InitUserData(
            g_1,
            4,
            Some(HSD_Free),
            temp_r3_14 as *mut c_void,
        );
        HSD_GObj_SetupProc(
            g_1,
            Some(fn_8025A090),
            1,
        );
        HSD_JObjReqAnimAll(jobj_7, 0.0_f32);
        HSD_JObjAnimAll(jobj_7);
        HSD_ForeachAnim(jobj_7 as *mut c_void, JOBJ_TYPE, ALL_TYPE_MASK, HSD_AObjStopAnim as usize as *mut c_void, AOBJ_ARG_AOV, &[(0) as usize, (0) as usize]);
        HSD_JObjSetTranslateX(jobj_7, 100.0_f32);
        (*temp_r3_14).x0 = mnStageSel_IconCount;
        (*temp_r3_14).x4 = 0;
        let mut jobj_8: *mut HSD_JObj = null_mut();
        let mut gobj_9: *mut HSD_GObj = GObj_Create(4, 5, 0x80);
        let mut temp_r3_15: *mut HSD_JObj = HSD_JObjLoadJoint(
            (*mnStageSel_804D6C98).x70.joint as *mut HSD_Joint,
        );
        let mut temp_r3_16: *mut i32 = null_mut();
        HSD_GObjObject_80390A70(
            gobj_9,
            HSD_GObj_JObjKind,
            temp_r3_15 as *mut c_void,
        );
        GObj_SetupGXLink(
            gobj_9,
            Some(
                HSD_GObj_JObjCallback
                   ,
            ),
            4,
            0x85,
        );
        HSD_GObj_SetupProc(
            gobj_9,
            Some(mn_8022EAE0),
            1,
        );
        HSD_JObjAddAnimAll(
            temp_r3_15,
            (*mnStageSel_804D6C98).x70.animjoint,
            (*mnStageSel_804D6C98).x70.matanim_joint,
            (*mnStageSel_804D6C98).x70.shapeanim_joint,
        );
        jobj_8 = (*gobj_9).hsd_obj as *mut HSD_JObj;
        temp_r3_16 = HSD_MemAlloc(core::mem::size_of::<i32>() as isize) as *mut i32;
        GObj_InitUserData(
            gobj_9,
            4,
            Some(HSD_Free),
            temp_r3_16 as *mut c_void,
        );
        *temp_r3_16 = 0 as i32;
        HSD_GObj_SetupProc(
            gobj_9,
            Some(fn_8025A560),
            1,
        );
        HSD_JObjSetTranslateX(jobj_8, 100.0_f32);
        lbAudioAx_80023F28(gmMainLib_8015ECB0());
    }
}
unsafe extern "C" fn get_pad(mut i: u8) -> *mut HSD_PadStatus {
    return (&raw mut HSD_PadCopyStatus as *mut HSD_PadStatus).offset(i as isize)
        as *mut HSD_PadStatus;
}
pub unsafe extern "C" fn mnStageSel_Scene_OnFrame() {
    if (*sss_data).force_stage_id as i32 >= 0 {
        mnStageSel_804D6CAF = 2;
        (*sss_data).vs.start.rules.stkind = (*sss_data).force_stage_id as u16;
        gm_801A4B60();
        return;
    }
    if (*sss_data).no_lras as i32 == 0
        && mn_8022F218() != 0
    {
        sfxBack();
        lb_800145F4();
        HSD_GObjFree(mnStageSel_804D6C9C);
        mn_8022F268();
        gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        gm_801A4B60();
        return;
    }
    if (mnStageSel_804D50A0 as i32) < 0 {
        mnStageSel_804D6CA0 = 0;
        mnStageSel_804D6CA0
            |= HSD_PadCopyStatus[0 as i32 as usize].trigger;
        mnStageSel_804D6CA0
            |= HSD_PadCopyStatus[1 as i32 as usize].trigger;
        mnStageSel_804D6CA0
            |= HSD_PadCopyStatus[2 as i32 as usize].trigger;
        mnStageSel_804D6CA0
            |= HSD_PadCopyStatus[3 as i32 as usize].trigger;
        let mut i: i32 = 0;
        i = 0;
        while i < 4 {
            mnStageSel_804D6CAC = (*get_pad(i as u8)).stickX;
            mnStageSel_804D6CAD = (*get_pad(i as u8)).stickY;
            if ((*get_pad(i as u8)).stickX as i32)
                < -(0x1e as i32)
                || (*get_pad(i as u8)).stickX as i32
                    > 0x1e
                || ((*get_pad(i as u8)).stickY as i32)
                    < -(0x1e as i32)
                || (*get_pad(i as u8)).stickY as i32
                    > 0x1e
            {
                break;
            }
            i += 1;
        }
    } else {
        mnStageSel_804D6CA0 = (*get_pad(mnStageSel_804D50A0 as u8)).trigger;
        mnStageSel_804D6CAC = (*get_pad(mnStageSel_804D50A0 as u8)).stickX;
        mnStageSel_804D6CAD = (*get_pad(mnStageSel_804D50A0 as u8)).stickY;
    }
    if (mnStageSel_804D6CAC as i32) < -(0x1e as i32) {
        mnStageSel_804D6CAC = (mnStageSel_804D6CAC as i32
            + 0x1e as i32) as i8;
    } else if mnStageSel_804D6CAC as i32 > 0x1e {
        mnStageSel_804D6CAC = (mnStageSel_804D6CAC as i32
            - 0x1e as i32) as i8;
    } else {
        mnStageSel_804D6CAC = 0;
    }
    if (mnStageSel_804D6CAD as i32) < -(0x1e as i32) {
        mnStageSel_804D6CAD = (mnStageSel_804D6CAD as i32
            + 0x1e as i32) as i8;
    } else if mnStageSel_804D6CAD as i32 > 0x1e {
        mnStageSel_804D6CAD = (mnStageSel_804D6CAD as i32
            - 0x1e as i32) as i8;
    } else {
        mnStageSel_804D6CAD = 0;
    }
    if mnStageSel_804D6CA4 != 0 {
        mnStageSel_804D6CA4 = mnStageSel_804D6CA4.wrapping_sub(1 as u32);
        return;
    }
    if (*sss_data).x1 as i32 == 0
        && mnStageSel_804D6CAF as i32 == 0
        && mnStageSel_MexChangePage() != 0
    {
        return;
    }
    if (*sss_data).x1 as i32 == 0
        && mnStageSel_804D6CA0 & 0x200 as u32 != 0
        && mnStageSel_804D6CAF as i32 == 0
    {
        sfxBack();
        gm_801A4B60();
    }
    if mnStageSel_804D6CAF as i32 == 2 {
        (*sss_data).vs.start.rules.stkind = mnStageSel_803F06D0[mnStageSel_804D6CAE
                as usize]
            .stkind as u16;
        extern "C" {
            #[link_name = "Frontend_CaptureIcon"]
            fn Frontend_CaptureIcon_0(which: i32, root: *mut HSD_JObj);
        }
        Frontend_CaptureIcon_0(
            1,
            mnStageSel_803F06D0[mnStageSel_804D6CAE as usize].x0,
        );
        gm_801A4B60();
    }
}
pub unsafe extern "C" fn mnStageSel_Scene_OnExit(
    mut exit_data: *mut c_void,
) {
    if !mnStageSel_804D6C94.is_null() {
        lbArchive_80016EFC(mnStageSel_804D6C94);
        mnStageSel_804D6C94 = null_mut();
    }
    mnStageSel_MexMap = null_mut();
    mnStageSel_MexActive = 0;
    mnStageSel_NumStages = NUM_STAGES_RETAIL;
    mnStageSel_IconCount = SSS_ICON_COUNT_RETAIL;
    let mut sss: *mut SSSData = sss_data;
    (*sss).start_game = (if mnStageSel_804D6CAF as i32
        == 2
    {
        1
    } else {
        0
    }) as u8;
    if (*sss).start_game != 0 {
        let mut cache: *mut PreloadedGameModeState = lbDvd_GetPreloadCacheScene();
        (*cache).game_cache.stkind = (*sss).vs.start.rules.stkind as StKind;
        lbDvd_80018254();
    }
}
pub unsafe extern "C" fn mnSelStageRandom() -> i32 {
    return mnStageSel_803F06D0[mnStageSel_802599EC() as usize].stkind
        as i32;
}
pub unsafe extern "C" fn mnStageSel_8025BC08(
    mut idx: i32,
) -> i32 {
    return mnStageSel_803F06D0[idx as usize].stkind as i32;
}
static mut mnStageSel_PcMexIcon: *mut HSD_JObj = null_mut();
static mut mnStageSel_PcMex: *mut MexMapData = null_mut();
static mut mnStageSel_PcCount: i32 = 0;
unsafe extern "C" fn mnStageSel_PcBestTex(
    mut j: *mut HSD_JObj,
    mut best: *mut *mut HSD_TObj,
    mut area: *mut i32,
    mut depth: i32,
) {
    while !j.is_null() && depth < 12 {
        let mut d: *mut HSD_DObj = HSD_JObjGetDObj(j) as *mut HSD_DObj;
        while !d.is_null() {
            let mut t: *mut HSD_TObj = if !(*d).mobj.is_null() {
                (*(*d).mobj).tobj
            } else {
                null_mut()
            };
            while !t.is_null() {
                if !(*t).imagedesc.is_null() && !(*(*t).imagedesc).image_ptr.is_null() {
                    let mut fmt: i32 = (*(*t).imagedesc).format
                        as i32;
                    let mut a: i32 = (*(*t).imagedesc).width
                        as i32
                        * (*(*t).imagedesc).height as i32
                        * (if fmt >= 4 {
                            4 as i32
                        } else {
                            1 as i32
                        });
                    if a > *area {
                        *area = a;
                        *best = t;
                    }
                }
                t = (*t).next;
            }
            d = (*d).next;
        }
        mnStageSel_PcBestTex(
            HSD_JObjGetChild(j),
            best,
            area,
            depth + 1 as i32,
        );
        j = if depth == 0 {
            null_mut()
        } else {
            HSD_JObjGetNext(j)
        };
    }
}
unsafe extern "C" fn mnStageSel_PcImage(
    mut t: *mut HSD_TObj,
    mut img: *mut *mut HSD_ImageDesc,
    mut tlut: *mut *mut HSD_Tlut,
) {
    *img = (if !t.is_null() {
        (*t).imagedesc
    } else {
        null_mut()
    }) as *mut HSD_ImageDesc;
    *tlut = (if t.is_null() {
        null_mut()
    } else if (*t).tlut_no != TOBJ_TLUT_NONE && !(*t).tluttbl.is_null() {
        *(*t).tluttbl.offset((*t).tlut_no as isize)
    } else {
        (*t).tlut as *mut _HSD_Tlut
    }) as *mut HSD_Tlut;
}
unsafe extern "C" fn mnStageSel_PcModel(mut m: *mut StaticModelDesc) -> *mut HSD_JObj {
    let mut j: *mut HSD_JObj = HSD_JObjLoadJoint((*m).joint as *mut HSD_Joint);
    HSD_JObjAddAnimAll(j, (*m).animjoint, (*m).matanim_joint, (*m).shapeanim_joint);
    return j;
}
static mut mnStageSel_PcName: *mut HSD_JObj = null_mut();
pub unsafe extern "C" fn mnStageSel_PcArtOpen(
    mut archive: *mut HSD_Archive,
) -> i32 {
    let mut data: *mut C2RustUnnamed_18 = null_mut();
    let mut n: i32 = 0;
    mnStageSel_PcMexIcon = null_mut();
    mnStageSel_PcMex = null_mut();
    mnStageSel_PcCount = 0;
    mnStageSel_PcName = null_mut();
    if archive.is_null() {
        return 0 as i32;
    }
    data = HSD_ArchiveGetPublicAddress(
        archive,
        c"MnSelectStageDataTable".as_ptr(),
    ) as *mut C2RustUnnamed_18;
    if data.is_null() {
        return 0 as i32;
    }
    mnStageSel_804D6C98 = &raw mut (*data).x10 as *mut mnStageSel_804D6C98_t;
    mnStageSel_PcCount = SSS_ICON_COUNT_RETAIL;
    mnStageSel_PcMex = HSD_ArchiveGetPublicAddress(
        archive,
        c"mexMapData".as_ptr(),
    ) as *mut MexMapData;
    n = Mex_SssIconCount();
    if !mnStageSel_PcMex.is_null() && !(*mnStageSel_PcMex).icon_joint.is_null()
        && n > SSS_ICON_COUNT_RETAIL && n <= SSS_ICON_MAX && !Mex_SssTable().is_null()
    {
        mnStageSel_PcCount = n;
        mnStageSel_PcMexIcon = HSD_JObjLoadJoint((*mnStageSel_PcMex).icon_joint);
        HSD_JObjAddAnimAll(
            mnStageSel_PcMexIcon,
            (*mnStageSel_PcMex).icon_animjoint,
            (*mnStageSel_PcMex).icon_matanim,
            null_mut(),
        );
    } else {
        mnStageSel_PcMex = null_mut();
    }
    return mnStageSel_PcCount;
}
unsafe extern "C" fn mnStageSel_PcWidest(
    mut j: *mut HSD_JObj,
    mut best: *mut *mut HSD_TObj,
    mut bw: *mut i32,
    mut depth: i32,
) {
    while !j.is_null() && depth < 12 {
        let mut d: *mut HSD_DObj = HSD_JObjGetDObj(j) as *mut HSD_DObj;
        while !d.is_null() {
            let mut t: *mut HSD_TObj = if !(*d).mobj.is_null() {
                (*(*d).mobj).tobj
            } else {
                null_mut()
            };
            while !t.is_null() {
                if !(*t).imagedesc.is_null() && !(*(*t).imagedesc).image_ptr.is_null()
                    && (*(*t).imagedesc).width as i32 > *bw
                {
                    *bw = (*(*t).imagedesc).width as i32;
                    *best = t;
                }
                t = (*t).next;
            }
            d = (*d).next;
        }
        mnStageSel_PcWidest(
            HSD_JObjGetChild(j),
            best,
            bw,
            depth + 1 as i32,
        );
        j = HSD_JObjGetNext(j);
    }
}
pub unsafe extern "C" fn mnStageSel_PcArtName(
    mut i: i32,
    mut img: *mut *mut HSD_ImageDesc,
    mut tlut: *mut *mut HSD_Tlut,
) -> i32 {
    let mut name: *mut HSD_JObj = null_mut();
    let mut best: *mut HSD_TObj = null_mut();
    let mut bw: i32 = 0;
    *img = null_mut();
    *tlut = null_mut();
    if i < 0 || i >= mnStageSel_PcCount
        || mnStageSel_804D6C98.is_null()
    {
        return 0 as i32;
    }
    if mnStageSel_PcName.is_null() {
        let mut m: *mut StaticModelDesc = &raw mut (*mnStageSel_804D6C98).x30;
        mnStageSel_PcName = HSD_JObjLoadJoint((*m).joint as *mut HSD_Joint);
        HSD_JObjAddAnimAll(
            mnStageSel_PcName,
            (*m).animjoint,
            if !mnStageSel_PcMex.is_null() {
                (*mnStageSel_PcMex).stagename_matanim
            } else {
                (*m).matanim_joint
            },
            (*m).shapeanim_joint,
        );
    }
    name = mnStageSel_PcName;
    if !mnStageSel_PcMex.is_null() {
        do_anim(name, i);
    } else {
        do_anim(
            name,
            20 as i32
                * mnStageSel_803F06D0[i as usize].x9 as i32,
        );
    }
    mnStageSel_PcWidest(name, &raw mut best, &raw mut bw, 0);
    mnStageSel_PcImage(best, img, tlut);
    return (best != null_mut()) as i32;
}
pub unsafe extern "C" fn mnStageSel_PcArtIcon(
    mut i: i32,
    mut ext: *mut i32,
    mut type_0: *mut i32,
    mut img: *mut *mut HSD_ImageDesc,
    mut tlut: *mut *mut HSD_Tlut,
) -> i32 {
    let mut j: *mut HSD_JObj = null_mut();
    let mut at: *mut HSD_JObj = null_mut();
    let mut t: *mut HSD_TObj = null_mut();
    let mut area: i32 = 0;
    *img = null_mut();
    *tlut = null_mut();
    if i < 0 || i >= mnStageSel_PcCount
        || mnStageSel_804D6C98.is_null()
    {
        return 0 as i32;
    }
    if !mnStageSel_PcMex.is_null() {
        let mut row: *mut MexSssIcon = ((Mex_SssTable
           )() as *mut MexSssIcon)
            .offset(i as isize) as *mut MexSssIcon;
        *ext = (*row).stkind as i32;
        *type_0 = (*row).type_0 as i32;
        if (*row).type_0 as i32 == 0 {
            return 1 as i32;
        }
        if (*row).type_0 as i32 == 3 {
            j = mnStageSel_PcModel(&raw mut (*mnStageSel_804D6C98).x10);
            at = j;
            do_anim(j, 2);
        } else {
            j = mnStageSel_PcMexIcon;
            at = j;
            do_anim(
                j,
                if (*row).type_0 as i32 == 1 {
                    1 as i32
                } else {
                    i + 2 as i32
                },
            );
        }
    } else {
        let mut row_0: *mut stagelistinfo = (&raw mut mnStageSel_803F06D0
            as *mut stagelistinfo)
            .offset(i as isize) as *mut stagelistinfo;
        *ext = (*row_0).stkind as i32;
        *type_0 = if i == NUM_STAGES_RETAIL {
            3 as i32
        } else {
            2 as i32
        };
        if i < 22 {
            j = mnStageSel_PcModel(&raw mut (*mnStageSel_804D6C98).x40);
            at = (if i & 1 as i32 != 0 {
                (*j).child
            } else {
                (*(*j).child).next
            }) as *mut HSD_JObj;
            HSD_JObjReqAnimAll(j, 0.0_f32);
            HSD_JObjReqAnimAllByFlags(
                at,
                0x10,
                ((*row_0).x9 as i32 / 2 as i32
                    + 2 as i32) as f32,
            );
            HSD_JObjAnimAll(j);
        } else if i < 24 {
            j = mnStageSel_PcModel(&raw mut (*mnStageSel_804D6C98).x0);
            at = j;
            do_anim(j, (*row_0).x9 as i32 - 0x14 as i32);
        } else if i < NUM_STAGES_RETAIL {
            j = mnStageSel_PcModel(&raw mut (*mnStageSel_804D6C98).x20);
            at = j;
            do_anim(j, (*row_0).x9 as i32 - 0x16 as i32);
        } else {
            j = mnStageSel_PcModel(&raw mut (*mnStageSel_804D6C98).x10);
            at = j;
            do_anim(j, 2);
        }
    }
    if !at.is_null() {
        mnStageSel_PcBestTex(at, &raw mut t, &raw mut area, 0);
    }
    mnStageSel_PcImage(t, img, tlut);
    return 1 as i32;
}
pub const NULL: i32 = 0;
