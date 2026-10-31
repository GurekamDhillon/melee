//! Port of src/melee/it/kinds/itspycloak.c: the Spy Cloak (logic table 33). Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itspycloak.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_mut, unused_variables, dead_code)]

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
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80276CB8(a0: *mut HSD_GObj) => unsafe { crate::tu::itmaplib::it_80276CB8(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F6370[]`
pub static mut it_803F6370: [ItemStateTable; 5] = [
    st_row(0, pred_b(itSpycloak_UnkMotion0_Anim), Some(itSpycloak_UnkMotion0_Phys), pred_b(itSpycloak_UnkMotion0_Coll)),
    st_row(0, pred_b(itSpycloak_UnkMotion3_Anim), Some(itSpycloak_UnkMotion1_Phys), pred_b(itSpycloak_UnkMotion1_Coll)),
    st_row(0, pred_b(itSpycloak_UnkMotion2_Anim), None, None),
    st_row(0, pred_b(itSpycloak_UnkMotion3_Anim), Some(itSpycloak_UnkMotion3_Phys), pred_b(itSpycloak_UnkMotion3_Coll)),
    st_row(0, pred_b(itSpycloak_UnkMotion4_Anim), Some(itSpycloak_UnkMotion4_Phys), pred_b(itSpycloak_UnkMotion4_Coll)),
];

pub unsafe extern "C" fn itSpyCloak_Logic33_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itSpyCloak_Logic33_Spawned");
    it_80295D04(gobj);
}

/// 295C68
pub unsafe extern "C" fn it_80295C68(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80295C68");
    let item = GET_ITEM(gobj);
    (*item).x40_vel.z = 0.0;
    (*item).x40_vel.y = 0.0;
    (*item).x40_vel.x = 0.0;
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itSpycloak_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSpycloak_UnkMotion0_Anim");
    false
}

pub unsafe extern "C" fn itSpycloak_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itSpycloak_UnkMotion0_Phys");}

pub unsafe extern "C" fn itSpycloak_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSpycloak_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_80295D04));
    ext::it_80276CB8(gobj);
    false
}

/// 295D04
pub unsafe extern "C" fn it_80295D04(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80295D04");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itSpycloak_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSpycloak_UnkMotion3_Anim");
    false
}

pub unsafe extern "C" fn itSpycloak_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSpycloak_UnkMotion1_Phys");
    let item = GET_ITEM(gobj);
    let attr = (*item).xCC_item_attr;

    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itSpycloak_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSpycloak_UnkMotion1_Coll");
    ext::it_8026E15C(gobj, Some(it_80295C68));
    false
}

pub unsafe extern "C" fn itSpyCloak_Logic33_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itSpyCloak_Logic33_PickedUp");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itSpycloak_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSpycloak_UnkMotion2_Anim");
    false
}

pub unsafe extern "C" fn itSpyCloak_Logic33_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itSpyCloak_Logic33_Dropped");
    ext::Item_80268E5C(gobj, 1, 6);
}

pub unsafe extern "C" fn itSpycloak_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSpycloak_UnkMotion3_Phys");
    let item = GET_ITEM(gobj);
    let attr = (*item).xCC_item_attr;

    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itSpycloak_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSpycloak_UnkMotion3_Coll");
    if ext::it_8026DA08(gobj) != 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itSpyCloak_Logic33_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itSpyCloak_Logic33_EnteredAir");
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itSpycloak_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSpycloak_UnkMotion4_Anim");
    false
}

pub unsafe extern "C" fn itSpycloak_UnkMotion4_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itSpycloak_UnkMotion4_Phys");}

pub unsafe extern "C" fn itSpycloak_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSpycloak_UnkMotion4_Coll");
    ext::it_8026E8C4(gobj, Some(it_80295C68), Some(it_80295D04));
    false
}

pub unsafe extern "C" fn itSpyCloak_Logic33_EvtUnk(gobj: *mut Item_GObj, r: *mut HSD_GObj) {
    melee_core::ct!("itSpyCloak_Logic33_EvtUnk");
    ext::it_8026B894(gobj, r);
}
