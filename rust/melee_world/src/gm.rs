
//! The game-mode / VS-scene globals of `src/melee/gm/gm_1A3F.c` (`state_machine`) and
//! `src/melee/gm/gmvs.c` (`controller`, the VS scene controller) with the plain accessors the stage,
//! HUD and effect TUs call (`gm_GetCurrentGameMode`, `gm_GetStartMeleeRules`, `gm_8016B238`, ...).
//! They live here, in the lowest crate that needs them; `melee_modes::tu::{gm_1A3F, gmvs}`
//! (which depend on this crate and port the rest of both files) re-export them, so there is a single
//! copy of each global and each accessor. The functions that call into other TUs
//! (`gm_8016C6C0` -> ftLib, `gm_8016ECE8` -> Player_*, ...) stay in `melee_modes`.
//! No float arithmetic except `gm_8016AF0C` (a multiply then a divide, nothing fused).
//! Checked by the modes' `tests/tu_gmvs.rs` / `tests/tu_gm_1A3F.rs`, which run on these items.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use core::ptr::addr_of_mut;

use melee_types::*;

/// `struct routingInfo` (gm_1A3F.c), GameCube size 6.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct routingInfo {
    /// `::GameModeKind`
    pub curr_mode: u8,
    /// `::GameModeKind`
    pub pending_mode: u8,
    /// `::GameModeKind`
    pub prev_mode: u8,
    /// from `::GameModeState::id`
    pub curr_state_id: u8,
    pub prev_state_id: u8,
    pub next_state_id: u8,
}

melee_core::gc_layout!(routingInfo, "routingInfo", size 6, align 1, [
    curr_mode @ 0x0: 1 "u8",
    pending_mode @ 0x1: 1 "u8",
    prev_mode @ 0x2: 1 "u8",
    curr_state_id @ 0x3: 1 "u8",
    prev_state_id @ 0x4: 1 "u8",
    next_state_id @ 0x5: 1 "u8",
]);

/// `struct stateMachine` (gm_1A3F.c), GameCube size 0x14.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct stateMachine {
    pub routing: routingInfo,
    pub backup_routing: routingInfo,
    /// `::bool`
    pub pending_mode_change: u8,
    pub get_override: Option<unsafe extern "C" fn() -> u8>,
}

melee_core::gc_layout!(stateMachine, "stateMachine", size 0x14, align 4, [
    routing @ 0x0: 6 "struct routingInfo",
    backup_routing @ 0x6: 6 "struct routingInfo",
    pending_mode_change @ 0xC: 1 "u8",
    get_override @ 0x10: 4 "u8 (*)(void)",
]);

/// `static struct stateMachine state_machine;`
pub static mut state_machine: stateMachine = stateMachine {
    routing: routingInfo { curr_mode: 0, pending_mode: 0, prev_mode: 0, curr_state_id: 0, prev_state_id: 0, next_state_id: 0 },
    backup_routing: routingInfo { curr_mode: 0, pending_mode: 0, prev_mode: 0, curr_state_id: 0, prev_state_id: 0, next_state_id: 0 },
    pending_mode_change: 0,
    get_override: None,
};

/// `static VsSceneController controller;` (gmvs.c).
pub use melee_runtime::tu::gmvs_state::controller;

/// `lbl_8046B488` (gm_16A2.c): the unlock / scene flags record (`gm_1601_GetUnkData`).
pub static mut lbl_8046B488: lbl_8046B488_t = unsafe { core::mem::zeroed() };

/// `lbl_8046B668` (gm_16A2.c).
pub static mut lbl_8046B668: lbl_8046B668_t = unsafe { core::mem::zeroed() };

pub use crate::tu::gmvs::GetMatchTimer_fp as GetMatchTimer;
pub use crate::tu::gmvs::gmVs_GetSceneController_fp as gmVs_GetSceneController;
pub use crate::tu::gmvs::gmVs_GetSceneState_fp as gmVs_GetSceneState;
pub use crate::tu::gm_16a2::gm_1601_GetUnkData;
pub use crate::tu::gm_16a2::gm_80169394_fp as gm_80169394;
pub use crate::tu::gm_16a2::gm_80169520_fp as gm_80169520;
pub use crate::tu::gm_16a2::gm_80169530_fp as gm_80169530;
pub use crate::tu::gm_16a2::gm_8016A1F8_fp as gm_8016A1F8;
pub use crate::tu::gm_16a2::gm_8016A97C_fp as gm_8016A97C;
pub use crate::tu::gm_16a2::gm_8016A98C_fp as gm_8016A98C;
pub use crate::tu::gmvs::gm_8016AE80_fp as gm_8016AE80;
pub use crate::tu::gmvs::gm_8016AEC8_fp as gm_8016AEC8;
pub use crate::tu::gmvs::gm_8016AEEC_fp as gm_8016AEEC;
pub use crate::tu::gmvs::gm_8016AF0C_fp as gm_8016AF0C;
pub use crate::tu::gmvs::gm_8016B0E8_fp as gm_8016B0E8;
pub use crate::tu::gmvs::gm_8016B110_fp as gm_8016B110;
pub use crate::tu::gmvs::gm_8016B124_fp as gm_8016B124;
pub use crate::tu::gmvs::gm_8016B168_fp as gm_8016B168;
pub use crate::tu::gmvs::gm_8016B184_fp as gm_8016B184;
pub use crate::tu::gmvs::gm_8016B1A8_fp as gm_8016B1A8;
pub use crate::tu::gmvs::gm_8016B238_fp as gm_8016B238;
pub use crate::tu::gmvs::gm_8016B258_fp as gm_8016B258;
pub use crate::tu::gmvs::gm_8016B774_fp as gm_8016B774;
pub use crate::tu::gm_1a3f::gm_GetCurrentGameMode;
pub use crate::tu::gmvs::gm_GetFrameCount_fp as gm_GetFrameCount;
pub use crate::tu::gm_1a3f::gm_GetPreviousGameMode;
pub use crate::tu::gmvs::gm_GetStartMeleeRules_fp as gm_GetStartMeleeRules;
