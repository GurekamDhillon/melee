//! Port of src/melee/mn/mncount.c: the "Count" page of the Records menu (power count, play times,
//! match totals, and the per-character rankings: longest time, smash champ, KO kingpin ...).
//!
//! Callees live in unported TUs and are `ext` stubs named like the C. State is the file's globals
//! (`globals!`); tests/tu_mncount.rs checks every function against the C compiled natively.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use melee_types::{
    AnimLoopSettings, CountEntry, FighterData, GObj_RenderFunc, GXColor, HSD_AnimJoint, HSD_Archive, HSD_GObj, HSD_GObjEvent,
    HSD_GObjProc, HSD_Joint, HSD_JObj, HSD_MatAnimJoint, HSD_ShapeAnimJoint, HSD_Text, MenuFlow, MenuInputState, MnCountData,
    SelectableCharacterKind, StaticModelDesc, JOBJ_HIDDEN, MENU_KIND_RECORDS, MENU_KIND_RECORDS_MISC, SELKIND_COUNT,
};

use super::hsd::{sfxBack, sfxMove};
use crate::oracle::Va;

pub use melee_types::{
    AVAILABLE_CHARACTERS, AVAILABLE_MAPS, BONUS_MATCH_TOTAL, COIN_MATCH_TOTAL, COMBINED_VS_PLAY_TIME, DISASTER_MASTER, KO_KINGPIN,
    KO_TOTAL, LONGEST_TIME, MATCH_RESET_COUNTER, NAME_TOTAL, NO_DEFENSE_NELLY, PLAY_TIME, POWER_COUNT, POWER_TIME, PUNCHING_BAG,
    SECOND_LONGEST_TIME, SELFDESTRUCT_TOTAL, SHORTEST_TIME, SINGLEPLAYER_TIME, SLUG_MEISTER, SMASH_CHAMP, SMASH_SAP,
    STOCK_MATCH_TOTAL, TIME_MATCH_TOTAL, TOTAL_DAMAGE, TROPHY_TOTAL, VS_PLAY_CONTESTANTS, VS_PLAY_MATCH_TOTAL, VS_PLAY_TIME,
};

const NUM_STAGES: i32 = 29;
const MAX_SCROLL: u8 = 20;
const MNCOUNT_VISIBLE_ROWS: usize = 10;
const MENU_INPUT_BACK: i64 = 1 << 5;
const MENU_INPUT_UP: i64 = 1 << 0;
const MENU_INPUT_DOWN: i64 = 1 << 1;

