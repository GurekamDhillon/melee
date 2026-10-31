//! Port of src/melee/it/kinds/itstar.c: the Star Man item (logic table 10; bounces around). `star.dir` is the
//! horizontal direction (+-1). Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked
//! against the C (whole TU, shipping contraction) by tests/kinds_itstar.rs.
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_802763E0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::helpers::it_802763e0(&mut *((*a0).user_data as *mut Item)) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F57A0[]`
pub static mut it_803F57A0: [ItemStateTable; 1] = [
    st_row(0, pred_b(itStar_UnkMotion0_Anim), Some(itStar_UnkMotion0_Phys), pred_b(itStar_UnkMotion0_Coll)),
];

pub unsafe extern "C" fn itStar_Logic10_Spawned(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itStar_Logic10_Spawned");
    let item = GET_ITEM(item_gobj);
    let star = (*(*item).xC4_article_data).x4_specialAttributes as *mut itStar_ItemVars;
    ext::it_8026B390(item_gobj);
    (*item).xDD4_itemVar.star.dir = if ext::HSD_Randi(2) != 0 { 1.0 } else { -1.0 };
    (*item).x40_vel.x = (*star).xvel * (*item).xDD4_itemVar.star.dir;
    (*item).x40_vel.y = (*star).yvel;
    (*item).x40_vel.z = 0.0;
    it_802846D4(item_gobj);
}

/// 2846D4
pub unsafe extern "C" fn it_802846D4(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802846D4");
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itStar_UnkMotion0_Anim(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itStar_UnkMotion0_Anim");
    false
}

pub unsafe extern "C" fn itStar_UnkMotion0_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itStar_UnkMotion0_Phys");
    let item = GET_ITEM(item_gobj);
    let attr = (*item).xCC_item_attr;
    ext::it_80272860(item_gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itStar_UnkMotion0_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itStar_UnkMotion0_Coll");
    let item = GET_ITEM(item_gobj);
    let mut val = false;

    if ext::it_8026DA08(item_gobj) != 0 {
        let item2 = GET_ITEM(item_gobj);
        let star = (*(*item2).xC4_article_data).x4_specialAttributes as *mut itStar_ItemVars;
        (*item2).x40_vel.x = (*star).xvel * (*item2).xDD4_itemVar.star.dir;
        (*item2).x40_vel.y = (*star).yvel;
        ext::Item_80268E5C(item_gobj, 0, ITEM_UNK_0x1 | ITEM_HIT_PRESERVE);
        val = true;
    }
    if ext::it_80276308(item_gobj) != 0 {
        (*item).xDD4_itemVar.star.dir = -(*item).xDD4_itemVar.star.dir;
        (*item).x40_vel.x = -(*item).x40_vel.x;
        val = true;
    }
    if ext::it_802763E0(item_gobj) != 0 {
        if (*item).x40_vel.y > 0.0 {
            (*item).x40_vel.y = 0.0;
        }
        val = true;
    }
    if val {
        ext::Item_8026AE84(item, 0xFA, 0x7F, 0x40);
    }
    false
}

pub unsafe extern "C" fn itStar_Logic10_DmgDealt(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itStar_Logic10_DmgDealt");
    true
}

pub unsafe extern "C" fn itStar_Logic10_EvtUnk(item_gobj: *mut Item_GObj, ref_: *mut HSD_GObj) {
    melee_core::ct!("itStar_Logic10_EvtUnk");
    ext::it_8026B894(item_gobj, ref_);
}
