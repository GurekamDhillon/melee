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
    itUnkAttributes, CollData, HSD_GObj, HitCapsule_Disabled, HitCapsule_Unk4, HurtCapsule,
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
const COLLIDE_LEFT_EDGE: i32 = Collide_LeftEdge;
const COLLIDE_RIGHT_EDGE: i32 = Collide_RightEdge;
const COLLIDE_LEFT_WALL_MASK: i32 = Collide_LeftWallMask;
const COLLIDE_RIGHT_WALL_MASK: i32 = Collide_RightWallMask;

// ---- it_26B1.c -------------------------------------------------------------------------

/// `it_8026B294`: the item's position.
pub fn it_8026b294(ip: &Item, pos: &mut GcVec) {
    *pos = ip.pos;
}

/// `itIsHeavy`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_is_heavy(ip: &Item) -> bool {
    (*ip.xCC_item_attr).x0_is_heavy() != 0
}

/// `it_8026B2D8` (`x0_is_heavy == 1`; the same for a 1-bit field).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_8026b2d8(ip: &Item) -> bool {
    (*ip.xCC_item_attr).x0_is_heavy() == 1
}

/// `itGetKind`.
pub fn it_get_kind(ip: &Item) -> ItemKind {
    ip.kind
}

/// `it_8026B30C`: `xCC_item_attr->x0_78` (unsigned 4-bit field).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_8026b30c(ip: &Item) -> i32 {
    (*ip.xCC_item_attr).x0_78() as i32
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

/// `it_8026B344`. 1 fused site: `x = fmadds(facing_dir, xBCC.x, pos.x)`.
pub fn it_8026b344(ip: &Item, pos: &mut GcVec) {
    pos.x = fmadds(ip.facing_dir, ip.xBCC_unk.x, ip.pos.x);
    pos.y = ip.pos.y + ip.xBCC_unk.y;
    pos.z = ip.pos.z;
}

/// `itGetGrabRangeX`.
pub fn it_get_grab_range_x(ip: &Item) -> f32 {
    ip.xBD4_grabRange.x
}

/// `itGetGrabRangeY`.
pub fn it_get_grab_range_y(ip: &Item) -> f32 {
    ip.xBD4_grabRange.y
}

/// `it_8026B390`: `xDC8_word.flags.x15 = 1`.
pub fn it_8026b390(ip: &mut Item) {
    ip.xDC8_word.flags.set_x15(1);
}

/// `it_8026B3A8`: `xDC8_word.flags.x15 = 0`.
pub fn it_8026b3a8(ip: &mut Item) {
    ip.xDC8_word.flags.set_x15(0);
}

/// `it_8026B424`. `common` is the C global `it_804D6D28`. 1 fused site:
/// `fmadds((f32) damage, xB8, xBC)`; `(int)` is gwtool's saturating conversion (= Rust `as`).
pub fn it_8026b424(damage: i32, common: &ItemCommonData) -> f32 {
    fmadds(damage as f32, common.xB8, common.xBC) as i32 as f32
}

/// `it_8026B47C`: heal value of the healing items, else 0.
pub fn it_8026b47c(ip: &Item) -> i32 {
    let v = &ip.xDD4_itemVar;
    unsafe {
        match ip.kind {
            It_Kind_Heart => v.heart.xDD4_heal,
            It_Kind_Tomato => v.tomato.heal_amount,
            It_Kind_Foods => v.foods.heal_amount as i32,
            It_Kind_Lucky_Egg => v.egg.heal_amount,
            It_Kind_WhispyHealApple => v.whispyapple.xDD4_heal,
            _ => 0,
        }
    }
}

/// `it_8026B4F0`: is a healing item.
pub fn it_8026b4f0(ip: &Item) -> bool {
    matches!(ip.kind, It_Kind_Heart | It_Kind_Tomato | It_Kind_Foods | It_Kind_Coin
        | It_Kind_Lucky_Egg | It_Kind_WhispyHealApple)
}

unsafe fn unk_attrs(ip: &Item) -> &itUnkAttributes {
    &*((*ip.xC4_article_data).x4_specialAttributes as *const itUnkAttributes)
}

