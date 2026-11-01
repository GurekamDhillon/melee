//! ittincle.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::itacc::{ip};

// ---- folded from itacc.rs (consolidation steps 21/22) ----
/// `it_802ECC8C_fp` (Tincle).
pub unsafe fn it_802ECC8C_fp(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("it_802ECC8C_fp");
    ip(gobj).xDD4_itemVar.tincle.x20
}

/// `it_802ECC98_fp` (Tincle).
pub unsafe fn it_802ECC98_fp(gobj: *mut HSD_GObj, arg1: f32) {
    melee_core::ct!("it_802ECC98_fp");
    ip(gobj).xDD4_itemVar.tincle.x54 = arg1;
}

/// `it_802ECCA4_fp` (Tincle): its position lifted by `x50`, and the state id.
pub unsafe fn it_802ECCA4_fp(gobj: *mut HSD_GObj, kind: *mut i32, pos: *mut Vec) {
    melee_core::ct!("it_802ECCA4_fp");
    let i = ip(gobj);
    *pos = i.pos;
    (*pos).y += i.xDD4_itemVar.tincle.x50;
    *kind = i.msid;
}
