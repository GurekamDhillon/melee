
//! Game-mode (`src/melee/gm/`) leaf functions: no calls, no globals. Each is a hand port of the
//! named C function (same name), checked bit for bit by `tests/oracle.rs` against the C cut
//! verbatim from its TU and compiled as the shipping build compiles it (contraction on).
#![allow(non_snake_case)]

use melee_core::fp::fmadds;

use melee_runtime::cabi;

use crate::GXColor;

// ---- enum values (forward.h; read from the C with libclang, TARGET_PC) ----
pub(crate) const OUTCOME_NO_CONTEST: u8 = 7;

pub(crate) const OUTCOME_RETRY: u8 = 8;

pub(crate) const CHKIND_MEX0: i32 = 34;

pub(crate) const CHKIND_CAP: i32 = 128;

pub(crate) const CKIND_KOOPA: i32 = 5;

pub(crate) const CKIND_SEAK: i32 = 19;

pub(crate) const CKIND_GKOOPS: i32 = 29;

pub(crate) const FT_KIND_MASTERH: i32 = 27;

pub(crate) const FT_KIND_CREZYH: i32 = 28;

pub(crate) const GM_PKIND_HUMAN: u8 = 0;

pub(crate) fn clamp_sym(v: i32, lim: i32) -> i32 {
    if v > lim {
        lim
    } else if v < -lim {
        -lim
    } else {
        v
    }
}

// ---- Round 2: callers of round-1 functions and readers of gm_1601.static.h's const tables ----

pub(crate) const NUM_UNLOCKABLE_CHARACTERS: usize = 11;

pub(crate) const NUM_UNLOCKABLE_STAGES: usize = 11;

pub(crate) const SELKIND_COUNT: u8 = 0x19;

pub(crate) const ST_KIND_UNK328: i32 = 328;

pub(crate) const CHKIND_POPO: usize = 0x20;

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

pub(crate) const fn ucd(idx: u8, selkind: u8, notification_id: u8, stkind: u16) -> UnlockableCharacterData {
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

pub use crate::tu::gm_17c0::BossHook_Kind_fp as BossHook_Kind;
pub use crate::tu::gmmain_lib::bitset64_mask;
pub use crate::tu::gmmain_lib::bitset_mask;
pub use crate::tu::gmvs::direction_fp as direction;
pub use crate::tu::gmfrontend::fe_ease;
pub use crate::tu::gmfrontend::fe_np_zero;
pub use crate::tu::gmfrontend::fe_rgba;
pub use crate::tu::gmfrontend::fe_row_y;
pub use crate::tu::gm_1601::fn_80160710_fp as fn_80160710;
pub use crate::tu::gm_1601::fn_801607A8_fp as fn_801607A8;
pub use crate::tu::gm_1601::fn_801607F4_fp as fn_801607F4;
pub use crate::tu::gm_1601::fn_80160840;
pub use crate::tu::gm_1601::fn_8016419C;
pub use crate::tu::gm_1601::fn_801641B4;
pub use crate::tu::gm_1601::fn_801653E8;
pub use crate::tu::gm_1601::fn_8016588C_clamp;
pub use crate::tu::gm_16f1::fn_801724C8_fp as fn_801724C8;
pub use crate::tu::gm_17ad::fn_8017AD04_fp as fn_8017AD04;
pub use crate::tu::gm_17ad::fn_8017AD28_fp as fn_8017AD28;
pub use crate::tu::gm_17ad::fn_8017AD48_fp as fn_8017AD48;
pub use crate::tu::gm_17ad::fn_8017AD78_fp as fn_8017AD78;
pub use crate::tu::gmtoulib::fn_8018F3D0;
pub use crate::tu::gmtoulib::fn_8018F6DC;
pub use crate::tu::gmtoulib::fn_8018F6FC;
pub use crate::tu::gmtoulib::fn_8018F71C;
pub use crate::tu::gmtoulib::fn_8018FA24_inline0_fp as fn_8018FA24_inline0;
pub use crate::tu::gmtou_0::fn_801953C8_GetPreviousPosition_fp as fn_801953C8_GetPreviousPosition;
pub use crate::tu::gm_1601::getStageUnlockIndex;
pub use crate::tu::gm_1601::getStageUnlockNotifyId;
pub use crate::tu::gm_1601::get_unlockable_selkind_by_bit_index;
pub use crate::tu::gmresultplayer::gmRst_IsMex;
pub use crate::tu::gm_1601::gm_801601C4_fp as gm_801601C4;
pub use crate::tu::gm_1601::gm_801601C4_inner;
pub use crate::tu::gm_1601::gm_801601C4_noinline;
pub use crate::tu::gm_1601::gm_80160244_fp as gm_80160244;
pub use crate::tu::gm_1601::gm_80160244_inner;
pub use crate::tu::gm_1601::gm_80160244_noinline;
pub use crate::tu::gm_1601::gm_80160854_fp as gm_80160854;
pub use crate::tu::gm_1601::gm_80160968_fp as gm_80160968;
pub use crate::tu::gm_1601::gm_801641CC_fp as gm_801641CC;
pub use crate::tu::gmregcommon::gm_8017BE84_fp as gm_8017BE84;
pub use crate::tu::gmtoulib::gm_801905F0_inline0_fp as gm_801905F0_inline0;
pub use crate::tu::gm_1601::gm_CKindToSelKind_fp as gm_CKindToSelKind;
pub use crate::tu::gm_1601::gm_CKindToUnlockIndex_fp as gm_CKindToUnlockIndex;
pub use crate::tu::gm_1601::gm_GetCKindByUnlockIndex_fp as gm_GetCKindByUnlockIndex;
pub use crate::tu::gm_1601::gm_GetChallengerStKind_fp as gm_GetChallengerStKind;
pub use crate::tu::gmevent::gm_GetNextColor_fp as gm_GetNextColor;
pub use crate::tu::gm_1A3F::gm_Is1PMode;
pub use crate::tu::gm_1601::gm_SelKindToCKind_fp as gm_SelKindToCKind;
pub use crate::tu::gm_1601::gm_SelKindToUnlockIndex_fp as gm_SelKindToUnlockIndex;
pub use crate::tu::gmresult::gm_WasMatchCanceled_fp as gm_WasMatchCanceled;
pub use crate::tu::gmmain_lib::selkind_bit_fp as selkind_bit;
pub use crate::tu::gmmain_lib::selkind_mask;
