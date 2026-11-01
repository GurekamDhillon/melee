#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::all)]
//! Native port of src/melee/gm/gmvsmelee.c (the whole TU, TARGET_PC build: VS Melee's CSS/SSS/match/
//! sudden-death/results hooks, the KO-star counts and the mode's OnInit/OnLoad, including the PC
//! online/scene-launch/m-ex branches). Calls out of the TU go through `ext` (a port where one
//! exists in this crate, else a TODO panic). Checked against the C (whole TU included verbatim,
//! shipping contraction) by tests/tu_gmvsmelee.rs.
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::lb::lbTime_8000AF74 as lbTime_8000AF74_gm_r4;
#[allow(unused_imports)]
use mt::MatchEnd as MatchEnd_gm_r4;
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::MatchEnd as MatchEnd_gm_ptr;
#[allow(unused_imports)]
use crate::gm_ptr::{PKIND_HUMAN};
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
        SceneLaunch_SeedVs(vs: *mut VsModeData, dummy_fallback: bool) -> bool = |vs, d| unsafe { crate::tu::gmscenelaunch::SceneLaunch_SeedVs(vs, d as i32) != 0 };
        SceneLaunch_Active() -> i32 = || todo!("TODO(port) SceneLaunch_Active (pc/platform)");
        SceneLaunch_EntryStateId() -> i32 = || todo!("TODO(port) SceneLaunch_EntryStateId (pc/platform)");
        SceneLaunch_PlayerStocks(slot: i32) -> i32 = |_| todo!("TODO(port) SceneLaunch_PlayerStocks (pc/platform)");
        // rp3 SceneLaunch_HudCount
        SceneLaunch_HudCount(start: *mut StartMeleeData) = |start: *mut StartMeleeData| unsafe { crate::tu::gm_1B03::SceneLaunch_HudCount(&mut *(start as *mut _)); };
        // rp3-end
        OSReport(msg: &str) = |_| todo!("TODO(port) OSReport");
        gm_GetGameModeStateEnterData(state: *mut GameModeState) -> *mut c_void = |state: *mut GameModeState| -> *mut c_void { unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateEnterData(state as _) as _ } };
        gm_GetGameModeStateExitData(state: *mut GameModeState) -> *mut c_void = |state: *mut GameModeState| -> *mut c_void { unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateExitData(state as _) as _ } };
        gm_ChangeGameModeAfterCurrentScene(pending_mode: i32) = |pending_mode: i32| { unsafe { crate::tu::gm_1A3F::gm_ChangeGameModeAfterCurrentScene(pending_mode as _); } };
        gm_SetNextGameModeStateId(id: u8) = |id: u8| { unsafe { crate::tu::gm_1A3F::gm_SetNextGameModeStateId(id as _); } };
        gm_InitVsMode(vs: *mut VsModeData) = |vs: *mut VsModeData| { unsafe { crate::tu::gm_1601::gm_InitVsMode(vs as _); } };
        gmMainLib_8015CDEC() = || { unsafe { crate::tu::gmmain_lib::gmMainLib_8015CDEC(); } };
        lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState = || -> *mut PreloadedGameModeState { unsafe { melee_runtime::tu::lbdvd::lbDvd_GetPreloadCacheScene() as _ } };
        lbDvd_SetupVsPreloadCache() = || { unsafe { melee_runtime::tu::lbdvd::lbDvd_SetupVsPreloadCache(); } };
        Frontend_OnlinePick() -> i32 = || -> i32 { unsafe { crate::tu::gmfrontend::Frontend_OnlinePick() as _ } };
        Frontend_OnlinePicked(which: i32, a: i32, b: i32) = |which: i32, a: i32, b: i32| { unsafe { crate::tu::gmfrontend::Frontend_OnlinePicked(which as _, a as _, b as _); } };
        Frontend_BackToOnline() = || { unsafe { crate::tu::gmfrontend::Frontend_BackToOnline(); } };
        lbAudioAx_80026E84(ckind: CharacterKind) -> u64 = |ckind: CharacterKind| -> u64 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026E84(ckind as _) as _ } };
        lbAudioAx_80026F2C(arg: u32) = |arg: u32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(arg as _); } };
        lbAudioAx_8002702C(flags: u32, mask: u64) = |flags: u32, mask: u64| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(flags as _, mask as _); } };
        lbAudioAx_80027168() = || { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027168(); } };
        lbAudioAx_80026EBC(stkind: StKind) -> u64 = |stkind: StKind| -> u64 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026EBC(stkind as _) as _ } };
        gm_80167FC4(sss: *mut SSSData) = |sss: *mut SSSData| { unsafe { crate::tu::gm_1601::gm_80167FC4(sss as _); } };
        gm_80167BC8(vs: *mut VsModeData) = |vs: *mut VsModeData| { unsafe { crate::tu::gm_1601::gm_80167BC8(vs as _); } };
        // rp3 gm_SetupSubColors
        gm_SetupSubColors(start: *mut StartMeleeData) = |start: *mut StartMeleeData| unsafe { crate::tu::gm_1B03::gm_SetupSubColors(&mut *(start as *mut _)); };
        // rp3-end
        gm_LoadRumbleEnabled(start: *mut StartMeleeData) = |start: *mut StartMeleeData| { unsafe { crate::tu::gmvs::gm_LoadRumbleEnabled(start as _); } };
        gm_LoadAnnouncer() = || { unsafe { crate::tu::gm_1601::gm_LoadAnnouncer(); } };
        Netplay_StageEnd(winner: i32, s0: i32, s1: i32) = |_, _, _| todo!("TODO(port) Netplay_StageEnd (gw_netplay.c)");
        Netplay_RunActive() -> i32 = || todo!("TODO(port) Netplay_RunActive (gw_netplay.c)");
        Netplay_GameResult(winner: i32) = |_| todo!("TODO(port) Netplay_GameResult (gw_netplay.c)");
        Netplay_Enabled() -> i32 = || todo!("TODO(port) Netplay_Enabled (gw_netplay.c)");
        Netplay_NoteTie() = || todo!("TODO(port) Netplay_NoteTie (gw_netplay.c)");
        Netplay_RematchPending() -> i32 = || todo!("TODO(port) Netplay_RematchPending (gw_netplay.c)");
        gm_WasMatchCanceled(outcome: u8) -> bool = crate::gm::gm_WasMatchCanceled;
        gm_MatchHasMultipleWinners(end: *mut MatchEnd) -> bool = |e| crate::gm_r4::gm_MatchHasMultipleWinners(unsafe { &*e });
        gm_80162574(ckind: u8, outcome: u8) = |ckind: u8, outcome: u8| { unsafe { crate::tu::gm_1601::gm_80162574(ckind as _, outcome as _); } };
        gm_SetupHumanResultsScreen(match_kind: u8, outcome: u8) = |match_kind: u8, outcome: u8| { unsafe { crate::tu::gm_1601::gm_SetupHumanResultsScreen(match_kind as _, outcome as _); } };
        gm_SetupResultsScreenPlayTime(seconds: u32, arg: u32) = |seconds: u32, arg: u32| { unsafe { crate::tu::gm_1601::gm_SetupResultsScreenPlayTime(seconds as _, arg as _); } };
        gm_80162800(end: *mut MatchEnd) -> u32 = |end: *mut MatchEnd| -> u32 { unsafe { crate::tu::gm_1601::gm_80162800(end as _) as _ } };
        // rp3 gm_SetupSuddenDeath
        gm_SetupSuddenDeath(start: *mut StartMeleeData, end: *mut MatchEnd) = |start: *mut StartMeleeData, end: *mut MatchEnd| unsafe { crate::tu::gm_1B03::gm_SetupSuddenDeath(&mut *(start as *mut _), &*(end as *const _)); };
        // rp3-end
        // rp3 gm_80166CCC
        gm_80166CCC(a: *mut MatchEnd, b: *mut MatchEnd) = |a: *mut MatchEnd, b: *mut MatchEnd| unsafe { crate::gm_ptr::gm_80166CCC(&mut *(a as *mut _), &mut *(b as *mut _)); };
        // rp3-end
        // rp3 gm_80177724
        gm_80177724(info: *mut ResultsMatchInfo) = |info: *mut ResultsMatchInfo| unsafe { crate::gm_r7::gm_80177724(&mut *(info as *mut _)); };
        // rp3-end
        gm_80168638(end: *mut MatchEnd) = |end: *mut MatchEnd| { unsafe { crate::tu::gm_1601::gm_80168638(end as _); } };
        gm_80168710(end: *mut MatchEnd, vs: *mut VsModeData) = |end: *mut MatchEnd, vs: *mut VsModeData| { unsafe { crate::tu::gm_1601::gm_80168710(end as _, vs as _); } };
        gm_8016247C(arg: i32) -> i32 = |arg: i32| -> i32 { unsafe { crate::tu::gm_1601::gm_8016247C(arg as _) as _ } };
        gm_801688AC(end: *mut MatchEnd) -> i32 = |e| crate::gm_r10::gm_801688AC(unsafe { &*e });
        gm_GetVsPlayMatchTotal() -> u32 = || -> u32 { unsafe { crate::tu::gm_1601::gm_GetVsPlayMatchTotal() as _ } };
        gm_80172DD4(arg: u32) -> u8 = |arg: u32| -> u8 { unsafe { crate::tu::gm_16f1::gm_80172DD4(arg as _) as _ } };
        gmMainLib_8015ED98() -> *mut gmm_retval_ED98 = || -> *mut gmm_retval_ED98 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015ED98() as _ } };
        // rp3 gm_InitChallengerData
        gm_InitChallengerData(human_ckind: u8, human_color: u8, human_slot: u8, human_nametag: u8, cpu_ckind: u8, curr_mode: u8) = |human_ckind: u8, human_color: u8, human_slot: u8, human_nametag: u8, cpu_ckind: u8, curr_mode: u8| unsafe { crate::tu::gm_1736::gm_InitChallengerData(human_ckind, human_color, human_slot, human_nametag, cpu_ckind, curr_mode); };
        // rp3-end
        gm_80172D78() -> u8 = || -> u8 { unsafe { crate::tu::gm_16f1::gm_80172D78() as _ } };
        gm_80172E74() -> u8 = || -> u8 { unsafe { crate::tu::gm_16f1::gm_80172E74() as _ } };
        gm_80172F00(arg: u32) -> u16 = |arg: u32| -> u16 { unsafe { crate::tu::gm_16f1::gm_80172F00(arg as _) as _ } };
        gmMainLib_8015EDB0() -> *mut gmm_retval_EDB0 = || -> *mut gmm_retval_EDB0 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EDB0() as _ } };
        gm_80164504(arg: u16) = |arg: u16| { unsafe { crate::tu::gm_1601::gm_80164504(arg as _); } };
        gm_80173DE4(end: *mut MatchEnd) = |end: *mut MatchEnd| { unsafe { crate::tu::gm_1736::gm_80173DE4(end as _); } };
        gm_80172898(arg: u16) -> i32 = |arg: u16| -> i32 { unsafe { crate::tu::gm_16f1::gm_80172898(arg as _) as _ } };
        gm_80173EEC() = || { unsafe { crate::tu::gm_1736::gm_80173EEC(); } };
        gm_801721EC() -> bool = || -> bool { unsafe { crate::tu::gm_16f1::gm_801721EC() != 0 } };
        lbCardNew_AllocWorkArea() = || { unsafe { melee_runtime::tu::lbcardnew::lbCardNew_AllocWorkArea(); } };
        lbCardGame_LoadArchive(arg: i32) = |arg: i32| { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_LoadArchive(arg as _); } };
        lbCardGame_SaveChanges() = || { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_SaveChanges(); } };
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
    melee_core::ct!("gmVsMelee_GetVsData");
    &mut (*gmMainLib_804D3EE0).modes.table[GmVsMode_Melee as usize]
}

