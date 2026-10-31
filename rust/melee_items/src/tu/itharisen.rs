//! Native port of src/melee/it/kinds/itharisen.c: the Harisen (paper fan) item: scale set-up, state
//! callbacks, logic callbacks. Callees in other TUs go through `ext`. Checked against the C (the
//! whole TU, shipping contraction, item/jobj state hashed field by field) by
//! tests/tu_itharisen.rs.
//!
//! No fused sites (`x0_scale * x60_scale` is a single multiply).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
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
    fn Item_8026849C(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026849C(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026B390(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_26B1::it_8026B894(a0, a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DF34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DF34(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272F7C(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::it_2725::it_80272F7C(a0, a1) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut attrs: *mut ItemAttr = unsafe { core::mem::zeroed() };
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

crate::jobj_inlines_it!();
crate::inl_itResetVelocity!();
crate::inl_Item_ApplyFallingPhysics!();

/// `ItemStateTable it_803F5FF0[10]`
pub static it_803F5FF0: [ItemStateTable; 10] = [
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHarisen_UnkMotion0_Anim), physics_updated: Some(itHarisen_UnkMotion0_Phys), collided: Some(itHarisen_UnkMotion0_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHarisen_UnkMotion8_Anim), physics_updated: Some(itHarisen_UnkMotion1_Phys), collided: Some(itHarisen_UnkMotion1_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHarisen_UnkMotion6_Anim), physics_updated: Some(itHarisen_UnkMotion6_Phys), collided: None },
    ItemStateTable { anim_id: 0, animated: Some(itHarisen_UnkMotion6_Anim), physics_updated: Some(itHarisen_UnkMotion6_Phys), collided: None },
    ItemStateTable { anim_id: 1, animated: Some(itHarisen_UnkMotion6_Anim), physics_updated: Some(itHarisen_UnkMotion6_Phys), collided: None },
    ItemStateTable { anim_id: 2, animated: Some(itHarisen_UnkMotion6_Anim), physics_updated: Some(itHarisen_UnkMotion6_Phys), collided: None },
    ItemStateTable { anim_id: 3, animated: Some(itHarisen_UnkMotion6_Anim), physics_updated: Some(itHarisen_UnkMotion6_Phys), collided: None },
    ItemStateTable { anim_id: 4, animated: Some(itHarisen_UnkMotion8_Anim), physics_updated: Some(itHarisen_UnkMotion8_Phys), collided: Some(itHarisen_UnkMotion7_Coll) },
    ItemStateTable { anim_id: 4, animated: Some(itHarisen_UnkMotion8_Anim), physics_updated: Some(itHarisen_UnkMotion8_Phys), collided: Some(itHarisen_UnkMotion8_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHarisen_UnkMotion9_Anim), physics_updated: Some(itHarisen_UnkMotion9_Phys), collided: Some(itHarisen_UnkMotion9_Coll) },
];

pub unsafe extern "C" fn it_802927E8(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802927E8");
    let ip = (*gobj).user_data as *mut Item;
    let item_jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itHarisen_DatAttrs;
    let scale = (*attrs).x0_scale * (*(*ip).xCC_item_attr).x60_scale;
    (*ip).scl = scale;
    ext::it_80272F7C(item_jobj, scale);
}

pub unsafe extern "C" fn it_8029282C(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8029282C");
    let ip = GET_ITEM(gobj);
    (*ip).scl = (*(*ip).xCC_item_attr).x60_scale;
    ext::Item_8026849C(gobj);
}

pub unsafe extern "C" fn itHarisen_Logic24_Spawned(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHarisen_Logic24_Spawned");
    it_8029290C(gobj);
}

pub unsafe extern "C" fn it_8029287C(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8029287C");
    let ip = GET_ITEM(gobj);
    ext::it_8026B390(gobj);
    itResetVelocity(ip);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itHarisen_UnkMotion0_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_UnkMotion0_Anim");
    0
}

pub unsafe extern "C" fn itHarisen_UnkMotion0_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itHarisen_UnkMotion0_Phys");}

pub unsafe extern "C" fn itHarisen_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_8029290C));
    0
}

pub unsafe extern "C" fn it_8029290C(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8029290C");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itHarisen_UnkMotion8_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_UnkMotion8_Anim");
    0
}

