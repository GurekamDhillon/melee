//! `src/MSL/float.c` contains data only (zero function definitions).
//! Keep the original NaN payload used by lbtrigf, rather than Rust's canonical NaN.
#![allow(non_upper_case_globals)]
pub const MSL_TrigF_80400770: [i32; 1] = [0x7fff_ffff];
pub const MSL_TrigF_80400774: [i32; 1] = [0x7f80_0000];
