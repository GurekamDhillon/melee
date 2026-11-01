
//! Menu (`src/melee/mn/`) leaf functions: no calls, no globals. Hand ports, same names as the
//! C, checked by `tests/oracle.rs`.
#![allow(non_snake_case)]

use melee_runtime::cabi;

/// sizeof(CSSIcon) on the GameCube (libclang ppc32: 0x1C).
pub const CSS_ICON_SIZE: u32 = 0x1C;

pub use crate::tu::mnmain::GetSelectionFrameOffset_fp as GetSelectionFrameOffset;
pub use crate::tu::mncharsel::equalU8;
pub use crate::tu::mncharsel::getIconOffset;
pub use crate::tu::mncount::inline_is_row_char_fp as inline_is_row_char;
pub use crate::tu::mncount::inline_is_row_time_fp as inline_is_row_time;
pub use crate::tu::mndiagram2::mnDiagram2_IsDistanceStat_fp as mnDiagram2_IsDistanceStat;
pub use crate::tu::mndiagram2::mnDiagram2_IsIconOnlyStat_fp as mnDiagram2_IsIconOnlyStat;
pub use crate::tu::mndiagram2::mnDiagram2_IsPercentageStat_fp as mnDiagram2_IsPercentageStat;
pub use crate::tu::mndiagram2::mnDiagram2_IsTimeStat_fp as mnDiagram2_IsTimeStat;
pub use crate::tu::mnvibration::mnVibration_GetNextCursorRow_fp as mnVibration_GetNextCursorRow;
pub use crate::tu::mnvibration::mnVibration_GetPadIndex_fp as mnVibration_GetPadIndex;
pub use crate::tu::mnvibration::mnVibration_GetPreviousCursorRow_fp as mnVibration_GetPreviousCursorRow;
pub use crate::tu::mnmain::mn_8022C010_fp as mn_8022C010;
pub use crate::tu::mnmain::mn_GetDigitAt_fp as mn_GetDigitAt;
pub use crate::tu::mnmain::mn_GetDigitCount_fp as mn_GetDigitCount;
pub use crate::tu::mnname::signedCharactersEqual_fp as signedCharactersEqual;
pub use crate::tu::mncharsel::toFeet;
pub use crate::tu::mncharsel::toMeters;
pub use crate::tu::mnname::unsignedCharacter_fp as unsignedCharacter;
