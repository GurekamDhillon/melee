//! src/melee/gm/gm_1879.c: the Regular-Mode intro scene (stage fly-through, "Press Start" prompt).
//!
//! The file's statics are `pub static mut` so the state oracle (tests/tu_gm_1879.rs against
//! gen/turef/tu_gm_1879_ref.c, the C TU compiled natively with shipping contraction) can seed and
//! compare them. The C bitfields are PPC (MSB-first) ones: `x36` is `stage_index:5, done:1,
//! flash:1, active:1`, `x37` is `frame_counter:4, anim_state:2, state2:2`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gm_r9::{gm_80187F48_EnterData as gm_80187F48_EnterData_gm_r9};
use crate::tu_util::*;
use core::ffi::{c_char, c_void, CStr};
use core::ptr::null_mut;
use melee_types::*;

/// Callees outside this TU (hooks; unset they panic `TODO: <name> is not ported`).
pub mod ext {
    pub use crate::tu_ext::{
        efAsync_LoadSync, efLib_Init, gm_801A4B60, gm_GetButtonsTriggered, gm_GetCurrentGameMode,
        lbArchive_80016DBC, lbArchive_80016EFC, lbAudioAx_80023F28, lbAudioAx_80024030,
        lbAudioAx_80026E84, lbAudioAx_80026F2C, lbAudioAx_8002702C, lbAudioAx_80027168,
        lbAudioAx_80027648, lbBgFlash_800209F4, lbBgFlash_8002063C, lb_80011AC4, lb_80011C18,
        lb_8000B09C, lb_8000C0E8, lb_8000FCDC, mpColl_80041C78, Camera_800304E0, GObj_Create,
        GObj_SetupGXLink, GObj_SetupGXLinkMax, GXSetZMode, Ground_801C0378, Ground_801C0498,
        HSD_CObjAddAnim, HSD_CObjAnim, HSD_CObjGetEyePosition, HSD_CObjGetInterest,
        HSD_CObjLoadDesc, HSD_CObjRemoveAnim, HSD_CObjReqAnim, HSD_CObjSetEyePosition,
        HSD_CObjSetInterest, HSD_GObjFree, HSD_GObjObject_80390A70, HSD_GObj_803910D8,
        HSD_GObj_CameraKind, HSD_GObj_JObjCallback, HSD_GObj_JObjKind, HSD_GObj_LObjCallback,
        HSD_GObj_LightKind, HSD_GObj_SetupProc, HSD_JObjAnimAll, HSD_JObjLoadJoint,
        HSD_JObjReqAnimAll, HSD_StateInvalidate, Item_80266F70, Item_80266FCC,
        Player_GetPlayerCharacter, Player_InitAllPlayers, Stage_802251E8, Stage_8022524C,
    };
}

