//! Port of src/melee/mn/mngallery.c: the "Gallery" page of the Records menu (movie player for
//! the two clips MvOmake15 / MvHowTo, its cursor and selection list, the movie camera).
//!
//! Callees live in unported TUs (HSD, lb, gm, mn) and are `ext` stubs named like the C. State is the
//! file's globals (`globals!`); tests/tu_mngallery.rs checks every function against the C compiled
//! natively.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};

use melee_types::{
    GObj_RenderFunc, HSD_AnimJoint, HSD_Archive, HSD_CObj, HSD_GObj, HSD_GObjEvent, HSD_GObjProc, HSD_JObj,
    HSD_MObj, HSD_MatAnimJoint, HSD_PadStatus, HSD_SObj, HSD_ShapeAnimJoint, HSD_TObj, HSD_Text, HSD_Joint,
    MenuFlow, MenuInputState, StaticModelDesc, Vec3, _HSD_RectS16, _Scissor, gmMainLib_8046B0F0_t,
};

use super::hsd::{jobj_child, jobj_next, sfxBack, sfxMove};
use super::mninl::{HSD_JObjGetTranslationX, HSD_JObjGetTranslationY, HSD_JObjGetTranslationZ, HSD_JObjSetTranslateX, HSD_JObjSetTranslateY, HSD_JObjSetTranslateZ};
use crate::oracle::Va;

/// `MenuInput_*` (mn/inlines.h).
const MENU_INPUT_UP: u64 = 1 << 0;
const MENU_INPUT_DOWN: u64 = 1 << 1;
const MENU_INPUT_BACK: u64 = 1 << 5;
const MENU_INPUT_START: u64 = 1 << 8;
const MENU_INPUT_A: u64 = 1 << 9;