/// 1A5250
pub unsafe fn gmVsMelee_GetKOCounts() -> *mut u8 {
    melee_core::ct!("gmVsMelee_GetKOCounts");
    ko_counts.as_mut_ptr()
}

/// 1A5258: per human player, `lbTime_8000AF74` of the count and the standing's x20.
pub unsafe fn gmVsMelee_UpdateKOCounts(ko: *mut u8, end: *mut MatchEnd) {
    melee_core::ct!("gmVsMelee_UpdateKOCounts");
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
    melee_core::ct!("findSmallestLoser");
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
    melee_core::ct!("gmVsMelee_Mode_OnInit");
    ext::gm_InitVsMode(&mut (*gmMainLib_804D3EE0).modes.table[GmVsMode_Melee as usize]);
    ext::gmMainLib_8015CDEC();
}

/// 1A55C4
pub unsafe fn gmVsMelee_ResetKOCounts() {
    melee_core::ct!("gmVsMelee_ResetKOCounts");
    ext::memzero(ko_counts.as_mut_ptr() as *mut c_void, core::mem::size_of_val(&ko_counts) as isize);
}

/// 1A55EC
pub unsafe fn gmVsMelee_Mode_OnLoad() {
    melee_core::ct!("gmVsMelee_Mode_OnLoad");
    // m-ex "Disable KO Star Deletion When Leaving CSS" (opt-in MELEE_MEX=keep_ko_stars), then the
    // scene launch seed (MELEE_SCENE=mode=vs;...), TARGET_PC.
    if ext::Mex_Enabled("keep_ko_stars") != 0 {
        return;
    }
    ext::SceneLaunch_SeedVs(&mut (*gmMainLib_804D3EE0).modes.table[GmVsMode_Melee as usize], false);
    gmVsMelee_ResetKOCounts();
}

