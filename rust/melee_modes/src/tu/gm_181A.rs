//! src/melee/gm/gm_181A.c: the Multi-Man Smash modes (10-man ... Cruel): the spawn-table driver,
//! the per-character record store and the "title demo" camera script.
//!
//! The file's statics are `pub static mut` so the state oracle (tests/tu_gm_181A.rs against
//! gen/turef/tu_gm_181A_ref.c, the C TU compiled natively with shipping contraction) can seed and
//! compare them. This follows the `TARGET_PC` branches of the C: the record state is its own
//! object `lbl_80473594` (on the console it overlays `lbl_80472ED8 + 0x6BC`, and
//! `gm_80182554` writes it through that overlay), and every record access maps the
//! character through `gm_MexVanillaKind`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use crate::tu_util::*;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

/// Callees outside this TU (hooks; unset they panic `TODO: <name> is not ported`).
pub mod ext {
    pub use crate::tu_ext::{
        fn_8016EF98, gmMainLib_8015D6BC, gmMainLib_8015D710, gmMainLib_8015DB00,
        gmMainLib_8015DB18, gm_8016AEEC, gm_8016AEFC, gm_8016B328, gm_8016B33C, gm_8016B350,
        gm_8016B364, gm_8016B378, gm_8016EDDC, gm_80167858, gm_80168F88, gm_801A4B60, gm_801A4BA8,
        gm_801BF6D8, gm_801BF6F8, gm_CKindToSelKind, gm_GetButtonsTriggered,
        gm_GetCurrentGameMode, gm_GetCurrentSceneIndex, gm_GetFrameCount, gm_MexVanillaKind,
        gm_SetNewGameModePending, gm_SetNextGameModeStateId, gm_SetPendingGameMode,
        gm_SetupPlayerDefaults, ifStock_802FA2D0, lbArchive_80016DBC, lbAudioAx_80023694,
        lbAudioAx_80024030, lbAudioAx_80024C84, un_802FD28C, Camera_8002E6FC, Camera_8002EB5C,
        Camera_8002EC7C, Camera_8002ED9C, Camera_8002EEC8, Camera_SetModeToStandard, GObj_Create,
        HSD_GObj_SetupProc, HSD_Randi, Mex_Enabled, Player_800368F8, Player_GetFalls,
        Player_GetNametagSlotID, Player_GetPlayerId, Player_GetPlayerSlotType, Player_GetTeam,
        Player_SetFalls, Player_SetMoreFlagsBit4, Player_SetNametagSlotID,
        Player_SetPlayerAndEntityCpuLevel, Player_SetSelfDestructs,
    };
}

/// `RegClearSpawnEntry` (16 bytes): one row of a Multi-Man spawn table.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RegClearSpawnEntry {
    pub x0: i32,
    pub x4: u8,
    pub x5: u8,
    pub x6: u8,
    pub x7: u8,
    pub x8: f32,
    pub xC: f32,
}
melee_core::gc_layout!(RegClearSpawnEntry, "RegClearSpawnEntry", size 0x10, align 4, [
    x0 @ 0x0: 4 "s32",
    x4 @ 0x4: 1 "u8",
    x5 @ 0x5: 1 "u8",
    x6 @ 0x6: 1 "u8",
    x7 @ 0x7: 1 "u8",
    x8 @ 0x8: 4 "f32",
    xC @ 0xC: 4 "f32",
]);

/// `RegClearRecordState` (0x14): the running result of the current run.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RegClearRecordState {
    pub x0: u8,
    pub pad_1: u8,
    pub x2: u16,
    pub x4: i32,
    pub x8: i32,
    pub xC: i32,
    pub x10: i8,
    pub x11: u8,
    pub pad_12: [u8; 2],
}
melee_core::gc_layout!(RegClearRecordState, "RegClearRecordState", size 0x14, align 4, [
    x0 @ 0x0: 1 "u8",
    pad_1 @ 0x1: 1 "u8",
    x2 @ 0x2: 2 "u16",
    x4 @ 0x4: 4 "int",
    x8 @ 0x8: 4 "int",
    xC @ 0xC: 4 "int",
    x10 @ 0x10: 1 "s8",
    x11 @ 0x11: 1 "u8",
    pad_12 @ 0x12: 2 "u8[2]",
]);

