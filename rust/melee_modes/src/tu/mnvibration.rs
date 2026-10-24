//! src/melee/mn/mnvibration.c: the Rumble options menu (per-port rumble toggles, the scrolling
//! name-tag rumble list, the port panels and the cursor), its intro / think procs and the
//! scene builder.
//! The retail build reads the anim settings, the name-position vector and the symbol-name strings
//! as one `MnVibrationDataLayout` through the address of the first; the PC build (and this port)
//! keeps them in one real object, `mnVibration_PcLayout`.
//! Callees in TUs that are not ported yet are `ext` stubs named like the C function.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_mut, unused_variables, unused_assignments, clippy::missing_safety_doc)]

use crate::mn_inl::*;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

/// `MnVibrationJointAssets` (mnvibration.c): the four archive section pointers of one model.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MnVibrationJointAssets {
    pub joint: *mut c_void,
    pub animjoint: *mut c_void,
    pub matanim: *mut c_void,
    pub shapeanim: *mut c_void,
}

melee_core::gc_layout!(MnVibrationJointAssets, "MnVibrationJointAssets", size 0x10, align 4, [
    joint @ 0x0: 4 "void*",
    animjoint @ 0x4: 4 "void*",
    matanim @ 0x8: 4 "void*",
    shapeanim @ 0xC: 4 "void*",
]);

/// `MnVibrationData` (mnvibration.c): the menu's user data (0x98 bytes on the GameCube).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MnVibrationData {
    /// `[0]` intro timer, `[1]` cursor row, `[2..6]` per-port rumble toggle.
    pub x0: [u8; 6],
    /// Per-port "controller connected".
    pub x6: [u8; 4],
    pub scroll_offset: u8,
    pub pad_0B: u8,
    pub jobjs: [*mut HSD_JObj; 25],
    pub texts: [*mut HSD_Text; 8],
    pub title_text: *mut HSD_Text,
    pub cursor_gobj: *mut HSD_GObj,
}

melee_core::gc_layout!(MnVibrationData, "MnVibrationData", size 0x98, align 4, [
    x0 @ 0x0: 6 "u8[6]",
    x6 @ 0x6: 4 "u8[4]",
    scroll_offset @ 0xA: 1 "u8",
    pad_0B @ 0xB: 1 "u8",
    jobjs @ 0xC: 100 "HSD_JObj*[25]",
    texts @ 0x70: 32 "HSD_Text*[8]",
    title_text @ 0x90: 4 "HSD_Text*",
    cursor_gobj @ 0x94: 4 "HSD_GObj*",
]);

/// `MnVibrationDataLayout` (mnvibration.c): three anim settings, a vector and the symbol-name /
/// message strings, read as one block.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MnVibrationDataLayout {
    pub intro_anim: AnimLoopSettings,
    pub main_anim: AnimLoopSettings,
    pub cursor_anim: AnimLoopSettings,
    pub name_pos: Vec3,
    pub user_data_error: [c_char; 0x18],
    pub file_name: [c_char; 0x10],
    pub user_data_name: [c_char; 0xC],
    pub convi_top_joint: [c_char; 0x18],
    pub convi_top_animjoint: [c_char; 0x1C],
    pub convi_top_matanim_joint: [c_char; 0x20],
    pub convi_top_shapeanim_joint: [c_char; 0x24],
    pub ctlvi_top_joint: [c_char; 0x18],
    pub ctlvi_top_animjoint: [c_char; 0x1C],
    pub ctlvi_top_matanim_joint: [c_char; 0x20],
    pub ctlvi_top_shapeanim_joint: [c_char; 0x24],
    pub onoffvi_top_joint: [c_char; 0x1C],
    pub onoffvi_top_animjoint: [c_char; 0x20],
    pub onoffvi_top_matanim_joint: [c_char; 0x24],
    pub onoffvi_top_shapeanim_joint: [c_char; 0x24],
    pub cursorvi_top_joint: [c_char; 0x20],
}

melee_core::gc_layout!(MnVibrationDataLayout, "MnVibrationDataLayout", size 0x1F8, align 4, [
    intro_anim @ 0x0: 12 "AnimLoopSettings",
    main_anim @ 0xC: 12 "AnimLoopSettings",
    cursor_anim @ 0x18: 12 "AnimLoopSettings",
    name_pos @ 0x24: 12 "Vec3",
    user_data_error @ 0x30: 24 "char[24]",
    file_name @ 0x48: 16 "char[16]",
    user_data_name @ 0x58: 12 "char[12]",
    convi_top_joint @ 0x64: 24 "char[24]",
    convi_top_animjoint @ 0x7C: 28 "char[28]",
    convi_top_matanim_joint @ 0x98: 32 "char[32]",
    convi_top_shapeanim_joint @ 0xB8: 36 "char[36]",
    ctlvi_top_joint @ 0xDC: 24 "char[24]",
    ctlvi_top_animjoint @ 0xF4: 28 "char[28]",
    ctlvi_top_matanim_joint @ 0x110: 32 "char[32]",
    ctlvi_top_shapeanim_joint @ 0x130: 36 "char[36]",
    onoffvi_top_joint @ 0x154: 28 "char[28]",
    onoffvi_top_animjoint @ 0x170: 32 "char[32]",
    onoffvi_top_matanim_joint @ 0x190: 36 "char[36]",
    onoffvi_top_shapeanim_joint @ 0x1B4: 36 "char[36]",
    cursorvi_top_joint @ 0x1D8: 32 "char[32]",
]);

pub mod ext {
    use super::*;
    use crate::ext_fns;
    ext_fns! {
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
        fn HSD_JObjRemoveAll(a0: *mut HSD_JObj);
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjAddChild(jobj: *mut HSD_JObj, child: *mut HSD_JObj);
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        extern "C" fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32) = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn mn_80229894(a0: i32, a1: u16, a2: i32) = crate::tu::adapt::mn_80229894__f7df64;
        fn mn_8022EC18(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings, a2: HSD_TypeMask) -> f32 = crate::tu::adapt::mn_8022EC18__bee2c1;
        fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32 = crate::tu::adapt::mn_8022ED6C__f73cae;
        fn mn_8022F3D8(a0: *mut HSD_JObj, a1: u8, a2: HSD_TypeMask) = crate::tu::adapt::mn_8022F3D8__9f7fac;
        fn HSD_SisLib_803A6754(arg0: i32, arg1: i32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A6754__02fb41;
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
        fn gm_GetButtonsTriggered(a0: u8) -> u64 = crate::tu::adapt::gm_GetButtonsTriggered__dca604;
        fn gm_801A36C0(a0: u8) -> u64 = crate::tu::adapt::gm_801A36C0__4faf6f;
        fn GetPersistentNameData(a0: i32) -> *mut NameTagData = crate::tu::gmmain_lib::GetPersistentNameData;
        fn GetRumbleSettingOfPort(port: isize) -> i32 = crate::tu::gmmain_lib::GetRumbleSettingOfPort;
        fn gmMainLib_SetRumbleEnabled(port: isize, enabled: i32) = crate::mn_inl::gmMainLib_SetRumbleEnabled_i32;
        fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) = crate::tu::adapt::lb_8000B1CC__69851d;
        fn HSD_PadRumbleRemoveAll() = crate::tu::adapt::HSD_PadRumbleRemoveAll__95e5b6;
        fn HSD_PadRumbleAdd(no: u8, id: i32, frame: i32, pri: i32, listp: *mut c_void) -> i32 = crate::tu::adapt::HSD_PadRumbleAdd__f8a1ce;
        fn HSD_PadRumbleOffN(no: u8) = crate::tu::adapt::HSD_PadRumbleOffN__6901ea;
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32);
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void);
        fn HSD_GObjFree(a0: *mut HSD_GObj);
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_GObjProc_RemoveProc(gproc: *mut HSD_GObjProc);
        fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void) -> ()>, data: *mut c_void, );
        extern "C" fn HSD_Free(ptr: *mut c_void) = crate::tu::adapt::HSD_Free__3ee311;
        fn HSD_MemAlloc(size: isize) -> *mut c_void = crate::tu::adapt::HSD_MemAlloc__849f1b;
        fn GetNameText(slot: u8) -> *mut c_char;
        fn GetNameCount() -> i32 = crate::tu::adapt::GetNameCount__b961fe;
        fn lbCardGame_SaveChanges() = crate::tu::adapt::lbCardGame_SaveChanges__0fc9e6;
    }
    pub static mut HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_804D783C: i32 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
}
use ext::*;

