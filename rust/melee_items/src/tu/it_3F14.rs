//! Port of src/melee/it/it_3F14.c. The file holds the common item logic table (`it_803F14C4`, with the GX
//! link table `it_803F1418`) and three globals: `it_804D6D40` (the per-item tuning record), `it_804D6D38`
//! and `it_804D6D30` (article pointer tables). This module ports the globals. The logic tables are data whose
//! entries are function addresses owned by other TUs; they are not ported here and are pending the
//! function-address table generator (see the crate's notes).
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use core::ptr::null_mut;
use melee_types::*;

/// `it_804D6D40` (the `_t` record is `it_804D6D40_t` in melee_types). Set by the crate's init.
pub static mut it_804D6D40: *mut it_804D6D40_t = null_mut();
/// `it_804D6D38`: article pointer table (`Article**`), set by the crate's init.
pub static mut it_804D6D38: *mut *mut Article = null_mut();
/// `it_804D6D04`, `it_804D6D20`, `it_804D6D24`, `it_804D6D28`: the item common-data globals iteffect.c fills from
/// `itPublicData` (`it_8027870C`). One static each: every item TU's `ext` block aliases these.
pub static mut it_804D6D04: *mut Fighter_804D653C_t = null_mut();
pub static mut it_804D6D20: *mut it_804D6D20_t = null_mut();
pub static mut it_804D6D24: *mut *mut Article = null_mut();
pub static mut it_804D6D28: *mut ItemCommonData = null_mut();
/// `it_804D6D30`: article pointer table (`Article**`), set by the crate's init.
pub static mut it_804D6D30: *mut *mut Article = null_mut();
