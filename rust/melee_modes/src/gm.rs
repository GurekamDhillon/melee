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

/// gmevent.c `gm_GetNextColor`.
pub fn gm_GetNextColor(color: u8) -> u8 {
    if color <= 2 {
        color + 1
    } else {
        0
    }
}

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

/// gmtoulib.c `fn_8018FA24_inline0`.
pub fn fn_8018FA24_inline0(char_kind: i32) -> i32 {
    if char_kind < CKIND_SEAK {
        return char_kind;
    }
    if char_kind == CKIND_GKOOPS {
        return 5;
    }
    char_kind.wrapping_add(1)
}

/// gmtoulib.c `gm_801905F0_inline0`.
pub fn gm_801905F0_inline0(c_kind: i32) -> i32 {
    if c_kind < CKIND_SEAK {
        return c_kind;
    }
    if c_kind == CKIND_GKOOPS {
        return CKIND_KOOPA;
    }
    c_kind.wrapping_add(1)
}

/// gmtou_0.c `fn_801953C8_GetPreviousPosition`.
pub fn fn_801953C8_GetPreviousPosition(offset: i32, position: i32) -> i32 {
    position.wrapping_sub(offset)
}

/// gmvs.c `direction` (NaN -> +1).
pub fn direction(x: f32) -> f32 {
    if x >= 0.0 {
        -1.0
    } else {
        1.0
    }
}

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

/// gm_17C0.c `BossHook_Kind` (TARGET_PC): the HP bosses driven by the 1P controller.
pub fn BossHook_Kind(kind: i32) -> i32 {
    (kind == FT_KIND_MASTERH || kind == FT_KIND_CREZYH) as i32
}

/// gm_1A3F.c `gm_Is1PMode`.
pub fn gm_Is1PMode(mode: u8) -> bool {
    // GM_CLASSIC 3, ADVENTURE 4, ALLSTAR 5, TARGET_TEST 15, TRAINING 28, HOME_RUN 32,
    // 10MAN 33, 100MAN 34, 3MIN 35, 15MIN 36, ENDLESS 37, CRUEL 38, EVENT 43
    matches!(mode, 3 | 4 | 5 | 15 | 28 | 32..=38 | 43)
}