pub mod ext {
    use super::*;
    crate::oext_fns! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void)>, data: *mut c_void) -> ();
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32) -> ();
        fn GObj_SetupGXLinkMax(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, priority: u32) -> ();
        fn HSD_CObjAlloc() -> *mut HSD_CObj = crate::tu::adapt::HSD_CObjAlloc__2bceb5;
        fn HSD_CObjEndCurrent() -> () = crate::tu::adapt::HSD_CObjEndCurrent__d69e71;
        fn HSD_CObjEraseScreen(cobj: *mut HSD_CObj, enable_color: i32, enable_alpha: i32, enable_depth: i32) -> () = crate::tu::adapt::HSD_CObjEraseScreen__c961ec;
        fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 = crate::tu::adapt::HSD_CObjSetCurrent__d18716;
        fn HSD_CObjSetEyePosition(cobj: *mut HSD_CObj, a1: *mut Vec3) -> ();
        fn HSD_CObjSetFar(cobj: *mut HSD_CObj, far: f32) -> () = crate::tu::adapt::HSD_CObjSetFar__5ef574;
        fn HSD_CObjSetInterest(cobj: *mut HSD_CObj, a1: *mut Vec3) -> ();
        fn HSD_CObjSetNear(cobj: *mut HSD_CObj, near: f32) -> () = crate::tu::adapt::HSD_CObjSetNear__29587b;
        fn HSD_CObjSetOrtho(a0: *mut HSD_CObj, top: f32, bottom: f32, left: f32, right: f32) -> () = crate::tu::adapt::HSD_CObjSetOrtho__9b799b;
        fn HSD_CObjSetProjectionType(a0: *mut HSD_CObj, a1: u32) -> ();
        fn HSD_CObjSetRoll(cobj: *mut HSD_CObj, a1: f32) -> ();
        fn HSD_CObjSetScissor(cobj: *mut HSD_CObj, a1: *mut _Scissor) -> () = crate::tu::adapt::HSD_CObjSetScissor__2d3b20;
        fn HSD_CObjSetViewport(cobj: *mut HSD_CObj, viewport: *mut _HSD_RectS16) -> () = crate::tu::adapt::HSD_CObjSetViewport__55aa72;
        fn HSD_Free(ptr: *mut c_void) -> () = crate::tu::adapt::HSD_Free__3ee311;
        fn HSD_GObjFree(a0: *mut HSD_GObj) -> ();
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void) -> ();
        fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32) -> () = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint) -> ();
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj) -> ();
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32) -> ();
        fn HSD_MObjAnim(mobj: *mut HSD_MObj) -> ();
        fn HSD_MObjReqAnim(mobj: *mut HSD_MObj, startframe: f32) -> ();
        fn HSD_MemAlloc(size: i32) -> *mut c_void = crate::tu::adapt::HSD_MemAlloc__c2b3bc;
        fn HSD_SObjLib_803A54EC(a0: *mut HSD_GObj, a1: i32) -> () = crate::tu::adapt::HSD_SObjLib_803A54EC__11e875;
        fn HSD_SetEraseColor(r: u8, g: u8, b: u8, a: u8) -> ();
        fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A5ACC__27fcb1;
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) -> () = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
        fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) -> () = crate::tu::adapt::HSD_SisLib_803A6368__2e8f0d;
        fn HSD_TObjAnim(tobj: *mut HSD_TObj) -> () = crate::tu::adapt::HSD_TObjAnim__0f2a76;
        fn HSD_TObjReqAnim(tobj: *mut HSD_TObj, startframe: f32) -> () = crate::tu::adapt::HSD_TObjReqAnim__41ec43;
        fn __assert(a0: *const i8, a1: u32, a2: *const i8) -> ();
        fn gmMainLib_8015ECB0() -> i32 = crate::tu::adapt::gmMainLib_8015ECB0__963013;
        fn gm_801ACC94() -> *mut c_void = crate::tu::adapt::gm_801ACC94__d71db4;
        fn lbAudioAx_800236DC() -> i32 = crate::tu::adapt::lbAudioAx_800236DC__5d9f3e;
        fn lbAudioAx_80023F28(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_80023F28__582fc2;
        fn lbAudioAx_80024E50(pause: i32) -> () = crate::tu::adapt::lbAudioAx_80024E50__461466;
        fn lbMthp_8001F410(filename: *const i8, rate_tbl: *mut u32, buf: *mut c_void, heap_size: u32, _loop: i32) -> () = crate::tu::adapt::lbMthp_8001F410__492048;
        fn lbMthp_8001F578() -> () = crate::tu::adapt::lbMthp_8001F578__cdae63;
        fn lbMthp_8001F604() -> i32 = crate::tu::adapt::lbMthp_8001F604__d79d45;
        fn lbMthp_8001F624(a0: *mut HSD_GObj, a1: i32, a2: i32) -> *mut HSD_SObj = crate::tu::adapt::lbMthp_8001F624__0c09ac;
        fn lbMthp_8001F67C(a0: *mut HSD_GObj, a1: i32) -> () = crate::tu::adapt::lbMthp_8001F67C__b1c012;
        fn lbMthp_8001F800() -> () = crate::tu::adapt::lbMthp_8001F800__fe22a6;
        fn lbAudioAx_80024030(a0: i32) -> () = crate::tu::adapt::lbAudioAx_80024030__36458a;
        fn mn_80229624(a0: u32) -> u32;
        fn mn_80229894(a0: i32, a1: u16, a2: i32) -> ();
        fn mn_8022BD6C() -> () = crate::tu::adapt::mn_8022BD6C__fc17af;
        fn mn_8022BD8C() -> () = crate::tu::adapt::mn_8022BD8C__db9576;
    }
    crate::ext_va! {
        /// os: `(fmt, ...)`
        fn OSReport(fmt: *const i8) va 0 -> ();
    }
    /// lbarchive.c, variadic in C: `lbArchive_LoadSections(archive, out0, name0, out1, name1, ..., 0)`.
    /// The recorder logs the two fixed arguments only (the names are literals in the caller).
    pub unsafe fn lbArchive_LoadSections(archive: *mut HSD_Archive, out0: *mut *mut c_void, _rest: &[*const c_void]) {
        use crate::oracle::{arg, ext_call, K};
        if ext_call("lbArchive_LoadSections", &[arg(archive), arg(out0)], K::V).is_some() {
            return;
        }
        panic!("TODO(ext) lbArchive_LoadSections: its TU is not ported")
    }
}

