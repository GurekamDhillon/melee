
//! Item library accessors (port round 2, night 2): the `Item`-only getters/setters and small
//! helpers of it_26B1.c, it_2725.c, ithitbox.c, itmaplib.c and itcoll.c. Each cites the C it
//! mirrors; verified against verbatim C copies in tests/c_oracle/round2.c (tests/oracle.rs).
//!
//! Conventions (same as `helpers.rs`): where the C takes an `HSD_GObj*`/`Item_GObj*` and only
//! uses its user data, the port takes the `Item`. A C pointer the function follows
//! (`xCC_item_attr`, `xC4_article_data`) stays a raw pointer in the native struct, so those
//! functions are `unsafe` with the C's precondition (the pointer is valid). GObj identities are
//! compared as raw pointers, as the C does. Fused multiply-adds: `tools/rust-port/fma_sites.py`
//! finds one f32 site in this set (`it_8026B344`) and one in `it_8026B424`; every other function
//! here has none.
#![allow(non_upper_case_globals)]

use crate::layout::{
    itUnkAttributes_legacy, CollData, HSD_GObj, HitCapsule_Disabled, HitCapsule_Unk4, HurtCapsule,
    HurtCapsuleState, Item, ItemCommonData, ItemKind, Vec as GcVec, Collide_FloorHug, Collide_FloorPush,
    Collide_LeftEdge, Collide_LeftWallMask, Collide_RightEdge, Collide_RightWallMask,
    It_Kind_Arwing_Laser, It_Kind_BombHei, It_Kind_Coin, It_Kind_F_Flower, It_Kind_Foods,
    It_Kind_Heart, It_Kind_Kuriboh, It_Kind_Kyasarin_Egg, It_Kind_L_Gun, It_Kind_Link_Bomb,
    It_Kind_Lucky_Egg, It_Kind_Octarock_Stone, It_Kind_Old_Kuri, It_Kind_S_Scope,
    It_Kind_StarRod, It_Kind_Tomato, It_Kind_WhispyApple, It_Kind_WhispyHealApple,
};

use crate::helpers::COLLIDE_CEILING_MASK;

use melee_core::fp::fmadds;

use std::ptr::null_mut;

/// `Collide_FloorMask` (src/melee/mp/forward.h).
pub const COLLIDE_FLOOR_MASK: i32 = Collide_FloorPush | Collide_FloorHug;

pub(crate) const COLLIDE_LEFT_EDGE: i32 = Collide_LeftEdge;

pub(crate) const COLLIDE_RIGHT_EDGE: i32 = Collide_RightEdge;

pub(crate) const COLLIDE_LEFT_WALL_MASK: i32 = Collide_LeftWallMask;

pub(crate) const COLLIDE_RIGHT_WALL_MASK: i32 = Collide_RightWallMask;

/// `itIsHeavy`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_is_heavy(ip: &Item) -> bool {
    (*ip.xCC_item_attr).x0_is_heavy() != 0
}

/// `itGetKind`.
pub fn it_get_kind(ip: &Item) -> ItemKind {
    ip.kind
}

/// `itGetHoldKind`: `xCC_item_attr->x0_hold_kind` (unsigned 3-bit field).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_get_hold_kind(ip: &Item) -> i32 {
    (*ip.xCC_item_attr).x0_hold_kind() as i32
}

/// `itGetDamageMultiplier`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_get_damage_multiplier(ip: &Item) -> f32 {
    (*ip.xCC_item_attr).x1C_damage_mul
}

/// `itGetGrabRangeX`.
pub fn it_get_grab_range_x(ip: &Item) -> f32 {
    ip.xBD4_grabRange.x
}

/// `itGetGrabRangeY`.
pub fn it_get_grab_range_y(ip: &Item) -> f32 {
    ip.xBD4_grabRange.y
}

pub(crate) unsafe fn unk_attrs(ip: &Item) -> &itUnkAttributes_legacy {
    &*((*ip.xC4_article_data).x4_specialAttributes as *const itUnkAttributes_legacy)
}

/// `itGetMotionId`.
pub fn it_get_motion_id(ip: &Item) -> i32 {
    ip.msid
}

/// `itGetTeamId`.
pub fn it_get_team_id(ip: &Item) -> u8 {
    ip.x20_team_id
}

/// `itGetAttackId`.
pub fn it_get_attack_id(ip: &Item) -> i32 {
    ip.xD88_attackID
}

