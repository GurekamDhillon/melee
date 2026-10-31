//! Native port of src/melee/it/kinds/itheiho.c: the Heiho (Shy Guy) item: spawn, the five motion
//! states (walk, jump, spin) and the blast-zone despawn. Callees in other TUs go through `ext`.
//! Checked against the C (the whole TU, shipping contraction, item/jobj state hashed field by
//! field) by tests/tu_itheiho.rs.
//!
//! Fused sites (fmadds, double-rounded as the C build models them): the gravity assist
//! `-((-0.04f * ABS(vx)) - vy)` = `-fmadds(-0.04, ABS(vx), -vy)` (negated addend, so NaN signs match), `vel.x += attr[5] * rand`, and `0.0174.. * ((8.0f * rand) + 1.0f)`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_AObjSetFlags(a0: *mut HSD_AObj, a1: u32) => unsafe { melee_world::tu_ext::HSD_AObjSetFlags(a0, a1) };
    fn HSD_AObjSetRate(a0: *mut HSD_AObj, a1: f32) => unsafe { melee_runtime::tu::aobj::HSD_AObjSetRate(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Stage_GetBlastZoneBottomOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneBottomOffset() };
    fn Stage_GetBlastZoneLeftOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneLeftOffset() };
    fn Stage_GetBlastZoneRightOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneRightOffset() };
    fn Stage_GetBlastZoneTopOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() };
    fn Stage_UnkSetVec3TCam_Offset(a0: *mut Vec) => unsafe { melee_world::tu::stage::Stage_UnkSetVec3TCam_Offset(a0 as _) };
    fn itFoods_Logic18_PickedUp(a0: *mut HSD_GObj) => unsafe { crate::tu::itfoods::itFoods_Logic18_PickedUp(a0 as _) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_26B1::it_8026B894(a0, a1) as _ };
    fn it_8026DA70(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA70(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80274A64(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274A64(a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::tu::itmaplib::it_802762BC(a0) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzako::it_8027B5B0(a0, a1 as _, a2, a3 as _, a4) };
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027B730(a0) };
    fn it_8027B798(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027B798(a0, a1 as _) as _ };
    fn it_8027CE44(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CE44(a0) };
    fn it_8028F8E4(a0: *mut HSD_GObj) => unsafe { crate::tu::itfreeze::it_8028F8E4(a0 as _) };
    fn it_8028F968(a0: *mut HSD_GObj) => unsafe { crate::tu::itfreeze::it_8028F968(a0 as _) };
    fn it_8028F9B8(a0: *mut HSD_GObj) => unsafe { crate::tu::itfreeze::it_8028F9B8(a0 as _) };
    fn it_8028F9D8(a0: *mut HSD_GObj, a1: *mut Vec, a2: f32) => unsafe { crate::tu::itfoods::it_8028F9D8(a0 as _, a1, a2) };
    fn it_8028FAF4(a0: *mut HSD_GObj, a1: *mut Vec) -> *mut HSD_GObj => unsafe { crate::tu::itfoods::it_8028FAF4(a0 as _, a1) };
    fn it_8028FC5C(a0: *mut HSD_GObj) => unsafe { crate::tu::itfoods::it_8028FC5C(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::jobj_inlines_it!();
crate::inl_jobj_set!();
crate::inl_itUpdateVelocityFromBone!();

/// `ABS(x)`: `x < 0 ? -x : x` (keeps `-0.0` and NaN as they are).
#[inline]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// The article's special attributes as a float array (`f32* attr`).
#[inline]
unsafe fn fattr(ip: *mut Item) -> *mut f32 {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut f32
}

/// `ItemStateTable it_803F83F0[5]`
pub static it_803F83F0: [ItemStateTable; 5] = [
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHeiho_UnkMotion0_Anim), physics_updated: Some(itHeiho_UnkMotion0_Phys), collided: Some(itHeiho_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itHeiho_UnkMotion1_Anim), physics_updated: Some(itHeiho_UnkMotion1_Phys), collided: Some(itHeiho_UnkMotion1_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHeiho_UnkMotion2_Anim), physics_updated: Some(itHeiho_UnkMotion2_Phys), collided: Some(itHeiho_UnkMotion2_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHeiho_UnkMotion3_Anim), physics_updated: Some(itHeiho_UnkMotion3_Phys), collided: Some(itHeiho_UnkMotion3_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itHeiho_UnkMotion4_Anim), physics_updated: Some(itHeiho_UnkMotion4_Phys), collided: Some(itHeiho_UnkMotion4_Coll) },
];

pub unsafe extern "C" fn it_802D8618(arg0: i32, arg1: *mut Vec3, arg2: i32, arg3: i32) {
    melee_core::ct!("it_802D8618");
    let gobj = ext::it_8027B5B0(It_Kind_Heiho as _, arg1, null_mut(), null_mut(), 1);
    if gobj.is_null() {
        return;
    }
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.heiho.x20 = arg0 as i8;
    (*ip).xDD4_itemVar.heiho.x21 = arg2 as i8;
    (*ip).xDD4_itemVar.heiho.x24 = arg3;
    it_802D8894(gobj);
}

pub unsafe extern "C" fn it_802D8688(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D8688");
    let ip = GET_ITEM(gobj);
    ext::it_8027B730(gobj);
    (*ip).xD5C = 0;
    (*ip).xDCC_flag.set_b3(0);
    (*ip).xDD4_itemVar.heiho.x54 = ext::it_8028FAF4(null_mut(), &raw mut (*ip).pos);
    if !(*ip).xDD4_itemVar.heiho.x54.is_null() {
        ext::it_8028F968((*ip).xDD4_itemVar.heiho.x54);
        ext::it_8028F9D8((*ip).xDD4_itemVar.heiho.x54, &raw mut (*ip).pos, 1.0);
        ext::itFoods_Logic18_PickedUp((*ip).xDD4_itemVar.heiho.x54);
    }
    (*ip).xDD4_itemVar.heiho.x2C = 0;
    (*ip).xDD4_itemVar.heiho.x22 = 0;
    (*ip).pos.z = 2.0;
    ext::it_802762BC(ip);
    if (*ip).pos.x < 0.0 {
        (*ip).facing_dir = 1.0;
        HSD_JObjSetRotationY(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 0.0);
    } else {
        (*ip).facing_dir = -1.0;
        HSD_JObjSetRotationY(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 3.1415927);
    }
    (*ip).init_facing_dir = 1.0;
    it_802D8894(gobj);
}

pub unsafe extern "C" fn it_802D8894(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D8894");
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itHeiho_UnkMotion0_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeiho_UnkMotion0_Anim");
    0
}

pub unsafe extern "C" fn itHeiho_UnkMotion0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeiho_UnkMotion0_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.heiho.x24 == 0 {
        it_802D8918(gobj);
        return;
    }
    (*ip).xDD4_itemVar.heiho.x24 = (*ip).xDD4_itemVar.heiho.x24.wrapping_sub(1);
}

pub unsafe extern "C" fn itHeiho_UnkMotion0_Coll(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeiho_UnkMotion0_Coll");
    0
}

pub unsafe extern "C" fn it_802D8918(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D8918");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.heiho.x24 = 0;
    it_802D98AC(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    it_802D98C4((*(*ip).xBBC_dynamicBoneTable).bones[1], ip);
    (*ip).on_accessory = Some(it_802D96B0);
}

/// `static inline itHeiho_UnkMotion1_Anim_inline(Item_GObj*, int i)`
pub unsafe fn itHeiho_UnkMotion1_Anim_inline(gobj: *mut HSD_GObj, i: i32) {
    melee_core::ct!("itHeiho_UnkMotion1_Anim_inline");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.heiho.x24 = 0;
    ext::Item_80268E5C(gobj, i, ITEM_ANIM_UPDATE as u32);
    it_802D98C4((*(*ip).xBBC_dynamicBoneTable).bones[1], ip);
    (*ip).on_accessory = Some(it_802D96B0);
}

pub unsafe extern "C" fn itHeiho_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeiho_UnkMotion1_Anim");
    let ip = GET_ITEM(gobj);
    let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    let child = HSD_JObjGetChild(jobj);
    it_802D98C4((*(*ip).xBBC_dynamicBoneTable).bones[1], ip);
    if ext::it_80272C6C(gobj) == 0 {
        itHeiho_UnkMotion1_Anim_inline(gobj, 1);
    }
    if (*ip).xDD4_itemVar.heiho.x2C > 0x3C0 {
        ext::HSD_AObjSetFlags((*child).aobj, 0x10000000);
    }
    0
}

pub unsafe extern "C" fn itHeiho_UnkMotion1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeiho_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let attr = fattr(ip);
    (*ip).x40_vel.x = (*ip).facing_dir * *attr.offset((*ip).xDD4_itemVar.heiho.x21 as isize + 1);
    if (*ip).xDD4_itemVar.heiho.x2C > 960 {
        (*ip).x40_vel.y = -fmadds(-0.04, abs_c((*ip).x40_vel.x), -(*ip).x40_vel.y);
    }
    it_802D9714(gobj);
    if (*ip).xDD4_itemVar.heiho.x24 != 0 {
        HSD_JObjAddRotationY(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 0.15707964);
        (*ip).xDD4_itemVar.heiho.x24 = (*ip).xDD4_itemVar.heiho.x24.wrapping_sub(1);
        return;
    }
    if (*ip).facing_dir == -1.0 {
        HSD_JObjSetRotationY(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 3.1415927);
    } else {
        HSD_JObjSetRotationY(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 0.0);
    }
}

