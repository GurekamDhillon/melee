//! Port of src/melee/vi/vi1202.c: the "Vi1202" Bowser (Koopa R) trophy-stand intro scene.
//! Callees of unported TUs are TODO stubs in `ext`.
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
        fn gm_801A4B60();
        fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
        fn lb_8000C1C0(a0: *mut HSD_JObj, a1: *mut HSD_JObj) => unsafe { melee_runtime::tu::lb_00B0::lb_8000C1C0(a0, a1) };
        fn lb_8000C290(a0: *mut HSD_JObj, a1: *mut HSD_JObj) => unsafe { melee_runtime::tu::lb_00B0::lb_8000C290(a0, a1) };
        fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(a0) };
        fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: i32, a3: i32) -> i32;
        fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0) };
        fn lb_800145F4();
        fn lbAudioAx_80023694() => unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80023694(); };
        fn lbAudioAx_800236DC() => unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC(); };
        fn lbAudioAx_80023F28(a0: i32) => unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0); };
        fn lbAudioAx_80024E50(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024E50(a0 != 0) };
        fn lbAudioAx_80026F2C(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(a0 as _) };
        fn lbAudioAx_8002702C(a0: u32, a1: u64) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(a0, a1) };
        fn lbAudioAx_80027168() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027168() };
        fn lbAudioAx_80027648() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027648() };
        fn Toy_803060BC(a0: i32, a1: i32) -> f32 => unsafe { crate::tu::toy::Toy_803060BC(a0, a1) };
        fn Toy_803124BC() => unsafe { crate::tu::toy::Toy_803124BC() };
        fn vi_8031C9B4(a0: i32, a1: i32) => unsafe { crate::tu::vi::vi_8031C9B4(a0, a1) };
        fn vi_8031CAAC() => unsafe { crate::tu::vi::vi_8031CAAC() };
        fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
        fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8, a3: u32);
        fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u32);
        fn HSD_CObjAddAnim(a0: *mut HSD_CObj, a1: *mut HSD_CameraAnim) => unsafe { melee_runtime::tu::cobj::HSD_CObjAddAnim(a0, a1) };
        fn HSD_CObjAnim(a0: *mut HSD_CObj) => unsafe { melee_runtime::tu::cobj::HSD_CObjAnim(a0) };
        fn HSD_CObjReqAnim(a0: *mut HSD_CObj, a1: f32) => unsafe { melee_runtime::tu::cobj::HSD_CObjReqAnim(a0, a1) };
        fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32);
        fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
        fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1) };
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
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
    }
    /// Variadic in C: `(filename, (void** out, const char* name)..., NULL)`; `args` follows `filename`.
    pub unsafe fn lbArchive_LoadSymbols(filename: *const i8, args: &[*mut c_void]) -> *mut HSD_Archive {
        { return unsafe { melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(filename as _, args as _) as _ }; }
    }
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_LightKind;
}

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *mut c_void
    };
}

static mut un_804D7040: *mut SceneDesc = null_mut();
static mut un_804D7044: *mut SceneDesc = null_mut();
static mut un_804D7048: *mut HSD_Joint = null_mut();
static mut un_804D704C: *mut HSD_JObj = null_mut();

pub unsafe extern "C" fn un_8032110C(gobj: *mut HSD_GObj) {
    melee_core::ct!("un_8032110C");
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
}

pub unsafe extern "C" fn un_80321130(gobj: *mut HSD_GObj) {
    melee_core::ct!("un_80321130");
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
}

pub unsafe extern "C" fn un_80321154(gobj: *mut HSD_GObj) {
    melee_core::ct!("un_80321154");
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
}

unsafe fn vi1202_SetupChild(child: *mut HSD_JObj) {
    ext::HSD_JObjSetTranslateXWithMtxDirty(child, -ext::Toy_803060BC(0x1F, 0));
    ext::HSD_JObjSetTranslateYWithMtxDirty(child, -ext::Toy_803060BC(0x1F, 1));
    ext::HSD_JObjSetTranslateZWithMtxDirty(child, -ext::Toy_803060BC(0x1F, 2));
    ext::HSD_JObjSetRotationYWithMtxDirty(child, -ext::Toy_803060BC(0x1F, 5));

    let a = ext::Toy_803060BC(0x1F, 4);
    let b = 1.0f32 / ext::Toy_803060BC(0x1F, 3);
    let scale = 0.49f32 * (a * b);

    ext::HSD_JObjSetScaleXWithMtxDirty(child, scale);
    ext::HSD_JObjSetScaleYWithMtxDirty(child, scale);
    ext::HSD_JObjSetScaleZWithMtxDirty(child, scale);
}

