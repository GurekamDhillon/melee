//! TARGET_PC port of src/melee/mn/mndiagram2.c.
#![allow(non_snake_case, non_upper_case_globals, unused_imports, unused_variables, unused_assignments, unused_mut, unused_unsafe, static_mut_refs, clippy::all)]
use core::ffi::c_void;
use melee_types::*;
use std::vec::Vec;
use crate::oracle::Va;
use super::mninl::*;
use super::hsd::{sfxBack,sfxForward,sfxMove};
use melee_core::fp::fmadds;
use super::hsd::jobj_child as HSD_JObjGetChild;
pub use crate::mn::{mnDiagram2_IsTimeStat,mnDiagram2_IsDistanceStat,mnDiagram2_IsPercentageStat,mnDiagram2_IsIconOnlyStat};
pub mod ext { use super::*; crate::oext_fns! {
fn __assert(a0: *const i8, a1: u32, a2: *const i8) -> ();
fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32) -> ();
fn lbAudioAx_80024030(a0: i32) -> ();
fn mn_80229624(a0: u32) -> u32;
fn mn_80229894(a0: i32, a1: u16, a2: i32) -> ();
fn mn_IsFighterUnlocked(a0: SelectableCharacterKind) -> i32;
fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32;
fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32) -> ();
fn HSD_JObjRemoveAll(a0: *mut HSD_JObj) -> ();
fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj;
fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint) -> ();
fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32) -> ();
fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, flags: u32) -> ();
fn HSD_JObjAddChild(jobj: *mut HSD_JObj, child: *mut HSD_JObj) -> ();
fn HSD_SisLib_803A6754(arg0: i32, arg1: i32) -> *mut HSD_Text;
fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) -> ();
fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text;
fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) -> ();
fn mnDiagram_GetFighterByIndex(idx: i32) -> u8;
fn mnDiagram_GetNameByIndex(idx: i32) -> u8;
fn mnDiagram_IsDistanceOverflow(distance: u32) -> i32;
fn mnDiagram_ConvertDistanceForDisplay(distance: u32) -> u32;
fn mnDiagram_GetHitPercentage(is_name_mode: u8, player_index: u8) -> i32;
fn mnDiagram_GetPlayPercentage(is_name_mode: u8, player_index: u8) -> i32;
fn mnDiagram_GetAveragePlayerCount(is_name_mode: u8, player_index: u8) -> i32;
fn mnDiagram_GetNameTotalKOs(field_index: u8) -> i32;
fn mnDiagram_GetNameTotalFalls(field_index: u8) -> i32;
fn mnDiagram_GetFighterTotalKOs(field_index: u8) -> i32;
fn mnDiagram_GetFighterTotalFalls(field_index: u8) -> i32;
fn mnDiagram_FormatDecimalNumber(buf: *mut i8, val: u32, decimal_places: i32) -> ();
fn mnDiagram_FormatTime(buf: *mut i8, seconds: i32) -> ();
fn mnDiagram_IntToStr(buf: *mut i8, val: u32) -> ();
fn mnDiagram_GetPrevNameIndex(idx: i32) -> u8;
fn mnDiagram_GetNextNameIndex(idx: i32) -> u8;
fn mnDiagram_GetPrevFighterIndex(idx: i32) -> u8;
fn mnDiagram_GetNextFighterIndex(idx: i32) -> u8;
fn mnDiagram_GetNamePlayTimeByFighter(name_idx: i32, fighter_idx: SelectableCharacterKind) -> u32;
fn mnDiagram_GetRankedFighterForName(rank: i32, name_idx: i32, func: Option<
            unsafe extern "C" fn(i32, i32) -> u32,
        >) -> i32;
