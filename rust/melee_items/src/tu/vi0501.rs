//! Native port of src/melee/vi/vi0501.c: the Kirby / Green Greens opening vignette (the chosen
//! fighter plus three Kirby copies on the stage). Every function of the TU is here; the `vi.c`
//! helpers are the sibling port [`super::vi`]. Other callees go through `ext` (an unset hook
//! panics "TODO: <name> is not ported"). `lbArchive_LoadSymbols` is variadic in C: the hook takes
//! the `(out, name)` pairs as a slice (the NULL terminator is the slice end).
//!
//! Native-layout note: `un_8031D9F8`'s last parameter is declared `int` in C and receives the
//! ADDRESS of `ViCharaDesc.spawn_count` (cast through `int`, then read back as a `u8*`); that
//! truncates on a 64-bit host, so here it is a `usize` address.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]
use super::vi::{vi_8031C9B4, viGetCharAnimByIndex, vi_8031CA04, vi_8031CAAC};
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn lbArchive_LoadSymbols(a0: *const i8, a1: &[*mut c_void]) -> *mut HSD_Archive => unsafe { melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(a0 as _, a1) };
    fn Camera_Init(a0: i32);
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn Ground_801C0378(a0: i32) => unsafe { melee_world::tu::ground::Ground_801C0378(a0) };
    fn Ground_801C0498() -> f32;
    fn Ground_SetParamY(a0: f32);
    fn HSD_CObjAddAnim(a0: *mut HSD_CObj, a1: *mut HSD_CameraAnim) => unsafe { melee_runtime::tu::cobj::HSD_CObjAddAnim(a0, a1) };
    fn HSD_CObjAnim(a0: *mut HSD_CObj) => unsafe { melee_runtime::tu::cobj::HSD_CObjAnim(a0) };
    fn HSD_CObjEndCurrent() => unsafe { melee_runtime::tu::cobj::HSD_CObjEndCurrent() };
    fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32) => unsafe { melee_runtime::tu::cobj::HSD_CObjEraseScreen(a0, a1, a2, a3) };
    fn HSD_CObjReqAnim(a0: *mut HSD_CObj, a1: f32) => unsafe { melee_runtime::tu::cobj::HSD_CObjReqAnim(a0, a1) };
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 => unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0) as _ };
    fn HSD_FogLoadDesc(a0: *mut HSD_FogDesc) -> *mut HSD_Fog;
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32);
    fn HSD_GObj_FogCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_FogCallback(a0, a1) };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
    fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1) };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    fn Item_80266FA8() => unsafe { crate::tu::item::Item_80266FA8() };
    fn Item_80266FCC() => unsafe { crate::tu::item::Item_80266FCC() };
    fn Player_80031DA8(a0: i32, a1: i32);
    fn Player_80032768(a0: i32, a1: *mut Vec);
    fn Player_80036E20(a0: u32, a1: *mut HSD_Archive, a2: i32);
    fn Player_80036F34(a0: i32, a1: i32);
    fn Player_GetEntity(a0: i32) -> *mut HSD_GObj;
    fn Player_InitAllPlayers();
    fn Player_SetCostumeId(a0: i32, a1: i32);
    fn Player_SetFacingDirection(a0: i32, a1: f32);
    fn Player_SetFlagsBit1(a0: i32);
    fn Player_SetPlayerCharacter(a0: i32, a1: u32);
    fn Player_SetPlayerId(a0: i32, a1: i32);
    fn Player_SetSlottype(a0: i32, a1: Gm_PKind);
    fn Player_SetUnk4D(a0: i32, a1: i8);
    fn Stage_802251E8(a0: u32, a1: *mut i32);
    fn Stage_8022524C();
    fn Stage_8022532C(a0: u32, a1: i32);
    fn efAsync_LoadSync(a0: i32);
    fn efLib_Init() => unsafe { melee_world::tu::eflib::efLib_Init() };
    fn ftDemo_ObjAllocInit();
    fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
    fn gm_801A4B60();
    fn grLib_801C9A10() -> *mut Vec => unsafe { melee_world::tu::grlib::grLib_801C9A10() as _ };
    fn lbAudioAx_800236DC() -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC() };
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0, a1, a2) };
    fn lbAudioAx_80026F2C(a0: u32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(a0) };
    fn lbAudioAx_8002702C(a0: u32, a1: u64) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(a0, a1) };
    fn lbAudioAx_80027168();
    fn lbAudioAx_80027648() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027648() };
    fn lbShadow_8000F38C(a0: i32) => unsafe { melee_runtime::tu::lbshadow::lbShadow_8000F38C(a0) };
    fn lb_8000FCDC() => unsafe { melee_runtime::tu::lb_00F9::lb_8000FCDC() };
    fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(a0) };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0) };
    fn lb_800145F4();
    fn mn_8022EAE0(a0: *mut HSD_GObj);
    fn mpColl_80041C78() => unsafe { melee_world::tu::mpcoll::mpColl_80041C78() };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_FogCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_FogCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_LObjCallback(a0, a1) }
        pub unsafe extern "C" fn mn_8022EAE0(a0: *mut HSD_GObj) { super::mn_8022EAE0(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_CameraKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_FogKind: i8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_LightKind: i8 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

macro_rules! cs {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *mut i8
    };
}

/* 4D6F70 */
static mut un_804D6F70: *mut SceneDesc = core::ptr::null_mut();
/* 4D6F74 */
static mut un_804D6F74: *mut HSD_Archive = core::ptr::null_mut();
/* 4D6F78 */
static mut un_804D6F78: *mut HSD_Archive = core::ptr::null_mut();
/* erase_colors_vi0501 */
static mut erase_colors_vi0501: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };
/* 4D6F80 */
static mut un_804D6F80: f32 = 0.0;
/* 4D6F84 */
pub static mut un_804D6F84: [u8; 8] = [0; 8];
/* 4A2E98 */
static mut un_804A2E98: [*mut HSD_GObj; 4] = [core::ptr::null_mut(); 4];

