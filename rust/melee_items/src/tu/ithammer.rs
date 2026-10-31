//! Native port of src/melee/it/kinds/ithammer.c: the Hammer item (spawn, pick-up/drop states, the
//! swing that throws a Hammer Head). Callees in other TUs go through `ext`. Checked against the C
//! (the whole TU, shipping contraction, item/jobj state hashed field by field) by
//! tests/tu_ithammer.rs.
//!
//! No fused sites (`x8 / 100.0f` is a divide; the vector difference is a plain subtract).
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026B390(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_26B1::it_8026B894(a0, a1) as _ };
    fn it_8026BC14(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BC14(a0) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026DF34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DF34(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272A3C(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A3C(a0) };
    fn it_80272CC0(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj => unsafe { crate::tu::it_2725::it_80272CC0(a0, a1) };
    fn it_80299C48(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: f32) => unsafe { crate::tu::ithammerhead::it_80299C48(a0, a1 as _, a2 as _, a3) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 => unsafe { melee_world::tu_ext::lbVector_Normalize(a0 as _) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_world::tu_ext::lb_8000B1CC(a0, a1 as _, a2 as _) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::jobj_inlines_it!();
crate::inl_itResetVelocity!();

/// `typedef struct itHammerData` (the Hammer's special attributes: swing chance range and percent).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct itHammerData {
    pub x0: u32,
    pub x4: u32,
    pub x8: f32,
}
impl itHammerData {
    #[inline]
    pub fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

/// `ItemStateTable it_803F6150[6]`
pub static it_803F6150: [ItemStateTable; 6] = [
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHammer_UnkMotion0_Anim), physics_updated: Some(itHammer_UnkMotion0_Phys), collided: Some(itHammer_UnkMotion0_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHammer_UnkMotion4_Anim), physics_updated: Some(itHammer_UnkMotion1_Phys), collided: Some(itHammer_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itHammer_UnkMotion3_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 1, animated: Some(itHammer_UnkMotion3_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHammer_UnkMotion4_Anim), physics_updated: Some(itHammer_UnkMotion4_Phys), collided: Some(itHammer_UnkMotion4_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHammer_UnkMotion5_Anim), physics_updated: Some(itHammer_UnkMotion5_Phys), collided: Some(itHammer_UnkMotion5_Coll) },
];

/// `static inline void thing(void* gobj)`
pub unsafe extern "C" fn thing(gobj: *mut c_void) {
    melee_core::ct!("thing");
    let it = GET_ITEM(gobj as *mut HSD_GObj);
    if (*it).xD4C <= 0 {
        ext::it_80272A3C(ext::it_80272CC0(gobj as *mut HSD_GObj, 3));
    }
}

pub unsafe extern "C" fn it_80293D94(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80293D94");
    let item = GET_ITEM(gobj);
    if (*item).xDC8_word.flags.xB() == 0 && (*item).xDC8_word.flags.xA() != 0 {
        ext::it_8026BC14(gobj);
    }
}

pub unsafe extern "C" fn it_80293DCC(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("it_80293DCC");
    let item = GET_ITEM(gobj);
    let article = (*item).xC4_article_data;
    let temp_r31 = (*article).x4_specialAttributes as *mut itHammerData;
    if ext::HSD_Randf() < (*temp_r31).x8 / 100.0 {
        let rng = ext::HSD_Randi(((*temp_r31).x4.wrapping_sub((*temp_r31).x0)) as i32);
        return rng.wrapping_add((*temp_r31).x0 as i32);
    }
    0
}

pub unsafe extern "C" fn it_80293E34(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80293E34");
    let mut sp30: Vec3 = core::mem::zeroed();
    let mut sp24: Vec3 = core::mem::zeroed();
    let mut result: Vec3 = core::mem::zeroed();
    let temp_r31 = GET_ITEM(gobj);
    let temp_r3 = (*temp_r31).xD4C;
    if temp_r3 > 0 {
        (*temp_r31).xD4C = temp_r3.wrapping_sub(1);
        ext::lb_8000B1CC(ext::it_80272CC0(gobj, 3), null_mut(), &raw mut sp30);
        ext::lb_8000B1CC(ext::it_80272CC0(gobj, 1), null_mut(), &raw mut sp24);
        result.x = sp30.x - sp24.x;
        result.y = sp30.y - sp24.y;
        result.z = 0.0;
        ext::lbVector_Normalize(&raw mut result);
        let asd: f32 = if result.x < 0.0 { -1.0 } else { 1.0 };
        result.x = if result.x < 0.0 { -result.x } else { result.x };
        ext::it_80299C48((*temp_r31).owner as *mut HSD_GObj, &raw mut sp30, &raw mut result, asd);
    }
    thing(gobj as *mut c_void);
}

pub unsafe extern "C" fn itHammer_Logic28_Spawned(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammer_Logic28_Spawned");
    (*GET_ITEM(gobj)).xD4C = 1;
    it_8029402C(gobj);
}

pub unsafe extern "C" fn it_80293F84(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80293F84");
    let temp_r4 = GET_ITEM(gobj);
    itResetVelocity(temp_r4);
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    thing(gobj as *mut c_void);
}

pub unsafe extern "C" fn itHammer_UnkMotion0_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammer_UnkMotion0_Anim");
    0
}

pub unsafe extern "C" fn itHammer_UnkMotion0_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itHammer_UnkMotion0_Phys");}

pub unsafe extern "C" fn itHammer_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammer_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_8029402C));
    0
}