pub mod ext {
    use super::*;
    crate::oext_fns! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void)>, data: *mut c_void) -> ();
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32) -> ();
        fn GetNameCount() -> i32 = crate::tu::adapt::GetNameCount__b961fe;
        fn GetPersistentFighterData(a0: SelectableCharacterKind) -> *mut FighterData = crate::tu::adapt::GetPersistentFighterData__578ad2;
        fn HSD_Free(ptr: *mut c_void) -> () = crate::tu::adapt::HSD_Free__3ee311;
        fn HSD_GObjFree(a0: *mut HSD_GObj) -> ();
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void) -> ();
        fn HSD_GObjProc_RemoveProc(gproc: *mut HSD_GObjProc) -> ();
        fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: isize) -> () = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint) -> ();
        fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, flags: u32) -> ();
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32) -> ();
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32) -> ();
        fn HSD_MemAlloc(size: i32) -> *mut c_void = crate::tu::adapt::HSD_MemAlloc__c2b3bc;
        fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A5ACC__27fcb1;
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) -> () = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
        fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) -> () = crate::tu::adapt::HSD_SisLib_803A6368__2e8f0d;
        fn HSD_SisLib_803A6754(arg0: i32, arg1: i32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A6754__02fb41;
        fn Toy_GetTrophyTotal() -> i32 = crate::tu::adapt::Toy_GetTrophyTotal__16ecae;
        fn __assert(a0: *const i8, a1: u32, a2: *const i8) -> ();
        fn gmMainLib_GetBonusMatchTotal() -> *mut u32 = crate::tu::adapt::gmMainLib_GetBonusMatchTotal__02f786;
        fn gmMainLib_GetCoinMatchTotal() -> *mut u32 = crate::tu::adapt::gmMainLib_GetCoinMatchTotal__b5956d;
        fn gmMainLib_GetCombinedVSPlayTime() -> *mut c_void = crate::tu::adapt::gmMainLib_GetCombinedVSPlayTime__1127ef;
        fn gmMainLib_GetKOTotal() -> *mut i32 = crate::tu::adapt::gmMainLib_GetKOTotal__df5440;
        fn gmMainLib_GetMatchResetCounter() -> *mut u32 = crate::tu::adapt::gmMainLib_GetMatchResetCounter__75c307;
        fn gmMainLib_GetPowerCount() -> *mut c_void = crate::tu::adapt::gmMainLib_GetPowerCount__4fdcf9;
        fn gmMainLib_GetSelfDestructTotal() -> *mut c_void = crate::tu::adapt::gmMainLib_GetSelfDestructTotal__53cf84;
        fn gmMainLib_GetSingleplayerTime() -> *mut c_void = crate::tu::adapt::gmMainLib_GetSingleplayerTime__730b1b;
        fn gmMainLib_GetStockMatchTotal() -> *mut u32 = crate::tu::adapt::gmMainLib_GetStockMatchTotal__5e0d06;
        fn gmMainLib_GetTimeMatchTotal() -> *mut u32 = crate::tu::adapt::gmMainLib_GetTimeMatchTotal__16c4ef;
        fn gmMainLib_GetTotalDamage() -> *mut c_void = crate::tu::adapt::gmMainLib_GetTotalDamage__6cca88;
        fn gmMainLib_GetVsPlayContestants() -> *mut c_void = crate::tu::adapt::gmMainLib_GetVsPlayContestants__57546a;
        fn gmMainLib_GetVsPlayTime() -> *mut c_void = crate::tu::adapt::gmMainLib_GetVsPlayTime__b5e5c4;
        fn gm_80160B40(a0: *mut HSD_Text, a1: u8, a2: u8) -> () = crate::tu::adapt::gm_80160B40__54949f;
        fn gm_801641CC(a0: u8) -> u16 = crate::tu::adapt::gm_801641CC__003261;
        fn gm_80164430(a0: u16) -> i32 = crate::tu::adapt::gm_80164430__8808be;
        fn gm_GetPlayTime() -> i32 = crate::tu::adapt::gm_GetPlayTime__729b79;
        fn gm_GetPowerTime() -> *mut i32 = crate::tu::adapt::gm_GetPowerTime__6cb0de;
        fn gm_GetVsPlayMatchTotal() -> u32 = crate::tu::adapt::gm_GetVsPlayMatchTotal__0222a7;
        fn gm_IsCKindUnlocked(ckind: u8) -> i32 = crate::tu::adapt::gm_IsCKindUnlocked__b6c64b;
        fn gm_SelKindToCKind(selkind: u8) -> u8 = crate::tu::adapt::gm_SelKindToCKind__b7f955;
        fn lbAudioAx_80024030(a0: i32) -> () = crate::tu::adapt::lbAudioAx_80024030__36458a;
        fn mnDiagram_GetFighterTotalFalls(field_index: u8) -> i32 = crate::tu::adapt::mnDiagram_GetFighterTotalFalls__925ac9;
        fn mnDiagram_GetFighterTotalKOs(field_index: u8) -> i32 = crate::tu::adapt::mnDiagram_GetFighterTotalKOs__14ab9d;
        fn mn_80229624(a0: u32) -> u32 = crate::tu::adapt::mn_80229624__84b7ce;
        fn mn_80229894(a0: i32, a1: u16, a2: i32) -> () = crate::tu::adapt::mn_80229894__f7df64;
        fn mn_8022EA78(buf: *mut i8, buf_end: i32, num: u32) -> () = crate::tu::adapt::mn_8022EA78__f27e4b;
        fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32 = crate::tu::adapt::mn_8022ED6C__f73cae;
    }
    crate::ext_va! {
        /// os: `(fmt, ...)`
        fn OSReport(fmt: *const i8) va 0 -> ();
        /// sislib.c: `(text, x, y, fmt, ...)`
        fn HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const i8) va 3 -> i32;
    }
    crate::ext_vk! {
        /// lbspdisplay: `(jobj, out, idx, -1)`
        fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj) vk "ii" -> i32;
    }

    /// lbarchive.c, variadic in C: `lbArchive_LoadSections(archive, out0, name0, out1, name1, ..., 0)`.
    pub unsafe fn lbArchive_LoadSections(archive: *mut HSD_Archive, out0: *mut *mut c_void, _rest: &[*const c_void]) { crate::tu::ext_hook::archive_sections(archive, out0, _rest) }
}

const TEXT_COLOR: GXColor = GXColor { r: 0xAA, g: 0xAA, b: 0xAA, a: 0xFF };

// menuguard: the menu state is mnmain's and gobj.c's, not a private copy (a NULL mn_804D6BB8 faulted the Event menu)
pub use super::mnmain::{mn_804A04F0, mn_804D4B64, mn_804D6BB8, mn_804D6BC8};
pub use melee_runtime::tu::gobj_native::{HSD_GObj_804D783C, HSD_GObj_JObjKind};
// menuguard: the menu state is mnmain's and gobj.c's, not a private copy (a NULL mn_804D6BB8 faulted the Event menu)
pub use melee_runtime::tu::gobj_native::{HSD_GObj_CurrentInvokedProc};
// the oracle state still covers the re-exported menu / gobj globals (registered in the C oracle's order, tests/tu_mncount.rs)
crate::globals_ext! {
    rw mn_804A04F0 = super::mnmain::mn_804A04F0: MenuFlow;
    rw mn_804D6BC8 = super::mnmain::mn_804D6BC8: MenuInputState;
    rw mn_804D6BB8 = super::mnmain::mn_804D6BB8: *mut HSD_Archive;
    rw mn_804D4B64 = super::mnmain::mn_804D4B64: GXColor;
    rw HSD_GObj_CurrentInvokedProc = melee_runtime::tu::gobj_native::HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc;
    rw HSD_GObj_804D783C = melee_runtime::tu::gobj_native::HSD_GObj_804D783C: i32;
    rw HSD_GObj_JObjKind = melee_runtime::tu::gobj_native::HSD_GObj_JObjKind: u8;
}
crate::globals! {
    // ---- defined by this TU
    ro pub mnCount_803EFA88: [AnimLoopSettings; 2] = [
        AnimLoopSettings { start_frame: 0.0, end_frame: 19.0, loop_frame: -0.1 },
        AnimLoopSettings { start_frame: 20.0, end_frame: 29.0, loop_frame: -0.1 },
    ];
    ro mnCount_803EFAA0: AnimLoopSettings = AnimLoopSettings { start_frame: 0.0, end_frame: 199.0, loop_frame: 0.0 };
    rw pub model_desc: StaticModelDesc = StaticModelDesc { joint: null_mut(), animjoint: null_mut(), matanim_joint: null_mut(), shapeanim_joint: null_mut() };
    rw pub menu_gobj: *mut HSD_GObj = null_mut();
    ro mnCount_sis_idx: [u16; 30] = [
        0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF, 0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC,
        0xDD, 0xDE, 0xDF, 0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6,
    ];
    // ---- defined by other TUs (placeholders until they are ported)
}

