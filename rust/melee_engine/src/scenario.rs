//! The scene launch configuration (`MELEE_SCENE`, pc/platform/gw_runtime.c) as the engine's own answer to the
//! `SceneLaunch_*` queries that `SceneLaunch_SeedVs` (melee_modes::tu::gmscenelaunch, called from the VS mode's
//! `on_load`) reads. This is the platform half of the launcher: no game logic, just the configured values.

use melee_modes::tu::gmscenelaunch::ext::hook as h;

/// CharacterKind (gr/forward.h order), the launcher's `GW_SL_CK_*`.
pub fn ckind_by_name(name: &str) -> Option<i32> {
    Some(match name {
        "falcon" | "captain" => 0,
        "dk" | "donkey" => 1,
        "fox" => 2,
        "gw" => 3,
        "kirby" => 4,
        "bowser" | "koopa" => 5,
        "link" => 6,
        "luigi" => 7,
        "mario" => 8,
        "marth" | "mars" => 9,
        "mewtwo" => 10,
        "ness" => 11,
        "peach" => 12,
        "pikachu" => 13,
        "ics" | "popo" => 14,
        "jigglypuff" | "purin" => 15,
        "samus" => 16,
        "yoshi" => 17,
        "zelda" => 18,
        "sheik" | "seak" => 19,
        "falco" => 20,
        "younglink" | "clink" => 21,
        "drmario" => 22,
        "roy" | "emblem" => 23,
        "pichu" => 24,
        "ganon" => 25,
        _ => return None,
    })
}

