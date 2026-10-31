//! Native port of src/melee/vi/vi0601.c: the Star Fox / Corneria opening vignette (a camera
//! flythrough of the stage with its Great Fox model parts). Every function of the TU is here; the
//! `vi.c` helpers are the sibling port [`super::vi`]. Other callees go through `ext` (an unset hook
//! panics "TODO: <name> is not ported"). `lbArchive_LoadSymbols` is variadic in C: the hook takes
//! the `(out, name)` pairs as a slice (the NULL terminator is the slice end).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]
use super::vi::{vi_8031C9B4, vi_8031CAAC};
use core::ffi::c_void;
use melee_types::inline::{HSD_JObjGetScaleX, HSD_JObjMtxIsDirty, HSD_JObjSetScaleX, HSD_JObjSetScaleY, HSD_JObjSetScaleZ, JObjDirty};
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
    fn Camera_GetBackgroundColor() -> GXColor;
    fn Camera_800310A0(a0: u8);
    fn Camera_Init(a0: i32) => unsafe { melee_world::tu::cmcamera::Camera_Init(a0) };
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
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
    fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1) };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    fn Item_80266FA8() => unsafe { crate::tu::item::Item_80266FA8() };
    fn Item_80266FCC() => unsafe { crate::tu::item::Item_80266FCC() };
    fn Player_InitAllPlayers();
    fn Stage_802251E8(a0: u32, a1: *mut i32) => unsafe { melee_world::tu::stage::Stage_802251E8(a0, a1) };
    fn Stage_8022524C() => unsafe { melee_world::tu::stage::Stage_8022524C() };
    fn Stage_8022532C(a0: u32, a1: i32) => unsafe { melee_world::tu::stage::Stage_8022532C(a0, a1) };
    fn efAsync_LoadSync(a0: i32);
    fn efLib_Init() => unsafe { melee_world::tu::eflib::efLib_Init() };
    fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
    fn gm_801A4B60();
    fn grCorneria_801E1BF0() -> *mut HSD_GObj => unsafe { melee_world::tu::grcorneria::grCorneria_801E1BF0() };
    fn lbAudioAx_800236DC() -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC() };
    fn lbAudioAx_80023F28(a0: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0) };
    fn lbAudioAx_80024E50(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024E50(a0 != 0) };
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
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_LObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
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

/// `JObjDirty` host: the shipping `HSD_JObjSetMtxDirty` is `if (!HSD_JObjMtxIsDirty(j))
/// HSD_JObjSetMtxDirtySub(j)`, and the sub is in the unported `jobj.c`.
struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, jobj: &mut HSD_JObj) {
        if !HSD_JObjMtxIsDirty(jobj) {
            ext::HSD_JObjSetMtxDirtySub(jobj);
        }
    }
    fn set_mtx_dirty_sub(&mut self, jobj: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

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

/* 4D6FB0 */
static mut un_804D6FB0: *mut SceneDesc = core::ptr::null_mut();
/* erase_colors_vi0601 */
static mut erase_colors_vi0601: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };

/// `void vi_8031E6EC(HSD_GObj*)`
pub unsafe extern "C" fn vi_8031E6EC(gobj: *mut HSD_GObj) {
    melee_core::ct!("vi_8031E6EC");
    ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
}

/// `static void vi0601_GObj_OnRender(HSD_GObj*, int)`
unsafe extern "C" fn vi0601_GObj_OnRender(gobj: *mut HSD_GObj, _code: i32) {
    let mut cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    if ext::HSD_CObjSetCurrent(cobj) != 0 {
        ext::HSD_SetEraseColor(erase_colors_vi0601.r, erase_colors_vi0601.g, erase_colors_vi0601.b, erase_colors_vi0601.a);
        cobj = (*gobj).hsd_obj as *mut HSD_CObj;
        ext::HSD_CObjEraseScreen(cobj, 1, 0, 1);
        ext::Camera_800310A0(2);
        (*gobj).gxlink_prios = 0x9;
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::Camera_800310A0(1);
        (*gobj).gxlink_prios = 0x8;
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::Camera_800310A0(0);
        (*gobj).gxlink_prios = 0x8;
        ext::HSD_GObj_80390ED0(gobj, 7);
        (*gobj).gxlink_prios = 0x8A1;
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_CObjEndCurrent();
    }
}