/// `it_8026B54C`: special attributes `x0_float`.
/// # Safety
/// `ip.xC4_article_data` and its `x4_specialAttributes` must be valid.
pub unsafe fn it_8026b54c(ip: &Item) -> f32 {
    unk_attrs(ip).x0_float
}

/// `it_8026B560` (same body as `it_8026B54C`).
/// # Safety
/// As `it_8026b54c`.
pub unsafe fn it_8026b560(ip: &Item) -> f32 {
    unk_attrs(ip).x0_float
}

/// `it_8026B574`: special attributes `x4_float`.
/// # Safety
/// As `it_8026b54c`.
pub unsafe fn it_8026b574(ip: &Item) -> f32 {
    unk_attrs(ip).x4_float
}

/// `it_8026B588`: `it_804D6D28->xDC`.
pub fn it_8026b588(common: &ItemCommonData) -> i32 {
    common.xDC
}

/// `it_8026B594`: a shooting item with no ammo left. (The C case list falls through to
/// `default`; Harisen is listed under `default`.)
pub fn it_8026b594(ip: &Item) -> bool {
    match ip.kind {
        It_Kind_L_Gun | It_Kind_S_Scope | It_Kind_StarRod | crate::layout::It_Kind_LipStick
        | It_Kind_F_Flower => ip.xD4C <= 0,
        _ => false,
    }
}

/// `it_8026B6C8`: is a stage item (Kuriboh..Octarock_Stone or Old_Kuri..Arwing_Laser), except
/// Kyasarin_Egg and WhispyApple.
pub fn it_8026b6c8(ip: &Item) -> bool {
    let kind = ip.kind;
    if (kind >= It_Kind_Kuriboh && kind < It_Kind_Octarock_Stone)
        || (kind >= It_Kind_Old_Kuri && kind < It_Kind_Arwing_Laser)
    {
        if kind != It_Kind_Kyasarin_Egg && kind != It_Kind_WhispyApple {
            return true;
        }
    }
    false
}

/// `it_8026B718`.
pub fn it_8026b718(ip: &mut Item, hitlag_frames: f32) {
    ip.xCBC_hitlagFrames = hitlag_frames;
}

/// `it_8026B724`: `xDC8_word.flags.x3 = 1`.
pub fn it_8026b724(ip: &mut Item) {
    ip.xDC8_word.flags.set_x3(1);
}

/// `it_8026B73C`.
pub fn it_8026b73c(ip: &mut Item) {
    if ip.xDC8_word.flags.x7() != 0 {
        ip.xDC8_word.flags.set_x5(1);
    }
    if ip.xDC8_word.flags.x3() != 0 {
        ip.xDC8_word.flags.set_x3(0);
    }
}

/// `it_8026B774`: bit `arg1` of the 4-bit `xDC8_word.flags.xF`. The C shift is `int << u8`;
/// shifts of 32+ are UB there, and `arg1` is a small constant at every call site, so it is
/// limited to 0..31 here (the oracle covers 0..31).
pub fn it_8026b774(ip: &Item, arg1: u8) -> bool {
    (ip.xDC8_word.flags.xF() as i32 & (1i32.wrapping_shl(arg1 as u32))) != 0
}

/// `itGetMotionId`.
pub fn it_get_motion_id(ip: &Item) -> i32 {
    ip.msid
}

/// `itGetTeamId`.
pub fn it_get_team_id(ip: &Item) -> u8 {
    ip.x20_team_id
}

/// `it_8026B7BC`: `xDC8_word.flags.x14`.
pub fn it_8026b7bc(ip: &Item) -> i32 {
    ip.xDC8_word.flags.x14() as i32
}

/// `it_8026B7CC`.
pub fn it_8026b7cc(ip: &Item) -> i32 {
    ip.x1C
}

/// `it_8026B7E8`: `xDC8_word.flags.x1`.
pub fn it_8026b7e8(ip: &Item) -> i32 {
    ip.xDC8_word.flags.x1() as i32
}

