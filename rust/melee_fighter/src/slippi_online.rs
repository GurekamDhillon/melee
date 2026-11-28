//! Slippi online's desync checksum over the live fighters (FN_COMPUTE_CHECKSUM, slippi-online-patches.md 2.2).
//! Read-only; only called when Slippi online mode is on.

use melee_core::slippi_online::{Checksum, FighterSample};
use melee_types::Fighter;

/// The checksum of the current state: player slots 0..3, raw entities `player_entity[0]` then `[1]` (not through
/// `transformed`), then each slot's stocks byte.
/// The checksum's inputs as one text line (`slippi-netplay --trace`): per present entity, slot motion x y percent spawn.
pub fn trace_line() -> String {
    let mut t = String::new();
    for slot in 0..4 {
        let p = unsafe { &*core::ptr::addr_of!(crate::stale::PLAYER_SLOTS[slot]) };
        for g in p.player_entity {
            if g.is_null() {
                continue;
            }
            let fp = unsafe { &*((*g).user_data as *const Fighter) };
            t += &format!("{slot}:{:x} {} {} {} {} ", fp.motion_id, fp.cur_pos.x, fp.cur_pos.y, fp.dmg.x1830_percent, fp.x8_spawnNum);
        }
    }
    t
}

/// The current state as one line per present fighter (slot, position, percent, action state) and each slot's stocks:
/// the desync report's dump (not the state of the compared frame: the session may be ahead of it).
pub fn state_dump() -> String {
    let mut t = String::new();
    for slot in 0..4 {
        let p = unsafe { &*core::ptr::addr_of!(crate::stale::PLAYER_SLOTS[slot]) };
        let mut any = false;
        for (i, g) in p.player_entity.into_iter().enumerate() {
            if g.is_null() {
                continue;
            }
            any = true;
            let fp = unsafe { &*((*g).user_data as *const Fighter) };
            t += &format!("p{slot}.{i} x {:.3} y {:.3} pct {:.1} action {:#x}; ", fp.cur_pos.x, fp.cur_pos.y, fp.dmg.x1830_percent, fp.motion_id);
        }
        if any {
            t += &format!("p{slot} stocks {}; ", p.stocks);
        }
    }
    t
}

/// [`checksum`] over the fighters a replay records only: each slot's active entity (`player_entity[transformed[0]]`)
/// and an Ice Climbers' Nana. Zelda/Sheik's inactive form is in Slippi's checksum but has no Post-Frame event, so
/// tools that rebuild the checksum from a .slp (tools/rust-port/slippi/hard_desync.py) compare against this one.
pub fn checksum_recorded() -> u32 {
    checksum_recorded_with_mask(true).0
}

/// [`checksum_recorded`] and the fighters it covered: bit `slot * 2 + entity` (entity 1 = Nana / the sub entity).
/// `with_nana = false` leaves Nana out: a replay stops recording her once she is dead, while her entity lives on.
pub fn checksum_recorded_with_mask(with_nana: bool) -> (u32, u8) {
    let mut c = Checksum::new();
    let mut mask = 0u8;
    for slot in 0..4 {
        let p = unsafe { &*core::ptr::addr_of!(crate::stale::PLAYER_SLOTS[slot]) };
        let active = p.transformed[0] as usize;
        for (i, g) in p.player_entity.into_iter().enumerate() {
            if g.is_null() {
                continue;
            }
            let fp = unsafe { &*((*g).user_data as *const Fighter) };
            let nana = fp.kind == crate::types::Ft_Kind_Nana as _;
            if (i != active && !nana) || (nana && !with_nana) {
                continue;
            }
            mask |= 1 << (slot * 2 + i); // semantics: allow(host: replay-check port mask, not game arithmetic)
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
    (c.finish(), mask)
}

pub fn checksum() -> u32 {
    let mut c = Checksum::new();
    for slot in 0..4 {
        let p = unsafe { &*core::ptr::addr_of!(crate::stale::PLAYER_SLOTS[slot]) };
        for g in p.player_entity {
            if g.is_null() {
                continue;
            }
            let fp = unsafe { &*((*g).user_data as *const Fighter) };
            if std::env::var_os("MELEE_SLIPPI_CK_TRACE").is_some() {
                eprintln!("slippi-ck: slot {slot} motion {:#x} pos {:08x} {:08x} pct {:08x} spawn {}", fp.motion_id, fp.cur_pos.x.to_bits(), fp.cur_pos.y.to_bits(), fp.dmg.x1830_percent.to_bits(), fp.x8_spawnNum);
            }
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

/// The desync-recovery record of the current frame (StartEngineLoop.asm: `0x8046B6C8` seconds remaining, then per slot
/// `PlayerBlock_LoadStocksLeft` and `PlayerBlock_LoadDamage`).
pub fn synced_state() -> melee_core::slippi_online::SyncedState {
    use melee_core::slippi_online::{SyncedFighter, SyncedState};
    let mut s = SyncedState { seconds: unsafe { (*core::ptr::addr_of!(melee_runtime::tu::gmvs_state::controller)).state.timer_seconds }, ..Default::default() };
    for slot in 0..4 {
        s.fighters[slot] = SyncedFighter { stocks: crate::stale::stocks(slot as i32).clamp(0, 255) as u8, percent: crate::player::get_damage(slot as i32).clamp(0, 0xFFFF) as u16 };
    }
    s
}

/// `FN_END_GAME` (StartEngineLoop.asm): the writes that make the game's own end-of-match check end the game as a
/// quit-out (LRAS) by player `pauser`, with a 90-frame GAME! screen. Idempotent; the caller repeats it every frame
/// while the game is ending (a rollback load rewinds these statics).
pub fn end_game_now(pauser: i8) {
    unsafe {
        let c = &mut *core::ptr::addr_of_mut!(melee_runtime::tu::gmvs_state::controller);
        c.state.pauser = pauser;
        c.state.match_result = melee_core::slippi_online::END_GAME_EXIT_LRAS;
        c.start.xD = melee_core::slippi_online::END_GAME_THINK_FRAMES;
    }
}

/// A recovery game's starting state (`prepareOnlineMatchState`: the match block's per-slot percent): the slots' damage
/// and their fighters' percent, before the first frame.
pub fn set_start_percent(slot: i32, percent: u16) {
    crate::player::set_hud_damage(slot, percent as i32);
}

/// A recovery game's timer: the match block's timer word is `StartMeleeRules::time_limit` (seconds); the scene's rules
/// are whole minutes and the scene's start copied them already, so the synced seconds are written over the copy (and the
/// running timer) before the first frame.
pub fn set_start_seconds(seconds: u32) {
    unsafe {
        let c = &mut *core::ptr::addr_of_mut!(melee_runtime::tu::gmvs_state::controller);
        c.start.time_limit = seconds;
        c.state.timer_seconds = seconds;
    }
}
