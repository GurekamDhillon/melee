//! Port of src/melee/it/kinds/itscball.c: the Smash Ball (logic table 30). Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itscball.rs.
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

/// `Item_ApplyFallingPhysics` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

/// `ItemStateTable it_803F6220[]`
pub static mut it_803F6220: [ItemStateTable; 5] = [
    st_row(-1, pred_b(itScball_UnkMotion0_Anim), Some(itScball_UnkMotion0_Phys), pred_b(itScball_UnkMotion0_Coll)),
    st_row(-1, pred_b(itScball_UnkMotion1_Anim), Some(itScball_UnkMotion1_Phys), pred_b(itScball_UnkMotion1_Coll)),
    st_row(-1, pred_b(itScball_UnkMotion2_Anim), None, None),
    st_row(0, pred_b(itScball_UnkMotion3_Anim), Some(itScball_UnkMotion3_Phys), pred_b(itScball_UnkMotion3_Coll)),
    st_row(-1, pred_b(itScball_UnkMotion4_Anim), Some(itScball_UnkMotion4_Phys), pred_b(itScball_UnkMotion4_Coll)),
];

pub unsafe extern "C" fn itScball_Logic30_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itScball_Logic30_Spawned");
    it_80294B58(gobj);
}

/// 294AD0
pub unsafe extern "C" fn it_80294AD0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80294AD0");
    let item = GET_ITEM(gobj);

    itResetVelocity(item);
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itScball_UnkMotion0_Anim(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itScball_UnkMotion0_Anim");
    false
}

pub unsafe extern "C" fn itScball_UnkMotion0_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itScball_UnkMotion0_Phys");}

pub unsafe extern "C" fn itScball_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itScball_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_80294B58));
    false
}

/// 294B58
pub unsafe extern "C" fn it_80294B58(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80294B58");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itScball_UnkMotion1_Anim(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itScball_UnkMotion1_Anim");
    false
}

pub unsafe extern "C" fn itScball_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itScball_UnkMotion1_Phys");
    let item = GET_ITEM(gobj);
    let attr = (*item).xCC_item_attr;

    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itScball_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itScball_UnkMotion1_Coll");
    ext::it_8026E15C(gobj, Some(it_80294AD0));
    false
}

pub unsafe extern "C" fn itScball_Logic30_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itScball_Logic30_PickedUp");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itScball_UnkMotion2_Anim(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itScball_UnkMotion2_Anim");
    false
}

pub unsafe extern "C" fn itScball_Logic30_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itScball_Logic30_Dropped");
    ext::Item_80268E5C(gobj, 1, 6);
}

pub unsafe extern "C" fn itScball_Logic30_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itScball_Logic30_Thrown");
    ext::Item_80268E5C(gobj, 3, 6);
}

pub unsafe extern "C" fn itScball_UnkMotion3_Anim(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itScball_UnkMotion3_Anim");
    false
}

pub unsafe extern "C" fn itScball_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itScball_UnkMotion3_Phys");
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itScball_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itScball_UnkMotion3_Coll");
    ext::it_8026E15C(gobj, Some(it_80294AD0));
    false
}

pub unsafe extern "C" fn itScball_Logic30_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itScball_Logic30_EnteredAir");
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itScball_UnkMotion4_Anim(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itScball_UnkMotion4_Anim");
    false
}

pub unsafe extern "C" fn itScball_UnkMotion4_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itScball_UnkMotion4_Phys");}

pub unsafe extern "C" fn itScball_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itScball_UnkMotion4_Coll");
    ext::it_8026E8C4(gobj, Some(it_80294AD0), Some(it_80294B58));
    false
}

pub unsafe extern "C" fn itScball_Logic30_DmgDealt(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itScball_Logic30_DmgDealt");
    true
}

pub unsafe extern "C" fn itScball_Logic30_Clanked(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itScball_Logic30_Clanked");
    true
}

pub unsafe extern "C" fn itScball_Logic30_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itScball_Logic30_Reflected");
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn itScball_Logic30_HitShield(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itScball_Logic30_HitShield");
    ext::itColl_BounceOffVictim(gobj);
    false
}
