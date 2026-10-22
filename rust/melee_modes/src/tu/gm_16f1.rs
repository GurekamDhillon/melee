//! Native port of src/melee/gm/gm_16F1.c: the match-bonus ("special message") decision table,
//! the player ranking helpers, and the unlock/trophy-message conditions. Verified against the C by
//! tests/tu_gm_16f1.rs (state oracle).
//!
//! Notes on the port:
//! * `fn_801701C0` keeps `scores` in a 6-element array (the C declares `s32 scores[4]` but indexes it
//!   with the player slot 0..5; the oracle compiles the C with `scores[6]` for the same reason).
//! * The m-ex hooks (`Mex_Enabled`) are the TARGET_PC branches of the C.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::ext_hook::memzero;
use crate::ext_fns;
use core::ffi::c_void;
use melee_types::*;

mod data;
pub use data::{lbl_803B7A60, lbl_803B7AD0, lbl_803D5648, lbl_803D5A4C};

/// `struct lbl_803D5A4C_t` (gm_16F1.static.h): one decision-table row.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_803D5A4C_t {
    pub kind: i16,
    pub x2: u16,
    pub x4: u8,
    /// `Gm_DecType`
    pub x5: u8,
    pub x6: u8,
}

/// The anonymous struct of `lbl_803D5A4C`: 257 rows (0x80C bytes) + padding.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_803D5A4C_tbl {
    pub entries: [lbl_803D5A4C_t; 257],
    pub pad: [u8; 4],
}

/// `struct lbl_803B7A60_t` (gm_16F1.static.h): a block of zero rows used as array initialisers.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_803B7A60_t {
    pub x0: [i32; 4],
    pub x10: [i32; 4],
    pub x20: [i32; 4],
    pub x30: [i32; 4],
    pub x40: [f32; 4],
    pub x50: [u32; 4],
    pub x60: [u32; 4],
}

/// `struct lbl_804D65A8_t` (gm_16F1.c): the 7 ranking bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_804D65A8_t {
    pub x0: u8,
    pub x1: u8,
    pub x2: u8,
    pub x3: u8,
    pub x4: u8,
    pub x5: u8,
    pub x6: u8,
}

/// The anonymous struct of `lbl_8046DBC8`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_8046DBC8_x2 {
    pub x0: u8,
    pub x1: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_8046DBC8_t {
    pub x0: u8,
    pub x1: u8,
    pub x2: [lbl_8046DBC8_x2; 6],
}

/// The anonymous struct of `lbl_804D65A0`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_804D65A0_t {
    pub x0: u8,
    pub x1: [u8; 7],
}

/// `struct lbl_803B7AD0_t` (gm_16F1.c).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_803B7AD0_t {
    pub x0: u8,
    pub x1: u8,
    pub x2: u8,
    pub x4: u16,
}

/// `struct lbl_803D6300_t` (gm_16F1.c): a trophy-message condition. `x4` returns a `bool` (int).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_803D6300_t {
    pub x0: u16,
    pub x2: u16,
    pub x4: Option<unsafe fn() -> i32>,
}

pub static mut lbl_8046DBC8: lbl_8046DBC8_t = unsafe { core::mem::zeroed() };
pub static mut lbl_804D65A0: lbl_804D65A0_t = unsafe { core::mem::zeroed() };
pub static mut lbl_804D65A8: [u8; 6] = [0; 6];
pub static mut lbl_804D65B0: i32 = 0;

/// Callees in TUs that are not ported yet.
pub mod ext {
    use super::*;
    ext_fns! {
        fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32);
        fn Mex_Enabled(a0: *const i8) -> i32;
        fn Player_GetFalls(a0: i32) -> i32;
        fn Player_GetKOsByPlayerIndex(a0: i32, a1: i32) -> i32;
        fn Player_GetPlayerSlotType(a0: i32) -> Gm_PKind;
        fn Player_GetSelfDestructs(a0: i32) -> u32;
        fn Player_GetTeam(a0: i32) -> i32;
        fn Toy_803048C0(a0: i32) -> i32;
        fn Toy_GetTrophyTotal() -> i32;
        fn fn_80162CCC() -> i32;
        fn fn_801630C4() -> i32;
        fn fn_80163D24() -> i32;
        fn fn_80163D74() -> i32;
        fn fn_80163FA4(a0: u8) -> i32;
        fn fn_8016F160(a0: i32, a1: i32) -> i32;
        fn fn_80160710(a0: i32) -> u8;
        fn fn_801607A8(a0: i32) -> u8;
        fn fn_801607F4(a0: i32) -> u8;
        fn fn_8017DEC8(a0: i32) -> *mut Unk1PData;
        fn fn_8017DF28() -> *mut Unk1PData;
        fn fn_8017E0E4() -> i32;
        fn fn_8017E160() -> i32;
        fn gmMainLib_8015CCE4() -> *mut gmm_x0_44_t;
        fn gmMainLib_8015CEFC(a0: i32) -> i32;
        fn gmMainLib_8015CF94() -> i32;
        fn gmMainLib_8015CFCC(a0: u8) -> i32;
        fn gmMainLib_8015D0F4(a0: u8) -> i32;
        fn gmMainLib_8015D194(a0: u8) -> *mut u8;
        fn gmMainLib_8015D1AC(a0: u8) -> i32;
        fn gmMainLib_8015D21C(a0: u8) -> i32;
        fn gmMainLib_8015D2BC(a0: u8) -> *mut u8;
        fn gmMainLib_8015D2D4(a0: u8) -> i32;
        fn gmMainLib_8015D344(a0: u8) -> i32;
        fn gmMainLib_8015D3E4(a0: u8) -> *mut u8;
        fn gmMainLib_8015D4A8(a0: u8) -> i32;
        fn gmMainLib_8015D508() -> i32;
        fn gmMainLib_8015D5DC() -> i32;
        fn gmMainLib_8015D804(a0: i32) -> *mut u32;
        fn gmMainLib_8015D818(a0: u32) -> i32;
        fn gmMainLib_8015D8B0(a0: u32);
        fn gmMainLib_8015D8D8(a0: u32) -> i32;
        fn gmMainLib_8015D8FC(a0: u32);
        fn gmMainLib_8015D924(a0: u32);
        fn gmMainLib_8015D94C(a0: u32) -> i32;
        fn gmMainLib_8015D984(a0: u32) -> i32;
        fn gmMainLib_8015DA1C(a0: u32) -> i32;
        fn gmMainLib_8015DA68(a0: u32);
        fn gmMainLib_8015DB6C(a0: u8) -> u8;
        fn gmMainLib_8015EDA4() -> *mut u16;
        fn gmMainLib_8015EDC8() -> *mut gmm_x1868_1A8_t;
        fn gmMainLib_8015ED98() -> *mut gmm_retval_ED98;
        fn gmMainLib_8015EDE4();
        fn gmMainLib_8015EDF8();
        fn gmMainLib_8015EE1C();
        fn gmMainLib_8015EE30();
        fn gmMainLib_8015EE54();
        fn gmMainLib_8015EE68();
        fn gmMainLib_8015EEA0();
        fn gmMainLib_8015EEB4();
        fn gm_80162D1C() -> i32;
        fn gm_80162EC8() -> i32;
        fn gm_80162F18() -> i32;
        fn gm_80163114() -> i32;
        fn gm_80164430(a0: u16) -> i32;
        fn gm_80164600() -> i32;
        fn gm_80164ABC() -> i32;
        fn gm_8016B3D8() -> i32;
        fn gm_8016B774() -> *mut MatchEnd;
        fn gm_801BEBC0(a0: u8) -> u8;
        fn gm_CKindToUnlockIndex(a0: CharacterKind) -> i8;
        fn gm_GetAdventureData() -> *mut UnkAdventureData;
        fn gm_GetCKindByUnlockIndex(a0: i32) -> u8;
        fn gm_GetStartMeleeRules() -> *mut StartMeleeRules;
        fn gm_GetVsPlayMatchTotal() -> u32;
        fn gm_IsCKindUnlocked(a0: u8) -> i32;
        fn gm_IsCurrently1PMode_inline() -> i32;
        fn gm_SelKindToUnlockIndex(a0: SelectableCharacterKind) -> u8;
        fn lbLang_GetLanguageSetting() -> i32;
        fn lbLang_IsSettingUS() -> i32;
        fn lbTime_GetTimeInSeconds() -> u32;
        fn memzero(a0: *mut c_void, a1: i32) = super::memzero;
        fn pl_80038914() -> *mut pl_804D6470_t;
        fn pl_80039418(a0: i32, a1: i32) -> u32;
        fn pl_80039450(a0: i32);
        fn pl_800407C8(a0: i32) -> f32;
        fn pl_80040870(a0: i32) -> f32;
        fn pl_80040894(a0: i32) -> u32;
        fn pl_800408B8(a0: i32) -> u32;
        fn pl_800408DC(a0: i32) -> i32;
        fn un_80304470() -> i32;
        fn un_80304510() -> i32;
    }
}

