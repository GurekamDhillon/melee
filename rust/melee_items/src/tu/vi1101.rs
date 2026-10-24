//! Port of src/melee/vi/vi1101.c: the "Vi1101" character-demo scene (Mario/Luigi trophy-style intro).
//! Callees of unported TUs are TODO stubs in `ext`. `vi_RunCamera` (vi.h static inline) is
//! reproduced locally.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

#[inline]
unsafe fn GET_JOBJ(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}
#[inline]
unsafe fn GET_COBJ(gobj: *mut HSD_GObj) -> *mut HSD_CObj {
    (*gobj).hsd_obj as *mut HSD_CObj
}

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code, unused_variables)]
    use super::*;
    macro_rules! todo_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            $( pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                panic!(concat!("TODO: ", stringify!($name), " is not ported"))
            } )*
        };
    }
    todo_fns! {
        fn efAsync_LoadSync(a0: i32) -> i32;
        fn efLib_Init();
        fn ftDemo_ObjAllocInit();
        fn gm_801A4B60();
        fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
        fn gm_IsCKindUnlocked(a0: u8) -> bool;
        fn grLib_801C9A10() -> *mut Vec;
        fn Item_80266FA8();
        fn Item_80266FCC();
        fn Stage_8022524C();
        fn Stage_8022532C(a0: StKind, a1: i32);
        fn Stage_InitScene(a0: StKind, a1: i32);
        fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj;
        fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj;
        fn lb_800145F4();
        fn lbAudioAx_800236DC();
        fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32;
        fn lbAudioAx_80023F28(a0: i32);
        fn lbAudioAx_80024E50(a0: i32);
        fn lbAudioAx_80026E84(a0: CharacterKind) -> u64;
        fn lbAudioAx_80026F2C(a0: i32);
        fn lbAudioAx_8002702C(a0: u32, a1: u64);
        fn lbAudioAx_80027168();
        fn lbAudioAx_80027648();
        fn lbShadow_8000F38C(a0: i32);
        fn Player_80032768(a0: i32, a1: *mut Vec);
        fn Player_80036E20(a0: CharacterKind, a1: *mut HSD_Archive, a2: i32);
        fn Player_80036F34(a0: i32, a1: i32);
        fn Player_GetEntity(a0: i32) -> *mut HSD_GObj;
        fn Player_InitAllPlayers();
        fn Player_SetCostumeId(a0: i32, a1: i32);
        fn Player_SetFacingDirection(a0: i32, a1: f32);
        fn Player_SetFlagsBit5(a0: i32, a1: u8);
        fn Player_SetPlayerCharacter(a0: i32, a1: CharacterKind);
        fn Player_SetPlayerId(a0: i32, a1: i32);
        fn Player_SetSlottype(a0: i32, a1: Gm_PKind);
        fn viGetCharAnimByIndex(a0: i32) -> *mut i8;
        fn vi_8031C9B4(a0: i32, a1: i32);
        fn vi_8031CA04(a0: *mut HSD_GObj);
        fn vi_8031CAAC();
        fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
        fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8, a3: u32);
        fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u32);
        fn HSD_CObjAddAnim(a0: *mut HSD_CObj, a1: *mut HSD_CameraAnim);
        fn HSD_CObjAnim(a0: *mut HSD_CObj);
        fn HSD_CObjEndCurrent();
        fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32);
        fn HSD_CObjReqAnim(a0: *mut HSD_CObj, a1: f32);
        fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32;
        fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u8);
        fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32);
        fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32);
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj;
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
        fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    }
    /// Variadic in C: `(filename, (void** out, const char* name)..., NULL)`; `args` follows `filename`.
    pub unsafe fn lbArchive_LoadSymbols(filename: *const i8, args: &[*mut c_void]) -> *mut HSD_Archive {
        { return unsafe { melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(filename as _, args as _) as _ }; }
    }
    pub static mut HSD_GObj_JObjKind: u8 = 0;
    pub static mut HSD_GObj_CameraKind: u8 = 0;
    pub static mut HSD_GObj_LightKind: i8 = 0;
}

/// `vi_RunCamera` (vi/vi.h, static inline).
unsafe fn vi_RunCamera(gobj: *mut HSD_GObj, erase_colors: *const u8, prio: u64) {
    if ext::HSD_CObjSetCurrent(GET_COBJ(gobj)) != 0 {
        ext::HSD_SetEraseColor(*erase_colors, *erase_colors.add(1), *erase_colors.add(2), *erase_colors.add(3));
        ext::HSD_CObjEraseScreen(GET_COBJ(gobj), 1, 0, 1);
        ext::vi_8031CA04(gobj);
        (*gobj).gxlink_prios = prio;
        ext::HSD_GObj_80390ED0(gobj, 0x7);
        ext::HSD_CObjEndCurrent();
    }
}

