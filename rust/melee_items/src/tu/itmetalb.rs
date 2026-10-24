//! Port of src/melee/it/kinds/itmetalb.c: Metal Box (logic table 32). Callees of unported TUs go through
//! `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itmetalb.rs.
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
    fn ftLib_80086BE0(a0: *mut HSD_GObj) -> u8;
    fn ftLib_800871A8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftLib_800874BC(a0: *mut HSD_GObj) -> i32;
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_8027236C(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::tu::itcoll::it_8027236C(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80276CB8(a0: *mut HSD_GObj) => unsafe { crate::tu::itmaplib::it_80276CB8(a0) };
    fn pl_8003E17C(a0: i32, a1: i32, a2: *mut HSD_GObj);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F62C0[]`
pub static mut it_803F62C0: [ItemStateTable; 5] = [
    st_row(-1, pred_b(itMetalb_UnkMotion0_Anim), Some(itMetalb_UnkMotion0_Phys), pred_b(itMetalb_UnkMotion0_Coll)),
    st_row(-1, pred_b(itMetalb_UnkMotion3_Anim), Some(itMetalb_UnkMotion1_Phys), pred_b(itMetalb_UnkMotion1_Coll)),
    st_row(-1, pred_b(itMetalb_UnkMotion2_Anim), None, None),
    st_row(-1, pred_b(itMetalb_UnkMotion3_Anim), Some(itMetalb_UnkMotion3_Phys), pred_b(itMetalb_UnkMotion3_Coll)),
    st_row(-1, pred_b(itMetalb_UnkMotion4_Anim), Some(itMetalb_UnkMotion4_Phys), pred_b(itMetalb_UnkMotion4_Coll)),
];

pub unsafe extern "C" fn itMetalB_Logic32_Spawned(gobj: *mut Item_GObj) {
    let it = GET_ITEM(gobj);
    (*it).xDCE_flag.anon_0.set_b7(0);
    it_80295498(gobj);
}

pub unsafe extern "C" fn it_802953FC(gobj: *mut Item_GObj) {
    let it = (*gobj).user_data as *mut Item;
    itResetVelocity(it);

    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMetalb_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itMetalb_UnkMotion0_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itMetalb_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_80295498));
    ext::it_80276CB8(gobj);

    false
}

pub unsafe extern "C" fn it_80295498(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMetalb_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itMetalb_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let it = (*gobj).user_data as *mut Item;
    ext::it_80272860(gobj, (*(*it).xCC_item_attr).x10_fall_speed, (*(*it).xCC_item_attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itMetalb_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_802953FC));

    false
}

pub unsafe extern "C" fn itMetalB_Logic32_PickedUp(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMetalb_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itMetalB_Logic32_Dropped(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
}

pub unsafe extern "C" fn itMetalb_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    let it = (*gobj).user_data as *mut Item;
    ext::it_80272860(gobj, (*(*it).xCC_item_attr).x10_fall_speed, (*(*it).xCC_item_attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itMetalb_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    if ext::it_8026DA08(gobj) != 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itMetalB_Logic32_DmgReceived(gobj: *mut Item_GObj) -> bool {
    let it = GET_ITEM(gobj);
    let go = ext::it_8027236C(gobj);

    if !go.is_null() {
        if (*go).classifier as i32 == HSD_GOBJ_CLASS_FIGHTER && (*it).xDCF_flag.anon_0.b6() != 0 {
            ext::ftLib_800871A8(go, gobj);
            ext::pl_8003E17C((ext::ftLib_80086BE0(go) & 0xFF) as i32, ext::ftLib_800874BC(go), gobj);
            return true;
        } else {
            (*it).xCEC_fighterGObj = null_mut();
        }
    }

    true
}

pub unsafe extern "C" fn itMetalB_Logic32_EnteredAir(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMetalb_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itMetalb_UnkMotion4_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itMetalb_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E8C4(gobj, Some(it_802953FC), Some(it_80295498));

    false
}

pub unsafe extern "C" fn itMetalB_Logic32_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