/// `void fn_8031E800(HSD_GObj*)`
pub unsafe extern "C" fn fn_8031E800(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8031E800");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let child: *mut HSD_JObj = if jobj.is_null() { core::ptr::null_mut() } else { (*jobj).child };

    let scale: f32 = 0.65f32 * HSD_JObjGetScaleX(&*child);
    HSD_JObjSetScaleX(&mut *child, scale, &mut Dirty);
    HSD_JObjSetScaleY(&mut *child, scale, &mut Dirty);
    HSD_JObjSetScaleZ(&mut *child, scale, &mut Dirty);
}

/// `void un_8031E9B8(void)`
pub unsafe fn un_8031E9B8() {
    melee_core::ct!("un_8031E9B8");
    let mut gobj = ext::GObj_Create(0xE, 0xF, 0);
    let mut jobj = ext::HSD_JObjLoadJoint((**(*un_804D6FB0).models).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::cb::HSD_GObj_JObjCallback), 0xB, 0);
    ext::gm_8016895C(jobj, *(*un_804D6FB0).models, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    ext::HSD_GObj_SetupProc(gobj, Some(vi_8031E6EC), 0x17);

    let mut i: i32 = 0;
    while !(*(*un_804D6FB0).models.add(i as usize)).is_null() {
        if ((i as u32).wrapping_sub(1)) <= 2 {
            gobj = ext::grCorneria_801E1BF0();
            let child: *mut HSD_JObj;
            if (*gobj).hsd_obj.is_null() {
                child = core::ptr::null_mut();
            } else {
                jobj = (*gobj).hsd_obj as *mut HSD_JObj;
                child = (*jobj).child;
            }
            ext::HSD_GObj_SetupProc(gobj, Some(fn_8031E800), 2);
            ext::gm_8016895C(child, *(*un_804D6FB0).models.add(i as usize), 0);
            ext::HSD_JObjReqAnimAll(child, 0.0);
            ext::HSD_JObjAnimAll(child);
        }
        i += 1;
    }

    ext::lbAudioAx_80026F2C(0x18);
    ext::lbAudioAx_8002702C(8, 8u64 << 32);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
}

/// `void vi0601_RunFrame(HSD_GObj*)`
pub unsafe extern "C" fn vi0601_RunFrame(gobj: *mut HSD_GObj) {
    melee_core::ct!("vi0601_RunFrame");
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    ext::HSD_CObjAnim(cobj);

    if 537.0f32 == (*(*cobj).aobj).curr_frame || 559.0f32 == (*(*cobj).aobj).curr_frame || 580.0f32 == (*(*cobj).aobj).curr_frame {
        vi_8031C9B4(0x23, 0);
    }
    if (*(*cobj).aobj).curr_frame == (*(*cobj).aobj).end_frame {
        ext::lb_800145F4();
        ext::gm_801A4B60();
    }
}

/// `void vi0601_Scene_OnEnter(void*)`
pub unsafe fn vi0601_Scene_OnEnter(enter_data: *mut c_void) {
    melee_core::ct!("vi0601_Scene_OnEnter");
    ext::lbAudioAx_800236DC();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);
    ext::lbAudioAx_80023F28(0x57);
    ext::lbAudioAx_80024E50(1);

    ext::lbArchive_LoadSymbols(cs!("Vi0601.dat"), &[&raw mut un_804D6FB0 as *mut c_void, cs!("visual0601Scene") as *mut c_void]);

    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::lb_80013B14(&raw mut (*(*(*un_804D6FB0).cameras).desc).perspective);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj, Some(vi0601_GObj_OnRender), 2);
    ext::HSD_CObjAddAnim(cobj, *(*(*un_804D6FB0).cameras).anims.add(0));
    ext::HSD_CObjReqAnim(cobj, 0.0);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(gobj, Some(vi0601_RunFrame), 0);

    Stage_InitScene(St_Kind_Corneria, core::ptr::null_mut());
    ext::Item_80266FA8();
    ext::Item_80266FCC();
    ext::Stage_8022524C();
    ext::Stage_8022532C(St_Kind_Corneria as u32, 0);

    erase_colors_vi0601 = ext::Camera_GetBackgroundColor();
    un_8031E9B8();

    let gobj2 = ext::GObj_Create(0xB, 3, 0);
    let lobj = ext::lb_80011AC4((*un_804D6FB0).lights);
    ext::HSD_GObjObject_80390A70(gobj2, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj2, Some(ext::cb::HSD_GObj_LObjCallback), 0, 0);

    ext::Player_InitAllPlayers();
    ext::lbAudioAx_80024E50(0);
}

/// `void vi0601_Scene_OnFrame(void)`
pub unsafe fn vi0601_Scene_OnFrame() {
    melee_core::ct!("vi0601_Scene_OnFrame");
    vi_8031CAAC();
}