/// `struct gm_1832_804736C0_t` (0x40 on the GC): the scene state shared by the intro callbacks.
/// `x36` / `x37` hold PPC bitfield runs (see the module doc; accessors below).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_1832_804736C0_t {
    pub x0: *mut SceneDesc,
    pub x4: *mut SceneCameraDesc,
    pub x8: *mut HSD_GObj,
    pub pad_C: [u8; 0x28],
    pub x34: u16,
    pub x36: u8,
    pub x37: u8,
    pub x38: u8,
    pub pad_39: [u8; 7],
}
melee_core::gc_layout!(gm_1832_804736C0_t, "gm_1832_804736C0_t", size 0x40, align 4, [
    x0 @ 0x0: 4 "SceneDesc*",
    x4 @ 0x4: 4 "SceneCameraDesc*",
    x8 @ 0x8: 4 "HSD_GObj*",
    pad_C @ 0xC: 40 "u8[40]",
    x34 @ 0x34: 2 "u16",
    stage_index @ 0x36: 1 "u8" bits 0 + 5,
    done @ 0x36: 1 "u8" bits 5 + 1,
    flash @ 0x36: 1 "u8" bits 6 + 1,
    active @ 0x36: 1 "u8" bits 7 + 1,
    frame_counter @ 0x37: 1 "u8" bits 0 + 4,
    anim_state @ 0x37: 1 "u8" bits 4 + 2,
    state2 @ 0x37: 1 "u8" bits 6 + 2,
    x38 @ 0x38: 1 "u8",
    pad_39 @ 0x39: 7 "u8[7]",
]);
impl gm_1832_804736C0_t {
    pub fn stage_index(&self) -> u8 { bits8_get(self.x36, 0, 5) }
    pub fn done(&self) -> u8 { bits8_get(self.x36, 5, 1) }
    pub fn flash(&self) -> u8 { bits8_get(self.x36, 6, 1) }
    pub fn active(&self) -> u8 { bits8_get(self.x36, 7, 1) }
    pub fn set_stage_index(&mut self, v: u8) { bits8_set(&mut self.x36, 0, 5, v) }
    pub fn set_done(&mut self, v: u8) { bits8_set(&mut self.x36, 5, 1, v) }
    pub fn set_flash(&mut self, v: u8) { bits8_set(&mut self.x36, 6, 1, v) }
    pub fn set_active(&mut self, v: u8) { bits8_set(&mut self.x36, 7, 1, v) }
    pub fn frame_counter(&self) -> u8 { bits8_get(self.x37, 0, 4) }
    pub fn anim_state(&self) -> u8 { bits8_get(self.x37, 4, 2) }
    pub fn state2(&self) -> u8 { bits8_get(self.x37, 6, 2) }
    pub fn set_frame_counter(&mut self, v: u8) { bits8_set(&mut self.x37, 0, 4, v) }
    pub fn set_anim_state(&mut self, v: u8) { bits8_set(&mut self.x37, 4, 2, v) }
    pub fn set_state2(&mut self, v: u8) { bits8_set(&mut self.x37, 6, 2, v) }
}

pub static mut lbl_804736C0: gm_1832_804736C0_t = unsafe { core::mem::zeroed() };
pub static mut lbl_804D6620: *mut HSD_Archive = null_mut();

#[inline]
unsafe fn model(data: &gm_1832_804736C0_t, i: usize) -> *mut DynamicModelDesc {
    *(*data.x0).models.add(i)
}

/// `fn_80187AB4_LoadAnim` (static inline in the C).
unsafe fn fn_80187AB4_LoadAnim(jobj: *mut HSD_JObj, data: &mut gm_1832_804736C0_t) {
    let desc = model(data, 11 - data.stage_index() as usize);
    if !(*desc).anims.is_null() {
        let anim_state = data.anim_state() as i32;
        if !(*(*desc).anims.add(anim_state as usize)).is_null() {
            ext::lb_8000C0E8(jobj, anim_state, desc);
            ext::HSD_JObjReqAnimAll(jobj, 0.0);
            ext::HSD_JObjAnimAll(jobj);
        }
    }
}

pub unsafe extern "C" fn fn_80187910(arg0: *mut HSD_GObj) {
    melee_core::ct!("fn_80187910");
    let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let data = &mut lbl_804736C0;
    let cobj = (*arg0).hsd_obj as *mut HSD_CObj;
    if ext::gm_GetButtonsTriggered(data.x38) & PAD_BUTTON_A as u64 != 0 {
        // `data->x37.frame_counter = (s32) curr_frame / 300;` truncates to the 4-bit field
        let q = (melee_core::ppc::fctiwz_f32((*(*(*cobj).eyepos).aobj).curr_frame)) / 300;
        data.set_frame_counter(q as u8);
        // `++frame_counter >= 8U`: the incremented value is re-read from the 4-bit field
        data.set_frame_counter(data.frame_counter().wrapping_add(1));
        if data.frame_counter() as u32 >= 8 {
            data.set_frame_counter(0);
        }
        let frame = data.frame_counter() as i32 * 0x12C;
        ext::HSD_CObjRemoveAnim(cobj);
        ext::HSD_CObjAddAnim(cobj, *(*data.x4).anims);
        ext::HSD_CObjReqAnim(cobj, frame as f32);
    }
    ext::HSD_CObjAnim(cobj);
    ext::HSD_CObjGetInterest(cobj, &mut sp10);
    let mut scale = ext::Ground_801C0498();
    sp10.x *= scale;
    sp10.y *= scale;
    sp10.z *= scale;
    ext::HSD_CObjSetInterest(cobj, &mut sp10);
    ext::HSD_CObjGetEyePosition(cobj, &mut sp10);
    scale = ext::Ground_801C0498();
    sp10.x *= scale;
    sp10.y *= scale;
    sp10.z *= scale;
    ext::HSD_CObjSetEyePosition(cobj, &mut sp10);
}

