
//! Port round 11 (night 3, round 3): hand ports of 6 item functions whose only unported callees
//! are now native elsewhere in the workspace: group 0's jobj.h inline getters
//! (`melee_types::inline::HSD_JObjGetChild` / `GetRotation*` / `GetTranslation`, here as the
//! same field reads), group 1's MSL `sqrtf_accurate` (`melee_runtime::gekko`) and `HSD_Randi`
//! (`melee_runtime::random::HsdRand`). Verified against verbatim C copies
//! (tests/c_oracle/round11_bodies.h, extracted by gen_round11.py) in tests/oracle_round11.rs.
//!
//! Conventions as in round 3 / round 10: a followed C pointer makes the function `unsafe`
//! (precondition: valid, as in C); globals are parameters: `it_804D6D28` -> `&ItemCommonData`,
//! `Item_804A0CCC` -> `&Item_FtTrack`, `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]` -> the list head
//! (TODO(group1), as itlib3.rs), the RNG -> `&mut HsdRand`.
//!
//! Fused sites (shipping-build contraction): `calc_dist_2d_accurate` 1 f32 (`SQ(x) + SQ(y)` ->
//! `fmadds(x, x, y*y)`), plus `sqrtf_accurate`'s f64 Newton steps (as `gekko::sqrtf_accurate`).
//! No other function here has arithmetic that contracts.
#![allow(non_snake_case)]

use crate::helpers::it_coll_chk_ecb_overlap;

use crate::itlib::it_is_heavy;

use crate::layout::{
    HSD_GObj, HSD_JObj, Item, ItemCommonData, Item_FtTrack, Quaternion, Vec as GcVec, GA_Ground,
};

use melee_core::fp::fmadds;

use melee_runtime::gekko::sqrtf_accurate;

use melee_runtime::random::HsdRand;

#[inline(always)]
pub(crate) unsafe fn item<'a>(gobj: *mut HSD_GObj) -> &'a mut Item {
    &mut *((*gobj).user_data as *mut Item)
}

#[inline(always)]
pub(crate) unsafe fn jobj_of(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

/// jobj.h `HSD_JObjGetChild` (NULL-safe).
#[inline(always)]
pub(crate) unsafe fn child(j: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_types::inline::HSD_JObjGetChild(j.as_ref())
}

#[inline(always)]
pub(crate) unsafe fn gobj_item_ptr(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

// ---- kinds -----------------------------------------------------------------------------

/// `itLinkArrow_802A850C_inline_2` (itlinkarrow.c, static inline): read the JObj rotation into
/// `quat`; the 4 trail points (x24..x48) become `x18` with z - 0.02, the 4 trail rotations
/// (x54..x84) `quat`.
/// # Safety
/// `gobj` is a live item GObj with a JObj.
pub unsafe fn it_link_arrow_802a850c_inline_2(gobj: *mut HSD_GObj, quat: &mut Quaternion) {
    let ip = item(gobj);
    let jobj = &*jobj_of(gobj);
    *quat = jobj.rotate;
    let la = &mut ip.xDD4_itemVar.linkarrow;
    let pts: [*mut GcVec; 4] = [&mut la.x24, &mut la.x30, &mut la.x3C, &mut la.x48];
    for p in pts {
        *p = la.x18;
        (*p).z -= 0.02f32;
    }
    la.x54 = *quat;
    la.x64 = *quat;
    la.x74 = *quat;
    la.x84 = *quat;
}

/// `calc_dist_2d_accurate` (itluigifireball.c, static): MSL `sqrtf_accurate(x*x + y*y)` as f64.
pub fn calc_dist_2d_accurate(v: &GcVec) -> f64 {
    sqrtf_accurate(v.x * v.x + v.y * v.y) as f64 // VEC2_SQ_LEN(*v): not fused on the console
}

pub use crate::tu::itcoll::it_80271b60;
pub use crate::tu::itcoll::it_80271f78;
pub use crate::tu::it_2725::it_80272cc0;
pub use crate::tu::it_2725::it_80274990_fp as it_80274990;
