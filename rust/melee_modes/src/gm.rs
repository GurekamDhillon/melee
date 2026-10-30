//! Game-mode (`src/melee/gm/`) leaf functions: no calls, no globals. Each is a hand port of the
//! named C function (same name), checked bit for bit by `tests/oracle.rs` against the C cut
//! verbatim from its TU and compiled as the shipping build compiles it (contraction on).
#![allow(non_snake_case)]

use melee_core::fp::fmadds;
use melee_runtime::cabi;

use crate::GXColor;

// ---- enum values (forward.h; read from the C with libclang, TARGET_PC) ----
const OUTCOME_NO_CONTEST: u8 = 7;
const OUTCOME_RETRY: u8 = 8;
const CHKIND_MEX0: i32 = 34;
const CHKIND_CAP: i32 = 128;
const CKIND_KOOPA: i32 = 5;
const CKIND_SEAK: i32 = 19;
const CKIND_GKOOPS: i32 = 29;
const FT_KIND_MASTERH: i32 = 27;
const FT_KIND_CREZYH: i32 = 28;
const GM_PKIND_HUMAN: u8 = 0;

pub use crate::tu::gmevent::gm_GetNextColor;

/// gmmain_lib.c `bitset_mask`.
pub fn bitset_mask(bit: u32) -> u32 {
    1u32 << (bit % 32)
}

/// gmmain_lib.c `bitset64_mask`: `1LL << bit`. Verified for bit < 64; a larger shift is
/// undefined in C (gwtool's 64-bit shift rule is not modelled by the oracle): 0 here.
pub fn bitset64_mask(bit: u32) -> i64 {
    1i64.checked_shl(bit).unwrap_or(0)
}

/// gmmain_lib.c `selkind_mask`: `1 << kind` (kind >= 32 uses gwtool's oversize-shift rule).
pub fn selkind_mask(kind: u8) -> i32 {
    cabi::shl(1, kind as u32) as i32
}

/// gmfrontend.c `fe_np_zero`.
pub fn fe_np_zero() -> i32 {
    0
}

/// gmfrontend.c `fe_rgba`.
pub fn fe_rgba(r: u8, g: u8, b: u8, a: u8) -> GXColor {
    GXColor { r, g, b, a }
}

/// gmfrontend.c `fe_ease`: `1 - u*u*u` is one fused site (fmuladd(-(u*u), u, 1)).
pub fn fe_ease(t: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let u = 1.0f32 - t;
    fmadds(-(u * u), u, 1.0)
}

/// gmfrontend.c `fe_row_y`: `FE_ROW_Y + FE_ROW_STEP * slot`, fused.
pub fn fe_row_y(slot: i32) -> f32 {
    fmadds(34.0, slot as f32, 114.0)
}

/// gmregcommon.c `gm_8017BE84`.
pub fn gm_8017BE84(arg0: u32) -> i32 {
    ((arg0 >> 3) & 0x1F) as i32
}

/// gmresult.c `gm_WasMatchCanceled`.
pub fn gm_WasMatchCanceled(outcome: u8) -> bool {
    outcome == OUTCOME_NO_CONTEST || outcome == OUTCOME_RETRY
}

/// gmresultplayer.c `gmRst_IsMex` (TARGET_PC: an m-ex character kind).
pub fn gmRst_IsMex(ck: i32) -> bool {
    ck >= CHKIND_MEX0 && ck < CHKIND_CAP
}

/// gmtoulib.c `fn_8018F3D0`.
pub fn fn_8018F3D0(arg0: i32) -> i32 {
    if arg0 == 0xE || (0x10..=0x13).contains(&arg0) || arg0 == 0xA {
        return 1;
    }
    if arg0 <= 0xD {
        return 0;
    }
    2
}

/// gmtoulib.c `fn_8018F6DC`.
pub fn fn_8018F6DC(arg0: i32) -> i32 {
    if arg0 >= 0x13 {
        if arg0 == 0x1D {
            return 5;
        }
        return arg0.wrapping_sub(1);
    }
    arg0
}

