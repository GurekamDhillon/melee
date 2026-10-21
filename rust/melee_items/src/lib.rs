//! Items (it), trophies (ty), vi.
//!
//! Port group 5 (docs/rust-port/inventory.md). Raw c2rust output for this group's TUs is
//! written to `src/raw/` by `tools/rust-port/transpile.sh`; it is reference input for the
//! hand port and is deliberately not declared as a module here.
#![allow(dead_code)]

pub use melee_core;

pub mod hash;
pub mod helpers;
pub mod item;
pub mod itlib;
pub mod itlib3;
pub mod itkinds;
pub mod itkinds5;
pub mod itfwd;
pub mod itfwd6;
pub mod itkinds6;
pub mod itfwd7;
pub mod itkinds7;
pub mod itfwd8;
pub mod itkinds8;
pub mod itkinds9;
pub mod itkinds10;
pub mod itkinds11;
/// The native types (with GC offset tables) are the single generated set in `melee_types`;
/// `layout` keeps group 5's path to them.
pub mod layout {
    pub use melee_types::*;
    /// Any C function pointer (untyped until the callee crates exist).
    pub type FnPtr = Option<unsafe extern "C" fn()>;
}