pub unsafe extern "C" fn itHarisen_UnkMotion1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHarisen_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itHarisen_UnkMotion1_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_UnkMotion1_Coll");
    ext::it_8026E15C(gobj, Some(it_8029287C));
    0
}

pub unsafe extern "C" fn it_80292998(gobj: *mut HSD_GObj, frame_speed: f32) {
    melee_core::ct!("it_80292998");
    let ip = GET_ITEM(gobj);
    (*ip).x5D0_animFrameSpeed = frame_speed;
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn it_802929C8(gobj: *mut HSD_GObj, frame_speed: f32) {
    melee_core::ct!("it_802929C8");
    let ip = GET_ITEM(gobj);
    (*ip).x5D0_animFrameSpeed = frame_speed;
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn it_802929F8(gobj: *mut HSD_GObj, frame_speed: f32) {
    melee_core::ct!("it_802929F8");
    let ip = GET_ITEM(gobj);
    (*ip).x5D0_animFrameSpeed = frame_speed;
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn it_80292A28(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80292A28");
    let ip = GET_ITEM(gobj);
    (*ip).x5D0_animFrameSpeed = 1.0;
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itHarisen_Logic24_PickedUp(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHarisen_Logic24_PickedUp");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itHarisen_UnkMotion6_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_UnkMotion6_Anim");
    0
}

pub unsafe extern "C" fn itHarisen_UnkMotion6_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itHarisen_UnkMotion6_Phys");}

pub unsafe extern "C" fn itHarisen_Logic24_Dropped(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHarisen_Logic24_Dropped");
    let mut ip = (*gobj).user_data as *mut Item;
    (*ip).x5D0_animFrameSpeed = 1.0;
    ip = (*gobj).user_data as *mut Item;
    (*ip).scl = (*(*ip).xCC_item_attr).x60_scale;
    ext::Item_8026849C(gobj);
    ext::Item_80268E5C(gobj, 8, 6);
}

pub unsafe extern "C" fn itHarisen_UnkMotion8_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_UnkMotion8_Coll");
    let ip = GET_ITEM(gobj);
    if (*ip).xD4C != 0 {
        ext::it_8026E15C(gobj, Some(it_8029287C));
        return 0;
    }
    ext::it_8026DF34(gobj)
}

pub unsafe extern "C" fn itHarisen_Logic24_Thrown(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHarisen_Logic24_Thrown");
    ext::Item_80268E5C(gobj, 7, 6);
}

pub unsafe extern "C" fn itHarisen_UnkMotion8_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHarisen_UnkMotion8_Phys");
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itHarisen_UnkMotion7_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_UnkMotion7_Coll");
    ext::it_8026E15C(gobj, Some(it_8029287C));
    0
}

pub unsafe extern "C" fn itHarisen_Logic24_DmgDealt(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_Logic24_DmgDealt");
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 7 || (*ip).msid == 8 {
        ext::itColl_BounceOffVictim(gobj);
    }
    0
}

pub unsafe extern "C" fn itHarisen_Logic24_EnteredAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHarisen_Logic24_EnteredAir");
    ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itHarisen_UnkMotion9_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_UnkMotion9_Anim");
    0
}

pub unsafe extern "C" fn itHarisen_UnkMotion9_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itHarisen_UnkMotion9_Phys");}

pub unsafe extern "C" fn itHarisen_UnkMotion9_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_UnkMotion9_Coll");
    ext::it_8026E8C4(gobj, Some(it_8029287C), Some(it_8029290C));
    0
}

pub unsafe extern "C" fn itHarisen_Logic24_Clanked(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_Logic24_Clanked");
    itHarisen_Logic24_DmgDealt(gobj)
}

pub unsafe extern "C" fn itHarisen_Logic24_Reflected(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_Logic24_Reflected");
    ext::it_80273030(gobj)
}

pub unsafe extern "C" fn itHarisen_Logic24_HitShield(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_Logic24_HitShield");
    itHarisen_Logic24_DmgDealt(gobj)
}

pub unsafe extern "C" fn itHarisen_Logic24_ShieldBounced(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHarisen_Logic24_ShieldBounced");
    ext::itColl_BounceOffShield(gobj)
}

pub unsafe extern "C" fn itHarisen_Logic24_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("itHarisen_Logic24_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
