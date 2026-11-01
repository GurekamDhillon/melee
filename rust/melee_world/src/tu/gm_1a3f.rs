//! gm_1A3F.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::gm::{state_machine};

// ---- folded from gm.rs (consolidation steps 21/22) ----
/// `gm_GetCurrentGameMode()`.
pub fn gm_GetCurrentGameMode() -> u8 {
    melee_core::ct!("gm_GetCurrentGameMode");
    unsafe { state_machine.routing.curr_mode }
}

/// `gm_GetPreviousGameMode()`.
pub fn gm_GetPreviousGameMode() -> u8 {
    melee_core::ct!("gm_GetPreviousGameMode");
    unsafe { state_machine.routing.prev_mode }
}