/// gmtoulib.c `fn_8018F6FC` (CSSIconHud -> CharacterKind).
pub fn fn_8018F6FC(arg0: i32) -> i32 {
    if arg0 >= 0x13 {
        if arg0 == 0x1D {
            return 5;
        }
        return arg0.wrapping_add(1);
    }
    arg0
}

/// gmtoulib.c `fn_8018F71C`: int arithmetic (-fwrapv), then one int->float conversion.
pub fn fn_8018F71C(arg0: i32, arg1: i32) -> f32 {
    arg0.wrapping_add(arg1.wrapping_mul(0x1E)) as f32
}

pub use crate::tu::gmtoulib::fn_8018FA24_inline0;

pub use crate::tu::gmtoulib::gm_801905F0_inline0;

pub fn fn_801953C8_GetPreviousPosition(offset: i32, position: i32) -> i32 {
    unsafe { crate::tu::gmtou_0::fn_801953C8_GetPreviousPosition(offset, position) }
}

pub use crate::tu::gmvs::direction;

/// gm_1601.c `gm_801601C4` (a pan/volume curve).
pub fn gm_801601C4(arg0: i8) -> i32 {
    let mut t = (arg0 as i32 + 100) as f32 / 200.0;
    t *= 100.0;
    if t == 50.0 {
        return 0x7F;
    }
    if t > 50.0 {
        return cabi::f2i32((127.0f32 * ((100.0f32 - t) / 50.0)) as f64);
    }
    0x7F
}

/// gm_1601.c `gm_80160244`.
pub fn gm_80160244(arg0: i8) -> i32 {
    let mut t = (arg0 as i32 + 100) as f32 / 200.0;
    t *= 100.0;
    if t == 50.0 || t > 50.0 {
        return 0x7F;
    }
    cabi::f2i32((127.0f32 * (t / 50.0)) as f64)
}

/// gm_1601.c `fn_80160840`.
pub fn fn_80160840(arg0: u8) -> u8 {
    if arg0 >= 4 {
        4
    } else {
        arg0
    }
}

/// gm_1601.c `gm_80160854`: the HUD colour index for a slot.
pub fn gm_80160854(slot: u8, team: u8, is_teams: u8, slot_type: u8) -> u32 {
    let human = slot_type == GM_PKIND_HUMAN;
    if is_teams != 0 {
        match team {
            0 => return if human { 0 } else { 5 },
            1 => return if human { 1 } else { 6 },
            2 => return if human { 3 } else { 8 },
            3 => return if human { 2 } else { 7 },
            4 => return 4,
            _ => {}
        }
    } else {
        if !human {
            return 4;
        }
        if slot <= 3 {
            return slot as u32;
        }
    }
    0
}

/// gm_1601.c `fn_801653E8`.
pub fn fn_801653E8(arg0: u8) -> f32 {
    0.1f32 * arg0 as f32
}

/// gm_1601.c `fn_8016588C_clamp`.
pub fn fn_8016588C_clamp(v: i32) -> i32 {
    let lim: i32 = (1 << 24) - 1;
    if v > lim {
        lim
    } else if v < -lim {
        -lim
    } else {
        v
    }
}

/// gm_16F1.c `fn_801724C8`.
pub fn fn_801724C8() -> bool {
    false
}

/// gm_17AD.c `fn_8017AD04`: clamp to [-arg1, arg1] (upper bound tested first).
pub fn fn_8017AD04(arg0: i32, arg1: i32) -> i32 {
    if arg0 > arg1 {
        return arg1;
    }
    let neg = arg1.wrapping_neg();
    if arg0 < neg {
        return neg;
    }
    arg0
}

fn clamp_sym(v: i32, lim: i32) -> i32 {
    if v > lim {
        lim
    } else if v < -lim {
        -lim
    } else {
        v
    }
}

/// gm_17AD.c `fn_8017AD28`.
pub fn fn_8017AD28(arg0: i32) -> i32 {
    clamp_sym(arg0, 9999)
}

/// gm_17AD.c `fn_8017AD48`.
pub fn fn_8017AD48(arg0: i32) -> i32 {
    clamp_sym(arg0, 9999999)
}

