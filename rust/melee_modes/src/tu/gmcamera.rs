//! src/melee/gm/gmcamera.c: the VS-mode camera UI (the state machine over `gmCamera_VsCamUiState`, its
//! digit and text display, the snapshot and the Pokemon-less camera options). All 23 functions are
//! checked bit for bit against the C by a state oracle (tests/tu_gmcamera.rs over
//! gen/turef/tu_gmcamera_ref.c). The `static inline` helpers of the C (sfx*, HSD_JObjSetTranslateX,
//! the FreeTexts pair, update_selection, 31FC_inline) are ported as private functions.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs, unused_unsafe)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
use core::ffi::c_void;
use melee_types::*;

/// `gmCameraUnkStruct4`: one per side (0 = left, 1 = right) of the camera-select UI.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct gmCameraUnkStruct4 {
    pub x0: bool,
    pub x4: i32,
    pub x8: i32,
    pub xC: i32,
}

/// `gmCameraUnkStruct` (gmcamera.h): the VS camera UI's state.
#[repr(C)]
pub struct gmCameraUnkStruct {
    /// Interface .dat file loaded from `gmCamera_801A31FC`.
    pub ifvscam: *mut c_void,
    pub x4: *mut HSD_JObj,
    pub x8: *mut HSD_JObj,
    pub xC: i32,
    pub x10: i32,
    pub x14: i32,
    pub x18: i32,
    pub snap_image: *mut c_void,
    pub x20: i32,
    pub x24: [gmCameraUnkStruct4; 2],
    pub x44: i32,
    pub x48: [*mut HSD_Text; 3],
    pub x54: i32,
}

