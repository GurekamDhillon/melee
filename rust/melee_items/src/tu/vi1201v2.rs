//! Port of src/melee/vi/vi1201v2.c: the "Vi1201v2" Giga Bowser intro scene (Final Destination).
//! Callees of unported TUs are TODO stubs in `ext`; `vi_RunCamera` (vi.h inline) and the file's
//! static-inline scene helpers are reproduced.
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
                $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported"))
            } )*
        };
    }
    todo_fns! {
        fn efAsync_LoadSync(a0: i32) -> i32;
        fn efLib_Init() => unsafe { melee_world::tu::efn::eflib::efLib_Init() };
        fn ftDemo_ObjAllocInit();
        fn gm_801A4B60();
        fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
        fn Item_80266FA8() => unsafe { crate::tu::item::Item_80266FA8() };
        fn Item_80266FCC() => unsafe { crate::tu::item::Item_80266FCC() };
        fn Stage_8022524C() => unsafe { melee_world::tu::stage::Stage_8022524C() };
        fn Stage_8022532C(a0: StKind, a1: i32) => unsafe { melee_world::tu::stage::Stage_8022532C(a0, a1) };
        fn Stage_InitScene(a0: StKind, a1: i32);
        fn lb_8000C1C0(a0: *mut HSD_JObj, a1: *mut HSD_JObj) => unsafe { melee_runtime::tu::lb_00B0::lb_8000C1C0(a0, a1) };
        fn lb_8000C290(a0: *mut HSD_JObj, a1: *mut HSD_JObj) => unsafe { melee_runtime::tu::lb_00B0::lb_8000C290(a0, a1) };
        fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(a0) };
        fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: i32, a3: i32) -> i32;
        fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0) };
        fn lb_800145F4();
        fn lbAudioAx_80023F28(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0); };
        fn lbAudioAx_80024E50(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024E50(a0 != 0) };
        fn lbAudioAx_80026F2C(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(a0 as _) };
        fn lbAudioAx_8002702C(a0: u32, a1: u64) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(a0, a1) };
        fn lbAudioAx_80027168() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027168() };
        fn lbAudioAx_80027648() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027648() };
        fn lbShadow_8000F38C(a0: i32) => unsafe { melee_runtime::tu::lbshadow::lbShadow_8000F38C(a0) };
        fn mn_8022EAE0(a0: *mut HSD_GObj);
        fn mn_8022F298(a0: *mut HSD_JObj) -> f32;
        fn Player_80032768(a0: i32, a1: *mut Vec);
        fn Player_80032BB0(a0: i32) -> f32;
        fn Player_80036E20(a0: CharacterKind, a1: *mut HSD_Archive, a2: i32);
        fn Player_80036F34(a0: i32, a1: i32);
        fn Player_InitAllPlayers();
        fn Player_SetCostumeId(a0: i32, a1: i32);
        fn Player_SetFacingDirection(a0: i32, a1: f32);
        fn Player_SetPlayerAndEntityFacingDirection(a0: i32, a1: f32);
        fn Player_SetPlayerCharacter(a0: i32, a1: CharacterKind);
        fn Player_SetPlayerId(a0: i32, a1: i32);
        fn Player_SetScale(a0: i32, a1: f32);
        fn Player_SetSlottype(a0: i32, a1: Gm_PKind);
        fn Toy_803060BC(a0: i32, a1: i32) -> f32 => unsafe { crate::tu::toy::Toy_803060BC(a0, a1) };
        fn Toy_803124BC() => unsafe { crate::tu::toy::Toy_803124BC() };
        fn viGetCharAnimByIndex(a0: i32) -> *mut i8 => unsafe { crate::tu::vi::viGetCharAnimByIndex(a0) };
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
        fn HSD_FogInterpretAnim(a0: *mut HSD_Fog);
        fn HSD_FogLoadDesc(a0: *mut HSD_FogDesc) -> *mut HSD_Fog;
        fn HSD_FogReqAnim(a0: *mut HSD_Fog, a1: f32);
        fn HSD_Fog_8037DE7C(a0: *mut HSD_Fog, a1: *mut HSD_AObjDesc);
        fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u8);
        fn HSD_GObj_FogCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_FogCallback(a0, a1) };
        fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
        fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1) };
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
        fn HSD_GObjFree(a0: *mut HSD_GObj);
        fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
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
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_FogKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_LightKind;
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

