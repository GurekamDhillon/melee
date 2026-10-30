//! src/melee/gm/gm_18A1.c: Classic-mode stage table getters and the All-Star result flags.
//!
//! The PPC bitfield `u8_bits { b7:1, b6:1, ..., b0:1 }` (first declared = most significant bit)
//! makes `bN` the mask `1 << N`. Checked against the C TU by tests/tu_gm_18A1.rs (the native C
//! numbers its bitfields from the LSB, so the test bit-reverses those three bytes).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use melee_types::*;

/// Callees outside this TU (hooks; unset they panic `TODO: <name> is not ported`).
pub mod ext {
    pub use crate::tu_ext::{
        fn_8017E318, gmMainLib_8015CDE0, gm_GetFrameCount, gm_GetStartMeleeRules, Player_GetDamage,
    };
}

/// `ClassicStageEntry` (0x10): one Classic-mode stage row.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClassicStageEntry {
    pub stage_kind: u8,
    pub pad_1: u8,
    /// Divided by 100 to get the ratio.
    pub scale0_pct: u16,
    pub scale1_pct: u16,
    pub pad_6: [u8; 0xA],
}
melee_core::gc_layout!(ClassicStageEntry, "ClassicStageEntry", size 0x10, align 2, [
    stage_kind @ 0x0: 1 "u8",
    pad_1 @ 0x1: 1 "u8",
    scale0_pct @ 0x2: 2 "u16",
    scale1_pct @ 0x4: 2 "u16",
    pad_6 @ 0x6: 10 "u8[10]",
]);
const _: () = assert!(core::mem::size_of::<ClassicStageEntry>() == 0x10);

/// `UnkAllstarData gm_80473A18` (shared with other gm TUs).
pub static mut gm_80473A18: UnkAllstarData = unsafe { core::mem::zeroed() };

