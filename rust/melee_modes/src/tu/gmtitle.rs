//! src/melee/gm/gmtitle.c: the title scene (animated logo and background, camera, light and fog
//! objects, archive load, OnFrame input handling, OnEnter). No float arithmetic, so no fused
//! sites (tools/rust-port/fma_sites.py: 0 fmuladd).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::{addr_of_mut, null_mut};

use melee_types::*;

use super::gmtoulib::hook::{self, Word};
use super::gmtoulib::PAD_BUTTON_START;

pub mod ext {
    use super::*;

    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_FogKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_LightKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use crate::tu::gmopening::gm_804D67EC;
    pub use crate::tu::gmopening::gm_804D67F0;
    pub static mut DbLevel: u32 = 0;
    pub static mut db_build_timestamp: [c_char; 64] = [0; 64];

    crate::gmtou_ext! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        fn HSD_JObjLoadJoint(arg0: *mut HSD_Joint) -> *mut HSD_JObj;
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void);
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32);
        fn GObj_SetupGXLinkMax(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, priority: u32);
        fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32);
        fn HSD_GObj_LObjCallback(gobj: *mut HSD_GObj, unused: i32);
        fn HSD_GObj_FogCallback(gobj: *mut HSD_GObj, unused: i32);
        fn HSD_GObj_80390ED0(gobj: *mut HSD_GObj, mask: u32);
        fn HSD_JObjAddAnimAll(jobj: *mut HSD_JObj, anim: *mut HSD_AnimJoint, matanim: *mut HSD_MatAnimJoint, shapeanim: *mut HSD_ShapeAnimJoint);
        fn HSD_JObjReqAnimAll(jobj: *mut HSD_JObj, frame: f32);
        fn HSD_JObjAnimAll(jobj: *mut HSD_JObj);
        fn HSD_JObjSetFlagsAll(jobj: *mut HSD_JObj, flags: u32);
        fn HSD_JObjSetMtxDirtySub(jobj: *mut HSD_JObj);
        fn lb_80011E24(jobj: *mut HSD_JObj, result: *mut *mut HSD_JObj, a: i32, b: i32);
        fn lb_80011AC4(arg0: *mut *mut LightList) -> *mut HSD_LObj;
        fn lb_80013B14(arg0: *mut HSD_CObjDesc) -> *mut HSD_CObj;
        fn mn_8022ED6C(jobj: *mut HSD_JObj, settings: *mut AnimLoopSettings);
        fn mn_8022F298(jobj: *mut HSD_JObj) -> f32;
        fn gm_GetCurrentGameMode() -> u8;
        fn gm_GetCurrentSceneIndex() -> u8;
        fn gm_IsCKindUnlocked(ckind: u8) -> i32;
        fn gm_801692E8(secs: u32, time: *mut datetime);
        fn lbTime_GetTimeInSeconds() -> u32;
        fn HSD_Rand() -> u32;
        fn gm_SetupTitleDemo();
        fn gm_PreloadTitleDemo();
        fn lbMthp_8001F614(arg0: i32);
        fn HSD_GObj_SetupProc(arg0: *mut HSD_GObj, arg1: HSD_GObjEvent, arg2: u8) -> *mut HSD_GObjProc;
        fn HSD_CObjSetCurrent(cobj: *mut HSD_CObj) -> i32;
        fn HSD_CObjEndCurrent();
        fn HSD_SetEraseColor(r: u8, g: u8, b: u8, a: u8);
        fn HSD_CObjEraseScreen(cobj: *mut HSD_CObj, a: i32, b: i32, c: i32);
        fn HSD_FogInterpretAnim(fog: *mut HSD_Fog);
        fn HSD_FogLoadDesc(desc: *mut HSD_FogDesc) -> *mut HSD_Fog;
        fn lbLang_IsSettingUS() -> i32;
        fn gm_GetButtonsTriggered(arg0: u8) -> u64;
        fn gm_GetCurrentSceneExitData() -> *mut c_void;
        fn gm_801A4B60();
        fn gmMainLib_8015ECBC();
        fn lbAudioAx_800236DC() -> i32;
        fn lbAudioAx_80024030(arg0: i32);
        fn lbAudioAx_80026F2C(arg0: u32);
        fn lbAudioAx_8002702C(flags: u32, mask: u64);
        fn lbAudioAx_80027168();
        fn lbAudioAx_80027648();
        fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32;
        fn HSD_SisLib_803A6754(arg0: i32, arg1: i32) -> *mut HSD_Text;
        fn HSD_SisLib_803A7548(arg0: *mut HSD_Text, arg1: i32, arg2: f32, arg3: f32);
    }
    /// `HSD_SisLib_803A6B98(text, x, y, fmt, ...)` with one pointer variadic (`"%s", buf`).
    pub unsafe extern "C" fn HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const c_char, va: *const c_char) -> i32 {
        <i32 as Word>::from_w(hook::call("HSD_SisLib_803A6B98", &[text.to_w(), x.to_w(), y.to_w(), fmt.to_w(), va.to_w()]))
    }
    /// `lbArchive_LoadSymbols(filename, symbols, ...)`: `va` are the remaining variadic words.
    pub unsafe extern "C" fn lbArchive_LoadSymbols(filename: *const c_char, symbols: *mut c_void, va: &[u64]) -> *mut HSD_Archive {
        let mut a = vec![filename.to_w(), symbols.to_w()];
        a.extend_from_slice(va);
        <*mut HSD_Archive as Word>::from_w(hook::call("lbArchive_LoadSymbols", &a))
    }
}