/// `it_8026B894`: clear every reference to `ref_gobj`; true when it was the owner.
pub fn it_8026b894(ip: &mut Item, ref_gobj: *mut HSD_GObj) -> bool {
    let mut result = false;
    if ip.owner == ref_gobj {
        ip.owner = null_mut();
        result = true;
    }
    if ip.xC64_reflectGObj == ref_gobj {
        ip.xC64_reflectGObj = null_mut();
    }
    if ip.xC90_absorbGObj == ref_gobj {
        ip.xC90_absorbGObj = null_mut();
    }
    if ip.xCEC_fighterGObj == ref_gobj {
        ip.xCEC_fighterGObj = null_mut();
        ip.xCB0_source_ply = 6;
    }
    if ip.xCF4_fighterGObjUnk == ref_gobj {
        ip.xCF4_fighterGObjUnk = null_mut();
    }
    if ip.toucher == ref_gobj {
        ip.toucher = null_mut();
    }
    result
}

/// `it_8026B924`: remaining ammo of the shooting items, else -1.
pub fn it_8026b924(ip: &Item) -> i32 {
    match ip.kind {
        It_Kind_S_Scope | It_Kind_StarRod | It_Kind_L_Gun | It_Kind_F_Flower => ip.xD4C,
        _ => -1,
    }
}

/// `it_8026B960`: a bomb's timer, else -1.0.
pub fn it_8026b960(ip: &Item) -> f32 {
    let mut unk_timer = -1.0f32;
    match ip.kind {
        It_Kind_BombHei => {
            if ip.msid != 0xB {
                unk_timer = unsafe { ip.xDD4_itemVar.bombhei.xDEC };
            }
        }
        It_Kind_Link_Bomb => {
            if ip.msid != 5 {
                unk_timer = ip.xD44_lifeTimer;
            }
        }
        _ => {}
    }
    unk_timer
}

/// `it_8026BB88`: the ECB centre. `offset_xz` is a literal 0.0 that the C still adds.
pub fn it_8026bb88(ip: &Item, pos: &mut GcVec) {
    let ecb = &ip.x378_itemColl.ecb;
    let offset_xz = 0.0f32;
    let offset_y = 0.5f32 * (ecb.top.y + ecb.bottom.y);
    pos.x = ip.pos.x + offset_xz;
    pos.y = ip.pos.y + offset_y;
    pos.z = ip.pos.z + offset_xz;
}

/// `it_8026BBCC`: the same from `last_pos` and `xE4_ecb`.
pub fn it_8026bbcc(ip: &Item, pos: &mut GcVec) {
    let cd = &ip.x378_itemColl;
    let ecb = &cd.xE4_ecb;
    let offset_xz = 0.0f32;
    let offset_y = 0.5f32 * (ecb.top.y + ecb.bottom.y);
    pos.x = cd.last_pos.x + offset_xz;
    pos.y = cd.last_pos.y + offset_y;
    pos.z = cd.last_pos.z + offset_xz;
}

/// `it_8026BC68`: `xDD0_flag.b0`.
pub fn it_8026bc68(ip: &Item) -> bool {
    unsafe { ip.xDD0_flag.anon_0.b0() != 0 }
}

/// `itGetAttackId`.
pub fn it_get_attack_id(ip: &Item) -> i32 {
    ip.xD88_attackID
}

/// `it_8026BCF4`: `xDCD_flag.b2 = 0`.
pub fn it_8026bcf4(ip: &mut Item) {
    ip.xDCD_flag.anon_0.set_b2(0);
}

/// `it_8026BD0C`: `xDCD_flag.b2 = 1`.
pub fn it_8026bd0c(ip: &mut Item) {
    ip.xDCD_flag.anon_0.set_b2(1);
}

/// `it_8026BD24`: `xDD0_flag.b3 = 1`.
pub fn it_8026bd24(ip: &mut Item) {
    unsafe { ip.xDD0_flag.anon_0.set_b3(1) }
}

/// `it_8026BD3C`: `xDCC_flag.b3 = 1`.
pub fn it_8026bd3c(ip: &mut Item) {
    ip.xDCC_flag.set_b3(1);
}

