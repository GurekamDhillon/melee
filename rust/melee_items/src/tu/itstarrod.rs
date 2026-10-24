//! Port of src/melee/it/kinds/itstarrod.c: the Star Rod item (logic table 22). `xD4C` is the remaining
//! number of shots. Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C
//! (whole TU, shipping contraction) by tests/kinds_itstarrod.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
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
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DF34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DF34(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802988E4(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32, a3: f32) => unsafe { crate::tu::itstarrodstar::it_802988E4(a0 as _, a1, a2, a3) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `StarRodAttributes` (local to itstarrod.c)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StarRodAttributes {
    pub x0: i32,
    pub x4: Vec3,
}

/// `Item_ApplyFallingPhysics` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

/// `ItemStateTable it_803F5F90[]`
pub static mut it_803F5F90: [ItemStateTable; 6] = [
    st_row(-1, pred_b(itStarrod_UnkMotion0_Anim), Some(itStarrod_UnkMotion0_Phys), pred_b(itStarrod_UnkMotion0_Coll)),
    st_row(-1, pred_b(itStarrod_UnkMotion4_Anim), Some(itStarrod_UnkMotion1_Phys), pred_b(itStarrod_UnkMotion1_Coll)),
    st_row(-1, pred_b(itStarrod_UnkMotion2_Anim), Some(itStarrod_UnkMotion2_Phys), None),
    st_row(0, pred_b(itStarrod_UnkMotion4_Anim), Some(itStarrod_UnkMotion4_Phys), pred_b(itStarrod_UnkMotion3_Coll)),
    st_row(0, pred_b(itStarrod_UnkMotion4_Anim), Some(itStarrod_UnkMotion4_Phys), pred_b(itStarrod_UnkMotion4_Coll)),
    st_row(-1, pred_b(itStarrod_UnkMotion5_Anim), Some(itStarrod_UnkMotion5_Phys), pred_b(itStarrod_UnkMotion5_Coll)),
];

pub unsafe extern "C" fn itStarRod_Logic22_Spawned(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let x = (*(*ip).xC4_article_data).x4_specialAttributes as *mut StarRodAttributes;
    (*ip).xD4C = (*x).x0;
    it_80292488(gobj);
}

/// 292394
pub unsafe extern "C" fn it_80292394(gobj: *mut Item_GObj, arg1: *mut Vec) {
    let ip = GET_ITEM(gobj);
    let x = (*(*ip).xC4_article_data).x4_specialAttributes as *mut StarRodAttributes;
    *arg1 = (*x).x4;
}

/// 2923BC
pub unsafe extern "C" fn it_802923BC(gobj: *mut Item_GObj, arg1: *mut Vec, arg2: f32, arg3: i32) {
    let ip = GET_ITEM(gobj);
    if (*ip).xD4C > 0 {
        (*ip).xD4C -= 1;
    }
    ext::it_802988E4((*ip).owner, arg1, arg3, arg2);
}

/// 2923F8
pub unsafe extern "C" fn it_802923F8(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8026B390(gobj);
    itResetVelocity(ip);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itStarrod_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itStarrod_UnkMotion0_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itStarrod_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_80292488));
    false
}

/// 292488
pub unsafe extern "C" fn it_80292488(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itStarrod_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itStarrod_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let x = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*x).x10_fall_speed, (*x).x14_fall_speed_max);
}

pub unsafe extern "C" fn itStarrod_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_802923F8));
    false
}

pub unsafe extern "C" fn itStarRod_Logic22_PickedUp(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itStarrod_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itStarrod_UnkMotion2_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itStarRod_Logic22_Dropped(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
}

pub unsafe extern "C" fn itStarrod_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xD4C != 0 {
        ext::it_8026E15C(gobj, Some(it_802923F8));
        return false;
    }
    ext::it_8026DF34(gobj) != 0
}

pub unsafe extern "C" fn itStarRod_Logic22_Thrown(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
}

pub unsafe extern "C" fn itStarrod_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itStarrod_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xD4C != 0 {
        ext::it_8026E15C(gobj, Some(it_802923F8));
        return false;
    }
    ext::it_8026DF34(gobj) != 0
}

pub unsafe extern "C" fn itStarRod_Logic22_DmgDealt(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 || (*ip).msid == 4 {
        ext::itColl_BounceOffVictim(gobj);
    }
    false
}

pub unsafe extern "C" fn itStarRod_Logic22_EnteredAir(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itStarrod_UnkMotion5_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itStarrod_UnkMotion5_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itStarrod_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E8C4(gobj, Some(it_802923F8), Some(it_80292488));
    false
}

pub unsafe extern "C" fn itStarRod_Logic22_Clanked(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 || (*ip).msid == 4 {
        ext::itColl_BounceOffVictim(gobj);
    }
    false
}

pub unsafe extern "C" fn itStarRod_Logic22_Reflected(gobj: *mut Item_GObj) -> bool {
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn itStarRod_Logic22_HitShield(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 || (*ip).msid == 4 {
        ext::itColl_BounceOffVictim(gobj);
    }
    false
}

pub unsafe extern "C" fn itStarRod_Logic22_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn itStarRod_Logic22_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