pub static mut un_804D6FD8: [u8; 8] = [0; 8];
pub static mut un_804D5B08: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0xff };

#[repr(C)]
pub struct Vi1101Data {
    pub spawns: [Vec3; 3],
    pub vi1101_dat: [u8; 0xC],
    pub visual1101_scene: [u8; 0x10],
    pub visual1101_cam2_scene: [u8; 0x14],
}

const fn pad<const N: usize>(s: &str) -> [u8; N] {
    let b = s.as_bytes();
    let mut o = [0u8; N];
    let mut i = 0;
    while i < b.len() {
        o[i] = b[i];
        i += 1;
    }
    o
}

const fn v3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

static mut un_80400200: Vi1101Data = Vi1101Data {
    spawns: [v3(0.0, 0.0, 0.0), v3(15.8512, 67.0, 0.0), v3(-15.7657, 67.0, 0.0)],
    vi1101_dat: pad("Vi1101.dat"),
    visual1101_scene: pad("visual1101Scene"),
    visual1101_cam2_scene: pad("visual1101Cam2Scene"),
};

static mut un_804D6FC0: *mut SceneDesc = null_mut();
static mut un_804D6FC4: *mut SceneDesc = null_mut();
static mut un_804D6FC8: *mut HSD_Archive = null_mut();
static mut un_804D6FCC: *mut HSD_Archive = null_mut();
static mut un_804D6FD0: [*mut HSD_GObj; 2] = [null_mut(); 2];

pub unsafe extern "C" fn un_8031F294(arg0: i32, arg1: i32) {
    ext::Stage_InitScene(St_Kind_Battle, 0);
    ext::Item_80266FA8();
    ext::Item_80266FCC();
    ext::Stage_8022524C();
    ext::Stage_8022532C(St_Kind_Battle, 0);
    ext::ftDemo_ObjAllocInit();
    ext::Player_InitAllPlayers();
    let spawns = &raw mut un_80400200.spawns as *mut Vec;
    ext::Player_80036E20(arg0 as CharacterKind, un_804D6FC8, 3);
    ext::Player_SetPlayerCharacter(0, arg0 as CharacterKind);
    ext::Player_SetCostumeId(0, arg1);
    ext::Player_SetPlayerId(0, 0);
    ext::Player_SetSlottype(0, Gm_PKind_Demo);
    ext::Player_SetFacingDirection(0, 1.0);
    ext::Player_80032768(0, spawns);
    ext::Player_80036F34(0, 8);
    let pmtx = ext::grLib_801C9A10();
    ext::Player_80036E20(CKind_Mario, un_804D6FCC, 5);
    ext::Player_SetPlayerCharacter(1, CKind_Mario);
    ext::Player_SetCostumeId(1, 0);
    ext::Player_SetPlayerId(1, 0);
    ext::Player_SetSlottype(1, Gm_PKind_Demo);
    ext::Player_SetFacingDirection(1, 0.0);
    ext::Player_80032768(1, spawns);
    ext::Player_SetFlagsBit5(1, 1);
    ext::Player_80036F34(1, 0xA);
    un_804D6FD0[0] = ext::Player_GetEntity(1);
    let mut jobj = GET_JOBJ(un_804D6FD0[0]);
    ext::HSD_JObjReqAnimAll(jobj, 432.0);
    ext::HSD_JObjAnimAll(jobj);
    *pmtx.add(1) = un_80400200.spawns[1];
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    if ext::gm_IsCKindUnlocked(CKind_Luigi as u8) {
        ext::Player_80036E20(CKind_Luigi, un_804D6FCC, 5);
        ext::Player_SetPlayerCharacter(2, CKind_Luigi);
        ext::Player_SetCostumeId(2, 0);
        ext::Player_SetPlayerId(2, 0);
        ext::Player_SetSlottype(2, Gm_PKind_Demo);
        ext::Player_SetFacingDirection(2, 0.0);
        ext::Player_80032768(2, spawns);
        ext::Player_SetFlagsBit5(2, 1);
        ext::Player_80036F34(2, 0xA);
        un_804D6FD0[1] = ext::Player_GetEntity(2);
        jobj = GET_JOBJ(un_804D6FD0[1]);
        ext::HSD_JObjReqAnimAll(jobj, 432.0);
        ext::HSD_JObjAnimAll(jobj);
        *pmtx.add(2) = un_80400200.spawns[2];
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
    }
    ext::lbAudioAx_80026F2C(0x1C);
    ext::lbAudioAx_8002702C(0xC, (0x20u64 << 48) | ext::lbAudioAx_80026E84(arg0 as CharacterKind));
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
}

pub unsafe extern "C" fn fn_8031F548(gobj: *mut HSD_GObj) {
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
}