/// `it_8026BD54`: `xDCC_flag.b3 = 0`.
pub fn it_8026bd54(ip: &mut Item) {
    ip.xDCC_flag.set_b3(0);
}

/// `it_8026BD6C`: `xDCD_flag.b3 = 1`.
pub fn it_8026bd6c(ip: &mut Item) {
    ip.xDCD_flag.anon_0.set_b3(1);
}

/// `it_8026BD84`: `xDCD_flag.b4 = 1`.
pub fn it_8026bd84(ip: &mut Item) {
    ip.xDCD_flag.anon_0.set_b4(1);
}

/// `it_8026BD9C`: `xDC8_word.flags.x1A = 1`.
pub fn it_8026bd9c(ip: &mut Item) {
    ip.xDC8_word.flags.set_x1A(1);
}

/// `it_8026BDCC` (five inlined setters).
pub fn it_8026bdcc(ip: &mut Item) {
    unsafe { ip.xDD0_flag.anon_0.set_b3(0) };
    ip.xDCC_flag.set_b3(1);
    ip.xDCD_flag.anon_0.set_b3(0);
    ip.xDCD_flag.anon_0.set_b4(0);
    ip.xDC8_word.flags.set_x1A(1);
}

/// `it_8026BE28` (the inverse of `it_8026BDCC`).
pub fn it_8026be28(ip: &mut Item) {
    unsafe { ip.xDD0_flag.anon_0.set_b3(1) };
    ip.xDCC_flag.set_b3(0);
    ip.xDCD_flag.anon_0.set_b3(1);
    ip.xDCD_flag.anon_0.set_b4(1);
    ip.xDC8_word.flags.set_x1A(0);
}

/// `it_8026C1E8`: not grabbing, and not held by an owner.
pub fn it_8026c1e8(ip: &Item) -> bool {
    !(!ip.grab_victim.is_null() || (ip.xDC8_word.flags.x13() != 0 && !ip.owner.is_null()))
}

/// `it_8026C334`: position offset by the ECB bottom.
pub fn it_8026c334(ip: &Item, pos: &mut GcVec) {
    let offset_xz = 0.0f32;
    let offset_y = ip.x378_itemColl.ecb.bottom.y;
    pos.x = ip.pos.x + offset_xz;
    pos.y = ip.pos.y + offset_y;
    pos.z = ip.pos.z + offset_xz;
}

// ---- it_2725.c -------------------------------------------------------------------------

/// `it_80273030`: bounce back (`vel = -vel * xC70`, turn around, reset the life timer).
pub fn it_80273030(ip: &mut Item) -> bool {
    ip.x40_vel.x = -ip.x40_vel.x * ip.xC70;
    ip.x40_vel.y = -ip.x40_vel.y * ip.xC70;
    ip.facing_dir = -ip.facing_dir;
    ip.xD44_lifeTimer = ip.xD48_halfLifeTimer;
    false
}

/// `it_80273130`: count the life timer down; true when it ran out.
pub fn it_80273130(ip: &mut Item) -> bool {
    ip.xD44_lifeTimer -= 1.0;
    ip.xD44_lifeTimer <= 0.0
}

/// `it_8027346C`.
pub fn it_8027346c(ip: &mut Item) {
    ip.x64_vec_unk2 = GcVec { x: 0.0, y: 0.0, z: 0.0 };
}

/// `it_80273484`.
pub fn it_80273484(ip: &mut Item) {
    ip.x58_vec_unk = GcVec { x: 0.0, y: 0.0, z: 0.0 };
}

/// `it_8027349C`.
pub fn it_8027349c(ip: &mut Item) {
    ip.x70_nudge = GcVec { x: 0.0, y: 0.0, z: 0.0 };
}

/// `it_802734B4`.
pub fn it_802734b4(ip: &mut Item) {
    let z = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    ip.x7C = z;
    ip.x88 = z;
    ip.x94 = z;
    ip.xA0 = z;
    ip.xAC_unk.x = 0.0;
    ip.xAC_unk.z = 0.0;
    ip.xAC_unk.y = 1.0;
}