/// `gmTraining_ClassicStages[65]`.
pub static gmTraining_ClassicStages: [ClassicStageEntry; 65] = [
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [0, 1, 0x17, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 70, scale1_pct: 140, pad_6: [1, 1, 0x17, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 80, scale1_pct: 120, pad_6: [3, 1, 0x17, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 90, scale1_pct: 100, pad_6: [5, 1, 0x17, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 100, scale1_pct: 95, pad_6: [7, 1, 0x17, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [1, 2, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 70, scale1_pct: 140, pad_6: [3, 2, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 80, scale1_pct: 115, pad_6: [4, 2, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 90, scale1_pct: 100, pad_6: [6, 2, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 100, scale1_pct: 95, pad_6: [8, 2, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [2, 3, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 70, scale1_pct: 140, pad_6: [4, 3, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 80, scale1_pct: 110, pad_6: [5, 3, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 90, scale1_pct: 100, pad_6: [7, 3, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 100, scale1_pct: 95, pad_6: [9, 3, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [3, 1, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 70, scale1_pct: 140, pad_6: [5, 1, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 80, scale1_pct: 120, pad_6: [6, 1, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 90, scale1_pct: 105, pad_6: [8, 1, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 100, scale1_pct: 95, pad_6: [9, 1, 4, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [0, 2, 0x17, 0, 2, 0x17, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 67, scale1_pct: 140, pad_6: [2, 2, 0x17, 2, 2, 0x17, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 78, scale1_pct: 120, pad_6: [3, 2, 0x17, 3, 2, 0x17, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 90, scale1_pct: 105, pad_6: [5, 2, 4, 4, 2, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 100, scale1_pct: 100, pad_6: [6, 2, 4, 6, 2, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [1, 3, 0x17, 1, 3, 0x17, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 67, scale1_pct: 140, pad_6: [3, 3, 4, 3, 3, 0x17, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 78, scale1_pct: 120, pad_6: [4, 3, 4, 3, 3, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 90, scale1_pct: 105, pad_6: [5, 3, 4, 5, 3, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 100, scale1_pct: 100, pad_6: [7, 3, 4, 7, 3, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [2, 1, 0x17, 1, 1, 0x17, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 70, scale1_pct: 140, pad_6: [3, 1, 4, 3, 1, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 78, scale1_pct: 120, pad_6: [4, 1, 4, 4, 1, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 90, scale1_pct: 105, pad_6: [6, 1, 4, 5, 1, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 102, scale1_pct: 100, pad_6: [8, 1, 4, 7, 1, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [2, 2, 4, 2, 2, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 70, scale1_pct: 130, pad_6: [3, 2, 4, 3, 2, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 80, scale1_pct: 110, pad_6: [5, 2, 4, 4, 2, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 90, scale1_pct: 105, pad_6: [7, 2, 4, 5, 2, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 105, scale1_pct: 100, pad_6: [9, 2, 4, 8, 2, 4, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [0, 3, 0x17, 0, 3, 0x17, 0, 3, 0x17, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 67, scale1_pct: 130, pad_6: [2, 3, 0x17, 2, 3, 0x17, 1, 3, 0x17, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 78, scale1_pct: 110, pad_6: [3, 3, 0x17, 3, 3, 0x17, 3, 3, 0x17, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 89, scale1_pct: 100, pad_6: [4, 3, 4, 4, 3, 0x17, 4, 3, 0x17, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 100, scale1_pct: 100, pad_6: [6, 3, 4, 6, 3, 4, 6, 3, 0x17, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [1, 1, 0x17, 1, 1, 0x17, 1, 1, 0x17, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 69, scale1_pct: 125, pad_6: [2, 1, 0x17, 2, 1, 0x17, 2, 1, 0x17, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 75, scale1_pct: 110, pad_6: [4, 1, 0x11, 4, 1, 0x17, 3, 1, 0x17, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 89, scale1_pct: 100, pad_6: [5, 1, 4, 4, 1, 0x11, 4, 1, 0x17, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 100, scale1_pct: 100, pad_6: [7, 1, 4, 7, 1, 4, 7, 1, 0x11, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [1, 2, 0x17, 1, 2, 0x17, 1, 2, 0x17, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 70, scale1_pct: 125, pad_6: [3, 2, 0x11, 3, 2, 0x17, 3, 2, 0x17, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 79, scale1_pct: 112, pad_6: [4, 2, 4, 4, 2, 0x11, 4, 2, 0x17, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 90, scale1_pct: 100, pad_6: [5, 2, 4, 5, 2, 4, 4, 2, 0x11, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 100, scale1_pct: 100, pad_6: [8, 2, 4, 8, 2, 4, 7, 2, 4, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 50, scale1_pct: 170, pad_6: [2, 3, 0x17, 2, 3, 0x17, 1, 3, 0x17, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 70, scale1_pct: 125, pad_6: [4, 3, 0x11, 4, 3, 4, 3, 3, 0x17, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 80, scale1_pct: 110, pad_6: [5, 3, 4, 4, 3, 0x11, 4, 3, 0x17, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 92, scale1_pct: 100, pad_6: [6, 3, 4, 5, 3, 4, 5, 3, 0x11, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 105, scale1_pct: 96, pad_6: [9, 3, 4, 8, 3, 4, 8, 3, 0x11, 0] },
    ClassicStageEntry { stage_kind: 4, pad_1: 0, scale0_pct: 20, scale1_pct: 360, pad_6: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 30, scale1_pct: 320, pad_6: [2, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 3, pad_1: 0, scale0_pct: 40, scale1_pct: 300, pad_6: [3, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 2, pad_1: 0, scale0_pct: 48, scale1_pct: 240, pad_6: [5, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    ClassicStageEntry { stage_kind: 1, pad_1: 0, scale0_pct: 55, scale1_pct: 200, pad_6: [7, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
];

#[inline]
fn row(difficulty: u8, stage_slot: u8) -> &'static ClassicStageEntry {
    &gmTraining_ClassicStages[stage_slot as usize + difficulty as usize * 5]
}

pub fn gm_8018A160(difficulty: u8, stage_slot: u8) -> u8 {
    melee_core::ct!("gm_8018A160");
    row(difficulty, stage_slot).stage_kind
}

pub fn gm_8018A188(difficulty: u8, stage_slot: u8) -> f32 {
    melee_core::ct!("gm_8018A188");
    row(difficulty, stage_slot).scale0_pct as f32 / 100.0
}

pub fn gm_8018A1D8(difficulty: u8, stage_slot: u8) -> f32 {
    melee_core::ct!("gm_8018A1D8");
    row(difficulty, stage_slot).scale1_pct as f32 / 100.0
}

/// `pad_6[n]` of a row, where `n` may run past the row into the next one (the C indexes the
/// 10-byte array with `idx * 3 + k`).
#[inline]
fn pad6(difficulty: u8, stage_slot: u8, n: usize) -> u8 {
    unsafe { *row(difficulty, stage_slot).pad_6.as_ptr().add(n) }
}

pub fn gm_8018A228(difficulty: u8, stage_slot: u8, idx: u8) -> u8 {
    melee_core::ct!("gm_8018A228");
    pad6(difficulty, stage_slot, idx as usize * 3)
}

pub fn gm_8018A25C(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    melee_core::ct!("gm_8018A25C");
    pad6(difficulty, stage_slot, arg2 as usize * 3 + 1)
}

pub fn gm_8018A290(difficulty: u8, stage_slot: u8, idx: u8) -> u8 {
    melee_core::ct!("gm_8018A290");
    pad6(difficulty, stage_slot, 2 + idx as usize * 3)
}

pub fn gm_8018A2C4(difficulty: u8, stage_slot: u8) -> f32 {
    melee_core::ct!("gm_8018A2C4");
    row(difficulty, stage_slot).scale0_pct as f32 / 100.0
}

pub fn gm_8018A314(difficulty: u8, stage_slot: u8) -> f32 {
    melee_core::ct!("gm_8018A314");
    row(difficulty, stage_slot).scale1_pct as f32 / 100.0
}

/// `((u8_bits*) &arg0->_x448[i])->bN = 1`
#[inline]
fn set_bit(arg0: &mut MatchEnd, i: usize, n: u32) {
    arg0._x448[i] |= 1 << n;
}

/// `fn_8018A364(int)`: the C takes the MatchEnd pointer as an `int` (32-bit guest address).
pub unsafe fn fn_8018A364(arg0: *mut MatchEnd) {
    melee_core::ct!("fn_8018A364");
    let arg0 = &mut *arg0;
    let rules = ext::gm_GetStartMeleeRules();
    let data = &mut gm_80473A18;

    if ext::fn_8017E318() > 0 {
        set_bit(arg0, 2, 3);
    }

    if (*rules).x4_5() != 0 {
        let main_data = ext::gmMainLib_8015CDE0();
        let total_time = data.x0.xC.x20.wrapping_add(ext::gm_GetFrameCount());
        set_bit(arg0, 0, 5);

        if data.x0.x0.cpu_level == 4 {
            set_bit(arg0, 0, 2);
        }

        if total_time < 0x6270 {
            set_bit(arg0, 2, 6);
        } else if total_time < 0x9AB0 {
            set_bit(arg0, 2, 7);
        }

        if data.x0.xC.x1C as u32 == 0 && arg0.player_standings[0].x44 == 0 {
            set_bit(arg0, 1, 1);
        }

        if data.x0.xC.xE != 0 {
            set_bit(arg0, 2, 5);
        }

        if data.x0.xC.xF != 0 {
            set_bit(arg0, 2, 4);
        }

        if (data.x0.xC.x1C as u32).wrapping_add(arg0.player_standings[0].x44) == ext::Player_GetDamage(0) as u32 {
            set_bit(arg0, 0, 1);
        }

        if data.x0.xC.xD != 0 {
            set_bit(arg0, 1, 0);
            return;
        }

        if arg0.player_standings[0].stocks as i32 == (*main_data).stocks as i32 {
            set_bit(arg0, 1, 2);
        }
    }
}
