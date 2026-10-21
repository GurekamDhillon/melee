//! Round 3: `src/melee/gm/` leaf functions that take pointers to game structs (no calls, no
//! globals). Same names as the C. The structs are the melee_types records (native layout, GC
//! offsets in their `GcLayout`). Checked by `tests/oracle_ptr.rs`: the C runs over a seeded
//! buffer viewed as its own struct type, the Rust over the same bytes viewed as the record,
//! and the return value and the bytes afterwards must match.
//!
//! Pointer conventions: a pointer the C dereferences is a reference here (a slice when the C
//! indexes it, `Option` when the C checks it for NULL); a pointer the C only stores or
//! compares stays a raw pointer.
#![allow(non_snake_case)]

use melee_types as mt;
use mt::{BracketEntry, CSSData, GameModeState, HSD_GObj, HSD_JObj, HSD_Text, HSD_psAppSRT, MatchEnd, PlayerInitData,
         StartMeleeData, TmData};

const CHKIND_NONE: i8 = mt::ChKind_None as i8;
const PKIND_NA: u8 = mt::Gm_PKind_NA as u8;
const PKIND_HUMAN: u8 = mt::Gm_PKind_Human as u8;
/// gm/forward.h `GM_NAMETAG_COUNT`.
pub const GM_NAMETAG_COUNT: u8 = 120;

/// `gmClassicMatchup` (gmclassic.c; the .c defines it, so melee_types only has it opaque).
/// TODO(group0): generate records defined in .c files.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct gmClassicMatchup {
    pub x00: u16,
    /// `union { s8 x02[3]; u8 x02_u8[3]; }`
    pub x02: [u8; 3],
    pub x05: u8,
}
melee_core::gc_layout!(gmClassicMatchup, "gmClassicMatchup", size 6, align 2, [
    x00 @ 0x0: 2 "u16",
    x02 @ 0x2: 3 "u8[3]",
    x05 @ 0x5: 1 "u8",
]);

/// gmcamera.c `gmCamera_801A2BF0_get_translate_x`.
pub fn gmCamera_801A2BF0_get_translate_x(px18: &i32) -> f32 {
    if *px18 != 0 { 5.0 } else { -5.0 }
}

/// gmclassic.c `gmClassic_GetMatchupCount`: entries up to the 0x148 terminator.
pub fn gmClassic_GetMatchupCount(matchups: &[gmClassicMatchup]) -> i32 {
    let mut count = 0usize;
    while matchups[count].x00 != 0x148 {
        count += 1;
    }
    count as i32
}

/// gmgiant.c `fn_801B9060` (Giant Melee player setup).
pub fn fn_801B9060(arg0: &mut PlayerInitData, _unused: *mut PlayerInitData) {
    arg0.model_scale = 1.8;
    arg0.defense_ratio = 1.0;
    arg0.attack_ratio = 1.5;
    arg0.xB = 2;
}

/// gmlightning.c `fn_801BA7AC` (Lightning Melee).
pub fn fn_801BA7AC(start: &mut StartMeleeData, _vs: *mut StartMeleeData) {
    start.rules.game_speed = 1.25;
}

/// gmmain_lib.c `gmMainLib_AdjustNameTag`.
pub fn gmMainLib_AdjustNameTag(tag_ptr: &mut u8, tag: u8) {
    if *tag_ptr == tag {
        *tag_ptr = GM_NAMETAG_COUNT;
    } else if *tag_ptr > tag && *tag_ptr != GM_NAMETAG_COUNT {
        *tag_ptr -= 1;
    }
}

/// gmmain_lib.c `gmMainLib_ClearNameTag`.
pub fn gmMainLib_ClearNameTag(tag_ptr: &mut u8, tag: u8) {
    if *tag_ptr == tag {
        *tag_ptr = 0;
    } else if *tag_ptr > tag && *tag_ptr != GM_NAMETAG_COUNT {
        *tag_ptr -= 1;
    }
}

