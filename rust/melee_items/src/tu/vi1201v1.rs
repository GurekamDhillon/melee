//! Port of src/melee/vi/vi1201v1.c: the "Vi1201v1" Bowser-vs-fighter intro scene (Final
//! Destination). Callees of unported TUs are TODO stubs in `ext`; `vi_RunCamera` (vi.h inline) and
//! the file's static-inline scene helpers are reproduced.
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

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *mut c_void
    };
}

pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code, unused_variables)]
    use super::*;
    macro_rules! todo_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            $( pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported"))
            } )*
        };
    }
    todo_fns! {
        fn efAsync_LoadSync(a0: i32) -> i32;
        fn efLib_Init() => unsafe { melee_world::tu::eflib::efLib_Init() };
        fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec, a3: *mut f32, a4: *mut f32, a5: u32, a6: u32) -> *mut c_void;
        fn ftDemo_ObjAllocInit();
        fn gm_801A4B60();
        fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
        fn gm_80160438(a0: i32) -> *mut i8;
        fn Ground_InitScene() => unsafe { crate::tu::vi0102::Ground_InitScene() };
        fn Item_80266FA8() => unsafe { crate::tu::item::Item_80266FA8() };
        fn Item_80266FCC() => unsafe { crate::tu::item::Item_80266FCC() };
        fn Stage_802251E8(a0: StKind, a1: i32);
        fn Stage_8022524C() => unsafe { melee_world::tu::stage::Stage_8022524C() };
        fn Stage_8022532C(a0: StKind, a1: i32) => unsafe { melee_world::tu::stage::Stage_8022532C(a0, a1) };
        fn lb_8000C1C0(a0: *mut HSD_JObj, a1: *mut HSD_JObj) => unsafe { melee_runtime::tu::lb_00B0::lb_8000C1C0(a0, a1) };
        fn lb_8000C290(a0: *mut HSD_JObj, a1: *mut HSD_JObj) => unsafe { melee_runtime::tu::lb_00B0::lb_8000C290(a0, a1) };
        fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(a0) };
        fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: i32, a3: i32) -> i32;
        fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0) };
        fn lb_800145F4();
        fn lbAudioAx_800236DC();
        fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0, a1, a2) };
        fn lbAudioAx_80023F28(a0: i32);
        fn lbAudioAx_80024E50(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024E50(a0 != 0) };
        fn lbAudioAx_80026F2C(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(a0 as _) };
        fn lbAudioAx_8002702C(a0: u32, a1: u64) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(a0, a1) };
        fn lbAudioAx_80027168() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027168() };
        fn lbAudioAx_80027648() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027648() };
        fn lbShadow_8000F38C(a0: i32) => unsafe { melee_runtime::tu::lbshadow::lbShadow_8000F38C(a0) };
        fn mn_8022F298(a0: *mut HSD_JObj) -> f32;
        fn Player_80032768(a0: i32, a1: *mut Vec);
        fn Player_80036E20(a0: CharacterKind, a1: *mut HSD_Archive, a2: i32);
        fn Player_80036F34(a0: i32, a1: i32);
        fn Player_GetEntity(a0: i32) -> *mut HSD_GObj;
        fn Player_InitAllPlayers();
        fn Player_SetCostumeId(a0: i32, a1: i32);
        fn Player_SetFacingDirection(a0: i32, a1: f32);
        fn Player_SetPlayerCharacter(a0: i32, a1: CharacterKind);
        fn Player_SetPlayerId(a0: i32, a1: i32);
        fn Player_SetSlottype(a0: i32, a1: Gm_PKind);
        fn Toy_803060BC(a0: i32, a1: i32) -> f32 => unsafe { crate::tu::toy::Toy_803060BC(a0, a1) };
        fn Toy_803124BC() => unsafe { crate::tu::toy::Toy_803124BC() };
        fn vi_8031C9B4(a0: i32, a1: i32) => unsafe { crate::tu::vi::vi_8031C9B4(a0, a1) };
        fn vi_8031CA04(a0: *mut HSD_GObj) => unsafe { crate::tu::vi::vi_8031CA04(a0) };
        fn vi_8031CAAC() => unsafe { crate::tu::vi::vi_8031CAAC() };
        fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
        fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8, a3: u32);
        fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u32);
        fn HSD_CObjAddAnim(a0: *mut HSD_CObj, a1: *mut HSD_CameraAnim) => unsafe { melee_runtime::tu::cobj::HSD_CObjAddAnim(a0, a1) };
        fn HSD_CObjAnim(a0: *mut HSD_CObj) => unsafe { melee_runtime::tu::cobj::HSD_CObjAnim(a0) };
        fn HSD_CObjEndCurrent() => unsafe { melee_runtime::tu::cobj::HSD_CObjEndCurrent() };
        fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32) => unsafe { melee_runtime::tu::cobj::HSD_CObjEraseScreen(a0, a1, a2, a3) };
        fn HSD_CObjReqAnim(a0: *mut HSD_CObj, a1: f32) => unsafe { melee_runtime::tu::cobj::HSD_CObjReqAnim(a0, a1) };
        fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 => unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0) as _ };
        fn HSD_FogLoadDesc(a0: *mut HSD_FogDesc) -> *mut HSD_Fog;
        fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u8);
        fn HSD_GObj_FogCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_FogCallback(a0, a1) };
        fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
        fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1) };
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
        fn HSD_GObjFree(a0: *mut HSD_GObj);
        fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
        fn HSD_JObjGetTranslation2(a0: *mut HSD_JObj, a1: *mut Vec) => unsafe { crate::tu::itklap::HSD_JObjGetTranslation2(a0, a1) };
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
        fn HSD_JObjSetRotationYWithMtxDirty(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::itdosei::HSD_JObjSetRotationYWithMtxDirty(a0, a1) };
        fn HSD_JObjSetScaleX(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::ittincle::HSD_JObjSetScaleX(a0, a1) };
        fn HSD_JObjSetScaleY(a0: *mut HSD_JObj, a1: f32) => unsafe { melee_world::tu::grinl::HSD_JObjSetScaleY(a0, a1) };
        fn HSD_JObjSetScaleZ(a0: *mut HSD_JObj, a1: f32) => unsafe { melee_world::tu::grinl::HSD_JObjSetScaleZ(a0, a1) };
        fn HSD_JObjSetScaleXWithMtxDirty(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::jobj::HSD_JObjSetScaleXWithMtxDirty(a0, a1) };
        fn HSD_JObjSetScaleYWithMtxDirty(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::jobj::HSD_JObjSetScaleYWithMtxDirty(a0, a1) };
        fn HSD_JObjSetScaleZWithMtxDirty(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::jobj::HSD_JObjSetScaleZWithMtxDirty(a0, a1) };
        fn HSD_JObjSetTranslateXWithMtxDirty(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::jobj::HSD_JObjSetTranslateXWithMtxDirty(a0, a1) };
        fn HSD_JObjSetTranslateYWithMtxDirty(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::jobj::HSD_JObjSetTranslateYWithMtxDirty(a0, a1) };
        fn HSD_JObjSetTranslateZWithMtxDirty(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::jobj::HSD_JObjSetTranslateZWithMtxDirty(a0, a1) };
        fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    }
    /// Variadic in C: `(filename, (void** out, const char* name)..., NULL)`; `args` follows `filename`.
    pub unsafe fn lbArchive_LoadSymbols(filename: *const i8, args: &[*mut c_void]) -> *mut HSD_Archive {
        { return unsafe { melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(filename as _, args as _) as _ }; }
    }
    pub static mut HSD_GObj_JObjKind: u8 = 0;
    pub static mut HSD_GObj_CameraKind: u8 = 0;
    pub static mut HSD_GObj_FogKind: i8 = 0;
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

