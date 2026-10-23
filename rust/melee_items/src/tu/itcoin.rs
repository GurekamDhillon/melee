//! Native port of src/melee/it/kinds/itcoin.c (the Coin item): spawn, the bounce/landing states
//! (`it_803F93C8`), the damage response and the figure-visibility toggle. Callees in other TUs go
//! through `ext`; callees ported in this crate are called directly. The C globals `it_804D6D40`
//! and `p_ftCommonData` are ext statics. `*_p` wrappers return `i32` for the state table
//! (`HSD_GObjPredicate`). Checked by tests/tu_itcoin.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use crate::itlib::{it_8026b390, it_8026b894, it_802756d0, it_8026b3a8};
use crate::itlib3::{it_80272860, it_802756e0, it_8027cbfc};
use crate::item::it_802762bc;
use crate::itkinds10::it_8027b798;

/// `ITEM_ANIM_UPDATE` (src/melee/it/forward.h): `1 << 1`.
pub const ITEM_ANIM_UPDATE: u32 = 1 << 1;

// BEGIN GENERATED EXT (callees of the C TU that are not ported in this crate yet)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268B18(spawn: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, flag: bool);
    fn Item_80267454(gobj: *mut HSD_GObj);
    fn Item_80268E5C(gobj: *mut HSD_GObj, msid: i32, flags: u32);
    fn it_2725_Logic9_Destroyed(gobj: *mut HSD_GObj);
    fn it_8027CE44(gobj: *mut HSD_GObj);
    fn it_8026EECC(gobj: *mut HSD_GObj, arg1: i32);
    fn it_8026E414(gobj: *mut HSD_GObj, cb: HSD_GObjEvent);
    fn it_8026D62C(gobj: *mut HSD_GObj, cb: HSD_GObjEvent);
    fn it_8026E15C(gobj: *mut HSD_GObj, cb: HSD_GObjEvent);
    fn it_8026E8C4(gobj: *mut HSD_GObj, a: HSD_GObjEvent, b: HSD_GObjEvent);
    fn it_8027163C(gobj: *mut HSD_GObj);
    fn grFigureGet_80219C34(gobj: *mut HSD_GObj);
    fn grFigureGet_80219C50(gobj: *mut HSD_GObj) -> i32;
    fn gm_GetDbPauseFlag(i: i32) -> bool;
    fn gm_8017E280(a0: u16, a1: u32) -> i8;
    fn Camera_80031144() -> f32;
    fn Ground_801C1D84() -> u32;
    fn Ground_801C0498() -> f32;
    fn tyDisplay_8031C5E4(kind: i32) -> *mut c_void;
    fn mpCollSetFacingDir(coll: *mut CollData, dir: i32);
    fn lbVector_Normalize(v: *mut Vec3) -> f32;
    fn lbAudioAx_800237A8(sfx_id: i32, vol: i32, pan: i32) -> i32;
    fn HSD_JObjSetFlagsAll(jobj: *mut HSD_JObj, flags: u32);
    fn HSD_JObjClearFlagsAll(jobj: *mut HSD_JObj, flags: u32);
    fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, v: f32);
    fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, v: f32);
    fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, v: f32);
    fn HSD_JObjSetScaleX(jobj: *mut HSD_JObj, v: f32);
    fn HSD_JObjSetScaleY(jobj: *mut HSD_JObj, v: f32);
    fn HSD_JObjSetScaleZ(jobj: *mut HSD_JObj, v: f32);
    fn HSD_GObjObject_80390B0C(gobj: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, o: *mut c_void);
    }
    // Data defined in other TUs.
    pub static mut it_804D6D40: *const melee_types::it_804D6D40_t = core::ptr::null();
    pub static mut p_ftCommonData: *const ftCommonData = core::ptr::null();
}
// END GENERATED EXT

/// `HSD_GObj_JObjKind` (sysdolphin/baselib/gobj.h).
pub const HSD_GObj_JObjKind: u8 = 4;
/// `JOBJ_HIDDEN` (sysdolphin/baselib/jobj.h).
pub const JOBJ_HIDDEN: u32 = 1 << 4;

macro_rules! pred_i32 {
    ($($name:ident => $f:ident;)*) => { $( pub unsafe extern "C" fn $name(g: *mut HSD_GObj) -> i32 { $f(g) as i32 } )* };
}

#[inline]
unsafe fn item_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `ip->xC4_article_data->x4_specialAttributes` as a `T`.
#[inline]
unsafe fn sa<'a, T>(ip: *mut Item) -> &'a T {
    &*((*(*ip).xC4_article_data).x4_specialAttributes as *const T)
}