/// gmmain_lib.c `gmMainLib_8015EF30`: clear a stats block.
pub fn gmMainLib_8015EF30(stats: &mut mt::GmStats) {
    stats.sd_count = 0;
    stats.attacks_hit = 0;
    stats.attacks_total = 0;
    stats.damage_dealt = 0;
    stats.damage_taken = 0;
    stats.peak_damage = 0;
    stats.match_count = 0;
    stats.victories = 0;
    stats.losses = 0;
    stats.play_time = 0;
    stats.total_player_count = 0;
    stats.damage_recovered = 0;
    stats.walk_distance = 0;
    stats.run_distance = 0;
    stats.fall_distance = 0;
    stats.peak_height = 0;
    stats.coins_swiped = 0;
    stats.coins_lost = 0;
    stats.coins_collected = 0;
}

/// gmregcommon.c `gm_8017BE8C`: opponents that are not ChKind_None (NUM_OPPONENTS = 3).
pub fn gm_8017BE8C(opps: &[i8]) -> i32 {
    opps[..3].iter().filter(|&&o| o != CHKIND_NONE).count() as i32
}

/// gmregenddisp.c `gm_801A9094_get_entry`.
pub fn gm_801A9094_get_entry(entries: &[i32], i: i32) -> i32 {
    entries[i as usize]
}

/// gmresult.c `gmResultFindNth`: index of the n-th nonzero value (256 if there is none).
pub fn gmResultFindNth(values: &[u8], mut n: i32) -> i32 {
    let mut i = 0;
    while i < 256 {
        if values[i as usize] != 0 {
            n = n.wrapping_sub(1);
            if n == 0 {
                break;
            }
        }
        i += 1;
    }
    i
}

/// gmresult.c `matchWasSkipped`.
pub fn matchWasSkipped(me: &MatchEnd) -> bool {
    me.outcome == mt::OUTCOME_NO_CONTEST as u8 || me.outcome == mt::OUTCOME_RETRY as u8
}

/// gmslomo.c `fn_801BA5B4` (Slo-Mo Melee).
pub fn fn_801BA5B4(data: &mut StartMeleeData, _unused: *mut StartMeleeData) {
    data.rules.game_speed = 0.5;
}

/// gmstaffroll.c `gm_801AB200_SetAppSRT`: copy a joint's world translation and 1.3x scale.
/// No fused site.
pub fn gm_801AB200_SetAppSRT(appsrt: &mut HSD_psAppSRT, jobj: &HSD_JObj) {
    appsrt.translate.x = jobj.mtx[0][3];
    appsrt.translate.y = jobj.mtx[1][3];
    appsrt.translate.z = jobj.mtx[2][3];
    appsrt.scale.x = 1.3f32 * jobj.scale.x;
    appsrt.scale.y = 1.3f32 * jobj.scale.y;
    appsrt.scale.z = 1.3f32 * jobj.scale.z;
}

/// gmsupersudden.c `fn_801B8C5C` (Super Sudden Death: 300%).
pub fn fn_801B8C5C(arg0: &mut PlayerInitData, _unused: *mut PlayerInitData) {
    arg0.damage1 = 0x12C;
}

/// gmtiny.c `fn_801B8E50` (Tiny Melee player setup).
pub fn fn_801B8E50(arg0: &mut PlayerInitData, _unused: *mut PlayerInitData) {
    arg0.model_scale = 0.35;
    arg0.defense_ratio = 0.5;
    arg0.attack_ratio = 1.0;
    arg0.xB = 1;
}

/// gmtoulib.c `gmTournament_SetBracketByes`.
pub fn gmTournament_SetBracketByes(entries: &mut [BracketEntry], entrant_count: i32) {
    match entrant_count {
        1 => entries[5].slots[1].x32 = 1,
        3 => {
            entries[10].slots[0].x32 = 1;
            entries[11].slots[1].x32 = 1;
        }
        5 => entries[23].slots[1].x32 = 1,
        7 => {
            entries[46].slots[1].x32 = 1;
            entries[47].slots[1].x32 = 1;
        }
        _ => {}
    }
}

