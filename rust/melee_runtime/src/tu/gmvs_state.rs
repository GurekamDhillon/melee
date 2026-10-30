//! The VS scene controller global of `src/melee/gm/gmvs.c` (`controller`, 0x8046B6A0) and its
//! accessors, which the fighter/character code reads from below `melee_modes`
//! (`gm_8016B1D8` from Purin's rollout). The static and the accessors were in
//! `melee_modes::tu::gmvs` (still re-exported from there: one global, not a copy).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use core::ptr::addr_of_mut;
use melee_types::{StartMeleeRules, VsSceneController, VsSceneState};

/* 46B6A0 */
pub static mut controller: VsSceneController = unsafe { core::mem::zeroed() };

pub unsafe fn gmVs_GetSceneController() -> *mut VsSceneController {
    melee_core::ct!("gmVs_GetSceneController");
    addr_of_mut!(controller)
}

pub unsafe fn gmVs_GetSceneState() -> *mut VsSceneState {
    melee_core::ct!("gmVs_GetSceneState");
    addr_of_mut!(controller.state)
}

pub unsafe fn gm_GetStartMeleeRules() -> *mut StartMeleeRules {
    melee_core::ct!("gm_GetStartMeleeRules");
    addr_of_mut!(controller.start)
}

/// `gm_8016B1D8`: `StartMeleeRules::x5_3`.
pub unsafe fn gm_8016B1D8() -> i32 {
    melee_core::ct!("gm_8016B1D8");
    (*gm_GetStartMeleeRules()).x5_3() as i32
}
