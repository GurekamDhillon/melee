//! Port of src/melee/it/kinds/itoldkuri.c: the Old Kuribo (Goomba) enemy. Callees of unported TUs go through `ext`
//! (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itoldkuri.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

use super::kinds_inl::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_Randf() -> f32;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn ftLib_80086D40(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn it_8026B684(a0: *mut Vec) -> f32;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_8026DFB0(a0: *mut HSD_GObj) -> i32;
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32;
    fn it_80274740(a0: *mut HSD_GObj);
    fn it_80274C88(a0: *mut HSD_GObj);
    fn it_80274CAC(a0: *mut HSD_GObj);
    fn it_802750E8(a0: *mut HSD_GObj, a1: i32) -> i32;
    fn it_80275270(a0: *mut HSD_GObj);
    fn it_8027542C(a0: *mut HSD_GObj);
    fn it_80275474(a0: *mut HSD_GObj);
    fn it_802756D0(a0: *mut HSD_GObj);
    fn it_802756E0(a0: *mut HSD_GObj);
    fn it_802762B0(a0: *mut Item);
    fn it_802762BC(a0: *mut Item);
    fn it_80276308(a0: *mut HSD_GObj) -> i32;
    fn it_8027B730(a0: *mut HSD_GObj);
    fn it_8027B798(a0: *mut HSD_GObj, a1: *mut Vec) -> i32;
    fn it_8027B964(a0: *mut HSD_GObj, a1: i32);
    fn it_8027BA54(a0: *mut HSD_GObj, a1: *mut Vec) -> i32;
    fn it_8027C0A8(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_8027C56C(a0: *mut HSD_GObj, a1: f32);
    fn it_8027C794(a0: *mut HSD_GObj) -> i32;
    fn it_8027C79C(a0: *mut HSD_GObj) -> i32;
    fn it_8027C824(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj) -> i32>) -> i32;
    fn it_8027C8D0(a0: *mut Vec, a1: *mut Vec, a2: f32);
    fn it_8027C9D8(a0: *mut Item);
    fn it_8027CAD8(a0: *mut HSD_GObj);
    fn it_8027CB3C(a0: *mut HSD_GObj);
    fn it_8027CBA4(a0: *mut HSD_GObj);
    fn it_8027CBFC(a0: *mut HSD_GObj) -> f32;
    fn it_8027CC88(a0: *mut HSD_GObj) -> *mut HSD_GObj;
    fn it_8027CE44(a0: *mut HSD_GObj);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D40: *mut it_804D6D40_t = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `itOldkuriAttributes` (file-local typedef of the C)
#[repr(C)]
pub struct itOldkuriAttributes {
    pub x0: *mut i32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
}
melee_core::gc_layout!(itOldkuriAttributes, "itOldkuriAttributes", size 0x10, align 4, [
    x0 @ 0x0: 4 "s32 *",
    x4 @ 0x4: 4 "f32",
    x8 @ 0x8: 4 "f32",
    xC @ 0xC: 4 "f32",
]);

/// `ItemStateTable it_803F8320[]`
pub static mut it_803F8320: [ItemStateTable; 12] = [
    st_row(-1, pred_b(itOldkuri_UnkMotion0_Anim), Some(itOldkuri_UnkMotion0_Phys), pred_b(itOldkuri_UnkMotion0_Coll)),
    st_row(0, pred_b(itOldkuri_UnkMotion1_Anim), Some(itOldkuri_UnkMotion1_Phys), pred_b(itOldkuri_UnkMotion1_Coll)),
    st_row(1, pred_b(itOldkuri_UnkMotion2_Anim), Some(itOldkuri_UnkMotion2_Phys), pred_b(itOldkuri_UnkMotion2_Coll)),
    st_row(0, pred_b(itOldkuri_UnkMotion3_Anim), Some(itOldkuri_UnkMotion3_Phys), pred_b(itOldkuri_UnkMotion3_Coll)),
    st_row(1, pred_b(itOldkuri_UnkMotion4_Anim), Some(itOldkuri_UnkMotion4_Phys), pred_b(itOldkuri_UnkMotion4_Coll)),
    st_row(3, pred_b(itOldkuri_UnkMotion5_Anim), Some(itOldkuri_UnkMotion5_Phys), None),
    st_row(4, pred_b(itOldkuri_UnkMotion6_Anim), Some(itOldkuri_UnkMotion6_Phys), pred_b(itOldkuri_UnkMotion6_Coll)),
    st_row(2, pred_b(itOldkuri_UnkMotion7_Anim), Some(itOldkuri_UnkMotion7_Phys), pred_b(itOldkuri_UnkMotion7_Coll)),
    st_row(2, pred_b(itOldkuri_UnkMotion8_Anim), Some(itOldkuri_UnkMotion8_Phys), pred_b(itOldkuri_UnkMotion8_Coll)),
    st_row(3, pred_b(itOldkuri_UnkMotion9_Anim), Some(itOldkuri_UnkMotion9_Phys), pred_b(itOldkuri_UnkMotion9_Coll)),
    st_row(3, pred_b(itOldkuri_UnkMotion10_Anim), Some(itOldkuri_UnkMotion10_Phys), pred_b(itOldkuri_UnkMotion10_Coll)),
    st_row(3, pred_b(itOldkuri_UnkMotion11_Anim), Some(itOldkuri_UnkMotion11_Phys), pred_b(itOldkuri_UnkMotion11_Coll)),
];

/// `Item_UpdateZakoVelocity` (kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_UpdateZakoVelocity(gobj: *mut Item_GObj, ip: *mut Item) {
    ext::it_802762BC(ip);
    ext::it_8027BA54(gobj, &mut (*ip).x40_vel);
    ext::it_802762BC(ip);
}