/// gmtoulib.c `fn_8018B090_inline4`: first slot whose x4C is 0 (4 if none).
pub fn fn_8018B090_inline4(entry: &BracketEntry) -> i32 {
    entry.slots.iter().position(|s| s.x4C == 0).unwrap_or(4) as i32
}

/// gmtoulib.c `fn_8018F62C`: `(u32) gobj->user_data`.
pub fn fn_8018F62C(gobj: &HSD_GObj) -> u32 {
    gobj.user_data as usize as u32
}

/// gmtou_0.c `fn_80191240_save_root_jobj`.
pub fn fn_80191240_save_root_jobj(jobj: *mut HSD_JObj, jobj_copy: &mut *mut HSD_JObj) {
    *jobj_copy = jobj;
}

/// gmtou_0.c `fn_80191FD4_is_selected` (`hud` is an `enum CSSIconHud`).
pub fn fn_80191FD4_is_selected(hud: i32, slot: i32, tm: &TmData) -> bool {
    tm.x37[slot as usize].x3 as u32 == hud as u32
}

/// gmtou_0.c `fn_80195CCC_IsUniqueEntry`.
pub fn fn_80195CCC_IsUniqueEntry(tm: &TmData, count: i32, slot: i32, selected: i32) -> i32 {
    for i in 0..count.max(0) {
        if i != slot && selected == tm.x37[i as usize].x9 as i32 {
            return 0;
        }
    }
    1
}

/// gmtou_1.c `fn_80196564`.
pub fn fn_80196564(arg0: &TmData) -> bool {
    (arg0.stage_selection_type == 0 && arg0.x32 == 0) || arg0.stage_selection_type == 1
}

/// gmtou_1.c `fn_80196594`.
pub fn fn_80196594(arg0: &TmData) -> bool {
    (arg0.stage_selection_type == 2 && arg0.x32 == 0) || arg0.stage_selection_type == 3
}

/// gmvs.c `getPort`.
pub fn getPort(player: &PlayerInitData, i: i32) -> i32 {
    let slot = player.slot as i32;
    if slot == 0 { i } else { slot - 1 }
}

/// gmvsmelee.c `gmVsMelee_WasAnyPlayerHuman` (GM_MAX_PLAYERS = 6).
pub fn gmVsMelee_WasAnyPlayerHuman(end: &MatchEnd) -> bool {
    end.player_standings.iter().any(|p| p.pkind == PKIND_HUMAN)
}

/// gm_1601.c `fn_80161C90_count_players` (first 4 standings).
pub fn fn_80161C90_count_players(match_end: &MatchEnd) -> u32 {
    match_end.player_standings[..4].iter().filter(|p| p.pkind != 3).count() as u32
}

/// gm_1601.c `is_character_unlocked` (`n < 64`; Zelda/Sheik's 0xB is always unlocked).
pub fn is_character_unlocked(n: u8, mask: &u16) -> bool {
    n == 0xB || (*mask as i64 & (1i64 << n)) != 0
}

/// gm_1601.c `fn_80165418`: the last present player who is not a small loser.
pub fn fn_80165418(match_end: &MatchEnd) -> i32 {
    let mut winner = 0;
    for i in 0..4 {
        let p = &match_end.player_standings[i];
        if p.pkind != PKIND_NA && p.is_small_loser == 0 {
            winner = i as i32;
        }
    }
    winner
}

/// gm_1601.c `fn_801654A0`: the same over the 5 teams.
pub fn fn_801654A0(match_end: &MatchEnd) -> i32 {
    let mut winner = 0;
    for i in 0..5 {
        let t = &match_end.team_standings[i];
        if t.active != 0 && t.is_small_loser == 0 {
            winner = i as i32;
        }
    }
    winner
}

/// gm_1601.c `fn_80165548`.
pub fn fn_80165548(me: &MatchEnd, arg1: i32, team: i32) -> i32 {
    let ps = &me.player_standings;
    let mut ret = 0i32;
    if me.is_teams == 1 {
        for i in 0..4 {
            if ps[i].pkind != 3 && ps[i].team as i32 == team {
                let mut result = i as i32;
                for j in 0..4 {
                    if ps[j].pkind != 3 && ps[j].team as i32 == team && i != j
                        && ps[i].is_small_loser > ps[j].is_small_loser
                    {
                        result = -1;
                        break;
                    }
                }
                if result >= 0 {
                    ret = ret.wrapping_add(result);
                    break;
                }
            }
        }
    } else {
        ret = arg1;
    }
    ret
}