pub unsafe extern "C" fn itHeiho_UnkMotion1_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeiho_UnkMotion1_Coll");
    let ip = GET_ITEM(gobj);
    let attr = fattr(ip);
    let temp_r31 = ext::it_8026DA70(gobj);
    if (*ip).xDD4_itemVar.heiho.x24 == 0 && ext::it_80276308(gobj) != 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        (*ip).x40_vel.x = (*ip).facing_dir * *attr.offset((*ip).xDD4_itemVar.heiho.x21 as isize + 1);
        (*ip).xDD4_itemVar.heiho.x24 = 0x14;
    } else if temp_r31 == 1 {
        itHeiho_UnkMotion1_Anim_inline(gobj, 1);
    }
    0
}

pub unsafe extern "C" fn itHeiho_UnkMotion2_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeiho_UnkMotion2_Anim");
    0
}

/// `static it_802D8EC8_inline(Item_GObj*)`
pub unsafe fn it_802D8EC8_inline(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D8EC8_inline");
    let ip = (*gobj).user_data as *mut Item;
    let attr = fattr(ip);
    let rand = 2.0f32 * (ext::HSD_Randf() - 0.5);
    (*ip).x40_vel.x = fmadds(*attr.add(5), rand, (*ip).x40_vel.x);
    (*ip).x40_vel.y = 2.0;
    (*ip).x40_vel.z = 1.5;
    if (*ip).facing_dir == -1.0 {
        HSD_JObjSetRotationY(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 3.1415927);
    } else {
        HSD_JObjSetRotationY(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 0.0);
    }
}

