//! Native port of src/melee/pl/plstale.c: the global attack-instance counter and the per-player
//! staling move queue.
//!
//! `plStale_InitAttackInstance`, `plStale_ResetStaleMoveTableForPlayer`,
//! `plStale_IncrementAttackInstance` and the global `staleAttackInstance` already have a port in
//! `crate::stale` and are re-exported under the C names. The two `UpdateStaleMovesFrom*` take the
//! gobjs as the C does and forward to the shared queue push there.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc, unused_unsafe)]

use crate::types::*;

pub use crate::stale::{
    increment_attack_instance as plStale_IncrementAttackInstance, init_attack_instance as plStale_InitAttackInstance,
    reset_stale_move_table as plStale_ResetStaleMoveTableForPlayer, STALE_ATTACK_INSTANCE as staleAttackInstance,
};

type G = *mut HSD_GObj;

/// `plStale_UpdateStaleMovesFromFighter(gobj, gobj_2)`.
pub unsafe fn plStale_UpdateStaleMovesFromFighter(gobj: G, gobj_2: G) {
    if gobj != gobj_2 {
        let ft = (*gobj).user_data as *mut Fighter;
        crate::stale::update_stale_moves_from_fighter(&*ft, false);
    }
}

/// `plStale_UpdateStaleMovesFromItem(gobj, gobj_2)`.
pub unsafe fn plStale_UpdateStaleMovesFromItem(gobj: G, gobj_2: G) {
    let it = (*gobj).user_data as *mut Item;
    let owner = (*it).owner;
    if crate::ftlib::ftlib_80086960(owner) && owner != gobj_2 {
        let owner_fp = (*owner).user_data as *const Fighter;
        crate::stale::update_stale_moves_from_item(Some(&*owner_fp), (*it).xD88_attackID, (*it).xD8C_attack_instance);
    }
}
