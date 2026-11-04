//! Port of src/melee/lb/lbcardgame.c (the game's save-file layer over lbcardnew: card probing, the
//! save/load task state machine, the "saving" overlay scene). Verified against the C (whole TU,
//! shipping contraction) by tests/tu_lbcardgame.rs (`python rust/melee_runtime/gen/mtu/run_tu.py lbcardgame`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn CARDProbe(a0: i32) -> i32;
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn HSD_CObjLoadDesc(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj = crate::tu::cobj::HSD_CObjLoadDesc(a0);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) = crate::tu::gobj::HSD_GObj_JObjCallback(a0, a1);
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::jobj::HSD_JObjLoadJoint(a0);
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Mex_Enabled(a0: *const i8) -> i32;
    fn OSGetTime() -> i64;
    fn OSTicksToCalendarTime(a0: i64, a1: *mut OSCalendarTime);
    fn gmMainLib_8015FC74() -> i32;
    fn gmMainLib_GetCardData() -> *mut GmCardData;
    fn gmMainLib_GetNameTagDataBanks() -> *mut NameTagDataBank;
    fn gm_80164ABC() -> i32;
    fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
    fn gm_GetPowerTime() -> *mut i32;
    fn lbArchive_80016DBC(a0: *const i8, a1: *mut c_void, a2: *const i8, a3: i32) -> *mut c_void;
    fn lbCardNew_CompleteNextTask() -> i32 = crate::tu::lbcardnew::lbCardNew_CompleteNextTask();
    fn lbCardNew_ProbeEx(a0: i32) -> i32 = crate::tu::lbcardnew::lbCardNew_ProbeEx(a0);
    fn lbLang_IsSettingJP() -> i32 = crate::tu::lblanguage::lbLang_IsSettingJP() as _;
    fn lb_8001B7E0(a0: i32, a1: *mut i8, a2: *mut c_void, a3: *mut c_void, a4: *mut i32) -> u32 = crate::tu::lbcardnew::lb_8001B7E0(a0, a1, a2, a3, a4);
    fn lb_8001BA44(a0: i32, a1: *const i8, a2: *mut c_void) -> i32 = crate::tu::lbcardnew::lb_8001BA44(a0, a1, a2);
    fn lb_8001BC18(a0: i32, a1: *mut i8, a2: *mut *mut c_void, a3: *mut c_void, a4: *mut i8, a5: *mut c_void, a6: *mut c_void, a7: *mut i32) -> i32;
    fn lb_8001BD34(a0: i32, a1: *const i8, a2: *mut c_void, a3: *mut c_void) -> i32 = crate::tu::lbcardnew::lb_8001BD34(a0, a1, a2, a3);
    fn lb_8001BE30(a0: i32, a1: *const i8, a2: *mut c_void, a3: *mut i8, a4: *mut c_void, a5: *mut c_void, a6: *mut c_void, a7: *mut c_void) -> i32;
    fn un_80304470() -> i32;
    fn wait_idle();
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::gobj_native::HSD_GObj_CameraKind;
    pub use crate::tu::gobj_native::HSD_GObj_JObjKind;
    pub use crate::tu::perf::__OSBusClock as OS_BusClock;
}
// END GENERATED EXT

pub const LbCardStatus_0: u32 = 0;
pub const LbCardStatus_1: u32 = 1;
pub const LbCardStatus_2: u32 = 2;
pub const LbCardStatus_3: u32 = 3;
pub const LbCardStatus_4: u32 = 4;

pub const LbCardResult_Ready: u32 = 0;
pub const LbCardResult_2: u32 = 2;
pub const LbCardResult_Busy: u32 = 11;
pub const LbCardResult_Invalid: u32 = 13;

const CARD_COMMENT_SIZE: usize = 64;
const HSD_GOBJ_PLINK_MEMCARD: u8 = 63 - 2;

pub use super::lbcardnew::LbCardEntry;

