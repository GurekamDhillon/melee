//! TARGET_PC port of src/melee/mn/mndatadel.c.
#![allow(non_snake_case, non_upper_case_globals, unused_imports, unused_variables, unused_assignments, unused_mut, unused_unsafe, static_mut_refs, clippy::all)]
use core::ffi::c_void;
use melee_types::*;
use std::vec::Vec;
use crate::oracle::Va;
use super::mninl::*;
use super::hsd::{sfxBack,sfxForward,sfxMove};
use melee_core::fp::fmadds;
pub mod ext { use super::*; crate::oext_fns! {
fn __assert(a0: *const i8, a1: u32, a2: *const i8) -> ();
fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32) -> () = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
fn lbAudioAx_800237A8(sfx_id: enum_t, sfx_vol: i32, sfx_pan: i32) -> i32 = crate::tu::adapt::lbAudioAx_800237A8__04460b;
fn lbAudioAx_80027AB0(a0: i32) -> () = crate::tu::adapt::lbAudioAx_80027AB0__27430d;
fn mn_80229624(a0: u32) -> u32 = crate::tu::adapt::mn_80229624__84b7ce;
fn mn_80229894(a0: i32, a1: u16, a2: i32) -> () = crate::tu::adapt::mn_80229894__f7df64;
fn mn_8022EC18(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings, a2: HSD_TypeMask) -> f32 = crate::tu::adapt::mn_8022EC18__bee2c1;
fn mn_8022EE84(arg0: *mut HSD_JObj, arg1: *mut AnimLoopSettings, arg2: HSD_TypeMask) -> f32 = crate::tu::adapt::mn_8022EE84__fdce2b;
fn mn_8022EFD8(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32 = crate::tu::adapt::mn_8022EFD8__c1f371;
fn mn_8022F298(a0: *mut HSD_JObj) -> f32 = crate::tu::adapt::mn_8022F298__0e9b78;
fn mn_8022F3D8(a0: *mut HSD_JObj, a1: u8, a2: HSD_TypeMask) -> () = crate::tu::adapt::mn_8022F3D8__9f7fac;
fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32) -> ();
fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint) -> ();
fn HSD_JObjAnimAll(a0: *mut HSD_JObj) -> ();
fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32) -> ();
fn HSD_JObjAddChild(jobj: *mut HSD_JObj, child: *mut HSD_JObj) -> ();
fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) -> () = crate::tu::adapt::HSD_SisLib_803A6368__2e8f0d;
fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A5ACC__27fcb1;
fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) -> () = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
fn mn_80231634(a0: *mut mn_80231634_t) -> isize; // jobj->child, pointer-wide (mnmainrule.rs)
fn gm_801603B0() -> () = crate::tu::adapt::gm_801603B0__bf3f82;
fn gm_801641CC(a0: u8) -> u16 = crate::tu::adapt::gm_801641CC__003261;
fn gm_801641E4(stage: u8, enable: u8) -> () = crate::tu::adapt::gm_801641E4__a151c2;
fn gm_IsStageUnlocked(stkind: u16) -> i32 = crate::tu::adapt::gm_IsStageUnlocked__19a514;
fn gm_80164430(a0: u16) -> i32 = crate::tu::adapt::gm_80164430__8808be;
fn gm_801647D0() -> () = crate::tu::adapt::gm_801647D0__084fbc;
fn gm_8016505C() -> () = crate::tu::adapt::gm_8016505C__87cdec;
fn gm_801729EC() -> () = crate::tu::adapt::gm_801729EC__181596;
fn gm_80174238() -> () = crate::tu::adapt::gm_80174238__25ce87;
fn gm_801A3EF4() -> () = crate::tu::adapt::gm_801A3EF4__94d282;
fn gmMainLib_8015DB80() -> () = crate::tu::adapt::gmMainLib_8015DB80__095b2c;
fn gmMainLib_8015EEC8() -> () = crate::tu::adapt::gmMainLib_8015EEC8__753a45;
fn gmMainLib_8015F150() -> () = crate::tu::adapt::gmMainLib_8015F150__b503e7;
fn gmMainLib_8015F260() -> () = crate::tu::adapt::gmMainLib_8015F260__097b6f;
fn gmMainLib_8015F464() -> () = crate::tu::adapt::gmMainLib_8015F464__37a6c0;
fn gmMainLib_8015F490() -> () = crate::tu::adapt::gmMainLib_8015F490__cf404e;
fn gmMainLib_8015F4BC() -> () = crate::tu::adapt::gmMainLib_8015F4BC__88898e;
fn gmMainLib_8015F4E8() -> u32 = crate::tu::adapt::gmMainLib_8015F4E8__7f030c;
fn gmMainLib_8015F588(a0: i32) -> () = crate::tu::adapt::gmMainLib_8015F588__d30fee;
fn gmMainLib_8015FBA4() -> () = crate::tu::adapt::gmMainLib_8015FBA4__bdb1fc;
fn lbCardGame_SaveChanges() -> () = crate::tu::adapt::lbCardGame_SaveChanges__0fc9e6;
fn lbLang_GetSavedLanguage() -> enum_t = crate::tu::adapt::lbLang_GetSavedLanguage__c4684f;
fn lbLang_SetSavedLanguage(language: enum_t) -> () = crate::tu::adapt::lbLang_SetSavedLanguage__3000cb;
fn lbLang_IsSavedLanguageUS() -> i32 = crate::tu::adapt::lbLang_IsSavedLanguageUS__90e41e;
fn Toy_80311960() -> () = crate::tu::adapt::Toy_80311960__db40d0;
fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32) -> ();
fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void) -> ();
fn HSD_GObjFree(a0: *mut HSD_GObj) -> ();
fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
fn HSD_GObjProc_RemoveProc(gproc: *mut HSD_GObjProc) -> ();
fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void) -> ()>, data: *mut c_void) -> ();
fn HSD_Free(ptr: *mut c_void) -> () = crate::tu::adapt::HSD_Free__3ee311;
fn HSD_MemAlloc(size: ssize_t) -> *mut c_void = crate::tu::adapt::HSD_MemAlloc__057bba;
}
pub use super::super::mncount::ext::lbArchive_LoadSections;
crate::ext_va! { fn OSReport(fmt: *const i8) va 0 -> (); fn HSD_SisLib_803A6B98(t: *mut HSD_Text, x:f32,y:f32,fmt:*const i8) va 3 -> i32; }
crate::ext_vk! { fn lb_80011E24(j:*mut HSD_JObj,out:*mut *mut HSD_JObj) vk "ii" -> i32; }
}
use ext::*;
// menuguard: the menu state is mnmain's and gobj.c's, not a private copy (a NULL mn_804D6BB8 faulted the Event menu)
pub use super::mnmain::{mn_804A04F0, mn_804D6BB8, mn_804D6BC8};
pub use melee_runtime::tu::gobj_native::{HSD_GObj_804D783C, HSD_GObj_JObjKind};
// menuguard: the menu state is mnmain's and gobj.c's, not a private copy (a NULL mn_804D6BB8 faulted the Event menu)
pub use melee_runtime::tu::gobj_native::{HSD_GObj_CurrentInvokedProc};
crate::globals! {
rw pub mnDataDel_803EF870: MnDataDelData  = MnDataDelData {
    x0: AnimLoopSettings {
        start_frame: 0 as i32 as f32,
        end_frame: 19 as i32 as f32,
        loop_frame: -0.1f64 as f32,
    },
    xC: AnimLoopSettings {
        start_frame: 20 as i32 as f32,
        end_frame: 29 as i32 as f32,
        loop_frame: -0.1f64 as f32,
    },
};
rw pub mnDataDel_803EF888: AnimLoopSettings  = AnimLoopSettings {
    start_frame: 0 as i32 as f32,
    end_frame: 5 as i32 as f32,
    loop_frame: -0.1f32,
};
rw pub mnDataDel_803EF894: AnimLoopSettings  = AnimLoopSettings {
    start_frame: 0 as i32 as f32,
    end_frame: 0 as i32 as f32,
    loop_frame: -0.1f32,
};
rw pub mnDataDel_803EF8A0: AnimLoopSettings  = AnimLoopSettings {
    start_frame: 0.0f32,
    end_frame: 9.0f32,
    loop_frame: -0.1f32,
};
rw pub mnDataDel_803EF8AC: [u32; 7]  = [
    1 as i32 as u32,
    2 as i32 as u32,
    3 as i32 as u32,
    4 as i32 as u32,
    5 as i32 as u32,
    6 as i32 as u32,
    7 as i32 as u32,
];
rw pub mnDataDel_803EF8C8: [u16; 6]  = [
    0xc0 as i32 as u16,
    0xc1 as i32 as u16,
    0xc2 as i32 as u16,
    0xc3 as i32 as u16,
    0xc4 as i32 as u16,
    0xc6 as i32 as u16,
];
rw pub lbl_803EF8D4: Vec3  = Vec3 {
    x: -5.5f32,
    y: -2.8f32,
    z: 23 as i32 as f32,
};
rw pub mnDataDel_PcAssets: [StaticModelDesc; 3]  = [StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
}; 3];
rw pub at_erase_plan: [[i8; 9]; 6]  = [
    [
        AT_ER_GM_8016505C as i32 as i8,
        AT_ER_MAINLIB_8015F464 as i32 as i8,
        AT_ER_GM_801729EC as i32 as i8,
        AT_ER_SAVE as i32 as i8,
        AT_ER_MAINLIB_8015DB80 as i32 as i8,
        -(1 as i32) as i8,
        0,
        0,
        0,
    ],
    [
        AT_ER_GM_801647D0 as i32 as i8,
        AT_ER_MAINLIB_8015F490 as i32 as i8,
        AT_ER_STAGE_RELOCK as i32 as i8,
        AT_ER_GM_801729EC as i32 as i8,
        AT_ER_SAVE as i32 as i8,
        -(1 as i32) as i8,
        0,
        0,
        0,
    ],
    [
        AT_ER_MAINLIB_8015EEC8 as i32 as i8,
        AT_ER_GM_801729EC as i32 as i8,
        AT_ER_SAVE as i32 as i8,
        -(1 as i32) as i8,
        0,
        0,
        0,
        0,
        0,
    ],
    [
        AT_ER_MAINLIB_8015F150 as i32 as i8,
        AT_ER_MAINLIB_8015F260 as i32 as i8,
        AT_ER_GM_801729EC as i32 as i8,
        AT_ER_SAVE as i32 as i8,
        -(1 as i32) as i8,
        0,
        0,
        0,
        0,
    ],
    [
        AT_ER_TOY_80311960 as i32 as i8,
        AT_ER_MAINLIB_8015F4BC as i32 as i8,
        AT_ER_GM_80174238 as i32 as i8,
        AT_ER_GM_801729EC as i32 as i8,
        AT_ER_SAVE as i32 as i8,
        -(1 as i32) as i8,
        0,
        0,
        0,
    ],
    [
        AT_ER_LANG_SAVE as i32 as i8,
        AT_ER_MAINLIB_8015FBA4 as i32 as i8,
        AT_ER_GM_801A3EF4 as i32 as i8,
        AT_ER_LANG_RESTORE as i32 as i8,
        AT_ER_GM_801603B0 as i32 as i8,
        AT_ER_DEFLICKER_APPLY as i32 as i8,
        AT_ER_GM_801729EC as i32 as i8,
        AT_ER_SAVE as i32 as i8,
        -(1 as i32) as i8,
    ],
];
rw pub mnDataDel_804D6C6C: *mut HSD_Text  = ::core::ptr::null::<HSD_Text>()
    as *mut HSD_Text;