pub unsafe extern "C" fn itHeiho_UnkMotion2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeiho_UnkMotion2_Phys");
    let ip = (*gobj).user_data as *mut Item;
    let it_attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*it_attr).x10_fall_speed, (*it_attr).x14_fall_speed_max);
    let rand = ext::HSD_Randi(3);
    (*ip).xD3C_spinSpeed = 0.017453292 * fmadds(8.0, rand as f32, 1.0);
    ext::it_80274A64(gobj);
}

pub unsafe extern "C" fn itHeiho_UnkMotion2_Coll(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeiho_UnkMotion2_Coll");
    0
}

pub unsafe extern "C" fn itHeiho_UnkMotion3_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeiho_UnkMotion3_Anim");
    0
}

pub unsafe extern "C" fn itHeiho_UnkMotion3_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeiho_UnkMotion3_Phys");
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
    if (*ip).xDD4_itemVar.heiho.x24 != 0 {
        (*ip).xDD4_itemVar.heiho.x24 = (*ip).xDD4_itemVar.heiho.x24.wrapping_sub(1);
        return;
    }
    it_802D9168(gobj);
}

pub unsafe extern "C" fn itHeiho_UnkMotion3_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeiho_UnkMotion3_Coll");
    ext::it_8026DA70(gobj);
    0
}

