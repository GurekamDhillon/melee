#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::all)]
//! Native port of src/melee/gm/gmvsmode.c (the whole TU, TARGET_PC build: the VS mode and DebugVs
//! state tables and their enter/exit hooks, including the port's loading screen). Calls out of
//! the TU go through `ext` (a port where one exists in this crate, else a TODO panic). Checked
//! against the C (whole TU included verbatim, shipping contraction) by tests/tu_gmvsmode.rs.
use super::globals::{gmVsMelee_ApproachData, if_Scene_Prize_EnterData};
use super::gmvsmelee::{gmVsMelee_CssData, gmVsMelee_ResultsEnterData, gmVsMelee_SssData, gmVsMelee_StartData, gmVsMelee_SuddenDeathExitInfo, gmVsMelee_VsExitInfo};
pub use super::gmvsmelee::{
    gmVsMode_State_Approach, gmVsMode_State_ApproachVs, gmVsMode_State_Css, gmVsMode_State_Loading, gmVsMode_State_Prize, gmVsMode_State_Results,
    gmVsMode_State_Sss, gmVsMode_State_SuddenDeath, gmVsMode_State_Vs, GM_FRONTEND,
};
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

/// `GS_FRONTEND` (gmfrontend.h, TARGET_PC).
pub const GS_FRONTEND: GameSceneKind = GS_COUNT + 1;

/// Calls out of the TU. `ext::hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use crate::tu::ext_fns_c as ext_fns;
    use crate::tu::gmvsmelee as vm;
    use core::ffi::c_void;
    use melee_types::*;
    ext_fns! {
        gm_GetGameModeStateEnterData(state: *mut GameModeState) -> *mut c_void = |state: *mut GameModeState| -> *mut c_void { unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateEnterData(state as _) as _ } };
        gm_GetGameModeStateExitData(state: *mut GameModeState) -> *mut c_void = |state: *mut GameModeState| -> *mut c_void { unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateExitData(state as _) as _ } };
        gm_SetNextGameModeStateId(id: u8) = |id: u8| { unsafe { crate::tu::gm_1A3F::gm_SetNextGameModeStateId(id as _); } };
        gm_SetupRulesDefaults(rules: *mut StartMeleeRules) = |rules: *mut StartMeleeRules| { unsafe { crate::tu::gm_1601::gm_SetupRulesDefaults(rules as _); } };
        gm_SetupPlayerDefaults(p: *mut PlayerInitData) = |p: *mut PlayerInitData| { unsafe { crate::tu::gm_1601::gm_SetupPlayerDefaults(p as _); } };
        gm_LoadAnnouncer() = || { unsafe { crate::tu::gm_1601::gm_LoadAnnouncer(); } };
        gmFrontend_SelectScene(state: *mut GameModeState, sss: i32) = |state: *mut GameModeState, sss: i32| { unsafe { crate::tu::gmfrontend::gmFrontend_SelectScene(state as _, sss as _); } };
        gmFrontend_BeginLoading() = || { unsafe { crate::tu::gmfrontend::gmFrontend_BeginLoading(); } };
        gmVsMelee_GetVsData() -> *mut VsModeData = || unsafe { vm::gmVsMelee_GetVsData() };
        gmVsMelee_EnterCss(state: *mut GameModeState, vs: *mut VsModeData, mt: CSSMatchType) = |s, v, m| unsafe { vm::gmVsMelee_EnterCss(s, v, m) };
        gmVsMelee_ExitCss(state: *mut GameModeState, vs: *mut VsModeData) = |s, v| unsafe { vm::gmVsMelee_ExitCss(s, v) };
        gmVsMelee_EnterSss(state: *mut GameModeState, vs: *mut VsModeData) = |s, v| unsafe { vm::gmVsMelee_EnterSss(s, v) };
        gmVsMelee_ExitSss(state: *mut GameModeState, vs: *mut VsModeData, cancel: u8) = |s, v, c| unsafe { vm::gmVsMelee_ExitSss(s, v, c) };
        gmVsMelee_EnterVs(state: *mut GameModeState, vs: *mut VsModeData, scb: vm::gm_StartMeleeCallback, pcb: vm::gm_PlayerInitCallback)
            = |s, v, a, b| unsafe { vm::gmVsMelee_EnterVs(s, v, a, b) };
        gmVsMelee_ExitVs(state: *mut GameModeState, id0: u8, id1: u8) = |s, a, b| unsafe { vm::gmVsMelee_ExitVs(s, a, b) };
        gmVsMelee_EnterSuddenDeath(state: *mut GameModeState, vs: *mut VsModeData, scb: vm::gm_StartMeleeCallback, pcb: vm::gm_PlayerInitCallback)
            = |s, v, a, b| unsafe { vm::gmVsMelee_EnterSuddenDeath(s, v, a, b) };
        gmVsMelee_ExitSuddenDeath(state: *mut GameModeState) = |s| unsafe { vm::gmVsMelee_ExitSuddenDeath(s) };
        gmVsMelee_EnterResults(state: *mut GameModeState) = |s| unsafe { vm::gmVsMelee_EnterResults(s) };
        gmVsMelee_ExitResults(state: *mut GameModeState, vs: *mut VsModeData, id: u8) = |s, v, i| unsafe { vm::gmVsMelee_ExitResults(s, v, i) };
        gm_80162A98(arg: i32) -> *mut gmm_x1868_1A8_t = |arg: i32| -> *mut gmm_x1868_1A8_t { unsafe { crate::tu::gm_1601::gm_80162A98(arg as _) as _ } };
        gm_RecordSelfDestructs(amount: i32) = |amount: i32| { unsafe { crate::tu::gm_1601::gm_RecordSelfDestructs(amount as _); } };
        gm_80162A4C(amount: i32) -> i32 = |amount: i32| -> i32 { unsafe { crate::tu::gm_1601::gm_80162A4C(amount as _) as _ } };
        gm_WasMatchCanceled(outcome: u8) -> bool = crate::gm::gm_WasMatchCanceled;
        gm_801623A4(end: *mut MatchEnd) -> i32 = |end: *mut MatchEnd| -> i32 { unsafe { crate::tu::gm_1601::gm_801623A4(end as _) as _ } };
        gm_ModeState_Approach_OnEnter(state: *mut GameModeState) = |state: *mut GameModeState| { unsafe { crate::tu::gm_1BFA::gm_ModeState_Approach_OnEnter(state as _); } };
        gm_ModeState_ApproachVs_OnEnter(state: *mut GameModeState) = |state: *mut GameModeState| { unsafe { crate::tu::gm_1BFA::gm_ModeState_ApproachVs_OnEnter(state as _); } };
        gm_ModeState_ApproachVs_OnExit(state: *mut GameModeState) = |state: *mut GameModeState| { unsafe { crate::tu::gmmovieend::gm_ModeState_ApproachVs_OnExit(state as _); } };
        gm_ModeState_Prize_OnEnter(state: *mut GameModeState) = |state: *mut GameModeState| { unsafe { crate::tu::gm_1BFA::gm_ModeState_Prize_OnEnter(state as _); } };
        gm_ModeState_Prize_OnExit(state: *mut GameModeState) = |state: *mut GameModeState| { unsafe { crate::tu::gmmovieend::gm_ModeState_Prize_OnExit(state as _); } };
    }
}

