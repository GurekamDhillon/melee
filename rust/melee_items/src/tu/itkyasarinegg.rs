//! Port of src/melee/it/kinds/itkyasarinegg.c: Chansey's egg (spawn, throw physics, hit handling).
//! `bool` callbacks return an `i32`-sized C bool; `itKyasarinegg_UnkMotion4_Anim` has no `return` in the C, the
//! port returns what the PPC leaves in r3 (the `it_802751D8` result). Callees of unported TUs go through `ext`
//! (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itkyasarinegg.rs.
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
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_8026DFB0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DFB0(a0) };
    fn it_8026F3D4(a0: *mut HSD_GObj, a1: *mut it_8026F3D4_arg1_t, a2: i32, a3: i32) -> i32 => unsafe { crate::tu::itdrop::it_8026F3D4(a0, a1, a2, a3) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C08(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272C08(a0) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80279BBC(a0: *mut Item) => unsafe { crate::tu::itanimlist::it_80279BBC(a0) };
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzako::it_8027B5B0(a0, a1 as _, a2, a3 as _, a4) };
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) };
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

/// `ItemStateTable it_803F9328[]`
pub static mut it_803F9328: [ItemStateTable; 5] = [
    st_row(0, None, None, pred_b(itKyasarinegg_UnkMotion0_Coll)),
    st_row(-1, None, Some(itKyasarinegg_UnkMotion1_Phys), Some(itKyasarinegg_UnkMotion1_Coll)),
    st_row(-1, None, None, None),
    st_row(1, None, Some(itKyasarinegg_UnkMotion3_Phys), pred_b(itKyasarinegg_UnkMotion3_Coll)),
    st_row(2, Some(itKyasarinegg_UnkMotion4_Anim), None, None),
];

/// 2EFA44: spawns an egg at `pos` facing `dir` (only when `catherine` is non-null).
pub unsafe extern "C" fn it_802EFA44(catherine: *mut Item_GObj, pos: *mut Vec3, dir: f32) {
    melee_core::ct!("it_802EFA44");
    if !catherine.is_null() {
        let egg = ext::it_8027B5B0(It_Kind_Kyasarin_Egg as i32, pos, null_mut(), null_mut(), 1);
        if !egg.is_null() {
            let eggData = GET_ITEM(egg);
            (*eggData).facing_dir = dir;
            let direction = if (*eggData).facing_dir == -1.0 { -1 } else { 1 };
            ext::mpCollSetFacingDir(&mut (*eggData).x378_itemColl, direction);
            (*eggData).xDD0_flag.anon_0.set_b6(0);
            (*eggData).xD40 = 0.0;
            (*eggData).xDD4_itemVar.kyasarinEgg.x20 = 0;
            ext::it_80279BBC(eggData);
            if (*eggData).xDD0_flag.anon_0.b7() == 0 {
                ext::it_802756E0(egg);
            }
            it_802EFB0C(egg);
        }
    }
}

/// 2EFB0C
pub unsafe extern "C" fn it_802EFB0C(egg_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EFB0C");
    let egg = GET_ITEM(egg_gobj);
    let ap = (*(*egg).xC4_article_data).x4_specialAttributes as *mut itKyasarinEggAttributes;
    ext::it_8026B390(egg_gobj);
    (*egg).x40_vel.x = (*ap).x4 * (*egg).facing_dir;
    (*egg).x40_vel.z = 0.0;
    (*egg).x40_vel.y = 0.0;
    ext::Item_80268E5C(egg_gobj, 0, 2);
}

pub unsafe extern "C" fn itKyasarinegg_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itKyasarinegg_UnkMotion0_Coll");
    if ext::it_8026DAA8(gobj) & 0xF != 0 {
        it_802EFCC0(gobj);
    }
    false
}

pub unsafe extern "C" fn itKyasarinEgg_Logic28_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itKyasarinEgg_Logic28_PickedUp");
    ext::Item_80268E5C(gobj, 2, 2);
}

pub unsafe extern "C" fn itKyasarinEgg_Logic28_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itKyasarinEgg_Logic28_Dropped");
    itKyasarinEgg_Logic28_Thrown(gobj);
}

