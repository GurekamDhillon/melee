//! All TARGET_PC definitions from baselib/random.c, over its existing native
//! state owner. Simulation and render draws delegate to the original Rust port.
#![allow(non_snake_case)]

pub use crate::random::{HsdRand, SeedSlot};

pub fn HSD_RandRenderBegin(state: &mut HsdRand) {
    state.render_begin();
}
pub fn HSD_RandRenderEnd(state: &mut HsdRand) {
    state.render_end();
}

/// random.c's private selector, exposed for callers that need its pointer
/// identity. The pointer is valid while the state (or redirected seed) stays put.
pub fn HSD_RandCurrentSeed(state: &mut HsdRand) -> *mut u32 {
    if state.render_depth != 0 {
        core::ptr::addr_of_mut!(state.render_seed)
    } else {
        match state.seed_ptr {
            SeedSlot::Default => core::ptr::addr_of_mut!(state.seed),
            SeedSlot::Redirected(p) => p,
        }
    }
}

pub fn HSD_Rand(state: &mut HsdRand) -> i32 {
    state.rand()
}
pub fn HSD_Randf(state: &mut HsdRand) -> f32 {
    state.randf()
}
pub fn HSD_Randi(state: &mut HsdRand, max_val: i32) -> i32 {
    state.randi(max_val)
}
pub fn _HSD_RandForgetMemory(
    state: &mut HsdRand,
    low: *const u8,
    high: *const u8,
) {
    state.forget_memory(low, high);
}
/// random.h's inline empty-range wrapper.
pub fn randi(state: &mut HsdRand, max_val: i32) -> i32 {
    state.randi_safe(max_val)
}

/// random.c's file statics (`seed`, `HSD_RandSeedPtr`, `render_seed`, `render_depth`) as the one
/// process-wide state, for the native TUs that call `HSD_Rand` / `HSD_Randf` / `HSD_Randi` with the
/// C signatures (no state argument). The arithmetic is `HsdRand`'s (checked by `oracle_*`).
pub mod global {
    use super::*;
    pub static mut STATE: HsdRand =
        HsdRand { seed: 1, seed_ptr: SeedSlot::Default, render_seed: 0, render_depth: 0, trace: None };

    #[allow(static_mut_refs)]
    #[inline]
    pub fn state() -> &'static mut HsdRand {
        unsafe { &mut *core::ptr::addr_of_mut!(STATE) }
    }
    pub fn HSD_Rand() -> i32 {
        state().rand()
    }
    pub fn HSD_Randf() -> f32 {
        state().randf()
    }
    pub fn HSD_Randi(max_val: i32) -> i32 {
        state().randi(max_val)
    }
    pub fn HSD_RandRenderBegin() {
        state().render_begin()
    }
    pub fn HSD_RandRenderEnd() {
        state().render_end()
    }
}

/// `HSD_Randi(max)` over the one process-wide state ([`global`]).
pub unsafe fn HSD_Randi_global(max_val: i32) -> i32 {
    global::HSD_Randi(max_val)
}
/// `HSD_Randf()` over the one process-wide state ([`global`]).
pub unsafe fn HSD_Randf_global() -> f32 {
    global::HSD_Randf()
}