#[inline]
unsafe fn get_mncount(g: *mut HSD_GObj) -> *mut MnCountData {
    (*g).user_data.cast()
}

/// `static inline inline_is_row_time`.
pub fn inline_is_row_time(row: i32) -> bool {
    melee_core::ct!("inline_is_row_time");
    matches!(row as u32, POWER_TIME | PLAY_TIME | SINGLEPLAYER_TIME | VS_PLAY_TIME | COMBINED_VS_PLAY_TIME)
}

/// `static inline inline_is_row_char`.
pub fn inline_is_row_char(row: i32) -> bool {
    melee_core::ct!("inline_is_row_char");
    matches!(
        row as u32,
        LONGEST_TIME | SECOND_LONGEST_TIME | SHORTEST_TIME | SMASH_CHAMP | SMASH_SAP | SLUG_MEISTER | PUNCHING_BAG | KO_KINGPIN
            | NO_DEFENSE_NELLY | DISASTER_MASTER
    )
}

/// `static inline inline_update_entries`.
pub unsafe fn inline_update_entries(gobj: *mut HSD_GObj) {
    melee_core::ct!("inline_update_entries");
    let userdata = get_mncount(gobj);
    for i in 0..MNCOUNT_VISIBLE_ROWS as i32 {
        mnCount_CreateRow(gobj, i, (*userdata).scroll_pos as u32 + i as u32);
    }
}

pub unsafe fn mnCount_8025035C_inline() -> bool {
    melee_core::ct!("mnCount_8025035C_inline");
    for i in 0..SELKIND_COUNT {
        if (*ext::GetPersistentFighterData(i)).stats.play_time != 0 {
            return false;
        }
    }
    true
}

pub unsafe extern "C" fn mnCount_GetMatchTime(selkind: SelectableCharacterKind) -> u32 {
    melee_core::ct!("mnCount_GetMatchTime");
    (*ext::GetPersistentFighterData(selkind)).stats.play_time
}

pub unsafe extern "C" fn mnCount_GetKOKingpin(selkind: SelectableCharacterKind) -> u32 {
    melee_core::ct!("mnCount_GetKOKingpin");
    ext::mnDiagram_GetFighterTotalKOs(selkind as u8) as u32
}

pub unsafe extern "C" fn mnCount_GetNoDefenseNelly(selkind: SelectableCharacterKind) -> u32 {
    melee_core::ct!("mnCount_GetNoDefenseNelly");
    ext::mnDiagram_GetFighterTotalFalls(selkind as u8) as u32
}

pub unsafe extern "C" fn mnCount_GetDisasterMaster(selkind: SelectableCharacterKind) -> u32 {
    melee_core::ct!("mnCount_GetDisasterMaster");
    (*ext::GetPersistentFighterData(selkind)).stats.sd_count as u32
}

pub type GetVal = Option<unsafe extern "C" fn(SelectableCharacterKind) -> u32>;

