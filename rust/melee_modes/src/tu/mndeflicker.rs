//! Port of src/melee/mn/mndeflicker.c: the "Deflicker" option screen of the Data menu (a model with
//! an on/off switch, its intro/exit animation procs and the input proc).
//!
//! Callees live in unported TUs (HSD, lb, gmMainLib, mn) and are `ext` stubs named like the C.
//! State is the file's globals (`globals!`), driven by the oracle through `oracle_globals()`
//! (checked by tests/tu_mndeflicker.rs against the C compiled natively).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use melee_types::{
    AnimLoopSettings, HSD_Archive, HSD_GObj, HSD_GObjEvent, HSD_GObjProc, HSD_JObj, HSD_MatAnimJoint, HSD_Joint,
    HSD_AnimJoint, HSD_ShapeAnimJoint, HSD_Text, HSD_TypeMask, Menu, MenuFlow, MenuInputState, StaticModelDesc,
    GObj_RenderFunc, HSD_GOBJ_CLASS_ITEM,
};

use super::hsd::{get_jobj, sfxBack, sfxMove};
use crate::oracle::Va;

/// `MenuInput_Back` / `MenuInput_AButton` (mn/inlines.h).
const MENU_INPUT_BACK: u64 = 1 << 5;
const MENU_INPUT_A: u64 = 1 << 9;

pub mod ext {
    use super::*;
    crate::oext_fns! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void)>, data: *mut c_void) -> ();
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32) -> ();
        fn HSD_Free(ptr: *mut c_void) -> ();
        fn HSD_GObjFree(a0: *mut HSD_GObj) -> ();
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void) -> ();
        fn HSD_GObjProc_RemoveProc(gproc: *mut HSD_GObjProc) -> ();
        fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32) -> ();
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint) -> ();
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj) -> ();
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj;
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32) -> ();
        fn HSD_MemAlloc(size: i32) -> *mut c_void;
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) -> ();
        fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, x: f32, y: f32, z: f32, w: f32, h: f32) -> *mut HSD_Text;
        fn HSD_SisLib_803A6368(text: *mut HSD_Text, idx: i32) -> ();
        fn __assert(a0: *const i8, a1: u32, a2: *const i8) -> ();
        fn gmMainLib_8015F4E8() -> u32;
        fn gmMainLib_8015F4F4(arg0: u8) -> ();
        fn gmMainLib_8015F588(a0: i32) -> ();
        fn lbCardGame_SaveChanges() -> ();
        fn mn_80229624(a0: u32) -> u32;
        fn mn_80229894(a0: i32, a1: u16, a2: i32) -> ();
        fn mn_8022EC18(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings, a2: HSD_TypeMask) -> f32;
        fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32;
        fn mn_8022F3D8(a0: *mut HSD_JObj, a1: u8, a2: HSD_TypeMask) -> ();
    }
    crate::ext_va! {
        /// os: `(fmt, ...)`
        fn OSReport(fmt: *const i8) va 0 -> ();
    }
    crate::ext_vk! {
        /// lbspdisplay: `(jobj, out, idx, -1)`
        fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj) vk "ii" -> i32;
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

crate::globals! {
    // ---- defined by this TU
    rw pub mnDeflicker_804D6C3C: u8 = 0;
    rw pub mnDeflicker_804D6C38: *mut HSD_GObj = null_mut();
    rw pub mnDeflicker_804A08B8: StaticModelDesc = StaticModelDesc { joint: null_mut(), animjoint: null_mut(), matanim_joint: null_mut(), shapeanim_joint: null_mut() };
    ro pub mnDeflicker_803EEFD8: AnimLoopSettings = AnimLoopSettings { start_frame: 0.0, end_frame: 19.0, loop_frame: -0.1 };
    ro pub mnDeflicker_803EEFE4: AnimLoopSettings = AnimLoopSettings { start_frame: 20.0, end_frame: 30.0, loop_frame: -0.1 };
    ro pub mnDeflicker_803EEFF0: AnimLoopSettings = AnimLoopSettings { start_frame: 50.0, end_frame: 350.0, loop_frame: 50.0 };
    // ---- defined by other TUs (placeholders until they are ported)
    /// mnmain.c
    rw pub mn_804A04F0: MenuFlow = unsafe { core::mem::zeroed() };
    /// mnmain.c
    rw pub mn_804D6BC8: MenuInputState = unsafe { core::mem::zeroed() };
    /// mnmain.c
    rw pub mn_804D6BB8: *mut HSD_Archive = null_mut();
    /// baselib/gobjproc.c
    rw pub HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = null_mut();
    /// baselib/gobj.c
    rw pub HSD_GObj_804D783C: i32 = 0;
    /// baselib/gobjobject.c
    rw pub HSD_GObj_JObjKind: u8 = 0;
}