fn mnDiagram_GetLeastPlayedFighter(name_idx: u8) -> u8;
fn mnDiagram_CountUnlockedFighters() -> i32;
fn mnDiagram_CreateFighterIcon(idx: i32, arg1: i32) -> *mut HSD_JObj;
fn mnDiagram_Init(arg0: u8, arg1: u8) -> ();
fn mnDiagram3_Init(arg0: *mut c_void) -> ();
fn gm_80160B40(a0: *mut HSD_Text, a1: u8, a2: u8) -> ();
fn gm_SelKindToCKind(selkind: u8) -> u8;
fn gmMainLib_GetGameRules() -> *mut GameRules;
fn GetPersistentFighterData(a0: SelectableCharacterKind) -> *mut FighterData;
fn GetPersistentNameData(a0: i32) -> *mut NameTagData;
fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) -> ();
fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32) -> ();
fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void) -> ();
fn HSD_GObjFree(a0: *mut HSD_GObj) -> ();
fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
fn HSD_GObjProc_RemoveProc(gproc: *mut HSD_GObjProc) -> ();
fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void) -> ()>, data: *mut c_void) -> ();
fn HSD_Free(ptr: *mut c_void) -> ();
fn HSD_MemAlloc(size: ssize_t) -> *mut c_void;
fn GetNameText(slot: u8) -> *mut i8;
fn GetNameCount() -> i32;
}
crate::ext_va! { fn OSReport(fmt: *const i8) va 0 -> (); fn HSD_SisLib_803A6B98(t: *mut HSD_Text, x:f32,y:f32,fmt:*const i8) va 3 -> i32; }
crate::ext_vk! { fn lb_80011E24(j:*mut HSD_JObj,out:*mut *mut HSD_JObj) vk "ii" -> i32; }
}
use ext::*;
crate::globals! {
rw pub HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = unsafe { core::mem::zeroed() };
rw pub HSD_GObj_804D783C: i32 = unsafe { core::mem::zeroed() };
rw pub HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
rw pub mn_804A04F0: MenuFlow = unsafe { core::mem::zeroed() };
rw pub MenMainConB2_Top: StaticModelDesc = unsafe { core::mem::zeroed() };
rw pub mnDiagram2_804D4FB8: GXColor  = GXColor {
    r: 0 as u8,
    g: 0 as u8,
    b: 0 as u8,
    a: 0xff as u8,
};
rw pub mnDiagram2_804D4FBC: GXColor  = GXColor {
    r: 0xff as u8,
    g: 0xc8 as u8,
    b: 0 as u8,
    a: 0xff as u8,
};
rw pub mnDiagram2_803EEAD0: MnDiagram2RowLayout  = MnDiagram2RowLayout {
    header_pos: Vec3 {
        x: -2.5f32,
        y: 0.3f32,
        z: 0.0f32,
    },
    label_pos: Vec3 {
        x: -2.2f32,
        y: 0.5f32,
        z: 0.0f32,
    },
    value_pos: Vec3 {
        x: -1.0f32,
        y: 0.5f32,
        z: 0.0f32,
    },
    icon_pos: Vec3 {
        x: -2.0f32,
        y: 0.0f32,
        z: 0.0f32,
    },
    label_ids: [
        74 as i32 as u16,
        75 as i32 as u16,
        76 as i32 as u16,
        77 as i32 as u16,
        78 as i32 as u16,
        79 as i32 as u16,
        80 as i32 as u16,
        81 as i32 as u16,
        82 as i32 as u16,
        83 as i32 as u16,
        84 as i32 as u16,
        85 as i32 as u16,
        86 as i32 as u16,
        87 as i32 as u16,
        88 as i32 as u16,
        89 as i32 as u16,
        90 as i32 as u16,
        91 as i32 as u16,
        92 as i32 as u16,
        93 as i32 as u16,
        94 as i32 as u16,
        95 as i32 as u16,
        96 as i32 as u16,
        97 as i32 as u16,
    ],
    unit_glyph_ids: [
        122 as i32 as u16,
        122 as i32 as u16,
        122 as i32 as u16,
        124 as i32 as u16,
        124 as i32 as u16,
        124 as i32 as u16,
        124 as i32 as u16,
        124 as i32 as u16,
        122 as i32 as u16,
        122 as i32 as u16,
        122 as i32 as u16,
        0xffff as i32 as u16,
        124 as i32 as u16,
        123 as i32 as u16,
        126 as i32 as u16,
        126 as i32 as u16,
        126 as i32 as u16,
        126 as i32 as u16,
        125 as i32 as u16,
        125 as i32 as u16,
        125 as i32 as u16,
        0xffff as i32 as u16,
        0xffff as i32 as u16,
        0xffff as i32 as u16,
    ],
};
rw pub mnDiagram2_803EEB60: [AnimLoopSettings; 2]  = [
    AnimLoopSettings {
        start_frame: 10.0f32,
        end_frame: 19.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 0.0f32,
        end_frame: 199.0f32,
        loop_frame: 0.0f32,
    },
];
rw pub mnDiagram2_804D6C18: *mut HSD_GObj  = ::core::ptr::null::<HSD_GObj>()
    as *mut HSD_GObj;
rw pub mnDiagram2_804D4FD0: [u8; 3]  = [
    0x81 as i32 as u8,
    0x7c as i32 as u8,
    0 as i32 as u8,
];
}
unsafe fn entry_value(e:&mnDiagram2_SortEntry)->u64{((e.anon_0.x8 as u32 as u64)<<32)|(e.anon_0.xC as u32 as u64)}
unsafe fn HSD_GObjGetHSDObj(g:*mut HSD_GObj)->*mut c_void { (*g).hsd_obj }
unsafe fn HSD_GObjGetUserData(g:*mut HSD_GObj)->*mut c_void { (*g).user_data }
unsafe fn Menu_GetAllInputs()->u64 { mn_804A04F0.buttons = mn_80229624(4) as u64; mn_804A04F0.buttons }
pub unsafe extern "C" fn mnDiagram2_ClearStatRows(mut gobj: *mut HSD_GObj) {
    let mut ptr: *mut Diagram2 = ::core::ptr::null_mut::<Diagram2>();
    let mut data: *mut Diagram2 = ::core::ptr::null_mut::<Diagram2>();
    let mut i: i32 = 0;
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    data = HSD_GObjGetUserData(gobj) as *mut Diagram2;
    ptr = data;
    i = 0 as i32;
    while i < 10 as i32 {
        if !(*data).row_labels[i as usize].is_null() {
            HSD_SisLib_803A5CC4((*ptr).row_labels[i as usize]);
            (*data).row_labels[i as usize] = ::core::ptr::null_mut::<HSD_Text>();
        }
        if !(*data).row_values[i as usize].is_null() {
            HSD_SisLib_803A5CC4((*ptr).row_values[i as usize]);
            (*data).row_values[i as usize] = ::core::ptr::null_mut::<HSD_Text>();
        }
        if !(*data).row_icons[i as usize].is_null() {
            HSD_SisLib_803A5CC4((*ptr).row_icons[i as usize]);
            (*data).row_icons[i as usize] = ::core::ptr::null_mut::<HSD_Text>();
        }
        i += 1;
    }
    jobj = HSD_JObjGetChild((*data).icon_parent as *mut HSD_JObj);
    if !jobj.is_null() {
        HSD_JObjRemoveAll(jobj);
    }
}
pub unsafe extern "C" fn mnDiagram2_UpdateHeader(
    mut gobj: *mut HSD_GObj,
    mut is_name_mode: u8,
    mut entity_idx: u8,
) {
    let mut sp18: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut data: *mut Diagram2 = ::core::ptr::null_mut::<Diagram2>();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut tmp: *mut c_void = ::core::ptr::null_mut::<
        c_void,
    >();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut nametag_slot_or_selkind: u8 = 0;
    let mut _pad: [u8; 4] = [0; 4];
    data = (*gobj).user_data as *mut Diagram2;
    if is_name_mode as i32 != 0 as i32 {
        nametag_slot_or_selkind = mnDiagram_GetNameByIndex(
            entity_idx as i32,
        );
    } else {
        nametag_slot_or_selkind = mnDiagram_GetFighterByIndex(
            entity_idx as i32,
        );
    }
    if is_name_mode as i32 == 0 as i32 {
        tmp = (*data).x18 as *mut c_void;
        if tmp.is_null() {
            tmp = ::core::ptr::null_mut::<c_void>();
        } else {
            tmp = (*(tmp as *mut HSD_JObj)).child as *mut c_void;
        }
        jobj = tmp as *mut HSD_JObj;
        if !jobj.is_null() {
            HSD_JObjRemoveAll(jobj);
        }
        HSD_JObjAddChild(
            (*data).x18 as *mut HSD_JObj,
            mnDiagram_CreateFighterIcon(
                nametag_slot_or_selkind as i32,
                0 as i32,
            ),
        );
    }
    if !(*data).header_text.is_null() {
        HSD_SisLib_803A5CC4((*data).header_text);
    }
    text = HSD_SisLib_803A6754(0 as i32, 1 as i32);
    (*data).header_text = text;
    if is_name_mode as i32 != 0 as i32 {
        lb_8000B1CC(
            (*data).x24 as *mut HSD_JObj,
            &raw mut mnDiagram2_803EEAD0.header_pos,
            &raw mut sp18,
        );
    } else {
        lb_8000B1CC(
            (*data).x1C as *mut HSD_JObj,
            &raw mut mnDiagram2_803EEAD0.header_pos,
            &raw mut sp18,
        );
    }
    let mut font: f32 = 0.045f32;
    (*text).font_size.x = font;
    (*text).font_size.y = font;
    (*text).text_color = mnDiagram2_804D4FB8;
    let mut ny: f32 = -sp18.y;
    let mut z: f32 = sp18.z;
    (*text).pos_x = sp18.x;
    (*text).pos_y = ny;
    (*text).pos_z = z;
    (*text).default_alignment = 1 as u8;
    if is_name_mode as i32 != 0 as i32 {
        HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, GetNameText(nametag_slot_or_selkind), &[]);
    } else {
        gm_80160B40(text, gm_SelKindToCKind(nametag_slot_or_selkind), 0 as u8);
    };
}
unsafe extern "C" fn getCurrentDiagramData() -> *mut Diagram2 {
    return (*mnDiagram2_804D6C18).user_data as *mut Diagram2;
}
unsafe extern "C" fn mnDiagram2_RefreshStatRows() {
    let mut d: *mut HSD_GObj = mnDiagram2_804D6C18;
    let mut data: *mut Diagram2 = getCurrentDiagramData();
    let mut entity_idx: u8 = 0;
    mnDiagram2_ClearStatRows(d);
    if (*data).is_name_mode as i32 != 0 as i32 {
        entity_idx = (*data).selected_name_idx;
    } else {
        entity_idx = (*data).selected_fighter_idx;
    }
    mnDiagram2_PopulateStatRows(
        d,
        (*data).scroll_offset as u8,
        (*data).is_name_mode,
        entity_idx,
    );
}
unsafe extern "C" fn saveDiagramSelection() {
    let mut data: *mut Diagram2 = (*mnDiagram2_804D6C18).user_data as *mut Diagram2;
    (*gmMainLib_GetGameRules()).x12 = (*data).selected_fighter_idx;
    (*gmMainLib_GetGameRules()).x13 = (*data).selected_name_idx;
    (*gmMainLib_GetGameRules()).xD = (*data).is_name_mode;
}
unsafe extern "C" fn updateDiagramMode(mut is_name_mode: u8) {
    let mut gobj: *mut HSD_GObj = mnDiagram2_804D6C18;
    let mut data: *mut Diagram2 = (*gobj).user_data as *mut Diagram2;
    let mut entity_idx: u8 = 0;
    if is_name_mode as i32 != 0 as i32 {
        HSD_JObjSetFlagsAll(
            (*data).fighter_mode_header as *mut HSD_JObj,
            JOBJ_HIDDEN as u32,
        );
        HSD_JObjClearFlagsAll(
            (*data).name_mode_header as *mut HSD_JObj,
            JOBJ_HIDDEN as u32,
        );
    } else {
        HSD_JObjClearFlagsAll(
            (*data).fighter_mode_header as *mut HSD_JObj,
            JOBJ_HIDDEN as u32,
        );
        HSD_JObjSetFlagsAll(
            (*data).name_mode_header as *mut HSD_JObj,
            JOBJ_HIDDEN as u32,
        );
    }
    if is_name_mode as i32 != 0 as i32 {
        entity_idx = (*data).selected_name_idx;
    } else {
        entity_idx = (*data).selected_fighter_idx;
    }
    mnDiagram2_UpdateHeader(gobj, is_name_mode, entity_idx);
}
pub unsafe extern "C" fn mnDiagram2_HandleInput(mut gobj: *mut HSD_GObj) {
    let mut data: *mut Diagram2 = ::core::ptr::null_mut::<Diagram2>();
    let mut result: i32 = 0;
    let mut entity_idx: u8 = 0;
    let mut selection: i32 = 0;
    let mut _pad: [u8; 40] = [0; 40];
    data = getCurrentDiagramData();
    mn_804A04F0.buttons = mn_80229624(4 as u32) as u64;
    result = mn_804A04F0.buttons as i32;
    if result & 0x20 as i32 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8;
        saveDiagramSelection();
        mn_80229894(0x1c as i32, 0 as u16, 3 as i32);
        mnDiagram2_ClearStatRows(mnDiagram2_804D6C18);
        return;
    }
    if result & 0xc0 as i32 != 0 {
        sfxForward();
        saveDiagramSelection();
        mnDiagram2_ClearStatRows(mnDiagram2_804D6C18);
        HSD_GObjFree(gobj);
        if result & 0x40 as i32 != 0 {
            mnDiagram_Init(0 as u8, 0 as u8);
            return;
        }
        mnDiagram3_Init(::core::ptr::null_mut::<c_void>());
        return;
    }
    if result & 0xc00 as i32 != 0 {
        if GetNameCount() == 0 as i32 {
            lbAudioAx_80024030(3 as i32);
            return;
        }
        sfxForward();
        (*data).is_name_mode = (if (*data).is_name_mode as i32
            == 0 as i32
        {
            1 as i32
        } else {
            0 as i32
        }) as u8;
        if (*data).is_name_mode as i32 == 0 as i32
            && (*data).scroll_offset as i32 + 10 as i32
                > 0x15 as i32
        {
            (*data).scroll_offset = 0 as u16;
        }
        mnDiagram2_RefreshStatRows();
        if (*data).is_name_mode as i32 != 0 as i32 {
            entity_idx = (*data).selected_name_idx;
        } else {
            entity_idx = (*data).selected_fighter_idx;
        }
        mnDiagram2_UpdateHeader(mnDiagram2_804D6C18, (*data).is_name_mode, entity_idx);
        updateDiagramMode((*data).is_name_mode);
        return;
    }
    if (*data).is_name_mode as i32 != 0 as i32 {
        GetNameCount();
        if result & 1 as i32 != 0 {
            if (*data).scroll_offset as i32 != 0 as i32 {
                sfxMove();
                (*data).scroll_offset = ((*data).scroll_offset as i32
                    - 1 as i32) as u16;
                mnDiagram2_RefreshStatRows();
            }
        } else if result & 2 as i32 != 0 {
            if ((*data).scroll_offset as i32 + 10 as i32)
                < 0x18 as i32
            {
                sfxMove();
                (*data).scroll_offset = ((*data).scroll_offset as i32
                    + 1 as i32) as u16;
                mnDiagram2_RefreshStatRows();
            }
        } else if result & 4 as i32 != 0 {
            selection = mnDiagram_GetPrevNameIndex((*data).selected_name_idx as i32)
                as i32;
            if (*data).selected_name_idx as i32 != selection {
                sfxMove();
                (*data).selected_name_idx = selection as u8;
                mnDiagram2_RefreshStatRows();
                mnDiagram2_UpdateHeader(
                    mnDiagram2_804D6C18,
                    (*data).is_name_mode,
                    (*data).selected_name_idx,
                );
            }
        } else if result & 8 as i32 != 0 {
            selection = mnDiagram_GetNextNameIndex((*data).selected_name_idx as i32)
                as i32;
            if (*data).selected_name_idx as i32 != selection {
                sfxMove();
                (*data).selected_name_idx = selection as u8;
                mnDiagram2_RefreshStatRows();
                mnDiagram2_UpdateHeader(
                    mnDiagram2_804D6C18,
                    (*data).is_name_mode,
                    (*data).selected_name_idx,
                );
            }
        }
    } else {
        mnDiagram_CountUnlockedFighters();
        if result & 1 as i32 != 0 {
            if (*data).scroll_offset as i32 != 0 as i32 {
                sfxMove();
                (*data).scroll_offset = ((*data).scroll_offset as i32
                    - 1 as i32) as u16;
                mnDiagram2_RefreshStatRows();
            }
        } else if result & 2 as i32 != 0 {
            if ((*data).scroll_offset as i32 + 10 as i32)
                < 0x15 as i32
            {
                sfxMove();
                (*data).scroll_offset = ((*data).scroll_offset as i32
                    + 1 as i32) as u16;
                mnDiagram2_RefreshStatRows();
            }
        } else if result & 4 as i32 != 0 {
            selection = mnDiagram_GetPrevFighterIndex(
                (*data).selected_fighter_idx as i32,
            ) as i32;
            if (*data).selected_fighter_idx as i32 != selection {
                sfxMove();
                (*data).selected_fighter_idx = selection as u8;
                mnDiagram2_RefreshStatRows();
                mnDiagram2_UpdateHeader(
                    mnDiagram2_804D6C18,
                    (*data).is_name_mode,
                    (*data).selected_fighter_idx,
                );
            }
        } else if result & 8 as i32 != 0 {
            selection = mnDiagram_GetNextFighterIndex(
                (*data).selected_fighter_idx as i32,
            ) as i32;
            if (*data).selected_fighter_idx as i32 != selection {
                sfxMove();
                (*data).selected_fighter_idx = selection as u8;
                mnDiagram2_RefreshStatRows();
                mnDiagram2_UpdateHeader(
                    mnDiagram2_804D6C18,
                    (*data).is_name_mode,
                    (*data).selected_fighter_idx,
                );
            }
        }
    };
}
pub unsafe extern "C" fn mnDiagram2_GetStatValue(
    mut is_name_mode: u8,
    mut stat_type: u8,
    mut entity_idx: u8,
) -> i32 {
    let mut typeVal: u8 = 0;
    let mut idxVal: u8 = 0;
    let mut val16: u16 = 0;
    typeVal = stat_type;
    idxVal = entity_idx;
    match typeVal as i32 {
        0 => {
            if is_name_mode != 0 {
                return mnDiagram_GetNameTotalKOs(idxVal);
            }
            return mnDiagram_GetFighterTotalKOs(idxVal);
        }
        1 => {
            if is_name_mode != 0 {
                return mnDiagram_GetNameTotalFalls(idxVal);
            }
            return mnDiagram_GetFighterTotalFalls(idxVal);
        }
        2 => {
            if is_name_mode != 0 {
                val16 = (*GetPersistentNameData(idxVal as i32)).stats.sd_count;
            } else {
                val16 = (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                    .stats
                    .sd_count;
            }
            return val16 as i32;
        }
        3 => {
            return mnDiagram_GetHitPercentage(is_name_mode, idxVal) as i32;
        }
        4 => {
            if is_name_mode != 0 {
                return (*GetPersistentNameData(idxVal as i32)).stats.damage_dealt
                    as i32;
            }
            return (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                .stats
                .damage_dealt as i32;
        }
        5 => {
            if is_name_mode != 0 {
                return (*GetPersistentNameData(idxVal as i32)).stats.damage_taken
                    as i32;
            }
            return (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                .stats
                .damage_taken as i32;
        }
        6 => {
            if is_name_mode != 0 {
                return (*GetPersistentNameData(idxVal as i32)).stats.damage_recovered
                    as i32;
            }
            return (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                .stats
                .damage_recovered as i32;
        }
        7 => {
            if is_name_mode != 0 {
                val16 = (*GetPersistentNameData(idxVal as i32)).stats.peak_damage;
            } else {
                val16 = (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                    .stats
                    .peak_damage;
            }
            return val16 as i32;
        }
        8 => {
            if is_name_mode != 0 {
                val16 = (*GetPersistentNameData(idxVal as i32)).stats.match_count;
            } else {
                val16 = (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                    .stats
                    .match_count;
            }
            return val16 as i32;
        }
        9 => {
            if is_name_mode != 0 {
                val16 = (*GetPersistentNameData(idxVal as i32)).stats.victories;
            } else {
                val16 = (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                    .stats
                    .victories;
            }
            return val16 as i32;
        }
        10 => {
            if is_name_mode != 0 {
                val16 = (*GetPersistentNameData(idxVal as i32)).stats.losses;
            } else {
                val16 = (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                    .stats
                    .losses;
            }
            return val16 as i32;
        }
        11 => {
            if is_name_mode != 0 {
                return (*GetPersistentNameData(idxVal as i32)).stats.play_time
                    as i32;
            }
            return (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                .stats
                .play_time as i32;
        }
        12 => {
            return mnDiagram_GetPlayPercentage(is_name_mode, idxVal)
                as i32;
        }
        13 => {
            return mnDiagram_GetAveragePlayerCount(is_name_mode, idxVal)
                as i32;
        }
        14 => {
            if is_name_mode != 0 {
                return (*GetPersistentNameData(idxVal as i32)).stats.walk_distance
                    as i32;
            }
            return (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                .stats
                .walk_distance as i32;
        }
        15 => {
            if is_name_mode != 0 {
                return (*GetPersistentNameData(idxVal as i32)).stats.run_distance
                    as i32;
            }
            return (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                .stats
                .run_distance as i32;
        }
        16 => {
            if is_name_mode != 0 {
                return (*GetPersistentNameData(idxVal as i32)).stats.fall_distance
                    as i32;
            }
            return (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                .stats
                .fall_distance as i32;
        }
        17 => {
            if is_name_mode != 0 {
                return (*GetPersistentNameData(idxVal as i32)).stats.peak_height
                    as i32;
            }
            return (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                .stats
                .peak_height as i32;
        }
        18 => {
            if is_name_mode != 0 {
                return (*GetPersistentNameData(idxVal as i32)).stats.coins_collected
                    as i32;
            }
            return (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                .stats
                .coins_collected as i32;
        }
        19 => {
            if is_name_mode != 0 {
                return (*GetPersistentNameData(idxVal as i32)).stats.coins_swiped
                    as i32;
            }
            return (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                .stats
                .coins_swiped as i32;
        }
        20 => {
            if is_name_mode != 0 {
                return (*GetPersistentNameData(idxVal as i32)).stats.coins_lost
                    as i32;
            }
            return (*GetPersistentFighterData(idxVal as SelectableCharacterKind))
                .stats
                .coins_lost as i32;
        }
        21 => {
            return mnDiagram_GetRankedFighterForName(
                0 as i32,
                idxVal as i32,
                ::core::mem::transmute::<
                    *mut c_void,
                    Option<
                        unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> u32,
                    >,
                >(
                    ::core::mem::transmute::<
                        Option<
                            unsafe extern "C" fn(
                                i32,
                                SelectableCharacterKind,
                            ) -> u32,
                        >,
                        *mut c_void,
                    >(
                        Some(
                            mnDiagram_GetNamePlayTimeByFighter
                                as unsafe extern "C" fn(
                                    i32,
                                    SelectableCharacterKind,
                                ) -> u32,
                        ),
                    ),
                ),
            );
        }
        22 => {
            return mnDiagram_GetRankedFighterForName(
                1 as i32,
                idxVal as i32,
                ::core::mem::transmute::<
                    *mut c_void,
                    Option<
                        unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> u32,
                    >,
                >(
                    ::core::mem::transmute::<
                        Option<
                            unsafe extern "C" fn(
                                i32,
                                SelectableCharacterKind,
                            ) -> u32,
                        >,
                        *mut c_void,
                    >(
                        Some(
                            mnDiagram_GetNamePlayTimeByFighter
                                as unsafe extern "C" fn(
                                    i32,
                                    SelectableCharacterKind,
                                ) -> u32,
                        ),
                    ),
                ),
            );
        }
        23 => return mnDiagram_GetLeastPlayedFighter(idxVal) as i32,
        _ => {}
    }
    panic!("Reached end of non-void function without returning");
}
pub unsafe extern "C" fn mnDiagram2_CreateStatRow(
    mut gobj: *mut HSD_GObj,
    mut is_name_mode: u8,
    mut stat_type: u8,
    mut row_idx: u8,
    mut entity_idx: u8,
) {
    let mut row: u32 = 0;
    let mut px: f32 = 0.;
    let mut py: f32 = 0.;
    let mut pz: f32 = 0.;
    let mut text2: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut unit_glyph_id: i32 = 0;
    let mut data: *mut Diagram2 = ::core::ptr::null_mut::<Diagram2>();
    let mut base: *mut MnDiagram2RowLayout = ::core::ptr::null_mut::<
        MnDiagram2RowLayout,
    >();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut f31: f32 = 0.;
    let mut str: [u8; 8] = [0; 8];
    let mut f30: f32 = 0.;
    let mut position: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut entity: u8 = entity_idx;
    data = (*gobj).user_data as *mut Diagram2;
    base = &raw mut mnDiagram2_803EEAD0;
    f31 = HSD_JObjGetTranslationY((*data).row0_ref as *mut HSD_JObj);
    f30 = HSD_JObjGetTranslationY((*data).row1_ref as *mut HSD_JObj) - f31;
    lb_8000B1CC(
        (*data).row0_ref as *mut HSD_JObj,
        &raw mut (*base).label_pos,
        &raw mut position,
    );
    let mut ny: f32 = 0.;
    let mut row_y_offset: f32 = 0.;
    row = row_idx as u32;
    ny = -position.y;
    row_y_offset = -f30 * row as f32;
    text = HSD_SisLib_803A5ACC(
        0 as i32,
        1 as i32,
        position.x,
        ny + row_y_offset,
        position.z,
        320.0f32,
        240.0f32,
    );
    (*data).row_labels[row_idx as usize] = text;
    HSD_SisLib_803A6368(text, (*base).label_ids[stat_type as usize] as i32);
    unit_glyph_id = (*base).unit_glyph_ids[stat_type as usize] as i32;
    if unit_glyph_id != 0xffff as i32 {
        lb_8000B1CC(
            (*data).row0_ref as *mut HSD_JObj,
            &raw mut (*base).label_pos,
            &raw mut position,
        );
        text2 = HSD_SisLib_803A5ACC(
            0 as i32,
            1 as i32,
            12.0f32 + position.x,
            -position.y + row_y_offset,
            position.z,
            1.0f32,
            1.0f32,
        );
        (*data).row_icons[row_idx as usize] = text2;
        (*text2).default_alignment = 1 as u8;
        (*text2).text_color = mnDiagram2_804D4FBC;
        if mnDiagram2_IsDistanceStat(stat_type)
            && mnDiagram_IsDistanceOverflow(
                mnDiagram2_GetStatValue(is_name_mode, stat_type, entity) as u32,
            ) != 0
        {
            HSD_SisLib_803A6368(text2, 0x7f as i32);
        } else {
            HSD_SisLib_803A6368(text2, unit_glyph_id as i32);
        }
    }
    if mnDiagram2_IsIconOnlyStat(stat_type)
        && (mnDiagram2_GetStatValue(is_name_mode, stat_type, entity) as u32)
            < SELKIND_COUNT as i32 as u32
    {
        jobj = mnDiagram_CreateFighterIcon(
            mnDiagram2_GetStatValue(is_name_mode, stat_type, entity),
            0 as i32,
        );
        HSD_JObjSetTranslateX(jobj, (*base).icon_pos.x);
        HSD_JObjSetTranslateY(jobj, f30 * row_idx as f32 + (*base).icon_pos.y);
        HSD_JObjSetTranslateZ(jobj, (*base).icon_pos.z);
        HSD_JObjAddChild((*data).icon_parent as *mut HSD_JObj, jobj);
        return;
    }
    text2 = HSD_SisLib_803A6754(0 as i32, 1 as i32);
    (*data).row_values[row_idx as usize] = text2;
    (*text2).font_size.x = 0.03f32 as f32;
    (*text2).font_size.y = 0.035f32 as f32;
    lb_8000B1CC(
        (*data).icon_parent as *mut HSD_JObj,
        &raw mut (*base).value_pos,
        &raw mut position,
    );
    px = position.x;
    py = -position.y + row_y_offset;
    pz = position.z;
    (*text2).pos_x = px;
    (*text2).pos_y = py;
    (*text2).pos_z = pz;
    (*text2).text_color = mnDiagram2_804D4FBC;
    (*text2).default_alignment = 2 as u8;
    if mnDiagram2_IsTimeStat(stat_type) {
        let mut val: i32 = mnDiagram2_GetStatValue(
            is_name_mode,
            stat_type,
            entity,
        );
        let mut unsigned_val: u32 = val as u32;
        if unsigned_val > 0x927bf as i32 as u32 {
            val = 0x927bf as i32;
        }
        mnDiagram_FormatTime(
            &raw mut str as *mut u8 as *mut i8,
            val as i32,
        );
    } else if mnDiagram2_IsDistanceStat(stat_type) {
        let mut val_0: u32 = mnDiagram2_GetStatValue(is_name_mode, stat_type, entity)
            as u32;
        val_0 = mnDiagram_ConvertDistanceForDisplay(val_0);
        if val_0 > 0x98967f as i32 as u32 {
            val_0 = 0x98967f as i32 as u32;
        }
        mnDiagram_IntToStr(&raw mut str as *mut u8 as *mut i8, val_0);
    } else if mnDiagram2_IsPercentageStat(stat_type) {
        let mut val_1: i32 = mnDiagram2_GetStatValue(
            is_name_mode,
            stat_type,
            entity,
        );
        if val_1 as u32 > 0xf423f as i32 as u32 {
            val_1 = 0xf423f as i32;
        }
        mnDiagram_FormatDecimalNumber(
            &raw mut str as *mut u8 as *mut i8,
            val_1 as u32,
            2 as i32,
        );
    } else if mnDiagram2_IsIconOnlyStat(stat_type) {
        str[0 as i32 as usize] = mnDiagram2_804D4FD0[0
            as i32 as usize];
        str[1 as i32 as usize] = mnDiagram2_804D4FD0[1
            as i32 as usize];
        str[2 as i32 as usize] = mnDiagram2_804D4FD0[2
            as i32 as usize];
    } else {
        let mut val_2: i32 = mnDiagram2_GetStatValue(
            is_name_mode,
            stat_type,
            entity,
        );
        if val_2 as u32 > 0x98967f as i32 as u32 {
            val_2 = 0x98967f as i32;
        }
        mnDiagram_FormatDecimalNumber(
            &raw mut str as *mut u8 as *mut i8,
            val_2 as u32,
            0 as i32,
        );
    }
    HSD_SisLib_803A6B98(text2, 0.0f32, 0.0f32, &raw mut str as *mut u8 as *mut i8, &[]);
}
pub unsafe extern "C" fn mnDiagram2_PopulateStatRows(
    mut gobj: *mut HSD_GObj,
    mut scroll_offset: u8,
    mut is_name_mode: u8,
    mut entity_idx: u8,
) {
    let mut limit: i32 = 0;
    let mut idx: i32 = 0;
    let mut i: i32 = 0;
    let mut var_r28: i32 = 0;
    if is_name_mode as i32 != 0 as i32 {
        var_r28 = mnDiagram_GetNameByIndex(entity_idx as i32)
            as i32;
    } else {
        var_r28 = mnDiagram_GetFighterByIndex(entity_idx as i32)
            as i32;
    }
    if is_name_mode as i32 != 0 as i32 {
        limit = 0x18 as i32;
    } else {
        limit = 0x15 as i32;
    }
    i = 0 as i32;
    idx = scroll_offset as i32;
    while i < 10 as i32 {
        let mut val: i32 = 0;
        if idx >= limit {
            val = idx - limit;
        } else {
            val = idx;
        }
        mnDiagram2_CreateStatRow(
            gobj,
            is_name_mode,
            val as u8,
            i as u8,
            var_r28 as u8,
        );
        i += 1;
        idx += 1;
    }
}
pub unsafe extern "C" fn mnDiagram2_OnAnimComplete(mut gobj: *mut HSD_GObj) {
    let mut data: *mut Diagram2 = ::core::ptr::null_mut::<Diagram2>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut table: *mut AnimLoopSettings = ::core::ptr::null_mut::<AnimLoopSettings>();
    data = HSD_GObjGetUserData(gobj) as *mut Diagram2;
    jobj = (*data).xC as *mut HSD_JObj;
    table = &raw mut mnDiagram2_803EEB60 as *mut AnimLoopSettings;
    if mn_8022ED6C(jobj as *mut HSD_JObj, table) >= (*table).end_frame {
        HSD_GObjFree(gobj);
    }
}
pub unsafe extern "C" fn mnDiagram2_UpdateScrollArrows(mut gobj: *mut HSD_GObj) {
    let mut data: *mut Diagram2 = ::core::ptr::null_mut::<Diagram2>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    data = HSD_GObjGetUserData(gobj) as *mut Diagram2;
    jobj = (*data).down_arrow as *mut HSD_JObj;
    mn_8022ED6C(
        jobj as *mut HSD_JObj,
        (&raw mut mnDiagram2_803EEB60 as *mut AnimLoopSettings)
            .offset(1 as i32 as isize) as *mut AnimLoopSettings,
    );
    if (*data).is_name_mode != 0 {
        if ((*data).scroll_offset as i32 + 10 as i32)
            < 0x18 as i32
        {
            HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        } else {
            HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    } else if ((*data).scroll_offset as i32 + 10 as i32)
        < 0x15 as i32
    {
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
    jobj = (*data).up_arrow as *mut HSD_JObj;
    mn_8022ED6C(
        jobj as *mut HSD_JObj,
        (&raw mut mnDiagram2_803EEB60 as *mut AnimLoopSettings)
            .offset(1 as i32 as isize) as *mut AnimLoopSettings,
    );
    if (*data).scroll_offset != 0 {
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
    jobj = (*data).left_arrow as *mut HSD_JObj;
    mn_8022ED6C(
        jobj as *mut HSD_JObj,
        (&raw mut mnDiagram2_803EEB60 as *mut AnimLoopSettings)
            .offset(1 as i32 as isize) as *mut AnimLoopSettings,
    );
    if (*data).is_name_mode != 0 {
        if (*data).selected_name_idx != 0 {
            HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        } else {
            HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    } else if (*data).selected_fighter_idx as i32
        != 0 as i32
    {
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
    jobj = (*data).right_arrow as *mut HSD_JObj;
    mn_8022ED6C(
        jobj as *mut HSD_JObj,
        (&raw mut mnDiagram2_803EEB60 as *mut AnimLoopSettings)
            .offset(1 as i32 as isize) as *mut AnimLoopSettings,
    );
    if (*data).is_name_mode as i32 != 0 as i32 {
        if (*data).selected_name_idx as i32
            != mnDiagram_GetNextNameIndex((*data).selected_name_idx as i32)
                as i32
        {
            HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        } else {
            HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    } else if (*data).selected_fighter_idx as i32
        != mnDiagram_GetNextFighterIndex((*data).selected_fighter_idx as i32)
            as i32
    {
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    };
}
pub unsafe extern "C" fn mnDiagram2_Think(mut gobj: *mut HSD_GObj) {
    let mut data: *mut Diagram2 = ::core::ptr::null_mut::<Diagram2>();
    let mut src: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut mode: u8 = 0;
    let mut _pad: [u8; 8] = [0; 8];
    src = &raw mut mn_804A04F0 as *mut u8;
    data = (*gobj).user_data as *mut Diagram2;
    mode = *src.offset(0 as i32 as isize);
    if mode as i32 != 0x1e as i32
        || *src.offset(0x10 as i32 as isize) as i32
            != 1 as i32
    {
        if mode as i32 == 0x1e as i32 {
            HSD_GObjFree(gobj);
        } else {
            let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
            HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
            proc_0 = HSD_GObj_SetupProc(
                gobj,
                Some(
                    mnDiagram2_OnAnimComplete
                        as unsafe extern "C" fn(*mut HSD_GObj) -> (),
                ),
                0 as u8,
            );
            (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
            HSD_JObjSetFlagsAll(
                *(data as *mut *mut HSD_JObj).offset(4 as i32 as isize),
                JOBJ_HIDDEN as u32,
            );
            if !(*data).header_text.is_null() {
                HSD_SisLib_803A5CC4((*data).header_text);
                (*data).header_text = ::core::ptr::null_mut::<HSD_Text>();
            }
            return;
        }
    }
    mnDiagram2_UpdateScrollArrows(gobj);
}
pub unsafe extern "C" fn mnDiagram2_FreeUserData(mut data: *mut Diagram2) {
    if !(*data).header_text.is_null() {
        HSD_SisLib_803A5CC4((*data).header_text);
    }
    HSD_Free(data as *mut c_void);
}
pub unsafe extern "C" fn mnDiagram2_InitUserData(
    mut arg: *mut c_void,
    mut unused: i32,
) {
    let mut data: *mut Diagram2 = arg as *mut Diagram2;
    let mut src: *mut u8 = &raw mut mn_804A04F0 as *mut u8;
    let mut i: i32 = 0;
    (*data).saved_menu = *src.offset(0 as i32 as isize);
    (*data).saved_selection = *(src.offset(2 as i32 as isize)
        as *mut u16);
    (*data).anim_state = 1 as u8;
    (*data).scroll_offset = 0 as u16;
    (*data).selected_fighter_idx = 0 as u8;
    (*data).selected_name_idx = 0 as u8;
    (*data).is_name_mode = (*gmMainLib_GetGameRules()).xD;
    i = 0 as i32;
    while i < 10 as i32 {
        (*data).row_labels[i as usize] = ::core::ptr::null_mut::<HSD_Text>();
        (*data).row_values[i as usize] = ::core::ptr::null_mut::<HSD_Text>();
        (*data).row_icons[i as usize] = ::core::ptr::null_mut::<HSD_Text>();
        i += 1;
    }
    (*data).header_text = ::core::ptr::null_mut::<HSD_Text>();
}
pub unsafe extern "C" fn mnDiagram2_Create(mut arg0: i32) {
    let mut entity_val: i32 = 0;
    let mut user_data: *mut Diagram2 = ::core::ptr::null_mut::<Diagram2>();
    let mut entity_idx: u8 = 0;
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut new_var: *mut Diagram2 = ::core::ptr::null_mut::<Diagram2>();
    let mut j: i32 = 0;
    let mut threshold: i32 = 0;
    let mut archive: *mut StaticModelDesc = &raw mut MenMainConB2_Top;
    let mut scroll: i32 = 0;
    let mut i: i32 = 0;
    let mut is_name: u32 = 0;
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut offset: i32 = 0;
    let mut user_data2: *mut Diagram2 = ::core::ptr::null_mut::<Diagram2>();
    let mut cursor: *mut *mut HSD_JObj = ::core::ptr::null_mut::<*mut HSD_JObj>();
    gobj = GObj_Create(6 as u16, 7 as u8, 0x80 as u8);
    mnDiagram2_804D6C18 = gobj;
    jobj = HSD_JObjLoadJoint((*archive).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, i32) -> (),
        ),
        6 as u8,
        0x80 as u32,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*archive).animjoint,
        (*archive).matanim_joint,
        (*archive).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0f32);
    user_data = HSD_MemAlloc(::core::mem::size_of::<Diagram2>() as ssize_t)
        as *mut Diagram2;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const i8
                as *mut i8, &[]);
        __assert(
            b"src/melee/mn/mndiagram2.c\0" as *const u8 as *const i8,
            986 as u32,
            b"user_data\0" as *const u8 as *const i8,
        );
    };
    mnDiagram2_InitUserData(user_data as *mut c_void, arg0);
    GObj_InitUserData(
        gobj,
        0 as u8,
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut Diagram2) -> ()>,
            Option<unsafe extern "C" fn(*mut c_void) -> ()>,
        >(Some(mnDiagram2_FreeUserData as unsafe extern "C" fn(*mut Diagram2) -> ())),
        user_data as *mut c_void,
    );
    offset = 0 as i32;
    i = offset;
    cursor = (user_data as *mut *mut HSD_JObj).offset(i as isize);
    loop {
        lb_80011E24(jobj as *mut HSD_JObj, cursor.offset(2 as i32 as isize), &[Va::I32(i), Va::I32(-(1 as i32))]);
        i += 1;
        cursor = cursor.offset(1);
        if !(i < 15 as i32) {
            break;
        }
    }
    HSD_GObj_SetupProc(
        gobj,
        Some(mnDiagram2_Think as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
    new_var = user_data;
    is_name = (*user_data).is_name_mode as u32;
    if is_name != 0 as u32 {
        entity_idx = (*new_var).selected_name_idx;
    } else {
        entity_idx = (*new_var).selected_fighter_idx;
    }
    scroll = (*user_data).scroll_offset as i32
        & 0xff as i32;
    if is_name != 0 {
        entity_val = mnDiagram_GetNameByIndex(entity_idx as i32)
            as i32;
    } else {
        entity_val = mnDiagram_GetFighterByIndex(entity_idx as i32)
            as i32;
    }
    if is_name != 0 as u32 {
        threshold = 0x18 as i32;
    } else {
        threshold = 0x15 as i32;
    }
    j = 0 as i32;
    while j < 10 as i32 {
        if scroll >= threshold {
            offset = scroll - threshold;
        } else {
            offset = scroll;
        }
        mnDiagram2_CreateStatRow(
            gobj,
            is_name as u8,
            offset as u8,
            j as u8,
            entity_val as u8,
        );
        scroll += 1;
        j += 1;
    }
    let mut mode: u32 = (*user_data).is_name_mode as u32;
    is_name = mode;
    user_data2 = (*gobj).user_data as *mut Diagram2;
    if is_name != 0 {
        HSD_JObjSetFlagsAll(
            (*user_data2).fighter_mode_header as *mut HSD_JObj,
            JOBJ_HIDDEN as u32,
        );
        HSD_JObjClearFlagsAll(
            (*user_data2).name_mode_header as *mut HSD_JObj,
            JOBJ_HIDDEN as u32,
        );
    } else {
        HSD_JObjClearFlagsAll(
            (*user_data2).fighter_mode_header as *mut HSD_JObj,
            JOBJ_HIDDEN as u32,
        );
        HSD_JObjSetFlagsAll(
            (*user_data2).name_mode_header as *mut HSD_JObj,
            JOBJ_HIDDEN as u32,
        );
    }
    if is_name as u8 != 0 {
        entity_idx = (*user_data2).selected_name_idx;
    } else {
        entity_idx = (*user_data2).selected_fighter_idx;
    }
    mnDiagram2_UpdateHeader(gobj, is_name as u8, entity_idx);
}
pub unsafe extern "C" fn mnDiagram2_Init() {
    let mut data: *mut MenuFlow = &raw mut mn_804A04F0;
    let mut gobj: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    (*data).x10 = 1 as u8;
    (*data).hovered_selection = 0 as u16;
    mnDiagram2_Create(0 as i32);
    gobj = HSD_GObj_SetupProc(
        GObj_Create(0 as u16, 1 as u8, 0x80 as u8),
        Some(mnDiagram2_HandleInput as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
    (*gobj).set_flags_3(HSD_GObj_804D783C as u8 as u8);
}
pub unsafe extern "C" fn mnDiagram2_GetRankedFighter(
    mut stat_type: u8,
    mut rank: u8,
) -> u8 {
    let mut j: i32 = 0;
    let mut entries: [mnDiagram2_SortEntry; 25] = [mnDiagram2_SortEntry {
        anon_0: mnDiagram2_SortEntry_anon_9 {
            idx: 0,
            pad1: [0; 7],
            x8: 0,
            xC: 0,
        },
    }; 25];
    let mut temp: mnDiagram2_SortEntry = mnDiagram2_SortEntry {
        anon_0: mnDiagram2_SortEntry_anon_9 {
            idx: 0,
            pad1: [0; 7],
            x8: 0,
            xC: 0,
        },
    };
    let mut i: i32 = 0;
    let mut k: i32 = 0;
    let mut maxIdx: i32 = 0;
    let mut neg1: i32 = 0;
    let mut selkind: SelectableCharacterKind = SELKIND_CAPTAIN;
    i = 0 as i32;
    neg1 = -(1 as i32);
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        selkind = mnDiagram_GetFighterByIndex(i) as SelectableCharacterKind;
        entries[i as usize].anon_0.idx = selkind as u8;
        if mn_IsFighterUnlocked(selkind) != 0 as i32 {
            entries[i as usize].anon_0.xC = mnDiagram2_GetStatValue(
                0 as u8,
                stat_type,
                selkind as u8,
            ) as i32;
            entries[i as usize].anon_0.x8 = 0 as i32 as i32;
        } else {
            entries[i as usize].anon_0.xC = neg1 as i32;
            entries[i as usize].anon_0.x8 = neg1 as i32;
        }
        i += 1;
    }
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        k = i + 1 as i32;
        maxIdx = i;
        while k < SELKIND_COUNT as i32 {
            if entry_value(&entries[k as usize]) != neg1 as u64 {
                if entry_value(&entries[k as usize])
                    > entry_value(&entries[maxIdx as usize])
                    || entry_value(&entries[i as usize]) == neg1 as u64
                {
                    maxIdx = k;
                }
            }
            k += 1;
        }
        if maxIdx != i {
            temp = entries[maxIdx as usize];
            j = maxIdx;
            while j > i {
                entries[j as usize] = entries[(j - 1 as i32) as usize];
                j -= 1;
            }
            entries[i as usize] = temp;
        }
        i += 1;
    }
    if entry_value(&entries[rank as usize])
        == -(1 as i32) as u64
    {
        return SELKIND_COUNT as i32 as u8;
    }
    return entries[rank as usize].anon_0.idx;
}
pub unsafe extern "C" fn mnDiagram2_GetRankedName(
    mut stat_type: u8,
    mut rank: u8,
) -> u8 {
    let mut entries: [mnDiagram2_SortEntry; 120] = [mnDiagram2_SortEntry {
        anon_0: mnDiagram2_SortEntry_anon_9 {
            idx: 0,
            pad1: [0; 7],
            x8: 0,
            xC: 0,
        },
    }; 120];
    let mut temp: mnDiagram2_SortEntry = mnDiagram2_SortEntry {
        anon_0: mnDiagram2_SortEntry_anon_9 {
            idx: 0,
            pad1: [0; 7],
            x8: 0,
            xC: 0,
        },
    };
    let mut base: *mut mnDiagram2_SortEntry = ::core::ptr::null_mut::<
        mnDiagram2_SortEntry,
    >();
    let mut ptr: *mut mnDiagram2_SortEntry = ::core::ptr::null_mut::<
        mnDiagram2_SortEntry,
    >();
    let mut count: i32 = 0;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut maxIdx: i32 = 0;
    let mut zero: i32 = 0;
    count = GetNameCount();
    if rank as i32 >= count {
        return 0x78 as u8;
    }
    base = &raw mut entries as *mut mnDiagram2_SortEntry;
    ptr = base;
    i = 0 as i32;
    zero = 0 as i32;
    while i < count {
        (*ptr).anon_0.idx = mnDiagram_GetNameByIndex(i);
        (*ptr).anon_0.xC = mnDiagram2_GetStatValue(
            1 as u8,
            stat_type,
            mnDiagram_GetNameByIndex(i),
        ) as i32;
        i += 1;
        (*ptr).anon_0.x8 = zero as i32;
        ptr = ptr.offset(1);
    }
    i = 0 as i32;
    while i < count {
        j = i + 1 as i32;
        maxIdx = i;
        while j < count {
            if entry_value(&entries[maxIdx as usize])
                < entry_value(&entries[j as usize])
            {
                maxIdx = j;
            }
            j += 1;
        }
        if maxIdx != i {
            ptr = (&raw mut entries as *mut mnDiagram2_SortEntry).offset(maxIdx as isize)
                as *mut mnDiagram2_SortEntry;
            temp = *ptr;
            j = maxIdx;
            while j > i {
                *ptr = *ptr.offset(-(1 as i32 as isize));
                ptr = ptr.offset(-1);
                j -= 1;
            }
            *base = temp;
        }
        base = base.offset(1);
        i += 1;
    }
    return entries[rank as usize].anon_0.idx;
}
pub unsafe extern "C" fn mnDiagram2_GetAggregatedFighterRank(
    mut out: *mut mnDiagram2_SortEntry,
    mut type_0: u8,
    mut idx: u8,
) {
    let mut entries: [mnDiagram2_SortEntry; 25] = [mnDiagram2_SortEntry {
        anon_0: mnDiagram2_SortEntry_anon_9 {
            idx: 0,
            pad1: [0; 7],
            x8: 0,
            xC: 0,
        },
    }; 25];
    let mut temp: mnDiagram2_SortEntry = mnDiagram2_SortEntry {
        anon_0: mnDiagram2_SortEntry_anon_9 {
            idx: 0,
            pad1: [0; 7],
            x8: 0,
            xC: 0,
        },
    };
    let mut base: *mut mnDiagram2_SortEntry = ::core::ptr::null_mut::<
        mnDiagram2_SortEntry,
    >();
    let mut curr: *mut mnDiagram2_SortEntry = ::core::ptr::null_mut::<
        mnDiagram2_SortEntry,
    >();
    let mut count: i32 = 0;
    let mut ptr: *mut mnDiagram2_SortEntry = ::core::ptr::null_mut::<
        mnDiagram2_SortEntry,
    >();
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut k: i32 = 0;
    let mut zero: i32 = 0;
    let mut n: i32 = 0;
    let mut m: i32 = 0;
    let mut res: i32 = 0;
    base = &raw mut entries as *mut mnDiagram2_SortEntry;
    ptr = base;
    n = 0 as i32;
    zero = 0 as i32;
    loop {
        (*ptr).anon_0.idx = mnDiagram_GetFighterByIndex(n);
        n += 1;
        (*ptr).anon_0.xC = zero as i32;
        (*ptr).anon_0.x8 = zero as i32;
        ptr = ptr.offset(1);
        if !(n < SELKIND_COUNT as i32) {
            break;
        }
    }
    count = GetNameCount();
    i = 0 as i32;
    while i < count {
        match type_0 as i32 {
            21 => {
                res = mnDiagram_GetRankedFighterForName(
                    0 as i32,
                    i,
                    ::core::mem::transmute::<
                        *mut c_void,
                        Option<
                            unsafe extern "C" fn(
                                i32,
                                i32,
                            ) -> u32,
                        >,
                    >(
                        ::core::mem::transmute::<
                            Option<
                                unsafe extern "C" fn(
                                    i32,
                                    SelectableCharacterKind,
                                ) -> u32,
                            >,
                            *mut c_void,
                        >(
                            Some(
                                mnDiagram_GetNamePlayTimeByFighter
                                    as unsafe extern "C" fn(
                                        i32,
                                        SelectableCharacterKind,
                                    ) -> u32,
                            ),
                        ),
                    ),
                );
            }
            22 => {
                res = mnDiagram_GetRankedFighterForName(
                    1 as i32,
                    i,
                    ::core::mem::transmute::<
                        *mut c_void,
                        Option<
                            unsafe extern "C" fn(
                                i32,
                                i32,
                            ) -> u32,
                        >,
                    >(
                        ::core::mem::transmute::<
                            Option<
                                unsafe extern "C" fn(
                                    i32,
                                    SelectableCharacterKind,
                                ) -> u32,
                            >,
                            *mut c_void,
                        >(
                            Some(
                                mnDiagram_GetNamePlayTimeByFighter
                                    as unsafe extern "C" fn(
                                        i32,
                                        SelectableCharacterKind,
                                    ) -> u32,
                            ),
                        ),
                    ),
                );
            }
            23 => {
                res = mnDiagram_GetLeastPlayedFighter(i as u8) as i32;
            }
            _ => {}
        }
        if res != SELKIND_COUNT as i32 {
            ptr = base;
            k = 0 as i32;
            m = SELKIND_COUNT as i32;
            while m > 0 as i32 {
                if res == (*ptr).anon_0.idx as i32 {
                    let nv = entry_value(&entries[k as usize]).wrapping_add(1 as u64);
                    entries[k as usize].anon_0.x8 = (nv >> 32) as u32 as i32;
                    entries[k as usize].anon_0.xC = nv as u32 as i32;
                    break;
                } else {
                    ptr = ptr.offset(1);
                    k += 1;
                    m -= 1;
                }
            }
        }
        i += 1;
    }
    j = 0 as i32;
    while j < SELKIND_COUNT as i32 {
        k = j + 1 as i32;
        curr = (&raw mut entries as *mut mnDiagram2_SortEntry).offset(k as isize)
            as *mut mnDiagram2_SortEntry;
        while k < SELKIND_COUNT as i32 {
            if entry_value(&*base) < entry_value(&*curr) {
                temp = *base;
                *base = *curr;
                *curr = temp;
            }
            curr = curr.offset(1);
            k += 1;
        }
        base = base.offset(1);
        j += 1;
    }
    if entry_value(&entries[idx as usize]) != 0 as u64 {
        *out = entries[idx as usize];
    } else {
        entries[idx as usize].anon_0.idx = SELKIND_COUNT as i32
            as u8;
        *out = entries[idx as usize];
    };
}
pub unsafe extern "C" fn mnDiagram2_ClearDetailView(mut gobj: *mut HSD_GObj) {
    let mut data: *mut Diagram3 = ::core::ptr::null_mut::<Diagram3>();
    let mut ptr: *mut Diagram3 = ::core::ptr::null_mut::<Diagram3>();
    let mut i: i32 = 0;
    let mut tmp: *mut c_void = ::core::ptr::null_mut::<
        c_void,
    >();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    data = HSD_GObjGetUserData(gobj) as *mut Diagram3;
    ptr = data;
    if !(*data).title_text.is_null() {
        HSD_SisLib_803A5CC4((*data).title_text);
        (*data).title_text = ::core::ptr::null_mut::<HSD_Text>();
    }
    i = 0 as i32;
    while i < 5 as i32 {
        if !(*data).row_icons[i as usize].is_null() {
            HSD_SisLib_803A5CC4((*ptr).row_icons[i as usize]);
            (*data).row_icons[i as usize] = ::core::ptr::null_mut::<HSD_Text>();
        }
        i += 1;
    }
    if !(*data).value_text.is_null() {
        HSD_SisLib_803A5CC4((*data).value_text);
        (*data).value_text = ::core::ptr::null_mut::<HSD_Text>();
    }
    tmp = (*data).jobjs[6 as i32 as usize] as *mut c_void;
    if tmp.is_null() {
        tmp = ::core::ptr::null_mut::<c_void>();
    } else {
        tmp = (*(*data).jobjs[6 as i32 as usize]).child
            as *mut c_void;
    }
    jobj = tmp as *mut HSD_JObj;
    if !jobj.is_null() {
        HSD_JObjRemoveAll(jobj);
    }
}