/// 1A5614
pub fn gm_Mode_Vs_OnUnload() { melee_core::ct!("gm_Mode_Vs_OnUnload");}

/// 1A5618
pub unsafe fn gmVsMelee_EnterCss(state: *mut GameModeState, vs: *mut VsModeData, match_type: CSSMatchType) {
    melee_core::ct!("gmVsMelee_EnterCss");
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
    melee_core::ct!("gmVsMelee_ExitCss");
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
    melee_core::ct!("gmVsMelee_EnterSss");
    let sss = ext::gm_GetGameModeStateEnterData(state) as *mut SSSData;
    (*sss).vs = *vs;
    ext::gm_80167FC4(sss);
}

/// 1A57A8
pub unsafe fn gmVsMelee_ExitSss(state: *mut GameModeState, vs: *mut VsModeData, cancel_state_id: u8) {
    melee_core::ct!("gmVsMelee_ExitSss");
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
    melee_core::ct!("gmVsMelee_EnterVs");
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
    melee_core::ct!("gmVsMelee_ExitVs");
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
    melee_core::ct!("gmVsMelee_EnterSuddenDeath");
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
    melee_core::ct!("gmVsMelee_ExitSuddenDeath");
    let exit_data = ext::gm_GetGameModeStateExitData(state) as *mut MatchExitInfo;
    ext::gm_80166CCC(&mut gmVsMelee_VsExitInfo.match_end, &mut (*exit_data).match_end);
}

