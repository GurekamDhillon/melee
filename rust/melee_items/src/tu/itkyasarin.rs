//! Native port of src/melee/it/kinds/itkyasarin.c: the Kyasarin (Inishie ruins "kyasarin" enemy) item:
//! spawn, destroy, the walk / turn / wait / throw-egg states, the fall / landing states and the
//! on-screen-time check `it_802EDDC0`, plus the state table `it_803F90D0`. Callees in other TUs go
//! through `ext` (hooks; unset panics "TODO"). Checked against the C (the whole TU, shipping
//! contraction) by tests/tu_itkyasarin.rs.
//!
//! Header inlines compiled into the TU: `HSD_JObjMtxIsDirty` / `HSD_JObjSetRotationY` /
//! `HSD_JObjSetRotationXWithMtxDirty` (jobj.h, with their `HSD_ASSERT`s), `GET_ITEM` / `GET_JOBJ`.
//!
//! Float math: the contracted site is `spawn_pos.x += attr->x38 * ip->facing_dir` in
//! `itKyasarin_UnkMotion7_Anim` (Gekko `fmadds`); `M_PI * facing` / `2 * M_PI * x30` are f64 products
//! rounded to f32 on the call, as in the C.
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
    fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_Randf() -> f32;
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn grInishie2_801FD448(a0: *mut HSD_GObj) => unsafe { melee_world::tu::grinishie2::grInishie2_801FD448(a0) };
    fn grInishie2_801FD4CC(a0: *mut HSD_GObj) => unsafe { melee_world::tu::grinishie2::grInishie2_801FD4CC(a0) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80276100(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::itmaplib::it_80276100(a0, a1 as _) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzako::it_8027B5B0(a0, a1 as _, a2, a3 as _, a4) };
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027b730(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B964(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itzako::it_8027B964(a0, a1) };
    fn it_8027BA54(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027BA54(a0, a1 as _) };
    fn it_8027C794(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itfwd8::it_8027c794(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_8027C79C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzako::it_8027C79C(a0) as _ };
    fn it_802EFA44(a0: *mut HSD_GObj, a1: *mut Vec, a2: f32) => unsafe { crate::tu::itkyasarinegg::it_802EFA44(a0 as _, a1 as _, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirty(a0) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D40: *mut it_804D6D40_t = unsafe { core::mem::zeroed() };
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

const M_PI: f64 = 3.14159265358979323846;

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

#[inline]
unsafe fn get_jobj(g: *mut HSD_GObj) -> *mut HSD_JObj {
    (*g).hsd_obj as *mut HSD_JObj
}

#[inline]
unsafe fn attrs(ip: *mut Item) -> *mut itKyasarinAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itKyasarinAttributes
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
unsafe fn HSD_JObjSetMtxDirtyMacro(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `HSD_JObjSetRotationY` (jobj.h).
#[inline]
unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirtyMacro(jobj);
    }
}

/// `HSD_JObjSetRotationXWithMtxDirty` (jobj.h): calls the out-of-line `HSD_JObjSetMtxDirty`.
#[inline]
unsafe fn HSD_JObjSetRotationXWithMtxDirty(jobj: *mut HSD_JObj, x: f32) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        ext::HSD_JObjSetMtxDirty(jobj);
    }
}

pub static mut it_803F90D0: [ItemStateTable; 11] = [
    ItemStateTable { anim_id: 0, animated: Some(itKyasarin_UnkMotion0_Anim), physics_updated: None, collided: Some(itKyasarin_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itKyasarin_UnkMotion1_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 0, animated: Some(itKyasarin_UnkMotion2_Anim), physics_updated: None, collided: Some(itKyasarin_UnkMotion2_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itKyasarin_UnkMotion3_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 0, animated: Some(itKyasarin_UnkMotion4_Anim), physics_updated: None, collided: Some(itKyasarin_UnkMotion4_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itKyasarin_UnkMotion5_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 1, animated: Some(itKyasarin_UnkMotion6_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 2, animated: Some(itKyasarin_UnkMotion7_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 3, animated: Some(itKyasarin_UnkMotion8_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 3, animated: Some(itKyasarin_UnkMotion9_Anim), physics_updated: Some(itKyasarin_UnkMotion9_Phys), collided: Some(itKyasarin_UnkMotion9_Coll) },
    ItemStateTable { anim_id: 3, animated: Some(itKyasarin_UnkMotion10_Anim), physics_updated: Some(itKyasarin_UnkMotion10_Phys), collided: Some(itKyasarin_UnkMotion10_Coll) },
];

pub unsafe extern "C" fn itKyasarin_Logic25_Destroyed(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    if !(*ip).xDD4_itemVar.kyasarin.x20.is_null() {
        ext::grInishie2_801FD448((*ip).xDD4_itemVar.kyasarin.x20);
    }
    (*ip).xDD4_itemVar.kyasarin.x20 = null_mut();
}

pub unsafe extern "C" fn it_802ECD1C(gobj: *mut Item_GObj, v: *mut Vec3) {
    *v = (*get_item(gobj)).pos;
}

pub unsafe extern "C" fn it_802ECD3C(owner: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32) -> *mut HSD_GObj {
    let item_gobj = ext::it_8027B5B0(It_Kind_Kyasarin, pos, null_mut(), null_mut(), 1);
    if !item_gobj.is_null() {
        let ip = get_item(item_gobj);
        let jobj = get_jobj(item_gobj);
        (*ip).xDD4_itemVar.kyasarin.x20 = owner;
        (*ip).xDD4_itemVar.kyasarin.x38 = 0;
        (*ip).facing_dir = facing_dir;
        (*ip).xDD4_itemVar.kyasarin.x30 = facing_dir;
        if (*ip).facing_dir == 1.0 {
            HSD_JObjSetRotationY(jobj, (M_PI * (*ip).facing_dir as f64) as f32);
        }
        (*ip).xDCC_flag.set_b3(0);
        ext::it_8026B3A8(item_gobj);
        it_802ECEB0(item_gobj);
    }
    item_gobj
}

pub unsafe extern "C" fn it_802ECE90(gobj: *mut Item_GObj) {
    ext::it_8027B730(gobj);
}

pub unsafe extern "C" fn it_802ECEB0(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    let attr = attrs(ip);
    (*ip).x40_vel.x = (*attr).x4 * (*ip).facing_dir;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itKyasarin_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    if ext::it_80272C6C(gobj) == 0 {
        let ip = get_item(gobj);
        let attr = attrs(ip);
        (*ip).x40_vel.x = (*attr).x4 * (*ip).facing_dir;
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.y = 0.0;
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as _);
    }
    0
}

pub unsafe extern "C" fn itKyasarin_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    let attr = attrs(ip);
    if (*ip).facing_dir == 1.0 {
        if (*ip).pos.x >= (*attr).x2C {
            it_802ECFE0(gobj);
        }
    } else if (*ip).pos.x <= (*attr).x28 {
        it_802ECFE0(gobj);
    }
    0
}

pub unsafe extern "C" fn it_802ECFE0(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    let attr = attrs(ip);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
    ext::it_80273454(gobj);
    (*ip).xDD4_itemVar.kyasarin.x24 = (*attr).x8;
}

pub unsafe extern "C" fn itKyasarin_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    (*ip).xDD4_itemVar.kyasarin.x24 = (*ip).xDD4_itemVar.kyasarin.x24 - 1.0;
    if (*ip).xDD4_itemVar.kyasarin.x24 < 0.0 {
        it_802ED0D0(gobj);
    }
    0
}

pub unsafe extern "C" fn itKyasarin_UnkMotion3_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    (*ip).xDD4_itemVar.kyasarin.x24 = (*ip).xDD4_itemVar.kyasarin.x24 - 1.0;
    if (*ip).xDD4_itemVar.kyasarin.x24 < 0.0 {
        it_802ED25C(gobj);
    }
    0
}

pub unsafe extern "C" fn it_802ED0D0(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    let attr = attrs(ip);
    (*ip).x40_vel.x = -(*attr).x4 * (*ip).facing_dir;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itKyasarin_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    if ext::it_80272C6C(gobj) == 0 {
        let ip = get_item(gobj);
        let attr = attrs(ip);
        (*ip).x40_vel.x = -(*attr).x4 * (*ip).facing_dir;
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.y = 0.0;
        ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as _);
    }
    0
}

pub unsafe extern "C" fn itKyasarin_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    let attr = attrs(ip);
    if (*ip).facing_dir == 1.0 {
        if (*ip).pos.x <= (*attr).x34 {
            ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as _);
            ext::it_80273454(gobj);
            (*ip).xDD4_itemVar.kyasarin.x24 = (*attr).xC;
        }
    } else if (*ip).pos.x >= (*attr).x30 {
        ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as _);
        ext::it_80273454(gobj);
        (*ip).xDD4_itemVar.kyasarin.x24 = (*attr).xC;
    }
    0
}

