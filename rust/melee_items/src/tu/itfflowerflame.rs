//! Native port of src/melee/it/kinds/itfflowerflame.c: the Fire Flower flame shot item (spawn off a
//! parent bone, scale / offset updates, the state callbacks and the state table `it_803F66C8`).
//! Callees in other TUs go through `ext` (hooks; unset panics "TODO"). Checked against the C (the
//! whole TU, shipping contraction) by tests/tu_itfflowerflame.rs.
//!
//! Header inlines compiled into the TU: `Item_InitSpawnPositionFromParent`,
//! `Item_InitSpawnCommonFields`, `HSD_JObjSetTranslate` / `HSD_JObjGetTranslation` (jobj.h, with
//! their `HSD_ASSERT`s and the `HSD_JObjSetMtxDirty` macro), `GET_ITEM`.
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026DF34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DF34(a0) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272CC0(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj => unsafe { crate::itkinds11::it_80272cc0(a0, a1) };
    fn it_80272F7C(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::it_2725::it_80272F7C(a0, a1) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `HSD_ASSERT(line, cond)`: the failing path is never taken by well-formed state.
#[inline]
fn hsd_assert(cond: bool, what: &str) {
    if !cond {
        panic!("HSD_ASSERT failed: {what}");
    }
}

/// `HSD_JObjMtxIsDirty` (jobj.h).
#[inline]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    hsd_assert(!jobj.is_null(), "jobj");
    let mut result = false;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0 {
        result = true;
    }
    result
}

/// `HSD_JObjSetTranslate` (jobj.h) with the `HSD_JObjSetMtxDirty` macro.
#[inline]
unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert(!translate.is_null(), "translate");
    (*jobj).translate = *translate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `HSD_JObjGetTranslation` (jobj.h).
#[inline]
unsafe fn HSD_JObjGetTranslation(jobj: *mut HSD_JObj, translate: *mut Vec) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert(!translate.is_null(), "translate");
    *translate = (*jobj).translate;
}

/// `Item_InitSpawnPositionFromParent` (kinds/inlines.h).
#[inline]
unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &mut (*spawn).pos);
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
    (*spawn).x44_flag.x0.set_b0(initial_collision as u8);
    (*spawn).x40 = 0;
}

pub static mut it_803F66C8: [ItemStateTable; 5] = [
    ItemStateTable { anim_id: 0, animated: Some(itFflowerflame_UnkMotion0_Anim), physics_updated: Some(itFflowerflame_UnkMotion0_Phys), collided: Some(itFflowerflame_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itFflowerflame_UnkMotion1_Anim), physics_updated: Some(itFflowerflame_UnkMotion1_Phys), collided: Some(itFflowerflame_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itFflowerflame_UnkMotion2_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 0, animated: Some(itFflowerflame_UnkMotion3_Anim), physics_updated: Some(itFflowerflame_UnkMotion3_Phys), collided: Some(itFflowerflame_UnkMotion3_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itFflowerflame_UnkMotion4_Anim), physics_updated: Some(itFflowerflame_UnkMotion4_Phys), collided: Some(itFflowerflame_UnkMotion4_Coll) },
];

pub unsafe extern "C" fn it_8029A748(arg0: *mut Item_GObj, arg1: *mut Vec, arg2: Fighter_Part, arg3: f32) -> *mut HSD_GObj {
    melee_core::ct!("it_8029A748");
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_F_Flower_Flame;
    Item_InitSpawnPositionFromParent(&mut spawn, arg0, arg1);
    Item_InitSpawnCommonFields(&mut spawn, arg0, arg3, true);
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let mut temp_pos: Vec = core::mem::zeroed();
        let item = get_item(gobj);
        let lifetime = *((*(*item).xC4_article_data).x4_specialAttributes as *mut f32);
        ext::it_80275158(gobj, lifetime);
        ext::Item_8026AB54(gobj, arg0, arg2);
        ext::it_8026B3A8(gobj);
        let jobj = ext::it_80272CC0(gobj, 2);
        HSD_JObjGetTranslation(jobj, &mut temp_pos);
        (*item).xDD4_itemVar.fflowerflame.pos.x = temp_pos.x;
        (*item).xDD4_itemVar.fflowerflame.pos.y = temp_pos.y;
        (*item).xDD4_itemVar.fflowerflame.pos.z = temp_pos.z;
    }
    gobj
}

pub unsafe extern "C" fn it_8029A89C(arg0: *mut Item_GObj, arg1: f32) {
    melee_core::ct!("it_8029A89C");
    if !arg0.is_null() {
        let item = get_item(arg0);
        if !item.is_null() {
            let temp_r3 = ext::it_80272CC0(arg0, 1);
            (*item).xDD4_itemVar.fflowerflame.unk = arg1;
            ext::it_80272F7C(temp_r3, (*item).xDD4_itemVar.fflowerflame.unk);
        }
    }
}

pub unsafe extern "C" fn it_8029A8F4(arg0: *mut Item_GObj, arg1: *mut Vec) {
    melee_core::ct!("it_8029A8F4");
    if !arg0.is_null() {
        let item = get_item(arg0);
        if !item.is_null() {
            let jobj = ext::it_80272CC0(arg0, 2);
            let mut temp_pos: Vec = core::mem::zeroed();
            temp_pos.x = (*item).xDD4_itemVar.fflowerflame.pos.x + (*arg1).x;
            temp_pos.y = (*item).xDD4_itemVar.fflowerflame.pos.y + (*arg1).y;
            temp_pos.z = (*item).xDD4_itemVar.fflowerflame.pos.z + (*arg1).z;
            HSD_JObjSetTranslate(jobj, &mut temp_pos);
        }
    }
}

