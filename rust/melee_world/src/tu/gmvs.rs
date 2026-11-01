//! gmvs.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use core::ptr::addr_of_mut;
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use melee_runtime::tu::gmvs_state::controller;

// ---- folded from gm.rs (consolidation steps 21/22) ----
/// `gmVs_GetSceneController_fp`.
pub unsafe fn gmVs_GetSceneController_fp() -> *mut VsSceneController {
    melee_core::ct!("gmVs_GetSceneController_fp");
    addr_of_mut!(controller)
}

/// `gmVs_GetSceneState_fp`.
pub unsafe fn gmVs_GetSceneState_fp() -> *mut VsSceneState {
    melee_core::ct!("gmVs_GetSceneState_fp");
    addr_of_mut!(controller.state)
}

/// `gm_GetStartMeleeRules_fp`.
pub unsafe fn gm_GetStartMeleeRules_fp() -> *mut StartMeleeRules {
    melee_core::ct!("gm_GetStartMeleeRules_fp");
    addr_of_mut!(controller.start)
}

/// `gm_8016B238_fp`.
pub unsafe fn gm_8016B238_fp() -> i32 {
    melee_core::ct!("gm_8016B238_fp");
    (*gm_GetStartMeleeRules_fp()).x6 as i32
}

/// `gm_8016B168_fp`: is it a team battle.
pub unsafe fn gm_8016B168_fp() -> bool {
    melee_core::ct!("gm_8016B168_fp");
    (*gm_GetStartMeleeRules_fp()).is_teams == 1
}

/// `gm_8016AE80_fp`: the item frequency.
pub unsafe fn gm_8016AE80_fp() -> i32 {
    melee_core::ct!("gm_8016AE80_fp");
    (*gm_GetStartMeleeRules_fp()).item_freq as i32
}

/// `gm_8016AEC8_fp`: the sudden death penalty.
pub unsafe fn gm_8016AEC8_fp() -> i32 {
    melee_core::ct!("gm_8016AEC8_fp");
    (*gm_GetStartMeleeRules_fp()).sd_penalty as i32
}

/// `gm_8016B0E8_fp`.
pub unsafe fn gm_8016B0E8_fp() -> i32 {
    melee_core::ct!("gm_8016B0E8_fp");
    (*gm_GetStartMeleeRules_fp()).x4_2() as i32
}

/// `gm_8016B110_fp`: does the timer count up.
pub unsafe fn gm_8016B110_fp() -> i32 {
    melee_core::ct!("gm_8016B110_fp");
    (*gm_GetStartMeleeRules_fp()).timer_counts_up() as i32
}

/// `gm_8016B124_fp`.
pub unsafe fn gm_8016B124_fp() -> i32 {
    melee_core::ct!("gm_8016B124_fp");
    (*gm_GetStartMeleeRules_fp()).x2_1() as i32
}

/// `gm_8016B1A8_fp`.
pub unsafe fn gm_8016B1A8_fp() -> bool {
    melee_core::ct!("gm_8016B1A8_fp");
    (*gm_GetStartMeleeRules_fp()).x9 == 1
}

/// `gm_8016B184_fp`: is it a single-player scene.
pub unsafe fn gm_8016B184_fp() -> bool {
    melee_core::ct!("gm_8016B184_fp");
    if (*gmVs_GetSceneController_fp()).state.is_singleplayer == 1 {
        return true;
    }
    false
}

/// `gm_8016B258_fp`.
pub unsafe fn gm_8016B258_fp(arg0: i32) -> i32 {
    melee_core::ct!("gm_8016B258_fp");
    (*gmVs_GetSceneController_fp()).state.fighters[arg0 as usize].anon_4.anon_0.x4_b3() as i32
}

/// `gm_8016B774_fp`: the match-end record.
pub unsafe fn gm_8016B774_fp() -> *mut MatchEnd {
    melee_core::ct!("gm_8016B774_fp");
    addr_of_mut!(controller.state.x24C)
}

/// `gm_GetFrameCount_fp`.
pub unsafe fn gm_GetFrameCount_fp() -> u32 {
    melee_core::ct!("gm_GetFrameCount_fp");
    (*gmVs_GetSceneController_fp()).state.frame_count
}

/// `gm_8016AEEC_fp`: the timer seconds.
pub unsafe fn gm_8016AEEC_fp() -> u32 {
    melee_core::ct!("gm_8016AEEC_fp");
    (*gmVs_GetSceneController_fp()).state.timer_seconds
}

/// `gm_8016AF0C_fp`: the on-screen timer fraction as a 0..99 value.
pub unsafe fn gm_8016AF0C_fp() -> u16 {
    melee_core::ct!("gm_8016AF0C_fp");
    let tmp = (99.0f32 * (*gmVs_GetSceneController_fp()).state.unk_2C as f32 / 59.0f32) as i32;
    let mut var_r31 = tmp as u16;
    if gm_8016B110_fp() == 0 {
        var_r31 = (0x63u16).wrapping_sub(var_r31);
    }
    var_r31
}

/// `GetMatchTimer_fp`: the displayed timer into `*arg0`; false when the match has no timer.
pub unsafe fn GetMatchTimer_fp(arg0: *mut i32) -> bool {
    melee_core::ct!("GetMatchTimer_fp");
    let data = gmVs_GetSceneController_fp();
    if !arg0.is_null() && (*data).start.timer_enabled() != 0 {
        if gm_8016B110_fp() != 0 {
            *arg0 = (*data).start.time_limit.wrapping_sub((*data).state.timer_seconds) as i32;
        } else {
            *arg0 = (*data).state.timer_seconds as i32;
        }
        return true;
    }
    false
}