/// mn/inlines.h `Menu_InitCenterText`.
unsafe fn Menu_InitCenterText(menu: *mut Menu, val: i32) {
    let text = ext::HSD_SisLib_803A5ACC(0, 1, -9.5, 9.1, 17.0, 364.68332, 38.38772);
    (*menu).text = text;
    (*text).font_size.x = 0.0521;
    (*text).font_size.y = 0.0521;
    ext::HSD_SisLib_803A6368(text, val);
}

/// `GET_MENU(gobj)`: `HSD_GObjGetUserData`.
#[inline]
unsafe fn get_menu(g: *mut HSD_GObj) -> *mut Menu {
    (*g).user_data.cast()
}

/// 24A168: the input proc of the screen.
pub unsafe extern "C" fn mnDeflicker_8024A168(_gobj: *mut HSD_GObj) {
    let mut asd: [*mut HSD_JObj; 2] = [null_mut(); 2];
    let menu = get_menu(mnDeflicker_804D6C38);
    if mn_804D6BC8.cooldown != 0 {
        // Menu_DecrementAnimTimer
        mn_804D6BC8.cooldown = mn_804D6BC8.cooldown.wrapping_sub(1);
        mn_804D6BC8.x2 = 0;
        mn_804D6BC8.x4 = 0;
        return;
    }
    // Menu_GetAllInputs
    mn_804A04F0.buttons = ext::mn_80229624(4) as u64;
    let events = mn_804A04F0.buttons;
    if events & MENU_INPUT_BACK != 0 {
        sfxBack();
        ext::gmMainLib_8015F4F4((*get_menu(mnDeflicker_804D6C38)).cursor);
        ext::lbCardGame_SaveChanges();
        mn_804A04F0.entering_menu = 0;
        ext::mn_80229894(4, 2, 3);
    } else if mnDeflicker_804D6C3C != 0 && events & MENU_INPUT_A != 0 {
        sfxMove();
        (*menu).cursor = ((*menu).cursor == 0) as u8;
        let x = (*menu).cursor as u32;
        ext::lb_80011E24((*mnDeflicker_804D6C38).hsd_obj.cast(), asd.as_mut_ptr(), &[Va::I32(5), Va::I32(-1)]);
        ext::HSD_JObjReqAnimAll(asd[0], x as f32);
        ext::mn_8022F3D8(asd[0], 0xFF, 0x20);
        ext::HSD_JObjAnimAll(asd[0]);
        ext::gmMainLib_8015F588(x as i32);
        ext::gmMainLib_8015F4F4((*get_menu(mnDeflicker_804D6C38)).cursor);
    }
}

/// 24A2E8: exit animation; frees the gobj when it has run out.
pub unsafe extern "C" fn mnDeflicker_8024A2E8(gobj: *mut HSD_GObj) {
    let f = ext::mn_8022EC18(get_jobj(gobj), (addr_of!(mnDeflicker_803EEFE4) as *mut AnimLoopSettings), 0x400);
    if f == mnDeflicker_803EEFE4.end_frame {
        ext::HSD_GObjFree(gobj);
    }
}

/// 24A344: idle proc (loops the switch animation).
pub unsafe extern "C" fn mnDeflicker_8024A344(gobj: *mut HSD_GObj) {
    let menu = get_menu(gobj);
    let jobj = get_jobj(gobj);
    if mn_804A04F0.cur_menu != 0x15 {
        ext::HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        let p = ext::HSD_GObj_SetupProc(gobj, Some(mnDeflicker_8024A2E8), 0);
        (*p).set_flags_3(HSD_GObj_804D783C as u8);
        ext::HSD_SisLib_803A5CC4((*menu).text);
    } else {
        let mut asd: *mut HSD_JObj = null_mut();
        ext::lb_80011E24(jobj, &mut asd, &[Va::I32(6), Va::I32(-1)]);
        ext::mn_8022ED6C(asd, (addr_of!(mnDeflicker_803EEFF0) as *mut AnimLoopSettings));
    }
}

/// 24A3E8: intro animation proc.
pub unsafe extern "C" fn mnDeflicker_8024A3E8(gobj: *mut HSD_GObj) {
    let menu = get_menu(gobj);
    let jobj = get_jobj(gobj);
    if mn_804A04F0.cur_menu != 0x15 {
        ext::HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        let p = ext::HSD_GObj_SetupProc(gobj, Some(mnDeflicker_8024A2E8), 0);
        (*p).set_flags_3(HSD_GObj_804D783C as u8);
        ext::HSD_SisLib_803A5CC4((*menu).text);
    } else {
        let f = ext::mn_8022EC18(jobj, (addr_of!(mnDeflicker_803EEFD8) as *mut AnimLoopSettings), 0x400);
        if f == mnDeflicker_803EEFD8.end_frame {
            ext::HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
            let p = ext::HSD_GObj_SetupProc(gobj, Some(mnDeflicker_8024A344), 0);
            (*p).set_flags_3(HSD_GObj_804D783C as u8);
            mnDeflicker_804D6C3C = 1;
        }
    }
}

