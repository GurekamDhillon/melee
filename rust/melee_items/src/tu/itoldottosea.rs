//! Port of src/melee/it/kinds/itoldottosea.c: the Old Ottosea (Seal) enemy. Callees of unported TUs go through `ext`
//! (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itoldottosea.rs.
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
    fn HSD_Randf() -> f32;
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn gm_8016AE80() -> i32;
    fn itOldottosea_UnkMotion10_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itwhitebea::itOldottosea_UnkMotion10_Anim(a0 as _) as _ };
    fn itOldottosea_UnkMotion10_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itwhitebea::itOldottosea_UnkMotion10_Coll(a0 as _) as _ };
    fn itOldottosea_UnkMotion10_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itwhitebea::itOldottosea_UnkMotion10_Phys(a0 as _) };
    fn itOldottosea_UnkMotion11_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itwhitebea::itOldottosea_UnkMotion11_Anim(a0 as _) as _ };
    fn itOldottosea_UnkMotion11_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itwhitebea::itOldottosea_UnkMotion11_Coll(a0 as _) as _ };
    fn itOldottosea_UnkMotion11_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itwhitebea::itOldottosea_UnkMotion11_Phys(a0 as _) };
    fn itOldottosea_UnkMotion8_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itwhitebea::itOldottosea_UnkMotion8_Anim(a0 as _) as _ };
    fn itOldottosea_UnkMotion8_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itwhitebea::itOldottosea_UnkMotion8_Phys(a0 as _) };
    fn itOldottosea_UnkMotion9_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itwhitebea::itOldottosea_UnkMotion9_Anim(a0 as _) as _ };
    fn itOldottosea_UnkMotion9_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itwhitebea::itOldottosea_UnkMotion9_Coll(a0 as _) as _ };
    fn itOldottosea_UnkMotion9_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itwhitebea::itOldottosea_UnkMotion9_Phys(a0 as _) };
    fn it_8026B684(a0: *mut Vec) -> f32 => unsafe { crate::tu::it_26B1::it_8026B684(a0 as _) };
    fn it_8026BD9C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd9c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026BDB4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BDB4(a0) };
    fn it_8026D324(a0: i32) -> i32 => unsafe { crate::tu::itspawn::it_8026D324(a0) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026D8A4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D8A4(a0, a1) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_8026E664(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E664(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_802750E8(a0: *mut HSD_GObj, a1: i32) -> i32 => unsafe { crate::itlib::it_802750e8(&*((*a0).user_data as *mut Item), a1) };
    fn it_80275270(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275270(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027542C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8027542c(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275474(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275474(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027b730(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B798(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027B798(a0, a1 as _) as _ };
    fn it_8027C0A8(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::tu::itzako::it_8027C0A8(a0, a1, a2) };
    fn it_8027C56C(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::itzako::it_8027C56C(a0, a1) };
    fn it_8027C8D0(a0: *mut Vec, a1: *mut Vec, a2: f32) => unsafe { crate::tu::itzako::it_8027C8D0(a0 as _, a1 as _, a2) };
    fn it_8027C9D8(a0: *mut Item) => unsafe { crate::tu::itzako::it_8027C9D8(a0) };
    fn it_8027CAD8(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CAD8(a0) };
    fn it_8027CBFC(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::itzako::it_8027CBFC(a0) };
    fn it_8027CE44(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CE44(a0) };
    fn it_8028EB88(a0: *mut HSD_GObj, a1: *mut Vec, a2: f32, a3: f32) -> *mut HSD_GObj => unsafe { crate::tu::itfreeze::it_8028EB88(a0 as _, a1, a2, a3) as _ };
    fn it_8028EC98(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::itfreeze::it_8028EC98(a0 as _, a1) };
    fn it_8028ECE0(a0: *mut HSD_GObj) => unsafe { crate::itkinds5::it_8028ece0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802E3400(a0: *mut HSD_GObj) => unsafe { crate::tu::itwhitebea::it_802E3400(a0 as _) };
    fn it_802E3528(a0: *mut HSD_GObj) => unsafe { crate::tu::itwhitebea::it_802E3528(a0 as _) };
    fn it_802E35CC(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itwhitebea::it_802E35CC(a0 as _) as _ };
    fn it_802E37A4(a0: *mut HSD_GObj) => unsafe { crate::tu::itwhitebea::it_802E37A4(a0 as _) };
    fn lbLang_IsSettingJP() -> i32 => unsafe { melee_runtime::tu::lblanguage::lbLang_IsSettingJP() as _ };
    fn lbLang_IsSettingUS() -> i32 => unsafe { melee_runtime::tu::lblanguage::lbLang_IsSettingUS() as _ };
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn itOldottosea_UnkMotion10_Anim(a0: *mut HSD_GObj) -> i32 { super::itOldottosea_UnkMotion10_Anim(a0) }
        pub unsafe extern "C" fn itOldottosea_UnkMotion10_Coll(a0: *mut HSD_GObj) -> i32 { super::itOldottosea_UnkMotion10_Coll(a0) }
        pub unsafe extern "C" fn itOldottosea_UnkMotion10_Phys(a0: *mut HSD_GObj) { super::itOldottosea_UnkMotion10_Phys(a0) }
        pub unsafe extern "C" fn itOldottosea_UnkMotion11_Anim(a0: *mut HSD_GObj) -> i32 { super::itOldottosea_UnkMotion11_Anim(a0) }
        pub unsafe extern "C" fn itOldottosea_UnkMotion11_Coll(a0: *mut HSD_GObj) -> i32 { super::itOldottosea_UnkMotion11_Coll(a0) }
        pub unsafe extern "C" fn itOldottosea_UnkMotion11_Phys(a0: *mut HSD_GObj) { super::itOldottosea_UnkMotion11_Phys(a0) }
        pub unsafe extern "C" fn itOldottosea_UnkMotion8_Anim(a0: *mut HSD_GObj) -> i32 { super::itOldottosea_UnkMotion8_Anim(a0) }
        pub unsafe extern "C" fn itOldottosea_UnkMotion8_Phys(a0: *mut HSD_GObj) { super::itOldottosea_UnkMotion8_Phys(a0) }
        pub unsafe extern "C" fn itOldottosea_UnkMotion9_Anim(a0: *mut HSD_GObj) -> i32 { super::itOldottosea_UnkMotion9_Anim(a0) }
        pub unsafe extern "C" fn itOldottosea_UnkMotion9_Coll(a0: *mut HSD_GObj) -> i32 { super::itOldottosea_UnkMotion9_Coll(a0) }
        pub unsafe extern "C" fn itOldottosea_UnkMotion9_Phys(a0: *mut HSD_GObj) { super::itOldottosea_UnkMotion9_Phys(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D40: *mut it_804D6D40_t = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `ItemStateTable it_803F89C8[]`
pub static mut it_803F89C8: [ItemStateTable; 12] = [
    st_row(0, pred_b(itOldottosea_UnkMotion0_Anim), Some(itOldottosea_UnkMotion0_Phys), pred_b(itOldottosea_UnkMotion0_Coll)),
    st_row(0, pred_b(itOldottosea_UnkMotion2_Anim), Some(itOldottosea_UnkMotion2_Phys), pred_b(itOldottosea_UnkMotion2_Coll)),
    st_row(1, pred_b(itOldottosea_UnkMotion2_Anim), Some(itOldottosea_UnkMotion2_Phys), pred_b(itOldottosea_UnkMotion2_Coll)),
    st_row(2, pred_b(itOldottosea_UnkMotion3_Anim), Some(itOldottosea_UnkMotion3_Phys), pred_b(itOldottosea_UnkMotion3_Coll)),
    st_row(0, pred_b(itOldottosea_UnkMotion4_Anim), Some(itOldottosea_UnkMotion4_Phys), pred_b(itOldottosea_UnkMotion4_Coll)),
    st_row(3, pred_b(itOldottosea_UnkMotion5_Anim), Some(itOldottosea_UnkMotion5_Phys), pred_b(itOldottosea_UnkMotion5_Coll)),
    st_row(4, pred_b(itOldottosea_UnkMotion7_Anim), Some(itOldottosea_UnkMotion7_Phys), pred_b(itOldottosea_UnkMotion7_Coll)),
    st_row(5, pred_b(itOldottosea_UnkMotion7_Anim), Some(itOldottosea_UnkMotion7_Phys), pred_b(itOldottosea_UnkMotion7_Coll)),
    st_row(3, Some(ext::cb::itOldottosea_UnkMotion8_Anim), Some(ext::cb::itOldottosea_UnkMotion8_Phys), None),
    st_row(6, Some(ext::cb::itOldottosea_UnkMotion9_Anim), Some(ext::cb::itOldottosea_UnkMotion9_Phys), Some(ext::cb::itOldottosea_UnkMotion9_Coll)),
    st_row(3, Some(ext::cb::itOldottosea_UnkMotion10_Anim), Some(ext::cb::itOldottosea_UnkMotion10_Phys), Some(ext::cb::itOldottosea_UnkMotion10_Coll)),
    st_row(3, Some(ext::cb::itOldottosea_UnkMotion11_Anim), Some(ext::cb::itOldottosea_UnkMotion11_Phys), Some(ext::cb::itOldottosea_UnkMotion11_Coll)),
];

/// `Item_ZakoDefeat` (kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_ZakoDefeat(gobj: *mut Item_GObj, ip: *mut Item) {
    ext::it_8027C9D8(ip);
    ext::it_802756D0(gobj);
    ext::it_80275474(gobj);
    ext::it_8027CE44(gobj);
    ext::Camera_RequestQuake(QuakeKind_Small, &mut (*ip).pos);
}

#[inline]
unsafe fn attrs_of(ip: *mut Item) -> *mut itOldottoseaAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOldottoseaAttributes
}

pub unsafe extern "C" fn it_802E2470(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);

    ext::it_8027B730(gobj);
    (*ip).facing_dir = ext::it_8026B684(&mut (*ip).pos);
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    let facing = if (*ip).facing_dir == -1.0 { -1 } else { 1 };
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
    (*ip).xD5C = 0;
    (*ip).xDCC_flag.set_b3(0);
    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_8027542C(gobj);
    ext::it_80275270(gobj);
    (*ip).xDC8_word.flags.set_x19(1);
    (*ip).xDD4_itemVar.oldottosea.x28 = 1;
    (*ip).xDD4_itemVar.oldottosea.x20 = null_mut();
    if ext::gm_8016AE80() == -1 {
        (*ip).xDD4_itemVar.oldottosea.x2C = 0;
    } else if ext::it_8026D324(It_Kind_Freeze) != 0 && ext::HSD_Randi((*attr).x28 as i32) == 0 {
        (*ip).xDD4_itemVar.oldottosea.x2C = 1;
    } else {
        (*ip).xDD4_itemVar.oldottosea.x2C = 0;
    }
    it_802E269C(gobj);
}

pub unsafe extern "C" fn it_2725_Logic8_DmgReceived(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);

    (*ip).init_facing_dir = (*ip).facing_dir;
    (*ip).xC9C = ((*ip).xC9C as f32 + ext::it_8027CBFC(gobj)) as i32;
    if (*ip).xC9C > (*(*attr).x0).x0 || (*ip).msid == 9 {
        if !(*ip).xDD4_itemVar.oldottosea.x20.is_null() {
            ext::it_8028ECE0((*ip).xDD4_itemVar.oldottosea.x20);
            ext::it_802E37A4(gobj);
        }
        Item_ZakoDefeat(gobj, ip);
        if ext::HSD_Randf() < (*ext::it_804D6D40).x8 {
            ext::it_802E3528(gobj);
        } else {
            ext::it_802E3400(gobj);
        }
    } else {
        it_802E2C80(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802E269C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027CAD8(gobj);
    ext::it_8027C0A8(gobj, (*ip).facing_dir, 1.0);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOldottosea_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.oldottosea.x28 == 0 {
        it_802E27B4(gobj);
        return;
    }
    (*ip).xDD4_itemVar.oldottosea.x28 = (*ip).xDD4_itemVar.oldottosea.x28.wrapping_sub(1);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_802E2BC0));
    ext::it_802E35CC(gobj) != 0
}

pub unsafe extern "C" fn it_802E27B4(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);
    let mut pos: Vec3 = core::mem::zeroed();

    itResetVelocity(ip);
    (*ip).xDD4_itemVar.oldottosea.x24 = 0;

    let int_dir = if -1.0 == (*ip).facing_dir { -1 } else { 1 };
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, int_dir);

    ext::it_802756E0(gobj);

    if (*ip).xDD4_itemVar.oldottosea.x2C != 0 {
        pos = (*ip).pos;
        pos.x = fmadds((*attr).x14, (*ip).facing_dir, pos.x);
        (*ip).xDD4_itemVar.oldottosea.x20 = ext::it_8028EB88(gobj, &mut pos, (*ip).facing_dir, (*attr).x24);
        if !(*ip).xDD4_itemVar.oldottosea.x20.is_null() {
            ext::it_8028EC98((*ip).xDD4_itemVar.oldottosea.x20, (*ip).x40_vel.x * (*attr).x1C);
        }
        (*ip).xDD4_itemVar.oldottosea.x2C = 0;
    }

    if !(*ip).xDD4_itemVar.oldottosea.x20.is_null() {
        ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    }
}

pub unsafe extern "C" fn itOldottosea_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);
    let threshold: i32;

    if ext::lbLang_IsSettingUS() != 0 {
        if ext::lbLang_IsSettingJP() != 0 {
            threshold = 0x2A;
        } else {
            threshold = 0x50;
        }
        if (*ip).xDD4_itemVar.oldottosea.x24 >= threshold {
            if ext::HSD_Randi((*attr).x10 as i32) == 0 {
                it_802E2E30(gobj);
            } else {
                (*ip).x40_vel.x = 0.0;
                (*ip).xDD4_itemVar.oldottosea.x28 = 0;
                (*ip).xDD4_itemVar.oldottosea.x24 = 0;
                if ext::it_80272C6C(gobj) == 0 {
                    if !(*ip).xDD4_itemVar.oldottosea.x20.is_null() {
                        ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
                    } else {
                        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
                    }
                }
            }
        }
    } else {
        if ext::lbLang_IsSettingJP() != 0 {
            threshold = 0x2A;
        } else {
            threshold = 0x50;
        }
        if (*ip).xDD4_itemVar.oldottosea.x24 >= threshold {
            if ext::HSD_Randi((*attr).x10 as i32) == 0 {
                it_802E2E30(gobj);
            } else {
                (*ip).x40_vel.x = 0.0;
                (*ip).xDD4_itemVar.oldottosea.x28 = 0;
                (*ip).xDD4_itemVar.oldottosea.x24 = 0;
                if !(*ip).xDD4_itemVar.oldottosea.x20.is_null() {
                    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
                } else {
                    ext::Item_80268E5C(gobj, 1, 0x12);
                }
            }
        }
    }
    false
}

pub unsafe extern "C" fn itOldottosea_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);

    let jp_offset = if ext::lbLang_IsSettingJP() != 0 { 10 } else { 0 };

    if (*ip).xDD4_itemVar.oldottosea.x24 == jp_offset {
        (*ip).x40_vel.x = (*ip).facing_dir * (*(*attr).x0).x4;
    }

    (*ip).xDD4_itemVar.oldottosea.x24 = (*ip).xDD4_itemVar.oldottosea.x24.wrapping_add(1);

    if (*ip).x40_vel.x != 0.0 {
        if (*ip).xDD4_itemVar.oldottosea.x20.is_null() {
            if ext::it_802750E8(gobj, 2) != 0 {
                it_802E3098(gobj);
            }
        }
    }

    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0A8(gobj, (*ip).facing_dir, 5.0);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if ext::it_8026D8A4(gobj, Some(it_802E3098)) != 0 {
        if (*ip).x40_vel.x != 0.0 {
            if ext::it_80276308(gobj) != 0 {
                it_802E3098(gobj);
            }
        }
    } else {
        it_802E2BC0(gobj);
    }
    ext::it_802E35CC(gobj) != 0
}

pub unsafe extern "C" fn it_802E2BC0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);
    (*ip).xDD4_itemVar.oldottosea.x28 = 0;
    (*ip).x40_vel.x *= (*attr).x4;
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itOldottosea_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E414(gobj, Some(it_802E269C));
    ext::it_802E35CC(gobj) != 0
}

