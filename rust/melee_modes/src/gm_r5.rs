//! Round 5: gm leaf functions, verified by `tests/oracle_r5.rs`.
use melee_types::VsModeData;

/// gmmain_lib.c `SetDefaultHandicaps`: every player's handicap to 9 (GM_MAX_PLAYERS = 6).
pub fn SetDefaultHandicaps(mode: &mut VsModeData) {
    for p in mode.start.players.iter_mut().take(6) {
        p.handicap = 9;
    }
}