/// `lbl_80472ED8_t` (0x6BC on the GC).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_80472ED8_t {
    pub x0: i32,
    pub x4: i32,
    pub x8: i32,
    pub xC: PlayerInitData,
    pub pad_30: [u8; 0x24],
    pub x54: [RegClearSpawnEntry; 101],
    pub x6A4: *mut RegClearSpawnEntry,
    pub x6A8: *mut RegClearSpawnEntry,
    pub x6AC: *mut RegClearSpawnEntry,
    pub x6B0: *mut RegClearSpawnEntry,
    pub x6B4: *mut RegClearSpawnEntry,
    pub x6B8: *mut RegClearSpawnEntry,
}
melee_core::gc_layout!(lbl_80472ED8_t, "lbl_80472ED8_t", size 0x6BC, align 4, [
    x0 @ 0x0: 4 "s32",
    x4 @ 0x4: 4 "s32",
    x8 @ 0x8: 4 "s32",
    xC @ 0xC: 36 "PlayerInitData",
    pad_30 @ 0x30: 36 "u8[36]",
    x54 @ 0x54: 1616 "RegClearSpawnEntry[101]",
    x6A4 @ 0x6A4: 4 "RegClearSpawnEntry*",
    x6A8 @ 0x6A8: 4 "RegClearSpawnEntry*",
    x6AC @ 0x6AC: 4 "RegClearSpawnEntry*",
    x6B0 @ 0x6B0: 4 "RegClearSpawnEntry*",
    x6B4 @ 0x6B4: 4 "RegClearSpawnEntry*",
    x6B8 @ 0x6B8: 4 "RegClearSpawnEntry*",
]);

/// `RecordBlock` (0xC0): per-mode best scores / icons / times, indexed by character.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RecordBlock {
    pub scores: [u32; 27],
    pub icons: [u8; 28],
    pub times: [u16; 28],
}
melee_core::gc_layout!(RecordBlock, "RecordBlock", size 0xC0, align 4, [
    scores @ 0x0: 108 "u32[27]",
    icons @ 0x6C: 28 "u8[28]",
    times @ 0x88: 56 "u16[28]",
]);

/// The C table initialises 26 of the 27 scores (the last one is zero).
const fn rb() -> RecordBlock {
    let mut scores = [0x0FFF_FFFF; 27];
    scores[26] = 0;
    RecordBlock { scores, icons: [0; 28], times: [0; 28] }
}
pub static mut lbl_803D8D08: [RecordBlock; 6] = [rb(), rb(), rb(), rb(), rb(), rb()];
pub static mut lbl_80472ED8: lbl_80472ED8_t = unsafe { core::mem::zeroed() };
pub static mut lbl_80473594: RegClearRecordState = unsafe { core::mem::zeroed() };
pub static mut lbl_804D65E0: UnkMultimanData = unsafe { core::mem::zeroed() };
pub static mut lbl_804D65E8: u8 = 0;

#[inline]
fn sfx_forward() {
    ext::lbAudioAx_80024030(1);
}

pub unsafe fn gm_80181A14() -> i32 {
    lbl_80473594.x0 as i32
}

pub unsafe fn gm_80181A24() -> u32 {
    lbl_80473594.x2 as u32
}

pub unsafe fn gm_80181A34() -> i32 {
    lbl_80473594.x4
}

/// Mode `0x21..=0x26` -> block `0..=5`.
#[inline]
fn block_of(mode: i32) -> Option<usize> {
    if (0x21..=0x26).contains(&mode) {
        Some((mode - 0x21) as usize)
    } else {
        None
    }
}

