//! src/melee/gm/gmtitlemode.c: the title game mode (state table, `gmTitleMode_OnEnter`,
//! `onExit`). No float arithmetic, so no fused sites.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::all)]

use core::ptr::{addr_of_mut, null_mut};

use melee_types::*;

use super::gmtoulib::PAD_BUTTON_START;

pub mod ext {
    pub static mut DbLevel: u32 = 0;
    crate::gmtou_ext! {
        fn lbDvd_SetupVsPreloadCache();
        fn gm_SetPendingGameMode(mode: i32);
        fn gm_SetNewGameModePending();
        fn gm_80173EEC();
        fn gm_80172898(arg0: i32);
        fn gm_80173754(arg0: i32, arg1: i32) -> i32;
        fn gm_801BF708(arg0: i32);
    }
}

/// `struct exitData { int buttons; UNK_T x4; }`, the title scene's exit data.
#[repr(C)]
pub struct exitData {
    pub buttons: i32,
    pub x4: *mut core::ffi::c_void,
}
pub static mut exit_data: exitData = exitData { buttons: 0, x4: null_mut() };

/// `gm_Mode_Title_States`.
pub static mut gm_Mode_Title_States: [GameModeState; 2] = [
    GameModeState {
        id: 0,
        preload: lbDvdPreload_3 as u8,
        flags: 0,
        on_enter: Some(gmTitleMode_OnEnter),
        on_exit: Some(onExit),
        info: GameSceneInfo {
            scene_kind: GS_TITLE as u8,
            enter_data: null_mut(),
            exit_data: addr_of_mut!(exit_data) as *mut core::ffi::c_void,
        },
    },
    GameModeState {
        id: 0xFF,
        preload: 0,
        flags: 0,
        on_enter: None,
        on_exit: None,
        info: GameSceneInfo { scene_kind: 0, enter_data: null_mut(), exit_data: null_mut() },
    },
];

pub unsafe extern "C" fn gmTitleMode_OnEnter(_state: *mut GameModeState) {
    ext::lbDvd_SetupVsPreloadCache();
}

pub unsafe extern "C" fn onExit(scene: *mut GameModeState) {
    // gm_GetGameModeStateExitData(scene)
    let buttons = (*scene).info.exit_data as *mut i32;
    // TARGET_PC: no /develop.ini on retail ISOs, so DbLevel stays Master: let Y (or B) open the
    // debug menu.
    if *buttons & (HSD_PAD_Y | HSD_PAD_B) != 0 {
        ext::gm_SetPendingGameMode(GM_DEBUG as i32);
        ext::gm_SetNewGameModePending();
        return;
    }
    if ext::DbLevel >= DbLKind_DebugRom {
        if *buttons & HSD_PAD_A != 0 {
            ext::gm_SetPendingGameMode(GM_DEBUG_VS as i32);
        } else if *buttons & PAD_BUTTON_START as i32 != 0 {
            ext::gm_80173EEC();
            ext::gm_80172898(0x100);
            if ext::gm_80173754(1, 0) == 0 {
                ext::gm_SetPendingGameMode(GM_MENU as i32);
            }
        } else if *buttons & HSD_PAD_X != 0 {
            ext::gm_SetPendingGameMode(GM_DEBUG_SOUND_TEST as i32);
        } else if *buttons & HSD_PAD_Y != 0 {
            ext::gm_SetPendingGameMode(GM_DEBUG as i32);
        } else {
            ext::gm_801BF708(1);
            ext::gm_SetPendingGameMode(GM_OPENING_MV as i32);
        }
    } else if *buttons & PAD_BUTTON_START as i32 != 0 {
        ext::gm_80173EEC();
        ext::gm_80172898(0x100);
        if ext::gm_80173754(1, 0) == 0 {
            ext::gm_SetPendingGameMode(GM_MENU as i32);
        }
    } else {
        ext::gm_801BF708(1);
        ext::gm_SetPendingGameMode(GM_OPENING_MV as i32);
    }
    ext::gm_SetNewGameModePending();
}