static mut un_804D6FE0: *mut SceneDesc = null_mut();
static mut un_804D6FE4: *mut SceneDesc = null_mut();
static mut un_804D6FE8: *mut HSD_Archive = null_mut();
static mut un_804D6FEC: *mut HSD_Joint = null_mut();
static mut un_804D6FF0: *mut HSD_JObj = null_mut();
static mut un_804D6FF4: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };
static mut un_804D6FF8: i32 = 0;
static mut un_804D6FFC: u8 = 0;
static mut un_804D6FFD: u8 = 0;
static mut un_804D7000: *mut c_void = null_mut();
pub static mut un_804D7004: [u8; 8] = [0; 8];

pub static mut player_spawn: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

pub unsafe extern "C" fn un_8031F980(arg0: i32, arg1: i32) {
    un_804D7004[0] = arg0 as u8;
    un_804D7004[1] = arg1 as u8;
}

pub unsafe extern "C" fn un_8031F990(gobj: *mut HSD_GObj) {
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
}

pub unsafe extern "C" fn un_8031F9B4(gobj: *mut HSD_GObj) {
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
}

pub unsafe extern "C" fn un_8031F9D8(char_index: CharacterKind, costume_id: i32) {
    ext::ftDemo_ObjAllocInit();
    ext::Player_InitAllPlayers();
    ext::Player_80036E20(char_index, un_804D6FE8, 0);
    ext::Player_SetPlayerCharacter(0, char_index);
    ext::Player_SetCostumeId(0, costume_id);
    ext::Player_SetPlayerId(0, 0);
    ext::Player_SetSlottype(0, 2);
    ext::Player_SetFacingDirection(0, 0.0);
    ext::Player_80032768(0, &raw mut player_spawn);
    ext::Player_80036F34(0, 1);
    un_804D7000 = ext::Player_GetEntity(0) as *mut c_void;
    ext::lbAudioAx_80026F2C(0x18);
    ext::lbAudioAx_8002702C(0x8, 0x20u64 << 48);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
}