/// External StKind of the launcher's stage names.
pub fn stage_by_name(name: &str) -> Option<i32> {
    Some(match name {
        "fd" | "final" | "finaldestination" => 32,
        "bf" | "battle" | "battlefield" => 31,
        "dl" | "dreamland" | "greens" => 17,
        "ys" | "story" | "yoshistory" => 8,
        "fod" | "izumi" | "fountain" => 2,
        "ps" | "pstadium" | "stadium" => 3,
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerSpec {
    pub ckind: i32,
    /// `Gm_PKind`: 0 human, 1 CPU.
    pub slot_type: i32,
    pub cpu_level: i32,
    /// costume index (-1 = the launcher's default, the slot number)
    pub color: i32,
    /// team id (-1 = unset)
    pub team: i32,
    /// starting stocks (-1 = unset)
    pub stocks: i32,
    /// handicap (-1 = unset)
    pub handicap: i32,
}

#[derive(Clone, Debug)]
pub struct Scenario {
    pub stage: i32,
    pub players: [Option<PlayerSpec>; 6],
    /// 0 time, 1 stock
    pub rule_match: i32,
    pub rule_minutes: i32,
    /// `items=` value (-2 = off, else the frequency index), -1 leaves it alone
    pub item_freq: i32,
    /// `is_teams` (-1 = leave alone)
    pub teams: i32,
    /// timer in seconds, 0 = no timer (-1 = leave alone)
    pub time_limit: i32,
    /// rule stock count (-1 = unset)
    pub rule_stocks: i32,
    /// `mode=` as a GameModeKind for the 1P modes (`classic` 3; -1 = the VS match), gw_sl_cfg.game_mode
    pub onep_mode: i32,
    /// `step=` (gw_sl_cfg.onep_step, -1 unset) and `difficulty=` (onep_level, -1 unset)
    pub onep_step: i32,
    pub onep_level: i32,
}

impl Scenario {
    /// The `record.sh` scene strings: `p1=falcon/human;p2=fox/human;stage=fd;match=time;minutes=0;items=off`.
    pub fn parse(scene: &str) -> Result<Scenario, String> {
        let mut s = Scenario { stage: -1, players: [None; 6], rule_match: -1, rule_minutes: -1, item_freq: -1, teams: -1, time_limit: -1, rule_stocks: -1, onep_mode: -1, onep_step: -1, onep_level: -1 };
        for kv in scene.split(';') {
            let Some((k, v)) = kv.split_once('=') else { continue };
            match k {
                "stage" => {
                    s.stage = match v.strip_prefix("ext:") {
                        Some(n) => n.parse().map_err(|_| format!("stage {v}"))?,
                        None => stage_by_name(v).ok_or(format!("unknown stage {v}"))?,
                    }
                }
                // coverage scenarios (scenarios.tsv): modes and rules the engine's launcher does not answer yet
                "teams" | "stocks" => return Err(format!("scene key {k} not wired")),
                // gw_sl_modes (pc/platform/gw_runtime.c): the 1P modes a scene launches through their OnLoad (gm_OnePSceneDirect)
                "mode" => s.onep_mode = match v { "classic" => 3, "adventure" => 4, "allstar" => 5, _ => -1 },
                "step" => s.onep_step = v.parse().map_err(|_| "step")?,
                "difficulty" => s.onep_level = v.parse().map_err(|_| "difficulty")?,
                "match" => s.rule_match = if v == "stock" { 1 } else { 0 },
                "minutes" => s.rule_minutes = v.parse().map_err(|_| "minutes")?,
                "items" => s.item_freq = if v == "off" { -2 } else { v.parse().map_err(|_| "items")? },
                _ if k.len() == 2 && k.starts_with('p') => {
                    let n = (k.as_bytes()[1] - b'1') as usize;
                    if v == "none" || n >= 6 {
                        continue;
                    }
                    let mut it = v.split('/');
                    let name = it.next().unwrap();
                    let ck = match name.strip_prefix("ck:") {
                        Some(n) => n.parse().map_err(|_| format!("fighter {v}"))?,
                        None => ckind_by_name(name).ok_or(format!("unknown fighter in {v}"))?,
                    };
                    // `<fighter>[/<kind>][/cN][/stocksN]` (gw_runtime.c's 1P launch: color and stocks of the player)
                    let (mut kind, mut color, mut stocks) = ("human", -1, -1);
                    for part in it {
                        if let Some(n) = part.strip_prefix("stocks") {
                            stocks = n.parse().map_err(|_| format!("stocks in {v}"))?;
                        } else if let Some(n) = part.strip_prefix('c').filter(|n| n.parse::<i32>().is_ok()) {
                            color = n.parse().unwrap();
                        } else if part == "human" || part == "idle" || part.starts_with("cpu") {
                            kind = part;
                        } else {
                            return Err(format!("team suffix in {v} not wired"));
                        }
                    }
                    let kind = if kind == "idle" { "human" } else { kind };
                    let (slot_type, cpu_level) = match kind.strip_prefix("cpu") {
                        Some(l) => (1, l.parse().unwrap_or(-1)),
                        None => (0, -1),
                    };
                    s.players[n] = Some(PlayerSpec { ckind: ck, slot_type, cpu_level, color, team: -1, stocks, handicap: -1 });
                }
                _ => {}
            }
        }
        Ok(s)
    }
}

static mut SCN: Option<Scenario> = None;

fn p(n: i32) -> Option<PlayerSpec> {
    unsafe { SCN.as_ref().and_then(|s| s.players.get(n as usize).copied().flatten()) }
}

/// The mode a scenario boots into: its 1P mode, else VS.
pub fn boot_mode(s: &Scenario) -> u8 {
    if s.onep_mode >= 0 { s.onep_mode as u8 } else { melee_types::GM_VS as u8 }
}

/// Make the 1P modes' `SceneLaunch_OneP*` queries answer for `s` (gw_runtime.c gw_SceneLaunch_OnePDirect / Step / Level).
fn install_onep() {
    unsafe {
        use melee_modes::tu::gmclassic::ext::hook as c;
        c::SceneLaunch_OnePDirect = Some(|m| SCN.as_ref().map_or(0, |s| (s.onep_mode >= 0 && s.onep_mode == m && p(0).is_some_and(|x| x.ckind >= 0)) as i32));
        c::SceneLaunch_OnePStep = Some(|| {
            let Some(s) = SCN.as_ref() else { return 0 };
            let step = s.onep_step.max(0);
            // gm_Mode_*_States: Classic 0..10, Adventure 0..11, All-Star 0..12
            let max = match s.onep_mode { 3 => 10, 4 => 11, 5 => 12, _ => return step };
            step.min(max)
        });
        c::SceneLaunch_OnePLevel = Some(|| SCN.as_ref().map_or(2, |s| if s.onep_level < 0 { 2 } else { s.onep_level }));
        c::SceneLaunch_PlayerCKind = Some(|n| p(n).map_or(-1, |x| x.ckind));
        c::SceneLaunch_PlayerColor = Some(|n| p(n).map_or(-1, |x| x.color));
        c::SceneLaunch_PlayerStocks = Some(|n| p(n).map_or(-1, |x| x.stocks));
    }
}

/// Make `SceneLaunch_*` answer for `s`, as `MELEE_SCENE` does in the C build (entry state 2 = the match).
pub fn install(s: Scenario) {
    if s.onep_mode >= 0 {
        unsafe { SCN = Some(s) };
        install_onep();
        return;
    }
    unsafe {
        SCN = Some(s);
        h::SceneLaunch_Active = Some(|| 1);
        h::SceneLaunch_VsModeIndex = Some(|| melee_types::GmVsMode_Melee as i32);
        h::SceneLaunch_EntryStateId = Some(|| 2);
        h::SceneLaunch_StageExternal = Some(|| SCN.as_ref().map_or(-1, |s| s.stage));
        h::SceneLaunch_PlayerCKind = Some(|n| p(n).map_or(-1, |x| x.ckind));
        h::SceneLaunch_PlayerSlotType = Some(|n| p(n).map_or(-1, |x| x.slot_type));
        h::SceneLaunch_PlayerCpuLevel = Some(|n| p(n).map_or(-1, |x| x.cpu_level));
        h::SceneLaunch_RuleMatch = Some(|| SCN.as_ref().map_or(-1, |s| s.rule_match));
        h::SceneLaunch_RuleMinutes = Some(|| SCN.as_ref().map_or(-1, |s| s.rule_minutes));
        h::SceneLaunch_PlayerColor = Some(|n| p(n).map_or(-1, |x| x.color));
        h::SceneLaunch_PlayerTeam = Some(|n| p(n).map_or(-1, |x| x.team));
        h::SceneLaunch_PlayerHandicap = Some(|n| p(n).map_or(-1, |x| x.handicap));
        h::SceneLaunch_Teams = Some(|| SCN.as_ref().map_or(-1, |s| s.teams));
        h::SceneLaunch_TimeLimit = Some(|| SCN.as_ref().map_or(-1, |s| s.time_limit));
        h::SceneLaunch_RuleStocks = Some(|| SCN.as_ref().map_or(-1, |s| s.rule_stocks));
        h::SceneLaunch_ItemFreq = Some(|| SCN.as_ref().map_or(-1, |s| s.item_freq));
        // the VS mode's own copies of the queries (gmvsmelee.rs)
        use melee_modes::tu::gmvsmelee::ext::hook as v;
        v::SceneLaunch_Active = Some(|| 1);
        v::SceneLaunch_EntryStateId = Some(|| 2);
        v::SceneLaunch_PlayerStocks = Some(|n| p(n).map_or(-1, |x| x.stocks));
        h::SceneLaunch_PlayerStocks = Some(|n| p(n).map_or(-1, |x| x.stocks));
        // gm_1B03's copies (team costumes, `SceneLaunch_UntintedEnemy`): gw_sl_cfg.enemy_team_colors defaults to 0
        use melee_modes::tu::gm_1B03::ext as t;
        t::SCENELAUNCH_ACTIVE = Some(|| 1);
        t::SCENELAUNCH_ENEMYTEAMCOLORS = Some(|| 0);
    }
}