/// gm_1601.c `fn_801656A8`: a sort key for player `arg1` (`arg1 < 6`).
pub fn fn_801656A8(arg0: &MatchEnd, arg1: u32) -> u32 {
    let ps = &arg0.player_standings;
    let me = &ps[arg1 as usize];
    if me.pkind == 3 {
        return 0;
    }
    let cnt_gt = ps.iter().filter(|p| p.pkind != 3 && me.x20 > p.x20).count() as u32;
    let cnt_lt = ps.iter().filter(|p| p.pkind != 3 && (me.x24 as u32) < (p.x24 as u32)).count() as u32;
    let ret = cnt_gt.wrapping_mul(0x10);
    let ret = ret.wrapping_add(cnt_lt).wrapping_mul(0x10);
    ret.wrapping_add(6u32.wrapping_sub(arg1))
}

/// gm_1601.c `fn_80166CBC`.
pub fn fn_80166CBC(arg0: &MatchEnd, index: i32) -> u32 {
    arg0.player_standings[index as usize].is_small_loser as u32
}

/// `team_standings[i].active` for i up to 5. The C's second team loop in `gm_80166CCC` runs
/// to 6 over a 5-entry array, so it reads `team_standings[5].active`, a byte inside
/// `player_standings[0]`. MatchEnd has no pointer before player_standings, so that byte is
/// the same one on the GameCube and natively.
fn team_active(me: &MatchEnd, i: usize) -> u8 {
    let off = core::mem::offset_of!(MatchEnd, team_standings)
        + i * core::mem::size_of::<mt::MatchTeamData>()
        + core::mem::offset_of!(mt::MatchTeamData, active);
    debug_assert!(off < core::mem::size_of::<MatchEnd>());
    // SAFETY: `off` lies inside *me (checked above for i <= 5).
    unsafe { *(me as *const MatchEnd as *const u8).add(off) }
}

/// gm_1601.c `gm_80166CCC`: merge a sudden-death result (`arg1`) into the match result.
pub fn gm_80166CCC(arg0: &mut MatchEnd, arg1: &mut MatchEnd) {
    let result = arg1.outcome;
    let mut player_count = 0i32;
    let mut team_count = 0i32;
    if result == mt::OUTCOME_NO_CONTEST as u8 || result == mt::OUTCOME_RETRY as u8 {
        arg0.outcome = result;
    }
    if arg1.n_winners > 1 {
        for i in 0..6 {
            if arg1.player_standings[i].pkind != 3 {
                arg1.player_standings[i].is_big_loser = arg0.player_standings[i].is_big_loser
                    .wrapping_add(arg1.player_standings[i].is_small_loser);
            }
        }
    }
    if arg1.n_team_winners > 1 {
        for i in 0..5 {
            if arg1.team_standings[i].active != 0 {
                arg1.team_standings[i].is_big_loser = arg0.team_standings[i].is_big_loser
                    .wrapping_add(arg1.team_standings[i].is_small_loser);
            }
        }
    }
    for i in 0..6 {
        if arg1.player_standings[i].pkind != 3 {
            player_count += 1;
        }
    }
    if arg0.n_winners > 1 {
        for i in 0..6 {
            if arg0.player_standings[i].pkind != 3 {
                let b = &arg1.player_standings[i];
                let (b_ftkind, b_xe, b_pkind, b_big) = (b.ftkind, b.xE, b.pkind, b.is_big_loser);
                let a = &mut arg0.player_standings[i];
                a.ftkind = b_ftkind;
                a.xE = a.xE.wrapping_add(b_xe);
                if a.is_big_loser == 0 {
                    if b_pkind == 3 {
                        a.is_big_loser = a.is_big_loser.wrapping_add(player_count as u8);
                        a.is_small_loser = a.is_small_loser.wrapping_add(player_count as u8);
                    } else {
                        a.is_big_loser = b_big;
                        a.is_small_loser = b_big;
                    }
                }
            }
        }
    }
    for i in 0..6 {
        if team_active(arg1, i) != 0 {
            team_count += 1;
        }
    }
    if arg0.n_team_winners > 1 {
        for i in 0..5 {
            if arg0.team_standings[i].active != 0 && arg0.team_standings[i].is_big_loser == 0 {
                let (b_active, b_big) = (arg1.team_standings[i].active, arg1.team_standings[i].is_big_loser);
                let a = &mut arg0.team_standings[i];
                if b_active != 0 {
                    a.is_big_loser = b_big;
                    a.is_small_loser = b_big;
                } else {
                    a.is_big_loser = a.is_big_loser.wrapping_add(team_count as u8);
                    a.is_small_loser = a.is_small_loser.wrapping_add(team_count as u8);
                }
            }
        }
    }
}