struct Dirty;
impl melee_types::inline::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !melee_types::inline::HSD_JObjMtxIsDirty(j) {
            unsafe { ext::HSD_JObjSetMtxDirtySub(j) };
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        unsafe { ext::HSD_JObjSetMtxDirtySub(j) };
    }
}

// ---------------------------------------------------------------------------------------------
// data
// ---------------------------------------------------------------------------------------------

pub static mut model_desc_0: StaticModelDesc = unsafe { core::mem::zeroed() };
pub static mut model_desc_1: StaticModelDesc = unsafe { core::mem::zeroed() };
pub static mut debug_text_buffer: [u8; 0x80] = [0; 0x80];
pub static mut loop_settings_0: AnimLoopSettings = AnimLoopSettings { start_frame: 0.0, end_frame: 1600.0, loop_frame: 400.0 };
pub static mut loop_settings_1: AnimLoopSettings = AnimLoopSettings { start_frame: 0.0, end_frame: 1330.0, loop_frame: 130.0 };
pub static mut jobj_translate: Vec3 = Vec3 { x: 0.0, y: -3.0, z: 0.0 };

pub static mut cobj_desc: *mut HSD_CObjDesc = null_mut();
pub static mut list_list: *mut *mut LightList = null_mut();
pub static mut fog_desc: *mut HSD_FogDesc = null_mut();
pub static mut countdown_timer: i32 = 0;
pub static mut frame_count: u32 = 0;
pub static mut bg_initialized: GXBool = 0;

const GX_FALSE: GXBool = 0;
const GX_TRUE: GXBool = 1;

#[inline]
unsafe fn sfxForward() {
    ext::lbAudioAx_80024030(1);
}

#[inline]
unsafe fn gobj_jobj(g: *mut HSD_GObj) -> *mut HSD_JObj {
    (*g).hsd_obj as *mut HSD_JObj
}

/// Hides the child found by `lb_80011E24(jobj, &result, idx, -1)`.
#[inline]
unsafe fn hide_lb_child(jobj: *mut HSD_JObj, idx: i32) {
    let mut result: *mut HSD_JObj = null_mut();
    ext::lb_80011E24(jobj, &mut result, idx, -1);
    ext::HSD_JObjSetFlagsAll(result, JOBJ_HIDDEN as u32);
}

// ---------------------------------------------------------------------------------------------
// functions
// ---------------------------------------------------------------------------------------------

pub unsafe extern "C" fn gmTitle_801A12C4() -> *mut HSD_GObj {
    melee_core::ct!("gmTitle_801A12C4");
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 15, 0);
    let jobj = ext::HSD_JObjLoadJoint(model_desc_0.joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 9, 0);
    ext::HSD_JObjAddAnimAll(jobj, model_desc_0.animjoint, model_desc_0.matanim_joint, model_desc_0.shapeanim_joint);
    ext::HSD_JObjReqAnimAll(jobj, loop_settings_0.loop_frame);
    ext::HSD_JObjAnimAll(jobj);

    melee_types::inline::HSD_JObjSetTranslate(&mut *jobj, &jobj_translate, &mut Dirty);
    hide_lb_child(jobj, 3);
    hide_lb_child(jobj, 1);
    gobj
}

/// Animation callback for background.
pub unsafe extern "C" fn gmTitle_801A146C(gobj: *mut HSD_GObj) {
    melee_core::ct!("gmTitle_801A146C");
    ext::mn_8022ED6C(gobj_jobj(gobj), addr_of_mut!(loop_settings_1));
}