pub unsafe extern "C" fn it_802ED25C(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    let attr = attrs(ip);
    (*ip).x40_vel.x = (*attr).x4 * (*ip).facing_dir;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itKyasarin_UnkMotion4_Anim(gobj: *mut Item_GObj) -> i32 {
    if ext::it_80272C6C(gobj) == 0 {
        let ip = get_item(gobj);
        let attr = attrs(ip);
        (*ip).x40_vel.x = (*attr).x4 * (*ip).facing_dir;
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.y = 0.0;
        ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as _);
    }
    0
}

#[inline]
pub unsafe fn itKyasarin_Randi(range: i32) -> i32 {
    ext::HSD_Randi(range)
}

/// The body shared by both facing branches of `itKyasarin_UnkMotion4_Coll`.
#[inline]
unsafe fn motion4_coll_hit(gobj: *mut Item_GObj, ip: *mut Item, attr: *mut itKyasarinAttributes) {
    let mut rand = itKyasarin_Randi((*attr).x14 as i32);
    (*ip).xDD4_itemVar.kyasarin.x2C = (*attr).x10 + rand as f32;
    rand = itKyasarin_Randi((*attr).x1C as i32);
    (*ip).xDD4_itemVar.kyasarin.x28 = (*attr).x18 + rand as f32;
    ext::it_80273454(gobj);
    if ext::HSD_Randi(2) != 0 {
        (*ip).xDD4_itemVar.kyasarin.x34 = (*attr).x48;
    } else {
        (*ip).xDD4_itemVar.kyasarin.x34 = 0;
    }
    it_802ED4F8(gobj);
}

