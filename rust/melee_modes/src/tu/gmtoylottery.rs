#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]
//! Native port of src/melee/gm/gmtoylottery.c (the whole TU: the Toy Lottery mode state table
//! with its enter/exit hooks). Calls out of the TU go through `ext`. Checked against the C (whole
//! TU included verbatim, shipping contraction) by tests/tu_gmtoylottery.rs.
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

/// Calls out of the TU (gm_1A3F.c, gm_16F1.c, lbcardnew.c, lbcardgame.c: unported).
pub mod ext {
    use crate::tu::ext_fns_c as ext_fns;
    ext_fns! {
        lbCardNew_AllocWorkArea() = || { unsafe { melee_runtime::tu::lbcardnew::lbCardNew_AllocWorkArea(); } };
        lbCardGame_LoadArchive(arg: i32) = |arg: i32| { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_LoadArchive(arg as _); } };
        gm_80172898(arg: u16) -> i32 = |arg: u16| -> i32 { unsafe { crate::tu::gm_16f1::gm_80172898(arg as _) as _ } };
        gm_80173754(a: u8, b: u8) -> bool = |a: u8, b: u8| -> bool { unsafe { crate::tu::gm_1736::gm_80173754(a as _, b as _) != 0 } };
        gm_SetPendingGameMode(mode: u8) = |mode: u8| { unsafe { crate::tu::gm_1A3F::gm_SetPendingGameMode(mode as _); } };
        gm_SetNewGameModePending() = || { unsafe { crate::tu::gm_1A3F::gm_SetNewGameModePending(); } };
    }
}

/* 4D6910 */ pub static mut exit_data: [*mut c_void; 2] = [null_mut(); 2];

/// `gm_Mode_ToyLottery_States`: one state (preload `lbDvdPreload_2`) and the `{ -1 }` terminator.
pub static mut gm_Mode_ToyLottery_States: [GameModeState; 2] = [
    GameModeState {
        id: 0,
        preload: lbDvdPreload_2 as u8,
        flags: 0,
        on_enter: Some(onEnter),
        on_exit: Some(onExit),
        info: GameSceneInfo {
            scene_kind: GS_TOY_LOTTERY as u8,
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

/// `onEnter` (static, 1BECA8).
pub unsafe extern "C" fn onEnter(_state: *mut GameModeState) {
    melee_core::ct!("onEnter");
    ext::lbCardNew_AllocWorkArea();
    ext::lbCardGame_LoadArchive(0);
}

/// `onExit` (static, 1BECD0).
pub unsafe extern "C" fn onExit(_state: *mut GameModeState) {
    melee_core::ct!("onExit");
    ext::gm_80172898(4);
    if !ext::gm_80173754(1, 0) {
        ext::gm_SetPendingGameMode(GM_MENU as u8);
        ext::gm_SetNewGameModePending();
    }
}
