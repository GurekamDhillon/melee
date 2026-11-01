
//! Round 9 (night 3 round 5): mn getters over records the .c files define, verified bit for
//! bit against the C by `tests/oracle_r9.rs`.
use melee_types::{HSD_GObj, HSD_JObj, HSD_Text};

/// mnvibration.c `MnVibrationData`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MnVibrationData {
    pub x0: [u8; 6],
    pub x6: [u8; 4],
    pub scroll_offset: u8,
    pub pad_0B: u8,
    pub jobjs: [*mut HSD_JObj; 25],
    pub texts: [*mut HSD_Text; 8],
    pub title_text: *mut HSD_Text,
    pub cursor_gobj: *mut HSD_GObj,
}

melee_core::gc_layout!(MnVibrationData, "MnVibrationData", size 0x98, align 4, [
    x0 @ 0x0: 6 "u8[6]",
    x6 @ 0x6: 4 "u8[4]",
    scroll_offset @ 0xA: 1 "u8",
    pad_0B @ 0xB: 1 "u8",
    jobjs @ 0xC: 0x64 "HSD_JObj*[25]",
    texts @ 0x70: 0x20 "HSD_Text*[8]",
    title_text @ 0x90: 4 "HSD_Text*",
    cursor_gobj @ 0x94: 4 "HSD_GObj*",
]);

pub use crate::tu::mnvibration::mnVibration_GetCursorRow_fp as mnVibration_GetCursorRow;
