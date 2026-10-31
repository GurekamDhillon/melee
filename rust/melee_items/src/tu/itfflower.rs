//! Native port of src/melee/it/kinds/itfflower.c: the Fire Flower item (spawn, the flame shots
//! with their ignore-id rotation, the state callbacks and the state table `it_803F6090`). Callees
//! in other TUs go through `ext` (hooks; unset panics "TODO"). Checked against the C (the whole TU,
//! shipping contraction) by tests/tu_itfflower.rs.
//!
//! Header inlines compiled into the TU: `Item_InitSpawnPosition`, `Item_InitSpawnCommonFields`,
//! `Item_StopAndEnterState`, `itResetVelocity`. The variadic `efSync_Spawn` is
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
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE60() -> u32 => unsafe { crate::tu::item::Item_8026AE60() };
    fn efLib_DestroyAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_DestroyAll(a0) };
    fn efSync_Spawn_N(a0: i32, a1: *mut HSD_GObj, a2: &[*mut c_void]) -> *mut c_void;
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BE28(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026be28(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DF34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DF34(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274F10(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274f10(&mut *((*a0).user_data as *mut Item)) };
    fn it_802996D0(a0: *mut HSD_GObj, a1: *mut Vec, a2: u32, a3: f32) -> *mut HSD_GObj => unsafe { crate::tu::itlgunbeam::it_802996D0(a0, a1 as _, a2, a3) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
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

/// `Item_StopAndEnterState` (kinds/inlines.h).
#[inline]
unsafe fn Item_StopAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    itResetVelocity(ip);
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as _);
}

pub static mut it_803F6090: [ItemStateTable; 8] = [
    ItemStateTable { anim_id: 0, animated: Some(itFflower_UnkMotion0_Anim), physics_updated: Some(itFflower_UnkMotion0_Phys), collided: Some(itFflower_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itFflower_UnkMotion6_Anim), physics_updated: Some(itFflower_UnkMotion1_Phys), collided: Some(itFflower_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itFflower_UnkMotion5_Anim), physics_updated: Some(itFflower_UnkMotion5_Phys), collided: None },
    ItemStateTable { anim_id: 1, animated: Some(itFflower_UnkMotion5_Anim), physics_updated: Some(itFflower_UnkMotion5_Phys), collided: None },
    ItemStateTable { anim_id: 2, animated: Some(itFflower_UnkMotion5_Anim), physics_updated: Some(itFflower_UnkMotion5_Phys), collided: None },
    ItemStateTable { anim_id: 3, animated: Some(itFflower_UnkMotion5_Anim), physics_updated: Some(itFflower_UnkMotion5_Phys), collided: None },
    ItemStateTable { anim_id: 4, animated: Some(itFflower_UnkMotion6_Anim), physics_updated: Some(itFflower_UnkMotion6_Phys), collided: Some(itFflower_UnkMotion6_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itFflower_UnkMotion7_Anim), physics_updated: Some(itFflower_UnkMotion7_Phys), collided: Some(itFflower_UnkMotion7_Coll) },
];

pub unsafe extern "C" fn msid_check(gobj: *mut Item_GObj, ip: *mut Item) {
    melee_core::ct!("msid_check");
    match (*ip).msid {
        0 | 1 | 2 | 3 | 5 | 6 | 7 => {
            ext::efLib_DestroyAll(gobj);
            (*ip).xDD4_itemVar.fflower.x4 = 0;
        }
        4 => {
            if (*ip).xD4C <= 0 {
                ext::efLib_DestroyAll(gobj);
                (*ip).xDD4_itemVar.fflower.x4 = 0;
            }
        }
        _ => {}
    }
}

pub unsafe extern "C" fn it_80292D48(vec: *mut Vec) -> *mut Item_GObj {
    melee_core::ct!("it_80292D48");
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_F_Flower;
    Item_InitSpawnPosition(&mut spawn, vec, true);
    Item_InitSpawnCommonFields(&mut spawn, null_mut(), -1.0, true);
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        it_80292FF0(gobj);
    }
    gobj
}

pub unsafe extern "C" fn itFFlower_Logic25_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itFFlower_Logic25_Spawned");
    let ip = get_item(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut FFlowerAttr;
    (*ip).xD4C = (*attr).x0;
    (*ip).xAC4_ignoreItemID = ext::Item_8026AE60();
    (*ip).xDD4_itemVar.fflower.x0 = 0;
    (*ip).xDD4_itemVar.fflower.x4 = 0;
    it_8029313C(gobj);
}

pub unsafe extern "C" fn it_80292E64(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80292E64");
    let ip = get_item(gobj);
    match (*ip).msid {
        2 | 5 => ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as _),
        _ => {}
    }
}

pub unsafe extern "C" fn it_80292EAC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80292EAC");
    let ip = get_item(gobj);
    match (*ip).msid {
        3 | 4 => ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as _),
        _ => {}
    }
}

pub unsafe extern "C" fn it_80292EF8(_unused: *mut Item_GObj, vec: *mut Vec) {
    melee_core::ct!("it_80292EF8");
    (*vec).x = 0.0;
    (*vec).y = 5.5;
    (*vec).z = 2.5;
}