const Gm_PKind_NA: u8 = 3;
const LANG_JP: i32 = 0;
const LANG_US: i32 = 1;
const NUM_UNLOCKABLE_CHARACTERS: i32 = 11;

#[inline]
pub unsafe fn entries() -> *mut lbl_803D5A4C_t {
    lbl_803D5A4C.entries.as_mut_ptr()
}

pub unsafe fn fn_8016F180(kind: i32) -> i32 {
    let mut curr = entries();
    while (*curr).kind as i32 != kind {
        if (*curr).kind == 0x29A {
            return 0;
        }
        curr = curr.add(1);
    }
    (*curr).x4 as i32
}

pub unsafe fn gmDecisionGetType(kind: i32) -> u32 {
    let mut curr = entries();
    while (*curr).kind as i32 != kind {
        if (*curr).kind == 0x29A {
            return 0; // Gm_DecType_Flag
        }
        curr = curr.add(1);
    }
    (*curr).x5 as u32
}

pub unsafe fn fn_8016F1F0(idx: i32) -> i32 {
    lbl_803D5A4C.entries[idx as usize].kind as i32
}

pub unsafe fn gm_8016F208(kind: i32) -> i32 {
    let mut curr = entries();
    while (*curr).kind as i32 != kind {
        if (*curr).kind == 0x29A {
            return 0;
        }
        curr = curr.add(1);
    }
    if (*curr).x2 == 0xDE && ext::lbLang_IsSettingUS() != 0 {
        return 0x102;
    }
    (*curr).x2 as i32
}

pub unsafe fn fn_8016F280(arg0: i32) -> i32 {
    gm_8016F208(arg0).wrapping_sub(2)
}

pub unsafe fn gm_8016F2F8(kind: i32, _arg1: u8) -> i32 {
    let mut curr = entries();
    while (*curr).kind as i32 != kind {
        if (*curr).kind == 0x29A {
            return -1;
        }
        curr = curr.add(1);
    }
    lbl_803D5648[((*curr).x2 as i32 - 2) as usize]
}

pub unsafe fn fn_8016F344(match_end: *mut MatchEnd) {
    for i in 0..4 {
        if (*match_end).player_standings[i].pkind != Gm_PKind_NA {
            ext::pl_80039450(i as i32);
        }
    }
}

/// `static inline u16 fn_8016F39C_GetSisTextId(int kind)`: gm_8016F208 with a u16 result.
pub unsafe fn fn_8016F39C_GetSisTextId(kind: i32) -> u16 {
    let mut curr = entries();
    while (*curr).kind as i32 != kind {
        if (*curr).kind == 0x29A {
            return 0;
        }
        curr = curr.add(1);
    }
    if (*curr).x2 == 0xDE && ext::lbLang_IsSettingUS() != 0 {
        return 0x102;
    }
    (*curr).x2
}

pub unsafe fn fn_8016F39C(arg0: *mut *mut HSD_Text, arg1: *mut MatchEnd, arg2: u8, arg3: u16, arg4: u8, arg5: u8) -> i32 {
    let mut count: i32 = 0;

    if arg3 as u32 >= 0x101 {
        return -1;
    }

    let mut idx = arg3 as i32;
    while (idx as u32) < 0x101 {
        let mut matched = 0;
        if (lbl_803D5A4C.entries[idx as usize].kind as i32) < 0xD7 {
            let flags = fn_8016F180(idx) as u8;
            if arg4 & flags != 0 && ext::pl_80039418(arg5 as i32, idx) != 0 {
                matched = 1;
            }
        } else {
            let flags = fn_8016F180(idx) as u8;
            if arg4 & flags != 0 && fn_801701C0(arg1, arg5 as i32, idx) as u32 != 0 {
                matched = 1;
            }
        }

        if matched != 0 {
            ext::HSD_SisLib_803A6368(*arg0.add(count as usize), fn_8016F39C_GetSisTextId(idx) as i32);
            count += 1;
            if count == arg2 as i32 {
                break;
            }
        }
        idx += 1;
    }
    count
}

pub unsafe fn fn_8016F548(arg0: *mut MatchEnd, arg1: u16, mask: u8, player_id: u8) -> i32 {
    if arg1 as u32 >= 0x101 {
        return -1;
    }

    let mut i = arg1 as i32;
    while (i as u32) < 0x101 {
        if (lbl_803D5A4C.entries[i as usize].kind as i32) < 0xD7 {
            if mask & fn_8016F180(i) as u8 != 0 && ext::pl_80039418(player_id as i32, i) != 0 {
                return i;
            }
        } else if mask & fn_8016F180(i) as u8 != 0 && fn_801701C0(arg0, player_id as i32, i) as u32 != 0 {
            return i;
        }
        i += 1;
    }

    let mut i = 0x100;
    while i >= 0 {
        if (lbl_803D5A4C.entries[i as usize].kind as i32) < 0xD7 {
            if mask & fn_8016F180(i) as u8 != 0 && ext::pl_80039418(player_id as i32, i) != 0 {
                return i;
            }
        } else if mask & fn_8016F180(i) as u8 != 0 && fn_801701C0(arg0, player_id as i32, i) as u32 != 0 {
            return i;
        }
        i -= 1;
    }

    -1
}

pub unsafe fn fn_8016F740(arg0: *mut MatchEnd, arg1: u16, mask: u8, player_id: u8) -> i32 {
    let temp = (arg1 as u32).wrapping_add(1);
    if temp >= 0x101 {
        return -1;
    }

    let mut i = temp as i32;
    while (i as u32) < 0x101 {
        if (lbl_803D5A4C.entries[i as usize].kind as i32) < 0xD7 {
            let flags = fn_8016F180(i) as u8;
            if mask & flags != 0 && ext::pl_80039418(player_id as i32, i) != 0 {
                return i;
            }
        } else {
            let flags = fn_8016F180(i) as u8;
            if mask & flags != 0 && fn_801701C0(arg0, player_id as i32, i) as u32 != 0 {
                return i;
            }
        }
        i += 1;
    }

    -1
}

pub unsafe fn fn_8016F870(arg0: *mut MatchEnd, arg1: u16, mask: u8, player_id: u8) -> i32 {
    if arg1 > 0x101 || arg1 == 0 {
        return -1;
    }

    let mut i = arg1 as i32 - 1;
    while i >= 0 {
        let kind = lbl_803D5A4C.entries[i as usize].kind as i32;

        if kind < 0xD7 {
            let flags = fn_8016F180(i) as u8;
            if mask & flags != 0 && ext::pl_80039418(player_id as i32, i) != 0 {
                return i;
            }
        } else {
            let flags = fn_8016F180(i) as u8;
            if mask & flags != 0 && fn_801701C0(arg0, player_id as i32, i) as u32 != 0 {
                return i;
            }
        }
        i -= 1;
    }

    -1
}

pub unsafe fn fn_8016F9A8(arg0: *mut MatchEnd, arg1: u16, mask: u8, player_id: u8) -> i32 {
    let mut count = 0;
    if arg1 as u32 >= 0x101 {
        return -1;
    }

    let mut i = arg1 as i32;
    while (i as u32) < 0x101 {
        if (lbl_803D5A4C.entries[i as usize].kind as i32) < 0xD7 {
            let flags = fn_8016F180(i) as u8;
            if mask & flags != 0 && ext::pl_80039418(player_id as i32, i) != 0 {
                count += 1;
            }
        } else {
            let flags = fn_8016F180(i) as u8;
            if mask & flags != 0 && fn_801701C0(arg0, player_id as i32, i) as u32 != 0 {
                count += 1;
            }
        }
        i += 1;
    }

    count
}