/// `it_80274C60`: `xDC8_word.flags.xC = 0`.
pub fn it_80274c60(ip: &mut Item) {
    ip.xDC8_word.flags.set_xC(0);
}

/// `it_80274C78`: `xDC8_word.flags.xC`.
pub fn it_80274c78(ip: &Item) -> bool {
    ip.xDC8_word.flags.xC() != 0
}

/// `it_80274C88`.
pub fn it_80274c88(ip: &mut Item) {
    unsafe {
        ip.xDD0_flag.anon_0.set_b0(0);
        ip.anon_daa.xDAA_flag.anon_0.set_b3(0);
    }
}

/// `it_80274D6C`: the `xB54.x8` box from the ECB (`z = right.y + pos.y + top.y`, left to right).
pub fn it_80274d6c(ip: &mut Item) {
    let ecb = &ip.x378_itemColl.ecb;
    let (right_x, right_y, left_x, top_y) = (ecb.right.x, ecb.right.y, ecb.left.x, ecb.top.y);
    ip.xB54.x8.y = right_x + ip.pos.x;
    ip.xB54.x8.x = left_x + ip.pos.x;
    ip.xB54.x8.z = right_y + ip.pos.y + top_y;
}

/// `it_80274DAC`: clear `xB54.unk_x/y`, then `it_80274D6C`'s body (inlined in the C).
pub fn it_80274dac(ip: &mut Item) {
    ip.xB54.unk_x = 0.0;
    ip.xB54.unk_y = 0.0;
    it_80274d6c(ip);
}

/// `it_80274DFC`: scale `xBCC` and the grab range by `scl`.
pub fn it_80274dfc(ip: &mut Item) {
    ip.xBCC_unk.x *= ip.scl;
    ip.xBCC_unk.y *= ip.scl;
    ip.xBD4_grabRange.x *= ip.scl;
    ip.xBD4_grabRange.y *= ip.scl;
}

/// `it_80274E44`: scale `xBEC` and `xBDC` by `scl`.
pub fn it_80274e44(ip: &mut Item) {
    let s = ip.scl;
    ip.xBEC.top *= s;
    ip.xBEC.bottom *= s;
    ip.xBEC.right *= s;
    ip.xBEC.left *= s;
    ip.xBDC.top *= s;
    ip.xBDC.bottom *= s;
    ip.xBDC.right *= s;
    ip.xBDC.left *= s;
}

/// `it_80274ECC`: `xD09 = arg1` (C `bool` stored to a `u8`).
pub fn it_80274ecc(ip: &mut Item, arg1: i32) {
    ip.xD09 = arg1 as u8;
}

/// `it_80274EF8`: `xDC8_word.flags.x2 = 0`.
pub fn it_80274ef8(ip: &mut Item) {
    ip.xDC8_word.flags.set_x2(0);
}

/// `it_80274F10`: `xDC8_word.flags.x2 = 1`.
pub fn it_80274f10(ip: &mut Item) {
    ip.xDC8_word.flags.set_x2(1);
}

/// `it_802750E8`: `xDC0 & mask`. The C return type `bool` is `int` in this codebase
/// (src/MSL/stdbool.h), so the masked bits are returned, not 0/1.
pub fn it_802750e8(ip: &Item, mask: i32) -> i32 {
    (ip.xDC0 & mask as u32) as i32
}

/// `it_802751D8` (same body as `it_80273130`).
pub fn it_802751d8(ip: &mut Item) -> bool {
    ip.xD44_lifeTimer -= 1.0;
    ip.xD44_lifeTimer <= 0.0
}

/// `it_80275210`: `xDC8_word.flags.x1C = 1`.
pub fn it_80275210(ip: &mut Item) {
    ip.xDC8_word.flags.set_x1C(1);
}

/// `it_80275228`: `xDC8_word.flags.x1C = 0`.
pub fn it_80275228(ip: &mut Item) {
    ip.xDC8_word.flags.set_x1C(0);
}

