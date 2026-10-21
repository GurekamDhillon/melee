//! Round 4: gm (and textlib) callers of rounds 1-3. Verified bit for bit against the C
//! (`tests/oracle_r4.rs`, oracle/gen_oracle_r4.py: the callees run as the game's own C there).
//! No fused site in any of these (`tools/rust-port/fma_sites.py`).
#![allow(non_snake_case)]

use crate::gm::bitset_mask;
use crate::gm_ptr::{fn_80165418, fn_801654A0, fn_80167194, gm_801B05F4, gm_801B07E8};
use crate::hud::DevText_Clamp;
use crate::lb::lbTime_8000AF74;
use melee_types as mt;
use mt::{CSSData, DevText, MatchEnd, PlayerInitData, ResultsData};

/// gmmain_lib.c `bitset_set`.
pub fn bitset_set(words: &mut [u32], bit: u32) {
    words[(bit / 32) as usize] |= bitset_mask(bit);
}

/// gmmain_lib.c `bitset_clear`.
pub fn bitset_clear(words: &mut [u32], bit: u32) {
    words[(bit / 32) as usize] &= !bitset_mask(bit);
}

/// gmmain_lib.c `bitset_test`: the masked word (not a bool).
pub fn bitset_test(words: &[u32], bit: u32) -> u32 {
    let mask = bitset_mask(bit);
    words[(bit / 32) as usize] & mask
}

/// gmmain_lib.c `bitset_test_word`: the same as `bitset_test`.
pub fn bitset_test_word(words: &[u32], bit: u32) -> u32 {
    let word = words[(bit / 32) as usize];
    word & bitset_mask(bit)
}

/// gm_1B03.c `gm_SetupHumanPlayer`.
pub fn gm_SetupHumanPlayer(player: &mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8) {
    player.slot_type = mt::Gm_PKind_Human as u8;
    player.ckind = ckind as i8;
    player.color = color;
    player.stocks = stocks as i8;
    gm_801B05F4(player, slot as i32);
}

/// gm_1B03.c `gm_SetupCpuPlayer`: as the human setup, then team 4.
pub fn gm_SetupCpuPlayer(player: &mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8) {
    player.slot_type = mt::Gm_PKind_Cpu as u8;
    player.ckind = ckind as i8;
    player.color = color;
    player.stocks = stocks as i8;
    gm_801B05F4(player, slot as i32);
    player.team = 4;
}

/// gm_1601.c `gm_MatchHasMultipleWinners`.
pub fn gm_MatchHasMultipleWinners(end: &MatchEnd) -> bool {
    if end.outcome as i32 != mt::OUTCOME_NO_CONTEST as i32 {
        let winners = if end.is_teams == 1 { end.n_team_winners as i32 } else { fn_80167194(end) };
        if winners > 1 {
            return true;
        }
    }
    false
}

/// gmtrainingmode.c `gm_801B07E8_layer` (static): forwards to gm_1B03's `gm_801B07E8`.
pub fn gm_801B07E8_layer(css_data: &CSSData, c_kind: Option<&mut i8>, stocks: Option<&mut i8>, color: Option<&mut i8>,
                         arg4: Option<&mut i8>, level: Option<&mut u8>) {
    gm_801B07E8(css_data, c_kind, stocks, color, arg4, level);
}

/// gmvsmelee.c `gmVsMelee_UpdateKOCounts`: per human player, `lbTime_8000AF74` of the count
/// and the standing's x20 (u8 -> u32 in, u32 -> int for x20, truncated back to u8).
pub fn gmVsMelee_UpdateKOCounts(ko_counts: &mut [u8], end: &MatchEnd) {
    for i in 0..mt::GM_MAX_PLAYERS as usize {
        if end.player_standings[i].pkind as i32 == mt::Gm_PKind_Human as i32 {
            ko_counts[i] = lbTime_8000AF74(ko_counts[i] as u32, end.player_standings[i].x20 as i32) as u8;
        }
    }
}

/// gmresult.c `fn_801771C0`: winner (x4), winning team (x5) and, in teams, the first player of
/// that team no teammate beats on `is_small_loser` (x6).
pub fn fn_801771C0(data: &mut ResultsData) {
    // SAFETY: x94 is the results' MatchEnd, set before this runs (the C dereferences it unchecked).
    let me = unsafe { &*data.x94 };
    data.x4 = fn_80165418(me) as u8;
    data.x5 = fn_801654A0(me) as u8;
    if me.is_teams == 1 {
        let ps = &me.player_standings;
        let na = mt::Gm_PKind_NA as i32;
        for i in 0..4usize {
            if ps[i].pkind as i32 != na && ps[i].team == data.x5 {
                let mut result = i as i32;
                for j in 0..4usize {
                    if ps[j].pkind as i32 != na && ps[j].team == data.x5 && i != j
                        && ps[i].is_small_loser > ps[j].is_small_loser
                    {
                        result = -1;
                        break;
                    }
                }
                if result >= 0 {
                    data.x6 = result as u8;
                    break;
                }
            }
        }
    } else {
        data.x6 = data.x4;
    }
}

/// textlib.c `DevText_SetCursorX`.
pub fn DevText_SetCursorX(text: &mut DevText, x: i32) {
    text.cursor_x = DevText_Clamp(x, text.w as i32) as u8;
}