// The state callbacks of other TUs, as C-ABI function items the state tables can point at.
pub unsafe extern "C" fn gm_ModeState_Approach_OnEnter(s: *mut GameModeState) {
    melee_core::ct!("gm_ModeState_Approach_OnEnter");
    ext::gm_ModeState_Approach_OnEnter(s)
}
pub unsafe extern "C" fn gm_ModeState_ApproachVs_OnEnter(s: *mut GameModeState) {
    melee_core::ct!("gm_ModeState_ApproachVs_OnEnter");
    ext::gm_ModeState_ApproachVs_OnEnter(s)
}
pub unsafe extern "C" fn gm_ModeState_ApproachVs_OnExit(s: *mut GameModeState) {
    melee_core::ct!("gm_ModeState_ApproachVs_OnExit");
    ext::gm_ModeState_ApproachVs_OnExit(s)
}
pub unsafe extern "C" fn gm_ModeState_Prize_OnEnter(s: *mut GameModeState) {
    melee_core::ct!("gm_ModeState_Prize_OnEnter");
    ext::gm_ModeState_Prize_OnEnter(s)
}
pub unsafe extern "C" fn gm_ModeState_Prize_OnExit(s: *mut GameModeState) {
    melee_core::ct!("gm_ModeState_Prize_OnExit");
    ext::gm_ModeState_Prize_OnExit(s)
}

type Cb = Option<unsafe extern "C" fn(*mut GameModeState)>;

const fn state(id: u8, preload: lbDvdPreloadKind, on_enter: Cb, on_exit: Cb, kind: GameSceneKind, enter: *mut c_void, exit: *mut c_void) -> GameModeState {
    GameModeState { id, preload: preload as u8, flags: 0, on_enter, on_exit, info: GameSceneInfo { scene_kind: kind as u8, enter_data: enter, exit_data: exit } }
}
/// `{ GM_GAMEMODESTATE_TERMINATE }`
const TERMINATOR: GameModeState = GameModeState {
    id: GM_GAMEMODESTATE_TERMINATE as u8,
    preload: 0,
    flags: 0,
    on_enter: None,
    on_exit: None,
    info: GameSceneInfo { scene_kind: 0, enter_data: null_mut(), exit_data: null_mut() },
};