pub unsafe fn gm_80181A44(c_kind: i32, arg1: i32, arg2: bool) {
    let c_kind = ext::gm_MexVanillaKind(c_kind, 1);
    if let Some(b) = block_of(arg1) {
        *lbl_803D8D08[b].icons.as_mut_ptr().offset(c_kind as isize) = arg2 as u8;
    }
}

pub unsafe fn gm_80181AC8(c_kind: i32, arg1: i32, arg2: u16) {
    let c_kind = ext::gm_MexVanillaKind(c_kind, 1);
    if let Some(b) = block_of(arg1) {
        *lbl_803D8D08[b].times.as_mut_ptr().offset(c_kind as isize) = arg2;
    }
}

pub unsafe fn gm_80181B64(c_kind: i32, arg1: i32, arg2: i32) {
    let c_kind = ext::gm_MexVanillaKind(c_kind, 1);
    if let Some(b) = block_of(arg1) {
        *lbl_803D8D08[b].scores.as_mut_ptr().offset(c_kind as isize) = arg2 as u32;
    }
}

pub unsafe fn fn_80181BFC(arg0: *mut i32) -> i32 {
    let mut count = 0;
    for i in 1..6i32 {
        if ext::Player_GetFalls(i) == 0 && ext::Player_GetPlayerSlotType(i) != Gm_PKind_NA as i32 {
            count += 1;
        } else if !arg0.is_null() {
            *arg0 = i;
        }
    }
    count
}

/// `fn_80181C80_CountPlayers`
unsafe fn count_players(out: &mut i32) -> i32 {
    let mut count = 0;
    for i in 1..6i32 {
        if ext::Player_GetFalls(i) == 0 && ext::Player_GetPlayerSlotType(i) != Gm_PKind_NA as i32 {
            count += 1;
        } else {
            *out = i;
        }
    }
    count
}

pub unsafe fn fn_80181C80(arg0: i32) {
    let data = &mut lbl_80472ED8;
    let mut sp38: i32 = 0;

    ext::gm_GetCurrentGameMode();
    let mut sp10 = data.xC;
    let count = count_players(&mut sp38);

    let e = data.x54[arg0 as usize];
    if e.x4 as i32 > count && data.x8 > 0x5A {
        if ext::Player_GetPlayerSlotType(sp38) != Gm_PKind_NA as i32 {
            ext::Player_SetFalls(sp38, 0);
            ext::Player_SetSelfDestructs(sp38, 0);
            ext::fn_8016EF98(sp38);
        }
        data.x54[arg0 as usize].x0 = -2;
        sp10.team = (ext::Player_GetTeam(0) == 0) as u8;
        sp10.ckind = e.x5 as i8;
        sp10.cpu_level = e.x6;
        sp10.cpu_kind = e.x7;
        sp10.attack_ratio = e.x8;
        sp10.defense_ratio = e.xC;
        ext::gm_8016EDDC(sp38, &mut sp10);
        ext::Player_SetNametagSlotID(sp38, 0x78);
        ext::un_802FD28C(sp38);
        data.x0 += 1;
    }
}

/// `max(entry_idx - fn_80181BFC(NULL), 0)`
unsafe fn live_offset(entry_idx: i32) -> i32 {
    let t = entry_idx - fn_80181BFC(null_mut());
    if t < 0 { 0 } else { t }
}

pub unsafe extern "C" fn fn_80181E18(_gobj: *mut HSD_GObj) {
    fn_80181E18_body()
}

