//! Port of src/melee/vi/vi0801.c: the "Vi0801" Big Blue Route opening scene (camera, model gobjs,
//! stage init, fog and light gobjs). Callees of unported TUs are TODO stubs in `ext`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *mut c_void
    };
}

#[inline]
unsafe fn GET_JOBJ(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code, unused_variables)]
    use super::*;
    // `hook::<name>`: set by the engine (melee_engine::seams::install_vi_hooks) for the callees in crates this one cannot
    // name (the players, the scene loop, the menus); checked before the default
    macro_rules! todo_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                #[allow(unused_imports)]
                use super::*;
                $( pub static mut $name: Option<unsafe fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                if let Some(f) = unsafe { hook::$name } {
                    return f($($a),*);
                }
                $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported"))
            } )*
        };
    }
    todo_fns! {
        fn Camera_800310A0(a0: i32);
        fn efAsync_LoadSync(a0: i32) -> i32;
        fn efLib_Init() => unsafe { melee_world::tu::efn::eflib::efLib_Init() };
        fn gm_801A4B60();
        fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
        fn grBigBlueRoute_8020DAB4(a0: *mut *mut HSD_JObj, a1: f32, a2: i32) => unsafe { melee_world::tu::grbigblueroute::grBigBlueRoute_8020DAB4(a0, a1, a2) };
        fn Item_80266FA8() => unsafe { crate::tu::item::Item_80266FA8() };
        fn Item_80266FCC() => unsafe { crate::tu::item::Item_80266FCC() };
        fn Stage_8022524C() => unsafe { melee_world::tu::stage::Stage_8022524C() };
        fn Stage_8022532C(a0: StKind, a1: i32) => unsafe { melee_world::tu::stage::Stage_8022532C(a0, a1) };
        fn Stage_InitScene(a0: StKind, a1: i32);
        fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(a0) };
        fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: i32, a3: i32) -> i32;
        fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0) };
        fn lb_800145F4();
        fn lbAudioAx_800236DC() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC(); };
        fn lbAudioAx_80023F28(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0); };
        fn lbAudioAx_80024E50(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024E50(a0 != 0) };
        fn lbAudioAx_80026F2C(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(a0 as _) };
        fn lbAudioAx_8002702C(a0: u32, a1: u64) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(a0, a1) };
        fn lbAudioAx_80027168() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027168() };
        fn lbAudioAx_80027648() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027648() };
        fn Player_InitAllPlayers();
        fn vi_8031C9B4(a0: i32, a1: i32) => unsafe { crate::tu::vi::vi_8031C9B4(a0, a1) };
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
        fn HSD_GObj_FogCallback(a0: *mut HSD_GObj, a1: isize) => unsafe { melee_runtime::tu::gobj::HSD_GObj_FogCallback(a0, a1 as i32) };
        fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: isize) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1 as i32) };
        fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: isize) => unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1 as i32) };
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
        fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
        fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    }
    /// `lbArchive_LoadSymbols(filename, (void** out, const char* name)..., NULL)`: variadic in C;
    /// `args` is the argument list after `filename`, NULL-terminated.
    pub unsafe fn lbArchive_LoadSymbols(filename: *const i8, args: &[*mut c_void]) -> *mut HSD_Archive {
        { return unsafe { melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(filename as _, args as _) as _ }; }
    }
    // Data defined in other TUs.
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_FogKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_LightKind;
}

static mut un_804D6FB8: *mut SceneDesc = null_mut();
static mut un_804D6FBC: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };

pub static mut un_804A2EA8: [*mut HSD_JObj; 24] = [null_mut(); 24];

pub static mut un_80400128: [[i32; 2]; 23] = [
    [1, 2], [1, 3], [1, 4], [1, 5], [1, 6],
    [1, 7], [1, 8], [1, 9], [1, 10], [0, 2],
    [0, 3], [0, 4], [0, 5], [0, 6], [0, 7],
    [0, 8], [0, 9], [0, 10], [0, 11], [2, 2],
    [2, 3], [2, 4], [2, 5],
];

unsafe extern "C" fn vi0801_8031ED70(gobj: *mut HSD_GObj, unused: isize) {
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj) != 0 {
        let colors = &raw mut un_804D6FBC;
        ext::HSD_SetEraseColor((*colors).r, (*colors).g, (*colors).b, (*colors).a);
        ext::HSD_CObjEraseScreen((*gobj).hsd_obj as *mut HSD_CObj, 1, 0, 1);
        ext::Camera_800310A0(2);
        (*gobj).gxlink_prios = 9;
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::Camera_800310A0(1);
        (*gobj).gxlink_prios = 8;
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::Camera_800310A0(0);
        (*gobj).gxlink_prios = 8;
        ext::HSD_GObj_80390ED0(gobj, 7);
        (*gobj).gxlink_prios = 0x8A1;
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_CObjEndCurrent();
    }
}