#[inline]
unsafe fn isActiveTitle() -> bool {
    if ext::gm_GetCurrentGameMode() as i32 == GM_TITLE as i32
        || (ext::gm_GetCurrentGameMode() as i32 == GM_OPENING_MV as i32 && ext::gm_GetCurrentSceneIndex() as i32 == GS_VS as i32)
    {
        return false;
    }
    true
}

/// Set up title screen animated background.
pub unsafe extern "C" fn fn_801A1498_inline() {
    melee_core::ct!("fn_801A1498_inline");
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 15, 0);
    let jobj = ext::HSD_JObjLoadJoint(model_desc_1.joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 3, 0);
    ext::HSD_JObjAddAnimAll(jobj, model_desc_1.animjoint, model_desc_1.matanim_joint, model_desc_1.shapeanim_joint);
    ext::HSD_GObj_SetupProc(gobj, Some(gmTitle_801A146C), 0);
    if isActiveTitle() {
        ext::HSD_JObjReqAnimAll(jobj, loop_settings_1.start_frame);
    } else {
        ext::HSD_JObjReqAnimAll(jobj, 130.0);
    }
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe extern "C" fn fn_801A1498(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801A1498");
    let jobj = gobj_jobj(gobj);
    if ext::gm_804D67EC > 5400 {
        ext::mn_8022ED6C(jobj, addr_of_mut!(loop_settings_0));
    } else {
        ext::HSD_JObjReqAnimAll(jobj, ext::gm_804D67EC.wrapping_sub(5130) as f32);
        ext::HSD_JObjAnimAll(jobj);
    }
    if bg_initialized == GX_FALSE {
        if ext::mn_8022F298(jobj) >= 270.0 {
            fn_801A1498_inline();
            ext::lbMthp_8001F614(0);
            bg_initialized = GX_TRUE;
        }
    }
}

pub unsafe extern "C" fn gmTitle_801A1630(gobj: *mut HSD_GObj) {
    melee_core::ct!("gmTitle_801A1630");
    ext::mn_8022ED6C(gobj_jobj(gobj), addr_of_mut!(loop_settings_0));
}

#[inline]
unsafe fn isEmblemUnlocked() -> bool {
    if ext::gm_IsCKindUnlocked(CKind_Mars as u8) != 0 || ext::gm_IsCKindUnlocked(CKind_Emblem as u8) != 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn gmTitle_801A165C() -> *mut HSD_GObj {
    melee_core::ct!("gmTitle_801A165C");
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 15, 0);
    let jobj = ext::HSD_JObjLoadJoint(model_desc_0.joint);
    let kind = ext::HSD_GObj_JObjKind;

    ext::HSD_GObjObject_80390A70(gobj, kind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 9, 0);
    ext::HSD_JObjAddAnimAll(jobj, model_desc_0.animjoint, model_desc_0.matanim_joint, model_desc_0.shapeanim_joint);
    if isActiveTitle() {
        bg_initialized = GX_FALSE;
        ext::HSD_JObjReqAnimAll(jobj, loop_settings_0.start_frame);
        ext::HSD_GObj_SetupProc(gobj, Some(fn_801A1498), 0);
    } else {
        ext::HSD_JObjReqAnimAll(jobj, 400.0);
        ext::HSD_GObj_SetupProc(gobj, Some(gmTitle_801A1630), 0);
    }
    ext::HSD_JObjAnimAll(jobj);
    if !isEmblemUnlocked() {
        hide_lb_child(jobj, 7);
    }
    {
        let mut time: datetime = core::mem::zeroed();
        ext::gm_801692E8(ext::lbTime_GetTimeInSeconds(), &mut time);
        let mut second = time.second as i32;
        while second != 0 {
            ext::HSD_Rand();
            second -= 1;
        }
    }
    ext::gm_SetupTitleDemo();
    gobj
}

pub unsafe extern "C" fn gmTitle_801A1814(gobj: *mut HSD_GObj, _unused: i32) {
    melee_core::ct!("gmTitle_801A1814");
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj) != 0 {
        ext::HSD_GObj_80390ED0(gobj, 0x7);
        ext::HSD_CObjEndCurrent();
    }
}

pub unsafe extern "C" fn gmTitle_801A185C() {
    melee_core::ct!("gmTitle_801A185C");
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_CAMERA as u16, 20, 0);
    let cobj = ext::lb_80013B14(cobj_desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj, Some(gmTitle_801A1814), 0xC);
    (*gobj).gxlink_prios = 0x209;
}

