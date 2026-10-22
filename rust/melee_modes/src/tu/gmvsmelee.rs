#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::all)]
//! Native port of src/melee/gm/gmvsmelee.c (the whole TU, TARGET_PC build: VS Melee's CSS/SSS/match/
//! sudden-death/results hooks, the KO-star counts and the mode's OnInit/OnLoad, including the PC
//! online/scene-launch/m-ex branches). Calls out of the TU go through `ext` (a port where one
//! exists in this crate, else a TODO panic). Checked against the C (whole TU included verbatim,
//! shipping contraction) by tests/tu_gmvsmelee.rs.
use super::globals::gmMainLib_804D3EE0;
use core::ffi::c_void;
use melee_types::*;

/// `GM_FRONTEND` / `GS_FRONTEND` (gmfrontend.h, TARGET_PC): one past the retail kinds.
pub const GM_FRONTEND: i32 = GM_COUNT as i32 + 1;

pub type gm_StartMeleeCallback = Option<unsafe extern "C" fn(*mut StartMeleeData, *mut StartMeleeData)>;
pub type gm_PlayerInitCallback = Option<unsafe extern "C" fn(*mut PlayerInitData, *mut PlayerInitData)>;

/// Calls out of the TU. `ext::hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use crate::tu::ext_fns_c as ext_fns;
    use core::ffi::c_void;
    use melee_types::*;
    ext_fns! {
        lbTime_8000AF74(a: u32, b: i32) -> u32 = crate::lb::lbTime_8000AF74;
        memzero(mem: *mut c_void, size: isize) = |m, n| unsafe { core::ptr::write_bytes(m as *mut u8, 0, n as usize) };
        Mex_Enabled(name: &str) -> i32 = |_| todo!("TODO(port) Mex_Enabled (pc/platform mex)");
        SceneLaunch_SeedVs(vs: *mut VsModeData, dummy_fallback: bool) -> bool = |_, _| todo!("TODO(port) SceneLaunch_SeedVs (gmscenelaunch.h, TARGET_PC)");
        SceneLaunch_Active() -> i32 = || todo!("TODO(port) SceneLaunch_Active (pc/platform)");
        SceneLaunch_EntryStateId() -> i32 = || todo!("TODO(port) SceneLaunch_EntryStateId (pc/platform)");
        SceneLaunch_PlayerStocks(slot: i32) -> i32 = |_| todo!("TODO(port) SceneLaunch_PlayerStocks (pc/platform)");
        SceneLaunch_HudCount(start: *mut StartMeleeData) = |_| todo!("TODO(port) SceneLaunch_HudCount (gmscenelaunch.h, TARGET_PC)");
        OSReport(msg: &str) = |_| todo!("TODO(port) OSReport");
        gm_GetGameModeStateEnterData(state: *mut GameModeState) -> *mut c_void = |_| todo!("TODO(port) gm_GetGameModeStateEnterData (gm_1A3F.c)");
        gm_GetGameModeStateExitData(state: *mut GameModeState) -> *mut c_void = |_| todo!("TODO(port) gm_GetGameModeStateExitData (gm_1A3F.c)");
        gm_ChangeGameModeAfterCurrentScene(pending_mode: i32) = |_| todo!("TODO(port) gm_ChangeGameModeAfterCurrentScene (gm_1A3F.c)");
        gm_SetNextGameModeStateId(id: u8) = |_| todo!("TODO(port) gm_SetNextGameModeStateId (gm_1A3F.c)");
        gm_InitVsMode(vs: *mut VsModeData) = |_| todo!("TODO(port) gm_InitVsMode (gm_1B03.c)");
        gmMainLib_8015CDEC() = || todo!("TODO(port) gmMainLib_8015CDEC (gmmain_lib.c)");
        lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState = || todo!("TODO(port) lbDvd_GetPreloadCacheScene (lbdvd.c)");
        lbDvd_SetupVsPreloadCache() = || todo!("TODO(port) lbDvd_SetupVsPreloadCache (lbdvd.c)");
        Frontend_OnlinePick() -> i32 = || todo!("TODO(port) Frontend_OnlinePick (gmfrontend)");
        Frontend_OnlinePicked(which: i32, a: i32, b: i32) = |_, _, _| todo!("TODO(port) Frontend_OnlinePicked (gmfrontend)");
        Frontend_BackToOnline() = || todo!("TODO(port) Frontend_BackToOnline (gmfrontend)");
        lbAudioAx_80026E84(ckind: CharacterKind) -> u64 = |_| todo!("TODO(port) lbAudioAx_80026E84 (lbaudio_ax.c)");
        lbAudioAx_80026F2C(arg: u32) = |_| todo!("TODO(port) lbAudioAx_80026F2C (lbaudio_ax.c)");
        lbAudioAx_8002702C(flags: u32, mask: u64) = |_, _| todo!("TODO(port) lbAudioAx_8002702C (lbaudio_ax.c)");
        lbAudioAx_80027168() = || todo!("TODO(port) lbAudioAx_80027168 (lbaudio_ax.c)");
        lbAudioAx_80026EBC(stkind: StKind) -> u64 = |_| todo!("TODO(port) lbAudioAx_80026EBC (lbaudio_ax.c)");
        gm_80167FC4(sss: *mut SSSData) = |_| todo!("TODO(port) gm_80167FC4 (gm_1601.c)");
        gm_80167BC8(vs: *mut VsModeData) = |_| todo!("TODO(port) gm_80167BC8 (gm_1601.c)");
        gm_SetupSubColors(start: *mut StartMeleeData) = |_| todo!("TODO(port) gm_SetupSubColors (gm_1601.c)");
        gm_LoadRumbleEnabled(start: *mut StartMeleeData) = |_| todo!("TODO(port) gm_LoadRumbleEnabled (gm_1B03.c)");
        gm_LoadAnnouncer() = || todo!("TODO(port) gm_LoadAnnouncer (gm_1601.c)");
        Netplay_StageEnd(winner: i32, s0: i32, s1: i32) = |_, _, _| todo!("TODO(port) Netplay_StageEnd (gw_netplay.c)");
        Netplay_RunActive() -> i32 = || todo!("TODO(port) Netplay_RunActive (gw_netplay.c)");
        Netplay_GameResult(winner: i32) = |_| todo!("TODO(port) Netplay_GameResult (gw_netplay.c)");
        Netplay_Enabled() -> i32 = || todo!("TODO(port) Netplay_Enabled (gw_netplay.c)");
        Netplay_NoteTie() = || todo!("TODO(port) Netplay_NoteTie (gw_netplay.c)");
        Netplay_RematchPending() -> i32 = || todo!("TODO(port) Netplay_RematchPending (gw_netplay.c)");
        gm_WasMatchCanceled(outcome: u8) -> bool = crate::gm::gm_WasMatchCanceled;
        gm_MatchHasMultipleWinners(end: *mut MatchEnd) -> bool = |e| crate::gm_r4::gm_MatchHasMultipleWinners(unsafe { &*e });
        gm_80162574(ckind: u8, outcome: u8) = |_, _| todo!("TODO(port) gm_80162574 (gm_1601.c)");
        gm_SetupHumanResultsScreen(match_kind: u8, outcome: u8) = |_, _| todo!("TODO(port) gm_SetupHumanResultsScreen (gm_1601.c)");
        gm_SetupResultsScreenPlayTime(seconds: u32, arg: u32) = |_, _| todo!("TODO(port) gm_SetupResultsScreenPlayTime (gm_1601.c)");
        gm_80162800(end: *mut MatchEnd) -> u32 = |_| todo!("TODO(port) gm_80162800 (gm_1601.c)");
        gm_SetupSuddenDeath(start: *mut StartMeleeData, end: *mut MatchEnd) = |_, _| todo!("TODO(port) gm_SetupSuddenDeath (gm_1601.c)");
        gm_80166CCC(a: *mut MatchEnd, b: *mut MatchEnd) = |_, _| todo!("TODO(port) gm_80166CCC (gm_1601.c)");
        gm_80177724(info: *mut ResultsMatchInfo) = |_| todo!("TODO(port) gm_80177724 (gmresult.c)");
        gm_80168638(end: *mut MatchEnd) = |_| todo!("TODO(port) gm_80168638 (gm_1601.c)");
        gm_80168710(end: *mut MatchEnd, vs: *mut VsModeData) = |_, _| todo!("TODO(port) gm_80168710 (gm_1601.c)");
        gm_8016247C(arg: i32) -> i32 = |_| todo!("TODO(port) gm_8016247C (gm_1601.c)");
        gm_801688AC(end: *mut MatchEnd) -> i32 = |e| crate::gm_r10::gm_801688AC(unsafe { &*e });
        gm_GetVsPlayMatchTotal() -> u32 = || todo!("TODO(port) gm_GetVsPlayMatchTotal (gm_1601.c)");
        gm_80172DD4(arg: u32) -> u8 = |_| todo!("TODO(port) gm_80172DD4 (gm_16F1.c)");
        gmMainLib_8015ED98() -> *mut gmm_retval_ED98 = || todo!("TODO(port) gmMainLib_8015ED98 (gmmain_lib.c)");
        gm_InitChallengerData(human_ckind: u8, human_color: u8, human_slot: u8, human_nametag: u8, cpu_ckind: u8, curr_mode: u8)
            = |_, _, _, _, _, _| todo!("TODO(port) gm_InitChallengerData (gm_16F1.c)");
        gm_80172D78() -> u8 = || todo!("TODO(port) gm_80172D78 (gm_16F1.c)");
        gm_80172E74() -> u8 = || todo!("TODO(port) gm_80172E74 (gm_16F1.c)");
        gm_80172F00(arg: u32) -> u16 = |_| todo!("TODO(port) gm_80172F00 (gm_16F1.c)");
        gmMainLib_8015EDB0() -> *mut gmm_retval_EDB0 = || todo!("TODO(port) gmMainLib_8015EDB0 (gmmain_lib.c)");
        gm_80164504(arg: u16) = |_| todo!("TODO(port) gm_80164504 (gm_1601.c)");
        gm_80173DE4(end: *mut MatchEnd) = |_| todo!("TODO(port) gm_80173DE4 (gm_16F1.c)");
        gm_80172898(arg: u16) -> i32 = |_| todo!("TODO(port) gm_80172898 (gm_16F1.c)");
        gm_80173EEC() = || todo!("TODO(port) gm_80173EEC (gm_16F1.c)");
        gm_801721EC() -> bool = || todo!("TODO(port) gm_801721EC (gm_16F1.c)");
        lbCardNew_AllocWorkArea() = || todo!("TODO(port) lbCardNew_AllocWorkArea (lbcardnew.c)");
        lbCardGame_LoadArchive(arg: i32) = |_| todo!("TODO(port) lbCardGame_LoadArchive (lbcardgame.c)");
        lbCardGame_SaveChanges() = || todo!("TODO(port) lbCardGame_SaveChanges (lbcardgame.c)");
    }
}

