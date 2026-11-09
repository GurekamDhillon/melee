
//! Port round 13 (night 3, round 5): hand ports of 29 item functions whose callees are all
//! ported in this crate (`it_8027C794`, `it_802DDBE8`, `it_802E37A4`, `itFlipper_Repel_inline`)
//! or call nothing. Verified against verbatim C copies (tests/c_oracle/round13_bodies.h,
//! extracted by gen_round13.py) in tests/oracle_round13.rs.
//!
//! Conventions as in rounds 3 / 10-12: a followed C pointer makes the function `unsafe`
//! (precondition: valid, as in C); globals are parameters: `Item_804A0C64` ->
//! `&mut HSD_ObjAllocUnk`, `gm_80473A18` -> `&mut UnkAllstarData`, `it_804D6D24` -> the article
//! table pointer, `it_804A0F60` -> the stage article table, `it_803F74F8` (itpeachparasol.c's
//! state table) -> a slice. Callbacks that C passes a GObj take the `Item` when they only read
//! `GET_ITEM(gobj)` (as itfwd8.rs / itkinds5.rs).
//!
//! No fused sites: every f32 expression here is a single operation (`vel.y += x8`,
//! `facing_dir * y`) or a compare.
#![allow(non_snake_case)]

use crate::itfwd8::it_8027c794;

use crate::itkinds10::it_flipper_repel_inline;

use crate::itkinds5::it_802e37a4;

use crate::itkinds6::it_802ddbe8;

use crate::layout::{
    Article, CollData, HSD_AnimJoint, HSD_GObj, HSD_ObjAllocUnk, Item, ItemStateTable, KinokoAnim,
    UnkAllstarData, Vec as GcVec, itClimbersBlizzardAttributes, itDoseiAttributes,
    itLikelikeAttributes, itLipstickAttributes, itSScopeAttributes, itSword_UnkArticle1,
    itSword_UnkBytes, It_Kind_Kinoko,
};

#[inline(always)]
pub(crate) unsafe fn sa<'a, T>(ip: &Item) -> &'a mut T {
    &mut *((*ip.xC4_article_data).x4_specialAttributes as *mut T)
}

// ---- item.c -----------------------------------------------------------------------------

/// `checkHitLag` (static): raise `xCBC_hitlagFrames` to `min_value`, set `xDC8 flags.xA`.
pub fn check_hit_lag(min_value: f32, ip: &mut Item) {
    if ip.xCBC_hitlagFrames < min_value {
        ip.xCBC_hitlagFrames = min_value;
    }
    ip.xDC8_word.flags.set_xA(1);
}

// ---- it_26B1.c --------------------------------------------------------------------------

/// `it_ResetStageArticleTable`: clear the 30 stage article slots (`it_804A0F60`).
pub fn it_reset_stage_article_table(tbl: &mut [*mut Article; 30]) {
    for e in tbl.iter_mut() {
        *e = core::ptr::null_mut();
    }
}

// ---- kinds ------------------------------------------------------------------------------

/// `itClimbersBlizzard_UnkMotion0_Phys`: `vel.y += attrs->x8`.
/// # Safety
/// The special attributes are an `itClimbersBlizzardAttributes`.
pub unsafe fn it_climbers_blizzard_unk_motion0_phys(ip: &mut Item) {
    let a = sa::<itClimbersBlizzardAttributes>(ip);
    ip.x40_vel.y += a.x8;
}

/// `itDosei_UnkMotion4_GetAttrs` (static inline): the special attributes.
/// # Safety
/// `ip.xC4_article_data` is valid.
pub unsafe fn it_dosei_unk_motion4_get_attrs(ip: &Item) -> *mut itDoseiAttributes {
    (*ip.xC4_article_data).x4_specialAttributes as *mut itDoseiAttributes
}

/// `itFlipper_Repel`: `itFlipper_Repel_inline` with a discarded local vector.
pub fn it_flipper_repel(ip: &mut Item, kind: i32, pos: &GcVec) {
    let mut v = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    it_flipper_repel_inline(ip, kind, pos, &mut v);
}

/// `itFreeze_ClearLinkedItem` (static inline): release the linked item, clear the link.
/// # Safety
/// A non-null `freeze.unk_1C` is a live item GObj.
pub unsafe fn it_freeze_clear_linked_item(ip: &mut Item) {
    let linked = ip.xDD4_itemVar.freeze.unk_1C;
    if !linked.is_null() {
        it_802e37a4(Some(&mut *((*linked).user_data as *mut Item)));
        ip.xDD4_itemVar.freeze.unk_1C = core::ptr::null_mut();
    }
}

/// `itFreeze_Logic17_Destroyed`: as `itFreeze_ClearLinkedItem`.
/// # Safety
/// A non-null `freeze.unk_1C` is a live item GObj.
pub unsafe fn it_freeze_logic17_destroyed(ip: &mut Item) {
    if !ip.xDD4_itemVar.freeze.unk_1C.is_null() {
        let l = ip.xDD4_itemVar.freeze.unk_1C;
        it_802e37a4(Some(&mut *((*l).user_data as *mut Item)));
        ip.xDD4_itemVar.freeze.unk_1C = core::ptr::null_mut();
    }
}