pub unsafe extern "C" fn it_8029402C(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8029402C");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    thing(gobj as *mut c_void);
}

pub unsafe extern "C" fn itHammer_UnkMotion4_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammer_UnkMotion4_Anim");
    0
}

pub unsafe extern "C" fn itHammer_UnkMotion1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammer_UnkMotion1_Phys");
    let it = GET_ITEM(gobj);
    let temp_r4 = (*it).xCC_item_attr;
    ext::it_80272860(gobj, (*temp_r4).x10_fall_speed, (*temp_r4).x14_fall_speed_max);
}

pub unsafe extern "C" fn itHammer_UnkMotion1_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammer_UnkMotion1_Coll");
    let it = GET_ITEM(gobj);
    if (*it).xD4C != 0 {
        ext::it_8026E15C(gobj, Some(it_80293F84));
        return 0;
    }
    ext::it_8026DF34(gobj)
}

pub unsafe extern "C" fn itHammer_Logic28_PickedUp(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammer_Logic28_PickedUp");
    let item = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
    (*item).entered_hitlag = core::mem::transmute::<unsafe extern "C" fn(*mut HSD_GObj), HSD_GObjEvent>(it_80293D94);
    thing(gobj as *mut c_void);
    ext::it_802725D4(gobj);
}

pub unsafe extern "C" fn itHammer_UnkMotion3_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammer_UnkMotion3_Anim");
    let it = GET_ITEM(gobj);
    if (*it).xD4C == 0 {
        it_802941A4(gobj);
    }
    0
}

pub unsafe extern "C" fn it_802941A4(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802941A4");
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as u32);
    thing(gobj as *mut c_void);
}

pub unsafe extern "C" fn itHammer_Logic28_Dropped(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammer_Logic28_Dropped");
    ext::Item_80268E5C(gobj, 1, 6);
    thing(gobj as *mut c_void);
}

pub unsafe extern "C" fn itHammer_UnkMotion4_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammer_UnkMotion4_Phys");
    let it = GET_ITEM(gobj);
    let temp_r4 = (*it).xCC_item_attr;
    ext::it_80272860(gobj, (*temp_r4).x10_fall_speed, (*temp_r4).x14_fall_speed_max);
}

pub unsafe extern "C" fn itHammer_UnkMotion4_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammer_UnkMotion4_Coll");
    if ext::it_8026DA08(gobj) != 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn itHammer_Logic28_EnteredAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammer_Logic28_EnteredAir");
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as u32);
    thing(gobj as *mut c_void);
}

pub unsafe extern "C" fn itHammer_UnkMotion5_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammer_UnkMotion5_Anim");
    0
}

pub unsafe extern "C" fn itHammer_UnkMotion5_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itHammer_UnkMotion5_Phys");}

pub unsafe extern "C" fn itHammer_UnkMotion5_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammer_UnkMotion5_Coll");
    ext::it_8026E8C4(gobj, Some(it_80293F84), Some(it_8029402C));
    0
}

pub unsafe extern "C" fn itHammer_Logic28_EvtUnk(igobj: *mut HSD_GObj, gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammer_Logic28_EvtUnk");
    ext::it_8026B894(igobj, gobj);
}