/// Ranks the (up to 6) present players by `x20 - (x24 - sd) + sd_penalty * sd`; `rankings[6]` is the
/// worst rank. Shared by fn_8016FAD4 and fn_801701C0.
pub unsafe fn rank_players(rules: *mut MatchEnd, scores: &mut [i32; 6], rankings: &mut [u8; 7]) {
    let x58 = (*rules).player_standings.as_mut_ptr();
    for i in 0..6 {
        if (*x58.add(i)).pkind != Gm_PKind_NA {
            let sd = (*x58.add(i)).self_destructs;
            scores[i] = (*x58.add(i))
                .x20
                .wrapping_sub((*x58.add(i)).x24.wrapping_sub(sd as i32) as u32)
                .wrapping_add(((*rules).sd_penalty as i32).wrapping_mul(sd as i32) as u32) as i32;
        }
    }

    for i in 0..6 {
        if (*x58.add(i)).pkind != Gm_PKind_NA {
            for j in 0..6 {
                if (*x58.add(j)).pkind != Gm_PKind_NA && i != j && scores[i] < scores[j] {
                    rankings[i] += 1;
                }
            }
            if rankings[6] < rankings[i] {
                rankings[6] = rankings[i];
            }
        }
    }
}

pub unsafe fn fn_8016FAD4(rules: *mut MatchEnd, kind: i32, flags: i32, player: u8) -> i32 {
    let mut entry = entries();
    let x58 = (*rules).player_standings.as_mut_ptr();
    let mut rankings = [0u8; 7];
    let mut scores = [0i32; 6];

    while (*entry).kind as i32 != kind {
        if (*entry).kind == 0x29A {
            return -1;
        }
        entry = entry.add(1);
    }

    rank_players(rules, &mut scores, &mut rankings);

    let base = lbl_803D5648[((*entry).x2 as i32 - 2) as usize];
    if ((*entry).x6 & 2) != 0 && (flags & 2) != 0 {
        let pr = rankings[player as usize];
        if pr == 0 {
            return base.wrapping_mul(2);
        }
        if pr == rankings[6] {
            return base / 2;
        }
    }

    if ((*entry).x6 & 1) != 0 && (flags & 4) != 0 {
        return ((base / 10) as u32).wrapping_mul(ext::pl_80039418(player as i32, kind)) as i32;
    }

    if (*entry).x5 == 1 {
        let p = x58.add(player as usize);
        if kind == 0xE3 {
            return (base as u32).wrapping_mul((*p).x20) as i32;
        } else if kind == 0xE4 {
            return base.wrapping_mul((*p).x24.wrapping_sub((*p).self_destructs as i32));
        } else if kind == 0xE5 {
            return base.wrapping_mul((*p).self_destructs as i32);
        } else {
            return (base as u32).wrapping_mul(ext::pl_80039418(player as i32, kind)) as i32;
        }
    }

    base
}

pub unsafe fn fn_8016FFD4(arg0: *mut MatchEnd, arg1: i32, arg2: u8) -> i32 {
    let mut count: i32 = 0;

    for i in 0..0x101i32 {
        if (lbl_803D5A4C.entries[i as usize].kind as i32) < 0xD7 {
            if (arg1 & 0xFF) & (fn_8016F180(i) as u8 as i32) != 0 && ext::pl_80039418(arg2 as i32, i) != 0 {
                count = count.wrapping_add(fn_8016FAD4(arg0, i, arg1, arg2));
            }
        } else if (arg1 & 0xFF) & (fn_8016F180(i) as u8 as i32) != 0 {
            if fn_801701C0(arg0, arg2 as i32, i) as u32 != 0 {
                count = count.wrapping_add(fn_8016FAD4(arg0, i, arg1, arg2));
            }
        }
    }
    count
}

pub unsafe fn fn_80170110(arg0: *mut MatchEnd, idx: i32, mask: i32, player_id: u8) -> i32 {
    let kind = lbl_803D5A4C.entries[idx as usize].kind as i32;

    let flags = fn_8016F180(kind) as u8;
    if (mask as u8) & flags != 0 {
        if kind < 0xD7 {
            return ext::pl_80039418(player_id as i32, kind) as i32;
        }
        return fn_801701C0(arg0, player_id as i32, kind);
    }
    0
}

pub unsafe fn gm_801701A0() {
    lbl_804D65A0.x0 = 1;
}

pub unsafe fn fn_801701AC() {
    lbl_804D65A0.x0 = 0;
}

pub unsafe fn fn_801701B8() -> i32 {
    lbl_804D65A0.x0 as i32
}

/// Sorts `vals[0..4]` descending (bubble sort as in the C).
#[inline]
fn sort4_desc<T: PartialOrd + Copy>(vals: &mut [T]) {
    let mut j = 3;
    while j >= 1 {
        for i in 0..j {
            if vals[i] < vals[i + 1] {
                let tmp = vals[i + 1];
                vals[i + 1] = vals[i];
                vals[i] = tmp;
            }
        }
        j -= 1;
    }
}

