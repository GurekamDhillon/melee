
//! Round 7 (night 3 round 3): textlib.c caller of round 6's `memzero`, verified bit for bit
//! against the C by `tests/oracle_r7.rs`.
use crate::lb_r6::memzero;

use melee_types::DevText;

pub use crate::tu::textlib::DevText_Erase_fp as DevText_Erase;
