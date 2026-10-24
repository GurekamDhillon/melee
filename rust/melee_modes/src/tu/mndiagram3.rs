//! TARGET_PC port of src/melee/mn/mndiagram3.c, checked against the native C state oracle.
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
fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: ::core::ffi::c_int)  -> ();
fn lbAudioAx_80024030(a0: ::core::ffi::c_int)  -> ();
fn mn_80229624(a0: u32)  -> u32;
fn mn_80229894(a0: i32, a1: u16, a2: i32)  -> ();
fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings)  -> ::core::ffi::c_float;
fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32)  -> ();
fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj)  -> ();
fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj)  -> ();
fn HSD_JObjLoadJoint(a0: *mut HSD_Joint)  -> *mut HSD_JObj;
fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint,)  -> ();
fn HSD_JObjAnimAll(a0: *mut HSD_JObj)  -> ();
fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32)  -> ();
fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, flags: u32)  -> ();
fn HSD_JObjAddChild(jobj: *mut HSD_JObj, child: *mut HSD_JObj)  -> ();
fn HSD_SisLib_803A6754(arg0: ::core::ffi::c_int, arg1: ::core::ffi::c_int,)  -> *mut HSD_Text;
fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32)  -> ();
fn HSD_SisLib_803A5ACC(a0: ::core::ffi::c_int, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32,)  -> *mut HSD_Text;
fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text)  -> ();
fn mnDiagram_IsDistanceOverflow(distance: u32)  -> i32;
fn mnDiagram_ConvertDistanceForDisplay(distance: u32)  -> u32;
fn mnDiagram_FormatDecimalNumber(buf: *mut ::core::ffi::c_char, val: u32, decimal_places: ::core::ffi::c_int,)  -> ();
fn mnDiagram_FormatTime(buf: *mut ::core::ffi::c_char, seconds: i32)  -> ();
fn mnDiagram_IntToStr(buf: *mut ::core::ffi::c_char, val: u32)  -> ();
fn mnDiagram_CreateFighterIcon(idx: ::core::ffi::c_int, arg1: ::core::ffi::c_int,)  -> *mut HSD_JObj;
fn mnDiagram_Init(arg0: u8, arg1: u8)  -> ();
fn mnDiagram2_IsTimeStat(stat_type: u8)  -> i32;
fn mnDiagram2_IsDistanceStat(stat_type: u8)  -> i32;
fn mnDiagram2_IsPercentageStat(stat_type: u8)  -> i32;
fn mnDiagram2_IsIconOnlyStat(stat_type: u8)  -> i32;
fn mnDiagram2_GetStatValue(is_name_mode: u8, stat_type: u8, entity_idx: u8,)  -> ::core::ffi::c_int;
fn mnDiagram2_Init()  -> ();
fn mnDiagram2_GetRankedFighter(stat_type: u8, rank: u8)  -> u8;
fn mnDiagram2_GetRankedName(stat_type: u8, rank: u8)  -> u8;
fn mnDiagram2_GetAggregatedFighterRank(out: *mut mnDiagram2_SortEntry, type_0: u8, idx: u8,)  -> ();
fn mnDiagram2_ClearDetailView(gobj: *mut HSD_GObj)  -> ();
fn GetNameText(slot: ::core::ffi::c_int)  -> *mut ::core::ffi::c_char;
fn GetNameCount()  -> ::core::ffi::c_int;
fn gmMainLib_GetGameRules()  -> *mut GameRules;
fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3)  -> ();
fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32,)  -> ();
fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut ::core::ffi::c_void,)  -> ();
fn HSD_GObjFree(a0: *mut HSD_GObj)  -> ();
fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8,)  -> *mut HSD_GObjProc;
fn HSD_GObjProc_RemoveProc(gproc: *mut HSD_GObjProc)  -> ();
fn GObj_InitUserData(gobj:*mut HSD_GObj, kind:u8, remove_func:Option<unsafe extern "C" fn(*mut core::ffi::c_void)>, data:*mut core::ffi::c_void)->();
fn HSD_Free(ptr: *mut ::core::ffi::c_void)  -> ();
fn HSD_MemAlloc(size: i32)  -> *mut ::core::ffi::c_void;
}
crate::ext_va! { fn OSReport(fmt: *const i8) va 0 -> (); fn HSD_SisLib_803A6B98(t: *mut HSD_Text, x:f32,y:f32,fmt:*const i8) va 3 -> i32; }
crate::ext_vk! { fn lb_80011E24(j:*mut HSD_JObj,out:*mut *mut HSD_JObj) vk "ii" -> i32; }
}
use ext::*;
crate::globals! {
rw pub mnDiagram3_804D6C20: *mut HSD_GObj  = ::core::ptr::null::<HSD_GObj>()
    as *mut HSD_GObj;
rw pub mnDiagram3_803EEC10: AnimLoopSettings  = AnimLoopSettings {
    start_frame: 10.0f32,
    end_frame: 19.0f32,
    loop_frame: -0.1f32,
};
rw pub mnDiagram3_803EEC1C: AnimLoopSettings  = AnimLoopSettings {
    start_frame: 0.0f32,
    end_frame: 199.0f32,
    loop_frame: 0.0f32,
};
rw pub mnDiagram3_803EEC28: mnDiagram3_PosTable  = mnDiagram3_PosTable {
    x0: Vec3 {
        x: 3.3f32,
        y: 0.5f32,
        z: 0.0f32,
    },
    xC: Vec3 {
        x: -2.0f32,
        y: 0.57f32,
        z: 0.0f32,
    },
    x18: Vec3 {
        x: 8.0f32,
        y: 0.57f32,
        z: 0.0f32,
    },
};
rw pub mnDiagram3_803EEC4C: mnDiagram3_StatTable  = mnDiagram3_StatTable {
    label_ids: [
        0x62 as ::core::ffi::c_int as u16,
        0x63 as ::core::ffi::c_int as u16,
        0x64 as ::core::ffi::c_int as u16,
        0x65 as ::core::ffi::c_int as u16,
        0x66 as ::core::ffi::c_int as u16,
        0x67 as ::core::ffi::c_int as u16,
        0x68 as ::core::ffi::c_int as u16,
        0x69 as ::core::ffi::c_int as u16,
        0x6a as ::core::ffi::c_int as u16,
        0x6b as ::core::ffi::c_int as u16,
        0x6c as ::core::ffi::c_int as u16,
        0x6d as ::core::ffi::c_int as u16,
        0x6e as ::core::ffi::c_int as u16,
        0x6f as ::core::ffi::c_int as u16,
        0x70 as ::core::ffi::c_int as u16,
        0x71 as ::core::ffi::c_int as u16,
        0x72 as ::core::ffi::c_int as u16,
        0x73 as ::core::ffi::c_int as u16,
        0x74 as ::core::ffi::c_int as u16,
        0x75 as ::core::ffi::c_int as u16,
        0x76 as ::core::ffi::c_int as u16,
        0x77 as ::core::ffi::c_int as u16,
        0x78 as ::core::ffi::c_int as u16,
        0x79 as ::core::ffi::c_int as u16,
    ],
    unit_glyph_ids: [
        0x7a as ::core::ffi::c_int as u16,
        0x7a as ::core::ffi::c_int as u16,
        0x7a as ::core::ffi::c_int as u16,
        0x7c as ::core::ffi::c_int as u16,
        0x7c as ::core::ffi::c_int as u16,
        0x7c as ::core::ffi::c_int as u16,
        0x7c as ::core::ffi::c_int as u16,
        0x7c as ::core::ffi::c_int as u16,
        0x7a as ::core::ffi::c_int as u16,
        0x7a as ::core::ffi::c_int as u16,
        0x7a as ::core::ffi::c_int as u16,
        0xffff as ::core::ffi::c_int as u16,
        0x7c as ::core::ffi::c_int as u16,
        0x7b as ::core::ffi::c_int as u16,
        0x7e as ::core::ffi::c_int as u16,
        0x7e as ::core::ffi::c_int as u16,
        0x7e as ::core::ffi::c_int as u16,
        0x7e as ::core::ffi::c_int as u16,
        0x7d as ::core::ffi::c_int as u16,
        0x7d as ::core::ffi::c_int as u16,
        0x7d as ::core::ffi::c_int as u16,
        0x7b as ::core::ffi::c_int as u16,
        0x7b as ::core::ffi::c_int as u16,
        0x7b as ::core::ffi::c_int as u16,
    ],
};
rw pub HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = unsafe {core::mem::zeroed()};
rw pub HSD_GObj_804D783C: i32 = unsafe {core::mem::zeroed()};
rw pub HSD_GObj_JObjKind: u8 = unsafe {core::mem::zeroed()};
rw pub mn_804A04F0: MenuFlow = unsafe {core::mem::zeroed()};
rw pub MenMainConB3_Top: StaticModelDesc = unsafe {core::mem::zeroed()};
rw pub MenMainCursorB3_Top: StaticModelDesc = unsafe {core::mem::zeroed()};
rw pub mn_804D4B64: GXColor = unsafe {core::mem::zeroed()};
}
pub unsafe fn Menu_GetAllInputs()->u64 { mn_804A04F0.buttons=mn_80229624(4) as u64;mn_804A04F0.buttons }

