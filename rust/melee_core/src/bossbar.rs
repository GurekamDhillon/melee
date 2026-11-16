//! The boss HP bar's snapshot: written by the boss host each logic frame, read by the presentation's overlay
//! (melee_platform::overlay::boss_bar). Display only: nothing the simulation reads, so rollback and hashes do not
//! see it (a restored frame rewrites it on its next tick).
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

static HP: AtomicU32 = AtomicU32::new(0);
static MAX: AtomicU32 = AtomicU32::new(0);
/// the boss's index in [`NAMES`] plus one (0: no bar)
static WHO: AtomicU32 = AtomicU32::new(0);

/// The names the bar shows, by the boss host's name.
pub const NAMES: &[(&str, &str)] = &[
    ("porky", "PORKY"),
    ("ridley", "RIDLEY"),
    ("metaridley", "META RIDLEY"),
    ("galleom", "GALLEOM"),
    ("tabuu", "TABUU"),
    ("rayquaza", "RAYQUAZA"),
    ("duon", "DUON"),
    ("petey", "PETEY PIRANHA"),
];

/// Set the bar: the boss `name` (a [`NAMES`] key) with `hp` of `max` (both x 10). An unknown name or `max` 0 clears it.
pub fn set(name: &str, hp: u32, max: u32) {
    let who = NAMES.iter().position(|n| n.0 == name).map_or(0, |k| k as u32 + 1);
    HP.store(hp.min(max), Relaxed);
    MAX.store(max, Relaxed);
    WHO.store(if max == 0 { 0 } else { who }, Relaxed);
}

/// Hide the bar.
pub fn clear() {
    WHO.store(0, Relaxed);
}

/// (shown name, hp, max) while a bar is up.
pub fn get() -> Option<(&'static str, u32, u32)> {
    let w = WHO.load(Relaxed) as usize;
    let n = NAMES.get(w.checked_sub(1)?)?.1;
    Some((n, HP.load(Relaxed), MAX.load(Relaxed).max(1)))
}