pub unsafe extern "C" fn itFFlowerFlame_Logic41_Spawned(arg0: *mut Item_GObj) {
    melee_core::ct!("itFFlowerFlame_Logic41_Spawned");
    ext::it_8026B3A8(arg0);
}

pub unsafe extern "C" fn it_8029AA1C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8029AA1C");
    let item = get_item(gobj);
    (*item).x40_vel.z = 0.0;
    (*item).x40_vel.y = 0.0;
    (*item).x40_vel.x = 0.0;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as _);
    ext::it_8026B3A8(gobj);
}

pub unsafe extern "C" fn itFflowerflame_UnkMotion0_Anim(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflowerflame_UnkMotion0_Anim");
    let ip = get_item(arg0);
    ext::it_80272F7C(ext::it_80272CC0(arg0, 1), (*ip).xDD4_itemVar.fflowerflame.unk);
    let jobj = ext::it_80272CC0(arg0, 2);
    HSD_JObjSetTranslate(jobj, &mut (*ip).xDD4_itemVar.fflowerflame.pos);
    0
}

pub unsafe extern "C" fn itFflowerflame_UnkMotion0_Phys(_arg0: *mut HSD_GObj) { melee_core::ct!("itFflowerflame_UnkMotion0_Phys");}

pub unsafe extern "C" fn itFflowerflame_UnkMotion0_Coll(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflowerflame_UnkMotion0_Coll");
    ext::it_8026D62C(arg0, Some(it_8029AB90));
    0
}

pub unsafe extern "C" fn it_8029AB90(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8029AB90");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
    ext::it_8026B3A8(gobj);
}

pub unsafe extern "C" fn itFflowerflame_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflowerflame_UnkMotion1_Anim");
    let item = get_item(gobj);
    ext::it_80272F7C(ext::it_80272CC0(gobj, 1), (*item).xDD4_itemVar.fflowerflame.unk);
    let jobj = ext::it_80272CC0(gobj, 2);
    HSD_JObjSetTranslate(jobj, &mut (*item).xDD4_itemVar.fflowerflame.pos);
    ext::it_80273130(gobj)
}

pub unsafe extern "C" fn itFflowerflame_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFflowerflame_UnkMotion1_Phys");
    let item = get_item(gobj);
    let temp_r4 = (*item).xCC_item_attr;
    ext::it_80272860(gobj, (*temp_r4).x10_fall_speed, (*temp_r4).x14_fall_speed_max);
}

pub unsafe extern "C" fn itFflowerflame_UnkMotion1_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itFflowerflame_UnkMotion1_Coll");
    ext::it_8026DF34(gobj)
}

pub unsafe extern "C" fn itFFlowerFlame_Logic41_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itFFlowerFlame_Logic41_PickedUp");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as _);
    ext::it_8026B3A8(gobj);
}

pub unsafe extern "C" fn itFflowerflame_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflowerflame_UnkMotion2_Anim");
    let ip = get_item(gobj);
    ext::it_80272F7C(ext::it_80272CC0(gobj, 1), (*ip).xDD4_itemVar.fflowerflame.unk);
    0
}

pub unsafe extern "C" fn itFFlowerFlame_Logic41_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itFFlowerFlame_Logic41_Dropped");
    ext::Item_80268E5C(gobj, 1, 6);
    ext::it_8026B3A8(gobj);
}

pub unsafe extern "C" fn itFflowerflame_UnkMotion3_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflowerflame_UnkMotion3_Anim");
    let item = get_item(gobj);
    ext::it_80272F7C(ext::it_80272CC0(gobj, 1), (*item).xDD4_itemVar.fflowerflame.unk);
    let jobj = ext::it_80272CC0(gobj, 2);
    HSD_JObjSetTranslate(jobj, &mut (*item).xDD4_itemVar.fflowerflame.pos);
    ext::it_80273130(gobj)
}

pub unsafe extern "C" fn itFflowerflame_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFflowerflame_UnkMotion3_Phys");
    let item = get_item(gobj);
    let attrs = (*item).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itFflowerflame_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflowerflame_UnkMotion3_Coll");
    if ext::it_8026DA08(gobj) != 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn itFFlowerFlame_Logic41_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itFFlowerFlame_Logic41_EnteredAir");
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as _);
    ext::it_8026B3A8(gobj);
}

pub unsafe extern "C" fn itFflowerflame_UnkMotion4_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflowerflame_UnkMotion4_Anim");
    let item = get_item(gobj);
    ext::it_80272F7C(ext::it_80272CC0(gobj, 1), (*item).xDD4_itemVar.fflowerflame.unk);
    let jobj = ext::it_80272CC0(gobj, 2);
    HSD_JObjSetTranslate(jobj, &mut (*item).xDD4_itemVar.fflowerflame.pos);
    ext::it_80273130(gobj)
}

pub unsafe extern "C" fn itFflowerflame_UnkMotion4_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itFflowerflame_UnkMotion4_Phys");}

pub unsafe extern "C" fn itFflowerflame_UnkMotion4_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFflowerflame_UnkMotion4_Coll");
    ext::it_8026E8C4(gobj, Some(it_8029AA1C), Some(it_8029AB90));
    0
}

pub unsafe extern "C" fn itFFlowerFlame_Logic41_EvtUnk(arg0: *mut Item_GObj, arg1: *mut Item_GObj) {
    melee_core::ct!("itFFlowerFlame_Logic41_EvtUnk");
    ext::it_8026B894(arg0, arg1);
}
