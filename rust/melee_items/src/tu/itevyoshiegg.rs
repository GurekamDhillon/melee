//! Native port of src/melee/it/kinds/itevyoshiegg.c: the Event-match Yoshi egg item (spawn, the
//! bounce-off hits, the break timer, the state callbacks and the state table `it_803F6728`).
//! Callees in other TUs go through `ext` (hooks; unset panics "TODO"). Checked against the C (the
//! whole TU, shipping contraction) by tests/tu_itevyoshiegg.rs.
//!
//! Header inlines compiled into the TU: `Item_InitSpawnCommonFields`, `itResetVelocity`,
//! `Item_ApplyFallingPhysics`. The variadic `efSync_Spawn` is
//! `ext::efSync_Spawn_N(gfx, gobj, &[args])` (the oracle counts the arguments).
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efSync_Spawn_N(a0: i32, a1: *mut HSD_GObj, a2: &[*mut c_void]) -> *mut c_void;
    fn gm_801BEB68(a0: i32);
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026E32C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026E32C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `Item_InitSpawnCommonFields` (kinds/inlines.h).
#[inline]
unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.x = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.z = 0.0;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as u8);
    (*spawn).x40 = 0;
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

pub static mut it_803F6728: [ItemStateTable; 6] = [
    ItemStateTable { anim_id: -1, animated: Some(itEvyoshiegg_UnkMotion0_Anim), physics_updated: Some(itEvyoshiegg_UnkMotion0_Phys), collided: Some(itEvyoshiegg_UnkMotion0_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itEvyoshiegg_UnkMotion3_Anim), physics_updated: Some(itEvyoshiegg_UnkMotion1_Phys), collided: Some(itEvyoshiegg_UnkMotion1_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itEvyoshiegg_UnkMotion2_Anim), physics_updated: Some(itEvyoshiegg_UnkMotion2_Phys), collided: None },
    ItemStateTable { anim_id: 0, animated: Some(itEvyoshiegg_UnkMotion3_Anim), physics_updated: Some(itEvyoshiegg_UnkMotion3_Phys), collided: Some(itEvyoshiegg_UnkMotion3_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itEvyoshiegg_UnkMotion4_Anim), physics_updated: Some(itEvyoshiegg_UnkMotion4_Phys), collided: Some(itEvyoshiegg_UnkMotion4_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itEvyoshiegg_UnkMotion5_Anim), physics_updated: Some(itEvyoshiegg_UnkMotion5_Phys), collided: Some(itEvyoshiegg_UnkMotion5_Coll) },
];

pub unsafe extern "C" fn itEvYoshiEgg_Spawn(pos: *mut Vec) {
    melee_core::ct!("itEvYoshiEgg_Spawn");
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_EvYoshiEgg;
    spawn.prev_pos = *pos;
    spawn.pos = *pos;
    Item_InitSpawnCommonFields(&mut spawn, null_mut(), 1.0, true);
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let it = get_item(gobj);
        it_8029B268(gobj);
        (*it).xDD4_itemVar.evyoshiegg.xDD4 = 0;
    }
}

pub unsafe extern "C" fn itEvYoshiEgg_Logic42_Destroyed(_gobj: *mut Item_GObj) {
    melee_core::ct!("itEvYoshiEgg_Logic42_Destroyed");
    ext::gm_801BEB68(1);
}

pub unsafe extern "C" fn itEvYoshiEgg_Logic42_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itEvYoshiEgg_Logic42_Spawned");
    let it = get_item(gobj);
    (*it).xDD4_itemVar.evyoshiegg.xDD4 = 0;
    it_8029B268(gobj);
}

pub unsafe extern "C" fn it_8029B1D8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8029B1D8");
    let ip = get_item(gobj);
    ext::it_8026B390(gobj);
    itResetVelocity(ip);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvyoshiegg_UnkMotion0_Anim");
    0
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itEvyoshiegg_UnkMotion0_Phys");}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvyoshiegg_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_8029B268));
    0
}

pub unsafe extern "C" fn it_8029B268(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8029B268");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvyoshiegg_UnkMotion3_Anim");
    0
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itEvyoshiegg_UnkMotion1_Phys");
    let ip = get_item(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvyoshiegg_UnkMotion1_Coll");
    ext::it_8026E32C(gobj, Some(it_8029B1D8));
    0
}