pub unsafe extern "C" fn fn_8031FAA8(gobj: *mut HSD_GObj) {
    let mut pos: Vec = core::mem::zeroed();
    let mut child: *mut HSD_JObj = null_mut();

    ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
    let mut jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if ext::mn_8022F298(jobj) != 120.0 {
        return;
    }
    ext::lb_80011E24(jobj, &mut child, 1, -1);
    jobj = child;
    ext::HSD_JObjGetTranslation2(jobj, &mut pos);
    let mut scale_x: f32 = 0.0;
    let mut scale_y: f32 = 1.0;

    ext::efSync_Spawn(0x42B, gobj, &mut pos, &mut scale_x, &mut scale_y, 0x00FFFFFF, 0x00808080);
    ext::lbAudioAx_800237A8(0x61, 0x7F, 0x40);
}

pub unsafe extern "C" fn fn_8031FB90(gobj: *mut HSD_GObj, unused: i32) {
    if !un_804D7000.is_null() {
        ext::lbShadow_8000F38C(0);
    }
    vi_RunCamera(gobj, &raw const un_804D6FF4 as *const u8, 0x881);
}

pub unsafe extern "C" fn fn_8031FC30(gobj: *mut HSD_GObj) {
    let cobj = GET_COBJ(gobj);
    ext::HSD_CObjAnim(cobj);
    if (*(*cobj).aobj).curr_frame == 120.0 {
        ext::vi_8031C9B4(0xD, 0);
    }
    if (*(*cobj).aobj).curr_frame == 100.0 {
        un_8031F9D8(un_804D6FFC as CharacterKind, un_804D6FFD as i32);
    }
    if (*(*cobj).aobj).curr_frame == (*(*cobj).aobj).end_frame {
        ext::lb_800145F4();
        ext::gm_801A4B60();
    }
}

pub unsafe extern "C" fn fn_8031FCBC(gobj: *mut HSD_GObj) {
    if (un_804D6FF8 as f32) >= 100.0 {
        ext::HSD_GObjFree(gobj);
    } else {
        un_804D6FF8 += 1;
    }
}

unsafe fn un_8031FD18_SetupScene() {
    let mut i: isize = 0;
    while !(*(*un_804D6FE0).models.offset(i)).is_null() {
        let model = *(*un_804D6FE0).models.offset(i);
        let gobj = ext::GObj_Create(0xE, 0xF, 0);
        let jobj = ext::HSD_JObjLoadJoint((*model).joint);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
        ext::gm_8016895C(jobj, model, 0);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        ext::HSD_GObj_SetupProc(gobj, Some(fn_8031FAA8), 0);
        ext::lb_80011E24(jobj, &raw mut un_804D6FF0, 3, -1);
        i += 1;
    }

    ext::Ground_InitScene();
}