/// lb/inlines.h `lbCardGame_SetupArchive` (static inline).
unsafe fn lbCardGame_SetupArchive() {
    ext::lbCardNew_AllocWorkArea();
    ext::lbCardGame_LoadArchive(0);
    ext::lbCardGame_SaveChanges();
}

/* 4807B0 */ pub static mut gmVsMelee_CssData: CSSData = unsafe { core::mem::zeroed() };
/* 480668 */ pub static mut gmVsMelee_SssData: SSSData = unsafe { core::mem::zeroed() };
/* 480530 */ pub static mut gmVsMelee_StartData: StartMeleeData = unsafe { core::mem::zeroed() };
/* 479D98 */ pub static mut gmVsMelee_VsExitInfo: MatchExitInfo = unsafe { core::mem::zeroed() };
/* 47E2A4 */ pub static mut gmVsMelee_SuddenDeathExitInfo: MatchExitInfo = unsafe { core::mem::zeroed() };
/* 47C020 */ pub static mut gmVsMelee_ResultsEnterData: ResultsMatchInfo = unsafe { core::mem::zeroed() };
/* 4D6730 */ pub static mut ko_counts: [u8; GM_MAX_PLAYERS as usize] = [0; GM_MAX_PLAYERS as usize];

/// 1A5244
pub unsafe fn gmVsMelee_GetVsData() -> *mut VsModeData {
    &mut (*gmMainLib_804D3EE0).modes.table[GmVsMode_Melee as usize]
}