/// 25035C
pub unsafe extern "C" fn mnCount_8025035C(mut skip_count: i32, get_val_func: GetVal) -> i32 {
    melee_core::ct!("mnCount_8025035C");
    let f = get_val_func.unwrap();
    let n = SELKIND_COUNT as i32;
    let mut sp18 = [CountEntry { selkind: 0, pad: [0; 3], stat_value: 0 }; SELKIND_COUNT as usize];
    if mnCount_8025035C_inline() {
        return n;
    }
    let mut i: i32 = 0;
    while i < n {
        sp18[i as usize].selkind = i as u8;
        sp18[i as usize].stat_value = f(i as u32);
        i += 1;
    }
    i = 0;
    while i < n {
        let mut best_idx = i;
        let mut j = i + 1;
        while j < n {
            if sp18[best_idx as usize].stat_value < sp18[j as usize].stat_value {
                best_idx = j;
            }
            j += 1;
        }
        if best_idx != i {
            let temp = sp18[best_idx as usize];
            j = best_idx;
            while j > i {
                sp18[j as usize] = sp18[(j - 1) as usize];
                j -= 1;
            }
            sp18[i as usize] = temp;
        }
        i += 1;
    }
    i = 0;
    while i < n {
        if ext::gm_IsCKindUnlocked(ext::gm_SelKindToCKind(sp18[i as usize].selkind)) == 0 {
            i += 1;
            continue;
        }
        if skip_count != 0 {
            skip_count -= 1;
            let mut j = i + 1;
            while j < n {
                if ext::gm_IsCKindUnlocked(ext::gm_SelKindToCKind(sp18[j as usize].selkind)) != 0
                    && f(sp18[i as usize].selkind as u32) == f(sp18[j as usize].selkind as u32)
                {
                    i += 1;
                    if skip_count != 0 {
                        skip_count -= 1;
                    } else {
                        return n;
                    }
                }
                j += 1;
            }
        } else {
            let mut j = i + 1;
            while j < n {
                if ext::gm_IsCKindUnlocked(ext::gm_SelKindToCKind(sp18[j as usize].selkind)) == 0 {
                    j += 1;
                    continue;
                }
                if f(sp18[i as usize].selkind as u32) == f(sp18[j as usize].selkind as u32) {
                    return n;
                }
                j += 1;
            }
            return sp18[i as usize].selkind as i32;
        }
        i += 1;
    }
    n
}

pub unsafe extern "C" fn mnCount_GetSmashChamp(selkind: SelectableCharacterKind) -> u32 {
    melee_core::ct!("mnCount_GetSmashChamp");
    (*ext::GetPersistentFighterData(selkind)).stats.victories as u32
}

pub unsafe extern "C" fn mnCount_GetSmashSap(selkind: SelectableCharacterKind) -> u32 {
    melee_core::ct!("mnCount_GetSmashSap");
    (*ext::GetPersistentFighterData(selkind)).stats.losses as u32
}

pub unsafe extern "C" fn mnCount_GetSlugMeister(selkind: SelectableCharacterKind) -> u32 {
    melee_core::ct!("mnCount_GetSlugMeister");
    (*ext::GetPersistentFighterData(selkind)).stats.damage_dealt as u32
}

pub unsafe extern "C" fn mnCount_GetPunchingBag(selkind: SelectableCharacterKind) -> u32 {
    melee_core::ct!("mnCount_GetPunchingBag");
    (*ext::GetPersistentFighterData(selkind)).stats.damage_taken as u32
}

/// 25072C: break a tie between `entries[start_idx..]` by KOs, then falls.
pub unsafe extern "C" fn mnCount_8025072C(entries: *mut CountEntry, start_idx: i32, mode: i32) -> i32 {
    melee_core::ct!("mnCount_8025072C");
    let e = |i: i32| -> CountEntry { *entries.offset(i as isize) };
    let kos = |i: i32| -> u32 { mnCount_GetKOKingpin(e(i).selkind as u32) };
    let falls = |i: i32| -> u32 { mnCount_GetNoDefenseNelly(e(i).selkind as u32) };
    let mut tie = false;
    let mut best_idx = start_idx;
    let mut idx = start_idx + 1;
    while idx < SELKIND_COUNT as i32 {
        if e(start_idx).stat_value == e(idx).stat_value {
            if mode != 0 {
                if kos(best_idx) > kos(idx) {
                    best_idx = idx;
                }
            } else if kos(best_idx) < kos(idx) {
                best_idx = idx;
            }
            if kos(best_idx) == kos(idx) {
                tie = true;
                break;
            }
        }
        idx += 1;
    }
    if !tie {
        return e(best_idx).selkind as i32;
    }
    tie = false;
    idx = start_idx + 1;
    while idx < SELKIND_COUNT as i32 {
        if idx == best_idx {
            idx += 1;
            continue;
        }
        if e(start_idx).stat_value == e(idx).stat_value && kos(start_idx) == kos(idx) {
            if falls(best_idx) == falls(idx) {
                tie = true;
                break;
            }
            if mode != 0 {
                if falls(best_idx) < falls(idx) {
                    best_idx = idx;
                }
            } else if falls(best_idx) > falls(idx) {
                best_idx = idx;
            }
        }
        idx += 1;
    }
    if tie {
        SELKIND_COUNT as i32
    } else {
        e(best_idx).selkind as i32
    }
}

pub unsafe fn mnCount_8025092C_inline(_i: i32) -> bool {
    melee_core::ct!("mnCount_8025092C_inline");
    for i in 0..SELKIND_COUNT {
        if mnCount_GetMatchTime(i) != 0 {
            return false;
        }
    }
    true
}