/* 3DD9A0 */
pub static mut gm_Mode_Vs_States: [GameModeState; 10] = [
    state(gmVsMode_State_Css, lbDvdPreload_3, Some(onEnterCss), Some(onExitCss), GS_CSS, addr_of_mut!(gmVsMelee_CssData) as *mut c_void, addr_of_mut!(gmVsMelee_CssData) as *mut c_void),
    state(gmVsMode_State_Sss, lbDvdPreload_3, Some(onEnterSss), Some(onExitSss), GS_SSS, addr_of_mut!(gmVsMelee_SssData) as *mut c_void, addr_of_mut!(gmVsMelee_SssData) as *mut c_void),
    state(gmVsMode_State_Vs, lbDvdPreload_3, Some(onEnterVs), Some(onExitVs), GS_VS, addr_of_mut!(gmVsMelee_StartData) as *mut c_void, addr_of_mut!(gmVsMelee_VsExitInfo) as *mut c_void),
    state(
        gmVsMode_State_SuddenDeath,
        lbDvdPreload_3,
        Some(onEnterSuddenDeath),
        Some(onExitSuddenDeath),
        GS_SUDDEN_DEATH,
        addr_of_mut!(gmVsMelee_StartData) as *mut c_void,
        addr_of_mut!(gmVsMelee_SuddenDeathExitInfo) as *mut c_void,
    ),
    state(
        gmVsMode_State_Results,
        lbDvdPreload_3,
        Some(onEnterResults),
        Some(onExitResults),
        GS_RESULTS,
        addr_of_mut!(gmVsMelee_ResultsEnterData) as *mut c_void,
        null_mut(),
    ),
    state(
        gmVsMode_State_Approach,
        lbDvdPreload_2,
        Some(gm_ModeState_Approach_OnEnter),
        None,
        GS_APPROACH,
        addr_of_mut!(gmVsMelee_ApproachData) as *mut c_void,
        addr_of_mut!(gmVsMelee_ApproachData) as *mut c_void,
    ),
    state(
        gmVsMode_State_ApproachVs,
        lbDvdPreload_2,
        Some(gm_ModeState_ApproachVs_OnEnter),
        Some(gm_ModeState_ApproachVs_OnExit),
        GS_VS,
        addr_of_mut!(gmVsMelee_StartData) as *mut c_void,
        addr_of_mut!(gmVsMelee_VsExitInfo) as *mut c_void,
    ),
    state(
        gmVsMode_State_Prize,
        lbDvdPreload_2,
        Some(gm_ModeState_Prize_OnEnter),
        Some(gm_ModeState_Prize_OnExit),
        GS_PRIZE_INTERFACE,
        addr_of_mut!(if_Scene_Prize_EnterData) as *mut c_void,
        null_mut(),
    ),
    // TARGET_PC: the loading screen the SSS jumps to on "start"; it continues to the match
    state(gmVsMode_State_Loading, lbDvdPreload_3, Some(onEnterLoading), Some(onExitLoading), GS_FRONTEND, null_mut(), null_mut()),
    TERMINATOR,
];

const state_debug_vs: u8 = 1;
const state_debug_results: u8 = 3;

/* 3DDA78 */
pub static mut gm_Mode_DebugVs_States: [GameModeState; 3] = [
    state(state_debug_vs, lbDvdPreload_2, Some(onEnterDebugVs), None, GS_VS, addr_of_mut!(gmVsMelee_StartData) as *mut c_void, addr_of_mut!(gmVsMelee_VsExitInfo) as *mut c_void),
    state(state_debug_results, lbDvdPreload_2, Some(onEnterResults), None, GS_RESULTS, addr_of_mut!(gmVsMelee_ResultsEnterData) as *mut c_void, null_mut()),
    TERMINATOR,
];

/// 1B13B8
pub unsafe extern "C" fn onEnterDebugVs(state: *mut GameModeState) {
    melee_core::ct!("onEnterDebugVs");
    let start = ext::gm_GetGameModeStateEnterData(state) as *mut StartMeleeData;

    ext::gm_SetupRulesDefaults(&mut (*start).rules);
    (*start).rules.stkind = St_Kind_Last as u16;
    (*start).rules.item_freq = -1;
    (*start).rules.sd_penalty = -1;
    (*start).rules.set_match_kind(MatchKind_Time as u32);

    for i in 0..Gm_Player_NumMax as usize {
        ext::gm_SetupPlayerDefaults(&mut (*start).players[i]);
        (*start).players[i].stocks = 0;
        (*start).players[i].cpu_kind = 4;
    }

    (*start).players[0].ckind = CKind_Link as i8;
    (*start).players[1].ckind = CKind_Mario as i8;
    (*start).players[2].ckind = CKind_Link as i8;
    (*start).players[3].ckind = CKind_Link as i8;

    (*start).players[0].slot_type = Gm_PKind_Human as u8;
    (*start).players[1].slot_type = Gm_PKind_Human as u8;
    (*start).players[2].slot_type = Gm_PKind_NA as u8;
    (*start).players[3].slot_type = Gm_PKind_NA as u8;

    (*start).players[0].set_rumble_enabled(0);
    (*start).players[1].set_rumble_enabled(0);
    (*start).players[2].set_rumble_enabled(0);
    (*start).players[3].set_rumble_enabled(0);

    ext::gm_LoadAnnouncer();
}

