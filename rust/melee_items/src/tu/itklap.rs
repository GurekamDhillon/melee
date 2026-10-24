//! Native port of src/melee/it/kinds/itklap.c: the Klap (Kongo Jungle barrel-cannon "klap" enemy
//! item): spawn, destroy, the idle / walk / fall / flip states, damage reaction and the ground-follow
//! accessory, plus the state table `it_803F8940`. Callees in other TUs go through `ext` (hooks;
//! unset panics "TODO"). Checked against the C (the whole TU, shipping contraction) by
//! tests/tu_itklap.rs.
//!
//! Header inlines compiled into the TU: `HSD_JObjGetTranslation2` / `HSD_JObjGetRotation` /
//! `HSD_JObjSetRotation` / `HSD_JObjGetRotationX` / `HSD_JObjSetRotationX` / `HSD_JObjSetTranslate` /
//! `HSD_JObjGetChild` / `HSD_JObjMtxIsDirty` (jobj.h, with their `HSD_ASSERT`s and the
//! `HSD_JObjSetMtxDirty` macro), `GET_ITEM` / `GET_JOBJ`.
//!
//! Float math: the contracted sites are `pos.x += 6.0f * -sinf(angle)` and `pos.y += 6.0f * cosf(angle)`
//! in `it_802E215C` (Gekko `fmadds`, see `melee_core::fp::fmadds`). `pos.y -= 5.0` and the `2 * M_PI`
//! wraps are done in f64 and rounded to f32 on the store, as in the C.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Ground_801C4DA0(a0: *mut Vec, a1: *mut f32) -> i32 => unsafe { melee_world::tu::ground::Ground_801C4DA0(a0 as _, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_80269CA0(a0: *mut Item, a1: i32) => unsafe { crate::item::item_80269ca0(&mut *a0, a1) };
    fn grKongo_801D8058(a0: *mut HSD_GObj) => unsafe { melee_world::tu::grkongo::grKongo_801D8058(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BDB4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BDB4(a0) };
    fn it_80270CD8(a0: *mut Item, a1: *mut HitCapsule) -> f32 => unsafe { crate::tu::itcoll::it_80270CD8(a0, a1) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzako::it_8027B5B0(a0, a1 as _, a2, a3 as _, a4) };
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027b730(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B798(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027B798(a0, a1 as _) as _ };
    fn lbColl_80008D30(a0: *mut HitCapsule, a1: *mut lbColl_80008D30_arg1) => unsafe { melee_runtime::tu::lbcollision::lbColl_80008D30(a0, a1 as _) };
    fn lb_8000B134(a0: *mut HSD_JObj) -> i32 => unsafe { melee_runtime::tu::lb_00B0::lb_8000B134(a0) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

const M_PI: f64 = 3.14159265358979323846;

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
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

/// `HSD_JObjSetMtxDirty(jobj)` (jobj.h macro).
#[inline]
unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `HSD_JObjGetTranslation2` (jobj.h).
#[inline]
pub unsafe fn HSD_JObjGetTranslation2(jobj: *mut HSD_JObj, translate: *mut Vec) {
    hsd_assert(!jobj.is_null(), "jobj");
    *translate = (*jobj).translate;
}

/// `HSD_JObjGetRotation` (jobj.h).
#[inline]
unsafe fn HSD_JObjGetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert(!rotate.is_null(), "rotate");
    *rotate = (*jobj).rotate;
}

/// `HSD_JObjSetRotation` (jobj.h).
#[inline]
unsafe fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert(!rotate.is_null(), "rotate");
    (*jobj).rotate = *rotate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjGetRotationX` (jobj.h).
#[inline]
unsafe fn HSD_JObjGetRotationX(jobj: *mut HSD_JObj) -> f32 {
    hsd_assert(!jobj.is_null(), "jobj");
    (*jobj).rotate.x
}

/// `HSD_JObjSetRotationX` (jobj.h).
#[inline]
unsafe fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, x: f32) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjSetTranslate` (jobj.h).
#[inline]
unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert(!translate.is_null(), "translate");
    (*jobj).translate = *translate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjGetChild` (jobj.h).
#[inline]
unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() { null_mut() } else { (*jobj).child }
}

pub static mut it_803F8940: [ItemStateTable; 5] = [
    ItemStateTable { anim_id: 0, animated: Some(itKlap_UnkMotion1_Anim), physics_updated: Some(itKlap_UnkMotion1_Phys), collided: Some(itKlap_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itKlap_UnkMotion1_Anim), physics_updated: Some(itKlap_UnkMotion1_Phys), collided: Some(itKlap_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itKlap_UnkMotion2_Anim), physics_updated: Some(itKlap_UnkMotion2_Phys), collided: Some(itKlap_UnkMotion2_Coll) },
    ItemStateTable { anim_id: 3, animated: Some(itKlap_UnkMotion3_Anim), physics_updated: Some(itKlap_UnkMotion3_Phys), collided: Some(itKlap_UnkMotion3_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itKlap_UnkMotion4_Anim), physics_updated: Some(itKlap_UnkMotion4_Phys), collided: Some(itKlap_UnkMotion4_Coll) },
];

pub unsafe extern "C" fn it_802E1820(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    ext::it_8027B730(gobj);
    (*ip).facing_dir = 0.0;
    (*ip).xD5C = 0;
    (*ip).xDD4_itemVar.klap.x20 = null_mut();
    it_802E1930(gobj);
}

pub unsafe extern "C" fn itKlap_Logic10_Destroyed(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    if !(*ip).xDD4_itemVar.klap.x20.is_null() {
        ext::grKongo_801D8058((*ip).xDD4_itemVar.klap.x20);
        (*ip).xDD4_itemVar.klap.x20 = null_mut();
    }
}

pub unsafe fn it_802E18B4_inline(gobj: *mut Item_GObj, spawned_gobj: *mut Item_GObj) {
    let ip = get_item(spawned_gobj);
    (*ip).xDD4_itemVar.klap.x20 = gobj;
    ext::it_8026BDB4(spawned_gobj);
}

pub unsafe extern "C" fn it_802E18B4(gobj: *mut Item_GObj) -> *mut Item_GObj {
    let mut v: Vec = core::mem::zeroed();
    let mut spawned_gobj: *mut Item_GObj = null_mut();

    if !gobj.is_null() {
        ext::lb_8000B1CC((*gobj).hsd_obj as *mut HSD_JObj, null_mut(), &mut v);
        spawned_gobj = ext::it_8027B5B0(It_Kind_Klap as _, &mut v, null_mut(), null_mut(), 1);
        if !spawned_gobj.is_null() {
            it_802E18B4_inline(gobj, spawned_gobj);
        }
    }
    spawned_gobj
}

pub unsafe extern "C" fn it_802E1930(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itKlap_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn itKlap_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);

    if ext::lb_8000B134((*gobj).hsd_obj as *mut HSD_JObj) != 0 && ext::HSD_Randi(0x64) < 0x1E {
        if (*ip).msid == 0 {
            it_802E1C4C(gobj);
            return;
        }
        it_802E1930(gobj);
    }
}

pub unsafe extern "C" fn itKlap_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    let mut pos: Vec = core::mem::zeroed();
    let mut quat: Quaternion = core::mem::zeroed();
    let ip = get_item(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    if !(*ip).xDD4_itemVar.klap.x20.is_null() {
        let ref_jobj = (*(*ip).xDD4_itemVar.klap.x20).hsd_obj as *mut HSD_JObj;
        HSD_JObjGetTranslation2(ref_jobj, &mut pos);
        HSD_JObjGetRotation(ref_jobj, &mut quat);
        pos.y = (pos.y as f64 - 5.0) as f32;
        (*ip).pos = pos;
        HSD_JObjSetTranslate(jobj, &mut pos);
        HSD_JObjSetRotation(jobj, &mut quat);
    } else {
        it_802E1C84(gobj);
    }

    let child = HSD_JObjGetChild(jobj);
    if !child.is_null() {
        ext::lb_8000B1CC(child, null_mut(), &mut pos);
        if pos.z >= 0.0 {
            it_802E1C84(gobj);
        }
    }
    0
}

pub unsafe extern "C" fn it_802E1C4C(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn it_802E1C84(gobj: *mut Item_GObj) {
    let mut pos: Vec = core::mem::zeroed();
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let ip = get_item(gobj);

    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xDCC_flag.set_b3(1);

    let child = HSD_JObjGetChild(jobj);
    ext::lb_8000B1CC(child, null_mut(), &mut pos);
    (*ip).pos = pos;
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itKlap_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn itKlap_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
    let mut rot = HSD_JObjGetRotationX(jobj);
    let diff = 1.5707964f32 - rot;
    if diff > 0.034906585f32 {
        rot += 0.034906585f32;
    } else if diff < -0.034906585f32 {
        rot -= 0.034906585f32;
    } else {
        rot = 1.5707964f32;
    }
    HSD_JObjSetRotationX(jobj, rot);
}

pub unsafe extern "C" fn itKlap_UnkMotion2_Coll(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn it_2725_Logic10_DmgReceived(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    (*ip).xDCC_flag.set_b3(1);
    it_802E1E94(gobj);
    0
}

pub unsafe extern "C" fn it_802E1E94(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    ext::it_802762BC(ip);
    if ext::it_8027B798(gobj, &mut (*ip).x40_vel) != 0 {
        ext::it_802762BC(ip);
    }
    let rand_val = ext::HSD_Randf();
    (*ip).xDD4_itemVar.klap.x24 = (rand_val - 0.5f32) * 0.17453292f32;
    let rand_val = ext::HSD_Randf();
    (*ip).xDD4_itemVar.klap.x28 = (rand_val - 0.5f32) * 0.17453292f32;
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itKlap_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn itKlap_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut quat: Quaternion = core::mem::zeroed();
    let tau = 2.0 * M_PI;

    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
    HSD_JObjGetRotation(jobj, &mut quat);
    quat.y += (*ip).xDD4_itemVar.klap.x24;
    if quat.y as f64 > tau {
        quat.y = (quat.y as f64 - tau) as f32;
    } else if (quat.y as f64) < -tau {
        quat.y = (quat.y as f64 + tau) as f32;
    }
    quat.z += (*ip).xDD4_itemVar.klap.x28;
    if quat.z as f64 > tau {
        quat.z = (quat.z as f64 - tau) as f32;
    } else if (quat.z as f64) < -tau {
        quat.z = (quat.z as f64 + tau) as f32;
    }
    HSD_JObjSetRotation(jobj, &mut quat);
}

pub unsafe extern "C" fn itKlap_UnkMotion4_Coll(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn it_802E20D8(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    ext::it_802762BC(ip);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as _);
    ext::it_802756D0(gobj);
    (*ip).on_accessory = Some(it_802E215C);
}

pub unsafe extern "C" fn itKlap_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn itKlap_UnkMotion3_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itKlap_UnkMotion3_Coll(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn it_802E215C(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut angle: f32 = 0.0;
    let mut pos: Vec = core::mem::zeroed();
    let mut quat: Quaternion = core::mem::zeroed();

    ext::Ground_801C4DA0(&mut pos, &mut angle);
    pos.x = fmadds(6.0, -melee_runtime::gekko::sinf(angle), pos.x);
    pos.y = fmadds(6.0, melee_runtime::gekko::cosf(angle), pos.y);
    (*ip).pos = pos;
    HSD_JObjSetTranslate(jobj, &mut (*ip).pos);
    quat.x = 1.5707964;
    quat.z = angle;
    quat.w = 0.0;
    quat.y = 0.0;
    HSD_JObjSetRotation(jobj, &mut quat);
}

pub unsafe fn it_802E2330_inline(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    ext::it_802762BC(ip);
    if ext::it_8027B798(gobj, &mut (*ip).x40_vel) != 0 {
        ext::it_802762BC(ip);
    }
    let rand = ext::HSD_Randf();
    (*ip).xDD4_itemVar.klap.x24 = 0.17453292f32 * (rand - 0.5f32);
    let rand = ext::HSD_Randf();
    (*ip).xDD4_itemVar.klap.x28 = 0.17453292f32 * (rand - 0.5f32);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn it_802E2330(gobj: *mut Item_GObj, pos: *mut Vec, arg2: *mut lbColl_80008D30_arg1, mut angle: f32) {
    let mut hit: SmallerHitCapsule = core::mem::zeroed();
    let ip = get_item(gobj);

    (*ip).pos = *pos;
    ext::lbColl_80008D30(&mut hit as *mut SmallerHitCapsule as *mut HitCapsule, arg2);
    ext::Item_80269CA0(ip, hit.damage as i32);
    if angle < 0.0 {
        angle = (angle as f64 + 360.0) as f32;
    }
    (*ip).xCC8_knockback = ext::it_80270CD8(ip, &mut hit as *mut SmallerHitCapsule as *mut HitCapsule);
    (*ip).xCAC_angle = angle as i32;
    (*ip).xCCC_incDamageDirection = (*ip).facing_dir;
    it_802E2330_inline(gobj);
}

pub unsafe extern "C" fn it_802E2450(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