/// 25092C
pub unsafe extern "C" fn mnCount_8025092C(mut rank: i32, get_val: GetVal, mode: i32) -> i32 {
    melee_core::ct!("mnCount_8025092C");
    let f = get_val.unwrap();
    let n = SELKIND_COUNT as i32;
    let mut entries = [CountEntry { selkind: 0, pad: [0; 3], stat_value: 0 }; SELKIND_COUNT as usize];
    if mnCount_8025092C_inline(0) {
        return n;
    }
    let mut i: i32 = 0;
    while i < n {
        entries[i as usize].selkind = i as u8;
        entries[i as usize].stat_value = f(i as u32);
        i += 1;
    }
    i = 0;
    while i < n {
        let mut min = i;
        let mut j = i + 1;
        while j < n {
            if entries[min as usize].stat_value < entries[j as usize].stat_value {
                min = j;
            }
            j += 1;
        }
        if min != i {
            let tmp = entries[min as usize];
            j = min;
            while j > i {
                entries[j as usize] = entries[(j - 1) as usize];
                j -= 1;
            }
            entries[i as usize] = tmp;
        }
        i += 1;
    }
    i = 0;
    while i < n {
        if ext::gm_IsCKindUnlocked(ext::gm_SelKindToCKind(entries[i as usize].selkind)) == 0 {
            i += 1;
            continue;
        }
        if rank != 0 {
            rank -= 1;
            let mut j = i + 1;
            while j < n {
                if ext::gm_IsCKindUnlocked(ext::gm_SelKindToCKind(entries[j as usize].selkind)) == 0 {
                    j += 1;
                    continue;
                }
                if f(entries[i as usize].selkind as u32) == f(entries[j as usize].selkind as u32) {
                    i += 1;
                    if rank != 0 {
                        rank -= 1;
                    } else {
                        return n;
                    }
                }
                j += 1;
            }
        } else {
            let mut j = i + 1;
            while j < n {
                if ext::gm_IsCKindUnlocked(ext::gm_SelKindToCKind(entries[j as usize].selkind)) == 0 {
                    j += 1;
                    continue;
                }
                if f(entries[i as usize].selkind as u32) == f(entries[j as usize].selkind as u32) {
                    return mnCount_8025072C(entries.as_mut_ptr(), i, mode);
                }
                j += 1;
            }
            return entries[i as usize].selkind as i32;
        }
        i += 1;
    }
    n
}

pub unsafe fn mnCount_CountUnlockedChars() -> i32 {
    melee_core::ct!("mnCount_CountUnlockedChars");
    let mut c = 0;
    for i in 0..SELKIND_COUNT {
        if ext::gm_IsCKindUnlocked(ext::gm_SelKindToCKind(i as u8)) != 0 {
            c += 1;
        }
    }
    c
}

pub unsafe fn mnCount_CountUnlockedMaps() -> i32 {
    melee_core::ct!("mnCount_CountUnlockedMaps");
    let mut c = 0;
    for i in 0..NUM_STAGES {
        if ext::gm_80164430(ext::gm_801641CC(i as u8)) != 0 {
            c += 1;
        }
    }
    c
}

/// 250C7C
pub unsafe extern "C" fn mnCount_GetRowValue_Character(row: u32) -> SelectableCharacterKind {
    melee_core::ct!("mnCount_GetRowValue_Character");
    match row {
        LONGEST_TIME => mnCount_8025035C(0, Some(mnCount_GetMatchTime)) as u32,
        SECOND_LONGEST_TIME => mnCount_8025035C(1, Some(mnCount_GetMatchTime)) as u32,
        SHORTEST_TIME => {
            let c = mnCount_CountUnlockedChars();
            if c < 3 {
                return SELKIND_COUNT;
            }
            mnCount_8025035C(c - 1, Some(mnCount_GetMatchTime)) as u32
        }
        SMASH_CHAMP => mnCount_8025092C(0, Some(mnCount_GetSmashChamp), 0) as u32,
        SMASH_SAP => mnCount_8025092C(0, Some(mnCount_GetSmashSap), 1) as u32,
        SLUG_MEISTER => mnCount_8025092C(0, Some(mnCount_GetSlugMeister), 0) as u32,
        PUNCHING_BAG => mnCount_8025092C(0, Some(mnCount_GetPunchingBag), 1) as u32,
        KO_KINGPIN => mnCount_8025035C(0, Some(mnCount_GetKOKingpin)) as u32,
        NO_DEFENSE_NELLY => mnCount_8025035C(0, Some(mnCount_GetNoDefenseNelly)) as u32,
        DISASTER_MASTER => mnCount_8025035C(0, Some(mnCount_GetDisasterMaster)) as u32,
        _ => SELKIND_COUNT,
    }
}

pub unsafe fn mnCount_GetCombinedVSPlayTimeValue() -> u32 {
    melee_core::ct!("mnCount_GetCombinedVSPlayTimeValue");
    *(ext::gmMainLib_GetCombinedVSPlayTime() as *mut u32)
}

