
//! The LAB's per-fighter movement caps (`pc/gameworld/script_fighter_caps_movement.inc`) that the
//! fighter and character code read from below `melee_modes`: the air-jump limit
//! (`ScriptGame_CapsMaxJumps`, called from `ftCo_JumpAerial*`, `ftcpuattack` and the Sheik/Zelda
//! up-B). It was in `melee_modes::tu::script_game` (still re-exported from there).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_types::{Fighter, HSD_GObj, ScriptCapsMovement, HSD_GOBJ_PLINK_FIGHTER};

pub static mut script_caps_movement: [ScriptCapsMovement; 12] = unsafe { core::mem::zeroed() };

/// `ScriptGame_CapsEntity(fp)`: the script entity (1..=12) of a fighter, 0 when it has none.
pub unsafe extern "C" fn ScriptGame_CapsEntity(fp: *mut Fighter) -> i32 {
    if fp.is_null() {
        return 0;
    }
    for entity in 1..=12 {
        if ScriptGame_CapsFighter(entity) == fp {
            return entity;
        }
    }
    0
}

pub unsafe extern "C" fn script_caps_max_jumps_value(fp: *mut Fighter, entity: i32) -> i32 {
    if fp.is_null() {
        return 0;
    }
    if !(1..=12).contains(&entity) {
        return (*fp).co_attrs.max_jumps;
    }
    let value = (&raw mut script_caps_movement as *mut ScriptCapsMovement).add((entity - 1) as usize);
    if (*value).owner != 0 && (*value).jumps >= 0 {
        (*value).jumps + 1
    } else {
        (*fp).co_attrs.max_jumps
    }
}

pub unsafe extern "C" fn ScriptGame_CapsMaxJumps(fp: *mut Fighter) -> i32 {
    script_caps_max_jumps_value(fp, ScriptGame_CapsEntity(fp))
}

pub use crate::tu::script_spine_refs_tests::ScriptGame_CapsFighter_fp as ScriptGame_CapsFighter;
pub use crate::tu::script_game::script_gobj_live_fp as script_gobj_live;
