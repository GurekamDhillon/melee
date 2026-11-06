//! TARGET_PC port of mndiagram.c. C-native seeded state oracle: tests/tu_mndiagram.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments, unused_variables, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::HSD_Text as HSD_Text_mn_ptr;
use melee_types::*;
use std::vec::Vec;
use crate::oracle::Va;
use super::mninl::*;
use super::hsd::{sfxBack,sfxForward,sfxMove,jobj_child as HSD_JObjGetChild};
unsafe fn HSD_GObjGetUserData(g:*mut HSD_GObj)->*mut core::ffi::c_void { (*g).user_data }
unsafe fn Menu_GetAllInputs()->u64 { mn_804A04F0.buttons = ext::mn_80229624(4) as u64; mn_804A04F0.buttons }
pub mod ext { use super::*; crate::oext_fns! {
fn __assert(a0: *const i8,a1: u32,a2: *const i8) -> ();
fn GObj_Create(a0: u16,a1: u8,a2: u8) -> *mut HSD_GObj;
fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj,a1: i32) -> () = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
fn lbAudioAx_80024030(a0: i32) -> () = crate::tu::adapt::lbAudioAx_80024030__36458a;
fn mn_80229624(a0: u32) -> u32 = crate::tu::adapt::mn_80229624__84b7ce;
fn mn_80229894(a0: i32,a1: u16,a2: i32) -> () = crate::tu::adapt::mn_80229894__f7df64;
fn mn_IsFighterUnlocked(a0: SelectableCharacterKind) -> i32 = crate::tu::adapt::mn_IsFighterUnlocked__2c05f5;
fn mn_GetDigitAt(a0: i32,a1: i32) -> i32 = crate::tu::adapt::mn_GetDigitAt__4d1d41;
fn mn_GetDigitCount(a0: i32) -> i32 = crate::tu::adapt::mn_GetDigitCount__861d08;
fn mn_8022ED6C(a0: *mut HSD_JObj,a1: *mut AnimLoopSettings) -> f32 = crate::tu::adapt::mn_8022ED6C__f73cae;
fn mn_8022EFD8(a0: *mut HSD_JObj,a1: *mut AnimLoopSettings) -> f32 = crate::tu::adapt::mn_8022EFD8__c1f371;
fn mn_8022F3D8(a0: *mut HSD_JObj,a1: u8,a2: HSD_TypeMask) -> () = crate::tu::adapt::mn_8022F3D8__9f7fac;
fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj,a1: f32) -> ();
fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) -> ();
fn HSD_JObjRemoveAll(a0: *mut HSD_JObj) -> ();
fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj,a1: *mut HSD_AnimJoint,a2: *mut HSD_MatAnimJoint,a3: *mut HSD_ShapeAnimJoint) -> ();
fn HSD_JObjAnimAll(a0: *mut HSD_JObj) -> ();
fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj,a1: u32) -> ();
fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj,a1: u32) -> ();
fn HSD_JObjAddChild(a0: *mut HSD_JObj,a1: *mut HSD_JObj) -> ();
fn HSD_JObjAnim(a0: *mut HSD_JObj) -> ();
fn HSD_SisLib_803A6754(a0: i32,a1: i32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A6754__02fb41;
fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) -> () = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
fn mnDiagram2_Init() -> () = crate::tu::adapt::mnDiagram2_Init__f8f7d5;
fn mnDiagram3_Init(a0: *mut ::core::ffi::c_void) -> () = crate::tu::adapt::mnDiagram3_Init__fc06c3;
fn GetNameText(a0: i32) -> *mut i8 = crate::tu::adapt::GetNameText__a3637e;
fn GetNameCount() -> i32 = crate::tu::adapt::GetNameCount__b961fe;
fn gm_80160C90(a0: *mut HSD_Text,a1: u8,a2: u8) -> () = crate::tu::adapt::gm_80160C90__c185a6;
fn gm_SelKindToCKind(a0: u8) -> u8 = crate::tu::adapt::gm_SelKindToCKind__b7f955;
fn gm_IsCKindUnlocked(a0: u8) -> i32 = crate::tu::adapt::gm_IsCKindUnlocked__b6c64b;
fn gmMainLib_GetGameRules() -> *mut GameRules = crate::tu::adapt::gmMainLib_GetGameRules__34f6cb;
fn GetPersistentFighterData(a0: SelectableCharacterKind) -> *mut FighterData = crate::tu::adapt::GetPersistentFighterData__578ad2;
fn GetPersistentNameData(a0: i32) -> *mut NameTagData = crate::tu::adapt::GetPersistentNameData__529241;
fn lb_8000B1CC(a0: *mut HSD_JObj,a1: *mut Vec3,a2: *mut Vec3) -> () = crate::tu::adapt::lb_8000B1CC__69851d;
fn powi(a0: i32,a1: i32) -> i32 = crate::tu::adapt::powi__640168;
fn lbLang_IsSavedLanguageUS() -> i32 = crate::tu::adapt::lbLang_IsSavedLanguageUS__90e41e;
fn GObj_SetupGXLink(a0: *mut HSD_GObj,a1: GObj_RenderFunc,a2: u8,a3: u32) -> ();
fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj,a1: u8,a2: *mut ::core::ffi::c_void) -> ();
fn HSD_GObjFree(a0: *mut HSD_GObj) -> ();
fn HSD_GObj_SetupProc(a0: *mut HSD_GObj,a1: HSD_GObjEvent,a2: u8) -> *mut HSD_GObjProc;
fn HSD_GObjProc_RemoveProc(a0: *mut HSD_GObjProc) -> ();
fn GObj_InitUserData(a0: *mut HSD_GObj,a1: u8,a2: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>, a3: *mut core::ffi::c_void) -> ();
fn HSD_Free(a0: *mut ::core::ffi::c_void) -> () = crate::tu::adapt::HSD_Free__bb56fc;
fn HSD_MemAlloc(a0: i32) -> *mut ::core::ffi::c_void = crate::tu::adapt::HSD_MemAlloc__e04cd7;
}
pub use super::super::mncount::ext::{OSReport,HSD_SisLib_803A6B98,lb_80011E24,lbArchive_LoadSections};
}
use ext::*;
// menuguard: the menu state is mnmain's and gobj.c's, not a private copy (a NULL mn_804D6BB8 faulted the Event menu)
pub use super::mnmain::{mn_804A04F0, mn_804D6BB8};
pub use melee_runtime::tu::gobj_native::{HSD_GObj_804D783C, HSD_GObj_JObjKind};
// menuguard: the menu state is mnmain's and gobj.c's, not a private copy (a NULL mn_804D6BB8 faulted the Event menu)
pub use melee_runtime::tu::gobj_native::{HSD_GObj_CurrentInvokedProc};
crate::globals! {
rw pub MenMainCursorB1_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};

rw pub MenMainSubB1_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};

rw pub MenMainNmB_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};

rw pub MenMainFaceB_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};

rw pub MenMainConB1_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};

rw pub mnDiagram_ScreenGObj: *mut HSD_GObj = ::core::ptr::null::<HSD_GObj>()
    as *mut HSD_GObj;

rw pub MenMainCursorB3_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};

rw pub MenMainConB3_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};

rw pub MenMainConB2_Top: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
rw pub mnDiagram_PcBlock: mnDiagram_Assets = mnDiagram_Assets {
    sorted_fighters: [0; 25],
    pad_19: [0; 3],
    sorted_names: [0; 120],
    SubB1: [::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void; 4],
    NmB: [::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void; 4],
    FaceB: [::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void; 4],
    CursorB1: [::core::ptr::null::<::core::ffi::c_void>()
        as *mut ::core::ffi::c_void; 4],
    ConB1: [::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void; 4],
    ConB2: [::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void; 4],
    ConB3: [::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void; 4],
    CursorB3: [::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void; 5],
};
rw pub mnDiagram_PopupTextOffsets: mnDiagram_PopupAnimTableHead = mnDiagram_PopupAnimTableHead {
    points: [
        Vec3 {
            x: 4.0f32,
            y: 1.0f32,
            z: 0.0f32,
        },
        Vec3 {
            x: -3.0f32,
            y: 0.8f32,
            z: 0.0f32,
        },
        Vec3 {
            x: -1.0f32,
            y: 0.7f32,
            z: 0.0f32,
        },
    ],
};
rw pub mnDiagram_DefaultFighterOrder: [u8; 28] = [
    8 as i32 as u8,
    1 as i32 as u8,
    6 as i32 as u8,
    0x10 as i32 as u8,
    0x11 as i32 as u8,
    4 as i32 as u8,
    2 as i32 as u8,
    0xd as i32 as u8,
    0xb as i32 as u8,
    0 as i32 as u8,
    5 as i32 as u8,
    0xc as i32 as u8,
    0xe as i32 as u8,
    0x12 as i32 as u8,
    7 as i32 as u8,
    0xf as i32 as u8,
    0xa as i32 as u8,
    9 as i32 as u8,
    3 as i32 as u8,
    0x15 as i32 as u8,
    0x18 as i32 as u8,
    0x13 as i32 as u8,
    0x14 as i32 as u8,
    0x17 as i32 as u8,
    0x16 as i32 as u8,
    0 as i32 as u8,
    0 as i32 as u8,
    0 as i32 as u8,
];
rw pub mnDiagram_IntroAnim: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0.0f32,
    end_frame: 9.0f32,
    loop_frame: -0.1f32,
};
rw pub mnDiagram_PopupExitAnim: AnimLoopSettings = AnimLoopSettings {
    start_frame: 10.0f32,
    end_frame: 19.0f32,
    loop_frame: -0.1f32,
};
rw pub mnDiagram_ArrowAnim: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0.0f32,
    end_frame: 199.0f32,
    loop_frame: 0.0f32,
};
rw pub mnDiagram_PcAnimTable: mnDiagram_AnimTable = unsafe {
    mnDiagram_AnimTable {
        points: [
            Vec3 {
                x: 4.0f32,
                y: 1.0f32,
                z: 0.0f32,
            },
            Vec3 {
                x: -3.0f32,
                y: 0.8f32,
                z: 0.0f32,
            },
            Vec3 {
                x: -1.0f32,
                y: 0.7f32,
                z: 0.0f32,
            },
        ],
        default_fighter_order: [
            8 as i32 as u8,
            1 as i32 as u8,
            6 as i32 as u8,
            0x10 as i32 as u8,
            0x11 as i32 as u8,
            4 as i32 as u8,
            2 as i32 as u8,
            0xd as i32 as u8,
            0xb as i32 as u8,
            0 as i32 as u8,
            5 as i32 as u8,
            0xc as i32 as u8,
            0xe as i32 as u8,
            0x12 as i32 as u8,
            7 as i32 as u8,
            0xf as i32 as u8,
            0xa as i32 as u8,
            9 as i32 as u8,
            3 as i32 as u8,
            0x15 as i32 as u8,
            0x18 as i32 as u8,
            0x13 as i32 as u8,
            0x14 as i32 as u8,
            0x17 as i32 as u8,
            0x16 as i32 as u8,
            0 as i32 as u8,
            0 as i32 as u8,
            0 as i32 as u8,
        ],
        intro_anim: AnimLoopSettings {
            start_frame: 0.0f32,
            end_frame: 9.0f32,
            loop_frame: -0.1f32,
        },
        exit_anim: AnimLoopSettings {
            start_frame: 10.0f32,
            end_frame: 19.0f32,
            loop_frame: -0.1f32,
        },
        arrow_anim: AnimLoopSettings {
            start_frame: 0.0f32,
            end_frame: 199.0f32,
            loop_frame: 0.0f32,
        },
        cursor_anim: AnimLoopSettings {
            start_frame: 0.0f32,
            end_frame: 10.0f32,
            loop_frame: -0.1f32,
        },
        user_data_error: ::core::mem::transmute::<
            [u8; 24],
            [i8; 24],
        >(*b"Can't get user_data.\n\0\0\0"),
        file_name: ::core::mem::transmute::<
            [u8; 12],
            [i8; 12],
        >(*b"mndiagram.c\0"),
        user_data_name: ::core::mem::transmute::<
            [u8; 20],
            [i8; 20],
        >(*b"user_data\0\0\0\0\0\0\0\0\0\0\0"),
    }
};
rw pub mnDiagram_PopupTextColor: GXColor = GXColor {
    r: 0 as u8,
    g: 0 as u8,
    b: 0 as u8,
    a: 0xff as u8,
};

rw pub mnDiagram_StringTerminator: [i8; 1] = unsafe {
    ::core::mem::transmute::<[u8; 1], [i8; 1]>(*b"\0")
};


}
pub unsafe extern "C" fn mnDiagram_GetFighterByIndex(
    mut idx: i32,
) -> u8 {
    melee_core::ct!("mnDiagram_GetFighterByIndex");
    return (*(&raw mut mnDiagram_PcBlock.sorted_fighters as *mut u8
        as *mut [u8; 28]))[idx as usize];
}

pub unsafe extern "C" fn mnDiagram_GetNameByIndex(mut idx: i32) -> u8 {
    melee_core::ct!("mnDiagram_GetNameByIndex");
    return mnDiagram_PcBlock.sorted_names[idx as usize];
}

pub unsafe extern "C" fn mnDiagram_IsDistanceOverflow(mut distance: u32) -> i32 {
    melee_core::ct!("mnDiagram_IsDistanceOverflow");
    if lbLang_IsSavedLanguageUS() != 0 as i32 {
        if distance >= 0x274a6 as i32 as u32 {
            return 1;
        }
        return 0;
    } else {
        if distance >= 0x186a0 as i32 as u32 {
            return 1;
        }
        return 0;
    };
}

pub unsafe extern "C" fn mnDiagram_ConvertDistanceForDisplay(
    mut distance: u32,
) -> u32 {
    melee_core::ct!("mnDiagram_ConvertDistanceForDisplay");
    if lbLang_IsSavedLanguageUS() != 0 as i32 {
        if distance >= 0x274a6 as i32 as u32 {
            return distance.wrapping_div(160934 as i32 as u32);
        }
        return melee_core::ppc::fcvt_u32((distance as f64 / 30.4788f64));
    }
    if distance >= 0x186a0 as i32 as u32 {
        return distance.wrapping_div(100000 as i32 as u32);
    }
    return distance.wrapping_div(100 as u32);
}

pub unsafe extern "C" fn mnDiagram_GetHitPercentage(
    mut is_name_mode: u8,
    mut player_index: u8,
) -> i32 {
    melee_core::ct!("mnDiagram_GetHitPercentage");
    let mut player_attacks: f32 = 0.;
    let mut tag_player_attacks: f32 = 0.;
    if is_name_mode as i32 != 0 as i32 {
        if (*GetPersistentNameData(player_index as i32)).stats.attacks_total
            != 0 as u32
        {
            tag_player_attacks = (*GetPersistentNameData(player_index as i32))
                .stats
                .attacks_total as f32;
            return melee_core::ppc::fctiwz_f32(100.0f32
                * (100.0f32
                    * ((*GetPersistentNameData(player_index as i32)).stats.attacks_hit
                        as f32 / tag_player_attacks)));
        }
        return 0 as i32;
    }
    if (*GetPersistentFighterData(player_index as SelectableCharacterKind))
        .stats
        .attacks_total != 0 as u32
    {
        player_attacks = (*GetPersistentFighterData(
            player_index as SelectableCharacterKind,
        ))
            .stats
            .attacks_total as f32;
        return melee_core::ppc::fctiwz_f32(100.0f32
            * (100.0f32
                * ((*GetPersistentFighterData(player_index as SelectableCharacterKind))
                    .stats
                    .attacks_hit as f32 / player_attacks)));
    }
    return 0 as i32;
}

pub unsafe extern "C" fn mnDiagram_GetPlayPercentage(
    mut is_name_mode: u8,
    mut player_index: u8,
) -> i32 {
    melee_core::ct!("mnDiagram_GetPlayPercentage");
    let mut total_play_time: f32 = 0.;
    let mut i: i32 = 0;
    let mut zero: f32 = 0.0f32;
    if is_name_mode as i32 != 0 as i32 {
        total_play_time = 0.0f32 as f32;
        i = 0 as i32 as i32;
        while i < 0x78 as i32 {
            total_play_time += (*GetPersistentNameData(i)).stats.play_time as f32;
            i += 1;
        }
        if total_play_time != zero {
            return melee_core::ppc::fctiwz_f32(100.0f32
                * (100.0f32
                    * ((*GetPersistentNameData(player_index as i32)).stats.play_time
                        as f32 / total_play_time)));
        }
        return 0 as i32;
    }
    total_play_time = 0.0f32 as f32;
    i = 0 as i32 as i32;
    while i < 0x19 as i32 {
        total_play_time
            += (*GetPersistentFighterData(i as SelectableCharacterKind)).stats.play_time
                as f32;
        i += 1;
    }
    if total_play_time != zero {
        return melee_core::ppc::fctiwz_f32(100.0f32
            * (100.0f32
                * ((*GetPersistentFighterData(player_index as SelectableCharacterKind))
                    .stats
                    .play_time as f32 / total_play_time)));
    }
    return 0 as i32;
}

pub unsafe extern "C" fn mnDiagram_GetAveragePlayerCount(
    mut is_name_mode: u8,
    mut player_index: u8,
) -> i32 {
    melee_core::ct!("mnDiagram_GetAveragePlayerCount");
    let mut temp_f31: f32 = 0.;
    let mut temp_f31_2: f32 = 0.;
    if is_name_mode as i32 != 0 as i32 {
        if (*GetPersistentNameData(player_index as i32)).stats.match_count
            as i32 != 0 as i32
        {
            temp_f31_2 = (*GetPersistentNameData(player_index as i32)).stats.match_count
                as f32;
            return melee_core::ppc::fctiwz_f32(100.0f32
                * ((*GetPersistentNameData(player_index as i32)).stats.total_player_count
                    as f32 / temp_f31_2));
        }
        return 0 as i32;
    }
    if (*GetPersistentFighterData(player_index as i32 as SelectableCharacterKind))
        .stats
        .match_count as i32 != 0 as i32
    {
        temp_f31 = (*GetPersistentFighterData(
            player_index as i32 as SelectableCharacterKind,
        ))
            .stats
            .match_count as f32;
        return melee_core::ppc::fctiwz_f32(100.0f32
            * ((*GetPersistentFighterData(
                player_index as i32 as SelectableCharacterKind,
            ))
                .stats
                .total_player_count as f32 / temp_f31));
    }
    return 0 as i32;
}

