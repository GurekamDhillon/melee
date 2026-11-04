//! Shared foundation for the native Rust port: Gekko float helpers, GameCube layout metadata
//! and the byte-order-neutral state hash. No game logic lives here.
//!
//! Determinism rules for every crate (RUST_PORT.md): keep the C build's f32 operation order,
//! never use `f32::mul_add` for a C expression, no fast-math; integer arithmetic wraps like
//! the C build's `-fwrapv` (use `wrapping_*` where it matters; overflow checks are off).
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod codes;
pub mod ct;
pub mod fp;
pub mod gc;
pub mod gcdecode;
pub mod hash;
pub mod modhook;
pub mod vec;

/// Crate version, used by the `melee` binary to prove each crate links.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
