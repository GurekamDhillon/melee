//! src/melee/mn/mnsnap.c: the Snap (photo) menu of the data screen: the memory-card photo browser
//! with its 4-up thumbnail page, slot cards, copy / delete / view dialogs and the card
//! save-load state machine.
//! Callees in TUs that are not ported yet are `ext` stubs named like the C function.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_mut, unused_variables, unused_assignments, clippy::missing_safety_doc)]

use crate::mn_inl::*;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

/// `mnSnap_State` (mnsnap.c): the Snap menu's state (0x180 bytes on the GameCube).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct mnSnap_State {
    pub frame_count: i32,
    pub timer: i32,
    pub main_gobj: *mut HSD_GObj,
    pub cursor_gobj: *mut HSD_GObj,
    pub sub_gobj: *mut HSD_GObj,
    pub warn_gobj: *mut HSD_GObj,
    pub main_joint: *mut c_void,
    pub main_animjoint: *mut c_void,
    pub main_matanim: *mut c_void,
    pub main_shapeanim: *mut c_void,
    pub csr_joint: *mut c_void,
    pub csr_animjoint: *mut c_void,
    pub csr_matanim: *mut c_void,
    pub csr_shapeanim: *mut c_void,
    pub photo_joint: *mut c_void,
    pub sub_animjoint: *mut c_void,
    pub sub_matanim: *mut c_void,
    pub sub_shapeanim: *mut c_void,
    pub page_joint: *mut c_void,
    pub load_joint: *mut c_void,
    pub load_animjoint: *mut c_void,
    pub load_matanim: *mut c_void,
    pub arrows_joint: *mut c_void,
    pub arrows_animjoint: *mut c_void,
    pub arrows_matanim: *mut c_void,
    pub arrows_shapeanim: *mut c_void,
    pub warn_joint: *mut c_void,
    pub warn_animjoint: *mut c_void,
    pub warn_matanim: *mut c_void,
    pub warn_shapeanim: *mut c_void,
    pub thumb_jobjs: [*mut HSD_JObj; 4],
    pub select_jobj: *mut HSD_JObj,
    pub move_jobj: *mut HSD_JObj,
    pub arrow_jobj: *mut HSD_JObj,
    pub scroll_jobj: *mut HSD_JObj,
    pub slot_jobjs: [*mut HSD_JObj; 4],
    pub thumb_root: *mut HSD_JObj,
    pub submenu_jobj: *mut HSD_JObj,
    pub thumb_start: *mut HSD_JObj,
    pub thumb_end: *mut HSD_JObj,
    pub option_jobjs: [*mut HSD_JObj; 5],
    pub fullview_jobj: *mut HSD_JObj,
    pub dlg_root: *mut HSD_JObj,
    pub dlg_pos: *mut HSD_JObj,
    pub dlg_x36: *mut HSD_JObj,
    pub dlg_x37: *mut HSD_JObj,
    pub yes_jobj: *mut HSD_JObj,
    pub no_jobj: *mut HSD_JObj,
    pub dlg_frame: *mut HSD_JObj,
    pub dlg_btn_l: *mut HSD_JObj,
    pub dlg_btn_r: *mut HSD_JObj,
    pub progress_jobj: *mut HSD_JObj,
    pub thumb_labels: [*mut HSD_Text; 4],
    pub count_texts: [*mut HSD_Text; 2],
    pub page_text: *mut HSD_Text,
    pub total_text: *mut HSD_Text,
    pub dlg_text: *mut HSD_Text,
    pub state: i32,
    pub photo_count: [i32; 2],
    pub card_status: [i16; 2],
    pub cursor_idx: i32,
    pub move_idx: i32,
    pub menu_sel: i32,
    pub x4E: i32,
    pub cur_page: i32,
    pub active_slot: i32,
    pub pending_loads: i32,
    pub load_idx: i32,
    pub dlg_active: i32,
    pub dlg_timer: i32,
    pub dlg_type: i32,
    pub dlg_result: i32,
    pub card_result: i32,
    pub thumb_loaded: [i32; 4],
    pub left_btn: *mut HSD_JObj,
    pub right_btn: *mut HSD_JObj,
    pub btn_idx: i32,
    pub blank_img: *mut c_void,
}

melee_core::gc_layout!(mnSnap_State, "mnSnap_State", size 0x180, align 4, [
    frame_count @ 0x0: 4 "s32",
    timer @ 0x4: 4 "s32",
    main_gobj @ 0x8: 4 "HSD_GObj*",
    cursor_gobj @ 0xC: 4 "HSD_GObj*",
    sub_gobj @ 0x10: 4 "HSD_GObj*",
    warn_gobj @ 0x14: 4 "HSD_GObj*",
    main_joint @ 0x18: 4 "void*",
    main_animjoint @ 0x1C: 4 "void*",
    main_matanim @ 0x20: 4 "void*",
    main_shapeanim @ 0x24: 4 "void*",
    csr_joint @ 0x28: 4 "void*",
    csr_animjoint @ 0x2C: 4 "void*",
    csr_matanim @ 0x30: 4 "void*",
    csr_shapeanim @ 0x34: 4 "void*",
    photo_joint @ 0x38: 4 "void*",
    sub_animjoint @ 0x3C: 4 "void*",
    sub_matanim @ 0x40: 4 "void*",
    sub_shapeanim @ 0x44: 4 "void*",
    page_joint @ 0x48: 4 "void*",
    load_joint @ 0x4C: 4 "void*",
    load_animjoint @ 0x50: 4 "void*",
    load_matanim @ 0x54: 4 "void*",
    arrows_joint @ 0x58: 4 "void*",
    arrows_animjoint @ 0x5C: 4 "void*",
    arrows_matanim @ 0x60: 4 "void*",
    arrows_shapeanim @ 0x64: 4 "void*",
    warn_joint @ 0x68: 4 "void*",
    warn_animjoint @ 0x6C: 4 "void*",
    warn_matanim @ 0x70: 4 "void*",
    warn_shapeanim @ 0x74: 4 "void*",
    thumb_jobjs @ 0x78: 16 "HSD_JObj*[4]",
    select_jobj @ 0x88: 4 "HSD_JObj*",
    move_jobj @ 0x8C: 4 "HSD_JObj*",
    arrow_jobj @ 0x90: 4 "HSD_JObj*",
    scroll_jobj @ 0x94: 4 "HSD_JObj*",
    slot_jobjs @ 0x98: 16 "HSD_JObj*[4]",
    thumb_root @ 0xA8: 4 "HSD_JObj*",
    submenu_jobj @ 0xAC: 4 "HSD_JObj*",
    thumb_start @ 0xB0: 4 "HSD_JObj*",
    thumb_end @ 0xB4: 4 "HSD_JObj*",
    option_jobjs @ 0xB8: 20 "HSD_JObj*[5]",
    fullview_jobj @ 0xCC: 4 "HSD_JObj*",
    dlg_root @ 0xD0: 4 "HSD_JObj*",
    dlg_pos @ 0xD4: 4 "HSD_JObj*",
    dlg_x36 @ 0xD8: 4 "HSD_JObj*",
    dlg_x37 @ 0xDC: 4 "HSD_JObj*",
    yes_jobj @ 0xE0: 4 "HSD_JObj*",
    no_jobj @ 0xE4: 4 "HSD_JObj*",
    dlg_frame @ 0xE8: 4 "HSD_JObj*",
    dlg_btn_l @ 0xEC: 4 "HSD_JObj*",
    dlg_btn_r @ 0xF0: 4 "HSD_JObj*",
    progress_jobj @ 0xF4: 4 "HSD_JObj*",
    thumb_labels @ 0xF8: 16 "HSD_Text*[4]",
    count_texts @ 0x108: 8 "HSD_Text*[2]",
    page_text @ 0x110: 4 "HSD_Text*",
    total_text @ 0x114: 4 "HSD_Text*",
    dlg_text @ 0x118: 4 "HSD_Text*",
    state @ 0x11C: 4 "s32",
    photo_count @ 0x120: 8 "s32[2]",
    card_status @ 0x128: 4 "s16[2]",
    cursor_idx @ 0x12C: 4 "s32",
    move_idx @ 0x130: 4 "s32",
    menu_sel @ 0x134: 4 "s32",
    x4E @ 0x138: 4 "s32",
    cur_page @ 0x13C: 4 "s32",
    active_slot @ 0x140: 4 "s32",
    pending_loads @ 0x144: 4 "s32",
    load_idx @ 0x148: 4 "s32",
    dlg_active @ 0x14C: 4 "s32",
    dlg_timer @ 0x150: 4 "s32",
    dlg_type @ 0x154: 4 "s32",
    dlg_result @ 0x158: 4 "s32",
    card_result @ 0x15C: 4 "s32",
    thumb_loaded @ 0x160: 16 "s32[4]",
    left_btn @ 0x170: 4 "HSD_JObj*",
    right_btn @ 0x174: 4 "HSD_JObj*",
    btn_idx @ 0x178: 4 "s32",
    blank_img @ 0x17C: 4 "void*",
]);

/// `static mnSnap_State mnSnap_804A0A10`.
static mut mnSnap_804A0A10: mnSnap_State = unsafe { core::mem::zeroed() };


/// The anonymous `struct { s32 index; } cursor` local of `fn_802545C4`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct fn_802545C4_cursor {
    pub index: i32,
}

melee_core::gc_layout!(fn_802545C4_cursor, "fn_802545C4_cursor", size 4, align 4, [
    index @ 0x0: 4 "s32",
]);

pub mod ext {
    use super::*;
    use crate::ext_fns;
    ext_fns! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32);
        fn mn_80229894(a0: i32, a1: u16, a2: i32);
        fn HSD_AObjSetFlags(aobj: *mut HSD_AObj, flags: u32);
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
        fn HSD_JObjSetFlags(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjClearFlags(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjAddChild(jobj: *mut HSD_JObj, child: *mut HSD_JObj);
        fn HSD_JObjReqAnim(jobj: *mut HSD_JObj, frame: f32);
        fn HSD_JObjAnim(jobj: *mut HSD_JObj);
        fn HSD_SisLib_803A7664(a0: *mut HSD_Text);
        fn HSD_SisLib_803A6754(arg0: i32, arg1: i32) -> *mut HSD_Text;
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text);
        fn memzero(mem: *mut c_void, size: isize);
        fn lbCardNew_CompleteNextTask() -> enum_t;
        fn lb_8001CDB4();
        fn lbLang_IsSavedLanguageJP() -> i32;
        fn lbSnap_8001D338(a0: i32) -> i32;
        fn lbSnap_8001D350(i: i32) -> i32;
        fn lbSnap_8001D394(a0: i32) -> i32;
        fn lbSnap_8001D3B0(a0: i32) -> i32;
        fn lbSnap_8001D3CC(a0: i32) -> i32;
        fn lbSnap_8001D3E8(a0: i32, a1: i32) -> i32;
        fn lbSnap_8001D40C(a0: i32) -> i32;
        fn lbSnap_8001D5FC(a0: i32, a1: i32) -> i32;
        fn lbSnap_8001D7B0(a0: i32, a1: i32, a2: i32) -> i32;
        fn lbSnap_8001DE8C(a0: *mut c_void) -> i32;
        fn lbSnap_8001DF6C(a0: i32) -> i32;
        fn lbSnap_8001E058(a0: i32, a1: i32) -> i32;
        fn HSD_DObjSetFlags(dobj: *mut HSD_DObj, flags: u32);
        fn HSD_DObjClearFlags(dobj: *mut HSD_DObj, flags: u32);
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32);
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void);
        fn HSD_GObjFree(a0: *mut HSD_GObj);
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
    }
    pub static mut HSD_GObj_804D783C: i32 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
}
use ext::*;