/// 1A5F00
pub unsafe fn gmVsMelee_EnterResults(state: *mut GameModeState) {
    melee_core::ct!("gmVsMelee_EnterResults");
    let enter_data = ext::gm_GetGameModeStateEnterData(state) as *mut ResultsMatchInfo;
    ext::gm_80177724(enter_data);
    (*enter_data).match_end = gmVsMelee_VsExitInfo.match_end;
}

/// 1A5F64
pub unsafe fn gmVsMelee_ExitResults(state: *mut GameModeState, vs: *mut VsModeData, state_id: u8) {
    melee_core::ct!("gmVsMelee_ExitResults");
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

// ---- folded from gm_ptr.rs (consolidation steps 21/22) ----
/// gmvsmelee.c `gmVsMelee_WasAnyPlayerHuman_fp` (GM_MAX_PLAYERS = 6).
pub fn gmVsMelee_WasAnyPlayerHuman_fp(end: &MatchEnd_gm_ptr) -> bool {
    melee_core::ct!("gmVsMelee_WasAnyPlayerHuman_fp");
    end.player_standings.iter().any(|p| p.pkind == PKIND_HUMAN)
}

// ---- folded from gm_r4.rs (consolidation steps 21/22) ----
/// gmvsmelee.c `gmVsMelee_UpdateKOCounts_fp`: per human player, `lbTime_8000AF74_gm_r4` of the count
/// and the standing's x20 (u8 -> u32 in, u32 -> int for x20, truncated back to u8).
pub fn gmVsMelee_UpdateKOCounts_fp(ko_counts_p: &mut [u8], end: &MatchEnd_gm_r4) {
    melee_core::ct!("gmVsMelee_UpdateKOCounts_fp");
    for i in 0..mt::GM_MAX_PLAYERS as usize {
        if end.player_standings[i].pkind as i32 == mt::Gm_PKind_Human as i32 {
            ko_counts_p[i] = lbTime_8000AF74_gm_r4(ko_counts_p[i] as u32, end.player_standings[i].x20 as i32) as u8;
        }
    }
}
