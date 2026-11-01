
//! Group 3 round 10: the middle of src/melee/pl/plbonuslib.c, `pl_8003DF44` .. `pl_8003ED0C`
//! (24 C definitions incl. the static inline `match_item_kind`), field logic over
//! `pl_StaleMoveTableExt_t`. Calls out go through `crate::hooks` (plbonus.c, items, ftLib).
//! One fused site: `pl_8003ED0C`'s `xCA0 * (n - 1) + arg2` (fmadds, fma_sites.py).
//! Checked against the C (cut verbatim) by tests/plbonus_ref.rs (gen/cref/plbonus_ref.c).
//!
//! Not ported here: pl_8003E70C / pl_8003E7D4 (index the 8-entry `x598` with item kinds
//! 161..190, i.e. past the array into neighbouring fields), the fn_8003EE2C.. tail of the range.
#![allow(non_upper_case_globals)]

use crate::hooks as h;

use crate::plbonus::{check_if_same_team, calculate_average, pl_8003fc88, pl_8003ff44, pl_800419ac, pl_80040ed4};

use crate::player::get_entity_at_index;

use crate::types::*;

use melee_core::fp::fmadds;

pub(crate) type I = *mut Item_GObj;

#[inline]
pub(crate) fn ext(slot: i32) -> &'static mut pl_StaleMoveTableExt_t {
    unsafe { &mut *crate::player::get_stale_move_table_index_ptr2(slot) }
}

pub use crate::tu::plbonuslib::fn_8003e998;
pub use crate::tu::plbonuslib::match_item_kind_fp as match_item_kind;
pub use crate::tu::plbonuslib::pl_8003df44;
pub use crate::tu::plbonuslib::pl_8003dff4;
pub use crate::tu::plbonuslib::pl_8003e058;
pub use crate::tu::plbonuslib::pl_8003e0e8;
pub use crate::tu::plbonuslib::pl_8003e114;
pub use crate::tu::plbonuslib::pl_8003e150;
pub use crate::tu::plbonuslib::pl_8003e17c;
pub use crate::tu::plbonuslib::pl_8003e2cc;
pub use crate::tu::plbonuslib::pl_8003e334;
pub use crate::tu::plbonuslib::pl_8003e39c;
pub use crate::tu::plbonuslib::pl_8003e420;
pub use crate::tu::plbonuslib::pl_8003e4a4;
pub use crate::tu::plbonuslib::pl_8003e854_fp as pl_8003e854;
pub use crate::tu::plbonuslib::pl_8003e978;
pub use crate::tu::plbonuslib::pl_8003ea08;
pub use crate::tu::plbonuslib::pl_8003ea40;
pub use crate::tu::plbonuslib::pl_8003ea74;
pub use crate::tu::plbonuslib::pl_8003eaac;
pub use crate::tu::plbonuslib::pl_8003eb30;
pub use crate::tu::plbonuslib::pl_8003ec30;
pub use crate::tu::plbonuslib::pl_8003ec9c;
pub use crate::tu::plbonuslib::pl_8003ed0c;
