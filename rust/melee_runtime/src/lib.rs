//! Runtime, math and HSD: MSL math_ppc (bit-exact `__frsqrte` sqrtf), gekko_fp, mtx, sysdolphin baselib, lb, db, sfx.
//!
//! Port group 1 (docs/rust-port/inventory.md). Raw c2rust output for this group's TUs is
//! written to `src/raw/` by `tools/rust-port/transpile.sh`; it is reference input for the
//! hand port and is deliberately not declared as a module here.
#![allow(dead_code)]

pub use melee_core;