pub unsafe extern "C" fn it_802E2C80(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);

    ext::it_802762BC(ip);

    if ext::it_8027B798(gobj, &mut (*ip).x40_vel) != 0 {
        ext::it_802762BC(ip);
    } else {
        ext::it_802762B0(ip);
    }

    (*ip).facing_dir = (*ip).init_facing_dir;
    ext::Item_80268E5C(gobj, 5, 2);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);

    if ext::it_80272C6C(gobj) == 0 {
        if (*ip).ground_or_air == GA_Air {
            ext::Item_80268E5C(gobj, 5, 2);
        } else {
            it_802E2DF4(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn itOldottosea_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
    }
}

pub unsafe extern "C" fn itOldottosea_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        ext::it_8026E664(gobj, Some(it_802E2DF4));
    } else {
        ext::it_8026D62C(gobj, Some(it_802E2BC0));
    }
    ext::it_802E35CC(gobj) != 0
}

pub unsafe extern "C" fn it_802E2DF4(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xCCC_incDamageDirection == (*ip).facing_dir {
        it_802E3098(gobj);
        return;
    }
    it_802E27B4(gobj);
}

pub unsafe extern "C" fn it_802E2E30(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
    (*ip).xDD4_itemVar.oldottosea.x24 = 0;
    if !(*ip).xDD4_itemVar.oldottosea.x20.is_null() {
        ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    }
}

