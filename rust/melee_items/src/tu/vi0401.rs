//! Native port of src/melee/vi/vi0401.c: the Samus / Zebes opening vignette (one fighter on
//! Brinstar with the damage-HUD overlay camera). Every function of the TU is here; the `vi.c`
//! helpers are the sibling port [`super::vi`]. Other callees go through `ext` (an unset hook
//! panics "TODO: <name> is not ported"). `lbArchive_LoadSymbols` is variadic in C: the hook takes
//! the `(out, name)` pairs as a slice (the NULL terminator is the slice end); `lb_80011E24` is
//! variadic with a fixed `(jobj, out, 1, -1)` call here.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]
use super::vi::{vi_8031C9B4, viGetCharAnimByIndex, vi_8031CA04};
use core::ffi::c_void;
use melee_types::inline::HSD_JObjGetTranslation2;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn lbArchive_LoadSymbols(a0: *const i8, a1: &[*mut c_void]) -> *mut HSD_Archive => unsafe { melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(a0 as _, a1) };
    fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: i32, a3: i32) -> i32;
    fn Camera_Init(a0: i32) => unsafe { melee_world::tu::camera::Camera_Init(a0) };
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn Ground_801C0378(a0: i32) => unsafe { melee_world::tu::ground::Ground_801C0378(a0) };
    fn HSD_CObjAddAnim(a0: *mut HSD_CObj, a1: *mut HSD_CameraAnim) => unsafe { melee_runtime::tu::cobj::HSD_CObjAddAnim(a0, a1) };
    fn HSD_CObjAnim(a0: *mut HSD_CObj) => unsafe { melee_runtime::tu::cobj::HSD_CObjAnim(a0) };
    fn HSD_CObjEndCurrent() => unsafe { melee_runtime::tu::cobj::HSD_CObjEndCurrent() };
    fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32) => unsafe { melee_runtime::tu::cobj::HSD_CObjEraseScreen(a0, a1, a2, a3) };
    fn HSD_CObjReqAnim(a0: *mut HSD_CObj, a1: f32) => unsafe { melee_runtime::tu::cobj::HSD_CObjReqAnim(a0, a1) };
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 => unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0) as _ };
    fn HSD_FogLoadDesc(a0: *mut HSD_FogDesc) -> *mut HSD_Fog;
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32);
    fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32);
    fn HSD_GObj_FogCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_FogCallback(a0, a1) };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
    fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1) };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    fn Item_80266FA8() => unsafe { crate::tu::item::Item_80266FA8() };
    fn Item_80266FCC() => unsafe { crate::tu::item::Item_80266FCC() };
    fn Player_80032768(a0: i32, a1: *mut Vec);
    fn Player_80036E20(a0: u32, a1: *mut HSD_Archive, a2: i32);
    fn Player_80036F34(a0: i32, a1: i32);
    fn Player_InitAllPlayers();
    fn Player_SetCostumeId(a0: i32, a1: i32);
    fn Player_SetFacingDirection(a0: i32, a1: f32);
    fn Player_SetPlayerCharacter(a0: i32, a1: u32);
    fn Player_SetPlayerId(a0: i32, a1: i32);
    fn Player_SetSlottype(a0: i32, a1: Gm_PKind);
    fn Stage_802251E8(a0: u32, a1: *mut i32) => unsafe { melee_world::tu::stage::Stage_802251E8(a0, a1) };
    fn Stage_8022524C() => unsafe { melee_world::tu::stage::Stage_8022524C() };
    fn Stage_8022532C(a0: u32, a1: i32) => unsafe { melee_world::tu::stage::Stage_8022532C(a0, a1) };
    fn efAsync_LoadSync(a0: i32);
    fn efLib_Init() => unsafe { melee_world::tu::efn::eflib::efLib_Init() };
    fn ftDemo_ObjAllocInit();
    fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
    fn gm_8017E430() -> u8;
    fn gm_801A4B60();
    fn gm_GetButtonsTriggered(a0: u8) -> u64;
    fn lbAudioAx_800236DC() -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC() };
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0, a1, a2) };
    fn lbAudioAx_80024634(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024634(a0) };
    fn lbAudioAx_80026F2C(a0: u32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(a0) };
    fn lbAudioAx_8002702C(a0: u32, a1: u64) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(a0, a1) };
    fn lbAudioAx_80027168() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027168() };
    fn lbAudioAx_80027648() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027648() };
    fn lb_8000FCDC() => unsafe { melee_runtime::tu::lb_00F9::lb_8000FCDC() };
    fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(a0) };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0) };
    fn lb_800145F4();
    fn mpColl_80041C78() => unsafe { melee_world::tu::mpcoll::mpColl_80041C78() };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_803910D8(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_FogCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_FogCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_LObjCallback(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_FogKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_LightKind;
}
// END GENERATED EXT

macro_rules! cs {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *mut i8
    };
}

/// `PAD_BUTTON_START` (dolphin/pad.h).
const PAD_BUTTON_START: u64 = 0x1000;

