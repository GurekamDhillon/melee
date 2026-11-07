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
    /// `pN=.../idle`: a CPU that stands still (gw_sl_cfg.p[slot].cpu_idle)
    pub cpu_idle: bool,
    /// costume shade, `PlayerInitData.sub_color` (-1 = unset; token `shadeN`)
    pub sub_color: i32,
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
    /// gw_sl_modes' game mode for `mode=vs|melee|lab|workbench` (GM_VS 2, GM_LAB 0x2F); -1 = not named (VS).
    pub game_mode: i32,
    /// `at=css|sss|match` (0 / 1 / 2); -1 = the match.
    pub entry_state: i32,
}

impl Scenario {
    /// The `record.sh` scene strings: `p1=falcon/human;p2=fox/human;stage=fd;match=time;minutes=0;items=off`.
    pub fn parse(scene: &str) -> Result<Scenario, String> {
        let mut s = Scenario { stage: -1, players: [None; 6], rule_match: -1, rule_minutes: -1, item_freq: -1, teams: -1, time_limit: -1, rule_stocks: -1, onep_mode: -1, onep_step: -1, onep_level: -1, game_mode: -1, entry_state: -1 };
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
                // the saved VS rules (gw_sl_set: teams=0|1, stocks=N)
                "teams" => s.teams = v.starts_with('1') as i32,
                "stocks" => s.rule_stocks = v.parse().map_err(|_| "stocks")?,
                // gw_sl_modes (pc/platform/gw_runtime.c): the 1P modes a scene launches through their OnLoad (gm_OnePSceneDirect)
                "at" => s.entry_state = match v { "css" => 0, "sss" => 1, "match" => 2, _ => return Err(format!("at={v}")) },
                // VS and Geno's LAB (geno_lab_mode.h GM_LAB = GM_COUNT + 2): the modes that seed players and take at=
                "mode" if matches!(v, "vs" | "melee") => s.game_mode = melee_types::GM_VS as i32,
                "mode" if matches!(v, "lab" | "workbench") => s.game_mode = melee_modes::tu::geno_lab_mode::GM_LAB as i32,
                "mode" => s.onep_mode = match v {
                    "classic" => 3, "adventure" => 4, "allstar" => 5,
                    // `{"tt", 0x0F}`: Target Test, booted through gm_Mode_TargetTest_OnLoad's TestTargetTestCKind direct launch
                    "tt" => 0x0F,
                    // port test entry, not in gw_sl_modes: GM_CLASSIC_GOVER (0x15), where gm_8017CBAC sends a
                    // cleared Classic (the ending movie, the credits, the trophy) - boots it without beating Master Hand
                    "classicend" => 0x15,
                    _ => -1,
                },
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
                    // gw_sl_parse_char: ck:N, then a Geno define by key or name ("geno:<key>" names only a define), then
                    // a retail name
                    let ck = match name.strip_prefix("ck:") {
                        Some(n) => n.parse().map_err(|_| format!("fighter {v}"))?,
                        None => {
                            let g = crate::geno_host::DefineResolve(name.strip_prefix("geno:").unwrap_or(name));
                            if g >= 0 {
                                g
                            } else if name.starts_with("geno:") {
                                return Err(format!("no Geno define {name} is mounted"));
                            } else if crate::mex_host::is_reference(name) || ckind_by_name(name).is_none() {
                                // gw_sl_parse_char: an m-ex reference or a name the disc's MxDt.dat may carry is
                                // resolved once the disc is mounted (`install`)
                                pending_fighter(name)
                            } else {
                                ckind_by_name(name).unwrap()
                            }
                        }
                    };
                    // gw_runtime.c's slot tokens after the fighter: human/hu, cpu, cpuN, idle (a CPU that stands still),
                    // teamN, stocksN, hmpN, cN (color)
                    let (mut slot_type, mut cpu_level, mut cpu_idle) = (0, -1, false);
                    let (mut team, mut stocks, mut handicap, mut color, mut sub_color) = (-1, -1, -1, -1, -1);
                    let digits = |t: &str, pre: &str| t.strip_prefix(pre).filter(|r| !r.is_empty() && r.bytes().all(|b| b.is_ascii_digit())).map(|r| r.parse::<i32>().unwrap());
                    for t in it {
                        if t == "human" || t == "hu" { slot_type = 0; }
                        else if t == "cpu" { slot_type = 1; }
                        else if t == "idle" { cpu_idle = true; slot_type = 1; }
                        else if let Some(n) = digits(t, "team") { team = n; }
                        else if let Some(n) = digits(t, "stocks") { stocks = n; }
                        else if let Some(n) = digits(t, "hmp") { handicap = n; }
                        else if let Some(n) = digits(t, "shade") { sub_color = n; }
                        else if let Some(n) = digits(t, "c") { color = n; }
                        else if let Some(n) = digits(t, "cpu") { slot_type = 1; cpu_level = n; }
                        else { return Err(format!("slot token {t} in {v} not wired")); }
                    }
                    s.players[n] = Some(PlayerSpec { ckind: ck, slot_type, cpu_level, color, team, stocks, handicap, cpu_idle, sub_color });
                }
                _ => {}
            }
        }
        // the 1P launch takes `idle` as the human player standing still (gw_runtime.c's 1P slot)
        if s.onep_mode >= 0 {
            for x in s.players.iter_mut().flatten() { if x.cpu_idle { x.cpu_idle = false; x.slot_type = 0; } }
        }
        Ok(s)
    }
}