/// `GET_JOBJ(gobj)`.
#[inline]
unsafe fn jobj_of(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

/// `it_802F13B4` (itcoin.c): show the coin's figure or its stack, depending on pause and camera.
pub unsafe extern "C" fn it_802F13B4(gobj: *mut HSD_GObj, arg1: i32) {
    let ip = item_of(gobj);
    let attr = sa::<itCoinAttributes>(ip);
    let jobj = (*jobj_of(gobj)).child;
    let x14 = (*ip).xDD4_itemVar.coin.x14 as *mut HSD_GObj;

    if (*ip).xDD4_itemVar.coin.x4 != 0 {
        if (ext::gm_GetDbPauseFlag(1) != false && ext::grFigureGet_80219C50(x14) == 0)
            || (ext::Camera_80031144() < attr.x48 && ext::grFigureGet_80219C50(x14) == 0)
        {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN);
            ext::HSD_JObjClearFlagsAll((*jobj).next, JOBJ_HIDDEN);
        } else {
            ext::HSD_JObjSetFlagsAll((*jobj).next, JOBJ_HIDDEN);
            ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN);
        }
    } else if (ext::gm_GetDbPauseFlag(1) != false && ext::Ground_801C1D84() == 0)
        || ext::Camera_80031144() < attr.x48
    {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN);
        ext::HSD_JObjClearFlagsAll((*jobj).next, JOBJ_HIDDEN);
    } else {
        ext::HSD_JObjSetFlagsAll((*jobj).next, JOBJ_HIDDEN);
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN);
    }
    ext::it_8026EECC(gobj, arg1);
}

/// `itCoin_Logic116_Spawned`: clear the coin's variables and go to motion 0.
pub unsafe extern "C" fn itCoin_Logic116_Spawned(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.coin.x4 = 0;
    (*ip).xDD4_itemVar.coin.x0 = 0;
    (*ip).xDD4_itemVar.coin.x10 = 0;
    it_802F1588(gobj);
}

/// `itCoin_Logic116_Destroyed`: release the figure, or award the coin if it was collected.
pub unsafe extern "C" fn itCoin_Logic116_Destroyed(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    if (*ip).xDD4_itemVar.coin.x4 != 0 {
        ext::grFigureGet_80219C34((*ip).xDD4_itemVar.coin.x14 as *mut HSD_GObj);
    } else if (*ip).xDC8_word.flags.x13() != 0 {
        ext::gm_8017E280((*ip).xDD4_itemVar.coin.x0 as u16, 1);
        ext::it_8027CE44(gobj);
    }
    ext::it_2725_Logic9_Destroyed(gobj);
}

/// `it_802F1588`: enter motion 0 keeping the velocity.
pub unsafe extern "C" fn it_802F1588(gobj: *mut HSD_GObj) {
    ext::Item_80268E5C(gobj, 0, ITEM_UNK_0x1 as u32);
}

/// `itCoin_UnkMotion0_Anim` (returns false).
pub unsafe extern "C" fn itCoin_UnkMotion0_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itCoin_UnkMotion0_Phys`: fall, or follow the thrown speed while `x4` is set.
pub unsafe extern "C" fn itCoin_UnkMotion0_Phys(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    if (*ip).xDD4_itemVar.coin.x4 == 0 {
        let attr = &*(*ip).xCC_item_attr;
        it_80272860(&mut *ip, attr.x10_fall_speed, attr.x14_fall_speed_max);
        return;
    }
    let (x8, xc) = ((*ip).xDD4_itemVar.coin.x8, (*ip).xDD4_itemVar.coin.xC);
    it_80272860(&mut *ip, x8, xc);
}

/// `itCoin_UnkMotion0_Coll`: detect the landing into `it_802F1630`.
pub unsafe extern "C" fn itCoin_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026E414(gobj, Some(it_802F1630));
    false
}

/// `itCoin_ResetRotation` (static inline): zero the three rotations.
unsafe fn itCoin_ResetRotation(gobj: *mut HSD_GObj) {
    let jobj = jobj_of(gobj);
    ext::HSD_JObjSetRotationX(jobj, 0.0);
    ext::HSD_JObjSetRotationY(jobj, 0.0);
    ext::HSD_JObjSetRotationZ(jobj, 0.0);
}

/// `it_802F1630`: come to rest on the floor (motion 1).
pub unsafe extern "C" fn it_802F1630(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
    itCoin_ResetRotation(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_UNK_0x1 as u32);
    it_802756d0(&mut *ip);
}

/// `itCoin_UnkMotion1_Anim` (returns false).
pub unsafe extern "C" fn itCoin_UnkMotion1_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itCoin_UnkMotion1_Phys` (empty).
pub unsafe extern "C" fn itCoin_UnkMotion1_Phys(_gobj: *mut HSD_GObj) {}