pub unsafe extern "C" fn itEvYoshiEgg_Logic42_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itEvYoshiEgg_Logic42_PickedUp");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvyoshiegg_UnkMotion2_Anim");
    0
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itEvyoshiegg_UnkMotion2_Phys");}

pub unsafe extern "C" fn itEvYoshiEgg_Logic42_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itEvYoshiEgg_Logic42_Dropped");
    itEvYoshiEgg_Logic42_Thrown(gobj);
}

pub unsafe extern "C" fn itEvYoshiEgg_Logic42_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itEvYoshiEgg_Logic42_Thrown");
    ext::Item_80268E5C(gobj, 3, 6);
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itEvyoshiegg_UnkMotion3_Phys");
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvyoshiegg_UnkMotion3_Coll");
    if ext::it_8026DA08(gobj) != 0 {
        return it_3F14_Logic42_DmgDealt(gobj);
    }
    0
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion5_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvyoshiegg_UnkMotion5_Anim");
    let ip = get_item(gobj);
    if (*ip).xDD4_itemVar.evyoshiegg.xDD8 == 40 {
        ext::gm_801BEB68(1);
    }
    (*ip).xDD4_itemVar.evyoshiegg.xDD8 -= 1;
    if (*ip).xDD4_itemVar.evyoshiegg.xDD8 > 0 {
        return 0;
    }
    1
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion5_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itEvyoshiegg_UnkMotion5_Phys");}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion5_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvyoshiegg_UnkMotion5_Coll");
    0
}

pub unsafe extern "C" fn itEvyoshiegg_BounceOff(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvyoshiegg_BounceOff");
    let ip = get_item(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 1.5;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
    0
}

pub unsafe extern "C" fn it_3F14_Logic42_DmgDealt(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_3F14_Logic42_DmgDealt");
    itEvyoshiegg_BounceOff(gobj)
}

pub unsafe extern "C" fn it_3F14_Logic42_Clanked(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_3F14_Logic42_Clanked");
    itEvyoshiegg_BounceOff(gobj)
}

pub unsafe extern "C" fn it_3F14_Logic42_HitShield(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_3F14_Logic42_HitShield");
    itEvyoshiegg_BounceOff(gobj)
}

pub unsafe extern "C" fn it_3F14_Logic42_Reflected(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_3F14_Logic42_Reflected");
    itEvyoshiegg_BounceOff(gobj)
}

pub unsafe extern "C" fn dmgReceived(gobj: *mut Item_GObj) {
    melee_core::ct!("dmgReceived");
    let ip = get_item(gobj);
    ext::HSD_JObjSetFlagsAll((*gobj).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as _);
    ext::it_802756D0(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).xDD4_itemVar.evyoshiegg.xDD4 = 1;
    (*ip).xDD4_itemVar.evyoshiegg.xDD8 = 60;
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itEvYoshiEgg_Logic42_DmgReceived(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvYoshiEgg_Logic42_DmgReceived");
    let ip = get_item(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itEvYoshiEgg_DatAttrs;
    let mut val: f32;
    if (*ip).xDD4_itemVar.evyoshiegg.xDD4 != 0 {
        return 0;
    }
    if (*ip).xC9C < (*attrs).x0 {
        return 0;
    }
    val = 1.5;
    ext::efSync_Spawn_N(1231, gobj, &[&mut (*ip).pos as *mut Vec as *mut c_void, &mut val as *mut f32 as *mut c_void]);
    ext::efSync_Spawn_N(1231, gobj, &[&mut (*ip).pos as *mut Vec as *mut c_void, &mut val as *mut f32 as *mut c_void]);
    ext::Item_8026AE84(ip, 244, 0x7F, 0x40);
    dmgReceived(gobj);
    0
}

pub unsafe extern "C" fn itEvYoshiEgg_Logic42_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itEvYoshiEgg_Logic42_EnteredAir");
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvyoshiegg_UnkMotion4_Anim");
    0
}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion4_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itEvyoshiegg_UnkMotion4_Phys");}

pub unsafe extern "C" fn itEvyoshiegg_UnkMotion4_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itEvyoshiegg_UnkMotion4_Coll");
    ext::it_8026E8C4(gobj, Some(it_8029B1D8), Some(it_8029B268));
    0
}

pub unsafe extern "C" fn itEvYoshiEgg_Logic42_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itEvYoshiEgg_Logic42_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