pub use melee_types::{mnGallery_804D6C88_userdata, mnGallery_child_userdata};
pub use crate::mn_ptr::float_order_helper;

const HEAP_SIZE: u32 = 0x271000;

crate::globals! {
    // ---- defined by this TU
    rw pub mnGallery_804A0BA0: StaticModelDesc = StaticModelDesc { joint: null_mut(), animjoint: null_mut(), matanim_joint: null_mut(), shapeanim_joint: null_mut() };
    rw pub mnGallery_804A0BB0: StaticModelDesc = StaticModelDesc { joint: null_mut(), animjoint: null_mut(), matanim_joint: null_mut(), shapeanim_joint: null_mut() };
    rw pub mnGallery_804D6C88: *mut HSD_GObj = null_mut();
    rw pub rate_table: *mut u32 = null_mut();
    // ---- defined by other TUs (placeholders until they are ported)
    /// mnmain.c
    rw pub mn_804A04F0: MenuFlow = unsafe { core::mem::zeroed() };
    /// mnmain.c
    rw pub mn_804D6BC8: MenuInputState = unsafe { core::mem::zeroed() };
    /// mnmain.c
    rw pub mn_804D6BB8: *mut HSD_Archive = null_mut();
    /// baselib/gobj.c
    rw pub HSD_GObj_804D783C: i32 = 0;
    /// baselib/gobj.c
    rw pub HSD_GObj_CameraKind: u8 = 0;
    /// baselib/gobjobject.c
    rw pub HSD_GObj_JObjKind: u8 = 0;
    /// baselib/controller.c
    rw pub HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
    /// gm/gmmain_lib.c
    rw pub gmMainLib_8046B0F0: gmMainLib_8046B0F0_t = unsafe { core::mem::zeroed() };
}

#[inline]
unsafe fn get_804d6c88(g: *mut HSD_GObj) -> *mut mnGallery_804D6C88_userdata {
    (*g).user_data.cast()
}

/// 258940: allocate the movie heap.
pub unsafe extern "C" fn mnGallery_80258940() {
    rate_table = ext::HSD_MemAlloc(HEAP_SIZE as i32).cast();
}

/// 25896C: render callback of the movie camera.
pub unsafe extern "C" fn mnGallery_8025896C(gobj: *mut HSD_GObj, render_pass: i32) {
    let data = mnGallery_804D6C88;
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    if (*get_804d6c88(data)).state == 3 {
        if ext::HSD_CObjSetCurrent(cobj) != 0 {
            ext::HSD_SetEraseColor(0, 0, 0, 0xFF);
            ext::HSD_CObjEraseScreen(cobj, 1, 0, 0);
            ext::HSD_CObjEndCurrent();
        }
        ext::HSD_SObjLib_803A54EC(gobj, render_pass);
    }
}

/// 258A08: the movie camera.
pub unsafe extern "C" fn mnGallery_80258A08(gobj: *mut HSD_GObj, width: u16, height: u16, priority: u32) {
    let roll = 0.0f32;
    let near_val = 0.0f32;
    let mut eye = Vec3 { x: 0.0, y: 0.0, z: 1.0 };
    let mut interest = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let far = 2.0f32;
    let bottom = (-(height as i32)) as f32;
    let right = width as f32;
    let top = 0.0f32;
    let left = 0.0f32;

    let mut viewport: _HSD_RectS16 = core::mem::zeroed();
    let mut scissor: _Scissor = core::mem::zeroed();
    viewport.xmin = 0;
    scissor.left = 0;
    viewport.xmax = width as _;
    scissor.right = width as _;
    viewport.ymin = 0;
    scissor.top = 0;
    viewport.ymax = height as _;
    scissor.bottom = height as _;

    let cobj = ext::HSD_CObjAlloc();
    ext::HSD_CObjSetProjectionType(cobj, 3);
    ext::HSD_CObjSetViewport(cobj, &mut viewport);
    ext::HSD_CObjSetScissor(cobj, &mut scissor);
    ext::HSD_CObjSetEyePosition(cobj, &mut eye);
    ext::HSD_CObjSetInterest(cobj, &mut interest);
    ext::HSD_CObjSetRoll(cobj, roll);
    ext::HSD_CObjSetNear(cobj, near_val);
    ext::HSD_CObjSetFar(cobj, far);
    ext::HSD_CObjSetOrtho(cobj, top, bottom, left, right);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_CameraKind, cobj.cast());
    ext::GObj_SetupGXLinkMax(gobj, Some(mnGallery_8025896C), priority);
}

