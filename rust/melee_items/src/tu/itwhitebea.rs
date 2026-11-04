//! Port of src/melee/it/kinds/itwhitebea.c: Old Ottosea / white-bea (Ice Climber's Icicle Mountain enemy, logic
//! table 9). Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU,
//! shipping contraction) by tests/kinds_itwhitebea.rs.
//!
//! Fused sites (checked in the IR): `it_802E35CC` distance is `sqrtf(fmadds(dz, dz, fmadds(dx, dx, dy * dy)))`;
//! `sqrtf` is the `melee_runtime::gekko` port of math_ppc.h. The `itOldottosea*` names are duplicates of the
//! `itWhitebea*` bodies. `ip->xC9C += it_8027CBFC(gobj)` adds a float to an int (converted back by truncation).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::helpers::it_80275158;
#[allow(unused_imports)]
use crate::item::it_802762bc;
#[allow(unused_imports)]
use crate::itkinds::it_8028b988;
#[allow(unused_imports)]
use crate::itkinds::it_capsule_unk_motion4_phys;
#[allow(unused_imports)]
use crate::itlib::it_802755c0;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_80274658;
#[allow(unused_imports)]
use crate::itlib3::it_coll_bounce_off_victim;
#[allow(unused_imports)]
use crate::layout::itPikachuthunderAttributes;
#[allow(unused_imports)]
use crate::layout::itRshellAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruCann_DatAttrs;
#[allow(unused_imports)]
use crate::layout::itThunderPokemonAttributes;
#[allow(unused_imports)]
use crate::layout::itUnknownAttributes;
#[allow(unused_imports)]
use crate::layout::ItLGunAttr;
#[allow(unused_imports)]
use crate::layout::MaximTomatoSpecialAttr;
#[allow(unused_imports)]
use crate::layout::ScopeBeamAttrs;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::HitCapsule_Enabled;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds5;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::helpers::decrease_lifetimer;
#[allow(unused_imports)]
use crate::itkinds::it_802d5884;
#[allow(unused_imports)]
use crate::itkinds::it_802e657c;
#[allow(unused_imports)]
use crate::itkinds::it_chicorita_phys;
#[allow(unused_imports)]
use crate::itkinds::it_harisen_logic24_dmg_dealt;
#[allow(unused_imports)]
use crate::itkinds::it_lipstick_logic23_dmg_dealt;
#[allow(unused_imports)]
use crate::itkinds::it_lizardon_unk_motion1_phys;
#[allow(unused_imports)]
use crate::itkinds::it_maril_unk_motion2_anim_inline;
#[allow(unused_imports)]
use crate::itkinds::it_maril_unk_motion2_phys;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_8026b724;
#[allow(unused_imports)]
use crate::itlib::it_8026b73c;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_80273030;
#[allow(unused_imports)]
use crate::itlib::it_80273130;
#[allow(unused_imports)]
use crate::itlib::it_802751d8;
#[allow(unused_imports)]
use crate::itlib::it_80275870;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
#[allow(unused_imports)]
use crate::itlib3::it_8027b730;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::gekko::sqrtf;
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn Stage_GetBlastZoneBottomOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneBottomOffset() };
    fn Stage_GetBlastZoneLeftOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneLeftOffset() };
    fn Stage_GetBlastZoneRightOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneRightOffset() };
    fn Stage_GetBlastZoneTopOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() };
    fn grIceMt_801FA6D8() => unsafe { melee_world::tu::gricemt::grIceMt_801FA6D8() };
    fn it_2725_Logic9_Destroyed(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_2725_Logic9_Destroyed(a0) };
    fn it_8026B684(a0: *mut Vec) -> f32 => unsafe { crate::tu::it_26B1::it_8026B684(a0 as _) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026D6F4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D6F4(a0, a1) };
    fn it_8026D8A4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D8A4(a0, a1) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_802750E8(a0: *mut HSD_GObj, a1: i32) -> i32 => unsafe { crate::itlib::it_802750e8(&*((*a0).user_data as *mut Item), a1) };
    fn it_80275258(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275258(&mut *((*a0).user_data as *mut Item)) };
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
    fn it_8027B964(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itzako::it_8027B964(a0, a1) };
    fn it_8027BA54(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027BA54(a0, a1 as _) };
    fn it_8027C56C(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::itzako::it_8027C56C(a0, a1) };
    fn it_8027C794(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itfwd8::it_8027c794(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_8027C79C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzako::it_8027C79C(a0) as _ };
    fn it_8027C824(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj) -> i32>) -> i32 => unsafe { crate::tu::itzako::it_8027C824(a0, a1) };
    fn it_8027C8D0(a0: *mut Vec, a1: *mut Vec, a2: f32) => unsafe { crate::tu::itzako::it_8027C8D0(a0 as _, a1 as _, a2) };
    fn it_8027C9D8(a0: *mut Item) => unsafe { crate::tu::itzako::it_8027C9D8(a0) };
    fn it_8027CAD8(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CAD8(a0) };
    fn it_8027CB3C(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CB3C(a0) };
    fn it_8027CBA4(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CBA4(a0) };
    fn it_8027CBFC(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::itzako::it_8027CBFC(a0) };
    fn it_8027CE44(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CE44(a0) };
    fn it_8028EC98(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::itfreeze::it_8028EC98(a0 as _, a1) };
    fn it_8028ECE0(a0: *mut HSD_GObj) => unsafe { crate::itkinds5::it_8028ece0(&mut *((*a0).user_data as *mut Item)) };
    fn it_8028ECF0(a0: *mut HSD_GObj, a1: *mut Vec) -> *mut Item => unsafe { crate::tu::itfreeze::it_8028ECF0(a0 as _, a1) };
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) };
    fn mpLib_80056A1C(a0: i32, a1: *mut i32) -> *mut i32 => unsafe { melee_world::tu::mplib::mpLib_80056A1C(a0, a1) };
    fn mpLineGetFlags(a0: i32) -> u32 => unsafe { melee_world::tu::mplib::mpLineGetFlags(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D40;
}
// END GENERATED EXT

/// `ItemStateTable it_803F8A88[]`
pub static mut it_803F8A88: [ItemStateTable; 12] = [
    st_row(0, pred_b!(itWhitebea_UnkMotion0_Anim), Some(itWhitebea_UnkMotion0_Phys), pred_b!(itWhitebea_UnkMotion0_Coll)),
    st_row(1, pred_b!(itWhitebea_UnkMotion1_Anim), Some(itWhitebea_UnkMotion1_Phys), pred_b!(itWhitebea_UnkMotion1_Coll)),
    st_row(2, pred_b!(itWhitebea_UnkMotion2_Anim), Some(itWhitebea_UnkMotion2_Phys), pred_b!(itWhitebea_UnkMotion2_Coll)),
    st_row(1, pred_b!(itWhitebea_UnkMotion3_Anim), Some(itWhitebea_UnkMotion3_Phys), pred_b!(itWhitebea_UnkMotion3_Coll)),
    st_row(3, pred_b!(itWhitebea_UnkMotion4_Anim), Some(itWhitebea_UnkMotion4_Phys), pred_b!(itWhitebea_UnkMotion4_Coll)),
    st_row(4, pred_b!(itWhitebea_UnkMotion5_Anim), Some(itWhitebea_UnkMotion5_Phys), pred_b!(itWhitebea_UnkMotion5_Coll)),
    st_row(5, pred_b!(itWhitebea_UnkMotion6_Anim), Some(itWhitebea_UnkMotion6_Phys), pred_b!(itWhitebea_UnkMotion6_Coll)),
    st_row(6, pred_b!(itWhitebea_UnkMotion7_Anim), Some(itWhitebea_UnkMotion7_Phys), pred_b!(itWhitebea_UnkMotion7_Coll)),
    st_row(3, pred_b!(itWhitebea_UnkMotion8_Anim), Some(itWhitebea_UnkMotion8_Phys), None),
    st_row(7, pred_b!(itWhitebea_UnkMotion9_Anim), Some(itWhitebea_UnkMotion9_Phys), pred_b!(itWhitebea_UnkMotion9_Coll)),
    st_row(3, pred_b!(itWhitebea_UnkMotion10_Anim), Some(itWhitebea_UnkMotion10_Phys), pred_b!(itWhitebea_UnkMotion10_Coll)),
    st_row(3, pred_b!(itWhitebea_UnkMotion11_Anim), Some(itWhitebea_UnkMotion11_Phys), pred_b!(itWhitebea_UnkMotion11_Coll)),
];

/// `Item_UpdateZakoVelocity` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_UpdateZakoVelocity(gobj: *mut Item_GObj, ip: *mut Item) {
    ext::it_802762BC(ip);
    ext::it_8027BA54(gobj, &mut (*ip).x40_vel);
    ext::it_802762BC(ip);
}

