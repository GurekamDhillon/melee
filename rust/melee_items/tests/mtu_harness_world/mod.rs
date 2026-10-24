//! The shared-pool-world variant of the mtu harness: the base harness plus world.rs (needs the
//! generated `g::fill_*` / `g::hash_*` / `g::pick_*` of a gen written for the shared world).
#![allow(dead_code, static_mut_refs)]
#[path = "../mtu_harness/mod.rs"]
mod base;
pub use base::*;
#[path = "../mtu_harness/world.rs"]
pub mod world;
pub use world::*;
