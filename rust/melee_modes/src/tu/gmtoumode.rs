//! src/melee/gm/gmtoumode.c: the tournament game mode (seven-state table and its enter/exit
//! hooks). No float arithmetic, so no fused sites.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::all)]

use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};

use melee_types::*;

use super::gmtoulib::{gm_801905F0, gm_80477738, gm_804771C4, gm_8018F1B0, gm_GetTournamentData};

pub mod ext {
    use super::*;
    crate::gmtou_ext! {
        fn lbCardNew_AllocWorkArea();
        fn lbCardGame_LoadArchive(arg0: i32);
        fn gm_SetNextGameModeStateId(pending_scene: u8);
        fn gmVsMelee_WasAnyPlayerHuman(me: *mut MatchEnd) -> i32;
        fn gm_SetupHumanResultsScreen(match_kind: u8, outcome: u8);
        fn gm_SetupResultsScreenPlayTime(secs: u32, arg1: u32);
        fn gm_80162800(me: *mut MatchEnd) -> u32;
        fn gm_MatchHasMultipleWinners(me: *mut MatchEnd) -> i32;
        fn gm_SetupSuddenDeath(smd: *mut StartMeleeData, me: *mut MatchEnd);
        fn gm_801629B4(arg0: u32);
        fn gm_8016247C(arg0: i32);
    }
}

/// `gm_804876D8` / `gm_80487810`: the match setup and the match result of the current game.
pub static mut gm_804876D8: StartMeleeData = unsafe { core::mem::zeroed() };
pub static mut gm_80487810: MatchExitInfo = unsafe { core::mem::zeroed() };
/// Raw scene buffers (`UNK_T[0x2288/4]`, `[0x148/4]`, `[0x23C8/4]`), sized in bytes.
pub static mut gm_80489A98: [u8; 0x2288] = [0; 0x2288];
pub static mut gm_8048BD20: [u8; 0x148] = [0; 0x148];
pub static mut gm_8048BE68: [u8; 0x23C8] = [0; 0x23C8];

macro_rules! state {
    ($id:expr, $enter:expr, $exit:expr, $scene:expr, $en:expr, $ex:expr) => {
        GameModeState {
            id: $id,
            preload: lbDvdPreload_3 as u8,
            flags: 0,
            on_enter: $enter,
            on_exit: $exit,
            info: GameSceneInfo { scene_kind: $scene as u8, enter_data: $en, exit_data: $ex },
        }
    };
}

/// `gm_Mode_Tournament_States`.
pub static mut gm_Mode_Tournament_States: [GameModeState; 8] = [
    state!(0, Some(gm_801B1724), None, GS_TOU_SETUP, null_mut(), null_mut()),
    state!(1, None, None, GS_TOU_BRACKET, null_mut(), null_mut()),
    state!(2, None, None, GS_TOU_ALT, null_mut(), null_mut()),
    state!(
        3,
        Some(gm_801B174C),
        Some(gm_801B1788),
        GS_SSS,
        addr_of_mut!(gm_8048BD20) as *mut c_void,
        addr_of_mut!(gm_8048BD20) as *mut c_void
    ),
    state!(
        4,
        Some(gm_801B1810),
        Some(gm_801B1834),
        GS_VS,
        addr_of_mut!(gm_804876D8) as *mut c_void,
        addr_of_mut!(gm_80487810) as *mut c_void
    ),
    state!(
        5,
        Some(gm_801B18D4),
        Some(gm_801B1A2C),
        GS_SUDDEN_DEATH,
        addr_of_mut!(gm_804876D8) as *mut c_void,
        addr_of_mut!(gm_8048BE68) as *mut c_void
    ),
    state!(
        6,
        Some(gm_801B1A84),
        Some(gm_801B1AD4),
        GS_RESULTS,
        addr_of_mut!(gm_80489A98) as *mut c_void,
        null_mut()
    ),
    GameModeState {
        id: 0xFF,
        preload: 0,
        flags: 0,
        on_enter: None,
        on_exit: None,
        info: GameSceneInfo { scene_kind: 0, enter_data: null_mut(), exit_data: null_mut() },
    },
];