pub unsafe extern "C" fn vi0801_8031EE60(gobj: *mut HSD_GObj) {
    melee_core::ct!("vi0801_8031EE60");
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
}

pub unsafe extern "C" fn vi0801_8031EE84() {
    melee_core::ct!("vi0801_8031EE84");
    let mut i: i32 = 0;
    while !(*(*un_804D6FB8).models.offset(i as isize)).is_null() {
        let model = *(*un_804D6FB8).models.offset(i as isize);
        let gobj = ext::GObj_Create(0xE, 0xF, 0);
        let jobj = ext::HSD_JObjLoadJoint((*model).joint);
        let temp = jobj;
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, temp as *mut c_void);
        ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
        ext::gm_8016895C(jobj, model, 0);
        ext::HSD_JObjReqAnimAll(temp, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        ext::HSD_GObj_SetupProc(gobj, Some(vi0801_8031EE60), 23);

        let mut j: usize = 0;
        while j < 23 {
            if i == un_80400128[j][0] {
                ext::lb_80011E24(jobj, &raw mut un_804A2EA8[j], un_80400128[j][1], -1);
            }
            j += 1;
        }
        i += 1;
    }

    ext::lbAudioAx_80026F2C(0x18);
    ext::lbAudioAx_8002702C(8, 0x200000000000u64);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
}

pub unsafe extern "C" fn vi0801_8031EFE4(gobj: *mut HSD_GObj) {
    melee_core::ct!("vi0801_8031EFE4");
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    ext::HSD_CObjAnim(cobj);
    if (*(*cobj).aobj).curr_frame == 75.0 || (*(*cobj).aobj).curr_frame == 160.0 {
        ext::vi_8031C9B4(0xC, 0);
    }
    if (*(*cobj).aobj).curr_frame == 120.0 {
        ext::vi_8031C9B4(0x10, 0);
    }
    if (*(*cobj).aobj).curr_frame == (*(*cobj).aobj).end_frame {
        ext::lb_800145F4();
        ext::gm_801A4B60();
    }
}

pub unsafe extern "C" fn vi0801_Scene_OnEnter(unused: *mut c_void) {
    melee_core::ct!("vi0801_Scene_OnEnter");
    ext::lbAudioAx_800236DC();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);
    ext::lbAudioAx_80023F28(0x5B);
    ext::lbAudioAx_80024E50(1);

    ext::lbArchive_LoadSymbols(
        cstr!("Vi0801.dat") as *const i8,
        &[&raw mut un_804D6FB8 as *mut c_void, cstr!("visual0801Scene"), null_mut()],
    );

    let mut gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::lb_80013B14(&raw mut (*(*(*un_804D6FB8).cameras).desc).perspective);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj, Some(vi0801_8031ED70), 8);
    ext::HSD_CObjAddAnim(cobj, *(*(*un_804D6FB8).cameras).anims);
    ext::HSD_CObjReqAnim(cobj, 0.0);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(gobj, Some(vi0801_8031EFE4), 0);

    vi0801_8031EE84();

    ext::Stage_InitScene(St_Kind_BigBlueRoute, 0);
    ext::Item_80266FA8();
    ext::Item_80266FCC();
    ext::Stage_8022524C();
    ext::Stage_8022532C(St_Kind_BigBlueRoute, 0);

    gobj = ext::GObj_Create(0xB, 3, 0);
    let fog = ext::HSD_FogLoadDesc((*(*un_804D6FB8).fogs).desc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_FogKind as u8, fog as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_FogCallback), 0, 0);
    un_804D6FBC = (*fog).color;

    let gobj2 = ext::GObj_Create(0xB, 3, 0);
    let lobj = ext::lb_80011AC4((*un_804D6FB8).lights);
    ext::HSD_GObjObject_80390A70(gobj2, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj2, Some(ext::HSD_GObj_LObjCallback), 0, 0);

    ext::grBigBlueRoute_8020DAB4(&raw mut un_804A2EA8 as *mut *mut HSD_JObj, 0.5, 0x17);

    ext::Player_InitAllPlayers();
    ext::lbAudioAx_80024E50(0);
}

pub unsafe extern "C" fn vi0801_Scene_OnFrame() {
    melee_core::ct!("vi0801_Scene_OnFrame");
    ext::vi_8031CAAC();
}