/// 258BC4: start the selected movie.
pub unsafe fn mnGallery_80258BC4(data: *mut mnGallery_804D6C88_userdata) {
    let mode = (*data).unk15 as i32;
    if (*data).unk0 != 0 {
        return;
    }
    if !(0..2).contains(&mode) {
        return;
    }
    (*data).unk2 = mode as u8;
    let gobj = ext::GObj_Create(6, 7, 0x80);
    (*data).gobj8 = gobj;
    ext::GObj_SetupGXLink(gobj, Some(ext::lbMthp_8001F67C), 8, 0x80);
    if (*data).unk2 == 0 {
        let sobj = ext::lbMthp_8001F624(gobj, 0x1C0, 0x150);
        (*data).sobj = sobj;
        float_order_helper(&mut *sobj);
    } else if (*data).unk2 == 1 {
        let sobj = ext::lbMthp_8001F624(gobj, 0x280, 0x1E0);
        (*data).sobj = sobj;
        (*sobj).x10 = 0.0;
        (*sobj).x14 = 0.0;
    }
    match mode {
        0 => {
            ext::lbAudioAx_800236DC();
            ext::lbAudioAx_80023F28(0x52);
            ext::lbAudioAx_80024E50(1);
            ext::lbMthp_8001F410(c"MvOmake15.mth".as_ptr(), null_mut(), rate_table.cast(), HEAP_SIZE, 0);
            ext::lbAudioAx_80024E50(0);
        }
        1 => {
            ext::lbAudioAx_800236DC();
            ext::lbAudioAx_80023F28(0x24);
            ext::lbAudioAx_80024E50(1);
            let rt = ext::gm_801ACC94();
            ext::lbMthp_8001F410(c"MvHowTo.mth".as_ptr(), rt.cast(), rate_table.cast(), HEAP_SIZE, 0);
            ext::lbAudioAx_80024E50(0);
        }
        _ => {}
    }
    (*data).unk0 = 1;
}

/// 258D50: stop the movie.
pub unsafe fn mnGallery_80258D50(data: *mut mnGallery_804D6C88_userdata) {
    if (*data).unk0 != 0 {
        ext::lbMthp_8001F800();
        ext::lbAudioAx_800236DC();
        ext::lbAudioAx_80023F28(ext::gmMainLib_8015ECB0());

        (*data).unk0 = 0;
        (*data).unk1 = 0;

        if !(*data).gobj8.is_null() {
            ext::HSD_GObjFree((*data).gobj8);
            (*data).gobj8 = null_mut();
        }
    }
}

/// 258DBC: input while a movie plays.
pub unsafe fn mnGallery_80258DBC(_gobj: *mut HSD_GObj, data: *mut mnGallery_804D6C88_userdata) {
    let mut skip = 0u32;
    let mut buttons = HSD_PadCopyStatus[0].trigger | HSD_PadCopyStatus[1].trigger;
    buttons |= HSD_PadCopyStatus[2].trigger;
    buttons |= HSD_PadCopyStatus[3].trigger;
    ext::lbMthp_8001F578();
    if ext::lbMthp_8001F604() != 0 {
        (*data).unk1 = (*data).unk1.wrapping_add(1);
    }
    if gmMainLib_8046B0F0.xC != 0 && ext::lbMthp_8001F604() == 0 {
        skip = 1;
    }
    let pressed = buttons & 0x1300;
    if pressed != 0 || (*data).unk1 > 0x32 || skip != 0 {
        if pressed != 0 {
            sfxBack();
        }
        (*data).state = 1;
        ext::mn_8022BD8C();
        if (*data).unk0 != 0 {
            ext::lbMthp_8001F800();
            ext::lbAudioAx_800236DC();
            ext::lbAudioAx_80023F28(ext::gmMainLib_8015ECB0());
            (*data).unk0 = 0;
            (*data).unk1 = 0;
            if !(*data).gobj8.is_null() {
                ext::HSD_GObjFree((*data).gobj8);
                (*data).gobj8 = null_mut();
            }
        }
    }
}

