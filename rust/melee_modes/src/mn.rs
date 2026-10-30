//! Menu (`src/melee/mn/`) leaf functions: no calls, no globals. Hand ports, same names as the
//! C, checked by `tests/oracle.rs`.
#![allow(non_snake_case)]

use melee_runtime::cabi;

/// sizeof(CSSIcon) on the GameCube (libclang ppc32: 0x1C).
pub const CSS_ICON_SIZE: u32 = 0x1C;

/// mncharsel.c `toMeters`.
pub fn toMeters(cm: f32) -> f32 {
    cabi::f2i32((10.0f32 * (cm / 100.0)) as f64) as f32 / 10.0
}

/// mncharsel.c `toFeet`.
pub fn toFeet(cm: f32) -> f32 {
    cabi::f2i32((10.0f32 * (cm / 30.4788)) as f64) as f32 / 10.0
}

/// mncharsel.c `equalU8`.
pub fn equalU8(lhs: u8, rhs: u8) -> bool {
    lhs == rhs
}

/// mncharsel.c `getIconOffset`.
pub fn getIconOffset(icon_idx: u32) -> i32 {
    icon_idx.wrapping_mul(CSS_ICON_SIZE) as i32
}

pub use crate::tu::mncount::inline_is_row_time;

pub use crate::tu::mncount::inline_is_row_char;

/// mndiagram2.c `mnDiagram2_IsTimeStat` (VSSTAT_PLAY_TIME = 11).
pub fn mnDiagram2_IsTimeStat(stat_type: u8) -> bool {
    stat_type == 11
}

/// mndiagram2.c `mnDiagram2_IsDistanceStat` (WALK/RUN/FALL_DISTANCE, PEAK_HEIGHT = 14..17).
pub fn mnDiagram2_IsDistanceStat(stat_type: u8) -> bool {
    (14..=17).contains(&stat_type)
}

/// mndiagram2.c `mnDiagram2_IsPercentageStat` (HIT_PERCENTAGE 3, PLAY_PERCENTAGE 12, AVG_PLAYERS 13).
pub fn mnDiagram2_IsPercentageStat(stat_type: u8) -> bool {
    matches!(stat_type, 3 | 12 | 13)
}

/// mndiagram2.c `mnDiagram2_IsIconOnlyStat` (MOST/SECOND/LEAST_PLAYED = 21..23).
pub fn mnDiagram2_IsIconOnlyStat(stat_type: u8) -> bool {
    (21..=23).contains(&stat_type)
}

/// mnmain.c `GetSelectionFrameOffset`.
pub fn GetSelectionFrameOffset(selection: i32) -> i32 {
    selection.wrapping_mul(2)
}

/// mnmain.c `mn_8022C010`: which main-menu entry a menu kind belongs to.
/// MENU_KIND_34 and up fall off the end of the C function (no return value; undefined); this
/// returns 0 there, which is not verified against anything.
pub fn mn_8022C010(menu_kind: i32, selection: i32) -> i32 {
    match menu_kind {
        0 => selection,
        1 | 6 | 7 | 8 | 9 | 10 | 33 => 0,
        2 | 11..=18 => 1,
        3 => 2,
        4 | 19..=24 => 3,
        5 | 25..=32 => 4,
        _ => 0, // TODO(group6): UB in the C; check the shipping build's actual return register.
    }
}

/// mnname.c `signedCharactersEqual`.
pub fn signedCharactersEqual(lhs: i8, rhs: u8) -> bool {
    lhs == rhs as i8
}

/// mnname.c `unsignedCharacter`.
pub fn unsignedCharacter(character: i32) -> u8 {
    character as u8
}

/// mnvibration.c `mnVibration_GetPreviousCursorRow`.
pub fn mnVibration_GetPreviousCursorRow(cursor_row: i32) -> i32 {
    cursor_row.wrapping_sub(1)
}

/// mnvibration.c `mnVibration_GetNextCursorRow`.
pub fn mnVibration_GetNextCursorRow(cursor_row: i32) -> i32 {
    cursor_row.wrapping_add(1)
}

/// mnvibration.c `mnVibration_GetPadIndex`.
pub fn mnVibration_GetPadIndex(port: i32) -> u8 {
    port as u8
}

// ---- Round 2 ----

/// mnmain.c `mn_GetDigitAt`: `(n / 10^place) % 10` through lb_00CE `powi` (place 0..31, where
/// the wrapping 10^place is never 0).
pub fn mn_GetDigitAt(arg0: i32, arg1: i32) -> i32 {
    arg0.wrapping_div(crate::lb::powi(10, arg1)).wrapping_rem(10)
}

/// mnmain.c `mn_GetDigitCount`: decimal digits of `num`, at most 11.
pub fn mn_GetDigitCount(num: i32) -> i32 {
    if num == 0 {
        return 1;
    }
    let mut count = 0;
    while num / crate::lb::powi(10, count) != 0 {
        count += 1;
        if count >= 11 {
            break;
        }
    }
    count
}