/// The file-static state record at 433318 (`state`); field comments give the console offsets.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LbCardGameState {
    /// +0
    pub probe_status: i32,
    /// +4
    pub unk_status: i32,
    /// +8
    pub card_status: u32,
    /// +C
    pub dirty: i32,
    /// +10
    pub x10: u32,
    /// +14
    pub failed: i32,
    /// +18
    pub enable: i32,
    /// +1C
    pub comment: [i8; CARD_COMMENT_SIZE],
    /// +5C
    pub icon_data: *mut *mut c_void,
    /// +60 (`ssize_t`); always set to 0 by `lbCardGame_LoadArchive`
    pub jobj_translate_idx: isize,
    /// +64
    pub scene_data: *mut SceneDesc,
}
melee_core::gc_layout!(LbCardGameState, "lbcardgame_state", size 0x68, align 4, [
    probe_status @ 0x0: 4 "bool",
    unk_status @ 0x4: 4 "bool",
    card_status @ 0x8: 4 "LbCardStatus",
    dirty @ 0xC: 4 "bool",
    x10 @ 0x10: 4 "statex10",
    failed @ 0x14: 4 "bool",
    enable @ 0x18: 4 "bool",
    comment @ 0x1C: 64 "char[64]",
    icon_data @ 0x5C: 4 "void**",
    jobj_translate_idx @ 0x60: 4 "ssize_t",
    scene_data @ 0x64: 4 "SceneDesc*",
]);

/// 433318
pub static mut state: LbCardGameState = unsafe { core::mem::zeroed() };

/// `lb_803BAB60` (the card-file header template).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LbCardHeader {
    pub x0: u32,
    pub x4: u32,
    pub x8: u32,
    pub pad: [u32; 2],
}
melee_core::gc_layout!(LbCardHeader, "lb_803BAB60", size 0x14, align 4, [
    x0 @ 0x0: 4 "u32",
    x4 @ 0x4: 4 "u32",
    x8 @ 0x8: 4 "u32",
    pad @ 0xC: 8 "u32[2]",
]);
pub static mut lb_803BAB60: LbCardHeader = LbCardHeader { x0: 0x2000100, x4: 0, x8: 0x300, pad: [0; 2] };

const fileType_SaveData: i32 = 0;
const fileType_NameTag: i32 = 1;
const fileType_3: i32 = 3;

const fn entry(size: usize, flags: i32) -> LbCardEntry {
    LbCardEntry { file_size: size as i32, file_flags: flags, data: null_mut() }
}
pub static mut manifest: [LbCardEntry; 10] = [
    entry(0, fileType_3),
    entry(core::mem::size_of::<GmSaveData>(), fileType_SaveData),
    entry(core::mem::size_of::<NameTagDataBank>(), fileType_NameTag),
    entry(core::mem::size_of::<NameTagDataBank>(), fileType_NameTag),
    entry(core::mem::size_of::<NameTagDataBank>(), fileType_NameTag),
    entry(core::mem::size_of::<NameTagDataBank>(), fileType_NameTag),
    entry(core::mem::size_of::<NameTagDataBank>(), fileType_NameTag),
    entry(core::mem::size_of::<NameTagDataBank>(), fileType_NameTag),
    entry(core::mem::size_of::<NameTagDataBank>(), fileType_NameTag),
    LbCardEntry { file_size: -1, file_flags: 0, data: null_mut() },
];

/// `char filename[] = "SuperSmashBros0110290334"`
pub static mut filename: [u8; 25] = *b"SuperSmashBros0110290334\0";

/// 01C600
pub unsafe extern "C" fn lb_8001C600() {
    melee_core::ct!("lb_8001C600");
    let prev = state.probe_status;
    state.probe_status = ext::CARDProbe(0);
    if state.probe_status != prev {
        state.unk_status = 1;
    }
}

/// 01C658
pub unsafe extern "C" fn lb_8001C658() -> *mut i8 {
    melee_core::ct!("lb_8001C658");
    let ticks = ext::OSGetTime();
    let timer_clock = (ext::OS_BusClock / 4) as i64;
    let seconds = (ticks / timer_clock) as u32;
    let mut time: OSCalendarTime = core::mem::zeroed();
    ext::OSTicksToCalendarTime((seconds as u64).wrapping_mul(timer_clock as u64) as i64, &mut time);
    for i in 0..CARD_COMMENT_SIZE {
        state.comment[i] = 0;
    }
    let title = if ext::lbLang_IsSettingJP() != 0 {
        "大乱闘スマッシュブラザーズＤＸ  セーブデータ"
    } else {
        "Super Smash Bros. Melee         Game Data"
    };
    let text = format!("{} {:4}/{:02}/{:02}", title, time.year, time.mon.wrapping_add(1), time.mday);
    let bytes = text.as_bytes();
    let n = bytes.len().min(CARD_COMMENT_SIZE - 1);
    for k in 0..n {
        state.comment[k] = bytes[k] as i8;
    }
    state.comment[n] = 0;
    state.comment.as_mut_ptr()
}