pub unsafe extern "C" fn it_80292F14(gobj: *mut Item_GObj, arg1: *mut Vec, arg8: f32) {
    melee_core::ct!("it_80292F14");
    let ip = get_item(gobj);
    if (*ip).xD4C > 0 {
        if (*ip).xDD4_itemVar.fflower.x4 == 0 {
            ext::efSync_Spawn_N(0x473, gobj, &[(*(*ip).xBBC_dynamicBoneTable).bones[4] as *mut c_void]);
            (*ip).xDD4_itemVar.fflower.x4 = 1;
        }
        (*ip).xD4C -= 1;
        ext::it_802996D0((*ip).owner, arg1, (*ip).xAC4_ignoreItemID, arg8);
        (*ip).xDD4_itemVar.fflower.x0 += 1;
        (*ip).xDD4_itemVar.fflower.x0 %= 6;
        if (*ip).xDD4_itemVar.fflower.x0 == 0 {
            (*ip).xAC4_ignoreItemID = ext::Item_8026AE60();
        }
    }
}

pub unsafe extern "C" fn it_80292FF0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80292FF0");
    let ip = get_item(gobj);
    Item_StopAndEnterState(gobj, ip, 0);
}

pub unsafe extern "C" fn itFflower_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflower_UnkMotion0_Anim");
    let ip = get_item(gobj);
    msid_check(gobj, ip);
    let flag = (*ip).xDC8_word.flags.x2() as i32;
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as _);
        if flag != 0 {
            ext::it_80274F10(gobj);
            ext::it_8026BE28(gobj);
        }
    }
    0
}

pub unsafe extern "C" fn itFflower_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itFflower_UnkMotion0_Phys");}

pub unsafe extern "C" fn itFflower_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflower_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_8029313C));
    0
}

pub unsafe extern "C" fn it_8029313C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8029313C");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itFflower_UnkMotion6_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflower_UnkMotion6_Anim");
    let ip = get_item(gobj);
    msid_check(gobj, ip);
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
    }
    0
}

pub unsafe extern "C" fn itFflower_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFflower_UnkMotion1_Phys");
    let ip = get_item(gobj);
    let attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itFflower_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflower_UnkMotion1_Coll");
    let ip = get_item(gobj);
    if (*ip).xD4C != 0 {
        ext::it_8026E15C(gobj, Some(it_80292FF0));
        return 0;
    }
    ext::it_8026DF34(gobj)
}

pub unsafe extern "C" fn itFFlower_Logic25_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itFFlower_Logic25_PickedUp");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itFflower_UnkMotion5_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflower_UnkMotion5_Anim");
    let ip = get_item(gobj);
    msid_check(gobj, ip);
    match (*ip).msid {
        2 => {
            if ext::it_80272C6C(gobj) == 0 {
                ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as _);
            }
        }
        3 | 4 => {
            if ext::it_80272C6C(gobj) == 0 {
                ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as _);
            }
        }
        5 => {
            if ext::it_80272C6C(gobj) == 0 {
                ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as _);
            }
        }
        _ => {}
    }
    0
}

pub unsafe extern "C" fn itFflower_UnkMotion5_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itFflower_UnkMotion5_Phys");}

pub unsafe extern "C" fn itFFlower_Logic25_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itFFlower_Logic25_Dropped");
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itFFlower_Logic25_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itFFlower_Logic25_Thrown");
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itFflower_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFflower_UnkMotion6_Phys");
    let ip = get_item(gobj);
    let attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itFflower_UnkMotion6_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflower_UnkMotion6_Coll");
    let ip = get_item(gobj);
    if (*ip).xD4C != 0 {
        ext::it_8026E15C(gobj, Some(it_80292FF0));
        return 0;
    }
    ext::it_8026DF34(gobj)
}

pub unsafe extern "C" fn itFFlower_Logic25_DmgDealt(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFFlower_Logic25_DmgDealt");
    ext::itColl_BounceOffVictim(gobj);
    0
}

pub unsafe extern "C" fn itFFlower_Logic25_Clanked(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFFlower_Logic25_Clanked");
    ext::itColl_BounceOffVictim(gobj);
    0
}

pub unsafe extern "C" fn itFFlower_Logic25_HitShield(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFFlower_Logic25_HitShield");
    ext::itColl_BounceOffVictim(gobj);
    0
}

pub unsafe extern "C" fn itFFlower_Logic25_Reflected(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFFlower_Logic25_Reflected");
    ext::it_80273030(gobj)
}

pub unsafe extern "C" fn itFFlower_Logic25_ShieldBounced(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFFlower_Logic25_ShieldBounced");
    ext::itColl_BounceOffShield(gobj)
}

pub unsafe extern "C" fn itFFlower_Logic25_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itFFlower_Logic25_EnteredAir");
    ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itFflower_UnkMotion7_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflower_UnkMotion7_Anim");
    let ip = get_item(gobj);
    msid_check(gobj, ip);
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE as _);
    }
    0
}

pub unsafe extern "C" fn itFflower_UnkMotion7_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itFflower_UnkMotion7_Phys");}

pub unsafe extern "C" fn itFflower_UnkMotion7_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflower_UnkMotion7_Coll");
    ext::it_8026E8C4(gobj, Some(it_80292FF0), Some(it_8029313C));
    0
}

pub unsafe extern "C" fn itFFlower_Logic25_EvtUnk(gobj: *mut Item_GObj, ref_: *mut Item_GObj) {
    melee_core::ct!("itFFlower_Logic25_EvtUnk");
    ext::it_8026B894(gobj, ref_);
}