/// gm_17AD.c `fn_8017AD78`.
pub fn fn_8017AD78(arg0: i32) -> i32 {
    clamp_sym(arg0, 99999)
}

pub fn BossHook_Kind(kind: i32) -> i32 {
    unsafe { crate::tu::gm_17c0::BossHook_Kind(kind) }
}

/// gm_1A3F.c `gm_Is1PMode`.
pub fn gm_Is1PMode(mode: u8) -> bool {
    // GM_CLASSIC 3, ADVENTURE 4, ALLSTAR 5, TARGET_TEST 15, TRAINING 28, HOME_RUN 32,
    // 10MAN 33, 100MAN 34, 3MIN 35, 15MIN 36, ENDLESS 37, CRUEL 38, EVENT 43
    matches!(mode, 3 | 4 | 5 | 15 | 28 | 32..=38 | 43)
}

// ---- Round 2: callers of round-1 functions and readers of gm_1601.static.h's const tables ----

const NUM_UNLOCKABLE_CHARACTERS: usize = 11;
const NUM_UNLOCKABLE_STAGES: usize = 11;
const SELKIND_COUNT: u8 = 0x19;
const ST_KIND_UNK328: i32 = 328;
const CHKIND_POPO: usize = 0x20;

/// gm_1601.static.h `lbl_803B7808` (s16[30]).
pub static LBL_803B7808: [i16; 30] = [
    0x04, 0x05, 0x0D, 0x06, 0x08, 0x02, 0x07, 0x03, 0x0A, 0x09, 0x19, 0x13, 0x0B, 0x0C, 0x0E, 0x0F,
    0x10, 0x11, 0x16, 0x17, 0x18, 0x12, 0x14, 0x1B, 0x1F, 0x20, 0x1C, 0x1D, 0x1E, 0x00,
];

/// gm_1601.static.h `lbl_803B7864` (GXColor[9]).
pub static LBL_803B7864: [GXColor; 9] = [
    GXColor { r: 0x99, g: 0x1A, b: 0x1A, a: 0xFF },
    GXColor { r: 0x33, g: 0x33, b: 0x80, a: 0xFF },
    GXColor { r: 0x80, g: 0x66, b: 0x00, a: 0xFF },
    GXColor { r: 0x1A, g: 0x66, b: 0x1A, a: 0xFF },
    GXColor { r: 0x66, g: 0x66, b: 0x66, a: 0xFF },
    GXColor { r: 0x66, g: 0x4D, b: 0x4D, a: 0xFF },
    GXColor { r: 0x4D, g: 0x4D, b: 0x66, a: 0xFF },
    GXColor { r: 0x66, g: 0x59, b: 0x33, a: 0xFF },
    GXColor { r: 0x4D, g: 0x66, b: 0x4D, a: 0xFF },
];

/// gm_1601.static.h `selkind_to_ckind_map` (u8[0x1C]; the last three are zero-filled).
pub static SELKIND_TO_CKIND_MAP: [u8; 0x1C] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x10, 0x11, 0x12, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0, 0, 0,
];

/// gm_1601.static.h `ckind_to_selkind_map` (u8[ChKind_Max = 0x21]). Its comments name the
/// mn/types.h selkinds, where FALCO is 0x13 (Zelda and Sheik share 0x12).
pub static CKIND_TO_SELKIND_MAP: [u8; 0x21] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x10, 0x11, 0x12, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x08, 0x08, 0x08, SELKIND_COUNT,
    0x08, 0x08, 0x00,
];

/// gm_1601.static.h `struct UnlockableCharacterData` (size 6 on the GameCube).
#[derive(Clone, Copy, Debug)]
pub struct UnlockableCharacterData {
    pub idx: u8,
    pub selkind: u8,
    pub notification_id: u8,
    pub x3: u8,
    pub stkind: u16,
}

melee_core::gc_layout!(UnlockableCharacterData, "UnlockableCharacterData", size 6, align 2, [
    idx @ 0x0: 1 "u8",
    selkind @ 0x1: 1 "u8",
    notification_id @ 0x2: 1 "u8",
    x3 @ 0x3: 1 "u8",
    stkind @ 0x4: 2 "u16",
]);