/// `getCurrentIcon`
pub unsafe extern "C" fn getCurrentIcon() -> *mut c_void {
    melee_core::ct!("getCurrentIcon");
    if ext::Mex_Enabled(b"use_save_banner_1\0".as_ptr() as *const i8) != 0 {
        return *state.icon_data.add(1);
    }
    let idx = if ext::un_80304470() != 0 {
        2
    } else if ext::gm_80164ABC() != 0 {
        1
    } else {
        0
    };
    *state.icon_data.add(idx)
}

/// 01C87C
pub unsafe extern "C" fn lb_8001C87C() -> u32 {
    melee_core::ct!("lb_8001C87C");
    gc_stage();
    let r = ext::lb_8001B7E0(
        0,
        filename.as_mut_ptr() as *mut i8,
        (&raw mut manifest) as *mut c_void,
        (&raw mut lb_803BAB60) as *mut c_void,
        (&raw mut state.unk_status) as *mut i32,
    );
    gc_unstage();
    r
}

/// 01C8BC
pub unsafe extern "C" fn lb_8001C8BC() -> i32 {
    melee_core::ct!("lb_8001C8BC");
    assert!(state.enable != 0, "lbcardgame.c:320 enable");
    let a4 = lb_8001C658();
    let a5 = getCurrentIcon();
    gc_stage();
    ext::lb_8001BC18(
        0,
        filename.as_mut_ptr() as *mut i8,
        (&raw mut manifest) as *mut *mut c_void,
        (&raw mut lb_803BAB60) as *mut c_void,
        a4,
        a5,
        *state.icon_data.add(3),
        (&raw mut state.unk_status) as *mut i32,
    )
}

/// `updateCardStatus`
pub unsafe extern "C" fn updateCardStatus() -> u32 {
    melee_core::ct!("updateCardStatus");
    let unk_status = state.unk_status;
    state.unk_status = 0;
    match state.card_status {
        0 => {
            if unk_status != 0 {
                state.card_status = LbCardStatus_1;
            }
        }
        3 => {
            if ext::lbCardNew_ProbeEx(0) as u32 != LbCardResult_Ready {
                state.card_status = LbCardStatus_4;
            }
        }
        4 => {
            if unk_status != 0 && ext::lbCardNew_ProbeEx(0) as u32 == LbCardResult_Ready {
                state.card_status = LbCardStatus_3;
            }
        }
        _ => {}
    }
    state.card_status
}

/// 01CBAC
pub unsafe extern "C" fn lbCardGame_SetCardStatus(status: u32) {
    melee_core::ct!("lbCardGame_SetCardStatus");
    state.card_status = status;
}

/// 01CBBC
pub unsafe extern "C" fn lb_8001CBBC() -> u32 {
    melee_core::ct!("lb_8001CBBC");
    if updateCardStatus() != LbCardStatus_0 {
        return LbCardResult_Invalid;
    }
    gc_stage();
    let result = ext::lb_8001BD34(
        0,
        filename.as_ptr() as *const i8,
        (&raw mut manifest) as *mut c_void,
        (&raw mut state.unk_status) as *mut c_void,
    ) as u32;
    gc_unstage();
    if GC_IMAGE {
        let errs: SVec<i32> = (1..9).map(|i| crate::tu::lbcardnew::lb_8001B6E0(i)).collect();
        eprintln!("gw: card: load result {} (per-file errors, save + 7 name-tag banks: {:?})", result, errs);
    }
    if result != LbCardResult_Ready && result != LbCardResult_2 {
        state.card_status = LbCardStatus_2;
    }
    result
}

/// 01CC30
pub unsafe extern "C" fn fn_8001CC30(arg0: i32) {
    melee_core::ct!("fn_8001CC30");
    if arg0 != 0 {
        state.card_status = LbCardStatus_2;
    }
}

/// 01CC4C
pub unsafe extern "C" fn lb_8001CC4C() -> i32 {
    melee_core::ct!("lb_8001CC4C");
    ext::lb_8001BA44(0, filename.as_ptr() as *const i8, (&raw mut state.unk_status) as *mut c_void)
}