pub unsafe extern "C" fn un_80321178() {
    melee_core::ct!("un_80321178");
    let mut i: isize = 0;
    while !(*(*un_804D7040).models.offset(i)).is_null() {
        let model = *(*un_804D7040).models.offset(i);
        let gobj = ext::GObj_Create(0xE, 0xF, 0);
        let jobj = ext::HSD_JObjLoadJoint((*model).joint);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
        ext::gm_8016895C(jobj, model, 0);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        ext::HSD_GObj_SetupProc(gobj, Some(un_80321154), 0x17);
        ext::lb_80011E24(jobj, &raw mut un_804D704C, 2, -1);
        i += 1;
    }
    ext::lbAudioAx_80026F2C(0x18);
    ext::lbAudioAx_8002702C(8, 0x0020000000000000u64);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
}

pub unsafe extern "C" fn vi1202_RunFrame(gobj: *mut HSD_GObj) {
    melee_core::ct!("vi1202_RunFrame");
    let cobj = GET_COBJ(gobj);
    ext::HSD_CObjAnim(cobj);
    if (*(*cobj).aobj).curr_frame == 100.0 {
        ext::vi_8031C9B4(0xE, 0);
    }
    if (*(*cobj).aobj).curr_frame == (*(*cobj).aobj).end_frame {
        ext::lb_800145F4();
        ext::gm_801A4B60();
    }
}

pub unsafe extern "C" fn vi1202_Scene_OnEnter(arg: *mut c_void) {
    melee_core::ct!("vi1202_Scene_OnEnter");
    ext::lbAudioAx_80023694();
    ext::lbAudioAx_800236DC();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);
    ext::lbAudioAx_80023F28(0x5A);
    ext::lbAudioAx_80024E50(1);

    ext::lbArchive_LoadSymbols(
        cstr!("Vi1202.dat") as *const i8,
        &[&raw mut un_804D7040 as *mut c_void, cstr!("visual1202Scene"), null_mut()],
    );
    ext::lbArchive_LoadSymbols(
        cstr!("TyKoopaR.dat") as *const i8,
        &[&raw mut un_804D7048 as *mut c_void, cstr!("ToyKoopaRModel_TopN_joint"), null_mut()],
    );
    ext::lbArchive_LoadSymbols(
        cstr!("GmRgStnd.dat") as *const i8,
        &[&raw mut un_804D7044 as *mut c_void, cstr!("standScene"), null_mut()],
    );
    ext::Toy_803124BC();

    let mut gobj = ext::GObj_Create(0xB, 0x3, 0);
    let lobj = ext::lb_80011AC4((*un_804D7040).lights);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_LObjCallback), 0, 0);

    let cam_gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::lb_80013B14(&raw mut (*(*(*un_804D7040).cameras).desc).perspective);
    ext::HSD_GObjObject_80390A70(cam_gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(cam_gobj, Some(ext::HSD_GObj_803910D8), 0x8);
    (*cam_gobj).gxlink_prios = 0x8A9;
    ext::HSD_CObjAddAnim(cobj, *(*(*un_804D7040).cameras).anims);
    ext::HSD_CObjReqAnim(cobj, 0.0);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(cam_gobj, Some(vi1202_RunFrame), 0);
    un_80321178();

    gobj = ext::GObj_Create(0xE, 0xF, 0);
    let mut jobj = ext::HSD_JObjLoadJoint(un_804D7048);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);

    ext::HSD_JObjSetScaleX(jobj, 0.49);
    ext::HSD_JObjSetScaleY(jobj, 0.49);
    ext::HSD_JObjSetScaleZ(jobj, 0.49);

    ext::lb_8000C1C0(jobj, un_804D704C);
    ext::lb_8000C290(jobj, un_804D704C);
    ext::HSD_GObj_SetupProc(gobj, Some(un_80321130), 0x17);

    gobj = ext::GObj_Create(0xE, 0xF, 0);
    jobj = ext::HSD_JObjLoadJoint((**(*un_804D7044).models).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(un_8032110C), 0x17);

    let child = if jobj.is_null() { null_mut() } else { (*jobj).child };

    vi1202_SetupChild(child);

    ext::lb_8000C1C0(jobj, un_804D704C);
    ext::lb_8000C290(jobj, un_804D704C);
    ext::lbAudioAx_80024E50(0);
}

pub unsafe extern "C" fn vi1202_Scene_OnFrame() {
    melee_core::ct!("vi1202_Scene_OnFrame");
    ext::vi_8031CAAC();
}
