//! Port of src/melee/it/kinds/itpatapata.c: the Patapata (Koopa Paratroopa) enemy. Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itpatapata.rs.
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
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn ftLib_80086D40(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn it_8026B684(a0: *mut Vec) -> f32 => unsafe { crate::tu::it_26B1::it_8026B684(a0 as _) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D564(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D564(a0) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_8027327C(a0: *mut HSD_GObj, a1: i32, a2: i32) => unsafe { crate::tu::it_2725::it_8027327C(a0, a1, a2) };
    fn it_80274C88(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274c88(&mut *((*a0).user_data as *mut Item)) };
    fn it_80274CAC(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274CAC(a0) };
    fn it_802750E8(a0: *mut HSD_GObj, a1: i32) -> i32 => unsafe { crate::itlib::it_802750e8(&*((*a0).user_data as *mut Item), a1) };
    fn it_80275270(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275270(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027542C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8027542c(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275474(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275474(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzako::it_8027B5B0(a0, a1 as _, a2, a3 as _, a4) };
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027b730(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B798(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027B798(a0, a1 as _) as _ };
    fn it_8027C56C(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::itzako::it_8027C56C(a0, a1) };
    fn it_8027CAD8(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CAD8(a0) };
    fn it_8027CB3C(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CB3C(a0) };
    fn it_8027CBA4(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CBA4(a0) };
    fn it_8027CE44(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CE44(a0) };
    fn it_802DCE00(a0: *mut HSD_GObj) => unsafe { crate::tu::itnokonoko::it_802DCE00(a0 as _) };
    fn it_802DD7F0(a0: i32, a1: *mut Vec, a2: *mut Vec, a3: i32) -> *mut HSD_GObj => unsafe { crate::tu::itnokonoko::it_802DD7F0(a0, a1 as _, a2 as _, a3) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) };
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

/// `ItemStateTable it_803F88C0[]`
pub static mut it_803F88C0: [ItemStateTable; 8] = [
    st_row(0, None, None, None),
    st_row(1, pred_b(itPatapata_UnkMotion1_Anim), Some(itPatapata_UnkMotion1_Phys), pred_b(itPatapata_UnkMotion1_Coll)),
    st_row(1, pred_b(itPatapata_UnkMotion2_Anim), Some(itPatapata_UnkMotion3_Phys), pred_b(itPatapata_UnkMotion3_Coll)),
    st_row(2, pred_b(itPatapata_UnkMotion3_Anim), Some(itPatapata_UnkMotion3_Phys), pred_b(itPatapata_UnkMotion3_Coll)),
    st_row(-1, pred_b(itPatapata_UnkMotion4_Anim), Some(itPatapata_UnkMotion4_Phys), pred_b(itPatapata_UnkMotion4_Coll)),
    st_row(3, pred_b(itPatapata_UnkMotion5_Anim), Some(itPatapata_UnkMotion5_Phys), None),
    st_row(4, pred_b(itPatapata_UnkMotion6_Anim), Some(itPatapata_UnkMotion6_Phys), pred_b(itPatapata_UnkMotion6_Coll)),
    st_row(1, pred_b(itPatapata_UnkMotion7_Anim), Some(itPatapata_UnkMotion7_Phys), pred_b(itPatapata_UnkMotion7_Coll)),
];

/// `Item_InitZakoCollision` (kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitZakoCollision(gobj: *mut Item_GObj, ip: *mut Item) {
    (*ip).xD5C = 0;
    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_8027542C(gobj);
    ext::it_80275270(gobj);
    (*ip).xDC8_word.flags.set_x19(1);
}

#[inline]
unsafe fn attrs_of(ip: *mut Item) -> *mut itPatapataAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPatapataAttributes
}

pub unsafe extern "C" fn it_802E05A0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);

    ext::it_8027B730(gobj);
    (*ip).facing_dir = ext::it_8026B684(&mut (*ip).pos);
    let facing = if (*ip).facing_dir == -1.0 { -1 } else { 1 };
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    Item_InitZakoCollision(gobj, ip);
    (*ip).xDD4_itemVar.patapata.x28 = 0xA;
    (*ip).xDD4_itemVar.patapata.x24 = 0;
    (*ip).xDD4_itemVar.patapata.x20 = 0;
    (*ip).xDD4_itemVar.patapata.x40 = 2;
    ext::it_802762BC(ip);
    it_802E0678(gobj);
}

pub unsafe extern "C" fn it_802E0678(gobj: *mut Item_GObj) {
    ext::it_8027CAD8(gobj);
    it_802E1648(gobj, 1, 2);
}

pub unsafe extern "C" fn itPatapata_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itPatapata_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.patapata.x28 == 0 {
        it_802E0734(gobj);
        return;
    }
    (*ip).xDD4_itemVar.patapata.x28 = (*ip).xDD4_itemVar.patapata.x28.wrapping_sub(1);
}

pub unsafe extern "C" fn itPatapata_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    let it = GET_ITEM(gobj);
    if (*it).ground_or_air == GA_Air {
        ext::it_8026DA08(gobj);
    } else {
        ext::it_8026D564(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802E0734(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    match (*ip).xDD4_itemVar.patapata.x40 {
        0 => {
            let x10sq: i32 = (*attrs).x10.wrapping_mul((*attrs).x10);
            let xC = (*attrs).xC;
            (*ip).x40_vel.x = (*ip).facing_dir * (*(*attrs).x0).x4;
            (*ip).x40_vel.y = 0.0;
            (*ip).xDD4_itemVar.patapata.x30 = -1.0 * ((2.0 * xC) / x10sq as f32);
        }
        1 => {
            (*ip).x40_vel.x = (*ip).facing_dir * (*(*attrs).x0).x4;
            let half_width = (*attrs).x14 / 2 as f32;
            let frames = (half_width / (*(*attrs).x0).x4) as i32;
            (*ip).xDD4_itemVar.patapata.x20 = frames;
            (*ip).xDD4_itemVar.patapata.x28 = frames / 2;
            (*ip).xDD4_itemVar.patapata.x2C = -(*ip).facing_dir * ((2.0 * half_width) / frames.wrapping_mul(frames) as f32);
            let x24 = (*attrs).x1C / 2;
            let half_height = (*attrs).x18 / 2 as f32;
            (*ip).xDD4_itemVar.patapata.x24 = x24;
            (*ip).x40_vel.y = -1.0 * (half_height / x24 as f32);
            (*ip).xDD4_itemVar.patapata.x30 = (2.0 * half_height) / x24.wrapping_mul(x24) as f32;
        }
        2 => {
            let x20 = (*attrs).x24 / 2;
            let half_height = (*attrs).x20 / 2 as f32;
            (*ip).xDD4_itemVar.patapata.x20 = x20;
            (*ip).x40_vel.y = -1.0 * (half_height / x20 as f32);
            (*ip).xDD4_itemVar.patapata.x30 = (2.0 * half_height) / x20.wrapping_mul(x20) as f32;
        }
        _ => {}
    }
    (*ip).x40_vel.z = 0.0;
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    ext::it_802756E0(gobj);
    it_802E1648(gobj, 2, 2);
}

pub unsafe extern "C" fn itPatapata_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        it_802E1648(gobj, 2, 2);
    }
    false
}

pub unsafe extern "C" fn itPatapata_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    if ext::it_802750E8(gobj, 2) != 0 && (*ip).msid == 2 {
        it_802E0D9C(gobj);
    }
    match (*ip).xDD4_itemVar.patapata.x40 {
        0 => {
            (*ip).x40_vel.y += (*ip).xDD4_itemVar.patapata.x30;
        }
        1 => {
            if (*ip).xDD4_itemVar.patapata.x20 == 0 {
                (*ip).xDD4_itemVar.patapata.x20 = (((*attrs).x14 / 2 as f32) / (*(*attrs).x0).x4) as i32;
                (*ip).xDD4_itemVar.patapata.x2C *= -1.0;
            } else {
                (*ip).x40_vel.x += (*ip).xDD4_itemVar.patapata.x2C;
                (*ip).xDD4_itemVar.patapata.x20 = (*ip).xDD4_itemVar.patapata.x20.wrapping_sub(1);
                if (*ip).xDD4_itemVar.patapata.x28 == (*ip).xDD4_itemVar.patapata.x20 {
                    it_802E0D9C(gobj);
                }
            }
            if (*ip).xDD4_itemVar.patapata.x24 == 0 {
                (*ip).xDD4_itemVar.patapata.x24 = (*attrs).x1C / 2;
                (*ip).xDD4_itemVar.patapata.x30 *= -1.0;
            } else {
                (*ip).x40_vel.y += (*ip).xDD4_itemVar.patapata.x30;
                (*ip).xDD4_itemVar.patapata.x24 = (*ip).xDD4_itemVar.patapata.x24.wrapping_sub(1);
            }
        }
        2 => {
            if (*ip).xDD4_itemVar.patapata.x20 == 0 {
                (*ip).xDD4_itemVar.patapata.x20 = (*attrs).x24 / 2;
                (*ip).xDD4_itemVar.patapata.x30 *= -1.0;
            } else {
                (*ip).x40_vel.y += (*ip).xDD4_itemVar.patapata.x30;
                (*ip).xDD4_itemVar.patapata.x20 = (*ip).xDD4_itemVar.patapata.x20.wrapping_sub(1);
            }
        }
        _ => {}
    }
}

pub unsafe extern "C" fn itPatapata_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    let landed = ext::it_8026DA08(gobj);

    if ext::it_80276308(gobj) != 0 {
        match (*ip).xDD4_itemVar.patapata.x40 {
            0 => {
                if (*ip).msid == 2 {
                    let gobj2 = gobj; // permuterslop
                    it_802E0D9C(gobj2);
                }
            }
            1 => {
                (*ip).x40_vel.x = -(*ip).facing_dir * (*(*attrs).x0).x4;
                let width = (*attrs).x14;
                let frames = (width / (*(*attrs).x0).x4) as i32;
                (*ip).xDD4_itemVar.patapata.x20 = frames;
                (*ip).xDD4_itemVar.patapata.x28 = frames / 2;
                (*ip).xDD4_itemVar.patapata.x2C = (*ip).facing_dir * ((2.0 * width) / frames.wrapping_mul(frames) as f32);
                if (*ip).msid == 2 {
                    it_802E0D9C(gobj);
                }
            }
            2 => {}
            _ => {}
        }
    }
    if landed != 0 {
        match (*ip).xDD4_itemVar.patapata.x40 {
            0 => {
                let x10 = (*attrs).x10;
                let dir = (*ip).facing_dir;
                let xC = (*attrs).xC;
                (*ip).x40_vel.x = dir * (*(*attrs).x0).x4;
                (*ip).x40_vel.y = xC / x10 as f32;
            }
            1 => {
                (*ip).x40_vel.y *= -1.0;
                (*ip).xDD4_itemVar.patapata.x30 *= -1.0;
            }
            2 => {
                let frames = (*attrs).x24;
                let height = (*attrs).x20;
                (*ip).xDD4_itemVar.patapata.x20 = frames;
                (*ip).x40_vel.y = height / frames as f32;
                (*ip).xDD4_itemVar.patapata.x30 = -1.0 * ((2.0 * height) / frames.wrapping_mul(frames) as f32);
            }
            _ => {}
        }
    }
    false
}

