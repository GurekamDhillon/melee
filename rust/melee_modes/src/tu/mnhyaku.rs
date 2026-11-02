//! src/melee/mn/mnhyaku.c: native port (c2rust-derived body; callees of unported TUs are `ext` TODO hooks).
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused_unsafe, unused_mut, unused_assignments, unused_variables, unused_parens, dead_code, static_mut_refs, clippy::all)]

use melee_types::*;
#[allow(unused_imports)] use self::ext::*;
#[allow(unused_imports)] use super::mnmain::{mn_802295AC, mn_80229624, mn_80229860, mn_80229894, mn_804A04F0, mn_804D6BB8, mn_804D6BC8};

pub type u8_0 = u8;
pub type u16_0 = u16;
pub type u32_0 = u32;
pub type u64_0 = u64;
pub type f32_0 = f32;
pub type f64_0 = f64;
pub const MenuInput_Right: _MenuInput = 8;
pub const MenuInput_Left: _MenuInput = 4;
pub const MenuInput_Confirm: _MenuInput = 16;
pub const MenuInput_Back: _MenuInput = 32;
pub type _MenuInput = ::core::ffi::c_int;


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
pub const HSD_GOBJ_CLASS_ITEM: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
#[inline]
pub unsafe extern "C" fn Menu_DecrementAnimTimer() {
    melee_core::ct!("Menu_DecrementAnimTimer");
    mn_804D6BC8.cooldown = mn_804D6BC8.cooldown.wrapping_sub(1);
    mn_804D6BC8.x2 = 0 as u16_0;
    mn_804D6BC8.x4 = 0 as ::core::ffi::c_int as s32;
}
#[inline]
pub unsafe extern "C" fn Menu_GetAllInputs() -> u64_0 {
    melee_core::ct!("Menu_GetAllInputs");
    mn_804A04F0.buttons = mn_80229624(4 as u32_0) as u64_0;
    return mn_804A04F0.buttons;
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
#[inline]
pub unsafe extern "C" fn Menu_InitCenterText(mut menu: *mut Menu, mut val: s32) {
    melee_core::ct!("Menu_InitCenterText");
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
pub static mut mnHyaku_804D6C58: *mut HSD_GObj = ::core::ptr::null::<HSD_GObj>()
    as *mut HSD_GObj;
pub static mut mnHyaku_803EF668: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0.0f32,
    end_frame: 19.0f32,
    loop_frame: -0.1f32,
};
pub static mut mnHyaku_803EF674: AnimLoopSettings = AnimLoopSettings {
    start_frame: 20.0f32,
    end_frame: 29.0f32,
    loop_frame: -0.1f32,
};
pub static mut mnHyaku_803EF680: [f32_0; 6] = [
    0.0f32,
    1.0f32,
    2.0f32,
    3.0f32,
    4.0f32,
    5.0f32,
];
pub static mut mnHyaku_804A08E8: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
#[inline]
pub unsafe extern "C" fn mnHyaku_8024C68C_inline(mut jobj: *mut HSD_JObj, mut val: s32) {
    melee_core::ct!("mnHyaku_8024C68C_inline");
    let mut temp_f31: f32_0 = mn_8022F298(jobj as *mut HSD_JObj);
    HSD_JObjReqAnimAll(jobj, mnHyaku_803EF680[val as usize]);
    HSD_JObjAnimAll(jobj);
    HSD_JObjReqAnimAll(jobj, temp_f31);
    mn_8022F3D8(jobj as *mut HSD_JObj, 0xff as u8_0, MOBJ_MASK);
    HSD_JObjAnimAll(jobj);
}
pub static mut vals: [u8_0; 8] = [
    0xab as ::core::ffi::c_int as u8_0,
    0xac as ::core::ffi::c_int as u8_0,
    0xad as ::core::ffi::c_int as u8_0,
    0xae as ::core::ffi::c_int as u8_0,
    0xaf as ::core::ffi::c_int as u8_0,
    0xb0 as ::core::ffi::c_int as u8_0,
    0 as ::core::ffi::c_int as u8_0,
    0 as ::core::ffi::c_int as u8_0,
];
#[inline]
pub unsafe extern "C" fn mnHyaku_8024C68C_inline_2(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mnHyaku_8024C68C_inline_2");
    let mut menu: *mut Menu = HSD_GObjGetUserData(gobj) as *mut Menu;
    if !(*menu).text.is_null() {
        HSD_SisLib_803A5CC4((*menu).text);
    }
    Menu_InitCenterText(menu, vals[(*menu).cursor as usize] as s32);
}
pub unsafe extern "C" fn mnHyaku_8024C68C(mut arg0: *mut HSD_GObj) {
    melee_core::ct!("mnHyaku_8024C68C");
    let mut events: u64_0 = 0;
    let mut menu: *mut Menu = HSD_GObjGetUserData(mnHyaku_804D6C58) as *mut Menu;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    if mn_804D6BC8.cooldown as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        Menu_DecrementAnimTimer();
        return;
    }
    events = Menu_GetAllInputs();
    if events & MenuInput_Back as ::core::ffi::c_int as u64_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_80229894(9 as s32, 2 as u16_0, 3 as s32);
        return;
    }
    if events & MenuInput_Confirm as ::core::ffi::c_int as u64_0 != 0 {
        sfxForward();
        mn_802295AC();
        gm_801677E8();
        match (*menu).cursor as ::core::ffi::c_int {
            0 => {
                mn_80229860(GM_10MAN_VS as ::core::ffi::c_int as s8);
                return;
            }
            1 => {
                mn_80229860(GM_100MAN_VS as ::core::ffi::c_int as s8);
                return;
            }
            2 => {
                mn_80229860(GM_3MIN_VS as ::core::ffi::c_int as s8);
                return;
            }
            3 => {
                mn_80229860(GM_15MIN_VS as ::core::ffi::c_int as s8);
                return;
            }
            4 => {
                mn_80229860(GM_ENDLESS_VS as ::core::ffi::c_int as s8);
                return;
            }
            5 => {
                mn_80229860(GM_CRUEL_VS as ::core::ffi::c_int as s8);
                return;
            }
            _ => {}
        }
    } else {
        if events & MenuInput_Left as ::core::ffi::c_int as u64_0 != 0 {
            sfxMove();
            if (*menu).cursor as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                (*menu).cursor = 5 as u8_0;
            } else {
                (*menu).cursor = (*menu).cursor.wrapping_sub(1);
            }
            mnHyaku_8024C68C_inline(
                (*mnHyaku_804D6C58).hsd_obj as *mut HSD_JObj,
                (*menu).cursor as s32,
            );
            mnHyaku_8024C68C_inline_2(mnHyaku_804D6C58);
            return;
        }
        if events & MenuInput_Right as ::core::ffi::c_int as u64_0 != 0 {
            sfxMove();
            if (*menu).cursor as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                (*menu).cursor = 0 as u8_0;
            } else {
                (*menu).cursor = (*menu).cursor.wrapping_add(1);
            }
            mnHyaku_8024C68C_inline(
                (*mnHyaku_804D6C58).hsd_obj as *mut HSD_JObj,
                (*menu).cursor as s32,
            );
            mnHyaku_8024C68C_inline_2(mnHyaku_804D6C58);
        }
    };
}
pub unsafe extern "C" fn mnHyaku_8024C9F0(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mnHyaku_8024C9F0");
    let mut jobj: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    let mut ret: f32_0 = mn_8022EC18(
        jobj as *mut HSD_JObj,
        &raw mut mnHyaku_803EF674,
        MOBJ_MASK,
    ) as f32_0;
    if ret >= mnHyaku_803EF674.end_frame {
        HSD_GObjFree(gobj);
    }
}
pub unsafe extern "C" fn mnHyaku_8024CA50(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mnHyaku_8024CA50");
    let mut gobj_proc: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut menu: *mut Menu = HSD_GObjGetUserData(gobj) as *mut Menu;
    if mn_804A04F0.cur_menu as ::core::ffi::c_int != 0x21 as ::core::ffi::c_int {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        gobj_proc = HSD_GObj_SetupProc(
            gobj,
            Some(mnHyaku_8024C9F0 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8_0,
        );
        (*gobj_proc).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        HSD_SisLib_803A5CC4((*menu).text);
    }
}
pub unsafe extern "C" fn mnHyaku_8024CAC8(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mnHyaku_8024CAC8");
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut gobj_proc: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut menu: *mut Menu = ::core::ptr::null_mut::<Menu>();
    let mut res: f32_0 = 0.;
    let mut _pad: [::core::ffi::c_uchar; 12] = [0; 12];
    jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    menu = HSD_GObjGetUserData(gobj) as *mut Menu;
    if mn_804A04F0.cur_menu as ::core::ffi::c_int != 0x21 as ::core::ffi::c_int {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        gobj_proc = HSD_GObj_SetupProc(
            gobj,
            Some(mnHyaku_8024C9F0 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8_0,
        );
        (*gobj_proc).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        HSD_SisLib_803A5CC4((*menu).text);
        return;
    }
    res = mn_8022EC18(jobj as *mut HSD_JObj, &raw mut mnHyaku_803EF668, MOBJ_MASK)
        as f32_0;
    if res == mnHyaku_803EF668.end_frame {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        gobj_proc = HSD_GObj_SetupProc(
            gobj,
            Some(mnHyaku_8024CA50 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8_0,
        );
        (*gobj_proc).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
    }
}
pub unsafe extern "C" fn mnHyaku_8024CB94(mut arg0: u8_0) {
    melee_core::ct!("mnHyaku_8024CB94");
    let mut user_data: *mut Menu = ::core::ptr::null_mut::<Menu>();
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut menu: *mut Menu = ::core::ptr::null_mut::<Menu>();
    gobj = GObj_Create(HSD_GOBJ_CLASS_ITEM as u16_0, 7 as u8_0, 0x80 as u8_0);
    mnHyaku_804D6C58 = gobj;
    jobj = HSD_JObjLoadJoint(mnHyaku_804A08E8.joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut ::core::ffi::c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
        ),
        4 as u8_0,
        0x80 as u32_0,
    );
    HSD_JObjAddAnimAll(
        jobj,
        mnHyaku_804A08E8.animjoint,
        mnHyaku_804A08E8.matanim_joint,
        mnHyaku_804A08E8.shapeanim_joint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0f32);
    HSD_JObjAnimAll(jobj);
    user_data = HSD_MemAlloc(::core::mem::size_of::<Menu>() as ssize_t) as *mut Menu;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char, &[]);
        __assert(
            b"src/melee/mn/mnhyaku.c\0" as *const u8 as *const ::core::ffi::c_char,
            187 as u32_0,
            b"user_data\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*user_data).cursor = arg0;
    (*user_data).text = ::core::ptr::null_mut::<HSD_Text>();
    GObj_InitUserData(
        gobj,
        0 as u8_0,
        Some(HSD_Free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        user_data as *mut ::core::ffi::c_void,
    );
    mnHyaku_8024C68C_inline(jobj, (*user_data).cursor as s32);
    proc_0 = HSD_GObj_SetupProc(
        gobj,
        Some(mnHyaku_8024CAC8 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
    menu = (*gobj).user_data as *mut Menu;
    if !(*menu).text.is_null() {
        HSD_SisLib_803A5CC4((*menu).text);
    }
    Menu_InitCenterText(menu, vals[(*menu).cursor as usize] as s32);
}
pub unsafe extern "C" fn mnHyaku_8024CD64(mut arg0: u8_0) {
    melee_core::ct!("mnHyaku_8024CD64");
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut archive: *mut HSD_Archive = ::core::ptr::null_mut::<HSD_Archive>();
    mn_804D6BC8.cooldown = 5 as u16_0;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 33 as MenuKind8;
    mn_804A04F0.hovered_selection = 0 as u16_0;
    lbAudioAx_80026F2C(0x12 as u32_0);
    lbAudioAx_8002702C(2 as u32_0, 8 as u64_0);
    lbAudioAx_80027168();
    lbAudioAx_80027648();
    archive = mn_804D6BB8;
    lbArchive_LoadSections(archive, &raw mut mnHyaku_804A08E8 as *mut *mut ::core::ffi::c_void, &[(b"MenMainConKm_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnHyaku_804A08E8.animjoint) as usize, (b"MenMainConKm_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnHyaku_804A08E8.matanim_joint) as usize, (b"MenMainConKm_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnHyaku_804A08E8.shapeanim_joint) as usize, (b"MenMainConKm_Top_shapeanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (0 as ::core::ffi::c_int) as usize]);
    mnHyaku_8024CB94(arg0);
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
        Some(mnHyaku_8024C68C as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
    lbAudioAx_800237A8(
        0x7530 as enum_t,
        0x7f as ::core::ffi::c_int,
        0x40 as ::core::ffi::c_int,
    );
}

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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
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
    hook!(H_OSReport, OSReport, (a0: *mut ::core::ffi::c_char, va: &[usize]));
    hook!(H___assert, __assert, (a0: *const ::core::ffi::c_char, a1: u32_0, a2: *const ::core::ffi::c_char) -> !);
    hook!(H_GObj_Create, GObj_Create, (a0: u16_0, a1: u8_0, a2: u8_0) -> *mut HSD_GObj);
    pub static mut H_HSD_GObj_JObjCallback: Option<unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: ::core::ffi::c_int)  {
                match H_HSD_GObj_JObjCallback {
                    Some(f) => f(a0, a1),
                    None => { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } return; },
                }
            }
    hook!(H_HSD_GObjObject_80390A70, HSD_GObjObject_80390A70, (a0: *mut HSD_GObj, a1: u8_0, a2: *mut ::core::ffi::c_void));
    hook!(H_HSD_GObj_SetupProc, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8_0) -> *mut HSD_GObjProc);
    hook!(H_HSD_GObjProc_RemoveProc, HSD_GObjProc_RemoveProc, (a0: *mut HSD_GObjProc));
    pub static mut H_lbAudioAx_800237A8: Option<unsafe extern "C" fn(enum_t, ::core::ffi::c_int, ::core::ffi::c_int) -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn lbAudioAx_800237A8(a0: enum_t, a1: ::core::ffi::c_int, a2: ::core::ffi::c_int) -> ::core::ffi::c_int {
                match H_lbAudioAx_800237A8 {
                    Some(f) => f(a0, a1, a2),
                    None => { return unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0 as _, a1 as _, a2 as _) as _ }; },
                }
            }
    pub static mut H_lbAudioAx_80024030: Option<unsafe extern "C" fn(::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn lbAudioAx_80024030(a0: ::core::ffi::c_int)  {
                match H_lbAudioAx_80024030 {
                    Some(f) => f(a0),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0 as _); } return; },
                }
            }
    pub static mut H_lbAudioAx_80026F2C: Option<unsafe extern "C" fn(u32_0) > = None;
            pub unsafe extern "C" fn lbAudioAx_80026F2C(a0: u32_0)  {
                match H_lbAudioAx_80026F2C {
                    Some(f) => f(a0),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(a0 as _); } return; },
                }
            }
    pub static mut H_lbAudioAx_8002702C: Option<unsafe extern "C" fn(u32_0, u64_0) > = None;
            pub unsafe extern "C" fn lbAudioAx_8002702C(a0: u32_0, a1: u64_0)  {
                match H_lbAudioAx_8002702C {
                    Some(f) => f(a0, a1),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_lbAudioAx_80027168: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn lbAudioAx_80027168()  {
                match H_lbAudioAx_80027168 {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027168(); } return; },
                }
            }
    pub static mut H_lbAudioAx_80027648: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn lbAudioAx_80027648()  {
                match H_lbAudioAx_80027648 {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027648(); } return; },
                }
            }
    pub static mut H_mn_8022EC18: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut AnimLoopSettings, HSD_TypeMask) -> ::core::ffi::c_float> = None;
            pub unsafe extern "C" fn mn_8022EC18(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings, a2: HSD_TypeMask) -> ::core::ffi::c_float {
                match H_mn_8022EC18 {
                    Some(f) => f(a0, a1, a2),
                    None => { return unsafe { crate::tu::mn_22EC::mn_8022EC18(a0 as _, a1 as _, a2 as _) as _ }; },
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
                    None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A6368(a0 as _, a1 as _); } return; },
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
    hook!(H_lbArchive_LoadSections, lbArchive_LoadSections, (a0: *mut HSD_Archive, a1: *mut *mut ::core::ffi::c_void, va: &[usize]));
    hook!(H_GObj_SetupGXLink, GObj_SetupGXLink, (a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8_0, a3: u32_0));
    hook!(H_HSD_GObjFree, HSD_GObjFree, (a0: *mut HSD_GObj));
    hook!(H_GObj_InitUserData, GObj_InitUserData, (a0: *mut HSD_GObj, a1: u8_0, a2: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>, a3: *mut ::core::ffi::c_void));
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
    hook!(H_gm_801677E8, gm_801677E8, ());
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CurrentInvokedProc;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_804D783C;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
}
