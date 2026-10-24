#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]
//! Native port of src/melee/gm/gmtoygallery.c (the whole TU: the Toy Gallery mode state table
//! and its exit hook). Calls out of the TU go through `ext`. Checked against the C (whole TU
//! included verbatim, shipping contraction) by tests/tu_gmtoygallery.rs.
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

/// Calls out of the TU (gm_1A3F.c: unported).
pub mod ext {
    use crate::tu::ext_fns_c as ext_fns;
    ext_fns! {
        gm_SetPendingGameMode(mode: u8) = |mode: u8| { unsafe { let _ = crate::tu::gm_1A3F::gm_SetPendingGameMode(mode as _); } };
        gm_SetNewGameModePending() = || { unsafe { let _ = crate::tu::gm_1A3F::gm_SetNewGameModePending(); } };
    }
}

/* 4D6908 */ pub static mut exit_data: [*mut c_void; 2] = [null_mut(); 2];
/* 4D6910 */ static mut toy_lottery_exit_data: [*mut c_void; 2] = [null_mut(); 2];
/* 4D6918 */ pub static mut toy_collection_exit_data: [*mut c_void; 2] = [null_mut(); 2];

/// `gm_Mode_ToyGallery_States`: one state (preload `lbDvdPreload_2`, exit hook `onExit`) and the
/// `{ -1 }` terminator.
pub static mut gm_Mode_ToyGallery_States: [GameModeState; 2] = [
    GameModeState {
        id: 0,
        preload: lbDvdPreload_2 as u8,
        flags: 0,
        on_enter: None,
        on_exit: Some(onExit),
        info: GameSceneInfo {
            scene_kind: GS_TOY_GALLERY as u8,
            enter_data: null_mut(),
            exit_data: addr_of_mut!(exit_data) as *mut c_void,
        },
    },
    GameModeState {
        id: -1i8 as u8,
        preload: 0,
        flags: 0,
        on_enter: None,
        on_exit: None,
        info: GameSceneInfo { scene_kind: 0, enter_data: null_mut(), exit_data: null_mut() },
    },
];

/// `onExit` (static, 1BEC80): leave for the main menu.
pub unsafe extern "C" fn onExit(_state: *mut GameModeState) {
    ext::gm_SetPendingGameMode(GM_MENU as u8);
    ext::gm_SetNewGameModePending();
}
