//! Fighter engine (ft/*.c), shared fighter states (ftCommon), players (pl).
//!
//! Port group 3 (docs/rust-port/inventory.md). Raw c2rust output for this group's TUs is
//! written to `src/raw/` by `tools/rust-port/transpile.sh`; it is reference input for the
//! hand port and is deliberately not declared as a module here.
#![allow(dead_code)]

pub use melee_core;

pub mod types;
pub mod compat;

/// Mechanical c2rust translation of every group-3 TU (feature `c2r`; generated, gitignored).
#[cfg(feature = "c2r")]
pub mod c2r;
pub mod neutral_hash;
pub mod hooks;
pub mod ftcommon;
pub mod ftcommon2;
pub mod stale;
pub mod player;
pub mod ftlib;
pub mod plbonus;
pub mod plbonus2;
pub mod plbonus3;
pub mod ft0877;
pub mod ft0881;
pub mod ft0892;
pub mod pltrick;
pub mod ftcommon3;
pub mod ftaction;
pub mod fighter;
pub mod ftanim;
pub mod ftcoll;
pub mod ft084e;
