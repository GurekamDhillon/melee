//! Clean-room decoder and script runner for Brawl boss parameter files (Petey Piranha's `BossPackun`), phase 1 of
//! the Petey port: no gameplay. Written from the reader's prose specification only.
//!
//! * [`archive`]: find the file in the stage archive (ARC, LZ77), pure functions over bytes.
//! * [`file`] / [`params`] / [`script`]: decode the file into typed data and typed command lists.
//! * [`runner`]: timelines, status scripts, the rule engine and the 16 built-ins, against the [`world::World`] trait.
//!
//! Everything that can reach the deterministic simulation is integer or fixed-point ([`fx::Fx`], 1/60000 units, the
//! file's own scalar unit). Randomness comes only from a caller-supplied [`rng::RandomSource`].
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

pub mod archive;
pub mod error;
pub mod file;
pub mod fx;
pub mod level;
pub mod params;
pub mod rng;
pub mod runner;
pub mod script;
pub mod world;

pub use error::{Error, Result};
pub use file::BossFile;
pub use fx::Fx;
pub use runner::{Config, FrameInput, Runner};
