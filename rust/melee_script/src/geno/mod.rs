//! Geno: the fighter-extension engine's native half (pc/platform/geno_registry.c) and, later, the Lua domain and the LAB.
pub mod consts;
pub mod json;
pub mod registry;
pub mod tables;
pub use registry::{Env, NoEnv, Profile, Registry};
