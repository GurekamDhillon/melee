//! src/melee/gm/gmscenelaunch.h (TARGET_PC): the scene-launch seeders `SceneLaunch_SeedPlayers`,
//! `SceneLaunch_SeedVs` and the helpers they call. The C keeps them `static` in a header, so every
//! including TU compiles its own copy; there is one here. The query half of the header
//! (`SceneLaunch_Active`, `SceneLaunch_PlayerCKind`, ... in pc/platform/gw_runtime.c) is the
//! platform layer: the defaults below answer "no scene configured" (Active 0, every other query
//! -1 / 0), `ext::hook::<name>` installs a configuration.
//!
//! Checked against the C (the header included verbatim into a wrapper TU, shipping contraction,
//! every callee a recording stub): `gen/cref/tu/gmscenelaunch.c`, `tests/tu_gmscenelaunch.rs`.
//! `OSReport` text is a diagnostic and is not compared.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_types::*;

const GM_MAX: usize = GM_MAX_PLAYERS as usize;
const PKIND_NA: u8 = Gm_PKind_NA as u8;
const PKIND_CPU: u8 = Gm_PKind_Cpu as u8;
const PKIND_HUMAN: u8 = Gm_PKind_Human as u8;

/// Calls out of the header: the platform queries and the gm / lb functions of other TUs.
pub mod ext {
    use crate::tu::ext_fns_c as ext_fns;
    use melee_types::*;
    pub use crate::tu::globals::gm_80473814;
    pub fn OSReport(msg: &str) {
        eprint!("{msg}");
    }
    ext_fns! {
        SceneLaunch_Active() -> i32 = || 0;
        SceneLaunch_EnemyTeamColors() -> i32 = || 0;
        SceneLaunch_EntryStateId() -> i32 = || -1;
        SceneLaunch_ItemFreq() -> i32 = || -1;
        SceneLaunch_PlayerCKind(a0: i32) -> i32 = |_| -1;
        SceneLaunch_PlayerColor(a0: i32) -> i32 = |_| -1;
        SceneLaunch_PlayerCpuKind(a0: i32) -> i32 = |_| -1;
        SceneLaunch_PlayerCpuLevel(a0: i32) -> i32 = |_| -1;
        SceneLaunch_PlayerHandicap(a0: i32) -> i32 = |_| -1;
        SceneLaunch_PlayerNametag(a0: i32) -> i32 = |_| -1;
        SceneLaunch_PlayerSlotType(a0: i32) -> i32 = |_| -1;
        SceneLaunch_PlayerStocks(a0: i32) -> i32 = |_| -1;
        SceneLaunch_PlayerTeam(a0: i32) -> i32 = |_| -1;
        SceneLaunch_PreloadCKind(a0: i32) -> i32 = |_| -1;
        SceneLaunch_RuleMatch() -> i32 = || -1;
        SceneLaunch_RuleMinutes() -> i32 = || -1;
        SceneLaunch_RulePause() -> i32 = || -1;
        SceneLaunch_RuleStocks() -> i32 = || -1;
        SceneLaunch_StageExternal() -> i32 = || -1;
        SceneLaunch_Teams() -> i32 = || -1;
        SceneLaunch_TimeLimit() -> i32 = || -1;
        SceneLaunch_VsModeIndex() -> i32 = || -1;
        gmMainLib_GetGamePrefs() -> *mut GamePrefs = || unsafe { crate::tu::gmmain_lib::gmMainLib_GetGamePrefs() };
        gmMainLib_GetGameRules() -> *mut GameRules = || unsafe { crate::tu::gmmain_lib::gmMainLib_GetGameRules() };
        gm_80169264(a0: u8) -> u8 = |c| unsafe { crate::tu::gm_1601::gm_80169264(c as _) as _ };
        gm_80169290(a0: u8) -> u8 = |c| unsafe { crate::tu::gm_1601::gm_80169290(c as _) as _ };
        gm_801692BC(a0: u8) -> u8 = |c| unsafe { crate::tu::gm_1601::gm_801692BC(c as _) as _ };
        gm_GetNumCostumesForCKind(a0: u8) -> u8 = |c| unsafe { crate::tu::gm_1601::gm_GetNumCostumesForCKind(c) };
        gm_SetGameModeStateId(a0: u8) = |id| crate::tu::gm_1A3F::gm_SetGameModeStateId(id);
        gm_SetupAllPlayerDefaults(a0: *mut PlayerInitData) = |p| unsafe { crate::tu::gm_1601::gm_SetupAllPlayerDefaults(p) };
        gm_SetupPlayerDefaults(a0: *mut PlayerInitData) = |p| unsafe { crate::tu::gm_1601::gm_SetupPlayerDefaults(p) };
        lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState = || unsafe { melee_runtime::tu::lbdvd::lbDvd_GetPreloadCacheScene() };
        lbDvd_SetupVsPreloadCache() = || unsafe { melee_runtime::tu::lbdvd::lbDvd_SetupVsPreloadCache() };
        lbDvd_TrySetupSixSlotCache() -> i32 = || unsafe { melee_runtime::tu::lbdvd::lbDvd_TrySetupSixSlotCache() };
    }
}

