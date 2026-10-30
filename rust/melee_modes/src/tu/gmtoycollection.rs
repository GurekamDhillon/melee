//! src/melee/gm/gmtoycollection.c: the trophy-collection game mode (state table and its exit
//! hook). No float arithmetic.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::all)]

use core::ptr::{addr_of_mut, null_mut};

use melee_types::*;

pub mod ext {
    crate::gmtou_ext! {
        fn gm_SetPendingGameMode(mode: i32);
        fn gm_SetNewGameModePending();
    }
}

/// `gm_804D6918[2]` (UNK_T): the scene's exit-data slot.
pub static mut gm_804D6918: [*mut core::ffi::c_void; 2] = [null_mut(); 2];

/// `gm_Mode_ToyCollection_States`.
pub static mut gm_Mode_ToyCollection_States: [GameModeState; 2] = [
    GameModeState {
        id: 0,
        preload: lbDvdPreload_2 as u8,
        flags: 0,
        on_enter: None,
        on_exit: Some(gm_801BED14),
        info: GameSceneInfo {
            scene_kind: GS_TOY_COLLECTION as u8,
            enter_data: null_mut(),
            exit_data: addr_of_mut!(gm_804D6918) as *mut core::ffi::c_void,
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

pub unsafe extern "C" fn gm_801BED14(_arg: *mut GameModeState) {
    melee_core::ct!("gm_801BED14");
    ext::gm_SetPendingGameMode(GM_MENU as i32);
    ext::gm_SetNewGameModePending();
}