pub unsafe fn fn_801701C0(rules: *mut MatchEnd, arg1: i32, arg2: i32) -> i32 {
    let zeroes = &lbl_803B7A60;
    let flags = (*rules)._x448.as_mut_ptr();
    let x58 = (*rules).player_standings.as_mut_ptr();
    let mut scores = [0i32; 6];
    let mut rankings = [0u8; 7];

    if lbl_804D65A0.x0 != 0 {
        return 0;
    }
    if rules.is_null() || x58.is_null() {
        return 0;
    }

    if (*rules).match_kind == 3 {
        fn_80171B64(rankings.as_mut_ptr() as *mut lbl_804D65A8_t);
    } else {
        rank_players(rules, &mut scores, &mut rankings);
    }

    let p = |k: usize| x58.add(k);
    let rk = |k: i32| rankings[k as usize];

    match arg2 {
        0xD7 => {
            if fn_801701C0(rules, arg1, 0xD9) as u32 != 0 {
                return 0;
            }
            for i in 0..4 {
                if (*p(i)).pkind != Gm_PKind_NA && i as i32 != arg1 && rankings[i] == 0 {
                    return 0;
                }
            }
            if rk(arg1) == 0 {
                return 1;
            }
            0
        }
        0xD8 => {
            if fn_801701C0(rules, arg1, 0xDA) as u32 != 0 {
                return 0;
            }
            for i in 0..4 {
                if (*p(i)).pkind != Gm_PKind_NA && i as i32 != arg1 && rankings[i] == rankings[6] {
                    return 0;
                }
            }
            if rk(arg1) == rankings[6] {
                return 1;
            }
            0
        }
        0xD9 => {
            for k in 0..4 {
                if (*p(k)).pkind != Gm_PKind_NA && k as i32 != arg1 && fn_80171B00(k as i32) != 0 {
                    return 0;
                }
            }
            if fn_80171B00(arg1) != 0 {
                return 1;
            }
            0
        }
        0xDA => {
            for k in 0..4 {
                if (*p(k)).pkind != Gm_PKind_NA && k as i32 != arg1 && fn_80171B2C(k as i32) != 0 {
                    return 0;
                }
            }
            if fn_80171B2C(arg1) != 0 {
                return 1;
            }
            0
        }
        0xDB => {
            let mut vals = [0i32; 6];
            if (*p(arg1 as usize)).x20 >= 3 {
                vals[..4].copy_from_slice(&zeroes.x0);
                for i in 0..4 {
                    if (*p(i)).pkind != Gm_PKind_NA {
                        vals[i] = (*p(i)).x20 as i32;
                    }
                }
                sort4_desc(&mut vals);
                if vals[0] as u32 == (*p(arg1 as usize)).x20 && vals[0] >= vals[1].wrapping_mul(2) {
                    return 1;
                }
            }
            0
        }
        0xDC => {
            if fn_801701C0(rules, arg1, 0xDB) as u32 == 0 {
                for i in 0..4 {
                    if (*p(i)).pkind != Gm_PKind_NA && i as i32 != arg1 && (*p(i)).x20 >= (*p(arg1 as usize)).x20 {
                        return 0;
                    }
                }
                if (*p(arg1 as usize)).x20 != 0 {
                    return 1;
                }
            }
            0
        }
        0xDD => {
            let mut vals = [0i32; 4];
            if (*p(arg1 as usize)).x40 >= 3 {
                vals.copy_from_slice(&zeroes.x10);
                for i in 0..4 {
                    if (*p(i)).pkind != Gm_PKind_NA {
                        vals[i] = (*p(i)).x40 as i32;
                    }
                }
                sort4_desc(&mut vals);
                if vals[0] as u32 == (*p(arg1 as usize)).x40 && vals[0] >= vals[1].wrapping_mul(2) {
                    return 1;
                }
            }
            0
        }
        0xDE => {
            if fn_801701C0(rules, arg1, 0xDD) as u32 == 0 {
                for i in 0..4 {
                    if (*p(i)).pkind != Gm_PKind_NA && i as i32 != arg1 && (*p(i)).x40 >= (*p(arg1 as usize)).x40 {
                        return 0;
                    }
                }
                if (*p(arg1 as usize)).x40 != 0 {
                    return 1;
                }
            }
            0
        }
        0xDF => {
            let mut vals = [0i32; 4];
            let player_net = (*p(arg1 as usize)).x24.wrapping_sub((*p(arg1 as usize)).self_destructs as i32);
            if player_net as u32 >= 3 {
                vals.copy_from_slice(&zeroes.x20);
                for i in 0..4 {
                    if (*p(i)).pkind != Gm_PKind_NA {
                        vals[i] = (*p(i)).x24.wrapping_sub((*p(i)).self_destructs as i32);
                    }
                }
                sort4_desc(&mut vals);
                if vals[0] as u32
                    == (*p(arg1 as usize)).x24.wrapping_sub((*p(arg1 as usize)).self_destructs as i32) as u32
                    && vals[0] >= vals[1].wrapping_mul(2)
                {
                    return 1;
                }
            }
            0
        }
        0xE0 => {
            if fn_801701C0(rules, arg1, 0xDF) as u32 == 0 {
                for i in 0..4 {
                    if (*p(i)).pkind != Gm_PKind_NA
                        && i as i32 != arg1
                        && ((*p(i)).x24.wrapping_sub((*p(i)).self_destructs as i32) as u32)
                            >= ((*p(arg1 as usize)).x24.wrapping_sub((*p(arg1 as usize)).self_destructs as i32) as u32)
                    {
                        return 0;
                    }
                }
                if (*p(arg1 as usize)).x24.wrapping_sub((*p(arg1 as usize)).self_destructs as i32) != 0 {
                    return 1;
                }
            }
            0
        }
        0xE1 => {
            let mut vals = [0i32; 4];
            if (*p(arg1 as usize)).self_destructs >= 3 {
                vals.copy_from_slice(&zeroes.x30);
                for i in 0..4 {
                    if (*p(i)).pkind != Gm_PKind_NA {
                        vals[i] = (*p(i)).self_destructs as i32;
                    }
                }
                sort4_desc(&mut vals);
                if vals[0] == (*p(arg1 as usize)).self_destructs as i32 && vals[0] >= vals[1].wrapping_mul(2) {
                    return 1;
                }
            }
            0
        }
        0xE2 => {
            if fn_801701C0(rules, arg1, 0xE1) as u32 == 0 {
                for i in 0..4 {
                    if (*p(i)).pkind != Gm_PKind_NA
                        && i as i32 != arg1
                        && (*p(i)).self_destructs >= (*p(arg1 as usize)).self_destructs
                    {
                        return 0;
                    }
                }
                if (*p(arg1 as usize)).self_destructs != 0 {
                    return 1;
                }
            }
            0
        }
        0xE3 => ((*p(arg1 as usize)).x20 != 0) as i32,
        0xE4 => ((*p(arg1 as usize)).x24.wrapping_sub((*p(arg1 as usize)).self_destructs as i32) != 0) as i32,
        0xE5 => ((*p(arg1 as usize)).self_destructs != 0) as i32,
        0xE6 => ((*flags.add(1) >> 1) & 1) as i32,
        0xE7 => ((*flags.add(1) >> 2) & 1) as i32,
        0xE8 => (*flags.add(1) & 1) as i32,
        0xE9 => ((*flags.add(2) >> 7) & 1) as i32,
        0xEA => ((*flags.add(2) >> 6) & 1) as i32,
        0xEB => {
            let f0 = *flags;
            (((f0 >> 1) & 1) != 0 && ((f0 >> 5) & 1) != 0) as i32
        }
        0xEC => ((*flags >> 7) & 1) as i32,
        0xED => ((*flags >> 6) & 1) as i32,
        0xEE => ((*flags >> 5) & 1) as i32,
        0xEF => {
            let f0 = *flags;
            ((((f0 >> 4) & 1) != 0) || (((f0 >> 3) & 1) != 0) || (((f0 >> 2) & 1) != 0)) as i32
        }
        0xF6 => ((((*flags.add(2) >> 5) & 1) != 0) && (((*flags >> 7) & 1) != 0)) as i32,
        0xF7 => ((*flags.add(2) >> 3) & 1) as i32,
        0xF8 => ((*flags.add(2) >> 4) & 1) as i32,
        0xF0 => (*flags & 1) as i32,
        0xF1 => ((*flags.add(1) >> 7) & 1) as i32,
        0xF2 => ((*flags.add(1) >> 6) & 1) as i32,
        0xF3 => ((*flags.add(1) >> 5) & 1) as i32,
        0xF4 => ((*flags.add(1) >> 4) & 1) as i32,
        0xF5 => ((*flags.add(1) >> 3) & 1) as i32,
        0xF9 => {
            let mut vals = [0.0f32; 4];
            vals.copy_from_slice(&zeroes.x40);
            for i in 0..4 {
                if (*p(i)).pkind != Gm_PKind_NA {
                    vals[i] = ext::pl_800407C8(i as i32);
                }
            }
            sort4_desc(&mut vals);
            if vals[0] == ext::pl_800407C8(arg1) && vals[0] > 2.0f32 * vals[1] {
                return 1;
            }
            0
        }
        0xFA => {
            if ext::fn_8016F160(arg1, 0x60) as u32 != 0 && rk(arg1) != rankings[6] {
                return 1;
            }
            0
        }
        0xFB => {
            let mut min_dmg = 0.0f32;
            for i in 0..4 {
                if (*p(i)).pkind != Gm_PKind_NA && min_dmg > ext::pl_80040870(i as i32) {
                    min_dmg = ext::pl_80040870(i as i32);
                }
            }
            if min_dmg < 0.0 && min_dmg == ext::pl_80040870(arg1) {
                return 1;
            }
            0
        }
        0xFC => {
            for i in 0..4 {
                if (*p(i)).pkind != Gm_PKind_NA && i as i32 != arg1 && ((*p(i)).anon_3.x3 & 1) != 0 {
                    return 0;
                }
            }
            if (*rules).match_kind == 3 {
                for i in 0..4 {
                    if (*p(i)).pkind != Gm_PKind_NA && i as i32 != arg1 && rankings[i] == 0 {
                        return 0;
                    }
                }
                if ((*p(arg1 as usize)).anon_3.anon_0.x3_b7() & 1) == 0
                    && rk(arg1) == 0
                    && (*p(arg1 as usize)).x20 == 0
                {
                    return 1;
                }
            } else {
                let mode = if fn_80171A88() == 0 { 0 } else { 6 };
                if mode == 0 {
                    for i in 0..4 {
                        if (*p(i)).pkind != Gm_PKind_NA && i as i32 != arg1 && (*p(i)).is_big_loser == 0 {
                            return 0;
                        }
                    }
                    if ((*p(arg1 as usize)).anon_3.anon_0.x3_b7() & 1) == 0
                        && (*p(arg1 as usize)).is_big_loser == 0
                        && (*p(arg1 as usize)).x20 == 0
                    {
                        return 1;
                    }
                } else if ((*p(arg1 as usize)).anon_3.anon_0.x3_b7() & 1) == 0 && (*p(arg1 as usize)).x20 == 0 {
                    return 1;
                }
            }
            0
        }
        0xFD => {
            if ext::pl_800408DC(arg1) as u32 != 0 && (*rules).x7 as u32 == ext::pl_800408DC(arg1) as u32 {
                return 1;
            }
            0
        }
        0xFE => {
            let mut vals = [0u32; 4];
            if ext::pl_800408B8(arg1) == 0 {
                return 0;
            }
            let threshold = (*ext::pl_80038914()).x13C;
            if ext::pl_800408B8(arg1) < threshold {
                let mode = if fn_80171A88() == 0 { 0 } else { 6 };
                if mode == 0 {
                    vals.copy_from_slice(&zeroes.x50);
                    for i in 0..4 {
                        if (*p(i)).pkind != Gm_PKind_NA && ext::pl_800408B8(i as i32) != 0 {
                            vals[i] = ext::pl_800408B8(i as i32);
                        }
                    }
                    // ascending
                    let mut j = 3;
                    while j >= 1 {
                        for k in 0..j {
                            if vals[k] > vals[k + 1] {
                                let tmp = vals[k + 1];
                                vals[k + 1] = vals[k];
                                vals[k] = tmp;
                            }
                        }
                        j -= 1;
                    }
                    if vals[0] == ext::pl_800408B8(arg1) && vals[0] <= vals[1] / 2 {
                        return 1;
                    }
                } else {
                    return 1;
                }
            }
            0
        }
        0xFF => {
            let mut vals = [0u32; 4];
            let threshold = (*ext::pl_80038914()).x140;
            if ext::pl_80040894(arg1) > threshold {
                let mode = if fn_80171A88() == 0 { 0 } else { 6 };
                if mode == 0 {
                    vals.copy_from_slice(&zeroes.x60);
                    for i in 0..4 {
                        if (*p(i)).pkind != Gm_PKind_NA {
                            vals[i] = ext::pl_80040894(i as i32);
                        }
                    }
                    sort4_desc(&mut vals);
                    if vals[0] == ext::pl_80040894(arg1) && vals[0] >= vals[1].wrapping_mul(2) {
                        return 1;
                    }
                } else {
                    return 1;
                }
            }
            0
        }
        _ => 0,
    }
}