static mut initial_pos: Vec = Vec { x: 0.0, y: 0.0, z: 0.0 };

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

/// `static f32 getScale(void)`
unsafe fn getScale() -> f32 {
    1.4285715f32 * un_804D6F80
}

/// `void un_8031D9E4(int, int, int)`
pub unsafe fn un_8031D9E4(arg0: i32, arg1: i32, arg2: i32) {
    un_804D6F84[0] = arg0 as u8;
    un_804D6F84[1] = arg1 as u8;
    un_804D6F84[3] = arg2 as u8;
}

/// `void un_8031D9F8(CharacterKind, int, int, int)`; `spawn_count` is the address of three `u8`s.
pub unsafe fn un_8031D9F8(char_kind: CharacterKind, costume: i32, spawn_mode: i32, spawn_count: usize) {
    let mut v: Vec = core::mem::zeroed();

    Stage_InitScene(St_Kind_Greens, core::ptr::null_mut());
    ext::Item_80266FA8();
    ext::Item_80266FCC();
    un_804D6F80 = ext::Ground_801C0498();
    ext::Ground_SetParamY(0.7);
    ext::Stage_8022524C();
    ext::Stage_8022532C(St_Kind_Greens as u32, 0);
    ext::ftDemo_ObjAllocInit();
    ext::Player_InitAllPlayers();
    ext::Player_80036E20(char_kind as u32, un_804D6F78, 3);
    ext::Player_SetPlayerCharacter(0, char_kind as u32);
    ext::Player_SetCostumeId(0, costume);
    ext::Player_SetPlayerId(0, 0);
    ext::Player_SetSlottype(0, Gm_PKind_Demo);
    ext::Player_SetFacingDirection(0, 1.0);
    ext::Player_80032768(0, &raw mut initial_pos);
    ext::Player_80036F34(0, 8);

    let counts = spawn_count as *mut u8;
    let pos = ext::grLib_801C9A10();
    let mut i: i32 = 1;
    while i < 4 {
        ext::Player_80036E20(CKind_Kirby as u32, un_804D6F74, 6);
        let count = *counts.add((i - 1) as usize) as i32;
        ext::Player_80031DA8(count, spawn_mode);
        ext::Player_SetFlagsBit1(i);
        ext::Player_SetPlayerCharacter(i, CKind_Kirby as u32);
        ext::Player_SetCostumeId(i, spawn_mode);
        ext::Player_SetPlayerId(i, 0);
        ext::Player_SetSlottype(i, Gm_PKind_Demo);
        ext::Player_SetFacingDirection(i, 1.0);
        ext::Player_80032768(i, &raw mut initial_pos);
        ext::Player_SetUnk4D(i, *counts.add((i - 1) as usize) as i8);
        ext::Player_80036F34(i, i + 0xA);
        un_804A2E98[(i - 1) as usize] = ext::Player_GetEntity(i);
        let jobj = (*un_804A2E98[(i - 1) as usize]).hsd_obj as *mut HSD_JObj;
        ext::HSD_JObjReqAnimAll(jobj, 140.0);
        ext::HSD_JObjAnimAll(jobj);
        HSD_JObjGetTranslation2(&*((*un_804A2E98[(i - 1) as usize]).hsd_obj as *mut HSD_JObj), &mut v);
        let scale = getScale();
        v.x *= scale;
        v.y *= scale;
        v.z *= scale;
        *pos.add(i as usize) = v;
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        i += 1;
    }
    ext::lbAudioAx_80026F2C(0x1C);
    ext::lbAudioAx_8002702C(0xC, 0x80000004000);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
}

/// `void vi_8031DC80(HSD_GObj*, int)`
pub unsafe extern "C" fn vi_8031DC80(gobj: *mut HSD_GObj, unused: i32) {
    ext::lbShadow_8000F38C(0);
    vi_RunCamera(gobj, &raw mut erase_colors_vi0501 as *mut u8, 0x281);
}

