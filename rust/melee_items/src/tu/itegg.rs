//! Native port of src/melee/it/kinds/itegg.c: Chansey's egg item (spawn, the hatch roll, the
//! state callbacks and the state table `it_803F5988`). Callees in other TUs go through `ext`
//! (hooks; unset panics "TODO"). Checked against the C (the whole TU, shipping contraction) by
//! tests/tu_itegg.rs.
//!
//! Header inlines compiled into the TU: `Item_InitSpawnPosition`, `itResetVelocity`,
//! `Item_ApplyFallingPhysics`, `GET_ITEM`, `GET_JOBJ`, `HSD_JObjSetFlagsAll`'s callers. The variadic
//! `efSync_Spawn` is `ext::efSync_Spawn_N(gfx, gobj, &[args])` (the oracle counts the arguments).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

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
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efSync_Spawn_N(a0: i32, a1: *mut HSD_GObj, a2: &[*mut c_void]) -> *mut c_void;
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_8026F3D4(a0: *mut HSD_GObj, a1: *mut it_8026F3D4_arg1_t, a2: i32, a3: i32) -> i32 => unsafe { crate::tu::itdrop::it_8026F3D4(a0, a1, a2, a3) };
    fn it_8026F8B4(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: i32) -> i32 => unsafe { crate::tu::itdrop::it_8026F8B4(a0, a1 as _, a2 as _, a3) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C08(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272C08(a0) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80279BBC(a0: *mut Item) => unsafe { crate::tu::itanimlist::it_80279BBC(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `Item_InitSpawnPosition` (kinds/inlines.h).
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `itResetVelocity`.
#[inline]
unsafe fn itResetVelocity(ip: *mut Item) {
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
}

/// `Item_ApplyFallingPhysics`.
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*get_item(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

pub static mut it_803F5988: [ItemStateTable; 7] = [
    ItemStateTable { anim_id: -1, animated: Some(itEgg_UnkMotion0_Anim), physics_updated: Some(itEgg_UnkMotion0_Phys), collided: Some(itEgg_UnkMotion0_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itEgg_UnkMotion3_Anim), physics_updated: Some(itEgg_UnkMotion1_Phys), collided: Some(itEgg_UnkMotion1_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itEgg_UnkMotion2_Anim), physics_updated: Some(itEgg_UnkMotion2_Phys), collided: None },
    ItemStateTable { anim_id: 0, animated: Some(itEgg_UnkMotion3_Anim), physics_updated: Some(itEgg_UnkMotion3_Phys), collided: Some(itEgg_UnkMotion3_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itEgg_UnkMotion4_Anim), physics_updated: Some(itEgg_UnkMotion4_Phys), collided: Some(itEgg_UnkMotion4_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itEgg_UnkMotion5_Anim), physics_updated: Some(itEgg_UnkMotion5_Phys), collided: Some(itEgg_UnkMotion5_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itEgg_UnkMotion6_Anim), physics_updated: Some(itEgg_UnkMotion6_Phys), collided: Some(itEgg_UnkMotion6_Coll) },
];

pub unsafe extern "C" fn it_80288C88(gobj: *mut Item_GObj, pos: *mut Vec, vel: *mut Vec, dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut item_gobj: *mut Item_GObj = null_mut();
    if !gobj.is_null() {
        let ip = get_item(gobj);
        spawn.kind = It_Kind_Egg;
        Item_InitSpawnPosition(&mut spawn, pos, true);
        spawn.facing_dir = dir;
        spawn.x3C_damage = 0;
        spawn.vel = *vel;
        spawn.x0_parent_gobj = (*ip).owner;
        spawn.x4_parent_gobj2 = gobj;
        spawn.x44_flag.anon_0.set_b0(1);
        spawn.x40 = 0;
        item_gobj = ext::Item_80268B18(&mut spawn);
        if !item_gobj.is_null() {
            let ip = get_item(item_gobj);
            (*ip).xDD0_flag.anon_0.set_b6(0);
            (*ip).xD40 = 0.0;
            ext::it_80279BBC(ip);
            if (*ip).xDD0_flag.anon_0.b7() == 0 {
                ext::it_802756E0(item_gobj);
            }
        }
    }
    item_gobj
}

pub unsafe extern "C" fn itEgg_Logic3_Spawned(gobj: *mut Item_GObj) {
    (*get_item(gobj)).xDD4_itemVar.egg.x0 = 0;
    it_80288EFC(gobj);
}

pub unsafe extern "C" fn attrRand(attrs: *mut itEgg_ItemVars) -> i32 {
    ext::HSD_Randi((*attrs).rand_max)
}

pub unsafe extern "C" fn it_80288DC4(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itEgg_ItemVars;
    if attrRand(attrs) == 0 {
        return 1;
    }
    {
        let mut pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
        if ext::it_8026F8B4(gobj, &mut (*ip).pos, &mut pos, 0) != 0 {
            return 0;
        }
    }
    ext::it_8026F3D4(gobj, null_mut(), (*attrs).x0, 0);
    0
}

pub unsafe extern "C" fn it_80288E6C(gobj: *mut Item_GObj) {
    let item = get_item(gobj);
    ext::it_8026B390(gobj);
    itResetVelocity(item);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itEgg_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn itEgg_UnkMotion0_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itEgg_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026D62C(gobj, Some(it_80288EFC));
    0
}

pub unsafe extern "C" fn it_80288EFC(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itEgg_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn itEgg_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let attrs = (*get_item(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itEgg_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026E15C(gobj, Some(it_80288E6C));
    0
}

pub unsafe extern "C" fn itEgg_Logic3_PickedUp(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itEgg_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn itEgg_UnkMotion2_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itEgg_Logic3_Dropped(gobj: *mut Item_GObj) {
    itEgg_Logic3_Thrown(gobj);
}

pub unsafe extern "C" fn itEgg_Logic3_Thrown(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 3, 6);
}

pub unsafe extern "C" fn itEgg_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itEgg_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    if ext::it_8026DA08(gobj) != 0 {
        return itEgg_Logic3_DmgDealt(gobj);
    }
    0
}

pub unsafe extern "C" fn it_80289094(gobj: *mut Item_GObj) {
    let ip = get_item(gobj);
    ext::it_8026BB44(gobj);
    ext::it_80272C08(gobj);
    ext::it_802756D0(gobj);
    ext::it_8026B3A8(gobj);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).xDCF_flag.anon_0.set_b2(1);
    (*ip).xDD4_itemVar.egg.x0 = 1;
    (*ip).xDD4_itemVar.egg.rand_max = 20;
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itEgg_UnkMotion5_Anim(gobj: *mut Item_GObj) -> i32 {
    ext::it_802751D8(gobj)
}

pub unsafe extern "C" fn itEgg_UnkMotion5_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itEgg_UnkMotion5_Coll(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn it_80289158(gobj: *mut Item_GObj) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let ip = get_item(gobj);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as _);
    ext::it_802756D0(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).xDCF_flag.anon_0.set_b2(1);
    (*ip).xDD4_itemVar.egg.x0 = 1;
    (*ip).xDD4_itemVar.egg.rand_max = 40;
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itEgg_UnkMotion6_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = get_item(gobj);
    (*ip).xDD4_itemVar.egg.rand_max -= 1;
    if (*ip).xDD4_itemVar.egg.rand_max > 0 { 0 } else { 1 }
}

pub unsafe extern "C" fn itEgg_UnkMotion6_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itEgg_UnkMotion6_Coll(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn itEgg_Logic3_DmgDealt(gobj: *mut Item_GObj) -> i32 {
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).xDD4_itemVar.egg.x0 != 0 {
        return 0;
    }
    if it_80288DC4(gobj) != 0 {
        it_80289094(gobj);
        return 0;
    }
    ext::efSync_Spawn_N(1232, gobj, &[&raw mut (*ip).pos as *mut c_void]);
    ext::Item_8026AE84(ip, 244, 127, 64);
    it_80289158(gobj);
    0
}

pub unsafe extern "C" fn itEgg_Logic3_Clanked(gobj: *mut Item_GObj) -> i32 {
    itEgg_Logic3_DmgDealt(gobj)
}

pub unsafe extern "C" fn itEgg_Logic3_HitShield(gobj: *mut Item_GObj) -> i32 {
    itEgg_Logic3_DmgDealt(gobj)
}

pub unsafe extern "C" fn itEgg_Logic3_Reflected(gobj: *mut Item_GObj) -> i32 {
    itEgg_Logic3_DmgDealt(gobj)
}

pub unsafe extern "C" fn itEgg_Logic3_DmgReceived(gobj: *mut Item_GObj) -> i32 {
    itEgg_Logic3_DmgDealt(gobj)
}

pub unsafe extern "C" fn itEgg_Logic3_EnteredAir(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itEgg_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn itEgg_UnkMotion4_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itEgg_UnkMotion4_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026E8C4(gobj, Some(it_80288E6C), Some(it_80288EFC));
    0
}

pub unsafe extern "C" fn itEgg_Logic3_EvtUnk(gobj: *mut Item_GObj, ref_: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_);
}