unsafe extern "C" fn mnVibration_JObjGetTranslationX(mut jobj: *mut HSD_JObj) -> f32 {
    if jobj.is_null() {
        __assert("jobj.h", 0x3e1, "jobj");
    };
    return (*jobj).translate.x;
}
unsafe extern "C" fn mnVibration_JObjGetTranslationY(mut jobj: *mut HSD_JObj) -> f32 {
    if jobj.is_null() {
        __assert("jobj.h", 0x3ee, "jobj");
    };
    return (*jobj).translate.y;
}
unsafe extern "C" fn mnVibration_JObjGetTranslationZ(mut jobj: *mut HSD_JObj) -> f32 {
    if jobj.is_null() {
        __assert("jobj.h", 0x3fb, "jobj");
    };
    return (*jobj).translate.z;
}
unsafe extern "C" fn mnVibration_JObjSetTranslateX(
    mut jobj: *mut HSD_JObj,
    mut x: f32,
) {
    let mut temp: *mut HSD_JObj = jobj;
    if jobj.is_null() {
        __assert("jobj.h", 0x3a4, "jobj");
    };
    (*jobj).translate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(temp);
    }
}
unsafe extern "C" fn mnVibration_JObjSetTranslateY(
    mut jobj: *mut HSD_JObj,
    mut y: f32,
    mut dirty_jobj: *mut HSD_JObj,
) {
    if jobj.is_null() {
        __assert("jobj.h", 0x3b3, "jobj");
    };
    (*jobj).translate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(dirty_jobj);
    }
}
unsafe extern "C" fn mnVibration_JObjSetTranslateZ(
    mut jobj: *mut HSD_JObj,
    mut z: f32,
) {
    let mut temp: *mut HSD_JObj = jobj;
    if jobj.is_null() {
        __assert("jobj.h", 0x3c2, "jobj");
    };
    (*jobj).translate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(temp);
    }
}
static mut mnVibration_PortPanelJointIds: [u16; 4] = [
    0x16 as i32 as u16,
    0x15 as i32 as u16,
    0x14 as i32 as u16,
    0x13 as i32 as u16,
];
pub static mut mnVibration_804D4FF0: i32 = 0x20010000;
pub static mut mnVibration_804D4FF4: [c_char; 7] = unsafe {
    core::mem::transmute::<[u8; 7], [c_char; 7]>(*b"jobj.h\0")
};
pub static mut mnVibration_804D4FFC: [c_char; 5] = unsafe {
    core::mem::transmute::<[u8; 5], [c_char; 5]>(*b"jobj\0")
};
pub static mut mnVibration_803EECE0: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0.0_f32,
    end_frame: 20.0_f32,
    loop_frame: -0.1_f32,
};
pub static mut mnVibration_803EECEC: AnimLoopSettings = AnimLoopSettings {
    start_frame: 50.0_f32,
    end_frame: 70.0_f32,
    loop_frame: -0.1_f32,
};
pub static mut mnVibration_803EECF8: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0.0_f32,
    end_frame: 14.0_f32,
    loop_frame: -0.1_f32,
};
pub static mut mnVibration_803EED04: Vec3 = Vec3 {
    x: -0.4_f32,
    y: 0.5_f32,
    z: 0.0_f32,
};
static mut mnVibration_PcLayout: MnVibrationDataLayout = unsafe {
    MnVibrationDataLayout {
        intro_anim: AnimLoopSettings {
            start_frame: 0.0_f32,
            end_frame: 20.0_f32,
            loop_frame: -0.1_f32,
        },
        main_anim: AnimLoopSettings {
            start_frame: 50.0_f32,
            end_frame: 70.0_f32,
            loop_frame: -0.1_f32,
        },
        cursor_anim: AnimLoopSettings {
            start_frame: 0.0_f32,
            end_frame: 14.0_f32,
            loop_frame: -0.1_f32,
        },
        name_pos: Vec3 {
            x: -0.4_f32,
            y: 0.5_f32,
            z: 0.0_f32,
        },
        user_data_error: core::mem::transmute::<
            [u8; 24],
            [c_char; 24],
        >(*b"Can't get user_data.\n\0\0\0"),
        file_name: core::mem::transmute::<
            [u8; 16],
            [c_char; 16],
        >(*b"mnvibration.c\0\0\0"),
        user_data_name: core::mem::transmute::<
            [u8; 12],
            [c_char; 12],
        >(*b"user_data\0\0\0"),
        convi_top_joint: core::mem::transmute::<
            [u8; 24],
            [c_char; 24],
        >(*b"MenMainConVi_Top_joint\0\0"),
        convi_top_animjoint: core::mem::transmute::<
            [u8; 28],
            [c_char; 28],
        >(*b"MenMainConVi_Top_animjoint\0\0"),
        convi_top_matanim_joint: core::mem::transmute::<
            [u8; 32],
            [c_char; 32],
        >(*b"MenMainConVi_Top_matanim_joint\0\0"),
        convi_top_shapeanim_joint: core::mem::transmute::<
            [u8; 36],
            [c_char; 36],
        >(*b"MenMainConVi_Top_shapeanim_joint\0\0\0\0"),
        ctlvi_top_joint: core::mem::transmute::<
            [u8; 24],
            [c_char; 24],
        >(*b"MenMainCtlVi_Top_joint\0\0"),
        ctlvi_top_animjoint: core::mem::transmute::<
            [u8; 28],
            [c_char; 28],
        >(*b"MenMainCtlVi_Top_animjoint\0\0"),
        ctlvi_top_matanim_joint: core::mem::transmute::<
            [u8; 32],
            [c_char; 32],
        >(*b"MenMainCtlVi_Top_matanim_joint\0\0"),
        ctlvi_top_shapeanim_joint: core::mem::transmute::<
            [u8; 36],
            [c_char; 36],
        >(*b"MenMainCtlVi_Top_shapeanim_joint\0\0\0\0"),
        onoffvi_top_joint: core::mem::transmute::<
            [u8; 28],
            [c_char; 28],
        >(*b"MenMainOnoffVi_Top_joint\0\0\0\0"),
        onoffvi_top_animjoint: core::mem::transmute::<
            [u8; 32],
            [c_char; 32],
        >(*b"MenMainOnoffVi_Top_animjoint\0\0\0\0"),
        onoffvi_top_matanim_joint: core::mem::transmute::<
            [u8; 36],
            [c_char; 36],
        >(*b"MenMainOnoffVi_Top_matanim_joint\0\0\0\0"),
        onoffvi_top_shapeanim_joint: core::mem::transmute::<
            [u8; 36],
            [c_char; 36],
        >(*b"MenMainOnoffVi_Top_shapeanim_joint\0\0"),
        cursorvi_top_joint: core::mem::transmute::<
            [u8; 32],
            [c_char; 32],
        >(*b"MenMainCursorVi_Top_joint\0\0\0\0\0\0\0"),
    }
};
pub static mut mnVibration_804D6C28: *mut HSD_GObj = null_mut();
static mut mnVibration_PcAssets: [MnVibrationJointAssets; 4] = [MnVibrationJointAssets {
    joint: null_mut(),
    animjoint: null_mut(),
    matanim: null_mut(),
    shapeanim: null_mut(),
}; 4];
unsafe extern "C" fn mnVibration_GetNameSlot(
    mut data: *mut MnVibrationData,
    mut j: i32,
) -> u8 {
    let mut scroll_offset: u8 = (*data).scroll_offset;
    let mut count: i32 = GetNameCount() as i32;
    let mut name_idx: i32 = 0;
    if count < 8 && j >= count {
        return 0xff as u8;
    }
    name_idx = scroll_offset as i32 + j;
    if count <= name_idx {
        return 0xff as u8;
    }
    return name_idx as u8;
}
unsafe extern "C" fn mnVibration_GetNameSlotRaw(
    mut data: *mut MnVibrationData,
    mut j: i32,
) -> i32 {
    let mut scroll_offset: u8 = (*data).scroll_offset;
    let mut count: i32 = GetNameCount() as i32;
    let mut name_idx: i32 = 0;
    if count < 8 && j >= count {
        return 0xff as i32;
    }
    name_idx = scroll_offset as i32 + j;
    if count <= name_idx {
        return 0xff as i32;
    }
    return name_idx;
}
unsafe extern "C" fn mnVibration_GetNameRumble(mut name_idx: i32) -> u8 {
    return (*GetPersistentNameData(name_idx)).rumble_enabled;
}
unsafe extern "C" fn mnVibration_GetCursorRow(mut data: *mut MnVibrationData) -> u8 {
    return (*data).x0[1 as i32 as usize];
}
unsafe extern "C" fn mnVibration_GetPreviousCursorRow(mut cursor_row: i32) -> i32 {
    return cursor_row - 1 as i32;
}
unsafe extern "C" fn mnVibration_GetNextCursorRow(mut cursor_row: i32) -> i32 {
    return cursor_row + 1 as i32;
}
unsafe extern "C" fn mnVibration_GetPortRumble(mut port: i32) -> u8 {
    return GetRumbleSettingOfPort(port as isize) as u8;
}
unsafe extern "C" fn mnVibration_FreeNameTexts(mut data: *mut MnVibrationData) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < 8 {
        HSD_SisLib_803A5CC4((*data).texts[i as usize]);
        i += 1;
    }
}
pub unsafe extern "C" fn mnVibration_GetNameRowJObj(mut count: i32) -> *mut HSD_JObj {
    let mut temp_r4: *mut HSD_JObj = null_mut();
    let mut var_r4: *mut HSD_JObj = null_mut();
    let mut data: *mut MnVibrationData = null_mut();
    data = (*mnVibration_804D6C28).user_data as *mut MnVibrationData;
    temp_r4 = (*data).jobjs[17 as i32 as usize];
    if temp_r4.is_null() {
        var_r4 = null_mut();
    } else {
        var_r4 = (*temp_r4).child as *mut HSD_JObj;
    }
    while count > 0 {
        if var_r4.is_null() {
            var_r4 = null_mut();
        } else {
            var_r4 = (*var_r4).next as *mut HSD_JObj;
        }
        count -= 1;
    }
    return var_r4 as *mut HSD_JObj;
}
unsafe extern "C" fn mnVibration_GetCursorYSpacing(
    mut base_y: f32,
    mut jobj: *mut HSD_JObj,
) -> f32 {
    return HSD_JObjGetTranslationY(jobj) - base_y;
}
unsafe extern "C" fn mnVibration_AnimatePortPanel(
    mut data: *mut MnVibrationData,
    mut port: i32,
) {
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut state: u8 = 0;
    let mut frame: u16 = 0;
    state = ((*data).x0[(port as i32 + 2 as i32) as usize]
        as i32 & 0xff as i32) as u8;
    jobj = (*((*mnVibration_804D6C28).user_data as *mut MnVibrationData))
        .jobjs[mnVibration_PortPanelJointIds[port as u8 as usize] as usize];
    frame = state as u16;
    HSD_JObjReqAnimAll(jobj, frame as f32);
    HSD_JObjAnimAll(jobj);
}
unsafe extern "C" fn mnVibration_AnimateNameRow(mut row: u8, mut state: u8) {
    let mut jobj: *mut HSD_JObj = null_mut();
    jobj = mnVibration_GetNameRowJObj(row as i32) as *mut HSD_JObj;
    HSD_JObjReqAnimAll(jobj, state as f32);
    HSD_JObjAnimAll(jobj);
}
unsafe extern "C" fn setCursorTranslateX(mut jobj: *mut HSD_JObj, mut x: f32) {
    let mut dirty_jobj: *mut HSD_JObj = jobj;
    if jobj.is_null() {
        __assert("jobj.h", 0x3a4, "jobj");
    };
    (*jobj).translate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
            HSD_JObjSetMtxDirtySub(dirty_jobj);
        }
    }
}
unsafe extern "C" fn setCursorTranslateY(mut jobj: *mut HSD_JObj, mut y: f32) {
    let mut dirty_jobj: *mut HSD_JObj = jobj;
    if jobj.is_null() {
        __assert("jobj.h", 0x3b3, "jobj");
    };
    (*jobj).translate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
            HSD_JObjSetMtxDirtySub(dirty_jobj);
        }
    }
}
unsafe extern "C" fn setCursorTranslateZ(mut jobj: *mut HSD_JObj, mut z: f32) {
    let mut dirty_jobj: *mut HSD_JObj = jobj;
    if jobj.is_null() {
        __assert("jobj.h", 0x3c2, "jobj");
    };
    (*jobj).translate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
            HSD_JObjSetMtxDirtySub(dirty_jobj);
        }
    }
}
pub unsafe extern "C" fn mnVibration_HandleInput(mut gobj: *mut HSD_GObj) {
    let mut data: *mut MnVibrationData = (*mnVibration_804D6C28).user_data
        as *mut MnVibrationData;
    let mut var_ctr: i32 = 0;
    let mut i: i32 = 0;
    let mut inputs: u64 = 0;
    let mut inputs_repeat: u64 = 0;
    let mut cursor_row: u8 = 0;
    let mut scroll_offset: u8 = 0;
    let mut name_count: i32 = 0;
    let mut name_idx_tmp: i32 = 0;
    let mut name_idx: u8 = 0;
    let mut rumble_setting: u8 = 0;
    let mut loaded_joint: *mut HSD_JObj = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut panel_jobj2: *mut HSD_JObj = null_mut();
    let mut panel_jobj3: *mut HSD_JObj = null_mut();
    if mn_804D6BC8.cooldown as i32 != 0 {
        Menu_DecrementAnimTimer();
        return;
    }
    Menu_GetAllInputs();
    inputs = gm_GetButtonsTriggered(PAD_MAX_CONTROLLERS as u8);
    if inputs as u64 & PAD_CANCEL != 0 {
        let mut exit_data: *mut MnVibrationData = core::ptr::null_mut::<
            MnVibrationData,
        >();
        sfxBack();
        mn_804A04F0.entering_menu = 0;
        mn_80229894(4, 0, 3);
        exit_data = (*mnVibration_804D6C28).user_data as *mut MnVibrationData;
        mnVibration_FreeNameTexts(exit_data);
        if !(*exit_data).title_text.is_null() {
            HSD_SisLib_803A5CC4((*exit_data).title_text);
        }
        HSD_GObjFree(mnVibration_804D6C28);
        HSD_PadRumbleRemoveAll();
        lbCardGame_SaveChanges();
        return;
    }
    if (*data).x0[0 as i32 as usize] as i32 as f32
        <= mnVibration_803EECE0.end_frame
    {
        (*data).x0[0 as i32 as usize] = (*data)
            .x0[0 as i32 as usize]
            .wrapping_add(1);
        return;
    }
    i = 0 as i32;
    while i < 4 {
        if (*data).x0[(i as i32 + 2 as i32) as usize]
            as i32 == 0
        {
            inputs = gm_GetButtonsTriggered(i as u8);
            if inputs as u64 & PAD_CONFIRM != 0 {
                let mut temp_jobj: *mut HSD_JObj = null_mut();
                sfxForward();
                if GetRumbleSettingOfPort(i as isize) != 0 {
                    gmMainLib_SetRumbleEnabled(i as isize, 0);
                    rumble_setting = mnVibration_GetPortRumble(i);
                    temp_jobj = (*((*mnVibration_804D6C28).user_data
                        as *mut MnVibrationData))
                        .jobjs[23 as i32 as usize];
                    if temp_jobj.is_null() {
                        jobj = null_mut();
                    } else {
                        jobj = (*temp_jobj).child as *mut HSD_JObj;
                    }
                    var_ctr = i;
                    while var_ctr > 0 {
                        if jobj.is_null() {
                            jobj = null_mut();
                        } else {
                            jobj = (*jobj).next as *mut HSD_JObj;
                        }
                        var_ctr -= 1;
                    }
                    lb_80011E24(jobj, &raw mut panel_jobj2, &[2, -1]);
                    HSD_JObjReqAnimAll(panel_jobj2, rumble_setting as f32);
                    HSD_JObjAnimAll(panel_jobj2);
                    HSD_PadRumbleRemoveAll();
                    HSD_PadRumbleOffN(i as u8);
                    return;
                } else {
                    gmMainLib_SetRumbleEnabled(i as isize, 1);
                    rumble_setting = mnVibration_GetPortRumble(i);
                    temp_jobj = (*((*mnVibration_804D6C28).user_data
                        as *mut MnVibrationData))
                        .jobjs[23 as i32 as usize];
                    if temp_jobj.is_null() {
                        jobj = null_mut();
                    } else {
                        jobj = (*temp_jobj).child as *mut HSD_JObj;
                    }
                    var_ctr = i;
                    while var_ctr > 0 {
                        if jobj.is_null() {
                            jobj = null_mut();
                        } else {
                            jobj = (*jobj).next as *mut HSD_JObj;
                        }
                        var_ctr -= 1;
                    }
                    lb_80011E24(jobj, &raw mut panel_jobj3, &[2, -1]);
                    HSD_JObjReqAnimAll(panel_jobj3, rumble_setting as f32);
                    HSD_JObjAnimAll(panel_jobj3);
                    HSD_PadRumbleAdd(
                        i as u8,
                        0,
                        14,
                        0,
                        &raw mut mnVibration_804D4FF0 as *mut c_void,
                    );
                    return;
                }
            }
        }
        i += 1;
    }
    if GetNameCount() == 0 {
        return;
    }
    let mut j: i32 = 0;
    j = 0 as i32;
    while j < 4 {
        inputs = gm_GetButtonsTriggered(j as u8);
        if inputs as u64 & PAD_ANY_LEFT != 0
            && (*data).x0[(j as i32 + 2 as i32) as usize]
                as i32 == 1
        {
            sfxMove();
            (*data).x0[(j as i32 + 2 as i32) as usize] = 0
                as u8;
            mnVibration_AnimatePortPanel(data, j);
        } else {
            inputs = gm_GetButtonsTriggered(j as u8);
            if inputs as u64 & PAD_ANY_RIGHT != 0
                && (*data)
                    .x0[(j as i32 + 2 as i32) as usize]
                    as i32 == 0
            {
                sfxMove();
                (*data)
                    .x0[(j as i32 + 2 as i32) as usize] = 1
                    as u8;
                mnVibration_AnimatePortPanel(data, j);
            }
        }
        j += 1;
    }
    i = j;
    inputs_repeat = 0;
    i = 0 as i32;
    while i < 4 {
        let mut selected: u8 = (*data)
            .x0[(i as i32 + 2 as i32) as usize];
        if selected as i32 == 1 {
            inputs_repeat |= gm_801A36C0(i as u8);
        }
        i += 1;
    }
    if inputs_repeat
        & ((1 as i64) << 32) as u64 != 0
    {
        sfxForward();
        cursor_row = mnVibration_GetCursorRow(data);
        name_idx = mnVibration_GetNameSlot(data, cursor_row as i32);
        rumble_setting = mnVibration_GetNameRumble(name_idx as i32);
        if rumble_setting as i32 == 1 {
            (*GetPersistentNameData(name_idx as i32)).rumble_enabled = 0;
        } else {
            (*GetPersistentNameData(name_idx as i32)).rumble_enabled = 1;
        }
        rumble_setting = mnVibration_GetNameRumble(name_idx as i32);
        cursor_row = (*data).x0[1 as i32 as usize];
        mnVibration_AnimateNameRow(cursor_row, rumble_setting);
        return;
    }
    if inputs_repeat
        & ((0x10 as i64) << 32) as u64 != 0
    {
        cursor_row = mnVibration_GetCursorRow(data);
        if cursor_row as i32 != 0 {
            let mut name_row: i32 = mnVibration_GetPreviousCursorRow(cursor_row as i32);
            name_idx = mnVibration_GetNameSlot(data, name_row);
            if name_idx as i32 != 0xff {
                let mut data2: *mut MnVibrationData = core::ptr::null_mut::<
                    MnVibrationData,
                >();
                let mut jobj17: *mut HSD_JObj = null_mut();
                let mut jobj18: *mut HSD_JObj = null_mut();
                let mut base_y: f32 = 0.;
                let mut spacing: f32 = 0.;
                let mut cursor_jobj: *mut HSD_JObj = null_mut();
                sfxMove();
                (*data).x0[1 as i32 as usize] = (*data)
                    .x0[1 as i32 as usize]
                    .wrapping_sub(1);
                data2 = (*mnVibration_804D6C28).user_data as *mut MnVibrationData;
                cursor_jobj = (*(*data).cursor_gobj).hsd_obj as *mut HSD_JObj;
                jobj17 = (*data2).jobjs[17 as i32 as usize];
                cursor_row = mnVibration_GetCursorRow(data);
                base_y = HSD_JObjGetTranslationY(jobj17);
                jobj18 = (*data2).jobjs[18 as i32 as usize];
                spacing = mnVibration_GetCursorYSpacing(base_y, jobj18);
                jobj17 = (*data2).jobjs[17 as i32 as usize];
                setCursorTranslateX(cursor_jobj, HSD_JObjGetTranslationX(jobj17));
                setCursorTranslateY(
                    cursor_jobj,
                    fmadds(
                        spacing,
                        cursor_row as f32,
                        HSD_JObjGetTranslationY((*data2).jobjs[17 as i32 as usize]),
                    ),
                );
                setCursorTranslateZ(
                    cursor_jobj,
                    HSD_JObjGetTranslationZ(
                        (*data2).jobjs[17 as i32 as usize],
                    ) as f32,
                );
            }
        } else if GetNameCount() > 8
            && (*data).scroll_offset as i32 != 0
        {
            sfxMove();
            (*data).scroll_offset = (*data).scroll_offset.wrapping_sub(1);
            mnVibration_RefreshNameRows(mnVibration_804D6C28);
        }
    } else if inputs_repeat
        & ((0x20 as i64) << 32) as u64 != 0
    {
        cursor_row = mnVibration_GetCursorRow(data);
        if (cursor_row as i32) < 7 {
            let mut name_row_0: i32 = mnVibration_GetNextCursorRow(cursor_row as i32);
            name_idx = mnVibration_GetNameSlot(data, name_row_0);
            if name_idx as i32 != 0xff {
                let mut data2_0: *mut MnVibrationData = core::ptr::null_mut::<
                    MnVibrationData,
                >();
                let mut jobj17_0: *mut HSD_JObj = null_mut();
                let mut jobj18_0: *mut HSD_JObj = null_mut();
                let mut base_y_0: f32 = 0.;
                let mut spacing_0: f32 = 0.;
                let mut cursor_jobj_0: *mut HSD_JObj = core::ptr::null_mut::<
                    HSD_JObj,
                >();
                sfxMove();
                (*data).x0[1 as i32 as usize] = (*data)
                    .x0[1 as i32 as usize]
                    .wrapping_add(1);
                data2_0 = (*mnVibration_804D6C28).user_data as *mut MnVibrationData;
                cursor_jobj_0 = (*(*data).cursor_gobj).hsd_obj as *mut HSD_JObj;
                jobj17_0 = (*data2_0).jobjs[17 as i32 as usize];
                cursor_row = mnVibration_GetCursorRow(data);
                base_y_0 = HSD_JObjGetTranslationY(jobj17_0);
                jobj18_0 = (*data2_0).jobjs[18 as i32 as usize];
                spacing_0 = HSD_JObjGetTranslationY(jobj18_0) - base_y_0;
                jobj17_0 = (*data2_0).jobjs[17 as i32 as usize];
                setCursorTranslateX(cursor_jobj_0, HSD_JObjGetTranslationX(jobj17_0));
                setCursorTranslateY(
                    cursor_jobj_0,
                    fmadds(
                        spacing_0,
                        cursor_row as f32,
                        HSD_JObjGetTranslationY((*data2_0).jobjs[17 as i32 as usize]),
                    ),
                );
                setCursorTranslateZ(
                    cursor_jobj_0,
                    HSD_JObjGetTranslationZ(
                        (*data2_0).jobjs[17 as i32 as usize],
                    ) as f32,
                );
            }
        } else if GetNameCount() > 8 {
            if mnVibration_GetNameSlotRaw(data, 8) as u8 as i32
                != 0xff
            {
                sfxMove();
                (*data).scroll_offset = (*data).scroll_offset.wrapping_add(1);
                if (*data).scroll_offset as i32 >= GetNameCount() {
                    (*data).scroll_offset = 0;
                }
                mnVibration_RefreshNameRows(mnVibration_804D6C28);
            }
        }
    }
}
pub unsafe extern "C" fn mnVibration_CursorThink(mut gobj: *mut HSD_GObj) {
    if mn_804A04F0.cur_menu as i32 != 0x13 {
        HSD_GObjFree(gobj);
    }
}
pub unsafe extern "C" fn mnVibration_UpdatePortPanel(
    mut arg0: *mut HSD_JObj,
    mut arg1: u8,
    mut arg2: u8,
) {
    let mut sp14: *mut HSD_JObj = null_mut();
    let mut sp10: *mut HSD_JObj = null_mut();
    let mut temp_ret: u8 = 0;
    let mut floats: *mut MnVibrationDataLayout = &raw mut mnVibration_PcLayout;
    lb_80011E24(arg0, &raw mut sp14, &[1, -1]);
    if arg2 as i32 != 0 {
        HSD_JObjReqAnimAll(sp14, arg1 as f32);
        HSD_JObjAnimAll(sp14);
        if GetRumbleSettingOfPort(arg1 as isize) != 0 {
            HSD_JObjReqAnimAll(sp14, (*floats).cursor_anim.end_frame);
        } else {
            HSD_JObjReqAnimAll(sp14, (*floats).cursor_anim.start_frame);
        }
        mn_8022F3D8(sp14, 0xff, MOBJ_MASK);
        HSD_JObjAnimAll(sp14);
    } else {
        HSD_JObjReqAnimAll(sp14, 20.0_f32);
        HSD_JObjAnimAll(sp14);
        HSD_JObjReqAnimAll(sp14, (*floats).cursor_anim.end_frame);
        mn_8022F3D8(sp14, 0xff, MOBJ_MASK);
        HSD_JObjAnimAll(sp14);
    }
    lb_80011E24(arg0, &raw mut sp14, &[3, -1]);
    if arg2 as i32 != 0 {
        HSD_JObjReqAnimAll(
            sp14,
            (arg1 as i32 + 1 as i32) as f32,
        );
    } else {
        HSD_JObjReqAnimAll(sp14, 20.0_f32);
    }
    HSD_JObjAnimAll(sp14);
    if arg2 as i32 != 0 {
        temp_ret = GetRumbleSettingOfPort(arg1 as isize) as u8;
        lb_80011E24(arg0, &raw mut sp10, &[2, -1]);
        HSD_JObjReqAnimAll(sp10, temp_ret as f32);
        HSD_JObjAnimAll(sp10);
        return;
    }
    lb_80011E24(arg0, &raw mut sp14, &[2, -1]);
    HSD_JObjReqAnimAll(sp14, 20.0_f32);
    HSD_JObjAnimAll(sp14);
}
pub unsafe extern "C" fn mnVibration_CreatePortPanels(mut arg0: *mut HSD_GObj) {
    let mut data: *mut MnVibrationData = null_mut();
    let mut assets: *mut MnVibrationJointAssets = (&raw mut mnVibration_PcAssets
        as *mut MnVibrationJointAssets)
        .offset(1 as i32 as isize) as *mut MnVibrationJointAssets;
    let mut jobj0: *mut HSD_JObj = null_mut();
    let mut jobj1: *mut HSD_JObj = null_mut();
    let mut base_x: f32 = 0.;
    let mut spacing: f32 = 0.;
    let mut new_jobj: *mut HSD_JObj = null_mut();
    let mut i: i32 = 0;
    let mut connected: i32 = 0;
    data = (*arg0).user_data as *mut MnVibrationData;
    jobj0 = (*data).jobjs[23 as i32 as usize];
    base_x = HSD_JObjGetTranslationX(jobj0);
    jobj1 = (*data).jobjs[24 as i32 as usize];
    spacing = HSD_JObjGetTranslationX(jobj1) - base_x;
    i = 0 as i32;
    while i < 4 {
        new_jobj = HSD_JObjLoadJoint((*assets).joint as *mut HSD_Joint);
        HSD_JObjAddAnimAll(
            new_jobj,
            (*assets).animjoint as *mut HSD_AnimJoint,
            (*assets).matanim as *mut HSD_MatAnimJoint,
            (*assets).shapeanim as *mut HSD_ShapeAnimJoint,
        );
        HSD_JObjReqAnimAll(new_jobj, 0.0_f32);
        HSD_JObjAnimAll(new_jobj);
        HSD_JObjSetTranslateX(new_jobj, spacing * i as f32);
        if HSD_PadCopyStatus[i as u8 as usize].err as u8 as i8 as i32
            != 0
        {
            connected = 0 as i32;
        } else {
            connected = 1 as i32;
        }
        mnVibration_UpdatePortPanel(new_jobj, i as u8, connected as u8);
        HSD_JObjSetFlagsAll(new_jobj, JOBJ_HIDDEN as u32);
        HSD_JObjAddChild((*data).jobjs[23 as i32 as usize], new_jobj);
        i += 1;
    }
}
pub unsafe extern "C" fn mnVibration_CreateNameRow(
    mut arg0: *mut HSD_GObj,
    mut arg1: u8,
    mut arg2: u8,
) {
    let mut sp20: Point3d = Vec3 { x: 0., y: 0., z: 0. };
    let mut data: *mut MnVibrationData = null_mut();
    let mut assets: *mut MnVibrationJointAssets = core::ptr::null_mut::<
        MnVibrationJointAssets,
    >();
    let mut jobj17: *mut HSD_JObj = null_mut();
    let mut jobj18: *mut HSD_JObj = null_mut();
    let mut text: *mut HSD_Text = null_mut();
    let mut name_flag: u8 = 0;
    let mut new_jobj: *mut HSD_JObj = null_mut();
    let mut spacing: f32 = 0.;
    let mut pos_x: f32 = 0.;
    let mut pos_y: f32 = 0.;
    let mut pos_z: f32 = 0.;
    assets = (&raw mut mnVibration_PcAssets as *mut MnVibrationJointAssets)
        .offset(2 as i32 as isize) as *mut MnVibrationJointAssets;
    data = (*arg0).user_data as *mut MnVibrationData;
    name_flag = mnVibration_GetNameRumble(arg1 as i32);
    jobj17 = (*data).jobjs[17 as i32 as usize];
    spacing = HSD_JObjGetTranslationY(jobj17);
    jobj18 = (*data).jobjs[18 as i32 as usize];
    spacing = HSD_JObjGetTranslationY(jobj18) - spacing;
    lb_8000B1CC(
        (*data).jobjs[17 as i32 as usize],
        &raw mut mnVibration_803EED04,
        &raw mut sp20,
    );
    text = HSD_SisLib_803A6754(0, 1);
    (*data).texts[arg2 as usize] = text;
    pos_y = -fmadds(spacing, arg2 as f32, sp20.y);
    pos_z = sp20.z;
    pos_x = sp20.x;
    (*text).pos_x = pos_x;
    (*text).pos_y = pos_y;
    (*text).pos_z = pos_z;
    (*text).font_size.x = 0.03_f32 as f32;
    (*text).font_size.y = 0.03_f32 as f32;
    HSD_SisLib_803A6B98(text, 0.0_f32, 0.0_f32, GetNameText(arg1), &[]);
    new_jobj = HSD_JObjLoadJoint((*assets).joint as *mut HSD_Joint);
    HSD_JObjAddAnimAll(
        new_jobj,
        (*assets).animjoint as *mut HSD_AnimJoint,
        (*assets).matanim as *mut HSD_MatAnimJoint,
        (*assets).shapeanim as *mut HSD_ShapeAnimJoint,
    );
    HSD_JObjReqAnimAll(new_jobj, name_flag as f32);
    HSD_JObjAnimAll(new_jobj);
    HSD_JObjSetTranslateY(new_jobj, spacing * arg2 as f32);
    HSD_JObjAddChild((*data).jobjs[17 as i32 as usize], new_jobj);
}
pub unsafe extern "C" fn mnVibration_RefreshNameRows(mut arg0: *mut HSD_GObj) {
    let mut j: i32 = 0;
    let mut ptr2: *mut MnVibrationData = null_mut();
    let mut data: *mut MnVibrationData = null_mut();
    let mut i: i32 = 0;
    let mut jobj17: *mut HSD_JObj = null_mut();
    let mut child: *mut HSD_JObj = null_mut();
    let mut name_idx: i32 = 0;
    data = (*arg0).user_data as *mut MnVibrationData;
    ptr2 = data;
    i = 0 as i32;
    while i < 8 {
        if !(*data).texts[i as usize].is_null() {
            HSD_SisLib_803A5CC4((*ptr2).texts[i as usize]);
            (*data).texts[i as usize] = null_mut();
        }
        i += 1;
    }
    jobj17 = (*data).jobjs[17 as i32 as usize];
    child = HSD_JObjGetChild(jobj17);
    if !child.is_null() {
        HSD_JObjRemoveAll(HSD_JObjGetChild(jobj17));
    }
    j = 0 as i32;
    while j < 8 {
        name_idx = mnVibration_GetNameSlot(data, j) as i32;
        if name_idx != 0xff {
            mnVibration_CreateNameRow(arg0, name_idx as u8, j as u8);
        }
        j += 1;
    }
}
pub unsafe extern "C" fn mnVibration_OnAnimComplete(mut gobj: *mut HSD_GObj) {
    let mut data: *mut MnVibrationData = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut table: *mut AnimLoopSettings = null_mut();
    let mut frame: f32 = 0.;
    let mut pad: i32 = 0;
    data = (*gobj).user_data as *mut MnVibrationData;
    jobj = (*data).jobjs[1 as i32 as usize];
    table = &raw mut mnVibration_803EECEC;
    pad = 0;
    frame = mn_8022ED6C(jobj, table) as f32;
    if frame >= (*table).end_frame {
        HSD_GObjFree(gobj);
    }
}
unsafe extern "C" fn mnVibration_GetPortChildAt(
    mut gobj: *mut HSD_GObj,
    mut n: i32,
) -> *mut HSD_JObj {
    let mut child: *mut HSD_JObj = null_mut();
    let mut i: i32 = 0;
    child = HSD_JObjGetChild(
        (*((*gobj).user_data as *mut MnVibrationData))
            .jobjs[23 as i32 as usize],
    );
    i = 0 as i32;
    while i < n {
        child = HSD_JObjGetNext(child);
        i += 1;
    }
    return child;
}
unsafe extern "C" fn mnVibration_GetPadIndex(mut port: i32) -> u8 {
    return port as u8;
}
pub unsafe extern "C" fn mnVibration_Think(mut gobj: *mut HSD_GObj) {
    let mut data: *mut MnVibrationData = (*gobj).user_data as *mut MnVibrationData;
    let mut port_anim_jobj: *mut HSD_JObj = null_mut();
    let mut panel_jobj: *mut HSD_JObj = null_mut();
    let mut port_child: *mut HSD_JObj = null_mut();
    let mut port: i32 = 0;
    let mut pad_idx: u8 = 0;
    let mut pad_err: u8 = 0;
    let mut anim_state: f32 = 0.;
    let mut active_child: *mut HSD_JObj = null_mut();
    let mut disconnected_child: *mut HSD_JObj = null_mut();
    if mn_804A04F0.cur_menu as i32 != 0x13 {
        let mut proc_0: *mut HSD_GObjProc = null_mut();
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(
                mnVibration_OnAnimComplete,
            ),
            0,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
        return;
    }
    port = 0 as i32;
    while port < 4 {
        port_child = mnVibration_GetPortChildAt(gobj, port);
        lb_80011E24(port_child, &raw mut port_anim_jobj, &[1, -1]);
        if GetRumbleSettingOfPort(port as isize) != 0 {
            mn_8022EC18(port_anim_jobj, &raw mut mnVibration_803EECF8, MOBJ_MASK);
        } else {
            HSD_JObjReqAnimAll(port_anim_jobj, 0.0_f32);
            mn_8022F3D8(port_anim_jobj, 0xff, MOBJ_MASK);
            HSD_JObjAnimAll(port_anim_jobj);
        }
        port += 1;
    }
    port = 0 as i32;
    while port < 4 {
        pad_idx = mnVibration_GetPadIndex(port);
        pad_err = HSD_PadCopyStatus[pad_idx as usize].err as u8;
        if pad_err as i8 as i32 != 0
            && (*data).x6[port as usize] as i32 != 0
            || pad_err as i8 as i32 == 0
                && (*data).x6[port as usize] as i32
                    == 0
        {
            if pad_err as i8 as i32 != 0 {
                HSD_JObjSetFlagsAll(
                    (*data).jobjs[mnVibration_PortPanelJointIds[port as usize] as usize],
                    JOBJ_HIDDEN as u32,
                );
                disconnected_child = mnVibration_GetPortChildAt(gobj, port);
                mnVibration_UpdatePortPanel(disconnected_child, port as u8, 0);
                (*data).x6[port as usize] = 0;
            } else {
                let mut toggle_jobj: *mut HSD_JObj = null_mut();
                panel_jobj = (*data)
                    .jobjs[mnVibration_PortPanelJointIds[port as usize] as usize];
                HSD_JObjClearFlagsAll(panel_jobj, JOBJ_HIDDEN as u32);
                (*data)
                    .x0[(port as i32 + 2 as i32)
                    as usize] = 0;
                anim_state = (*data)
                    .x0[(port as i32 + 2 as i32) as usize]
                    as f32;
                toggle_jobj = (*((*mnVibration_804D6C28).user_data
                    as *mut MnVibrationData))
                    .jobjs[mnVibration_PortPanelJointIds[pad_idx as usize] as usize];
                HSD_JObjReqAnimAll(toggle_jobj, anim_state);
                HSD_JObjAnimAll(toggle_jobj);
                active_child = mnVibration_GetPortChildAt(gobj, port);
                mnVibration_UpdatePortPanel(active_child, port as u8, 1);
                (*data).x6[port as usize] = 1;
            }
        }
        port += 1;
    }
}
pub unsafe extern "C" fn mnVibration_IntroProc(mut arg0: *mut HSD_GObj) {
    let mut data: *mut MnVibrationData = null_mut();
    let mut frame: f32 = 0.;
    let mut assets: *mut MnVibrationJointAssets = core::ptr::null_mut::<
        MnVibrationJointAssets,
    >();
    let mut data2: *mut MnVibrationData = null_mut();
    let mut cursor_gobj: *mut HSD_GObj = null_mut();
    let mut dirty_jobj: *mut HSD_JObj = null_mut();
    let mut data3: *mut MnVibrationData = null_mut();
    let mut cursor_jobj: *mut HSD_JObj = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut jobj2: *mut HSD_JObj = null_mut();
    let mut jobj3: *mut HSD_JObj = null_mut();
    let mut jobj4: *mut HSD_JObj = null_mut();
    let mut jobj17: *mut HSD_JObj = null_mut();
    let mut jobj18: *mut HSD_JObj = null_mut();
    let mut loaded_joint: *mut HSD_JObj = null_mut();
    let mut base_y: f32 = 0.;
    let mut spacing: f32 = 0.;
    let mut temp_x: f32 = 0.;
    let mut temp_z: f32 = 0.;
    let mut cursor_row: u8 = 0;
    data = (*arg0).user_data as *mut MnVibrationData;
    let mut temp_jobj: *mut HSD_JObj = (*data).jobjs[1 as i32 as usize];
    frame = mn_8022ED6C(temp_jobj, &raw mut mnVibration_803EECE0) as f32;
    if mnVibration_804DC050 == frame {
        jobj = (*((*arg0).user_data as *mut MnVibrationData))
            .jobjs[23 as i32 as usize];
        if jobj.is_null() {
            jobj = null_mut();
        } else {
            jobj = (*jobj).child as *mut HSD_JObj;
        }
        let mut i: i32 = 0;
        while i < 0 {
            if jobj.is_null() {
                jobj = null_mut();
            } else {
                jobj = (*jobj).next as *mut HSD_JObj;
            }
            i += 1;
        }
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        if (*data).x6[0 as i32 as usize] as i32
            != 0
        {
            HSD_JObjClearFlagsAll(
                (*data).jobjs[22 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        }
    } else if mnVibration_804DC054 == frame {
        jobj = (*((*arg0).user_data as *mut MnVibrationData))
            .jobjs[23 as i32 as usize];
        if jobj.is_null() {
            jobj = null_mut();
        } else {
            jobj = (*jobj).child as *mut HSD_JObj;
        }
        if jobj.is_null() {
            jobj = null_mut();
        } else {
            jobj = (*jobj).next as *mut HSD_JObj;
        }
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        if (*data).x6[1 as i32 as usize] as i32
            != 0
        {
            HSD_JObjClearFlagsAll(
                (*data).jobjs[21 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        }
    } else if mnVibration_804DC058 == frame {
        jobj = (*((*arg0).user_data as *mut MnVibrationData))
            .jobjs[23 as i32 as usize];
        if jobj.is_null() {
            jobj2 = null_mut();
        } else {
            jobj2 = (*jobj).child as *mut HSD_JObj;
        }
        if jobj2.is_null() {
            jobj3 = null_mut();
        } else {
            jobj3 = (*jobj2).next as *mut HSD_JObj;
        }
        if jobj3.is_null() {
            jobj4 = null_mut();
        } else {
            jobj4 = (*jobj3).next as *mut HSD_JObj;
        }
        HSD_JObjClearFlagsAll(jobj4, JOBJ_HIDDEN as u32);
        if (*data).x6[2 as i32 as usize] as i32
            != 0
        {
            HSD_JObjClearFlagsAll(
                (*data).jobjs[20 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        }
    } else if mnVibration_804DC05C == frame {
        jobj = (*((*arg0).user_data as *mut MnVibrationData))
            .jobjs[23 as i32 as usize];
        if jobj.is_null() {
            jobj2 = null_mut();
        } else {
            jobj2 = (*jobj).child as *mut HSD_JObj;
        }
        if jobj2.is_null() {
            jobj3 = null_mut();
        } else {
            jobj3 = (*jobj2).next as *mut HSD_JObj;
        }
        if jobj3.is_null() {
            jobj4 = null_mut();
        } else {
            jobj4 = (*jobj3).next as *mut HSD_JObj;
        }
        if jobj4.is_null() {
            jobj = null_mut();
        } else {
            jobj = (*jobj4).next as *mut HSD_JObj;
        }
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        if (*data).x6[3 as i32 as usize] as i32
            != 0
        {
            HSD_JObjClearFlagsAll(
                (*data).jobjs[19 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        }
    } else if mnVibration_804DC060 == frame {
        if GetNameCount() != 0 {
            mnVibration_RefreshNameRows(arg0);
            assets = (&raw mut mnVibration_PcAssets as *mut MnVibrationJointAssets)
                .offset(0 as i32 as isize) as *mut MnVibrationJointAssets;
            data2 = (*arg0).user_data as *mut MnVibrationData;
            cursor_gobj = GObj_Create(6, 7, 0x80);
            (*data2).cursor_gobj = cursor_gobj;
            loaded_joint = HSD_JObjLoadJoint((*assets).joint as *mut HSD_Joint);
            HSD_GObjObject_80390A70(
                cursor_gobj,
                HSD_GObj_JObjKind,
                loaded_joint as *mut c_void,
            );
            GObj_SetupGXLink(
                cursor_gobj,
                Some(
                    HSD_GObj_JObjCallback
                       ,
                ),
                4,
                0x80,
            );
            HSD_GObj_SetupProc(
                cursor_gobj,
                Some(
                    mnVibration_CursorThink,
                ),
                0,
            );
            data3 = (*arg0).user_data as *mut MnVibrationData;
            cursor_jobj = (*cursor_gobj).hsd_obj as *mut HSD_JObj;
            jobj17 = (*data3).jobjs[17 as i32 as usize];
            cursor_row = (*data2).x0[1 as i32 as usize];
            base_y = mnVibration_JObjGetTranslationY(jobj17);
            jobj18 = (*data3).jobjs[18 as i32 as usize];
            spacing = mnVibration_GetCursorYSpacing(base_y, jobj18);
            let mut position_jobj: *mut HSD_JObj = (*data3)
                .jobjs[17 as i32 as usize];
            temp_x = mnVibration_JObjGetTranslationX(position_jobj);
            mnVibration_JObjSetTranslateX(cursor_jobj, temp_x);
            let mut position_jobj_0: *mut HSD_JObj = (*data3)
                .jobjs[17 as i32 as usize];
            base_y = spacing;
            base_y = fmadds(
                base_y,
                cursor_row as f32,
                mnVibration_JObjGetTranslationY(position_jobj_0),
            );
            dirty_jobj = cursor_jobj;
            mnVibration_JObjSetTranslateY(cursor_jobj, base_y, dirty_jobj);
            let mut position_jobj_1: *mut HSD_JObj = (*data3)
                .jobjs[17 as i32 as usize];
            temp_z = mnVibration_JObjGetTranslationZ(position_jobj_1);
            mnVibration_JObjSetTranslateZ(cursor_jobj, temp_z);
        }
    }
    if frame >= mnVibration_803EECE0.end_frame {
        let mut proc_0: *mut HSD_GObjProc = null_mut();
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            arg0,
            Some(mnVibration_Think),
            0,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
    }
}
pub unsafe extern "C" fn mnVibration_CreateScreen(mut arg0: i32) {
    let mut assets: *mut MnVibrationJointAssets = core::ptr::null_mut::<
        MnVibrationJointAssets,
    >();
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut text: *mut HSD_Text = null_mut();
    let mut i: i32 = 0;
    let mut data: *mut MnVibrationData = null_mut();
    let mut layout: *mut MnVibrationDataLayout = &raw mut mnVibration_PcLayout;
    assets = (&raw mut mnVibration_PcAssets as *mut MnVibrationJointAssets)
        .offset(3 as i32 as isize) as *mut MnVibrationJointAssets;
    gobj = GObj_Create(6, 7, 0x80);
    mnVibration_804D6C28 = gobj;
    jobj = HSD_JObjLoadJoint((*assets).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x80,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*assets).animjoint as *mut HSD_AnimJoint,
        (*assets).matanim as *mut HSD_MatAnimJoint,
        (*assets).shapeanim as *mut HSD_ShapeAnimJoint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0_f32);
    data = HSD_MemAlloc(core::mem::size_of::<MnVibrationData>() as isize)
        as *mut MnVibrationData;
    if data.is_null() {
        OSReport((*layout).user_data_error.as_ptr());
        __assert("mnvibration.c", 0x3a7, "user_data");
    }
    (*data).x0[0 as i32 as usize] = 0;
    (*data).x0[1 as i32 as usize] = 0;
    i = 0 as i32;
    while i < 4 {
        (*data).x0[(i as i32 + 2 as i32) as usize] = 0
            as u8;
        (*data).x6[i as usize] = (if HSD_PadCopyStatus[i as u8 as usize].err
            as i32 != 0
        {
            0 as i32
        } else {
            1 as i32
        }) as u8;
        i += 1;
    }
    (*data).scroll_offset = 0;
    (*data).texts[0 as i32 as usize] = core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).texts[1 as i32 as usize] = core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).texts[2 as i32 as usize] = core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).texts[3 as i32 as usize] = core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).texts[4 as i32 as usize] = core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).texts[5 as i32 as usize] = core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).texts[6 as i32 as usize] = core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).texts[7 as i32 as usize] = core::ptr::null_mut::<
        HSD_Text,
    >();
    GObj_InitUserData(
        gobj,
        0,
        Some(HSD_Free),
        data as *mut c_void,
    );
    let mut k: i32 = 0;
    k = 0 as i32;
    while k < 0x19 {
        lb_80011E24(jobj, (&raw mut (*data).jobjs as *mut *mut HSD_JObj).offset(k as isize)
                as *mut *mut HSD_JObj, &[k, -1]);
        k += 1;
    }
    HSD_JObjSetFlagsAll(
        (*data).jobjs[22 as i32 as usize],
        JOBJ_HIDDEN as u32,
    );
    HSD_JObjSetFlagsAll(
        (*data).jobjs[21 as i32 as usize],
        JOBJ_HIDDEN as u32,
    );
    HSD_JObjSetFlagsAll(
        (*data).jobjs[20 as i32 as usize],
        JOBJ_HIDDEN as u32,
    );
    HSD_JObjSetFlagsAll(
        (*data).jobjs[19 as i32 as usize],
        JOBJ_HIDDEN as u32,
    );
    HSD_GObj_SetupProc(
        gobj,
        core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>,
            HSD_GObjEvent,
        >(Some(mnVibration_IntroProc)),
        0,
    );
    mnVibration_CreatePortPanels(gobj);
    if !(*data).title_text.is_null() {
        HSD_SisLib_803A5CC4((*data).title_text);
    }
    text = HSD_SisLib_803A5ACC(
        0,
        1,
        -9.5_f32,
        9.1_f32,
        17.0_f32,
        364.68332_f32,
        38.38772_f32,
    );
    (*data).title_text = text;
    (*text).font_size.x = 0.0521_f32 as f32;
    (*text).font_size.y = 0.0521_f32 as f32;
    HSD_SisLib_803A6368(text, 0xa7);
}
pub unsafe extern "C" fn mnVibration_Init(mut arg0: i32) {
    let mut archive: *mut HSD_Archive = null_mut();
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    let mut assets: *mut MnVibrationJointAssets = core::ptr::null_mut::<
        MnVibrationJointAssets,
    >();
    let mut strings: *mut MnVibrationDataLayout = core::ptr::null_mut::<
        MnVibrationDataLayout,
    >();
    strings = &raw mut mnVibration_PcLayout;
    assets = (&raw mut mnVibration_PcAssets as *mut MnVibrationJointAssets)
        .offset(0 as i32 as isize) as *mut MnVibrationJointAssets;
    mn_804D6BC8.cooldown = 5;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 19 as MenuKind8;
    mn_804A04F0.hovered_selection = 0;
    archive = mn_804D6BB8;
    lbArchive_LoadSections(archive, &[(&raw mut (*assets.offset(3 as i32 as isize)).joint as *mut *mut c_void, &raw mut (*strings).convi_top_joint as *mut c_char), (&raw mut (*assets.offset(3 as i32 as isize)).animjoint as *mut *mut c_void, &raw mut (*strings).convi_top_animjoint as *mut c_char), (&raw mut (*assets.offset(3 as i32 as isize)).matanim as *mut *mut c_void, &raw mut (*strings).convi_top_matanim_joint as *mut c_char), (&raw mut (*assets.offset(3 as i32 as isize)).shapeanim as *mut *mut c_void, &raw mut (*strings).convi_top_shapeanim_joint as *mut c_char), (&raw mut (*assets.offset(1 as i32 as isize)).joint as *mut *mut c_void, &raw mut (*strings).ctlvi_top_joint as *mut c_char), (&raw mut (*assets.offset(1 as i32 as isize)).animjoint as *mut *mut c_void, &raw mut (*strings).ctlvi_top_animjoint as *mut c_char), (&raw mut (*assets.offset(1 as i32 as isize)).matanim as *mut *mut c_void, &raw mut (*strings).ctlvi_top_matanim_joint as *mut c_char), (&raw mut (*assets.offset(1 as i32 as isize)).shapeanim as *mut *mut c_void, &raw mut (*strings).ctlvi_top_shapeanim_joint as *mut c_char), (&raw mut (*assets.offset(2 as i32 as isize)).joint as *mut *mut c_void, &raw mut (*strings).onoffvi_top_joint as *mut c_char), (&raw mut (*assets.offset(2 as i32 as isize)).animjoint as *mut *mut c_void, &raw mut (*strings).onoffvi_top_animjoint as *mut c_char), (&raw mut (*assets.offset(2 as i32 as isize)).matanim as *mut *mut c_void, &raw mut (*strings).onoffvi_top_matanim_joint as *mut c_char), (&raw mut (*assets.offset(2 as i32 as isize)).shapeanim as *mut *mut c_void, &raw mut (*strings).onoffvi_top_shapeanim_joint as *mut c_char), (&raw mut (*assets.offset(0 as i32 as isize)).joint as *mut *mut c_void, &raw mut (*strings).cursorvi_top_joint as *mut c_char)]);
    mnVibration_CreateScreen(arg0 as i32);
    gobj = GObj_Create(0, 1, 0x80);
    proc_0 = HSD_GObj_SetupProc(
        gobj,
        core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>,
            HSD_GObjEvent,
        >(Some(mnVibration_HandleInput)),
        0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
}
pub static mut mnVibration_804DC050: f32 = 10.0_f32;
pub static mut mnVibration_804DC054: f32 = 11.0_f32;
pub static mut mnVibration_804DC058: f32 = 12.0_f32;
pub static mut mnVibration_804DC05C: f32 = 13.0_f32;
pub static mut mnVibration_804DC060: f32 = 14.0_f32;
pub const NULL: i32 = 0;