pub use crate::tu::it_26B1::it_8026b294;
pub use crate::tu::it_26B1::it_8026b2d8;
pub use crate::tu::it_26B1::it_8026b30c;
pub use crate::tu::it_26B1::it_8026b344;
pub use crate::tu::it_26B1::it_8026b390;
pub use crate::tu::it_26B1::it_8026b3a8;
pub use crate::tu::it_26B1::it_8026b424;
pub use crate::tu::it_26B1::it_8026b47c;
pub use crate::tu::it_26B1::it_8026b4f0;
pub use crate::tu::it_26B1::it_8026b54c;
pub use crate::tu::it_26B1::it_8026b560;
pub use crate::tu::it_26B1::it_8026b574;
pub use crate::tu::it_26B1::it_8026b588;
pub use crate::tu::it_26B1::it_8026b594_fp as it_8026b594;
pub use crate::tu::it_26B1::it_8026b6c8;
pub use crate::tu::it_26B1::it_8026b718;
pub use crate::tu::it_26B1::it_8026b724;
pub use crate::tu::it_26B1::it_8026b73c;
pub use crate::tu::it_26B1::it_8026b774;
pub use crate::tu::it_26B1::it_8026b7bc;
pub use crate::tu::it_26B1::it_8026b7cc;
pub use crate::tu::it_26B1::it_8026b7e8;
pub use crate::tu::it_26B1::it_8026b894;
pub use crate::tu::it_26B1::it_8026b924;
pub use crate::tu::it_26B1::it_8026b960;
pub use crate::tu::it_26B1::it_8026bb88;
pub use crate::tu::it_26B1::it_8026bbcc;
pub use crate::tu::it_26B1::it_8026bc68;
pub use crate::tu::it_26B1::it_8026bcf4;
pub use crate::tu::it_26B1::it_8026bd0c;
pub use crate::tu::it_26B1::it_8026bd24;
pub use crate::tu::it_26B1::it_8026bd3c;
pub use crate::tu::it_26B1::it_8026bd54;
pub use crate::tu::it_26B1::it_8026bd6c;
pub use crate::tu::it_26B1::it_8026bd84;
pub use crate::tu::it_26B1::it_8026bd9c;
pub use crate::tu::it_26B1::it_8026bdcc;
pub use crate::tu::it_26B1::it_8026be28;
pub use crate::tu::it_26B1::it_8026c1e8;
pub use crate::tu::it_26B1::it_8026c334;
pub use crate::tu::itcoll::it_802714c0;
pub use crate::tu::itcoll::it_80271508_fp as it_80271508;
pub use crate::tu::itcoll::it_80271534_fp as it_80271534;
pub use crate::tu::itcoll::it_80271590_fp as it_80271590;
pub use crate::tu::itcoll::it_80272280_fp as it_80272280;
pub use crate::tu::itcoll::it_80272298_fp as it_80272298;
pub use crate::tu::it_2725::it_80273030_fp as it_80273030;
pub use crate::tu::it_2725::it_80273130_fp as it_80273130;
pub use crate::tu::it_2725::it_8027346c;
pub use crate::tu::it_2725::it_80273484_fp as it_80273484;
pub use crate::tu::it_2725::it_8027349c;
pub use crate::tu::it_2725::it_802734b4;
pub use crate::tu::it_2725::it_80274c60;
pub use crate::tu::it_2725::it_80274c78;
pub use crate::tu::it_2725::it_80274c88;
pub use crate::tu::it_2725::it_80274d6c;
pub use crate::tu::it_2725::it_80274dac;
pub use crate::tu::it_2725::it_80274dfc;
pub use crate::tu::it_2725::it_80274e44;
pub use crate::tu::it_2725::it_80274ecc;
pub use crate::tu::it_2725::it_80274ef8;
pub use crate::tu::it_2725::it_80274f10;
pub use crate::tu::it_2725::it_802750e8;
pub use crate::tu::it_2725::it_802751d8;
pub use crate::tu::it_2725::it_80275210_fp as it_80275210;
pub use crate::tu::it_2725::it_80275228_fp as it_80275228;
pub use crate::tu::it_2725::it_80275240_fp as it_80275240;
pub use crate::tu::it_2725::it_80275258_fp as it_80275258;
pub use crate::tu::it_2725::it_80275270_fp as it_80275270;
pub use crate::tu::it_2725::it_802753bc;
pub use crate::tu::ithitbox::it_802753dc;
pub use crate::tu::ithitbox::it_80275414_fp as it_80275414;
pub use crate::tu::ithitbox::it_8027542c;
pub use crate::tu::ithitbox::it_80275444_fp as it_80275444;
pub use crate::tu::ithitbox::it_80275474_fp as it_80275474;
pub use crate::tu::ithitbox::it_802754a4;
pub use crate::tu::ithitbox::it_802754bc;
pub use crate::tu::ithitbox::it_802754d4;
pub use crate::tu::ithitbox::it_80275504_fp as it_80275504;
pub use crate::tu::ithitbox::it_80275534_fp as it_80275534;
pub use crate::tu::ithitbox::it_80275594_fp as it_80275594;
pub use crate::tu::ithitbox::it_802755c0;
pub use crate::tu::ithitbox::it_802756d0;
pub use crate::tu::ithitbox::it_8027570c;
pub use crate::tu::ithitbox::it_8027572c;
pub use crate::tu::ithitbox::it_80275788_fp as it_80275788;
pub use crate::tu::ithitbox::it_80275820_fp as it_80275820;
pub use crate::tu::ithitbox::it_80275870_fp as it_80275870;
pub use crate::tu::ithitbox::it_802758d4;
pub use crate::tu::itmaplib::it_802762d8;
pub use crate::tu::itmaplib::it_80276308_fp as it_80276308;
pub use crate::tu::itmaplib::it_80276348_fp as it_80276348;
pub use crate::tu::itmaplib::it_802763b8;
pub use crate::tu::itmaplib::it_80276408_fp as it_80276408;