/// `Item_ZakoDefeat` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_ZakoDefeat(gobj: *mut Item_GObj, ip: *mut Item) {
    ext::it_8027C9D8(ip);
    ext::it_802756D0(gobj);
    ext::it_80275474(gobj);
    ext::it_8027CE44(gobj);
    ext::Camera_RequestQuake(QuakeKind_Small as u32, &mut (*ip).pos);
}

#[inline]
fn facing_i(f: f32) -> i32 {
    if f == -1.0 {
        -1
    } else {
        1
    }
}

#[inline]
fn abs_i(x: i32) -> i32 {
    if x < 0 {
        -x
    } else {
        x
    }
}

pub unsafe extern "C" fn it_802E31F8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E31F8");
    let ip = (*gobj).user_data as *mut Item;
    if !(*ip).xDD4_itemVar.whitebea.x20.is_null() {
        ext::it_8028ECE0((*ip).xDD4_itemVar.whitebea.x20);
        it_802E37A4(gobj);
    }
    ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion8_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOldottosea_UnkMotion8_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        let ip = GET_ITEM(gobj);
        if !(*ip).xDD4_itemVar.whitebea.x20.is_null() {
            ext::it_8028ECE0((*ip).xDD4_itemVar.whitebea.x20);
            it_802E37A4(gobj);
        }
        ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOldottosea_UnkMotion8_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itOldottosea_UnkMotion8_Phys");}