pub unsafe fn fn_80181E18_body() {
    let data = &mut lbl_80472ED8;
    let mode = ext::gm_GetCurrentGameMode() as i32;

    if data.x8 <= 0x5A {
        data.x8 += 1;
    }

    match mode {
        0x21 | 0x22 => {
            ext::Player_GetFalls(0);
        }
        0x23 | 0x24 => {
            if ext::gm_8016AEEC() == 0 && ext::gm_8016AEFC() == 0x3B {
                lbl_80473594.x0 = 1;
                ext::gm_8016B33C(7);
                ext::gm_8016B328();
            }
        }
        0x25 | 0x26 => {
            if ext::Player_GetFalls(0) != 0 {
                ext::gm_8016B33C(5);
                ext::gm_8016B328();
            }
        }
        _ => {}
    }

    let mut entry_idx = 0i32;
    while entry_idx < 101 {
        if data.x54[entry_idx as usize].x0 == -2 {
            entry_idx += 1;
            continue;
        }

        match lbl_80473594.x8 {
            0x21 | 0x22 => lbl_80473594.x4 = ext::gm_GetFrameCount() as i32,
            _ => {}
        }

        let mut temp = live_offset(entry_idx);
        let next = data.x4;
        lbl_80473594.x2 = (temp + next) as i16 as u16;

        match mode {
            0x21 => {
                temp = live_offset(entry_idx);
                ext::ifStock_802FA2D0(0xA - (temp + data.x4));
            }
            0x22 => {
                temp = live_offset(entry_idx);
                ext::ifStock_802FA2D0(0x64 - (temp + data.x4));
            }
            _ => {
                temp = live_offset(entry_idx);
                ext::ifStock_802FA2D0(temp + data.x4);
            }
        }

        let next = data.x54[entry_idx as usize].x0;

        if next == -1 {
            fn_80181C80(entry_idx);
            return;
        }

        if next == 0x3E7 {
            let mut count = 0;
            for i in 1..6i32 {
                if ext::Player_GetFalls(i) == 0 && ext::Player_GetPlayerSlotType(i) != Gm_PKind_NA as i32 {
                    count += 1;
                }
            }
            if count == 0 {
                lbl_80473594.x0 = 1;
                ext::gm_8016B33C(7);
                ext::gm_8016B328();
            }
        } else {
            let mut k = next;
            while k < entry_idx {
                data.x54[k as usize].x0 = -1;
                data.x4 += 1;
                k += 1;
            }
            fn_80181C80(data.x54[entry_idx as usize].x0);
            temp = live_offset(entry_idx);
            ext::ifStock_802FA2D0(temp + data.x4);
        }
        break;
    }
}

/// Copy a spawn table into `x54` up to and including the `0x3E7` terminator (or 101 rows).
unsafe fn copy_spawn_table(mut src: *const RegClearSpawnEntry) {
    let mut dst = lbl_80472ED8.x54.as_mut_ptr();
    for _ in 0..101 {
        (*dst).x0 = (*src).x0;
        (*dst).x4 = (*src).x4;
        (*dst).x5 = (*src).x5;
        (*dst).x8 = (*src).x8;
        (*dst).xC = (*src).xC;
        (*dst).x6 = (*src).x6;
        (*dst).x7 = (*src).x7;
        if (*src).x0 == 0x3E7 {
            break;
        }
        src = src.add(1);
        dst = dst.add(1);
    }
}

pub unsafe fn gm_80182174() {
    let mode = ext::gm_GetCurrentGameMode() as i32;

    ext::lbArchive_80016DBC(
        c"GmKumite.dat".as_ptr(),
        &[
            &raw mut lbl_80472ED8.x6A4 as *mut c_void,
            c"gmKumiteSystemTable10man".as_ptr() as *mut c_void,
            &raw mut lbl_80472ED8.x6A8 as *mut c_void,
            c"gmKumiteSystemTable100man".as_ptr() as *mut c_void,
            &raw mut lbl_80472ED8.x6AC as *mut c_void,
            c"gmKumiteSystemTable10min".as_ptr() as *mut c_void,
            &raw mut lbl_80472ED8.x6B0 as *mut c_void,
            c"gmKumiteSystemTable60min".as_ptr() as *mut c_void,
            &raw mut lbl_80472ED8.x6B4 as *mut c_void,
            c"gmKumiteSystemTableEndless".as_ptr() as *mut c_void,
            &raw mut lbl_80472ED8.x6B8 as *mut c_void,
            c"gmKumiteSystemTableMercilessly".as_ptr() as *mut c_void,
        ],
    );

    lbl_80472ED8.x0 = 0;
    lbl_80472ED8.x4 = 0;
    lbl_80472ED8.x8 = 0;

    ext::gm_SetupPlayerDefaults(&mut lbl_80472ED8.xC);

    lbl_80472ED8.xC.ckind = CKind_Boy as i8;
    lbl_80472ED8.xC.slot_type = 1;
    lbl_80472ED8.xC.stocks = 1;
    lbl_80472ED8.xC.set_xD_b4(1);

    match mode {
        0x21 => copy_spawn_table(lbl_80472ED8.x6A4),
        0x22 => copy_spawn_table(lbl_80472ED8.x6A8),
        0x23 => copy_spawn_table(lbl_80472ED8.x6AC),
        0x24 => copy_spawn_table(lbl_80472ED8.x6B0),
        0x25 => copy_spawn_table(lbl_80472ED8.x6B4),
        0x26 => copy_spawn_table(lbl_80472ED8.x6B8),
        _ => {}
    }

    lbl_80473594.x10 = ext::Player_GetPlayerId(0) as i8;
    lbl_80473594.x11 = ext::Player_GetNametagSlotID(0) as u8;
    ext::HSD_GObj_SetupProc(ext::GObj_Create(0xF, 0x11, 0), Some(fn_80181E18), 0x15);
    ext::gm_80168F88();
}