/// `dont_inline_helper`
pub unsafe extern "C" fn dont_inline_helper() -> u32 {
    melee_core::ct!("dont_inline_helper");
    if updateCardStatus() != 0 {
        return LbCardResult_Invalid;
    }
    let icon = getCurrentIcon();
    let comment = lb_8001C658();
    gc_stage();
    ext::lb_8001BE30(
        0,
        filename.as_ptr() as *const i8,
        (&raw mut manifest) as *mut c_void,
        comment,
        icon,
        *state.icon_data.add(3),
        (&raw mut state.unk_status) as *mut c_void,
        fn_8001CC30 as *const () as *mut c_void,
    ) as u32
}

/// 01CC84
pub unsafe extern "C" fn lb_8001CC84() {
    melee_core::ct!("lb_8001CC84");
    loop {
        match state.x10 {
            0 => {
                if state.dirty != 0 {
                    state.dirty = 0;
                    if state.card_status == 0 {
                        if dont_inline_helper() != LbCardResult_Busy {
                            state.failed = 1;
                        } else {
                            state.x10 = 1;
                        }
                    }
                }
            }
            1 => {
                let result = ext::lbCardNew_CompleteNextTask();
                if result != LbCardResult_Busy as i32 {
                    if result != 0 {
                        state.failed = 1;
                    }
                    state.x10 = 0;
                }
            }
            _ => {}
        }
        if !(state.x10 != 1 && state.dirty != 0) {
            break;
        }
    }
}

/// 01CDB4
pub unsafe extern "C" fn lb_8001CDB4() {
    melee_core::ct!("lb_8001CDB4");
    while state.dirty != 0 || state.x10 != 0 {
        ext::wait_idle();
        lb_8001CC84();
    }
}

/// 01CE00
pub unsafe extern "C" fn lbCardGame_SaveChanges() {
    melee_core::ct!("lbCardGame_SaveChanges");
    assert!(state.enable != 0, "lbcardgame.c:675 enable");
    let add = ext::gmMainLib_8015FC74();
    let p = ext::gm_GetPowerTime();
    *p = (*p).wrapping_add(add);
    state.dirty = 1;
}

/// 01CE78
pub unsafe extern "C" fn lbCardGame_DecideGameMode() -> u8 {
    melee_core::ct!("lbCardGame_DecideGameMode");
    updateCardStatus();
    if state.card_status != LbCardStatus_0 && state.card_status != LbCardStatus_4 {
        return GM_MEMCARD as u8;
    }
    GM_COUNT as u8
}

/// `gobj1_Proc`
pub unsafe extern "C" fn gobj1_Proc(gobj: *mut HSD_GObj) {
    melee_core::ct!("gobj1_Proc");
    ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
}

/// `gobj0_RenderFunc`
pub unsafe extern "C" fn gobj0_RenderFunc(gobj: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("gobj0_RenderFunc");
    if state.x10 == 1 {
        ext::HSD_GObj_803910D8(gobj, arg1);
    }
}

pub static mut jobj_translate: [Vec2; 1] = [Vec2 { x: 21.5, y: 16.5 }];

/// `HSD_JObjMtxIsDirty` (jobj.h inline)
unsafe fn jobj_mtx_is_dirty(jobj: *mut HSD_JObj) -> bool {
    assert!(!jobj.is_null());
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// `HSD_JObjSetMtxDirty` (jobj.h macro)
unsafe fn jobj_set_mtx_dirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !jobj_mtx_is_dirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `HSD_JObjSetTranslateX` (jobj.h inline)
unsafe fn jobj_set_translate_x(jobj: *mut HSD_JObj, x: f32) {
    assert!(!jobj.is_null());
    (*jobj).translate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        jobj_set_mtx_dirty(jobj);
    }
}

/// `HSD_JObjSetTranslateY` (jobj.h inline)
unsafe fn jobj_set_translate_y(jobj: *mut HSD_JObj, y: f32) {
    assert!(!jobj.is_null());
    (*jobj).translate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        jobj_set_mtx_dirty(jobj);
    }
}