pub unsafe extern "C" fn it_802E32B4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E32B4");
    ext::it_8027CBA4(gobj);
    let ip = (*gobj).user_data as *mut Item;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWhiteBeaAttributes;
    (*ip).xDD4_itemVar.whitebea.x28 = 0;
    (*ip).x40_vel.x *= (*attrs).x4;
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn it_802E3314(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E3314");
    let ip = GET_ITEM(gobj);
    ext::it_8027CB3C(gobj);
    (*ip).xDD4_itemVar.whitebea.x28 = 0;
    ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion9_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOldottosea_UnkMotion9_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOldottosea_UnkMotion9_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itOldottosea_UnkMotion9_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion9_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOldottosea_UnkMotion9_Coll");
    ext::it_8027C824(gobj, None) != 0
}

pub unsafe extern "C" fn it_802E3400(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E3400");
    let ip = GET_ITEM(gobj);
    (*ip).xDCC_flag.set_b3(1);
    ext::it_802762BC(ip);
    ext::it_8027B964(gobj, 0);
    (*ip).xDD4_itemVar.whitebea.x18 = 0;
    ext::Item_80268E5C(gobj, 0xA, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion10_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOldottosea_UnkMotion10_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0xA, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOldottosea_UnkMotion10_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itOldottosea_UnkMotion10_Phys");}

pub unsafe extern "C" fn itOldottosea_UnkMotion10_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOldottosea_UnkMotion10_Coll");
    ext::it_8027C79C(gobj) != 0
}

pub unsafe extern "C" fn it_2725_Logic3_Destroyed(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic3_Destroyed");
    let ip = GET_ITEM(gobj);
    if !(*ip).xDD4_itemVar.whitebea.x20.is_null() {
        ext::it_8028ECE0((*ip).xDD4_itemVar.whitebea.x20);
    }
    it_802E37A4(gobj);
    ext::it_2725_Logic9_Destroyed(gobj);
}

pub unsafe extern "C" fn it_802E3528(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E3528");
    let ip = GET_ITEM(gobj);
    Item_UpdateZakoVelocity(gobj, ip);
    ext::Item_80268E5C(gobj, 0xB, 3);
}

pub unsafe extern "C" fn itOldottosea_UnkMotion11_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOldottosea_UnkMotion11_Anim");
    false
}

pub unsafe extern "C" fn itOldottosea_UnkMotion11_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itOldottosea_UnkMotion11_Phys");
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
}

pub unsafe extern "C" fn itOldottosea_UnkMotion11_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOldottosea_UnkMotion11_Coll");
    it_802E35CC(gobj)
}