/// `SceneLaunch_ApplyItemFrequency`: `Off` is a parser sentinel (-2), not frequency index zero.
pub unsafe fn SceneLaunch_ApplyItemFrequency(rules: *mut StartMeleeRules, prefs: *mut GamePrefs) {
    melee_core::ct!("SceneLaunch_ApplyItemFrequency");
    let frequency = ext::SceneLaunch_ItemFreq();
    if frequency != -1 {
        (*rules).item_freq = (if frequency == -2 { -1 } else { frequency }) as i8;
        (*prefs).item_freq = (*rules).item_freq as u8;
    }
}

/// `SceneLaunch_HudCount`: the highest occupied slot selects the HUD layout, including a lone P6.
pub use crate::tu::gm_1B03::SceneLaunch_HudCount;

fn SceneLaunch_UntintedEnemy(start: &StartMeleeData, i: usize) -> bool {
    ext::SceneLaunch_Active() != 0
        && ext::SceneLaunch_EnemyTeamColors() != 0
        && start.rules.is_teams != 0
        && start.players[i].slot_type != PKIND_NA
        && start.players[i].team != start.players[0].team
}

/// `SceneLaunch_ApplyTeamCostumes`: share costumes without duplicate tinting on the opposing team.
pub fn SceneLaunch_ApplyTeamCostumes(start: &mut StartMeleeData) {
    melee_core::ct!("SceneLaunch_ApplyTeamCostumes");
    for i in 0..GM_MAX {
        if SceneLaunch_UntintedEnemy(start, i) {
            let team = start.players[i].team;
            let ckind = start.players[i].ckind as u8;
            start.players[i].color = if team == 1 {
                ext::gm_801692BC(ckind)
            } else if team == 2 {
                ext::gm_80169290(ckind)
            } else {
                ext::gm_80169264(ckind)
            };
            start.players[i].sub_color = 0;
        }
    }
}

