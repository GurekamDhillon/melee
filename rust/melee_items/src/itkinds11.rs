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
unsafe fn item<'a>(gobj: *mut HSD_GObj) -> &'a mut Item {
    &mut *((*gobj).user_data as *mut Item)
}
#[inline(always)]
unsafe fn jobj_of(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}
/// jobj.h `HSD_JObjGetChild` (NULL-safe).
#[inline(always)]
unsafe fn child(j: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_types::inline::HSD_JObjGetChild(j.as_ref())
}

// ---- it_2725.c -------------------------------------------------------------------------

/// `it_80272CC0`: bone `idx` of the item model: from the dynamic bone table when the model
/// describes bones, else the `idx`-th first child down from the root JObj.
/// # Safety
/// `gobj` is a live item GObj; its article data / model desc (and the bone table when used) valid.
pub unsafe fn it_80272cc0(gobj: *mut HSD_GObj, mut idx: i32) -> *mut HSD_JObj {
    let ip = item(gobj);
    let mut jobj = jobj_of(gobj);
    if (*(*ip.xC4_article_data).x10_modelDesc).x4_bone_count != 0 {
        return (*ip.xBBC_dynamicBoneTable).bones[idx as usize];
    }
    if idx != 0 {
        while idx > 0 {
            jobj = child(jobj);
            idx -= 1;
        }
    }
    jobj
}

/// `it_80274990`: the model's spin angle: descend `(xC_bit_field >> 6) & 3` children, then read
/// rotation z / x / y by `xDC8_word.flags.x17` (0 / 1 / other).
/// # Safety
/// As `it_80272CC0`; the JObj reached must be non-null (the C asserts).
pub unsafe fn it_80274990(gobj: *mut HSD_GObj) -> f32 {
    let ip = item(gobj);
    let mut j = jobj_of(gobj);
    let bit_chk = ((*(*ip.xC4_article_data).x10_modelDesc).xC_bit_field as u32 >> 6) & 3;
    let mut n = bit_chk as i32;
    while n > 0 {
        j = if j.is_null() { core::ptr::null_mut() } else { (*j).child };
        n -= 1;
    }
    let j = &*j;
    match ip.xDC8_word.flags.x17() {
        0 => j.rotate.z,
        1 => j.rotate.x,
        _ => j.rotate.y,
    }
}

// ---- itcoll.c --------------------------------------------------------------------------

/// `it_80271B60`: push the item sideways off any tracked fighter ECB it overlaps
/// (`xDC0 |= 1`); exactly level picks a random side.
/// # Safety
/// `gobj` is a live item GObj with a JObj.
pub unsafe fn it_80271b60(
    rand: &mut HsdRand,
    gobj: *mut HSD_GObj,
    track: &Item_FtTrack,
    common: &ItemCommonData,
) {
    let jobj = jobj_of(gobj);
    let ip = item(gobj);
    if track.x154.anon_0.b0() != 1 {
        let sp24: GcVec = (*jobj).translate;
        let mut cnt: u32 = 0;
        while (cnt as usize) < track.count {
            let ecb = &track.ecb_offset_arr[cnt as usize];
            let y_pos = track.ft_pos_arr[cnt as usize].y;
            let x_pos = track.ft_pos_arr[cnt as usize].x;
            if it_coll_chk_ecb_overlap(x_pos, y_pos, &ip.xBEC, ecb, &sp24) {
                let d = sp24.x - x_pos;
                let dir = if (if d < 0.0 { -d } else { d }) < 0.001f32 {
                    if rand.randi(2) != 0 { 1.0f32 } else { -1.0f32 }
                } else if sp24.x - x_pos < 0.0 {
                    -1.0f32
                } else {
                    1.0f32
                };
                ip.x70_nudge.x = common.x7C_float * dir;
                ip.xDC0 |= 1;
            }
            cnt += 1;
        }
    }
}

/// `it_80271F78`: push the item off grounded heavy items it overlaps (`xDC0 |= 2`).
/// # Safety
/// `gobj` is a live item GObj with a JObj; `head` starts a valid `next`-linked list of item
/// GObjs (user data Items with valid attrs, JObjs).
pub unsafe fn it_80271f78(
    rand: &mut HsdRand,
    gobj: *mut HSD_GObj,
    head: *mut HSD_GObj,
    common: &ItemCommonData,
) {
    let jobj = jobj_of(gobj);
    let arg_item = gobj_item_ptr(gobj);
    let sp34: GcVec = (*jobj).translate;
    let mut item_gobj = head;
    while !item_gobj.is_null() {
        let item_jobj = jobj_of(item_gobj);
        let it = gobj_item_ptr(item_gobj);
        if gobj != item_gobj
            && (*it).xDC8_word.flags.x13() == 0
            && (*it).ground_or_air == GA_Ground
            && (*it).xDD1_flag.anon_0.b0() == 0
            && it_is_heavy(&*it)
        {
            let sp28: GcVec = (*item_jobj).translate;
            if it_coll_chk_ecb_overlap(sp28.x, sp28.y, &(*arg_item).xBEC, &(*it).xBEC, &sp34) {
                let d = sp34.x - sp28.x;
                let dir = if (if d < 0.0 { -d } else { d }) < 0.001f32 {
                    if (*it).xDC8_word.flags.x1B() == 0 {
                        if (*it).x70_nudge.x < 0.0 { 1.0f32 } else { -1.0f32 }
                    } else if rand.randi(2) != 0 {
                        1.0f32
                    } else {
                        -1.0f32
                    }
                } else if sp34.x - sp28.x < 0.0 {
                    -1.0f32
                } else {
                    1.0f32
                };
                (*arg_item).x70_nudge.x = common.x7C_float * dir;
                (*arg_item).xDC0 |= 2;
            }
        }
        item_gobj = (*item_gobj).next;
    }
}

#[inline(always)]
unsafe fn gobj_item_ptr(gobj: *mut HSD_GObj) -> *mut Item {
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
    sqrtf_accurate(fmadds(v.x, v.x, v.y * v.y)) as f64
}