pub static mut mnSnap_804A0B90: [*mut mnSnap_804A0B90_t; 4] = [core::ptr::null::<
    mnSnap_804A0B90_t,
>() as *mut mnSnap_804A0B90_t; 4];
pub unsafe extern "C" fn mnSnap_80253184() {
    let mut snap: *mut mnSnap_State = &raw mut mnSnap_804A0A10;
    let mut p57: *mut i32 = null_mut();
    let mut p52: *mut i32 = null_mut();
    let mut p51: *mut i32 = null_mut();
    let mut p4F: *mut i32 = null_mut();
    let mut p50: *mut i32 = null_mut();
    let mut p58: *mut i32 = null_mut();
    p4F = &raw mut (*snap).cur_page;
    p52 = &raw mut (*snap).load_idx;
    p50 = &raw mut (*snap).active_slot;
    p57 = &raw mut (*snap).card_result;
    *p57 = lbSnap_8001E058(*p50, *p52 + *p4F * 4 as i32) as i32;
    if *p57 == 8 {
        mnSnap_80254298();
        return;
    }
    if *p57 != 11 {
        p58 = (&raw mut (*snap).thumb_loaded as *mut i32)
            .offset(0 as i32 as isize) as *mut i32;
        *p58.offset(*p52 as isize) = 1 as i32;
        *p52 += 1;
        p51 = &raw mut (*snap).pending_loads;
        *p51 -= 1;
        if *p51 != 0 {
            *p57 = lbSnap_8001E058(*p50, *p52 + *p4F * 4 as i32) as i32;
            if *p57 == 8 {
                mnSnap_80254298();
                return;
            }
            if *p57 != 11 {
                *p58.offset(*p52 as isize) = 1 as i32;
                *p52 += 1;
                *p51 -= 1;
                if *p51 != 0 {
                    mnSnap_80253184();
                }
            }
        }
    }
}
unsafe extern "C" fn mnSnap_GetLoadIdx(mut snap: *mut mnSnap_State) -> *mut i32 {
    return &raw mut (*snap).load_idx;
}
unsafe extern "C" fn mnSnap_GetThumbImage(
    mut load_idx: *const i32,
) -> *mut c_void {
    let mut image: *mut c_void = mnSnap_804A0B90[(*load_idx
        % 4 as i32) as usize] as *mut c_void;
    return image;
}
unsafe extern "C" fn mnSnap_GetDObj(mut jobj: *mut HSD_JObj) -> *mut HSD_DObj {
    return (*jobj).u.dobj as *mut HSD_DObj;
}
unsafe extern "C" fn mnSnap_GetThumbJObj(
    mut load_idx: *const i32,
    mut snap: *mut mnSnap_State,
) -> *mut HSD_JObj {
    return (*snap).thumb_jobjs[*load_idx as usize];
}
unsafe extern "C" fn mnSnap_GetImageDesc(mut jobj: *mut HSD_JObj) -> *mut HSD_ImageDesc {
    return (*(*(*(*(*(*jobj).u.dobj).next).next).mobj).tobj).imagedesc
        as *mut HSD_ImageDesc;
}
unsafe extern "C" fn mnSnap_GetLoadPhotoIdx(
    mut snap: *mut mnSnap_State,
    mut load_idx: *const i32,
) -> i32 {
    return *load_idx + (*snap).cur_page * 4 as i32;
}
unsafe extern "C" fn mnSnap_8025329C() {
    let mut snap: *mut mnSnap_State = &raw mut mnSnap_804A0A10;
    let mut p52: *mut i32 = null_mut();
    let mut p51: *mut i32 = null_mut();
    let mut flags: *mut i32 = null_mut();
    let mut result: i32 = 0;
    result = lbCardNew_CompleteNextTask() as i32;
    if result == 11 {
        return;
    }
    if result == 0 {
        let mut jobj: *mut HSD_JObj = null_mut();
        let mut img: *mut c_void = core::ptr::null_mut::<
            c_void,
        >();
        p52 = mnSnap_GetLoadIdx(snap);
        if lbSnap_8001DE8C(
            mnSnap_804A0B90[((*snap).load_idx as i32
                % 4 as i32) as usize] as *mut c_void,
        ) == 1
        {
            img = mnSnap_GetThumbImage(p52);
            jobj = mnSnap_GetThumbJObj(p52, snap);
            if jobj.is_null() {
                __assert("src/melee/mn/mnsnap.c", 208, "jobj");
            };
            if (*jobj).u.dobj.is_null() {
                __assert("src/melee/mn/mnsnap.c", 209, "jobj->u.dobj");
            };
            if (*(*jobj).u.dobj).next.is_null() {
                __assert("src/melee/mn/mnsnap.c", 216, "jobj->u.dobj->next");
            };
            if (*(*(*jobj).u.dobj).next).next.is_null() {
                __assert("src/melee/mn/mnsnap.c", 217, "jobj->u.dobj->next->next");
            };
            if (*(*(*(*jobj).u.dobj).next).next).mobj.is_null() {
                __assert("src/melee/mn/mnsnap.c", 218, "jobj->u.dobj->next->next->mobj");
            };
            if (*(*(*(*(*jobj).u.dobj).next).next).mobj).tobj.is_null() {
                __assert("src/melee/mn/mnsnap.c", 219, "jobj->u.dobj->next->next->mobj->tobj");
            };
            if !(*(*(*(*(*(*jobj).u.dobj).next).next).mobj).tobj).imagedesc.is_null()
            {} else {
                __assert("src/melee/mn/mnsnap.c", 220, "jobj->u.dobj->next->next->mobj->tobj->imagedesc");
            };
            let ref mut fresh0 = (*mnSnap_GetImageDesc(jobj)).image_ptr;
            *fresh0 = img;
            (*(*(*(*(*(*(*jobj).u.dobj).next).next).mobj).tobj).imagedesc).width = 0x280
                as u16;
            (*(*(*(*(*(*mnSnap_GetDObj(jobj)).next).next).mobj).tobj).imagedesc)
                .height = 0x1e0;
        } else {
            flags = (&raw mut (*snap).thumb_loaded as *mut i32)
                .offset(0 as i32 as isize) as *mut i32;
            *flags.offset((*p52 % 4 as i32) as isize) = 1
                as i32;
        }
        p51 = &raw mut (*snap).pending_loads;
        *p52 += 1;
        *p51 -= 1;
        if *p51 != 0 {
            (*snap).card_result = lbSnap_8001E058(
                (*snap).active_slot as i32,
                *p52 + (*snap).cur_page as i32 * 4 as i32,
            ) as i32;
            if (*snap).card_result == 8 {
                mnSnap_80254298();
                return;
            }
            if (*snap).card_result != 11 {
                flags = (&raw mut (*snap).thumb_loaded as *mut i32)
                    .offset(0 as i32 as isize) as *mut i32;
                *flags.offset(*p52 as isize) = 1 as i32;
                *p52 += 1;
                *p51 -= 1;
                if *p51 != 0 {
                    mnSnap_80253184();
                }
            }
        }
    } else if (result as i32 - 2 as i32) as u32
        <= 1
    {
        p52 = &raw mut (*snap).load_idx;
        flags = (&raw mut (*snap).thumb_loaded as *mut i32)
            .offset(0 as i32 as isize) as *mut i32;
        *flags.offset((*p52 % 4 as i32) as isize) = 1
            as i32;
        p51 = &raw mut (*snap).pending_loads;
        *p52 += 1;
        *p51 -= 1;
        if *p51 != 0 {
            (*snap).card_result = lbSnap_8001E058(
                (*snap).active_slot as i32,
                mnSnap_GetLoadPhotoIdx(snap, p52) as i32,
            ) as i32;
            if (*snap).card_result == 8 {
                mnSnap_80254298();
                return;
            }
            if (*snap).card_result != 11 {
                *flags.offset(*p52 as isize) = 1 as i32;
                *p52 += 1;
                *p51 -= 1;
                if *p51 != 0 {
                    mnSnap_80253184();
                }
            }
        }
    } else if result == 8 {
        mnSnap_80254298();
    } else {
        mnSnap_80254298();
    };
}
unsafe extern "C" fn mnSnap_GetBlankImg(
    mut snap: *mut mnSnap_State,
) -> *mut c_void {
    return (*snap).blank_img;
}
pub unsafe extern "C" fn mnSnap_80253640(mut page: i32) {
    let mut count: i32 = 0;
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut img: *mut c_void = core::ptr::null_mut::<
        c_void,
    >();
    let mut snap: *mut mnSnap_State = null_mut();
    let mut p4F: *mut i32 = null_mut();
    let mut p52: *mut i32 = null_mut();
    let mut p50: *mut i32 = null_mut();
    let mut p58: *mut i32 = null_mut();
    let mut p51: *mut i32 = null_mut();
    let mut p48: *mut i32 = null_mut();
    let mut i: i32 = 0;
    let mut t: f32 = 0.;
    snap = &raw mut mnSnap_804A0A10;
    p48 = (&raw mut (*snap).photo_count as *mut i32)
        .offset(0 as i32 as isize) as *mut i32;
    p4F = &raw mut (*snap).cur_page;
    p50 = &raw mut (*snap).active_slot;
    *p4F = page;
    count = (*p48.offset(*p50 as isize) as i32
        - page as i32 * 4 as i32) as i32;
    if count > 4 {
        count = 4 as i32;
    }
    p51 = &raw mut (*snap).pending_loads;
    *p51 = count;
    i = 0 as i32;
    p52 = &raw mut (*snap).load_idx;
    *p52 = 0 as i32;
    p58 = (&raw mut (*snap).thumb_loaded as *mut i32)
        .offset(0 as i32 as isize) as *mut i32;
    *p58 = 0 as i32;
    (*snap).thumb_loaded[1 as i32 as usize] = 0 as i32
        as i32;
    (*snap).thumb_loaded[2 as i32 as usize] = 0 as i32
        as i32;
    (*snap).thumb_loaded[3 as i32 as usize] = 0 as i32
        as i32;
    while i < count {
        HSD_DObjClearFlags(
            (*(*(*(*snap).thumb_jobjs[i as usize]).u.dobj).next).next,
            1,
        );
        jobj = (*snap).thumb_jobjs[i as usize];
        img = mnSnap_GetBlankImg(snap);
        if jobj.is_null() {
            __assert("src/melee/mn/mnsnap.c", 334, "jobj");
        };
        if (*jobj).u.dobj.is_null() {
            __assert("src/melee/mn/mnsnap.c", 335, "jobj->u.dobj");
        };
        if (*(*jobj).u.dobj).next.is_null() {
            __assert("src/melee/mn/mnsnap.c", 336, "jobj->u.dobj->next");
        };
        if (*(*(*jobj).u.dobj).next).next.is_null() {
            __assert("src/melee/mn/mnsnap.c", 337, "jobj->u.dobj->next->next");
        };
        if (*(*(*(*jobj).u.dobj).next).next).mobj.is_null() {
            __assert("src/melee/mn/mnsnap.c", 338, "jobj->u.dobj->next->next->mobj");
        };
        if (*(*(*(*(*jobj).u.dobj).next).next).mobj).tobj.is_null() {
            __assert("src/melee/mn/mnsnap.c", 339, "jobj->u.dobj->next->next->mobj->tobj");
        };
        if !(*(*(*(*(*(*jobj).u.dobj).next).next).mobj).tobj).imagedesc.is_null()
        {} else {
            __assert("src/melee/mn/mnsnap.c", 340, "jobj->u.dobj->next->next->mobj->tobj->imagedesc");
        };
        i += 1;
        (*(*(*(*(*(*(*jobj).u.dobj).next).next).mobj).tobj).imagedesc).image_ptr = img;
        (*(*(*(*(*(*(*jobj).u.dobj).next).next).mobj).tobj).imagedesc).width = 0x40
            as u16;
        (*(*(*(*(*(*(*jobj).u.dobj).next).next).mobj).tobj).imagedesc).height = 0x30
            as u16;
    }
    while i < 4 {
        HSD_DObjSetFlags(
            (*(*(*(*snap).thumb_jobjs[i as usize]).u.dobj).next).next,
            1,
        );
        i += 1;
    }
    (*snap).card_result = lbSnap_8001E058(*p50, *p52 + *p4F * 4 as i32)
        as i32;
    if (*snap).card_result == 8 {
        mnSnap_80254298();
    } else if (*snap).card_result != 0xb {
        *p58.offset(*p52 as isize) = 1 as i32;
        *p52 += 1;
        *p51 -= 1;
        if *p51 != 0 {
            mnSnap_80253184();
        }
    }
    if *p50 == 0 {
        t = 0.0_f32 as f32;
    } else {
        t = 50.0_f32 as f32;
    }
    let mut scroll: *mut *mut HSD_JObj = &raw mut (*snap).scroll_jobj;
    HSD_JObjReqAnimAll(*scroll, t);
    HSD_JObjAnimAll(*scroll);
    if *p48.offset(*p50 as isize) <= 4 {
        HSD_JObjSetFlagsAll((*snap).arrow_jobj, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjClearFlagsAll((*snap).arrow_jobj, JOBJ_HIDDEN as u32);
    };
}
pub unsafe extern "C" fn mnSnap_80253964() {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut page: i32 = mnSnap_804A0A10.cur_page;
    let mut base: i32 = page * 4 as i32;
    i = 0 as i32;
    while i < 4 {
        HSD_SisLib_803A7664(mnSnap_804A0A10.thumb_labels[i as usize]);
        if mnSnap_804A0A10.state >= 4
            && base < mnSnap_804A0A10.photo_count[mnSnap_804A0A10.active_slot as usize]
        {
            HSD_SisLib_803A6B98(mnSnap_804A0A10.thumb_labels[i as usize], 0.0_f32, 0.0_f32, c"%03d".as_ptr(), &[(base as i32 + 1 as i32) as usize]);
        }
        i += 1;
        base += 1;
    }
    i = 0 as i32;
    j = 0 as i32;
    while i < 2 {
        HSD_SisLib_803A7664(mnSnap_804A0A10.count_texts[i as usize]);
        if mnSnap_804A0A10.state >= 4
            && mnSnap_804A0A10.card_status[j as usize] as i32
                == 1
        {
            HSD_SisLib_803A6B98(mnSnap_804A0A10.count_texts[i as usize], 0.0_f32, 0.0_f32, c"%d".as_ptr(), &[(mnSnap_804A0A10.photo_count[i as usize]) as usize]);
        }
        i += 1;
        j += 1;
    }
    HSD_SisLib_803A7664(mnSnap_804A0A10.page_text);
    if mnSnap_804A0A10.state >= 4 {
        HSD_SisLib_803A6B98(mnSnap_804A0A10.page_text, 0.0_f32, 0.0_f32, c"%d".as_ptr(), &[(page as i32 + 1 as i32) as usize]);
    }
    HSD_SisLib_803A7664(mnSnap_804A0A10.total_text);
    if mnSnap_804A0A10.state >= 4 {
        HSD_SisLib_803A6B98(mnSnap_804A0A10.total_text, 0.0_f32, 0.0_f32, c"%d".as_ptr(), &[((mnSnap_804A0A10.photo_count[mnSnap_804A0A10.active_slot as usize]
                + 3 as i32) / 4 as i32) as usize]);
    }
}
pub unsafe extern "C" fn mnSnap_80253AE4(mut mode: i32) {
    let mut snap: *mut mnSnap_State = &raw mut mnSnap_804A0A10;
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < 5 {
        let mut jobj: *mut HSD_JObj = (*snap).option_jobjs[i as usize];
        let mut t: f32 = 5.0_f32 * i as f32;
        if i == 4 && (*snap).active_slot == 0
        {
            t = 25.0_f32 as f32;
        }
        if i == (*snap).menu_sel {
            if mode == 0 {
                t += 1.0_f32;
            } else {
                t += 4.0_f32;
            }
        }
        HSD_JObjReqAnimAll(jobj, t);
        HSD_JObjAnimAll(jobj);
        i += 1;
    }
    if mode == 0 {
        (*snap).timer = 3 as i32;
    } else {
        (*snap).timer = 0 as i32;
    };
}
pub unsafe extern "C" fn mnSnap_80253BE0(
    mut buttons: u64,
    mut cursor: *mut i32,
    mut count: i32,
) -> i32 {
    let mut next: i32 = *cursor;
    let mut cur: i32 = next;
    if buttons & 1 as u64 != 0 {
        if next as i32 & 1 as i32
            == 1
        {
            next -= 1;
        }
    } else if buttons & 2 as u64 != 0 {
        if next as i32 & 1 as i32 == 0
            && (next as i32 + 1 as i32) < count
        {
            next += 1;
        }
    } else if buttons & 4 as u64 != 0 {
        if next >= 2 {
            next -= 2;
        }
    } else if buttons & 8 as u64 != 0 {
        if ((next as i32 & !(1 as i32))
            + 2 as i32) < count
        {
            next += 2;
            if next == count {
                next -= 1;
            }
        }
    } else if buttons & 0x40 as u64 != 0 {
        if next >= 4 {
            next -= 4;
        } else {
            next = ((count as i32 - 1 as i32
                & !(3 as i32))
                + next as i32 % 4 as i32) as i32;
            if next >= count {
                next &= !(3 as i32);
            }
        }
    } else if buttons & 0x80 as u64 != 0 {
        if ((next as i32 & !(3 as i32))
            + 4 as i32) < count
        {
            next += 4;
            if next >= count {
                next &= !(3 as i32);
            }
        } else {
            next &= 3;
        }
    }
    *cursor = next;
    if next == cur {
        return 0 as i32;
    }
    sfxMove();
    if next as i32 / 4 as i32
        == cur as i32 / 4 as i32
    {
        return 1 as i32;
    }
    return 2 as i32;
}
pub unsafe extern "C" fn fn_80253DB4(mut gobj: *mut HSD_GObj, mut rendermode: i32) {
    if mnSnap_804A0A10.state >= 4 {
        HSD_GObj_JObjCallback(gobj, rendermode as i32);
    }
}
pub unsafe extern "C" fn fn_80253DE8(mut gobj: *mut HSD_GObj, mut rendermode: i32) {
    if mnSnap_804A0A10.state == 2 {
        HSD_GObj_JObjCallback(gobj, rendermode as i32);
    }
}
pub unsafe extern "C" fn fn_80253E1C(mut gobj: *mut HSD_GObj, mut rendermode: i32) {
    let mut state: i32 = mnSnap_804A0A10.state;
    if state == 6
        || (state as i32 - 10 as i32) as u32
            <= 1
    {
        HSD_GObj_JObjCallback(gobj, rendermode as i32);
    }
}
pub unsafe extern "C" fn fn_80253E5C(mut gobj: *mut HSD_GObj, mut rendermode: i32) {
    if mnSnap_804A0A10.dlg_active == 1 {
        HSD_GObj_JObjCallback(gobj, rendermode as i32);
    }
}
pub unsafe extern "C" fn mnSnap_80253E90(mut idx: i32) {
    let mut snap: *mut mnSnap_State = &raw mut mnSnap_804A0A10;
    let mut result: i32 = lbSnap_8001D40C(idx as i32) as i32;
    if result == 0 {
        (*snap).photo_count[idx as usize] = lbSnap_8001D394(idx as i32)
            as i32;
        (*snap).card_status[idx as usize] = 1;
    } else if result == 0xf {
        (*snap).card_status[idx as usize] = 0;
    } else if result == 9 {
        (*snap).card_status[idx as usize] = -1 as i16;
    } else if result == 0xc {
        (*snap).card_status[idx as usize] = -2 as i16;
    } else {
        (*snap).card_status[idx as usize] = -3 as i16;
    };
}
pub unsafe extern "C" fn mnSnap_80253F60() {
    let mut slot_index: i32 = 0;
    let mut i: i32 = 0;
    let mut status_idx: i32 = 0;
    i = 0 as i32;
    while i < 2 {
        slot_index = (i as i32 * 2 as i32
            + 1 as i32) as i32;
        if mnSnap_804A0A10.card_status[status_idx as usize] as i32
            != 0
        {
            let mut t: f32 = 0.;
            if mnSnap_804A0A10.active_slot == i {
                t = 1.0_f32 as f32;
            } else {
                t = 0.0_f32 as f32;
            }
            HSD_JObjReqAnimAll(mnSnap_804A0A10.slot_jobjs[slot_index as usize], t);
        } else {
            HSD_JObjReqAnimAll(mnSnap_804A0A10.slot_jobjs[slot_index as usize], 2.0_f32);
        }
        HSD_JObjAnimAll(mnSnap_804A0A10.slot_jobjs[slot_index as usize]);
        status_idx += 1;
        i += 1;
    }
}
pub unsafe extern "C" fn mnSnap_80254014() {
    let mut snap: *mut mnSnap_State = &raw mut mnSnap_804A0A10;
    let mut ptr: *mut *mut HSD_JObj = core::ptr::null_mut::<*mut HSD_JObj>();
    let mut i: i32 = 0;
    ptr = &raw mut (*snap).submenu_jobj;
    HSD_JObjReqAnimAll(*ptr, 0.0_f32);
    HSD_JObjAnimAll(*ptr);
    i = 0 as i32;
    while i < 5 {
        HSD_JObjClearFlagsAll((*snap).option_jobjs[i as usize], JOBJ_HIDDEN as u32);
        i += 1;
    }
    HSD_JObjSetFlagsAll((*snap).move_jobj, JOBJ_HIDDEN as u32);
}
pub unsafe extern "C" fn mnSnap_8025409C(mut dlg_type: i32) {
    let mut left: *mut HSD_JObj = null_mut();
    let mut right: *mut HSD_JObj = null_mut();
    let mut p5E: *mut i32 = null_mut();
    let mut p38: *mut *mut HSD_JObj = core::ptr::null_mut::<*mut HSD_JObj>();
    let mut p39: *mut *mut HSD_JObj = core::ptr::null_mut::<*mut HSD_JObj>();
    mnSnap_804A0A10.dlg_type = dlg_type;
    if dlg_type == 0 {
        HSD_JObjSetFlags(mnSnap_804A0A10.yes_jobj, JOBJ_HIDDEN as u32);
        HSD_JObjSetFlags(mnSnap_804A0A10.no_jobj, JOBJ_HIDDEN as u32);
        return;
    }
    p38 = &raw mut mnSnap_804A0A10.yes_jobj;
    HSD_JObjClearFlags(*p38, JOBJ_HIDDEN as u32);
    p39 = &raw mut mnSnap_804A0A10.no_jobj;
    HSD_JObjClearFlags(*p39, JOBJ_HIDDEN as u32);
    p5E = &raw mut mnSnap_804A0A10.btn_idx;
    *p5E = 0 as i32;
    if dlg_type == 1 {
        left = *p38;
        right = *p39;
    } else if lbLang_IsSavedLanguageJP() != 0 {
        left = *p39;
        right = *p38;
    } else {
        left = *p38;
        right = *p39;
        *p5E = 1 as i32;
    }
    HSD_JObjSetTranslateX(left, -3.5_f32);
    HSD_JObjSetTranslateX(right, 3.5_f32);
    let mut f: f32 = 0.;
    if *p5E == 0 {
        f = 1.0_f32 as f32;
    } else {
        f = 0.0_f32 as f32;
    }
    HSD_JObjReqAnimAll(left, f);
    let mut f_0: f32 = 0.;
    if *p5E == 1 {
        f_0 = 1.0_f32 as f32;
    } else {
        f_0 = 0.0_f32 as f32;
    }
    HSD_JObjReqAnimAll(right, f_0);
    HSD_JObjAnimAll(left);
    HSD_JObjAnimAll(right);
    mnSnap_804A0A10.left_btn = left;
    mnSnap_804A0A10.right_btn = right;
}
unsafe extern "C" fn mnSnap_RefreshSlotSelection(
    mut snap: *mut mnSnap_State,
    mut p50: *const i32,
    mut p51: *mut i32,
) {
    mnSnap_80253964();
    mnSnap_80253E90(0 as i32);
    mnSnap_80253E90(1 as i32);
    let mut i: i32 = 0;
    let mut slot_index: i32 = 0;
    let mut status_idx: i32 = 0;
    i = 0 as i32;
    status_idx = 0 as i32;
    while i < 2 {
        slot_index = (i as i32 * 2 as i32
            + 1 as i32) as i32;
        if (*snap).card_status[status_idx as usize] as i32
            != 0
        {
            let mut t: f32 = 0.;
            if *p50 == i {
                t = 1.0_f32 as f32;
            } else {
                t = 0.0_f32 as f32;
            }
            HSD_JObjReqAnimAll((*snap).slot_jobjs[slot_index as usize], t);
        } else {
            HSD_JObjReqAnimAll((*snap).slot_jobjs[slot_index as usize], 2.0_f32);
        }
        HSD_JObjAnimAll((*snap).slot_jobjs[slot_index as usize]);
        status_idx += 1;
        i += 1;
    }
    *p51 = 0 as i32;
}
unsafe extern "C" fn mnSnap_ShowSubmenu(mut snap: *mut mnSnap_State) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < 5 {
        HSD_JObjClearFlagsAll((*snap).option_jobjs[i as usize], JOBJ_HIDDEN as u32);
        i += 1;
    }
    HSD_JObjSetFlagsAll((*snap).move_jobj, JOBJ_HIDDEN as u32);
}
unsafe extern "C" fn mnSnap_HideSubmenu(mut snap: *mut mnSnap_State) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < 5 {
        HSD_JObjSetFlagsAll((*snap).option_jobjs[i as usize], JOBJ_HIDDEN as u32);
        i += 1;
    }
    HSD_JObjClearFlagsAll((*snap).move_jobj, JOBJ_HIDDEN as u32);
}
pub unsafe extern "C" fn mnSnap_80254298() {
    let mut jobj_slot: *mut *mut HSD_JObj = core::ptr::null_mut::<*mut HSD_JObj>();
    let mut text_slot: *mut *mut HSD_Text = core::ptr::null_mut::<*mut HSD_Text>();
    let mut p50: *mut i32 = &raw mut mnSnap_804A0A10.active_slot;
    let mut p51: *mut i32 = null_mut();
    mnSnap_804A0A10.timer = 0xb as i32;
    *p50 = 0 as i32;
    jobj_slot = (&raw mut mnSnap_804A0A10.slot_jobjs as *mut *mut HSD_JObj)
        .offset(0 as i32 as isize) as *mut *mut HSD_JObj;
    HSD_JObjReqAnim(*jobj_slot, 0.0_f32);
    HSD_JObjAnim(*jobj_slot);
    jobj_slot = (&raw mut mnSnap_804A0A10.slot_jobjs as *mut *mut HSD_JObj)
        .offset(2 as i32 as isize) as *mut *mut HSD_JObj;
    HSD_JObjReqAnim(*jobj_slot, 0.0_f32);
    HSD_JObjAnim(*jobj_slot);
    p51 = &raw mut mnSnap_804A0A10.pending_loads;
    *p51 = 0 as i32;
    mnSnap_804A0A10.state = 2 as i32;
    HSD_JObjSetFlagsAll(mnSnap_804A0A10.fullview_jobj, JOBJ_HIDDEN as u32);
    mnSnap_804A0A10.dlg_active = 0 as i32;
    text_slot = &raw mut mnSnap_804A0A10.dlg_text;
    mnSnap_804A0A10.dlg_timer = 0 as i32;
    if !(*text_slot).is_null() {
        HSD_SisLib_803A5CC4(*text_slot);
        *text_slot = null_mut();
    }
    jobj_slot = &raw mut mnSnap_804A0A10.submenu_jobj;
    HSD_JObjReqAnimAll(*jobj_slot, 0.0_f32);
    HSD_JObjAnimAll(*jobj_slot);
    mnSnap_ShowSubmenu(&raw mut mnSnap_804A0A10);
    mnSnap_RefreshSlotSelection(&raw mut mnSnap_804A0A10, p50, p51);
}
unsafe extern "C" fn mnSnap_8025441C(mut buttons: u64) {
    let mut snap: *mut mnSnap_State = &raw mut mnSnap_804A0A10;
    let mut result: *mut i32 = &raw mut mnSnap_804A0A10.dlg_result;
    *result = 0 as i32;
    if buttons & 0x20 as u64 != 0 {
        *result = 2 as i32;
        return;
    }
    if buttons & 0x200 as u64 != 0 {
        if mnSnap_804A0A10.dlg_type == 0 {
            *result = 2 as i32;
        } else if *(&raw mut mnSnap_804A0A10.left_btn)
            .offset(mnSnap_804A0A10.btn_idx as isize) == mnSnap_804A0A10.no_jobj
        {
            *result = 2 as i32;
        } else {
            *result = 1 as i32;
        }
        return;
    }
    if mnSnap_804A0A10.dlg_type != 0 && buttons & 0x8 as u64 != 0
        && mnSnap_804A0A10.btn_idx == 0
    {
        sfxMove();
        HSD_JObjReqAnimAll(mnSnap_804A0A10.left_btn, 0.0_f32);
        mnSnap_804A0A10.btn_idx = 1 as i32;
        HSD_JObjAnimAll(mnSnap_804A0A10.left_btn);
        HSD_JObjAnimAll(mnSnap_804A0A10.right_btn);
        return;
    }
    if mnSnap_804A0A10.dlg_type != 0 && buttons & 0x4 as u64 != 0
        && mnSnap_804A0A10.btn_idx == 1
    {
        sfxMove();
        HSD_JObjReqAnimAll(mnSnap_804A0A10.right_btn, 0.0_f32);
        mnSnap_804A0A10.btn_idx = 0 as i32;
        HSD_JObjAnimAll((*snap).left_btn);
        HSD_JObjAnimAll(mnSnap_804A0A10.right_btn);
    }
}
unsafe extern "C" fn mnSnap_InitDialogText() {
    let mut t: *mut HSD_Text = null_mut();
    if !mnSnap_804A0A10.dlg_text.is_null() {
        HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
    }
    t = HSD_SisLib_803A5ACC(
        0,
        1,
        0.0_f32,
        -2.9_f32,
        23.0_f32,
        440.0_f32,
        10.0_f32,
    );
    mnSnap_804A0A10.dlg_text = t;
    (*mnSnap_804A0A10.dlg_text).default_alignment = 1;
    (*mnSnap_804A0A10.dlg_text).default_kerning = 1;
    t = mnSnap_804A0A10.dlg_text;
    (*t).font_size.x = 0.03_f32 as f32;
    (*t).font_size.y = 0.03_f32 as f32;
}
unsafe extern "C" fn mnSnap_GetCursorIdx() -> i32 {
    return mnSnap_804A0A10.cursor_idx;
}
unsafe extern "C" fn mnSnap_GetMoveJObj() -> *mut HSD_JObj {
    return mnSnap_804A0A10.move_jobj;
}
unsafe extern "C" fn mnSnap_GetActivePhotoCount() -> i32 {
    return mnSnap_804A0A10.photo_count[mnSnap_804A0A10.active_slot as usize];
}
unsafe extern "C" fn mnSnap_GetCurrentThumbImage(
    mut snap: *mut mnSnap_State,
) -> *mut c_void {
    return mnSnap_804A0B90[((*snap).cursor_idx as i32
        % 4 as i32) as usize] as *mut c_void;
}
unsafe extern "C" fn mnSnap_AnimateCardSlots(mut active_slot: *const i32) {
    let mut i: i32 = 0;
    let mut slot_index: i32 = 0;
    let mut t: f32 = 0.;
    let mut status_idx: i32 = 0;
    i = 0 as i32;
    while i < 2 {
        slot_index = (i as i32 * 2 as i32
            + 1 as i32) as i32;
        if mnSnap_804A0A10.card_status[status_idx as usize] as i32
            != 0
        {
            if *active_slot == i {
                t = 1.0_f32 as f32;
            } else {
                t = 0.0_f32 as f32;
            }
            HSD_JObjReqAnimAll(mnSnap_804A0A10.slot_jobjs[slot_index as usize], t);
        } else {
            HSD_JObjReqAnimAll(mnSnap_804A0A10.slot_jobjs[slot_index as usize], 2.0_f32);
        }
        HSD_JObjAnimAll(mnSnap_804A0A10.slot_jobjs[slot_index as usize]);
        status_idx += 1;
        i += 1;
    }
}
unsafe extern "C" fn mnSnap_UpdateSlotStatus(mut slot: i32) {
    if lbSnap_8001D338(slot as i32) != 0 {
        mnSnap_80253E90(slot);
    }
}
unsafe extern "C" fn mnSnap_CheckCopy(mut source_slot: i32, mut cursor: i32) -> i32 {
    let mut other_slot: i32 = 0;
    let mut result: i32 = 0;
    let mut card_status: *mut i16 = null_mut();
    card_status = &raw mut mnSnap_804A0A10.card_status as *mut i16;
    other_slot = (source_slot as i32 ^ 1 as i32) as i32;
    result = *card_status.offset(other_slot as isize) as i32;
    if result == 0 {
        result = 2 as i32;
    } else if result == -1 {
        result = 3 as i32;
    } else if result == -2 {
        result = 4 as i32;
    } else if result == -3 {
        result = 5 as i32;
    } else if lbSnap_8001D3CC(other_slot as i32)
        == 0
    {
        result = 1 as i32;
    } else if lbSnap_8001D3E8(
        source_slot as i32,
        cursor as i32,
    ) > lbSnap_8001D3B0(other_slot as i32)
    {
        result = 1 as i32;
    } else {
        result = 0 as i32;
    }
    return result;
}
unsafe extern "C" fn mnSnap_UpdateSelectionCursor(mut snap_state: *mut mnSnap_State) {
    if (*snap_state).cursor_idx as i32 / 4 as i32
        == (*snap_state).cur_page
    {
        HSD_JObjSetTranslate(
            (*snap_state).select_jobj,
            &raw mut (**(&raw mut (*snap_state).thumb_jobjs as *mut *mut HSD_JObj)
                .offset(
                    ((*snap_state).cursor_idx as i32
                        % 4 as i32) as isize,
                ))
                .translate,
        );
        HSD_JObjClearFlagsAll((*snap_state).select_jobj, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjSetFlagsAll((*snap_state).select_jobj, JOBJ_HIDDEN as u32);
    };
}
unsafe extern "C" fn mnSnap_ReadCardStatus(mut slot: i32) -> i32 {
    return mnSnap_804A0A10.card_status[slot as usize] as i32;
}
pub unsafe extern "C" fn fn_802545C4() {
    let mut cursor: fn_802545C4_cursor = fn_802545C4_cursor { index: 0 };
    let mut buttons: u64 = 0;
    let mut state: i32 = 0;
    let mut t: f32 = 0.;
    let mut i: i32 = 0;
    let mut result: i32 = 0;
    let mut slot: i32 = 0;
    let mut slot_index: i32 = 0;
    let mut status_idx: i32 = 0;
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut jobj2: *mut HSD_JObj = null_mut();
    let mut translate: *mut Vec3 = null_mut();
    mn_804A04F0.buttons = mn_80229624(4 as u32) as u64;
    buttons = mn_804A04F0.buttons;
    HSD_JObjAnimAll(mnSnap_804A0A10.select_jobj);
    jobj2 = mnSnap_804A0A10.move_jobj;
    HSD_JObjAnimAll(jobj2);
    state = mnSnap_804A0A10.state;
    if state >= 4 {
        if (state as i32 - 15 as i32) as u32 > 1
        {
            slot = mnSnap_804A0A10.active_slot;
            if lbSnap_8001D338(slot as i32) != 0 {
                mnSnap_80253E90(slot);
            }
        }
        let mut cs0: i32 = mnSnap_ReadCardStatus(mnSnap_804A0A10.active_slot);
        if cs0 != 1 {
            if mnSnap_804A0A10.state == 15
                || mnSnap_804A0A10.state == 16
            {
                while lbCardNew_CompleteNextTask() == 0xb {}
            }
            mnSnap_804A0A10.timer = 0xb as i32;
            mnSnap_804A0A10.active_slot = 0 as i32;
            HSD_JObjReqAnim(
                mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                0.0_f32,
            );
            HSD_JObjAnim(mnSnap_804A0A10.slot_jobjs[0 as i32 as usize]);
            HSD_JObjReqAnim(
                mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                0.0_f32,
            );
            HSD_JObjAnim(mnSnap_804A0A10.slot_jobjs[2 as i32 as usize]);
            mnSnap_804A0A10.pending_loads = 0 as i32;
            mnSnap_804A0A10.state = 2 as i32;
            HSD_JObjSetFlagsAll(mnSnap_804A0A10.fullview_jobj, JOBJ_HIDDEN as u32);
            mnSnap_804A0A10.dlg_active = 0 as i32;
            mnSnap_804A0A10.dlg_timer = 0 as i32;
            if !mnSnap_804A0A10.dlg_text.is_null() {
                HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                mnSnap_804A0A10.dlg_text = null_mut();
            }
            mnSnap_80254014();
            mnSnap_80253964();
            mnSnap_80253E90(0 as i32);
            mnSnap_80253E90(1 as i32);
            mnSnap_80253F60();
            mnSnap_804A0A10.pending_loads = 0 as i32;
            return;
        } else {
            let mut cs: i32 = mnSnap_804A0A10
                .card_status[(mnSnap_804A0A10.active_slot as i32
                ^ 1 as i32) as usize] as i32;
            if cs == 1 {
                if mnSnap_804A0A10.state != 15
                    && mnSnap_804A0A10.state != 16
                {
                    if cs == 1 {
                        let mut other_slot: i32 = mnSnap_804A0A10.active_slot ^ 1 as i32;
                        if lbSnap_8001D350(other_slot as i32)
                            != 0
                        {
                            mnSnap_804A0A10.card_status[other_slot as usize] = 0;
                        }
                    }
                    if mnSnap_804A0A10
                        .card_status[(mnSnap_804A0A10.active_slot as i32
                        ^ 1 as i32) as usize] as i32
                        != 1
                    {
                        mnSnap_80253964();
                    }
                }
            }
        }
    }
    if mnSnap_804A0A10.dlg_active != 0 {
        mnSnap_8025441C(buttons);
        if mnSnap_804A0A10.dlg_timer != 0 {
            mnSnap_804A0A10.dlg_timer -= 1;
            HSD_JObjReqAnim(
                mnSnap_804A0A10.dlg_frame,
                (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
            );
            HSD_JObjAnim(mnSnap_804A0A10.dlg_frame);
            HSD_JObjReqAnim(
                mnSnap_804A0A10.dlg_pos,
                (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
            );
            HSD_JObjAnim(mnSnap_804A0A10.dlg_pos);
            (*mnSnap_804A0A10.dlg_text).pos_x = (HSD_JObjGetTranslationX(
                mnSnap_804A0A10.dlg_pos,
            ) as f32 - 6.0_f32) as f32;
        }
    }
    state = mnSnap_804A0A10.state;
    let mut current_block_1097: u64;
    match state {
        0 => {
            if mnSnap_804A0A10.timer != 0 {
                mnSnap_804A0A10.timer -= 1;
                if mnSnap_804A0A10.timer == 5 {
                    mnSnap_80253E90(0 as i32);
                } else if mnSnap_804A0A10.timer == 2 {
                    mnSnap_80253E90(1 as i32);
                }
            } else {
                let mut active_slot: *mut i32 = &raw mut mnSnap_804A0A10.active_slot;
                mnSnap_804A0A10.state = 2 as i32;
                mnSnap_804A0A10.timer = 0xb as i32;
                *active_slot = 0 as i32;
                HSD_JObjReqAnim(
                    mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                    0.0_f32,
                );
                HSD_JObjAnim(
                    mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                );
                HSD_JObjReqAnim(
                    mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                    0.0_f32,
                );
                HSD_JObjAnim(
                    mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                );
                mnSnap_804A0A10.pending_loads = 0 as i32;
                if mnSnap_804A0A10.card_status[0 as i32 as usize]
                    as i32 != 0
                {
                    *active_slot = 0 as i32;
                } else if mnSnap_804A0A10.card_status[1 as i32 as usize]
                    as i32 != 0
                {
                    *active_slot = 1 as i32;
                } else {
                    *active_slot = -1 as i32;
                    mnSnap_804A0A10.dlg_timer = 9 as i32;
                    mnSnap_804A0A10.dlg_active = 1 as i32;
                    mnSnap_8025409C(0 as i32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.dlg_btn_l, JOBJ_HIDDEN as u32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.dlg_btn_r, JOBJ_HIDDEN as u32);
                    mnSnap_InitDialogText();
                    HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x140);
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.dlg_frame,
                        (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                    );
                    HSD_JObjAnim(mnSnap_804A0A10.dlg_frame);
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.dlg_pos,
                        (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                    );
                    HSD_JObjAnim(mnSnap_804A0A10.dlg_pos);
                    jobj = mnSnap_804A0A10.dlg_pos;
                    (*mnSnap_804A0A10.dlg_text).pos_x = (HSD_JObjGetTranslationX(jobj)
                        as f32 - 6.0_f32) as f32;
                    lbAudioAx_80024030(3 as i32);
                }
                mnSnap_AnimateCardSlots(active_slot);
            }
        }
        2 => {
            let mut active_slot_0: *mut i32 = null_mut();
            let mut card_status: *mut i16 = null_mut();
            result = 0 as i32;
            if mnSnap_804A0A10.dlg_timer == 0
                && mnSnap_804A0A10.timer == 0
            {
                if lbSnap_8001D338(0 as i32) != 0 {
                    mnSnap_80253E90(0 as i32);
                }
                if lbSnap_8001D338(1 as i32) != 0 {
                    mnSnap_80253E90(1 as i32);
                }
                result = 1 as i32;
            }
            if mnSnap_804A0A10.timer != 0 {
                mnSnap_804A0A10.timer -= 1;
                HSD_JObjAnim(
                    mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                );
                HSD_JObjAnim(
                    mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                );
            }
            if result != 0 {
                card_status = (&raw mut mnSnap_804A0A10.card_status as *mut i16)
                    .offset(0 as i32 as isize) as *mut i16;
                if *card_status as i32 != 0
                    || mnSnap_804A0A10.card_status[1 as i32 as usize]
                        as i32 != 0
                {
                    mnSnap_804A0A10.dlg_active = 0 as i32;
                    mnSnap_804A0A10.dlg_timer = 0 as i32;
                    if !mnSnap_804A0A10.dlg_text.is_null() {
                        HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                        mnSnap_804A0A10.dlg_text = null_mut();
                    }
                }
                active_slot_0 = &raw mut mnSnap_804A0A10.active_slot;
                if *active_slot_0 >= 0 {
                    if mnSnap_804A0A10.card_status[*active_slot_0 as usize]
                        as i32 == 0
                    {
                        let mut other_slot_0: i32 = *active_slot_0 ^ 1 as i32;
                        if mnSnap_804A0A10.card_status[other_slot_0 as usize]
                            as i32 != 0
                        {
                            *active_slot_0 = other_slot_0;
                        } else {
                            *active_slot_0 = -1 as i32;
                            mnSnap_804A0A10.dlg_timer = 9 as i32;
                            mnSnap_804A0A10.dlg_active = 1 as i32;
                            mnSnap_8025409C(0 as i32);
                            HSD_JObjSetFlags(
                                mnSnap_804A0A10.dlg_btn_l,
                                JOBJ_HIDDEN as u32,
                            );
                            HSD_JObjSetFlags(
                                mnSnap_804A0A10.dlg_btn_r,
                                JOBJ_HIDDEN as u32,
                            );
                            mnSnap_InitDialogText();
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x140);
                            HSD_JObjReqAnim(
                                mnSnap_804A0A10.dlg_frame,
                                (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                            );
                            HSD_JObjAnim(mnSnap_804A0A10.dlg_frame);
                            HSD_JObjReqAnim(
                                mnSnap_804A0A10.dlg_pos,
                                (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                            );
                            HSD_JObjAnim(mnSnap_804A0A10.dlg_pos);
                            jobj = mnSnap_804A0A10.dlg_pos;
                            (*mnSnap_804A0A10.dlg_text).pos_x = (HSD_JObjGetTranslationX(
                                jobj,
                            ) as f32 - 6.0_f32) as f32;
                            lbAudioAx_80024030(3 as i32);
                        }
                    }
                } else if *card_status as i32 != 0 {
                    *active_slot_0 = 0 as i32;
                } else if mnSnap_804A0A10.card_status[1 as i32 as usize]
                    as i32 != 0
                {
                    *active_slot_0 = 1 as i32;
                }
                status_idx = 0 as i32;
                cursor.index = 0;
                while cursor.index < 2 {
                    slot_index = (cursor.index * 2 as i32
                        + 1 as i32) as i32;
                    if mnSnap_804A0A10.card_status[status_idx as usize]
                        as i32 != 0
                    {
                        if *active_slot_0 == cursor.index {
                            t = 1.0_f32 as f32;
                        } else {
                            t = 0.0_f32 as f32;
                        }
                        HSD_JObjReqAnimAll(
                            mnSnap_804A0A10.slot_jobjs[slot_index as usize],
                            t,
                        );
                    } else {
                        HSD_JObjReqAnimAll(
                            mnSnap_804A0A10.slot_jobjs[slot_index as usize],
                            2.0_f32,
                        );
                    }
                    HSD_JObjAnimAll(mnSnap_804A0A10.slot_jobjs[slot_index as usize]);
                    status_idx += 1;
                    cursor.index += 1;
                }
                slot = *active_slot_0;
                if slot == 0
                    && mnSnap_804A0A10.card_status[1 as i32 as usize]
                        as i32 != 0
                    && buttons & 8 as u64 != 0
                {
                    sfxMove();
                    *active_slot_0 = 1 as i32;
                    mnSnap_AnimateCardSlots(active_slot_0);
                } else if 1 as i32 == slot
                    && *card_status as i32 != 0
                    && buttons & 4 as u64 != 0
                {
                    sfxMove();
                    *active_slot_0 = 0 as i32;
                    mnSnap_AnimateCardSlots(active_slot_0);
                } else if slot >= 0
                    && buttons & 0x200 as u64 != 0
                {
                    if mnSnap_804A0A10.card_status[slot as usize] as i32
                        != 1
                    {
                        mnSnap_804A0A10.state = 3 as i32;
                        lbAudioAx_80024030(3 as i32);
                        mnSnap_804A0A10.dlg_timer = 9 as i32;
                        mnSnap_804A0A10.dlg_active = 1 as i32;
                        mnSnap_8025409C(0 as i32);
                        HSD_JObjSetFlags(
                            mnSnap_804A0A10.dlg_btn_l,
                            JOBJ_HIDDEN as u32,
                        );
                        HSD_JObjSetFlags(
                            mnSnap_804A0A10.dlg_btn_r,
                            JOBJ_HIDDEN as u32,
                        );
                        mnSnap_InitDialogText();
                        if mnSnap_804A0A10.card_status[*active_slot_0 as usize]
                            as i32 == -1
                        {
                            if *active_slot_0 == 0 {
                                HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14b);
                            } else {
                                HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14c);
                            }
                        } else if mnSnap_804A0A10.card_status[*active_slot_0 as usize]
                            as i32 == -2
                        {
                            if *active_slot_0 == 0 {
                                HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14d);
                            } else {
                                HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14e);
                            }
                        } else if *active_slot_0 == 0 {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x149);
                        } else {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14a);
                        }
                        HSD_JObjReqAnim(
                            mnSnap_804A0A10.dlg_frame,
                            (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                        );
                        HSD_JObjAnim(mnSnap_804A0A10.dlg_frame);
                        HSD_JObjReqAnim(
                            mnSnap_804A0A10.dlg_pos,
                            (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                        );
                        HSD_JObjAnim(mnSnap_804A0A10.dlg_pos);
                        jobj = mnSnap_804A0A10.dlg_pos;
                        (*mnSnap_804A0A10.dlg_text).pos_x = (HSD_JObjGetTranslationX(
                            jobj,
                        ) as f32 - 6.0_f32) as f32;
                    } else if mnSnap_804A0A10.photo_count[slot as usize]
                        == 0
                    {
                        mnSnap_804A0A10.state = 3 as i32;
                        lbAudioAx_80024030(3 as i32);
                        mnSnap_804A0A10.dlg_timer = 9 as i32;
                        mnSnap_804A0A10.dlg_active = 1 as i32;
                        mnSnap_8025409C(0 as i32);
                        HSD_JObjSetFlags(
                            mnSnap_804A0A10.dlg_btn_l,
                            JOBJ_HIDDEN as u32,
                        );
                        HSD_JObjSetFlags(
                            mnSnap_804A0A10.dlg_btn_r,
                            JOBJ_HIDDEN as u32,
                        );
                        mnSnap_InitDialogText();
                        if *active_slot_0 == 0 {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x141);
                        } else {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x142);
                        }
                        HSD_JObjReqAnim(
                            mnSnap_804A0A10.dlg_frame,
                            (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                        );
                        HSD_JObjAnim(mnSnap_804A0A10.dlg_frame);
                        HSD_JObjReqAnim(
                            mnSnap_804A0A10.dlg_pos,
                            (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                        );
                        HSD_JObjAnim(mnSnap_804A0A10.dlg_pos);
                        jobj = mnSnap_804A0A10.dlg_pos;
                        (*mnSnap_804A0A10.dlg_text).pos_x = (HSD_JObjGetTranslationX(
                            jobj,
                        ) as f32 - 6.0_f32) as f32;
                        lbAudioAx_80024030(3 as i32);
                    } else {
                        sfxForward();
                        mnSnap_804A0A10.state = 4 as i32;
                        mnSnap_804A0A10.cursor_idx = 0 as i32;
                        mnSnap_80253640(0 as i32);
                        mnSnap_80253964();
                        if mnSnap_804A0A10.cursor_idx as i32
                            / 4 as i32 == mnSnap_804A0A10.cur_page
                        {
                            jobj = mnSnap_804A0A10.select_jobj;
                            translate = &raw mut (**(&raw mut mnSnap_804A0A10.thumb_jobjs
                                as *mut *mut HSD_JObj)
                                .offset(
                                    (mnSnap_804A0A10.cursor_idx as i32
                                        % 4 as i32) as isize,
                                ))
                                .translate;
                            HSD_JObjSetTranslate(jobj, translate);
                            HSD_JObjClearFlagsAll(
                                mnSnap_804A0A10.select_jobj,
                                JOBJ_HIDDEN as u32,
                            );
                        } else {
                            HSD_JObjSetFlagsAll(
                                mnSnap_804A0A10.select_jobj,
                                JOBJ_HIDDEN as u32,
                            );
                        }
                    }
                }
            }
        }
        3 => {
            mnSnap_UpdateSlotStatus(mnSnap_804A0A10.active_slot);
            if mnSnap_804A0A10.dlg_result != 0 {
                sfxBack();
            }
            if mnSnap_804A0A10.card_status[mnSnap_804A0A10.active_slot as usize]
                as i32 == 0
                || mnSnap_804A0A10.dlg_result != 0
            {
                mnSnap_804A0A10.state = 2 as i32;
                mnSnap_804A0A10.dlg_active = 0 as i32;
                mnSnap_804A0A10.dlg_timer = 0 as i32;
                if !mnSnap_804A0A10.dlg_text.is_null() {
                    HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                    mnSnap_804A0A10.dlg_text = null_mut();
                }
            }
        }
        4 => {
            if buttons & 0x200 as u64 != 0 {
                if mnSnap_804A0A10
                    .thumb_loaded[(mnSnap_804A0A10.cursor_idx as i32
                    % 4 as i32) as usize] != 0
                {
                    mnSnap_804A0A10.state = 5 as i32;
                    mnSnap_804A0A10.dlg_timer = 9 as i32;
                    mnSnap_804A0A10.dlg_active = 1 as i32;
                    mnSnap_8025409C(1 as i32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.dlg_btn_l, JOBJ_HIDDEN as u32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.dlg_btn_r, JOBJ_HIDDEN as u32);
                    mnSnap_InitDialogText();
                    HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x153);
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.dlg_frame,
                        (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                    );
                    HSD_JObjAnim(mnSnap_804A0A10.dlg_frame);
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.dlg_pos,
                        (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                    );
                    HSD_JObjAnim(mnSnap_804A0A10.dlg_pos);
                    jobj = mnSnap_804A0A10.dlg_pos;
                    (*mnSnap_804A0A10.dlg_text).pos_x = (HSD_JObjGetTranslationX(jobj)
                        as f32 - 6.0_f32) as f32;
                    lbAudioAx_80024030(3 as i32);
                } else {
                    mnSnap_804A0A10.state = 6 as i32;
                    sfxForward();
                    mnSnap_804A0A10.menu_sel = 0 as i32;
                    mnSnap_80253AE4(0 as i32);
                }
            } else if buttons & 0xcf as u64 != 0 {
                result = mnSnap_80253BE0(
                    buttons,
                    &raw mut mnSnap_804A0A10.cursor_idx,
                    mnSnap_804A0A10.photo_count[mnSnap_804A0A10.active_slot as usize],
                );
                if result == 2 {
                    mnSnap_80253640(mnSnap_804A0A10.cursor_idx / 4 as i32);
                    mnSnap_80253964();
                    if mnSnap_804A0A10.cursor_idx as i32
                        / 4 as i32 == mnSnap_804A0A10.cur_page
                    {
                        jobj = mnSnap_804A0A10.select_jobj;
                        translate = &raw mut (**(&raw mut mnSnap_804A0A10.thumb_jobjs
                            as *mut *mut HSD_JObj)
                            .offset(
                                (mnSnap_804A0A10.cursor_idx as i32
                                    % 4 as i32) as isize,
                            ))
                            .translate;
                        HSD_JObjSetTranslate(jobj, translate);
                        HSD_JObjClearFlagsAll(
                            mnSnap_804A0A10.select_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                    } else {
                        HSD_JObjSetFlagsAll(
                            mnSnap_804A0A10.select_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                    }
                } else if result == 1 {
                    if mnSnap_804A0A10.cursor_idx as i32
                        / 4 as i32 == mnSnap_804A0A10.cur_page
                    {
                        jobj = mnSnap_804A0A10.select_jobj;
                        translate = &raw mut (**(&raw mut mnSnap_804A0A10.thumb_jobjs
                            as *mut *mut HSD_JObj)
                            .offset(
                                (mnSnap_804A0A10.cursor_idx as i32
                                    % 4 as i32) as isize,
                            ))
                            .translate;
                        HSD_JObjSetTranslate(jobj, translate);
                        HSD_JObjClearFlagsAll(
                            mnSnap_804A0A10.select_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                    } else {
                        HSD_JObjSetFlagsAll(
                            mnSnap_804A0A10.select_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                    }
                }
            }
        }
        5 => {
            if mnSnap_804A0A10.dlg_result == 1 {
                let mut poll_result: i32 = 0;
                sfxForward();
                result = lbSnap_8001D5FC(
                    mnSnap_804A0A10.active_slot as i32,
                    mnSnap_804A0A10.cursor_idx as i32,
                ) as i32;
                if result != 0xb {
                    mnSnap_80254298();
                    result = 1 as i32;
                } else {
                    loop {
                        poll_result = lbCardNew_CompleteNextTask() as i32;
                        if !(poll_result == 0xb) {
                            break;
                        }
                    }
                    if poll_result != 0 {
                        mnSnap_80254298();
                        result = 1 as i32;
                    } else {
                        mnSnap_80253E90(mnSnap_804A0A10.active_slot);
                        result = 0 as i32;
                    }
                }
                if result == 0 {
                    mnSnap_804A0A10.dlg_active = 0 as i32;
                    mnSnap_804A0A10.dlg_timer = 0 as i32;
                    if !mnSnap_804A0A10.dlg_text.is_null() {
                        HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                        mnSnap_804A0A10.dlg_text = null_mut();
                    }
                    if mnSnap_804A0A10.cursor_idx
                        >= mnSnap_804A0A10
                            .photo_count[mnSnap_804A0A10.active_slot as usize]
                    {
                        mnSnap_804A0A10.cursor_idx -= 1;
                        if mnSnap_804A0A10
                            .photo_count[mnSnap_804A0A10.active_slot as usize]
                            == 0
                        {
                            mnSnap_804A0A10.timer = 0xb as i32;
                            mnSnap_804A0A10.active_slot = 0 as i32;
                            HSD_JObjReqAnim(
                                mnSnap_804A0A10
                                    .slot_jobjs[0 as i32 as usize],
                                0.0_f32,
                            );
                            HSD_JObjAnim(
                                mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                            );
                            HSD_JObjReqAnim(
                                mnSnap_804A0A10
                                    .slot_jobjs[2 as i32 as usize],
                                0.0_f32,
                            );
                            HSD_JObjAnim(
                                mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                            );
                            mnSnap_804A0A10.pending_loads = 0 as i32
                                as i32;
                            mnSnap_804A0A10.state = 2 as i32;
                            HSD_JObjSetFlagsAll(
                                mnSnap_804A0A10.fullview_jobj,
                                JOBJ_HIDDEN as u32,
                            );
                            mnSnap_804A0A10.dlg_active = 0 as i32;
                            mnSnap_804A0A10.dlg_timer = 0 as i32;
                            if !mnSnap_804A0A10.dlg_text.is_null() {
                                HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                                mnSnap_804A0A10.dlg_text = core::ptr::null_mut::<
                                    HSD_Text,
                                >();
                            }
                            mnSnap_80254014();
                            mnSnap_80253964();
                            mnSnap_80253E90(0 as i32);
                            mnSnap_80253E90(1 as i32);
                            mnSnap_80253F60();
                            mnSnap_804A0A10.pending_loads = 0 as i32
                                as i32;
                            current_block_1097 = 2033130240696481971;
                        } else {
                            current_block_1097 = 10756600493854360864;
                        }
                    } else {
                        current_block_1097 = 10756600493854360864;
                    }
                    match current_block_1097 {
                        2033130240696481971 => {}
                        _ => {
                            mnSnap_80253640(mnSnap_804A0A10.cursor_idx / 4 as i32);
                            if mnSnap_804A0A10.cursor_idx as i32
                                / 4 as i32 == mnSnap_804A0A10.cur_page
                            {
                                jobj = mnSnap_804A0A10.select_jobj;
                                translate = &raw mut (**(&raw mut mnSnap_804A0A10
                                    .thumb_jobjs as *mut *mut HSD_JObj)
                                    .offset(
                                        (mnSnap_804A0A10.cursor_idx as i32
                                            % 4 as i32) as isize,
                                    ))
                                    .translate;
                                HSD_JObjSetTranslate(jobj, translate);
                                HSD_JObjClearFlagsAll(
                                    mnSnap_804A0A10.select_jobj,
                                    JOBJ_HIDDEN as u32,
                                );
                            } else {
                                HSD_JObjSetFlagsAll(
                                    mnSnap_804A0A10.select_jobj,
                                    JOBJ_HIDDEN as u32,
                                );
                            }
                            mnSnap_80253964();
                            mnSnap_804A0A10.state = 4 as i32;
                        }
                    }
                }
            } else if mnSnap_804A0A10.dlg_result == 2 {
                sfxBack();
                mnSnap_804A0A10.dlg_active = 0 as i32;
                mnSnap_804A0A10.dlg_timer = 0 as i32;
                if !mnSnap_804A0A10.dlg_text.is_null() {
                    HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                    mnSnap_804A0A10.dlg_text = null_mut();
                }
                mnSnap_804A0A10.state = 4 as i32;
            }
        }
        6 => {
            if mnSnap_804A0A10.timer != 0 {
                HSD_JObjAnimAll(
                    mnSnap_804A0A10.option_jobjs[mnSnap_804A0A10.menu_sel as usize],
                );
                mnSnap_804A0A10.timer -= 1;
            }
            if buttons & 1 as u64 != 0 {
                sfxMove();
                mnSnap_804A0A10.menu_sel -= 1;
                if mnSnap_804A0A10.menu_sel < 0 {
                    mnSnap_804A0A10.menu_sel = 4 as i32;
                }
                mnSnap_80253AE4(0 as i32);
            } else if buttons & 2 as u64 != 0 {
                sfxMove();
                mnSnap_804A0A10.menu_sel += 1;
                if mnSnap_804A0A10.menu_sel > 4 {
                    mnSnap_804A0A10.menu_sel = 0 as i32;
                }
                mnSnap_80253AE4(0 as i32);
            } else if buttons & 0x20 as u64 != 0 {
                sfxBack();
                mnSnap_804A0A10.state = 4 as i32;
            } else if buttons & 0x200 as u64 != 0 {
                while mnSnap_804A0A10.timer != 0 {
                    HSD_JObjAnimAll(
                        mnSnap_804A0A10.option_jobjs[mnSnap_804A0A10.menu_sel as usize],
                    );
                    mnSnap_804A0A10.timer -= 1;
                }
                if mnSnap_804A0A10.menu_sel == 0 {
                    let mut thumb_img: *mut c_void = core::ptr::null_mut::<
                        c_void,
                    >();
                    sfxForward();
                    mnSnap_804A0A10.state = 0xa as i32;
                    HSD_JObjClearFlagsAll(
                        mnSnap_804A0A10.fullview_jobj,
                        JOBJ_HIDDEN as u32,
                    );
                    jobj = mnSnap_804A0A10.fullview_jobj;
                    thumb_img = mnSnap_GetCurrentThumbImage(&raw mut mnSnap_804A0A10);
                    if jobj.is_null() {
                        __assert("src/melee/mn/mnsnap.c", 1537, "jobj");
                    };
                    if (*jobj).u.dobj.is_null() {
                        __assert("src/melee/mn/mnsnap.c", 1538, "jobj->u.dobj");
                    };
                    if (*(*jobj).u.dobj).mobj.is_null() {
                        __assert("src/melee/mn/mnsnap.c", 1539, "jobj->u.dobj->mobj");
                    };
                    if (*(*(*jobj).u.dobj).mobj).tobj.is_null() {
                        __assert("src/melee/mn/mnsnap.c", 1540, "jobj->u.dobj->mobj->tobj");
                    };
                    if (*(*(*(*jobj).u.dobj).mobj).tobj).imagedesc.is_null() {
                        __assert("src/melee/mn/mnsnap.c", 1541, "jobj->u.dobj->mobj->tobj->imagedesc");
                    };
                    (*(*(*(*(*jobj).u.dobj).mobj).tobj).imagedesc).image_ptr = thumb_img;
                    (*(*(*(*(*jobj).u.dobj).mobj).tobj).imagedesc).width = 640;
                    (*(*(*(*(*jobj).u.dobj).mobj).tobj).imagedesc).height = 480;
                } else if mnSnap_804A0A10.menu_sel == 1 {
                    sfxForward();
                    mnSnap_804A0A10.state = 0xb as i32;
                    HSD_JObjAnimAll(mnSnap_804A0A10.submenu_jobj);
                    mnSnap_804A0A10.timer = 4 as i32;
                    mnSnap_HideSubmenu(&raw mut mnSnap_804A0A10);
                    mnSnap_804A0A10.move_idx = mnSnap_804A0A10.cursor_idx;
                    if mnSnap_804A0A10.move_idx as i32
                        / 4 as i32 == mnSnap_804A0A10.cur_page
                    {
                        HSD_JObjSetTranslateWithMtxDirty(
                            mnSnap_804A0A10.move_jobj,
                            &raw mut (**(&raw mut mnSnap_804A0A10.thumb_jobjs
                                as *mut *mut HSD_JObj)
                                .offset(
                                    (mnSnap_804A0A10.move_idx as i32
                                        % 4 as i32) as isize,
                                ))
                                .translate,
                        );
                        HSD_JObjClearFlagsAll(
                            mnSnap_804A0A10.move_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                    } else {
                        HSD_JObjSetFlagsAll(
                            mnSnap_804A0A10.move_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                    }
                } else if mnSnap_804A0A10.menu_sel == 2 {
                    sfxForward();
                    mnSnap_804A0A10.dlg_timer = 9 as i32;
                    mnSnap_804A0A10.dlg_active = 1 as i32;
                    mnSnap_8025409C(1 as i32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.dlg_btn_l, JOBJ_HIDDEN as u32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.dlg_btn_r, JOBJ_HIDDEN as u32);
                    mnSnap_InitDialogText();
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x144);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x143);
                    }
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.dlg_frame,
                        (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                    );
                    HSD_JObjAnim(mnSnap_804A0A10.dlg_frame);
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.dlg_pos,
                        (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                    );
                    HSD_JObjAnim(mnSnap_804A0A10.dlg_pos);
                    jobj = mnSnap_804A0A10.dlg_pos;
                    (*mnSnap_804A0A10.dlg_text).pos_x = (HSD_JObjGetTranslationX(jobj)
                        as f32 - 6.0_f32) as f32;
                    mnSnap_804A0A10.state = 0xc as i32;
                } else if mnSnap_804A0A10.menu_sel == 3 {
                    mnSnap_804A0A10.dlg_timer = 9 as i32;
                    mnSnap_804A0A10.dlg_active = 1 as i32;
                    mnSnap_8025409C(2 as i32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.dlg_btn_l, JOBJ_HIDDEN as u32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.dlg_btn_r, JOBJ_HIDDEN as u32);
                    mnSnap_InitDialogText();
                    HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x150);
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.dlg_frame,
                        (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                    );
                    HSD_JObjAnim(mnSnap_804A0A10.dlg_frame);
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.dlg_pos,
                        (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                    );
                    HSD_JObjAnim(mnSnap_804A0A10.dlg_pos);
                    jobj = mnSnap_804A0A10.dlg_pos;
                    (*mnSnap_804A0A10.dlg_text).pos_x = (HSD_JObjGetTranslationX(jobj)
                        as f32 - 6.0_f32) as f32;
                    lbAudioAx_80024030(7 as i32);
                    mnSnap_804A0A10.state = 0x14 as i32;
                } else if mnSnap_804A0A10.menu_sel == 4 {
                    sfxForward();
                    mnSnap_804A0A10.dlg_timer = 9 as i32;
                    mnSnap_804A0A10.dlg_active = 1 as i32;
                    mnSnap_8025409C(1 as i32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.dlg_btn_l, JOBJ_HIDDEN as u32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.dlg_btn_r, JOBJ_HIDDEN as u32);
                    mnSnap_InitDialogText();
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x152);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x151);
                    }
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.dlg_frame,
                        (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                    );
                    HSD_JObjAnim(mnSnap_804A0A10.dlg_frame);
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.dlg_pos,
                        (9 as i32 - mnSnap_804A0A10.dlg_timer) as f32,
                    );
                    HSD_JObjAnim(mnSnap_804A0A10.dlg_pos);
                    jobj = mnSnap_804A0A10.dlg_pos;
                    (*mnSnap_804A0A10.dlg_text).pos_x = (HSD_JObjGetTranslationX(jobj)
                        as f32 - 6.0_f32) as f32;
                    mnSnap_804A0A10.state = 0x15 as i32;
                }
            }
        }
        10 => {
            if buttons & 0x200 as u64 != 0 {
                mnSnap_804A0A10.state = 4 as i32;
                sfxBack();
                HSD_JObjSetFlagsAll(mnSnap_804A0A10.fullview_jobj, JOBJ_HIDDEN as u32);
            } else if buttons & 0x20 as u64 != 0 {
                mnSnap_804A0A10.state = 6 as i32;
                sfxBack();
                HSD_JObjSetFlagsAll(mnSnap_804A0A10.fullview_jobj, JOBJ_HIDDEN as u32);
            }
        }
        11 => {
            state = 0 as i32;
            if mnSnap_804A0A10.timer != 0 {
                HSD_JObjAnimAll(mnSnap_804A0A10.submenu_jobj);
                mnSnap_804A0A10.timer -= 1;
            }
            if buttons & 0x200 as u64 != 0 {
                if mnSnap_804A0A10.move_idx != mnSnap_804A0A10.cursor_idx {
                    sfxForward();
                    let mut mi: i32 = mnSnap_804A0A10.move_idx;
                    result = lbSnap_8001D7B0(
                        mnSnap_804A0A10.active_slot as i32,
                        mnSnap_GetCursorIdx() as i32,
                        mi as i32,
                    ) as i32;
                    if result != 8 {
                        while lbCardNew_CompleteNextTask() == 0xb {}
                        mnSnap_80253E90(mnSnap_804A0A10.active_slot);
                    }
                    mnSnap_804A0A10.cur_page = -1 as i32;
                    state = 4 as i32;
                } else {
                    lbAudioAx_80024030(3 as i32);
                }
            } else if buttons & 0x20 as u64 != 0 {
                state = 6 as i32;
                sfxBack();
            } else if buttons & 0xcf as u64 != 0 {
                result = mnSnap_80253BE0(
                    buttons,
                    &raw mut mnSnap_804A0A10.move_idx,
                    mnSnap_GetActivePhotoCount(),
                );
                if result == 2 {
                    mnSnap_80253640(mnSnap_804A0A10.move_idx / 4 as i32);
                    mnSnap_80253964();
                    mnSnap_UpdateSelectionCursor(&raw mut mnSnap_804A0A10);
                    if mnSnap_804A0A10.move_idx as i32
                        / 4 as i32 == mnSnap_804A0A10.cur_page
                    {
                        jobj = mnSnap_804A0A10.move_jobj;
                        translate = &raw mut (**(&raw mut mnSnap_804A0A10.thumb_jobjs
                            as *mut *mut HSD_JObj)
                            .offset(
                                (mnSnap_804A0A10.move_idx as i32
                                    % 4 as i32) as isize,
                            ))
                            .translate;
                        HSD_JObjSetTranslate(jobj, translate);
                        HSD_JObjClearFlagsAll(
                            mnSnap_804A0A10.move_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                    } else {
                        HSD_JObjSetFlagsAll(mnSnap_GetMoveJObj(), JOBJ_HIDDEN as u32);
                    }
                } else if result == 1 {
                    if mnSnap_804A0A10.move_idx as i32
                        / 4 as i32 == mnSnap_804A0A10.cur_page
                    {
                        jobj2 = mnSnap_804A0A10.move_jobj;
                        translate = &raw mut (**(&raw mut mnSnap_804A0A10.thumb_jobjs
                            as *mut *mut HSD_JObj)
                            .offset(
                                (mnSnap_804A0A10.move_idx as i32
                                    % 4 as i32) as isize,
                            ))
                            .translate;
                        HSD_JObjSetTranslate(jobj2, translate);
                        HSD_JObjClearFlagsAll(
                            mnSnap_804A0A10.move_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                    } else {
                        HSD_JObjSetFlagsAll(
                            mnSnap_804A0A10.move_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                    }
                }
            }
            if state != 0 {
                mnSnap_804A0A10.state = state;
                HSD_JObjReqAnimAll(mnSnap_804A0A10.submenu_jobj, 0.0_f32);
                HSD_JObjAnimAll(mnSnap_804A0A10.submenu_jobj);
                mnSnap_ShowSubmenu(&raw mut mnSnap_804A0A10);
                mnSnap_80253AE4(1 as i32);
                if mnSnap_804A0A10.cursor_idx as i32
                    / 4 as i32 != mnSnap_804A0A10.cur_page
                {
                    mnSnap_80253640(mnSnap_804A0A10.cursor_idx / 4 as i32);
                    mnSnap_80253964();
                    mnSnap_UpdateSelectionCursor(&raw mut mnSnap_804A0A10);
                }
            }
        }
        12 => {
            if mnSnap_804A0A10.dlg_result == 1 {
                let mut other_slot_1: i32 = mnSnap_804A0A10.active_slot ^ 1 as i32;
                if lbSnap_8001D338(other_slot_1 as i32)
                    != 0
                {
                    mnSnap_80253E90(other_slot_1);
                }
                result = mnSnap_CheckCopy(
                    mnSnap_804A0A10.active_slot,
                    mnSnap_804A0A10.cursor_idx,
                );
                if result == 0 {
                    sfxForward();
                    mnSnap_804A0A10.state = 0xf as i32;
                    HSD_JObjClearFlags(mnSnap_804A0A10.dlg_btn_r, JOBJ_HIDDEN as u32);
                    HSD_JObjSetTranslateX(mnSnap_804A0A10.progress_jobj, -6.9_f32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.yes_jobj, JOBJ_HIDDEN as u32);
                    HSD_JObjSetFlags(mnSnap_804A0A10.no_jobj, JOBJ_HIDDEN as u32);
                    let mut ci: i32 = mnSnap_804A0A10.cursor_idx;
                    mnSnap_804A0A10.card_result = lbSnap_8001E058(
                        mnSnap_804A0A10.active_slot as i32,
                        ci as i32,
                    ) as i32;
                    if mnSnap_804A0A10.card_result == 8 {
                        mnSnap_804A0A10.timer = 0xb as i32;
                        mnSnap_804A0A10.active_slot = 0 as i32;
                        HSD_JObjReqAnim(
                            mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                            0.0_f32,
                        );
                        HSD_JObjAnim(
                            mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                        );
                        HSD_JObjReqAnim(
                            mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                            0.0_f32,
                        );
                        HSD_JObjAnim(
                            mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                        );
                        mnSnap_804A0A10.pending_loads = 0 as i32;
                        mnSnap_804A0A10.state = 2 as i32;
                        HSD_JObjSetFlagsAll(
                            mnSnap_804A0A10.fullview_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                        mnSnap_804A0A10.dlg_active = 0 as i32;
                        mnSnap_804A0A10.dlg_timer = 0 as i32;
                        if !mnSnap_804A0A10.dlg_text.is_null() {
                            HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                            mnSnap_804A0A10.dlg_text = core::ptr::null_mut::<
                                HSD_Text,
                            >();
                        }
                        mnSnap_80254014();
                        mnSnap_80253964();
                        mnSnap_80253E90(0 as i32);
                        mnSnap_80253E90(1 as i32);
                        mnSnap_80253F60();
                        mnSnap_804A0A10.pending_loads = 0 as i32;
                    } else if mnSnap_804A0A10.card_result != 0xb {
                        mnSnap_804A0A10.timer = 0xb as i32;
                        mnSnap_804A0A10.active_slot = 0 as i32;
                        HSD_JObjReqAnim(
                            mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                            0.0_f32,
                        );
                        HSD_JObjAnim(
                            mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                        );
                        HSD_JObjReqAnim(
                            mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                            0.0_f32,
                        );
                        HSD_JObjAnim(
                            mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                        );
                        mnSnap_804A0A10.pending_loads = 0 as i32;
                        mnSnap_804A0A10.state = 2 as i32;
                        HSD_JObjSetFlagsAll(
                            mnSnap_804A0A10.fullview_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                        mnSnap_804A0A10.dlg_active = 0 as i32;
                        mnSnap_804A0A10.dlg_timer = 0 as i32;
                        if !mnSnap_804A0A10.dlg_text.is_null() {
                            HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                            mnSnap_804A0A10.dlg_text = core::ptr::null_mut::<
                                HSD_Text,
                            >();
                        }
                        mnSnap_80254014();
                        mnSnap_80253964();
                        mnSnap_80253E90(0 as i32);
                        mnSnap_80253E90(1 as i32);
                        mnSnap_80253F60();
                        mnSnap_804A0A10.pending_loads = 0 as i32;
                    }
                } else if result == 2 {
                    lbAudioAx_80024030(3 as i32);
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x148);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x147);
                    }
                    mnSnap_8025409C(0 as i32);
                    mnSnap_804A0A10.state = 0xd as i32;
                } else if result == 1 {
                    lbAudioAx_80024030(3 as i32);
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x146);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x145);
                    }
                    mnSnap_8025409C(0 as i32);
                    mnSnap_804A0A10.state = 0xe as i32;
                } else if result == 3 {
                    lbAudioAx_80024030(3 as i32);
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14c);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14b);
                    }
                    mnSnap_8025409C(0 as i32);
                    mnSnap_804A0A10.state = 0xe as i32;
                } else if result == 4 {
                    lbAudioAx_80024030(3 as i32);
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14e);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14d);
                    }
                    mnSnap_8025409C(0 as i32);
                    mnSnap_804A0A10.state = 0xe as i32;
                } else if result == 5 {
                    lbAudioAx_80024030(3 as i32);
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14a);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x149);
                    }
                    mnSnap_8025409C(0 as i32);
                    mnSnap_804A0A10.state = 0xe as i32;
                }
            } else if mnSnap_804A0A10.dlg_result == 2 {
                sfxBack();
                mnSnap_804A0A10.dlg_active = 0 as i32;
                mnSnap_804A0A10.dlg_timer = 0 as i32;
                if !mnSnap_804A0A10.dlg_text.is_null() {
                    HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                    mnSnap_804A0A10.dlg_text = null_mut();
                }
                mnSnap_804A0A10.state = 6 as i32;
            }
        }
        13 => {
            if mnSnap_804A0A10.dlg_result == 2 {
                sfxBack();
                mnSnap_804A0A10.state = 6 as i32;
                mnSnap_804A0A10.dlg_active = 0 as i32;
                mnSnap_804A0A10.dlg_timer = 0 as i32;
                if !mnSnap_804A0A10.dlg_text.is_null() {
                    HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                    mnSnap_804A0A10.dlg_text = null_mut();
                }
            } else {
                mnSnap_UpdateSlotStatus(mnSnap_804A0A10.active_slot ^ 1 as i32);
                if mnSnap_804A0A10
                    .card_status[(mnSnap_804A0A10.active_slot as i32
                    ^ 1 as i32) as usize] as i32
                    != 0
                {
                    mnSnap_804A0A10.state = 0xc as i32;
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x144);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x143);
                    }
                    mnSnap_8025409C(1 as i32);
                }
            }
        }
        14 => {
            if mnSnap_804A0A10.dlg_result != 0 {
                sfxBack();
                mnSnap_804A0A10.state = 6 as i32;
                mnSnap_804A0A10.dlg_active = 0 as i32;
                mnSnap_804A0A10.dlg_timer = 0 as i32;
                if !mnSnap_804A0A10.dlg_text.is_null() {
                    HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                    mnSnap_804A0A10.dlg_text = null_mut();
                }
            } else {
                mnSnap_UpdateSlotStatus(mnSnap_804A0A10.active_slot ^ 1 as i32);
                if mnSnap_804A0A10
                    .card_status[(mnSnap_804A0A10.active_slot as i32
                    ^ 1 as i32) as usize] as i32
                    == 0
                {
                    mnSnap_804A0A10.state = 0xc as i32;
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x144);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x143);
                    }
                    mnSnap_8025409C(1 as i32);
                }
            }
        }
        15 => {
            result = lbCardNew_CompleteNextTask() as i32;
            if result != 0xb {
                if lbCardNew_CompleteNextTask() != 0 {
                    mnSnap_804A0A10.timer = 0xb as i32;
                    mnSnap_804A0A10.active_slot = 0 as i32;
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                        0.0_f32,
                    );
                    HSD_JObjAnim(
                        mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                    );
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                        0.0_f32,
                    );
                    HSD_JObjAnim(
                        mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                    );
                    mnSnap_804A0A10.pending_loads = 0 as i32;
                    mnSnap_804A0A10.state = 2 as i32;
                    HSD_JObjSetFlagsAll(
                        mnSnap_804A0A10.fullview_jobj,
                        JOBJ_HIDDEN as u32,
                    );
                    mnSnap_804A0A10.dlg_active = 0 as i32;
                    mnSnap_804A0A10.dlg_timer = 0 as i32;
                    if !mnSnap_804A0A10.dlg_text.is_null() {
                        HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                        mnSnap_804A0A10.dlg_text = null_mut();
                    }
                    mnSnap_80254014();
                    mnSnap_80253964();
                    mnSnap_80253E90(0 as i32);
                    mnSnap_80253E90(1 as i32);
                    mnSnap_80253F60();
                    mnSnap_804A0A10.pending_loads = 0 as i32;
                } else {
                    result = lbSnap_8001DF6C(
                        mnSnap_804A0A10.active_slot as i32
                            ^ 1 as i32,
                    ) as i32;
                    if result != 0xb {
                        mnSnap_804A0A10.timer = 0xb as i32;
                        mnSnap_804A0A10.active_slot = 0 as i32;
                        HSD_JObjReqAnim(
                            mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                            0.0_f32,
                        );
                        HSD_JObjAnim(
                            mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                        );
                        HSD_JObjReqAnim(
                            mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                            0.0_f32,
                        );
                        HSD_JObjAnim(
                            mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                        );
                        mnSnap_804A0A10.pending_loads = 0 as i32;
                        mnSnap_804A0A10.state = 2 as i32;
                        HSD_JObjSetFlagsAll(
                            mnSnap_804A0A10.fullview_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                        mnSnap_804A0A10.dlg_active = 0 as i32;
                        mnSnap_804A0A10.dlg_timer = 0 as i32;
                        if !mnSnap_804A0A10.dlg_text.is_null() {
                            HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                            mnSnap_804A0A10.dlg_text = core::ptr::null_mut::<
                                HSD_Text,
                            >();
                        }
                        mnSnap_80254014();
                        mnSnap_80253964();
                        mnSnap_80253E90(0 as i32);
                        mnSnap_80253E90(1 as i32);
                        mnSnap_80253F60();
                        mnSnap_804A0A10.pending_loads = 0 as i32;
                    } else {
                        mnSnap_804A0A10.state = 0x10 as i32;
                        HSD_JObjSetTranslateX(mnSnap_804A0A10.progress_jobj, -3.0_f32);
                        mnSnap_804A0A10.timer = 0x50 as i32;
                    }
                }
            }
        }
        16 => {
            mnSnap_804A0A10.timer -= 1;
            if mnSnap_804A0A10.timer == 0 {
                let mut p_jobj: *mut HSD_JObj = null_mut();
                mnSnap_804A0A10.timer = 0x64 as i32;
                p_jobj = mnSnap_804A0A10.progress_jobj;
                t = HSD_JObjGetTranslationX(p_jobj);
                if t < 3.0_f32 {
                    t += 3.0_f32;
                    HSD_JObjSetTranslateX(p_jobj, t);
                }
            }
            result = lbCardNew_CompleteNextTask() as i32;
            if result != 0xb {
                if lbCardNew_CompleteNextTask() != 0 {
                    mnSnap_804A0A10.timer = 0xb as i32;
                    mnSnap_804A0A10.active_slot = 0 as i32;
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                        0.0_f32,
                    );
                    HSD_JObjAnim(
                        mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                    );
                    HSD_JObjReqAnim(
                        mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                        0.0_f32,
                    );
                    HSD_JObjAnim(
                        mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                    );
                    mnSnap_804A0A10.pending_loads = 0 as i32;
                    mnSnap_804A0A10.state = 2 as i32;
                    HSD_JObjSetFlagsAll(
                        mnSnap_804A0A10.fullview_jobj,
                        JOBJ_HIDDEN as u32,
                    );
                    mnSnap_804A0A10.dlg_active = 0 as i32;
                    mnSnap_804A0A10.dlg_timer = 0 as i32;
                    if !mnSnap_804A0A10.dlg_text.is_null() {
                        HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                        mnSnap_804A0A10.dlg_text = null_mut();
                    }
                    mnSnap_80254014();
                    mnSnap_80253964();
                    mnSnap_80253E90(0 as i32);
                    mnSnap_80253E90(1 as i32);
                    mnSnap_80253F60();
                    mnSnap_804A0A10.pending_loads = 0 as i32;
                } else {
                    mnSnap_80253E90(mnSnap_804A0A10.active_slot ^ 1 as i32);
                    mnSnap_80253964();
                    HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14f);
                    mnSnap_804A0A10.state = 0x11 as i32;
                    mnSnap_804A0A10.timer = 0x1e as i32;
                    HSD_JObjSetTranslateX(mnSnap_804A0A10.progress_jobj, 5.0_f32);
                }
            }
        }
        17 => {
            if mnSnap_804A0A10.timer != 0 {
                mnSnap_804A0A10.timer -= 1;
            } else {
                mnSnap_804A0A10.dlg_active = 0 as i32;
                mnSnap_804A0A10.dlg_timer = 0 as i32;
                if !mnSnap_804A0A10.dlg_text.is_null() {
                    HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                    mnSnap_804A0A10.dlg_text = null_mut();
                }
                mnSnap_804A0A10.state = 4 as i32;
            }
        }
        20 => {
            if mnSnap_804A0A10.dlg_result == 1 {
                let mut do_delete: i32 = 0;
                let mut poll_result_0: i32 = 0;
                sfxForward();
                if lbSnap_8001D5FC(
                    mnSnap_804A0A10.active_slot as i32,
                    mnSnap_804A0A10.cursor_idx as i32,
                ) != 0xb
                {
                    mnSnap_80254298();
                    do_delete = 1 as i32;
                } else {
                    loop {
                        poll_result_0 = lbCardNew_CompleteNextTask() as i32;
                        if !(poll_result_0 == 0xb) {
                            break;
                        }
                    }
                    if poll_result_0 != 0 {
                        mnSnap_80254298();
                        do_delete = 1 as i32;
                    } else {
                        mnSnap_80253E90(mnSnap_804A0A10.active_slot);
                        do_delete = 0 as i32;
                    }
                }
                if do_delete == 0 {
                    let mut idx: i32 = 0;
                    mnSnap_804A0A10.dlg_active = 0 as i32;
                    mnSnap_804A0A10.dlg_timer = 0 as i32;
                    if !mnSnap_804A0A10.dlg_text.is_null() {
                        HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                        mnSnap_804A0A10.dlg_text = null_mut();
                    }
                    idx = mnSnap_804A0A10.cursor_idx;
                    if idx
                        >= mnSnap_804A0A10
                            .photo_count[mnSnap_804A0A10.active_slot as usize]
                    {
                        mnSnap_804A0A10.cursor_idx = (idx as i32
                            - 1 as i32) as i32;
                        if mnSnap_804A0A10
                            .photo_count[mnSnap_804A0A10.active_slot as usize]
                            == 0
                        {
                            mnSnap_804A0A10.timer = 0xb as i32;
                            mnSnap_804A0A10.active_slot = 0 as i32;
                            HSD_JObjReqAnim(
                                mnSnap_804A0A10
                                    .slot_jobjs[0 as i32 as usize],
                                0.0_f32,
                            );
                            HSD_JObjAnim(
                                mnSnap_804A0A10.slot_jobjs[0 as i32 as usize],
                            );
                            HSD_JObjReqAnim(
                                mnSnap_804A0A10
                                    .slot_jobjs[2 as i32 as usize],
                                0.0_f32,
                            );
                            HSD_JObjAnim(
                                mnSnap_804A0A10.slot_jobjs[2 as i32 as usize],
                            );
                            mnSnap_804A0A10.pending_loads = 0 as i32
                                as i32;
                            mnSnap_804A0A10.state = 2 as i32;
                            HSD_JObjSetFlagsAll(
                                mnSnap_804A0A10.fullview_jobj,
                                JOBJ_HIDDEN as u32,
                            );
                            mnSnap_804A0A10.dlg_active = 0 as i32;
                            mnSnap_804A0A10.dlg_timer = 0 as i32;
                            if !mnSnap_804A0A10.dlg_text.is_null() {
                                HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                                mnSnap_804A0A10.dlg_text = core::ptr::null_mut::<
                                    HSD_Text,
                                >();
                            }
                            mnSnap_80254014();
                            mnSnap_80253964();
                            mnSnap_80253E90(0 as i32);
                            mnSnap_80253E90(1 as i32);
                            mnSnap_80253F60();
                            mnSnap_804A0A10.pending_loads = 0 as i32
                                as i32;
                            current_block_1097 = 2033130240696481971;
                        } else {
                            current_block_1097 = 4079176381495551589;
                        }
                    } else {
                        current_block_1097 = 4079176381495551589;
                    }
                    match current_block_1097 {
                        2033130240696481971 => {}
                        _ => {
                            mnSnap_80253640(mnSnap_804A0A10.cursor_idx / 4 as i32);
                            if mnSnap_804A0A10.cursor_idx as i32
                                / 4 as i32 == mnSnap_804A0A10.cur_page
                            {
                                jobj = mnSnap_804A0A10.select_jobj;
                                translate = &raw mut (**(&raw mut mnSnap_804A0A10
                                    .thumb_jobjs as *mut *mut HSD_JObj)
                                    .offset(
                                        (mnSnap_804A0A10.cursor_idx as i32
                                            % 4 as i32) as isize,
                                    ))
                                    .translate;
                                HSD_JObjSetTranslate(jobj, translate);
                                HSD_JObjClearFlagsAll(
                                    mnSnap_804A0A10.select_jobj,
                                    JOBJ_HIDDEN as u32,
                                );
                            } else {
                                HSD_JObjSetFlagsAll(
                                    mnSnap_804A0A10.select_jobj,
                                    JOBJ_HIDDEN as u32,
                                );
                            }
                            mnSnap_80253964();
                            mnSnap_804A0A10.state = 4 as i32;
                        }
                    }
                }
            } else if mnSnap_804A0A10.dlg_result == 2 {
                sfxBack();
                mnSnap_804A0A10.dlg_active = 0 as i32;
                mnSnap_804A0A10.dlg_timer = 0 as i32;
                if !mnSnap_804A0A10.dlg_text.is_null() {
                    HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                    mnSnap_804A0A10.dlg_text = null_mut();
                }
                mnSnap_804A0A10.state = 6 as i32;
            }
        }
        21 => {
            if mnSnap_804A0A10.dlg_result == 1 {
                let mut card_status_0: i16 = 0;
                mnSnap_UpdateSlotStatus(mnSnap_804A0A10.active_slot ^ 1 as i32);
                card_status_0 = mnSnap_804A0A10
                    .card_status[(mnSnap_804A0A10.active_slot as i32
                    ^ 1 as i32) as usize];
                if card_status_0 as i32 == 0 {
                    lbAudioAx_80024030(3 as i32);
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x148);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x147);
                    }
                    mnSnap_8025409C(0 as i32);
                    mnSnap_804A0A10.state = 0x16 as i32;
                } else if (card_status_0 as i32) < 0
                {
                    if card_status_0 as i32 == -1
                    {
                        if mnSnap_804A0A10.active_slot == 0 {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14c);
                        } else {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14b);
                        }
                    } else if card_status_0 as i32
                        == -2
                    {
                        if mnSnap_804A0A10.active_slot == 0 {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14e);
                        } else {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14d);
                        }
                    } else if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14a);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x149);
                    }
                    mnSnap_8025409C(0 as i32);
                    lbAudioAx_80024030(3 as i32);
                    mnSnap_804A0A10.state = 0x17 as i32;
                } else if mnSnap_804A0A10
                    .photo_count[(mnSnap_804A0A10.active_slot as i32
                    ^ 1 as i32) as usize] == 0
                {
                    lbAudioAx_80024030(3 as i32);
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x142);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x141);
                    }
                    mnSnap_8025409C(0 as i32);
                    mnSnap_804A0A10.state = 0x17 as i32;
                    lbAudioAx_80024030(3 as i32);
                } else {
                    let mut new_slot: i32 = 0;
                    sfxForward();
                    mnSnap_804A0A10.dlg_active = 0 as i32;
                    mnSnap_804A0A10.dlg_timer = 0 as i32;
                    if !mnSnap_804A0A10.dlg_text.is_null() {
                        HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                        mnSnap_804A0A10.dlg_text = null_mut();
                    }
                    mnSnap_804A0A10.state = 4 as i32;
                    new_slot = (mnSnap_804A0A10.active_slot as i32
                        ^ 1 as i32) as i32;
                    mnSnap_804A0A10.active_slot = new_slot;
                    mnSnap_804A0A10.cursor_idx = 0 as i32;
                    mnSnap_80253640(0 as i32);
                    mnSnap_80253964();
                    if mnSnap_804A0A10.cursor_idx as i32
                        / 4 as i32 == mnSnap_804A0A10.cur_page
                    {
                        jobj = mnSnap_804A0A10.select_jobj;
                        translate = &raw mut (**(&raw mut mnSnap_804A0A10.thumb_jobjs
                            as *mut *mut HSD_JObj)
                            .offset(
                                (mnSnap_804A0A10.cursor_idx as i32
                                    % 4 as i32) as isize,
                            ))
                            .translate;
                        HSD_JObjSetTranslate(jobj, translate);
                        HSD_JObjClearFlagsAll(
                            mnSnap_804A0A10.select_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                    } else {
                        HSD_JObjSetFlagsAll(
                            mnSnap_804A0A10.select_jobj,
                            JOBJ_HIDDEN as u32,
                        );
                    }
                }
            } else if mnSnap_804A0A10.dlg_result == 2 {
                sfxBack();
                mnSnap_804A0A10.dlg_active = 0 as i32;
                mnSnap_804A0A10.dlg_timer = 0 as i32;
                if !mnSnap_804A0A10.dlg_text.is_null() {
                    HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                    mnSnap_804A0A10.dlg_text = null_mut();
                }
                mnSnap_804A0A10.state = 6 as i32;
            }
        }
        22 => {
            if mnSnap_804A0A10.dlg_result == 2 {
                sfxBack();
                mnSnap_804A0A10.state = 6 as i32;
                mnSnap_804A0A10.dlg_active = 0 as i32;
                mnSnap_804A0A10.dlg_timer = 0 as i32;
                if !mnSnap_804A0A10.dlg_text.is_null() {
                    HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                    mnSnap_804A0A10.dlg_text = null_mut();
                }
            } else {
                let mut card_status_1: i16 = 0;
                mnSnap_UpdateSlotStatus(mnSnap_804A0A10.active_slot ^ 1 as i32);
                card_status_1 = mnSnap_804A0A10
                    .card_status[(mnSnap_804A0A10.active_slot as i32
                    ^ 1 as i32) as usize];
                if card_status_1 as i32 != 0 {
                    if (card_status_1 as i32) < 0 {
                        if card_status_1 as i32
                            == -1
                        {
                            if mnSnap_804A0A10.active_slot == 0 {
                                HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14c);
                            } else {
                                HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14b);
                            }
                        } else if card_status_1 as i32
                            == -2
                        {
                            if mnSnap_804A0A10.active_slot == 0 {
                                HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14e);
                            } else {
                                HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14d);
                            }
                        } else if mnSnap_804A0A10.active_slot == 0
                        {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x14a);
                        } else {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x149);
                        }
                        mnSnap_8025409C(0 as i32);
                        lbAudioAx_80024030(3 as i32);
                        mnSnap_804A0A10.state = 0x17 as i32;
                    } else if mnSnap_804A0A10
                        .photo_count[(mnSnap_804A0A10.active_slot as i32
                        ^ 1 as i32) as usize] == 0
                    {
                        if mnSnap_804A0A10.active_slot == 0 {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x142);
                        } else {
                            HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x141);
                        }
                        mnSnap_8025409C(0 as i32);
                        mnSnap_804A0A10.state = 0x17 as i32;
                        lbAudioAx_80024030(3 as i32);
                    } else {
                        let mut new_slot_0: i32 = 0;
                        mnSnap_804A0A10.dlg_active = 0 as i32;
                        mnSnap_804A0A10.dlg_timer = 0 as i32;
                        if !mnSnap_804A0A10.dlg_text.is_null() {
                            HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                            mnSnap_804A0A10.dlg_text = core::ptr::null_mut::<
                                HSD_Text,
                            >();
                        }
                        mnSnap_804A0A10.state = 4 as i32;
                        new_slot_0 = (mnSnap_804A0A10.active_slot as i32
                            ^ 1 as i32) as i32;
                        mnSnap_804A0A10.active_slot = new_slot_0;
                        mnSnap_804A0A10.cursor_idx = 0 as i32;
                        mnSnap_80253640(0 as i32);
                        mnSnap_80253964();
                        if mnSnap_804A0A10.cursor_idx as i32
                            / 4 as i32 == mnSnap_804A0A10.cur_page
                        {
                            jobj = mnSnap_804A0A10.select_jobj;
                            translate = &raw mut (**(&raw mut mnSnap_804A0A10.thumb_jobjs
                                as *mut *mut HSD_JObj)
                                .offset(
                                    (mnSnap_804A0A10.cursor_idx as i32
                                        % 4 as i32) as isize,
                                ))
                                .translate;
                            HSD_JObjSetTranslate(jobj, translate);
                            HSD_JObjClearFlagsAll(
                                mnSnap_804A0A10.select_jobj,
                                JOBJ_HIDDEN as u32,
                            );
                        } else {
                            HSD_JObjSetFlagsAll(
                                mnSnap_804A0A10.select_jobj,
                                JOBJ_HIDDEN as u32,
                            );
                        }
                    }
                }
            }
        }
        23 => {
            if mnSnap_804A0A10.dlg_result == 2 {
                sfxBack();
                mnSnap_804A0A10.state = 6 as i32;
                mnSnap_804A0A10.dlg_active = 0 as i32;
                mnSnap_804A0A10.dlg_timer = 0 as i32;
                if !mnSnap_804A0A10.dlg_text.is_null() {
                    HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
                    mnSnap_804A0A10.dlg_text = null_mut();
                }
            } else {
                let mut card_status_2: i16 = 0;
                mnSnap_UpdateSlotStatus(mnSnap_804A0A10.active_slot ^ 1 as i32);
                card_status_2 = mnSnap_804A0A10
                    .card_status[(mnSnap_804A0A10.active_slot as i32
                    ^ 1 as i32) as usize];
                if card_status_2 as i32 == 0 {
                    if mnSnap_804A0A10.active_slot == 0 {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x152);
                    } else {
                        HSD_SisLib_803A6368(mnSnap_804A0A10.dlg_text, 0x151);
                    }
                    mnSnap_8025409C(1 as i32);
                    mnSnap_804A0A10.state = 0x15 as i32;
                }
            }
        }
        _ => {}
    }
    while mnSnap_804A0A10.pending_loads != 0 {
        mnSnap_8025329C();
    }
}
pub unsafe extern "C" fn fn_80257D7C() {
    let mut snap: *mut mnSnap_State = &raw mut mnSnap_804A0A10;
    let mut i: i32 = 0;
    let mut buttons: u64 = 0;
    if mn_804D6BC8.cooldown as i32 != 0 {
        Menu_DecrementAnimTimer();
        return;
    }
    buttons = Menu_GetAllInputs();
    if mnSnap_804A0A10.state == 4
        || mnSnap_804A0A10.state == 2
    {
        if buttons & MenuInput_Back as i32 as u64 != 0 {
            sfxBack();
            mn_804A04F0.entering_menu = 0;
            mn_80229894(5, 0, 3);
            if !mnSnap_804A0A10.main_gobj.is_null() {
                HSD_GObjFree(mnSnap_804A0A10.main_gobj);
            }
            if !mnSnap_804A0A10.sub_gobj.is_null() {
                HSD_GObjFree(mnSnap_804A0A10.sub_gobj);
            }
            if !(*snap).cursor_gobj.is_null() {
                HSD_GObjFree((*snap).cursor_gobj);
            }
            if !mnSnap_804A0A10.warn_gobj.is_null() {
                HSD_GObjFree(mnSnap_804A0A10.warn_gobj);
            }
            let mut text_p: *mut *mut HSD_Text = core::ptr::null_mut::<
                *mut HSD_Text,
            >();
            text_p = snap as *mut *mut HSD_Text;
            i = 0 as i32;
            while i < 4 {
                if !(*text_p.offset(0x3e as i32 as isize)).is_null() {
                    HSD_SisLib_803A5CC4(
                        *text_p.offset(0x3e as i32 as isize),
                    );
                }
                i += 1;
                text_p = text_p.offset(1);
            }
            let mut text_p_0: *mut *mut HSD_Text = core::ptr::null_mut::<
                *mut HSD_Text,
            >();
            i = 0 as i32;
            text_p_0 = snap as *mut *mut HSD_Text;
            while i < 2 {
                if !(*text_p_0.offset(0x42 as i32 as isize)).is_null() {
                    HSD_SisLib_803A5CC4(
                        *text_p_0.offset(0x42 as i32 as isize),
                    );
                }
                i += 1;
                text_p_0 = text_p_0.offset(1);
            }
            if !mnSnap_804A0A10.page_text.is_null() {
                HSD_SisLib_803A5CC4(mnSnap_804A0A10.page_text);
            }
            if !mnSnap_804A0A10.total_text.is_null() {
                HSD_SisLib_803A5CC4(mnSnap_804A0A10.total_text);
            }
            if !mnSnap_804A0A10.dlg_text.is_null() {
                HSD_SisLib_803A5CC4(mnSnap_804A0A10.dlg_text);
            }
            return;
        }
    }
    mnSnap_804A0A10.frame_count += 1;
}
unsafe extern "C" fn mnSnap_LoadPageIndicator(
    mut page_joint: *mut *mut c_void,
    mut snap: *mut mnSnap_State,
    mut parent: *mut *mut HSD_JObj,
    mut jobj: *mut *mut HSD_JObj,
) {
    *jobj = HSD_JObjLoadJoint(*page_joint as *mut HSD_Joint);
    (*snap).fullview_jobj = *jobj;
    HSD_JObjAddChild(*parent, *jobj);
}
unsafe extern "C" fn mnSnap_GetCardStatus(mut snap: *mut mnSnap_State) -> *mut i16 {
    return &raw mut (*snap).card_status as *mut i16;
}
unsafe extern "C" fn mnSnap_InitPageText(mut text: *mut *mut HSD_Text) {
    let mut snap: *mut mnSnap_State = &raw mut mnSnap_804A0A10;
    (*snap).page_text = HSD_SisLib_803A6754(
        0,
        0,
    );
    *text = (*snap).page_text;
    (**text).pos_x = -1.8_f32 as f32;
    (**text).pos_y = 9.2_f32 as f32;
    (**text).pos_z = 17.0_f32 as f32;
}
unsafe extern "C" fn mnSnap_GetMainJoint(
    mut snap: *mut mnSnap_State,
) -> *mut *mut c_void {
    return &raw mut (*snap).main_joint;
}
unsafe extern "C" fn mnSnap_GetMainShapeAnim(
    mut snap: *mut mnSnap_State,
) -> *mut *mut c_void {
    return &raw mut (*snap).main_shapeanim;
}
unsafe extern "C" fn mnSnap_CreateThumbnails(
    mut snap: *mut mnSnap_State,
    mut thumb_root_ptr: *mut *mut HSD_JObj,
    mut photo_joint: *mut *mut c_void,
    mut sub_animjoint: *mut *mut c_void,
    mut sub_matanim: *mut *mut c_void,
    mut sub_shapeanim: *mut *mut c_void,
) {
    let mut jobj2: *mut HSD_JObj = null_mut();
    let mut step_z: f32 = 0.;
    let mut step_y: f32 = 0.;
    let mut step_x: f32 = 0.;
    let mut i: i32 = 0;
    let mut end_pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut start_pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    HSD_JObjGetTranslation((*snap).thumb_start, &raw mut start_pos);
    HSD_JObjGetTranslation((*snap).thumb_end, &raw mut end_pos);
    step_x = end_pos.x - start_pos.x;
    step_y = end_pos.y - start_pos.y;
    step_z = end_pos.z - start_pos.z;
    i = 0 as i32;
    while i < 5 {
        jobj2 = HSD_JObjLoadJoint(*photo_joint as *mut HSD_Joint);
        HSD_JObjAddAnimAll(
            jobj2,
            *sub_animjoint as *mut HSD_AnimJoint,
            *sub_matanim as *mut HSD_MatAnimJoint,
            *sub_shapeanim as *mut HSD_ShapeAnimJoint,
        );
        end_pos.x = step_x * i as f32 + start_pos.x;
        end_pos.y = step_y * i as f32 + start_pos.y;
        end_pos.z = step_z * i as f32 + start_pos.z;
        HSD_JObjSetTranslate(jobj2, &raw mut end_pos);
        (*snap).option_jobjs[i as usize] = jobj2;
        HSD_JObjAddChild(*thumb_root_ptr, jobj2);
        i += 1;
    }
}
pub unsafe extern "C" fn mnSnap_80257F24() {
    let mut page_name: *const c_char = core::ptr::null::<
        c_char,
    >();
    let mut snap: *mut mnSnap_State = &raw mut mnSnap_804A0A10;
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut jobj2: *mut HSD_JObj = null_mut();
    let mut move_jobj_ptr: *mut *mut HSD_JObj = core::ptr::null_mut::<*mut HSD_JObj>();
    let mut slot_jobj_ptr: *mut *mut HSD_JObj = core::ptr::null_mut::<*mut HSD_JObj>();
    let mut thumb_root_ptr: *mut *mut HSD_JObj = core::ptr::null_mut::<
        *mut HSD_JObj,
    >();
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    let mut text: *mut HSD_Text = null_mut();
    let mut archive: *mut HSD_Archive = null_mut();
    let mut main_joint: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut main_shapeanim: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut main_matanim: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut csr_shapeanim: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut csr_matanim: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut csr_animjoint: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut csr_joint: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut photo_joint: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut sub_shapeanim: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut sub_matanim: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut sub_animjoint: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut page_joint: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut arrows_joint: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut arrows_animjoint: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut arrows_shapeanim: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut warn_joint: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut main_animjoint: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut warn_shapeanim: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut warn_matanim: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut warn_animjoint: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut arrows_matanim: *mut *mut c_void = core::ptr::null_mut::<
        *mut c_void,
    >();
    let mut i: i32 = 0;
    let mut zero: i32 = 0;
    mn_804D6BC8.cooldown = 5;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x19 as MenuKind8;
    mn_804A04F0.hovered_selection = zero as u16;
    lb_8001CDB4();
    memzero(
        &raw mut mnSnap_804A0A10 as *mut c_void,
        core::mem::size_of::<mnSnap_State>() as isize,
    );
    (*snap).state = zero;
    (*snap).timer = 6 as i32;
    (*snap).photo_count[0 as i32 as usize] = zero;
    let mut photo_count: *mut i32 = &raw mut (*snap).photo_count as *mut i32;
    *photo_count.offset(1 as i32 as isize) = zero;
    (*snap).card_status[0 as i32 as usize] = zero as i16;
    let mut card_status: *mut i16 = mnSnap_GetCardStatus(snap);
    *card_status.offset(1 as i32 as isize) = zero as i16;
    archive = mn_804D6BB8;
    main_animjoint = &raw mut (*snap).main_animjoint;
    main_matanim = &raw mut (*snap).main_matanim;
    main_shapeanim = &raw mut (*snap).main_shapeanim;
    csr_joint = &raw mut (*snap).csr_joint;
    csr_animjoint = &raw mut (*snap).csr_animjoint;
    csr_matanim = &raw mut (*snap).csr_matanim;
    csr_shapeanim = &raw mut (*snap).csr_shapeanim;
    photo_joint = &raw mut (*snap).photo_joint;
    sub_animjoint = &raw mut (*snap).sub_animjoint;
    sub_matanim = &raw mut (*snap).sub_matanim;
    sub_shapeanim = &raw mut (*snap).sub_shapeanim;
    page_joint = &raw mut (*snap).page_joint;
    arrows_joint = &raw mut (*snap).arrows_joint;
    arrows_animjoint = &raw mut (*snap).arrows_animjoint;
    arrows_matanim = &raw mut (*snap).arrows_matanim;
    arrows_shapeanim = &raw mut (*snap).arrows_shapeanim;
    warn_joint = &raw mut (*snap).warn_joint;
    warn_animjoint = &raw mut (*snap).warn_animjoint;
    warn_matanim = &raw mut (*snap).warn_matanim;
    warn_shapeanim = &raw mut (*snap).warn_shapeanim;
    main_joint = &raw mut (*snap).main_joint;
    page_name = c"MenMainPhotoSn_Top_joint".as_ptr();
    lbArchive_LoadSections(archive, &[(mnSnap_GetMainJoint(snap) as *mut *mut c_void, c"MenMainConSn_Top_joint".as_ptr()), (main_animjoint as *mut *mut c_void, c"MenMainConSn_Top_animjoint".as_ptr()), (main_matanim as *mut *mut c_void, c"MenMainConSn_Top_matanim_joint".as_ptr()), (mnSnap_GetMainShapeAnim(snap) as *mut *mut c_void, c"MenMainConSn_Top_shapeanim_joint".as_ptr()), (csr_joint as *mut *mut c_void, c"MenMainSubSn_Top_joint".as_ptr()), (csr_animjoint as *mut *mut c_void, c"MenMainSubSn_Top_animjoint".as_ptr()), (csr_matanim as *mut *mut c_void, c"MenMainSubSn_Top_matanim_joint".as_ptr()), (csr_shapeanim as *mut *mut c_void, c"MenMainSubSn_Top_shapeanim_joint".as_ptr()), (photo_joint as *mut *mut c_void, c"MenMainSubCsrSn_Top_joint".as_ptr()), (sub_animjoint as *mut *mut c_void, c"MenMainSubCsrSn_Top_animjoint".as_ptr()), (sub_matanim as *mut *mut c_void, c"MenMainSubCsrSn_Top_matanim_joint".as_ptr()), (sub_shapeanim as *mut *mut c_void, c"MenMainSubCsrSn_Top_shapeanim_joint".as_ptr()), (page_joint as *mut *mut c_void, page_name), (arrows_joint as *mut *mut c_void, c"MenMainLoadSn_Top_joint".as_ptr()), (arrows_animjoint as *mut *mut c_void, c"MenMainLoadSn_Top_animjoint".as_ptr()), (arrows_matanim as *mut *mut c_void, c"MenMainLoadSn_Top_matanim_joint".as_ptr()), (arrows_shapeanim as *mut *mut c_void, c"MenMainLoadSn_Top_shapeanim_joint".as_ptr()), (warn_joint as *mut *mut c_void, c"MenMainWarCmn_Top_joint".as_ptr()), (warn_animjoint as *mut *mut c_void, c"MenMainWarCmn_Top_animjoint".as_ptr()), (warn_matanim as *mut *mut c_void, c"MenMainWarCmn_Top_matanim_joint".as_ptr()), (warn_shapeanim as *mut *mut c_void, c"MenMainWarCmn_Top_shapeanim_joint".as_ptr())]);
    gobj = GObj_Create(6, 7, 0x80);
    (*snap).main_gobj = gobj;
    jobj = HSD_JObjLoadJoint(*main_joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> ()>,
            GObj_RenderFunc,
        >(Some(fn_80253DB4)),
        4,
        0x80,
    );
    HSD_JObjAddAnimAll(
        jobj,
        *main_animjoint as *mut HSD_AnimJoint,
        *main_matanim as *mut HSD_MatAnimJoint,
        *main_shapeanim as *mut HSD_ShapeAnimJoint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0_f32);
    lb_80011E24(jobj as *mut HSD_JObj, (&raw mut (*snap).thumb_jobjs as *mut *mut HSD_JObj)
            .offset(0 as i32 as isize) as *mut *mut HSD_JObj, &[8, 9, 0xa, 0xb, 0xc, 0xd, 6, 2, 1, -1]);
    slot_jobj_ptr = (&raw mut (*snap).slot_jobjs as *mut *mut HSD_JObj)
        .offset(0 as i32 as isize) as *mut *mut HSD_JObj;
    jobj2 = *slot_jobj_ptr;
    (*snap).blank_img = (*(*(*(*(*jobj2).u.dobj).mobj).tobj).imagedesc).image_ptr;
    if (*snap).photo_count[(*snap).active_slot as usize] <= 4 {
        HSD_JObjSetFlagsAll((*snap).arrow_jobj, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjClearFlagsAll((*snap).arrow_jobj, JOBJ_HIDDEN as u32);
    }
    HSD_AObjSetFlags(
        (*(*(*(*(*snap).select_jobj).u.dobj).mobj).tobj).aobj,
        0x20000000 as i32 as u32,
    );
    move_jobj_ptr = &raw mut (*snap).move_jobj;
    HSD_JObjSetFlagsAll(*move_jobj_ptr, JOBJ_HIDDEN as u32);
    HSD_AObjSetFlags(
        (*(*(*(**move_jobj_ptr).u.dobj).mobj).tobj).aobj,
        0x20000000 as i32 as u32,
    );
    HSD_GObj_SetupProc(
        gobj,
        core::mem::transmute::<
            Option<unsafe extern "C" fn() -> ()>,
            HSD_GObjEvent,
        >(Some(fn_802545C4)),
        0,
    );
    gobj = GObj_Create(6, 7, 0x80);
    (*snap).sub_gobj = gobj;
    jobj = HSD_JObjLoadJoint(*arrows_joint as *mut HSD_Joint);
    HSD_JObjSetTranslateX(jobj, 3.3_f32);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> ()>,
            GObj_RenderFunc,
        >(Some(fn_80253DE8)),
        4,
        0x80,
    );
    HSD_JObjAddAnimAll(
        jobj,
        *arrows_animjoint as *mut HSD_AnimJoint,
        *arrows_matanim as *mut HSD_MatAnimJoint,
        *arrows_shapeanim as *mut HSD_ShapeAnimJoint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0_f32);
    HSD_JObjAnimAll(jobj);
    lb_80011E24(jobj as *mut HSD_JObj, slot_jobj_ptr as *mut *mut HSD_JObj, &[1, 2, 3, 4, -1]);
    gobj = GObj_Create(6, 7, 0x80);
    (*snap).cursor_gobj = gobj;
    jobj = HSD_JObjLoadJoint(*csr_joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> ()>,
            GObj_RenderFunc,
        >(Some(fn_80253E1C)),
        6,
        0x80,
    );
    HSD_JObjAddAnimAll(
        jobj,
        *csr_animjoint as *mut HSD_AnimJoint,
        *csr_matanim as *mut HSD_MatAnimJoint,
        *csr_shapeanim as *mut HSD_ShapeAnimJoint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0_f32);
    HSD_JObjAnimAll(jobj);
    thumb_root_ptr = &raw mut (*snap).thumb_root;
    lb_80011E24(jobj as *mut HSD_JObj, thumb_root_ptr as *mut *mut HSD_JObj, &[0, 1, 7, 8, -1]);
    mnSnap_CreateThumbnails(
        &raw mut mnSnap_804A0A10,
        thumb_root_ptr,
        photo_joint,
        sub_animjoint,
        sub_matanim,
        sub_shapeanim,
    );
    mnSnap_LoadPageIndicator(page_joint, snap, thumb_root_ptr, &raw mut jobj2);
    HSD_JObjSetFlagsAll(jobj2, JOBJ_HIDDEN as u32);
    i = 0 as i32;
    while i < 4 {
        mnSnap_804A0A10.thumb_labels[i as usize] = HSD_SisLib_803A6754(
            0,
            0,
        );
        i += 1;
    }
    text = (*snap).thumb_labels[0 as i32 as usize];
    (*text).pos_x = -11.2_f32 as f32;
    (*text).pos_y = -2.6_f32 as f32;
    (*text).pos_z = 19.8_f32 as f32;
    (*text).font_size.x = 0.028_f32 as f32;
    (*text).font_size.y = 0.031_f32 as f32;
    (*text).default_alignment = 1;
    text = (*snap).thumb_labels[1 as i32 as usize];
    (*text).pos_x = -11.2_f32 as f32;
    (*text).pos_y = 4.4_f32 as f32;
    (*text).pos_z = 19.8_f32 as f32;
    (*text).font_size.x = 0.028_f32 as f32;
    (*text).font_size.y = 0.031_f32 as f32;
    (*text).default_alignment = 1;
    text = (*snap).thumb_labels[2 as i32 as usize];
    (*text).pos_x = 10.9_f32 as f32;
    (*text).pos_y = -2.6_f32 as f32;
    (*text).pos_z = 19.8_f32 as f32;
    (*text).font_size.x = 0.028_f32 as f32;
    (*text).font_size.y = 0.031_f32 as f32;
    (*text).default_alignment = 1;
    text = (*snap).thumb_labels[3 as i32 as usize];
    (*text).pos_x = 10.9_f32 as f32;
    (*text).pos_y = 4.4_f32 as f32;
    (*text).pos_z = 19.8_f32 as f32;
    (*text).font_size.x = 0.028_f32 as f32;
    (*text).font_size.y = 0.031_f32 as f32;
    (*text).default_alignment = 1;
    i = 0 as i32;
    while i < 2 {
        mnSnap_804A0A10.count_texts[i as usize] = HSD_SisLib_803A6754(
            0,
            0,
        );
        i += 1;
    }
    text = (*snap).count_texts[0 as i32 as usize];
    (*text).pos_x = -11.6_f32 as f32;
    (*text).pos_y = 9.0_f32 as f32;
    (*text).pos_z = 17.3_f32 as f32;
    (*text).font_size.x = 0.045_f32 as f32;
    (*text).font_size.y = 0.045_f32 as f32;
    (*text).default_alignment = 1;
    text = (*snap).count_texts[1 as i32 as usize];
    (*text).pos_x = 11.0_f32 as f32;
    (*text).pos_y = 9.0_f32 as f32;
    (*text).pos_z = 17.3_f32 as f32;
    (*text).font_size.x = 0.045_f32 as f32;
    (*text).font_size.y = 0.045_f32 as f32;
    (*text).default_alignment = 1;
    mnSnap_InitPageText(&raw mut text);
    (*text).font_size.x = 0.04_f32 as f32;
    (*text).font_size.y = 0.04_f32 as f32;
    (*text).default_alignment = 1;
    (*snap).total_text = HSD_SisLib_803A6754(
        0,
        0,
    );
    text = (*snap).total_text;
    (*text).pos_x = 1.6_f32 as f32;
    (*text).pos_y = 9.2_f32 as f32;
    (*text).pos_z = 17.0_f32 as f32;
    (*text).font_size.x = 0.04_f32 as f32;
    (*text).font_size.y = 0.04_f32 as f32;
    (*text).default_alignment = 1;
    mnSnap_80253964();
    gobj = GObj_Create(6, 7, 0x80);
    (*snap).warn_gobj = gobj;
    jobj = HSD_JObjLoadJoint(*warn_joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> ()>,
            GObj_RenderFunc,
        >(Some(fn_80253E5C)),
        6,
        0x80,
    );
    HSD_JObjAddAnimAll(
        jobj,
        *warn_animjoint as *mut HSD_AnimJoint,
        *warn_matanim as *mut HSD_MatAnimJoint,
        *warn_shapeanim as *mut HSD_ShapeAnimJoint,
    );
    HSD_JObjReqAnimAll(jobj, 10.0_f32);
    HSD_JObjAnimAll(jobj);
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut mnSnap_804A0A10.dlg_root, &[0, 2, 4, 5, 6, 7, 8, 0xa, 0xb, 0xd, -1]);
    (*snap).dlg_active = 0 as i32;
    gobj = GObj_Create(0, 1, 0x80);
    proc_0 = HSD_GObj_SetupProc(
        gobj,
        core::mem::transmute::<
            Option<unsafe extern "C" fn() -> ()>,
            HSD_GObjEvent,
        >(Some(fn_80257D7C)),
        0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
}
pub const NULL: i32 = 0;
