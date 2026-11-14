//! The Envoy Item Lab (owner request 2026-11-13): a sandbox Envoy session where every item, modifier and effect can be
//! equipped, stacked, fired on demand and seen. Shared vocabulary of the three halves:
//! * the engine (melee_modes::tu::envoy `LAB`): the session's setup words, set by the UI before arming (menu-time
//!   state, read once by the sim at arm time like the saved run record), and the host-side time controls;
//! * envoy_sim: lab commands arrive as synced input (a serial stream on the reserved pad bits, `gd_ui_sim_cmd`), so
//!   rollback, resimulation and the synctest see them on the same frame; the sim publishes the lab words below;
//! * envoy_ui (lab.rs): the setup screen, the panel and the trigger log.
//! Everything item-shaped is read from `items::ITEMS` / `TRIGGER_TEXT` (nothing here names an item), so new items,
//! modifiers and triggers show up in the Lab without a change here.

use crate::items::*;

// ---- the setup words (melee_modes envoy LAB; gd_ui_envoy_lab / gd_envoy_lab)
pub const LAB_WORDS: usize = 16;
pub const L_ON: usize = 0; // 1: the armed session is a Lab session (nothing is saved or recorded)
pub const L_STAGE: usize = 1; // ground kind (0: the first Envoy stage)
pub const L_FOES: usize = 2; // FOE_*
pub const L_DUMMY: usize = 3; // DUMMY_* (FOE_DUMMY only)
pub const L_LEVEL: usize = 4; // CPU level 1-9
pub const L_FOE_CK: usize = 5; // the foe's character kind (-1: the run's pick)
pub const L_MAIN: usize = 6; // the player's fighter (character kind; -1 the CSS / picker)
pub const L_PARTNER: usize = 7;
pub const L_SUBSPACE: usize = 8; // the Subspace enemy kind for FOE_SUBSPACE (test_waves code: 100 + kind)

pub const FOE_NONE: i32 = 0;
pub const FOE_DUMMY: i32 = 1;
pub const FOE_CPU: i32 = 2;
pub const FOE_CROWD: i32 = 3;
pub const FOE_GAUNTLET: i32 = 4;
pub const FOE_SUBSPACE: i32 = 5;
pub const FOE_TEXT: [&str; 6] = ["None", "Dummy", "CPU fighter", "Crowd 1v3", "Gauntlet", "Subspace enemies"];

pub const DUMMY_STAND: i32 = 0;
pub const DUMMY_JUMP: i32 = 1;
pub const DUMMY_SHIELD: i32 = 2;
pub const DUMMY_CPU: i32 = 3;
pub const DUMMY_TEXT: [&str; 4] = ["Stand", "Jump", "Shield", "CPU level"];

// ---- time controls (host-side, never sim state: a skipped logic tick is simply not run)
pub const TIME_NORMAL: i32 = 0;
pub const TIME_HALF: i32 = 1;
pub const TIME_QUARTER: i32 = 2;
pub const TIME_PAUSED: i32 = 3;
pub const TIME_TEXT: [&str; 4] = ["Normal", "1/2 speed", "1/4 speed", "Paused (frame advance)"];

// ---- commands (24 bits: op << 16 | a << 8 | b), streamed on the pad's reserved bits
pub const LC_SET: i32 = 1; // a: item id, b: copies (0 removes)
pub const LC_ADD: i32 = 2; // a: item id (+1 copy)
pub const LC_SUB: i32 = 3; // a: item id (-1 copy)
pub const LC_ALL: i32 = 4; // one copy of every item (b: copies, 0 = 1)
pub const LC_CLEAR: i32 = 5; // no items
pub const LC_RANDOM: i32 = 6; // a seeded random loadout (a: salt, b: items)
pub const LC_FIRE: i32 = 7; // a: item id: its effect now, whatever its trigger
pub const LC_TRIGGER: i32 = 8; // a: trigger (TR_*): every held item on it fires now
pub const LC_PCT_ME: i32 = 9; // a | b << 8: percent for the players
pub const LC_PCT_FOE: i32 = 10; // a | b << 8: percent for every foe
pub const LC_RESET: i32 = 11; // positions and percents back to the start (respawn)
pub const LC_COINS: i32 = 12; // a | b << 8: the purse
pub const LC_LIMIT: i32 = 13; // a: the stock limit, b: stocks now (0: keep)
pub const LC_OFF: i32 = 14; // a: item id, b: 1 off / 0 on (a modifier or any item switched off)
pub const LC_CATS: i32 = 15; // a: the category bits switched off (CAT_*)
pub const LC_BREAK: i32 = 16; // a: 0 the player, 1 the first foe: the combo breaker fires now
pub const LC_CROWD: i32 = 17; // a: 0 off, 1 on (the crowd knockback on every foe hit)
pub const LC_SHIFT: i32 = 18; // a live stage transition (the rift sweep) to the next Envoy stage
pub const LC_ROOM: i32 = 19; // a: ROOM_*: preview a room
pub const LC_DAMAGE: i32 = 20; // a: percent added to every foe (a hit with no knockback)
pub const LC_ROLL_NAMES: [&str; 21] = [
    "", "set", "add", "sub", "all", "clear", "random", "fire", "trigger", "pct me", "pct foe", "reset", "coins", "limit",
    "off", "cats", "break", "crowd", "shift", "room", "damage",
];

