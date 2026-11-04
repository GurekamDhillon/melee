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
pub use melee_runtime::shared::it_804D6D38;
/// `it_804D6D04`, `it_804D6D20`, `it_804D6D24`, `it_804D6D28`: the item common-data globals iteffect.c fills from
/// `itPublicData` (`it_8027870C`). One static each: every item TU's `ext` block aliases these.
/// `it_804D6D00` (it_3F14.c): the Warp Star's last picked animation, reset to -1 by the item init (item.c), read and
/// written by `it_80294364` (itwstar.c). One definition: item.rs and itwstar.rs re-export it (each had its own copy, so
/// the reset never reached the Warp Star and its first pick excluded animation 0).
pub static mut it_804D6D00: i8 = 0;
pub static mut it_804D6D04: *mut Fighter_804D653C_t = null_mut();
pub static mut it_804D6D20: *mut it_804D6D20_t = null_mut();
pub static mut it_804D6D24: *mut *mut Article = null_mut();
pub use melee_runtime::shared::it_804D6D28;
/// `it_804D6D30`: article pointer table (`Article**`), set by the crate's init.
pub static mut it_804D6D30: *mut *mut Article = null_mut();

// The item logic and render-link tables of it_3F14.c, it_279C.c and it_3F2F.c. Their rows are function and table
// addresses owned by other TUs; `melee_link::retail_it::install_retail_it` (gen_retail_it.py) fills them at boot.
pub static mut it_803F1418: [sdata_ItemGXLink; 43] = unsafe { core::mem::zeroed() };
pub static mut it_803F14C4: [ItemLogicTable; 43] = unsafe { core::mem::zeroed() };
pub static mut it_803F2310: [sdata_ItemGXLink; 47] = unsafe { core::mem::zeroed() };
pub static mut it_803F23CC: [ItemLogicTable; 47] = unsafe { core::mem::zeroed() };
pub static mut it_803F2F28: [sdata_ItemGXLink; 118] = unsafe { core::mem::zeroed() };
pub static mut it_803F3100: [ItemLogicTable; 118] = unsafe { core::mem::zeroed() };
pub static mut it_803F4CA8: [sdata_ItemGXLink; 30] = unsafe { core::mem::zeroed() };
pub static mut it_803F4D20: [ItemLogicTable; 30] = unsafe { core::mem::zeroed() };
