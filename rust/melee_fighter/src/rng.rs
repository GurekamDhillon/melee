//! The game's global random state (`baselib/random.c`): one `HsdRand` owned here, with the C
//! entry points the fighter TUs call (`HSD_Randi`, `HSD_Randf`, `HSD_RandSeedPtr`). The generator
//! itself is `melee_runtime::random::HsdRand` (checked against the C there).
#![allow(non_snake_case, static_mut_refs)]

use melee_runtime::random::{HsdRand, SeedSlot};

static mut STATE: HsdRand = HsdRand { seed: 1, seed_ptr: SeedSlot::Default, render_seed: 0, render_depth: 0, trace: None };

/// The global state (for savestate / seeding by the match setup).
pub fn state() -> &'static mut HsdRand {
    unsafe { &mut STATE }
}

/// `HSD_Randi(max)`.
pub fn HSD_Randi(max_val: i32) -> i32 {
    melee_runtime::tu::random::HSD_Randi(state(), max_val)
}

/// `HSD_Randf()`.
pub fn HSD_Randf() -> f32 {
    melee_runtime::tu::random::HSD_Randf(state())
}

/// `HSD_RandSeedPtr()`: the seed word the next draw advances.
pub fn HSD_RandSeedPtr() -> *mut u32 {
    melee_runtime::tu::random::HSD_RandCurrentSeed(state())
}
