//! Every game struct, enum and constant as native little-endian Rust types, each with its GameCube layout (`melee_core::gc::GcLayout`).
//!
//! Port group 0 (docs/rust-port/inventory.md). Raw c2rust output for this group's TUs is
//! written to `src/raw/` by `tools/rust-port/transpile.sh`; it is reference input for the
//! hand port and is deliberately not declared as a module here.
#![allow(dead_code)]

pub use melee_core;
