//! Native port of src/melee/it/kinds/itgamewatchchef.c: Mr. Game & Watch's Chef (the thrown food: the
//! side-special's pan contents): spawn with a launch table entry, a spinning arc under gravity, the
//! bounce / land transition into state 1 (the fade-out blink), and the reflect / absorb / clank callbacks.
//! Callees in other TUs go through `ext` (unported: panics "TODO"); the header inlines the TU uses
//! (`Item_InitSpawnPositionFromParent`, `itReflectItemAndUpdateRotation`, the jobj.h accessors) are ported
//! here. Checked against the C (the whole TU, shipping contraction) by tests/tu_itgamewatchchef.rs.
//!
//! No fused multiply-add sites. `itGamewatchchefAttributes::entries` is a one-element array in the type but
//! indexed with the launch-table index (the real data has five entries): indexed through the raw pointer.
//! `(s32) xD44_lifeTimer` is the saturating float-to-int conversion (`as i32`); the reflect inline's
//! `PI_2` is an `f32` constant (`const f32 PI_2 = M_PI / 2`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::inline as ji;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32;
    fn it_8026B3A8(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026BB20(a0: *mut HSD_GObj);
    fn it_8026BB44(a0: *mut HSD_GObj);
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32;
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_80273030(a0: *mut HSD_GObj) -> i32;
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    fn it_8027CE64(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: *mut c_void);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// jobj.h's `HSD_JObjSetMtxDirty` macro / out-of-line dirty marks.
struct Dirty;
impl ji::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !ji::HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

/// `HSD_ASSERT(line, jobj)` of the jobj.h accessors: a NULL jobj hangs the console.
#[inline]
unsafe fn jref<'a>(j: *mut HSD_JObj) -> &'a mut HSD_JObj {
    assert!(!j.is_null(), "jobj.h: HSD_ASSERT(jobj)");
    &mut *j
}

#[inline]
unsafe fn item_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

#[inline]
unsafe fn jobj_of(g: *mut HSD_GObj) -> *mut HSD_JObj {
    (*g).hsd_obj as *mut HSD_JObj
}

/// `attrs->entries[i]` (the type declares one entry; the table has more).
#[inline]
unsafe fn entry(attrs: *mut itGamewatchchefAttributes, i: i32) -> *mut itGamewatchchefAttrEntry {
    (&raw mut (*attrs).entries as *mut itGamewatchchefAttrEntry).offset(i as isize)
}

/// `Item_InitSpawnPositionFromParent` (itkinds inlines.h).
#[inline]
pub unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *const Vec) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &mut (*spawn).pos);
}

/// `itReflectItemAndUpdateRotation` (itkinds inlines.h): reflects and turns the model to the new facing.
pub unsafe fn itReflectItemAndUpdateRotation(gobj: *mut Item_GObj) -> i32 {
    let pi_2: f32 = (core::f64::consts::PI / 2.0) as f32;
    let ip = item_of(gobj);
    let jobj = jobj_of(gobj);

    ext::it_80273030(gobj);
    ji::HSD_JObjSetRotationY(jref(jobj), pi_2 * (*ip).facing_dir, &mut Dirty);
    0
}

/// `ItemStateTable it_803F79E0[2]`.
pub static it_803F79E0: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itGamewatchchef_UnkMotion0_Anim), physics_updated: Some(itGamewatchchef_UnkMotion0_Phys), collided: Some(itGamewatchchef_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itGamewatchchef_UnkMotion1_Anim), physics_updated: Some(itGamewatchchef_UnkMotion1_Phys), collided: Some(itGamewatchchef_UnkMotion1_Coll) },
];

/// `HSD_GObj* it_802C837C(Item_GObj* parent, Vec3*, enum_t kind, u32 arg3, float facing_dir)`: spawns the
/// chef item with launch entry `arg3`.
pub unsafe extern "C" fn it_802C837C(parent: *mut Item_GObj, pos: *mut Vec, kind: enum_t, arg3: u32, facing_dir: f32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind;
    Item_InitSpawnPositionFromParent(&mut spawn, parent, pos);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = parent;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let attrs = (*(*item_of(gobj)).xC4_article_data).x4_specialAttributes as *mut itGamewatchchefAttributes;
        it_802C84A0(gobj, arg3 as i32);
        ext::db_80225DD8(gobj, parent);
        ext::it_8027CE64(gobj, parent, (*attrs).x0);
        return gobj;
    }
    null_mut()
}

/// `bool itGameWatchChef_Logic112_DmgDealt(Item_GObj*)`.
pub unsafe extern "C" fn itGameWatchChef_Logic112_DmgDealt(gobj: *mut Item_GObj) -> i32 {
    it_802C875C(gobj);
    0
}