pub unsafe extern "C" fn itKyasarinEgg_Logic28_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itKyasarinEgg_Logic28_Thrown");
    ext::Item_80268E5C(gobj, 3, 6);
}

pub unsafe extern "C" fn itKyasarinegg_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itKyasarinegg_UnkMotion3_Phys");
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itKyasarinegg_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itKyasarinegg_UnkMotion3_Coll");
    if ext::it_8026DA08(gobj) != 0 {
        return it_2725_Logic28_DmgDealt(gobj);
    }
    false
}

/// 2EFCC0: bounce off a wall.
pub unsafe extern "C" fn it_802EFCC0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EFCC0");
    let egg = GET_ITEM(gobj);
    let ap = (*(*egg).xC4_article_data).x4_specialAttributes as *mut itKyasarinEggAttributes;
    ext::Item_80268E5C(gobj, 1, 2);
    (*egg).x40_vel.x = -(*egg).x40_vel.x;
    (*egg).x40_vel.y = (*ap).x8;
}

pub unsafe extern "C" fn itKyasarinegg_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itKyasarinegg_UnkMotion1_Phys");
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itKyasarinegg_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKyasarinegg_UnkMotion1_Coll");
    ext::it_8026DFB0(gobj)
}

/// 2EFD84: breaks into a stationary egg.
pub unsafe extern "C" fn it_802EFD84(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EFD84");
    let egg = GET_ITEM(gobj);
    ext::it_8026BB44(gobj);
    ext::it_80272C08(gobj);
    ext::it_802756D0(gobj);
    ext::it_8026B3A8(gobj);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    (*egg).x40_vel.x = 0.0;
    (*egg).x40_vel.y = 0.0;
    (*egg).xDD4_itemVar.kyasarinEgg.x20 = 1;
    ext::Item_80268E5C(gobj, 4, 2);
}

/// No `return` in the C: the result is whatever `it_802751D8` left in r3.
pub unsafe extern "C" fn itKyasarinegg_UnkMotion4_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKyasarinegg_UnkMotion4_Anim");
    ext::it_802751D8(gobj)
}

/// `static inline it_damage_inline`
#[inline]
unsafe fn it_damage_inline(gobj: *mut Item_GObj) -> bool {
    let egg = GET_ITEM(gobj);
    if (*egg).xDD4_itemVar.kyasarinEgg.x20 != 0 {
        return false;
    }
    if (*egg).msid != 3 {
        let ap = (*(*egg).xC4_article_data).x4_specialAttributes as *mut itKyasarinEggAttributes;
        ext::Item_80268E5C(gobj, 1, 2);
        (*egg).x40_vel.x = -(*egg).x40_vel.x;
        (*egg).x40_vel.y = (*ap).x8;
        false
    } else {
        ext::efSync_Spawn(0x4d0, gobj, &mut (*egg).pos);
        ext::Item_8026AE84(egg, 0xf4, 0x7f, 0x40);
        let ap = (*(*GET_ITEM(gobj)).xC4_article_data).x4_specialAttributes as *mut itKyasarinEggAttributes;
        if ext::HSD_Randi((*ap).x10) == 0 {
            it_802EFD84(gobj);
            false
        } else {
            ext::it_8026F3D4(gobj, null_mut(), (*ap).xC, 0);
            true
        }
    }
}

pub unsafe extern "C" fn it_2725_Logic28_DmgDealt(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic28_DmgDealt");
    it_damage_inline(gobj)
}

pub unsafe extern "C" fn it_2725_Logic28_Clanked(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic28_Clanked");
    it_damage_inline(gobj)
}

pub unsafe extern "C" fn it_2725_Logic28_HitShield(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic28_HitShield");
    it_damage_inline(gobj)
}

pub unsafe extern "C" fn it_2725_Logic28_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic28_Reflected");
    it_damage_inline(gobj)
}

pub unsafe extern "C" fn itKyasarinEgg_Logic28_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itKyasarinEgg_Logic28_ShieldBounced");
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 {
        ext::itColl_BounceOffShield(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802F022C(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802F022C");
    it_damage_inline(gobj)
}

pub unsafe extern "C" fn it_802F0320(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802F0320");
    ext::it_8026B894(gobj, ref_gobj);
}
