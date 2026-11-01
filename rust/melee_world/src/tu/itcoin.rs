//! itcoin.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::itacc::{ip};

// ---- folded from itacc.rs (consolidation steps 21/22) ----
/// `it_802F2014_fp` (coin): the only joint of the item collision.
pub unsafe fn it_802F2014_fp(gobj: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("it_802F2014_fp");
    ip(gobj).x378_itemColl.joint_id_only = arg1;
}

/// `it_802F2020_fp` (coin).
pub unsafe fn it_802F2020_fp(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("it_802F2020_fp");
    ip(gobj).xDD4_itemVar.coin.x0
}