pub unsafe extern "C" fn it_802D8EC8(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("it_802D8EC8");
    let mut ip = (*gobj).user_data as *mut Item;
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut *mut i32;
    if !(*ip).xDD4_itemVar.heiho.x54.is_null() {
        ext::it_8028F8E4((*ip).xDD4_itemVar.heiho.x54);
        ext::it_8028FC5C((*ip).xDD4_itemVar.heiho.x54);
        (*ip).xDD4_itemVar.heiho.x54 = null_mut();
    }
    (*ip).xDCC_flag.set_b3(1);
    if (*ip).xC9C > (**attr as f32 * 0.8f32) as i32 {
        ext::it_8027CE44(gobj);
        it_802D8EC8_inline(gobj);
        it_802D98AC(gobj);
        ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
    } else {
        ip = (*gobj).user_data as *mut Item;
        if ext::it_8027B798(gobj, &raw mut (*ip).x40_vel) != 0 {
            ext::it_802762BC(ip);
        }
        (*ip).xDD4_itemVar.heiho.x24 = 0xC;
        it_802D98AC(gobj);
        ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as u32);
    }
    0
}

pub unsafe extern "C" fn it_802D9168(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D9168");
    let zero: f32 = 0.0;
    let mut sp14: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    (*ip).xDD4_itemVar.heiho.x24 = 0;
    ext::Stage_UnkSetVec3TCam_Offset(&raw mut sp14);
    if (*ip).pos.x < sp14.x {
        HSD_JObjGetRotationY(jobj);
        if (*jobj).rotate.y == zero {
            (*ip).xDD4_itemVar.heiho.x24 = 0x14;
        }
        (*ip).facing_dir = -1.0;
    } else if (*ip).pos.x > sp14.x {
        HSD_JObjGetRotationY(jobj);
        if (*jobj).rotate.y != zero {
            (*ip).xDD4_itemVar.heiho.x24 = 0x14;
        }
        (*ip).facing_dir = 1.0;
    }
    it_802D98AC(gobj);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as u32);
    it_802D98C4((*(*ip).xBBC_dynamicBoneTable).bones[1], ip);
    (*ip).on_accessory = Some(it_802D96B0);
}

pub unsafe extern "C" fn itHeiho_UnkMotion4_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeiho_UnkMotion4_Anim");
    let ip = GET_ITEM(gobj);
    let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    it_802D98C4((*(*ip).xBBC_dynamicBoneTable).bones[1], ip);
    if ext::it_80272C6C(gobj) == 0 {
        itHeiho_UnkMotion1_Anim_inline(gobj, 4);
    }
    let child = HSD_JObjGetChild(jobj);
    if (*ip).xDD4_itemVar.heiho.x2C > 0x3C0 {
        ext::HSD_AObjSetFlags((*child).aobj, 0x10000000);
    }
    if (*ip).xDD4_itemVar.heiho.x2C > 0x3C0 {
        ext::HSD_AObjSetFlags((*child).aobj, 0x10000000);
    } else {
        ext::HSD_AObjSetRate((*child).aobj, 2.0);
    }
    ext::HSD_AObjSetRate((*(*(*(*child).u.dobj).mobj).tobj).aobj, 2.0);
    0
}

pub unsafe extern "C" fn itHeiho_UnkMotion4_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeiho_UnkMotion4_Phys");
    let ip = GET_ITEM(gobj);
    let attr = fattr(ip);
    (*ip).x40_vel.x = 1.5f32 * ((*ip).facing_dir * *attr.offset((*ip).xDD4_itemVar.heiho.x21 as isize + 1));
    if (*ip).xDD4_itemVar.heiho.x2C > 960 {
        (*ip).x40_vel.y = -fmadds(-0.04, abs_c((*ip).x40_vel.x), -(*ip).x40_vel.y);
    }
    if (*ip).xDD4_itemVar.heiho.x24 != 0 {
        HSD_JObjAddRotationY(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 0.15707964);
        (*ip).xDD4_itemVar.heiho.x24 = (*ip).xDD4_itemVar.heiho.x24.wrapping_sub(1);
        return;
    }
    if (*ip).facing_dir == -1.0 {
        HSD_JObjSetRotationY(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 3.1415927);
    } else {
        HSD_JObjSetRotationY(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 0.0);
    }
}