pub unsafe extern "C" fn mnDiagram3_PopulateRankings(mut gobj: *mut HSD_GObj) {
    let mut position: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut max_time: u32 = 0;
    let mut sp58: [u8; 16] = [0; 16];
    let mut first_rank: mnDiagram2_SortEntry = mnDiagram2_SortEntry {
        anon_0: mnDiagram2_SortEntry_anon_9 {
            idx: 0,
            pad1: [0; 7],
            x8: 0,
            xC: 0,
        },
    };
    let mut fighter_rank: mnDiagram2_SortEntry = mnDiagram2_SortEntry {
        anon_0: mnDiagram2_SortEntry_anon_9 {
            idx: 0,
            pad1: [0; 7],
            x8: 0,
            xC: 0,
        },
    };
    let mut rank_value: mnDiagram2_SortEntry = mnDiagram2_SortEntry {
        anon_0: mnDiagram2_SortEntry_anon_9 {
            idx: 0,
            pad1: [0; 7],
            x8: 0,
            xC: 0,
        },
    };
    let mut data: *mut Diagram3 = ::core::ptr::null_mut::<Diagram3>();
    let mut table: *mut mnDiagram3_DataTable = ::core::ptr::null_mut::<
        mnDiagram3_DataTable,
    >();
    let mut row0: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut neg_spacing: f32 = 0.;
    let mut row_spacing: f32 = 0.;
    let mut divider: f32 = 0.;
    let mut new_var: ::core::ffi::c_float = 0.;
    let mut stat_type: u8 = 0;
    let mut rank: u8 = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut title_text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut value_text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut icon: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut entity: i32 = 0;
    let mut max_percentage: u32 = 0;
    let mut unit_glyph_ids: *mut u16 = ::core::ptr::null_mut::<u16>();
    data = (*gobj).user_data as *mut Diagram3;
    table = &raw mut mnDiagram3_803EEC10 as *mut mnDiagram3_DataTable;
    let mut offset: u8 = 0;
    let mut scroll: u8 = 0;
    let mut limit: u8 = 0;
    scroll = (*data).cursor_row;
    offset = (*data).scroll_offset;
    limit = (if (*data).is_name_mode as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        0x18 as ::core::ffi::c_int
    } else {
        0x15 as ::core::ffi::c_int
    }) as u8;
    let mut sum: ::core::ffi::c_int = offset as ::core::ffi::c_int
        + scroll as ::core::ffi::c_int;
    let mut val: ::core::ffi::c_int = sum;
    if val >= limit as ::core::ffi::c_int {
        val = val - limit as ::core::ffi::c_int;
    } else {
        val = val as u8 as ::core::ffi::c_int;
    }
    row0 = (*data).jobjs[6 as ::core::ffi::c_int as usize] as *mut HSD_JObj;
    stat_type = val as u8;
    divider = HSD_JObjGetTranslationY(row0);
    row_spacing = HSD_JObjGetTranslationY(
        (*data).jobjs[7 as ::core::ffi::c_int as usize],
    );
    let mut max_distance: u32 = 0;
    unit_glyph_ids = &raw mut mnDiagram3_803EEC4C.unit_glyph_ids as *mut u16;
    row_spacing = row_spacing - divider;
    max_distance = 0x5f5e0ff as ::core::ffi::c_int as u32;
    max_percentage = 0x98967f as ::core::ffi::c_int as u32;
    max_time = 0x5b8d7f as ::core::ffi::c_int as u32;
    neg_spacing = -row_spacing;
    let mut current_block_96: u64;
    i = 0 as ::core::ffi::c_int;
    while i < 5 as ::core::ffi::c_int {
        if (*data).is_name_mode as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if mnDiagram2_IsIconOnlyStat(stat_type) == 0 {
                if i == 0 as ::core::ffi::c_int {
                    lb_8000B1CC(
                        (*data).jobjs[6 as ::core::ffi::c_int as usize],
                        &raw mut mnDiagram3_803EEC28.xC,
                        &raw mut position,
                    );
                    title_text = HSD_SisLib_803A6754(
                        0 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                    (*data).title_text = title_text;
                    (*title_text).font_size.x = 0.035f32 as f32;
                    (*title_text).font_size.y = 0.035f32 as f32;
                    let mut y: f32 = position.y;
                    let mut z: f32 = position.z;
                    (*title_text).pos_x = position.x;
                    new_var = -y as ::core::ffi::c_float;
                    (*title_text).pos_y = new_var as f32;
                    (*title_text).pos_z = z;
                }
                entity = mnDiagram2_GetRankedName(stat_type, i as u8) as i32;
                if entity == 0x78 as ::core::ffi::c_int {
                    current_block_96 = 14763689060501151050;
                } else {
                    HSD_SisLib_803A6B98(title_text, 0.0f32, neg_spacing as ::core::ffi::c_float * i as ::core::ffi::c_float
                            / 0.035f32, GetNameText(entity as ::core::ffi::c_int), &[]);
                    current_block_96 = 3160140712158701372;
                }
            } else {
                current_block_96 = 3160140712158701372;
            }
        } else {
            entity = mnDiagram2_GetRankedFighter(stat_type, i as u8) as i32;
            icon = mnDiagram_CreateFighterIcon(
                entity as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            HSD_JObjSetTranslateY(icon, row_spacing * i as f32);
            HSD_JObjAddChild((*data).jobjs[6 as ::core::ffi::c_int as usize], icon);
            current_block_96 = 3160140712158701372;
        }
        match current_block_96 {
            3160140712158701372 => {
                if i == 0 as ::core::ffi::c_int {
                    lb_8000B1CC(
                        (*data).jobjs[6 as ::core::ffi::c_int as usize],
                        &raw mut mnDiagram3_803EEC28.x18,
                        &raw mut position,
                    );
                    value_text = HSD_SisLib_803A6754(
                        0 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                    (*data).value_text = value_text;
                    (*value_text).font_size.x = 0.035f32 as f32;
                    (*value_text).font_size.y = 0.035f32 as f32;
                    let mut y_0: f32 = position.y;
                    let mut z_0: f32 = position.z;
                    (*value_text).pos_x = position.x;
                    new_var = -y_0 as ::core::ffi::c_float;
                    (*value_text).pos_y = new_var as f32;
                    (*value_text).pos_z = z_0;
                    (*value_text).default_alignment = 2 as u8;
                    (*value_text).text_color = mn_804D4B64;
                }
                if mnDiagram2_IsIconOnlyStat(stat_type) != 0
                    && {
                        rank = i as u8;
                        mnDiagram2_GetAggregatedFighterRank(
                            &raw mut first_rank,
                            stat_type,
                            rank,
                        );
                        first_rank.anon_0.idx as ::core::ffi::c_int
                            != 0x19 as ::core::ffi::c_int
                    }
                {
                    let mut rank_0: u8 = i as u16 as u8;
                    mnDiagram2_GetAggregatedFighterRank(
                        &raw mut fighter_rank,
                        stat_type,
                        rank_0,
                    );
                    icon = mnDiagram_CreateFighterIcon(
                        fighter_rank.anon_0.idx as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    HSD_JObjSetTranslateY(icon, row_spacing * i as f32);
                    HSD_JObjAddChild(
                        (*data).jobjs[6 as ::core::ffi::c_int as usize],
                        icon,
                    );
                    let mut rank_1: u8 = i as u8;
                    mnDiagram2_GetAggregatedFighterRank(
                        &raw mut rank_value,
                        stat_type,
                        rank_1,
                    );
                    mnDiagram_FormatDecimalNumber(
                        &raw mut sp58 as *mut u8 as *mut ::core::ffi::c_char,
                        rank_value.anon_0.xC as u32,
                        0 as ::core::ffi::c_int,
                    );
                    let mut offset_y: f32 = neg_spacing * i as f32 / 0.035f32;
                    HSD_SisLib_803A6B98(value_text, 0.0f32, offset_y as ::core::ffi::c_float, &raw mut sp58 as *mut u8 as *mut ::core::ffi::c_char, &[]);
                    current_block_96 = 14541395414537699361;
                } else {
                    if mnDiagram2_IsTimeStat(stat_type) != 0 {
                        let mut val_0: ::core::ffi::c_int = mnDiagram2_GetStatValue(
                            (*data).is_name_mode,
                            stat_type,
                            entity as u8,
                        );
                        if val_0 as u32 > max_time {
                            val_0 = max_time as ::core::ffi::c_int;
                        }
                        mnDiagram_FormatTime(
                            &raw mut sp58 as *mut u8 as *mut ::core::ffi::c_char,
                            val_0 as i32,
                        );
                        current_block_96 = 5005389895767293342;
                    } else if mnDiagram2_IsDistanceStat(stat_type) != 0 {
                        let mut val_1: u32 = mnDiagram2_GetStatValue(
                            (*data).is_name_mode,
                            stat_type,
                            entity as u8,
                        ) as u32;
                        val_1 = mnDiagram_ConvertDistanceForDisplay(val_1);
                        if val_1 > max_distance {
                            val_1 = max_distance;
                        }
                        mnDiagram_IntToStr(
                            &raw mut sp58 as *mut u8 as *mut ::core::ffi::c_char,
                            val_1,
                        );
                        current_block_96 = 5005389895767293342;
                    } else if mnDiagram2_IsPercentageStat(stat_type) != 0 {
                        let mut val_2: u32 = mnDiagram2_GetStatValue(
                            (*data).is_name_mode,
                            stat_type,
                            entity as u8,
                        ) as u32;
                        if val_2 > max_percentage {
                            val_2 = max_percentage;
                        }
                        mnDiagram_FormatDecimalNumber(
                            &raw mut sp58 as *mut u8 as *mut ::core::ffi::c_char,
                            val_2,
                            2 as ::core::ffi::c_int,
                        );
                        current_block_96 = 5005389895767293342;
                    } else if mnDiagram2_IsIconOnlyStat(stat_type) != 0 {
                        current_block_96 = 14763689060501151050;
                    } else {
                        let mut val_3: u32 = mnDiagram2_GetStatValue(
                            (*data).is_name_mode,
                            stat_type,
                            entity as u8,
                        ) as u32;
                        if val_3 > max_distance {
                            val_3 = max_distance;
                        }
                        mnDiagram_FormatDecimalNumber(
                            &raw mut sp58 as *mut u8 as *mut ::core::ffi::c_char,
                            val_3,
                            0 as ::core::ffi::c_int,
                        );
                        current_block_96 = 5005389895767293342;
                    }
                    match current_block_96 {
                        14763689060501151050 => {}
                        _ => {
                            let mut offset_y_0: f32 = neg_spacing * i as f32
                                / 0.035f32;
                            HSD_SisLib_803A6B98(value_text, 0.0f32, offset_y_0 as ::core::ffi::c_float, &raw mut sp58 as *mut u8 as *mut ::core::ffi::c_char, &[]);
                            current_block_96 = 14541395414537699361;
                        }
                    }
                }
                match current_block_96 {
                    14763689060501151050 => {}
                    _ => {
                        let mut icon_id: ::core::ffi::c_int = *unit_glyph_ids
                            .offset(stat_type as isize) as ::core::ffi::c_int;
                        if !(icon_id == 0xffff as ::core::ffi::c_int) {
                            lb_8000B1CC(
                                (*data).jobjs[6 as ::core::ffi::c_int as usize],
                                &raw mut mnDiagram3_803EEC28.x18,
                                &raw mut position,
                            );
                            let mut icon_text: *mut HSD_Text = ::core::ptr::null_mut::<
                                HSD_Text,
                            >();
                            icon_text = HSD_SisLib_803A5ACC(
                                0 as ::core::ffi::c_int,
                                1 as i32,
                                1.5f32 + position.x,
                                neg_spacing * i as f32 + -position.y,
                                position.z,
                                1.0f32,
                                1.0f32,
                            );
                            (*data).row_icons[i as usize] = icon_text;
                            (*icon_text).default_alignment = 1 as u8;
                            (*icon_text).text_color = mn_804D4B64;
                            if mnDiagram2_IsDistanceStat(stat_type) != 0 {
                                if mnDiagram_IsDistanceOverflow(
                                    mnDiagram2_GetStatValue(
                                        (*data).is_name_mode,
                                        stat_type,
                                        entity as u8,
                                    ) as u32,
                                ) != 0
                                {
                                    HSD_SisLib_803A6368(icon_text, 0x7f as i32);
                                    current_block_96 = 14763689060501151050;
                                } else {
                                    current_block_96 = 1428307939028130064;
                                }
                            } else {
                                current_block_96 = 1428307939028130064;
                            }
                            match current_block_96 {
                                14763689060501151050 => {}
                                _ => {
                                    HSD_SisLib_803A6368(icon_text, icon_id as i32);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
}
pub use crate::mn_r4::mnDiagram3_GetRowSpacing;
#[inline]
pub unsafe extern "C" fn mnDiagram3_ClearRowLabels(mut data: *mut Diagram3) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut check_data: *mut Diagram3 = data;
    let mut text_data: *mut Diagram3 = data;
    loop {
        if !(*check_data).row_labels[i as usize].is_null() {
            HSD_SisLib_803A5CC4((*text_data).row_labels[i as usize]);
            (*check_data).row_labels[i as usize] = ::core::ptr::null_mut::<HSD_Text>();
        }
        i += 1;
        if !(i < 10 as ::core::ffi::c_int) {
            break;
        }
    };
}
pub use crate::mn_ptr::mnDiagram3_GetRowStat;
#[inline]
pub unsafe extern "C" fn mnDiagram3_RebuildRowLabels(
    mut data: *mut Diagram3,
    mut base: *mut ::core::ffi::c_char,
    mut pos: *mut Vec3,
    mut count: ::core::ffi::c_int,
) {
    let mut spacing: f32 = 0.;
    let mut i: ::core::ffi::c_int = 0;
    let mut base_idx: ::core::ffi::c_int = 0;
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    base_idx = (*data).scroll_offset as ::core::ffi::c_int;
    spacing = HSD_JObjGetTranslationY((*data).jobjs[9 as ::core::ffi::c_int as usize])
        - HSD_JObjGetTranslationY((*data).jobjs[8 as ::core::ffi::c_int as usize]);
    lb_8000B1CC(
        (*data).jobjs[8 as ::core::ffi::c_int as usize],
        &raw mut mnDiagram3_803EEC28.x0,
        pos,
    );
    i = 0 as ::core::ffi::c_int;
    while i < count {
        text = HSD_SisLib_803A5ACC(
            0 as ::core::ffi::c_int,
            1 as i32,
            (*pos).x - 6.5f32,
            -spacing * i as f32 + -(*pos).y,
            (*pos).z,
            6.5f32,
            240.0f32,
        );
        (*data).row_labels[i as usize] = text;
        HSD_SisLib_803A6368(
            text,
            mnDiagram3_803EEC4C
                .label_ids[mnDiagram3_GetRowStat(&*data, base_idx as u8, i as u8)
                as usize] as i32,
        );
        i += 1;
    }
}
#[inline]
pub unsafe extern "C" fn mnDiagram3_RefreshRankings(mut diagram_gobj: *mut HSD_GObj) {
    mnDiagram2_ClearDetailView(diagram_gobj);
    mnDiagram3_PopulateRankings(diagram_gobj);
}
#[inline]
pub unsafe extern "C" fn mnDiagram3_PositionPopup(
    mut popup: *mut HSD_JObj,
    mut n: u8,
    mut cur: *mut Diagram3,
) {
    let mut spacing: f32 = 0.;
    spacing = mnDiagram3_GetRowSpacing(&*cur);
    HSD_JObjSetTranslateX(
        popup,
        HSD_JObjGetTranslationX((*cur).jobjs[8 as ::core::ffi::c_int as usize]),
    );
    HSD_JObjSetTranslateY(
        popup,
        spacing * n as f32
            + HSD_JObjGetTranslationY((*cur).jobjs[8 as ::core::ffi::c_int as usize]),
    );
    HSD_JObjSetTranslateZ(
        popup,
        HSD_JObjGetTranslationZ((*cur).jobjs[8 as ::core::ffi::c_int as usize]) as f32,
    );
}
pub unsafe extern "C" fn mnDiagram3_HandleInput(mut gobj: *mut HSD_GObj) {
    let mut base: *mut ::core::ffi::c_char = &raw mut mnDiagram3_803EEC10
        as *mut ::core::ffi::c_char;
    let mut data: *mut Diagram3 = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
    let mut input: u32 = Menu_GetAllInputs() as u32;
    let mut _pad: [::core::ffi::c_uchar; 32] = [0; 32];
    if input & MenuInput_Back as ::core::ffi::c_int as u32 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8;
        (*gmMainLib_GetGameRules()).xD = (*((*mnDiagram3_804D6C20).user_data
            as *mut Diagram3))
            .is_name_mode;
        mnDiagram2_ClearDetailView(mnDiagram3_804D6C20);
        HSD_GObjFree((*data).popup_gobj as *mut HSD_GObj);
        data = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
        mnDiagram3_ClearRowLabels(data);
        mn_80229894(0x1c as i32, 0 as u16, 3 as i32);
        return;
    }
    if input
        & (MenuInput_LTrigger as ::core::ffi::c_int
            | MenuInput_RTrigger as ::core::ffi::c_int) as u32 != 0
    {
        sfxForward();
        (*gmMainLib_GetGameRules()).xD = (*((*mnDiagram3_804D6C20).user_data
            as *mut Diagram3))
            .is_name_mode;
        mnDiagram2_ClearDetailView(mnDiagram3_804D6C20);
        HSD_GObjFree((*data).popup_gobj as *mut HSD_GObj);
        data = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
        mnDiagram3_ClearRowLabels(data);
        HSD_GObjFree(gobj);
        if input & MenuInput_LTrigger as ::core::ffi::c_int as u32 != 0 {
            mnDiagram2_Init();
            return;
        }
        mnDiagram_Init(0 as u8, 0 as u8);
        return;
    }
    if input
        & (MenuInput_XButton as ::core::ffi::c_int
            | MenuInput_YButton as ::core::ffi::c_int) as u32 != 0
    {
        let mut mode_label_pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
        let mut __0: [::core::ffi::c_uchar; 16] = [0; 16];
        if GetNameCount() == 0 as ::core::ffi::c_int {
            lbAudioAx_80024030(3 as ::core::ffi::c_int);
            return;
        }
        sfxForward();
        (*data).is_name_mode = (if (*data).is_name_mode as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as u8;
        if (*data).is_name_mode as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && (*data).scroll_offset as ::core::ffi::c_int + 0xa as ::core::ffi::c_int
                >= 0x15 as ::core::ffi::c_int
        {
            (*data).scroll_offset = 0 as u8;
        }
        data = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
        mnDiagram3_ClearRowLabels(data);
        data = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
        mnDiagram3_RebuildRowLabels(
            data,
            base,
            &raw mut mode_label_pos,
            10 as ::core::ffi::c_int,
        );
        mnDiagram3_RefreshRankings(mnDiagram3_804D6C20);
        return;
    }
    if input & 1 as u32 != 0 {
        if (*data).cursor_row as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            let mut popup: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            let mut cur: *mut Diagram3 = ::core::ptr::null_mut::<Diagram3>();
            sfxMove();
            (*data).cursor_row = ((*data).cursor_row as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as u8;
            popup = (*(*data).popup_gobj).hsd_obj as *mut HSD_JObj;
            cur = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
            mnDiagram3_PositionPopup(popup, (*data).cursor_row, cur);
            mnDiagram3_RefreshRankings(mnDiagram3_804D6C20);
            return;
        }
        if (*data).scroll_offset as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            let mut up_label_pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
            let mut __1: [::core::ffi::c_uchar; 12] = [0; 12];
            sfxMove();
            (*data).scroll_offset = ((*data).scroll_offset as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as u8;
            data = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
            mnDiagram3_ClearRowLabels(data);
            data = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
            mnDiagram3_RebuildRowLabels(
                data,
                base,
                &raw mut up_label_pos,
                10 as ::core::ffi::c_int,
            );
            mnDiagram3_RefreshRankings(mnDiagram3_804D6C20);
        }
    } else if input & 2 as u32 != 0 {
        let mut down_limit: u32 = 0;
        if ((*data).cursor_row as ::core::ffi::c_int) < 9 as ::core::ffi::c_int {
            let mut popup_0: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            let mut cur_0: *mut Diagram3 = ::core::ptr::null_mut::<Diagram3>();
            sfxMove();
            (*data).cursor_row = ((*data).cursor_row as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as u8;
            popup_0 = (*(*data).popup_gobj).hsd_obj as *mut HSD_JObj;
            cur_0 = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
            mnDiagram3_PositionPopup(popup_0, (*data).cursor_row, cur_0);
            mnDiagram3_RefreshRankings(mnDiagram3_804D6C20);
            return;
        }
        down_limit = (if (*data).is_name_mode as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            0x18 as ::core::ffi::c_int
        } else {
            0x15 as ::core::ffi::c_int
        }) as u32;
        if (((*data).scroll_offset as ::core::ffi::c_int + 0xa as ::core::ffi::c_int)
            as u32) < down_limit
        {
            let mut down_label_pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
            let mut __2: [::core::ffi::c_uchar; 16] = [0; 16];
            sfxMove();
            (*data).scroll_offset = ((*data).scroll_offset as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as u8;
            data = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
            mnDiagram3_ClearRowLabels(data);
            data = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
            mnDiagram3_RebuildRowLabels(
                data,
                base,
                &raw mut down_label_pos,
                10 as ::core::ffi::c_int,
            );
            mnDiagram3_RefreshRankings(mnDiagram3_804D6C20);
        }
    }
}




pub unsafe extern "C" fn mnDiagram3_UpdateScrollArrows(mut gobj: *mut HSD_GObj) {
    let mut data: *mut Diagram3 = ::core::ptr::null_mut::<Diagram3>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut new_var: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut limit: ::core::ffi::c_int = 0;
    data = (*gobj).user_data as *mut Diagram3;
    jobj = (*data).jobjs[3 as ::core::ffi::c_int as usize] as *mut HSD_JObj;
    mn_8022ED6C(jobj as *mut HSD_JObj, &raw mut mnDiagram3_803EEC1C);
    if (*data).is_name_mode as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        limit = 0x18 as ::core::ffi::c_int;
    } else {
        limit = 0x15 as ::core::ffi::c_int;
    }
    if ((*data).scroll_offset as ::core::ffi::c_int + 10 as ::core::ffi::c_int) < limit {
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
    jobj = (*data).jobjs[4 as ::core::ffi::c_int as usize] as *mut HSD_JObj;
    new_var = jobj;
    mn_8022ED6C(new_var as *mut HSD_JObj, &raw mut mnDiagram3_803EEC1C);
    if (*data).scroll_offset as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        HSD_JObjClearFlagsAll(new_var, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjSetFlagsAll(new_var, JOBJ_HIDDEN as u32);
    };
}
pub unsafe extern "C" fn mnDiagram3_OnAnimComplete(mut gobj: *mut HSD_GObj) {
    let mut data: *mut Diagram3 = ::core::ptr::null_mut::<Diagram3>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut table: *mut AnimLoopSettings = ::core::ptr::null_mut::<AnimLoopSettings>();
    let mut pad: ::core::ffi::c_int = 0;
    data = (*gobj).user_data as *mut Diagram3;
    jobj = (*data).jobjs[1 as ::core::ffi::c_int as usize] as *mut HSD_JObj;
    table = &raw mut mnDiagram3_803EEC10;
    pad = 0 as ::core::ffi::c_int;
    if mn_8022ED6C(jobj as *mut HSD_JObj, table) >= (*table).end_frame {
        HSD_GObjFree(gobj);
    }
}
pub unsafe extern "C" fn mnDiagram3_Think(mut gobj: *mut HSD_GObj) {
    let mut data: *mut Diagram3 = ::core::ptr::null_mut::<Diagram3>();
    let mut src: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut mode: u8 = 0;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    src = &raw mut mn_804A04F0 as *mut u8;
    data = (*gobj).user_data as *mut Diagram3;
    mode = *src.offset(0 as ::core::ffi::c_int as isize);
    if mode as ::core::ffi::c_int != 0x1e as ::core::ffi::c_int
        || *src.offset(0x10 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 2 as ::core::ffi::c_int
    {
        if mode as ::core::ffi::c_int == 0x1e as ::core::ffi::c_int {
            HSD_GObjFree(gobj);
        } else {
            let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
            HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
            proc_0 = HSD_GObj_SetupProc(
                gobj,
                Some(
                    mnDiagram3_OnAnimComplete
                        as unsafe extern "C" fn(*mut HSD_GObj) -> (),
                ),
                0 as u8,
            );
            (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
            HSD_JObjSetFlagsAll(
                (*data).jobjs[2 as ::core::ffi::c_int as usize],
                JOBJ_HIDDEN as u32,
            );
        }
    } else {
        mnDiagram3_UpdateScrollArrows(gobj);
    };
}
pub unsafe extern "C" fn mnDiagram3_FreeUserData(mut arg0: *mut ::core::ffi::c_void) {
    HSD_Free(arg0);
}
pub unsafe extern "C" fn mnDiagram3_InitUserData(
    mut data: *mut Diagram3,
    mut arg1: ::core::ffi::c_int,
) {
    let mut src: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut i: ::core::ffi::c_int = 0;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    src = &raw mut mn_804A04F0 as *mut u8;
    (*data).saved_menu = *src.offset(0 as ::core::ffi::c_int as isize);
    (*data).cursor_row = *(src.offset(2 as ::core::ffi::c_int as isize) as *mut u16)
        as u8;
    (*data).anim_state = 1 as u8;
    (*data).scroll_offset = 0 as u8;
    (*data).is_name_mode = (*gmMainLib_GetGameRules()).xD;
    i = 0 as ::core::ffi::c_int;
    while i < 10 as ::core::ffi::c_int {
        (*data).row_labels[i as usize] = ::core::ptr::null_mut::<HSD_Text>();
        i += 1;
    }
    (*data).popup_gobj = ::core::ptr::null_mut::<HSD_GObj>();
    (*data).title_text = ::core::ptr::null_mut::<HSD_Text>();
    (*data).value_text = ::core::ptr::null_mut::<HSD_Text>();
    (*data).row_icons[0 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).row_icons[1 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).row_icons[2 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).row_icons[3 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).row_icons[4 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
}
#[inline]
pub unsafe extern "C" fn mnDiagram3_LoadJoint(
    mut archive: *mut StaticModelDesc,
) -> *mut HSD_JObj {
    return HSD_JObjLoadJoint((*archive).joint as *mut HSD_Joint);
}
pub unsafe extern "C" fn mnDiagram3_Create(mut arg0: ::core::ffi::c_int) {
    let mut archive: *mut StaticModelDesc = &raw mut MenMainConB3_Top;
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut user_data: *mut Diagram3 = ::core::ptr::null_mut::<Diagram3>();
    let mut i: ::core::ffi::c_int = 0;
    gobj = GObj_Create(6 as u16, 7 as u8, 0x80 as u8);
    mnDiagram3_804D6C20 = gobj;
    jobj = mnDiagram3_LoadJoint(archive);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut ::core::ffi::c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
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
    HSD_JObjAnimAll(jobj);
    user_data = HSD_MemAlloc(::core::mem::size_of::<Diagram3>() as i32)
        as *mut Diagram3;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char, &[]);
        __assert(
            b"src/melee/mn/mndiagram3.c\0" as *const u8 as *const ::core::ffi::c_char,
            621 as u32,
            b"user_data\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    mnDiagram3_InitUserData(user_data, arg0);
    GObj_InitUserData(
        gobj,
        0 as u8,
        Some(
            mnDiagram3_FreeUserData
                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        user_data as *mut ::core::ffi::c_void,
    );
    i = 0 as ::core::ffi::c_int;
    while i < 10 as ::core::ffi::c_int {
        lb_80011E24(jobj as *mut HSD_JObj, (&raw mut (*user_data).jobjs as *mut *mut HSD_JObj).offset(i as isize)
                as *mut *mut HSD_JObj, &[Va::I32(i), Va::I32(-(1 as ::core::ffi::c_int))]);
        i += 1;
    }
    HSD_GObj_SetupProc(
        gobj,
        Some(mnDiagram3_Think as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
}
#[inline]
pub unsafe extern "C" fn mnDiagram3_SetupRows(
    mut jobj: *mut HSD_JObj,
    mut popup_data: *mut Diagram3,
    mut count: ::core::ffi::c_int,
) {
    let mut spacing: f32 = 0.;
    let mut row_spacing: f32 = 0.;
    let mut i: ::core::ffi::c_int = 0;
    let mut pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut data: *mut Diagram3 = ::core::ptr::null_mut::<Diagram3>();
    let mut scroll: ::core::ffi::c_int = 0;
    let mut stat_idx: u8 = 0;
    HSD_JObjSetTranslateZWithMtxDirty(
        jobj,
        HSD_JObjGetTranslationZ((*popup_data).jobjs[8 as ::core::ffi::c_int as usize])
            as f32,
    );
    data = (*mnDiagram3_804D6C20).user_data as *mut Diagram3;
    scroll = (*data).scroll_offset as ::core::ffi::c_int;
    spacing = HSD_JObjGetTranslationY((*data).jobjs[9 as ::core::ffi::c_int as usize])
        - HSD_JObjGetTranslationY((*data).jobjs[8 as ::core::ffi::c_int as usize]);
    lb_8000B1CC(
        (*data).jobjs[8 as ::core::ffi::c_int as usize],
        &raw mut mnDiagram3_803EEC28.x0,
        &raw mut pos,
    );
    row_spacing = 6.5f32 as f32;
    i = 0 as ::core::ffi::c_int;
    while i < count {
        let mut fi: f32 = i as f32;
        let mut text: *mut HSD_Text = HSD_SisLib_803A5ACC(
            0 as ::core::ffi::c_int,
            1 as i32,
            pos.x - row_spacing,
            -spacing * fi + -pos.y,
            pos.z,
            row_spacing,
            240.0f32,
        );
        (*data).row_labels[i as usize] = text;
        let mut type_idx: u8 = i as u8;
        let mut val: ::core::ffi::c_int = 0;
        let mut limit: ::core::ffi::c_int = 0;
        if (*data).is_name_mode as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            limit = 0x18 as ::core::ffi::c_int;
        } else {
            limit = 0x15 as ::core::ffi::c_int;
        }
        limit = limit as u8 as ::core::ffi::c_int;
        stat_idx = scroll as u8;
        val = stat_idx as ::core::ffi::c_int + type_idx as ::core::ffi::c_int;
        if val >= limit {
            limit = val - limit;
        } else {
            limit = val as u8 as ::core::ffi::c_int;
        }
        let mut entry: *mut u16 = (&raw mut mnDiagram3_803EEC4C.label_ids
            as *mut u16)
            .offset(limit as u8 as isize) as *mut u16;
        HSD_SisLib_803A6368(text, *entry as i32);
        i += 1;
    }
}
pub use crate::mn_r4::mnDiagram3_GetPopupSpacing;
pub use crate::mn_r4::mnDiagram3_GetPopupY;
#[inline]
pub unsafe extern "C" fn mnDiagram3_CreatePopup(
    mut data: *mut Diagram3,
    mut archive: *mut StaticModelDesc,
) -> *mut HSD_JObj {
    let mut popup: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut popup_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    popup = GObj_Create(6 as u16, 7 as u8, 0x80 as u8);
    (*data).popup_gobj = popup as *mut HSD_GObj;
    popup_jobj = mnDiagram3_LoadJoint(archive);
    HSD_GObjObject_80390A70(
        popup,
        HSD_GObj_JObjKind,
        popup_jobj as *mut ::core::ffi::c_void,
    );
    GObj_SetupGXLink(
        popup,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
        ),
        4 as u8,
        0x80 as u32,
    );
    HSD_JObjAddAnimAll(
        popup_jobj,
        (*archive).animjoint,
        (*archive).matanim_joint,
        (*archive).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(popup_jobj, 0.0f32);
    HSD_JObjAnimAll(popup_jobj);
    return popup_jobj;
}
pub unsafe extern "C" fn mnDiagram3_Init(mut arg0: *mut ::core::ffi::c_void) {
    let mut new_var: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
        ::core::ffi::c_void,
    >();
    let mut data: *mut Diagram3 = ::core::ptr::null_mut::<Diagram3>();
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut archive: *mut StaticModelDesc = ::core::ptr::null_mut::<StaticModelDesc>();
    let mut row_spacing: f32 = 0.;
    let mut flow: *mut MenuFlow = &raw mut mn_804A04F0;
    (*flow).x10 = 2 as u8;
    (*flow).hovered_selection = 0 as u16;
    mnDiagram3_Create(arg0 as ::core::ffi::c_int);
    gobj = mnDiagram3_804D6C20;
    archive = &raw mut MenMainCursorB3_Top;
    let mut popup_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    data = (*gobj).user_data as *mut Diagram3;
    popup_jobj = mnDiagram3_CreatePopup(data, archive);
    new_var = (*gobj).user_data;
    data = new_var as *mut Diagram3;
    row_spacing = mnDiagram3_GetPopupSpacing(&*(*data).jobjs[8], &*data);
    HSD_JObjSetTranslateXWithMtxDirty(
        popup_jobj,
        HSD_JObjGetTranslationX((*data).jobjs[8 as ::core::ffi::c_int as usize]),
    );
    row_spacing = mnDiagram3_GetPopupY(&*(*data).jobjs[8], row_spacing, 0);
    HSD_JObjSetTranslateYWithMtxDirty(popup_jobj, row_spacing);
    mnDiagram3_SetupRows(popup_jobj, data, 10 as ::core::ffi::c_int);
    mnDiagram3_PopulateRankings(mnDiagram3_804D6C20);
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0 as u16, 1 as u8, 0x80 as u8),
        Some(mnDiagram3_HandleInput as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
}