pub unsafe fn fn_80171A88() -> i32 {
    let mut result = 0;
    if ext::gm_IsCurrently1PMode_inline() != 0 {
        result = ext::fn_8017E0E4();
        if result == -1 {
            result = 1;
        }
    }
    result
}

pub unsafe fn fn_80171AD4() {
    ext::memzero(
        core::ptr::addr_of_mut!(lbl_8046DBC8) as *mut c_void,
        core::mem::size_of::<lbl_8046DBC8_t>() as i32,
    );
}

pub unsafe fn fn_80171B00(arg0: i32) -> i32 {
    (lbl_8046DBC8.x2[arg0 as usize].x0 != 0) as i32
}

pub unsafe fn fn_80171B2C(arg0: i32) -> i32 {
    (lbl_8046DBC8.x1 != 0 && lbl_8046DBC8.x2[arg0 as usize].x1 != 0) as i32
}

pub unsafe fn fn_80171B64(arg0: *mut lbl_804D65A8_t) {
    (*arg0).x0 = lbl_804D65A8[0];
    (*arg0).x1 = lbl_804D65A8[1];
    (*arg0).x2 = lbl_804D65A8[2];
    (*arg0).x3 = lbl_804D65A8[3];
    (*arg0).x4 = lbl_804D65A8[4];
    (*arg0).x5 = lbl_804D65A8[5];
    (*arg0).x6 = lbl_804D65B0 as u8;
}

pub unsafe fn fn_80171BA4(arg0: *mut c_void) -> i32 {
    let rules = arg0 as *mut u8;
    let mut scores = [0i32; 6];
    let mut result: i32 = 0;

    ext::memzero(scores.as_mut_ptr() as *mut c_void, core::mem::size_of_val(&scores) as i32);
    ext::memzero(lbl_804D65A8.as_mut_ptr() as *mut c_void, core::mem::size_of_val(&lbl_804D65A8) as i32);
    lbl_804D65B0 = Gm_PKind_Human as i32;

    for player in 0..6 {
        if ext::Player_GetPlayerSlotType(player) != Gm_PKind_NA as Gm_PKind {
            let mut ko_count: i32 = 0;
            let mut falls = ext::Player_GetFalls(player);
            let mut self_destructs = ext::Player_GetSelfDestructs(player);
            if *rules.add(6) == 1 {
                let team = ext::Player_GetTeam(player);
                for j in 0..6 {
                    if ext::Player_GetPlayerSlotType(j) != Gm_PKind_NA as Gm_PKind {
                        if team != ext::Player_GetTeam(j) {
                            if player != j {
                                ko_count = ko_count.wrapping_add(ext::Player_GetKOsByPlayerIndex(player, j));
                            }
                        } else {
                            self_destructs =
                                self_destructs.wrapping_add(ext::Player_GetKOsByPlayerIndex(player, j) as u32);
                            falls = falls.wrapping_add(ext::Player_GetKOsByPlayerIndex(player, j));
                        }
                    }
                }
            } else {
                for j in 0..6 {
                    if ext::Player_GetPlayerSlotType(j) != Gm_PKind_NA as Gm_PKind {
                        if player != j {
                            ko_count = ko_count.wrapping_add(ext::Player_GetKOsByPlayerIndex(player, j));
                        } else {
                            self_destructs =
                                self_destructs.wrapping_add(ext::Player_GetKOsByPlayerIndex(player, j) as u32);
                            falls = falls.wrapping_add(ext::Player_GetKOsByPlayerIndex(player, j));
                        }
                    }
                }
            }
            falls = falls.wrapping_sub(self_destructs as i32);
            scores[player as usize] = ko_count
                .wrapping_sub(falls)
                .wrapping_add((self_destructs as i32).wrapping_mul(*rules.add(0xC) as i8 as i32));
        }
    }

    for player in 0..6 {
        result = ext::Player_GetPlayerSlotType(player) as i32;
        if result != Gm_PKind_NA as i32 {
            for j in 0..6 {
                if ext::Player_GetPlayerSlotType(j) != Gm_PKind_NA as Gm_PKind
                    && player != j
                    && scores[player as usize] < scores[j as usize]
                {
                    lbl_804D65A8[player as usize] = lbl_804D65A8[player as usize].wrapping_add(1);
                }
            }
            result = lbl_804D65A8[player as usize] as i32;
            if lbl_804D65B0 < result {
                lbl_804D65B0 = result;
            }
        }
    }
    result
}

pub unsafe fn fn_80171DC4() {
    let rules = ext::gm_8016B774();

    fn_80171BA4(rules as *mut c_void);

    if lbl_8046DBC8.x0 == 0 && 0 < lbl_804D65B0 {
        lbl_8046DBC8.x0 = 1;
        lbl_8046DBC8.x1 = 1;
        for i in 0..6 {
            if (*rules).player_standings[i].pkind != Gm_PKind_NA {
                if lbl_804D65A8[i] == 0 {
                    lbl_8046DBC8.x2[i].x0 = 1;
                } else if lbl_804D65A8[i] as i32 == lbl_804D65B0 {
                    lbl_8046DBC8.x2[i].x1 = 1;
                }
            }
        }
    } else {
        for i in 0..6 {
            if (*rules).player_standings[i].pkind != Gm_PKind_NA {
                if lbl_8046DBC8.x2[i].x0 == 1 && lbl_804D65A8[i] != 0 {
                    lbl_8046DBC8.x2[i].x0 = 0;
                }
                if lbl_8046DBC8.x2[i].x1 == 1 && (lbl_804D65A8[i] == 0 || lbl_804D65A8[i] as i32 != lbl_804D65B0) {
                    lbl_8046DBC8.x2[i].x1 = 0;
                }
            }
        }
    }
}

pub unsafe fn gm_801720B4() -> i32 {
    if ext::gm_8016B3D8() != 0 {
        return ext::fn_8017E160();
    }
    ((*ext::gm_GetStartMeleeRules()).is_teams == 1) as i32
}

pub unsafe fn gm_801720F8() -> i32 {
    if ext::gm_8016B3D8() != 0 {
        return 1;
    }
    ((*ext::gm_GetStartMeleeRules()).match_kind() == 1) as i32
}

pub unsafe fn gm_80172140() -> i32 {
    let temp_r3 = ext::fn_8017DF28();
    if !temp_r3.is_null() {
        return (*temp_r3).xC.x10 as i32;
    }
    0
}

pub unsafe fn gm_80172174() {
    ext::memzero(
        ext::gmMainLib_8015CCE4() as *mut c_void,
        core::mem::size_of::<gmm_x0_44_t>() as i32,
    );
}

pub unsafe fn gm_8017219C(arg0: u32) -> i32 {
    (ext::gmMainLib_8015D94C(arg0) != 0 && ext::gmMainLib_8015D8D8(arg0) != 0) as i32
}

#[inline]
pub unsafe fn gm_801721EC_1(i: u32) -> bool {
    ext::gmMainLib_8015D94C(i) != 0 && ext::gmMainLib_8015D8D8(i) != 0
}