/// `itHeart_Logic8_Destroyed`: free the all-star heart slot `xDDC` if `xDD8.b0`.
/// # Safety
/// `xDDC` indexes inside `gm` (as in C, `x90[i]` is not bounds-checked).
pub unsafe fn it_heart_logic8_destroyed(ip: &mut Item, gm: &mut UnkAllstarData) {
    let h = &ip.xDD4_itemVar.heart;
    if h.xDD8.x0.b0() != 0 {
        *(gm.x90.as_mut_ptr()).offset(h.xDDC as isize) = 0;
    }
}

/// `itKyasarin_GetCollData` (static inline): `&ip->x378_itemColl`.
pub fn it_kyasarin_get_coll_data(ip: &mut Item) -> *mut CollData {
    &mut ip.x378_itemColl
}

/// `itKyasarin_UnkMotion9_Coll`: `it_8027C794`.
pub fn it_kyasarin_unk_motion9_coll(ip: &mut Item) -> bool {
    it_8027c794(ip)
}

/// `itLeadead_UnkMotion16_Coll`: `it_8027C794`.
pub fn it_leadead_unk_motion16_coll(ip: &mut Item) -> bool {
    it_8027c794(ip)
}

/// `itLikelike_UnkMotion21_Coll`: `it_8027C794`.
pub fn it_likelike_unk_motion21_coll(ip: &mut Item) -> bool {
    it_8027c794(ip)
}

/// `itOctarock_UnkMotion8_Coll`: `it_8027C794`.
pub fn it_octarock_unk_motion8_coll(ip: &mut Item) -> bool {
    it_8027c794(ip)
}

/// `itOldkuri_UnkMotion11_Coll`: `it_8027C794`.
pub fn it_oldkuri_unk_motion11_coll(ip: &mut Item) -> bool {
    it_8027c794(ip)
}

/// `itOldkuri_UnkMotion7_Coll`: `it_8027C794`.
pub fn it_oldkuri_unk_motion7_coll(ip: &mut Item) -> bool {
    it_8027c794(ip)
}

/// `itWhitebea_UnkMotion11_Coll`: `it_8027C794`.
pub fn it_whitebea_unk_motion11_coll(ip: &mut Item) -> bool {
    it_8027c794(ip)
}

/// `itSwapVelocity` (itlikelike.c, static inline): turn round, `vel.x = facing * attr->x0->y`.
/// # Safety
/// The special attributes are an `itLikelikeAttributes` whose `x0.x0_f32` is valid.
pub unsafe fn it_swap_velocity(ip: &mut Item) {
    let attr = sa::<itLikelikeAttributes>(ip);
    ip.facing_dir = -ip.facing_dir;
    ip.x40_vel.x = ip.facing_dir * (*attr.x0).x4;
}

/// `swapVelocity` (itlikelike.c, static inline): `itSwapVelocity(gobj->user_data)`.
/// # Safety
/// As `it_swap_velocity`.
pub unsafe fn swap_velocity(ip: &mut Item) {
    it_swap_velocity(ip);
}

/// `itMball_Destroyed`: one fewer live M-ball (`Item_804A0C64.x1C`) unless `mball.b1`.
pub fn it_mball_destroyed(ip: &Item, unk: &mut HSD_ObjAllocUnk) {
    if unsafe { ip.xDD4_itemVar.mball.b1() } == 0 {
        unk.x1C = unk.x1C.wrapping_sub(1);
    }
}

/// `itTomato_Logic9_Destroyed`: free the all-star tomato slot `x8` if `x4_b0`.
/// # Safety
/// `x8` indexes inside `gm` (as in C, `x90[i]` is not bounds-checked).
pub unsafe fn it_tomato_logic9_destroyed(ip: &mut Item, gm: &mut UnkAllstarData) {
    let t = &ip.xDD4_itemVar.tomato;
    if t.x4_b0() != 0 {
        *(gm.x90.as_mut_ptr()).offset(t.x8 as isize) = 0;
    }
}

/// `itZrshell_UnkMotion4_Anim` (itzgshell.c): `it_802DDBE8`, returns false.
/// # Safety
/// The special attributes are an `itGShell_Attrs`.
pub unsafe fn it_zrshell_unk_motion4_anim(ip: &mut Item) -> bool {
    it_802ddbe8(ip);
    false
}

pub use crate::tu::itsword::it_80285300_fp as it_80285300;
pub use crate::tu::itsscope::it_80291ccc;
pub use crate::tu::itsscope::it_80291cf4;
pub use crate::tu::itkinoko::it_80293660_fp as it_80293660;
pub use crate::tu::itlipstick::it_80295748_fp as it_80295748;
pub use crate::tu::ithammerhead::it_80299fb4_get;
pub use crate::tu::itpeachparasol::it_802bda40;
pub use crate::tu::itlikelike::it_802d9b78;