/// `void fn_8031DD14(HSD_GObj*)`
pub unsafe extern "C" fn fn_8031DD14(gobj: *mut HSD_GObj) {
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;

    ext::HSD_CObjAnim(cobj);
    let mut frame: f32 = (*(*cobj).aobj).curr_frame;

    if 60.0f32 == frame || 70.0f32 == frame || 85.0f32 == frame {
        vi_8031C9B4(0xC, 0);
        ext::lbAudioAx_800237A8(0x222F9, 0x7F, 0x40);

        if 60.0f32 == (*(*cobj).aobj).curr_frame {
            ext::lbAudioAx_800237A8(0x73, 0x7F, 0x40);
        }
        if 70.0f32 == (*(*cobj).aobj).curr_frame {
            ext::lbAudioAx_800237A8(0x74, 0x7F, 0x40);
        }
        if 85.0f32 == (*(*cobj).aobj).curr_frame {
            ext::lbAudioAx_800237A8(0x73, 0x7F, 0x40);
        }
    }

    frame = (*(*cobj).aobj).curr_frame;
    if 98.0f32 == frame || 108.0f32 == frame || 123.0f32 == frame {
        ext::lbAudioAx_800237A8(0x22308, 0x7F, 0x40);
    }

    if (*(*cobj).aobj).curr_frame == (*(*cobj).aobj).end_frame {
        ext::lb_800145F4();
        ext::gm_801A4B60();
    }
}

/// `void vi0501_Scene_OnEnter(void*)`
pub unsafe fn vi0501_Scene_OnEnter(arg: *mut c_void) {
    let desc = arg as *mut ViCharaDesc;
    ext::lbAudioAx_800236DC();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);
    let char_index = (*desc).p1_char_index;
    un_804D6F74 = ext::lbArchive_LoadSymbols(cs!("Vi0501.dat"), &[&raw mut un_804D6F70 as *mut c_void, cs!("visual0501Scene") as *mut c_void]);
    un_804D6F78 = ext::lbArchive_LoadSymbols(viGetCharAnimByIndex(char_index as i32), &[]);
    let fog_gobj = ext::GObj_Create(0xB, 3, 0);
    let fog = ext::HSD_FogLoadDesc((*(*un_804D6F70).fogs).desc);
    ext::HSD_GObjObject_80390A70(fog_gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(fog_gobj, Some(ext::cb::HSD_GObj_FogCallback), 0, 0);
    erase_colors_vi0501 = (*fog).color;
    let light_gobj = ext::GObj_Create(0xB, 3, 0);
    let lobj = ext::lb_80011AC4((*un_804D6F70).lights);
    ext::HSD_GObjObject_80390A70(light_gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(light_gobj, Some(ext::cb::HSD_GObj_LObjCallback), 0, 0);
    let camera_gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::lb_80013B14(&raw mut (*(*(*un_804D6F70).cameras).desc).perspective);
    ext::HSD_GObjObject_80390A70(camera_gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(camera_gobj, Some(vi_8031DC80), 5);
    ext::HSD_CObjAddAnim(cobj, *(*(*un_804D6F70).cameras).anims.add(0));
    ext::HSD_CObjReqAnim(cobj, 0.0);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(camera_gobj, Some(fn_8031DD14), 0);
    let mut i: usize = 0;
    while !(*(*un_804D6F70).models.add(i)).is_null() {
        let model_gobj = ext::GObj_Create(0xE, 0xF, 0);
        let jobj = ext::HSD_JObjLoadJoint((**(*un_804D6F70).models.add(i)).joint);
        let new_var = jobj;
        ext::HSD_GObjObject_80390A70(model_gobj, ext::HSD_GObj_JObjKind, new_var as *mut c_void);
        ext::GObj_SetupGXLink(model_gobj, Some(ext::cb::HSD_GObj_JObjCallback), 9, 0);
        ext::gm_8016895C(jobj, *(*un_804D6F70).models.add(i), (!(*(*un_804D6F70).models.add(i)).is_null()) as i32 * 0);
        ext::HSD_JObjReqAnimAll(new_var, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        ext::HSD_GObj_SetupProc(model_gobj, Some(ext::cb::mn_8022EAE0), 0x17);
        i += 1;
    }

    un_8031D9F8(
        (*desc).p1_char_index as CharacterKind,
        (*desc).p1_costume_index as i32,
        (*desc).p2_costume_index as i32,
        &raw mut (*desc).spawn_count as usize,
    );
    ext::lbAudioAx_800237A8(0x20B, 0x7F, 0x40);
    ext::lbAudioAx_800237A8(0x20C, 0x7F, 0x40);
}

/// `void vi0501_Scene_OnFrame(void)`
pub unsafe fn vi0501_Scene_OnFrame() {
    vi_8031CAAC();
}