/// gm_1601.c `fn_80167194`.
pub fn fn_80167194(me: &MatchEnd) -> i32 {
    me.n_winners as i32
}

/// gm_1601.c `gm_801677C0`.
pub fn gm_801677C0(arg0: &mut mt::gm_801677C0_s) {
    arg0.x0 = 0;
    arg0.x1 = 0;
    arg0.x2 = 0;
    arg0.x3 = 0;
    arg0.unk_28 = 0;
    arg0.unk_20 = 0;
}

/// gm_1601.c `gm_80168940`.
pub fn gm_80168940(match_end: &MatchEnd) -> i32 {
    if match_end.player_standings[0].pkind == 0 {
        return match_end.player_standings[0].xE as i32;
    }
    0
}

/// gm_1798.c `get_big_loser`.
pub fn get_big_loser(slot: i32, match_end: &MatchEnd) -> i32 {
    match_end.player_standings[slot as usize].is_big_loser as i32
}

/// gm_17C0.c `gm_8017CE34_CountEnemies` (TARGET_PC: compares with ChKind_None, per m-ex).
pub fn gm_8017CE34_CountEnemies(arg0: &[i8]) -> i32 {
    arg0[..3].iter().filter(|&&c| c as i32 != mt::ChKind_None as i32).count() as i32
}

/// gm_17C0.c `gm_8017CE34_GetCpuLevel`.
pub fn gm_8017CE34_GetCpuLevel(arg1: &mt::Unk1PData) -> u8 {
    arg1.x0.cpu_level
}

/// gm_17C0.c `gm_8017DB58`: clear three enemy entries.
pub fn gm_8017DB58(arg0: &mut [mt::Unk1PData_x24]) {
    for e in &mut arg0[..3] {
        e.ckind = CHKIND_NONE;
    }
}

/// gm_17C0.c `gm_8017DB6C`.
pub fn gm_8017DB6C(arg0: &[mt::gm_8017DB6C_arg0_t], index: i32) -> u8 {
    arg0[index as usize].x0
}

/// gm_17C0.c `gm_8017DB78`.
pub fn gm_8017DB78(arg0: &[mt::gm_8017DB6C_arg0_t], index: i32) -> u8 {
    arg0[index as usize].x1
}

/// gm_181A.c `fn_80181E18_ComputeRemaining100`.
pub fn fn_80181E18_ComputeRemaining100(x4: &i32, count: i32) -> i32 {
    0x64i32.wrapping_sub(count.wrapping_add(*x4))
}

/// gm_181A.c `gm_80182578_GetIndexFromPointer`.
pub fn gm_80182578_GetIndexFromPointer(idx_ptr: &i32) -> i32 {
    *idx_ptr
}

/// gm_1884.c `resetText`.
pub fn resetText(text: &mut HSD_Text) {
    text.x34.x = 0.7;
    text.x34.y = 0.6;
}

/// gm_1A3F.c `firstState`. The C's `break` only leaves its `do {} while (0)`, so the loop
/// always returns on its first pass: the first state's id, or 0 if the table starts with the
/// 0xFF terminator. `next_id` has no effect.
pub fn firstState(state: &[GameModeState], _next_id: u8) -> u8 {
    if state[0].id != 0xFF { state[0].id } else { 0 }
}