pub unsafe extern "C" fn it_802E35CC(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802E35CC");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOldottoseaAttributes;
    let mut sp30: Vec3 = core::mem::zeroed();

    if (*ip).pos.x > ext::Stage_GetBlastZoneRightOffset() {
        (*ip).xDCC_flag.set_b3(1);
    }
    if (*ip).pos.x < ext::Stage_GetBlastZoneLeftOffset() {
        (*ip).xDCC_flag.set_b3(1);
    }
    if (*ip).pos.y > ext::Stage_GetBlastZoneTopOffset() {
        (*ip).xDCC_flag.set_b3(1);
    }
    if (*ip).pos.y < ext::Stage_GetBlastZoneBottomOffset() {
        (*ip).xDCC_flag.set_b3(1);
    }
    if (*ip).xDCC_flag.b3() == 1 {
        if !(*ip).xDD4_itemVar.whitebea.x20.is_null() {
            ext::it_8028ECE0((*ip).xDD4_itemVar.whitebea.x20);
        }
        ext::it_2725_Logic9_Destroyed(gobj);
        return true;
    }
    if !(*ip).xDD4_itemVar.whitebea.x20.is_null() {
        let ip2 = HSD_GObjGetUserData(gobj) as *mut Item;
        let x20 = (*ip2).xDD4_itemVar.whitebea.x20;
        ext::it_8028ECF0(x20, &mut sp30);
        let pos = &mut (*ip2).pos as *mut Vec;
        let dx = (*pos).x - sp30.x;
        let dy = (*pos).y - sp30.y;
        let dz = (*pos).z - sp30.z;
        let dist = sqrtf(fmadds(dz, dz, fmadds(dx, dx, dy * dy)));
        if dist > (*attrs).x18 {
            ext::it_8028ECE0((*ip).xDD4_itemVar.whitebea.x20);
            it_802E37A4(gobj);
        } else if dist <= (*attrs).x14 {
            if (*ip).msid == 7 {
                ext::it_8028EC98((*ip).xDD4_itemVar.whitebea.x20, (*ip).x40_vel.x * (*attrs).x20);
            } else {
                ext::it_8028EC98((*ip).xDD4_itemVar.whitebea.x20, (*ip).x40_vel.x * (*attrs).x1C);
            }
        }
    }
    false
}

pub unsafe extern "C" fn it_802E3784(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E3784");
    ext::it_8026B894(gobj, ref_gobj);
}

/// Only called by itFreeze_Logic17_PickedUp (itfreeze), hence the freeze item vars.
pub unsafe extern "C" fn it_802E37A4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E37A4");
    if !gobj.is_null() {
        // the C's `freeze.x20` is the 4-byte slot at itemVar+0x20: the Topi's (`oldottosea.x20`) link to its ice block, the
        // item this is called on. Natively freeze.x20 sits at +0x28 (after the 8-byte unk_1C) over oldottosea.x24/x28.
        (*GET_ITEM(gobj)).xDD4_itemVar.oldottosea.x20 = null_mut();
    }
}

pub unsafe extern "C" fn it_802E37BC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E37BC");
    let ip = GET_ITEM(gobj);
    ext::it_8027B730(gobj);
    (*ip).facing_dir = ext::it_8026B684(&mut (*ip).pos);
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    let facing = facing_i((*ip).facing_dir);
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
    (*ip).xD5C = 0;
    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_8027542C(gobj);
    ext::it_80275270(gobj);
    ext::it_80274740(gobj);
    (*ip).xDC8_word.flags.set_x19(1);
    (*ip).xDD4_itemVar.whitebea.x3C = 1;
    (*ip).xDD4_itemVar.whitebea.x40 = 0;
    it_802E3DA0(gobj);
}

pub unsafe extern "C" fn it_802E3884(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802E3884");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWhiteBeaAttributes;
    (*ip).init_facing_dir = (*ip).facing_dir;
    (*ip).xC9C = ((*ip).xC9C as f32 + ext::it_8027CBFC(gobj)) as i32;
    if (*ip).xC9C > (*(*attr).x0).x0 || (*ip).msid == 9 {
        Item_ZakoDefeat(gobj, ip);
        if ext::HSD_Randf() < (*ext::it_804D6D40).x8 {
            it_802E4980(gobj);
        } else {
            it_802E48B4(gobj);
        }
    } else {
        it_802E3ED0(gobj);
    }
    false
}

pub unsafe extern "C" fn fn_802E398C(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802E398C");
    ext::it_8027CAD8(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion0_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itWhitebea_UnkMotion0_Phys");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.whitebea.x40 == 0 {
        let rand = ext::HSD_Randi(2);
        if (*ip).xDD4_itemVar.whitebea.x3C != 0 || rand != 0 {
            it_802E3AC8(gobj);
            return;
        }
        it_802E436C(gobj);
    } else {
        (*ip).xDD4_itemVar.whitebea.x40 = (*ip).xDD4_itemVar.whitebea.x40.wrapping_sub(1);
    }
}