pub unsafe extern "C" fn fn_80187AB4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80187AB4");
    let data = &mut lbl_804736C0;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let state = data.anim_state() as i32;

    match state {
        0 => {
            if ext::lb_8000B09C(jobj) == 0 {
                data.set_anim_state(1);
                fn_80187AB4_LoadAnim(jobj, data);
                data.x34 = 0x960;
            }
        }
        1 => {
            if data.active() as u32 == 1 {
                data.set_flash(1);
            }
            if data.flash() != 0 {
                data.set_anim_state(2);
                data.set_done(1);
                ext::lbBgFlash_8002063C(1);
            }
            if ext::lb_8000B09C(jobj) == 0 {
                fn_80187AB4_LoadAnim(jobj, data);
            }
        }
        2 => {
            if ext::lb_8000B09C(jobj) == 0 {
                data.set_done(1);
                ext::HSD_GObjFree(gobj);
            }
        }
        _ => {}
    }
    ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
}

pub unsafe extern "C" fn fn_80187C9C(gobj: *mut HSD_GObj, arg1: isize) {
    melee_core::ct!("fn_80187C9C");
    ext::GXSetZMode(0, GX_NEVER as i32, 0);
    ext::HSD_GObj_JObjCallback(gobj, arg1);
    ext::HSD_StateInvalidate(0x40);
}

/// Request the animation `anim_state` of model 12 (the "Press Start" prompt) when it exists.
unsafe fn prompt_anim(data: &gm_1832_804736C0_t, jobj: *mut HSD_JObj, anim_state: i32) {
    let desc = model(data, 12);
    if !(*(*desc).anims.add(anim_state as usize)).is_null() {
        ext::lb_8000C0E8(jobj, anim_state, desc);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
    }
}

pub unsafe extern "C" fn fn_80187CF4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80187CF4");
    let data = &mut lbl_804736C0;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let state = data.state2() as i32;

    match state {
        0 => {
            if ext::lb_8000B09C(jobj) == 0 {
                data.set_state2(1);
                prompt_anim(data, jobj, data.state2() as i32);
            }
        }
        1 => {
            if ext::lb_8000B09C(jobj) == 0 {
                data.set_state2(2);
                prompt_anim(data, jobj, data.state2() as i32);
            }
        }
        2 => {
            if ext::gm_GetButtonsTriggered(data.x38) & PAD_BUTTON_START as u64 != 0 {
                ext::lbAudioAx_80024030(1); // sfxForward()
                data.set_active(1);
                data.set_state2(3);
                prompt_anim(data, jobj, data.state2() as i32);
            } else if ext::lb_8000B09C(jobj) == 0 {
                prompt_anim(data, jobj, data.state2() as i32);
            }
        }
        3 => {
            if ext::lb_8000B09C(jobj) == 0 {
                ext::HSD_GObjFree(gobj);
            }
        }
        _ => {}
    }
    ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
}

pub static gm_1832_NormalArchiveNames: [&CStr; 12] = [
    c"mc01", c"mc02", c"mc03", c"mc04", c"mc05", c"mc06", c"mc07", c"mc08", c"mc09", c"mc10",
    c"mc11", c"mc12",
];

pub static gm_1832_NormalBgmIds: [i32; 12] = [0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x14, 0x14, 0x1C];

pub static gm_1832_NormalAudioMasks: [u64; 12] = [
    0x0002000010000000, 0x0000002000000100, 0x0002000000010000, 0x0000010004000000,
    0x0000080200004000, 0x0000000800000C00, 0x0001000001000000, 0x0002000000000040,
    0x0000008000200000, 0x0000000000002000, 0x0000000000060000, 0x0020000000009000,
];