/// `gmCameraUnkFuncTable`: one row per mode of the camera state machine.
#[repr(C)]
pub struct gmCameraUnkFuncTable {
    pub flags: gmCameraUnkFuncFlags,
    pub x4: Option<unsafe fn()>,
    pub x8: Option<unsafe fn()>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct gmCameraUnkFuncFlags {
    pub x0: u8,
    pub x2: u16,
}

/// `static gmCameraUnkStruct gmCamera_VsCamUiState;` (zeroed at boot).
pub static mut gmCamera_VsCamUiState: gmCameraUnkStruct = gmCameraUnkStruct {
    ifvscam: core::ptr::null_mut(),
    x4: core::ptr::null_mut(),
    x8: core::ptr::null_mut(),
    xC: 0,
    x10: 0,
    x14: 0,
    x18: 0,
    snap_image: core::ptr::null_mut(),
    x20: 0,
    x24: [gmCameraUnkStruct4 { x0: false, x4: 0, x8: 0, xC: 0 }; 2],
    x44: 0,
    x48: [core::ptr::null_mut(); 3],
    x54: 0,
};

/// `f32 gmCamera_803DA630[12]`: the text placements (x/y, font size) of the two sides and the header.
pub static gmCamera_803DA630: [f32; 12] = [
    0.6, 0.6, 40.0, 416.0, 0.6, 0.6, 340.0, 416.0, 0.6, 0.6, 40.0, 44.0,
];

/// `gmCamera_VsCamStateTable[9]`: `{ flags, on_enter, on_update }` per mode index.
pub static gmCamera_VsCamStateTable: [gmCameraUnkFuncTable; 9] = [
    gmCameraUnkFuncTable { flags: gmCameraUnkFuncFlags { x0: 1, x2: 0 }, x4: Some(gmCamera_801A26C0), x8: Some(gmCamera_801A2798) },
    gmCameraUnkFuncTable { flags: gmCameraUnkFuncFlags { x0: 0, x2: 4 }, x4: Some(gmCamera_801A2800), x8: Some(gmCamera_801A28AC) },
    gmCameraUnkFuncTable { flags: gmCameraUnkFuncFlags { x0: 0, x2: 2 }, x4: Some(gmCamera_801A292C), x8: Some(gmCamera_801A2AAC) },
    gmCameraUnkFuncTable { flags: gmCameraUnkFuncFlags { x0: 0, x2: 0x54 }, x4: None, x8: Some(gmCamera_801A2BB0) },
    gmCameraUnkFuncTable { flags: gmCameraUnkFuncFlags { x0: 0, x2: 0x34 }, x4: None, x8: Some(gmCamera_801A2BB0) },
    gmCameraUnkFuncTable { flags: gmCameraUnkFuncFlags { x0: 0, x2: 0x94 }, x4: None, x8: Some(gmCamera_801A2BB0) },
    gmCameraUnkFuncTable { flags: gmCameraUnkFuncFlags { x0: 0, x2: 0x1204 }, x4: Some(gmCamera_801A2BF0), x8: Some(gmCamera_801A2D44) },
    gmCameraUnkFuncTable { flags: gmCameraUnkFuncFlags { x0: 0, x2: 0x1604 }, x4: Some(gmCamera_801A2FBC), x8: Some(gmCamera_801A2FFC) },
    gmCameraUnkFuncTable { flags: gmCameraUnkFuncFlags { x0: 0, x2: 0x1A14 }, x4: None, x8: Some(gmCamera_801A2BB0) },
];

/// Pad button masks (dolphin/pad.h).
const PAD_BUTTON_LEFT: u32 = 1 << 0;
const PAD_BUTTON_RIGHT: u32 = 1 << 1;
const PAD_TRIGGER_Z: u32 = 1 << 4;
const PAD_BUTTON_A: u32 = 1 << 8;
const PAD_BUTTON_B: u32 = 1 << 9;
const PAD_BUTTON_START: u32 = 1 << 12;
const PAD_STICK_LEFT: u32 = 1 << 18;
const PAD_STICK_RIGHT: u32 = 1 << 19;

/// Calls into TUs that are not ported yet (engine, UI, snap, archive, audio, pad). Each is a settable hook;
/// an unset hook panics. The C's `static inline` helpers (sfx*, JObj setters) are reproduced below.
pub mod ext {
    use super::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// The digit table `HSD_SisLib_804D1124` (a `SisLibUnkStruct` chain whose `x08_arr` the TU reads).
    hook!(HSD_SISLIB_804D1124_ARR, HSD_SisLib_804D1124_arr, () -> *const u8);
    // sysdolphin/baselib/sislib.c
    pub static mut HSD_SISLIB_803A5ACC: Option<fn(i32, i32, f32, f32, f32, f32, f32) -> *mut HSD_Text> = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A5ACC(a: i32, b: i32, x: f32, y: f32, w: f32, h: f32, s: f32) -> *mut HSD_Text {
                match unsafe { HSD_SISLIB_803A5ACC } {
                    Some(f) => f(a, b, x, y, w, h, s),
                    None => { return unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a as _, b as _, x as _, y as _, w as _, h as _, s as _) as _ }; },
                }
            }
    pub static mut HSD_SISLIB_803A6530: Option<fn(i32, i32, i32) -> *mut u8> = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A6530(a: i32, b: i32, c: i32) -> *mut u8 {
                match unsafe { HSD_SISLIB_803A6530 } {
                    Some(f) => f(a, b, c),
                    None => { return unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6530(a as _, b as _, c as _) as _ }; },
                }
            }
    pub static mut HSD_SISLIB_803A660C: Option<fn(i32, i32, i32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A660C(a: i32, b: i32, c: i32)  {
                match unsafe { HSD_SISLIB_803A660C } {
                    Some(f) => f(a, b, c),
                    None => { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A660C(a as _, b as _, c as _); } return; },
                }
            }
    pub static mut HSD_SISLIB_803A6368: Option<fn(*mut HSD_Text, i32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A6368(text: *mut HSD_Text, v: i32)  {
                match unsafe { HSD_SISLIB_803A6368 } {
                    Some(f) => f(text, v),
                    None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A6368(text as _, v as _); } return; },
                }
            }
    hook!(HSD_SISLIB_803A62A0, HSD_SisLib_803A62A0, (a: i32, b: &str, c: &str));
    hook!(HSD_SISLIB_803A611C, HSD_SisLib_803A611C, (a: i32, g: *mut HSD_GObj, b: u16, c: u8, d: u8, e: u8, f: u8, h: u32) -> i32, crate::tu::adapt::HSD_SisLib_803A611C__de78e8);
    pub static mut HSD_SISLIB_803A5CC4: Option<fn(*mut HSD_Text)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_SisLib_803A5CC4(text: *mut HSD_Text) {
        match unsafe { HSD_SISLIB_803A5CC4 } {
            Some(f) => f(text),
            None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5CC4(text as _); } },
        }
    }
    // lb/lbsnap.c
    pub static mut LBSNAP_8001D40C: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbSnap_8001D40C(i: i32) -> i32 {
                match unsafe { LBSNAP_8001D40C } {
                    Some(f) => f(i),
                    None => { return unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D40C(i as _) as _ }; },
                }
            }
    pub static mut LBSNAP_8001D3B0: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbSnap_8001D3B0(i: i32) -> i32 {
                match unsafe { LBSNAP_8001D3B0 } {
                    Some(f) => f(i),
                    None => { return unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D3B0(i as _) as _ }; },
                }
            }
    pub static mut LBSNAP_8001D3CC: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbSnap_8001D3CC(i: i32) -> i32 {
                match unsafe { LBSNAP_8001D3CC } {
                    Some(f) => f(i),
                    None => { return unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D3CC(i as _) as _ }; },
                }
            }
    pub static mut LBSNAP_8001D338: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbSnap_8001D338(i: i32) -> i32 {
                match unsafe { LBSNAP_8001D338 } {
                    Some(f) => f(i),
                    None => { return unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D338(i as _) as _ }; },
                }
            }
    pub static mut LBSNAP_8001DC0C: Option<fn(*mut u8) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbSnap_8001DC0C(img: *mut u8) -> i32 {
                match unsafe { LBSNAP_8001DC0C } {
                    Some(f) => f(img),
                    None => { return unsafe { melee_runtime::tu::lbsnap::lbSnap_8001DC0C(img as _) as _ }; },
                }
            }
    pub static mut LBSNAP_8001DF20: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbSnap_8001DF20() -> i32 {
                match unsafe { LBSNAP_8001DF20 } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbsnap::lbSnap_8001DF20() as _ }; },
                }
            }
    pub static mut LBSNAP_8001DF6C: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbSnap_8001DF6C(chan: i32) -> i32 {
                match unsafe { LBSNAP_8001DF6C } {
                    Some(f) => f(chan),
                    None => { return unsafe { melee_runtime::tu::lbsnap::lbSnap_8001DF6C(chan as _) as _ }; },
                }
            }
    // gm/gm_16AE38.c (the VS scene controller), gm/gmscene.c (pause flags), gm/gm_1A0FEC.c
    pub static mut GMVS_GETSCENECONTROLLER: Option<fn() -> *mut VsSceneController> = None;
            #[allow(non_snake_case)]
            pub fn gmVs_GetSceneController() -> *mut VsSceneController {
                match unsafe { GMVS_GETSCENECONTROLLER } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmvs::gmVs_GetSceneController() as _ }; },
                }
            }
    pub static mut GM_GETDBPAUSEFLAG: Option<fn(i32) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetDbPauseFlag(bit: i32) -> bool {
                match unsafe { GM_GETDBPAUSEFLAG } {
                    Some(f) => f(bit),
                    None => { return unsafe { crate::tu::gmscene::gm_GetDbPauseFlag(bit as _) != 0 }; },
                }
            }
    pub static mut GM_CLEARDBPAUSEFLAG: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_ClearDbPauseFlag(bit: i32)  {
                match unsafe { GM_CLEARDBPAUSEFLAG } {
                    Some(f) => f(bit),
                    None => { unsafe { crate::tu::gmscene::gm_ClearDbPauseFlag(bit as _); } return; },
                }
            }
    pub static mut GM_SETDBPAUSEFLAG: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetDbPauseFlag(bit: i32)  {
                match unsafe { GM_SETDBPAUSEFLAG } {
                    Some(f) => f(bit),
                    None => { unsafe { crate::tu::gmscene::gm_SetDbPauseFlag(bit as _); } return; },
                }
            }
    pub static mut GM_801A0FEC: Option<fn(i32, u8) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801A0FEC(a: i32, b: u8)  {
                match unsafe { GM_801A0FEC } {
                    Some(f) => f(a, b),
                    None => { unsafe { crate::tu::gmpause::gm_801A0FEC(a as _, b as _); } return; },
                }
            }
    pub static mut GM_801A10FC: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801A10FC(a: i32)  {
                match unsafe { GM_801A10FC } {
                    Some(f) => f(a),
                    None => { unsafe { crate::tu::gmpause::gm_801A10FC(a as _); } return; },
                }
            }
    // lb/lbaudio_ax.c, if/ifall.c, sysdolphin/baselib/controller.c (rumble), cm/cmsnap.c
    pub static mut LBAUDIOAX_80024E84: Option<fn(bool) > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80024E84(v: bool)  {
                match unsafe { LBAUDIOAX_80024E84 } {
                    Some(f) => f(v),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024E84(v as _); } return; },
                }
            }
    pub static mut LBAUDIOAX_80024030: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80024030(id: i32)  {
                match unsafe { LBAUDIOAX_80024030 } {
                    Some(f) => f(id),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(id as _); } return; },
                }
            }
    pub static mut IFALL_802F33CC: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn ifAll_802F33CC()  {
                match unsafe { IFALL_802F33CC } {
                    Some(f) => f(),
                    None => { unsafe { melee_world::tu::ifall::ifAll_802F33CC(); } return; },
                }
            }
    pub static mut IFALL_802F3394: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn ifAll_802F3394()  {
                match unsafe { IFALL_802F3394 } {
                    Some(f) => f(),
                    None => { unsafe { melee_world::tu::ifall::ifAll_802F3394(); } return; },
                }
            }
    pub static mut HSD_PADRUMBLEUNPAUSEALL: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn HSD_PadRumbleUnpauseAll()  {
                match unsafe { HSD_PADRUMBLEUNPAUSEALL } {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::rumble::HSD_PadRumbleUnpauseAll(); } return; },
                }
            }
    pub static mut HSD_PADRUMBLEPAUSEALL: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn HSD_PadRumblePauseAll()  {
                match unsafe { HSD_PADRUMBLEPAUSEALL } {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::rumble::HSD_PadRumblePauseAll(); } return; },
                }
            }
    hook!(CMSNAP_800315C8, cmSnap_800315C8, ());
    hook!(CMSNAP_80031618, cmSnap_80031618, () -> *mut c_void);
    hook!(CMSNAP_800316B4, cmSnap_800316B4, ());
    // sysdolphin/baselib/jobj.c, anim, archive, gobj
    hook!(LB_80011E24, lb_80011E24, (src: *mut HSD_JObj, out: *mut *mut HSD_JObj, idx: i32, last: i32) -> i32);
    hook!(HSD_JOBJSETFLAGSALL, HSD_JObjSetFlagsAll, (j: *mut HSD_JObj, f: u32));
    hook!(HSD_JOBJCLEARFLAGSALL, HSD_JObjClearFlagsAll, (j: *mut HSD_JObj, f: u32));
    hook!(HSD_JOBJSETMTXDIRTYSUB, HSD_JObjSetMtxDirtySub, (j: *mut HSD_JObj));
    hook!(HSD_FOREACHANIM, HSD_ForeachAnim, (obj: *mut c_void, ty: u32, mask: u32, func: usize, arg_type: i32, a: i32, b: i32));
    pub static mut LBCARDNEW_COMPLETENEXTTASK: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbCardNew_CompleteNextTask() -> i32 {
                match unsafe { LBCARDNEW_COMPLETENEXTTASK } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbcardnew::lbCardNew_CompleteNextTask() as _ }; },
                }
            }
    hook!(LBARCHIVE_LOADARCHIVE, lbArchive_LoadArchive, (name: &str) -> *mut HSD_Archive);
    hook!(HSD_ARCHIVEGETPUBLICADDRESS, HSD_ArchiveGetPublicAddress, (a: *mut HSD_Archive, name: &str) -> *mut c_void);
    hook!(GOBJ_CREATE, GObj_Create, (classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj);
    pub static mut HSD_JOBJLOADJOINT: Option<fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn HSD_JObjLoadJoint(joint: *mut HSD_Joint) -> *mut HSD_JObj {
        match unsafe { HSD_JOBJLOADJOINT } {
            Some(f) => f(joint),
            None => { unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(joint as _) as _ } },
        }
    }
    hook!(HSD_GOBJOBJECT_80390A70, HSD_GObjObject_80390A70, (gobj: *mut HSD_GObj, kind: u8, obj: *mut HSD_JObj));
    hook!(GOBJ_SETUPGXLINK, GObj_SetupGXLink, (gobj: *mut HSD_GObj, render: usize, gx_link: u8, priority: u32));
    pub static mut GM_8016895C: Option<fn(*mut HSD_JObj, *mut DynamicModelDesc, i32)> = None;
    #[allow(non_snake_case)]
    pub fn gm_8016895C(jobj: *mut HSD_JObj, mdl: *mut DynamicModelDesc, idx: i32) {
        match unsafe { GM_8016895C } {
            Some(f) => f(jobj, mdl, idx),
            None => { unsafe { crate::tu::gm_1601::gm_8016895C(jobj as _, mdl as _, idx as _); } },
        }
    }
    hook!(HSD_JOBJREQANIMALL, HSD_JObjReqAnimAll, (jobj: *mut HSD_JObj, frame: f32));
    hook!(HSD_JOBJANIMALL, HSD_JObjAnimAll, (jobj: *mut HSD_JObj));
    hook!(HSD_GOBJ_SETUPPROC, HSD_GObj_SetupProc, (gobj: *mut HSD_GObj, proc: usize, prio: u8));
    /// `HSD_GObj_JObjKind` (sysdolphin/baselib/gobj.c).
    hook!(HSD_GOBJ_JOBJKIND, hsd_gobj_jobj_kind, () -> u8);
    /// `HSD_PadCopyStatus[idx].trigger` (sysdolphin/baselib/controller.c).
    hook!(HSD_PADCOPYSTATUS_TRIGGER, hsd_pad_copy_status_trigger, (idx: usize) -> u32);

    /// `HSD_GObj_JObjCallback` (sysdolphin/baselib/gobj.c): the GX link's render callback; address only.
    pub unsafe extern "C" fn HSD_GObj_JObjCallback(_gobj: *mut HSD_GObj, _code: i32) {
        { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(_gobj as _, _code as _); } return; };
    }
    /// `HSD_AObjStopAnim` (sysdolphin/baselib/aobj.c): passed as a callback address only.
    pub fn HSD_AObjStopAnim() {
        panic!("TODO(sysdolphin): HSD_AObjStopAnim is not ported");
    }
}