pub fn gm_IsMultimanSmashMode() -> bool {
    match ext::gm_GetCurrentGameMode() as i32 {
        x if x == GM_10MAN_VS as i32
            || x == GM_100MAN_VS as i32
            || x == GM_3MIN_VS as i32
            || x == GM_15MIN_VS as i32
            || x == GM_ENDLESS_VS as i32
            || x == GM_CRUEL_VS as i32 =>
        {
            true
        }
        _ => false,
    }
}

/// The C writes these through `(regclear_record_state*) &lbl_80472ED8` (offsets 0x6BC..0x6C8);
/// on the console that is the storage of `lbl_80473594`.
pub unsafe fn gm_80182554(arg0: i32, arg1: i32) {
    let s = &mut lbl_80473594;
    s.xC = arg0;
    s.x8 = arg1;
    s.x0 = 0;
    s.x4 = 0;
    s.x2 = 0;
}

/// `gm_80182578_GetRecordTime`
unsafe fn get_record_time(idx: i32, mode: i32) -> i32 {
    let idx = ext::gm_MexVanillaKind(idx, 1);
    match block_of(mode) {
        Some(b) => *lbl_803D8D08[b].times.as_ptr().offset(idx as isize) as i32,
        None => 0,
    }
}

/// `gm_80182578_GetRecordScore`
unsafe fn get_record_score(idx: i32, mode: i32) -> u32 {
    let idx = ext::gm_MexVanillaKind(idx, 1);
    match block_of(mode) {
        Some(b) => *lbl_803D8D08[b].scores.as_ptr().offset(idx as isize),
        None => 0,
    }
}

/// `gm_80182578_SetTime`
unsafe fn set_record_time(idx: i32, mode: i32, value: u16) {
    let idx = ext::gm_MexVanillaKind(idx, 1);
    if let Some(b) = block_of(mode) {
        *lbl_803D8D08[b].times.as_mut_ptr().offset(idx as isize) = value;
    }
}

