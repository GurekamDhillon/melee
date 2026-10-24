//! Native port of src/melee/it/kinds/itfoxillusion.c: Fox's Illusion (SpecialS) afterimage item. The item
//! follows the fighter's ghost positions (`ftFx_SpecialS_*`), optionally owns a second "ghost" jobj loaded
//! from the article's joint, and counts its lifetime down. Callees in other TUs go through `ext` (unported:
//! panics "TODO"); the header inlines the TU uses (`Item_InitSpawnPosition`, `Item_InitSpawnCommonFields`,
//! the jobj.h accessors) are ported here. Checked against the C (the whole TU, shipping contraction) by
//! tests/tu_itfoxillusion.rs.
//!
//! No fused multiply-add sites. `it_8029CD78` hangs forever (`OSReport` then `while (1)`) when the ghost jobj
//! exists already; the port loops the same way (the oracle never reaches it).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::inline as ji;
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
    fn HSD_GObj_80390EB8(a0: i32) -> u32 => unsafe { melee_runtime::gobj::hsd_gobj_80390eb8(a0) };
    fn HSD_JObjDispAll(a0: *mut HSD_JObj, a1: *mut c_void, a2: u32, a3: u32);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjRemoveAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftFx_SpecialS_CheckGhostRemove(a0: *mut HSD_GObj) -> i32;
    fn ftFx_SpecialS_CopyGhostPosIndexed(a0: *mut HSD_GObj, a1: i32, a2: *mut Vec);
    fn ftFx_SpecialS_GetCmdVar2(a0: *mut HSD_GObj) -> i32;
    fn ftFx_SpecialS_ReturnFloatVarIndexed(a0: *mut HSD_GObj, a1: i32) -> f32;
    fn ftLib_800865CC(a0: *mut HSD_GObj) -> i32;
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026EECC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itdraw::it_8026EECC(a0, a1) };
    fn it_80272A3C(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A3C(a0) };
    fn it_80272F7C(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::it_2725::it_80272F7C(a0, a1) };
    fn it_80274594(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274594(a0) };
    fn it_802750F8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802750F8(a0) };
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