pub unsafe fn fn_80258ED0_helper() {
    let data: *mut mnGallery_804D6C88_userdata = (*mnGallery_804D6C88).user_data.cast();
    if !(*data).gobj4.is_null() {
        ext::HSD_GObjFree((*data).gobj4);
    }
    if !(*data).gobj8.is_null() {
        ext::HSD_GObjFree((*data).gobj8);
    }
    (*data).gobj4 = null_mut();
    (*data).gobj8 = null_mut();
    mnGallery_80258D50(data);
}

/// 258ED0: the input proc.
pub unsafe extern "C" fn fn_80258ED0(gobj: *mut HSD_GObj) {
    let data = get_804d6c88(mnGallery_804D6C88);

    if mn_804D6BC8.cooldown != 0 {
        // Menu_DecrementAnimTimer
        mn_804D6BC8.cooldown = mn_804D6BC8.cooldown.wrapping_sub(1);
        mn_804D6BC8.x2 = 0;
        mn_804D6BC8.x4 = 0;
        return;
    }

    let state = (*data).state;
    if state == 3 {
        mnGallery_80258DBC(gobj, data);
        return;
    }
    if state == 2 {
        return;
    }

    // Menu_GetAllInputs
    mn_804A04F0.buttons = ext::mn_80229624(4) as _;
    let buttons = mn_804A04F0.buttons as u64;

    if buttons & MENU_INPUT_BACK != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0;
        ext::mn_80229894(5, 1, 3);
        ext::HSD_SisLib_803A5CC4((*data).text);
        fn_80258ED0_helper();
        (*data).state = 2;
        return;
    }

    if (*data).state == 1 {
        if buttons & MENU_INPUT_UP != 0 {
            if (*data).unk15 != 0 {
                sfxMove();
                (*data).unk15 = (*data).unk15.wrapping_sub(1);
                ext::HSD_SisLib_803A6368((*data).text, 0xC7);
            }
        } else if buttons & MENU_INPUT_DOWN != 0 {
            if (*data).unk15 < 1 {
                sfxMove();
                (*data).unk15 = (*data).unk15.wrapping_add(1);
                ext::HSD_SisLib_803A6368((*data).text, 0xC8);
            }
        } else if buttons & (MENU_INPUT_START | MENU_INPUT_A) != 0 {
            (*data).state = 3;
            ext::mn_8022BD6C();
            mnGallery_80258BC4(data);
        }
    }
}

pub unsafe fn fn_802590C4_inline(gobj: *mut HSD_GObj) {
    let tmp: *mut mnGallery_804D6C88_userdata = (*gobj).user_data.cast();
    ext::HSD_GObjFree(gobj);
    for i in 0..2 {
        ext::HSD_GObjFree((*tmp).gobjs[i]);
        (*tmp).gobjs[i] = null_mut();
    }
}

/// 2590C4: animation proc of the list model.
pub unsafe extern "C" fn fn_802590C4(gobj: *mut HSD_GObj) {
    let data: *mut mnGallery_804D6C88_userdata = (*gobj).user_data.cast();
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    if (*data).state == 3 {
        return;
    }

    if (*data).state == 0 {
        if (*data).frame < 0x13 {
            (*data).frame += 1;
        } else {
            (*data).state = 1;
        }
    } else if (*data).state == 2 {
        if (*data).frame < 0x1D {
            (*data).frame += 1;
        } else {
            (*data).state = 4;
            fn_802590C4_inline(gobj);
        }
    }

    ext::HSD_JObjReqAnimAll(jobj, (*data).frame as f32);
    ext::HSD_JObjAnimAll(jobj);
}