/// 01CF18
pub unsafe extern "C" fn lbCardGame_InitScene() {
    melee_core::ct!("lbCardGame_InitScene");
    if !state.scene_data.is_null() {
        let scene = &*state.scene_data;
        let mut gobj = ext::GObj_Create(HSD_GOBJ_CLASS_MEMCARD as u16, HSD_GOBJ_PLINK_MEMCARD, 0);
        let cobj = ext::HSD_CObjLoadDesc((*scene.cameras).desc);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
        ext::GObj_SetupGXLinkMax(gobj, Some(gobj0_RenderFunc), 14);
        (*gobj).gxlink_prios = 1 << 19;

        gobj = ext::GObj_Create(HSD_GOBJ_CLASS_MEMCARD as u16, HSD_GOBJ_PLINK_MEMCARD, 0);
        let model = *scene.models;
        let jobj = ext::HSD_JObjLoadJoint((*model).joint);
        let tr = jobj_translate.as_ptr().offset(state.jobj_translate_idx);
        jobj_set_translate_x(jobj, (*tr).x);
        let tr = jobj_translate.as_ptr().offset(state.jobj_translate_idx);
        jobj_set_translate_y(jobj, (*tr).y);

        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        ext::GObj_SetupGXLink(gobj, Some(ext::cb::HSD_GObj_JObjCallback), 19, 0);
        ext::HSD_GObj_SetupProc(gobj, Some(gobj1_Proc), 0);
        ext::gm_8016895C(jobj, *scene.models, 0);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
    }
}

/// 01D164
pub unsafe extern "C" fn lbCardGame_LoadArchive(jobj_translate_idx: i32) {
    melee_core::ct!("lbCardGame_LoadArchive");
    if state.icon_data.is_null() {
        ext::lbArchive_80016DBC(
            b"LbMcGame.\0".as_ptr() as *const i8,
            (&raw mut state.icon_data) as *mut c_void,
            b"MemCardIconData\0".as_ptr() as *const i8,
            0,
        );
        ext::lbArchive_80016DBC(
            b"NtMemAc\0".as_ptr() as *const i8,
            (&raw mut state.scene_data) as *mut c_void,
            b"ScNtcCommon_scene_data\0".as_ptr() as *const i8,
            0,
        );
        state.jobj_translate_idx = jobj_translate_idx as isize;
        state.enable = 1;
    }
}

/// 01D1F4
pub unsafe extern "C" fn lbCardGame_Reset() {
    melee_core::ct!("lbCardGame_Reset");
    state.icon_data = null_mut();
    state.scene_data = null_mut();
    state.enable = 0;
    state.dirty = 0;
    state.x10 = 0;
    state.failed = 0;
}

/// 01D21C
pub unsafe extern "C" fn lbCardGame_Init() {
    melee_core::ct!("lbCardGame_Init");
    state.probe_status = ext::CARDProbe(0);
    state.unk_status = 0;
    state.card_status = LbCardStatus_0;
    lbCardGame_Reset();
    manifest[1].data = (&raw mut (*ext::gmMainLib_GetCardData()).save_data) as *mut c_void;
    for i in 0..GM_NAMETAG_BANK_COUNT as usize {
        manifest[2 + i].data = ext::gmMainLib_GetNameTagDataBanks().add(i) as *mut c_void;
    }
}

// ---- TARGET_PC: the save files in GameCube byte order ----------------------------------------------------------
use melee_core::gcdecode::GcDecode;
use std::vec::Vec as SVec;
//
// The C build keeps game memory big-endian, so its lbcardnew writes GmSaveData and the name-tag banks to the card as
// GameCube bytes. Here those structs are native, so with a card installed (`GC_IMAGE`, set by the engine's card
// shim) the manifest points at staging buffers holding the GameCube image instead: each call that hands the
// manifest to lbcardnew first encodes native -> image, and each read decodes image -> native afterwards. That is the
// C's in-place semantics (a read overwrites the live struct with whatever came off the card, valid or not), and the
// card bytes match the C build's, so a save made by one build loads in the other. Off (the TU oracle and the
// save-less golden runs), the manifest points at the native structs exactly as the C does.

/// Set by the card shim: the manifest's data is the GameCube image of the save structs.
pub static mut GC_IMAGE: bool = false;

