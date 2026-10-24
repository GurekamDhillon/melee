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