unsafe fn un_8031FD18_SetupCamera() {
    let camera_gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::lb_80013B14(&raw mut (*(*(*un_804D6FE0).cameras).desc).perspective);
    ext::HSD_GObjObject_80390A70(camera_gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(camera_gobj, Some(fn_8031FB90), 8);
    ext::HSD_CObjAddAnim(cobj, *(*(*un_804D6FE0).cameras).anims);
    ext::HSD_CObjReqAnim(cobj, 0.0);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(camera_gobj, Some(fn_8031FC30), 0);
}

unsafe fn un_8031FD18_SetupKoopa() {
    let koopa_gobj = ext::GObj_Create(0xE, 0xF, 0);
    let jobj = ext::HSD_JObjLoadJoint(un_804D6FEC);
    ext::HSD_GObjObject_80390A70(koopa_gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(koopa_gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);

    ext::HSD_JObjSetScaleX(jobj, 0.55);
    ext::HSD_JObjSetScaleY(jobj, 0.55);
    ext::HSD_JObjSetScaleZ(jobj, 0.55);

    ext::lb_8000C1C0(jobj, un_804D6FF0);
    ext::lb_8000C290(jobj, un_804D6FF0);
    ext::HSD_GObj_SetupProc(koopa_gobj, Some(un_8031F9B4), 0);
}

unsafe fn un_8031FD18_LoadAssets(input: *const u8) {
    let char_index = *input;

    un_804D6FE8 = ext::lbArchive_LoadSymbols(
        cstr!("Vi1201v1.dat") as *const i8,
        &[&raw mut un_804D6FE0 as *mut c_void, cstr!("visual1201v1Scene"), null_mut()],
    );
    ext::lbArchive_LoadSymbols(
        cstr!("TyKoopa.dat") as *const i8,
        &[&raw mut un_804D6FEC as *mut c_void, cstr!("ToyKoopaModel_TopN_joint"), null_mut()],
    );
    ext::lbArchive_LoadSymbols(
        cstr!("GmRgStnd.dat") as *const i8,
        &[&raw mut un_804D6FE4 as *mut c_void, cstr!("standScene"), null_mut()],
    );
    ext::Toy_803124BC();
    un_804D6FE8 = ext::lbArchive_LoadSymbols(ext::gm_80160438(char_index as i32), &[null_mut()]);
}

unsafe fn un_8031FD18_SetupStand() {
    let stand_gobj = ext::GObj_Create(0xE, 0xF, 0);
    let jobj = ext::HSD_JObjLoadJoint((**(*un_804D6FE4).models).joint);
    ext::HSD_GObjObject_80390A70(stand_gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(stand_gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::HSD_GObj_SetupProc(stand_gobj, Some(un_8031F990), 0);

    let child = if jobj.is_null() { null_mut() } else { (*jobj).child };

    ext::HSD_JObjSetTranslateXWithMtxDirty(child, -ext::Toy_803060BC(0x1E, 0));
    ext::HSD_JObjSetTranslateYWithMtxDirty(child, -ext::Toy_803060BC(0x1E, 1));
    ext::HSD_JObjSetTranslateZWithMtxDirty(child, -ext::Toy_803060BC(0x1E, 2));

    let mut scale = -ext::Toy_803060BC(0x1E, 5);
    ext::HSD_JObjSetRotationYWithMtxDirty(child, scale);

    let a = ext::Toy_803060BC(0x1E, 4);
    let b = 1.0f32 / ext::Toy_803060BC(0x1E, 3);
    scale = 0.55f32 * (a * b);

    ext::HSD_JObjSetScaleXWithMtxDirty(child, scale);
    ext::HSD_JObjSetScaleYWithMtxDirty(child, scale);
    ext::HSD_JObjSetScaleZWithMtxDirty(child, scale);

    ext::lb_8000C1C0(jobj, un_804D6FF0);
    ext::lb_8000C290(jobj, un_804D6FF0);
}

pub unsafe extern "C" fn vi1201v1_Scene_OnEnter(arg: *mut c_void) {
    let input = arg as *const u8;

    un_804D6FFC = *input;
    un_804D6FFD = *input.add(1);
    un_804D7000 = null_mut();

    ext::lbAudioAx_800236DC();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);
    ext::lbAudioAx_80023F28(0x59);
    ext::lbAudioAx_80024E50(1);

    un_8031FD18_LoadAssets(input);

    un_8031FD18_SetupCamera();

    un_8031FD18_SetupScene();
    ext::Stage_802251E8(St_Kind_Last, 0);
    ext::Item_80266FA8();
    ext::Item_80266FCC();
    ext::Stage_8022524C();
    ext::Stage_8022532C(St_Kind_Last, 0x19);

    un_8031FD18_SetupKoopa();

    un_8031FD18_SetupStand();

    let fog_gobj = ext::GObj_Create(0xB, 3, 0);
    let fog = ext::HSD_FogLoadDesc((*(*un_804D6FE0).fogs).desc);
    ext::HSD_GObjObject_80390A70(fog_gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(fog_gobj, Some(ext::HSD_GObj_FogCallback), 0, 0);
    ext::HSD_GObj_SetupProc(fog_gobj, Some(fn_8031FCBC), 0);
    un_804D6FF4 = (*fog).color;
    un_804D6FF8 = 0;

    let light_gobj = ext::GObj_Create(0xB, 3, 0);
    let lobj = ext::lb_80011AC4((*un_804D6FE0).lights);
    ext::HSD_GObjObject_80390A70(light_gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(light_gobj, Some(ext::HSD_GObj_LObjCallback), 0, 0);

    ext::lbAudioAx_80024E50(0);
}

pub unsafe extern "C" fn vi1201v1_Scene_OnFrame() {
    ext::vi_8031CAAC();
}
