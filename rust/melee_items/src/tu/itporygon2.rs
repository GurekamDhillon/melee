//! Port of src/melee/it/kinds/itporygon2.c: Porygon2 (Pokemon item, logic table 26). Callees of unported TUs
//! go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itporygon2.rs.
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
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_8027A160(a0: *mut HSD_JObj, a1: *mut Item) => unsafe { crate::tu::it_279C::it_8027A160(a0, a1) };
    fn it_8027A344(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027a344(&mut *((*a0).user_data as *mut Item)) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F8250[]`
pub static mut it_803F8250: [ItemStateTable; 2] = [
    st_row(0, pred_b(itPorygon2_UnkMotion0_Anim), Some(itPorygon2_UnkMotion0_Phys), pred_b(itPorygon2_UnkMotion0_Coll)),
    st_row(1, pred_b(itPorygon2_UnkMotion1_Anim), Some(itPorygon2_UnkMotion1_Phys), pred_b(itPorygon2_UnkMotion1_Coll)),
];

/// `Item_SetEffectHitlagCallbacks` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_SetEffectHitlagCallbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

pub unsafe extern "C" fn itPorygon2_Logic26_Spawned(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itPorygon2_Logic26_Spawned");
    let item = GET_ITEM(item_gobj);
    ext::it_80279CDC(item_gobj, (*item).scl);
    ext::Item_8026AE84(item, 0x273E, 0x7F, 0x40);
    it_802D5B14(item_gobj);
}

pub unsafe extern "C" fn it_802D5B14(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D5B14");
    let item = GET_ITEM(item_gobj);
    ext::Item_80268E5C(item_gobj as *mut HSD_GObj, 0, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(item);
}

pub unsafe extern "C" fn itPorygon2_UnkMotion0_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPorygon2_UnkMotion0_Anim");
    let item = GET_ITEM(item_gobj);
    if ext::it_80272C6C(item_gobj) == 0 {
        it_802D5C00(item_gobj);
    }
    ext::it_8027A160((*(*item).xBBC_dynamicBoneTable).bones[1], item);
    false
}

pub unsafe extern "C" fn itPorygon2_UnkMotion0_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itPorygon2_UnkMotion0_Phys");
    ext::it_8027A344(item_gobj);
}

pub unsafe extern "C" fn itPorygon2_UnkMotion0_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPorygon2_UnkMotion0_Coll");
    ext::it_8026DA08(item_gobj);
    false
}

pub unsafe extern "C" fn it_802D5C00(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D5C00");
    let item = GET_ITEM(item_gobj);
    ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(item);
    ext::Item_8026AE84(item, 0x273F, 0x7F, 0x40);
}

pub unsafe extern "C" fn itPorygon2_UnkMotion1_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPorygon2_UnkMotion1_Anim");
    let item = GET_ITEM(item_gobj);
    if ext::it_80272C6C(item_gobj) == 0 {
        return true;
    }
    ext::it_8027A160((*(*item).xBBC_dynamicBoneTable).bones[1], item);
    false
}

pub unsafe extern "C" fn itPorygon2_UnkMotion1_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itPorygon2_UnkMotion1_Phys");
    ext::it_8027A344(item_gobj);
}

pub unsafe extern "C" fn itPorygon2_UnkMotion1_Coll(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPorygon2_UnkMotion1_Coll");
    false
}

pub unsafe extern "C" fn it_802D5CD8(item_gobj: *mut Item_GObj, arg1: *mut HSD_GObj) {
    melee_core::ct!("it_802D5CD8");
    ext::it_8026B894(item_gobj, arg1);
}
