//! Helpers shared by the TU ports (not a TU itself).
use core::ffi::c_void;
use melee_types::{GameModeState, GameSceneInfo};

pub type StateFn = Option<unsafe extern "C" fn(*mut GameModeState)>;

/// One `GameModeState` table row: `{ id, preload, 0, on_enter, on_exit, { scene, enter, exit } }`.
pub const fn state(id: u8, preload: u32, on_enter: StateFn, on_exit: StateFn, scene: u32,
                   enter: *mut c_void, exit: *mut c_void) -> GameModeState {
    GameModeState {
        id,
        preload: preload as u8,
        flags: 0,
        on_enter,
        on_exit,
        info: GameSceneInfo { scene_kind: scene as u8, enter_data: enter, exit_data: exit },
    }
}

/// The `{ -1 }` row that ends every table.
pub const fn end() -> GameModeState {
    state(0xFF, 0, None, None, 0, core::ptr::null_mut(), core::ptr::null_mut())
}

pub const NULL: *mut c_void = core::ptr::null_mut();
