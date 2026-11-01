//! Port of src/melee/it/kinds/itoctarock.c: the Octorok. Callees of unported TUs go through `ext` (unset: panics
//! "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itoctarock.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::helpers::it_80275158;
#[allow(unused_imports)]
use crate::helpers::it_rshell_clamp_vel;
#[allow(unused_imports)]
use crate::helpers::itRShell_Attrs;
#[allow(unused_imports)]
use crate::item::decelerate_item_x;
#[allow(unused_imports)]
use crate::item::it_802762bc;
#[allow(unused_imports)]
use crate::itlib::it_80275444;
#[allow(unused_imports)]
use crate::itlib::it_802756d0;
#[allow(unused_imports)]
use crate::itlib::it_8027572c;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_802756e0;
#[allow(unused_imports)]
use crate::itlib3::it_coll_bounce_off_victim;
#[allow(unused_imports)]
use crate::layout::itClimbersIceAttributes;
#[allow(unused_imports)]
use crate::layout::itCoinAttributes;
#[allow(unused_imports)]
use crate::layout::itDoseiAttributes;
#[allow(unused_imports)]
use crate::layout::itFireAttributes;
#[allow(unused_imports)]
use crate::layout::itFlipper_DatAttrs;
#[allow(unused_imports)]
use crate::layout::itFreezerAttributes;
#[allow(unused_imports)]
use crate::layout::itGamewatchchefAttributes;
#[allow(unused_imports)]
use crate::layout::itHammerheadAttributes;
#[allow(unused_imports)]
use crate::layout::itHitodemanAttributes;
#[allow(unused_imports)]
use crate::layout::itHououAttr;
#[allow(unused_imports)]
use crate::layout::itLeadeadAttributes;
#[allow(unused_imports)]
use crate::layout::itLinkArrowAttributes;
#[allow(unused_imports)]
use crate::layout::itLinkBombAttributes;
#[allow(unused_imports)]
use crate::layout::itLizardonAttributes;
#[allow(unused_imports)]
use crate::layout::itMatadogasAttributes;
#[allow(unused_imports)]
use crate::layout::itPeachToadSporeAttributes;
#[allow(unused_imports)]
use crate::layout::itSamusMissileAttributes;
#[allow(unused_imports)]
use crate::layout::itsonansAttributes;
#[allow(unused_imports)]
use crate::layout::HeartContainerAttr;
#[allow(unused_imports)]
use crate::layout::ItemAttr;
#[allow(unused_imports)]
use crate::layout::KinokoAttrs;
#[allow(unused_imports)]
use crate::layout::MewVars;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::GA_Air;
#[allow(unused_imports)]
use crate::layout::GA_Ground;
#[allow(unused_imports)]
use crate::layout::It_Kind_Lugia_Aeroblast;
#[allow(unused_imports)]
use crate::layout::It_Kind_Lugia_Aeroblast2;
#[allow(unused_imports)]
use crate::layout::It_Kind_Lugia_Aeroblast3;
#[allow(unused_imports)]
use crate::layout::It_Kind_Matadogas_Gas1;
#[allow(unused_imports)]
use crate::layout::It_Kind_Matadogas_Gas2;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds;
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn ftLib_8008627C(a0: *mut Vec, a1: *mut HSD_GObj) -> *mut HSD_GObj;
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::ftlib::ftLib_80086644(a0, a1) };
    fn it_8026B684(a0: *mut Vec) -> f32 => unsafe { crate::tu::it_26B1::it_8026B684(a0 as _) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D5CC(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D5CC(a0) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026DA70(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA70(a0) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80275270(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275270(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027542C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8027542c(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275474(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275474(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_802763E0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::helpers::it_802763e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027b730(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B798(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027B798(a0, a1 as _) as _ };
    fn it_8027B964(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itzako::it_8027B964(a0, a1) };
    fn it_8027BA54(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027BA54(a0, a1 as _) };
    fn it_8027C0A8(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::tu::itzako::it_8027C0A8(a0, a1, a2) };
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
    fn it_802E8ADC(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32, a3: f32, a4: f32, a5: f32) => unsafe { crate::tu::itoctarockstone::it_802E8ADC(a0 as _, a1 as _, a2, a3, a4, a5) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D40;
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `ItemStateTable it_803F8B48[]`
pub static mut it_803F8B48: [ItemStateTable; 9] = [
    st_row(0, pred_b!(itOctarock_UnkMotion0_Anim), Some(itOctarock_UnkMotion0_Phys), pred_b!(itOctarock_UnkMotion0_Coll)),
    st_row(1, pred_b!(itOctarock_UnkMotion1_Anim), Some(itOctarock_UnkMotion1_Phys), pred_b!(itOctarock_UnkMotion1_Coll)),
    st_row(-1, pred_b!(itOctarock_UnkMotion2_Anim), Some(itOctarock_UnkMotion2_Phys), pred_b!(itOctarock_UnkMotion2_Coll)),
    st_row(3, pred_b!(itOctarock_UnkMotion3_Anim), Some(itOctarock_UnkMotion3_Phys), pred_b!(itOctarock_UnkMotion3_Coll)),
    st_row(2, pred_b!(itOctarock_UnkMotion4_Anim), Some(itOctarock_UnkMotion4_Phys), pred_b!(itOctarock_UnkMotion4_Coll)),
    st_row(3, pred_b!(itOctarock_UnkMotion5_Anim), Some(itOctarock_UnkMotion5_Phys), None),
    st_row(4, pred_b!(itOctarock_UnkMotion6_Anim), Some(itOctarock_UnkMotion6_Phys), pred_b!(itOctarock_UnkMotion6_Coll)),
    st_row(3, pred_b!(itOctarock_UnkMotion7_Anim), Some(itOctarock_UnkMotion7_Phys), pred_b!(itOctarock_UnkMotion7_Coll)),
    st_row(3, pred_b!(itOctarock_UnkMotion8_Anim), Some(itOctarock_UnkMotion8_Phys), pred_b!(itOctarock_UnkMotion8_Coll)),
];

/// `itOctarock_SetFacingDir` (static inline)
#[inline]
unsafe fn itOctarock_SetFacingDir(ip: *mut Item) {
    let dir = if (*ip).facing_dir == -1.0 { -1 } else { 1 };
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, dir);
}

/// `Item_InitZakoCollision` (kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitZakoCollision(gobj: *mut Item_GObj, ip: *mut Item) {
    (*ip).xD5C = 0;
    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_8027542C(gobj);
    ext::it_80275270(gobj);
    (*ip).xDC8_word.flags.set_x19(1);
}

/// `Item_ZakoDefeat` (kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_ZakoDefeat(gobj: *mut Item_GObj, ip: *mut Item) {
    ext::it_8027C9D8(ip);
    ext::it_802756D0(gobj);
    ext::it_80275474(gobj);
    ext::it_8027CE44(gobj);
    ext::Camera_RequestQuake(QuakeKind_Small, &mut (*ip).pos);
}

/// `Item_UpdateZakoVelocity` (kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_UpdateZakoVelocity(gobj: *mut Item_GObj, ip: *mut Item) {
    ext::it_802762BC(ip);
    ext::it_8027BA54(gobj, &mut (*ip).x40_vel);
    ext::it_802762BC(ip);
}

/// `itUpdateVelocityFromBone` (kinds/inlines.h, static inline)
#[inline]
unsafe fn itUpdateVelocityFromBone(jobj: *mut HSD_JObj, ip: *mut Item, previous_pos: *mut Vec3) {
    let mut pos: Vec3 = core::mem::zeroed();
    let mut zero: Vec3 = core::mem::zeroed();

    if !jobj.is_null() {
        zero.z = 0.0;
        zero.y = 0.0;
        zero.x = 0.0;
        HSD_JObjGetTranslation(jobj, &mut pos);
        (*ip).x40_vel.x = (*ip).facing_dir * (pos.z - (*previous_pos).z);
        (*ip).x40_vel.y = pos.y - (*previous_pos).y;
        (*ip).x40_vel.z = pos.x - (*previous_pos).x;
        *previous_pos = pos;
        HSD_JObjSetTranslate(jobj, &mut zero);
    }
}

pub unsafe extern "C" fn it_802E4A44(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E4A44");
    let ip = GET_ITEM(gobj);
    ext::it_8027B730(gobj);
    (*ip).facing_dir = ext::it_8026B684(&mut (*ip).pos);
    itOctarock_SetFacingDir(ip);
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    Item_InitZakoCollision(gobj, ip);
    (*ip).xDD4_itemVar.octarock.x32 = 0;
    it_802E4DB4(gobj);
}

pub unsafe extern "C" fn it_802E4B00(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802E4B00");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOctarockAttributes;
    (*ip).init_facing_dir = (*ip).facing_dir;
    (*ip).xC9C = ((*ip).xC9C as f32 + ext::it_8027CBFC(gobj)) as i32;
    if (*ip).xC9C > *(*attr).x0 || (*ip).msid == 6 {
        Item_ZakoDefeat(gobj, ip);
        if ext::HSD_Randf() < (*ext::it_804D6D40).x8 {
            it_802E58A0(gobj);
        } else {
            it_802E57D4(gobj);
        }
    } else {
        it_802E4E6C(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802E4C08(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E4C08");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOctarockAttributes;
    ext::it_8027CAD8(gobj);
    (*ip).xDD4_itemVar.octarock.x32 = (*attr).x1C;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOctarock_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion0_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOctarock_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarock_UnkMotion0_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOctarockAttributes;
    if (*ip).xDD4_itemVar.octarock.x32 == 0 {
        match it_802E52E0(gobj) {
            0 => (*ip).xDD4_itemVar.octarock.x32 = (*attr).x1C,
            1 => it_802E53C8(gobj),
            2 => it_802E503C(gobj),
            _ => {}
        }
    } else {
        (*ip).xDD4_itemVar.octarock.x32 = (*ip).xDD4_itemVar.octarock.x32.wrapping_sub(1);
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0A8(gobj, (*ip).facing_dir, 0.0);
}

pub unsafe extern "C" fn itOctarock_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_802E4DB4));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E4DB4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E4DB4");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOctarockAttributes;
    (*ip).x40_vel.x *= (*attr).x4;
    ext::Item_80268E5C(gobj, 2, 3);
}

pub unsafe extern "C" fn itOctarock_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion2_Anim");
    false
}

pub unsafe extern "C" fn itOctarock_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarock_UnkMotion2_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itOctarock_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion2_Coll");
    ext::it_8026E414(gobj, Some(it_802E4C08));
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E4E6C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E4E6C");
    let ip = GET_ITEM(gobj);
    if ext::it_8027B798(gobj, &mut (*ip).x40_vel) != 0 {
        if (*ip).x40_vel.y <= 0.2 {
            (*ip).x40_vel.y = 0.2;
        }
        ext::it_802762BC(ip);
    } else {
        ext::it_802762B0(ip);
    }
    (*ip).facing_dir = (*ip).init_facing_dir;
    itOctarock_SetFacingDir(ip);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOctarock_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion3_Anim");
    let ip = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        if (*ip).ground_or_air == GA_Air {
            ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
        } else {
            it_802E4C08(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn itOctarock_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarock_UnkMotion3_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
    }
}

pub unsafe extern "C" fn itOctarock_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion3_Coll");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        ext::it_8026E414(gobj, Some(it_802E4C08));
    } else {
        ext::it_8026D62C(gobj, Some(it_802E4DB4));
    }
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E503C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E503C");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.octarock.x32 = 0x15;
    itResetVelocity(ip);
    (*ip).xDD4_itemVar.octarock.x2C = 0;
    it_802E5944(gobj);
    ext::it_802762BC(ip);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOctarock_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion4_Anim");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOctarockAttributes;
    it_802E595C((*(*ip).xBBC_dynamicBoneTable).bones[1], ip);
    if (*ip).xDD4_itemVar.octarock.x32 == 0 {
        let mut v: Vec3 = core::mem::zeroed();
        ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[0xE], null_mut(), &mut v);
        ext::it_802E8ADC(gobj, &mut v, (*ip).facing_dir as i32, (*attr).x10, (*attr).x14, (*attr).x18);
        ext::Item_8026AE84(ip, 0x137, 0x7F, 0x40);
    }
    if ext::it_80272C6C(gobj) == 0 {
        ext::it_8026DA08(gobj);
        (*ip).xDD4_itemVar.octarock.x32 = 0;
        it_802E4C08(gobj);
    }
    (*ip).xDD4_itemVar.octarock.x32 = (*ip).xDD4_itemVar.octarock.x32.wrapping_sub(1);
    false
}

pub unsafe extern "C" fn itOctarock_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarock_UnkMotion4_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.octarock.x2C == 0 && (*ip).x40_vel.y > 0.0 {
        ext::it_802762BC(ip);
        (*ip).xDD4_itemVar.octarock.x2C = 1;
    }
}

pub unsafe extern "C" fn itOctarock_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion4_Coll");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Ground {
        if ext::it_8026D5CC(gobj) == 0 {
            it_802E4DB4(gobj);
        }
    } else if ext::it_8026DA70(gobj) != 0 {
        (*ip).xDD4_itemVar.octarock.x2C = 0;
    } else {
        ext::it_802763E0(gobj);
        (*ip).pos = (*ip).x378_itemColl.cur_pos;
    }
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E52E0(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802E52E0");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itOctarockAttributes;
    let mut result = 0;
    let fighter = ext::ftLib_8008627C(&mut (*ip).pos, null_mut());

    if !fighter.is_null() {
        let mut v: Vec3 = core::mem::zeroed();
        ext::ftLib_80086644(fighter, &mut v);
        let dx = v.x - (*ip).pos.x;
        if (if dx < 0.0 { -dx } else { dx }) <= (*attr).x8 {
            let dy = v.y - (*ip).pos.y;
            if (if dy < 0.0 { -dy } else { dy }) <= (*attr).xC {
                let facing = if (*ip).pos.x < v.x { 1.0 } else { -1.0 };
                if facing != (*ip).facing_dir {
                    result = 1;
                } else {
                    result = 2;
                }
            }
        }
    }
    result
}

pub unsafe extern "C" fn it_802E53C8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E53C8");
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
    (*ip).xDD4_itemVar.octarock.x32 = 0x16;
    (*ip).xDD4_itemVar.octarock.x2C = 0;
    it_802E5944(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOctarock_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion1_Anim");
    let ip = GET_ITEM(gobj);
    it_802E595C((*(*ip).xBBC_dynamicBoneTable).bones[1], ip);
    if (*ip).xDD4_itemVar.octarock.x32 == 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        itOctarock_SetFacingDir(ip);
    }
    if ext::it_80272C6C(gobj) == 0 {
        ext::it_8026D5CC(gobj);
        (*ip).xDD4_itemVar.octarock.x32 = 0;
        it_802E4C08(gobj);
        return false;
    }
    (*ip).xDD4_itemVar.octarock.x32 = (*ip).xDD4_itemVar.octarock.x32.wrapping_sub(1);
    false
}

