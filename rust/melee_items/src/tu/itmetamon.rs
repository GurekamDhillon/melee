//! Port of src/melee/it/kinds/itmetamon.c: Ditto (Pokeball Pokemon, logic table 19). `ItMetamonVars` is the
//! TU-local layout of the article's special attributes (one f32). Callees of unported TUs go through `ext`
//! (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itmetamon.rs.
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
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `static Item_GObj* it_804D6D50;` (written by `it_802D3008`, never read in this TU)
pub static mut it_804D6D50: *mut Item_GObj = null_mut();

/// `ItMetamonVars` (TU-local typedef)
#[repr(C)]
pub struct ItMetamonVars {
    pub x0: f32,
}

/// `ItemStateTable it_803F7FC8[]`
pub static mut it_803F7FC8: [ItemStateTable; 3] = [
    st_row(0, pred_b!(itMetamon_UnkMotion0_Anim), Some(itMetamon_UnkMotion0_Phys), pred_b!(itMetamon_UnkMotion0_Coll)),
    st_row(1, pred_b!(itMetamon_UnkMotion1_Anim), Some(itMetamon_UnkMotion1_Phys), pred_b!(itMetamon_UnkMotion1_Coll)),
    st_row(-1, pred_b!(itMetamon_UnkMotion2_Anim), Some(itMetamon_UnkMotion2_Phys), pred_b!(itMetamon_UnkMotion2_Coll)),
];

pub unsafe extern "C" fn it_802D3008(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D3008");
    let it = GET_ITEM(item_gobj);
    let vars = (*(*it).xC4_article_data).x4_specialAttributes as *mut ItMetamonVars;
    ext::it_80279CDC(item_gobj, (*vars).x0);
    it_802D31B4(item_gobj);
    ext::Item_8026AE84(it, 0x2728, 0x7F, 0x40);
    it_804D6D50 = item_gobj;
}

pub unsafe extern "C" fn it_802D306C(_item_gobj: *mut Item_GObj) { melee_core::ct!("it_802D306C");}

pub unsafe extern "C" fn itMetamon_Logic19_EvtUnk(item_gobj: *mut Item_GObj, gobj: *mut Item_GObj) {
    melee_core::ct!("itMetamon_Logic19_EvtUnk");
    ext::it_8026B894(item_gobj, gobj);
}

pub unsafe extern "C" fn it_802D3090(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D3090");
    ext::it_80273454(item_gobj);
    it_802D30C4(item_gobj);
}

pub unsafe extern "C" fn it_802D30C4(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D30C4");
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMetamon_UnkMotion0_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMetamon_UnkMotion0_Anim");
    GET_ITEM(item_gobj);
    if ext::it_80272C6C(item_gobj) == 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itMetamon_UnkMotion0_Phys(_item_gobj: *mut Item_GObj) { melee_core::ct!("itMetamon_UnkMotion0_Phys");}

pub unsafe extern "C" fn itMetamon_UnkMotion0_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMetamon_UnkMotion0_Coll");
    ext::it_8026E15C(item_gobj, Some(it_802D306C));
    false
}

pub unsafe extern "C" fn itMetamon_UnkMotion1_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMetamon_UnkMotion1_Anim");
    if ext::it_80272C6C(item_gobj) == 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itMetamon_UnkMotion1_Phys(_item_gobj: *mut Item_GObj) { melee_core::ct!("itMetamon_UnkMotion1_Phys");}

pub unsafe extern "C" fn itMetamon_UnkMotion1_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMetamon_UnkMotion1_Coll");
    ext::it_8026E15C(item_gobj, Some(it_802D306C));
    false
}

pub unsafe extern "C" fn it_802D31B4(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D31B4");
    ext::it_802762BC(GET_ITEM(item_gobj));
    ext::Item_80268E5C(item_gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMetamon_UnkMotion2_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMetamon_UnkMotion2_Anim");
    ext::it_80279FF8(item_gobj);
    false
}

pub unsafe extern "C" fn itMetamon_UnkMotion2_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itMetamon_UnkMotion2_Phys");
    ext::it_8027A09C(item_gobj);
}

pub unsafe extern "C" fn itMetamon_UnkMotion2_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMetamon_UnkMotion2_Coll");
    ext::it_8027A118(item_gobj, Some(it_802D3090)) != 0
}
