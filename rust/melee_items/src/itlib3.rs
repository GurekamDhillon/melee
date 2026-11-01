
//! Item library, port round 3 (night 3): the next layer of it_2725.c, it_26B1.c, it_279C.c,
//! itanimlist.c, itcoll.c, item.c, ithitbox.c, itmaplib.c, itspawn.c and itzako.c. Every
//! function here either calls nothing or calls only functions already ported in `itlib.rs` /
//! `helpers.rs`. Each cites the C it mirrors; verified against verbatim C copies in
//! tests/c_oracle/round3.c (tests/oracle_round3.rs).
//!
//! Conventions as in `itlib.rs`: an `Item_GObj*` used only for its user data becomes the
//! `Item`; a followed C pointer makes the function `unsafe` (precondition: valid). Globals
//! become parameters: `it_804D6D28` -> `&ItemCommonData`, `it_804D6D40` -> `&it_804D6D40_t`,
//! `Item_804A0E24` -> `&PokemonSelectionState`, `it_804D6D14` -> `&mut u32`, and
//! `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]` -> the list head pointer (TODO(group1): take it
//! from the native GObj runtime once that exists).
//!
//! Fused multiply-adds (`tools/rust-port/fma_sites.py`, shipping-build contraction):
//! `itColl_BounceOffVictim` 1, `it_80270CD8` 6 (3 per branch), `it_8026CA4C` 1 (f32, so
//! `fmadds`); every other function here has none.
#![allow(non_upper_case_globals)]

use crate::helpers::COLLIDE_CEILING_MASK;

use crate::itlib::{
    it_8026b3a8, it_8026b724, it_8026b73c, it_8026bb88, it_8026bbcc, it_802714c0, it_8027346c,
    it_80273484, it_8027349c, it_802734b4, it_80275228, it_80275240, it_80275594,
    COLLIDE_FLOOR_MASK,
};

use crate::layout::{
    it_2728_DatAttrs, it_804D6D40_t, itPokemonSpawn_DatAttrs, CommandInfo, Collide_LeftWallMask,
    Collide_RightWallMask, Fighter, HSD_GObj, HitCapsule, Item, ItemCommonData, ItemKind,
    ItemLink, ItemPickTable, ItemStateDesc, PokemonSelectionState, Vec as GcVec,
    lb_80014638_arg1_t, HSD_GOBJ_CLASS_ITEM, It_Kind_L_Gun_Ray, It_PKind_Marumine,
    It_PKind_Start, It_PKind_Terminate,
};

use melee_core::fp::fmadds;

use std::ptr::null_mut;

/// `itColl_BounceOffVictim`. 1 fused site: `vel.y = fmadds(vel.y, x5C, x60)`.
pub fn it_coll_bounce_off_victim(ip: &mut Item, common: &ItemCommonData) {
    ip.x40_vel.x *= common.x58_float;
    ip.x40_vel.y = fmadds(ip.x40_vel.y, common.x5C_float, common.x60_float);
}

/// `Item_IsGrabbable`: grabbable flag set and a `picked_up` callback present.
/// # Safety
/// `ip.xB8_itemLogicTable` must be valid when `xDC8_word.flags.x15` is set.
pub unsafe fn item_is_grabbable(ip: &Item) -> bool {
    ip.xDC8_word.flags.x15() != 0 && (*ip.xB8_itemLogicTable).picked_up.is_some()
}

/// C `(s32) f` as the shipping build runs it: gwtool lowers `fptosi` to the saturating
/// `llvm.fptosi.sat.i32` (NaN -> 0), which is exactly Rust's `as i32`.
#[inline]
pub(crate) fn c_f32_to_s32(f: f32) -> i32 {
    f as i32
}

pub use crate::tu::it_26B1::it_8026b3c0;
pub use crate::tu::it_26B1::it_8026c3fc;
pub use crate::tu::it_26B1::it_8026c42c;
pub use crate::tu::itspawn::it_8026ca4c;
pub use crate::tu::itcoll::it_80270cd8;
pub use crate::tu::it_2725::it_80272818_fp as it_80272818;
pub use crate::tu::it_2725::it_80272828_fp as it_80272828;
pub use crate::tu::it_2725::it_80272860_fp as it_80272860;
pub use crate::tu::it_2725::it_80272d1c;
pub use crate::tu::it_2725::it_80273500_fp as it_80273500;
pub use crate::tu::it_2725::it_80274658_fp as it_80274658;
pub use crate::tu::it_2725::it_80274d04;
pub use crate::tu::it_2725::it_80275174_fp as it_80275174;
pub use crate::tu::it_2725::it_80275328_fp as it_80275328;
pub use crate::tu::it_2725::it_80275390_fp as it_80275390;
pub use crate::tu::ithitbox::it_802756e0;
pub use crate::tu::itmaplib::it_80276cec;
pub use crate::tu::itanimlist::it_802795ec;
pub use crate::tu::itanimlist::it_802796fc;
pub use crate::tu::it_279C::it_8027a13c;
pub use crate::tu::it_279C::it_8027a344;
pub use crate::tu::it_279C::it_8027a364;
pub use crate::tu::it_279C::it_8027aaa0;
pub use crate::tu::itzako::it_8027b730;
pub use crate::tu::itzako::it_8027cbfc;
pub use crate::tu::item::item_80268e40;
pub use crate::tu::item::item_80269cc4;
pub use crate::tu::item::item_8026ae60;
