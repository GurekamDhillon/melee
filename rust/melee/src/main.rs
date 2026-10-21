//! `melee`: the native executable. Links every port crate; boots nothing yet.
use melee_core::hash::{StateHash, StateHasher};

fn main() {
    // Touch each crate so the link is real.
    let _ = (melee_types::melee_core::VERSION, melee_runtime::melee_core::VERSION,
             melee_world::melee_core::VERSION, melee_fighter::melee_core::VERSION,
             melee_chara::melee_core::VERSION, melee_items::melee_core::VERSION,
             melee_modes::melee_core::VERSION);
    let mut h = StateHasher::new();
    1.0f32.state_hash(&mut h);
    println!("melee (rust port skeleton) core {} hash-selftest {:016x}", melee_core::VERSION, h.finish());
}