pub static gm_1832_NormalSceneDataName: &CStr = c"ScItrNormal_scene_data";

/// `gm_80187F48_EnterData` (4 bytes on the GC).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_80187F48_EnterData {
    pub x0: u8,
    pub stage_index: u8,
    pub stkind: u16,
}
melee_core::gc_layout!(gm_80187F48_EnterData, "gm_80187F48_EnterData", size 0x4, align 2, [
    x0 @ 0x0: 1 "u8",
    stage_index @ 0x1: 1 "u8",
    stkind @ 0x2: 2 "u16",
]);

#[inline]
unsafe fn gm_80187F48_GetAudioConfig(stage_index: u8) -> u64 {
    ext::lbAudioAx_80026E84(ext::Player_GetPlayerCharacter(0)) | gm_1832_NormalAudioMasks[stage_index as usize]
}

/// `gm_Scene_IntroNormal_OnEnter` (the C inlines `gm_80187F48_OnEnter_inline` into it).
pub unsafe fn gm_Scene_IntroNormal_OnEnter(arg0: *mut c_void) {
    melee_core::ct!("gm_Scene_IntroNormal_OnEnter");
    let arg0 = &*(arg0 as *const gm_80187F48_EnterData);
    let data = &mut lbl_804736C0;
    data.x38 = arg0.x0;
    ext::lb_8000FCDC();
    ext::mpColl_80041C78();
    ext::Ground_801C0378(0x40);
    ext::Stage_802251E8(arg0.stkind as i32, null_mut());
    ext::Item_80266F70();
    ext::Item_80266FCC();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);
    ext::efAsync_LoadSync(0x1F);
    ext::Stage_8022524C();
    ext::Player_InitAllPlayers();
    ext::lbBgFlash_800209F4();

    let stage_index = arg0.stage_index;
    data.set_anim_state(0);
    data.set_frame_counter(0);
    data.x34 = 0;
    data.set_done(0);
    data.set_flash(0);
    data.set_stage_index(stage_index);
    data.set_state2(0);
    data.set_active(0);

    {
        let si = arg0.stage_index as usize;
        lbl_804D6620 = ext::lbArchive_80016DBC(
            c"IrNml".as_ptr(),
            &[
                &raw mut data.x0 as *mut c_void,
                gm_1832_NormalSceneDataName.as_ptr() as *mut c_void,
                &raw mut data.x4 as *mut c_void,
                gm_1832_NormalArchiveNames[si].as_ptr() as *mut c_void,
            ],
        );

        ext::lbAudioAx_80026F2C(gm_1832_NormalBgmIds[si] as u32);
        ext::lbAudioAx_8002702C(gm_1832_NormalBgmIds[si] as u32, gm_80187F48_GetAudioConfig(si as u8));
    }
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();

    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    data.x8 = gobj;
    let cobj = ext::HSD_CObjLoadDesc((*data.x4).desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    // `(GObj_RenderFunc) (Event) Camera_800304E0`: the C calls it with a surplus argument
    let render: unsafe extern "C" fn(*mut HSD_GObj, isize) = core::mem::transmute(ext::Camera_800304E0 as unsafe extern "C" fn(*mut HSD_GObj));
    ext::GObj_SetupGXLinkMax(gobj, Some(render), 8);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_80187910), 0);
    ext::HSD_CObjAddAnim(cobj, *(*data.x4).anims);
    ext::HSD_CObjReqAnim(cobj, 0.0);

    {
        let scene_gobj = ext::GObj_Create(0x13, 0x14, 0);
        let scene_cobj = ext::HSD_CObjLoadDesc((*(*data.x0).cameras).desc);
        ext::HSD_GObjObject_80390A70(scene_gobj, ext::HSD_GObj_CameraKind, scene_cobj as *mut c_void);
        ext::GObj_SetupGXLinkMax(scene_gobj, Some(ext::HSD_GObj_803910D8), 8);
        (*scene_gobj).gxlink_prios = 0xC00;
    }

    {
        let light_gobj = ext::GObj_Create(0xB, 3, 0);
        let lobj = ext::lb_80011AC4((*data.x0).lights);
        ext::HSD_GObjObject_80390A70(light_gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
        ext::GObj_SetupGXLink(light_gobj, Some(ext::HSD_GObj_LObjCallback), 0xA, 0);
    }

    {
        let model_gobj = ext::GObj_Create(0xE, 0xF, 0);
        let model_jobj = ext::HSD_JObjLoadJoint((*model(data, 11 - data.stage_index() as usize)).joint);
        ext::lb_80011C18(model_jobj, 0x0800_0000);
        ext::HSD_GObjObject_80390A70(model_gobj, ext::HSD_GObj_JObjKind, model_jobj as *mut c_void);
        ext::GObj_SetupGXLink(model_gobj, Some(fn_80187C9C), 0xB, 0xB);

        let model_stage = data.stage_index() as usize;
        let model_desc = model(data, 11 - model_stage);
        let anims = (*model_desc).anims;
        if !anims.is_null() {
            let model_anim_idx = data.anim_state() as i32;
            if !(*anims.add(model_anim_idx as usize)).is_null() {
                ext::lb_8000C0E8(model_jobj, model_anim_idx, model_desc);
                ext::HSD_JObjReqAnimAll(model_jobj, 0.0);
                ext::HSD_JObjAnimAll(model_jobj);
            }
        }
        ext::HSD_GObj_SetupProc(model_gobj, Some(fn_80187AB4), 0);
    }

    {
        let model_gobj = ext::GObj_Create(0xE, 0xF, 0);
        let model_jobj = ext::HSD_JObjLoadJoint((*model(data, 12)).joint);
        ext::lb_80011C18(model_jobj, 0x0800_0000);
        ext::HSD_GObjObject_80390A70(model_gobj, ext::HSD_GObj_JObjKind, model_jobj as *mut c_void);
        ext::GObj_SetupGXLink(model_gobj, Some(fn_80187C9C), 0xB, 0xB);

        let model_anim_idx = data.state2() as i32;
        let model_desc = model(data, 12);
        if !(*(*model_desc).anims.add(model_anim_idx as usize)).is_null() {
            ext::lb_8000C0E8(model_jobj, model_anim_idx, model_desc);
            ext::HSD_JObjReqAnimAll(model_jobj, 0.0);
            ext::HSD_JObjAnimAll(model_jobj);
        }

        ext::HSD_JObjReqAnimAll(model_jobj, 0.0);
        ext::HSD_JObjAnimAll(model_jobj);
        ext::HSD_GObj_SetupProc(model_gobj, Some(fn_80187CF4), 0);
    }
    ext::lbAudioAx_80023F28(0x2E);
}