/// `skipmemcard=0|1` of a scene string (gw_sl_cfg.skip_memcard, `c->skip_memcard = (val[0] == '1')`): -1 = auto (a
/// configured scene skips the memory-card prompt and runs save-less); 0 runs the boot scene with the card (a scene
/// launch that saves, as the C's `skipmemcard=0`).
pub fn skip_memcard(scene: &str) -> i32 {
    scene.split(';').filter_map(|kv| kv.split_once('=')).filter(|(k, _)| *k == "skipmemcard").map(|(_, v)| v.starts_with('1') as i32).last().unwrap_or(-1)
}

static mut SCN: Option<Scenario> = None;

/// Fighter tokens the parse could not resolve without the disc (m-ex names and references): `ckind` holds
/// `PENDING - index` until `install` resolves it.
static mut PENDING_NAMES: Vec<String> = Vec::new();
const PENDING: i32 = -1000;

fn pending_fighter(name: &str) -> i32 {
    unsafe {
        PENDING_NAMES.push(name.to_string());
        PENDING - (PENDING_NAMES.len() as i32 - 1)
    }
}

/// Resolve the pending fighter tokens of `s` against the mounted disc (gw_sl_parse_char's m-ex branches, then the
/// retail names). An unknown name is logged and leaves the slot empty-handed (-1), as the C's parse error does.
fn resolve_pending(s: &mut Scenario) {
    // the disc's MxDt.dat is read when the host boots (mex_host::install): a launch installed before that (the retail
    // flow's MELEE_SCENE) resolves on the first SceneLaunch_PlayerCKind query instead
    if !crate::host::Host::is_booted() {
        return;
    }
    for (n, p) in s.players.iter_mut().enumerate() {
        let Some(p) = p else { continue };
        if p.ckind > PENDING {
            // a retail name on an m-ex disc: the disc's own name for that slot wins (gw_sl_mex_name)
            continue;
        }
        let tok = unsafe { PENDING_NAMES.get((PENDING - p.ckind) as usize).cloned().unwrap_or_default() };
        match crate::mex_host::resolve_fighter(&tok).or_else(|| ckind_by_name(&tok)) {
            Some(ck) => {
                eprintln!("scene: p{} '{tok}' -> ck={ck}", n + 1);
                p.ckind = ck;
            }
            None => {
                eprintln!("scene: p{} '{tok}' is not a fighter on this disc", n + 1);
                p.ckind = -1;
            }
        }
    }
}

fn p(n: i32) -> Option<PlayerSpec> {
    unsafe { SCN.as_ref().and_then(|s| s.players.get(n as usize).copied().flatten()) }
}

/// `gw_Script_CpuIdleDefault(slot)`: 2 for a scene `idle` slot, else 0 (no `cpus=` / MELEE_CPU_IDLE default in the engine).
pub fn cpu_idle_default(slot: i32) -> i32 {
    if p(slot).is_some_and(|x| x.cpu_idle) { 2 } else { 0 }
}

/// gw_SceneLaunch_TargetTestCKind (gw_runtime.c): p1's character when the scene launches Target Test, else -1 (not set).
pub fn target_test_ckind() -> i32 {
    unsafe { SCN.as_ref().filter(|s| s.onep_mode == 0x0F).and_then(|_| p(0)).map_or(-1, |x| x.ckind) }
}

/// The mode a scenario boots into: its 1P mode, LAB, else VS.
pub fn boot_mode(s: &Scenario) -> u8 {
    if s.onep_mode >= 0 { s.onep_mode as u8 } else if s.game_mode >= 0 { s.game_mode as u8 } else { melee_types::GM_VS as u8 }
}