pub unsafe extern "C" fn itKyasarin_UnkMotion4_Coll(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    let attr = attrs(ip);
    if (*ip).facing_dir == 1.0 {
        if (*ip).pos.x >= (*attr).x2C {
            motion4_coll_hit(gobj, ip, attr);
        }
    } else if (*ip).pos.x <= (*attr).x28 {
        motion4_coll_hit(gobj, ip, attr);
    }
    0
}

pub unsafe extern "C" fn it_802ED4F8(gobj: *mut HSD_GObj) {
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE as _);
}

#[inline]
pub unsafe fn itKyasarin_TurnAround(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    let jobj = get_jobj(gobj);
    let attr = attrs(ip);

    it_802ED8BC(gobj);
    if (*ip).xDD4_itemVar.kyasarin.x30 == -1.0 {
        HSD_JObjSetRotationY(jobj, (M_PI * (*ip).xDD4_itemVar.kyasarin.x30 as f64) as f32);
    } else {
        HSD_JObjSetRotationY(jobj, (2.0 * M_PI * (*ip).xDD4_itemVar.kyasarin.x30 as f64) as f32);
    }
    (*ip).facing_dir = -(*ip).xDD4_itemVar.kyasarin.x30;
    (*ip).xDCC_flag.set_b3(1);
    (*ip).x40_vel.x = (*ip).x5D0_animFrameSpeed * ((*attr).x4 * (*ip).facing_dir);
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
}

pub unsafe extern "C" fn itKyasarin_UnkMotion6_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    (*ip).xDD4_itemVar.kyasarin.x2C -= 1.0;
    (*ip).xDD4_itemVar.kyasarin.x28 -= 1.0;
    if (*ip).xDD4_itemVar.kyasarin.x2C < 0.0 {
        itKyasarin_TurnAround(gobj);
    } else if (*ip).xDD4_itemVar.kyasarin.x28 < 0.0 {
        it_802ED774(gobj);
    } else if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE as _);
    }
    0
}

pub unsafe extern "C" fn it_802ED774(gobj: *mut HSD_GObj) {
    ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itKyasarin_UnkMotion7_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        let attr = attrs(ip);
        let mut spawn_pos: Vec3 = (*ip).pos;
        spawn_pos.x = fmadds((*attr).x38, (*ip).facing_dir, spawn_pos.x);
        spawn_pos.y += (*attr).x3C;
        spawn_pos.z = 0.0;
        ext::it_802EFA44(gobj, &mut spawn_pos, (*ip).facing_dir);
        (*ip).xDD4_itemVar.kyasarin.x34 -= 1;
        if (*ip).xDD4_itemVar.kyasarin.x34 <= 0 {
            let r = ext::HSD_Randi((*attr).x1C as i32);
            (*ip).xDD4_itemVar.kyasarin.x28 = (*attr).x18 + r as f32;
            ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE as _);
        } else {
            ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE as _);
        }
    }
    0
}

pub unsafe extern "C" fn it_802ED8BC(gobj: *mut HSD_GObj) {
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itKyasarin_UnkMotion5_Anim(gobj: *mut Item_GObj) -> i32 {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as _);
    }
    0
}

#[inline]
pub unsafe fn itKyasarin_UnkMotion8_case3(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    let attr = attrs(ip);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as _);
    ext::it_80273454(gobj);
    (*ip).xDD4_itemVar.kyasarin.x24 = (*attr).xC;
}

