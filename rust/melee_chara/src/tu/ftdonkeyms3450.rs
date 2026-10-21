//! src/melee/ft/kinds/ftDonkey/ftdonkeyms3450.c: DK cargo-hold turn check.
//! No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftdonkeyms3450.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use melee_types::*;

/// Calls into TUs that are not ported yet (settable hooks; unset they panic, except the ones
/// with a verified port in melee_fighter, which they fall back to).
pub mod ext {
    use melee_types::*;
    pub static mut FTCO_800C97A8: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftCo_800C97A8(g: *mut HSD_GObj) -> bool {
        match unsafe { FTCO_800C97A8 } {
            Some(f) => f(g),
            None => melee_fighter::tu::ftCo_Turn::ftCo_800C97A8(g),
        }
    }
    pub static mut FTCO_TURN_ENTER: Option<fn(*mut HSD_GObj, i32, u32, f32, f32, f32)> = None;
    pub fn ftCo_Turn_Enter(g: *mut HSD_GObj, msid: i32, flags: u32, a3: f32, frames_to_turn: f32, anim_start: f32) {
        match unsafe { FTCO_TURN_ENTER } {
            Some(f) => f(g, msid, flags, a3, frames_to_turn, anim_start),
            None => melee_fighter::tu::ftCo_Turn::ftCo_Turn_Enter(g, msid, flags, a3, frames_to_turn, anim_start),
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ifSomething` (static).
pub fn ifSomething(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = (*fp).x2CC;
        ext::ftCo_Turn_Enter(gobj, (*a).motion_state + 4, 0, 0.0, (*a).cargo_hold.x20_TURN_SPEED, 0.0);
    }
}

/// Starts the cargo turn when the common turn check fires.
pub fn ftDk_MS_345_800E0134(gobj: *mut HSD_GObj) -> bool {
    if ext::ftCo_800C97A8(gobj) {
        ifSomething(gobj);
        return true;
    }
    false
}