pub unsafe extern "C" fn itWhitebea_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_802E3DA0));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E3AC8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E3AC8");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWhiteBeaAttributes;
    (*ip).x40_vel.x = (*ip).facing_dir * (*(*attrs).x0).x4;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    let facing = facing_i((*ip).facing_dir);
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
    ext::it_802756E0(gobj);
    let timer = (*ip).xDD4_itemVar.whitebea.x3C;
    if timer < 2 || timer > (*attrs).xE as i32 {
        let range = abs_i((*attrs).xE as i32 - (*attrs).xC as i32);
        (*ip).xDD4_itemVar.whitebea.x3C = ((*attrs).xC as i32).wrapping_add(ext::HSD_Randi(range));
    }
    (*ip).xDD4_itemVar.whitebea.x44 = 4;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion1_Anim");
    let ip = GET_ITEM(gobj);
    let mut local: i32 = 0;
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE | ITEM_HIT_PRESERVE);
    }
    if (*ip).xDD4_itemVar.whitebea.x44 <= 0 {
        let sfx = ext::mpLib_80056A1C(ext::mpLineGetFlags((*ip).xC30) as i32, &mut local);
        (*ip).xDD4_itemVar.whitebea.x44 = 14;
        ext::Item_8026AE84(ip, 0x13C, 0x7F, 0x40);
        if *sfx != -1 {
            ext::Item_8026AE84(ip, *sfx, 0x7F, 0x40);
        }
    } else {
        (*ip).xDD4_itemVar.whitebea.x44 = (*ip).xDD4_itemVar.whitebea.x44.wrapping_sub(1);
    }
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itWhitebea_UnkMotion1_Phys");
    let ip = (*gobj).user_data as *mut Item;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWhiteBeaAttributes;

    if (*ip).xDD4_itemVar.whitebea.x3C == 0 {
        let range = abs_i((*attrs).xA as i32 - (*attrs).x8 as i32);
        (*ip).xDD4_itemVar.whitebea.x40 = ((*attrs).x8 as i32).wrapping_add(ext::HSD_Randi(range));
        ext::it_8027CAD8(gobj);
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    } else {
        (*ip).xDD4_itemVar.whitebea.x3C = (*ip).xDD4_itemVar.whitebea.x3C.wrapping_sub(1);
    }
    if ext::it_802750E8(gobj, 2) != 0 {
        it_802E4190(gobj);
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion1_Coll");
    if ext::it_8026D8A4(gobj, Some(it_802E4190)) != 0 {
        if ext::it_80276308(gobj) != 0 {
            it_802E4190(gobj);
        }
    } else {
        it_802E3DA0(gobj);
    }
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E3DA0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E3DA0");
    let ip = (*gobj).user_data as *mut Item;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWhiteBeaAttributes;
    (*ip).xDD4_itemVar.whitebea.x40 = 0;
    (*ip).x40_vel.x *= (*attrs).x4;
    if ((*ip).facing_dir > 0.0 && (*ip).x70_nudge.x < 0.0) || ((*ip).facing_dir < 0.0 && (*ip).x70_nudge.x > 0.0) {
        (*ip).facing_dir = -(*ip).facing_dir;
        let int_dir = if -1.0 == (*ip).facing_dir { -1 } else { 1 };
        ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, int_dir);
    }
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion3_Anim");
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itWhitebea_UnkMotion3_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion3_Coll");
    ext::it_8026E414(gobj, Some(fn_802E398C));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E3ED0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E3ED0");
    let ip = GET_ITEM(gobj);
    ext::it_802762BC(ip);
    if ext::it_8027B798(gobj, &mut (*ip).x40_vel) != 0 {
        if (*ip).x40_vel.y as f64 <= 0.2 {
            (*ip).x40_vel.y = 0.2;
        } else {
            (*ip).x40_vel.y *= 0.2;
        }
        ext::it_802762BC(ip);
    } else {
        ext::it_802762B0(ip);
        (*ip).x40_vel.x *= 0.2;
    }
    (*ip).facing_dir = (*ip).init_facing_dir;
    let facing = facing_i((*ip).facing_dir);
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
    ext::Item_80268E5C(gobj, 4, 3);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion4_Anim");
    let ip = GET_ITEM(gobj);

    if ext::it_80272C6C(gobj) == 0 {
        if (*ip).ground_or_air == GA_Air {
            ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
        } else {
            it_802E40A4(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itWhitebea_UnkMotion4_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
    }
}

pub unsafe extern "C" fn itWhitebea_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion4_Coll");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        ext::it_8026E414(gobj, Some(it_802E40A4));
    } else {
        ext::it_8026D62C(gobj, Some(it_802E3DA0));
    }
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E40A4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E40A4");
    let ip = HSD_GObjGetUserData(gobj) as *mut Item;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWhiteBeaAttributes;
    (*ip).x40_vel.x = (*ip).facing_dir * (*(*attrs).x0).x4;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    let facing = facing_i((*ip).facing_dir);
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
    ext::it_802756E0(gobj);
    if 2 > (*ip).xDD4_itemVar.whitebea.x3C || (*ip).xDD4_itemVar.whitebea.x3C > (*attrs).xE as i32 {
        let range = abs_i((*attrs).xE as i32 - (*attrs).xC as i32);
        (*ip).xDD4_itemVar.whitebea.x3C = ((*attrs).xC as i32).wrapping_add(ext::HSD_Randi(range));
    }
    (*ip).xDD4_itemVar.whitebea.x44 = 4;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn it_802E4190(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E4190");
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
    (*ip).xDD4_itemVar.whitebea.x40 = 0xF;
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion2_Anim");
    let mut ip = HSD_GObjGetUserData(gobj) as *mut Item;
    if (*ip).xDD4_itemVar.whitebea.x40 == 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        let facing = facing_i((*ip).facing_dir);
        ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
    }
    (*ip).xDD4_itemVar.whitebea.x40 = (*ip).xDD4_itemVar.whitebea.x40.wrapping_sub(1);
    if ext::it_80272C6C(gobj) == 0 {
        (*ip).xDD4_itemVar.whitebea.x40 = 0;
        ip = HSD_GObjGetUserData(gobj) as *mut Item;
        let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWhiteBeaAttributes;
        (*ip).x40_vel.x = (*ip).facing_dir * (*(*attrs).x0).x4;
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.y = 0.0;
        ext::it_8027C56C(gobj, (*ip).facing_dir);
        let facing = facing_i((*ip).facing_dir);
        ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
        ext::it_802756E0(gobj);
        if 2 > (*ip).xDD4_itemVar.whitebea.x3C || (*ip).xDD4_itemVar.whitebea.x3C > (*attrs).xE as i32 {
            let range = abs_i((*attrs).xE as i32 - (*attrs).xC as i32);
            (*ip).xDD4_itemVar.whitebea.x3C = ((*attrs).xC as i32).wrapping_add(ext::HSD_Randi(range));
        }
        (*ip).xDD4_itemVar.whitebea.x44 = 4;
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itWhitebea_UnkMotion2_Phys");}

pub unsafe extern "C" fn itWhitebea_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion2_Coll");
    ext::it_8026D62C(gobj, Some(it_802E3DA0));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E436C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E436C");
    let ip = GET_ITEM(gobj);
    ext::it_802762B0(ip);
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    (*ip).xDD4_itemVar.whitebea.x40 = 0x14;
}