/// `it_80275240`: `xDC8_word.flags.x1E = 0`.
pub fn it_80275240(ip: &mut Item) {
    ip.xDC8_word.flags.set_x1E(0);
}

/// `it_80275258`: `xDD0_flag.b4 = 0`.
pub fn it_80275258(ip: &mut Item) {
    unsafe { ip.xDD0_flag.anon_0.set_b4(0) }
}

/// `it_80275270`: `xDD0_flag.b4 = 1`.
pub fn it_80275270(ip: &mut Item) {
    unsafe { ip.xDD0_flag.anon_0.set_b4(1) }
}

/// `it_802753BC`: reset the damage counters, `xC9C = arg1`.
pub fn it_802753bc(ip: &mut Item, arg1: i16) {
    ip.xC94 = 0;
    ip.xC9C = arg1 as i32;
    ip.xCA0 = 0;
    ip.xCA4 = 0;
}

// ---- ithitbox.c ------------------------------------------------------------------------

/// `it_802753DC`.
pub fn it_802753dc(ip: &mut Item) {
    ip.xC34_damageDealt = 0;
    ip.xC40 = 1.0;
    ip.xC44 = 1.0;
    ip.xC3C = 1.0;
    ip.xC48 = 0;
    ip.xC4C = 0;
    ip.xC50 = 0;
    unsafe { ip.xDCE_flag.anon_0.set_b6(0) }
}

/// `it_80275414`: `xDCD_flag.b7 = 1`.
pub fn it_80275414(ip: &mut Item) {
    ip.xDCD_flag.anon_0.set_b7(1);
}

/// `it_8027542C`: `xDCD_flag.b7 = 0`.
pub fn it_8027542c(ip: &mut Item) {
    ip.xDCD_flag.anon_0.set_b7(0);
}

/// `it_80275444`: `xDCD_flag.b5 = b7 = b6 = 1`.
pub fn it_80275444(ip: &mut Item) {
    let f = &mut ip.xDCD_flag.anon_0;
    f.set_b5(1);
    f.set_b7(1);
    f.set_b6(1);
}

/// `it_80275474`: `xDCD_flag.b5 = b7 = b6 = 0`.
pub fn it_80275474(ip: &mut Item) {
    let f = &mut ip.xDCD_flag.anon_0;
    f.set_b5(0);
    f.set_b7(0);
    f.set_b6(0);
}

/// `it_802754A4`: `xDCE_flag.b2 = 1`.
pub fn it_802754a4(ip: &mut Item) {
    unsafe { ip.xDCE_flag.anon_0.set_b2(1) }
}

/// `it_802754BC`: `xDCE_flag.b2 = 0`.
pub fn it_802754bc(ip: &mut Item) {
    unsafe { ip.xDCE_flag.anon_0.set_b2(0) }
}

/// `it_802754D4`: `xDCE_flag.b0 = b2 = b1 = 1`.
pub fn it_802754d4(ip: &mut Item) {
    unsafe {
        let f = &mut ip.xDCE_flag.anon_0;
        f.set_b0(1);
        f.set_b2(1);
        f.set_b1(1);
    }
}

/// `it_80275504`: `xDCE_flag.b0 = b2 = b1 = 0`.
pub fn it_80275504(ip: &mut Item) {
    unsafe {
        let f = &mut ip.xDCE_flag.anon_0;
        f.set_b0(0);
        f.set_b2(0);
        f.set_b1(0);
    }
}

/// `it_80275534`: set the scale of every enabled hitbox.
pub fn it_80275534(ip: &mut Item, scale: f32) {
    for hb in ip.x5D4_hitboxes.iter_mut() {
        if hb.hit.state != HitCapsule_Disabled {
            hb.hit.scale = scale;
        }
    }
}

/// `it_80275594`: multiply one hitbox's scale. Panics on an `idx` outside 0..4 (the C would
/// index past the array).
pub fn it_80275594(ip: &mut Item, idx: i32, mult: f32) {
    let hit = &mut ip.x5D4_hitboxes[idx as usize].hit;
    if hit.state != HitCapsule_Disabled {
        hit.scale *= mult;
    }
}

