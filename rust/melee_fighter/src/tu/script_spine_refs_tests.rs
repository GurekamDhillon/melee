//! script_spine_refs_tests.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::script_caps::script_gobj_live;
use melee_types::{Fighter, HSD_GObj, ScriptCapsMovement, HSD_GOBJ_PLINK_FIGHTER};

// ---- folded from script_caps.rs (consolidation steps 21/22) ----
/// `ScriptGame_CapsFighter_fp(entity)`: the live fighter of script entity 1..=12 (slot + 6 * sub).
pub unsafe extern "C" fn ScriptGame_CapsFighter_fp(entity: i32) -> *mut Fighter {
    melee_core::ct!("ScriptGame_CapsFighter_fp");
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