pub unsafe extern "C" fn itWhitebea_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion5_Anim");
    let ip = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 || (*ip).xDD4_itemVar.whitebea.x40 == 0 {
        it_802E4464(gobj);
    } else {
        (*ip).xDD4_itemVar.whitebea.x40 = (*ip).xDD4_itemVar.whitebea.x40.wrapping_sub(1);
    }
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion5_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itWhitebea_UnkMotion5_Phys");}

pub unsafe extern "C" fn itWhitebea_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion5_Coll");
    ext::it_8026D6F4(gobj, Some(it_802E3DA0));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E4464(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E4464");
    let ip = (*gobj).user_data as *mut Item;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWhiteBeaAttributes;
    ext::it_802762BC(ip);
    (*ip).x40_vel.y = (*attrs).x10;
    ext::it_80275258(gobj);
    ext::Item_8026AE84(ip, 0x4A, 0x7F, 0x40);
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion6_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion6_Anim");
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itWhitebea_UnkMotion6_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion6_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion6_Coll");
    ext::it_8026E414(gobj, Some(it_802E4558));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E4558(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E4558");
    let ip = (*gobj).user_data as *mut Item;
    ext::it_802762B0(ip);
    ext::Item_8026AE84(ip, 0x46, 0x7F, 0x40);
    ext::Item_8026AE84(ip, 0x13C, 0x7F, 0x40);
    ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE);
    ext::grIceMt_801FA6D8();
}

