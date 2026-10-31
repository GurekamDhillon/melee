//! Port of src/melee/it/kinds/itcapsule.c: the capsule (logic table 0; opens into an item or explodes).
//! `capsule.x0` is the "opened an item" result of `it_8027CF30`, `capsule.x4` the "already exploded"
//! flag. Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C
//! (whole TU, shipping contraction) by tests/kinds_itcapsule.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::kinds_inl::*;

// // BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_8026F3D4(a0: *mut HSD_GObj, a1: *mut it_8026F3D4_arg1_t, a2: i32, a3: i32) -> i32 => unsafe { crate::tu::itdrop::it_8026F3D4(a0, a1, a2, a3) };
    fn it_8026F8B4(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: i32) -> i32 => unsafe { crate::tu::itdrop::it_8026F8B4(a0, a1 as _, a2 as _, a3) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C08(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272C08(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn lb_800119DC(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32) => unsafe { melee_runtime::tu::lb_00F9::lb_800119DC(a0, a1, a2, a3, a4) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

/// `ItemStateTable it_803F5468[]`
pub static mut it_803F5468: [ItemStateTable; 7] = [
    st_row(-1, pred_b(itCapsule_UnkMotion0_Anim), Some(itCapsule_UnkMotion0_Phys), pred_b(itCapsule_UnkMotion0_Coll)),
    st_row(-1, pred_b(itCapsule_UnkMotion4_Anim), Some(itCapsule_UnkMotion4_Phys), pred_b(itCapsule_UnkMotion4_Coll)),
    st_row(-1, pred_b(itCapsule_UnkMotion2_Anim), Some(itCapsule_UnkMotion2_Phys), None),
    st_row(0, pred_b(itCapsule_UnkMotion4_Anim), Some(itCapsule_UnkMotion3_Phys), pred_b(itCapsule_UnkMotion3_Coll)),
    st_row(0, pred_b(itCapsule_UnkMotion4_Anim), Some(itCapsule_UnkMotion4_Phys), pred_b(itCapsule_UnkMotion4_Coll)),
    st_row(1, pred_b(itCapsule_UnkMotion5_Anim), Some(itCapsule_UnkMotion5_Phys), pred_b(itCapsule_UnkMotion5_Coll)),
    st_row(-1, pred_b(itCapsule_UnkMotion6_Anim), Some(itCapsule_UnkMotion6_Phys), pred_b(itCapsule_UnkMotion6_Coll)),
];

pub unsafe extern "C" fn itCapsule_Logic0_Spawned(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itCapsule_Logic0_Spawned");
    let it = GET_ITEM(item_gobj);
    (*it).xDD4_itemVar.capsule.x0 = 0;
    (*it).xDD4_itemVar.capsule.x4 = 0;
    it_8027D148(item_gobj);
}

/// 27CF30: tries to open the capsule into an item.
pub unsafe extern "C" fn it_8027CF30(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8027CF30");
    let it = GET_ITEM(gobj);
    let article = (*it).xC4_article_data;
    let attr = (*article).x4_specialAttributes as *mut ItCapsuleAttr;
    let mut sp18: Vec3 = core::mem::zeroed();

    sp18.z = 0.0;
    sp18.y = 0.0;
    sp18.x = 0.0;
    if ext::it_8026F8B4(gobj, &mut (*it).pos, &mut sp18, 0) == 0 {
        if ext::HSD_Randi((*attr).x4) == 0 {
            it_8027D2DC(gobj);
            (*it).xDD4_itemVar.capsule.x0 = 0;
        } else {
            ext::it_8026F3D4(gobj, null_mut(), (*attr).x0, 0);
            (*it).xDD4_itemVar.capsule.x0 = 1;
        }
    } else {
        (*it).xDD4_itemVar.capsule.x0 = 1;
    }
}

/// 27CFE8: explodes.
pub unsafe extern "C" fn it_8027CFE8(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_8027CFE8");
    let jobj = GET_JOBJ(item_gobj);
    let it = GET_ITEM(item_gobj);

    ext::it_8026B3A8(item_gobj);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*it).x5D0_animFrameSpeed = 1.0;
    ext::it_80273454(item_gobj);
    (*it).xDD4_itemVar.capsule.x4 = 1;
    ext::it_80272C08(item_gobj);
    ext::it_80275444(item_gobj);
    ext::it_802756D0(item_gobj);
    ext::it_8026BD24(item_gobj);
    ext::it_8027518C(item_gobj);
    let mut sp14: Vec3 = (*it).pos;
    ext::lb_800119DC(&mut sp14, 0x78, 1.0, 0.02, (60.0f64 * core::f64::consts::PI / 180.0) as f32);
    ext::Item_80268E5C(item_gobj, 5, 2);
}

/// 27D0B8
pub unsafe extern "C" fn it_8027D0B8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8027D0B8");
    let it = GET_ITEM(gobj);
    ext::it_8026B390(gobj);
    (*it).x40_vel.z = 0.0;
    (*it).x40_vel.y = 0.0;
    (*it).x40_vel.x = 0.0;
    ext::Item_80268E5C(gobj, 0, 2);
}

pub unsafe extern "C" fn itCapsule_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_UnkMotion0_Anim");
    false
}

pub unsafe extern "C" fn itCapsule_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itCapsule_UnkMotion0_Phys");}

pub unsafe extern "C" fn itCapsule_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_8027D148));
    false
}