#[inline]
pub unsafe fn gm_801721EC_2() -> bool {
    for i in 0..0x42u32 {
        if gm_801721EC_1(i) {
            return true;
        }
    }
    false
}

#[inline]
pub unsafe fn gm_801721EC_3(j: u32) -> bool {
    ext::gmMainLib_8015DA1C(j) != 0
}

#[inline]
pub unsafe fn gm_801721EC_4() -> bool {
    for j in 0..TY_TROPHY_COUNT as u32 {
        if gm_801721EC_3(j) {
            return true;
        }
    }
    false
}

pub unsafe fn gm_801721EC() -> i32 {
    // m-ex "Spoof No Pending Messages" (opt-in MELEE_MEX=no_pending_messages)
    if ext::Mex_Enabled(b"no_pending_messages\0".as_ptr() as *const i8) != 0 {
        return 0;
    }
    (gm_801721EC_2() || gm_801721EC_4()) as i32
}

pub unsafe fn fn_801722BC() -> i32 {
    (ext::fn_80163FA4(0x1E) == 0x1E) as i32
}

pub unsafe fn fn_801722F4() -> i32 {
    (ext::fn_80163FA4(0x33) == 0x33) as i32
}

pub unsafe fn fn_8017232C() -> i32 {
    for i in 0..0x19 {
        if *ext::gmMainLib_8015D194(i) == 4 {
            return 1;
        }
    }
    0
}

pub unsafe fn fn_80172380() -> i32 {
    for i in 0..0x19 {
        if *ext::gmMainLib_8015D2BC(i) == 4 {
            return 1;
        }
    }
    0
}

pub unsafe fn fn_801723D4() -> i32 {
    for i in 0..0x19 {
        if *ext::gmMainLib_8015D3E4(i) == 4 {
            return 1;
        }
    }
    0
}

pub unsafe fn fn_80172428() -> i32 {
    for i in 0..0x19 {
        if ext::gmMainLib_8015D1AC(i) == 1 {
            return 1;
        }
    }
    0
}

pub unsafe fn fn_80172478() -> i32 {
    for i in 0..0x19 {
        if ext::gmMainLib_8015D2D4(i) == 1 {
            return 1;
        }
    }
    0
}

/// Same as `crate::gm::fn_801724C8`, but int-returning so its address fits the `bool (*)(void)` table slots
/// (`lbl_803D6300_t::x4`), which hold `unsafe fn() -> i32`.
pub unsafe fn fn_801724C8() -> i32 {
    0
}

pub unsafe fn fn_801724D0() -> i32 {
    (ext::gm_GetVsPlayMatchTotal() >= 0x64) as i32
}

pub unsafe fn fn_80172504() -> i32 {
    (ext::gm_GetVsPlayMatchTotal() >= 0x3E8) as i32
}

pub unsafe fn fn_80172538() -> i32 {
    (ext::gm_GetVsPlayMatchTotal() >= 0x2710) as i32
}

pub unsafe fn fn_8017256C() -> i32 {
    (ext::gm_GetVsPlayMatchTotal() >= 0x186A0) as i32
}

pub unsafe fn fn_801725A8() -> i32 {
    (ext::gm_GetVsPlayMatchTotal() >= 0xF4240) as i32
}

pub unsafe fn fn_801725E4() -> i32 {
    (ext::lbLang_GetLanguageSetting() == LANG_JP && ext::gm_GetVsPlayMatchTotal() >= 0x1A90) as i32
}

pub unsafe fn fn_80172624() -> i32 {
    (ext::lbLang_GetLanguageSetting() == LANG_US && ext::gm_GetVsPlayMatchTotal() >= 0x1383) as i32
}

pub unsafe fn fn_80172664() -> i32 {
    (ext::gm_GetVsPlayMatchTotal() >= 0xC350) as i32
}

pub unsafe fn fn_80172698() -> i32 {
    (ext::Toy_GetTrophyTotal() >= 0x32) as i32
}

pub unsafe fn fn_801726CC() -> i32 {
    (ext::Toy_GetTrophyTotal() >= 0x64) as i32
}

pub unsafe fn fn_80172700() -> i32 {
    (ext::Toy_GetTrophyTotal() >= 0x96) as i32
}

pub unsafe fn fn_80172734() -> i32 {
    (ext::Toy_GetTrophyTotal() >= 0xC8) as i32
}

pub unsafe fn fn_80172768() -> i32 {
    (ext::Toy_GetTrophyTotal() >= 0xFA) as i32
}

/// The trophy-message condition table (42 rows, ended by id 0x42).
pub static mut lbl_803D6300: [lbl_803D6300_t; 42] = [
    lbl_803D6300_t { x0: 0x0016, x2: 0xFFFF, x4: Some(fn_801735F0) },
    lbl_803D6300_t { x0: 0x0017, x2: 0x0001, x4: Some(fn_80173644) },
    lbl_803D6300_t { x0: 0x0018, x2: 0xFFFF, x4: Some(fn_80173510) },
    lbl_803D6300_t { x0: 0x0019, x2: 0xFFFF, x4: None },
    lbl_803D6300_t { x0: 0x001A, x2: 0xFFFF, x4: None },
    lbl_803D6300_t { x0: 0x001B, x2: 0xFFFF, x4: Some(fn_8017367C) },
    lbl_803D6300_t { x0: 0x001C, x2: 0xFFFF, x4: Some(ext::gm_80164ABC) },
    lbl_803D6300_t { x0: 0x001D, x2: 0xFFFF, x4: Some(ext::gm_80164600) },
    lbl_803D6300_t { x0: 0x001E, x2: 0x0040, x4: Some(ext::fn_80162CCC) },
    lbl_803D6300_t { x0: 0x001F, x2: 0x0040, x4: Some(ext::gm_80162EC8) },
    lbl_803D6300_t { x0: 0x0020, x2: 0x0040, x4: Some(ext::fn_801630C4) },
    lbl_803D6300_t { x0: 0x0021, x2: 0x0040, x4: Some(ext::gm_80162D1C) },
    lbl_803D6300_t { x0: 0x0022, x2: 0x0040, x4: Some(ext::gm_80162F18) },
    lbl_803D6300_t { x0: 0x0023, x2: 0x0040, x4: Some(ext::gm_80163114) },
    lbl_803D6300_t { x0: 0x0024, x2: 0x0010, x4: Some(fn_801722BC) },
    lbl_803D6300_t { x0: 0x0025, x2: 0x0010, x4: Some(fn_801722F4) },
    lbl_803D6300_t { x0: 0x0026, x2: 0x0080, x4: Some(ext::gmMainLib_8015D508) },
    lbl_803D6300_t { x0: 0x0027, x2: 0x0020, x4: Some(ext::fn_80163D24) },
    lbl_803D6300_t { x0: 0x0028, x2: 0x0020, x4: Some(ext::fn_80163D74) },
    lbl_803D6300_t { x0: 0x0029, x2: 0x0040, x4: Some(fn_8017232C) },
    lbl_803D6300_t { x0: 0x002A, x2: 0x0040, x4: Some(fn_80172428) },
    lbl_803D6300_t { x0: 0x002B, x2: 0x0040, x4: Some(fn_80172380) },
    lbl_803D6300_t { x0: 0x002C, x2: 0x0040, x4: Some(fn_80172478) },
    lbl_803D6300_t { x0: 0x002D, x2: 0x0040, x4: Some(fn_801723D4) },
    lbl_803D6300_t { x0: 0x002E, x2: 0x0040, x4: Some(fn_801724C8) },
    lbl_803D6300_t { x0: 0x002F, x2: 0x0001, x4: Some(fn_801724D0) },
    lbl_803D6300_t { x0: 0x0030, x2: 0x0001, x4: Some(fn_80172504) },
    lbl_803D6300_t { x0: 0x0031, x2: 0x0001, x4: Some(fn_80172538) },
    lbl_803D6300_t { x0: 0x0032, x2: 0x0001, x4: Some(fn_8017256C) },
    lbl_803D6300_t { x0: 0x0033, x2: 0x0001, x4: Some(fn_801725A8) },
    lbl_803D6300_t { x0: 0x0034, x2: 0x0001, x4: Some(fn_801725E4) },
    lbl_803D6300_t { x0: 0x0035, x2: 0x0001, x4: Some(fn_80172624) },
    lbl_803D6300_t { x0: 0x0036, x2: 0x0001, x4: Some(fn_80172664) },
    lbl_803D6300_t { x0: 0x0037, x2: 0xFFFF, x4: Some(fn_80172698) },
    lbl_803D6300_t { x0: 0x0038, x2: 0xFFFF, x4: Some(fn_801726CC) },
    lbl_803D6300_t { x0: 0x0039, x2: 0xFFFF, x4: Some(fn_80172700) },
    lbl_803D6300_t { x0: 0x003A, x2: 0xFFFF, x4: Some(fn_80172734) },
    lbl_803D6300_t { x0: 0x003B, x2: 0xFFFF, x4: Some(fn_80172768) },
    lbl_803D6300_t { x0: 0x003C, x2: 0xFFFF, x4: Some(ext::un_80304470) },
    lbl_803D6300_t { x0: 0x003D, x2: 0xFFFF, x4: Some(ext::un_80304510) },
    lbl_803D6300_t { x0: 0x0041, x2: 0x0010, x4: Some(ext::gmMainLib_8015CF94) },
    lbl_803D6300_t { x0: 0x0042, x2: 0x0000, x4: None },
];