/// The GameCube byte each native byte of `T` is decoded from (`u32::MAX`: none). `T` must be pointer-free plain data
/// whose decode is a byte permutation (scalars swapped, bitfield runs copied); `gc_image` checks that once.
pub fn gc_permutation<T: GcDecode>() -> SVec<u32> {
    let n = core::mem::size_of::<T>();
    let g = T::GC_BYTES as usize;
    assert!(g < 0xFF00, "gc_permutation: {} bytes", g);
    let relocs = melee_core::gcdecode::RelocSet::default();
    let decode = |bytes: &[u8]| -> SVec<u8> {
        let mut d = melee_core::gcdecode::GcDecoder::new(bytes, &relocs);
        let v = T::gc_decode(&mut d, 0);
        let p = &v as *const T as *const u8;
        let out = unsafe { core::slice::from_raw_parts(p, n) }.to_vec();
        core::mem::forget(v);
        out
    };
    let lo: SVec<u8> = (0..g).map(|i| i as u8).collect();
    let hi: SVec<u8> = (0..g).map(|i| ((i >> 8) + 1) as u8).collect();
    let (nl, nh) = (decode(&lo), decode(&hi));
    let perm: SVec<u32> = (0..n).map(|j| if nh[j] == 0 { u32::MAX } else { (((nh[j] as u32) - 1) << 8) | nl[j] as u32 }).collect();
    // a byte permutation decodes any image to the same bytes the map predicts
    let probe: SVec<u8> = (0..g).map(|i| (i as u32).wrapping_mul(2654435761).rotate_left(7) as u8 | 1).collect();
    let np = decode(&probe);
    for j in 0..n {
        let want = if perm[j] == u32::MAX { 0 } else { probe[perm[j] as usize] };
        assert_eq!(np[j], want, "gc_permutation: byte {} of a {}-byte struct is not a plain copy", j, n);
    }
    perm
}

/// `GmSaveData`'s word tables that the C indexes past their declared length into the padding behind them (the
/// declarations are short; the padding is the rest of the table): `unk_30.x4C` (s32[4] + 84 bytes, the Target Test
/// best times, gmMainLib_8015D450 by selkind 0..24) and `x1A70` (s32[4] + 0xBC bytes, the 51 event scores,
/// gmMainLib_8015CF70). The C build stores those words big-endian like the declared ones, so the image takes the
/// padding bytes as words too: (native byte offset, GC byte offset, words).
pub fn save_word_tables() -> [(usize, usize, usize); 2] {
    use core::mem::offset_of;
    [
        (offset_of!(GmSaveData, unk_30) + offset_of!(gmm_retval_EDBC, x4C), 0x30 + 0x4C, 25),
        (offset_of!(GmSaveData, x1A70), 0x208, 51),
    ]
}

/// [`gc_permutation`] of `GmSaveData` with [`save_word_tables`] as big-endian words.
pub fn save_permutation() -> SVec<u32> {
    let mut perm = gc_permutation::<GmSaveData>();
    for (nat, gc, words) in save_word_tables() {
        for w in 0..words {
            for b in 0..4 {
                perm[nat + 4 * w + b] = (gc + 4 * w + 3 - b) as u32;
            }
        }
    }
    perm
}

/// native -> image (bytes the native struct does not cover keep what the image held: what was read off the card)
pub unsafe fn gc_encode(perm: &[u32], native: *const u8, image: &mut [u8]) {
    for (j, &k) in perm.iter().enumerate() {
        if k != u32::MAX {
            image[k as usize] = *native.add(j);
        }
    }
}

/// image -> native
pub unsafe fn gc_decode_into(perm: &[u32], image: &[u8], native: *mut u8) {
    for (j, &k) in perm.iter().enumerate() {
        if k != u32::MAX {
            *native.add(j) = image[k as usize];
        }
    }
}

struct GcImage {
    save_perm: SVec<u32>,
    tag_perm: SVec<u32>,
    save: SVec<u8>,
    tags: SVec<SVec<u8>>,
}
static mut IMAGE: Option<GcImage> = None;

unsafe fn image() -> &'static mut GcImage {
    if IMAGE.is_none() {
        IMAGE = Some(GcImage {
            save_perm: save_permutation(),
            tag_perm: gc_permutation::<NameTagDataBank>(),
            save: std::vec![0; <GmSaveData as GcDecode>::GC_BYTES as usize],
            tags: (0..GM_NAMETAG_BANK_COUNT as usize).map(|_| std::vec![0; <NameTagDataBank as GcDecode>::GC_BYTES as usize]).collect(),
        });
    }
    IMAGE.as_mut().unwrap()
}

