
//! The player table of `src/melee/pl/player.c` that the stage, HUD and match code read: the
//! `player_slots` array, the `ftMapping_list` character-to-fighter table, `Player_CheckSlot` and the
//! plain field accessors (`Player_GetTeam`, `Player_GetEntity`, ...). They live here, in the lowest
//! crate that needs them, because the world TUs call them and the fighter crate (which depends on
//! this one) owns the rest of player.c; `melee_fighter::{stale, player, tu::player}` re-export these
//! items, so there is a single copy of the table and of each accessor.
//!
//! The functions that go through a fighter (`Player_8003219C` -> `ftLib_IsSleeping`, ...) stay in
//! `melee_fighter::tu::player`. No float arithmetic here. Checked by the fighter's
//! `tests/player_ref.rs` / `tests/stale_ref.rs` (gen/cref/player_ref.c), which run on these items.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use melee_types::*;

/// `Gm_Player_NumMax`.
pub const PLAYER_NUM_MAX: usize = 6;

/// `ftMapping` (player.c), one row per CharacterKind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FtMapping {
    pub internal_id: i8,
    pub extra_internal_id: i8,
    pub has_transformation: i8,
}

pub(crate) const NONE: i8 = -1;

pub(crate) const fn m(a: FighterKind, b: i8, t: i8) -> FtMapping {
    FtMapping { internal_id: a as i8, extra_internal_id: b, has_transformation: t }
}

pub(crate) const fn one(a: FighterKind) -> FtMapping {
    m(a, NONE, 0)
}

pub(crate) const fn mapping_init() -> [FtMapping; ChKind_Cap as usize] {
    let mut t = [FtMapping { internal_id: 0, extra_internal_id: 0, has_transformation: 0 }; ChKind_Cap as usize];
    let rows = [
        one(Ft_Kind_Captain), one(Ft_Kind_Donkey), one(Ft_Kind_Fox), one(Ft_Kind_GameWatch),
        one(Ft_Kind_Kirby), one(Ft_Kind_Koopa), one(Ft_Kind_Link), one(Ft_Kind_Luigi),
        one(Ft_Kind_Mario), one(Ft_Kind_Mars), one(Ft_Kind_Mewtwo), one(Ft_Kind_Ness),
        one(Ft_Kind_Peach), one(Ft_Kind_Pikachu), m(Ft_Kind_Popo, Ft_Kind_Nana as i8, 0),
        one(Ft_Kind_Purin), one(Ft_Kind_Samus), one(Ft_Kind_Yoshi),
        m(Ft_Kind_Zelda, Ft_Kind_Seak as i8, 1), m(Ft_Kind_Seak, Ft_Kind_Zelda as i8, 1),
        one(Ft_Kind_Falco), one(Ft_Kind_CLink), one(Ft_Kind_DrMario), one(Ft_Kind_Emblem),
        one(Ft_Kind_Pichu), one(Ft_Kind_Ganon), one(Ft_Kind_MasterH), one(Ft_Kind_Boy),
        one(Ft_Kind_Girl), one(Ft_Kind_GKoops), one(Ft_Kind_CrezyH), one(Ft_Kind_Sandbag),
        one(Ft_Kind_Popo),
    ];
    let mut i = 0;
    while i < rows.len() {
        t[i] = rows[i];
        i += 1;
    }
    t
}

/// `player_slots` (player.c .bss).
pub static mut player_slots: [StaticPlayer; PLAYER_NUM_MAX] = unsafe { core::mem::zeroed() };

/// `ftMapping_list[ChKind_Cap]` (TARGET_PC size; m-ex rows are written by `mex_set_mapping`).
pub static mut ftMapping_list: [FtMapping; ChKind_Cap as usize] = mapping_init();

/// The retail initial value of `ftMapping_list`.
pub const FT_MAPPING_INIT: [FtMapping; ChKind_Cap as usize] = mapping_init();

/// `&player_slots[slot]` after `Player_CheckSlot(slot)`.
#[inline]
pub fn sp(slot: i32) -> &'static mut StaticPlayer {
    Player_CheckSlot(slot);
    unsafe { &mut player_slots[slot as usize] }
}

/// `ftMapping_list[ckind]`.
#[inline]
pub fn map(ckind: CharacterKind) -> &'static mut FtMapping {
    unsafe { &mut ftMapping_list[ckind as usize] }
}

/// The entity `i` of the player: `player_entity[transformed[i]]`.
#[inline]
pub fn ent(p: &StaticPlayer, i: usize) -> *mut HSD_GObj {
    p.player_entity[p.transformed[i] as usize]
}

pub use crate::tu::player::Player_800325C8_fp as Player_800325C8;
pub use crate::tu::player::Player_CheckSlot_fp as Player_CheckSlot;
pub use crate::tu::player::Player_GetCoins_fp as Player_GetCoins;
pub use crate::tu::player::Player_GetDamage_fp as Player_GetDamage;
pub use crate::tu::player::Player_GetEntity_fp as Player_GetEntity;
pub use crate::tu::player::Player_GetEntityAtIndex_fp as Player_GetEntityAtIndex;
pub use crate::tu::player::Player_GetMoreFlagsBit2_fp as Player_GetMoreFlagsBit2;
pub use crate::tu::player::Player_GetNametagSlotID_fp as Player_GetNametagSlotID;
pub use crate::tu::player::Player_GetPlayerCharacter_fp as Player_GetPlayerCharacter;
pub use crate::tu::player::Player_GetPlayerId_fp as Player_GetPlayerId;
pub use crate::tu::player::Player_GetPlayerSlotType_fp as Player_GetPlayerSlotType;
pub use crate::tu::player::Player_GetPlayerState_fp as Player_GetPlayerState;
pub use crate::tu::player::Player_GetRemainingHP_fp as Player_GetRemainingHP;
pub use crate::tu::player::Player_GetStocks_fp as Player_GetStocks;
pub use crate::tu::player::Player_GetTeam_fp as Player_GetTeam;
pub use crate::tu::player::Player_LoadPlayerCoords_fp as Player_LoadPlayerCoords;