pub unsafe extern "C" fn itOctarock_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarock_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Ground && (*ip).x40_vel.y > 0.0 {
        ext::it_802762BC(ip);
    }
}

pub unsafe extern "C" fn itOctarock_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion1_Coll");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Ground {
        if ext::it_8026D5CC(gobj) == 0 {
            it_802E4DB4(gobj);
        }
    } else if ext::it_8026DA70(gobj) != 0 {
        ext::it_802762B0(ip);
    } else {
        ext::it_802763E0(gobj);
        (*ip).pos = (*ip).x378_itemColl.cur_pos;
    }
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn itOctarock_Logic2_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarock_Logic2_PickedUp");
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOctarock_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion5_Anim");
    let _ip = GET_ITEM(gobj); // NOTE: unused
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOctarock_UnkMotion5_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itOctarock_UnkMotion5_Phys");}

pub unsafe extern "C" fn itOctarock_Logic2_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarock_Logic2_Dropped");
    ext::it_8027CBA4(gobj);
    it_802E4DB4(gobj);
}

pub unsafe extern "C" fn itOctarock_Logic2_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarock_Logic2_Thrown");
    let ip = GET_ITEM(gobj);
    ext::it_8027CB3C(gobj);
    (*ip).xDD4_itemVar.octarock.x32 = 0;
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOctarock_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion6_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOctarock_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarock_UnkMotion6_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itOctarock_UnkMotion6_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion6_Coll");
    ext::it_8027C824(gobj, None) != 0
}