/// 1B14A0
pub unsafe extern "C" fn onEnterCss(state: *mut GameModeState) {
    melee_core::ct!("onEnterCss");
    ext::gmVsMelee_EnterCss(state, ext::gmVsMelee_GetVsData(), VS_MELEE);
    ext::gmFrontend_SelectScene(state, 0); // the port's own character select
}

/// 1B14DC
pub unsafe extern "C" fn onExitCss(state: *mut GameModeState) {
    melee_core::ct!("onExitCss");
    ext::gmVsMelee_ExitCss(state, ext::gmVsMelee_GetVsData());
}

/// 1B1514
pub unsafe extern "C" fn onEnterSss(state: *mut GameModeState) {
    melee_core::ct!("onEnterSss");
    ext::gmVsMelee_EnterSss(state, ext::gmVsMelee_GetVsData());
    ext::gmFrontend_SelectScene(state, 1); // the port's own stage select
}

/// 1B154C
pub unsafe extern "C" fn onExitSss(state: *mut GameModeState) {
    melee_core::ct!("onExitSss");
    ext::gmVsMelee_ExitSss(state, ext::gmVsMelee_GetVsData(), gmVsMode_State_Css);
    // Starting the match goes through the loading screen first.
    if (*(ext::gm_GetGameModeStateExitData(state) as *mut SSSData)).start_game != 0 {
        ext::gm_SetNextGameModeStateId(gmVsMode_State_Loading);
    }
}

/// `onEnterLoading` (static, TARGET_PC)
pub unsafe extern "C" fn onEnterLoading(_state: *mut GameModeState) {
    melee_core::ct!("onEnterLoading");
    ext::gmFrontend_BeginLoading();
}

/// `onExitLoading` (static, TARGET_PC)
pub unsafe extern "C" fn onExitLoading(_state: *mut GameModeState) {
    melee_core::ct!("onExitLoading");
    ext::gm_SetNextGameModeStateId(gmVsMode_State_Vs);
}

/// 1B1588
pub unsafe extern "C" fn onEnterVs(state: *mut GameModeState) {
    melee_core::ct!("onEnterVs");
    ext::gmVsMelee_EnterVs(state, ext::gmVsMelee_GetVsData(), None, None);
}

/// 1B15C8
pub unsafe extern "C" fn onExitVs(state: *mut GameModeState) {
    melee_core::ct!("onExitVs");
    ext::gmVsMelee_ExitVs(state, gmVsMode_State_Results, gmVsMode_State_SuddenDeath);
    let mei = ext::gm_GetGameModeStateExitData(state) as *mut MatchExitInfo;
    for i in 0..GM_MAX_PLAYERS as usize {
        if (*mei).match_end.player_standings[i].pkind as i32 != Gm_PKind_NA as i32 {
            ext::gm_80162A98((*mei).match_end.player_standings[i].x20 as i32);
            ext::gm_RecordSelfDestructs((*mei).match_end.player_standings[i].self_destructs as i32);
            ext::gm_80162A4C((*mei).match_end.player_standings[i].x44 as i32);
        }
    }
}

/// 1B1648
pub unsafe extern "C" fn onEnterSuddenDeath(state: *mut GameModeState) {
    melee_core::ct!("onEnterSuddenDeath");
    ext::gmVsMelee_EnterSuddenDeath(state, ext::gmVsMelee_GetVsData(), None, None);
}

/// 1B1688
pub unsafe extern "C" fn onExitSuddenDeath(state: *mut GameModeState) {
    melee_core::ct!("onExitSuddenDeath");
    ext::gmVsMelee_ExitSuddenDeath(state);
}

/// 1B16A8
pub unsafe extern "C" fn onEnterResults(state: *mut GameModeState) {
    melee_core::ct!("onEnterResults");
    ext::gmVsMelee_EnterResults(state);
}

/// 1B16C8
pub unsafe extern "C" fn onExitResults(state: *mut GameModeState) {
    melee_core::ct!("onExitResults");
    ext::gmVsMelee_ExitResults(state, ext::gmVsMelee_GetVsData(), gmVsMode_State_Css);
    if !ext::gm_WasMatchCanceled(gmVsMelee_ResultsEnterData.match_end.outcome) {
        ext::gm_801623A4(&mut gmVsMelee_ResultsEnterData.match_end);
    }
}
