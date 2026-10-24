//! The player table of `src/melee/pl/player.c` that the stage, HUD and match code read: the
//! `player_slots` array, the `ftMapping_list` character-to-fighter table, `Player_CheckSlot` and the
//! plain field accessors (`Player_GetTeam`, `Player_GetEntity`, ...). They live here, in the lowest
//! crate that needs them, because the world TUs call them and the fighter crate (which depends on
//! this one) owns the rest of player.c; `melee_fighter::{stale, player, tu::player}` re-export these
//! items, so there is a single copy of the table and of each accessor.
//!
//! The functions that go through a fighter (`Player_8003219C` -> `ftLib_8008701C`, ...) stay in
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

const NONE: i8 = -1;
const fn m(a: FighterKind, b: i8, t: i8) -> FtMapping {
    FtMapping { internal_id: a as i8, extra_internal_id: b, has_transformation: t }
}
const fn one(a: FighterKind) -> FtMapping {
    m(a, NONE, 0)
}

const fn mapping_init() -> [FtMapping; ChKind_Cap as usize] {
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

/// `Player_CheckSlot`: `HSD_ASSERTREPORT(102, 0, "cant get player struct! %d\n", slot)`. The C
/// reports and carries on; an out-of-range slot is a bug either way, so this panics.
#[inline]
pub fn Player_CheckSlot(slot: i32) {
    if slot < 0 || !(slot < Gm_Player_NumMax as i32) {
        panic!("src/melee/pl/player.c:102: cant get player struct! {}", slot);
    }
}

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

/// `Player_GetPlayerSlotType`.
pub fn Player_GetPlayerSlotType(slot: i32) -> Gm_PKind {
    sp(slot).pkind
}

/// `Player_GetEntity`.
pub fn Player_GetEntity(slot: i32) -> *mut HSD_GObj {
    let player = sp(slot);
    ent(player, 0)
}

/// `Player_GetEntityAtIndex`.
pub fn Player_GetEntityAtIndex(slot: i32, index: i32) -> *mut HSD_GObj {
    let player = sp(slot);
    let entity_index = player.transformed[index as usize] as usize;
    player.player_entity[entity_index]
}

/// `Player_LoadPlayerCoords`.
pub fn Player_LoadPlayerCoords(slot: i32, arg_vec: *mut Vec) {
    let player = sp(slot);
    unsafe { *arg_vec = player.player_poses.byIndex[player.transformed[0] as usize] };
}

/// `Player_GetTeam`.
pub fn Player_GetTeam(slot: i32) -> i32 {
    sp(slot).team as i32
}

/// `Player_GetStocks`.
pub fn Player_GetStocks(slot: i32) -> i32 {
    sp(slot).stocks as i32
}

/// `Player_GetPlayerCharacter`.
pub fn Player_GetPlayerCharacter(slot: i32) -> CharacterKind {
    sp(slot).ckind
}

/// `Player_GetPlayerId`.
pub fn Player_GetPlayerId(slot: i32) -> i32 {
    sp(slot).player_id as i32
}

/// `Player_GetNametagSlotID`.
pub fn Player_GetNametagSlotID(slot: i32) -> u8 {
    sp(slot).nametag_slot_id as u8
}

/// `Player_GetPlayerState`.
pub fn Player_GetPlayerState(slot: i32) -> i32 {
    sp(slot).player_state
}

/// `Player_GetDamage`.
pub fn Player_GetDamage(slot: i32) -> i32 {
    let player = sp(slot);
    unsafe { player.staminas.byIndex[player.transformed[0] as usize] as i32 }
}

/// `Player_GetMoreFlagsBit2`.
pub fn Player_GetMoreFlagsBit2(slot: i32) -> bool {
    sp(slot).more_flags.b2() != 0
}

/// `Player_GetRemainingHP`.
pub fn Player_GetRemainingHP(slot: i32) -> i32 {
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

/// `Player_800325C8`: the fighter kind of a character (`b == 0`) or its extra fighter (`b == 1`).
pub fn Player_800325C8(kind: CharacterKind, b: i32) -> i8 {
    if b == 0 {
        return map(kind).internal_id;
    } else if b == 1 {
        return map(kind).extra_internal_id;
    }
    -1
}

/// `Player_GetCoins`.
pub fn Player_GetCoins(slot: i32) -> i32 {
    sp(slot).current_coins
}