/// 250DE4
pub unsafe extern "C" fn mnCount_GetRowValue_Number(row: i32) -> u32 {
    melee_core::ct!("mnCount_GetRowValue_Number");
    let mut ret: u32;
    match row as u32 {
        POWER_COUNT => ret = *(ext::gmMainLib_GetPowerCount() as *mut u32),
        POWER_TIME => ret = *ext::gm_GetPowerTime() as u32,
        PLAY_TIME => ret = ext::gm_GetPlayTime() as u32,
        SINGLEPLAYER_TIME => ret = *(ext::gmMainLib_GetSingleplayerTime() as *mut u32),
        VS_PLAY_TIME => ret = *(ext::gmMainLib_GetVsPlayTime() as *mut u32),
        COMBINED_VS_PLAY_TIME => ret = mnCount_GetCombinedVSPlayTimeValue(),
        VS_PLAY_MATCH_TOTAL => ret = ext::gm_GetVsPlayMatchTotal(),
        TIME_MATCH_TOTAL => ret = *ext::gmMainLib_GetTimeMatchTotal(),
        STOCK_MATCH_TOTAL => ret = *ext::gmMainLib_GetStockMatchTotal(),
        COIN_MATCH_TOTAL => ret = *ext::gmMainLib_GetCoinMatchTotal(),
        BONUS_MATCH_TOTAL => ret = *ext::gmMainLib_GetBonusMatchTotal(),
        VS_PLAY_CONTESTANTS => ret = *(ext::gmMainLib_GetVsPlayContestants() as *mut u32),
        MATCH_RESET_COUNTER => ret = *ext::gmMainLib_GetMatchResetCounter(),
        TOTAL_DAMAGE => ret = *(ext::gmMainLib_GetTotalDamage() as *mut u32),
        KO_TOTAL => ret = *ext::gmMainLib_GetKOTotal() as u32,
        SELFDESTRUCT_TOTAL => ret = *(ext::gmMainLib_GetSelfDestructTotal() as *mut u32),
        AVAILABLE_CHARACTERS => ret = mnCount_CountUnlockedChars() as u32,
        AVAILABLE_MAPS => ret = mnCount_CountUnlockedMaps() as u32,
        TROPHY_TOTAL => ret = ext::Toy_GetTrophyTotal() as u32,
        NAME_TOTAL => ret = ext::GetNameCount() as u32,
        _ => {
            if inline_is_row_time(row) {
                ret = 359940;
            } else {
                ret = 9999;
            }
        }
    }
    if inline_is_row_time(row) {
        if ret > 3599999940u32 {
            ret = 3599999940u32;
        }
    } else if ret > 999999999 {
        ret = 999999999;
    }
    ret
}

/// 250FF8: (re)build one visible row's label and value texts.
pub unsafe extern "C" fn mnCount_CreateRow(gobj: *mut HSD_GObj, visible_row: i32, data_row: u32) {
    melee_core::ct!("mnCount_CreateRow");
    let userdata = get_mncount(gobj);
    let label_text = addr_of_mut!((*userdata).labels[visible_row as usize]);
    if !(*label_text).is_null() {
        ext::HSD_SisLib_803A5CC4((*userdata).labels[visible_row as usize]);
        *label_text = null_mut();
    }
    let y = melee_core::fp::fmadds(1.4f32, visible_row as f32, -6.4f32);
    let text = ext::HSD_SisLib_803A5ACC(0, 1, -13.0, y, 17.0, 500.0, 38.38772);
    *label_text = text;
    (*text).font_size.x = 0.03;
    (*text).font_size.y = 0.03;
    (*text).text_color = TEXT_COLOR;
    ext::HSD_SisLib_803A6368(text, mnCount_sis_idx[data_row as usize] as i32);
    let value_text = addr_of_mut!((*userdata).values[visible_row as usize]);
    if !(*value_text).is_null() {
        ext::HSD_SisLib_803A5CC4((*userdata).values[visible_row as usize]);
        *value_text = null_mut();
    }
    let text = ext::HSD_SisLib_803A6754(0, 1);
    *value_text = text;
    (*text).pos_x = 13.0;
    (*text).pos_y = y;
    (*text).pos_z = 17.0;
    (*text).text_color = mn_804D4B64;
    (*text).default_alignment = 2;
    if inline_is_row_time(data_row as i32) {
        // PPC keeps the two 4-byte buffers adjacent: `buf` and `buf - 4`
        let mut area = [0u8; 8];
        let buf = area.as_mut_ptr().add(4) as *mut i8;
        (*text).font_size.x = 0.03;
        (*text).font_size.y = 0.03;
        let row_value = mnCount_GetRowValue_Number(data_row as i32);
        ext::mn_8022EA78(buf, 2, row_value / 60 / 60);
        ext::mn_8022EA78(buf.sub(4), 2, row_value / 60 % 60);
        ext::HSD_SisLib_803A6B98(text, 0.0, 0.0, c"%u:%s".as_ptr(), &[Va::I32((row_value / 60 / 60) as i32), Va::P(buf.sub(4).cast())]);
    } else if inline_is_row_char(data_row as i32) {
        (*text).font_size.x = 0.03;
        (*text).font_size.y = 0.03;
        let selkind = mnCount_GetRowValue_Character(data_row);
        if selkind == SELKIND_COUNT {
            ext::HSD_SisLib_803A6B98(text, 0.0, 0.0, c"\x81| ".as_ptr(), &[]);
        } else {
            ext::gm_80160B40(text, ext::gm_SelKindToCKind(selkind as u8), 0);
        }
    } else {
        (*text).font_size.x = 0.03;
        (*text).font_size.y = 0.03;
        ext::HSD_SisLib_803A6B98(text, 0.0, 0.0, c"%u".as_ptr(), &[Va::I32(mnCount_GetRowValue_Number(data_row as i32) as i32)]);
    }
}