const fn ucd(idx: u8, selkind: u8, notification_id: u8, stkind: u16) -> UnlockableCharacterData {
    UnlockableCharacterData { idx, selkind, notification_id, x3: 0, stkind }
}

/// gm_1601.static.h `lbl_803B78C8`.
pub static LBL_803B78C8: [UnlockableCharacterData; NUM_UNLOCKABLE_CHARACTERS] = [
    ucd(0, 0x03, 4, 0x116),
    ucd(1, 0x07, 2, 0x114),
    ucd(2, 0x09, 3, 0x115),
    ucd(3, 0x0A, 1, 0x113),
    ucd(4, 0x0F, 0, 0x112),
    ucd(5, 0x13, 8, 0x11A),
    ucd(6, 0x14, 7, 0x119),
    ucd(7, 0x15, 5, 0x117),
    ucd(8, 0x16, 0xA, 0x11C),
    ucd(9, 0x17, 9, 0x11B),
    ucd(0xA, 0x18, 6, 0x118),
];

/// gm_1601.static.h `lbl_803B790C` (stage unlock index, grkind, notification id).
pub static LBL_803B790C: [[u8; 3]; 0xB] = [
    [0x00, 0x09, 0x0B],
    [0x01, 0x11, 0x0D],
    [0x02, 0x13, 0x0E],
    [0x03, 0x15, 0x0C],
    [0x04, 0x19, 0x0F],
    [0x05, 0x1B, 0x10],
    [0x06, 0x24, 0x11],
    [0x07, 0x25, 0x12],
    [0x08, 0x1C, 0x13],
    [0x09, 0x1D, 0x14],
    [0x0A, 0x1E, 0x15],
];

/// gm_1601.static.h `lbl_803B7930` (Vec2[9], handicap damage ratios: the f32 nearest each
/// decimal literal, as the C initialiser rounds them).
pub static LBL_803B7930: [[f32; 2]; 9] = [
    [0.35, 2.857143],
    [0.4, 2.5],
    [0.46, 2.173913],
    [0.53, 1.8867925],
    [0.61, 1.6393442],
    [0.7, 1.4285715],
    [0.8, 1.25],
    [0.93, 1.0752689],
    [1.07, 0.9345794],
];

/// gm_1601.c `gm_801601C4_inner`.
pub fn gm_801601C4_inner(arg0: i8) -> i32 {
    gm_801601C4(arg0)
}

/// gm_1601.c `gm_801601C4_noinline`.
pub fn gm_801601C4_noinline(arg0: i8) -> i32 {
    gm_801601C4_inner(arg0)
}

/// gm_1601.c `gm_80160244_inner`.
pub fn gm_80160244_inner(arg0: i8) -> i32 {
    gm_80160244(arg0)
}

/// gm_1601.c `gm_80160244_noinline`.
pub fn gm_80160244_noinline(arg0: i8) -> i32 {
    gm_80160244_inner(arg0)
}

/// gm_1601.c `gm_SelKindToUnlockIndex` (the selkind enum is an int compared to the u8 field).
pub fn gm_SelKindToUnlockIndex(selkind: i32) -> u8 {
    for e in &LBL_803B78C8 {
        if selkind == e.selkind as i32 {
            return e.idx;
        }
    }
    NUM_UNLOCKABLE_CHARACTERS as u8
}

/// gm_1601.c `gm_CKindToUnlockIndex` (`ckind` truncates to u8 at the gm_CKindToSelKind call).
pub fn gm_CKindToUnlockIndex(ckind: i32) -> i8 {
    gm_SelKindToUnlockIndex(gm_CKindToSelKind(ckind as u8) as i32) as i8
}

/// gm_1601.c `get_unlockable_selkind_by_bit_index`.
pub fn get_unlockable_selkind_by_bit_index(unlock_bit_index: i32) -> u8 {
    for e in &LBL_803B78C8 {
        if unlock_bit_index == e.idx as i32 {
            return e.selkind;
        }
    }
    SELKIND_COUNT
}