/// 2591BC: animation proc of a cursor.
pub unsafe extern "C" fn mnGallery_802591BC(gobj: *mut HSD_GObj) {
    let ud: *mut mnGallery_child_userdata = (*gobj).user_data.cast();
    let gallery = mnGallery_804D6C88;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let inner = get_804d6c88(gallery);
    let index = (*ud).index;

    let mut child = jobj_child((*(*ud).parent_gobj).hsd_obj as *mut HSD_JObj);
    for _ in 0..index {
        child = jobj_next(child);
    }

    if !child.is_null() {
        HSD_JObjSetTranslateX(jobj, HSD_JObjGetTranslationX(child));
        HSD_JObjSetTranslateY(jobj, HSD_JObjGetTranslationY(child));
        HSD_JObjSetTranslateZ(jobj, HSD_JObjGetTranslationZ(child));
    }

    let dobj = |j: *mut HSD_JObj| (*jobj_child(j)).u.dobj;
    if (*inner).unk15 as i32 == index {
        ext::HSD_MObjReqAnim((*dobj(jobj)).mobj, 1.0);
        ext::HSD_MObjAnim((*dobj(jobj)).mobj);
        ext::HSD_MObjReqAnim((*(*dobj(jobj)).next).mobj, 1.0);
        ext::HSD_MObjAnim((*(*dobj(jobj)).next).mobj);
    } else {
        ext::HSD_MObjReqAnim((*dobj(jobj)).mobj, 0.0);
        ext::HSD_MObjAnim((*dobj(jobj)).mobj);
        ext::HSD_MObjReqAnim((*(*dobj(jobj)).next).mobj, 0.0);
        ext::HSD_MObjAnim((*(*dobj(jobj)).next).mobj);
    }

    ext::HSD_TObjReqAnim((*(*dobj(jobj)).mobj).tobj, index as f32);
    ext::HSD_TObjAnim((*(*dobj(jobj)).mobj).tobj);
    ext::HSD_TObjReqAnim((*(*(*dobj(jobj)).next).mobj).tobj, index as f32);
    ext::HSD_TObjAnim((*(*(*dobj(jobj)).next).mobj).tobj);
}

/// 259604: clear the user data.
pub unsafe fn mnGallery_80259604(data: *mut mnGallery_804D6C88_userdata) {
    (*data).unk0 = 0;
    (*data).unk1 = 0;
    (*data).unk2 = 0;
    (*data).gobj4 = null_mut();
    (*data).gobj8 = null_mut();
    (*data).sobj = null_mut();
    (*data).text = null_mut();
    (*data).state = 0;
    (*data).unk15 = 0;
    (*data).frame = 0;
    (*data).gobjs[0] = null_mut();
    (*data).gobjs[1] = null_mut();
}

