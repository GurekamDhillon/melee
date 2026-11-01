//! ithitbox.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::itacc::{ip, it_802714C0};

// ---- folded from itacc.rs (consolidation steps 21/22) ----
/// `it_80275414_fp`.
pub unsafe fn it_80275414_fp(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80275414_fp");
    ip(gobj).xDCD_flag.anon_0.set_b7(1);
}

/// `it_802756D0_fp`.
pub unsafe fn it_802756D0_fp(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802756D0_fp");
    ip(gobj).xD0C = 2;
}

/// `it_802756E0_fp`.
pub unsafe fn it_802756E0_fp(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802756E0_fp");
    ip(gobj).xD0C = 0;
    it_802714C0(gobj);
}