/// 251278: the input proc.
pub unsafe extern "C" fn mnCount_HandleUserInput(_gobj: *mut HSD_GObj) {
    melee_core::ct!("mnCount_HandleUserInput");
    let userdata = get_mncount(menu_gobj);
    if mn_804D6BC8.cooldown != 0 {
        mn_804D6BC8.cooldown = mn_804D6BC8.cooldown.wrapping_sub(1);
        mn_804D6BC8.x2 = 0;
        mn_804D6BC8.x4 = 0;
    } else {
        mn_804A04F0.buttons = ext::mn_80229624(4) as u64;
        let x = mn_804A04F0.buttons as i64;
        if x & MENU_INPUT_BACK != 0 {
            sfxBack();
            mn_804A04F0.entering_menu = 0;
            ext::mn_80229894(MENU_KIND_RECORDS as i32, 2, 3);
        } else if x & MENU_INPUT_UP != 0 {
            if (*userdata).scroll_pos != 0 {
                (*userdata).scroll_pos -= 1;
                sfxMove();
                inline_update_entries(menu_gobj);
            }
        } else if x & MENU_INPUT_DOWN != 0 && (*userdata).scroll_pos < MAX_SCROLL {
            sfxMove();
            (*userdata).scroll_pos += 1;
            inline_update_entries(menu_gobj);
        }
    }
}

unsafe fn update_arrows(userdata: *mut MnCountData, jobj: *mut HSD_JObj) {
    let mut child: *mut HSD_JObj = null_mut();
    // up arrow
    ext::lb_80011E24(jobj, &mut child, &[Va::I32(2), Va::I32(-1)]);
    if (*userdata).scroll_pos != 0 {
        ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32);
    }
    // down arrow
    ext::lb_80011E24(jobj, &mut child, &[Va::I32(1), Va::I32(-1)]);
    if (*userdata).scroll_pos < MAX_SCROLL {
        ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32);
    }
    ext::mn_8022ED6C(jobj, addr_of!(mnCount_803EFAA0) as *mut AnimLoopSettings);
}

/// 2513F4
pub unsafe extern "C" fn mnCount_UpdateArrowIndicators(gobj: *mut HSD_GObj) {
    melee_core::ct!("mnCount_UpdateArrowIndicators");
    let userdata = get_mncount(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    update_arrows(userdata, jobj);
}

pub unsafe fn mnCount_UpdateArrowIndicators_noinline(gobj: *mut HSD_GObj) {
    melee_core::ct!("mnCount_UpdateArrowIndicators_noinline");
    mnCount_UpdateArrowIndicators(gobj);
}

/// 2514B8
pub unsafe extern "C" fn fn_802514B8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_802514B8");
    ext::HSD_GObjFree(gobj);
}

pub unsafe fn fn_802514D8_inline(userdata: *mut MnCountData, gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_802514D8_inline");
    if mn_804A04F0.cur_menu as u32 != MENU_KIND_RECORDS_MISC {
        ext::HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        let proc = ext::HSD_GObj_SetupProc(gobj, Some(fn_802514B8), 0);
        (*proc).set_flags_3(HSD_GObj_804D783C as u8);
        let userdata2 = get_mncount(gobj);
        let userdata3 = userdata2;
        for i in 0..MNCOUNT_VISIBLE_ROWS {
            if !(*userdata2).labels[i].is_null() {
                ext::HSD_SisLib_803A5CC4((*userdata3).labels[i]);
                (*userdata2).labels[i] = null_mut();
            }
            if !(*userdata2).values[i].is_null() {
                ext::HSD_SisLib_803A5CC4((*userdata3).values[i]);
                (*userdata2).values[i] = null_mut();
            }
        }
        ext::HSD_SisLib_803A5CC4((*userdata).title);
    } else {
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        update_arrows(userdata, jobj);
    }
}

/// 2514D8
pub unsafe extern "C" fn fn_802514D8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_802514D8");
    let userdata = get_mncount(gobj);
    fn_802514D8_inline(userdata, gobj);
}

pub unsafe fn fn_80251640_FreeText(gobj: *mut HSD_GObj, userdata: *mut MnCountData, mut i: i32) {
    melee_core::ct!("fn_80251640_FreeText");
    let userdata2 = get_mncount(gobj);
    let userdata3 = userdata2;
    loop {
        if !(*userdata2).labels[i as usize].is_null() {
            ext::HSD_SisLib_803A5CC4((*userdata3).labels[i as usize]);
            (*userdata2).labels[i as usize] = null_mut();
        }
        if !(*userdata2).values[i as usize].is_null() {
            ext::HSD_SisLib_803A5CC4((*userdata3).values[i as usize]);
            (*userdata2).values[i as usize] = null_mut();
        }
        i += 1;
        if i >= MNCOUNT_VISIBLE_ROWS as i32 {
            break;
        }
    }
    ext::HSD_SisLib_803A5CC4((*userdata).title);
}

pub unsafe fn fn_80251640_InitModel(gobj: *mut HSD_GObj, userdata: *mut MnCountData, mut i: i32) {
    melee_core::ct!("fn_80251640_InitModel");
    while i < MNCOUNT_VISIBLE_ROWS as i32 {
        mnCount_CreateRow(gobj, i, (*userdata).scroll_pos as u32 + i as u32);
        i += 1;
    }
    let md = addr_of_mut!(model_desc);
    let jobj = ext::HSD_JObjLoadJoint((*md).joint);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 4, 0x80);
    ext::HSD_JObjAddAnimAll(jobj, (*md).animjoint, (*md).matanim_joint, (*md).shapeanim_joint);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
}

