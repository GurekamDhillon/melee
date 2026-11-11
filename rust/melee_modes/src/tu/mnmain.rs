//! src/melee/mn/mnmain.c: native port (c2rust-derived body; callees of unported TUs are `ext` TODO hooks).
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused_unsafe, unused_mut, unused_assignments, unused_variables, unused_parens, dead_code, static_mut_refs, clippy::all)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::AnimLoopSettings as AnimLoopSettings_mn_ptr;
use melee_types::*;
#[allow(unused_imports)] use self::ext::*;
#[allow(unused_imports)] use super::mnhyaku::{mnHyaku_8024CD64};
#[allow(unused_imports)] use super::mninfo::{mnInfo_80252758};
#[allow(unused_imports)] use super::mninfobonus::{mnInfoBonus_80252F8C};
#[allow(unused_imports)] use super::mnlanguage::{mnLanguage_8024C5C0};
#[allow(unused_imports)] use super::mnmainrule::{mn_80231714};
#[allow(unused_imports)] use super::mnname::{mnName_8023AC40};

pub type u8_0 = u8;
pub type u16_0 = u16;
pub type u32_0 = u32;
pub type u64_0 = u64;
pub type f32_0 = f32;
pub type f64_0 = f64;
pub const MenuInput_Right: _MenuInput = 8;
pub const MenuInput_Left: _MenuInput = 4;
pub const MenuInput_Down: _MenuInput = 2;
pub const MenuInput_Up: _MenuInput = 1;
pub const MenuInput_YButton: _MenuInput = 2048;
pub const MenuInput_XButton: _MenuInput = 1024;
pub const MenuInput_RTrigger: _MenuInput = 128;
pub const MenuInput_LTrigger: _MenuInput = 64;
pub const MenuInput_Back: _MenuInput = 32;
pub const MenuInput_Confirm: _MenuInput = 16;
pub const MenuInput_StartButton: _MenuInput = 256;
pub const MenuInput_AButton: _MenuInput = 512;
pub const LbCardResult_Busy: C2RustUnnamed_13 = 11;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub type bool_0 = ::core::ffi::c_int;
pub type MainMenuData = _MainMenuData;
pub type MainMenuPanelData = _MainMenuPanelData;
pub type MenuFlow = _MenuFlow;
pub type MenuInputState = _MenuInputState;
pub type _MenuInput = ::core::ffi::c_int;
pub type C2RustUnnamed_13 = ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _MainMenuData {
    pub menu_kind: MenuKind8,
    pub hovered_selection: MenuKind8,
    pub state: MenuState8,
    pub pad_3: u8_0,
    pub tree: [*mut HSD_JObj; 42],
    pub description: *mut HSD_Text,
}
melee_core::gc_layout!(_MainMenuData, "_MainMenuData", size 0xB0, align 4, [
    menu_kind @ 0x0: 1 "MenuKind8",
    hovered_selection @ 0x1: 1 "MenuKind8",
    state @ 0x2: 1 "MenuState8",
    pad_3 @ 0x3: 1 "u8_0",
    tree @ 0x4: 168 "[*mut HSD_JObj; 42]",
    description @ 0xAC: 4 "*mut HSD_Text",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _MainMenuPanelData {
    pub cur_menu: MenuKind8,
    pub prev_menu: MenuKind8,
    pub x2: u8_0,
    pub state: MenuState8,
}
melee_core::gc_layout!(_MainMenuPanelData, "_MainMenuPanelData", size 0x4, align 1, [
    cur_menu @ 0x0: 1 "MenuKind8",
    prev_menu @ 0x1: 1 "MenuKind8",
    x2 @ 0x2: 1 "u8_0",
    state @ 0x3: 1 "MenuState8",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _MenuFlow {
    pub cur_menu: MenuKind8,
    pub prev_menu: MenuKind8,
    pub hovered_selection: u16_0,
    pub confirmed_selection: u8_0,
    pub pad_5: [u8_0; 3],
    pub buttons: u64_0,
    pub x10: u8_0,
    pub entering_menu: u8_0,
    pub light_lerp_frames: u8_0,
    pub pad_13: u8_0,
    pub light_color: *mut GXColor,
}
melee_core::gc_layout!(_MenuFlow, "_MenuFlow", size 0x18, align 8, [
    cur_menu @ 0x0: 1 "MenuKind8",
    prev_menu @ 0x1: 1 "MenuKind8",
    hovered_selection @ 0x2: 2 "u16_0",
    confirmed_selection @ 0x4: 1 "u8_0",
    pad_5 @ 0x5: 3 "[u8_0; 3]",
    buttons @ 0x8: 8 "u64_0",
    x10 @ 0x10: 1 "u8_0",
    entering_menu @ 0x11: 1 "u8_0",
    light_lerp_frames @ 0x12: 1 "u8_0",
    pad_13 @ 0x13: 1 "u8_0",
    light_color @ 0x14: 4 "*mut GXColor",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _MenuInputState {
    pub cooldown: u16_0,
    pub x2: u16_0,
    pub x4: s32,
}
melee_core::gc_layout!(_MenuInputState, "_MenuInputState", size 0x8, align 4, [
    cooldown @ 0x0: 2 "u16_0",
    x2 @ 0x2: 2 "u16_0",
    x4 @ 0x4: 4 "s32",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_12 {
    pub value: *mut HSD_JObj,
}
melee_core::gc_layout!(C2RustUnnamed_12, "C2RustUnnamed_12", size 0x4, align 4, [
    value @ 0x0: 4 "*mut HSD_JObj",
]);

#[inline]
pub unsafe extern "C" fn HSD_GObjGetUserData(
    mut gobj: *mut HSD_GObj,
) -> *mut ::core::ffi::c_void {
    melee_core::ct!("HSD_GObjGetUserData");
    return (*gobj).user_data;
}
#[inline]
pub unsafe extern "C" fn HSD_GObjGetHSDObj(
    mut gobj: *mut HSD_GObj,
) -> *mut ::core::ffi::c_void {
    melee_core::ct!("HSD_GObjGetHSDObj");
    return (*gobj).hsd_obj;
}
pub const JOBJ_HIDDEN: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 4 as ::core::ffi::c_int;
pub const JOBJ_MTX_DIRTY: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 6 as ::core::ffi::c_int;
pub const JOBJ_USER_DEF_MTX: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 23 as ::core::ffi::c_int;
pub const JOBJ_MTX_INDEP_SRT: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 25 as ::core::ffi::c_int;
#[inline]
pub unsafe extern "C" fn HSD_JObjMtxIsDirty(mut jobj: *mut HSD_JObj) -> bool_0 {
    melee_core::ct!("HSD_JObjMtxIsDirty");
    let mut result: bool_0 = 0;
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            228 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    result = false_0 as bool_0;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32_0 == 0
        && (*jobj).flags & JOBJ_MTX_DIRTY as u32_0 != 0
    {
        result = true_0 as bool_0;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn HSD_JObjSetRotationZ(mut jobj: *mut HSD_JObj, mut z: f32_0) {
    melee_core::ct!("HSD_JObjSetRotationZ");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            337 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if (*jobj).flags & ((1 as ::core::ffi::c_int) << 17 as ::core::ffi::c_int) as u32_0
        == 0
    {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            338 as u32_0,
            b"!(jobj->flags & JOBJ_USE_QUATERNION)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    (*jobj).rotate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
#[inline]
pub unsafe extern "C" fn HSD_JObjSetScaleY(mut jobj: *mut HSD_JObj, mut y: f32_0) {
    melee_core::ct!("HSD_JObjSetScaleY");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            420 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).scale.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
#[inline]
pub unsafe extern "C" fn HSD_JObjGetScaleY(mut jobj: *mut HSD_JObj) -> f32_0 {
    melee_core::ct!("HSD_JObjGetScaleY");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            469 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return (*jobj).scale.y;
}
#[inline]
pub unsafe extern "C" fn HSD_JObjSetTranslate(
    mut jobj: *mut HSD_JObj,
    mut translate: *mut Vec3,
) {
    melee_core::ct!("HSD_JObjSetTranslate");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            481 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if !translate.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            482 as u32_0,
            b"translate\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate = *translate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
#[inline]
pub unsafe extern "C" fn HSD_JObjGetTranslation(
    mut jobj: *mut HSD_JObj,
    mut translate: *mut Vec3,
) {
    melee_core::ct!("HSD_JObjGetTranslation");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            567 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if !translate.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            568 as u32_0,
            b"translate\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    *translate = (*jobj).translate;
}
#[inline]
pub unsafe extern "C" fn HSD_JObjGetTranslationX(mut jobj: *mut HSD_JObj) -> f32_0 {
    melee_core::ct!("HSD_JObjGetTranslationX");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            582 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return (*jobj).translate.x;
}
#[inline]
pub unsafe extern "C" fn HSD_JObjAddScaleY(
    mut jobj: *mut HSD_JObj,
    mut y: ::core::ffi::c_float,
) {
    melee_core::ct!("HSD_JObjAddScaleY");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            645 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).scale.y += y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
#[inline]
pub unsafe extern "C" fn HSD_JObjAddTranslationX(
    mut jobj: *mut HSD_JObj,
    mut x: ::core::ffi::c_float,
) {
    melee_core::ct!("HSD_JObjAddTranslationX");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            663 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate.x += x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
#[inline]
pub unsafe extern "C" fn HSD_JObjAddTranslationY(
    mut jobj: *mut HSD_JObj,
    mut y: ::core::ffi::c_float,
) {
    melee_core::ct!("HSD_JObjAddTranslationY");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            672 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate.y += y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
#[inline]
pub unsafe extern "C" fn HSD_JObjAddTranslationZ(
    mut jobj: *mut HSD_JObj,
    mut z: ::core::ffi::c_float,
) {
    melee_core::ct!("HSD_JObjAddTranslationZ");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            690 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate.z += z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
#[inline]
pub unsafe extern "C" fn Menu_DecrementAnimTimer() {
    melee_core::ct!("Menu_DecrementAnimTimer");
    mn_804D6BC8.cooldown = mn_804D6BC8.cooldown.wrapping_sub(1);
    mn_804D6BC8.x2 = 0 as u16_0;
    mn_804D6BC8.x4 = 0 as ::core::ffi::c_int as s32;
}
#[inline]
pub unsafe extern "C" fn sfxBack() {
    melee_core::ct!("sfxBack");
    lbAudioAx_80024030(0 as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn sfxForward() {
    melee_core::ct!("sfxForward");
    lbAudioAx_80024030(1 as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn sfxMove() {
    melee_core::ct!("sfxMove");
    lbAudioAx_80024030(2 as ::core::ffi::c_int);
}
pub const PAD_TRIGGER_R: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 5 as ::core::ffi::c_int;
pub const PAD_TRIGGER_L: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 6 as ::core::ffi::c_int;
pub const PAD_BUTTON_A: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 8 as ::core::ffi::c_int;
pub const PAD_BUTTON_X: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 10 as ::core::ffi::c_int;
pub const PAD_BUTTON_Y: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 11 as ::core::ffi::c_int;
pub const PAD_BUTTON_START: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 12 as ::core::ffi::c_int;
pub const PAD_CONFIRM: ::core::ffi::c_ulonglong = (1 as ::core::ffi::c_ulonglong)
    << 32 as ::core::ffi::c_int;
pub const PAD_CANCEL: ::core::ffi::c_ulonglong = (1 as ::core::ffi::c_ulonglong)
    << 33 as ::core::ffi::c_int;
pub const PAD_ANY_UP: ::core::ffi::c_ulonglong = (1 as ::core::ffi::c_ulonglong)
    << 36 as ::core::ffi::c_int;
pub const PAD_ANY_DOWN: ::core::ffi::c_ulonglong = (1 as ::core::ffi::c_ulonglong)
    << 37 as ::core::ffi::c_int;
pub const PAD_ANY_LEFT: ::core::ffi::c_ulonglong = (1 as ::core::ffi::c_ulonglong)
    << 38 as ::core::ffi::c_int;
pub const PAD_ANY_RIGHT: ::core::ffi::c_ulonglong = (1 as ::core::ffi::c_ulonglong)
    << 39 as ::core::ffi::c_int;
pub static mut mn_804D6BA8: *mut HSD_GObj = ::core::ptr::null::<HSD_GObj>() as *mut HSD_GObj;
pub static mut mn_804D6BAC: *mut HSD_GObj = ::core::ptr::null::<HSD_GObj>() as *mut HSD_GObj;
pub static mut mn_804D6BB0: *mut HSD_GObj = ::core::ptr::null::<HSD_GObj>() as *mut HSD_GObj;
pub static mut mn_804D6BC8: MenuInputState = _MenuInputState {
    cooldown: 0,
    x2: 0,
    x4: 0,
};
pub static mut MenMain_cam: *mut HSD_CObjDesc = ::core::ptr::null::<HSD_CObjDesc>()
    as *mut HSD_CObjDesc;
pub static mut MenMain_fog: *mut HSD_FogDesc = ::core::ptr::null::<HSD_FogDesc>()
    as *mut HSD_FogDesc;
pub static mut MenMain_lights: *mut *mut LightList = ::core::ptr::null::<
    *mut LightList,
>() as *mut *mut LightList;
pub static mut mn_804D6BB8: *mut HSD_Archive = ::core::ptr::null::<HSD_Archive>()
    as *mut HSD_Archive;
pub static mut mn_804D6BB5: u8_0 = 0;
pub static mut mn_804D6BB4: u8_0 = 0;
pub static mut mn_804A04F0: MenuFlow = _MenuFlow {
    cur_menu: 0,
    prev_menu: 0,
    hovered_selection: 0,
    confirmed_selection: 0,
    pad_5: [0; 3],
    buttons: 0,
    x10: 0,
    entering_menu: 0,
    light_lerp_frames: 0,
    pad_13: 0,
    light_color: ::core::ptr::null::<GXColor>() as *mut GXColor,
};
pub static mut mn_803EAE68: [u16_0; 10] = [
    0x4 as ::core::ffi::c_int as u16_0,
    0x5 as ::core::ffi::c_int as u16_0,
    0x6 as ::core::ffi::c_int as u16_0,
    0x7 as ::core::ffi::c_int as u16_0,
    0x8 as ::core::ffi::c_int as u16_0,
    0x9 as ::core::ffi::c_int as u16_0,
    0xa as ::core::ffi::c_int as u16_0,
    0xb as ::core::ffi::c_int as u16_0,
    0xc as ::core::ffi::c_int as u16_0,
    0xd as ::core::ffi::c_int as u16_0,
];
pub static mut mn_803EAE7C: [u16_0; 8] = [
    0x2 as ::core::ffi::c_int as u16_0,
    0x3 as ::core::ffi::c_int as u16_0,
    0x4 as ::core::ffi::c_int as u16_0,
    0x9 as ::core::ffi::c_int as u16_0,
    0xb as ::core::ffi::c_int as u16_0,
    0xc as ::core::ffi::c_int as u16_0,
    0xd as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
];
pub static mut mn_803EAE8C: [[AnimLoopSettings; 3]; 34] = [
    [
        AnimLoopSettings {
            start_frame: 0 as ::core::ffi::c_int as f32_0,
            end_frame: 99 as ::core::ffi::c_int as f32_0,
            loop_frame: 0 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 0 as ::core::ffi::c_int as f32_0,
            end_frame: 0 as ::core::ffi::c_int as f32_0,
            loop_frame: 0 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 100 as ::core::ffi::c_int as f32_0,
            end_frame: 199 as ::core::ffi::c_int as f32_0,
            loop_frame: 100 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 200 as ::core::ffi::c_int as f32_0,
            end_frame: 249 as ::core::ffi::c_int as f32_0,
            loop_frame: 200 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 250 as ::core::ffi::c_int as f32_0,
            end_frame: 299 as ::core::ffi::c_int as f32_0,
            loop_frame: 250 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 300 as ::core::ffi::c_int as f32_0,
            end_frame: 399 as ::core::ffi::c_int as f32_0,
            loop_frame: 300 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 400 as ::core::ffi::c_int as f32_0,
            end_frame: 449 as ::core::ffi::c_int as f32_0,
            loop_frame: 400 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 450 as ::core::ffi::c_int as f32_0,
            end_frame: 499 as ::core::ffi::c_int as f32_0,
            loop_frame: 450 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 500 as ::core::ffi::c_int as f32_0,
            end_frame: 599 as ::core::ffi::c_int as f32_0,
            loop_frame: 500 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 600 as ::core::ffi::c_int as f32_0,
            end_frame: 649 as ::core::ffi::c_int as f32_0,
            loop_frame: 600 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 650 as ::core::ffi::c_int as f32_0,
            end_frame: 699 as ::core::ffi::c_int as f32_0,
            loop_frame: 650 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 700 as ::core::ffi::c_int as f32_0,
            end_frame: 799 as ::core::ffi::c_int as f32_0,
            loop_frame: 700 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 800 as ::core::ffi::c_int as f32_0,
            end_frame: 849 as ::core::ffi::c_int as f32_0,
            loop_frame: 800 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 850 as ::core::ffi::c_int as f32_0,
            end_frame: 899 as ::core::ffi::c_int as f32_0,
            loop_frame: 850 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 900 as ::core::ffi::c_int as f32_0,
            end_frame: 999 as ::core::ffi::c_int as f32_0,
            loop_frame: 900 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 1000 as ::core::ffi::c_int as f32_0,
            end_frame: 1049 as ::core::ffi::c_int as f32_0,
            loop_frame: 1000 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 1050 as ::core::ffi::c_int as f32_0,
            end_frame: 1099 as ::core::ffi::c_int as f32_0,
            loop_frame: 1050 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 1100 as ::core::ffi::c_int as f32_0,
            end_frame: 1199 as ::core::ffi::c_int as f32_0,
            loop_frame: 1100 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 1200 as ::core::ffi::c_int as f32_0,
            end_frame: 1249 as ::core::ffi::c_int as f32_0,
            loop_frame: 1220 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 1250 as ::core::ffi::c_int as f32_0,
            end_frame: 1299 as ::core::ffi::c_int as f32_0,
            loop_frame: 1270 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 1300 as ::core::ffi::c_int as f32_0,
            end_frame: 1399 as ::core::ffi::c_int as f32_0,
            loop_frame: 1320 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 1400 as ::core::ffi::c_int as f32_0,
            end_frame: 1449 as ::core::ffi::c_int as f32_0,
            loop_frame: 1420 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 1450 as ::core::ffi::c_int as f32_0,
            end_frame: 1499 as ::core::ffi::c_int as f32_0,
            loop_frame: 1470 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 1500 as ::core::ffi::c_int as f32_0,
            end_frame: 1599 as ::core::ffi::c_int as f32_0,
            loop_frame: 1520 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 1600 as ::core::ffi::c_int as f32_0,
            end_frame: 1649 as ::core::ffi::c_int as f32_0,
            loop_frame: 1620 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 1650 as ::core::ffi::c_int as f32_0,
            end_frame: 1699 as ::core::ffi::c_int as f32_0,
            loop_frame: 1670 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 1700 as ::core::ffi::c_int as f32_0,
            end_frame: 1799 as ::core::ffi::c_int as f32_0,
            loop_frame: 1720 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 1800 as ::core::ffi::c_int as f32_0,
            end_frame: 1849 as ::core::ffi::c_int as f32_0,
            loop_frame: 1820 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 1850 as ::core::ffi::c_int as f32_0,
            end_frame: 1899 as ::core::ffi::c_int as f32_0,
            loop_frame: 1870 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 1900 as ::core::ffi::c_int as f32_0,
            end_frame: 1999 as ::core::ffi::c_int as f32_0,
            loop_frame: 1920 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 2000 as ::core::ffi::c_int as f32_0,
            end_frame: 2049 as ::core::ffi::c_int as f32_0,
            loop_frame: 2020 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 2050 as ::core::ffi::c_int as f32_0,
            end_frame: 2099 as ::core::ffi::c_int as f32_0,
            loop_frame: 2070 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 2100 as ::core::ffi::c_int as f32_0,
            end_frame: 2199 as ::core::ffi::c_int as f32_0,
            loop_frame: 2120 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 2400 as ::core::ffi::c_int as f32_0,
            end_frame: 2449 as ::core::ffi::c_int as f32_0,
            loop_frame: 2420 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 2450 as ::core::ffi::c_int as f32_0,
            end_frame: 2499 as ::core::ffi::c_int as f32_0,
            loop_frame: 2470 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 2500 as ::core::ffi::c_int as f32_0,
            end_frame: 2599 as ::core::ffi::c_int as f32_0,
            loop_frame: 2520 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 2600 as ::core::ffi::c_int as f32_0,
            end_frame: 2649 as ::core::ffi::c_int as f32_0,
            loop_frame: 2620 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 2650 as ::core::ffi::c_int as f32_0,
            end_frame: 2699 as ::core::ffi::c_int as f32_0,
            loop_frame: 2670 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 2700 as ::core::ffi::c_int as f32_0,
            end_frame: 2799 as ::core::ffi::c_int as f32_0,
            loop_frame: 2720 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 2800 as ::core::ffi::c_int as f32_0,
            end_frame: 2849 as ::core::ffi::c_int as f32_0,
            loop_frame: 2820 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 2850 as ::core::ffi::c_int as f32_0,
            end_frame: 2899 as ::core::ffi::c_int as f32_0,
            loop_frame: 2870 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 2900 as ::core::ffi::c_int as f32_0,
            end_frame: 2999 as ::core::ffi::c_int as f32_0,
            loop_frame: 2920 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 3000 as ::core::ffi::c_int as f32_0,
            end_frame: 3049 as ::core::ffi::c_int as f32_0,
            loop_frame: 3020 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3050 as ::core::ffi::c_int as f32_0,
            end_frame: 3099 as ::core::ffi::c_int as f32_0,
            loop_frame: 3070 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3100 as ::core::ffi::c_int as f32_0,
            end_frame: 3199 as ::core::ffi::c_int as f32_0,
            loop_frame: 3120 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 6200 as ::core::ffi::c_int as f32_0,
            end_frame: 6249 as ::core::ffi::c_int as f32_0,
            loop_frame: 6220 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 6250 as ::core::ffi::c_int as f32_0,
            end_frame: 6299 as ::core::ffi::c_int as f32_0,
            loop_frame: 6270 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 6300 as ::core::ffi::c_int as f32_0,
            end_frame: 6399 as ::core::ffi::c_int as f32_0,
            loop_frame: 6320 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 6000 as ::core::ffi::c_int as f32_0,
            end_frame: 6049 as ::core::ffi::c_int as f32_0,
            loop_frame: 6020 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 6050 as ::core::ffi::c_int as f32_0,
            end_frame: 6099 as ::core::ffi::c_int as f32_0,
            loop_frame: 6070 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 6100 as ::core::ffi::c_int as f32_0,
            end_frame: 6199 as ::core::ffi::c_int as f32_0,
            loop_frame: 6120 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 6400 as ::core::ffi::c_int as f32_0,
            end_frame: 6449 as ::core::ffi::c_int as f32_0,
            loop_frame: 6420 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 6450 as ::core::ffi::c_int as f32_0,
            end_frame: 6499 as ::core::ffi::c_int as f32_0,
            loop_frame: 6470 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 6500 as ::core::ffi::c_int as f32_0,
            end_frame: 6599 as ::core::ffi::c_int as f32_0,
            loop_frame: 6520 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 3000 as ::core::ffi::c_int as f32_0,
            end_frame: 3049 as ::core::ffi::c_int as f32_0,
            loop_frame: 3020 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3050 as ::core::ffi::c_int as f32_0,
            end_frame: 3099 as ::core::ffi::c_int as f32_0,
            loop_frame: 3070 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3100 as ::core::ffi::c_int as f32_0,
            end_frame: 3199 as ::core::ffi::c_int as f32_0,
            loop_frame: 3120 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 3200 as ::core::ffi::c_int as f32_0,
            end_frame: 3249 as ::core::ffi::c_int as f32_0,
            loop_frame: 3220 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3250 as ::core::ffi::c_int as f32_0,
            end_frame: 3299 as ::core::ffi::c_int as f32_0,
            loop_frame: 3270 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3300 as ::core::ffi::c_int as f32_0,
            end_frame: 3399 as ::core::ffi::c_int as f32_0,
            loop_frame: 3320 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 3400 as ::core::ffi::c_int as f32_0,
            end_frame: 3449 as ::core::ffi::c_int as f32_0,
            loop_frame: 3420 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3450 as ::core::ffi::c_int as f32_0,
            end_frame: 3499 as ::core::ffi::c_int as f32_0,
            loop_frame: 3470 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3500 as ::core::ffi::c_int as f32_0,
            end_frame: 3599 as ::core::ffi::c_int as f32_0,
            loop_frame: 3520 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 3600 as ::core::ffi::c_int as f32_0,
            end_frame: 3649 as ::core::ffi::c_int as f32_0,
            loop_frame: 3620 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3650 as ::core::ffi::c_int as f32_0,
            end_frame: 3699 as ::core::ffi::c_int as f32_0,
            loop_frame: 3670 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3700 as ::core::ffi::c_int as f32_0,
            end_frame: 3799 as ::core::ffi::c_int as f32_0,
            loop_frame: 3720 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 3800 as ::core::ffi::c_int as f32_0,
            end_frame: 3849 as ::core::ffi::c_int as f32_0,
            loop_frame: 3820 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3850 as ::core::ffi::c_int as f32_0,
            end_frame: 3899 as ::core::ffi::c_int as f32_0,
            loop_frame: 3870 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 3900 as ::core::ffi::c_int as f32_0,
            end_frame: 3999 as ::core::ffi::c_int as f32_0,
            loop_frame: 3920 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 4000 as ::core::ffi::c_int as f32_0,
            end_frame: 4049 as ::core::ffi::c_int as f32_0,
            loop_frame: 4020 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 4050 as ::core::ffi::c_int as f32_0,
            end_frame: 4099 as ::core::ffi::c_int as f32_0,
            loop_frame: 4070 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 4100 as ::core::ffi::c_int as f32_0,
            end_frame: 4199 as ::core::ffi::c_int as f32_0,
            loop_frame: 4120 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 4200 as ::core::ffi::c_int as f32_0,
            end_frame: 4249 as ::core::ffi::c_int as f32_0,
            loop_frame: 4220 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 4250 as ::core::ffi::c_int as f32_0,
            end_frame: 4299 as ::core::ffi::c_int as f32_0,
            loop_frame: 4270 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 4300 as ::core::ffi::c_int as f32_0,
            end_frame: 4399 as ::core::ffi::c_int as f32_0,
            loop_frame: 4320 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 4400 as ::core::ffi::c_int as f32_0,
            end_frame: 4449 as ::core::ffi::c_int as f32_0,
            loop_frame: 4420 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 4450 as ::core::ffi::c_int as f32_0,
            end_frame: 4499 as ::core::ffi::c_int as f32_0,
            loop_frame: 4470 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 4500 as ::core::ffi::c_int as f32_0,
            end_frame: 4599 as ::core::ffi::c_int as f32_0,
            loop_frame: 4520 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 4600 as ::core::ffi::c_int as f32_0,
            end_frame: 4649 as ::core::ffi::c_int as f32_0,
            loop_frame: 4620 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 4650 as ::core::ffi::c_int as f32_0,
            end_frame: 4699 as ::core::ffi::c_int as f32_0,
            loop_frame: 4670 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 4700 as ::core::ffi::c_int as f32_0,
            end_frame: 4799 as ::core::ffi::c_int as f32_0,
            loop_frame: 4720 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 4800 as ::core::ffi::c_int as f32_0,
            end_frame: 4849 as ::core::ffi::c_int as f32_0,
            loop_frame: 4820 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 4850 as ::core::ffi::c_int as f32_0,
            end_frame: 4899 as ::core::ffi::c_int as f32_0,
            loop_frame: 4870 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 4900 as ::core::ffi::c_int as f32_0,
            end_frame: 4999 as ::core::ffi::c_int as f32_0,
            loop_frame: 4920 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 5000 as ::core::ffi::c_int as f32_0,
            end_frame: 5049 as ::core::ffi::c_int as f32_0,
            loop_frame: 5020 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 5050 as ::core::ffi::c_int as f32_0,
            end_frame: 5099 as ::core::ffi::c_int as f32_0,
            loop_frame: 5070 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 5100 as ::core::ffi::c_int as f32_0,
            end_frame: 5199 as ::core::ffi::c_int as f32_0,
            loop_frame: 5120 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 5200 as ::core::ffi::c_int as f32_0,
            end_frame: 5249 as ::core::ffi::c_int as f32_0,
            loop_frame: 5220 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 5250 as ::core::ffi::c_int as f32_0,
            end_frame: 5299 as ::core::ffi::c_int as f32_0,
            loop_frame: 5270 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 5300 as ::core::ffi::c_int as f32_0,
            end_frame: 5399 as ::core::ffi::c_int as f32_0,
            loop_frame: 5320 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 7400 as ::core::ffi::c_int as f32_0,
            end_frame: 7449 as ::core::ffi::c_int as f32_0,
            loop_frame: 7420 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 7450 as ::core::ffi::c_int as f32_0,
            end_frame: 7499 as ::core::ffi::c_int as f32_0,
            loop_frame: 7470 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 7500 as ::core::ffi::c_int as f32_0,
            end_frame: 7599 as ::core::ffi::c_int as f32_0,
            loop_frame: 7520 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 7600 as ::core::ffi::c_int as f32_0,
            end_frame: 7649 as ::core::ffi::c_int as f32_0,
            loop_frame: 7620 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 7650 as ::core::ffi::c_int as f32_0,
            end_frame: 7699 as ::core::ffi::c_int as f32_0,
            loop_frame: 7670 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 7700 as ::core::ffi::c_int as f32_0,
            end_frame: 7799 as ::core::ffi::c_int as f32_0,
            loop_frame: 7720 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 7800 as ::core::ffi::c_int as f32_0,
            end_frame: 7849 as ::core::ffi::c_int as f32_0,
            loop_frame: 7820 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 7850 as ::core::ffi::c_int as f32_0,
            end_frame: 7899 as ::core::ffi::c_int as f32_0,
            loop_frame: 7870 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 7900 as ::core::ffi::c_int as f32_0,
            end_frame: 7999 as ::core::ffi::c_int as f32_0,
            loop_frame: 7920 as ::core::ffi::c_int as f32_0,
        },
    ],
    [
        AnimLoopSettings {
            start_frame: 5800 as ::core::ffi::c_int as f32_0,
            end_frame: 5849 as ::core::ffi::c_int as f32_0,
            loop_frame: 5820 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 5850 as ::core::ffi::c_int as f32_0,
            end_frame: 5899 as ::core::ffi::c_int as f32_0,
            loop_frame: 5870 as ::core::ffi::c_int as f32_0,
        },
        AnimLoopSettings {
            start_frame: 5900 as ::core::ffi::c_int as f32_0,
            end_frame: 5999 as ::core::ffi::c_int as f32_0,
            loop_frame: 5920 as ::core::ffi::c_int as f32_0,
        },
    ],
];
pub static mut mn_803EB354: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0 as ::core::ffi::c_int as f32_0,
    end_frame: 799 as ::core::ffi::c_int as f32_0,
    loop_frame: 0 as ::core::ffi::c_int as f32_0,
};
pub static mut mn_803EB360: [AnimLoopSettings; 2] = [
    AnimLoopSettings {
        start_frame: 0 as ::core::ffi::c_int as f32_0,
        end_frame: 49 as ::core::ffi::c_int as f32_0,
        loop_frame: 0 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 50 as ::core::ffi::c_int as f32_0,
        end_frame: 99 as ::core::ffi::c_int as f32_0,
        loop_frame: 50 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB378: [AnimLoopSettings; 2] = [
    AnimLoopSettings {
        start_frame: 0 as ::core::ffi::c_int as f32_0,
        end_frame: 49 as ::core::ffi::c_int as f32_0,
        loop_frame: -0.1f64 as f32_0,
    },
    AnimLoopSettings {
        start_frame: 50 as ::core::ffi::c_int as f32_0,
        end_frame: 250 as ::core::ffi::c_int as f32_0,
        loop_frame: 50 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB390: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0 as ::core::ffi::c_int as f32_0,
    end_frame: 10 as ::core::ffi::c_int as f32_0,
    loop_frame: -0.1f64 as f32_0,
};
pub static mut mn_803EB39C: AnimLoopSettings = AnimLoopSettings {
    start_frame: 320 as ::core::ffi::c_int as f32_0,
    end_frame: 339 as ::core::ffi::c_int as f32_0,
    loop_frame: -0.1f64 as f32_0,
};
pub static mut mn_803EB3A8: AnimLoopSettings = AnimLoopSettings {
    start_frame: 380 as ::core::ffi::c_int as f32_0,
    end_frame: 399 as ::core::ffi::c_int as f32_0,
    loop_frame: -0.1f64 as f32_0,
};
pub static mut mn_803EB3B4: AnimLoopSettings = AnimLoopSettings {
    start_frame: 360 as ::core::ffi::c_int as f32_0,
    end_frame: 379 as ::core::ffi::c_int as f32_0,
    loop_frame: -0.1f64 as f32_0,
};
pub static mut mn_803EB3C0: AnimLoopSettings = AnimLoopSettings {
    start_frame: 340 as ::core::ffi::c_int as f32_0,
    end_frame: 359 as ::core::ffi::c_int as f32_0,
    loop_frame: -0.1f64 as f32_0,
};
pub static mut mn_803EB3FC: [AnimLoopSettings; 5] = [
    AnimLoopSettings {
        start_frame: 0 as ::core::ffi::c_int as f32_0,
        end_frame: 49 as ::core::ffi::c_int as f32_0,
        loop_frame: 20 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 50 as ::core::ffi::c_int as f32_0,
        end_frame: 99 as ::core::ffi::c_int as f32_0,
        loop_frame: 70 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 100 as ::core::ffi::c_int as f32_0,
        end_frame: 149 as ::core::ffi::c_int as f32_0,
        loop_frame: 120 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 150 as ::core::ffi::c_int as f32_0,
        end_frame: 199 as ::core::ffi::c_int as f32_0,
        loop_frame: 170 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 200 as ::core::ffi::c_int as f32_0,
        end_frame: 249 as ::core::ffi::c_int as f32_0,
        loop_frame: 220 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB438: AnimLoopSettings = AnimLoopSettings {
    start_frame: 3550 as ::core::ffi::c_int as f32_0,
    end_frame: 3599 as ::core::ffi::c_int as f32_0,
    loop_frame: 3570 as ::core::ffi::c_int as f32_0,
};
pub static mut mn_803EB444: [AnimLoopSettings; 5] = [
    AnimLoopSettings {
        start_frame: 400 as ::core::ffi::c_int as f32_0,
        end_frame: 449 as ::core::ffi::c_int as f32_0,
        loop_frame: 420 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 450 as ::core::ffi::c_int as f32_0,
        end_frame: 499 as ::core::ffi::c_int as f32_0,
        loop_frame: 470 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 500 as ::core::ffi::c_int as f32_0,
        end_frame: 549 as ::core::ffi::c_int as f32_0,
        loop_frame: 520 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 550 as ::core::ffi::c_int as f32_0,
        end_frame: 599 as ::core::ffi::c_int as f32_0,
        loop_frame: 570 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 600 as ::core::ffi::c_int as f32_0,
        end_frame: 649 as ::core::ffi::c_int as f32_0,
        loop_frame: 620 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB480: AnimLoopSettings = AnimLoopSettings {
    start_frame: 3500 as ::core::ffi::c_int as f32_0,
    end_frame: 3549 as ::core::ffi::c_int as f32_0,
    loop_frame: 3520 as ::core::ffi::c_int as f32_0,
};
pub static mut mn_803EB48C: [AnimLoopSettings; 5] = [
    AnimLoopSettings {
        start_frame: 700 as ::core::ffi::c_int as f32_0,
        end_frame: 749 as ::core::ffi::c_int as f32_0,
        loop_frame: 720 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 750 as ::core::ffi::c_int as f32_0,
        end_frame: 799 as ::core::ffi::c_int as f32_0,
        loop_frame: 770 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 800 as ::core::ffi::c_int as f32_0,
        end_frame: 849 as ::core::ffi::c_int as f32_0,
        loop_frame: 820 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 850 as ::core::ffi::c_int as f32_0,
        end_frame: 899 as ::core::ffi::c_int as f32_0,
        loop_frame: 870 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 900 as ::core::ffi::c_int as f32_0,
        end_frame: 949 as ::core::ffi::c_int as f32_0,
        loop_frame: 920 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB4C8: [AnimLoopSettings; 4] = [
    AnimLoopSettings {
        start_frame: 1000 as ::core::ffi::c_int as f32_0,
        end_frame: 1049 as ::core::ffi::c_int as f32_0,
        loop_frame: 1020 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1050 as ::core::ffi::c_int as f32_0,
        end_frame: 1099 as ::core::ffi::c_int as f32_0,
        loop_frame: 1070 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1100 as ::core::ffi::c_int as f32_0,
        end_frame: 1149 as ::core::ffi::c_int as f32_0,
        loop_frame: 1120 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1150 as ::core::ffi::c_int as f32_0,
        end_frame: 1199 as ::core::ffi::c_int as f32_0,
        loop_frame: 1170 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB4F8: [AnimLoopSettings; 6] = [
    AnimLoopSettings {
        start_frame: 1300 as ::core::ffi::c_int as f32_0,
        end_frame: 1349 as ::core::ffi::c_int as f32_0,
        loop_frame: 1320 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1350 as ::core::ffi::c_int as f32_0,
        end_frame: 1399 as ::core::ffi::c_int as f32_0,
        loop_frame: 1370 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1400 as ::core::ffi::c_int as f32_0,
        end_frame: 1449 as ::core::ffi::c_int as f32_0,
        loop_frame: 1420 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1450 as ::core::ffi::c_int as f32_0,
        end_frame: 1499 as ::core::ffi::c_int as f32_0,
        loop_frame: 1470 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1500 as ::core::ffi::c_int as f32_0,
        end_frame: 1549 as ::core::ffi::c_int as f32_0,
        loop_frame: 1520 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1550 as ::core::ffi::c_int as f32_0,
        end_frame: 1599 as ::core::ffi::c_int as f32_0,
        loop_frame: 1570 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB540: [AnimLoopSettings; 5] = [
    AnimLoopSettings {
        start_frame: 1600 as ::core::ffi::c_int as f32_0,
        end_frame: 1649 as ::core::ffi::c_int as f32_0,
        loop_frame: 1620 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1650 as ::core::ffi::c_int as f32_0,
        end_frame: 1699 as ::core::ffi::c_int as f32_0,
        loop_frame: 1670 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1700 as ::core::ffi::c_int as f32_0,
        end_frame: 1749 as ::core::ffi::c_int as f32_0,
        loop_frame: 1720 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1750 as ::core::ffi::c_int as f32_0,
        end_frame: 1799 as ::core::ffi::c_int as f32_0,
        loop_frame: 1770 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 1800 as ::core::ffi::c_int as f32_0,
        end_frame: 1849 as ::core::ffi::c_int as f32_0,
        loop_frame: 1820 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB57C: [AnimLoopSettings; 3] = [
    AnimLoopSettings {
        start_frame: 2000 as ::core::ffi::c_int as f32_0,
        end_frame: 2049 as ::core::ffi::c_int as f32_0,
        loop_frame: 2020 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2050 as ::core::ffi::c_int as f32_0,
        end_frame: 2099 as ::core::ffi::c_int as f32_0,
        loop_frame: 2070 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2100 as ::core::ffi::c_int as f32_0,
        end_frame: 2149 as ::core::ffi::c_int as f32_0,
        loop_frame: 2120 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB5A0: [AnimLoopSettings; 3] = [
    AnimLoopSettings {
        start_frame: 3150 as ::core::ffi::c_int as f32_0,
        end_frame: 3199 as ::core::ffi::c_int as f32_0,
        loop_frame: 3170 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 3200 as ::core::ffi::c_int as f32_0,
        end_frame: 3249 as ::core::ffi::c_int as f32_0,
        loop_frame: 3220 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 3250 as ::core::ffi::c_int as f32_0,
        end_frame: 3299 as ::core::ffi::c_int as f32_0,
        loop_frame: 3270 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB5C4: [AnimLoopSettings; 3] = [
    AnimLoopSettings {
        start_frame: 2250 as ::core::ffi::c_int as f32_0,
        end_frame: 2299 as ::core::ffi::c_int as f32_0,
        loop_frame: 2270 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2300 as ::core::ffi::c_int as f32_0,
        end_frame: 2349 as ::core::ffi::c_int as f32_0,
        loop_frame: 2320 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2350 as ::core::ffi::c_int as f32_0,
        end_frame: 2399 as ::core::ffi::c_int as f32_0,
        loop_frame: 2370 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB5E8: [AnimLoopSettings; 10] = [
    AnimLoopSettings {
        start_frame: 2550 as ::core::ffi::c_int as f32_0,
        end_frame: 2599 as ::core::ffi::c_int as f32_0,
        loop_frame: 2570 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2600 as ::core::ffi::c_int as f32_0,
        end_frame: 2649 as ::core::ffi::c_int as f32_0,
        loop_frame: 2620 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2650 as ::core::ffi::c_int as f32_0,
        end_frame: 2699 as ::core::ffi::c_int as f32_0,
        loop_frame: 2670 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2700 as ::core::ffi::c_int as f32_0,
        end_frame: 2749 as ::core::ffi::c_int as f32_0,
        loop_frame: 2720 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2750 as ::core::ffi::c_int as f32_0,
        end_frame: 2799 as ::core::ffi::c_int as f32_0,
        loop_frame: 2770 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2800 as ::core::ffi::c_int as f32_0,
        end_frame: 2849 as ::core::ffi::c_int as f32_0,
        loop_frame: 2820 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2850 as ::core::ffi::c_int as f32_0,
        end_frame: 2899 as ::core::ffi::c_int as f32_0,
        loop_frame: 2870 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2900 as ::core::ffi::c_int as f32_0,
        end_frame: 2949 as ::core::ffi::c_int as f32_0,
        loop_frame: 2920 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2900 as ::core::ffi::c_int as f32_0,
        end_frame: 2949 as ::core::ffi::c_int as f32_0,
        loop_frame: 2920 as ::core::ffi::c_int as f32_0,
    },
    AnimLoopSettings {
        start_frame: 2900 as ::core::ffi::c_int as f32_0,
        end_frame: 2949 as ::core::ffi::c_int as f32_0,
        loop_frame: 2920 as ::core::ffi::c_int as f32_0,
    },
];
pub static mut mn_803EB660: [u16_0; 6] = [
    0x81 as ::core::ffi::c_int as u16_0,
    0x82 as ::core::ffi::c_int as u16_0,
    0x83 as ::core::ffi::c_int as u16_0,
    0x84 as ::core::ffi::c_int as u16_0,
    0x85 as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
];
pub static mut mn_803EB66C: [u16_0; 6] = [
    0x86 as ::core::ffi::c_int as u16_0,
    0x87 as ::core::ffi::c_int as u16_0,
    0x88 as ::core::ffi::c_int as u16_0,
    0x89 as ::core::ffi::c_int as u16_0,
    0x8a as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
];
pub static mut mn_803EB678: [u16_0; 6] = [
    0x8e as ::core::ffi::c_int as u16_0,
    0x8f as ::core::ffi::c_int as u16_0,
    0x90 as ::core::ffi::c_int as u16_0,
    0x91 as ::core::ffi::c_int as u16_0,
    0x92 as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
];
pub static mut mn_803EB684: [u16_0; 6] = [
    0x9a as ::core::ffi::c_int as u16_0,
    0x9b as ::core::ffi::c_int as u16_0,
    0x9c as ::core::ffi::c_int as u16_0,
    0x9d as ::core::ffi::c_int as u16_0,
    0x9d as ::core::ffi::c_int as u16_0,
    0x9e as ::core::ffi::c_int as u16_0,
];
pub static mut mn_803EB690: [u16_0; 6] = [
    0x9f as ::core::ffi::c_int as u16_0,
    0xa0 as ::core::ffi::c_int as u16_0,
    0xa1 as ::core::ffi::c_int as u16_0,
    0xa2 as ::core::ffi::c_int as u16_0,
    0xa3 as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
];
pub static mut mn_803EB69C: [u16_0; 10] = [
    0xb1 as ::core::ffi::c_int as u16_0,
    0xb2 as ::core::ffi::c_int as u16_0,
    0xb3 as ::core::ffi::c_int as u16_0,
    0xb4 as ::core::ffi::c_int as u16_0,
    0xb5 as ::core::ffi::c_int as u16_0,
    0xb6 as ::core::ffi::c_int as u16_0,
    0xb7 as ::core::ffi::c_int as u16_0,
    0xb8 as ::core::ffi::c_int as u16_0,
    0xb9 as ::core::ffi::c_int as u16_0,
    0xba as ::core::ffi::c_int as u16_0,
];
pub static mut mn_804D4B30: [u16_0; 4] = [
    0x8b as ::core::ffi::c_int as u16_0,
    0x8c as ::core::ffi::c_int as u16_0,
    0x8d as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
];
pub static mut mn_804D4B38: [u16_0; 4] = [
    0x96 as ::core::ffi::c_int as u16_0,
    0x97 as ::core::ffi::c_int as u16_0,
    0x98 as ::core::ffi::c_int as u16_0,
    0x99 as ::core::ffi::c_int as u16_0,
];
pub static mut mn_804D4B40: [u16_0; 4] = [
    0xa4 as ::core::ffi::c_int as u16_0,
    0xa5 as ::core::ffi::c_int as u16_0,
    0xa6 as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
];
pub static mut mn_804D4B48: [u16_0; 4] = [
    0xa8 as ::core::ffi::c_int as u16_0,
    0xa9 as ::core::ffi::c_int as u16_0,
    0xaa as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
];
pub static mut mn_804D4B50: GXColor = _GXColor {
    r: 0x5a as u8_0,
    g: 0x73 as u8_0,
    b: 0xff as u8_0,
    a: 0xff as u8_0,
};
pub static mut mn_804D4B54: GXColor = _GXColor {
    r: 0xff as u8_0,
    g: 0x5a as u8_0,
    b: 0x41 as u8_0,
    a: 0xff as u8_0,
};
pub static mut mn_804D4B58: GXColor = _GXColor {
    r: 0xe as u8_0,
    g: 0xd2 as u8_0,
    b: 0x41 as u8_0,
    a: 0xff as u8_0,
};
pub static mut mn_804D4B5C: GXColor = _GXColor {
    r: 0xf0 as u8_0,
    g: 0xc8 as u8_0,
    b: 0x5a as u8_0,
    a: 0xff as u8_0,
};
pub static mut mn_804D4B60: GXColor = _GXColor {
    r: 0x9b as u8_0,
    g: 0x41 as u8_0,
    b: 0xff as u8_0,
    a: 0xff as u8_0,
};
pub static mut mn_804D4B64: GXColor = _GXColor {
    r: 0xff as u8_0,
    g: 0xc8 as u8_0,
    b: 0 as u8_0,
    a: 0xff as u8_0,
};
pub static mut mn_803EB6B0: [MenuKindData; 34] = unsafe {
    [
        MenuKindData {
            anim_loop: &raw const mn_803EB3FC as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: &raw const mn_803EB660 as *mut u16_0,
            selection_count: 0x5 as u8_0,
            think: Some(mn_8022DB10 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        },
        MenuKindData {
            anim_loop: &raw const mn_803EB444 as *mut AnimLoopSettings,
            start_frame: 20 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: &raw const mn_803EB66C as *mut u16_0,
            selection_count: 0x5 as u8_0,
            think: Some(mn_8022D7F4 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        },
        MenuKindData {
            anim_loop: &raw const mn_803EB48C as *mut AnimLoopSettings,
            start_frame: 40 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: &raw const mn_803EB678 as *mut u16_0,
            selection_count: 0x5 as u8_0,
            think: Some(mn_8022D594 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        },
        MenuKindData {
            anim_loop: &raw const mn_803EB4C8 as *mut AnimLoopSettings,
            start_frame: 60 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: &raw const mn_804D4B38 as *mut u16_0,
            selection_count: 0x4 as u8_0,
            think: Some(mn_8022D34C as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        },
        MenuKindData {
            anim_loop: &raw const mn_803EB4F8 as *mut AnimLoopSettings,
            start_frame: 80 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: &raw const mn_803EB684 as *mut u16_0,
            selection_count: 0x6 as u8_0,
            think: Some(mn_8022D104 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        },
        MenuKindData {
            anim_loop: &raw const mn_803EB540 as *mut AnimLoopSettings,
            start_frame: 100 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: &raw const mn_803EB690 as *mut u16_0,
            selection_count: 0x5 as u8_0,
            think: Some(mn_8022CE6C as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        },
        MenuKindData {
            anim_loop: &raw const mn_803EB57C as *mut AnimLoopSettings,
            start_frame: 120 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: &raw const mn_804D4B30 as *mut u16_0,
            selection_count: 0x3 as u8_0,
            think: Some(mn_8022CC28 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: &raw const mn_803EB5C4 as *mut AnimLoopSettings,
            start_frame: 160 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: &raw const mn_804D4B48 as *mut u16_0,
            selection_count: 0x3 as u8_0,
            think: Some(mn_8022C7CC as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: &raw const mn_803EB5E8 as *mut AnimLoopSettings,
            start_frame: 200 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: &raw const mn_803EB69C as *mut u16_0,
            selection_count: 0xa as u8_0,
            think: Some(mn_8022C4F4 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x7 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x6 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: &raw const mn_803EB5A0 as *mut AnimLoopSettings,
            start_frame: 240 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: &raw const mn_804D4B40 as *mut u16_0,
            selection_count: 0x3 as u8_0,
            think: Some(mn_8022CA54 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
        MenuKindData {
            anim_loop: ::core::ptr::null::<AnimLoopSettings>() as *mut AnimLoopSettings,
            start_frame: 0 as ::core::ffi::c_int as ::core::ffi::c_float,
            description_indices: ::core::ptr::null::<u16_0>() as *mut u16_0,
            selection_count: 0x1 as u8_0,
            think: None,
        },
    ]
};
pub unsafe extern "C" fn mn_802295AC() -> u8_0 {
    melee_core::ct!("mn_802295AC");
    let mut port_idx: s32 = 0 as s32;
    while port_idx < 4 as ::core::ffi::c_int {
        let mut res: u64_0 = gm_GetButtonsTriggered(port_idx as u8_0);
        if res as ::core::ffi::c_ulonglong & PAD_CONFIRM != 0 {
            return port_idx as u8_0;
        }
        port_idx += 1 as ::core::ffi::c_int;
    }
    return 0 as u8_0;
}
pub unsafe extern "C" fn mn_80229624(mut slot: u32_0) -> u32_0 {
    melee_core::ct!("mn_80229624");
    let mut ret: u32_0 = 0 as u32_0;
    let mut inputs_repeated: u64_0 = 0;
    let mut inputs_trigger: u64_0 = 0;
    inputs_repeated = gm_801A36C0(slot as u8_0);
    inputs_trigger = gm_GetButtonsTriggered(slot as u8_0);
    if mn_804D6BC8.cooldown as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        Menu_DecrementAnimTimer();
        return 0 as u32_0;
    }
    if inputs_trigger & PAD_BUTTON_A as u64_0 != 0 {
        ret |= MenuInput_AButton as ::core::ffi::c_int as u32_0;
    }
    if inputs_trigger & PAD_BUTTON_START as u64_0 != 0 {
        ret |= MenuInput_StartButton as ::core::ffi::c_int as u32_0;
    }
    if inputs_trigger as ::core::ffi::c_ulonglong & PAD_CONFIRM != 0 {
        ret |= MenuInput_Confirm as ::core::ffi::c_int as u32_0;
    }
    if inputs_trigger as ::core::ffi::c_ulonglong & PAD_CANCEL != 0 {
        ret |= MenuInput_Back as ::core::ffi::c_int as u32_0;
    }
    if inputs_trigger & PAD_TRIGGER_L as u64_0 != 0 {
        ret |= MenuInput_LTrigger as ::core::ffi::c_int as u32_0;
    }
    if inputs_trigger & PAD_TRIGGER_R as u64_0 != 0 {
        ret |= MenuInput_RTrigger as ::core::ffi::c_int as u32_0;
    }
    if inputs_trigger & PAD_BUTTON_X as u64_0 != 0 {
        ret |= MenuInput_XButton as ::core::ffi::c_int as u32_0;
    }
    if inputs_trigger & PAD_BUTTON_Y as u64_0 != 0 {
        ret |= MenuInput_YButton as ::core::ffi::c_int as u32_0;
    }
    if inputs_repeated as ::core::ffi::c_ulonglong & PAD_ANY_UP != 0 {
        ret |= MenuInput_Up as ::core::ffi::c_int as u32_0;
    }
    if inputs_repeated as ::core::ffi::c_ulonglong & PAD_ANY_DOWN != 0 {
        ret |= MenuInput_Down as ::core::ffi::c_int as u32_0;
    }
    if inputs_repeated as ::core::ffi::c_ulonglong & PAD_ANY_LEFT != 0 {
        ret |= MenuInput_Left as ::core::ffi::c_int as u32_0;
    }
    if inputs_repeated as ::core::ffi::c_ulonglong & PAD_ANY_RIGHT != 0 {
        ret |= MenuInput_Right as ::core::ffi::c_int as u32_0;
    }
    return ret;
}
pub unsafe extern "C" fn mn_80229860(mut pending_mode: s8) {
    melee_core::ct!("mn_80229860");
    let mut data: *mut MenuExitData = gm_GetCurrentSceneExitData() as *mut MenuExitData;
    (*data).pending_mode = pending_mode;
    gm_801A4B60();
}
pub unsafe extern "C" fn mn_80229894(mut arg0: s32, mut arg1: u16_0, mut arg2: s32) {
    melee_core::ct!("mn_80229894");
    let mut temp_r3: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut temp_r0: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    if gmFrontend_NativeReturn(arg0 as ::core::ffi::c_int, arg1 as ::core::ffi::c_int)
        != 0
    {
        let mut exit: *mut MenuExitData = gm_GetCurrentSceneExitData()
            as *mut MenuExitData;
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.cur_menu = arg0 as MenuKind8;
        mn_804A04F0.hovered_selection = arg1;
        (*exit).pending_mode = (GM_COUNT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            as s8;
        gm_801A4B60();
        HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
        return;
    }
    mn_804D6BC8.cooldown = 5 as u16_0;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = arg0 as MenuKind8;
    mn_804A04F0.hovered_selection = arg1;
    HSD_GObj_80390CD4(mn_8022B3A0(arg2 as u8_0) as *mut HSD_GObj);
    HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
    temp_r0 = mn_803EB6B0[arg0 as usize].think
        as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
    if temp_r0.is_some() {
        temp_r3 = HSD_GObj_SetupProc(
            GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
            temp_r0 as HSD_GObjEvent,
            0 as u8_0,
        );
        (*temp_r3).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
    }
}
pub unsafe extern "C" fn mn_80229938(
    mut menu_kind: MenuKind,
    mut selection: s32,
) -> bool_0 {
    melee_core::ct!("mn_80229938");
    if menu_kind as ::core::ffi::c_int == MENU_KIND_REG as ::core::ffi::c_int
        && selection == SEL_REG_ALLSTAR as ::core::ffi::c_int
    {
        if gmMainLib_8015EDD4() != 0 {
            return true_0;
        }
        return false_0;
    }
    if menu_kind as ::core::ffi::c_int == MENU_KIND_DATA as ::core::ffi::c_int
        && selection == SEL_DATA_SOUND as ::core::ffi::c_int
    {
        if gmMainLib_8015EE90() != 0 {
            return true_0;
        }
        return false_0;
    }
    if menu_kind as ::core::ffi::c_int == MENU_KIND_SETTINGS as ::core::ffi::c_int
        && selection == SEL_SETTINGS_3 as ::core::ffi::c_int
    {
        return false_0;
    }
    if menu_kind as ::core::ffi::c_int == MENU_KIND_1P as ::core::ffi::c_int
        && selection == SEL_1P_2 as ::core::ffi::c_int
    {
        return false_0;
    }
    if menu_kind as ::core::ffi::c_int == MENU_KIND_TOY as ::core::ffi::c_int {
        match selection {
            2 => return false_0,
            _ => return true_0,
        }
    }
    return true_0;
}
pub unsafe extern "C" fn mn_80229A04(
    mut kind: MenuKind,
    mut selection: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    melee_core::ct!("mn_80229A04");
    if envoy_main6(kind as ::core::ffi::c_int) && (0..6).contains(&selection) {
        return ENVOY_SLOT[selection as usize];
    }
    let mut i: ::core::ffi::c_int = 0;
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    i = 0 as ::core::ffi::c_int;
    while i < selection {
        if mn_80229938(kind, i as s32) != false_0 {
            count += 1;
        }
        i += 1;
    }
    return count;
}
#[inline]
pub unsafe extern "C" fn mn_80229A04_dontinline(
    mut kind: MenuKind,
    mut selection: ::core::ffi::c_int,
) -> u8_0 {
    melee_core::ct!("mn_80229A04_dontinline");
    return mn_80229A04(kind, selection) as u8_0;
}
pub unsafe extern "C" fn mn_80229A7C(
    mut data: *mut MainMenuData,
    mut menu_kind: MenuKind,
    mut selection: ::core::ffi::c_int,
) {
    melee_core::ct!("mn_80229A7C");
    let mut temp_r3: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut sis_idx: *mut u16_0 = ::core::ptr::null_mut::<u16_0>();
    if !(*data).description.is_null() {
        HSD_SisLib_803A5CC4((*data).description);
        (*data).description = ::core::ptr::null_mut::<HSD_Text>();
    }
    sis_idx = mn_803EB6B0[menu_kind as usize].description_indices;
    if !sis_idx.is_null() {
        text = HSD_SisLib_803A5ACC(
            0 as ::core::ffi::c_int,
            mn_804D6BB4 as s32,
            -9.5f32,
            9.1f32,
            17.0f32,
            364.68332f32,
            38.38772f32,
        );
        (*data).description = text;
        (*text).font_size.x = 0.0521f32 as f32_0;
        (*text).font_size.y = 0.0521f32 as f32_0;
        HSD_SisLib_803A6368(text, *sis_idx.offset(envoy_anim_sel(menu_kind as ::core::ffi::c_int, selection) as isize) as s32);
    }
}
#[inline]
pub unsafe extern "C" fn mn_80229A7C_dontinline(
    mut arg0: *mut ::core::ffi::c_void,
    mut arg1: ::core::ffi::c_int,
    mut arg2: ::core::ffi::c_int,
) {
    melee_core::ct!("mn_80229A7C_dontinline");
    mn_80229A7C(arg0 as *mut MainMenuData, arg1 as MenuKind, arg2);
}
pub static mut MenMainBack_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub unsafe extern "C" fn mn_80229B2C() -> *mut HSD_GObj {
    melee_core::ct!("mn_80229B2C");
    let mut temp_r30: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut temp_r3: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    temp_r30 = GObj_Create(4 as u16_0, 5 as u8_0, 0x80 as u8_0);
    temp_r3 = HSD_JObjLoadJoint(MenMainBack_Top.joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(
        temp_r30,
        HSD_GObj_JObjKind,
        temp_r3 as *mut ::core::ffi::c_void,
    );
    GObj_SetupGXLink(
        temp_r30,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, isize) -> (),
        ),
        2 as u8_0,
        0x80 as u32_0,
    );
    HSD_GObj_SetupProc(
        temp_r30,
        Some(mn_8022EAE0 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    HSD_JObjAddAnimAll(
        temp_r3,
        MenMainBack_Top.animjoint,
        MenMainBack_Top.matanim_joint,
        MenMainBack_Top.shapeanim_joint,
    );
    HSD_JObjReqAnimAll(temp_r3, 0.0f32);
    HSD_JObjAnimAll(temp_r3);
    return temp_r30 as *mut HSD_GObj;
}
pub static mut MenMainPanel_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainConTop_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursor_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainConRl_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorRl_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainNmRl_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorTr01_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorTr02_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorTr03_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorTr04_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorRl01_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorRl02_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorRl03_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorRl04_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorRl05_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainConIs_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorIs_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainConSs_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut MenMainCursorSs_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub unsafe extern "C" fn fn_80229BF4(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80229BF4");
    let mut temp_f1: f32_0 = 0.;
    let mut temp_r28: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp20: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp1C: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp10: AnimLoopSettings = AnimLoopSettings {
        start_frame: 0.,
        end_frame: 0.,
        loop_frame: 0.,
    };
    let mut data: *mut MainMenuPanelData = ::core::ptr::null_mut::<MainMenuPanelData>();
    temp_r28 = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    data = (*gobj).user_data as *mut MainMenuPanelData;
    lb_80011E24(temp_r28 as *mut HSD_JObj, &raw mut sp20, &[(4 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    lb_80011E24(temp_r28 as *mut HSD_JObj, &raw mut sp1C, &[(0x29 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    if (*data).cur_menu as ::core::ffi::c_int
        != mn_804A04F0.cur_menu as ::core::ffi::c_int
    {
        (*data).prev_menu = (*data).cur_menu;
        (*data).cur_menu = mn_804A04F0.cur_menu;
        if mn_804A04F0.entering_menu as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            HSD_JObjReqAnimAll(
                sp1C,
                mn_803EAE8C[(*data).cur_menu as usize][0 as ::core::ffi::c_int as usize]
                    .start_frame,
            );
            (*data).state = 0 as MenuState8;
        } else {
            HSD_JObjReqAnimAll(
                sp1C,
                mn_803EAE8C[(*data).prev_menu as usize][1 as ::core::ffi::c_int as usize]
                    .start_frame,
            );
            (*data).state = 1 as MenuState8;
        }
    }
    match (*data).state as ::core::ffi::c_int {
        2 => {
            mn_8022ED6C(
                sp1C as *mut HSD_JObj,
                (&raw mut *(&raw mut mn_803EAE8C as *mut [AnimLoopSettings; 3])
                    .offset((*data).cur_menu as isize) as *mut AnimLoopSettings)
                    .offset(2 as ::core::ffi::c_int as isize) as *mut AnimLoopSettings,
            );
        }
        0 | 1 => {
            temp_f1 = mn_8022F298(sp1C as *mut HSD_JObj);
            if (*data).state as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                sp10 = mn_803EAE8C[(*data).cur_menu
                    as usize][0 as ::core::ffi::c_int as usize];
            } else {
                sp10 = mn_803EAE8C[(*data).prev_menu
                    as usize][1 as ::core::ffi::c_int as usize];
            }
            if temp_f1 >= sp10.end_frame {
                (*data).state = 2 as MenuState8;
                HSD_JObjReqAnimAll(
                    sp1C,
                    mn_803EAE8C[(*data).cur_menu
                            as usize][2 as ::core::ffi::c_int as usize]
                        .start_frame,
                );
            }
            HSD_JObjAnimAll(sp1C);
        }
        _ => {}
    }
    mn_8022ED6C(sp20 as *mut HSD_JObj, &raw mut mn_803EB354);
}
pub unsafe extern "C" fn mn_80229DC0() -> *mut HSD_GObj {
    melee_core::ct!("mn_80229DC0");
    let mut spC: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp8: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut temp_r31: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut temp_r3: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut user_data: *mut MainMenuPanelData = ::core::ptr::null_mut::<
        MainMenuPanelData,
    >();
    let mut tmp: u8_0 = 0;
    temp_r31 = GObj_Create(5 as u16_0, 6 as u8_0, 0x80 as u8_0);
    temp_r3 = HSD_JObjLoadJoint(MenMainPanel_Top.joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(
        temp_r31,
        HSD_GObj_JObjKind,
        temp_r3 as *mut ::core::ffi::c_void,
    );
    GObj_SetupGXLink(
        temp_r31,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, isize) -> (),
        ),
        3 as u8_0,
        0x80 as u32_0,
    );
    HSD_GObj_SetupProc(
        temp_r31,
        Some(fn_80229BF4 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    HSD_JObjAddAnimAll(
        temp_r3,
        MenMainPanel_Top.animjoint,
        MenMainPanel_Top.matanim_joint,
        MenMainPanel_Top.shapeanim_joint,
    );
    lb_80011E24(temp_r3 as *mut HSD_JObj, &raw mut spC, &[(4 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    lb_80011E24(temp_r3 as *mut HSD_JObj, &raw mut sp8, &[(0x29 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    HSD_JObjReqAnimAll(spC, mn_803EB354.start_frame);
    HSD_JObjAnimAll(spC);
    if mn_804A04F0.cur_menu as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        HSD_JObjReqAnimAll(
            sp8,
            mn_803EAE8C[mn_804A04F0.cur_menu as usize][0 as ::core::ffi::c_int as usize]
                .start_frame,
        );
    } else {
        HSD_JObjReqAnimAll(
            sp8,
            mn_803EAE8C[mn_804A04F0.cur_menu as usize][2 as ::core::ffi::c_int as usize]
                .start_frame,
        );
    }
    HSD_JObjAnimAll(sp8);
    user_data = HSD_MemAlloc(::core::mem::size_of::<MainMenuPanelData>() as ssize_t)
        as *mut MainMenuPanelData;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char, &[]);
        __assert(
            b"src/melee/mn/mnmain.c\0" as *const u8 as *const ::core::ffi::c_char,
            923 as u32_0,
            b"user_data\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    GObj_InitUserData(
        temp_r31,
        0 as u8_0,
        Some(mn_8022EB04 as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        user_data as *mut ::core::ffi::c_void,
    );
    tmp = mn_804A04F0.cur_menu as u8_0;
    (*user_data).cur_menu = tmp as MenuKind8;
    (*user_data).prev_menu = tmp as MenuKind8;
    (*user_data).state = 0 as MenuState8;
    return temp_r31 as *mut HSD_GObj;
}
pub static mut mn_803B84E8: Vec3 = Vec3 {
    x: 0 as ::core::ffi::c_int as f32_0,
    y: 0.,
    z: 0.,
};
pub static mut mn_803B84F4: Vec3 = Vec3 {
    x: 0 as ::core::ffi::c_int as f32_0,
    y: 0.,
    z: 0.,
};
pub unsafe extern "C" fn mn_80229F60(
    mut gp: *mut HSD_GObj,
    mut root: *mut HSD_JObj,
    mut selection: MainMenuSelection,
) {
    melee_core::ct!("mn_80229F60");
    let mut temp_f31: ::core::ffi::c_float = 0.;
    let mut data: *mut MainMenuData = ::core::ptr::null_mut::<MainMenuData>();
    let mut flow: *mut MenuFlow = ::core::ptr::null_mut::<MenuFlow>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp54: [*mut HSD_JObj; 7] = [::core::ptr::null_mut::<HSD_JObj>(); 7];
    let mut new_var: ::core::ffi::c_float = 0.;
    let mut pad_50: [u8_0; 4] = [0; 4];
    let mut sp44: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut sp38: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut sp2C: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut sp20: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut pad2: [u8_0; 12] = [0; 12];
    sp44 = mn_803B84E8;
    data = (*gp).user_data as *mut MainMenuData;
    lb_8001204C(
        root as *mut HSD_JObj,
        &raw mut sp54 as *mut *mut HSD_JObj,
        &raw mut mn_803EAE7C as *mut u16_0,
        7 as ::core::ffi::c_int,
    );
    mn_803EB360[0 as ::core::ffi::c_int as usize].start_frame = mn_803EB360[0
            as ::core::ffi::c_int as usize]
        .start_frame;
    jobj = sp54[1 as ::core::ffi::c_int as usize];
    flow = &raw mut mn_804A04F0;
    HSD_JObjReqAnimAll(jobj, 1.0f32);
    HSD_JObjAnimAll(jobj);
    HSD_JObjReqAnim(
        jobj,
        mn_803EB6B0[(*flow).cur_menu as usize].start_frame as f32_0
            + (selection as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as f32_0,
    );
    mn_8022F3D8(jobj as *mut HSD_JObj, 0xc as u8_0, TOBJ_MASK);
    mn_8022F3D8(jobj as *mut HSD_JObj, 0xd as u8_0, TOBJ_MASK);
    mn_8022F3D8(jobj as *mut HSD_JObj, 0xe as u8_0, TOBJ_MASK);
    mn_8022F3D8(jobj as *mut HSD_JObj, 0xf as u8_0, TOBJ_MASK);
    mn_8022F3D8(jobj as *mut HSD_JObj, 0x10 as u8_0, TOBJ_MASK);
    mn_8022F3D8(jobj as *mut HSD_JObj, 0x11 as u8_0, TOBJ_MASK);
    mn_8022F3D8(jobj as *mut HSD_JObj, 0x12 as u8_0, TOBJ_MASK);
    mn_8022F3D8(jobj as *mut HSD_JObj, 0x13 as u8_0, TOBJ_MASK);
    HSD_JObjAnim(jobj);
    new_var = mn_8022F298(sp54[0 as ::core::ffi::c_int as usize])
        as ::core::ffi::c_float;
    HSD_JObjReqAnimAll(
        sp54[0 as ::core::ffi::c_int as usize],
        mn_803EB360[1 as ::core::ffi::c_int as usize].start_frame
            + (new_var as f32_0
                - mn_803EB360[0 as ::core::ffi::c_int as usize].start_frame),
    );
    HSD_JObjAnimAll(sp54[0 as ::core::ffi::c_int as usize]);
    new_var = mn_8022F298(sp54[2 as ::core::ffi::c_int as usize])
        as ::core::ffi::c_float;
    HSD_JObjReqAnimAll(
        sp54[2 as ::core::ffi::c_int as usize],
        mn_803EB378[1 as ::core::ffi::c_int as usize].start_frame
            + (new_var as f32_0
                - mn_803EB378[0 as ::core::ffi::c_int as usize].start_frame),
    );
    HSD_JObjAnimAll(sp54[2 as ::core::ffi::c_int as usize]);
    HSD_JObjReqAnimAll(sp54[3 as ::core::ffi::c_int as usize], mn_803EB390.start_frame);
    HSD_JObjAnimAll(sp54[3 as ::core::ffi::c_int as usize]);
    HSD_JObjClearFlagsAll(sp54[3 as ::core::ffi::c_int as usize], JOBJ_HIDDEN as u32_0);
    HSD_JObjClearFlagsAll(sp54[4 as ::core::ffi::c_int as usize], JOBJ_HIDDEN as u32_0);
    HSD_JObjSetScaleY(sp54[5 as ::core::ffi::c_int as usize], 2.0f32);
    mn_803EB6B0[(*flow).cur_menu as usize].start_frame
        += 0 as ::core::ffi::c_int as ::core::ffi::c_float;
    HSD_JObjSetScaleY(sp54[6 as ::core::ffi::c_int as usize], 2.0f32);
    HSD_JObjGetTranslation(sp54[5 as ::core::ffi::c_int as usize], &raw mut sp20);
    HSD_JObjSetTranslate(sp54[6 as ::core::ffi::c_int as usize], &raw mut sp20);
    lb_8000B1CC(sp54[5 as ::core::ffi::c_int as usize], &raw mut sp44, &raw mut sp2C);
    lb_8000B1CC(
        (*data).tree[41 as ::core::ffi::c_int as usize],
        &raw mut sp44,
        &raw mut sp38,
    );
    temp_f31 = atan2f(
        sp38.y as ::core::ffi::c_float - sp2C.y as ::core::ffi::c_float,
        sp38.x as ::core::ffi::c_float - sp2C.x as ::core::ffi::c_float,
    );
    HSD_JObjSetRotationZ(sp54[5 as ::core::ffi::c_int as usize], temp_f31 as f32_0);
    HSD_JObjSetRotationZ(sp54[6 as ::core::ffi::c_int as usize], temp_f31 as f32_0);
}
pub unsafe extern "C" fn mn_8022A440(
    mut gp: *mut HSD_GObj,
    mut root: *mut HSD_JObj,
    mut selection: MainMenuSelection,
) {
    melee_core::ct!("mn_8022A440");
    let mut sp24: [*mut HSD_JObj; 7] = [::core::ptr::null_mut::<HSD_JObj>(); 7];
    let mut r29: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut flow: *mut MenuFlow = ::core::ptr::null_mut::<MenuFlow>();
    let mut f: ::core::ffi::c_float = 0.;
    let mut _pad: [::core::ffi::c_uchar; 16] = [0; 16];
    lb_8001204C(
        root as *mut HSD_JObj,
        &raw mut sp24 as *mut *mut HSD_JObj,
        &raw mut mn_803EAE7C as *mut u16_0,
        7 as ::core::ffi::c_int,
    );
    r29 = sp24[1 as ::core::ffi::c_int as usize];
    flow = &raw mut mn_804A04F0;
    flow.is_null() as ::core::ffi::c_int;
    HSD_JObjReqAnimAll(r29, 0.0f32);
    HSD_JObjAnimAll(r29);
    HSD_JObjReqAnim(
        r29,
        mn_803EB6B0[(*flow).cur_menu as usize].start_frame as f32_0
            + (selection as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as f32_0,
    );
    mn_8022F3D8(r29 as *mut HSD_JObj, 0xc as u8_0, TOBJ_MASK);
    mn_8022F3D8(r29 as *mut HSD_JObj, 0xd as u8_0, TOBJ_MASK);
    mn_8022F3D8(r29 as *mut HSD_JObj, 0xe as u8_0, TOBJ_MASK);
    mn_8022F3D8(r29 as *mut HSD_JObj, 0xf as u8_0, TOBJ_MASK);
    mn_8022F3D8(r29 as *mut HSD_JObj, 0x10 as u8_0, TOBJ_MASK);
    mn_8022F3D8(r29 as *mut HSD_JObj, 0x11 as u8_0, TOBJ_MASK);
    mn_8022F3D8(r29 as *mut HSD_JObj, 0x12 as u8_0, TOBJ_MASK);
    mn_8022F3D8(r29 as *mut HSD_JObj, 0x13 as u8_0, TOBJ_MASK);
    HSD_JObjAnim(r29);
    f = mn_8022F298(sp24[0 as ::core::ffi::c_int as usize]) as ::core::ffi::c_float;
    HSD_JObjReqAnimAll(
        sp24[0 as ::core::ffi::c_int as usize],
        f as f32_0 - mn_803EB360[1 as ::core::ffi::c_int as usize].start_frame
            + mn_803EB360[0 as ::core::ffi::c_int as usize].start_frame,
    );
    HSD_JObjAnimAll(sp24[0 as ::core::ffi::c_int as usize]);
    mn_8022F298(sp24[2 as ::core::ffi::c_int as usize]);
    HSD_JObjReqAnimAll(
        sp24[2 as ::core::ffi::c_int as usize],
        mn_803EB378[0 as ::core::ffi::c_int as usize].start_frame,
    );
    HSD_JObjAnimAll(sp24[2 as ::core::ffi::c_int as usize]);
    HSD_JObjSetFlagsAll(sp24[4 as ::core::ffi::c_int as usize], JOBJ_HIDDEN as u32_0);
}
pub unsafe extern "C" fn mn_8022A5D0(
    mut gp: *mut HSD_GObj,
    mut selection: MainMenuSelection,
) {
    melee_core::ct!("mn_8022A5D0");
    let mut _pad: [u8_0; 8] = [0; 8];
    // C: HSD_JObj* spA0[7], indexed up to selection_count (10 in Special Melee): the C overruns its frame; host: room
    let mut spA0: [*mut HSD_JObj; 32] = [::core::ptr::null_mut::<HSD_JObj>(); 32];
    let mut sp84: [*mut HSD_JObj; 7] = [::core::ptr::null_mut::<HSD_JObj>(); 7];
    let mut sp80: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp74: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut sp68: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut sp5C: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut sp50: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut var_r0: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut var_r5: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut var_r4: *mut u16_0 = ::core::ptr::null_mut::<u16_0>();
    let mut temp_r4: u32_0 = 0;
    let mut data: *mut MainMenuData = ::core::ptr::null_mut::<MainMenuData>();
    let mut option_count: ::core::ffi::c_int = 0;
    let mut var_r26: ::core::ffi::c_int = 0;
    let mut var_r24: ::core::ffi::c_int = 0;
    let mut var_r23: ::core::ffi::c_int = 0;
    let mut temp_r22_2: ::core::ffi::c_int = 0;
    let mut temp_r3: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut i: ::core::ffi::c_int = 0;
    var_r4 = &raw mut mn_803EAE68 as *mut u16_0;
    sp68 = mn_803B84F4;
    data = (*gp).user_data as *mut MainMenuData;
    option_count = sel_count((*data).menu_kind as ::core::ffi::c_int);
    i = 0 as ::core::ffi::c_int;
    while i < option_count {
        temp_r3 = (*data).tree[*var_r4.offset(i as isize) as usize] as *mut HSD_JObj;
        if temp_r3.is_null() {
            var_r0 = ::core::ptr::null_mut::<HSD_JObj>();
        } else {
            var_r0 = (*temp_r3).child as *mut HSD_JObj;
        }
        spA0[i as usize] = var_r0;
        i += 1;
    }
    var_r26 = 0 as ::core::ffi::c_int;
    while var_r26 < option_count {
        if mn_80229938(mn_804A04F0.cur_menu as MenuKind, var_r26 as s32)
            != 0 as ::core::ffi::c_int
        {
            var_r24 = mn_80229A04(mn_804A04F0.cur_menu as MenuKind, var_r26);
            lb_8001204C(
                spA0[var_r24 as usize],
                &raw mut sp84 as *mut *mut HSD_JObj,
                &raw mut mn_803EAE7C as *mut u16_0,
                7 as ::core::ffi::c_int,
            );
            temp_r22_2 = (selection as ::core::ffi::c_int == var_r26)
                as ::core::ffi::c_int * 0xc as ::core::ffi::c_int;
            mn_8022ED6C(
                sp84[0 as ::core::ffi::c_int as usize],
                (&raw mut mn_803EB360 as *mut AnimLoopSettings)
                    .offset(
                        (var_r26 == selection as ::core::ffi::c_int)
                            as ::core::ffi::c_int as isize,
                    ) as *mut AnimLoopSettings,
            );
            mn_8022ED6C(
                sp84[2 as ::core::ffi::c_int as usize],
                (&raw mut mn_803EB378 as *mut AnimLoopSettings)
                    .offset(
                        (var_r26 == selection as ::core::ffi::c_int)
                            as ::core::ffi::c_int as isize,
                    ) as *mut AnimLoopSettings,
            );
            if mn_8022ED6C(sp84[3 as ::core::ffi::c_int as usize], &raw mut mn_803EB390)
                >= mn_803EB390.end_frame
            {
                HSD_JObjSetFlagsAll(
                    sp84[3 as ::core::ffi::c_int as usize],
                    JOBJ_HIDDEN as u32_0,
                );
            }
            if selection as ::core::ffi::c_int == var_r26 {
                if HSD_JObjGetScaleY(sp84[6 as ::core::ffi::c_int as usize]) == 2.0f32 {
                    lb_8000B1CC(
                        sp84[5 as ::core::ffi::c_int as usize],
                        &raw mut sp68,
                        &raw mut sp50,
                    );
                    lb_8000B1CC(
                        (*data).tree[41 as ::core::ffi::c_int as usize],
                        &raw mut sp68,
                        &raw mut sp5C,
                    );
                    sp74.x = sp5C.x - sp50.x;
                    sp74.y = sp5C.y - sp50.y;
                    sp74.z = sp5C.z - sp50.z;
                    HSD_JObjAddTranslationX(
                        sp84[6 as ::core::ffi::c_int as usize],
                        sp74.x as ::core::ffi::c_float / 10.0f32,
                    );
                    HSD_JObjAddTranslationY(
                        sp84[6 as ::core::ffi::c_int as usize],
                        sp74.y as ::core::ffi::c_float / 10.0f32,
                    );
                    HSD_JObjAddTranslationZ(
                        sp84[6 as ::core::ffi::c_int as usize],
                        sp74.z as ::core::ffi::c_float / 10.0f32,
                    );
                    if sp74.x >= 0.0f32
                        && HSD_JObjGetTranslationX(
                            sp84[6 as ::core::ffi::c_int as usize],
                        ) >= sp74.x
                        || sp74.x < 0.0f32
                            && HSD_JObjGetTranslationX(
                                sp84[6 as ::core::ffi::c_int as usize],
                            ) <= sp74.x
                    {
                        HSD_JObjSetTranslate(
                            sp84[6 as ::core::ffi::c_int as usize],
                            &raw mut sp74,
                        );
                        HSD_JObjAddScaleY(
                            sp84[6 as ::core::ffi::c_int as usize],
                            -0.2f32,
                        );
                        HSD_JObjAddScaleY(
                            sp84[5 as ::core::ffi::c_int as usize],
                            -0.2f32,
                        );
                    }
                } else if HSD_JObjGetScaleY(sp84[6 as ::core::ffi::c_int as usize])
                    > 0.0f32
                {
                    HSD_JObjAddScaleY(sp84[6 as ::core::ffi::c_int as usize], -0.2f32);
                    HSD_JObjAddScaleY(sp84[5 as ::core::ffi::c_int as usize], -0.2f32);
                    if HSD_JObjGetScaleY(sp84[6 as ::core::ffi::c_int as usize])
                        <= 0.0f32
                    {
                        HSD_JObjSetScaleY(
                            sp84[6 as ::core::ffi::c_int as usize],
                            0.0f32,
                        );
                        HSD_JObjSetScaleY(
                            sp84[5 as ::core::ffi::c_int as usize],
                            0.0f32,
                        );
                        HSD_JObjSetFlagsAll(
                            sp84[4 as ::core::ffi::c_int as usize],
                            JOBJ_HIDDEN as u32_0,
                        );
                    }
                }
            }
        }
        var_r26 += 1;
    }
}
#[inline]
pub unsafe extern "C" fn GetAnimStartFrame(mut anim_loop: *mut AnimLoopSettings) -> f32_0 {
    melee_core::ct!("GetAnimStartFrame");
    return (*anim_loop).start_frame;
}
#[inline]
pub unsafe extern "C" fn GetAnimEndFrame(mut anim_loop: *mut AnimLoopSettings) -> f32_0 {
    melee_core::ct!("GetAnimEndFrame");
    return (*anim_loop).end_frame;
}
pub unsafe extern "C" fn mn_8022ADD8(
    mut gp: *mut HSD_GObj,
    mut selection_changed: bool_0,
) {
    melee_core::ct!("mn_8022ADD8");
    let mut data: *mut MainMenuData = ::core::ptr::null_mut::<MainMenuData>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut cur_frame: f32_0 = 0.;
    let mut anim_loop2: *mut AnimLoopSettings = ::core::ptr::null_mut::<
        AnimLoopSettings,
    >();
    let mut anim_loop: *mut AnimLoopSettings = ::core::ptr::null_mut::<
        AnimLoopSettings,
    >();
    data = (*gp).user_data as *mut MainMenuData;
    jobj = (*data).tree[14 as ::core::ffi::c_int as usize] as *mut HSD_JObj;
    match (*data).state as ::core::ffi::c_int {
        2 | 4 => {
            HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32_0);
        }
        0 | 1 | 3 | 5 => {
            data = (*gp).user_data as *mut MainMenuData;
            anim_loop2 = mn_803EB6B0[(*data).menu_kind as usize].anim_loop;
            if !anim_loop2.is_null() {
                if (*data).menu_kind as ::core::ffi::c_int
                    == MENU_KIND_MAIN as ::core::ffi::c_int
                    && (*data).hovered_selection as ::core::ffi::c_int
                        == SEL_MAIN_DATA as ::core::ffi::c_int
                    && gmMainLib_8015EE90() == 0 as ::core::ffi::c_int
                {
                    anim_loop = &raw mut mn_803EB438;
                } else if (*data).menu_kind as ::core::ffi::c_int
                    == MENU_KIND_1P as ::core::ffi::c_int
                    && (*data).hovered_selection as ::core::ffi::c_int
                        == SEL_1P_REG as ::core::ffi::c_int
                    && gmMainLib_8015EDD4() == 0 as ::core::ffi::c_int
                {
                    anim_loop = &raw mut mn_803EB480;
                } else {
                    anim_loop = anim_loop2.offset(envoy_anim_sel((*data).menu_kind as ::core::ffi::c_int, (*data).hovered_selection as ::core::ffi::c_int) as isize)
                        as *mut AnimLoopSettings;
                }
                cur_frame = mn_8022F298(jobj as *mut HSD_JObj);
                if selection_changed as u8_0 as ::core::ffi::c_int != false_0
                    || !(GetAnimStartFrame(anim_loop) <= cur_frame)
                    || !(cur_frame <= GetAnimEndFrame(anim_loop))
                {
                    HSD_JObjReqAnimAll(jobj, GetAnimStartFrame(anim_loop));
                }
                mn_8022ED6C(jobj as *mut HSD_JObj, anim_loop);
                if envoy_main6((*data).menu_kind as ::core::ffi::c_int) {
                    // port: Envoy hovered: the retail right-hand panel is hidden (the mod draws its stickers there)
                    if (*data).hovered_selection as ::core::ffi::c_int == ENVOY_SEL {
                        HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32_0);
                    } else {
                        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32_0);
                    }
                }
            }
        }
        _ => {}
    };
}
pub unsafe extern "C" fn fn_8022AF10(mut gp: *mut HSD_GObj) {
    melee_core::ct!("fn_8022AF10");
    let mut data: *mut MainMenuData = ::core::ptr::null_mut::<MainMenuData>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut anim_loop: *mut AnimLoopSettings = ::core::ptr::null_mut::<
        AnimLoopSettings,
    >();
    data = (*gp).user_data as *mut MainMenuData;
    jobj = (*data).tree[3 as ::core::ffi::c_int as usize] as *mut HSD_JObj;
    match (*data).state as ::core::ffi::c_int {
        2 => {
            anim_loop = &raw mut mn_803EB3A8;
        }
        4 => {
            anim_loop = &raw mut mn_803EB3C0;
        }
        _ => {}
    }
    if mn_8022F298(jobj as *mut HSD_JObj) >= (*anim_loop).end_frame {
        match (*data).state as ::core::ffi::c_int {
            2 | 4 => {
                HSD_GObjFree(gp);
                return;
            }
            _ => {}
        }
    }
    HSD_JObjAnim(jobj);
}
pub unsafe extern "C" fn fn_8022AFEC(mut gp: *mut HSD_GObj) {
    melee_core::ct!("fn_8022AFEC");
    let mut think: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut data2: *mut MainMenuData = ::core::ptr::null_mut::<MainMenuData>();
    let mut temp_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut anim_loop: *mut AnimLoopSettings = ::core::ptr::null_mut::<
        AnimLoopSettings,
    >();
    let mut data: *mut MainMenuData = ::core::ptr::null_mut::<MainMenuData>();
    let mut var_r26: u8_0 = 0;
    let mut selection_changed: bool_0 = 0;
    let mut final_data: *mut MainMenuData = ::core::ptr::null_mut::<MainMenuData>();
    let mut hovered_selection: MainMenuSelection = SEL_MAIN_1P;
    let mut state: u8_0 = 0;
    let mut option_count: u8_0 = 0;
    let mut sp20: [*mut HSD_JObj; 12] = [::core::ptr::null_mut::<HSD_JObj>(); 12];
    let mut _pad: [::core::ffi::c_uchar; 18] = [0; 18];
    var_r26 = 0 as u8_0;
    selection_changed = false_0 as bool_0;
    let mut tmp_data: *mut MainMenuData = (*gp).user_data as *mut MainMenuData;
    data = tmp_data;
    state = (*data).state as u8_0;
    if (state as ::core::ffi::c_int == MENU_STATE_5 as ::core::ffi::c_int
        || state as ::core::ffi::c_int <= MENU_STATE_ENTER_TO as ::core::ffi::c_int
        || state as ::core::ffi::c_int == MENU_STATE_EXIT_TO as ::core::ffi::c_int)
        && (*data).menu_kind as ::core::ffi::c_int
            != mn_804A04F0.cur_menu as ::core::ffi::c_int
    {
        if mn_804A04F0.entering_menu as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            (*data).state = MENU_STATE_ENTER_FROM as ::core::ffi::c_int as MenuState8;
        } else {
            (*data).state = MENU_STATE_EXIT_FROM as ::core::ffi::c_int as MenuState8;
        }
        match (*data).state as ::core::ffi::c_int {
            1 => {
                anim_loop = &raw mut mn_803EB39C;
            }
            2 => {
                anim_loop = &raw mut mn_803EB3A8;
            }
            3 => {
                anim_loop = &raw mut mn_803EB3B4;
            }
            4 => {
                anim_loop = &raw mut mn_803EB3C0;
            }
            _ => {}
        }
        jobj = (*data).tree[3 as ::core::ffi::c_int as usize] as *mut HSD_JObj;
        HSD_JObjReqAnim(jobj, (*anim_loop).start_frame);
        HSD_JObjAnim(jobj);
        if (*data).state as ::core::ffi::c_int == MENU_STATE_5 as ::core::ffi::c_int
            || (*data).state as ::core::ffi::c_int
                <= MENU_STATE_ENTER_TO as ::core::ffi::c_int
            || (*data).state as ::core::ffi::c_int
                == MENU_STATE_EXIT_TO as ::core::ffi::c_int
        {
            var_r26 = 1 as u8_0;
            selection_changed = true_0 as bool_0;
        } else {
            HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
            think = HSD_GObj_SetupProc(
                gp,
                Some(fn_8022AF10 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
                0 as u8_0,
            );
            (*think).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        }
    }
    if (*data).state as ::core::ffi::c_int == MENU_STATE_ENTER_TO as ::core::ffi::c_int
        || (*data).state as ::core::ffi::c_int
            == MENU_STATE_EXIT_TO as ::core::ffi::c_int
    {
        jobj = (*data).tree[3 as ::core::ffi::c_int as usize] as *mut HSD_JObj;
        match (*data).state as ::core::ffi::c_int {
            1 => {
                anim_loop = &raw mut mn_803EB39C;
            }
            3 => {
                anim_loop = &raw mut mn_803EB3B4;
            }
            2 => {
                anim_loop = &raw mut mn_803EB3A8;
            }
            4 => {
                anim_loop = &raw mut mn_803EB3C0;
            }
            _ => {}
        }
        if mn_8022F298(jobj as *mut HSD_JObj) >= (*anim_loop).end_frame {
            match (*data).state as ::core::ffi::c_int {
                1 | 3 => {
                    (*data).state = MENU_STATE_IDLE as ::core::ffi::c_int as MenuState8;
                }
                2 | 4 => {
                    HSD_GObjFree(gp);
                    return;
                }
                _ => {}
            }
        }
        HSD_JObjAnim(jobj);
    }
    if (*data).state as ::core::ffi::c_int == MENU_STATE_IDLE as ::core::ffi::c_int
        || (*data).state as ::core::ffi::c_int
            == MENU_STATE_ENTER_TO as ::core::ffi::c_int
        || (*data).state as ::core::ffi::c_int
            == MENU_STATE_EXIT_TO as ::core::ffi::c_int
    {
        if (*data).hovered_selection as ::core::ffi::c_int
            != mn_804A04F0.hovered_selection as ::core::ffi::c_int
        {
            selection_changed = true_0 as bool_0;
        }
        data2 = HSD_GObjGetUserData(gp) as *mut MainMenuData;
        option_count = sel_count((*data2).menu_kind as ::core::ffi::c_int) as u8_0;
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < option_count as ::core::ffi::c_int {
            temp_jobj = (*data2).tree[mn_803EAE68[i as usize] as usize] as *mut HSD_JObj;
            if temp_jobj.is_null() {
                temp_jobj = ::core::ptr::null_mut::<HSD_JObj>();
            } else {
                temp_jobj = (*temp_jobj).child as *mut HSD_JObj;
            }
            sp20[i as usize] = temp_jobj;
            i += 1;
        }
        if selection_changed as u8_0 as ::core::ffi::c_int != false_0 {
            let mut cur_menu: MenuKind = mn_804A04F0.cur_menu as MenuKind;
            mn_80229F60(
                gp,
                sp20[mn_80229A04_dontinline(
                    cur_menu,
                    mn_804A04F0.hovered_selection as ::core::ffi::c_int,
                ) as usize],
                mn_804A04F0.hovered_selection as MainMenuSelection,
            );
            let mut cur_menu_0: MenuKind = mn_804A04F0.cur_menu as MenuKind;
            mn_8022A440(
                gp,
                sp20[mn_80229A04_dontinline(
                    cur_menu_0,
                    (*data2).hovered_selection as ::core::ffi::c_int,
                ) as usize],
                (*data2).hovered_selection as MainMenuSelection,
            );
        }
        if selection_changed as u8_0 as ::core::ffi::c_int != false_0 {
            hovered_selection = mn_804A04F0.hovered_selection as MainMenuSelection;
        } else {
            hovered_selection = (*data2).hovered_selection as MainMenuSelection;
        }
        mn_8022A5D0(gp, hovered_selection as u8_0 as MainMenuSelection);
    }
    mn_8022ADD8(gp, selection_changed);
    final_data = (*gp).user_data as *mut MainMenuData;
    state = selection_changed as u8_0;
    if state as s32 != false_0 {
        hovered_selection = mn_804A04F0.hovered_selection as MainMenuSelection;
    } else {
        hovered_selection = (*final_data).hovered_selection as MainMenuSelection;
    }
    match (*final_data).state as ::core::ffi::c_int {
        2 | 4 => {
            if !(*final_data).description.is_null() {
                final_data = (*gp).user_data as *mut MainMenuData;
                HSD_SisLib_803A5CC4((*final_data).description);
                (*final_data).description = ::core::ptr::null_mut::<HSD_Text>();
            }
        }
        0 => {
            if state as s32 != false_0 {
                mn_80229A7C_dontinline(
                    final_data as *mut ::core::ffi::c_void,
                    (*final_data).menu_kind as ::core::ffi::c_int,
                    hovered_selection as ::core::ffi::c_int,
                );
            }
            (*(*final_data).description).hidden = 0 as u8_0;
            // port: the Envoy row's description is the mod's own
            if envoy_main6((*final_data).menu_kind as ::core::ffi::c_int) && hovered_selection as ::core::ffi::c_int == ENVOY_SEL {
                (*(*final_data).description).hidden = 1 as u8_0;
            }
        }
        1 | 3 | 5 | _ => {}
    }
    if var_r26 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        (*data).menu_kind = mn_804A04F0.cur_menu;
    }
    if selection_changed as u8_0 as ::core::ffi::c_int != false_0 {
        (*data).hovered_selection = mn_804A04F0.hovered_selection as MenuKind8;
    }
}
/// port: the main menu's sixth row, Envoy (envoy-v3-ui.md). Selection 5 is virtual, shown in slot 2 (between Vs.
/// Mode and Trophies); the retail rows 2-4 move down one slot. Only while an Envoy mod registered its door
/// (`envoy::DOOR_ON`); otherwise every helper returns the retail value.
#[inline]
pub unsafe fn envoy_main6(kind: ::core::ffi::c_int) -> bool {
    crate::tu::envoy::DOOR_ON != 0 && kind == MENU_KIND_MAIN as ::core::ffi::c_int
}
pub const ENVOY_SEL: ::core::ffi::c_int = 5;
/// The Envoy row's plate keeps the disc's plate and loses its label (the mod draws ours on it): the cursor model's
/// label joints, hidden on that one instance. `MELEE_ENVOY_HIDEJ` (comma list of joint indices) overrides, for checks.
unsafe fn envoy_hide_label(cursor: *mut HSD_JObj) {
    let list = std::env::var("MELEE_ENVOY_HIDEJ").unwrap_or_else(|_| String::from(ENVOY_LABEL_JOINTS));
    for t in list.split(',') {
        let Ok(idx) = t.trim().parse::<usize>() else { continue };
        let mut j: *mut HSD_JObj = ::core::ptr::null_mut();
        lb_80011E24(cursor, &raw mut j, &[idx, (-(1 as ::core::ffi::c_int)) as usize]);
        if !j.is_null() {
            HSD_JObjSetFlagsAll(j, JOBJ_HIDDEN as u32_0);
        }
    }
}
const ENVOY_LABEL_JOINTS: &str = "";
const ENVOY_SLOT: [::core::ffi::c_int; 6] = [0, 1, 3, 4, 5, 2];
const ENVOY_ORDER: [::core::ffi::c_int; 6] = [0, 1, 5, 2, 3, 4];
#[inline]
unsafe fn sel_count(kind: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if envoy_main6(kind) { 6 } else { mn_803EB6B0[kind as usize].selection_count as ::core::ffi::c_int & 0xff }
}
/// The anim-loop / description index for a selection: Envoy borrows Vs. Mode's (its panel is hidden).
#[inline]
unsafe fn envoy_anim_sel(kind: ::core::ffi::c_int, sel: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if envoy_main6(kind) && sel == ENVOY_SEL { 1 } else { sel }
}
#[inline]
pub unsafe extern "C" fn GetSelectionFrameOffset(
    mut selection: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    melee_core::ct!("GetSelectionFrameOffset");
    return 2 as ::core::ffi::c_int * selection;
}
#[inline]
pub unsafe extern "C" fn CountUnlockedSelections(mut kind: MenuKind) -> u8_0 {
    melee_core::ct!("CountUnlockedSelections");
    if envoy_main6(kind as ::core::ffi::c_int) {
        return 6;
    }
    let mut i: ::core::ffi::c_int = 0;
    let mut count: u8_0 = 0 as u8_0;
    i = 0 as ::core::ffi::c_int;
    while i < mn_803EB6B0[kind as usize].selection_count as ::core::ffi::c_int {
        if mn_80229938(kind, i as s32) != 0 as ::core::ffi::c_int {
            count = count.wrapping_add(1);
        }
        i += 1;
    }
    return count;
}
pub unsafe extern "C" fn mn_8022B3A0(mut state: u8_0) -> *mut HSD_GObj {
    melee_core::ct!("mn_8022B3A0");
    let mut option_jobjs: [*mut HSD_JObj; 12] = [::core::ptr::null_mut::<
        HSD_JObj,
    >(); 12];
    let mut cursor_parts: [*mut HSD_JObj; 7] = [::core::ptr::null_mut::<HSD_JObj>(); 7];
    let mut i: ::core::ffi::c_int = 0;
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut cursor_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut hover_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut idx: ::core::ffi::c_int = 0;
    let mut unlocked_index: u8_0 = 0;
    let mut hover_anim: *mut AnimLoopSettings = ::core::ptr::null_mut::<
        AnimLoopSettings,
    >();
    let mut option_count: u8_0 = 0;
    let mut cur_menu: u8_0 = 0;
    let mut menu_kind: MenuKind = MENU_KIND_MAIN;
    let mut user_data: *mut MainMenuData = ::core::ptr::null_mut::<MainMenuData>();
    let mut model: *mut StaticModelDesc = &raw mut MenMainConTop_Top;
    let mut hovered_selection: u8_0 = 0;
    let mut anim_loop: *mut AnimLoopSettings = ::core::ptr::null_mut::<
        AnimLoopSettings,
    >();
    let mut jobj: C2RustUnnamed_12 = C2RustUnnamed_12 {
        value: ::core::ptr::null_mut::<HSD_JObj>(),
    };
    let mut unlocked_count: u32_0 = 0;
    let mut root_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut _pad: [::core::ffi::c_uchar; 16] = [0; 16];
    cur_menu = mn_804A04F0.cur_menu as u8_0;
    anim_loop = ((mn_803EB6B0[cur_menu as usize].anim_loop as *mut u8_0)
        .offset(::core::mem::size_of::<AnimLoopSettings>() as usize as isize)
        as *mut AnimLoopSettings)
        .offset(-(1 as ::core::ffi::c_int as isize));
    option_count = sel_count(cur_menu as ::core::ffi::c_int) as u8_0;
    hovered_selection = mn_804A04F0.hovered_selection as u8_0;
    unlocked_count = CountUnlockedSelections(cur_menu as MenuKind) as u32_0;
    gobj = GObj_Create(6 as u16_0, 7 as u8_0, 0x80 as u8_0);
    root_jobj = HSD_JObjLoadJoint((*model).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(
        gobj,
        HSD_GObj_JObjKind,
        root_jobj as *mut ::core::ffi::c_void,
    );
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, isize) -> (),
        ),
        4 as u8_0,
        0x80 as u32_0,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(fn_8022AFEC as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    HSD_JObjAddAnimAll(
        root_jobj,
        (*model).animjoint,
        (*model).matanim_joint,
        (*model).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(root_jobj, 0.0f32);
    user_data = HSD_MemAlloc(::core::mem::size_of::<MainMenuData>() as ssize_t)
        as *mut MainMenuData;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char, &[]);
        __assert(
            b"src/melee/mn/mnmain.c\0" as *const u8 as *const ::core::ffi::c_char,
            1448 as u32_0,
            b"user_data\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    GObj_InitUserData(
        gobj,
        0 as u8_0,
        Some(mn_8022EB04 as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        user_data as *mut ::core::ffi::c_void,
    );
    (*user_data).menu_kind = mn_804A04F0.cur_menu;
    (*user_data).hovered_selection = mn_804A04F0.hovered_selection as MenuKind8;
    (*user_data).state = state as MenuState8;
    (*user_data).description = ::core::ptr::null_mut::<HSD_Text>();
    idx = 0 as ::core::ffi::c_int;
    while idx
        < (::core::mem::size_of::<[*mut HSD_JObj; 42]>() as usize)
            .wrapping_div(::core::mem::size_of::<*mut HSD_JObj>() as usize)
            as ::core::ffi::c_int
    {
        lb_80011E24(root_jobj as *mut HSD_JObj, (&raw mut (*user_data).tree as *mut *mut HSD_JObj).offset(idx as isize)
                as *mut *mut HSD_JObj, &[(idx) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
        idx += 1;
    }
    if (*user_data).state as ::core::ffi::c_int != MENU_STATE_IDLE as ::core::ffi::c_int
    {
        match (*user_data).state as ::core::ffi::c_int {
            1 => {
                HSD_JObjReqAnim(
                    (*user_data).tree[3 as ::core::ffi::c_int as usize],
                    mn_803EB39C.start_frame,
                );
            }
            3 => {
                HSD_JObjReqAnim(
                    (*user_data).tree[3 as ::core::ffi::c_int as usize],
                    mn_803EB3B4.start_frame,
                );
            }
            2 | 4 | 5 | _ => {}
        }
        HSD_JObjAnim((*user_data).tree[3 as ::core::ffi::c_int as usize]);
    }
    i = 0 as ::core::ffi::c_int;
    while i < option_count as ::core::ffi::c_int {
        option_jobjs[i as usize] = (*user_data).tree[mn_803EAE68[i as usize] as usize]
            as *mut HSD_JObj;
        i += 1;
    }
    i = 0 as ::core::ffi::c_int;
    while i < option_count as ::core::ffi::c_int {
        model = &raw mut MenMainCursor_Top;
        if mn_80229938(mn_804A04F0.cur_menu as MenuKind, i as s32)
            != 0 as ::core::ffi::c_int
        {
            menu_kind = mn_804A04F0.cur_menu as MenuKind;
            unlocked_index = mn_80229A04(menu_kind, i) as u8_0;
            HSD_JObjReqAnim(
                option_jobjs[unlocked_index as usize],
                unlocked_count as f32_0,
            );
            HSD_JObjAnim(option_jobjs[unlocked_index as usize]);
            cursor_jobj = HSD_JObjLoadJoint((*model).joint as *mut HSD_Joint);
            HSD_JObjAddAnimAll(
                cursor_jobj,
                (*model).animjoint,
                (*model).matanim_joint,
                (*model).shapeanim_joint,
            );
            lb_8001204C(
                cursor_jobj as *mut HSD_JObj,
                &raw mut cursor_parts as *mut *mut HSD_JObj,
                &raw mut mn_803EAE7C as *mut u16_0,
                (::core::mem::size_of::<[*mut HSD_JObj; 7]>() as usize)
                    .wrapping_div(::core::mem::size_of::<*mut HSD_JObj>() as usize)
                    as ::core::ffi::c_int,
            );
            HSD_JObjReqAnim(
                cursor_parts[0 as ::core::ffi::c_int as usize],
                mn_803EB360[(i == hovered_selection as ::core::ffi::c_int)
                        as ::core::ffi::c_int as usize]
                    .start_frame,
            );
            HSD_JObjAnim(cursor_parts[0 as ::core::ffi::c_int as usize]);
            jobj.value = cursor_parts[1 as ::core::ffi::c_int as usize];
            HSD_JObjReqAnimAll(
                jobj.value,
                (i == hovered_selection as ::core::ffi::c_int) as ::core::ffi::c_int
                    as f32_0,
            );
            HSD_JObjAnimAll(jobj.value);
            HSD_JObjReqAnim(
                jobj.value,
                mn_803EB6B0[mn_804A04F0.cur_menu as usize].start_frame as f32_0
                    + GetSelectionFrameOffset(i) as f32_0,
            );
            mn_8022F3D8(jobj.value as *mut HSD_JObj, 0xc as u8_0, TOBJ_MASK);
            mn_8022F3D8(jobj.value as *mut HSD_JObj, 0xd as u8_0, TOBJ_MASK);
            mn_8022F3D8(jobj.value as *mut HSD_JObj, 0xe as u8_0, TOBJ_MASK);
            mn_8022F3D8(jobj.value as *mut HSD_JObj, 0xf as u8_0, TOBJ_MASK);
            mn_8022F3D8(jobj.value as *mut HSD_JObj, 0x10 as u8_0, TOBJ_MASK);
            mn_8022F3D8(jobj.value as *mut HSD_JObj, 0x11 as u8_0, TOBJ_MASK);
            mn_8022F3D8(jobj.value as *mut HSD_JObj, 0x12 as u8_0, TOBJ_MASK);
            mn_8022F3D8(jobj.value as *mut HSD_JObj, 0x13 as u8_0, TOBJ_MASK);
            HSD_JObjAnim(jobj.value);
            if i == hovered_selection as ::core::ffi::c_int {
                HSD_JObjReqAnimAll(
                    cursor_parts[2 as ::core::ffi::c_int as usize],
                    mn_803EB378[(i == hovered_selection as ::core::ffi::c_int)
                            as ::core::ffi::c_int as usize]
                        .start_frame,
                );
            } else {
                HSD_JObjReqAnimAll(
                    cursor_parts[2 as ::core::ffi::c_int as usize],
                    mn_803EB378[(i == hovered_selection as ::core::ffi::c_int)
                            as ::core::ffi::c_int as usize]
                        .end_frame,
                );
            }
            HSD_JObjAnimAll(cursor_parts[2 as ::core::ffi::c_int as usize]);
            if i == hovered_selection as ::core::ffi::c_int {
                HSD_JObjReqAnim(
                    cursor_parts[3 as ::core::ffi::c_int as usize],
                    mn_803EB390.start_frame,
                );
            } else {
                HSD_JObjSetFlagsAll(
                    cursor_parts[3 as ::core::ffi::c_int as usize],
                    JOBJ_HIDDEN as u32_0,
                );
            }
            HSD_JObjAnim(cursor_parts[3 as ::core::ffi::c_int as usize]);
            if i != hovered_selection as ::core::ffi::c_int {
                HSD_JObjSetFlagsAll(
                    cursor_parts[4 as ::core::ffi::c_int as usize],
                    JOBJ_HIDDEN as u32_0,
                );
            }
            if envoy_main6(cur_menu as ::core::ffi::c_int) && i == ENVOY_SEL {
                envoy_hide_label(cursor_jobj);
            }
            HSD_JObjAddChild(option_jobjs[unlocked_index as usize], cursor_jobj);
        }
        i += 1;
    }
    hover_jobj = (*user_data).tree[14 as ::core::ffi::c_int as usize] as *mut HSD_JObj;
    if (*user_data).menu_kind as ::core::ffi::c_int
        == MENU_KIND_MAIN as ::core::ffi::c_int
        && (*user_data).hovered_selection as ::core::ffi::c_int
            == SEL_MAIN_DATA as ::core::ffi::c_int
        && gmMainLib_8015EE90() == 0 as ::core::ffi::c_int
    {
        hover_anim = &raw mut mn_803EB438;
    } else if (*user_data).menu_kind as ::core::ffi::c_int
        == MENU_KIND_1P as ::core::ffi::c_int
        && (*user_data).hovered_selection as ::core::ffi::c_int
            == SEL_1P_REG as ::core::ffi::c_int
        && gmMainLib_8015EDD4() == 0 as ::core::ffi::c_int
    {
        hover_anim = &raw mut mn_803EB480;
    } else {
        hover_anim = anim_loop.offset(envoy_anim_sel((*user_data).menu_kind as ::core::ffi::c_int, (*user_data).hovered_selection as ::core::ffi::c_int) as isize)
            as *mut AnimLoopSettings;
    }
    HSD_JObjReqAnimAll(hover_jobj, (*hover_anim).start_frame);
    HSD_JObjAnimAll(hover_jobj);
    mn_80229A7C(
        user_data,
        (*user_data).menu_kind as MenuKind,
        (*user_data).hovered_selection as ::core::ffi::c_int,
    );
    return gobj as *mut HSD_GObj;
}
pub static mut mn_803B8500: Vec3 = Vec3 {
    x: 0 as ::core::ffi::c_int as f32_0,
    y: 1 as ::core::ffi::c_int as f32_0,
    z: 0 as ::core::ffi::c_int as f32_0,
};
pub unsafe extern "C" fn mn_8022BA1C(mut gp: *mut HSD_GObj) {
    melee_core::ct!("mn_8022BA1C");
    let mut _pad: [u8_0; 16] = [0; 16];
    let mut sp54: Mtx = [[0.; 4]; 3];
    let mut sp48: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut sp3C: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut sp30: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut sp24: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut sp18: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut _2: [u8_0; 8] = [0; 8];
    let mut var_f0: ::core::ffi::c_float = 0.;
    let mut x: ::core::ffi::c_float = 0.;
    let mut var_f31: ::core::ffi::c_float = 0.;
    let mut var_f30: ::core::ffi::c_float = 0.;
    let mut y: ::core::ffi::c_float = 0.;
    let mut new_var: *mut HSD_PadStatus = ::core::ptr::null_mut::<HSD_PadStatus>();
    let mut i: ::core::ffi::c_int = 0;
    let mut cobj: *mut HSD_CObj = HSD_GObjGetHSDObj(gp) as *mut HSD_CObj;
    new_var = &raw mut HSD_PadCopyStatus as *mut HSD_PadStatus;
    i = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        x = (*new_var.offset(i as u8_0 as isize)).nml_subStickX as ::core::ffi::c_float;
        y = (*new_var.offset(i as u8_0 as isize)).nml_subStickY as ::core::ffi::c_float;
        if !(-0.4f32 < x && x < 0.4f32 && -0.4f32 < y && y < 0.4f32) {
            break;
        }
        i += 1;
    }
    if -0.4f32 < x && x < 0.4f32 && -0.4f32 < y && y < 0.4f32 {
        var_f30 = 0.0f32;
        var_f31 = 0.0f32;
    } else {
        if -0.4f32 < x && x < 0.4f32 {
            var_f31 = 0.0f32;
        } else {
            if x < 0.0f32 {
                var_f0 = -0.4f32;
            } else {
                var_f0 = 0.4f32;
            }
            var_f31 = 30.0f32 * ((x - var_f0) / 0.6f32);
        }
        if -0.4f32 < y && y < 0.4f32 {
            var_f30 = 0.0f32;
        } else {
            if y < 0.0f32 {
                var_f0 = -0.4f32;
            } else {
                var_f0 = 0.4f32;
            }
            var_f30 = 30.0f32 * ((y - var_f0) / 0.6f32);
        }
    }
    HSD_CObjInit(cobj, MenMain_cam);
    sp30 = mn_803B8500;
    HSD_CObjGetEyeVector(cobj, &raw mut sp48);
    HSD_CObjGetInterest(cobj, &raw mut sp18);
    PSVECCrossProduct(&raw mut sp30, &raw mut sp48, &raw mut sp3C);
    PSVECNormalize(&raw mut sp3C, &raw mut sp3C);
    PSMTXRotAxisRad(
        &raw mut sp54 as *mut [f32_0; 4],
        &raw mut sp3C,
        0.017453292f32 * var_f30 as f32_0,
    );
    PSMTXMultVec(&raw mut sp54 as *mut [f32_0; 4], &raw mut sp48, &raw mut sp48);
    PSMTXRotAxisRad(
        &raw mut sp54 as *mut [f32_0; 4],
        &raw mut sp30,
        0.017453292f32 * var_f31 as f32_0,
    );
    PSMTXMultVec(&raw mut sp54 as *mut [f32_0; 4], &raw mut sp48, &raw mut sp48);
    PSVECScale(&raw mut sp48, &raw mut sp48, HSD_CObjGetEyeDistance(cobj));
    PSVECSubtract(&raw mut sp18, &raw mut sp48, &raw mut sp24);
    HSD_CObjSetEyePosition(cobj, &raw mut sp24);
}
pub unsafe extern "C" fn fn_8022BCD4(
    mut gobj: *mut HSD_GObj,
    mut unused: isize,
) {
    melee_core::ct!("fn_8022BCD4");
    HSD_FogSet((*gobj).hsd_obj as *mut HSD_Fog);
}
pub unsafe extern "C" fn mn_8022BCF8() -> *mut HSD_GObj {
    melee_core::ct!("mn_8022BCF8");
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut fog: *mut HSD_Fog = ::core::ptr::null_mut::<HSD_Fog>();
    gobj = GObj_Create(1 as u16_0, 2 as u8_0, 0x80 as u8_0);
    mn_804D6BA8 = gobj;
    fog = HSD_FogLoadDesc(MenMain_fog);
    HSD_GObjObject_80390A70(
        gobj,
        HSD_GObj_FogKind as u8_0,
        fog as *mut ::core::ffi::c_void,
    );
    GObj_SetupGXLink(
        gobj,
        Some(
            fn_8022BCD4 as unsafe extern "C" fn(*mut HSD_GObj, isize) -> (),
        ),
        1 as u8_0,
        0x80 as u32_0,
    );
    return gobj as *mut HSD_GObj;
}
pub unsafe extern "C" fn mn_8022BD6C() {
    melee_core::ct!("mn_8022BD6C");
    (*mn_804D6BAC).gxlink_prios = 0 as u64_0;
    (*mn_804D6BB0).gxlink_prios = 0 as u64_0;
}
pub unsafe extern "C" fn mn_8022BD8C() {
    melee_core::ct!("mn_8022BD8C");
    (*mn_804D6BAC).gxlink_prios = 0x7f as u64_0;
    (*mn_804D6BB0).gxlink_prios = 0x80 as u64_0;
}
pub unsafe extern "C" fn fn_8022BDB4(
    mut gobj: *mut HSD_GObj,
    mut unused: isize,
) {
    melee_core::ct!("fn_8022BDB4");
    let mut fog: *mut HSD_Fog = HSD_GObjGetHSDObj(mn_804D6BA8) as *mut HSD_Fog;
    if HSD_CObjSetCurrent(HSD_GObjGetHSDObj(gobj) as *mut HSD_CObj)
        != 0 as ::core::ffi::c_int
    {
        HSD_SetEraseColor(
            (*fog).color.r,
            (*fog).color.g,
            (*fog).color.b,
            (*fog).color.a,
        );
        HSD_CObjEraseScreen(
            HSD_GObjGetHSDObj(gobj) as *mut HSD_CObj,
            1 as s32,
            0 as s32,
            1 as s32,
        );
        HSD_GObj_80390ED0(gobj, 7 as u32_0);
        HSD_CObjEndCurrent();
    }
}
pub unsafe extern "C" fn mn_8022BE34() -> *mut HSD_GObj {
    melee_core::ct!("mn_8022BE34");
    let mut gobj: *mut HSD_GObj = GObj_Create(2 as u16_0, 3 as u8_0, 0x80 as u8_0);
    let mut cobj: *mut HSD_CObj = ::core::ptr::null_mut::<HSD_CObj>();
    let mut pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    mn_804D6BAC = gobj;
    cobj = HSD_CObjLoadDesc(MenMain_cam);
    HSD_CObjGetEyePosition(cobj, &raw mut pos);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_CameraKind, cobj as *mut ::core::ffi::c_void);
    GObj_SetupGXLinkMax(
        gobj,
        Some(
            fn_8022BDB4 as unsafe extern "C" fn(*mut HSD_GObj, isize) -> (),
        ),
        0 as u32_0,
    );
    (*gobj).gxlink_prios = 0x7f as u64_0;
    HSD_GObj_SetupProc(
        gobj,
        Some(mn_8022BA1C as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    return gobj as *mut HSD_GObj;
}
#[inline]
pub unsafe extern "C" fn mn_8022BE34_OnEnter(mut pos: *mut Vec3) -> *mut HSD_GObj {
    melee_core::ct!("mn_8022BE34_OnEnter");
    let mut gobj: *mut HSD_GObj = GObj_Create(2 as u16_0, 3 as u8_0, 0x80 as u8_0);
    let mut cobj: *mut HSD_CObj = ::core::ptr::null_mut::<HSD_CObj>();
    mn_804D6BAC = gobj;
    cobj = HSD_CObjLoadDesc(MenMain_cam);
    HSD_CObjGetEyePosition(cobj, pos);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_CameraKind, cobj as *mut ::core::ffi::c_void);
    GObj_SetupGXLinkMax(
        gobj,
        Some(
            fn_8022BDB4 as unsafe extern "C" fn(*mut HSD_GObj, isize) -> (),
        ),
        0 as u32_0,
    );
    (*gobj).gxlink_prios = 0x7f as u64_0;
    HSD_GObj_SetupProc(
        gobj,
        Some(mn_8022BA1C as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    return gobj;
}
pub unsafe extern "C" fn mn_8022BEDC(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mn_8022BEDC");
    let mut temp_r3: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut cobj: *mut HSD_CObj = ::core::ptr::null_mut::<HSD_CObj>();
    mn_804D6BB5 = HSD_SisLib_803A611C(
        0 as ::core::ffi::c_int,
        gobj,
        7 as u16_0,
        8 as u8_0,
        0x80 as u8_0,
        5 as u8_0,
        0x80 as u8_0,
        0 as u32_0,
    ) as u8_0;
    temp_r3 = GObj_Create(2 as u16_0, 3 as u8_0, 0x80 as u8_0);
    mn_804D6BB0 = temp_r3;
    cobj = HSD_CObjLoadDesc(MenMain_cam);
    HSD_GObjObject_80390A70(
        temp_r3,
        HSD_GObj_CameraKind,
        cobj as *mut ::core::ffi::c_void,
    );
    GObj_SetupGXLinkMax(
        temp_r3,
        Some(
            HSD_GObj_803910D8
                as unsafe extern "C" fn(*mut HSD_GObj, isize) -> (),
        ),
        0 as u32_0,
    );
    (*temp_r3).gxlink_prios = 0x80 as u64_0;
    HSD_GObj_SetupProc(
        temp_r3,
        Some(mn_8022BA1C as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    mn_804D6BB4 = HSD_SisLib_803A611C(
        0 as ::core::ffi::c_int,
        temp_r3,
        7 as u16_0,
        8 as u8_0,
        0x80 as u8_0,
        7 as u8_0,
        0x80 as u8_0,
        0 as u32_0,
    ) as u8_0;
}
pub unsafe extern "C" fn mn_8022BFBC(mut arg0: ::core::ffi::c_int) -> *mut GXColor {
    melee_core::ct!("mn_8022BFBC");
    match arg0 {
        0 => return &raw mut mn_804D4B50,
        1 => return &raw mut mn_804D4B54,
        2 => return &raw mut mn_804D4B58,
        3 => return &raw mut mn_804D4B5C,
        4 => return &raw mut mn_804D4B60,
        // port: the Envoy row's light (envoy_main6), Party pack sunny
        5 => return &raw mut ENVOY_LIGHT,
        _ => {}
    }
    panic!("Reached end of non-void function without returning");
}
pub static mut ENVOY_LIGHT: GXColor = _GXColor { r: 0xff as u8_0, g: 0xd8 as u8_0, b: 0x4a as u8_0, a: 0xff as u8_0 };
pub unsafe extern "C" fn mn_8022C010(
    mut menu_kind: ::core::ffi::c_int,
    mut selection: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    melee_core::ct!("mn_8022C010");
    if menu_kind == MENU_KIND_MAIN as ::core::ffi::c_int {
        return selection;
    }
    match menu_kind {
        1 | 6 | 7 | 8 | 9 | 10 | 33 => return 0 as ::core::ffi::c_int,
        2 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 => return 1 as ::core::ffi::c_int,
        3 => return 2 as ::core::ffi::c_int,
        4 | 19 | 20 | 21 | 22 | 23 | 24 => return 3 as ::core::ffi::c_int,
        5 | 25 | 26 | 27 | 28 | 29 | 30 | 31 | 32 => return 4 as ::core::ffi::c_int,
        _ => {}
    }
    panic!("Reached end of non-void function without returning");
}
#[inline]
pub unsafe extern "C" fn LerpLightColor(
    mut lobj: *mut HSD_LObj,
    mut div: ::core::ffi::c_int,
) {
    melee_core::ct!("LerpLightColor");
    let mut diff: ::core::ffi::c_int = 0;
    while (*lobj).flags as ::core::ffi::c_int & LOBJ_POINT == 0 {
        lobj = (*lobj).next;
    }
    if div != 0 as ::core::ffi::c_int {
        diff = (*mn_804A04F0.light_color).r as ::core::ffi::c_int
            - (*lobj).color.r as ::core::ffi::c_int;
        if diff != 0 as ::core::ffi::c_int {
            (*lobj).color.r = ((*lobj).color.r as ::core::ffi::c_int + <i32 as melee_core::ppc::Ppc>::pdiv(diff, div))
                as u8_0;
        }
        diff = (*mn_804A04F0.light_color).g as ::core::ffi::c_int
            - (*lobj).color.g as ::core::ffi::c_int;
        if diff != 0 as ::core::ffi::c_int {
            (*lobj).color.g = ((*lobj).color.g as ::core::ffi::c_int + <i32 as melee_core::ppc::Ppc>::pdiv(diff, div))
                as u8_0;
        }
        diff = (*mn_804A04F0.light_color).b as ::core::ffi::c_int
            - (*lobj).color.b as ::core::ffi::c_int;
        if diff != 0 as ::core::ffi::c_int {
            (*lobj).color.b = ((*lobj).color.b as ::core::ffi::c_int + <i32 as melee_core::ppc::Ppc>::pdiv(diff, div))
                as u8_0;
        }
    } else {
        (*lobj).color.r = (*mn_804A04F0.light_color).r;
        (*lobj).color.g = (*mn_804A04F0.light_color).g;
        (*lobj).color.b = (*mn_804A04F0.light_color).b;
    };
}
pub unsafe extern "C" fn mn_8022C068(
    mut lobj: *mut HSD_LObj,
    mut unused: ::core::ffi::c_int,
    mut div: ::core::ffi::c_int,
) {
    melee_core::ct!("mn_8022C068");
    let mut diff: ::core::ffi::c_int = 0;
    while (*lobj).flags as ::core::ffi::c_int & LOBJ_POINT == 0 {
        lobj = (*lobj).next;
    }
    if div != 0 as ::core::ffi::c_int {
        diff = (*mn_804A04F0.light_color).r as ::core::ffi::c_int
            - (*lobj).color.r as ::core::ffi::c_int;
        if diff != 0 as ::core::ffi::c_int {
            (*lobj).color.r = ((*lobj).color.r as ::core::ffi::c_int + <i32 as melee_core::ppc::Ppc>::pdiv(diff, div))
                as u8_0;
        }
        diff = (*mn_804A04F0.light_color).g as ::core::ffi::c_int
            - (*lobj).color.g as ::core::ffi::c_int;
        if diff != 0 as ::core::ffi::c_int {
            (*lobj).color.g = ((*lobj).color.g as ::core::ffi::c_int + <i32 as melee_core::ppc::Ppc>::pdiv(diff, div))
                as u8_0;
        }
        diff = (*mn_804A04F0.light_color).b as ::core::ffi::c_int
            - (*lobj).color.b as ::core::ffi::c_int;
        if diff != 0 as ::core::ffi::c_int {
            (*lobj).color.b = ((*lobj).color.b as ::core::ffi::c_int + <i32 as melee_core::ppc::Ppc>::pdiv(diff, div))
                as u8_0;
        }
    } else {
        (*lobj).color.r = (*mn_804A04F0.light_color).r;
        (*lobj).color.g = (*mn_804A04F0.light_color).g;
        (*lobj).color.b = (*mn_804A04F0.light_color).b;
    };
}
pub unsafe extern "C" fn fn_8022C128(mut arg0: *mut HSD_GObj) {
    melee_core::ct!("fn_8022C128");
    let mut color: *mut GXColor = ::core::ptr::null_mut::<GXColor>();
    let mut lobj: *mut HSD_LObj = HSD_GObjGetHSDObj(arg0) as *mut HSD_LObj;
    HSD_LObjAnimAll(lobj);
    color = mn_8022BFBC(
        mn_8022C010(
            mn_804A04F0.cur_menu as ::core::ffi::c_int,
            mn_804A04F0.hovered_selection as ::core::ffi::c_int,
        ),
    );
    if color != mn_804A04F0.light_color {
        mn_804A04F0.light_lerp_frames = 0x10 as u8_0;
        mn_804A04F0.light_color = color;
    }
    if mn_804A04F0.light_lerp_frames as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        mn_804A04F0.light_lerp_frames = mn_804A04F0.light_lerp_frames.wrapping_sub(1);
        LerpLightColor(lobj, mn_804A04F0.light_lerp_frames as ::core::ffi::c_int);
    }
}
pub unsafe extern "C" fn mn_8022C304() {
    melee_core::ct!("mn_8022C304");
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut lobj: *mut HSD_LObj = ::core::ptr::null_mut::<HSD_LObj>();
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    gobj = GObj_Create(3 as u16_0, 4 as u8_0, 0x80 as u8_0);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_LObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, isize) -> (),
        ),
        0 as u8_0,
        0x80 as u32_0,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(fn_8022C128 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    lobj = lb_80011AC4(MenMain_lights);
    HSD_GObjObject_80390A70(
        gobj,
        HSD_GObj_LightKind as u8_0,
        lobj as *mut ::core::ffi::c_void,
    );
    mn_804A04F0.light_lerp_frames = 0 as u8_0;
    mn_804A04F0.light_color = mn_8022BFBC(
        mn_8022C010(
            mn_804A04F0.cur_menu as ::core::ffi::c_int,
            mn_804A04F0.hovered_selection as ::core::ffi::c_int,
        ),
    );
    LerpLightColor(lobj, mn_804A04F0.light_lerp_frames as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn x2_dec(mut temp_r29: u8_0) {
    melee_core::ct!("x2_dec");
    loop {
        if mn_804A04F0.hovered_selection as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            mn_804A04F0.hovered_selection = mn_804A04F0
                .hovered_selection
                .wrapping_sub(1);
        } else {
            mn_804A04F0.hovered_selection = (temp_r29 as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as u16_0;
        }
        if !(mn_80229938(
            mn_804A04F0.cur_menu as MenuKind,
            mn_804A04F0.hovered_selection as s32,
        ) == 0)
        {
            break;
        }
    };
}
#[inline]
pub unsafe extern "C" fn x2_inc(mut temp_r29: u8_0) {
    melee_core::ct!("x2_inc");
    loop {
        if mn_804A04F0.hovered_selection as ::core::ffi::c_int
            == temp_r29 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        {
            mn_804A04F0.hovered_selection = 0 as u16_0;
        } else {
            mn_804A04F0.hovered_selection = mn_804A04F0
                .hovered_selection
                .wrapping_add(1);
        }
        if !(mn_80229938(
            mn_804A04F0.cur_menu as MenuKind,
            mn_804A04F0.hovered_selection as s32,
        ) == 0)
        {
            break;
        }
    };
}
#[inline]
pub unsafe extern "C" fn decrement_selection(mut selection_count: u8_0, mut menu_kind: s32) {
    melee_core::ct!("decrement_selection");
    loop {
        if mn_804A04F0.hovered_selection as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            mn_804A04F0.hovered_selection = mn_804A04F0
                .hovered_selection
                .wrapping_sub(1);
        } else {
            mn_804A04F0.hovered_selection = (selection_count as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as u16_0;
        }
        if !(mn_80229938(menu_kind as MenuKind, mn_804A04F0.hovered_selection as s32)
            == 0 as ::core::ffi::c_int)
        {
            break;
        }
    };
}
#[inline]
pub unsafe extern "C" fn increment_selection(
    mut selection_count: u8_0,
    mut menu_kind: MenuKind,
) {
    melee_core::ct!("increment_selection");
    loop {
        if mn_804A04F0.hovered_selection as ::core::ffi::c_int
            == selection_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        {
            mn_804A04F0.hovered_selection = 0 as u16_0;
        } else {
            mn_804A04F0.hovered_selection = mn_804A04F0
                .hovered_selection
                .wrapping_add(1);
        }
        if !(mn_80229938(menu_kind, mn_804A04F0.hovered_selection as s32)
            == 0 as ::core::ffi::c_int)
        {
            break;
        }
    };
}
pub unsafe extern "C" fn mn_8022C4F4(mut gp: *mut HSD_GObj) {
    melee_core::ct!("mn_8022C4F4");
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut temp_r3_2: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut selection_count: ::core::ffi::c_int = mn_803EB6B0[MENU_KIND_SPECIAL
            as ::core::ffi::c_int as usize]
        .selection_count as ::core::ffi::c_int & 0xff as ::core::ffi::c_int;
    let mut data: *mut MenuExitData = ::core::ptr::null_mut::<MenuExitData>();
    let mut temp_r28: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut buttons: u32_0 = mn_80229624(4 as u32_0);
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    mn_804A04F0.buttons = buttons as u64_0;
    if buttons & MenuInput_Confirm as ::core::ffi::c_int as u32_0 != 0 {
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.entering_menu = 1 as u8_0;
        match mn_804A04F0.hovered_selection as ::core::ffi::c_int {
            0 => {
                if crate::menuguard::deny(GM_CAMERA_MODE as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_CAMERA_MODE as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            1 => {
                if crate::menuguard::deny(GM_STAMINA_VS as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_STAMINA_VS as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            2 => {
                if crate::menuguard::deny(GM_SUPER_SUDDEN_DEATH_VS as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_SUPER_SUDDEN_DEATH_VS as ::core::ffi::c_int
                    as s8;
                gm_801A4B60();
            }
            3 => {
                if crate::menuguard::deny(GM_GIANT_VS as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_GIANT_VS as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            4 => {
                if crate::menuguard::deny(GM_TINY_VS as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_TINY_VS as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            5 => {
                if crate::menuguard::deny(GM_INVISIBLE_VS as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_INVISIBLE_VS as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            6 => {
                if crate::menuguard::deny(GM_CAMERA_VS as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_CAMERA_VS as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            7 => {
                if crate::menuguard::deny(GM_SINGLE_BUTTON_VS as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_SINGLE_BUTTON_VS as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            8 => {
                if crate::menuguard::deny(GM_LIGHTNING_VS as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_LIGHTNING_VS as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            9 => {
                if crate::menuguard::deny(GM_SLOMO_VS as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_SLOMO_VS as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            _ => {}
        }
    } else if buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
        mn_804A04F0.cur_menu = MENU_KIND_VS as ::core::ffi::c_int as MenuKind8;
        mn_804A04F0.hovered_selection = SEL_VS_SPECIAL as ::core::ffi::c_int as u16_0;
        HSD_GObj_80390CD4(mn_8022B3A0(3 as u8_0) as *mut HSD_GObj);
        HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
        temp_r28 = mn_803EB6B0[MENU_KIND_VS as ::core::ffi::c_int as usize].think
            as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
        if temp_r28.is_some() {
            gobj = GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0);
            temp_r3_2 = HSD_GObj_SetupProc(gobj, temp_r28 as HSD_GObjEvent, 0 as u8_0);
            (*temp_r3_2).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        }
    } else if buttons & MenuInput_Up as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        x2_dec(selection_count as u8_0);
    } else if buttons & MenuInput_Down as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        x2_inc(selection_count as u8_0);
    }
}
#[inline]
pub unsafe extern "C" fn mn_8022C7CC_inline() -> u8_0 {
    melee_core::ct!("mn_8022C7CC_inline");
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        if gm_GetButtonsTriggered(i as u8_0) as ::core::ffi::c_ulonglong & PAD_CONFIRM
            != 0
        {
            return i as u8_0;
        }
        i += 1;
    }
    return 0 as u8_0;
}
pub unsafe extern "C" fn mn_8022C7CC(mut gp: *mut HSD_GObj) {
    melee_core::ct!("mn_8022C7CC");
    let mut think: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut data: *mut MenuExitData = ::core::ptr::null_mut::<MenuExitData>();
    let mut selection_count: ::core::ffi::c_int = mn_803EB6B0[MENU_KIND_STADIUM
            as ::core::ffi::c_int as usize]
        .selection_count as ::core::ffi::c_int & 0xff as ::core::ffi::c_int;
    let mut temp_r28: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut buttons: u32_0 = mn_80229624(4 as u32_0);
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    mn_804A04F0.buttons = buttons as u64_0;
    if buttons & MenuInput_Confirm as ::core::ffi::c_int as u32_0 != 0 {
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.entering_menu = 1 as u8_0;
        match mn_804A04F0.hovered_selection as ::core::ffi::c_int {
            0 => {
                if crate::menuguard::deny(GM_TARGET_TEST as i32) { return; } // menuguard: not ported yet
                sfxForward();
                gm_801677E8(mn_8022C7CC_inline() as s8);
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_TARGET_TEST as ::core::ffi::c_int as s8;
                gm_801A4B60();
                return;
            }
            1 => {
                if crate::menuguard::deny(GM_HOME_RUN_CONTEST as i32) { return; } // menuguard: not ported yet
                sfxForward();
                gm_801677E8(mn_8022C7CC_inline() as s8);
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_HOME_RUN_CONTEST as ::core::ffi::c_int as s8;
                gm_801A4B60();
                return;
            }
            2 => {
                sfxForward();
                mnHyaku_8024CD64(0 as u8_0);
                HSD_GObjFree(gp);
                return;
            }
            _ => {}
        }
    } else if buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
        mn_804A04F0.cur_menu = MENU_KIND_1P as ::core::ffi::c_int as MenuKind8;
        mn_804A04F0.hovered_selection = SEL_1P_STADIUM as ::core::ffi::c_int as u16_0;
        HSD_GObj_80390CD4(mn_8022B3A0(3 as u8_0) as *mut HSD_GObj);
        HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
        temp_r28 = mn_803EB6B0[1 as ::core::ffi::c_int as usize].think
            as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
        if temp_r28.is_some() {
            let mut gobj: *mut HSD_GObj = GObj_Create(
                0 as u16_0,
                1 as u8_0,
                0x80 as u8_0,
            );
            think = HSD_GObj_SetupProc(gobj, temp_r28 as HSD_GObjEvent, 0 as u8_0);
            (*think).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        }
    } else if buttons & MenuInput_Up as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        x2_dec(selection_count as u8_0);
    } else if buttons & MenuInput_Down as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        x2_inc(selection_count as u8_0);
    }
}
pub unsafe extern "C" fn mn_8022CA54(mut gp: *mut HSD_GObj) {
    melee_core::ct!("mn_8022CA54");
    let mut think: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut buttons: u32_0 = 0;
    let mut temp_r30: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    buttons = mn_80229624(4 as u32_0);
    mn_804A04F0.buttons = buttons as u64_0;
    if buttons & MenuInput_Confirm as ::core::ffi::c_int as u32_0 != 0 {
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.entering_menu = 1 as u8_0;
        match mn_804A04F0.hovered_selection as RecordsMenuSelection as ::core::ffi::c_int
        {
            0 => {
                sfxForward();
                mnDiagram_Init(1 as u8_0, 1 as u8_0);
                HSD_GObjFree(gp);
                return;
            }
            1 => {
                sfxForward();
                mnInfoBonus_80252F8C();
                HSD_GObjFree(gp);
                return;
            }
            2 => {
                sfxForward();
                mnCount_Create();
                HSD_GObjFree(gp);
                return;
            }
            _ => {}
        }
    } else if buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
        mn_804A04F0.cur_menu = MENU_KIND_DATA as ::core::ffi::c_int as MenuKind8;
        mn_804A04F0.hovered_selection = SEL_DATA_RECORDS as u16_0;
        HSD_GObj_80390CD4(mn_8022B3A0(3 as u8_0) as *mut HSD_GObj);
        HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
        temp_r30 = mn_803EB6B0[MENU_KIND_DATA as ::core::ffi::c_int as usize].think
            as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
        if temp_r30.is_some() {
            think = HSD_GObj_SetupProc(
                GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                temp_r30 as HSD_GObjEvent,
                0 as u8_0,
            );
            (*think).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        }
    } else if buttons & MenuInput_Up as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        if mn_804A04F0.hovered_selection as RecordsMenuSelection as ::core::ffi::c_int
            == SEL_RECORDS_VS as ::core::ffi::c_int
        {
            mn_804A04F0.hovered_selection = SEL_RECORDS_MISC as u16_0;
        } else {
            mn_804A04F0.hovered_selection = mn_804A04F0
                .hovered_selection
                .wrapping_sub(1);
        }
    } else if buttons & MenuInput_Down as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        if mn_804A04F0.hovered_selection as RecordsMenuSelection as ::core::ffi::c_int
            == SEL_RECORDS_MISC as ::core::ffi::c_int
        {
            mn_804A04F0.hovered_selection = SEL_RECORDS_VS as u16_0;
        } else {
            mn_804A04F0.hovered_selection = mn_804A04F0
                .hovered_selection
                .wrapping_add(1);
        }
    }
}
pub unsafe extern "C" fn mn_8022CC28(mut gp: *mut HSD_GObj) {
    melee_core::ct!("mn_8022CC28");
    let mut think: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut buttons: u32_0 = 0;
    let mut data: *mut MenuExitData = ::core::ptr::null_mut::<MenuExitData>();
    let mut temp_r28: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    buttons = mn_80229624(4 as u32_0);
    mn_804A04F0.buttons = buttons as u64_0;
    if buttons & MenuInput_Confirm as ::core::ffi::c_int as u32_0 != 0 {
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.entering_menu = 1 as u8_0;
        gm_801677E8(mn_8022C7CC_inline() as s8);
        match mn_804A04F0.hovered_selection as RegMatchMenuSelection
            as ::core::ffi::c_int
        {
            0 => {
                if crate::menuguard::deny(GM_CLASSIC as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_CLASSIC as ::core::ffi::c_int as s8;
                gm_801A4B60();
                return;
            }
            1 => {
                if crate::menuguard::deny(GM_ADVENTURE as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_ADVENTURE as ::core::ffi::c_int as s8;
                gm_801A4B60();
                return;
            }
            2 => {
                if crate::menuguard::deny(GM_ALLSTAR as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_ALLSTAR as ::core::ffi::c_int as s8;
                gm_801A4B60();
                return;
            }
            _ => {}
        }
    } else if buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
        mn_804A04F0.cur_menu = MENU_KIND_1P as ::core::ffi::c_int as MenuKind8;
        mn_804A04F0.hovered_selection = SEL_1P_REG as u16_0;
        HSD_GObj_80390CD4(mn_8022B3A0(3 as u8_0) as *mut HSD_GObj);
        HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
        temp_r28 = mn_803EB6B0[MENU_KIND_1P as ::core::ffi::c_int as usize].think
            as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
        if temp_r28.is_some() {
            think = HSD_GObj_SetupProc(
                GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                temp_r28 as HSD_GObjEvent,
                0 as u8_0,
            );
            (*think).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        }
    } else if buttons & MenuInput_Up as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as ::core::ffi::c_int
                != SEL_REG_CLASSIC as ::core::ffi::c_int
            {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_sub(1);
            } else {
                mn_804A04F0.hovered_selection = SEL_REG_ALLSTAR as u16_0;
            }
            if !(mn_80229938(MENU_KIND_REG, mn_804A04F0.hovered_selection as s32)
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    } else if buttons & MenuInput_Down as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as RegMatchMenuSelection
                as ::core::ffi::c_int == SEL_REG_ALLSTAR as ::core::ffi::c_int
            {
                mn_804A04F0.hovered_selection = SEL_REG_CLASSIC as u16_0;
            } else {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_add(1);
            }
            if !(mn_80229938(MENU_KIND_REG, mn_804A04F0.hovered_selection as s32)
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    }
}
pub unsafe extern "C" fn mn_8022CE6C(mut gp: *mut HSD_GObj) {
    melee_core::ct!("mn_8022CE6C");
    let mut think: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut think2: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut buttons: u32_0 = 0;
    let mut temp_r29: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut temp_r29_2: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    buttons = mn_80229624(4 as u32_0);
    mn_804A04F0.buttons = buttons as u64_0;
    if buttons & MenuInput_Confirm as ::core::ffi::c_int as u32_0 != 0 {
        mn_804A04F0.entering_menu = 1 as u8_0;
        match mn_804A04F0.hovered_selection as DataMenuSelection as ::core::ffi::c_int {
            0 => {
                sfxForward();
                mnSnap_80257F24();
                HSD_GObjFree(gp);
            }
            1 => {
                sfxForward();
                mnGallery_80259868();
                HSD_GObjFree(gp);
            }
            2 => {
                lbAudioAx_80023694();
                lbAudioAx_800236DC();
                mnSoundTest_8024BEE0(1 as s32);
                HSD_GObjFree(gp);
            }
            3 => {
                sfxForward();
                mn_804D6BC8.cooldown = 5 as u16_0;
                mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
                mn_804A04F0.cur_menu = MENU_KIND_RECORDS as ::core::ffi::c_int
                    as MenuKind8;
                mn_804A04F0.hovered_selection = 0 as u16_0;
                HSD_GObj_80390CD4(mn_8022B3A0(1 as u8_0) as *mut HSD_GObj);
                HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
                temp_r29 = mn_803EB6B0[28 as ::core::ffi::c_int as usize].think
                    as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
                if temp_r29.is_some() {
                    think = HSD_GObj_SetupProc(
                        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                        temp_r29 as HSD_GObjEvent,
                        0 as u8_0,
                    );
                    (*think).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
                }
            }
            4 => {
                sfxForward();
                mnInfo_80252758();
                HSD_GObjFree(gp);
            }
            _ => {}
        }
    } else if buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
        mn_804A04F0.cur_menu = MENU_KIND_MAIN as ::core::ffi::c_int as MenuKind8;
        mn_804A04F0.hovered_selection = SEL_MAIN_DATA as u16_0;
        HSD_GObj_80390CD4(mn_8022B3A0(3 as u8_0) as *mut HSD_GObj);
        HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
        temp_r29_2 = mn_803EB6B0[MENU_KIND_MAIN as ::core::ffi::c_int as usize].think
            as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
        if temp_r29_2.is_some() {
            think2 = HSD_GObj_SetupProc(
                GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                temp_r29_2 as HSD_GObjEvent,
                0 as u8_0,
            );
            (*think2).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        }
    } else if buttons & MenuInput_Up as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as ::core::ffi::c_int
                != SEL_DATA_SNAP as ::core::ffi::c_int
            {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_sub(1);
            } else {
                mn_804A04F0.hovered_selection = SEL_DATA_SPECIAL as u16_0;
            }
            if !(mn_80229938(MENU_KIND_DATA, mn_804A04F0.hovered_selection as s32)
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    } else if buttons & MenuInput_Down as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as DataMenuSelection as ::core::ffi::c_int
                == SEL_DATA_SPECIAL as ::core::ffi::c_int
            {
                mn_804A04F0.hovered_selection = SEL_DATA_SNAP as u16_0;
            } else {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_add(1);
            }
            if !(mn_80229938(MENU_KIND_DATA, mn_804A04F0.hovered_selection as s32)
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    }
}
pub unsafe extern "C" fn mn_8022D104(mut gp: *mut HSD_GObj) {
    melee_core::ct!("mn_8022D104");
    let mut selection_count: ::core::ffi::c_int = 0;
    let mut buttons: u32_0 = 0;
    let mut temp_r3_2: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut temp_r28: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    selection_count = mn_803EB6B0[MENU_KIND_SETTINGS as ::core::ffi::c_int as usize]
        .selection_count as ::core::ffi::c_int & 0xff as ::core::ffi::c_int;
    buttons = mn_80229624(4 as u32_0);
    mn_804A04F0.buttons = buttons as u64_0;
    if buttons & MenuInput_Confirm as ::core::ffi::c_int as u32_0 != 0 {
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.entering_menu = 1 as u8_0;
        match mn_804A04F0.hovered_selection as ::core::ffi::c_int {
            0 => {
                sfxForward();
                mnVibration_Init(1 as ::core::ffi::c_int);
                HSD_GObjFree(gp);
            }
            1 => {
                sfxForward();
                mnSound_8024A09C(1 as ::core::ffi::c_int);
                HSD_GObjFree(gp);
            }
            2 => {
                sfxForward();
                mnDeflicker_8024A6C4(1 as ::core::ffi::c_int);
                HSD_GObjFree(gp);
            }
            4 => {
                sfxForward();
                mnLanguage_8024C5C0(1 as ::core::ffi::c_int as *mut HSD_GObj);
                HSD_GObjFree(gp);
            }
            5 => {
                sfxForward();
                mnDataDel_80250170();
                HSD_GObjFree(gp);
            }
            _ => {}
        }
    } else if buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
        mn_804A04F0.cur_menu = MENU_KIND_MAIN as ::core::ffi::c_int as MenuKind8;
        mn_804A04F0.hovered_selection = SEL_MAIN_SETTINGS as u16_0;
        HSD_GObj_80390CD4(mn_8022B3A0(3 as u8_0) as *mut HSD_GObj);
        HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
        temp_r28 = mn_803EB6B0[MENU_KIND_MAIN as ::core::ffi::c_int as usize].think
            as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
        if temp_r28.is_some() {
            temp_r3_2 = HSD_GObj_SetupProc(
                GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                temp_r28 as HSD_GObjEvent,
                0 as u8_0,
            );
            (*temp_r3_2).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        }
    } else if buttons & MenuInput_Up as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as ::core::ffi::c_int
                != SEL_SETTINGS_RUMBLE as ::core::ffi::c_int
            {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_sub(1);
            } else {
                mn_804A04F0.hovered_selection = (selection_count as u8_0
                    as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u16_0;
            }
            if !(mn_80229938(MENU_KIND_SETTINGS, mn_804A04F0.hovered_selection as s32)
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    } else if buttons & 2 as u32_0 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as ::core::ffi::c_int
                == selection_count as u8_0 as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
            {
                mn_804A04F0.hovered_selection = SEL_SETTINGS_RUMBLE as ::core::ffi::c_int
                    as u16_0;
            } else {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_add(1);
            }
            if !(mn_80229938(MENU_KIND_SETTINGS, mn_804A04F0.hovered_selection as s32)
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    }
}
pub unsafe extern "C" fn mn_8022D34C(mut gp: *mut HSD_GObj) {
    melee_core::ct!("mn_8022D34C");
    let mut think: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut buttons: u32_0 = 0;
    let mut var_r3: u8_0 = 0;
    let mut data: *mut MenuExitData = ::core::ptr::null_mut::<MenuExitData>();
    let mut temp_r28: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    buttons = mn_80229624(4 as u32_0);
    mn_804A04F0.buttons = buttons as u64_0;
    if buttons & MenuInput_Confirm as ::core::ffi::c_int as u32_0 != 0 {
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.entering_menu = 1 as u8_0;
        gm_801677E8(mn_8022C7CC_inline() as s8);
        match mn_804A04F0.hovered_selection as ::core::ffi::c_int {
            0 => {
                if crate::menuguard::deny(GM_TOY_GALLERY as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_TOY_GALLERY as ::core::ffi::c_int as s8;
                gm_801A4B60();
                return;
            }
            1 => {
                if crate::menuguard::deny(GM_TOY_LOTTERY as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_TOY_LOTTERY as ::core::ffi::c_int as s8;
                gm_801A4B60();
                return;
            }
            3 => {
                if crate::menuguard::deny(GM_TOY_COLLECTION as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_TOY_COLLECTION as ::core::ffi::c_int as s8;
                gm_801A4B60();
                return;
            }
            _ => {}
        }
    } else if buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
        mn_804A04F0.cur_menu = MENU_KIND_MAIN as ::core::ffi::c_int as MenuKind8;
        mn_804A04F0.hovered_selection = SEL_MAIN_TOY as u16_0;
        HSD_GObj_80390CD4(mn_8022B3A0(3 as u8_0) as *mut HSD_GObj);
        HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
        temp_r28 = mn_803EB6B0[MENU_KIND_MAIN as ::core::ffi::c_int as usize].think
            as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
        if temp_r28.is_some() {
            think = HSD_GObj_SetupProc(
                GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                temp_r28 as HSD_GObjEvent,
                0 as u8_0,
            );
            (*think).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        }
    } else if buttons & MenuInput_Up as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as ::core::ffi::c_int
                != SEL_TOY_GALLERY as ::core::ffi::c_int
            {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_sub(1);
            } else {
                mn_804A04F0.hovered_selection = SEL_TOY_COLLECTION as u16_0;
            }
            if !(mn_80229938(MENU_KIND_TOY, mn_804A04F0.hovered_selection as s32)
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    } else if buttons & MenuInput_Down as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as TrophyMenuSelection as ::core::ffi::c_int
                == SEL_TOY_COLLECTION as ::core::ffi::c_int
            {
                mn_804A04F0.hovered_selection = SEL_TOY_GALLERY as ::core::ffi::c_int
                    as u16_0;
            } else {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_add(1);
            }
            if !(mn_80229938(MENU_KIND_TOY, mn_804A04F0.hovered_selection as s32)
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    }
}
pub unsafe extern "C" fn mn_8022D594(mut gp: *mut HSD_GObj) {
    melee_core::ct!("mn_8022D594");
    let mut think: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut think2: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut buttons: u32_0 = 0;
    let mut temp_r28: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut temp_r28_2: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut data: *mut MenuExitData = ::core::ptr::null_mut::<MenuExitData>();
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    buttons = mn_80229624(4 as u32_0);
    mn_804A04F0.buttons = buttons as u64_0;
    if buttons & MenuInput_Confirm as ::core::ffi::c_int as u32_0 != 0 {
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.entering_menu = 1 as u8_0;
        match mn_804A04F0.hovered_selection as ::core::ffi::c_int {
            0 => {
                if crate::menuguard::deny(GM_VS as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_VS as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            1 => {
                if crate::menuguard::deny(GM_TOURNAMENT as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_TOURNAMENT as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            2 => {
                sfxForward();
                mn_804D6BC8.cooldown = 5 as u16_0;
                mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
                mn_804A04F0.cur_menu = MENU_KIND_SPECIAL as ::core::ffi::c_int
                    as MenuKind8;
                mn_804A04F0.hovered_selection = SEL_SPECIAL_VS_CAMERA as u16_0;
                HSD_GObj_80390CD4(mn_8022B3A0(1 as u8_0) as *mut HSD_GObj);
                HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
                temp_r28 = mn_803EB6B0[MENU_KIND_SPECIAL as ::core::ffi::c_int as usize]
                    .think as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
                if temp_r28.is_some() {
                    think = HSD_GObj_SetupProc(
                        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                        temp_r28 as HSD_GObjEvent,
                        0 as u8_0,
                    );
                    (*think).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
                }
            }
            3 => {
                sfxForward();
                mn_80231714();
                HSD_GObjFree(gp);
            }
            4 => {
                sfxForward();
                mnName_8023AC40();
                HSD_GObjFree(gp);
            }
            _ => {}
        }
    } else if buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
        mn_804A04F0.cur_menu = MENU_KIND_MAIN as ::core::ffi::c_int as MenuKind8;
        mn_804A04F0.hovered_selection = SEL_MAIN_VS as u16_0;
        HSD_GObj_80390CD4(mn_8022B3A0(3 as u8_0) as *mut HSD_GObj);
        HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
        temp_r28_2 = mn_803EB6B0[MENU_KIND_MAIN as ::core::ffi::c_int as usize].think
            as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
        if temp_r28_2.is_some() {
            think2 = HSD_GObj_SetupProc(
                GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                temp_r28_2 as HSD_GObjEvent,
                0 as u8_0,
            );
            (*think2).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        }
    } else if buttons & MenuInput_Up as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        if mn_804A04F0.hovered_selection as VsMenuSelection as ::core::ffi::c_int
            == SEL_VS_MELEE as ::core::ffi::c_int
        {
            mn_804A04F0.hovered_selection = SEL_VS_NAME as ::core::ffi::c_int as u16_0;
        } else {
            mn_804A04F0.hovered_selection = mn_804A04F0
                .hovered_selection
                .wrapping_sub(1);
        }
    } else if buttons & MenuInput_Down as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        if mn_804A04F0.hovered_selection as VsMenuSelection as ::core::ffi::c_int
            == SEL_VS_NAME as ::core::ffi::c_int
        {
            mn_804A04F0.hovered_selection = SEL_VS_MELEE as ::core::ffi::c_int as u16_0;
        } else {
            mn_804A04F0.hovered_selection = mn_804A04F0
                .hovered_selection
                .wrapping_add(1);
        }
    }
}
pub unsafe extern "C" fn mn_8022D7F4(mut gp: *mut HSD_GObj) {
    melee_core::ct!("mn_8022D7F4");
    let mut selection_count: ::core::ffi::c_int = 0;
    let mut temp_r3_2: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut temp_r3_3: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut temp_r3_4: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut buttons: u32_0 = 0;
    let mut temp_r27: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut data: *mut MenuExitData = ::core::ptr::null_mut::<MenuExitData>();
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    selection_count = mn_803EB6B0[MENU_KIND_1P as ::core::ffi::c_int as usize]
        .selection_count as ::core::ffi::c_int & 0xff as ::core::ffi::c_int;
    buttons = mn_80229624(4 as u32_0);
    mn_804A04F0.buttons = buttons as u64_0;
    if buttons & MenuInput_Confirm as ::core::ffi::c_int as u32_0 != 0 {
        mn_804A04F0.entering_menu = 1 as u8_0;
        gm_801677E8(mn_8022C7CC_inline() as s8);
        match mn_804A04F0.hovered_selection as ::core::ffi::c_int {
            0 => {
                sfxForward();
                mn_804D6BC8.cooldown = 5 as u16_0;
                mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
                mn_804A04F0.cur_menu = MENU_KIND_REG as ::core::ffi::c_int as MenuKind8;
                mn_804A04F0.hovered_selection = SEL_REG_CLASSIC as u16_0;
                HSD_GObj_80390CD4(mn_8022B3A0(1 as u8_0) as *mut HSD_GObj);
                HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
                temp_r27 = mn_803EB6B0[MENU_KIND_REG as ::core::ffi::c_int as usize]
                    .think as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
                if temp_r27.is_some() {
                    temp_r3_2 = HSD_GObj_SetupProc(
                        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                        temp_r27 as HSD_GObjEvent,
                        0 as u8_0,
                    );
                    (*temp_r3_2).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
                }
            }
            3 => {
                sfxForward();
                mn_804D6BC8.cooldown = 5 as u16_0;
                mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
                mn_804A04F0.cur_menu = MENU_KIND_STADIUM as ::core::ffi::c_int
                    as MenuKind8;
                mn_804A04F0.hovered_selection = SEL_STADIUM_TARGET as u16_0;
                HSD_GObj_80390CD4(mn_8022B3A0(1 as u8_0) as *mut HSD_GObj);
                HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
                temp_r27 = mn_803EB6B0[MENU_KIND_STADIUM as ::core::ffi::c_int as usize]
                    .think as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
                if temp_r27.is_some() {
                    temp_r3_3 = HSD_GObj_SetupProc(
                        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                        temp_r27 as HSD_GObjEvent,
                        0 as u8_0,
                    );
                    (*temp_r3_3).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
                }
            }
            1 => {
                if crate::menuguard::deny(GM_EVENT as i32) { return; } // menuguard: not ported yet
                sfxForward();
                mnEvent_8024E838(0 as ::core::ffi::c_int, true_0);
                HSD_GObjFree(gp);
            }
            4 => {
                if crate::menuguard::deny(GM_TRAINING as i32) { return; } // menuguard: not ported yet
                sfxForward();
                data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
                (*data).pending_mode = GM_TRAINING as ::core::ffi::c_int as s8;
                gm_801A4B60();
            }
            _ => {}
        }
    } else if buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
        mn_804A04F0.cur_menu = MENU_KIND_MAIN as ::core::ffi::c_int as MenuKind8;
        mn_804A04F0.hovered_selection = SEL_MAIN_1P as u16_0;
        HSD_GObj_80390CD4(mn_8022B3A0(3 as u8_0) as *mut HSD_GObj);
        HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
        temp_r27 = mn_803EB6B0[MENU_KIND_MAIN as ::core::ffi::c_int as usize].think
            as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
        if temp_r27.is_some() {
            temp_r3_4 = HSD_GObj_SetupProc(
                GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                temp_r27 as HSD_GObjEvent,
                0 as u8_0,
            );
            (*temp_r3_4).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        }
    } else if buttons & MenuInput_Up as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as ::core::ffi::c_int
                != SEL_1P_REG as ::core::ffi::c_int
            {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_sub(1);
            } else {
                mn_804A04F0.hovered_selection = (selection_count as u8_0
                    as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u16_0;
            }
            if !(mn_80229938(MENU_KIND_1P, mn_804A04F0.hovered_selection as s32)
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    } else if buttons & MenuInput_Down as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as ::core::ffi::c_int
                == selection_count as u8_0 as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
            {
                mn_804A04F0.hovered_selection = SEL_1P_REG as u16_0;
            } else {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_add(1);
            }
            if !(mn_80229938(MENU_KIND_1P, mn_804A04F0.hovered_selection as s32)
                == 0 as ::core::ffi::c_int)
            {
                break;
            }
        }
    }
}
pub unsafe extern "C" fn mn_8022DB10(mut gp: *mut HSD_GObj) {
    melee_core::ct!("mn_8022DB10");
    let mut data: *mut MenuExitData = ::core::ptr::null_mut::<MenuExitData>();
    let mut menu_kind: MenuKind = MENU_KIND_MAIN;
    let mut temp_r28: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut selection_count: ::core::ffi::c_int = 0;
    let mut hovered_selection: u16_0 = 0;
    let mut temp_r3_2: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut mf: *mut MenuFlow = &raw mut mn_804A04F0;
    let mut prev_menu_ptr: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut selection_ptr: *mut u16_0 = ::core::ptr::null_mut::<u16_0>();
    let mut buttons: u32_0 = 0;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    selection_count = mn_803EB6B0[MENU_KIND_MAIN as ::core::ffi::c_int as usize]
        .selection_count as ::core::ffi::c_int & 0xff as ::core::ffi::c_int;
    buttons = mn_80229624(4 as u32_0);
    (*mf).buttons = buttons as u64_0;
    // port: the Envoy row (envoy_main6): the display order 1-P, Vs., Envoy, Trophies, Options, Data
    if envoy_main6(MENU_KIND_MAIN as ::core::ffi::c_int) && buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 == 0 {
        let pos = ENVOY_ORDER.iter().position(|&x| x == (*mf).hovered_selection as ::core::ffi::c_int).unwrap_or(0);
        if buttons & MenuInput_Confirm as ::core::ffi::c_int as u32_0 != 0 {
            if (*mf).hovered_selection as ::core::ffi::c_int == ENVOY_SEL {
                mn_804D6BC8.cooldown = 5 as u16_0;
                if crate::tu::envoy::door_main_open() {
                    sfxForward();
                } else {
                    sfxBack();
                }
                return;
            }
        } else if buttons & MenuInput_Up as ::core::ffi::c_int as u32_0 != 0 {
            sfxMove();
            (*mf).hovered_selection = ENVOY_ORDER[(pos + 5) % 6] as u16_0;
            return;
        } else if buttons & MenuInput_Down as ::core::ffi::c_int as u32_0 != 0 {
            sfxMove();
            (*mf).hovered_selection = ENVOY_ORDER[(pos + 1) % 6] as u16_0;
            return;
        }
    }
    if buttons & MenuInput_Confirm as ::core::ffi::c_int as u32_0 != 0 {
        mn_804D6BC8.cooldown = 5 as u16_0;
        (*mf).entering_menu = 1 as u8_0;
        sfxForward();
        prev_menu_ptr = &raw mut (*mf).prev_menu as *mut u8_0;
        selection_ptr = &raw mut (*mf).hovered_selection;
        *prev_menu_ptr = (*mf).cur_menu as u8_0;
        match (*mf).hovered_selection as ::core::ffi::c_int {
            0 => {
                menu_kind = MENU_KIND_1P;
                hovered_selection = 0 as u16_0;
            }
            1 => {
                menu_kind = MENU_KIND_VS;
                hovered_selection = 0 as u16_0;
            }
            2 => {
                menu_kind = MENU_KIND_TOY;
                hovered_selection = 0 as u16_0;
            }
            3 => {
                menu_kind = MENU_KIND_SETTINGS;
                hovered_selection = 0 as u16_0;
            }
            4 => {
                menu_kind = MENU_KIND_DATA;
                hovered_selection = 0 as u16_0;
            }
            _ => {}
        }
        mn_804D6BC8.cooldown = 5 as u16_0;
        *prev_menu_ptr = (*mf).cur_menu as u8_0;
        (*mf).cur_menu = menu_kind as MenuKind8;
        *selection_ptr = hovered_selection;
        HSD_GObj_80390CD4(mn_8022B3A0(1 as u8_0) as *mut HSD_GObj);
        HSD_GObjFree(HSD_GObj_CurrentInvokedProcGObj);
        temp_r28 = mn_803EB6B0[menu_kind as u64_0 as usize].think
            as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
        if temp_r28.is_some() {
            temp_r3_2 = HSD_GObj_SetupProc(
                GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
                temp_r28 as HSD_GObjEvent,
                0 as u8_0,
            );
            (*temp_r3_2).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        }
    } else if buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 != 0 {
        sfxBack();
        (*mf).entering_menu = 0 as u8_0;
        mn_804D6BC8.cooldown = 5 as u16_0;
        data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
        (*data).pending_mode = GM_TITLE as ::core::ffi::c_int as s8;
        gm_801A4B60();
    } else if buttons & MenuInput_Up as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        decrement_selection(
            selection_count as u8_0,
            MENU_KIND_MAIN as ::core::ffi::c_int as s32,
        );
    } else if buttons & MenuInput_Down as ::core::ffi::c_int as u32_0 != 0 {
        sfxMove();
        increment_selection(selection_count as u8_0, MENU_KIND_MAIN);
    }
}
pub unsafe extern "C" fn mnMain_Scene_OnFrame() {
    melee_core::ct!("mnMain_Scene_OnFrame");
    let mut data: *mut MenuExitData = ::core::ptr::null_mut::<MenuExitData>();
    // port: the Envoy camp started the run (melee_modes::tu::envoy::LAUNCH_VS): the menu's own VS pick
    if crate::tu::envoy::BACK_TO_CAMP == 2 {
        crate::tu::envoy::BACK_TO_CAMP = 0;
        crate::tu::envoy::DOOR_OPEN = 1;
    }
    if crate::tu::envoy::LAUNCH_VS != 0 {
        crate::tu::envoy::LAUNCH_VS = 0;
        crate::tu::envoy::DOOR_HOVER = 0;
        if !crate::menuguard::deny(GM_VS as i32) {
            data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
            (*data).pending_mode = GM_VS as ::core::ffi::c_int as s8;
            gm_801A4B60();
            return;
        }
    }
    if mn_8022F218() != 0
        && mn_804A04F0.cur_menu as ::core::ffi::c_int
            != MENU_KIND_MAIN as ::core::ffi::c_int
    {
        lbAudioAx_80023694();
        sfxBack();
        mn_8022F268();
        gm_801603B0();
        lbCardNew_CompleteAllTasks(LbCardResult_Busy as ::core::ffi::c_int);
        lbMthp_8001F800();
        mn_8022EBDC();
        HSD_SisLib_803A5E70();
        data = gm_GetCurrentSceneExitData() as *mut MenuExitData;
        (*data).pending_mode = GM_MENU as ::core::ffi::c_int as s8;
        gm_801A4B60();
    }
}
pub unsafe extern "C" fn mn_PcOpenNative(mut kind: u8_0, mut sel: u8_0) {
    melee_core::ct!("mn_PcOpenNative");
    mn_804D6BC8.cooldown = 5 as u16_0;
    mn_804A04F0.entering_menu = 1 as u8_0;
    mn_804A04F0.cur_menu = kind as MenuKind8;
    mn_804A04F0.hovered_selection = sel as u16_0;
    OSReport(b"mnmain: frontend opens native screen kind %d selection %d\n\0" as *const u8
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char, &[(kind as ::core::ffi::c_int) as usize, (sel as ::core::ffi::c_int) as usize]);
    match kind as ::core::ffi::c_int {
        1 => {
            if sel as ::core::ffi::c_int == SEL_1P_EVENT as ::core::ffi::c_int {
                mnEvent_8024E838(0 as ::core::ffi::c_int, true_0);
                return;
            }
        }
        2 => {
            if sel as ::core::ffi::c_int == SEL_VS_RULES as ::core::ffi::c_int {
                mn_80231714();
                return;
            }
            if sel as ::core::ffi::c_int == SEL_VS_NAME as ::core::ffi::c_int {
                mnName_8023AC40();
                return;
            }
        }
        4 => {
            match sel as ::core::ffi::c_int {
                0 => {
                    mnVibration_Init(1 as ::core::ffi::c_int);
                    return;
                }
                1 => {
                    mnSound_8024A09C(1 as ::core::ffi::c_int);
                    return;
                }
                2 => {
                    mnDeflicker_8024A6C4(1 as ::core::ffi::c_int);
                    return;
                }
                4 => {
                    mnLanguage_8024C5C0(1 as ::core::ffi::c_int as *mut HSD_GObj);
                    return;
                }
                5 => {
                    mnDataDel_80250170();
                    return;
                }
                _ => {}
            }
        }
        5 => {
            match sel as ::core::ffi::c_int {
                0 => {
                    mnSnap_80257F24();
                    return;
                }
                1 => {
                    mnGallery_80259868();
                    return;
                }
                2 => {
                    lbAudioAx_80023694();
                    lbAudioAx_800236DC();
                    mnSoundTest_8024BEE0(1 as s32);
                    return;
                }
                4 => {
                    mnInfo_80252758();
                    return;
                }
                _ => {}
            }
        }
        28 => {
            match sel as ::core::ffi::c_int {
                0 => {
                    mnDiagram_Init(1 as u8_0, 1 as u8_0);
                    return;
                }
                1 => {
                    mnInfoBonus_80252F8C();
                    return;
                }
                2 => {
                    mnCount_Create();
                    return;
                }
                _ => {}
            }
        }
        _ => {}
    }
    OSReport(b"mnmain: no native screen for kind %d selection %d - opening the menu\n\0"
            as *const u8 as *const ::core::ffi::c_char as *mut ::core::ffi::c_char, &[(kind as ::core::ffi::c_int) as usize, (sel as ::core::ffi::c_int) as usize]);
    HSD_GObj_SetupProc(
        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
        if mn_803EB6B0[kind as usize].think.is_some() {
            mn_803EB6B0[kind as usize].think as HSD_GObjEvent
        } else {
            Some(mn_8022DB10 as unsafe extern "C" fn(*mut HSD_GObj) -> ())
        },
        0 as u8_0,
    );
    mn_8022B3A0(0 as u8_0);
}
#[inline]
pub unsafe extern "C" fn mn_8022DDA8_inline(mut sp2B4: *const u16_0) {
    melee_core::ct!("mn_8022DDA8_inline");
    let mut temp_r29: ::core::ffi::c_int = 0;
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut lobj: *mut HSD_LObj = ::core::ptr::null_mut::<HSD_LObj>();
    gobj = GObj_Create(3 as u16_0, 4 as u8_0, 0x80 as u8_0);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_LObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, isize) -> (),
        ),
        0 as u8_0,
        0x80 as u32_0,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(fn_8022C128 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    lobj = lb_80011AC4(MenMain_lights);
    HSD_GObjObject_80390A70(
        gobj,
        HSD_GObj_LightKind as u8_0,
        lobj as *mut ::core::ffi::c_void,
    );
    mn_804A04F0.light_lerp_frames = 0 as u8_0;
    temp_r29 = mn_8022C010(
        mn_804A04F0.cur_menu as ::core::ffi::c_int,
        *sp2B4 as ::core::ffi::c_int,
    );
    mn_804A04F0.light_color = mn_8022BFBC(temp_r29);
    mn_8022C068(lobj, temp_r29, mn_804A04F0.light_lerp_frames as ::core::ffi::c_int);
}
pub unsafe extern "C" fn mnMain_Scene_OnEnter(mut user_data: *mut ::core::ffi::c_void) {
    melee_core::ct!("mnMain_Scene_OnEnter");
    let mut pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut hovered_selection: *mut u16_0 = ::core::ptr::null_mut::<u16_0>();
    let mut temp_r3_8: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut menu_kind: u8_0 = 0;
    let mut var_r4: Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()> = None;
    let mut data: *mut MenuEnterData = user_data as *mut MenuEnterData;
    let mut pc_native: bool_0 = false_0;
    let mut pc_native_kind: u8_0 = 0 as u8_0;
    let mut pc_native_sel: u8_0 = 0 as u8_0;
    let mut _pad: [u8_0; 20] = [0; 20];
    crate::tu::envoy::DOOR_HOVER = 0; // port: the Envoy row starts unhovered
    mn_804D6BC8.cooldown = 0x14 as u16_0;
    mn_804D6BC8.x2 = 0 as u16_0;
    mn_804D6BC8.x4 = 0 as ::core::ffi::c_int as s32;
    mn_804A04F0.buttons = 0 as u64_0;
    hovered_selection = &raw mut mn_804A04F0.hovered_selection;
    menu_kind = (*data).menu_kind;
    mn_804A04F0.cur_menu = menu_kind as MenuKind8;
    mn_804A04F0.prev_menu = menu_kind as MenuKind8;
    *hovered_selection = (*data).hovered_selection as u16_0;
    mn_804D6BAC = ::core::ptr::null_mut::<HSD_GObj>();
    mn_804D6BB0 = ::core::ptr::null_mut::<HSD_GObj>();
    if (*data).load_assets as ::core::ffi::c_int != false_0 {
        mn_804D6BB8 = lbArchive_LoadSymbols(b"MnMaAll\0" as *const u8 as *const ::core::ffi::c_char, &raw mut MenMainBack_Top.joint as *mut ::core::ffi::c_void, &[(b"MenMainBack_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainBack_Top.animjoint) as usize, (b"MenMainBack_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainBack_Top.matanim_joint) as usize, (b"MenMainBack_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainBack_Top.shapeanim_joint) as usize, (b"MenMainBack_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMain_cam) as usize, (b"ScMenMain_cam_int1_camera\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMain_lights) as usize, (b"ScMenMain_scene_lights\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMain_fog) as usize, (b"ScMenMain_fog\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainPanel_Top.joint) as usize, (b"MenMainPanel_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainPanel_Top.animjoint) as usize, (b"MenMainPanel_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainPanel_Top.matanim_joint) as usize, (b"MenMainPanel_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainPanel_Top.shapeanim_joint) as usize, (b"MenMainPanel_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConTop_Top.joint) as usize, (b"MenMainConTop_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConTop_Top.animjoint) as usize, (b"MenMainConTop_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConTop_Top.matanim_joint) as usize, (b"MenMainConTop_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConTop_Top.shapeanim_joint) as usize, (b"MenMainConTop_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursor_Top.joint) as usize, (b"MenMainCursor_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursor_Top.animjoint) as usize, (b"MenMainCursor_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursor_Top.matanim_joint) as usize, (b"MenMainCursor_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursor_Top.shapeanim_joint) as usize, (b"MenMainCursor_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConRl_Top.joint) as usize, (b"MenMainConRl_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConRl_Top.animjoint) as usize, (b"MenMainConRl_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConRl_Top.matanim_joint) as usize, (b"MenMainConRl_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConRl_Top.shapeanim_joint) as usize, (b"MenMainConRl_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl_Top.joint) as usize, (b"MenMainCursorRl_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl_Top.animjoint) as usize, (b"MenMainCursorRl_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl_Top.matanim_joint) as usize, (b"MenMainCursorRl_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl_Top.shapeanim_joint) as usize, (b"MenMainCursorRl_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainNmRl_Top.joint) as usize, (b"MenMainNmRl_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainNmRl_Top.animjoint) as usize, (b"MenMainNmRl_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainNmRl_Top.matanim_joint) as usize, (b"MenMainNmRl_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainNmRl_Top.shapeanim_joint) as usize, (b"MenMainNmRl_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr01_Top.joint) as usize, (b"MenMainCursorTr01_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr01_Top.animjoint) as usize, (b"MenMainCursorTr01_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr01_Top.matanim_joint) as usize, (b"MenMainCursorTr01_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr01_Top.shapeanim_joint) as usize, (b"MenMainCursorTr01_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr02_Top.joint) as usize, (b"MenMainCursorTr02_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr02_Top.animjoint) as usize, (b"MenMainCursorTr02_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr02_Top.matanim_joint) as usize, (b"MenMainCursorTr02_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr02_Top.shapeanim_joint) as usize, (b"MenMainCursorTr02_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr03_Top.joint) as usize, (b"MenMainCursorTr03_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr03_Top.animjoint) as usize, (b"MenMainCursorTr03_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr03_Top.matanim_joint) as usize, (b"MenMainCursorTr03_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr03_Top.shapeanim_joint) as usize, (b"MenMainCursorTr03_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr04_Top.joint) as usize, (b"MenMainCursorTr04_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr04_Top.animjoint) as usize, (b"MenMainCursorTr04_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr04_Top.matanim_joint) as usize, (b"MenMainCursorTr04_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr04_Top.shapeanim_joint) as usize, (b"MenMainCursorTr04_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl01_Top.joint) as usize, (b"MenMainCursorRl01_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl01_Top.animjoint) as usize, (b"MenMainCursorRl01_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl01_Top.matanim_joint) as usize, (b"MenMainCursorRl01_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl01_Top.shapeanim_joint) as usize, (b"MenMainCursorRl01_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl02_Top.joint) as usize, (b"MenMainCursorRl02_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl02_Top.animjoint) as usize, (b"MenMainCursorRl02_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl02_Top.matanim_joint) as usize, (b"MenMainCursorRl02_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl02_Top.shapeanim_joint) as usize, (b"MenMainCursorRl02_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl03_Top.joint) as usize, (b"MenMainCursorRl03_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl03_Top.animjoint) as usize, (b"MenMainCursorRl03_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl03_Top.matanim_joint) as usize, (b"MenMainCursorRl03_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl03_Top.shapeanim_joint) as usize, (b"MenMainCursorRl03_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl04_Top.joint) as usize, (b"MenMainCursorRl04_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl04_Top.animjoint) as usize, (b"MenMainCursorRl04_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl04_Top.matanim_joint) as usize, (b"MenMainCursorRl04_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl04_Top.shapeanim_joint) as usize, (b"MenMainCursorRl04_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl05_Top.joint) as usize, (b"MenMainCursorRl05_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl05_Top.animjoint) as usize, (b"MenMainCursorRl05_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl05_Top.matanim_joint) as usize, (b"MenMainCursorRl05_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl05_Top.shapeanim_joint) as usize, (b"MenMainCursorRl05_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConIs_Top.joint) as usize, (b"MenMainConIs_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConIs_Top.animjoint) as usize, (b"MenMainConIs_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConIs_Top.matanim_joint) as usize, (b"MenMainConIs_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConIs_Top.shapeanim_joint) as usize, (b"MenMainConIs_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorIs_Top.joint) as usize, (b"MenMainCursorIs_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorIs_Top.animjoint) as usize, (b"MenMainCursorIs_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorIs_Top.matanim_joint) as usize, (b"MenMainCursorIs_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorIs_Top.shapeanim_joint) as usize, (b"MenMainCursorIs_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConSs_Top.joint) as usize, (b"MenMainConSs_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConSs_Top.animjoint) as usize, (b"MenMainConSs_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConSs_Top.matanim_joint) as usize, (b"MenMainConSs_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConSs_Top.shapeanim_joint) as usize, (b"MenMainConSs_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorSs_Top.joint) as usize, (b"MenMainCursorSs_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorSs_Top.animjoint) as usize, (b"MenMainCursorSs_Top_animjoint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorSs_Top.matanim_joint) as usize, (b"MenMainCursorSs_Top_matanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorSs_Top.shapeanim_joint) as usize, (b"MenMainCursorSs_Top_shapeanim_joint\0" as *const u8
                as *const ::core::ffi::c_char) as usize, (0 as ::core::ffi::c_int) as usize]);
        if lbLang_IsSavedLanguageUS() != 0 {
            HSD_SisLib_803A62A0(
                0 as s32,
                b"SdMenu.usd\0" as *const u8 as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                b"SIS_MenuData\0" as *const u8 as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            );
        } else {
            HSD_SisLib_803A62A0(
                0 as s32,
                b"SdMenu.dat\0" as *const u8 as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                b"SIS_MenuData\0" as *const u8 as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            );
        }
        HSD_SisLib_803A62A0(
            3 as s32,
            b"SdToy.dat\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            b"SIS_ToyData\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        );
        gm_801BA8FC();
        lbAudioAx_8002392C();
    }
    mn_8022DDA8_inline(hovered_selection);
    mn_8022BCF8();
    mn_8022BEDC(mn_8022BE34_OnEnter(&raw mut pos));
    mn_80229B2C();
    mn_80229DC0();
    match (*data).menu_kind as ::core::ffi::c_int {
        7 => {
            mnEvent_8024E838(gm_801BEB80() as ::core::ffi::c_int, false_0);
        }
        33 => {
            mnHyaku_8024CD64((*data).hovered_selection);
        }
        _ => {
            if gmFrontend_NativeRequest(&raw mut pc_native_kind, &raw mut pc_native_sel)
                != 0
            {
                pc_native = true_0 as bool_0;
            } else {
                temp_r3_8 = GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0);
                var_r4 = mn_803EB6B0[mn_804A04F0.cur_menu as usize].think
                    as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
                if var_r4.is_none() {
                    var_r4 = Some(
                        mn_8022DB10 as unsafe extern "C" fn(*mut HSD_GObj) -> (),
                    ) as Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>;
                }
                HSD_GObj_SetupProc(temp_r3_8, var_r4 as HSD_GObjEvent, 0 as u8_0);
                mn_8022B3A0(0 as u8_0);
            }
        }
    }
    lbAudioAx_80023F28(gmMainLib_8015ECB0());
    lbCardGame_SaveChanges();
    if pc_native != 0
        && gmFrontend_TakeNativeRequest(&raw mut pc_native_kind, &raw mut pc_native_sel)
            != 0
    {
        mn_PcOpenNative(pc_native_kind, pc_native_sel);
    }
}
pub static mut null_terminator: [::core::ffi::c_char; 1] = unsafe {
    ::core::mem::transmute::<[u8; 1], [::core::ffi::c_char; 1]>(*b"\0")
};
pub unsafe extern "C" fn mn_IsFighterUnlocked(
    mut selkind: SelectableCharacterKind,
) -> bool_0 {
    melee_core::ct!("mn_IsFighterUnlocked");
    return gm_IsCKindUnlocked(gm_SelKindToCKind(selkind as u8_0));
}
pub unsafe extern "C" fn mn_8022E978(mut item_idx: u8_0, mut enable: u8_0) {
    melee_core::ct!("mn_8022E978");
    if enable != 0 {
        (*gmMainLib_GetGamePrefs()).item_mask
            |= (<i32 as melee_core::ppc::Ppc>::pshl(1 as ::core::ffi::c_int, (item_idx as ::core::ffi::c_int) as i64)) as u64_0;
    } else {
        (*gmMainLib_GetGamePrefs()).item_mask
            &= !(<i32 as melee_core::ppc::Ppc>::pshl(1 as ::core::ffi::c_int, (item_idx as ::core::ffi::c_int) as i64)) as u64_0;
    };
}
pub unsafe extern "C" fn mn_8022EA08(mut buf: *mut ::core::ffi::c_char, mut num: u32_0) {
    melee_core::ct!("mn_8022EA08");
    let mut buf_end: ::core::ffi::c_int = mn_GetDigitCount(num as s32)
        as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < buf_end {
        *buf.offset((buf_end - 1 as ::core::ffi::c_int - i) as isize) = (mn_GetDigitAt(
            num as s32,
            i as s32,
        ) as ::core::ffi::c_int + '0' as i32) as ::core::ffi::c_char;
        i += 1;
    }
    *buf.offset(buf_end as isize) = null_terminator[0 as ::core::ffi::c_int as usize];
}
pub unsafe extern "C" fn mn_8022EA78(
    mut buf: *mut ::core::ffi::c_char,
    mut buf_end: ::core::ffi::c_int,
    mut num: u32_0,
) {
    melee_core::ct!("mn_8022EA78");
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < buf_end {
        *buf.offset((buf_end - 1 as ::core::ffi::c_int - i) as isize) = (mn_GetDigitAt(
            num as s32,
            i as s32,
        ) as ::core::ffi::c_int + '0' as i32) as ::core::ffi::c_char;
        i += 1;
    }
    *buf.offset(buf_end as isize) = null_terminator[0 as ::core::ffi::c_int as usize];
}
pub unsafe extern "C" fn mn_8022EAE0(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mn_8022EAE0");
    HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
}
pub unsafe extern "C" fn mn_8022EB04(mut user_data: *mut ::core::ffi::c_void) {
    melee_core::ct!("mn_8022EB04");
    HSD_Free(user_data);
}
pub unsafe extern "C" fn mn_GetDigitAt(mut arg0: s32, mut arg1: s32) -> s32 {
    melee_core::ct!("mn_GetDigitAt");
    return <i32 as melee_core::ppc::Ppc>::pdiv(arg0, powi(10 as s32, arg1)) % 10 as s32;
}
pub unsafe extern "C" fn mn_GetDigitCount(mut num: s32) -> s32 {
    melee_core::ct!("mn_GetDigitCount");
    let mut count: s32 = 0;
    if num == 0 as ::core::ffi::c_int {
        return 1 as s32;
    }
    count = 0 as ::core::ffi::c_int as s32;
    while <i32 as melee_core::ppc::Ppc>::pdiv(num, powi(10 as s32, count)) != 0 as ::core::ffi::c_int {
        count += 1 as ::core::ffi::c_int;
        if !(count < 11 as ::core::ffi::c_int) {
            break;
        }
    }
    return count;
}
pub unsafe extern "C" fn mn_8022EBDC() {
    melee_core::ct!("mn_8022EBDC");
    let mut i: ::core::ffi::c_int = 0;
    i = 1 as ::core::ffi::c_int;
    while i < 9 as ::core::ffi::c_int {
        mn_8022F0F0(i);
        i += 1;
    }
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
        ($h:ident, $name:ident, ($($a:ident: $t:ty),*) -> !) => {
            pub static mut $h: Option<unsafe extern "C" fn($($t),*) -> !> = None;
            pub unsafe extern "C" fn $name($($a: $t),*) -> ! {
                match $h {
                    Some(f) => f($($a),*),
                    None => {
                        // the engine's by-name table (__assert: melee_runtime debug::__assert, which reports file:line and panics)
                        let mut w: std::vec::Vec<u64> = std::vec::Vec::new();
                        $( $crate::tu::ext_hook::PushW::push_w($a, &mut w); )*
                        $crate::tu::ext_hook::fallback(stringify!($name), &w);
                        panic!(concat!(stringify!($name), " returned"))
                    }
                }
            }
        };
        ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $h: Option<unsafe extern "C" fn($($t),*) $(-> $r)?> = None;
            pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                match $h {
                    Some(f) => f($($a),*),
                    // the engine's by-name table (ext_hook::fallback: the canonical port or the host seam), else the TODO stop
                    None => {
                        let mut w: std::vec::Vec<u64> = std::vec::Vec::new();
                        $( $crate::tu::ext_hook::PushW::push_w($a, &mut w); )*
                        <$crate::ext_rty!($($r)?) as $crate::tu::ext_hook::Val>::from_u64($crate::tu::ext_hook::fallback(stringify!($name), &w))
                    }
                }
            }
        };
    }
    hook!(H_PSMTXRotAxisRad, PSMTXRotAxisRad, (a0: *mut [f32_0; 4], a1: *mut Vec3, a2: f32_0));
    hook!(H_PSMTXMultVec, PSMTXMultVec, (a0: *mut [f32_0; 4], a1: *mut Vec3, a2: *mut Vec3));
    hook!(H_PSVECSubtract, PSVECSubtract, (a0: *mut Vec3, a1: *mut Vec3, a2: *mut Vec3), crate::tu::adapt::PSVECSubtract__594d9f);
    hook!(H_PSVECScale, PSVECScale, (a0: *mut Vec3, a1: *mut Vec3, a2: f32_0));
    hook!(H_PSVECNormalize, PSVECNormalize, (a0: *mut Vec3, a1: *mut Vec3), crate::tu::adapt::PSVECNormalize__01cbbc);
    hook!(H_PSVECCrossProduct, PSVECCrossProduct, (a0: *mut Vec3, a1: *mut Vec3, a2: *mut Vec3), crate::tu::adapt::PSVECCrossProduct__0fba16);
    hook!(H_OSReport, OSReport, (a0: *mut ::core::ffi::c_char, va: &[usize]));
    hook!(H___assert, __assert, (a0: *const ::core::ffi::c_char, a1: u32_0, a2: *const ::core::ffi::c_char) -> !);
    pub static mut H_mn_8022ED6C: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut AnimLoopSettings) -> ::core::ffi::c_float> = None;
            pub unsafe extern "C" fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> ::core::ffi::c_float {
                match H_mn_8022ED6C {
                    Some(f) => f(a0, a1),
                    None => { return unsafe { crate::tu::mn_22EC::mn_8022ED6C(a0 as _, a1 as _) as _ }; },
                }
            }
    pub static mut H_mn_8022F0F0: Option<unsafe extern "C" fn(::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn mn_8022F0F0(a0: ::core::ffi::c_int)  {
                match H_mn_8022F0F0 {
                    Some(f) => f(a0),
                    None => { unsafe { crate::tu::mn_22EC::mn_8022F0F0(a0 as _); } return; },
                }
            }
    pub static mut H_mn_8022F218: Option<unsafe extern "C" fn() -> bool_0> = None;
            pub unsafe extern "C" fn mn_8022F218() -> bool_0 {
                match H_mn_8022F218 {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::mn_22EC::mn_8022F218() as _ }; },
                }
            }
    pub static mut H_mn_8022F268: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn mn_8022F268()  {
                match H_mn_8022F268 {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::mn_22EC::mn_8022F268(); } return; },
                }
            }
    pub static mut H_mn_8022F298: Option<unsafe extern "C" fn(*mut HSD_JObj) -> f32_0> = None;
            pub unsafe extern "C" fn mn_8022F298(a0: *mut HSD_JObj) -> f32_0 {
                match H_mn_8022F298 {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::mn_22EC::mn_8022F298(a0 as _) as _ }; },
                }
            }
    pub static mut H_mn_8022F3D8: Option<unsafe extern "C" fn(*mut HSD_JObj, u8_0, HSD_TypeMask) > = None;
            pub unsafe extern "C" fn mn_8022F3D8(a0: *mut HSD_JObj, a1: u8_0, a2: HSD_TypeMask)  {
                match H_mn_8022F3D8 {
                    Some(f) => f(a0, a1, a2),
                    None => { unsafe { crate::tu::mn_22EC::mn_8022F3D8(a0 as _, a1 as _, a2 as _); } return; },
                }
            }
    pub static mut H_atan2f: Option<unsafe extern "C" fn(::core::ffi::c_float, ::core::ffi::c_float) -> ::core::ffi::c_float> = None;
            pub unsafe extern "C" fn atan2f(a0: ::core::ffi::c_float, a1: ::core::ffi::c_float) -> ::core::ffi::c_float {
                match H_atan2f {
                    Some(f) => f(a0, a1),
                    None => { return unsafe { melee_runtime::lbtrigf::atan2f(a0 as _, a1 as _) as _ }; },
                }
            }
    hook!(H_HSD_GObj_803910D8, HSD_GObj_803910D8, (a0: *mut HSD_GObj, a1: isize));
    hook!(H_GObj_Create, GObj_Create, (a0: u16_0, a1: u8_0, a2: u8_0) -> *mut HSD_GObj);
    pub static mut H_HSD_GObj_JObjCallback: Option<unsafe extern "C" fn(*mut HSD_GObj, isize) > = None;
            pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: isize)  {
                match H_HSD_GObj_JObjCallback {
                    Some(f) => f(a0, a1),
                    None => { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } return; },
                }
            }
    hook!(H_HSD_GObj_80390CD4, HSD_GObj_80390CD4, (a0: *mut HSD_GObj));
    pub static mut H_HSD_GObj_LObjCallback: Option<unsafe extern "C" fn(*mut HSD_GObj, isize) > = None;
            pub unsafe extern "C" fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: isize)  {
                match H_HSD_GObj_LObjCallback {
                    Some(f) => f(a0, a1),
                    None => { unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0 as _, a1 as _); } return; },
                }
            }
    hook!(H_HSD_GObj_80390ED0, HSD_GObj_80390ED0, (a0: *mut HSD_GObj, a1: u32_0));
    pub static mut H_lbAudioAx_80023694: Option<unsafe extern "C" fn() -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn lbAudioAx_80023694() -> ::core::ffi::c_int {
                match H_lbAudioAx_80023694 {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023694() as _ }; },
                }
            }
    pub static mut H_lbAudioAx_800236DC: Option<unsafe extern "C" fn() -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn lbAudioAx_800236DC() -> ::core::ffi::c_int {
                match H_lbAudioAx_800236DC {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC() as _ }; },
                }
            }
    pub static mut H_lbAudioAx_8002392C: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn lbAudioAx_8002392C()  {
                match H_lbAudioAx_8002392C {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002392C(); } return; },
                }
            }
    pub static mut H_lbAudioAx_80023F28: Option<unsafe extern "C" fn(::core::ffi::c_int) -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn lbAudioAx_80023F28(a0: ::core::ffi::c_int) -> ::core::ffi::c_int {
                match H_lbAudioAx_80023F28 {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0 as _) as _ }; },
                }
            }
    pub static mut H_lbAudioAx_80024030: Option<unsafe extern "C" fn(::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn lbAudioAx_80024030(a0: ::core::ffi::c_int)  {
                match H_lbAudioAx_80024030 {
                    Some(f) => f(a0),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0 as _); } return; },
                }
            }
    pub static mut H_lb_80011AC4: Option<unsafe extern "C" fn(*mut *mut LightList) -> *mut HSD_LObj> = None;
            pub unsafe extern "C" fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj {
                match H_lb_80011AC4 {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(a0 as _) as _ }; },
                }
            }
    hook!(H_lb_80011E24, lb_80011E24, (a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, va: &[usize]) -> ::core::ffi::c_int);
    pub static mut H_lb_8001204C: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut *mut HSD_JObj, *mut u16_0, ::core::ffi::c_int) -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn lb_8001204C(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: *mut u16_0, a3: ::core::ffi::c_int) -> ::core::ffi::c_int {
                match H_lb_8001204C {
                    Some(f) => f(a0, a1, a2, a3),
                    None => { return unsafe { melee_runtime::tu::lbspdisplay::lb_8001204C(a0 as _, a1 as _, a2 as _, a3 as _) as _ }; },
                }
            }
    hook!(H_HSD_JObjReqAnimAll, HSD_JObjReqAnimAll, (a0: *mut HSD_JObj, a1: f32_0));
    hook!(H_HSD_JObjSetMtxDirtySub, HSD_JObjSetMtxDirtySub, (a0: *mut HSD_JObj));
    pub static mut H_HSD_JObjLoadJoint: Option<unsafe extern "C" fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
            pub unsafe extern "C" fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj {
                match H_HSD_JObjLoadJoint {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ }; },
                }
            }
    hook!(H_HSD_JObjAddAnimAll, HSD_JObjAddAnimAll, (a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint));
    hook!(H_HSD_JObjAnimAll, HSD_JObjAnimAll, (a0: *mut HSD_JObj));
    hook!(H_HSD_JObjSetFlagsAll, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, a1: u32_0));
    hook!(H_HSD_JObjClearFlagsAll, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, a1: u32_0));
    hook!(H_HSD_JObjAddChild, HSD_JObjAddChild, (a0: *mut HSD_JObj, a1: *mut HSD_JObj));
    hook!(H_HSD_JObjReqAnim, HSD_JObjReqAnim, (a0: *mut HSD_JObj, a1: f32_0));
    hook!(H_HSD_JObjAnim, HSD_JObjAnim, (a0: *mut HSD_JObj));
    pub static mut H_HSD_CObjEraseScreen: Option<unsafe extern "C" fn(*mut HSD_CObj, s32, s32, s32) > = None;
            pub unsafe extern "C" fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: s32, a2: s32, a3: s32)  {
                match H_HSD_CObjEraseScreen {
                    Some(f) => f(a0, a1, a2, a3),
                    None => { unsafe { melee_runtime::tu::cobj::HSD_CObjEraseScreen(a0 as _, a1 as _, a2 as _, a3 as _); } return; },
                }
            }
    hook!(H_HSD_CObjSetEyePosition, HSD_CObjSetEyePosition, (a0: *mut HSD_CObj, a1: *mut Vec3));
    pub static mut H_HSD_CObjSetCurrent: Option<unsafe extern "C" fn(*mut HSD_CObj) -> bool_0> = None;
            pub unsafe extern "C" fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> bool_0 {
                match H_HSD_CObjSetCurrent {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0 as _) as _ }; },
                }
            }
    pub static mut H_HSD_CObjEndCurrent: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn HSD_CObjEndCurrent()  {
                match H_HSD_CObjEndCurrent {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::cobj::HSD_CObjEndCurrent(); } return; },
                }
            }
    hook!(H_HSD_CObjGetEyePosition, HSD_CObjGetEyePosition, (a0: *mut HSD_CObj, a1: *mut Vec3));
    hook!(H_HSD_CObjGetEyeVector, HSD_CObjGetEyeVector, (a0: *mut HSD_CObj, a1: *mut Vec3) -> ::core::ffi::c_int);
    hook!(H_HSD_CObjGetInterest, HSD_CObjGetInterest, (a0: *mut HSD_CObj, a1: *mut Vec3));
    hook!(H_HSD_CObjGetEyeDistance, HSD_CObjGetEyeDistance, (a0: *mut HSD_CObj) -> f32_0);
    pub static mut H_HSD_CObjInit: Option<unsafe extern "C" fn(*mut HSD_CObj, *mut HSD_CObjDesc) > = None;
            pub unsafe extern "C" fn HSD_CObjInit(a0: *mut HSD_CObj, a1: *mut HSD_CObjDesc)  {
                match H_HSD_CObjInit {
                    Some(f) => f(a0, a1),
                    None => { unsafe { melee_runtime::tu::cobj::HSD_CObjInit(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_HSD_CObjLoadDesc: Option<unsafe extern "C" fn(*mut HSD_CObjDesc) -> *mut HSD_CObj> = None;
            pub unsafe extern "C" fn HSD_CObjLoadDesc(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj {
                match H_HSD_CObjLoadDesc {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::cobj::HSD_CObjLoadDesc(a0 as _) as _ }; },
                }
            }
    pub static mut H_HSD_SisLib_803A6368: Option<unsafe extern "C" fn(*mut HSD_Text, s32) > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: s32)  {
                match H_HSD_SisLib_803A6368 {
                    Some(f) => f(a0, a1),
                    None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A6368(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_HSD_SisLib_803A62A0: Option<unsafe extern "C" fn(s32, *mut ::core::ffi::c_char, *mut ::core::ffi::c_char) > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A62A0(a0: s32, a1: *mut ::core::ffi::c_char, a2: *mut ::core::ffi::c_char)  {
                match H_HSD_SisLib_803A62A0 {
                    Some(f) => f(a0, a1, a2),
                    None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A62A0(a0 as _, a1 as _, a2 as _); } return; },
                }
            }
    pub static mut H_HSD_SisLib_803A611C: Option<unsafe extern "C" fn(::core::ffi::c_int, *mut HSD_GObj, u16_0, u8_0, u8_0, u8_0, u8_0, u32_0) -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn HSD_SisLib_803A611C(a0: ::core::ffi::c_int, a1: *mut HSD_GObj, a2: u16_0, a3: u8_0, a4: u8_0, a5: u8_0, a6: u8_0, a7: u32_0) -> ::core::ffi::c_int {
                match H_HSD_SisLib_803A611C {
                    Some(f) => f(a0, a1, a2, a3, a4, a5, a6, a7),
                    None => { return unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A611C(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _) as _ }; },
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
                    None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5CC4(a0 as _); } return; },
                }
            }
    pub static mut H_HSD_SisLib_803A5E70: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A5E70()  {
                match H_HSD_SisLib_803A5E70 {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5E70(); } return; },
                }
            }
    pub static mut H_mnCount_Create: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn mnCount_Create()  {
                match H_mnCount_Create {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::mncount::mnCount_Create(); } return; },
                }
            }
    pub static mut H_mnDataDel_80250170: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn mnDataDel_80250170()  {
                match H_mnDataDel_80250170 {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::mndatadel::mnDataDel_80250170(); } return; },
                }
            }
    pub static mut H_mnDeflicker_8024A6C4: Option<unsafe extern "C" fn(::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn mnDeflicker_8024A6C4(a0: ::core::ffi::c_int)  {
                match H_mnDeflicker_8024A6C4 {
                    Some(f) => f(a0),
                    None => { unsafe { crate::tu::mndeflicker::mnDeflicker_8024A6C4(a0 as _); } return; },
                }
            }
    pub static mut H_mnDiagram_Init: Option<unsafe extern "C" fn(u8_0, u8_0) > = None;
            pub unsafe extern "C" fn mnDiagram_Init(a0: u8_0, a1: u8_0)  {
                match H_mnDiagram_Init {
                    Some(f) => f(a0, a1),
                    None => { unsafe { crate::tu::mndiagram::mnDiagram_Init(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_mnEvent_8024E838: Option<fn(::core::ffi::c_int, bool_0)> = None;
    #[allow(non_snake_case)]
    pub fn mnEvent_8024E838(a0: ::core::ffi::c_int, a1: bool_0) {
        match unsafe { H_mnEvent_8024E838 } {
            Some(f) => f(a0, a1),
            None => { unsafe { crate::tu::mnevent::mnEvent_8024E838(a0 as _, (a1 != 0)); } },
        }
    }
    pub static mut H_mnGallery_80259868: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn mnGallery_80259868() {
        match unsafe { H_mnGallery_80259868 } {
            Some(f) => f(),
            None => { unsafe { crate::tu::mngallery::mnGallery_80259868(); } },
        }
    }
    hook!(H_HSD_GObjObject_80390A70, HSD_GObjObject_80390A70, (a0: *mut HSD_GObj, a1: u8_0, a2: *mut ::core::ffi::c_void));
    hook!(H_HSD_GObj_SetupProc, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8_0) -> *mut HSD_GObjProc);
    hook!(H_HSD_GObjProc_RemoveProc, HSD_GObjProc_RemoveProc, (a0: *mut HSD_GObjProc));
    pub static mut H_mnSnap_80257F24: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn mnSnap_80257F24()  {
                match H_mnSnap_80257F24 {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::mnsnap::mnSnap_80257F24(); } return; },
                }
            }
    pub static mut H_mnSound_8024A09C: Option<unsafe extern "C" fn(::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn mnSound_8024A09C(a0: ::core::ffi::c_int)  {
                match H_mnSound_8024A09C {
                    Some(f) => f(a0),
                    None => { unsafe { crate::tu::mnsound::mnSound_8024A09C(a0 as _); } return; },
                }
            }
    pub static mut H_mnSoundTest_8024BEE0: Option<unsafe extern "C" fn(s32) -> *mut HSD_GObjProc> = None;
            pub unsafe extern "C" fn mnSoundTest_8024BEE0(a0: s32) -> *mut HSD_GObjProc {
                match H_mnSoundTest_8024BEE0 {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::mnsoundtest::mnSoundTest_8024BEE0(a0 as _) as _ }; },
                }
            }
    pub static mut H_mnVibration_Init: Option<unsafe extern "C" fn(::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn mnVibration_Init(a0: ::core::ffi::c_int)  {
                match H_mnVibration_Init {
                    Some(f) => f(a0),
                    None => { unsafe { crate::tu::mnvibration::mnVibration_Init(a0 as _); } return; },
                }
            }
    pub static mut H_gm_801603B0: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn gm_801603B0()  {
                match H_gm_801603B0 {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::gm_1601::gm_801603B0(); } return; },
                }
            }
    pub static mut H_gm_SelKindToCKind: Option<unsafe extern "C" fn(u8_0) -> u8_0> = None;
            pub unsafe extern "C" fn gm_SelKindToCKind(a0: u8_0) -> u8_0 {
                match H_gm_SelKindToCKind {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::gm::gm_SelKindToCKind(a0 as _) as _ }; },
                }
            }
    pub static mut H_gm_IsCKindUnlocked: Option<unsafe extern "C" fn(u8_0) -> bool_0> = None;
            pub unsafe extern "C" fn gm_IsCKindUnlocked(a0: u8_0) -> bool_0 {
                match H_gm_IsCKindUnlocked {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::gm_1601::gm_IsCKindUnlocked(a0 as _) as _ }; },
                }
            }
    pub static mut H_gm_801677E8: Option<unsafe extern "C" fn(s8) > = None;
            pub unsafe extern "C" fn gm_801677E8(a0: s8)  {
                match H_gm_801677E8 {
                    Some(f) => f(a0),
                    None => { unsafe { crate::tu::gm_1601::gm_801677E8(a0 as _); } return; },
                }
            }
    pub static mut H_gm_GetButtonsTriggered: Option<unsafe extern "C" fn(u8_0) -> u64_0> = None;
            pub unsafe extern "C" fn gm_GetButtonsTriggered(a0: u8_0) -> u64_0 {
                match H_gm_GetButtonsTriggered {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::gm_1A36::gm_GetButtonsTriggered(a0 as _) as _ }; },
                }
            }
    pub static mut H_gm_801A36C0: Option<unsafe extern "C" fn(u8_0) -> u64_0> = None;
            pub unsafe extern "C" fn gm_801A36C0(a0: u8_0) -> u64_0 {
                match H_gm_801A36C0 {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::gm_1A36::gm_801A36C0(a0 as _) as _ }; },
                }
            }
    pub static mut H_gm_801BA8FC: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn gm_801BA8FC()  {
                match H_gm_801BA8FC {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::gmevent::gm_801BA8FC(); } return; },
                }
            }
    pub static mut H_gm_801BEB80: Option<unsafe extern "C" fn() -> u8_0> = None;
            pub unsafe extern "C" fn gm_801BEB80() -> u8_0 {
                match H_gm_801BEB80 {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmevent::gm_801BEB80() as _ }; },
                }
            }
    pub static mut H_gm_801A4B60: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn gm_801A4B60()  {
                match H_gm_801A4B60 {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::gmscene::gm_801A4B60(); } return; },
                }
            }
    pub static mut H_gm_GetCurrentSceneExitData: Option<unsafe extern "C" fn() -> *mut ::core::ffi::c_void> = None;
            pub unsafe extern "C" fn gm_GetCurrentSceneExitData() -> *mut ::core::ffi::c_void {
                match H_gm_GetCurrentSceneExitData {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmscene::gm_GetCurrentSceneExitData() as _ }; },
                }
            }
    pub static mut H_gmMainLib_GetGamePrefs: Option<unsafe extern "C" fn() -> *mut GamePrefs> = None;
            pub unsafe extern "C" fn gmMainLib_GetGamePrefs() -> *mut GamePrefs {
                match H_gmMainLib_GetGamePrefs {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_GetGamePrefs() as _ }; },
                }
            }
    pub static mut H_gmMainLib_8015ECB0: Option<unsafe extern "C" fn() -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn gmMainLib_8015ECB0() -> ::core::ffi::c_int {
                match H_gmMainLib_8015ECB0 {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015ECB0() as _ }; },
                }
            }
    pub static mut H_gmMainLib_8015EDD4: Option<unsafe extern "C" fn() -> s32> = None;
            pub unsafe extern "C" fn gmMainLib_8015EDD4() -> s32 {
                match H_gmMainLib_8015EDD4 {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015EDD4() as _ }; },
                }
            }
    pub static mut H_gmMainLib_8015EE90: Option<unsafe extern "C" fn() -> s32> = None;
            pub unsafe extern "C" fn gmMainLib_8015EE90() -> s32 {
                match H_gmMainLib_8015EE90 {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015EE90() as _ }; },
                }
            }
    pub static mut H_lb_8000B1CC: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut Vec3, *mut Vec3) > = None;
            pub unsafe extern "C" fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec3, a2: *mut Vec3)  {
                match H_lb_8000B1CC {
                    Some(f) => f(a0, a1, a2),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } return; },
                }
            }
    pub static mut H_powi: Option<unsafe extern "C" fn(s32, s32) -> s32> = None;
            pub unsafe extern "C" fn powi(a0: s32, a1: s32) -> s32 {
                match H_powi {
                    Some(f) => f(a0, a1),
                    None => { return unsafe { crate::lb::powi(a0 as _, a1 as _) as _ }; },
                }
            }
    hook!(H_lbArchive_LoadSymbols, lbArchive_LoadSymbols, (a0: *const ::core::ffi::c_char, a1: *mut ::core::ffi::c_void, va: &[usize]) -> *mut HSD_Archive);
    pub static mut H_lbCardNew_CompleteAllTasks: Option<unsafe extern "C" fn(::core::ffi::c_int) -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn lbCardNew_CompleteAllTasks(a0: ::core::ffi::c_int) -> ::core::ffi::c_int {
                match H_lbCardNew_CompleteAllTasks {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::lbcardnew::lbCardNew_CompleteAllTasks(a0 as _) as _ }; },
                }
            }
    pub static mut H_lbCardGame_SaveChanges: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn lbCardGame_SaveChanges()  {
                match H_lbCardGame_SaveChanges {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_SaveChanges(); } return; },
                }
            }
    pub static mut H_lbLang_IsSavedLanguageUS: Option<unsafe extern "C" fn() -> bool_0> = None;
            pub unsafe extern "C" fn lbLang_IsSavedLanguageUS() -> bool_0 {
                match H_lbLang_IsSavedLanguageUS {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageUS() as _ }; },
                }
            }
    pub static mut H_lbMthp_8001F800: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn lbMthp_8001F800()  {
                match H_lbMthp_8001F800 {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::lbmthp::lbMthp_8001F800(); } return; },
                }
            }
    hook!(H_HSD_SetEraseColor, HSD_SetEraseColor, (a0: u8_0, a1: u8_0, a2: u8_0, a3: u8_0));
    hook!(H_HSD_FogSet, HSD_FogSet, (a0: *mut HSD_Fog));
    hook!(H_HSD_FogLoadDesc, HSD_FogLoadDesc, (a0: *mut HSD_FogDesc) -> *mut HSD_Fog);
    hook!(H_GObj_SetupGXLink, GObj_SetupGXLink, (a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8_0, a3: u32_0));
    hook!(H_GObj_SetupGXLinkMax, GObj_SetupGXLinkMax, (a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u32_0));
    hook!(H_HSD_GObjFree, HSD_GObjFree, (a0: *mut HSD_GObj));
    hook!(H_GObj_InitUserData, GObj_InitUserData, (a0: *mut HSD_GObj, a1: u8_0, a2: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>, a3: *mut ::core::ffi::c_void));
    // rp3 HSD_LObjAnimAll
    pub static mut H_HSD_LObjAnimAll: Option<unsafe extern "C" fn(*mut HSD_LObj)> = None;
    #[allow(non_snake_case, unused_unsafe)]
    pub unsafe extern "C" fn HSD_LObjAnimAll(a0: *mut HSD_LObj) {
        match unsafe { H_HSD_LObjAnimAll } {
            Some(f) => f(a0),
            None => unsafe { melee_runtime::tu::lobj::HSD_LObjAnimAll(a0); },
        }
    }
    // rp3-end

    pub static mut H_HSD_Free: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) > = None;
            pub unsafe extern "C" fn HSD_Free(a0: *mut ::core::ffi::c_void)  {
                match H_HSD_Free {
                    Some(f) => f(a0),
                    None => { unsafe { melee_runtime::tu::memory::HSD_Free(a0 as _); } return; },
                }
            }
    pub static mut H_HSD_MemAlloc: Option<unsafe extern "C" fn(ssize_t) -> *mut ::core::ffi::c_void> = None;
            pub unsafe extern "C" fn HSD_MemAlloc(a0: ssize_t) -> *mut ::core::ffi::c_void {
                match H_HSD_MemAlloc {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ }; },
                }
            }
    pub static mut H_gmFrontend_NativeRequest: Option<unsafe extern "C" fn(*mut u8_0, *mut u8_0) -> bool_0> = None;
            pub unsafe extern "C" fn gmFrontend_NativeRequest(a0: *mut u8_0, a1: *mut u8_0) -> bool_0 {
                match H_gmFrontend_NativeRequest {
                    Some(f) => f(a0, a1),
                    None => { return unsafe { crate::tu::gmfrontend::gmFrontend_NativeRequest(a0 as _, a1 as _) as _ }; },
                }
            }
    pub static mut H_gmFrontend_TakeNativeRequest: Option<unsafe extern "C" fn(*mut u8_0, *mut u8_0) -> bool_0> = None;
            pub unsafe extern "C" fn gmFrontend_TakeNativeRequest(a0: *mut u8_0, a1: *mut u8_0) -> bool_0 {
                match H_gmFrontend_TakeNativeRequest {
                    Some(f) => f(a0, a1),
                    None => { return unsafe { crate::tu::gmfrontend::gmFrontend_TakeNativeRequest(a0 as _, a1 as _) as _ }; },
                }
            }
    pub static mut H_gmFrontend_NativeReturn: Option<unsafe extern "C" fn(::core::ffi::c_int, ::core::ffi::c_int) -> bool_0> = None;
            pub unsafe extern "C" fn gmFrontend_NativeReturn(a0: ::core::ffi::c_int, a1: ::core::ffi::c_int) -> bool_0 {
                match H_gmFrontend_NativeReturn {
                    Some(f) => f(a0, a1),
                    None => { return unsafe { crate::tu::gmfrontend::gmFrontend_NativeReturn(a0 as _, a1 as _) as _ }; },
                }
            }
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CurrentInvokedProcGObj;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CurrentInvokedProc;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_804D783C;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_FogKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_LightKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use melee_runtime::tu::controller::HSD_PadCopyStatus;
}