pub unsafe fn gm_80182578() {
    let rec = &mut lbl_80473594;
    let mut mode = rec.x8;
    let mut idx = rec.xC;
    idx = ext::gm_MexVanillaKind(idx, 1);
    let time_val = get_record_time(idx, mode);
    let score_val = get_record_score(idx, mode);

    match mode {
        0x21 | 0x22 => {
            if mode == 0x21 {
                mode = ext::gmMainLib_8015D6BC(ext::gm_CKindToSelKind(idx as u8));
            } else {
                mode = ext::gmMainLib_8015D710(ext::gm_CKindToSelKind(idx as u8));
            }
            if rec.x0 != 0 {
                let score_store = rec.x4 as u32;
                if score_store < score_val {
                    let i = idx;
                    let m = rec.x8;
                    if let Some(b) = block_of(m) {
                        *lbl_803D8D08[b].icons.as_mut_ptr().offset(i as isize) = rec.x0;
                    }
                    if let Some(b) = block_of(m) {
                        *lbl_803D8D08[b].scores.as_mut_ptr().offset(i as isize) = score_store;
                    }
                    set_record_time(i, m, rec.x2);
                }
            } else if rec.x2 as i32 > time_val && mode == 0 {
                let i = idx;
                let m = rec.x8;
                if let Some(b) = block_of(m) {
                    *lbl_803D8D08[b].times.as_mut_ptr().offset(i as isize) = rec.x2;
                }
            }
        }
        0x23 | 0x24 => {
            if rec.x0 != 0 {
                let time_store = rec.x2;
                if time_store as i32 > time_val {
                    if let Some(b) = block_of(mode) {
                        *lbl_803D8D08[b].icons.as_mut_ptr().offset(idx as isize) = rec.x0;
                    }
                    if let Some(b) = block_of(mode) {
                        *lbl_803D8D08[b].times.as_mut_ptr().offset(idx as isize) = time_store;
                    }
                }
            }
        }
        0x25 | 0x26 => {
            if rec.x2 as i32 > time_val {
                if let Some(b) = block_of(mode) {
                    *lbl_803D8D08[b].times.as_mut_ptr().offset(idx as isize) = rec.x2;
                }
            }
        }
        _ => {}
    }
}

pub unsafe fn fn_80182B5C() {
    let rec = &lbl_80473594;
    let mut idx = rec.xC;
    let mode = rec.x8;

    idx = ext::gm_MexVanillaKind(idx, 1);

    // fn_80182B5C_GetTime / fn_80182B5C_GetScore re-map the raw record index
    let time = {
        let i = ext::gm_MexVanillaKind(rec.xC, 1);
        match block_of(rec.x8) {
            Some(b) => *lbl_803D8D08[b].times.as_ptr().offset(i as isize) as i32,
            None => 0,
        }
    };
    let score = {
        let i = ext::gm_MexVanillaKind(rec.xC, 1);
        match block_of(rec.x8) {
            Some(b) => *lbl_803D8D08[b].scores.as_ptr().offset(i as isize),
            None => 0,
        }
    };

    match mode {
        0x21 | 0x22 => {
            if mode == 0x21 {
                ext::gmMainLib_8015D6BC(ext::gm_CKindToSelKind(idx as u8));
            } else {
                ext::gmMainLib_8015D710(ext::gm_CKindToSelKind(idx as u8));
            }
            if rec.x0 != 0 {
                if (rec.x4 as u32) < score {
                    ext::gm_8016B350(0x9C40);
                    ext::gm_8016B364(0x144);
                    ext::gm_80167858(rec.x10 as i32, rec.x11 as i32, 0xD, 0x5A);
                }
            } else {
                ext::gm_8016B364(0x148);
                ext::gm_8016B378(0x28);
            }
        }
        0x23 | 0x24 => {
            if rec.x0 != 0 && rec.x2 as i32 > time {
                ext::gm_8016B350(0x9C40);
                ext::gm_8016B364(0x144);
                ext::gm_80167858(rec.x10 as i32, rec.x11 as i32, 0xD, 0x5A);
            }
        }
        0x25 | 0x26 => {
            if rec.x2 as i32 > time {
                ext::gm_8016B350(0x9C40);
                ext::gm_8016B364(0x144);
                ext::gm_80167858(rec.x10 as i32, rec.x11 as i32, 0xD, 0x5A);
            }
        }
        _ => {}
    }
}

pub unsafe fn gm_80182DF0(c_kind: i32, arg1: i32) -> *mut UnkMultimanData {
    let result = &raw mut lbl_804D65E0;
    let c_kind = ext::gm_MexVanillaKind(c_kind, 1);
    let ck = c_kind as isize;

    if let Some(b) = block_of(arg1) {
        let blk = &lbl_803D8D08[b];
        (*result).set_x0_0(*blk.icons.as_ptr().offset(ck) as u16);
        (*result).x2 = *blk.times.as_ptr().offset(ck);
        if b < 2 {
            (*result).x4 = *blk.scores.as_ptr().offset(ck) as i32;
        }
    }
    result
}

