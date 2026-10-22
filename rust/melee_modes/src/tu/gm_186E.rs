//! src/melee/gm/gm_186E.c: the All-Star intro scene (character reveal before the match).
//!
//! The file's statics are `pub static mut` so the state oracle (tests/tu_gm_186E.rs against
//! gen/turef/tu_gm_186E_ref.c, the C TU compiled natively with shipping contraction) can seed and
//! compare them.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use crate::tu_util::*;
use core::ptr::null_mut;
use melee_types::inline as mi;
use melee_types::*;

/// Callees outside this TU (hooks; unset they panic `TODO: <name> is not ported`).
pub mod ext {
    pub use crate::tu_ext::{
        efAsync_LoadSync, efLib_Init, ftDemo_ObjAllocInit, gm_8016895C, gm_801A4B60,
        lbArchive_80016DBC, lbArchive_80016EFC, lbAudioAx_800236DC, lbAudioAx_80023F28,
        lb_8000FCDC, lb_80011AC4, lb_80011E24, mpColl_80041C78, Camera_800313E0, Camera_Init,
        GObj_Create, GObj_SetupGXLink, GObj_SetupGXLinkMax, Ground_801C0378, HSD_CObjEndCurrent,
        HSD_CObjLoadDesc, HSD_CObjSetCurrent, HSD_FogLoadDesc, HSD_GObjObject_80390A70,
        HSD_GObj_80390ED0, HSD_GObj_CameraKind, HSD_GObj_FogCallback, HSD_GObj_FogKind,
        HSD_GObj_JObjCallback, HSD_GObj_JObjKind, HSD_GObj_LObjCallback, HSD_GObj_LightKind,
        HSD_GObj_SetupProc, HSD_JObjAnimAll, HSD_JObjLoadJoint, HSD_JObjReqAnimAll,
        Item_80266FA8, Item_80266FCC, Player_80036E20, Player_80036F34, Player_GetEntity,
        Player_GetEntityAtIndex, Player_InitAllPlayers, Player_SetCostumeId,
        Player_SetFacingDirection, Player_SetModelScale, Player_SetPlayerCharacter,
        Player_SetPlayerId, Player_SetSlottype, Stage_802251E8, Stage_8022524C,
    };
}

/// `static struct enterdata { int x0, x4; } lbl_804D6618` (the character/costume to introduce).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct enterdata {
    pub x0: i32,
    pub x4: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_804736B0_t {
    pub x0: i32,
    pub x4: i32,
    pub x8: *mut HSD_JObj,
    pub xC: *mut HSD_JObj,
}

pub static mut lbl_804D6610: *mut HSD_Archive = null_mut();
pub static mut lbl_804D6614: *mut SceneDesc = null_mut();
pub static mut lbl_804D6618: enterdata = enterdata { x0: 0, x4: 0 };
pub static mut lbl_804736B0: gm_804736B0_t =
    gm_804736B0_t { x0: 0, x4: 0, x8: null_mut(), xC: null_mut() };