pub unsafe extern "C" fn mnDiagram_GetNameTotalKOs(
    mut field_index: u8,
) -> i32 {
    melee_core::ct!("mnDiagram_GetNameTotalKOs");
    let mut total: i32 = 0 as i32;
    let mut i: i32 = 0;
    i = 0 as i32 as i32;
    while i < 0x78 as i32 {
        if !GetNameText(i as i32 & 0xff as i32).is_null() {
            total
                += (*GetPersistentNameData(field_index as i32))
                    .vs_kos[i as u8 as usize] as i32;
        }
        i += 1;
    }
    return total;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_SumNameFalls(
    mut field_index: u8,
) -> i32 {
    melee_core::ct!("mnDiagram_SumNameFalls");
    let mut total: i32 = 0 as i32;
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < 0x78 as i32 {
        if !GetNameText(i & 0xff as i32).is_null() {
            total
                += (*GetPersistentNameData(i as u8 as i32))
                    .vs_kos[field_index as usize] as i32;
        }
        i += 1;
    }
    if total > 999999 as i32 {
        total = 999999 as i32;
    }
    return total;
}

pub unsafe extern "C" fn mnDiagram_GetNameTotalFalls(
    mut field_index: u8,
) -> i32 {
    melee_core::ct!("mnDiagram_GetNameTotalFalls");
    let mut _pad: [u8; 16] = [0; 16];
    return mnDiagram_SumNameFalls(field_index);
}

pub unsafe extern "C" fn mnDiagram_GetFighterTotalKOs(
    mut field_index: u8,
) -> i32 {
    melee_core::ct!("mnDiagram_GetFighterTotalKOs");
    let mut total: i32 = 0 as i32;
    let mut selkind: i32 = 0;
    selkind = 0 as i32;
    while selkind < SELKIND_COUNT as i32 {
        if mn_IsFighterUnlocked(selkind as SelectableCharacterKind)
            != 0 as i32
        {
            total
                += (*GetPersistentFighterData(field_index as SelectableCharacterKind))
                    .fighter_kos[selkind as u8 as usize] as i32;
        }
        selkind += 1;
    }
    return total;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_SumFighterKOs(
    mut field_index: u8,
) -> i32 {
    melee_core::ct!("mnDiagram_SumFighterKOs");
    let mut total: i32 = 0 as i32;
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        if mn_IsFighterUnlocked(i as SelectableCharacterKind) != 0 as i32
        {
            total
                += (*GetPersistentFighterData(field_index as SelectableCharacterKind))
                    .fighter_kos[i as u8 as usize] as i32;
        }
        i += 1;
    }
    return total;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_SumFighterFalls(
    mut field_index: u8,
) -> i32 {
    melee_core::ct!("mnDiagram_SumFighterFalls");
    let mut total: i32 = 0 as i32;
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        if mn_IsFighterUnlocked(i as SelectableCharacterKind) != 0 {
            total
                += (*GetPersistentFighterData(
                    (i & 0xff as i32) as SelectableCharacterKind,
                ))
                    .fighter_kos[field_index as usize] as i32;
        }
        i += 1;
    }
    return total;
}

pub unsafe extern "C" fn mnDiagram_GetFighterTotalFalls(
    mut field_index: u8,
) -> i32 {
    melee_core::ct!("mnDiagram_GetFighterTotalFalls");
    let mut _pad: [u8; 16] = [0; 16];
    return mnDiagram_SumFighterFalls(field_index);
}
#[inline]
pub unsafe extern "C" fn mnDiagram_CountUnlockedFightersForHeaders() -> i32 {
    melee_core::ct!("mnDiagram_CountUnlockedFightersForHeaders");
    let mut i: i32 = 0;
    let mut count: i32 = 0 as i32;
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        if mn_IsFighterUnlocked(i as SelectableCharacterKind) != 0 {
            count += 1;
        }
        i += 1;
    }
    return count;
}

pub unsafe extern "C" fn mnDiagram_FormatDecimalNumber(
    mut buf: *mut i8,
    mut val: u32,
    mut decimal_places: i32,
) {
    melee_core::ct!("mnDiagram_FormatDecimalNumber");
    let mut i: i32 = 0;
    let mut digit_count: i32 = 0;
    let mut integer_part: u32 = 0;
    integer_part = val.wrapping_div(powi(10 as i32, decimal_places as i32) as u32);
    digit_count = mn_GetDigitCount(integer_part as i32) as i32;
    i = 0 as i32;
    while i < digit_count {
        *buf.offset(i as isize) = (mn_GetDigitAt(
            integer_part as i32,
            digit_count as i32 - 1 as i32 - i as i32,
        ) as i32 + '0' as i32) as i8;
        i += 1;
    }
    if decimal_places != 0 as i32 {
        let mut decimal_part: i32 = 0;
        let fresh0 = digit_count;
        digit_count = digit_count + 1;
        *buf.offset(fresh0 as isize) = '.' as i32 as i8;
        decimal_part = val.wrapping_rem(powi(10 as i32, decimal_places as i32) as u32)
            as i32;
        i = 0 as i32;
        while i < decimal_places {
            let fresh1 = digit_count;
            digit_count = digit_count + 1;
            *buf.offset(fresh1 as isize) = (mn_GetDigitAt(
                decimal_part as i32,
                decimal_places as i32 - 1 as i32 - i as i32,
            ) as i32 + '0' as i32) as i8;
            i += 1;
        }
    }
    *buf.offset(digit_count as isize) = *(&raw mut mnDiagram_StringTerminator
        as *mut i8);
}

pub unsafe extern "C" fn mnDiagram_FormatTime(
    mut buf: *mut i8,
    mut seconds: i32,
) {
    melee_core::ct!("mnDiagram_FormatTime");
    let mut i: i32 = 0;
    let mut digit_count: i32 = 0;
    let mut minutes: i32 = 0;
    let mut secs: i32 = 0;
    minutes = (seconds as i32 / 60 as i32) as i32;
    secs = (seconds as i32 % 60 as i32) as i32;
    digit_count = mn_GetDigitCount(minutes) as i32;
    i = 0 as i32;
    while i < digit_count {
        *buf.offset(i as isize) = (mn_GetDigitAt(
            minutes,
            digit_count as i32 - 1 as i32 - i as i32,
        ) as i32 + '0' as i32) as i8;
        i += 1;
    }
    let fresh2 = digit_count;
    digit_count = digit_count + 1;
    *buf.offset(fresh2 as isize) = ':' as i32 as i8;
    let fresh3 = digit_count;
    digit_count = digit_count + 1;
    *buf.offset(fresh3 as isize) = (secs as i32 / 10 as i32
        + '0' as i32) as i8;
    let fresh4 = digit_count;
    digit_count = digit_count + 1;
    *buf.offset(fresh4 as isize) = (secs as i32 % 10 as i32
        + '0' as i32) as i8;
    *buf.offset(digit_count as isize) = *(&raw mut mnDiagram_StringTerminator
        as *mut i8);
}

pub unsafe extern "C" fn mnDiagram_IntToStr(
    mut buf: *mut i8,
    mut val: u32,
) {
    melee_core::ct!("mnDiagram_IntToStr");
    let mut i: i32 = 0;
    let mut digit_count: i32 = 0;
    digit_count = mn_GetDigitCount(val as i32) as i32;
    i = 0 as i32;
    while i < digit_count {
        *buf.offset(i as isize) = (mn_GetDigitAt(
            val as i32,
            digit_count as i32 - 1 as i32 - i as i32,
        ) as i32 + '0' as i32) as i8;
        i += 1;
    }
    *buf.offset(digit_count as isize) = *(&raw mut mnDiagram_StringTerminator
        as *mut i8);
}

pub unsafe extern "C" fn mnDiagram_GetPrevNameIndex(mut idx: i32) -> u8 {
    melee_core::ct!("mnDiagram_GetPrevNameIndex");
    let mut original: i32 = 0;
    let mut i: i32 = 0;
    i = idx;
    original = i;
    loop {
        i -= 1;
        if i < 0 as i32 {
            return original as u8;
        }
        if !GetNameText(i as i32 & 0xff as i32).is_null() {
            break;
        }
    }
    return i as u8;
}

pub unsafe extern "C" fn mnDiagram_GetNextNameIndex(mut idx: i32) -> u8 {
    melee_core::ct!("mnDiagram_GetNextNameIndex");
    let mut original: i32 = 0;
    let mut i: i32 = 0;
    i = idx;
    original = i;
    loop {
        i += 1;
        if i >= 0x78 as i32 {
            return original as u8;
        }
        if !GetNameText(i as i32 & 0xff as i32).is_null() {
            break;
        }
    }
    return i as u8;
}

pub unsafe extern "C" fn mnDiagram_GetPrevFighterIndex(mut idx: i32) -> u8 {
    melee_core::ct!("mnDiagram_GetPrevFighterIndex");
    let mut ptr: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut original: i32 = 0;
    ptr = (&raw mut *(&raw mut mnDiagram_PcBlock.sorted_fighters as *mut u8
        as *mut [u8; 28]) as *mut u8)
        .offset(idx as isize);
    original = idx;
    loop {
        idx -= 1;
        ptr = ptr.offset(-1);
        if idx < 0 as i32 {
            return original as u8;
        }
        if !(mn_IsFighterUnlocked(*ptr as SelectableCharacterKind)
            == 0 as i32)
        {
            break;
        }
    }
    return idx as u8;
}

pub unsafe extern "C" fn mnDiagram_GetNextFighterIndex(mut idx: i32) -> u8 {
    melee_core::ct!("mnDiagram_GetNextFighterIndex");
    let mut ptr: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut original: i32 = 0;
    ptr = (&raw mut *(&raw mut mnDiagram_PcBlock.sorted_fighters as *mut u8
        as *mut [u8; 28]) as *mut u8)
        .offset(idx as isize);
    original = idx;
    loop {
        idx += 1;
        ptr = ptr.offset(1);
        if idx >= SELKIND_COUNT as i32 {
            return original as u8;
        }
        if !(mn_IsFighterUnlocked(*ptr as SelectableCharacterKind)
            == 0 as i32)
        {
            break;
        }
    }
    return idx as u8;
}

pub unsafe extern "C" fn mnDiagram_GetNamePlayTimeByFighter(
    mut name_idx: i32,
    mut selkind: SelectableCharacterKind,
) -> u32 {
    melee_core::ct!("mnDiagram_GetNamePlayTimeByFighter");
    return (*GetPersistentNameData(name_idx as i32))
        .play_time_by_fighter[selkind as usize];
}
#[inline]
pub unsafe extern "C" fn CheckAllZeroPlayTime(
    mut name_idx: i32,
) -> i32 {
    melee_core::ct!("CheckAllZeroPlayTime");
    let mut i: i32 = 0 as i32;
    let mut offset: i32 = i;
    loop {
        if (*GetPersistentNameData(name_idx as i32)).play_time_by_fighter[i as usize]
            != 0 as u32
        {
            return 0 as i32;
        }
        i += 1;
        offset += 4 as i32;
        if i >= SELKIND_COUNT as i32 {
            return 1 as i32;
        }
    };
}

pub unsafe extern "C" fn mnDiagram_GetRankedFighterForName(
    mut rank: i32,
    mut name_idx: i32,
    mut func: Option<
        unsafe extern "C" fn(i32, i32) -> u32,
    >,
) -> i32 {
    melee_core::ct!("mnDiagram_GetRankedFighterForName");
    let mut _pad: [i32; 2] = [0; 2];
    let mut entries: [RankEntry; 25] = [RankEntry { selkind: 0, value: 0 }; 25];
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    if CheckAllZeroPlayTime(name_idx) != 0 as i32 {
        return SELKIND_COUNT as i32;
    }
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        entries[i as usize].selkind = i as u8;
        entries[i as usize].value = func
            .expect("non-null function pointer")(name_idx, i);
        i += 1;
    }
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        let mut max_idx: i32 = i;
        j = i + 1 as i32;
        while j < SELKIND_COUNT as i32 {
            if entries[max_idx as usize].value < entries[j as usize].value {
                max_idx = j;
            }
            j += 1;
        }
        if max_idx != i {
            let mut temp: RankEntry = entries[max_idx as usize];
            j = max_idx;
            while j > i {
                entries[j as usize] = entries[(j - 1 as i32) as usize];
                j -= 1;
            }
            entries[i as usize] = temp;
        }
        i += 1;
    }
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        if gm_IsCKindUnlocked(gm_SelKindToCKind(entries[i as usize].selkind))
            != 0 as i32
        {
            if rank != 0 as i32 {
                rank -= 1;
                j = i + 1 as i32;
                while j < SELKIND_COUNT as i32 {
                    if func
                        .expect(
                            "non-null function pointer",
                        )(name_idx, entries[i as usize].selkind as i32)
                        == func
                            .expect(
                                "non-null function pointer",
                            )(
                            name_idx,
                            entries[j as usize].selkind as i32,
                        )
                    {
                        i += 1;
                        if rank != 0 as i32 {
                            rank -= 1;
                        } else {
                            return SELKIND_COUNT as i32
                        }
                    }
                    j += 1;
                }
            } else {
                if func
                    .expect(
                        "non-null function pointer",
                    )(name_idx, entries[i as usize].selkind as i32)
                    != 0 as u32
                {
                    if (i + 1 as i32)
                        < SELKIND_COUNT as i32
                    {
                        if func
                            .expect(
                                "non-null function pointer",
                            )(
                            name_idx,
                            entries[i as usize].selkind as i32,
                        )
                            == func
                                .expect(
                                    "non-null function pointer",
                                )(
                                name_idx,
                                entries[(i + 1 as i32) as usize].selkind
                                    as i32,
                            )
                        {
                            return SELKIND_COUNT as i32;
                        }
                    }
                    return entries[i as usize].selkind as i32;
                }
                return SELKIND_COUNT as i32;
            }
        }
        i += 1;
    }
    return SELKIND_COUNT as i32;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_AllPlayTimesZero(mut name: u8) -> i32 {
    melee_core::ct!("mnDiagram_AllPlayTimesZero");
    let mut i: i32 = 0;
    let mut offset: i32 = 0;
    i = 0 as i32;
    offset = i;
    loop {
        if (*GetPersistentNameData(name as i32)).play_time_by_fighter[i as usize]
            != 0 as u32
        {
            return 0 as i32;
        }
        i += 1;
        offset += 4 as i32;
        if i >= 0x19 as i32 {
            return 1 as i32;
        }
    };
}

pub unsafe extern "C" fn mnDiagram_GetLeastPlayedFighter(mut name_idx: u8) -> u8 {
    melee_core::ct!("mnDiagram_GetLeastPlayedFighter");
    let mut i: i32 = 0;
    let mut min_fighter: i32 = 0;
    let mut count: i32 = 0;
    if mnDiagram_AllPlayTimesZero(name_idx) != 0 {
        return SELKIND_COUNT as i32 as u8;
    }
    min_fighter = 0 as i32;
    i = 1 as i32;
    while i < SELKIND_COUNT as i32 {
        if mn_IsFighterUnlocked(i as SelectableCharacterKind) != 0 as i32
        {
            if (*GetPersistentNameData(name_idx as i32))
                .play_time_by_fighter[min_fighter as usize]
                > (*GetPersistentNameData(name_idx as i32))
                    .play_time_by_fighter[i as usize]
            {
                min_fighter = i;
            }
        }
        i += 1;
    }
    count = 0 as i32 as i32;
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        if mn_IsFighterUnlocked(i as SelectableCharacterKind) != 0 as i32
            && (*GetPersistentNameData(name_idx as i32)).play_time_by_fighter[i as usize]
                == 0 as u32
        {
            count += 1;
        }
        i += 1;
    }
    if count >= 2 as i32 {
        return 0x19 as u8;
    }
    count = 0 as i32 as i32;
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        if mn_IsFighterUnlocked(i as SelectableCharacterKind) != 0 as i32
            && i != min_fighter
        {
            if (*GetPersistentNameData(name_idx as i32)).play_time_by_fighter[i as usize]
                == (*GetPersistentNameData(name_idx as i32))
                    .play_time_by_fighter[min_fighter as usize]
            {
                count += 1;
            }
        }
        i += 1;
    }
    if count != 0 as i32 {
        return 0x19 as u8;
    }
    return min_fighter as u8;
}

