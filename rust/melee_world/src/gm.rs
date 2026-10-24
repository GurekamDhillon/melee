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
pub static mut controller: VsSceneController = unsafe { core::mem::zeroed() };

/// `gm_GetCurrentGameMode()`.
pub fn gm_GetCurrentGameMode() -> u8 {
    unsafe { state_machine.routing.curr_mode }
}

/// `gm_GetPreviousGameMode()`.
pub fn gm_GetPreviousGameMode() -> u8 {
    unsafe { state_machine.routing.prev_mode }
}

/// `gmVs_GetSceneController`.
pub unsafe fn gmVs_GetSceneController() -> *mut VsSceneController {
    addr_of_mut!(controller)
}

/// `gmVs_GetSceneState`.
pub unsafe fn gmVs_GetSceneState() -> *mut VsSceneState {
    addr_of_mut!(controller.state)
}

/// `gm_GetStartMeleeRules`.
pub unsafe fn gm_GetStartMeleeRules() -> *mut StartMeleeRules {
    addr_of_mut!(controller.start)
}

/// `gm_8016B238`.
pub unsafe fn gm_8016B238() -> i32 {
    (*gm_GetStartMeleeRules()).x6 as i32
}

/// `gm_8016B168`: is it a team battle.
pub unsafe fn gm_8016B168() -> bool {
    (*gm_GetStartMeleeRules()).is_teams == 1
}

/// `gm_8016AE80`: the item frequency.
pub unsafe fn gm_8016AE80() -> i32 {
    (*gm_GetStartMeleeRules()).item_freq as i32
}

/// `gm_8016AEC8`: the sudden death penalty.
pub unsafe fn gm_8016AEC8() -> i32 {
    (*gm_GetStartMeleeRules()).sd_penalty as i32
}

/// `gm_8016B0E8`.
pub unsafe fn gm_8016B0E8() -> i32 {
    (*gm_GetStartMeleeRules()).x4_2() as i32
}

/// `gm_8016B110`: does the timer count up.
pub unsafe fn gm_8016B110() -> i32 {
    (*gm_GetStartMeleeRules()).timer_counts_up() as i32
}

/// `gm_8016B124`.
pub unsafe fn gm_8016B124() -> i32 {
    (*gm_GetStartMeleeRules()).x2_1() as i32
}

/// `gm_8016B1A8`.
pub unsafe fn gm_8016B1A8() -> bool {
    (*gm_GetStartMeleeRules()).x9 == 1
}

/// `gm_8016B184`: is it a single-player scene.
pub unsafe fn gm_8016B184() -> bool {
    if (*gmVs_GetSceneController()).state.is_singleplayer == 1 {
        return true;
    }
    false
}

/// `gm_8016B258`.
pub unsafe fn gm_8016B258(arg0: i32) -> i32 {
    (*gmVs_GetSceneController()).state.fighters[arg0 as usize].anon_4.anon_0.x4_b3() as i32
}

/// `gm_8016B774`: the match-end record.
pub unsafe fn gm_8016B774() -> *mut MatchEnd {
    addr_of_mut!(controller.state.x24C)
}

/// `gm_GetFrameCount`.
pub unsafe fn gm_GetFrameCount() -> u32 {
    (*gmVs_GetSceneController()).state.frame_count
}

/// `gm_8016AEEC`: the timer seconds.
pub unsafe fn gm_8016AEEC() -> u32 {
    (*gmVs_GetSceneController()).state.timer_seconds
}

/// `gm_8016AF0C`: the on-screen timer fraction as a 0..99 value.
pub unsafe fn gm_8016AF0C() -> u16 {
    let tmp = (99.0f32 * (*gmVs_GetSceneController()).state.unk_2C as f32 / 59.0f32) as i32;
    let mut var_r31 = tmp as u16;
    if gm_8016B110() == 0 {
        var_r31 = (0x63u16).wrapping_sub(var_r31);
    }
    var_r31
}

/// `GetMatchTimer`: the displayed timer into `*arg0`; false when the match has no timer.
pub unsafe fn GetMatchTimer(arg0: *mut i32) -> bool {
    let data = gmVs_GetSceneController();
    if !arg0.is_null() && (*data).start.timer_enabled() != 0 {
        if gm_8016B110() != 0 {
            *arg0 = (*data).start.time_limit.wrapping_sub((*data).state.timer_seconds) as i32;
        } else {
            *arg0 = (*data).state.timer_seconds as i32;
        }
        return true;
    }
    false
}

/// `lbl_8046B488` (gm_16A2.c): the unlock / scene flags record (`gm_1601_GetUnkData`).
pub static mut lbl_8046B488: lbl_8046B488_t = unsafe { core::mem::zeroed() };
/// `lbl_8046B668` (gm_16A2.c).
pub static mut lbl_8046B668: lbl_8046B668_t = unsafe { core::mem::zeroed() };

/// `gm_1601_GetUnkData`.
pub unsafe fn gm_1601_GetUnkData() -> *mut lbl_8046B488_t {
    addr_of_mut!(lbl_8046B488)
}

/// `gm_80169394`.
pub unsafe fn gm_80169394() -> i32 {
    (*gm_1601_GetUnkData()).unk_10_b3() as i32
}

/// `gm_80169520`.
pub unsafe fn gm_80169520() -> *mut core::ffi::c_void {
    (*gm_1601_GetUnkData()).x20.as_mut_ptr() as *mut core::ffi::c_void
}

/// `gm_80169530`.
pub unsafe fn gm_80169530() -> *mut core::ffi::c_void {
    (*gm_1601_GetUnkData()).xA2.as_mut_ptr() as *mut core::ffi::c_void
}

/// `gm_8016A1F8`.
pub unsafe fn gm_8016A1F8() -> bool {
    (*gm_1601_GetUnkData()).unk_10_b1() != 0
}

/// `gm_8016A97C`.
pub unsafe fn gm_8016A97C() -> *mut core::ffi::c_void {
    lbl_8046B668.arr2.as_mut_ptr() as *mut core::ffi::c_void
}

/// `gm_8016A98C`.
pub unsafe fn gm_8016A98C() -> *mut lbl_8046B668_t {
    addr_of_mut!(lbl_8046B668)
}
