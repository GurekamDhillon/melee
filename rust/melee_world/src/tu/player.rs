//! player.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::pl::{ent, map, sp};

// ---- folded from pl.rs (consolidation steps 21/22) ----
/// `Player_CheckSlot_fp`: `HSD_ASSERTREPORT(102, 0, "cant get player struct! %d\n", slot)`. The C
/// reports and carries on; an out-of-range slot is a bug either way, so this panics.
#[inline]
pub fn Player_CheckSlot_fp(slot: i32) {
    melee_core::ct!("Player_CheckSlot_fp");
    if slot < 0 || !(slot < Gm_Player_NumMax as i32) {
        panic!("src/melee/pl/player.c:102: cant get player struct! {}", slot);
    }
}

/// `Player_GetPlayerSlotType_fp`.
pub fn Player_GetPlayerSlotType_fp(slot: i32) -> Gm_PKind {
    melee_core::ct!("Player_GetPlayerSlotType_fp");
    sp(slot).pkind
}

/// `Player_GetEntity_fp`.
pub fn Player_GetEntity_fp(slot: i32) -> *mut HSD_GObj {
    melee_core::ct!("Player_GetEntity_fp");
    let player = sp(slot);
    ent(player, 0)
}

/// `Player_GetEntityAtIndex_fp`.
pub fn Player_GetEntityAtIndex_fp(slot: i32, index: i32) -> *mut HSD_GObj {
    melee_core::ct!("Player_GetEntityAtIndex_fp");
    let player = sp(slot);
    let entity_index = player.transformed[index as usize] as usize;
    player.player_entity[entity_index]
}

/// `Player_LoadPlayerCoords_fp`.
pub fn Player_LoadPlayerCoords_fp(slot: i32, arg_vec: *mut Vec) {
    melee_core::ct!("Player_LoadPlayerCoords_fp");
    let player = sp(slot);
    unsafe { *arg_vec = player.player_poses.byIndex[player.transformed[0] as usize] };
}

/// `Player_GetTeam_fp`.
pub fn Player_GetTeam_fp(slot: i32) -> i32 {
    melee_core::ct!("Player_GetTeam_fp");
    sp(slot).team as i32
}

/// `Player_GetStocks_fp`.
pub fn Player_GetStocks_fp(slot: i32) -> i32 {
    melee_core::ct!("Player_GetStocks_fp");
    sp(slot).stocks as i32
}

/// `Player_GetPlayerCharacter_fp`.
pub fn Player_GetPlayerCharacter_fp(slot: i32) -> CharacterKind {
    melee_core::ct!("Player_GetPlayerCharacter_fp");
    sp(slot).ckind
}

/// `Player_GetPlayerId_fp`.
pub fn Player_GetPlayerId_fp(slot: i32) -> i32 {
    melee_core::ct!("Player_GetPlayerId_fp");
    sp(slot).player_id as i32
}

/// `Player_GetNametagSlotID_fp`.
pub fn Player_GetNametagSlotID_fp(slot: i32) -> u8 {
    melee_core::ct!("Player_GetNametagSlotID_fp");
    sp(slot).nametag_slot_id as u8
}

/// `Player_GetPlayerState_fp`.
pub fn Player_GetPlayerState_fp(slot: i32) -> i32 {
    melee_core::ct!("Player_GetPlayerState_fp");
    sp(slot).player_state
}

/// `Player_GetDamage_fp`.
pub fn Player_GetDamage_fp(slot: i32) -> i32 {
    melee_core::ct!("Player_GetDamage_fp");
    let player = sp(slot);
    unsafe { player.staminas.byIndex[player.transformed[0] as usize] as i32 }
}

/// `Player_GetMoreFlagsBit2_fp`.
pub fn Player_GetMoreFlagsBit2_fp(slot: i32) -> bool {
    melee_core::ct!("Player_GetMoreFlagsBit2_fp");
    sp(slot).more_flags.b2() != 0
}

/// `Player_GetRemainingHP_fp`.
pub fn Player_GetRemainingHP_fp(slot: i32) -> i32 {
    melee_core::ct!("Player_GetRemainingHP_fp");
    let player = sp(slot);
    let result: i32 = unsafe {
        player.staminas.byName.stamina as i32 - player.staminas.byIndex[player.transformed[0] as usize] as i32
    };
    let mut result2 = result as u32;
    if result < 0 {
        result2 = 0;
    }
    result2 as i32
}

/// `Player_800325C8_fp`: the fighter kind of a character (`b == 0`) or its extra fighter (`b == 1`).
pub fn Player_800325C8_fp(kind: CharacterKind, b: i32) -> i8 {
    melee_core::ct!("Player_800325C8_fp");
    if b == 0 {
        return map(kind).internal_id;
    } else if b == 1 {
        return map(kind).extra_internal_id;
    }
    -1
}

/// `Player_GetCoins_fp`.
pub fn Player_GetCoins_fp(slot: i32) -> i32 {
    melee_core::ct!("Player_GetCoins_fp");
    sp(slot).current_coins
}