/* 4D6F48 */
static mut un_804D6F48: *mut SceneDesc = core::ptr::null_mut();
/* 4D6F4C */
static mut un_804D6F4C: *mut SceneDesc = core::ptr::null_mut();
/* 4D6F50 */
static mut un_804D6F50: *mut SceneDesc = core::ptr::null_mut();
/* 4D6F54 */
static mut un_804D6F54: *mut HSD_Archive = core::ptr::null_mut();
/* 4D6F58 */
static mut un_804D6F58: *mut HSD_JObj = core::ptr::null_mut();
/* erase_colors_vi0401 */
static mut erase_colors_vi0401: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };
/* 4D6F60 */
pub static mut un_804D6F60: [u8; 8] = [0; 8];

/// `Ground_InitScene` (gr/inlines.h, static inline).
#[inline]
unsafe fn Ground_InitScene() {
    ext::Camera_Init(6);
    ext::lb_8000FCDC();
    ext::mpColl_80041C78();
    ext::Ground_801C0378(0x40);
}

/// `Stage_InitScene` (gr/inlines.h, static inline).
#[inline]
unsafe fn Stage_InitScene(stkind: StKind, arg1: *mut i32) {
    Ground_InitScene();
    ext::Stage_802251E8(stkind as u32, arg1);
}

/// `vi_RunCamera` (vi/vi.h, static inline).
#[inline]
unsafe fn vi_RunCamera(gobj: *mut HSD_GObj, erase_colors: *mut u8, prio: u64) {
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj) != 0 {
        ext::HSD_SetEraseColor(*erase_colors.add(0), *erase_colors.add(1), *erase_colors.add(2), *erase_colors.add(3));
        ext::HSD_CObjEraseScreen((*gobj).hsd_obj as *mut HSD_CObj, 1, 0, 1);
        vi_8031CA04(gobj);
        (*gobj).gxlink_prios = prio;
        ext::HSD_GObj_80390ED0(gobj, 0x7);
        ext::HSD_CObjEndCurrent();
    }
}

/// `void vi0401_8031D020(int, int)`
pub unsafe fn vi0401_8031D020(arg0: i32, arg1: i32) {
    melee_core::ct!("vi0401_8031D020");
    un_804D6F60[0] = arg0 as u8;
    un_804D6F60[1] = arg1 as u8;
}

/// `void un_8031D030(CharacterKind, int)`
pub unsafe fn un_8031D030(char_kind: CharacterKind, costume: i32) {
    melee_core::ct!("un_8031D030");
    let mut pos: Vec = core::mem::zeroed();

    Stage_InitScene(St_Kind_Zebes, core::ptr::null_mut());
    ext::Item_80266FA8();
    ext::Item_80266FCC();
    ext::Stage_8022524C();
    ext::Stage_8022532C(St_Kind_Zebes as u32, 0);

    ext::ftDemo_ObjAllocInit();
    ext::Player_InitAllPlayers();

    ext::Player_80036E20(char_kind as u32, un_804D6F54, 3);
    ext::Player_SetPlayerCharacter(0, char_kind as u32);
    ext::Player_SetCostumeId(0, costume);
    ext::Player_SetPlayerId(0, 0);
    ext::Player_SetSlottype(0, Gm_PKind_Demo);
    ext::Player_SetFacingDirection(0, 1.0);

    HSD_JObjGetTranslation2(&*un_804D6F58, &mut pos);
    ext::Player_80032768(0, &mut pos);
    ext::Player_80036F34(0, 8);

    ext::lbAudioAx_80026F2C(0x18);
    ext::lbAudioAx_8002702C(8, 0x100u64 << 32);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
    ext::lbAudioAx_80024634(0x88);
}

/// `static void fn_8031D168(HSD_GObj*)`
unsafe extern "C" fn fn_8031D168(gobj: *mut HSD_GObj) {
    ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
}

/// `static void vi0401_8031D18C(HSD_GObj*)`
unsafe extern "C" fn vi0401_8031D18C(gobj: *mut HSD_GObj) {
    ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
}

/// `static void vi0401_8031D1B0(HSD_GObj*, int)`
unsafe extern "C" fn vi0401_8031D1B0(gobj: *mut HSD_GObj, unused: i32) {
    vi_RunCamera(gobj, &raw mut erase_colors_vi0401 as *mut u8, 0x281);
}

/// `static void vi0401_8031D23C(HSD_GObj*)`
unsafe extern "C" fn vi0401_8031D23C(gobj: *mut HSD_GObj) {
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;

    ext::HSD_CObjAnim(cobj);
    let aobj = (*cobj).aobj;
    if (*aobj).curr_frame == (*aobj).end_frame {
        ext::lb_800145F4();
        ext::gm_801A4B60();
    }
}