/// gm_1B03.c `player_standings_inline`.
pub fn player_standings_inline(arg0: &mut StartMeleeData, arg1: &MatchEnd, i: i32, is_big_loser: u32, var_r7: i32) {
    let i = i as usize;
    if is_big_loser == 0 && var_r7 > 0 {
        let mut ckind = arg1.player_standings[i].ckind;
        if ckind == 0x12 || ckind == 0x13 {
            ckind = if arg1.player_standings[i].ftkind == 7 { 0x13 } else { 0x12 };
        }
        arg0.players[i].ckind = ckind;
        arg0.players[i].stocks = 1;
        arg0.players[i].damage1 = 300;
    } else {
        arg0.players[i].slot_type = PKIND_NA;
    }
}

/// gm_1B03.c `gm_801B0474_inline`.
pub fn gm_801B0474_inline(arg1: &MatchEnd, i: i32) -> i32 {
    if arg1.match_kind == 1 {
        let p = &arg1.player_standings[i as usize];
        if arg1.outcome == mt::OUTCOME_TIMEOUT as u8 {
            return p.stocks as i32;
        }
        let stocks = p.stocks as u8;
        if p.x28 < arg1.frame_count || stocks != 0 {
            return stocks as i32;
        }
    }
    1
}

/// gm_1B03.c `gm_801B05F4`: port slot and team (slots 2 and 3 swap teams).
pub fn gm_801B05F4(player: &mut PlayerInitData, mut slot: i32) {
    player.slot = slot.wrapping_add(1) as u8;
    if slot == 2 {
        slot = 3;
    } else if slot == 3 {
        slot = 2;
    }
    player.team = slot as u8;
}

/// gm_1B03.c `gm_801B0730`: read the CSS player's choices (`unk_0x0` is the 1-based slot).
pub fn gm_801B0730(css_data: &CSSData, c_kind: Option<&mut i8>, stocks: Option<&mut u8>, color: Option<&mut u8>,
                   nametag: Option<&mut u8>, level: Option<&mut u8>) {
    let p = &css_data.vs.start.players[(css_data.unk_0x0 as i32 - 1) as usize];
    if let Some(c) = c_kind {
        *c = p.ckind;
    }
    if let Some(s) = stocks {
        *s = p.stocks as u8;
    }
    if let Some(c) = color {
        *c = p.color;
    }
    if let Some(l) = level {
        *l = p.cpu_level;
    }
    if let Some(n) = nametag {
        *n = p.nametag;
    }
}

/// gm_1B03.c `gm_801B07B4`: write the opponent's choices (slot 1, or 0 when `arg6 != 0`).
pub fn gm_801B07B4(css_data: &mut CSSData, c_kind: i8, stocks: i8, color: i8, arg4: u8, level: u8, arg6: u8) {
    let p = &mut css_data.vs.start.players[if arg6 == 0 { 1 } else { 0 }];
    p.ckind = c_kind;
    p.stocks = stocks;
    p.color = color as u8;
    p.cpu_level = level;
    p.nametag = arg4;
}

/// gm_1B03.c `gm_801B07E8`: read the opponent's choices.
pub fn gm_801B07E8(css_data: &CSSData, c_kind: Option<&mut i8>, stocks: Option<&mut i8>, color: Option<&mut i8>,
                   arg4: Option<&mut i8>, level: Option<&mut u8>) {
    let slot = if css_data.unk_0x0 as i32 - 1 == 0 { 1 } else { 0 };
    let p = &css_data.vs.start.players[slot];
    if let Some(c) = c_kind {
        *c = p.ckind;
    }
    if let Some(s) = stocks {
        *s = p.stocks;
    }
    if let Some(c) = color {
        *c = p.color as i8;
    }
    if let Some(l) = level {
        *l = p.cpu_level;
    }
    if let Some(n) = arg4 {
        *n = p.nametag as i8;
    }
}
