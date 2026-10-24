//! Port of src/melee/it/kinds/itnokonoko.c: the Nokonoko (Koopa Troopa) enemy and its shells' spawner. Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itnokonoko.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

use super::kinds_inl::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn ScriptGame_EnemyDefeated(a0: *mut HSD_GObj) -> i32;
    fn ftLib_80086D40(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D5CC(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D5CC(a0) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026D82C(a0: *mut HSD_GObj) => unsafe { crate::tu::itgroundcoll::it_8026D82C(a0) };
    fn it_8026D8A4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D8A4(a0, a1) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_8026DB40(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DB40(a0) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_8027327C(a0: *mut HSD_GObj, a1: i32, a2: i32) => unsafe { crate::tu::it_2725::it_8027327C(a0, a1, a2) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_80274C88(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274c88(&mut *((*a0).user_data as *mut Item)) };
    fn it_80274CAC(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274CAC(a0) };
    fn it_802750E8(a0: *mut HSD_GObj, a1: i32) -> i32 => unsafe { crate::itlib::it_802750e8(&*((*a0).user_data as *mut Item), a1) };
    fn it_80275270(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275270(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027542C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8027542c(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275474(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275474(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275D5C(a0: *mut HSD_GObj, a1: *mut _itECB) => unsafe { crate::tu::itmaplib::it_80275D5C(a0, a1 as _) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzako::it_8027B5B0(a0, a1 as _, a2, a3 as _, a4) };
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027b730(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B798(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027B798(a0, a1 as _) as _ };
    fn it_8027C0A8(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::tu::itzako::it_8027C0A8(a0, a1, a2) };
    fn it_8027C56C(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::itzako::it_8027C56C(a0, a1) };
    fn it_8027C794(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itfwd8::it_8027c794(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_8027C8D0(a0: *mut Vec, a1: *mut Vec, a2: f32) => unsafe { crate::tu::itzako::it_8027C8D0(a0 as _, a1 as _, a2) };
    fn it_8027CAD8(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CAD8(a0) };
    fn it_8027CB3C(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CB3C(a0) };
    fn it_8027CBA4(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CBA4(a0) };
    fn it_8027CBFC(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::itzako::it_8027CBFC(a0) };
    fn it_8027CE44(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CE44(a0) };
    fn it_802E0100(a0: i32, a1: *mut Vec, a2: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzgshell::it_802E0100(a0, a1 as _, a2) as _ };
    fn it_802E0488(a0: i32, a1: *mut Vec, a2: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzrshell::it_802E0488(a0, a1 as _, a2) as _ };
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) };
    fn mpColl_800471F8(a0: *mut CollData) -> i32 => unsafe { melee_world::tu::mpcoll::mpColl_800471F8(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `ItemStateTable it_803F85F0[]`
pub static mut it_803F85F0: [ItemStateTable; 11] = [
    st_row(0, None, None, None),
    st_row(-1, pred_b(itNokonoko_UnkMotion1_Anim), Some(itNokonoko_UnkMotion1_Phys), pred_b(itNokonoko_UnkMotion1_Coll)),
    st_row(1, pred_b(itNokonoko_UnkMotion2_Anim), Some(itNokonoko_UnkMotion2_Phys), pred_b(itNokonoko_UnkMotion2_Coll)),
    st_row(2, pred_b(itNokonoko_UnkMotion3_Anim), Some(itNokonoko_UnkMotion3_Phys), pred_b(itNokonoko_UnkMotion3_Coll)),
    st_row(1, pred_b(itNokonoko_UnkMotion4_Anim), Some(itNokonoko_UnkMotion4_Phys), pred_b(itNokonoko_UnkMotion6_Coll)),
    st_row(3, pred_b(itNokonoko_UnkMotion5_Anim), Some(itNokonoko_UnkMotion5_Phys), pred_b(itNokonoko_UnkMotion5_Coll)),
    st_row(3, pred_b(itNokonoko_UnkMotion6_Anim), Some(itNokonoko_UnkMotion6_Phys), pred_b(itNokonoko_UnkMotion6_Coll)),
    st_row(4, pred_b(itNokonoko_UnkMotion8_Anim), Some(itNokonoko_UnkMotion7_Phys), pred_b(itNokonoko_UnkMotion8_Coll)),
    st_row(4, pred_b(itNokonoko_UnkMotion8_Anim), Some(itNokonoko_UnkMotion8_Phys), pred_b(itNokonoko_UnkMotion8_Coll)),
    st_row(4, pred_b(itNokonoko_UnkMotion9_Anim), Some(itNokonoko_UnkMotion9_Phys), None),
    st_row(5, pred_b(itNokonoko_UnkMotion10_Anim), Some(itNokonoko_UnkMotion10_Phys), pred_b(itNokonoko_UnkMotion10_Coll)),
];

/// `coll_inline` (static inline)
#[inline]
unsafe fn coll_inline(ip: *mut Item) {
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, if (*ip).facing_dir == -1.0 { -1 } else { 1 });
}

#[inline]
unsafe fn attrs_of(ip: *mut Item) -> *mut itNokoNoko_DatAttrs {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itNokoNoko_DatAttrs
}

pub unsafe extern "C" fn it_802DC69C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027B730(gobj);
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    coll_inline(ip);
    (*ip).xD5C = 0;

    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_8027542C(gobj);
    ext::it_80275270(gobj);
    (*ip).xDC8_word.flags.set_x19(1);
    (*ip).xDD4_itemVar.nokonoko.x28 = 0;
    (*ip).xDD4_itemVar.nokonoko.x40 = 0;
    (*ip).xDD4_itemVar.nokonoko.x20 = 0;
    (*ip).xDD4_itemVar.nokonoko.x74 = (*ip).xC1C;
    it_802DCB9C(gobj);
}

pub unsafe extern "C" fn itNokonoko_Logic3_DmgReceived(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let da = attrs_of(ip);
    (*ip).init_facing_dir = (*ip).facing_dir;
    (*ip).xC9C = ((*ip).xC9C as f32 + ext::it_8027CBFC(gobj)) as i32;
    if (*ip).msid == 0xA {
        it_802DD78C(gobj);
    } else if (*ip).xC9C > (*(*da).x0).x0 {
        it_802DCFBC(gobj);
    } else {
        it_802DCCCC(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802DC83C(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    (*ip).init_facing_dir = (*ip).facing_dir;
    ext::it_80274C88(gobj);
    it_802DCFBC(gobj);
    ext::Camera_RequestQuake(QuakeKind_Small, &mut (*ip).pos);
    ext::Item_8026AE84(ip, 0x12E, 0x7F, 0x40);
    ext::ftLib_80086D40((*ip).xCFC, 0xC, 0);
    false
}

pub unsafe extern "C" fn fn_802DC8B8(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027CAD8(gobj);
    ext::it_8027C0A8(gobj, (*ip).facing_dir, 5.0);
    it_802DD4A8(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itNokonoko_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.nokonoko.x28 <= 0 {
        it_802DC990(gobj);
        return;
    }
    (*ip).xDD4_itemVar.nokonoko.x28 = (*ip).xDD4_itemVar.nokonoko.x28.wrapping_sub(1);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_802DCB9C));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802DC990(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let da = attrs_of(ip);
    (*ip).x40_vel.x = (*ip).facing_dir * (*(*da).x0).x4;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    coll_inline(ip);
    ext::it_802756E0(gobj);
    if (*ip).xDD4_itemVar.nokonoko.x20 == 0 {
        (*ip).xDD4_itemVar.nokonoko.x20 = ((*da).x8 / (*(*da).x0).x4) as i32;
    }
    it_802DD4A8(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        it_802DD4A8(gobj, 2, 0x12);
    }
    false
}

pub unsafe extern "C" fn itNokonoko_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.nokonoko.x20 == 0 {
        it_802DD290(gobj);
    } else {
        (*ip).xDD4_itemVar.nokonoko.x20 = (*ip).xDD4_itemVar.nokonoko.x20.wrapping_sub(1);
    }
    if ext::it_802750E8(gobj, 2) != 0 {
        it_802DD290(gobj);
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0A8(gobj, (*ip).facing_dir, 5.0);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    if ext::it_8026D8A4(gobj, Some(it_802DD290)) != 0 {
        if ext::it_80276308(gobj) != 0 {
            it_802DD290(gobj);
        }
    } else {
        it_802DCB9C(gobj);
    }
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802DCB9C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let da = attrs_of(ip);
    (*ip).x40_vel.x *= (*da).x4;
    (*ip).xDD4_itemVar.nokonoko.x28 = 0;
    if (((*ip).facing_dir > 0.0) && ((*ip).x70_nudge.x < 0.0)) || (((*ip).facing_dir < 0.0) && ((*ip).x70_nudge.x > 0.0)) {
        (*ip).facing_dir = -(*ip).facing_dir;
        coll_inline(ip);
    }
    it_802DD4A8(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itNokonoko_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    let temp_r4 = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*temp_r4).x10_fall_speed, (*temp_r4).x14_fall_speed_max);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion6_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E414(gobj, Some(fn_802DC8B8));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802DCCCC(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.nokonoko.x28 = 0;
    ext::it_80275474(gobj);
    ext::it_8027B798(gobj, &mut (*ip).x40_vel);
    if ((*ip).x40_vel.y as f64) <= 0.2 {
        (*ip).x40_vel.y = 0.2;
    }
    ext::it_802762BC(ip);
    (*ip).facing_dir = (*ip).init_facing_dir;
    it_802DD4A8(gobj, 7, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion8_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        it_802DD4A8(gobj, 7, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itNokonoko_UnkMotion7_Phys(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == 1 {
        (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
    }
}

pub unsafe extern "C" fn itNokonoko_UnkMotion8_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E414(gobj, Some(it_802DCEC4));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802DCE00(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_802762BC(ip);
    (*ip).xDD4_itemVar.nokonoko.x28 = 10;
    ext::it_802756D0(gobj);
    (*ip).xCC8_knockback = (*ip).xDD4_itemVar.nokonoko.x34;
    (*ip).xCAC_angle = (*ip).xDD4_itemVar.nokonoko.x3C;
    (*ip).xCCC_incDamageDirection = (*ip).xDD4_itemVar.nokonoko.x38;
    ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
    if (*ip).xDD4_itemVar.nokonoko.x28 == 0 {
        ext::it_802756E0(gobj);
        return;
    }
    (*ip).xDD4_itemVar.nokonoko.x28 = (*ip).xDD4_itemVar.nokonoko.x28.wrapping_sub(1);
}

pub unsafe extern "C" fn it_802DCEC4(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.nokonoko.x28 = 0;
    ext::it_802756E0(gobj);
    if (*ip).xCCC_incDamageDirection == (*ip).facing_dir {
        it_802DD2DC(gobj);
        return;
    }
    it_802DC990(gobj);
}

pub unsafe extern "C" fn it_802DCFBC(gobj: *mut Item_GObj) {
    let mut box_: itECB;
    let ip = GET_ITEM(gobj);
    ext::it_802756D0(gobj);
    if ext::it_8027B798(gobj, &mut (*ip).x40_vel) != 0 {
        ext::it_802762BC(ip);
    } else {
        ext::it_802762B0(ip);
    }
    (*ip).facing_dir = (*ip).init_facing_dir;
    coll_inline(ip);
    (*ip).xDD4_itemVar.nokonoko.x28 = 0;
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    box_ = (*ip).xC1C;
    box_.top = 4.5;
    box_.right = 3.0;
    box_.left = 3.0;
    ext::it_80275D5C(gobj, &mut box_);
}

/// `lt_zero` (static inline)
#[inline]
fn lt_zero(val: f32) -> f32 {
    if val < 0.0 {
        -1.0
    } else {
        1.0
    }
}

pub unsafe extern "C" fn itNokonoko_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    let mut pos: Vec3;
    let var_f2: f32;
    let ip_2: *mut Item_GObj;
    let ip = GET_ITEM(gobj);
    if (ext::it_80272C6C(gobj) == 0) && ((*ip).ground_or_air as i32 == GA_Ground as i32) {
        let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
        var_f2 = lt_zero(57.29578 * HSD_JObjGetRotationY(jobj));
        pos = (*ip).pos;
        pos.y += 0.1;
        if (*ip).xDD4_itemVar.nokonoko.x40 < 2 {
            ip_2 = ext::it_802E0100((*ip).xDD4_itemVar.nokonoko.x40, &mut pos, var_f2 as i32);
        } else {
            ip_2 = ext::it_802E0488((*ip).xDD4_itemVar.nokonoko.x40, &mut pos, var_f2 as i32);
        }
        if !ip_2.is_null() {
            (*GET_ITEM(ip_2)).xDD4_itemVar.nokonoko.x4 = (*ip).xDD4_itemVar.nokonoko.x4;
            (*ip).xDD4_itemVar.nokonoko.x4 = -1;
        }
        // TARGET_PC: conversion ends the enemy even if shell allocation failed.
        ext::ScriptGame_EnemyDefeated(gobj);
        return true;
    }
    false
}

pub unsafe extern "C" fn itNokonoko_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
    }
}

pub unsafe extern "C" fn itNokonoko_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).ground_or_air == GA_Air {
        if ext::it_8026DB40(gobj) & 1 != 0 {
            ext::it_802762B0(ip);
            itResetVelocity(ip);
        }
    } else {
        ext::it_8026D82C(gobj);
    }
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn itNokonoko_UnkMotion6_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itNokonoko_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    ext::it_8027C0A8(gobj, (*GET_ITEM(gobj)).facing_dir, 0.0);
}

/// `it_802DD290_inline` (static inline)
#[inline]
pub unsafe fn it_802DD290_inline(gobj: *mut Item_GObj, val: i32) {
    let ip = (*gobj).user_data as *mut Item;
    itResetVelocity(ip);
    (*ip).xDD4_itemVar.nokonoko.x20 = 0;
    (*ip).xDD4_itemVar.nokonoko.x28 = 0x26;
    it_802DD4A8(gobj, 0x3, val as _);
}

pub unsafe extern "C" fn it_802DD290(gobj: *mut Item_GObj) {
    it_802DD290_inline(gobj, 0x12);
}

pub unsafe extern "C" fn it_802DD2DC(gobj: *mut HSD_GObj) {
    it_802DD290_inline(gobj, 2);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.nokonoko.x28 == 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        coll_inline(ip);
    }
    (*ip).xDD4_itemVar.nokonoko.x28 = (*ip).xDD4_itemVar.nokonoko.x28.wrapping_sub(1);
    if ext::it_80272C6C(gobj) == 0 {
        let ip = GET_ITEM(gobj);
        ext::it_8026D5CC(gobj);
        (*ip).xDD4_itemVar.nokonoko.x28 = 0;
        it_802DC990(gobj);
    }
    false
}

pub unsafe extern "C" fn itNokonoko_UnkMotion3_Phys(_gobj: *mut HSD_GObj) {}

pub unsafe extern "C" fn itNokonoko_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_802DCB9C));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802DD4A8(gobj: *mut HSD_GObj, arg1: i32, arg2: Item_StateChangeFlags) {
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, arg1, arg2);
    ext::it_80274CAC(gobj);
    (*ip).jumped_on = pred_b(it_802DC83C);
}

pub unsafe extern "C" fn it_802DD4F4(gobj: *mut Item_GObj) {
    (*GET_ITEM(gobj)).xDD4_itemVar.nokonoko.x28 = 0;
    ext::it_80274740(gobj);
    ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion9_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        it_802DD4F4(gobj);
    }
    false
}

pub unsafe extern "C" fn itNokonoko_UnkMotion9_Phys(_gobj: *mut HSD_GObj) {}

/// `it_802DD59C_inline` (static inline)
#[inline]
pub unsafe fn it_802DD59C_inline(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    let da = attrs_of(ip);
    (*ip).x40_vel.x *= (*da).x4;
    (*ip).xDD4_itemVar.nokonoko.x28 = 0;
    let temp_f2 = (*ip).facing_dir;
    if ((temp_f2 > 0.0) && ((*ip).x70_nudge.x < 0.0)) || ((temp_f2 < 0.0) && ((*ip).x70_nudge.x > 0.0)) {
        (*ip).facing_dir = -temp_f2;
        coll_inline(ip);
    }
    it_802DD4A8(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn it_802DD59C(gobj: *mut Item_GObj) {
    ext::it_8027CBA4(gobj);
    it_802DD59C_inline(gobj);
}

pub unsafe extern "C" fn it_802DD67C(gobj: *mut Item_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    ext::it_8027CB3C(gobj);
    ext::it_80274C88(gobj);
    (*ip).xDD4_itemVar.nokonoko.x28 = 0;
    ext::Item_80268E5C(gobj, 0xA, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion10_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0xA, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itNokonoko_UnkMotion10_Phys(gobj: *mut Item_GObj) {
    let temp_r4 = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*temp_r4).x10_fall_speed, (*temp_r4).x14_fall_speed_max);
}

pub unsafe extern "C" fn itNokonoko_UnkMotion10_Coll(gobj: *mut Item_GObj) -> bool {
    if ext::it_8026DAA8(gobj) & 0xF != 0 {
        it_802DD78C(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802DD78C(gobj: *mut Item_GObj) {
    ext::it_8027327C(gobj, 0x444, 0x130);
    ext::it_8027CE44(gobj);
    ext::Item_8026A8EC(gobj);
}

pub unsafe extern "C" fn it_802DD7D0(gobj: *mut Item_GObj, arg1: *mut Item_GObj) {
    ext::it_8026B894(gobj, arg1);
}

pub unsafe extern "C" fn it_802DD7F0(arg0: i32, arg1: *mut Vec3, arg2: *mut Vec3, arg3: i32) -> *mut Item_GObj {
    let gobj = ext::it_8027B5B0(It_Kind_Nokonoko, arg1, null_mut(), arg2, 1);
    if !gobj.is_null() {
        let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
        let ip = GET_ITEM(gobj);
        (*ip).facing_dir = arg3 as f32;
        coll_inline(ip);
        let temp_f31 = ((*ip).facing_dir as f64 * 1.5707963267948966) as f32;
        HSD_JObjSetRotationY(jobj, temp_f31);
        (*ip).xDD4_itemVar.nokonoko.x40 = arg0;
        if (*ip).xDD4_itemVar.nokonoko.x40 == 2 {
            ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
            ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
        }
        if it_802DDA84(gobj) {
            ext::it_802762B0(ip);
            fn_802DC8B8(gobj);
        } else {
            ext::it_802762BC(ip);
            it_802DD59C_inline(gobj);
        }
    }
    gobj
}

pub unsafe extern "C" fn it_802DDA84(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    let col: *mut CollData = &mut (*ip).x378_itemColl;
    (*ip).x378_itemColl.cur_pos = (*ip).pos;
    (*ip).x378_itemColl.last_pos = (*ip).x378_itemColl.cur_pos;
    (*ip).x378_itemColl.last_pos.y += 10.0;
    (*ip).x378_itemColl.cur_pos.y -= 10.0;
    let cond = ext::mpColl_800471F8(col);
    (*ip).pos = (*col).cur_pos;
    cond != 0
}