pub unsafe extern "C" fn fn_80186EFC(gobj: *mut HSD_GObj) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let state = &mut lbl_804736B0;
    ext::HSD_JObjReqAnimAll(state.xC, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    let counter = &mut state.x4;
    if *counter < 0x8C {
        *counter += 1;
    } else {
        state.x0 = 1;
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_80186F6C_Entry {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub x14: f32,
}
const fn e(x0: f32, x4: f32, x8: f32, s: f32) -> gm_80186F6C_Entry {
    gm_80186F6C_Entry { x0, x4, x8, xC: s, x10: s, x14: s }
}
pub static lbl_803D9498: [gm_80186F6C_Entry; 28] = [
    e(0.0, -4.5, 0.0, 1.0),
    e(0.0, 1.5, 0.0, 0.6),
    e(0.0, -5.3, 0.0, 1.3),
    e(0.0, 1.0, 0.0, 1.0),
    e(0.0, 5.5, 0.0, 0.75),
    e(-1.1, -2.0, 0.0, 0.5),
    e(-1.0, -3.5, 0.0, 1.3),
    e(-1.0, -0.5, 0.0, 1.35),
    e(-1.0, -2.0, 0.0, 1.4),
    e(0.0, 0.0, 0.0, 1.0),
    e(0.0, -6.0, 0.0, 1.0),
    e(-1.0, -0.5, 0.0, 1.1),
    e(-2.0, -6.5, 0.0, 1.3),
    e(-0.5, 2.5, 0.0, 1.0),
    e(-0.8, 5.0, 0.0, 0.7),
    e(-1.0, 4.0, 0.0, 0.8),
    e(0.0, -5.0, 0.0, 1.0),
    e(0.0, 2.0, 0.0, 0.9),
    e(-1.0, -12.5, 0.0, 1.8),
    e(-3.5, 0.0, 0.0, 1.8),
    e(0.0, -1.0, 0.0, 1.0),
    e(0.0, 1.0, 0.0, 1.0),
    e(0.0, -2.5, 0.0, 1.1),
    e(0.0, 0.0, 0.0, 1.0),
    e(0.0, 5.5, 0.0, 0.4),
    e(1.0, -11.0, 0.0, 1.3),
    e(0.0, 0.0, 0.0, 1.0),
    e(-1.1, 2.5, 0.0, 0.4),
];

pub unsafe extern "C" fn fn_80186F6C(arg0: *mut HSD_GObj) {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    let child = lbl_804736B0.x8;
    let ent = &lbl_803D9498[lbl_804D6618.x0 as usize];

    mi::HSD_JObjGetTranslation(&*child, &mut pos);

    pos.x -= 7.0;
    pos.x += ent.x0;
    pos.y += ent.x4;
    pos.z += ent.x8;

    mi::HSD_JObjSetTranslate(&mut *jobj, &pos, &mut Dirty);

    mi::HSD_JObjSetScaleX(&mut *jobj, ent.xC, &mut Dirty);
    mi::HSD_JObjSetScaleY(&mut *jobj, ent.x10, &mut Dirty);
    mi::HSD_JObjSetScaleZ(&mut *jobj, ent.x14, &mut Dirty);

    {
        let entity2 = ext::Player_GetEntityAtIndex(0, 1);
        if !entity2.is_null() {
            let jobj2 = (*entity2).hsd_obj as *mut HSD_JObj;

            mi::HSD_JObjGetTranslation(&*jobj, &mut pos);
            pos.x += 3.0;
            pos.z -= 5.0;
            pos.y += 4.0;
            mi::HSD_JObjSetTranslate(&mut *jobj2, &pos, &mut Dirty);

            mi::HSD_JObjGetScale(&*jobj, &mut pos);
            mi::HSD_JObjSetScale(&mut *jobj2, &pos, &mut Dirty);
        }
    }
}

pub unsafe fn fn_801873F0() -> *mut HSD_GObjProc {
    ext::ftDemo_ObjAllocInit();
    ext::Player_InitAllPlayers();
    ext::Player_80036E20(lbl_804D6618.x0, lbl_804D6610, 1);
    ext::Player_SetPlayerCharacter(0, lbl_804D6618.x0);
    ext::Player_SetCostumeId(0, lbl_804D6618.x4);
    ext::Player_SetPlayerId(0, 0);
    ext::Player_SetSlottype(0, Gm_PKind_Demo as i32);
    ext::Player_SetFacingDirection(0, 0.0);
    ext::Player_SetModelScale(0, 1.0);
    ext::Player_80036F34(0, 6);
    ext::HSD_GObj_SetupProc(ext::Player_GetEntity(0), Some(fn_80186F6C), 0x16)
}

pub unsafe extern "C" fn fn_80187494(gobj: *mut HSD_GObj, _arg1: i32) {
    let temp_ret = ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj);
    if temp_ret != 0 {
        ext::Camera_800313E0(gobj, 1);
        (*gobj).gxlink_prios = 0x801;
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_CObjEndCurrent();
    }
}

pub unsafe fn fn_801874FC() {
    let mut sp8: *mut HSD_JObj = null_mut();

    let gobj = ext::GObj_Create(0xE, 0xF, 0);
    let models0 = *(*lbl_804D6614).models;
    let jobj = ext::HSD_JObjLoadJoint((*models0).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut core::ffi::c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_80186EFC), 0);
    ext::gm_8016895C(jobj, models0, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    ext::lb_80011E24(jobj, &mut sp8, 0x3A, -1);
    mi::HSD_JObjSetTranslateZ(&mut *sp8, 10000.0, &mut Dirty);
    ext::lb_80011E24(jobj, &mut sp8, 0x3B, -1);
    mi::HSD_JObjSetTranslateZ(&mut *sp8, 10000.0, &mut Dirty);
    ext::lb_80011E24(jobj, &mut lbl_804736B0.x8, 5, -1);
    ext::lb_80011E24(jobj, &mut lbl_804736B0.xC, 6, -1);
}

pub fn fn_80187714() {
    ext::Camera_Init(6);
    ext::lb_8000FCDC();
    ext::mpColl_80041C78();
    ext::Ground_801C0378(64);
    ext::Stage_802251E8(St_Kind_Dummy as i32, null_mut());
    ext::Stage_8022524C();
    ext::Item_80266FA8();
    ext::Item_80266FCC();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);
}

