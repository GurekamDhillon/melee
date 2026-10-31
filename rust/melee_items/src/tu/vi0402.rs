//! Native port of src/melee/vi/vi0402.c: an opening vignette made of a camera flythrough and
//! a model set (no fighters). Every function of the TU is here; the `vi.c` helpers are the sibling
//! port [`super::vi`]. Other callees go through `ext` (an unset hook panics "TODO: <name> is not
//! ported"). `lbArchive_LoadSymbols` is variadic in C: the hook takes the `(out, name)` pairs as a
//! slice (the NULL terminator is the slice end).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]
use super::vi::{vi_8031C9B4, vi_8031CAAC};
use core::ffi::c_void;
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
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn HSD_CObjAddAnim(a0: *mut HSD_CObj, a1: *mut HSD_CameraAnim) => unsafe { melee_runtime::tu::cobj::HSD_CObjAddAnim(a0, a1) };
    fn HSD_CObjAnim(a0: *mut HSD_CObj) => unsafe { melee_runtime::tu::cobj::HSD_CObjAnim(a0) };
    fn HSD_CObjReqAnim(a0: *mut HSD_CObj, a1: f32) => unsafe { melee_runtime::tu::cobj::HSD_CObjReqAnim(a0, a1) };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
    fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1) };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn Player_InitAllPlayers();
    fn efAsync_LoadSync(a0: i32);
    fn efLib_Init() => unsafe { melee_world::tu::eflib::efLib_Init() };
    fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
    fn gm_801A4B60();
    fn lbAudioAx_800236DC() -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC() };
    fn lbAudioAx_80023F28(a0: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0) };
    fn lbAudioAx_80024E50(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024E50(a0 != 0) };
    fn lbAudioAx_80026F2C(a0: u32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(a0) };
    fn lbAudioAx_8002702C(a0: u32, a1: u64) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(a0, a1) };
    fn lbAudioAx_80027168() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027168() };
    fn lbAudioAx_80027648() => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027648() };
    fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(a0) };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0) };
    fn lb_800145F4();
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_803910D8(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_LObjCallback(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_LightKind;
}
// END GENERATED EXT

macro_rules! cs {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *mut i8
    };
}

/* 4D6F68 */
static mut un_804D6F68: *mut SceneDesc = core::ptr::null_mut();

/// `void un_8031D6E4(HSD_GObj*)`
pub unsafe extern "C" fn un_8031D6E4(gobj: *mut HSD_GObj) {
    melee_core::ct!("un_8031D6E4");
    ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
}

/// `void un_8031D708(void)`
pub unsafe fn un_8031D708() {
    melee_core::ct!("un_8031D708");
    let mut i: usize = 0;
    while !(*(*un_804D6F68).models.add(i)).is_null() {
        let gobj = ext::GObj_Create(0xE, 0xF, 0);
        let jobj = ext::HSD_JObjLoadJoint((**(*un_804D6F68).models.add(i)).joint);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        ext::GObj_SetupGXLink(gobj, Some(ext::cb::HSD_GObj_JObjCallback), 3, 0);
        ext::gm_8016895C(jobj, *(*un_804D6F68).models.add(i), 0);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        ext::HSD_GObj_SetupProc(gobj, Some(un_8031D6E4), 0x17);
        i += 1;
    }

    ext::lbAudioAx_80026F2C(0x18);
    ext::lbAudioAx_8002702C(8, 0x10000000000u64);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
}

/// `void vi_8031D80C(HSD_GObj*)`
pub unsafe extern "C" fn vi_8031D80C(gobj: *mut HSD_GObj) {
    melee_core::ct!("vi_8031D80C");
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    ext::HSD_CObjAnim(cobj);
    let aobj = (*cobj).aobj;

    if (*aobj).curr_frame == (*aobj).end_frame {
        ext::lb_800145F4();
        ext::gm_801A4B60();
    }
}

/// `void vi0402_Scene_OnEnter(void*)`
pub unsafe fn vi0402_Scene_OnEnter(arg0: *mut c_void) {
    melee_core::ct!("vi0402_Scene_OnEnter");
    ext::lbAudioAx_800236DC();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);
    ext::lbAudioAx_80023F28(0x58);
    ext::lbAudioAx_80024E50(1);
    ext::lbArchive_LoadSymbols(cs!("Vi0402.dat"), &[&raw mut un_804D6F68 as *mut c_void, cs!("visual0402Scene") as *mut c_void]);

    let light_gobj = ext::GObj_Create(0xB, 0x3, 0);
    let lobj = ext::lb_80011AC4((*un_804D6F68).lights);
    ext::HSD_GObjObject_80390A70(light_gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(light_gobj, Some(ext::cb::HSD_GObj_LObjCallback), 0, 0);

    let cam_gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::lb_80013B14(&raw mut (*(*(*un_804D6F68).cameras.add(0)).desc).perspective);
    ext::HSD_GObjObject_80390A70(cam_gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(cam_gobj, Some(ext::cb::HSD_GObj_803910D8), 8);
    (*cam_gobj).gxlink_prios = 0x89;
    ext::HSD_CObjAddAnim(cobj, *(*(*un_804D6F68).cameras.add(0)).anims.add(0));
    ext::HSD_CObjReqAnim(cobj, 0.0);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(cam_gobj, Some(vi_8031D80C), 0);

    un_8031D708();
    vi_8031C9B4(0x22, 0);
    ext::Player_InitAllPlayers();
    ext::lbAudioAx_80024E50(0);
}

/// `void vi0402_Scene_OnFrame(void)`
pub unsafe fn vi0402_Scene_OnFrame() {
    melee_core::ct!("vi0402_Scene_OnFrame");
    vi_8031CAAC();
}