pub unsafe fn fn_8017279C(arg0: i32, arg1: u16) -> i32 {
    let mut cur = lbl_803D6300.as_ptr();
    while (*cur).x0 != 0x42 {
        if (*cur).x0 as i32 == arg0 && ((*cur).x2 & arg1) != 0 {
            if let Some(f) = (*cur).x4 {
                return f();
            }
        }
        cur = cur.add(1);
    }
    0
}

pub unsafe fn fn_8017280C() {
    if ext::gmMainLib_8015D94C(0x18) != 0 {
        ext::gmMainLib_8015EE54();
    } else {
        ext::gmMainLib_8015EE68();
    }
    if ext::gmMainLib_8015D94C(0x17) != 0 {
        ext::gmMainLib_8015EE1C();
    } else {
        ext::gmMainLib_8015EE30();
    }
    if ext::gmMainLib_8015D94C(0x16) != 0 {
        ext::gmMainLib_8015EDE4();
    } else {
        ext::gmMainLib_8015EDF8();
    }
    if ext::gmMainLib_8015D94C(0x1B) != 0 {
        ext::gmMainLib_8015EEA0();
        return;
    }
    ext::gmMainLib_8015EEB4();
}

pub unsafe fn gm_80172898(arg0: u16) -> i32 {
    let mut count = 0;

    // m-ex "Disable Trophy Messages" (opt-in MELEE_MEX=no_trophy_messages)
    if ext::Mex_Enabled(b"no_trophy_messages\0".as_ptr() as *const i8) != 0 {
        return 0;
    }

    for i in 0..0x42u32 {
        let var_r3 = if ext::gmMainLib_8015D94C(i) == 0 && fn_8017279C(i as i32, arg0) != 0 {
            ext::gmMainLib_8015D818(i)
        } else {
            0
        };
        if var_r3 != 0 {
            count += 1;
        }
    }
    if ext::Toy_803048C0(0xE6) > 0 && ext::gmMainLib_8015D94C(0x3F) == 0 {
        ext::gmMainLib_8015D818(0x3F);
    }
    if ext::Toy_803048C0(0xC9) > 0 && ext::gmMainLib_8015D94C(0x40) == 0 {
        ext::gmMainLib_8015D818(0x40);
    }
    fn_8017280C();
    count
}

pub unsafe fn gm_8017297C() {
    for i in 0..0x42u32 {
        if ext::gmMainLib_8015D94C(i) == 0 {
            let temp_r31 = ext::gmMainLib_8015D804(i as i32);
            *temp_r31 = ext::lbTime_GetTimeInSeconds();
            ext::gmMainLib_8015D8FC(i);
            ext::gmMainLib_8015D8B0(i);
        }
    }
}

pub unsafe fn inline3(i: i32) -> bool {
    let mut cur = lbl_803D6300.as_ptr();
    while (*cur).x0 != 0x42 {
        if (*cur).x0 as i32 == i && (*cur).x2 as i32 != 0 {
            return match (*cur).x4 {
                None => false,
                Some(f) => f() == 0,
            };
        }
        cur = cur.add(1);
    }
    false
}

pub unsafe fn gm_801729EC() {
    for i in 0..0x42i32 {
        if inline3(i) {
            let tmp = ext::gmMainLib_8015D804(i);
            *tmp = 0;
            ext::gmMainLib_8015D8B0(i as u32);
            ext::gmMainLib_8015D924(i as u32);
        }
    }

    for i in 0..11i32 {
        if ext::gm_IsCKindUnlocked(ext::gm_GetCKindByUnlockIndex(i)) == 0 {
            ext::gmMainLib_8015D924(ext::fn_80160710(i) as u32);
        }
    }

    for i in 0..11i32 {
        let temp_r30 = ext::fn_801607A8(i);
        let tmp = ext::gmMainLib_8015EDA4();
        if (*tmp as i32 & (1 << i)) == 0 {
            ext::gmMainLib_8015D924(ext::fn_801607F4(temp_r30 as i32) as u32);
        }
    }

    if ext::Toy_803048C0(0xE6) == 0 {
        let tmp = ext::gmMainLib_8015D804(0x3F);
        *tmp = 0;
        ext::gmMainLib_8015D8B0(0x3F);
        ext::gmMainLib_8015D924(0x3F);
    }
    if ext::Toy_803048C0(0xC9) == 0 {
        let tmp = ext::gmMainLib_8015D804(0x40);
        *tmp = 0;
        ext::gmMainLib_8015D8B0(0x40);
        ext::gmMainLib_8015D924(0x40);
    }
    if ext::Toy_803048C0(0x96) == 0 {
        let tmp = ext::gmMainLib_8015D804(0x1A);
        *tmp = 0;
        ext::gmMainLib_8015D8B0(0x1A);
        ext::gmMainLib_8015D924(0x1A);
    }
    fn_8017280C();
}

pub unsafe fn gm_80172BC4() -> i32 {
    if ext::gmMainLib_8015D94C(0x19) == 0 {
        ext::gmMainLib_8015D818(0x19);
        return 1;
    }
    0
}

pub unsafe fn gm_80172C04() -> i32 {
    if ext::gmMainLib_8015D94C(0x1A) == 0 {
        ext::gmMainLib_8015D818(0x1A);
        return 1;
    }
    0
}

pub unsafe fn gm_80172C44(arg0: u32) {
    ext::gmMainLib_8015DA68(arg0);
    ext::gmMainLib_8015D984(arg0);
}

pub unsafe fn fn_80172C78(arg0: i32) -> i32 {
    if ext::gmMainLib_8015D984(arg0 as u32) != 0 {
        ext::gmMainLib_8015DA68(arg0 as u32);
        return 1;
    }
    0
}

pub unsafe fn inline2(cpu_ckind: u8) -> *const lbl_803B7AD0_t {
    let temp_r3 = ext::gm_CKindToUnlockIndex(cpu_ckind as CharacterKind) as u8;
    let tmp = lbl_803B7AD0.as_ptr();
    for i in 0..0xB {
        if temp_r3 == (*tmp.add(i)).x0 {
            return tmp.add(i);
        }
    }
    core::ptr::null()
}

/// Returns the CPU level (0-9).
pub unsafe fn gm_DecideChallengerCpuLevel(cpu_ckind: u8, _human_nametag: u8) -> u8 {
    let var_r31 = inline2(cpu_ckind);

    let mut var_r0 = (*var_r31).x1 as i32
        - (*var_r31).x2 as i32
            * ext::gmMainLib_8015DB6C(ext::gm_CKindToUnlockIndex(cpu_ckind as CharacterKind) as u8) as i32;
    if var_r0 < 0 {
        var_r0 = 0;
    } else if var_r0 > 9 {
        var_r0 = 9;
    }
    var_r0 as u8
}

pub unsafe fn gm_80172D78() -> u8 {
    let temp_r31 = core::ptr::addr_of_mut!((*ext::gmMainLib_8015ED98()).x4);
    if ext::gm_IsCKindUnlocked(CKind_Mewtwo as u8) == 0 && *temp_r31 >= 0x11940 {
        return CKind_Mewtwo as u8;
    }
    ChKind_None as u8
}