pub static mut un_804002F8: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
pub static mut un_80400304: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

static mut un_804D7010: *mut SceneDesc = null_mut();
static mut un_804D7014: *mut SceneDesc = null_mut();
static mut un_804D7018: *mut HSD_Archive = null_mut();
static mut un_804D701C: *mut HSD_Archive = null_mut();
static mut un_804D7020: *mut HSD_Joint = null_mut();
static mut un_804D7024: *mut HSD_JObj = null_mut();
static mut un_804D7028: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };
static mut un_804D702C: i32 = 0;
static mut un_804D7030: *mut HSD_GObj = null_mut();
static mut un_804D7034: *mut HSD_GObj = null_mut();
pub static mut un_804D7038: [u8; 8] = [0; 8];

pub unsafe extern "C" fn un_803204B0(arg0: i32, arg1: i32) {
    melee_core::ct!("un_803204B0");
    un_804D7038[0] = arg0 as u8;
    un_804D7038[1] = arg1 as u8;
}

pub unsafe extern "C" fn un_803204C0(gobj: *mut HSD_GObj) {
    melee_core::ct!("un_803204C0");
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
}

pub unsafe extern "C" fn un_803204E4(gobj: *mut HSD_GObj) {
    melee_core::ct!("un_803204E4");
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
}

pub unsafe extern "C" fn un_80320508(char_kind: CharacterKind, costume: i32) {
    melee_core::ct!("un_80320508");
    ext::Stage_InitScene(St_Kind_Last, 0);
    ext::Item_80266FA8();
    ext::Item_80266FCC();
    ext::Stage_8022524C();
    ext::Stage_8022532C(St_Kind_Last, 0x1A);
    ext::ftDemo_ObjAllocInit();
    ext::Player_InitAllPlayers();
    ext::Player_80036E20(char_kind, un_804D7018, 3);
    ext::Player_SetPlayerCharacter(0, char_kind);
    ext::Player_SetCostumeId(0, costume);
    ext::Player_SetPlayerId(0, 0);
    ext::Player_SetSlottype(0, 2);
    ext::Player_SetFacingDirection(0, -1.0);
    ext::Player_80032768(0, &raw mut un_804002F8);
    ext::Player_80036F34(0, 8);
    ext::Player_SetPlayerAndEntityFacingDirection(0, 0.0);
}