/// `it_802755C0`: multiply the scale of every enabled hitbox.
pub fn it_802755c0(ip: &mut Item, scale: f32) {
    for hb in ip.x5D4_hitboxes.iter_mut() {
        if hb.hit.state != HitCapsule_Disabled {
            hb.hit.scale *= scale;
        }
    }
}

/// `it_802756D0`: `xD0C = 2`.
pub fn it_802756d0(ip: &mut Item) {
    ip.xD0C = 2;
}

/// `it_8027570C`: `x5D4_hitboxes[idx].hit.x40_b0 = 0`.
pub fn it_8027570c(ip: &mut Item, idx: i32) {
    ip.x5D4_hitboxes[idx as usize].hit.set_x40_b0(0);
}

/// `it_8027572C`: `x5D4_hitboxes[idx].hit.x40_b0 = 1`.
pub fn it_8027572c(ip: &mut Item, idx: i32) {
    ip.x5D4_hitboxes[idx as usize].hit.set_x40_b0(1);
}

/// `it_80275788`: every enabled hitbox goes to `HitCapsule_Unk4`, and sets `xDAA_flag.b2`.
pub fn it_80275788(ip: &mut Item) {
    for k in 0..4 {
        if ip.x5D4_hitboxes[k].hit.state != HitCapsule_Disabled {
            ip.x5D4_hitboxes[k].hit.state = HitCapsule_Unk4;
            unsafe { ip.anon_daa.xDAA_flag.anon_0.set_b2(1) };
        }
    }
}

/// `it_80275820`: set one enabled hitbox's `x58 = *arg2`, `x4C = *arg1`.
pub fn it_80275820(ip: &mut Item, arg1: &GcVec, arg2: &GcVec, idx: i32) {
    let hit = &mut ip.x5D4_hitboxes[idx as usize].hit;
    if hit.state != HitCapsule_Disabled {
        hit.x58 = *arg2;
        hit.x4C = *arg1;
    }
}

/// `it_80275870`: any hitbox enabled.
pub fn it_80275870(ip: &Item) -> bool {
    ip.x5D4_hitboxes.iter().any(|hb| hb.hit.state != HitCapsule_Disabled)
}

/// `it_802758D4`: the largest damage of the enabled hitboxes (0.0 when none; `<=` keeps the
/// later of equal values, which only matters for signed zeros).
pub fn it_802758d4(ip: &Item) -> f32 {
    let mut damage = 0.0f32;
    if it_80275870(ip) {
        for hb in ip.x5D4_hitboxes.iter() {
            if hb.hit.state != HitCapsule_Disabled && damage <= hb.hit.damage {
                damage = hb.hit.damage;
            }
        }
    }
    damage
}

// ---- itmaplib.c ------------------------------------------------------------------------

/// `it_802762D8`: on a left or right edge.
pub fn it_802762d8(ip: &Item) -> bool {
    let f = ip.x378_itemColl.env_flags;
    f & COLLIDE_LEFT_EDGE != 0 || f & COLLIDE_RIGHT_EDGE != 0
}

/// `it_80276308`: 8 for a left wall, 4 for a right wall (right wins), recording its index.
pub fn it_80276308(ip: &mut Item) -> i32 {
    let mut ret = 0;
    let cd = &ip.x378_itemColl;
    if cd.env_flags & COLLIDE_LEFT_WALL_MASK != 0 {
        ret = 8;
        ip.xC30 = cd.left_facing_wall.index;
    }
    if cd.env_flags & COLLIDE_RIGHT_WALL_MASK != 0 {
        ip.xC30 = cd.right_facing_wall.index;
        ret = 4;
    }
    ret
}