pub unsafe fn inline1(arg0: u32) -> *const lbl_803B7AD0_t {
    let mut var_r29: *const lbl_803B7AD0_t = core::ptr::null();
    let mut var_r30: u16 = 0xFFFF;
    for i in 0..NUM_UNLOCKABLE_CHARACTERS as usize {
        if lbl_803B7AD0[i].x4 as u32 <= arg0
            && ext::gm_IsCKindUnlocked(ext::gm_GetCKindByUnlockIndex(lbl_803B7AD0[i].x0 as i32)) == 0
        {
            if lbl_803B7AD0[i].x4 < var_r30 {
                var_r30 = lbl_803B7AD0[i].x4;
                var_r29 = &lbl_803B7AD0[i];
            }
        }
    }
    var_r29
}

pub unsafe fn gm_80172DD4(arg0: u32) -> u8 {
    let var_r29 = inline1(arg0);
    if !var_r29.is_null() {
        return ext::gm_GetCKindByUnlockIndex((*var_r29).x0 as i32);
    }
    ChKind_None as u8
}

pub unsafe fn gm_80172E74() -> u8 {
    let mut count = 0;

    for i in 0..SELKIND_COUNT as u8 {
        if ext::gm_SelKindToUnlockIndex(i as SelectableCharacterKind) as i32 == NUM_UNLOCKABLE_CHARACTERS
            && ext::gmMainLib_8015CFCC(i) != 0
        {
            count += 1;
        }
    }
    if count >= 0xE && ext::gm_IsCKindUnlocked(CKind_Mars as u8) == 0 {
        return CKind_Mars as u8;
    }
    ChKind_None as u8
}

pub unsafe fn gm_80172F00(arg0: u32) -> u16 {
    if arg0 >= 0x32 && ext::gm_80164430(0xF) == 0 {
        return 0xF;
    }
    if arg0 >= 0x64 && ext::gm_80164430(0x12) == 0 {
        return 0x12;
    }
    if arg0 >= 0x96 && ext::gm_80164430(0x18) == 0 {
        return 0x18;
    }
    if arg0 >= 0xC8 && ext::gm_80164430(0x17) == 0 {
        return 0x17;
    }
    0x148
}

pub unsafe fn fn_80172FAC() -> i32 {
    let mut var_r31 = true;

    for j in 0..0x19u8 {
        if j != 3 && ext::gmMainLib_8015D4A8(j) == 0 {
            var_r31 = false;
            break;
        }
    }
    if var_r31 {
        return 1;
    }

    var_r31 = true;
    for i in 0..0x19u8 {
        if i != 3 && ext::gmMainLib_8015D0F4(i) == 0 {
            var_r31 = false;
            break;
        }
    }
    if var_r31 {
        return 1;
    }

    var_r31 = true;
    for i in 0..0x19u8 {
        if i != 3 && ext::gmMainLib_8015D21C(i) == 0 {
            var_r31 = false;
            break;
        }
    }
    var_r31 as i32
}

pub unsafe fn fn_80173098_CountUnlocked() -> i32 {
    let mut count = 0;
    for i in 0..0x19u8 {
        if ext::gmMainLib_8015D0F4(i) != 0 {
            count += 1;
        } else if ext::gmMainLib_8015D21C(i) != 0 {
            count += 1;
        } else if ext::gmMainLib_8015D344(i) != 0 {
            count += 1;
        }
    }
    count
}

pub unsafe fn fn_80173098(arg0: i32) -> u8 {
    let temp_r3 = ext::fn_8017DEC8(arg0);
    if (*temp_r3).xC.xD == 0 {
        if (*temp_r3).x0.ckind as i32 == CKind_Mars as i32 && ext::gm_IsCKindUnlocked(CKind_Emblem as u8) == 0 {
            return CKind_Emblem as u8;
        }
        if (*temp_r3).x0.ckind as i32 == CKind_Mario as i32 && ext::gm_IsCKindUnlocked(CKind_DrMario as u8) == 0 {
            return CKind_DrMario as u8;
        }
    }
    let unlocked_chars_count = fn_80173098_CountUnlocked();
    if unlocked_chars_count >= 10 && ext::gm_IsCKindUnlocked(CKind_CLink as u8) == 0 {
        return CKind_CLink as u8;
    }
    if fn_80172FAC() != 0 && ext::gm_IsCKindUnlocked(CKind_GameWatch as u8) == 0 {
        return CKind_GameWatch as u8;
    }
    if arg0 == 0 {
        let temp_r31 = ext::gm_GetAdventureData();
        if ext::gm_IsCKindUnlocked(CKind_Luigi as u8) == 0 && (*temp_r31).x74 != 0 && (*temp_r31).x75 != 0 {
            return CKind_Luigi as u8;
        }
    }
    if ext::gm_IsCKindUnlocked(CKind_Purin as u8) == 0 {
        return CKind_Purin as u8;
    }
    ChKind_None as u8
}

pub unsafe fn gm_80173224(arg0: i32, arg1: i32) -> u8 {
    let mut ckind = ChKind_None as u8;
    if arg1 != 0 {
        ckind = fn_80173098(arg0);
    }
    if ckind == ChKind_None as u8 {
        ckind = gm_80172E74();
    }
    ckind
}

/// check for event character unlocks?
pub unsafe fn gm_801732D8(arg0: u8) -> u8 {
    if ext::gm_IsCKindUnlocked(CKind_Ganon as u8) == 0 && ext::gm_801BEBC0(arg0) == 0x1C {
        return CKind_Ganon as u8;
    }
    if ext::gm_IsCKindUnlocked(CKind_Pichu as u8) == 0 && ext::gm_801BEBC0(arg0) == 0xE {
        return CKind_Pichu as u8;
    }
    ChKind_None as u8
}

pub unsafe fn gm_8017335C() -> u16 {
    let mut var_r31 = 1;
    for i in 0..0x33 {
        if ext::gmMainLib_8015CEFC(i) == 0 {
            var_r31 = 0;
            break;
        }
    }
    if var_r31 == 1 && ext::gm_80164430(0x20) == 0 {
        return 0x20;
    }
    0x148
}

pub unsafe fn gm_801733D8() -> u8 {
    if ext::gm_IsCKindUnlocked(CKind_GameWatch as u8) == 0 && fn_80172FAC() != 0 {
        return CKind_GameWatch as u8;
    }
    ChKind_None as u8
}

pub unsafe fn gm_8017341C() -> u16 {
    if ext::gmMainLib_8015D5DC() != 0 && ext::gm_80164430(0x1C) == 0 {
        return 0x1C;
    }
    0x148
}

pub unsafe fn gm_80173460(_arg0: i8) -> u8 {
    if ext::gm_IsCKindUnlocked(CKind_Falco as u8) == 0 {
        return CKind_Falco as u8;
    }
    ChKind_None as u8
}

pub unsafe fn gm_80173498() -> u16 {
    if ext::gm_80164430(0x1E) == 0 {
        return 0x1E;
    }
    0x148
}

pub unsafe fn gm_801734D0(arg0: u32) -> u16 {
    if arg0 >= 0x9C40 && ext::gm_80164430(0x1D) == 0 {
        return 0x1D;
    }
    0x148
}

pub unsafe fn fn_80173510() -> i32 {
    let temp_r31 = ext::gmMainLib_8015EDA4();
    let mut var_r30 = true;

    if (*ext::gmMainLib_8015EDC8()).x5 == 0 {
        for i in 0..11 {
            if i != 6 && i != 7 && i != 8 && i != 9 && i != 10 && (*temp_r31 as i64 & (1i64 << i)) == 0 {
                var_r30 = false;
                break;
            }
        }
        if var_r30 {
            (*ext::gmMainLib_8015EDC8()).x5 = 1;
        }
    }
    (*ext::gmMainLib_8015EDC8()).x5 as i32
}

pub unsafe fn fn_801735F0() -> i32 {
    if (*ext::gmMainLib_8015EDC8()).x4 == 0 && ext::gm_80164ABC() != 0 {
        (*ext::gmMainLib_8015EDC8()).x4 = 1;
    }
    (*ext::gmMainLib_8015EDC8()).x4 as i32
}

pub unsafe fn fn_80173644() -> i32 {
    ((*ext::gmMainLib_8015EDC8()).x0 >= 0x1388) as i32
}

pub unsafe fn fn_8017367C() -> i32 {
    if (*ext::gmMainLib_8015EDC8()).x6 == 0 && ext::gm_80164ABC() != 0 && ext::gm_80164600() != 0 {
        (*ext::gmMainLib_8015EDC8()).x6 = 1;
    }
    (*ext::gmMainLib_8015EDC8()).x6 as i32
}
