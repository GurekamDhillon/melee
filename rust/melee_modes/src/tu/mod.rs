//! One file per game translation unit (port rounds "shard"): src/melee/gm/<stem>.c -> tu/<stem>.rs.
//! Callees of unported TUs are `ext` stubs in each file; every file's functions are checked by the
//! state oracle (`crate::oracle`, `gen/tu/run.py`) against the C compiled natively.
pub mod gmprogressivemode;
pub mod gmregcommon;
pub mod gmregtyfall;

pub mod gmregclear;
pub mod gmregenddisp;
pub mod gmresult;
pub mod hsd;