pub unsafe extern "C" fn gm_801B1724(_arg0: *mut GameModeState) {
    ext::lbCardNew_AllocWorkArea();
    ext::lbCardGame_LoadArchive(0);
}

pub unsafe extern "C" fn gm_801B174C(arg0: *mut GameModeState) {
    let sss = (*arg0).info.enter_data as *mut SSSData;
    (*sss).x1 = 0;
    (*sss).no_lras = 1;
    (*sss).force_stage_id = -1;
    (*sss).unk_stage = 0;
}

pub unsafe extern "C" fn gm_801B1788(arg0: *mut GameModeState) {
    let sss = (*arg0).info.exit_data as *mut SSSData;
    if (*sss).start_game != 0 {
        ext::gm_SetNextGameModeStateId(4);
        let stage = (*sss).vs.start.rules.stkind;
        (*gm_GetTournamentData()).x28 = stage as u32;
        return;
    }
    if gm_804771C4.match_type == 0 {
        super::gmtou_1::gm_8019A828();
        ext::gm_SetNextGameModeStateId(1);
        return;
    }
    (*gm_GetTournamentData()).x32 = 0;
    ext::gm_SetNextGameModeStateId(2);
}

pub unsafe extern "C" fn gm_801B1810(arg0: *mut GameModeState) {
    let data = (*arg0).info.enter_data as *mut StartMeleeData;
    gm_801905F0(data);
}

pub unsafe extern "C" fn gm_801B1834(state: *mut GameModeState) {
    let mei = (*state).info.exit_data as *mut MatchExitInfo;
    if ext::gmVsMelee_WasAnyPlayerHuman(&mut (*mei).match_end) != 0 {
        ext::gm_SetupHumanResultsScreen((*mei).match_end.match_kind, (*mei).match_end.outcome);
        ext::gm_SetupResultsScreenPlayTime((*mei).match_end.frame_count / GM_FPS as u32, ext::gm_80162800(&mut (*mei).match_end));
    }
    if ext::gm_MatchHasMultipleWinners(&mut (*mei).match_end) != 0 && gm_8018F1B0(&mut (*mei).match_end) != 0 {
        ext::gm_SetNextGameModeStateId(5);
    } else {
        ext::gm_SetNextGameModeStateId(6);
    }
}

pub unsafe extern "C" fn gm_801B18D4(arg0: *mut GameModeState) {
    let src = addr_of_mut!(gm_804876D8);
    let smd = (*arg0).info.enter_data as *mut StartMeleeData;

    let mut i = 0;
    while i < 4 {
        (*smd).players[i] = (*src).players[i];
        i += 1;
    }

    // TARGET_PC: gm_80487810 is not contiguous with gm_804876D8 on PC.
    ext::gm_SetupSuddenDeath(smd, addr_of_mut!(gm_80487810.match_end));
}

pub unsafe extern "C" fn gm_801B1A2C(arg0: *mut GameModeState) {
    let mei = (*arg0).info.exit_data as *mut MatchExitInfo;
    ext::gm_801629B4((*mei).match_end.frame_count / 60);
    crate::gm_ptr::gm_80166CCC(&mut gm_80487810.match_end, &mut (*mei).match_end);
}

pub unsafe extern "C" fn gm_801B1A84(arg0: *mut GameModeState) {
    let rmi = (*arg0).info.enter_data as *mut ResultsMatchInfo;
    (*rmi).match_end = gm_80487810.match_end;
}

pub unsafe extern "C" fn gm_801B1AD4(arg0: *mut GameModeState) {
    let rmi = (*arg0).info.enter_data as *mut ResultsMatchInfo;
    gm_80477738 = (*rmi).match_end;

    ext::gm_8016247C(crate::gm_r10::gm_801688AC(&(*rmi).match_end));
    if gm_804771C4.match_type == 0 {
        ext::gm_SetNextGameModeStateId(1);
        (*gm_GetTournamentData()).cur_option = 0x1F;
        return;
    }
    super::gmtou_2::gm_8019E634();
    ext::gm_SetNextGameModeStateId(2);
}