/// `void vi0401_Scene_OnEnter(void*)`
pub unsafe fn vi0401_Scene_OnEnter(data: *mut c_void) {
    melee_core::ct!("vi0401_Scene_OnEnter");
    let desc = data as *mut ViCharaDesc;

    ext::lbAudioAx_800236DC();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);

    let char_index = (*desc).p1_char_index;

    ext::lbArchive_LoadSymbols(
        cs!("Vi0401.dat"),
        &[
            &raw mut un_804D6F48 as *mut c_void,
            cs!("visual0401Scene") as *mut c_void,
            &raw mut un_804D6F4C as *mut c_void,
            cs!("visual0401InfoScene") as *mut c_void,
        ],
    );
    ext::lbArchive_LoadSymbols(cs!("IfAll.dat"), &[&raw mut un_804D6F50 as *mut c_void, cs!("ScInfDmg_scene_data") as *mut c_void]);
    un_804D6F54 = ext::lbArchive_LoadSymbols(viGetCharAnimByIndex(char_index as i32), &[]);

    let fog_gobj = ext::GObj_Create(0xB, 3, 0);
    let fog = ext::HSD_FogLoadDesc((*(*un_804D6F48).fogs).desc);
    ext::HSD_GObjObject_80390A70(fog_gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(fog_gobj, Some(ext::cb::HSD_GObj_FogCallback), 0, 0);
    erase_colors_vi0401 = (*fog).color;

    let light_gobj = ext::GObj_Create(0xB, 3, 0);
    let lobj = ext::lb_80011AC4((*un_804D6F48).lights);
    ext::HSD_GObjObject_80390A70(light_gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(light_gobj, Some(ext::cb::HSD_GObj_LObjCallback), 0, 0);

    let cam_gobj = ext::GObj_Create(0x13, 0x14, 0);
    let mut cobj = ext::lb_80013B14(&raw mut (*(*(*un_804D6F48).cameras).desc).perspective);
    ext::HSD_GObjObject_80390A70(cam_gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(cam_gobj, Some(vi0401_8031D1B0), 0x5);
    let mut idx: usize = 0;
    (*cam_gobj).gxlink_prios = 0x2A9;
    ext::HSD_CObjAddAnim(cobj, *(*(*un_804D6F48).cameras).anims.add(0));
    ext::HSD_CObjReqAnim(cobj, 0.0);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(cam_gobj, Some(vi0401_8031D23C), 0);

    while !(*(*un_804D6F48).models.add(idx)).is_null() {
        let gobj = ext::GObj_Create(0xE, 0xF, 0);
        let jobj = ext::HSD_JObjLoadJoint((**(*un_804D6F48).models.add(idx)).joint);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        ext::GObj_SetupGXLink(gobj, Some(ext::cb::HSD_GObj_JObjCallback), 9, 0);
        ext::gm_8016895C(jobj, *(*un_804D6F48).models.add(idx), (!(*(*un_804D6F48).models.add(idx)).is_null()) as i32 * 0);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        ext::HSD_GObj_SetupProc(gobj, Some(vi0401_8031D18C), 0x17);
        if idx == 0 {
            ext::lb_80011E24(jobj, &raw mut un_804D6F58, 1, -1);
        }
        idx += 1;
    }

    let cam_gobj2 = ext::GObj_Create(0x13, 0x14, 0);
    cobj = ext::lb_80013B14(&raw mut (*(*(*un_804D6F50).cameras).desc).perspective);
    ext::HSD_GObjObject_80390A70(cam_gobj2, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(cam_gobj2, Some(ext::cb::HSD_GObj_803910D8), 0x8);
    (*cam_gobj2).gxlink_prios = 0x801;

    idx = 0;
    while !(*(*un_804D6F4C).models.add(idx)).is_null() {
        let gobj2 = ext::GObj_Create(0xE, 0xF, 0);
        let jobj2 = ext::HSD_JObjLoadJoint((**(*un_804D6F4C).models.add(idx)).joint);
        ext::HSD_GObjObject_80390A70(gobj2, ext::HSD_GObj_JObjKind, jobj2 as *mut c_void);
        ext::GObj_SetupGXLink(gobj2, Some(ext::cb::HSD_GObj_JObjCallback), 0xB, 0);
        ext::gm_8016895C(jobj2, *(*un_804D6F4C).models.add(idx), (!(*(*un_804D6F4C).models.add(idx)).is_null()) as i32 * 0);
        ext::HSD_JObjReqAnimAll(jobj2, 0.0);
        ext::HSD_JObjAnimAll(jobj2);
        ext::HSD_GObj_SetupProc(gobj2, Some(fn_8031D168), 0x17);
        idx += 1;
    }
    un_8031D030((*desc).p1_char_index as CharacterKind, (*desc).p1_costume_index as i32);
    vi_8031C9B4(1, 0);

    ext::lbAudioAx_800237A8(0x61a88, 0x7F, 0x40);
    ext::lbAudioAx_800237A8(0x61a89, 0x7F, 0x40);
    ext::lbAudioAx_800237A8(0x61a8a, 0x7F, 0x40);
}

/// `void vi0401_Scene_OnFrame(void)`
pub unsafe fn vi0401_Scene_OnFrame() {
    melee_core::ct!("vi0401_Scene_OnFrame");
    let result: u64 = ext::gm_GetButtonsTriggered(ext::gm_8017E430());
    if (result & PAD_BUTTON_START) != 0 {
        ext::lb_800145F4();
        ext::gm_801A4B60();
    }
}