pub unsafe extern "C" fn gmTitle_801A18D4(gobj: *mut HSD_GObj, _unused: i32) {
    melee_core::ct!("gmTitle_801A18D4");
    let erase_color: GXColor = (*fog_desc).color;
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj) != 0 {
        ext::HSD_SetEraseColor(erase_color.r, erase_color.g, erase_color.b, erase_color.a);
        ext::HSD_CObjEraseScreen((*gobj).hsd_obj as *mut HSD_CObj, 1, 0, 1);
        ext::HSD_CObjEndCurrent();
    }
}

pub unsafe extern "C" fn gmTitle_801A1944() {
    melee_core::ct!("gmTitle_801A1944");
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_CAMERA as u16, 20, 0);
    let cobj = ext::lb_80013B14(cobj_desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj, Some(gmTitle_801A18D4), 0);
}

pub unsafe extern "C" fn gmTitle_801A19AC() {
    melee_core::ct!("gmTitle_801A19AC");
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_LIGHT as u16, 3, 128);
    let lobj = ext::lb_80011AC4(list_list);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_LObjCallback), 0, 0);
}

pub unsafe extern "C" fn gmTitle_801A1A18(gobj: *mut HSD_GObj) {
    melee_core::ct!("gmTitle_801A1A18");
    ext::HSD_FogInterpretAnim((*gobj).hsd_obj as *mut HSD_Fog);
}

pub unsafe extern "C" fn gmTitle_801A1A3C() -> *mut HSD_GObj {
    melee_core::ct!("gmTitle_801A1A3C");
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_FOG as u16, 3, 0);
    let fog = ext::HSD_FogLoadDesc(fog_desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_FogCallback), 0, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(gmTitle_801A1A18), 0);
    gobj
}

pub unsafe extern "C" fn gmTitle_801A1AC0() -> *mut HSD_Archive {
    melee_core::ct!("gmTitle_801A1AC0");
    let dat = b"GmTtAll.dat\0";
    let usd = b"GmTtAll.usd\0";
    let p = |s: &'static [u8]| s.as_ptr() as usize as u64;

    let name = if ext::lbLang_IsSettingUS() != 0 { usd.as_ptr() } else { dat.as_ptr() };
    ext::lbArchive_LoadSymbols(
        name as *const c_char,
        addr_of_mut!(model_desc_0.joint) as *mut c_void,
        &[
            p(b"TtlMoji_Top_joint\0"),
            addr_of_mut!(model_desc_0.animjoint) as usize as u64,
            p(b"TtlMoji_Top_animjoint\0"),
            addr_of_mut!(model_desc_0.matanim_joint) as usize as u64,
            p(b"TtlMoji_Top_matanim_joint\0"),
            addr_of_mut!(model_desc_0.shapeanim_joint) as usize as u64,
            p(b"TtlMoji_Top_shapeanim_joint\0"),
            addr_of_mut!(cobj_desc) as usize as u64,
            p(b"ScTitle_cam_int1_camera\0"),
            addr_of_mut!(list_list) as usize as u64,
            p(b"ScTitle_scene_lights\0"),
            addr_of_mut!(fog_desc) as usize as u64,
            p(b"ScTitle_fog\0"),
            addr_of_mut!(model_desc_1.joint) as usize as u64,
            p(b"TtlBg_Top_joint\0"),
            addr_of_mut!(model_desc_1.animjoint) as usize as u64,
            p(b"TtlBg_Top_animjoint\0"),
            addr_of_mut!(model_desc_1.matanim_joint) as usize as u64,
            p(b"TtlBg_Top_matanim_joint\0"),
            addr_of_mut!(model_desc_1.shapeanim_joint) as usize as u64,
            p(b"TtlBg_Top_shapeanim_joint\0"),
            addr_of_mut!(ext::gm_804D67F0) as usize as u64,
            p(b"TitleMark_sobjdesc\0"),
            0,
        ],
    )
}