pub unsafe extern "C" fn mnDiagram_SortFightersByKOs() {
    melee_core::ct!("mnDiagram_SortFightersByKOs");
    let mut totals: [u32; 25] = [0; 25];
    let mut dst: *mut u8 = &raw mut *(&raw mut mnDiagram_PcBlock.sorted_fighters
        as *mut u8 as *mut [u8; 28]) as *mut u8;
    let mut dst_iter: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut candidate: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut sp: [u8; 12] = [0; 12];
    dst_iter = dst;
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        let mut fighter: u8 = 0;
        *dst_iter = mnDiagram_DefaultFighterOrder[i as usize];
        fighter = mnDiagram_DefaultFighterOrder[i as usize];
        totals[mnDiagram_DefaultFighterOrder[i as usize] as usize] = mnDiagram_SumFighterKOs(
            fighter,
        ) as u32;
        i += 1;
        dst_iter = dst_iter.offset(1);
    }
    i = 0 as i32;
    while i < SELKIND_COUNT as i32 {
        let mut max_idx: i32 = 0;
        j = i;
        j += 1;
        candidate = dst.offset(j as isize) as *mut u8;
        max_idx = i;
        while j < SELKIND_COUNT as i32 {
            if mn_IsFighterUnlocked(*candidate as SelectableCharacterKind)
                != 0 as i32
            {
                if totals[(*(&raw mut mnDiagram_PcBlock.sorted_fighters as *mut u8
                    as *mut [u8; 28]))[max_idx as usize] as usize]
                    < totals[*candidate as usize]
                    || mn_IsFighterUnlocked(
                        (*(&raw mut mnDiagram_PcBlock.sorted_fighters as *mut u8
                            as *mut [u8; 28]))[max_idx as usize]
                            as SelectableCharacterKind,
                    ) == 0 as i32
                        && mn_IsFighterUnlocked(*candidate as SelectableCharacterKind)
                            != 0 as i32
                {
                    max_idx = j;
                }
            }
            candidate = candidate.offset(1);
            j += 1;
        }
        if max_idx != i {
            let mut temp: u8 = (*(&raw mut mnDiagram_PcBlock.sorted_fighters
                as *mut u8 as *mut [u8; 28]))[max_idx as usize];
            while max_idx > i {
                (*(&raw mut mnDiagram_PcBlock.sorted_fighters as *mut u8
                    as *mut [u8; 28]))[max_idx as usize] = (*(&raw mut mnDiagram_PcBlock
                    .sorted_fighters as *mut u8
                    as *mut [u8; 28]))[(max_idx - 1 as i32) as usize];
                max_idx -= 1;
            }
            (*(&raw mut mnDiagram_PcBlock.sorted_fighters as *mut u8
                as *mut [u8; 28]))[i as usize] = temp;
        }
        i += 1;
    }
}
#[inline]
pub unsafe extern "C" fn mnDiagram_SumNameKOs(mut field_index: u8) -> i32 {
    melee_core::ct!("mnDiagram_SumNameKOs");
    let mut total: i32 = 0;
    let mut j: i32 = 0;
    total = 0 as i32;
    j = total;
    while j < 0x78 as i32 {
        if !GetNameText(j & 0xff as i32).is_null() {
            total
                += (*GetPersistentNameData(field_index as i32))
                    .vs_kos[j as u8 as usize] as i32;
        }
        j += 1;
    }
    return total;
}

pub unsafe extern "C" fn mnDiagram_SortNamesByKOs() {
    melee_core::ct!("mnDiagram_SortNamesByKOs");
    let mut j: i32 = 0;
    let mut max_idx: i32 = 0;
    let mut dst_iter: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut i: i32 = 0;
    let mut assets: *mut mnDiagram_Assets = &raw mut mnDiagram_PcBlock.sorted_fighters
        as *mut u8 as *mut [u8; 28] as *mut mnDiagram_Assets;
    let mut dst: *mut u8 = &raw mut (*assets).sorted_names as *mut u8;
    let mut tp: *mut u32 = ::core::ptr::null_mut::<u32>();
    let mut candidate: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut n: i32 = 0;
    let mut totals: [u32; 120] = [0; 120];
    let mut _pad: [u8; 12] = [0; 12];
    dst_iter = dst;
    tp = &raw mut totals as *mut u32;
    n = 0 as i32;
    while n < 0x78 as i32 {
        *dst_iter = n as u8;
        *tp = mnDiagram_SumNameKOs((n & 0xff as i32) as u8) as u32;
        n += 1;
        dst_iter = dst_iter.offset(1);
        tp = tp.offset(1);
    }
    i = 0 as i32;
    while i < 0x78 as i32 {
        j = i;
        j += 1;
        candidate = (&raw mut mnDiagram_PcBlock.sorted_names as *mut u8)
            .offset(j as isize) as *mut u8;
        max_idx = i;
        while j < 0x78 as i32 {
            if !GetNameText(*candidate as i32).is_null()
                && (totals[mnDiagram_PcBlock.sorted_names[max_idx as usize] as usize]
                    < totals[*candidate as usize]
                    || GetNameText(
                            mnDiagram_PcBlock.sorted_names[max_idx as usize]
                                as i32,
                        )
                        .is_null()
                        && !GetNameText(*candidate as i32).is_null())
            {
                max_idx = j;
            }
            candidate = candidate.offset(1);
            j += 1;
        }
        if max_idx != i {
            let mut p: *mut u8 = (&raw mut (*assets).sorted_fighters as *mut u8)
                .offset(max_idx as isize) as *mut u8;
            p = p.offset(::core::mem::size_of::<[u8; 28]>() as usize as isize);
            let mut temp: u8 = *p;
            while max_idx > i {
                *p = *p.offset(-(1 as i32 as isize));
                p = p.offset(-1);
                max_idx -= 1;
            }
            *dst.offset(i as isize) = temp;
        }
        i += 1;
    }
}

pub unsafe extern "C" fn mnDiagram_CountUnlockedFighters() -> i32 {
    melee_core::ct!("mnDiagram_CountUnlockedFighters");
    let mut i: i32 = 0;
    let mut count: i32 = 0;
    i = 0 as i32;
    count = 0 as i32 as i32;
    while i < SELKIND_COUNT as i32 {
        if mn_IsFighterUnlocked(i as SelectableCharacterKind) != 0 as i32
        {
            count += 1;
        }
        i += 1;
    }
    return count as i32;
}