/// `itCoin_UnkMotion1_Coll`: a hit while resting restarts the bounce.
pub unsafe extern "C" fn itCoin_UnkMotion1_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_802F1588));
    false
}

/// `itCoin_Logic116_PickedUp`: show the coin and go to motion 2.
pub unsafe extern "C" fn itCoin_Logic116_PickedUp(gobj: *mut HSD_GObj) {
    let jobj = jobj_of(gobj);
    let child = (*jobj).child;
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN);
    ext::Item_80268E5C(gobj, 2, ITEM_UNK_0x1 as u32);
}

/// `itCoin_UnkMotion2_Anim` (returns false).
pub unsafe extern "C" fn itCoin_UnkMotion2_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itCoin_UnkMotion2_Phys` (empty).
pub unsafe extern "C" fn itCoin_UnkMotion2_Phys(_gobj: *mut HSD_GObj) {}

/// `itCoin_Logic116_EvtUnk`: `it_8026B894`.
pub unsafe extern "C" fn itCoin_Logic116_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    it_8026b894(&mut *item_of(gobj), ref_gobj);
}

/// `itCoin_Logic116_DmgReceived`: launch the coin from the hit, scaled by the damage and the
/// coin's kind. Returns false.
pub unsafe extern "C" fn itCoin_Logic116_DmgReceived(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    let attr = sa::<itCoinAttributes>(ip);
    let zero: f32 = 0.0;

    it_802762bc(&mut *ip);
    (*ip).xC9C = ((*ip).xC9C as f32 + it_8027cbfc(&*ip, &*ext::it_804D6D40)) as i32;
    it_8027b798(&mut *ip, &mut (*ip).x40_vel, &*ext::p_ftCommonData);
    if (*ip).x40_vel.x == zero && (*ip).x40_vel.y == zero {
        (*ip).x40_vel.x = 0.1 * -(*ip).xCCC_incDamageDirection;
        (*ip).x40_vel.y = 0.1;
    }
    ext::lbVector_Normalize(&mut (*ip).x40_vel);
    if (*ip).pos.x > zero && -1.0 == (*ip).xCCC_incDamageDirection {
        (*ip).x40_vel.x *= -1.0;
    }
    if (*ip).pos.x < zero && 1.0 == (*ip).xCCC_incDamageDirection {
        (*ip).x40_vel.x *= -1.0;
    }
    let dmg = (*ip).xC9C as f32;
    if (*ip).pos.x > zero {
        if (*ip).xCCC_incDamageDirection == -1.0 {
            (*ip).x40_vel.x *= attr.x30 * (attr.x24 + dmg);
        } else {
            (*ip).x40_vel.x *= attr.x2C * (attr.x24 + dmg);
        }
        if (*ip).x40_vel.x <= -attr.x3C {
            (*ip).x40_vel.x = -attr.x3C;
        }
    } else {
        if -1.0 == (*ip).xCCC_incDamageDirection {
            (*ip).x40_vel.x *= attr.x2C * (attr.x24 + dmg);
        } else {
            (*ip).x40_vel.x *= attr.x30 * (attr.x24 + dmg);
        }
        if (*ip).x40_vel.x >= attr.x3C {
            (*ip).x40_vel.x = attr.x3C;
        }
    }
    (*ip).x40_vel.y *= attr.x38 * (attr.x28 + dmg);
    if (*ip).x40_vel.y >= attr.x40 {
        (*ip).x40_vel.y = attr.x40;
    }
    (*ip).x40_vel.z = zero;
    (*ip).xC9C = 0;
    match (*ip).xDD4_itemVar.coin.x4 {
        2 => (*ip).xDD4_itemVar.coin.x8 += attr.x14,
        3 => (*ip).xDD4_itemVar.coin.x8 += attr.x20,
        _ => (*ip).xDD4_itemVar.coin.x8 += attr.x8,
    }
    ext::Item_80268E5C(gobj, 3, ITEM_UNK_0x1 as u32);
    false
}

/// `itCoin_UnkMotion3_Anim` (returns false).
pub unsafe extern "C" fn itCoin_UnkMotion3_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itCoin_UnkMotion3_Phys`: damp the horizontal speed, then fall with the thrown speed.
pub unsafe extern "C" fn itCoin_UnkMotion3_Phys(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attr = sa::<itCoinAttributes>(ip);
    (*ip).x40_vel.x *= attr.x34;
    let (x8, xc) = ((*ip).xDD4_itemVar.coin.x8, (*ip).xDD4_itemVar.coin.xC);
    it_80272860(&mut *ip, x8, xc);
}