pub unsafe fn gm_Scene_IntroNormal_OnLeave(_arg0: *mut c_void) {
    melee_core::ct!("gm_Scene_IntroNormal_OnLeave");
    ext::lbArchive_80016EFC(lbl_804D6620);
}

pub fn gm_Scene_IntroNormal_OnFrame() {
    melee_core::ct!("gm_Scene_IntroNormal_OnFrame");
    unsafe {
        if lbl_804736C0.done() != 0 {
            ext::gm_801A4B60();
        }
    }
}

pub unsafe fn gm_801883C0() -> f32 {
    melee_core::ct!("gm_801883C0");
    let gobj = lbl_804736C0.x8;
    let cobj: *mut HSD_CObj;
    if !gobj.is_null() {
        cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    } else {
        return 0.0;
    }

    let mut aobj = (*(*cobj).eyepos).aobj;
    if !aobj.is_null() {
        return (*aobj).curr_frame;
    }

    aobj = (*(*cobj).interest).aobj;
    if !aobj.is_null() {
        return (*aobj).curr_frame;
    }

    0.0
}

pub fn gm_8018841C() -> bool {
    melee_core::ct!("gm_8018841C");
    ext::gm_GetCurrentGameMode() as i32 == GM_TRAINING as i32
}

// ---- folded from gm_r9.rs (consolidation steps 21/22) ----
/// gm_1879.c `getStKind`.
pub fn getStKind(data: &gm_80187F48_EnterData_gm_r9) -> i32 {
    melee_core::ct!("getStKind");
    data.stkind as i32
}