pub unsafe extern "C" fn itOldKuri_Logic29_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn it_802D73F0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027B730(gobj);
    (*ip).facing_dir = ext::it_8026B684(&mut (*ip).pos);
    (*ip).xD5C = 0;
    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_8027542C(gobj);
    ext::it_80275270(gobj);
    ext::it_80274740(gobj);
    (*ip).xDD4_itemVar.oldkuri.xDF8 = 0;
    (*ip).xDD4_itemVar.oldkuri.xDF4 = 0.0;
    it_802D747C(gobj);
}

pub unsafe extern "C" fn it_802D747C(gobj: *mut Item_GObj) {
    ext::it_8027CAD8(gobj);
    ext::it_8027C0A8(gobj, 0.0, 5.0);
    ext::it_802756E0(gobj);
    it_802D848C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.oldkuri.xDF8 == 0 {
        if (*ip).facing_dir == 1.0 {
            it_802D775C(gobj);
        } else {
            it_802D758C(gobj);
        }
    } else {
        (*ip).xDD4_itemVar.oldkuri.xDF8 = (*ip).xDD4_itemVar.oldkuri.xDF8.wrapping_sub(1);
    }
    ext::it_8027C0A8(gobj, 0.0, 5.0);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_802D7AF0));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802D758C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOldkuriAttributes;
    (*ip).xDD4_itemVar.oldkuri.xDF4 = (*ip).facing_dir * *((*attr).x0 as *mut f32).add(1);
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.oldkuri.xDF4;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).facing_dir = -1.0;
    (*ip).xDD4_itemVar.oldkuri.xDFC = 0;
    it_802D848C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);

    if ext::it_80272C6C(gobj) == 0 {
        if (*ip).facing_dir == -1.0 {
            (*ip).xDD4_itemVar.oldkuri.xDFC = 1;
            it_802D848C(gobj, 1, 0x12);
        } else {
            it_802D775C(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.oldkuri.xDFC != 0 && ext::it_802750E8(gobj, 2) != 0 {
        it_802D775C(gobj);
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0A8(gobj, 0.0, 5.0);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    ext::it_8026D62C(gobj, Some(it_802D7AF0));
    if ext::it_80276308(gobj) == 4 && (*ip).xDD4_itemVar.oldkuri.xDFC != 0 {
        it_802D775C(gobj);
    }
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802D775C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOldkuriAttributes;
    (*ip).xDD4_itemVar.oldkuri.xDF4 = (*ip).facing_dir * *((*attr).x0 as *mut f32).add(1);
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.oldkuri.xDF4;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).facing_dir = 1.0;
    (*ip).xDD4_itemVar.oldkuri.xDFC = 0;
    it_802D848C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        if (*ip).facing_dir == 1.0 {
            (*ip).xDD4_itemVar.oldkuri.xDFC = 1;
            it_802D848C(gobj, 2, 0x12);
        } else {
            // inlined copy of it_802D758C
            it_802D758C(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.oldkuri.xDFC != 0 && ext::it_802750E8(gobj, 2) != 0 {
        // inlined copy of it_802D758C
        it_802D758C(gobj);
    }
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.oldkuri.xDF4;
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0A8(gobj, 0.0, 5.0);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    ext::it_8026D62C(gobj, Some(it_802D7AF0));
    if ext::it_80276308(gobj) == 8 && (*ip).xDD4_itemVar.oldkuri.xDFC != 0 {
        // inlined copy of it_802D758C
        it_802D758C(gobj);
    }
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn itOldkuri_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        it_802D848C(gobj, 3, ITEM_ANIM_UPDATE | ITEM_HIT_PRESERVE);
    }
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOldkuriAttributes;
    (*ip).xDD4_itemVar.oldkuri.xDF8 = (*attr).xC as i32;
    ext::it_8026E414(gobj, Some(it_802D747C));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802D7AF0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).facing_dir > 0.0 {
        it_802D848C(gobj, 4, 0x12);
    } else {
        it_802D848C(gobj, 3, 0x12);
    }
}

pub unsafe extern "C" fn itOldkuri_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        it_802D848C(gobj, 4, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOldkuriAttributes;
    (*ip).xDD4_itemVar.oldkuri.xDF8 = (*attr).xC as i32;
    ext::it_8026E414(gobj, Some(it_802D747C));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn itOldKuri_Logic0_PickedUp(gobj: *mut Item_GObj) {
    ext::it_80275474(gobj);
    ext::it_80274740(gobj);
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::it_80275474(gobj);
        ext::it_80274740(gobj);
        ext::Item_80268E5C(gobj, 5, 2);
    }
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion5_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn it_2725_Logic0_Dropped(gobj: *mut Item_GObj) {
    ext::it_8027CBA4(gobj);
    {
        let ip = GET_ITEM(gobj);
        if (*ip).facing_dir > 0.0 {
            it_802D848C(gobj, 4, ITEM_ANIM_UPDATE | ITEM_HIT_PRESERVE);
        } else {
            it_802D848C(gobj, 3, ITEM_ANIM_UPDATE | ITEM_HIT_PRESERVE);
        }
    }
}

pub unsafe extern "C" fn it_2725_Logic0_Thrown(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027CB3C(gobj);
    ext::it_80274C88(gobj);
    (*ip).xDD4_itemVar.oldkuri.xDF8 = 0;
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    ext::it_8027C56C(gobj, 0.0);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion6_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8027C824(gobj, None) != 0
}

pub unsafe extern "C" fn itOldkuri_UnkMotion9_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        if (*ip).ground_or_air == GA_Air {
            ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE);
        } else {
            ext::it_8027CAD8(gobj);
            ext::it_8027C0A8(gobj, 0.0, 5.0);
            ext::it_802756E0(gobj);
            it_802D848C(gobj, 0, ITEM_ANIM_UPDATE);
        }
    }
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion9_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
    }
}