/// 1A5250
pub unsafe fn gmVsMelee_GetKOCounts() -> *mut u8 {
    ko_counts.as_mut_ptr()
}

/// 1A5258: per human player, `lbTime_8000AF74` of the count and the standing's x20.
pub unsafe fn gmVsMelee_UpdateKOCounts(ko: *mut u8, end: *mut MatchEnd) {
    for i in 0..GM_MAX_PLAYERS as usize {
        if (*end).player_standings[i].pkind as i32 == Gm_PKind_Human as i32 {
            *ko.add(i) = ext::lbTime_8000AF74(*ko.add(i) as u32, (*end).player_standings[i].x20 as i32) as u8;
        }
    }
}

/// 1A52D0
pub use crate::gm_ptr::gmVsMelee_WasAnyPlayerHuman;

/// 1A5360 (static): the human player with the smallest `is_small_loser` (teams: team's in the high byte).
pub unsafe fn findSmallestLoser(end: *mut MatchEnd) -> u8 {
    let mut loser: i32 = 0xFFFF; // U16_MAX
    let mut player_idx: isize = 0;
    let mut losers = [0i32; GM_MAX_PLAYERS as usize];

    if (*end).is_teams == 1 {
        for i in 0..GM_MAX_PLAYERS as usize {
            losers[i] = (*end).player_standings[i].is_small_loser as i32
                + (((*end).team_standings[(*end).player_standings[i].team as usize].is_small_loser as i32) << 8);
        }
    } else {
        for i in 0..GM_MAX_PLAYERS as usize {
            losers[i] = (*end).player_standings[i].is_small_loser as i32;
        }
    }

    for i in 0..GM_MAX_PLAYERS as usize {
        if (*end).player_standings[i].pkind as i32 == Gm_PKind_Human as i32 && losers[i] < loser {
            loser = losers[i];
            player_idx = i as isize;
        }
    }

    if loser != 0xFFFF {
        return player_idx as u8;
    }

    panic!("HSD_ASSERT gmvsmelee.c:178 (no human player in findSmallestLoser)");
}

