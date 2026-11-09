//! Geno: the fighter-extension engine's native half (pc/platform/geno_registry.c) and, later, the Lua domain and the LAB.
pub mod consts;
pub mod json;
pub mod registry;
pub mod tables;
pub mod define;
pub mod code;
pub mod combo;
pub mod plan;
pub mod controls;

/// The newest geno.json format this build reads: the C's GENO_VERSION (11, consts.rs is generated from pc/geno/geno.h)
/// plus the Rust-only geno 12 (a define's `wasm` fighter code; the C build is frozen at 11).
pub const GENO_FORMAT: u32 = 12;

/// `GS_GENO_MOTION_BASE`: Geno action states are motion 0x400 + n.
pub const GENO_MOTION_BASE: i32 = 0x400;
pub use registry::{Env, NoEnv, Profile, Registry};
