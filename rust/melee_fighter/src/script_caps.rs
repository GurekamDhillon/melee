//! The LAB's per-fighter movement caps (`pc/gameworld/script_fighter_caps_movement.inc`) that the
//! fighter and character code read from below `melee_modes`: the air-jump limit
//! (`ScriptGame_CapsMaxJumps`, called from `ftCo_JumpAerial*`, `ftcpuattack` and the Sheik/Zelda
//! up-B). It was in `melee_modes::tu::script_game` (still re-exported from there).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_types::{Fighter, HSD_GObj, ScriptCapsMovement, HSD_GOBJ_PLINK_FIGHTER};

pub static mut script_caps_movement: [ScriptCapsMovement; 12] = unsafe { core::mem::zeroed() };

/// `script_gobj_live(gobj)`: `gobj` is on the fighter player link.
pub unsafe extern "C" fn script_gobj_live(gobj: *mut HSD_GObj) -> i32 {
    // The C returns 0 while `HSD_GObjPLinkHead` is NULL; an unset hook is that case here.
    if gobj.is_null() || crate::hooks::PLINK_HEAD.is_none() {
        return 0;
    }
    let mut cur = crate::hooks::plink_head(HSD_GOBJ_PLINK_FIGHTER as i32);
    while !cur.is_null() {
        if cur == gobj {
            return 1;
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    0
}

/// `ScriptGame_CapsFighter(entity)`: the live fighter of script entity 1..=12 (slot + 6 * sub).
pub unsafe extern "C" fn ScriptGame_CapsFighter(entity: i32) -> *mut Fighter {
    if !(1..=12).contains(&entity) {
        return core::ptr::null_mut();
    }
    let g = crate::player::get_entity_at_index((entity - 1) % 6, (entity - 1) / 6);
    if script_gobj_live(g) != 0 && !(*g).user_data.is_null() {
        (*g).user_data as *mut Fighter
    } else {
        core::ptr::null_mut()
    }
}

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
