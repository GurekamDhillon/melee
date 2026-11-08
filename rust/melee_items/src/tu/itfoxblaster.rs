//! Native port of src/melee/it/kinds/itfoxblaster.c: the Fox / Falco / Kirby-as-Fox/Falco blaster item
//! (the gun held while firing SpecialN): model transforms driven by the three item vars `xDD4` /
//! `xDD8` / `set_sfx_var2`, the muzzle-flash shot history (six slots of effect id, flag, offset and
//! angle), the spawn functions and the three item states. Callees in other TUs go through `ext`
//! (unported: panics "TODO"); the header inlines the TU uses (`Item_InitSpawnPosition`,
//! `Item_InitSpawnCommonFields`, `Item_CopyJObjScale`, the jobj.h accessors) are ported here.
//! Checked against the C (the whole TU, shipping contraction) by tests/tu_itfoxblaster.rs.
//!
//! Float math: only subtractions and table reads; the shipping build fuses no site in this TU.
//! `efSync_Spawn` is variadic in C (`gfx_id, gobj, &pos, &angle`): the ext hook takes the four
//! arguments the TU passes, and the `(s32)` cast of its pointer result is `as usize as i32` (the
//! console pointer is 32 bits).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::helpers::it_link_lerp;
#[allow(unused_imports)]
use crate::itlib::it_80271534;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_80274d6c;
#[allow(unused_imports)]
use crate::itlib::it_80274ecc;
#[allow(unused_imports)]
use crate::itlib::it_8027572c;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_80274658;
#[allow(unused_imports)]
use crate::itlib3::it_80274d04;
#[allow(unused_imports)]
use crate::itlib3::it_802756e0;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
#[allow(unused_imports)]
use crate::itkinds::itGShell_Attrs;
#[allow(unused_imports)]
use crate::layout::ftCollisionBox;
#[allow(unused_imports)]
use crate::layout::itLinkHookshotAttributes;
#[allow(unused_imports)]
use crate::layout::itMarilAttributes;
#[allow(unused_imports)]
use crate::layout::itSScopeAttributes;
#[allow(unused_imports)]
use crate::layout::itSword_UnkArticle1;
#[allow(unused_imports)]
use crate::layout::itTaruAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruCann_DatAttrs;
#[allow(unused_imports)]
use crate::layout::itToolsAttributes;
#[allow(unused_imports)]
use crate::layout::itUnkAttributes;
#[allow(unused_imports)]
use crate::layout::itWstarAttributes;
#[allow(unused_imports)]
use crate::layout::Collide_LeftWallMask;
#[allow(unused_imports)]
use crate::layout::Collide_RightWallMask;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::HurtCapsule;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::layout::ItemLink;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::It_Kind_L_Gun_Ray;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds6;
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn efLib_DestroyAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_DestroyAll(a0) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec, a3: *mut f32) -> *mut c_void => unsafe { melee_world::tu_ext::efSync_Spawn(a0, a1, a2 as _, a3) };
    fn ftCo_800BF228(a0: *mut HSD_GObj) -> i32;
    fn ftFx_SpecialN_CheckBlasterAction(a0: *mut HSD_GObj) -> i32;
    fn ftFx_SpecialN_CheckRemoveBlaster(a0: *mut HSD_GObj) -> i32;
    fn ftFx_SpecialN_ClearBlaster(a0: *mut HSD_GObj);
    fn ftFx_SpecialN_FtGetHoldJoint(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftFx_SpecialN_GetBlasterAction(a0: *mut HSD_GObj) -> i32;
    fn ftFx_SpecialN_ItGetHoldJoint(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftKb_SpecialNFx_800FDC00(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftKb_SpecialNFx_800FDC70(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftKb_SpecialNFx_800FDD14(a0: *mut HSD_GObj) -> i32;
    fn ftKb_SpecialNFx_800FDD4C(a0: *mut HSD_GObj) -> u32;
    fn ftKb_SpecialNFx_800FDDF4(a0: *mut HSD_GObj) -> i32;
    fn ftKb_SpecialNFx_800FDEB4(a0: *mut HSD_GObj);
    fn ftLib_GetFacingDir(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftLib_GetFacingDir(a0) };
    fn ftLib_GetPos(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::ftlib::ftLib_GetPos(a0, a1) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
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

/// `HSD_JObjGetChild(jobj)`.
#[inline]
unsafe fn child_of(j: *mut HSD_JObj) -> *mut HSD_JObj {
    ji::HSD_JObjGetChild((j as *const HSD_JObj).as_ref())
}

/// `Item_InitSpawnPosition` (itkinds inlines.h).
#[inline]
pub unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *const Vec, on_plane: bool) {
    melee_core::ct!("Item_InitSpawnPosition");
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_InitSpawnCommonFields` (itkinds inlines.h).
#[inline]
pub unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    melee_core::ct!("Item_InitSpawnCommonFields");
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

/// `Item_CopyJObjScale` (itkinds inlines.h): the scale of `src`'s Y onto all three axes of `dst`.
#[inline]
pub unsafe fn Item_CopyJObjScale(dst: *mut HSD_JObj, src: *mut HSD_JObj, scale: *mut Vec) {
    melee_core::ct!("Item_CopyJObjScale");
    let s = ji::HSD_JObjGetScaleY(jref(src));
    (*scale).z = s;
    (*scale).y = s;
    (*scale).x = s;
    ji::HSD_JObjSetScale(jref(dst), &*scale, &mut Dirty);
}

/// `ItemStateTable it_803F6CA8[11]` (3F6CA8): states 0..8 share the SpecialN-follow callbacks,
/// 9 clears the references, 10 copies the owner's scale.
pub static it_803F6CA8: [ItemStateTable; 11] = [
    ItemStateTable { anim_id: 0, animated: Some(itFoxblaster_UnkMotion8_Anim), physics_updated: Some(itFoxblaster_UnkMotion8_Phys), collided: Some(itFoxblaster_UnkMotion8_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itFoxblaster_UnkMotion8_Anim), physics_updated: Some(itFoxblaster_UnkMotion8_Phys), collided: Some(itFoxblaster_UnkMotion8_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itFoxblaster_UnkMotion8_Anim), physics_updated: Some(itFoxblaster_UnkMotion8_Phys), collided: Some(itFoxblaster_UnkMotion8_Coll) },
    ItemStateTable { anim_id: 3, animated: Some(itFoxblaster_UnkMotion8_Anim), physics_updated: Some(itFoxblaster_UnkMotion8_Phys), collided: Some(itFoxblaster_UnkMotion8_Coll) },
    ItemStateTable { anim_id: 4, animated: Some(itFoxblaster_UnkMotion8_Anim), physics_updated: Some(itFoxblaster_UnkMotion8_Phys), collided: Some(itFoxblaster_UnkMotion8_Coll) },
    ItemStateTable { anim_id: 5, animated: Some(itFoxblaster_UnkMotion8_Anim), physics_updated: Some(itFoxblaster_UnkMotion8_Phys), collided: Some(itFoxblaster_UnkMotion8_Coll) },
    ItemStateTable { anim_id: 6, animated: Some(itFoxblaster_UnkMotion8_Anim), physics_updated: Some(itFoxblaster_UnkMotion8_Phys), collided: Some(itFoxblaster_UnkMotion8_Coll) },
    ItemStateTable { anim_id: 7, animated: Some(itFoxblaster_UnkMotion8_Anim), physics_updated: Some(itFoxblaster_UnkMotion8_Phys), collided: Some(itFoxblaster_UnkMotion8_Coll) },
    ItemStateTable { anim_id: 8, animated: Some(itFoxblaster_UnkMotion8_Anim), physics_updated: Some(itFoxblaster_UnkMotion8_Phys), collided: Some(itFoxblaster_UnkMotion8_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itFoxblaster_UnkMotion9_Anim), physics_updated: Some(itFoxblaster_UnkMotion9_Phys), collided: Some(itFoxblaster_UnkMotion9_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itFoxblaster_UnkMotion10_Anim), physics_updated: Some(itFoxblaster_UnkMotion10_Phys), collided: Some(itFoxblaster_UnkMotion10_Coll) },
];

/// Used for jobj->translate.z calc for xDD4.
pub static it_803F6D58: [f32; 14] = [
    0.0, -0.51, -1.02, -1.53, -1.39, -1.251, -1.112, -0.973, -0.834, -0.695, -0.556, -0.417, -0.278, -0.139,
];
/// Used in jobj->scale.y/z calcs for xDD4.
pub static it_803F6D90: [f32; 14] = [0.5, 0.5, 1.75, 3.0, 2.375, 1.75, 1.125, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5];
/// Used for jobj->rotate.x calc for xDD4.
pub static it_803F6DC8: [f32; 14] = [0.0, -42.0, -20.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
/// Used in jobj->translate.y calc for xDD8.
pub static it_803F6E28: [f32; 5] = [-0.425, -0.595, -0.765, -0.935, -0.85];
/// The length of the ItemStateTable.
pub static it_803F6E3C: [i32; 11] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// Fighter's state / ftFx_SpecialNIndex gets indexed into this table for the item state.
pub static it_803F6E68: [i32; 11] = [0, 9, 9, 3, 9, 9, 6, 7, 8, 9, 10];

/// `itFoxBlaster_PlaySFX` (static inline): the Fox or the Falco sound by item kind.
pub unsafe fn itFoxBlaster_PlaySFX(item: *mut Item, fox_sfx: u32, falco_sfx: u32) {
    melee_core::ct!("itFoxBlaster_PlaySFX");
    match (*item).kind {
        It_Kind_Kirby_FoxBlaster | It_Kind_Fox_Blaster => ext::Item_8026AE84(item, fox_sfx as i32, 0x7F, 0x40),
        It_Kind_Kirby_FalcoBlaster | It_Kind_Falco_Blaster => ext::Item_8026AE84(item, falco_sfx as i32, 0x7F, 0x40),
        _ => {}
    }
}

/// `void it_802ADDD0(Item_GObj*, s32 visibility)`: sets the sfx and the jobj hidden flags from whether
/// the blaster model is visible (and Fox vs Falco).
pub unsafe extern "C" fn it_802ADDD0(item_gobj: *mut Item_GObj, visibility: i32) {
    melee_core::ct!("it_802ADDD0");
    if item_gobj.is_null() {
        return;
    }
    let item = item_of(item_gobj);
    if item.is_null() {
        return;
    }
    let item_jobj = jobj_of(item_gobj);
    let child_jobj = child_of(item_jobj);
    if (*item).xDD4_itemVar.foxblaster.set_sfx_var2 != visibility {
        (*item).xDD4_itemVar.foxblaster.set_sfx_var2 = visibility;
        if visibility == 2 {
            itFoxBlaster_PlaySFX(item, 0x1AE14, 0x18700);
        }
    }
    match (*item).xDD4_itemVar.foxblaster.set_sfx_var2 {
        0 | 2 => {
            ext::HSD_JObjSetFlagsAll(child_jobj, JOBJ_HIDDEN as u32);
        }
        1 => {
            ext::HSD_JObjClearFlagsAll(child_jobj, JOBJ_HIDDEN as u32);
        }
        _ => {}
    }
}

/// `void it_802ADEF0(HSD_GObj*)`: redirects to `it_802ADF10`.
pub unsafe extern "C" fn it_802ADEF0(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802ADEF0");
    it_802ADF10(item_gobj);
}

/// `itFoxBlaster_SetShotOffset` (static inline).
pub unsafe fn itFoxBlaster_SetShotOffset(item: *mut Item, fighter_pos: *mut Vec, fighter_muzzle_pos: *mut Vec) {
    melee_core::ct!("itFoxBlaster_SetShotOffset");
    let fb = &mut (*item).xDD4_itemVar.foxblaster;
    fb.xE14[0].x = (*fighter_muzzle_pos).x - (*fighter_pos).x;
    fb.xE14[0].y = (*fighter_muzzle_pos).y - (*fighter_pos).y;
    fb.xE14[0].z = (*fighter_muzzle_pos).z - (*fighter_pos).z;
}

/// `itFoxBlaster_SetShotAngle` (static inline).
pub unsafe fn itFoxBlaster_SetShotAngle(item: *mut Item, fighter_muzzle_pos: *mut Vec, item_muzzle_pos: *mut Vec) {
    melee_core::ct!("itFoxBlaster_SetShotAngle");
    let fb = &mut (*item).xDD4_itemVar.foxblaster;
    fb.angle[0] = ext::atan2f(
        (*fighter_muzzle_pos).y - (*item_muzzle_pos).y,
        (*fighter_muzzle_pos).x - (*item_muzzle_pos).x,
    );
    fb.xDFC[0] = 0;
}

/// `void it_802ADF10(HSD_GObj*)`: if the blaster fired this frame, spawns the muzzle effect at the hold
/// joints and shifts the six-slot shot history up, clearing the newest slot.
pub unsafe extern "C" fn it_802ADF10(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802ADF10");
    let mut ft_cur_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut ft_hold_joint_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut it_hold_joint_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut item = item_of(item_gobj);

    if (*item).owner == (*item).xDD4_itemVar.foxblaster.owner && (*item).xDD4_itemVar.foxblaster.gfx_spawn_var != 0 {
        (*item).xDD4_itemVar.foxblaster.gfx_spawn_var = 0;
        ext::ftLib_GetPos((*item).xDD4_itemVar.foxblaster.owner, &mut ft_cur_pos);

        match (*item).kind {
            It_Kind_Fox_Blaster | It_Kind_Falco_Blaster => {
                ext::ftFx_SpecialN_FtGetHoldJoint((*item).xDD4_itemVar.foxblaster.owner, &mut ft_hold_joint_pos);
                itFoxBlaster_SetShotOffset(item, &mut ft_cur_pos, &mut ft_hold_joint_pos);
                ext::ftFx_SpecialN_ItGetHoldJoint((*item).xDD4_itemVar.foxblaster.owner, &mut it_hold_joint_pos);
                itFoxBlaster_SetShotAngle(item, &mut ft_hold_joint_pos, &mut it_hold_joint_pos);
                (*item).xDD4_itemVar.foxblaster.xDE4[0] = ext::efSync_Spawn(
                    1166,
                    item_gobj,
                    &mut ft_hold_joint_pos,
                    &raw mut (*item).xDD4_itemVar.foxblaster.angle[0],
                ) as usize as i32;
            }
            It_Kind_Kirby_FoxBlaster | It_Kind_Kirby_FalcoBlaster => {
                ext::ftKb_SpecialNFx_800FDC00((*item).xDD4_itemVar.foxblaster.owner, &mut ft_hold_joint_pos);
                itFoxBlaster_SetShotOffset(item, &mut ft_cur_pos, &mut ft_hold_joint_pos);
                ext::ftKb_SpecialNFx_800FDC70((*item).xDD4_itemVar.foxblaster.owner, &mut it_hold_joint_pos);
                itFoxBlaster_SetShotAngle(item, &mut ft_hold_joint_pos, &mut it_hold_joint_pos);
                (*item).xDD4_itemVar.foxblaster.xDE4[0] = ext::efSync_Spawn(
                    1196,
                    item_gobj,
                    &mut ft_hold_joint_pos,
                    &raw mut (*item).xDD4_itemVar.foxblaster.angle[0],
                ) as usize as i32;
            }
            _ => {}
        }

        let mut i = 5usize;
        while i > 0 {
            let fb = &mut (*item).xDD4_itemVar.foxblaster;
            fb.xDE4[i] = fb.xDE4[i - 1];
            fb.xDFC[i] = fb.xDFC[i - 1];
            fb.xE14[i] = fb.xE14[i - 1];
            fb.angle[i] = fb.angle[i - 1];
            i -= 1;
        }

        item = item_of(item_gobj);
        let fb = &mut (*item).xDD4_itemVar.foxblaster;
        fb.xDE4[0] = 0;
        fb.xDFC[0] = 0;
        fb.xE14[0].z = 0.0;
        fb.xE14[0].y = 0.0;
        fb.xE14[0].x = 0.0;
        fb.angle[0] = 0.0;
    }
}

/// `void it_802AE1D0(Item_GObj*)`: starts the model-transform countdown and installs the accessory callback.
pub unsafe extern "C" fn it_802AE1D0(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802AE1D0");
    let item = item_of(item_gobj);
    if !item_gobj.is_null() && !item.is_null() {
        (*item).xDD4_itemVar.foxblaster.xDD4 = 1;
        (*item).on_accessory = Some(it_802ADEF0);
        (*item).xDD4_itemVar.foxblaster.gfx_spawn_var = 1;
    }
}

/// `void it_802AE200(Item_GObj*)`: transforms the fifth-level jobj by `xDD4` and advances the countdown.
pub unsafe extern "C" fn it_802AE200(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802AE200");
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let item = (*item_gobj).user_data as *mut Item;
    let child_jobj_1 = if item_jobj.is_null() { null_mut() } else { (*item_jobj).child };
    let child_jobj_2 = if child_jobj_1.is_null() { null_mut() } else { (*child_jobj_1).child };
    let child_jobj_3 = if child_jobj_2.is_null() { null_mut() } else { (*child_jobj_2).child };
    let child_jobj_4 = if child_jobj_3.is_null() { null_mut() } else { (*child_jobj_3).child };
    let jobj_1 = if child_jobj_4.is_null() { null_mut() } else { (*child_jobj_4).next };
    if !item_gobj.is_null() && !item.is_null() && !jobj_1.is_null() {
        let k = (*item).xDD4_itemVar.foxblaster.xDD4 as usize;
        ji::HSD_JObjSetTranslateZ(jref(jobj_1), it_803F6D58[k], &mut Dirty);

        let jobj_2 = if jobj_1.is_null() { null_mut() } else { (*jobj_1).next };
        let k = (*item).xDD4_itemVar.foxblaster.xDD4 as usize;
        ji::HSD_JObjSetScaleY(jref(jobj_2), it_803F6D90[k], &mut Dirty);
        let k = (*item).xDD4_itemVar.foxblaster.xDD4 as usize;
        ji::HSD_JObjSetScaleZ(jref(jobj_2), it_803F6D90[k], &mut Dirty);

        let jobj_3 = if jobj_2.is_null() { null_mut() } else { (*jobj_2).next };
        let k = (*item).xDD4_itemVar.foxblaster.xDD4 as usize;
        ji::HSD_JObjSetRotationX(jref(jobj_3), it_803F6DC8[k], &mut Dirty);

        if (*item).xDD4_itemVar.foxblaster.xDD4 > 0 && (*item).xDD4_itemVar.foxblaster.xDD4 < 14 {
            (*item).xDD4_itemVar.foxblaster.xDD4 += 1;
            if (*item).xDD4_itemVar.foxblaster.xDD4 >= 14 {
                (*item).xDD4_itemVar.foxblaster.xDD4 = 0;
            }
        }
    }
}

/// `void it_802AE538(Item_GObj*)`: gun-out start (cmd_var3 = 1): sets the draw-out vars and plays the sound once.
pub unsafe extern "C" fn it_802AE538(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802AE538");
    let item = (*item_gobj).user_data as *mut Item;
    if (*item).xDD4_itemVar.foxblaster.xDD8 != 4 && !item_gobj.is_null() && !item.is_null() {
        (*item).xDD4_itemVar.foxblaster.xDD8 = 1;
        (*item).xDD4_itemVar.foxblaster.xDDC = 1;
        if (*item).xDD4_itemVar.foxblaster.xDE0 == 0 {
            itFoxBlaster_PlaySFX(item, 0x1AE05, 0x186F1);
            (*item).xDD4_itemVar.foxblaster.xDE0 = 1;
        }
    }
}

/// `void it_802AE608(Item_GObj*)`: gun put away (cmd_var3 = 2).
pub unsafe extern "C" fn it_802AE608(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802AE608");
    let item = (*item_gobj).user_data as *mut Item;
    if (*item).xDD4_itemVar.foxblaster.xDD8 != 0 && !item_gobj.is_null() && !item.is_null() {
        (*item).xDD4_itemVar.foxblaster.xDD8 = 3;
        (*item).xDD4_itemVar.foxblaster.xDDC = -1;
    }
}

/// `void it_802AE63C(Item_GObj*)`: slides the fourth-level jobj's Y by `xDD8` and steps `xDD8`.
pub unsafe extern "C" fn it_802AE63C(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802AE63C");
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let item = (*item_gobj).user_data as *mut Item;
    let child_jobj_1 = if item_jobj.is_null() { null_mut() } else { (*item_jobj).child };
    let child_jobj_2 = if child_jobj_1.is_null() { null_mut() } else { (*child_jobj_1).child };
    let child_jobj_3 = if child_jobj_2.is_null() { null_mut() } else { (*child_jobj_2).child };
    let target_jobj = if child_jobj_3.is_null() { null_mut() } else { (*child_jobj_3).child };
    if !item_gobj.is_null() && !item.is_null() && !target_jobj.is_null() {
        let k = (*item).xDD4_itemVar.foxblaster.xDD8 as usize;
        ji::HSD_JObjSetTranslateY(jref(target_jobj), it_803F6E28[k], &mut Dirty);

        if (*item).xDD4_itemVar.foxblaster.xDD8 > 0 && (*item).xDD4_itemVar.foxblaster.xDD8 < 4 {
            (*item).xDD4_itemVar.foxblaster.xDD8 += (*item).xDD4_itemVar.foxblaster.xDDC;
            if (*item).xDD4_itemVar.foxblaster.xDD8 >= 5 {
                (*item).xDD4_itemVar.foxblaster.xDD8 = 4;
                (*item).xDD4_itemVar.foxblaster.xDDC = 0;
                return;
            }
            if (*item).xDD4_itemVar.foxblaster.xDD8 <= 0 {
                (*item).xDD4_itemVar.foxblaster.xDD8 = 0;
                (*item).xDD4_itemVar.foxblaster.xDDC = 0;
            }
        }
    }
}

/// `itFoxBlaster_SetCommandVars` (static inline): all four command vars.
pub unsafe fn itFoxBlaster_SetCommandVars(item: *mut Item, value: i32) {
    melee_core::ct!("itFoxBlaster_SetCommandVars");
    (*item).xDB8_itcmd_var3 = value as u32;
    (*item).xDB4_itcmd_var2 = value as u32;
    (*item).xDB0_itcmd_var1 = value as u32;
    (*item).xDAC_itcmd_var0 = value as u32;
}

/// `itFoxBlaster_ClearShot` (static inline): zeroes shot slot `index`.
pub unsafe fn itFoxBlaster_ClearShot(item_gobj: *mut Item_GObj, index: i32) {
    melee_core::ct!("itFoxBlaster_ClearShot");
    let item = item_of(item_gobj);
    let i = index as usize;
    (*item).xDD4_itemVar.foxblaster.xDE4[i] = 0;
    (*item).xDD4_itemVar.foxblaster.xDFC[i] = 0;
    (*item).xDD4_itemVar.foxblaster.xE14[i].z = 0.0;
    (*item).xDD4_itemVar.foxblaster.xE14[i].y = 0.0;
    (*item).xDD4_itemVar.foxblaster.xE14[i].x = 0.0;
    (*item).xDD4_itemVar.foxblaster.angle[i] = 0.0;
}

/// `void it_802AE7B8(Item_GObj*)`: clears every blaster var and command var.
pub unsafe extern "C" fn it_802AE7B8(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802AE7B8");
    let item = item_of(item_gobj);
    itFoxBlaster_SetCommandVars(item, 0);
    (*item).xDCC_flag.set_b3(0);
    (*item).xDD4_itemVar.foxblaster.xDD4 = 0;
    (*item).xDD4_itemVar.foxblaster.xDD8 = 0;
    (*item).xDD4_itemVar.foxblaster.xDDC = 0;
    (*item).xDD4_itemVar.foxblaster.xDE0 = 0;
    (*item).xDD4_itemVar.foxblaster.gfx_spawn_var = 0;
    (*item).xDD4_itemVar.foxblaster.set_sfx_var2 = 1;

    itFoxBlaster_ClearShot(item_gobj, 0);
    itFoxBlaster_ClearShot(item_gobj, 1);
    itFoxBlaster_ClearShot(item_gobj, 2);
    itFoxBlaster_ClearShot(item_gobj, 3);
    itFoxBlaster_ClearShot(item_gobj, 4);
    itFoxBlaster_ClearShot(item_gobj, 5);
}

/// `Item_GObj* it_802AE8A8(f32 facing_dir, Fighter_GObj*, Vec3*, Fighter_Part, ItemKind)`: spawns the
/// blaster at `arg2` and gives it to the fighter at `ft_part`.
pub unsafe extern "C" fn it_802AE8A8(
    facing_dir: f32,
    fighter_gobj: *mut Fighter_GObj,
    arg2: *mut Vec,
    ft_part: Fighter_Part,
    it_kind: ItemKind,
) -> *mut Item_GObj {
    melee_core::ct!("it_802AE8A8");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let item_gobj: *mut Item_GObj;
    if !fighter_gobj.is_null() {
        spawn.kind = it_kind;
        Item_InitSpawnPosition(&mut spawn, arg2, true);
        Item_InitSpawnCommonFields(&mut spawn, fighter_gobj, facing_dir, false);
        item_gobj = ext::Item_80268B18(&mut spawn);
        if !item_gobj.is_null() {
            let item = (*item_gobj).user_data as *mut Item;
            (*item).xDD4_itemVar.foxblaster.owner = fighter_gobj;
            it_802AE7B8(item_gobj);
            ext::Item_8026AB54(item_gobj, fighter_gobj, ft_part);
        }
    } else {
        item_gobj = null_mut();
    }
    item_gobj
}

/// `Item_GObj* it_802AE994(Fighter_GObj*, Fighter_Part, ItemKind)`: spawns the blaster at the fighter and
/// starts the command vars at 1.
pub unsafe extern "C" fn it_802AE994(owner_gobj: *mut Fighter_GObj, ft_part: Fighter_Part, it_kind: ItemKind) -> *mut Item_GObj {
    melee_core::ct!("it_802AE994");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut sp18 = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let item_gobj: *mut Item_GObj;
    if !owner_gobj.is_null() {
        spawn.kind = it_kind;
        ext::ftLib_GetPos(owner_gobj, &mut sp18);
        Item_InitSpawnPosition(&mut spawn, &sp18, true);
        let dir = ext::ftLib_GetFacingDir(owner_gobj);
        Item_InitSpawnCommonFields(&mut spawn, owner_gobj, dir, false);
        item_gobj = ext::Item_80268B18(&mut spawn);
        if !item_gobj.is_null() {
            let item = item_of(item_gobj);
            (*item).xDD4_itemVar.foxblaster.owner = owner_gobj;
            it_802AE7B8(item_gobj);
            itFoxBlaster_SetCommandVars(item, 1);
            ext::Item_8026AB54(item_gobj, owner_gobj, ft_part);
        }
    } else {
        item_gobj = null_mut();
    }
    item_gobj
}

/// `void it_802AEAB4(Item_GObj*)`: drops the owner links, kills the effects and destroys the item.
pub unsafe extern "C" fn it_802AEAB4(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802AEAB4");
    if !item_gobj.is_null() {
        let item = (*item_gobj).user_data as *mut Item;
        (*item).owner = null_mut();
        (*item).xDD4_itemVar.foxblaster.owner = null_mut();
        ext::efLib_DestroyAll(item_gobj);
        ext::Item_8026A8EC(item_gobj);
    }
}

/// `void itFoxBlaster_Logic96_PickedUp(Item_GObj*)`: picks the item state from the fighter's SpecialN action
/// (or state 10 while a command var is set), then advances the animation and script.
pub unsafe extern "C" fn itFoxBlaster_Logic96_PickedUp(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itFoxBlaster_Logic96_PickedUp");
    let mut ft_special_state: i32 = 9;
    let item = (*item_gobj).user_data as *mut Item;
    if ((*item).xDB8_itcmd_var3 | ((*item).xDB4_itcmd_var2 | ((*item).xDAC_itcmd_var0 | (*item).xDB0_itcmd_var1))) != 0 {
        ext::Item_80268E5C(item_gobj, 0xA, ITEM_ANIM_UPDATE);
    } else {
        match (*item).kind {
            It_Kind_Fox_Blaster | It_Kind_Falco_Blaster => {
                ft_special_state = ext::ftFx_SpecialN_GetBlasterAction((*item).xDD4_itemVar.foxblaster.owner);
            }
            It_Kind_Kirby_FoxBlaster | It_Kind_Kirby_FalcoBlaster => {
                ft_special_state = ext::ftKb_SpecialNFx_800FDD4C((*item).xDD4_itemVar.foxblaster.owner) as i32;
            }
            _ => {}
        }
        ext::Item_80268E5C(item_gobj, it_803F6E68[ft_special_state as usize], ITEM_ANIM_UPDATE);
    }
    ext::Item_802694CC(item_gobj);
}

/// `clear_blaster_references` (static inline): clears the references between the blaster item and the fighter.
pub unsafe fn clear_blaster_references(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("clear_blaster_references");
    let item = (*item_gobj).user_data as *mut Item;
    if !item_gobj.is_null() && !item.is_null() {
        if (*item).owner == (*item).xDD4_itemVar.foxblaster.owner {
            match (*item).kind {
                It_Kind_Fox_Blaster | It_Kind_Falco_Blaster => {
                    ext::ftFx_SpecialN_ClearBlaster((*item).xDD4_itemVar.foxblaster.owner);
                }
                It_Kind_Kirby_FoxBlaster | It_Kind_Kirby_FalcoBlaster => {
                    ext::ftKb_SpecialNFx_800FDEB4((*item).xDD4_itemVar.foxblaster.owner);
                }
                _ => {}
            }
        }
        (*item).owner = null_mut();
        (*item).xDD4_itemVar.foxblaster.owner = null_mut();
        ext::efLib_DestroyAll(item_gobj);
    }
}

/// `clear_blaster` (static inline): clears the references (as above) and destroys the item.
pub unsafe fn clear_blaster(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("clear_blaster");
    let mut item = (*item_gobj).user_data as *mut Item;
    if !item_gobj.is_null() && !item.is_null() {
        if (*item).owner == (*item).xDD4_itemVar.foxblaster.owner {
            match (*item).kind {
                It_Kind_Fox_Blaster | It_Kind_Falco_Blaster => {
                    ext::ftFx_SpecialN_ClearBlaster((*item).xDD4_itemVar.foxblaster.owner);
                }
                It_Kind_Kirby_FoxBlaster | It_Kind_Kirby_FalcoBlaster => {
                    ext::ftKb_SpecialNFx_800FDEB4((*item).xDD4_itemVar.foxblaster.owner);
                }
                _ => {}
            }
        }
        (*item).owner = null_mut();
        (*item).xDD4_itemVar.foxblaster.owner = null_mut();
        ext::efLib_DestroyAll(item_gobj);
    }
    if !item_gobj.is_null() {
        item = (*item_gobj).user_data as *mut Item;
        (*item).owner = null_mut();
        (*item).xDD4_itemVar.foxblaster.owner = null_mut();
        ext::efLib_DestroyAll(item_gobj);
        ext::Item_8026A8EC(item_gobj);
    }
}

/// `bool itFoxblaster_UnkMotion8_Anim(HSD_GObj*)`: follows the fighter's SpecialN action (changes the item
/// state to match), and removes the blaster when the fighter no longer needs it.
pub unsafe extern "C" fn itFoxblaster_UnkMotion8_Anim(item_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itFoxblaster_UnkMotion8_Anim");
    let item = item_of(item_gobj);

    match (*item).msid {
        2 | 5 | 6 | 7 | 8 => {}
        _ => {
            let mut ft_special_state: i32 = 9;
            match (*item).kind {
                It_Kind_Fox_Blaster | It_Kind_Falco_Blaster => {
                    ft_special_state = ext::ftFx_SpecialN_GetBlasterAction((*item).xDD4_itemVar.foxblaster.owner);
                }
                It_Kind_Kirby_FoxBlaster | It_Kind_Kirby_FalcoBlaster => {
                    ft_special_state = ext::ftKb_SpecialNFx_800FDD4C((*item).xDD4_itemVar.foxblaster.owner) as i32;
                }
                _ => {}
            }
            if (*item).msid != it_803F6E3C[ft_special_state as usize] {
                ext::Item_80268E5C(item_gobj, ft_special_state, ITEM_ANIM_UPDATE);
                ext::Item_802694CC(item_gobj);
            }
        }
    }

    if !(*item).xDD4_itemVar.foxblaster.owner.is_null() {
        let mut blaster_action: i32 = 9;
        let mut blaster_removed: i32 = 1;
        match (*item).kind {
            It_Kind_Fox_Blaster | It_Kind_Falco_Blaster => {
                blaster_action = ext::ftFx_SpecialN_GetBlasterAction((*item).xDD4_itemVar.foxblaster.owner);
                blaster_removed = ext::ftFx_SpecialN_CheckRemoveBlaster((*item).xDD4_itemVar.foxblaster.owner);
            }
            It_Kind_Kirby_FoxBlaster | It_Kind_Kirby_FalcoBlaster => {
                blaster_action = ext::ftKb_SpecialNFx_800FDD4C((*item).xDD4_itemVar.foxblaster.owner) as i32;
                blaster_removed = ext::ftKb_SpecialNFx_800FDD14((*item).xDD4_itemVar.foxblaster.owner);
            }
            _ => {}
        }
        if blaster_action == 9 || blaster_removed == 1 {
            clear_blaster(item_gobj);
        } else {
            let blaster_not_needed: i32 = match (*item).kind {
                It_Kind_Fox_Blaster | It_Kind_Falco_Blaster => {
                    ext::ftFx_SpecialN_CheckBlasterAction((*item).xDD4_itemVar.foxblaster.owner)
                }
                It_Kind_Kirby_FoxBlaster | It_Kind_Kirby_FalcoBlaster => {
                    ext::ftKb_SpecialNFx_800FDDF4((*item).xDD4_itemVar.foxblaster.owner)
                }
                _ => 1,
            };
            if blaster_not_needed != 0 {
                (*item).xDD4_itemVar.foxblaster.owner = null_mut();
                ext::Item_8026A8EC(item_gobj);
                ext::efLib_DestroyAll(item_gobj);
            }
        }
    } else {
        clear_blaster(item_gobj);
    }
    0
}

/// `void itFoxblaster_UnkMotion8_Phys(HSD_GObj*)`: the two model transforms.
pub unsafe extern "C" fn itFoxblaster_UnkMotion8_Phys(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("itFoxblaster_UnkMotion8_Phys");
    it_802AE63C(item_gobj);
    it_802AE200(item_gobj);
}

/// `bool itFoxblaster_UnkMotion8_Coll(HSD_GObj*)`: does nothing.
pub unsafe extern "C" fn itFoxblaster_UnkMotion8_Coll(item_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itFoxblaster_UnkMotion8_Coll");
    0
}

/// `bool itFoxblaster_UnkMotion9_Anim(HSD_GObj*)`: clears the references and ends the item.
pub unsafe extern "C" fn itFoxblaster_UnkMotion9_Anim(item_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itFoxblaster_UnkMotion9_Anim");
    ext::efLib_DestroyAll(item_gobj);
    clear_blaster_references(item_gobj);
    1
}

/// `void itFoxblaster_UnkMotion9_Phys(HSD_GObj*)`: does nothing.
pub unsafe extern "C" fn itFoxblaster_UnkMotion9_Phys(item_gobj: *mut HSD_GObj) { melee_core::ct!("itFoxblaster_UnkMotion9_Phys");}

/// `bool itFoxblaster_UnkMotion9_Coll(HSD_GObj*)`: identical to the Anim callback.
pub unsafe extern "C" fn itFoxblaster_UnkMotion9_Coll(item_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itFoxblaster_UnkMotion9_Coll");
    ext::efLib_DestroyAll(item_gobj);
    clear_blaster_references(item_gobj);
    1
}

/// `bool itFoxblaster_UnkMotion10_Anim(HSD_GObj*)`: copies the owner's scale onto the item when the owner says so.
pub unsafe extern "C" fn itFoxblaster_UnkMotion10_Anim(item_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itFoxblaster_UnkMotion10_Anim");
    if !item_gobj.is_null() {
        let it = item_of(item_gobj);
        if !it.is_null() && !(*it).xDD4_itemVar.foxblaster.owner.is_null() {
            if ext::ftCo_800BF228((*it).xDD4_itemVar.foxblaster.owner) == 1 {
                let mut scale = Vec { x: 0.0, y: 0.0, z: 0.0 };
                let owner = (*it).xDD4_itemVar.foxblaster.owner;
                Item_CopyJObjScale(jobj_of(item_gobj), jobj_of(owner), &mut scale);
            }
        }
    }
    0
}

/// `void itFoxblaster_UnkMotion10_Phys(HSD_GObj*)`: does nothing.
pub unsafe extern "C" fn itFoxblaster_UnkMotion10_Phys(item_gobj: *mut HSD_GObj) { melee_core::ct!("itFoxblaster_UnkMotion10_Phys");}

/// `bool itFoxblaster_UnkMotion10_Coll(HSD_GObj*)`: does nothing.
pub unsafe extern "C" fn itFoxblaster_UnkMotion10_Coll(item_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itFoxblaster_UnkMotion10_Coll");
    0
}

/// `void itFoxBlaster_Logic96_EvtUnk(Item_GObj*, HSD_GObj*)`: removes the gobj interaction references.
pub unsafe extern "C" fn itFoxBlaster_Logic96_EvtUnk(item_gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("itFoxBlaster_Logic96_EvtUnk");
    ext::it_8026B894(item_gobj, ref_gobj);
}

// ---- folded from itkinds6.rs (consolidation steps 21/22) ----
/// `it_802AE608` (itfoxblaster.c): if `xDD8 != 0`, `xDD8 = 3; xDDC = -1`.
pub fn it_802ae608(ip: &mut Item) {
    melee_core::ct!("it_802ae608");
    // SAFETY: item-var union fields are plain data.
    unsafe {
        let fb = &mut ip.xDD4_itemVar.foxblaster;
        if fb.xDD8 != 0 {
            fb.xDD8 = 3;
            fb.xDDC = -1;
        }
    }
}
