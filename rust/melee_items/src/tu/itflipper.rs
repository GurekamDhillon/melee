//! Native port of src/melee/it/kinds/itflipper.c: the Flipper item (spawn on a stage joint, the
//! spin model, throw / settle / active states, hit reactions and the state table
//! `ItemStateTable_Flipper`). Callees in other TUs go through `ext` (hooks; unset panics "TODO").
//! Checked against the C (the whole TU, shipping contraction) by tests/tu_itflipper.rs.
//!
//! Header inlines compiled into the TU: `Item_InitSpawnPosition`, `Item_InitSpawnCommonFields`,
//! `itGetJObjGrandchild`, `HSD_JObjSetRotationX` (jobj.h, with its `HSD_ASSERT`s and the
//! `HSD_JObjSetMtxDirty` macro), `sqrtf` (math_ppc.h: `melee_runtime::gekko::sqrtf`),
//! `MTXDegToRad`, `ABS`, `Item_StopAndEnterState`, `itResetVelocity`.
//!
//! Float math, contracted sites (Gekko `fmadds`: f64 multiply, f64 add, one rounding to f32):
//! `vx*vx + vy*vy` (as `fmadds(vx, vx, vy*vy)`) and `-1.0f * vel.y + it_804D6D28->x60_float`.
//! `ABS(x)` is `x < 0 ? -x : x` (so `ABS(-0.0)` is `-0.0`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::gekko::sqrtf;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn ftLib_800866DC(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftLib_80086960(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_80086960(a0) as _ };
    fn ftLib_80086BEC(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::ftlib::ftLib_80086BEC(a0, a1) };
    fn ftLib_80087284(a0: *mut HSD_GObj) -> i32;
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_8026E0F4(a0: *mut HSD_GObj) => unsafe { crate::tu::itgroundcoll::it_8026E0F4(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272560(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::it_2725::it_80272560(a0, a1) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272674(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::it_2725::it_80272674(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_802728C8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802728C8(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_8027542C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8027542c(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn lbColl_80008440(a0: *mut HitCapsule) => unsafe { melee_runtime::tu::lbcollision::lbColl_80008440(a0) };
    fn lbVector_NormalizeXY(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_NormalizeXY(a0) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut child: *mut HSD_JObj = unsafe { core::mem::zeroed() };
    pub use crate::tu::it_3F14::it_804D6D28;
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `ABS(x)` (Runtime/platform.h).
#[inline]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `MTXDegToRad(a)`.
#[inline]
fn MTXDegToRad(a: f32) -> f32 {
    a * 0.01745329252
}

/// `HSD_ASSERT(line, cond)`: the failing path is never taken by well-formed state.
#[inline]
fn hsd_assert(cond: bool, what: &str) {
    if !cond {
        panic!("HSD_ASSERT failed: {what}");
    }
}

/// `HSD_JObjMtxIsDirty` (jobj.h).
#[inline]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    hsd_assert(!jobj.is_null(), "jobj");
    let mut result = false;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0 {
        result = true;
    }
    result
}

/// `HSD_JObjSetRotationX` (jobj.h) with the `HSD_JObjSetMtxDirty` macro.
#[inline]
unsafe fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, x: f32) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `itGetJObjGrandchild` (it/inlines.h).
#[inline]
unsafe fn itGetJObjGrandchild(gobj: *mut Item_GObj) -> *mut HSD_JObj {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut child = if jobj.is_null() { null_mut() } else { (*jobj).child };
    child = if child.is_null() { null_mut() } else { (*child).child };
    child
}

/// `Item_InitSpawnPosition` (kinds/inlines.h).
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_InitSpawnCommonFields` (kinds/inlines.h).
#[inline]
unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.x = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.z = 0.0;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as u8);
    (*spawn).x40 = 0;
}

/// `itResetVelocity`.
#[inline]
unsafe fn itResetVelocity(ip: *mut Item) {
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
}

/// `Item_StopAndEnterState` (kinds/inlines.h).
#[inline]
unsafe fn Item_StopAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    itResetVelocity(ip);
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as _);
}

#[inline]
unsafe fn attrs_of(ip: *mut Item) -> *mut itFlipper_DatAttrs {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itFlipper_DatAttrs
}

pub static mut ItemStateTable_Flipper: [ItemStateTable; 7] = [
    ItemStateTable { anim_id: -1, animated: Some(itFlipper_Resting_Anim), physics_updated: Some(itFlipper_Resting_Phys), collided: Some(itFlipper_Resting_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itFlipper_Falling_Anim), physics_updated: Some(itFlipper_Falling_Phys), collided: Some(itFlipper_Falling_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itFlipper_Held_Anim), physics_updated: Some(itFlipper_Held_Phys), collided: None },
    ItemStateTable { anim_id: 0, animated: Some(itFlipper_Inflight_Anim), physics_updated: Some(itFlipper_Inflight_Phys), collided: Some(itFlipper_Inflight_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itFlipper_Airborne_Anim), physics_updated: Some(itFlipper_Airborne_Phys), collided: Some(itFlipper_Airborne_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itFlipper_Active_Anim), physics_updated: Some(itFlipper_Spinning_Phys), collided: Some(itFlipper_Spinning_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itFlipper_Spinning_Anim), physics_updated: Some(itFlipper_Spinning_Phys), collided: Some(itFlipper_Spinning_Coll) },
];

pub unsafe extern "C" fn itFlipper_Spawn(jobj: *mut HSD_JObj) -> *mut HSD_GObj {
    melee_core::ct!("itFlipper_Spawn");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut pos: Vec = core::mem::zeroed();
    let mut gobj: *mut Item_GObj = null_mut();
    if !jobj.is_null() {
        ext::lb_8000B1CC(jobj, null_mut(), &mut pos);
        spawn.kind = It_Kind_Flipper;
        Item_InitSpawnPosition(&mut spawn, &mut pos, false);
        Item_InitSpawnCommonFields(&mut spawn, null_mut(), 1.0, false);
        gobj = ext::Item_80268B18(&mut spawn);
        if !gobj.is_null() {
            let ip = get_item(gobj);
            (*ip).xDD4_itemVar.flipper.xDE8_isStageFixed = 1;
            (*ip).xDD4_itemVar.flipper.xDEC_stageAnchor = jobj;
            itFlipper_Settle(gobj);
        }
    }
    gobj
}

pub unsafe extern "C" fn itFlipper_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_Spawned");
    let ip = get_item(gobj);
    (*ip).xDD4_itemVar.flipper.xDD4_flightTimer = 0;
    (*ip).xDD4_itemVar.flipper.xDD8_isSettled = 0;
    (*ip).xDD4_itemVar.flipper.xDDC_hitboxTimer = 0;
    (*ip).xDD4_itemVar.flipper.xDE0_spinAngle = 0.0;
    (*ip).xDD4_itemVar.flipper.xDE4_spinVel = 0.0;
    (*ip).xDD4_itemVar.flipper.xDE8_isStageFixed = 0;
    (*ip).xDD4_itemVar.flipper.xDEC_stageAnchor = null_mut();
    ext::it_802756D0(gobj);
    itFlipper_EnterFalling(gobj);
}

pub unsafe extern "C" fn itFlipper_UpdateSpin(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_UpdateSpin");
    let ip = (*gobj).user_data as *mut Item;
    let attrs = attrs_of(ip);
    let jobj = itGetJObjGrandchild(gobj);

    if (*ip).xDD4_itemVar.flipper.xDE4_spinVel > 3.1415927 {
        (*ip).xDD4_itemVar.flipper.xDE4_spinVel -= 6.2831855;
    }
    if (*ip).xDD4_itemVar.flipper.xDE4_spinVel < -3.1415927 {
        (*ip).xDD4_itemVar.flipper.xDE4_spinVel += 6.2831855;
    }

    {
        let ip2 = ip;
        if abs_c((*ip2).xDD4_itemVar.flipper.xDE4_spinVel) > 0.0 {
            if abs_c((*ip2).xDD4_itemVar.flipper.xDE4_spinVel) < (*attrs).x20_spinDecay {
                (*ip).xDD4_itemVar.flipper.xDE4_spinVel = 0.0;
            } else if (*ip2).xDD4_itemVar.flipper.xDE4_spinVel > 0.0 {
                (*ip2).xDD4_itemVar.flipper.xDE4_spinVel -= (*attrs).x20_spinDecay;
            } else {
                (*ip).xDD4_itemVar.flipper.xDE4_spinVel += (*attrs).x20_spinDecay;
            }
        }

        (*ip).xDD4_itemVar.flipper.xDE0_spinAngle += (*ip2).xDD4_itemVar.flipper.xDE4_spinVel;
        HSD_JObjSetRotationX(jobj, (*ip2).xDD4_itemVar.flipper.xDE0_spinAngle);
    }
}

pub unsafe extern "C" fn itFlipper_AddSpinImpulse(gobj: *mut Item_GObj, pos: *mut Vec, vel: f32) {
    melee_core::ct!("itFlipper_AddSpinImpulse");
    let ip = get_item(gobj);
    let attrs = attrs_of(ip);
    let speed = vel + (*attrs).x1C_baseSpinOnHit;

    if (*pos).x > (*ip).pos.x {
        if (*pos).y > (*ip).pos.y {
            (*ip).xDD4_itemVar.flipper.xDE4_spinVel += speed;
        } else {
            (*ip).xDD4_itemVar.flipper.xDE4_spinVel -= speed;
        }
    } else if (*pos).y > (*ip).pos.y {
        (*ip).xDD4_itemVar.flipper.xDE4_spinVel -= speed;
    } else {
        (*ip).xDD4_itemVar.flipper.xDE4_spinVel += speed;
    }

    if (*ip).xDD4_itemVar.flipper.xDE4_spinVel > (*attrs).x24_maxSpinVel {
        (*ip).xDD4_itemVar.flipper.xDE4_spinVel = (*attrs).x24_maxSpinVel;
    }
    if (*ip).xDD4_itemVar.flipper.xDE4_spinVel < -(*attrs).x24_maxSpinVel {
        (*ip).xDD4_itemVar.flipper.xDE4_spinVel = -(*attrs).x24_maxSpinVel;
    }
}

pub unsafe extern "C" fn itFlipper_SpinSpeedFromFighter(gobj: *mut Item_GObj, fighter: *mut HSD_GObj, pos: *mut Vec, vel: *mut Vec) -> f32 {
    melee_core::ct!("itFlipper_SpinSpeedFromFighter");
    let ip = get_item(gobj);
    let attrs = attrs_of(ip);
    ext::ftLib_800866DC(fighter, pos);
    ext::ftLib_80086BEC(fighter, vel);
    (*attrs).x18_spinMultiplier * sqrtf((*vel).x * (*vel).x + (*vel).y * (*vel).y)
}

pub unsafe extern "C" fn itFlipper_SpinFromFighter_inline(gobj: *mut Item_GObj, vel: *mut Vec, pos: *mut Vec, mut speed: f32) {
    melee_core::ct!("itFlipper_SpinFromFighter_inline");
    let ip = get_item(gobj);
    *pos = (*ip).pos;

    if !(*ip).xCF4_fighterGObjUnk.is_null() {
        let fighter = (*ip).xCF4_fighterGObjUnk;
        if ext::ftLib_80086960(fighter) != 0 {
            speed = itFlipper_SpinSpeedFromFighter(gobj, fighter, pos, vel);
        }
        (*ip).xCF4_fighterGObjUnk = null_mut();
    }
    itFlipper_AddSpinImpulse(gobj, pos, MTXDegToRad(speed));
}

pub unsafe extern "C" fn itFlipper_SpinFromFighter(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_SpinFromFighter");
    let ip = get_item(gobj);
    let mut speed: f32 = 10.0;
    let mut pos: Vec = core::mem::zeroed();
    let mut vel: Vec = core::mem::zeroed();
    pos = (*ip).pos;

    if !(*ip).xCF4_fighterGObjUnk.is_null() {
        let fighter = (*ip).xCF4_fighterGObjUnk;
        if ext::ftLib_80086960(fighter) != 0 {
            speed = itFlipper_SpinSpeedFromFighter(gobj, fighter, &mut pos, &mut vel);
        }
        (*ip).xCF4_fighterGObjUnk = null_mut();
    }
    itFlipper_AddSpinImpulse(gobj, &mut pos, MTXDegToRad(speed));
}

pub unsafe extern "C" fn itFlipper_Repel_inline(gobj: *mut Item_GObj, kind: i32, pos: *mut Vec, vec: *mut Vec) {
    melee_core::ct!("itFlipper_Repel_inline");
    let ip = get_item(gobj);
    (*vec).z = 0.0;
    (*vec).y = 0.0;
    (*vec).x = 0.0;

    if kind == It_Kind_Flipper {
        (*vec).x = (*ip).pos.x - (*pos).x;
        (*vec).y = (*ip).pos.y - (*pos).y;
        ext::lbVector_NormalizeXY(vec);
        (*vec).x *= 0.2;
        (*vec).y *= 0.2;
        (*ip).x40_vel.x += (*vec).x;
        (*ip).x40_vel.y += (*vec).y;
    }
}

pub unsafe extern "C" fn itFlipper_Repel(gobj: *mut Item_GObj, kind: i32, pos: *mut Vec) {
    melee_core::ct!("itFlipper_Repel");
    let mut vec: Vec = core::mem::zeroed();
    itFlipper_Repel_inline(gobj, kind, pos, &mut vec);
}

pub unsafe extern "C" fn itFlipper_EnterResting(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_EnterResting");
    let ip = get_item(gobj);
    Item_StopAndEnterState(gobj, ip, FLIPPER_MS_RESTING as _);
}

pub unsafe extern "C" fn itFlipper_Resting_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Resting_Anim");
    0
}

pub unsafe extern "C" fn itFlipper_Resting_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itFlipper_Resting_Phys");}

pub unsafe extern "C" fn itFlipper_Resting_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Resting_Coll");
    ext::it_8026D62C(gobj, Some(itFlipper_EnterFalling));
    0
}

pub unsafe extern "C" fn itFlipper_EnterFalling(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_EnterFalling");
    ext::Item_80268E5C(gobj, FLIPPER_MS_FALLING as _, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itFlipper_Falling_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Falling_Anim");
    0
}

pub unsafe extern "C" fn itFlipper_Falling_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_Falling_Phys");
    let ip = get_item(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itFlipper_Falling_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Falling_Coll");
    ext::it_8026E15C(gobj, Some(itFlipper_EnterResting));
    0
}

pub unsafe extern "C" fn itFlipper_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_PickedUp");
    ext::Item_80268E5C(gobj, FLIPPER_MS_HELD as _, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itFlipper_Held_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Held_Anim");
    0
}

pub unsafe extern "C" fn itFlipper_Held_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itFlipper_Held_Phys");}

pub unsafe extern "C" fn itFlipper_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_Dropped");
    itFlipper_Thrown(gobj);
}

pub unsafe extern "C" fn itFlipper_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_Thrown");
    let ip = get_item(gobj);
    let attrs = attrs_of(ip);
    if !(*ip).owner.is_null() {
        if ext::ftLib_80087284((*ip).owner) != 0 {
            (*ip).xDD4_itemVar.flipper.xDD4_flightTimer = (*attrs).x4_smashThrowDuration;
        } else {
            (*ip).xDD4_itemVar.flipper.xDD4_flightTimer = (*attrs).x0_throwDuration;
        }
    }
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, FLIPPER_MS_INFLIGHT as _, (ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE) as _);
}

pub unsafe extern "C" fn itFlipper_Inflight_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Inflight_Anim");
    let ip = get_item(gobj);
    (*ip).xDD4_itemVar.flipper.xDD4_flightTimer -= 1;
    if (*ip).xDD4_itemVar.flipper.xDD4_flightTimer <= 0 {
        itFlipper_Settle(gobj);
    }
    0
}

pub unsafe extern "C" fn itFlipper_Inflight_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_Inflight_Phys");
    let ip = get_item(gobj);
    let item_attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*item_attr).x10_fall_speed, (*item_attr).x14_fall_speed_max);
    if (*ip).xDD4_itemVar.flipper.xDD4_flightTimer <= 0xA {
        let t = (*ip).xDD4_itemVar.flipper.xDD4_flightTimer as f32;
        (*ip).x40_vel.x *= t / (1.0 + t);
        let t = (*ip).xDD4_itemVar.flipper.xDD4_flightTimer as f32;
        (*ip).x40_vel.y *= t / (1.0 + t);
    }
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

pub unsafe extern "C" fn itFlipper_Inflight_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Inflight_Coll");
    let ip = get_item(gobj);
    let attrs = attrs_of(ip);
    let coll = ext::it_8026DAA8(gobj);
    if coll != 0 {
        if coll & 0xC != 0 {
            (*ip).x40_vel.x *= -(*attrs).xC_wallBounce;
        }
        (*ip).x40_vel.y *= (*attrs).xC_wallBounce;
        if coll & 2 != 0 {
            (*ip).x40_vel.x *= (*attrs).xC_wallBounce;
        }
        (*ip).x40_vel.y *= -(*attrs).xC_wallBounce;
        if coll & 1 != 0 {
            (*ip).x40_vel.x *= (*attrs).x10_floorBounce;
        }
        (*ip).x40_vel.y *= -(*attrs).x10_floorBounce;
    }
    0
}

pub unsafe extern "C" fn itFlipper_Settle(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_Settle");
    let ip = get_item(gobj);
    let attrs = attrs_of(ip);
    (*ip).xD5C = 0;
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).xDD1_flag.anon_0.set_b1(1);
    (*ip).xDD4_itemVar.flipper.xDD8_isSettled = 1;
    ext::it_80275444(gobj);
    ext::it_802725D4(gobj);
    if (*ip).xDD4_itemVar.flipper.xDE8_isStageFixed != 0 {
        ext::it_8027542C(gobj);
    }
    (*ip).xD44_lifeTimer = (*attrs).x8_lifetime as f32;
    ext::it_8026B3A8(gobj);
    ext::it_802756E0(gobj);
    itFlipper_EnterActive(gobj);
}

pub unsafe extern "C" fn itFlipper_EnterActive(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_EnterActive");
    ext::Item_80268E5C(gobj, FLIPPER_MS_ACTIVE as _, 0x12);
}

pub unsafe extern "C" fn itFlipper_RefreshHitboxes(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_RefreshHitboxes");
    let ip = get_item(gobj);
    ext::it_80272674(gobj, 0);
    ext::it_80272674(gobj, 1);
    ext::lbColl_80008440(&mut (*ip).x5D4_hitboxes[0].hit);
    ext::lbColl_80008440(&mut (*ip).x5D4_hitboxes[1].hit);
    ext::it_802756E0(gobj);
}

pub unsafe extern "C" fn itFlipper_UpdateActive(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_UpdateActive");
    let ip = get_item(gobj);

    itFlipper_UpdateSpin(gobj);

    if abs_c((*ip).x40_vel.x) >= 0.01 {
        (*ip).x40_vel.x *= 0.9;
    } else {
        (*ip).x40_vel.x = 0.0;
    }

    if abs_c((*ip).x40_vel.y) >= 0.01 {
        (*ip).x40_vel.y *= 0.9;
    } else {
        (*ip).x40_vel.y = 0.0;
    }

    {
        let timer = (*ip).xDD4_itemVar.flipper.xDDC_hitboxTimer;
        if timer > 0 {
            (*ip).xDD4_itemVar.flipper.xDDC_hitboxTimer = timer - 1;
            if (*ip).xDD4_itemVar.flipper.xDDC_hitboxTimer == 0 {
                itFlipper_RefreshHitboxes(gobj);
            }
        }
    }

    if (*ip).xDD4_itemVar.flipper.xDE8_isStageFixed == 0 {
        (*ip).xD44_lifeTimer -= 1.0;
        if (*ip).xD44_lifeTimer <= 0.0 {
            return 1;
        }
        if (*ip).xD44_lifeTimer <= (*ext::it_804D6D28).x34 as f32 {
            ext::it_802728C8(gobj);
        }
    }
    0
}

pub unsafe extern "C" fn itFlipper_Active_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Active_Anim");
    let ip = get_item(gobj);
    if (*ip).xDD4_itemVar.flipper.xDE4_spinVel != 0.0 {
        ext::Item_80268E5C(gobj, FLIPPER_MS_SPINNING as _, 0x12);
    }
    itFlipper_UpdateActive(gobj)
}

pub unsafe extern "C" fn itFlipper_Spinning_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Spinning_Anim");
    let ip = get_item(gobj);
    if (*ip).xDD4_itemVar.flipper.xDE4_spinVel == 0.0 {
        ext::Item_80268E5C(gobj, FLIPPER_MS_ACTIVE as _, 0x12);
    }
    itFlipper_UpdateActive(gobj)
}

pub unsafe extern "C" fn itFlipper_Spinning_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_Spinning_Phys");
    let ip = get_item(gobj);
    if (*ip).xDD4_itemVar.flipper.xDE8_isStageFixed != 0 && !(*ip).xDD4_itemVar.flipper.xDEC_stageAnchor.is_null() {
        ext::lb_8000B1CC((*ip).xDD4_itemVar.flipper.xDEC_stageAnchor, null_mut(), &mut (*ip).pos);
    }
}

pub unsafe extern "C" fn itFlipper_Spinning_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Spinning_Coll");
    let ip = get_item(gobj);
    if (*ip).xDD4_itemVar.flipper.xDE8_isStageFixed == 0 {
        ext::it_8026E0F4(gobj);
    }
    0
}

pub unsafe extern "C" fn itFlipper_DmgDealt(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_DmgDealt");
    let ip = get_item(gobj);
    let attrs = attrs_of(ip);
    let mut pos: Vec = core::mem::zeroed();
    let mut pos2: Vec = core::mem::zeroed();
    let mut vel: Vec = core::mem::zeroed();
    if (*ip).xDD4_itemVar.flipper.xDD8_isSettled == 0 {
        if (*ip).xDD4_itemVar.flipper.xDD4_flightTimer >= 6 {
            ext::itColl_BounceOffVictim(gobj);
        } else {
            (*ip).x40_vel.x = -1.0 * (*ip).x40_vel.x;
            (*ip).x40_vel.y = fmadds(-1.0, (*ip).x40_vel.y, (*ext::it_804D6D28).x60_float);
        }
    } else {
        (*ip).xDD4_itemVar.flipper.xDDC_hitboxTimer = (*attrs).x14_hitboxInterval;
        itFlipper_SpinFromFighter_inline(gobj, &mut vel, &mut pos, 10.0);
        ext::it_80272560(gobj, 0);
        ext::it_80272560(gobj, 1);
        ext::it_802756D0(gobj);
        if (*ip).xDD4_itemVar.flipper.xDE8_isStageFixed == 0 {
            itFlipper_Repel_inline(gobj, (*ip).xC38, &mut (*ip).xCD4, &mut pos2);
        }
    }
    0
}

pub unsafe extern "C" fn itFlipper_Clanked(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Clanked");
    let ip = (*gobj).user_data as *mut Item;
    let attrs = attrs_of(ip);
    if (*ip).xDD4_itemVar.flipper.xDD8_isSettled == 0 {
        if (*ip).xDD4_itemVar.flipper.xDD4_flightTimer >= 6 {
            ext::itColl_BounceOffVictim(gobj);
        } else {
            (*ip).x40_vel.x = -1.0 * (*ip).x40_vel.x;
            (*ip).x40_vel.y = fmadds(-1.0, (*ip).x40_vel.y, (*ext::it_804D6D28).x60_float);
        }
    } else {
        (*ip).xDD4_itemVar.flipper.xDDC_hitboxTimer = (*attrs).x14_hitboxInterval;
        itFlipper_SpinFromFighter(gobj);
        ext::it_80272560(gobj, 0);
        ext::it_80272560(gobj, 1);
        ext::it_802756D0(gobj);
        if (*ip).xDD4_itemVar.flipper.xDE8_isStageFixed == 0 {
            itFlipper_Repel(gobj, (*ip).xC38, &mut (*ip).xCD4);
        }
    }
    0
}

pub unsafe extern "C" fn itFlipper_HitShield(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_HitShield");
    let ip = (*gobj).user_data as *mut Item;
    let attrs = attrs_of(ip);
    if (*ip).xDD4_itemVar.flipper.xDD8_isSettled == 0 {
        if (*ip).xDD4_itemVar.flipper.xDD4_flightTimer >= 6 {
            ext::itColl_BounceOffVictim(gobj);
        } else {
            (*ip).x40_vel.x = -1.0 * (*ip).x40_vel.x;
            (*ip).x40_vel.y = fmadds(-1.0, (*ip).x40_vel.y, (*ext::it_804D6D28).x60_float);
        }
    } else {
        (*ip).xDD4_itemVar.flipper.xDDC_hitboxTimer = (*attrs).x14_hitboxInterval;
        itFlipper_SpinFromFighter(gobj);
        ext::it_80272560(gobj, 0);
        ext::it_80272560(gobj, 1);
        ext::it_802756D0(gobj);
        if (*ip).xDD4_itemVar.flipper.xDE8_isStageFixed == 0 {
            itFlipper_Repel(gobj, (*ip).xC38, &mut (*ip).xCD4);
        }
    }
    0
}

pub unsafe extern "C" fn itFlipper_Reflected(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Reflected");
    ext::it_80273030(gobj)
}

pub unsafe extern "C" fn itFlipper_ShieldBounced(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_ShieldBounced");
    ext::itColl_BounceOffShield(gobj)
}

pub unsafe extern "C" fn itFlipper_SpinFromFighterRecv(gobj: *mut Item_GObj, vel: *mut Vec, pos: *mut Vec, mut speed: f32) {
    melee_core::ct!("itFlipper_SpinFromFighterRecv");
    let ip = get_item(gobj);
    *pos = (*ip).pos;

    if !(*ip).xCEC_fighterGObj.is_null() {
        let fighter = (*ip).xCEC_fighterGObj;
        if ext::ftLib_80086960(fighter) != 0 {
            speed = itFlipper_SpinSpeedFromFighter(gobj, fighter, pos, vel);
        }
        (*ip).xCEC_fighterGObj = null_mut();
    }
    itFlipper_AddSpinImpulse(gobj, pos, MTXDegToRad(speed));
}

pub unsafe extern "C" fn itFlipper_DmgReceived(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_DmgReceived");
    let ip = get_item(gobj);
    let attrs = attrs_of(ip);
    let mut pos: Vec = core::mem::zeroed();
    let mut vec: Vec = core::mem::zeroed();
    let mut vel: Vec = core::mem::zeroed();
    if (*ip).xDD4_itemVar.flipper.xDD8_isSettled != 0 {
        (*ip).xDD4_itemVar.flipper.xDDC_hitboxTimer = (*attrs).x14_hitboxInterval;
        itFlipper_SpinFromFighterRecv(gobj, &mut vel, &mut pos, 10.0);
        ext::it_80272560(gobj, 0);
        ext::it_80272560(gobj, 1);
        ext::it_802756D0(gobj);
        if (*ip).xDD4_itemVar.flipper.xDE8_isStageFixed == 0 {
            itFlipper_Repel_inline(gobj, (*ip).xCB4, &mut (*ip).xCE0, &mut vec);
        }
    }
    0
}

pub unsafe extern "C" fn itFlipper_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_EnteredAir");
    ext::Item_80268E5C(gobj, FLIPPER_MS_AIRBORNE as _, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itFlipper_Airborne_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Airborne_Anim");
    0
}

pub unsafe extern "C" fn itFlipper_Airborne_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itFlipper_Airborne_Phys");}

pub unsafe extern "C" fn itFlipper_Airborne_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFlipper_Airborne_Coll");
    ext::it_8026E8C4(gobj, Some(itFlipper_EnterResting), Some(itFlipper_EnterFalling));
    0
}

pub unsafe extern "C" fn itFlipper_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itFlipper_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