/// 1A5598
pub unsafe fn gmVsMelee_Mode_OnInit() {
    ext::gm_InitVsMode(&mut (*gmMainLib_804D3EE0).modes.table[GmVsMode_Melee as usize]);
    ext::gmMainLib_8015CDEC();
}

/// 1A55C4
pub unsafe fn gmVsMelee_ResetKOCounts() {
    ext::memzero(ko_counts.as_mut_ptr() as *mut c_void, core::mem::size_of_val(&ko_counts) as isize);
}

/// 1A55EC
pub unsafe fn gmVsMelee_Mode_OnLoad() {
    // m-ex "Disable KO Star Deletion When Leaving CSS" (opt-in MELEE_MEX=keep_ko_stars), then the
    // scene launch seed (MELEE_SCENE=mode=vs;...), TARGET_PC.
    if ext::Mex_Enabled("keep_ko_stars") != 0 {
        return;
    }
    ext::SceneLaunch_SeedVs(&mut (*gmMainLib_804D3EE0).modes.table[GmVsMode_Melee as usize], false);
    gmVsMelee_ResetKOCounts();
}

/// 1A5614
pub fn gm_Mode_Vs_OnUnload() {}

/// 1A5618
pub unsafe fn gmVsMelee_EnterCss(state: *mut GameModeState, vs: *mut VsModeData, match_type: CSSMatchType) {
    let css = ext::gm_GetGameModeStateEnterData(state) as *mut CSSData;
    if (*vs).start.players[4].slot_type as i32 != Gm_PKind_NA as i32 || (*vs).start.players[5].slot_type as i32 != Gm_PKind_NA as i32 {
        ext::OSReport("gw: scene: six-slot CSS route refused; returning to menus\n");
        for i in 4..GM_MAX_PLAYERS as usize {
            (*vs).start.players[i].slot_type = Gm_PKind_NA as u8;
            (*vs).start.players[i].ckind = ChKind_None as i8;
        }
        for i in 4..8usize {
            (*ext::lbDvd_GetPreloadCacheScene()).game_cache.entries[i].char_id = ChKind_None as i32;
        }
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
    }
    (*css).match_type = match_type as u8;
    (*css).ko_counts = ko_counts.as_mut_ptr();
    (*css).vs = *vs;
    ext::lbDvd_SetupVsPreloadCache();
}

