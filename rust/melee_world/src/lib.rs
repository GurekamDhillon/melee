//! World: map (mp), camera (cm), effects (ef), stages (gr), HUD (if).
//!
//! Port group 2 (docs/rust-port/inventory.md). Raw c2rust output for this group's TUs is
//! written to `src/raw/` by `tools/rust-port/transpile.sh`; it is reference input for the
//! hand port and is deliberately not declared as a module here.
//! Status: `docs/rust-port/group-2-world.md`.
#![allow(dead_code)]

pub use melee_core;

pub mod cm;
pub mod ef;
pub mod gr;
pub mod hud;
pub mod gekko;
pub mod mp;
pub mod pl;
pub mod ftlib;
pub mod gm;
pub mod vec;
pub mod tu;
/// Generated hooks for callees outside the per-TU ports (gen/turef/tu_shard585.py).
pub mod tu_ext;