pub fn gm_Scene_IntroAllstar_OnFrame() {
    unsafe {
        if lbl_804736B0.x0 != 0 {
            ext::lbArchive_80016EFC(lbl_804D6610);
            ext::lbAudioAx_800236DC();
            ext::gm_801A4B60();
        }
    }
}

pub unsafe fn gm_Scene_IntroAllstar_OnEnter(arg0_: *mut core::ffi::c_void) {
    let arg0 = arg0_ as *mut enterdata;

    lbl_804736B0.x0 = 0;
    lbl_804736B0.x4 = 0;

    lbl_804D6618 = *arg0;
    fn_80187714();
    lbl_804D6610 = ext::lbArchive_80016DBC(
        c"IrAls".as_ptr(),
        &[
            &raw mut lbl_804D6614 as *mut core::ffi::c_void,
            c"ScItrAllstar_scene_data".as_ptr() as *mut core::ffi::c_void,
        ],
    );

    let temp_r30 = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::HSD_CObjLoadDesc((*(*lbl_804D6614).cameras).desc);
    ext::HSD_GObjObject_80390A70(temp_r30, ext::HSD_GObj_CameraKind, cobj as *mut core::ffi::c_void);
    ext::GObj_SetupGXLinkMax(temp_r30, Some(fn_80187494), 8);
    (*temp_r30).gxlink_prios = 0x801;

    let temp_r30_2 = ext::GObj_Create(0xB, 3, 0);
    let lobj = ext::lb_80011AC4((*lbl_804D6614).lights);
    ext::HSD_GObjObject_80390A70(temp_r30_2, ext::HSD_GObj_LightKind as u8, lobj as *mut core::ffi::c_void);
    ext::GObj_SetupGXLink(temp_r30_2, Some(ext::HSD_GObj_LObjCallback), 0, 0);
    fn_801874FC();

    let temp_r30_3 = ext::GObj_Create(0xE, 0xF, 0);
    let fog = ext::HSD_FogLoadDesc((*(*lbl_804D6614).fogs).desc);
    ext::HSD_GObjObject_80390A70(temp_r30_3, ext::HSD_GObj_FogKind as u8, fog as *mut core::ffi::c_void);
    ext::GObj_SetupGXLink(temp_r30_3, Some(ext::HSD_GObj_FogCallback), 0xB, 0);
    fn_801873F0();
    ext::lbAudioAx_80023F28(0x2D);
}