pub unsafe extern "C" fn gm_Scene_Title_OnFrame() {
    melee_core::ct!("gm_Scene_Title_OnFrame");
    let input = ext::gm_GetButtonsTriggered(4) as i32;
    if countdown_timer != 0 {
        countdown_timer -= 1;
        return;
    }
    frame_count = frame_count.wrapping_add(1);
    if frame_count > 600 {
        let tmp = ext::gm_GetCurrentSceneExitData() as *mut i32;
        *tmp = 0;
        ext::gm_801A4B60();
    } else if input & PAD_BUTTON_START as i32 != 0 {
        ext::lbAudioAx_80026F2C(0x1C);
        ext::lbAudioAx_8002702C(0xC, 0);
        ext::lbAudioAx_80027168();
        ext::lbAudioAx_80027648();
        sfxForward();
        ext::gmMainLib_8015ECBC();
        let tmp = ext::gm_GetCurrentSceneExitData() as *mut i32;
        *tmp = input;
        ext::gm_801A4B60();
    } else if input & (HSD_PAD_Y | HSD_PAD_B) != 0 {
        // TARGET_PC: Y or B opens the debug menu
        sfxForward();
        let tmp = ext::gm_GetCurrentSceneExitData() as *mut i32;
        *tmp = input;
        ext::gm_801A4B60();
    } else if ext::DbLevel >= DbLKind_DebugRom {
        if input & HSD_PAD_Y != 0 {
            sfxForward();
            let tmp = ext::gm_GetCurrentSceneExitData() as *mut i32;
            *tmp = input;
            ext::gm_801A4B60();
        } else if input & HSD_PAD_A != 0 {
            sfxForward();
            let tmp = ext::gm_GetCurrentSceneExitData() as *mut i32;
            *tmp = input;
            ext::gm_801A4B60();
        } else if input & HSD_PAD_X != 0 {
            sfxForward();
            let tmp = ext::gm_GetCurrentSceneExitData() as *mut i32;
            *tmp = input;
            ext::gm_801A4B60();
        }
    }
}

/// Converts ASCII to the Shift-JIS full-width run the SIS text renderer takes; returns the
/// position of the terminator.
pub unsafe extern "C" fn gmTitle_801A1D38(src: *const c_char, dst: *mut c_char) -> *mut c_char {
    melee_core::ct!("gmTitle_801A1D38");
    let mut src = src;
    let mut dst = dst;
    while *src != 0 {
        let c = *src as i8 as i32;
        if c == 0x20 {
            *dst = (-0x7F_i32) as c_char;
            *dst.add(1) = 0x40;
            dst = dst.add(2);
        } else if c >= 0x30 && c <= 0x39 {
            *dst = (-0x7E_i32) as c_char;
            *dst.add(1) = (c + 0x1F) as c_char;
            dst = dst.add(2);
        } else if c >= 0x41 && c <= 0x5A {
            *dst = (-0x7E_i32) as c_char;
            *dst.add(1) = (c + 0x1F) as c_char;
            dst = dst.add(2);
        } else if c >= 0x61 && c <= 0x7A {
            *dst = (-0x7E_i32) as c_char;
            *dst.add(1) = (c - 0xE0) as c_char;
            dst = dst.add(2);
        } else {
            *dst = (-0x7F_i32) as c_char;
            *dst.add(1) = 0x44;
            dst = dst.add(2);
        }
        src = src.add(1);
    }
    *dst = 0;
    dst
}

pub unsafe extern "C" fn gm_Scene_Title_OnEnter(_unused: *mut c_void) {
    melee_core::ct!("gm_Scene_Title_OnEnter");
    ext::lbAudioAx_800236DC();
    countdown_timer = 20;
    frame_count = 0;

    let _archive = gmTitle_801A1AC0();

    ext::lbAudioAx_80026F2C((1 << 1) | (1 << 4));
    ext::lbAudioAx_8002702C(2, 4);
    ext::lbAudioAx_80027168();

    gmTitle_801A1A3C();
    gmTitle_801A19AC();
    gmTitle_801A1944();
    gmTitle_801A185C();
    gmTitle_801A165C();

    ext::lbAudioAx_80027648();
    ext::gm_PreloadTitleDemo();

    fn_801A1498_inline();

    // Debug shows the build timestamp on the title screen
    if ext::DbLevel >= DbLKind_NoDebugRom {
        ext::HSD_SisLib_803A611C(0, null_mut(), 9, 0xD, 0, 0xE, 0, 0x13);
        let text = ext::HSD_SisLib_803A6754(0, 0);
        gmTitle_801A1D38(ext::db_build_timestamp.as_ptr(), debug_text_buffer.as_mut_ptr() as *mut c_char);
        let scale = ext::HSD_SisLib_803A6B98(text, 30.0, 30.0, b"%s\0".as_ptr() as *const c_char, debug_text_buffer.as_ptr() as *const c_char);
        (*text).default_kerning = 1;
        ext::HSD_SisLib_803A7548(text, scale, 0.7, 0.55);
    }
}