/// `Item_InitSpawnPosition` (itkinds inlines.h).
#[inline]
pub unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *const Vec, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_InitSpawnCommonFields` (itkinds inlines.h).
#[inline]
pub unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.z = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.x = 0.0;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as u8);
    (*spawn).x40 = 0;
}

/// `ItemStateTable it_803F6818[3]`.
pub static it_803F6818: [ItemStateTable; 3] = [
    ItemStateTable { anim_id: 0, animated: Some(itFoxillusion_UnkMotion0_Anim), physics_updated: Some(itFoxillusion_UnkMotion0_Phys), collided: Some(itFoxillusion_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itFoxillusion_UnkMotion1_Anim), physics_updated: Some(itFoxillusion_UnkMotion1_Phys), collided: Some(itFoxillusion_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itFoxillusion_UnkMotion2_Anim), physics_updated: Some(itFoxillusion_UnkMotion2_Phys), collided: Some(itFoxillusion_UnkMotion2_Coll) },
];

/// `void it_8029CD18(Item_GObj*, int)`: draws the ghost jobj, when there is one, after the item.
pub unsafe extern "C" fn it_8029CD18(item_gobj: *mut Item_GObj, arg1: i32) {
    let item = item_of(item_gobj);
    let jobj = (*item).xDD4_itemVar.foxillusion.xDDC;
    ext::it_8026EECC(item_gobj, arg1);
    if !jobj.is_null() {
        let mask = ext::HSD_GObj_80390EB8(arg1);
        ext::HSD_JObjDispAll(jobj, null_mut(), mask, 0);
    }
}

/// `HSD_JObj* it_8029CD78(Item_GObj*)`: loads the ghost jobj from the article joint, with the item jobj's
/// rotation and the item scale.
pub unsafe extern "C" fn it_8029CD78(item_gobj: *mut Item_GObj) -> *mut HSD_JObj {
    let mut quat: Quaternion = core::mem::zeroed();
    let item = item_of(item_gobj);
    let temp_r30 = jobj_of(item_gobj);
    if !(*item).xDD4_itemVar.foxillusion.xDDC.is_null() {
        // OSReport("illusion add twice.\n");
        loop {}
    }
    let temp_r29 = ext::HSD_JObjLoadJoint((*item).xDD4_itemVar.foxillusion.xDD4);
    ji::HSD_JObjGetRotation(jref(temp_r30), &mut quat);
    ji::HSD_JObjSetRotation(jref(temp_r29), &quat, &mut Dirty);
    ext::it_80272F7C(temp_r29, (*item).scl);
    temp_r29
}

/// `Item_GObj* it_8029CEB4(HSD_GObj* parent, Vec3* pos, ItemKind, f32 dir)`: spawns the illusion item.
pub unsafe extern "C" fn it_8029CEB4(parent: *mut HSD_GObj, pos: *mut Vec, kind: ItemKind, dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = kind;
    Item_InitSpawnPosition(&mut spawn, pos, false);
    Item_InitSpawnCommonFields(&mut spawn, parent, dir, true);
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        it_8029CFF0(item_gobj);
        ext::db_80225DD8(item_gobj, parent);
        ext::it_802750F8(item_gobj);
        return item_gobj;
    }
    null_mut()
}

/// `bool itFoxIllusion_Logic14_DmgDealt(Item_GObj*)`: clears the hitlag counter.
pub unsafe extern "C" fn itFoxIllusion_Logic14_DmgDealt(item_gobj: *mut Item_GObj) -> i32 {
    let item = item_of(item_gobj);
    (*item).xCA8 = 0;
    0
}

/// `void itFoxIllusion_Logic14_Destroyed(Item_GObj*)`: removes the ghost jobj and the owner link.
pub unsafe extern "C" fn itFoxIllusion_Logic14_Destroyed(item_gobj: *mut Item_GObj) {
    let item = item_of(item_gobj);
    if !(*item).xDD4_itemVar.foxillusion.xDDC.is_null() {
        ext::HSD_JObjRemoveAll((*item).xDD4_itemVar.foxillusion.xDDC);
    }
    (*item).xDD4_itemVar.foxillusion.xDDC = null_mut();
    (*item).owner = null_mut();
    (*item).xDC8_word.flags.set_x13(0);
}

/// `void it_8029CFF0(Item_GObj*)`: item init: lifetime from the attributes, the model joint, the first state.
pub unsafe extern "C" fn it_8029CFF0(item_gobj: *mut Item_GObj) {
    let item = item_of(item_gobj);
    (*item).xD44_lifeTimer = *((*(*item).xC4_article_data).x4_specialAttributes as *const f32);
    (*item).xD5C = 0;
    (*item).xDD4_itemVar.foxillusion.xDD4 = (*(*(*item).xC4_article_data).x10_modelDesc).x0_joint;
    ext::it_8026B3A8(item_gobj);
    if ext::ftLib_800865CC((*item).owner) == 1 {
        ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
    }
    ext::it_80274594(item_gobj);
    (*item).xDD4_itemVar.foxillusion.xDDC = null_mut();
}

/// `bool itFoxillusion_UnkMotion0_Anim(Item_GObj*)`: creates the ghost jobj when the fighter's cmd var is 2,
/// ends the item when the fighter says so, counts the lifetime down.
pub unsafe extern "C" fn itFoxillusion_UnkMotion0_Anim(item_gobj: *mut Item_GObj) -> i32 {
    let temp_r31 = item_of(item_gobj);
    if !(*temp_r31).owner.is_null()
        && ext::ftFx_SpecialS_GetCmdVar2((*temp_r31).owner) == 2
        && (*temp_r31).xDD4_itemVar.foxillusion.xDDC.is_null()
    {
        (*temp_r31).xDD4_itemVar.foxillusion.xDDC = it_8029CD78(item_gobj);
    }
    let item = item_of(item_gobj);
    let remove = if !(*item).owner.is_null() { ext::ftFx_SpecialS_CheckGhostRemove((*item).owner) } else { 1 };
    if remove != 0 {
        return 1;
    }
    (*temp_r31).xD44_lifeTimer -= 1.0;
    if (*temp_r31).xD44_lifeTimer <= 0.0 {
        (*temp_r31).xD44_lifeTimer = 0.0;
        it_8029D798(item_gobj);
    }
    0
}

/// `itFoxillusion_Phys` (static inline): copies the fighter's ghost positions / rotations onto the item and
/// the ghost jobj.
pub unsafe fn itFoxillusion_Phys(item_gobj: *mut Item_GObj, ghost_pos: *mut Vec) {
    let item = item_of(item_gobj);
    let jobj = jobj_of(item_gobj);
    let ghost_jobj = (*item).xDD4_itemVar.foxillusion.xDDC;
    if !(*item).owner.is_null() {
        ext::ftFx_SpecialS_CopyGhostPosIndexed((*item).owner, 1, &mut (*item).pos);
        let rot = ext::ftFx_SpecialS_ReturnFloatVarIndexed((*item).owner, 1);
        ji::HSD_JObjSetRotationX(jref(jobj), rot, &mut Dirty);
        if !ghost_jobj.is_null() {
            ext::ftFx_SpecialS_CopyGhostPosIndexed((*item).owner, 3, ghost_pos);
            ji::HSD_JObjSetTranslate(jref(ghost_jobj), &*ghost_pos, &mut Dirty);
            let rot = ext::ftFx_SpecialS_ReturnFloatVarIndexed((*item).owner, 3);
            ji::HSD_JObjSetRotationX(jref(ghost_jobj), rot, &mut Dirty);
        }
    }
}

/// `void itFoxillusion_UnkMotion0_Phys(Item_GObj*)`.
pub unsafe extern "C" fn itFoxillusion_UnkMotion0_Phys(item_gobj: *mut Item_GObj) {
    let mut ghost_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
    itFoxillusion_Phys(item_gobj, &mut ghost_pos);
}

/// `bool itFoxillusion_UnkMotion0_Coll(Item_GObj*)`: does nothing.
pub unsafe extern "C" fn itFoxillusion_UnkMotion0_Coll(arg0: *mut Item_GObj) -> i32 {
    0
}

/// `bool itFoxillusion_UnkMotion1_Anim(Item_GObj*)`: same as state 0.
pub unsafe extern "C" fn itFoxillusion_UnkMotion1_Anim(item_gobj: *mut Item_GObj) -> i32 {
    itFoxillusion_UnkMotion0_Anim(item_gobj)
}

/// `void itFoxillusion_UnkMotion1_Phys(Item_GObj*)`.
pub unsafe extern "C" fn itFoxillusion_UnkMotion1_Phys(item_gobj: *mut Item_GObj) {
    let mut ghost_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
    itFoxillusion_Phys(item_gobj, &mut ghost_pos);
}

/// `bool itFoxillusion_UnkMotion1_Coll(Item_GObj*)`: does nothing.
pub unsafe extern "C" fn itFoxillusion_UnkMotion1_Coll(arg0: *mut Item_GObj) -> i32 {
    0
}

/// `void it_8029D798(Item_GObj*)`: enters the fade-out state with the second lifetime from the attributes.
pub unsafe extern "C" fn it_8029D798(item_gobj: *mut Item_GObj) {
    let item = item_of(item_gobj);
    let jobj = jobj_of(item_gobj);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *const f32;
    (*item).xD44_lifeTimer = *attr.add(1);
    ext::it_80272A3C(jobj);
    ext::Item_80268E5C(item_gobj, 2, ITEM_ANIM_UPDATE);
}

/// `bool itFoxillusion_UnkMotion2_Anim(Item_GObj*)`: fade-out countdown.
pub unsafe extern "C" fn itFoxillusion_UnkMotion2_Anim(item_gobj: *mut Item_GObj) -> i32 {
    let item = item_of(item_gobj);
    let tmp = if !(*item).owner.is_null() { ext::ftFx_SpecialS_CheckGhostRemove((*item).owner) != 0 } else { true };
    if tmp {
        return 1;
    }
    (*item).xD44_lifeTimer -= 1.0;
    if (*item).xD44_lifeTimer <= 0.0 {
        (*item).xD44_lifeTimer = 0.0;
        return 1;
    }
    0
}

/// `void itFoxillusion_UnkMotion2_Phys(Item_GObj*)`: moves the ghost jobj to the fighter's ghost position.
pub unsafe extern "C" fn itFoxillusion_UnkMotion2_Phys(item_gobj: *mut Item_GObj) {
    let mut sp10 = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let item = item_of(item_gobj);
    let jobj = (*item).xDD4_itemVar.foxillusion.xDDC;
    if !(*item).owner.is_null() && !jobj.is_null() {
        ext::ftFx_SpecialS_CopyGhostPosIndexed((*item).owner, 3, &mut sp10);
        ji::HSD_JObjSetTranslate(jref(jobj), &sp10, &mut Dirty);
    }
}

/// `bool itFoxillusion_UnkMotion2_Coll(Item_GObj*)`: does nothing.
pub unsafe extern "C" fn itFoxillusion_UnkMotion2_Coll(arg0: *mut Item_GObj) -> i32 {
    0
}

/// `void it_8029D948(Item_GObj*, HSD_GObj*)`: removes the gobj interaction references.
pub unsafe extern "C" fn it_8029D948(item_gobj: *mut Item_GObj, r#ref: *mut HSD_GObj) {
    ext::it_8026B894(item_gobj, r#ref);
}