pub unsafe extern "C" fn itOldottosea_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    let mut ip = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        (*ip).xDD4_itemVar.oldottosea.x28 = 0;
        ip = GET_ITEM(gobj);
        ext::it_8027CAD8(gobj);
        ext::it_8027C0A8(gobj, (*ip).facing_dir, 1.0);
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOldottosea_UnkMotion7_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);
    let mut threshold: i32;

    if ext::lbLang_IsSettingJP() != 0 {
        threshold = 0x19;
    } else {
        threshold = 0x19;
    }
    if (*ip).xDD4_itemVar.oldottosea.x24 == threshold {
        (*ip).x40_vel.x = (*ip).facing_dir * (*attr).x8;
    } else {
        if ext::lbLang_IsSettingJP() != 0 {
            threshold = 0x19;
        } else {
            threshold = 0x19;
        }
        if (*ip).xDD4_itemVar.oldottosea.x24 > threshold {
            (*ip).x40_vel.x = fmadds(-(*ip).facing_dir, (*attr).xC, (*ip).x40_vel.x);
        }
    }
    (*ip).xDD4_itemVar.oldottosea.x24 = (*ip).xDD4_itemVar.oldottosea.x24.wrapping_add(1);
    if (*ip).x40_vel.x != 0.0 && (*ip).xDD4_itemVar.oldottosea.x20.is_null() && ext::it_802750E8(gobj, 2) != 0 {
        it_802E3098(gobj);
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0A8(gobj, (*ip).facing_dir, 5.0);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion7_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    ext::it_8026D62C(gobj, Some(it_802E2BC0));
    if (*ip).x40_vel.x != 0.0 {
        if ext::it_80276308(gobj) != 0 {
            it_802E3098(gobj);
        }
    }
    ext::it_802E35CC(gobj) != 0
}

pub unsafe extern "C" fn it_802E3098(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);

    itResetVelocity(ip);

    let val = if ext::lbLang_IsSettingJP() != 0 { 0x37 } else { 0x28 };
    (*ip).xDD4_itemVar.oldottosea.x28 = val;

    ext::Item_80268E5C(gobj, 3, 2);
    ext::it_8026BDB4(gobj);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.oldottosea.x28 == 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        let int_dir = if -1.0 == (*ip).facing_dir { -1 } else { 1 };
        ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, int_dir);
    }
    (*ip).xDD4_itemVar.oldottosea.x28 = (*ip).xDD4_itemVar.oldottosea.x28.wrapping_sub(1);
    if ext::it_80272C6C(gobj) == 0 {
        (*ip).xDD4_itemVar.oldottosea.x28 = 0;
        ext::it_8026BD9C(gobj);
        it_802E27B4(gobj);
    }
    false
}

pub unsafe extern "C" fn itOldottosea_UnkMotion3_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itOldottosea_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_802E2BC0));
    ext::it_802E35CC(gobj) != 0
}