pub unsafe extern "C" fn un_803205F4() {
    melee_core::ct!("un_803205F4");
    let gobj = ext::GObj_Create(0xE, 0xF, 0);
    let model1 = *(*un_804D7010).models.offset(1);
    let jobj = ext::HSD_JObjLoadJoint((*model1).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::gm_8016895C(jobj, model1, 0);
    ext::HSD_JObjReqAnimAll(jobj, 251.0);
    ext::HSD_GObj_SetupProc(gobj, Some(ext::mn_8022EAE0), 0);

    ext::Player_80036E20(CKind_GKoops, un_804D701C, 8);
    ext::Player_SetPlayerCharacter(1, CKind_GKoops);
    ext::Player_SetCostumeId(1, 0);
    ext::Player_SetPlayerId(1, 0);
    ext::Player_SetSlottype(1, 2);
    ext::Player_SetFacingDirection(1, -1.0);
    ext::Player_80032768(1, &raw mut un_80400304);
    ext::Player_80036F34(1, 0xF);
    ext::Player_SetScale(1, 1.0f32 / ext::Player_80032BB0(1));

    ext::lbAudioAx_80026F2C(0x18);
    ext::lbAudioAx_8002702C(8, 0x0020000000000000u64);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
}

pub unsafe extern "C" fn un_8032074C(gobj: *mut HSD_GObj) {
    melee_core::ct!("un_8032074C");
    let jobj = GET_JOBJ(gobj);
    ext::HSD_JObjAnimAll(jobj);
    if ext::mn_8022F298(jobj) == 251.0 {
        if !un_804D7030.is_null() {
            ext::HSD_GObjFree(un_804D7030);
            un_804D7030 = null_mut();
        }
        if !un_804D7034.is_null() {
            ext::HSD_GObjFree(un_804D7034);
            un_804D7034 = null_mut();
        }
        un_803205F4();
    }
}

#[inline]
unsafe fn GET_EVENTDATA() -> *mut HSD_GObj {
    ext::GObj_Create(0xE, 0xF, 0)
}

pub unsafe extern "C" fn un_803207C4() {
    melee_core::ct!("un_803207C4");
    let mut i: isize = 0;
    while !(*(*un_804D7010).models.offset(i)).is_null() {
        if i != 1 {
            let model = *(*un_804D7010).models.offset(i);
            let gobj = GET_EVENTDATA();
            let jobj = ext::HSD_JObjLoadJoint((*model).joint);
            let new_var = gobj;
            ext::HSD_GObjObject_80390A70(new_var, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
            ext::GObj_SetupGXLink(new_var, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
            ext::gm_8016895C(jobj, model, 0);
            ext::HSD_JObjReqAnimAll(jobj, 0.0);
            ext::HSD_JObjAnimAll(jobj);
            if i == 0 {
                ext::HSD_GObj_SetupProc(new_var, Some(un_8032074C), 0);
                ext::lb_80011E24(jobj, &raw mut un_804D7024, 2, -1);
            } else {
                ext::HSD_GObj_SetupProc(new_var, Some(ext::mn_8022EAE0), 0);
            }
        }
        i += 1;
    }
}

pub unsafe extern "C" fn un_803208F0(gobj: *mut HSD_GObj, unused: i32) {
    melee_core::ct!("un_803208F0");
    ext::lbShadow_8000F38C(0);
    vi_RunCamera(gobj, &raw const un_804D7028 as *const u8, 0x881);
}

pub unsafe extern "C" fn un_80320984(gobj: *mut HSD_GObj) {
    melee_core::ct!("un_80320984");
    let cobj = GET_COBJ(gobj);
    ext::HSD_CObjAnim(cobj);
    if (*(*cobj).aobj).curr_frame == 130.0 || (*(*cobj).aobj).curr_frame == 190.0 {
        ext::vi_8031C9B4(0xD, 0);
    }
    if (*(*cobj).aobj).curr_frame == 251.0 {
        ext::vi_8031C9B4(1, 0xDB);
    }
    if (*(*cobj).aobj).curr_frame == (*(*cobj).aobj).end_frame {
        ext::lb_800145F4();
        ext::gm_801A4B60();
    }
}

pub unsafe extern "C" fn fn_80320A1C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80320A1C");
    ext::HSD_FogInterpretAnim((*gobj).hsd_obj as *mut HSD_Fog);
}

unsafe fn un_80320A40_LoadAssets(input: *const u8) {
    let char_index = *input;

    un_804D701C = ext::lbArchive_LoadSymbols(
        cstr!("Vi1201v2.dat") as *const i8,
        &[&raw mut un_804D7010 as *mut c_void, cstr!("visual1201v2Scene"), null_mut()],
    );
    ext::lbArchive_LoadSymbols(
        cstr!("TyKoopa.dat") as *const i8,
        &[&raw mut un_804D7020 as *mut c_void, cstr!("ToyKoopaModel_TopN_joint"), null_mut()],
    );
    ext::lbArchive_LoadSymbols(
        cstr!("GmRgStnd.dat") as *const i8,
        &[&raw mut un_804D7014 as *mut c_void, cstr!("standScene"), null_mut()],
    );
    ext::Toy_803124BC();
    un_804D7018 = ext::lbArchive_LoadSymbols(ext::viGetCharAnimByIndex(char_index as i32), &[null_mut()]);
}

unsafe fn un_80320A40_SetupCamera() {
    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::lb_80013B14(&raw mut (*(*(*un_804D7010).cameras).desc).perspective);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj, Some(un_803208F0), 8);
    ext::HSD_CObjAddAnim(cobj, *(*(*un_804D7010).cameras).anims);
    ext::HSD_CObjReqAnim(cobj, 0.0);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(gobj, Some(un_80320984), 0);
}

unsafe fn un_80320A40_SetupKoopa() {
    let gobj = ext::GObj_Create(0xE, 0xF, 0);
    un_804D7030 = gobj;
    let jobj = ext::HSD_JObjLoadJoint(un_804D7020);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::HSD_JObjSetScaleX(jobj, 0.55);
    ext::HSD_JObjSetScaleY(jobj, 0.55);
    ext::HSD_JObjSetScaleZ(jobj, 0.55);
    ext::lb_8000C1C0(jobj, un_804D7024);
    ext::lb_8000C290(jobj, un_804D7024);
    ext::HSD_GObj_SetupProc(gobj, Some(un_803204E4), 0x17);
}

unsafe fn un_80320A40_SetupStand() {
    let gobj = ext::GObj_Create(0xE, 0xF, 0);
    un_804D7034 = gobj;
    let jobj = ext::HSD_JObjLoadJoint((**(*un_804D7014).models).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(un_803204C0), 0x17);

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

    ext::lb_8000C1C0(jobj, un_804D7024);
    ext::lb_8000C290(jobj, un_804D7024);
}

pub unsafe extern "C" fn vi1201v2_Scene_OnEnter(arg: *mut c_void) {
    melee_core::ct!("vi1201v2_Scene_OnEnter");
    let input = arg as *const u8;

    ext::efLib_Init();
    ext::efAsync_LoadSync(0);
    ext::lbAudioAx_80023F28(0x5C);
    ext::lbAudioAx_80024E50(1);

    un_80320A40_LoadAssets(input);

    un_80320A40_SetupCamera();

    un_803207C4();
    un_80320508(*input as CharacterKind, *input.add(1) as i32);

    un_80320A40_SetupKoopa();

    un_80320A40_SetupStand();

    let fog_gobj = ext::GObj_Create(0xB, 3, 0);
    let fog = ext::HSD_FogLoadDesc((*(*un_804D7010).fogs).desc);
    ext::HSD_GObjObject_80390A70(fog_gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(fog_gobj, Some(ext::HSD_GObj_FogCallback), 0, 0);
    ext::HSD_Fog_8037DE7C(fog, (**(*(*un_804D7010).fogs).anims).aobjdesc);
    ext::HSD_FogReqAnim(fog, 0.0);
    ext::HSD_FogInterpretAnim(fog);
    ext::HSD_GObj_SetupProc(fog_gobj, Some(fn_80320A1C), 0x17);
    un_804D7028 = (*fog).color;
    un_804D702C = 0;

    let light_gobj = ext::GObj_Create(0xB, 3, 0);
    let lobj = ext::lb_80011AC4((*un_804D7010).lights);
    ext::HSD_GObjObject_80390A70(light_gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(light_gobj, Some(ext::HSD_GObj_LObjCallback), 0, 0);

    ext::lbAudioAx_80024E50(0);
}

pub unsafe extern "C" fn vi1201v2_Scene_OnFrame() {
    melee_core::ct!("vi1201v2_Scene_OnFrame");
    ext::vi_8031CAAC();
}
