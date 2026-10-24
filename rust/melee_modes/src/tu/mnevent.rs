//! TARGET_PC port of src/melee/mn/mnevent.c, checked against the native C state oracle.
#![allow(non_snake_case, non_upper_case_globals, unused_imports, unused_variables, unused_assignments, unused_mut, static_mut_refs, clippy::all)]
use melee_types::*;
use std::vec::Vec;
use crate::oracle::Va;
use super::mninl::*;
use super::hsd::{sfxBack,sfxForward,sfxMove};
use melee_core::fp::fmadds;
pub mod ext { use super::*; crate::oext_fns! {
fn __assert(a0: *const ::core::ffi::c_char, a1: u32, a2: *const ::core::ffi::c_char,)  -> ();
fn GObj_Create(classifier: u16, p_link: u8, priority: u8)  -> *mut HSD_GObj;
fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: ::core::ffi::c_int)  -> () = crate::tu::adapt::HSD_GObj_JObjCallback__b37ff8;
fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32)  -> ();
fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj)  -> ();
fn HSD_JObjLoadJoint(a0: *mut HSD_Joint)  -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint,)  -> ();
fn HSD_JObjAnimAll(a0: *mut HSD_JObj)  -> ();
fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32)  -> ();
fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, flags: u32)  -> ();
fn lbAudioAx_80024030(a0: ::core::ffi::c_int)  -> () = crate::tu::adapt::lbAudioAx_80024030__52c54d;
fn mn_802295AC()  -> u8 = crate::tu::adapt::mn_802295AC__671d2a;
fn mn_80229624(a0: u32)  -> u32;
fn mn_80229860(a0: i8)  -> () = crate::tu::adapt::mn_80229860__033db5;
fn mn_80229894(a0: i32, a1: u16, a2: i32)  -> ();
fn mn_8022EA78(buf: *mut ::core::ffi::c_char, buf_end: ::core::ffi::c_int, num: u32,)  -> ();
fn mn_8022EC18(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings, a2: HSD_TypeMask,)  -> ::core::ffi::c_float = crate::tu::adapt::mn_8022EC18__c43f51;
fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings)  -> ::core::ffi::c_float = crate::tu::adapt::mn_8022ED6C__a278c3;
fn HSD_SisLib_803A6754(arg0: ::core::ffi::c_int, arg1: ::core::ffi::c_int,)  -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A6754__6086ec;
fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32)  -> () = crate::tu::adapt::HSD_SisLib_803A6368__2e8f0d;
fn HSD_SisLib_803A5ACC(a0: ::core::ffi::c_int, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32,)  -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A5ACC__97831a;
fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text)  -> () = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
fn gm_80162EC8()  -> i32 = crate::tu::adapt::gm_80162EC8__625f2c;
fn gm_CKindToSelKind(ckind: u8)  -> u8 = crate::tu::adapt::gm_CKindToSelKind__a9d153;
fn gm_IsCKindUnlocked(ckind: u8)  -> i32 = crate::tu::adapt::gm_IsCKindUnlocked__b6c64b;
fn gm_801677E8(arg0: i8)  -> () = crate::tu::adapt::gm_801677E8__7189c6;
fn gm_801BEB74(a0: u8)  -> () = crate::tu::adapt::gm_801BEB74__1db024;
fn gm_801BEB8C(a0: u8)  -> i32 = crate::tu::adapt::gm_801BEB8C__797c45;
fn gm_801BEBA8(a0: u8)  -> u8 = crate::tu::adapt::gm_801BEBA8__dc2738;
fn gm_801BEBC0(a0: u8)  -> u8 = crate::tu::adapt::gm_801BEBC0__760ddc;
fn gm_801BEBF8(arg0: u8)  -> u8 = crate::tu::adapt::gm_801BEBF8__50ad08;
fn gmMainLib_8015CEFC(a0: ::core::ffi::c_int)  -> i32 = crate::tu::adapt::gmMainLib_8015CEFC__f899c1;
fn gmMainLib_8015CF5C(a0: i32)  -> i32 = crate::tu::adapt::gmMainLib_8015CF5C__8fd145;
fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32,)  -> ();
fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut ::core::ffi::c_void,)  -> ();
fn HSD_GObjFree(a0: *mut HSD_GObj)  -> ();
fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8,)  -> *mut HSD_GObjProc;
fn HSD_GObjProc_RemoveProc(gproc: *mut HSD_GObjProc)  -> ();
fn GObj_InitUserData(gobj:*mut HSD_GObj, kind:u8, remove_func:Option<unsafe extern "C" fn(*mut core::ffi::c_void)>, data:*mut core::ffi::c_void)->();
fn HSD_Free(ptr: *mut ::core::ffi::c_void) -> () = crate::tu::adapt::HSD_Free__bb56fc;
fn HSD_MemAlloc(size: i32) -> *mut ::core::ffi::c_void = crate::tu::adapt::HSD_MemAlloc__e04cd7;
}
pub use super::super::mncount::ext::lbArchive_LoadSections;
crate::ext_va! { fn OSReport(fmt: *const i8) va 0 -> (); fn HSD_SisLib_803A6B98(t: *mut HSD_Text, x:f32,y:f32,fmt:*const i8) va 3 -> i32; }
crate::ext_vk! { fn lb_80011E24(j:*mut HSD_JObj,out:*mut *mut HSD_JObj) vk "ii" -> i32; }
}
use ext::*;
crate::globals! {
rw pub MenMainConEv: StaticModelDesc  = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
rw pub MenMainMarkEv: StaticModelDesc  = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
rw pub mnEvent_804D6C60: *mut HSD_GObj  = ::core::ptr::null::<HSD_GObj>()
    as *mut HSD_GObj;
rw pub mnEvent_804D6C64: u8  = 0;
rw pub mnEvent_804D6C65: u8  = 0;
rw pub mnEvent_803EF740: AnimLoopSettings  = AnimLoopSettings {
    start_frame: 0 as i32 as f32,
    end_frame: 0 as i32 as f32,
    loop_frame: -0.1f32,
};
rw pub mnEvent_803EF74C: AnimLoopSettings  = AnimLoopSettings {
    start_frame: 0 as i32 as f32,
    end_frame: 0 as i32 as f32,
    loop_frame: -0.1f32,
};
rw pub mnEvent_803EF758: AnimLoopSettings  = AnimLoopSettings {
    start_frame: 0 as i32 as f32,
    end_frame: 199.0f32,
    loop_frame: 0 as i32 as f32,
};
rw pub mnEvent_803EF764: Vec3  = Vec3 {
    x: -3.8f32,
    y: -0.6f32,
    z: 0 as i32 as f32,
};
rw pub mnEvent_803EF770: Vec3  = Vec3 {
    x: 1.0f32,
    y: -0.6f32,
    z: 0 as i32 as f32,
};
rw pub mnEvent_804D5028: GXColor  = GXColor {
    r: 0xca as u8,
    g: 0xbc as u8,
    b: 0x9f as u8,
    a: 0xff as u8,
};
rw pub mnEvent_804D502C: GXColor  = GXColor {
    r: 0 as u8,
    g: 0 as u8,
    b: 0 as u8,
    a: 0xff as u8,
};
/// mnmain.c
rw pub mn_804A04F0: MenuFlow = unsafe { core::mem::zeroed() };
/// mnmain.c
rw pub mn_804D6BC8: MenuInputState = unsafe { core::mem::zeroed() };
/// mnmain.c
rw pub mn_804D6BB8: *mut HSD_Archive = ::core::ptr::null_mut();
/// baselib/gobj.c
rw pub HSD_GObj_804D783C: i32 = 0;
/// baselib/gobjobject.c
rw pub HSD_GObj_JObjKind: u8 = 0;
/// db
rw pub HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = ::core::ptr::null_mut();
rw pub DbLevel: i32 = 0;
}
unsafe extern "C" fn mnEvent_CountUnlocked() -> i32 {
    let mut i: i32 = 0;
    let mut count: i32 = 0 as i32;
    i = 0 as i32 as i32;
    while i < 0x33 as i32 {
        if gmMainLib_8015CEFC(i as i32) != 0 as i32 {
            count += 1 as i32;
        }
        i += 1;
    }
    return count;
}
pub unsafe extern "C" fn mnEvent_8024CE74() -> i32 {
    let mut count: i32 = 0;
    if DbLevel as i32 > DbLKind_DebugDevelop as i32 {
        return 0x2a as i32;
    }
    count = mnEvent_CountUnlocked();
    if count <= 5 as i32 {
        return 1 as i32;
    }
    if count <= 9 as i32 {
        return 6 as i32;
    }
    if count <= 0xf as i32 {
        return 0xb as i32;
    }
    if count <= 0x15 as i32 {
        return 0x10 as i32;
    }
    if count <= 0x1a as i32 {
        return 0x14 as i32;
    }
    if count >= 0x32 as i32 {
        if gm_80162EC8() != 0 {
            return 0x2a as i32;
        }
        return 0x29 as i32;
    }
    if gm_IsCKindUnlocked(CKind_Falco as i32 as u8) != 0
        && gm_IsCKindUnlocked(CKind_Luigi as i32 as u8) != 0
        && gm_IsCKindUnlocked(CKind_Purin as i32 as u8) != 0
        && gm_IsCKindUnlocked(CKind_CLink as i32 as u8) != 0
    {
        if gm_IsCKindUnlocked(CKind_Mewtwo as i32 as u8) != 0
            && gm_IsCKindUnlocked(CKind_Mars as i32 as u8) != 0
            && gm_IsCKindUnlocked(CKind_GameWatch as i32 as u8) != 0
            && gm_IsCKindUnlocked(CKind_Pichu as i32 as u8) != 0
            && gm_IsCKindUnlocked(CKind_Ganon as i32 as u8) != 0
            && gm_IsCKindUnlocked(CKind_DrMario as i32 as u8) != 0
            && gm_IsCKindUnlocked(CKind_Emblem as i32 as u8) != 0
        {
            return 0x29 as i32;
        }
        return 0x1e as i32;
    }
    return 0x15 as i32;
}
pub unsafe extern "C" fn mnEvent_8024D014(mut gobj: *mut HSD_GObj) {
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut data: *mut MnEventData = (*gobj).user_data as *mut MnEventData;
    let mut tree: *mut HSD_JObj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    lb_80011E24(tree, &raw mut jobj, &[Va::I32(3 as i32), Va::I32(-(1 as i32))]);
    if (*data).first_event == 0 as i32 {
        HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
    lb_80011E24(tree, &raw mut jobj, &[Va::I32(2 as i32), Va::I32(-(1 as i32))]);
    if (*data).first_event == mnEvent_8024CE74() {
        HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    };
}
pub unsafe extern "C" fn mnEvent_8024D0CC(mut gobj: *mut HSD_GObj, mut ckind: i32) {
    let mut tree: *mut HSD_JObj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut frame: f32 = 0.;
    let mut _0: u32 = 0 as u32;
    if ckind == ChKind_None as i32 {
        frame = 25.0f32 as f32;
    } else {
        frame = gm_CKindToSelKind(ckind as u8) as f32;
    }
    lb_80011E24(tree, &raw mut jobj, &[Va::I32(4 as i32), Va::I32(-(1 as i32))]);
    HSD_JObjReqAnimAll(jobj, frame);
    HSD_JObjAnimAll(jobj);
}
unsafe extern "C" fn mnEvent_GetData() -> *mut MnEventData {
    return (*mnEvent_804D6C60).user_data as *mut MnEventData;
}
unsafe extern "C" fn getMainConEv() -> *mut StaticModelDesc {
    return &raw mut MenMainConEv;
}
unsafe extern "C" fn getMainMarkEv() -> *mut StaticModelDesc {
    return &raw mut MenMainMarkEv;
}
unsafe extern "C" fn mnEvent_CreateIconForSlot(
    mut idx: i32,
    mut slot: *mut *mut HSD_GObj,
    mut jobj_0A: *mut *mut HSD_JObj,
    mut jobj_0C: *mut *mut HSD_JObj,
    mut pos: *mut Vec3,
) {
    let mut icon_gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut icon_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut tree: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut spacing: f32 = 0.;
    let mut assets: *mut StaticModelDesc = ::core::ptr::null_mut::<StaticModelDesc>();
    assets = getMainMarkEv();
    tree = (*mnEvent_804D6C60).hsd_obj as *mut HSD_JObj;
    lb_80011E24(tree, jobj_0A, &[Va::I32(0xa as i32), Va::I32(-(1 as i32))]);
    lb_80011E24(tree, jobj_0C, &[Va::I32(0xc as i32), Va::I32(-(1 as i32))]);
    spacing = HSD_JObjGetTranslationY(*jobj_0A);
    spacing = HSD_JObjGetTranslationY(*jobj_0C) - spacing;
    HSD_JObjGetTranslation(*jobj_0A, pos);
    (*pos).y = (*pos).y + idx as f32 * spacing;
    icon_gobj = GObj_Create(6 as u16, 7 as u8, 0x80 as u8);
    icon_jobj = HSD_JObjLoadJoint((*assets).joint);
    HSD_GObjObject_80390A70(
        icon_gobj,
        HSD_GObj_JObjKind,
        icon_jobj as *mut ::core::ffi::c_void,
    );
    GObj_SetupGXLink(
        icon_gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, i32) -> (),
        ),
        4 as u8,
        0x80 as u32,
    );
    mnEvent_8024D4E0(icon_jobj, pos);
    *slot = icon_gobj;
}
#[no_mangle]pub unsafe extern "C" fn mnEvent_8024D15C(mut idx: i32, mut event_id: i32) {
    let mut jobj_0A: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut jobj_0C: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    let mut icon_pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut icon_jobj_0C: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut icon_jobj_0A: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut data: *mut MnEventData = ::core::ptr::null_mut::<MnEventData>();
    let mut text_base: *mut *mut HSD_Text = ::core::ptr::null_mut::<*mut HSD_Text>();
    let mut text_slot: *mut *mut HSD_Text = ::core::ptr::null_mut::<*mut HSD_Text>();
    let mut icon_base: *mut *mut HSD_Text = ::core::ptr::null_mut::<*mut HSD_Text>();
    let mut icon_slot: *mut *mut HSD_Text = ::core::ptr::null_mut::<*mut HSD_Text>();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut icon_text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut spacing: f32 = 0.;
    let mut text_x: f32 = 0.;
    let mut text_y: f32 = 0.;
    let mut tree: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sis_idx: i32 = 0;
    let mut __0: [::core::ffi::c_uchar; 40] = [0; 40];
    tree = (*mnEvent_804D6C60).hsd_obj as *mut HSD_JObj;
    data = mnEvent_GetData();
    lb_80011E24(tree, &raw mut jobj_0A, &[Va::I32(0xa as ::core::ffi::c_int), Va::I32(-(1 as ::core::ffi::c_int))]);
    lb_80011E24(tree, &raw mut jobj_0C, &[Va::I32(0xc as ::core::ffi::c_int), Va::I32(-(1 as ::core::ffi::c_int))]);
    spacing = HSD_JObjGetTranslationY(jobj_0A);
    spacing = HSD_JObjGetTranslationY(jobj_0C) - spacing;
    HSD_JObjGetTranslation(jobj_0A, &raw mut pos);
    pos.y = -(idx as f32 * spacing + pos.y);
    if !(*data).gobjs[idx as usize].is_null() {
        let mut gobjs: *mut *mut HSD_GObj = &raw mut (*data).gobjs as *mut *mut HSD_GObj;
        let mut old_gobj: *mut HSD_GObj = *gobjs.offset(idx as isize);
        HSD_GObjFree(old_gobj);
        (*data).gobjs[idx as usize] = ::core::ptr::null_mut::<HSD_GObj>();
    }
    if gmMainLib_8015CEFC(event_id as ::core::ffi::c_int) != 0 as ::core::ffi::c_int {
        mnEvent_CreateIconForSlot(
            idx,
            (&raw mut (*data).gobjs as *mut *mut HSD_GObj).offset(idx as isize)
                as *mut *mut HSD_GObj,
            &raw mut icon_jobj_0A,
            &raw mut icon_jobj_0C,
            &raw mut icon_pos,
        );
    }
    text_base = &raw mut (*data).texts as *mut *mut HSD_Text;
    text_slot = text_base.offset(idx as isize) as *mut *mut HSD_Text;
    if !(*text_slot).is_null() {
        HSD_SisLib_803A5CC4((*data).texts[idx as usize]);
    }
    text = HSD_SisLib_803A6754(0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    *text_slot = text;
    (*text).font_size.x = 0.035f32 as f32;
    (*text).font_size.y = 0.035f32 as f32;
    text_x = pos.x + mnEvent_803EF764.x;
    text_y = pos.y + mnEvent_803EF764.y;
    (*text).pos_x = text_x;
    (*text).pos_y = text_y;
    (*text).pos_z = 17.0f32 as f32;
    (*text).default_kerning = 1 as u8;
    (*text).text_color = mnEvent_804D5028;
    HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, b"\xEF\xBC\xAC\xEF\xBD\x96\xEF\xBC\x8E %d\0" as *const u8
            as *const ::core::ffi::c_char, &[Va::I32((event_id as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as i32)]);
    icon_base = &raw mut (*data).icons as *mut *mut HSD_Text;
    icon_slot = icon_base.offset(idx as isize) as *mut *mut HSD_Text;
    if !(*icon_slot).is_null() {
        HSD_SisLib_803A5CC4((*data).icons[idx as usize]);
    }
    icon_text = HSD_SisLib_803A5ACC(
        0 as ::core::ffi::c_int,
        1 as i32,
        pos.x + mnEvent_803EF770.x,
        pos.y + mnEvent_803EF770.y,
        17.0f32,
        364.68332f32,
        38.38772f32,
    );
    *icon_slot = icon_text;
    (*icon_text).font_size.x = 0.035f32 as f32;
    (*icon_text).font_size.y = 0.035f32 as f32;
    sis_idx = ((gm_801BEBA8(event_id as u8) as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int & 0x1fe as ::core::ffi::c_int)
        + 0x154 as ::core::ffi::c_int) as i32;
    HSD_SisLib_803A6368(icon_text, sis_idx);
}
pub unsafe extern "C" fn mnEvent_8024D4E0(
    mut jobj: *mut HSD_JObj,
    mut translate: *mut Vec3,
) {
    HSD_JObjSetTranslate(jobj, translate);
}
pub unsafe extern "C" fn mnEvent_8024D5B0(mut gobj: *mut HSD_GObj, mut event: u8) {
    let mut time_text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut temp_r3: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut sp18: [::core::ffi::c_char; 4] = [0; 4];
    let mut sp14: [::core::ffi::c_char; 4] = [0; 4];
    let mut sp10: [::core::ffi::c_char; 4] = [0; 4];
    let mut temp_r30: u32 = 0;
    let mut temp_r28: *mut MnEventData = ::core::ptr::null_mut::<MnEventData>();
    let mut temp_r27: u8 = 0;
    temp_r28 = (*gobj).user_data as *mut MnEventData;
    temp_r27 = event;
    temp_r3 = (*temp_r28).name_text;
    if !temp_r3.is_null() {
        HSD_SisLib_803A5CC4((*temp_r28).name_text);
    }
    time_text = HSD_SisLib_803A6754(0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    (*temp_r28).name_text = time_text;
    (*time_text).pos_x = 3.8f32 as f32;
    (*time_text).pos_y = 6.9f32 as f32;
    (*time_text).pos_z = 17.0f32 as f32;
    (*time_text).text_color = mnEvent_804D502C;
    (*time_text).default_alignment = 2 as u8;
    (*time_text).font_size.x = 0.03f32 as f32;
    (*time_text).font_size.y = 0.03f32 as f32;
    temp_r30 = gmMainLib_8015CF5C(
        gm_801BEBC0(
            (temp_r27 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as u8,
        ) as i32,
    ) as u32;
    if gm_801BEB8C(
        gm_801BEBC0(
            (temp_r27 as ::core::ffi::c_uint & 0xff as ::core::ffi::c_uint
                & 0xff as ::core::ffi::c_uint) as u8,
        ) as u32 as u8,
    ) as ::core::ffi::c_uint & 0xff as ::core::ffi::c_uint != 0 as ::core::ffi::c_uint
    {
        if gmMainLib_8015CEFC(
            gm_801BEBC0(
                (event as ::core::ffi::c_uint & 0xffffffff as ::core::ffi::c_uint)
                    as u8,
            ) as ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int
        {
            mn_8022EA78(
                &raw mut sp18 as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
                temp_r30
                    .wrapping_div(60 as u32)
                    .wrapping_div(60 as u32)
                    .wrapping_rem(60 as u32),
            );
            mn_8022EA78(
                &raw mut sp14 as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
                temp_r30.wrapping_div(60 as u32).wrapping_rem(60 as u32),
            );
            mn_8022EA78(
                &raw mut sp10 as *mut ::core::ffi::c_char,
                2 as ::core::ffi::c_int,
                (99.0f32 * temp_r30.wrapping_rem(60 as u32) as ::core::ffi::c_float
                    / 59.0f32) as i32 as u32,
            );
            HSD_SisLib_803A6B98(time_text, 0.0f32, 0.0f32, b"%s:%s %s\0" as *const u8 as *const ::core::ffi::c_char, &[Va::P((&raw mut sp18 as *mut ::core::ffi::c_char).cast()), Va::P((&raw mut sp14 as *mut ::core::ffi::c_char).cast()), Va::P((&raw mut sp10 as *mut ::core::ffi::c_char).cast())]);
            return;
        }
        (*time_text).pos_x = 4.25f32 as f32;
        (*time_text).pos_y = 6.9f32 as f32;
        (*time_text).pos_z = 17.0f32 as f32;
        (*time_text).default_kerning = 1 as u8;
        HSD_SisLib_803A6B98(time_text, 0.0f32, 0.0f32, b"\xEF\xBC\x8D\xEF\xBC\x8D:\xEF\xBC\x8D\xEF\xBC\x8D \xEF\xBC\x8D\xEF\xBC\x8D\0"
                as *const u8 as *const ::core::ffi::c_char, &[Va::P((&raw mut sp18 as *mut ::core::ffi::c_char).cast()), Va::P((&raw mut sp14 as *mut ::core::ffi::c_char).cast()), Va::P((&raw mut sp10 as *mut ::core::ffi::c_char).cast())]);
        return;
    }
    if gmMainLib_8015CEFC(
        gm_801BEBC0(
            (event as ::core::ffi::c_ulonglong
                & 0xffffffffffffffff as ::core::ffi::c_ulonglong) as u8,
        ) as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        HSD_SisLib_803A6B98(time_text, 0.0f32, 0.0f32, b"%d\0" as *const u8 as *const ::core::ffi::c_char, &[Va::I32((temp_r30) as i32)]);
        return;
    }
    (*time_text).default_kerning = 1 as u8;
    HSD_SisLib_803A6B98(time_text, 0.0f32, 0.0f32, b"\xEF\xBC\x8D\0" as *const u8 as *const ::core::ffi::c_char, &[]);
}
pub unsafe extern "C" fn mnEvent_8024D7E0(mut gobj: *mut HSD_GObj, mut idx: i32) {
    let mut data: *mut MnEventData = (*gobj).user_data as *mut MnEventData;
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut sis_idx: i32 = 0;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    if !(*data).desc_text.is_null() {
        HSD_SisLib_803A5CC4((*data).desc_text);
    }
    sis_idx = (idx as ::core::ffi::c_int * 2 as ::core::ffi::c_int
        + 0x155 as ::core::ffi::c_int) as i32;
    text = HSD_SisLib_803A5ACC(
        0 as ::core::ffi::c_int,
        1 as i32,
        -9.5f32,
        8.0f32,
        17.0f32,
        364.68332f32,
        38.38772f32,
    );
    (*data).desc_text = text;
    (*text).font_size.x = 0.0521f32 as f32;
    (*text).font_size.y = 0.0521f32 as f32;
    HSD_SisLib_803A6368(text, sis_idx);
}
#[inline]
pub unsafe extern "C" fn mnEvent_RefreshRows(mut first_event: i32) {
    let mut i: i32 = 0;
    i = 0 as ::core::ffi::c_int as i32;
    while i < 9 as ::core::ffi::c_int {
        mnEvent_8024D15C(i, first_event + i);
        i += 1;
    }
}
#[inline]
pub unsafe extern "C" fn mnEvent_ShowSelected(
    mut user_data: *mut MnEventData,
    mut jobj_09: *mut *mut HSD_JObj,
) {
    let mut event: i32 = 0;
    let mut event_id: i32 = 0;
    let mut event_gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    event = gm_801BEBA8(
        (((*user_data).first_event as ::core::ffi::c_int
            + (*user_data).page as ::core::ffi::c_int) as u32 & 0xff as u32) as u8,
    ) as i32;
    event_gobj = mnEvent_804D6C60;
    event_id = event;
    mnEvent_8024D0CC(
        event_gobj,
        gm_801BEBF8(
            (event_id as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as u8,
        ) as i8 as i32,
    );
    mnEvent_8024D7E0(event_gobj, event_id);
    mnEvent_8024D5B0(event_gobj, event as u8);
    lb_80011E24((*event_gobj).hsd_obj as *mut HSD_JObj, jobj_09, &[Va::I32(9 as ::core::ffi::c_int), Va::I32(-(1 as ::core::ffi::c_int))]);
    HSD_JObjReqAnimAll(
        *jobj_09,
        gm_801BEB8C(
            (gm_801BEBC0(event_id as u8) as ::core::ffi::c_int
                & 0xff as ::core::ffi::c_int) as u8,
        ) as u8 as f32,
    );
    HSD_JObjAnimAll(*jobj_09);
    mnEvent_8024D014(event_gobj);
}
#[inline]
pub unsafe extern "C" fn mnEvent_SetPageCursorY(
    mut cursor_data: *mut MnEventData,
    mut jobj_0A: *mut *mut HSD_JObj,
    mut jobj_0C: *mut *mut HSD_JObj,
    mut jobj_0B: *mut *mut HSD_JObj,
) {
    let mut page: u8 = 0;
    let mut tree: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut y_a: f32 = 0.;
    let mut y_b: f32 = 0.;
    page = (*cursor_data).page;
    tree = (*mnEvent_804D6C60).hsd_obj as *mut HSD_JObj;
    lb_80011E24(tree, jobj_0A, &[Va::I32(0xa as ::core::ffi::c_int), Va::I32(-(1 as ::core::ffi::c_int))]);
    lb_80011E24(tree, jobj_0C, &[Va::I32(0xc as ::core::ffi::c_int), Va::I32(-(1 as ::core::ffi::c_int))]);
    y_a = HSD_JObjGetTranslationY(*jobj_0A);
    y_b = HSD_JObjGetTranslationY(*jobj_0C);
    lb_80011E24(tree, jobj_0B, &[Va::I32(0xb as ::core::ffi::c_int), Va::I32(-(1 as ::core::ffi::c_int))]);
    HSD_JObjSetTranslateY(*jobj_0B, page as f32 * (y_b - y_a));
}
pub unsafe extern "C" fn mnEvent_8024D864(mut gobj: *mut HSD_GObj) {
    let mut page: u8 = 0;
    let mut data: *mut MnEventData = ::core::ptr::null_mut::<MnEventData>();
    let mut up_jobj_0B: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut inputs: u64 = 0;
    let mut up_jobj_0C: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut up_jobj_0A: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut down_jobj_0B: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut down_jobj_0C: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut down_jobj_0A: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    if mn_804D6BC8.cooldown as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        mn_804D6BC8.cooldown = (mn_804D6BC8.cooldown as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as u16;
        mn_804D6BC8.x2 = 0 as u16;
        mn_804D6BC8.x4 = 0 as ::core::ffi::c_int as i32;
        return;
    }
    mn_804A04F0.buttons = mn_80229624(4 as u32) as u64;
    inputs = mn_804A04F0.buttons;
    if inputs & MenuInput_Back as ::core::ffi::c_int as u64 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8;
        mn_80229894(1 as i32, 1 as u16, 3 as i32);
        return;
    }
    if mnEvent_804D6C64 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        mnEvent_804D6C64 = (mnEvent_804D6C64 as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as u8;
        return;
    }
    if mnEvent_804D6C60.is_null() {
        mnEvent_8024E524(mnEvent_804D6C65 as i32);
    }
    data = (*mnEvent_804D6C60).user_data as *mut MnEventData;
    if inputs & MenuInput_Confirm as ::core::ffi::c_int as u64 != 0 {
        sfxForward();
        gm_801BEB74(
            ((*data).first_event as ::core::ffi::c_int
                + (*data).page as ::core::ffi::c_int) as u8,
        );
        gm_801677E8(mn_802295AC() as i8);
        mn_80229860(GM_EVENT as ::core::ffi::c_int as i8);
        return;
    }
    if inputs & MenuInput_XButton as ::core::ffi::c_int as u64 != 0 {
        let mut jobj_09: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
        let mut max_events: i32 = mnEvent_8024CE74();
        if ((*data).first_event as ::core::ffi::c_int + 9 as ::core::ffi::c_int)
            < max_events || (*data).first_event != max_events
        {
            sfxMove();
            if ((*data).first_event as ::core::ffi::c_int + 9 as ::core::ffi::c_int)
                < max_events
            {
                (*data).first_event += 9 as ::core::ffi::c_int;
            } else {
                (*data).first_event = max_events;
            }
            mnEvent_RefreshRows((*data).first_event);
            mnEvent_ShowSelected(data, &raw mut jobj_09);
        }
    } else if inputs & MenuInput_YButton as ::core::ffi::c_int as u64 != 0 {
        let mut jobj_09_0: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
        if (*data).first_event as ::core::ffi::c_int - 9 as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int || (*data).first_event != 0 as ::core::ffi::c_int
        {
            sfxMove();
            if (*data).first_event as ::core::ffi::c_int - 9 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                (*data).first_event -= 9 as ::core::ffi::c_int;
            } else {
                (*data).first_event = 0 as ::core::ffi::c_int as i32;
            }
            mnEvent_RefreshRows((*data).first_event);
            mnEvent_ShowSelected(data, &raw mut jobj_09_0);
        }
        let mut _pad: [::core::ffi::c_uchar; 4] = [0; 4];
    } else if inputs & MenuInput_Up as ::core::ffi::c_int as u64 != 0 {
        page = (*data).page;
        if page as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            let mut jobj_09_1: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            sfxMove();
            (*data).page = ((*data).page as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                as u8;
            mnEvent_SetPageCursorY(
                data,
                &raw mut up_jobj_0A,
                &raw mut up_jobj_0C,
                &raw mut up_jobj_0B,
            );
            mnEvent_ShowSelected(data, &raw mut jobj_09_1);
            return;
        }
        if (*data).first_event != 0 as ::core::ffi::c_int {
            let mut jobj_09_2: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            sfxMove();
            (*data).first_event -= 1 as ::core::ffi::c_int;
            mnEvent_RefreshRows((*data).first_event);
            mnEvent_ShowSelected(data, &raw mut jobj_09_2);
        }
        let mut __0: [::core::ffi::c_uchar; 4] = [0; 4];
    } else if inputs & MenuInput_Down as ::core::ffi::c_int as u64 != 0 {
        page = (*data).page;
        if (page as ::core::ffi::c_int) < 8 as ::core::ffi::c_int {
            let mut jobj_09_3: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            sfxMove();
            (*data).page = ((*data).page as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                as u8;
            mnEvent_SetPageCursorY(
                data,
                &raw mut down_jobj_0A,
                &raw mut down_jobj_0C,
                &raw mut down_jobj_0B,
            );
            mnEvent_ShowSelected(data, &raw mut jobj_09_3);
            return;
        }
        if (*data).first_event < mnEvent_8024CE74() {
            let mut jobj_09_4: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            sfxMove();
            (*data).first_event += 1 as ::core::ffi::c_int;
            mnEvent_RefreshRows((*data).first_event);
            mnEvent_ShowSelected(data, &raw mut jobj_09_4);
        }
    }
    let mut __1: [::core::ffi::c_uchar; 52] = [0; 52];
}
#[inline]
pub unsafe extern "C" fn GET_EVENTDATA(mut gobj: *mut HSD_GObj) -> *mut MnEventData {
    return (*gobj).user_data as *mut MnEventData;
}
pub unsafe extern "C" fn mnEvent_8024E1B4(mut gobj: *mut HSD_GObj) {
    let mut tree: *mut HSD_JObj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut tmp: *mut MnEventData = ::core::ptr::null_mut::<MnEventData>();
    let mut data: *mut MnEventData = GET_EVENTDATA(gobj);
    let mut iter: *mut MnEventData = ::core::ptr::null_mut::<MnEventData>();
    let mut i: ::core::ffi::c_int = 0;
    if mn_8022EC18(tree, &raw mut mnEvent_803EF74C, MOBJ_MASK)
        >= mnEvent_803EF74C.end_frame
    {
        tmp = data;
        iter = data;
        i = 0 as ::core::ffi::c_int;
        while i < 9 as ::core::ffi::c_int {
            if !(*iter).gobjs[0 as ::core::ffi::c_int as usize].is_null() {
                HSD_GObjFree((*tmp).gobjs[i as usize]);
                (*iter).gobjs[0 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                    HSD_GObj,
                >();
            }
            if !(*iter).texts[0 as ::core::ffi::c_int as usize].is_null() {
                HSD_SisLib_803A5CC4((*tmp).texts[i as usize]);
                (*iter).texts[0 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                    HSD_Text,
                >();
            }
            if !(*iter).icons[0 as ::core::ffi::c_int as usize].is_null() {
                HSD_SisLib_803A5CC4((*tmp).icons[i as usize]);
                (*iter).icons[0 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
                    HSD_Text,
                >();
            }
            iter = (iter as *mut u8).add(core::mem::size_of::<*mut HSD_GObj>())
                as *mut MnEventData;
            i += 1;
        }
        HSD_GObjFree(gobj);
    }
}
pub unsafe extern "C" fn mnEvent_8024E2A0(mut gobj: *mut HSD_GObj) {
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut tree: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut data: *mut MnEventData = (*gobj).user_data as *mut MnEventData;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    tree = (*gobj).hsd_obj as *mut HSD_JObj;
    if mn_804A04F0.cur_menu as ::core::ffi::c_int != 7 as ::core::ffi::c_int {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(mnEvent_8024E1B4 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
        HSD_SisLib_803A5CC4((*data).desc_text);
        HSD_SisLib_803A5CC4((*data).name_text);
    } else {
        lb_80011E24(tree, &raw mut jobj, &[Va::I32(1 as ::core::ffi::c_int), Va::I32(-(1 as ::core::ffi::c_int))]);
        mn_8022ED6C(jobj, &raw mut mnEvent_803EF758);
    };
}
pub unsafe extern "C" fn mnEvent_8024E34C(mut gobj: *mut HSD_GObj) {
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut tree: *mut HSD_JObj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut data: *mut MnEventData = (*gobj).user_data as *mut MnEventData;
    let mut _pad: [::core::ffi::c_uchar; 16] = [0; 16];
    if mn_804A04F0.cur_menu as ::core::ffi::c_int != 7 as ::core::ffi::c_int {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(mnEvent_8024E1B4 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
        HSD_SisLib_803A5CC4((*data).desc_text);
        HSD_SisLib_803A5CC4((*data).name_text);
    } else {
        let mut frame: ::core::ffi::c_float = mn_8022EC18(
            tree,
            &raw mut mnEvent_803EF740,
            MOBJ_MASK,
        );
        if frame == mnEvent_803EF740.end_frame {
            HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
            proc_0 = HSD_GObj_SetupProc(
                gobj,
                Some(mnEvent_8024E2A0 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
                0 as u8,
            );
            (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
        }
    };
}
#[inline]
pub unsafe extern "C" fn mnEvent_SetPageY(
    mut jobj_0B: *mut HSD_JObj,
    mut page: u8,
    mut y_a: f32,
    mut y_b: f32,
) {
    y_b = page as f32 * (y_b - y_a);
    HSD_JObjSetTranslateY(jobj_0B, y_b);
}
pub unsafe extern "C" fn mnEvent_8024E420(
    mut data: *mut MnEventData,
    mut event_idx: i32,
) {
    let mut max_events: i32 = 0;
    let mut i: ::core::ffi::c_int = 0;
    max_events = mnEvent_8024CE74();
    if event_idx < max_events {
        (*data).first_event = event_idx;
        (*data).page = 0 as u8;
    } else {
        (*data).first_event = max_events;
        (*data).page = (event_idx - max_events) as u8;
    }
    (*data).desc_text = ::core::ptr::null_mut::<HSD_Text>();
    (*data).name_text = ::core::ptr::null_mut::<HSD_Text>();
    i = 0 as ::core::ffi::c_int;
    while i < 9 as ::core::ffi::c_int {
        (*data).gobjs[i as usize] = ::core::ptr::null_mut::<HSD_GObj>();
        (*data).texts[i as usize] = ::core::ptr::null_mut::<HSD_Text>();
        (*data).icons[i as usize] = ::core::ptr::null_mut::<HSD_Text>();
        i += 1;
    }
}
pub unsafe extern "C" fn mnEvent_8024E524(mut event_idx: i32) {
    let mut page: u8 = 0;
    let mut jobj_0B: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut jobj_0C: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut jobj_0A: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut jobj_09: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut tree: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut user_data: *mut MnEventData = ::core::ptr::null_mut::<MnEventData>();
    let mut assets: *mut StaticModelDesc = ::core::ptr::null_mut::<StaticModelDesc>();
    let mut y_a: f32 = 0.;
    let mut y_b: f32 = 0.;
    assets = getMainConEv();
    gobj = GObj_Create(6 as u16, 7 as u8, 0x80 as u8);
    mnEvent_804D6C60 = gobj;
    tree = HSD_JObjLoadJoint((*assets).joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, tree as *mut ::core::ffi::c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
        ),
        4 as u8,
        0x80 as u32,
    );
    HSD_JObjAddAnimAll(
        tree,
        (*assets).animjoint,
        (*assets).matanim_joint,
        (*assets).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(tree, 0.0f32);
    HSD_JObjAnimAll(tree);
    user_data = HSD_MemAlloc(::core::mem::size_of::<MnEventData>() as i32)
        as *mut MnEventData;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char, &[]);
        __assert(
            b"src/melee/mn/mnevent.c\0" as *const u8 as *const ::core::ffi::c_char,
            680 as u32,
            b"user_data\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    mnEvent_8024E420(user_data, event_idx);
    GObj_InitUserData(
        gobj,
        0 as u8,
        Some(HSD_Free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        user_data as *mut ::core::ffi::c_void,
    );
    page = (*user_data).page;
    lb_80011E24(tree, &raw mut jobj_0A, &[Va::I32(0xa as ::core::ffi::c_int), Va::I32(-(1 as ::core::ffi::c_int))]);
    lb_80011E24(tree, &raw mut jobj_0C, &[Va::I32(0xc as ::core::ffi::c_int), Va::I32(-(1 as ::core::ffi::c_int))]);
    y_a = HSD_JObjGetTranslationY(jobj_0A);
    y_b = HSD_JObjGetTranslationY(jobj_0C);
    lb_80011E24(tree, &raw mut jobj_0B, &[Va::I32(0xb as ::core::ffi::c_int), Va::I32(-(1 as ::core::ffi::c_int))]);
    mnEvent_SetPageY(jobj_0B, page, y_a, y_b);
    proc_0 = HSD_GObj_SetupProc(
        gobj,
        Some(mnEvent_8024E34C as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
    mnEvent_RefreshRows((*user_data).first_event);
    mnEvent_ShowSelected(user_data, &raw mut jobj_09);
}
pub unsafe extern "C" fn mnEvent_8024E838(
    mut event_idx: ::core::ffi::c_int,
    mut first_time: bool,
) {
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut archive: *mut HSD_Archive = ::core::ptr::null_mut::<HSD_Archive>();
    mn_804D6BC8.cooldown = 5 as u16;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 7 as MenuKind8;
    mn_804A04F0.hovered_selection = 0 as u16;
    mnEvent_804D6C65 = event_idx as u8;
    if first_time {
        mnEvent_804D6C64 = 0x14 as u8;
    } else {
        mnEvent_804D6C64 = 0 as u8;
    }
    mnEvent_804D6C60 = ::core::ptr::null_mut::<HSD_GObj>();
    archive = mn_804D6BB8;
    lbArchive_LoadSections(
        archive,
        &raw mut MenMainConEv.joint as *mut *mut ::core::ffi::c_void,
        &[
            c"MenMainConEv_Top_joint".as_ptr().cast(),
            &raw mut MenMainConEv.animjoint as *mut ::core::ffi::c_void as *const ::core::ffi::c_void,
            c"MenMainConEv_Top_animjoint".as_ptr().cast(),
            &raw mut MenMainConEv.matanim_joint as *mut ::core::ffi::c_void as *const ::core::ffi::c_void,
            c"MenMainConEv_Top_matanim_joint".as_ptr().cast(),
            &raw mut MenMainConEv.shapeanim_joint as *mut ::core::ffi::c_void as *const ::core::ffi::c_void,
            c"MenMainConEv_Top_shapeanim_joint".as_ptr().cast(),
            &raw mut MenMainMarkEv.joint as *mut ::core::ffi::c_void as *const ::core::ffi::c_void,
            c"MenMainMarkEv_Top_joint".as_ptr().cast(),
            ::core::ptr::null(),
        ],
    );
    if !first_time {
        mnEvent_8024E524(event_idx as i32);
    }
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0 as u16, 1 as u8, 0x80 as u8),
        Some(mnEvent_8024D864 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
}
