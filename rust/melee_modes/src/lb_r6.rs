
//! Round 6 (night 3 round 2): lb leaf functions (lbrefract.c, lb_00B0.c), verified bit for bit
//! against the C by `tests/oracle_r6.rs`. TODO(group1): lb leftovers; move into melee_runtime
//! if group 1 takes these TUs.
use melee_core::fp::fmadds;

use melee_types::HSD_JObj;

pub use crate::tu::lb_00b0::lb_8000B074_fp as lb_8000B074;
pub use crate::tu::lb_00b0::memzero_fp as memzero;
pub use crate::tu::lbrefract::my_fmodf_fp as my_fmodf;