pub unsafe extern "C" fn mnDiagram_PopupInputProc(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mnDiagram_PopupInputProc");
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut data: *mut Diagram = (*mnDiagram_ScreenGObj).user_data as *mut Diagram;
    let mut input: u64 = Menu_GetAllInputs();
    if input as u32 & MenuInput_Back as i32 as u32 != 0 {
        sfxBack();
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(mnDiagram_InputProc as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
        HSD_GObjFree((*data).popup_gobj);
        (*data).popup_gobj = ::core::ptr::null_mut::<HSD_GObj>();
    }
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetVisibleNameFrom(
    mut sorted: *mut u8,
    mut start: i32,
    mut rank: i32,
) -> u8 {
    melee_core::ct!("mnDiagram_GetVisibleNameFrom");
    let mut p: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut p2: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut remaining: i32 = 0;
    let mut idx: i32 = 0;
    p = sorted;
    p = p.offset(start as isize);
    remaining = rank;
    idx = start;
    p = p.offset(0x1c as i32 as isize);
    while remaining > 0 as i32 {
        p2 = p;
        loop {
            idx += 1;
            p2 = p2.offset(1);
            p = p.offset(1);
            if idx >= 0x78 as i32 {
                return 0x78 as u8;
            }
            if !GetNameText(*p2 as i32).is_null() {
                break;
            }
        }
        remaining -= 1;
    }
    p = sorted;
    p = p.offset(idx as isize);
    return *p.offset(0x1c as i32 as isize);
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetVisibleNameRowForInput(
    mut sorted: *mut u8,
    mut start: i32,
    mut rank: i32,
) -> u8 {
    melee_core::ct!("mnDiagram_GetVisibleNameRowForInput");
    let mut p: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut p2: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut remaining: i32 = 0;
    let mut idx: i32 = 0;
    remaining = rank;
    p = sorted;
    p = p.offset(start as isize);
    idx = start;
    p = p.offset(0x1c as i32 as isize);
    while remaining > 0 as i32 {
        p2 = p;
        loop {
            idx += 1;
            p2 = p2.offset(1);
            p = p.offset(1);
            if idx >= 0x78 as i32 {
                return 0x78 as u8;
            }
            if !GetNameText(*p2 as i32).is_null() {
                break;
            }
        }
        remaining -= 1;
    }
    p = sorted;
    p = p.offset(idx as isize);
    return *p.offset(0x1c as i32 as isize);
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetVisibleNameFrom2(
    mut sorted: *mut u8,
    mut start: i32,
    mut rank: i32,
) -> u8 {
    melee_core::ct!("mnDiagram_GetVisibleNameFrom2");
    let mut remaining: i32 = 0;
    let mut idx: i32 = 0;
    let mut p: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut p2: *mut u8 = ::core::ptr::null_mut::<u8>();
    p = sorted;
    p = p.offset(start as isize);
    remaining = rank;
    idx = start;
    p = p.offset(0x1c as i32 as isize);
    while remaining > 0 as i32 {
        p2 = p;
        loop {
            idx += 1;
            p2 = p2.offset(1);
            p = p.offset(1);
            if idx >= 0x78 as i32 {
                return 0x78 as u8;
            }
            if !GetNameText(*p2 as i32).is_null() {
                break;
            }
        }
        remaining -= 1;
    }
    p = sorted;
    p = p.offset(idx as isize);
    return *p.offset(0x1c as i32 as isize);
}
#[inline]
pub unsafe extern "C" fn mnDiagram_FindPrevFighter(
    mut sorted: *mut u8,
    mut cur: SelectableCharacterKind,
) -> i32 {
    melee_core::ct!("mnDiagram_FindPrevFighter");
    let mut p: *mut u8 = sorted.offset(cur as i32 as isize);
    let mut found: SelectableCharacterKind = cur;
    loop {
        found -= 1;
        p = p.offset(-1);
        if (found as i32) < 0 as i32 {
            return cur as i32;
        }
        if !(mn_IsFighterUnlocked(*p as SelectableCharacterKind)
            == 0 as i32)
        {
            break;
        }
    }
    return found as u8 as i32;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_FindNextFighter(
    mut sorted: *mut u8,
    mut cur: SelectableCharacterKind,
) -> u8 {
    melee_core::ct!("mnDiagram_FindNextFighter");
    let mut p: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut found: SelectableCharacterKind = SELKIND_CAPTAIN;
    found = cur;
    p = sorted.offset(cur as i32 as isize);
    loop {
        found += 1;
        p = p.offset(1);
        if found as i32 >= SELKIND_COUNT as i32 {
            return cur as u8;
        }
        if !(mn_IsFighterUnlocked(*p as SelectableCharacterKind)
            == 0 as i32)
        {
            break;
        }
    }
    return found as u8;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_FindPrevName(mut cur: i32) -> i32 {
    melee_core::ct!("mnDiagram_FindPrevName");
    let mut found: i32 = cur;
    loop {
        found -= 1;
        if found < 0 as i32 {
            return cur;
        }
        if !GetNameText(found as i32 & 0xff as i32)
            .is_null()
        {
            break;
        }
    }
    return found as u8 as i32;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_FindPrevNameWrap(mut cur: i32) -> i32 {
    melee_core::ct!("mnDiagram_FindPrevNameWrap");
    let mut found: i32 = cur;
    loop {
        found -= 1;
        if found < 0 as i32 {
            return cur as u8 as i32;
        }
        if !GetNameText(found as i32 & 0xff as i32)
            .is_null()
        {
            break;
        }
    }
    return found as u8 as i32;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_FindPrevFighterWrap(
    mut sorted: *mut u8,
    mut cur: i32,
) -> i32 {
    melee_core::ct!("mnDiagram_FindPrevFighterWrap");
    let mut p: *mut u8 = sorted.offset(cur as isize);
    let mut found: i32 = cur;
    loop {
        found -= 1;
        p = p.offset(-1);
        if found < 0 as i32 {
            return cur as u8 as i32;
        }
        if !(mn_IsFighterUnlocked(*p as SelectableCharacterKind)
            == 0 as i32)
        {
            break;
        }
    }
    return found as u8 as i32;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_FindNextName(mut cur: i32) -> i32 {
    melee_core::ct!("mnDiagram_FindNextName");
    let mut found: i32 = cur;
    loop {
        found += 1;
        if found >= 0x78 as i32 {
            return cur as u8 as i32;
        }
        if !GetNameText(found as i32 & 0xff as i32)
            .is_null()
        {
            break;
        }
    }
    return found as u8 as i32;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetVisibleNameCursorFrom(
    mut start: i32,
    mut rank: i32,
) -> u8 {
    melee_core::ct!("mnDiagram_GetVisibleNameCursorFrom");
    while rank > 0 as i32 {
        loop {
            start += 1;
            if start >= 0x78 as i32 {
                return 0x78 as u8;
            }
            if !GetNameText(mnDiagram_GetNameByIndex(start) as i32)
                .is_null()
            {
                break;
            }
        }
        rank -= 1;
    }
    return mnDiagram_GetNameByIndex(start);
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetVisibleFighterCursorFrom(
    mut sorted: *mut u8,
    mut start: i32,
    mut rank: i32,
) -> u8 {
    melee_core::ct!("mnDiagram_GetVisibleFighterCursorFrom");
    let mut result: u8 = 0;
    let mut remaining: i32 = 0;
    let mut p2: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut p: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut idx: i32 = 0;
    remaining = rank;
    idx = start;
    p = sorted.offset(start as isize);
    's_20: while remaining >= 0 as i32 {
        if remaining == 0 as i32 {
            result = *sorted.offset(idx as isize);
            break;
        } else {
            p2 = p;
            loop {
                idx += 1;
                p2 = p2.offset(1);
                p = p.offset(1);
                if idx >= 0x19 as i32 {
                    result = 0x19 as u8;
                    break 's_20;
                } else {
                    if mn_IsFighterUnlocked(*p2 as SelectableCharacterKind)
                        == 0 as i32
                    {
                        continue;
                    }
                    remaining -= 1;
                    break;
                }
            }
        }
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetVisibleFighterColumnForInput(
    mut sorted: *mut u8,
    mut start: i32,
    mut rank: i32,
    mut index: *mut i32,
) -> u8 {
    melee_core::ct!("mnDiagram_GetVisibleFighterColumnForInput");
    let mut result: u8 = 0;
    let mut p: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut p2: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut remaining: i32 = 0;
    remaining = rank;
    *index = start as i32;
    p = sorted.offset(start as isize);
    's_18: while remaining >= 0 as i32 {
        if remaining == 0 as i32 {
            result = *sorted.offset(*index as isize);
            break;
        } else {
            p2 = p;
            loop {
                *index += 1;
                p2 = p2.offset(1);
                p = p.offset(1);
                if *index >= 0x19 as i32 {
                    result = 0x19 as u8;
                    break 's_18;
                } else {
                    if mn_IsFighterUnlocked(*p2 as SelectableCharacterKind)
                        == 0 as i32
                    {
                        continue;
                    }
                    remaining -= 1;
                    break;
                }
            }
        }
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetVisibleFighterRowForInput(
    mut sorted: *mut u8,
    mut data: *mut Diagram,
    mut selection: *const u16,
    mut index: *mut i32,
) -> u8 {
    melee_core::ct!("mnDiagram_GetVisibleFighterRowForInput");
    let mut selected_word: u16 = 0;
    let mut cursor_word: u16 = 0;
    let mut result: u8 = 0;
    let mut p: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut p2: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut remaining: i32 = 0;
    selected_word = *selection;
    cursor_word = (*data).fighter_cursor_pos;
    remaining = selected_word as i32 >> 8 as i32;
    *index = (cursor_word as i32 >> 8 as i32) as i32;
    p = sorted.offset(*index as isize);
    's_28: while remaining >= 0 as i32 {
        if remaining == 0 as i32 {
            result = *sorted.offset(*index as isize);
            break;
        } else {
            p2 = p;
            loop {
                *index += 1;
                p2 = p2.offset(1);
                p = p.offset(1);
                if *index >= 0x19 as i32 {
                    result = 0x19 as u8;
                    break 's_28;
                } else {
                    if mn_IsFighterUnlocked(*p2 as SelectableCharacterKind)
                        == 0 as i32
                    {
                        continue;
                    }
                    remaining -= 1;
                    break;
                }
            }
        }
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetVisibleFighterFromPointer(
    mut sorted: *const u8,
    mut p: *mut u8,
    mut start: i32,
    mut rank: i32,
) -> u8 {
    melee_core::ct!("mnDiagram_GetVisibleFighterFromPointer");
    let mut result: u8 = 0;
    let mut p2: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut remaining: i32 = 0;
    let mut idx: i32 = 0;
    remaining = rank;
    idx = start;
    's_15: while remaining >= 0 as i32 {
        if remaining == 0 as i32 {
            result = *sorted.offset(idx as isize);
            break;
        } else {
            p2 = p;
            loop {
                idx += 1;
                p2 = p2.offset(1);
                p = p.offset(1);
                if idx >= 0x19 as i32 {
                    result = 0x19 as u8;
                    break 's_15;
                } else {
                    if mn_IsFighterUnlocked(*p2 as SelectableCharacterKind)
                        == 0 as i32
                    {
                        continue;
                    }
                    remaining -= 1;
                    break;
                }
            }
        }
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetVisibleFighterCursorFrom2(
    mut start: i32,
    mut rank: i32,
) -> u8 {
    melee_core::ct!("mnDiagram_GetVisibleFighterCursorFrom2");
    while rank >= 0 as i32 {
        if rank == 0 as i32 {
            return mnDiagram_GetFighterByIndex(start);
        }
        loop {
            start += 1;
            if start >= SELKIND_COUNT as i32 {
                return SELKIND_COUNT as i32 as u8;
            }
            if !(mn_IsFighterUnlocked(
                mnDiagram_GetFighterByIndex(start) as SelectableCharacterKind,
            ) == 0 as i32)
            {
                break;
            }
        }
        rank -= 1;
    }
    panic!("Reached end of non-void function without returning");
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetCurrentDiagramData() -> *mut Diagram {
    melee_core::ct!("mnDiagram_GetCurrentDiagramData");
    return (*mnDiagram_ScreenGObj).user_data as *mut Diagram;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetVisibleNameColumnForInput(
    mut start: i32,
    mut rank: i32,
) -> u8 {
    melee_core::ct!("mnDiagram_GetVisibleNameColumnForInput");
    while rank > 0 as i32 {
        loop {
            start += 1;
            if start >= 0x78 as i32 {
                return 0x78 as u8;
            }
            if !GetNameText(mnDiagram_GetNameByIndex(start) as i32)
                .is_null()
            {
                break;
            }
        }
        rank -= 1;
    }
    return mnDiagram_GetNameByIndex(start);
}
#[inline]
pub unsafe extern "C" fn mnDiagram_CountUnlockedFightersForInput() -> i32 {
    melee_core::ct!("mnDiagram_CountUnlockedFightersForInput");
    let mut i: i32 = 0;
    let mut count: i32 = 0 as i32;
    i = 0 as i32;
    while i < 0x19 as i32 {
        if mn_IsFighterUnlocked(i as SelectableCharacterKind) != 0 as i32
        {
            count += 1;
        }
        i += 1;
    }
    return count;
}

pub unsafe extern "C" fn mnDiagram_InputProc(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mnDiagram_InputProc");
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut row_result: u8 = 0;
    let mut i: i32 = 0;
    let mut selection: *mut u16 = ::core::ptr::null_mut::<u16>();
    let mut new_var: i16 = 0;
    let mut sorted: *mut u8 = &raw mut *(&raw mut mnDiagram_PcBlock.sorted_fighters
        as *mut u8 as *mut [u8; 28]) as *mut u8;
    let mut data: *mut Diagram = mnDiagram_GetCurrentDiagramData();
    let mut input: u32 = mn_80229624(4 as u32);
    let mut count: i32 = 0;
    let mut col: i32 = 0;
    let mut row: i32 = 0;
    let mut new_var2: i32 = 0;
    let mut row3: i32 = 0;
    let mut row4: i32 = 0;
    let mut row5: i32 = 0;
    let mut row6: i32 = 0;
    let mut col_result: u8 = 0;
    let mut col_result2: u8 = 0;
    let mut row_result2: u8 = 0;
    let mut col_result3: u8 = 0;
    let mut col_result4: u8 = 0;
    let mut row_result3: u8 = 0;
    let mut row_result4: u8 = 0;
    let mut found: i32 = 0;
    let mut cur: i32 = 0;
    let mut cursor_pos: i32 = 0;
    let mut count2: i32 = 0;
    let mut _pad: [u8; 24] = [0; 24];
    mn_804A04F0.buttons = input as u64;
    count2 = 0 as i32 as i32;
    if input & MenuInput_Confirm as i32 as u32 != 0 {
        sfxForward();
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        i = 0 as i32 as i32;
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(mnDiagram_PopupInputProc as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            i as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
        if (*data).is_name_mode as i32 != 0 as i32 {
            col = mn_804A04F0.hovered_selection as i32;
            cur = col;
            col_result = mnDiagram_GetVisibleNameColumnForInput(
                (*data).name_cursor_pos as u8 as i32,
                cur as u8 as i32,
            );
            row = (mn_804A04F0.hovered_selection as i32
                >> 8 as i32) as i32;
            cursor_pos = (*data).name_cursor_pos as i32;
            row_result = mnDiagram_GetVisibleNameRowForInput(
                sorted,
                cursor_pos as i32 >> 8 as i32,
                row as i32,
            );
            mnDiagram_CreatePopup(col_result as i32, row_result as i32, 1 as i32);
            return;
        }
        selection = &raw mut mn_804A04F0.hovered_selection;
        row_result2 = (*data).fighter_cursor_pos as u8;
        col_result2 = mnDiagram_GetVisibleFighterColumnForInput(
            sorted,
            row_result2 as i32,
            *selection as u8 as i32,
            &raw mut i,
        );
        row_result2 = mnDiagram_GetVisibleFighterRowForInput(
            sorted,
            data,
            selection,
            &raw mut col,
        );
        mnDiagram_CreatePopup(col_result2 as i32, row_result2 as i32, 0 as i32);
        return;
    }
    if input & MenuInput_Back as i32 as u32 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = count2 as u8;
        let mut d: *mut Diagram = mnDiagram_GetCurrentDiagramData();
        (*gmMainLib_GetGameRules()).xE = ((*d).fighter_cursor_pos as i32
            >> 8 as i32) as u8;
        (*gmMainLib_GetGameRules()).xF = (*d).fighter_cursor_pos as u8;
        (*gmMainLib_GetGameRules()).unk_x10 = ((*d).name_cursor_pos as i32
            >> 8 as i32) as u8;
        (*gmMainLib_GetGameRules()).x11 = (*d).name_cursor_pos as u8;
        (*gmMainLib_GetGameRules()).xD = (*d).is_name_mode;
        mn_80229894(0x1c as i32, 0 as u16, 3 as i32);
        return;
    }
    if input
        & (MenuInput_LTrigger as i32
            | MenuInput_RTrigger as i32) as u32 != 0
    {
        sfxForward();
        let mut d_0: *mut Diagram = mnDiagram_GetCurrentDiagramData();
        (*gmMainLib_GetGameRules()).xE = ((*d_0).fighter_cursor_pos as i32
            >> 8 as i32) as u8;
        (*gmMainLib_GetGameRules()).xF = (*d_0).fighter_cursor_pos as u8;
        (*gmMainLib_GetGameRules()).unk_x10 = ((*d_0).name_cursor_pos
            as i32 >> 8 as i32) as u8;
        (*gmMainLib_GetGameRules()).x11 = (*d_0).name_cursor_pos as u8;
        (*gmMainLib_GetGameRules()).xD = (*d_0).is_name_mode;
        HSD_GObjFree(gobj);
        if input & MenuInput_LTrigger as i32 as u32 != 0 {
            mnDiagram3_Init(::core::ptr::null_mut::<::core::ffi::c_void>());
            return;
        }
        mnDiagram2_Init();
        return;
    }
    if input
        & (MenuInput_XButton as i32
            | MenuInput_YButton as i32) as u32 != 0
    {
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
            count2 as i32
        }) as u8;
        if (*data).is_name_mode as i32 != 0 as i32 {
            cur = GetNameCount() as i32;
            if mn_804A04F0.hovered_selection as u8 as i32 >= cur {
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection
                    as i32 & 0xff00 as i32
                    | (cur as i32 - 1 as i32) as u8
                        as i32) as u16;
            }
            if mn_804A04F0.hovered_selection as i32
                >> 8 as i32 >= cur
            {
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection as u8
                    as i32
                    | (cur as i32 - 1 as i32)
                        << 8 as i32) as u16;
            }
            mnDiagram_UpdateScrollArrowVisibility(
                mnDiagram_ScreenGObj,
                cur as i32,
            );
            mnDiagram_RefreshGrid(
                mnDiagram_ScreenGObj,
                (*data).name_cursor_pos as u8 as i32,
                (*data).name_cursor_pos as i32 >> 8 as i32,
            );
            return;
        }
        count = mnDiagram_CountUnlockedFightersForInput();
        new_var2 = count;
        if mn_804A04F0.hovered_selection as u8 as i32 >= new_var2 {
            mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection
                as i32 & 0xff00 as i32
                | (new_var2 as i32 - 1 as i32) as u8
                    as i32) as u16;
        }
        if mn_804A04F0.hovered_selection as i32 >> 8 as i32
            >= new_var2
        {
            mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection as u8
                as i32
                | (new_var2 as i32 - 1 as i32)
                    << 8 as i32) as u16;
        }
        mnDiagram_UpdateScrollArrowVisibility(
            mnDiagram_ScreenGObj,
            new_var2 as i32,
        );
        mnDiagram_RefreshGrid(
            mnDiagram_ScreenGObj,
            (*data).fighter_cursor_pos as u8 as i32,
            (*data).fighter_cursor_pos as i32 >> 8 as i32,
        );
        return;
    }
    if (*data).is_name_mode as i32 != 0 as i32 {
        count = GetNameCount() as i32;
        if input & 1 as u32 != 0 {
            col = mn_804A04F0.hovered_selection as u8 as i32;
            if col > 0 as i32
                && count > col as i32 - 1 as i32
            {
                sfxMove();
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection
                    as i32 & 0xff00 as i32
                    | col as i32 - 1 as i32
                        & 0xff as i32) as u16;
                return;
            }
            if count > 0xa as i32 {
                cur = (*data).name_cursor_pos as u8 as i32;
                found = mnDiagram_FindPrevName(cur) as u8 as i32;
                if cur != found {
                    sfxMove();
                    (*data).name_cursor_pos = ((*data).name_cursor_pos as i32
                        & 0xff00 as i32 | found) as u16;
                    mnDiagram_RefreshGrid(
                        mnDiagram_ScreenGObj,
                        (*data).name_cursor_pos as u8 as i32,
                        (*data).name_cursor_pos as i32
                            >> 8 as i32,
                    );
                }
            }
        } else if input & 2 as u32 != 0 {
            let mut next_name: u8 = 0;
            col = mn_804A04F0.hovered_selection as u8 as i32;
            if col < 9 as i32
                && count > col as i32 + 1 as i32
            {
                sfxMove();
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection
                    as i32 & 0xff00 as i32
                    | col as i32 + 1 as i32
                        & 0xff as i32) as u16;
                return;
            }
            if count > 0xa as i32 {
                cur = (*data).name_cursor_pos as u8 as i32;
                next_name = mnDiagram_FindNextName(cur) as u8;
                if cur != next_name as i32 {
                    col_result3 = mnDiagram_GetVisibleNameFrom(
                        sorted,
                        cur as i32,
                        0xa as i32,
                    );
                    if col_result3 as i32 != 0x78 as i32 {
                        sfxMove();
                        (*data).name_cursor_pos = ((*data).name_cursor_pos
                            as i32 & 0xff00 as i32
                            | next_name as i32) as u16;
                        mnDiagram_RefreshGrid(
                            mnDiagram_ScreenGObj,
                            (*data).name_cursor_pos as u8 as i32,
                            (*data).name_cursor_pos as i32
                                >> 8 as i32,
                        );
                    }
                }
            }
        } else if input & 4 as u32 != 0 {
            row3 = (mn_804A04F0.hovered_selection as i32
                >> 8 as i32) as i32;
            if (0 as i32) < row3
                && count > row3 as i32 - 1 as i32
            {
                sfxMove();
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection as u8
                    as i32
                    | (row3 as i32 - 1 as i32)
                        << 8 as i32) as u16;
                return;
            }
            if count > 7 as i32 {
                cur = ((*data).name_cursor_pos as i32
                    >> 8 as i32) as i32;
                found = mnDiagram_FindPrevNameWrap(cur) as u8 as i32;
                if cur != found {
                    sfxMove();
                    (*data).name_cursor_pos = ((*data).name_cursor_pos as u8 as i32
                        | found << 8 as i32) as u16;
                    mnDiagram_RefreshGrid(
                        mnDiagram_ScreenGObj,
                        (*data).name_cursor_pos as u8 as i32,
                        (*data).name_cursor_pos as i32
                            >> 8 as i32,
                    );
                }
            }
        } else if input & 8 as u32 != 0 {
            let mut next_name_0: u8 = 0;
            row4 = (mn_804A04F0.hovered_selection as i32
                >> 8 as i32) as i32;
            if row4 < 6 as i32
                && count > row4 as i32 + 1 as i32
            {
                sfxMove();
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection as u8
                    as i32
                    | (row4 as i32 + 1 as i32)
                        << 8 as i32) as u16;
                return;
            }
            if count > 7 as i32 {
                cur = ((*data).name_cursor_pos as i32
                    >> 8 as i32) as i32;
                next_name_0 = mnDiagram_FindNextName(cur) as u8;
                if cur != next_name_0 as i32 {
                    row_result3 = mnDiagram_GetVisibleNameFrom(
                        sorted,
                        cur as i32,
                        7 as i32,
                    );
                    if row_result3 as i32 != 0x78 as i32 {
                        sfxMove();
                        (*data).name_cursor_pos = ((*data).name_cursor_pos as u8
                            as i32
                            | (next_name_0 as i32)
                                << 8 as i32) as u16;
                        mnDiagram_RefreshGrid(
                            mnDiagram_ScreenGObj,
                            (*data).name_cursor_pos as u8 as i32,
                            (*data).name_cursor_pos as i32
                                >> 8 as i32,
                        );
                    }
                }
            }
        }
    } else {
        new_var = 0 as i16;
        count2 = mnDiagram_CountUnlockedFightersForHeaders() as i32;
        if input & 1 as u32 != 0 {
            col = mn_804A04F0.hovered_selection as u8 as i32;
            if col > new_var as i32
                && count2 > col as i32 - 1 as i32
            {
                sfxMove();
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection
                    as i32 & 0xff00 as i32
                    | col as i32 - 1 as i32
                        & 0xff as i32) as u16;
                return;
            }
            if count2 > 0xa as i32 {
                cur = (*data).fighter_cursor_pos as u8 as i32;
                found = mnDiagram_FindPrevFighter(sorted, cur as SelectableCharacterKind)
                    as u8 as i32;
                if cur != found {
                    sfxMove();
                    (*data).fighter_cursor_pos = ((*data).fighter_cursor_pos as i32
                        & 0xff00 as i32 | found) as u16;
                    mnDiagram_RefreshGrid(
                        mnDiagram_ScreenGObj,
                        (*data).fighter_cursor_pos as u8 as i32,
                        (*data).fighter_cursor_pos as i32
                            >> 8 as i32,
                    );
                }
            }
        } else if input & 2 as u32 != 0 {
            let mut nav_ptr: *mut u8 = ::core::ptr::null_mut::<u8>();
            col = mn_804A04F0.hovered_selection as u8 as i32;
            if col < 9 as i32
                && count2 > col as i32 + 1 as i32
            {
                sfxMove();
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection
                    as i32 & 0xff00 as i32
                    | col as i32 + 1 as i32
                        & 0xff as i32) as u16;
                return;
            }
            if count2 > 0xa as i32 {
                cur = (*data).fighter_cursor_pos as u8 as i32;
                nav_ptr = sorted.offset(cur as isize);
                row5 = mnDiagram_FindNextFighter(sorted, cur as SelectableCharacterKind)
                    as i32;
                found = row5;
                col_result4 = mnDiagram_GetVisibleFighterFromPointer(
                    sorted,
                    nav_ptr,
                    cur as i32,
                    0xa as i32,
                );
                if col_result4 as i32 != 0x19 as i32 {
                    sfxMove();
                    (*data).fighter_cursor_pos = ((*data).fighter_cursor_pos as i32
                        & 0xff00 as i32 | found) as u16;
                    mnDiagram_RefreshGrid(
                        mnDiagram_ScreenGObj,
                        (*data).fighter_cursor_pos as u8 as i32,
                        (*data).fighter_cursor_pos as i32
                            >> 8 as i32,
                    );
                }
            }
        } else if input & 4 as u32 != 0 {
            row5 = (mn_804A04F0.hovered_selection as i32
                >> 8 as i32) as i32;
            if row5 > new_var as i32
                && count2 > row5 as i32 - 1 as i32
            {
                sfxMove();
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection as u8
                    as i32
                    | (row5 as i32 - 1 as i32)
                        << 8 as i32) as u16;
                return;
            }
            if count2 > 7 as i32 {
                cur = ((*data).fighter_cursor_pos as i32
                    >> 8 as i32) as i32;
                found = mnDiagram_FindPrevFighterWrap(sorted, cur) as u8 as i32;
                if cur != found {
                    sfxMove();
                    (*data).fighter_cursor_pos = ((*data).fighter_cursor_pos as u8
                        as i32 | found << 8 as i32) as u16;
                    mnDiagram_RefreshGrid(
                        mnDiagram_ScreenGObj,
                        (*data).fighter_cursor_pos as u8 as i32,
                        (*data).fighter_cursor_pos as i32
                            >> 8 as i32,
                    );
                }
            }
        } else if input & 8 as u32 != 0 {
            let mut nav_ptr_0: *mut u8 = ::core::ptr::null_mut::<u8>();
            row6 = (mn_804A04F0.hovered_selection as i32
                >> 8 as i32) as i32;
            if row6 < 6 as i32
                && count2 > row6 as i32 + 1 as i32
            {
                sfxMove();
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection as u8
                    as i32
                    | (row6 as i32 + 1 as i32)
                        << 8 as i32) as u16;
                return;
            }
            if count2 > 7 as i32 {
                cur = ((*data).fighter_cursor_pos as i32
                    >> 8 as i32) as i32;
                nav_ptr_0 = sorted.offset(cur as isize);
                found = mnDiagram_FindNextFighter(sorted, cur as SelectableCharacterKind)
                    as i32;
                row_result4 = mnDiagram_GetVisibleFighterFromPointer(
                    sorted,
                    nav_ptr_0,
                    cur as i32,
                    7 as i32,
                );
                if row_result4 as i32 != 0x19 as i32 {
                    sfxMove();
                    (*data).fighter_cursor_pos = ((*data).fighter_cursor_pos as u8
                        as i32 | found << 8 as i32) as u16;
                    mnDiagram_RefreshGrid(
                        mnDiagram_ScreenGObj,
                        (*data).fighter_cursor_pos as u8 as i32,
                        (*data).fighter_cursor_pos as i32
                            >> 8 as i32,
                    );
                }
            }
        }
    };
}

pub unsafe extern "C" fn mnDiagram_PopupCleanup(mut arg0: *mut ::core::ffi::c_void) {
    melee_core::ct!("mnDiagram_PopupCleanup");
    let mut data: *mut mnDiagram_PopupData = arg0 as *mut mnDiagram_PopupData;
    if !(*data).text[0 as i32 as usize].is_null() {
        HSD_SisLib_803A5CC4((*data).text[0 as i32 as usize]);
    }
    if !(*data).text[1 as i32 as usize].is_null() {
        HSD_SisLib_803A5CC4((*data).text[1 as i32 as usize]);
    }
    if !(*data).text[2 as i32 as usize].is_null() {
        HSD_SisLib_803A5CC4((*data).text[2 as i32 as usize]);
    }
    if !(*data).text[3 as i32 as usize].is_null() {
        HSD_SisLib_803A5CC4((*data).text[3 as i32 as usize]);
    }
    if !(*data).text[4 as i32 as usize].is_null() {
        HSD_SisLib_803A5CC4((*data).text[4 as i32 as usize]);
    }
    HSD_Free(arg0);
}
#[inline]
pub unsafe extern "C" fn mnDiagram_PopupAnimProc_Inline(
    mut arg0: *mut mnDiagram_AnimTable,
    mut arg1: i32,
) -> *mut Vec3 {
    melee_core::ct!("mnDiagram_PopupAnimProc_Inline");
    return (&raw mut (*arg0).points as *mut Point3d).offset(arg1 as isize) as *mut Vec3;
}

pub unsafe extern "C" fn mnDiagram_PopupAnimProc(mut arg0: *mut HSD_GObj) {
    melee_core::ct!("mnDiagram_PopupAnimProc");
    let mut data: *mut mnDiagram_PopupData = (*arg0).user_data
        as *mut mnDiagram_PopupData;
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut tbl: *mut mnDiagram_AnimTable = &raw mut mnDiagram_PcAnimTable;
    let mut pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut new_var: f32 = 0.;
    let mut anim_frame: f32 = 0.;
    let mut _pad: [u8; 24] = [0; 24];
    HSD_JObjAnimAll((*data).jobjs[5 as i32 as usize]);
    text = (*data).text[0 as i32 as usize];
    lb_8000B1CC(
        (*data).jobjs[8 as i32 as usize],
        mnDiagram_PopupAnimProc_Inline(tbl, 0 as i32),
        &raw mut pos,
    );
    let mut y: f32 = pos.y;
    let mut z: f32 = pos.z;
    (*text).pos_x = pos.x;
    new_var = -y as f32;
    (*text).pos_y = new_var as f32;
    (*text).pos_z = z;
    (*text).default_alignment = 0 as u8;
    HSD_JObjAnim((*data).jobjs[9 as i32 as usize]);
    if !(*data).text[1 as i32 as usize].is_null() {
        let mut t: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
        let mut y_0: f32 = 0.;
        let mut z_0: f32 = 0.;
        lb_8000B1CC(
            (*data).jobjs[11 as i32 as usize],
            mnDiagram_PopupAnimProc_Inline(tbl, 1 as i32),
            &raw mut pos,
        );
        y_0 = pos.y;
        z_0 = pos.z;
        t = (*data).text[1 as i32 as usize];
        (*t).pos_x = pos.x;
        new_var = -y_0 as f32;
        (*t).pos_y = new_var as f32;
        (*t).pos_z = z_0;
        (*text).default_alignment = 1 as u8;
    }
    if !(*data).text[2 as i32 as usize].is_null() {
        let mut t_0: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
        let mut y_1: f32 = 0.;
        let mut z_1: f32 = 0.;
        lb_8000B1CC(
            (*data).jobjs[10 as i32 as usize],
            mnDiagram_PopupAnimProc_Inline(tbl, 2 as i32),
            &raw mut pos,
        );
        y_1 = pos.y;
        z_1 = pos.z;
        t_0 = (*data).text[2 as i32 as usize];
        (*t_0).pos_x = pos.x;
        new_var = -y_1 as f32;
        (*t_0).pos_y = new_var as f32;
        (*t_0).pos_z = z_1;
        (*text).default_alignment = 1 as u8;
    }
    HSD_JObjAnim((*data).jobjs[1 as i32 as usize]);
    let mut t_1: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut y_2: f32 = 0.;
    let mut z_2: f32 = 0.;
    lb_8000B1CC(
        (*data).jobjs[3 as i32 as usize],
        mnDiagram_PopupAnimProc_Inline(tbl, 1 as i32),
        &raw mut pos,
    );
    y_2 = pos.y;
    z_2 = pos.z;
    t_1 = (*data).text[3 as i32 as usize];
    (*t_1).pos_x = pos.x;
    new_var = -y_2 as f32;
    (*t_1).pos_y = new_var as f32;
    (*t_1).pos_z = z_2;
    (*text).default_alignment = 1 as u8;
    if !(*data).text[4 as i32 as usize].is_null() {
        let mut t_2: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
        let mut y_3: f32 = 0.;
        let mut z_3: f32 = 0.;
        lb_8000B1CC(
            (*data).jobjs[2 as i32 as usize],
            mnDiagram_PopupAnimProc_Inline(tbl, 2 as i32),
            &raw mut pos,
        );
        y_3 = pos.y;
        z_3 = pos.z;
        t_2 = (*data).text[4 as i32 as usize];
        (*t_2).pos_x = pos.x;
        new_var = -y_3 as f32;
        (*t_2).pos_y = new_var as f32;
        (*t_2).pos_z = z_3;
        (*text).default_alignment = 1 as u8;
    }
    anim_frame = mn_8022EFD8(
        (*data).jobjs[12 as i32 as usize],
        &raw mut (*tbl).cursor_anim,
    ) as f32;
    let mut t_3: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut y_4: f32 = 0.;
    let mut z_4: f32 = 0.;
    lb_8000B1CC(
        (*data).jobjs[13 as i32 as usize],
        mnDiagram_PopupAnimProc_Inline(tbl, 1 as i32),
        &raw mut pos,
    );
    y_4 = pos.y;
    z_4 = pos.z;
    t_3 = (*data).text[3 as i32 as usize];
    (*t_3).pos_x = pos.x;
    new_var = -y_4 as f32;
    (*t_3).pos_y = new_var as f32;
    (*t_3).pos_z = z_4;
    (*text).default_alignment = 1 as u8;
    if anim_frame == (*tbl).cursor_anim.end_frame {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
    }
}
#[inline]
pub unsafe extern "C" fn mnDiagram_FormatPopupNumber(
    mut buf: *mut i8,
    mut val: u32,
) {
    melee_core::ct!("mnDiagram_FormatPopupNumber");
    let mut digit_count: i32 = mn_GetDigitCount(val as i32)
        as i32;
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < digit_count {
        *buf.offset((digit_count - 1 as i32 - i) as isize) = (mn_GetDigitAt(
            val as i32,
            i as i32,
        ) as i32 + '0' as i32) as i8;
        i += 1;
    }
    *buf.offset(digit_count as isize) = *(&raw mut mnDiagram_StringTerminator
        as *mut i8);
}

pub unsafe extern "C" fn mnDiagram_CreatePopupTexts(
    mut arg0: *mut HSD_GObj,
    mut selkind_or_nametag_slot_id: i32,
    mut arg2: i32,
    mut use_nametag: i32,
) {
    melee_core::ct!("mnDiagram_CreatePopupTexts");
    let mut data: *mut mnDiagram_PopupData = (*arg0).user_data
        as *mut mnDiagram_PopupData;
    let mut new_var: f32 = 0.;
    let mut pos: Point3d = Vec3 { x: 0., y: 0., z: 0. };
    let mut buf: [i8; 8] = [0; 8];
    let mut kos: u32 = 0;
    let mut sd_count: u32 = 0;
    let mut text: *mut HSD_Text = HSD_SisLib_803A6754(
        0 as i32,
        1 as i32,
    );
    let mut sp: [u8; 24] = [0; 24];
    (*data).text[0 as i32 as usize] = text;
    lb_8000B1CC(
        (*data).jobjs[8 as i32 as usize],
        (&raw mut mnDiagram_PopupTextOffsets.points as *mut Point3d)
            .offset(0 as i32 as isize) as *mut Vec3,
        &raw mut pos,
    );
    (*text).font_size.x = 0.0521f32 as f32;
    (*text).font_size.y = 0.0521f32 as f32;
    let mut y: f32 = pos.y;
    let mut z: f32 = pos.z;
    (*text).pos_x = pos.x;
    new_var = -y as f32;
    (*text).pos_y = new_var as f32;
    (*text).pos_z = z;
    (*text).default_alignment = 0 as u8;
    (*text).text_color = mnDiagram_PopupTextColor;
    if use_nametag != 0 as i32 {
        HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, GetNameText(selkind_or_nametag_slot_id as u8 as i32), &[]);
    } else {
        gm_80160C90(
            text,
            gm_SelKindToCKind(selkind_or_nametag_slot_id as u8),
            0 as u8,
        );
    }
    if use_nametag != 0 as i32 && selkind_or_nametag_slot_id != arg2 {
        let mut label_text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
        label_text = HSD_SisLib_803A6754(
            0 as i32,
            1 as i32,
        );
        (*data).text[2 as i32 as usize] = label_text;
        lb_8000B1CC(
            (*data).jobjs[10 as i32 as usize],
            (&raw mut mnDiagram_PopupTextOffsets.points as *mut Point3d)
                .offset(2 as i32 as isize) as *mut Vec3,
            &raw mut pos,
        );
        (*label_text).font_size.x = 0.035f32 as f32;
        (*label_text).font_size.y = 0.05f32 as f32;
        let mut y_0: f32 = pos.y;
        let mut z_0: f32 = pos.z;
        (*label_text).pos_x = pos.x;
        new_var = -y_0 as f32;
        (*label_text).pos_y = new_var as f32;
        (*label_text).pos_z = z_0;
        (*label_text).default_alignment = 1 as u8;
        HSD_SisLib_803A6B98(label_text, 0.0f32, 0.0f32, GetNameText(arg2 as i32 & 0xff as i32), &[]);
        let mut label_text_0: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
        label_text_0 = HSD_SisLib_803A6754(
            0 as i32,
            1 as i32,
        );
        (*data).text[4 as i32 as usize] = label_text_0;
        lb_8000B1CC(
            (*data).jobjs[2 as i32 as usize],
            (&raw mut mnDiagram_PcAnimTable.points as *mut Point3d)
                .offset(2 as i32 as isize) as *mut Vec3,
            &raw mut pos,
        );
        (*label_text_0).font_size.x = 0.035f32 as f32;
        (*label_text_0).font_size.y = 0.05f32 as f32;
        let mut y_1: f32 = pos.y;
        let mut z_1: f32 = pos.z;
        (*label_text_0).pos_x = pos.x;
        new_var = -y_1 as f32;
        (*label_text_0).pos_y = new_var as f32;
        (*label_text_0).pos_z = z_1;
        (*label_text_0).default_alignment = 1 as u8;
        HSD_SisLib_803A6B98(label_text_0, 0.0f32, 0.0f32, GetNameText(arg2 as u8 as i32), &[]);
    }
    if use_nametag == 0 as i32 || selkind_or_nametag_slot_id != arg2 {
        text = HSD_SisLib_803A6754(0 as i32, 1 as i32);
        (*data).text[1 as i32 as usize] = text;
        lb_8000B1CC(
            (*data).jobjs[11 as i32 as usize],
            (&raw mut mnDiagram_PopupTextOffsets.points as *mut Point3d)
                .offset(1 as i32 as isize) as *mut Vec3,
            &raw mut pos,
        );
        (*text).font_size.x = 0.0521f32 as f32;
        (*text).font_size.y = 0.0521f32 as f32;
        (*text).default_alignment = 1 as u8;
        let mut y_2: f32 = pos.y;
        let mut z_2: f32 = pos.z;
        (*text).pos_x = pos.x;
        new_var = -y_2 as f32;
        (*text).pos_y = new_var as f32;
        (*text).pos_z = z_2;
        if use_nametag != 0 as i32 {
            kos = (*GetPersistentNameData(selkind_or_nametag_slot_id as u8 as i32))
                .vs_kos[arg2 as u8 as usize] as u32;
            mnDiagram_FormatPopupNumber(&raw mut buf as *mut i8, kos);
        } else {
            kos = (*GetPersistentFighterData(
                selkind_or_nametag_slot_id as u8 as SelectableCharacterKind,
            ))
                .fighter_kos[arg2 as u8 as usize] as u32;
            mnDiagram_FormatPopupNumber(&raw mut buf as *mut i8, kos);
        }
        HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, &raw mut buf as *mut i8, &[]);
    }
    if selkind_or_nametag_slot_id == arg2 {
        text = HSD_SisLib_803A6754(0 as i32, 1 as i32);
        (*data).text[3 as i32 as usize] = text;
        lb_8000B1CC(
            (*data).jobjs[13 as i32 as usize],
            (&raw mut mnDiagram_PopupTextOffsets.points as *mut Point3d)
                .offset(1 as i32 as isize) as *mut Vec3,
            &raw mut pos,
        );
        (*text).font_size.x = 0.0521f32 as f32;
        (*text).font_size.y = 0.0521f32 as f32;
        (*text).default_alignment = 1 as u8;
        let mut y_3: f32 = pos.y;
        let mut z_3: f32 = pos.z;
        (*text).pos_x = pos.x;
        new_var = -y_3 as f32;
        (*text).pos_y = new_var as f32;
        (*text).pos_z = z_3;
        if use_nametag != 0 as i32 {
            sd_count = (*GetPersistentNameData(
                selkind_or_nametag_slot_id as u8 as i32,
            ))
                .stats
                .sd_count as u32;
            mnDiagram_FormatPopupNumber(
                &raw mut buf as *mut i8,
                sd_count,
            );
        } else {
            sd_count = (*GetPersistentFighterData(
                selkind_or_nametag_slot_id as u8 as SelectableCharacterKind,
            ))
                .stats
                .sd_count as u32;
            mnDiagram_FormatPopupNumber(
                &raw mut buf as *mut i8,
                sd_count,
            );
        }
        HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, &raw mut buf as *mut i8, &[]);
    } else {
        text = HSD_SisLib_803A6754(0 as i32, 1 as i32);
        (*data).text[3 as i32 as usize] = text;
        lb_8000B1CC(
            (*data).jobjs[3 as i32 as usize],
            (&raw mut mnDiagram_PopupTextOffsets.points as *mut Point3d)
                .offset(1 as i32 as isize) as *mut Vec3,
            &raw mut pos,
        );
        (*text).font_size.x = 0.0521f32 as f32;
        (*text).font_size.y = 0.0521f32 as f32;
        (*text).default_alignment = 1 as u8;
        let mut y_4: f32 = pos.y;
        let mut z_4: f32 = pos.z;
        (*text).pos_x = pos.x;
        new_var = -y_4 as f32;
        (*text).pos_y = new_var as f32;
        (*text).pos_z = z_4;
        if use_nametag != 0 as i32 {
            let mut count: u32 = (*GetPersistentNameData(arg2 as u8 as i32))
                .vs_kos[selkind_or_nametag_slot_id as u8 as usize] as u32;
            mnDiagram_FormatPopupNumber(&raw mut buf as *mut i8, count);
        } else {
            let mut count_0: u32 = (*GetPersistentFighterData(
                arg2 as u8 as SelectableCharacterKind,
            ))
                .fighter_kos[selkind_or_nametag_slot_id as u8 as usize] as u32;
            mnDiagram_FormatPopupNumber(
                &raw mut buf as *mut i8,
                count_0,
            );
        }
        HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, &raw mut buf as *mut i8, &[]);
    };
}