/// 1A5680
pub unsafe fn gmVsMelee_ExitCss(state: *mut GameModeState, vs: *mut VsModeData) {
    let css = ext::gm_GetGameModeStateExitData(state) as *mut CSSData;
    // ONLINE PLAY sent the player here to pick a fighter: back to it, not on to the SSS
    if ext::Frontend_OnlinePick() == 1 {
        let mut ck: i32 = -1;
        let mut color: i32 = 0;
        // confirmed (START: 1), not backed out (2)
        if (*css).pending_scene_change as i32 != CSSPendingSceneChange_2 as i32 {
            for i in 0..4usize {
                let p = &(*css).vs.start.players[i];
                if p.slot_type as i32 == Gm_PKind_Human as i32 && p.ckind as i32 >= 0 && p.ckind as i32 != ChKind_None as i32 {
                    ck = p.ckind as i32;
                    color = p.color as i32;
                    break;
                }
            }
        }
        ext::Frontend_OnlinePicked(1, ck, color);
        ext::gm_ChangeGameModeAfterCurrentScene(GM_FRONTEND);
        return;
    }
    if (*css).pending_scene_change as i32 == CSSPendingSceneChange_2 as i32 {
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }

    *vs = (*css).vs;
    let mut mask: u64 = 0;
    for i in 0..GM_MAX_PLAYERS as usize {
        mask |= ext::lbAudioAx_80026E84((*css).vs.start.players[i].ckind as CharacterKind);
    }
    ext::lbAudioAx_80026F2C(20);
    ext::lbAudioAx_8002702C(4, mask);
    ext::lbAudioAx_80027168();
}

/// 1A5754
pub unsafe fn gmVsMelee_EnterSss(state: *mut GameModeState, vs: *mut VsModeData) {
    let sss = ext::gm_GetGameModeStateEnterData(state) as *mut SSSData;
    (*sss).vs = *vs;
    ext::gm_80167FC4(sss);
}

