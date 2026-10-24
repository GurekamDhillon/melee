//! The C build's file-scope state of baselib/random.c and baselib/state.c + tev.c as native singletons,
//! so callers outside the owning TU (which in C just name the global) can reach the ports that take the
//! state explicitly (`&mut HsdRand`, `&mut HsdRender`). Single-threaded like the game.
#![allow(static_mut_refs)]
use super::state::HsdRender;
use crate::random::HsdRand;

static mut RAND: Option<HsdRand> = None;
static mut RENDER: Option<HsdRender> = None;

/// random.c's `seed` / render seed / forget-memory range.
pub fn rand() -> &'static mut HsdRand {
    unsafe { RAND.get_or_insert_with(HsdRand::default) }
}

/// state.c's / tev.c's statics (`HsdRender`).
pub fn render() -> &'static mut HsdRender {
    unsafe { RENDER.get_or_insert_with(HsdRender::default) }
}