/// `itCoin_UnkMotion3_Coll`: land into `it_802F1630`.
pub unsafe extern "C" fn itCoin_UnkMotion3_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_802F1630));
    false
}

/// `itCoin_Logic116_Thrown`: reset the rotation and go to motion 4.
pub unsafe extern "C" fn itCoin_Logic116_Thrown(gobj: *mut HSD_GObj) {
    itCoin_ResetRotation(gobj);
    ext::Item_80268E5C(gobj, 4, ITEM_UNK_0x1 as u32);
}

/// `itCoin_UnkMotion4_Anim` (returns false).
pub unsafe extern "C" fn itCoin_UnkMotion4_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itCoin_UnkMotion4_Phys`: fall.
pub unsafe extern "C" fn itCoin_UnkMotion4_Phys(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attr = &*(*ip).xCC_item_attr;
    it_80272860(&mut *ip, attr.x10_fall_speed, attr.x14_fall_speed_max);
}

/// `itCoin_UnkMotion4_Coll`: land into `it_802F1630`.
pub unsafe extern "C" fn itCoin_UnkMotion4_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026E414(gobj, Some(it_802F1630));
    false
}

/// `itCoin_Logic116_EnteredAir`: face the direction of travel and set the collision facing.
pub unsafe extern "C" fn itCoin_Logic116_EnteredAir(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);

    let item_dir: f32 = if (*ip).x40_vel.x + (*ip).x7C.x + (*ip).x88.x >= 0.0 { 1.0 } else { -1.0 };

    (*ip).facing_dir = item_dir;
    let coll_dir: i32 = if (*ip).facing_dir == -1.0 { -1 } else { 1 };
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, coll_dir);
}

/// `itCoin_UnkMotion5_Anim` (returns false).
pub unsafe extern "C" fn itCoin_UnkMotion5_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itCoin_UnkMotion5_Phys` (empty).
pub unsafe extern "C" fn itCoin_UnkMotion5_Phys(_gobj: *mut HSD_GObj) {}

/// `itCoin_UnkMotion5_Coll`: land into `it_802F1630`, or back to `it_802F1588`.
pub unsafe extern "C" fn itCoin_UnkMotion5_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026E8C4(gobj, Some(it_802F1630), Some(it_802F1588));
    false
}

/// `it_802F2014`: set the item's joint id.
pub unsafe extern "C" fn it_802F2014(gobj: *mut HSD_GObj, arg1: i32) {
    (*item_of(gobj)).x378_itemColl.joint_id_only = arg1;
}

/// `it_802F2020`: the coin's value.
pub unsafe extern "C" fn it_802F2020(gobj: *mut HSD_GObj) -> i32 {
    (*item_of(gobj)).xDD4_itemVar.coin.x0
}

/// `it_802F202C`: the landing sound; stop the horizontal speed and cap the fall.
pub unsafe extern "C" fn it_802F202C(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    ext::lbAudioAx_800237A8(0xAA, 0x7F, 0x40);
    (*ip).x40_vel.x = 0.0;
    if (*ip).x40_vel.y < -1.5 {
        (*ip).x40_vel.y = -1.5;
    }
    it_802756d0(&mut *ip);
}