pub unsafe extern "C" fn itHeiho_UnkMotion4_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeiho_UnkMotion4_Coll");
    let ip = GET_ITEM(gobj);
    let attr = fattr(ip);
    let temp_r31 = ext::it_8026DA70(gobj);
    if (*ip).xDD4_itemVar.heiho.x24 == 0 && ext::it_80276308(gobj) != 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        (*ip).x40_vel.x = (*ip).facing_dir * *attr.offset((*ip).xDD4_itemVar.heiho.x21 as isize + 1);
        (*ip).xDD4_itemVar.heiho.x24 = 0x14;
    } else if temp_r31 == 1 {
        it_802D9168(gobj);
    }
    0
}

pub unsafe extern "C" fn it_802D96B0(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D96B0");
    let ip = GET_ITEM(gobj);
    if !(*ip).xDD4_itemVar.heiho.x54.is_null() {
        let mut vec: Vec3 = (*ip).pos;
        vec.z = -2.0 * (*ip).facing_dir;
        ext::it_8028F9D8((*ip).xDD4_itemVar.heiho.x54, &raw mut vec, (*ip).facing_dir);
    }
}

/// `static inline it_802D9714_inline(Item_GObj*)`
pub unsafe fn it_802D9714_inline(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D9714_inline");
    let ip = GET_ITEM(gobj);
    (*ip).xDCC_flag.set_b3(1);
    if !(*ip).xDD4_itemVar.heiho.x54.is_null() {
        ext::it_8028F9B8((*ip).xDD4_itemVar.heiho.x54);
        (*ip).xDD4_itemVar.heiho.x54 = null_mut();
    }
}

pub unsafe extern "C" fn it_802D9714(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D9714");
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).xDD4_itemVar.heiho.x22 == 0 {
        if (*ip).pos.x > (20.0 + ext::Stage_GetBlastZoneLeftOffset())
            && (*ip).pos.x < (ext::Stage_GetBlastZoneRightOffset() - 20.0)
        {
            (*ip).xDD4_itemVar.heiho.x22 = 1;
        }
    } else if (*ip).pos.x > (20.0 + ext::Stage_GetBlastZoneRightOffset()) {
        it_802D9714_inline(gobj);
    } else if (*ip).pos.x < (ext::Stage_GetBlastZoneLeftOffset() - 20.0) {
        it_802D9714_inline(gobj);
    } else if (*ip).pos.y > (20.0 + ext::Stage_GetBlastZoneTopOffset()) {
        it_802D9714_inline(gobj);
    } else if (*ip).pos.y < (ext::Stage_GetBlastZoneBottomOffset() - 20.0) {
        it_802D9714_inline(gobj);
    }
}

pub unsafe extern "C" fn it_802D98AC(arg0: *mut HSD_GObj) {
    melee_core::ct!("it_802D98AC");
    let ip = (*arg0).user_data as *mut Item;
    (*ip).xDD4_itemVar.heiho.x3C.x = 0.0;
    (*ip).xDD4_itemVar.heiho.x3C.y = 0.0;
    (*ip).xDD4_itemVar.heiho.x3C.z = 0.0;
}

pub unsafe extern "C" fn it_802D98C4(jobj: *mut HSD_JObj, arg1: *mut Item) {
    melee_core::ct!("it_802D98C4");
    itUpdateVelocityFromBone(jobj, arg1, &raw mut (*arg1).xDD4_itemVar.heiho.x3C);
}

pub unsafe extern "C" fn it_802D9A0C(jobj: *mut HSD_GObj, arg1: *mut HSD_GObj) {
    melee_core::ct!("it_802D9A0C");
    ext::it_8026B894(jobj, arg1);
}
