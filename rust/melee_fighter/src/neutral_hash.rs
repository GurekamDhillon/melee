//! The byte-order-neutral gameplay hash, format NHV1 (docs/rust-port/neutral-hash.md).
//!
//! There is one implementation, `melee_types::neutral_hash`; this module keeps group 3's names
//! for it so `tests/nh_ref.rs` (the C `nh_Fighter` compiled natively) keeps checking that one copy.

pub use melee_types::neutral_hash::*;
use melee_types::{CpuFighter, Fighter};

/// "NHV1": format version, the first word mixed into the game hash.
pub const NHV1: u32 = 0x4E48_5631;
pub const FNV_START: u32 = 0x811C_9DC5;

/// `nh_Mix`.
#[inline]
pub const fn mix(h: u32, v: u32) -> u32 {
    nh_mix(h, v)
}

/// `nh_CpuFlags`.
pub fn cpu_flags(c: &CpuFighter) -> u32 {
    nh_cpu_flags(c)
}

/// `nh_Fighter`: one fighter's word (`s0`..`s11` columns).
pub fn fighter_word(fp: &Fighter, slot: u32, stocks: i32, cpu: bool) -> u32 {
    nh_fighter(fp, slot as i32, stocks, cpu)
}