/// One line with the fields a persistence test checks (MELEE_CARD_DIAG=1 also logs every write).
unsafe fn log_save(what: &str) {
    let sd = &*(native_save() as *const GmSaveData);
    eprintln!(
        "gw: card: {}: unlocked_characters={:#06x} rumble={:?} sound_balance={} deflicker={} language={} trophies={} vs_matches(time/stock)={}/{} vs_play_time={}",
        what, sd.unlocked_characters, sd.x1CB0.rumble_enabled, sd.x1CB0.sound_balance, sd.x1CB0.deflicker, sd.x1CB0.saved_language,
        sd.trophy_count, sd.time_matches, sd.stock_matches, sd.x1A38
    );
    // the 1P records: the event clears and scores, the per-fighter records (Classic / Adventure / All-Star / Target
    // Test / Home-Run / Multi-Man) and the Stadium words, as an FNV-1a each so a run can tell they changed
    let fnv = |p: *const u8, n: usize| core::slice::from_raw_parts(p, n).iter().fold(0x811C9DC5u32, |h, &b| (h ^ b as u32).wrapping_mul(0x0100_0193));
    let ev = (&raw const sd.x1A70) as *const i32;
    let ev_sum: i64 = (0..51).map(|i| *ev.add(i) as i64).sum();
    eprintln!(
        "gw: card: {}: events_cleared={:#015x} event_score_sum={} fighter_records_fnv={:08x} stadium_fnv={:08x} mode_records_fnv={:08x}",
        what, sd.x1A68, ev_sum,
        fnv((&raw const sd.x1F2C) as *const u8, core::mem::size_of_val(&sd.x1F2C)),
        fnv((&raw const sd.x1B40) as *const u8, (&raw const sd.x1CB0) as usize - (&raw const sd.x1B40) as usize),
        // unk_8 .. unk_1A8: the per-mode record tables (the Home-Run distances are unk_30.xB0)
        fnv((&raw const sd.unk_8) as *const u8, (&raw const sd.time_matches) as usize - (&raw const sd.unk_8) as usize)
    );
}

unsafe fn native_save() -> *mut u8 {
    (&raw mut (*ext::gmMainLib_GetCardData()).save_data) as *mut u8
}

/// Before a call that takes the manifest: point it at the image and refresh the image from the native structs.
unsafe fn gc_stage() {
    if !GC_IMAGE {
        return;
    }
    if std::env::var_os("MELEE_CARD_DIAG").is_some() {
        log_save("native -> save image");
    }
    let im = image();
    gc_encode(&im.save_perm, native_save(), &mut im.save);
    manifest[1].data = im.save.as_mut_ptr() as *mut c_void;
    manifest[1].file_size = im.save.len() as i32;
    for i in 0..GM_NAMETAG_BANK_COUNT as usize {
        gc_encode(&im.tag_perm, ext::gmMainLib_GetNameTagDataBanks().add(i) as *const u8, &mut im.tags[i]);
        manifest[2 + i].data = im.tags[i].as_mut_ptr() as *mut c_void;
        manifest[2 + i].file_size = im.tags[i].len() as i32;
    }
}

/// After a read: the native structs take what the card read left in the image.
unsafe fn gc_unstage() {
    if !GC_IMAGE {
        return;
    }
    let im = image();
    gc_decode_into(&im.save_perm, &im.save, native_save());
    log_save("save image -> native");
    // MELEE_CARD_DUMP=PREFIX: the GameCube image the card read (PREFIX.save.bin, PREFIX.tagN.bin), to compare a save
    // field by field with the C build's (the files on the card are encrypted, so one changed byte changes the rest)
    if let Some(pre) = std::env::var_os("MELEE_CARD_DUMP") {
        let pre = pre.to_string_lossy().into_owned();
        let _ = std::fs::write(format!("{pre}.save.bin"), &im.save);
        for (i, t) in im.tags.iter().enumerate() {
            let _ = std::fs::write(format!("{pre}.tag{i}.bin"), t);
        }
    }
    for i in 0..GM_NAMETAG_BANK_COUNT as usize {
        gc_decode_into(&im.tag_perm, &im.tags[i], ext::gmMainLib_GetNameTagDataBanks().add(i) as *mut u8);
    }
}