/// gw_SceneLaunch_BootGameMode: the configured scene's mode, -1 none.
pub fn boot_game_mode() -> i32 {
    unsafe { SCN.as_ref().map_or(-1, |s| boot_mode(s) as i32) }
}
/// gw_SceneLaunch_EntryStateId: 0 CSS, 1 SSS, 2 match (the default) for the VS-machinery modes; -1 otherwise.
pub fn entry_state_id() -> i32 {
    unsafe { SCN.as_ref().filter(|s| s.onep_mode < 0).map_or(-1, |s| if s.entry_state < 0 { 2 } else { s.entry_state }) }
}
/// gw_SceneLaunch_ItemFreq: -2 off, -1 unset, else the frequency index.
pub fn item_freq() -> i32 {
    unsafe { SCN.as_ref().map_or(-1, |s| s.item_freq) }
}
/// gw_SceneLaunch_LoadForTest: configure from `text` (None: no scene), as MELEE_SCENE would.
pub fn load_for_test(text: Option<&str>) {
    match text.map(Scenario::parse) {
        Some(Ok(s)) => install(s),
        Some(Err(e)) => { eprintln!("gw: scene: {e}"); unsafe { SCN = None } }
        None => unsafe { SCN = None },
    }
    install_geno();
}
/// The LAB's and the Geno tests' copies of the scene queries.
fn install_geno() {
    unsafe {
        use melee_modes::tu::geno_lab_mode::ext::hook as l;
        l::SceneLaunch_BootGameMode = Some(boot_game_mode);
        l::SceneLaunch_ItemFreq = Some(item_freq);
        use melee_modes::tu::geno_tests::ext::hook as t;
        t::SceneLaunch_BootGameMode = Some(boot_game_mode);
        t::SceneLaunch_EntryStateId = Some(entry_state_id);
        t::SceneLaunch_StageExternal = Some(|| SCN.as_ref().map_or(-1, |s| s.stage));
        t::SceneLaunch_ItemFreq = Some(item_freq);
        t::SceneLaunch_PlayerCKind = Some(|n| p(n).map_or(-1, |x| x.ckind));
        t::SceneLaunch_PlayerSlotType = Some(|n| p(n).map_or(-1, |x| x.slot_type));
        t::SceneLaunch_LoadForTest = Some(|t| { let s = if t.is_null() { None } else { Some(core::ffi::CStr::from_ptr(t).to_string_lossy().into_owned()) }; load_for_test(s.as_deref()) });
    }
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
pub fn install(mut s: Scenario) {
    resolve_pending(&mut s);
    install_geno();
    if s.onep_mode >= 0 {
        unsafe { SCN = Some(s) };
        install_onep();
        return;
    }
    unsafe {
        SCN = Some(s);
        h::SceneLaunch_Active = Some(|| 1);
        h::SceneLaunch_VsModeIndex = Some(|| melee_types::GmVsMode_Melee as i32);
        h::SceneLaunch_EntryStateId = Some(entry_state_id);
        h::SceneLaunch_StageExternal = Some(|| SCN.as_ref().map_or(-1, |s| s.stage));
        h::SceneLaunch_PlayerCKind = Some(|n| {
            if let Some(s) = SCN.as_mut() {
                resolve_pending(s);
            }
            p(n).map_or(-1, |x| x.ckind)
        });
        h::SceneLaunch_PlayerSlotType = Some(|n| p(n).map_or(-1, |x| x.slot_type));
        h::SceneLaunch_PlayerCpuLevel = Some(|n| p(n).map_or(-1, |x| x.cpu_level));
        h::SceneLaunch_RuleMatch = Some(|| SCN.as_ref().map_or(-1, |s| s.rule_match));
        h::SceneLaunch_RuleMinutes = Some(|| SCN.as_ref().map_or(-1, |s| s.rule_minutes));
        h::SceneLaunch_PlayerColor = Some(|n| p(n).map_or(-1, |x| x.color));
        h::SceneLaunch_PlayerTeam = Some(|n| p(n).map_or(-1, |x| x.team));
        h::SceneLaunch_PlayerHandicap = Some(|n| p(n).map_or(-1, |x| x.handicap));
        h::SceneLaunch_PlayerSubColor = Some(|n| p(n).map_or(-1, |x| x.sub_color));
        h::SceneLaunch_Teams = Some(|| SCN.as_ref().map_or(-1, |s| s.teams));
        h::SceneLaunch_TimeLimit = Some(|| SCN.as_ref().map_or(-1, |s| s.time_limit));
        h::SceneLaunch_RuleStocks = Some(|| SCN.as_ref().map_or(-1, |s| s.rule_stocks));
        h::SceneLaunch_ItemFreq = Some(|| SCN.as_ref().map_or(-1, |s| s.item_freq));
        // the VS mode's own copies of the queries (gmvsmelee.rs)
        use melee_modes::tu::gmvsmelee::ext::hook as v;
        v::SceneLaunch_Active = Some(|| 1);
        v::SceneLaunch_EntryStateId = Some(entry_state_id);
        v::SceneLaunch_PlayerStocks = Some(|n| p(n).map_or(-1, |x| x.stocks));
        h::SceneLaunch_PlayerStocks = Some(|n| p(n).map_or(-1, |x| x.stocks));
        // gm_1B03's copies (team costumes, `SceneLaunch_UntintedEnemy`): gw_sl_cfg.enemy_team_colors defaults to 0
        use melee_modes::tu::gm_1B03::ext as t;
        t::SCENELAUNCH_ACTIVE = Some(|| 1);
        t::SCENELAUNCH_ENEMYTEAMCOLORS = Some(|| 0);
    }
}