/// `SceneLaunch_SeedPlayers`: separate player seeding from preload/scene side effects. Returns how many
/// slots the scene named.
pub unsafe fn SceneLaunch_SeedPlayers(vs: *mut VsModeData, dummy_fallback: i32) -> i32 {
    melee_core::ct!("SceneLaunch_SeedPlayers");
    let players = &mut (*vs).start.players;
    let mut seeded = 0;
    for i in 0..GM_MAX {
        let ckind = ext::SceneLaunch_PlayerCKind(i as i32);
        if i >= 4 {
            ext::gm_SetupPlayerDefaults(&raw mut players[i]);
        }
        if ckind < 0 {
            // Extra slots must not inherit Multi-Man/previous-match state.
            if i >= 4 {
                players[i].ckind = ChKind_None as i8;
                players[i].slot_type = PKIND_NA;
            }
            continue;
        }
        players[i].ckind = ckind as i8;
        // A named slot is a human by default; the seeder only overrides what was asked for.
        let mut v = ext::SceneLaunch_PlayerSlotType(i as i32);
        players[i].slot_type = (if v >= 0 {
            v
        } else if ckind == ChKind_None as i32 {
            PKIND_NA as i32
        } else {
            PKIND_HUMAN as i32
        }) as u8;
        v = ext::SceneLaunch_PlayerColor(i as i32);
        {
            let mut costumes = if ckind == ChKind_None as i32 { 1 } else { ext::gm_GetNumCostumesForCKind(ckind as u8) as i32 };
            if costumes <= 0 {
                costumes = 1;
            }
            players[i].color = (<i32 as melee_core::ppc::Ppc>::prem(if v >= 0 { v } else { i as i32 }, costumes)) as u8;
        }
        v = ext::SceneLaunch_PlayerCpuKind(i as i32);
        if v >= 0 {
            players[i].cpu_kind = v as u8;
        } else if players[i].slot_type == PKIND_CPU {
            // VS mode's own CPUs are kind 4, the fighting AI; 0 is Training's stand-still dummy.
            players[i].cpu_kind = if dummy_fallback != 0 { 0 } else { 4 };
        }
        v = ext::SceneLaunch_PlayerCpuLevel(i as i32);
        if v >= 0 {
            players[i].cpu_level = v as u8;
        }
        v = ext::SceneLaunch_PlayerHandicap(i as i32);
        if v >= 0 {
            players[i].handicap = v as i8;
        }
        v = ext::SceneLaunch_PlayerTeam(i as i32);
        if v >= 0 {
            players[i].team = v as u8;
        } else if i >= 4 && ext::SceneLaunch_Teams() == 1 {
            players[i].team = 1;
        }
        v = ext::SceneLaunch_PlayerStocks(i as i32);
        if v >= 0 {
            players[i].stocks = v as i8;
        }
        v = ext::SceneLaunch_PlayerNametag(i as i32);
        if v >= 0 {
            players[i].nametag = v as u8;
        }
        // What the CSS would have written: slot = the player id + 1; sub_color is the shade, not a pad.
        players[i].slot = (i + 1) as u8;
        players[i].sub_color = 0;
        seeded += 1;
    }

    // Training dummy. Leaving it at ChKind_None walks off the end of ftMapping_list's retail rows.
    if dummy_fallback != 0 && players[0].ckind >= 0 && ext::SceneLaunch_PlayerCKind(1) < 0 {
        players[1] = players[0];
        players[1].slot_type = PKIND_CPU;
        players[1].cpu_kind = 0;
        players[1].cpu_level = 0;
        players[1].color = 1;
        players[1].slot = 2;
        players[1].sub_color = 0;
    }

    // Training extra CPUs (the pause menu CPU count, 2 and 3): do what the CSS exit does; a slot
    // the scene names keeps it.
    if dummy_fallback != 0 && players[1].ckind >= 0 && players[1].ckind != ChKind_None as i8 {
        for i in 2..4 {
            if ext::SceneLaunch_PlayerCKind(i as i32) >= 0 {
                continue;
            }
            players[i] = players[1];
            let mut ncost = ext::gm_GetNumCostumesForCKind(players[i].ckind as u8);
            if ncost == 0 {
                ncost = 1;
            }
            players[i].color = (<i32 as melee_core::ppc::Ppc>::prem(players[i - 1].color as i32 + 1, ncost as i32)) as u8;
            if players[i].color == players[0].color {
                players[i].color = (<i32 as melee_core::ppc::Ppc>::prem(players[i].color as i32 + 1, ncost as i32)) as u8;
            }
            players[i].slot_type = PKIND_NA; // the menu turns them on (fn_80188550)
            players[i].slot = (i + 1) as u8;
            players[i].sub_color = 0;
        }
    }
    seeded
}