/// 1A57A8
pub unsafe fn gmVsMelee_ExitSss(state: *mut GameModeState, vs: *mut VsModeData, cancel_state_id: u8) {
    let sss = ext::gm_GetGameModeStateExitData(state) as *mut SSSData;
    // ONLINE PLAY sent the host here to pick the stage: back to it, not into a match
    if ext::Frontend_OnlinePick() == 2 {
        ext::Frontend_OnlinePicked(2, if (*sss).start_game != 0 { (*sss).vs.start.rules.stkind as i32 } else { -1 }, 0);
        ext::gm_ChangeGameModeAfterCurrentScene(GM_FRONTEND);
        return;
    }
    if (*sss).start_game != 0 {
        *vs = (*sss).vs;
        ext::lbAudioAx_80026F2C(24);
        ext::lbAudioAx_8002702C(8, ext::lbAudioAx_80026EBC((*vs).start.rules.stkind as StKind));
        ext::lbAudioAx_80027168();
    } else {
        ext::gm_SetNextGameModeStateId(cancel_state_id);
    }
}

/// 1A583C
pub unsafe fn gmVsMelee_EnterVs(state: *mut GameModeState, vs: *mut VsModeData, start_cb: gm_StartMeleeCallback, player_cb: gm_PlayerInitCallback) {
    let start = ext::gm_GetGameModeStateEnterData(state) as *mut StartMeleeData;

    ext::gm_80167BC8(vs);
    // Saved VS rules have one global stock count: restore explicit per-slot values.
    if ext::SceneLaunch_Active() != 0 && ext::SceneLaunch_EntryStateId() == 2 {
        for i in 0..GM_MAX_PLAYERS as usize {
            let stocks = ext::SceneLaunch_PlayerStocks(i as i32);
            if stocks >= 0 {
                (*vs).start.players[i].stocks = stocks as i8;
            }
        }
    }
    (*start).rules = (*vs).start.rules;

    if (*start).rules.match_kind() == MatchKind_Stock as u32 {
        (*start).rules.set_is_stock(1);
    }
    (*start).rules.set_is_vs(1);

    if let Some(cb) = start_cb {
        cb(start, &mut (*vs).start);
    }

    for i in 0..Gm_Player_NumMax as usize {
        (*start).players[i] = (*vs).start.players[i];
    }
    if let Some(cb) = player_cb {
        for i in 0..Gm_Player_NumMax as usize {
            cb(&mut (*start).players[i], &mut (*vs).start.players[i]);
        }
    }

    ext::SceneLaunch_HudCount(start); // after saved rules, callbacks and player copy
    ext::gm_SetupSubColors(start);
    ext::gm_LoadRumbleEnabled(start);
    ext::gm_LoadAnnouncer();
}

/// 1A5AF0: `id0` is the next state id if one or zero winners, `id1` if several.
pub unsafe fn gmVsMelee_ExitVs(state: *mut GameModeState, mut id0: u8, id1: u8) {
    let exit = ext::gm_GetGameModeStateExitData(state) as *mut MatchExitInfo;
    // an online STAGE RUN: the peers exchange the stage-end tuple; returns at once outside a run
    let mut w: i32 = -1;
    if !ext::gm_WasMatchCanceled((*exit).match_end.outcome) && (*exit).match_end.n_winners == 1 && (*exit).match_end.winners[0] <= 1 {
        w = (*exit).match_end.winners[0] as i32;
    }
    ext::Netplay_StageEnd(w, (*exit).match_end.player_standings[0].stocks as i32, (*exit).match_end.player_standings[1].stocks as i32);
    if ext::Netplay_RunActive() != 0 {
        ext::Netplay_GameResult(w);
        ext::Frontend_BackToOnline();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_FRONTEND);
        return;
    }
    if (*exit).match_end.player_standings[4].pkind as i32 != Gm_PKind_NA as i32 || (*exit).match_end.player_standings[5].pkind as i32 != Gm_PKind_NA as i32 {
        ext::OSReport("gw: scene: six-slot VS finished; returning to menus (CSS/results round-trip unsupported)\n");
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }

    // m-ex "Skip Result Screen" (opt-in MELEE_MEX=skip_result_screen); online results must run
    if ext::Netplay_Enabled() == 0 && ext::Mex_Enabled("skip_result_screen") != 0 {
        id0 = 0;
    }

    // ONLINE: a tie does not go to sudden death; it is a no-contest the lobby replays
    if ext::Netplay_Enabled() != 0 && ext::gm_MatchHasMultipleWinners(&mut (*exit).match_end) {
        (*exit).match_end.outcome = OUTCOME_NO_CONTEST as u8;
        ext::Netplay_NoteTie();
    }

    for i in 0..GM_MAX_PLAYERS as usize {
        if (*exit).match_end.player_standings[i].pkind as i32 == Gm_PKind_Human as i32 {
            ext::gm_80162574((*exit).match_end.player_standings[i].ckind as u8, (*exit).match_end.outcome);
        }
    }

    if gmVsMelee_WasAnyPlayerHuman(&(*exit).match_end) {
        ext::gm_SetupHumanResultsScreen((*exit).match_end.match_kind, (*exit).match_end.outcome);
        ext::gm_SetupResultsScreenPlayTime((*exit).match_end.frame_count / GM_FPS as u32, ext::gm_80162800(&mut (*exit).match_end));
    }

    if !ext::gm_MatchHasMultipleWinners(&mut (*exit).match_end) {
        ext::gm_SetNextGameModeStateId(id0);
    } else {
        ext::gm_SetNextGameModeStateId(id1);
    }
}