pub unsafe extern "C" fn it_802E0D9C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    match (*ip).xDD4_itemVar.patapata.x40 {
        0 => {
            (*ip).x40_vel.x = -(*ip).facing_dir * (*(*attrs).x0).x4;
        }
        _ => {}
    }
    it_802E1648(gobj, 3, 2);
    (*ip).facing_dir = -(*ip).facing_dir;
}

pub unsafe extern "C" fn itPatapata_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        let facing = if (*ip).facing_dir == -1.0 { -1 } else { 1 };
        ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
        ext::it_8027C56C(gobj, (*ip).facing_dir);
        {
            let vel_x = (*ip).x40_vel.x;
            if (vel_x < 0.0 && (*ip).facing_dir == 1.0) || (vel_x > 0.0 && (*ip).facing_dir == -1.0) {
                (*ip).x40_vel.x *= -1.0;
            }
        }
        it_802E1648(gobj, 2, 2);
    }
    false
}

pub unsafe extern "C" fn it_2725_Logic4_DmgReceived(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 6 {
        it_802E1694(gobj);
    } else {
        it_802E11E0(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802E0F1C(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    it_802E11E0(gobj);
    ext::Item_8026AE84(ip, 0x12E, 0x7F, 0x40);
    ext::ftLib_80086D40((*ip).xCFC, 12, 0);
    false
}

pub unsafe extern "C" fn itPatapata_Logic4_PickedUp(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itPatapata_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itPatapata_UnkMotion5_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn it_2725_Logic4_Dropped(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027CBA4(gobj);
    (*ip).xDD4_itemVar.patapata.x24 = 0;
    (*ip).xDD4_itemVar.patapata.x20 = 0;
    (*ip).xDD4_itemVar.patapata.x28 = 2;
    ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itPatapata_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.patapata.x28 <= 0 {
        ext::it_8027CAD8(gobj);
        it_802E1648(gobj, 1, 2);
    } else {
        (*ip).xDD4_itemVar.patapata.x28 = (*ip).xDD4_itemVar.patapata.x28.wrapping_sub(1);
    }
    false
}

pub unsafe extern "C" fn itPatapata_UnkMotion7_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itPatapata_UnkMotion7_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026DA08(gobj);
    false
}

pub unsafe extern "C" fn it_2725_Logic4_Thrown(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027CB3C(gobj);
    ext::it_80274C88(gobj);
    (*ip).xDD4_itemVar.patapata.x28 = 0;
    (*ip).xDD4_itemVar.patapata.x24 = 0;
    (*ip).xDD4_itemVar.patapata.x20 = 0;
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itPatapata_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itPatapata_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itPatapata_UnkMotion6_Coll(gobj: *mut Item_GObj) -> bool {
    if ext::it_8026DAA8(gobj) & 0xF != 0 {
        it_802E1694(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802E11E0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    (*ip).init_facing_dir = (*ip).facing_dir;
    ext::it_8027B798(gobj, &mut (*ip).x40_vel);
    (*ip).facing_dir = (*ip).init_facing_dir;
    let coll_facing = if (*ip).facing_dir == -1.0 { -1 } else { 1 };
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, coll_facing);
    ext::HSD_JObjSetFlagsAll((*(*ip).xBBC_dynamicBoneTable).bones[0], JOBJ_HIDDEN as u32);
    ext::HSD_JObjClearFlagsAll((*(*ip).xBBC_dynamicBoneTable).bones[18], JOBJ_HIDDEN as u32);
    ext::HSD_JObjClearFlagsAll((*(*ip).xBBC_dynamicBoneTable).bones[11], JOBJ_HIDDEN as u32);
    ext::it_802756D0(gobj);
    ext::it_80274C88(gobj);
    ext::it_80275474(gobj);
    it_802E15B0(gobj);
    (*ip).xDD4_itemVar.patapata.x20 = (*attrs).x28 as i32;
    (*ip).xDD4_itemVar.patapata.x28 = (*attrs).x2C;
    (*ip).xDD4_itemVar.patapata.x44 = 0;

    let absf = |v: f32| if v < 0.0 { -v } else { v };
    let range = (absf((*attrs).x30) + absf((*attrs).x34)) as i32;

    let mut rand = ext::HSD_Randi(range);
    let mut val = (*attrs).x30;
    while rand != 0 {
        val += 0.1;
        rand -= 1;
    }
    (*ip).xDD4_itemVar.patapata.x2C = val;

    rand = ext::HSD_Randi(range);
    val = (*attrs).x30;
    while rand != 0 {
        val += 0.1;
        rand -= 1;
    }
    (*ip).x40_vel.x += val;

    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itPatapata_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.patapata.x28 == 0 {
        let attrs = attrs_of(ip);
        (*ip).xDD4_itemVar.patapata.x28 = (*attrs).x2C;
        let toggle = (*ip).xDD4_itemVar.patapata.x44 ^ 1;
        (*ip).xDD4_itemVar.patapata.x44 = toggle;
        if toggle != 0 {
            ext::HSD_JObjSetFlagsAll((*(*ip).xBBC_dynamicBoneTable).bones[18], JOBJ_HIDDEN as u32);
            ext::HSD_JObjSetFlagsAll((*(*ip).xBBC_dynamicBoneTable).bones[11], JOBJ_HIDDEN as u32);
        } else {
            ext::HSD_JObjClearFlagsAll((*(*ip).xBBC_dynamicBoneTable).bones[18], JOBJ_HIDDEN as u32);
            ext::HSD_JObjClearFlagsAll((*(*ip).xBBC_dynamicBoneTable).bones[11], JOBJ_HIDDEN as u32);
        }
    } else {
        (*ip).xDD4_itemVar.patapata.x28 = (*ip).xDD4_itemVar.patapata.x28.wrapping_sub(1);
    }
    false
}

pub unsafe extern "C" fn itPatapata_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    HSD_JObjAddTranslationZ((*(*ip).xBBC_dynamicBoneTable).bones[18], (*ip).xDD4_itemVar.patapata.x2C);
    ext::it_80272860(gobj, (*attrs).x38, (*attrs).x3C);
}

pub unsafe extern "C" fn itPatapata_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.patapata.x20 == 0 {
        (*ip).xDCC_flag.set_b3(1);
        return true;
    }
    (*ip).xDD4_itemVar.patapata.x20 = (*ip).xDD4_itemVar.patapata.x20.wrapping_sub(1);
    false
}

pub unsafe extern "C" fn it_802E15B0(gobj: *mut Item_GObj) {
    let mut pos: Vec3 = core::mem::zeroed();
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let ip = GET_ITEM(gobj);

    ext::lb_8000B1CC(jobj, null_mut(), &mut pos);
    ext::Camera_RequestQuake(QuakeKind_Small, &mut (*ip).pos);
    let new_gobj = ext::it_802DD7F0((*ip).xDD4_itemVar.patapata.x40, &mut pos, &mut (*ip).x40_vel, (*ip).facing_dir as i32);
    let new_ip = GET_ITEM(new_gobj);
    (*new_ip).xDD4_itemVar.nokonoko.x34 = (*ip).xCC8_knockback;
    (*new_ip).xDD4_itemVar.nokonoko.x3C = (*ip).xCAC_angle;
    (*new_ip).xDD4_itemVar.nokonoko.x38 = (*ip).xCCC_incDamageDirection;
    (*new_ip).xDD4_itemVar.nokonoko.x4 = (*ip).xDD4_itemVar.patapata.x4;
    (*ip).xDD4_itemVar.patapata.x4 = -1;
    ext::it_802DCE00(new_gobj);
}

pub unsafe extern "C" fn it_802E1648(gobj: *mut Item_GObj, arg1: i32, arg2: i32) {
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, arg1, arg2 as _);
    ext::it_80274CAC(gobj);
    (*ip).jumped_on = pred_b(it_802E0F1C);
}

pub unsafe extern "C" fn it_802E1694(gobj: *mut Item_GObj) {
    ext::it_8027327C(gobj, 0x444, 0x130);
    ext::it_8027CE44(gobj);
    ext::Item_8026A8EC(gobj);
}

pub unsafe extern "C" fn it_802E16D8(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn it_802E16F8(arg0: i32, pos: *mut Vec3, facing: i32) -> *mut Item_GObj {
    let mut vel: Vec3 = core::mem::zeroed();
    let gobj = ext::it_8027B5B0(It_Kind_Patapata, pos, null_mut(), &mut vel, 1);

    if !gobj.is_null() {
        let mut ip = GET_ITEM(gobj);
        (*ip).facing_dir = facing as f32;
        ext::it_8027C56C(gobj, (*ip).facing_dir);
        let coll_facing = if (*ip).facing_dir == -1.0 { -1 } else { 1 };
        ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, coll_facing);
        (*ip).xDD4_itemVar.patapata.x40 = arg0;
        if (*ip).xDD4_itemVar.patapata.x40 < 2 {
            ext::Item_80268E5C(gobj, 0, 2);
            ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
        }
        ext::it_8027CAD8(gobj);
        ip = GET_ITEM(gobj);
        ext::Item_80268E5C(gobj, 1, 2);
        ext::it_80274CAC(gobj);
        (*ip).jumped_on = pred_b(it_802E0F1C);
    }
    gobj
}