/// `it_802F2094` (itcoin.c): spawn a coin (`arg3` is the coin kind; 0 is a stack, else a launched
/// coin), owned by the figure `arg0`, with value `arg2`.
pub unsafe extern "C" fn it_802F2094(arg0: *mut HSD_GObj, pos: *mut Vec3, arg2: i32, arg3: i32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_Coin;
    ext::Item_InitSpawnPosition(&mut spawn, pos, false);
    spawn.facing_dir = 0.0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x3C_damage = 0;
    spawn.x0_parent_gobj = null_mut();
    spawn.x4_parent_gobj2 = null_mut();
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;

    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let ip = item_of(gobj);
        let attr = sa::<itCoinAttributes>(ip);
        (*ip).xDD4_itemVar.coin.x0 = arg2;
        (*ip).xDD4_itemVar.coin.x4 = arg3;
        (*ip).xDD4_itemVar.coin.x14 = arg0 as *mut Item_GObj;
        let var_r29: *mut c_void;
        if (*ip).xDD4_itemVar.coin.x4 != 0 {
            it_802756e0(&mut *ip);
            it_8026b3a8(&mut *ip);
            match arg3 {
                2 => {
                    (*ip).xDD4_itemVar.coin.x8 = attr.xC;
                    (*ip).xDD4_itemVar.coin.xC = attr.x10;
                }
                3 => {
                    (*ip).xDD4_itemVar.coin.x8 = attr.x18;
                    (*ip).xDD4_itemVar.coin.xC = attr.x1C;
                }
                _ => {
                    (*ip).xDD4_itemVar.coin.x8 = attr.x0;
                    (*ip).xDD4_itemVar.coin.xC = attr.x4;
                }
            }
            var_r29 = ext::tyDisplay_8031C5E4(arg2);
        } else {
            ext::Item_80267454(gobj);
            it_802756d0(&mut *ip);
            it_8026b390(&mut *ip);
            (*ip).xD5C = 0;
            var_r29 = ext::tyDisplay_8031C5E4(arg2);
        }
        ext::HSD_GObjObject_80390B0C(gobj);
        ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, var_r29);
        ext::it_8027163C(gobj);
        (*ip).scl = ext::Ground_801C0498();
        (*ip).scl *= attr.x44;

        let j = (*gobj).hsd_obj as *mut HSD_JObj;
        ext::HSD_JObjSetScaleX(j, (*ip).scl);
        ext::HSD_JObjSetScaleY(j, (*ip).scl);
        ext::HSD_JObjSetScaleZ(j, (*ip).scl);
    }
    gobj
}

pred_i32! {
    itCoin_UnkMotion0_Anim_p => itCoin_UnkMotion0_Anim;
    itCoin_UnkMotion0_Coll_p => itCoin_UnkMotion0_Coll;
    itCoin_UnkMotion1_Anim_p => itCoin_UnkMotion1_Anim;
    itCoin_UnkMotion1_Coll_p => itCoin_UnkMotion1_Coll;
    itCoin_UnkMotion3_Anim_p => itCoin_UnkMotion3_Anim;
    itCoin_UnkMotion3_Coll_p => itCoin_UnkMotion3_Coll;
    itCoin_UnkMotion4_Anim_p => itCoin_UnkMotion4_Anim;
    itCoin_UnkMotion4_Coll_p => itCoin_UnkMotion4_Coll;
    itCoin_UnkMotion5_Anim_p => itCoin_UnkMotion5_Anim;
    itCoin_UnkMotion5_Coll_p => itCoin_UnkMotion5_Coll;
    itCoin_UnkMotion2_Anim_p => itCoin_UnkMotion2_Anim;
}

/// `ITEM_UNK_0x1` (src/melee/it/forward.h).
pub const ITEM_UNK_0x1: i32 = 1;

/// `it_803F93C8[]` (itcoin.c).
pub static it_803F93C8: [ItemStateTable; 6] = [
    ItemStateTable { anim_id: -1, animated: Some(itCoin_UnkMotion0_Anim_p), physics_updated: Some(itCoin_UnkMotion0_Phys), collided: Some(itCoin_UnkMotion0_Coll_p) },
    ItemStateTable { anim_id: -1, animated: Some(itCoin_UnkMotion1_Anim_p), physics_updated: Some(itCoin_UnkMotion1_Phys), collided: Some(itCoin_UnkMotion1_Coll_p) },
    ItemStateTable { anim_id: -1, animated: Some(itCoin_UnkMotion2_Anim_p), physics_updated: Some(itCoin_UnkMotion2_Phys), collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itCoin_UnkMotion3_Anim_p), physics_updated: Some(itCoin_UnkMotion3_Phys), collided: Some(itCoin_UnkMotion3_Coll_p) },
    ItemStateTable { anim_id: -1, animated: Some(itCoin_UnkMotion4_Anim_p), physics_updated: Some(itCoin_UnkMotion4_Phys), collided: Some(itCoin_UnkMotion4_Coll_p) },
    ItemStateTable { anim_id: -1, animated: Some(itCoin_UnkMotion5_Anim_p), physics_updated: Some(itCoin_UnkMotion5_Phys), collided: Some(itCoin_UnkMotion5_Coll_p) },
];