pub unsafe extern "C" fn itWhitebea_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion7_Anim");
    let ip = (*gobj).user_data as *mut Item;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWhiteBeaAttributes;
    if ext::it_80272C6C(gobj) == 0 {
        (*ip).xDD4_itemVar.whitebea.x3C = 1;
        (*ip).xDD4_itemVar.whitebea.x40 = (*attrs).x14 as i32;
        ext::it_8027CAD8(gobj);
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion7_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itWhitebea_UnkMotion7_Phys");}

pub unsafe extern "C" fn itWhitebea_UnkMotion7_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion7_Coll");
    ext::it_8026D6F4(gobj, Some(it_802E3DA0));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn itWhiteBea_Logic9_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itWhiteBea_Logic9_PickedUp");
    ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion8_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion8_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion8_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itWhitebea_UnkMotion8_Phys");}

pub unsafe extern "C" fn it_2725_Logic9_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic9_Dropped");
    ext::it_8027CBA4(gobj);
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWhiteBeaAttributes;
    (*ip).xDD4_itemVar.whitebea.x40 = 0;
    (*ip).x40_vel.x *= (*attrs).x4;
    if ((*ip).facing_dir > 0.0 && (*ip).x70_nudge.x < 0.0) || ((*ip).facing_dir < 0.0 && (*ip).x70_nudge.x > 0.0) {
        (*ip).facing_dir = -(*ip).facing_dir;
        let int_dir = if -1.0 == (*ip).facing_dir { -1 } else { 1 };
        ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, int_dir);
    }
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn it_2725_Logic9_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic9_Thrown");
    let ip = GET_ITEM(gobj);
    ext::it_8027CB3C(gobj);
    (*ip).xDD4_itemVar.whitebea.x40 = 0;
    (*ip).xDD4_itemVar.whitebea.x3C = 1;
    ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion9_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion9_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion9_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itWhitebea_UnkMotion9_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion9_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion9_Coll");
    ext::it_8027C824(gobj, None) != 0
}

pub unsafe extern "C" fn it_802E48B4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E48B4");
    let ip = GET_ITEM(gobj);
    ext::it_802762BC(ip);
    ext::it_8027B964(gobj, 0);
    (*ip).xDD4_itemVar.whitebea.x18 = 0;
    ext::Item_80268E5C(gobj, 0xA, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion10_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion10_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0xA, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion10_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itWhitebea_UnkMotion10_Phys");}

pub unsafe extern "C" fn itWhitebea_UnkMotion10_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion10_Coll");
    ext::it_8027C79C(gobj) != 0
}

pub unsafe extern "C" fn it_802E4980(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E4980");
    let ip = GET_ITEM(gobj);
    Item_UpdateZakoVelocity(gobj, ip);
    ext::Item_80268E5C(gobj, 0xB, 3);
}

pub unsafe extern "C" fn itWhitebea_UnkMotion11_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion11_Anim");
    false
}

pub unsafe extern "C" fn itWhitebea_UnkMotion11_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itWhitebea_UnkMotion11_Phys");
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
}

pub unsafe extern "C" fn itWhitebea_UnkMotion11_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itWhitebea_UnkMotion11_Coll");
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E4A24(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E4A24");
    ext::it_8026B894(gobj, ref_gobj);
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802E3784` (src/melee/it/kinds/itwhitebea.c): `it_8026B894(..)`.
pub fn it_802e3784(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802e3784");
    it_8026b894(ip, ref_gobj);
}

/// `it_802E4A24` (src/melee/it/kinds/itwhitebea.c): `it_8026B894(..)`.
pub fn it_802e4a24(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802e4a24");
    it_8026b894(ip, ref_gobj);
}

// ---- folded from itkinds5.rs (consolidation steps 21/22) ----
/// `it_802E37A4` (itwhitebea.c): `freeze.x20 = NULL` if there is an item.
pub fn it_802e37a4(ip: Option<&mut Item>) {
    melee_core::ct!("it_802e37a4");
    if let Some(ip) = ip {
        ip.xDD4_itemVar.freeze.x20 = null_mut_itkinds5();
    }
}