/// `void it_802C84A0(Item_GObj*, s32 index)`: launch with table entry `index`.
pub unsafe extern "C" fn it_802C84A0(gobj: *mut Item_GObj, index: i32) {
    let ip = item_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGamewatchchefAttributes;
    (*ip).xDD4_itemVar.gamewatchchef.x4 = index;
    ext::it_80275158(gobj, (*attrs).x8);
    (*ip).xD5C = 0;
    (*ip).x40_vel.x = (*entry(attrs, (*ip).xDD4_itemVar.gamewatchchef.x4)).x0 * (*ip).facing_dir;
    (*ip).x40_vel.y = (*entry(attrs, (*ip).xDD4_itemVar.gamewatchchef.x4)).x4;
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

/// `bool itGamewatchchef_UnkMotion0_Anim(Item_GObj*)`: spin and lifetime countdown.
pub unsafe extern "C" fn itGamewatchchef_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = (*gobj).user_data as *mut Item;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGamewatchchefAttributes;
    let mut rot_x = ji::HSD_JObjGetRotationX(jref(jobj));
    rot_x += (*entry(attrs, (*ip).xDD4_itemVar.gamewatchchef.x4)).x10;
    ji::HSD_JObjSetRotationX(jref(jobj), rot_x, &mut Dirty);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        (*ip).xD44_lifeTimer = 0.0;
        it_802C875C(gobj);
    }
    0
}

/// `void itGamewatchchef_UnkMotion0_Phys(Item_GObj*)`: gravity with the entry's fall values.
pub unsafe extern "C" fn itGamewatchchef_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = item_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGamewatchchefAttributes;
    let index = (*ip).xDD4_itemVar.gamewatchchef.x4;
    ext::it_80272860(gobj, (*entry(attrs, index)).x8, (*entry(attrs, index)).xC);
}

/// `bool itGamewatchchef_UnkMotion0_Coll(Item_GObj*)`: wall bounce, floor / ceiling -> state 1.
pub unsafe extern "C" fn itGamewatchchef_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    let ip = item_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGamewatchchefAttributes;
    if 0.0 != (*ip).x40_vel.x {
        let result = ext::it_8026DAA8(gobj);
        if result & 0xC != 0 {
            (*ip).x40_vel.x *= -(*attrs).x4;
        }
        if result & 3 != 0 {
            it_802C875C(gobj);
        }
    }
    0
}

/// `void it_802C875C(Item_GObj*)`: stops and enters state 1 with the second lifetime.
pub unsafe extern "C" fn it_802C875C(gobj: *mut Item_GObj) {
    let ip = item_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGamewatchchefAttributes;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    ext::it_80275158(gobj, (*attrs).xC);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

/// `bool itGamewatchchef_UnkMotion1_Anim(Item_GObj*)`: lifetime countdown with the blink.
pub unsafe extern "C" fn itGamewatchchef_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = item_of(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        (*ip).xD44_lifeTimer = 0.0;
        return 1;
    }
    if ((*ip).xD44_lifeTimer as i32) & 2 != 0 {
        ext::it_8026BB20(gobj);
    } else {
        ext::it_8026BB44(gobj);
    }
    0
}

/// `void itGamewatchchef_UnkMotion1_Phys(Item_GObj*)`.
pub unsafe extern "C" fn itGamewatchchef_UnkMotion1_Phys(gobj: *mut Item_GObj) {}

/// `bool itGamewatchchef_UnkMotion1_Coll(Item_GObj*)`.
pub unsafe extern "C" fn itGamewatchchef_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    itGamewatchchef_UnkMotion0_Coll(gobj)
}

/// `bool it_2725_Logic112_Clanked(Item_GObj*)`: stops and enters state 1.
pub unsafe extern "C" fn it_2725_Logic112_Clanked(gobj: *mut Item_GObj) -> i32 {
    let ip = item_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGamewatchchefAttributes;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    ext::it_80275158(gobj, (*attrs).xC);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    0
}

/// `bool it_2725_Logic112_HitShield(Item_GObj*)`: same as clanked.
pub unsafe extern "C" fn it_2725_Logic112_HitShield(gobj: *mut Item_GObj) -> i32 {
    let ip = item_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGamewatchchefAttributes;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    ext::it_80275158(gobj, (*attrs).xC);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    0
}

/// `bool it_2725_Logic112_Absorbed(Item_GObj*)`: as clanked, and the lifetime ends.
pub unsafe extern "C" fn it_2725_Logic112_Absorbed(gobj: *mut Item_GObj) -> i32 {
    let ip = item_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGamewatchchefAttributes;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    ext::it_80275158(gobj, (*attrs).xC);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    (*ip).xD44_lifeTimer = 0.0;
    0
}

/// `bool itGameWatchChef_Logic112_ShieldBounced(Item_GObj*)`.
pub unsafe extern "C" fn itGameWatchChef_Logic112_ShieldBounced(gobj: *mut Item_GObj) -> i32 {
    ext::itColl_BounceOffShield(gobj)
}

/// `bool itGameWatchChef_Logic112_Reflected(Item_GObj*)`.
pub unsafe extern "C" fn itGameWatchChef_Logic112_Reflected(gobj: *mut Item_GObj) -> i32 {
    itReflectItemAndUpdateRotation(gobj)
}

/// `void itGameWatchChef_Logic112_EvtUnk(Item_GObj*, Item_GObj*)`: removes the gobj interaction references.
pub unsafe extern "C" fn itGameWatchChef_Logic112_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