/// 1A5C3C
pub unsafe fn gmVsMelee_EnterSuddenDeath(state: *mut GameModeState, vs: *mut VsModeData, start_cb: gm_StartMeleeCallback, player_cb: gm_PlayerInitCallback) {
    let start = ext::gm_GetGameModeStateEnterData(state) as *mut StartMeleeData;
    (*start).rules = (*vs).start.rules;

    if let Some(cb) = start_cb {
        cb(start, &mut (*vs).start);
    }

    for i in 0..GM_MAX_PLAYERS as usize {
        (*start).players[i] = (*vs).start.players[i];
    }
    if let Some(cb) = player_cb {
        for i in 0..GM_MAX_PLAYERS as usize {
            cb(&mut (*start).players[i], &mut (*vs).start.players[i]);
        }
    }

    ext::gm_SetupSubColors(start);
    ext::gm_LoadRumbleEnabled(start);
    ext::gm_SetupSuddenDeath(start, &mut gmVsMelee_VsExitInfo.match_end);
}

/// 1A5EC8
pub unsafe fn gmVsMelee_ExitSuddenDeath(state: *mut GameModeState) {
    let exit_data = ext::gm_GetGameModeStateExitData(state) as *mut MatchExitInfo;
    ext::gm_80166CCC(&mut gmVsMelee_VsExitInfo.match_end, &mut (*exit_data).match_end);
}

/// 1A5F00
pub unsafe fn gmVsMelee_EnterResults(state: *mut GameModeState) {
    let enter_data = ext::gm_GetGameModeStateEnterData(state) as *mut ResultsMatchInfo;
    ext::gm_80177724(enter_data);
    (*enter_data).match_end = gmVsMelee_VsExitInfo.match_end;
}

