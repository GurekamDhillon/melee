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

type I = *mut Item_GObj;

#[inline]
fn ext(slot: i32) -> &'static mut pl_StaleMoveTableExt_t {
    unsafe { &mut *crate::player::get_stale_move_table_index_ptr2(slot) }
}

pub use crate::tu::plbonuslib::pl_8003DF44 as pl_8003df44;

pub use crate::tu::plbonuslib::pl_8003DFF4 as pl_8003dff4;

pub use crate::tu::plbonuslib::pl_8003E058 as pl_8003e058;

pub use crate::tu::plbonuslib::pl_8003E0E8 as pl_8003e0e8;

pub use crate::tu::plbonuslib::pl_8003E114 as pl_8003e114;

pub use crate::tu::plbonuslib::pl_8003E150 as pl_8003e150;

pub use crate::tu::plbonuslib::pl_8003E17C as pl_8003e17c;

pub use crate::tu::plbonuslib::pl_8003E2CC as pl_8003e2cc;

pub use crate::tu::plbonuslib::pl_8003E334 as pl_8003e334;

pub use crate::tu::plbonuslib::pl_8003E39C as pl_8003e39c;

pub use crate::tu::plbonuslib::pl_8003E420 as pl_8003e420;

pub use crate::tu::plbonuslib::match_item_kind;

/// `pl_8003E4A4`. C quirk not reproduced: an unknown kind gives -1, which passes the C's signed
/// `< Pl_ItemLog_Terminate` test and stores to `seen[-1]` (out of bounds on the stack); here it
/// is skipped. The oracle only drives kinds that map.
pub fn pl_8003e4a4(slot: i32, _arg1: bool, moves: &[i32]) {
    let t = ext(slot);
    let mut seen = [0u32; Pl_ItemLog_Terminate as usize];
    for &m in moves {
        let k = match_item_kind(m);
        if k >= 0 && k < Pl_ItemLog_Terminate as i32 {
            seen[k as usize] = 1;
        }
    }
    let lim = h::pl_804d6470().x138;
    for i in 0..Pl_ItemLog_Terminate as usize {
        let s = &mut t.x0_staleMoveTable;
        if seen[i] == 1 {
            s.x7AC[i] = s.x7AC[i].wrapping_add(1);
            if s.x7AC[i] as u32 == lim {
                h::pl_80038788(slot, 0x9D, 1);
                s.x7AC[i] = 0;
            }
        } else {
            s.x7AC[i] = 0;
        }
    }
}

pub use crate::tu::plbonuslib::pl_8003E854 as pl_8003e854;

pub use crate::tu::plbonuslib::pl_8003E978 as pl_8003e978;

pub use crate::tu::plbonuslib::fn_8003E998 as fn_8003e998;

pub use crate::tu::plbonuslib::pl_8003EA08 as pl_8003ea08;

pub use crate::tu::plbonuslib::pl_8003EA40 as pl_8003ea40;

pub use crate::tu::plbonuslib::pl_8003EA74 as pl_8003ea74;

pub use crate::tu::plbonuslib::pl_8003EAAC as pl_8003eaac;

pub use crate::tu::plbonuslib::pl_8003EB30 as pl_8003eb30;

pub use crate::tu::plbonuslib::pl_8003EC30 as pl_8003ec30;

pub use crate::tu::plbonuslib::pl_8003EC9C as pl_8003ec9c;

pub use crate::tu::plbonuslib::pl_8003ED0C as pl_8003ed0c;