unsafe extern "C" fn fn_8031F56C(gobj: *mut HSD_GObj, unused: i32) {
    ext::lbShadow_8000F38C(0);
    vi_RunCamera(gobj, &raw const un_804D5B08 as *const u8, 0x281);
}

pub unsafe extern "C" fn fn_8031F600(gobj: *mut HSD_GObj) {
    let cobj = GET_COBJ(gobj);
    ext::HSD_CObjAnim(cobj);
    let aobj = (*cobj).aobj;

    if 170.0f32 == (*aobj).curr_frame {
        if ext::gm_IsCKindUnlocked(CKind_Luigi as u8) {
            ext::vi_8031C9B4(0xD, 0);
            ext::lbAudioAx_800237A8(0x209, 0x7F, 0x40);
        }
    }
    if 190.0f32 == (*aobj).curr_frame {
        ext::vi_8031C9B4(0xD, 0);
        ext::lbAudioAx_800237A8(0x209, 0x7F, 0x40);
    }
    if 241.0f32 == (*aobj).curr_frame {
        if ext::gm_IsCKindUnlocked(CKind_Luigi as u8) {
            ext::lbAudioAx_800237A8(0x20A, 0x7F, 0x40);
        }
    }
    if 271.0f32 == (*aobj).curr_frame {
        ext::lbAudioAx_800237A8(0x20A, 0x7F, 0x40);
    }
    if (*aobj).curr_frame == (*aobj).end_frame {
        ext::lb_800145F4();
        ext::gm_801A4B60();
    }
}

pub unsafe extern "C" fn vi1101_Scene_OnEnter(arg: *mut c_void) {
    let mut index: u8 = 0;
    let input = arg as *const u8;
    let data = &raw mut un_80400200;
    ext::lbAudioAx_800236DC();
    ext::efLib_Init();
    ext::efAsync_LoadSync(index as i32);
    ext::lbAudioAx_80023F28(0x55);
    ext::lbAudioAx_80024E50(1);

    index = *input;
    un_804D6FCC = ext::lbArchive_LoadSymbols(
        (*data).vi1101_dat.as_ptr() as *const i8,
        &[
            &raw mut un_804D6FC0 as *mut c_void,
            (*data).visual1101_scene.as_mut_ptr() as *mut c_void,
            &raw mut un_804D6FC4 as *mut c_void,
            (*data).visual1101_cam2_scene.as_mut_ptr() as *mut c_void,
            null_mut(),
        ],
    );
    un_804D6FC8 = ext::lbArchive_LoadSymbols(ext::viGetCharAnimByIndex(index as i32), &[null_mut()]);

    let light_gobj = ext::GObj_Create(0xB, 3, 0);
    ext::HSD_GObjObject_80390A70(
        light_gobj,
        (ext::HSD_GObj_LightKind as u8 as u32 & 0xFFFF) as u8,
        ext::lb_80011AC4((*un_804D6FC0).lights) as *mut c_void,
    );
    ext::GObj_SetupGXLink(light_gobj, Some(ext::HSD_GObj_LObjCallback), 0, 0);

    let scene = if ext::gm_IsCKindUnlocked(CKind_Luigi as u8) { un_804D6FC0 } else { un_804D6FC4 };

    let camera_gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::lb_80013B14(&raw mut (*(*(*scene).cameras).desc).perspective);
    ext::HSD_GObjObject_80390A70(camera_gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(camera_gobj, Some(fn_8031F56C), 5);
    ext::HSD_CObjAddAnim(cobj, *(*(*scene).cameras).anims);
    ext::HSD_CObjReqAnim(cobj, 0.0);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(camera_gobj, Some(fn_8031F600), 0);

    let mut i: isize = 0;
    while !(*(*un_804D6FC0).models.offset(i)).is_null() {
        let model = *(*un_804D6FC0).models.offset(i);
        let model_gobj = ext::GObj_Create(0xE, 0xF, 0);
        let joint = (*model).joint;
        let jobj = ext::HSD_JObjLoadJoint(joint);
        let jobj_kind = ext::HSD_GObj_JObjKind;
        ext::HSD_GObjObject_80390A70(model_gobj, jobj_kind, jobj as *mut c_void);
        ext::GObj_SetupGXLink(model_gobj, Some(ext::HSD_GObj_JObjCallback), 9, 0);
        ext::gm_8016895C(jobj, model, 0);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        ext::HSD_GObj_SetupProc(model_gobj, Some(fn_8031F548), 0x17);
        i += 1;
    }

    un_8031F294(*input as i32, *input.add(1) as i32);
    ext::lbAudioAx_80024E50(0);
}

pub unsafe extern "C" fn vi1101_Scene_OnFrame() {
    ext::vi_8031CAAC();
}