/// 1A5F64
pub unsafe fn gmVsMelee_ExitResults(state: *mut GameModeState, vs: *mut VsModeData, state_id: u8) {
    let ko = ko_counts.as_mut_ptr();
    let match_end: *mut MatchEnd = &mut gmVsMelee_VsExitInfo.match_end;
    // an online match in a room: back to ONLINE PLAY (same room) instead of the CSS
    if ext::Netplay_RematchPending() != 0 {
        let mut w: i32 = -1;
        if !ext::gm_WasMatchCanceled((*match_end).outcome) && (*match_end).n_winners == 1 && (*match_end).winners[0] <= 1 {
            w = (*match_end).winners[0] as i32;
        }
        ext::Netplay_GameResult(w);
        gmVsMelee_UpdateKOCounts(ko, match_end);
        ext::Frontend_BackToOnline();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_FRONTEND);
        return;
    }
    if !ext::gm_WasMatchCanceled((*match_end).outcome) {
        ext::gm_80168638(match_end);
        ext::gm_80168710(match_end, vs);
    }

    gmVsMelee_UpdateKOCounts(ko, match_end);

    if gmVsMelee_WasAnyPlayerHuman(&*match_end) {
        ext::gm_8016247C(ext::gm_801688AC(match_end));
        if (*state.add(1)).id as i32 != GM_GAMEMODESTATE_TERMINATE {
            ext::gm_GetVsPlayMatchTotal();
            let mut unk_bool = false;
            let idx = findSmallestLoser(match_end) as usize;
            let mut unk = ext::gm_80172DD4((*ext::gmMainLib_8015ED98()).x0);
            let none = ChKind_None as i32;
            if unk as i32 != none {
                ext::gm_InitChallengerData(
                    (*match_end).player_standings[idx].ckind as u8,
                    (*match_end).player_standings[idx].anon_3.anon_0.x3_b0(),
                    idx as u8,
                    (*match_end).player_standings[idx].x4,
                    unk,
                    0,
                );
                ext::gm_SetNextGameModeStateId(gmVsMode_State_Approach);
                unk_bool = true;
            } else if {
                unk = ext::gm_80172D78();
                unk as i32 != none
            } {
                ext::gm_InitChallengerData(
                    (*match_end).player_standings[idx].ckind as u8,
                    (*match_end).player_standings[idx].anon_3.anon_0.x3_b0(),
                    idx as u8,
                    (*match_end).player_standings[idx].x4,
                    unk,
                    0,
                );
                ext::gm_SetNextGameModeStateId(gmVsMode_State_Approach);
                unk_bool = true;
            } else if {
                unk = ext::gm_80172E74();
                unk as i32 != none
            } {
                ext::gm_InitChallengerData(
                    (*match_end).player_standings[idx].ckind as u8,
                    (*match_end).player_standings[idx].anon_3.anon_0.x3_b0(),
                    idx as u8,
                    (*match_end).player_standings[idx].x4,
                    unk,
                    0,
                );
                ext::gm_SetNextGameModeStateId(gmVsMode_State_Approach);
                unk_bool = true;
            }
            let foo = ext::gm_80172F00((*ext::gmMainLib_8015EDB0()).x0 as u32);
            if foo != 328 {
                ext::gm_80164504(foo);
            }
            ext::gm_80173DE4(&mut gmVsMelee_VsExitInfo.match_end);
            ext::gm_80172898(1);
            ext::gm_80173EEC();
            if !unk_bool && ext::gm_801721EC() {
                ext::gm_InitChallengerData(ChKind_None as u8, 0, idx as u8, 120, unk, 0);
                ext::gm_SetNextGameModeStateId(gmVsMode_State_Prize);
                unk_bool = true;
            }
            if unk_bool {
                lbCardGame_SetupArchive();
                return;
            }
        }
    }

    lbCardGame_SetupArchive();
    ext::gm_SetNextGameModeStateId(state_id);
}

/// gmvsmode.h `gmVsMode_StateId` values the hooks above hand to `gm_SetNextGameModeStateId`.
pub const gmVsMode_State_Css: u8 = 0;
pub const gmVsMode_State_Sss: u8 = 1;
pub const gmVsMode_State_Vs: u8 = 2;
pub const gmVsMode_State_SuddenDeath: u8 = 3;
pub const gmVsMode_State_Results: u8 = 4;
pub const gmVsMode_State_Approach: u8 = 2 << 6;
pub const gmVsMode_State_ApproachVs: u8 = (2 << 6) + 1;
pub const gmVsMode_State_Prize: u8 = 3 << 6;
pub const gmVsMode_State_Loading: u8 = 1 << 6;