/// 25963C: the list model and the two cursors.
pub unsafe extern "C" fn mnGallery_8025963C() {
    let mut model = addr_of_mut!(mnGallery_804A0BA0);

    let gobj = ext::GObj_Create(6, 7, 0x80);
    mnGallery_804D6C88 = gobj;
    let mut jobj = ext::HSD_JObjLoadJoint((*model).joint);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 4, 0x80);
    ext::HSD_JObjAddAnimAll(jobj, (*model).animjoint, (*model).matanim_joint, (*model).shapeanim_joint);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);

    let user_data = ext::HSD_MemAlloc(core::mem::size_of::<mnGallery_804D6C88_userdata>() as i32) as *mut mnGallery_804D6C88_userdata;
    if user_data.is_null() {
        ext::OSReport(c"Can't get user_data.\n".as_ptr(), &[]);
        ext::__assert(c"mngallery.c".as_ptr(), 439, c"user_data".as_ptr());
    }

    mnGallery_80259604(user_data);
    ext::GObj_InitUserData(gobj, 0, Some(ext::HSD_Free), user_data.cast());
    let mut proc = ext::HSD_GObj_SetupProc(gobj, Some(fn_802590C4), 0);
    (*proc).set_flags_3(HSD_GObj_804D783C as u8);

    model = addr_of_mut!(mnGallery_804A0BB0);
    for i in 0..2 {
        let child_gobj = ext::GObj_Create(6, 7, 0x80);
        (*user_data).gobjs[i] = child_gobj;
        jobj = ext::HSD_JObjLoadJoint((*model).joint);
        ext::HSD_GObjObject_80390A70(child_gobj, HSD_GObj_JObjKind, jobj.cast());
        ext::GObj_SetupGXLink(child_gobj, Some(ext::HSD_GObj_JObjCallback), 4, 0x80);
        ext::HSD_JObjAddAnimAll(jobj, (*model).animjoint, (*model).matanim_joint, (*model).shapeanim_joint);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);

        let cursor_user_data = ext::HSD_MemAlloc(core::mem::size_of::<mnGallery_child_userdata>() as i32) as *mut mnGallery_child_userdata;
        if cursor_user_data.is_null() {
            ext::OSReport(c"Can't get user_data.\n".as_ptr(), &[]);
            ext::__assert(c"mngallery.c".as_ptr(), 459, c"cursor_user_data".as_ptr());
        }

        (*cursor_user_data).parent_gobj = gobj;
        (*cursor_user_data).index = i as i32;
        ext::GObj_InitUserData(child_gobj, 0, Some(ext::HSD_Free), cursor_user_data.cast());
        proc = ext::HSD_GObj_SetupProc(child_gobj, Some(mnGallery_802591BC), 0);
        (*proc).set_flags_3(HSD_GObj_804D783C as u8);
        mnGallery_802591BC(child_gobj);
    }
}

/// 259868: enter the screen.
pub unsafe extern "C" fn mnGallery_80259868() {
    mn_804D6BC8.cooldown = 5;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x1A;
    mn_804A04F0.hovered_selection = 0;
    let archive = mn_804D6BB8;

    ext::lbArchive_LoadSections(
        archive,
        addr_of_mut!(mnGallery_804A0BA0.joint).cast(),
        &[
            c"MenMainConGa_Top_joint".as_ptr().cast(),
            addr_of_mut!(mnGallery_804A0BA0.animjoint).cast(),
            c"MenMainConGa_Top_animjoint".as_ptr().cast(),
            addr_of_mut!(mnGallery_804A0BA0.matanim_joint).cast(),
            c"MenMainConGa_Top_matanim_joint".as_ptr().cast(),
            addr_of_mut!(mnGallery_804A0BA0.shapeanim_joint).cast(),
            c"MenMainConGa_Top_shapeanim_joint".as_ptr().cast(),
            addr_of_mut!(mnGallery_804A0BB0.joint).cast(),
            c"MenMainCursorGa_Top_joint".as_ptr().cast(),
            addr_of_mut!(mnGallery_804A0BB0.animjoint).cast(),
            c"MenMainCursorGa_Top_animjoint".as_ptr().cast(),
            addr_of_mut!(mnGallery_804A0BB0.matanim_joint).cast(),
            c"MenMainCursorGa_Top_matanim_joint".as_ptr().cast(),
            addr_of_mut!(mnGallery_804A0BB0.shapeanim_joint).cast(),
            c"MenMainCursorGa_Top_shapeanim_joint".as_ptr().cast(),
            null_mut::<c_void>(),
        ],
    );

    let gobj = ext::GObj_Create(0, 1, 0x80);
    let proc = ext::HSD_GObj_SetupProc(gobj, Some(fn_80258ED0), 0);
    (*proc).set_flags_3(HSD_GObj_804D783C as u8);
    mnGallery_8025963C();

    let inner = get_804d6c88(mnGallery_804D6C88);
    let text = ext::HSD_SisLib_803A5ACC(0, 1, -9.5, 9.1, 17.0, 364.68332, 38.38772);
    (*text).font_size.x = 0.0521;
    (*text).font_size.y = 0.0521;
    ext::HSD_SisLib_803A6368(text, 0xC7);
    (*inner).text = text;

    let cam_gobj = ext::GObj_Create(6, 3, 0x80);
    mnGallery_80258A08(cam_gobj, 0x280, 0x1E0, 1);
    (*cam_gobj).gxlink_prios = 0x100;
    (*inner).gobj4 = cam_gobj;
}