/// `static inline mnDeflicker_8024A4BC_inline`: (re)creates the title text.
unsafe fn mnDeflicker_8024A4BC_inline() {
    let menu = get_menu(mnDeflicker_804D6C38);
    if !(*menu).text.is_null() {
        ext::HSD_SisLib_803A5CC4((*menu).text);
    }
    Menu_InitCenterText(menu, 0xBD);
}

/// 24A4BC: build the screen's gobj (model, animation, user data, intro proc).
pub unsafe extern "C" fn mnDeflicker_8024A4BC(_arg0: *mut HSD_GObj) {
    let mut sp1c: *mut HSD_JObj = null_mut();
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_ITEM as u16, 7, 0x80);
    mnDeflicker_804D6C38 = gobj;
    let jobj = ext::HSD_JObjLoadJoint(mnDeflicker_804A08B8.joint);
    let temp_r29 = HSD_GObj_JObjKind;
    ext::HSD_GObjObject_80390A70(gobj, temp_r29, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 4, 0x80);
    ext::HSD_JObjAddAnimAll(jobj, mnDeflicker_804A08B8.animjoint, mnDeflicker_804A08B8.matanim_joint, mnDeflicker_804A08B8.shapeanim_joint);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    let user_data = ext::HSD_MemAlloc(core::mem::size_of::<Menu>() as i32) as *mut Menu;
    if user_data.is_null() {
        ext::OSReport(c"Can't get user_data.\n".as_ptr(), &[]);
        ext::__assert(c"mndeflicker.c".as_ptr(), 344, c"user_data".as_ptr());
    }
    (*user_data).cursor = ext::gmMainLib_8015F4E8() as u8;
    (*user_data).text = null_mut();
    ext::GObj_InitUserData(gobj, 0, Some(ext::HSD_Free), user_data.cast());
    let proc = ext::HSD_GObj_SetupProc(gobj, Some(mnDeflicker_8024A3E8), 0);
    (*proc).set_flags_3(HSD_GObj_804D783C as u8);
    let temp_r29 = (*user_data).cursor;
    ext::lb_80011E24(get_jobj(gobj), &mut sp1c, &[Va::I32(5), Va::I32(-1)]);
    ext::HSD_JObjReqAnimAll(sp1c, temp_r29 as f32);
    ext::mn_8022F3D8(sp1c, 0xFF, 0x20);
    ext::HSD_JObjAnimAll(sp1c);
    ext::gmMainLib_8015F588(temp_r29 as i32);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::mn_8022F3D8(jobj, 0xFF, 0x400);
    ext::HSD_JObjAnimAll(jobj);
    mnDeflicker_8024A4BC_inline();
}

/// 24A6C4: enter the screen.
pub unsafe extern "C" fn mnDeflicker_8024A6C4(arg0: *mut HSD_GObj) {
    mn_804D6BC8.cooldown = 5;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x15;
    mn_804A04F0.hovered_selection = 0;
    mnDeflicker_804D6C3C = 0;
    let archive = mn_804D6BB8;
    ext::lbArchive_LoadSections(
        archive,
        addr_of_mut!(mnDeflicker_804A08B8.joint).cast(),
        &[
            c"MenMainConDf_Top_joint".as_ptr().cast(),
            addr_of_mut!(mnDeflicker_804A08B8.animjoint).cast(),
            c"MenMainConDf_Top_animjoint".as_ptr().cast(),
            addr_of_mut!(mnDeflicker_804A08B8.matanim_joint).cast(),
            c"MenMainConDf_Top_matanim_joint".as_ptr().cast(),
            addr_of_mut!(mnDeflicker_804A08B8.shapeanim_joint).cast(),
            c"MenMainConDf_Top_shapeanim_joint".as_ptr().cast(),
            null_mut::<c_void>(),
        ],
    );
    mnDeflicker_8024A4BC(arg0);
    let gobj = ext::GObj_Create(0, 1, 0x80);
    let temp_r3 = ext::HSD_GObj_SetupProc(gobj, Some(mnDeflicker_8024A168), 0);
    (*temp_r3).set_flags_3(HSD_GObj_804D783C as u8);
}