rw pub mnDataDel_804D6C68: *mut HSD_GObj  = ::core::ptr::null::<HSD_GObj>()
    as *mut HSD_GObj;
rw pub mnDataDel_erase_lang: i32  = 0;
}
const AT_ER_DEFLICKER_APPLY: i32 = 19;
const AT_ER_GM_801603B0: i32 = 18;
const AT_ER_LANG_RESTORE: i32 = 17;
const AT_ER_GM_801A3EF4: i32 = 16;
const AT_ER_MAINLIB_8015FBA4: i32 = 15;
const AT_ER_LANG_SAVE: i32 = 14;
const AT_ER_GM_80174238: i32 = 13;
const AT_ER_MAINLIB_8015F4BC: i32 = 12;
const AT_ER_TOY_80311960: i32 = 11;
const AT_ER_MAINLIB_8015F260: i32 = 10;
const AT_ER_MAINLIB_8015F150: i32 = 9;
const AT_ER_MAINLIB_8015EEC8: i32 = 8;
const AT_ER_STAGE_RELOCK: i32 = 7;
const AT_ER_MAINLIB_8015F490: i32 = 6;
const AT_ER_GM_801647D0: i32 = 5;
const AT_ER_MAINLIB_8015DB80: i32 = 4;
const AT_ER_SAVE: i32 = 3;
const AT_ER_GM_801729EC: i32 = 2;
const AT_ER_MAINLIB_8015F464: i32 = 1;
const AT_ER_GM_8016505C: i32 = 0;
const AT_ER_COUNT: i32 = 20;
type AtEraseRec = Option<unsafe extern "C" fn(i32) -> ()>;
#[repr(C)] #[derive(Copy,Clone)] pub struct mn_80231634_t { pub pad_0:[i8;16], pub x10:i32 }
unsafe fn HSD_GObjGetHSDObj(g:*mut HSD_GObj)->*mut c_void { (*g).hsd_obj }
unsafe fn HSD_GObjGetUserData(g:*mut HSD_GObj)->*mut c_void { (*g).user_data }
unsafe fn Menu_GetAllInputs()->u64 { mn_804A04F0.buttons = mn_80229624(4) as u64; mn_804A04F0.buttons }
unsafe fn Menu_DecrementAnimTimer() { mn_804D6BC8.cooldown = mn_804D6BC8.cooldown.wrapping_sub(1); mn_804D6BC8.x2 = 0; mn_804D6BC8.x4 = 0; }
unsafe extern "C" fn at_erase_calls(
    mut category: i32,
) -> i32 {
    let mut n: i32 = 0 as i32;
    if category < 0 as i32 || category >= 6 as i32 {
        return 0 as i32;
    }
    while at_erase_plan[category as usize][n as usize] as i32
        >= 0 as i32
    {
        n += 1;
    }
    return n;
}
unsafe extern "C" fn at_erase_run(
    mut category: i32,
    mut confirmed: i32,
    mut rec: AtEraseRec,
) -> i32 {
    let mut n: i32 = 0;
    let mut k: i32 = 0;
    if confirmed != 1 as i32 || rec.is_none()
        || category < 0 as i32 || category >= 6 as i32
    {
        return 0 as i32;
    }
    n = at_erase_calls(category);
    k = 0 as i32;
    while k < n {
        rec
            .expect(
                "non-null function pointer",
            )(at_erase_plan[category as usize][k as usize] as i32);
        k += 1;
    }
    return n;
}
unsafe extern "C" fn mnDataDel_8024E940_inline(mut arg0: *mut HSD_JObj) -> f32 {
    return mn_8022F298(arg0 as *mut HSD_JObj);
}
pub unsafe extern "C" fn mnDataDel_8024E940() {
    melee_core::ct!("mnDataDel_8024E940");
    let mut pad1: u32 = 0;
    let mut pad2: u32 = 0;
    let mut pad3: u32 = 0;
    let mut sp18: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut tmp: i32 = 0;
    let mut tmp2: f32 = 0.;
    let mut temp_f31: f32 = 0.;
    let mut data: *mut i32 = ::core::ptr::null_mut::<i32>();
    let mut temp_ret: isize = 0;
    let mut temp_r31: *mut MnDataDelGObjUserData = ::core::ptr::null_mut::<
        MnDataDelGObjUserData,
    >();
    let mut i: i32 = 0;
    let mut var_r30: i32 = 0;
    let mut pad4: u32 = 0;
    temp_r31 = (*mnDataDel_804D6C68).user_data as *mut MnDataDelGObjUserData;
    temp_ret = mn_80231634(
        (*temp_r31).x10[mnDataDel_803EF8AC[1 as i32 as usize] as usize]
            as *mut mn_80231634_t,
    ) as isize;
    lb_80011E24(temp_ret as *mut HSD_JObj, &raw mut sp18, &[Va::I32(1 as i32), Va::I32(-(1 as i32))]);
    temp_f31 = mnDataDel_8024E940_inline(sp18);
    HSD_JObjReqAnimAll(sp18, 1.0f32);
    mn_8022F3D8(sp18 as *mut HSD_JObj, 0xff as u8, MOBJ_MASK);
    tmp2 = temp_f31;
    HSD_JObjAnimAll(sp18);
    tmp = 1152 as _HSD_TypeMask as i32;
    HSD_JObjReqAnimAll(sp18, tmp2);
    mn_8022F3D8(sp18 as *mut HSD_JObj, 0xff as u8, tmp as HSD_TypeMask);
    HSD_JObjAnimAll(sp18);
    (*temp_r31).x3[1 as i32 as usize] = 1 as u8;
    gm_801647D0();
    gmMainLib_8015F490();
    i = 0 as i32;
    var_r30 = i as i32;
    i = 0 as i32;
    while i < 0x1d as i32 {
        if gm_80164430(gm_801641CC(i as u8)) != 0 as i32
            && gm_IsStageUnlocked(i as u16) as i32
                & (0xffff as i32 & 0xff as i32)
                != 0 as i32
        {
            var_r30 = 1 as i32 as i32;
            break;
        } else {
            i += 1;
        }
    }
    if var_r30 == 0 as i32 {
        gm_801641E4(0 as u8, 1 as u8);
    }
    gm_801729EC();
    lbCardGame_SaveChanges();
}
unsafe extern "C" fn mnDataDel_8024EA6C_inline(
    mut jobj: *mut *mut HSD_JObj,
    mut frame: f32,
    mut start_frame: f32,
) {
    HSD_JObjReqAnimAll(*jobj, start_frame);
    mn_8022F3D8(*jobj, 0xff as u8, MOBJ_MASK);
    HSD_JObjAnimAll(*jobj);
    HSD_JObjReqAnimAll(*jobj, frame);
    mn_8022F3D8(*jobj, 0xff as u8, 1152 as HSD_TypeMask);
    HSD_JObjAnimAll(*jobj);
}
pub unsafe extern "C" fn mnDataDel_8024EA6C() {
    melee_core::ct!("mnDataDel_8024EA6C");
    let mut temp_f30: f32 = 0.;
    let mut temp_r31: *mut MnDataDelGObjUserData = ::core::ptr::null_mut::<
        MnDataDelGObjUserData,
    >();
    let mut data: *mut u32 = ::core::ptr::null_mut::<u32>();
    let mut lang: enum_t = 0;
    let mut i: i32 = 0;
    let mut temp_ret: isize = 0;
    let mut pad: [u8; 8] = [0; 8];
    let mut sp18: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut _pad: [u8; 16] = [0; 16];
    temp_f30 = 1.0f32 as f32;
    temp_r31 = (*mnDataDel_804D6C68).user_data as *mut MnDataDelGObjUserData;
    temp_ret = 0;
    i = temp_ret as i32;
    data = (&raw mut mnDataDel_803EF8AC as *mut u32).offset(i as isize) as *mut u32;
    loop {
        temp_ret = mn_80231634(
            (*((*mnDataDel_804D6C68).user_data as *mut MnDataDelGObjUserData))
                .x10[*data as usize] as *mut mn_80231634_t,
        ) as isize;
        lb_80011E24(temp_ret as *mut HSD_JObj, &raw mut sp18, &[Va::I32(1 as i32), Va::I32(-(1 as i32))]);
        mnDataDel_8024EA6C_inline(
            &raw mut sp18,
            mn_8022F298(sp18 as *mut HSD_JObj),
            temp_f30,
        );
        (*temp_r31).x3[i as usize] = 1 as u8;
        data = data.offset(1);
        i += 1;
        if !(i < 6 as i32) {
            break;
        }
    }
    lang = lbLang_GetSavedLanguage();
    gmMainLib_8015FBA4();
    gm_801A3EF4();
    if lang == lbLang_GetSavedLanguage() {
        lbAudioAx_800237A8(
            0xbf as enum_t,
            0x7f as i32,
            0x40 as i32,
        );
    } else {
        lbLang_SetSavedLanguage(lang);
        lbAudioAx_80027AB0(0xbf as i32);
    }
    gm_801603B0();
    gmMainLib_8015F588(gmMainLib_8015F4E8() as u8 as i32);
    gm_801729EC();
    lbCardGame_SaveChanges();
}
pub unsafe extern "C" fn mnDataDel_8024EBC8(
    mut root: *mut HSD_JObj,
    mut unused: u8,
    mut a: u8,
    mut b: u8,
) {
    melee_core::ct!("mnDataDel_8024EBC8");
    let mut j2: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut j1: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut frame: f32 = 0.;
    lb_80011E24(root as *mut HSD_JObj, &raw mut j1, &[Va::I32(WARN_JOINT_PANEL_TEXT_BOTTOM as i32), Va::I32(-(1 as i32))]);
    HSD_JObjReqAnimAll(j1, if a as i32 != 0 { 1.0f32 } else { 0.0f32 });
    mn_8022F3D8(j1 as *mut HSD_JObj, 0xff as u8, TOBJ_MASK);
    HSD_JObjAnimAll(j1);
    lb_80011E24(root as *mut HSD_JObj, &raw mut j2, &[Va::I32(WARN_JOINT_BACKGROUND as i32), Va::I32(-(1 as i32))]);
    if a != 0 {
        if b != 0 {
            frame = mnDataDel_803EF888.end_frame;
        } else {
            frame = mnDataDel_803EF888.start_frame;
        }
    } else if b != 0 {
        frame = mnDataDel_803EF894.end_frame;
    } else {
        frame = mnDataDel_803EF894.start_frame;
    }
    HSD_JObjReqAnimAll(j2, frame);
    mn_8022F3D8(j2 as *mut HSD_JObj, 0xff as u8, TOBJ_MASK);
    HSD_JObjAnimAll(j2);
}
unsafe extern "C" fn mnDataDel_GetWarnData() -> *mut WarnCmnData {
    return (*mnDataDel_804D6C68).user_data as *mut WarnCmnData;
}
unsafe extern "C" fn mnDataDel_AnimateWarning(
    mut root: *mut HSD_JObj,
    mut gobj: *mut HSD_GObj,
    mut data: *mut WarnCmnData,
    mut panel: *mut *mut HSD_JObj,
    mut exclaim: *mut *mut HSD_JObj,
) {
    let mut curr_frame: f32 = 0.;
    let mut sis_id: i32 = 0;
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut cursor_idx: u8 = 0;
    lb_80011E24(root as *mut HSD_JObj, panel as *mut *mut HSD_JObj, &[Va::I32(WARN_JOINT_PANEL as i32), Va::I32(-(1 as i32))]);
    if (*data).visible as i32 != 0 as i32 {
        curr_frame = mn_8022F298(root as *mut HSD_JObj);
        if mnDataDel_803EF8A0.start_frame <= curr_frame
            && curr_frame < mnDataDel_803EF8A0.end_frame
        {
            curr_frame = mn_8022EFD8(*panel, &raw mut mnDataDel_803EF8A0) as f32;
            lb_80011E24(root as *mut HSD_JObj, exclaim as *mut *mut HSD_JObj, &[Va::I32(WARN_JOINT_EXCLAIM as i32), Va::I32(-(1 as i32))]);
            mn_8022EFD8(*exclaim, &raw mut mnDataDel_803EF8A0);
            if curr_frame >= mnDataDel_803EF8A0.end_frame {
                let mut visible: u8 = (*data).visible;
                if !mnDataDel_804D6C6C.is_null() {
                    HSD_SisLib_803A5CC4(mnDataDel_804D6C6C);
                }
                if visible as i32 == 1 as i32 {
                    sis_id = 318 as i32 as i32;
                } else {
                    sis_id = 319 as i32 as i32;
                }
                text = HSD_SisLib_803A5ACC(
                    0 as i32,
                    1 as i32,
                    lbl_803EF8D4.x,
                    lbl_803EF8D4.y,
                    lbl_803EF8D4.z,
                    250.0f32,
                    5.0f32,
                );
                mnDataDel_804D6C6C = text;
                (*text).font_size.x = 0.05f32 as f32;
                (*text).font_size.y = 0.05f32 as f32;
                (*text).default_alignment = 1 as u8;
                HSD_SisLib_803A6368(text, sis_id);
            }
        } else {
            // cursor.slots[3] / slots[4] of the GC union are the `yes` / `no` fields.
            let mut cursor_yes: *mut HSD_JObj = ::core::ptr::null_mut();
            let mut cursor_no: *mut HSD_JObj = ::core::ptr::null_mut();
            cursor_idx = (*data).cursor_idx;
            lb_80011E24(root as *mut HSD_JObj, &raw mut cursor_yes, &[Va::I32(WARN_JOINT_CURSOR_YES as i32), Va::I32(-(1 as i32))]);
            lb_80011E24(root as *mut HSD_JObj, &raw mut cursor_no, &[Va::I32(WARN_JOINT_CURSOR_NO as i32), Va::I32(-(1 as i32))]);
            if cursor_idx as i32 != 0 as i32 {
                HSD_JObjReqAnimAll(cursor_yes, 1.0f32);
                HSD_JObjReqAnimAll(cursor_no, 0.0f32);
            } else {
                HSD_JObjReqAnimAll(cursor_yes, 0.0f32);
                HSD_JObjReqAnimAll(cursor_no, 1.0f32);
            }
            HSD_JObjAnimAll(cursor_yes);
            HSD_JObjAnimAll(cursor_no);
        }
    } else {
        HSD_SisLib_803A5CC4(mnDataDel_804D6C6C);
        mnDataDel_804D6C6C = ::core::ptr::null_mut::<HSD_Text>();
        HSD_GObjFree(gobj);
    };
}
pub unsafe extern "C" fn fn_8024ECCC(mut arg0: *mut HSD_GObj) {
    melee_core::ct!("fn_8024ECCC");
    let mut panel: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut exclaim: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut data: *mut WarnCmnData = ::core::ptr::null_mut::<WarnCmnData>();
    let mut root: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut curr_frame: f32 = 0.;
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut sis_id: i32 = 0;
    let mut _pad: [u8; 4] = [0; 4];
    root = HSD_GObjGetHSDObj(arg0) as *mut HSD_JObj;
    data = mnDataDel_GetWarnData();
    mnDataDel_AnimateWarning(root, arg0, data, &raw mut panel, &raw mut exclaim);
}
pub unsafe extern "C" fn mnDataDel_8024EEC0() {
    melee_core::ct!("mnDataDel_8024EEC0");
    let mut child: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut cursor_yes: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut cursor_no: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut wrn_modal: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut root: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut cursor: i32 = 0;
    let mut assets: *mut StaticModelDesc = ::core::ptr::null_mut::<StaticModelDesc>();
    let mut data: *mut WarnCmnData = ::core::ptr::null_mut::<WarnCmnData>();
    let mut _pad: [u8; 8] = [0; 8];
    assets = (&raw mut mnDataDel_PcAssets as *mut StaticModelDesc)
        .offset(2 as i32 as isize) as *mut StaticModelDesc;
    data = mnDataDel_GetWarnData();
    wrn_modal = GObj_Create(6 as u16, 7 as u8, 0x80 as u8);
    root = HSD_JObjLoadJoint((*assets).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(
        wrn_modal,
        HSD_GObj_JObjKind,
        root as *mut c_void,
    );
    GObj_SetupGXLink(
        wrn_modal,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, i32) -> (),
        ),
        6 as u8,
        0x80 as u32,
    );
    HSD_GObj_SetupProc(
        wrn_modal,
        Some(fn_8024ECCC as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
    HSD_JObjAddAnimAll(
        root,
        (*assets).animjoint,
        (*assets).matanim_joint,
        (*assets).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(root, mnDataDel_803EF8A0.start_frame);
    HSD_JObjAnimAll(root);
    lb_80011E24(root as *mut HSD_JObj, &raw mut child, &[Va::I32(WARN_JOINT_EXCLAIM_ARROW as i32), Va::I32(-(1 as i32))]);
    HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32);
    lb_80011E24(root as *mut HSD_JObj, &raw mut child, &[Va::I32(WARN_JOINT_PROGRESS_BAR as i32), Va::I32(-(1 as i32))]);
    HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32);
    cursor = ((*data).cursor_idx as i32 & 0xff as i32)
        as i32;
    let mut no: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut yes: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    lb_80011E24(root as *mut HSD_JObj, &raw mut yes, &[Va::I32(WARN_JOINT_CURSOR_YES as i32), Va::I32(-(1 as i32))]);
    lb_80011E24(root as *mut HSD_JObj, &raw mut no, &[Va::I32(WARN_JOINT_CURSOR_NO as i32), Va::I32(-(1 as i32))]);
    if cursor != 0 as i32 {
        HSD_JObjReqAnimAll(yes, 1.0f32);
        HSD_JObjReqAnimAll(no, 0.0f32);
    } else {
        HSD_JObjReqAnimAll(yes, 0.0f32);
        HSD_JObjReqAnimAll(no, 1.0f32);
    }
    HSD_JObjAnimAll(yes);
    HSD_JObjAnimAll(no);
    if lbLang_IsSavedLanguageUS() != 0 {
        lb_80011E24(root as *mut HSD_JObj, &raw mut cursor_yes, &[Va::I32(WARN_JOINT_CURSOR_YES as i32), Va::I32(-(1 as i32))]);
        lb_80011E24(root as *mut HSD_JObj, &raw mut cursor_no, &[Va::I32(WARN_JOINT_CURSOR_NO as i32), Va::I32(-(1 as i32))]);
        let mut yes_pos: f32 = HSD_JObjGetTranslationX(cursor_yes);
        let mut no_pos: f32 = HSD_JObjGetTranslationX(cursor_no);
        HSD_JObjSetTranslateX(cursor_yes, no_pos);
        HSD_JObjSetTranslateX(cursor_no, yes_pos);
    }
}
pub unsafe extern "C" fn fn_8024F1D4(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8024F1D4");
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut data: *mut WarnCmnData = ::core::ptr::null_mut::<WarnCmnData>();
    let mut input: u32 = 0;
    let mut _pad: [u8; 16] = [0; 16];
    data = (*mnDataDel_804D6C68).user_data as *mut WarnCmnData;
    input = Menu_GetAllInputs() as u32;
    if input & MenuInput_Confirm as i32 as u32 != 0 {
        if (*data).cursor_idx as i32 != 0 as i32 {
            mnDataDel_8024EA6C();
        } else {
            sfxBack();
        }
        mn_804D6BC8.cooldown = 5 as u16;
        (*data).visible = 0 as u8;
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(fn_8024F840 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
    } else if input & MenuInput_Back as i32 as u32 != 0 {
        sfxBack();
        mn_804D6BC8.cooldown = 5 as u16;
        (*data).visible = 0 as u8;
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(fn_8024F840 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
    } else if input & 8 as u32 != 0 || input & 4 as u32 != 0 {
        sfxMove();
        if (*data).cursor_idx as i32 != 0 as i32 {
            (*data).cursor_idx = 0 as u8;
        } else {
            (*data).cursor_idx = 1 as u8;
        }
    }
}
unsafe extern "C" fn mnDataDel_GetMenuJObj(
    mut data: *mut MnDataDelGObjUserData,
    mut idx: i32,
) -> *mut mn_80231634_t {
    return (*data).x10[mnDataDel_803EF8AC[idx as usize] as usize] as *mut mn_80231634_t;
}
pub unsafe extern "C" fn fn_8024F318(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8024F318");
    let mut _pad0: [u8; 44] = [0; 44];
    let mut sp68: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut _pad1: u32 = 0;
    let mut sp60: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut _pad2: u32 = 0;
    let mut sp58: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut _pad3: u32 = 0;
    let mut sp50: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut temp_gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut tmp2: f32 = 0.;
    let mut frame: f32 = 0.;
    let mut temp_ret: isize = 0;
    let mut sis_id: i32 = 0;
    let mut buttons: u32 = 0;
    let mut cursor: u8 = 0;
    let mut user_data: *mut MnDataDelGObjUserData = ::core::ptr::null_mut::<
        MnDataDelGObjUserData,
    >();
    let mut case_user_data: *mut MnDataDelGObjUserData = ::core::ptr::null_mut::<
        MnDataDelGObjUserData,
    >();
    let mut _pad: [u8; 48] = [0; 48];
    temp_gobj = gobj;
    user_data = (*mnDataDel_804D6C68).user_data as *mut MnDataDelGObjUserData;
    buttons = mn_80229624(4 as u32);
    mn_804A04F0.buttons = buttons as u64;
    if buttons & 0x10 as u32 != 0 {
        mn_804D6BC8.cooldown = 5 as u16;
        cursor = (*user_data).x0;
        if cursor as i32 == 5 as i32 {
            HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
            if (*user_data).x2 as i32 != 0 as i32 {
                (*user_data).x1 = 2 as u8;
                (*user_data).x2 = 0 as u8;
                proc_0 = HSD_GObj_SetupProc(
                    temp_gobj,
                    Some(fn_8024F1D4 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
                    0 as u8,
                );
                (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
                cursor = (*user_data).x1;
                if !mnDataDel_804D6C6C.is_null() {
                    HSD_SisLib_803A5CC4(mnDataDel_804D6C6C);
                }
                if cursor as i32 == 1 as i32 {
                    sis_id = 0x13e as i32 as i32;
                } else {
                    sis_id = 0x13f as i32 as i32;
                }
                text = HSD_SisLib_803A5ACC(
                    0 as i32,
                    1 as i32,
                    lbl_803EF8D4.x,
                    lbl_803EF8D4.y,
                    lbl_803EF8D4.z,
                    250.0f32,
                    5.0f32,
                );
                mnDataDel_804D6C6C = text;
                (*text).font_size.x = 0.05f32 as f32;
                (*text).font_size.y = 0.05f32 as f32;
                (*text).default_alignment = 1 as u8;
                HSD_SisLib_803A6368(text, sis_id);
                mn_804D6BC8.cooldown = 0xa as u16;
                lbAudioAx_800237A8(
                    0xbd as enum_t,
                    0x7f as i32,
                    0x40 as i32,
                );
                return;
            }
            sfxBack();
            (*user_data).x1 = 0 as u8;
            (*user_data).x2 = 0 as u8;
            proc_0 = HSD_GObj_SetupProc(
                temp_gobj,
                Some(fn_8024F840 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
                0 as u8,
            );
            (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
            return;
        }
        if (*user_data).x2 as i32 != 0 as i32 {
            match cursor as i32 {
                0 => {
                    case_user_data = (*mnDataDel_804D6C68).user_data
                        as *mut MnDataDelGObjUserData;
                    temp_ret = mn_80231634(
                        mnDataDel_GetMenuJObj(case_user_data, 0 as i32),
                    ) as isize;
                    lb_80011E24(temp_ret as *mut HSD_JObj, &raw mut sp68, &[Va::I32(1 as i32), Va::I32(-(1 as i32))]);
                    frame = mnDataDel_8024E940_inline(sp68);
                    HSD_JObjReqAnimAll(sp68, 1.0f32);
                    mn_8022F3D8(sp68 as *mut HSD_JObj, 0xff as u8, MOBJ_MASK);
                    tmp2 = frame;
                    HSD_JObjAnimAll(sp68);
                    HSD_JObjReqAnimAll(sp68, tmp2);
                    mn_8022F3D8(
                        sp68 as *mut HSD_JObj,
                        0xff as u8,
                        1152 as HSD_TypeMask,
                    );
                    HSD_JObjAnimAll(sp68);
                    (*case_user_data).x3[0 as i32 as usize] = 1 as u8;
                    gm_8016505C();
                    gmMainLib_8015F464();
                    gm_801729EC();
                    lbCardGame_SaveChanges();
                    gmMainLib_8015DB80();
                }
                1 => {
                    mnDataDel_8024E940();
                }
                2 => {
                    case_user_data = (*mnDataDel_804D6C68).user_data
                        as *mut MnDataDelGObjUserData;
                    temp_ret = mn_80231634(
                        mnDataDel_GetMenuJObj(case_user_data, 2 as i32),
                    ) as isize;
                    lb_80011E24(temp_ret as *mut HSD_JObj, &raw mut sp60, &[Va::I32(1 as i32), Va::I32(-(1 as i32))]);
                    frame = mnDataDel_8024E940_inline(sp60);
                    HSD_JObjReqAnimAll(sp60, 1.0f32);
                    mn_8022F3D8(sp60 as *mut HSD_JObj, 0xff as u8, MOBJ_MASK);
                    tmp2 = frame;
                    HSD_JObjAnimAll(sp60);
                    HSD_JObjReqAnimAll(sp60, tmp2);
                    mn_8022F3D8(
                        sp60 as *mut HSD_JObj,
                        0xff as u8,
                        1152 as HSD_TypeMask,
                    );
                    HSD_JObjAnimAll(sp60);
                    (*case_user_data).x3[2 as i32 as usize] = 1 as u8;
                    gmMainLib_8015EEC8();
                    gm_801729EC();
                    lbCardGame_SaveChanges();
                }
                3 => {
                    case_user_data = (*mnDataDel_804D6C68).user_data
                        as *mut MnDataDelGObjUserData;
                    temp_ret = mn_80231634(
                        mnDataDel_GetMenuJObj(case_user_data, 3 as i32),
                    ) as isize;
                    lb_80011E24(temp_ret as *mut HSD_JObj, &raw mut sp58, &[Va::I32(1 as i32), Va::I32(-(1 as i32))]);
                    frame = mnDataDel_8024E940_inline(sp58);
                    HSD_JObjReqAnimAll(sp58, 1.0f32);
                    mn_8022F3D8(sp58 as *mut HSD_JObj, 0xff as u8, MOBJ_MASK);
                    tmp2 = frame;
                    HSD_JObjAnimAll(sp58);
                    HSD_JObjReqAnimAll(sp58, tmp2);
                    mn_8022F3D8(
                        sp58 as *mut HSD_JObj,
                        0xff as u8,
                        1152 as HSD_TypeMask,
                    );
                    HSD_JObjAnimAll(sp58);
                    (*case_user_data).x3[3 as i32 as usize] = 1 as u8;
                    gmMainLib_8015F150();
                    gmMainLib_8015F260();
                    gm_801729EC();
                    lbCardGame_SaveChanges();
                }
                4 => {
                    case_user_data = (*mnDataDel_804D6C68).user_data
                        as *mut MnDataDelGObjUserData;
                    temp_ret = mn_80231634(
                        mnDataDel_GetMenuJObj(case_user_data, 4 as i32),
                    ) as isize;
                    lb_80011E24(temp_ret as *mut HSD_JObj, &raw mut sp50, &[Va::I32(1 as i32), Va::I32(-(1 as i32))]);
                    frame = mnDataDel_8024E940_inline(sp50);
                    HSD_JObjReqAnimAll(sp50, 1.0f32);
                    mn_8022F3D8(sp50 as *mut HSD_JObj, 0xff as u8, MOBJ_MASK);
                    tmp2 = frame;
                    HSD_JObjAnimAll(sp50);
                    HSD_JObjReqAnimAll(sp50, tmp2);
                    mn_8022F3D8(
                        sp50 as *mut HSD_JObj,
                        0xff as u8,
                        1152 as HSD_TypeMask,
                    );
                    HSD_JObjAnimAll(sp50);
                    (*case_user_data).x3[4 as i32 as usize] = 1 as u8;
                    Toy_80311960();
                    gmMainLib_8015F4BC();
                    gm_80174238();
                    gm_801729EC();
                    lbCardGame_SaveChanges();
                }
                _ => {}
            }
            lbAudioAx_800237A8(
                0xbe as enum_t,
                0x7f as i32,
                0x40 as i32,
            );
        } else {
            sfxBack();
        }
        (*user_data).x1 = 0 as u8;
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            temp_gobj,
            Some(fn_8024F840 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
        return;
    }
    if buttons & 0x20 as u32 != 0 {
        sfxBack();
        mn_804D6BC8.cooldown = 5 as u16;
        (*user_data).x1 = 0 as u8;
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            temp_gobj,
            Some(fn_8024F840 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
        return;
    }
    if buttons & 8 as u32 != 0 || buttons & 4 as u32 != 0 {
        sfxMove();
        if (*user_data).x2 as i32 != 0 as i32 {
            (*user_data).x2 = 0 as u8;
            return;
        }
        (*user_data).x2 = 1 as u8;
    }
}
unsafe extern "C" fn fn_8024F840_inline(
    mut buttons: u64,
    mut user_data: *mut *mut MnDataDelGObjUserData,
) {
    let mut joint: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut sis_id: u16 = 0;
    let mut cursor: u8 = 0;
    let mut temp_user_data: *mut MnDataDelGObjUserData = ::core::ptr::null_mut::<
        MnDataDelGObjUserData,
    >();
    if buttons & 2 as u64 != 0 {
        sfxMove();
        temp_user_data = (*mnDataDel_804D6C68).user_data as *mut MnDataDelGObjUserData;
        joint = mn_80231634(
            (*temp_user_data).x10[mnDataDel_803EF8AC[(**user_data).x0 as usize] as usize]
                as *mut mn_80231634_t,
        ) as *mut HSD_JObj;
        mnDataDel_8024EBC8(joint, (**user_data).x0, 0 as u8, 0 as u8);
        cursor = (**user_data).x0;
        if cursor as i32 == 5 as i32 {
            (**user_data).x0 = 0 as u8;
        } else {
            (**user_data).x0 = (cursor as i32 + 1 as i32)
                as u8;
        }
        temp_user_data = (*mnDataDel_804D6C68).user_data as *mut MnDataDelGObjUserData;
        joint = mn_80231634(
            (*temp_user_data).x10[mnDataDel_803EF8AC[(**user_data).x0 as usize] as usize]
                as *mut mn_80231634_t,
        ) as *mut HSD_JObj;
        mnDataDel_8024EBC8(joint, (**user_data).x0, 1 as u8, 0 as u8);
        *user_data = (*mnDataDel_804D6C68).user_data as *mut MnDataDelGObjUserData;
        if !(**user_data).xC.is_null() {
            HSD_SisLib_803A5CC4((**user_data).xC);
        }
        sis_id = mnDataDel_803EF8C8[(**user_data).x0 as usize];
        text = HSD_SisLib_803A5ACC(
            0 as i32,
            0 as i32,
            -9.5f32,
            9.1f32,
            17.0f32,
            364.68332f32,
            38.38772f32,
        );
        (**user_data).xC = text;
        (*text).font_size.x = 0.0521f32 as f32;
        (*text).font_size.y = 0.0521f32 as f32;
        HSD_SisLib_803A6368(text, sis_id as i32);
    }
}
unsafe extern "C" fn mnDataDel_UpdateDescription(
    mut user_data: *mut *mut MnDataDelGObjUserData,
) {
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut sis_id: u16 = 0;
    *user_data = (*mnDataDel_804D6C68).user_data as *mut MnDataDelGObjUserData;
    if !(**user_data).xC.is_null() {
        HSD_SisLib_803A5CC4((**user_data).xC);
    }
    sis_id = mnDataDel_803EF8C8[(**user_data).x0 as usize];
    text = HSD_SisLib_803A5ACC(
        0 as i32,
        0 as i32,
        -9.5f32,
        9.1f32,
        17.0f32,
        364.68332f32,
        38.38772f32,
    );
    (**user_data).xC = text;
    (*text).font_size.x = 0.0521f32 as f32;
    (*text).font_size.y = 0.0521f32 as f32;
    HSD_SisLib_803A6368(text, sis_id as i32);
}
pub unsafe extern "C" fn fn_8024F840(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8024F840");
    let mut zero: i32 = 0;
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut joint: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut buttons: u64 = 0;
    let mut cursor: u8 = 0;
    let mut user_data: *mut MnDataDelGObjUserData = ::core::ptr::null_mut::<
        MnDataDelGObjUserData,
    >();
    let mut temp_user_data: *mut MnDataDelGObjUserData = ::core::ptr::null_mut::<
        MnDataDelGObjUserData,
    >();
    let mut _pad: [u8; 40] = [0; 40];
    user_data = (*mnDataDel_804D6C68).user_data as *mut MnDataDelGObjUserData;
    if mn_804D6BC8.cooldown as i32 != 0 as i32 {
        Menu_DecrementAnimTimer();
        return;
    }
    buttons = Menu_GetAllInputs();
    zero = 0 as i32 as i32;
    if buttons & 0x20 as u64 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = zero as u8;
        mn_80229894(4 as i32, 5 as u16, 3 as i32);
        return;
    }
    if buttons & 0x10 as u64 != 0
        && *(&raw mut (*user_data).x3 as *mut u8).offset((*user_data).x0 as isize)
            as i32 == 0 as i32
    {
        (*user_data).x1 = 1 as u8;
        (*user_data).x2 = zero as u8;
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(fn_8024F318 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
        mnDataDel_8024EEC0();
        mn_804D6BC8.cooldown = 0xa as u16;
        lbAudioAx_800237A8(
            0xbc as enum_t,
            0x7f as i32,
            0x40 as i32,
        );
        return;
    }
    if buttons & 1 as u64 != 0 {
        sfxMove();
        temp_user_data = (*mnDataDel_804D6C68).user_data as *mut MnDataDelGObjUserData;
        joint = mn_80231634(
            (*temp_user_data).x10[mnDataDel_803EF8AC[(*user_data).x0 as usize] as usize]
                as *mut mn_80231634_t,
        ) as *mut HSD_JObj;
        mnDataDel_8024EBC8(joint, (*user_data).x0, 0 as u8, 0 as u8);
        cursor = (*user_data).x0;
        if cursor as i32 != 0 as i32 {
            (*user_data).x0 = (cursor as i32 - 1 as i32)
                as u8;
        } else {
            (*user_data).x0 = 5 as u8;
        }
        temp_user_data = (*mnDataDel_804D6C68).user_data as *mut MnDataDelGObjUserData;
        joint = mn_80231634(
            (*temp_user_data).x10[mnDataDel_803EF8AC[(*user_data).x0 as usize] as usize]
                as *mut mn_80231634_t,
        ) as *mut HSD_JObj;
        mnDataDel_8024EBC8(joint, (*user_data).x0, 1 as u8, 0 as u8);
        mnDataDel_UpdateDescription(&raw mut user_data);
        return;
    }
    fn_8024F840_inline(buttons, &raw mut user_data);
}
pub unsafe extern "C" fn fn_8024FBA4(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8024FBA4");
    let mut i: i32 = 0;
    let mut frame: f32 = 0.;
    let mut data: *mut MnDataDelData = ::core::ptr::null_mut::<MnDataDelData>();
    let mut user_data: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut _pad: [u8; 8] = [0; 8];
    data = &raw mut mnDataDel_803EF870;
    user_data = (*gobj).user_data as *mut u8;
    frame = mn_8022EE84(
        HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj,
        &raw mut mnDataDel_803EF870.xC,
        1152 as HSD_TypeMask,
    ) as f32;
    i = 0 as i32 as i32;
    while i < 6 as i32 {
        mn_8022EE84(
            *((&raw mut (*(user_data as *mut MnDataDelGObjUserData)).x10[mnDataDel_803EF8AC[i as usize] as usize]) as *mut *mut HSD_JObj),
            &raw mut (*data).xC,
            1152 as HSD_TypeMask,
        );
        i += 1;
    }
    if frame >= (*data).xC.end_frame {
        HSD_GObjFree(gobj);
    }
}
unsafe extern "C" fn fn_8024FC48_inline(mut arg0: isize) -> *mut HSD_JObj { // C: int; host: pointer-wide (mn_80231634)
    return arg0 as *mut HSD_JObj;
}
pub unsafe extern "C" fn fn_8024FC48(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8024FC48");
    let mut i: i32 = 0;
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut user_data: *mut u8 = ::core::ptr::null_mut::<u8>();
    user_data = (*gobj).user_data as *mut u8;
    if mn_804A04F0.cur_menu as i32 != 0x18 as i32 {
        let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(fn_8024FBA4 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
        HSD_SisLib_803A5CC4(
            (*(user_data as *mut MnDataDelGObjUserData)).xC,
        );
    } else {
        i = 0 as i32 as i32;
        while i < 6 as i32 {
            lb_80011E24(fn_8024FC48_inline(
                    mn_80231634(
                        *((&raw mut (*((*gobj).user_data as *mut MnDataDelGObjUserData)).x10[mnDataDel_803EF8AC[i as usize] as usize]) as *mut *mut mn_80231634_t),
                    ),
                ) as *mut HSD_JObj, &raw mut jobj, &[Va::I32(1 as i32), Va::I32(-(1 as i32))]);
            if *user_data.offset(0 as i32 as isize) as i32
                == i
            {
                mn_8022EC18(
                    jobj as *mut HSD_JObj,
                    &raw mut mnDataDel_803EF888,
                    TOBJ_MASK,
                );
            } else {
                mn_8022EC18(
                    jobj as *mut HSD_JObj,
                    &raw mut mnDataDel_803EF894,
                    TOBJ_MASK,
                );
            }
            i += 1;
        }
    };
}
unsafe extern "C" fn mnDataDel_GetAnimSettings(
    mut data: *mut MnDataDelData,
) -> *mut AnimLoopSettings {
    return &raw mut (*data).x0;
}
pub unsafe extern "C" fn fn_8024FD40(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8024FD40");
    let mut frame: f32 = 0.;
    let mut user_data: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut data: *mut MnDataDelData = ::core::ptr::null_mut::<MnDataDelData>();
    let mut i: i32 = 0;
    let mut _pad: [u8; 16] = [0; 16];
    jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    user_data = (*gobj).user_data as *mut u8;
    data = &raw mut mnDataDel_803EF870;
    if mn_804A04F0.cur_menu as i32 != 0x18 as i32 {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(fn_8024FBA4 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
        HSD_SisLib_803A5CC4(
            (*(user_data as *mut MnDataDelGObjUserData)).xC,
        );
        return;
    }
    frame = mn_8022EE84(
        jobj as *mut HSD_JObj,
        &raw mut mnDataDel_803EF870.x0,
        1152 as HSD_TypeMask,
    ) as f32;
    i = 0 as i32 as i32;
    while i < 6 as i32 {
        mn_8022EE84(
            *((&raw mut (*(user_data as *mut MnDataDelGObjUserData)).x10[mnDataDel_803EF8AC[i as usize] as usize]) as *mut *mut HSD_JObj),
            mnDataDel_GetAnimSettings(data),
            1152 as HSD_TypeMask,
        );
        i += 1;
    }
    if frame == (*data).x0.end_frame {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(fn_8024FC48 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
    }
}
pub unsafe extern "C" fn mnDataDel_8024FE4C(mut arg0: u8) {
    melee_core::ct!("mnDataDel_8024FE4C");
    let mut child: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut root: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut joint: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sis_id: u16 = 0;
    let mut is_enabled: u8 = 0;
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut frame: f32 = 0.;
    let mut enabled: i32 = 0;
    let mut i: i32 = 0;
    let mut assets: *mut StaticModelDesc = ::core::ptr::null_mut::<StaticModelDesc>();
    let mut user_data: *mut MnDataDelGObjUserData = ::core::ptr::null_mut::<
        MnDataDelGObjUserData,
    >();
    let mut cursor: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut _pad: [u8; 20] = [0; 20];
    assets = (&raw mut mnDataDel_PcAssets as *mut StaticModelDesc)
        .offset(0 as i32 as isize) as *mut StaticModelDesc;
    gobj = GObj_Create(6 as u16, 7 as u8, 0x80 as u8);
    mnDataDel_804D6C68 = gobj;
    root = HSD_JObjLoadJoint((*assets).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, root as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, i32) -> (),
        ),
        4 as u8,
        0x80 as u32,
    );
    HSD_JObjAddAnimAll(
        root,
        (*assets).animjoint,
        (*assets).matanim_joint,
        (*assets).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(root, 0.0f32);
    HSD_JObjAnimAll(root);
    user_data = HSD_MemAlloc(::core::mem::size_of::<MnDataDelGObjUserData>() as ssize_t)
        as *mut MnDataDelGObjUserData;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const i8
                as *mut i8, &[]);
        __assert(
            b"src/melee/mn/mndatadel.c\0" as *const u8 as *const i8,
            812 as u32,
            b"user_data\0" as *const u8 as *const i8,
        );
    };
    (*user_data).x0 = arg0;
    (*user_data).x1 = 0 as u8;
    (*user_data).x2 = 0 as u8;
    (*user_data).x3[0 as i32 as usize] = 0 as u8;
    (*user_data).x3[1 as i32 as usize] = 0 as u8;
    (*user_data).x3[2 as i32 as usize] = 0 as u8;
    (*user_data).x3[3 as i32 as usize] = 0 as u8;
    (*user_data).x3[4 as i32 as usize] = 0 as u8;
    (*user_data).x3[5 as i32 as usize] = 0 as u8;
    (*user_data).xC = ::core::ptr::null_mut::<HSD_Text>();
    GObj_InitUserData(
        gobj,
        0 as u8,
        Some(HSD_Free as unsafe extern "C" fn(*mut c_void) -> ()),
        user_data as *mut c_void,
    );
    enabled = 0 as i32 as i32;
    i = enabled;
    cursor = (user_data as *mut u8)
        .offset((i as i32 * 4 as i32) as isize);
    loop {
        lb_80011E24(root as *mut HSD_JObj, (&raw mut (*user_data).x10[i as usize]) as *mut *mut HSD_JObj /* menuguard: the C's ud+0x10+4*i, host field */, &[Va::I32(i), Va::I32(-(1 as i32))]);
        i += 1;
        cursor = cursor.offset(4 as i32 as isize);
        if !(i
            < (::core::mem::size_of::<[*mut HSD_JObj; 8]>() as usize)
                .wrapping_div(::core::mem::size_of::<*mut HSD_JObj>() as usize)
                as i32)
        {
            break;
        }
    }
    proc_0 = HSD_GObj_SetupProc(
        gobj,
        Some(fn_8024FD40 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
    assets = (&raw mut mnDataDel_PcAssets as *mut StaticModelDesc)
        .offset(1 as i32 as isize) as *mut StaticModelDesc;
    i = 0 as i32 as i32;
    while i < 6 as i32 {
        joint = HSD_JObjLoadJoint((*assets).joint as *mut HSD_Joint);
        HSD_JObjAddAnimAll(
            joint,
            (*assets).animjoint,
            (*assets).matanim_joint,
            (*assets).shapeanim_joint,
        );
        HSD_JObjReqAnimAll(joint, i as f32);
        HSD_JObjAnimAll(joint);
        jobj = joint;
        HSD_JObjAddChild(
            (*user_data).x10[mnDataDel_803EF8AC[i as usize] as usize],
            jobj,
        );
        mnDataDel_8024EBC8(
            jobj,
            i as u8,
            (i == (*user_data).x0 as i32) as i32 as u8,
            1 as u8,
        );
        is_enabled = (*user_data).x3[i as usize];
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut child, &[Va::I32(1 as i32), Va::I32(-(1 as i32))]);
        frame = mn_8022F298(child as *mut HSD_JObj);
        if is_enabled as i32 != 0 as i32 {
            enabled = 1 as i32 as i32;
        } else {
            enabled = 0 as i32 as i32;
        }
        HSD_JObjReqAnimAll(child, enabled as f32);
        mn_8022F3D8(child as *mut HSD_JObj, 0xff as u8, MOBJ_MASK);
        HSD_JObjAnimAll(child);
        HSD_JObjReqAnimAll(child, frame);
        mn_8022F3D8(child as *mut HSD_JObj, 0xff as u8, 1152 as HSD_TypeMask);
        HSD_JObjAnimAll(child);
        i += 1;
    }
    user_data = (*gobj).user_data as *mut MnDataDelGObjUserData;
    if !(*user_data).xC.is_null() {
        HSD_SisLib_803A5CC4((*user_data).xC);
    }
    sis_id = mnDataDel_803EF8C8[(*user_data).x0 as usize];
    text = HSD_SisLib_803A5ACC(
        0 as i32,
        0 as i32,
        -9.5f32,
        9.1f32,
        17.0f32,
        364.68332f32,
        38.38772f32,
    );
    (*user_data).xC = text;
    (*text).font_size.x = 0.0521f32 as f32;
    (*text).font_size.y = 0.0521f32 as f32;
    HSD_SisLib_803A6368(text, sis_id as i32);
}
pub unsafe extern "C" fn mnDataDel_80250170() {
    melee_core::ct!("mnDataDel_80250170");
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut archive: *mut HSD_Archive = ::core::ptr::null_mut::<HSD_Archive>();
    let mut assets: *mut StaticModelDesc = ::core::ptr::null_mut::<StaticModelDesc>();
    assets = (&raw mut mnDataDel_PcAssets as *mut StaticModelDesc)
        .offset(0 as i32 as isize) as *mut StaticModelDesc;
    mn_804D6BC8.cooldown = 5 as u16;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x18 as MenuKind8;
    mn_804A04F0.hovered_selection = 0 as u16;
    mnDataDel_804D6C6C = ::core::ptr::null_mut::<HSD_Text>();
    archive = mn_804D6BB8;
    lbArchive_LoadSections(
        archive,
        &raw mut (*assets.offset(0 as i32 as isize)).joint as *mut *mut c_void,
        &[
            (b"MenMainConDl_Top_joint\0" as *const u8 as *const i8) as *const c_void,
            (&raw mut (*assets.offset(0 as i32 as isize)).animjoint) as *const c_void,
            (b"MenMainConDl_Top_animjoint\0" as *const u8 as *const i8) as *const c_void,
            (&raw mut (*assets.offset(0 as i32 as isize)).matanim_joint) as *const c_void,
            (b"MenMainConDl_Top_matanim_joint\0" as *const u8 as *const i8) as *const c_void,
            (&raw mut (*assets.offset(0 as i32 as isize)).shapeanim_joint) as *const c_void,
            (b"MenMainConDl_Top_shapeanim_joint\0" as *const u8 as *const i8) as *const c_void,
            (&raw mut (*assets.offset(1 as i32 as isize)).joint) as *const c_void,
            (b"MenMainCursorDl_Top_joint\0" as *const u8 as *const i8) as *const c_void,
            (&raw mut (*assets.offset(1 as i32 as isize)).animjoint) as *const c_void,
            (b"MenMainCursorDl_Top_animjoint\0" as *const u8 as *const i8) as *const c_void,
            (&raw mut (*assets.offset(1 as i32 as isize)).matanim_joint) as *const c_void,
            (b"MenMainCursorDl_Top_matanim_joint\0" as *const u8 as *const i8) as *const c_void,
            (&raw mut (*assets.offset(1 as i32 as isize)).shapeanim_joint) as *const c_void,
            (b"MenMainCursorDl_Top_shapeanim_joint\0" as *const u8 as *const i8) as *const c_void,
            (&raw mut (*assets.offset(2 as i32 as isize)).joint) as *const c_void,
            (b"MenMainWarCmn_Top_joint\0" as *const u8 as *const i8) as *const c_void,
            (&raw mut (*assets.offset(2 as i32 as isize)).animjoint) as *const c_void,
            (b"MenMainWarCmn_Top_animjoint\0" as *const u8 as *const i8) as *const c_void,
            (&raw mut (*assets.offset(2 as i32 as isize)).matanim_joint) as *const c_void,
            (b"MenMainWarCmn_Top_matanim_joint\0" as *const u8 as *const i8) as *const c_void,
            (&raw mut (*assets.offset(2 as i32 as isize)).shapeanim_joint) as *const c_void,
            (b"MenMainWarCmn_Top_shapeanim_joint\0" as *const u8 as *const i8) as *const c_void,
            core::ptr::null(),
        ],
    );
    mnDataDel_8024FE4C(0 as u8);
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0 as u16, 1 as u8, 0x80 as u8),
        Some(fn_8024F840 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
}
unsafe extern "C" fn mnDataDel_StageRelock() {
    let mut i: i32 = 0;
    let mut found: i32 = 0 as i32;
    i = 0 as i32;
    while i < 0x1d as i32 {
        if gm_80164430(gm_801641CC(i as u8)) != 0 as i32
            && gm_IsStageUnlocked(i as u16) as i32
                & (0xffff as i32 & 0xff as i32)
                != 0 as i32
        {
            found = 1 as i32;
            break;
        } else {
            i += 1;
        }
    }
    if found == 0 as i32 {
        gm_801641E4(0 as u8, 1 as u8);
    }
}
unsafe extern "C" fn mnDataDel_RunCall(mut id: i32) {
    match id {
        0 => {
            gm_8016505C();
        }
        1 => {
            gmMainLib_8015F464();
        }
        2 => {
            gm_801729EC();
        }
        3 => {
            lbCardGame_SaveChanges();
        }
        4 => {
            gmMainLib_8015DB80();
        }
        5 => {
            gm_801647D0();
        }
        6 => {
            gmMainLib_8015F490();
        }
        7 => {
            mnDataDel_StageRelock();
        }
        8 => {
            gmMainLib_8015EEC8();
        }
        9 => {
            gmMainLib_8015F150();
        }
        10 => {
            gmMainLib_8015F260();
        }
        11 => {
            Toy_80311960();
        }
        12 => {
            gmMainLib_8015F4BC();
        }
        13 => {
            gm_80174238();
        }
        14 => {
            mnDataDel_erase_lang = lbLang_GetSavedLanguage() as i32;
        }
        15 => {
            gmMainLib_8015FBA4();
        }
        16 => {
            gm_801A3EF4();
        }
        17 => {
            if mnDataDel_erase_lang != lbLang_GetSavedLanguage() {
                lbLang_SetSavedLanguage(mnDataDel_erase_lang as enum_t);
            }
        }
        18 => {
            gm_801603B0();
        }
        19 => {
            gmMainLib_8015F588(gmMainLib_8015F4E8() as u8 as i32);
        }
        _ => {}
    };
}
pub unsafe extern "C" fn mnDataDel_Erase(mut category: i32) {
    melee_core::ct!("mnDataDel_Erase");
    at_erase_run(
        category,
        1 as i32,
        Some(mnDataDel_RunCall as unsafe extern "C" fn(i32) -> ()),
    );
}