// ---- folded from mn.rs (consolidation steps 21/22) ----
/// mnmain.c `GetSelectionFrameOffset_fp`.
pub fn GetSelectionFrameOffset_fp(selection: i32) -> i32 {
    melee_core::ct!("GetSelectionFrameOffset_fp");
    selection.wrapping_mul(2)
}

/// mnmain.c `mn_8022C010_fp`: which main-menu entry a menu kind belongs to.
/// MENU_KIND_34 and up fall off the end of the C function (no return value; undefined); this
/// returns 0 there, which is not verified against anything.
pub fn mn_8022C010_fp(menu_kind: i32, selection: i32) -> i32 {
    melee_core::ct!("mn_8022C010_fp");
    match menu_kind {
        0 => selection,
        1 | 6 | 7 | 8 | 9 | 10 | 33 => 0,
        2 | 11..=18 => 1,
        3 => 2,
        4 | 19..=24 => 3,
        5 | 25..=32 => 4,
        _ => 0, // TODO(group6): UB in the C; check the shipping build's actual return register.
    }
}

// ---- Round 2 ----

/// mnmain.c `mn_GetDigitAt_fp`: `(n / 10^place) % 10` through lb_00CE `powi` (place 0..31, where
/// the wrapping 10^place is never 0).
pub fn mn_GetDigitAt_fp(arg0: i32, arg1: i32) -> i32 {
    melee_core::ct!("mn_GetDigitAt_fp");
    arg0.wrapping_div(crate::lb::powi(10, arg1)).wrapping_rem(10)
}

/// mnmain.c `mn_GetDigitCount_fp`: decimal digits of `num`, at most 11.
pub fn mn_GetDigitCount_fp(num: i32) -> i32 {
    melee_core::ct!("mn_GetDigitCount_fp");
    if num == 0 {
        return 1;
    }
    let mut count = 0;
    while <i32 as melee_core::ppc::Ppc>::pdiv(num, crate::lb::powi(10, count)) != 0 {
        count += 1;
        if count >= 11 {
            break;
        }
    }
    count
}

// ---- folded from mn_ptr.rs (consolidation steps 21/22) ----
/// mnmain.c `GetAnimStartFrame_fp`.
pub fn GetAnimStartFrame_fp(anim_loop: &AnimLoopSettings_mn_ptr) -> f32 {
    melee_core::ct!("GetAnimStartFrame_fp");
    anim_loop.start_frame
}

/// mnmain.c `GetAnimEndFrame_fp`.
pub fn GetAnimEndFrame_fp(anim_loop: &AnimLoopSettings_mn_ptr) -> f32 {
    melee_core::ct!("GetAnimEndFrame_fp");
    anim_loop.end_frame
}
