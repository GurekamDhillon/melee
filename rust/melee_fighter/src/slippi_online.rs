//! Slippi online's desync checksum over the live fighters (FN_COMPUTE_CHECKSUM, slippi-online-patches.md 2.2).
//! Read-only; only called when Slippi online mode is on.

use melee_core::slippi_online::{Checksum, FighterSample};
use melee_types::Fighter;

/// The checksum of the current state: player slots 0..3, raw entities `player_entity[0]` then `[1]` (not through
/// `transformed`), then each slot's stocks byte.
pub fn checksum() -> u32 {
    let mut c = Checksum::new();
    for slot in 0..4 {
        let p = unsafe { &*core::ptr::addr_of!(crate::stale::PLAYER_SLOTS[slot]) };
        for g in p.player_entity {
            if g.is_null() {
                continue;
            }
            let fp = unsafe { &*((*g).user_data as *const Fighter) };
            c.fighter(&FighterSample {
                motion_id: fp.motion_id as u32,
                pos_x: fp.cur_pos.x,
                pos_y: fp.cur_pos.y,
                percent: fp.dmg.x1830_percent,
                spawn_num: fp.x8_spawnNum as u32,
            });
        }
        c.slot_end(p.stocks as u8);
    }
    c.finish()
}
