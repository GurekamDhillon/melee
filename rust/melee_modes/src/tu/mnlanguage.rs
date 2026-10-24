//! src/melee/mn/mnlanguage.c: native port (c2rust-derived body; callees of unported TUs are `ext` TODO hooks).
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused_unsafe, unused_mut, unused_assignments, unused_variables, unused_parens, dead_code, static_mut_refs, clippy::all)]

use melee_types::*;
#[allow(unused_imports)] use self::ext::*;
#[allow(unused_imports)] use super::mnmain::{mn_80229624, mn_80229860, mn_80229894, mn_804A04F0, mn_804D6BB8, mn_804D6BC8};

pub type u8_0 = u8;
pub type u16_0 = u16;
pub type u32_0 = u32;
pub type u64_0 = u64;
pub type f32_0 = f32;
pub type f64_0 = f64;
pub const MenuInput_AButton: _MenuInput = 512;
pub const MenuInput_Back: _MenuInput = 32;
pub type _MenuInput = ::core::ffi::c_int;


#[inline]
pub unsafe extern "C" fn HSD_GObjGetUserData(
    mut gobj: *mut HSD_GObj,
) -> *mut ::core::ffi::c_void {
    return (*gobj).user_data;
}
#[inline]
pub unsafe extern "C" fn HSD_GObjGetHSDObj(
    mut gobj: *mut HSD_GObj,
) -> *mut ::core::ffi::c_void {
    return (*gobj).hsd_obj;
}
pub const HSD_GOBJ_CLASS_ITEM: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
#[inline]
pub unsafe extern "C" fn Menu_GetAllInputs() -> u64_0 {
    mn_804A04F0.buttons = mn_80229624(4 as u32_0) as u64_0;
    return mn_804A04F0.buttons;
}
#[inline]
pub unsafe extern "C" fn sfxBack() {
    lbAudioAx_80024030(0 as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn sfxMove() {
    lbAudioAx_80024030(2 as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn Menu_InitCenterText(mut menu: *mut Menu, mut val: s32) {
    let mut text: *mut HSD_Text = HSD_SisLib_803A5ACC(
        0 as ::core::ffi::c_int,
        1 as s32,
        -9.5f32,
        9.1f32,
        17.0f32,
        364.68332f32,
        38.38772f32,
    );
    (*menu).text = text;
    (*text).font_size.x = 0.0521f32 as f32_0;
    (*text).font_size.y = 0.0521f32 as f32_0;
    HSD_SisLib_803A6368(text, val);
}
pub static mut mn_gobj: *mut HSD_GObj = ::core::ptr::null::<HSD_GObj>() as *mut HSD_GObj;
pub static mut model_desc: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut lang_jobj_frames: [f32_0; 2] = [1.0f32, 0.0f32];
pub static mut mnLanguage_803EF5A0: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0.0f32,
    end_frame: 19.0f32,
    loop_frame: -0.1f32,
};
pub static mut mnLanguage_803EF5AC: AnimLoopSettings = AnimLoopSettings {
    start_frame: 20.0f32,
    end_frame: 29.0f32,
    loop_frame: -0.1f32,
};
pub unsafe extern "C" fn mnLanguage_8024BFE0(mut unused: *mut HSD_GObj) {
    let mut temp_r3: u64_0 = 0;
    let mut lang: u8_0 = 0;
    let mut user_data: *mut Menu = HSD_GObjGetUserData(mn_gobj) as *mut Menu;
    if mn_804D6BC8.cooldown as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        mn_804D6BC8.cooldown = (mn_804D6BC8.cooldown as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as u16_0;
        mn_804D6BC8.x2 = 0 as u16_0;
        mn_804D6BC8.x4 = 0 as ::core::ffi::c_int as s32;
        return;
    }
    temp_r3 = Menu_GetAllInputs();
    if temp_r3 & MenuInput_Back as ::core::ffi::c_int as u64_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_80229894(4 as s32, 4 as u16_0, 3 as s32);
        return;
    }
    if temp_r3 & MenuInput_AButton as ::core::ffi::c_int as u64_0 != 0 {
        user_data = HSD_GObjGetUserData(mn_gobj) as *mut Menu;
        if (*user_data).unk1 as ::core::ffi::c_int
            != (*user_data).cursor as ::core::ffi::c_int
        {
            lbLang_SetSavedLanguage((*user_data).cursor as enum_t);
            lbCardGame_SaveChanges();
            lbAudioAx_80027AB0(0xad as ::core::ffi::c_int);
            mn_80229860(GM_MENU as ::core::ffi::c_int as s8);
            mn_8022F1A8(1 as u16_0, 9 as u16_0);
            mn_804A04F0.cur_menu = 4 as MenuKind8;
            mn_804A04F0.hovered_selection = 4 as u16_0;
        }
    } else if (*user_data).unk2 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if temp_r3 & ((1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int) as u64_0 != 0
        {
            sfxMove();
            (*user_data).cursor = (if (*user_data).cursor as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as u8_0;
            lang = (*user_data).cursor;
            let mut result: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            let mut _pad: [::core::ffi::c_uchar; 4] = [0; 4];
            lb_80011E24((*mn_gobj).hsd_obj as *mut HSD_JObj, &raw mut result, &[(1 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
            HSD_JObjReqAnimAll(result, lang_jobj_frames[lang as usize]);
            mn_8022F3D8(
                result as *mut HSD_JObj,
                -(1 as ::core::ffi::c_int) as u8_0,
                JOBJ_MASK,
            );
            HSD_JObjAnimAll(result);
        } else if temp_r3
            & ((1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int) as u64_0 != 0
        {
            sfxMove();
            (*user_data).cursor = (if (*user_data).cursor as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as u8_0;
            lang = (*user_data).cursor;
            let mut result_0: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            lb_80011E24(HSD_GObjGetHSDObj(mn_gobj) as *mut HSD_JObj, &raw mut result_0, &[(1 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
            HSD_JObjReqAnimAll(result_0, lang_jobj_frames[lang as usize]);
            mn_8022F3D8(
                result_0 as *mut HSD_JObj,
                -(1 as ::core::ffi::c_int) as u8_0,
                JOBJ_MASK,
            );
            HSD_JObjAnimAll(result_0);
        }
    }
}
pub unsafe extern "C" fn fn_8024C210(mut gobj: *mut HSD_GObj) {
    let mut _pad: [::core::ffi::c_uchar; 4] = [0; 4];
    let mut jobj: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    if mn_8022EC18(jobj as *mut HSD_JObj, &raw mut mnLanguage_803EF5AC, MOBJ_MASK)
        >= mnLanguage_803EF5AC.end_frame
    {
        HSD_GObjFree(gobj);
    }
}
pub unsafe extern "C" fn fn_8024C270(mut gobj: *mut HSD_GObj) {
    let mut gobjproc: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut menu: *mut Menu = HSD_GObjGetUserData(gobj) as *mut Menu;
    let mut _pad: [::core::ffi::c_uchar; 12] = [0; 12];
    if mn_804A04F0.cur_menu as ::core::ffi::c_int != 23 as ::core::ffi::c_int {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        gobjproc = HSD_GObj_SetupProc(
            gobj,
            Some(fn_8024C210 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8_0,
        );
        (*gobjproc).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        HSD_SisLib_803A5CC4((*menu).text);
    }
}
pub unsafe extern "C" fn fn_8024C2E8(mut gobj: *mut HSD_GObj) {
    let mut gobjproc: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut menu: *mut Menu = (*gobj).user_data as *mut Menu;
    let mut jobj: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    let mut tmp: f32_0 = 0.;
    if mn_804A04F0.cur_menu as ::core::ffi::c_int != 23 as ::core::ffi::c_int {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        gobjproc = HSD_GObj_SetupProc(
            gobj,
            Some(fn_8024C210 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8_0,
        );
        (*gobjproc).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        HSD_SisLib_803A5CC4((*menu).text);
        return;
    }
    tmp = mn_8022EC18(jobj as *mut HSD_JObj, &raw mut mnLanguage_803EF5A0, MOBJ_MASK)
        as f32_0;
    if tmp == mnLanguage_803EF5A0.end_frame {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        gobjproc = HSD_GObj_SetupProc(
            gobj,
            Some(fn_8024C270 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8_0,
        );
        (*gobjproc).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        (*menu).unk2 = 1 as u8_0;
    }
}
pub unsafe extern "C" fn mnLanguage_8024C3C4(mut arg0: *mut HSD_GObj) {
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut gobjproc: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut user_data: *mut Menu = ::core::ptr::null_mut::<Menu>();
    let mut menu: *mut Menu = ::core::ptr::null_mut::<Menu>();
    let mut lang: u8_0 = 0;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    gobj = GObj_Create(
        HSD_GOBJ_CLASS_ITEM as u16_0,
        7 as u8_0,
        MOBJ_MASK as ::core::ffi::c_int as u8_0,
    );
    mn_gobj = gobj;
    jobj = HSD_JObjLoadJoint(model_desc.joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut ::core::ffi::c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
        ),
        4 as u8_0,
        MOBJ_MASK as ::core::ffi::c_int as u32_0,
    );
    HSD_JObjAddAnimAll(
        jobj,
        model_desc.animjoint,
        model_desc.matanim_joint,
        model_desc.shapeanim_joint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0f32);
    HSD_JObjAnimAll(jobj);
    user_data = HSD_MemAlloc(::core::mem::size_of::<Menu>() as ssize_t) as *mut Menu;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char, &[]);
        __assert(
            b"src/melee/mn/mnlanguage.c\0" as *const u8 as *const ::core::ffi::c_char,
            173 as u32_0,
            b"user_data\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    lang = lbLang_GetSavedLanguage() as u8_0;
    (*user_data).cursor = lang;
    (*user_data).unk1 = lang;
    (*user_data).text = ::core::ptr::null_mut::<HSD_Text>();
    (*user_data).unk2 = 0 as u8_0;
    GObj_InitUserData(
        gobj,
        0 as u8_0,
        Some(HSD_Free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        user_data as *mut ::core::ffi::c_void,
    );
    gobjproc = HSD_GObj_SetupProc(
        gobj,
        Some(fn_8024C2E8 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    (*gobjproc).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
    lang = (*user_data).cursor;
    let mut sp1C: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut __0: [::core::ffi::c_uchar; 8] = [0; 8];
    lb_80011E24(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, &raw mut sp1C, &[(1 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    HSD_JObjReqAnimAll(sp1C, lang_jobj_frames[lang as usize]);
    mn_8022F3D8(sp1C as *mut HSD_JObj, -(1 as ::core::ffi::c_int) as u8_0, JOBJ_MASK);
    HSD_JObjAnimAll(sp1C);
    HSD_JObjReqAnimAll(jobj, 0.0f32);
    mn_8022F3D8(jobj as *mut HSD_JObj, -(1 as ::core::ffi::c_int) as u8_0, MOBJ_MASK);
    HSD_JObjAnimAll(jobj);
    menu = HSD_GObjGetUserData(mn_gobj) as *mut Menu;
    if !(*menu).text.is_null() {
        HSD_SisLib_803A5CC4((*menu).text);
    }
    Menu_InitCenterText(menu, 0xbf as s32);
}
pub unsafe extern "C" fn mnLanguage_8024C5C0(mut gobj: *mut HSD_GObj) {
    let mut gobjproc: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut archive: *mut HSD_Archive = ::core::ptr::null_mut::<HSD_Archive>();
    mn_804D6BC8.cooldown = 5 as u16_0;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 23 as MenuKind8;
    mn_804A04F0.hovered_selection = 0 as u16_0;
    archive = mn_804D6BB8;
    lbArchive_LoadSections(archive, &raw mut model_desc.joint as *mut *mut ::core::ffi::c_void, &[(b"MenMainConLa_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut model_desc.animjoint) as usize, (b"MenMainConLa_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut model_desc.matanim_joint) as usize, (b"MenMainConLa_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut model_desc.shapeanim_joint) as usize, (b"MenMainConLa_Top_shapeanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (0 as ::core::ffi::c_int) as usize]);
    mnLanguage_8024C3C4(gobj);
    gobjproc = HSD_GObj_SetupProc(
        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
        Some(mnLanguage_8024BFE0 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    (*gobjproc).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
}
pub const NULL: ::core::ffi::c_long = 0 as ::core::ffi::c_long;

/// Callees in unported TUs: settable hooks named like the C function; unset they panic with a TODO.
pub mod ext {
    #![allow(non_snake_case, non_upper_case_globals, dead_code, unused_imports)]
    use melee_types::*;
    use super::*;
    macro_rules! hook {
        ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $h: Option<unsafe extern "C" fn($($t),*) $(-> $r)?> = None;
            pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                match $h {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
        ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $h: Option<unsafe extern "C" fn($($t),*) $(-> $r)?> = None;
            pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                match $h {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    hook!(H_OSReport, OSReport, (a0: *mut ::core::ffi::c_char, va: &[usize]));
    hook!(H___assert, __assert, (a0: *const ::core::ffi::c_char, a1: u32_0, a2: *const ::core::ffi::c_char) -> !);
    hook!(H_GObj_Create, GObj_Create, (a0: u16_0, a1: u8_0, a2: u8_0) -> *mut HSD_GObj);
    pub static mut H_HSD_GObj_JObjCallback: Option<unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: ::core::ffi::c_int)  {
                match H_HSD_GObj_JObjCallback {
                    Some(f) => f(a0, a1),
                    None => { unsafe { let _ = melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_lbAudioAx_80024030: Option<unsafe extern "C" fn(::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn lbAudioAx_80024030(a0: ::core::ffi::c_int)  {
                match H_lbAudioAx_80024030 {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0 as _); } return; },
                }
            }
    pub static mut H_lbAudioAx_80027AB0: Option<unsafe extern "C" fn(::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn lbAudioAx_80027AB0(a0: ::core::ffi::c_int)  {
                match H_lbAudioAx_80027AB0 {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80027AB0(a0 as _); } return; },
                }
            }
    hook!(H_lb_80011E24, lb_80011E24, (a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, va: &[usize]) -> ::core::ffi::c_int);
    pub static mut H_mn_8022EC18: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut AnimLoopSettings, HSD_TypeMask) -> ::core::ffi::c_float> = None;
            pub unsafe extern "C" fn mn_8022EC18(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings, a2: HSD_TypeMask) -> ::core::ffi::c_float {
                match H_mn_8022EC18 {
                    Some(f) => f(a0, a1, a2),
                    None => { return unsafe { crate::tu::mn_22EC::mn_8022EC18(a0 as _, a1 as _, a2 as _) as _ }; },
                }
            }
    pub static mut H_mn_8022F1A8: Option<unsafe extern "C" fn(u16_0, u16_0) > = None;
            pub unsafe extern "C" fn mn_8022F1A8(a0: u16_0, a1: u16_0)  {
                match H_mn_8022F1A8 {
                    Some(f) => f(a0, a1),
                    None => { unsafe { let _ = crate::tu::mn_22EC::mn_8022F1A8(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_mn_8022F3D8: Option<unsafe extern "C" fn(*mut HSD_JObj, u8_0, HSD_TypeMask) > = None;
            pub unsafe extern "C" fn mn_8022F3D8(a0: *mut HSD_JObj, a1: u8_0, a2: HSD_TypeMask)  {
                match H_mn_8022F3D8 {
                    Some(f) => f(a0, a1, a2),
                    None => { unsafe { let _ = crate::tu::mn_22EC::mn_8022F3D8(a0 as _, a1 as _, a2 as _); } return; },
                }
            }
    hook!(H_HSD_JObjReqAnimAll, HSD_JObjReqAnimAll, (a0: *mut HSD_JObj, a1: f32_0));
    pub static mut H_HSD_JObjLoadJoint: Option<unsafe extern "C" fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
            pub unsafe extern "C" fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj {
                match H_HSD_JObjLoadJoint {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ }; },
                }
            }
    hook!(H_HSD_JObjAddAnimAll, HSD_JObjAddAnimAll, (a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint));
    hook!(H_HSD_JObjAnimAll, HSD_JObjAnimAll, (a0: *mut HSD_JObj));
    pub static mut H_HSD_SisLib_803A6368: Option<unsafe extern "C" fn(*mut HSD_Text, s32) > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: s32)  {
                match H_HSD_SisLib_803A6368 {
                    Some(f) => f(a0, a1),
                    None => { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A6368(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_HSD_SisLib_803A5ACC: Option<unsafe extern "C" fn(::core::ffi::c_int, s32, f32_0, f32_0, f32_0, f32_0, f32_0) -> *mut HSD_Text> = None;
            pub unsafe extern "C" fn HSD_SisLib_803A5ACC(a0: ::core::ffi::c_int, a1: s32, a2: f32_0, a3: f32_0, a4: f32_0, a5: f32_0, a6: f32_0) -> *mut HSD_Text {
                match H_HSD_SisLib_803A5ACC {
                    Some(f) => f(a0, a1, a2, a3, a4, a5, a6),
                    None => { return unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ }; },
                }
            }
    pub static mut H_HSD_SisLib_803A5CC4: Option<unsafe extern "C" fn(*mut HSD_Text) > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text)  {
                match H_HSD_SisLib_803A5CC4 {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A5CC4(a0 as _); } return; },
                }
            }
    hook!(H_lbArchive_LoadSections, lbArchive_LoadSections, (a0: *mut HSD_Archive, a1: *mut *mut ::core::ffi::c_void, va: &[usize]));
    pub static mut H_lbCardGame_SaveChanges: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn lbCardGame_SaveChanges()  {
                match H_lbCardGame_SaveChanges {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::lbcardgame::lbCardGame_SaveChanges(); } return; },
                }
            }
    pub static mut H_lbLang_GetSavedLanguage: Option<unsafe extern "C" fn() -> enum_t> = None;
            pub unsafe extern "C" fn lbLang_GetSavedLanguage() -> enum_t {
                match H_lbLang_GetSavedLanguage {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lblanguage::lbLang_GetSavedLanguage() as _ }; },
                }
            }
    pub static mut H_lbLang_SetSavedLanguage: Option<unsafe extern "C" fn(enum_t) > = None;
            pub unsafe extern "C" fn lbLang_SetSavedLanguage(a0: enum_t)  {
                match H_lbLang_SetSavedLanguage {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = melee_runtime::tu::lblanguage::lbLang_SetSavedLanguage(a0 as _); } return; },
                }
            }
    hook!(H_GObj_SetupGXLink, GObj_SetupGXLink, (a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8_0, a3: u32_0));
    hook!(H_HSD_GObjObject_80390A70, HSD_GObjObject_80390A70, (a0: *mut HSD_GObj, a1: u8_0, a2: *mut ::core::ffi::c_void));
    hook!(H_HSD_GObjFree, HSD_GObjFree, (a0: *mut HSD_GObj));
    hook!(H_HSD_GObj_SetupProc, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8_0) -> *mut HSD_GObjProc);
    hook!(H_HSD_GObjProc_RemoveProc, HSD_GObjProc_RemoveProc, (a0: *mut HSD_GObjProc));
    hook!(H_GObj_InitUserData, GObj_InitUserData, (a0: *mut HSD_GObj, a1: u8_0, a2: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>, a3: *mut ::core::ffi::c_void));
    pub static mut H_HSD_Free: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) > = None;
            pub unsafe extern "C" fn HSD_Free(a0: *mut ::core::ffi::c_void)  {
                match H_HSD_Free {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = melee_runtime::tu::memory::HSD_Free(a0 as _); } return; },
                }
            }
    pub static mut H_HSD_MemAlloc: Option<unsafe extern "C" fn(ssize_t) -> *mut ::core::ffi::c_void> = None;
            pub unsafe extern "C" fn HSD_MemAlloc(a0: ssize_t) -> *mut ::core::ffi::c_void {
                match H_HSD_MemAlloc {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ }; },
                }
            }
    pub static mut HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_804D783C: s32 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8_0 = unsafe { core::mem::zeroed() };
}
