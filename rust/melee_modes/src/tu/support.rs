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

/// Read `w` bits of a GameCube big-endian bitfield byte run, `shift` bits up from its LSB
/// (the same helper melee_types' generated accessors use).
#[inline]
pub fn bits_get(b: &[u8], shift: u32, w: u32) -> u64 {
    let mut v: u128 = 0;
    for &x in b {
        v = (v << 8) | x as u128;
    }
    ((v >> shift) as u64) & (((1u128 << w) - 1) as u64)
}

#[inline]
pub fn bits_set(b: &mut [u8], shift: u32, w: u32, val: u64) {
    let mut v: u128 = 0;
    for &x in b.iter() {
        v = (v << 8) | x as u128;
    }
    let m = ((1u128 << w) - 1) << shift;
    v = (v & !m) | (((val as u128) << shift) & m);
    for x in b.iter_mut().rev() {
        *x = v as u8;
        v >>= 8;
    }
}