#[inline]
pub unsafe fn itKyasarin_UnkMotion8_case8(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    (*ip).xDD4_itemVar.kyasarin.x3C = (*ip).msid as i32;
    ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE as _);
    ext::it_80273454(gobj);
    ext::it_802756D0(gobj);
}

pub unsafe extern "C" fn itKyasarin_UnkMotion8_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        match (*ip).xDD4_itemVar.kyasarin.x3C {
            0 => it_802ECEB0(gobj),
            1 => it_802ECFE0(gobj),
            2 => it_802ED0D0(gobj),
            3 => itKyasarin_UnkMotion8_case3(gobj),
            4 => it_802ED25C(gobj),
            5 => itKyasarin_TurnAround(gobj),
            6 => it_802ED4F8(gobj),
            7 => it_802ED774(gobj),
            8 => itKyasarin_UnkMotion8_case8(gobj),
            _ => {}
        }
        ext::it_802756E0(gobj);
    }
    0
}

pub unsafe extern "C" fn itKyasarin_UnkMotion9_Anim(gobj: *mut Item_GObj) -> i32 {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE as _);
    }
    0
}

pub unsafe extern "C" fn itKyasarin_UnkMotion9_Phys(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
}

pub unsafe extern "C" fn itKyasarin_UnkMotion9_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8027C794(gobj)
}

pub unsafe extern "C" fn itKyasarin_UnkMotion10_Anim(gobj: *mut Item_GObj) -> i32 {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0xA, ITEM_ANIM_UPDATE as _);
    }
    0
}

pub unsafe extern "C" fn itKyasarin_UnkMotion10_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itKyasarin_UnkMotion10_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8027C79C(gobj)
}

#[inline]
pub unsafe fn itKyasarin_GetCollData(ip: *mut Item) -> *mut CollData {
    &raw mut (*ip).x378_itemColl
}

#[inline]
pub unsafe fn itKyasarin_SetFallCollPos(ip: *mut Item, coll: *mut CollData, pos: *mut Vec3) {
    (*coll).cur_pos = (*ip).pos;
    *pos = (*ip).pos;
}

#[inline]
pub unsafe fn itKyasarin_FlipAndFall(gobj: *mut Item_GObj, jobj: *mut HSD_JObj, ip: *mut Item, pos: *mut Vec3) {
    let coll: *mut CollData = itKyasarin_GetCollData(ip);

    HSD_JObjSetRotationXWithMtxDirty(jobj, M_PI as f32);
    ext::it_802762BC(ip);
    itKyasarin_SetFallCollPos(ip, coll, pos);
    ext::it_80276100(gobj, pos);
    ext::it_802756D0(gobj);
}

#[inline]
pub unsafe fn itKyasarin_SaveStateAndStop(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    (*ip).xDD4_itemVar.kyasarin.x3C = (*ip).msid as i32;
    ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE as _);
    ext::it_80273454(gobj);
    ext::it_802756D0(gobj);
}

pub unsafe extern "C" fn it_802EDDC0(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    let attr = attrs(ip);

    (*ip).xDD4_itemVar.kyasarin.x38 += (*ip).xCA0;
    if (*ip).xDD4_itemVar.kyasarin.x38 < (*attr).x40 {
        itKyasarin_SaveStateAndStop(gobj);
    } else if (*ip).xDD4_itemVar.kyasarin.x38 < (*attr).x44 {
        (*ip).x5D0_animFrameSpeed = (*attr).x24;
        itKyasarin_TurnAround(gobj);
    } else {
        {
            let ip = get_item(gobj);
            if !(*ip).xDD4_itemVar.kyasarin.x20.is_null() {
                ext::grInishie2_801FD4CC((*ip).xDD4_itemVar.kyasarin.x20);
            }
        }
        (*ip).xDCC_flag.set_b3(1);
        if ext::HSD_Randf() < (*ext::it_804D6D40).x8 {
            let mut pos: Vec3 = core::mem::zeroed();
            let ip = get_item(gobj);
            itKyasarin_FlipAndFall(gobj, get_jobj(gobj), ip, &mut pos);
            ext::it_8027BA54(gobj, &raw mut (*ip).x40_vel);
            ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE as _);
        } else {
            let mut pos: Vec3 = core::mem::zeroed();
            let ip = get_item(gobj);
            itKyasarin_FlipAndFall(gobj, get_jobj(gobj), ip, &mut pos);
            ext::it_8027B964(gobj, 0);
            (*ip).xDD4_itemVar.kyasarin.x18 = 0;
            ext::Item_80268E5C(gobj, 0xA, ITEM_ANIM_UPDATE as _);
        }
    }
    0
}

pub unsafe extern "C" fn it_802EE1E0(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