pub unsafe extern "C" fn it_802E57D4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E57D4");
    let ip = GET_ITEM(gobj);
    ext::it_802762BC(ip);
    ext::it_8027B964(gobj, 0);
    (*ip).xDD4_itemVar.octarock.x18 = 0;
    ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOctarock_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion7_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itOctarock_UnkMotion7_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itOctarock_UnkMotion7_Phys");}

pub unsafe extern "C" fn itOctarock_UnkMotion7_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion7_Coll");
    ext::it_8027C79C(gobj) != 0
}

pub unsafe extern "C" fn it_802E58A0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E58A0");
    let ip = GET_ITEM(gobj);
    Item_UpdateZakoVelocity(gobj, ip);
    ext::Item_80268E5C(gobj, 8, 3);
}

pub unsafe extern "C" fn itOctarock_UnkMotion8_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion8_Anim");
    false
}

pub unsafe extern "C" fn itOctarock_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarock_UnkMotion8_Phys");
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
}

pub unsafe extern "C" fn itOctarock_UnkMotion8_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarock_UnkMotion8_Coll");
    ext::it_8027C794(gobj) != 0
}

pub unsafe extern "C" fn it_802E5944(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E5944");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.octarock.x20.x = 0.0;
    (*ip).xDD4_itemVar.octarock.x20.y = 0.0;
    (*ip).xDD4_itemVar.octarock.x20.z = 0.0;
}

pub unsafe extern "C" fn it_802E595C(jobj: *mut HSD_JObj, ip: *mut Item) {
    melee_core::ct!("it_802E595C");
    itUpdateVelocityFromBone(jobj, ip, &mut (*ip).xDD4_itemVar.octarock.x20);
}

pub unsafe extern "C" fn it_802E5AA4(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E5AA4");
    ext::it_8026B894(gobj, ref_gobj);
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802E5AA4` (src/melee/it/kinds/itoctarock.c): `it_8026B894(..)`.
pub fn it_802e5aa4(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802e5aa4");
    it_8026b894(ip, ref_gobj);
}

// ---- folded from itkinds.rs (consolidation steps 21/22) ----
/// `it_802E5944` (itoctarock.c): `octarock.x20 = 0`.
pub fn it_802e5944(ip: &mut Item) {
    melee_core::ct!("it_802e5944");
    unsafe {
        let v = &mut ip.xDD4_itemVar.octarock.x20;
        v.x = 0.0;
        v.y = 0.0;
        v.z = 0.0;
    }
}
