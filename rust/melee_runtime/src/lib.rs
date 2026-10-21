//! Runtime, math and HSD: MSL math_ppc (bit-exact `__frsqrte` sqrtf), gekko_fp, mtx, sysdolphin baselib, lb, db, sfx.
//!
//! Port group 1 (docs/rust-port/inventory.md). Raw c2rust output for this group's TUs is
//! written to `src/raw/` by `tools/rust-port/transpile.sh`; it is reference input for the
//! hand port and is deliberately not declared as a module here.
//!
//! Float rule (applies to every crate): the C build compiles with clang's default
//! `-ffp-contract=on`, and gwtool lowers each contracted f32 `a*b+c` to
//! `melee_core::fp::fmadds` and each f64 one to a true fma. A port must fuse exactly where clang
//! fused, no more and no less. `oracle/build.sh` regenerates the C reference vectors that
//! `tests/oracle.rs` checks bit for bit.
#![allow(dead_code)]

pub use melee_core;

pub mod aobj;
pub mod cabi;
pub mod dat;
pub mod fobj;
pub mod gekko;
pub mod gobj;
pub mod hsd_mtx;
pub mod hsd_util;
pub mod id;
pub mod lbtrigf;
pub mod lbvector;
pub mod list;
pub mod mtx;
pub mod quatlib;
pub mod random;
pub mod spline;