pub const fn cmd(op: i32, a: i32, b: i32) -> u32 {
    ((op as u32 & 0xFF) << 16) | ((a as u32 & 0xFF) << 8) | (b as u32 & 0xFF)
}

/// The command stream's wire: on each logic input a port carries SIG_CLK (a bit is on the wire) and SIG_DATA (its
/// value). A command is a start bit (1) then CMD_BITS bits, high first; a frame without SIG_CLK resets the reader.
pub const SIG_CLK: u32 = 0x4000;
pub const SIG_DATA: u32 = 0x8000;
pub const CMD_BITS: u32 = 24;

// ---- categories (the effect toggles): what an item makes, by its tags
pub const CAT_PROJECTILE: i32 = 1;
pub const CAT_BURST: i32 = 2;
pub const CAT_TRAIL: i32 = 4;
pub const CAT_AURA: i32 = 8;
pub const CAT_TEXT: [&str; 4] = ["Projectiles", "Bursts", "Trails", "Auras"];

/// The categories item `it` belongs to (a modifier belongs to none: it is switched on its own).
pub const fn item_cats(it: &Item) -> i32 {
    let t = it.tags;
    (if t & (TG_PROJECTILE | TG_SPARK) != 0 { CAT_PROJECTILE } else { 0 })
        | (if t & (TG_BURST | TG_LAUNCH) != 0 { CAT_BURST } else { 0 })
        | (if t & TG_TRAIL != 0 { CAT_TRAIL } else { 0 })
        | (if t & TG_ORBITAL != 0 { CAT_AURA } else { 0 })
}

// ---- rooms the Lab can preview
pub const ROOM_ITEM: i32 = 1;
pub const ROOM_BOSS_DROP: i32 = 2;
pub const ROOM_CAMPFIRE: i32 = 3;
pub const ROOM_SHOP: i32 = 4;
pub const ROOM_DEVIL: i32 = 5;
pub const ROOM_ANGEL: i32 = 6;
pub const ROOM_SECRET: i32 = 7;
pub const ROOM_TEXT: [&str; 8] = ["", "Item room", "Boss drop", "Campfire", "Shop", "Devil room", "Angel room", "Secret room"];

// ---- the view words (envoy_data view: the top of the view, clear of every other block)
pub const V_LAB: usize = 1440;
pub const VL_ON: usize = V_LAB; // 1: a Lab session
pub const VL_OFF: usize = V_LAB + 1; // 2 words: item ids switched off (bit id, ids 0-63)
pub const VL_CATS: usize = V_LAB + 3; // categories switched off
pub const VL_CROWD: usize = V_LAB + 4; // 1: crowd knockback forced on
pub const VL_CMDS: usize = V_LAB + 5; // commands run so far
pub const VL_LAST: usize = V_LAB + 6; // the last command (24 bits)
pub const VL_FOES: usize = V_LAB + 7; // FOE_*
pub const VL_ROOMS: usize = V_LAB + 8; // bit r: room r can be previewed on this build
pub const VL_TLOG_N: usize = V_LAB + 9; // trigger log entries written (ring index = n % TLOG)
pub const VL_TLOG: usize = V_LAB + 12;
pub const TLOG: usize = 16;
pub const TL_W: usize = 4; // play frame, player, trigger (TR_*; LT_FIRE + id for an on-demand fire), items fired
pub const VL_END: usize = VL_TLOG + TLOG * TL_W; // 1516
/// A trigger log entry for a fire on demand (the Lab's button): LT_FIRE + item id.
pub const LT_FIRE: i32 = 1000;

const _: () = assert!(VL_END <= crate::view::VIEW_WORDS);
const _: () = assert!(crate::view::V_BURST_N < V_LAB);

/// The modifiers (TR_MODIFIER) that change item `it` (its tags meet theirs), as indices into ITEMS.
pub fn modifiers_of(it: &Item, out: &mut [usize]) -> usize {
    let mut n = 0;
    for (i, m) in ITEMS.iter().enumerate() {
        if m.trigger == TR_MODIFIER && m.mods & it.tags != 0 && n < out.len() {
            out[n] = i;
            n += 1;
        }
    }
    n
}

/// Number of triggers (TRIGGER_TEXT names every one; TR_MODIFIER is not something that happens).
pub const fn triggers() -> usize {
    TRIGGER_TEXT.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_item_has_a_category_or_is_a_modifier_or_passive() {
        for it in ITEMS.iter() {
            assert!(item_cats(it) != 0 || it.trigger == TR_MODIFIER || it.tags == 0, "{} has tags but no category", it.key);
        }
    }
    #[test]
    fn commands_pack() {
        assert_eq!(cmd(LC_SET, 12, 10), 0x01_0C0A);
        assert!(ITEM_IDS <= 64, "the off mask holds ids 0-63");
    }
    #[test]
    fn modifiers_find_their_items() {
        let mut o = [0usize; 8];
        let spark = item_by_key("spark_ring").unwrap();
        assert!(modifiers_of(spark, &mut o) >= 2);
    }
}
