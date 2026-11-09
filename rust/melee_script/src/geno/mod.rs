//! Geno: the fighter-extension engine's native half (pc/platform/geno_registry.c) and, later, the Lua domain and the LAB.
pub mod consts;
pub mod json;
pub mod registry;
pub mod tables;
pub mod define;
pub mod lua;
pub mod combo;
pub mod plan;
pub mod controls;

/// The newest geno.json format this build reads: the C's GENO_VERSION (11, consts.rs is generated from pc/geno/geno.h)
/// plus the Rust-only geno 12 (a define's `wasm` fighter code; the C build is frozen at 11).
pub const GENO_FORMAT: u32 = 12;
pub use registry::{Env, NoEnv, Profile, Registry};