/// `it_80276348`: `it_80276308` plus the wall normal.
pub fn it_80276348(ip: &mut Item, vec: &mut GcVec) -> i32 {
    let mut ret = 0;
    let cd = &ip.x378_itemColl;
    if cd.env_flags & COLLIDE_LEFT_WALL_MASK != 0 {
        ret = 8;
        ip.xC30 = cd.left_facing_wall.index;
        *vec = cd.left_facing_wall.normal;
    }
    if cd.env_flags & COLLIDE_RIGHT_WALL_MASK != 0 {
        ip.xC30 = cd.right_facing_wall.index;
        *vec = cd.right_facing_wall.normal;
        ret = 4;
    }
    ret
}

/// `it_802763B8`: on a floor, recording its index.
pub fn it_802763b8(ip: &mut Item) -> bool {
    if ip.x378_itemColl.env_flags & COLLIDE_FLOOR_MASK != 0 {
        ip.xC30 = ip.x378_itemColl.floor.index;
        return true;
    }
    false
}

/// `it_80276408`: the normal of the touched surface (floor > ceiling > right > left). The C's
/// `item_gobj` is unused.
pub fn it_80276408(coll: &CollData, vec: &mut GcVec) {
    if coll.env_flags & COLLIDE_LEFT_WALL_MASK != 0 {
        *vec = coll.left_facing_wall.normal;
    }
    if coll.env_flags & COLLIDE_RIGHT_WALL_MASK != 0 {
        *vec = coll.right_facing_wall.normal;
    }
    if coll.env_flags & COLLIDE_CEILING_MASK != 0 {
        *vec = coll.ceiling.normal;
    }
    if coll.env_flags & COLLIDE_FLOOR_MASK != 0 {
        *vec = coll.floor.normal;
    }
}

// ---- itcoll.c --------------------------------------------------------------------------

/// `it_802714C0`: clear `skip_update_pos` on the item's hurtboxes, set `xDAA_flag.b1`.
/// Panics if `xAC8_hurtboxNum > 2` (the C would index past the array).
pub fn it_802714c0(ip: &mut Item) {
    for k in 0..ip.xAC8_hurtboxNum as usize {
        ip.xACC_itemHurtbox[k].set_skip_update_pos(0);
    }
    unsafe { ip.anon_daa.xDAA_flag.anon_0.set_b1(1) };
}

/// `it_80271508`: set the state of the item's hurtboxes.
pub fn it_80271508(ip: &mut Item, state: HurtCapsuleState) {
    for k in 0..ip.xAC8_hurtboxNum as usize {
        ip.xACC_itemHurtbox[k].state = state;
    }
}

/// `it_80271534`: copy offsets and scale into hurtbox `index`, if the article has hurt bones.
/// # Safety
/// `ip.xC4_article_data` must be valid.
pub unsafe fn it_80271534(ip: &mut Item, index: i32, arg_hurt: &HurtCapsule) {
    if !(*ip.xC4_article_data).x8_hurtbones.is_null() {
        let hurt = &mut ip.xACC_itemHurtbox[index as usize];
        hurt.a_offset = arg_hurt.a_offset;
        hurt.b_offset = arg_hurt.b_offset;
        hurt.scale = arg_hurt.scale;
    }
}

/// `it_80271590`: read hurtbox `index`'s offsets and scale (zeros without hurt bones).
/// # Safety
/// `ip.xC4_article_data` must be valid.
pub unsafe fn it_80271590(ip: &Item, index: i32, arg_hurt: &mut HurtCapsule) {
    if !(*ip.xC4_article_data).x8_hurtbones.is_null() {
        let hurt = &ip.xACC_itemHurtbox[index as usize];
        arg_hurt.a_offset = hurt.a_offset;
        arg_hurt.b_offset = hurt.b_offset;
        arg_hurt.scale = hurt.scale;
        return;
    }
    let z = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    arg_hurt.a_offset = z;
    arg_hurt.b_offset = z;
    arg_hurt.scale = 0.0;
}

/// `it_80272280`: `xDC8_word.flags.x1B = 0`.
pub fn it_80272280(ip: &mut Item) {
    ip.xDC8_word.flags.set_x1B(0);
}

/// `it_80272298`: `xDC8_word.flags.x1B = 1`.
pub fn it_80272298(ip: &mut Item) {
    ip.xDC8_word.flags.set_x1B(1);
}
