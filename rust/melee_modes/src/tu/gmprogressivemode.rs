//! Port of src/melee/gm/gmprogressivemode.c: the progressive-scan prompt's game mode table and its
//! two state callbacks.
#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_void;
use core::ptr::addr_of_mut;

use melee_types::{lbDvdPreload_2, GameModeState, GameSceneInfo, GM_BOOT, GS_PROG_SCAN};

pub mod ext {
    use super::*;
    crate::oext_fns! {
        /// gm_1A3F.c
        fn gm_GetGameModeStateEnterData(scene: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateEnterData__d31bdf;
        /// gm_1A3F.c
        fn gm_GetGameModeStateExitData(state: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateExitData__c3c609;
        /// gm_1A3F.c
        fn gm_ChangeGameModeAfterCurrentScene(pending_mode: i32) -> () = crate::tu::adapt::gm_ChangeGameModeAfterCurrentScene__2292c1;
    }
}

crate::globals! {
    /// `static UNK_T gm_804D6938` (void*)
    rw pub gm_804D6938: *mut c_void = core::ptr::null_mut();
    /// `static UNK_T gm_804D693C` (void*)
    rw pub gm_804D693C: *mut c_void = core::ptr::null_mut();
}

/// `gm_Mode_ProgScan_States[]`: the one state (enter/exit below) and the terminator (id -1).
pub static mut gm_Mode_ProgScan_States: [GameModeState; 2] = [
    GameModeState {
        id: 0,
        preload: lbDvdPreload_2 as u8,
        flags: 0,
        on_enter: Some(gm_801BF8F8),
        on_exit: Some(gm_801BF920),
        info: GameSceneInfo {
            scene_kind: GS_PROG_SCAN as u8,
            enter_data: unsafe { addr_of_mut!(gm_804D6938) as *mut c_void },
            exit_data: unsafe { addr_of_mut!(gm_804D693C) as *mut c_void },
        },
    },
    GameModeState {
        id: 0xFF,
        preload: 0,
        flags: 0,
        on_enter: None,
        on_exit: None,
        info: GameSceneInfo { scene_kind: 0, enter_data: core::ptr::null_mut(), exit_data: core::ptr::null_mut() },
    },
];

/// gm_801BF8F8: the enter data is a word; set it to 1.
pub unsafe extern "C" fn gm_801BF8F8(arg0: *mut GameModeState) {
    let val = ext::gm_GetGameModeStateEnterData(arg0) as *mut i32;
    *val = 1;
}

/// gm_801BF920: leave to the boot mode.
pub unsafe extern "C" fn gm_801BF920(arg0: *mut GameModeState) {
    ext::gm_GetGameModeStateExitData(arg0);
    ext::gm_ChangeGameModeAfterCurrentScene(GM_BOOT as i32);
}
