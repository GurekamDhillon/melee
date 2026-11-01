
//! Round 5: lbcommand.c script commands over nested pointers that round 3 skipped, verified
//! bit for bit against the C by `tests/oracle_r5.rs`. TODO(group1): lb leftovers; move into
//! melee_runtime if group 1 takes lbcommand.c.
use melee_types as mt;

use mt::{CmdUnion, CommandInfo};

pub use crate::tu::lbcommand::Command_04_fp as Command_04;
pub use crate::tu::lbcommand::Command_05_fp as Command_05;
pub use crate::tu::lbcommand::Command_07_fp as Command_07;