pub unsafe extern "C" fn itOldkuri_UnkMotion9_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        ext::it_8026E414(gobj, Some(it_802D747C));
    } else {
        ext::it_8026D62C(gobj, Some(it_802D7AF0));
    }
    ext::it_8027C794(gobj) != 0
}

/// `it_2725_Logic0_DmgReceived_inline` (static inline)
#[inline]
unsafe fn it_2725_Logic0_DmgReceived_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if ext::it_8027B798(gobj, &mut (*ip).x40_vel) != 0 {
        if ((*ip).x40_vel.y as f64) <= 0.2 {
            (*ip).x40_vel.y = 0.2;
        }
        ext::it_802762BC(ip);
    } else {
        ext::it_802762B0(ip);
    }
    (*ip).xDD4_itemVar.oldkuri.xDF8 = 0;
    ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn it_2725_Logic0_DmgReceived(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOldkuriAttributes;

    (*ip).xC9C = ((*ip).xC9C as f32 + ext::it_8027CBFC(gobj)) as i32;
    if (*ip).xC9C > *(*attr).x0 || (*ip).msid == 6 {
        ext::it_8027C9D8(ip);
        ext::it_80274C88(gobj);
        ext::it_802756D0(gobj);
        ext::it_80275474(gobj);
        ext::it_8027CE44(gobj);
        ext::Camera_RequestQuake(QuakeKind_Small, &mut (*ip).pos);
        if ext::HSD_Randf() < (*ext::it_804D6D40).x8 {
            it_802D839C(gobj);
        } else {
            it_802D82C4(gobj);
        }
    } else {
        it_2725_Logic0_DmgReceived_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802D8098(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    ext::it_802756D0(gobj);
    ext::it_80274C88(gobj);
    ext::it_80275474(gobj);
    ext::it_8027CE44(gobj);
    ext::Camera_RequestQuake(QuakeKind_Small, &mut (*ip).pos);
    if (*ip).ground_or_air == GA_Air {
        it_802D81FC(gobj);
    } else {
        it_802D813C(gobj);
    }
    ext::Item_8026AE84(ip, 0x12E, 0x7F, 0x40);
    ext::ftLib_80086D40((*ip).xCFC, 0xC, 0x0);
    false
}

pub unsafe extern "C" fn it_802D813C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
    ext::it_8027C0A8(gobj, 0.0, 0.0);
    ext::Item_80268E5C(gobj, 7, 2);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::it_8027CC88(gobj);
        return true;
    }
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion7_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itOldkuri_UnkMotion7_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802D81FC(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOldkuriAttributes;
    (*ip).x40_vel.x *= (*attr).x8;
    ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion8_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion8_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026DFB0(gobj) != 0
}

pub unsafe extern "C" fn it_802D82C4(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_802762BC(ip);
    ext::it_8027B964(gobj, 0);
    (*ip).xDD4_itemVar.oldkuri.xDEC = 0;
    ext::it_8027C56C(gobj, 0.0);
    ext::Item_80268E5C(gobj, 0xA, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion10_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0xA, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion10_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itOldkuri_UnkMotion10_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8027C79C(gobj) != 0
}

pub unsafe extern "C" fn it_802D839C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    Item_UpdateZakoVelocity(gobj, ip);
    ext::it_8027C56C(gobj, 0.0);
    ext::Item_80268E5C(gobj, 0xB, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldkuri_UnkMotion11_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0xB, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOldkuri_UnkMotion11_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
}

pub unsafe extern "C" fn itOldkuri_UnkMotion11_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802D848C(gobj: *mut Item_GObj, arg1: i32, arg2: Item_StateChangeFlags) {
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, arg1, arg2);
    ext::it_80274CAC(gobj);
    (*ip).jumped_on = pred_b(it_802D8098);
}

pub unsafe extern "C" fn it_802D84D8(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
