//! itcoll.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::itacc::{ip};

// ---- folded from itacc.rs (consolidation steps 21/22) ----
/// `it_802714C0_fp`: let the hurtboxes follow the item again.
pub unsafe fn it_802714C0_fp(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802714C0_fp");
    let i = ip(gobj);
    for k in 0..i.xAC8_hurtboxNum as usize {
        i.xACC_itemHurtbox[k].set_skip_update_pos(0);
    }
    i.anon_daa.xDAA_flag.anon_0.set_b1(1);
}