pub unsafe extern "C" fn mnDiagram_CreatePopup(
    mut arg0: i32,
    mut arg1: i32,
    mut use_nametag: i32,
) {
    melee_core::ct!("mnDiagram_CreatePopup");
    let mut i: i32 = 0;
    let mut tbl: *mut mnDiagram_AnimTable = ::core::ptr::null_mut::<
        mnDiagram_AnimTable,
    >();
    let mut model: *mut StaticModelDesc = ::core::ptr::null_mut::<StaticModelDesc>();
    let mut data: *mut Diagram = ::core::ptr::null_mut::<Diagram>();
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut user_data: *mut mnDiagram_PopupData = ::core::ptr::null_mut::<
        mnDiagram_PopupData,
    >();
    tbl = &raw mut mnDiagram_PcAnimTable;
    model = &raw mut MenMainSubB1_Top;
    data = HSD_GObjGetUserData(mnDiagram_ScreenGObj) as *mut Diagram;
    gobj = GObj_Create(6 as u16, 7 as u8, 0x80 as u8);
    (*data).popup_gobj = gobj;
    jobj = HSD_JObjLoadJoint((*model).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut ::core::ffi::c_void);
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
        (*model).animjoint,
        (*model).matanim_joint,
        (*model).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0f32);
    HSD_JObjAnimAll(jobj);
    user_data = HSD_MemAlloc(::core::mem::size_of::<mnDiagram_PopupData>() as i32)
        as *mut mnDiagram_PopupData;
    if user_data.is_null() {
        OSReport(&raw mut (*tbl).user_data_error as *mut i8, &[]);
        __assert(
            &raw mut (*tbl).file_name as *mut i8,
            0x5f8 as u32,
            &raw mut (*tbl).user_data_name as *mut i8,
        );
    }
    GObj_InitUserData(
        gobj,
        0 as u8,
        Some(
            mnDiagram_PopupCleanup
                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        user_data as *mut ::core::ffi::c_void,
    );
    (*user_data).text[4 as i32 as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*user_data).text[3 as i32 as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*user_data).text[2 as i32 as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*user_data).text[1 as i32 as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*user_data).text[0 as i32 as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    i = 0 as i32;
    while i < 14 as i32 {
        lb_80011E24(jobj as *mut HSD_JObj, (&raw mut (*user_data).jobjs as *mut *mut HSD_JObj).offset(i as isize)
                as *mut *mut HSD_JObj, &[Va::I32(i), Va::I32(-(1 as i32))]);
        i += 1;
    }
    HSD_GObj_SetupProc(
        gobj,
        Some(mnDiagram_PopupAnimProc as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
    mnDiagram_CreatePopupTexts(gobj, arg0, arg1, use_nametag);
    if use_nametag != 0 as i32 {
        if arg0 == arg1 {
            HSD_JObjSetFlagsAll(
                (*user_data).jobjs[9 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        }
        if arg0 == arg1 {
            HSD_JObjSetFlagsAll(
                (*user_data).jobjs[1 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        }
        if arg0 != arg1 {
            HSD_JObjSetFlagsAll(
                (*user_data).jobjs[12 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        }
    } else {
        let mut icon: *mut HSD_JObj = mnDiagram_CreateFighterIcon(
            arg0 as i32,
            1 as i32,
        );
        mn_8022F3D8(icon as *mut HSD_JObj, 1 as u8, TOBJ_MASK);
        mn_8022F3D8(icon as *mut HSD_JObj, 0xa as u8, TOBJ_MASK);
        HSD_JObjAddChild((*user_data).jobjs[7 as i32 as usize], icon);
        icon = mnDiagram_CreateFighterIcon(
            arg1 as i32,
            1 as i32,
        );
        HSD_JObjSetTranslateX(icon, -1.0f32);
        mn_8022F3D8(icon as *mut HSD_JObj, 1 as u8, TOBJ_MASK);
        HSD_JObjAddChild((*user_data).jobjs[10 as i32 as usize], icon);
        if arg0 == arg1 {
            HSD_JObjSetFlagsAll(
                (*user_data).jobjs[1 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        } else {
            icon = mnDiagram_CreateFighterIcon(
                arg1 as i32,
                1 as i32,
            );
            HSD_JObjSetTranslateX(icon, -1.0f32);
            mn_8022F3D8(icon as *mut HSD_JObj, 1 as u8, TOBJ_MASK);
            HSD_JObjAddChild((*user_data).jobjs[2 as i32 as usize], icon);
        }
        if arg0 != arg1 {
            HSD_JObjSetFlagsAll(
                (*user_data).jobjs[12 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        }
    };
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetUserData(
    mut gobj: *mut HSD_GObj,
) -> *mut ::core::ffi::c_void {
    melee_core::ct!("mnDiagram_GetUserData");
    return (*gobj).user_data;
}

pub unsafe extern "C" fn mnDiagram_ClearGrid(mut arg0: *mut HSD_GObj) {
    melee_core::ct!("mnDiagram_ClearGrid");
    let mut data: *mut Diagram = mnDiagram_GetUserData(arg0)
        as *mut Diagram;
    let mut child: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    child = HSD_JObjGetChild((*data).jobjs[7]);
    if !child.is_null() {
        HSD_JObjRemoveAll(child);
    }
    child = HSD_JObjGetChild((*data).jobjs[9]);
    if !child.is_null() {
        HSD_JObjRemoveAll(child);
    }
    child = HSD_JObjGetChild((*data).jobjs[11]);
    if !child.is_null() {
        HSD_JObjRemoveAll(child);
    }
    if !(*data).col_header_text.is_null() {
        HSD_SisLib_803A5CC4((*data).col_header_text);
        (*data).col_header_text = ::core::ptr::null_mut::<
            HSD_Text,
        >();
    }
    if !(*data).row_header_text.is_null() {
        HSD_SisLib_803A5CC4((*data).row_header_text);
        (*data).row_header_text = ::core::ptr::null_mut::<
            HSD_Text,
        >();
    }
}

pub unsafe extern "C" fn mnDiagram_RefreshGrid(
    mut arg0: *mut HSD_GObj,
    mut arg1: i32,
    mut arg2: i32,
) {
    melee_core::ct!("mnDiagram_RefreshGrid");
    let mut data: *mut Diagram = HSD_GObjGetUserData(arg0) as *mut Diagram;
    mnDiagram_ClearGrid(arg0);
    mnDiagram_DrawGridValues(
        arg0,
        arg1 as i32,
        arg2 as i32,
        ((*data).is_name_mode as i32 == 1 as i32)
            as i32 as u8,
    );
    if (*data).is_name_mode as i32 == 0 as i32 {
        mnDiagram_DrawFighterHeaders(arg0, arg1, arg2);
    } else {
        mnDiagram_DrawNameHeaders(arg0, arg1 as i32, arg2 as i32);
    };
}

pub unsafe extern "C" fn mnDiagram_UpdateScrollArrows(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mnDiagram_UpdateScrollArrows");
    let mut result2: u8 = 0;
    let mut data: *mut Diagram = (*gobj).user_data as *mut Diagram;
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut ptr2: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut ptr: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut count: i32 = 0;
    let mut i: i32 = 0;
    let mut sorted: *mut u8 = &raw mut *(&raw mut mnDiagram_PcBlock.sorted_fighters
        as *mut u8 as *mut [u8; 28]) as *mut u8;
    let mut result: i32 = 0;
    let mut name_count: i32 = 0;
    let mut jobj2: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut jobj3: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut _pad: [u8; 8] = [0; 8];
    jobj = (*data).jobjs[3 as i32 as usize] as *mut HSD_JObj;
    mn_8022ED6C(jobj as *mut HSD_JObj, &raw mut mnDiagram_ArrowAnim);
    if (*data).is_name_mode as i32 != 0 as i32 {
        result = mnDiagram_GetVisibleNameFrom(
            sorted,
            (*data).name_cursor_pos as u8 as i32,
            10 as i32,
        ) as i32;
        if result as u8 as i32 != 0x78 as i32 {
            HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        } else {
            HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    } else {
        result = mnDiagram_GetVisibleFighterCursorFrom(
            sorted,
            (*data).fighter_cursor_pos as u8 as i32,
            10 as i32,
        ) as i32;
        if result as u8 as i32 != 0x19 as i32 {
            HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        } else {
            HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    }
    jobj2 = (*data).jobjs[4 as i32 as usize] as *mut HSD_JObj;
    mn_8022ED6C(jobj2 as *mut HSD_JObj, &raw mut mnDiagram_ArrowAnim);
    if (*data).is_name_mode as i32 != 0 as i32 {
        result = (*data).name_cursor_pos as u8 as i32;
    } else {
        result = (*data).fighter_cursor_pos as u8 as i32;
    }
    if result != 0 as i32 {
        HSD_JObjClearFlagsAll(jobj2, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjSetFlagsAll(jobj2, JOBJ_HIDDEN as u32);
    }
    jobj2 = (*data).jobjs[5 as i32 as usize] as *mut HSD_JObj;
    mn_8022ED6C(jobj2 as *mut HSD_JObj, &raw mut mnDiagram_ArrowAnim);
    if (*data).is_name_mode as i32 != 0 as i32 {
        i = ((*data).name_cursor_pos as i32 >> 8 as i32)
            as i32;
    } else {
        i = ((*data).fighter_cursor_pos as i32 >> 8 as i32)
            as i32;
    }
    if i != 0 as i32 {
        HSD_JObjClearFlagsAll(jobj2, JOBJ_HIDDEN as u32);
    } else {
        HSD_JObjSetFlagsAll(jobj2, JOBJ_HIDDEN as u32);
    }
    jobj3 = (*data).jobjs[6 as i32 as usize] as *mut HSD_JObj;
    mn_8022ED6C(jobj3 as *mut HSD_JObj, &raw mut mnDiagram_ArrowAnim);
    if (*data).is_name_mode as i32 != 0 as i32 {
        let mut current_block_62: u64;
        name_count = 7 as i32 as i32;
        i = ((*data).name_cursor_pos as i32 >> 8 as i32)
            as i32;
        ptr = sorted.offset(i as isize);
        ptr = ptr.offset(0x1c as i32 as isize);
        's_217: loop {
            if !(name_count > 0 as i32) {
                current_block_62 = 9627623479216730126;
                break;
            }
            ptr2 = ptr;
            loop {
                i += 1;
                ptr2 = ptr2.offset(1);
                ptr = ptr.offset(1);
                if i >= 0x78 as i32 {
                    result = 0x78 as i32 as i32;
                    current_block_62 = 9668934872721435809;
                    break 's_217;
                } else if !GetNameText(*ptr2 as i32).is_null() {
                    break;
                }
            }
            name_count -= 1;
        }
        match current_block_62 {
            9627623479216730126 => {
                ptr = sorted;
                ptr = ptr.offset(i as isize);
                result = *ptr.offset(0x1c as i32 as isize) as i32;
            }
            _ => {}
        }
        if result as u8 as i32 != 0x78 as i32 {
            HSD_JObjClearFlagsAll(jobj3, JOBJ_HIDDEN as u32);
        } else {
            HSD_JObjSetFlagsAll(jobj3, JOBJ_HIDDEN as u32);
        }
    } else {
        count = 7 as i32 as i32;
        i = ((*data).fighter_cursor_pos as i32 >> 8 as i32)
            as i32;
        ptr2 = sorted.offset(i as isize);
        's_305: loop {
            if count == 0 as i32 {
                result2 = *sorted.offset(i as isize);
                break;
            } else {
                ptr = ptr2;
                loop {
                    i += 1;
                    ptr = ptr.offset(1);
                    ptr2 = ptr2.offset(1);
                    if i >= 0x19 as i32 {
                        result2 = 0x19 as u8;
                        break 's_305;
                    } else if !(mn_IsFighterUnlocked(*ptr as SelectableCharacterKind)
                        == 0 as i32)
                    {
                        break;
                    }
                }
                count -= 1;
                if !(count >= 0 as i32) {
                    break;
                }
            }
        }
        if result2 as i32 != 0x19 as i32 {
            HSD_JObjClearFlagsAll(jobj3, JOBJ_HIDDEN as u32);
        } else {
            HSD_JObjSetFlagsAll(jobj3, JOBJ_HIDDEN as u32);
        }
    };
}

pub unsafe extern "C" fn mnDiagram_ExitAnimProc(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mnDiagram_ExitAnimProc");
    let mut data: *mut Diagram = ::core::ptr::null_mut::<
        Diagram,
    >();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    data = (*gobj).user_data as *mut Diagram;
    mnDiagram_UpdateScrollArrows(gobj);
    jobj = (*data).jobjs[1];
    if mn_8022ED6C(jobj as *mut HSD_JObj, &raw mut mnDiagram_PopupExitAnim)
        >= mnDiagram_PopupExitAnim.end_frame
    {
        HSD_GObjFree(gobj);
    }
}

pub unsafe extern "C" fn mnDiagram_UpdateScrollArrowVisibility(
    mut gobj: *mut HSD_GObj,
    mut count: i32,
) {
    melee_core::ct!("mnDiagram_UpdateScrollArrowVisibility");
    let mut data: *mut Diagram = (*gobj).user_data as *mut Diagram;
    let mut _pad: [u8; 8] = [0; 8];
    if count <= 7 as i32 {
        HSD_JObjSetFlagsAll(
            (*data).jobjs[5 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
        HSD_JObjSetFlagsAll(
            (*data).jobjs[6 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
    } else {
        HSD_JObjClearFlagsAll(
            (*data).jobjs[5 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
        HSD_JObjClearFlagsAll(
            (*data).jobjs[6 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
    }
    if count <= 10 as i32 {
        HSD_JObjSetFlagsAll(
            (*data).jobjs[4 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
        HSD_JObjSetFlagsAll(
            (*data).jobjs[3 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
    } else {
        HSD_JObjClearFlagsAll(
            (*data).jobjs[4 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
        HSD_JObjClearFlagsAll(
            (*data).jobjs[3 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
    };
}

pub unsafe extern "C" fn mnDiagram_OnFrame(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mnDiagram_OnFrame");
    let mut data: *mut Diagram = HSD_GObjGetUserData(gobj) as *mut Diagram;
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut data2: *mut Diagram = ::core::ptr::null_mut::<Diagram>();
    let mut count: i32 = 0;
    if mn_804A04F0.cur_menu as i32 != 0x1e as i32
        || mn_804A04F0.x10 as i32 != 0 as i32
    {
        if mn_804A04F0.cur_menu as i32 == 0x1e as i32 {
            mnDiagram_ClearGrid(gobj);
            HSD_GObjFree(gobj);
            return;
        }
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(mnDiagram_ExitAnimProc as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
        HSD_JObjSetFlagsAll(
            (*data).jobjs[2 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
        mnDiagram_ClearGrid(gobj);
        return;
    }
    if (*data).anim_state as i32 == 1 as i32 {
        let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
        let mut frame: f32 = 0.;
        let mut end_frame: f32 = 0.;
        frame = mn_8022ED6C(
            (*data).jobjs[1 as i32 as usize],
            &raw mut mnDiagram_IntroAnim,
        ) as f32;
        end_frame = mnDiagram_IntroAnim.end_frame;
        jobj = (*data).jobjs[2 as i32 as usize] as *mut HSD_JObj;
        if frame >= end_frame {
            HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
            (*data).anim_state = 0 as u8;
            mnDiagram_CreateCursor();
            if (*data).is_name_mode as i32 != 0 as i32 {
                let mut indices: u16 = (*data).name_cursor_pos;
                let mut row_idx: i32 = 0;
                let mut col_idx: u8 = 0;
                data2 = (*gobj).user_data as *mut Diagram;
                row_idx = (indices as i32 >> 8 as i32)
                    as i32;
                col_idx = indices as u8;
                mnDiagram_ClearGrid(gobj);
                mnDiagram_DrawGridValues(
                    gobj,
                    col_idx as i32,
                    row_idx,
                    ((*data2).is_name_mode as i32
                        == 1 as i32) as i32 as u8,
                );
                if (*data2).is_name_mode as i32 == 0 as i32
                {
                    mnDiagram_DrawFighterHeaders(
                        gobj,
                        col_idx as i32,
                        row_idx as i32,
                    );
                } else {
                    mnDiagram_DrawNameHeaders(gobj, col_idx as i32, row_idx);
                }
            } else {
                let mut indices_0: u16 = (*data).fighter_cursor_pos;
                let mut row_idx_0: i32 = 0;
                let mut col_idx_0: u8 = 0;
                data2 = (*gobj).user_data as *mut Diagram;
                row_idx_0 = (indices_0 as i32 >> 8 as i32)
                    as i32;
                col_idx_0 = indices_0 as u8;
                mnDiagram_ClearGrid(gobj);
                mnDiagram_DrawGridValues(
                    gobj,
                    col_idx_0 as i32,
                    indices_0 as i32 >> 8 as i32,
                    ((*data2).is_name_mode as i32
                        == 1 as i32) as i32 as u8,
                );
                if (*data2).is_name_mode as i32 == 0 as i32
                {
                    mnDiagram_DrawFighterHeaders(
                        gobj,
                        col_idx_0 as i32,
                        row_idx_0 as i32,
                    );
                } else {
                    mnDiagram_DrawNameHeaders(gobj, col_idx_0 as i32, row_idx_0);
                }
            }
            if (*data).is_name_mode as i32 != 0 as i32 {
                count = GetNameCount();
            } else {
                count = mnDiagram_CountUnlockedFighters();
            }
            data2 = (*gobj).user_data as *mut Diagram;
            if count <= 7 as i32 {
                HSD_JObjSetFlagsAll(
                    (*data2).jobjs[5 as i32 as usize],
                    JOBJ_HIDDEN as u32,
                );
                HSD_JObjSetFlagsAll(
                    (*data2).jobjs[6 as i32 as usize],
                    JOBJ_HIDDEN as u32,
                );
            } else {
                HSD_JObjClearFlagsAll(
                    (*data2).jobjs[5 as i32 as usize],
                    JOBJ_HIDDEN as u32,
                );
                HSD_JObjClearFlagsAll(
                    (*data2).jobjs[6 as i32 as usize],
                    JOBJ_HIDDEN as u32,
                );
            }
            if count <= 10 as i32 {
                HSD_JObjSetFlagsAll(
                    (*data2).jobjs[4 as i32 as usize],
                    JOBJ_HIDDEN as u32,
                );
                HSD_JObjSetFlagsAll(
                    (*data2).jobjs[3 as i32 as usize],
                    JOBJ_HIDDEN as u32,
                );
            } else {
                HSD_JObjClearFlagsAll(
                    (*data2).jobjs[4 as i32 as usize],
                    JOBJ_HIDDEN as u32,
                );
                HSD_JObjClearFlagsAll(
                    (*data2).jobjs[3 as i32 as usize],
                    JOBJ_HIDDEN as u32,
                );
            }
        } else {
            HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    }
    mnDiagram_UpdateScrollArrows(gobj);
}

pub unsafe extern "C" fn mnDiagram_DrawCellValue(
    mut arg0: *mut HSD_GObj,
    mut arg1: u8,
    mut arg2: u8,
    mut arg3: i32,
) {
    melee_core::ct!("mnDiagram_DrawCellValue");
    let mut data_alias: *mut Diagram = ::core::ptr::null_mut::<Diagram>();
    let mut row_offset_adj: f32 = 0.;
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut jobj2: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut data: *mut Diagram = ::core::ptr::null_mut::<Diagram>();
    let mut model: *mut StaticModelDesc = ::core::ptr::null_mut::<StaticModelDesc>();
    let mut digit_count: i32 = 0;
    let mut digit: i32 = 0;
    let mut i: i32 = 0;
    let mut x_spacing: f32 = 0.;
    let mut y_spacing: f32 = 0.;
    let mut base: f32 = 0.;
    let mut rowf: f32 = 0.;
    let mut row_offset: f32 = 0.;
    let mut col_offset: f32 = 0.;
    let mut col: u8 = arg1;
    let mut row: u8 = arg2;
    let mut y_offset: f32 = 0.;
    data = (*arg0).user_data as *mut Diagram;
    data_alias = data;
    jobj = (*data).jobjs[11 as i32 as usize] as *mut HSD_JObj;
    base = HSD_JObjGetTranslationX(jobj);
    jobj2 = (*data).jobjs[12 as i32 as usize] as *mut HSD_JObj;
    x_spacing = HSD_JObjGetTranslationX(jobj2) - base;
    jobj = (*data).jobjs[7 as i32 as usize] as *mut HSD_JObj;
    base = HSD_JObjGetTranslationX(jobj);
    jobj2 = (*data).jobjs[8 as i32 as usize] as *mut HSD_JObj;
    y_spacing = HSD_JObjGetTranslationX(jobj2) - base;
    jobj = (*data).jobjs[9 as i32 as usize] as *mut HSD_JObj;
    base = HSD_JObjGetTranslationY(jobj);
    jobj2 = (*data).jobjs[10 as i32 as usize] as *mut HSD_JObj;
    y_offset = HSD_JObjGetTranslationY(jobj2) - base;
    digit_count = mn_GetDigitCount(arg3 as i32);
    col_offset = y_spacing * col as f32;
    rowf = row as f32;
    row_offset = y_offset * rowf;
    row_offset_adj = (row_offset as f32 - 0.4f32) as f32;
    model = &raw mut MenMainNmB_Top;
    i = 0 as i32 as i32;
    while i < digit_count {
        digit = mn_GetDigitAt(arg3 as i32, i);
        jobj = HSD_JObjLoadJoint((*model).joint as *mut HSD_Joint);
        HSD_JObjAddAnimAll(
            jobj,
            (*model).animjoint,
            (*model).matanim_joint,
            (*model).shapeanim_joint,
        );
        base = digit as f32;
        HSD_JObjReqAnimAll(jobj, base);
        HSD_JObjAnimAll(jobj);
        if (col as i32) < 7 as i32 {
            HSD_JObjSetTranslateX(jobj, melee_core::fp::fmadds(x_spacing, i as f32, col_offset));
        } else {
            HSD_JObjSetTranslateX(jobj, melee_core::fp::fmadds(x_spacing, i as f32, col_offset) + 0.4f32);
        }
        if (row as i32) < 10 as i32 {
            HSD_JObjSetTranslateY(jobj, row_offset);
        } else {
            HSD_JObjSetTranslateY(jobj, row_offset_adj);
        }
        HSD_JObjAddChild((*data_alias).jobjs[11 as i32 as usize], jobj);
        i += 1;
    }
}
#[inline]
pub unsafe extern "C" fn mnDiagram_GetFighterPairKOs(
    mut fighter: u8,
    mut opponent: u8,
) -> i32 {
    melee_core::ct!("mnDiagram_GetFighterPairKOs");
    let mut kind: SelectableCharacterKind = fighter as SelectableCharacterKind;
    return (*GetPersistentFighterData(kind)).fighter_kos[opponent as usize]
        as i32;
}

pub unsafe extern "C" fn mnDiagram_DrawGridValues(
    mut arg0: *mut HSD_GObj,
    mut row_start: i32,
    mut col_start: i32,
    mut arg3: u8,
) {
    melee_core::ct!("mnDiagram_DrawGridValues");
    let mut name_col: i32 = 0;
    let mut is_name_mode: u8 = arg3;
    let mut unlocked_count: i32 = 0;
    let mut col_unlocked_count: i32 = 0;
    let mut bottom_unlocked_count: i32 = 0;
    let mut bottom_col: i32 = 0;
    let mut fighter_col: i32 = 0;
    let mut row: i32 = 0;
    let mut entry_count: i32 = 0;
    let mut total_kos: i32 = 0;
    let mut row_name: i32 = 0;
    let mut col_name: i32 = 0;
    let mut row_fighter: i32 = 0;
    let mut col_fighter: u8 = 0;
    let mut _pad: [u8; 16] = [0; 16];
    row = 0 as i32 as i32;
    while row <= 0xa as i32 {
        if row == 0xa as i32 {
            bottom_col = 0 as i32 as i32;
            while bottom_col < 7 as i32 {
                if is_name_mode as i32 != 0 as i32 {
                    entry_count = GetNameCount() as i32;
                    if entry_count > bottom_col {
                        mnDiagram_DrawCellValue(
                            arg0,
                            bottom_col as u8,
                            row as u8,
                            mnDiagram_SumNameFalls(
                                mnDiagram_GetVisibleNameCursorFrom(
                                    col_start as i32,
                                    bottom_col as i32,
                                ),
                            ),
                        );
                    }
                } else {
                    bottom_unlocked_count = mnDiagram_CountUnlockedFightersForHeaders()
                        as i32;
                    if bottom_unlocked_count > bottom_col {
                        mnDiagram_DrawCellValue(
                            arg0,
                            bottom_col as u8,
                            row as u8,
                            mnDiagram_SumFighterFalls(
                                mnDiagram_GetVisibleFighterCursorFrom2(
                                    col_start as i32,
                                    bottom_col as i32,
                                ),
                            ),
                        );
                    }
                }
                bottom_col += 1;
            }
        } else if is_name_mode as i32 != 0 as i32 {
            entry_count = GetNameCount() as i32;
            if entry_count > row {
                name_col = 0 as i32 as i32;
                while name_col <= 7 as i32 {
                    if name_col == 7 as i32
                        || {
                            entry_count = GetNameCount() as i32;
                            entry_count > name_col
                        }
                    {
                        row_name = mnDiagram_GetVisibleNameCursorFrom(
                            row_start as i32,
                            row as i32,
                        ) as i32;
                        if name_col == 7 as i32 {
                            total_kos = mnDiagram_GetNameTotalKOs(row_name as u8)
                                as i32;
                            mnDiagram_DrawCellValue(
                                arg0,
                                name_col as u8,
                                row as u8,
                                total_kos as i32,
                            );
                        } else {
                            let mut ko_count: i32 = 0;
                            col_name = mnDiagram_GetVisibleNameCursorFrom(
                                col_start as i32,
                                name_col as i32,
                            ) as i32;
                            ko_count = (*GetPersistentNameData(row_name as u8 as i32))
                                .vs_kos[col_name as usize] as i32;
                            mnDiagram_DrawCellValue(
                                arg0,
                                name_col as u8,
                                row as u8,
                                ko_count,
                            );
                        }
                    }
                    name_col += 1 as i32;
                }
            }
        } else {
            unlocked_count = mnDiagram_CountUnlockedFightersForHeaders() as i32;
            if unlocked_count > row {
                fighter_col = 0 as i32 as i32;
                while fighter_col <= 7 as i32 {
                    if fighter_col == 7 as i32
                        || {
                            col_unlocked_count = mnDiagram_CountUnlockedFightersForHeaders()
                                as i32;
                            col_unlocked_count > fighter_col
                        }
                    {
                        row_fighter = mnDiagram_GetVisibleFighterCursorFrom2(
                            row_start as i32,
                            row as i32,
                        ) as i32;
                        if fighter_col == 7 as i32 {
                            total_kos = mnDiagram_GetFighterTotalKOs(row_fighter as u8)
                                as i32;
                            total_kos = (if total_kos > 999999 as i32 {
                                999999 as i32
                            } else {
                                total_kos as i32
                            }) as i32;
                            mnDiagram_DrawCellValue(
                                arg0,
                                fighter_col as u8,
                                row as u8,
                                total_kos as i32,
                            );
                        } else {
                            col_fighter = mnDiagram_GetVisibleFighterCursorFrom2(
                                col_start as i32,
                                fighter_col as i32,
                            );
                            mnDiagram_DrawCellValue(
                                arg0,
                                fighter_col as u8,
                                row as u8,
                                mnDiagram_GetFighterPairKOs(
                                    row_fighter as u8,
                                    col_fighter,
                                ),
                            );
                        }
                    }
                    fighter_col += 1 as i32;
                }
            }
        }
        row += 1 as i32;
    }
}
pub unsafe extern "C" fn mnDiagram_DrawNameHeaders(
    mut arg0: *mut HSD_GObj,
    mut arg1: i32,
    mut arg2: i32,
) {
    melee_core::ct!("mnDiagram_DrawNameHeaders");
    let mut data: *mut Diagram = (*arg0).user_data as *mut Diagram;
    let mut sorted: *mut u8 = &raw mut *(&raw mut mnDiagram_PcBlock.sorted_fighters
        as *mut u8 as *mut [u8; 28]) as *mut u8;
    let mut row_text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut name_byte: u8 = 0;
    let mut name_id: i32 = 0;
    let mut pos: Vec2 = Vec2 { x: 0., y: 0. };
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    text = HSD_SisLib_803A6754(0 as i32, 1 as i32);
    (*data).col_header_text = text;
    (*text).font_size.x = 0.02f32 as f32;
    (*text).font_size.y = 0.03f32 as f32;
    let mut j: *mut HSD_JObj = (*data).jobjs[7 as i32 as usize];
    mnDiagram_TextSetPos(&mut *text,
        -1.3f32 + HSD_JObjGetTranslationX(j),
        -0.5f32 - HSD_JObjGetTranslationY(j),
        HSD_JObjGetTranslationZ(j) as f32,
    );
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < 7 as i32 {
        if GetNameCount() > i {
            let mut x_spacing: f32 = 0.;
            name_byte = mnDiagram_GetVisibleNameFrom(
                sorted,
                arg2 as i32,
                i,
            );
            name_id = name_byte as i32;
            x_spacing = HSD_JObjGetTranslationX(
                (*data).jobjs[8 as i32 as usize],
            ) - HSD_JObjGetTranslationX((*data).jobjs[7 as i32 as usize]);
            HSD_SisLib_803A6B98(text, x_spacing as f32 * i as f32 / 0.02f32, 0.0f32, GetNameText(name_id as i32), &[]);
        }
        i += 1;
    }
    row_text = HSD_SisLib_803A6754(0 as i32, 1 as i32);
    (*data).row_header_text = row_text;
    (*row_text).font_size.x = 0.02f32 as f32;
    (*row_text).font_size.y = 0.03f32 as f32;
    let mut j_0: *mut HSD_JObj = (*data).jobjs[9 as i32 as usize];
    let mut z: f32 = HSD_JObjGetTranslationZ(j_0) as f32;
    pos.y = -0.5f32 - HSD_JObjGetTranslationY(j_0);
    pos.x = -1.3f32 + HSD_JObjGetTranslationX(j_0);
    (*row_text).pos_x = pos.x;
    (*row_text).pos_y = pos.y;
    (*row_text).pos_z = z;
    let mut i_0: i32 = 0;
    i_0 = 0 as i32;
    while i_0 < 0xa as i32 {
        if GetNameCount() > i_0 {
            let mut y_spacing: f32 = 0.;
            name_id = (mnDiagram_GetVisibleNameFrom2(
                sorted,
                arg1 as i32,
                i_0,
            ) as u64
                & 0xffffffffffffffff as u64) as i32;
            y_spacing = HSD_JObjGetTranslationY(
                (*data).jobjs[10 as i32 as usize],
            ) - HSD_JObjGetTranslationY((*data).jobjs[9 as i32 as usize]);
            HSD_SisLib_803A6B98(row_text, 0.0f32, -(y_spacing as f32 * i_0 as f32
                    / 0.03f32), GetNameText(name_id as i32), &[]);
        }
        i_0 += 1;
    }
}

pub unsafe extern "C" fn mnDiagram_CreateFighterIcon(
    mut idx: i32,
    mut arg1: i32,
) -> *mut HSD_JObj {
    melee_core::ct!("mnDiagram_CreateFighterIcon");
    let mut sp10: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut model: *mut StaticModelDesc = &raw mut MenMainFaceB_Top;
    let mut temp_r3: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut var_f1: f32 = 0.;
    temp_r3 = HSD_JObjLoadJoint((*model).joint as *mut HSD_Joint);
    HSD_JObjAddAnimAll(
        temp_r3,
        (*model).animjoint,
        (*model).matanim_joint,
        (*model).shapeanim_joint,
    );
    if arg1 != 0 as i32 {
        var_f1 = 1.0f32 as f32;
    } else {
        var_f1 = 0.0f32 as f32;
    }
    HSD_JObjReqAnimAll(temp_r3, var_f1);
    HSD_JObjAnimAll(temp_r3);
    lb_80011E24(temp_r3 as *mut HSD_JObj, &raw mut sp10, &[Va::I32(2 as i32), Va::I32(-(1 as i32))]);
    HSD_JObjReqAnimAll(sp10, idx as f32);
    HSD_JObjAnimAll(sp10);
    return temp_r3;
}
#[inline]
pub unsafe extern "C" fn mnDiagram_LoadHeaderIcon(
    mut joint_data: *mut *mut ::core::ffi::c_void,
    mut fighter_id: i32,
    mut child: *mut *mut HSD_JObj,
) -> *mut HSD_JObj {
    melee_core::ct!("mnDiagram_LoadHeaderIcon");
    let mut jobj: *mut HSD_JObj = HSD_JObjLoadJoint(
        *joint_data.offset(0 as i32 as isize) as *mut HSD_Joint,
    );
    HSD_JObjAddAnimAll(
        jobj,
        *joint_data.offset(1 as i32 as isize) as *mut HSD_AnimJoint,
        *joint_data.offset(2 as i32 as isize) as *mut HSD_MatAnimJoint,
        *joint_data.offset(3 as i32 as isize) as *mut HSD_ShapeAnimJoint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0f32);
    HSD_JObjAnimAll(jobj);
    lb_80011E24(jobj as *mut HSD_JObj, child as *mut *mut HSD_JObj, &[Va::I32(2 as i32), Va::I32(-(1 as i32))]);
    HSD_JObjReqAnimAll(*child, (fighter_id & 0xff as i32) as f32);
    HSD_JObjAnimAll(*child);
    return jobj;
}

pub unsafe extern "C" fn mnDiagram_DrawFighterHeaders(
    mut arg0: *mut HSD_GObj,
    mut arg1: i32,
    mut arg2: i32,
) {
    melee_core::ct!("mnDiagram_DrawFighterHeaders");
    let mut col_ref: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut row_ref: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut col_cursor: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut sorted: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut col_next: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut row_next: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut row_fighter: i32 = 0;
    let mut unlocked_count: i32 = 0;
    let mut joint_data: *mut *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
        *mut ::core::ffi::c_void,
    >();
    let mut row_cursor: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut row_idx: i32 = 0;
    let mut row_remaining: i32 = 0;
    let mut data: *mut Diagram = HSD_GObjGetUserData(arg0) as *mut Diagram;
    let mut col_fighter: i32 = 0;
    let mut col_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut row_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut col_idx: i32 = 0;
    let mut col_remaining: i32 = 0;
    let mut assets: *mut mnDiagram_Assets = &raw mut mnDiagram_PcBlock.sorted_fighters
        as *mut u8 as *mut [u8; 28] as *mut mnDiagram_Assets;
    let mut x_spacing: f32 = 0.;
    let mut y_spacing: f32 = 0.;
    let mut i: i32 = 0;
    let mut _pad: [u8; 8] = [0; 8];
    i = 0 as i32;
    while i < 7 as i32 {
        sorted = &raw mut *(&raw mut mnDiagram_PcBlock.sorted_fighters as *mut u8
            as *mut [u8; 28]) as *mut u8;
        joint_data = &raw mut (*assets).FaceB as *mut *mut ::core::ffi::c_void;
        unlocked_count = mnDiagram_CountUnlockedFightersForHeaders() as i32;
        if unlocked_count > i {
            let mut child: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            col_remaining = i;
            col_idx = arg2;
            col_cursor = sorted.offset(col_idx as isize);
            's_89: while col_remaining >= 0 as i32 {
                if col_remaining == 0 as i32 {
                    col_fighter = *sorted.offset(col_idx as isize) as i32;
                    break;
                } else {
                    col_next = col_cursor;
                    loop {
                        col_idx += 1;
                        col_next = col_next.offset(1);
                        col_cursor = col_cursor.offset(1);
                        if col_idx >= SELKIND_COUNT as i32 {
                            col_fighter = SELKIND_COUNT as i32;
                            break 's_89;
                        } else if !(mn_IsFighterUnlocked(
                            *col_next as SelectableCharacterKind,
                        ) == 0 as i32)
                        {
                            break;
                        }
                    }
                    col_remaining -= 1;
                }
            }
            col_jobj = mnDiagram_LoadHeaderIcon(joint_data, col_fighter, &raw mut child);
            col_ref = (*data).jobjs[7 as i32 as usize] as *mut HSD_JObj;
            let mut reference_x: f32 = HSD_JObjGetTranslationX(col_ref);
            col_ref = (*data).jobjs[8 as i32 as usize] as *mut HSD_JObj;
            x_spacing = HSD_JObjGetTranslationX(col_ref) - reference_x;
            HSD_JObjSetTranslateX(col_jobj, x_spacing * i as f32);
            HSD_JObjAddChild((*data).jobjs[7 as i32 as usize], col_jobj);
        }
        i += 1;
    }
    joint_data = &raw mut (*assets).FaceB as *mut *mut ::core::ffi::c_void;
    i = 0 as i32;
    while i < 10 as i32 {
        sorted = &raw mut *(&raw mut mnDiagram_PcBlock.sorted_fighters as *mut u8
            as *mut [u8; 28]) as *mut u8;
        unlocked_count = mnDiagram_CountUnlockedFightersForHeaders() as i32;
        if unlocked_count > i {
            let mut row_child: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            row_remaining = i;
            row_idx = arg1;
            row_cursor = sorted.offset(row_idx as isize);
            's_204: while row_remaining >= 0 as i32 {
                if row_remaining == 0 as i32 {
                    row_fighter = *sorted.offset(row_idx as isize) as i32;
                    break;
                } else {
                    row_next = row_cursor;
                    loop {
                        row_idx += 1;
                        row_next = row_next.offset(1);
                        row_cursor = row_cursor.offset(1);
                        if row_idx >= SELKIND_COUNT as i32 {
                            row_fighter = SELKIND_COUNT as i32;
                            break 's_204;
                        } else if !(mn_IsFighterUnlocked(
                            *row_next as SelectableCharacterKind,
                        ) == 0 as i32)
                        {
                            break;
                        }
                    }
                    row_remaining -= 1;
                }
            }
            row_jobj = HSD_JObjLoadJoint(
                *joint_data.offset(0 as i32 as isize) as *mut HSD_Joint,
            );
            HSD_JObjAddAnimAll(
                row_jobj,
                *joint_data.offset(1 as i32 as isize)
                    as *mut HSD_AnimJoint,
                *joint_data.offset(2 as i32 as isize)
                    as *mut HSD_MatAnimJoint,
                *joint_data.offset(3 as i32 as isize)
                    as *mut HSD_ShapeAnimJoint,
            );
            HSD_JObjReqAnimAll(row_jobj, 0.0f32);
            HSD_JObjAnimAll(row_jobj);
            lb_80011E24(row_jobj as *mut HSD_JObj, &raw mut row_child, &[Va::I32(2 as i32), Va::I32(-(1 as i32))]);
            HSD_JObjReqAnimAll(
                row_child,
                (row_fighter & 0xff as i32) as f32,
            );
            HSD_JObjAnimAll(row_child);
            row_ref = (*data).jobjs[9 as i32 as usize] as *mut HSD_JObj;
            let mut reference_y: f32 = HSD_JObjGetTranslationY(row_ref);
            row_ref = (*data).jobjs[10 as i32 as usize] as *mut HSD_JObj;
            y_spacing = HSD_JObjGetTranslationY(row_ref) - reference_y;
            HSD_JObjSetTranslateY(row_jobj, y_spacing * i as f32);
            HSD_JObjAddChild((*data).jobjs[9 as i32 as usize], row_jobj);
        }
        i += 1;
    }
}

pub unsafe extern "C" fn mnDiagram_CursorProc(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("mnDiagram_CursorProc");
    let mut sp_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut selection: *mut u16 = ::core::ptr::null_mut::<u16>();
    let mut col: i32 = 0;
    let mut row: i32 = 0;
    let mut x_spacing: f32 = 0.;
    let mut y_spacing: f32 = 0.;
    let mut data: *mut Diagram = ::core::ptr::null_mut::<Diagram>();
    let mut _pad: [u8; 8] = [0; 8];
    if mn_804A04F0.cur_menu as i32 != 0x1e as i32
        || mn_804A04F0.x10 as i32 != 0 as i32
    {
        HSD_GObjFree(gobj);
        return;
    }
    data = mnDiagram_GetCurrentDiagramData();
    lb_80011E24((*gobj).hsd_obj as *mut HSD_JObj, &raw mut sp_jobj, &[Va::I32(3 as i32), Va::I32(-(1 as i32))]);
    selection = &raw mut mn_804A04F0 as *mut u16;
    selection = selection.offset(1);
    col = *selection as i32 >> 8 as i32;
    x_spacing = HSD_JObjGetTranslationX((*data).jobjs[8 as i32 as usize])
        - HSD_JObjGetTranslationX((*data).jobjs[7 as i32 as usize]);
    HSD_JObjSetTranslateX(sp_jobj, x_spacing * (col - 3 as i32) as f32);
    lb_80011E24((*gobj).hsd_obj as *mut HSD_JObj, &raw mut sp_jobj, &[Va::I32(4 as i32), Va::I32(-(1 as i32))]);
    row = *selection as i32 & 0xff as i32;
    y_spacing = HSD_JObjGetTranslationY((*data).jobjs[10 as i32 as usize])
        - HSD_JObjGetTranslationY((*data).jobjs[9 as i32 as usize]);
    HSD_JObjSetTranslateY(
        sp_jobj,
        (y_spacing as f64 * (row as f64 - 4.5f64)
            - 0.1f64) as f32,
    );
    lb_80011E24((*gobj).hsd_obj as *mut HSD_JObj, &raw mut sp_jobj, &[Va::I32(2 as i32), Va::I32(-(1 as i32))]);
    HSD_JObjSetTranslateX(sp_jobj, x_spacing * (col - 3 as i32) as f32);
    HSD_JObjSetTranslateY(
        sp_jobj,
        (y_spacing as f64 * (row as f64 - 4.5f64)
            - 0.1f64) as f32,
    );
}

pub unsafe extern "C" fn mnDiagram_CreateCursor() {
    melee_core::ct!("mnDiagram_CreateCursor");
    let mut model: *mut StaticModelDesc = ::core::ptr::null_mut::<StaticModelDesc>();
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut _pad: [u8; 40] = [0; 40];
    model = &raw mut MenMainCursorB1_Top;
    gobj = GObj_Create(6 as u16, 7 as u8, 0x80 as u8);
    jobj = HSD_JObjLoadJoint((*model).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut ::core::ffi::c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, i32) -> (),
        ),
        4 as u8,
        0x80 as u32,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(mnDiagram_CursorProc as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
}

pub unsafe extern "C" fn mnDiagram_CreateScreen(mut arg0: u8) {
    melee_core::ct!("mnDiagram_CreateScreen");
    let mut col_idx: i32 = 0;
    let mut row_idx: i32 = 0;
    let mut d: *mut Diagram = ::core::ptr::null_mut::<Diagram>();
    let mut col_idx2: i32 = 0;
    let mut row_idx2: i32 = 0;
    let mut d2: *mut Diagram = ::core::ptr::null_mut::<Diagram>();
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut anim_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut user_data: *mut Diagram = ::core::ptr::null_mut::<Diagram>();
    let mut data2: *mut Diagram = ::core::ptr::null_mut::<Diagram>();
    let mut count: i32 = 0;
    let mut model: *mut StaticModelDesc = ::core::ptr::null_mut::<StaticModelDesc>();
    let mut i: i32 = 0;
    let mut indices: u16 = 0;
    let mut stack_obj: [u8; 8] = [0; 8];
    &raw mut stack_obj;
    model = &raw mut MenMainConB1_Top;
    gobj = GObj_Create(6 as u16, 7 as u8, 0x80 as u8);
    mnDiagram_ScreenGObj = gobj;
    jobj = HSD_JObjLoadJoint((*model).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut ::core::ffi::c_void);
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
        (*model).animjoint,
        (*model).matanim_joint,
        (*model).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0f32);
    user_data = HSD_MemAlloc(::core::mem::size_of::<Diagram>() as i32)
        as *mut Diagram;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const i8
                as *mut i8, &[]);
        __assert(
            b"src/melee/mn/mndiagram.c\0" as *const u8 as *const i8,
            2781 as u32,
            b"user_data\0" as *const u8 as *const i8,
        );
    };
    (*user_data).saved_menu = mn_804A04F0.cur_menu as u8;
    (*user_data).saved_selection = mn_804A04F0.hovered_selection;
    (*user_data).anim_state = arg0;
    (*user_data).fighter_cursor_pos = (*user_data).fighter_cursor_pos as u8 as u16;
    (*user_data).fighter_cursor_pos = ((*user_data).fighter_cursor_pos
        as i32 & 0xff00 as i32) as u16;
    (*user_data).name_cursor_pos = (*user_data).name_cursor_pos as u8 as u16;
    (*user_data).name_cursor_pos = ((*user_data).name_cursor_pos as i32
        & 0xff00 as i32) as u16;
    (*user_data).popup_gobj = ::core::ptr::null_mut::<HSD_GObj>();
    (*user_data).row_header_text = ::core::ptr::null_mut::<HSD_Text>();
    (*user_data).col_header_text = ::core::ptr::null_mut::<HSD_Text>();
    if GetNameCount() != 0 as i32 {
        (*user_data).is_name_mode = (*gmMainLib_GetGameRules()).xD;
    } else {
        (*gmMainLib_GetGameRules()).xD = 0 as u8;
        (*user_data).is_name_mode = (*gmMainLib_GetGameRules()).xD;
    }
    GObj_InitUserData(
        gobj,
        0 as u8,
        Some(HSD_Free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        user_data as *mut ::core::ffi::c_void,
    );
    i = 0 as i32;
    while i < 13 as i32 {
        lb_80011E24(jobj as *mut HSD_JObj, (&raw mut (*user_data).jobjs as *mut *mut HSD_JObj).offset(i as isize)
                as *mut *mut HSD_JObj, &[Va::I32(i), Va::I32(-(1 as i32))]);
        i += 1;
    }
    HSD_GObj_SetupProc(
        gobj,
        Some(mnDiagram_OnFrame as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
    if arg0 as i32 == 0 as i32 {
        anim_jobj = (*user_data).jobjs[1 as i32 as usize]
            as *mut HSD_JObj;
        HSD_JObjReqAnimAll(anim_jobj, mnDiagram_IntroAnim.end_frame);
        HSD_JObjAnimAll(anim_jobj);
        mnDiagram_CreateCursor();
        if (*user_data).is_name_mode as i32 != 0 as i32 {
            count = GetNameCount();
        } else {
            count = mnDiagram_CountUnlockedFighters();
        }
        data2 = HSD_GObjGetUserData(gobj) as *mut Diagram;
        if count <= 7 as i32 {
            HSD_JObjSetFlagsAll(
                (*data2).jobjs[5 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
            HSD_JObjSetFlagsAll(
                (*data2).jobjs[6 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        } else {
            HSD_JObjClearFlagsAll(
                (*data2).jobjs[5 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
            HSD_JObjClearFlagsAll(
                (*data2).jobjs[6 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        }
        if count <= 0xa as i32 {
            HSD_JObjSetFlagsAll(
                (*data2).jobjs[4 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
            HSD_JObjSetFlagsAll(
                (*data2).jobjs[3 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        } else {
            HSD_JObjClearFlagsAll(
                (*data2).jobjs[4 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
            HSD_JObjClearFlagsAll(
                (*data2).jobjs[3 as i32 as usize],
                JOBJ_HIDDEN as u32,
            );
        }
        if (*user_data).is_name_mode as i32 != 0 as i32 {
            indices = (*user_data).name_cursor_pos;
            row_idx = (indices as i32 >> 8 as i32) as i32;
            col_idx = indices as i32 & 0xff as i32;
            d = HSD_GObjGetUserData(gobj) as *mut Diagram;
            mnDiagram_ClearGrid(gobj);
            mnDiagram_DrawGridValues(
                gobj,
                col_idx as i32,
                row_idx,
                ((*d).is_name_mode as i32 == 1 as i32)
                    as i32 as u8,
            );
            if (*d).is_name_mode as i32 == 0 as i32 {
                mnDiagram_DrawFighterHeaders(
                    gobj,
                    col_idx,
                    row_idx as i32,
                );
            } else {
                mnDiagram_DrawNameHeaders(gobj, col_idx as i32, row_idx);
            }
        } else {
            indices = (*user_data).fighter_cursor_pos;
            row_idx2 = (indices as i32 >> 8 as i32) as i32;
            col_idx2 = indices as i32 & 0xff as i32;
            d2 = HSD_GObjGetUserData(gobj) as *mut Diagram;
            mnDiagram_ClearGrid(gobj);
            mnDiagram_DrawGridValues(
                gobj,
                col_idx2 as i32,
                row_idx2,
                ((*d2).is_name_mode as i32 == 1 as i32)
                    as i32 as u8,
            );
            if (*d2).is_name_mode as i32 == 0 as i32 {
                mnDiagram_DrawFighterHeaders(
                    gobj,
                    col_idx2,
                    row_idx2 as i32,
                );
            } else {
                mnDiagram_DrawNameHeaders(gobj, col_idx2 as i32, row_idx2);
            }
        }
    }
}

pub unsafe extern "C" fn mnDiagram_Init(mut arg0: u8, mut arg1: u8) {
    melee_core::ct!("mnDiagram_Init");
    let mut assets: *mut mnDiagram_Assets = &raw mut mnDiagram_PcBlock.sorted_fighters
        as *mut u8 as *mut [u8; 28] as *mut mnDiagram_Assets;
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut archive: *mut HSD_Archive = ::core::ptr::null_mut::<HSD_Archive>();
    let mut mode_storage: [u8; 4] = [0; 4];
    mode_storage[0 as i32 as usize] = arg1;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x1e as MenuKind8;
    mn_804A04F0.x10 = 0 as u8;
    mn_804A04F0.hovered_selection = 0 as u16;
    if arg0 != 0 {
        archive = mn_804D6BB8;
        lbArchive_LoadSections(archive, (&raw mut (*assets).ConB1 as *mut *mut ::core::ffi::c_void)
                .offset(0 as i32 as isize)
                as *mut *mut ::core::ffi::c_void, &[(b"MenMainConB1_Top_joint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).ConB1 as *mut *mut ::core::ffi::c_void)
                .offset(1 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainConB1_Top_animjoint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).ConB1 as *mut *mut ::core::ffi::c_void)
                .offset(2 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainConB1_Top_matanim_joint\0" as *const u8
                as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).ConB1 as *mut *mut ::core::ffi::c_void)
                .offset(3 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainConB1_Top_shapeanim_joint\0" as *const u8
                as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).CursorB1 as *mut *mut ::core::ffi::c_void)
                .offset(0 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainCursorB1_Top_joint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).FaceB as *mut *mut ::core::ffi::c_void)
                .offset(0 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainFaceB_Top_joint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).FaceB as *mut *mut ::core::ffi::c_void)
                .offset(1 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainFaceB_Top_animjoint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).FaceB as *mut *mut ::core::ffi::c_void)
                .offset(2 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainFaceB_Top_matanim_joint\0" as *const u8
                as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).FaceB as *mut *mut ::core::ffi::c_void)
                .offset(3 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainFaceB_Top_shapeanim_joint\0" as *const u8
                as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).NmB as *mut *mut ::core::ffi::c_void)
                .offset(0 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainNmB_Top_joint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).NmB as *mut *mut ::core::ffi::c_void)
                .offset(1 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainNmB_Top_animjoint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).NmB as *mut *mut ::core::ffi::c_void)
                .offset(2 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainNmB_Top_matanim_joint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).NmB as *mut *mut ::core::ffi::c_void)
                .offset(3 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainNmB_Top_shapeanim_joint\0" as *const u8
                as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).SubB1 as *mut *mut ::core::ffi::c_void)
                .offset(0 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainSubB1_Top_joint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).SubB1 as *mut *mut ::core::ffi::c_void)
                .offset(1 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainSubB1_Top_animjoint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).SubB1 as *mut *mut ::core::ffi::c_void)
                .offset(2 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainSubB1_Top_matanim_joint\0" as *const u8
                as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).SubB1 as *mut *mut ::core::ffi::c_void)
                .offset(3 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainSubB1_Top_shapeanim_joint\0" as *const u8
                as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).ConB2 as *mut *mut ::core::ffi::c_void)
                .offset(0 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainConB2_Top_joint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).ConB2 as *mut *mut ::core::ffi::c_void)
                .offset(1 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainConB2_Top_animjoint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).ConB2 as *mut *mut ::core::ffi::c_void)
                .offset(2 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainConB2_Top_matanim_joint\0" as *const u8
                as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).ConB2 as *mut *mut ::core::ffi::c_void)
                .offset(3 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainConB2_Top_shapeanim_joint\0" as *const u8
                as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).ConB3 as *mut *mut ::core::ffi::c_void)
                .offset(0 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainConB3_Top_joint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).ConB3 as *mut *mut ::core::ffi::c_void)
                .offset(1 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainConB3_Top_animjoint\0" as *const u8 as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).ConB3 as *mut *mut ::core::ffi::c_void)
                .offset(2 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainConB3_Top_matanim_joint\0" as *const u8
                as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).ConB3 as *mut *mut ::core::ffi::c_void)
                .offset(3 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainConB3_Top_shapeanim_joint\0" as *const u8
                as *const i8) as *const core::ffi::c_void, ((&raw mut (*assets).CursorB3 as *mut *mut ::core::ffi::c_void)
                .offset(0 as i32 as isize)
                as *mut *mut ::core::ffi::c_void) as *const core::ffi::c_void, (b"MenMainCursorB3_Top_joint\0" as *const u8 as *const i8) as *const core::ffi::c_void, (0 as i32) as *const core::ffi::c_void]);
        let map = [&raw mut MenMainSubB1_Top, &raw mut MenMainNmB_Top, &raw mut MenMainFaceB_Top, &raw mut MenMainCursorB1_Top, &raw mut MenMainConB1_Top, &raw mut MenMainConB2_Top, &raw mut MenMainConB3_Top, &raw mut MenMainCursorB3_Top];
        let mut src: [*mut *mut ::core::ffi::c_void; 8] = [::core::ptr::null_mut::<
            *mut ::core::ffi::c_void,
        >(); 8];
        let mut k: i32 = 0;
        src[0 as i32 as usize] = &raw mut (*assets).SubB1
            as *mut *mut ::core::ffi::c_void;
        src[1 as i32 as usize] = &raw mut (*assets).NmB
            as *mut *mut ::core::ffi::c_void;
        src[2 as i32 as usize] = &raw mut (*assets).FaceB
            as *mut *mut ::core::ffi::c_void;
        src[3 as i32 as usize] = &raw mut (*assets).CursorB1
            as *mut *mut ::core::ffi::c_void;
        src[4 as i32 as usize] = &raw mut (*assets).ConB1
            as *mut *mut ::core::ffi::c_void;
        src[5 as i32 as usize] = &raw mut (*assets).ConB2
            as *mut *mut ::core::ffi::c_void;
        src[6 as i32 as usize] = &raw mut (*assets).ConB3
            as *mut *mut ::core::ffi::c_void;
        src[7 as i32 as usize] = &raw mut (*assets).CursorB3
            as *mut *mut ::core::ffi::c_void;
        k = 0 as i32;
        while k
            < 8usize
                as i32
        {
            (*map[k as usize]).joint = *src[k as usize]
                .offset(0 as i32 as isize) as *mut HSD_Joint
                as *mut HSD_Joint;
            (*map[k as usize]).animjoint = *src[k as usize]
                .offset(1 as i32 as isize) as *mut HSD_AnimJoint;
            (*map[k as usize]).matanim_joint = *src[k as usize]
                .offset(2 as i32 as isize) as *mut HSD_MatAnimJoint;
            (*map[k as usize]).shapeanim_joint = *src[k as usize]
                .offset(3 as i32 as isize) as *mut HSD_ShapeAnimJoint;
            k += 1;
        }
    }
    mnDiagram_SortFightersByKOs();
    mnDiagram_SortNamesByKOs();
    mnDiagram_CreateScreen(mode_storage[0 as i32 as usize]);
    gobj = GObj_Create(0 as u16, 1 as u8, 0x80 as u8);
    proc_0 = HSD_GObj_SetupProc(
        gobj,
        Some(mnDiagram_InputProc as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8 as u8);
}

// ---- folded from mn_ptr.rs (consolidation steps 21/22) ----
/// mndiagram.c `mnDiagram_TextSetPos`.
pub fn mnDiagram_TextSetPos(text: &mut HSD_Text_mn_ptr, x: f32, y: f32, z: f32) {
    melee_core::ct!("mnDiagram_TextSetPos");
    text.pos_x = x;
    text.pos_y = y;
    text.pos_z = z;
}
