//! The Geno registry (melee_script::geno, ported from pc/platform/geno_registry.c and its slices) and the script-facing
//! game interface (melee_script::game::ScriptGame, the C's gw_ScriptGame_* surface, implemented by
//! melee_engine::script_host::EngineGame for the wasm Lab). The C build's Lua scripting host is not part of the Rust
//! port: scripts and the LAB are wasm mods (melee_modhost), and a define's fighter code is the geno 12 `wasm` block.
pub mod game;
pub mod geno;

pub use game::{NoGame, RecRead, RecRow, RecStatus, ScriptGame};