/// 251640
pub unsafe extern "C" fn fn_80251640(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80251640");
    let userdata = get_mncount(gobj);
    if mn_804A04F0.cur_menu as u32 != MENU_KIND_RECORDS_MISC {
        ext::HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        let proc = ext::HSD_GObj_SetupProc(gobj, Some(fn_802514B8), 0);
        (*proc).set_flags_3(HSD_GObj_804D783C as u8);
        fn_80251640_FreeText(gobj, userdata, 0);
        return;
    }
    if (*userdata).expand_anim_duration != 0 {
        (*userdata).expand_anim_duration -= 1;
        return;
    }
    fn_80251640_InitModel(gobj, userdata, 0);
    mnCount_UpdateArrowIndicators_noinline(gobj);
    ext::HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
    let proc = ext::HSD_GObj_SetupProc(gobj, Some(fn_802514D8), 0);
    (*proc).set_flags_3(HSD_GObj_804D783C as u8);
}

/// 2517E0
pub unsafe extern "C" fn mnCount_InitUserData(userdata: *mut MnCountData) {
    melee_core::ct!("mnCount_InitUserData");
    (*userdata).scroll_pos = 0;
    (*userdata).expand_anim_duration = 10;
    for i in 0..MNCOUNT_VISIBLE_ROWS {
        (*userdata).labels[i] = null_mut();
        (*userdata).values[i] = null_mut();
    }
}

pub unsafe fn mnCount_InitUserData_noinline(userdata: *mut MnCountData) {
    melee_core::ct!("mnCount_InitUserData_noinline");
    mnCount_InitUserData(userdata);
}

pub unsafe fn mnCount_AllocUserData() -> *mut MnCountData {
    melee_core::ct!("mnCount_AllocUserData");
    let user_data = ext::HSD_MemAlloc(core::mem::size_of::<MnCountData>() as i32) as *mut MnCountData;
    if user_data.is_null() {
        ext::OSReport(c"Can't get user_data.\n".as_ptr(), &[]);
        ext::__assert(c"mncount.c".as_ptr(), 1298, c"user_data".as_ptr());
    }
    user_data
}

/// 25186C
pub unsafe extern "C" fn mnCount_Create() {
    melee_core::ct!("mnCount_Create");
    mn_804D6BC8.cooldown = 5;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = MENU_KIND_RECORDS_MISC as u8;
    mn_804A04F0.hovered_selection = 0;

    let archive = mn_804D6BB8;
    ext::lbArchive_LoadSections(
        archive,
        addr_of_mut!(model_desc.joint).cast(),
        &[
            c"MenMainConCo_Top_joint".as_ptr().cast(),
            addr_of_mut!(model_desc.animjoint).cast(),
            c"MenMainConCo_Top_animjoint".as_ptr().cast(),
            addr_of_mut!(model_desc.matanim_joint).cast(),
            c"MenMainConCo_Top_matanim_joint".as_ptr().cast(),
            addr_of_mut!(model_desc.shapeanim_joint).cast(),
            c"MenMainConCo_Top_shapeanim_joint".as_ptr().cast(),
            null_mut::<c_void>(),
        ],
    );

    let mut gobj = ext::GObj_Create(6, 7, 0x80);
    menu_gobj = gobj;

    let userdata = mnCount_AllocUserData();
    mnCount_InitUserData_noinline(userdata);
    ext::GObj_InitUserData(gobj, 0, Some(ext::HSD_Free), userdata.cast());

    let proc = ext::HSD_GObj_SetupProc(gobj, Some(fn_80251640), 0);
    (*proc).set_flags_3(HSD_GObj_804D783C as u8);

    {
        let menu_data = (*gobj).user_data as *mut MnCountData;
        if !(*menu_data).title.is_null() {
            ext::HSD_SisLib_803A5CC4((*menu_data).title);
        }
        let text = ext::HSD_SisLib_803A5ACC(0, 1, -9.5, 9.1, 17.0, 364.68332, 38.38772);
        (*menu_data).title = text;
        (*text).font_size.x = 0.0521;
        (*text).font_size.y = 0.0521;
        ext::HSD_SisLib_803A6368(text, 166);
    }

    gobj = ext::GObj_Create(0, 1, 0x80);
    let proc = ext::HSD_GObj_SetupProc(gobj, Some(mnCount_HandleUserInput), 0);
    (*proc).set_flags_3(HSD_GObj_804D783C as u8);
}

// ---- folded from mn.rs (consolidation steps 21/22) ----
/// mncount.c `inline_is_row_time_fp` (POWER_TIME..COMBINED_VS_PLAY_TIME = 1..5).
pub fn inline_is_row_time_fp(row: i32) -> bool {
    melee_core::ct!("inline_is_row_time_fp");
    (1..=5).contains(&row)
}

/// mncount.c `inline_is_row_char_fp` (LONGEST_TIME..DISASTER_MASTER = 20..29).
pub fn inline_is_row_char_fp(row: i32) -> bool {
    melee_core::ct!("inline_is_row_char_fp");
    (20..=29).contains(&row)
}