/// `SceneLaunch_SeedVs`: seeds a `VsModeData` from the configured scene and jumps the mode state machine
/// to the requested screen. True when a scene was applied, false when nothing was configured.
pub unsafe fn SceneLaunch_SeedVs(vs: *mut VsModeData, dummy_fallback: i32) -> i32 {
    melee_core::ct!("SceneLaunch_SeedVs");
    if ext::SceneLaunch_Active() == 0 || ext::SceneLaunch_VsModeIndex() < 0 {
        return 0;
    }

    let _seeded = SceneLaunch_SeedPlayers(vs, dummy_fallback);

    if ext::SceneLaunch_Teams() >= 0 {
        (*vs).start.rules.is_teams = ext::SceneLaunch_Teams() as u8;
    }
    SceneLaunch_HudCount(&mut (*vs).start);
    SceneLaunch_ApplyTeamCostumes(&mut (*vs).start);
    if ext::SceneLaunch_TimeLimit() >= 0 {
        (*vs).start.rules.set_timer_enabled((ext::SceneLaunch_TimeLimit() != 0) as u32);
        (*vs).start.rules.time_limit = ext::SceneLaunch_TimeLimit() as u32;
    }

    // The saved rules. VS mode rebuilds every match rules from GameRules/GamePrefs when it starts the
    // match, so anything configured is written there too.
    {
        let gr = ext::gmMainLib_GetGameRules();
        let gp = ext::gmMainLib_GetGamePrefs();
        let mut any = false;
        let mut player_stocks = -1;
        // per-player /stocksN: GameRules has one stock count, so the largest one named
        for k in 0..GM_MAX {
            if ext::SceneLaunch_PlayerStocks(k as i32) > player_stocks {
                player_stocks = ext::SceneLaunch_PlayerStocks(k as i32);
            }
        }
        if ext::SceneLaunch_RuleMatch() >= 0 {
            (*gr).mode = ext::SceneLaunch_RuleMatch() as u8;
            any = true;
        } else if ext::SceneLaunch_RuleStocks() >= 0 || player_stocks >= 0 {
            (*gr).mode = 1; // a stock count with no match= means a stock match
            any = true;
        } else if ext::SceneLaunch_TimeLimit() > 0 || ext::SceneLaunch_RuleMinutes() > 0 {
            (*gr).mode = 0; // a time with no match= means a timed match
            any = true;
        }
        if ext::SceneLaunch_RuleStocks() >= 0 {
            (*gr).stock_count = ext::SceneLaunch_RuleStocks() as u8;
            any = true;
        } else if player_stocks >= 0 {
            (*gr).stock_count = player_stocks as u8;
            any = true;
        }
        if ext::SceneLaunch_RuleMinutes() >= 0 {
            (*gr).time_limit = ext::SceneLaunch_RuleMinutes() as u8;
            (*gr).stock_time_limit = ext::SceneLaunch_RuleMinutes() as u8;
            any = true;
        } else if ext::SceneLaunch_TimeLimit() >= 0 {
            // GameRules counts whole minutes and 0 means no limit: round seconds up
            let minutes = (ext::SceneLaunch_TimeLimit() + 59) / 60;
            (*gr).time_limit = minutes as u8;
            (*gr).stock_time_limit = minutes as u8;
            any = true;
        }
        if ext::SceneLaunch_RulePause() >= 0 {
            (*gr).pause = ext::SceneLaunch_RulePause() as u8;
        }
        SceneLaunch_ApplyItemFrequency(&raw mut (*vs).start.rules, gp);
        if any {
            // the rest of a clean competitive rule set: no handicap, 1.0x damage
            (*gr).handicap = 0;
            (*gr).damage_ratio = 10;
        }
    }

    // The three stage seeds, which must agree.
    let mut stkind = ext::SceneLaunch_StageExternal();
    if stkind < 0 {
        stkind = St_Kind_Izumi as i32;
    }
    (*vs).start.rules.stkind = (stkind as StKind) as u16;
    ext::gm_80473814.stage_id = stkind as i16;

    let cache = ext::lbDvd_GetPreloadCacheScene();
    (*cache).game_cache.stkind = stkind as StKind;
    for i in 0..GM_MAX {
        (*cache).game_cache.entries[i].char_id = (*vs).start.players[i].ckind as i32;
        (*cache).game_cache.entries[i].color = (*vs).start.players[i].color;
        if (*vs).start.players[4].slot_type != PKIND_NA
            || (*vs).start.players[5].slot_type != PKIND_NA
            || ext::SceneLaunch_PreloadCKind(0) >= 0
        {
            (*cache).game_cache.entries[i].x5 = 1;
        }
    }
    for i in 0..2usize {
        let ck = ext::SceneLaunch_PreloadCKind(i as i32);
        (*cache).game_cache.entries[i + GM_MAX].char_id = if ck >= 0 { ck } else { ChKind_None as i32 };
        (*cache).game_cache.entries[i + GM_MAX].color = 0;
        (*cache).game_cache.entries[i + GM_MAX].x5 = 1;
    }
    // Six-slot file admission is checked before starting asynchronous loads.
    if ((*vs).start.players[4].slot_type != PKIND_NA
        || (*vs).start.players[5].slot_type != PKIND_NA
        || ext::SceneLaunch_PreloadCKind(0) >= 0)
        && ext::lbDvd_TrySetupSixSlotCache() == 0
    {
        ext::gm_SetupAllPlayerDefaults((*vs).start.players.as_mut_ptr());
        for i in 0..8 {
            (*cache).game_cache.entries[i].char_id = ChKind_None as i32;
        }
        ext::OSReport("gw: scene: six-slot memory admission refused; returning to selection without loading fighters\n");
        ext::gm_SetGameModeStateId(0);
        return 0;
    }
    if (*vs).start.players[4].slot_type == PKIND_NA
        && (*vs).start.players[5].slot_type == PKIND_NA
        && ext::SceneLaunch_PreloadCKind(0) < 0
    {
        ext::lbDvd_SetupVsPreloadCache();
    }

    let mut entry = ext::SceneLaunch_EntryStateId();
    if dummy_fallback != 0
        && entry == 2
        && ((*vs).start.players[0].ckind < 0 || (*vs).start.players[0].ckind == ChKind_None as i8)
    {
        // Training with no player 1 fighter: open the CSS instead of the match.
        ext::OSReport("gw: scene: player 1 has no fighter - opening the CSS instead of the match\n");
        entry = 0;
    }
    if entry >= 0 {
        // CSS is 0, SSS is 1 and the playable state is 2 in both GM_VS and GM_TRAINING.
        ext::gm_SetGameModeStateId(entry as u8);
    }
    1
}

// The oracle's wrapper TU exports the three seeders under these names (gen/cref/tu/gmscenelaunch_src.c).
pub use SceneLaunch_ApplyItemFrequency as w_ApplyItemFrequency;
pub use SceneLaunch_SeedPlayers as w_SeedPlayers;
pub use SceneLaunch_SeedVs as w_SeedVs;
