//! Native port of `db_80225DD8` (src/melee/db/dbitem.c) and its helper `fn_8022697C` (db/dbanim.c): the debug
//! item-collision-bubble flag byte. `db_ShowItemCollisionBubbles` is dbitem.c's file static.
//! Checked by tests/native_dbitem.rs.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

use super::ftlib::ftLib_80086960;
use melee_types::*;

/// `static int db_ShowItemCollisionBubbles` (dbitem.static.h).
pub static mut db_ShowItemCollisionBubbles: i32 = 0;

/// `fn_8022697C` (dbanim.c): the owner's x21FC flag byte, or 0 for a non-fighter.
pub unsafe fn fn_8022697C(owner: *mut HSD_GObj) -> u8 {
    if ftLib_80086960(owner) {
        let ft = (*owner).user_data as *mut Fighter;
        (*ft).x21FC_flag.byte
    } else {
        0
    }
}

pub unsafe fn db_80225DD8(item: *mut HSD_GObj, owner: *mut HSD_GObj) {
    let it = (*item).user_data as *mut Item;
    if !ftLib_80086960(owner) {
        (*it).anon_daa.xDAA_byte |= db_ShowItemCollisionBubbles as u8;
    } else {
        (*it).anon_daa.xDAA_byte &= 0xFC;
        let x = fn_8022697C(owner) as i32;
        if x != 0 {
            (*it).anon_daa.xDAA_byte |= (x & 3) as u8;
        }
    }
}

/// dbitem.static.h's `static struct { ... } db_ItemAndPokemonMenu` (the three `u32 : 1` bitfields kept as `u32`s,
/// only this module reads them).
#[repr(C)]
pub struct DbItemAndPokemonMenu {
    pub DisplayStatus: u32,
    pub DisplayFadeTimer: u32,
    pub ItemSpawnsEnabled: i32,
    pub Player: i32,
    pub CurrentlySelectedItem: i32,
    pub CurrentlySelectedPokemon: i32,
    pub LastSelectedItem: i32,
    pub LastSelectedPokemon: i32,
    pub ShowEnemyStompRange: u32,
    pub ShowItemPickupRange: u32,
    pub ShowCoinPickupRange: u32,
}

pub static mut db_ItemAndPokemonMenu: DbItemAndPokemonMenu = DbItemAndPokemonMenu {
    DisplayStatus: 0,
    DisplayFadeTimer: 0,
    ItemSpawnsEnabled: 0,
    Player: 0,
    CurrentlySelectedItem: 0,
    CurrentlySelectedPokemon: 0,
    LastSelectedItem: 0,
    LastSelectedPokemon: 0,
    ShowEnemyStompRange: 0,
    ShowItemPickupRange: 0,
    ShowCoinPickupRange: 0,
};

pub unsafe fn db_ShowEnemyStompRange() -> u32 {
    db_ItemAndPokemonMenu.ShowEnemyStompRange
}

pub unsafe fn db_ShowItemPickupRange() -> u32 {
    db_ItemAndPokemonMenu.ShowItemPickupRange
}

pub unsafe fn db_ShowCoinPickupRange() -> u32 {
    db_ItemAndPokemonMenu.ShowCoinPickupRange
}

pub unsafe fn db_GetCurrentlySelectedPokemon() -> i32 {
    db_ItemAndPokemonMenu.CurrentlySelectedPokemon
}

pub unsafe fn db_AreItemSpawnsEnabled() -> i32 {
    db_ItemAndPokemonMenu.ItemSpawnsEnabled
}

pub unsafe fn db_80225D64(item: *mut HSD_GObj, _owner: *mut HSD_GObj) {
    let it = (*item).user_data as *mut Item;
    (*it).anon_daa.xDAA_byte |= db_ShowItemCollisionBubbles as u8;
}