/// `HSD_JObjSetTranslateX(jobj, x)` (sysdolphin/baselib/jobj.h, static inline) and its dirty check.
#[inline(always)]
unsafe fn HSD_JObjSetTranslateX(jobj: *mut HSD_JObj, x: f32) {
    (*jobj).translate.x = x;
    if (*jobj).flags & (JOBJ_MTX_INDEP_SRT as u32) == 0 {
        // HSD_JObjSetMtxDirty(jobj): `if (jobj && !HSD_JObjMtxIsDirty(jobj)) HSD_JObjSetMtxDirtySub(jobj)`.
        let flags = (*jobj).flags;
        let is_dirty = (flags & (JOBJ_USER_DEF_MTX as u32)) == 0 && (flags & (JOBJ_MTX_DIRTY as u32)) != 0;
        if !jobj.is_null() && !is_dirty {
            ext::HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}

/// `sfxForward()` / `sfxBack()` / `sfxMove()` (mn/inlines.h, static inline): the menu sound effects.
#[inline(always)]
fn sfxForward() {
    ext::lbAudioAx_80024030(1);
}
#[inline(always)]
fn sfxBack() {
    ext::lbAudioAx_80024030(0);
}
#[inline(always)]
fn sfxMove() {
    ext::lbAudioAx_80024030(2);
}

/// `gmCamera_801A2224(arg0, arg1)`: writes the decimal digits of `arg1` (clamped to 9999) as the
/// (2-byte) sprite indices of the digit table, returns the position after the NUL terminator.
pub unsafe fn gmCamera_801A2224(mut arg0: *mut u8, mut arg1: u32) -> *mut u8 {
    melee_core::ct!("gmCamera_801A2224");
    let mut masked_arg1: u32;
    let mut cond_flag: u32 = 0;
    let slus2_arr_ptr: *const u8 = ext::HSD_SisLib_804D1124_arr();

    if arg1 >= 0x2710 {
        arg1 = 0x270F;
    }
    if arg1 >= 0x3E8 {
        masked_arg1 = (arg1 / 500) & 0x07FF_FFFE;
        *arg0 = *slus2_arr_ptr.add(masked_arg1 as usize);
        *arg0.add(1) = *slus2_arr_ptr.add(1 + masked_arg1 as usize);
        arg1 %= 0x3E8;
        cond_flag = 1;
        arg0 = arg0.add(2);
    }
    if arg1 >= 0x64 || cond_flag != 0 {
        masked_arg1 = (arg1 / 50) & 0x0FFF_FFFE;
        *arg0 = *slus2_arr_ptr.add(masked_arg1 as usize);
        *arg0.add(1) = *slus2_arr_ptr.add(1 + masked_arg1 as usize);
        arg1 %= 0x64;
        cond_flag += 1;
        arg0 = arg0.add(2);
    }
    if arg1 >= 0xA || cond_flag != 0 {
        masked_arg1 = (arg1 / 5) & 0x3FFF_FFFE;
        *arg0 = *slus2_arr_ptr.add(masked_arg1 as usize);
        *arg0.add(1) = *slus2_arr_ptr.add(1 + masked_arg1 as usize);
        arg1 %= 0xA;
        arg0 = arg0.add(2);
    }
    *arg0 = *slus2_arr_ptr.add(arg1 as usize * 2);
    arg0 = arg0.add(1);
    *arg0 = *slus2_arr_ptr.add(1 + arg1 as usize * 2);
    arg0 = arg0.add(1);
    *arg0 = 0;
    arg0
}

/// `gmCamera_801A2334(arg0, arg1..arg4)`: the score/label text of side `arg0`, placed at (arg1, arg2).
pub unsafe fn gmCamera_801A2334(arg0: i32, arg1: f32, arg2: f32, arg3: f32, arg4: f32) -> *mut HSD_Text {
    melee_core::ct!("gmCamera_801A2334");
    let gcu = &raw mut gmCamera_VsCamUiState;
    let text = ext::HSD_SisLib_803A5ACC(3, (*gcu).x54, arg1, arg2, 0.0, 640.0, 32.0);
    (*text).font_size.x = arg3;
    (*text).font_size.y = arg4;
    (*text).default_kerning = 1;
    let (group, digits) = if arg0 == 0 {
        (6, ext::HSD_SisLib_803A6530(3, 6, 8))
    } else {
        (7, ext::HSD_SisLib_803A6530(3, 7, 9))
    };
    let status = (*gcu).x24[arg0 as usize].x0 as u32;
    match status {
        0 => {
            if (*gcu).x24[arg0 as usize].x8 == 0 {
                (*gcu).x24[arg0 as usize].xC = 3;
                ext::HSD_SisLib_803A660C(3, group, 0xC);
            } else {
                let entries = (*gcu).x24.as_mut_ptr();
                let blocks_ptr: *mut i32 = &mut (*entries.add(arg0 as usize)).x4;
                let blocks = *blocks_ptr;
                if blocks == 0 {
                    (*gcu).x24[arg0 as usize].xC = 2;
                    ext::HSD_SisLib_803A660C(3, group, 0xC);
                } else {
                    let result = if blocks >= (*gcu).x20 {
                        if arg0 != 0 {
                            0
                        } else {
                            1
                        }
                    } else {
                        2
                    };
                    (*gcu).x24[arg0 as usize].xC = result;
                    gmCamera_801A2224(digits, *blocks_ptr as u32);
                    let label = if *blocks_ptr != 1 { 0xB } else { 0xA };
                    ext::HSD_SisLib_803A660C(3, group, label);
                }
            }
        }
        15 => {
            (*gcu).x24[arg0 as usize].xC = 5;
            ext::HSD_SisLib_803A660C(3, group, 0x10);
        }
        9 => {
            (*gcu).x24[arg0 as usize].xC = 4;
            ext::HSD_SisLib_803A660C(3, group, 0xD);
        }
        12 => {
            (*gcu).x24[arg0 as usize].xC = 4;
            ext::HSD_SisLib_803A660C(3, group, 0xE);
        }
        _ => {
            (*gcu).x24[arg0 as usize].xC = 4;
            ext::HSD_SisLib_803A660C(3, group, 0xF);
        }
    }
    ext::HSD_SisLib_803A6368(text, group);
    text
}

/// `gmCamera_801A253C(arg0, arg1)`: the header state (min of both sides) and the best count, when asked.
pub unsafe fn gmCamera_801A253C(arg0: *mut i32, arg1: *mut i32) {
    melee_core::ct!("gmCamera_801A253C");
    let gcu = &raw mut gmCamera_VsCamUiState;
    let result = (*gcu).x24[0].xC.min((*gcu).x24[1].xC);
    (*gcu).x44 = result;
    if !arg0.is_null() {
        *arg0 = (*gcu).x44;
    }
    if !arg1.is_null() {
        let mut var_r3: i32 = 0;
        let mut var_r6: i32 = 0;
        if (*gcu).x24[0].xC == 1 {
            var_r3 = (*gcu).x24[0].x4;
        }
        if (*gcu).x24[1].xC == 0 {
            var_r6 = (*gcu).x24[1].x4;
        }
        *arg1 = var_r3.max(var_r6);
    }
}

/// `gmCamera_801A25C8()`: read the snapshot slots of both sides.
pub unsafe fn gmCamera_801A25C8() {
    melee_core::ct!("gmCamera_801A25C8");
    let gcu = &raw mut gmCamera_VsCamUiState;
    for i in 0..2 {
        (*gcu).x24[i].x0 = ext::lbSnap_8001D40C(i as i32) != 0;
        if !(*gcu).x24[i].x0 {
            (*gcu).x24[i].x4 = ext::lbSnap_8001D3B0(i as i32);
            (*gcu).x24[i].x8 = ext::lbSnap_8001D3CC(i as i32);
        }
    }
}

/// `gmCamera_801A2640()`: the header text id.
pub unsafe fn gmCamera_801A2640() -> i32 {
    melee_core::ct!("gmCamera_801A2640");
    (*&raw const gmCamera_VsCamUiState).x54
}

/// `gmCamera_801A2650()`: the VS camera's interface (the "SdVsCam" text set) and header text.
pub unsafe fn gmCamera_801A2650() {
    melee_core::ct!("gmCamera_801A2650");
    let gcu = &raw mut gmCamera_VsCamUiState;
    (*gcu).x20 = 2;
    ext::HSD_SisLib_803A62A0(3, "SdVsCam", "SIS_VsCameraData");
    (*gcu).x54 = ext::HSD_SisLib_803A611C(3, core::ptr::null_mut(), 9, 13, 0, 14, 0, 11);
}

/// `static inline gmCamera_801A26C0_FreeTexts(unk)`: release the three texts (zeroing the slots).
unsafe fn gmCamera_801A26C0_FreeTexts(unk: *mut gmCameraUnkStruct) {
    if !(*unk).x48[0].is_null() {
        for i in 0..3 {
            ext::HSD_SisLib_803A5CC4((*unk).x48[i]);
            (*unk).x48[i] = core::ptr::null_mut();
        }
    }
}

/// `static inline gmCamera_FreeTextsWithZero(zero)`: as above, storing `zero` in the slots.
unsafe fn gmCamera_FreeTextsWithZero(zero: *mut HSD_Text) {
    let gcu = &raw mut gmCamera_VsCamUiState;
    if !(*gcu).x48[0].is_null() {
        for i in 0..3 {
            ext::HSD_SisLib_803A5CC4((*gcu).x48[i]);
            (*gcu).x48[i] = zero;
        }
    }
}

/// `gmCamera_801A26C0()`: leave the camera mode: unpause the scene and drop the texts.
pub unsafe fn gmCamera_801A26C0() {
    melee_core::ct!("gmCamera_801A26C0");
    let hud = ext::gmVs_GetSceneController();
    if ext::gm_GetDbPauseFlag(3) {
        ext::gm_ClearDbPauseFlag(3);
        if ext::gm_GetDbPauseFlag(1) {
            ext::gm_801A0FEC((*hud).state.pauser as i32, 1);
        } else {
            ext::lbAudioAx_80024E84(false);
            ext::ifAll_802F33CC();
            ext::HSD_PadRumbleUnpauseAll();
        }
        (*hud).state.hud_enabled = 1;
        (*hud).state.unk_3 = 0;
    }
    gmCamera_801A26C0_FreeTexts(&raw mut gmCamera_VsCamUiState);
}

/// `gmCamera_801A2798()`: B toggles the camera select; Z confirms into the snapshot mode.
pub unsafe fn gmCamera_801A2798() {
    melee_core::ct!("gmCamera_801A2798");
    let trig = ext::hsd_pad_copy_status_trigger(3);
    if trig & PAD_BUTTON_B != 0 {
        let new_var = (gmCamera_VsCamUiState.x14 as u32).wrapping_add(1);
        gmCamera_VsCamUiState.x14 = (new_var % 2) as i32;
        return;
    }
    if trig & PAD_TRIGGER_Z != 0 {
        ext::lbAudioAx_80024030(6);
        gmCamera_801A3048(1);
    }
}

/// `gmCamera_801A2800()`: enter the snapshot mode: show the camera model, pause the game.
pub unsafe fn gmCamera_801A2800() {
    melee_core::ct!("gmCamera_801A2800");
    let temp_r3 = ext::gmVs_GetSceneController();
    gmCamera_VsCamUiState.x10 = 0;
    let mut jobj: *mut HSD_JObj = core::ptr::null_mut();
    ext::lb_80011E24(gmCamera_VsCamUiState.x8, &mut jobj, 2, -1);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::gm_SetDbPauseFlag(3);
    if ext::gm_GetDbPauseFlag(1) {
        ext::gm_801A10FC((*temp_r3).state.pauser as i32);
    } else {
        ext::lbAudioAx_80024E84(true);
        ext::ifAll_802F3394();
        ext::HSD_PadRumblePauseAll();
    }
    (*temp_r3).state.hud_enabled = 0;
    (*temp_r3).state.unk_3 = 1;
    ext::cmSnap_800315C8();
}

/// `gmCamera_801A28AC()`: after a short wait, take the snapshot and check it.
pub unsafe fn gmCamera_801A28AC() {
    melee_core::ct!("gmCamera_801A28AC");
    gmCamera_VsCamUiState.x10 += 1;
    if gmCamera_VsCamUiState.x10 > 2 {
        let snap_buf = ext::cmSnap_80031618();
        gmCamera_VsCamUiState.snap_image = snap_buf;
        if !snap_buf.is_null() {
            if ext::lbSnap_8001DC0C(gmCamera_VsCamUiState.snap_image as *mut u8) != 0 {
                gmCamera_VsCamUiState.x20 = ext::lbSnap_8001DF20();
                gmCamera_801A3048(2);
            } else {
                gmCamera_801A3048(0);
            }
        }
    }
}

/// `gmCamera_801A292C()`: build the result texts for the two sides and the header.
pub unsafe fn gmCamera_801A292C() {
    melee_core::ct!("gmCamera_801A292C");
    let tbl = &gmCamera_803DA630;
    gmCamera_FreeTextsWithZero(core::ptr::null_mut());

    for i in 0..2 {
        gmCamera_VsCamUiState.x24[i].x0 = ext::lbSnap_8001D40C(i as i32) != 0;
        if !gmCamera_VsCamUiState.x24[i].x0 {
            gmCamera_VsCamUiState.x24[i].x4 = ext::lbSnap_8001D3B0(i as i32);
            gmCamera_VsCamUiState.x24[i].x8 = ext::lbSnap_8001D3CC(i as i32);
        }
    }

    let text = ext::HSD_SisLib_803A5ACC(3, gmCamera_VsCamUiState.x54, tbl[10], tbl[11], 0.0, 640.0, 32.0);
    let new_var = tbl[9];
    (*text).font_size.x = tbl[8];
    (*text).font_size.y = new_var;
    (*text).default_kerning = 1;
    gmCamera_801A2224(ext::HSD_SisLib_803A6530(3, 3, 4), gmCamera_VsCamUiState.x20 as u32);
    ext::HSD_SisLib_803A660C(3, 3, 5);
    ext::HSD_SisLib_803A6368(text, 3);
    gmCamera_VsCamUiState.x48[2] = text;
    gmCamera_VsCamUiState.x48[0] = gmCamera_801A2334(0, tbl[2], tbl[3], tbl[0], tbl[1]);
    gmCamera_VsCamUiState.x48[1] = gmCamera_801A2334(1, tbl[6], tbl[7], tbl[4], tbl[5]);
    gmCamera_VsCamUiState.x44 = gmCamera_VsCamUiState.x24[0].xC.min(gmCamera_VsCamUiState.x24[1].xC);
}

/// `gmCamera_801A2AAC()`: the result screen: A continues into the chosen mode, B backs out.
pub unsafe fn gmCamera_801A2AAC() {
    melee_core::ct!("gmCamera_801A2AAC");
    if ext::lbSnap_8001D338(0) != 0 || ext::lbSnap_8001D338(1) != 0 {
        gmCamera_801A3048(2);
    } else if ext::hsd_pad_copy_status_trigger(3) & PAD_BUTTON_A != 0 {
        sfxForward();
        match gmCamera_VsCamUiState.x44 {
            0 | 1 => {
                gmCamera_801A3048(6);
                return;
            }
            _ => {}
        }
        ext::lbAudioAx_80024030(3);
        let save_state = gmCamera_VsCamUiState.x44;
        match save_state {
            5 => {
                gmCamera_801A3048(3);
                return;
            }
            2 | 3 => {
                gmCamera_801A3048(4);
                return;
            }
            _ => {
                gmCamera_801A3048(5);
            }
        }
    } else if ext::hsd_pad_copy_status_trigger(3) & PAD_BUTTON_B != 0 {
        sfxBack();
        gmCamera_801A3048(0);
    }
}

/// `gmCamera_801A2BB0()`: B backs out to mode 0.
pub unsafe fn gmCamera_801A2BB0() {
    melee_core::ct!("gmCamera_801A2BB0");
    if ext::hsd_pad_copy_status_trigger(3) & PAD_BUTTON_B != 0 {
        sfxBack();
        gmCamera_801A3048(0);
    }
}

/// `gmCamera_801A2BF0_get_translate_x(px18)`: the cursor's x offset for the selected side.
pub unsafe fn gmCamera_801A2BF0_get_translate_x(px18: *const i32) -> f32 {
    melee_core::ct!("gmCamera_801A2BF0_get_translate_x");
    if *px18 != 0 {
        return 5.0;
    }
    -5.0
}

/// `static inline gmCamera_801A2BF0_get_jobj(px8)`.
#[inline(always)]
unsafe fn gmCamera_801A2BF0_get_jobj(px8: *mut *mut HSD_JObj) -> *mut HSD_JObj {
    *px8
}

/// `gmCamera_801A2BF0()`: mode 6 enter: animate the cursor and select side 0.
pub unsafe fn gmCamera_801A2BF0() {
    melee_core::ct!("gmCamera_801A2BF0");
    let gcus = &raw mut gmCamera_VsCamUiState;
    let px8: *mut *mut HSD_JObj = &mut (*gcus).x8;
    let mut jobj_a: *mut HSD_JObj = core::ptr::null_mut();
    let mut jobj_b: *mut HSD_JObj = core::ptr::null_mut();
    ext::lb_80011E24(gmCamera_801A2BF0_get_jobj(px8), &mut jobj_a, 9, -1);
    let var_f1 = if (*gcus).x44 == 1 { 1.0 } else { 2.0 };
    ext::HSD_JObjReqAnimAll(jobj_a, var_f1);
    ext::HSD_JObjAnimAll(jobj_a);
    ext::HSD_ForeachAnim(jobj_a as *mut c_void, JOBJ_TYPE as u32, TOBJ_MASK as u32, ext::HSD_AObjStopAnim as usize, AOBJ_ARG_AOV as i32, 0, 0);
    let px18: *mut i32 = &mut (*gcus).x18;
    *px18 = 0;
    ext::lb_80011E24(*px8, &mut jobj_b, 0xC, -1);
    let translate_x = gmCamera_801A2BF0_get_translate_x(px18);
    HSD_JObjSetTranslateX(jobj_b, translate_x);
}

/// `static inline gmCamera_801A2D44_update_selection(&jobj_b, &jobj)`: left / right moves the cursor.
unsafe fn gmCamera_801A2D44_update_selection(jobj_b: *mut *mut HSD_JObj, jobj: *mut *mut HSD_JObj) {
    let gcus = &raw mut gmCamera_VsCamUiState;
    let trig = ext::hsd_pad_copy_status_trigger(3);
    if trig & (PAD_BUTTON_LEFT | PAD_STICK_LEFT) != 0 {
        let px18: *mut i32 = &mut (*gcus).x18;
        if *px18 != 0 {
            sfxMove();
            *px18 = 0;
            ext::lb_80011E24((*gcus).x8, jobj, 0xC, -1);
            let translate_x = if *px18 != 0 { 5.0 } else { -5.0 };
            HSD_JObjSetTranslateX(*jobj, translate_x);
            return;
        }
    }
    if trig & (PAD_BUTTON_RIGHT | PAD_STICK_RIGHT) != 0 {
        let px18: *mut i32 = &mut (*gcus).x18;
        if *px18 != 1 {
            sfxMove();
            *px18 = 1;
            ext::lb_80011E24((*gcus).x8, jobj_b, 0xC, -1);
            let translate_x = if *px18 != 0 { 5.0 } else { -5.0 };
            HSD_JObjSetTranslateX(*jobj_b, translate_x);
        }
    }
}

/// `gmCamera_801A2D44()`: the camera select: confirm, back, or move the cursor.
pub unsafe fn gmCamera_801A2D44() {
    melee_core::ct!("gmCamera_801A2D44");
    let gcus = &raw mut gmCamera_VsCamUiState;
    let mut jobj: *mut HSD_JObj = core::ptr::null_mut();
    let mut jobj_b: *mut HSD_JObj = core::ptr::null_mut();
    if ext::lbSnap_8001D338(0) != 0 || ext::lbSnap_8001D338(1) != 0 {
        gmCamera_801A3048(2);
        return;
    }
    let trig = ext::hsd_pad_copy_status_trigger(3);
    if trig & (PAD_BUTTON_A | PAD_BUTTON_START) != 0 {
        if (*gcus).x18 == 0 {
            sfxForward();
            gmCamera_801A3048(7);
            return;
        }
        sfxBack();
        gmCamera_801A3048(0);
        return;
    }
    if trig & PAD_BUTTON_B != 0 {
        sfxBack();
        gmCamera_801A3048(0);
        return;
    }
    gmCamera_801A2D44_update_selection(&mut jobj_b, &mut jobj);
}

/// `gmCamera_801A2FBC()`: the snapshot's picture for the chosen side.
pub unsafe fn gmCamera_801A2FBC() {
    melee_core::ct!("gmCamera_801A2FBC");
    let chan = if gmCamera_VsCamUiState.x44 == 0 { 1 } else { 0 };
    ext::lbSnap_8001DF6C(chan);
}

/// `gmCamera_801A2FFC()`: wait for the card task; done → mode 0, a failure → mode 8.
pub unsafe fn gmCamera_801A2FFC() {
    melee_core::ct!("gmCamera_801A2FFC");
    let i = ext::lbCardNew_CompleteNextTask();
    if i != 0xB {
        if i == 0 {
            gmCamera_801A3048(0);
        } else {
            ext::lbAudioAx_80024030(3);
            gmCamera_801A3048(8);
        }
    }
}

/// `gmCamera_801A3048(i)`: switch the state machine to mode `i` and run its enter.
pub unsafe fn gmCamera_801A3048(i: i32) {
    melee_core::ct!("gmCamera_801A3048");
    gmCamera_VsCamUiState.xC = i;
    if let Some(f) = gmCamera_VsCamStateTable[gmCamera_VsCamUiState.xC as usize].x4 {
        f();
    }
}

/// `gmCamera_801A3098()`: run the current mode's update.
pub unsafe fn gmCamera_801A3098() {
    melee_core::ct!("gmCamera_801A3098");
    if let Some(f) = gmCamera_VsCamStateTable[gmCamera_VsCamUiState.xC as usize].x8 {
        f();
    }
}

/// `gmCamera_801A30E4()`: show or hide the camera UI's parts for the current mode.
pub unsafe fn gmCamera_801A30E4() {
    melee_core::ct!("gmCamera_801A30E4");
    let gcus = &raw mut gmCamera_VsCamUiState;
    let pxc = (*gcus).xC as usize;
    if gmCamera_VsCamStateTable[pxc].flags.x0 != 0 && (*gcus).x14 != 0 && !ext::gm_GetDbPauseFlag(1) {
        ext::HSD_JObjClearFlagsAll((*gcus).x4, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjSetFlagsAll((*gcus).x4, JOBJ_HIDDEN as u32);
    }
    let px8: *mut *mut HSD_JObj = &mut (*gcus).x8;
    ext::HSD_JObjSetFlagsAll((*gcus).x8, JOBJ_HIDDEN as u32);
    for i in 0..16 {
        if (<u32 as melee_core::ppc::Ppc>::pshl(1u32, (i) as i64)) & (gmCamera_VsCamStateTable[pxc].flags.x2 as u32) != 0 {
            let mut jobj: *mut HSD_JObj = core::ptr::null_mut();
            ext::lb_80011E24(*px8, &mut jobj, i, -1);
            ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    }
}

/// `fn_801A31D8(gobj)`: the camera model's per-frame joint update (the GX proc).
pub unsafe extern "C" fn fn_801A31D8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801A31D8");
    ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
}

/// `static inline gmCamera_801A31FC_inline(mdl)`: the camera model's object (its GX link and proc).
#[inline(always)]
unsafe fn gmCamera_801A31FC_inline(mdl: *mut DynamicModelDesc) {
    let gobj = ext::GObj_Create(0xE, 0x10, 0);
    let jobj = ext::HSD_JObjLoadJoint((*mdl).joint);
    gmCamera_VsCamUiState.x8 = jobj;
    ext::HSD_GObjObject_80390A70(gobj, ext::hsd_gobj_jobj_kind(), jobj as *mut HSD_JObj);
    ext::GObj_SetupGXLink(gobj, ext::HSD_GObj_JObjCallback as usize, 0xB, 0);
    ext::gm_8016895C(jobj, mdl, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    HSD_JObjSetFlagsAllInline(jobj, JOBJ_HIDDEN as u32);
    ext::HSD_GObj_SetupProc(gobj, fn_801A31D8 as usize, 0);
}

/// `HSD_JObjSetFlagsAll` as called from the TU (a sysdolphin function, hooked).
#[inline(always)]
unsafe fn HSD_JObjSetFlagsAllInline(j: *mut HSD_JObj, f: u32) {
    ext::HSD_JObjSetFlagsAll(j, f);
}

/// `gmCamera_801A31FC()`: enter the VS camera UI: the interface archive, the models, the header text.
pub unsafe fn gmCamera_801A31FC() {
    melee_core::ct!("gmCamera_801A31FC");
    ext::cmSnap_800316B4();
    gmCamera_VsCamUiState.x14 = 1;
    gmCamera_VsCamUiState.xC = 0;
    if let Some(f) = gmCamera_VsCamStateTable[gmCamera_VsCamUiState.xC as usize].x4 {
        f();
    }
    gmCamera_VsCamUiState.ifvscam = ext::lbArchive_LoadArchive("IfVsCam") as *mut c_void;
    {
        let joint_a = ext::HSD_ArchiveGetPublicAddress(gmCamera_VsCamUiState.ifvscam as *mut HSD_Archive, "IfCameraInfo_Top_model_set")
            as *mut *mut HSD_Joint;
        let gobj_a = ext::GObj_Create(0xE, 0x10, 0);
        let jobj_a = ext::HSD_JObjLoadJoint(*joint_a);
        gmCamera_VsCamUiState.x4 = jobj_a;
        ext::HSD_GObjObject_80390A70(gobj_a, ext::hsd_gobj_jobj_kind(), jobj_a as *mut HSD_JObj);
        ext::GObj_SetupGXLink(gobj_a, ext::HSD_GObj_JObjCallback as usize, 0xB, 0);
    }
    let mdl = ext::HSD_ArchiveGetPublicAddress(gmCamera_VsCamUiState.ifvscam as *mut HSD_Archive, "IfCamera_Top_model_set")
        as *mut DynamicModelDesc;
    gmCamera_801A31FC_inline(mdl);
    gmCamera_VsCamUiState.x20 = 2;
    ext::HSD_SisLib_803A62A0(3, "SdVsCam", "SIS_VsCameraData");
    gmCamera_VsCamUiState.x54 = ext::HSD_SisLib_803A611C(3, core::ptr::null_mut(), 9, 0xD, 0, 0xE, 0, 0xB);
    gmCamera_VsCamUiState.x48[0] = core::ptr::null_mut();
    gmCamera_VsCamUiState.x48[1] = core::ptr::null_mut();
    gmCamera_VsCamUiState.x48[2] = core::ptr::null_mut();
}

// ---- folded from gm_ptr.rs (consolidation steps 21/22) ----
/// gmcamera.c `gmCamera_801A2BF0_get_translate_x_fp`.
pub fn gmCamera_801A2BF0_get_translate_x_fp(px18: &i32) -> f32 {
    melee_core::ct!("gmCamera_801A2BF0_get_translate_x_fp");
    if *px18 != 0 { 5.0 } else { -5.0 }
}