/// gm_1601.c `gm_GetCKindByUnlockIndex`. An index with no entry gives SELKIND_COUNT (0x19),
/// which is inside selkind_to_ckind_map.
pub fn gm_GetCKindByUnlockIndex(index: i32) -> u8 {
    gm_SelKindToCKind(get_unlockable_selkind_by_bit_index(index))
}

/// gm_1601.c `fn_80160710`: the notification id for an unlock index (0x42 if none).
pub fn fn_80160710(arg0: i32) -> u8 {
    for e in &LBL_803B78C8 {
        if arg0 == e.idx as i32 {
            return e.notification_id;
        }
    }
    0x42
}

/// gm_1601.c `gm_GetChallengerStKind`.
pub fn gm_GetChallengerStKind(selkind: i32) -> i32 {
    for e in &LBL_803B78C8 {
        if selkind == e.selkind as i32 {
            return e.stkind as i32;
        }
    }
    ST_KIND_UNK328
}

/// gm_1601.c `fn_801607A8`.
pub fn fn_801607A8(arg0: i32) -> u8 {
    for e in &LBL_803B790C {
        if arg0 == e[0] as i32 {
            return e[1];
        }
    }
    0x6F
}

/// gm_1601.c `fn_801607F4`.
pub fn fn_801607F4(arg0: i32) -> u8 {
    for e in &LBL_803B790C {
        if arg0 == e[1] as i32 {
            return e[2];
        }
    }
    0x42
}

/// gm_1601.c `gm_80160968` (index `(u8) arg0`; above 8 the C reads past the table, this panics).
pub fn gm_80160968(arg0: u32) -> GXColor {
    LBL_803B7864[arg0 as u8 as usize]
}

/// gm_1601.c `gm_SelKindToCKind` (selkind < 0x1C).
pub fn gm_SelKindToCKind(selkind: u8) -> u8 {
    SELKIND_TO_CKIND_MAP[selkind as usize]
}

/// gm_1601.c `gm_CKindToSelKind` (TARGET_PC `GM_CK_TO_SELKIND`: an m-ex kind reads Popo's row).
/// ckind 0x21 and >= 0x80 read past the C table; this panics there.
pub fn gm_CKindToSelKind(ckind: u8) -> u8 {
    let ck = ckind as i32;
    if ck >= CHKIND_MEX0 && ck < CHKIND_CAP {
        CKIND_TO_SELKIND_MAP[CHKIND_POPO]
    } else {
        CKIND_TO_SELKIND_MAP[ckind as usize]
    }
}

/// gm_1601.c `fn_8016419C` (handicap 1..9).
pub fn fn_8016419C(arg0: u8) -> f32 {
    LBL_803B7930[(arg0 as i32 - 1) as usize][0]
}

/// gm_1601.c `fn_801641B4` (handicap 1..9).
pub fn fn_801641B4(arg0: u8) -> f32 {
    LBL_803B7930[(arg0 as i32 - 1) as usize][1]
}

/// gm_1601.c `gm_801641CC` (the s16 table entry as u16; arg0 < 30).
pub fn gm_801641CC(arg0: u8) -> u16 {
    LBL_803B7808[arg0 as usize] as u16
}

/// gm_1601.c `getStageUnlockIndex`.
pub fn getStageUnlockIndex(grkind: u8) -> u8 {
    for e in LBL_803B790C.iter().take(NUM_UNLOCKABLE_STAGES) {
        if grkind as i32 == e[1] as i32 {
            return e[0];
        }
    }
    NUM_UNLOCKABLE_STAGES as u8
}

/// gm_1601.c `getStageUnlockNotifyId`.
pub fn getStageUnlockNotifyId(unlock_idx: u8) -> u8 {
    for e in LBL_803B790C.iter().take(NUM_UNLOCKABLE_STAGES) {
        if unlock_idx as i32 == e[0] as i32 {
            return e[2];
        }
    }
    0x42
}

/// gmmain_lib.c `selkind_bit`.
pub fn selkind_bit(word: i32, kind: u8) -> i32 {
    word & selkind_mask(kind)
}