/// 27D148
pub unsafe extern "C" fn it_8027D148(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8027D148");
    ext::Item_80268E5C(gobj, 1, 2);
}

pub unsafe extern "C" fn itCapsule_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_UnkMotion4_Anim");
    false
}

pub unsafe extern "C" fn itCapsule_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itCapsule_UnkMotion4_Phys");
    let it = GET_ITEM(gobj);
    ext::it_80272860(gobj, (*(*it).xCC_item_attr).x10_fall_speed, (*(*it).xCC_item_attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itCapsule_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_UnkMotion4_Coll");
    ext::it_8026E15C(gobj, Some(it_8027D0B8));
    false
}

pub unsafe extern "C" fn itCapsule_Logic0_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itCapsule_Logic0_PickedUp");
    ext::Item_80268E5C(gobj, 2, 2);
}

pub unsafe extern "C" fn itCapsule_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_UnkMotion2_Anim");
    false
}

pub unsafe extern "C" fn itCapsule_UnkMotion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itCapsule_UnkMotion2_Phys");}

pub unsafe extern "C" fn itCapsule_Logic0_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itCapsule_Logic0_Dropped");
    ext::Item_80268E5C(gobj, 4, 6);
}

pub unsafe extern "C" fn itCapsule_Logic0_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itCapsule_Logic0_Thrown");
    ext::Item_80268E5C(gobj, 3, 6);
}

pub unsafe extern "C" fn itCapsule_UnkMotion3_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itCapsule_UnkMotion3_Phys");
    itCapsule_UnkMotion4_Phys(item_gobj);
    ext::it_80274658(item_gobj, (*ext::it_804D6D28).x68_float);
}

pub unsafe extern "C" fn itCapsule_UnkMotion3_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_UnkMotion3_Coll");
    let it = GET_ITEM(item_gobj);
    ext::it_8026E414(item_gobj, Some(it_8027CF30));
    (*it).xDD4_itemVar.capsule.x0 != 0
}

/// 27D2DC
pub unsafe extern "C" fn it_8027D2DC(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_8027D2DC");
    let it = GET_ITEM(item_gobj);
    if (*it).xDD4_itemVar.capsule.x4 == 0 {
        it_8027CFE8(item_gobj);
    }
}

pub unsafe extern "C" fn itCapsule_UnkMotion5_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_UnkMotion5_Anim");
    ext::it_802751D8(item_gobj) != 0
}

pub unsafe extern "C" fn itCapsule_UnkMotion5_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itCapsule_UnkMotion5_Phys");}

pub unsafe extern "C" fn itCapsule_UnkMotion5_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_UnkMotion5_Coll");
    false
}

pub unsafe extern "C" fn itCapsule_Logic0_DmgDealt(item_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itCapsule_Logic0_DmgDealt");
    let it = GET_ITEM(item_gobj);

    if (*it).msid != 5 {
        it_8027CF30(item_gobj);
    }
    (*it).xDD4_itemVar.capsule.x0 != 0
}

/// `static bool itCapsule_Logic0_DmgDealt_autoinlined`
pub unsafe extern "C" fn itCapsule_Logic0_DmgDealt_autoinlined(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_Logic0_DmgDealt_autoinlined");
    let it = GET_ITEM(item_gobj);
    if (*it).msid != 5 {
        it_8027CF30(item_gobj);
    }
    (*it).xDD4_itemVar.capsule.x0 != 0
}

pub unsafe extern "C" fn itCapsule_Logic0_DmgReceived(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_Logic0_DmgReceived");
    itCapsule_Logic0_DmgDealt_autoinlined(item_gobj)
}

pub unsafe extern "C" fn itCapsule_Logic0_EnteredAir(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itCapsule_Logic0_EnteredAir");
    ext::Item_80268E5C(item_gobj, 6, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itCapsule_UnkMotion6_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_UnkMotion6_Anim");
    false
}

pub unsafe extern "C" fn itCapsule_UnkMotion6_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itCapsule_UnkMotion6_Phys");}

pub unsafe extern "C" fn itCapsule_UnkMotion6_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_UnkMotion6_Coll");
    ext::it_8026E8C4(item_gobj, Some(it_8027D0B8), Some(it_8027D148));
    false
}

pub unsafe extern "C" fn itCapsule_Logic0_Clanked(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_Logic0_Clanked");
    itCapsule_Logic0_DmgDealt_autoinlined(item_gobj)
}

pub unsafe extern "C" fn itCapsule_Logic0_Reflected(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_Logic0_Reflected");
    itCapsule_Logic0_DmgDealt_autoinlined(item_gobj)
}

pub unsafe extern "C" fn itCapsule_Logic0_HitShield(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_Logic0_HitShield");
    let it = GET_ITEM(item_gobj);
    if (*it).msid == 3 {
        it_8027CF30(item_gobj);
        return (*it).xDD4_itemVar.capsule.x0 != 0;
    }
    false
}

pub unsafe extern "C" fn itCapsule_Logic0_ShieldBounced(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itCapsule_Logic0_ShieldBounced");
    let it = GET_ITEM(item_gobj);
    if (*it).msid == 3 || (*it).msid == 4 {
        ext::itColl_BounceOffShield(item_gobj);
    }
    false
}

pub unsafe extern "C" fn itCapsule_Logic0_EvtUnk(item_gobj: *mut Item_GObj, ref_: *mut HSD_GObj) {
    melee_core::ct!("itCapsule_Logic0_EvtUnk");
    ext::it_8026B894(item_gobj, ref_);
}