pub unsafe extern "C" fn fn_80182F40(_unused: *mut HSD_GObj) {
    if ext::gm_GetButtonsTriggered(PAD_MAX_CONTROLLERS as u8) & (HSD_PAD_START as u64 | HSD_PAD_A as u64) != 0 {
        ext::lbAudioAx_80024C84();
        ext::lbAudioAx_80023694();
        sfx_forward();
        ext::gm_801A4B60();
        ext::gm_SetPendingGameMode(GM_TITLE as u8);
        ext::gm_SetNewGameModePending();
        return;
    }
    if ext::gm_801A4BA8() >= 0x4B0 {
        ext::lbAudioAx_80024C84();
        ext::lbAudioAx_80023694();
        // m-ex "Disable Movies / Title InGame Demo": the demo-start block is skipped
        if ext::Mex_Enabled(c"no_title_demo".as_ptr()) != 0 {
            ext::gm_801A4B60();
            return;
        }
        if ext::gm_GetCurrentSceneIndex() == 3 && ext::gmMainLib_8015DB00() % 2 == 0 {
            ext::gmMainLib_8015DB18();
            ext::gm_SetNextGameModeStateId(0);
        }
        ext::gm_801A4B60();
        return;
    }
    match lbl_804D65E8 {
        0 => {
            if ext::gm_801A4BA8() == 0x190 {
                lbl_804D65E8 = 1;
                let temp_r31 = ext::gm_801BF6D8();
                ext::Camera_8002EEC8(60.0);
                ext::Camera_8002E6FC(temp_r31);
                ext::Camera_8002ED9C(30.0);
                ext::Camera_8002EC7C(0.017453292f32 * (ext::HSD_Randi(0x47) - 0x23) as f32);
                ext::Camera_8002EB5C(0.017453292f32 * ext::HSD_Randi(0x10) as f32);
                for i in 0..4i32 {
                    if i != ext::gm_801BF6D8() {
                        ext::Player_SetPlayerAndEntityCpuLevel(i, 2);
                    }
                    ext::Player_SetMoreFlagsBit4(i, 1);
                }
            }
        }
        1 => {
            if ext::gm_801A4BA8() == 0x280 || ext::Player_800368F8(ext::gm_801BF6D8()) == 0 {
                lbl_804D65E8 = 2;
                let temp_r31_2 = ext::gm_801BF6F8();
                ext::Camera_8002EEC8(60.0);
                ext::Camera_8002E6FC(temp_r31_2);
                ext::Camera_8002ED9C(30.0);
                ext::Camera_8002EC7C(0.017453292f32 * (ext::HSD_Randi(0x47) - 0x23) as f32);
                ext::Camera_8002EB5C(0.017453292f32 * ext::HSD_Randi(0x10) as f32);
                ext::Player_SetPlayerAndEntityCpuLevel(ext::gm_801BF6F8(), 9);
                for i in 0..4i32 {
                    if i != ext::gm_801BF6F8() {
                        ext::Player_SetPlayerAndEntityCpuLevel(i, 2);
                    }
                }
            }
        }
        2 => {
            if ext::gm_801A4BA8() == 0x370 || ext::Player_800368F8(ext::gm_801BF6F8()) == 0 {
                ext::Camera_SetModeToStandard();
                for i in 0..4i32 {
                    ext::Player_SetPlayerAndEntityCpuLevel(i, 9);
                    ext::Player_SetMoreFlagsBit4(i, 0);
                }
            }
        }
        _ => {}
    }
}

pub unsafe fn gm_80183218() {
    lbl_804D65E8 = 0;
    ext::HSD_GObj_SetupProc(ext::GObj_Create(0xF, 0x11, 0), Some(fn_80182F40), 0x15);
}
